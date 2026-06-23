# Core IR

> Canonical language: English. Japanese companion:
> [../ja/core_ir.md](../ja/core_ir.md).

## Purpose

`CoreIr` is the backend-neutral logical representation produced by
`mizar-core` phase 9. It is the first non-source-shaped semantic layer after
`ResolvedTypedAst`: terms, formulas, proof skeletons, definitions, algorithm
shells, provenance, and obligation seeds are normalized for later
control-flow preparation, VC generation, deterministic discharge, ATP
translation, and kernel checking.

This task-2 document specifies the data shape and invariants only. It does not
implement lowering, `ControlFlowIr`, VC generation, artifact schemas, proof
acceptance, or kernel replay.

## References

- [architecture 01](../../architecture/en/01.ir_layers.md) defines `CoreIr`
  and `ControlFlowIr` as immutable internal IR layers.
- [architecture 06](../../architecture/en/06.elaboration_and_core_ir.md)
  defines phase-9 elaboration responsibilities and the initial `CoreIr`
  interface shape.
- [architecture 07](../../architecture/en/07.vc_generation.md) defines how
  obligation seeds later become concrete `VcId`s.
- [architecture 16](../../architecture/en/16.substitution_and_binding.md)
  defines binder identity, alpha-equivalence, and substitution replay.
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md)
  defines the split between snapshot-local `VcId`s and cross-edit
  `ObligationAnchor` candidates.
- [checker resolved typed AST](../../mizar-checker/en/resolved_typed_ast.md)
  defines the source-shaped semantic input to elaboration.
- [spec 03](../../../spec/en/03.type_system.md), [spec 13](../../../spec/en/13.term_expression.md),
  [spec 14](../../../spec/en/14.formulas.md), [spec 16](../../../spec/en/16.theorems_and_proofs.md),
  and [spec 20](../../../spec/en/20.algorithm_and_verification.md) define the
  language behavior represented by core nodes.

## Responsibility

`core_ir` owns:

- dense ids and tables for core items, terms, formulas, definitions, proof
  trees, algorithm shells, generated-origin records, and obligation seeds;
- backend-neutral term and formula node shapes after name, type,
  registration, overload, and inserted-view decisions are final;
- stable definition expansion boundaries without eager inlining;
- proof-skeleton data that records thesis transitions, assumptions, citations,
  and terminal goals without performing proof search;
- algorithm-shell data that preserves lowered contracts, ghost/runtime
  classification, source-shaped statement order, and phase-10 handoff material
  without building a CFG;
- source/core provenance for every diagnostic-, obligation-, or
  artifact-facing node;
- deterministic debug rendering used by task-local tests, snapshots, and
  audits.

`core_ir` does not own:

- source walking or source-to-checker payload extraction;
- name, type, registration, cluster, overload, or proof search;
- capture-avoiding substitution algorithms or alpha-equivalence decisions,
  which are specified in `binder_normalization.md`;
- `ControlFlowIr` construction, use-before-assignment analysis, or unreachable
  diagnostics;
- concrete `VcId` assignment, `VcIr`, ATP encodings, proof certificates,
  verified artifact schemas, cache records, or public diagnostic code-space.

## Data Model

All ids are dense within one `CoreIr` snapshot unless a field explicitly names
an upstream or downstream id. Dense ids are deterministic by insertion order
defined in this document and the lowering specs. Public enum types are
forward-compatible unless task 21 records an explicit exhaustive exception.

Illustrative Rust names below are normative for concepts and relationships, but
task 3 may choose ergonomic concrete fields while preserving these invariants.

```rust
struct CoreIr {
    module_id: ModuleId,
    source_id: SourceId,
    items: CoreItemTable,
    terms: CoreTermTable,
    formulas: CoreFormulaTable,
    definitions: CoreDefinitionTable,
    proofs: CoreProofTable,
    proof_nodes: CoreProofNodeTable,
    algorithms: CoreAlgorithmTable,
    algorithm_statements: CoreAlgorithmStmtTable,
    generated: GeneratedOriginTable,
    obligation_seeds: ObligationSeedTable,
    source_map: CoreSourceMap,
    diagnostics: CoreDiagnosticTable,
}
```

The root `CoreIr` is immutable once constructed. Builders may exist, but they
must validate references before producing the immutable value.

