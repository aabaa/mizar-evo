# Module: smtlib_encoder

> 正本は英語です。英語版:
> [../en/smtlib_encoder.md](../en/smtlib_encoder.md)。

## 目的

`smtlib_encoder` module は、validated backend-neutral `AtpProblem` を
candidate-producing SMT backend 向けの deterministic な SMT-LIB 2 text に lower する。
これは concrete emitter だけである。proof search、backend process 実行、
`mizar-kernel` 呼び出し、backend output の解釈、artifact witness の発行、
trusted acceptance material の生成は行わない。

出力された SMT-LIB document は backend input である。kernel evidence、SAT problem、
instantiated-formula payload、unsat core、SMT proof certificate ではない。backend は後続で
この document から untrusted candidate を生成してよいが、formula/substitution evidence を
受け取り、自身で instantiated formula と deterministic SAT problem を導出した後に受理できる
唯一の component は kernel のままである。

## 範囲

Task 11 は specification-only である。将来の task-12 source module が `AtpProblem` から
SMT-LIB text を出力することを許可するが、Rust source、backend process 実行、portfolio
policy、kernel checking、proof-witness publication、cache promotion、legacy certificate
handling、SMT proof-object handling は追加しない。

task-12 implementation は、ここで記述する uninterpreted SMT-LIB profile だけを support してよい。
arithmetic theory、array、datatype、bit-vector、native property declaration、
quantifier-instantiation pragma、solver option、`get-proof`、`get-unsat-core`、
backend-specific command は、paired English/Japanese spec が exact semantics と test を定義するまで
`deferred` のままである。

## 入力と出力

概念上の task-12 API は次を消費する:

```text
SmtLibEncodingInput
  problem: AtpProblem
  dialect: Uninterpreted
```

そして次を生成する:

```text
SmtLibEncodingOutput
  text: byte-identical SMT-LIB document
  symbol_map: deterministic ATP-symbol to SMT-LIB-symbol metadata
  assertion_labels: deterministic :named label to AtpProblem item metadata
```

semantic text は 1 つの SMT-LIB `set-logic` command、declaration、assertion、1 つの
`check-sat` command だけを含まなければならない。non-semantic diagnostic、source payload、
backend configuration、wall-clock data、process id、random seed、backend command line、
backend log、unsat core、proof trace、SMT proof object を SMT-LIB document に埋め込んでは
ならない。task 12 が diagnostic を公開する場合、それは `text` の外で返すか、明示的に
non-semantic とされた別の deterministic debug rendering に限る。

## dialect coverage

Task 12 は次の fail-closed subset を support する:

| Problem profile field | task-12 uninterpreted SMT-LIB の要件 |
|---|---|
| `ConcreteFormat` | `logic_profile.concrete_formats()` が `ConcreteFormat::SmtLib` を含む。 |
| `LogicFragment` | `LogicFragment::SmtLibUninterpreted`。`Fof` と `TffLike` は unsupported。 |
| `ExpectedBackendResult` | `ExpectedBackendResult::Unsat`。SMT-LIB file は premise と negated goal を assert し、`check-sat` を出力する。後続 result classification では `unsat` backend result だけが profile contract に一致し得る。 |
| `EqualitySupport` | `Unsupported` は equality formula を reject する。`Supported` は SMT-LIB `=` を出力する。 |
| `QuantifierPolicy` | `PropositionalOnly` は `Forall` と `Exists` を reject し、`QF_UF` を選択する。`FirstOrder` は SMT-LIB quantifier を出力し、`UF` を選択する。logic selection は profile-driven であり、backend output や formula simplification に基づかない。 |
| `SoftTypeStrategy` | `GuardPredicates` のみ。`BackendSorts` と `SortsAndGuards` は、現在の `AtpProblem` model が function/predicate sort signature を持たず、sort-only encoding が required soft-type fact を消してはならないため task 12 では unsupported。 |
| `NativePropertySupport` | native declaration を許可しない。`EncodedProperty::native_declaration` は concrete native SMT-LIB spec が存在するまで unsupported。 |

