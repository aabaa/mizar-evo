# Source Template Type-Parameter Association

> Canonical language: English. Japanese companion: [../ja/source_template_type_parameter_association.md](../ja/source_template_type_parameter_association.md).

## Task 277B-L Template Type-Parameter Association

This is the durable module owner for the standalone checker module
`crates/mizar-checker/src/source_template_type_parameter_association.rs`.
The frozen orchestration, source/test scope, baselines, and readiness boundary
are owned by [Task 277B-L](../../task_contracts/en/277B-L.md). The module is
implemented; its neutral transport still does **not** make Task 277B ready.

The module consumes completed R1
`TemplateTypeParameterSourceCollection` and an existing `TypedAst`. It returns
an immutable association handoff. It neither extends 277A `source_template`,
installs a Typed/Resolved slot, nor creates a production runner route.

### Public API

`SourceTemplateTypeParameterAssociationId` exposes only `new(index: usize) ->
Self` and `index(self) -> usize`. `SourceTemplateTypeParameterAssociation` has
getters:

- `binding() -> TemplateTypeParameterBindingId`;
- `definition_block()`, `parameter()`, `binder()`, `type_head()`, and
  `identifier()` each returning `TypedNodeId`;
- `parameter_range()` and `type_head_range()` each returning `SourceRange`; and
- `parameter_source_ordinal()` and `type_head_source_ordinal()` each returning
  `usize`.

`SourceTemplateTypeParameterAssociationTable` exposes only
`get(SourceTemplateTypeParameterAssociationId) ->
Option<&SourceTemplateTypeParameterAssociation>`, `iter() -> impl
Iterator<Item = (SourceTemplateTypeParameterAssociationId,
&SourceTemplateTypeParameterAssociation)>`, `len() -> usize`, and `is_empty()
-> bool`.

`SourceTemplateTypeParameterAssociationHandoff` owns and exposes `source_id()
-> SourceId`, `module_id() -> &ModuleId`, `associations() ->
&SourceTemplateTypeParameterAssociationTable`, and `debug_text() -> String`.
It is the sole output owner; there is no caller DTO.
`SourceTemplateTypeParameterAssociationError` is
`#[non_exhaustive]` and has `EnvironmentMismatch` plus
`InvalidAssociation { association: SourceTemplateTypeParameterAssociationId }`.
`SourceTemplateTypeParameterAssociationProducer` exposes exactly:

```rust
build(
    collection: &TemplateTypeParameterSourceCollection,
    typed_ast: &TypedAst,
) -> Result<
    SourceTemplateTypeParameterAssociationHandoff,
    SourceTemplateTypeParameterAssociationError,
>
```

### Invariants and validation

Rows retain R1's binding identity, ranges, and source ordinals. They are dense
in the existing resolver link order, which is already authoritative for order
and ambiguity; the checker adds no reorder or duplicate error variant.

Validation is fail-closed and ordered: environment; R1 binding lookup; then an
exactly-one scan match where `TypedNode.resolved_node == Some(the R1
ResolvedNodeId)` for each of the five sites; normal recovery; exact node kind
(`DefinitionBlockItem`, `TemplateParameter`, canonical `Identifier` for both
binder and generator identifier, and `TypeHead`);
range anchors; R1 range equality; binder-within-parameter,
parameter/type-head-within-definition, and identifier-within-type-head ranges;
then direct `definition -> parameter`, `parameter -> binder`, and
`type_head -> identifier` edges. Every post-environment failure returns the
association-specific invalid error. Zero/multiple scan matches fail; dense-ID
casts and range/name inference are forbidden. The producer is deterministic,
does not mutate `TypedAst`, and adds no Typed/Resolved link slot.

The R1 fixture association is binding `0`,
`DefinitionBlockItem#53` / `TemplateParameter#31` / `Identifier#2` to
`TypeHead#39` / `Identifier#21`, in the 57-node arena rooted at 56, with
parameter range `606..620`, type-head range `678..679`, and both ordinals 0.

### Module boundary and tests

The implementation may modify only the new module, checker `lib.rs`, checker
lint-policy inventory, the private mizar-test leaf, and its `tests.rs` include.
It must not edit resolver sources, `source_template.rs`, 277A, Typed/Resolved
installation, Cargo, canonical specifications/tests/metadata, or production
runner/facade/dispatch.

