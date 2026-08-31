# Task CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259: Task259 predicate item context

> canonical English:
> [EN contract](../en/CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259.md)。

Status: implementation/verification complete、exact task-only commit pending。Core Task 33で最初のdependency-minimalな
nonempty source-item/Core-item associationであり、zero-semantic/zero-creditである。
Core 33完了や`MT10-CIR-TE` activationはclaimしない。

## Identity、authority、decision

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259` |
| Primary owner | `mizar-core::elaborator`、Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/ja/00.crate_plan.md) |
| Checker dependency | existing Task-248 Profile-B `SourceBindingContextHandoff`とactive Task-259 `SourcePredicateDefinitionHandoff` |
| Core dependency | completed 33LB handoff |
| Consumer | complete Core 33--35後のfuture `MT10-CIR-TE` |
| User decision | checker-authenticated owner linkをstandalone immutable Core handoffがconsumeし、public Core input/fieldを追加しない |
| Coverage | zero。Task277Bはnot-ready/zero-credit |

Authority orderは`doc/spec/en/`、existing `.miz`、trace、expectation、design、source。
Chapter 4/11/12はscope、source/declaration order、module identity、definition-block
boundaryをfixする。Task259はone normal public predicateのwhole `SymbolId`とTask248
Profile-B definition contextを既にauthenticateする。

Exact joinは`SourcePredicateDefinition.context()`から同じ`BindingContextId`の
`SourceContextLink.item`を選び、retained whole `SymbolId`でexisting Core itemを選ぶ。
Range/display name/numeric indexはjoinに使用しない。これはexact Task259 sliceの
`design_drift`/`source_drift`/`test_gap`をcloseするderived transportであり、新しい
language semanticsや`spec_gap`ではない。Remote baseline差はreport-only
`repo_metadata_conflict`で自動修復しない。

## Frozen public API

`crates/mizar-core/src/elaborator.rs`へ次を追加する。

- `SourcePredicateCoreItemAssociation`と`source_item()`、`definition()`、
  `symbol()`、`core_item()`；
- source-ordered `SourcePredicateCoreItemAssociationTable`と
  `get(SourcePredicateDefinitionId)`、`iter()`、`len()`、`is_empty()`；
- complete 33LB handoff、Task248 source context、Task259 owner、tableをby-valueで
  retainする`SourcePredicateCoreContextHandoff`と`source_id()`、`module_id()`、
  `context()`、`source_bindings()`、`source_context()`、`checker_owner()`、
  `items()`、non-authoritative `debug_text()`；
- `#[non_exhaustive] SourcePredicateCoreContextError`：
  `EnvironmentMismatch`、`InvalidSourceBindingContext`、
  `InvalidCheckerOwner`、`InvalidCoreContext`、`InvalidItemAssociation`；
- `SourcePredicateCoreContextProducer::build(
  SourceBindingCoreContextHandoff,
  SourceBindingContextHandoff,
  SourcePredicateDefinitionHandoff,
  ) -> Result<SourcePredicateCoreContextHandoff,
  SourcePredicateCoreContextError>`。

全fieldはprivateで、complete postvalidation後だけpublishする。Constructor、adapter、
installer、unchecked admission、`CoreContextInput`/`CoreContext`/`CoreIr` field、
Typed/Resolved slotは追加しない。

## Cardinality、identity、order、oracle

Admitするのはexact Task259 profileだけである。Task248はnormal `DefinitionBlock` item
1件、ordered parameter/binding 2件、context link 2件、diagnostic/recovery 0件。
Task259はnormal predicate 1件、parameter 2件、guard/property/correctness各1件。
Coreは同じwhole `SymbolId`のvalid public `Predicate` item 1件、pending definitional
boundary 1件で、extra/missing item、dependency、diagnosticは0件。

Associationはexact 1 rowで、Task259 definition id、definition contextから
`SourceContextLink`で選択したTask248 source item id、whole `SymbolId`、exact registry
lookupで選択した`CoreItemId`をretainする。OrderはTask259 definition-table orderで、
sortしない。Core item/boundary sourceはinner predicate definition rangeを使い、outer
definition-block rangeへ置換しない。Provenance keyは
`source-predicate-core-item-v1.definition.0`。

Default-denyはsource/module mismatch、33LB/Task248 environment mismatch、stale
fingerprint、nonexact Task248/259 cardinality/role/context/owner/recovery/diagnostic、
missing/extra/duplicate/reordered/stale/mismatch/orphan row、missing/foreign/`None` context link、
wrong Core symbol/kind/visibility/status/source/provenance/source-map/worklist/dependency/
diagnostic/boundaryをrejectする。Display name、spelling、FQN-only、range-only、numeric
index、shell ordinal、seed order、map iterationによるjoin、およびnumeric IDの相互
reinterpretation、sort/repair/inference/partial publicationは禁止する。

