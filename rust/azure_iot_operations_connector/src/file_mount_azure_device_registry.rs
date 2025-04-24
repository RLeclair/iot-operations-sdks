// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Utilities for interacting with the file mount of Azure Device Registry.

use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::task::Context;
use std::time::Duration;

use notify::{
    RecommendedWatcher,
    event::{self, EventKind},
};
use notify_debouncer_full::{RecommendedCache, new_debouncer};
use thiserror::Error;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

/// Environment variable name for the mount path of the Azure Device Registry resources.
const ADR_RESOURCES_NAME_MOUNT_PATH: &str = "ADR_RESOURCES_NAME_MOUNT_PATH";

/// Represents an error that occured while interacting with the file mount of Azure Device Registry.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(#[from] ErrorKind);

/// Represents the kinds of errors that may occur while interacting with the file mount of Azure Device Registry.
#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ErrorKind {
    /// An error occurred when parsing the environment variable for the mount path.
    #[error("{0}")]
    EnvironmentVariableError(#[from] env::VarError),

    /// An error occured while accessing the file mount.
    #[error("{0}")]
    IoError(#[from] std::io::Error),

    /// An error occurred while creating the file mount watcher.
    #[error("{0}")]
    WatcherError(#[from] notify::Error),

    /// An error occurred while parsing the files in the file mount.
    #[error("{0}")]
    ParseError(String),
}

// ~~~~~~~~~~~~~~~~~ Helper Functions and Structs ~~~~~~~~~~~~~~~~~~~~~

/// Gets the mount path of the Azure Device Registry resources.
///
/// Returns Ok([`PathBuf`]) if the mount path is successfully retrieved, otherwise returns an [`Error`].
///
/// # Errors
///
/// - [`Error`] of kind [`ErrorKind::EnvironmentVariableError`] if the environment variable was not able to be read.
pub fn get_mount_path() -> Result<PathBuf, Error> {
    Ok(env::var(ADR_RESOURCES_NAME_MOUNT_PATH)
        .map_err(ErrorKind::from)?
        .into())
}

/// Get names of all available device endpoints from the file mount.
///
/// Returns Ok([`Vec<DeviceEndpointRef>`]) if the device endpoint names are successfully retrieved, otherwise returns an [`Error`].
///
/// # Errors
///
/// - [`Error`] of kind [`ErrorKind::IoError`] if there are issues accessing the file mount.
/// - [`Error`] of kind [`ErrorKind::ParseError`] if the file names cannot be parsed into [`DeviceEndpointRef`].
pub fn get_device_endpoint_names(mount_path: &Path) -> Result<Vec<DeviceEndpointRef>, Error> {
    // TODO: There is probably a way to do this without needing the below for loop.
    let mut device_endpoint_refs = Vec::new();
    for entry in std::fs::read_dir(mount_path).map_err(ErrorKind::from)? {
        match entry.map_err(ErrorKind::from)?.path().file_name() {
            Some(file_name) => {
                // TODO: Handle case where file name has invalid UTF-8 characters (remove need for to_string_lossy)
                // TODO: Handle case where file name is not a file but a directory
                let device_endpoint_string = file_name.to_string_lossy().to_string();
                device_endpoint_refs.push(device_endpoint_string.try_into()?);
            }
            None => {
                // TODO: Happens when the path ends with "..", skip it and log a warning for now
                log::warn!(
                    "Failed to get file name from device endpoint directory, path ends in .."
                );
            }
        }
    }
    Ok(device_endpoint_refs)
}

/// Gets the names of all assets associated with a [`DeviceEndpointRef`] from the file mount.
///
/// Returns Ok([`Vec<AssetRef>`]) if the asset names are successfully retrieved, otherwise returns an [`Error`].
///
/// # Errors
///
/// - [`Error`] of kind [`ErrorKind::IoError`] if there are issues accessing the file mount.
/// - [`Error`] of kind [`ErrorKind::ParseError`] if the device endpoint's file content cannot be parsed.
fn get_asset_names(
    mount_path: &Path,
    device_endpoint: &DeviceEndpointRef,
) -> Result<Vec<AssetRef>, Error> {
    // Create the file path for the device endpoint
    let file_path = mount_path.join(device_endpoint.to_string());

    // Get the content of the file
    let file_content = String::from_utf8(std::fs::read(file_path).map_err(ErrorKind::from)?)
        .map_err(|e| ErrorKind::ParseError(e.to_string()))?;

    // If the file is empty, return an empty vector
    if file_content.is_empty() {
        return Ok(vec![]);
    }

    // Split the file content by ';' and create a vector of AssetRef
    Ok(file_content
        .split(';')
        .map(|asset_name| AssetRef {
            name: asset_name.to_string(),
            device_name: device_endpoint.device_name.clone(),
            endpoint_name: device_endpoint.endpoint_name.clone(),
        })
        .collect())
}

/// A map that tracks device endpoints and their associated assets.
///
/// This struct contains a map of device endpoints to their associated assets and a channel for
/// sending notifications about device endpoint creation.
///
/// Each device endpoint is associated with a tuple containing an unbounded sender for asset creation
/// notifications and a hash map of asset references to their associated deletion tokens.
struct FileMountMap {
    file_mount_map: HashMap<
        DeviceEndpointRef,
        (
            UnboundedSender<(AssetRef, AssetDeletionToken)>,
            HashMap<AssetRef, oneshot::Sender<()>>,
        ),
    >,
    create_device_tx: UnboundedSender<(DeviceEndpointRef, AssetCreateObservation)>,
}

impl FileMountMap {
    /// Creates a new instance of [`FileMountMap`].
    pub fn new(
        create_device_tx: UnboundedSender<(DeviceEndpointRef, AssetCreateObservation)>,
    ) -> FileMountMap {
        FileMountMap {
            file_mount_map: HashMap::new(),
            create_device_tx,
        }
    }

    /// Inserts a new [`DeviceEndpointRef`] into the file mount map.
    /// 
    /// This function creates a new channel for asset creation notifications and adds the device endpoint
    /// to the file mount map. If the device endpoint already exists, it does nothing. It will also
    /// notify the `create_device_tx` channel about the new device endpoint.
    pub fn insert_device_endpoint(&mut self, device: &DeviceEndpointRef) {
        // Check if the device already exists already
        if self.file_mount_map.contains_key(device) {
            return;
        }

        // Create a new channel for asset creation notifications for this device
        let (asset_creation_tx, asset_creation_rx) = mpsc::unbounded_channel();

        // Create a new entry in the file mount map for the device
        self.file_mount_map
            .insert(device.clone(), (asset_creation_tx, HashMap::new()));

        // Notify on device creation
        if self
            .create_device_tx
            .send((
                device.clone(),
                AssetCreateObservation::new(asset_creation_rx),
            ))
            .is_err()
        {
            // TODO: The cases in which this fails are:
            // 1. The receiver is closed which means the `FileMountMap` is dropped, this should not happen.
            // 2. The receiver is full which means we are not receiving notifications fast enough or
            //    are out of space, this should be handled by the caller via a retry.
            log::warn!("Failed to send device creation notification");
            panic!("Failed to send device creation notification");
        }
    }

    /// Updates the assets associated with a device endpoint.
    /// 
    /// This function takes a device endpoint and a vector of assets, and updates the file mount map
    /// with the new assets. It also cleans up any assets that are no longer present. If the device
    /// endpoint does not exist in the file mount map, it does nothing.
    pub fn update_assets(&mut self, device: &DeviceEndpointRef, assets: Vec<AssetRef>) {
        // Get the asset creation channel and the tracked assets for this device
        let (create_asset_tx, tracked_assets) = match self.file_mount_map.get_mut(device) {
            Some((create_asset_tx, tracked_assets)) => (create_asset_tx, tracked_assets),
            None => {
                // If the device is non-existent we can't update the assets. Most likely a create
                // notification has not been parsed yet but this function will be called again once
                // the device is created.
                return;
            }
        };

        // Clean up the assets that are no longer in the file mount
        let asset_set: HashSet<_> = assets.iter().collect();
        // When an asset is not retained it gets dropped and its deletion token is triggered when
        // the channel associated with it is dropped.
        tracked_assets.retain(|tracked_asset, _| asset_set.contains(tracked_asset));

        // Iterate over the assets and check if they are already tracked, if not, add them
        assets.iter().for_each(|asset| {
            if !tracked_assets.contains_key(asset) {
                // Create a one shot channel for asset deletion
                let (asset_deletion_tx, asset_deletion_rx) = oneshot::channel();

                let asset_deletion_token = AssetDeletionToken(asset_deletion_rx);

                // Add the new asset to the tracked assets with the one shot sender so when the asset is
                // deleted the channel is closed and the receiver is notified.
                tracked_assets.insert(asset.clone(), asset_deletion_tx);

                // Notify that an asset has been created
                if create_asset_tx
                    .send((asset.clone(), asset_deletion_token))
                    .is_err()
                {
                    // TODO: The cases in which this fails are:
                    // 1. The receiver is closed which means the `FileMountMap` is dropped, this should not happen.
                    // 2. The receiver is full which means we are not receiving notifications fast enough or
                    //    are out of space, this should be handled by the caller via a retry.
                    log::warn!("Failed to send device creation notification");
                    panic!("Failed to send device creation notification");
                }
            }
        });
    }

    /// Removes a device endpoint from the file mount map.
    /// 
    /// When a device endpoint is removed, all the assets associated with it are also removed which 
    /// triggers the deletion token for each asset.
    pub fn remove_device_endpoint(&mut self, device: &DeviceEndpointRef) {
        // Remove entry from the file mount map
        self.file_mount_map.remove(device);
    }
}

// ~~~~~~~~~~~~~~~~~ Device and Asset References ~~~~~~~~~~~~~~~~~~~~~

/// Represents a device and its associated endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceEndpointRef {
    /// The name of the device
    pub device_name: String,
    /// The name of the endpoint
    pub endpoint_name: String,
}

impl TryFrom<String> for DeviceEndpointRef {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // The below assumes the format is always {device_name}_{endpoint_name} with no additional
        // `_` in the names.
        // TODO: Add a warning in case the format is not as expected
        match value.split_once('_') {
            Some((device_name, endpoint_name)) => {
                if endpoint_name.contains('_') {
                    log::warn!("DeviceEndpointRef contains an underscore in the endpoint name");
                }
                Ok(Self {
                    device_name: device_name.to_string(),
                    endpoint_name: endpoint_name.to_string(),
                })
            }
            None => Err(Error(ErrorKind::ParseError(
                "Failed to parse DeviceEndpointRef from string".to_string(),
            ))),
        }
    }
}