The four checker tests exhaustively cover exact mapping and public getters;
source/module mismatch; missing or ambiguous matches at each of five sites;
all-site kind and recovery corruption; prefix-spoofed non-canonical
`Identifier` kinds; non-range/wrong-source/empty anchors, exact-range and
containment failures, and each direct-edge removal; then deterministic,
non-mutating rebuilds for empty, singleton, and multi-link profiles.
One private mizar-test real-fixture probe constructs its own F5 `TypedAst` from
the same validated Surface/Resolved 57-node profile, attaching resolver IDs
only through the arena mapping, and calls this producer directly. No existing
helper or 277A route supplies that typed arena. It has no active semantic,
diagnostic, or coverage effect.

The implementation adds this module to checker lint policy's
public-enum module list, source/spec module list, public-API path allowlist,
and `lib.rs` public-module allowlist. The paired source/spec audits now carry
the crate-export row and exact public-item inventory. Test-sufficiency review
is **NO FINDINGS**; implementation review is **NO FINDINGS** after the
canonical-`Identifier` prefix-spoof fix. Source/documentation re-review is **NO
FINDINGS** after the EN/JA CLI-tense fix; bilingual and boundary reviews are
**NO FINDINGS**. Checker/mizar-test lint, focused and full libraries, package/
workspace Clippy, full tests, formatting/diff checks, metadata, unchanged CLI
hashes, and protected-surface gates all pass. Final-quality review found one
Medium missing identifier-within-type-head range containment. The repair added
the containment and a corruption assertion to
`task277bl_rejects_kind_range_recovery_and_direct_edge_corruption` and
synchronized this EN/JA owner and contract; finding-specific re-review is **NO
FINDINGS**. All nine hard gates PASS uncapped at valid `100/100`
(`20/20/15/15/10/10/5/5`). Exact staging/cached-diff review, task-only commit,
post-implementation proof, and fresh successor inventory are closed in the
central [historical checkpoint](../../task_contracts/en/277B-L.md#post-implementation-checkpoint);
no successor is selected. This implementation retains Task 277B's
not-ready, zero-semantic-credit boundary.

## Public Enum Policy

| Enum | Policy | Exhaustive exception |
|---|---|---|
| `SourceTemplateTypeParameterAssociationError` | `#[non_exhaustive]` | none |
| `SourceTemplateFraenkelStructuralCompositionError` | `#[non_exhaustive]` | none |

No exhaustive public enum exceptions are owned by this module.

## Task 277C Frozen Planned Public Extension

The canonical [277C contract](../../task_contracts/en/CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md)
records the completed neutral structural-composition family in this existing module:
`SourceTemplateFraenkelStructuralCompositionId`, row, table, handoff,
`#[non_exhaustive] SourceTemplateFraenkelStructuralCompositionError`, and
producer. The producer exposes `build(template, generators, typed_ast)` and
returns the immutable composition handoff. The contract exclusively owns the
exact error precedence, row getters, validation, F5 profile, test matrix, and
measured completion evidence; the public-enum policy above now applies.

It remains a standalone `SourceTemplateTypeParameterAssociationHandoff` +
`FraenkelGeneratorVariableSourceCollection` + `TypedAst` composition: it added
no R1 direct input, state installation, source-owner route, semantic credit, or
production activation. The legacy-stable heading preserves existing owner links.
Broad verification passes, and the independent source/documentation, bilingual,
and boundary reviews report **NO FINDINGS**. Final-quality review reports **NO
FINDINGS** at valid uncapped `100/100` (`20/20/15/15/10/10/5/5`). The
historical closeout records that exact staging/cached review, the task-only
implementation commit, post-commit proof, and fresh successor inventory are
closed in the language-local [central historical checkpoint](../../task_contracts/en/CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md#historical-immediate-post-implementation-checkpoint).
No successor is selected; Task 277B remains not ready with zero semantic credit.

## Task 257C4A Fraenkel generator dependency boundary

The planned [C4A](../../task_contracts/en/CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A.md)
is the nonduplicative consumer of completed 277C and R2. It consumes their
authoritative rows together with `TypedAst`, rechecks the unique
resolved-to-typed binder relation without an ID cast, and does not extend this
module's 277C association ABI. Structural/debug summaries alone are insufficient.
The lower clones and raw resolver nodes remain opaque, while all term,
type/sethood, diagnostic, installation, route, and Task-277B conclusions stay
outside this dependency boundary.
