// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Azure Device Registry Client that uses file mount to get names and create/delete notifications.

use std::collections::{HashMap, HashSet};
use std::env::{self, VarError};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::task::Context;
use std::time::Duration;

use notify::{
    RecommendedWatcher,
    event::{self, EventKind},
};
use notify_debouncer_full::{RecommendedCache, new_debouncer};
use tokio::sync::broadcast::{self, Receiver};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

/// Environment variable name for the directory containing device and asset names.
const ADR_RESOURCES_NAME_MOUNT_PATH: &str = "ADR_RESOURCES_NAME_MOUNT_PATH";

// FIN: HELPER FUNCTIONS

/// Write docs for this
pub fn get_mount_path() -> Result<PathBuf, FileMountError> {
    match env::var(ADR_RESOURCES_NAME_MOUNT_PATH) {
        Ok(path) => Ok(path.into()),
        Err(VarError::NotPresent) => todo!(),
        Err(VarError::NotUnicode(_)) => todo!(),
    }
}

/// Gets names of all devices from the file mount.
///
/// # Returns
/// A vector of [`DeviceEndpointRef`].
///
/// # Errors
/// Returns an error if the file mount cannot be accessed or if there is an issue with the watcher.
pub fn get_device_endpoint_names(
    mount_path: &Path,
) -> Result<Vec<DeviceEndpointRef>, FileMountError> {
    // Access the directory
    Ok(std::fs::read_dir(mount_path)
        .unwrap()
        .map(|entry| {
            entry
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .try_into()
                .unwrap()
        })
        .collect::<Vec<_>>())
}

/// Get names of all available assets from the [`DeviceEndpointRef`].
///
/// # Arguments
/// * `_device_endpoint` - A reference to the device endpoint for which to get asset names.
///
/// # Returns
/// A vector of [`AssetRef`].
///  
/// # Errors
/// Returns an error if the file mount cannot be accessed or if there is an issue with the watcher.
fn get_asset_names(
    mount_path: &Path,
    device_endpoint: &DeviceEndpointRef,
) -> Result<Vec<AssetRef>, FileMountError> {
    // Access the file and parse out the asset names
    let file_path = mount_path.join(device_endpoint.to_string());

    match std::fs::read(file_path) {
        Ok(file_content) => {
            // convert the file content to a string
            let file_content = String::from_utf8(file_content).unwrap();

            Ok(file_content
                .split(';')
                .map(|asset_name| AssetRef {
                    name: asset_name.to_string(),
                    device_name: device_endpoint.device_name.clone(),
                    endpoint_name: device_endpoint.endpoint_name.clone(),
                })
                .collect())
        }
        Err(_) => todo!(),
    }
}

struct FileMountMap {
    /// A map that tracks devices and their associated assets.
    file_mount_map: HashMap<
        DeviceEndpointRef,
        (
            UnboundedSender<(AssetRef, AssetDeletionToken)>,
            HashMap<AssetRef, oneshot::Sender<()>>,
        ),
    >,
    /// Used for sending notifications about device creation.
    create_device_tx: UnboundedSender<(DeviceEndpointRef, AssetCreateObservation)>,
}

impl FileMountMap {
    pub fn new(
        create_device_tx: UnboundedSender<(DeviceEndpointRef, AssetCreateObservation)>,
    ) -> FileMountMap {
        FileMountMap {
            file_mount_map: HashMap::new(),
            create_device_tx,
        }
    }

    pub fn insert_device_endpoint(&mut self, device: &DeviceEndpointRef) {
        // Check if the device already exists
        if self.file_mount_map.contains_key(device) {
            return;
        }

        let (asset_creation_tx, asset_creation_rx) = mpsc::unbounded_channel();

        self.file_mount_map
            .insert(device.clone(), (asset_creation_tx, HashMap::new()));

        // Notify on file creation
        self.create_device_tx
            .send((
                device.clone(),
                AssetCreateObservation::new(asset_creation_rx),
            ))
            .unwrap();
    }

