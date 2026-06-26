# mizar-atp property encoding

> 正本は英語です。英語版:
> [../en/property_encoding.md](../en/property_encoding.md)。

## 目的

この文書は、解決済みの Mizar 側 property を `mizar-atp` が backend-neutral な
`EncodedProperty` row として表現する方法を仕様化する。これは
[architecture 09](../../architecture/ja/09.atp_interface_protocol.md) の
「性質のエンコード」節を詳細化する。

Property encoding は candidate-production input にすぎない。VC を証明せず、backend
を実行せず、trusted acceptance material を作らない。Kernel acceptance は引き続き
`mizar-kernel` が検査する formula/substitution/provenance/target evidence に基づく。

## 範囲

Task 7 は仕様のみである。将来の task 8 source module が、明示的で既に利用可能な
property fact を `AtpProblem.properties` に encode することを許可する。Rust source、
具体的な TPTP/SMT-LIB text、backend process execution、portfolio policy、artifact
witness publication は追加しない。

Property encoder は、より前の Mizar-owned phase が生成した structured property input
だけを消費してよい。symbol name、backend declaration、library naming convention、
proof hint、trace、log、backend response から property を推測してはならない。

## 入力と identity

各 encoded property は次を持たなければならない:

- VC/core 側から供給された stable source property identity。
- 下表の supported property family。
- `AtpProblem` で既に宣言済みの backend-visible target symbol。
- family が要求する target symbol kind と arity。
- `AtpSourceRef::EncodedProperty` として表現できる source provenance。
- 選択された `LogicProfile` と、その property/native-extension capability。

Deterministic property identity は source property identity、property family、target
symbol source identity、target arity、選択された encoding strategy に基づく。traversal
order、map iteration、display spelling 単体、source range 単体、backend output order、
process id、random state、wall-clock time を使ってはならない。

Duplicate source property identity または duplicate encoded identity は fail-closed
producer input error である。同じ target symbol に異なる property family が適用できるのは、
それぞれが明示的な source property fact を持つ場合だけである。

## Supported family

Task 8 implementation は、後続仕様が表を拡張するまで、次の property family だけを
encode してよい:

| Family | Target | Formula Shape | Default Encoding |
|---|---|---|---|
| `commutativity` | binary function | `forall a b. F(a, b) = F(b, a)` | axiom formula |
| `symmetry` | binary predicate | `forall a b. P(a, b) -> P(b, a)` | axiom formula |
| `reflexivity` | binary predicate | `forall a. P(a, a)` | axiom formula |
| `idempotence` | binary function | `forall a. F(a, a) = a` | axiom formula |
| `involutiveness` | unary function | `forall a. F(F(a)) = a` | axiom formula |
| `projectivity` | unary function | `forall a. F(F(a)) = F(a)` | axiom formula |
| `asymmetry` | binary predicate | `forall a b. P(a, b) -> not P(b, a)` | axiom formula |
| `connectedness` | binary predicate | `forall a b. a != b -> P(a, b) or P(b, a)` | axiom formula |
| `irreflexivity` | binary predicate | `forall a. not P(a, a)` | axiom formula |

ここにない family は `deferred` である。target kind の不一致、arity の不一致、missing
declaration、missing symbol-map row、missing provenance、malformed source identity、
unsupported formula shape は fail closed する。

## Encoding strategy

### Axiom formula

既定の strategy は `EncodedProperty::axiom` である。生成される formula は backend text
ではなく structured `AtpFormulaTree` でなければならない。必要な formal variable すべてに
対して universal であり、`target_symbol` field に現れるものと同じ backend-visible target
symbol を使わなければならない。

選択された `LogicProfile` は formula が使うすべての construct を support しなければならない:
universal family の quantifier、function property と connectedness disequality の equality、
出現する implication、disjunction、negation、nested unary-function property の first-order term。
現在の backend-neutral `LogicProfile` は Boolean connective を baseline first-order formula-tree
construct として扱う。そのため、quantifier と equality が support されるなら、`connectedness` は
`AtpFormulaTree::Or` を使ってよい。将来の profile が explicit connective gate を追加する場合、
`connectedness` は disjunction support を要求し、それがないとき fail closed しなければならない。
profile が必要な construct を support しない場合、その property はその profile では unsupported
である。encoder は property を近似したり、黙って落としたり、`true` に置き換えたりしてはならない。

### Generated binder

Property axiom が導入する各 formal variable は、formula construction 前に generated binder
symbol として表現しなければならない。各 binder について、encoder は次を作る:

- kind が `AtpDeclarationKind::GeneratedBinder`、arity が `0` の `AtpDeclaration`。
- `AtpSymbolSource::GeneratedBinder` を持つ `AtpSymbolMapEntry`。
- generated binder identity を説明する provenance。

