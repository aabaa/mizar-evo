# Task DOC-269GT-COMPACT: proof-Given type completion compaction

> canonical English:
> [../en/DOC-269GT-COMPACT.md](../en/DOC-269GT-COMPACT.md)。

本derived documentation-maintenance contractはcompleted checker-first task familyを
削除前にfreezeする。language behavior、test intent、API、diagnostic、traceability、
coverage creditを追加・再解釈しない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269GT-COMPACT` |
| Status | 完了。migrationはschema-2 ledgerに登録済みであり、task-local completion evidenceがcommitted migrationとclean replayを保存する。 |
| Purpose | Task-269GT implementation-completion evidenceをcentralizeし、全prerequisite/verification/H2 product/runner/trace/TODO/semantic ownerを保持。 |
| Owners | repository migration policy、[historical 269GT contract](./269GT.md#completion-evidence)、[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Consumers | checker-first EN/JA design documents 38件、Task Index 4件、post-migration schema-v1 ledger/lint |
| Dependencies | 本batch prerequisite `133128bc`、Task-269GT prerequisite `35bc97b9`、implementation `1fc6cc01`、generic manifest consumer `0ec5fce2`、prior compaction `f3dd80bc` |
| Readiness | post-prerequisite clean HEAD `133128bc`、`origin/main...HEAD=0/2`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。編集前に全38 preimagesをreplayし、blocking authority gapなし。 |

## Authority And Classification

authorityはuserのchecker-first consolidation decision、
[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、completed
Task-269GT recordsとfrozen H2/H3 design owners。source behaviorはnormativeでない。

| Class | Decision |
|---|---|
| `design_drift` | completion-only H3 38節がimplementation status/measurement/exclusion/trace state/review evidenceを38 pathsで反復。削除前にexact preimage/fact ownerをfreezeする。 |
| `test_gap` | なし。schema v1はunique `(path, task)`ごとのcomplete-section redirectを扱い、generic 15-test consumerが本shapeをcoverする。 |
| `spec_gap` | structural migrationにはなし。semantic issueを選択しない。 |
| `source_drift` | なし。production sourceはprotected。 |
| `source_undocumented_behavior` | 導入・推測しない。 |
| `test_expectation_drift` | なし。specification、`.miz`、fixture、sidecar、expectation、trace TOML、metadataはprotected。 |
| `boundary_violation` | historical completion measurementをpaired 269GT contractへ移し、全H2とplan-local prerequisite/verification H3 8件を保持して回避。 |
| `repo_metadata_conflict` | なし。`origin/main`よりlocal 2 commits aheadはexpected preceding compactionと本batch prerequisiteによる。report-onlyでrepair/push禁止。 |

## Frozen Preimage Inventory

language-neutral
[`DOC-269GT-COMPACT.sources.tsv`](../DOC-269GT-COMPACT.sources.tsv)はbyte-sorted
data rows 38件、comments 2行、final LFを持つ。各rowはtask、language、component、exact
path、ATX level、prefixなしexact heading text、complete-section SHA-256、physical linesを
記録。raw headingは`#` 3 bytes、space、heading textでreconstructし、sectionは次のvisible
H3以上ATX heading直前まで。migration前にclean `f3dd80bc`へ全row replayが必要。

commentsを除くdata-row SHA-256は
`62cec2bf5412bd1ea89791b4df75dd1709d182d4f1aad699af8ffe988725482a`、40-line
physical TSV SHA-256は
`1dfde440f4ad4a2b7f203dee472640ccb0ea7cba2e6d937b2eeedaeac6809d86`。
future manifest expanded-inventory hashではない。

