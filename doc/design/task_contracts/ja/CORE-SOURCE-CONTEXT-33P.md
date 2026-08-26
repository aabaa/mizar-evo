# Task CORE-SOURCE-CONTEXT-33P: C4C8 Core context prerequisite boundary

> 正本は英語です。canonical English:
> [../en/CORE-SOURCE-CONTEXT-33P.md](../en/CORE-SOURCE-CONTEXT-33P.md)。

状態: complete documentation-only prerequisite。C4C8完了後にuserが選択した
dependency-minimal boundaryを記録する。Rust実装もlater association APIも認可しない。

## Identity、authority、readiness

| field | frozen value |
|---|---|
| task | `CORE-SOURCE-CONTEXT-33P` |
| primary owner | Core [crate plan](../../mizar-core/ja/00.crate_plan.md)、[source-family decomposition](../../mizar-core/ja/source_family_decomposition.md)、[TODO](../../mizar-core/ja/todo.md)による`mizar-core` Task 33 |
| upstream owner | Checker [C4C8](CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md)。implementation `c7595b60e7784728967cfbac9b02522f7290c942`、closure `c5792708e5451701f86a72ac6123df99bc1d3687` |
| authority | `doc/spec/en/`、exact C4C7 `.miz`、unchanged expectation/trace、C4C8R、C4C8、derived Core design/source inventory |
| user decision | Candidate 1、すなわちgeneric zero-semantic Core-33 prerequisiteをfreezeし、API選択前にstandalone immutable C4C8 association seamをreserveする |
| readiness | 本documentation taskだけがuniquely ready。Owner association、destination API、allocator mapping、complete corruption oracleが一意でないためactual Core33/C4C8 transportは**not ready**。 |
| coverage | semantic creditはzero。Task 277Bはnot ready/zero credit。 |

AuthorityはC4C7の意味を固定する。Inner mapperの`x`,`y` useはresolved binding
identityでouter generatorを参照し、inner generator `z`はlocal noncapture。C4C8は
exact `3/1/0/2/2` resolver graphを保持し、captured subvectorはprivate source/
declaration orderの`x,y`。C4C4 outer-`x` projectionはby-valueで`captured`は空のまま。

Fresh inventoryが固定した欠落事項:

- C4C7/C4C8にはcontaining functorを認証するchecker `SourceItemId`、`SymbolId`、
  `DefinitionId`、Core item identityがない。
- C4C8はstandalone immutable、syntax-free、Core-ID-free checker graphであり、Typed/
  Resolvedへinstallされずinstallerもない。
- checker Task 248のclosed source-context profileはnested comprehension generatorを
  表さず、そのresolver binding identityとcontaining functor symbol/definitionを結ばない。
- 現在の`CoreContextInput`はcaller-supplied item/variable/binder/generated seedを受けるが、
  missing authenticated owner bridgeもgraph allocator ruleも持たない。
- checker/Core/resolverのnumeric ID domainを相互にreinterpretできない。

したがってmissing Core associationは`design_drift`、future executable
source-derived sliceの不在は`test_gap`。C4C8R/C4C8 defectでもnew language semanticsでもない。

## Frozen ownerとsoundness boundary

Core Task 33はfuture context/item/binder identity、source/checker provenance、fresh
snapshot-local `CoreVarId` allocation、authenticated Core itemとC4C8 graphのdurable
associationのsole owner。本taskはstandalone immutable seamだけをreserveし、private/public
representationを選ばず、`CoreContextInput`、`CoreContext`、Typed、Resolvedへ追加しない。

Core Task 34はtype/attribute/evidence/coercion/view loweringをownする。Core Task 35は
Task 34後にcomplete Task-33 associationだけをconsumeし、term/formula/Fraenkel loweringと
`GeneratedOrigin`をownする。Task 35はassociationをallocate/repair/infer/recoverしない。
Generator-domain operandはcaptured parameter/argument subvectorと別であり、later join
可能なのは後者だけ、C4C8 private orderによるpositional one-to-one joinだけである。

Future association oracleはmissing/extra/duplicate/reordered/stale/foreign-owner/
cross-module/recovered/partial/mismatched/orphan rowをrejectする。Display name join、numeric
ID reinterpret、sort、repair、inference、unchecked admissionは禁止。これはminimum
default-deny requirementであり、frozen APIやcomplete executable oracleではない。

本taskはtask-semantic implementation ID、Rust type/field/adapter/installer/route/
destination slot/allocator、item/variable/parameter/argument/functor/generated origin、
diagnostic、expectation、trace credit、active runner behaviorを作らない。

## Candidate比較と未決定事項

| candidate | disposition | reason |
|---|---|---|
| Generic Core-33 base + reserved standalone immutable association seam | 本documentation prerequisiteに限りselected | Core33 ownerとzero-semantic/default-deny boundaryを守り、missing owner bridgeを仮造しない。 |
| C4C8-specific private Core-33 association | deferred | Complete private destination/API、owner key、allocator mapping、parameter order、corruption oracleが必要。 |
| Public `CoreContextInput`/`CoreContext` extension | deferred | Complete consumer contractより先にpublic API/ownership exposureを変更する。 |
| Task 248 extensionまたはpublic checker route | deferred | checker closed profile/API boundaryの別authority決定が必要。 |
| Second Typed/Resolved receipt | current contractではforbidden | C4C6がexact existing receiptをownし、C4C8はstandaloneを選択済み。 |
| Core Task 35 direct lowering | forbidden | Core33 association ownerを飛ばし、Task35にsoundness-critical identityをinfer/allocateさせる。 |

