namespace Akri.Dtdl.Codegen
{
    using System.Collections.Generic;
    using DTDLParser.Models;

    public partial class ObjectJsonSchema : ITemplateTransform
    {
        private readonly string genNamespace;
        private readonly string objBareId;
        private readonly string versionSuffix;
        private readonly string description;
        private readonly string schema;
        private readonly List<(string, string, DTSchemaInfo, int)> nameDescSchemaIndices;
        private readonly DtmiToSchemaName dtmiToSchemaName;
        private readonly bool setIndex;

        public ObjectJsonSchema(string genNamespace, string objectId, string description, string schema, List<(string, string, DTSchemaInfo, int)> nameDescSchemaIndices, DtmiToSchemaName dtmiToSchemaName, bool setIndex)
        {
            int semiIndex = objectId.IndexOf(';');
            int bareIdLength = semiIndex >= 0 ? semiIndex : objectId.Length;

            this.objBareId = objectId.Substring(0, bareIdLength);
            this.versionSuffix = objectId.Substring(bareIdLength);

            this.genNamespace = genNamespace;
            this.description = description;
            this.schema = schema;
            this.nameDescSchemaIndices = nameDescSchemaIndices;
            this.dtmiToSchemaName = dtmiToSchemaName;
            this.setIndex = setIndex;
        }

        public string FileName { get => $"{this.schema}.schema.json"; }

        public string FolderPath { get => this.genNamespace; }
    }
}
