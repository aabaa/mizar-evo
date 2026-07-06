# mizar-checker: registration resolution

> 正本は英語です。英語版:
> [../en/registration_resolution.md](../en/registration_resolution.md)。

## 目的

`registration_resolution` は、phase-6 type checking が `TypedAst`、
normalized type、type fact を生成した後に走る phase-7 registration handling を
仕様化する。これは次を精緻化する。

- [architecture 04](../../architecture/ja/04.type_and_registration_resolution.md)
  Step 5 "Resolve Registration Closure";
- [architecture 04](../../architecture/ja/04.type_and_registration_resolution.md)
  Step 6 "Validate Pending Registration Declarations";
- [architecture 17](../../architecture/ja/17.cluster_trace_format.md) の
  replayable cluster / reduction trace 要件;
- [spec chapter 17](../../../spec/ja/17.clusters_and_registrations.md) の
  existential、conditional、functorial、reduction registration;
- [`typed_ast.md`](./typed_ast.md) の fact、obligation、diagnostic、recovery table;
- [`type_checker.md`](./type_checker.md) の normalized type、coercion、fact-query
  contract。

task 13 は specification-only task である。Rust source、active registration
inference、`ResolutionTrace` 実装、verifier policy、`VcId` assignment、proof
acceptance は追加しない。task 14 と task 16-20 が以下の named section を実装する。

## 境界

`registration_resolution` が所有するもの:

- pending registration declaration と activated registration の checker-side 分離;
- checker 境界内の deterministic registration id、index、status record、
  diagnostic、source-contribution tracking;
- checker-ready registration payload について、well-formed pattern、互換性の
  ある referenced symbol、parameter typing、stable provenance を検証すること;
- `existence`、`coherence`、`reducibility` 条件に対する checker-local
  `InitialObligationId` の発行;
- activated conditional / functorial registration 上の cluster fact closure;
- attribute 付き型使用の existential gate;
- activated reduction による reduction normalization と、canonical
  `ResolutionTrace` に十分な provenance;
- recoverable registration error 後の deterministic partial output と diagnostic。

`registration_resolution` が所有しないもの:

- registration syntax の parser / resolver extraction;
- resolver symbol allocation、label lookup、visibility、import/export check、
  opaque signature-shell construction;
- activated registration summary の public artifact schema;
- `VcId`、`ObligationAnchor`、VC generation、ATP search、proof acceptance、
  verifier policy decision、kernel replay;
- final overload root selection、active refinement joining、inserted
  overload-disambiguating view;
- checker diagnostic code-space が外部 planning gate である間の public
  diagnostic-code allocation。

automatic type fact、reduction step、attribute 付き型の existence gate に寄与してよいのは
activated registration だけである。pending、malformed、unverified、failed、
externally blocked な registration は diagnostic や local metadata に記録してよいが、
発火したり gate を満たしたりしてはならない。

## 入力と出力

registration resolver は次を消費する。

- phase 6 からの `TypedAst` 1 つ;
- `type_checker` からの normalized type entry と type fact;
- `TypeFactQueryEngine` の deterministic fact query;
- origin / identity anchor としての resolver `SymbolEnv` registration declaration;
- resolver/source-extraction task が供給する場合の checker-ready registration payload;
- artifact/reuse integration が accepted summary を供給する場合の activated dependency
  registration summary;
- 後続 proof/artifact phase が公開する場合に限る accepted verifier-status input。

registration resolver は checker-local phase-7 output を生成する。

```rust
struct RegistrationResolutionOutput {
    typed_ast: TypedAst,
    registrations: RegistrationDatabase,
    facts: TypeFactTable,
    trace: ResolutionTrace,
    diagnostics: TypeDiagnosticTable,
}
```

この形は論理的なものである。task 14 は、task 15 が concrete trace module を実装する
前に database を実装してよい。この文書の phase-7 output は stable published
artifact schema ではない。

## registration data model

