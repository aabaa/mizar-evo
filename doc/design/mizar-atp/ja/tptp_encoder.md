# Module: tptp_encoder

> 正本は英語です。英語版:
> [../en/tptp_encoder.md](../en/tptp_encoder.md)。

## 目的

`tptp_encoder` module は、validated backend-neutral `AtpProblem` を
candidate-producing ATP backend 向けの deterministic な TPTP text に lower する。
これは concrete emitter だけである。proof search、backend process 実行、
`mizar-kernel` 呼び出し、backend output の解釈、artifact witness の発行、
trusted acceptance material の生成は行わない。

出力された TPTP file は backend input である。kernel evidence、SAT problem、
instantiated-formula payload、proof certificate ではない。backend は後続でこの file から
untrusted candidate を生成してよいが、formula/substitution evidence を受け取り、自身で
instantiated formula と deterministic SAT problem を導出した後に受理できる唯一の component は
kernel のままである。

## 範囲

Task 9 は specification-only である。将来の task-10 source module が `AtpProblem` から
TPTP text を出力することを許可するが、Rust source、backend process 実行、portfolio policy、
kernel checking、proof-witness publication、cache promotion、legacy certificate handling は
追加しない。

task-10 implementation は、ここで記述する FOF dialect だけを support してよい。TPTP
TFF/TXF/THF、CNF clausification、include file、arithmetic theory、typed native property
declaration、backend-specific pragma は、paired English/Japanese spec が exact semantics と
test を定義するまで `deferred` のままである。

## 入力と出力

概念上の task-10 API は次を消費する:

```text
TptpEncodingInput
  problem: AtpProblem
  dialect: Fof
```

そして次を生成する:

```text
TptpEncodingOutput
  text: byte-identical TPTP document
  symbol_map: deterministic ATP-symbol to TPTP-name metadata
  formula_labels: deterministic TPTP-label to AtpProblem item metadata
```

semantic text は TPTP entry だけを含まなければならない。non-semantic diagnostic、source
payload、backend configuration、wall-clock data、process id、random seed、backend command
line、backend log、proof trace を TPTP document に埋め込んではならない。task 10 が
diagnostic を公開する場合、それは `text` の外で返すか、明示的に non-semantic とされた別の
deterministic debug rendering に限る。

## dialect coverage

Task 10 は次の fail-closed subset を support する:

| Problem profile field | task-10 FOF の要件 |
|---|---|
| `ConcreteFormat` | `logic_profile.concrete_formats()` が `ConcreteFormat::Tptp` を含む。 |
| `LogicFragment` | `LogicFragment::Fof`。`TffLike` と `SmtLibUninterpreted` は unsupported。 |
| `ExpectedBackendResult` | `ExpectedBackendResult::Unsat`。TPTP file は goal を conjecture として提示する。後続 backend-result classification は successful refutation/proof output を、変更しない `Unsat` contract に対応付けなければならない。 |
| `EqualitySupport` | `Unsupported` は equality formula を reject する。`Supported` は TPTP `=` を出力する。 |
| `QuantifierPolicy` | `PropositionalOnly` は `Forall` と `Exists` を reject する。`FirstOrder` は TPTP quantifier を出力する。 |
| `SoftTypeStrategy` | `GuardPredicates` のみ。`BackendSorts` と `SortsAndGuards` は、task 10 が typed TPTP semantics を持たず soft-type fact を消してはならないため FOF では unsupported。 |
| `NativePropertySupport` | native declaration を許可しない。`EncodedProperty::native_declaration` は concrete native TPTP spec が存在するまで unsupported。 |

FOF emission は unsorted first-order emission である。sort を持つ `AtpBinder`、
backend sort を必要とする formula/profile combination、または rendered use としての
`AtpDeclarationKind::Sort` は、この dialect では fail closed する。未使用の sort declaration が
存在するだけでは失敗しない。FOF は declaration を render せず、unsupported なのは sorted
binder または sort-dependent formula/type-guard use である。guard predicate と explicit
type-guard formula は、declaration が FOF で support される predicate/function である場合、
ordinary FOF axiom として出力してよい。

## entry role と順序

encoder は次の deterministic order で entry を出力する:

1. `AtpProblem.axioms()` を `fof(ax_<id>, axiom, <formula>).` として出力する。
2. `AtpProblem.type_context().guards()` を
   `fof(tg_<id>, axiom, <formula>).` として出力する。
3. `EncodedProperty::axiom` row を
   `fof(prop_<id>, axiom, <formula>).` として出力する。
4. `AtpProblem.conjecture()` を
   `fof(conj_<id>, conjecture, <formula>).` として出力する。

goal を axiom section に copy してはならない。FOF では text は declaration entry を含まない。
declaration は validation、arity、kind、binder、name-mangling metadata のために消費される。
`EncodedProperty::native_declaration` row は task 10 では render せず、unsupported/deferred
failure を返さなければならない。

