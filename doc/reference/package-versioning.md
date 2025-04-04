# Package Versioning 

The new Azure.IoT.Operations SDKs, currently implemented in .NET, Rust and Go languages, will be consumed by the Protocol Compiler for producing network-protocol-bound (MQTT) programmable interfaces (named `Envoys`) from the data and messaging constructs directly defined in a DTDL model document.

These messaging constructs currently encompass `telemetry` and `commands`, but they could be modified and expanded in later versions of DTDL.
Azure.Iot.Operations.MQTT could add more MQTT v5 features as the project progresses.

Finally, the Service clients could have changes in their public APIs, as well as internally (e.g., bug fixes) or in their language-specific library dependencies.

Known requirements for clients and codegen tool:
- All clients and code generation tool MUST be publicly stored in the same GitHub repository
- No additional repositories will be created for individual clients to publish their code

Worth noting, there must be:
- A version of each Azure.Iot.Operations.Protocol (in the different languages supported) that can communicate with each other (through the `client` and `server` entities, a.k.a. `invoker` and `executor` respectively)
- A way for identifying which version(s) of each of the clients work with each other
- A way for identifying which version(s) of MQTT-patterns Code-Gen work with which versions of each individual client.
- A manner for each individual package to make their own changes (internal or public) and be able to bump their own version tag.

## Version Control and Release

Each Azure.Iot.Operations.Services and Azure.Iot.Operations.Protocol APIs/libraries have:
- Their own standard directory structure within the IoT Operations repository.
- Freedom to control their own version and release process (1)
- Versions will be based on SemVer2, with cross-compatibility and documentation using <major>.<minor> version.

(1) Plus a specific process to coordinate the testing and documentation of cross-compatibility between clients and code-generation tool.

The Code Generation Tool will:
* Map only the minimum required major versions of each client (e.g.: CodeGen 2.x => .NET Client 3.x, Rust Client 2.x, Go Client 2.x)
* A new feature in any client (within same major version) does not get consumed by Code Gen until that client gets a new major version release and Code Gen gets a new major version release to consume that.
* Minor version updates in the Code Gen tool concern only updates to the Code Gen code only, not to the generation templates (unless a bug needs to be addressed).

`Azure.Iot.Operations.Protocol`, `Azure.Iot.Operations.Mqtt` and `Azure.Iot.Operations.Services` packages will be independently versioned within each language.

## Consolidate Info about Clients, Versions and Package Release

