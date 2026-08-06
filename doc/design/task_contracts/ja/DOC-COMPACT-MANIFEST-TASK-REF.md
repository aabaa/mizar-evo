# Task DOC-COMPACT-MANIFEST-TASK-REF: cross-batch historical-task reference

> canonical English:
> [../en/DOC-COMPACT-MANIFEST-TASK-REF.md](../en/DOC-COMPACT-MANIFEST-TASK-REF.md)。
> 本文書は同一logical taskの日本語companionである。

これはlanguage behavior、test intent、diagnostic、public API、coverage、既完了
migrationを変更せずlegacy-compaction ledgerを拡張するderived documentation/
test-policy prerequisiteである。一つのcanonical historical task contractに対し、
ownershipを移動・複製せず、disjoint source-file setsを持つ複数の独立した
whole-section compaction batchを関連付ける。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-COMPACT-MANIFEST-TASK-REF` |
| Status | documentation prerequisite完了。independent policy/EN-JA reviewはno findings、全required verificationはpass。exact task-only staging/commitが残り、そのdedicated commitまではimplementation禁止。 |
| Primary owner | repository legacy-compaction schemaと`mizar-test` lint consumer |
| Consumers | 既登録historical taskの追加sectionをcompactする後続coherent batch。最初の予定consumerは`DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT` |
| Dependencies | `DOC-COMPACT-MANIFEST`、`DOC-COMPACT-PATH-SCOPE`、完了済み`DOC-258B4C-DOC-REVIEW-COMPACT` batch |
| Readiness | dependency-ready。schema version 1ではglobal task-row ownership違反なしに追加Task-258B4C batchを表現できず、完了済み先行batchの変更はそのfrozen migration boundaryに反する。 |

[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index)が本contractをindexする。
元の[manifest contract](./DOC-COMPACT-MANIFEST.md)はschema-1 implementationの
historical recordとして残り、本contractがlive schema-2 deltaを所有する。

## Authority And Classified Gap

authorityはchecker-first consolidationを求めるuserの明示指示、
[`AGENTS.md`](../../../../AGENTS.md)、および
[migration policy](../../autonomous_crate_development.md#migration-policy)である。
本derived policy taskは`doc/spec/en/`、`.miz`、expectation、trace、source behaviorを
authorityとして消費しない。

| Class | Decision |
|---|---|
| `design_drift` | schema 1は各task rowをglobalに1 batchへ束縛する一方、canonical historical taskには複数のdisjoint coherent duplication familyが残り得る。 |
| `boundary_violation` | completed batchを変更せず、historical ownershipを複製せず、ledger rowに削除authorityを持たせないことで回避する。 |
| `test_gap` | lintにtwo-batch/one-task positive vectorとundeclared/duplicate/self-owned/wrong-batch task referenceのfail-closed vectorがない。 |
| `spec_gap` | なし。本taskはderived repository policyだけを定義する。 |
| `source_drift`、`source_undocumented_behavior`、`test_expectation_drift` | なし。production/semantic artifactsは不変。 |
| `repo_metadata_conflict` | なし。選定時HEADは`1d32ed06cc110ed98e9116dd59af82e9ef724b15`、worktree clean、`origin/main...HEAD`は`0/9`、protected `stash@{0}`は`f65cf4a13752ec380710814a9ac6392ccb9d75d4`のまま。aheadはreport-only、pushはscope外。 |

## Documentation-Prerequisite Boundary

本prerequisiteは本EN/JA pairとchecker/test EN/JA crate plan各1 Task Index rowの
exact 6 filesだけを変更する。`AGENTS.md`、protocol、design index、existing task contract、
manifest、Rust、Cargo、specification、fixture、sidecar、expectation、trace、coverage
audit、redirect、source-section count/hashは変更しない。task-contract Markdown countは
57/57から58/58へ移り、864-line schema-1 ledgerとphysical SHA-256
`876dbd36c52952d029257d3c16d0ae8f7cb2a9fac9f09fedafae6c4df3e026bf`
は不変である。

## Documentation-Prerequisite Evidence

first policy reviewのoverbroad cross-batch claimとambiguous old-contract boundaryを、
disjoint source-file setsおよびschema-owner noticeへ限定した。bilingual re-reviewで
incorrect document-global one-redirect claimを削除した。final policy/EN-JA re-reviewsは
**NO FINDINGS**である。

focused/full 15-test lint-policy target、recursive local links/fragments、
`cargo fmt --all --check`、offline Cargo metadata、
`cargo clippy --all-targets --all-features -- -D warnings`、full workspace
`cargo test`はpass。five corpus CLIsはすべてexit zeroで、plan/parse/declaration/
type/proof hashは順に
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`
で不変、existing 23 warnings/zero errorsも不変である。protected specification、test、
expectation、trace、coverage、source、Cargo、manifest、stash surfaceは不変。
`git diff --check`はpassする。

## Frozen Schema-2 Delta

implementationはfirst data rowを`schema<TAB>1`から`schema<TAB>2`へ変え、exact
1 record kindを追加する。

| Kind | kind後のfields |
|---|---|
| `task_ref` | referencing batch ID、existing canonical historical task ID |

全schema-1 recordとbyte grammarは不変とする。`task` rowは引き続きtask IDとEN/JA
historical contract pathのsole global ownerである。`task_ref`はそのexisting ownerへの
batch-local relationだけでありcontract pathを持たず、`task` rowを置換・複製しない。
exact rulesは次のとおり。

