# Incremental/Parallel Equivalence Gate

> 正本は英語です。英語版:
> [../en/incremental_parallel_equivalence.md](../en/incremental_parallel_equivalence.md)。

状態: task 24 implemented-seam gate complete。

## 目的

task 24 は、この checkout で実装済みの `mizar-build` seams に対する
scheduler-level architecture-22 regression gate を追加する。この gate は clean
sequential execution を reference projection として扱い、同じ `BuildSnapshotId`
と synthetic verifier/artifact policy 上の clean parallel、incremental sequential、
incremental parallel runs が同一の externally visible results を publish することを
確認する。

この gate は、`mizar-build` に既に存在する build-side planning、task-graph、
scheduler、cache seam、cancellation、artifact-commit boundaries に意図的に限定する。
driver session、`mizar-ir` output handle、cache-key construction、dependency
fingerprint、proof-reuse validation、producer artifact publication token、proof
authority は追加しない。

## 範囲

implemented-seam equivalence projection は次を比較する。

- committed manifest hash。
- published module entries の package/module identity、source file、artifact file、
  source hash、artifact hash、interface hash、implementation hash、module-summary
  references、registration-summary references、proof-witness entries、
  diagnostics hash。
- scheduler-visible output references と payload labels。
- canonical scheduler diagnostics と result diagnostics。
- failure records と blocked task records。

Task execution state 自体は externally visible semantic projection ではない:
incremental run は clean reference が `Completed` を記録する箇所で `CacheHit` を
記録してよい。ただし visible manifest、hashes、proof-witness references、
canonical diagnostics は一致しなければならない。Cache miss は通常の work を enqueue
し、deterministic commit boundary を乱してはならない。

この gate は stale-publication behavior も確認する。completed-before-publication
checkpoint で破棄された validated hit は、その stale module や downstream dependents
を current artifacts として publish してはならない。superseded snapshot は current
manifest updates を publish してはならない。

## gap classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| BUILD-G-017 | `external_dependency_gap` | task 24 は architecture-22 equivalence を実装済み `mizar-build` seams に限って覆う。real `mizar-driver` build sessions、real `mizar-ir` output rehydration、producer-owned artifact projection、producer publication tokens はまだ利用できない。 | gate は synthetic かつ build-side に保つ。placeholder driver、IR、producer-token、cache-key、dependency-fingerprint、proof-reuse、proof-authority APIs を追加しない。full real end-to-end equivalence は、それらの seams が存在した後の external integration に属する。 |

BUILD-G-016 は `sorted_manifest_updates` helper の direct standalone coverage に
関する non-blocking `test_gap` のままである。task 24 はその artifact-commit
hardening gap を閉じない。

## test evidence

task 24 は `crates/mizar-build/tests/determinism_suite.rs` に次を追加する。

- `clean_and_incremental_parallel_runs_publish_identical_visible_projection`:
  同じ task graph と snapshot 上で clean sequential、clean parallel、incremental
  sequential、incremental parallel runs を比較する。
- `superseded_or_stale_incremental_results_do_not_publish_current_artifacts`:
  manifest publication 前の stale validated hits と snapshot supersession を確認する。

tests は caller-supplied cache decisions を使う。Validated hits は外部 validation 後の
execution skip としてのみ model される。misses は通常 execution に fallback する。
projection は artifact-facing hashes、summary references、proof-witness entries を
含むが、proof evidence を作らない。

## non-authority rules

- Cache-aware scheduling は optimization-only execution skip のままである。
- cache hit は semantic acceptance、proof authority、producer publication authority、
  trusted-status promotion にならない。
- artifact record と manifest commit は proof trust を昇格しない。
- worker completion order と cache hit/miss timing は canonical diagnostics、
  published artifact order、interface hashes、dependency-facing summaries、
  proof acceptance を決定しない。
- 欠けている real driver、IR、producer-token integrations は
  `external_dependency_gap` のままである。

## handoff

task 25 はこの gate に対して source/spec correspondence と bilingual documentation
audits を再実行する。その follow-up では、この file、更新された determinism suite、
未変更の external dependency gaps を audit inputs に含める。
