# Crate exit report: mizar-driver

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

状態: task D-022 で完了。

## Result

Status: complete。
Quality score: 94/100。
Score caps applied: none。hard gate failure、未解決 blocking/high finding、
`source_undocumented_behavior`、`test_expectation_drift`、required verification
failure、unapproved semantic behavior change は存在しない。

## 範囲

この report は `mizar-driver` autonomous task stream を close する。完了済み task
commit、hard-gate evidence、quality score、残る分類済み gap、最終 ownership shape、
verification、next-phase handoff を記録する。

Milestone scope: `mizar-driver` を build request、build session、phase service
registration、driver-owned query boundary、scheduler submission、
protocol-agnostic build event、CLI batch entry、watch-mode orchestration の
orchestration owner にする。

Included:

- paired EN/JA crate plan、task TODO、owner-boundary spec、audit、この exit report;
- request/session、registry、driver core、event、CLI、watch orchestration、
  determinism、module-boundary guard の Rust source/test;
- 既存の `mizar-build`、`mizar-ir`、`mizar-diagnostics`、隣接 owner API を消費し、
  それらの authority を奪わない integration seam。

Excluded:

- phase semantics、type checking、name resolution、proof acceptance、trusted
  status、kernel acceptance、cache compatibility decision、artifact serialization、
  artifact publication token、real producer payload ownership、LSP protocol
  conversion、まだ unavailable な owner seam;
- `mizar-driver` 外の report-only repository metadata conflict の修復。
  `mizar-artifact` closeout report の欠落を含む;
- `.miz` language test、language specification file、coverage traceability metadata、
  expectation sidecar。この milestone は language behavior を変更しない。

## Task commit

| Task | Commit | Summary |
|---|---|---|
| D-000 | `b359f54` | autonomous crate plan を追加 |
| D-001 | `c44fc47` | `mizar-driver` crate scaffold |
| D-002 | `0e6cead` | request session を仕様化 |
| D-003 | `34ba74a` | request session を実装 |
| D-004 | `02715b8` | phase registry を仕様化 |
| D-005 | `1083b02` | phase registry を実装 |
| D-006 | `d043db6` | frontend adapter gap を分類 |
| D-007 | `0ce3356` | driver core を仕様化 |
| D-008 | `9dce878` | driver core を実装 |
| D-009 | `b532e82` | build event を仕様化 |
| D-010 | `5702ab5` | build event を実装 |
| D-011 | `9ebe2b4` | cancellation flow を完了 |
| D-012 | `54fac26` | CLI surface を仕様化 |
| D-013 | `e817f5e` | CLI batch entry point を追加 |
| D-014 | `cc5560d` | watch orchestration を追加 |
| D-015 | `f4914a2` | phase adapter readiness を分類 |
| D-016 | `b3fafeb` | determinism suite を追加 |
| D-017 | `cdbc4a1` | enum compatibility policy を文書化 |
| D-018 | `8b7b59e` | source/spec audit を追加 |
| D-019 | `b70fab1` | bilingual driver docs を監査 |
| D-020 | `e1d71e9` | architecture-22 follow-up を監査 |
| D-021 | `d0045c5` | driver helper module を分割 |

D-022 closeout commit は、この file を commit した後の final handoff で報告する。

## 最終形

`mizar-driver` が所有するもの:

- `BuildRequest` / `BuildSession` の snapshot-boundary envelope、lane、
  generation、lifecycle transition、obsolete-publication decision;
- deterministic な `PhaseService` registration、duplicate rejection、requirement
  classification、driver-owned salsa/query-compatible boundary;
- `CompilerDriver` submission、cancellation、scheduler authority consumption、
  missing service classification、dispatch-gap blocking、stored session event replay;
- protocol-agnostic な `BuildEventStream` record、deterministic ordering、
  currentness check、stale-output suppression、diagnostics owner record、
  publication authority を持たない artifact-boundary readiness event;
- library-level `mizar build` batch entry point、stable human/JSON rendering、
  exit-code mapping、明示的な owner-gap output;
- owner-provided changed path / snapshot input 上の watch-mode orchestration、
  superseded replay suppression、任意の real `mizar-ir` `PhaseOutputPublisher`
  snapshot replacement;
