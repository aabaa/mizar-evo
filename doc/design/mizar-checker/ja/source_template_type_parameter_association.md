# Source Template Type-Parameter Association

> canonical English: [../en/source_template_type_parameter_association.md](../en/source_template_type_parameter_association.md)。正本は英語です。

## Task 277B-L Template Type-Parameter Association

これは standalone checker module
`crates/mizar-checker/src/source_template_type_parameter_association.rs` のdurable module
ownerである。orchestration、source/test scope、baseline、readiness boundaryは
[Task 277B-L](../../task_contracts/ja/277B-L.md) が所有する。moduleは実装済みだが、
neutral transportはTask 277Bを**readyにしない**。

moduleはcomplete R1 の `TemplateTypeParameterSourceCollection` とexisting `TypedAst` を
consumeしimmutable association handoffを返す。277A `source_template` をextendせず、
Typed/Resolved slotをinstallせず、production runner routeを作らない。

### Public API

`SourceTemplateTypeParameterAssociationId` は `new(index: usize) -> Self` と
`index(self) -> usize` だけをexposeする。`SourceTemplateTypeParameterAssociation` のgetter:

- `binding() -> TemplateTypeParameterBindingId`;
- `definition_block()`、`parameter()`、`binder()`、`type_head()`、`identifier()` は各
  `TypedNodeId`;
- `parameter_range()`、`type_head_range()` は各 `SourceRange`; および
- `parameter_source_ordinal()`、`type_head_source_ordinal()` は各 `usize`。

`SourceTemplateTypeParameterAssociationTable` は
`get(SourceTemplateTypeParameterAssociationId) ->
Option<&SourceTemplateTypeParameterAssociation>`、`iter() -> impl Iterator<Item =
(SourceTemplateTypeParameterAssociationId,
&SourceTemplateTypeParameterAssociation)>`、`len() -> usize`、`is_empty() -> bool`だけを
exposeする。

`SourceTemplateTypeParameterAssociationHandoff` は `source_id() -> SourceId`、
`module_id() -> &ModuleId`、`associations() ->
&SourceTemplateTypeParameterAssociationTable`、`debug_text() -> String`をown/exposeする
唯一のoutput ownerで、caller DTOはない。
`SourceTemplateTypeParameterAssociationError` は `#[non_exhaustive]`、
`EnvironmentMismatch` と
`InvalidAssociation { association: SourceTemplateTypeParameterAssociationId }` を持つ。
`SourceTemplateTypeParameterAssociationProducer` はexactに:

```rust
build(
    collection: &TemplateTypeParameterSourceCollection,
    typed_ast: &TypedAst,
) -> Result<
    SourceTemplateTypeParameterAssociationHandoff,
    SourceTemplateTypeParameterAssociationError,
>
```

### Invariant と validation

rowはR1 binding identity、range、source ordinalをretainし、既にorder/ambiguity authorityである
resolver link orderにdense。checkerはreorder/duplicate error variantを追加しない。

validationはfail-closedかつordered: environment; R1 binding lookup; 5 siteそれぞれを
`TypedNode.resolved_node == Some(R1 ResolvedNodeId)`でscanしてexactly-one match; normal
recovery; exact node kind（`DefinitionBlockItem`、`TemplateParameter`、binder/generator identifier
双方のcanonical `Identifier`、`TypeHead`）; Range anchor; R1 range equality; binder-within-parameter、
parameter/type-head-within-definition、identifier-within-type-head range; direct `definition -> parameter`、`parameter -> binder`、
`type_head -> identifier` edge。environment後のfailureはassociation-specific invalid error。
0件/複数match、dense-ID cast、range/name inferenceはfailし、producerはdeterministicで
`TypedAst`をmutateせず、新Typed/Resolved link slotを追加しない。

R1 fixture associationはbinding `0`、57-node arena/root 56 の
`DefinitionBlockItem#53` / `TemplateParameter#31` / `Identifier#2` から
`TypeHead#39` / `Identifier#21`、parameter range `606..620`、type-head range
`678..679`、両ordinal 0。

### Module boundary と test

implementationが変更できるのはnew module、checker `lib.rs`、checker lint-policy inventory、
private mizar-test leaf、その `tests.rs` includeだけ。resolver source、`source_template.rs`、
277A、Typed/Resolved install、Cargo、canonical specification/test/metadata、production runner/
facade/dispatchはeditしない。

