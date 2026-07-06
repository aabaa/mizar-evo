# mizar-proof Architecture-22 Follow-Up Audit

> 正本: [../en/architecture_22_audit.md](../en/architecture_22_audit.md)。

## 範囲

task 18 は task 17 の proof-reuse metadata export 契約を次に照らして再監査する:

- [`selection.md`](./selection.md) と `crates/mizar-proof/src/selection.rs`;
- [`status.md`](./status.md) と `crates/mizar-proof/src/status.rs`;
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md);
- [architecture 11](../../architecture/ja/11.artifact_and_incremental_build.md);
- [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md);
- `mizar-proof` の English/Japanese paired documentation。

この audit は documentation-only である。cache lookup、artifact publication stub、
ATP backend wiring、proof acceptance、trusted-status promotion は追加しない。

## 結果

task 17 の reuse metadata 契約について、`mizar-proof` 内に blocking な
`spec_gap`、`test_gap`、`design_drift`、`source_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation` は残っていない。

task 18 中に範囲外の `repo_metadata_conflict` が 1 件 `mizar-atp` に観測された。
`mizar-atp` の task-28 closeout guard は、現在の `mizar-proof` workflow が formal
scaffold を明示的に許可し完了しているにもかかわらず、まだ `crates/mizar-proof` を
forbidden placeholder crate と見なしていた。この audit はその conflict を記録するだけ
だった。この conflict は task-20 closeout 前に focused metadata correction commit
`36d1a9c` で解消済みである。

## Architecture-22 Trace

| Requirement | Source/doc evidence | Result |
|---|---|---|
| `ObligationAnchor` は reuse-candidate key であり proof authority ではない。 | `ProofObligationIdentity` が `ObligationAnchor` を保持し、`StatusReuseMetadata` が export する。status docs は anchor だけでは不十分と述べる。 | consistent |
| reuse には canonical VC、local-context、dependency-slice、policy、witness/discharge identity が必要である。 | `StatusReuseMetadata` は obligation、VC、context、dependency-slice、policy、selected witness、deterministic discharge、validation hash field を export する。 | consistent |
| dependency artifact/schema compatibility は reuse validation に参加する。 | `ProofReuseDependencyCompatibility` は dependency artifact fingerprint、dependency schema version、proof-reuse schema version を記録する。tests はそれぞれ独立に mutate する。 | consistent |
| proof evidence identity は stable かつ deterministic である。 | `ProofEvidenceReuseIdentity` は selected candidate id、selected provenance hash、selected evidence/witness/discharge hash、tie-break hash、selection reason を expose する。 | consistent |
| arrival order、completion time、runtime duration、cache timing は identity ではない。 | selection tie-break は stable priority/hash/provenance/source-id field を使う。determinism tests は candidate order を shuffle する。timing/cache-hit field は存在しない。 | consistent |
| externally attested evidence は cache reuse によって upgrade されない。 | `PolicyPermittedExternal` は `ExternallyAttested` に project され、trusted `used_axioms` は absent のままである。`cache_reuse_predicate_complete` はすべての non-trusted class で false を返す。 | consistent |
| trusted reuse completeness には accepted witness/discharge identity が必要である。 | `KernelVerified` completeness は `selected_proof_witness_hash` を要求し、`DischargedBuiltin` completeness は `deterministic_discharge_hash` を要求する。dependency compatibility 欠落は incomplete である。 | consistent |
| cache record は proof authority ではない。 | `mizar-proof` は metadata と validation hash だけを expose する。cache lookup、cache hit、cache record reader、cache promotion API は存在しない。 | consistent |

## Bilingual Sync

task 17 と task 18 で変更された English canonical files には同期した Japanese
companion がある:

| English canonical | Japanese companion | Task-18 result |
|---|---|---|
| `00.crate_plan.md` | `00.crate_plan.md` | completed-audit inventory が同期している |
| `selection.md` | `selection.md` | task-17 reuse field が同期している |
| `status.md` | `status.md` | validation hash と class-aware completeness が同期している |
| `todo.md` | `todo.md` | task-17 status は同期済みであり、task 18 が本 task で更新される |
| `task_ledger.md` | `task_ledger.md` | task-17 hash backfill と task-18 entry が同期している |
| `architecture_22_audit.md` | `architecture_22_audit.md` | 本 task で作成 |

Japanese companion は Rust type name、enum variant、status string、command name、
gap identifier を意図的に English のまま保持する。これは drift ではない。

## Remaining Gaps

| ID | Class | Evidence | Handling |
|---|---|---|---|
| `PROOF18-G001` | `external_dependency_gap` | `mizar-cache` は現在存在し、proof-reuse validation、lookup、cache hit/miss decision、policy compatibility check を所有する。この proof milestone は validation metadata を export するが、それらの API は呼ばない。 | `mizar-proof` metadata は validation input としてだけ保つ。proof/cache wiring は owner-scoped integration task で追加する。 |
| `PROOF18-G002` | resolved roadmap drift with remaining `external_dependency_gap` | 詳細な cache proof-reuse document は現在 `doc/design/mizar-cache/` の下に存在する。`mizar-proof` に placeholder cache doc/API は不要である。 | metadata API を stable に保ち、validation semantics は `mizar-cache` が所有する。 |
| `PROOF18-G003` | `external_dependency_gap` | 現在の artifact witness schema はまだ `DischargedBuiltin` witness ref を publish できない。 | accepted proof-obligation kernel evidence 後の internal trusted class として `DischargedBuiltin` を保ち、deterministic discharge hash を export し、artifact witness publication は defer する。 |
| `PROOF18-G004` | `external_dependency_gap` | `CommittedWitnessPublicationProof` は artifact-owned production token がないため opaque のままである。 | witness publication は artifact manifest reachability integration まで block されたままである。 |
| `PROOF18-G005` | `external_dependency_gap` | `TrustedKernelWitnessMetadata` は copied kernel/artifact acceptance metadata がないため opaque のままである。 | `mizar-proof` は caller-synthesized kernel acceptance metadata を trust してはならない。 |
| `PROOF18-G006` | `external_dependency_gap` | live ATP early-stop adoption と backend cancellation は `mizar-proof` の外に残る。 | policy hook は stable metadata/API のままにし、backend execution や cancellation stub は追加しない。 |
| `PROOF18-G007` | `deferred` | task-17 completeness test は kernel-with-witness、kernel-without-witness、built-in discharge、externally attested evidence を覆う。他の non-trusted class は同じ明示的な match arm を共有するが、その test では個別列挙していない。 | task 19 または closeout が exhaustive non-trusted class fixture を求める場合だけ non-blocking branch-coverage follow-up として扱う。 |

## Repo Metadata Conflict

| ID | Classification | Evidence | Handling |
|---|---|---|---|
| `PROOF18-RM001` | resolved `repo_metadata_conflict` | task 18 中、`cargo test -p mizar-atp` は `atp_task_twenty_eight_crate_exit_report_is_documented` で失敗した。`mizar-atp` closeout guard が workspace member `crates/mizar-proof` と `crates/mizar-proof` directory を task-28 placeholder として reject したためである。 | task 18 では report のみ。focused metadata correction commit `36d1a9c` で後に解消済み。proof policy ownership は `mizar-atp` へ移していない。 |

## 結論

task 17 の proof-reuse metadata export 契約は、`mizar-proof` の ownership boundary
について architecture 22 を満たしている。crate は将来の cache consumer 向けに安定し
決定的な validation metadata を export しつつ、trusted proof acceptance を accepted
proof-obligation kernel evidence だけに結びつけている。残作業は downstream integration
または記録済み branch-coverage follow-up であり、cache、external、diagnostic、
consistency-check、witness metadata を trusted status へ promote する許可ではない。