- CLI output rendering、driver event/scheduler/watch helper、registry phase catalog /
  fingerprinting 用の private helper module。

`mizar-driver` が所有しないもの:

- phase semantics、type checking、name resolution、overload resolution、VC
  generation、ATP solving、kernel acceptance、proof acceptance、trusted status、
  proof reuse、cache compatibility decision、artifact serialization、artifact
  publication token、real producer output payload、LSP protocol conversion。

## Hard gate

| Gate | Status | Evidence |
|---|---|---|
| blocking/high specification inconsistency が残らない | Pass | D-018 source/spec audit、D-019 bilingual audit、D-020 architecture-22 audit、D-021 module-boundary gate、final review は blocking/high finding なし。 |
| public driver behavior が文書化され trace されている | Pass | `source_spec_correspondence.md` が public API と promised behavior を source/test または classified gap へ trace する。 |
| source behavior が design/spec/test intent と一致 | Pass | `cargo test -p mizar-driver`、lint-policy guard、source/spec audit、full implementation review が pass。 |
| implementation に合わせるだけの `.miz` / spec metadata 変更なし | Pass | この stream は Rust driver code/test と design docs だけを変更した。`.miz`、expectation、traceability、language spec は編集していない。 |
| driver ownership boundary を維持 | Pass | source guard と module spec は phase semantics、proof/cache/artifact authority、diagnostics identity、LSP conversion を driver の外に保つ。 |
| required verification が pass | Pass | 下の verification table を参照。 |
| 残る risk は分類済み | Pass | 残る分類済み gap を参照。 |

## Verification

Final D-021/D-022 closeout verification:

| Command | Status |
|---|---|
| `cargo fmt --check` | Passed |
| `cargo test -p mizar-driver` | Passed |
| `cargo clippy -p mizar-driver --all-targets -- -D warnings` | Passed |
| `cargo test -p mizar-build` | Passed |
| `cargo test -p mizar-ir` | Passed |
| `cargo test -p mizar-diagnostics` | Passed |
| `cargo test -p mizar-frontend` | Passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed |
| `cargo test` | Passed |
| `git diff --check` | Passed |
| `git diff --cached --check` | D-022 task path を stage した後に pass |

上記の package-specific adjacent-crate test は final workspace `cargo test` にも含まれるが、
crate plan が orchestration / integration seam confidence の closeout command として列挙する
ため、個別にも記録する。

## Quality score

Quality score: **94/100**。

根拠:

- hard gate が pass;
- public driver API と promised behavior は paired EN/JA docs に文書化済み;
- implemented seam は targeted Rust test と determinism coverage を持つ;
- source guard は public enum compatibility、dependency boundary、source surface、
  private helper visibility、non-owner authority term を cover する;
- full workspace clippy と test が pass;
- 残る risk は fake implementation ではなく owner-seam gap として分類済み。

減点理由:

- full real clean / incremental / parallel equivalence は real producer/cache/artifact/proof
  seam が存在するまで deferred;
- semantic/proof/artifact phase adapter は unavailable owner seam のまま;
- `mizar-artifact` closeout metadata は report-only repository metadata conflict のまま;
- real LSP bridge はまだ wired されていない。

上の hard gate が後で失敗した場合、この score は無効である。

## Score breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

## 残る分類済み gap

| Gap | Classification | Closeout status |
|---|---|---|
| `DRIVER-G-001` | `repo_metadata_conflict` | `mizar-artifact` closeout report が欠落。報告のみ。この stream では修復しない。 |
| `DRIVER-G-005` | `external_dependency_gap` | artifact publication token と full phase-15 producer emission は unavailable。 |
| `DRIVER-G-007` | `deferred` | full real clean / incremental / parallel equivalence は real cache、producer、artifact、proof、worker-race、multi-task dispatch seam を要求する。 |
| `DRIVER-G-009` | `repo_metadata_conflict` | `mizar-proof` に関する既存 artifact metadata drift は report-only。 |
| `DRIVER-G-010` | `external_dependency_gap` | frontend canonical producer payload と diagnostics bridge readiness は unavailable。 |
| `DRIVER-G-011` | `external_dependency_gap` | scheduler-selected real phase dispatch callback は `mizar-build` から公開されていない。 |
| `DRIVER-G-012` | `external_dependency_gap` / `deferred` | real file watcher / coalescing owner と LSP build bridge は driver の外に残る。 |
| `DRIVER-G-013` | `external_dependency_gap` | semantic/proof/artifact phase adapter は complete owner-provided driver-callable seam を欠く。 |
| `DRIVER-G-014` | `deferred` | documentation extraction は `mizar-doc` owner crate / surface を待つ。 |