### Core Items

`CoreItem` is the module-level logical boundary. Each accepted or partially
lowered declaration that later phases may inspect has exactly one item row.

```rust
struct CoreItem {
    id: CoreItemId,
    symbol: SymbolId,
    kind: CoreItemKind,
    visibility: CoreVisibility,
    status: CoreItemStatus,
    dependencies: Vec<CoreItemId>,
    source: CoreSourceRef,
    diagnostics: Vec<CoreDiagnosticId>,
}
```

`CoreItemKind` covers at least:

- structures, modes, attributes, predicates, functors, theorems, lemmas,
  schemes/templates after instantiation metadata is concrete, registrations,
  reductions, local generated definitions, and algorithms.

`CoreItemStatus` distinguishes valid, skipped, partial, and error-preserving
items. A skipped or partial item may retain source maps and diagnostics, but it
must not provide verified premises to downstream phases.

Ordering:

- module items follow canonical source order after dependency summaries are
  loaded;
- generated items are ordered by owning item, generated-origin kind, local path,
  and normalized key;
- skipped/error items remain in traversal order so diagnostics stay stable.

### Core Terms

`CoreTerm` represents logical terms after overload and inserted-view decisions
are final.

```rust
struct CoreTermNode {
    id: CoreTermId,
    kind: CoreTermKind,
    source: CoreSourceRef,
}

enum CoreTermKind {
    Var(CoreVarId),
    Const(SymbolId),
    Apply { functor: SymbolId, args: Vec<CoreTermId> },
    Select { selector: SymbolId, base: CoreTermId },
    Tuple(Vec<CoreTermId>),
    SetEnum(Vec<CoreTermId>),
    Generated { origin: GeneratedOriginId, args: Vec<CoreTermId> },
    Error(CoreDiagnosticId),
}
```

Rules:

- `Var` uses canonical core variable ids, not display names.
- `Apply` functors and `Const` symbols are canonical `SymbolId`s.
- Stable choice terms such as `the T` are represented as ordinary `Apply`
  nodes whose functor is the generated `choice_T` symbol and whose arguments
  are the captured free parameters. The corresponding `GeneratedOrigin` record
  owns the generated symbol key and evidence. There is no magic choice-node
  semantics in core, and `CoreTermKind::Generated` must not be used for stable
  choice terms.
- Fraenkel comprehensions lower to generated set-valued terms with captured
  free parameters, sethood evidence in the generated-origin record, and a
  definitional membership-axiom obligation seed for the generated set.
- Source-written or inserted `qua` views do not change the underlying term;
  their evidence is represented in provenance, explicit predicates, and
  obligation seeds.
- `Error` terms are first-class recovery nodes. They are never accepted as
  valid logical terms.

### Core Formulas

`CoreFormula` represents logical propositions and type predicates.

```rust
struct CoreFormulaNode {
    id: CoreFormulaId,
    kind: CoreFormulaKind,
    source: CoreSourceRef,
}

enum CoreFormulaKind {
    True,
    False,
    Atom { predicate: SymbolId, args: Vec<CoreTermId> },
    Equals { left: CoreTermId, right: CoreTermId },
    TypePred { subject: CoreTermId, ty: CoreTypePredicate },
    Not(CoreFormulaId),
    And(Vec<CoreFormulaId>),
    Or(Vec<CoreFormulaId>),
    Implies { premise: CoreFormulaId, conclusion: CoreFormulaId },
    Iff { left: CoreFormulaId, right: CoreFormulaId },
    Forall { binders: Vec<CoreBinder>, body: CoreFormulaId },
    Exists { binders: Vec<CoreBinder>, body: CoreFormulaId },
    Error(CoreDiagnosticId),
}
```

Rules:

- Type erasure always leaves explicit `TypePred` formulas, local assumptions,
  view provenance, diagnostics, or obligation seeds.
- Conjunctions generated by type erasure use stable predicate ordering.
- Quantifier binders are represented by `CoreBinder` rows compatible with
  `binder_normalization.md`.
- Formula nodes do not preserve surface precedence, parentheses, or notation
  spelling.

### Binders And Variables

