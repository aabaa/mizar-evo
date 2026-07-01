# Crate Exit Report: mizar-diagnostics

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the `mizar-diagnostics` internal diagnostics milestone.
Quality score: 95/100.
Score caps applied: none.

## Scope

Milestone scope:

- build the `mizar-diagnostics` workspace crate from task 0 through task 21 and
  this closeout task;
- own permanent `DiagnosticCode` identity, built-in code allocation,
  retirement, compatibility validation, and registry metadata;
- own structured `DiagnosticDraft` and immutable `DiagnosticRecord` values
  with `SourceId`, `SourceRange`, snapshot freshness, failure categories,
  structured details, fix attachments, and lazy explanation handles;
- own producer-side draft collection through `DiagnosticSink` and sealed
  batches, without formatting, protocol conversion, or phase-status authority;
- own deterministic aggregation into immutable `BuildDiagnosticIndex` values,
  including normalization, deduplication, stable ordering, dense snapshot-local
  ids, obsolete-snapshot accounting, and current-publication suppression for
  stale snapshots;
- own deterministic CLI rendering, structured fix-suggestion payloads, and lazy
  explanation handle resolution as projections from records.

Excluded:

- syntax, static semantics, proof semantics, type behavior, name resolution,
  overload behavior, parser recovery, proof acceptance, trusted status, kernel
  acceptance, verifier policy, ATP search, cache reuse authority, artifact
  mutation, driver session orchestration, LSP protocol conversion, or open
  buffer publication;
- migration of existing lexer/frontend/parser/resolver diagnostics before a
  real adoption seam and public code-allocation decision exist;
- placeholder adapters, stub resolver APIs, fake driver events, provisional LSP
  bridges, invented artifact publication hooks, or provisional consumer
  dependencies.

## Task Commits

| Task | Commit | Subject |
|---:|---|---|
| 0 | `a9a2c55f651890b6872d7a2084c509f8ecc3745b` | `docs(diagnostics-task-0): add autonomous crate plan` |
| 1 | `ef2cc113524f9480f9789ceac326b2c3926b0d60` | `feat(diagnostics-task-1): scaffold diagnostics crate` |
| 2 | `dd49535e0979bbd71a32c57579983189c1b04d41` | `docs(diagnostics-task-2): specify diagnostic registry` |
| 3 | `ea83691ca19dc5e0f49d5088f2b0e95b46bac250` | `feat(diagnostics-task-3): implement diagnostic registry` |
| 4 | `4d0eb687447284d8e8ebbb46123b0d39c0c82298` | `docs(diagnostics-task-4): specify failure records` |
| 5 | `6cdf1b91b4ad7d2e01e9e0d43c9badf5acb65f31` | `feat(diagnostics-task-5): implement failure records` |
| 6 | `ee13daea4a9352783b27b521074320039870af3d` | `docs(diagnostics-task-6): specify diagnostic sink` |
| 7 | `211ab08c89fc4cd3b9b60e6fdb5753ef2624a9d9` | `feat(diagnostics-task-7): implement diagnostic sink` |
| 8 | `67d49a625c74e54d370e83ba63afa9ee8635bc62` | `docs(diagnostics-task-8): specify diagnostic aggregator` |
| 9 | `987412579d67564f783e652eeba15bbe33d12241` | `feat(diagnostics-task-9): implement diagnostic aggregation` |
| 10 | `f1e10df23f4715f6a91775b4e47e67a89972b612` | `docs(diagnostics-task-10): specify CLI rendering` |
| 11 | `cbcb2e263680fce0bc3459c9c81b6c13a1898fb0` | `feat(diagnostics-task-11): implement CLI rendering` |
| 12 | `6ec14bc067ecc66b961bfbb47acf8ff67131dcbc` | `docs(diagnostics-task-12): specify fix suggestions` |
| 13 | `939edef546f77b089c1bd5ed174c96f4f3d735e3` | `feat(diagnostics-task-13): implement fix suggestions` |
| 14 | `bf8c0fd8dbb0e89c3a2c6c1fcfdd67407aac91d4` | `docs(diagnostics-task-14): specify explanations` |
| 15 | `169d586f0aab7ebce27eeff4525a3c15f9d3ebf6` | `feat(diagnostics-task-15): implement explanations` |
| 16 | `1046ae22ef1e24d587fe1bcca4f4d6488adf6b8b` | `docs(diagnostics-task-16): defer consumer adoption` |
| 17 | `a9d530a64a77c0aa32e86aa1d72cd59b67d8b16d` | `test(diagnostics-task-17): add determinism suite` |
| 18 | `f8e4d9b4fe144a8d76212d6677abcdf5ab09d804` | `test(diagnostics-task-18): guard enum compatibility` |
| 19 | `183112477ec6dc90a3cf78ee7cf42c4bcc88dc76` | `docs(diagnostics-task-19): audit source spec correspondence` |
| 20 | `abea82a684e484060d11cf3c3770e875cf8e7fd1` | `docs(diagnostics-task-20): audit bilingual docs` |
| 21 | `5fb55e65e5f5de2b810de0c5b19f4e46313e28fd` | `refactor(diagnostics-task-21): split private module helpers` |
| 22 | pending self-hash | `docs(diagnostics-closeout): add diagnostics exit report` |