checker は 2 つの distinct store を維持する。

```rust
struct RegistrationDatabase {
    pending: PendingRegistrationTable,
    activated: ActivatedRegistrationIndex,
    rejected: RejectedRegistrationTable,
    diagnostics: TypeDiagnosticTable,
    initial_obligations: InitialObligationTable,
}

struct PendingRegistration {
    id: CheckerRegistrationId,
    resolver_registration: RegistrationId,
    symbol: Option<SymbolId>,
    label: LabelRef,
    kind: RegistrationKind,
    pattern: RegistrationPatternStatus,
    parameters: Vec<TypedRegistrationParameter>,
    correctness: CorrectnessCondition,
    source: RegistrationSource,
    contribution: SourceContributionId,
    status: PendingRegistrationStatus,
    obligations: Vec<InitialObligationId>,
}

struct ActivatedRegistration {
    id: CheckerRegistrationId,
    resolver_registration: RegistrationId,
    label: LabelRef,
    kind: RegistrationKind,
    validation_kind: Option<RegistrationValidationKind>,
    pattern: RegistrationPattern,
    parameters: Vec<TypedRegistrationParameter>,
    correctness: AcceptedCorrectness,
    source: RegistrationSource,
    contribution: SourceContributionId,
    activation: ActivationEvidence,
}
```

`RegistrationPatternStatus` は usable checker-ready payload と、missing、
malformed、unsupported、external-gap payload を区別する。`ActivationEvidence` は
obligation creation から checker が生成するものではない。accepted dependency summary
または後続 phase が供給した accepted verifier status を参照しなければならない。

checker-side `ActivatedRegistrationIndex` は resolver-side `RegistrationIndex` ではない。
resolver index は checker validation 前に declaration identity、symbol link、kind、
target shell、visibility/export metadata、dependency mention、recovery state、source
contribution を供給する。checker index は typed pattern と accepted correctness
evidence を持つ accepted registration だけを含む。task 20 以降、activated record は
activation input または valid companion validation payload が供給する場合に accepted checker
validation kind も保持する。existential gate のように cluster subkind を要求する consumer
は、caller-supplied candidate tag を信用せず、この field を確認しなければならない。

## pending database と activated database

pending table は、現在の module で見つかった registration declaration、またはまだ
accepted local summary になっていない imported registration declaration を記録する。
pending record は diagnostic、`InitialObligationId`、source contribution link、後で
active になるための stable identity を保持してよい。automatic inference には決して使わない。

activated index は次だけを含む。

- verified summary を通じて import された accepted dependency registration;
- prior accepted pass または explicit accepted verifier-status input から得られた
  accepted local registration;
- well-formedness、correctness obligation、activation evidence が欠けている
  registration は含めない。

activation は item-ordered かつ deterministic である。registration は、自身の
correctness condition が accepted になり、その pass の checker input が accepted status
を含む場合だけ、後続 item に対して active になってよい。より前の item は、その
registration が最初から active だったかのように retroactive に再検査されない。
accepted proof/artifact input のない単一 pass では、checker が initial obligation の
発行に成功しても、新しい local registration は pending のままである。この accepted input
不在状態は、spec 17.1 の asynchronous acceptance 契約に対する暫定的な保守近似である。
module または依存する use site を pending にしてよいが、先行する local registration が
後続 item を決して正当化できないという最終的な言語規則ではない。`mizar-vc`、
`mizar-proof`、artifact integration が source order 上の accepted status を供給したら、
完了した checker pass は受理済み local registration を後続 item に対して activate し、
以前の item は retroactive に扱わない。

順序要件:

1. resolver-origin id と source contribution id を保持する。
2. activated trigger list は canonical trigger key、origin module path、declaration
   source order、label FQN、registration kind、fingerprint の順で sort する。
3. pending / rejected table は source contribution、declaration order、registration
   kind、label の順で render する。
