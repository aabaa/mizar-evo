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

## Task 249M Frozen Standalone Mode-RHS Extension

### Selection, Authority, And Classification

Fresh Task-262 preflight confirms that each existing
`SourceTypeApplicationInput` is deliberately linked one-to-one to a
`BindingId`. The exact mode definition has two parameter bindings but three
written type expressions: parameter types at roots 0/1 and an independently
written mode RHS at root 2. Treating root 2 as a third application would
fabricate a binding and is a `boundary_violation`. The missing independent
RHS owner is `source_drift`; the already committed Task-262 upper contract
repairs the corresponding `design_drift`. There is no blocking `spec_gap`:
Chapter 7 Sections 7.1--7.3 and 7.6--7.8 distinguish the parameter tuple from
the mode RHS/expansion and require the latter to denote an inhabited type.

Checker Task 249M is the mandatory lower-stage prerequisite for Task 262. Its
authority and consumer are limited to that exact Chapter-7 RHS and the frozen
141-byte Task-262 source/54-row Surface oracle. It changes no language
semantics, canonical specification, existing `.miz`, sidecar, expectation,
trace row/status/count, runner/resolver/parser code, public diagnostic, or
Cargo metadata. Task 262 production and its corpus/trace activation remain a
later logical task.

### Exact Additive Public ABI

Task 249M extends the immutable `SourceTypeApplicationHandoff` without
weakening the binding-linked application table or reusing Task 249R return
semantics. The exact new public types are:

```rust
pub struct SourceTypeModeRhsId(usize);

pub struct SourceTypeModeRhsExtensionInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub rhs: Vec<SourceTypeModeRhsInput>,
}

pub struct SourceTypeModeRhsInput {
    pub definition_site: TypedSiteRef,
    pub definition_range: SourceRange,
    pub source_ordinal: usize,
    pub expression: SourceTypeExpressionInput,
}

pub struct SourceTypeModeRhsTable { /* private entries */ }

pub struct SourceTypeModeRhs {
    /* private id, definition_site, definition_range, source_ordinal, root */
}

pub struct SourceTypeModeRhsProducer;
```

The ID derives `Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord +
Hash`; both input structs and the immutable row derive `Debug + Clone +
PartialEq + Eq`; and the table derives those traits plus `Default`. The exact
read-only methods and constness are:

```rust
impl SourceTypeModeRhsId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}

impl SourceTypeModeRhsTable {
    pub fn get(&self, id: SourceTypeModeRhsId) -> Option<&SourceTypeModeRhs>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceTypeModeRhsId, &SourceTypeModeRhs)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

impl SourceTypeModeRhs {
    pub const fn id(&self) -> SourceTypeModeRhsId;
    pub const fn definition_site(&self) -> &TypedSiteRef;
    pub const fn definition_range(&self) -> SourceRange;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn root(&self) -> SourceTypeExpressionId;
}

impl SourceTypeApplicationHandoff {
    pub const fn mode_rhs(&self) -> &SourceTypeModeRhsTable;
}
```

The producer surface is exactly:

```rust
impl SourceTypeModeRhsProducer {
    pub fn extend(
        base: &SourceTypeApplicationHandoff,
        input: SourceTypeModeRhsExtensionInput,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError>;
}
```

`SourceTypeProducer::build` initializes empty definition-return and mode-RHS
tables. `extend` is one-shot, leaves the borrowed base unchanged on every
failure, and returns a new immutable handoff on success. `SourceTypeError`
gains the non-exhaustive variants `EmptyModeRhs`,
`ModeRhsCardinalityMismatch`, `ModeRhsAlreadyPresent`,
`InvalidModeRhsBase`, `InvalidModeRhs { mode_rhs: SourceTypeModeRhsId }`,
`InvalidModeRhsSite { mode_rhs: SourceTypeModeRhsId }`, and
`UnsupportedModeRhs { mode_rhs: SourceTypeModeRhsId }`. Error precedence is
already-present, empty, non-singleton cardinality, environment identity,
base, row, owner/site, then unsupported expression.

### Exact Task-262 Lower Profile And Validation