```rust
struct CoreBinder {
    var: CoreVarId,
    role: CoreVarRole,
    ty_guard: Option<CoreFormulaId>,
    source_name: Option<String>,
    source: CoreSourceRef,
}
```

`source_name` is diagnostic-only. Semantic equality, hashing, substitution, and
normalization use `CoreVarId` plus the canonical binder representation selected
by task 4. Free-variable sets and substitution side conditions are represented
by the binder module, but core nodes must store enough binder/source
provenance for that module to run without inspecting source syntax.

### Definitions And Expansion Boundaries

`CoreDefinitionTable` records the semantic boundary for every definition that
can be unfolded by later phases.

```rust
struct CoreDefinition {
    id: CoreDefinitionId,
    item: CoreItemId,
    symbol: SymbolId,
    params: Vec<CoreBinder>,
    body: DefinitionBody,
    expansion: ExpansionPolicy,
    correctness: Vec<ObligationSeedId>,
    generated_dependencies: Vec<GeneratedOriginId>,
    source: CoreSourceRef,
}
```

`DefinitionBody` distinguishes term definiens, formula equivalence, guarded
definiens branches, algorithm-backed computable bodies, and unavailable/error
bodies.

`ExpansionPolicy` covers at least opaque, transparent, reducible, and
computable policies. The policy permits later phases to unfold or reduce a
definition; it never forces eager inlining during elaboration.

Guarded definitions preserve branch order. Overlap, coverage, compatibility,
coherence, existence, uniqueness, and reducibility checks are represented as
obligation seeds, not as already-accepted proof results.

### Proof Table

`CoreProofTable` records proof skeletons, not proof acceptance.

```rust
struct CoreProof {
    id: CoreProofId,
    item: CoreItemId,
    proposition: CoreFormulaId,
    root: CoreProofNodeId,
    status: CoreProofStatus,
    source: CoreSourceRef,
}

enum CoreProofStatus {
    Open,
    Assumed,
    Conditional,
    Error,
}

enum CoreProofNodeKind {
    IntroduceBinder { binder: CoreBinder, child: CoreProofNodeId },
    Assume { label: Option<CoreLabelRef>, formula: CoreFormulaId, child: CoreProofNodeId },
    Step { label: Option<CoreLabelRef>, formula: CoreFormulaId, justification: CoreJustification },
    CurrentGoal { thesis: CoreFormulaId, child: CoreProofNodeId },
    Sequence { children: Vec<CoreProofNodeId> },
    Branch { kind: ProofBranchKind, children: Vec<CoreProofNodeId> },
    TerminalGoal { obligation: ObligationSeedId, citations: Vec<CoreCitation> },
    Error(CoreDiagnosticId),
}
```

Rules:

- `thesis` is resolved to the current `CoreFormulaId`; it is not preserved as a
  magic identifier. Explicit current-goal transitions are represented by
  `CurrentGoal` nodes.
- `Sequence` nodes preserve ordered proof blocks and carry labels along one
  proof path; `Branch` children keep isolated sibling scopes.
- Citations remain symbolic references to labels, proof-like symbols
  (`Theorem`, `Lemma`, or `Scheme`) from the current module or dependency
  summaries, or generated origins. Core does not decide ATP premise selection,
  and non-proof local symbols such as functors or modes are not valid proof
  citations. Raw `CoreIr` validation enforces the kind of local symbols present
  in the item table; external dependency-symbol citation kinds are guaranteed
  by elaborator/context validation before Core IR construction and remain
  symbolic in this table set.
- Terminal goals store their durable citation list on the terminal proof node
  as well as referencing the generated theorem-proof obligation seed.
- `open`, `assumed`, and `conditional` statuses are preserved as policy input.
  Core does not accept or reject the proof.
- `error` is a recovery status only; it records malformed proof skeleton input
  without accepting or rejecting the proof.
- Every terminal proof obligation references an `ObligationSeedId`.

### Algorithm Shells

`CoreAlgorithmTable` holds phase-9 algorithm shells. It is not
`ControlFlowIr`.

```rust
struct CoreAlgorithm {
    id: CoreAlgorithmId,
    item: CoreItemId,
    symbol: SymbolId,
    params: Vec<CoreBinder>,
    result: Option<CoreBinder>,
    contracts: CoreContractSet,
    statements: Vec<CoreAlgorithmStmtId>,
    ghost_effects: Vec<GhostEffectKey>,
    source: CoreSourceRef,
    diagnostics: Vec<CoreDiagnosticId>,
}
```

