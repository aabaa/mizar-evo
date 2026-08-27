# Task CHECKER-FRAENKEL-CAPTURE-GRAPH-OWNER-33C: opaque graph-owner receipt

> 正本言語は英語。canonical English:
> [../en/CHECKER-FRAENKEL-CAPTURE-GRAPH-OWNER-33C.md](../en/CHECKER-FRAENKEL-CAPTURE-GRAPH-OWNER-33C.md)。

Owner planは[mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。Durable ownerはchecker
[source formula composition](../../mizar-checker/ja/source_formula_composition.md#task-33c-opaque-capture-graph-owner-receipt)と
test [harness](../../mizar-test/ja/harness.md#checker-task-33c-private-graph-owner-probe)。

## Status、decision、readiness

**Status:** implementation、全review、broad verification、protected check、final-quality
complete。Staging/commit pending。

Completed C4C8/Task33Rのdependency-minimal zero-semantic successor。Fresh read-only
inventoryにより、existing checker `source_formula_composition` ownerがunchanged C4C8 graphと
resolver Task33R owner receiptをopaque scalar receipt 1件としてretainする構成だけが残る。
Typed/Resolved/Coreをownerにするとinstallation/semantic destinationを早期選択するため禁止。

従来authorityがexact composite API/oracleを未選択だったため、userがreviewed recommendationを
採用したdecision authorityにより、checker-owned immutable one-to-one receipt、both inputsの
by-value retention、table/dense idなし、common source/moduleとtwo receiptsだけのgetter、全mismatch
fail closedを固定する。Missing compositionは`design_drift`、replay/corruption/freshness/display-
independence/real-fixture不足は`test_gap`。Blocking `spec_gap`/`repo_metadata_conflict`なし。

## Authority、dependencies、fixed meaning

Authority順は`doc/spec/en/`、exact existing
[C4C7 source](../../../../tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.miz)、
canonical [trace](../../../../tests/coverage/spec_trace.toml)、unchanged
[expectation](../../../../tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.expect.toml)、
`doc/design/`、最後にnon-normative source。DependencyはC4C5 `72662d38`、C4C7
`3d28af5f`、C4C8R `a710b4f1`、C4C8/closure `c7595b60`/`c5792708`、Core-33P
`332d752c`、Task33R `e94f36cf9785b9d1ffe965045b19aa42b89caedc`。

Inner mapper `x`,`y`はouter generator resolved binding identityを参照し、inner `z`はlocalで
captureしない。Associationはdisplay spellingを使わない。C4C4はby-value/empty `captured`。
Resolver/checker/Core numeric idは別domain。C4C5/C4C6 exact-one receipt/Typed-Resolved
installationはseparateのままで、本taskはgeneralize/replaceしない。

## Frozen public APIとownership

Existing checker `source_formula_composition` moduleがexact public items
`SourceNestedFraenkelCaptureGraphOwnerHandoff`、non-exhaustive
`SourceNestedFraenkelCaptureGraphOwnerError`、
`SourceNestedFraenkelCaptureGraphOwnerProducer`をsole-ownする。New id/row/table/installer/
adapter/builder/default/mutator/unchecked constructorはない。

Producerのexact signatureはcanonical EN code blockがsole ownerで、graph/ownerをby valueで
consumeする。Handoffは`#[derive(Clone, PartialEq, Eq)]`、private fields。Public getterはexact
`source_id`、`module_id`、`graph`、`owner`、`debug_text`だけ。Complete validationはpublic
consumer未選択のためcrate-private。Private scalar associationはcommon `SourceId`/`ModuleId`、
definition-block/functor-definition `ResolvedNodeId`だけをretainし、second owner row/Core identityを
公開しない。Exact debug grammarはcanonical EN text blockがsole ownerで、symbol spellingはdebug
only、association/admission keyではない。

## Frozen associationとdefault-deny oracle

Validation precedenceはexact:

1. `InvalidGraphDependency`: C4C8 `validate_complete()`のdependency/row failure。
2. `InvalidOwnerDependency`: Task33R `validate_resolver_collection()`のowner failure、またはgraph
   retained resolverとのexact inequality。
3. `InvalidAssociation`: common source/module、fresh private scalar snapshot、または全graph
   generator/mapper/predicate rowのdefinition-block/functor identityがTask33R ownerと不一致。

Cardinalityはscalar one graph/one owner。New observable orderはなく、C4C8 private orderとexact
`3/1/0/2/2`は不変。Associationはtyped identityを直接比較し、display/range joinやnumeric-id
reinterpretを行わない。

Errorはfieldless `InvalidGraphDependency`、`InvalidOwnerDependency`、`InvalidAssociation`のexact
3 variants、`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`、`#[non_exhaustive]`、`Display`/
`Error`実装。Exact English display 3件と順序はcanonical EN contractがsole owner。Producerは
`#[derive(Debug, Clone, Copy)]`。

Missing/extra/duplicate/reordered/stale/foreign/recovered/partial/mismatched/corrupt/cross-
snapshot inputはatomic reject。Sort/repair/inference/display-name join/range join/numeric-id
reinterpret/partial recovery/unchecked admissionは禁止。

## Exact scope、tests、protected surface

Exact 24 pathsはpaired contract、paired checker plan/source owner/TODO/source-spec/module-
boundary/bilingual、paired mizar-test plan/harness/bilingual、central coverage audit、checker
source/lint、existing private `fraenkel_nested_capture_identity.rs`。Exact path listはcanonical ENが
sole owner。

Checker exact tests 4件:

- `task33c_builds_exact_graph_owner_handoff`;
- `task33c_rejects_graph_and_owner_dependencies_in_precedence`;
- `task33c_rejects_source_module_identity_and_retained_association_corruption`;
- `task33c_replays_immutably_and_rejects_stale_or_display_joined_pairs`。

Private mizar-test exact
`task33c_real_fixture_pairs_capture_graph_with_exact_functor_owner`はunchanged C4C7からresolver、
C4C8 graph、Task33R owner、本receiptを構築し、borrowed identity、common owner ids、immutable
replay、local-inner exclusion、unchanged import augmentation、zero semantic installationをassertする。

Core source、Typed/Resolved field、C4C4 captured、active runner、diagnostic、Cargo、`doc/spec`、
existing `.miz`/expectation/traceは変更しない。Parameter/argument order、`GeneratedOrigin`、
semantic result、active route、coverage credit、Task277B readinessは作らない。

## Baseline、expected impact、exit

Entry HEAD/originは両方`e94f36cf9785b9d1ffe965045b19aa42b89caedc`、divergence `0/0`、
worktree/index clean、stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`。Contract tree
`107/107 -> 108/108`。Rust 3 pathのline/byte/hash、test-list baseline hash、protected C4C7
3 hashはcanonical EN table/paragraphがsole exact owner。

Checker lib `576 -> 580`、mizar-test lib `626 -> 627`、checker public-enum `9 -> 10`。
Central coverage auditはnew derived zero-credit ownerとunchanged follow-up boundaryだけをrecordし、
Chapter 13 `partial`、trace requirement/status/count、fixture、expectation、diagnostic、semantic
coverageは不変。Schema-v2 ledgerも不変。

Rust edit前にindependent spec/equivalence、bilingual/boundary、実装後にtest-sufficiency、
implementation、source/docs/API、final-quality reviewを**NO FINDINGS**まで行う。Focused 5 tests、
C4C8/Task33R compatibility、checker/mizar-test lib/lint/metadata、parser comprehension、fmt、
offline metadata、workspace warnings-denied Clippy/full tests、diff/scope/count/hash/link/protected
checksを実行する。

Exitはhard gate `9/9`、90/100以上、exact task-only commit、clean postcommit、fresh inventory。
Core33 installation、Core34/35 transport、free/generated-parameter order、`GeneratedOrigin`、actual
semantics、active route、Task277Bはseparate authority-gated/deferred。

Next-task handoffはclean Task33C commitからfresh read-only inventoryを開始し、same-milestoneの
dependency-minimal successorをauditすること。Authorityがowner/dependency/scope/public API/
association/cardinality-order/complete fail-closed oracle/protected impactを一意に固定する場合だけ
new taskをfreezeする。Core Task33/35 destination、free/generated-parameter order、
`GeneratedOrigin`、semantics、active routeを推論しない。Parent authority/final hard gateは
GPT-5.6 Sol `xhigh`、contract freeze後のbounded workだけGPT-5.6 Luna `xhigh`、Luna evidenceが
cross-module精度不足の場合だけTerra `high`へescalateする。

## Precommit implementation evidence

Implementationはfrozen exact 24 paths（docs 21 + Rust 3）を変更する。Contract treeは
`108/108`、checker/mizar-test library inventoryはexact `580`/`627` tests。Final raw-list
SHA-256、Rust 3 pathのline/byte/hash tableはcanonical EN sectionがsole exact ownerで、本JA
companionは同じ測定を論理的に同期する。

Focused checker 4件、private real-fixture probe、C4C8/Task33R/parser-comprehension compatibility、
checker/mizar-test lib/lint、mizar-test metadata、format、offline Cargo metadata、workspace
warnings-denied Clippy、full workspace all-feature test/doctest、recursive contract/link lint、
`git diff --check`はPASS。Combined precedence testとstale EN/JA audit tenseをrepair後、
independent test-sufficiency/implementation/source-doc-API/bilingual-boundary reviewは全て
**NO FINDINGS**。Exact 24-path、contract `108/108`、final Rust count/hash、protected C4C7
3 hashもPASS。Sorted 24-path inventory SHA-256はcanonical ENがsole exact owner。
Independent final-quality reviewは**NO FINDINGS**、hard gate `9/9` PASS、score capなしのvalid
uncapped `100/100`。Exact category scoreはcanonical ENに同期する。Exact staging、commit、
postcommit inventoryはpending。