|Subject|.NET|Rust|Go|
|-|-|-|-|
|Relative path in repository|./dotnet|./rust|./go|
|New version/release requires a commit?|Yes|Yes|No|
|Version control|To be done directly in .csproj of the client projects.|Version stored in Cargo.toml files for each Rust crate (package).|Not stored in any files of the code base. A new version of the library is "published" by tagging the git repo. It gets updated on pkg.go.dev when someone pulls the (public) repo and it gets cached in the default proxy. Example: [azure-sdk-for-go/azidentity](https://pkg.go.dev/github.com/Azure/azure-sdk-for-go/sdk/azidentity)|
|Version control files in repo|./src/Azure.Iot.Operations.Mqtt/Azure.Iot.Operations.Mqtt.csproj, ./src/Azure.Iot.Operations.Protocol/Azure.Iot.Operations.Protocol.csproj, ./src/Azure.Iot.Operations.Services/Azure.Iot.Operations.Services.csproj|./azure_iot_operations_mqtt/Cargo.toml, ./azure_iot_operations_protocol/Cargo.toml.|None. Comment from Carl LeCompt: "If we wanted to tie updating the version to a new commit, one thing we've done in previous projects is store the version in the library's magefile.go and only updated the tags with corresponding mage commands. It's not required, but it's something to consider.". We shall consider it as we implement the control system.|
|Release process|The .csproj files are manually updated with the new version number, a new pull-request is created to merge the changes, having to pass all the standard and cross-compatibility tests. Once merged, it is published to nuget.org.|The Cargo.toml files are manually updated with the new version number (under [package] "version"), a new pull-request is created to merge the changes, having to pass all the standard and cross-compatibility tests. Once merged, it is published to crates.io.|Done directly by updating the package listing in golang.org.|
|Package signing|Required for public-preview and GA. Nuget packages to be signed using DevOps pipeline provided by .NET team.|None. See [ongoing discussion](https://foundation.rust-lang.org/news/2023-12-21-improving-supply-chain-security/).|None.|

## Public Documentation of Cross-Compatibility

This is the information that should available to customers:

1. Azure.Iot.Operations.Services and Azure.Iot.Operations.Protocol Supported Features per Version
1. Supported Versions across Azure.Iot.Operations.Protocol APIs
1. Code Generation Tool Minimum Required Azure.Iot.Operations.Protocol Versions 

Examples:

**Azure.Iot.Operations.Protocol Supported Features per Version**

|Feature|Service|.NET|Rust|Go|
|-|-|-|-|-|
|Feature Z|>= 1.0|1.2|>= 1.1, < 2.*|>= 1.1, < 2.*|
|Feature Y|>= 1.1|>= 1.1|>= 1.2|Planned|
|Feature X|Not Applicable|Not Supported|>= 1.1|>= 1.2|
|Telemetry|>= 1.0|>= 1.0|>= 1.0|>= 1.0|
|Commands|>= 1.0|>= 1.0|>= 1.0|>= 1.0|

**Supported Versions Across Azure IoT Operations Protocol and Clients**

This relates only to cross-compatibility from the perspective of the Azure.Iot.Operations.Protocol.

For compatibility between clients about specific features customers should consult the table above.

Note that the latest version of the protocol is back-compatible with all the previous releases within the major version (obviously as per SemVer2.0).

If the protocol cannot fully honor that it must release a new major version. And all the clients will then need new major versions released to support the new protocol version.

If the latest protocol version is back-compatible with all its previous minor versions, so are all the versions of a client that do support that protocol version.

Question: if we do validate that client version 1.x is back-compatible with 1.(x-1), and 1.(x-1) is back-compatible with 1.(x-2), is there any condition where 1.x will not be back-compatible with 1.(x-2)? In such case, the test matrix will be greatly reduced to verifying only the previous minor version back-compatibility.

When a new major version is introduced to any of the clients (in the main branch), the previous major version MUST have a dedicated branch created for it so bugs can be fixed in previous actively-supported client versions.

Initial data:

|Protocol|.NET|Rust|Go|
|-|-|-|-|
|1.0|1.0|1.0|1.0|

.NET has several releases:

|Protocol|.NET|Rust|Go|
|-|-|-|-|
|1.0|**1.6**|1.0|1.0|

Rust and Go both release new packages:

|Protocol|.NET|Rust|Go|
|-|-|-|-|
|1.0|1.6|**1.1**|**1.1**|

Protocol adds a new feature and fixes some bugs. .NET client then releases new version:

|Protocol|.NET|Rust|Go|
|-|-|-|-|
|**1.1**|**1.7**|-|-|
|1.0|1.6|1.1|1.1|

Other clients release new versions:

|Protocol|.NET|Rust|Go|
|-|-|-|-|
|1.1|1.7|**1.2**|**1.2**|
|1.0|1.6|1.1|1.1|

Protocol adds a breaking feature, adds new major version. Other clients release new versions:

|Protocol|.NET|Rust|Go|
|-|-|-|-|
|**2.0**|**2.0**|-|**2.0**|
|1.1|1.7|1.2|1.2|
|1.0|1.6|1.1|1.1|

Rust client releases new 2.0 version, other clients release new versions:

|Protocol|.NET|Rust|Go|
|-|-|-|-|
|2.0|**2.1**|**2.0**|**2.1**|
|1.1|1.7|1.2|1.2|
|1.0|1.6|1.1|1.1|

A fix is released to previous version of .NET client. Only the patch version of .NET 1.7 is changed, so no change occurs in the table:

|Protocol|.NET|Rust|Go|
|-|-|-|-|
|2.0|2.1|2.0|2.1|
|1.1|**1.7**|1.2|1.2|
|1.0|1.6|1.1|1.1|

Question: will we ever have to back-port a new feature to an older version of a client? Like for example: feature N is back-ported to .NET client 1.7. causing it to bump to version 1.8.

|Protocol|.NET|Rust|Go|
|-|-|-|-|
|2.0|2.1|2.0|2.1|
|1.1|**1.8**|1.2|1.2|
|1.0|1.6|1.1|1.1|

Question: At which point will we deprecate previous versions?

**Code Generation Tool Minimum Required Azure IoT Operations Client Versions**

|CodeGen|.NET|Rust|Go|
|-|-|-|-|
|5.*|2.*|3.*|2.*|
|4.*|2.*|3.*|1.*|
|3.*|2.*|2.*|1.*|
|2.*|2.*|1.*|1.*|
|1.*|1.*|1.*|1.*|

## Code Generation Tool

The code generator will map only major versions of the clients in its compatibility matrix.

## Cross-Client Validation

Done by gated builds, every pull-request will need to run a Cross-Compatibility check.

The cross-compatibility check:

* Is not required to merge a pull-request to any dev/feature branch (i.e, not main).
* Is required to pass to merge to main. Note: All release PRs are expected to be against main branch.

All the code in main shall always be back and cross-compatible. Non-cross-compatible code will then live in feature branches until ready to merge. This makes the customer's lives easier - if they clone from main everything works.

For clients that do not save any info related to package releasing (e.g., version tags) in code and do not depend on pull-requests to release - like Go -, the PRs can validate just whatever code the PR contains.

Overall logic for pull-requests:

```mermaid
flowchart TB

Begin(Begin) --> isPRForRelease{Is Pull-Request for a new release?}
isPRForRelease -->|No| nonReleaseChanges[Make code changes]
isPRForRelease -->|Yes| updateClientVersionInFiles[Update client version]
updateClientVersionInFiles --> addNewVersionToDB[Add new version to DB `versiondb -add <language/version> ...`]
nonReleaseChanges --> createOrUpdatePR[Create or Update Pull-Request]
addNewVersionToDB --> createOrUpdatePR
createOrUpdatePR --> runRegularGateChecks[Run all regular gate checks]
runRegularGateChecks --> allRegularTestsPassed{All regular tests passed?}
allRegularTestsPassed -->|No| fixPR[Make necessary changes to fix PR]
allRegularTestsPassed -->|Yes| runCrossCompatTests[[Run cross-compatibility checks]]
fixPR --> createOrUpdatePR
runCrossCompatTests --> allCrossCompatTestsPassed{All Cross-Compatibility checks passed?}
allCrossCompatTestsPassed -->|No| isPRAgainstMain{Is Pull-Request against main branch?}
allCrossCompatTestsPassed -->|Yes| mergePR[Merge Pull-Request]
isPRAgainstMain -->|No| mergePR
isPRAgainstMain -->|Yes| fixPR
mergePR --> triggerPkgRelease[Trigger package release]
triggerPkgRelease --> End(End)
```

Calling `versiondb -add ...` will add a new client version in the database (a json file...) that is marked as not tested.

Example:

```json
[
    {
        "language": "dotnet",
        "versions": [
            {
                "number": "1.1.6",
                "verified": false,
                "compatibility": [
                    {
                        "target": "protocol",
                        "version": "1.0" // for the protocol, this is the exact version supported.
                    }, 
                    {
                        "target": "rust",
                        "version": "1.1.1" // for the clients, this is the mininum version supported. Get the latest related min-version for compatibility checks.
                    }, 
                    {
                        "target": "go",
                        "version": "1.0.7" // for the clients, this is the mininum version supported. Get the latest related min-version for compatibility checks.
                    }
                ]
            }
        ] 
    },
]
```

The cross-compatibility checks will be automated and will verify:

```mermaid
flowchart LR

Begin(Begin) --> isThereAPreviousMinVersion{Is there a</br>previous minor version release</br>with this major version?}
isThereAPreviousMinVersion -->|Yes| verifyBackCompatibility[Verify same language</br>back-compatibility]
isThereAPreviousMinVersion -->|No| getUnverifiedVersionsFromDB[Get list of unverified</br>versions from DB]
verifyBackCompatibility --> isBackCompatTestPassed{Is back-compatibility</br>test successful?}
isBackCompatTestPassed -->|Yes| getUnverifiedVersionsFromDB
isBackCompatTestPassed -->|No| failChecks[Fail cross-compatibility</br>checks]
getUnverifiedVersionsFromDB --> isClientVersionInFiles{Does client have</br>version tag in files?}
isClientVersionInFiles -->|Yes| verifyFilesMatchVersionInDB[Verify target files</br>match version in DB]
isClientVersionInFiles -->|No| runCrossCompatTests[Run cross-compatibility</br>tests between clients]
verifyFilesMatchVersionInDB --> isVersionInFilesCorrect{Is version tag</br>in files correct?}
isVersionInFilesCorrect -->|Yes| runCrossCompatTests
isVersionInFilesCorrect -->|No| failChecks
runCrossCompatTests --> areCrossCompatTestsPassed{Are cross-compatibility</br>tests successful?}
areCrossCompatTestsPassed -->|Yes| updateVersionDB[Mark version as</br>verified in version DB]
areCrossCompatTestsPassed -->|No| failChecks
updateVersionDB --> succeedChecks[Return success for</br>cross-compatibility checks]
failChecks --> End(End)
succeedChecks --> End
```

The cross-compatibility tests should be run between the code of the client that the PR is related to against the versions of the published packages of the other clients (not from code, if packages are published separately).

TBD: we should also have a nightly build to validate published packages (Rust and .NET, not Go) to make sure what was published matches what was verified in code. Also to validate if the published packages that are supposed to be actively-supported are indeed still published (e.g, the wrong nuget package could have been deleted).

## Package Versioning Definitions of Each Language

### .NET Package Versioning (Basic)

"A specific version number is in the form `Major.Minor.Patch[-Suffix]`, where the components have the following meanings:

Major: Breaking changes

Minor: New features, but backwards compatible

Patch: Backwards compatible bug fixes only

Suffix (optional): a hyphen followed by a string denoting a pre-release version (following the Semantic Versioning or SemVer convention)."

### .NET Package Versioning (Pre-release)

Package creators can use any string as a suffix to denote a pre-release version. 

However, these are the conventions:
* alpha: Alpha release, typically used for work-in-progress and experimentation.
* beta: Beta release, typically one that is feature complete for the next planned release, but may contain known bugs.
* rc: Release candidate

When ordering versions by precedence, NuGet follows the SemVer standard and chooses a version without a suffix first, then applies precedence to pre-release versions in reverse alphabetical order and treats dot notation numbers with numerical order.

Note: Prerelease numbers with dot notation, as in 1.0.1-build.23, are considered are part of the SemVer 2.0.0 standard, and as such are only supported with NuGet 4.3.0+.

### Rust Crate Versioning

The Cargo.toml file for each package is called its manifest.

The TOML format defines the `version` information within the `package` definition of a Cargo file.

Reference: https://doc.rust-lang.org/cargo/reference/manifest.html

> "Cargo bakes in the concept of Semantic Versioning, so make sure you follow some basic rules:
> 
> Before you reach 1.0.0, anything goes, but if you make breaking changes, increment the minor version. In Rust, breaking changes include adding fields to structs or variants to enums.
> 
> After 1.0.0, only make breaking changes when you increment the major version. Don’t break the build.
> 
> After 1.0.0, don’t add any new public API (no new pub anything) in patch-level versions. Always increment the minor version if you add any new pub structs, traits, fields, types, functions, methods or anything else.
> 
> Use version numbers with three numeric parts such as 1.0.0 rather than 1.0."

### Default Go Package Versioning

A released module is published with a version number in the semantic versioning model: "v1.4.0-beta.2"

| Version stage | Example | Message to developers |
|-|-|-|
| In development | Automatic pseudo-version number v0.x.x | ...no backward compatibility or stability guarantees. |
| Major version	| v1.x.x | ... backward-incompatible public API changes. |
| Minor version	| vx.4.x | ... backward-compatible public API changes. |
| Patch version	| vx.x.1 | ...changes that don't affect the module's public API or its dependencies. |
| Pre-release version |	vx.x.x-beta.2 | ...carries no stability guarantees.

Reference: https://go.dev/doc/modules/version-numbers
