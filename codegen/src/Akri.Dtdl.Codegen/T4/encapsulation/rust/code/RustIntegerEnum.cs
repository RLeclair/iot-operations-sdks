namespace Akri.Dtdl.Codegen
{
    using System.Collections.Generic;

    public partial class RustIntegerEnum : ITemplateTransform
    {
        private readonly string genNamespace;
        private readonly EnumType enumType;

        public RustIntegerEnum(string genNamespace, EnumType enumType)
        {
            this.genNamespace = genNamespace;
            this.enumType = enumType;
        }

        public string FileName { get => $"{NamingSupport.ToSnakeCase(this.enumType.SchemaName)}.rs"; }

        public string FolderPath { get => Path.Combine(SubPaths.Rust, this.genNamespace); }
    }
}
