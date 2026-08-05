# Task DOC-249PI-DOC-REVIEW-COMPACT: property-type review-evidence compaction

> canonical English:
> [../en/DOC-249PI-DOC-REVIEW-COMPACT.md](../en/DOC-249PI-DOC-REVIEW-COMPACT.md)。

本maintenance contractはchecker-only historical review family 1件をfreezeする。
language behavior、test intent、API、diagnostic、traceability、coverageを変更できない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-249PI-DOC-REVIEW-COMPACT` |
| Status | Documentation prerequisite committed。exact migration、independent reviews、full verification、final quality complete。exact staging/commitが残る。 |
| Purpose | 全durable implementation/runner ownerを保持し、repeated Task-249PI documentation-prerequisite/frozen-review evidenceをcentralizeする。 |
| Owners | migration policy、historical [249PI](./249PI.md#completion-evidence)、[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Consumers | checker source paths 8件、Task Index 4件、post-migration schema-v1 ledger/lint |
| Sequence | `4c3f74b0` -> `7e194bb3` -> `73a34f94` -> `52cf07be` |
| Documentation prerequisite | `6796433eb27f16768a36fb88e0fbd6bae43ea412` |
| Readiness | clean selection HEAD `bee5a905c3e0b291018a33165b382d14bb5eb9fd`、`origin/main...HEAD=0/16`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。dependency-ready。 |

## Authority And Classification

authorityはuser-approved checker-first compaction program、
[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、historical
contractのretained canonical/test owners、reviewed historyである。source behaviorは
normativeではない。

| Class | Decision |
|---|---|
| `design_drift` | checker 8 sectionsが同じ`7e194bb3` prerequisite freeze/review checkpointを反復し、historical contractがshared evidence ownerとなる。 |
| `spec_gap` / `test_gap` | 本structural taskにはなし。Task-249PI historical classification/findings/closuresは不変。 |
| `source_drift` / `source_undocumented_behavior` | 導入しない。production sourceはprotected。 |
| `test_expectation_drift` | なし。specification、`.miz`、expectations、sidecars、traceはprotected。 |
| `boundary_violation` | duplicated plan headings、source-type API/implementation-verification sections、全Typed/final/payload-family owner、checker/runner TODO、全runner sections、implementation sections、active APIs、audits、unlisted contentを保持して回避する。tempting plan sectionsは同一headingがTODO ownersにもありschema-v1 global forbidden-heading enforcementに違反するため除外する。 |
| `repo_metadata_conflict` | current `0/16`はreport-onlyでrepairしない。fetch/reset/push/stash mutationは禁止。 |

## Frozen Preimage And Anchors

[`DOC-249PI-DOC-REVIEW-COMPACT.sources.tsv`](../DOC-249PI-DOC-REVIEW-COMPACT.sources.tsv)
はbyte-sorted 8 rows、comments 2行、final LFを持つ。data-row SHA-256は
`f4acd99daffb0d77a53ef2ca76735f4f88c64f2313f245016be2f6a4cb2341e5`、
complete-file SHA-256は
`5d61e5c9982432deb1a671ed45168ca2a811b33981cf90cd6d2dfb5657220d2e`。

selectionはchecker 8 paths上のglobally unique H2 8 sections、physical 130行、EN
`4/76`、JA `4/54`。nested heading/table/fenceはない。retained EN
preceding/following ownersは次の通り。

| Source | Retained anchors |
|---|---|
| `bilingual_sync_audit.md` | `## Task 264 Frozen-Contract Synchronization` / `## Task 249PI Implementation Synchronization` |
| `module_boundary_audit.md` | `## Task 264 Frozen Module Boundary` / `## Task 249PI Implemented Module Boundary` |
| `source_spec_audit.md` | `## Task 264 Frozen Source/Specification Status` / `## Task 249PI Implemented Source/Specification Audit` |
| `source_type.md` | `## Task 249PI Frozen Property-Implementation Composition` / `## Task 249PI Implementation Verification` |

JA companionsはmatching levels/language-local equivalent anchorsを持つ。全8 headingsは
uniqueで、preimageにTask-249PI contract/index/ledger identityはない。

## Frozen Protected Baseline

prerequisite/migrationのexpected deltaは全rowでzero。

| Surface | Paths | Path SHA-256 | Content SHA-256 |
|---|---:|---|---|
| specification | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

traceは`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`、
coverage auditは`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`、
678-line ledgerは
`a26fe1fedd9f6b634de66daff85682d3ef63871242df77953eb4b881ec2a1d3a`。
expected CLI stdout hashesはplan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。

## Scope, Verification, And Exit

prerequisiteはexact 9 paths、本pair、historical pair、source TSV、four plansを
変更する。plansはhistorical-task/batch rows、計8 index rowsを追加する。selected
preimages、ledger、protected artifacts、counts/hashes/statuses、public behavior、
`spec_coverage_audit.md`は不変。ownership/trace status/creditが変わらないためaudit
impactはない。

separate prerequisite commitとfresh replay後、migrationはdeclared 8 sectionsだけを
language-local `249PI.md#completion-evidence` redirectへ置換できる。変更は8 sources、
本pair、`legacy_compactions.tsv`の11 paths。130行はredirect+separator 16行、114行
削減となる。ledger impactはbatch 1、task 1、distinct 8 paths上のredirect 8、index
8、expanded-inventory hash 1件。source TSV、historical pair、indexesはimmutable。