| Component | Relative file | Selected section per language |
|---|---|---|
| mizar-checker | `00.crate_plan.md` | Task-269GT implementation status |
| mizar-checker | `bilingual_sync_audit.md` | implementation synchronization |
| mizar-checker | `binding_env.md` | implemented overlay |
| mizar-checker | `module_boundary_audit.md` | implemented boundary |
| mizar-checker | `payload_family_decomposition.md` | implemented payload delta |
| mizar-checker | `resolved_typed_ast.md` | implemented final owner |
| mizar-checker | `semantic_spec_audit.md` | implementation semantic audit |
| mizar-checker | `source_proof_local_declaration.md` | implemented consumer status |
| mizar-checker | `source_spec_audit.md` | implemented source/API delta |
| mizar-checker | `source_statement.md` | implemented statement boundary |
| mizar-checker | `source_type.md` | implementation verification status |
| mizar-checker | `todo.md` | implementation handoff |
| mizar-checker | `typed_ast.md` | implemented typed owner |
| mizar-test | `00.crate_plan.md` | Task-269GT implementation status |
| mizar-test | `bilingual_sync_audit.md` | implementation synchronization |
| mizar-test | `harness.md` | implemented dormant harness |
| mizar-test | `module_boundary_audit.md` | implemented runner boundary |
| mizar-test | `todo.md` | implementation handoff |
| mizar-test | `traceability.md` | implementation trace status |
| **Total** | **paired relative files 19 / physical paths 38** | **EN 19 + JA 19、physical lines 205** |

selected `(path, task)` identityは全unique。plan-local H3としてEN
`Task-269GT frozen source-type prerequisite`、EN
`Task-269GT documentation-prerequisite verification`、EN/JA
`Task-269GT frozen dormant type consumer`、JA
`Task-269GT frozen source-type prerequisite`、JA 2件の
`Task-269GT documentation prerequisite verification`を保持する。8 sectionsと全H2は
TSV外。

## Documentation-Prerequisite Scope

prerequisiteはexact 9 pathsを変更。本EN/JA、historical 269GT EN/JA、language-neutral
source TSV、checker/test EN/JA crate plans。各planへ269GT/batchのlanguage-local Task
Index records 2件、合計8 recordsを追加する。

migration sources 38のselected sections、existing `legacy_compactions.tsv`、Rust、Cargo、
specification、`.miz`、fixture、sidecar、expectation、trace TOML、metadata、root audit、
count/hash/status、executable behaviorは変更しない。coverage/design mapping/follow-up
ownership/trace status/semantic deferralに影響しないため
`doc/design/spec_coverage_audit.md`は不変。

## Frozen Migration And Ownership Boundary

prerequisite commit/fresh replay後、listed complete H3をhistorical contract
`#completion-evidence`へのlanguage-local redirect 1件で置換。変更はsource paths 38、本
EN/JA status/evidence、`legacy_compactions.tsv`だけの41 paths。ledgerへbatch 1、task 1、
38 sourcesのredirect 38、index records 8、new independently computed expanded-inventory
hashを追加。source TSV/historical contractはimmutable。

historical contractはcompletion measurement/review evidenceをown。module/runner/audit/
trace/bilingual/TODO/plan docsはdurable H2 contractを保持し、plansはfrozen-prerequisite/
documentation-verification H3 8件も保持。H2/unlisted H3の削除・rewrite禁止。

migrationはTask-269GT frozen source-provenance resultだけを保持。direct Given-binding/
generic type、condition/fact、existential/Skolem、assumption/guard、goal/initial obligation、
use/capture/export、proof/discharge/acceptance、Core/CFG/VC/ATP、active dispatch、coverage
credit、later-task behaviorを追加しない。

## Documentation-Prerequisite Evidence

