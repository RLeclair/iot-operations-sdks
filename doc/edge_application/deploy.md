# Deploy an edge application

The following instruction outline how to create a container image for your application, push it to the a registry, and deploy to the cluster.

* [Container types](#container-types)
* [.NET Dockerfile](#net-dockerfile)
* [Rust Dockerfile](#rust-dockerfile)
* [Go Dockerfile](#go-dockerfile)
* [Build the container image](#build-the-container-image)
* [Import the container image](#import-the-container-image)
* [Deploying to the cluster](#deploying-to-the-cluster)

## Container types

Some languages have built in container support, however all binaries can be containerized using a Dockerfile. A Dockerfile can be created to support both building the project and the creating the deployable container for repeatable container creation by using [multi-stage builds](https://docs.docker.com/build/building/multi-stage/).

> [!NOTE]
> The Dockerfile examples below are for reference only, and will need to be adapted for your project

## .NET Dockerfile

> [!NOTE]
> 
> Refer to [Containerize a .NET app](https://learn.microsoft.com/dotnet/core/docker/build-container) for details on building and creating a container directly from a .NET project.

This definition uses the [Official .NET SDK image](https://github.com/dotnet/dotnet-docker/blob/main/README.sdk.md) to build the application, and then copies the resulting package into the [.NET runtime image](https://hub.docker.com/_/alpine).

```dockerfile
# Build application
FROM mcr.microsoft.com/dotnet/sdk:9.0 AS build
WORKDIR /build
COPY . .
RUN dotnet publish -o dist

# Build runtime image
FROM mcr.microsoft.com/dotnet/runtime:9.0
WORKDIR /app
COPY --from=build /build/dist .
ENTRYPOINT ["./MyApplication"]
```

## Rust Dockerfile

This definition uses the [Official Rust image](https://hub.docker.com/_/rust) to build the release application, and then copies the resulting binary (called "rust-application" in the example below) into the [Alpine image](https://hub.docker.com/_/alpine).

```dockerfile
# Build application
FROM rust:1 AS build
WORKDIR /build
COPY . .
RUN cargo build

# Build runtime image
FROM debian:bookworm-slim
WORKDIR /
RUN apt update; apt install -y libssl3
COPY --from=build work/rust-application .
ENTRYPOINT ["./rust-application"]
```

## Go Dockerfile

This definition uses the [Official Golang image](https://hub.docker.com/_/golang) to build the application, and then copies the resulting binary (called "go-application" in the example below) into the [Alpine image](https://hub.docker.com/_/alpine).

```dockerfile
# Build application
FROM golang:1 AS build
WORKDIR /build
COPY . .
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o .

# Build runtime image
FROM debian:bookworm-slim
WORKDIR /
COPY --from=build /build/go-application .
ENTRYPOINT ["./go-application"]
```

## Build the container image

1. Create a file called `Dockerfile` in your application directory, and define the container *(using above as a reference)*.

1. Build the container image, substituting "sample" with your own tag name as needed:

    ```bash
    docker build --tag sample .
    ```

1. The resulting binary will be available 

    ```bash
    docker image ls sample
    ```

    ```output
    REPOSITORY   TAG       IMAGE ID       CREATED          SIZE
    sample       latest    7b76808165ed   10 minutes ago   73.9MB
    ```

## Import the container image

Instead of uploading the container image to a container registry, you can choose to import the image directly into the k3d cluster using the [k3d image import](https://k3d.io/v5.1.0/usage/commands/k3d_image_import/) command. This avoids the need to use a container registry, and is ideal for developing locally.

```bash
k3d image import <image-name>
```

> [!TIP]
> If using the k3d import method described here, then make sure the `imagePullPolicy` in the container definition is set to `Never`, otherwise the cluster will attempt to download the image.

## Deploying to the cluster

The following yaml can be used as a reference for deploying your application to the cluster.

The Deployment contains the following information:

| Type | Value | Description |
|-|-|-|
| ServiceAccountName | `mqtt-client` | The service account to generate the auth token from |
| Volume | `mqtt-client-token` | The SAT for mounting into the counter |
| Volume | `aio-ca-trust-bundle` | The broker trust-bundle for validating the server |
| Container | `sdk-application` | The definition of the container, including the container image and the mount locations for the SAT and broker trust-bundle |
| Container.image | <image-name> | The container image to be deployed |
| Container.imagePullPolicy | `Never` | The pull policy can be set to `Never` to allow cached images to be used.
| Container.env | `MQTT_*`, `AIO_*` | The environment variables used to configure the connection to the MQTT broker. Refer to [MQTT broker access](/doc/setup.md#mqtt-broker-access) for details on settings these values to match the development environment |

1. Create a file called `app.yaml` containing the following:

    ```yaml
    apiVersion: apps/v1
    kind: Deployment
    metadata:
      name: sdk-application
      namespace: azure-iot-operations
    spec:
      selector:
        matchLabels:
          app: sdk-application
      template:
        metadata:
          labels:
            app: sdk-application
        spec:
          serviceAccountName: mqtt-client   # The service account to use for generating the token
          volumes:
            - name: mqtt-client-token
              projected:
                sources:
                  - serviceAccountToken:
                      path: mqtt-client-token
                      audience: aio-internal
                      expirationSeconds: 86400

            - name: aio-ca-trust-bundle
              configMap:
                name: azure-iot-operations-aio-ca-trust-bundle

          containers:
            - name: sdk-application
              image: <image-name>       # Use the container image name previously created
              imagePullPolicy: Never    # Set to Never to use the imported image

              volumeMounts:
                - name: mqtt-client-token
                  mountPath: /var/run/secrets/tokens
                - name: aio-ca-trust-bundle
                  mountPath: /var/run/certs/aio-ca

              env:
                - name: MQTT_CLIENT_ID
                  value: "<client-id>"  # Replace this with the MQTT client id of your choosing
                - name: AIO_BROKER_HOSTNAME
                  value: "aio-broker"
                - name: AIO_BROKER_TCP_PORT
                  value: "18883"
                - name: AIO_MQTT_USE_TLS
                  value: "true"
                - name: AIO_TLS_CA_FILE
                  value: "/var/run/certs/aio-ca/ca.crt"
                - name: AIO_SAT_FILE
                  value: "/var/run/secrets/tokens/mqtt-client-token"
    ```

1. Apply the yaml to the cluster:

    ```bash
    kubectl apply -f app.yml
    ```
