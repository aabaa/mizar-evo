# Crate Exit Report: mizar-proof

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## 結果

Status: `mizar-proof` proof-policy milestone として complete。
Quality score: 94/100。
Score caps applied: none。

Post-closeout metadata correction: task 20 は、formal `crates/mizar-proof` を
placeholder として扱い続ける stale な `mizar-atp` task-28 closeout guard を発見した。
focused correction commit `36d1a9c` が、この report finalization 前に repository
metadata drift を解消した。

## 範囲

Milestone scope:

- `mizar-proof` workspace crate を task 0 から task 19、および本 closeout task
  まで構築する。
- untrusted candidate evidence、明示的な kernel-check output、deterministic
  built-in discharge evidence、externally attested record、policy assumption、
  open obligation に対する proof policy evaluation を所有する。
- deterministic winner selection と artifact-facing proof-selection merge を所有し、
  arrival order、completion time、runtime duration を proof identity に使わない。
- proof status projection を所有する。trusted `used_axioms` は status が
  `Accepted` の `mizar-kernel::checker::KernelCheckResult` からだけ伝播する。
- proof witness staging、manifest-gated publication reference、validation
  predicate としてだけ export される stable proof-reuse metadata を所有する。
- ATP early-stop policy query を所有する。query は class/rank based であり、
  backend progress、diagnostic、cache record、external attestation を trusted
  acceptance にしない。

Included:

- `doc/design/mizar-proof/{en,ja}/` 以下の English/Japanese crate plan、module
  specification、source/spec audit、bilingual sync audit、architecture-22 audit、
  module-boundary audit、本 exit report。
- `crates/mizar-proof/src/` 以下の Rust source。
- `crates/mizar-proof/src/<module>/tests.rs` 以下の private unit tests。
- `crates/mizar-proof/tests/` 以下の crate-local determinism / lint-policy
  integration tests。

Excluded:

- kernel acceptance、SAT solving、ATP backend execution、proof search、premise
  selection、substitution invention、overload resolution、cluster search、
  implicit coercion insertion、fallback inference、cache lookup、artifact
  manifest commit、source-derived corpus extraction。
- backend proof method、resolution trace、SMT proof object、backend log、
  externally attested record、cache record、backend diagnostic を trusted proof
  status または trusted `used_axioms` に昇格させること。
- 未完成の `mizar-cache`、`mizar-artifact`、`mizar-atp` consumer/producer との
  placeholder downstream integration。
- `mizar-atp` による `mizar-proof` API の downstream adoption。commit `36d1a9c`
  は stale ATP closeout metadata だけを補正し、integration wiring は行わない。

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 0 | `a1f6d1dba4ecc46aa2d434c8da8ae0279a2c23ec` | `docs(proof-task-0): add autonomous crate plan` |
| 1 | `bc7d33efd8b7427fa026807e729642219f62f809` | `feat(proof-task-1): scaffold proof policy crate` |
| 2 | `193598cd5db448b17c5cddb721a5e22010b241b6` | `docs(proof-task-2): specify proof policy` |
| 3 | `f3ac274d9cf7266f5ebdc4de94fe963f72ba67d5` | `feat(proof-task-3): implement policy evaluator` |
| 4 | `986f62a184fa8d0a7fa0c98ef8f6c9669d85d844` | `feat(proof-task-4): handle external evidence admission` |
| 5 | `6802bbec9f75d33f6c28b06391d60101726b2d51` | `docs(proof-task-5): specify winner selection` |
| 6 | `9230c36464a58a4f35a43ae1f7dc9fcde6e5e94d` | `feat(proof-task-6): implement winner selection` |
| 7 | `6e9a5a0400aae2c4c8b5b8098594ecc0bd3d2949` | `feat(proof-task-7): merge artifact proof selections` |
| 8 | `76f112237c55dab875cd3e26cbfa11d45439fe5e` | `docs(proof-task-8): specify status projection` |
| 9 | `ccd0e05820f61c02614ac523bf444556c5b29fa5` | `feat(proof-task-9): project proof statuses` |
| 10 | `d77163f8bd1f2c254a7935164e2135567ba9b3a0` | `docs(proof-task-10): specify witness store` |
| 11 | `6d2efc72e9514a44e5f4e81fbbaf7e78ecd73dba` | `feat(proof-task-11): implement witness store` |
| 12 | `b575fd0cf2e176e5c76fd117c56b18142059b997` | `feat(proof-task-12): add portfolio early-stop hooks` |
| 13 | `ba8c696e7b154a6fd9389222334d6ea8f3b5d6d7` | `test(proof-task-13): add determinism suite` |
| 14 | `c3cd67512a460afde3319101504ef45524ccb302` | `test(proof-task-14): guard public enum compatibility` |
| 15 | `7fdfe4945ba885f8d4f6990d023a6ee0aa35744d` | `docs(proof-task-15): audit source spec correspondence` |
| 16 | `f58c9a6203179da1b360024fbc3a071263271c3b` | `docs(proof-task-16): audit bilingual sync` |
| 17 | `f53d6e2e9adc4db740c94f480f49a064662bd190` | `feat(proof-task-17): export proof reuse metadata` |
| 18 | `aaf14d6f83357691d01ea2ce60b8fda99e89ac9c` | `docs(proof-task-18): audit architecture 22 reuse metadata` |
| 19 | `fe0735bc527935532a9ce6038f5597c7a03ecf57` | `refactor(proof-task-19): split private test modules` |
| 20 | pending self-hash | `docs(proof-task-20): add crate exit report` |