両commitはapplicable independent contract/equivalence、test-sufficiency、boundary、
source/document/EN-JA、final-quality reviewsを**NO FINDINGS**まで要求する。
verificationはpreimage/anchor replay、generic schema/link/fragment/full lint、checker/
runner/metadata tests、format、Cargo metadata、warnings-denied Clippy、workspace
tests、five CLIs、protected counts/hashes、`git diff --check`、exact staging、全9 hard
gates、capなしscore `>=90/100`。push/stash mutationは禁止。

## Documentation-Prerequisite Evidence

initial inventoryはMedium `boundary_violation`を発見した。tempting plan headingsは
retained TODO ownersにも存在しschema-v1 global forbidden-heading enforcementに失敗
するため、selectionをglobally unique source-type documentation-verification pairと
audit summaries 6件へ修正した。そのunique historical hashes/counts/review claimsを
historical pairへ移した。independent specification/equivalenceとtest-sufficiency/
schema reviewsは**NO FINDINGS**。source/document reviewが発見したreview-history
claims 2件とchecker-plan owner linksの欠落を両言語で修正し、re-reviewは
**NO FINDINGS**。Rust、schema、test、trace、coverage、追加document変更は不要。

全`8/130` preimagesはfrozen hashでreplayし、globally unique flat H2である。両TSV
hashes、adjacent anchors、chronology、exact API/error/site claims、ownership/semantic
exclusions、exact nine-path scope/index 8 rows、protected no-op、audit no-impact、
language-local links、future schema-v1 `1/1/8/8` ledger planを確認した。recursive
pairing/manifest/link/fragment lintは`1/1`、full checker/runner lintは各`15/15`、
checker/runner librariesは`530/530`と`600/600`、runner metadataは`137/137`をPASS。

`cargo fmt --all --check`、offline Cargo metadata、warnings-denied all-target/
all-feature Clippy、benchmarksを含むfull all-target/all-feature workspace suiteは
PASS。five CLIsは各exit zero、stderr 23行、errors zeroで全frozen stdout hashを
再現した。specification、`.miz`、expectation、checker/runner production、Cargo、
trace、coverage audit、678-line ledgerはdelta zeroで、immutable source TSVはfrozen
full-file hashを持つ。`git diff --check`はPASS。final read-only qualityは
**NO FINDINGS**、全9 hard gates PASS、score capなし、valid scoreは`100/100`
(`20/20/15/15/10/10/5/5`)。prerequisiteは
`6796433eb27f16768a36fb88e0fbd6bae43ea412`としてseparate commitされた。

## Migration Evidence

fresh post-prerequisite inventoryは`origin/main...HEAD=0/17`でclean、protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`は不変。edit前にimmutable
preimages 8件/130行と全frozen hashをreplayした。

mechanical migrationはdeclared checker sources 8件、本EN/JA pair、
`legacy_compactions.tsv`のexact 11 pathsを変更する。complete sections 8件だけを
language-local redirectへ置換し、physical 130行はredirect+separator 16行、114行
削減となる。source diffは8 additions/122 deletions。duplicated plans、全TODO、
frozen source-type API、implementation/implementation-verification sections、
Typed/final/payload owners、全runner document、trace、coverage、unlisted contentを
保持する。

ledgerは696 physical lines。本batchはtask 1、distinct 8 source paths上のredirect
8、index records 8を追加した。expanded-inventory SHA-256は
`b5d183379643e68f5b87b530c59b8e5425dc2b3a286eaaea54b6fee116a1ea76`、
complete physical SHA-256は
`7f5ef689d418f8605282e90ba67b446a3f404031af99f44d5683a5158e1d16e8`。
immutable source TSVは
`5d61e5c9982432deb1a671ed45168ca2a811b33981cf90cd6d2dfb5657220d2e`。
focused generic-ledger/link/fragment lintと`git diff --check`はPASS。

independent equivalence/boundary、test-sufficiency/schema、source-documentation/
EN-JA reviewsは**NO FINDINGS**。全preimage/postimage/redirect/neighbor anchor、unique
historical claim、retained owner、ledger relation/hash、protected surface、audit
no-impact decisionをreplayした。existing generic schema-v1 lintは十分で、Rust、
schema、test、fixture、expectation、trace、coverage、追加document変更は不要。

full checker/runner lintは各`15/15`、checker/runner librariesは`530/530`と
`600/600`、runner metadataは`137/137`をPASS。format、offline Cargo metadata、
warnings-denied all-target/all-feature Clippy、benchmarksを含むfull all-target/
all-feature workspace suite、`git diff --check`はPASS。five CLIsは各exit zero、
stderr 23行、errors zeroで全frozen stdout hashを再現した。protected specification、
`.miz`、expectation、checker/runner production、Cargo、trace、coverage audit、
immutable source TSVは不変。final read-only qualityは**NO FINDINGS**、全9 hard
gates PASS、score capなし、valid scoreは`100/100`
(`20/20/15/15/10/10/5/5`)。exact staging/task-only commitが残る。

## Handoff

exact staging/task-only commitを完了する。その後next schema-v1-safe checker familyを
clean fresh inventoryする。parentは`xhigh`、independent reviewsは`high`、
deterministic inventoryは`medium`。
