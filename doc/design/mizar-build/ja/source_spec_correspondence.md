# mizar-build source/spec 対応監査

> 正本は英語です。英語版:
> [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md)。

## 範囲

task 22 は task 21 後の `mizar-build` を監査する。crate plan、TODO、module specs
で約束された public API family と挙動を source と tests へ trace する。

監査対象の設計入力は [00.crate_plan.md](./00.crate_plan.md)、
[todo.md](./todo.md)、[planner.md](./planner.md)、
[module_index.md](./module_index.md)、[task_graph.md](./task_graph.md)、
[scheduler.md](./scheduler.md)、[resource.md](./resource.md)、
[cancel.md](./cancel.md)、[failure_state.md](./failure_state.md)、
[artifact_commit.md](./artifact_commit.md)、[cache_seam.md](./cache_seam.md)、
[batch_integration.md](./batch_integration.md)、および
[determinism_suite.md](./determinism_suite.md) である。

分類結果:

- `spec_gap`: 実装済み `mizar-build` behavior には見つからない。
- `test_gap`: `sorted_manifest_updates` の direct standalone coverage について
  BUILD-G-016 を開いた。commit-order behavior は `commit_manifest_updates` を通じて
  すでに covered である。
- `design_drift`: 実装済み `mizar-build` behavior には見つからない。
- `source_drift`: 実装済み `mizar-build` behavior には見つからない。
- `source_undocumented_behavior`: 実装済み `mizar-build` behavior には見つからない。
- `test_expectation_drift`: 実装済み `mizar-build` behavior には見つからない。
- `boundary_violation`: 見つからない。
- `repo_metadata_conflict`: 見つからない。
- `external_dependency_gap`: 既存の driver、IR、producer-token、full real
  clean/incremental integration gaps は下記のとおり残る。

## public API 対応

