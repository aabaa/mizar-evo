# Task CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8: 正規化nested Fraenkel capture graph

> 正本言語は英語。canonical English:
> [../en/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md](../en/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md)。

Owner planは[mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。Durable ownerはchecker
[source formula composition](../../mizar-checker/ja/source_formula_composition.md#task-257c4c8-normalized-multi-capture-graph)とprivate test
[harness](../../mizar-test/ja/harness.md#checker-task-257c4c8-private-normalized-capture-graph-probe)。

## Status、決定、目的

**Status:** complete。Implementation commit
`c7595b60e7784728967cfbac9b02522f7290c942`、clean postcommit proof、fresh
successor inventoryはbelowでclosed。このpostimplementation documentation closureは
stale lifecycle wordingだけをrepairする。

Clean resolver commit `a710b4f1d99fd2efea36aecf9c2b00cf81437c57`後の独立inventoryでは
owner/boundaryは一意だったがnormalized representationが2案残った。Userがparent推奨を採用し、
checker Task-257C `source_formula_composition` sole owner、standalone immutable/
syntax-free/Core-ID-free destination、5 dense table `3 generators / 1 mapper /
0 predicates / 2 distinct captures / 2 occurrences`、local `z` noncapture、retained
C4C8R resolver snapshot、private declaration/source order、dependency→cardinality→layout→
provenance→capture identity→occurrence precedenceを人間決定としてfreezeした。

これでchecker `design_drift`、bounded `source_drift`、checker/private-fixture `test_gap`をcloseする。
Implementation/test-sufficiencyのindependent reviewはrepair後**NO FINDINGS**。
Source-documentation/APIとbilingual/boundaryのfinding-specific re-reviewもdocumentation
repair後**NO FINDINGS**で、required broad verificationはPASS。Independent final-quality
reviewも**NO FINDINGS**、全9 hard gates PASS、valid uncapped scoreは`100/100`。
`spec_gap`/`repo_metadata_conflict`はない。
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
occurrenceの順。Provenanceはcapture/occurrence stageより先にoccurrence node/rangeをauthenticateする。
Dependencyが公開するgenerator segment/binderとoccurrence identifier rangeを再検証し、全ownerはresolved node identityで再検証する。
Occurrence stageはremaining associationを検証し、node/rangeをdefensiveに再検証してよい。Missing/extra/
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
Library testsはchecker `572->576`、mizar-test `624->625`、current countsはchecker `576`/mizar-test `625`、baseline raw hashes
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

## Implementation completion evidence

Documentation prerequisite commitは`481a599877803e855307381901b82ae38365ce4a`。
Current source measurementはchecker `12132 / 472546`、SHA-256
`e7242ebf7344b1e89646fefe2dd9e1ad41d40be22b526c872327540ba7abad12`、private mizar-test leaf
`816 / 32987`、SHA-256 `14f1db22b0d4a45cad31db5a1e11f4c28b89e0cab1047b6f8fd4982a8e7d8041`。
Focused checker 4件 + imported-fixture probe 1件はPASS。Checker/mizar-test libraryは
`576/576`と`625/625`、final sorted raw test-list SHA-256は
`20a2a07a078580b3253a0fbcb5ac8387c42df19fe568d1e5a97b3a709a7bdcd3` /
`6679e5558b1a8884baacaa4c0bb1d6c000d7352002b98c5ff33642034f68f49e`。
Checker productionは`32` paths / `199351` lines、unchanged path SHA-256
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`、final
content-manifest SHA-256
`9d41d59f42e05c46b084bb26e75b53091e8a4f59cbd83880cca4a91b46f8e5e0`。
Contract treeは`105/105`、C4C7 protected 3 hashesはfrozen値と一致する。

Implementation、test-sufficiency、source-documentation/API、bilingual/boundaryのindependent
reviewはimplementation repair 2件とexact source-spec inventory/status repair後**NO
FINDINGS**。Compatibility、両package library/lint、metadata、format、offline Cargo
metadata、workspace all-target/all-feature warnings-denied Clippy、full all-feature workspace
tests/doctests、diff/scope/count/hash/protected-surface checkはPASS。追加の`cargo test
--workspace --all-targets --all-features`は通常test targetを失敗なしで完了したが、長時間の
Criterion performance measurementだけを意図的に停止した。これはPASS済みrequired
full-workspace test gateの代替として扱わない。Independent final-quality reviewは**NO
FINDINGS**。全9 autonomous crate exit gatesはPASS、valid uncapped `100/100`（spec
completeness `20/20`、test contract/coverage `20/20`、traceability `15/15`、implementation
correctness `15/15`、design/source sync `10/10`、boundary `10/10`、verification `5/5`、
handoff `5/5`）。Exact 15-path staging/cached-diff reviewはPASSし、task-only
implementation commit `c7595b60e7784728967cfbac9b02522f7290c942`がsource changeをcloseした。

## Postimplementation closureとfresh successor inventory

Implementation commit直後、worktreeはclean、HEADは
`c7595b60e7784728967cfbac9b02522f7290c942`、origin/mainは
`481a599877803e855307381901b82ae38365ce4a`、divergenceは`0/1`。Protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`、C4C7 source/sidecar/traceのfrozen hashは
不変。Implementation commitはexact 15 pathsで、sorted path-list SHA-256は
`23c102b51678e49f0fd749f9689d4e12dbf85d06dc2cfa870c22623e20b2b541`。

Fresh checker/Core/oracle independent inventoryは一意にreadyなsemantic successorを
選択しない。方向は固定されている。Core 33がcontext/item/binder identity、provenance、
snapshot-local `CoreVarId` allocation、durable checker-graph associationをownし、Core 34が
type/evidence/coercion/view lowering、Core 35がTask-33 associationをconsumeしてlater
term/formula/Fraenkel `GeneratedOrigin` loweringをownする。しかしexact Task-33 C4C8
handoff/API、allocator/owner mapping、captured parameter/argument surface/order、
`GeneratedOrigin` key/functor/source corruption oracleは一意でない。Typed/Resolved slotや
numeric IDのreuse、Core 35によるassociationのallocate/inferは引き続き禁止する。

候補は(1) standalone immutable C4C8 association seamをreserveするgeneric Core-33
context/item/binder base contractを先にfreeze、(2) C4C8-specific private Core-33 associationを
直接freeze、(3) public `CoreContextInput`/`CoreContext` surfaceをextend、の3案。(1)は
prerequisite allocator/owner mapが未実装でzero-semantic/default-deny boundaryを保持するため
推奨する。(2)/(3)はseparate explicit API decisionを要する。Userはcurrent continuationで
候補(1)をacceptし、documentation-only Core-33 prerequisiteの方向だけをselectした。Exact
task identity/API/files/oracleはfresh inventoryとseparately frozen contractに従う。本closureは
successor task ID、API、field、adapter、installer、route、semantic、creditを作らない。これは
`design_drift`とfuture source-derived `test_gap`であり、semantic parameter orderをinventする
権限ではない。Task277Bはnot ready/zero creditのまま。

Paired checker plan、mizar-test harness、TODO recordのstale precommit wordingは
`design_drift`。本contract pairとその4 EN/JA owner/TODO pairsだけがexact 10-document
closure scope。Specification、test intent、traceability、expectation、source、public API、
route、diagnostic、protected hash、coverage creditは変更しない。

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
hash/protected、exact staging/commitsを要求。Implementation hard gateは全`9/9`、valid
uncapped `100/100`でPASSし、上記clean postcommit proof/fresh inventoryでtaskをcloseする。
Actual capture semantics、Typed/Resolved install、Core33--35、GeneratedOrigin、active
execution、Task277Bはdefer。
