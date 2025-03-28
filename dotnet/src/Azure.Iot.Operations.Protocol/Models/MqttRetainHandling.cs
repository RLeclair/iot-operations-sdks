// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Azure.Iot.Operations.Protocol.Models
{
    public enum MqttRetainHandling
    {
        SendAtSubscribe = 0,

        SendAtSubscribeIfNewSubscriptionOnly = 1,

        DoNotSendOnSubscribe = 2
    }
}