impl TryFrom<&PathBuf> for DeviceEndpointRef {
    type Error = Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        // TODO: Handle case where file name is not a file but a directory
        // TODO: Handle case where file name has invalid UTF-8 characters (remove need for to_string_lossy)
        Ok(value
            .file_name()
            .ok_or(ErrorKind::ParseError("File path ends in ..".to_string()))?
            .to_string_lossy()
            .to_string()
            .try_into()?)
    }
}

impl Display for DeviceEndpointRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.device_name, self.endpoint_name)
    }
}

/// Represents an asset associated with a specific device and endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetRef {
    /// The name of the asset
    pub name: String,
    /// The name of the device
    pub device_name: String,
    /// The name of the endpoint
    pub endpoint_name: String,
}

// ~~~~~~~~~~~~~~~~~ Observations ~~~~~~~~~~~~~~~~~~~~~

/// Represents an observation for device endpoint creation events.
#[allow(dead_code)]
pub struct DeviceEndpointCreateObservation {
    /// A file watcher used to monitor changes in the file mount.
    debouncer: notify_debouncer_full::Debouncer<RecommendedWatcher, RecommendedCache>,
    /// A channel for receiving notifications about device endpoint creation events.
    create_device_rx: UnboundedReceiver<(DeviceEndpointRef, AssetCreateObservation)>,
}

