// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

using System.Collections.Generic;

namespace Azure.Iot.Operations.Protocol.Models
{
    public class MqttClientUnsubscribeOptions
    {
        public MqttClientUnsubscribeOptions()
        {
        }

        public MqttClientUnsubscribeOptions(MqttTopicFilter mqttTopicFilter)
        {
            TopicFilters.Add(mqttTopicFilter.Topic);
        }

        public MqttClientUnsubscribeOptions(string topic)
        {
            TopicFilters.Add(topic);
        }

        /// <summary>
        ///     Gets or sets a list of topic filters the client wants to unsubscribe from.
        ///     Topic filters can include regular topics or wild cards.
        /// </summary>
        public List<string> TopicFilters { get; set; } = [];

        /// <summary>
        ///     Gets or sets the user properties.
        ///     In MQTT 5, user properties are basic UTF-8 string key-value pairs that you can append to almost every type of MQTT
        ///     packet.
        ///     As long as you don’t exceed the maximum message size, you can use an unlimited number of user properties to add
        ///     metadata to MQTT messages and pass information between publisher, broker, and subscriber.
        ///     The feature is very similar to the HTTP header concept.
        ///     <remarks>MQTT 5.0.0+ feature.</remarks>
        /// </summary>
        public List<MqttUserProperty>? UserProperties { get; set; }

        public void AddUserProperty(string key, string value)
        {
            UserProperties ??= [];
            UserProperties.Add(new MqttUserProperty(key, value));
        }
    }
}