Binder identity、backend-neutral symbol name、declaration、symbol-map row、provenance payload は、
canonical property identity、target symbol source identity、binder position、任意の sort identity
から導出する。display spelling 単体、source range 単体、traversal order、map iteration、random
state、backend output に依存してはならない。Quantified formula 内の binder order は family
definition によって canonical である。unary family は単一 position `0` を使い、binary family は
position `0`、`1` の順に使う。

Generated-binder declaration、symbol-map row、provenance row が欠落または重複する場合、
`AtpProblem` construction 前に fail closed する。Property axiom は caller-supplied instantiated
variable を trusted payload として再利用してはならない。property encoder が backend-neutral
candidate problem 用の generated binder identity を所有する。

### Native declaration

`EncodedProperty::native_declaration` は、次をすべて満たす場合だけ使ってよい:

- `LogicProfile::native_properties()` が supported である。
- 選択された concrete encoder spec が、VC context で利用可能な明示的 property fact と
  semantic に強すぎず弱すぎない backend-native declaration を定義している。
- native declaration が独自の `AtpDeclaration`、`symbol_map` row、`AtpProvenance` を持つ。
- candidate evidence と used-property reporting が参照できるよう、encoding decision が
  `EncodedProperty` に記録される。

Backend-native support は proof method ではなく、trusted acceptance material でもない。
backend が native declaration の使用を報告しても、それは kernel evidence checking が対応する
formula/substitution evidence を検証するまで参考情報にとどまる。

Native AC-style declaration は、native backend construct が利用可能な property fact と完全に一致する
場合だけ許可される。backend construct が associativity と commutativity を結合している場合、
両方の fact が既に明示的に利用可能でなければならない。commutativity だけを AC に格上げしてはならない。

Task 8 はまだ native declaration を emit してはならない。Task 8 時点では、exact native semantics
を定義する concrete TPTP/SMT-LIB encoder spec が後続 task のままである。そのため
`LogicProfile::native_properties()` が supported でも、native-property request は `deferred` または
unsupported/open-status outcome に分類する。後続 task は、関連する concrete encoder spec と test が
exact semantics、declaration shape、symbol-map row、provenance を定義した後にだけ
`EncodedProperty::native_declaration` を有効化してよい。

## Provenance

各 encoded property row は `AtpSourceRef::EncodedProperty` を持つ `AtpProvenance` entry を
持たなければならない。source binding は resolved property fact と target symbol identity を
名前付けする。provenance payload は resolved property fact から導出される deterministic な
producer-side anchor でなければならず、backend text、backend proof method、backend log、
resolution trace、SMT proof object、instantiated formula、SAT problem であってはならない。

Native declaration では、provenance は `EncodedProperty` row と backend-visible declaration の
両方に属する。declaration provenance は生成された declaration を説明し、property provenance は
その property を許可した Mizar 側 fact を説明する。

## Determinism

等価な validated input は byte-identical な `AtpProblem` debug rendering と同じ problem id を
生成しなければならない。Property row は canonical property identity から導いた deterministic
dense id で順序付ける。Task 8 が property row を構築するときは、dense id を割り当てる前に
input を sort し、`AtpProblem` construction 前に duplicate key を reject しなければならない。

Diagnostic は unsupported property family や profile に言及してよいが、proof acceptance には
参加しない。

## Failure semantics

- `source_drift`: source がここで仕様化されていない property family を後で encode する場合、
  実装前に英語/日本語ペアの仕様更新を追加する。
- `deferred`: associativity、transitivity、antisymmetry、compatibility、monotonicity、
  domain-specific algebraic package は、後続 task が family table を拡張するまで未仕様である。
- `external_dependency_gap`: downstream proof/cache/artifact crate からの property input は、
  それらの crate が stable producer contract を定義するまで利用できない。
- `boundary_violation`: backend-native property declaration、backend proof method、log、trace、
  used-axiom report を trusted acceptance material として扱うことは禁止する。

Task 8 implementation は malformed、duplicate、missing、unavailable な property input では
fail closed しなければならない。well-formed だが unsupported な property/profile combination は、
別の明示的 profile または後続 pipeline step のために VC を open のままにし、accepted proof status を
作らない。

## Task 8 test expectations

Task 8 は次の Rust coverage を追加しなければならない:

- supported family すべての axiom-form coverage。
- unary / binary property axiom の generated-binder declaration、symbol-map、provenance、
  canonical identity、binder order。
- function/predicate family group ごとの wrong target kind と arity。
- missing declaration、symbol-map row、provenance。
- quantifier/equality support が足りない profile の rejection。
- `Or` formula-tree branch を通る connectedness coverage と、将来 profile が explicit connective
  capability flag を追加した場合の connective-gate rejection。
- native declaration request は、concrete encoder spec が exact native semantics を定義するまで
  deferred または fail closed されること。
- shuffled property input に対する deterministic row order と problem identity。
- duplicate source property identity と duplicate encoded identity の rejection。
- property API に backend proof method、backend log、resolution trace、instantiated formula、
  SAT problem、accepted-proof status field が存在しないこと。