task-12 の sort encoding は意図的に保守的である。すべての term は `mizar_universe` という 1 つの
fixed uninterpreted sort に属し、Mizar の soft-type fact は explicit guard predicate と type-guard
assertion として保存する。`Function` declaration は `mizar_universe` の引数から
`mizar_universe` への function として emit し、`Predicate` declaration は `mizar_universe` の引数から
`Bool` への predicate として emit する。

sort を持つ `AtpBinder`、backend sort を必要とする profile、または rendered use としての
`AtpDeclarationKind::Sort` は、この dialect では fail closed する。未使用の sort declaration が
存在するだけでは失敗しない。task 12 は problem-owned sort declaration を render しない。将来の
sorted SMT-LIB profile は、`BackendSorts` または `SortsAndGuards` を有効化する前に、明示的な
declaration signature、type preservation rule、test を追加しなければならない。

## command ordering と label

encoder は次の deterministic order で command を出力する:

1. `PropositionalOnly` では `(set-logic QF_UF)`、`FirstOrder` では `(set-logic UF)`。
2. `(declare-sort mizar_universe 0)`。
3. normalized `AtpProblem.declarations()` の順で、すべての `AtpDeclarationKind::Function`
   の function declaration。
4. normalized `AtpProblem.declarations()` の順で、すべての `AtpDeclarationKind::Predicate`
   の predicate declaration。
5. `AtpProblem.axioms()` を named assertion `ax_<id>` として出力する。
6. `AtpProblem.type_context().guards()` を named assertion `tg_<id>` として出力する。
7. `EncodedProperty::axiom` row を named assertion `prop_<id>` として出力する。
8. negated `AtpProblem.conjecture()` を named assertion `neg_conj_<id>` として出力する。
9. `(check-sat)`。

goal を positive に assert してはならず、premise section に copy してもならない。現在の `Unsat`
contract では、SMT-LIB validity checking は premise、generated type guard、encoded property axiom、
`not(goal)` を assert し、backend に `unsat` を期待する。

named assertion は次の exact shape を使う:

```text
(assert (! <formula> :named <label>))
```

negated conjecture label は、assert される formula が conjecture そのものではないため
`neg_conj_` prefix を持つ。formula label は source row の stable dense id を base-10 decimal で
使う。leading zero は付けず、id `0` は `0` として render する。

## SMT-LIB grammar

renderer は fixed textual grammar を使う:

- 各 command は 1 行で、その後に `\n` を置く。
- `set-logic` command はちょうど 1 つだけ出力され、先頭行である。
- document は `(check-sat)` の後にちょうど 1 つの newline を持って終わる。
- semantic text に comment は出力しない。
- declaration argument sort は 1 つの space で区切る。
- application argument は 1 つの space で区切る。
- assertion label は上で述べた fixed prefix と decimal dense id だけを使う。
- solver option、`push` / `pop`、`reset`、`get-model`、`get-proof`、
  `get-unsat-core`、backend-specific command は出力しない。

function と predicate declaration は次のように render する:

```text
(declare-fun <function> (mizar_universe ...) mizar_universe)
(declare-fun <predicate> (mizar_universe ...) Bool)
```

arity 0 の function と predicate は empty argument list `()` を使う。`GeneratedBinder`
declaration は top-level constant ではない。active quantifier binder を render するときだけ消費される。

## formula rendering

Task 12 は problem に保存された backend text ではなく、structured formula tree を render する。
対応は次の通りである:

| `AtpFormulaTree` | SMT-LIB rendering |
|---|---|
| `True` | `true` |
| `False` | `false` |
| `Atom(P, args)` | `<pred>` または `(<pred> <term1> <term2> ...)` |
| `Equality { left, right }` | equality が support される場合 `(= <term> <term>)` |
| `Not(f)` | `(not <f>)` |
| `And(fs)` | `(and <f1> ... <fn>)`。empty list は reject |
| `Or(fs)` | `(or <f1> ... <fn>)`。empty list は reject |
| `Implies(a, b)` | `(=> <a> <b>)` |
| `Forall { binders, body }` | quantifier が first-order の場合 `(forall ((<var> mizar_universe) ...) <body>)` |
| `Exists { binders, body }` | quantifier が first-order の場合 `(exists ((<var> mizar_universe) ...) <body>)` |

