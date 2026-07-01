# Source/Spec Correspondence Audit: mizar-diagnostics

> Canonical language: English. English source:
> [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md).

## Scope

元の task-19 audit は task 18 の後、bilingual sync gate と module-boundary gate の
前に実施した。Task 21 は private module split 後に source-inventory scope を再実行し、
移動した private helpers についてだけこの文書を更新する。この audit は crate-owned
public API と module specification が約束する behavior を source と tests に照合する。

この audit は producer adapter、driver event、LSP protocol shape、artifact projection、
resolver diagnostic code、または既存 lexer/frontend/parser/resolver diagnostics の
migration wiring を追加しない。

## Method

- Source inventory: `crates/mizar-diagnostics/src/{registry,failure_record,sink,aggregator,render,fix,explain}.rs`、
  private `src/registry/builtin.rs`、
  private `src/failure_record/{validation,debug}.rs`、crate root。
- Specification inventory:
  `registry.md`、`failure_record.md`、`sink.md`、`aggregator.md`、`render.md`、
  `fix.md`、`explain.md`、`consumer_adoption_decision.md`、crate plan、`todo.md`。
- Test inventory:
  `tests/{registry,failure_record,sink,aggregator,render,fix,explain,determinism,lint_policy}.rs`。
- gap vocabulary は autonomous-development protocol に従う:
  `spec_gap`、`test_gap`、`design_drift`、`source_drift`、
  `source_undocumented_behavior`、`test_expectation_drift`、
  `boundary_violation`、`repo_metadata_conflict`、`external_dependency_gap`、`deferred`。

## Public API Trace

| Area | Public API surface | Spec coverage | Test coverage | Audit result |
|---|---|---|---|---|
| Crate root | public modules `aggregator`、`explain`、`failure_record`、`fix`、`registry`、`render`、`sink`; public macro なし。 | `00.crate_plan.md`、`todo.md`、module specs。 | `tests/lint_policy.rs`。 | Covered。root は crate-owned diagnostics modules だけを公開する。 |
| Registry | `DiagnosticSeverity`、`PhaseFamily`、`DiagnosticCode`、`DiagnosticCodeError`、`DiagnosticStatus`、`DiagnosticDescriptor`、`BUILTIN_DESCRIPTORS`、`DiagnosticRegistry`、`validate_descriptors`、`validate_registry_compatibility`、`RegistryValidationError`、および constructors/accessors/lookup helpers。 | `registry.md`、spec 22.1.1、spec 22.7。 | `tests/registry.rs`、`tests/lint_policy.rs`。 | Covered。code identity、allocation、retirement、compatibility、lookup、alias、public enum compatibility を trace した。 |
| Failure records | `PipelinePhase`、`FailureCategory`、`DiagnosticId`、`DiagnosticHandle`、freshness/span/note/detail enums、`DiagnosticSpan`、`DiagnosticNote`、`DiagnosticDetails`、`DiagnosticDetailValue`、`DiagnosticDraftInput`、`DiagnosticDraft`、`DiagnosticRecord`、`DiagnosticRecordError`、`is_valid_detail_key`、および constructors/accessors/debug snapshots。 | `failure_record.md`、architecture 19、spec 22.1.2。 | `tests/failure_record.rs`、`tests/aggregator.rs`、`tests/render.rs`、`tests/determinism.rs`、`tests/lint_policy.rs`。 | Covered。draft/record construction、descriptor metadata、source ranges、freshness、structured details、attachment slots、debug snapshots を trace した。 |
| Producer sink | `DiagnosticProducerScope`、`DiagnosticSink`、`DiagnosticBatch`、`DiagnosticSinkError`、および scope/emit/seal/batch/debug accessors。 | `sink.md`、internal 03。 | `tests/sink.rs`、`tests/lint_policy.rs`。 | Covered。sink は drafts だけを collect し、CLI/LSP/phase authority を持たない。 |
| Aggregator | `DiagnosticAggregationInput`、`DiagnosticSourceKey`、`ObsoleteDiagnosticDraft`、`BuildDiagnosticIndex`、`DiagnosticAggregationError`、および aggregate/index/source/obsolete/debug accessors。 | `aggregator.md`、architecture 19、architecture 20、architecture 22。 | `tests/aggregator.rs`、`tests/determinism.rs`、`tests/lint_policy.rs`。 | Covered。ordering、deduplication、snapshot-scoped handles、obsolete-snapshot suppression、deterministic debug snapshots を trace した。 |
| Rendering | `DiagnosticSourceContext`、`RenderOptions`、`RenderStyle`、`DiagnosticRenderInput`、`render_diagnostics`、および input/options accessors。 | `render.md`、spec 22.1.2。 | `tests/render.rs`、`tests/determinism.rs`、`tests/lint_policy.rs`。 | Covered。rendering は byte-stable projection であり、LSP payload や phase outcome を作らない。 |
| Fix suggestions | `FixSuggestionId`、`FixCommandRef`、`FixApplicability`、`FixSafety`、`FixEdit`、`FixSuggestionInput`、`FixSuggestion`、`FixSuggestionError`、および constructors/accessors/debug snapshots。 | `fix.md`、spec 22.1.3。 | `tests/fix.rs`、`tests/failure_record.rs`、`tests/aggregator.rs`、`tests/render.rs`、`tests/determinism.rs`、`tests/lint_policy.rs`。 | Covered。structured payloads、preconditions、dedup identity、rendering projection、no-auto-apply boundary を trace した。 |
| Explanations | preview constants、`ExplanationHandleId`、`ExplanationKind`、`ExplanationSubject`、`ExplanationSourceRef`、`ExplanationPreviewFormat`、`ExplanationPreview`、`ExplanationHandleInput`、`ExplanationHandle`、`ExplanationPayload`、`ExplanationResolution`、`ExplanationMissingReason`、`ExplanationStore`、`ExplanationError`、および constructors/accessors/debug snapshots。 | `explain.md`、internal 03、architecture 12。 | `tests/explain.rs`、`tests/failure_record.rs`、`tests/aggregator.rs`、`tests/render.rs`、`tests/determinism.rs`、`tests/lint_policy.rs`。 | Covered。lazy handles、bounded previews、optional backing payloads、stale/missing resolution、attachment validation、no LSP/driver ownership を trace した。 |