## Human review surface

human reviewer が主に確認するべき範囲:

- `doc/design/mizar-driver/en/` と `doc/design/mizar-driver/ja/`、特に
  `00.crate_plan.md`、`todo.md`、`request.md`、`registry.md`、`driver.md`、
  `events.md`、`cli.md`、audit report、この exit report;
- `crates/mizar-driver/src/` の public module と private helper module;
- `crates/mizar-driver/tests/`、特に lint/source-boundary guard と determinism coverage;
- `mizar-build`、`mizar-ir`、`mizar-cache`、`mizar-diagnostics` の closeout report と、
  report-only として扱う `mizar-artifact` closeout report 欠落。

この milestone の human review surface には `doc/spec`、`.miz` test、test expectation、
spec-trace metadata の変更は含まれない。

## Test expectation summary

| Test surface | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| `crates/mizar-driver` unit tests | request/session、registry、driver、event、CLI、watch behavior が文書化された orchestration contract と一致すること。 | Pass。 | Driver orchestration only。 | driver event と owner readiness record を使う。diagnostic identity は作らない。 | Spec 23 と `doc/design/mizar-driver/en/*.md`。 |
| `crates/mizar-driver/tests/lint_policy.rs` | dependency、public enum、source-surface、private-helper、non-owner boundary rule を保つ。 | Pass。 | Source-boundary guard。 | なし。structural source check のみ。 | Crate plan、module layout、driver design docs。 |
| Determinism tests | implemented seam 上の clean/incremental/parallel projection の stable replay/rendering。 | crate-local implemented seam では pass。real full equivalence は deferred。 | Driver event / CLI projection。 | owner record を維持し、message text は presentation のみ。 | Architecture 20/22 と driver event docs。 |
| Workspace verification | 隣接 owner crate が driver integration surface とともに compile/test できること。 | Pass。 | Cross-crate integration confidence。 | diagnostics authority は `mizar-diagnostics` に残る。 | Internal 01、architecture 22、隣接 crate closeout report。 |
| `.miz` / expectation corpus | この stream は language behavior を変更していない。 | Not changed。 | Not applicable。 | Not changed。 | 既存の `doc/spec/en/` と test corpus が authoritative。 |

## Review

autonomous workflow では task stream 全体を通じて specification/docs、test
sufficiency、full implementation、source-doc consistency、source-boundary/quality の
review-only agent を使用した。blocking/high/medium finding は各 task commit 前に修正した。
final D-022 closeout review は protocol-template field の欠落を見つけた。この report は
それを修正する。繰り返し D-022 review が blocking/high finding なしを報告する場合に限り、
この closeout score は有効である。

## Next-phase handoff

Recommended reasoning: **high**。

理由: 次 phase は owner-seam integration task になる可能性が高い。慎重な boundary reasoning
と source/test review が必要だが、crate-wide autonomous buildout ほど広くはない。

次 task が real semantic/proof/artifact phase adapter integration、cache/proof reuse
validation、artifact publication、LSP bridge orchestration を試みる場合は **xhigh** に上げる。
docs-only follow-up や狭い test-only source guard なら **medium** に下げてもよい。

Suggested prompt:

```text
Continue from the completed mizar-driver closeout. Pick one remaining
classified owner seam, preferably DRIVER-G-011 scheduler-selected real phase
dispatch or DRIVER-G-013 one real phase adapter whose owner crate now exposes a
complete driver-callable seam. Follow AGENTS.md. Do not create fake adapters,
provisional publication tokens, stub producer outputs, cache/proof authority,
artifact serialization, or LSP protocol conversion in mizar-driver. Update
paired EN/JA docs, add focused tests, run required verification, and commit one
task-sized change.
```
