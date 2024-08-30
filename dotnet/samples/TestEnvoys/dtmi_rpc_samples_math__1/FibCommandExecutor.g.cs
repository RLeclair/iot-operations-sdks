/* This is an auto-generated file.  Do not modify. */

#nullable enable

namespace TestEnvoys.dtmi_rpc_samples_math__1
{
    using System;
    using System.Collections.Generic;
    using System.Linq;
    using System.Xml;
    using Azure.Iot.Operations.Protocol;
    using Azure.Iot.Operations.Protocol.RPC;
    using Azure.Iot.Operations.Protocol.Models;
    using TestEnvoys;

    public static partial class Math
    {
        /// <summary>
        /// Specializes a <c>CommandExecutor</c> class for Command 'fib'.
        /// </summary>
        [CommandBehavior(idempotent: true, cacheableDuration: "PT10M")]
        public class FibCommandExecutor : CommandExecutor<FibCommandRequest, FibCommandResponse>
        {
            /// <summary>
            /// Initializes a new instance of the <see cref="FibCommandExecutor"/> class.
            /// </summary>
            internal FibCommandExecutor(IMqttPubSubClient mqttClient)
                : base(mqttClient, "fib", new ProtobufSerializer<FibCommandRequest, FibCommandResponse>())
            {
            }
        }
    }
}
