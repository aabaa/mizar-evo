# Resolved Typed AST

> Canonical language: English. Japanese companion:
> [../ja/resolved_typed_ast.md](../ja/resolved_typed_ast.md).

## Purpose

`ResolvedTypedAst` is the final checker-owned, source-shaped semantic AST
before elaboration. It projects the phase-6 typed source shape plus phase-7
cluster fact rows/provenance ids and phase-8 overload selections into one
immutable layer that LSP, artifacts, VC generation, and elaboration can consume
without re-running name resolution, type checking, registration closure, or
overload resolution.

This document specifies the task-28 data shape. It does not implement assembly,
artifact emission, proof acceptance, or source-to-checker extraction.

## References

- [architecture 01](../../architecture/en/01.ir_layers.md) defines
  `ResolvedTypedAst` as the final source-shaped semantic AST.
- [architecture 05](../../architecture/en/05.overload_resolution.md) Step 6
  defines the overload-to-`ResolvedTypedAst` boundary.
- [typed_ast.md](./typed_ast.md) defines the source-shaped typed arena and
  partial typing model.
- [type_checker.md](./type_checker.md) defines normalized types, coercion
  candidates, type facts, and initial obligations.
- [cluster_trace.md](./cluster_trace.md) defines replayable cluster/reduction
  trace material.
- [overload_resolution.md](./overload_resolution.md) defines overload result,
  inserted-view, and failed-site preservation semantics.

## Responsibility

`ResolvedTypedAst` owns:

- the source-shaped resolved node arena;
- final expression/type metadata for each projected expression or overload
  site;
- final overload resolution records, including failed records;
- inserted `qua`/coercion view metadata that later phases must observe;
- final cluster/type facts that are visible at each projected expression;
- diagnostics and recovery metadata needed by LSP and artifacts;
- deterministic source maps back to `TypedAst` nodes and source ranges.

`ResolvedTypedAst` does not own:

- lowered logical clauses or kernel terms;
- VC-specific local proof contexts;
- ATP premises or proof search results;
- artifact serialization schemas or cache readers;
- source walking, resolver-shell parsing, or fabrication of missing checker
  payloads.

## Inputs

Task 28 assembly consumes explicit checker-owned outputs:

- `TypedAst` nodes, statuses, local contexts, and typed-site references;
- the optional complete source/binding-context handoff already owned and
  validated by `TypedAst`;
- final `TypeFactTable` / type-fact query output from phase 6;
- accepted cluster closure fact rows with their existing provenance ids;
- overload collection, template expansion, viability, and specificity graph
  outputs that provide site owners, source ranges, pre-filter and viable
  candidate tables, rejection/blocking reasons, and graph ids;
- selected overload results and inserted views from
  `OverloadSelectionOutput`, including inserted-view kind, reason, evidence,
  and path;
- checker-local diagnostics already produced by preceding phases;
- caller-supplied `ExpressionMetadataInput` rows that map stable source
  expression ids to `TypedSiteRef` owners and already-computed cluster fact
  references;
- optional `ResolvedNodeKindHint` rows for source-preserved, resolved-use, or
  degraded node roles that cannot yet be inferred from the checker tables.

Missing source-derived inputs are `external_dependency_gap` records. Missing
checker-owned precursor tables are task-28 assembly blockers, not permission to
scan raw syntax. Assembly must not inspect raw syntax or opaque resolver shells
to fill either kind of gap.

Expression metadata inputs are canonicalized by expression id before dense ids
are assigned. Duplicate expression ids or duplicate `TypedSiteRef` owners are
assembly errors because site-based lookup and resolved-node attachment must be
unambiguous. Sites without an `ExpressionMetadataInput` row simply have no
expression metadata entry in task 28; AST-wide extraction of all source
expression ids remains a deferred source-to-checker integration task.

Current source-derived runner note: the `mizar-test` type-elaboration runner
now supplies real `ExpressionMetadataInput` rows for the bounded reserve-only
bare-builtin declaration pass bridge. Reserve declaration nodes and
binding-specific type-expression nodes are source-preserved `ResolvedTypedAst`
nodes with final types when declaration checking succeeds. Same-module
attributed builtin and local-mode reserve heads are active fail slices only;
the active runner may use the same assembly helper to collect stable
diagnostic keys, but only diagnostic-free bare-builtin output is credited as
`ResolvedTypedAst` readiness. The active runner now also passes the successful
bare-builtin `ResolvedTypedAst` payload to `mizar-core`'s
`ResolvedTypedAstSummary::from_ast` and verifies summary-readiness. This does
not execute `mizar-core` lowering, publish artifacts, allocate public
diagnostics, or promote CoreIr/ControlFlowIr/VC/proof corpus rows.

## Data Shape

The public data layer should keep dense ids local to the assembled output:

```rust
struct ResolvedTypedAst {
    source_id: SourceId,
    module_id: ModuleId,
    source_context: Option<SourceBindingContextHandoff>,
    source_type: Option<SourceTypeApplicationHandoff>,
    source_attribute: Option<SourceAttributeHandoff>,
    nodes: ResolvedTypedArena,
    expr_metadata: ExpressionMetadataTable,
    collection_candidates: OverloadCandidateSummaryTable,
    expanded_candidates: OverloadCandidateSummaryTable,
    template_expansions: TemplateExpansionSummaryTable,
    viable_candidates: OverloadCandidateSummaryTable,
    viability_decisions: CandidateViabilitySummaryTable,
    specificity_graphs: ResolvedSpecificityGraphTable,
    resolved_overloads: OverloadResolutionTable,
    inserted_coercions: CoercionInsertionTable,
    cluster_facts: ClusterFactTable,
    diagnostics: ResolvedTypedDiagnosticTable,
    checked_formulas: CheckedFormulaTable,
    statement_semantics: StatementSemanticTable,
}
```

Task 248 permits no separately replaceable source-context assembler input.
Assembly clones `source_context` only from the supplied `TypedAst`, so the
final layer cannot diverge from the checker-owned source-item, declaration,
binding, or local-context links. Absence preserves legacy debug bytes; presence
adds deterministic nonempty handoff rendering.

Task 249 applies the same rule to `source_type`: assembly clones the immutable
handoff only from `TypedAst`, accepts no independent source-type input, and
therefore cannot diverge from the already authenticated flat application/
expression/argument tables. Absence preserves legacy debug bytes.

Task 250 applies the same clone-only rule to `source_attribute`. Assembly
accepts no independent attribute-chain input and copies the immutable handoff
only from `TypedAst`, so the final layer cannot diverge from the authenticated
chain, polarity, qualifier, group, actual, provenance, or Task-249 association
tables. Absence preserves legacy debug bytes.

### Resolved Nodes

```rust
struct ResolvedTypedNode {
    id: ResolvedTypedNodeId,
    typed_node: TypedNodeId,
    source_range: SourceRange,
    children: Vec<ResolvedTypedNodeId>,
    kind: ResolvedTypedNodeKind,
    final_type: Option<NormalizedTypeId>,
    metadata: Option<ExpressionMetadataId>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
    recovery: ResolvedNodeRecovery,
}

enum ResolvedTypedNodeKind {
    SourcePreserved { role: SourceNodeRole },
    ResolvedUse { symbol: SymbolId },
    FailedOverload { result: OverloadResolutionId },
    Degraded { reason: ResolvedNodeRecoveryReason },
}
```

The arena preserves source shape. A failed overload site remains a node with a
failed overload result id; it must not be rewritten into a successful
`ResolvedUse`.

### Expression Metadata

```rust
struct ExpressionMetadata {
    id: ExpressionMetadataId,
    expr: ExprId,
    typed_site: TypedSiteRef,
    source_range: SourceRange,
    final_type: Option<NormalizedTypeId>,
    visible_facts: Vec<TypeFactId>,
    cluster_facts: Vec<ClusterFactId>,
    overload: Option<OverloadResolutionId>,
    inserted_views: Vec<CoercionInsertionId>,
    local_context: Option<LocalTypeContextId>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}
```

Expression metadata is the stable lookup surface for hover,
`@show_resolution`, artifact summaries, and downstream elaboration. It records
ids produced by earlier phases; it does not recompute facts or overload
choices.

`ExprId` is the source-file identity for expression metadata. The
`ExpressionMetadataTable` must provide a deterministic `ExprId` to
`ExpressionMetadataId` lookup, and `ExpressionMetadataId` is only the dense row
id inside this `ResolvedTypedAst`. Task 28 tests should assert lookup by
`ExprId` and should not treat table insertion order as expression identity.

`final_type` is resolved by final semantic precedence. If the expression has a
successful overload result, assembly first uses `exposed_result.result` when it
is present, then the selected root candidate's result type when available. If
there is no successful overload result, assembly uses a handoff-available
`TypeEntryActual::Known` type from `TypedAst`. Open
`TypeEntryActual::CandidateSet` entries are not final types by themselves; if
they are not resolved through a successful overload result, `final_type` remains
`None` and the failed/open state remains visible through diagnostics and
overload metadata.

### Overload Candidate And Graph Summaries

`ResolvedTypedAst` copies the candidate and specificity graph summaries needed
for `@show_resolution`, diagnostics, artifacts, and downstream elaboration. It
does not require later consumers to retain the task-22 through task-25
precursor outputs.

Candidate ids are dense inside their owning predecessor table, so task 28 keeps
three explicit candidate namespaces. `collection_candidates` copies the task-22
collection table. `expanded_candidates` copies the task-23 template-expansion
candidate table used by viability decisions, including non-template candidates
and instantiated template candidates. `viable_candidates` copies the
viability/specificity candidate table used by specificity graphs, overload
selections, and inserted views. `TemplateExpansionSummary` is the explicit
bridge from collection `source_candidate` ids to optional expanded
`instantiated_candidate` ids. `CandidateViabilitySummary` is the explicit bridge
from expanded `source_candidate` ids to optional viable `output_candidate` ids.
All `OverloadResolutionRecord`, `ResolvedSpecificityGraph`, and
`CoercionInsertion` candidate references use the viable namespace.

```rust
struct OverloadCandidateSummary {
    candidate: OverloadCandidateId,
    site: OverloadSiteId,
    symbol: SymbolId,
    ordinary_root: SymbolId,
    declaration_kind: CandidateDeclarationKind,
    parameters: Vec<NormalizedTypeId>,
    result: Option<NormalizedTypeId>,
    origin: CandidateOrigin,
    template: Option<TemplateCandidatePayload>,
    coherence: Option<CoherenceStatus>,
    provenance: CandidateProvenance,
    status: OverloadCandidateStatus,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

struct TemplateExpansionSummary {
    id: TemplateExpansionId,
    source_candidate: OverloadCandidateId,
    site: OverloadSiteId,
    template: SymbolId,
    instantiation_key: TemplateInstantiationKey,
    substitutions: Vec<TemplateSubstitution>,
    instantiated_candidate: Option<OverloadCandidateId>,
    status: TemplateExpansionStatus,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

struct CandidateViabilitySummary {
    id: CandidateViabilityId,
    source_candidate: OverloadCandidateId,
    site: OverloadSiteId,
    output_candidate: Option<OverloadCandidateId>,
    status: CandidateViabilityStatus,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

struct ResolvedSpecificityComparison {
    id: SpecificityComparisonId,
    left: OverloadCandidateId,
    right: OverloadCandidateId,
    status: SpecificityComparisonOutcome,
    reasons: Vec<SpecificityReasonKey>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

struct ResolvedSpecificityGraph {
    graph: SpecificityGraphId,
    site: OverloadSiteId,
    nodes: Vec<SpecificityNode>,
    comparisons: Vec<ResolvedSpecificityComparison>,
    edges: Vec<SpecificityEdge>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}
```

