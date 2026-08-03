# mizar-checker: Source Type Application Projection

> Canonical language: English. Japanese companion:
> [../ja/source_type.md](../ja/source_type.md).

## Purpose And Authority

`source_type` implements the Task 249 source type-head/application/argument
producer frozen in [`00.crate_plan.md`](./00.crate_plan.md). Its canonical
authority is Chapters 03 §§3.2-3.3, 05 §§5.2/5.6, 07 §§7.2-7.3/7.6, 08
§§8.1/8.3, 12 §§12.3/12.5/12.6.1/12.7, 18 §§18.1/18.2.2, and Appendix A.
The bounded audit owners are MC-G014, MC-G016, and MC-G020.

## Boundary And Model

The module accepts no `SurfaceAst`, `SurfaceNodeId`, or `SyntaxKind`. One
syntax-free `SourceTypeHandoffInput` contains dense outer-application,
expression/head, and ordered argument vectors. Applications link authenticated
reserve or definition bindings to root expressions. Expressions retain written
and head sites, ranges, spellings, recovery, form, and builtin or
resolver-authenticated mode/structure heads. Arguments retain exactly
`TermSite`, recursive `TypeSite`, or `QuaSite` input; term and `qua` sites carry
`SemanticOrigin` but no selected `BindingId`.

`SourceTypeProducer` authenticates the input against the actual `BindingEnv`,
`SymbolEnv`, and `TypedArena` before publishing
`SourceTypeApplicationHandoff`. The legacy reserve bridge exposes
`prepare_binding_env` as an input-only path; it validates symbol heads and
builds the real binding environment without declaration checking or type
normalization. Definition-parameter applications require an actual resolver
`DeclarationShell` owner; a generated context is never authenticated as a
declaration.

## Validation And Atomicity

Validation rejects cross-source/module data, stale binding identity/order/type
sites, unsupported head kinds, stale contribution provenance, local heads not
active before their use, invisible imported heads, missing or out-of-closure
import edges/targets, invalid or duplicate typed sites, empty spellings,
range/recovery mismatches, and invalid `SemanticOrigin`. Term/`qua` provenance
must use the exact identifier range, current source/module, no import edge,
matching recovery, and deterministic
`[parent-expression, argument-ordinal]` structural path.

The flat graph rejects dangling, cyclic, multiply parented, forward-parent,
duplicate-child, wrong-form, unreachable, non-contained, and overlapping
sibling/top-level relationships. Cycle and reachability checks use iterative
worklists, so public flat input does not consume the call stack. Validation
never sorts or repairs input. Failure publishes no partial handoff.

Every expression, head, term, and `qua` site is checked against its actual
typed-arena node both during production and during `TypedAst` installation.
The owning node must have a same-source range containing the narrower row range
and exactly matching recovery. This permits distinct role sites on the
existing Task-248 item nodes without changing that arena.

## Ownership, Consumers, And Exclusions

`TypedAst` owns the optional immutable handoff. `ResolvedTypedAst` can only
clone it from that typed AST; no separately replaceable resolved input exists.
Conditional debug rendering keeps legacy bytes unchanged when the handoff is
absent.

The broad real consumer traverses exactly ten reserve written types and
publishes 10 applications, 13 expressions/heads, and 6 arguments. The
Task-248 route separately co-installs two `Bare`/builtin-`set` rows and no
arguments using its actual checker-owned binding environment. Expansion,
normalization, inhabitation, subtyping, evidence, term or `qua` binding
selection, facts, declaration/proof acceptance, and Core/CFG/VC production are
outside Task 249.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceTypeApplicationForm` | `#[non_exhaustive]`; callers must tolerate later source-written forms. |
| `SourceTypeHead` | `#[non_exhaustive]`; callers must tolerate later authenticated head kinds. |
| `SourceTypeArgument` | `#[non_exhaustive]`; callers must tolerate later syntax-free argument shapes. |
| `SourceTypeError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Task 249 Classification

| Class | Result |
|---|---|
| `test_gap` | Closed only for the exact Task-249 handoff and Task-248 dependency consumer. |
| `source_drift` | Repaired for complete type-head/application/argument and final-handoff transport, import-closure authentication, and real `DeclarationShell` ownership. |
| `design_drift` | Repaired by the paired component, plan, todo, audit, and runner documents. |
| `boundary_violation` | Recursive public-input graph traversal found in implementation review was replaced by iterative worklists; syntax remains runner-owned and semantic result fabrication is forbidden. |
| `spec_gap` | None for this bounded input-handoff slice. |
| `repo_metadata_conflict` | None observed. |

## Task 251 Evidence-Association Addendum

`SourceTypeApplicationHandoff` is the authenticated parent input for every
Task-251 request. Unattributed requests retain the root expression, owner/head
sites, expression range/recovery, and application source ordinal. Attributed
requests retain the same application/expression identity while the independent
Task-250 chain supplies the request site/range/recovery; the request ordinal
remains the Task-249 application ordinal. Resolver-authenticated mode and
structure heads select the two unattributed request kinds, while builtin heads
emit none. Task 251 does not alter source-type tables or infer expansion,
inhabitation, normalization, or acceptance.

## Task 249R Frozen Definition-Return Extension

### Selection, Authority, And Classification

Fresh Task-260 implementation preflight found that the completed Task-249
producer deliberately requires one `SourceTypeApplicationId` per authenticated
`BindingId`. Task 248 Profile B has two definition-parameter bindings, so a
claimed Task-260 `4/4/0` source-type profile cannot represent two parameter
types plus two functor return types without inventing two bindings. Such
fabrication would be a `boundary_violation`. The frozen Task-260 documentation
therefore had nonblocking `design_drift`, while the missing independent
return-type transport is `source_drift`. There is no blocking `spec_gap`:
Chapter 10 §10.1 requires every `func` definition to carry the independently
written type expression after `->`, and §10.5 permits that type to depend on
input values.

Checker Task 249R is the dependency-ready lower-stage prerequisite for Task
260. Its authority is limited to Chapter 10 §§10.1 and 10.5 and the already
frozen Task-260 exact source/Surface profile. Its consumer is Task 260 only.
It changes no language semantics, canonical specification, existing `.miz`,
sidecar, expectation, trace row/status/count, runner code, resolver code, or
Cargo metadata.

### Exact Additive Public ABI

Task 249R extends the existing immutable `SourceTypeApplicationHandoff`; it
does not weaken or overload the binding-linked application table. The exact
new public types are:

```rust
pub struct SourceTypeDefinitionReturnId(usize);