independent contract/test-sufficiency/equivalence reviewsはhistorical prerequisite commitを
`35bc97b92ce075226105e8fcd4c1e43c8621995c`へ訂正後、すべて **NO FINDINGS**。
parent replayはselected sections 38、unique paths 38、physical lines 205、EN 19/JA 19、
checker 26/runner 12、excluded plan-local H3 8、new Task Index records 8を確認。data-row/
physical TSV hashesは上記frozen valuesと一致し、protected trace hashは
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`のまま。

recursive task-contract/link/fragment lint、full runner lint-policy 15 tests、checker lib
530 tests、runner lib 600 tests、metadata 137 tests、checker lint-policy 15 tests、
`cargo fmt --all --check`、Cargo metadata、warnings-denied all-target/all-feature Clippy、
full workspace tests、`git diff --check`はPASS。5 CLI stdout hashesは不変：

| Route | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

prerequisiteはexact 9 documentation pathsのまま。production、specification、tests、
fixtures、expectations、traceability、ledger、root coverage auditは不変。final read-only
quality reviewは **NO FINDINGS**、全9 hard gates PASS、score capなし **100/100**。
commit identityはexact staging/commit後に記録する。

## Implementation Evidence

documentation prerequisiteはcommit
`133128bc4d4909b8be8c3c2b5f8206fe8b94649b`。fresh post-commit inventoryはclean、
`origin/main...HEAD=0/2`、protected stash不変。編集前にparent/independent deterministic
replayが全38 frozen preimagesのheading/hash/physical line count/language/component
partition/neighboring anchorsと一致。

mechanical migrationはdeclared source documents 38、本EN/JA status/evidence pair、
`legacy_compactions.tsv`のexact 41 pathsを変更する。complete sections 38はseparator
blank lines 38を含むphysical 205 lines。replacementはcompletion-content 167 linesを
削除し、standard language-local redirect lines 38を追加、separatorを保持してmapped
intervalを129 lines削減。forbidden headings 38はすべて消滅、redirects 38は全unique、
全H2/excluded plan-local H3 8件は残り、unlisted task sectionは不変。

450-line ledgerはbatch 1、task record 1、distinct source paths 38のredirects 38、index
records 8をexact追加。expanded-inventory SHA-256は
`319638d715de101065fe65fd16a15f7bacbc07dc52db12dd8479cbcd492ad5e2`、complete
physical SHA-256は
`8c896ee2812b36435113bfb55cd1f65885d5d329d967401fa9251ad4c935ca37`。
source inventory physical hashは
`1dfde440f4ad4a2b7f203dee472640ccb0ea7cba2e6d937b2eeedaeac6809d86`のまま。

specification、`.miz`、fixture、sidecar、expectation、trace TOML/status/backlink、
coverage credit、active outcome、production、Cargo、public API、diagnostic、root coverage
audit、source inventory、historical 269GT contractは不変。paired traceability design
documentsはselected completion evidenceのredirectだけを変更。protected trace hashは
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`のまま。

independent test-sufficiency/equivalence/boundary reviewsは **NO FINDINGS**。
source/document/EN-JA reviewのmedium stale repository-distance sentence 1件はexpected
two-commit distanceへ同期後、finding-specific re-reviewが **NO FINDINGS**。

focused/full runner lint policy 1/1・15/15、checker/runner libraries 530/530・600/600、
runner metadata 137/137、checker lint 15/15がPASS。`cargo fmt --all --check`、Cargo
metadata、warnings-denied all-target/all-feature Clippy、full workspace tests、上記
prerequisite hashesと同じ全5 CLI、protected count/hash replay、`git diff --check`もPASS。
final read-only quality reviewは **NO FINDINGS**、全9 hard gates PASS、score capなし
**100/100**、scope内residual riskなし。exact staging、commit identityが残る。

## Tests, Reviews, And Exit

prerequisite reviewは38 preimages/fact ownership、retained H2/H3 exclusions、EN/JA
equivalence、plan indexes、local linksを独立確認し、全reviewを **NO FINDINGS** にする。
verificationはTSV replay/count/hash、recursive task-contract pair/link/fragment lint、full
lint policy、checker/runner libraries、metadata、checker lint、format、Cargo metadata、
warnings-denied workspace Clippy、full tests、全5 CLI/protected hashes、protected scope、
`git diff --check`、exact 9-path staging、全9 gates/score capなし`>=90/100`。

one docs-only prerequisite commit後、fresh inventoryでsame batchを再選択して実装。
implementationはtest-sufficiency、equivalence/boundary、source/document/EN-JA、final
qualityを別reviewし、exact 41 paths、separate commit。pushしない。parent reasoningは
`xhigh`、bounded reviewは`high`を使用可能。