Algorithm shells preserve:

- parameter and result binders;
- lowered `requires`, `ensures`, `assert`, `invariant`, and `decreasing`
  formulas/terms;
- `Pick` sites produced by executable `the` occurrences;
- ghost/runtime classification;
- source statement order and enough local path information for phase 10.

They do not contain basic blocks, control-flow edges, use-before-assignment
facts, reachability diagnostics, or generated VCs.

`CoreAlgorithmStmtTable` owns source-ordered statement-shell rows referenced by
`CoreAlgorithm.statements`.
Every statement listed directly or nested through `If`, `While`, or `Match`
arms must have `owner` equal to the containing `CoreAlgorithmId`; phase 10 may
trust this owner relation when constructing `ControlFlowIr`.

```rust
struct CoreAlgorithmStmt {
    id: CoreAlgorithmStmtId,
    owner: CoreAlgorithmId,
    kind: CoreAlgorithmStmtKind,
    source: CoreSourceRef,
    diagnostics: Vec<CoreDiagnosticId>,
}

enum CoreAlgorithmStmtKind {
    Let { binder: CoreBinder, value: Option<CoreTermId>, ghost: bool },
    Assign { target: CorePlace, value: CoreTermId },
    Assert { formula: CoreFormulaId },
    If { condition: CoreFormulaId, then_body: Vec<CoreAlgorithmStmtId>, else_body: Vec<CoreAlgorithmStmtId> },
    While { condition: CoreFormulaId, invariants: Vec<CoreFormulaId>, decreasing: Vec<CoreTermId>, body: Vec<CoreAlgorithmStmtId> },
    Match { scrutinee: CoreTermId, arms: Vec<CoreAlgorithmMatchArm> },
    Return(Option<CoreTermId>),
    Break,
    Continue,
    Pick { binder: CoreBinder, witness_ty: Option<CoreFormulaId>, ghost: bool },
    Error(CoreDiagnosticId),
}
```

Task 3 may keep some variants minimal until task 13 and task 15 add behavior,
but it must provide an owning table, source refs, deterministic ordering, and
validation for statement references. `CoreAlgorithmStmt` rows are shells; they
do not encode CFG block ids.

## Generated Origins

Generated terms and internal symbols are tracked in `GeneratedOriginTable`.

```rust
struct GeneratedOrigin {
    id: GeneratedOriginId,
    owner: CoreItemId,
    kind: GeneratedOriginKind,
    key: GeneratedOriginKey,
    functor: Option<SymbolId>,
    params: Vec<CoreVarId>,
    evidence: Vec<CoreProvenance>,
    source: CoreSourceRef,
}
```

Kinds include stable choices, Fraenkel comprehensions, local abbreviation
expansion entries, generated type predicates, skipped/error placeholders, and
other generated bookkeeping records. Executable algorithm `Pick` bindings are
statement-local `CoreAlgorithmStmtKind::Pick` rows, not generated origins.
`GeneratedOriginKind::AlgorithmPick` is reserved for future non-executable
algorithm bookkeeping and is not emitted by task-13 shell lowering.

Generated keys use normalized semantic origins and alpha-normalized payloads.
They must not use source display names as identity. Generated names are
diagnostic-only unless an owning module spec later gives a stable artifact
projection.
Generated origins that correspond to an internal symbol record the generated
functor symbol in `functor`; origins that describe non-term bookkeeping may
leave it absent.

## Obligation Seeds

An `ObligationSeed` is the phase-9/10 handoff unit consumed by `mizar-vc`.
It is not a `VcId`, not proof evidence, and not an `ObligationAnchor` by
itself.

```rust
struct ObligationSeed {
    id: ObligationSeedId,
    owner: CoreItemId,
    kind: ObligationSeedKind,
    goal: Option<CoreFormulaId>,
    context: Vec<CoreFormulaId>,
    local_path: LocalProofOrProgramPath,
    label: Option<CoreLabelRef>,
    semantic_origin: NormalizedSemanticOrigin,
    provenance: Vec<CoreProvenance>,
    source: CoreSourceRef,
    core_refs: Vec<CoreNodeRef>,
    status: ObligationSeedStatus,
    diagnostics: Vec<CoreDiagnosticId>,
}
```

