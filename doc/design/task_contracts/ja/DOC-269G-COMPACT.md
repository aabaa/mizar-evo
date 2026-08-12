# Task DOC-269G-COMPACT: given-family completion-evidence compaction

> canonical English:
> [../en/DOC-269G-COMPACT.md](../en/DOC-269G-COMPACT.md)。
> 本文書は同一logical taskの日本語companionである。

これはcompleted historical evidenceを保持するderived documentation-maintenance
contractであり、language behavior、test intent、diagnostic、public API、coverage creditを
追加・再解釈しない。

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269G-COMPACT` |
| Status | 完了。migrationはschema-2 ledgerに登録済みであり、task-local completion evidenceがcommitted migrationとclean replayを保存する。 |
| Purpose | exact shared Task-269GUP/GCT/GCU completion sectionをcentralizeし、全nonidentical plan/audit/trace-status/verification/boundary/sequencing ownerを保持する。 |
| Owners | repository documentation policyとdata-driven `mizar-test` legacy-compaction lint |
| Consumers | [checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index)、[runner plan](../../mizar-test/ja/00.crate_plan.md#task-index)、declared source documents 40件、versioned compaction manifest |
| Dependencies | Task 269GUP `076c1425`、Task 269GCT `d6fb0ed2`、Task 269GCU `f984ae68`、`DOC-COMPACT-MANIFEST` `0ec5fce293a6105e04761c5298b605d3f4ff60ca`、generic multi-batch mutation prerequisite `deb2e823ef6bc5d68a53aa871a4a9dd7ed333253` |
| Readiness | implementationはclean HEAD `deb2e823`、`origin/main...HEAD=0/4`、protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`から開始。blocking authority gapなし。 |

## Authority And Classification

