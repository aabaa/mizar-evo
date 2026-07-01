# Crate Exit Report: mizar-diagnostics

> Canonical language: English. English source:
> [../en/crate_exit_report.md](../en/crate_exit_report.md).

## Result

Status: `mizar-diagnostics` internal diagnostics milestone は complete。
Quality score: 95/100。
Score caps applied: none。

## Scope

Milestone scope:

- task 0 から task 21 とこの closeout task までで `mizar-diagnostics`
  workspace crate を構築する。
- permanent `DiagnosticCode` identity、built-in code allocation、retirement、
  compatibility validation、registry metadata を所有する。
- `SourceId`、`SourceRange`、snapshot freshness、failure categories、
  structured details、fix attachments、lazy explanation handles を持つ
  structured `DiagnosticDraft` と immutable `DiagnosticRecord` を所有する。
- formatting、protocol conversion、phase-status authority を持たず、
  `DiagnosticSink` と sealed batches による producer-side draft collection を
  所有する。
- normalization、deduplication、stable ordering、dense snapshot-local ids、
  obsolete-snapshot accounting、stale snapshot の current-publication
  suppression を含む immutable `BuildDiagnosticIndex` への deterministic
  aggregation を所有する。
- deterministic CLI rendering、structured fix-suggestion payload、lazy
  explanation handle resolution を records からの projection として所有する。

Excluded:

- syntax、static semantics、proof semantics、type behavior、name resolution、
  overload behavior、parser recovery、proof acceptance、trusted status、kernel
  acceptance、verifier policy、ATP search、cache reuse authority、artifact
  mutation、driver session orchestration、LSP protocol conversion、open buffer
  publication。
- real adoption seam と public code-allocation decision が存在する前の既存
  lexer/frontend/parser/resolver diagnostics の migration。
- placeholder adapters、stub resolver APIs、fake driver events、provisional LSP
  bridges、invented artifact publication hooks、provisional consumer
  dependencies。

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
| Registry | `DiagnosticCode` が stable identity である。`DiagnosticRegistry` は allocation、retirement finality、metadata compatibility、alias compatibility、built-in descriptor lockstep を validate する。message text、localized text、rendering text、sort order、`DiagnosticId` は identity ではない。 |
| Failure records | `DiagnosticDraft` と `DiagnosticRecord` は code metadata、`SourceId`、`SourceRange`、snapshot freshness、failure category、structured notes/details、fix references、explanation handles、deterministic debug snapshots を保持する。current records は stale/obsolete freshness と retired codes を拒否する。 |
| Sink | `DiagnosticSink` は単一 producer scope の validated drafts を収集し、sealing まで local draft payloads を保持し、phase/snapshot mismatch を mutation なしで拒否し、sealed `DiagnosticBatch` のみを出す。CLI formatting、LSP shape、phase semantics は所有しない。 |
| Aggregator | aggregation は sealed batches を消費し、obsolete snapshots を current publication から除外し、message ではなく structured identity で deduplicate し、representative を決定的に選び、dense snapshot-local handles を割り当て、immutable `BuildDiagnosticIndex` を構築する。 |
| CLI render | rendering は records、registry metadata、caller-supplied source context からの deterministic projection である。headers、spans、notes、fix previews、explanation previews を format するが、diagnostic identity、source loading、LSP publication は所有しない。 |
| Fix suggestions | structured fix payload は stable ids、range-validated edits、applicability、safety、preconditions、commands、deterministic debug snapshots を含む。advisory only であり、この crate は auto-apply も LSP code action conversion もしない。 |
| Explanations | lazy explanation handles は compact structured references、snapshot/source/hash preconditions、bounded previews、fail-closed resolution status、aggregation 用 deterministic identities を持つ。large traces、proof data、artifact mutation、LSP request shaping は crate 外である。 |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | crate plan、module specs、TODO、source/spec audit、bilingual audit、module-boundary gate、closeout reviews に unresolved blocking/high inconsistency はない。 |
| Source behavior documented or deferred | passed | public APIs、private helper moves、tests、residual gaps は `source_spec_correspondence.md`、`bilingual_documentation_sync.md`、`module_boundary_refactor_gate.md`、本 report に trace される。 |
| Stable diagnostic identity | passed | registry と aggregation の specs/tests は tools が `DiagnosticCode` を key にし、message text、render text、localization、production order を key にしないことを要求する。codes は reuse ではなく retire される。 |
| Freshness and deterministic publication | passed | records は explicit snapshot freshness を持ち、aggregation は obsolete snapshots を current publication から抑止し、production-order-independent indexes を生成する。 |
| Producer boundary | passed | sink は drafts のみを収集する。producers はこの crate を通じて CLI output、LSP records、phase status、artifact mutation、driver sessions を所有しない。 |
| Projection boundary | passed | render、fix、explain は records から project し、proof acceptance、trusted status、kernel acceptance、cache reuse authority、LSP conversion、artifact publication、driver orchestration を所有しない。 |
| Test expectation integrity | passed | language specification、`.miz` fixture、traceability row、expectation sidecar を実装挙動へ合わせて変更していない。 |
| Design/source synchronization | passed | English canonical docs と Japanese companions は paired/synchronized で、source/docs audits は current drift なしを記録する。 |
| Downstream gaps classified | passed | resolver/LSP/driver/artifact と legacy diagnostic migration gaps は `external_dependency_gap` または `deferred` として分類済みで、placeholder adapter や stub API は追加していない。 |
| Verification | passed | crate-local fmt/test/clippy、full workspace fmt/clippy/test、diff checks、staged diff checks、closeout reviews は passed。 |

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