These summaries preserve candidate status, template payload, coherence status,
declaration spans/import provenance from candidate provenance, template
substitutions/skipped-template status, viability rejection/blocking reasons for
failed/no-match sites, and stable comparison evidence from the graph. All
diagnostic references are translated to `ResolvedTypedDiagnosticId`s. The
summaries are copied metadata, not a second overload-resolution engine.

### Overload Resolution Table

```rust
struct OverloadResolutionRecord {
    id: OverloadResolutionId,
    site: OverloadSiteId,
    typed_site: TypedSiteRef,
    source_range: SourceRange,
    status: OverloadResolutionStatus,
    candidates: Vec<OverloadCandidateId>,
    specificity_graph: Option<SpecificityGraphId>,
    diagnostics: Vec<ResolvedTypedDiagnosticId>,
}

enum OverloadResolutionStatus {
    Resolved {
        root: OverloadCandidateId,
        active_refinements: Vec<OverloadCandidateId>,
        exposed_result: Option<ExposedResultPayload>,
        inserted_views: Vec<CoercionInsertionId>,
    },
    NoMatch { rejected: Vec<OverloadCandidateId> },
    Ambiguous { candidates: Vec<OverloadCandidateId> },
    IncompatibleRefinementJoin {
        root: OverloadCandidateId,
        refinements: Vec<OverloadCandidateId>,
        reason: RefinementJoinFailure,
    },
    Blocked { reason: OverloadBlockedReason },
}
```

Resolved records are a projection of task-26 selection output. Failed records
are first-class metadata and are not valid elaboration inputs.

### Coercion Insertion Table

```rust
struct CoercionInsertion {
    id: CoercionInsertionId,
    typed_site: TypedSiteRef,
    source_range: SourceRange,
    target: NormalizedTypeId,
    selected_candidate: Option<OverloadCandidateId>,
    source: CoercionInsertionSource,
    reason: InsertedViewReasonKey,
    evidence_facts: Vec<TypeFactId>,
    path: Option<QuaPathKey>,
}

enum CoercionInsertionSource {
    SourceQua,
    InsertedWidening,
}
```

The table records semantic view metadata. It is not a source edit list. Failed
or blocked overload sites must not contribute inserted coercions. `path` is the
single source of truth for source-`qua` and inserted inheritance paths; the
top-level `reason` preserves the task-26 inserted-view reason for both
source-written and inserted views. The `source` enum records whether the view
was source-written or inserted and must not duplicate path or reason payload.

### Cluster Fact Table

`ResolvedTypedAst.cluster_facts` reuses the checker-owned
`cluster_trace::ClusterFactTable` row and provenance schema. It must not define
a second same-named row shape. Expression metadata may reference existing
`ClusterFactId`s and may build deterministic per-expression indexes over those
ids, but the fact fingerprint, source/attribute/generated-type payload, and
`ClusterFactProvenance` stay owned by `cluster_trace`.

Assembly does not fire registrations, close clusters, replay reductions, or
translate cluster facts into new `TypeFactId`s. Any future artifact-oriented
projection of cluster facts or full `ResolutionTrace` material is a separate
schema task. Task 28 preserves `ClusterFactProvenance::TraceStep` ids as part
of the reused fact rows but does not validate or store the full trace step
payload.

## Failure And Recovery

Recoverable failures are represented explicitly:

- failed overload sites keep `OverloadResolutionStatus` records;
- partial or degraded typed nodes keep their original typed status and
  diagnostics;
- missing external payloads remain diagnostics or failed records;
- no failed site may be converted to a successful resolved use;
- downstream elaboration must skip or degrade failed nodes.

## Determinism

Assembly must be deterministic:

- ids are dense in canonical source order;
- overload records sort by site/source order;
- inserted coercions sort by typed site, target, source, and stable reason;
- the reused `cluster_trace::ClusterFactTable` preserves its own canonical
  ordering; per-expression cluster fact references/indexes sort by owning
  `TypedSiteRef`, `ClusterFactId`, and existing provenance;
- equivalent inputs produce byte-identical debug rendering.

## Public Enum Policy

Task 31 applies the frontend task-25 public-enum decision procedure to this
module. All public checker-owned enums in `resolved_typed_ast` are
forward-compatible API surfaces and must remain `#[non_exhaustive]`;
downstream consumers must keep wildcard or fallback arms. Checker-internal
matches may remain exhaustive over the currently represented variants when
implementing the specified behavior.

| enum | decision |
|---|---|
| `ResolvedNodeKindHintKind` | Forward-compatible; source-shaped node hints may grow with downstream presentation needs. |
| `ResolvedTypedNodeKind` | Forward-compatible; resolved node categories may grow with later source-shaped projections. |
| `ResolvedNodeRecovery` | Forward-compatible; node recovery states may grow with partial assembly policy. |
| `ResolvedNodeRecoveryReason` | Forward-compatible; recovery reasons may grow as source extraction and failed-site handling expand. |
| `OverloadResolutionStatus` | Forward-compatible; projected overload statuses may grow with phase-8 result handling. |
| `CoercionInsertionSource` | Forward-compatible; insertion sources may grow with accepted coercion/view forms. |
| `ResolvedTypedDiagnosticSource` | Forward-compatible; diagnostic sources may grow with additional projection stages. |
| `ResolvedTypedDiagnosticSeverity` | Forward-compatible; diagnostic severity policy may grow with IDE/artifact consumers. |
| `CandidateSummaryNamespace` | Forward-compatible; candidate-summary namespaces may grow with additional overload tables. |
| `ResolvedTypedAstError` | Forward-compatible; assembly validation errors may grow with new projection invariants. |
| `TheoremPolicyIntent` | Forward-compatible; declaration-policy intent may grow with explicitly supported theorem modifiers. |
| `TheoremJustificationIntent` | Forward-compatible; justification intent may grow with explicitly extracted written proof forms. |
| `CheckedProofStatus` | Forward-compatible; checker-owned proof processing states may grow without implying acceptance. |
| `CheckedProofNodeKind` | Forward-compatible; checked proof skeleton nodes may grow through checker Task 247 descendants. |
| `CheckedCitation` | Forward-compatible empty carrier; citation variants remain deferred to checker Task 247 descendants. |
| `CheckedProofLabel` | Forward-compatible empty carrier; proof-label variants remain deferred to checker Task 247 descendants. |

No exhaustive public enum exceptions are owned by this module.

## Planned Task 28 Tests

Task 28 should add Rust coverage for:

- source-shaped assembly from explicit `TypedAst` and checker outputs;
- metadata lookup by `TypedSiteRef` / expression id;
- final-type precedence for successful overload results over open candidate
  sets;
- separate collection, expanded, and viable candidate namespaces, including a
  rejected expanded candidate that shifts viable output ids;
- template expansion summaries for instantiated, rejected, and deferred
  templates;
- resolved overload projection, including active refinements and inserted
  views;
- failed overload site preservation for `NoMatch`, `Ambiguous`, incompatible
  refinement join, and blocked statuses;
- no inserted coercion records for failed overload sites;
- deterministic debug rendering across equivalent input orderings;
- cluster fact id references and preservation of existing cluster-trace
  provenance.

## Deferred And External Gaps

The following remain deferred after task 28:

- AST-wide source-to-checker extraction of task-26 selection payloads and
  source expression metadata;
- artifact emission/reuse and stable artifact schemas;
- full `ResolutionTrace` artifact projection/validation;
- public diagnostic-code allocation;
- active `.miz` semantic fixtures for final overload and cluster projection.

These gaps do not permit fabricated success records, raw syntax scans, or
artifact-like side outputs in task 28.

## Task 266 Exact Statement-Semantic Projection

Task 266 adds an optional, syntax-free predecessor bundle to
`ResolvedTypedAstInputs`:

```rust
struct StatementSemanticInputs<'a> {
    owner: &'a CheckedStatementOwner,
    binding_env: &'a BindingEnv,
    term_formula: &'a TermFormulaInferenceOutput,
    rows: Vec<StatementSemanticInput>,
}

struct StatementSemanticInput {
    owner: SymbolId,
    owner_node: TypedNodeId,
    formula: CheckedFormulaId,
    formula_node: TypedNodeId,
}
```

When the bundle is absent, existing assembly produces empty checked-formula and
statement-semantic tables and retains byte-stable debug output. When present
for Task 180, it contains exactly one row in source order. Assembly copies the
existing checked formula table and emits one dense `StatementSemantic`
preserving resolver theorem symbol, theorem typed-node identity, owner range
and `SemanticOrigin`, the existing checked formula identity/site, and a
separate formula typed-node identity for the compact final tree.

The accepted typed tree is exactly module root -> theorem owner -> formula.
Every node is normal and successfully typed; the owner range equals the
validated theorem range, the formula range/recovery equals the checked formula,
the root contains the owner, and the formula is strictly inside the owner.
Source and module identities agree across `TypedAst`, `BindingEnv`, inference
output, and checked owner. The inference output contains only one normal
`Checked` `FormulaKind::Contradiction`; terms, type entries, normalized types,
candidates, facts, diagnostics, asserted type, expected constraints, and
deferred reasons are forbidden in this exact slice.

Assembly fails closed on an absent row in a supplied bundle, non-singleton,
duplicate-owner, duplicate-formula, reordered, unknown, recovered, deferred,
cross-source/module, tree, range, provenance, owner, or formula mismatch.
Validation of the real resolver theorem owner remains in `type_checker`; this
module does not scan `SymbolEnv` or raw syntax. The projection assigns no truth
value, publishes no fact, accepts no theorem, and adds no proof, terminal-goal,
CoreIr, ControlFlowIr, or VC semantics.

## Task 267 Omitted-Justification Proof-Handoff Contract

Task 267 fixes the target contract implemented by Task 268. It does not change
the current Rust surface by itself. `mizar-test` alone classifies the exact
source syntax: one unrecovered, unannotated ordinary theorem with one
contradiction formula child and no justification node. It converts those
checked syntactic facts into explicit `Unmodified` and `Omitted` intent values.
Neither checker assembly nor core lowering may infer either intent from a
missing row, an absent optional field, or raw syntax.

Task 268 adds one syntax-free input row with this exact shape:

```rust
struct StatementProofIntentInput {
    id: StatementProofIntentId,
    source_order: usize,
    statement: StatementSemanticId,
    source_id: SourceId,
    module_id: ModuleId,
    owner: SymbolId,
    owner_node: TypedNodeId,
    owner_range: SourceRange,
    owner_origin: SemanticOrigin,
    owner_visibility: Visibility,
    owner_export_status: ExportStatus,
    formula: CheckedFormulaId,
    formula_site: TypedSiteRef,
    formula_node: TypedNodeId,
    formula_range: SourceRange,
    recovery: NodeRecoveryState,
    policy: TheoremPolicyIntent,
    justification: TheoremJustificationIntent,
}

enum TheoremPolicyIntent { Unmodified }
enum TheoremJustificationIntent { Omitted }
```

