/* Code generated by Azure.Iot.Operations.ProtocolCompiler v0.10.0.0; DO NOT EDIT. */

#nullable enable

namespace Azure.Iot.Operations.Services.Akri.DiscoveredAssetResources
{
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using System.Threading;
    using System.Threading.Tasks;
    using Azure.Iot.Operations.Protocol.Models;
    using Azure.Iot.Operations.Protocol;
    using Azure.Iot.Operations.Protocol.RPC;
    using Azure.Iot.Operations.Protocol.Telemetry;
    using Azure.Iot.Operations.Services.Akri;

    [CommandTopic("akri/discovery/{modelId}/{invokerClientId}/command/{commandName}")]
    [System.CodeDom.Compiler.GeneratedCode("Azure.Iot.Operations.ProtocolCompiler", "0.10.0.0")]
    public static partial class DiscoveredAssetResources
    {
        [ServiceGroupId("MyServiceGroup")]
        public abstract partial class Service : IAsyncDisposable
        {
            private ApplicationContext applicationContext;
            private IMqttPubSubClient mqttClient;
            private readonly CreateDiscoveredAssetEndpointProfileCommandExecutor createDiscoveredAssetEndpointProfileCommandExecutor;
            private readonly CreateDiscoveredAssetCommandExecutor createDiscoveredAssetCommandExecutor;

            /// <summary>
            /// Construct a new instance of this service.
            /// </summary>
            /// <param name="applicationContext">The shared context for your application.</param>
            /// <param name="mqttClient">The MQTT client to use.</param>
            /// <param name="topicTokenMap">
            /// The topic token replacement map to use for all operations by default. Generally, this will include the token values
            /// for topic tokens such as "modelId" which should be the same for the duration of this service's lifetime. Note that
            /// additional topic tokens can be specified per-telemetry message.
            /// </param>
            public Service(ApplicationContext applicationContext, IMqttPubSubClient mqttClient, Dictionary<string, string>? topicTokenMap = null)
            {
                this.applicationContext = applicationContext;
                this.mqttClient = mqttClient;

                string? clientId = this.mqttClient.ClientId;
                if (string.IsNullOrEmpty(clientId))
                {
                    throw new InvalidOperationException("No MQTT client Id configured. Must connect to MQTT broker before invoking command.");
                }

                this.createDiscoveredAssetEndpointProfileCommandExecutor = new CreateDiscoveredAssetEndpointProfileCommandExecutor(applicationContext, mqttClient) { OnCommandReceived = CreateDiscoveredAssetEndpointProfileInt };
                this.createDiscoveredAssetCommandExecutor = new CreateDiscoveredAssetCommandExecutor(applicationContext, mqttClient) { OnCommandReceived = CreateDiscoveredAssetInt };

                if (topicTokenMap != null)
                {
                    foreach (string topicTokenKey in topicTokenMap.Keys)
                    {
                        this.createDiscoveredAssetEndpointProfileCommandExecutor.TopicTokenMap.TryAdd("ex:" + topicTokenKey, topicTokenMap[topicTokenKey]);
                        this.createDiscoveredAssetCommandExecutor.TopicTokenMap.TryAdd("ex:" + topicTokenKey, topicTokenMap[topicTokenKey]);
                    }
                }

                this.createDiscoveredAssetEndpointProfileCommandExecutor.TopicTokenMap.TryAdd("executorId", clientId);
                this.createDiscoveredAssetCommandExecutor.TopicTokenMap.TryAdd("executorId", clientId);
            }

            public CreateDiscoveredAssetEndpointProfileCommandExecutor CreateDiscoveredAssetEndpointProfileCommandExecutor { get => this.createDiscoveredAssetEndpointProfileCommandExecutor; }

            public CreateDiscoveredAssetCommandExecutor CreateDiscoveredAssetCommandExecutor { get => this.createDiscoveredAssetCommandExecutor; }

            public abstract Task<ExtendedResponse<CreateDiscoveredAssetEndpointProfileResponsePayload>> CreateDiscoveredAssetEndpointProfileAsync(CreateDiscoveredAssetEndpointProfileRequestPayload request, CommandRequestMetadata requestMetadata, CancellationToken cancellationToken);

            public abstract Task<ExtendedResponse<CreateDiscoveredAssetResponsePayload>> CreateDiscoveredAssetAsync(CreateDiscoveredAssetRequestPayload request, CommandRequestMetadata requestMetadata, CancellationToken cancellationToken);

            /// <summary>
            /// Begin accepting command invocations for all command executors.
            /// </summary>
            /// <param name="preferredDispatchConcurrency">The dispatch concurrency count for the command response cache to use.</param>
            /// <param name="cancellationToken">Cancellation token.</param>
            public async Task StartAsync(int? preferredDispatchConcurrency = null, CancellationToken cancellationToken = default)
            {
                string? clientId = this.mqttClient.ClientId;
                if (string.IsNullOrEmpty(clientId))
                {
                    throw new InvalidOperationException("No MQTT client Id configured. Must connect to MQTT broker before starting service.");
                }

                await Task.WhenAll(
                    this.createDiscoveredAssetEndpointProfileCommandExecutor.StartAsync(preferredDispatchConcurrency, cancellationToken),
                    this.createDiscoveredAssetCommandExecutor.StartAsync(preferredDispatchConcurrency, cancellationToken)).ConfigureAwait(false);
            }