4. hash-map iteration、worker order、import order、cache insertion order が firing
   order や diagnostic に影響してはならない。

### task 14: registration index data layer

task 14 はこの section を `src/registration_resolution.rs` として実装する。

最初の実装は、resolver `SymbolEnv` registration declaration から checker-owned な
pending、activated、rejected、diagnostic table を構築する。resolver registration は
identity / provenance record としてだけ扱う。checker は resolver registration id、
optional symbol id、resolver registration kind、opaque target-shell classification、
visibility、export status、normalized origin、source contribution、dependency、recovery
state を保持する。

accepted checker-owned activation input がない resolver entry から作られる pending
record は `external_dependency_gap` として印付けされ、automatic fact、reduction、
existential gate に決して寄与しない。malformed resolver target shell は rejected record
になる。activated record は、resolver kind、trigger key、accepted checker-owned pattern
key、accepted correctness key、activation evidence key、そして後続 task で追加された
subkind-sensitive consumer については accepted checker validation kind を含む explicit
caller-supplied activation input を通じてだけ作成できる。activation evidence だけでは不十分である。

task 14 は opaque resolver target shell の parse、semantic registration pattern の
validation、`InitialObligationId` の作成、proof acceptance、artifact summary の読み取り、
cluster closure、reduction 適用、existential gate の充足、`ResolutionTrace` step の生成を
行わない。後続 task は task-14 data layer を消費してよいが、explicit checker-owned
payload seam が利用可能になるまでは MC-G021 payload を external として扱い続けなければならない。

task 14 の canonical order:

1. pending / rejected record は source contribution id、origin structural path、
   resolver registration id、resolver registration kind、label/symbol fallback key、
   必要に応じて rejection reason の順で sort する;
2. activated trigger list は trigger key、origin module path、origin structural path、
   resolver registration id、label/symbol fallback key、resolver registration kind、
   fingerprint または pattern fallback key、checker registration id の順で sort する;
3. debug rendering は同じ checker-owned order を使い、resolver map や worker iteration
   order に依存しない。

## validation obligation

task 19 が validation を実装する。validation は checker-ready registration payload と
resolver declaration identity から始まる。

必須 check:

- resolver declaration が存在し、checker payload と互換性のある registration kind を
  持つ;
- label が registration label として解決され、stable checker registration id に束縛できる;
- 参照される attribute、mode、structure、functor、term、type head が互換性のある
  symbol に解決される;
- surrounding registration parameter が well typed であり、その local fact は context
  visibility rule を通じてだけ利用可能である;
- existential pattern は 1 つ以上の attribute を持つ attributed normalized type を含む;
- conditional pattern は互換性のある normalized type 上の antecedent / consequent
  attribute set を含む;
- functorial pattern は typed functor result pattern と declared result type 上の
  consequent attribute を含む;
- reduction pattern は typed `LHS` / `RHS` term を含み、`RHS` の free pattern variable
  はすべて `LHS` に free に出現し、variable occurrence count は増加せず、spec 17.6.4
  の固定 simplification order で `LHS` が `RHS` より strictly larger である;
- diagnostic、trace replay、dependency fingerprint 用の source provenance と source
  contribution id が存在する。

validation は pending registration と 1 つ以上の `InitialObligationId` を出力する。

| registration kind | correctness condition | checker-local obligation |
|---|---|---|
| Existential | `existence` | attribute 付き型の inhabitant witness |
| Conditional | `coherence` | antecedent attribute が consequent attribute を含意する |
| Functorial | `coherence` | matched functor result が consequent attribute を持つ |
| Reduction | `reducibility` | universally quantified equality `LHS = RHS` |

initial obligation の作成は、それを discharge しない。checker は local
`InitialObligationId` の代わりに `VcId`、prover output、kernel acceptance、accepted
verifier status を保存してはならない。

validation failure は rejected pending record または diagnostic 付き degraded pending
record を生成する。activated registration は生成せず、cluster closure、reduction
normalization、existential gating に影響しない。

