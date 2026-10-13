# Task SESSION-SOURCE-CAPTURE: acquire sources before snapshot-scoped allocation
Canonical language: English; [Japanese pointer](../ja/SESSION-SOURCE-CAPTURE.md).
Status: complete. Tier: full. Owner: [session plan](../../mizar-session/en/00.crate_plan.md).
Scope: session task 33, prerequisite for LSP tasks 6–7; no language behavior.
Authority: session source/snapshot/source-map owners; architecture 02/11/12 and internal 03/06.
Dependencies: existing native disk/open-buffer/generated loading, canonical snapshot validation and registry leases.
Gap: source acquisition allocates SourceId before the source summaries can determine its target BuildSnapshotId.

Add `SnapshotRegistry::load_snapshot(request, context, sources)`.
`context` is `(WorkspaceRoot, Vec<DependencyArtifactRef>, Hash lockfile,
ToolchainInfo, Hash verifier_config)`; `sources` is `Vec<(DiskSourceLoader, SourceInput)>`.
Return `(BuildSnapshot, SnapshotLease, Vec<(LoadedSource, SourceInput)>)`
inside `Result<_, SnapshotError>`; captured pairs preserve input order.
Use existing public records and private helpers; no new DTO, adapter or file.
Acquire/normalize each source once without SourceId. Retain original origin
descriptors and raw OpenBuffer Arc; correct the captured normalized path.
Validate all acquired metadata and context with existing rules before snapshot hashing
or allocating any source/lease id. Share canonical bytes with create_snapshot;
include edition, exclude session ids/origin/version. Reject duplicate modules;
upstream chooses disk versus buffer input, without implicit override here.
Allocate SourceIds/maps at that actual id; register only after all sources
assemble successfully, returning a real ActiveBuild lease. Native source errors
remain typed under `SnapshotError::SourceLoading { error: SourceLoadError }`.
Failures leave registry/current marks/live leases unchanged; consumed allocator
counters need not roll back. Caller owns returned source/map/raw-text resources.

Preserve existing load/create behavior, negative-version policy, normalization,
canonical schema and language/proof acceptance. No provisional ids, rereads,
raw-text hash substitute, invented artifact/cache/currentness or scheduler state.
Test all origins/maps/raw provenance, canonical equivalence/order/context,
actual allocator targets, late I/O mutation and rejection/allocation failures.
Require independent specification, test-sufficiency, implementation,
volume/scope and consistency reviews; full fmt, Clippy and cargo test.
Exit: native acquisition ordering holds; LSP 6–7, Steps 6–8 and Goal stay open.
