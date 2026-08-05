# Task DOC-248P-DOC-REVIEW-COMPACT: property-context review-evidence compaction

> canonical English:
> [../en/DOC-248P-DOC-REVIEW-COMPACT.md](../en/DOC-248P-DOC-REVIEW-COMPACT.md)。

本maintenance contractはchecker-only historical review familyをfreezeする。
language behavior、test intent、API、diagnostics、traceability、coverageを
変更できない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-248P-DOC-REVIEW-COMPACT` |
| Status | documentation prerequisiteはcommit済み。exact migration、independent migration reviews、full verification、final qualityは完了。exact staging/commitは未完。 |
| Purpose | durable implementation/runner ownerをすべて保持し、反復するTask-248P documentation-prerequisite/frozen-review evidenceを集約する。 |
| Owners | migration policy、historical [248P](./248P.md#completion-evidence)、[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Consumers | checker source 8 paths、4 Task Indexes、migration後のschema-v1 ledger/lint |
| Sequence | `db8c39e3` -> `1e3fa789` -> `1637380d` -> `4c3f74b0` |
| Documentation prerequisite | `b483bc298cc459e2b294bd07726ca6721d9fe298` |
| Readiness | clean selection HEAD `d94dfd6330c1dd067be8b26c814ac95e077b2639`、`origin/main...HEAD=0/14`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。dependency-ready。 |

## Authority And Classification

authorityはuser-approved checker-first compaction program、
[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、
historical contractのretained canonical/test owners、reviewed historyである。
source behaviorはnormativeではない。

| Class | Decision |
|---|---|
| `design_drift` | checker 8 sectionsが同じ`1e3fa789` prerequisite review/freeze checkpointを反復し、historical contractがshared evidence ownerとなる。 |
| `spec_gap` / `test_gap` | 本structural taskにはなし。Task-248P historical classifications/closuresは不変。 |
| `source_drift` / `source_undocumented_behavior` | 導入しない。production sourceはprotected。 |
| `test_expectation_drift` | なし。specification、`.miz`、expectations、sidecars、traceはprotected。 |
| `boundary_violation` | frozen plan contract、implementation/verification results、checker/runner TODO、全runner documents、active module/API owners、未列挙sectionsを保持して回避する。Task 258B4Bはcandidate H2がnested durable contentを混在するため別途除外する。 |
| `repo_metadata_conflict` | current `0/14`はreport-onlyでrepairしない。fetch/reset/push/stash mutationは禁止。 |

## Frozen Preimage And Anchors

[`DOC-248P-DOC-REVIEW-COMPACT.sources.tsv`](../DOC-248P-DOC-REVIEW-COMPACT.sources.tsv)
はbyte-sorted 8 rows、comments 2行、final LFを持つ。data-row SHA-256は
`cd19b044410fa454125c80ac1ea711dfbd0bb8eb0e6e05cb9c20a81c94510c84`、
complete-file SHA-256は
`ba3029c35715c3450c2d3bd863e4904ef7e940d568d3321f5644b5faf1e70285`。

selectionはchecker 8 paths上のunique H2 8 sections、physical 113行、EN
`4/60`、JA `4/53`。nested heading/table/fenceはない。retained EN
preceding/following ownersは次の通り。

| Source | Retained anchors |
|---|---|
| `00.crate_plan.md` | `## Task 248P Frozen Property Binding-Context Prerequisite` / `## Task 248P Implementation Result` |
| `bilingual_sync_audit.md` | `## Task 264R Implementation Synchronization` / `## Task 248P Implementation Synchronization` |
| `module_boundary_audit.md` | `## Task 264R Implemented No-Checker-Source Boundary` / `## Task 248P Implemented One-File Checker Boundary` |
| `source_spec_audit.md` | `## Task 264R Implemented Source/Specification Status` / `## Task 248P Implemented Source/Specification Status` |

JA companionsはmatching levels/language-local equivalent anchorsを持つ。全8
headingsはuniqueで、preimageにTask-248P contract/index/ledger identityはない。

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
660-line ledgerは
`f3fdbf5111f4c17cf19088f97844dfa4eeb8ac5b2051866e1c86f99b44efc301`。
expected CLI stdout hashesはplan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。

## Scope, Verification, And Exit

prerequisiteはexact 9 paths、本pair、historical pair、source TSV、four plansを
変更する。plansはhistorical-task/batch rows、計8 index rowsを追加する。selected
preimages、ledger、protected artifacts、count/hash/status、public behavior、
`spec_coverage_audit.md`は不変。ownership/trace status/creditが変わらないため
audit impactはない。

separate prerequisite commitとfresh replay後、migrationはdeclared 8 sections
だけをlanguage-local `248P.md#completion-evidence` redirectへ置換できる。変更は
8 sources、本pair、`legacy_compactions.tsv`の11 paths。113行はredirect+
separator 16行、97行削減となる。ledger impactはbatch 1、task 1、distinct 8
paths上のredirect 8、index 8、expanded-inventory hash 1件。source TSV、
historical pair、indexesはimmutableとなる。