impl DeviceEndpointCreateObservation {
    /// Creates an instance of [`DeviceEndpointCreateObservation`] to observe device endpoint creation events.
    /// 
    /// Returns Ok([`DeviceEndpointCreateObservation`]) if the observation is successfully created, otherwise returns an [`Error`].
    /// 
    /// # Arguments
    /// * `debounce_duration` - The duration to debounce incoming I/O events.
    /// 
    /// # Errors
    /// - [`Error`] of kind [`ErrorKind::EnvironmentVariableError`] if the environment variable was not able to be read.
    /// - [`Error`] of kind [`ErrorKind::WatcherError`] if the watcher could not be created.
    /// - [`Error`] of kind [`ErrorKind::IoError`] if there are issues accessing the file mount.
    /// - [`Error`] of kind [`ErrorKind::ParseError`] if there are issues parsing the file names and content.
    pub fn new(
        debounce_duration: Duration,
    ) -> Result<DeviceEndpointCreateObservation, Error> {
        let mount_path = get_mount_path()?;

        // This channel is used to send notifications about device endpoint creation
        let (create_device_tx, create_device_rx) = mpsc::unbounded_channel();

        // Tracks devices and assets in the file mount.
        let mut file_mount_map = FileMountMap::new(create_device_tx.clone());

        // Add the already existing device endpoints and their assets to the file mount map
        get_device_endpoint_names(&mount_path)?
            .iter()
            .try_for_each(|device| {
                file_mount_map.insert_device_endpoint(device);

                let assets = get_asset_names(&mount_path, device)?;
                file_mount_map.update_assets(device, assets);

                Ok::<_, Error>(())
            })?;

        // TODO: When the number of files being tracked is large, the watcher might not be able to keep up with the events.
        // In the future we should consider adding redundancy checks (like hashing device endpoint files)
        // to ensure that the file mount map is in sync with the file mount.
        let mut debouncer = new_debouncer(
            debounce_duration,
            None,
            move |res: Result<Vec<notify_debouncer_full::DebouncedEvent>, Vec<notify::Error>>| {
                match res {
                    Ok(events) => {
                        // TODO: There might be a case where we receive events out of order (i.e a modify before a create). The file mount map accounts for this but
                        // it might be better to just match on an Any.
                        events.iter().for_each(|debounced_event| {
                            match debounced_event.event.kind {
                                EventKind::Create(event::CreateKind::File) => { // Event signals the creation of a device endpoint
                                    debounced_event.paths.iter().for_each(|path| {
                                        // TODO: We could use an expect here since we are sure the path is valid
                                        let mount_path = match path.parent() {
                                            Some(path) => path,
                                            None => {
                                                log::warn!("Failed to get parent path from device endpoint");
                                                return;
                                            }
                                        };

                                        let device = match DeviceEndpointRef::try_from(path) {
                                            Ok(device) => device,
                                            Err(err) => {
                                                log::warn!("Failed to parse device endpoint from path: {:?}", err);
                                                return;
                                            }
                                        };

                                        // Insert the device endpoint into the file mount map
                                        file_mount_map.insert_device_endpoint(&device);

                                        // The create event is for a new file, so we need to get the assets
                                        let assets =
                                            match get_asset_names(mount_path, &device) {
                                                Ok(assets) => assets,
                                                Err(err) => {
                                                    log::warn!("Failed to get asset names: {:?}", err);
                                                    return;
                                                }
                                            };

                                        // Update the file mount map with the new assets
                                        file_mount_map.update_assets(&device, assets);
                                    });
                                }
                                EventKind::Modify(_) => { // Event signals the creation or deletion of an asset
                                    debounced_event.paths.iter().for_each(|path| {
                                        // TODO: We could use an expect here since we are sure the path is valid
                                        let mount_path = match path.parent() {
                                            Some(path) => path,
                                            None => {
                                                log::warn!("Failed to get parent path from device endpoint");
                                                return;
                                            }
                                        };

                                        let device = match DeviceEndpointRef::try_from(path) {
                                            Ok(device) => device,
                                            Err(err) => {
                                                log::warn!("Failed to parse device endpoint from path: {:?}", err);
                                                return;
                                            }
                                        };

                                        // Get assets in the file mount
                                        let assets =
                                            match get_asset_names(mount_path, &device) {
                                                Ok(assets) => assets,
                                                Err(err) => {
                                                    log::warn!("Failed to get asset names: {:?}", err);
                                                    return;
                                                }
                                            };

                                        // Update the file mount map with the new assets
                                        file_mount_map.update_assets(&device, assets);
                                    });
                                }
                                EventKind::Remove(event::RemoveKind::File) => { // Event signals the deletion of a device endpoint
                                    debounced_event.paths.iter().for_each(|path| {
                                        let device = match DeviceEndpointRef::try_from(path) {
                                            Ok(device) => device,
                                            Err(err) => {
                                                log::warn!("Failed to parse device endpoint from path: {:?}", err);
                                                return;
                                            }
                                        };

                                        // Remove the device endpoint from the file mount map
                                        // When the device endpoint is removed, all the assets are 
                                        // removed as well triggering the deletion token for each asset
                                        file_mount_map.remove_device_endpoint(&device);
                                    });
                                }
                                _ => { /* Ignore other events */ }
                            }
                        });
                    }
                    Err(err) => {
                        // TODO: There should be a way for us to surface this error to the user
                        err.iter().for_each(|e| {
                            log::error!("Error processing events from watcher: {:?}", e);
                        });
                    }
                }
            },
        ).map_err(ErrorKind::from)?;

        // Start watching the file mount path for create, modify and remove events
        debouncer
            .watch(&mount_path, notify::RecursiveMode::NonRecursive)
            .map_err(ErrorKind::from)?;

        Ok(Self {
            debouncer,
            create_device_rx,
        })
    }

