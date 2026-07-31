# ソース述語定義 intake

> 正規言語は英語です。英語版:
> [../en/source_predicate_definition.md](../en/source_predicate_definition.md)。

状態: Checker Task 259 frozen contract plus post-Task-248 contract correction
prerequisite。本書は将来の実装境界を凍結するものであり、producer の実装、
fixture/sidecar の追加、traceability/coverage credit の変更は行いません。
separately committed Task-248 Profile-B dependency は completed current
state として下記に記録します。

## Authority、scope、finding

canonical authority は次のとおりです。

- 通常の述語宣言、pattern、順序付き typed parameter、定義ローカルな
  `assume` guard、`means` definiens、述語 property を規定する Chapter 9
  §§9.1、9.3-9.5。
- 定義 biconditional、typed guard、domain restriction を規定する Chapter 9
  §§9.9.3-9.9.5。
- 定義時 correctness obligation を規定する Chapter 16 §16.6。
- 既存の parser predicate-definition/property-clause fixture と test、既存の
  predicate/functor 混合 type-elaboration boundary、resolver
  declaration-shell/signature test。
- Checker Tasks 248-258 で完成した public API。

exact contract の欠落は nonblocking `design_drift`、source-to-checker producer
の不在は `source_drift`、専用 real consumer の不在は `test_gap` です。
blocking な `spec_gap` はありません。Chapter 9 は `assume` guard が定義域を
制限することと symmetry property が obligation を作ることをそれぞれ規定しますが、
guard から symmetry VC への正確な式、量化、antecedent 構成は規定していません。
そのため Task 259 は deterministic で opaque な pending obligation identity を
transport するだけとし、semantic FOL goal 構築を明示的に defer します。現在の
source behavior からその構築を推測しません。

本taskが所有するのは predicate-definition intake、すなわち definition identity、
parameter、guard、既に生成済みの atomic definiens、property identity、initial
obligation link だけです。property proof、recursive unfolding、truth、acceptance、
axiom publication は所有しません。

## Exact future source

専用の将来 pass fixture は、final LF を含む次の正確な 165 UTF-8 bytes です。

```mizar
definition
  let x be set;
  let y be set;
  assume x = x;
  pred Task259PredicateDefinition: x task259_rel y means x = y;
  symmetry by computation(steps: 1);
end;
```

SHA-256 は
`91bdb5f51c0ea5f07bdd831700cb9803f2aa57e005921c7e4e1798ecbbf2bd9f`
です。このsourceは通常の `means` predicate 1件、個別に記述した `set`
parameter 2件、equality guard 1件、equality definiens 1件、明示的な
computation justification を伴う `symmetry` property 1件を含みます。functor、
theorem、proof block、import、correctness keyword、recovery は含みません。

## Frozen Surface profile

exact parser result は Surface row 71件、root node 70、root range `0..164`、
recovery 0件です。関連rowとrangeは次のとおりです。

| Node | Surface kind | Range | Direct role |
| ---: | --- | --- | --- |
| 38 | `TypeHead` | `22..25` | 1番目の builtin `set` head |
| 39 | `TypeExpression` | `22..25` | 1番目の written parameter type |
| 41 | `DefinitionParameter` | `13..26` | 1番目の `let x be set;` |
| 42 | `TypeHead` | `38..41` | 2番目の builtin `set` head |
| 43 | `TypeExpression` | `38..41` | 2番目の written parameter type |
| 45 | `DefinitionParameter` | `29..42` | 2番目の `let y be set;` |
| 50 | `BuiltinPredicateApplication` | `52..57` | guard equality |
| 51 | `FormulaExpression` | `52..57` | guard formula |
| 53 | `AssumptionStatement` | `45..58` | definition-local guard owner |
| 54 | `PredicatePattern` | `94..109` | `x task259_rel y` |
| 59 | `BuiltinPredicateApplication` | `116..121` | definiens equality |
| 60 | `FormulaExpression` | `116..121` | definiens formula |
| 61 | `FormulaDefiniens` | `116..121` | predicate definiens owner |
| 62 | `PredicateDefinition` | `61..122` | predicate declaration |
| 64 | `ComputationJustification` | `137..158` | proof-content subtree |
| 65 | `JustificationClause` | `134..158` | explicit justification subtree |
| 66 | `PropertyClause` | `125..159` | symmetry property |
| 67 | `DefinitionBlockItem` | `0..164` | 共通 direct owner |
| 70 | `Root` | `0..164` | complete Surface root |