Goal invariant:

- `status = Active` requires `goal = Some(_)`.
- `status = Skipped`, `Deferred`, or `Error` may use `goal = None` only when
  the seed is preserved for traceability after an invalid/skipped item,
  external dependency gap, or failed lowering site.
- A seed with `goal = None` must carry a diagnostic or provenance reason and
  must not be converted into a concrete VC.
- If a seed kind expands into multiple future VCs, the seed still records the
  aggregate normalized goal or a generated conjunction goal; the split belongs
  to `mizar-vc`.

Seed kinds cover at least:

- theorem or lemma proof terminal goals;
- definition existence, uniqueness, coherence, compatibility, coverage,
  overlap consistency, and reducibility correctness;
- checker-initial obligations carried forward from type/coercion checking;
- non-emptiness or sethood obligations for generated choice/comprehension
  terms;
- definitional membership axioms for generated Fraenkel comprehension sets;
- algorithm preconditions, postconditions, assertions, invariants, termination
  measures, ghost-erasure safety, and phase-10 flow-derived checks after task
  18.

`local_path` must be anchor-ready:

- proof paths record proof block, branch, step, and terminal-goal positions;
- program paths record algorithm statement, branch, loop, contract, and
  generated obligation positions;
- generated paths record owner item, generated-origin kind, and normalized key.

`semantic_origin` is normalized and independent of source display spelling. It
identifies the theorem, definition, registration, generated symbol, algorithm,
or checker-origin row that caused the seed.

`provenance` must include source/core information consumed by `mizar-vc`:

- source range and resolved/checker ids when available;
- core item/term/formula/proof/algorithm references involved in the seed;
- label or citation hints when available;
- generated-origin id when the seed comes from generated material;
- erasure/view/template/proof-skeleton provenance when applicable.

Seeds are ordered deterministically by owner item, source range, local path,
kind, label, normalized semantic origin, and dense id tie-breaker.

### Obligation Seed Handoff

Task 18 exposes an `ObligationSeedHandoff` view for `mizar-vc` seed intake.
The handoff is still core-owned phase-9/10 metadata. It does not assign
`VcId`s, compute `ObligationAnchor`s, build `VcIr`, fingerprint contexts, or
decide proof acceptance.

The handoff has a distinct, snapshot-local id space:

```rust
struct ObligationSeedHandoff {
    entries: ObligationHandoffTable,
    source_map: Map<ObligationHandoffId, CoreSourceRef>,
}

struct ObligationHandoffEntry {
    seed: ObligationSeed,
    origin: ObligationHandoffOrigin,
    flow_site: Option<ControlFlowObligationSite>,
}

enum ObligationHandoffOrigin {
    ExistingCore { seed: ObligationSeedId },
    FlowDerived { flow: ControlFlowId, algorithm: CoreAlgorithmId },
}
```

`ObligationHandoffId` is local to this handoff value. It is not an
`ObligationSeedId` and not a `VcId`. The handoff source map is keyed by
`ObligationHandoffId`; existing core seeds also retain their original
`CoreSourceMap.obligation_sources` entry through
`ObligationHandoffOrigin::ExistingCore`.

`ControlFlowObligationSite` identifies a CFG-local site without embedding
control-flow ids into `CoreNodeRef`. It records the site class and the relevant
flow-local indexes, such as contract-site ordinal, assertion-site ordinal,
loop-invariant-site ordinal, termination-measure-site ordinal,
partial-termination-site ordinal, `LocalId`, `AssignmentEffectId`, `LoopId`,
`BasicBlockId`, `ControlFlowExitId`, and statement id when applicable.

The handoff contains:

- every existing `CoreIr.obligation_seeds` row, sorted by the canonical seed
  order and linked back to its original `ObligationSeedId`;
- additional phase-10 seed rows derived from `ControlFlowIr` contract,
  termination, and ghost-erasure sites, linked to the originating
  `ControlFlowId`, `CoreAlgorithmId`, and local CFG site;
- a source map for the handoff seed ids so `mizar-vc` can trace every seed to
  source without inspecting raw syntax.