    /// Receives a notification for a newly created device endpoint.
    /// 
    /// Returns Some(([`DeviceEndpointRef`], [`AssetCreateObservation`])) if a notification is received or `None` 
    /// if there will be no more notifications (i.e. the channel is closed). This should not happen unless the
    /// [`DeviceEndpointCreateObservation`] is dropped.
    /// 
    /// The [`AssetCreateObservation`] should be used to receive notifications for asset creation events
    /// associated with the device endpoint.
    pub async fn recv_notification(
        &mut self,
    ) -> Option<(DeviceEndpointRef, AssetCreateObservation)> {
        self.create_device_rx.recv().await
    }
}

/// Represents an observation for asset creation events.
pub struct AssetCreateObservation {
    /// A channel for receiving notifications about asset creation events.
    asset_creation_rx: UnboundedReceiver<(AssetRef, AssetDeletionToken)>,
}

impl AssetCreateObservation {
    /// Creates an instance of [`AssetCreateObservation`] to observe asset creation events.
    /// 
    /// Returns a new [`AssetCreateObservation`] instance.
    pub(crate) fn new(
        asset_creation_rx: UnboundedReceiver<(AssetRef, AssetDeletionToken)>,
    ) -> AssetCreateObservation {
        Self { asset_creation_rx }
    }

    /// Receives a notification for a newly created asset.
    /// 
    /// Returns Some(([`AssetRef`], [`AssetDeletionToken`])) if a notification is received or `None`
    /// if there will be no more notifications (i.e. the device endpoint was deleted).
    /// 
    /// The [`AssetDeletionToken`] can be used to wait for the deletion of the asset.
    pub async fn recv_notification(&mut self) -> Option<(AssetRef, AssetDeletionToken)> {
        self.asset_creation_rx.recv().await
    }
}

/// Represents a token that can be used to wait for the deletion of an asset.
pub struct AssetDeletionToken(oneshot::Receiver<()>);

impl std::future::Future for AssetDeletionToken {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        mut cx: &mut Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match std::pin::Pin::new(&mut self.get_mut().0).poll(&mut cx) {
            std::task::Poll::Ready(Ok(())) => std::task::Poll::Ready(()),
            std::task::Poll::Ready(Err(_)) => std::task::Poll::Ready(()),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashSet;
    use std::fs;

    const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);

    struct TempFileMountManager {
        dir: tempfile::TempDir,
    }

    impl TempFileMountManager {
        fn new(dir_name: &str) -> Self {
            Self {
                dir: tempfile::TempDir::with_prefix(dir_name).unwrap(),
            }
        }

