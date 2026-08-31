# Task CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB: standalone source local-binder context

> 正本は英語です。canonical English:
> [../en/CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md](../en/CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md)。

Status: implementation/verification complete、exact task-only commit pending。本taskはCore Task 33内の
zero-semantic/zero-credit prerequisiteであり、Core 33をcompleteせず
`MT10-CIR-TE`もactivateしない。

## Identity、authority、decision

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB` |
| Primary owner | `mizar-core::elaborator`、Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/ja/00.crate_plan.md) |
| Checker dependency | 既存immutable `mizar_checker::binding_env::BindingEnv`。reserve-only Task-20 routeは直接所有し、Checker Task 248は`SourceBindingContextHandoff::binding_env()`から公開する |
| Prepared consumer | Future `MT10-CIR-TE`。残りのCore 33 item associationと該当Core 34/35 payloadがcompleteなreal `CoreIr`を生成できた後だけ |
| User decision | `CoreContextInput`、`CoreContext`、`CoreIr`、Typed、Resolvedを拡張せず、standalone Core-33 local-binder prerequisiteを採択 |
| Coverage | Semantic/execution credit zero。Broad Core rowとTask277Bはdeferred/not-readyのまま |

Authority順は`doc/spec/en/`、既存`.miz`、trace、expectation、design、source。
Chapter 4では`reserve`はvariable declarationではなくmodule default contextである。
一方、既存reserve-only expectationはCore itemやsemantic resultなしのbinder-only
`CoreContext` readinessを要求する。Checker Task 248はmodule reserve/defaultと
declaration-local parameterのidentity、context、visibility、source order、structural
shadowingをauthenticateする。これらをzero-semantic Core context transportとして扱う
ことは両authorityと整合するが、quantification、closure、Core item、factへ
reinterpretしてはならない。

Pre-freeze auditはrunnerの`CoreVarId::new(binding_id.index())`を
`boundary_violation`/`source_drift`と分類した。Checker/Core numeric domainは別物である。
一般source-itemから`SymbolId`/`CoreItemId`へのowner bridge欠落と`CoreIr`の
module context table欠落は`design_drift`、最初のreal `MT10-CIR-TE` baseline欠落は
`test_gap`のまま。期待された1-commit divergenceと異なり`origin/main == HEAD`である
状態はreport-only `repo_metadata_conflict`で、本taskは修復しない。

## Frozen public APIとownership

`crates/mizar-core/src/elaborator.rs`に次を追加する。

- private constructorと`binding()`/`core_var()` getterを持つimmutable
  `SourceBindingCoreVariable`。
- checker `BindingTable` iteration orderをexactに保持し、`get(BindingId)`、
  `iter()`、`len()`、`is_empty()`を持つimmutable
  `SourceBindingCoreVariableTable`。
- 更新済み`CoreContext`、complete checker `BindingEnv`、association tableをretainし、
  `source_id()`、`module_id()`、`context()`、`binding_env()`、`variables()`、
  non-authoritative `debug_text()`を持つimmutable
  `SourceBindingCoreContextHandoff`。
- precedence順に`EnvironmentMismatch`、`InvalidCoreContext`、
  `InvalidBindingEnvironment`、`CoreVariableAllocationOverflow`、
  `CoreVariableCollision { var: CoreVarId }`、`InvalidBindingAssociation`を持つ
  non-exhaustive `SourceBindingCoreContextError`。
- `SourceBindingCoreContextProducer::build(context: CoreContext,
  binding_env: BindingEnv) -> Result<SourceBindingCoreContextHandoff,
  SourceBindingCoreContextError>`。

Producerは両inputをby-valueでconsumeし、complete validation後だけhandoffを公開する。
Table/row/handoff fieldはprivate。Installer、adapter、unchecked constructor、numeric-ID
conversion、mutable public field、第2のTyped/Resolved slot、`CoreContextInput`/`CoreIr`
fieldを追加しない。

Exact error messageはEnglish正本に記載した6文字列とする。

## Cardinality、order、allocation、provenance

Admitするchecker payloadはnonempty/completeで、diagnostic、recovered/degraded
context/binding、binding diagnostic、captured-free-variable、unsupported kindを含まない。
次だけを受理する。

- normal module contextの`ReservedVariable` kind + `ReservedVariable` identity +
  `Reserved` status。
- normal declaration contextの`DefinitionParameter` kind + exact `ResolverLocal`
  identity + `Active` status。

Binding identity、spelling、declaration range、visible ordinal、owner context、local scope、
type siteはchecker ownerのまま。Coreはdisplay nameをidentity keyへcopyせず、source itemも
reconstructしない。Associationはchecker bindingごとにexact 1 rowで、既存
`BindingTable::iter()` orderを保持する。Display name、range、map iteration、
reconstructed ordinalでsortしない。

Coreは既存Core variable identity全体のchecked maximum + 1、空なら0からconsecutiveな
snapshot-local `CoreVarId`をallocateする。Declared variable、binder source、binder
frame、type-fact key、generated-origin parameterをused-ID validationへ含める。
Checker/resolver numeric valueをallocationへ使用しない。

各rowは`NormalizedVarClass::Free`、`NormalizedVarSort::Term`、empty type facts、
`reserved-variable`または`definition-parameter` roleとしてinstallする。これはtransport
metadataだけである。Checker binding id `n`のprovenance keyはchecker phaseの
`source-binding-core-variable-v1.binding.<n>`。Binding declaration rangeをdirect source
anchorとし、binder sourceとchecker-owned provenanceは同じsingle entryを持つ。

## Default-deny oracle

Sort、repair、inference、recovery、partial publicationなしで次をrejectする。

1. Core/checkerのsource/module mismatch。
2. 既存Core variable、binder source/frame、type fact、generated-origin referenceの不整合。
3. Empty、diagnostic-bearing、recovered、degraded、captured、unsupported、または
   identity/status/context mismatchのchecker binding state。
4. Allocation overflow/collision。
5. Missing/extra/duplicate/reordered/stale/mismatched/orphan association row。
6. Wrong role/class/sort/source range/provenance、nonempty type facts。
7. Authenticated table外のreserved source-binding Core role。

`BindingEnv::try_new`とChecker Task 248がprivate construction invariantのsole validatorで
ある。Coreはadmission subsetと自身のcomplete associationをvalidateし、checker oracleを
duplicateまたはweakenしない。

## Installation boundaryとdeferral

Reserve-only Task-20 runnerはcaller-built variable/binder seedを本producerへ置換し、
返されたassociationを`BindingId`でvalidateする。Checker Task 248 Profile Aのprivate
real-source consumerはexact 2-row reserve/local-parameter association、structural shadow
distinction、fresh allocation、deterministic replay、zero-semantic contextを証明する。
既存`.miz`、expectation、trace、coverage statusは変更しない。

Future `MT10-CIR-TE` producerは残りのCore 33 item associationとCore 34/35 loweringより前に
Task-248 `BindingEnv`を本handoffへconsumeできる。本handoff自体はserializeせずsnapshotを
activateしない。`CoreIr`にmodule context tableはなく、prepared consumerはcompleteな
`CoreIr::debug_text()` bytesを要求する。最初のactive baselineはreal sourceとcomplete
Core33--35 payloadを別contractでfreezeする。

Standalone C4C8 handoffは別物として不変。両方使う場合はlocal-binder producerを先に実行し、
C4C8がその`CoreContext`をextendできる。互いのhandoffへinstallしない。Source-item/
Core-item association、type/evidence、term/formula、parameter/argument、`GeneratedOrigin`、
diagnostic、active route、snapshot、Core 34/35 semantics、Task277B readinessはdeferred。

## Scope、baseline、review、exit

Implementation/test pathは次の3つだけ。

- `crates/mizar-core/src/elaborator.rs`。
- `crates/mizar-test/src/runner/type_elaboration/checker_handoff.rs`。
- `crates/mizar-test/src/runner/tests/type_elaboration/source_context.rs`。

Owned doc deltaは本contract pair、paired Core plan/TODO/task-ledger/source-family/elaborator/
source-spec audit/bilingual audit、paired mizar-test harness/bilingual audit、central
coverage audit。Specification、`.miz`、
expectation、trace row、checker source、C4C4/C4C8 state、diagnostic registry、manifest、active
route、legacy compaction recordはprotected。

Entry source baseline、test count、protected hash、stash、review/verification/exit条件は
English正本の同名sectionに記載した値をauthoritativeとする。Contract treeは`109/109`
から`110/110`になり、Core lib testは`159`からfocused test 4件増加予定、mizar-test test
countは既存testへのassertion追加だけで不変予定である。

## Completion evidence

Standalone producer、immutable association handoff、existing private consumer 2件は
complete。Final source measurementは`elaborator.rs` `19323/715066` bytes
SHA-256 `2de75000b5a5fd280d7b1ba313b78551640c28e688f9bd36bf02b102e8129f7b`、
reserve runner `1285/50148`
`f62fa1db0e9e9b7e20cbfb39529eb1138c647c2eafb18623fcabfeb09f4c8186`、Task-248
private leaf `3290/122501`
`46ca0d25d4d39ff420489a19ac03ca7c571b6b1e7baa6f363e3c91aa6a8fa1c2`。
Core lib testは`163`（`159 + 4`）、mizar-test lib testは`632`のまま、contract
treeは`110/110`。

Independent pre-source specification/equivalence、API feasibility、bilingual/boundary
reviewはno findings。Post-source implementation reviewもno findings。Test-sufficiencyと
source/documentation/API reviewでin-scope test matrix/derived-doc driftだけを検出し、
unsafe opaque-ID fixture除去と同期修正後のfinding-specific re-reviewは双方no findings。

Focused Core `4/4`、real Task-248 `1/1`、reserve/C4C8 no-regression probe、Core lint
`12/12`、mizar-test lint `15/15`、metadata `137/137`、format、offline metadata、
warnings-denied all-target/all-feature workspace Clippy、full all-feature workspace test/
doctestはPASS。Protected reserve/Task-248/C4C7/trace hashは全freeze値を再現し、stashは
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`のまま。

Parent reviewはautonomous hard gate `9/9` PASS、score capなしのvalid `99/100`。
Specification `20/20`、test contract `19/20`（public shared 33LB-to-C4C8 receipt fixture
不在をnon-blocking integration residualとする）、traceability `15/15`、implementation
`15/15`、design/source sync `10/10`、boundary discipline `10/10`、verification `5/5`、
handoff `5/5`。General Core 33 item association、Core 34/35、`GeneratedOrigin`、first real
`MT10-CIR-TE`、Task277Bはzero-creditでdeferred。Report-only `repo_metadata_conflict`も残る。
Task-only commit前のactual `origin/main`/`HEAD`はともに
`a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c`で、requested remote `774a4781` stateでは
なかった。Fetch/push/metadata repairは未実施。
