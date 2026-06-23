# Resolved Typed AST

## Purpose

`ResolvedTypedAst` is the final checker-owned, source-shaped semantic AST
before elaboration. It projects the phase-6 typed source shape plus phase-7
facts/traces and phase-8 overload selections into one immutable layer that LSP,
artifacts, VC generation, and elaboration can consume without re-running name
resolution, type checking, registration closure, or overload resolution.

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
- accepted cluster closure facts and replayable resolution traces;
- overload collection, viability, and specificity graph outputs that provide
  site owners, source ranges, candidate tables, and graph ids;
- selected overload results and inserted views from
  `OverloadSelectionOutput`, including inserted-view kind, reason, evidence,
  and path;
- checker-local diagnostics already produced by preceding phases.

Missing source-derived inputs are `external_dependency_gap` records. Missing
checker-owned precursor tables are task-28 assembly blockers, not permission to
scan raw syntax. Assembly must not inspect raw syntax or opaque resolver shells
to fill either kind of gap.

## Data Shape

The public data layer should keep dense ids local to the assembled output:

```rust
struct ResolvedTypedAst {
    source_id: SourceId,
    module_id: ModuleId,
    nodes: ResolvedTypedArena,
    expr_metadata: ExpressionMetadataTable,
    resolved_overloads: OverloadResolutionTable,
    inserted_coercions: CoercionInsertionTable,
    cluster_facts: ClusterFactTable,
    diagnostics: ResolvedTypedDiagnosticTable,
}
```

### Resolved Nodes

```rust
struct ResolvedTypedNode {
    id: ResolvedTypedNodeId,
    typed_node: TypedNodeId,
    source_range: SourceRange,
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
projection of cluster facts is a separate schema task.

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

## Planned Task 28 Tests

Task 28 should add Rust coverage for:

- source-shaped assembly from explicit `TypedAst` and checker outputs;
- metadata lookup by `TypedSiteRef` / expression id;
- resolved overload projection, including active refinements and inserted
  views;
- failed overload site preservation for `NoMatch`, `Ambiguous`, incompatible
  refinement join, and blocked statuses;
- no inserted coercion records for failed overload sites;
- deterministic debug rendering across equivalent input orderings;
- cluster fact id references and preservation of existing cluster-trace
  provenance.

## Deferred And External Gaps

The following remain deferred after task 27:

- Rust implementation of `ResolvedTypedAst` assembly in task 28;
- AST-wide source-to-checker extraction of task-26 selection payloads and
  source expression metadata;
- artifact emission/reuse and stable artifact schemas;
- public diagnostic-code allocation;
- active `.miz` semantic fixtures for final overload and cluster projection.

These gaps do not permit fabricated success records, raw syntax scans, or
artifact-like side outputs in task 28.