## 最終所有 surface

| Surface | Final shape |
|---|---|
| Proof policy | `ProofPolicyEvaluator` は明示的な policy candidate を分類し、policy diagnostic を記録し、policy fingerprint を計算し、external evidence admission を制御し、ATP early-stop query に答える。Accepted kernel input は明示的な `KernelPolicyInput` origin と accepted `KernelCheckResult` を通る場合だけ trusted になる。policy-tainted kernel output は non-trusted である。 |
| Deterministic selection | `select_winner` と artifact merge API は class rank と stable tie-break identity で選択する。raw completion order、runtime、backend timing は proof identity に参加しない。`require_kernel_certificates` は externally attested winner を block する。 |
| Status projection | `project_status` は selected proof evidence を artifact/diagnostic status へ map し、matching hash を持つ accepted kernel evidence からだけ trusted `used_axioms` を伝播する。External、backend diagnostic、cache、rejected、open status は区別された non-trusted のままである。 |
| Witness store | `witness_store` module は `stage` で deterministic witness payload hash を stage し、kernel-verified formula/substitution evidence には unpublished `ProofWitnessRef` candidate を提供し、opaque committed artifact-manifest reachability proof を持つ `publish_ref` を通してだけ committed publication ref を返す。`selected_proof_witness_hash` は selection/status metadata であり、committed publication ref ではない。`DischargedBuiltin` witness publication は artifact schema support が存在するまで unsupported のままである。 |
| ATP early stop | Early-stop decision は policy/class based であり、observed selectable class を要求し、同一または上位の pending selectable class により block される。time や backend partial diagnostic では cancel しない。downstream `mizar-atp` adoption は `external_dependency_gap` のままである。 |
| Proof-reuse metadata | Status metadata は policy fingerprint compatibility、obligation/context/dependency fingerprint、selected witness または deterministic discharge hash、evidence identity、dependency artifact/schema compatibility、selected provenance、stable selection reason、validation hash を export する。この metadata は cache validation predicate であり、proof authority ではない。 |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Paired module spec、source/spec audit、architecture-22 audit、module-boundary audit、closeout review は `mizar-proof` に unresolved blocking/high specification inconsistency がないことを記録する。 |
| Source behavior documented or deferred | passed | Public module、public item、test、promised behavior は `source_spec_audit.md` で trace され、残る cache/artifact/ATP integration は stub ではなく分類されている。 |
| Milestone-owned coverage | passed | Crate-local Rust test は policy classification、external admission、deterministic selection、merge behavior、status projection、trusted used-axiom boundary、witness staging/publication gate、early-stop query、public enum policy、determinism、proof-reuse metadata invalidation を覆う。 |
| Test expectation integrity | passed | 既存 `.miz` fixture、traceability row、expectation sidecar を implementation behavior に合わせるために変更していない。この crate は upstream data shape 上の Rust policy/projection/store behavior を所有するため `.miz` test は追加していない。 |
| Design/source synchronization | passed | Paired source/spec、bilingual、architecture-22、module-boundary audit は source layout と public module table に一致する。 |
| Boundary discipline | passed | `mizar-proof` は記録、選択、射影、stage、metadata export を行う。proof acceptance、SAT/ATP backend 呼び出し、proof search、cache lookup、artifact commit、external/cache/backend material の trusted status 昇格は行わない。 |
| Verification | passed | focused ATP metadata correction 後に、`mizar-proof`、隣接 kernel/VC/artifact/checker/ATP tests、full clippy、fmt、diff checks、full workspace `cargo test` は passed。 |
| Residual risk | classified item 付きで passed | 残る risk は下の `external_dependency_gap` または `deferred` として記録する。未解決の `repo_metadata_conflict` は残っていない。 |

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

