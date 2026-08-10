# Source Template Transport

> Canonical language: English. Japanese companion:
> [../ja/source_template.md](../ja/source_template.md).

## Task 277A Direct Parser-Origin Template Transport

The [central Task 277A contract](../../task_contracts/en/277A.md) owns the
frozen direct-parser profile, exact five two-row tables, public ABI, error
order, exclusions, baselines, and gates. This owner adds only the syntax-free
`source_template` producer described there: dense IDs, direct TypedArena
validation, immutable handoff/tables/getters/debug, and crate-private
installation validation,
and no resolver, spelling, semantic, substitution, target, or verdict state.
Implementation remains pending.

## Frozen Public Surface

Generated public IDs are `SourceTemplateParameterId`,
`SourceTemplateLociId`, `SourceTemplateLocusId`,
`SourceTemplateArgumentsId`, and `SourceTemplateArgumentId`.

Literal public items are `SourceTemplateHandoffInput`,
`SourceTemplateParameterInput`, `SourceTemplateLociInput`,
`SourceTemplateLocusInput`, `SourceTemplateArgumentsInput`,
`SourceTemplateArgumentInput`, `SourceTemplateRecovery`,
`SourceTemplateParentKind`, `SourceTemplateParameterKind`,
`SourceTemplateParameterTable`, `SourceTemplateParameter`,
`SourceTemplateLociTable`, `SourceTemplateLoci`,
`SourceTemplateLocusTable`, `SourceTemplateLocus`,
`SourceTemplateArgumentsTable`, `SourceTemplateArguments`,
`SourceTemplateArgumentTable`, `SourceTemplateArgument`,
`SourceTemplateHandoff`, `SourceTemplateError`, and
`SourceTemplateProducer`. No caller-supplied resolver or semantic item is
part of this surface.

## Public Enum Policy

| Enum | Policy | Exhaustive exception |
|---|---|---|
| `SourceTemplateRecovery` | `#[non_exhaustive]` | none |
| `SourceTemplateParentKind` | `#[non_exhaustive]` | none |
| `SourceTemplateParameterKind` | `#[non_exhaustive]` | none |
| `SourceTemplateError` | `#[non_exhaustive]` | none |

No exhaustive public enum exceptions are owned by this module.
