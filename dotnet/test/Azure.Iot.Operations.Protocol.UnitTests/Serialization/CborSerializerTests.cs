﻿// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

using System.Buffers;
using Azure.Iot.Operations.Protocol.UnitTests.Serializers.CBOR;

namespace Azure.Iot.Operations.Protocol.UnitTests.Serialization
{
    public class MyCborType
    {
        [Dahomey.Cbor.Attributes.CborPropertyAttribute(index: 1)]
        public int MyIntProperty { get; set; }
        [Dahomey.Cbor.Attributes.CborPropertyAttribute(index: 2)]
        public string MyStringProperty { get; set; } = string.Empty;
    }

    public class CborSerializerTests
    {
        [Fact]
        public void CborUsesFormatIndicatorAsZero()
        {
            Assert.Equal(Models.MqttPayloadFormatIndicator.Unspecified, CborSerializer.PayloadFormatIndicator);
        }

        [Fact]
        public void DeserializeEmtpy()
        {
            IPayloadSerializer cborSerializer = new CborSerializer();

            ReadOnlySequence<byte> emptyBytes = cborSerializer.ToBytes(new EmptyCbor()).SerializedPayload;
            Assert.True(emptyBytes.IsEmpty);
            EmptyCbor? empty = cborSerializer.FromBytes<EmptyCbor>(emptyBytes, null, Models.MqttPayloadFormatIndicator.Unspecified);
            Assert.NotNull(empty);
        }

        [Fact]
        public void DeserializeNullToNonEmptyThrows()
        {
            IPayloadSerializer cborSerializer = new CborSerializer();

            Assert.Throws<AkriMqttException>(() => { cborSerializer.FromBytes<MyCborType>(ReadOnlySequence<byte>.Empty, null, Models.MqttPayloadFormatIndicator.Unspecified); });
        }
    }
}