All six new enums in this section (`TheoremPolicyIntent`,
`TheoremJustificationIntent`, `CheckedProofStatus`, `CheckedProofNodeKind`, and
the empty carriers `CheckedCitation` and `CheckedProofLabel`) are public and
`#[non_exhaustive]` under the
module policy above. Task 268 adds their rows to the current-source policy
table in the same implementation commit; Task 267 does not put unimplemented
enums into that lint-guarded table. The row is supplied through a separate
optional top-level bundle, not inferred from `StatementSemanticInputs`:

```rust
struct StatementProofInputs<'a> {
    pub owner: &'a CheckedStatementOwner,
    pub rows: Vec<StatementProofIntentInput>,
}

struct ResolvedTypedAstInputs<'a> {
    // existing fields unchanged
    statement_semantics: Option<StatementSemanticInputs<'a>>,
    statement_proofs: Option<StatementProofInputs<'a>>,
}
```

Task 268 updates existing struct-literal callers to pass `None`. A supplied
proof bundle requires the statement bundle in the same assembly call and uses
its authenticated `CheckedStatementOwner`; a supplied exact Task-180 statement
bundle requires the proof bundle. The two bundles are validated together in
local state before either table family is published.

`StatementProofInputs` and `StatementProofIntentInput` are public input
structs with public fields. `StatementProofIntentId` is a public dense id with
`new(index)` and `index()` so `mizar-test` can construct the syntax-free row.
The Task-268 extension adds public `visibility()` and `export_status()` getters
to `CheckedStatementOwner`; the stored fields remain private.

For the exact Task-180 source, `id`, `source_order`, and `statement` are all
dense index zero. `formula_site` is the existing Task-266
`TypedSiteRef::Node`; `formula_node` is the distinct compact final-tree node
and must not replace or reconstruct that real site. Visibility/export are the
explicit resolver facts `Public`/`Exported`, recovery is `Normal`, and policy
and justification are the two variants above. Source, module, owner, owner
node/range/origin, formula id/site/node/range, and recovery must equal the
Task-266 statement and checked-formula data and the authenticated owner bundle.
Task 268 extends `CheckedStatementOwner` to preserve the resolver entry's
visibility/export status and validates them independently before copying the
proof-intent row. `Exported` describes resolver
name visibility only. It is not proof acceptance.

The accepted output is three all-or-none singleton dense tables:

```rust
struct CheckedProof {
    id: CheckedProofId,
    source_order: usize,
    statement: StatementSemanticId,
    owner: SymbolId,
    owner_node: TypedNodeId,
    owner_visibility: Visibility,
    owner_export_status: ExportStatus,
    proposition: CheckedFormulaId,
    policy: TheoremPolicyIntent,
    justification: TheoremJustificationIntent,
    root: CheckedProofNodeId,
    status: CheckedProofStatus,
    source_range: SourceRange,
    owner_origin: SemanticOrigin,
}

enum CheckedProofStatus { PendingAutomaticProof }

struct CheckedProofNode {
    id: CheckedProofNodeId,
    proof: CheckedProofId,
    kind: CheckedProofNodeKind,
    source_range: SourceRange,
    recovery: NodeRecoveryState,
}

enum CheckedProofNodeKind {
    TerminalGoal(CheckedTerminalGoalId),
}

struct CheckedTerminalGoal {
    id: CheckedTerminalGoalId,
    proof: CheckedProofId,
    node: CheckedProofNodeId,
    statement: StatementSemanticId,
    owner: SymbolId,
    formula: CheckedFormulaId,
    formula_site: TypedSiteRef,
    formula_node: TypedNodeId,
    source_range: SourceRange,
    recovery: NodeRecoveryState,
    citations: Vec<CheckedCitation>,
    active_context: Vec<CheckedFormulaId>,
    local_path: String,
    label: Option<CheckedProofLabel>,
}

#[non_exhaustive]
pub enum CheckedCitation {}

#[non_exhaustive]
pub enum CheckedProofLabel {}
```

`CheckedCitation` and `CheckedProofLabel` have no Task-267 variants, so the
exact values can only be an empty vector and `None`. The empty public enums are
lint-clean under the workspace `deny(warnings)` policy and cannot be
constructed by Task 268. These named forward-compatible carriers do not
authorize symbol, local-label, or generated-origin citation semantics. Checker
Task 247 owns their broader variants.

`ResolvedTypedAst` owns private fields of the public `CheckedProofTable`,
`CheckedProofNodeTable`, and `CheckedTerminalGoalTable` types and exposes `checked_proofs()`,
`checked_proof_nodes()`, and `checked_terminal_goals()` getters. Their public
dense ids expose `new(index)` and `index()` like the existing statement id.
Each table exposes `get`, source-order `iter`, `len`, and `is_empty`; mutation
remains assembly-private. The three row structs have public read-only fields,
matching `StatementSemantic`. Empty legacy assembly returns three empty tables.

When the three tables are nonempty, canonical `debug_text()` renders all three
in deterministic source/id order, including every row field and cross-reference.
When all three are empty, it emits no new proof section and remains byte-for-byte
identical to the Task-266 legacy projection. Task 268 owns both the exact
nonempty rendering assertion and the empty-output byte-stability regression.

All ids and `source_order` are zero. The proof source is the owner range; the
single root node and terminal goal use the formula range. The root is directly
`TerminalGoal(CheckedTerminalGoalId(0))`: there is no `CurrentGoal`,
`Sequence`, implicit `Thesis`, intermediate step, or synthesized child. The
terminal row points back to proof/node/statement zero, the same owner and
checked contradiction, the real formula site, and the separate compact node.
It has normal recovery, empty citations and active context, the exact nonempty
local path `proof/0`, and no label.

`Unmodified` is a declaration-policy axis. `PendingAutomaticProof` is a
separate processing axis meaning that the automatic proof attempt has not run.
It is not `Open`, `Assumed`, `Conditional`, `Error`, a published fact, proof
evidence, theorem acceptance, or discharge. Missing statement/proof
asymmetry, non-singleton input, duplicate or non-dense ids, a nonzero source
order, a nonzero statement reference, wrong root or cross-reference, a role
site, recovery, or any identity/range/provenance/status mismatch fails before
publication. In this singleton contract, “reordered” means that the explicit
source-order/id/reference chain is not exactly `0 -> 0 -> 0`. Assembly is
transactional and emits all three tables or none; it never substitutes an
error/partial proof row. Existing callers that supply neither the Task-266
statement bundle nor this proof-intent bundle retain legacy empty behavior.
After Task 268, an exact Task-180 statement bundle without the proof-intent
bundle is an error.

Task 268 owns only this producer and its corruption tests. It may not add
broader theorem/proof forms, truth or facts, proof search, acceptance,
CoreIr/ControlFlowIr/VC generation, fixture or expectation changes, or Step
6/7 behavior. Core Task 31 consumes these explicit tables; it may not recover
their intent by scanning source.

## Task 268 Implementation Completion

Task 268 implements the accepted contract above for only the exact Task-180
source. `ResolvedTypedAst::assemble` requires the statement and proof bundles
together, validates the explicit `Unmodified`/`Omitted` row against the
authenticated owner and checked contradiction, constructs all three singleton
tables in local state, and privately postvalidates their cardinalities, dense
ids, root/status, cross-references, empty carriers, and `proof/0` metadata
before publication. Every mismatch returns an error without a partial table.
Constructible proof/node/goal invariants are exercised through cloned-table
corruption. The single-variant policy/justification/status and empty citation/
label states have no safe invalid value; their private predicate is tested
independently, and status rejection additionally uses the same validator core
with a false status-match seam.

The exact nonempty debug projection renders every field in the proof, node,
and terminal-goal tables deterministically. A captured Task-266 empty-bundle
string guards byte-identical legacy rendering. The implementation adds no raw
syntax dependency to the checker, publishes no fact, accepts or discharges no
theorem, and creates no CoreIr, ControlFlowIr, or VC payload. Core Task 31 is
now the next dependency-authorized consumer; Steps 6/7 remain deferred.

## Task 251 Final-Handoff Addendum

`ResolvedTypedAst` clone-preserves the optional checker-owned
`SourceEvidenceHandoff` installed on `TypedAst` and exposes a borrowed
`source_evidence()` getter. Final assembly does not rebuild, reinterpret, or
accept evidence, and the legacy projection remains unchanged when the handoff
is absent.

## Task 256 Final-Handoff Addendum

`ResolvedTypedAst` revalidates the exact Task-252 and conditional
Task-253/254/255 fingerprints, resolver provenance, request associations,
nearest-family target partition, and arena sites, then clone-preserves the
optional checker-owned `SourceAtomicFormulaHandoff`. Its borrowed
`source_atomic_formula()` getter exposes the immutable handoff. Final assembly
does not rebuild or retarget dense IDs, select a predicate candidate, answer
an expected-input request, decide an assertion or formula, publish a fact, or
accept a theorem; the legacy projection is unchanged when the handoff is
absent.

## Task 252 Final-Handoff Addendum

`ResolvedTypedAst` clone-preserves the optional checker-owned
`SourcePrimaryTermHandoff` installed on `TypedAst` and exposes a borrowed
`source_term()` getter. Final assembly does not rebuild or reinterpret source
terms, choose numeric types, or create semantic results, and the legacy
projection remains unchanged when the handoff is absent.

## Task 253 Final-Handoff Addendum

`ResolvedTypedAst` revalidates the exact Task-252 fingerprint/primary-edge
association and clone-preserves the optional checker-owned
`SourceFunctorApplicationHandoff` installed on `TypedAst`. Its borrowed
`source_application()` getter exposes the immutable handoff. Final assembly
does not rebuild or retarget dense IDs, collect/select candidates, resolve a
signature/result type, or create semantic results; the legacy projection is
unchanged when the handoff is absent.

## Task 254 Final-Handoff Addendum

`ResolvedTypedAst` revalidates the exact Task-252 and conditional Task-253
fingerprints, root-only cross-family targets, and arena-site association, then
clone-preserves the optional checker-owned `SourceStructureHandoff` installed
on `TypedAst`. Its borrowed `source_structure()` getter exposes the immutable
handoff. Final assembly does not rebuild or retarget dense IDs, resolve
members or inheritance views, decide constructor coverage, compute selector
or update results, or create semantic results; the legacy projection is
unchanged when the handoff is absent.

## Task 255 Final-Handoff Addendum

`ResolvedTypedAst` revalidates the exact Task-252 and conditional Task-253/254
fingerprints, nearest-family cross-target partition, canonical spelling, and
arena-site associations, then clone-preserves the optional checker-owned
`SourceSetTermHandoff` installed on `TypedAst`. Its borrowed
`source_set_term()` getter exposes the immutable handoff. Final assembly does
not rebuild or retarget dense IDs, bind comprehension generators, resolve
conditions, decide sethood/nonemptiness/widening, compute result types, or
create semantic results; the legacy projection is unchanged when the handoff
is absent.

## Task 257A Final-Handoff Addendum

`ResolvedTypedAst::assemble` revalidates and clone-preserves the optional
checker-owned `SourceCompositeFormulaHandoff`; it never rebuilds the tree,
binder, or context from raw source. Its borrowed
`source_composite_formula()` getter exposes the same immutable transaction.
Assembly rejects invalid source-context coexistence or dependency drift and
does not answer any of the six unresolved requests.

