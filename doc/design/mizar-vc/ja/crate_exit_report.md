# Crate Exit Report: mizar-vc

> 正本言語: English。英語正本:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## 結果

Status: complete。
Quality score: 94/100。
Score caps applied: none。

## 範囲

Milestone scope:

- `mizar-vc` crate を preliminary task 0 から task 22 と closeout task まで構築する。
- pipeline phase 11-12 を所有する: VC generation、deterministic `VcId`
  assignment、status-policy projection、deterministic pre-ATP discharge、
  replayable in-memory discharge evidence、conservative dependency slice、
  reusable proof-reuse candidate key。
- 明示的な `mizar-core` `CoreIr`、`ControlFlowIr`、`ObligationSeedHandoff`
  payload を消費し、欠けている source、checker、registration data を再構築しない。
- proof acceptance、ATP execution、cache lookup、artifact publication、kernel
  validation、source-derived corpus extraction、利用不能な upstream payload family は
  external dependency gap または deferred work として分類する。

Included:

- `doc/design/mizar-vc/{en,ja}/` 配下の英日 crate plan、module spec、audit、
  closeout report。
- `crates/mizar-vc/src/` 配下の Rust source。
- `crates/mizar-vc/` 配下の crate-local unit test と integration test。
- `tests/coverage/spec_trace.toml` の deferred proof-verification corpus
  traceability row。

Excluded:

- `doc/spec` への直接編集。
- 既存 `.miz` fixture または expectation sidecar の rebaseline。
- active source-derived `proof_verification` fixture または snapshot。
- ATP problem encoding、backend process execution、certificate validation、
  trusted proof acceptance、cache hit acceptance、artifact publication。
- 利用不能な `mizar-atp`、`mizar-proof`、`mizar-cache` seam、または
  artifact/proof/cache consumer の placeholder crate / placeholder consumer。
- registration/redefinition/reduction、branch、match、loop、termination、
  ghost-erasure、trace、source-derived core formula、definition、quantified
  binder、source-derived obligation payload の仮造。

## Post-Closeout Evidence Correction

Task 24 は、`mizar-kernel` task 23-29 が legacy resolution-trace acceptance から
formula/substitution evidence と trusted in-process Rust SAT checking への checker-side
correction を完了した後、evidence-pipeline handoff documentation だけを再開する。
original mizar-vc closeout は phase 11-12、deterministic discharge、dependency slice、
proof-reuse candidate key について引き続き有効である。

Task 24 は paired [kernel_evidence_handoff.md](./kernel_evidence_handoff.md) spec を追加し、
古い downstream classification を更新する。`mizar-kernel` は修正後 evidence の trusted
checker として存在する一方、ATP candidate production、proof/cache consumer、
artifact witness consumer、kernel-evidence hash reuse integration は後続 task または
external gap のままである。この task は docs-only であり、Rust source、kernel call、
SAT solving、backend encoding、legacy certificate、捏造した formula/substitution/provenance
payload を追加しない。

