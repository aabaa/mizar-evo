# Task DOC-249S-ACTIVE-EVIDENCE-COMPACT: structure-member active evidence compaction

> canonical English:
> [../en/DOC-249S-ACTIVE-EVIDENCE-COMPACT.md](../en/DOC-249S-ACTIVE-EVIDENCE-COMPACT.md)。

本maintenance contractはchecker/runner historical completion-evidence family
1件をfreezeする。language behavior、test intent、public API、diagnostic、
traceability、coverageを変更できない。

## Identity と status

| Field | Frozen value |
|---|---|
| Task | `DOC-249S-ACTIVE-EVIDENCE-COMPACT` |
| Status | Documentation prerequisite commit済み。frozen migration、全review、full verification、final quality完了。exact staging、commit待ち。 |
| Purpose | durable/frozen ownerと全mixed sectionを保持し、Task-249S active implementation/no-runner evidenceを集約する。 |
| Owners | migration policy、historical [249S](./249S.md#completion-evidence)、[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Consumers | checker/runner source 24 paths、Task Index 4件、future schema-v1 ledger/lint |
| Sequence | `274917ab` -> `93d64c33` -> `1fe0b156` -> `f11a517e` |
| Readiness | selection HEAD `331fdc055d9416225ccc6e2acb22d199c17cb8ee`、`origin/main...HEAD=0/1`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。revised selection reviewは**NO FINDINGS**。 |

## Authority と classification

authorityはuser-approved checker-first compaction program、
[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、
historical contractのretained canonical/test owner、review済みGit historyである。
source behaviorはnormativeではない。

| Class | Decision |
|---|---|
| `design_drift` | paired component 12 paths上のactive 24節がTask-249S completion/no-runner evidenceを反復する。historical contractをshared completion-evidence ownerとする。 |
| `spec_gap` / `test_gap` | 本structural taskにはない。authority、test intent、prior closure、deferralは不変。 |
| `source_drift` / `source_undocumented_behavior` | 導入しない。production sourceはprotected。 |
| `test_expectation_drift` | なし。specification、`.miz`、fixture、sidecar、expectation、traceはprotected。 |
| `boundary_violation` | 初回26-section案は`resolved_typed_ast.md` active H2両方を含んだ。JA H2にはTask-269B completion redirectが既にあり、schema v1でwhole sectionをTask 249Sへ移せないため、bilingual symmetryとしてEN/JA両方を除外する。live coverage owner、全frozen/addendum/TODO/Task-263以降、全unlisted artifactも除外する。 |
| `repo_metadata_conflict` | selectionでは`0/1`を測定した。final review中に`origin/main`が外部から同じ`331fdc05`へ進み、現在`0/0`。reflogは2026-08-06 09:33:55 +0900の`update by push`を記録する。parentはpushしておらずtask-only targetは識別可能で、fetch/reset/push/stash mutationは禁止のまま。 |

## Frozen preimage と anchor

[`DOC-249S-ACTIVE-EVIDENCE-COMPACT.sources.tsv`](../DOC-249S-ACTIVE-EVIDENCE-COMPACT.sources.tsv)
はbyte-sorted data 24 rows、comments 2行、final LFを持つ。data-row SHA-256は
`981aca5b86370ef4513070334b8c7fc5710fb6e337fbbc56b2f1bee0bdef40d9`、
complete-file SHA-256は
`53e8b44ee40078f613f633a355c25691460adb28e4919e6f9d2c9d32a7bdf434`。

selectionはdistinct 24 paths上のglobally exhaustive H2 24節、256 physical
linesで、checker EN `7/87`、checker JA `7/82`、runner EN `5/44`、runner JA
`5/43`。全sectionはflatでnested heading/table/fence/existing redirect/removable
fragmentへのinbound linkを持たず、各raw headingはrepository-wideで一意。
retained anchorは次のとおり。

| Source | EN preceding / following owner | JA preceding / following owner |
|---|---|---|
| checker `00.crate_plan.md` | `## Task 263 Fresh Preflight: Mandatory Checker Task 249S` / `## Task 263 Frozen Structure-Definition Contract` | `## Task 263 fresh preflight: mandatory checker Task 249S` / `## Task 263 frozen structure-definition contract` |
| checker `bilingual_sync_audit.md` | `## Task 249S Frozen-Contract Synchronization` / `## Task 263 Frozen-Contract Synchronization` | `## Task 249S frozen-contract synchronization` / `## Task 263 frozen-contract synchronization` |
| checker `module_boundary_audit.md` | `## Task 249S Frozen Module Boundary` / `## Task 263 Frozen Module Boundary` | `## Task 249S frozen module boundary` / `## Task 263 frozen module boundary` |
| checker `payload_family_decomposition.md` | `## Task 249S Structure-Member Type Lower Family` / `## Task 264 Property-Implementation Family` | `## Task 249S structure-member type lower family` / `## Task 264 property-implementation family` |
| checker `source_spec_audit.md` | `## Task 249S Frozen Future Public-Surface Audit` / `## Task 263 Frozen Source/API Audit` | `## Task 249S frozen future public-surface audit` / `## Task 263 frozen source/API audit` |
| checker `source_type.md` | `## Task 249S Frozen Standalone Structure-Member Type Intake` / `## Task 263 Test-Only Lower Replay Seam` | `## Task 249S standalone structure-member type intake frozen contract` / `## Task 263 test-only lower replay seam` |
| checker `typed_ast.md` | `## Task 249S Standalone Member-Type Ownership Addendum` / `## Task 263 Frozen Typed Ownership` | `## Task 249S standalone member-type ownership addendum` / `## Task 263 frozen Typed ownership` |
| runner `00.crate_plan.md` | `## Checker Task 249S Frozen No-Runner Prerequisite` / `## Checker Task 263 Frozen Consumer Plan` | `## Checker Task 249S frozen no-runner prerequisite` / `## Checker Task 263 frozen consumer plan` |
| runner `bilingual_sync_audit.md` | `## Checker Task 249S Synchronization Addendum` / `## Checker Task 263 Frozen Consumer Synchronization` | `## Checker Task 249S synchronization addendum` / `## Checker Task 263 frozen consumer synchronization` |
| runner `harness.md` | `## Checker Task 249S No-Consumer Harness Boundary` / `## Checker Task 263 Frozen Harness Route` | `## Checker Task 249S no-consumer harness boundary` / `## Checker Task 263 frozen harness route` |
| runner `module_boundary_audit.md` | `## Checker Task 249S No-Runner Boundary` / `## Checker Task 263 Frozen Runner Boundary` | `## Checker Task 249S no-runner boundary` / `## Checker Task 263 frozen runner boundary` |
| runner `traceability.md` | `## Checker Task 249S Frozen Traceability No-Op` / `## Checker Task 263 Frozen Trace Intent` | `## Checker Task 249S frozen traceability no-op` / `## Checker Task 263 frozen trace intent` |

excluded final pairとlive coverage ownerはindependent evidence ownerのまま。
selected preimageにはhistorical `249S` contract、batch/source-inventory identity、
Task Index row、ledger task/redirectが存在しない。

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
788-line ledgerは
`1702d79a198685ce8603f65dbdd2947f7d2c78e7b9ea3e76a150caac29a48da7`、
expanded inventoryは
`bb38229607a2a3eaa81e7b8d4ab8218c8ce42f0f86de91dd7471b3f205ed0b66`。
expected CLI stdout hashはplan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。

## Scope、verification、exit

documentation prerequisiteは本pair、historical pair、source TSV、plans 4件の
exact 9 pathsを変更する。plansは両language/componentでhistorical-task rowと
batch rowを各1件追加し、total 8 index records。selected preimage、ledger、
protected artifact、count/hash/status、public behavior、
`spec_coverage_audit.md`は不変。ownership、trace status、coverage credit、
deferral stateを変えないためaudit impactはない。

separate prerequisite commit/fresh replay後、migrationはdeclared 24 sections
だけをlanguage-local `249S.md#completion-evidence` redirectへ置換できる。
変更はsource 24件、本pair、`legacy_compactions.tsv`のexact 27 paths。256
linesは48 redirect-plus-separator linesとなり208行減、expected source diffは
24 additions/232 deletions。ledger impactは34 lines、`788 -> 822`、batch 1、
task 1、distinct 24 paths上のredirect 24、index record 8、expanded-inventory
hash 1件。batch rowを除くcanonical task/redirect/index payload 33 rowsのfrozen
SHA-256は
`71017a5197eb6bac76a8d6e079ee17f24301db20a19ca84c00df120e24155acf`。
source TSV、historical pair、indexesはimmutableとなる。

両commitはapplicableなindependent contract/equivalence、schema/test-
sufficiency、boundary、source/document/EN-JA、final-quality reviewを
**NO FINDINGS**まで行う。verificationはpreimage/anchor replay、generic schema/
link/fragment/full lint、checker/runner/metadata test、format、Cargo metadata、
warnings-denied Clippy、workspace test、CLI 5種、protected count/hash、
`git diff --check`、exact staging、全9 hard gates、score capなし`>=90/100`を
含む。push/stash mutationは禁止。

## Documentation-prerequisite evidence

initial selectionはactive H2 26節だった。reviewはHigh
`boundary_violation`を1件発見した。JA final active H2にはTask-269B completion
redirectが既にあり、parentはfinal H2両方を除外した。revised selection review
は**NO FINDINGS**で、exact `7/87 + 7/82 + 5/44 + 5/43 = 24/256` scope、
globally unique heading 24件、stable retained anchor、migration外のlive coverage
ownerをfreezeした。

independent contract/equivalenceとsource-documentation/EN-JA reviewは
**NO FINDINGS**。schema/test-sufficiency reviewはMedium `design_drift` 1件を
検出した。first draftはfuture inventory cardinalityだけをfreezeしhashを
freezeしていなかった。parentはschema-v1 payloadを独立生成し、上記33-row
`71017a...` hashをEN/JAでfreezeした。同じreviewerの再監査は
**NO FINDINGS**。reviewはchronology、authority、API/profile、validation/
precedence、Typed/final/unchanged-obligation ownership、exact tests/historical
hash、consumer/no-runner boundary、deferral、全link/fragment、preimage/anchor
24件、exclusion、nine-path scope、future delta、ledger arithmetic、audit
no-impactをreplayした。

full prerequisite-state verificationはchecker/runner lint各`15/15`、checker/
runner library `530/530`/`600/600`、metadata `137/137`がPASS。
`cargo fmt --all --check`、offline Cargo metadata、warnings-denied all-target/
all-feature Clippy、frontend benchmarkを含むfull all-target/all-feature
workspace suiteもPASSした。CLI 5種は各exit zero、warnings 23/errors 0で、
frozen stdout hash 5件をすべて再現した。

protected count/path hashはspecification 64、`.miz` 343、expectation 435、
checker production 30、runner production 90、Cargo 21をexactly reproduceし、
zero protected diffが全frozen content hashを保持する。trace、coverage audit、
788-line ledger/physical hash、current expanded-inventory hash、source-TSV hash
2件、preimage 24件、future 33-row hash、exact nine-path scope、
`git diff --check`を再現した。exact staging、commit、fresh inventoryは未完了。
selection `0/1`からverification `0/0`へのexternal
`origin/main` advanceは上記report-only `repo_metadata_conflict`で、worktree
scope/protected stashは不変。

final independent read-only quality reviewは**NO FINDINGS**。全9 hard gates
PASS、score capなし、valid score `100/100` (`20/20/15/15/10/10/5/5`)。
independent replayはexact scope、全preimage、source/inventory hash、exclusion、
protected surface、EN/JA ownership、review closure、verification health、external-
origin classificationを確認した。residual riskはexact staging、commit、separate
migration前のfresh inventoryだけである。

## Migration evidence

prerequisiteは`2b5e1590a6e187e7d5285f61f4bc7a12783168af`としてcommit済み。
直後のfresh inventoryはclean、`origin/main...HEAD=0/1`、protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`不変で、編集前にsource-TSV
preimage/anchor 24件をすべてreplayした。

declared whole H2 24節だけをlanguage-local completion redirect 24件へ置換した。
physical shapeはexact `256 -> 48`、net 208行削減、source deltaはfrozen
`+24/-232`。forbidden raw heading 24件はすべて消失し、各sourceはredirect
exact 1件とfrozen neighboring anchorを保持する。excluded final section両方と
live coverage ownerは不変。

byte-sorted schema-v1 ledgerは822 physical lines、physical SHA-256
`1a3a07297f4f0aee4b13274df44322b52cf92bf71f0ed40824debd7d0aba6c59`。
generic lintはfrozen 33-row expanded-inventory SHA-256
`71017a5197eb6bac76a8d6e079ee17f24301db20a19ca84c00df120e24155acf`
とbatch/task/redirect/distinct-path/indexのexact cardinality `1/1/24/24/8`を
受理する。migration diffはsource 24件、本batch contract pair、ledgerのexact
27 paths。immutable source TSV、historical contract、Task Index 4件、protected
artifact、trace、coverage auditは不変で、frozen no-impact decisionを維持する。

independent migration-equivalence、schema/test-sufficiency、source-
documentation/EN-JA reviewは**NO FINDINGS**。全24 preimage、anchor、redirect、
forbidden heading、exclusion、historical owner、exact source/postimage delta、
ledger schema/order/hash/cardinality、paired redirect family 12件、不変のaudit/
protected surfaceをreplayした。

full migration-state verificationはchecker/runner lint各`15/15`、checker/
runner library `530/530`/`600/600`、metadata `137/137`がPASS。
`cargo fmt --all --check`、offline Cargo metadata、warnings-denied all-target/
all-feature Clippy、frontend benchmark 3件を含むfull all-target/all-feature
workspace suiteがPASSした。CLI 5種は各exit zero、warnings 23/errors 0で、
frozen stdout hash 5件を再現した。protected count/path/content hashはexactly
reproduceし、zero protected diffがspecification、test、expectation、production、
Cargo、trace、coverage audit、source TSV、historical contract、Task Indexを
保持する。822-line ledger physical hash、frozen expanded-inventory hash/
cardinality、exact 27-path scope、source delta、redirect/anchor replay、
`git diff --check`を再現した。

final independent read-only quality reviewは**NO FINDINGS**。全9 hard gates
PASS、score capなし、valid score `100/100` (`20/20/15/15/10/10/5/5`)。
independent replayはexact scope、preimage/postimage、hash、schema cardinality、
redirect/anchor、exclusion、protected surface、audit no-impact、verification
health、HEAD `2b5e1590a6e187e7d5285f61f4bc7a12783168af`、
`origin/main...HEAD=0/1`、protected stash identityを確認した。residual riskは
exact staging、commit、post-commit inventoryだけである。

## Handoff

verified exact 27-path migrationをexact-stage/commitし、next checker compaction
family選定前にfresh post-commit inventoryを行う。parent/final-quality reviewは
`xhigh`、bounded independent reviewは`high`を用いた。