## Task 257B1 Final-Handoff Addendum

`ResolvedTypedAst::assemble` revalidates the Task-252, Task-256, and second
Task-257 fingerprints, then clone-preserves the optional
`SourceFormulaCompositionHandoff` without rebuilding or renumbering its
atomic-edge or bound-use rows. The borrowed
`source_formula_composition()` getter exposes the immutable handoff. Absence
preserves legacy bytes; presence adds no formula result, fact, truth, theorem
acceptance, proof, or downstream IR.

Task 257B2 reuses this boundary: final assembly revalidates and clone-preserves
the exact third composite profile and `8/0` composition fingerprints. The
resolved result still contains no checked formula, statement semantics,
accepted theorem, proof, or IR output.

## Task 257B3 Frozen Final-Handoff Addendum

Final assembly now revalidates the Task-48 reserve provenance, Task-252/256
fingerprints, fourth composite profile, nested binding environment, and
`3/6` owning-edge/use associations before clone-preserving the handoffs.
Absence of a Task-248 source-context handoff is part of the exact profile.
No checked formula, witness, restriction result, closure, fact, theorem
acceptance, proof, or IR output is assembled.

The B3 orphan rejection, exact revalidation, and deterministic clone are now
covered by executable checker and runner tests.

## Task 257C1 Frozen Final-Handoff Addendum

`ResolvedTypedAst::assemble` will rerun the extended Task-256 nine-table
validation and clone-preserve the exact chain handoff. It neither reconstructs
segments nor selects the duplicated imported candidate. Orphan/shared-edge,
polarity-token, provenance, request, fingerprint, and legacy-profile
corruption fail closed. No checked formula, conjunction/negation result,
predicate winner, theorem acceptance, proof, or IR output is assembled.

The implemented Task 257C1 runner confirms byte-stable typed/resolved debug
output and equality of both Task-252 and Task-256 handoffs. Resolution remains
clone-only and adds no semantic selection.

## Task 255C1 Frozen Resolution Addendum

Final assembly will revalidate and clone-preserve the seventh Task-255 table,
colon and direct condition-wrapper arena anchors, authenticated condition
range, and unchanged Task-252/253 fingerprints. It does not construct a
Task-256 inner equality or Task-257 composition, retarget a condition
operand, or rebuild any dense ID.

## Task 255C1 Resolution Result

Final assembly revalidates and clone-preserves the exact Task-252/253/255
objects, including the recursive condition boundary. The resolved object has
no expression metadata, fact, diagnostic, Task-256 condition formula, or
Task-257 composition from this task.

## Task 257C2 Frozen Resolution Addendum

Final assembly will clone and revalidate the optional
`SourceConditionFormulaCompositionHandoff` only after the exact
Task-252/253/255/256 objects. It compares all four fingerprints and the
condition-to-atomic relation without rebuilding IDs or formula rows. Missing,
stale, substituted, or reordered dependencies fail final assembly through
the dedicated `ResolvedTypedAstError::InvalidSourceConditionFormulaComposition`
variant. The exact accessor is:

```rust
pub const fn source_condition_formula_composition(
    &self,
) -> Option<&SourceConditionFormulaCompositionHandoff>;
```

At the frozen pre-Task-256C1 baseline, this projection was gated until the
separate lower task made the authenticated condition containment valid in
both installation orders. Task 256C1 now passes both orders; the projection
is now implemented, revalidates all four dependency fingerprints and the
sole association, and clone-preserves the immutable handoff. Final assembly
does not compensate for or weaken that lower validation.

No checked formula, equality truth, fact, diagnostic, definition acceptance,
proof, or IR output is derived.

## Task 257C3 Frozen Final Projection

The later final projection revalidates and clone-preserves the optional
predicate-chain composition handoff after typed ownership succeeds. It adds
the matching accessor, deterministic debug placement, and
`InvalidSourcePredicateChainComposition`, but no expression metadata,
diagnostic, checked formula, truth, fact, or downstream semantic result. This
documentation prerequisite changes no resolved source or output.

```rust
pub const fn source_predicate_chain_composition(
    &self,
) -> Option<&SourcePredicateChainCompositionHandoff>;
```

The cloned C3 debug chunk occupies the final mutually exclusive formula-owner
slot after Task-252 source-term, Task-256 source-atomic-formula, and the
A/B/C2 slots, immediately before the resolved node/table section.

## Task 257C3 Implementation Result

Final assembly now revalidates the exact optional handoff against the cloned
Task-252/256 dependencies and arena, preserves it across `Clone`, and emits
its deterministic debug chunk in the frozen slot. Removing a required lower
handoff in a test-only state fails with
`InvalidSourcePredicateChainComposition`. Expression metadata, diagnostics,
truth, facts, proof, and downstream IR remain empty.

## Task 258A Frozen Final Projection

Final assembly will revalidate and clone-preserve the optional
`SourceStatementHandoff` after the exact Task-252 and Task-256 lower
handoffs. It adds only:

```rust
pub const fn source_statement(&self) -> Option<&SourceStatementHandoff>;
```

Missing, stale, substituted, or corrupt lower/owner/statement/context/fact
relations fail through
`ResolvedTypedAstError::InvalidSourceStatement`. Assembly does not rebuild
IDs, resolve a label, check the equality, or create any existing
`StatementSemanticInput` or `StatementProofIntentInput`. The Task-266/268
standalone-contradiction checked tables remain disjoint and unchanged.
Expression metadata, checked formulas, statement semantics, proofs, terminal
goals, facts, diagnostics, and downstream IR remain empty. This prerequisite
changes no resolved source or output.

The revalidation includes the handoff-owned exact `BindingEnv` and its
fingerprint. A test-injected typed input that contains both Task-248
`source_context` and Task-258A `source_statement` deterministically fails
with `InvalidSourceStatement`; no final output is published, the original
typed debug remains byte-identical, and either valid single-owner input can
be replayed.

### Task 258A Implementation Result

Final assembly clone-preserves the validated handoff after rechecking the
owned binding environment, Task-252/256 fingerprints, and arena. It rejects
nonempty typed semantic tables, cluster facts, overload-stage outputs,
expression inputs, statement semantics/proofs, and diagnostics before
output construction. Empty node hints and the exact complete
`source.statement.transport` source-preserved hint set are admitted because
they preserve syntax nodes only; every other nonempty hint set fails through
`InvalidSourceStatement`.

## Task 258B1 Frozen Final Statement Projection

`ResolvedTypedAst` adds the optional
`SourceStatementReferenceHandoff` clone and exact accessor:

```rust
pub const fn source_statement_references(
    &self,
) -> Option<&SourceStatementReferenceHandoff>;
```

Assembly admits only the complete B1 base/reference pair after typed
ownership. It revalidates source/module, the `3/1/0` environment,
Task-252/256 fingerprints, shared arena and statement topology, the stored
77-node/root-76 `ResolvedAst` with sole resolved `Label(0)` node 68,
projection/reference/result replay, both handoff fingerprints, and every
row. Missing one half, stale or substituted provenance, Task-248/257/258A
coexistence, or any nonempty semantic-stage input fails as
`ResolvedTypedAstError::InvalidSourceStatement` before publication.

The final debug places the reference chunk immediately after the base
statement chunk and before resolved nodes/tables. Clone preserves both exact
handoffs. Task-258A output remains byte-identical because its reference field
is absent. No name resolution is inferred at final assembly, and no fact,
checked formula, statement semantic, proof, goal, diagnostic, or downstream
IR/VC output is created. This prerequisite changes no resolved source.

### Task 258B1 Implementation Status

Final assembly clones and revalidates the B1 base/reference pair together and
rejects an orphan, missing, stale, or cross-profile half as
`InvalidSourceStatement`. The reference debug chunk follows the base chunk
before resolved nodes. All statement semantic, checked-formula, proof, goal,
diagnostic, and downstream tables remain empty.

### Task 258B2 Frozen Final Ownership

Final assembly admits the Task-258B2 base-only handoff after typed ownership.
It revalidates the exact 113-byte source and the frozen Task-48 `2/1/0`,
Task-252 `6/6/0`, Task-256 `3/0/0/0/0/0/0/6/6`, and statement
`1/3/3/3/3` profiles. The retained resolver provenance must contain the one
public/exported theorem at origin path `[2, 1]`, contribution 0, and no
proof-step label, citation, or reference key.

Any source/profile substitution, reference half, competing payload owner, or
nonempty semantic-stage input fails before publication. Clone and debug
preserve the base handoff exactly. Final assembly infers no new resolution
and publishes no accepted premise, fact, checked formula, statement
semantic, proof, goal, diagnostic, theorem result, or downstream IR/VC
output. This prerequisite changes no resolved source or test.

### Task 258B2 Implementation Closure

Final assembly revalidates and clone-preserves the exact base-only B2
handoff with no reference association. Any cluster fact, statement-proof
input, foreign source family, checked formula, statement semantic, proof,
goal, diagnostic, or other semantic output remains rejected; the successful
result keeps every such table empty.

### Task 258B3 Frozen Final Ownership

Final assembly admits the Task-258B3 statement/witness pair only after typed
ownership. It revalidates source/module identity, the base/lower
fingerprints, exact `1/2/2/2/2` + one-row profiles, shared 49-node arena,
term-2 exclusion from atomic edges, and combined source order `[0,1,2]`.
The result exposes equal `source_statement()` and
`source_statement_witnesses()` clones; no reference handoff is present.

An orphan witness, standalone B3 base, stale fingerprint, B1/B2 hybrid,
foreign source owner, cluster fact, statement-proof input, or nonempty
semantic output is `InvalidSourceStatement`. Final assembly infers no
resolver or witness semantics and leaves checked formulas, facts, statement
semantics, proofs, goals, and diagnostics empty. This prerequisite changes
no resolved source or test.

### Task 258B3 Final Ownership Result

Final assembly clone-preserves and revalidates the exact base/witness pair.
Standalone base, orphan witness, stale fingerprint, reference hybrid, and
nonempty semantic coexistence fail as `InvalidSourceStatement`. Successful
assembly keeps every semantic, proof, goal, and diagnostic table empty.

## Task 258B3N Planned Final Ownership

Final assembly will accept only an authenticated B3N base plus witness/name
bundle already owned by `TypedAst`. B3/B3N hybrids, standalone halves,
reference hybrids, stale name links/fingerprints, and any semantic/proof/goal
coexistence fail as `InvalidSourceStatement`. Successful B3N assembly remains
transport-only and clone-preserves the dense name table.

## Task 258B3N Implementation Result

Final assembly revalidates and clone-preserves the B3N base plus
witness/name bundle. Orphan or standalone halves, stale statement/primary
fingerprints, reference hybrids, nonempty typed tables, expression/cluster
metadata, proof input, and statement semantics fail as
`InvalidSourceStatement`; every semantic, proof, goal, and diagnostic table
remains empty on success.

## Task 258B3M1 Planned Final Ownership

Final assembly may clone-preserve only an already authenticated B3M1
base/witness pair with two witness rows, one name row, exact fingerprints,
and the 56-node arena. It revalidates both rows, their shared source ordinal,
dense within-`take` order, name links, and subtree ownership. B3/B3N/B3M1
hybrids, orphan halves, reference hybrids, stale dependencies, and nonempty
semantic/proof/goal tables fail as `InvalidSourceStatement`. No semantics
are inferred.

