# Crate Exit Report: mizar-ir

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: task 20 dispatch-input follow-up 後に完了。
Quality score: 95/100。
Score cap: なし。

## Scope

milestone scope:

- task 0 から task 19 とこの closeout task までで `mizar-ir` workspace crate を
  構築する。
- compiler-internal immutable IR output storage と typed `PhaseOutputRef<T>`
  handle を所有する。
- `mizar-session` の snapshot/source identity を消費しつつ、snapshot-scoped
  IR identity table と parent/derived phase-output lineage を所有する。
- complete output を seal し、current snapshot / work unit を validate し、
  unsealed、partial、obsolete output を current publication から隠す
  phase-output publisher を所有する。
- retained output の content-addressed internal blob、retention、collection、
  snapshot replacement behavior を所有する。
- `mizar-cache` record と validated lookup outcome を消費し、fail-closed
  validation 後にだけ handle を rehydrate する IR cache adapter boundary を
  所有する。
- seal 済み internal output から stable `mizar-artifact` draft schema への
  artifact projection を所有する。

excluded:

- proof acceptance、trusted status、verifier-policy selection、deterministic
  proof winner selection、kernel acceptance、trusted `used_axioms`、または
  proof-authority decision。
- `mizar-cache` `CacheKey` construction、dependency-fingerprint construction、
  dependency-slice ownership、proof-reuse validation policy。
- published artifact 内の raw `SurfaceAst`、`TypedAst`、`CoreIr`、
  `ControlFlowIr`、`VcIr`、`AtpProblem`、kernel-internal state、storage
  reference、inline proof witness payload。
- placeholder producer/diagnostics integration API、producer-publication
  token、artifact-publication token、または `mizar-ir` 内の `mizar-driver`
  dependency。

## Task Commits

| Task | Commit | Subject |
|---:|---|---|
| 0 | `58c515e05f1ffaaee16e080c6016254e671b30e8` | `docs(ir-task-0): add autonomous crate plan` |
| 1 | `ad26742f4a7ff11c9728defb17bd211da8911193` | `feat(ir-task-1): scaffold ir crate` |
| 2 | `992d5c66eb0726489762d78b224403f9eabb9388` | `docs(ir-task-2): specify ir identity` |
| 3 | `d395dd8399cc0285ce3a9a746ff445a543c270a5` | `feat(ir-task-3): add snapshot handle registry` |
| 4 | `fc93438743a753b39f080fe25a4690ce2b3557f0` | `docs(ir-task-4): specify ir storage` |
| 5 | `2200974e8f88245ec49e4ab31be1f70a78360e32` | `feat(ir-task-5): add sealed storage handles` |
| 6 | `2a987b3f0d8b0329422d0998457b364e5cbc7b85` | `feat(ir-task-6): add blob storage collection` |
| 7 | `585a84aa92a49d585334a78473cc6e11db4f351a` | `docs(ir-task-7): specify phase publisher` |
| 8 | `15fbb29342029d251176dafddf1d18f0b1122dbf` | `feat(ir-task-8): add phase output publisher` |
| 9 | `7149cc6cbbdf52ff858d76ab2a44c93db23722fe` | `docs(ir-task-9): specify cache adapter` |
| 10 | `8782f80b11cc1039a531e816f356d354709e8b48` | `feat(ir-task-10): add cache adapter` |
| 11 | `77d28cc0ab6de03df72b5da815b754f5f997f6a1` | `docs(ir-task-11): specify artifact projection` |
| 12 | `c96a8980d1634866f3ce3afb4a4155e4704cface` | `feat(ir-task-12): add artifact projection service` |
| 13 | `4ba76511c16630d63463d9bcd8b1f251dbaf503e` | `feat(ir-task-13): add snapshot replacement` |
| 14 | `89b36ab6a6de778dcc3d50f88848d0a8adce9492` | `test(ir-task-14): add determinism lifetime suite` |
| 15 | `c904f46ca806b1e4f18cffa26de26a2ed75c670f` | `docs(ir-task-15): record enum compatibility policy` |
| 16 | `331215b976d1896e5a4670ef8fbd89ec5ce56c2e` | `docs(ir-task-16): audit source spec correspondence` |
| 17 | `7a8d5efab3256e7fc7079cc63a367b61be01817c` | `docs(ir-task-17): audit bilingual documentation sync` |
| 18 | `ba01d4f6e6978c62e520292e314cd39ea412c89b` | `docs(ir-task-18): audit architecture 22 freshness` |
| 19 | `b0a0201bf783797ed03ca2ceeac3e500c9c322db` | `docs(ir-task-19): audit module boundaries` |
| 20 | pending self-hash | `feat(ir-task-20): add dispatch input boundary` |