binding declaration range は `x 17..18` と `y 33..34`、written type range は
`22..25` と `38..41` です。predicate declaration label は `66..92`、pattern
は `94..109`、その symbol token は `96..107` です。

node 41、45、53、62、66 はこの順序で node 67 の direct structural sibling
です。parameter と assumption は node 62 のchildではありません。したがって
sole predicate との共通normal block、順序、containment、associationを認証する
のはcheckerやresolverではなくprivate runner selectorです。guardとdefiniensの
lower-family traversalは2つのequality subtreeだけを選択し、pattern loci、
declaration label、property-justification subtreeをterm/formula occurrenceとして
扱ってはいけません。

## Frozen raw resolver profile

type-elaboration enrichment 前の raw resolver profile は正確に次のとおりです。

- declaration shell 3件、signature projection 2件、symbol diagnostic 0件、
  symbol 2件、definition 2件、local-source contribution 1件。
- shell 0: `DefinitionBlock`、node 67、ordinal 0、parentなし。
- shell 1: `PredicateDefinition`、node 62、ordinal 1、parent shell 0。
- shell 2: `PropertyClause`、node 66、ordinal 2、parent shell 0。
- predicate `DefinitionId(0)`: `SymbolKind::Predicate`、
  `DefinitionKind::Predicate`、spelling/notation は
  `x task259_rel y`、origin anchor `61..122`、structural path
  `[4,0,8,0]`、normal、conflict-free、localかつexported。
- generic property projection: `SymbolKind::Attribute`、
  `DefinitionKind::Attribute`、origin anchor `125..159`、structural path
  `[4,0,17,1]`。

generic property projection はresolver collection scaffoldingにすぎません。
Task 259 はこれを、propertyがpredicate propertyである、symmetricである、
proof済みである、acceptedであるというsemantic evidenceとして再解釈または
consumeしてはいけません。

`DefinitionParameter` と `AssumptionStatement` にはdeclaration shellがありません。
predicate resolver definitionのparameter/binder collectionは空でsyntactic arityも
ありません。Task 259はresolver fieldからarity 2、parameter identity、guard
ownershipを推測してはいけません。これらはexact private source selectorと、
別途拡張されたTask-248 handoffだけから得ます。

## 必須の別lower prerequisite — 完了

元のTask-259 contractをfreezeした時点では、Task-248 producerはreserve 1件＋
同名shadowing parameter 1件のProfile Aだけを受理し、このsourceのnormal
definition block 1件＋個別に型を記述したparameter 2件を拒否していました。
そのためcontractは、Task 259が`BindingEnv`を再構築せず、`BindingId`を
fabricateせず、Task-259 implementation commit内でTask 248を拡張しないことを
要求しました。この要求は引き続き有効です。

必要だった別logical taskは完了済みです。

1. `f9b47375acc18acebf56a69f5d8a7edec539c2be`が、normal
   `DefinitionBlock` 1件、順序付きで個別に記述された
   `DefinitionParameter` binding 2件、reserve itemなし、というProfile Bを
   freezeしました。
2. `ca54135f36c9fecfc02c2b8120ec4e63e8c6ca36`が、そのprofileとprivate
   exact runner selectorを実装し、既存`SourceBindingContextHandoff`を
   preserveしました。

post-`ca54135f` fresh inventoryにより、exact 165-byte sourceから
checker-authenticated `BindingId` 2件、definition-local
`BindingContextId` 1件、exact parameter site、shadow/capture edgeなしが
得られることを確認しました。Profile Aは不変です。したがって次の
dependency-ready logical taskはTask-259 implementationであり、このhandoffを
consumeしてownershipを重複させてはいけません。

## Frozen lower consumer bundle

別Task-248 extension後のexact lower bundleは次のとおりです。

| Owner | Exact profile |
| --- | --- |
| Task 248 | 既存`SourceBindingContextHandoff`。exact one-block/two-parameter profileだけを拡張 |
| Task 249 | application 2 / expression 2 / argument 0 |
| Task 252 | `VariableReference` term 4 / binding reference 4 / numeric request 0 |
| Task 256 | `Equality` formula 2 / wrapper 0 / segment 0 / head 0 / candidate 0 / type site 0 / attribute 0 / edge 4 / request 4 |

public Task-256
formula/wrapper/segment/head/candidate/type-site/attribute/edge/request順では、
last profileはexactly `2/0/0/0/0/0/0/4/4`です。

