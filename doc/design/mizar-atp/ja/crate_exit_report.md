# Crate Exit Report: mizar-atp

> 正本言語: 英語。英語正本:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: 現在の candidate-evidence producer milestone として完了。
Quality score: 94/100。
Score caps applied: none。

## Scope

Milestone scope:

- task 1 から task 28 までの `mizar-atp` workspace crate を構築する。
- `VcStatus::NeedsAtp` obligation 向け phase 13 candidate production を所有する:
  backend-neutral `AtpProblem` construction、deterministic TPTP/SMT-LIB
  encoding、generic backend process execution、mock candidate classification、
  no-early-stop portfolio candidate collection、run metadata、deterministic
  candidate handoff。
- この crate から出る kernel-checkable proof evidence は untrusted
  formula/substitution candidate payload だけである。Counterexample と externally
  attested record は untrusted diagnostic または policy-facing record に留まり、trusted
  acceptance material ではない。
- proof acceptance、SAT checking、kernel call、proof policy、winner selection、
  witness publication、proof-cache promotion、artifact writing、source-derived
  corpus extraction、利用できない real-backend extraction は deferred または
  external dependency gap として分類する。

Included:

- `doc/design/mizar-atp/{en,ja}/` 配下の英語/日本語 crate plan、module spec、
  source/spec audit、bilingual sync audit、module-boundary audit、この exit report。
- `crates/mizar-atp/src/` 配下の Rust source と private unit-test module。
- `crates/mizar-atp/tests/` 配下の crate-local lint、mock-backend corpus、
  determinism test。
- `tests/property/` 配下の metadata-only advanced-semantics corpus fixture と
  `tests/coverage/spec_trace.toml` の traceability row。

Excluded:

- `doc/spec` への直接編集。
- 既存 `.miz` fixture または expectation sidecar の rebaseline。
- real backend adapter、backend-output parser、backend-specific
  formula/substitution candidate extraction。
- backend proof method、resolution trace、SMT proof object、backend log、
  instantiated formula、caller-supplied SAT problem からの trusted acceptance。
- kernel checking、trusted SAT checking、proof search、premise selection、
  substitution invention、overload resolution、cluster search、implicit
  coercion insertion、fallback inference、proof-policy winner selection、
  artifact witness publication、proof-cache promotion。