## Final Owned Surfaces

| Surface | Final shape |
|---|---|
| Snapshot identity and handles | `SnapshotHandleRegistry` は exact snapshot、source/input、phase、work-unit、producer-path、parent identity input から IR-local id を deterministic に割り当てる。`BuildSnapshotId` と source identity construction は `mizar-session` に残り、incompatible snapshot は同一 handle family として lineage を reuse できない。 |
| Storage | `IrStorageService` は pending slot を allocate し、complete output を seal し、typed `PhaseOutputRef<T>` handle を返し、`typed_handle` で erased handle を validate し、side table を保存し、大きな canonical payload を content-addressed internal blob へ spill し、stale、collected、unsealed、wrong-type、foreign handle を fail closed する。 |
| Phase output publisher | `PhaseOutputPublisher` は current snapshot、allowed work unit、slot metadata、parent handle、deterministic content hash、side-table hash を validate してから seal する。unsealed、partial、obsolete、wrong-snapshot、superseded output は current publication から不可視である。 |
| Cache adapter | `IrCacheAdapter` は caller-supplied `mizar-cache` `CacheKey`、`CacheRecord` payload、validated lookup outcome を消費する。key/header/dependency/proof compatibility は `mizar-cache` から消費し、その後 adapter schema、cacheability marker、parent、payload hash、side-table hash、storage freshness、publisher acceptance を validate してから rehydrated handle を返す。unknown、incomplete、uncacheable、incompatible、corrupt、tampered、stale、undecodable state はすべて handle exposure 前に miss になる。 |
| Artifact projection | `ArtifactProjectionService` は current seal 済み handle を validate し、stable `mizar-artifact` schema を使って unpublished `VerifiedArtifactDraft` を作る。raw internal IR name、storage handle、kernel-internal state、inline proof witness payload、duplicate projected row、schema mismatch を reject する。 |
| Snapshot replacement | `replace_current_snapshot` は old snapshot を current publication から supersede しつつ、retain された old output を release/collection まで readable または cache-encodable に保つ。obsolete output は新 snapshot の validated cache-rehydration boundary を通る場合を除いて current result にならない。 |
| Dispatch input boundary | `PhaseInputIdentities`、snapshot-bound `PhaseDispatchInputBundle`、`SealedParentOutputHandle`、`PhaseDispatchInputProvider<Task>` は `mizar-ir` が所有する。parent output identity は validated sealed handle からだけ導出し、bundle/provider validation は owner input 欠落と invalid snapshot/storage/currentness failure を区別する。 |
| Integration boundary | real driver/front-door と diagnostics crate は現在存在するが、diagnostics registry/rendering integration、producer projection payload、artifact publication token、semantic/proof adapter、cache-compatibility wiring、LSP conversion、system-level clean/incremental/parallel equivalence は `external_dependency_gap` または `deferred` として残る。placeholder producer、stub API、fake token、`mizar-ir` への `mizar-driver` dependency は追加していない。 |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | `00.crate_plan.md`、task-scoped module spec、TODO、architecture-22 audit、source/spec audit、module-boundary gate、closeout review は unresolved blocking/high inconsistency なし。 |
| Source behavior documented or deferred | passed | public API、private canonicalization/retention/fail-closed/projection-filtering behavior、test、residual gap は `source_spec_correspondence.md`、`bilingual_documentation_sync.md`、`architecture_22_follow_up_audit.md`、`module_boundary_refactor_gate.md`、本 report に trace される。 |
| Deterministic snapshot identity | passed | identity spec/test は exact input からの deterministic id、duplicate/conflicting key rejection、parent lineage、incompatible snapshot rejection、IR-local id が proof-reuse authority ではないことを cover する。 |
| Sealed immutable outputs | passed | storage/publisher spec/test は seal 済み handle だけが escape すること、pre-seal/double-seal access の reject、type/generation/storage ownership validation、partial/obsolete output を current publication から外すことを要求する。 |
| Fail-closed cache rehydration | passed | cache adapter spec/test は cache miss、incomplete/unknown/uncacheable/incompatible record、corrupt payload、tampered payload/side-table hash、parent mismatch、stale/collected parent、decode error、publisher/storage failure を `PhaseOutputRef<T>` 返却前に reject する。 |
| Artifact projection boundary | passed | projection spec/test は stable draft schema だけを expose し、raw `SurfaceAst`、`TypedAst`、`CoreIr`、`ControlFlowIr`、`VcIr`、`AtpProblem`、storage handle、kernel-internal state、inline witness payload を reject する。 |
| Proof and cache authority boundary | passed | `mizar-ir` は proof acceptance、trusted status、policy selection、kernel acceptance、`CacheKey`、dependency fingerprint、proof-reuse validation を所有しない。lint guard と module test がこの boundary を cover する。 |
| Test expectation integrity | passed | current implementation behavior に合わせるために `doc/spec` language file、`.miz` fixture、traceability row、expectation sidecar を変更していない。 |
| Design/source synchronization | passed | English canonical docs と Japanese companion は paired/synchronized であり、source/docs audit と lint guard は current drift なしを記録する。 |
| Downstream gaps classified | passed | real producer、diagnostics rendering、artifact publication token、semantic/proof adapter、cache-compatibility、LSP conversion、full system-equivalence work は stub ではなく `external_dependency_gap` または `deferred` として分類されている。 |
| Verification | passed | crate-local と workspace Rust verification、隣接 cache/artifact/build/driver test、diff check、staged diff check、task-20 review が pass。`mizar-driver` が IR 所有 dispatch input bundle を消費するため、driver verification も含めた。 |

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