The required base is the Task-262 Task-248/Profile-B Task-249 profile
applications/expressions/arguments/definition-returns/mode-RHS
`2/2/0/0/0`. Task 249M appends one RHS row and one expression, producing
`2/3/0/0/1`:

| Row | Definition owner | RHS expression/head | Output root |
| ---: | --- | --- | ---: |
| 0 | node 49, `45..135`, ordinal 0 | nodes 44/43, `95..98`, `Bare`, builtin `set`, normal | 2 |

The exact base applications are `(binding 0, ordinal 0, root 0)` and
`(binding 1, ordinal 1, root 1)`. Base expressions 0/1 use expression/head
sites 35/34 and 39/38, ranges `22..25` and `38..41`, `Bare`, builtin `set`,
normal recovery, and `set` for both spellings. Arguments, definition returns,
and mode RHS rows are empty. Any other base shape, including a Task-249R
handoff, returns `InvalidModeRhsBase`.

Exactly one normal, argument-free, bare builtin-`set` RHS is admitted. Source
and module identity must equal the base. The dense row ID and source ordinal
are zero. The definition range is the exact same-source arena node-49 range,
contains the RHS expression, and is nonempty. Definition, expression, and head
sites are exact `TypedSiteRef::Node` identities; role sites are rejected. The
expression and head sites, ranges, and recovery are revalidated against the
actual arena. Both syntax-free spellings are exactly `set`. No new site may
duplicate any base expression/head or definition site. The new expression ID
appends at the prior expression length as root 2.

The base is fully revalidated before extension. Installation revalidates the
mode-RHS row and all three expressions. Definition-return and mode-RHS
extensions are mutually exclusive in every order; malformed combined states
fail closed. `TypedAst` remains the sole owner and `ResolvedTypedAst` only
clone-preserves the same handoff. Neither owner gains another field,
installer, parts field, or replaceable final input.

The existing debug prefix and every legacy/Task-249R byte remain identical
when the mode-RHS table is empty. With Task 249M present, mode-RHS rows render
after definition-return rows and before expression rows:

```text
mode-rhs#<id> ordinal=<n> definition_range=<start>..<end> definition_site=node#<id> root=<expression-id>
```

For the active lower profile the complete suffix is row 0 followed by dense
expressions 0--2; argument rows retain their existing final position. This
complete combined debug text is the future Task-262 source-type fingerprint.
Task 262 refers to row 0 through `SourceTypeModeRhsId`, never through
`SourceTypeApplicationId` or `SourceTypeDefinitionReturnId`.

### Tests, Exclusions, Audit Impact, And Exit

Implementation adds exactly four checker library tests:

1. `task_249m_exact_mode_rhs_extension_and_legacy_debug`;
2. `task_249m_mode_rhs_corruption_fails_atomically`;
3. `task_249m_one_shot_base_and_arena_drift_fail_closed`; and
4. `task_249m_typed_final_clone_replay_and_task_249r_isolation`.

They respectively own exact extension/API/debug plus legacy and Task-249R
byte stability; empty/multiple/environment/owner/expression/site/spelling/
recovery corruption and borrowed-base atomicity; exact-base, one-shot, arena,
and installation drift; and Typed-to-Resolved clone/replay, two-way Task-249R
isolation, and absence of semantic output. Every field and error class is
mutated independently. Tests 2 and 3 also own compound mutations across every
adjacent precedence boundary: already-present over empty, empty over
non-singleton cardinality, cardinality over environment mismatch, environment
mismatch over invalid base, invalid base over invalid row, invalid row over
invalid owner/site, and invalid owner/site over unsupported expression.

The checker baseline projects `449 -> 453`; runner/resolver/syntax remain
`520/144/59`. Metadata remains cases/requirements `424/392`, pass/fail
`231/193`, active parse/declaration/type/proof `101/7/201/1`, type
requirements `256/244`, and warnings/errors `23/0`. Task 262 later projects
checker/runner `458/524` and owns the sole corpus/trace delta. Production,
test-list, CLI, and manifest hashes are fresh-measured in implementation.