Existing core seeds preserve their original `kind`, `status`, goal, context,
local path, label, normalized semantic origin, provenance, source, diagnostics,
and `CoreNodeRef`s. This covers theorem/lemma terminal goals, definition
correctness, checker-initial obligations, generated choice/comprehension
obligations, and deferred/error traceability rows already created during
elaboration.

Flow-derived seeds are generated only from explicit `ControlFlowIr` metadata:
entry `requires`, return `ensures`, algorithm and statement assertions, loop
invariants, decreasing/partial termination sites, and ghost-only local or
assignment sites. They include `CoreNodeRef::Item`, `CoreNodeRef::Algorithm`,
the formula or term when one exists, and the statement ref when the site is
statement-owned. CFG-local ids stay in the handoff entry's flow-site metadata,
not in `CoreNodeRef`, because `CoreIr` must remain independent of
`ControlFlowIr` table ids.

Flow-derived seeds are `Deferred` in task 18, even when they carry a formula
goal. The concrete VC context for assertions and invariants, caller-side
substitution for `requires`, result substitution for `ensures`, termination
well-foundedness schemas, and ghost-erasure proof shape belong to `mizar-vc`.
The deferred seed preserves the anchor-ready program path, source/core/CFG
provenance, and status without pretending the obligation is already ready for
VC generation.

The handoff order is deterministic across the combined core and flow-derived
seed set: compare each seed's canonical key, then the origin class, flow id,
site kind, and local site indexes as tie-breakers. Handoff ids are local to
the handoff snapshot and are not `VcId`s.

## Source Map And Provenance

Every core item and every term/formula/proof/algorithm node that can produce a
diagnostic, obligation seed, snapshot line, artifact projection, or later
source-mapped metadata must have a source map entry.

```rust
struct CoreSourceMap {
    item_sources: Map<CoreItemId, CoreSourceRef>,
    term_sources: Map<CoreTermId, CoreSourceRef>,
    formula_sources: Map<CoreFormulaId, CoreSourceRef>,
    definition_sources: Map<CoreDefinitionId, CoreSourceRef>,
    proof_sources: Map<CoreProofNodeId, CoreSourceRef>,
    algorithm_sources: Map<CoreAlgorithmStmtId, CoreSourceRef>,
    generated_sources: Map<GeneratedOriginId, CoreSourceRef>,
    obligation_sources: Map<ObligationSeedId, CoreSourceRef>,
}
```

`CoreSourceRef` contains:

- `SourceId` and source range when available;
- upstream `ResolvedTypedAst` node/expression/metadata ids when available;
- originating symbol, label, or checker row id when available;
- `GeneratedFrom` markers for generated nodes;
- a sorted list of `CoreProvenance` entries.

`GeneratedFrom` is required when a node has no direct source range. It records
the owning source/core node, generated-origin kind, normalized key, and reason.
When a `GeneratedFrom` marker names an item owner, its `(owner, kind, key)` must
correspond to exactly one `GeneratedOrigin` row. Generated-origin rows are
unique by owner item, kind, and normalized key. Stable-choice terms still lower
to ordinary `Apply(choice_T(...))` terms; the `GeneratedOrigin` row records the
generated symbol and evidence, not a magic term node.

Source maps are required data, not debug extras. Task 3 tests must ensure every
node reachable from `CoreItem`s maps to direct source or carries
`GeneratedFrom`.

## Diagnostics And Error Nodes

Core diagnostics classify boundary failures such as:

- unresolved or blocked semantic input from `ResolvedTypedAst`;
- invalid type erasure or missing view evidence;
- unsupported lowering for a source construct;
- malformed proof skeleton;
- malformed generated-origin or source-map data;
- algorithm shell lowering failures.

Diagnostics are local structured records. Public diagnostic code allocation is
deferred to `mizar-diagnostics`.

The local diagnostic table has a minimal deterministic shape:

```rust
struct CoreDiagnostic {
    id: CoreDiagnosticId,
    class: CoreDiagnosticClass,
    severity: CoreDiagnosticSeverity,
    recovery: CoreDiagnosticRecovery,
    message_key: CoreDiagnosticMessageKey,
    primary_source: CoreSourceRef,
    related: Vec<CoreSourceRef>,
    owner: Option<CoreNodeRef>,
}
```