## Boundary、scope、deferral

Existing private Task259 real-source testはauthenticated Task259 rowだけからCore item seedを
作り、33LB後にproducerをcallしてexact one-row chain、determinism、default-deny matrixを
verifyする。Existing `.miz`、expectation、trace、active selection、diagnostic、coverageは
変更しない。

Multi-definition blockへのgeneralization、Task260+ owner family、reserve/property item、
aggregation policyはseparate descendantである。Core34/35/36 semantics、type/fact/term/
formula/body、semantic parameter/argument、`GeneratedOrigin`、C4C8、active snapshot、
diagnostic、Task277Bはdeferする。

Source scopeは`elaborator.rs`とexisting Task259 mizar-test leafだけ。Derived docsはpaired
contract、Core plan/source-family/TODO/elaborator/source-spec/bilingual/ledger、mizar-test
harness/bilingual、central coverage auditだけ。Checker API/ownerは不変なのでchecker docsは
変更しない。Central auditはzero-credit mappingとremaining follow-upだけを更新する。

Freeze baselineは`elaborator.rs` `19323 / 715066`、SHA
`2de75000b5a5fd280d7b1ba313b78551640c28e688f9bd36bf02b102e8129f7b`、Task259 leaf
`517 / 17989`、SHA
`95eca63c134d2a367e35f4feb277ff0f9bc4197ea254cc42e0445e383312b201`。
Contract treeは`110/110 -> 111/111`。Protected hash/stashのexact値はEN canonicalを
参照し、Task259/248/reserve/C4C7/trace/stashは全て不変とする。

Entry HEADは`9795ca073e081c23193cb7d51411fa00fddcfd6b`、actual
`origin/main`は`a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c`、divergence `0/1`。
Fetch/push/metadata repairは禁止。

Independent pre-source spec/equivalenceとbilingual/boundary review、post-source test/
implementation/source-doc-API reviewをfindingなしまで行う。Focused checks後にCore/
mizar-test lint、fmt、offline metadata、full warnings-denied Clippy、full all-feature tests、
protected/diff/staging/cached/commit/postcommit proofを行う。Exitは`9/9`、parent score
`>=90/100`、task-only commit、protected invariance、Task277B not-ready/zero-credit、fresh
successor inventoryを要求する。

## Completion evidence

Standalone producerとexact Task259 private consumerはcomplete。Final measurementは
`elaborator.rs` `19986 / 741842`、SHA-256
`82971830bd539f184a69675ac502aa317be3f7ebc3ffaab118b07870444ba161`、Task259 test leaf
`877 / 31757`、SHA-256
`309ef24a97f8d55212fea6c655bab1a7374f7b120dd4afe9e70fa0e0885cd4a9`。Core library
testは`163`のまま、mizar-test library testは`634`（`632 + 2`）、contract treeは
exact `111/111`。

Pre-source spec/equivalence reviewはfindingなし。Initial bilingual/boundary reviewのJA
contract sync finding 2件を修正し、再reviewはfindingなし。Post-source implementationと
source/doc/API reviewの共通findingであったiteration-selected Core itemをwhole
`SymbolId` exact registry lookupへ修正した。Test-sufficiency findingの33LB retention、
item/boundary、source-map assertion不足も修正し、3つのfinding-specific再reviewは全て
findingなし。

Focused Task259 Core-context `2/2`、Core lint `12/12`、mizar-test lint `15/15`、metadata
`137/137`、fmt、offline metadata、full warnings-denied Clippy、doctestを含むfull
all-feature testsがpass。Protected Task259/248/reserve/C4C7/trace hashは全てfreeze値を
再現し、stashは`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。

Parent hard gateは`9/9`、score capなしのvalid `99/100`。Immutable upstream handoffの
mutationはexisting producer suiteをreuseするためtest contractを`19/20`とし、他の
specification/traceability/implementation/design-sync/boundary/verification/handoff gateは
満点。Multi-definition association、Task260+、Core34/35/36、`GeneratedOrigin`、
`MT10-CIR-TE`、active route、diagnostic、coverage、Task277Bはdeferred/not-ready、zero
creditのまま。

Report-only `repo_metadata_conflict`は継続する。Precommit `HEAD`は`9795ca073e081c23193cb7d51411fa00fddcfd6b`、
actual `origin/main`は`a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c`、divergence `0/1`で、earlier requested
remote `774a4781` stateとは異なる。Fetch/push/metadata repairなし。Dependency-minimal
successor readinessはこのcontractではなくfresh postcommit inventoryで決める。
