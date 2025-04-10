/* Code generated by Azure.Iot.Operations.ProtocolCompiler v0.10.0.0; DO NOT EDIT. */

#nullable enable

namespace Azure.Iot.Operations.Services.AssetAndDeviceRegistry.AdrBaseService
{
    using System;
    using System.Collections.Generic;
    using System.Text.Json.Serialization;
    using Azure.Iot.Operations.Services.AssetAndDeviceRegistry;

    [System.CodeDom.Compiler.GeneratedCode("Azure.Iot.Operations.ProtocolCompiler", "0.10.0.0")]
    public partial class DetectedAssetDataPointSchemaElementSchema
    {
        /// <summary>
        /// The 'dataPointConfiguration' Field.
        /// </summary>
        [JsonPropertyName("dataPointConfiguration")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
        public string? DataPointConfiguration { get; set; } = default;

        /// <summary>
        /// The 'dataSource' Field.
        /// </summary>
        [JsonPropertyName("dataSource")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
        public string? DataSource { get; set; } = default;

        /// <summary>
        /// The 'lastUpdatedOn' Field.
        /// </summary>
        [JsonPropertyName("lastUpdatedOn")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
        public string? LastUpdatedOn { get; set; } = default;

        /// <summary>
        /// The 'name' Field.
        /// </summary>
        [JsonPropertyName("name")]
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
        public string? Name { get; set; } = default;

    }
}