## Task 258B3M1 Implementation Result

Final assembly clone-preserves the authenticated B3M1 pair and revalidates
the full lower/base/witness/name dependency graph. Orphan, stale,
reference-hybrid, B3/B3N/B3M1 cross-family, nonempty overload pipeline,
semantic, proof, and goal inputs all fail as `InvalidSourceStatement`.
Successful output keeps every semantic/proof/goal table empty.

## Task 258B3M2A Planned Final Ownership

Final assembly may clone-preserve only an already authenticated B3M2A
base/witness pair with one unnamed primary-numeral witness, no names, exact
Task-252 numeric request, fingerprints, and the complete 49-node arena. It
revalidates the row, source partition `[0,1,2]`, subtree exclusions, and
all lower dependencies. Standalone halves, B3/B3N/B3M1/B3M2A hybrids,
reference or numeric-request corruption, stale dependencies, and nonempty
semantic/proof/goal tables fail as `InvalidSourceStatement`. Successful
output infers no type, existential match, substitution, goal, or proof
effect.

## Task 258B3M2A Implementation Result

Final assembly now clone-preserves the authenticated B3M2A base/witness pair
only. It revalidates the exact one unnamed numeral witness, dependency
fingerprints, 49-node arena, and lower tables; standalone, hybrid, stale,
reference/numeric-request-corrupt, and semantic-coexisting states remain
`InvalidSourceStatement`. The successful final handoff has empty expression,
candidate, coercion, cluster, diagnostic, statement-semantic, proof, and goal
tables.

## Task 258B3M2B1 Frozen Final Ownership

Final assembly must clone-preserve only the authenticated 53-node
B3M2B1 base/witness pair with five roots, six primary rows, and one
parenthesized witness/no names. Standalone, hybrid, stale, parent/child,
reference-map, B3M2A, Tasks-253–255, or semantic-coexisting states remain
`InvalidSourceStatement`. It revalidates source partition `[0,1,2]`,
outer-wrapper witness targeting, the inner reference/parent edge, complete
witness-subtree exclusion, fingerprints, all lower dependencies, and the
53-node arena. Successful final output keeps expression, candidate,
coercion, cluster, diagnostic, statement-semantic, proof, and goal tables
empty and infers no type, existential match, substitution, goal, or proof
effect. No final public field or accessor changes.

## Task 258B3M2B1 Implementation Result

Final assembly now clone-preserves only the authenticated B3M2B1
base/witness pair. It revalidates the exact 53-node arena,
five-root/six-primary mapping, wrapper/child edge, one unnamed outer-term
witness, complete subtree exclusions, dependency fingerprints, and all
lower tables. Standalone, hybrid, stale, parent/reference-corrupt, and
semantic-coexisting states remain `InvalidSourceStatement`; successful
final semantic, proof, and goal tables remain empty.

## Task 258B3M2B2A Frozen Final Ownership

Future final assembly clone-preserves only the authenticated B3M2B2A
base/witness pair and revalidates the complete 57-node arena,
five-root/seven-primary mapping, two wrapper links, one unnamed outer
witness, subtree exclusions, fingerprints, and every lower table.
Standalone, hybrid, stale, parent/reference-corrupt, family-coexisting, or
semantic-coexisting states remain `InvalidSourceStatement`; success keeps
all semantic, proof, and goal tables empty. No final public API changes in
this documentation prerequisite.

## Task 258B3M2B2A Implementation Result

Final assembly now clone-preserves the authenticated paired base/witness
handoff only after revalidating the 57-node arena, Task-252 parent/reference
chain, Task-256 subtree exclusion, fingerprints, and source order. Every
standalone, hybrid, stale, corrupt, reversed, family-coexisting, or
semantic-coexisting state remains `InvalidSourceStatement`; semantic,
proof, and goal tables stay empty. No public final-AST API changed.

## Task 258B3M2B2B1A Final Bundle Revalidation Result

Final assembly clone-preserves the exact B1A application/statement/witness
bundle only after independently repeating its complete typed-stage
authentication. It revalidates the 63-node arena, imported
`parser.type_fixtures::++` application and resolver provenance, Task-252
arguments and numeric requests, Task-256 equality-only exclusion, base and
witness profiles, both lower fingerprints, and the optional B1A application
fingerprint.

The pre-existing standalone Task-253 application remains a valid final
bundle. A B1A application plus only a statement or witness, an orphan
statement/witness pair, an application-free B1A hybrid, a stale primary or
application fingerprint, substituted provenance, partial/reversed B1A family
installation, or semantic coexistence is rejected as
`InvalidSourceStatement`; cloning never repairs an invalid state. A successful
clone retains empty expression-semantic, candidate, coercion, cluster,
diagnostic, statement-semantic, proof, and goal outputs and infers no type,
proof step, substitution, or goal effect.

## Task 258B3M2B2B1B1 Frozen Final Bundle Revalidation

Final assembly must mirror the typed-stage enumeration: the existing
application/statement/witness bundle is valid only as exact B1A or exact
B1B1. B1B1 revalidates the 67-node arena, local theorem contribution and
label, wrapped Task-253 `1/1/1/2/2` application provenance and containment,
Task-252 `6/4/2`, Task-256 equality-only exclusion, base
`1/2/2/2/2`, one unnamed `Application(0)` witness/no names, and all
fingerprints. B1A remains the independent 63-node unwrapped profile with
unchanged bytes.

A partial, orphaned, stale, substituted, B1A/B1B1-hybrid, reversed,
family-coexisting, or semantic-coexisting state is
`InvalidSourceStatement`; cloning cannot repair it. A successful B1B1 clone
preserves all three handoffs byte-for-byte and retains empty type-semantic,
formula-semantic, proof, goal, overload, coercion, obligation, cluster, and
diagnostic outputs. No public final-AST API or semantic meaning is added.

## Task 258B3M2B2B1B1 Final Revalidation Result

Final assembly now accepts the exact B1B1 bundle as a separate private profile
and rejects every frozen partial/hybrid/substitution case. Clone revalidation
preserves the three handoffs byte-for-byte and all deferred upper tables
remain empty. `resolved_typed_ast.rs` is 7,225 lines; no public final-AST or
semantic/proof/goal API changed.

## Task 258B3M2B2B2A Frozen Final-AST Contract

ResolvedTypedAst gains no public accessor. Future assembly must require the
exact coexisting source-structure, source-statement, and structure-target
witness handoffs, revalidate the statement/primary/structure fingerprints
and every lower installation, revalidate equality-only Task 256 with
`Some(&structure)` and no direct structure fingerprint, and clone all three
byte-for-byte. The current blanket structure-plus-statement rejection is
relaxed only for this exact B2A triple. Missing, orphan, partial, stale,
application/structure hybrid, reverse, or repeated bundles reject atomically.
All semantic/proof/goal/overload tables remain empty.

## Task 258B3M2B2B2A Final Revalidation Result

Final assembly now admits only the exact coexisting structure, statement,
and structure-target witness bundle. It revalidates every lower
installation and fingerprint, including equality-only Task 256 against
`Some(&structure)`, and clone-preserves all three handoffs byte-for-byte.
Missing, partial, stale, repeated, reversed, or application/structure hybrid
states remain `InvalidSourceStatement`.

`resolved_typed_ast.rs` is 7,241 lines. No public final accessor, active
route, semantic/proof/goal owner, or coverage credit was added, and every
upper semantic table remains empty.

## Task 258B3M2B2B2B Frozen Final-AST Sibling

`ResolvedTypedAst` gains no accessor. Final assembly may coexist with a
source-structure statement only through one of the two exact authenticated
siblings: B2A is the 76-node constructor-witness profile targeting
`Structure(0)`, while B2B is the 79-node selector-witness profile targeting
`Structure(0)`, whose selector base is `Structure(1)`. Both have
`application = None` and `structure = Some`, so the full source, arena,
ownership, lower-table, target, and fingerprint profile—not that option
shape—selects the sibling.

For B2B, final assembly revalidates Task 252, Task 254, equality-only Task
256, Task 258 base rows, the witness edge, and all statement/primary/structure
fingerprints before cloning the authenticated bundle byte-for-byte. Task 256
owns formula application nodes `51/70`; enclosing `FormulaExpression` nodes
`52/71` remain unowned containers. B2A/B2B hybrids, swapped targets,
cross-profile fingerprints, partial or repeated bundles, and any application
coexistence reject atomically. Semantic, proof, goal, overload, and theorem
acceptance tables remain empty.

## Task 258B3M2B2B2B Final Revalidation Result

Final assembly now accepts B2B only as the exact 79-node sibling of B2A. It
revalidates the Task-48/252/254/256 and Task-258 base profiles, the
structure fingerprint, selector target `Structure(0)`, selector base
`Structure(1)`, ownership at `51/70`, and unowned containers `52/71`
before clone-preserving the three handoffs byte-for-byte.

B2A/B2B hybrids, generic structure-plus-statement bundles, stale
fingerprints, swapped targets, partial/repeated installation, and
application coexistence remain `InvalidSourceStatement`. All semantic,
proof, goal, overload, and theorem-acceptance outputs remain empty.
`resolved_typed_ast.rs` is 7,244 lines; no public final-AST API changed.

## Task 258B3M2B2B2C Frozen Final-AST Sibling

ResolvedTypedAst gains no API. Final assembly must enumerate B2C as a third
exact structure-statement sibling beside B2A/B2B, selected by the complete
181-byte/86-node profile rather than the common
`application = None` / `structure = Some` shape. It revalidates Task-252
`7/4/3`, Task-254 `2/0/1/3/1/4/9`, Task-256 equality pairs
`Primary(0/1)` and `Primary(5/6)`, Task-258 base `1/2/2/2/2`, and witness
`1/0` targeting update `Structure(0)`.

The final clone must preserve structure, statement, and witness handoffs
byte-for-byte only after every source, arena, provenance, ownership, row,
and fingerprint check succeeds. B2A/B2B/B2C or application hybrids, stale
fingerprints, swapped targets, partial/reverse/repeated installation, and
subtree ownership substitution reject atomically. All semantic, proof,
goal, overload, coercion, obligation, and theorem-acceptance outputs remain
empty. Implementation and its final-clone tests remain open.

## Task 258B3M2B2B2C Implemented Final-AST Sibling

Final assembly now enumerates B2C beside B2A/B2B and revalidates the complete
source, arena, Task-252/254/256/258, witness, structure-fingerprint, and
`Structure(0)` target contract before cloning. It adds no public accessor,
schema, or semantic output. Hybrid, stale, swapped, partial, reverse,
repeated, and subtree-substitution cases fail atomically.

The frozen checker final-clone test and runner typed/final rollback test pass;
the complete four-plus-five matrix passes and final implementation review has
no findings. Final source/documentation and quality reviews remain pending.

## Task 258B3M2B2B2C Broad Final-AST Verification

The broad format, Clippy, checker, runner, and full workspace gates, focused
`4/4` and `5/5`, and sibling `12/12` and `21/21` suites pass with unchanged
counts and hashes. This adds no final-AST surface or semantic claim.
Independent final source/documentation and quality reviews, the implementation
commit, and post-commit inventory remain pending.

## Task 258B3M2B2B2C Final Final-AST Review Status

Independent final source/documentation consistency and final quality report
**NO FINDINGS**. All nine hard gates PASS and the valid score is `98/100`;
final-AST evidence and boundaries remain unchanged. Only cached-diff/staging
audit, implementation commit, and post-commit inventory/fresh-next-task gates
remain pending.