    pub fn update_assets(&mut self, device: &DeviceEndpointRef, assets: Vec<AssetRef>) {
        // Get the current tracked assets for the device
        let (create_asset_tx, tracked_assets) = &mut self.file_mount_map.get_mut(device).unwrap();

        // Remove assets that are not in the current assets
        tracked_assets.retain(|tracked_asset, _| {
            // FIN: Optimize this, we should not be iterating over the whole list every time
            assets.iter().any(|asset| tracked_asset == asset)
        });

        // Add assets that are not being tracked
        assets.iter().for_each(|asset| {
            if !tracked_assets.contains_key(asset) {
                let (asset_deletion_tx, asset_deletion_rx) = oneshot::channel();

                let asset_deletion_token = AssetDeletionToken(asset_deletion_rx);

                // Add the new asset to the tracked assets
                tracked_assets.insert(asset.clone(), asset_deletion_tx);

                // Notify on asset creation
                create_asset_tx
                    .send((asset.clone(), asset_deletion_token))
                    .unwrap();
            }
        });
    }

    pub fn insert_asset(&mut self, device: &DeviceEndpointRef, asset: &AssetRef) {
        // Check if the asset already exists
        if self
            .file_mount_map
            .get(device)
            .unwrap()
            .1
            .contains_key(asset)
        {
            return;
        }

        let (asset_deletion_tx, asset_deletion_rx) = oneshot::channel();

        let asset_deletion_token = AssetDeletionToken(asset_deletion_rx);

        // Add the new asset to the tracked assets
        self.file_mount_map
            .get_mut(&device)
            .unwrap()
            .1
            .insert(asset.clone(), asset_deletion_tx);

        // Notify on asset creation // FIN: Optimize this, we are getting 0 and 1 here so we can just do it once.
        self.file_mount_map
            .get_mut(&device)
            .unwrap()
            .0
            .send((asset.clone(), asset_deletion_token))
            .unwrap();
    }

    pub fn remove_device_endpoint(&mut self, device: &DeviceEndpointRef) {
        // Remove entry from the file mount map
        self.file_mount_map.remove(device);
    }

    pub fn remove_asset(&mut self, device: &DeviceEndpointRef, asset: &AssetRef) {
        // Remove the asset from the tracked assets
        self.file_mount_map.get_mut(device).unwrap().1.remove(asset);
    }
}

/// A client that interacts with the file mount
///
/// This client provides functionality to retrieve device names and handle
/// create/delete notifications from the Azure Device Registry.
#[allow(dead_code)]
pub struct DeviceEndpointCreateObservation {
    /// A file watcher used to monitor changes in the file mount.
    debouncer: notify_debouncer_full::Debouncer<RecommendedWatcher, RecommendedCache>,
    create_device_rx: UnboundedReceiver<(DeviceEndpointRef, AssetCreateObservation)>,
}

impl DeviceEndpointCreateObservation {
    /// Observes the creation of device endpoints.
    ///
    /// # Returns
    /// Returns OK([`DeviceEndpointCreateObservation`]) if observation was success.
    /// The [`DeviceEndpointCreateObservation`] can be used to receive notifications.
    ///
    /// # Errors
    /// Returns an error if the file mount cannot be accessed or if there is an issue with the watcher.
    /// Returns an error if the file mount cannot be accessed or if there is an issue with the watcher.
    pub fn new_device_endpoint_create_observation(
        debounce_duration: Duration,
    ) -> Result<DeviceEndpointCreateObservation, FileMountError> {
        let mount_path = get_mount_path()?;

        let (create_device_tx, create_device_rx) = mpsc::unbounded_channel();

        // Tracks devices and assets in the file mount.
        let mut file_mount_map = FileMountMap::new(create_device_tx.clone());

        get_device_endpoint_names(&mount_path)?
            .iter()
            .for_each(|device| {
                file_mount_map.insert_device_endpoint(device);

                let assets = get_asset_names(&mount_path, device)
                    .unwrap()
                    .iter()
                    .for_each(|asset| {
                        file_mount_map.insert_asset(device, asset);
                    });
            });

        let debouncer = match new_debouncer(
            debounce_duration,
            None,
            move |res: Result<Vec<notify_debouncer_full::DebouncedEvent>, Vec<notify::Error>>| {
                match res {
                    Ok(events) => {
                        // Iterate over the events and check for relevant changes
                        // FIN: What if we receive events in a different order?
                        events.iter().for_each(|debounced_event| {
                            match debounced_event.event.kind {
                                EventKind::Create(event::CreateKind::File) => {
                                    debounced_event.paths.iter().for_each(|path| {
                                        let device: DeviceEndpointRef = path
                                            .file_name()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()
                                            .to_string()
                                            .try_into()
                                            .unwrap();

                                        file_mount_map.insert_device_endpoint(&device);
                                    });
                                }
                                EventKind::Modify(event::ModifyKind::Data(
                                    event::DataChange::Content,
                                )) => {
                                    debounced_event.paths.iter().for_each(|path| {
                                        let device: DeviceEndpointRef = path
                                            .file_name()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()
                                            .to_string()
                                            .try_into()
                                            .unwrap();

                                        // Get updated assets
                                        let assets = get_asset_names(path, &device).unwrap();

                                        file_mount_map.update_assets(&device, assets);
                                    });
                                }
                                EventKind::Remove(event::RemoveKind::File) => {
                                    // Notify on file removal
                                    debounced_event.paths.iter().for_each(|path| {
                                        let device: DeviceEndpointRef = path
                                            .file_name()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()
                                            .to_string()
                                            .try_into()
                                            .unwrap();

                                        // Remove entry from the file mount map
                                        file_mount_map.remove_device_endpoint(&device);
                                    });
                                }
                                _ => { /* Ignore other events */ }
                            }
                        });
                    }
                    Err(_err) => {
                        todo!();
                    }
                }
            },
        ) {
            Ok(mut debouncer) => {
                // Watch the directory for changes
                if let Err(err) = debouncer.watch(&mount_path, notify::RecursiveMode::NonRecursive)
                {
                    todo!();
                }
                debouncer
            }
            Err(_err) => todo!(),
        };

        Ok(Self {
            debouncer,
            create_device_rx,
        })
    }