singleton `And` / `Or` は `and` または `or` wrapper を使わず、単一の child formula として render する。
empty `And` / `Or` と empty quantifier binder list は fail closed する。

renderer は formula を simplify、clausify、Skolemize、connective operand の reorder、
duplicate operand の削除、quantifier structure の normalize、definition の inline、
substitution の発明、quantifier pattern の追加、solver-specific attribute の追加、unsupported
construct の近似をしてはならない。empty conjunction、disjunction、または quantifier binder list は、
その logical identity が `AtpProblem` に encoded されていないため producer error である。

term rendering は `AtpTerm::Variable` を SMT-LIB variable name に、`AtpTerm::Function` を
SMT-LIB function/constant name に対応付ける。formula atom は predicate declaration を参照し、
term は function または generated-binder declaration を matching arity で参照しなければならない。
通常は `AtpProblem::try_new` が防ぐ場合でも、不一致は fail-closed encoder error である。

encoder は active quantifier scope を追跡しなければならない。`AtpTerm::Variable` は、その
variable が現在の formula の active `Forall` / `Exists` binder stack によって束縛されている場合だけ
render できる。free variable は reject する。1 つの emitted formula 内では、quantifier 内の
duplicate binder variable と nested binder shadowing は unsupported で fail closed する。binder
variable は `GeneratedBinder` declaration と、対応する
`AtpSymbolSource::GeneratedBinder` symbol-map row に resolve しなければならない。declaration が
存在するだけでは不十分である。

## name mangling

SMT-LIB symbol は `AtpProblem` declaration と symbol-map source identity から deterministic に
導出する。raw display spelling を SMT-LIB syntax として再利用してはならない。

Task 12 は次の class を使う:

- fixed universe sort: literal `mizar_universe`。
- predicate/function/constant name: lower-case prefix `m_` と、canonical symbol key の
  deterministic lowercase hexadecimal encoding。
- generated binder 用 variable: lower-case prefix `v_` と、canonical binder key の
  deterministic lowercase hexadecimal encoding。
- assertion label: lower-case section prefix（`ax_`、`tg_`、`prop_`、`neg_conj_`）と
  stable dense id の base-10 decimal。leading zero は付けず、id `0` は `0` として render する。

canonical symbol key は、declaration kind、declaration arity、対応する
`AtpSymbolMapEntry.source`、tie-breaker としての backend-neutral `AtpDeclaration.symbol()` に
基づく、UTF-8 field の length-delimited sequence である。map iteration、source range 単体、
display spelling 単体、backend output order、process id、random state、wall-clock time に
基づいてはならない。対応する symbol-map row を持たない declaration は reject する。

canonical binder key は generated-binder symbol-map source、declaration id、declaration arity
`0`、quantifier 内 binder position に基づく。variable は `AtpDeclarationKind::GeneratedBinder`
と対応する `AtpSymbolSource::GeneratedBinder` row を持たなければならない。

encoding が injective であっても、implementation は output を返す前に duplicate SMT-LIB
symbol、reserved-word collision、illegal symbol character、fixed sort collision、assertion-label
collision を検査しなければならない。collision は fail closed する。復旧のために traversal-order
suffix を付け足してはならない。

## provenance と metadata

出力されたすべての assertion label は元の `AtpProblem` row と provenance id に trace できなければならない:

- `ax_<id>` は `AtpFormulaId` とその `AtpProvenanceId` に対応する。
- `tg_<id>` は `AtpTypeGuardId` とその `AtpProvenanceId` に対応する。
- `prop_<id>` は `AtpPropertyId`、target symbol、その `AtpProvenanceId` に対応する。
- `neg_conj_<id>` は conjecture formula id とその `AtpProvenanceId` に対応し、emitted assertion が
  `Unsat` validity contract のために negated されていることを記録する。

この metadata は、後続の untrusted candidate extraction が backend の言及した problem item を
説明する助けになる。backend-reported unsat core、used axiom、backend proof method、trace、log、
SMT proof object、resolution trace、legacy certificate を trusted acceptance material に変えるものではない。

## determinism

同等の validated `AtpProblem` input と同じ dialect は、byte-identical な SMT-LIB text と
byte-identical な side metadata を生成しなければならない。determinism は次を含む:

- profile からの logic selection。
- command ordering。
- declaration ordering。
- assertion label。
- symbol と variable mangling。
- connective と quantifier rendering。
- newline style は `\n`。
- semantic text に comment、timestamp、process id、randomized suffix、backend version string、
  environment path、diagnostic order、backend log、proof trace、SMT proof object を含めないこと。

encoder は `AtpProblem` を mutate したり、その semantic identity を再計算してはならない。
formatting difference は observable API behavior であり、golden test が必要である。

## public enum forward compatibility

task 22 は frontend task 25 の方針を `smtlib_encoder` module に適用する。この module が所有する
public enum は downstream crate 向けに `#[non_exhaustive]` とする: `SmtLibDialect`、
`SmtLibAssertionItem`、`SmtLibEncodingError`。

Public enum inventory: `SmtLibDialect`, `SmtLibAssertionItem`, `SmtLibEncodingError`.

将来の dialect、assertion item class、error variant は、source が使う前に仕様化しなければ
ならない。`mizar-atp` 内部では、SMT-LIB text、side metadata、unsupported-profile
classification、proof status に影響する match は、paired spec が意図的 fallback を記録しない限り、
明示的に保ち fail closed しなければならない。

## failure semantics

Task-12 SMT-LIB encoding は malformed producer input では fail closed し、supported
uninterpreted profile の外にある valid problem では unsupported/deferred outcome を返す。
どの failure mode も proof acceptance を作らない。

- `deferred`: arithmetic theory、array、datatype、bit-vector、sorted function/predicate
  signature、`BackendSorts`、`SortsAndGuards`、native property declaration、quantifier
  pattern、solver option、`get-proof`、`get-unsat-core`、backend-specific command。
- `source_drift`: ここで仕様化されていない SMT-LIB dialect、formula construct、sort
  encoding、native property shortcut、command を source が support する場合、実装前に paired
  spec update が必要である。
- `boundary_violation`: backend proof method、SMT proof object、backend log、instantiated
  formula、SAT problem、kernel-derived material、legacy certificate、resolution trace を trusted
  material として emit することは禁止である。
- `external_dependency_gap`: result classification、backend execution、candidate extraction、
  proof policy、witness publication、cache promotion は後続 task/crate に残る。

unsupported SMT-LIB profile は VC を open のままにするか、後続で別の explicit backend profile を選択してよい。
encoder は type guard を黙って落としたり、unsupported formula を `true` に置換したり、goal を
positive に assert したり、`sat` / `unknown` を proof として受理してはならない。

## Task-12 Test Expectations

Task 12 は次の focused Rust coverage を追加しなければならない:

- `set-logic`、fixed `mizar_universe` sort、declaration、`ax_`、`tg_`、`prop_`、
  `neg_conj_`、`(check-sat)` ordering を含む golden SMT-LIB output。
- `PropositionalOnly` の `QF_UF` selection と `FirstOrder` の `UF` selection。
- singleton `And` / `Or` rendering と final newline behavior を含む、support されるすべての
  formula/term form の exact rendering。
- premise plus negated conjecture polarity。positive goal assertion が存在しないこと。
- shuffled equivalent input と semantic problem content を変えない diagnostic 変更での
  byte-identical output。
- missing `ConcreteFormat::SmtLib`、非 `SmtLibUninterpreted` fragment、unsupported
  soft-type strategy、equality-disabled formula、quantifier-disabled formula、sorted binder、
  sort-dependent use の profile gate。
- support される guard-predicate profile では、未使用の `AtpDeclarationKind::Sort` row を
  accept し、rendering では無視し、SMT-LIB output に含めないこと。
- native-property declaration rejection。
- free-variable、duplicate-binder、shadowing、missing-declaration、missing-symbol-map、
  invalid-declaration、invalid-arity、invalid-binder source failure。
- function、predicate、constant、generated-binder spelling の raw-name injection avoidance。
- duplicate/illegal/reserved SMT-LIB symbol と assertion-label rejection。
- symbol-binding と assertion-label side metadata。
- backend runner、kernel/SAT checking、proof acceptance、witness、cache、legacy certificate、
  resolution trace、SMT proof object、unsat-core trust、trusted backend-material API surface がないこと。