label は `AtpProblem` row の stable dense id を用い、mangling 後に一意でなければならない。
後続 task が TFF support を追加する場合、typed declaration には source change より前に
paired spec で独自の ordered role と label rule を与えなければならない。

## formula rendering

Task 10 は problem に保存された backend text ではなく、structured formula tree を render する。
対応は次の通りである:

| `AtpFormulaTree` | TPTP FOF rendering |
|---|---|
| `True` | `$true` |
| `False` | `$false` |
| `Atom(P, args)` | `<pred>` または `<pred>(<term1>, <term2>, ...)` |
| `Equality { left, right }` | equality が support される場合 `(<term> = <term>)` |
| `Not(f)` | `~(<f>)` |
| `And(fs)` | `(<f1> & ... & <fn>)`。empty list は reject |
| `Or(fs)` | `(<f1> | ... | <fn>)`。empty list は reject |
| `Implies(a, b)` | `(<a> => <b>)` |
| `Forall { binders, body }` | quantifier が first-order の場合 `(! [<vars>] : (<body>))` |
| `Exists { binders, body }` | quantifier が first-order の場合 `(? [<vars>] : (<body>))` |

renderer は fixed grammar を使う:

- 各 TPTP entry は 1 行である:
  `fof(<label>, <role>, <formula>).` の後に `\n` を置く。
- document は final entry の後にちょうど 1 つの newline を持って終わる。
- label は後述の section prefix と base-10 dense id を使う。
- function / predicate argument は comma と 1 つの space で区切る。
- variable list は comma と 1 つの space で区切る。
- compound formula は table が記述する通りに必ず parenthesize する:
  equality は `(<left> = <right>)`、negation は `~(<formula>)`、
  n-ary conjunction/disjunction は `(<f1> & <f2> & ... & <fn>)` または
  `(<f1> | <f2> | ... | <fn>)`、implication は `(<left> => <right>)`、
  quantifier は `(! [<vars>] : (<body>))` / `(? [<vars>] : (<body>))`。
- singleton `And` / `Or` は `(<f1>)` として render する。empty `And` / `Or` と
  empty quantifier binder list は fail closed する。

renderer は formula を simplify、clausify、Skolemize、connective operand の reorder、
duplicate operand の削除、quantifier structure の normalize、definition の inline、
substitution の発明、unsupported construct の近似をしてはならない。empty conjunction、
disjunction、または quantifier binder list は、その logical identity が `AtpProblem` に encoded
されていないため producer error である。

term rendering は `AtpTerm::Variable` を TPTP variable name に、`AtpTerm::Function` を
TPTP function/constant name に対応付ける。arity 0 の function は constant として render する。
formula atom は predicate declaration を参照し、term は function または generated-binder
declaration を matching arity で参照しなければならない。通常は `AtpProblem::try_new` が防ぐ
場合でも、不一致は fail-closed encoder error である。

encoder は active quantifier scope を追跡しなければならない。`AtpTerm::Variable` は、その
variable が現在の formula の active `Forall` / `Exists` binder stack によって束縛されている場合だけ
render できる。free variable は reject し、TPTP の implicit universal quantification に任せては
ならない。1 つの emitted formula 内では、quantifier 内の duplicate binder variable と nested
binder shadowing は unsupported で fail closed する。binder variable は
`GeneratedBinder` declaration と、対応する `AtpSymbolSource::GeneratedBinder` symbol-map row に
resolve しなければならない。declaration が存在するだけでは不十分である。

## name mangling

TPTP name は `AtpProblem` declaration と symbol-map source identity から deterministic に
導出する。raw display spelling を TPTP syntax として再利用してはならない。

Task 10 は次の class を使う:

- predicate/function/constant name: lower-case prefix `m_` と、canonical symbol key の
  deterministic lowercase hexadecimal encoding。
- generated binder 用 variable: upper-case prefix `V_` と、canonical binder key の
  deterministic lowercase hexadecimal encoding。
- formula label: lower-case section prefix（`ax_`、`tg_`、`prop_`、`conj_`）と
  stable dense id の base-10 decimal。leading zero は付けず、id `0` は `0` として render する。

canonical symbol key は、declaration kind、declaration arity、対応する
`AtpSymbolMapEntry.source`、tie-breaker としての backend-neutral `AtpDeclaration.symbol()` に
基づく、UTF-8 field の length-delimited sequence である。map iteration、source range 単体、
display spelling 単体、backend output order、process id、random state、wall-clock time に
基づいてはならない。対応する symbol-map row を持たない declaration は reject する。