`message_key` is a crate-local stable key for tests and debug rendering, not a
public diagnostic code. Diagnostic rows are ordered by primary source range,
owner node, class, message key, and dense id tie-breaker. Related source refs
are sorted by phase, source range, and provenance. Error nodes must reference a
diagnostic row whose owner node, primary source, or `GeneratedFrom` marker
explains the failed lowering site.

Error nodes preserve invalid lowering sites without turning them into valid
logical terms or formulas. Downstream phases must treat `Error` nodes and
skipped/partial items as non-verified input.

## Deterministic Debug Rendering

Task 3 implements a deterministic debug renderer for the data shapes in this
document. Rendering is internal and test-facing; it is not a stable published
artifact schema.

Rendering rules:

- tables render in dense id order;
- symbol ids, source ids, labels, local paths, and generated keys use canonical
  textual forms;
- maps render by sorted key;
- provenance lists render by phase, source range, semantic origin, and dense
  id;
- error and skipped nodes render explicitly;
- source display names may appear for diagnostics, but semantic equality and
  generated keys must not depend on them.

## Validation And Test Obligations

Task 3 must add Rust tests for:

- constructing a minimal `CoreIr` with each core table present;
- stable dense ids and deterministic debug rendering;
- validation rejecting invalid references between items, terms, formulas,
  definitions, proofs, algorithms, algorithm statements, generated origins,
  obligation seeds, diagnostics, and source maps;
- every node reachable from items mapping to source or `GeneratedFrom`;
- `ObligationSeed` ordering and preservation of local paths, labels,
  normalized semantic origins, source/core provenance, and status;
- active obligation seeds requiring a goal, while skipped/deferred/error seeds
  without a goal carry diagnostic/provenance reasons and never become VCs;
- error nodes remaining explicit and non-verified.

No `.miz` fixture is required for task 2. Source-derived pass coverage remains
deferred until checker payload extraction and mizar-test stage support can feed
core lowering without fabrication.

## Public Enum Policy

Task 21 classifies every `core_ir` public enum as a downstream
forward-compatible API surface. Each enum must remain `#[non_exhaustive]` so
future semantic categories can be added without breaking downstream exhaustive
matches.

| Public enum | Decision |
|---|---|
| `CoreItemKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreItemStatus` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreTermKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreFormulaKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DefinitionBody` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DefinitionBranchBody` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ExpansionPolicy` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreProofStatus` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreProofNodeKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreCitation` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ProofBranchKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreAlgorithmStmtKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `GeneratedOriginKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ObligationSeedKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ObligationSeedStatus` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreSourceAnchor` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreProvenancePhase` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreDiagnosticClass` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreDiagnosticSeverity` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreDiagnosticRecovery` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreNodeRef` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoreIrError` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module. Internal
`mizar-core` matches may remain exhaustive where they deliberately enumerate the
current variants.

## Drift And Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| COREIR-G001 | `spec_gap` | No `core_ir.md` existed before task 2. | This document closes the module-spec gap for task 3. |
| COREIR-G002 | `test_gap` | `core_ir` source and tests do not exist before task 3. | Task 3 implements data shapes and Rust tests listed above. |
| COREIR-G003 | `external_dependency_gap` | Source-to-checker extraction and source-derived semantic pass fixtures remain deferred by checker closeout. | Use explicit Rust fixtures for task 3; defer source-derived `.miz` core fixtures until payload seams exist. |
| COREIR-G004 | `external_dependency_gap` | `mizar-vc`, `mizar-kernel`, and `mizar-proof` crates are not workspace members yet. | Specify seed and provenance shape only; do not implement downstream consumers. |
| COREIR-G005 | `deferred` | Published artifact schemas and public diagnostic code allocation belong to later crates. | Keep debug rendering internal and diagnostics local. |

## Forbidden Behavior

`core_ir` implementations must not:

- inspect raw syntax or perform source-to-checker extraction;
- run name/type/registration/overload/proof search;
- activate registrations or cluster rules;
- eagerly inline definitions outside explicit expansion policy;
- assign `VcId`s or cross-edit proof reuse identities;
- emit artifact schemas, ATP encodings, proof certificates, cache records, or
  public diagnostic codes;
- treat generated display names or source spelling as semantic identity.
