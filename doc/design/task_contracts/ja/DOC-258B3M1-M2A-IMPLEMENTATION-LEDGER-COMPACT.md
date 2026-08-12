# Task DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT: Early B3M Implementation-Ledger Compaction

> canonical English:
> [../en/DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT.md](../en/DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT.md)。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT` |
| Status | 完了。migrationはschema-2 ledgerに登録済みであり、task-local completion evidenceがcommitted migrationとclean replayを保存する。 |
| Purpose | Task-258B3M1/Task-258B3M2Aのcompleted implementation checklistsを集約し、documentation ledgersと全durable checker/runner ownersを残す。 |
| Historical owners | [Task 258B3M1](./258B3M1.md#completion-evidence)と[Task 258B3M2A](./258B3M2A.md#completion-evidence) |
| Plan indexes | [checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)と[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Selection HEAD | `b4f97b2ea5f9bba17bf084929214b749389b08b9` |
| Repository state | clean `main`、`origin/main...HEAD=0/6`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |
| Dependencies | historical documentation/implementation pairs 2組、lower-stage prerequisites、generic schema-2 ledger supportはselection HEADのancestors。 |

## Authority, Consumers, And Classification

authorityはuser-approved checker-first compaction program、
[`AGENTS.md`](../../../../AGENTS.md)、
[autonomous migration policy](../../autonomous_crate_development.md#migration-policy)、
historical contractsがlinkするretained canonical/task evidence、selected completed
sections 4件、durable owners。source behaviorはnormativeではない。generic
lint-policy consumerはrecursive contracts、links、fragments、plan indexes、section
anchors、manifest count/order/hash replayをownし、human readerがlanguage-local
redirectをconsumeする。

| Class | Decision |
|---|---|
| `design_drift` | checker EN/JA TODOがcentral historical owner外でcompleted B3M1/B3M2A implementation checklistsを反復する。本prerequisiteがmissing owner pairsを作る。 |
| `test_gap` | なし。existing generic schema-2 lintがowning task rows 2件、exact redirects 4件、indexes、links、fragments、hashes、counts、anchorsをcoverする。 |
| `boundary_violation` | task/source pairごとにimplementation-ledger section 1件だけを選び回避する。adjacent documentation ledgersと全lower-stage/frozen-contract/successor/runner/module/audit sectionsは残る。 |
| `spec_gap` / `source_drift` | 導入・修復しない。historical bounded task drift/closureはtime-local evidenceとして残る。 |
| `source_undocumented_behavior` / `test_expectation_drift` | 推測・変更しない。 |
| `repo_metadata_conflict` | selection時なし。historical Task-258B3M2A metadata incidentsはretained audit ownerだけに残る。 |

## Frozen Sources, Fingerprints, And Anchors

[`DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv`](../DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT.sources.tsv)
はbyte-sorted data rows 4件、comments 2件、final LFを持つ。data-row SHA-256は
`b4122c601f5fff6c2628a88163c1817c5fb5439cca9db4c9abcb816b13bb0c15`、
complete-file SHA-256は
`f0c3a76f37ff98b5e3e0553755eca0a63fb809f48fa50184144452ec68b75f56`。
source-local unique/unlinked H2 sectionsはnested heading/table/fence/redirectを含まず
合計55 physical lines。

| Task/source | Lines | Section SHA-256 | Previous H2 | Next H2 |
|---|---:|---|---|---|
| B3M1 EN checker TODO `4861-4875` | 15 | `d4bcfe36d6ccababe39e50fcb9932d1d3fb3eef8a5e23d3dac1ecbc2ca53ea50` | `## Checker Task 258B3M1 Documentation Ledger` | `## Checker Task 258B3M2A Documentation Ledger` |
| B3M1 JA checker TODO `4626-4639` | 14 | `72b6d7fe7feefec042e62173e6dae56b33fffa6da1e340b7161d5715b8dc16c7` | `## Checker Task 258B3M1 documentation ledger` | `## Checker Task 258B3M2A documentation ledger` |
| B3M2A EN checker TODO `4891-4903` | 13 | `10fbb26471d4389a0796de5488475a9e88208eeadf51f4b5fa95590709607e4d` | `## Checker Task 258B3M2A Documentation Ledger` | `## Checker Task 258B3M2B1 Frozen-Contract Ledger` |
| B3M2A JA checker TODO `4656-4668` | 13 | `9082b77bacd68cdf66536ba634e73106eb3ca2e9847596caf3a43f4bc068933e` | `## Checker Task 258B3M2A documentation ledger` | `## Checker Task 258B3M2B1 frozen-contract ledger` |