Task-252 source orderはguardの`x`、guardの`x`、bodyの`x`、bodyの`y`です。
最初の2 referenceは第1 parameter bindingを選び、後半2つは第1、第2 parameter
bindingを選びます。Task-256 formula 0はguard `x = x`、formula 1はdefiniens
`x = y`です。それぞれが`BuiltinLeftOperand`/`BuiltinRightOperand` edge 2件と
`OperandExpectedType` request 2件を所有します。

Task-259 handoffはexact Task-248、Task-249、Task-252、Task-256 handoffを
fingerprintします。Tasks 253-255、257、258はabsentです。特にdefinition-local
`AssumptionStatement`はTask-259 guardであり、`SourceStatement` rowでも
Task-258 assumption/factでもありません。

## Public syntax-free contract

implementationはchecker-owned source moduleに5つのdense ID familyを追加します。

```rust
pub struct SourcePredicateDefinitionId(usize);
pub struct SourcePredicateParameterId(usize);
pub struct SourcePredicateGuardId(usize);
pub struct SourcePredicatePropertyId(usize);
pub struct SourcePredicateCorrectnessId(usize);
```

各IDは`Copy + Eq + Ord + Hash`で、既存dense-IDの`new`と`index` APIを持ち、
vector順にallocateされます。public input/row familyは次のように凍結します。
raw `SurfaceAst`、`SurfaceNodeId`、`SyntaxKind`、parser nodeはこのseamを越えません。

```rust
pub struct SourcePredicateDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourcePredicateDefinitionInput>,
    pub parameters: Vec<SourcePredicateParameterInput>,
    pub guards: Vec<SourcePredicateGuardInput>,
    pub properties: Vec<SourcePredicatePropertyInput>,
    pub correctness: Vec<SourcePredicateCorrectnessInput>,
}

pub struct SourcePredicateDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
    pub definiens: SourceAtomicFormulaId,
}

pub struct SourcePredicateParameterInput {
    pub owner: SourcePredicateDefinitionId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
}

pub struct SourcePredicateGuardInput {
    pub owner: SourcePredicateDefinitionId,
    pub ordinal: usize,
    pub formula: SourceAtomicFormulaId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
}

pub struct SourcePredicatePropertyInput {
    pub owner: SourcePredicateDefinitionId,
    pub ordinal: usize,
    pub kind: SourcePredicatePropertyKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
}

pub struct SourcePredicateCorrectnessInput {
    pub owner: SourcePredicateDefinitionId,
    pub property: SourcePredicatePropertyId,
    pub ordinal: usize,
    pub source_anchor: SourceAnchor,
}

#[non_exhaustive]
pub enum SourcePredicatePropertyKind {
    Symmetry,
}

#[non_exhaustive]
pub enum SourcePredicateDefinitionRecovery {
    Normal,
    Degraded,
}
```

## Public immutable output API

immutable row type名と保存fieldはexactに次のとおりです。

| Row type | API順の保存field |
| --- | --- |
| `SourcePredicateDefinition` | `id`, `symbol`, `definition`, `contribution`, `site`, `source_range`, `source_ordinal`, `context`, `recovery`, `spelling`, `definiens`, derived `origin` |
| `SourcePredicateParameter` | `id`, `owner`, `ordinal`, `binding`, `written_type`, `site`, `source_range`, `declaration_range`, `context`, `recovery`, `spelling` |
| `SourcePredicateGuard` | `id`, `owner`, `ordinal`, `formula`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourcePredicateProperty` | `id`, `owner`, `ordinal`, `kind`, `site`, `source_range`, `justification`, `recovery`, `spelling` |
| `SourcePredicateCorrectness` | `id`, `owner`, `property`, `ordinal`, `source_anchor`, derived `obligation` |

各保存fieldは同名のread-only getterを1個持ちます。dense ID、resolver ID、
ordinal、range、context、recovery/kind enum、obligationはvalueで返します。
`symbol`、`site`、`origin`、`justification`、`source_anchor`はshared
reference、`spelling()`は`&str`を返します。setter、mutable getter、public
constructor、row replacement APIはありません。

table名とsignatureはexactに次のとおりです。

```rust
pub struct SourcePredicateDefinitionTable { /* private rows */ }
pub struct SourcePredicateParameterTable { /* private rows */ }
pub struct SourcePredicateGuardTable { /* private rows */ }
pub struct SourcePredicatePropertyTable { /* private rows */ }
pub struct SourcePredicateCorrectnessTable { /* private rows */ }
```

各tableは`get(id) -> Option<&Row>`、
`iter() -> impl Iterator<Item = (Id, &Row)>`、`const len() -> usize`、
`const is_empty() -> bool`だけを公開します。`iter`はdense source orderであり、
sortしません。

handoffとproducer surfaceはexactに次のとおりです。

```rust
pub struct SourcePredicateDefinitionHandoff { /* private fields */ }

