# mizar-ir Architecture-22 Follow-Up Audit

> 正本は英語です。英語版:
> [../en/architecture_22_follow_up_audit.md](../en/architecture_22_follow_up_audit.md)。

## Scope

この task-18 audit は、`mizar-ir` が所有する architecture-22 freshness contract
について source/spec 対応監査と二言語 document sync scope を再実行する。obsolete
output は current として publish できず、open-buffer output は package artifact に
ならず、old output は validated cache-input boundary を通った場合だけ再利用できる。

この audit では code change を必要とする source drift または documentation drift は
見つからなかった。downstream clean/incremental/parallel driver equivalence は、crate
plan が記録済みの `external_dependency_gap` risk tag のままである。

## Source/Spec Trace

| Architecture-22 rule | mizar-ir specification | Source and tests | Result |
|---|---|---|---|
| obsolete snapshot result は current result として publish できない。 | `publisher.md` は明示的な current/obsolete state を要求し、obsolete current publication を禁止する。`identity.md` と `storage.md` は retain された old handle が current result になれないと述べる。`projection.md` は obsolete draft を current artifact candidate として拒否する。 | `publisher.rs::validate_snapshot_state` は `ObsoleteSnapshot` を返す。publisher test `rejects_wrong_obsolete_open_and_stale_publication` と `snapshot_replacement_makes_old_outputs_stale_but_retained_until_release`、projection test `obsolete_snapshot_is_rejected_as_current_projection` が cover する。 | Covered。 |
| open-buffer/editor-only output は package artifact ではない。 | `publisher.md` は `OpenBuffer` / editor-only origin を current/package publication で拒否する。`cache_adapter.md` は open-buffer dry-run cache record を scope 外とする。`projection.md` は non-current package output から artifact draft を返さない。 | `publisher.rs::validate_snapshot_state` は `OpenBufferOutput` を返す。projection test `internal_only_open_output_is_rejected_before_draft_returns` と `internal_only_reseal_of_collected_current_output_is_rejected` が cover する。 | Covered。 |
| old output は retained stale data または validated cache input としてのみ読める。 | `publisher.md`、`storage.md`、`identity.md` は、retain された old output を diagnostics/explanations/LSP または validated cache input には許すが、current publication には使えないと述べる。 | storage の retain/collect test は stale output が retain 中だけ読めることを示す。cache adapter test `superseded_snapshot_output_can_still_encode_as_cache_input` は、old output の encode が cache-input path に残る一方、`validate_current_output` が current publication を拒否することを示す。 | Covered。 |
| cache hit rehydration は optimization-only で fail-closed である。 | `cache_adapter.md` は lookup/key/dependency/proof validation を `mizar-cache` に委ね、non-validated state を miss とし、payload、side-table、parent、schema、storage check 後にだけ current-snapshot handle を seal する。 | `cache_adapter.rs::rehydrate` は cache miss、corrupt/incompatible record、hash mismatch、parent mismatch、stale/collected parent、storage failure、decode error で allocation/seal 前に miss を返す。test はこれらの path で target lineage と successful rehydration が発生しないことを assert する。 | Covered。 |
| cache rehydration は proof/trust authority を作らない。 | `cache_adapter.md`、`publisher.md`、`projection.md`、`identity.md` は、cache hit と rehydrated handle が proof acceptance、trusted status、verifier policy、kernel acceptance を昇格しないと述べる。 | `rehydrated_handles_do_not_carry_cache_or_proof_authority`、`publisher_handles_do_not_carry_proof_cache_or_trust_authority`、`projection_does_not_expose_cache_or_proof_authority_markers` がこの boundary を cover する。 | Covered。 |
| published artifact は stable projection だけを出し、raw IR や storage/kernel internal は出さない。 | `projection.md` は raw `SurfaceAst`、`TypedAst`、`CoreIr`、`ControlFlowIr`、`VcIr`、`AtpProblem`、kernel-internal state、storage handle、inline proof-witness payload を禁止する。 | projection leakage test は export、expression、obligation、diagnostic、provenance、witness、dependency artifact reference 全体の raw marker と hash-ref rejection を cover する。 | Covered。 |

## Bilingual Sync Trace

scope 内の English document と Japanese companion は、architecture 22 terminology と
ownership boundary について同期している:

| Pair | Scoped result |
|---|---|
| `publisher.md` | 両言語とも obsolete/current validation、open-buffer rejection、retained stale output limit、validated cache-input handoff を cover する。 |
| `cache_adapter.md` | 両言語とも validated-hit-only rehydration、fail-closed miss state、stale-data freshness、cache-key ownership 不在、proof/trust elevation 不在を cover する。 |
| `identity.md` and `storage.md` | 両言語とも snapshot-scoped identity/storage、stale retained handle、replacement 後の current-result reuse 禁止を cover する。 |
| `projection.md` | 両言語とも current draft validation、obsolete/open rejection、raw IR leakage guard、external publication-token gap を cover する。 |
| `00.crate_plan.md` and `todo.md` | 両言語とも task 18 scope、completion condition、`external_dependency_gap` risk tag が揃っている。 |

## Classified Gaps

architecture-22 の publisher/cache/snapshot-replacement scope について、current な
`spec_gap`、`source_drift`、`test_expectation_drift`、`boundary_violation`、bilingual
drift は見つからなかった。

`IR-G-007` は system-level `test_gap` かつ `external_dependency_gap` risk として残る:
full clean/incremental/parallel driver equivalence は、この checkout で `mizar-ir` にまだ
wiring されていない downstream orchestration と real producer/cache/artifact seam を必要
とする。この task は `mizar-driver` dependency、placeholder diagnostics integration、
producer-token、cache-key、dependency-fingerprint、proof-policy API を追加しない。

## Audit Result

Task 18 は source change なしで close する。obsolete/open/incomplete/cache-miss rule は
module spec、source、test、bilingual companion doc へ trace 済みである。Task 19 は、この結果を
module boundary と private helper placement の audit input として使う。
