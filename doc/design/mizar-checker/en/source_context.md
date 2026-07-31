# mizar-checker: Source and Binding Context Projection

> Canonical language: English. Japanese companion:
> [../ja/source_context.md](../ja/source_context.md).

## Purpose And Authority

`source_context` implements the Task 248 source/binding-context producer frozen
in [`00.crate_plan.md`](./00.crate_plan.md). Profile A's authority is limited
to Chapters 04 §4.3 and §4.6, 11 §11.2, 12 §12.3 and §12.7, and 15 §15.10.
Profile B additionally uses Chapters 04 §§4.2/4.6, 09
§§9.1/9.3--9.5/9.9.3--9.9.5, and 18 §§18.2.1/18.2.6/18.6 plus Appendix A.
The module preserves source-item order, resolver shell provenance, distinct
reserve and definition-parameter identities, local shadowing, and checker
context links.

## Boundary

The module accepts syntax-free projections. Opaque `DeclarationShellId` values
must come from the resolver's real `DeclarationShellSet`; the checker neither
constructs shell identities nor imports `mizar-syntax`. `mizar-test` owns the
bounded `SurfaceAst` walk and supplies source ranges, typed sites, lexical
scope, source order, and resolver-shaped `LocalTermBinding` provenance.

Task 248 admits only the two named real-consumer profiles frozen in this
document. Profile A is the implemented module-level `reserve x for set;`
followed by one `definition` block with one local `set` parameter named `x`.
Profile B is the separately frozen one-normal-definition-block/two-parameter
extension below. The Vec-based input and table shapes preserve order, but no
other cardinality or role combination is accepted. Additional reserve items,
including canonical distinct-name multiple-reserve input, are valid language
shapes but are rejected as `UnsupportedTaskShape` because they are outside
these exact profiles. Only the replacement or duplicate rule for re-reserving
the same identifier is undefined by the cited canonical specification; that
nonblocking `spec_gap` requires later human-reviewed authority before it can
gain meaning.

The module does not normalize types, resolve use sites, evaluate RHS terms,
build facts or obligations, verify formulas/proofs, or implement Tasks 249+
or 269+. Steps 6/7 remain deferred.

## Projection Model

- `SourceBindingContextInput` carries source/module identity, the module typed
  site, ordered item shells, and ordered binding sites.
- Complete construction produces checker-owned source-item and declaration
  tables, one `BindingEnv`, exact binding-to-local-context links, and one
  immutable `SourceBindingContextHandoff` that owns its local-context table.
- `TypedAst` installs the handoff only when source/module identity, the entire
  local-context table, item/declaration sites, context links, and the module
  root owner agree. `ResolvedTypedAst` can only clone that installed handoff.
- Reserve and local parameter bindings retain distinct checker ids; the local
  row records the module reserve as its structural shadow predecessor.

## Validation, Recovery, And Atomicity

Validation rejects missing/duplicate/reordered rows, stale ordinals, source,
module or range mismatches, invalid parent/context/site links, unsupported
visibility, stale local provenance, wrong roles, duplicate local binders, and
partial payloads before publishing a complete handoff. Implemented Profile A
also requires both items to be top level and the definition parameter to have
the reserve spelling, so the structural shadow link cannot disappear. Frozen
Profile B requires one top-level definition item, two distinct same-scope
parameter spellings, and no shadow link.

A recovered definition shell is supported only when it claims no binding. The
producer then returns `SourceBindingContextIncomplete` with an empty recovered
context and one deterministic internal diagnostic. A recovered shell with a
binding is rejected. Incomplete or inconsistent data never installs any
source-context table in `TypedAst` or `ResolvedTypedAst`.

This recovery rule belongs only to Profile A. Profile B is normal-only: one
recovered definition item, either recovered parameter, or any partial
two-parameter payload is rejected and never publishes an incomplete
Profile-B handoff.

## Determinism And Coverage

