# Bilingual Documentation Sync Audit: mizar-atp

> 正本言語: 英語。英語正本:
> [../en/bilingual_sync_audit.md](../en/bilingual_sync_audit.md)。

Task 24 は source/spec correspondence audit 後の `mizar-atp` design documentation
pair を監査する。Task 25 は portfolio completion-order independence gate deferral
について、この audit record を更新する。Task 26 は Architecture-22 follow-up について
sync audit を再実行する。これらの audit edit は Rust production source、public API、
`.miz` fixture、expectation、language specification、backend route、kernel check、
proof policy、artifact witness、cache behavior、downstream integration を変更しない。

## Scope And Method

監査対象は `doc/design/mizar-atp/en/` 配下の現在の Markdown document と、
`doc/design/mizar-atp/ja/` 配下の companion の全ペアである。各ペアについて次を確認した:

- 両 language directory に同じ filename が存在すること;
- module responsibility、inputs/outputs、behavior rule、candidate-evidence status、
  deterministic ordering、proof/trust boundary、planned tests、public enum inventory、
  audit inventory、TODO task wording、follow-up classification の実質的な意味が
  同期していること;
- 既知の `external_dependency_gap` と `deferred` record が、黙って解消、弱化、拡大
  されず保持されていること;
- trusted acceptance、backend proof material、resolution trace、SMT proof object、
  caller-supplied SAT problem payload、proof policy、artifact witness publication、
  proof-cache promotion、placeholder downstream integration を新たに追加していないこと。

日本語 companion は自然な翻訳を使ってよく、Rust identifier、phase name、schema name、
task name は英語のままでもよい。同期規則は semantic なものであり、companion は英語
正本に対して normative な意味を省略、弱化、追加してはならない。

結果: 現在の document pair はすべて存在し、意味内容は同期している。bilingual drift、
欠けている companion、古い status、未分類の `design_drift`、`repo_metadata_conflict` は
観測されなかった。まだ利用できない挙動は
[source_spec_audit.md](./source_spec_audit.md) に記録済みの分類済み external/deferred work
である。

## Pair Inventory

| Document | 確認した同期内容 | 結果 |
|---|---|---|
| `00.crate_plan.md` | Crate responsibility、authority order、design/source inventory、known gap、task 26 までの task decomposition、hard gate、verification expectation。 | 同期済み。 |
| `problem.md` | Backend-neutral `AtpProblem` data shape、logic profile、formula/provenance/type-guard ownership、deterministic identity、禁止される trusted material、planned tests、public enum inventory。 | 同期済み。 |
| `translator.md` | explicit `VcIr` / kernel-handoff projection input、declaration/formula materialization、unsupported premise class の fail-closed handling、proof-hint non-pruning、deterministic ordering、planned tests、public enum inventory。 | 同期済み。 |
| `property_encoding.md` | axiom-form property projection、generated binder row、provenance / symbol-map requirement、native-declaration deferral、planned tests、public enum inventory。 | 同期済み。 |
| `tptp_encoder.md` | deterministic FOF emission、label / symbol metadata、name mangling、unsupported typed/native/backend route、planned tests、public enum inventory。 | 同期済み。 |
| `smtlib_encoder.md` | deterministic uninterpreted SMT-LIB emission、fixed universe sort、assertion metadata、unsupported theory/native/backend route、planned tests、public enum inventory。 | 同期済み。 |
| `backend.md` | generic backend runner、command fingerprint、resource limit、run metadata、candidate-evidence-only `Proved`、禁止される trusted backend material、failure semantics、public enum inventory。 | 同期済み。 |
| `portfolio.md` | policy-neutral planning、no-early-stop collection、candidate / evidence-set ordering、fail-closed result matching、downstream proof-policy boundary、determinism suite、task-25 deferred completion-order gate、public enum inventory。 | 同期済み。 |
| `source_spec_audit.md` | public module export、public surface inventory、cross-module evidence、task-25 G005 を含む ATP-AUDIT gap register、task-26 Architecture-22 follow-up result、`ProofWitnessRef` / `VerifiedArtifact` artifact-surface acknowledgement、source/spec drift なしの分類。 | 同期済み。 |
| `bilingual_sync_audit.md` | audit scope、method、pair inventory、classification、task-24 / task-25 / task-26 sync edits、remaining external/deferred work。 | この paired audit document により同期済み。 |
| `todo.md` | ordered task list、task 26 までの完了 task、deferred task 15/16 status、public enum task status、source/spec audit status、task-25 dependency-gap status、task-26 follow-up-audit status、次 task 27 audit wording、verification expectation。 | 同期済み。 |

## Classification

Task 24 と task-26 re-run は新しい `spec_gap`、`test_gap`、`design_drift`、`source_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、
`repo_metadata_conflict`、bilingual drift を記録しない。既存の分類済み record は残る:

- `external_dependency_gap`: concrete backend output を kernel-parseable
  formula/substitution candidate payload に map する paired real-output extraction
  spec/source module はなく、verification environment で利用できる supported real backend
  route もない。
- `external_dependency_gap`: `mizar-artifact` は既に formula/substitution kernel evidence
  向け `ProofWitnessRef` schema version `2.0` と `VerifiedArtifact` witness-reference
  validation を所有するが、real ATP producer output、proof-policy selection、
  proof-cache integration、real artifact witness publication は external のままである。
  `mizar-proof` と `mizar-cache` はこの workspace では design-only である。
- `deferred`: active `.miz` advanced-semantics execution と source-derived ATP extraction は、
  現在の metadata-only corpus fixture の外に残る。
- `deferred`: TPTP typed/CNF/include path、SMT arithmetic/sorted signature、solver
  option、proof command、native declaration、backend-native shortcut は、paired spec と
  test が存在するまで利用不可のままである。
- `external_dependency_gap`: task 25 は portfolio early-stop finality と winner
  selection を再評価する。これらは downstream proof policy に依存し、raw backend
  completion order は proof identity の外に残る。proof-policy oracle や placeholder
  `mizar-proof` adapter は導入しない。

## Task 24 Sync Edits

この task は paired bilingual sync audit document を追加し、paired TODO file で task 24
を完了にし、paired crate plan に task-24 status を記録し、
`crates/mizar-atp/tests/lint_policy.rs` に bilingual audit guard を追加する。

他の paired content に同期編集は不要だった。この audit は real backend adapter、
backend-output parser、kernel call、proof policy、witness writer、cache promotion、
placeholder `mizar-proof` / `mizar-cache` crate、trusted backend proof material を追加しない。

## Task 25 Sync Edits

Task 25 は portfolio completion-order independence gate を
deferred/external_dependency_gap re-evaluation としてだけ完了にする。paired TODO、
crate plan、portfolio spec、source/spec audit、この bilingual audit は、release-policy
winner/early-stop gate が実際の `mizar-proof` policy owner を必要とすることを記録する。
これらの edit は mock proof-policy oracle、placeholder proof-policy adapter、accepted proof
state、kernel call、witness/cache output、trusted backend proof material を追加しない。

## Task 26 Sync Edits

Task 26 は Architecture-22 follow-up について bilingual sync audit を再実行する。paired
TODO、crate plan、source/spec audit、この bilingual audit は、Architecture 22 が backend
completion order と runtime duration を semantic proof identity にすることを禁じることを
記録する。この re-run では bilingual drift、古い task status、`repo_metadata_conflict`、
新しい follow-up gap は見つからなかった。実際の `mizar-proof` policy owner が存在するまで、
ATP-AUDIT-G005 は単一の policy-boundary / completion-order follow-up として残る。