## Behavior Correspondence

| Promised behavior | Source anchor | Test anchor | Result |
|---|---|---|---|
| tools/consumers は message text ではなく `DiagnosticCode` を key にする。 | `registry.rs`、`failure_record.rs`、`aggregator.rs`、`render.rs`。 | `tests/registry.rs`、`tests/failure_record.rs`、`tests/aggregator.rs`、`tests/render.rs`、`tests/determinism.rs`。 | Covered。message text は lookup、dedup、render identity ではない。 |
| code は permanent であり、reuse せず retired され、descriptor metadata は compatibility-check される。 | `registry.rs`。 | `tests/registry.rs`。 | Covered。 |
| records は `SourceId`、`SourceRange`、`BuildSnapshotId`、freshness state を明示的に持つ。 | `failure_record.rs`。 | `tests/failure_record.rs`、`tests/aggregator.rs`、`tests/determinism.rs`。 | Covered。 |
| producer sink は drafts だけを collect し、formatting や LSP conversion をしない。 | `sink.rs`。 | `tests/sink.rs`。 | Covered。 |
| aggregation は deterministic、order-independent、structured identity で deduplicate し、obsolete snapshot を current publication から除外する。 | `aggregator.rs`。 | `tests/aggregator.rs`、`tests/determinism.rs`。 | Covered。 |
| CLI rendering は records/registry metadata からの projection であり、plain mode は byte-stable。 | `render.rs`。 | `tests/render.rs`、`tests/determinism.rs`。 | Covered。 |
| fix suggestion は structured、ordered、preconditioned であり、この crate は apply しない。 | `fix.rs`、`failure_record.rs`、`aggregator.rs`、`render.rs`。 | `tests/fix.rs`、`tests/failure_record.rs`、`tests/aggregator.rs`、`tests/render.rs`。 | Covered。 |
| explanation handle は bounded preview を持つ lazy reference であり、resolution は fail-closed。 | `explain.rs`、`failure_record.rs`、`aggregator.rs`、`render.rs`。 | `tests/explain.rs`、`tests/failure_record.rs`、`tests/aggregator.rs`、`tests/render.rs`。 | Covered。 |
| public enum は downstream matching のため forward-compatible である。 | all source modules。 | `tests/lint_policy.rs`。 | Covered。 |
| この crate は proof acceptance、trusted status、kernel acceptance、artifact mutation、LSP protocol conversion、driver session orchestration を所有しない。 | crate root dependency boundary と module implementations。 | `tests/lint_policy.rs` と module-specific boundary tests。 | この crate について Covered。real downstream adoption は deferred のまま。 |

## Gap Register

| ID | Class | Observation | Disposition |
|---|---|---|---|
| DIAG-AUDIT-001 | `external_dependency_gap` / `deferred` | 既存 lexer/frontend/parser/resolver diagnostics には、real shared-index consumer adoption seam がまだない。 | `consumer_adoption_decision.md` で記録済み。placeholder adapter、stub API、fake resolver adoption は追加しない。 |
| DIAG-AUDIT-002 | `external_dependency_gap` | `mizar-lsp` adoption は LSP-owned のままであり、後続の `mizar-driver` scaffold はまだ real driver sessions、events、publication orchestration を提供しない。 | `consumer_adoption_decision.md` で記録済み。diagnostics は protocol conversion や driver session API を追加しない。 |
| DIAG-AUDIT-003 | `external_dependency_gap` | artifact publication と durable artifact projection は artifact-owned のままである。 | `mizar-diagnostics` に artifact mutation または projection authority を追加しない。 |
| DIAG-AUDIT-004 | `repo_metadata_conflict` | preflight では、user-requested preflight が artifact closeout report を挙げていたにもかかわらず、`doc/design/mizar-artifact/en/crate_exit_report.md` を参照できなかった。 | report only。この crate task では backup/stash/history metadata を修復しない。 |

crate-owned public surface について新しい `spec_gap`、`test_gap`、`source_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation` は
見つからなかった。

## Documentation Correction

この task は crate plan の relevant-test table も修正し、実際の integration test files
である `crates/mizar-diagnostics/tests/` 以下を指すようにした。以前の文言は source-file
unit tests を指していたが、実装済み test suite は integration tests である。これは
`design_drift` であり、この task で修復した。

Task 21 scoped rerun は private helper moves に合わせて source inventory を更新する:
`src/registry/builtin.rs`、`src/failure_record/validation.rs`、
`src/failure_record/debug.rs`。これらの moved items は上記の public API trace や
behavior correspondence を変更しない。

## Audit Result

Task 19 は crate-owned surface について pass とする。public APIs は module specs に
表現され、focused Rust tests で exercise されているか、明示的に deferred/external と
文書化されている。syntax、static semantics、proof semantics、type behavior、name
resolution、overload behavior、parser recovery、user-visible language outcomes をこの
crate が変更しないため、language-behavior `.miz` test gap は発生しない。