- placeholder `mizar-proof` または `mizar-cache` crate。これらは design TODO を
  持つが、この milestone では workspace member ではない。

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 1 | `402020d2f8166c7ede5af6d4518711e3fdbba402` | `feat(atp-task-1): scaffold candidate evidence crate` |
| 2 | `9da4f2a6676937249035c19d3312574e5ea10334` | `docs(atp-task-2): specify backend-neutral problems` |
| 3 | `77a189282d5fccb8219f0070700ed933c5d08ca1` | `feat(atp-task-3): implement backend-neutral problems` |
| 4 | `3efa26876bf500248a8e588c0534a01cdec08756` | `docs(atp-task-4): specify vc translator` |
| 5 | `5ea8d2e823c90b739492b9bb7c72ea1b34a43d5a` | `feat(atp-task-5): translate declarations` |
| 6 | `6f00b61fd960ded7dc79db9ee5c3274fc5431425` | `feat(mizar-atp-task-6): translate axioms and conjectures` |
| 7 | `3bcd6ccee921c2325dac412d151493aeb97f8186` | `docs(mizar-atp-task-7): specify property encoding` |
| 8 | `975323520f65a35299eca645aca3060eb92da549` | `feat(mizar-atp-task-8): encode property axioms` |
| 9 | `06c42fff17c7cac070345eb1fb532669d26fbb6d` | `docs(mizar-atp-task-9): specify tptp encoder` |
| 10 | `0fe5bd55e62e1c7261097a2f6b06ae06149d5242` | `feat(mizar-atp-task-10): emit deterministic tptp fof` |
| 11 | `6af92de8fbe97157901e93cd69af9b43ccfa955c` | `docs(mizar-atp-task-11): specify smtlib encoder` |
| 12 | `90e96e87cdca02b670a92d710a212333759263d2` | `feat(mizar-atp-task-12): emit deterministic smtlib` |
| 13 | `2e77fa8854595756393fe93b10fc65731c40d945` | `docs(mizar-atp-task-13): specify backend runner` |
| 14 | `2f4f91f6c845c21dfb953eb11107b65b1457faa0` | `feat(mizar-atp-task-14): add backend runner` |
| 15 | `16582227516cfe961afdca6072aae4f9e7c9e6e5` | `docs(mizar-atp-task-15): defer concrete backend route` |
| 16 | `c3abdaab8f5e8ae9d61603199ea1ef83e392fb5a` | `docs(mizar-atp-task-16): defer real-output classification` |
| 17 | `4263b63e3c1f888a51531f593a39a197fb1649b4` | `docs(mizar-atp-task-17): specify portfolio handoff` |
| 18 | `e4e71979c928ebafcd41eaf6667c09b67a26e450` | `feat(mizar-atp-task-18): collect portfolio candidates` |
| 19 | `8cef987e782a7a4a46e853bc0b9bd42950729081` | `feat(mizar-atp-task-19): record backend run metadata` |
| 20 | `3b9d07fc619d324b021c10aba8467183924a7aa4` | `test(mizar-atp-task-20): add mock backend corpus suite` |
| 21 | `4d18e3f7de6f90600861d96c29e9cba54dfeecca` | `test(mizar-atp-task-21): add determinism suite` |
| 22 | `4269f43240bd4a5de003a695acebd7b894544f85` | `docs(mizar-atp-task-22): record enum forward compatibility` |
| 23 | `d3be92b92b9de8e810f6a31d2e0836d0e54561f5` | `docs(mizar-atp-task-23): add source spec audit` |
| 24 | `9d053bb129bc846733c430e2fafac44eb1c2c89b` | `docs(mizar-atp-task-24): add bilingual sync audit` |
| 25 | `93a480bde30dbb12014681ba1bac2300587b5a06` | `docs(mizar-atp-task-25): defer portfolio order gate` |
| 26 | `5dc6ab594c2ced35d8afc74e2c01707929a569d6` | `docs(mizar-atp-task-26): audit architecture order gate` |
| 27 | `f896d48f3ea4f915084343a4f88007b77f9941cb` | `refactor(mizar-atp-task-27): split private test modules` |
| 28 | pending self-hash | `docs(mizar-atp-task-28): add crate exit report` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | paired module spec、source/spec audit、bilingual sync audit、module-boundary audit、closeout review は unresolved blocking/high specification inconsistency を記録しない。 |
| Source behavior documented or deferred | passed | public module、public entry function、public enum、promised behavior は `source_spec_audit.md` で trace され、利用できない real-backend/downstream behavior は mock されず分類される。 |
| Milestone-owned coverage | passed | crate-local test は problem validation、translation、property encoding、concrete encoder、backend runner behavior、portfolio collection、metadata projection、public enum policy、mock corpus coverage、determinism を cover する。 |
| Test expectation integrity | passed | 既存 `.miz` fixture または expectation sidecar を実装挙動に合わせて変更していない。active source-derived ATP execution は deferred のままである。 |
| Design/source synchronization | passed | source/spec、bilingual、Architecture-22 follow-up、module-boundary audit は source layout と public module table に一致する。 |
| Boundary discipline | passed | `mizar-atp` は untrusted candidate だけを生産する。kernel call、SAT checking、proof acceptance、trusted winner selection、witness publication、cache promotion を行わない。 |
| Verification | passed | crate-local Rust command、broad workspace command、diff check は closeout commit 前に成功した。 |
| Residual risk | passed with classified items | 残る risk は下の `external_dependency_gap` または `deferred` として記録する。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

