# Task DOC-258AB-COMPACT: source-statement completion compaction

> canonical English:
> [../en/DOC-258AB-COMPACT.md](../en/DOC-258AB-COMPACT.md)。

本derived documentation-maintenance contractは、完成済みchecker-first taskの
coherent familyを削除前に凍結する。language behavior、test intent、API、
diagnostic、traceability、coverage creditを導入・再解釈しない。

## Identity and status

| Field | Frozen value |
|---|---|
| Task | `DOC-258AB-COMPACT` |
| Status | Documentation prerequisite commit済み。exact migration、independent review、required verification、全9 hard gatesをuncapped 100/100で完了。exact staging、separate migration commitが未完。 |
| Purpose | Tasks 258A/258B1/258B2のcompletion-only evidenceを集約し、全frozen contract、owner-local invariant、semantic deferral、verification ownerを保持する。 |
| Owners | repository migration policy、historical [258A](./258A.md#completion-evidence)、[258B1](./258B1.md#completion-evidence)、[258B2](./258B2.md#completion-evidence) contracts、[checker](../../mizar-checker/ja/00.crate_plan.md#task-index)/[runner](../../mizar-test/ja/00.crate_plan.md#task-index) Task Index |
| Consumers | checker/runner EN/JA design 18 paths、Task Index 4個、post-migration generic schema-v1 ledger/lint |
| Historical commits | 258A prerequisite/implementation `e0b4bb59`/`1e81db7a`、258B1 `ddcac673`/`e87b4a48`、258B2 `3dd38526`/`4d9ed4f5` |
| Documentation prerequisite | `d767941aad8f0339af76500c3801823675f2b139` |
| Readiness | clean post-prerequisite HEAD `d767941a`、`origin/main...HEAD=0/1`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。編集前に38 preimageをreplayし、blocking authority gapなし。 |

task orderはhistorical selection order、すなわち258A、pair-only 258B1 slice、
base-only 258B2 siblingであり、semantic dependency chainではない。B2はshared
258A/base ownerへ依存し、B1 reference edgeを持たない。

## Authority and classification

authorityはuserのchecker-first consolidation決定、[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、保持する
task plan記載のcanonical spec chapter、completed derived recordである。source
behaviorはnormativeではない。

| Class | Decision |
|---|---|
| `design_drift` | 38個のcompletion-only H3が18 pathsにimplementation status、measurement、exclusion、review evidenceを重複させている。paired historical contractをsingle ownerにする。 |
| `test_gap` | なし。schema v1はexact whole-section redirectを表現でき、generic lint consumerがこの形を覆う。 |
| `spec_gap` | structural migrationについてなし。language-semantic issueを選択しない。 |
| `source_drift` | なし。Rust/Cargoはprotected。 |
| `source_undocumented_behavior` | 導入も推測もしない。 |
| `test_expectation_drift` | なし。spec、`.miz`、fixture、sidecar、expectation、trace TOML、metadataはprotected。 |
| `boundary_violation` | 全H2、全unlisted H3、特にTask-258B2 frozen H3 owner EN/JA 12節を保持して回避する。 |
| `repo_metadata_conflict` | report-only。VS Code再起動後、`origin/main`が以前実測した3-commit distanceから、許可されたpushなしにclean HEAD `a1bf34e8`と一致した。repair/fetch/reset/pushは行わず、exact task filesは安全に特定できる。 |

## Frozen preimage inventory

[`DOC-258AB-COMPACT.sources.tsv`](../DOC-258AB-COMPACT.sources.tsv) はexactly
38 byte-sorted data rows、2 comments、final LFを持つ。各rowはtask、language、
component、exact path、ATX level、exact heading、complete-section SHA-256、
physical linesを記録する。sectionはH3から次のvisible H3-or-higher ATX heading
直前までである。

data-row SHA-256は
`f51fbef7c54f41065409b53eb8a5485b0d6ff6f67c42eb63cd4755909ad5c87d`、
40-line TSV全体は
`65fda187d1f5e0e5202269918c78cf3a74f7eda451d0d70fab9c7d9f3a2db119`。
選択は38 unique `(path, task)` sections、18 physical paths / 9 paired relative
files、502 physical lines、EN/JA `19/19`、checker/runner `28/10`。task別には
258A `10/155`、258B1 `16/229`、258B2 `12/118` sections/linesである。selected
sectionにnested ATX heading、table、fenceはなく、ledgerが既にforbidするheadingもない。

| Component | Relative files |
|---|---|
| mizar-checker | `00.crate_plan.md`、`binding_env.md`、`payload_family_decomposition.md`、`resolved_typed_ast.md`、`source_spec_audit.md`、`source_statement.md`、`typed_ast.md` |
| mizar-test | `00.crate_plan.md`、`harness.md` |

Task-258B2 checker-plan/source-statement implementation resultはH2のため保持する。
binding/payload/Typed/Resolved/runner plan/harnessのB2 frozen H3 EN/JA 6組も
保持する。全other H2、frozen section、unselected H3、adjacent owner-local fact、
later taskはTSV範囲外である。

## Documentation-prerequisite scope

prerequisiteはexactly 13 paths、すなわち本EN/JA batch pair、historical contract
3組、language-neutral TSV、checker/test EN/JA plan 4件を変更する。各planへTask
Index 4 rows、計16 recordsを追加する。

selected preimage、legacy ledger、production、Cargo、spec、`.miz`、fixture、
sidecar、expectation、trace TOML、metadata、root coverage audit、executable
count/hash/status、behaviorは変更しない。spec coverage/design mapping/trace
status/coverage credit/semantic ownershipが不変のため
`doc/design/spec_coverage_audit.md`は変更しない。

## Frozen migration and ownership boundary

専用prerequisite commitとfresh replay後、implementationが置換できるのは
inventory記載38 complete H3だけで、各historical contractの
`#completion-evidence`へのlanguage-local redirectにする。変更は18 sources、
本EN/JA batch pair、`legacy_compactions.tsv`のexact 21 paths。ledger impactは
batch 1、task 3、18 distinct source pathsにまたがるredirect 38、index 16
recordsと、新規計算するexpanded-inventory hash。
migration中TSV/historical contractsはimmutable。

historical contractはcompletion measurement/review evidenceを所有し、component
docはmodule API、ownership、validation、invariant、runner boundary、frozen planを
保持する。migrationはassumptionをaccepted factにせず、goal/guard composition、
proof/discharge/acceptance、theorem publication、diagnostic、Core/CFG/VC/ATP、
active dispatch、coverage creditを追加しない。Tasks 258B3/B4/B5と269–272が
各semantic/follow-up ownershipを保持する。

## Documentation-prerequisite evidence

pre-edit specification reviewはmedium design-drift riskを2件検出した。
chronological orderがsemantic dependency chainと誤読され得る点と、B2 frozen H3
owner 6組へのexplicit link不足である。draft contractで両方を解消した。contract
reviewはsource recordの存在を誤って示唆するmedium schema-v1 wording mismatchを
1件検出した。18 distinct source pathsにまたがる38 redirectsへ修正後の
finding-specific re-reviewは**NO FINDINGS**。独立test-sufficiency reviewと
equivalence/EN-JA/ownership reviewも**NO FINDINGS**。

parent replayは38 preimages、502 physical lines、両TSV hash、task partition
`10/155`、`16/229`、`12/118`、EN/JA `19/19`、checker/runner `28/10`、
18 distinct paths、13-path prerequisite scope、16 index recordsを一致確認した。
recursive task-contract/link/fragment lintとrunner lint-policy 15件がPASS。
checker/runner libraryは`530/530`、`600/600`、runner metadataは`137/137`、
checker lintは`15/15`。`cargo fmt --all --check`、Cargo metadata、warnings-denied
all-target/all-feature Clippy、full workspace test suite、`git diff --check`もPASS。
protected trace SHA-256は
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`のまま。

5 CLI stdout hashは不変:

| Route | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

変更13 pathsはすべてdocumentation-only。production、Cargo、spec、test
artifact、trace status/count/backlink、coverage audit、legacy ledger、executable
behaviorは不変。既存plan warningはbaseline warningでstdout hashを変えない。

final independent read-only quality reviewは**NO FINDINGS**、全9 hard gates
PASS、score capなし**100/100**。残る観測はreport-only remote-ref metadata
conflictだけで、exact stagingを妨げない。

## Implementation evidence

fresh post-prerequisite inventoryは編集前に38 frozen preimages、heading、hash、
physical-line count、language/component partition、neighbor anchorを全てreplayした。
migrationはその38 complete H3だけを対応するhistorical contractへの38
language-local redirectsへ置換する。selected intervalはseparator blank 38行を含む
502 physical linesだった。replacementはblankを保持し、completion-content 464行を
削除、redirect 38行を追加し、mapped intervalを426行削減した。全H2、全unlisted
H3、B2 frozen H3 owner 6組を保持する。

ledgerは508 physical linesで、exactly batch 1、task 3、18 distinct source pathsに
またがるredirect 38、index 16 recordsを追加する。expanded-inventory SHA-256は
`c472137844a8f41c6e3ad7ab96b8a8de559df962979b148c2dc706b1de6acbd8`、
complete physical SHA-256は
`4d6dd6103ee721e72b2c008247eeb84fcd30a7023e38cedbe8b73571ed621dd0`。
immutable 40-line source TSVは
`65fda187d1f5e0e5202269918c78cf3a74f7eda451d0d70fab9c7d9f3a2db119`のまま。

spec、`.miz`、fixture、sidecar、expectation、trace TOML/status/backlink、coverage
credit、source、Cargo、public API、diagnostic、root coverage audit、historical
contract、prerequisite Task Index 4件は不変。protected trace hashは
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`のまま。

独立test-sufficiency、equivalence/boundary、source/document/EN-JA consistency
reviewはすべて**NO FINDINGS**。focused/full runner lint policyは`1/1`、`15/15`、
checker/runner libraryは`530/530`、`600/600`、runner metadataは`137/137`、
checker lintは`15/15`。`cargo fmt --all --check`、Cargo metadata、warnings-denied
all-target/all-feature Clippy、full workspace test suite、上記prerequisite hashと
一致する全5 CLI、protected count/hash replay、`git diff --check`がPASS。

final independent read-only quality reviewは**NO FINDINGS**、全9 hard gates
PASS、score capなし**100/100**、migration scope内residual riskなし。
historical remote-ref movementはreport-only human-owned observationのまま。

## Tests, reviews, and exit

prerequisite reviewはpreimage/hash/count、fact ownership、sequencing/dependency
wording、全retained H2/H3 exclusion、EN/JA equivalence、index、linkを独立再現する。
test-sufficiency/source-document consistency reviewは**NO FINDINGS**必須。
verificationはpreimage replay、recursive task-contract/link/fragment lint、full
lint policy、checker/runner libraries、metadata、checker lint、format、Cargo
metadata、warnings-denied all-target/all-feature Clippy、full workspace tests、
全5 CLI/protected hash、`git diff --check`、exact 13-path staging、全9 hard gates、
score capなし`>=90/100`を含む。

prerequisite commit後はfresh inventoryで同batchを再選択する。migrationは別の
test-sufficiency、equivalence/boundary、source/document/EN-JA、final-quality review、
exact 21-path staging、別commitを受ける。pushおよびprotected stash変更は行わない。