pub struct SourceTypeDefinitionReturnExtensionInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub returns: Vec<SourceTypeDefinitionReturnInput>,
}

pub struct SourceTypeDefinitionReturnInput {
    pub definition_site: TypedSiteRef,
    pub definition_range: SourceRange,
    pub source_ordinal: usize,
    pub expression: SourceTypeExpressionInput,
}

pub struct SourceTypeDefinitionReturnTable { /* private entries */ }

pub struct SourceTypeDefinitionReturn {
    /* private id, definition_site, definition_range, source_ordinal, root */
}

pub struct SourceTypeDefinitionReturnProducer;
```

The ID derives `Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord +
Hash`; both input structs and the immutable row derive `Debug + Clone +
PartialEq + Eq`; and the table derives those traits plus `Default`.
The exact read-only methods and constness are:

```rust
impl SourceTypeDefinitionReturnId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}

impl SourceTypeDefinitionReturnTable {
    pub fn get(
        &self,
        id: SourceTypeDefinitionReturnId,
    ) -> Option<&SourceTypeDefinitionReturn>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (SourceTypeDefinitionReturnId, &SourceTypeDefinitionReturn),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

impl SourceTypeDefinitionReturn {
    pub const fn id(&self) -> SourceTypeDefinitionReturnId;
    pub const fn definition_site(&self) -> &TypedSiteRef;
    pub const fn definition_range(&self) -> SourceRange;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn root(&self) -> SourceTypeExpressionId;
}

impl SourceTypeApplicationHandoff {
    pub const fn definition_returns(
        &self,
    ) -> &SourceTypeDefinitionReturnTable;
}
```

The producer surface is exactly:

```rust
impl SourceTypeDefinitionReturnProducer {
    pub fn extend(
        base: &SourceTypeApplicationHandoff,
        input: SourceTypeDefinitionReturnExtensionInput,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError>;
}
```

`SourceTypeProducer::build` always initializes an empty definition-return
table. `extend` is one-shot, leaves the borrowed base unchanged on failure,
and returns a new immutable handoff on success. `SourceTypeError` gains the
non-exhaustive variants `EmptyDefinitionReturns`,
`DefinitionReturnCardinalityMismatch`, `DefinitionReturnsAlreadyPresent`,
`InvalidDefinitionReturnBase`,
`InvalidDefinitionReturn {
definition_return: SourceTypeDefinitionReturnId }`,
`InvalidDefinitionReturnSite { definition_return:
SourceTypeDefinitionReturnId }`, `UnsupportedDefinitionReturn {
definition_return: SourceTypeDefinitionReturnId }`, and
`OverlappingDefinitionReturns { definition_return:
SourceTypeDefinitionReturnId }`.

### Exact Task-260 Profile And Validation

The required base remains Task 248 Profile B's Task-249
applications/expressions/arguments `2/2/0`. Task 249R adds two return rows and
two expressions, yielding applications/expressions/arguments/definition
returns `2/4/0/2`:

| Return | Definition owner | Return expression/head | Output root |
| ---: | --- | --- | ---: |
| 0 | node 84, `61..118`, ordinal 0 | nodes 80/79, `105..108`, `Bare`, builtin `set`, normal | 2 |
| 1 | node 95, `121..179`, ordinal 1 | nodes 87/86, `163..166`, `Bare`, builtin `set`, normal | 3 |

The exact base applications are `(binding 0, ordinal 0, root 0)` and
`(binding 1, ordinal 1, root 1)`. Base expressions 0/1 use node/head sites
63/62 and 67/66, ranges `22..25` and `38..41`, `Bare`, builtin `set`, normal,
and spellings/head spellings `set`; the argument and definition-return tables
are empty. Any other base shape returns `InvalidDefinitionReturnBase`.

Exactly two rows of this normal, argument-free, bare builtin-`set`
return-expression shape are admitted. Source/module identity must equal the
base; return ordinals and
dense IDs are vector order; definition owner ranges are exact same-source
arena node ranges, ordered, nonempty, nonoverlapping, and contain their return
expression. All definition, expression, and head sites are exact
`TypedSiteRef::Node` identities; role sites are rejected. The expression and
head sites, ranges, and recovery are revalidated against the actual arena;
the syntax-free input spellings and head spellings must each equal `set`.
No definition, expression, or head site may duplicate another site within the
combined source-type handoff: the base expression/head sites plus the two new
definition/expression/head triples. Cross-family arena-site reuse remains
unchanged. New expression IDs append at the prior expression length.
The base is revalidated before extension, and `TypedAst` installation
revalidates both the return rows and all four expressions. `TypedAst` remains
the sole owner; final assembly trusts that already validated immutable value,
and `ResolvedTypedAst` only clone-preserves the same handoff. Neither owner
gains a second field or installation path.

The existing debug prefix and all legacy bytes remain identical when the
return table is empty. With Task 249R rows present, return rows appear after
all application rows and before all expression rows:

```text
definition-return#<id> ordinal=<n> definition_range=<start>..<end> definition_site=node#<id> root=<expression-id>
```

Expression rows then remain dense `0..3`; argument rows retain their existing
position. The complete combined debug text is Task 260's required source-type
fingerprint. Task 260 refers to return rows 0/1 through
`SourceTypeDefinitionReturnId`, never through `SourceTypeApplicationId`.

### Tests, Exclusions, Audit Impact, And Exit

Implementation adds exactly four checker library tests:

1. `task_249r_exact_definition_return_extension_and_legacy_debug`;
2. `task_249r_independent_return_corruption_fails_atomically`;
3. `task_249r_one_shot_base_environment_and_arena_drift_fail_closed`; and
4. `task_249r_typed_final_clone_replay_has_no_semantic_output`.

They respectively own exact extension/API/debug and empty-table byte stability;
independent return/owner/expression corruption with atomic failure; one-shot/
base/environment/arena fail-closed behavior; and TypedAst-to-ResolvedTypedAst
clone/replay with no semantic output. The checker baseline projects `435 ->
439`; runner/resolver/syntax
remain `512/144/59`. Task 260 then projects checker `439 -> 444` and runner
`512 -> 516`. All corpus, metadata, CLI, fixture, sidecar, expectation, and
trace counts/hashes remain unchanged during both Task-249R commits.

Forbidden and deferred are artificial `BindingId` rows; generalized
composite, attributed, or dependent-return graph intake; parameter/return
association beyond the exact owner row; expansion, normalization,
inhabitation, subtype/evidence decisions; goal/guard composition; proof,
discharge, acceptance, facts/axioms, Core/CFG/VC; public diagnostics; and all
Task-260 producer or runner work. The documentation prerequisite exits only
after synchronized EN/JA, repeated review-only **NO FINDINGS**, unchanged
executable/count/hash gates, all nine hard gates, quality at least 90, one
dedicated docs commit, and clean/stash-invariant fresh inventory. The separate
implementation exits with the four tests, exact `2/4/0/2` profile, full
verification/reviews/gates, one dedicated commit, and automatic return to Task
260 implementation.

### Task 249R Implementation Closure

The checker implementation now realizes the frozen additive ABI in
`source_type.rs`. `SourceTypeProducer::build` preserves the legacy empty-table
bytes; `SourceTypeDefinitionReturnProducer::extend` accepts only the exact
Task-249/Profile-B base and exact two-row Task-260 return profile; installation
revalidates every owner, expression, and head arena field; and the final owner
clone-preserves the same immutable handoff. The implementation adds only the
four frozen checker tests. It adds no runner/resolver/syntax code, corpus
artifact, trace row, diagnostic, fact, proof, acceptance, or VC behavior.

The fresh executable inventory is applications/expressions/arguments/returns
`2/4/0/2`, checker `439`, and unchanged runner/resolver/syntax
`512/144/59`. `source_type.rs` is `4407` lines and the checker production
manifest is `24/148143`. All five metadata CLI outputs and hashes remain
unchanged. Task 260 remains the sole next consumer; every semantic deferral
above remains in force.
