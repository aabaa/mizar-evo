# Task CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8: 正規化nested Fraenkel capture graph

> 正本言語は英語。canonical English:
> [../en/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md](../en/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md)。

Owner planは[mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。Durable ownerはchecker
[source formula composition](../../mizar-checker/ja/source_formula_composition.md#task-257c4c8-normalized-multi-capture-graph)とprivate test
[harness](../../mizar-test/ja/harness.md#checker-task-257c4c8-private-normalized-capture-graph-probe)。

## Status、決定、目的

**Status:** frozen documentation prerequisite。Implementation未着手。

Clean resolver commit `a710b4f1d99fd2efea36aecf9c2b00cf81437c57`後の独立inventoryでは
owner/boundaryは一意だったがnormalized representationが2案残った。Userがparent推奨を採用し、
checker Task-257C `source_formula_composition` sole owner、standalone immutable/
syntax-free/Core-ID-free destination、5 dense table `3 generators / 1 mapper /
0 predicates / 2 distinct captures / 2 occurrences`、local `z` noncapture、retained
C4C8R resolver snapshot、private declaration/source order、dependency→cardinality→layout→
provenance→capture identity→occurrence precedenceを人間決定としてfreezeした。

これでchecker `design_drift`をcloseする。Missing checker/private-fixture regressionsは
`test_gap`、未実装は本contractに対する`source_drift`。`spec_gap`/`repo_metadata_conflict`はない。
Capture semantics、C4C4 captured mutation、second AST slot、Core identity/origin、active route、
diagnostic、Task277B creditは`boundary_violation`。

## Authorityとexact input

AuthorityはChapter 4 §4.6、Chapter 13 §§13.4.3--13.4.4/13.8.6、existing exact
two-capture `.miz`、unchanged inactive expectation/trace、completed C4C5/C4C6/C4C7/C4C8R、
derived docs/sourceの順。

C4C8R inputはbinding `0/1/2 = z/x/y`、segment/binder ranges
`110..129/110..111`、`144..163/144..145`、`165..184/165..166`。Mapper uses
`0/1`は`x@98..99 -> 1`、`y@101..102 -> 2`、role/ordinal `Mapper 0/0, 1/1`。
Mapper owner `97..103`、inner/outer comprehension `95..131`/`93..186`はfrozen frontend
preflightで確認済みだが、resolver collectionはこれらownerのauthenticated resolved-node
identityだけを公開してrangeを公開しない。本graphはunavailable owner rangeをrepeat/
reconstructしない。Display spellingはdependency authenticationだけでjoin keyではない。
Resolver/checker graph/checker binding/Coreのnumeric IDを相互reinterpretしない。

## Frozen public API

Existing `dense_id!`でexact 5 IDsを追加する:

```rust
SourceNestedFraenkelCaptureGraphGeneratorId
SourceNestedFraenkelCaptureGraphMapperId
SourceNestedFraenkelCaptureGraphPredicateId
SourceNestedFraenkelCaptureGraphCaptureId
SourceNestedFraenkelCaptureGraphOccurrenceId
```

Private storageとexisting dense-ID derives、`new`/`index`だけを公開する。Exact 5 immutable
row/table pairsは`SourceNestedFraenkelCaptureGraph{Generator,Mapper,Predicate,Capture,Occurrence}`
と各`Table`。Table APIはdense `get`、source-ordered `iter`、`len`、`is_empty`のみ。
Private source/declaration/role ordinalはvalidateするがgetterを公開しない。

Generator getterはresolver binding、definition/functor/comprehension/segment/binder
`ResolvedNodeId`、segment/binder `SourceRange`のみ。Mapper/Predicate getterはdefinition/
functor/comprehension/owner `ResolvedNodeId`のみ。Predicate tableはexact emptyで、future
predicate semanticsを作らない。

Capture getterはgenerator graph ID、resolver binding ID、mapper graph ID、inner
comprehension owner context。Occurrence getterはmapper/capture graph ID、resolver use index/
binding、comprehension/role-owner/term-reference/identifier node、existing role、identifier
rangeだけ。

Exact familyは:

```rust
SourceNestedFraenkelCaptureGraphHandoff
#[non_exhaustive] SourceNestedFraenkelCaptureGraphError
SourceNestedFraenkelCaptureGraphProducer
```

Handoff getterはsource/module/resolver summaryと5 table、`debug_text()`だけ。Private
version/domain-tagged resolver cloneをretainする。Sole producerは
`build(&FraenkelGeneratorVariableSourceCollection) -> Result<Handoff, Error>`。Raw/unchecked
constructor、mutable getter、DTO/profile selector、Default、conversion、adapter、installer、
AST/Core routeは禁止。Crate-private complete validatorだけをlater checker consumer用に許可する。

Debug grammarは
`source-nested-fraenkel-capture-graph-v1|module=<package>.<path>|generators=3|mappers=1|predicates=0|captures=2|occurrences=2`。

## Exact graphとdefault-deny

Generator `0/1/2`はresolver `z/x/y`、mapper `0`は2 use共通inner owner、predicate empty、
capture `0/1`はgenerator `1/2`・resolver binding `1/2`・mapper `0`・inner owner、occurrence
`0/1`はresolver use `0/1`からcapture `0/1`へexact associationする。Graph IDはresolver IDの
numeric reinterpretationで作らず、resolver identity rowを選択後に別domainとして記録する。

Errorとdisplayはexact:

- `InvalidDependency`: `nested Fraenkel capture graph dependency is invalid`
- `InvalidCardinality`: `nested Fraenkel capture graph cardinality is invalid`
- `InvalidLayout`: `nested Fraenkel capture graph layout is invalid`
- `InvalidProvenance`: `nested Fraenkel capture graph provenance is invalid`
- `InvalidCaptureIdentity { capture }`: `nested Fraenkel capture graph identity <id> is invalid`
- `InvalidOccurrence { occurrence }`: `nested Fraenkel capture graph occurrence <id> is invalid`

Validationはdependency/environment/version/domain/summary/exact C4C8R、`3/1/0/2/2`
cardinality、dense/private order、all provenance、lowest invalid capture、lowest invalid
occurrenceの順。Provenance rangeはdependencyが公開するgenerator segment/binderとoccurrence
identifierだけを再検証し、全ownerはresolved node identityで再検証する。Missing/extra/
duplicate/reordered/stale/foreign/recovered/partial/mismatch/
numeric substitution/display-name joinはatomic fail。Sort/repair/inference/merge/unchecked
dedup/mutation/partial publishは禁止。

## Tests、scope、baseline

Implementation Rust scopeはexact 2 paths: checker `source_formula_composition.rs`とexisting
private `fraenkel_nested_capture_identity.rs`。Checker exact 4 tests:

1. `task257c4c8_builds_exact_normalized_capture_graph`;
2. `task257c4c8_rejects_dependency_cardinality_layout_and_provenance`;
3. `task257c4c8_rejects_capture_identity_and_occurrence_in_precedence`;
4. `task257c4c8_replays_immutably_and_rejects_near_miss_profiles`。

Private runner testはexact
`task257c4c8_real_imported_fixture_builds_exact_normalized_capture_graph`。Unchanged C4C7
sourceをfrontend/resolverへ通し、diagnostic/recovery 0、`3/1/0/2/2`、identity link、private
iteration、`z` exclusion、replay、empty import augmentationだけをassertしactive dispatchにしない。

Docs prerequisiteはexact 21 paths: contract pair、checker plan/source-formula/TODO/source-spec/
bilingual pair、mizar-test plan/harness/TODO/bilingual pair、central coverage audit。Completionは
exact 15 paths: Rust 2、contract pair、checker source-formula/TODO/source-spec/module-boundary pair、
mizar-test TODO pair、central audit。Auditはzero-credit 1 mapping、Chapter 13 `partial`、trace/
expectation/diagnostic/route counts不変。

Baseline HEAD `a710b4f1d99fd2efea36aecf9c2b00cf81437c57`、origin
`ffc882675141a3e25bc78a47affc018bfe3685e1`、divergence `0/8`、stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。Source baselinesはchecker
`10342/404395` SHA `dd27218581ebe6c252da33f6feb23480403afa88858de874970d88a9d1573d44`、
private leaf `704/27913` SHA `6a1717fec263e79d9295813b413d1ec323c3291297f9ee04e0bc7c8e59e2e754`。
Library testsはchecker `572->576`、mizar-test `624->625`、baseline raw hashes
`ac213696433d40a0649c3f6ca4eb7449ce7d053a40a7573209ef5c0af9716940`/
`21196d1cb959c5b6bd7b38f19efb83d334978ec7f1d0c99e35da19cec8afe385`。
Contract tree `104/104->105/105`。Checker productionは32 paths、path hash
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`。

C4C7 source/sidecar/trace hashesは
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`、
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`、
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`でprotected。
`source_term.rs`、Typed/Resolved、C4C4 captured、Core、
GeneratedOrigin、diagnostic、active route、Task277Bもprotected。

## Core boundary、review、exit

Core 33がlater context/item/binder identity/provenance、fresh snapshot-local `CoreVarId`と本graph
associationをownする。Core 34はtype/evidence/coercion/view。Core 35はTask33 associationを
consumeし、Task34後のterm/formula/Fraenkel GeneratedOrigin loweringをownしてallocate/infer
しない。Generator domain operandはcapture param/arg subvectorと分離し、後者だけがgraph
private orderでpositional join可能。本taskはCore input/output/ID/param/arg/functor/origin/useを
作らない。

Pre-source spec/equivalence・bilingual/boundary/API、post-source test-sufficiency・implementation・
source/docs/API・final-qualityをindependent reviewし、修正後finding-specific re-review。
Exact 5 tests、C4C2--C4C8R/C4C5/C4C6 compatibility、checker/mizar-test lib/lint/metadata、fmt、
offline metadata、workspace all-target/all-feature warnings-denied Clippy/full tests、diff/count/
hash/protected、exact staging/commitsを要求。Exitは9/9、valid 90/100以上、clean postcommit、
fresh successor inventory。Actual capture semantics、Typed/Resolved install、Core33--35、
GeneratedOrigin、active execution、Task277Bはdefer。
