# Source/Spec Correspondence Audit: mizar-diagnostics

> Canonical language: English. Japanese companion:
> [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

## Scope

The original task-19 audit was performed after task 18 and before the bilingual
sync and module-boundary gates. Task 21 re-runs the source-inventory scope after
the private module split and updates this document only for the moved private
helpers. The audit compares the crate-owned public API and promised behavior in
the module specifications with source and tests.

The audit does not add producer adapters, driver events, LSP protocol shapes,
artifact projections, resolver diagnostic codes, or migration wiring for
pre-existing lexer/frontend/parser/resolver diagnostics.

## Method

- Source inventory: `crates/mizar-diagnostics/src/{registry,failure_record,sink,aggregator,render,fix,explain}.rs`,
  private `src/registry/builtin.rs`,
  private `src/failure_record/{validation,debug}.rs`, and the crate root.
- Specification inventory:
  `registry.md`, `failure_record.md`, `sink.md`, `aggregator.md`, `render.md`,
  `fix.md`, `explain.md`, `consumer_adoption_decision.md`, this crate plan, and
  `todo.md`.
- Test inventory:
  `tests/{registry,failure_record,sink,aggregator,render,fix,explain,determinism,lint_policy}.rs`.
- Gap vocabulary follows the autonomous-development protocol:
  `spec_gap`, `test_gap`, `design_drift`, `source_drift`,
  `source_undocumented_behavior`, `test_expectation_drift`,
  `boundary_violation`, `repo_metadata_conflict`, `external_dependency_gap`, and
  `deferred`.

## Public API Trace

| Area | Public API surface | Spec coverage | Test coverage | Audit result |
|---|---|---|---|---|
| Crate root | Public modules `aggregator`, `explain`, `failure_record`, `fix`, `registry`, `render`, `sink`; no public macros. | `00.crate_plan.md`, `todo.md`, module specs. | `tests/lint_policy.rs`. | Covered. The root exposes only crate-owned diagnostic modules. |
| Registry | `DiagnosticSeverity`, `PhaseFamily`, `DiagnosticCode`, `DiagnosticCodeError`, `DiagnosticStatus`, `DiagnosticDescriptor`, `BUILTIN_DESCRIPTORS`, `DiagnosticRegistry`, `validate_descriptors`, `validate_registry_compatibility`, `RegistryValidationError`, and their constructors/accessors/lookup helpers. | `registry.md`, spec 22.1.1, spec 22.7. | `tests/registry.rs`, `tests/lint_policy.rs`. | Covered. Code identity, allocation, retirement, compatibility, lookup, aliases, and public enum compatibility are traced. |
| Failure records | `PipelinePhase`, `FailureCategory`, `DiagnosticId`, `DiagnosticHandle`, freshness/span/note/detail enums, `DiagnosticSpan`, `DiagnosticNote`, `DiagnosticDetails`, `DiagnosticDetailValue`, `DiagnosticDraftInput`, `DiagnosticDraft`, `DiagnosticRecord`, `DiagnosticRecordError`, `is_valid_detail_key`, and constructors/accessors/debug snapshots. | `failure_record.md`, architecture 19, spec 22.1.2. | `tests/failure_record.rs`, `tests/aggregator.rs`, `tests/render.rs`, `tests/determinism.rs`, `tests/lint_policy.rs`. | Covered. Draft/record construction, descriptor metadata, source ranges, freshness, structured details, attachment slots, and debug snapshots are traced. |
| Producer sink | `DiagnosticProducerScope`, `DiagnosticSink`, `DiagnosticBatch`, `DiagnosticSinkError`, and scope/emit/seal/batch/debug accessors. | `sink.md`, internal 03. | `tests/sink.rs`, `tests/lint_policy.rs`. | Covered. The sink collects drafts only and has no CLI/LSP/phase authority. |
| Aggregator | `DiagnosticAggregationInput`, `DiagnosticSourceKey`, `ObsoleteDiagnosticDraft`, `BuildDiagnosticIndex`, `DiagnosticAggregationError`, and aggregate/index/source/obsolete/debug accessors. | `aggregator.md`, architecture 19, architecture 20, architecture 22. | `tests/aggregator.rs`, `tests/determinism.rs`, `tests/lint_policy.rs`. | Covered. Ordering, deduplication, snapshot-scoped handles, obsolete-snapshot suppression, and deterministic debug snapshots are traced. |
| Rendering | `DiagnosticSourceContext`, `RenderOptions`, `RenderStyle`, `DiagnosticRenderInput`, `render_diagnostics`, and input/options accessors. | `render.md`, spec 22.1.2. | `tests/render.rs`, `tests/determinism.rs`, `tests/lint_policy.rs`. | Covered. Rendering is a byte-stable projection and does not create LSP payloads or phase outcomes. |
| Fix suggestions | `FixSuggestionId`, `FixCommandRef`, `FixApplicability`, `FixSafety`, `FixEdit`, `FixSuggestionInput`, `FixSuggestion`, `FixSuggestionError`, and constructors/accessors/debug snapshots. | `fix.md`, spec 22.1.3. | `tests/fix.rs`, `tests/failure_record.rs`, `tests/aggregator.rs`, `tests/render.rs`, `tests/determinism.rs`, `tests/lint_policy.rs`. | Covered. Structured payloads, preconditions, dedup identity, rendering projection, and no-auto-apply boundaries are traced. |
| Explanations | Preview constants, `ExplanationHandleId`, `ExplanationKind`, `ExplanationSubject`, `ExplanationSourceRef`, `ExplanationPreviewFormat`, `ExplanationPreview`, `ExplanationHandleInput`, `ExplanationHandle`, `ExplanationPayload`, `ExplanationResolution`, `ExplanationMissingReason`, `ExplanationStore`, `ExplanationError`, and constructors/accessors/debug snapshots. | `explain.md`, internal 03, architecture 12. | `tests/explain.rs`, `tests/failure_record.rs`, `tests/aggregator.rs`, `tests/render.rs`, `tests/determinism.rs`, `tests/lint_policy.rs`. | Covered. Lazy handles, bounded previews, optional backing payloads, stale/missing resolution, attachment validation, and no LSP/driver ownership are traced. |

## Behavior Correspondence

| Promised behavior | Source anchor | Test anchor | Result |
|---|---|---|---|
| Tools and consumers key on `DiagnosticCode`, not message text. | `registry.rs`, `failure_record.rs`, `aggregator.rs`, `render.rs`. | `tests/registry.rs`, `tests/failure_record.rs`, `tests/aggregator.rs`, `tests/render.rs`, `tests/determinism.rs`. | Covered; message text is not lookup, dedup, or render identity. |
| Codes are permanent, retired rather than reused, and descriptor metadata is compatibility-checked. | `registry.rs`. | `tests/registry.rs`. | Covered. |
| Records explicitly carry `SourceId`, `SourceRange`, `BuildSnapshotId`, and freshness state. | `failure_record.rs`. | `tests/failure_record.rs`, `tests/aggregator.rs`, `tests/determinism.rs`. | Covered. |
| Producer sinks collect drafts only and preserve local emission data without formatting or LSP conversion. | `sink.rs`. | `tests/sink.rs`. | Covered. |
| Aggregation is deterministic, order-independent, deduplicated by structured identity, and withholds obsolete snapshots from current publication. | `aggregator.rs`. | `tests/aggregator.rs`, `tests/determinism.rs`. | Covered. |
| CLI rendering is a projection from records/registry metadata and is byte-stable in plain mode. | `render.rs`. | `tests/render.rs`, `tests/determinism.rs`. | Covered. |
| Fix suggestions are structured, ordered, preconditioned, and never applied by this crate. | `fix.rs`, `failure_record.rs`, `aggregator.rs`, `render.rs`. | `tests/fix.rs`, `tests/failure_record.rs`, `tests/aggregator.rs`, `tests/render.rs`. | Covered. |
| Explanation handles are lazy references with bounded previews and fail-closed resolution. | `explain.rs`, `failure_record.rs`, `aggregator.rs`, `render.rs`. | `tests/explain.rs`, `tests/failure_record.rs`, `tests/aggregator.rs`, `tests/render.rs`. | Covered. |
| Public enums remain forward-compatible for downstream matching. | all source modules. | `tests/lint_policy.rs`. | Covered. |
| The crate does not own proof acceptance, trusted status, kernel acceptance, artifact mutation, LSP protocol conversion, or driver session orchestration. | crate root dependency boundary and module implementations. | `tests/lint_policy.rs` plus module-specific boundary tests. | Covered for this crate; real downstream adoption remains deferred. |

## Gap Register

| ID | Class | Observation | Disposition |
|---|---|---|---|
| DIAG-AUDIT-001 | `external_dependency_gap` / `deferred` | Existing lexer/frontend/parser/resolver diagnostics still have no real shared-index consumer adoption seam. | Already recorded by `consumer_adoption_decision.md`; no placeholder adapter, stub API, or fake resolver adoption is added. |
| DIAG-AUDIT-002 | `external_dependency_gap` | `mizar-lsp` and `mizar-driver` adoption remain unavailable: LSP conversion is LSP-owned, and `crates/mizar-driver` is not present in this checkout. | Already recorded by `consumer_adoption_decision.md`; no protocol conversion or driver session API is added. |
| DIAG-AUDIT-003 | `external_dependency_gap` | Artifact publication and durable artifact projection remain artifact-owned. | No artifact mutation or projection authority is added to `mizar-diagnostics`. |
| DIAG-AUDIT-004 | `repo_metadata_conflict` | Preflight found no `doc/design/mizar-artifact/en/crate_exit_report.md` to consult, although the user-requested preflight named an artifact closeout report. | Report only; do not repair backup/stash/history metadata from this crate task. |

No new `spec_gap`, `test_gap`, `source_drift`,
`source_undocumented_behavior`, `test_expectation_drift`, or
`boundary_violation` was found for the crate-owned public surface.

## Documentation Correction

This task also corrects the crate plan's relevant-test table to point at the
actual integration test files under `crates/mizar-diagnostics/tests/`. Earlier
wording referred to source-file unit tests, but the implemented test suite lives
in integration tests. This was `design_drift` and is repaired in this task.

Task 21 scoped rerun updates the source inventory for private helper moves:
`src/registry/builtin.rs`, `src/failure_record/validation.rs`, and
`src/failure_record/debug.rs`. These moved items do not change the public API
trace or behavior correspondence above.

## Audit Result

Task 19 passes for the crate-owned surface: public APIs are represented in the
module specs and exercised by focused Rust tests, or are explicitly documented
as deferred/external. No language-behavior `.miz` test gap is introduced because
this crate does not change syntax, static semantics, proof semantics, type
behavior, name resolution, overload behavior, parser recovery, or user-visible
language outcomes.
