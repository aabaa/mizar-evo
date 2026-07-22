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