        fn add_device_endpoint(
            &self,
            device_endpoint: &DeviceEndpointRef,
            asset_names: &[AssetRef],
        ) {
            let file_path = self.dir.path().join(device_endpoint.to_string());
            let content: Vec<_> = asset_names.iter().map(|asset| asset.name.clone()).collect();
            let content = content.join(";");
            fs::write(file_path, content).unwrap();
        }

        fn remove_device_endpoint(&self, device_endpoint: &DeviceEndpointRef) {
            let file_path = self.dir.path().join(device_endpoint.to_string());
            fs::remove_file(file_path).unwrap();
        }

        fn add_asset(&self, device_endpoint: &DeviceEndpointRef, asset: &AssetRef) {
            let file_path = self.dir.path().join(device_endpoint.to_string());
            let mut content = fs::read_to_string(&file_path).unwrap();

            // Make sure the asset name is not already present
            if content.contains(asset.name.as_str()) {
                return;
            }
            // Append the asset name to the file
            if !content.is_empty() {
                content.push(';');
            }
            content.push_str(asset.name.as_str());
            fs::write(file_path, content).unwrap();
        }

        fn remove_asset(&self, device_endpoint: &DeviceEndpointRef, asset: &AssetRef) {
            let file_path = self.dir.path().join(device_endpoint.to_string());
            let mut content = fs::read_to_string(&file_path).unwrap();

            // Remove the asset name from the file
            content = content
                .split(';')
                .filter(|&name| name != asset.name.as_str())
                .collect::<Vec<_>>()
                .join(";");

            fs::write(file_path, content).unwrap();
        }

        fn path(&self) -> &Path {
            self.dir.path()
        }
    }