Forbidden and deferred are artificial `BindingId` or application rows;
reuse/renaming of definition-return rows; generalized mode RHSs; attributed,
argument-bearing, resolver-symbol, structure, or recovery intake; request or
inhabitation response; expansion/normalization/acceptance; sethood goal/guard,
proof/discharge/fact behavior; public diagnostics; Core/CFG/VC; and every
Task-262 checker/runner/corpus/trace change.

This documentation prerequisite changes only synchronized design records and
exits after repeated review-only **NO FINDINGS**, unchanged executable/count/
hash gates, all nine hard gates, quality at least 90, one dedicated docs
commit, and clean/stash-invariant fresh inventory. The separate implementation
write scope is `crates/mizar-checker/src/source_type.rs` plus synchronized
design records only. It exits with exactly four tests, exact `2/3/0/0/1`
profile, full reviews/verification/gates, one dedicated commit, and automatic
fresh-inventory return to Task 262 implementation.

## Task 249M Active Implementation Result

The preceding future/prerequisite wording is historical. The frozen API is now
implemented in `crates/mizar-checker/src/source_type.rs`: one dense mode-RHS
ID, extension input and row input, immutable row/table, borrowed handoff
getter, unit producer, seven errors with the frozen precedence, exact base and
arena revalidation, one-shot atomic extension, installation validation,
deterministic debug ordering, and bidirectional Task-249R exclusion.

The exact four named tests are active and independently cover all fields and
error classes, every adjacent compound-precedence boundary, legacy/Task-249R
bytes, arena and installation drift, Typed/Resolved clone/replay, and empty
semantic outputs. The lower oracle is exactly `2/3/0/0/1`. Checker library is
`453`; raw/normalized test-list hashes are
`34f63b3b9fb1ae2f3b43d769184be2b0c23cc3ada13b5a8b45a933aed629fe25` /
`ee25ffd88d06e34491ced5c0499acc4198c1e8690ed40c3fb79fb276e3852db4`.
Production is `26/153116`, with path/content hashes
`e290d082e428124d3fd21919e76b88458daabfa44b7009a8cb1b3d8c430fec53` /
`3c85673ebb527cb33bb4b042b1b1194bda34a5348b4b6b20142617db47bde2f2`.
Runner/resolver/syntax and all corpus/trace/CLI/metadata values remain at the
frozen baselines. Task 262 remains the sole next consumer and a separate
logical task.
## Task 249S Frozen Standalone Structure-Member Type Intake

### Selection, authority, and classification

Fresh Checker Task-263 preflight confirms that the exact 320-byte structure
source has no parameter binding and has four independently written member type
expressions. `SourceTypeProducer` deliberately requires one nonempty
`SourceTypeApplicationInput` per authenticated `BindingId`; fabricating four
bindings is a `boundary_violation`. Reusing Task-249R definition-return rows or
the Task-249M mode-RHS row would also assign the wrong owner semantics. The
missing standalone member-type owner is `source_drift`; this frozen lower
contract repairs the corresponding `design_drift`. There is no blocking
`spec_gap`: Chapter 5 §§5.1--5.3 explicitly give every field and property a
written type, while §5.2 keeps property values out of constructor arguments.

Task 249S is the mandatory checker-only lower prerequisite for Task 263. Its
canonical source is exactly 320 bytes with a final LF and SHA-256
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`:

```mizar
definition
  struct Task263Base where
    field carrier -> set;
    property marker -> set;
  end;

  struct Task263Derived where
    field carrier -> set;
    property marker -> set;
  end;

  inherit Task263Derived extends Task263Base where
    field carrier from carrier;
    property marker from marker;
  end;
end;
```

Only the four declaration-member type expressions cross this lower boundary.
Structure-definition nodes, structure and selector symbols, member kind,
parent/root/path/view identity, inheritance targets and redefinitions, field
coverage, constructor/selector declarations, coherence, and obligations remain
Task-263 inputs or outputs.

### Exact additive public ABI

Task 249S adds the following public syntax-free types in this existing module.
No new public enum type is added; five variants append to the existing
non-exhaustive public `SourceTypeError` below.

```rust
pub struct SourceTypeStructureMemberId(usize);

pub struct SourceTypeStructureMemberHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub members: Vec<SourceTypeStructureMemberInput>,
}

