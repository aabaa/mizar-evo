# source mode-definition transport

> canonical language は英語である。英語版:
> [../en/source_mode_definition.md](../en/source_mode_definition.md)。

## Task 262 の scope と authority

Checker Task 262 は、ordinary parameterized `mode` definition、その
normalized RHS の inhabitation request、explicit `sethood` correctness
clause を checker に渡す syntax-free / immutable intake 1 個を所有する。
canonical authority は Chapter 7 §§7.1--7.10、特に §§7.2、7.7、7.8、
definition correctness が obligation boundary であることだけに限った
Chapter 16 §§16.6、16.7.2、既存 mode-definition parser pass/recovery
fixture、active mixed mode/structure definition gap とその sidecar/trace、
および commit 済み Tasks 248--261 public transport である。

本 task は missing exact producer の `source_drift` と canonical authority
由来の exact consumer `test_gap` 1 個だけを閉じる。RHS が inhabited かを
判定せず、mode を accept せず、`sethood` を discharge せず、quantified
goal を合成せず、sethood/interface fact や concrete witness を publish
せず、expansion/registration を activate せず、proof/Core/ControlFlow/VC
payload を lower しない。

specification reviewはmandatory lower `source_drift`を1件検出した。committed
Task 249は全`SourceTypeApplicationInput`をsame ordinalのbinding-linked rowに
限定するため、definition parameter 2個はthird RHS applicationをownできない。
third bindingの捏造は`boundary_violation`である。Chapter 7のexplicit mode RHSは
separate checker-only Task 249M standalone mode-RHS tableをfreezeするauthority
として十分である。Task 249M documentation/implementationはTask 262
implementation前の別commitとし、本contractにlower changeを混在させない。

## 凍結する exact source

future active source は final LF を含めて exact に次のとおりである。

```mizar
definition
  let x be set;
  let y be set;
  mode Task262ModeDefinition: Task262Mode [x, y] is set;
  sethood by computation(steps: 1);
end;
```

141 bytes、6 lines、SHA-256 は
`3271f243670bd781c7167ff0d3bf463263a318abbe261aabdde1842c532a725e`
である。normal definition block 1 個、direct builtin-`set` context
parameter 2 個、ordered parameter occurrences `x, y` を持つ bracket-form
mode application 1 個、bare builtin-`set` RHS/expansion 1 個、computation
justification を持つ explicit `sethood` clause 1 個を含む。

`assume`、`equals`/`means` style、`->` return type、term/formula
definiens、attribute chain、imported/qualified name、structure、property
implementation、theorem、proof block、redefinition、notation、recovery は
存在しない。mode-definition separator は `is` だけである。RHS `set` が
mode の definiens/expansion であり、functor-style return type ではなく、
Task-249R definition-return row を生成しない。

## 凍結する literal Surface oracle

frontend diagnostic は 0、dense Surface row は exact 54 個である。root は
node 53、range `0..140`、normal である。row 0--33 は child を持たない
leaf token である。

| Node | Token | Range | Node | Token | Range |
| ---: | --- | --- | ---: | --- | --- |
| 0 | `definition` | `0..10` | 17 | `,` | `87..88` |
| 1 | `let` | `13..16` | 18 | `y` | `89..90` |
| 2 | `x` | `17..18` | 19 | `]` | `90..91` |
| 3 | `be` | `19..21` | 20 | `is` | `92..94` |
| 4 | `set` | `22..25` | 21 | `set` | `95..98` |
| 5 | `;` | `25..26` | 22 | `;` | `98..99` |
| 6 | `let` | `29..32` | 23 | `sethood` | `102..109` |
| 7 | `y` | `33..34` | 24 | `by` | `110..112` |
| 8 | `be` | `35..37` | 25 | `computation` | `113..124` |
| 9 | `set` | `38..41` | 26 | `(` | `124..125` |
| 10 | `;` | `41..42` | 27 | `steps` | `125..130` |
| 11 | `mode` | `45..49` | 28 | `:` | `130..131` |
| 12 | `Task262ModeDefinition` | `50..71` | 29 | `1` | `132..133` |
| 13 | `:` | `71..72` | 30 | `)` | `133..134` |
| 14 | `Task262Mode` | `73..84` | 31 | `;` | `134..135` |
| 15 | `[` | `85..86` | 32 | `end` | `136..139` |
| 16 | `x` | `86..87` | 33 | `;` | `139..140` |