## Task 258B3M2B2B3A Frozen Final-AST Contract

`ResolvedTypedAst` adds only exact allow/revalidate/clone support for
`SetTerm(SourceSetTermId)` with application/structure fingerprints absent
and set fingerprint present. Revalidation follows source/AST, local resolver
plus label, Tasks 48/252/255/256/258 base, witness, atomic publication, then
final clone. Unsupported B3A statement/witness combinations and witness/
fingerprint revalidation failures yield
`ResolvedTypedAstError::InvalidSourceStatement`. Earlier lower-stage
mutations retain their existing owning variants according to that
precedence, including `InvalidSourceSetTerm` and
`InvalidSourceAtomicFormula`. Every failure publishes no partial state and
allows immediate clean replay. Lower `SourceStatementWitnessError` values
remain internal; no final error variant or display text changes.

Final clones preserve one witness/zero names, target `set-term#0`, the
optional set fingerprint after existing optional debug fields, literal
legacy bytes, and empty semantic/proof/goal/IR surfaces. No semantic result
or public route is added.

## Task 258B3M2B2B3A Implemented Final-AST Closure

`ResolvedTypedAst` now allows only the exact B3A statement plus set-only
witness tuple, revalidates the set/witness fingerprints, and clones the
source set, statement, and witness handoffs. Set and atomic defects retain
`InvalidSourceSetTerm` and `InvalidSourceAtomicFormula` precedence; unsupported
or stale upper combinations remain `InvalidSourceStatement`. Final clone,
replay, debug, and empty semantic/proof/goal surfaces pass their frozen
tests. No semantic result, error variant/text, or public/active route was
added. The second source/documentation consistency repeat and final
documentation/boundary reread report **NO FINDINGS**; parent final
verification listed in the crate plans passes, including exact `39`-file
scope. Independent final read-only quality review reports **NO FINDINGS**.
All nine hard gates PASS with no score cap; the valid score is `98/100`
(`20/20/15/14/10/10/5/4`). The stated semantic and coverage deferrals
remain unchanged as residual risk. Only the dedicated implementation
commit, post-commit invariant verification, and fresh next-task inventory
remain pending.

## Task 258B3M2B2B3B Frozen Final-Clone Boundary

Final assembly must recognize the exact B3B statement/witness profile,
revalidate its zero-edge Task-255 dependency and set-only fingerprint, and
clone it without changing any debug bytes or semantic tables. Corrupt
source ownership, label, Task-48/252/255/256/258 rows, witness linkage, or
fingerprint fails with the already owning final error. Choice,
comprehension, `qua`, existential matching, proof acceptance, Core/CFG/VC,
and all other semantics remain absent.

## Task 258B3M2B2B3B Implemented Final-AST Closure

`ResolvedTypedAst` now admits only the exact B3B statement/witness/set-only
tuple, revalidates the zero-edge Task-255 handoff and fingerprints, and
clones source set, statement, and witness state. Set and atomic defects keep
their existing lower error precedence; stale or hybrid upper state remains
`InvalidSourceStatement`. Final replay preserves debug bytes and empty
semantic/proof/goal tables. No error, public route, or semantic result was
added. The final implementation repeat reports **NO FINDINGS** after the
post-auth/stage-prefix guard additions. Source/documentation consistency
repeat also reports **NO FINDINGS**. Final documentation/boundary and
independent quality reviews report **NO FINDINGS**, all hard gates PASS,
valid `98/100`.

## Task 258B3M2B2B3C Frozen Final Boundary

The future B3C route must reuse
`with_source_set_term_statement_witnesses` and revalidate the exact
Task-48/252/255/256/258/witness tuple before cloning into
`ResolvedTypedAst`. Its only new graph edge is
`Witness(0) -> SetTerm(0)`; choice nonemptiness, stable choice symbols, facts,
proofs, and every semantic table remain empty. This documentation task makes
no resolved-AST source or public-API change.

## Task 258B3M2B2B3C Implemented Final-AST Closure

`ResolvedTypedAst` now admits only the exact B3C choice statement/witness/
set-only tuple, revalidates all Task-48/252/255/256/258 and witness fields,
and clones the authenticated source set, statement, and witness state.
Lower set/atomic error precedence is preserved; stale, hybrid, or
non-generic-guard upper state fails with the existing statement error.
Replay preserves debug bytes and empty semantic/proof/goal tables. No error,
public route, dependency, or semantic result was added.

## Task 258B3M2B2B3D Frozen Final Boundary

The future final projection accepts only the independent exact B3D set-only
fingerprint tuple, revalidates Task-48/252/255/256/258 and the one
`SetTerm(0)` witness, and clones the authenticated state. Stale/hybrid/family
mixes and wrong `QuaBase`/`QuaWidening` state fail through existing errors.
No schema, error, debug, dependency, active route, or semantic table changes.

## Task 258B3M2B2B3D Implemented Final-AST Inventory

`ResolvedTypedAst` now accepts only the authenticated exact B3D set-only
tuple, revalidates Task-48/252/255/256/258 plus the one `SetTerm(0)`
witness, and clones source-term, source-set-term, atomic, statement, and
witness state. Stale fingerprints, wrong `QuaBase`/request state, family
hybrids, occupied semantic tables, and proof/expression coexistence fail
through existing errors; replay and clone preserve debug bytes.

The final owner grows from 7,268 to 7,270 lines solely for the private
allowlist. Public schemas, errors/debug grammar, dependencies, active routes,
and all semantic/proof/goal tables remain unchanged or empty. Focused and
package tests, formatting, and Clippy pass. Independent implementation review
reports **NO FINDINGS**. Source/documentation consistency and boundary review
also report **NO FINDINGS** after the three bounded documentation fixes.
Full workspace tests, five CLIs, and count/hash reruns pass. Independent
final read-only quality review reports **NO FINDINGS**; all nine hard gates
PASS with no cap at valid `100/100` (`20/20/15/15/10/10/5/5`). Only
exact staging/cached-diff review, implementation commit, and
post-commit/fresh-next-task gates remain pending.

## Task 258B3M2B2B3E Frozen Final Boundary

The future final projection may accept only the authenticated B3E tuple:
Task-252 primary fingerprint, Task-255 comprehension fingerprint, Task-256
formula handoff, Task-258 statement handoff, and one set-target witness.
Clone/revalidation must preserve the generator/type-site tables while
semantic tables remain empty. Stale fingerprints, partial publication,
sibling hybrids, and any generator binding/capture state fail atomically.
No final-AST public schema changes.

## Task 258B3M2B2B3E Implemented Final-AST Inventory

`ResolvedTypedAst` now accepts only the authenticated exact B3E tuple,
revalidates Task-48/252/255/256/258 plus the one `SetTerm(0)` witness, and
clones generator/type-site, set-term, atomic-formula, statement, and witness
state. Wrong generator/type-site/request state, partial/extra ownership,
stale fingerprints, sibling hybrids, occupied semantic tables, and
proof/expression coexistence fail through existing errors. Replay and clone
preserve debug bytes.

The final owner grows from 7,270 to 7,272 lines solely for the private
allowlist. No public schema, error/debug grammar, dependency, active route,
or semantic/proof/goal table changed. Independent implementation review
reports **NO FINDINGS**. Final source/documentation consistency also reports
**NO FINDINGS** after the bounded design corrections. Full verification
PASSes; independent final quality reports **NO FINDINGS**, all nine hard
gates PASS, valid `100/100`. Staging and post-commit gates subsequently
closed in implementation commit
`e4479691db3b0a8785bb16e94d386bd71a394274`; fresh inventory selected
Task 258B4A.

## Task 258B4A Frozen Final Boundary

Final construction revalidates the optional composite-formula and
formula-composition statement fingerprints against the installed Task-257B1
handoffs, the exact `Composite(0)` statement/candidate links, the one-owner
resolver provenance, and mutual exclusion with atomic statement families.
Clone and debug output preserve the complete syntax-free transaction.

Contexts, types, facts, coercions, initial obligations, diagnostics, theorem
acceptance, and proof state remain empty. Any stale, partial, hybrid, or
cross-family tuple fails with
`ResolvedTypedAstError::InvalidSourceStatement` before final publication and
leaves replay available.

Repeated read-only documentation review reports **NO FINDINGS**. Independent
final quality passes all nine hard gates with no cap at valid `100/100`;
only staging, commit, and post-commit inventory remain.

## Task 258B4A Implemented Final Boundary

Final construction revalidates the installed composite, composition, and
statement fingerprints, exact `Composite(0)` statement/candidate links,
resolver-backed owner, lower rootless-arena contract, and atomic-family
exclusion before cloning. Nineteen statement corruptions, three absent-lower
tuples, stale/hybrid inputs, and occupied incompatible state fail through
`ResolvedTypedAstError::InvalidSourceStatement` without publication and
permit replay. Successful clone and debug bytes are stable; contexts, types,
facts, coercions, obligations, diagnostics, theorem acceptance, and proof
state remain empty.

## Task 258B4B Frozen Final Boundary

Final construction may accept the composite-statement family only as a
matched B4A/Task-257B1 or B4B/Task-257B2 tuple. B4B revalidates the exact
124-node rootless lower arena, the Task-252/256/257/B2 profiles and
fingerprints, one resolver owner at `0..165`/origin `[2,0]`, upper
`1/1/1/0/1`, and both `Composite(0)` links before cloning.
Final dispatch uses the two exact crate-private statement predicates rather
than the shared cardinality shape; a B4B statement may never enter B4A's
Task-257B1 branch or vice versa.

Every cross-profile hybrid, stale fingerprint, missing lower handoff,
rooted/relocated coherent near miss, atomic-statement coexistence, or
occupied semantic state fails through
`ResolvedTypedAstError::InvalidSourceStatement` without partial output and
permits replay. B4A bytes and all lower-owned error precedence remain
unchanged. No final-AST schema, debug grammar, or semantic table is added.

## Task 258B4C Frozen Final Boundary

Final construction may admit B4C only with the exact Task-257B3 lower
handoffs and exact B4C statement identity. Raw authentication of the private
139-byte source, 66-node/root-65 Surface identity, raw resolver origin
`[2,1]`, contribution 0 anchored at `0..18`, and enriched `1/1/1/1/0`
belongs to the runner selector and `SourceStatementProducer`. Before
cloning, final construction revalidates the producer-authenticated statement
handoff rows and identity retained by the typed AST, the matched lower
fingerprints, rootless lower arena, binding `4/4/0`, primary `6/6/0`, atomic
`3/0/0/0/0/0/0/6/6`, composite `3/0/1/3/3/2/6`, composition `3/6`,
the 24-site lower partition, and upper `1/1/1/0/1` before cloning.

Statement 0 and candidate 0 must both target `Composite(0)`; context 0 must
expose exactly `[0]`, input facts remain empty, and theorem node 62 is the
only upper-owned Surface node. Final dispatch recognizes only the exact
B1/B4A, B2/B4B, and B3/B4C pairings. Cross-family hybrids, stale or partial
handoffs, rooted/relocated arenas, altered ownership, an active atomic
statement family, occupied semantic state, or lower-selector mismatch fail
through the existing `ResolvedTypedAstError::InvalidSourceStatement`
boundary before publication and permit replay.

