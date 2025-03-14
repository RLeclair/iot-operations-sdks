/* Code generated by Azure.Iot.Operations.ProtocolCompiler v0.9.0.0; DO NOT EDIT. */

#nullable enable

namespace SampleCloudEvents.Oven
{
    using System.Collections.Generic;
    using Azure.Iot.Operations.Protocol;
    using Azure.Iot.Operations.Protocol.Telemetry;
    using Azure.Iot.Operations.Protocol.Models;
    using SampleCloudEvents;

    public static partial class Oven
    {
        /// <summary>
        /// Specializes the <c>TelemetrySender</c> class for type <c>TelemetryCollection</c>.
        /// </summary>
        public class TelemetrySender : TelemetrySender<TelemetryCollection>
        {
            /// <summary>
            /// Initializes a new instance of the <see cref="TelemetrySender"/> class.
            /// </summary>
            public TelemetrySender(ApplicationContext applicationContext, IMqttPubSubClient mqttClient)
                : base(applicationContext, mqttClient, new Utf8JsonSerializer())
            {
                TopicTokenMap["modelId"] = "dtmi:akri:samples:oven;1";
                if (mqttClient.ClientId != null)
                {
                    TopicTokenMap["senderId"] = mqttClient.ClientId;
                }
            }
        }
    }
}
