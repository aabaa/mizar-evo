# Task CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C: Fraenkel template structural composition

> canonical English: [../en/CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md](../en/CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md)。正本は英語です。

Owner plan は [mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)
と [mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。stable owner section は checker の [module API](../../mizar-checker/ja/source_template_type_parameter_association.md#task-277c-frozen-planned-public-extension)、[source/spec mapping](../../mizar-checker/ja/source_spec_audit.md#task-277c-frozen-sourcespecification-mapping)、[boundary](../../mizar-checker/ja/module_boundary_audit.md#task-277c-frozen-module-boundary)、[TODO](../../mizar-checker/ja/todo.md#task-277c-fraenkel-structural-composition)、[bilingual](../../mizar-checker/ja/bilingual_sync_audit.md#task-277c-frozen-contract-parity)、mizar-test の [harness](../../mizar-test/ja/harness.md#checker-task-277c-private-structural-composition-probe)、[boundary](../../mizar-test/ja/module_boundary_audit.md#checker-task-277c-frozen-module-boundary)、[TODO](../../mizar-test/ja/todo.md#checker-task-277c-private-structural-composition-probe)、[bilingual](../../mizar-test/ja/bilingual_sync_audit.md#checker-task-277c-frozen-contract-parity) である。

## 状態とauthority

**状態:** documentation prerequisiteをfreezeした段階であり、implementationは未開始である。本書は
同期companionであり、EN contractがcanonical authorityを持つ。

本taskは完了済みの`RESOLVE-TEMPLATE-TYPEPARAM-277R1`、`277B-L`、
`RESOLVE-FRAENKEL-GENERATOR-VAR-277R2`の後に置くneutral checker-only structural
compositionである。Task 277Bをreadyにせず、semantic coverage creditを与えず、仕様、fixture、
expectation、traceability、diagnostic、proof language、source behaviorを変更しない。authority順は
`doc/spec/en/13.term_expression.md` §§13.4.2、13.4.4、13.8.6、
`doc/spec/en/18.templates.md` §§18.2.1、18.2.2、18.2.6、18.10.2、immutableなF5
`.miz` fixture/expectation/trace、次にderived design/source recordである。既存authorityはread-onlyである。

閉じる対象はstructural handoffの`design_drift`、`source_drift`、Rust `test_gap`である。
`spec_gap`はなく、新しいtest intentも導出しない。language/type/proof/diagnostic、source producer、
production routeの判断は別途authorizeされたtaskへdeferする。

## Frozen boundaryとpublic ABI

implementationはexisting
`crates/mizar-checker/src/source_template_type_parameter_association.rs`内のneutral standalone
composition一つだけである。入力は次の三つに限定する。

- `&SourceTemplateTypeParameterAssociationHandoff`;
- `&FraenkelGeneratorVariableSourceCollection`; および
- `&TypedAst`.

R1 direct input、TypedAst/ResolvedAst slot/install、facade、production runner route、source-owner
変更、semantic interpretation、diagnostic、trace/coverage credit、`lib.rs`、lint Rustの変更は禁止する。

public familyは次のexact six typesである。

```rust
SourceTemplateFraenkelStructuralCompositionId
SourceTemplateFraenkelStructuralComposition
SourceTemplateFraenkelStructuralCompositionTable
SourceTemplateFraenkelStructuralCompositionHandoff
#[non_exhaustive] SourceTemplateFraenkelStructuralCompositionError
SourceTemplateFraenkelStructuralCompositionProducer
```

`SourceTemplateFraenkelStructuralCompositionProducer::build`は入力順を保つ
`build(template, generators, typed_ast) -> Result<
SourceTemplateFraenkelStructuralCompositionHandoff,
SourceTemplateFraenkelStructuralCompositionError>`である。error orderは次で固定する。

```rust
EnvironmentMismatch
InvalidTemplateAssociation { association }
InvalidGeneratorBinding { binding }
InvalidGeneratorUse { use_index }
InvalidComposition { composition: SourceTemplateFraenkelStructuralCompositionId }
UnmatchedTemplateAssociation { association: SourceTemplateTypeParameterAssociationId }
```

`InvalidComposition`はassociation IDではなくcomposition IDを保持し、orphan generator bindingも
表現可能にする。`UnmatchedTemplateAssociation`は全R2 binding candidate後に未消費のまま残る
最小dense association IDをreportする。spelling/source range/equal table position/castからrow identityを
推測してはならない。

IDは`new`と`index`のみ。row getterはexactに次の通りである。

- `template_association() -> SourceTemplateTypeParameterAssociationId`;
- `template_binding() -> TemplateTypeParameterBindingId`;
- `generator_binding() -> FraenkelGeneratorVariableBindingId`;
- `definition_block`、`parameter`、`template_binder`、`type_head`、`template_identifier`、
  `functor_definition`、`comprehension`、`segment`、`generator_binder`、`type_expression`、
  `mapper_role_owner`、`mapper_term_reference`、`mapper_identifier`、`first_condition_role_owner`、
  `first_condition_term_reference`、`first_condition_identifier`、`second_condition_role_owner`、
  `second_condition_term_reference`、`second_condition_identifier`は各`TypedNodeId`;
- `mapper_source_ordinal`、`mapper_role_source_ordinal`、`first_condition_source_ordinal`、
  `first_condition_role_source_ordinal`、`second_condition_source_ordinal`、
  `second_condition_role_source_ordinal`は各`usize`。

tableは`get`、`iter`、`len`、`is_empty`、handoffは`source_id`、`module_id`、`compositions`、
`debug_text`をexposeする。F5のdebug outputは
`source-template-fraenkel-structural-composition-v1|module=<module>|compositions=1|uses=3`で固定する。

## Structural validationとF5 oracle

validationはdeterministicかつfail-closedである。共通source/module environment、template
association、generator binding、各generator use、完了したcompositionの順にrevalidateする。
producerはprecomputed IDをtrustせずresolved-to-typed associationをuniqueにscanする。normal
recovery、exact node kind、exact range anchor/equality/containment、exact direct AST edge、exact resolver
provenanceを要求する。R2 bindingごとにsource orderでdefinition block/segmentを`TypedAst`へuniqueにmapし、
single direct normal `segment -> TypeExpression -> TypeHead` chainをfollowし、reached typed nodeと
`definition_block()`/`type_head()`が等しいtemplate associationをexactに1件matchする。equal-length tableを
zipしない。0件/複数matchまたはone associationの再利用はそのR2 candidateの`InvalidComposition`、
全R2 binding後の最小未消費associationは`UnmatchedTemplateAssociation`を返す。empty/emptyはvalidで、
それ以外のorphan/multiple relationはatomicにrejectする。candidate compositionはR2 generator-binding source
orderのdeterministic dense IDを持つ。template association IDとR2 binding/use ID/ordinalは不変で、producerは
spelling/range inference/reorderingを行わず、error後にpartial handoffを返さない。

F5のsole rowはassociation `0`、template binding `0`、generator binding `0`である。TypedNodeIdは順に
definition/parameter/template binder/type head/template identifier = 53/31/2/39/21、functor
definition/comprehension/segment/generator binder/type expression = 52/49/41/19/40、mapper
owner/reference/identifier = 38/37/17、first condition owner/reference/identifier = 48/42/24、second
condition owner/reference/identifier = 48/44/26である。ordinalはmapper source/role = 0/0、first
condition = 1/0、second condition = 2/1。exact direct-edge chainはdefinition block → template
parameterとfunctor definition、template parameter → template binder、functor definition →
`TermDefiniens#51` → `TermExpression#50` → comprehension、comprehension → mapper owner/
segment/condition owner、segment → generator binder/type expression、type expression → type head →
template identifier、mapper owner → term reference → identifier、condition owner
`FormulaExpression#48` → `PrefixFormula#47` → `BuiltinPredicateApplication#46` → respective
`TermExpression#43/#45` → term reference `#42/#44` → identifier `#24/#26`である。edge kind/
range/recovery/resolver provenanceはexactにcheckし、textからreconstructしない。

## Frozen implementation/test scope

prerequisite commitとfresh preflight後に変更できるRust pathはexactに三つである。

1. `crates/mizar-checker/src/source_template_type_parameter_association.rs`;
2. `crates/mizar-test/src/runner/tests.rs`; および
3. new private `crates/mizar-test/src/runner/tests/type_elaboration/template_fraenkel_structural_composition.rs`。

test matrixは四つのpublic-facing functionと一つのprivate fixture leafである。

1. `task277c_composes_exact_template_fraenkel_structural_handoff`;
2. `task277c_rejects_environment_missing_and_ambiguous_resolved_nodes`;
3. `task277c_rejects_recovery_kind_range_edge_and_provenance_corruption`;
4. `task277c_rebuilds_deterministically_without_mutating_typed_ast`（empty/empty、orphan R2
   binding、zero/multiple structural match、reused association、association-side orphanも含む）; および
5. `task277c_real_fixture_builds_exact_template_fraenkel_structural_composition`。

checker raw test listは`542 -> 546`、mizar-test raw listは`611 -> 612`で固定する。fixture、sidecar、
expectation、trace、coverage、Cargo、production runner、lint-policy sourceは変更しない。

## Baselineとprotected evidence

checker production inventoryは32 regular paths / 189180 lines、path SHA-256
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`、content SHA-256
`560c15585dd85de320c42c15668657cf3d03a967dfe677ea03be33a0ae905861`である。narrower Rust-only
subinventoryは30 paths / 189124 lines。owner moduleは1224 lines / SHA-256
`7ff46174cf7818722ea8acf6a2a55be77659ce821d68c531b583134ac12f8018`、mizar-test registrationは
64 lines / SHA-256 `8ae81a6ca4dadd9a58165f09bdde4d2ad3cdcd0884ad7521fe5d1ea90539b316`である。
protected checker lint policyは1955 lines / SHA-256
`f8c0c2c196e476b744716d51d8252a61f667536ef97a441246519b3b1a6dd2a0`。completeした277B-L/277R2
private leafはそれぞれ249 / 106 lines、SHA-256
`5fb342d357fb8cb92bd88278c019b276741cd1d6edb255e16e4f231f578dfe04` /
`69b54a4effcb7a740d6588070e6951e3a772cd1818ef9fedcb36426642bf3bf4`である。mizar-test production
inventoryは38 paths / 80090 lines、path SHA-256
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa`、content SHA-256
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`のまま。

protected authorityは64 English specs
(`d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` /
`b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` path/content)、343 `.miz`
(`d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` /
`54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb`)、435 expectations
(`22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` /
`b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea`)、21 Cargo files
(`d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` /
`146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca`)である。F5
fixture/expectation/trace hashはそれぞれ
`32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`、
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`、
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`である。
coverage auditはSHA-256
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`でimmutableに保つ。raw checker
testは542 / SHA-256 `e2b0e67d6066c7157b491e4c57c1f61200dc9339d0b03592af13b551ebfa4410`、raw
mizar-test testは611 / SHA-256 `6eaaca04215420028c57731bc14144e2b73ca719dc6cc35f64a5a421e2a1c426`である。

frozen CLI hashはplan `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`、parse-only
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`、declaration-symbol
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`、type-elaboration
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`、proof-verification
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`である。

## Documentation、gates、handoff

本prerequisiteは本EN/JA contract pairとplan Task Indexが列挙するpaired checker/mizar-test owner
recordだけ、exact 24 Markdown pathを変更しledgerは変更しない。task-contract treeは86/86から87/87へ増える。
completion surfaceはexact 20 owner/contract pathであり、four plan rowsはprerequisite-only indexである。
`spec_coverage_audit.md`へのimpactはなくunchangedとする。

implementation completionではauthority/scope、dependency readiness、ABI、structural invariant、test
sufficiency、implementation/equivalence、bilingual/source-documentation consistency、protected
baseline/count/hash、final independent quality review（hard-gate failureなし、90/100以上）の9 gateをpassする。
docs-only prerequisiteでは追加でexact-24 scope、`git diff --check`、recursive task-contract link validation、
checker/mizar-test lint-policy testを要求する。Sol xhighがauthority/integration/final review/stage/commitを
担当し、Terra highはbounded inventory/review route、Lunaはunavailableでeffective routingを記録する。
`doc/design/spec_coverage_audit.md`またはlegacy-compaction/ledger deltaはauthorizeしない。

**Next handoff:** separate docs-only prerequisite commit後、fresh HEAD/worktree/origin/stashと
count/hash/CLI preflightを通してから、上記のfrozen three Rust pathsとfive testsだけをimplementする。authority/
acceptanceはSol xhigh、bounded cross-module implementation/disagreementはTerra highを用い、authority矛盾、
public dependency issue、scope expansionでは停止する。