| Spec | 確認した public API | Source | Test evidence | Finding |
|---|---|---|---|---|
| [00.crate_plan.md](./00.crate_plan.md), [todo.md](./todo.md) | public modules `planner`, `module_index`, `task_graph`, `scheduler`, `resource`, `cancel`, `failure_state`, `artifact_commit`, `cache_seam` | `crates/mizar-build/src/lib.rs` | `tests/lint_policy.rs` が workspace lint opt-in と public enum policy を守る。crate tests は各 public module を実行する。 | No finding. |
| [planner.md](./planner.md) | manifest、lockfile、dependency graph、package plan、config、version constraint、diagnostic、validation、`BuildPlan` APIs; `parse_package_manifest`, `parse_workspace_manifest`, `parse_lockfile`, `validate_package_manifest`, `validate_package_id_spelling`, `validate_lockfile_for_workspace`, `produce_build_plan`, `is_lowercase_snake_case_package_id` | `crates/mizar-build/src/planner.rs` | planner unit tests が valid/invalid package/workspace/lockfile、deterministic diagnostic ordering、package-id spelling、lockfile consistency、dependency graph cycles、version conflicts、dev-dependency selection、unsupported editions、shuffled-input `BuildPlan` equality を覆う。 | No finding. |
| [module_index.md](./module_index.md) | `ModuleIndex`、package/namespace/module/dependency-summary entries、source layout provider、build-side provider traits、diagnostics、provider errors、`build_module_index` | `crates/mizar-build/src/module_index.rs`; downstream wildcard consumption は `crates/mizar-resolve/src/module_index.rs` | module-index unit tests が multi-package workspaces、dependency summaries、alias-independent module identity、deterministic source discovery、duplicate/conflict diagnostics、provider lookup、provider errors、dependency artifact validation を覆う。`cargo test -p mizar-resolve` が downstream provider seam を検証する。 | No finding. |
| [task_graph.md](./task_graph.md) | `TaskGraphVersion`, `TaskGraphInput`, `TaskGraph`, `BuildTask`, `TaskId`, task kinds, phases, work units, dependency coverage, resource/priority classes, module dependency overlays, VC/backend/evidence IDs, diagnostics, `build_task_graph` | `crates/mizar-build/src/task_graph.rs` | task-graph unit tests が deterministic IDs、package/module ordering、phase expansion、dependency-summary inputs、package/module dependency edges、coverage diagnostics、explicit VC descriptors、duplicate/cycle rejection、placeholder absence、non-authority boundaries を覆う。 | No finding. |
| [scheduler.md](./scheduler.md) | `SchedulerInput`, `SchedulerRun`, task state/result/event records, modes, cache policy, synthetic outcomes, output/diagnostic refs, queues, order keys, diagnostics, `CancellationPolicy` re-export, `run_scheduler` | `crates/mizar-build/src/scheduler.rs` | scheduler unit tests が readiness transitions、queues、priority hints、completion-order independence、cache hit/miss scheduling、resource admission、cancellation、failure/block propagation、event/result collation、immutable synthetic outputs、placeholder absence を覆う。 | No finding. |
| [resource.md](./resource.md) | `ResourceBudget`, `TaskResourceRequest`, request units, admission status, admission records, telemetry, scopes, `ResourceManager`, `resource_queue_rank` | `crates/mizar-build/src/resource.rs` | resource tests が hierarchical scopes、delayed admission without overcommit、impossible requests、idempotent duplicate admission、release accounting、worker/memory pools、ATP portfolio/process separation、backend fanout、deterministic telemetry を覆う。 | No finding. |
| [cancel.md](./cancel.md) | `CancellationGeneration`、policy/state/token/decision records、reasons、decisions、checkpoints、freshness/publication guards、graph-ordered decision helpers | `crates/mizar-build/src/cancel.rs`; scheduler integration は `src/scheduler.rs` | cancellation tests が monotonic generations、snapshot supersession、pending/ready/running/completed decisions、checkpoint decisions、obsolete result discard、idempotent requests、canonical ordering、scheduler cancellation、resource release を覆う。 | No finding. |
| [failure_state.md](./failure_state.md) | `FailureCategory`, `BlockReason`, `FailureSourceOrder`, `BuildFailureRecord`, `BlockedTaskRecord`, synthetic failure categories, stable sort keys | `crates/mizar-build/src/failure_state.rs`; scheduler integration は `src/scheduler.rs` | failure-state と scheduler tests が direct failures、bounded blockers、nearest blockers、independent failures、deterministic ordering、cancelled versus failed states、inherited producer outputs の不在を覆う。 | No finding. |
| [artifact_commit.md](./artifact_commit.md) | `ManifestCommitRequest`, `ScheduledManifestUpdate`, `ManifestCommitSummary`, `CommittedModuleUpdate`, `ArtifactCommitError`, `commit_manifest_updates`, `sorted_manifest_updates` | `crates/mizar-build/src/artifact_commit.rs` | artifact-commit tests は `commit_manifest_updates` を通じて shuffled update determinism、freshness rejection preserving previous manifests、`mizar-artifact` manifest error propagation、publication-authority placeholders の不在、batch/determinism suites からの commit-order integration を覆う。 | BUILD-G-016: `sorted_manifest_updates` の standalone public-helper coverage が不足している。 |
| [cache_seam.md](./cache_seam.md) | `CacheSchedulingPlan`, task decisions, hit/miss/unavailable/no-key outcomes, validated hit payloads, cache output/diagnostic refs, plan diagnostics, `validated_decision_map` | `crates/mizar-build/src/cache_seam.rs`; scheduler integration は `src/scheduler.rs` | cache-seam と scheduler tests が externally supplied validated hits、clean-equivalent scheduler payloads、fallback execution、disabled policy behavior、duplicate/unknown decisions、deterministic hit payload collation、local cache-key/fingerprint/proof-reuse logic の不在を覆う。 | No finding. |
| [batch_integration.md](./batch_integration.md) | planner、module index、task graph、scheduler、cache seam、artifact commit をまたぐ available batch path | `crates/mizar-build/tests/batch_integration.rs` | integration tests が plan -> graph -> schedule -> commit、cache hit non-authority、explicit external-gap placeholder guards を覆う。 | No finding. |
| [determinism_suite.md](./determinism_suite.md) | implemented seams の cross-boundary determinism | `crates/mizar-build/tests/determinism_suite.rs` | determinism tests が shuffled logical inputs、scheduler worker/priority/completion variants、cache hit/miss placement、shuffled manifest updates、boundary placeholder absence を覆う。 | No finding. |
| 全 module specs | 現在の全 public enum に対する public enum forward-compatibility policy | `crates/mizar-build/src/**/*.rs` の `#[non_exhaustive]` attributes; `crates/mizar-resolve/src/module_index.rs` の downstream wildcard arm | `tests/lint_policy.rs` が source を scan し、EN/JA policy rows の完全一致を確認し、downstream-compatible public enum declarations を要求する。`cargo test -p mizar-resolve` が現在の downstream build-side consumer を検証する。 | No finding. |

## 挙動対応

