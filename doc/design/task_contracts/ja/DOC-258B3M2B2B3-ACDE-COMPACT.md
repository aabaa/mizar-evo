# Task DOC-258B3M2B2B3-ACDE-COMPACT: set-witness completion compaction

> canonical English:
> [../en/DOC-258B3M2B2B3-ACDE-COMPACT.md](../en/DOC-258B3M2B2B3-ACDE-COMPACT.md)。

本documentation-maintenance contractは、completed Task-255 set-witness logの
checker-only familyをexact whole-section migration前にfreezeする。language
behavior、test intent、API、diagnostic、traceability、coverage creditを導入・
再解釈しない。

## Identity and status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B3-ACDE-COMPACT` |
| Status | Documentation-prerequisite reviews/verification/final qualityはcomplete。exact staging/prerequisite commitはpendingで、migrationはseparate later commit。 |
| Purpose | Tasks 258B3M2B2B3A/C/D/Eのcompletion-only payload-family evidenceを集約し、全frozen/durable ownerとasymmetric B3B recordを保持する。 |
| Owners | migration policy、historical [A](./258B3M2B2B3A.md#completion-evidence)/[C](./258B3M2B2B3C.md#completion-evidence)/[D](./258B3M2B2B3D.md#completion-evidence)/[E](./258B3M2B2B3E.md#completion-evidence) contracts、[checker](../../mizar-checker/ja/00.crate_plan.md#task-index)/[runner](../../mizar-test/ja/00.crate_plan.md#task-index) indexes |
| Consumers | paired checker payload-family documents、4 Task Indexes、post-migration generic schema-v1 ledger/lint |
| Historical sequence | A `f4ff4596` -> `a147bad8`; C `ea48ffc4` -> `7988a509`; D `43af562c` -> `08a7d1e3`; E `8075000b` -> `e4479691` |
| Documentation prerequisite | Pending |
| Readiness | clean selection HEAD `95b4ce9801bc0b5ec85dbdba30d40ec26d44d3d7`、`origin/main...HEAD=0/6`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。exact selectionはdependency-ready。 |

4 tasksは同じTask-255 set-term witness familyのadjacent completed siblingである。
groupingはtask identity/factを別々に保持し、enumeration/choice/`qua`/
comprehension間のnew semantic dependencyを主張しない。

## Authority and classification

authorityはuserのchecker-first consolidation方針、[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、各retained
task ownerがlinkするcanonical/test authority、completed reviewed historyである。
source behaviorは本taskのnormative authorityではない。

| Task | Exact canonical/executable authority | Existing test owner |
|---|---|---|
| A | Chapters 13 §§13.4.1/13.9、4 §4.4.3、15 §§15.4.4/15.11.5、16 §§16.2/16.3.3/16.7.3、[`pass_parser_simple_statements_001.miz`](../../../../tests/miz/pass/parser/pass_parser_simple_statements_001.miz)、[`fail_type_elaboration_set_enumeration_formula_gap_001.miz`](../../../../tests/miz/fail/types/fail_type_elaboration_set_enumeration_formula_gap_001.miz) + expectation/trace | exact checker 4 + runner 5 names/matricesを[checker](../../mizar-checker/ja/00.crate_plan.md#task-258b3m2b2b3a-frozen-contract)/[runner](../../mizar-test/ja/00.crate_plan.md#checker-task-258b3m2b2b3a-runner-frozen-contract) frozen contractsがownする。 |
| C | Chapters 13 §13.5、4 §4.4.3、15 §15.4.4、16 §16.3.3、[`pass_parser_primary_terms_001.miz`](../../../../tests/miz/pass/parser/pass_parser_primary_terms_001.miz)、[`fail_type_elaboration_local_set_choice_qua_term_gap_001.miz`](../../../../tests/miz/fail/types/fail_type_elaboration_local_set_choice_qua_term_gap_001.miz) choice member + expectation/trace | exact checker 4 + runner 5 names/matricesを[checker](../../mizar-checker/ja/00.crate_plan.md#task-258b3m2b2b3c-frozen-choice-witness-contract)/[runner](../../mizar-test/ja/00.crate_plan.md#checker-task-258b3m2b2b3c-runner-frozen-contract) frozen contractsがownする。 |
| D | Chapters 13 §13.6、4 §4.4.3、15 §15.4.4、16 §16.3.3、[`pass_parser_qua_terms_001.miz`](../../../../tests/miz/pass/parser/pass_parser_qua_terms_001.miz)、same fail fixtureの`equals 4 qua set;` member + expectation/trace | exact checker 4 + runner 5 names/matricesを[checker](../../mizar-checker/ja/00.crate_plan.md#task-258b3m2b2b3d-frozen-qua-witness-contract)/[runner](../../mizar-test/ja/00.crate_plan.md#checker-task-258b3m2b2b3d-runner-frozen-contract) frozen contractsがownする。 |
| E | Chapters 13 §§13.4/13.4.2、4 §4.4.3、15 §15.4.4、16 §16.3.3、[`pass_parser_set_comprehensions_001.miz`](../../../../tests/miz/pass/parser/pass_parser_set_comprehensions_001.miz) omitted-condition case、same fail fixtureの`{3 where candidate255 is set}` member + expectation/trace | exact checker 4 + runner 5 names/matricesを[checker](../../mizar-checker/ja/00.crate_plan.md#task-258b3m2b2b3e-condition-free-comprehension-witness-frozen-contract)/[runner](../../mizar-test/ja/00.crate_plan.md#checker-task-258b3m2b2b3e-runner-frozen-contract) frozen contractsがownする。 |

| Class | Decision |
|---|---|
| `design_drift` | paired completion-log H3 8節がEN/JA payload-family owner内で同じtime-local evidence形状を反復する。paired historical contractsをcompletion-evidence ownerとする。 |
| `spec_gap` / `test_gap` | 本structural migrationにはない。generic schema-v1 lintがexact whole-section shapeをcoverする。 |
| `source_drift` / `source_undocumented_behavior` | 導入・推測しない。Rust/Cargoはprotected。 |
| `test_expectation_drift` | なし。specificationと全test-intent artifactはprotected。 |
| `boundary_violation` | 全frozen H3/H2 owner、全unlisted section、全component API/invariant/runner/audit owner、B3B non-paired completion recordを保持して回避する。 |
| `repo_metadata_conflict` | historical remote-ref movementはreport-only/human-owned。current `0/6` distanceを実測し、fetch/reset/pushで修復しない。 |

## Frozen preimage and anchors

[`DOC-258B3M2B2B3-ACDE-COMPACT.sources.tsv`](../DOC-258B3M2B2B3-ACDE-COMPACT.sources.tsv)
はbyte-sort済みdata 8 rows + comment 2行 + final LF。data-row SHA-256は
`9046a2fa4a71e210ecf2e4d3fb1f115e426b070e7a5b434eb81dfc9fa4598fcc`、
complete 10-line TSV SHA-256は
`cad05407f570a7305bf31168a78de2a5dd577577b0abd6f7267fe07628010b5e`。

selectionは2 physical paths上のunique `(path, task)` H3 8節、157 physical
lines。EN/JA `4/79`・`4/78`、checker/runner `8/157`・`0/0`。nested heading、
table、fenceはなく、contract/index/ledger identity collisionもない。

| Task | Retained preceding / following same-or-higher heading |
|---|---|
| A EN | `### Task 258B3M2B2B3A Frozen Upper-Family Edge` / `### Task 258B3M2B2B3B Frozen Zero-Edge Family Boundary` |
| A JA | `### Task 258B3M2B2B3A frozen upper-family edge` / `### Task 258B3M2B2B3B frozen zero-edge family boundary` |
| C EN | `## Task 258B3M2B2B3C Choice-Witness Family` / `### Task 258B3M2B2B3D Frozen Qua-Witness Edge` |
| C JA | `## Task 258B3M2B2B3C choice-witness family` / `### Task 258B3M2B2B3D frozen qua-witness edge` |
| D EN | `### Task 258B3M2B2B3D Frozen Qua-Witness Edge` / `### Task 258B3M2B2B3E Frozen Comprehension-Witness Edge` |
| D JA | `### Task 258B3M2B2B3D frozen qua-witness edge` / `### Task 258B3M2B2B3E frozen condition-free-comprehension witness edge` |
| E EN | `### Task 258B3M2B2B3E Frozen Comprehension-Witness Edge` / `## Task 258B4 Composite-Root Decomposition` |
| E JA | `### Task 258B3M2B2B3E frozen condition-free-comprehension witness edge` / `### Task 258B4 composite-root decomposition` |

B3Bは明示的除外である。EN implementation resultはH3、JA completion ownerは
H2で、schema v1はこのasymmetryを表現できない。両sideとも変更しない。

## Frozen protected baseline

selection-HEAD replayは以下のtask-independent protected inventoryをfreezeする。
prerequisite/migrationのexpected deltaは全rowでzero。

| Surface | Paths | Path SHA-256 | Content SHA-256 |
|---|---:|---|---|
| specification | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

protected trace SHA-256は
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`。
expected CLI stdout hashはplan `700f4bf5`、parse-only `a8a7aa63`、
declaration-symbol `71e83ba0`、type-elaboration `4b2c7bd5`、
proof-verification `ccf3d2d4`。final evidenceがreproduced full hashを1回記録する。

## Scope, impact, and exit

prerequisiteはexact 15 pathsを変更する: 本EN/JA batch pair、historical
contract 4 pairs、language-neutral source TSV、crate plan 4件。各planへA/C/D/E/
batch rowsを追加し、index recordは20件。selected preimageと
`legacy_compactions.tsv`は変更しない。

separate prerequisite commitとfresh replay後、migrationは8 complete H3だけを
各historical contract `#completion-evidence`へのlanguage-local redirectへ置換
できる。変更はpayload 2 files、本EN/JA batch pair、ledgerのexact 5 paths。
mapped 157 physical linesはredirect+separator 16 linesとなり141減。expected
ledger impactはbatch 1、task 4、redirect 8 / distinct source path 2、index 20、
expanded-inventory hash 1。source TSV、historical contracts、4 Task Indexesは
immutableになる。

specification、`.miz`、fixture、sidecar、expectation、trace status/count/
backlink、coverage credit、root coverage audit、source、Cargo、public API、
diagnostic、executable behaviorはunchanged。goal/guard composition、proof/
discharge/acceptance、fact、Core/CFG/VC/ATP state、binding/capture、sethood/type
semantics、active dispatch、B4/B5 behavior、new coverage creditは禁止する。
`doc/design/spec_coverage_audit.md`はowned mapping/statusが不変なので変更しない。

prerequisite/migration reviewsはchronology、8 preimages/anchors、task-specific
fact、retained-owner link、EN/JA equivalence、exact scope、index/ledger arithmetic、
no-impact claimをindependently reproduceし、**NO FINDINGS**で終える。verificationは
recursive contract/link/fragment + generic-ledger lint、full lint policies、checker/
runner/metadata tests、fmt、Cargo metadata、warnings-denied Clippy、workspace tests、
全5 CLI、protected count/hash、`git diff --check`、exact staging、全9 hard gates、
uncapped `>=90/100`を含む。push/stash mutationは禁止する。

## Documentation-prerequisite evidence

pre-edit selection reviewは**NO FINDINGS**で、alternate Task-258B3 whole-result
familyはTyped/Resolved/runner-plan/boundary H3にdurable owner-local factを含む
ため除外した。contract reviewはexact canonical/executable authorityとexisting
test ownerがindirectというmedium `design_drift`を1件検出した。task-specific
authority/test mapとhistorical-contract rows追加後のfinding-specific re-reviewは
**NO FINDINGS**。equivalence/ownership/EN-JA reviewはA/D completion evidenceの
省略をmedium `design_drift` 2件として検出した。coverage/fresh chronology、
complete `qua` composition、test split、deferral、correction、operational residual、
pending/closure chronology復元後のre-reviewは**NO FINDINGS**。independent
test-sufficiency reviewも**NO FINDINGS**で、generic schema-v1 lintが十分で
task-specific Rust/schema変更は禁止されると確認した。

全8 preimages / 157 linesはexact heading/hash/anchorでreplayし、両TSV hashと
`4/79`、`4/78`、`8/157`、`0/0` partitionが一致した。recursive task-contract/
link/fragment lint `1/1`、full runner/checker lint policies `15/15`・`15/15`、
checker/runner libraries `530/530`・`600/600`、runner metadata `137/137`はPASS。
`cargo fmt --all --check`、Cargo metadata、warnings-denied all-target/all-feature
Clippy、full workspace tests、`git diff --check`もPASSした。

全5 CLIはunchanged 23 warningsとともにexit zeroし、stdout SHA-256は以下。

| CLI | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

protected count/path hash/content baseline/trace hash、selected preimage、legacy
ledger、root coverage auditはtask delta zeroでreproduceした。worktreeはexact
15-path prerequisite。

final read-only quality reviewは**NO FINDINGS**。全9 hard gatesはPASS、score
capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）。両lint policies
`15/15`、`git diff --check`、exact scope、20 indexes、TSV/preimage/anchor
replay、protected no-op、repository metadataをindependently reproduceした。
residual riskはseparate commitとなるmigrationとretained historical semantic/
coverage deferralだけ。exact task-only staging/cached review、commit、
post-commit replayがpending。

## Handoff

両task-only commitとclean post-commit inventory後、dependency-readyなchecker-owned
whole-section duplication familyをexactly 1件選択する。parentは`xhigh`、independent
semantic/equivalence reviewは`high`、deterministic inventoryだけ`medium`とする。