Dense ids follow validated source order. Identical input yields equal tables
and byte-identical nonempty debug text; reordered input is rejected rather than
sorted. The legacy `TypedAst` path with no source-context handoff retains an
exact full-string debug oracle.

The implemented Profile-A fixture
`pass_type_elaboration_source_binding_context_shadowing_001.miz` traverses the
frontend, resolver shells, producer, `TypedAst`, and `ResolvedTypedAst`. Its
runner test reconstructs corruption inputs only from those real opaque shell
ids, covers the frozen corruption/recovery/atomicity matrix, and keeps every
later type, fact, obligation, formula, statement, and proof payload empty.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceItemRole` | `#[non_exhaustive]`; callers must tolerate later source-item roles. |
| `SourceItemVisibility` | `#[non_exhaustive]`; Task 248 accepts only `Unspecified`. |
| `SourceItemRecovery` | `#[non_exhaustive]`; callers must handle later recovery states. |
| `SourceBindingContextOwner` | `#[non_exhaustive]`; callers must tolerate later owner forms. |
| `SourceBindingSiteRole` | `#[non_exhaustive]`; callers must tolerate later binding roles. |
| `SourceBindingContextBuild` | `#[non_exhaustive]`; callers must distinguish complete and incomplete results. |
| `SourceContextError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Task 248 Classification

| Class | Result |
|---|---|
| `test_gap` | Closed for Profile A; the frozen Profile-B focused Rust matrix remains open until the separate implementation. Broader canonical shapes remain with MC-G011/MC-G016. |
| `source_drift` | Repaired for Profile A; the closed profile gate and missing private Profile-B extractor remain bounded until the separate implementation. |
| `design_drift` | The Profile-B contract is synchronized here and in paired audits, plans, todos, and harness records. |
| `boundary_violation` | No current violation; shell fabrication and syntax imports are forbidden. |
| `spec_gap` | Only same-identifier re-reservation replacement/duplicate semantics remain undefined; this nonblocking gap does not authorize implementation. |
| `repo_metadata_conflict` | None observed. |

## Task 258A Downstream Exclusion

Task 258A reuses the Task-248 binding/context model as authority, but the
current exact `SourceBindingContextHandoff` profiles do not admit a
reserve-plus-theorem source transaction. The statement producer therefore
receives the Task-48-derived `BindingEnv` directly and must not fabricate,
extend, or install a Task-248 handoff. Task 258A owns its one theorem
visibility row in `source_statement`; this module retains its existing
tables, profiles, API, tests, counts, and hashes unchanged.

The later typed owner is exclusive. Production can only construct Task 248
first, after which Task 258A fails with
`TypedAstError::InvalidSourceStatement`; there is no Task-248
post-construction installer and this task adds none. The reverse logical
attempt is checker-test-only through
`with_source_context_for_test`, which executes the same validation and fails
with `TypedAstError::InvalidSourceContext`. Final assembly rejects
coexistence prepared only through `inject_source_statement_for_test` with
`ResolvedTypedAstError::InvalidSourceStatement`. Tests cover the production
direction, named reverse test seam, final rejection, byte-identical rollback,
and valid single-owner replay.

## Task 248 Two-Parameter Profile-Extension Frozen Contract

### Authority And Dependency Purpose

This section is the documentation prerequisite for the lower-stage extension
required by Checker Task 259. Canonical authority is:

- Chapter 4 Sections 4.2 and 4.6: a declaration creates a binding identity;
  same-scope redeclaration is rejected while only an inner binding may shadow
  an outer binding;
- Chapter 9 Sections 9.1 and 9.3--9.5: predicates have ordered typed
  parameters, definition-local assumptions, definiens, and correctness
  properties;
- Chapter 9 Sections 9.9.3--9.9.5: parameter types and guards constrain later
  logical meaning without granting this lower producer proof semantics;
- Chapter 18 Sections 18.2.1, 18.2.6, and 18.6: leading `let` declarations are
  ordered definition-block parameters whose scope is shared by the block; and
- Appendix A plus the existing parser/resolver fixtures: the concrete
  `DefinitionParameter` and `DefinitionBlock` shapes and the opaque real
  declaration-shell identity.

This authority is sufficient to preserve two ordered, separately written
definition-parameter identities in one scope. It does not authorize predicate
meaning, guard composition, property proof, type normalization, use-site
resolution, or any same-scope duplicate behavior beyond rejection. No human
semantic decision is required for this bounded transport.

Fresh inventory classifies the missing contract as `design_drift`, the
closed current profile gate and missing private extractor as bounded
`source_drift`, and the absent focused Rust matrix as `test_gap`. There is no
blocking `spec_gap`, `source_undocumented_behavior`,
`test_expectation_drift`, current `boundary_violation`, or
`repo_metadata_conflict`. A Task-259-private `BindingEnv` or private
`BindingId` reconstruction would be a `boundary_violation`; therefore this
extension is a separate Task-248 documentation task and a later separate
Task-248 implementation task.

### Exact Real Consumer

The only new consumer is the 165-byte, final-LF future Task-259 source frozen
in [`source_predicate_definition.md`](./source_predicate_definition.md), with
SHA-256
`91bdb5f51c0ea5f07bdd831700cb9803f2aa57e005921c7e4e1798ecbbf2bd9f`.
Task 259 owns exact-source selection and the full 71-row/root-70 normal AST.
Task 248 consumes only its direct lower slice:

| Surface row | Range | Task-248 meaning |
| ---: | --- | --- |
| `DefinitionBlockItem` 67 | `0..164` | sole top-level source item and real resolver shell 0 |
| `DefinitionParameter` 41 | `13..26` | parameter 0, `let x be set;` |
| identifier `x` | `17..18` | declaration range for binding 0 |
| type expression `set` | `22..25` | written type range for binding 0 |
| `DefinitionParameter` 45 | `29..42` | parameter 1, `let y be set;` |
| identifier `y` | `33..34` | declaration range for binding 1 |
| type expression `set` | `38..41` | written type range for binding 1 |

The real `DeclarationShellSet` supplies shell 0, ordinal 0,
`DeclarationShellKind::DefinitionBlock`, node 67, module/source/range,
normal recovery, no parent, and unspecified visibility. The resolver has no
parameter shells and its predicate projection has empty parameter/binder
collections and no syntactic arity. The private runner therefore derives
`LocalTermBinding` values only from the exact direct parameter syntax,
shell-derived scope `[0]`, source ordinals `0/1`, and declaration ranges. It
does not claim that the resolver predicate projection supplied either
parameter.

### Profile Preservation And Closed Admission

Profile A remains byte-for-byte behaviorally unchanged:

- exactly two top-level items, reserve then definition block;
- one reserve binding and, for a normal definition, one same-spelling local
  definition parameter with the existing structural shadow link;
- the existing recovered-definition-with-zero-bindings
  `SourceBindingContextIncomplete` result and diagnostic; and
- the current real fixture, active route, Task-249 co-installation, public
  errors, debug header, tables, counts, and semantics.

Profile B admits exactly:

- one normal top-level `DefinitionBlock` item, shell ordinal 0, no parent,
  unspecified visibility, lexical scope `[0]`, and no reserve item;
- two normal ordered `DefinitionParameter` rows owned by that shell, with
  source ordinals and resolver-local visible-after ordinals `0` and `1`;
- two nonempty distinct spellings in the syntax-free checker input, distinct
  declaration/type ranges and typed sites, the same module/source/shell/scope,
  and no same-scope duplicate; and
- no recovered item or binding, no third parameter, no additional source
  item, no reserve/definition hybrid, and no partial payload.

The public checker validates identities and structural consistency without
knowing literal source text or builtin type names. The private real-source
extractor additionally requires literal `x`, then `y`, and two bare
unattributed builtin `set` type expressions at the exact ranges above.
Unsupported cardinality/role combinations continue to fail as
`UnsupportedTaskShape`; existing more-specific validation errors still win
when generic corruption is detected first. `MissingRequiredShadow` applies
only to normal Profile A. Profile B never fabricates a shadow.

### Existing Syntax-Free ABI And Exact Tables

The implementation adds no public type, enum variant, error variant, method,
field, trait implementation, or crate dependency. It only broadens
`SourceBindingContextProducer::build`'s closed profile discriminator while
retaining the existing syntax-free:

```rust
SourceBindingContextInput
SourceItemInput
SourceBindingSiteInput
SourceBindingContextBuild
SourceBindingContextProjection
SourceBindingContextHandoff
```

For Profile B, source order produces exactly:

- one `SourceItemId(0)` for definition shell 0, with binding/local context 1,
  predecessor `None`, and the caller-supplied definition site;
- two `SourceDeclarationId`s and two `BindingId`s in `x`, `y` order;
- declaration predecessor links `None`, then declaration 0, but
  `shadowed_binding = None` for both rows;
- two active `BindingKind::DefinitionParameter` rows with
  `BinderIdentity::ResolverLocal { scope: [0], ordinal: 0/1,
  declaration_range: 17..18/33..34 }`,
  `BindingTypeSite::Source(22..25/38..41)`, empty captures, and owner context
  1;
- binding context 0 owned by the module with no bindings, and context 1 owned
  by definition shell 0 with parent 0, bindings and visible bindings `[0,1]`,
  lexical scope `[0]`, and normal recovery;
- local context 0 owned by the caller-supplied module site and empty, and local
  context 1 owned by the caller-supplied definition site, parent 0, with the
  two caller-supplied parameter sites and no facts or assumptions; and
- two context links: module context 0 to local context 0/item `None`, then
  definition context 1 to local context 1/item 0.

Thus the item/declaration/binding/binding-context/local-context/context-link/
diagnostic cardinalities are exactly `1/2/2/2/2/2/0`. The existing
`source-binding-context-debug-v1` header and row grammar remain unchanged.
Equal inputs produce equal handoffs and byte-identical debug output.

### Private Runner Extractor

The matching implementation adds this runner-private, dormant lower helper:

```rust
pub(in crate::runner) struct SourceTwoParameterDefinitionContextSites {
    pub module: TypedSiteRef,
    pub definition: TypedSiteRef,
    pub parameters: [TypedSiteRef; 2],
}

