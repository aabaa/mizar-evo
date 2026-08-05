# Task DOC-269G-INTERMEDIATE-COMPACT: given intermediate completion compaction

> canonical English:
> [../en/DOC-269G-INTERMEDIATE-COMPACT.md](../en/DOC-269G-INTERMEDIATE-COMPACT.md)。

本derived documentation-maintenance contractは削除前にcoherent legacy familyを
freezeする。language behavior、test intent、API、diagnostic、traceability、coverage
creditを追加・再解釈しない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269G-INTERMEDIATE-COMPACT` |
| Status | redirect migration、schema-v1 ledger expansion、independent reviews、full verification、final quality review完了。全9 gates PASS、score capなし100/100。exact staging/commitが残る。 |
| Purpose | contiguous GUPT → GU → GCP → GC chainのcompletion-only H3をcentralizeし、全frozen H2 product ownerを保持する。 |
| Owners | repository migration policy、paired historical contracts 4件、[checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |
| Consumers | checker-first EN/JA design documents 36件、`mizar-test` consumer docs、Task Index 4件、post-migration schema-v1 ledger/lint |
| Dependencies | GUPT `c5292451`、GU `998dc104`、GCP `59eb7de6`、GC `8181ae8f`、manifest consumer `0ec5fce2`、prior compaction `34b42908` |
| Readiness | documentation prerequisite commit `cb03a208`。fresh clean selection inventoryは`origin/main...HEAD=0/2`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`。blocking authority gapなし。 |

## Authority And Classification