row 34--53 は exact に次のとおりである。

| Node | Surface kind | Range | ordered children |
| ---: | --- | --- | --- |
| 34 | `TypeHead` | `22..25` | `[4]` |
| 35 | `TypeExpression` | `22..25` | `[34]` |
| 36 | `QualifiedVariableSegment` | `17..25` | `[2,3,35]` |
| 37 | `DefinitionParameter` | `13..26` | `[1,36,5]` |
| 38 | `TypeHead` | `38..41` | `[9]` |
| 39 | `TypeExpression` | `38..41` | `[38]` |
| 40 | `QualifiedVariableSegment` | `33..41` | `[7,8,39]` |
| 41 | `DefinitionParameter` | `29..42` | `[6,40,10]` |
| 42 | `ModePattern` | `73..91` | `[14,15,16,17,18,19]` |
| 43 | `TypeHead` | `95..98` | `[21]` |
| 44 | `TypeExpression` | `95..98` | `[43]` |
| 45 | `ComputationOption` | `125..133` | `[27,28,29]` |
| 46 | `ComputationJustification` | `113..134` | `[25,26,45,30]` |
| 47 | `JustificationClause` | `110..134` | `[24,46]` |
| 48 | `ModeProperty` | `102..135` | `[23,47,31]` |
| 49 | `ModeDefinition` | `45..135` | `[11,12,13,42,20,44,22,48]` |
| 50 | `DefinitionBlockItem` | `0..140` | `[0,37,41,49,32,33]` |
| 51 | `ItemList` | `0..140` | `[50]` |
| 52 | `CompilationUnit` | `0..140` | `[51]` |
| 53 | `Root` | `0..140` | tokens 0--33 followed by `[52]` |

private runner は syntax-free input 構築前に loaded byte、final LF、全 row
kind/range/recovery/ordered child、root identity、direct sibling order を
authenticate する。checker production は raw node kind/token/node number、
parser type、source text を受け取らない。

## 凍結する resolver provenance

resolver result は exact に shell 2 個、signature projection 1 個、symbol
diagnostic 0、mode symbol 1 個、mode definition 1 個、local-source
contribution 1 個である。

- shell 0 は `DefinitionBlock` node/range `50/0..140`、ordinal 0、parent
  なしである。
- shell 1 は `ModeDefinition` node/range `49/45..135`、ordinal 1、parent
  0 である。
- projection primary/notation spelling は exact に
  `Task262Mode [ x , y ]`、`SymbolKind::Mode`、`DefinitionKind::Mode`、
  syntactic arity なし、overloadable である。
- definition 0 は normal/local/public/exported/conflict-free、structural
  origin path `[4,0,10,0]` である。
- single contribution がその symbol と definition を所有する。

opaque parser signature role は `ModePattern`、`TypeExpression`、
`ModeProperty` である。resolver の `parameters`、`binders`、arity は
empty である。Task 262 はこの empty field から parameter declaration や
application argument を再構築せず、opaque signature text から
RHS/inhabitation/sethood semantics を推測せず、resolver success を
definition acceptance と扱わない。association owner は authenticated
Surface structure と lower handoff だけである。

## 凍結する lower bundle と ownership

separate Task-249M prerequisite後、exact sourceがconsumeするlower profileは
次だけである。

| Owner | exact profile | Task-262 ownership |
| --- | --- | --- |
| Task 248 | Profile B `1/2/2/2/2/2/0` | definition-block context と ordered bindings `x`, `y` |
| Task 249 | base applications/expressions/arguments `2/3/0` | parameter written typesはbinding-linked applications 0/1のまま、expression root 2はstandalone |
| Task 249M | mode-RHS rows `1` | fabricated bindingなしにexpression root 2とdefinition/RHS source identityをindependentにownする |
| Task 249R | absent | mode RHS は definition return ではない |
| Tasks 250--261 | absent | attribute/term/structure/set/formula/predicate/functor/attribute handoff なし |