This frozen path adds no final-AST schema, public API, debug/error grammar,
fact, theorem-acceptance, proof, or IR table. The separately committed
lower-selector prerequisite must complete before this final projection is
implemented.

## Task 258B4C Implemented Final Boundary

Final assembly recognizes B4C only when the typed AST retains exact B3 lower
handoffs, B4C upper rows, matching fingerprints, the rootless exact 66-node
arena, and `24/1/41` ownership. It validates all anchors and normal recovery
states before cloning. Partial/cross-family state, stale fingerprints,
relocation, atomic statement coexistence, or occupied semantic tables fail
without partial publication and permit deterministic replay.

The final AST remains a clone-preserving syntax/provenance projection:
checked formulas, statement semantics, proofs, proof nodes, and terminal
goals stay empty, and no schema or public API changed.

## Task 258B5A Frozen Final-Assembly Boundary

Final assembly may recognize B5A only when the typed AST contains the exact
93-node/root-92 arena, matched B5A base/reference fingerprints, 20/73
ownership, label scope `[0]`, citation scope `[0,1]`, and the resolver's sole
keyed node 82 mapped to label key 0. The preliminary resolver has no keyed
node; the final resolver has exactly one resolved id and no diagnostic,
name, import, or export entry.

Clone/replay must revalidate every row, range, origin, ordinal, recovery
state, scope-prefix relation, and empty semantic table before publication.
Partial or cross-profile installation, stale fingerprints, relocation,
recovery, wrong contribution or keyed node, Task-248/other-family
coexistence, and any occupied semantic output fail atomically. B1 debug
bytes and public APIs remain unchanged.

## Task 258B5A Implemented Final-Assembly Boundary

Final assembly now recognizes the frozen B5A transaction only when the typed
AST retains the exact matched base/reference profiles, all dependency
fingerprints, the 93-node/root-92 arena, `20/73` ownership, label scope
`[0]`, citation scope `[0,1]`, and resolver node 82 mapped to label key 0.
It revalidates every resolver node kind, range, child order, recovery state,
origin, ordinal, contribution, and scope-prefix relation before cloning.

The unchanged B1 same-scope transaction and B5A ancestor/descendant
transaction remain the only admitted reference profiles. Partial,
cross-profile, stale, relocated, recovered, wrongly keyed, or semantically
occupied state fails before publication and leaves replay available. Checked
formulas, accepted statements, proofs, proof nodes, terminal goals, facts,
and all downstream IR remain empty; no public schema or API changed.

## Task 258B5B Frozen Imported Final-Assembly Boundary

Final assembly may recognize B5B only after the lower opt-in environment is
exactly `8/1/1/3/1` and the typed AST retains matched base
`1/2/2/2/2`, reference `0/1`, all dependency fingerprints,
57-node/root-56 arena, and `8/49` ownership. Resolver node 48 is the sole
keyed resolved node; the replay has one resolved import, one resolved label
reference/id, and zero exports, name references, and diagnostics.

Resolved import id 0 is owned by unkeyed `ImportAliasDecl` node 29, range
`7..27`, spelling `import parser.type_fixtures;`, alias `None`, and resolves
to `<package>::parser.type_fixtures`. Its current-source/current-module
origin has anchor `7..27`, path `[0]`, no import edge, and normal recovery.
Nodes 28/29/30 remain unkeyed `NotApplicable` nodes with their exact
Surface identities; node 48 alone carries label key 0. The imported
projection origin independently uses the current source, declaring imported
module, anchor `7..27`, path `[1,0]`, no import edge, and normal recovery;
the reference origin uses the current source/current module, anchor
`136..139`, path `[48]`, no import edge, and normal recovery.

The immutable clone revalidates the imported/public/exported theorem
projection, `target=Imported`, `SimpleImported`, scope `[0]`, every origin,
module, namespace, contribution, anchor, structural path, range, ordinal,
node kind, child order, recovery state, and resolver key. It also preserves
exact B1/B5A local behavior and rejects all B1/B5A/B5B cross-pairs,
partial/stale/recovered state, and occupied semantics before publication.

Checked formulas, facts, accepted statements, proofs, proof nodes, goals,
status propagation, and downstream IR remain empty. The only public checker
surface change is the citation-target enum/field/getter and imported citation
kind frozen in the crate plan.

## Task 258B5B Implemented Imported Final-Assembly Boundary

Final assembly now clones a typed B5B installation only after independently
revalidating the resolved import owner/range/spelling/alias/result, imported
projection origin and public/exported theorem identity, reference candidate
origin, node 48 kind/key, resolution result, citation target/kind/ordinal,
and the complete `8/49` ownership partition. Any stale, relocated,
recovered, partial, wrongly keyed, or B1/B5A/B5B cross-paired state fails
before publication and leaves valid replay available.

The clone preserves byte-identical B1/B5A local behavior and the exact B5B
`label_node=absent`/`source=imported` debug schema. Checked formulas, facts,
accepted statements, proofs, proof nodes, terminal goals, diagnostics,
status propagation, and downstream IR remain empty. No public final-owner
schema or semantic API changes.

## Task 258B5C Frozen Final-Assembly Exclusion

Both B5C candidates resolve to `UnresolvedLabelRef` with
`has_unresolved = true`. The source-statement handoff therefore rejects them
before typed installation, and final assembly receives no B5C state to
clone. The R-032A structural mirror remains
`NodeResolutionState::NotApplicable` with no key and is not a semantic
success; there is no `LabelResolution::Resolved` result,
label/citation/reference DTO,
checked formula, fact, accepted statement, proof node, goal, diagnostic
carrier, or downstream IR row.

Final assembly must not turn an active resolver failure into an empty or
partial checker profile. B1/B5A/B5B clone validation and every existing
debug byte remain unchanged; all Surface nodes in the two negative sources
stay syntax-owned.