この score は unavailable downstream cache/artifact/ATP integration、deferred な
`DischargedBuiltin` artifact witness publication、downstream production token をまだ必要とする
copy 済み kernel/artifact acceptance metadata、producer-owned の byte-level witness payload
canonicality validation、missing active source-derived proof-verification corpus coverage、
private-test split 後も残る large production module、future downstream handoff work に対して
減点する。これらは分類済みでありこの crate の proof-policy ownership 外であるため score cap
にはならない。full workspace verification は clean に通っている。

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | Task-20 review は `36d1a9c` 後の stale ATP metadata-conflict wording と、witness metadata/canonicality gap の closeout carry-forward 欠落を最初に指摘した。focused update 後の final re-review は no findings。 |
| Test sufficiency review | Task-20 review は docs-only closeout に追加 proof test は不要と判断した。full verification と ATP metadata correction は passed。 |
| Full implementation review | Task-20 review は docs 側の witness metadata wording drift を指摘したが、production API / behavior change は不要だった。focused docs update で解消した。 |
| Source/documentation consistency review | Task-20 review は stale closeout wording と `ProofWitnessStore` type-name overclaim を最初に指摘した。EN/JA docs synchronization 後の focused re-review は no findings。 |
| Read-only crate quality review | Valid quality score: 94/100。Hard gates は blocking/high finding なしで pass し、未解決 repo metadata conflict はない。 |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| PROOF-CLOSEOUT-G001 | `external_dependency_gap` | `mizar-cache` は現在存在し proof-reuse validation を所有するが、この proof-policy closeout は cache lookup、hit/miss decision、cache promotion API を呼ばない。 | `mizar-proof` metadata は validation input としてだけ保つ。proof/cache consumer wiring は別の integration task で追加する。 |
| PROOF-CLOSEOUT-G002 | `external_dependency_gap` | downstream `mizar-atp` による early-stop query adoption と live backend cancellation は未配線。 | 別 task で `mizar-atp` が policy API を呼ぶ。policy / winner selection を ATP へ移さない。 |
| PROOF-CLOSEOUT-G003 | `external_dependency_gap` | Artifact committed publication token は `mizar-proof` では意図的に opaque。real phase-15 emission と manifest transaction integration は downstream のまま。 | `mizar-artifact` / emitter task が committed reachability proof と exact witness coverage を供給する。 |
| PROOF-CLOSEOUT-G004 | `external_dependency_gap` | `mizar-artifact` は現在 trusted `DischargedBuiltin` witness publication support を持たない。 | built-in discharge witness ref を publish する前に owning task で artifact witness schema を拡張する。 |
| PROOF-CLOSEOUT-G005 | `deferred` | source-derived proof-verification corpus と producer extraction は、この policy/projection crate では unavailable。 | `.miz` proof-policy fixture を追加する前に source runner と producer contract を追加する。 |
| PROOF-CLOSEOUT-G006 | `deferred` | task 19 後も production module は 1,100 lines を超えるが、review risk を増やさず complexity を下げるより小さい production helper boundary は見つかっていない。 | quality review または downstream work が concrete bottleneck を見つけた場合だけ後続 move-only refactor を行う。 |
| PROOF-CLOSEOUT-G007 | resolved `repo_metadata_conflict` | `mizar-atp` task-28 closeout guard は以前、workspace member `crates/mizar-proof` と `crates/mizar-proof` directory を forbidden placeholder として reject していた。 | Focused metadata correction commit `36d1a9c` で解消済み。proof policy は `mizar-atp` に移していない。 |
| PROOF-CLOSEOUT-G008 | `external_dependency_gap` | `TrustedKernelWitnessMetadata` は、kernel/artifact boundary が copy 済み kernel acceptance metadata を公開するまで opaque で production constructor を持たない。 | trusted witness draft は artifact/kernel-owned metadata を待つ。caller-synthesized acceptance metadata は受理しない。 |
| PROOF-CLOSEOUT-G009 | `deferred` | witness store は exact payload bytes と schema identity を hash するが、byte-level canonicality validation は concrete payload schema が validator を公開するまで producer-owned のままである。 | hash/attestation contract を stable に保つ。validator は producer-owned schema task でだけ追加する。 |