全 Task-248 row は authenticated block node 50 に属する。parameter は
`BindingId(0/1)`、`SourceTypeApplicationId(0/1)`、shared definition context
`BindingContextId(1)` を使う。bracket application は ordered parameter-row
vector `[0,1]` を所有し、term row を作らず pattern occurrence
`86..87`/`89..90` を authenticate する。expansion と inhabitation request
はいずれも canonical expression root 2がnode/range `44/95..98` の
`SourceTypeModeRhsId(0)` に link する。

Task 262 が fingerprint するのは Task 248 `source_context` と Task 249+
Task 249M `source_type` のone handoff fingerprintだけである。source-term/
application-term/structure-term/
set-term/atomic-formula/composite-formula/return-type/attribute/
evidence-response fingerprint はない。runner は既存 lower input を
compose してよいが、lower producer を変更したり missing row を捏造して
はならない。

separate Task-249M contractは`SourceTypeModeRhsId`、
`SourceTypeModeRhsInput`/immutable row/table各1個、exact-one-row extension
producer、`SourceTypeApplicationHandoff::mode_rhs()`、handoff deterministic
`debug_text()`のmode-RHS lineをfreezeする。rowはdefinition site/range、source
ordinal、expression root 2をownし、Task 249Rのindependent ownership shapeを
mirrorするがreturn semanticsをreuseしない。Task 262はone
`&SourceTypeApplicationHandoff`を受け、`mode_rhs().get(0)`/root 2をvalidateし、
complete source-type fingerprintがTask-249M rowをcoverする。exact names/
fields/errors/debug grammarとlower test 4個はseparate Task-249M frozen contract/
commitがownする。

## exact public syntax-free input

implementation は 6 個の dense ID family を持つ
`source_mode_definition.rs` を追加する。各 ID は `Copy + Eq + Ord + Hash`
で `new` と `index` だけを公開し、vector order で allocate する。

```rust
pub struct SourceModeDefinitionId(usize);
pub struct SourceModeParameterId(usize);
pub struct SourceModeApplicationId(usize);
pub struct SourceModeExpansionId(usize);
pub struct SourceModeInhabitationRequestId(usize);
pub struct SourceModePropertyId(usize);
```

exact public input は英語 canonical 文書に示す次の型で固定する。

```rust
pub struct SourceModeDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceModeDefinitionInput>,
    pub parameters: Vec<SourceModeParameterInput>,
    pub applications: Vec<SourceModeApplicationInput>,
    pub expansions: Vec<SourceModeExpansionInput>,
    pub inhabitation_requests: Vec<SourceModeInhabitationRequestInput>,
    pub properties: Vec<SourceModePropertyInput>,
}

pub struct SourceModeDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
    pub application: SourceModeApplicationId,
    pub expansion: SourceModeExpansionId,
    pub inhabitation_request: SourceModeInhabitationRequestId,
    pub property: Option<SourceModePropertyId>,
}

pub struct SourceModeParameterInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub pattern_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceModeApplicationInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub parameters: Vec<SourceModeParameterId>,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceModeExpansionInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub rhs: SourceTypeModeRhsId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceModeInhabitationRequestInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub expansion: SourceModeExpansionId,
    pub kind: SourceModeInhabitationRequestKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceModePropertyInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub kind: SourceModePropertyKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

#[non_exhaustive]
pub enum SourceModeInhabitationRequestKind { Rhs }

#[non_exhaustive]
pub enum SourceModePropertyKind { Sethood }

#[non_exhaustive]
pub enum SourceModeDefinitionRecovery { Normal, Degraded }
```

