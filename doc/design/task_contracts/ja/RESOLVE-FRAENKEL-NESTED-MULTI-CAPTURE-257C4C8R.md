# Task RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R: exact 2-capture resolver identity prerequisite

> 正本言語は英語。canonical English:
> [../en/RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R.md](../en/RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R.md)。

Owner planは[mizar-resolve](../../mizar-resolve/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。Durable ownerはresolver
[names](../../mizar-resolve/ja/names.md#resolver-task-257c4c8r-exact-nested-multi-capture-identity)と
mizar-test [harness](../../mizar-test/ja/harness.md#resolver-task-257c4c8r-private-two-capture-probe)。

## Status、目的、readiness

**Status:** implementation complete、task-only commit pending。Parser
[C4C8P prerequisite](PARSER-FRAENKEL-GENERATOR-DELIMITER-257C4C8P.md)は
`6bc8de3a007d0260d14d2c803dc335623b6aa912`でcommitされ、resolver source work前のmandatory
fresh exact-source preflightはC4C7 ASTがdiagnostics-free/unrecoveredと証明した。

Clean post-C4C7 inventoryが選んだdependency-minimal successor。既存resolver-owned
`FraenkelGeneratorVariableSourceCollection`をexact C4C7 sourceだけに拡張する。Public
Rust item/enum variant、checker capture、Typed/Resolved installation、Task252 occurrence
graph、type/sethood、semantic verdict、diagnostic、active runner、Core identity、generated
parameter/origin、Task277B creditは追加しない。

Chapter 13とC4C7 oracleは3 generator declarationと2 mapper referenceのresolved
binder identityを固定する。Existing R2/C4C2 public tableはmulti-row、resolved-node
provenance、global source order、dense ordinal、deterministic debugを既にownするため、
no-new-public-API resolver extensionが一意。Checker-private identity producerはresolver
identityを再生成/reinterpretする`boundary_violation`。Current rejectionは
`source_drift`、resolver/private real-fixture coverage欠落は`test_gap`。Later checker
graph/APIだけが`design_drift`で、本taskにblocking `spec_gap`/`repo_metadata_conflict`はない。

## Authorityとdependency

Authority順はcanonical Chapter 4 §4.6、Chapter 13 §§13.4.3--13.4.4/13.8.6、exact
existing two-capture `.miz`、existing trace/inactive sidecar、completed R2/C4C2/C4C7、
最後にnon-normative source inventory。Required commitsはC4C7 artifact
`3d28af5f6678519fe8d764fb29f27eb664db8f39`とclosure
`b4037c853632aed80a824d05b955e4ad6396f4e1`。C4C2 branchとC4C3--C4C6はprotected。

Docs commit `5b165dd38e5f1a560eeaff80ef65aa8e5eab0539`後のimplementation preflightで、
first outer generatorの`Element of NAT`がnext generator commaをconsumeし、exact sourceに
parser recoveryがあることを発見した。このparser `source_drift`はC4C8Pだけがownし、resolverの
reconstruction/repair/recovered-AST admissionは禁止。

## Frozen exact resolver relation

Existing R2/C4C2 public type/getter/enum/error/constructor/table/debug grammarをbyte-for-byte
reuseし、private exact candidate branch 1件だけを追加する。Existing global segment-range
sort後のbindingはinner `z` ID0（segment `110..129`、binder `110..111`）、outer `x`
ID1（`144..163`、`144..145`）、outer `y` ID2（`165..184`、`165..166`）。

Inner bracket mapperは`97..103`。`x@98..99`と`y@101..102`がsame inner
comprehension/mapper ownerのexisting `Mapper` linkとなり、binding ID1/2、global/
mapper-local ordinal `0/0`、`1/1`を得る。Inner `z`にはuse linkがなくouter capture
identityではない。Debug末尾はexisting grammarの`bindings=3|uses=2`。

この順序はexisting resolver source orderingでありlanguage semanticsやlater checker
capture-vector contractではない。Consumerのsort/repair/inference/unchecked dedup/
display-name join/numeric reinterpretationは禁止。Identityは`resolved_node_for`だけから得て、
`new`/`collect`はcomplete arenaをrevalidateする。

Admissionはexact/default-deny。One definition/functor、2 nested condition-free
comprehension、3 generator segments、inner `z`、outer `x`,`y`、全てexact normal
`Element of NAT`、inner mapper exact bracket application `[x, y]`だけを認める。Missing/
extra/duplicate/reordered/renamed/alternate-type/condition/wrapper/recovery/partial/extra nesting/
unsupported shapeはcandidate全体zero rows。Existing F5/R2 malformed nested/C4C2 outputは不変。

## Scope、baseline、expected impact

Docs prerequisiteはexact 11 paths: paired contract、両owner plan pair、paired resolver names、
paired mizar-test harness、coverage audit。Auditにはzero-credit section 1件だけを追加し、
Chapter-13 summaryは`partial`のまま。

Implementationはexact 3 Rust paths: `names.rs`、`names/tests.rs`、existing private
`fraenkel_nested_capture_identity.rs`。Artifact completionはpaired contractのstatus/evidenceと
dedicated coverage-audit paragraphのplanned->completed stateだけを追加更新でき、final
implementation commitはexact 6 paths。Other durable owner sectionはcompletion-neutral wordingの
ためstatus-only edit不要。Resolver testはexact 4件:
`task257c4c8r_collects_exact_nested_multi_capture_relation`、
`task257c4c8r_preserves_outer_scope_and_excludes_inner_generator`、
`task257c4c8r_rejects_near_miss_profiles`、
`task257c4c8r_revalidates_arena_and_replays_deterministically`。Private runner testはexact
`task257c4c8r_real_imported_fixture_links_both_outer_generators`。Real C4C7 sourceをexisting
frontend/resolver lowerへ通し、3-binding/2-link relation、normal provenance/replay、unchanged
empty import augmentationだけをassertし、production/advanced-semantics routeにしない。

Baseline HEADは`b4037c853632aed80a824d05b955e4ad6396f4e1`、origin/main
`ffc882675141a3e25bc78a47affc018bfe3685e1`、divergence `0/4`、stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。Source baselineは`names.rs`
`4183/131548` SHA `ac05067d09a8da784e6faa8f5078eb4e7b57c4dfa331d06b94594f7edc97254d`、
tests `4287/137729` SHA `feb8f5721131c5bc92ba8e04ced2cfe9634e16c21f64f876a2bafb27ed1858d1`、
private leaf `589/23379` SHA `86d9f5fcdc088fb678f5346fac01bf5f904821cf18455f75d2b7c6792a6e1e5a`。
Library testはresolver `160->164`、mizar-test `623->624`、contract tree `102/102->103/103`。

C4C7 source/sidecar hashは
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`/
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`、traceは
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`で不変。
`doc/spec`、`.miz`、sidecar、trace/metadata、parser/checker/Core production、C4C4 captured、
diagnostic、active route、Task277Bはprotected。

## Review、verification、exit、handoff

Pre-source specification/equivalenceとbilingual/boundary、post-source test-sufficiency、
implementation、source/docs/API、final-quality reviewを独立に行い、finding修正後は再review。
Focused C4C8R/C4C2/R2、resolver/mizar-test lib、both lint、metadata、parser comprehension、
fmt、offline metadata、workspace warnings-denied Clippy/full tests、diff/hash/count/protected
checksを実行する。Exitは9/9 hard gates、valid 90/100以上、exact task-only commit、clean
postcommit、fresh inventory。Checker C4C8はその時にgraph/API/cardinality/default-denyが
一意ならfreezeする。Core 33/35とTask277Bはdefer。

## Precommit implementation completion evidence

Fresh post-C4C8P frontend preflightはexact frozen C4C7 sourceを使い、diagnostic/recovery 0、
AST 95 nodes、2 set comprehension、3 generator segments、bracket application 1件とfrozen
mapper/segment/binder/type rangeを確認した。Resolver implementationはfrozen Rust 3 pathだけを
変更し、private exact candidate 1件を追加、existing public R2/C4C2 type/tableを全てreuse、
source-order ID採番前にuseをbinder node identityへmapし、resolver-owned resolved-node
provenanceだけをemitする。

Final source measurementsは以下。

| Path | Lines / bytes | SHA-256 |
|---|---:|---|
| `crates/mizar-resolve/src/names.rs` | `4415 / 140538` | `663ec040a0b9525cb79b532fe7ae6a548f67acb7510b8713df3b0cfe2b8d6166` |
| `crates/mizar-resolve/src/names/tests.rs` | `4798 / 153865` | `d53afc1d148b3ab55bdbf97a04d11f78f4fe454a0caf6ca43f8ea72d6a55c504` |
| `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs` | `704 / 27913` | `6a1717fec263e79d9295813b413d1ec323c3291297f9ee04e0bc7c8e59e2e754` |

Resolver/mizar-test libraryはexact `164`/`624` tests、sorted raw-list SHAは
`a01c16a16aead9868d30257e358a4e742dd7633a8da4f61c864d9197d9c1f1c8`/
`21196d1cb959c5b6bd7b38f19efb83d334978ec7f1d0c99e35da19cec8afe385`。
Exact C4C8R 5 tests、C4C2/R2 compatibility、resolver lib `164/164`/lint `11/11`、
mizar-test lib `624/624`/metadata `137/137`/lint `15/15`、parser C4C8P compatibility、
fmt、offline Cargo metadata、`git diff --check`はPASS。

Independent test-sufficiency/implementation reviewは**NO FINDINGS**。最初のreal-fixture
executionはnormal `end;` siblingsを拒否するimplementation-local mismatchを発見したため、
review前にexisting C4C2 one-functor child boundaryへ整合し、temporary debugを除去して
real fixtureをPASSさせた。Broad workspace verification、source/docs/API・bilingual/boundary
のうち、warnings-denied all-target/all-feature Clippyとfull all-feature workspace testsは
PASS。Bilingual/boundary reviewがC4C8P auditのstale sentence 1件を発見し、このcompleted
zero-credit mappingへ整合後のfinding-specific re-reviewは**NO FINDINGS**。Source/docs/API
reviewも**NO FINDINGS**で、exact scope/count/hash/public API/owner link/protected boundaryを
確認した。Autonomous-development rubricに対するindependent final-quality reviewも
**NO FINDINGS**、hard gate `9/9` PASS、valid uncapped `100/100`。Exact stagingとtask-only
commitだけが残るexit step。

C4C7 source/sidecar/traceはprotected hashを再現し、paired contract treeは`104/104`。
`doc/spec`、`.miz`、expectation、trace/metadata、parser/checker/Core production、C4C4
captured、diagnostic、active route、Task277B stateは不変。本taskがcloseするのはresolver
`source_drift`とprivate `test_gap`だけで、coverage creditはzeroのまま。
