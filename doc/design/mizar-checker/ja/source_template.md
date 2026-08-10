# Source Template Transport

> canonical言語: English。canonical English:
> [../en/source_template.md](../en/source_template.md)。

## Task 277A Direct Parser-Origin Template Transport

[central Task 277A contract](../../task_contracts/ja/277A.md) がdirect-parser
profile、exact five two-row table、public ABI/error order、exclusion、baseline、gateを
ownする。このownerはそこに定めるsyntax-free `source_template` producerだけをimplementする:
dense ID、direct TypedArena validation、immutable handoff/table/getter/debug、crate-private
installation validationであり、resolver/spelling/semantic/substitution/target/verdict
stateは持たない。source-identity mismatchは`EnvironmentMismatch`がownし、malformedな
zero-length/inverted row rangeはfamily-local `Invalid*` failureのままにする。committed
implementationは`1745` lines、physical SHA-256
`fdd6ac38557979ed37fd7c9ba13300b8577416e4ebbdaefe64b986f22aceb85b`。
independent review/final quality re-reviewは**NO FINDINGS**、全9 hard gateはscore
capなしの有効な`100/100`でPASS。exact staging/cached-diff reviewもPASSした。
implementation commit `b67b028e07337ff5b72422bc8f16fb8f187b5c06`の直後、read-only
post-implementation checkpointは
`HEAD=b67b028e07337ff5b72422bc8f16fb8f187b5c06`、clean worktree、
`origin/main...HEAD=0/1`、unchanged protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`をobserveした。Task 277Aはcomplete、
umbrella Task 277はpartialのままで、successorはseparately frozen/reviewedでなければ
ならない。

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
