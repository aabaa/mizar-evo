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
  ghost-erasure VC、trace、source-derived core formula、definition、quantified
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
artifact witness consumer と、その時点の VC reuse-identity work は後続 task または
external gap のままである。下の Task 26 は VC reuse-identity 部分だけを解消する。この task は
docs-only であり、Rust source、kernel call、SAT solving、backend encoding、legacy
certificate、捏造した formula/substitution/provenance payload を追加しない。

Task 25 は explicit payload 向け producer-side `kernel_evidence_handoff` Rust module を
追加する。immutable formula/substitution/provenance handoff package と canonical hash input
を構築し、imported context requirement と discharge diagnostic を canonical envelope の外に
保ち、不足 payload では fail closed する。引き続き kernel call、SAT solving、legacy
certificate acceptance、backend proof method encoding、artifact/cache proof status publication は
行わない。

Task 26 は、その producer-side handoff に対する dependency-slice / proof-reuse identity
integration を追加する。現在の canonical handoff hash は slice fingerprint と reuse candidate
key に参加し、current handoff がない legacy reuse は fail closed する。これは identity /
invalidation boundary のみであり、VC output を trusted proof material にせず、kernel call、
SAT checking、proof/cache/artifact consumer を追加しない。

Task 27 は producer-side handoff に explicit goal polarity を追加し、canonical package assembly
前に現在の各 proof-obligation VC kind に対する consistency polarity を拒否する。これは F1 polarity
boundary の producer half だけを閉じる。trusted checker-side acceptance binding は
`mizar-kernel` が所有する。

Task 28 は local-hypothesis、cited-premise、generated-VC-fact formula evidence binding 向けの
producer-side `context_identity` row と hash を追加する。この payload は dependency-slice と
proof-reuse identity に参加し、imported fact は context identity の外に残る。kernel-side
membership verification は `mizar-kernel` task 31 が所有する。

Task 29 は imported axiom/theorem formula evidence 向けの producer-side imported-statement
projection payload を追加する。handoff と dependency slice は architecture-18
imported-statement fingerprint から kernel formula-tree fingerprint への projection を運び、
unsupported、stale、mismatched、empty/noncanonical projection data を拒否する。kernel-side
trusted validation と pass fixture は `mizar-kernel` task 33 で実装済みである。

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
| 25 | `0ed1bc23e2bc7f66d2f4f53a8e289721d47105b9` | `feat(vc-task-25): add kernel evidence handoff builder` |
| 26 | `9c86900451068553a8e96938c420872b047c1d62` | `feat(vc-task-26): include kernel evidence in reuse identity` |
| 27 | `2d167bde40ccf7788b6de49cc9e324e7e7879987` | `feat(vc-task-27): require explicit handoff goal polarity` |
| 28 | `ab23833f70f3e8a0733621453e283246c1b5b7d1` | `feat(vc): add kernel context identity payload` |
| 29 | `83ff33edda6c308018d0d499259631c9160708d3` | `feat(vc-task-29): add imported statement projection handoff` |

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
| VC-CLOSEOUT-G001 | `external_dependency_gap` | Task 31 は one exact source-derived `proof_verification` runner/tag gate と VC baseline を実装済みだが、general coverage はまだない。 | VC Task 31 の最初の exact `MT10-VC-T180` route は complete で、後続 VC 32-55 は各 real `MT10-VC-PV/VC<n>` increment を所有する。empty runner は禁止。 |
| VC-CLOSEOUT-G002 | `external_dependency_gap` / `deferred` | original closeout は `mizar-kernel` を unavailable と扱っていた。`mizar-kernel` task 23-29 は formula/substitution evidence parsing、deterministic instantiation / SAT encoding、trusted SAT checker wrapping、SAT-backed check service、legacy-certificate audit gating を提供済みである。Tasks 25-29 は explicit data 向け VC producer-side handoff builder、canonical evidence identity、explicit goal polarity、context identity、imported-statement projection payload を追加し、paired `mizar-kernel` task 33 は kernel-side projection validation を実装済みである。ATP candidate production、proof/cache consumer、artifact witness consumer はまだ incomplete。 | Task 24 は VC/kernel handoff を仕様化する。tasks 25-29 は producer-side identity/projection payload を実装する。downstream ATP/proof/cache/artifact work は placeholder ではなくそれぞれの spec を使う。kernel-side task 33 validation は deferred ではなくなった。 |
| VC-CLOSEOUT-G003 | `external_dependency_gap`; VC 53 は bounded `spec_gap` | Source-derived theorem/formula/context、definition/property/term、registration/trace/direct-template、Chapter-20 VC/zero-VC integration payload は不完全。別途、VC 53 には canonical evidence-transport authority がない。 | Task 30 が exact VC 31-55 owner と Core 31-53 dependency を割り当てる。VC 40 は complete 済み VC 37/39 output と Core 40/A1。VC 53 は canonical authority が evidence producer/reference identity/schema、authentication contract/rule、owning test を命名していないことにより blocked のまま。それらを捏造しない。missing scheme/theorem role は direct VC 41 の外で S1 の背後に残る。dependency-ready な bounded row だけを real consumer と共に実装する。 |
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