blameはB3M1 heading/bodyをimplementation
`cffd46f810fb05f2efc78859382f30678ffe1c3d`、trailing separatorをB3M2A
prerequisite `0847727f7a3d62c2e241aa96de546761a26f5e0c`へassignする。B3M2A
heading/bodyはimplementation `477fe251fa21a5fb3d0cbb9956a3c61ee14b648d`、trailing
separatorはB3M2B1 prerequisite `da68793d126c3105564d127b08800538f262e789`。
全件selection HEADのancestors。

## Owners, Scope, Prohibitions, And Deferrals

historical contractsはstable checker plan/statement/binding/Typed/Resolved owners、
runner plan/harness/boundary owners、authority/bilingual audits、coverage addendaへ
linkする。本prerequisiteはexact 11 paths、すなわちnew historical EN/JA pairs
2組、本EN/JA pair、immutable source TSV、checker/test EN/JA plan各3 Task Index rows
（historical tasks 2件 + 本batch）を変更する。selected TODO sectionsと
`legacy_compactions.tsv`は不変、task-contract countsは`64/64 -> 67/67`。

specification、`.miz`、fixture、expectation、sidecar、trace metadata、coverage
audit、production、Cargo、public API、diagnostic、active behaviorは禁止。
Task-258B3M1/B3M2A documentation ledgers、B3M2A lexer prerequisite、全successor
section、owner-local API/invariant/runner/audit/trace materialは残る。binding
publication、abbreviation、substitution、obligation、fact、proof result、goal、
theorem acceptance、active-corpus ownership、remaining witness-term familiesは
existing ownership/deferralを維持する。mapping/status/deferred reason/trace
linkage/coverage credit不変なのでcoverage-audit更新は不要。

## Protected Baseline And Expected Migration

| Surface | Paths | Path SHA-256 | Content SHA-256 |
|---|---:|---|---|
| specification | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

trace SHA-256は`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`、
coverage auditは`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`。
ledger baselineは904 lines、physical SHA-256
`e98a7d74b69a574e7362026bfafec8e5f9b2832fcae37e295bc02d048faa4abc`、25
batches、35 tasks、two task references、600 redirects、240 indexes。

five CLI hashesはplan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、parse
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、declaration
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、type
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、proof
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。

prerequisite commit/clean replay後、migrationはexact five paths、すなわちchecker
TODO EN/JA、本EN/JA pair、ledgerを変更する。selected 55 linesはlanguage-local
redirects 4件となりexact source diffは`+4/-51`。全recorded neighboring anchorsと
unselected sectionsはbyte-identical。

ledger impactは19 lines、`904 -> 923`。one batch、two canonical tasks、twelve
indexes、two source pathsのredirects 4件で、`task_ref`は追加しない。canonical
18-row expanded-inventory SHA-256は
`103e804ae1fe2e561b4c5047048cba5f0c659c43b625776c60a3f9828b3512cb`、expected
physical ledger SHA-256は
`a5d809911f8ffe4996db8eb147fd17253c101a5921098ffc51be87fda3f99b3f`。
final cardinalitiesは26 batches、37 tasks、two task references、604 redirects、
252 indexes。

## Reviews, Tests, Audit Impact, And Exit

prerequisite/migrationは別々にevidence-equivalence、schema/test-sufficiency、
bilingual/boundary、independent final-quality reviewsを該当範囲で実行し、全件
**NO FINDINGS**を必要とする。全9 hard gatesはscore capなし`90/100`以上でPASS
する。new fixture/expectation/sidecar/trace row/semantic testはauthorizeせず、
existing generic lintだけがnew-contract consumer。

verificationはsource/commit/blame/anchor replay、recursive contract/link/fragment/
ledger lint、checker/runner libraries/metadata、formatting、offline metadata、
warnings-denied all-target/all-feature Clippy、full workspace tests、five CLI、
protected count/hash、ledger order/hash/cardinality、`git diff --check`、exact cached
review、unstaged/untracked inspectionを含む。push/fetch/reset/stash mutationは禁止。

prerequisiteはexact 11-path scope、unchanged selected sections/ledger、synchronized
EN/JA、all gates、one commit、clean replayでexitする。migrationはexact four
redirects/five paths、complete evidence preservation、generic schema replay、all
gates、one commit、clean replay後にnext checker duplication familyをfresh select
してexitする。

## Documentation-Prerequisite Evidence

