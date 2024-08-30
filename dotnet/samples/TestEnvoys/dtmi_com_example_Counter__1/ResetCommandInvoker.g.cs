/* This is an auto-generated file.  Do not modify. */

#nullable enable

namespace TestEnvoys.dtmi_com_example_Counter__1
{
    using System;
    using System.Collections.Generic;
    using Azure.Iot.Operations.Protocol;
    using Azure.Iot.Operations.Protocol.RPC;
    using Azure.Iot.Operations.Protocol.Models;
    using TestEnvoys;

    public static partial class Counter
    {
        /// <summary>
        /// Specializes the <c>CommandInvoker</c> class for Command 'reset'.
        /// </summary>
        public class ResetCommandInvoker : CommandInvoker<EmptyJson, EmptyJson>
        {
            /// <summary>
            /// Initializes a new instance of the <see cref="ResetCommandInvoker"/> class.
            /// </summary>
            internal ResetCommandInvoker(IMqttPubSubClient mqttClient)
                : base(mqttClient, "reset", new Utf8JsonSerializer())
            {
            }
        }
    }
}