pub struct SourceTypeStructureMemberInput {
    pub member_site: TypedSiteRef,
    pub member_range: SourceRange,
    pub source_ordinal: usize,
    pub expression: SourceTypeExpressionInput,
}

pub struct SourceTypeStructureMemberTable { /* private entries */ }

pub struct SourceTypeStructureMember {
    /* private id, member_site, member_range, source_ordinal, root */
}

pub struct SourceTypeStructureMemberProducer;
```

The ID derives `Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord +
Hash`; both inputs and the immutable row derive `Debug + Clone + PartialEq +
Eq`; the table additionally derives `Default`. The exact read-only surface is:

```rust
impl SourceTypeStructureMemberId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}

impl SourceTypeStructureMemberTable {
    pub fn get(
        &self,
        id: SourceTypeStructureMemberId,
    ) -> Option<&SourceTypeStructureMember>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceTypeStructureMemberId, &SourceTypeStructureMember)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

impl SourceTypeStructureMember {
    pub const fn id(&self) -> SourceTypeStructureMemberId;
    pub const fn member_site(&self) -> &TypedSiteRef;
    pub const fn member_range(&self) -> SourceRange;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn root(&self) -> SourceTypeExpressionId;
}

impl SourceTypeApplicationHandoff {
    pub const fn structure_members(&self) -> &SourceTypeStructureMemberTable;
}