### task 19: pending validation と activation gating

task 19 は、この section を `src/registration_resolution.rs` の explicit-payload data
layer として実装する。checker は resolver registration id で key 付けされた
caller-supplied `RegistrationValidationInput` payload を消費する。resolver の opaque
target shell を parse したり、raw syntax を walk したり、artifact summary を読んだり、
欠けている checker-ready registration payload を推測したりしない。

accepted activation evidence がない場合、validated payload は
`RegistrationPatternStatus::Validated(...)` を持つ pending
registration、`InitialObligationTable` 内の checker-local `InitialObligationId`、
`PendingRegistrationStatus::AwaitingVerifierAcceptance` を作る。これらは still pending
であり、常に `inference=false` を報告し、cluster fact、reduction step、existential gate
に寄与できない。発行された obligation は
`InitialObligationKind::RegistrationCorrectness` を使い、
`InitialObligationStatus::Pending` のままである。task 19 は `VcId`、verifier result、
proof witness、kernel acceptance、public artifact id を割り当てない。

checker は existential、conditional、functorial、reduction payload shape を
checker-owned field で検証する。existential、conditional、functorial payload は resolver
の `Cluster` declaration を要求し、reduction payload は resolver の `Reduction`
declaration を要求する。recovered resolver origin、missing / incompatible referenced
symbol、invalid parameter、missing correctness condition、malformed kind-specific
pattern、missing source provenance は deterministic な checker-local diagnostic で
rejected になる。reduction validation は spec 17.6.4 の固定 order を検査する: `RHS` の
free variable はすべて `LHS` に出現し、`RHS` occurrence count は `LHS` count を超えず、
checker-ready term payload が供給する alpha-normalized structural size は
`size(LHS) > size(RHS)` を満たさなければならない。caller が custom termination order を
選んだり証明したりすることはない。

activation は accepted external evidence で gate されたままである。task 14 と後続の
closure/reduction code が activated registration を作る唯一の経路は引き続き
`ActivationInput` であり、task 19 はその input が accepted verifier status または
artifact status を持つことを要求する。missing / rejected status は unaccepted activation
evidence として診断され、active record を作らない。valid pending registration と
generated obligation は proof acceptance ではない。
accepted activation evidence が同じ resolver registration の checker-ready validation
payload と一緒に供給された場合、task 19 はその companion payload を先に検証する。
それが invalid または duplicate なら activation は rejected になる。explicit accepted
validation kind も供給されている場合、その kind は validated companion kind と一致し、
resolver registration kind と互換でなければならない。valid なら accepted
activation record を作成し、その companion payload はその pass で新しい pending
obligation を発行しない。accepted status は external proof/artifact input であり、
checker が生成した obligation ではない。

task 19 では以下を意図的に deferred のままにする。

- `.miz` syntax から registration validation payload への source-to-checker extraction;
- accepted verifier/artifact status の生成または import;
- registration validation の active `.miz` semantic fixture;
- pending-validation / activation decision の artifact emission/reuse。

## existential gating

task 20 が attributed-type existence check を実装する。checker は、source construct が
attribute 付き型の値を導入または要求する場合に activated existential registration を参照する。

- `let x be A T`;
- definiens が attribute を含む mode definition;
- attribute を持つ functor return type;
- attribute 付き型の witness を主張する `consider`、`given`、`take` context;
- inhabited attributed type を明示的に要求する後続 checker-owned surface。

existential gate は、site で全 parameter / guard fact が visible である activated
existential registration が attributed normalized type の inhabited 性を証明する場合だけ
成功する。pending registration、generated but unaccepted obligation、missing proof status
は gate を満たさない。

gate が欠けている場合、checker は type error を発行し、後続 diagnostic のために degraded
typed output を保持してよい。degraded output は、その値を fully verified metadata として
export してはならず、inhabitation が証明済みであるかのように downstream fact を seed しては
ならない。

