# Task DOC-249M-ACTIVE-EVIDENCE-COMPACT: mode-RHS active evidence compaction

> canonical English:
> [../en/DOC-249M-ACTIVE-EVIDENCE-COMPACT.md](../en/DOC-249M-ACTIVE-EVIDENCE-COMPACT.md)。

本maintenance contractはchecker-only historical completion-evidence family
1件をfreezeする。language behavior、test intent、public API、diagnostic、
traceability、coverageを変更できない。

## Identity と status

| Field | Frozen value |
|---|---|
| Task | `DOC-249M-ACTIVE-EVIDENCE-COMPACT` |
| Status | Migration、全independent review、full verification、final quality完了。exact staging/commit待ち。 |
| Purpose | durable/frozen ownerと全excluded mixed evidenceを保持し、repeated Task-249M active implementation evidenceを集約する。 |
| Owners | migration policy、historical [249M](./249M.md#completion-evidence)、[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Consumers | checker source 16 paths、Task Index 4件、future schema-v1 ledger/lint |
| Sequence | `8c3fa20a` -> `b1b41012` -> `2baf83d3` -> `1fb192e3` |
| Readiness | clean selection HEAD `1ad52ed39cfa98d9a9b08f639e2d75f123de80cf`、`origin/main...HEAD=0/24`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。revised selection reviewは**NO FINDINGS**。 |

## Authority と classification

authorityはuser-approved checker-first compaction program、
[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、
historical contractのretained canonical/test owner、review済みGit historyである。
source behaviorはnormativeではない。

| Class | Decision |
|---|---|
| `design_drift` | checker 16節がpaired component 8 pathsにTask-249M active completionを反復する。historical contractをshared completion-evidence ownerとする。 |
| `spec_gap` / `test_gap` | 本structural taskにはない。authority、test intent、prior closure、deferralは不変。 |
| `source_drift` / `source_undocumented_behavior` | 導入しない。production sourceはprotected。 |
| `test_expectation_drift` | なし。specification、`.miz`、fixture、sidecar、expectation、traceはprotected。 |
| `boundary_violation` | 最初の18-section案はchecker bilingual-audit H2両方を含んだ。JA H2にはolder Task-258B4A final-review 5行が混在し、schema v1でwhole sectionをTask 249Mへ移せないためEN/JA両方を除外した。frozen/addendum/selection、runner/TODO/coverage owner、Task 262以降、全unlisted artifactも除外する。 |
| `repo_metadata_conflict` | 現在の`0/24`はreport-only。fetch/reset/push/stash mutationはunauthorizedで、task-only commit targetは識別可能。 |

## Frozen preimage と anchor

[`DOC-249M-ACTIVE-EVIDENCE-COMPACT.sources.tsv`](../DOC-249M-ACTIVE-EVIDENCE-COMPACT.sources.tsv)
はbyte-sorted data 16 rows、comments 2行、final LFを持つ。data-row SHA-256は
`ed26edcf9c657c747383b8c7aaf0f175fefb826ecc24637e92f6d9d2e0ccdfe9`、
complete-file SHA-256は
`4ffed4391aced54ebfb2ab13ed493f594359c858f9b543be645516be2669b658`。

selectionはchecker distinct 16 paths上のglobally exhaustive H2 16節、159
physical linesで、EN `8/81`、JA `8/78`。全selected sectionはflatで、nested
heading/table/fence/existing redirect/removable fragmentへのinbound linkを持たない。
raw heading 14種はglobally exhaustedし、EN/JA active-implementation-result
headingは各languageで2回、4 occurrence全部を選択する。retained anchorは次のとおり。

| Source | EN preceding / following owner | JA preceding / following owner |
|---|---|---|
| `00.crate_plan.md` | `## Task 262 Upper-Contract Commit And Task 249M Selection` / `## Task 262 Active Implementation Result` | `## Task 262 upper-contract commit と Task 249M selection` / `## Task 262 active implementation result` |
| `module_boundary_audit.md` | `## Task 249M Frozen Boundary` / `## Task 262 Active Module Boundary` | `## Task 249M frozen boundary` / `## Task 262 active module boundary` |
| `payload_family_decomposition.md` | `## Task 249M Mode-RHS Lower Family` / `## Task 249S Structure-Member Type Lower Family` | `## Task 249M mode-RHS lower family` / `## Task 249S structure-member type lower family` |
| `resolved_typed_ast.md` | `## Task 249M Mode-RHS Clone Addendum` / `## Task 262 Active Final Mode-Definition Ownership` | `## Task 249M mode-RHS clone addendum` / `## Task 262 active final mode-definition ownership` |
| `source_mode_definition.md` | `## Task 249M Lower-Contract Link` / `## Task 262 Active Implementation Result` | `## Task 249M lower-contract link` / `## Task 262 active implementation result` |
| `source_spec_audit.md` | `## Task 249M Frozen Future Public-Surface Audit` / `## Task 262 Active Source Audit` | `## Task 249M frozen future public-surface audit` / `## Task 262 active source audit` |
| `source_type.md` | `## Task 249M Frozen Standalone Mode-RHS Extension` / `## Task 249S Frozen Standalone Structure-Member Type Intake` | `## Task 249M frozen standalone mode-RHS extension` / `## Task 249S standalone structure-member type intake frozen contract` |
| `typed_ast.md` | `## Task 249M Mode-RHS Ownership Addendum` / `## Task 262 Active Mode-Definition Transaction` | `## Task 249M mode-RHS ownership addendum` / `## Task 262 active mode-definition transaction` |

selected preimageにはhistorical `249M` contract、batch/source-inventory identity、
Task Index row、ledger task/redirectが存在しない。excluded checker bilingual
recordとretained runner sectionは各自のevidence ownerのままである。

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
`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`、
762-line ledgerは
`512633c4d6b7f3f8c460a5e5ccd2a5b9717d2826626e08689b4a3205a8dadb11`、
expanded inventoryは
`3e081810f038edf8c3a75f9a222e02dcb8ea07d42b957d911df04ce8ad33b96f`。
expected CLI stdout hashはplan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。

## Scope、verification、exit

prerequisiteは本pair、historical pair、source TSV、plans 4件のexact 9 pathsを
変更する。plansはhistorical-task/batch rowを追加し、index rowsはtotal 8件。
selected preimage、ledger、protected artifact、count/hash/status、public behavior、
`spec_coverage_audit.md`は不変。ownership、trace status、coverage credit、
deferral stateを変えないためaudit impactはない。

separate prerequisite commit/fresh replay後、migrationはdeclared 16 sectionsだけを
language-local `249M.md#completion-evidence` redirectへ置換できる。変更はsource
16件、本pair、`legacy_compactions.tsv`のexact 19 paths。159 linesは32
redirect-plus-separator linesとなり127行減、expected source diffは
16 additions/143 deletions。ledger impactはbatch 1、task 1、distinct 16 paths上の
redirect 16、index record 8、expanded-inventory hash 1件。source TSV、historical
pair、indexesはimmutableとなる。

両commitはapplicableなindependent contract/equivalence、test sufficiency、
boundary、source/document/EN-JA、final-quality reviewを**NO FINDINGS**まで行う。
verificationはpreimage/anchor replay、generic schema/link/fragment/full lint、
checker/runner/metadata test、format、Cargo metadata、warnings-denied Clippy、
workspace test、CLI 5種、protected count/hash、`git diff --check`、exact staging、
全9 hard gates、score capなし`>=90/100`を含む。push/stash mutationは禁止。

## Documentation-prerequisite evidence

最初のselection reviewはHigh `boundary_violation` 1件を発見した。proposed JA
bilingual-audit H2にTask-258B4A final-review 5行がTask-249M evidenceとして
混在したため、parentはEN/JA bilingual H2両方を除外した。revised selection
re-reviewは**NO FINDINGS**で、exact `8/81 + 8/78 = 16/159` scope、globally
exhausted heading 14種、stable retained anchorをfreezeした。

independent contract/equivalence、schema/test-sufficiency、source-documentation/
EN-JA reviewはすべて**NO FINDINGS**。全section hash/line count、TSV hash 2件、
chronology、index rows 8件、owner link/fragment、exact API/profile/test/consumer/
deferral claim、audit no-impact、prerequisite 9-path scope、future `159 -> 32`、
`+16/-143`、`1/1/16/16/8` migration planをreplayした。最初のrecursive-link
lintはowning-plan linkとexact JA canonical markerの欠落を検出したが、
independent review前に修正し、その後recursive lintはPASSした。

full prerequisite-state verificationはchecker/runner lint各`15/15`、checker/
runner library `530/530`/`600/600`、metadata `137/137`がPASSした。
`cargo fmt --all --check`、offline Cargo metadata、warnings-denied all-target/
all-feature Clippy、frontend benchmarkを含むfull all-target/all-feature workspace
suiteがPASSした。CLI 5種は各exit zero、warnings 23/errors 0で、全frozen
stdout hashを再現した。

protected count/path hash 6組はexactly reproduceし、zero protected diffが全
frozen content hashを保持する。trace、coverage audit、762-line ledger、expanded-
inventory hash、source-TSV hash 2件、preimage 16件、exact 9-path scope、
`git diff --check`も再現した。final independent read-only qualityは
**NO FINDINGS**、全9 hard gates PASS、score capなし、valid `100/100`
(`20/20/15/15/10/10/5/5`)。residual riskはexact staging/commit/fresh inventoryと
separately frozen migrationだけである。

## Migration evidence

prerequisiteは`3d3f98767aa3818186f75e429dad468d97003ba7`としてcommit済み。
直後のfresh inventoryはclean、`origin/main...HEAD=0/25`、protected stash不変で、
migration前にsource TSV 16 preimages/anchorsをすべてreplayした。

declared whole H2 16節だけをlanguage-local completion redirect 16件へ置換した。
physical shapeはexact `159 -> 32`、net 127行削減。patience/histogram diffは
frozen source delta `+16/-143`を再現する。default Myers表示はEN/JA
`source_type.md`のseparator各1行をchurnとしてpairするため`+18/-145`だが、net
deltaは同じで、checked postimageは全sourceでredirect 1行とseparator 1行を持つ。
forbidden raw heading 14種はすべて消失し、redirect countは16件である。

byte-sorted schema-v1 ledgerは788 physical lines、physical SHA-256
`1702d79a198685ce8603f65dbdd2947f7d2c78e7b9ea3e76a150caac29a48da7`。
generic lintはexpanded-inventory SHA-256
`bb38229607a2a3eaa81e7b8d4ab8218c8ce42f0f86de91dd7471b3f205ed0b66`と
batch/task/redirect/distinct-path/indexのexact cardinality `1/1/16/16/8`を受理する。
migration diffはsource 16件、本batch contract pair、ledgerのexact 19 paths。
immutable source TSV、historical contract、Task Index 4件、protected artifact、
trace、coverage auditは不変で、frozen no-impact decisionを維持する。

independent equivalence/boundary、schema/test-sufficiency、source-documentation/
EN-JA reviewはすべて**NO FINDINGS**。recursive/full runner lint `15/15`、checker
lint `15/15`、checker/runner library `530/530`/`600/600`、metadata `137/137`、
`cargo fmt --all --check`、offline Cargo metadata、warnings-denied all-target/
all-feature Clippy、frontend benchmarkを含むfull all-target/all-feature
workspace suiteがすべてPASSした。CLI 5種は各exit zero、warnings 23/errors 0で、
frozen stdout hash 5件を再現した。

protected count/path hashはspecification 64、`.miz` 343、expectation 435、checker
production 30、runner production 90、Cargo 21を再現し、zero protected diffが各
frozen content hashを保持する。trace、coverage audit、source TSV、ledger、
forbidden-heading/redirect、exact scope、`git diff --check`もPASSした。staging
evidenceは未記録である。

final independent read-only quality reviewは**NO FINDINGS**。全9 hard gates
PASS、score capなし、valid score `100/100` (`20/20/15/15/10/10/5/5`)。
independent replayはexact scope、全preimage/postimage、ledger inventory、
protected surface、paired ownership、lint/format/metadata evidenceを確認した。
residual riskはexact staging、commit、post-commit inventoryだけである。

## Handoff

migration 19 pathsをexact-stage/commitし、次のchecker duplication familyを
fresh inventoryする。parentは`xhigh`を維持する。
