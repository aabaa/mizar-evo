# Source Template Transport

> Canonical language: English. Japanese companion:
> [../ja/source_template.md](../ja/source_template.md).

## Task 277A Direct Parser-Origin Template Transport

The [central Task 277A contract](../../task_contracts/en/277A.md) owns the
frozen direct-parser profile, exact five two-row tables, public ABI, error
order, exclusions, baselines, and gates. This owner implements only the syntax-free
`source_template` producer described there: dense IDs, direct TypedArena
validation, immutable handoff/tables/getters/debug, and crate-private
installation validation,
and no resolver, spelling, semantic, substitution, target, or verdict state.
Source-identity mismatch owns `EnvironmentMismatch`; malformed zero-length or
inverted row ranges remain family-local `Invalid*` failures. The worktree
implementation is `1745` lines with physical SHA-256
`fdd6ac38557979ed37fd7c9ba13300b8577416e4ebbdaefe64b986f22aceb85b`;
Independent reviews and final-quality re-review report **NO FINDINGS**; all
nine hard gates PASS without a score cap at valid `100/100`. Only
staging/cached-diff review, commit, and post-commit proof remain pending.

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