existential registration は activation-check される registration であり、validation 中に
`existence` obligation を発行してよいが、cluster fact closure の通常の
attribute-propagation edge ではない。phase 7 で自動的に使う箇所は、attribute 付き型の
inhabitation gate を満たす場合である。

### task 20: existential gate data layer

task 20 は existential gating を `src/registration_resolution.rs` の explicit-payload data
layer として実装する。checker は後続 source-to-checker extraction task が供給する
`ExistentialGateInput` record を消費し、raw syntax を walk したり resolver opaque shell を
parse したりしない。

gate input は typed site、source range、full accepted attributed type pattern key、
activation trigger、requested attribute、required guard key、checker-produced guard
evidence、candidate existential evidence を識別する。pattern key は parameter
instantiation を含む full normalized attributed type / pattern fingerprint であり、type
head だけでの matching は不十分である。これにより attributed `Subset of X` gate のような
parameterized existential registration を保つ。

candidate は同じ accepted activation record に bind されている場合だけ gate を満たせる。

- registration が `RegistrationDatabase::activated()` に存在する;
- active resolver kind が `Cluster` である;
- active record が、valid companion validation payload または explicit activation input
  から得た accepted validation kind `Existential` を持つ;
- candidate が `RegistrationValidationKind::Existential` として tag されている;
- accepted pattern、correctness key、activation evidence key、trigger、optional
  activation fingerprint が active record と一致する;
- gate pattern が accepted pattern と一致し、candidate がすべての requested attribute を
  cover する;
- すべての required guard が visible かつ consumable な checker-produced evidence を持ち、
  その evidence は既存 fact-query boundary からの `TypeFactId` で表される。

pending、rejected、external-gap、recovered、malformed、unaccepted、inactive、
non-existential、mismatched candidate は gate を満たさない。guard evidence は caller
assertion ではない。gate site で関連 fact query が visible consumable fact を見つけたことを
示す checker-owned proof である。後続 integration は explicit evidence payload を直接の
`TypeFactQueryEngine` call に置き換えてよいが、task 20 は fact を捏造しない。

gate result の precedence は deterministic である。

1. gate site が degraded なら `DegradedRecovery`;
2. valid candidate がすべての required guard evidence を持つなら `Satisfied`;
3. valid candidate はあるが guard が missing、invisible、または non-consumable なら
   `BlockedGuard`;
4. candidate はあるが valid なものがないなら `InvalidCandidate`;
5. candidate が存在しなければ `MissingExistential`。

verified downstream fact を seed してよいのは `Satisfied` gate だけである。
missing、blocked-guard、invalid-candidate、degraded-recovery gate は checker-local
diagnostic を発行し、後続 diagnostic 用の degraded output だけを保持してよいが、inhabited
metadata を export したり verified fact を seed したりしてはならない。

task 20 では以下を意図的に deferred のままにする。

- attributed-type gate site の source-to-checker extraction;
- task 19 activation input seam を超えた accepted verifier / artifact status の
  production/import;
- gate decision の artifact emission/reuse;
- existential gate の active `.miz` semantic fixture。

## cluster closure

task 16 が activated conditional / functorial registration 上の closure を実装し、task 17 が
deterministic loop / saturation diagnostic を実装する。

closure rule:

1. multi-consequent registration は firing 前に single-consequent internal rule へ正規化する。
2. fact set は normalized type、explicit attribute、context rule で visible な local
   assumption、recorded consumable fact から初期化する。
3. antecedent と parameter guard が consumable fact で満たされた activated registration
   だけを発火する。
4. type head `T` に登録された conditional registration は、phase-6 fact query が消費する
   同じ subtype relation と recorded fact に従い、`T` の互換 subtype にも適用する。
   exact-head matching だけでは不十分である。
5. 新しい derived fact は対応する resolution step に link した
   `FactProvenance::Registration` を持つ。