全 input struct は `Debug + Clone + PartialEq + Eq`、recovery/request/
property enum は `Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord +
Hash` を derive する。public input は `SemanticOrigin`、fingerprint、allocated
dense ID、`InitialObligationId`、result/evidence status、accepted fact、
formula、proof、VC を受け取らない。

## exact immutable output と public API

immutable row typeとstored fieldのAPI orderは次である。

| Row | stored fields |
| --- | --- |
| `SourceModeDefinition` | `id`, `symbol`, `definition`, `contribution`, `site`, `source_range`, `source_ordinal`, `context`, `recovery`, `spelling`, `application`, `expansion`, `inhabitation_request`, `property`, derived `origin` |
| `SourceModeParameter` | `id`, `owner`, `ordinal`, `binding`, `written_type`, `site`, `source_range`, `declaration_range`, `pattern_range`, `context`, `recovery`, `spelling` |
| `SourceModeApplication` | `id`, `owner`, `ordinal`, `parameters`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourceModeExpansion` | `id`, `owner`, `ordinal`, `rhs`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourceModeInhabitationRequest` | `id`, `owner`, `ordinal`, `expansion`, `kind`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourceModeProperty` | `id`, `owner`, `ordinal`, `kind`, `site`, `source_range`, `justification`, `recovery`, `spelling`, derived `obligation` |

全stored fieldはsame-named read-only getter 1個を持つ。Copy ID/enum/range/
ordinal/context/optional propertyはvalue、`parameters()`は
`&[SourceModeParameterId]`、symbol/site/origin/justificationはshared reference、
`spelling()`は`&str`を返す。public row constructor/setter/mutable getter/
replacement APIはない。

exact table/handoff surfaceは次である。

```rust
pub struct SourceModeDefinitionTable { /* private rows */ }
pub struct SourceModeParameterTable { /* private rows */ }
pub struct SourceModeApplicationTable { /* private rows */ }
pub struct SourceModeExpansionTable { /* private rows */ }
pub struct SourceModeInhabitationRequestTable { /* private rows */ }
pub struct SourceModePropertyTable { /* private rows */ }

pub struct SourceModeDefinitionHandoff { /* private fields */ }

impl SourceModeDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn source_context_fingerprint(&self) -> &str;
    pub fn source_type_fingerprint(&self) -> &str;
    pub const fn base_initial_obligation_count(&self) -> usize;
    pub const fn definitions(&self) -> &SourceModeDefinitionTable;
    pub const fn parameters(&self) -> &SourceModeParameterTable;
    pub const fn applications(&self) -> &SourceModeApplicationTable;
    pub const fn expansions(&self) -> &SourceModeExpansionTable;
    pub const fn inhabitation_requests(
        &self,
    ) -> &SourceModeInhabitationRequestTable;
    pub const fn properties(&self) -> &SourceModePropertyTable;
    pub fn debug_text(&self) -> String;
}
```

各tableは`get(id) -> Option<&Row>`、source-ordered
`iter() -> impl Iterator<Item = (Id, &Row)>`、`const len() -> usize`、
`const is_empty() -> bool`だけを公開する。fingerprint 2個はcomplete lower
`debug_text()`でcaller-suppliedではない。

exact projection/error/producer ABIは次である。

```rust
pub struct SourceModeDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourceModeDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourceModeDefinitionProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable;
    pub const fn handoff(&self) -> &SourceModeDefinitionHandoff;
    pub const fn initial_obligations(&self) -> &InitialObligationTable;
    pub fn into_parts(self) -> (
        InitialObligationTable,
        SourceModeDefinitionHandoff,
        InitialObligationTable,
    );
}

