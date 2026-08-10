# Source Template Transport

> canonical言語: English。canonical English:
> [../en/source_template.md](../en/source_template.md)。

## Task 277A Direct Parser-Origin Template Transport

[central Task 277A contract](../../task_contracts/ja/277A.md) がdirect-parser
profile、exact five two-row table、public ABI/error order、exclusion、baseline、gateを
ownする。このownerはそこに定めるsyntax-free `source_template` producerだけを扱う:
dense ID、direct TypedArena validation、immutable handoff/table/getter/debug、crate-private
installation validationであり、resolver/spelling/semantic/substitution/target/verdict
stateは持たない。implementationはpendingである。

## Frozen Public Surface

generated public IDは`SourceTemplateParameterId`、`SourceTemplateLociId`、
`SourceTemplateLocusId`、`SourceTemplateArgumentsId`、
`SourceTemplateArgumentId`。

literal public itemは`SourceTemplateHandoffInput`、
`SourceTemplateParameterInput`、`SourceTemplateLociInput`、
`SourceTemplateLocusInput`、`SourceTemplateArgumentsInput`、
`SourceTemplateArgumentInput`、`SourceTemplateRecovery`、
`SourceTemplateParentKind`、`SourceTemplateParameterKind`、
`SourceTemplateParameterTable`、`SourceTemplateParameter`、
`SourceTemplateLociTable`、`SourceTemplateLoci`、
`SourceTemplateLocusTable`、`SourceTemplateLocus`、
`SourceTemplateArgumentsTable`、`SourceTemplateArguments`、
`SourceTemplateArgumentTable`、`SourceTemplateArgument`、
`SourceTemplateHandoff`、`SourceTemplateError`、`SourceTemplateProducer`。
caller-supplied resolver/semantic itemはこのsurfaceに含めない。

## Public Enum Policy

| Enum | Policy | Exhaustive exception |
|---|---|---|
| `SourceTemplateRecovery` | `#[non_exhaustive]` | none |
| `SourceTemplateParentKind` | `#[non_exhaustive]` | none |
| `SourceTemplateParameterKind` | `#[non_exhaustive]` | none |
| `SourceTemplateError` | `#[non_exhaustive]` | none |

この module が所有する exhaustive public enum exception はない。