Task 25 は explicit payload 向け producer-side `kernel_evidence_handoff` Rust module を
追加する。immutable formula/substitution/provenance handoff package と canonical hash input
を構築し、imported context requirement と discharge diagnostic を canonical envelope の外に
保ち、不足 payload では fail closed する。引き続き kernel call、SAT solving、legacy
certificate acceptance、backend proof method encoding、artifact/cache proof status publication は
行わない。

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 0 | `9697036b0f012cfc578a015dc5a0d6f37bf85143` | `docs(vc-task-0): add autonomous crate plan` |
| 1 | `adfff1cbc3ebce9db13e73d4d29bfd9b1ac1971d` | `feat(vc-task-1): scaffold mizar-vc crate` |
| 2 | `ac778b008be75ea21eda4d2e69c7713a88b0d4ea` | `docs(vc-task-2): specify vc ir data shapes` |
| 3 | `c32d767368ef9d16fdcf92620c2b2afecb13fc9d` | `feat(vc-task-3): add vc ir data shapes` |
| 4 | `ba20db550cf92979bdb8809e9f64fbe5cd193c1b` | `feat(vc-task-4): add seed intake table` |
| 5 | `e324beab799f972dcf78e897b163aebd9414725e` | `docs(vc-task-5): specify generator contract` |
| 6 | `b5634eb878b39558b981bcbba972e8b36c3203c9` | `feat(vc-task-6): add core generation candidates` |
| 7 | `a15a2ee3e21974727fab2f8406b2e161b3f3c2f7` | `feat(vc-task-7): add algorithm generation candidates` |
| 8 | `6b4a7ef661886d6339f8ac24e21ad68e9f7ac302` | `feat(vc-task-8): normalize vc generation candidates` |
| 9 | `30c8e303c2c88d70a0dd69295ec001280471519a` | `feat(vc-task-9): add vc status policy projection` |
| 10 | `18c86f9b03318c28e39311162ae3e89adc0e2d2a` | `docs(vc-task-10): specify discharge contract` |
| 11 | `d4643a7f1078ec330640e63021942bc245d9a609` | `feat(vc-task-11): add deterministic discharge engine` |
| 12 | `57c4e247ca13cdcf05e92d9854e41f60fa5e0f49` | `feat(vc-task-12): add discharge evidence records` |
| 13 | `6238217eedc55e76ec277ab14bd1d78a3b57c6a6` | `docs(vc-task-13): specify dependency slices` |
| 14 | `26e5fea26769e1bf7ccb47e99d814709f035801f` | `feat(vc-task-14): compute dependency slices` |
| 15 | `beee07a8009245e2bc0096d98df3968ea1212ac3` | `docs(vc-task-15): record proof verification corpus gaps` |
| 16 | `8b183e538fa4007e82b0c2b2af058ebe566fca22` | `test(vc-task-16): add determinism suite` |
| 17 | `f65ff56d9a3a555586cf21189780aaaa1017359d` | `test(vc-task-17): guard public enum policy` |
| 18 | `373e943b43e2c17b5a1cad282160e71c4c51de89` | `docs(vc-task-18): add source spec audit` |
| 19 | `f36852c74d5f1d0724514f7ecda0b1a539ab6561` | `docs(vc-task-19): audit bilingual docs sync` |
| 20 | `2f3eb323be8080bf231e1b69dfc9e9e729bb45f9` | `feat(vc-task-20): wire cross-edit reuse identity` |
| 21 | `a8243c3498249fe75d3619fbbe4f5a2dc94b86a2` | `docs(vc-task-21): add architecture 22 follow-up audit` |
| 22 | `76f286f9a3d1e6d6f096b84be7b5f38873e48d42` | `docs(vc-task-22): add module boundary audit` |
| 24 | `c33c583d107c8211c22efcbb89d88144f32d163c` | `docs(vc-task-24): specify kernel evidence handoff` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Module spec、source/spec audit、architecture-22 audit、module-boundary audit、closeout review は unresolved blocking/high specification inconsistency がないことを記録する。 |
| Source behavior documented or deferred | passed | Public module と promised behavior は `source_spec_audit.md` に trace され、unsupported source-derived / downstream behavior は silent に実装せず分類済み。 |
| Milestone-owned coverage | passed | Crate-local Rust test が VC IR validation/rendering、seed intake、generation、normalization、status policy、discharge、evidence、dependency slice、determinism、public enum policy、reuse gating を cover する。 |
| Test expectation integrity | passed | 既存 `.miz` fixture または expectation sidecar は implementation behavior に合わせるため変更していない。Task 15 は fake active proof-verification fixture ではなく corpus gap を記録する。 |
| Design/source synchronization | passed | paired source/spec、bilingual、architecture-22、module-boundary audit は source layout と public module table に同期している。 |
| Boundary discipline | passed | `mizar-vc` は untrusted prover-independent obligation、evidence、slice、reuse candidate だけを生成する。ATP、kernel、proof、cache、artifact、source-extraction ownership は downstream / external のまま。 |
| Verification | passed | Closeout broad command と diff check は commit 前に通過済み。 |
| Residual risk | passed with classified items | 残る risk は下で `external_dependency_gap` または `deferred` として分類する。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 3/5 |
| Total | 94/100 |