score は、意図的に未完了の downstream adoption、つまり real resolver diagnostic-code
migration、LSP publication、driver sessions、artifact projection/publication
integration、legacy frontend-family migration のために差し引く。これらは classified
gaps であって stubbed behavior ではなく、すべての hard gates は pass しているため
score cap にはならない。missing `mizar-artifact` closeout report は
`repo_metadata_conflict` として report-only で残り、この crate の implemented surface は
変えない。

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | No findings. closeout scope、task commits、hard gates、owned/excluded boundaries、gap classifications、handoff は crate plan と module specs に整合する。 |
| Test sufficiency review | No findings. crate-local tests は registry compatibility、record validation、sink behavior、deterministic aggregation、stale-snapshot suppression、CLI rendering、structured fixes、lazy explanations、determinism、public enum compatibility を cover する。downstream adoption tests は real seams が存在するまで external として正しく分類されている。 |
| Full implementation review | No findings. closeout report は implemented source boundary と一致し、closeout に production source change は含まれない。 |
| Source/documentation consistency review | No findings. English/Japanese closeout reports は同期され、task hashes と subjects は git history と一致し、source/documentation ownership statements は一致する。 |
| Read-only crate quality review | Valid quality score: 95/100. score cap はなく、すべての hard gates は pass する。 |

## Deferred And External Dependency Items

| ID | Class / disposition | Owner / unblock condition |
|---|---|---|
| DIAG-G-003 | `spec_gap`, disposition `external_dependency_gap`/`deferred` | resolver public diagnostic-code allocation と real resolver consumer adoption は resolver/integration phase が決定する。この crate は resolver codes や adapters を発明していない。 |
| DIAG-G-004 | `design_drift`, disposition `external_dependency_gap` | `mizar-lsp` は protocol conversion、document-version handling、open-buffer publication、LSP explanation requests を所有する。この crate は records/indexes のみを expose する。 |
| DIAG-G-005 | `design_drift`, disposition `external_dependency_gap` | `mizar-driver` は存在しないため、real driver sessions、events、publication orchestration は crate 外に残る。placeholder dependency/event API は追加していない。 |
| DIAG-G-006 | `source_drift`, disposition `external_dependency_gap`/`deferred` | 既存 lexer/frontend/parser diagnostics は real migration seam と consumer tests が存在するまで owning-crate local のままである。 |
| DIAG-G-007 | `design_drift`, disposition `external_dependency_gap` | `mizar-artifact` は artifact mutation と publication を所有する。diagnostics は将来 artifact owner により project され得るが、この crate は manifests を mutate せず publication authority を mint しない。 |
| DIAG-G-008 | `repo_metadata_conflict`, report only | 要求された `mizar-artifact` closeout report はこの checkout に存在しない。この diagnostics stream は conflict を report し、artifact metadata を修復しない。 |
| DIAG-G-009 | `boundary_violation`, guarded constraint | proof acceptance、trusted status、kernel acceptance、cache reuse authority、driver orchestration、LSP conversion、artifact mutation をこの crate へ移すことは boundary violation である。specs、source shape、lint/review gates がこれを guard する。 |

## Test Expectation Summary

`mizar-diagnostics` milestone では language specification、`.miz` test、
coverage traceability metadata、expectation sidecar を変更していない。crate-owned
behavior は Rust integration tests、lint-policy guards、determinism tests、
source/spec audits、bilingual audits、module-boundary gate、explicit gap records で
cover される。

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

`mizar-resolve`、`mizar-lsp`、`mizar-build` の consumer-boundary commands は、
この closeout task が implemented consumer seam を変更しないため不要である。
`mizar-driver` は存在せず、`external_dependency_gap` のままである。

## Human Review Surface

Primary human review は次を確認する:

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

Recommended reasoning: `xhigh`。

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

resolver/LSP/driver/artifact integration と broad API migration を同時に扱う場合のみ
reasoning を `xhigh` より上げる。committed closeout hash の backfill や paired
documentation row の更新だけなら `high` へ下げてもよい。
