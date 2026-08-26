# Task TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7: 2 capture inactive test intent

> 正本言語は英語。canonical English:
> [../en/TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7.md](../en/TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7.md)。

Owner planは[mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。

Stable owner sectionはchecker
[source/spec分類](../../mizar-checker/ja/source_spec_audit.md#task-257c4c7-two-capture-test-intent)、
[future projection境界](../../mizar-checker/ja/source_formula_composition.md#task-257c4c7-multi-capture-projection-boundary)、
[TODO](../../mizar-checker/ja/todo.md#task-257c4c7-two-capture-prerequisite)、
[bilingual record](../../mizar-checker/ja/bilingual_sync_audit.md#task-257c4c7-frozen-contract-parity)、
mizar-test [corpus](../../mizar-test/ja/miz_corpus.md#task-257c4c7-frozen-corpus-increment)、
[traceability](../../mizar-test/ja/traceability.md#task-257c4c7-inactive-trace-increment)、
[TODO](../../mizar-test/ja/todo.md#task-257c4c7-two-capture-inactive-oracle)、
[bilingual record](../../mizar-test/ja/bilingual_sync_audit.md#task-257c4c7-frozen-contract-parity)。

## Status、authority、readiness

**Status:** complete。implementation/review/verification/final-quality scoring、
exact staging、task-only commit、clean postcommit proof、fresh successor inventoryを
完了した。userがparentの以下のauthority判断を採用した。本taskはexact inactive two-capture oracle、metadata/backlink/
audit、mechanical global count guardだけを追加し、capture実装/active routeは追加しない。

Authority順はcanonical Chapter 13 §§13.4.3、13.4.4、13.8.6、以下のexact
test-first `.miz`、実装後のexisting nested-capture trace rowと2 sidecar、completed
C4C1--C4C6 derived owner、最後にnon-normativeなcurrent source observation。

Chapter 13はmultiple generatorを許し、nested captureをdisplay spellingでなく
resolved binder identityに結び、generated `params`をsurrounding free variablesと
定義する。従ってexact 2 outer binder membership/cardinality oracleは完全に導出でき、
`test_gap`を閉じるがorderは決めない。generalized checker projectionとcorruption
coverageの欠落は`design_drift`とlater private-unit `test_gap`。display name、
resolver/checker/Core numeric ID、single-row C4C5 `source_ordinal`をgeneral joinに
使うことは`boundary_violation`。orderをobservable language semanticsにすることは
`spec_gap`で本taskでは禁止する。authority contradictionと
`repo_metadata_conflict`はない。

Imported `Element`/`NAT` profile、独立にcoveredなmultiple-generator/nested parser
shape、completed C4C6、accepted owner/boundary decisionにより本taskだけがready。
Exact-one-capture C4C2--C4C6 validatorはnew witnessをconsumeせずunchanged。
Task 277Bはnot-ready / zero-creditのまま。

## Frozen ownerとdownstream boundary decision

- `mizar-test`がinactive artifactをownし、checker Task-257C
  `source_formula_composition`がfuture complete immutable syntax-free
  Core-ID-free multi-capture projectionのsole lower owner。
- Associationはauthenticated resolved binder identity。complete projectionは
  generator declaration、mapper、optional predicate、capture occurrence、
  owner/context/source provenanceをcoverする。exact fields/APIはfresh post-C4C7
  contractでのみfreezeする。
- 同じidentityの複数occurrenceはdistinct capture 1件。future transportは
  authenticated outer generator declaration/source orderをprivate deterministic
  conventionとして使えるが、language result、diagnostic、`.miz` assertionにしない。
- Projectionはstandalone。boxed C4C6 Typed/Resolved receiptはimmutableのまま。
  C4C7はTyped/Resolvedをreopenせず、new slotやreplacementを作らない。
- missing/extra/duplicate/reordered/stale/recovered/partial/mismatched/
  display-name joinはfail closed。consumerのsort/repair/inference/unchecked
  dedup/numeric reinterpretationは禁止。
- Core Task 33がfresh snapshot-local `CoreVarId` allocationとdurable typed
  associationをownする。Core Task 35はTask 34後にassociationをconsumeし、actual
  Fraenkel loweringとGeneratedOrigin/namingをownする。Task 35はallocation/inference
  しない。
- Semantic operand `S`とcaptured parameter/argumentを区別する。checker private
  orderでpositional 1:1なのはcapture subvectorだけ。whole
  `params.len() == args.len()`は要求も許可もしない。

C4C7はRust type/field/adapter/installer/Core route/GeneratedOriginを作らない。
Artifact commit後のfresh inventory前にそれらを推測しない。

## Exact test-first sourceとsidecar

`tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.miz`
へfinal-LFでexactに追加する:

```mizar
import parser.nested_capture_fixtures;

definition
  func NestedCaptureTwo -> set equals
    { { [x, y] where z is Element of NAT }
      where x is Element of NAT, y is Element of NAT };
end;
```

`193` bytes、expected SHA-256
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`。
Inner mapperはresolved outer `x`と`y`だけを参照し、inner generator `z`はlocalで
captureされない。2-member capture setだけをassertし、orderはassertしない。

Matching `.expect.toml`はschema `1`、matching id/source、pass、
`advanced_semantics`、`set_expressions.nested_capture`、pass/type_check、empty
diagnostics、sole spec ref
`spec.en.13.set_expressions.nested_capture.semantic`。exact noteは:

```text
Inactive advanced_semantics pass oracle derived from Chapter 13 sections 13.4.3, 13.4.4, and 13.8.6: the inner mapper references both resolved outer generator identities x and y, while inner z remains local. It asserts capture membership/cardinality only, not generated-parameter/application-argument order. Frontend admission uses parser.nested_capture_fixtures; generalized resolver/checker capture transport, execution, Core lowering, and Task 277B remain deferred.
```

Final-LF sidecarは`885` bytes、expected SHA-256
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`。
active tagとfailure-only fieldはない。

## Trace、audit、exact implementation scope

Existing nested-capture requirementのid/source/section/
`advanced_semantics/covered/required/pass`とsole parser dependencyを保持し、sorted
`tests`へnew sidecarだけをappendする。noteはone/two-capture inactive seedと
zero executable/order creditを記録する。new requirement/status changeは禁止。

`doc/design/spec_coverage_audit.md`にはdedicated C4C7 zero-credit mapping/
follow-up section 1件だけを追加する。Chapter-13 summary rowはstatus/broad follow-up
不変なので`partial`のまま。dedicated sectionがexact second-oracle mappingと
downstream boundary deltaをownする。Historical numeric proseはrebaselineしない。

Documentation freezeはexact 22 paths: paired contract、checker/mizar-test EN/JA
Task Index plan、上記checker source-spec/source-formula/TODO/bilingualとmizar-test
corpus/traceability/TODO/bilingualのpaired owner record。Artifact completionでは
そのtask status/evidenceと以下8 pathsだけを更新でき、final scopeはexact 30 paths。

Contract/owner review後のartifact/verification変更はexact 8 paths: new `.miz`、new
sidecar、trace、coverage audit、および4 existing global metadata-count test files。
4 Rust filesでは`(429, 396) -> (430, 396)`と
`(236, 193) -> (237, 193)`だけを変更する。production Rustは変更しない。

## Baseline、expected impact、protected state

Clean baseline HEADは`60dbe59e26659ccce16c7999f81760597b3ef2fd`、
origin/mainは`ffc882675141a3e25bc78a47affc018bfe3685e1`、divergence `0/2`。
Protected stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`は不変。

- corpus pairs `344/344 -> 345/345`;
- cases/requirements `429/396 -> 430/396`;
- pass/fail `236/193 -> 237/193`;
- requirements、active route/stage、warning/error countは不変;
- contract trees `101/101 -> 102/102`;
- trace baseline `5924` lines / `464057` bytes / SHA-256
  `d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`;
- coverage audit baseline `7088` lines / `540634` bytes / SHA-256
  `1ec5de8dbccdf3afee01c710ac22f00af933ee57ec749e930cf89f8936b27cfd`。

Protected existing one-capture `.miz`とsidecar hashはそれぞれ
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`、
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`。
Trace baselineはpre-change evidenceで、frozen second backlink/note deltaだけを許可。
`doc/spec`、existing pair、other trace field/row、C4C4 captured state、diagnostic、
active route、checker/Core production、Task 277Bをprotectedとする。

## Review、verification、exit、handoff

Artifact追加前にspecification/equivalenceとEN/JA/boundary reviewを独立に実施し、
blocking/high findingをzeroにする。実装後にtest sufficiency、artifact/metadata、
source/docs/API、final qualityを独立reviewし、findingは修正後re-reviewする。

Focused parser、mizar-test metadata/lint/lib、checker lint、fmt、offline metadata、
workspace all-target/all-feature warnings-denied Clippy/full test、diff checkを実行。
New/protected hash、corpus/contract inventory、link/fragment、CLI counts、C4C4 empty
captured、zero production diff、Task-277B zero creditを再確認する。Exitは9/9 hard
gates、valid 90/100以上、exact staging/commit、clean postcommit proof、fresh
successor inventoryを要求する。

次候補はstandalone complete projectionのseparate checker C4C8 contract。
fresh inventoryでexact immutable API/fields、complete graph/cardinality validator、
private ordering oracle、destination/consumer、default-deny matrixがlanguage semanticsを
変えず一意になった場合だけready。そうでなければgap分類して停止する。

## Completion evidence

Exact 30-path implementationはcomplete。New sourceは`7` lines / `193` bytes /
SHA-256 `b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`、
sidecarは`13` lines / `885` bytes / SHA-256
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`。
Corpus pair `345/345`、contract tree `102/102`、metadata `137/137`、cases/
requirements `430/396`、pass/fail `237/193`、warnings/errors `23/0`。
Requirement/stage coverage/architecture matrix/active route countは不変。

Final traceは`5925` lines / `464335` bytes / SHA-256
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`、
coverage auditは`7107` lines / `541809` bytes / SHA-256
`18dbb5048e949461c03f8d59c61c2b0c63ac3bcea19d01b8d1fa2126dc8d8c39`。
Protected one-capture source/sidecar、`doc/spec`、other existing corpus/trace row、
production Rust、C4C4 captured state、diagnostic、active route、Core state、Task277B
は不変。

Independent specification/equivalence、bilingual/boundary、test-sufficiency、
artifact implementation、source/docs/API reviewは全て**NO FINDINGS**。Focused
parser `1/1`、mizar-test lib `623/623`、both lint policy `15/15`、metadata
`137/137`、fmt、offline Cargo metadata、full-workspace all-target/all-feature
warnings-denied Clippy、3 frontend benchmarkを含むfull tests、diff checkはPASS。

Independent final-quality reviewは**NO FINDINGS**。Parent adjudicationは全`9/9`
hard gatesをPASS、score capなし、valid uncapped `100/100`
（`20/20/15/15/10/10/5/5`）と確認した。Exact 30-path precommit status-path hashは
`38fe0671baff256460020a1b650a657f679d33690b0cf0b20e751c43d610e860`。
Exact task-only staging/cached reviewも**NO FINDINGS**。

## Postcommit proofとfresh successor inventory

Task-only artifact commitは
`3d28af5f6678519fe8d764fb29f27eb664db8f39`（`test(checker): add
two-capture Fraenkel oracle`）。Frozen exact 30 pathsだけを変更し、sorted
path-list SHA-256は
`38fe0671baff256460020a1b650a657f679d33690b0cf0b20e751c43d610e860`、
`git show --check`はPASS。直後のworktreeはclean、`origin/main...HEAD`は`0/3`、
origin/mainは`ffc882675141a3e25bc78a47affc018bfe3685e1`のまま、protected stashは
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`で不変。

Independent checker/Core/oracle inventoryはcomplete checker C4C8 projection自体が
まだdependency-readyでない点で一致した。Current C4C2 resolver candidateはexact
2 generator segmentsとsingle identifier mapper useだけをadmitするが、C4C7 witnessは
inner `z`、outer `x`/`y`、bracket application内の2 identifier useを持つ。
C4C3--C4C5もexact-oneで、C4C6 receiptはprotected。従ってexact complete-projection
field/cardinality/constructor/error precedenceとlater Core-33/35 associationはdeferし、
checker C4C8 API/task contractはここで作らない。

ただしparent authority adjudicationは、より狭いdependency-minimal successorを1件に
決定した。既存resolver R2/C4C2 collectionだけがrequired resolved binding identityを
authenticateでき、existing public multi-row tableとglobal source-order/dense-ordinal ruleが
transport shapeを既に決める。Checker-private replacementはresolver identityの再生成または
reinterpretationを要するため`boundary_violation`としてrejectする。従って本frozen witness
だけを対象にするexact zero-semantic/no-new-public-API resolver prerequisite
`RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R`を一意にselectする。Bindingは
inner `z`、outer `x`、`y`、mapper linkは`x`、`y`の順で、全てexisting authenticated
source orderに従う。Current rejectionは`source_drift`、resolver/private real-fixture
coverage欠落は`test_gap`、checker C4C8はprerequisite commit後のfresh inventoryで
complete graphをfreezeするまで`design_drift`。Task277Bはnot-ready/zero-creditのまま。
