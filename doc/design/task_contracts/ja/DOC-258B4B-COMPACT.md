# Task DOC-258B4B-COMPACT: B4B implementation-evidence compaction

> canonical English:
> [../en/DOC-258B4B-COMPACT.md](../en/DOC-258B4B-COMPACT.md)。

## identity and status

| field | frozen value |
|---|---|
| task | `DOC-258B4B-COMPACT` |
| status | 完了。migrationはschema-2 ledgerに登録済みであり、task-local completion evidenceがcommitted migrationとclean replayを保存する。 |
| purpose | frozen contractとdurable checker/runner ownerを保持し、Task-258B4Bのtask-wide implementation-completion section 4件を集約する。 |
| historical owner | [Task 258B4B](./258B4B.md#completion-evidence) |
| plan indexes | [checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)と[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| selection HEAD | `fee14f18c2301b1523250f25843d96b91f759b8e` |
| repository state | clean `main`、`origin/main...HEAD=0/7`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` |

## authority and classification

authorityはuser-approved checker-first compaction program、`AGENTS.md`、
autonomous migration policy、reviewed Git history、selected completed
sections 4件、surviving durable ownersである。source behaviorはnormative
ではない。

| class | decision |
|---|---|
| `design_drift` | checker plan/TODOがhistorical B4B implementation/review/count/hash/commit evidenceを単一historical ownerなしに反復する。 |
| `test_gap` | なし。path-scoped generic legacy-heading validationとregressionは`fa7c3acf89e2d66c1f9f21fd515da650f6226304`でcommit済みである。 |
| `boundary_violation` | source-statement、audit、runner sectionsを選ぶとdurable owner-local evidenceが混在する。checker plan/TODO 4 sectionsだけをmigrateする。 |
| `spec_gap` / `source_drift` | 導入も修復もしない。historical stateはdurable ownerに残る。 |
| `source_undocumented_behavior` / `test_expectation_drift` | 推測も変更もしない。 |
| `repo_metadata_conflict` | branchはobserved `origin/main`より7 commits aheadである。report onlyで修復しない。 |

## frozen sources and anchors

[`DOC-258B4B-COMPACT.sources.tsv`](../DOC-258B4B-COMPACT.sources.tsv)は
byte-sorted data rows 4件、comments 2件、final LFを持つ。data-row
SHA-256は`78395a61a864bbe0fb361151bb998bbba25d81d89dc0ca5307d9fe1166687485`、
complete-file SHA-256は
`ada3f07eaf309a3e91c210599481738a6074c936a686356db7bfe4ae6424e546`
である。sectionsはflat、source-local unique、unlinkedで、registered
redirectを含まず、合計207 physical linesである。EN/JA planは`76/70`、
EN/JA TODOは`32/29`である。

| source | previous H2 | next H2 |
|---|---|---|
| EN plan | `## Task 258B4B Frozen Connective/Grouping Composite Theorem Root` | `## Task 258B4C Frozen Restricted/Existential/Nested Theorem Root` |
| JA plan | `## Task 258B4B frozen connective/grouping composite theorem root` | `## Task 258B4C Frozen Restricted/Existential/Nested Theorem Root` |
| EN TODO | `## Checker Task 258B4B Documentation Prerequisite` | `## Checker Task 258B4C Documentation Prerequisite` |
| JA TODO | `## Checker Task 258B4B documentation prerequisite` | `## Checker Task 258B4C documentation prerequisite` |

implementation commit `752c17ae7d552d5268d1028612b8174e480b6f3e`が
completion bodyを導入した。successor prerequisite
`3c723316ae632a867d29e8f4fc36348be30df202`はimmutable post-commit/B4C
handoff tailsとfollowing B4C headingsだけを追加した。current `git blame`
ではselected linesはこの2 commitsだけに帰属する。

## retained owners and exclusions

B4B frozen plan sectionとcheckerのsource-statement、formula-composition、
Typed/Resolved AST、payload-family、source/specification、boundary、
bilingual ownersは全てunchangedである。runner plan/TODO、harness、
boundary、bilingual sectionsも、task-wide factを反復する箇所を含めて
durable runner evidenceとしてunchangedにする。registered B4A redirects
4件とrequired neighboring anchorsもselected sectionsの外に保つ。

B4C onward、specification、`.miz`、expectation、sidecar、traceability、
coverage audit、production、Cargo、API、active behaviorは禁止する。
separate `DOC-COMPACT-PATH-SCOPE` contractのpre-existing status wordingは
このtaskでreopenしない。

## protected baseline

prerequisite/migrationのexpected deltaはzeroである。

| surface | paths | path SHA-256 | content SHA-256 |
|---|---:|---|---|
| specification | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

traceは`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`、
coverage auditは
`2aa808aa033c04fa46ea76f65cfadcf0d8f0e1e53fba6d70edab223a7481685f`
のままである。850-line ledger baselineは
`7bd738ad591a40667cb95421dd68d386213c25c51274cbf5c79d8f24b0b1688a`。
current CLI stdout hashesはplan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`
である。

## prerequisite and expected migration

prerequisiteはexact 9 pathsを変更する。このpair、historical Task-258B4B
pair、source TSV、plan Task Index 4件である。各planはTask `258B4B`と
this batchを追加し、index recordsは合計8件である。selected sourcesと
ledgerはunchangedである。

prerequisite commit後のfresh replayを経て、migrationはexact 7 pathsを
変更する。source documents 4件、このpair、`legacy_compactions.tsv`で
ある。4 sectionsを`258B4B.md#completion-evidence`へのlanguage-local
redirect 4件へ置換する。exact source diffは`+4/-203`で、207 selected
linesをredirect-plus-separator 8 linesへ縮小する。

ledger impactは14 lines、`850 -> 864`である。batch 1、task 1、four
source pathsのredirect 4、index record 8である。canonical 13-row payload
SHA-256は`13f7b68977d3d669173e987662276d18bec940cbd484089a561c2fec390cb55a`、
expected physical ledger SHA-256は
`876dbd36c52952d029257d3c16d0ae8f7cb2a9fac9f09fedafae6c4df3e026bf`
である。mapping、ownership、status、deferred reason、creditを変更しない
ため`spec_coverage_audit.md`へのimpactはない。

## reviews, verification, and exit

prerequisite/migrationはそれぞれapplicableなequivalence、schema/test-
sufficiency、bilingual/boundary、final-quality reviewを要求し、全て
**NO FINDINGS**で終了しなければならない。全9 hard gates PASS、score
capなし、valid score `90/100`以上が必要である。

verificationはpreimage/history/anchor replay、generic recursive task-
contract/link/fragment/ledger lint、checker/runner lint/library、runner
metadata、formatting、offline Cargo metadata、warnings-denied all-target/
all-feature Clippy、full workspace tests、5 CLIs、protected count/hash、
ledger order/hash/cardinality、`git diff --check`、exact cached review、
unstaged/untracked inspectionを含む。push、fetch、reset、stash mutationは
行わない。

prerequisiteはexact nine-path scope、unchanged sources/ledger/protected
owners、synchronized EN/JA、complete reviews/verification、1 commit、clean
replayでexitする。migrationはexact four redirects/seven paths、ledger
replay、全gates、1 commit、clean replayでseparately exitする。

## documentation-prerequisite evidence

independent evidence-equivalence、schema/test-sufficiency、bilingual/
boundary reviewsは全て**NO FINDINGS**で終了した。4 preimages
`76/70/32/29`、207-line total、source TSV hashes、index records 8件、
prospective 13-row canonical hash、864-line physical ledger hash、
`+4/-203` migration deltaをindependently reproduceした。retained owner
linksとlanguage-local fragmentsは全てresolveする。selected source
sectionsと850-line ledgerはunchangedである。

checker/runner lintは各`15/15`、checker/runner librariesは`530/530`と
`600/600`、runner metadataは`137/137`でPASSした。formatting、offline
Cargo metadata、warnings-denied all-target/all-feature Clippy、3 long
frontend benchmarksを含むfull all-target/all-feature workspace suite、
generic recursive contract/link/fragment/ledger lint、`git diff --check`は
PASSした。

5 CLIsは全てexit zero、known warnings 23、errors zeroである。current
plan/requirementsは`428/395`、pass/failは`235/193`、active
parse/declaration/type/proofは`101/7/205/1`で、全stdoutはfrozen hashを
reproduceする。protected counts/path hashesはspecification 64、`.miz`
343、expectation 435、checker production 30、runner production 90、Cargo
21をreproduceする。zero protected diffが全frozen content hashを保持する。
trace、coverage audit、immutable source TSV、unchanged 850-line ledgerは
frozen hashesをreproduceする。final independent read-only quality reviewは
**NO FINDINGS**、全9 hard gates PASS、score capなし、**100/100**
（`20/20/15/15/10/10/5/5`）である。exact nine-path staging、cached
review、commit、clean replayがremainingである。

## migration evidence

documentation prerequisiteは
`158986616f91898d24c5c1ffc13c9446f38b2306`としてseparately commitした。
clean fresh replayは4 frozen preimages、source TSV hashes、unchanged
850-line ledger、protected surfaces、trace、coverage audit、stash
fingerprintをreproduceした。

selected sections 4件は`258B4B.md#completion-evidence`へのlanguage-local
redirect 4件になった。source deltaはexact `+4/-203`で、各
`76/70/32/29`-line sectionはredirect 1行とretained separatorになった。
全neighboring anchors、registered B4A redirects、retained ownersはin place
である。ledgerはexact 14 byte-sorted rows（batch 1、task 1、redirects 4、
indexes 8）を追加し、864 lines、physical SHA-256
`876dbd36c52952d029257d3c16d0ae8f7cb2a9fac9f09fedafae6c4df3e026bf`
とfrozen 13-row canonical hashをreproduceする。source TSV、historical
contract、Task Index rows、protected surfaces、trace、coverage auditは
unchangedである。

independent migration-equivalence、schema/test-sufficiency、bilingual/
boundary reviewsは全て**NO FINDINGS**で終了した。checker/runner lintは
各`15/15`、checker/runner librariesは`530/530`と`600/600`、runner
metadataは`137/137`でPASSした。formatting、offline Cargo metadata、
warnings-denied all-target/all-feature Clippy、3 long frontend benchmarksを
含むfull all-target/all-feature workspace suite、generic recursive
contract/link/fragment/ledger lint、`git diff --check`はPASSした。

5 CLIsは全てexit zero、known warnings 23、errors zeroで、5 frozen stdout
hashesをreproduceする。protected counts/path hashesはspecification 64、
`.miz` 343、expectation 435、checker production 30、runner production 90、
Cargo 21をreproduceし、zero protected diffが全frozen content hashを保持
する。trace、coverage audit、immutable source TSV、13-row canonical
payload、864-line ledgerはfrozen hashesをreproduceする。final independent
read-only quality reviewは**NO FINDINGS**、全9 hard gates PASS、score cap
なし、**100/100**（`20/20/15/15/10/10/5/5`）である。exact staging、
commit、clean replayがremainingである。

## handoff

migration reviews/verificationを完了し、exact seven frozen pathsだけを
stage/commitし、次のchecker duplication familyをfresh-inventoryする。
parentは`xhigh`、bounded independent reviewsは`high`を用いる。