両commitはapplicable independent contract/equivalence、test-sufficiency、
boundary、source/document/EN-JA、final-quality reviewsを**NO FINDINGS**まで
要求する。verificationはpreimage/anchor replay、generic schema/link/fragmentと
full lint、checker/runner/metadata tests、formatting、Cargo metadata、
warnings-denied Clippy、workspace tests、five CLIs、protected counts/hashes、
`git diff --check`、exact staging、全9 hard gates、capなしscore `>=90/100`。
push/stash mutationは禁止。

## Documentation-Prerequisite Evidence

independent contract/equivalence、test-sufficiency/schema、
source-documentation/EN-JA reviewsは**NO FINDINGS**で終了した。全`8/113`
preimages、両TSV hashes、chronology、unique API/invariant/classification claims、
ownership/exclusions、exact 9-path scope/index 8 rows、audit no-impact、protected
no-op、language-local links、future schema-v1 `1/1/8/8` ledger planをreplayした。
Rust/schema/traceability/coverage/additional documentation変更は不要である。

checker/runner lintは各`15/15`、checker/runner librariesは`530/530`と
`600/600`、runner metadataは`137/137`を通過した。`cargo fmt --all --check`、
offline Cargo metadata、warnings-denied all-target/all-feature Clippy、full
offline all-target/all-feature workspace suite、`git diff --check`はPASS。
five CLIsは各exit zero、stderr 23行、errors zeroで全frozen stdout hashを再現した。
specification、`.miz`、expectation、checker/runner production、Cargo、trace、
coverage audit、660-line ledgerはdelta zeroで、immutable source TSVはfrozen
full-file hashを保持する。final read-only qualityは**NO FINDINGS**、全9 hard
gatesはPASS、score capなし、valid scoreは`100/100`
(`20/20/15/15/10/10/5/5`)。exact staging/dedicated prerequisite commitが残る。

## Migration Evidence

prerequisiteは`b483bc298cc459e2b294bd07726ca6721d9fe298`としてcommitされた。
fresh post-commit inventoryは`origin/main...HEAD=0/15`でclean、protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`は不変で、edit前にfrozen
preimages 8件/113行をすべてreplayした。

mechanical migrationはdeclared checker sources 8件、本EN/JA pair、
`legacy_compactions.tsv`のexact 11 pathsを変更する。complete sections 8件だけを
language-local redirectへ置換し、physical 113行はredirect+separator 16行、97行
削減となる。source diffは8 additions/105 deletions。全TODO、runner、frozen
plan、implementation、implementation-verification、active API、audit、trace、
coverage、未列挙ownerを保持する。

ledgerは678 physical lines。batchはtask 1、distinct 8 source paths上のredirect
8、index records 8を追加した。expanded-inventory SHA-256は
`d3549958ec578a603d18a15d62175db616cd60d312e733e5bd3574ad9a534a21`、
complete physical SHA-256は
`a26fe1fedd9f6b634de66daff85682d3ef63871242df77953eb4b881ec2a1d3a`。
immutable source TSVは
`ba3029c35715c3450c2d3bd863e4904ef7e940d568d3321f5644b5faf1e70285`
のまま。focused generic-ledger/link/fragment lintと`git diff --check`はPASS。

independent equivalence/boundary、test-sufficiency/schema、
source-documentation/EN-JA reviewsはstale handoffのLow `design_drift` 1件を修正後、
**NO FINDINGS**で終了した。全preimage/postimage/anchor/redirect/unique claim/
retained owner、ledger relation/hash、protected scope、audit no-impactをreplayした。
existing generic schema-v1 lintは十分で、Rust/schema/fixture/expectation/test/
trace/coverage/additional documentation変更は不要である。

checker/runner lintは各`15/15`、checker/runner librariesは`530/530`と
`600/600`、runner metadataは`137/137`を通過した。format、offline Cargo
metadata、warnings-denied all-target/all-feature Clippy、full offline all-target/
all-feature workspace suite、`git diff --check`はPASS。five CLIsは各exit zero、
stderr 23行、errors zeroで全frozen stdout hashを再現した。protected
specification、`.miz`、expectation、checker/runner production、Cargo、trace、
coverage audit、immutable source TSVは不変。final read-only qualityは
**NO FINDINGS**、全9 hard gatesはPASS、score capなし、valid scoreは`100/100`
(`20/20/15/15/10/10/5/5`)。exact staging/task-only commitが残る。

## Handoff

exact staging/task-only commitが残る。clean post-commit inventory後、次の
schema-v1-safe checker duplication familyを選ぶ。current lower-risk candidateは
`249PI`で、`264`、`269A`、`259`にはより厳格なretained-owner reviewが必要である。
parentは`xhigh`、independent reviewは`high`、deterministic inventoryは`medium`。