## Human Review Surface

- `doc/design/mizar-proof/en/` 以下の canonical English docs。
- `doc/design/mizar-proof/ja/` 以下の Japanese companion。
- `crates/mizar-proof/src/` 以下の proof source。
- `crates/mizar-proof/tests/` 以下の proof tests と
  `crates/mizar-proof/src/<module>/tests.rs` 以下の private module tests。
- Upstream/downstream context:
  `doc/design/mizar-kernel/en/crate_exit_report.md`,
  `doc/design/mizar-vc/en/crate_exit_report.md`,
  `doc/design/mizar-atp/en/crate_exit_report.md`,
  `doc/design/mizar-artifact/en/todo.md`,
  `doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md`,
  `doc/design/internal/en/07.crate_module_layout.md`,
  `doc/design/architecture/en/08.reasoning_boundary.md`,
  `doc/design/architecture/en/15.kernel_certificate_format.md`,
  `doc/design/architecture/en/19.failure_semantics.md`,
  `doc/design/architecture/en/22.incremental_verification_contract.md`。

## Test Expectation Summary

既存 `.miz` fixture や expectation sidecar を implementation behavior に合わせる
ために変更していない。Milestone-owned behavior は Rust unit tests、integration
tests、lint-policy guards、determinism tests、source/spec audit、architecture-22
audit、module-boundary audit、または explicit deferred gap record で覆われている。
Source-derived semantic corpus coverage は上記の producer / runner gap により
引き続き block される。

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test -p mizar-proof` | passed |
| `cargo test -p mizar-kernel` | passed |
| `cargo test -p mizar-vc` | passed |
| `cargo test -p mizar-artifact` | passed |
| `cargo test -p mizar-checker` | passed |
| `cargo test -p mizar-atp` | focused metadata correction commit `36d1a9c` 後に passed |
| `cargo test` | focused metadata correction commit `36d1a9c` 後に passed |
| `git diff --check` | passed |
| `git diff --cached --check` | explicit task-20 path staging 後に passed |

Unrun deferred commands:

- original closeout では `mizar-cache` がまだ workspace crate ではなかったため、
  `cargo test -p mizar-cache` は実行していない。roadmap sync は cache が現在存在する
  ことを記録する。将来の proof/cache integration task では専用 cache check を実行する。

## Next-Phase Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Start the next evidence-pipeline integration phase after the mizar-proof task
20 closeout commit exists and `36d1a9c` is in HEAD history. Keep `mizar-proof`
as proof-policy/winner/status/witness/reuse owner. A good next task is a
focused integration task, such as wiring `mizar-cache` proof-reuse validation
into an owning cache/driver path or wiring `mizar-atp` to consume existing
early-stop policy APIs without moving policy or winner selection into ATP. Do
not add placeholder cache behavior、artifact witness publication、kernel
acceptance、SAT solving、backend proof trust、proof search。Run the AGENTS.md
review phases and full workspace verification before committing.
```

Rationale: downstream integration は proof/cache/artifact/ATP ownership boundary を
またぎ、trusted acceptance が kernel result だけに由来する規則を保つ必要がある。
`xhigh` を維持する。integration contract を変えない docs-only typo synchronization
だけなら lower でよい。