impl SourcePredicateDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn source_context_fingerprint(&self) -> &str;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn source_term_fingerprint(&self) -> &str;
    pub fn source_atomic_formula_fingerprint(&self) -> &str;
    pub const fn definitions(&self) -> &SourcePredicateDefinitionTable;
    pub const fn parameters(&self) -> &SourcePredicateParameterTable;
    pub const fn guards(&self) -> &SourcePredicateGuardTable;
    pub const fn properties(&self) -> &SourcePredicatePropertyTable;
    pub const fn correctness(&self) -> &SourcePredicateCorrectnessTable;
    pub fn debug_text(&self) -> String;
}

pub struct SourcePredicateDefinitionProducer;
```

4 fingerprintは対応するlower handoffの完全な`debug_text()` stringであり、
caller-supplied digestでもoptional valueでもありません。producerは4つのlower
handoffがexact source/module identityを共有し、同じtyped arenaに対してinstall
可能であることをvalidateした後でfingerprintをderiveします。

## Public Enum Policy

| Public enum | compatibility policy |
| --- | --- |
| `SourcePredicatePropertyKind` | `#[non_exhaustive]`。callerはlater explicitly-frozen predicate-property kindを許容する。 |
| `SourcePredicateDefinitionRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourcePredicateDefinitionError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

transactional build result/error surfaceも次のように凍結します。

```rust
pub struct SourcePredicateDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourcePredicateDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourcePredicateDefinitionProjection {
    pub const fn base_initial_obligations(
        &self,
    ) -> &InitialObligationTable;
    pub const fn handoff(&self) -> &SourcePredicateDefinitionHandoff;
    pub const fn initial_obligations(&self) -> &InitialObligationTable;
    pub fn into_parts(
        self,
    ) -> (
        InitialObligationTable,
        SourcePredicateDefinitionHandoff,
        InitialObligationTable,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePredicateDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition,
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidGuard { index: usize },
    InvalidProperty { index: usize },
    InvalidCorrectness { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

impl std::fmt::Display for SourcePredicateDefinitionError;
impl std::error::Error for SourcePredicateDefinitionError;

impl SourcePredicateDefinitionProducer {
    pub fn build(
        input: SourcePredicateDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        source_atomic_formula: &SourceAtomicFormulaHandoff,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<
        SourcePredicateDefinitionProjection,
        SourcePredicateDefinitionError,
    >;
}
```

producerはauthenticated baseline tableをcloneし、全existing row/IDを
byte-for-byte preserveしてexactly one rowを
`InitialObligationId(base_initial_obligations.len())`へappendします。
projectionはbaseline cloneとcompleted tableの両方をretainし、installationが
stale/colliding stateをrejectできるようにします。exact future fixtureの
baselineはemptyなのでnew rowは`InitialObligationId(0)`、completed tableは
one rowです。

`SourcePredicateDefinitionError`に`Default`/blanket conversionはありません。
variantはsource/provenance、dependency、各dense row family、obligation
construction、arena ownership、exact admitted profileに対するfail-closed
aggregate categoryです。enumはnon-exhaustiveなのでcallerはwildcardを
保持します。

immutable output rowは対応するdense `id`を追加し、definition rowはさらに
resolver-derived `SemanticOrigin`、correctness rowはproducer-allocated
`InitialObligationId`をstoreします。どちらのderived valueもcallerから
supplyしません。
5つのpublic immutable tableは`SourcePredicateDefinitionTable`、
`SourcePredicateParameterTable`、`SourcePredicateGuardTable`、
`SourcePredicatePropertyTable`、`SourcePredicateCorrectnessTable`です。
各tableが公開するのは`get`、source-ordered `iter`、`len`、`is_empty`だけで、
insert/replace APIは公開しません。

`SourcePredicateDefinitionHandoff`はsource/module identity、5 table、
`SourcePredicateDefinitionProducer::build`内でexact lower-handoff
`debug_text()`からderiveした4 dependency fingerprintを所有します。callerは
fingerprintをsupplyできません。getterはborrowed/read-onlyです。
stable row-family/debug keyは正確に次の5つです。

- `source.definition.predicate`
- `source.definition.predicate.parameter`
- `source.definition.predicate.guard`
- `source.definition.predicate.property`
- `source.definition.predicate.correctness`

すべてのenumは`#[non_exhaustive]`です。exact sourceが受理するのはnormal rowだけで、
`Degraded`はfail-closed extension boundaryとして存在し、このprofileでは拒否します。

## Frozen debug grammar

`SourcePredicateDefinitionHandoff::debug_text()`は唯一のdependency
fingerprintであり、exactに`source-predicate-definition-debug-v1\n`で
始まります。その後、次のline familyをこのexact orderで出力し、final LFを1件
持ち、blank lineは持ちません。

```text
source-predicate-definition-debug-v1
module: <ModuleId.path>
source-context-fingerprint: <Rust-debug String>
source-type-fingerprint: <Rust-debug String>
source-term-fingerprint: <Rust-debug String>
source-atomic-formula-fingerprint: <Rust-debug String>
definition#<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> ordinal=<n> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> origin_range=<start>..<end> origin_path=<Rust-debug [u32]> spelling=<Rust-debug String> definiens=<id>
parameter#<id> owner=<id> ordinal=<n> binding=<id> written_type=<id> range=<start>..<end> declaration_range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
guard#<id> owner=<id> ordinal=<n> formula=<id> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
property#<id> owner=<id> ordinal=<n> kind=<symmetry> range=<start>..<end> site=node#<id> justification=range:<start>..<end> recovery=<normal|degraded> spelling=<Rust-debug String>
correctness#<id> owner=<id> property=<id> ordinal=<n> anchor=range:<start>..<end> obligation=<id>
```

`Rust-debug String`はowned stringのstandard escaped `{:?}` renderingを
意味します。exact admitted profileは`TypedSiteRef::Node`と
`SourceAnchor::Range`だけを受理します。role site、point/generated anchor、
imported/recovered origin、extra grammar branchはfail closedです。definition
origin lineは、validationがsource/moduleをhandoff identity、
`import_edge = None`、`recovered = false`に固定するため、それらのfieldを
省略します。row lineはdefinition、parameter、guard、property、correctnessの
dense table orderで出力します。typed/final debug renderingはこの完全なtextを
exactly once含み、second Task-259 summaryを出力しません。

## Exact 5-table / obligation oracle

table cardinalityはdefinition/parameter/guard/property/correctness順に正確に
`1/2/1/1/1`です。

- Definition 0はresolver `DefinitionId(0)`とpredicate
  symbol/contribution/origin、source ordinal 0、range `61..122`、
  definition-local context、spelling
  `pred Task259PredicateDefinition: x task259_rel y means x = y;`、
  definiensとしてTask-256 `SourceAtomicFormulaId(1)`を認証します。
- Parameter 0/1はdefinition 0をownerとし、ordinal 0/1、Task-248の`x`/`y`
  binding、Task-249 application 0/1、range `13..26`/`29..42`、declaration
  range `17..18`/`33..34`、同一definition-local context、spelling
  `let x be set;`/`let y be set;`を保持します。
- Guard 0はdefinition 0をownerとし、Task-256
  `SourceAtomicFormulaId(0)`、range `45..58`、definition-local context、
  spelling `assume x = x;`を使います。
- Property 0はdefinition 0をownerとし、ordinal 0、kind `Symmetry`、range
  `125..159`、spelling `symmetry by computation(steps: 1);`を持ち、明示的な
  justification anchor `SourceAnchor::Range(134..158)`だけを保持します。
- Correctness 0はdefinition 0、property 0、ordinal 0、obligation 0、
  `SourceAnchor::Range(125..159)`をlinkします。

Task 259は`InitialObligationKind::PredicatePropertyCorrectness`を追加します。
producerが返すcomplete `InitialObligationTable` rowは正確に1件です。

| Field | Exact value |
| --- | --- |
| id | `InitialObligationId(0)` |
| kind | `PredicatePropertyCorrectness` |
| owner | property 0の認証済みtyped site |
| range | `125..159` |
| assumptions | empty |
| goal | opaque key `source.definition.predicate.correctness:property=0` |
| provenance | opaque key `source.definition.predicate:definition=0:property=0` |
| status | `Pending` |

goal/provenance stringはdeterministic transport identityであり、FOL formulaの主張
ではありません。通常のexact `means` predicateはexistence/uniqueness obligation
を追加しません。Task 259はguardからassumption factを作らないため、obligationの
`assumptions` vectorは空です。

## Authentication / validation

`SourcePredicateDefinitionProducer::build`はtransactionalです。syntax-free input、
`SymbolEnv`、拡張済みの既存`SourceBindingContextHandoff`、Task-249 type handoff、
Task-252 primary-term handoff、Task-256 atomic-formula handoff、認証済みtyped arena
に加えてauthenticated current `InitialObligationTable` baselineを受け取り、
baseline clone、complete immutable predicate handoff、completed obligation
tableを含むprojectionを返すか、いずれもpublishせずerrorを返します。

predicate resolver `SymbolEntry`、`DefinitionEntry`、contributionはsource/module、
local-source contribution、normal origin、range `61..122`、predicate
symbol/definition kind、exact spelling/notation、public local declarationの
visibility/export state、conflict-free definitionと一致しなければなりません。
すべてのlower ID、site、context、range、ordinal、spelling、4 dependency
fingerprintはowning handoffとtyped-arena source keyに一致しなければなりません。

propertyは、private runnerがnormal Surface node 66をnormal block 67内のpredicate
node 62より後のdirect siblingとして、source orderとnon-overlap rangeを含めて
証明し、supplied typed siteをtyped-arena source keyへ一致させることだけで認証
します。checkerはそのsyntax-free relationを検証します。resolverのgeneric
Attribute projectionからproperty meaningを導出しません。

missing、duplicate、reordered、dangling、cross-owner、cross-module、
recovered/degraded、stale site/context/range/origin/contribution、
stale symbol/definition/binding/lower ID/fingerprint、wrong kind/spelling/ordinal、
partial、extra rowはすべて拒否します。wrong obligation owner、range、kind、
status、goal、provenance、assumptions、property link、correctness anchor、
cardinalityも拒否します。input orderは検証し、sortやrepairをしません。

## Justification / semantic boundary

Task 259が保持するのは明示的な`134..158` justification anchorだけです。node 64
`ComputationJustification`またはnode 65 `JustificationClause`のproof contentを
consume、copy、lower、interpret、validateしません。Task 258はこのexact routeで
absentです。future Task 272のproof skeleton/justification content ownershipは
維持されます。Task 259はそのsubtreeをTask 272から除外せず、事前acceptもしません。

property proof、proof search、discharge result、`VcId`、accepted obligation、
accepted predicate definition、type fact、theorem fact、biconditional axiom、
ATP premiseは一切生成しません。

## Typed / final installation

`TypedAst`は1つのinstallation APIを通じてTask-259 handoffとcomplete obligation
tableをatomicにinstallします。installationは4 lower handoffを必須とし、そのexact
fingerprintを再現し、全typed siteを検証し、既存/partial Task-259 occupancyを
拒否します。current `InitialObligationTable`がprojection retained baselineと
exactly equalであることもrequireし、不一致ならfieldを一切変更せずdedicated
typed errorを返します。success時は全baseline row/IDをpreserveし、sole
producer-created rowだけをappendします。どちらのinstallation orderでもerror時
でも2 outputの片方だけを公開してはいけません。

exact public installation surfaceは次のとおりです。

```rust
impl TypedAst {
    pub fn with_source_predicate_definition(
        self,
        projection: SourcePredicateDefinitionProjection,
    ) -> Result<Self, TypedAstError>;

    pub const fn source_predicate_definition(
        &self,
    ) -> Option<&SourcePredicateDefinitionHandoff>;
}

// existing non-exhaustive enumへ追加
TypedAstError::InvalidSourcePredicateDefinition
```

`TypedAstParts`はTask-259 fieldを追加せず、second install pathにはなりません。
authenticated baseline obligation tableをestablishする既存roleを維持し、上の
one-shot methodだけをsole Task-259 allocator/publication pathとします。

`ResolvedTypedAst`はrunnerから別途replace可能なinputを受けません。typed-owned
handoff、lower fingerprint、5 table、obligation rowを再検証してclone-preserve
します。empty debug renderingはbyte-stableを維持し、nonempty renderingは5つの
frozen keyでdeterministicにします。clone、rerun、equivalent inputの結果は
byte-identicalでなければなりません。

`ResolvedTypedAst::assemble(ResolvedTypedAstInputs<'_>)`はexisting signatureを
維持し、Task 259を`inputs.typed_ast`だけから取得します。次を追加します。

```rust
impl ResolvedTypedAst {
    pub const fn source_predicate_definition(
        &self,
    ) -> Option<&SourcePredicateDefinitionHandoff>;
}

// existing non-exhaustive enumへ追加
ResolvedTypedAstError::InvalidSourcePredicateDefinition
```

`ResolvedTypedAstInputs`へseparate Task-259 fieldは追加しません。handoff
debug textは`source-predicate-definition-debug-v1`で始まり、typed/final debug
outputはoptional handoffがpresentの場合だけそのexact validated textを含みます。

このboundaryはfact、VC、proof status、accepted definition、artifact、Core IR、
Control-Flow IRを一切commitしません。

## Dedicated consumer / trace intent

future implementationが追加するのは正確に次のものです。

- `tests/miz/pass/types/pass_type_elaboration_predicate_definition_payload_001.miz`
- `tests/miz/pass/types/pass_type_elaboration_predicate_definition_payload_001.expect.toml`
- new covered trace requirement
  `spec.en.checker.type_elaboration.source_predicate_definition_payload` 1件

trace rowは次のfieldで凍結します。

```toml
source = "doc/design/mizar-checker/en/source_predicate_definition.md"
section = "Dedicated Consumer And Trace Intent"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
```

sidecarは`schema_version = 1`、id
`pass_type_elaboration_predicate_definition_payload_001`、kind `pass`、
stage `type_elaboration`、domain `checker.type_elaboration`、上記source path、
expected outcome `pass`、expected phase `type_check`、空の
`diagnostic_codes`/`diagnostic_payloads`、tag `active_type_elaboration`、
`spec_refs`はfuture trace idだけ、と凍結します。failure category、rejection
reason、stable detail key、failure payloadは持ちません。

trace rowはimplementationとreal consumerが実行可能になった時だけ追加します。
既存の`fail_type_elaboration_predicate_functor_definition_gap_001.miz`、そのsidecar、
現在の全trace rowはbyte-identicalに保ちます。そのmixed predicate/functor
boundaryはTask 260 gateのままで、Task 259によってselect、promote、reinterpret
しません。

## Frozen tests

Task-259 checker implementationでは次の5 focused testを凍結します。

1. `task_259_exact_predicate_definition_payload_and_pending_obligation`
2. `task_259_independent_row_and_field_corruption_fails_closed`
3. `task_259_dependency_and_obligation_corruption_fails_closed`
4. `task_259_typed_installation_is_transactional`
5. `task_259_final_clone_debug_determinism_and_family_isolation`

exact `1/2/1/1/1` payload、全field/row/lower dependency/fingerprintの独立corruption、
obligationのstatus/goal/provenance/assumptions/owner/range/link、
partial/duplicate/reordered occupancy、atomic installation、immutable final clone、
deterministic debug、Tasks 253-255/257/258/260+、fact、proof、accepted statusからの
isolationをcoverします。

transactional testはvalid nonempty baselineも使います。全baseline row/IDの
byte-for-byte preservation、new IDが`baseline.len()`に等しいこと、全projection
getter/`into_parts`が両tableをpreserveすること、exact-baseline installation
successをproveします。independent stale/missing/extra/reordered/colliding
current-baseline mutationはTask-259 row、obligation replacement、partial replayを
残さず`InvalidSourcePredicateDefinition`を返さなければなりません。final
clone/debug testはbaseline/appended rowの両方をpreserveします。

runner implementationでは次の4 focused testを凍結します。

1. `task259_real_source_surface_resolver_and_lower_bundle_is_exact`
2. `task259_source_ast_resolver_and_lower_mutations_fail_at_the_owner`
3. `task259_expectation_selection_and_mixed_definition_route_stay_isolated`
4. `task259_route_publishes_no_property_proof_fact_or_acceptance`

全165 source bytes、71 Surface rows、raw resolver profile、lower count/association、
subtree exclusion、exact pass sidecar selection、mixed-route preservation、replay、
mutation ownership、proof acceptance不在を認証します。

## Exact implementation consumer / write scope

implementation logical taskのexact code/test consumerは次のとおりです。

- `crates/mizar-checker/src/source_predicate_definition.rs`: complete
  producer、5 row/table、handoff/projection、validation、debug grammar、
  focused checker test 5件
- `crates/mizar-checker/src/lib.rs`、
  `crates/mizar-checker/src/typed_ast.rs`、
  `crates/mizar-checker/src/resolved_typed_ast.rs`: public module、one-shot
  typed installation、final clone/revalidation、getter、dedicated error、
  exact debug inclusion
- `crates/mizar-checker/src/type_checker.rs`と
  `crates/mizar-checker/src/registration_resolution.rs`: new
  `InitialObligationKind`の既存exhaustive debug serializerだけ
- `crates/mizar-checker/tests/lint_policy.rs`: documented-module、
  public-enum、source/spec-audit allowlist。syntax-boundary testはcheckerの
  全`.rs` fileを自動scanするためtask-specific allowlist entryを必要としない
- `crates/mizar-test/src/runner.rs`、
  `crates/mizar-test/src/runner/type_elaboration.rs`、new private
  `crates/mizar-test/src/runner/type_elaboration/source_predicate_definition.rs`、
  `crates/mizar-test/src/runner/tests.rs`、new
  `crates/mizar-test/src/runner/tests/type_elaboration/source_predicate_definition.rs`:
  exact selection、syntax-free input construction、lower-stage composition、
  routing、focused runner test 4件
- `crates/mizar-test/tests/metadata.rs`: active type countの既存mechanical
  assertion 4件だけ
- **Dedicated Consumer And Trace Intent**で命名したnew `.miz` 1件、
  same-stem sidecar 1件、trace row 1件
- checker/mizar-test EN/JA crate plan、todo、module-boundary audit、checker
  EN/JA source/spec audit、このEN/JA module spec、
  `doc/design/spec_coverage_audit.md`のsynchronized Task-259
  implementation/closure record

`ResolvedTypedAst`はtyped-owned complete `InitialObligationTable`をprivateに
clone-preserveしてTask-259 correctness linkをrevalidateしますが、新しいpublic
table getterやsecond inputは追加しません。Cargo manifestの変更は想定しません。
既存specification、既存`.miz`、既存sidecar/expectation/trace row、mixed
Task-260 case、すべてのTask-260+ implementation ownerはwrite scope外です。
独立監査で発見したmechanical consumerはclassification/review後だけ追加でき、
languageまたはtest intentをbroadenしてはいけません。

## Deferral / forbidden scope

Task 259は次を禁止します。

- recursive predicate unfolding/evaluation。
- guard-to-property FOL VCの構築。
- property justificationのconsume/proof、proof search、discharge。
- `VcId`、fact、axiom、ATP premise、theorem publication。
- accepted obligation、accepted definition、semantic truth。
- overload candidate collection/winner selection。
- resolverからのsignature parameter、binder identity、arity推論。
- Task 259内での`BindingEnv`再構築またはTask 248 widening。
- Core IR、Control-Flow IR、VC lowering、artifact、public diagnostic。
- Task 260または後続のdefinition/property-proof/overload/redefinition/semantic owner。

Tasks 260-264、269-279、later advanced-semantics runner、全downstream semantic
stageは既存ownershipを維持します。

## Baseline、audit impact、exit

このdocumentation prerequisiteはproduction source、test、`.miz` fixture、
sidecar、trace row/status/count/backlink、coverage credit、runner selectionを変更
しません。frozen baselineは次のとおりです。

- plan/requirements `421/389`
- pass/fail `228/193`
- active parse/declaration/type/proof `101/7/198/1`
- declaration coverage: requirements 12 = covered 7 + partial 5
- type coverage: requirements 253 = covered 241 + deferred 12
- warnings/errors `23/0`

future pass sidecar 1件とcovered trace row 1件の追加後oracleは
plan/requirements `422/390`、pass/fail `229/193`、active
parse/declaration/type/proof `101/7/199/1`、type coverage requirements 254 /
covered 242です。これはexpected deltaであり、freshなcount/hash実測の代替では
ありません。

元のdocumentation taskはcanonical EN/JA同期、review、verification、dedicated
commit後にexitしました。別Task-248 documentation/implementation prerequisiteは
`f9b47375` / `ca54135f`で完了済みです。このcontract-correction prerequisiteは
EN/JA同期、findingsなしになるまでのrepeated review、docs-only verification、
exact staging、専用documentation commit 1件、protected stash不変かつcleanな
post-commit inventory後にのみexitします。次taskはTask-259 production
implementationです。