    /// Finish docs for this
    pub async fn recv_notification(
        &mut self,
    ) -> Option<(DeviceEndpointRef, AssetCreateObservation)> {
        self.create_device_rx.recv().await
    }

    /// Observes for the deletion of a device endpoint.
    ///
    /// # Arguments
    /// * `device_endpoint` - A reference to the device endpoint for which to observe deletion.
    /// * `_timeout` - The duration to wait for the observation to complete.
    ///
    /// # Returns
    /// Returns OK([`DeviceEndpointDeleteObservation`]) if observation was success.
    /// The [`DeviceEndpointDeleteObservation`] can be used to receive notifications.
    ///
    /// # Errors
    /// Returns an error if the file mount cannot be accessed or if there is an issue with the watcher.
    pub async fn observe_device_endpoint_delete(
        &self,
        _device_endpoint: DeviceEndpointRef,
    ) -> Result<DeviceEndpointDeleteObservation, FileMountError> {
        let () = tokio::task::yield_now().await;
        Ok(DeviceEndpointDeleteObservation {
            receiver: tokio::sync::oneshot::channel().1,
        })
    }

    /// Observes the creation of assets for a specific device and endpoint.
    ///
    /// # Arguments
    /// * `_device_endpoint_ref` - A reference to the device endpoint for which to observe asset creation.
    /// * `_timeout` - The duration to wait for the observation to complete.
    ///
    /// # Returns
    /// Returns OK([`AssetCreateObservation`]) if observation was success.
    /// The [`AssetCreateObservation`] can be used to receive notifications.
    ///
    /// # Errors
    /// Returns an error if the file mount cannot be accessed or if there is an issue with the watcher.
    pub async fn observe_asset_create(
        &self,
        _device_endpoint_ref: DeviceEndpointRef,
        _timeout: Duration,
    ) -> Result<AssetCreateObservation, FileMountError> {
        let () = tokio::task::yield_now().await;
        Ok(AssetCreateObservation {
            asset_creation_rx: tokio::sync::mpsc::unbounded_channel().1,
        })
    }

    /// Observes for the deletion of an asset.
    ///
    /// # Arguments
    /// * `_asset_ref` - A reference to the asset for which to observe deletion.
    /// * `_timeout` - The duration to wait for the observation to complete.
    ///
    /// # Returns
    /// Returns OK([`AssetDeleteObservation`]) if observation was success.
    /// The [`AssetDeleteObservation`] can be used to receive notifications.
    ///
    /// # Errors
    /// Returns an error if the file mount cannot be accessed or if there is an issue with the watcher.
    pub async fn observe_asset_delete(
        &self,
        _asset_ref: AssetRef,
        _timeout: Duration,
    ) -> Result<AssetDeleteObservation, FileMountError> {
        let () = tokio::task::yield_now().await;
        Ok(AssetDeleteObservation {
            receiver: tokio::sync::oneshot::channel().1,
        })
    }
}

/// Represents an observation for device endpoint creation events.
///
/// This struct contains an internal channel for receiving notifications
/// about newly created device endpoints.
// pub struct DeviceEndpointCreateObservation {
//     receiver: UnboundedReceiver<DeviceEndpointRef>,
// }