1. `(batch ID, task ID)`はunique。両IDはexisting ID grammarに従う。
2. batchとcanonical `task` rowが存在し、referenced taskは別batch所有でなければならない。owner batchはexisting `task` rowを使う。
3. batchのexisting `task count` fieldは、そのbatch所有task rowsとそのbatch宣言task referencesを合わせたdistinct participating tasksを数える。
4. redirectのtaskはそのbatch所有または同batchの`task_ref`宣言済みでなければならない。resolutionはsole canonical task rowのlanguage-local contractと`#completion-evidence`を必ず使う。
5. historical taskのTask Index recordはoriginal owning batchだけが所有する。referencing batchは自身のbatch contractだけをindexし、二つ目のhistorical-task index rowを追加・claimしない。
6. `task_ref` rowはreferencing batchのcanonical expanded inventory bytes/hashに含む。existing batchはreferenceを追加しないためinventory/hashがbyte-identicalである。
7. recordはenforcement metadataに限る。referencing batch contract、historical task migration boundary、exact source inventory、equivalence reviewが各redirectを独立にauthorizeする。
8. one canonical taskについて、各referencing/owning batchのsource-file setは他batchのsetとdisjointでなければならない。existing unique `(source path, task ID)` redirect keyを変更せず、この制限をenforceする。他taskはsame file内にdistinct redirectを引き続き所有できる。

schema 2もexact whole ATX H2-H6 section replacementだけを表す。one taskのsame source
file内multiple sections、paragraph-level removal、mixed owner-local removalは引き続き
表現せず、occurrence-safe evidence identityを定義する別review済みprerequisiteを要する。

## Frozen Lint And Test Delta

sole consumerは
`task_contracts_are_recursively_paired_and_supported_links_resolve`のままで、test名と
15-test lint-policy listを変更しない。parser/relation checkerはnew strict three-field
recordをacceptし、canonical inventory bytes/participating-task countsへ含め、Rustへtask
ID、source path、batch-specific branchを埋め込まず上記ruleを適用する。

focused same-test vectorsは次を証明する。

- two batchesがone canonical taskを使用でき、second batchのone referenceからoriginal EN/JA contractへredirect resolveできる
- duplicate `(batch, task)` reference、undeclared batch/task、owner batchからのreference、wrong field count、invalid IDはfailする
- another batchのredirectはexact referenceなしでfailし、referenceありの場合だけpassする
- same taskのanother batchが既存batch所有source pathを再利用するとfailする
- referencing batchはreferenced historical taskを再indexできない
- count/expanded-inventory-hash mutationは引き続きfail closedする

current lint-policy raw list hashは
`b044e771a655e72131d0371636bbac5684ef93a3ea503984537a4bb9dd13a7cf`。
implementationはtest countを変更しない。schema lineだけを変更した864-line ledgerの
expected physical SHA-256は
`b7e9a943afcca7ee4773e6ac472e8a350624d17f96dbb54ca821fcb1f57d56cc`。
current 21 batch hashes、33 canonical task rows、592 redirects、216 index rowsは不変で、
implementationは`task_ref` rowをzero追加する。

## Implementation Scope And Prohibitions

prerequisite commit後、implementationはexact 9 pathsを変更する。本EN/JA pair、
`AGENTS.md`、`doc/design/README.md`、
`doc/design/autonomous_crate_development.md`、schema-2 supersession noticeだけを加える
EN/JA `DOC-COMPACT-MANIFEST` pair、
`doc/design/task_contracts/legacy_compactions.tsv`、
`crates/mizar-test/tests/lint_policy.rs`である。本prerequisiteの4 Task Index rowsは不変。

B4C reference/batchまたはlegacy migrationを追加せず、completed batch/source inventoryを
編集せず、production/public API/Cargo dependencyを変更せず、`doc/spec/**`、`.miz`、
fixture、sidecar、expectation、traceability、coverage status、diagnostic、CLI behavior、
protected `stash@{0}`に触れない。`doc/design/spec_coverage_audit.md`はownership/coverage
impactがなく不変とする。

completed historical migration task/batch contractsとregistered inventoriesはすべて
不変とする。先行`DOC-COMPACT-MANIFEST` pairはmigration batchではなくschema ownerであり、
そのdeltaは上記explicit live-policy supersession noticeだけである。

## Reviews, Verification, And Exit

independent specification/policy completeness、EN/JA logical equivalence、test sufficiency、
implementation correctness、source/docs consistency reviewを行い、findingを修正して
該当reviewを**NO FINDINGS**まで再実行する。final read-only reviewはautonomous nine
hard gates全PASS、score capなし、90/100以上を必要とする。

verificationはfocused 15-test lint-policy target、manifest mutation vectors、local
links/fragments、`cargo fmt --all --check`、offline metadata、
`cargo clippy --all-targets --all-features -- -D warnings`、full workspace
`cargo test`、five repository CLI routes、protected path/count/content hashes、
`git diff --check`、exact staged-content review、clean post-commit HEAD/origin/stash
inventoryを含む。

exitはgeneric schema-2 implementationだけがdedicated commitとなり、frozen count/hashが
再現し、B4C-specific dataがなく、全review/gateがpassすること。fresh inventory後、
`DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT`のexact checker TODO pairへ戻る。EN
5934-5958、25 lines、SHA-256
`b3232c301dc8df4b6da3cccb4d040c9a819b8931ed31d20e311ca574f86ba82e`、JA
5670-5693、24 lines、SHA-256
`200dcfb5ecd4e44ea25254d70c049338a211009d28c89cc05c147541e727417f`。
checker plan、owner/audit documents、lower-stage ledgers、全runner documentsはその
later two-section familyから除外する。
