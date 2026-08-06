# Task DOC-258B4A-COMPACT: B4A implementation-evidence compaction

> canonical English: [../en/DOC-258B4A-COMPACT.md](../en/DOC-258B4A-COMPACT.md)。

## Identity と status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B4A-COMPACT` |
| Status | documentation prerequisite/lower lint contract実装済み。全review/verification完了。exact staging/commit待ち。migrationは[DOC-COMPACT-PATH-SCOPE](./DOC-COMPACT-PATH-SCOPE.md)待ち。 |
| Purpose | frozen contract/durable ownerを保持し、Task-258B4A implementation completion 4節を集約する。 |
| Historical owner | [Task 258B4A](./258B4A.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index) / [runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Selection HEAD | `75d8af2d5e071f415d1cada9e1a8981aaef2d3b2` |
| Repository state | clean `main`、`origin/main...HEAD=0/4`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |

## Authority と classification

authorityはuser-approved checker-first compaction、`AGENTS.md`、autonomous
migration policy、review済みGit history、selected completed section、surviving
durable owner。source behaviorはnormativeではない。

| Class | Decision |
|---|---|
| `design_drift` | plan/TODOがB4A historical implementation/review/count/hash/commit evidenceを重複する。 |
| `test_gap` | generic ledger lintはdeclared legacy headingをrepository-globalに扱う。selected TODO headingはunselected mizar-test ownerにもあるためpath-scoped regressionが必要。 |
| `boundary_violation` | Tasks 266--268はregistered Task-247 redirectを内包/anchorするためreject。current global heading lintもB4A source boundaryをcrossする。flat B4A completion 4節だけをmigrateする。 |
| `spec_gap` / `source_drift` | 導入・修復なし。historical stateはdurable ownerに残る。 |
| `source_undocumented_behavior` / `test_expectation_drift` | 推測・変更なし。 |
| `repo_metadata_conflict` | observed `origin/main`より4 commits ahead。report only、repair禁止。 |

## Frozen source と anchor

[`DOC-258B4A-COMPACT.sources.tsv`](../DOC-258B4A-COMPACT.sources.tsv)は
byte-sorted data 4 rows/comments 2行/final LF。data-row SHAは
`7892258a006395a7372b4a30195cfe53043782569039bcbb716e1a6660fb1062`、
complete SHAは`73b007dda0100274c678c8a751dbb136604cc7284ffd3046f55a977b433488a4`。
flat/source-locally unique/unlinkedでtotal 154 lines、EN/JA plan `52/52`、
EN/JA TODO `25/25`。

| Source | Previous H2 | Next H2 |
|---|---|---|
| EN plan | `## Task 258B4A Frozen Explicit-Universal Composite Theorem Root` | `## Task 258B4B Frozen Connective/Grouping Composite Theorem Root` |
| JA plan | `## Checker Task 258B4A composite-theorem-root frozen contract` | `## Task 258B4B frozen connective/grouping composite theorem root` |
| EN TODO | `## Checker Task 258B4A Documentation Prerequisite` | `## Checker Task 258B4B Documentation Prerequisite` |
| JA TODO | `## Checker Task 258B4A documentation prerequisite` | `## Checker Task 258B4B documentation prerequisite` |

implementation commit `662adbde`がcompletion sectionを導入し、successor
prerequisite `b8a7b8257`はimmutable commit/post-inventory tailとfollowing B4B
headingだけを追加した。current blameはselected全行をこの2 commitsへassignする。

## Retained owner と exclusion

B4A frozen plan、source-statement/formula-composition/Typed/Resolved AST、payload-
family、source/spec、boundary、bilingual、runner、traceability、coverage ownerは
不変。same-name mizar-test TODOはdistinct unselected owner。B4B以降、Tasks
265--268、spec、`.miz`、expectation、sidecar、trace、coverage audit、production、
Cargo、API、active behaviorは禁止。

## Protected baseline

prerequisite/migration deltaは全てzero。count/path/content SHAはcanonical EN表
[Protected Baseline](../en/DOC-258B4A-COMPACT.md#protected-baseline)と同一。
spec 64、`.miz` 343、expectation 435、checker production 30、runner production
90、Cargo 21。traceは`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`、
coverage auditは`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`。
ledger baselineは836 lines/
`33c569ebeac13be3f353177f6c23ddf40c581435950e0e47f57bcdcd7f3528cb`。

## Prerequisite と expected migration

prerequisiteは本pair、historical Task-258B4A pair、lower-lint contract pair、
source TSV、plan Task Index 4件のexact 11 paths。各planにTask/batch/lower-lint
row、total index 12 recordsを追加する。selected source/ledgerは不変。

migration前にseparate
[Task DOC-COMPACT-PATH-SCOPE](./DOC-COMPACT-PATH-SCOPE.md)がpath-scoped generic-
lint correction/regressionを別commitで実装する。

両prerequisite commit/fresh replay後のmigrationはsource 4件、本pair、ledgerのexact
7 paths。4節をlanguage-local `258B4A.md#completion-evidence` redirectへ置換し、
exact source diff `+4/-150`、154 linesをredirect-plus-separator 8 linesへ縮小する。

ledger impactは14 lines、`836 -> 850`、batch 1/task 1/redirect 4/source 4/index
8。canonical 13-row SHAは
`6e082203fc14fa303969e13d1deebd3b630adbb3052b67019a874b3ed2643f2d`、
expected physical SHAは
`7bd738ad591a40667cb95421dd68d386213c25c51274cbf5c79d8f24b0b1688a`。
mapping/ownership/status/deferred/credit不変のためcoverage audit impactなし。

## Review、verification、exit

prerequisite/migrationはapplicableなequivalence、schema/test-sufficiency、
bilingual/boundary、final-quality reviewを別々に**NO FINDINGS**まで行う。hard
gate 9件PASS、capなし、valid score `>=90/100`。

preimage/history/anchor、generic recursive contract/link/fragment/ledger lint、
checker/runner lint/library、runner metadata、format、offline metadata、warnings-
denied all-target/all-feature Clippy、full workspace、CLI 5件、protected count/hash、
ledger order/hash/cardinality、diff check、cached/unstaged/untrackedをverifyする。
push/fetch/reset/stash mutationは禁止。

prerequisite exitはexact 11 paths、source/ledger/protected owner不変、EN/JA同期、
review/verification、1 commit、clean replay。migrationは別にexact redirects 4/
seven paths、ledger replay、全gate、1 commit、clean replayを要求する。

## Documentation-prerequisite evidence

independent contract/equivalence、schema/test-sufficiency、bilingual/boundary
reviewは全て**NO FINDINGS**。first passはmissing runner-owner linkとgeneric global-
heading lint defectを検出した。synchronized historical contractへretained runner
owner linkを追加し、後者の`test_gap`/`boundary_violation`を上記separate path-
scoped lint taskとしてfreezeした。re-reviewでは同taskのfuture staging boundaryも
lint test file 1件へ修正した。selected source、ledger row、protected owner、
coverage claimは変更していない。

checker/runner lintは各`15/15`、checker/runner libraryは`530/530`、`600/600`、
runner metadataは`137/137` PASS。format、offline Cargo metadata、warnings-denied
all-target/all-feature Clippy、long frontend benchmark 3件を含むfull all-target/all-
feature workspace、generic recursive task-contract/link/fragment/ledger lint、
`git diff --check`がPASS。CLI 5件は各exit zero、known warning 23、error zero。
stdout hashはplan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、parse
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`
を再現した。

selected 4 sectionsはfrozen `52/52/25/25` line count/hashを再現し、source TSVも
両frozen hashを再現。Task Index 12 recordsは全て存在する。protected count/path/
content hashはbaselineと完全一致しprotected diff zero。trace、coverage audit、
unchanged 836-line ledgerもfrozen hashを再現した。final independent read-only
quality reviewは**NO FINDINGS**、hard gate 9件PASS、capなし、**100/100**
（`20/20/15/15/10/10/5/5`）。independent replayもprospective 13-row canonical
hash/850-line physical ledger hashを再現した。classified residual riskはcurrent
global-heading lint defectであり、separate path-scoped correctionのPASS/commitまで
migrationはblocked。exact staging、commit、clean post-commit replayだけが残る。

## Handoff

本documentation prerequisiteだけをcomplete/commitする。その後
`DOC-COMPACT-PATH-SCOPE`をseparate implement/commitし、fresh replay後にfrozen
migrationを行う。parentは`xhigh`、bounded reviewは`high`。