// impl DeviceEndpointCreateObservation {
//     /// Receives a notification for a newly created device endpoint.
//     ///
//     /// # Returns
//     /// An `Option` containing a `DeviceEndpointRef` if a notification is received, or `None` if the channel is closed.
//     pub async fn recv_notification(&mut self) -> Option<DeviceEndpointRef> {
//         self.receiver.recv().await
//     }
// }

/// Represents an observation for device endpoint deletion events.
///
/// This struct contains an internal channel for receiving notifications
/// about deleted device endpoints.
pub struct DeviceEndpointDeleteObservation {
    /// The internal channel for receiving notifications for an device deletion event.
    receiver: oneshot::Receiver<DeviceEndpointRef>,
}

impl DeviceEndpointDeleteObservation {
    /// Receives a notification for a deleted device endpoint.
    ///
    /// # Returns
    /// An `Option` containing a `DeviceEndpointRef` if a notification is received, or `None` if the channel is closed.
    pub async fn recv_notification(self) -> Option<DeviceEndpointRef> {
        self.receiver.await.ok()
    }
}

/// Represents a device and its associated endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceEndpointRef {
    /// The name of the device
    pub device_name: String,
    /// The name of the endpoint
    pub endpoint_name: String,
}

impl TryFrom<String> for DeviceEndpointRef {
    type Error = (); // FIN: Define a proper error type

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // The below assumes the format is always {device_name}_{endpoint_name} with no additional
        // `_` in the names.
        match value.split_once('_') {
            Some((device_name, endpoint_name)) => Ok(Self {
                device_name: device_name.to_string(),
                endpoint_name: endpoint_name.to_string(),
            }),
            None => todo!(), // FIN: Define a proper error handling
        }
    }
}

impl Display for DeviceEndpointRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.device_name, self.endpoint_name)
    }
}

/// Represents an observation for asset creation events.
///
/// This struct contains an internal channel for receiving notifications
/// about newly created assets.
pub struct AssetCreateObservation {
    /// The internal channel for receiving notifications for an asset creation event.
    asset_creation_rx: UnboundedReceiver<(AssetRef, AssetDeletionToken)>,
}

impl AssetCreateObservation {
    pub(crate) fn new(
        asset_creation_rx: UnboundedReceiver<(AssetRef, AssetDeletionToken)>,
    ) -> AssetCreateObservation {
        Self { asset_creation_rx }
    }

    /// Receives a notification for a newly created asset.
    ///
    /// # Returns
    /// An `Option` containing an `AssetRef` if a notification is received, or `None` if the channel is closed.
    pub async fn recv_notification(&mut self) -> Option<(AssetRef, AssetDeletionToken)> {
        self.asset_creation_rx.recv().await
    }
}

/// Represents an observation for asset deletion events.
///
/// This struct contains an internal channel for receiving notifications
/// about deleted assets.
pub struct AssetDeleteObservation {
    /// The internal channel for receiving notifications for an asset deletion event.
    receiver: oneshot::Receiver<AssetRef>,
}