checker test 4件はexact mapping/public getter、source/module mismatch、5 site各々のmissing/ambiguous
match、all-site kind/recovery corruption、prefix-spoofされたnon-canonical `Identifier` kind、
non-range/wrong-source/empty anchor、exact-range/containment、各direct-edge removal、empty/singleton/
multi-link profileのdeterministic non-mutating rebuildをexhaustively coverする。private mizar-test real-fixture
probe 1件はsame validated Surface/Resolved 57-node profileからprivate F5 `TypedAst`を構築し、
resolver IDをarena mappingだけからattachしてproducerをdirect callする。existing helperや
277A routeはこのtyped arenaを供給しない。active semantic/diagnostic/coverage effectはない。

implementationはchecker lint policyのpublic-enum module list、source/spec module list、public-API
path allowlist、`lib.rs` public-module allowlistへ本moduleを追加した。paired source/spec auditは
crate-export rowとexact public-item inventoryを現在持つ。test-sufficiency reviewは**NO FINDINGS**、
implementation reviewはcanonical `Identifier` prefix-spoof fix後に**NO FINDINGS**。
source/documentation re-reviewはEN/JA CLI tense fix後に**NO FINDINGS**、bilingual/boundary reviewも
**NO FINDINGS**。checker/mizar-test lint、focused/full library、package/workspace Clippy、full test、
format/diff check、metadata、unchanged CLI hash、protected-surface gateは全てPASS。final-quality
reviewはidentifier-within-type-head range containment欠落のMedium 1件をfindingした。repairは
containmentとcorruption assertionを
`task277bl_rejects_kind_range_recovery_and_direct_edge_corruption`へ追加し、本EN/JA owner/contractを
同期した。finding-specific re-reviewは**NO FINDINGS**。全9 hard gateはscore capなしvalid
`100/100`（`20/20/15/15/10/10/5/5`）でPASS。exact staging/cached-diff review、task-only commit、
post-implementation proof、fresh successor inventoryはcentral [historical checkpoint](../../task_contracts/ja/277B-L.md#post-implementation-checkpoint)でclosedし、successorはselectしない。
この実装はTask 277Bのnot-ready/
zero-semantic-credit boundaryをretainする。

## Public Enum Policy

| enum | policy | exhaustive exception |
|---|---|---|
| `SourceTemplateTypeParameterAssociationError` | `#[non_exhaustive]` | none |
| `SourceTemplateFraenkelStructuralCompositionError` | `#[non_exhaustive]` | none |

この module が所有する exhaustive public enum exception はない。

## Task 277C frozen planned public extension

canonical [277C contract](../../task_contracts/ja/CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md) は本existing
moduleでcompleteしたneutral structural-composition family、
`SourceTemplateFraenkelStructuralCompositionId`、row、table、handoff、
`#[non_exhaustive] SourceTemplateFraenkelStructuralCompositionError`、producerをreserveする。
producerは`build(template, generators, typed_ast)`をexposeし、immutable composition handoffを返す。
contractだけがexact error precedence、row getter、validation、F5 profile、test matrix、measured completion
evidenceをownし、上のpublic-enum policyが適用される。

これは`SourceTemplateTypeParameterAssociationHandoff` +
`FraenkelGeneratorVariableSourceCollection` + `TypedAst`のstandalone compositionのままである。R1 direct
input、state installation、source-owner route、semantic credit、production activationは追加していない。legacy-stable
headingはexisting owner linkを保持する。
broad verificationはPASSし、independent source/documentation、bilingual、boundary reviewは
**NO FINDINGS**。final-quality reviewも**NO FINDINGS** / valid uncapped `100/100`
（`20/20/15/15/10/10/5/5`）である。historical closeoutをrecordする。exact staging/cached review、task-only implementation commit、
post-commit proof、fresh successor inventoryはlanguage-local [central historical checkpoint](../../task_contracts/ja/CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md#historical-immediate-post-implementation-checkpoint)でclosedしている。
successorはselectせず、Task 277Bはnot ready/semantic credit zeroのままである。

## Task 257C4A Fraenkel generator dependency boundary

completed [C4A](../../task_contracts/ja/CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A.md)はcompleted 277C/R2のnonduplicative
consumerである。authoritative rowと`TypedAst`をconsumeし、unique resolved-to-typed binder relationをID castなしでrecheckする。
このmoduleの277C association ABIをextendせず、structural/debug summaryだけをtrustしない。lower clone/raw resolver nodeはopaqueであり、term/type-sethood/diagnostic/install/route/Task277B conclusionはboundary外にdeferする。
implementation/test reviewはdependency boundary findingなし。277C association ABI/sourceは不変で、C4Aはfrozen consumer ownerだけに実装された。broad workspace verificationはPASS。independent source-doc/boundary reviewは**NO FINDINGS**。
final-qualityは**NO FINDINGS**、全`9/9` hard gates PASS、valid uncapped `100/100`（`20/20/15/15/10/10/5/5`）。exact staging/cached reviewはPASS。pendingはtask-only commit/post-commit proof/fresh inventoryだけ。