impl SourceTypeStructureMemberProducer {
    pub fn build(
        input: SourceTypeStructureMemberHandoffInput,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError>;
}
```

`SourceTypeProducer::build` initializes the new table empty, and the Task-249R
and Task-249M producers preserve that empty table. The standalone producer
transactionally constructs a new immutable handoff; it neither accepts nor
fabricates a base. `SourceTypeError` gains the non-exhaustive variants
`EmptyStructureMembers`, `StructureMemberCardinalityMismatch`,
`InvalidStructureMember { structure_member: SourceTypeStructureMemberId }`,
`InvalidStructureMemberSite { structure_member:
SourceTypeStructureMemberId }`, and `UnsupportedStructureMember {
structure_member: SourceTypeStructureMemberId }`.

### Exact Task-263 lower profile and validation

The successful handoff has applications/expressions/arguments/definition
returns/mode RHS/structure members `0/4/0/0/0/4`:

| Row | Member owner | Type expression/head | Root |
| ---: | --- | --- | ---: |
| 0 | node 53, `42..63`, ordinal 0 | nodes 52/51, `59..62`, `Bare`, builtin `set`, normal | 0 |
| 1 | node 56, `68..91`, ordinal 1 | nodes 55/54, `87..90`, `Bare`, builtin `set`, normal | 1 |
| 2 | node 61, `134..155`, ordinal 2 | nodes 60/59, `151..154`, `Bare`, builtin `set`, normal | 2 |
| 3 | node 64, `160..183`, ordinal 3 | nodes 63/62, `179..182`, `Bare`, builtin `set`, normal | 3 |

Exactly four rows are admitted. Dense ID, source ordinal, and root equal
vector order. Every row and expression has the input source/module identity;
each nonempty same-source member range is the exact normal arena-node range
and contains its expression. Member, expression, and head sites are distinct
exact `TypedSiteRef::Node` identities; role sites and duplicate sites are
rejected. Expression/head ranges and recovery are revalidated against the
arena. Every expression is argument-free, bare, normal builtin `set`, with
both spellings exactly `set`. The failure order is empty, non-four
cardinality, row/environment/range identity, site/arena identity, then
unsupported expression shape.

The handoff validator recognizes this profile while keeping it mutually
exclusive with binding applications, arguments, definition returns, and mode
RHS rows. `TypedAst` remains the sole owner through its existing optional
`source_type` field and installation path; `ResolvedTypedAst` only
clone-preserves the handoff. No second owner or installer is added.

When the member table is empty, every existing debug byte remains unchanged.
When present, member rows occur after mode-RHS rows and before expression rows:

```text
structure-member#<id> ordinal=<n> member_range=<start>..<end> member_site=node#<id> root=<expression-id>
```

The complete deterministic debug text is the Task-263 lower fingerprint.
Task 263 refers to roots only through `SourceTypeStructureMemberId`, never
through a fabricated `SourceTypeApplicationId`.

### Tests, exclusions, audit impact, and exit

Implementation adds exactly four checker library tests:

1. `task_249s_exact_structure_member_build_and_legacy_debug`;
2. `task_249s_member_corruption_fails_atomically`;
3. `task_249s_arena_and_installation_drift_fail_closed`; and
4. `task_249s_typed_final_replay_and_sibling_isolation`.

They own exact API/profile/debug and legacy stability; row/environment/range/
site/shape corruption and error precedence; arena plus installation
revalidation; and deterministic replay, Typed/final clone preservation, and
Task-249R/249M isolation. Checker library count projects `458 -> 462`;
runner/resolver/syntax stay `524/146/59`. This prerequisite adds no runner,
corpus source, sidecar, expectation, trace row/status/count, diagnostic,
obligation, or metadata case.

Forbidden and deferred are artificial bindings/applications; generalized
structure-member type graphs; parameters/context; field/property
classification; structure/member/resolver identity association; inheritance
parents, roots, paths, views, coverage, constructors, selectors, or
redefinitions; type equality/subtyping/inhabitation; coherence; goal/guard
composition; proof/discharge/acceptance/facts/axioms/Core/CFG/VC; public
diagnostics; and all Task-263 producer/runner/corpus work.

The documentation prerequisite changes no production, fixture, sidecar,
expectation, trace, test-list, CLI, manifest, or executable hash. It exits only
after synchronized EN/JA documents, repeated review-only **NO FINDINGS**, all
nine hard gates with uncapped quality at least 90, a dedicated docs commit,
clean post-commit inventory, unchanged origin classification, and protected
stash invariance. The separate implementation exits only after the exact four
tests, `0/4/0/0/0/4` profile, full reviews and verification, a dedicated
commit, and automatic return to Task 263.

## Task 249S Active Implementation Result

The frozen API and standalone `0/4/0/0/0/4` profile are implemented without
contract change. Validation performs global passes in the frozen order:
cardinality, all row/environment/range identities, all site/arena identities,
then all expression shapes. This prevents an earlier-row site or shape fault
from masking a later-row higher-priority fault. Mixed applications, arguments,
definition returns, or mode RHS rows fail closed before sibling validators.

All four exact tests pass, including every owner/expression/head arena node
under recovered and normal-wrong-range drift, all four mixed-table
corruptions, cross-row compound precedence, deterministic replay, and
Typed/final ownership. Legacy empty-member debug bytes remain unchanged.
`source_type.rs` is `6244` lines; checker inventory is `462` and its
raw/normalized list hashes are
`5f18c633183db679ecacb2781c9133dad5b4c48fdb00e33435dd4c1329105fd2` /
`e0da07dbaf28c659f9e3ac682ae5cf694e7ddd5cdb987abe5d2598ebbfc68d7d`.
Task 263 and every frozen semantic deferral remain separate.

## Task 263 Test-Only Lower Replay Seam

Task 263 adds one `cfg(test)`-only crate-private mutator that corrupts a stored
structure-member root so its later lower-relation category can be paired with
mapping and coherence faults. Production Task-249S validation, public API,
fingerprint grammar, and accepted `0/4/0/0/0/4` behavior are unchanged;
`source_type.rs` is now 6,253 lines.

## Task 249PI Frozen Property-Implementation Composition

### Selection, authority, and one-task scope

Fresh inventory after the Task-264 documentation prerequisite selects checker
Task 249PI as the only dependency-ready task. Canonical Chapter 5 §§5.1--5.2
require the written `set` return of both `carrier` and virtual `marker`, while
Chapter 7 §§7.4.1, 7.8.2, and 7.10 require the implementation parameter
`M: Task264Carrier`. The exact Task-264 means/equals sources, parser rows,
resolver local structure symbol, Task-248P binding 0, and the frozen Task-264
lower-bundle contract derive the bounded composition below. There is no
blocking `spec_gap`.

Current Task 249 can authenticate the parameter's binding-linked structure
application, and Task-249S can independently own structure-member return
types, but Task-249S intentionally admits only the standalone four-member
Task-263 profile. Their inability to coexist in one immutable handoff is a
lower `source_drift` with paired `design_drift`; the four canonical-derived
checker regressions below close the `test_gap`. Fabricating the member returns
inside Task 264, fabricating parameter applications for members, or reusing
definition-return/mode-RHS rows is a `boundary_violation`.

Task 249PI owns only the exact source-type composition required by Task 264.
It does not own property identity or member kind, the implementation target,
the lookup from `marker` to member row 1, parameters or binding contexts,
`equals`/`means`, definiens terms/formulas, `it`, correctness or initial
obligations, coherence, goals or guards, proofs, discharge, acceptance,
facts/axioms, diagnostics, runner selection, Core/CFG/VC, or Task-259 data.

### Exact additive API and errors

No new public input, row, table, ID, enum, handoff, owner, or debug family is
added. Task 249PI appends this method to the existing producer:

```rust
impl SourceTypeStructureMemberProducer {
    pub fn extend_property_implementation(
        base: &SourceTypeApplicationHandoff,
        input: SourceTypeStructureMemberHandoffInput,
        arena: &TypedArena,
    ) -> Result<SourceTypeApplicationHandoff, SourceTypeError>;
}
```

The method borrows an already authenticated Task-249 base, clones it only
after all base prerequisites pass, appends two existing structure-member rows
and their expression roots, validates the complete result, and returns a new
immutable handoff. The input source/module must equal the base. The base is
never mutated, even on success. The existing standalone `build(input, arena)`
signature and behavior remain exact.

The existing non-exhaustive `SourceTypeError` appends exactly these variants:

```rust
StructureMembersAlreadyPresent,
StructureMemberExtensionCardinalityMismatch,
InvalidStructureMemberBase,
```

Their display strings are respectively `source type structure members are
already installed`, `source type structure-member extension cardinality is
not the frozen pair`, and `source type structure-member base is invalid`.
Existing row/site/shape errors retain their exact fields and strings. Producer
failure precedence is already-present, empty, non-two extension cardinality,
source/module environment mismatch, invalid base, all row/environment/range
identities in ordinal order, all site/arena identities in ordinal order, then
all expression shapes in ordinal order. Thus a lower-priority earlier-row
fault cannot hide a higher-priority later-row fault.

### Frozen means and equals profiles

Both successful handoffs have applications/expressions/arguments/definition
returns/mode RHS/structure members `1/3/0/0/0/2`. Application 0 is exactly
binding 0, source ordinal 0, root expression 0. Arguments, definition returns,
and mode RHS remain empty. Member IDs and source ordinals are 0/1 and their
roots are appended expressions 1/2. All stored IDs equal vector order.

The parameter root is a normal, argument-free `Bare` symbol application whose
source and head spellings are exactly `Task264Carrier` and whose source/head
range is `130..144`. `SourceTypeProducer` has already authenticated its
`SourceTypeHead::Symbol` against the resolver as the current-module local
structure and local-source contribution 0. Task 249PI preserves that exact
`SymbolId` and contribution. In particular, it neither reconstructs an FQN
from the spelling nor replaces the resolver-generated FQN with a simplified
`<module>::Task264Carrier` string. The means expression/head nodes are 63/64;
the equals nodes are 45/46.

The appended rows are exact:

| Profile | Member | Owner node/range | Expression/head nodes and range | Root |
| --- | ---: | --- | --- | ---: |
| means | 0 | 56 / `45..66` | 55/54 / `62..65` | 1 |
| means | 1 | 59 / `71..94` | 58/57 / `90..93` | 2 |
| equals | 0 | 38 / `45..66` | 37/36 / `62..65` | 1 |
| equals | 1 | 41 / `71..94` | 40/39 / `90..93` | 2 |

Each member expression is normal, argument-free, `Bare`, builtin `set`, with
source/head spellings exactly `set`. Each member range is nonempty,
same-source, its exact normal owner-arena range, and contains its expression.
Every parameter/member/expression/head site is the exact distinct
`TypedSiteRef::Node` listed above; role, duplicate, missing, recovered, and
wrong-range sites fail closed. Every expression and head range is revalidated
against the supplied arena.

The complete validator recognizes exactly two mutually exclusive
structure-member profiles: legacy standalone Task-249S
`0/4/0/0/0/4`, byte-for-byte unchanged, and Task-249PI
`1/3/0/0/0/2`. Any mixed application/member shape outside these profiles,
including a Task-249R return or Task-249M RHS, fails closed. Empty-member
legacy Task-249 application handoffs remain valid and byte-identical.

### Debug fingerprint and ownership

The existing `source-type-application-debug-v1` grammar and ordering are
unchanged. A Task-249PI fingerprint emits the version line, module line,
application 0, member rows 0/1, then expressions 0/1/2. In the template below,
`<module-path>` is the exact `ModuleId.path`, `<resolver-fqn>` is the complete
FQN stored in the already authenticated resolver `SymbolId`, and each
angle-bracketed node choice takes its means or equals value consistently. The
concrete substitution of every placeholder is the complete fingerprint, with
exactly one LF after its final expression line and no extra blank line:

```text
source-type-application-debug-v1
module: <module-path>
application#0 binding=0 ordinal=0 root=0
structure-member#0 ordinal=0 member_range=45..66 member_site=node#<56-or-38> root=1
structure-member#1 ordinal=1 member_range=71..94 member_site=node#<59-or-41> root=2
expression#0 form=bare range=130..144 site=node:<63-or-45> head=symbol:<resolver-fqn>:contribution:0 head_range=130..144 head_site=node:<64-or-46> recovery=normal spelling="Task264Carrier" head_spelling="Task264Carrier"
expression#1 form=bare range=62..65 site=node:<55-or-37> head=builtin:set head_range=62..65 head_site=node:<54-or-36> recovery=normal spelling="set" head_spelling="set"
expression#2 form=bare range=90..93 site=node:<58-or-40> head=builtin:set head_range=90..93 head_site=node:<57-or-39> recovery=normal spelling="set" head_spelling="set"
```

`TypedAst` remains the sole owner through its existing optional `source_type`
field and one-shot installation. Installation revalidates the complete
profile and arena identities. `ResolvedTypedAst` only clone-preserves the same
handoff and exact fingerprint. Task 249PI adds no field or installer to either
type. Task 264 will consume the complete fingerprint and member ID 1; it may
not supply fingerprint text or infer lower rows.

### Tests, count/hash impact, and exit

Implementation changes only `crates/mizar-checker/src/source_type.rs` plus
synchronized derived design records and adds exactly four checker tests:

1. `task_249pi_exact_means_and_equals_extensions_and_debug`;
2. `task_249pi_base_and_member_corruption_fail_atomically`;
3. `task_249pi_arena_and_installation_drift_fail_closed`; and
4. `task_249pi_typed_final_replay_and_sibling_isolation`.

They own both exact profiles/fingerprints and legacy Task-249S bytes; every
base/row/range/site/shape/precedence and one-shot failure; arena plus
installation drift; deterministic replay, Typed/final clone ownership, and
Task-249R/249M/249S/259 isolation. Checker library count projects `469 ->
473`; runner/resolver/syntax stay `528/148/59`. There is no runner production,
fixture, sidecar, expectation, trace row/backlink/status/count, metadata,
diagnostic, CLI, or executable coverage delta.

The docs prerequisite exits after synchronized EN/JA records, repeated
review-only **NO FINDINGS**, all nine hard gates PASS without score cap at
90/100 or better, exact docs-only staging/commit, clean fresh inventory,
report-only origin divergence, and protected-stash invariance. The separate
implementation exits only after the exact four tests, reviews, focused/full
verification and count/hash gates, one-file task-only staging, a dedicated
commit, clean fresh inventory, and automatic return to Task 264.

## Task 249PI Documentation-Prerequisite Verification

Repeated specification and boundary/source-documentation reviews report **NO
FINDINGS** after correcting the resolver-FQN/debug template and closing stale
Task-264 checklist state. Independent final quality reports **NO FINDINGS**;
all nine hard gates PASS with no score cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). The delta is exactly 32 synchronized design files
and has zero canonical spec, production, test, fixture, sidecar, expectation,
trace, Cargo, or metadata change.

Focused parser/resolver/Profile-C checker tests pass `1/2/2`; checker/runner
lint policies pass `15/14`; metadata passes `137/137`; `cargo fmt --all
--check`, warnings-denied all-target/all-feature Clippy, full workspace tests,
Cargo metadata, all five CLIs, and `git diff --check` pass. The CLIs reproduce
cases/requirements `426/394`, pass/fail `233/193`, active
parse/declaration/type/proof `101/7/203/1`, type coverage `258=246+12`, and
warnings/errors `23/0`.

Checker/runner/resolver/syntax lists remain `469/528/148/59`; their exact
raw/normalized hashes remain respectively
`3e7712bb86277f45d8883e949cf9f59d9b20176693c4f224751184728b92ddc7` /
`2fdb8681cad17eeee4640433aaa0f54428fc83f9941cd62862652a9aebb859b4`,
`b8128fc8f77a50aebba6dfb75488cb838ccc84c8a3f9bd71f046304cd607784e` /
`5887a3aaf1818b44fba6d46d49b7275997928c4ce2587a6cc47343eee3a35456`,
`c99d9d179cf14ab9ccd274b11d0404bdc47a64d23a2aa914c69ba674d01a3fee` /
`1c76831124b1e680d708fd30ddfa7a96959aa82d20c840594dbb108dcd063490`,
and `512775259a51121a0c12ab9fbf0d1083273d3d140362d889cdd9e22184215da6` /
`c11a29c90fee3fe81d839f80f196dcd405cc43c7da86e83f37dc123042066540`.

Checker production remains `28/158478`, path/content
`6e4bc96ef04cb5f011d53c651bb93549992e3c7fd0e7595b851d7181c8a65dcd` /
`19a0dd0472f0e3b40c486ab9451322be03aab4322c53d30cff03ef5e6f8c8490`;
runner production remains `35/67939`, path/content
`4218936ff3ee3baaceb7c0723307ad266d722d0a2473e8b7f82e11c75aeb2b6e` /
`a543608c5075ffed97141626ebbf8d952a847051a34d6782097329b44aa1d09e`.
Trace remains
`cf0ef6d28a132bcbafc8aa1214ded935a715fdffdb3421c37d66c35954f2a06c`;
all Task-48 and mixed-gap source/sidecar hashes remain the exact prior values.
Exact staging, commit, and post-commit fresh inventory remain parent-owned.

## Task 249PI Implementation Verification

The frozen method, three errors, both exact profiles, debug bytes, one-shot
Typed/final ownership, and all semantic exclusions are implemented only in
`source_type.rs`. The four named tests pass; checker is `473` with
raw/normalized hashes
`5481b3b20fb75e4d2bab93ce575660f0941aaef01210b06544c9910ecace97cd` /
`db822929f96290beda1209837b0f517ee555f6e01e38b3f13a59918423bb327d`.
The source owner is `7423` lines with SHA-256
`ef6ec1978ab1b25d01f9ee6fb78538f4a1fb6c97c3a32ba3af618c981d0f4c86`;
checker production is `28/159648`, path/content
`6e4bc96ef04cb5f011d53c651bb93549992e3c7fd0e7595b851d7181c8a65dcd` /
`7d38e5c9fbc3ee2cb09d0d5d1187c4d29d1086c56f0b2dcd7f07cd0b60be283c`.

Test review first found missing adjacent precedence and member-1 corruption
coverage; implementation review then found an orphan-member installation
shape. Both classified gaps were repaired and both repeated reviews end **NO
FINDINGS**. Task-249 siblings, runner `528`, resolver `148`, syntax `59`,
runner production `35/67939`, corpus/metadata/CLI outputs, fixtures,
expectations, and trace hash remain exact. Formatting, warnings-denied
workspace Clippy, full workspace tests, five CLIs, and `git diff --check`
pass. Source/documentation consistency also ends **NO FINDINGS**; independent
quality passes all nine hard gates without score cap at `100/100`. Staging,
commit, and fresh Task-264 inventory remain the final parent-owned gates.