pub(in crate::runner) fn source_two_parameter_definition_context_projection(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    definition_node: SurfaceNodeId,
    nodes: &TypedArena,
    sites: SourceTwoParameterDefinitionContextSites,
) -> Result<SourceBindingContextProjection, String>;
```

The Task-259 caller remains responsible for exact source selection, full
AST/resolver authentication, typed-arena allocation, and supplying four
distinct sites from that single shared arena. The helper authenticates the
one real top-level definition shell, the two leading direct normal parameter
subtrees, exact token/range/type payloads, their shell-derived local scope,
and the Task-248 projection. Before publication it also resolves all four
sites against `nodes`, requires the module site to be the arena root, and
authenticates each normal node's exact range and local-context link
(`0` for the module and `1` for the definition and parameters). It returns
only the existing projection and does not create a `TypedAst`,
`ResolvedTypedAst`, Task-249 handoff, active runner result, diagnostic detail
key, or corpus selector.

After the two leading parameters, every guard, predicate definition,
property, justification, term, formula, proof, token wrapper, and descendant
subtree is excluded: the helper neither descends into it nor publishes a row
for it. It may scan direct child kinds only to reject an additional or
non-leading `DefinitionParameter`. Task 259 and later owners retain those
subtrees. The existing active `source_binding_context_output` remains
unchanged and cannot select the future source.

### Frozen Tests And Write Scope

The later implementation adds exactly four runner-library tests:

1. `task248_two_parameter_definition_profile_publishes_dense_context`;
2. `task248_two_parameter_definition_profile_rejects_corruption`;
3. `task248_two_parameter_definition_extractor_is_default_deny`; and
4. `task248_two_parameter_definition_installation_is_transactional_and_deterministic`.

The profile test freezes checker validation precedence, not merely rejection.
Every mutation asserts the exact `SourceContextError` variant and `index`
where present:

| Mutation | Exact checker result |
| --- | --- |
| no item | `MissingItems` |
| one normal definition item, zero bindings | `PartialItem { index: 0 }` |
| one normal definition item, one otherwise valid binding | `UnsupportedTaskShape` |
| one normal definition item, three otherwise valid bindings | `UnsupportedTaskShape` |
| recovered definition, zero bindings | `UnsupportedTaskShape` |
| recovered definition claiming binding 0 | `RecoveredItemClaimsBinding { index: 0 }` |
| recovered binding row `i` | `RecoveredBinding { index: i }` |
| duplicate same-scope spelling on row 1 | `DuplicateSameScopeBinding { index: 1 }` |
| stale source/local ordinal on row `i` | `StaleBindingOrdinal { index: i }` or `StaleLocalIdentity { index: i }`, according to the corrupted field |
| reordered declaration range on row 1 | `ReorderedBindings { index: 1 }` |
| wrong shell or context owner on row `i` | `UnknownBindingShell { index: i }` or `RoleMismatch { index: i }` |
| empty spelling on row `i` | `EmptyBindingSpelling { index: i }` |
| item module/source mismatch | `ModuleMismatch { index: 0 }` / `ItemSourceMismatch { index: 0 }` |
| binding source/out-of-item range on row `i` | `BindingSourceMismatch { index: i }` / `BindingRangeMismatch { index: i }` |
| stale shell ordinal or invalid parent/context/visibility | `StaleShellOrdinal { index: 0 }`, `InvalidParent { index: 0 }`, `InvalidItemContext { index: 0 }`, or `UnsupportedVisibility { index: 0 }` |
| duplicate shell or any duplicate typed site | `DuplicateShell { index: 1 }` or `DuplicateTypedSite` |
| coherent reserve/definition hybrid, extra item, or unsupported role/cardinality | `UnsupportedTaskShape` |

Valid Profile B and a syntax-free distinct-name substitution both complete:
literal `x`/`y` is runner authentication, not checker syntax knowledge.
Duplicate spelling fails with `DuplicateSameScopeBinding`, and no Profile-B
input, valid or corrupt, may return `MissingRequiredShadow`; that error is
asserted only by the preserved Profile-A matrix. Generic field errors must
continue to precede the closed profile discriminator exactly as they do now.

The extractor/default-deny test independently mutates every private
authentication predicate:

- cross-wired AST/shell/module inputs; missing, duplicate, wrong-kind,
  recovered, wrong-node, wrong-range, wrong-parent, wrong-ordinal, or wrong-
  visibility definition shell;
- non-direct, nested, non-leading, reordered, missing, duplicated, or third
  `DefinitionParameter` children;
- every `let`/`;`/identifier/`be` token, literal `x`/`y` order, one-segment
  topology, type-node kind, bare form, builtin `set` head/spelling,
  attributes, declaration/type range, scope, local identity, and ordinal;
- missing arena nodes, a role/non-root/wrong module site, cross-wired sites,
  wrong anchor or context for each module/definition/parameter node,
  recovered or degraded nodes, and every duplicate-site pairing; and
- an acceptance-invariance pair that changes only normal excluded
  guard/predicate/property/justification descendant tokens at equal ranges
  and proves the Task-248 projection/debug bytes are unchanged.

Each negative fails at the private helper before publication. The test uses
real parser/resolver output and `TypedArenaBuilder`; a synthetic shell id or
unchecked opaque site is not sufficient evidence.

Dormancy is asserted explicitly. For the exact future source, unchanged
`source_binding_context_output` and `source_binding_context_detail_keys`
both return `None`; no expectation field or metadata selects Profile B.
Installing the returned projection through the existing typed/final path
leaves source type, attribute, evidence, term, application, structure,
set-term, atomic/composite/composition, statement, and Task-259 handoffs
absent. Types, facts, coercions, obligations, diagnostics, checked formulas,
statement semantics, proofs, terminal goals, and all resolved semantic
tables remain empty.

The positive test freezes this exact handoff debug string, using test module
path `task248.two_parameter_profile`:

```text
source-binding-context-debug-v1
module: task248.two_parameter_profile
item#0 shell=0 ordinal=0 role=definition-block range=0..164 parent=none context=1 local_context=1 predecessor=none
declaration#0 item=0 binding=0 ordinal=0 role=definition-parameter range=17..18 type_range=22..25 context=1 local_context=1 shadowed=none predecessor=none
declaration#1 item=0 binding=1 ordinal=1 role=definition-parameter range=33..34 type_range=38..41 context=1 local_context=1 shadowed=none predecessor=0
context-link#0 binding_context=0 local_context=0 item=module
context-link#1 binding_context=1 local_context=1 item=0
```

Typed debug must contain that literal block exactly once immediately after
its `typed-ast-debug-v1` module/root/resolved-root prelude; final debug must
contain the identical block exactly once at the unchanged source-context
position. Full typed/final strings are compared across replay, and
Profile-A's existing full-string/conditional debug oracles remain unchanged.

Together the four tests cover the exact real AST/shell/ranges/types, all
table and identity fields, every profile discriminator, complete corruption
and exclusion matrices, transactional typed installation, final clone, and
deterministic serialization. No `.miz` fixture, sidecar, expectation, or
trace row is added because this lower helper is dormant until the Task-259
real consumer is implemented.

The exact later Rust write scope is:

- `crates/mizar-checker/src/source_context.rs`;
- `crates/mizar-test/src/runner/type_elaboration/source_context.rs`;
- `crates/mizar-test/src/runner/type_elaboration.rs`;
- `crates/mizar-test/src/runner/tests/support.rs`; and
- `crates/mizar-test/src/runner/tests/type_elaboration/source_context.rs`.

Production path counts remain checker 23 and runner 30. The checker library
test list remains 430; the four named tests project the runner library from
504 to 508. Resolver 144 and syntax 59 remain unchanged. Implementation must
fresh-measure all line counts and hashes rather than treating these projected
deltas as evidence.

### Documentation Baseline, Audit Impact, And Exit

This documentation prerequisite changes only synchronized derived design
records. Production/test source, specifications, existing `.miz` fixtures,
sidecars, expectations, `tests/coverage/spec_trace.toml`, trace status,
mapping, backlink, owner, active outcome, coverage credit, counts, and CLI
hashes remain unchanged. The frozen current metadata remains
cases/requirements `421/389`, pass/fail `228/193`, active
parse/declaration/type/proof `101/7/198/1`, declaration requirements
`12 = 7 covered + 5 partial`, type requirements
`253 = 241 covered + 12 deferred`, and warnings/errors `23/0`.

`doc/design/spec_coverage_audit.md` receives narrative dependency ownership
only; the trace file is a deliberate byte-level no-op. Exit requires
synchronized EN/JA design, findings-free specification review, docs-only
scope and verification, all nine protocol hard gates, a valid independent
quality score of at least 90/100, exact task-only staging, one dedicated
documentation commit, clean post-commit inventory, and unchanged protected
stash. Fresh inventory then selects the separate five-Rust-file Task-248
implementation, not Task 259.