減点理由は source-derived proof-verification coverage が利用不能であること、downstream
ATP/kernel/proof/cache consumer が未存在であること、upstream stable payload family が
不完全であること、および大きい implementation file が maintenance watchlist に残ることである。
これらは分類済みで、crate-local milestone の所有 seam ではなく hard gate failure でもないため
score cap はない。

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | closeout report、audit inventory、Task 22 hash backfill、closeout ledger wording を同期後、findings なし。 |
| Test sufficiency review | findings なし。docs-only closeout task には broad workspace verification と diff check で十分であり、新しい Rust、`.miz`、expectation、`doc/spec`、traceability change は不要。 |
| Full implementation review | findings なし。closeout commit は docs-only で source を変更せず、全 task commit、hard gate、residual gap、verification を記録する。 |
| Source/documentation consistency review | findings なし。English canonical docs と Japanese companion は paired で、source/public module boundary は audit と一致し、Task 22 は backfill 済み。 |
| Read-only crate quality review | hard gate は pass し、blocking/high/medium finding はない。Valid quality score は 94/100 で 90 以上。 |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| VC-CLOSEOUT-G001 | `external_dependency_gap` | `mizar-test` には active `proof_verification` runner/tag gate と real `.miz` corpus input 用 source-to-core/source-to-VC extraction seam がまだない。 | Active source-derived VC fixture を有効化する前に、owning staged-test / upstream extraction task で runner と extraction support を追加する。 |
| VC-CLOSEOUT-G002 | `external_dependency_gap` / `deferred` | original closeout は `mizar-kernel` を unavailable と扱っていた。`mizar-kernel` task 23-29 は formula/substitution evidence parsing、deterministic instantiation / SAT encoding、trusted SAT checker wrapping、SAT-backed check service、legacy-certificate audit gating を提供済みである。Task 25 は explicit payload 向け VC producer-side handoff builder を追加する。ATP candidate production、proof/cache consumer、artifact witness consumer、kernel-evidence hash reuse integration はまだ incomplete。 | Task 24 は VC/kernel handoff を仕様化する。task 25 は producer-side builder を実装する。task 26 は canonical kernel evidence hash を reuse identity に含めなければならない。downstream ATP/proof/cache/artifact work は placeholder ではなくそれぞれの spec を使う。 |
| VC-CLOSEOUT-G003 | `external_dependency_gap` | registration/redefinition/reduction details、call precondition、branch/match/range/collection loop obligation、term-only / partial termination、Pick non-emptiness、ghost erasure、complete trace family、source-derived core formula payload、definition payload、quantified binder payload、source-derived obligation payload family の upstream explicit/stable payload は不完全。 | Upstream checker/core/control-flow task が stable explicit payload を expose した後、`mizar-vc` に spec-backed generation/discharge/slice task を追加する。 |
| VC-CLOSEOUT-G004 | `deferred` | Proof-witness hash、ATP/kernel/proof/cache validation、artifact consumer、source-derived runner integration は、architecture-22 reuse を deterministic-discharge candidate key の外で受理する前に必要。 | Downstream proof/cache/artifact phase が、ここで生成する untrusted reusable input を validate する。 |
| VC-CLOSEOUT-G005 | `deferred` | 大きい `vc_ir`、`generator`、`dependency_slice` file は private helper/test split が有益になる可能性があるが、Task 22 は crate exit 前に必須の move-only split はないと判断した。 | reviewability bottleneck が生じた場合だけ、behavior や API change を混ぜず future move-only maintenance task を実施する。 |

`repo_metadata_conflict` は観測されなかった。

## Human Review Surface

- `doc/design/mizar-vc/en/` 配下の英語正本。
- `doc/design/mizar-vc/ja/` 配下の日本語 companion。
- `crates/mizar-vc/src/` 配下の VC source。
- `crates/mizar-vc/tests/` と module-local Rust test。
- `tests/coverage/spec_trace.toml` の deferred proof-verification traceability row。
- Upstream inputs:
  `doc/design/mizar-core/en/crate_exit_report.md`,
  `doc/design/mizar-core/en/core_ir.md`,
  `doc/design/mizar-core/en/control_flow.md`。

## Test Expectation Summary

既存 `.miz` fixture や expectation sidecar は implementation behavior に合わせるために
変更していない。Milestone-owned behavior は Rust unit test、integration test、
lint-policy guard、determinism test、source/spec audit、または explicit deferred
traceability row で cover される。Source-derived semantic corpus coverage は上記の
external runner / extraction gap により blocked のまま。

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | 明示 closeout path staging 後に passed |

Unrun deferred commands:

- `cargo test -p mizar-cache` と `cargo test -p mizar-proof` は、それらの crate が
  workspace に存在しないため dedicated consumer check としては実行していない。現在の workspace
  は broad `cargo test` で cover する。
- Dedicated `mizar-atp` check は同じ理由で未実行である。その crate はこの report
  ではまだ external gap である。`mizar-kernel` は現在存在し、task 23 から task 29 の
  correction commit で独自に verified 済みである。Task 24 は docs-only であり、
  Task 25 は kernel code を呼び出さない。

## Next-Task Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous correction from completed task 25. Before editing,
verify a clean worktree, confirm the task 25 commit in git log, and re-read
doc/design/mizar-vc/en/kernel_evidence_handoff.md,
doc/design/mizar-vc/en/dependency_slice.md,
doc/design/architecture/en/22.incremental_verification_contract.md,
doc/design/architecture/en/18.dependency_fingerprint.md,
crates/mizar-vc/src/kernel_evidence_handoff.rs, and
crates/mizar-vc/src/dependency_slice.rs. Implement task 26 only: extend
dependency-slice and proof-reuse identity to include the canonical kernel
evidence hash produced by the task-25 builder. Keep downstream proof/cache/
artifact schemas external; do not promote a handoff package to proof acceptance
or add placeholder consumers. Add focused Rust tests for hash-driven
invalidation and fail-closed unavailable reuse when kernel evidence or
downstream consumers are absent. Run cargo fmt --check, cargo test -p mizar-vc,
cargo clippy -p mizar-vc --all-targets --all-features -- -D warnings, git diff
--check, and git diff --cached --check after explicit path staging. Use
review-only agents for the required AGENTS.md review phases.
```

Rationale: task 26 は kernel evidence boundary で architecture-22 reuse identity を更新する。
hash は cached/reused proof candidate を invalidate できるが、acceptance material には
なってはならないため `xhigh` を保つ。typo-only documentation synchronization だけなら
lower reasoning が適切である。
