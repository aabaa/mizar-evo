# Task DOC-269CTGP-COMPACT: proof-local lower completion compaction

> canonical English:
> [../en/DOC-269CTGP-COMPACT.md](../en/DOC-269CTGP-COMPACT.md)。

本derived documentation-maintenance contractはcompleted checker-first sequenceを
exact whole-section migration前に凍結する。language behavior、test intent、API、
diagnostic、traceability、coverageを変更しない。

## Identity and status

| Field | Frozen value |
|---|---|
| Task | `DOC-269CTGP-COMPACT` |
| Status | 完了。migrationはschema-2 ledgerに登録済みであり、task-local completion evidenceがcommitted migrationとclean replayを保存する。 |
| Purpose | 269CT/269GP completion evidenceを集約し、全prerequisite、durable owner、later authority、semantic deferralを保持する。 |
| Owners | migration policy、historical [269CT](./269CT.md#completion-evidence)/[269GP](./269GP.md#completion-evidence)、[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Consumers | EN/JA checker/runner design 8 paths、Task Index 4件、post-migration generic schema-v1 ledger/lint |
| Historical commits | 269CT prerequisite/implementation `b1c91b1b`/`c6036197`、269GP `97a75fd9`/`adea7f0e` |
| Readiness | clean selection HEAD `5a83db6f82aa789e31b00601e66d57fe4cda2601`、`origin/main...HEAD=0/2`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。全preimage replay済み、blocking authority gapなし。 |

post-269CT inventoryが269GPを次に選択したのはcoherent historical sequenceであり、
semantic dependencyではない。GPはrunner-private syntax-onlyでCT checker/type
compositeをconsumeしない。completion-time scope blockerはhistoricalで、later
269GSが解消済みだがcompletion recordは変更しない。

## Authority and classification

authorityはuser-approved checker-first compaction、[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、retained
planが引用するcanonical chapter、completed reviewed recordである。

| Class | Decision |
|---|---|
| `design_drift` | completion H3 12節が8 pathsにmeasurement/exclusion/reviewを重複。paired historical contractをownerにする。 |
| `spec_gap` | structural migrationにはなし。historical GP scope conflictは269GSが後に解消し、復活・再解釈しない。 |
| `test_gap` | なし。generic schema-v1 lintがexact shapeを覆う。 |
| `source_drift` / `source_undocumented_behavior` | なし。sourceはprotectedかつnon-normative。 |
| `test_expectation_drift` | なし。spec/test/trace/expectation artifactはprotected。 |
| `boundary_violation` | 全H2、prerequisite H3 8節、later 269GS、全unselected ownerを保持して回避。historical 269CT review classificationはtask contractで保持する。 |
| `repo_metadata_conflict` | historical remote-ref movementはreport-only human-owned。current two-commit distanceを実測し、repair/fetch/reset/pushしない。 |

## Frozen preimage and scope

[`DOC-269CTGP-COMPACT.sources.tsv`](../DOC-269CTGP-COMPACT.sources.tsv)は12
byte-sorted data rows、2 comments、final LFを持つ。data-row SHA-256は
`3d3423a76a5dbdef0208733ce8a24332d9b39ee46ec15dde11fc89855d526c90`、
complete 14-line TSVは
`6d32ed76afb190c3669b48359ded7a7d2fdd54018b01e729d37a195b4dd8b0f9`。

selectionは12 unique `(path, task)` H3、8 paths / 4 paired relative files、
299 physical lines、EN/JA `6/6`、checker/runner `8/4`、269CT/269GP
`6/184`、`6/115`。nested heading/table/fenceとledger collisionはない。

prerequisiteはexact 11 paths: 本EN/JA pair、historical EN/JA 2組、TSV、plan 4件。
各planへ269CT/269GP/batchのTask Index、計12 rowsを追加する。selected preimage、
ledger、production、Cargo、spec、test、fixture、sidecar、expectation、trace、
metadata、coverage audit、count/hash/status、behaviorは変更しない。

専用prerequisite commitとfresh replay後、migrationは12 complete H3だけを
language-local redirectへ置換する。変更は8 sources、本EN/JA pair、ledgerのexact
11 paths。ledger impactはbatch 1、task 2、8 distinct source paths上のredirect
12、index 12、expanded-inventory hash 1件。TSV/historical contractはimmutable。
CT prerequisite H3 6節、GP prerequisite H3 2節、全H2/unlisted section、269GS、
later ownerを保持する。

spec coverage/design mapping/trace state/coverage credit/current semantic ownershipが
不変のため`doc/design/spec_coverage_audit.md`は変更しない。Given scope、binding/
type、condition/fact、proof/discharge/acceptance、goal/obligation、IR/VC/ATP、
diagnostic、dispatch、active coverageを追加しない。

## Reviews, verification, and exit

prerequisite reviewはpreimage、historical facts、sequence wording、GP/269GS history、
retained owner、index、EN/JA equivalence、linkを独立再現する。verificationはreplay/
hash、recursive contract/link/fragment/legacy-ledger lint、full lint policy、checker/
runner library、metadata、format、Cargo metadata、warnings-denied Clippy、workspace
tests、全5 CLI、protected hash、`git diff --check`、exact 11-path staging、全9 gates、
uncapped score `>=90/100`を含む。

prerequisite commit後はfresh inventoryで同batchを再選択。migrationには別のtest、
equivalence/boundary、source/docs/EN-JA、final-quality review、exact 11-path staging、
separate commitを課す。push/stash mutationは認可しない。

### Documentation-prerequisite evidence

independent contract、test-sufficiency、equivalence/bilingual reviewはすべて
**NO FINDINGS**。12 sections / 8 paths / 299 linesのpreimage、TSV両hash、exact
11-path scope、Task Index 12 rows、historical commit facts、CT-to-GP chronological
wording、historical GP blockerとlater 269GS resolution、retained owner、exclusion、
EN/JA equivalenceを再現した。

focused recursive contract/link/fragment lintは`1/1`、checker/runner full lint
policyは各`15/15`でpass。checker library test `530/530`、runner library test
`600/600`、runner metadata test `137/137`がpassした。`cargo fmt --all --check`、
Cargo metadata、warnings-denied all-target/all-feature Clippy、full workspace test、
protected-surface check、trace hash replay、`git diff --check`もpass。全5 CLIは
unchanged warnings 23件とともにsuccessし、stdout hashは次のとおり。

| CLI | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

protected trace manifestは
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`のまま。
final finding-specific re-reviewは**NO FINDINGS**。hard gate 9件はscore capなしで
全PASS、valid `100/100`。

## Migration evidence

documentation prerequisiteはcommit
`4177b69866d3d18c230938cbd50ed87801f7e990`。post-commit inventoryはclean、
`origin/main...HEAD=0/3`、protected stash不変だった。edit前のfresh parent replayと
independent deterministic replayはfrozen preimage 12件、section hash、heading、physical
line count、language/component partition、neighbor anchorをすべて再現した。

mechanical migrationが変更するのはdeclared source 8 paths、本EN/JA pair、
`legacy_compactions.tsv`のexact 11 paths。complete H3 12節だけをlanguage-local
redirect 12件へ置換する。299 physical linesは24 redirect-plus-separator linesとなり、
275 lines削減。全H2、prerequisite H3 8節、全unlisted section、later 269GS、全durable
ownerを保持する。

ledgerは535 physical linesで、batch 1、task records 2、8 distinct source paths上の
redirect 12、index records 12をexact追加する。expanded-inventory SHA-256は
`370df3c1cc663091ce777024d735534d25d562262e1f48515d2c3e13e1f87efe`、
complete physical SHA-256は
`14e45e9fbd12c2d68275f6d57a24e32b758327ae13ac71e374d8ceb992684bcd`。
immutable source TSVは
`6d32ed76afb190c3669b48359ded7a7d2fdd54018b01e729d37a195b4dd8b0f9`のまま。

spec、`.miz`、fixture、sidecar、expectation、trace TOML/status/backlink、coverage
credit、production source、Cargo、public API、diagnostic、root coverage audit、frozen
source inventory、historical contract、Task Index 4件は不変。protected trace hashは
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`。
focused generic-ledger lintと`git diff --check`はPASS。

independent test-sufficiency、equivalence/boundary、source/document/EN-JA
consistency reviewはすべて**NO FINDINGS**。preimage 12件、live fact owner、retained
prerequisite `6+2`節、CT-to-GP chronology、GP/269GS history、exact redirect/anchor/
index、schema arithmetic、hash、protected scope、audit no-impact decisionを独立再現した。
generic schema-v1 consumerが本exact whole-section shapeを既にcoverするため、新規Rust
testは不要。

focused/full runner lint policy `1/1` / `15/15`、checker/runner library
`530/530` / `600/600`、runner metadata `137/137`、checker lint `15/15`がPASS。
`cargo fmt --all --check`、Cargo metadata、warnings-denied all-target/all-feature
Clippy、full workspace test、上記prerequisite stdout hashとunchanged warnings 23件を持つ
全5 CLI、protected count/hash replay、`git diff --check`もPASS。

required next-task handoff追加後のfinal finding-specific re-reviewは**NO FINDINGS**。
hard gate 9件はscore capなしで全PASS、valid `100/100`。migration scope内のresidual
riskはなし。

## Handoff

exact task-only staging、separate migration commit、clean post-commit HEAD/origin/
stash check後、completed checker-owned documentationをfresh read-only inventoryする。
preimage、live owner、exclusion、count/hash impact、paired prerequisite contractを
freezeした後でのみ、coherentかつdependency-readyなduplication familyをexact 1件
selectする。本contractはlater compaction batchもsemantic taskもpreauthorizeしない。

authority interpretation、ownership decision、integration、final quality scoring、stage、
commitは`xhigh` parentが担当する。bounded independent reviewには`high`、semantic、
test intent、public API、acceptanceを決定しないdeterministic inventoryだけに`medium`を
使う。authority ambiguityまたはmixed owner-local sectionはedit前にparentへescalateする。
