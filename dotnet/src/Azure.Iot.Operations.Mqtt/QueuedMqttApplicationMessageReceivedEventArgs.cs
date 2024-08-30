﻿using Azure.Iot.Operations.Protocol;
using MQTTnet.Client;

namespace Azure.Iot.Operations.Mqtt
{
    internal class QueuedMqttApplicationMessageReceivedEventArgs : IDelayableQueueItem
    {
        private bool _manuallyAcknowledged;

        public MqttApplicationMessageReceivedEventArgs Args { get; }

        public QueuedMqttApplicationMessageReceivedEventArgs(MqttApplicationMessageReceivedEventArgs args)
        {
            Args = args;
        }

        public bool IsReady()
        {
            return _manuallyAcknowledged || Args.AutoAcknowledge;
        }

        public void MarkAsReady()
        {
            _manuallyAcknowledged = true;
        }
    }
}