Implementation successor開始前に、authenticated containing-owner bridge、exact association
destination/visibility、identity-preserving allocator mapping、immutable API、captured
parameter/argument cardinality/order、complete default-deny corruption oracleをauthorityが
一意に選ぶ必要がある。Authorityが変わらなければminimum human decisionは、Core33が
checker-private routeによるnew checker-authenticated containing-owner linkを受けるか、new
public Core inputを公開するか。本taskは選択しない。

## Exact documentation scopeとprotected surface

変更可能なのは次のexact 10 documentation pathsだけ:

1. `doc/design/task_contracts/en/CORE-SOURCE-CONTEXT-33P.md`;
2. `doc/design/task_contracts/ja/CORE-SOURCE-CONTEXT-33P.md`;
3. `doc/design/mizar-core/en/00.crate_plan.md`;
4. `doc/design/mizar-core/ja/00.crate_plan.md`;
5. `doc/design/mizar-core/en/source_family_decomposition.md`;
6. `doc/design/mizar-core/ja/source_family_decomposition.md`;
7. `doc/design/mizar-core/en/todo.md`;
8. `doc/design/mizar-core/ja/todo.md`;
9. `doc/design/task_contracts/en/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md`;
10. `doc/design/task_contracts/ja/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md`。

Central spec coverage auditとCore source/spec auditは変更しない。本taskはspec coverage、
implementation owner/API/source correspondence、trace/test ownership/deferred creditを変えない。
全`doc/spec`、existing `.miz`/expectation/trace/source/Cargo/diagnostic、C4C4 captured、
Typed/Resolved、active route artifactをprotectする。

Entry時の8 existing scoped docs baselineはEN/JA plan `330/42980` /
`312/45108`、EN/JA decomposition `215/21666` / `195/20965`、EN/JA TODO
`626/36734` / `599/40467`、EN/JA C4C8 contract `442/21798` /
`238/15528`。SHA-256はcanonical EN contractの対応表を正本とする。Paired contract treeは
`105/105 -> 106/106`。

Protected C4C7 source/expectation/trace hashはそれぞれ
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`、
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`、
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`。
Protected stashは`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。Entry HEAD
`c5792708e5451701f86a72ac6123df99bc1d3687`、origin/main
`481a599877803e855307381901b82ae38365ce4a`、divergence `0/2`。

## Review、verification、exit

Independent spec/equivalenceとbilingual/boundary reviewはno findingsまで行う。
Documentation-onlyなのでpost-freeze test-sufficiency/implementation-equivalence/
source-docs-API reviewはcode不在を確認し、認可しない。Final read-only reviewは9 hard gates
すべてPASS、`90/100`以上を必要とする。

Core/mizar-test lint、recursive paired contract/link validation、fmt、offline metadata、
workspace all-target/all-feature warnings-denied Clippy、full workspace tests、diff、exact
scope/count/hash/protected checks、exact staging、task-only commit、clean postcommit proofを行う。

Exitにはcontract pairと4 paired owner referenceの一致、non-doc changeなし、protected hash/
stash不変、Task 277B not ready/zero credit、fresh inventoryで上記全owner/API/oracleが一意に
ならない限りactual implementationを停止することを要求する。

## Precommit completion evidence

Independent spec/equivalence、bilingual/boundary、test-sufficiency、implementation/
source-documentation/API reviewは**NO FINDINGS**。Intermediate verification findingは
mechanicalな1件だけで、recursive contract lintがnew Core-plan `Task Index` backlinkと
exact JA `canonical English:` markerを要求した。Frozen scope内で修正してfocused lintを
再実行し、repaired stateのindependent reviewも**NO FINDINGS**。

Core lint `12/12`、recursive contract/link validationを含むmizar-test lint `15/15`、
`cargo fmt --all -- --check`、offline Cargo metadata、workspace all-target/all-feature
warnings-denied Clippy、full workspace all-feature tests/doctests、`git diff --check`はPASS。
Working treeはexact 10 frozen documentation pathsだけで、sorted path-list SHA-256は
`20cfc1f5339cc29760a37b3faaee19f5c25aa1c3f98f174ebffc31cd16d44084`。Contract treeは
exact `106/106`。3 protected C4C7 hash、stash、origin、source/API/route state、Task 277B
statusは不変。Independent final-quality reviewは**NO FINDINGS**、全`9/9` hard gateが
score capなしでPASSし、uncapped `100/100`（spec `20/20`、test `20/20`、trace
`15/15`、implementation equivalence `15/15`、sync `10/10`、boundary `10/10`、
verification `5/5`、handoff `5/5`）。Commitは自身のhashを含められないため、exact
commit identity、clean postcommit proof、fresh successor inventoryはexternal final
handoffに記録する。
