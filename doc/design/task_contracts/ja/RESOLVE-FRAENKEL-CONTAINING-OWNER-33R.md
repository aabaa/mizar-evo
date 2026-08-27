# Task RESOLVE-FRAENKEL-CONTAINING-OWNER-33R: exact containing-functor owner receipt

> 正本言語は英語。canonical English:
> [../en/RESOLVE-FRAENKEL-CONTAINING-OWNER-33R.md](../en/RESOLVE-FRAENKEL-CONTAINING-OWNER-33R.md)。

Owner planは[mizar-resolve](../../mizar-resolve/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。Durable ownerはresolver
[names](../../mizar-resolve/ja/names.md#resolver-task-33r-exact-surface-fingerprint)、
[symbols](../../mizar-resolve/ja/symbols.md#resolver-task-33r-exact-containing-functor-owner-receipt)、
mizar-test [harness](../../mizar-test/ja/harness.md#resolver-task-33r-private-containing-owner-probe)。

## Status、decision、readiness

**Status:** precommit implementation complete、exact staging・task-only commit pending。

Completed C4C8とdocs-only Core-33Pのdependency-minimal zero-semantic successor。
Fresh inventoryにより、validated declaration shell、その`ResolvedNodeId`、final
`SymbolId`/`DefinitionId`、`SourceContributionId`、`SemanticOrigin`を同時にownするのは
resolver symbol collectionだけと判明した。Checker、Typed/Resolved、Coreはname、range、
numeric idからこのrelationを再生成できない。

従来authorityが未選択だったcardinality/APIについて、userがreviewed recommendationを採用した
decision authorityにより、exact C4C8 profileはcontaining functor owner exact 1件、resolverは
existing symbol/definition allocation boundaryでopaque immutable receipt 1件を公開し、全mismatchを
fail closedと固定する。General Fraenkel semantics、active route、Core destinationは選択しない。
Missing linkは`design_drift`、freshness/association/corruption/real-fixture coverage欠落は
`test_gap`。Blocking `spec_gap`/`repo_metadata_conflict`はない。

## Authority、dependency、fixed meaning

Authority順は`doc/spec/en/`、exact existing C4C7 `.miz`、trace、unchanged expectation、
`doc/design/`、最後にnon-normative source。DependencyはC4C5 `72662d38`、C4C7
`3d28af5f`、C4C8R `a710b4f1`、C4C8/closure `c7595b60`/`c5792708`、Core-33P
`332d752c`。

Inner mapper `x`,`y`はouter generator resolved binding identityを参照し、inner `z`はlocalで
captureしない。Associationはdisplay spellingを使わない。C4C4はby-value/empty `captured`の
まま。Resolver/checker/Core IDは別domain。C4C5はseparate one-capture checker receiptであり、
C4C8 containing functor ownerへ推論接続しない。

## Frozen public APIとownership

`FraenkelGeneratorVariableSourceCollection`へprivate exact
`surface_fingerprint: String`を追加し、collectorが使ったcomplete deterministic
`SurfaceAst::snapshot_text()`だけを格納する。Immutable
`surface_fingerprint(&self) -> &str`を公開するが、existing constructor path/table/order/id/
`debug_text()` grammarは不変。Equalityはfingerprintを含み、same count summaryはstale
same-`SourceId` snapshotをauthenticateしない。

Existing `symbols` moduleがexact public items
`SourceNestedFraenkelFunctorOwnerHandoff`、non-exhaustive
`SourceNestedFraenkelFunctorOwnerError`、
`SourceNestedFraenkelFunctorOwnerProducer`をownする。Producerのentry pointはEN contract記載の
`build(ast, module, resolved, resolver)`だけ。

Handoffは`#[derive(Clone, PartialEq, Eq)]`、field/private construction、builder/mutator/default/
uncheckedなし。Exact getter/validator signatureとreturn typeはcanonical EN code blockがsole owner。
`source_id: SourceId`、`module_id: &ModuleId`、`surface_fingerprint: &str`、
`definition_block`/`functor_definition: ResolvedNodeId`、`declaration_shell: DeclarationShellId`、
`symbol: &SymbolId`、`definition: DefinitionId`、`contribution: SourceContributionId`、
`origin: &SemanticOrigin`、`debug_text: String`を返す。Public `validate_complete()`と
`validate_resolver_collection(...)`は同じerror型を返し、後者はcomplete oracle後にretained
collectionとのexact equalityを要求する。

Private dependencyはversion/domain、cloned surface AST、structural resolved arena、exact
resolver collection、declaration shells、parser-backed projections、symbol collection resultを
retainする。Producerがすべてinternal deriveし、callerはshell/projection/env/ID/originを供給・
forgeできない。

Allocation associationはfinished envから再構成せずin-flight captureする。Existing public
`SymbolCollector::collect()`はtargetなしprivate collection pathへdelegateしbehavior不変。
Producerはauthenticated `DeclarationShellId`をtargetとして同pathを呼び、same rowの
`CollectedProjection::new` symbol/origin/contributionとinsert時に返る`DefinitionId`をprivate
allocation rowへ記録する。Complete oracleはsame targeted collectionをrerunし、allocation rowと
final indexを比較する。Post-hoc name/range searchによるassociation作成は禁止。

## Frozen associationとdefault-deny oracle

Validation precedenceは次の通り。

1. `InvalidDependency`: version/domain、source/module、complete arena、exact surface
   fingerprint、fresh resolver recollection、retained dependency mismatch。
2. `InvalidResolverProfile`: exact C4C8R `3` bindings/`2` mapper uses、dense order、local
   inner binding 0、outer binding 1/2 capture、one mapper owner、common definition/functor以外。
3. `InvalidOwnerCardinality`: matching functor declaration shell/projectionがexact 1件でない。
4. `InvalidOwnerProvenance`: parent definition block、kind、module、recovery、resolved
   shell-to-node relation mismatch。
5. `InvalidSymbolAssociation`: diagnostic、symbol missing/duplicate、non-functor、wrong
   contribution/origin、recovery/conflict、canonical `CollectedProjection::new`以外。
6. `InvalidDefinitionAssociation`: definition missing/duplicate、wrong functor kind/symbol/
   contribution/origin/conflict。
7. `InvalidAssociation`: exposed handoff rowとfresh derived rowのmismatch。

7 variantsはすべてfieldless。Errorは`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`、
`#[non_exhaustive]`、`Display`/`Error`実装。Exact English display string 7件はcanonical EN
contractの同順code blockがsole owner。Producerは`#[derive(Debug, Clone, Copy)]`。

Shell nodeは`resolved_node_for(shell.node_id())`だけで変換しnumeric reinterpretを禁止。
Final symbol relationはexisting `CollectedProjection::new` allocation pathだけを使い、spelling/
range lookupを禁止。Display spelling/token textはexact public surface fingerprint内のretentionまたは
debug renderingには存在できるが、association/join/owner-admission keyには使わない。

Missing/extra/duplicate/reordered/stale/foreign-source/foreign-module/recovered/partial/
mismatched/orphan/corrupt rowはrejectする。Repair sort、owner inference、display-name join、ID
reinterpret、partial recovery、unchecked admissionは禁止。

## Exact scope、tests、protected surface

Exact 23 pathsはpaired contract、paired resolve/test plans、paired names/symbols/harness、central
coverage audit、`names.rs`/tests、`symbols.rs`/tests、resolver lint、existing private
`fraenkel_nested_capture_identity.rs`、paired resolver
`bilingual_documentation_synchronization.md` audit、paired mizar-test
`bilingual_sync_audit.md` audit。Resolver exact tests 7件:

- `task33r_surface_fingerprint_binds_exact_ast_snapshot`;
- `task33r_surface_fingerprint_distinguishes_stale_same_source_ast`;
- `task33r_builds_exact_containing_functor_owner`;
- `task33r_rejects_dependency_and_resolver_profile_mismatch`;
- `task33r_rejects_owner_cardinality_and_provenance`;
- `task33r_rejects_symbol_definition_and_retained_association_corruption`;
- `task33r_enforces_default_deny_precedence_and_replay`。

Private mizar-testはexact
`task33r_real_fixture_links_capture_graph_to_exact_functor_owner`を追加し、unchanged real C4C7から
resolver/C4C8 graph/owner receiptを構築、exact resolver equality、common resolved ownerからfinal
functor symbol/definitionへのlink、graph/import augmentation不変だけをassertする。Unregistered
library testのまま。

Checker/Core source、Typed/Resolved field、installer、active runner、diagnostic、Cargo、
`doc/spec`、existing `.miz`/expectation/traceは変更しない。C4C4 captured、generated parameter/
argument、`GeneratedOrigin`、semantic result、coverage credit、Task277B readinessは作らない。

## Baseline、expected impact、exit

Entry HEAD/originは両方`332d752c03292a1a100472322ce86e99080ce1bd`、divergence `0/0`、
worktree/index clean、stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`。Contract tree
`106/106 -> 107/107`。Rust baseline line/byte/hashはcanonical EN tableがsole owner。
Resolver lib `164 -> 171`、mizar-test lib `625 -> 626`。Baseline list hashは
`a01c16a16aead9868d30257e358a4e742dd7633a8da4f61c864d9197d9c1f1c8`/
`602a80e3a0ad30084154d2f857bd00251494ad40a79549aca0a76db9b9cde711`。

C4C7 source/expectation/trace protected hashは
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`、
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`、
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`。

Rust edit前にindependent spec/equivalence、bilingual/boundary、実装後にtest-sufficiency、
implementation、source/docs/API、final-quality reviewを**NO FINDINGS**まで行う。Focused tests、
resolver/mizar-test lib/lint、metadata、parser comprehension、fmt、offline metadata、workspace
warnings-denied Clippy/full tests、diff/scope/count/hash/link/protected checksを実行する。

Exitはhard gate `9/9`、90/100以上、exact task-only commit、clean postcommit、fresh inventory。
次候補はunchanged C4C8 graphと本resolver owner receiptをpairするchecker-owned opaque receiptだが、
fresh inventoryでassociation/oracleが一意な場合だけfreezeする。Core33 installation、Core34/35、
parameter order、GeneratedOrigin、actual semantics、active route、Task277Bはdeferする。

## Precommit implementation completion evidence

Implementationはfrozen 23 pathsだけを変更した。Paired contract treeは`107/107`、resolver/
mizar-test library inventoryはexact `171`/`626`、final sorted raw-list SHA-256は
`1e4b48bf53e4ad6ead624ac40d6fe8e6aeef90166c77fd4974b9c849c955d5ba` /
`e54d5c97f46e65d4657d5e99b7efa609cd39a096020c4339b512fdbf039b0694`。Final Rust
line/byte/hash tableはcanonical EN sectionがsole exact ownerで、本companionは同じ6 pathsと測定を
論理的に同期する。

Pre-source specification/equivalence・bilingual/boundary review、post-source
test-sufficiency・implementation・source/documentation/API・bilingual/boundary reviewは、
finding-specific repair後すべて**NO FINDINGS**。Independent final-quality reviewも**NO
FINDINGS**、hard gate `9/9` PASS、score capなしのvalid `100/100`
（`20/20`, `20/20`, `15/15`, `15/15`, `10/10`, `10/10`, `5/5`, `5/5`）。

Focused resolver 7件、private real-fixture 1件、C4C8R `4/4`、checker C4C8 `4/4`、runner
C4C8 `2/2`、resolver `171/171`・lint `11/11`、mizar-test `626/626`・lint `15/15`・
metadata `137/137`、parser set-comprehension、format、offline Cargo metadata、workspace
all-target/all-feature warnings-denied Clippy、full all-feature workspace test/doctest、diff、
scope/count/hash/link checkはすべてPASS。

C4C7 source/expectation/traceはfrozen 3 hashを再現する。Spec、existing corpus/expectation/
trace、checker/Core、Typed/Resolved、active route、diagnostic、C4C4 capture、semantic/coverage
stateは不変。Task277Bはnot-ready/zero-credit。Exact staging、task-only commit、clean postcommit、
fresh successor inventoryだけがexit operationとして残る。