real diagnostics rendering、producer-owned projection payload、
artifact-publication-token integration、semantic/proof adapter、
cache-compatibility wiring、LSP conversion、full clean/incremental/parallel
system equivalence は milestone 外の downstream gap であるため減点する。
これらは stub ではなく分類済みである。task 20 は dispatch-input boundary を追加し、
driver front-door verification を pass したため score を復帰させた。

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | finding なし。closeout scope、owned/excluded boundary、hard gate、external gap、quality score rationale、handoff は crate plan と module spec に対して complete かつ consistent。 |
| Test sufficiency review | finding なし。test は deterministic identity、sealed storage、publisher currentness、fail-closed cache rehydration、projection leakage、snapshot replacement、lifetime collection、enum policy、dispatch input validation、provider missing/error branch、driver consumption、boundary guard を cover する。system-level equivalence は `external_dependency_gap` として正しく分類される。 |
| Full implementation review | low documentation wording fix 後 finding なし。実装済み source boundary は dispatch input identity と seal 済み parent handle を `mizar-ir` が所有し、driver が scheduler-selected dispatch で消費する形になっている。 |
| Source/documentation consistency review | finding なし。英語版と日本語版の closeout report は同期し、task status と verification record は source/test と一致し、source/documentation ownership statement は一致する。 |
| Read-only crate quality review | 95/100。hard gate は pass し、unresolved blocking/high/medium finding は残っていない。 |

## Deferred And External Dependency Items