#[non_exhaustive]
pub enum SourceModeDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidApplication { index: usize },
    InvalidExpansion { index: usize },
    InvalidInhabitationRequest { index: usize },
    InvalidProperty { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

pub struct SourceModeDefinitionProducer;

impl SourceModeDefinitionProducer {
    pub fn build(
        input: SourceModeDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<SourceModeDefinitionProjection, SourceModeDefinitionError>;
}
```

output row/table/handoff/projection/errorは`Debug + Clone + PartialEq + Eq`、
errorは`Display`/`std::error::Error`をimplementし`Default`/blanket conversionを
持たない。producerはunit structである。

## Public Enum Policy

| public enum | compatibility policy |
| --- | --- |
| `SourceModeInhabitationRequestKind` | `#[non_exhaustive]`; later request kindにはseparately frozen ownerが必要 |
| `SourceModePropertyKind` | `#[non_exhaustive]`; later mode propertyにはcanonical authority/testが必要 |
| `SourceModeDefinitionRecovery` | `#[non_exhaustive]`; callerはlater recovery classをtolerateする |
| `SourceModeDefinitionError` | `#[non_exhaustive]`; callerはvalidation failureをexhaustive matchしない |

この module が所有する exhaustive public enum exception はない。

## exact active row と cardinality

active transaction は exact に `1/2/1/1/1/1` である。

| Table | exact active row |
| --- | --- |
| definition 0 | resolver symbol/definition/contribution 0、site node 49、range `45..135`、source ordinal 0、context 1、spelling bytes `mode Task262ModeDefinition: Task262Mode [x, y] is set;\n  sethood by computation(steps: 1);`、application/expansion/request 0 と `Some(property 0)` |
| parameters 0/1 | owners 0、ordinals 0/1、bindings 0/1、written types 0/1、sites 37/41、ranges `13..26`/`29..42`、declaration ranges `17..18`/`33..34`、pattern ranges `86..87`/`89..90`、context 1、spellings `let x be set;`/`let y be set;` |
| application 0 | owner 0、ordinal 0、parameter IDs `[0,1]`、site 42、range `73..91`、context 1、spelling `Task262Mode [ x , y ]` |
| expansion 0 | owner 0、ordinal 0、RHS mode-type row 0 / expression root 2、site 44、range `95..98`、context 1、spelling `set` |
| request 0 | owner 0、ordinal 0、expansion 0、kind `Rhs`、site 44、range `95..98`、context 1、spelling `set` |
| property 0 | owner 0、ordinal 0、kind `Sethood`、site 48、range `102..135`、justification node/range `46/113..134`、spelling `sethood by computation(steps: 1);` |

immutable output row はこれらを copy し dense ID を追加し、definition
だけに resolver-derived `SemanticOrigin` を追加する。property output は
derived `InitialObligationId` も所有する。table は `len`、`is_empty`、
`get`、ordered `iter` だけ、row は read-only accessor だけを公開する。
handoff は source/module identity、resolver identity、lower fingerprint 2
個、table 6 個を保持する。input order は canonical で sort/repair しない。

projection は baseline の exact clone、handoff、updated obligation table を
保持し、TypedAst の compare-and-swap を可能にする。

## inhabitation request と initial obligation boundary

Chapter 7 は RHS inhabitation を mandatory とするが、本 task には accepted
evidence response または base-shape evaluator がない。request 0 は
expansion 0 が RHS inhabitation evidence を必要とすることだけを記録する。
result、availability、witness、diagnostic、acceptance field は持たない。
特に source spelling `set` から Chapter-17 base-shape table の照会や
definition acceptance を主張してはならない。

`b`をexact baseline lengthとする。projectionはbaselineのbyte-identical cloneを
保持し、`[0,b)`の全row/ID/orderを保ち、`InitialObligationId(b)`にexact 1 rowを
appendする。ordinary existing-kind `Sethood`を含むarbitrary unrelated baseline
obligationはallow/preserveする。sibling-only kind
`PredicatePropertyCorrectness`、`FunctorExistence`、`FunctorUniqueness`を持つ
baselineはTasks 259/260とcoexistできないのでrejectする。property 0はappended
IDだけへlinkする。

| Field | exact value |
| --- | --- |
| `id` | `InitialObligationId(b)` |
| `kind` | existing `InitialObligationKind::Sethood` |
| `owner` | property site node 48 |
| `source_range` | `102..135` |
| `assumptions` | empty |
| `goal` | `source.definition.mode.correctness:definition=0:sethood` |
| `provenance` | `source.definition.mode:definition=0:property=0` |
| `status` | `InitialObligationStatus::Pending` |

empty assumption vector は representation に限られ、unguarded FOL
obligation を意味しない。Chapter 7 `ParamGuard` construction、quantified
goal、parameter tuple ごとの witness-set dependence、proof checking、
computation execution、discharge、acceptance、exported/private semantic fact
はすべて deferred である。justification anchor は provenance に限る。
separate mode-existence `InitialObligationKind` は発明せず、authorized
evidence consumer が存在するまで mandatory existence check は request 0
だけで表す。

updated table lengthはexact `b + 1`である。ID `b`より後のsuffix row、ID `b`
以外のproperty link、second Task-262 goal/provenance rowはkindが`Sethood`でも
invalidである。final validationはkind-wideでなくlink/prefix-basedである。
handoffありではproperty 0 -> ID `b`とsingle exact suffixをvalidateしunrelated
baseline `Sethood`をpreserveする。handoffなしではgoal/provenanceが
`source.definition.mode` domainのorphan rowをrejectする。general `Sethood`
obligationをkindだけでclaim/rejectしない。

## validation、determinism、failure atomicity

wrong source/module/arena、non-dense/wrong cardinality、missing/duplicate/
reordered/dangling/cross-owner/cross-context row、wrong binding/type/
application/expansion/request/property association、wrong site/range/ordinal/
spelling/kind、recovered/degraded row、stale resolver symbol/definition/
contribution/origin、stale lower fingerprint、wrong baseline、pre-existing
appended row、wrong obligation owner/range/kind/text/status、partial/extra row
を reject する。exact shape について `[0,1]` 以外の parameter vector と
Task-249M mode-RHS row 0 / expression root 2以外のRHSもrejectする。

production は output allocate 前に complete input を authenticate する。
error は partial handoff を返さず、input baseline/lower handoff/env/arena を
mutate しない。`debug_text()` は `source-mode-definition-debug-v1` で開始
し、fingerprint 2 個と table 6 個を ID order で render し、linked
obligation ID を含み、LF 1 個で終わる。repeated build/clone/typed install/
resolved assembly/rendering は byte-deterministic である。empty legacy
debug output は byte-identical のままである。

exact debug grammarはblank lineなしで次のとおりである。`Rust-debug`はstandard
escaped `{:?}` output、active siteはnode site、active rowはnormalである。

```text
source-mode-definition-debug-v1
module: <ModuleId.path>
source-context-fingerprint: <Rust-debug String>
source-type-fingerprint: <Rust-debug String>
base-initial-obligation-count: <n>
definition#<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> ordinal=<n> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> origin_range=<start>..<end> origin_path=<Rust-debug [u32]> spelling=<Rust-debug String> application=<id> expansion=<id> inhabitation_request=<id> property=<none|id>
parameter#<id> owner=<id> ordinal=<n> binding=<id> written_type=<id> range=<start>..<end> declaration_range=<start>..<end> pattern_range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
application#<id> owner=<id> ordinal=<n> parameters=<Rust-debug [usize]> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
expansion#<id> owner=<id> ordinal=<n> rhs=<id> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
inhabitation-request#<id> owner=<id> ordinal=<n> expansion=<id> kind=<rhs> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
property#<id> owner=<id> ordinal=<n> kind=<sethood> range=<start>..<end> site=node#<id> justification=range:<start>..<end> recovery=<normal|degraded> spelling=<Rust-debug String> obligation=<id>
```

## Typed と final ownership

`TypedAst` は optional field 1 個と one-shot installer 1 個を追加する。

```rust
pub fn with_source_mode_definition(
    self,
    projection: SourceModeDefinitionProjection,
) -> Result<Self, TypedAstError>;

pub const fn source_mode_definition(
    &self,
) -> Option<&SourceModeDefinitionHandoff>;

TypedAstError::InvalidSourceModeDefinition
```

installer は source context/type を必須とし、current obligation table と
projection baseline を比較し、handoff と appended row 1 個を validate し、
prior Task-262 occupancy を reject し、全 check pass 後だけ handoff と
updated obligations を publish する。Task 259/260/261 をすでに持つ AST
も reject する。`TypedAstParts` に Task-262 field/alternate path はない。

`ResolvedTypedAst::assemble` は typed owner からのみ Task 262 を取得し、
final lower handoff/obligations に対して clone-preserve/revalidate して、
次だけを追加する。

```rust
pub const fn source_mode_definition(
    &self,
) -> Option<&SourceModeDefinitionHandoff>;

ResolvedTypedAstError::InvalidSourceModeDefinition
```

`ResolvedTypedAstInputs` に replaceable field はない。final assembly は
older sibling installer が知らない reverse install order を含むすべての
mixed Task-259/260/261/262 state を reject する。Task 262 は Task 259
predicate correctness、Task 260 functor existence/uniqueness、Task 261
no-obligation boundary を弱めず、merge/reinterpret しない。

## dedicated runner consumer と trace intent

implementation は active pass pair を exact 1 個追加する。

- `tests/miz/pass/types/pass_type_elaboration_mode_definition_payload_001.miz`
- `tests/miz/pass/types/pass_type_elaboration_mode_definition_payload_001.expect.toml`

sidecar は `pass` / `type_elaboration` / `type_check`、public diagnostic と
payload は empty、requirement
`spec.en.checker.type_elaboration.source_mode_definition_payload` だけを cite
する。new required covered trace row 1 個がその sidecar だけを reciprocal
に cite する。これは transport pass であり、mode、RHS evidence、
computation、sethood proof、interface fact の acceptance を意味しない。

private runner route は exact source/hash、全54 Surface row、両 shell、
resolver projection/symbol/definition/contribution、Tasks-248/249 profile と
fingerprint を authenticate してから checker を呼ぶ。generic mixed
mode/structure gap より前に select する。既存
`fail_type_elaboration_mode_structure_definition_gap_001.miz`、sidecar、trace
row は structure half が Task 263 所有なので byte-identical に保つ。
parser pass/recovery fixture も変更しない。

frozen checker test は exact に次の 5 個である。

1. `task_262_mode_definition_exact_payload_and_obligations_are_deterministic`
2. `task_262_mode_definition_row_field_corruption_fails_closed`
3. `task_262_mode_definition_dependency_and_obligation_corruption_fails_closed`
4. `task_262_mode_definition_typed_installation_is_transactional`
5. `task_262_mode_definition_final_clone_debug_determinism_and_family_isolation`

frozen runner test は exact に次の 4 個である。

1. `task262_mode_definition_source_consumer_is_exact`
2. `task262_mode_definition_surface_resolver_lower_and_payload_corruption_fail_closed`
3. `task262_mode_definition_selection_and_family_isolation_are_exact`
4. `task262_mode_definition_justification_and_semantic_subtrees_are_not_published`

mutation は literal bytes/final LF、全 structural table、resolver origin/
identity、parameter/application order、lower fingerprint、request/property
association、obligation baseline/appended row、unrelated baseline `Sethood`
preservation、sibling-only kind rejection、linked-suffix/orphan rejection、
typed/final transaction、immutable clone、debug determinism、sibling-family/
route isolation、non-publication を独立に cover する。

## count、hash、audit、write scope

documentation prerequisite は Rust、fixture、sidecar、expectation、trace
row/status/backlink/count、test list、production path、Cargo metadata、CLI
output、recorded hash を変更しない。current executable baseline は
checker/runner/resolver/syntax `449/520/144/59`、metadata cases/requirements
`424/392`、pass/fail `231/193`、active parse/declaration/type/proof
`101/7/201/1`、type requirements `256/244`、warnings/errors `23/0` である。

separate Task-249M implementationはchecker test 4個でchecker `449 -> 453`、
corpus/runner deltaなしをprojectする。その後Task-262 implementationはchecker
`453 -> 458`、runner `520 -> 524`、new
case/requirement/pass/active-type/covered-type 各 1、metadata `425/393`、
`232/193`、`101/7/202/1`、type `257/245` を project する。resolver/syntax
は `144/59` のままである。production manifest、test-list hash、5 CLI
hash、fixture/sidecar/trace hash は予測せず fresh-measure する。

later implementation write scope は次に限定する。

- checker mode-definition module、export、TypedAst/final ownership、lint/
  source-spec inventory、frozen checker tests 5 個
- private runner leaf/test leaf 各 1 個と bounded facade/root registration
- new pass source/sidecar 1 pair と reciprocal covered trace row 1 個
- synchronized EN/JA plan/TODO/ledger/module/source/trace/spec-coverage audit

`doc/spec`、existing `.miz`、existing expectation/sidecar、parser/resolver
production、lower checker producer、Task-249M behavior、Task-259/260/261
semantic behavior、Core、VC、kernel、unrelated metadata は変更しない。

## explicit semantic deferral と exit criteria

deferred semantics は RHS evidence lookup/response、base-shape-table result、
attribute-chain inhabitation、registration order/activation、definition
acceptance/symbol activation、use-site mode-application checking、expansion
fact/normalization、`ParamGuard` construction、quantified existence/sethood
FOL、computation/proof/discharge、witness handling、exported/private sethood
fact、property implementation/coherence、fact/axiom、CoreIr、ControlFlowIr、
VC、全 mixed definition-family meaning である。Task 263 は structure
definition、Task 264 は property implementation を所有する。

Task 262 の完了条件は次である。

- prerequisite contract と全 synchronized audit の repeated review-only
  specification review が NO FINDINGS になり単独 commit される。
- separate Task-249M documentation/implementation commitがown review/hard
  gate/verification/staging/post-commit inventoryをPASSする。
- post-Task-249M fresh inventoryで追加lower changeなしにexact Task-262 sourceが
  dependency-readyと再確認される。
- implementation が上記全 row/association/request/obligation/fingerprint/
  ownership/isolation/consumer/mutation/exclusion と一致する。
- separate test-sufficiency、implementation、source/documentation review が
  NO FINDINGS になる。
- protocol hard gate 9 個が score cap なしですべて PASS、final read-only
  quality が 90/100 以上になる。
- focused/crate/library/metadata/lint/fmt/Clippy/workspace/CLI/count/hash/
  whitespace verification が pass する。
- Task-262 file だけを stage/commit し、clean HEAD/origin/stash inventory
  後に dependency-order Task 263 へ直接戻る。

## Task 249M lower-contract link

upper contractは`8c3fa20acef42477d38a66ddddec42dacced0863`としてcommit済み。
exact lower ABI/error precedence/debug grammar/`2/3/0/0/1` profile/test 4件は
[`source_type.md`](./source_type.md)の「Task 249M frozen standalone mode-RHS
extension」にcanonical freezeした。separate docs prerequisite/implementationが
commitされfresh inventoryがfingerprint seamを確認するまでTask-262
implementation authorityを与えない。

Completion evidence: [central Task-249M historical contract](../../task_contracts/ja/249M.md#completion-evidence)。

## Task 262 active implementation result

frozen six-table producerは`1/2/1/1/1/1`でactiveである。exact source/resolver
identityとTask-248/249/249M handoffをauthenticateし、そのfingerprintを保持し、
RHS requestをunresolvedのままにしてauthenticated baselineへPending `Sethood`
obligationをexact 1件appendする。Typed/final ownerはimmutable cloneだけをpublishし、
mixed Tasks 259--262、stale dependency、invalid link、orphan mode-domain rowを
rejectする。

exact pass pairとsole reciprocal covered trace rowはactiveである。checker 5件と
runner 4件のtestがfrozen corruption/transaction/final-assembly/isolation/
non-publication matrixを保護する。active countはchecker/runner/resolver/syntax
`458/524/144/59`、metadata `425/393`、pass/fail `232/193`、active stages
`101/7/202/1`、type coverage `257/245`、warnings/errors `23/0`である。上記の
semantic deferralは全て不変であり、Task 263はdedicated commit後のfresh
inventoryでのみselectする。