            public async Task StopAsync(CancellationToken cancellationToken = default)
            {
                await Task.WhenAll(
                    this.createDiscoveredAssetEndpointProfileCommandExecutor.StopAsync(cancellationToken),
                    this.createDiscoveredAssetCommandExecutor.StopAsync(cancellationToken)).ConfigureAwait(false);
            }

            private async Task<ExtendedResponse<CreateDiscoveredAssetEndpointProfileResponsePayload>> CreateDiscoveredAssetEndpointProfileInt(ExtendedRequest<CreateDiscoveredAssetEndpointProfileRequestPayload> req, CancellationToken cancellationToken)
            {
                ExtendedResponse<CreateDiscoveredAssetEndpointProfileResponsePayload> extended = await this.CreateDiscoveredAssetEndpointProfileAsync(req.Request!, req.RequestMetadata!, cancellationToken);
                return new ExtendedResponse<CreateDiscoveredAssetEndpointProfileResponsePayload> { Response = extended.Response, ResponseMetadata = extended.ResponseMetadata };
            }

            private async Task<ExtendedResponse<CreateDiscoveredAssetResponsePayload>> CreateDiscoveredAssetInt(ExtendedRequest<CreateDiscoveredAssetRequestPayload> req, CancellationToken cancellationToken)
            {
                ExtendedResponse<CreateDiscoveredAssetResponsePayload> extended = await this.CreateDiscoveredAssetAsync(req.Request!, req.RequestMetadata!, cancellationToken);
                return new ExtendedResponse<CreateDiscoveredAssetResponsePayload> { Response = extended.Response, ResponseMetadata = extended.ResponseMetadata };
            }

            public async ValueTask DisposeAsync()
            {
                await this.createDiscoveredAssetEndpointProfileCommandExecutor.DisposeAsync().ConfigureAwait(false);
                await this.createDiscoveredAssetCommandExecutor.DisposeAsync().ConfigureAwait(false);
            }

            public async ValueTask DisposeAsync(bool disposing)
            {
                await this.createDiscoveredAssetEndpointProfileCommandExecutor.DisposeAsync(disposing).ConfigureAwait(false);
                await this.createDiscoveredAssetCommandExecutor.DisposeAsync(disposing).ConfigureAwait(false);
            }
        }

        public abstract partial class Client : IAsyncDisposable
        {
            private ApplicationContext applicationContext;
            private IMqttPubSubClient mqttClient;
            private readonly CreateDiscoveredAssetEndpointProfileCommandInvoker createDiscoveredAssetEndpointProfileCommandInvoker;
            private readonly CreateDiscoveredAssetCommandInvoker createDiscoveredAssetCommandInvoker;

            /// <summary>
            /// Construct a new instance of this client.
            /// </summary>
            /// <param name="applicationContext">The shared context for your application.</param>
            /// <param name="mqttClient">The MQTT client to use.</param>
            /// <param name="topicTokenMap">
            /// The topic token replacement map to use for all operations by default. Generally, this will include the token values
            /// for topic tokens such as "modelId" which should be the same for the duration of this client's lifetime.
            /// </param>
            public Client(ApplicationContext applicationContext, IMqttPubSubClient mqttClient, Dictionary<string, string>? topicTokenMap = null)
            {
                this.applicationContext = applicationContext;
                this.mqttClient = mqttClient;

                this.createDiscoveredAssetEndpointProfileCommandInvoker = new CreateDiscoveredAssetEndpointProfileCommandInvoker(applicationContext, mqttClient);
                if (topicTokenMap != null)
                {
                    foreach (string topicTokenKey in topicTokenMap.Keys)
                    {
                        this.createDiscoveredAssetEndpointProfileCommandInvoker.TopicTokenMap.TryAdd("ex:" + topicTokenKey, topicTokenMap[topicTokenKey]);
                    }
                }
                this.createDiscoveredAssetCommandInvoker = new CreateDiscoveredAssetCommandInvoker(applicationContext, mqttClient);
                if (topicTokenMap != null)
                {
                    foreach (string topicTokenKey in topicTokenMap.Keys)
                    {
                        this.createDiscoveredAssetCommandInvoker.TopicTokenMap.TryAdd("ex:" + topicTokenKey, topicTokenMap[topicTokenKey]);
                    }
                }
            }

            public CreateDiscoveredAssetEndpointProfileCommandInvoker CreateDiscoveredAssetEndpointProfileCommandInvoker { get => this.createDiscoveredAssetEndpointProfileCommandInvoker; }