authorityはuserのchecker-first consolidation decision、
[`AGENTS.md`](../../../../AGENTS.md)、
[migration policy](../../autonomous_crate_development.md#migration-policy)、completed task
records 4件、current frozen H2/H3 design owners。source behaviorはnormativeでない。

| Class | Decision |
|---|---|
| `design_drift` | completion-only H3 142節がoverlapping historical status/measurement/exclusion/review evidenceを36 pathsで反復する。bytesはuniqueなので削除前にexact preimage/fact equivalenceをfreezeする。 |
| `test_gap` | なし。schema v1は`(path, task)`ごとのcomplete-section redirect 1件を扱え、generic 15-test consumerのRust/test-count変更不要。 |
| `spec_gap` | structural migrationにはなし。semantic issueを選択しない。 |
| `source_drift` | なし。production sourceはprotected。 |
| `source_undocumented_behavior` | 導入・推測しない。 |
| `test_expectation_drift` | なし。specification、`.miz`、fixture、sidecar、expectation、trace、metadataはprotected。 |
| `boundary_violation` | completion-only H3をhistorical contractへ移し、全frozen H2 module/audit/runner/trace/sequencing/deferral ownerを保持して回避。 |
| `repo_metadata_conflict` | implementation selection時はprior compactionと本task prerequisite commits後の`origin/main...HEAD=0/2`。その後reflogはexternal `update by push`による`cb03a208`でのalignment (`0/0`)を記録。両observationsはreport-onlyで、repair/agent pushは禁止。 |

## Frozen Preimage Inventory

language-neutral
[`DOC-269G-INTERMEDIATE-COMPACT.sources.tsv`](../DOC-269G-INTERMEDIATE-COMPACT.sources.tsv)
はbyte-sorted data rows 142件、comments 2行、final LFを持つ。各data rowはtask、
language、component、exact path、ATX level、prefixなしexact heading text、section
SHA-256、physical linesを記録する。raw headingは`#` 3 bytes、space、heading textで
reconstructし、sectionは次のvisible H3以上ATX heading直前まで。migration前にclean
HEADへexplicit replayして全row一致が必要。

commentsを除くtask-local preimage data-row SHA-256は
`038f79e147a1d3f04d20edc1ca1493974f151ef6aa757d29070216b41ce5bd2c`。
144-line physical TSV SHA-256は
`a6e539beb0d04137fdb0d90d011553eff86d9a655e86d63769ad0642a2d1eb55`。
これはreview evidenceであり、future manifest expanded-inventory hashでも現時点の
lint-enforced valueでもない。

| Task | Sections | Physical lines | Distinct paths | Historical owner |
|---|---:|---:|---:|---|
| 269GUPT | 34 | 305 | 34 | [contract](./269GUPT.md#completion-evidence) |
| 269GU | 36 | 303 | 36 | [contract](./269GU.md#completion-evidence) |
| 269GCP | 36 | 313 | 36 | [contract](./269GCP.md#completion-evidence) |
| 269GC | 36 | 437 | 36 | [contract](./269GC.md#completion-evidence) |
| **Total** | **142** | **1,358** | **36 union paths** |  |

exact EN/JA symmetric path/task matrix:

| Component | Relative file | Selected tasks per language |
|---|---|---|
| mizar-checker | `00.crate_plan.md` | GUPT, GU, GCP, GC |
| mizar-checker | `bilingual_sync_audit.md` | GUPT, GU, GCP, GC |
| mizar-checker | `binding_env.md` | GUPT, GU, GCP, GC |
| mizar-checker | `module_boundary_audit.md` | GUPT, GU, GCP, GC |
| mizar-checker | `payload_family_decomposition.md` | GUPT, GU, GCP, GC |
| mizar-checker | `resolved_typed_ast.md` | GUPT, GU, GCP, GC |
| mizar-checker | `semantic_spec_audit.md` | GUPT, GU, GCP, GC |
| mizar-checker | `source_proof_local_declaration.md` | GUPT, GU, GCP, GC |
| mizar-checker | `source_spec_audit.md` | GUPT, GU, GCP, GC |
| mizar-checker | `source_statement.md` | GUPT, GU, GCP, GC |
| mizar-checker | `source_term.md` | GU, GCP, GC |
| mizar-checker | `source_type.md` | GUPT, GU, GCP, GC |
| mizar-checker | `typed_ast.md` | GUPT, GU, GCP, GC |
| mizar-test | `00.crate_plan.md` | GUPT, GU, GCP, GC |
| mizar-test | `bilingual_sync_audit.md` | GUPT, GU, GCP, GC |
| mizar-test | `harness.md` | GUPT, GU, GCP, GC |
| mizar-test | `module_boundary_audit.md` | GUPT, GU, GCP, GC |
| mizar-test | `traceability.md` | GUPT, GU, GCP, GC |

selected headingはfrozen rows外に存在しない。GCP JA
`Task 269GCP condition-profile decomposition` H3はdurable payload ownerなので除外。
これによりschema v1のone redirect per `(path, task)`も保持する。selected GCP payload
completion H3はretained `Task 269GCP condition-profile decomposition` H3をactual
preceding same-level anchorとして使い、GCP H2を発明しない。

## Documentation-Prerequisite Scope

prerequisiteはexact 15 pathsを変更する。本EN/JA、historical contract 4 pairs、
language-neutral source TSV、checker/test EN/JA crate plans。各planへhistorical 4件+
batch 1件のlanguage-local Task Index rowsを5件、合計20 rows追加する。

migration sources 36、existing `legacy_compactions.tsv`、Rust、Cargo、specification、
`.miz`、fixture、sidecar、expectation、trace、metadata、root audit、count/hash/status、
executable behaviorは変更しない。coverage/design mapping/follow-up owner/deferral status
に影響しないため`doc/design/spec_coverage_audit.md`は不変。

## Frozen Migration And Ownership Boundary

prerequisite commit/fresh replay後、implementationはlisted complete H3をtask contract
`#completion-evidence`へのlanguage-local redirect 1件で置換する。変更はsource paths
36、本EN/JA status/evidence、`legacy_compactions.tsv`だけの39 paths。ledgerへbatch 1、
tasks 4、redirects 142、index records 20とnew independent manifest inventory hashを追加。
source TSV/historical contractsはimmutable。

historical contractsはconsolidated completion measurement/review evidenceだけを所有。
module docsはdurable public/private API、fingerprints、validation、binding/lookup、payload、
Typed/Resolved、runner、audit、trace、bilingual、sequencing、deferralをfrozen H2で保持。
H2は削除・rewriteしない。

given-scope statementはpreservation-only。bindingはown `such that`、innermost proof/
reasoning block残余、unshadowed descendantsで有効だがparent/sibling/post-exitで無効。
goal、guard、fact、equality、condition meaning、proof、discharge、acceptance、initial
obligation、capture/export、IR、VC、ATP、Task-270 behaviorを発明しない。GUPT、GU、
GCP、GCはdistinctのまま、GCT/GCU ownershipは不変。

## Tests, Reviews, And Exit

prerequisite reviewはpolicy/scope、全142 preimages/preserved facts、durable-owner
exclusions、EN/JA equivalence/linksを独立確認し **NO FINDINGS** にする。verificationは
explicit TSV replay、path/task/count/hash、GCP exclusion、recursive task-contract pair/
link/fragment lint、full lint policy、checker/runner libraries、metadata、checker lint、
format、Cargo metadata、warnings-denied workspace Clippy、full tests、全5 CLI/protected
hashes、protected-scope、`git diff --check`、exact 15-path staging、全9 gates/score capなし
`>=90/100`。

one docs-only prerequisite commit後、fresh inventoryでsame batchを再選択してから実装。
implementationはtest-sufficiency、equivalence/boundary、source/document/EN-JA、final
qualityを別reviewし、exact 39 paths、separate commit。pushしない。parent reasoningは
`xhigh`、bounded mechanically frozen reviewは`high`を使用できる。

## Documentation-Prerequisite Evidence

frozen module-size mapping、language-exact GCP exclusion/anchor、JA GU fragmentを修正後、
independent policy/equivalence、test-sufficiency、implementation-boundary reviewはすべて
**NO FINDINGS**。parent/independent replayはTSV 142 rows、36 paths、全section hash/
physical-line count、EN/JA 71/71、task別4 counts、retained GCP decomposition 2節と一致。
Task Index 4件はnew language-local records exact 20件を持つ。

focused recursive task-contract lintとfull 15-test lint policyはpass。
`mizar-checker` library tests 530/530、`mizar-test` library tests 600/600、runner
metadata tests 137/137、checker lint 15/15がpass。`cargo fmt --all --check`、Cargo
metadata、warnings-denied workspace Clippy、full workspace testsもpass。target CLI 5件は
全て正常終了し、output hashesはplan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、parse
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、declaration
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、type
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、proof
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`。
trace hashは
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`
のまま。protected specification/tests/traceability/production/Cargo/root audit/
migration sources/manifest pathsは不変。final read-only quality reviewは
**NO FINDINGS**、全9 hard gates **PASS**、score capなし **100/100**。prerequisite
commit前にcached 15-path auditだけが残る。

## Implementation Evidence

fresh post-prerequisite replayは編集前にfrozen preimages 142件すべてと一致。mechanical
migrationはdeclared source documents 36、本EN/JA status/evidence、
`legacy_compactions.tsv`のexact 39 pathsを変更する。completion-section lines 1,216件を
削除し、standard language-local redirect lines 142件を追加。matching historical headingは
durable GCP decomposition H3 owners 2件だけで、frozen H2 product ownerは変更しない。

ledgerはbatch 1、task records 4、distinct source paths 36のredirects 142、index
records 20をexact追加。declared expanded-inventory SHA-256は
`d934963a0043aa5a6b7c4b04bbc86ee27875484c6a2d58cff040fcb493c8b3b3`、
complete physical ledger SHA-256は
`f18988333588664aab1e9bb1c92382100f2b240ce04fb59229c09cea19a83283`。
unchanged generic schema-v1 lint consumerはcomplete migrationをacceptする。

specification、`.miz`、fixture、sidecar、expectation、trace TOML/status/backlink、
coverage credit、active outcome、production、Cargo、public API、diagnostic、root
coverage audit、source inventory TSV、historical contractsは不変。paired traceability
design documentsはselected completion evidenceのredirectだけを変更する。protected trace hashは
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`
のまま。

independent test-sufficiency/equivalence/boundary reviewsは **NO FINDINGS**。
source/document/EN-JA reviewのlow wording findings 2件はorigin distanceと
traceability scope wordingを修正し、finding-specific re-reviewが **NO FINDINGS**。
full `mizar-test` lint policy 15 tests、checker 530-test/runner 600-test libraries、
runner metadata 137/137、checker lint 15/15がpass。`cargo fmt --all --check`、Cargo
metadata、warnings-denied workspace Clippy、full `cargo test`、prerequisite記録と同じ
hashを持つtarget CLI 5件、protected count/hash checks、`git diff --check`もpass。
final read-only quality reviewのlow `repo_metadata_conflict` wording finding 1件を修正し、
finding-specific re-reviewは **NO FINDINGS**、全9 hard gates **PASS**、score capなし
**100/100**。commit前にexact stagingだけが残る。