6. すべての step を `ResolutionTrace` に記録する。hidden transitive closure は禁止する。
7. deterministic fixed point、contradiction、loop diagnostic、configured saturation bound
   のいずれかで停止する。

矛盾する derived attribute は registration-resolution diagnostic であり、一方を捨てて
黙って調停してはならない。contradiction は fatal かつ non-recoverable な soundness-boundary
failure である。checker は contextual diagnostic を報告してよいが、影響を受けた fact を
export 可能な partial output へ degrade して続行してはならない。bounded saturation は
failure であり、truncated verified fact set の export を許可しない。

## reduction rewrite

activated reduction registration 上の reduction normalization は最終的な phase-7 contract である。
reduction は semantic rewrite であり parser rewrite ではない。source provenance を保持する。
task 18 は explicit reduction payload 上の checker-local reduction trace data layer だけを
実装する。source term からの full typed-term matching、traversal、rule search/selection、
source-derived guard extraction は MC-G020 と MC-G021 により deferred のままである。
固定された rewrite site について、normalization は typed term、スコープ内の
activated reduction rule 集合、`such` guard 用の安定した local fact、trace、
citation evidence を供給する解消済み side-condition 集合に対して決定的である。

必須挙動:

- activated reduction だけが match してよい;
- matching は raw syntax string ではなく typed term と normalized guard 上で行う;
- 各 candidate は registration parameter が導入する type / attribute guard を満たす;
- surrounding registration parameter の `such` side condition は、その rule が適用される
  前に recorded local-fact、trace、citation evidence として利用可能でなければならない。
  `such` side condition は applicability guard であり、rule をより specific にはしない;
- traversal は leftmost-innermost;
- rule selection は non-applicable rule を除外し、まず `LHS` pattern を pattern
  subsumption で比較し、pattern 比較に一意な勝者がない場合だけ §19.2.3 の
  type/attribute guard 比較を位置ごとに使う。対応が欠ける場合や guard の勝者が
  位置ごとに混在する場合は比較不能とし、`such` guard は specificity から除外し、
  最後に stable tie breaker として lexicographically smallest rule FQN を選ぶ;
- 適用された rewrite ごとに source redex、target term、substitution、discharged guard、
  rule FQN、active rule-view fingerprint、selection key、redex path、
  enclosing-term fingerprint、source provenance を記録する;
- original source term は diagnostic と LSP metadata のために利用可能なままにする。

simplification-order check は registration validation の一部である。`RHS` が `LHS` より
strictly smaller でない reduction は activation 前に rejected になる。runtime rewrite step
limit を missing validation proof の補償に使ってはならない。

## diagnostic と recovery

diagnostic は、public checker diagnostic code-space が割り当てられるまで deterministic
かつ checker-local である。必須 class:

- checker-ready registration payload の欠落;
- incompatible resolver registration kind;
- malformed / unsupported registration pattern;
- missing / incompatible referenced symbol;
- invalid registration parameter;
- missing correctness condition;
- blocked obligation emission;
- unaccepted activation evidence;
- unavailable existential registration;
- cluster contradiction、loop、saturation bound;
- invalid reduction orientation、substitution、guard evidence、strategy audit key。

recoverable diagnostic は explicit な pending、rejected、skipped、degraded record を作る。
successful type、accepted obligation、activated registration、trace step、exported fact を
捏造してはならない。

cluster contradiction は recoverable diagnostic handling から除外する。これは fatal な
soundness failure であり、影響を受けた phase-7 output の export を停止する。

## Public Enum Policy

task 31 は frontend task-25 の public-enum decision procedure をこの module に適用する。
`registration_resolution` の public checker-owned enum はすべて forward-compatible API
surface であり、`#[non_exhaustive]` を維持しなければならない。downstream consumer は
wildcard または fallback arm を保持する。checker 内部の match は、仕様化済み behavior を
実装するために現在表現されている variant へ exhaustive のままにしてよい。