- `cargo test -p mizar-cache` と `cargo test -p mizar-proof` は、original closeout では
  それらの crate がその milestone context に存在しなかったため dedicated consumer check
  としては実行していない。roadmap sync は現在それらが存在することを記録する。将来の
  VC/proof/cache integration task がその seam に触れる場合は dedicated consumer check を実行する。
- Dedicated `mizar-atp` check は original closeout では同じ理由で未実行だった。roadmap
  sync は現在 `mizar-atp` も存在することを記録する。`mizar-kernel` は現在存在し、task 23 から task 29 の
  correction commit で独自に verified 済みである。Task 24 は docs-only であり、
  Task 25-26 は kernel code や SAT solver を呼び出さない。

## Draft Next-Crate Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
After the mizar-vc task 26 commit exists, continue the evidence-pipeline
crate-suite correction with mizar-artifact task 23. Before editing, verify a
clean worktree, confirm the task 26 commit in git log, and re-read
doc/design/mizar-artifact/en/todo.md,
doc/design/mizar-artifact/en/source_spec_audit.md,
doc/design/mizar-artifact/en/crate_exit_report.md if present,
doc/design/mizar-vc/en/kernel_evidence_handoff.md,
doc/design/mizar-vc/en/dependency_slice.md, and
doc/design/architecture/en/15.kernel_certificate_format.md. Start
mizar-artifact task 23 only: update the kernel evidence proof-witness schema
so artifact records project formula/substitution evidence handoff identity and
kernel-check results without becoming the checker. Do not implement placeholder
producers, proof/cache consumers, SAT solving, ATP backends, resolution traces,
or trusted backend proof-method acceptance. Re-evaluate artifact task 17 only
after task 23 is complete; if required producer/proof outputs are still
missing, record `external_dependency_gap` / `deferred` instead of stubbing.
Run the artifact crate's task-appropriate docs/source verification and use
review-only agents for the required AGENTS.md review phases.
```

Rationale: artifact task 23 は corrected kernel/vc evidence handoff の直下にある
witness-schema boundary である。artifact は stable witness projection を公開しつつ
acceptance oracle になってはならないため `xhigh` を保つ。typo-only documentation
synchronization だけなら lower reasoning が適切である。

## VC Task 30 STEP 5 ownership addendum

post-closeout Task 30 は original crate exit を reopen/weakening しない。paired
[source-derived VC decomposition](./source_vc_decomposition.md) が umbrella
source-derived gap を exact Task-31 mapping と bounded VC Tasks 32-55 に置き換える。
Task 31 と `MT10-VC-T180` は complete である。Tasks 32-55 は Core 33-53 により
dependency-paced、VC 40 は VC 37/39 と Core 40/A1、
VC 53 は bounded canonical-authority gap により blocked、missing scheme/theorem role は
direct VC 41 の外で S1 の背後に残る。
proof search、discharge、ATP/kernel/proof policy、acceptance、cache/artifact reuse、
MVM/extraction、Steps 6/7 はこの ownership-only update の外側に残る。source、fixture、
expectation、trace status/test、coverage、quality score を変更しない。

## VC Task 31 post-closeout implementation addendum

Task 31 は original crate-exit score を変えず exact contradiction slice を実装する。
new public exact adapter、private generator leaf、proof-verification runner、distinct
source/sidecar、complete VcIr baseline、covered trace row 1件は `MT10-VC-T180` だけを
閉じる。old type-elaboration Task-180 baseline は不変である。broad proof-verification
row は empty tests の deferred を保ち、prospective Task-31 wording だけを completed exact
exception に同期する。

adapter は marker-free、open、undischarged、unaccepted のままで、anchor が
`CanonicalGoalHash` を欠くため proof-reuse-ineligible である。VC 32-55、VC-40/VC-53
gate、Steps 6/7、ATP/kernel/proof/cache/artifact execution、general source-to-VC coverage
はこの addendum の外である。
