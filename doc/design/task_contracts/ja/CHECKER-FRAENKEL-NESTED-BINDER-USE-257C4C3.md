# Task CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3: nested Fraenkel binder/mapper-use transport

> canonical English:
> [../en/CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md](../en/CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md)。
> 本書はlogical synchronized Japanese companionである。

Owning planは[mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。durable ownerはchecker
[source formula composition](../../mizar-checker/ja/source_formula_composition.md#task-257c4c3-nested-fraenkel-bindermapper-use-transport)とtest
[harness](../../mizar-test/ja/harness.md#checker-task-257c4c3-private-nested-binderuse-probe)。

## status、purpose、readiness

**Status:** frozen / dependency-ready。implementation未開始。

clean `HEAD e5ffc6bc036ed5d7ba3c173e23671f1c4511ba6a`のfresh read-only inventoryと
人間のowner判断により、completed C4C2の最小successorをTask257C / existing
`source_formula_composition` ownerへ固定する。本taskはC4C2のsole inner mapper useをdistinct
outer generator binderへmapするimmutable checker handoff 1件だけを作り、解釈/installしない。

Chapter 13とexisting `.miz`/inactive sidecarはexact resolved identityを固定するため`spec_gap`はない。
missing checker handoffは`source_drift`、missing exact checker/private imported-fixture testsは
`test_gap`、以前のfirst checker owner未決は人間判断で解消された`design_drift`。exact-F5 C4A/C4B
reuse、Task252 occurrence追加/複製、capture/semantic result publishは`boundary_violation`。

## authority、依存、protected artifact

authority orderはcanonical Chapter 4 §4.6 / Chapter 13 §§13.4.2, 13.4.4, 13.8.6、existing
`pass_types_nested_comprehension_outer_generator_capture_001.miz`、sole trace row、inactive
expectation、completed C4C0/C4C1/C4C2、derived design/source inventory。source/sidecar/traceの
SHA-256はそれぞれ
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`、
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`、
`d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`のまま。
sidecarはinactive `advanced_semantics` / `pass/type_check` / diagnostic・active tagなし、traceは
test intentだけでexecution/semantic credit zero。

依存はC4C2 implementation/closeout
`601db2ab8fbcfa736d4b619e0eacbbf1291cc800` / `e5ffc6bc036ed5d7ba3c173e23671f1c4511ba6a`、
exact imported admission、C4C2 resolver collection、normal one-to-one Resolved→Typed projection。
C4A/C4Bはnegative compatibility profileでdependencyではない。

## selected splitと禁止代替

Complete resolver collection/`TypedAst`のprivate cloneをretainし、environmentとexact nested profileを
authenticateした後、resolver use/binding identityとtyped node site 2件だけをrow 1件でpublishする
dependency-minimal / zero-semantic / default-deny splitを選ぶ。Task252 term/reference先行、C4A/C4B
extension、`BindingEnv`/`BindingId`/`CapturedFreeVariables` construction、Typed/Resolved install、runner
routeはいずれも現oracleを越えるため禁止する。

## frozen public API / ownership

Sole production ownerは`crates/mizar-checker/src/source_formula_composition.rs`。exact familyは
`SourceNestedFraenkelBinderUseId`、`SourceNestedFraenkelBinderUse`、
`SourceNestedFraenkelBinderUseTable`、`SourceNestedFraenkelBinderUseHandoff`、
`#[non_exhaustive] SourceNestedFraenkelBinderUseError`、
`SourceNestedFraenkelBinderUseProducer`。

IDは`new(index)`/`index()`だけ。row getterはexact
`resolver_use_index() -> usize`、
`resolver_binding() -> FraenkelGeneratorVariableBindingId`、
`outer_binder() -> TypedNodeId`、`inner_mapper_use() -> TypedNodeId`、
`source_ordinal() -> usize`。tableは`get`、dense `iter`、`len`、`is_empty`。

Handoff getterは`source_id()`、`module_id()`、`resolver_summary()`、`binder_uses()`、
`debug_text()`。producerはexact
`build(&FraenkelGeneratorVariableSourceCollection, &TypedAst) -> Result<..., ...>`。
public dependency getter/mutation/unchecked constructor/role enum/capture flag/semantic value/install APIなし。
sole planned consumerはexisting private mizar-test
`fraenkel_nested_capture_identity.rs` regression 1件で、future production/semantic consumerはseparate contract必須。

`resolver_summary()`はexact non-authoritative
`fraenkel-generator-variable-source-v1|module=<package>.<path>|bindings=2|uses=1`で、authorityはretained
resolver snapshot側にある。Exact debug grammarは
`source-nested-fraenkel-binder-use-v1|module=<package>.<path>|binder-uses=1`。

## exact row、validation、default deny

Exact rowはID0、resolver use 0、resolver binding 1、outer binder typed node `x@136..137`、inner
mapper-use typed node `x@94..95`、source ordinal 0。Retained validationはshared definition/functor、
distinct inner/outer comprehension、両generator segment/binder、inner mapper owner/reference/identifier、
C4C2 order `inner y=0` / `outer x=1`、sole Mapper use0→binding1、ordinal/range、typed child edge、normal
recovery、一意Resolved→Typed mappingをfull authenticateする。spelling/rangeからidentityを推測しない。

Private snapshot version/domainは
`source-nested-fraenkel-binder-use-dependencies-v1` /
`source-nested-fraenkel-binder-use`。Non-exhaustive error enumは次のexact 4 variantだけを持ち、precedenceは
`EnvironmentMismatch`、`InvalidResolverDependency`、`InvalidTypedDependency`、
`InvalidBinderUse { binder_use: SourceNestedFraenkelBinderUseId }`。wrong row countはID0。
missing/extra/reorder/duplicate binding/use、F5、equal binder、alternate type、condition/extra nesting、recovery、
duplicate resolved mapping、detached edge、stale summary/snapshot、wrong site、partial matchはatomic reject。

## implementation/test/count/audit

Rust scopeはchecker ownerとexisting private
`crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`のexact 2 paths。
Checker testは`task257c4c3_builds_exact_nested_binder_use_handoff`、
`task257c4c3_rejects_environment_resolver_and_typed_dependency_corruption`、
`task257c4c3_rejects_row_cardinality_order_and_site_corruption`、
`task257c4c3_replays_deterministically_and_rejects_f5_profiles`の4件。Private leafは
`task257c4c3_real_imported_fixture_builds_checker_identity_handoff` 1件だけを追加し、route/activationなし。

Raw countはchecker `554 -> 558`、mizar-test `619 -> 620`。baseline sorted list hashはchecker
`78f0291fb13aed8a8adbbc5aa1db9df1a7415fc9d8cf35820e1ad9e40aad2ace`、mizar-test
`ad70984d911bd6ef84fc5efa15a50815acc7b4cc7daab1c89235263e022aa00b`。checker ownerは
`7958` lines / `90b339d9707f9f8d847b678721e8db0ef6a00e4a15dbb41474a0cf6980f47168`、
private leafは`169` / `fa70dc53fb92376fcbd71b4058d9830355ab12fc4e5d6f67050d129cb7f46ae9`。
Contract treeは`92/92 -> 93/93`。

Coverage auditはzero-credit design mappingだけupdateする。trace/expectationは不変で、execution、semantic、
sethood、occurrence、capture、request/result、diagnostic、route、Task277B creditは変えない。

## review、verification、exit、handoff

Independent specification/contract、test sufficiency、implementation、source/docs/API、bilingual/boundary、
final quality reviewをmaterial findingごとにNO FINDINGSまで反復する。Focused 5 test、checker/mizar-test
library、両lint、metadata、format、warnings-denied workspace Clippy、full workspace test、unchanged CLI 5 route、
protected hash、scope/diff、postcommit proofを通す。9 hard gate、valid `>=90/100`、task-only commit、
clean/stash-invariant fresh inventoryでexitする。Task277Bはnot-ready/zero-credit。

次はseparate Task252 mapper-primary-occurrence prerequisiteを最初にinventoryするが、authority/oracle/
dependency/sole ownerが一意な場合だけselectする。Type/sethood、semantic capture、generated-core parameter、
request/result、verdict、diagnostic、production install、runner activation、coverage creditはdeferred。
Authority/API/boundary/final scoringはSol `xhigh`、frozen implementation/reviewはTerra `high`/`xhigh`。

## frozen documentation prerequisite checkpoint

Independent specification/API reviewはmissing exact debug grammar/error variant set、independent
bilingual/boundary reviewはduplicate audit addendum/premature implementation tenseを初回findingとした。
修正後のfinding-specific re-reviewはいずれも**NO FINDINGS**。`git diff --check`、checker/mizar-test
lint-policy、metadata `137/137`はPASS。Implementationはpendingで、本docs prerequisite commitとclean
fresh inventoryがfrozen dependencyを確認するまで開始しない。
