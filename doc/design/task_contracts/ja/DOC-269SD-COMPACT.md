# Task DOC-269SD-COMPACT: legacy completion evidence 一括集約

> canonical English:
> [../en/DOC-269SD-COMPACT.md](../en/DOC-269SD-COMPACT.md)。
> 本文書は同一logical taskの日本語companionである。

これは派生documentation-maintenance contractであり、language behavior、test
intent、diagnostic、public API、coverage creditを追加・上書きしない。Task
269SDT完了後に、userが一つのcoherent cleanupとして明示承認したため、完了済み
2 taskだけをbatch対象とする。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269SD-COMPACT` |
| Status | implementation/verification完了、task-only commit準備済み。immutable commit identityはpost-commit evidenceとする。 |
| Purpose | 重複したTask-269SDP/269SDC completion evidenceをtaskごとの単一EN/JA historical contractへ移し、owner-local durable factをすべて保持し、同じcontract/link driftを今後防ぐ。 |
| Primary owners | repository documentation policyと`mizar-test` lint policy |
| Consumers | checker/test crate plan、checker/test owner-local design docs、audit、TODO、future autonomous-task agent |
| Dependencies | Task-269SDP implementation `2ba1ee910aea4939abc26b64a96a113e80c01306`、Task-269SDC implementation `b1c8c814655d58fff5e5445dd94132bab37965c7`、central-contract policy `f322a710`、Task-269SDT migration `ee91030f`、Task-269SDT implementation `c5389023eddf84600c5f7972b240712673e76d95` |
| Readiness | clean `c5389023`のfresh inventoryでbounded exact-duplication familyを確認し、repository-metadata conflictはない。 |

## Authority And Classification

本maintenance taskのauthorityは [`AGENTS.md`](../../../../AGENTS.md)、
[autonomous crate protocol](../../autonomous_crate_development.md#canonical-task-contracts)、
および残るduplicated documentationを一taskで整理するuserの明示承認である。
historical 2 taskのlanguage authorityは新contractからindexするだけで、ここで再解釈しない。

| Class | Decision |
|---|---|
| `design_drift` | Task-269SDP completion evidenceは40 Markdown files、Task-269SDCは42 filesに反復される。task-contract policyはあるがrecursive pair/link enforcementとexplicit batch-migration safety ruleがない。 |
| `test_gap` | paired task-contract pathとlocal Markdown target/fragmentをrecursiveに検査するrepository testがない。focused lint-policy test 1件でcloseする。 |
| `spec_gap` | documentation structureについてなし。historical Chapter-4/15 `set` disagreementは不変で、`z`/`q` semanticsを引き続きblockする。 |
| `source_drift` | なし。production Rustはscope外。 |
| `source_undocumented_behavior` | 導入・推測しない。 |
| `test_expectation_drift` | なし。expectationを保護する。 |
| `boundary_violation` | adjacent owner-local API/invariant/runner/audit/traceability/sequencing sectionをすべて保持することで回避する。 |
| `repo_metadata_conflict` | 選択時なし。`origin/main...HEAD`は`0/1`、worktree clean、protected stash不変。 |

## Frozen Migration Surface

source familyはexact 82 H2 completion-status sectionsである。

- Task-269SDP 40 sections：English `Implementation Status` 20件、Japanese
  lowercase `implementation status` 20件
- Task-269SDC 42 sections：English tree 20件、Japanese tree 20件、English
  root audit/roadmap 2件の `Implementation Status`

baseline bodyは3,027 lines。各source sectionを同位置でcompactなlanguage-local
linkへ置換し、対応historical contractの`Completion Evidence`へ向ける。対象sectionの
前後H2と、下記exact compact Task Index rows以外の全affected fileのnon-status byteを
保護する。English fileはEnglish
contract、Japanese fileはJapanese companion、English root 2 docsはEnglishへlinkする。

paired historical contracts [`269SDP`](./269SDP.md) と
[`269SDC`](./269SDC.md)、および本migration EN/JA pairを追加する。implementationで
checker/testのEN/JA Task Index 4 tablesへ3 contractsを1 rowずつ追加し、exact 12
planned new rowsとする。
本taskではexact status sections外の
historical task plan、API、test-design、runner、traceability、coverage mapping、
boundary、bilingual、TODO owner sectionを削除しない。

policy deltaは`AGENTS.md`、`doc/design/README.md`、
`doc/design/autonomous_crate_development.md`だけ。user-authorizedかつseparately
reviewedなbatch legacy-evidence compactionを、coherent duplication family、exact
redirect map、paired EN/JA owner、owner-local fact保存、link validation、behavior/
coverage不変を満たす場合だけ許可する。通常semantic task中のwholesale historical
rewriteは許可しない。

test deltaは`crates/mizar-test/tests/lint_policy.rs`だけ。新integration test 1件が
`doc/design/task_contracts/en`/`ja`をrecursive inventoryし、同一relative Markdown
paths、canonical/companion marker、owning crate plan Task Indexとのreciprocal
link、exact 82 legacy redirectsを検査し、task contractから到達するrepository-
supported inline relative Markdown file targetとATX-heading fragmentを検証する。
HTTP(S)、mail、bare non-file reference、reference-style link、escaped/nested-
parenthesis destination、code内Markdownはlint grammar外とする。fragmentは
duplicate-heading suffixを含むdeterministic repository ATX-heading slug functionで
検証し、complete GitHub Markdown parsingをclaimしない。既存269SDT pairと本taskで
追加する全pairにpassしなければならない。

task orderは[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index)がindexする。

## Protected And Forbidden Changes

`doc/spec/**`、`.miz`、fixture、sidecar、expectation、
`tests/coverage/spec_trace.toml`、trace row/status/backlink、Cargo、production Rust、
public API、diagnostic、parser/resolver output、active route、CLI result、executable
coverage creditを変更しない。owner-local durable sectionを書き換えず、current
sourceからlanguage meaningを推測しない。Chapter-4/15 `set` disagreementを解決せず、
goal/guard/fact/proof/discharge/acceptance/obligation/closure/capture semanticsをclaimしない。

## Baseline And Expected Impact

選択時`doc/design`は632 Markdown files、約383,168 lines。exact repeated status
familyは82 sections/3,027 lines。3 EN/JA pairsを構成する6 contract files、small
policy delta 3件、4 plan filesのexact 12 planned compact Task Index rows、lint-policy
test 1件を追加しつつ、
大幅なnet deletionを見込む。final countはrequired byte totalを投影せず実測する。

checker/runner library count、production file inventory/hash、corpus/requirement、
pass/fail、stage、type coverage、trace hash、全5 CLI hash、fixture/expectation hash、
active resultはTask-269SDT post-commit baseline不変。`lint_policy` integration targetの
test count/list hashだけexact 1 test分の変更を許可する。

## Reviews And Verification

compaction前にindependent specification/documentation reviewerがexact behavior-neutral
redirectをfreezeしていることを確認し、**NO FINDINGS**にする。edit後はindependent
test-sufficiency、implementation/equivalence、source/document/EN-JA/link-owner reviewを
すべて **NO FINDINGS** にする。parent final reviewは全9 hard gates、score capなし、
`90/100`以上を必要とする。

verificationはfocused `lint_policy` target、checker/runner libraries、metadata tests、
`cargo fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`、
`cargo test`、全5 CLI、protected artifact/count/hash replay、exact 82 section消滅と
redirect存在、recursive link/fragment replay、`git diff --check`、cached name/stat/
content/whitespaceとunstaged diffの明示監査を含む。

## Completion Evidence

- frozen 82 status sectionsは消滅し、fileと`#completion-evidence` fragmentが解決する
  exact 82 language-local redirectsへ置換された。owning Task Index 4 tablesには
  exact 12 new rowsがある。
- 本taskは52 pathsを変更する。legacy owners 42、paired contract files 6、policy
  files 3、`mizar-test` lint-policy test file 1である。protected specification、
  fixture、sidecar、expectation、trace、manifest、production source、public API、
  diagnostic、executable coverage artifactは変更していない。
- `doc/design`は638 Markdown files、381,026 linesであり、選択時の632
  files/約383,168 linesに対して、6-file増加はexact 3 EN/JA contract pairsだけである。
- `mizar-test` lint-policy targetは15 tests、raw test-list hashは
  `b044e771a655e72131d0371636bbac5684ef93a3ea503984537a4bb9dd13a7cf`である。
  sole added testがpairing、exact redirect/index、reciprocal owner、supported
  target/fragment resolutionをrecursiveに強制する。
- independent specification/documentation、test-sufficiency、implementation/
  equivalence、source/document/EN-JA/link-owner reviewはfinding修正後すべて
  **NO FINDINGS**で終了した。
- focused/full lint-policy tests、checker/runner library/metadata tests、
  `cargo fmt --all --check`、warnings denied workspace Clippy、full `cargo test`、
  全5 CLI、protected count/hash replay、`git diff --check`はpassする。frozen
  production/corpus/trace/CLI hash/countは不変で、trace hashは
  `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`のままである。

## Exit And Handoff

task-only 1 commit、全review/hard gates pass、shared completion factを保持するpaired
historical records、intentional compact index/policy外の全owner-local section保存、
clean post-commit inventory、protected stash不変、no pushをexitとする。commitは自己
hashを含められないためmigration commit hashはpost-commit reportへ記録する。

commit後はcanonical authority/public APIをfresh inventoryし、IDを先取りせず次の
dependency-ready semantic taskを選ぶ。parent reasoningは`xhigh`、bounded review agentは
`high`、lower settingはdeterministic inventoryだけに用いる。
