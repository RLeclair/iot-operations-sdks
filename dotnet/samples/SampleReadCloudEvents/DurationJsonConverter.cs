namespace SampleReadCloudEvents
{
    using System;
    using System.Text.Json;
    using System.Text.Json.Serialization;
    using System.Xml;

    /// <summary>
    /// Class for customized JSON conversion of <c>TimeSpan</c> values to/from string representations in ISO 8601 Duration format.
    /// </summary>
    internal sealed class DurationJsonConverter : JsonConverter<TimeSpan>
    {
        /// <inheritdoc/>
        public override TimeSpan Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            return XmlConvert.ToTimeSpan(reader.GetString()!);
        }

        /// <inheritdoc/>
        public override void Write(Utf8JsonWriter writer, TimeSpan value, JsonSerializerOptions options)
        {
            writer.WriteStringValue(XmlConvert.ToString(value));
        }
    }
}
