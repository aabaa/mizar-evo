# Task DRIVER-OPENBUFFER-GENUINE-ARTIFACT: genuine editor artifacts
Canonical language: English; [Japanese pointer](../ja/DRIVER-OPENBUFFER-GENUINE-ARTIFACT.md).
Status: frozen prerequisite; implementation blocked by the decisions below. Tier: full (new public producer seam; protected genuine source-theorem path).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md). Consumers: frozen [registry activation](LSP-SNAPSHOT-REGISTRY-ACTIVATION.md#test-matrix), its eight cases and genuine end-to-end check.
Purpose: genuine OpenBuffer production; preserve Disk outcomes.

## Authority and readiness

Authority: [producer](LSP-SNAPSHOT-REGISTRY-PRODUCER-DESIGN.md#scope-and-ownership), [retained sources](LSP-RETAINED-NATIVE-SOURCES.md), [native capture](LSP-NATIVE-OPEN-BUFFER-CAPTURE.md); snapshot [association](../../mizar-lsp/en/snapshot.md#source-association), [replacement](../../mizar-lsp/en/snapshot.md#freshness-and-replacement), [retention](../../mizar-lsp/en/snapshot.md#currentness-and-retention); Driver [capture](../../mizar-driver/en/request.md#native-source-capture), [draft](../../mizar-driver/en/frontend_adapter.md#source-theorem-unpublished-module-draft), [manifest](../../mizar-driver/en/frontend_adapter.md#source-theorem-module-manifest-transaction), [events](../../mizar-driver/en/events.md#freshness-and-suppression).
Gap: source_drift/test_gap; frontend_adapter.rs rejects non-Disk drafts. Existing Disk producer/source-theorem tests remain unchanged. No language, diagnostic, trace or coverage-credit change.
OPEN DECISION D1 — provenance carrier: VerifiedArtifact/BuildProvenance lack SourceId/origin/document-version fields; CapturedSnapshot records them. Authorities do not equate cohort association with every artifact recording origin. Options: serialized provenance (out-of-scope Artifact schema/API work), or authoritative Driver-retained association with unchanged standalone schema. Recommend retained association, requiring explicit owner acceptance; never encode buffer provenance as Disk.
OPEN DECISION D2 — storage/publication: architecture12 [open buffers](../../architecture/en/12.diagnostics_and_lsp.md#treat-open-buffers-as-build-artifacts) keeps unsaved snapshots in LSP memory; genuine manifests require package-relative staging files and PackageSource/CurrentPackage publication. Options: authorize isolated ephemeral Driver staging plus OpenBuffer/InternalOnly publication, or memory-only production compatible with manifest-first consumption. Recommend isolated staging only after explicit owner resolution of the memory boundary; otherwise separately scope memory-backed production.
Stop: freeze without Rust changes; implementation requires owner resolution of D1/D2 and resumed dispatch.

## Scope and ownership

Driver owns genuine production/coherence for both origins; Session retains source-version/origin definitions; LSP retains capture, editor eligibility and registry activation. No registry resume here.
Identity: captured SourceId, package, module/path, exact captured-text source_hash and OpenBuffer LspDocumentVersion. A new Disk capture has distinct provenance, even with equal text/hash. Preserve original source/map objects and association, never reconstruct them from equal build ids.
Provenance: every artifact and SnapshotPublished must honestly retain the accepted provenance carrier; OpenBuffer includes its version/hash, Disk remains Disk. Writing buffer text to a file and capturing it as Disk is forbidden.
Lifecycle: accepted edits (including version-only/equal-id changes), close/reopen and a transition to a new Disk-origin capture invalidate old editor currentness permanently. Retained reads keep original association, never publication authority. Save alone does not relabel a capture or introduce scheduling; a subsequent Disk production requires a genuine new Disk capture. Driver lane/request guards remain separate from LSP incarnation checks and IR publisher currentness.
Fixture route: ModuleIndex::native_open_buffer_input → BuildRequestDraft::capture_sources with genuine OpenBuffer descriptors → CompilerDriver::submit_captured with real registered services/resources → SnapshotPublished. Reuse original captured text and producer validations; no fabricated/copied/replayed payload. Storage/publication details await D2.
Implementation writes: crates/mizar-driver/** only; paired Driver owners only for boundary changes. No dependencies/Cargo.toml, other crates, frozen activation contract, spec/.miz/expectation changes. Preserve lock/config, source/dependency hashes, lineage, schema and manifest completeness validations; unsupported/incomplete input returns existing no_draft/no coherent event.
Freeze writes: this pair and exactly one Task Index row per Driver plan; no supporting plan or owner-boundary amendment.

## Test matrix

Cap: six new cases, genuine producer outputs only; existing regressions do not count.
1. OpenBuffer capture produces genuine artifacts and one complete cohort; missing manifest/member or incompatible schema/hash refuses completeness.
2. Every artifact/payload preserves true origin, captured version/hash; no buffer is recorded as Disk.
3. Accepted edit/new version/hash invalidates the old set; superseded capture cannot publish current.
4. Close or transition to Disk invalidates the OpenBuffer set permanently; later genuine Disk production is distinct.
5. Mixed Disk/OpenBuffer modules yield one complete coherent set with honest origins, or refuse completeness; never partial success.
6. Existing Disk producer and source-theorem tests pass unmodified with identical behavior.

## Verification and exit

Freeze: independent specification/test-intent review, both lint_policy suites, local links/fragments/pair/cap checks; no fmt/Clippy. Stop after freeze while D1/D2 remain open.
Implementation: full-tier sufficiency, precision, volume/scope, consistency and final assessment; cargo fmt then cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test -p mizar-checker --test lint_policy; cargo test -p mizar-test --test lint_policy; affected Driver groups and genuine build end-to-end check.
Full cargo test --locked only on a named cadence trigger. Exit after decisions: six cases satisfied, Disk unchanged, two local commits and clean tree; freeze plus resumed implementation counts once.