## Final Owned Surfaces

| Surface | Final shape |
|---|---|
| Registry | `DiagnosticCode` is the stable identity. `DiagnosticRegistry` validates allocation, retirement finality, metadata compatibility, alias compatibility, and built-in descriptor lockstep. Message text, localized text, rendering text, sort order, and `DiagnosticId` are never identity. |
| Failure records | `DiagnosticDraft` and `DiagnosticRecord` carry code metadata, `SourceId`, `SourceRange`, snapshot freshness, failure category, structured notes/details, fix references, explanation handles, and deterministic debug snapshots. Current records reject stale/obsolete freshness and retired codes. |
| Sink | `DiagnosticSink` collects validated drafts for one producer scope, preserves local draft payloads until sealing, rejects phase/snapshot mismatches without mutation, and emits sealed `DiagnosticBatch` values only. It owns no CLI formatting, LSP shape, or phase semantics. |
| Aggregator | Aggregation consumes sealed batches, filters obsolete snapshots out of current publication, deduplicates by structured identity rather than messages, chooses representatives deterministically, assigns dense snapshot-local handles, and builds immutable `BuildDiagnosticIndex` values. |
| CLI render | Rendering is a deterministic projection from records, registry metadata, and caller-supplied source context. It formats headers, spans, notes, fix previews, and explanation previews without creating diagnostic identity or owning source loading/LSP publication. |
| Fix suggestions | Structured fix payloads contain stable ids, range-validated edits, applicability, safety, preconditions, commands, and deterministic debug snapshots. They are advisory only and are never auto-applied or converted into LSP code actions by this crate. |
| Explanations | Lazy explanation handles carry compact structured references, snapshot/source/hash preconditions, bounded previews, fail-closed resolution status, and deterministic identities for aggregation. Large traces, proof data, artifact mutation, and LSP request shaping remain outside the crate. |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | The crate plan, module specs, TODO, source/spec audit, bilingual audit, module-boundary gate, and closeout reviews report no unresolved blocking/high inconsistency. |
| Source behavior documented or deferred | passed | Public APIs, private helper moves, tests, and residual gaps are traced in `source_spec_correspondence.md`, `bilingual_documentation_sync.md`, `module_boundary_refactor_gate.md`, and this report. |
| Stable diagnostic identity | passed | Registry and aggregation specs/tests require tools to key on `DiagnosticCode`, never message text, render text, localization, or production order. Codes are retired rather than reused. |
| Freshness and deterministic publication | passed | Records carry explicit snapshot freshness; aggregation suppresses obsolete snapshots from current publication and produces production-order-independent indexes. |
| Producer boundary | passed | Sinks collect drafts only. Producers do not format CLI output, build LSP records, choose phase status, mutate artifacts, or orchestrate driver sessions through this crate. |
| Projection boundary | passed | Render, fix, and explain modules project from records and do not own proof acceptance, trusted status, kernel acceptance, cache reuse authority, LSP conversion, artifact publication, or driver orchestration. |
| Test expectation integrity | passed | No language specification, `.miz` fixture, traceability row, or expectation sidecar was changed to match implementation behavior. |
| Design/source synchronization | passed | English canonical docs and Japanese companions are paired and synchronized, and source/docs audits record no current drift. |
| Downstream gaps classified | passed | Resolver/LSP/driver/artifact and legacy diagnostic migration gaps are classified as `external_dependency_gap` or `deferred`; no placeholder adapter or stub API was added. |
| Verification | passed | Crate-local fmt/test/clippy, full workspace fmt/clippy/test, diff checks, staged diff checks, and closeout reviews passed. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 19/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 3/5 |
| Total | 95/100 |

The score deducts for intentionally unfinished downstream adoption: real
resolver diagnostic-code migration, LSP publication, driver sessions, artifact
projection/publication integration, and legacy frontend-family migration. These
do not cap the score because they are classified gaps, not stubbed behavior,
and all hard gates pass. The missing `mizar-artifact` closeout report remains a
reported `repo_metadata_conflict` and does not change this crate's implemented
surface.

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | No findings. Closeout scope, task commits, hard gates, owned/excluded boundaries, gap classifications, and handoff are complete and consistent with the crate plan and module specs. |
| Test sufficiency review | No findings. Crate-local tests cover registry compatibility, record validation, sink behavior, deterministic aggregation, stale-snapshot suppression, CLI rendering, structured fixes, lazy explanations, determinism, and public enum compatibility. Downstream adoption tests remain correctly classified as external until real seams exist. |
| Full implementation review | No findings. The closeout report matches the implemented source boundary; no production source change is part of closeout. |
| Source/documentation consistency review | No findings. English and Japanese closeout reports are synchronized, task hashes and subjects match git history, and source/documentation ownership statements agree. |
| Read-only crate quality review | Valid quality score: 95/100. No score cap applies; all hard gates pass. |