canonical binder key は generated-binder symbol-map source、declaration id、declaration arity
`0`、quantifier 内 binder position に基づく。variable は `AtpDeclarationKind::GeneratedBinder`
と対応する `AtpSymbolSource::GeneratedBinder` row を持たなければならない。

encoding が injective であっても、implementation は output を返す前に duplicate TPTP name、
reserved-word collision、illegal initial character、formula label collision を検査しなければならない。
collision は fail closed する。復旧のために traversal-order suffix を付け足してはならない。

## provenance と metadata

出力されたすべての formula label は元の `AtpProblem` row と provenance id に trace できなければならない:

- `ax_<id>` は `AtpFormulaId` とその `AtpProvenanceId` に対応する。
- `tg_<id>` は `AtpTypeGuardId` とその `AtpProvenanceId` に対応する。
- `prop_<id>` は `AtpPropertyId`、target symbol、その `AtpProvenanceId` に対応する。
- `conj_<id>` は conjecture formula id とその `AtpProvenanceId` に対応する。

この metadata は、後続の untrusted candidate extraction が backend の言及した problem item を
説明する助けになる。backend-reported used axiom、backend proof method、trace、log、SMT
proof object、resolution trace、legacy certificate を trusted acceptance material に変えるものではない。

## determinism

同等の validated `AtpProblem` input と同じ dialect は、byte-identical な TPTP text と
byte-identical な side metadata を生成しなければならない。determinism は次を含む:

- entry ordering。
- formula label。
- symbol と variable mangling。
- connective と quantifier rendering。
- newline style は `\n`。
- timestamp、process id、randomized suffix、backend version string、environment path、
  diagnostic order を semantic text に含めないこと。

encoder は `AtpProblem` を mutate したり、その semantic identity を再計算してはならない。
formatting difference は observable API behavior であり、golden test が必要である。

## public enum forward compatibility

task 22 は frontend task 25 の方針を `tptp_encoder` module に適用する。この module が所有する
public enum は downstream crate 向けに `#[non_exhaustive]` とする: `TptpDialect`、
`TptpFormulaItem`、`TptpEncodingError`。

Public enum inventory: `TptpDialect`, `TptpFormulaItem`, `TptpEncodingError`.

将来の dialect、emitted item class、error variant は、source が使う前に仕様化しなければならない。
`mizar-atp` 内部では、TPTP text、side metadata、unsupported-profile classification、proof status
に影響する match は、paired spec が意図的 fallback を記録しない限り、明示的に保ち fail closed
しなければならない。

## failure semantics

Task-10 FOF encoding は malformed producer input では fail closed し、supported FOF dialect の外に
ある valid problem では unsupported/deferred outcome を返す。どの failure mode も proof
acceptance を作らない。

- `deferred`: TFF-like sorted output、THF/TXF、CNF、include file、arithmetic theory、
  native property declaration、backend-specific pragma。
- `source_drift`: ここで仕様化されていない TPTP dialect または formula construct を source が
  support する場合、実装前に paired spec update が必要である。
- `external_dependency_gap`: backend result parsing、evidence-candidate extraction、
  portfolio execution、proof-witness publication、cache promotion は後続 task または他 crate
  の所有である。
- `boundary_violation`: TPTP text、backend success、backend proof method、backend log、
  resolution trace、SMT proof object、legacy certificate を trusted acceptance material として
  扱うことは禁止である。

## Task-10 test expectation

Task 10 は focused Rust coverage として次を追加しなければならない:

- axiom、type guard、encoded property axiom、conjecture の golden FOF output。
- repeated run と shuffled-but-equivalent validated input をまたぐ byte-identical rendering。
- atom、equality、negation、conjunction、disjunction、implication、universal/existential
  quantifier、variable、constant、function application の formula rendering。exact separator、
  parenthesization、label、final-newline behavior を含む。
- `ConcreteFormat::Tptp` 欠落、non-FOF fragment、backend-sort strategy、unsupported equality、
  unsupported quantifier、sorted binder に対する profile rejection。
- native property declaration、empty `And`/`Or`、empty quantifier binder list の fail-closed
  rejection。
- free variable、duplicate binder、nested binder shadowing、declaration/symbol-map/kind/arity
  mismatch、name-mangling collision の fail-closed rejection。
- TPTP keyword、reserved `$` name、uppercase/lowercase edge case、punctuation、whitespace、
  newline-like payload のように見える source/backend-neutral spelling を用いた raw-name
  injection coverage。symbol / variable position が deterministic な `m_<hex>` と `V_<hex>`
  mangled name だけを使い、raw spelling が emitted symbol / variable name として semantic
  `text` に現れないことを assert する。
- 出力された全 entry の provenance side metadata。
- backend runner、kernel/SAT checking、proof policy、witness/cache integration、backend proof
  method、backend log、resolution trace、instantiated formula、SAT problem が追加されていない
  ことを確認する lint/API guard。