| enum | decision |
|---|---|
| `RegistrationPatternStatus` | 前方互換; pattern extraction state は checker-ready payload が入るにつれて増える可能性がある。 |
| `PendingRegistrationStatus` | 前方互換; pending state は validation と artifact handoff policy とともに増える可能性がある。 |
| `RejectedRegistrationReason` | 前方互換; rejection reason は validation surface の拡大とともに増える可能性がある。 |
| `ResolverTargetShell` | 前方互換; resolver shell projection は resolver payload とともに増える可能性がある。 |
| `RegistrationValidationKind` | 前方互換; validation category は追加の registration family とともに増える可能性がある。 |
| `RegistrationValidationPattern` | 前方互換; validation pattern はより豊かな semantic payload とともに増える可能性がある。 |
| `RegistrationReferencedSymbolRole` | 前方互換; referenced-symbol role は validation check の拡大とともに増える可能性がある。 |
| `ActivationVerifierStatus` | 前方互換; verifier/artifact status は proof handoff の接続に伴い増える可能性がある。 |
| `ExistentialGateRecovery` | 前方互換; existential-gate recovery state は source extraction とともに増える可能性がある。 |
| `ExistentialGateStatus` | 前方互換; existential-gate outcome は evidence と artifact reuse とともに増える可能性がある。 |
| `RegistrationDiagnosticClass` | 前方互換; diagnostic class は public checker diagnostic code が割り当てられる前に増える可能性がある。 |
| `RegistrationDiagnosticSeverity` | 前方互換; diagnostic severity policy は IDE/artifact consumer とともに増える可能性がある。 |
| `RegistrationDiagnosticRecovery` | 前方互換; diagnostic recovery state は partial registration policy とともに増える可能性がある。 |

この module が所有する exhaustive public enum exception はない。

## external dependency gap と deferral

| ID | class | evidence | required action |
|---|---|---|---|
| MC-G005 | `spec_gap` / `external_dependency_gap` | public checker diagnostic code-space はまだ存在しない。 | owning spec/design が public code を割り当てるまでは task-local diagnostic class と stable detail key を private に保つ。 |
| MC-G019 | `external_dependency_gap` | statement/proof assumption、theorem acceptance payload、phase-7 trace fact payload は task 11 fact query から利用できない。 | registration task は既存 checker fact table と visible context だけを query する。accepted proof fact を捏造しない。 |
| MC-G020 | `external_dependency_gap` / `deferred` | tasks 7-11 が使う checker-owned payload 用の AST 全体 source-to-checker extraction API は存在しない。 | registration task は利用可能な場合に explicit checker-owned registration payload を消費し、extraction が存在するまで source `.miz` semantic coverage を deferred に保つ。 |
| MC-G021 | `external_dependency_gap` / `deferred` | 現在の resolver registration index は declaration identity、kind、opaque target shell、visibility/export metadata、dependency、recovery state、source contribution を公開するが、checker-ready typed registration pattern、parameter type payload、correctness-condition anchor、accepted verifier status、active dependency-summary consumption、reduction `LHS` / `RHS` term payload、guard-evidence payload は公開しない。task 19 は explicit validation payload を消費して検証するが、その payload の source extraction や accepted status の作成はまだ行わない。 | task 14 は resolver registration を identity/origin record としてだけ使ってよい。task 16-20 は explicit checker-owned payload seam を使うか、opaque shell の parse、summary の創作、emitted obligation の accepted 扱いをせず behavior を defer する。 |
| MC-G025 | `external_dependency_gap` / `deferred` | task 19 は checker-local な registration-correctness `InitialObligationId` を発行し、activation を accepted verifier/artifact status で gate するが、その accepted status を作成または import する proof/artifact phase は `mizar-checker` に接続されていない。これは spec 17.1 の asynchronous acceptance 契約に対する暫定的な保守近似であり、後続 source item に対する最終 rejection policy ではない。 | explicit accepted status input が供給されるまで valid local registration は pending のままにする。generated obligation を activated registration に昇格してはならない。accepted status production/import が接続されたら近似を解除し、受理済みの先行 registration が source order 上の後続 item を正当化できるようにする。 |
| MC-G026 | `test_gap` / `external_dependency_gap` / `deferred` | task 20 は explicit payload 上で existential gate を実装するが、attributed-type gate site の source-to-checker extraction、task-19 activation input を超えた accepted-status production/import、artifact emission/reuse、active `.miz` existential gate fixture はまだ配線されていない。 | explicit gate payload 上の task-local Rust coverage に保つ。real source-derived gate case と artifact reuse は、owning extraction/proof/artifact task が input を提供するまで deferred とする。 |