| 仕様で約束された挙動 | Source/test correspondence | Finding |
|---|---|---|
| phase-0 planning は決定的で、invalid manifests、lockfiles、dependency cycles、version conflicts、unsupported editions、non-canonical paths を拒否する。 | `planner.rs` parser/validator/resolver source と focused planner unit tests。 | No finding. |
| module identity は package-scoped、alias-independent、provider-accessible であり、source/snapshot identity を割り当てない。 | `module_index.rs` source と provider/fixture tests。 | No finding. |
| task graph identity、correctness edges、dependency coverage、VC descriptor handling は deterministic で、proof/cache authority から分離される。 | `task_graph.rs` source と graph expansion、edge、coverage、boundary tests。 | No finding. |
| scheduler readiness、queue routing、priority hints、cache hits、cancellation、failures、resource admission は execution latency/state だけに影響し、canonical semantic/artifact ordering には影響しない。 | `scheduler.rs`, `resource.rs`, `cancel.rs`, `failure_state.rs`, `cache_seam.rs`, integration/determinism tests。 | No finding. |
| resource budgets は overcommit せず queue し、正確に一度 release し、ATP portfolio coordination と backend process slots を分離し、publication/proof authority を mint しない。 | `resource.rs` と scheduler resource-admission tests。 | No finding. |
| cancellation は cooperative、versioned、deterministic で、stale/partial current publication を防ぎ、proof failure や cache validation にはならない。 | `cancel.rs` と scheduler cancellation tests。 | No finding. |
| failure propagation は direct failures と bounded blocked states を記録し、producer outputs のコピー、diagnostics の創作、cancellation の proof failure 化を行わない。 | `failure_state.rs` と scheduler failure tests。 | No finding. |
| artifact commit は `mizar-artifact` manifest transactions と caller-supplied entries だけを消費し、artifact schema、producer payloads、tokens、proof authority を所有しない。 | `artifact_commit.rs`、`mizar-artifact` tests、batch/determinism suites。 | No finding. |
| cache-aware scheduling は外部で検証済みの cache decisions だけを消費する。cache hit は execution skip の候補だが、semantic acceptance、proof authority、trusted status を昇格しない。 | `cache_seam.rs`、scheduler cache tests、batch cache test、determinism suite、`mizar-cache` adjacent tests。 | No finding. |
| crate は `mizar-driver` に依存せず、driver-owned requests、sessions、event streams、registry dispatch、`salsa` query storage を実装しない。 | `Cargo.toml` dependency tree、`tests/batch_integration.rs`、boundary guard tests。 | No finding. |

## 残る gaps

task 22 は新しい blocking/high `spec_gap`、`test_gap`、`design_drift`、
`source_drift`、`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`、`repo_metadata_conflict` を開いていない。

既存の non-blocking follow-up gaps は残る:

| Gap | Class | Disposition |
|---|---|---|
| BUILD-G-016 | `test_gap` | `sorted_manifest_updates` は public helper であり、`commit_manifest_updates` を通じて間接的に exercised されているが、standalone canonical ordering の direct focused test がない。method-level helper coverage を主張する前に、後続 artifact-commit hardening slice で focused test を追加する。 |
| BUILD-G-002 / BUILD-G-011 | `external_dependency_gap` | `mizar-driver` が存在しないため、real requests、sessions、event streams、phase registry、cache-query adapter、driver-owned `salsa` boundary は消費できない。`mizar-build` は entry-point agnostic のままで、`mizar-driver` に依存しない。 |
| BUILD-G-003 / BUILD-G-012 | `external_dependency_gap` | `mizar-ir` が存在しないため、real sealed output handles、output storage、snapshot-handle rehydration は消費できない。実装済み scheduler tests は synthetic immutable output refs だけを使う。 |
| BUILD-G-004 / BUILD-G-013 | `external_dependency_gap` | Real producer artifact publication tokens と full phase-15 emission inputs はまだ利用できない。`mizar-build` は caller-supplied `mizar-artifact` entries を消費し、tokens を創作しない。 |
| BUILD-G-006 / BUILD-G-015 | `external_dependency_gap` | Full real resolver/checker/VC/proof/kernel/driver integration と clean/incremental/parallel equivalence は external seams が存在するまで利用できない。task 24 が implemented-seam equivalence gate を所有する。 |
| BUILD-G-009 | `external_dependency_gap` | Driver-owned cache query integration、`mizar-ir` output rehydration、producer publication tokens は存在しない。cache seam は caller-supplied decisions だけを消費し続ける。 |

## 検証

task 22 は documentation-only である。検証結果は task commit とともに記録し、
documentation diff checks を含める。直前の task-21 commit は、監査対象 source と
隣接する public-enum consumer changes が compile することをすでに検証している。