            public CreateDiscoveredAssetCommandInvoker CreateDiscoveredAssetCommandInvoker { get => this.createDiscoveredAssetCommandInvoker; }

            /// <summary>
            /// Invoke a command.
            /// </summary>
            /// <param name="request">The data for this command request.</param>
            /// <param name="requestMetadata">The metadata for this command request.</param>
            /// <param name="additionalTopicTokenMap">
            /// The topic token replacement map to use in addition to the topic tokens specified in the constructor. If this map
            /// contains any keys that the topic tokens specified in the constructor also has, then values specified in this map will take precedence.
            /// </param>
            /// <param name="commandTimeout">How long the command will be available on the broker for an executor to receive.</param>
            /// <param name="cancellationToken">Cancellation token.</param>
            /// <returns>The command response.</returns>
            public RpcCallAsync<CreateDiscoveredAssetEndpointProfileResponsePayload> CreateDiscoveredAssetEndpointProfileAsync(CreateDiscoveredAssetEndpointProfileRequestPayload request, CommandRequestMetadata? requestMetadata = null, Dictionary<string, string>? additionalTopicTokenMap = null, TimeSpan? commandTimeout = default, CancellationToken cancellationToken = default)
            {
                string? clientId = this.mqttClient.ClientId;
                if (string.IsNullOrEmpty(clientId))
                {
                    throw new InvalidOperationException("No MQTT client Id configured. Must connect to MQTT broker before invoking command.");
                }

                CommandRequestMetadata metadata = requestMetadata ?? new CommandRequestMetadata();
                additionalTopicTokenMap ??= new();

                Dictionary<string, string> prefixedAdditionalTopicTokenMap = new();
                foreach (string key in additionalTopicTokenMap.Keys)
                {
                    prefixedAdditionalTopicTokenMap["ex:" + key] = additionalTopicTokenMap[key];
                }

                prefixedAdditionalTopicTokenMap["invokerClientId"] = clientId;

                return new RpcCallAsync<CreateDiscoveredAssetEndpointProfileResponsePayload>(this.createDiscoveredAssetEndpointProfileCommandInvoker.InvokeCommandAsync(request, metadata, prefixedAdditionalTopicTokenMap, commandTimeout, cancellationToken), metadata.CorrelationId);
            }

            /// <summary>
            /// Invoke a command.
            /// </summary>
            /// <param name="request">The data for this command request.</param>
            /// <param name="requestMetadata">The metadata for this command request.</param>
            /// <param name="additionalTopicTokenMap">
            /// The topic token replacement map to use in addition to the topic tokens specified in the constructor. If this map
            /// contains any keys that the topic tokens specified in the constructor also has, then values specified in this map will take precedence.
            /// </param>
            /// <param name="commandTimeout">How long the command will be available on the broker for an executor to receive.</param>
            /// <param name="cancellationToken">Cancellation token.</param>
            /// <returns>The command response.</returns>
            public RpcCallAsync<CreateDiscoveredAssetResponsePayload> CreateDiscoveredAssetAsync(CreateDiscoveredAssetRequestPayload request, CommandRequestMetadata? requestMetadata = null, Dictionary<string, string>? additionalTopicTokenMap = null, TimeSpan? commandTimeout = default, CancellationToken cancellationToken = default)
            {
                string? clientId = this.mqttClient.ClientId;
                if (string.IsNullOrEmpty(clientId))
                {
                    throw new InvalidOperationException("No MQTT client Id configured. Must connect to MQTT broker before invoking command.");
                }

                CommandRequestMetadata metadata = requestMetadata ?? new CommandRequestMetadata();
                additionalTopicTokenMap ??= new();

                Dictionary<string, string> prefixedAdditionalTopicTokenMap = new();
                foreach (string key in additionalTopicTokenMap.Keys)
                {
                    prefixedAdditionalTopicTokenMap["ex:" + key] = additionalTopicTokenMap[key];
                }

                prefixedAdditionalTopicTokenMap["invokerClientId"] = clientId;

                return new RpcCallAsync<CreateDiscoveredAssetResponsePayload>(this.createDiscoveredAssetCommandInvoker.InvokeCommandAsync(request, metadata, prefixedAdditionalTopicTokenMap, commandTimeout, cancellationToken), metadata.CorrelationId);
            }

            public async ValueTask DisposeAsync()
            {
                await this.createDiscoveredAssetEndpointProfileCommandInvoker.DisposeAsync().ConfigureAwait(false);
                await this.createDiscoveredAssetCommandInvoker.DisposeAsync().ConfigureAwait(false);
            }

            public async ValueTask DisposeAsync(bool disposing)
            {
                await this.createDiscoveredAssetEndpointProfileCommandInvoker.DisposeAsync(disposing).ConfigureAwait(false);
                await this.createDiscoveredAssetCommandInvoker.DisposeAsync(disposing).ConfigureAwait(false);
            }
        }
    }
}
