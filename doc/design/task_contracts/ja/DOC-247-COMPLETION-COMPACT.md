# Task DOC-247-COMPLETION-COMPACT: payload-family completion compaction

> canonical English:
> [../en/DOC-247-COMPLETION-COMPACT.md](../en/DOC-247-COMPLETION-COMPACT.md)。

本maintenance contractはchecker-only historical completion family 1件をfreeze
する。language behavior、test intent、public API、diagnostic、traceability state、
coverage、descendant-task ownershipを変更できない。

## Identity と status

| Field | Frozen value |
|---|---|
| Task | `DOC-247-COMPLETION-COMPACT` |
| Status | Documentation-prerequisite review/full verification/final quality完了。exact staging/commit待ち。separate prerequisite commit/fresh replay前のmigrationは禁止。 |
| Purpose | durable graph/audit/runner/trace/coverage/sequencing ownerを全て保持し、Task-247 plan/TODO completion section 4件を集約する。 |
| Historical owner | [Task 247](./247.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)と[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Consumers | selected checker source 4 paths、Task Index 4件、future schema-v1 ledger/lint |
| Sequence | `b0930a0c` -> `0154ad74` -> 本prerequisite -> separate migration |
| Readiness | selection HEAD `cbacea8efa0c7ac60f16636c2932c49b877e3eae`、`origin/main...HEAD=0/2`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。repeated selection reviewは**NO FINDINGS**。 |

## Authority と classification

authorityはuser-approved checker-first compaction program、
[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、
accepted graphのrow-specific `doc/spec/en/`/`.miz` reference、current Task-247
trace deferred-reason 5 records、review済みGit historyである。source behaviorは
normativeではない。

| Class | Decision |
|---|---|
| `design_drift` | plan/TODO 4節がhistorical orchestration/completion evidenceを重複し、central Task-247 recordがなかった。 |
| `spec_gap` | compaction固有gapなし。retained MC-G005 external diagnostic-code gateはnonblocking。 |
| `test_gap` / `source_drift` | historical descendant gapはaccepted graphのassignどおりで不変。 |
| `source_undocumented_behavior` | 推測・導入なし。 |
| `test_expectation_drift` | Parser Task 47がomitted-`reconsider` disagreementを保持し、本taskはrepair/rebaselineできない。 |
| `boundary_violation` | exact flat 4 sectionsだけをmigrateできる。graph、semantic/source/bilingual audit、runner state、trace、coverage、Core/root sequencing、module/API、queue section、Task-263+ semanticsの選択は禁止。 |
| `repo_metadata_conflict` | branchはcleanでobserved `origin/main`より2 commits ahead。このreport-only metadata stateはtask-only targetを曖昧にせず、repair禁止。 |

## Frozen preimage と anchor

[`DOC-247-COMPLETION-COMPACT.sources.tsv`](../DOC-247-COMPLETION-COMPACT.sources.tsv)
はbyte-sorted data 4 rows、comments 2行、final LFを持つ。data-row SHA-256は
`85059c4125b162e5ab5dec2cd746fde488185027b288a4c19dbb847c48b78045`、
complete-file SHA-256は
`ad6280a95b24d6d549a0c9a64a0f313b321ccee80f84a0bf78ef0bf21997b2fc`。

selectionはdistinct 4 paths上のsource-locally unique flat H2 4節、116 physical
linesで、checker EN plan/TODO `34/27`、checker JA plan/TODO `32/23`。plan
headingはrepository-wideで各1件、EN/JA共通TODO headingはexactly 2 occurrences
で両方を選択するためinventoryはglobally exhaustive。nested heading/table/
fence/existing redirect/inbound fragment linkはない。

| Source | Preceding owner | Following owner |
|---|---|---|
| checker EN plan | `## Task 268 Completion` | `## Task 248 Frozen Source/Binding-Context Producer Contract` |
| checker JA plan | `## Task 268 completion` | `## Task 248 source/binding-context producer 確定 contract` |
| checker EN TODO | `## Tasks 266-268 Final Checker Handoff Queue` | `## Tasks 248-264 And 269-279 STEP 5 Source-Payload Producer Queue` |
| checker JA TODO | `## Tasks 266-268 Final Checker Handoff Queue` | `## Tasks 248-264/269-279 STEP 5 ソースペイロードproducer queue` |

plan sectionは`0154ad74`でEOFに33/31 linesとして導入された。commit
`0ed76c20`が後続Task-248 sectionをappendした際、current terminating separator
blankだけが加わり、frozen 34/32-line preimageとなった。TODO sectionは
`b0930a0c`のpending stateから`0154ad74`でcurrent 27/23-line completion textへ
変更され、同commit以後byte-stableである。

## Retained owner と exclusion

accepted [graph](../../mizar-checker/ja/payload_family_decomposition.md)の
Existing Boundary And Trace Ownership、Disagreement Classification、Task-247
Exit Criteriaはdurable decomposition ownerのまま。semantic/source audit、mixed
bilingual completion paragraph、runner plan/TODO state、runner traceability、
coverage audit、current trace deferred-reason 5件、root roadmap、Core Task-32
owner、module/API document、後続Tasks 248--264/269--279 queue sectionは不変。

retained ownerは全specification/test reference、family assignment、consumer、
boundary、gate、trace/coverage decision、sequencing factを保持する。existing
Task-247 trace metadataはselection stateのlines 2997/3008/3019/3030/5907に
matching current record exactly 5件を持つ。line numberはinventory evidenceで
ありstable link targetではない。

## Frozen protected baseline

prerequisite/migration expected deltaは全rowでzeroである。

| Surface | Paths | Path SHA-256 | Content SHA-256 |
|---|---:|---|---|
| specification | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

traceは`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`、
coverage auditは
`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`。
822-line ledgerは
`1a3a07297f4f0aee4b13274df44322b52cf92bf71f0ed40824debd7d0aba6c59`。
expected CLI stdout hashはplan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。

## Scope、expected migration、audit impact

documentation prerequisiteは本pair、historical pair、source TSV、plan Task
Index 4件のexact 9 pathsを変更する。各planは`247` row/batch row各1件、total
8 index recordsを追加する。selected preimage、ledger、protected artifact、
current trace/coverage state、public behavior、全retained ownerは不変。

separate prerequisite commit/fresh replay後、migrationはdeclared H2 4節だけを
language-local `247.md#completion-evidence` redirectへ置換できる。変更はsource
4件、本pair、`legacy_compactions.tsv`のexact 7 paths。116 linesは8 redirect-
plus-separator linesとなり108行減、exact source diffは`+4/-112`。

ledger impactは14 lines、`822 -> 836`、batch 1、task 1、distinct 4 paths上の
redirect 4、index record 8。batch rowを除くcanonical task/redirect/index
payload 13 rowsのfrozen SHA-256は
`12da6c943ebc9cdd21c2ab8be9d5c72a1350c2bce74e07a5e81cf272c921385c`。
expected 836-line physical ledger SHA-256は
`33c569ebeac13be3f353177f6c23ddf40c581435950e0e47f57bcdcd7f3528cb`。
source TSV、historical pair、Task Index contentはimmutableとなる。

current owner mapping、deferred reason、status、backlink、count、coverage credit
を変えないため`spec_coverage_audit.md`へのcompaction impactはない。

## Review、verification、exit

prerequisiteはindependent contract/equivalence、schema/test-sufficiency、
boundary、source-document/EN-JA reviewを**NO FINDINGS**まで行う。migrationも
applicableなequivalence/schema/bilingual/final-quality reviewをrepeatする。両
commitは全9 hard gates PASS、score capなし、valid score `>=90/100`が必要。

verificationはsource-preimage/anchor replay、generic schema/link/fragment lint、
checker/runner lint/library、metadata test、format、offline Cargo metadata、
warnings-denied all-target/all-feature Clippy、full workspace test、CLI 5種、
protected count/hash、exact ledger order/hash/cardinality、`git diff --check`、
cached scope/content/whitespace、unstaged/untracked inspectionを含む。push/fetch/
reset/stash mutationは禁止。

prerequisiteはexact nine-path docs scope、selected preimage/protected surface
不変、EN/JA同期、review/verification完了、task-only commit、clean fresh
inventoryでexitする。migrationはexact redirects 4件/seven paths、ledger
replay、separate review/verification、task-only commit、clean fresh inventoryで
exitする。

## Documentation-prerequisite evidence

fresh inventory/independent selection reviewはexact four-section familyを
**NO FINDINGS**とした。reviewはbroader Task-247 audit/runner/trace/coverage/
graph/queue surfaceをowner-local/mixedとしてrejectし、current trace inventoryを
Task-247 deferred-reason exactly 5件へ訂正し、4節がhistorical/flat/source-
locally unique/globally exhaustive/unlinked/bilingually pairedで、上記plan-
separator historyを持つcurrent selection byteとしてfreezeされることを確認した。
contract reviewはinitial Medium history accuracy/Low raw-heading uniqueness
`design_drift`を検出した。EN/JAはexact `0154ad74 -> 0ed76c20` separator history
とfour path-qualified occurrences上のraw headings 3件を記録するよう修正済み。
independent contract/equivalence、schema/test-sufficiency、source-document/
EN-JA/boundary re-reviewはすべて**NO FINDINGS**。long frontend benchmark 3件を
含むfull verificationはPASS。checker/runner lintは各`15/15`、checker/runner
libraryは`530/530`/`600/600`、runner metadataは`137/137`。format、offline Cargo
metadata、warnings-denied all-target/all-feature Clippy、full all-target/all-
feature workspace suite、`git diff --check`もPASS。CLI 5件は各23 warnings/zero
errorsでexit zeroし、frozen stdout hash 5件を再現する。

selected section 4件は`34/27/32/23` line count/frozen hash、source TSVは両
hash、Task Indexはexact 8 recordsを再現する。protected count/path hashは
specification 64、`.miz` 343、expectation 435、checker production 30、runner
production 90、Cargo 21を再現し、zero protected diffが全frozen content hashを
保持する。trace、coverage audit、822-line ledgerと各hashもexactに再現する。
final independent read-only quality reviewは**NO FINDINGS**、hard gate 9件PASS、
score capなしで**100/100**。exact staging、commit、post-commit replayは未完了。

## Handoff

本documentation prerequisiteだけをcomplete/commitする。source section/anchor
4件をfresh-replay後、separately frozen seven-path migrationを行う。parentは
`xhigh`、bounded independent reviewは`high`を用いる。