authorityはuserのchecker-first documentation-consolidation decision、
[`AGENTS.md`](../../../../AGENTS.md)、
[autonomous protocol](../../autonomous_crate_development.md#migration-policy)、3 historical
task records、exact current shared sectionsである。language specification/semantic test
authorityは変更しない。

| Class | Decision |
|---|---|
| `design_drift` | exact shared completion 116節・2,567行がcentral historical ownerなしに40 filesで反復する。 |
| `test_gap` | なし。generic schema-v1 manifest consumerがtask-specific Rust/test-count changeなしでencodeできる。 |
| `spec_gap` | structural migrationにはなし。semantic issueは選択しない。 |
| `source_drift` | なし。production sourceはprotected。 |
| `source_undocumented_behavior` | 導入・推測しない。 |
| `test_expectation_drift` | なし。`.miz`、sidecar、expectation、trace dataはprotected。 |
| `boundary_violation` | 全nonidentical/owner-local sectionを除外し、mixed checker-plan GCT heading 2件を削除せずdisambiguateすることで回避する。 |
| `repo_metadata_conflict` | safe targetを妨げない。documentation-prerequisite inventoryでは`origin/main`がHEADより2 commits behindだったが、implementation中にexternal refがagent pushなしで`deb2e823`（`0/0`）へ進んだ。report-only metadata movementであり、repair/pushは禁止。 |

## Documentation-Prerequisite Scope

prerequisiteはexact 12 Markdown pathsを変更する。269GUP/GCT/GCUのnew EN/JA historical
contracts、本EN/JA batch pair、checker/test EN/JA crate plans 4件である。各planへnew
contract 4件のTask Index rowをexact 4件、合計16 rows追加する。加えてnonidentical
checker-plan GCT headings 2件だけをrenameする。

- EN: `### Task 269GCT implementation status` →
  `### Task 269GCT plan-local implementation and GCU sequencing status`
- JA: `### Task 269GCT implementation status` →
  `### Task 269GCT plan-local implementation/GCU sequencing status`

heading下bodyはbyte-identicalに保持する。manifest、lint Rust、他design file、specification、
test、fixture、sidecar、expectation、trace、Cargo、production source、protected artifact
count/hash/status、executable behaviorは変更しない。

## Exact Shared Inventory

各sectionはexact listed H3 headingから次のvisible H3以上ATX heading直前まで。hashはheading、
内部/final LF bytes、blank linesを含むphysical UTF-8 section bytesを対象とし、following
headingを除外する。各row内sectionはbyte-identicalで、以下だけを置換できる。

| Task | Owner/language | Exact heading | SHA-256 | Sections | Lines |
|---|---|---|---|---:|---:|
| 269GUP | checker/en | `### Task 269GUP implemented binding profile` | `a2253a41346c83b0e4ea477d8ab864ca9171b015e2e0aba15e91af98edbd4af3` | 13 | 78 |
| 269GUP | checker/ja | `### Task 269GUP binding profile 実装状況` | `21f913d74088901df228d34b8e97d626f35e32522757a2a860bb8c1b40ee9ca9` | 13 | 78 |
| 269GUP | test/en | `### Task 269GUP implemented dormant runner` | `8e84a6249a185787602017e7645fb9ac7f62144827c35cdb3aebac428ece6222` | 6 | 36 |
| 269GUP | test/ja | `### Task 269GUP dormant runner 実装状況` | `6701ebec916449691c5acdbd09953c49323f1f508a281c1dfc8bdee95bff3c0e` | 6 | 36 |
| 269GCT | checker/en | `### Task 269GCT implementation status` | `b21d19691d7ee99d1bb27425fc1fdecd9986dcaa0090a984bf4ee218cc84b65f` | 13 | 416 |
| 269GCT | checker/ja | `### Task 269GCT implementation status` | `da82c262c070b9ac85f6e94d5d56df78d6eb02ce7154c0222515cd390ef293ed` | 13 | 403 |
| 269GCT | test/en | `### Task 269GCT implemented private runner status` | `521d1fd500d969bc8a7c7728372072a522bca17c7a60d15ac5a4d65e2c75443e` | 6 | 108 |
| 269GCT | test/ja | `### Task 269GCT implemented private runner status` | `b01173bc886ea3fd6476e3f7130486e4064aa9fac00e010482d70b6891dcf947` | 6 | 108 |
| 269GCU | checker/en | `### Task 269GCU implementation status` | `3569b601b33c119f5147b6953c4dc14bcd56cae084ffc08c003fe36faa91827a` | 14 | 532 |
| 269GCU | checker/ja | `### Task 269GCU implementation status` | `1b0492c909233d924eca8f725bccefa838256ecd3eca63677b190d0a7c9990f1` | 14 | 532 |
| 269GCU | test/en | `### Task 269GCU implemented private runner status` | `3319584f02a98e2dee03f48640fa32a3dd2edf7237bffc57531f27be9eb0ada5` | 6 | 120 |
| 269GCU | test/ja | `### Task 269GCU implemented private runner status` | `a2ed60dfa2cb8c4df00523be089dc9cc142c7186371c00ee9c97a4cbb26baeb9` | 6 | 120 |
| **Total** |  |  |  | **116** | **2,567** |

exact path/task matrixはEN/JA symmetricである。

| Component | Relative file | Authorized tasks per language |
|---|---|---|
| mizar-checker | [`00.crate_plan.md`](../../mizar-checker/ja/00.crate_plan.md#checker-task-269gcu-frozen-given-condition-termreference-plan) | 269GCU only |
| mizar-checker | [`binding_env.md`](../../mizar-checker/ja/binding_env.md#task-269gup-new-source-binding-profile) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`bilingual_sync_audit.md`](../../mizar-checker/ja/bilingual_sync_audit.md#task-269gup-documentation-synchronization) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`module_boundary_audit.md`](../../mizar-checker/ja/module_boundary_audit.md#task-269gup-frozen-boundary) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`payload_family_decomposition.md`](../../mizar-checker/ja/payload_family_decomposition.md#task-269gup-payload-delta) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`resolved_typed_ast.md`](../../mizar-checker/ja/resolved_typed_ast.md#task-269gup-final-owner-exclusion) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`semantic_spec_audit.md`](../../mizar-checker/ja/semantic_spec_audit.md#task-269gup-zero-semantic-boundary) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_proof_local_declaration.md`](../../mizar-checker/ja/source_proof_local_declaration.md#checker-task-269gup-frozen-use-profile-binding-prerequisite) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_spec_audit.md`](../../mizar-checker/ja/source_spec_audit.md#task-269gup-frozen-sourceapi-delta) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_statement.md`](../../mizar-checker/ja/source_statement.md#task-269gup-statement-boundary) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_term.md`](../../mizar-checker/ja/source_term.md#task-269gup-source-term-exclusion) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`source_type.md`](../../mizar-checker/ja/source_type.md#task-269gup-source-type-exclusion) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`todo.md`](../../mizar-checker/ja/todo.md#checker-task-269gup-proof-given-use-profile-binding-prerequisite) | 269GUP, 269GCT, 269GCU |
| mizar-checker | [`typed_ast.md`](../../mizar-checker/ja/typed_ast.md#task-269gup-typed-owner-exclusion) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`00.crate_plan.md`](../../mizar-test/ja/00.crate_plan.md#task-269gup-frozen-dormant-binding-profile-prerequisite) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`bilingual_sync_audit.md`](../../mizar-test/ja/bilingual_sync_audit.md#checker-task-269gup-documentation-synchronization) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`harness.md`](../../mizar-test/ja/harness.md#checker-task-269gup-frozen-dormant-harness) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`module_boundary_audit.md`](../../mizar-test/ja/module_boundary_audit.md#checker-task-269gup-frozen-runner-boundary) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`todo.md`](../../mizar-test/ja/todo.md#checker-task-269gup-dormant-use-profile-binding-prerequisite) | 269GUP, 269GCT, 269GCU |
| mizar-test | [`traceability.md`](../../mizar-test/ja/traceability.md#checker-task-269gup-zero-credit-trace-boundary) | 269GUP, 269GCT, 269GCU |

各languageでchecker 14 + test 6、exact distinct paths 40へexpandする。manifestは全116
path/task rowsとactual nearest same-or-higher-level anchorsを列挙し、wildcard/inferred
sourceを禁止する。

## Redirect And Manifest Contract

各authorized sectionをmatching historical contract `#completion-evidence`へのexact one
language-local reserved lineで置換する。ENはperiod、JAは`。`で終える。implementationは
`legacy_compactions.tsv`へ`batch` 1 (`DOC-269G-COMPACT`)、`task` 3、`redirect`
116、exact Task Index `index` 16 recordsを追加し、complete sorted rowsからcountsと
expanded-inventory SHA-256を再計算する。task-specific Rust branch/test/test-name/count
changeは禁止する。

## Explicit Exclusions And Deferrals

全nonidentical Task-269GUP checker-plan completion section、retained GCT plan-local body 2件、
GCT/GCU documentation-prerequisite verification section、frozen zero-credit trace-status H2、
shared H3 evidence周囲の全H2 owner sectionを除外する。root
`spec_coverage_audit.md` Task-269GUP implementation auditは不変。

migrationはgiven-scope semanticsを決定・変更しない。`given` bindingがinner shadowingを
伴いinnermost blockの残余とdescendant blockで有効で、descendant use/captureが別である
existing historical statementを保持する。goal、guard、fact、equality、proof、discharge、
acceptance、obligation、export、capture、IR、VC、Task-270 behaviorを発明しない。

`doc/design/spec_coverage_audit.md`はcoverage/ownership status impactなしで変更しない。
production、public API、tests、trace、corpus、全5 CLI outputs/hashesを保護する。

## Documentation-Prerequisite Evidence

- independent specification/policy、exact-inventory/boundary、EN/JA/owner-link
  reviewは、全stable owner links追加後に **NO FINDINGS**。recursive local
  link/fragment validationと`git diff --check`はpass。
- fresh replayはfrozen group hashes 12件とexact 116 sections、physical 2,567 lines、
  40 pathsを再現する。4 plansにはselected index rowがexact 16件。変更するのはdeclared
  checker-plan heading 2件だけで、retained bodiesはbyte-identical。
- existing manifestのphysical hashは
  `c537eda8401c1cdc0a3386ca648d112075b0728b702b56d03f89e353d4a4347f`
  のまま、one batch、two tasks、82 redirects、42 source paths、12 index rows。
  specification、test、trace、source、Cargo、coverage-audit path変更なし。
- full lint policy 15 tests、checker library 530、runner library 600、metadata 137、
  checker lint 15、`cargo fmt --all --check`、Cargo metadata、warnings-denied
  workspace Clippy、full `cargo test`はpass。plan/parse/declaration/type/proof CLI hashは
  順に
  `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、
  `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、
  `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、
  `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、
  `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`
  のまま。protected trace hashは
  `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`
  のまま。

## Implementation Evidence

- fresh implementation preflightでgeneric mutation-oracle `test_gap`が判明した。first
  byte-sorted H3 redirectによりhard-coded H3 malformed headingがlevel-consistentになった。
  isolated one-line prerequisiteはmutationをalways-invalid H1へ変更し、independent reviewは
  **NO FINDINGS**、全gates `100/100`でPASS、commit
  `deb2e823ef6bc5d68a53aa871a4a9dd7ed333253`完了後にclean inventoryから本batchを
  再適用した。
- declared 40 pathsのfrozen section byte range exact 116件だけをlanguage-local redirectへ
  置換した。checker/test documentsでforbidden legacy headings 8種は0件、matching
  redirectはexact 116件。nonidentical plan、prerequisite-verification、zero-credit trace、
  owner-local、sequencing、root coverage-audit exclusionは保持した。
- 235-line manifestのphysical SHA-256は
  `d794d78662b570260f777e1b074ff20d7f5fa3ed911bb3c3e8730471ff96a46a`。
  global declarationはtwo batches、five tasks、198 redirects、28 index rows。本batchの
  independent inventory SHA-256は
  `deba263f24954ac6f7e081a3919933277fbb7152e5f256c38b9b992231716b53`、
  three tasks、116 redirects、40 source paths、16 index rowsを再現する。
- independent equivalence、test-sufficiency、source/document/EN-JA reviewは、
  report-only external-origin wordingを正確にした後 **NO FINDINGS**。focused/full lint
  policyは各15 tests、checker/runner libraryは530/600 tests、metadataは137 testsでpass。
  Cargo format、metadata、warnings-denied workspace Clippy、full `cargo test`はpass。
  全5 CLI hashとprotected trace hashは上記frozen prerequisite valueから不変。manifest
  counts、physical/inventory hashes、exact 43-path scope、forbidden-heading zero、116
  redirects、protected-path exclusion、`git diff --check`は全てpass。commit前には
  cached/unstaged staging auditだけを残す。

## Reviews, Verification, And Exit

documentation prerequisiteはindependent specification/policy、exact-inventory/boundary、
EN/JA reviewを **NO FINDINGS** にし、全9 hard gates PASS、score capなし`>=90/100`、
exact 12-path staging、one docs-only commit、clean post-commit inventoryを必要とする。

fresh inventory後のimplementationはdeclared source paths 40、本EN/JA status/evidence pair、
TSV manifestだけを変更する。independent test sufficiency、equivalence、source/document/
EN-JA reviewを **NO FINDINGS** にする。focused/full lint policy、checker/runner libraries、
metadata/checker lint、format、warnings-denied workspace Clippy、full `cargo test`、全5 CLI、
exact 116/2,567/path/hash replay、manifest counts/hashes、protected trace/corpus/source hashes、
local links、`git diff --check`、cached/unstaged audit、final 9/9 hard gates `>=90/100`を
verificationする。one task-only commitで完了し、agentはpushしない。

## Final Quality And Handoff

final read-only quality reviewのfindingはterminal handoff欠落だけで、全9 hard gates PASS、
score capなし。paired EN/JA correctionとfinding-specific re-review後のfinal scoreは
`100/100`。focused lintとdiff checkを再通過してから同じexact 43 pathsだけをstageする。

commit後はclean read-only repository/canonical-authority inventoryから開始する。
`mizar-checker`をfirst inventoryし、`mizar-test`は該当時だけconsumerとして扱う。migration
policyに従うdependency-ready duplication familyをexact 1件だけ選択・freezeし、本contractは
特定のnext batch/semantic taskを事前承認しない。ownership/byte-preservation boundaryが多数の
文書にまたがるためparentは`xhigh`を維持し、boundedかつmechanically frozenなreview packet
だけ`high` review agentを使用できる。
