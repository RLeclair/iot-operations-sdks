﻿namespace Akri.Dtdl.Codegen
{
    using System;
    using System.IO;
    using System.Linq;
    using System.Text.Json;

    internal class EnvoyGenerator
    {
        public static void GenerateEnvoys(string language, string projectName, string annexFileName, DirectoryInfo workingDir, DirectoryInfo outDir, string genNamespace, string? sdkPath, bool syncApi, HashSet<string> sourceFilePaths)
        {
            string? relativeSdkPath = sdkPath != null ? Path.GetRelativePath(outDir.FullName, sdkPath) : null;
            using (JsonDocument annexDoc = JsonDocument.Parse(File.OpenText(Path.Combine(workingDir.FullName, genNamespace, annexFileName)).ReadToEnd()))
            {
                foreach (ITemplateTransform templateTransform in EnvoyTransformFactory.GetTransforms(language, projectName, annexDoc, workingDir.FullName, relativeSdkPath, syncApi, sourceFilePaths))
                {
                    string envoyFilePath = Path.Combine(outDir.FullName, templateTransform.FolderPath, templateTransform.FileName);
                    if (templateTransform is IUpdatingTransform updatingTransform)
                    {
                        string[] extantFiles = Directory.GetFiles(Path.Combine(outDir.FullName, templateTransform.FolderPath), updatingTransform.FilePattern);

                        if (extantFiles.Any())
                        {
                            if (updatingTransform.TryUpdateFile(extantFiles.First()))
                            {
                                Console.WriteLine($"  updated {extantFiles.First()}");
                            }

                            continue;
                        }
                    }

                    string envoyDirPath = Path.GetDirectoryName(envoyFilePath)!;
                    if (!Directory.Exists(envoyDirPath))
                    {
                        Directory.CreateDirectory(envoyDirPath);
                    }

                    File.WriteAllText(envoyFilePath, templateTransform.TransformText());
                    Console.WriteLine($"  generated {envoyFilePath}");
                    sourceFilePaths.Add(envoyFilePath);
                }
            }
        }
    }
}
