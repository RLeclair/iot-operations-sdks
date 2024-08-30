﻿#nullable enable

using Azure.Iot.Operations.Protocol;
using Azure.Iot.Operations.Protocol.RPC;

namespace TestEnvoys.Greeter;

public class GreeterEnvoy
{

    public class HelloRequest
    {
        public string Name { get; set; } = string.Empty;
    }

    public class HelloWithDelayRequest : HelloRequest
    {
        public TimeSpan Delay { get; set; } = TimeSpan.Zero;
    }

    public class HelloResponse
    {
        public string Message { get; set; } = string.Empty;
    }

    [CommandTopic("rpc/samples/hello")]
    public class SayHelloCommandExecutor : CommandExecutor<HelloRequest, HelloResponse>
    {
        public SayHelloCommandExecutor(IMqttPubSubClient mqttClient)
            : base(mqttClient, "sayHello", new Utf8JsonSerializer())
        {
        }
    }

    [CommandTopic("rpc/samples/hello/delay")]
    public class SayHelloWithDelayCommandExecutor : CommandExecutor<HelloWithDelayRequest, HelloResponse>
    {
        public SayHelloWithDelayCommandExecutor(IMqttPubSubClient mqttClient)
            : base(mqttClient, "sayHelloWithDelay", new Utf8JsonSerializer())
        {
            IsIdempotent = true;
            CacheableDuration = TimeSpan.FromSeconds(10);
            ExecutionTimeout = TimeSpan.FromSeconds(30);
        }
    }

    public abstract class Service : IAsyncDisposable
    {
        readonly SayHelloCommandExecutor sayHelloExecutor;
        readonly SayHelloWithDelayCommandExecutor sayHelloWithDelayExecutor;
        public Service(IMqttPubSubClient mqttClient)
        {
            sayHelloExecutor = new SayHelloCommandExecutor(mqttClient)
            {
                OnCommandReceived = SayHello,
            };

            sayHelloWithDelayExecutor = new SayHelloWithDelayCommandExecutor(mqttClient)
            {
                OnCommandReceived = SayHelloWithDelayAsync,
            };
        }

        public SayHelloCommandExecutor SayHelloCommandExecutor { get => sayHelloExecutor; }
        public SayHelloWithDelayCommandExecutor SayHelloWithDelayCommandExecutor { get => sayHelloWithDelayExecutor; }

        public async ValueTask DisposeAsync()
        {
            await SayHelloCommandExecutor.DisposeAsync();
            await SayHelloWithDelayCommandExecutor.DisposeAsync();
        }

        public async ValueTask DisposeAsync(bool disposing)
        {
            await SayHelloCommandExecutor.DisposeAsync(disposing);
            await SayHelloWithDelayCommandExecutor.DisposeAsync(disposing);
        }

        public abstract Task<ExtendedResponse<HelloResponse>> SayHello(ExtendedRequest<HelloRequest> request, CancellationToken cancellationToken);
        public abstract Task<ExtendedResponse<HelloResponse>> SayHelloWithDelayAsync(ExtendedRequest<HelloWithDelayRequest> request, CancellationToken cancellationToken);

        public async Task StartAsync(int? preferredDispatchConcurrency = null, CancellationToken cancellationToken = default)
        {
            await sayHelloExecutor.StartAsync(preferredDispatchConcurrency, cancellationToken);
            await sayHelloWithDelayExecutor.StartAsync(preferredDispatchConcurrency, cancellationToken);
        }

        public Task StopAsync(CancellationToken cancellationToken) => throw new NotImplementedException();

        public override string ToString()
        {
            return $"Service {nameof(GreeterEnvoy)} {Environment.NewLine}" +
                    $"\tSayHello: Idempotent {sayHelloExecutor.IsIdempotent} CacheDuration {sayHelloExecutor.CacheableDuration} {Environment.NewLine}" +
                    $"\tSayHelloWithDelay: Idempotent {sayHelloWithDelayExecutor.IsIdempotent} CacheDuration {sayHelloWithDelayExecutor.CacheableDuration} {Environment.NewLine}";

        }
    }

    [CommandTopic("rpc/samples/hello")]
    public class SayHelloCommandInvoker : CommandInvoker<HelloRequest, HelloResponse>
    {
        public SayHelloCommandInvoker(IMqttPubSubClient mqttClient)
            : base(mqttClient, "sayHello", new Utf8JsonSerializer())
        {
        }
    }

    [CommandTopic("rpc/samples/hello/delay")]
    public class SayHelloWithDelayCommandInvoker : CommandInvoker<HelloWithDelayRequest, HelloResponse>
    {
        public SayHelloWithDelayCommandInvoker(IMqttPubSubClient mqttClient)
            : base(mqttClient, "sayHelloWithDelay", new Utf8JsonSerializer())
        {
        }
    }

    public class Client : IAsyncDisposable
    {
        readonly SayHelloCommandInvoker sayHelloInvoker;
        readonly SayHelloWithDelayCommandInvoker sayHelloWithDelayInvoker;

        public Client(IMqttPubSubClient mqttClient)
        {
            sayHelloInvoker = new SayHelloCommandInvoker(mqttClient);
            sayHelloWithDelayInvoker = new SayHelloWithDelayCommandInvoker(mqttClient);
        }

        public SayHelloCommandInvoker SayHelloCommandInvoker { get => sayHelloInvoker; }
        public SayHelloWithDelayCommandInvoker SayHelloWithDelayCommandInvoker { get => sayHelloWithDelayInvoker; }

        public RpcCallAsync<HelloResponse> SayHello(ExtendedRequest<HelloRequest> request, CommandRequestMetadata? md = default, TimeSpan? timeout = default)
        {
            CommandRequestMetadata metadata = md == default ? new CommandRequestMetadata() : md;
            return new RpcCallAsync<HelloResponse>(sayHelloInvoker.InvokeCommandAsync(string.Empty, request.Request, metadata, timeout), metadata.CorrelationId);
        }

        public RpcCallAsync<HelloResponse> SayHelloWithDelay(ExtendedRequest<HelloWithDelayRequest> request, TimeSpan? timeout = default)
        {
            CommandRequestMetadata metadata = new CommandRequestMetadata();
            return new RpcCallAsync<HelloResponse>(sayHelloWithDelayInvoker.InvokeCommandAsync(string.Empty, request.Request, metadata, timeout), metadata.CorrelationId);
        }

        public async ValueTask DisposeAsync()
        {
            GC.SuppressFinalize(this);
            await sayHelloInvoker.DisposeAsync();
            await sayHelloWithDelayInvoker.DisposeAsync();
        }
    }
}