independent evidence-equivalence、schema/test-sufficiency、bilingual/boundary
reviewsは全件**NO FINDINGS**。review中、最初に`\t`をliteral 2 bytesとして扱い、
次にfixed schema rowをdata-row sortへ含めたscratch計算を修正した。ledgerのcomment/
schemaをlines 1-2へ固定したliteral-tab replayがfrozen 18-row/923-line physical
ledger hashesを独立に再現する。historical ownersはB3M2A prior debug grammarの維持を
明記する。reviewersはselected preimages 4件、anchors、blame/history、source TSV
hashes、exact 11-path scope、Task Index rows、owner links、classifications、
deferrals、protected no-opも再現し、authority/semantic expansionはない。

generic lintは`15/15`、checker `530/530`、runner `600/600`、metadata `137/137`
testsはPASS。`cargo fmt --all --check`、offline Cargo metadata、warnings-denied
all-target/all-feature Clippy、full offline workspace suiteもPASS。five CLIsはexisting
23 warnings/errors zeroでexit zeroとなりfrozen plan/parse/declaration/type/proof
hashesを再現した。

protected path count/NUL-delimited path hashはspecification `64`、`.miz` `343`、
expectation `435`、checker production `30`、runner production `90`、Cargo `21`を
exactに再現し、zero protected diffが全frozen content hashを保持する。trace、
coverage audit、unchanged 904-line ledger、selected preimages 4件、immutable source
TSVはfrozen hashesを再現した。task contractsは`67/67`、`git diff --check`はPASS。

repository inventoryはselection HEAD/clean-base `main`上のtask-only 11-path
worktree、`origin/main...HEAD=0/6`、protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`を保持する。push/fetch/reset/stash
mutationは行っていない。independent final read-only qualityは**NO FINDINGS**、全9
hard gates PASS、score capなし、valid **100/100**
（`20/20/15/15/10/10/5/5`）。exact staging、commit、clean post-commit replayが
remaining。

## Migration Evidence

documentation prerequisiteはseparate
`e604125f8b9be8052ebc686fa294bcb926448906`としてcommit済み。migration前のclean
fresh replayはfrozen preimages 4件、source TSV hashes、unchanged 904-line ledger、
protected no-op、`67/67` contracts、`origin/main...HEAD=0/7`、protected stashを再現
した。

selected Task-258B3M1/Task-258B3M2A implementation-ledger sectionsはhistorical
completion evidenceへのlanguage-local redirects 4件となった。exact source diffは
`+4/-51`。forbidden implementation headings/bodies 4件は消えた。EN/JA両方の
documentation ledgers、全recorded neighboring anchors、全unselected TODO sectionは
残る。

ledgerはexact 19 byte-sorted rows、すなわちone batch、two canonical tasks、
twelve indexes、four redirectsを追加する。923 lines、physical SHA-256
`a5d809911f8ffe4996db8eb147fd17253c101a5921098ffc51be87fda3f99b3f`、canonical
18-row SHA-256
`103e804ae1fe2e561b4c5047048cba5f0c659c43b625776c60a3f9828b3512cb`を再現し、
26 batches、37 tasks、two task references、604 redirects、252 indexesを測定する。
historical contracts、source TSV、four plans、protected surfaces、trace、coverage
auditは不変。generic lintは`15/15`、`git diff --check`はPASS。

independent evidence-equivalence、schema/test-sufficiency、bilingual/boundary
migration reviewsは全件**NO FINDINGS**。reviewersは全frozen preimage/retained fact、
exact five-path scope、language-local redirects/fragments、neighboring anchors、
source TSV/ledger hashes、schema rows/counts/order、protected no-op、EN/JA parityを
独立に再現した。

generic lintは`15/15`、checker `530/530`、runner `600/600`、metadata `137/137`
testsはPASS。`cargo fmt --all --check`、offline Cargo metadata、warnings-denied
all-target/all-feature Clippy、full offline workspace suiteもPASS。five CLIsはexisting
23 warnings/errors zeroでexit zeroとなり、frozen plan/parse/declaration/type/proof
hashesを再現した。

protected path count/NUL-delimited path hashはspecification `64`、`.miz` `343`、
expectation `435`、checker production `30`、runner production `90`、Cargo `21`を
exactに再現し、zero protected diffが全frozen content hashを保持する。trace/
coverage-audit hashesも再現した。immutable source TSV、historical contracts 4件、
plan indexes 4件は不変で、ledger/cardinalitiesは上記値を再現する。`git diff
--check`はPASS。最初のmetadata test invocationは存在しないtarget name
`metadata_consistency`を使用したが、repository target discoveryで`metadata`を
確認し、required `137/137` testsがPASSした。

independent final read-only qualityは**NO FINDINGS**。全9 hard gates PASS、score cap
なし、valid scoreは**100/100**（`20/20/15/15/10/10/5/5`）。exact five-path
staging、commit、clean post-commit replayがremaining。
