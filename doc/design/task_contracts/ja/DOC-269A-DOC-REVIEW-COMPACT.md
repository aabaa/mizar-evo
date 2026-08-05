# Task DOC-269A-DOC-REVIEW-COMPACT: named-witness review-evidence compaction

> canonical English:
> [../en/DOC-269A-DOC-REVIEW-COMPACT.md](../en/DOC-269A-DOC-REVIEW-COMPACT.md)。

本maintenance contractはchecker-only historical review family 1件をfreezeする。
language behavior、test intent、API、diagnostic、traceability、coverageを変更できない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269A-DOC-REVIEW-COMPACT` |
| Status | Documentation-prerequisite reviews/full verification/final quality complete。exact staging/commitが残る。migrationは未開始。 |
| Purpose | 全durable implementation/runner ownerを保持し、repeated Task-269A documentation-prerequisite/frozen-review evidenceをcentralizeする。 |
| Owners | migration policy、historical [269A](./269A.md#completion-evidence)、[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Consumers | checker source paths 8件、Task Index 4件、post-migration schema-v1 ledger/lint |
| Sequence | `52cf07be` -> `1360a9c0` -> `f548ceb9` -> `3d462b1f` |
| Readiness | clean selection HEAD `6b139bf1ab37cdc6c0d7239d202802db1efe113f`、`origin/main...HEAD=0/18`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。dependency-ready。 |

## Authority And Classification

authorityはuser-approved checker-first compaction program、
[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、historical
contractのretained canonical/test owners、reviewed historyである。source behaviorは
normativeではない。

| Class | Decision |
|---|---|
| `design_drift` | checker 8 sectionsが同じ`1360a9c0` documentation-prerequisite freeze/review checkpointを反復し、historical contractがshared evidence ownerとなる。 |
| `spec_gap` / `test_gap` | 本structural taskにはなし。Task-269A historical classification/test intent/findings/closuresは不変。 |
| `source_drift` / `source_undocumented_behavior` | 導入しない。production sourceはprotected。 |
| `test_expectation_drift` | なし。specification、`.miz`、expectations、sidecars、traceはprotected。 |
| `boundary_violation` | frozen plan/API、proof-local owner、binding/Typed/final/payload/semantic/statement owners、implementation/active sections、checker/runner TODO、全runner sections、trace、coverage、unlisted contentを保持して回避する。Task264 candidate数件はprior Task-249PI redirectをstructurally含みschema v1でsafeにnestできないため選択しない。 |
| `repo_metadata_conflict` | current `0/18`とTask264をrejectする際に観測したunrelated legacy mizar-test Task Index identity `264`はreport-onlyでrepairしない。fetch/reset/push/stash mutationは禁止。 |

## Frozen Preimage And Anchors

[`DOC-269A-DOC-REVIEW-COMPACT.sources.tsv`](../DOC-269A-DOC-REVIEW-COMPACT.sources.tsv)
はbyte-sorted 8 rows、comments 2行、final LFを持つ。data-row SHA-256は
`4e38239f3d55704e3ac42131144ee72d990a5a9b6bf6951ed0acf329da06edfa`、
complete-file SHA-256は
`2cd696c1957ade3232b5c66e88325514b0afd35352285a7fa718842e764661d2`。

selectionはchecker 8 paths上のglobally unique H2 8 sections、physical 157行、
EN `4/82`、JA `4/75`。nested heading/table/fence/existing ledger redirectはない。
retained EN preceding/following ownersは次のとおり。

| Source | Retained anchors |
|---|---|
| `00.crate_plan.md` | `## Checker Task 269A Frozen Named-Witness Binding Plan` / `## Checker Task 269A Implementation Measurement` |
| `bilingual_sync_audit.md` | `## Task 264 Active Implementation Synchronization` / `## Task 269A Active Implementation Synchronization` |
| `module_boundary_audit.md` | `## Task 264 Implemented Module Boundary` / `## Task 269B module-boundary no-op` |
| `source_spec_audit.md` | `## Task 264 Implemented Source/Specification Audit` / `## Task 269A Implemented Source/Specification Audit` |

JA companionはmatching level/language-local equivalent anchorsを持つ。preimageに
Task-269A contract/index/ledger identityはない。

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
696-line ledgerは
`7f5ef689d418f8605282e90ba67b446a3f404031af99f44d5683a5158e1d16e8`。
expected CLI stdout hashはplan
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
language-local `269A.md#completion-evidence` redirectへ置換できる。変更は8 sources、
本pair、`legacy_compactions.tsv`の11 paths。157行はredirect+separator 16行、141行
削減となる。ledger impactはbatch 1、task 1、distinct 8 paths上のredirect 8、index
8、expanded-inventory hash 1件。source TSV、historical pair、indexesはimmutable。

両commitはapplicable independent contract/equivalence、test-sufficiency、boundary、
source/document/EN-JA、final-quality reviewsを**NO FINDINGS**まで要求する。
verificationはpreimage/anchor replay、generic schema/link/fragment/full lint、checker/
runner/metadata tests、format、Cargo metadata、warnings-denied Clippy、workspace
tests、five CLIs、protected counts/hashes、`git diff --check`、exact staging、全9 hard
gates、capなしscore `>=90/100`。push/stash mutationは禁止。

## Documentation-Prerequisite Evidence

initial independent inventory/schema/boundary reviewは本exact familyを
**NO FINDINGS**で選択し、prior redirect overlapのためTask264をrejectした。
equivalence reviewはMedium historical defectを2件発見した。first draftはlater
Task-269 authorityをimportし、exact historical verification/module-guard factsを
欠いていた。source/document reviewは同authority gap、implementation hash/repository
evidence不足、base-binding fingerprintを誤ってtransactionと呼ぶLow findingを
発見した。全てEN/JAで修正し、independent equivalence、test-sufficiency/schema/
boundary、source-documentation/EN-JA re-reviewは**NO FINDINGS**となった。

全`8/157` preimagesはfrozen hashでreplayしglobally unique flat H2である。source
TSVの両hash、byte order/final LF、adjacent anchors、chronology、exact historical
authority/API/validation/runner claims、semantic exclusions、exact nine-path scope/
index 8 rows、protected no-op、audit no-impact、language-local future links、schema-v1
`1/1/8/8` ledger planを確認した。recursive contract/index/link/fragment lintは
`1/1`、full checker/runner lintは各`15/15`、checker/runner libraryは
`530/530`、`600/600`、runner metadataは`137/137`をPASSした。

`cargo fmt --all --check`、offline Cargo metadata、warnings-denied all-target/
all-feature Clippy、benchmarksを含むfull all-target/all-feature workspace suiteは
PASS。CLI 5件は各exit zero、stderr 23行、errors zeroで全frozen stdout hashを
再現した。specification、`.miz`、expectation、checker/runner production、Cargo、
trace、coverage audit、696-line ledgerはdelta zero、source TSVはfrozen complete-
file hashを保持する。`git diff --check`はPASS。final read-only qualityは
**NO FINDINGS**、全9 hard gates PASS、score capなし、valid scoreは`100/100`
(`20/20/15/15/10/10/5/5`)。exact staging/task-only prerequisite commitが残る。

## Migration Evidence

未開始。

## Handoff

exact staging/task-only prerequisite commitを完了する。その後immutable
preimagesをfresh replayしてexact migrationを行う。parentは`xhigh`、independent
reviewsは`high`、deterministic inventoryは`medium`。