The default-deny R-032B traversal requires exact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock`, exact-one normal Root/CompilationUnit structural children, and
direct-normal theorem scanning. It leaves other item children and excluded
formula/token/wrapper,
unsupported/recovered/malformed, qualified/grouped/bulk, and template nodes
without a collected row, ordinal, or descent, so none can become a
`ResolvedTypedAst` owner. Its positive-edge, forbidden-relocation, and
mixed-list tests remain resolver-owned.

Runner mutations of environment module, projection module/namespace/
contribution, or contribution cardinality/id/kind/record module/source id
must stop at `proof_scope_input`; they cannot reach final assembly or be
reclassified as authenticated confinement.

## Task 259 Frozen Final Handoff

Future final assembly has one optional
`SourcePredicateDefinitionHandoff`. It may appear only with the exact
Task-248/249/252/256 lower handoffs and the atomically installed complete
obligation table. `ResolvedTypedAst` revalidates the four fingerprints, all
five dense tables, resolver-derived predicate origin, every cross-family ID,
and the sole `Pending` `PredicatePropertyCorrectness` row before publishing.

Clone and debug rendering preserve exact IDs, order, strings, anchors, and
dependency bytes. Final assembly cannot replace the handoff, reinterpret the
guard or property, consume the justification subtree, construct a FOL goal,
or turn the pending obligation into a fact, proof, VC, accepted definition,
or axiom.

## Task 248 Two-Parameter Profile-B Final Clone

Final assembly uses the unchanged source-context clone path. It accepts
Profile B only after typed validation has retained exact
`1/2/2/2/2/2/0` state and revalidates the existing handoff invariants; no
separately replaceable runner input is added. Clone/debug order and bytes are
deterministic, and invalid Profile-B state cannot be repaired or partially
published. No Task-249+, Task-259, fact, obligation, proof, or semantic row is
created by this lower profile.

## Task 260 Final Functor-Definition Ownership

`ResolvedTypedAst` gains one optional immutable Task-260 handoff cloned only
through the typed-owned finalization path. It revalidates the five tables,
all lower fingerprints, the exact baseline-plus-appended obligation table,
and the required absence of a Task-259 handoff and
`PredicatePropertyCorrectness` baseline row. There is no second final input/
getter for obligations and no resolver-side reconstruction. Clone and debug
order are deterministic; failure cannot repair, renumber, or partially
publish either family.

When Task 260 is absent, any `FunctorExistence` or `FunctorUniqueness` row is
an orphan and final assembly rejects it. When Task 260 is present, final
assembly requires exactly the two final rows linked by its correctness table
and rejects any additional row of either functor kind.

## Task 249R Definition-Return Clone Addendum

`ResolvedTypedAst` gains no Task-249R input or field. Final assembly trusts the
immutable handoff already revalidated by `TypedAst` installation and only
clone-preserves its definition-return table and roots. The final debug
contains the same combined source-type fingerprint exactly once; it never
reconstructs return rows from resolver payloads.

## Task 249M Mode-RHS Clone Addendum

`ResolvedTypedAst` gains no Task-249M input, field, or getter. Final assembly
only clone-preserves the mode-RHS table already revalidated inside the single
typed-owned source-type handoff. The combined fingerprint occurs exactly once;
final assembly neither reconstructs the RHS from resolver spelling nor grants
inhabitation, expansion, sethood, acceptance, proof, or fact semantics.

## Task 249M Active Clone Boundary

The implemented mode-RHS table remains inside the existing typed-owned
source-type handoff. `ResolvedTypedAst` adds no field or input and clone-
preserves the validated `2/3/0/0/1` fingerprint exactly once; the four Task-
249M tests confirm that all final semantic result tables remain empty.

## Task 262 Active Final Mode-Definition Ownership

Final assembly clone-preserves the optional Task-262 handoff only from its
typed owner and revalidates the exact Task-248/249/249M fingerprints, all six
dense tables, baseline count, property-to-obligation link, and single pending
`Sethood` suffix. `ResolvedTypedAstInputs` gains no replaceable Task-262 field;
the only new public projection is the read-only getter.

Every mixed Task-259/260/261/262 state is rejected, including reverse install
orders that older sibling installers cannot detect. Without the handoff, a
goal or provenance in the `source.definition.mode` domain is an orphan and is
rejected, while unrelated existing-kind `Sethood` rows remain permitted. Final
assembly does not answer the inhabitation request or publish acceptance,
expansion/sethood facts, proof, Core, control flow, or VC output.
## Task 249S Standalone Member-Type Clone Addendum

Task 249S adds no final field or semantic route. `ResolvedTypedAst` clone-
preserves the already validated `SourceTypeApplicationHandoff` through its
existing `source_type` field. Final assembly creates no expression metadata,
candidate, coercion, fact, diagnostic, formula, statement, proof, obligation,
acceptance, or Task-263 structure result from the four lower owner rows.

## Task 249S Active Clone-Preservation Result

Repeated final assembly preserves the exact immutable source-type handoff and
deterministic debug fingerprint by value equality. It adds no final field,
installer, semantic projection, diagnostic, obligation, fact, proof, or
acceptance output.

## Task 263 Frozen Final Ownership

Final assembly may obtain the future structure-definition handoff only by
cloning `TypedAst`. It revalidates the complete source-type fingerprint,
`2/4/1/2/0` graph, zero coherence requests, and unchanged obligation table,
then exposes one getter. `ResolvedTypedAstInputs` has no replacement field and
`ResolvedTypedAstError::InvalidSourceStructureDefinition` is the only new
failure variant. Reverse mixed Tasks-259--262 states fail closed; no semantic
table or accepted result is synthesized.

Final validation compares the current complete obligation table with the
private byte-equal snapshot as well as its frozen count. A changed row with an
unchanged length therefore fails. Snapshot bytes do not enter stable
`debug_text()` and no replaceable final input can bypass equality.

## Task 263 Active Final Ownership

Final assembly now clone-preserves and revalidates the Task-263 handoff,
complete unchanged obligation table, private baseline snapshot, lower
fingerprint, rows, zero-coherence profile, and arena ownership. It rejects
mixed Tasks 259--263 and publishes no semantic result beyond immutable source
transport.

## Task 248P Property Context Final Ownership

Task 248P adds no final input or assembler branch. `ResolvedTypedAst` only
clone-preserves the Profile C handoff installed in `TypedAst` and reuses the
existing source-context validation/debug position. A caller cannot replace the
handoff or promote recovered input. Property payload/provenance and every
semantic result remain absent until Task 264.

## Task 248P Active Final Ownership Boundary

The implemented source-context handoff remains owned through the existing
`TypedAst` clone path. No final input, assembler branch, getter, replacement
path, debug field, or semantic result was added; recovered Profile C still
cannot reach final assembly.

## Task 264 Frozen Final Ownership

`ResolvedTypedAst` will expose one read-only
`source_property_implementation()` getter. Final assembly accepts it only when
the typed source/context/type/term and optional lower fingerprints replay
exactly and the final obligation suffix matches style: two linked pending rows
for means, zero for equals. Orphan/extra property kinds, a Task-259 handoff,
or mismatched fingerprint fail as
`ResolvedTypedAstError::InvalidSourcePropertyImplementation`.

Final assembly clone-preserves transport only. It adds no expression metadata,
checked formula, fact, proof, diagnostic, acceptance, property value, or IR/VC
row. The docs prerequisite and Task 249PI add no final Task-264 owner.

## Task 249PI Final Ownership Boundary

Task 249PI adds no `ResolvedTypedAst` field, constructor input, getter, or
serializer. The existing `source_type` handoff is clone-preserved once, with
its complete `source-type-application-debug-v1` fingerprint and exact
`1/3/0/0/0/2` profile. Final installation rejects lower drift through the
existing Typed source-type validation. Property identity, return association,
initial obligations, diagnostics, facts, proof, acceptance, and Task-259 data
remain absent until their separately frozen owners.

## Task 249PI Implemented Final Ownership

Final assembly clone-preserves the exact combined source-type handoff and
rejects the repaired orphan-member installation shape. No final field, getter,
semantic result, or Task-259/264 ownership was added.

## Task 264 Active Final Ownership

Final assembly now clone-preserves the typed property-implementation handoff
only after replaying every frozen lower fingerprint, row, arena identity, and
style-specific obligation suffix. It rejects orphan/extra property kinds and
all Task-259--263 sibling coexistence with
`InvalidSourcePropertyImplementation`. The read-only getter publishes
transport only; expression metadata, checked formulas, facts, diagnostics,
proof/acceptance state, property values, Core, CFG, and VC remain unchanged.

## Task 269A Frozen Final Ownership

Final assembly clone-preserves the typed
`SourceProofLocalDeclarationHandoff` through one read-only getter and no
`ResolvedTypedAstInputs` replacement field. It recomputes all lower/final
fingerprints and the binding transition before publication. An orphan,
half-installed, stale, or corrupt value fails as
`InvalidSourceProofLocalDeclaration`.

The final owner publishes only the definition-site association and extended
binding environment. Expression metadata, overload/coercion/cluster tables,
checked formulas, statement semantics, proofs/goals, obligations, diagnostics,
facts, Core, CFG, and VC remain empty or unchanged.

## Task 269A Active Final Ownership

Final assembly now clone-preserves the optional handoff and replays the exact
lower/final transaction before publishing it. A top-level preflight checks
term, atomic formula, statement, witness, and forbidden-reference presence
before legacy lower validators, so every half-installed injection fails with
the dedicated final error. Orphan and same-length stale injections fail the
same way; valid clone/replay is deterministic and every deferred semantic
table remains empty.

## Task 269B frozen final replay increment

Final assembly reuses the same proof-local getter and seven-phase replay for
the exact B3M1 bundle. The top preflight still requires term, atomic,
statement, and witness ownership and forbids statement references. It then
rejects cross-profile/stale/partial inputs through the existing dedicated
error. No final field, error, node role, or semantic table is added.

## Task 269B active final replay increment

Final assembly now clone-preserves and revalidates the exact B3M1 handoff
through the existing getter and seven phases. Valid replay is deterministic;
all orphan, partial, stale, cross-profile, and arena-corrupt inputs fail with
the existing proof-local error. Every proof, goal, fact, obligation, diagnostic,
and downstream semantic table remains empty or unchanged.

## Task 269CP final-owner exclusion

Task 269CP creates no `TypedAst` or `ResolvedTypedAst` field, getter,
installer, debug section, replay phase, or final clone. Its lower output stays
inside `mizar-test`. Task 269C must separately freeze any final owner and may
not infer one from this prerequisite.

## Task 269C frozen final owner

`ResolvedTypedAst` clone-preserves the new binding-only handoff after complete
revalidation against the empty node/semantic profile. It exposes a read-only
getter and appends the handoff's deterministic debug section. It adds no
expression, candidate, formula, statement-semantic, proof, goal, obligation,
fact, or diagnostic row. Orphan, cross-family, stale, or semantic-coexistence
input is rejected; Task-269A/B final bytes remain unchanged.

## Task 269C Active Final Ownership

Final assembly now performs that complete replay and clone-preservation.
Dedicated tests reject orphan, stale, cross-family, and nonempty semantic
inputs while valid cloning preserves the exact handoff and empty semantic
profile.

The final optional handoff is privately boxed for stack-size stability while
preserving the frozen getter, debug, clone, and replay behavior.

## Task 269CT Final Composite Replay

`ResolvedTypedAst` adds one boxed optional
`SourceProofLocalLetTypeHandoff`, exposed by a const getter and cloned only
from the authenticated typed owner. The exact three typed nodes replay
one-for-one with source-preserved role `source.proof-local.let.type`; direct
source-type/Task-269C fields and every resolved semantic table remain empty.
Malformed, duplicate, occupied, or semantically nonempty profiles fail with
`InvalidSourceProofLocalLetType`.

## Task 269CT Implemented Final Replay

Final assembly clones and revalidates the composite before arena construction,
requires the exact three-node Typed profile and otherwise empty source/
semantic/overload state, and uses a Task-specific predicate that requires
`node_hints.is_empty()`. All three nodes map one-for-one to role
`source.proof-local.let.type`. Statement-transport hints and nonempty expression
metadata fail atomically with `InvalidSourceProofLocalLetType`.

## Task 269GP No-Final-Owner Boundary

Task 269GP publishes no `ResolvedTypedAst` sibling or node hint. Existing
Task-269C/CT replay and all final semantic exclusions remain byte-identical.

The implemented runner route confirms this with no final owner or serializer
change.

## Task 269GS No-Final-Owner Reconciliation

Resolving lexical scope does not install a `ResolvedTypedAst` field or final
owner. A future 269G contract must decide the binding handoff and replay from
the canonical block rule; 269GT remains the separate type owner. All semantic
tables remain unchanged in Task 269GS.

## Task 269G Final Owner

`ResolvedTypedAst` clone-preserves one validated
`SourceProofLocalGivenBindingHandoff` through a read-only getter and rejects
duplicate, stale, cross-family, or semantic-coexisting installation with
`InvalidSourceProofLocalGivenBinding`. It adds no node or semantic table; the
debug block follows the existing `let` binding/type slot.

## Task 269G Active Final Ownership

Final assembly now revalidates and clone-preserves the complete Given binding
handoff. Dedicated final-replay tests reject stale, cross-family, and node-hint
conflicts while valid replay preserves the exact lexical binding and empty
semantic profile; the Typed installer tests independently reject all six
nonempty semantic-table families before final assembly. Private boxed storage
keeps the aggregate's stack size stable without changing the frozen getter,
installer, clone, or debug contract.

## Task 269GT Frozen Final Owner

Final assembly will clone and revalidate one boxed
`SourceProofLocalGivenTypeHandoff`, map its exact three-node arena with role
`source.proof-local.given.type`, and reject stale, sibling, node-hint, or
semantic coexistence. The direct Given binding and generic source-type slots
remain empty because the composite owns both dependencies. No semantic table
is populated.

### Task 269GT implemented final owner

Resolved assembly clones only `SourceProofLocalGivenTypeHandoff`, rejects occupied or semantic inputs, and maps all three typed nodes one-for-one to source-preserved role `source.proof-local.given.type`. Direct source-type/Given-binding/Let owners and every semantic table remain empty.

## Task 269GUP Final-owner Exclusion

GUP adds no `ResolvedTypedAst` field, getter, error, node role, clone replay,
or final table. Final assembly is byte-identical. GUPT/GU may freeze later
owners only after consuming the GUP dependency in order.
### Task 269GUP implemented binding profile

The frozen six-file transaction and its exact four checker/four runner tests are implemented. Libraries measure `502/564`; checker/runner production is `30/172531` and `37/74826`, with unchanged path hashes and content hashes `e0342952a01a0b379cf7b06ad243cd40a1656e940480196323cf43fbe7d8f7c5` / `8fe7c8c0b7e855e5113f3830873e133f42c8048a3272055e2fddd5ebd9cbb1bc`.

This closes only dormant private lexical-binding evidence and grants zero active corpus, trace, type, term/use, condition/fact, goal/proof, obligation, diagnostic, or CLI credit. Task 269GUPT is next; Task 269GU, capture, and Task 270 remain deferred.

## Task 269GUPT Frozen Final Ownership

`ResolvedTypedAst` adds only boxed optional `source_proof_local_given_use_type` and its getter after the old Given-type slot. Assembly revalidates the exact Typed profile, clones the composite, and maps all three nodes to source-preserved role `source.proof-local.given-use.type`. `InvalidSourceProofLocalGivenUseType` reports `resolved typed AST source proof-local given-use type handoff is invalid`. All semantic tables, node-hint inputs, old owners, and direct binding/type fields remain empty and mutually exclusive.

### Task 269GUPT implemented final owner

The boxed owner, exact revalidation, three source-preserved nodes, and
both-order exclusion are implemented and tested. Final semantic tables and
node-hint inputs remain empty.

## Task 269GU Frozen Final Ownership

`ResolvedTypedAst` adds only boxed optional
`source_proof_local_given_use_term` and its getter. Assembly revalidates the
exact Typed composite, clones it, and maps all six nodes to source-preserved
role `source.proof-local.given-use.term`. The exact invalid-handoff string is
`resolved typed AST source proof-local given-use term handoff is invalid`.
Node hints and every semantic table remain empty; all old owners are mutually
exclusive in both orders.

### Task 269GU implemented final owner

The boxed owner, exact revalidation, all six source-preserved nodes, and
both-order exclusion are implemented and tested. Final semantic tables and
node-hint inputs remain empty.

## Task 269GCP Final-owner Exclusion

GCP adds no `ResolvedTypedAst` owner, getter, error, clone path, role, or node
hint. Final assembly remains byte-identical and cannot observe the private
lower row. GC also remains a non-final dependency; GCT/GCU must later freeze
their own mutually exclusive final owner.