## planned tests

task 14:

- pending entry は closure、reduction、existential gate で発火しない;
- activation は accepted entry を deterministic trigger list へ移す;
- rejected / external-gap entry は diagnostic だけから見える;
- source contribution id は pending、activated、rejected record を通じて round-trip する。

task 16:

- conditional / functorial closure は deterministic fixed point に到達する;
- conditional cluster は exact type head だけでなく互換 subtype にも適用される;
- pending、rejected、unaccepted、external-gap registration は、antecedent が
  match しても cluster fact に寄与しない;
- すべての derived fact が registration provenance と trace step を持つ;
- multi-consequent registration は single-consequent rule へ正規化される;
- repeated run は同じ fact / trace order を生成する。

task 17:

- direct / indirect cluster loop は stable diagnostic で停止する;
- 矛盾する cluster derivation は fatal であり、truncated / degraded verified fact set を
  export しない;
- saturation bound は configuration-visible で deterministic rendering に含まれる;
- truncated closure は verified output として export されない。

task 18:

- すべての reduction で redex path、substitution、guard evidence、rule FQN、
  selection key、source redex、target term、active rule-view fingerprint、
  enclosing-term fingerprint、source provenance が記録される;
- `such` side condition は rule 適用前に recorded local-fact、trace、citation
  evidence を持たなければならず、specificity ranking に影響しない;
- pending、rejected、unaccepted、external-gap reduction は、pattern が match しても
  term を rewrite しない;
- invalid reduction substitution と mismatched strategy-audit key は stable diagnostic で
  rejected になる;
- unguarded / unsupported match は rejected になる;
- deterministic rule selection は insertion order / import order に依存しない。

task 19:

- malformed pattern と missing referenced symbol は rejected になる;
- kind-specific validation は attributed existential pattern、compatible conditional
  type head、functor result pattern、reduction の free-variable / occurrence-count /
  simplification-order / source-provenance requirement を cover する;
- validation は `VcId` を割り当てずに `InitialObligationId` を発行する;
- generated but unaccepted obligation は registration を activation しない;
- local activation 前に accepted verifier-status input を要求する。

task 20:

- missing existential registration は stable diagnostic で attributed type use を失敗させる;
- activated existential registration は guard が visible な場合だけ gate を満たす;
- pending、rejected、unaccepted、external-gap existential registration は
  attributed-type gate を満たさない;
- missing existence 後の degraded output は verified downstream fact を seed しない。

## task mapping

- task 14 は pending / activated database と deterministic index を実装する。
- task 15 は task 16-18 が消費する concrete `ResolutionTrace` shape を仕様化する。
- task 16 は cluster closure と trace recording を実装する。
- task 17 は loop / saturation diagnostic を実装する。
- task 18 は provenance 付き reduction rewrite を実装する。
- task 19 は pending registration validation と activation gating を実装する。
- task 20 は attribute 付き型使用の existential gate を実装する。

implementation task は、current source behavior に合わせるために `doc/spec` や既存 `.miz`
expectation file を変更することから始めてはならない。必要な input が欠けている場合は
external dependency gap または deferral として分類し、behavior は inactive のままにする。