この score は、意図的に deferred とした real-output extraction route、active
source-derived ATP corpus execution の不足、proof/cache/artifact consumer の不在により
減点する。これらは現在の crate milestone の外であり、明示的に分類済みであるため score
cap は適用しない。

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | task 27 について paired module-boundary docs と audit が同期された後、findings なし。 |
| Test sufficiency review | task-27 doc marker check が移動済み test module のうち二つだけを見ていた low finding は、七つすべてを guard することで修正した。最終再レビューは findings なし。 |
| Full implementation review | findings なし。task-27 split は cfg gate、include path、public API、trust boundary を保持する。 |
| Source/documentation consistency review | findings なし。source layout、EN/JA docs、lint-policy expected file set は一致する。 |
| Read-only crate quality review | hard gate は blocking/high finding なしで pass。Valid quality score: 94/100、90 以上。 |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| ATP-CLOSEOUT-G001 | `external_dependency_gap` | concrete backend output を kernel-parseable formula/substitution candidate payload に map する paired real-output extraction spec/source module がなく、この環境で検証済みの supported real backend executable route もない。 | task 15-16 を再開する前に、backend-specific EN/JA extraction spec、guarded fixture、backend proof material を除外する candidate mapping を追加する。 |
| ATP-CLOSEOUT-G002 | `external_dependency_gap` | `mizar-proof` は design-only で workspace crate ではないため、release-policy finality、deterministic winner selection、early-stop authority、proof-status projection、witness selection の implementation owner がない。 | 明示的に許可された後でのみ `mizar-proof` を自身の TODO に従って scaffold / complete する。proof policy を `mizar-atp` に追加しない。 |
| ATP-CLOSEOUT-G003 | `external_dependency_gap` | `mizar-cache` は design-only で workspace crate ではないため、proof-reuse validation、cache lookup、policy-compatible reuse は利用できない。 | 明示的に許可された後でのみ `mizar-cache` を自身の TODO に従って scaffold / complete する。cache reuse は evidence を upgrade してはならない。 |
| ATP-CLOSEOUT-G004 | `external_dependency_gap` / `deferred` | real artifact witness publication と proof/cache consumer integration は未完成である。`mizar-artifact` は witness schema/projection を所有し、ATP acceptance は所有しない。 | downstream proof/cache/artifact owner が、それぞれの spec で checked kernel evidence と published witness ref を接続する。 |
| ATP-CLOSEOUT-G005 | `deferred` | active `.miz` advanced-semantics execution と source-derived ATP extraction は利用できない。task 20 は metadata-only corpus anchor を使う。 | staged runner と source extraction contract を追加してから、metadata-only coverage を active corpus coverage に置き換える。 |
| ATP-CLOSEOUT-G006 | `deferred` | Typed TPTP/CNF/include route、SMT arithmetic/sorted signature/option/proof command、native declaration、backend-native shortcut は現在の spec で未対応である。 | 各 concrete extension に paired spec と test を追加してから実装する。 |

`repo_metadata_conflict` は観測されなかった。

## Human Review Surface

- `doc/design/mizar-atp/en/` 配下の英語正本。
- `doc/design/mizar-atp/ja/` 配下の日本語 companion。
- `crates/mizar-atp/src/` 配下の ATP source。
- `crates/mizar-atp/tests/` と module-local Rust test。
- `tests/property/` の metadata-only corpus fixture と
  `tests/coverage/spec_trace.toml` の traceability row。
- upstream/downstream context:
  `doc/design/mizar-kernel/en/crate_exit_report.md`,
  `doc/design/mizar-vc/en/crate_exit_report.md`,
  `doc/design/mizar-artifact/en/todo.md`,
  `doc/design/mizar-proof/en/todo.md`,
  `doc/design/mizar-cache/en/todo.md`,
  `doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md`。

## Test Expectation Summary

既存 `.miz` fixture または expectation sidecar を実装挙動に合わせて変更していない。
Milestone-owned behavior は Rust unit test、integration test、lint-policy guard、
determinism test、mock-backend corpus test、source/spec audit、明示的な deferred gap
record により cover される。Active source-derived semantic corpus coverage は、上記の
external runner と extraction gap により blocked のままである。

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo test -p mizar-atp --test lint_policy --offline` | passed |
| `cargo test -p mizar-atp --offline` | passed |
| `cargo clippy -p mizar-atp --all-targets --all-features --offline -- -D warnings` | passed |
| `cargo clippy --all-targets --all-features --offline -- -D warnings` | passed |
| `cargo test --offline` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | 明示的な task-28 path staging 後に passed |

Unrun deferred commands:

- `cargo test -p mizar-proof` と `cargo test -p mizar-cache` は、これらの crate が
  workspace member ではないため実行していない。Design TODO は external dependency gap
  として記録し、placeholder は追加しない。

## Next-Phase Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
mizar-atp task 28 closeout commit が存在する状態から evidence-pipeline correction を
継続してください。まず `git status --short` が clean であること、および
`f896d48f3ea4f915084343a4f88007b77f9941cb` と task-28 closeout commit が HEAD 履歴に
存在することを確認してください。次 phase が scaffold work を明示的に許可しない限り、
placeholder `mizar-proof` / `mizar-cache` crate は作らないでください。proof-policy winner
selection、proof-cache validation、real backend output extraction、real artifact witness
publication、active source-derived ATP corpus execution は、owner spec と workspace crate が
存在するまで external_dependency_gap / deferred として扱ってください。
```