    #[test]
    fn test_get_device_endpoint_names() {
        let file_mount_manager = TempFileMountManager::new("test_mount");

        let device1_endpoint1 = DeviceEndpointRef {
            device_name: "device1".to_string(),
            endpoint_name: "endpoint1".to_string(),
        };
        let device1_endpoint1_assets = vec![AssetRef {
            name: "asset1".to_string(),
            device_name: device1_endpoint1.device_name.clone(),
            endpoint_name: device1_endpoint1.endpoint_name.clone(),
        }];
        let device1_endpoint2 = DeviceEndpointRef {
            device_name: "device1".to_string(),
            endpoint_name: "endpoint2".to_string(),
        };
        let device1_endpoint2_assets = vec![
            AssetRef {
                name: "asset2".to_string(),
                device_name: device1_endpoint2.device_name.clone(),
                endpoint_name: device1_endpoint2.endpoint_name.clone(),
            },
            AssetRef {
                name: "asset3".to_string(),
                device_name: device1_endpoint2.device_name.clone(),
                endpoint_name: device1_endpoint2.endpoint_name.clone(),
            },
        ];
        let device2_endpoint3 = DeviceEndpointRef {
            device_name: "device2".to_string(),
            endpoint_name: "endpoint3".to_string(),
        };
        let device2_endpoint3_assets = vec![AssetRef {
            name: "asset3".to_string(),
            device_name: device2_endpoint3.device_name.clone(),
            endpoint_name: device2_endpoint3.endpoint_name.clone(),
        }];

        file_mount_manager.add_device_endpoint(&device1_endpoint1, &device1_endpoint1_assets);
        file_mount_manager.add_device_endpoint(&device1_endpoint2, &device1_endpoint2_assets);
        file_mount_manager.add_device_endpoint(&device2_endpoint3, &device2_endpoint3_assets);

        temp_env::with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            || {
                let mount_path = get_mount_path().unwrap();

                let device_endpoints = get_device_endpoint_names(mount_path.as_path()).unwrap();

                assert_eq!(device_endpoints.len(), 3);
                assert!(device_endpoints.contains(&device1_endpoint1));
                assert!(device_endpoints.contains(&device1_endpoint2));
                assert!(device_endpoints.contains(&device2_endpoint3));
            },
        )
    }

    #[test]
    fn test_get_asset_names() {
        let file_mount_manager = TempFileMountManager::new("test_mount");
        let device1_endpoint1 = DeviceEndpointRef {
            device_name: "device1".to_string(),
            endpoint_name: "endpoint1".to_string(),
        };
        let device1_endpoint1_assets = vec![
            AssetRef {
                name: "asset1".to_string(),
                device_name: device1_endpoint1.device_name.clone(),
                endpoint_name: device1_endpoint1.endpoint_name.clone(),
            },
            AssetRef {
                name: "asset2".to_string(),
                device_name: device1_endpoint1.device_name.clone(),
                endpoint_name: device1_endpoint1.endpoint_name.clone(),
            },
            AssetRef {
                name: "asset3".to_string(),
                device_name: device1_endpoint1.device_name.clone(),
                endpoint_name: device1_endpoint1.endpoint_name.clone(),
            },
        ];

        file_mount_manager.add_device_endpoint(&device1_endpoint1, &device1_endpoint1_assets);

        temp_env::with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            || {
                let mount_path = get_mount_path().unwrap();

                let device_endpoints = get_device_endpoint_names(mount_path.as_path()).unwrap();

                let device1_endpoint1_assets =
                    get_asset_names(mount_path.as_path(), &device_endpoints[0]).unwrap();

                assert_eq!(
                    device1_endpoint1_assets,
                    vec![
                        AssetRef {
                            name: "asset1".to_string(),
                            device_name: "device1".to_string(),
                            endpoint_name: "endpoint1".to_string(),
                        },
                        AssetRef {
                            name: "asset2".to_string(),
                            device_name: "device1".to_string(),
                            endpoint_name: "endpoint1".to_string(),
                        },
                        AssetRef {
                            name: "asset3".to_string(),
                            device_name: "device1".to_string(),
                            endpoint_name: "endpoint1".to_string(),
                        },
                    ]
                );
            },
        )
    }

    #[tokio::test]
    async fn test_device_endpoint_create_observation_pre_mounted_success() {
        let file_mount_manager = TempFileMountManager::new("test_mount");

        let device1_endpoint1 = DeviceEndpointRef {
            device_name: "device1".to_string(),
            endpoint_name: "endpoint1".to_string(),
        };
        let device1_endpoint1_assets = vec![];
        let device1_endpoint2 = DeviceEndpointRef {
            device_name: "device1".to_string(),
            endpoint_name: "endpoint2".to_string(),
        };
        let device1_endpoint2_assets = vec![];
        let device2_endpoint3 = DeviceEndpointRef {
            device_name: "device2".to_string(),
            endpoint_name: "endpoint3".to_string(),
        };
        let device2_endpoint3_assets = vec![];

        file_mount_manager.add_device_endpoint(&device1_endpoint1, &device1_endpoint1_assets);
        file_mount_manager.add_device_endpoint(&device1_endpoint2, &device1_endpoint2_assets);
        file_mount_manager.add_device_endpoint(&device2_endpoint3, &device2_endpoint3_assets);

        let mut device_endpoints = HashSet::from([
            device1_endpoint1.clone(),
            device1_endpoint2.clone(),
            device2_endpoint3.clone(),
        ]);

        temp_env::async_with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            async {
                let mut test_device_endpoint_create_observation =
                    DeviceEndpointCreateObservation::new(
                        DEBOUNCE_DURATION,
                    )
                    .unwrap();
        
                while !device_endpoints.is_empty() {
                    tokio::select! {
                        Some((device_endpoint, _)) = test_device_endpoint_create_observation.recv_notification() => {
                            assert!(device_endpoints.remove(&device_endpoint));
                        },
                        _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                            panic!("Failed to receive device endpoint creation notification");
                        }
                    };
                }
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_device_endpoint_create_observation_live_mount_success() {
        let file_mount_manager = TempFileMountManager::new("test_mount");

        temp_env::async_with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            async {
                let mut test_device_endpoint_create_observation =
                    DeviceEndpointCreateObservation::new(
                        DEBOUNCE_DURATION,
                    )
                    .unwrap();

                let device1_endpoint1 = DeviceEndpointRef {
                    device_name: "device1".to_string(),
                    endpoint_name: "endpoint1".to_string(),
                };
                let device1_endpoint1_assets = vec![];
                let device1_endpoint2 = DeviceEndpointRef {
                    device_name: "device1".to_string(),
                    endpoint_name: "endpoint2".to_string(),
                };
                let device1_endpoint2_assets = vec![];
                let device2_endpoint3 = DeviceEndpointRef {
                    device_name: "device2".to_string(),
                    endpoint_name: "endpoint3".to_string(),
                };
                let device2_endpoint3_assets = vec![];

                file_mount_manager.add_device_endpoint(&device1_endpoint1, &device1_endpoint1_assets);
                file_mount_manager.add_device_endpoint(&device1_endpoint2, &device1_endpoint2_assets);
                file_mount_manager.add_device_endpoint(&device2_endpoint3, &device2_endpoint3_assets);
        
                let mut device_endpoints = HashSet::from([
                    device1_endpoint1.clone(),
                    device1_endpoint2.clone(),
                    device2_endpoint3.clone(),
                ]);
            
        
                while !device_endpoints.is_empty() {
                    tokio::select! {
                        Some((device_endpoint, _)) = test_device_endpoint_create_observation.recv_notification() => {
                            assert!(device_endpoints.remove(&device_endpoint));
                        },
                        _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                            panic!("Failed to receive device endpoint creation notification");
                        }
                    };
                }
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_asset_create_observation_pre_mounted_success() {
        let file_mount_manager = TempFileMountManager::new("test_mount");

        let device1_endpoint1 = DeviceEndpointRef {
            device_name: "device1".to_string(),
            endpoint_name: "endpoint1".to_string(),
        };
        let device1_endpoint1_assets = vec![
            AssetRef {
                name: "asset1".to_string(),
                device_name: device1_endpoint1.device_name.clone(),
                endpoint_name: device1_endpoint1.endpoint_name.clone(),
            },
            AssetRef {
                name: "asset2".to_string(),
                device_name: device1_endpoint1.device_name.clone(),
                endpoint_name: device1_endpoint1.endpoint_name.clone(),
            },
        ];

        file_mount_manager.add_device_endpoint(&device1_endpoint1, &device1_endpoint1_assets);

        let mut assets: HashSet<AssetRef> = HashSet::from_iter(device1_endpoint1_assets.clone());

        temp_env::async_with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            async {
                let mut test_device_endpoint_create_observation =
                    DeviceEndpointCreateObservation::new(
                        DEBOUNCE_DURATION,
                    )
                    .unwrap();

                tokio::select! {
                    Some((device_endpoint, mut asset_observation)) = test_device_endpoint_create_observation.recv_notification() => {
                        assert_eq!(device_endpoint, device1_endpoint1);
                        // Observe asset creation
                        while !assets.is_empty() {
                            tokio::select! {
                                Some((asset, _)) = asset_observation.recv_notification() => {
                                    assert!(assets.remove(&asset));
                                },
                                _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                                    panic!("Failed to receive asset creation notification");
                                }
                            };
                        }
                    },
                    _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                        panic!("Failed to receive device endpoint creation notification");
                    }
                }
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_asset_create_observation_live_mount_success() {
        let file_mount_manager = TempFileMountManager::new("test_mount");

        temp_env::async_with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            async {
                let mut test_device_endpoint_create_observation =
                    DeviceEndpointCreateObservation::new(
                        DEBOUNCE_DURATION,
                    )
                    .unwrap();

                let device1_endpoint1 = DeviceEndpointRef {
                    device_name: "device1".to_string(),
                    endpoint_name: "endpoint1".to_string(),
                };
                let device1_endpoint1_assets = vec![
                    AssetRef {
                        name: "asset1".to_string(),
                        device_name: device1_endpoint1.device_name.clone(),
                        endpoint_name: device1_endpoint1.endpoint_name.clone(),
                    },
                    AssetRef {
                        name: "asset2".to_string(),
                        device_name: device1_endpoint1.device_name.clone(),
                        endpoint_name: device1_endpoint1.endpoint_name.clone(),
                    },
                ];
        
                file_mount_manager.add_device_endpoint(&device1_endpoint1, &device1_endpoint1_assets);
        
                let mut assets: HashSet<AssetRef> = HashSet::from_iter(device1_endpoint1_assets.clone());

                tokio::select! {
                    Some((device_endpoint, mut asset_observation)) = test_device_endpoint_create_observation.recv_notification() => {
                        assert_eq!(device_endpoint, device1_endpoint1);
                        // Observe asset creation
                        while !assets.is_empty() {
                            tokio::select! {
                                Some((asset, _)) = asset_observation.recv_notification() => {
                                    assert!(assets.remove(&asset));
                                },
                                _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                                    panic!("Failed to receive asset creation notification");
                                }
                            };
                        }
                    },
                    _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                        panic!("Failed to receive device endpoint creation notification");
                    }
                }
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_device_endpoint_remove_triggers_asset_deletion_tokens_success() {
        let file_mount_manager = TempFileMountManager::new("test_mount");

        let device1_endpoint1 = DeviceEndpointRef {
            device_name: "device1".to_string(),
            endpoint_name: "endpoint1".to_string(),
        };
        let device1_endpoint1_assets = vec![
            AssetRef {
                name: "asset1".to_string(),
                device_name: device1_endpoint1.device_name.clone(),
                endpoint_name: device1_endpoint1.endpoint_name.clone(),
            },
            AssetRef {
                name: "asset2".to_string(),
                device_name: device1_endpoint1.device_name.clone(),
                endpoint_name: device1_endpoint1.endpoint_name.clone(),
            },
        ];

        file_mount_manager.add_device_endpoint(&device1_endpoint1, &device1_endpoint1_assets);

        temp_env::async_with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            async {
                let mut test_device_endpoint_create_observation =
                    DeviceEndpointCreateObservation::new(
                        DEBOUNCE_DURATION,
                    )
                    .unwrap();

                let mut asset_deletion_tokens = Vec::new();

                tokio::select! {
                    Some((device_endpoint, mut asset_observation)) = test_device_endpoint_create_observation.recv_notification() => {
                        assert_eq!(device_endpoint, device1_endpoint1);

                        // Collect asset deletion tokens
                        for _ in 0..device1_endpoint1_assets.len() {
                            tokio::select! {
                                Some((_, deletion_token)) = asset_observation.recv_notification() => {
                                    asset_deletion_tokens.push(deletion_token);
                                },
                                _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                                    panic!("Failed to receive asset creation notification");
                                }
                            };
                        }

                        // Remove the device endpoint
                        file_mount_manager.remove_device_endpoint(&device1_endpoint1);

                        // Wait for the device endpoint create observation to return None 
                        tokio::select! {
                            res = asset_observation.recv_notification() => {
                                assert!(res.is_none(), "Device endpoint create observation should return None after device endpoint removal");
                            },
                            _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                                panic!("Failed to receive device endpoint deletion notification");
                            }
                        }
                    },
                    _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                        panic!("Failed to receive device endpoint creation notification");
                    }
                }

                // Ensure all asset deletion tokens are triggered
                for deletion_token in asset_deletion_tokens {
                    tokio::select! {
                        _ = deletion_token => {
                            // Token triggered successfully
                        },
                        _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                            panic!("Asset deletion token was not triggered");
                        }
                    }
                }
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_single_asset_removal_triggers_deletion_token_success() {
        let file_mount_manager = TempFileMountManager::new("test_mount");

        let device1_endpoint1 = DeviceEndpointRef {
            device_name: "device1".to_string(),
            endpoint_name: "endpoint1".to_string(),
        };
        let device1_endpoint1_assets = vec![
            AssetRef {
                name: "asset1".to_string(),
                device_name: device1_endpoint1.device_name.clone(),
                endpoint_name: device1_endpoint1.endpoint_name.clone(),
            },
            AssetRef {
                name: "asset2".to_string(),
                device_name: device1_endpoint1.device_name.clone(),
                endpoint_name: device1_endpoint1.endpoint_name.clone(),
            },
        ];

        file_mount_manager.add_device_endpoint(&device1_endpoint1, &device1_endpoint1_assets);

        temp_env::async_with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            async {
                let mut test_device_endpoint_create_observation =
                    DeviceEndpointCreateObservation::new(
                        DEBOUNCE_DURATION,
                    )
                    .unwrap();

                let mut asset_deletion_tokens = HashMap::new();

                tokio::select! {
                    Some((device_endpoint, mut asset_observation)) = test_device_endpoint_create_observation.recv_notification() => {
                        assert_eq!(device_endpoint, device1_endpoint1);

                        // Collect asset deletion tokens
                        for _ in 0..device1_endpoint1_assets.len() {
                            tokio::select! {
                                Some((asset, deletion_token)) = asset_observation.recv_notification() => {
                                    asset_deletion_tokens.insert(asset, deletion_token);
                                },
                                _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                                    panic!("Failed to receive asset creation notification");
                                }
                            };
                        }

                        // Remove a single asset
                        let asset_to_remove = &device1_endpoint1_assets[0];
                        file_mount_manager.remove_asset(&device1_endpoint1, asset_to_remove);

                        // Wait for the asset deletion token to be triggered
                        if let Some(deletion_token) = asset_deletion_tokens.remove(asset_to_remove) {
                            tokio::select! {
                                _ = deletion_token => {
                                    // Token triggered successfully
                                },
                                _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                                    panic!("Asset deletion token was not triggered");
                                }
                            }
                        } else {
                            panic!("Asset deletion token not found for the removed asset");
                        }

                        // Check that the other asset's deletion token is still valid
                        let remaining_asset = &device1_endpoint1_assets[1];

                        if let Some(deletion_token) = asset_deletion_tokens.remove(remaining_asset) {
                            tokio::select! {
                                _ = deletion_token => {
                                    panic!("Asset deletion token was triggered for the remaining asset");
                                },
                                _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                                    // Token not triggered, which is expected
                                }
                            }
                        } else {
                            panic!("Asset deletion token not found for the remaining asset");
                        }
                    },
                    _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                        panic!("Failed to receive device endpoint creation notification");
                    }
                }
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_single_asset_addition_triggers_creation_notification_success() {
        let file_mount_manager = TempFileMountManager::new("test_mount");

        let device1_endpoint1 = DeviceEndpointRef {
            device_name: "device1".to_string(),
            endpoint_name: "endpoint1".to_string(),
        };
        let device1_endpoint1_assets = vec![AssetRef {
            name: "asset1".to_string(),
            device_name: device1_endpoint1.device_name.clone(),
            endpoint_name: device1_endpoint1.endpoint_name.clone(),
        }];

        file_mount_manager.add_device_endpoint(&device1_endpoint1, &device1_endpoint1_assets);

        temp_env::async_with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            async {
                let mut test_device_endpoint_create_observation =
                    DeviceEndpointCreateObservation::new(
                        DEBOUNCE_DURATION,
                    )
                    .unwrap();

                tokio::select! {
                    Some((device_endpoint, mut asset_observation)) = test_device_endpoint_create_observation.recv_notification() => {
                        assert_eq!(device_endpoint, device1_endpoint1);

                        // Collect initial asset creation notifications
                        for _ in 0..device1_endpoint1_assets.len() {
                            tokio::select! {
                                Some((asset, _)) = asset_observation.recv_notification() => {
                                    assert!(device1_endpoint1_assets.contains(&asset));
                                },
                                _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                                    panic!("Failed to receive initial asset creation notification");
                                }
                            };
                        }

                        // Add a new asset
                        let new_asset = AssetRef {
                            name: "asset2".to_string(),
                            device_name: device1_endpoint1.device_name.clone(),
                            endpoint_name: device1_endpoint1.endpoint_name.clone(),
                        };
                        file_mount_manager.add_asset(&device1_endpoint1, &new_asset);

                        // Wait for the new asset creation notification
                        tokio::select! {
                            Some((asset, _)) = asset_observation.recv_notification() => {
                                assert_eq!(asset, new_asset);
                            },
                            _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                                panic!("Failed to receive new asset creation notification");
                            }
                        }
                    },
                    _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                        panic!("Failed to receive device endpoint creation notification");
                    }
                }
            },
        )
        .await
    }
}
