﻿namespace Azure.Iot.Operations.ProtocolCompiler
{
    using System;
    using System.Collections.Generic;
    using System.ComponentModel;
    using System.Diagnostics;
    using System.Linq;

    internal static class TypesGenerator
    {
        static readonly Dictionary<string, ISchemaStandardizer> SchemaStandardizers = new()
        {
            { ".schema.json", new JsonSchemaStandardizer() },
            { ".avsc", new AvroSchemaStandardizer() },
        };

        static readonly Dictionary<string, ITypeGenerator> TypeGenerators = new()
        {
            { "csharp", new DotNetTypeGenerator() },
            { "go", new GoTypeGenerator() },
            { "rust", new RustTypeGenerator() },
        };

        public static void GenerateType(string langName, TargetLanguage lang, string projectName, string schemaFileName, DirectoryInfo workingDir, string genRoot, CodeName genNamespace)
        {
            string schemaFileFolder = Path.Combine(workingDir.FullName, genNamespace.GetFolderName(TargetLanguage.Independent));
            string schemaFilePath = Path.Combine(schemaFileFolder, schemaFileName);
            string schemaIncludeFolder = Path.Combine(workingDir.FullName, ResourceNames.IncludeFolder);

            if (!Directory.Exists(genRoot))
            {
                Directory.CreateDirectory(genRoot);
            }

            if (schemaFileName.EndsWith(".proto"))
            {
                try
                {
                    Process.Start("protoc", $"--{langName}_out={Path.Combine(genRoot, genNamespace.GetFolderName(lang))} --proto_path={schemaFileFolder} --proto_path={schemaIncludeFolder} {schemaFileName}");
                }
                catch (Win32Exception)
                {
                    Console.WriteLine("protoc tool not found; install per instructions: https://github.com/protocolbuffers/protobuf/releases/latest");
                    Environment.Exit(1);
                }
            }
            else if (SchemaStandardizers.Any(ss => schemaFileName.EndsWith(ss.Key)))
            {
                ITypeGenerator typeGenerator = TypeGenerators[langName];
                ISchemaStandardizer schemaStandardizer = SchemaStandardizers.First(ss => schemaFileName.EndsWith(ss.Key)).Value;

                foreach (SchemaType schemaType in schemaStandardizer.GetStandardizedSchemas(schemaFilePath, genNamespace))
                {
                    typeGenerator.GenerateTypeFromSchema(projectName, schemaType, schemaStandardizer.SerializationFormat, genRoot);
                }
            }
        }
    }
}