impl AssetDeleteObservation {
    /// Receives a notification for a deleted asset.
    ///
    /// # Returns
    /// An `Option` containing an `AssetRef` if a notification is received, or `None` if the channel is closed.
    pub async fn recv_notification(self) -> Option<AssetRef> {
        self.receiver.await.ok()
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

/// FIN: Add documentation here
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
// FIN: Pr into feature/rust-akri
#[derive(Debug, thiserror::Error)]
/// Represents errors that can occur while interacting with the file mount.
pub enum FileMountError {
    #[error("Failed to access filesystem: {0}")]
    /// Error that occurs when accessing the filesystem.
    /// NOT retriable
    FilesystemError(#[from] std::io::Error),

    /// Error that occurs when there is an issue with the file watcher.
    /// retriable ??
    #[error("Watcher error: {0}")]
    NotifyError(#[from] notify::Error),

    /// Error that occurs when parsing file content fails.
    /// retriable
    #[error("Failed to parse file content: {0}")]
    ParseError(String),
    // Add other error variants as needed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use test_case::test_case;

    const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

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
            device_name: &str,
            endpoint_name: &str,
            asset_names: &[&str],
        ) {
            let file_name = format!("{}_{}", device_name, endpoint_name);
            let file_path = self.dir.path().join(&file_name);
            let content = asset_names.join(";");
            fs::write(file_path, content).unwrap();
        }

        fn remove_device_endpoint(&self, device_name: &str, endpoint_name: &str) {
            let file_name = format!("{}_{}", device_name, endpoint_name);
            let file_path = self.dir.path().join(&file_name);
            fs::remove_file(file_path).unwrap();
        }

        fn add_asset(&self, device_name: &str, endpoint_name: &str, asset_name: &str) {
            let file_name = format!("{}_{}", device_name, endpoint_name);
            let file_path = self.dir.path().join(&file_name);
            let mut content = fs::read_to_string(&file_path).unwrap();

            // Make sure the asset name is not already present
            if content.contains(asset_name) {
                return;
            }
            // Append the asset name to the file
            if !content.is_empty() {
                content.push(';');
            }
            content.push_str(asset_name);
            fs::write(file_path, content).unwrap();
        }

        fn remove_asset(&self, device_name: &str, endpoint_name: &str, asset_name: &str) {
            let file_name = format!("{}_{}", device_name, endpoint_name);
            let file_path = self.dir.path().join(&file_name);
            let mut content = fs::read_to_string(&file_path).unwrap();

            // Remove the asset name from the file
            content = content
                .split(';')
                .filter(|&name| name != asset_name)
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

        file_mount_manager.add_device_endpoint("device1", "endpoint1", &["asset1"]);
        file_mount_manager.add_device_endpoint("device1", "endpoint2", &["asset2", "asset3"]);
        file_mount_manager.add_device_endpoint("device2", "endpoint3", &["asset3"]);

        temp_env::with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            || {
                let mount_path = get_mount_path().unwrap();

                let device_endpoints = get_device_endpoint_names(mount_path.as_path()).unwrap();

                assert_eq!(device_endpoints.len(), 3);
                assert!(device_endpoints.contains(&DeviceEndpointRef {
                    device_name: "device1".to_string(),
                    endpoint_name: "endpoint1".to_string(),
                }));
                assert!(device_endpoints.contains(&DeviceEndpointRef {
                    device_name: "device1".to_string(),
                    endpoint_name: "endpoint2".to_string(),
                }));
                assert!(device_endpoints.contains(&DeviceEndpointRef {
                    device_name: "device2".to_string(),
                    endpoint_name: "endpoint3".to_string(),
                }));
            },
        )
    }

    #[test]
    fn test_get_asset_names() {
        let file_mount_manager = TempFileMountManager::new("test_mount");
        file_mount_manager.add_device_endpoint(
            "device1",
            "endpoint1",
            &["asset1", "asset2", "asset3"],
        );

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

                assert_eq!(device1_endpoint1_assets.len(), 3);
                assert!(device1_endpoint1_assets.contains(&AssetRef {
                    name: "asset1".to_string(),
                    device_name: "device1".to_string(),
                    endpoint_name: "endpoint1".to_string(),
                }));
                assert!(device1_endpoint1_assets.contains(&AssetRef {
                    name: "asset2".to_string(),
                    device_name: "device1".to_string(),
                    endpoint_name: "endpoint1".to_string(),
                }));
                assert!(device1_endpoint1_assets.contains(&AssetRef {
                    name: "asset3".to_string(),
                    device_name: "device1".to_string(),
                    endpoint_name: "endpoint1".to_string(),
                }));
            },
        )
    }

    #[tokio::test]
    async fn test_device_endpoint_create_observation_pre_mounted_success() {
        let file_mount_manager = TempFileMountManager::new("test_mount");

        file_mount_manager.add_device_endpoint("device1", "endpoint1", &Vec::new());
        file_mount_manager.add_device_endpoint("device1", "endpoint2", &Vec::new());
        file_mount_manager.add_device_endpoint("device2", "endpoint3", &Vec::new());

        temp_env::async_with_vars(
            [(
                ADR_RESOURCES_NAME_MOUNT_PATH,
                Some(file_mount_manager.path()),
            )],
            async {
                let mut test_device_endpoint_create_observation =
                    DeviceEndpointCreateObservation::new_device_endpoint_create_observation(
                        DEBOUNCE_DURATION,
                    )
                    .unwrap();
        
                tokio::select! {
                    Some((device_endpoint, _)) = test_device_endpoint_create_observation.recv_notification() => {
                        assert_eq!(device_endpoint.device_name, "device1");
                        assert_eq!(device_endpoint.endpoint_name, "endpoint1");
                    },
                    _ = tokio::time::sleep(DEBOUNCE_DURATION * 2) => {
                        panic!("Failed to receive device endpoint creation notification");
                    }
                };
            },
        )
        .await
    }
}