| ID | Crate-plan class | Risk tag / status | Owner / unblock condition |
|---|---|---|---|
| IR-G-004 | `design_drift` | `external_dependency_gap` | `mizar-driver` は現在 registry/front door を提供するが、real producer dispatch input、real record 上の driver-owned cache lookup、cache scheduling integration、publication freshness wiring はまだ downstream integration を必要とする。`mizar-ir` は引き続き `mizar-driver` dependency を持ってはならない。 |
| IR-G-005 | `design_drift` | `external_dependency_gap` | real `mizar-diagnostics` registry/rendering integration は diagnostics owner が提供する必要がある。`mizar-ir` は stable side-table/projection reference だけを保存し、stub diagnostics crate/API を追加しない。 |
| IR-G-006 | `design_drift` | `external_dependency_gap` | real resolver/checker/core/VC/ATP/kernel/proof producer projection payload、producer publication token、artifact publication token は owning crate が供給する必要がある。projection は stable draft schema 上に留まり、token を mint しない。 |
| IR-G-007 | `test_gap` | `external_dependency_gap` | full clean/incremental/parallel driver equivalence は downstream orchestration と real producer/cache/artifact/driver seam を必要とする。その phase が存在するまで、crate-local deterministic/lifetime test が実装済み boundary を cover する。 |
| IR-G-008 | `boundary_violation` | guarded ownership constraint | `mizar-cache` key、dependency fingerprint、proof-reuse validation、proof trusted status、kernel acceptance の再実装は boundary violation になる。現在の source/spec/test はこれを guard している。 |
| IR-G-009 | `design_drift` | resolved locally | cache-key または snapshot-identity ownership を `mizar-ir` に割り当てるように読める古い internal API sketch は、`mizar-session` identity と `mizar-cache` validated record だけを消費する形で解決済み。 |

## Test Expectation Summary

`mizar-ir` milestone では language specification、`.miz` test、coverage
traceability metadata、expectation sidecar を変更していない。crate-owned
behavior は Rust unit test、integration test、lint-policy guard、
determinism/lifetime suite、source/spec audit、architecture-22 audit、
module-boundary audit、bilingual audit、explicit gap record で cover する。

## Verification Commands

| Command | Result |
|---|---|
| `cargo test -p mizar-ir` | passed |
| `cargo clippy -p mizar-ir --all-targets -- -D warnings` | passed |
| `cargo fmt --check` | passed |
| `cargo test -p mizar-cache` | passed |
| `cargo test -p mizar-artifact` | passed |
| `cargo test -p mizar-build` | passed |
| `cargo test -p mizar-driver` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | passed |

task 20 は earlier no-driver condition を supersede した。`mizar-driver` は現在存在し、
IR 所有 dispatch input bundle を消費するため検証済みである。staged diff check は
task-20 commit の直前に pass した。

## Human Review Surface

primary human review では次を確認する:

- [00.crate_plan.md](./00.crate_plan.md)
- [todo.md](./todo.md)
- [identity.md](./identity.md)
- [storage.md](./storage.md)
- [publisher.md](./publisher.md)
- [cache_adapter.md](./cache_adapter.md)
- [projection.md](./projection.md)
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_sync.md](./bilingual_documentation_sync.md)
- [architecture_22_follow_up_audit.md](./architecture_22_follow_up_audit.md)
- [module_boundary_refactor_gate.md](./module_boundary_refactor_gate.md)
- `crates/mizar-ir/src/`
- `crates/mizar-ir/tests/`

## Next-Phase Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
mizar-ir closeout commit が存在した後で、次の integration phase を開始する。
`mizar-ir` は compiler-internal IR storage、deterministic snapshot-scoped
handles、sealed typed `PhaseOutputRef<T>`、phase output publication、
cache-adapter rehydration boundary、artifact projection draft、snapshot
replacement の owner のままにする。proof acceptance、trusted status、
verifier-policy selection、kernel acceptance、`mizar-cache` `CacheKey`
construction、dependency fingerprint、proof-reuse validation を `mizar-ir` に
移してはならない。

よい次 task は real downstream integration phase である: driver front door を通る
producer dispatch input、real record 上の driver-owned cache lookup、cache scheduling
integration、diagnostics registry integration、producer projection payload、
artifact publication token のいずれかから始める。
owning crate が real seam を提供するまで、既存の `external_dependency_gap`
classification を保つ。placeholder crate、stub API、fake publication token、
`mizar-driver` dependency を `mizar-ir` に追加しない。

Cache hit rehydration は optimization-only のままである。incomplete、unknown、
uncacheable、incompatible、corrupt、stale、unvalidated record は handle を
reconstruct する前に miss にし、rehydrated handle は proof/trusted status を
昇格しない。
```

driver/cache/artifact/proof integration と広い API migration を同時に扱う場合だけ
`xhigh` より上げる。committed task hash の backfill や paired documentation row
更新のような狭い docs-only follow-up なら `high` に下げてよい。