## Deferred And External Dependency Items

| ID | Class / disposition | Owner / unblock condition |
|---|---|---|
| DIAG-G-003 | `spec_gap`, disposition `external_dependency_gap`/`deferred` | Resolver public diagnostic-code allocation and real resolver consumer adoption must be decided by the resolver/integration phase. This crate did not invent resolver codes or adapters. |
| DIAG-G-004 | `design_drift`, disposition `external_dependency_gap` | `mizar-lsp` owns protocol conversion, document-version handling, open-buffer publication, and LSP explanation requests. This crate exposes records/indexes only. |
| DIAG-G-005 | `design_drift`, disposition `external_dependency_gap` | `mizar-driver` is absent, so real driver sessions, events, and publication orchestration remain outside this crate. No placeholder dependency or event API was added. |
| DIAG-G-006 | `source_drift`, disposition `external_dependency_gap`/`deferred` | Existing lexer/frontend/parser diagnostics remain owning-crate local until a real migration seam and consumer tests exist. |
| DIAG-G-007 | `design_drift`, disposition `external_dependency_gap` | `mizar-artifact` owns artifact mutation and publication. Diagnostics may be projected by future artifact owners, but this crate does not mutate manifests or mint publication authority. |
| DIAG-G-008 | `repo_metadata_conflict`, report only | The requested `mizar-artifact` closeout report is absent in this checkout. This diagnostics stream reports the conflict and does not repair artifact metadata. |
| DIAG-G-009 | `boundary_violation`, guarded constraint | Moving proof acceptance, trusted status, kernel acceptance, cache reuse authority, driver orchestration, LSP conversion, or artifact mutation into this crate would violate the boundary. Specs, source shape, and lint/review gates guard against it. |

## Test Expectation Summary

No language specification, `.miz` test, coverage traceability metadata, or
expectation sidecar was changed for the `mizar-diagnostics` milestone.
Crate-owned behavior is covered by Rust integration tests, lint-policy guards,
determinism tests, source/spec audits, bilingual audits, the module-boundary
gate, and explicit gap records.

## Verification Commands

| Command | Result |
|---|---|
| `cargo test -p mizar-diagnostics` | passed |
| `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings` | passed |
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | passed |

Consumer-boundary commands for `mizar-resolve`, `mizar-lsp`, and `mizar-build`
were not required for closeout because this task does not change an implemented
consumer seam. `mizar-driver` is absent and remains an `external_dependency_gap`.

## Human Review Surface

Primary human review should inspect:

- [00.crate_plan.md](./00.crate_plan.md)
- [todo.md](./todo.md)
- [registry.md](./registry.md)
- [failure_record.md](./failure_record.md)
- [sink.md](./sink.md)
- [aggregator.md](./aggregator.md)
- [render.md](./render.md)
- [fix.md](./fix.md)
- [explain.md](./explain.md)
- [consumer_adoption_decision.md](./consumer_adoption_decision.md)
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_sync.md](./bilingual_documentation_sync.md)
- [module_boundary_refactor_gate.md](./module_boundary_refactor_gate.md)
- `crates/mizar-diagnostics/src/`
- `crates/mizar-diagnostics/tests/`

## Next-Phase Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Start the next integration phase after the mizar-diagnostics closeout commit
exists. Keep `mizar-diagnostics` as the owner of stable `DiagnosticCode`
identity, registry compatibility, structured diagnostic records, producer draft
sinks, deterministic build diagnostic aggregation, CLI rendering, structured
fix suggestions, and lazy explanation handles. Tools and consumers must key on
`DiagnosticCode`, not message text, render text, localized text, production
order, or snapshot-local diagnostic ids.

The best next task is a real consumer adoption phase owned by the consuming
crate: resolver diagnostic-code allocation and migration, LSP publication and
protocol conversion, driver session publication, or artifact projection. Do not
add placeholder adapters, stub APIs, fake resolver adoption, provisional LSP
bridges, fake driver events, or artifact mutation authority in
`mizar-diagnostics`.

Preserve the boundary that `mizar-diagnostics` does not own phase semantics,
proof acceptance, trusted status, kernel acceptance, cache reuse authority,
artifact publication, LSP protocol conversion, or driver session orchestration.
Obsolete snapshot diagnostics must not be published as current diagnostics.
```

Raise reasoning above `xhigh` only for simultaneous resolver/LSP/driver/artifact
integration with broad API migration. Lower to `high` for a narrow docs-only
follow-up, such as backfilling a committed closeout hash or updating one paired
documentation row.
