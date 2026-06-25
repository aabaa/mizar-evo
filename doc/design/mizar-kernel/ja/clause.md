# Module: clause

> 正本は英語です。英語版: [../en/clause.md](../en/clause.md)。

## 目的

`clause` module は、trusted kernel が消費する normalized SAT clause 表現を
所有する。この文書は
[architecture 15](../../architecture/ja/15.kernel_certificate_format.md) の
「節の表現」を精緻化する。

Clause value は evidence data である。well-formed な clause は proof ではなく、
それだけで acceptance を与えない。後続の certificate checker と trace checker
が信頼してよいのは、この正準値の上で explicit evidence を replay した肯定的な
結果だけである。

## Trust statement

この module は trusted kernel code である。literal と clause の構造、正準順序、
決定的 rendering、hash input を検査する。小さく、決定的で、bounded input に
対して total でなければならない。

この module は proof search、heuristic premise selection、overload resolution、
cluster search、ATP search、implicit coercion insertion、fallback inference、
backend-reported success だけによる acceptance、mutable compiler-global state、
wall-clock time、random state、filesystem cache、unordered iteration、
allocation address の hidden read を行ってはならない。

## 所有する挙動

この module が所有するもの:

- literal polarity と atom identity;
- canonical literal ordering;
- normalization 中の duplicate literal removal;
- atom、term、clause の structural well-formedness check;
- explicit kernel-profile option に従う tautology classification;
- tests と diagnostics で使う deterministic text rendering;
- nondeterministic data を除外した stable hash input bytes。

この module が所有しないもの:

- certificate parsing;
- parent-reference validation;
- resolution trace replay;
- substitution または alpha-equivalence checking;
- imported-fact availability;
- proof-policy projection。

## Data model

実装は以下の論理 data shape を使う。具体的な Rust 名は task 3 で選ぶ source style
に従ってよい。

```text
Clause
  profile
  literals
  form

Literal
  polarity
  atom

Atom
  symbol
  arguments

Term
  variable
  application
  binder_normalized
```

`profile` は schema version、tautology policy、canonical encoding version を
記録する。`form` は以下のいずれかである:

- `ordinary`: 1 つ以上の canonical literal を持つ;
- `empty`: 0 個の literal を持ち、refutation trace の contradiction endpoint を
  表す;
- `tautology`: profile が explicit tautology marker を許す場合に限り、0 個の
  literal を持つ。

`empty` と `tautology` は別の form である。Empty clause は resolution replay に
よって導出され得る unsatisfiable evidence である。Tautology marker は常に真で
ある clause を記録し、contradiction として使ってはならない。

Symbol、variable、generated binder は、earlier phase または certificate が与える
stable normalized id だけで比較する。display name と source range は diagnostics
専用であり、semantic equality と hash から除外する。

## Canonical ordering

Canonical ordering は total かつ platform-independent である:

1. polarity。negative literal を positive literal より前に置く;
2. atom symbol kind。順序は predicate、functor-as-predicate、equality、
   built-in relation の固定順;
3. symbol kind 内の stable symbol id;
4. atom arity;
5. normalized argument encoding bytes;
6. 最終 tie breaker として literal canonical bytes。

Term ordering は term の normalized encoding の byte order である。Variable term
は canonical variable id を encode し、parser encounter order、allocation address、
display name、hash-map iteration order は使わない。

Canonical bytes は length-prefixed binary grammar を使う:

```text
u8 schema_tag
u8 form_or_term_tag
u32 length
bytes payload
```

すべての integer は unsigned big-endian value である。Nested payload は canonical
order の length-prefixed canonical child bytes を連結して encode する。この byte
grammar が ordering tie breaker であり hash input source である。

この module は ordering から semantic fact を導出してはならない。Ordering は
equality、rendering、hashing を決定的にするためだけに存在する。

## Structural well-formedness

Literal が well formed である条件:

- atom が stable symbol id を持つ;
- arity が encoded argument 数と一致する;
- すべての argument が normalized encoding を持つ;
- すべての variable id が certificate context に対して canonical form である;
- display-only field が equality または hashing に参加しない。

Clause が well formed である条件:

- 1 つの explicit clause profile に結び付いている;
- form/literal payload が profile rule と一致する。`ordinary` は少なくとも 1 つの
  literal を持ち、`empty` は 0 個の literal を持ち、`tautology` は 0 個の literal
  を持ち、profile が tautology marker を許す場合に限られる;
- ordinary literal が canonical order で sort されている;
- duplicate ordinary literal が除去されている;
- 設定された literal-count、term-size、term-recursion-depth bound を守る。

Task 3 は minimal clause-local validation context を使う:

```text
ClauseValidationContext
  profile
  allowed_symbol_kinds
  known_symbol_ids
  canonical_variable_ids
  max_literals
  max_term_encoding_bytes
  max_term_recursion_depth
```

この context は clause construction への explicit input である。Global symbol table
ではなく、resolver、checker、ATP、cache、artifact state から populate してはならない。
後続の certificate parsing は normalized certificate metadata から context を与えて
よいが、task 3 は certificate を parse せずに clause-local validation を test できなければ
ならない。

Malformed structure は caller により certificate error または kernel error として
reject される。この module は precise structural reason を返すが、phase-level
diagnostic policy は決めない。

## Normalization

Normalization は決定的な pure function である:

```text
raw literals + ClauseValidationContext -> normalized clause or rejection
```

Normalization が行うこと:

- すべての literal と term を validate する;
- ordinary literal を canonical order で sort する;
- duplicate ordinary literal を除去する;
- opposite-polarity duplicate atom を検出する;
- profile の tautology rule を適用する。

Profile が tautology を reject する場合、ある atom とその opposite-polarity counterpart
を同時に含む clause は invalid である。Profile が explicit tautology marker を許す
場合、normalized result は zero-literal `tautology` form を使い、矛盾する literal pair
や witness pair を保持しない。この form では raw input の non-contradictory literal も
破棄し、すべての tautological clause がただ 1 つの rendering と hash を持つようにする。

Literal を持たない input は zero-literal `empty` form に normalize され、tautology
marker にはならない。

## Rendering and hashing

Debug rendering は決定的で snapshot tests に適していなければならない。Rendering は
以下を使う:

- schema/profile version;
- clause form;
- literal count;
- 順序付き canonical literal rendering;
- stable id と normalized argument encoding。

Stable clause hash は以下を含む canonical bytes から計算する:

- kernel clause 用 domain separator;
- schema/profile version;
- tautology policy;
- clause form;
- canonical literal bytes。

Hash は file path、source range、display name、timestamp、backend runtime log、
allocation address、map/set iteration order、worker completion order を含んでは
ならない。

Trusted replay module は、大きな temporary byte vector を構築する前に resource
accounting を必要とする場合がある。この accounting に使う non-allocating canonical
length または bounded-writer helper は clause module が所有する。Caller は canonical
size を見積もるために clause encoder を重複実装してはならない。

Trusted replay module は、すでに所有されている canonical clause parts をより小さい
replay budget の下で再検査する必要もある。この経路で使う borrowed canonical-part
validation helper は clause module が所有し、caller が literal-count、term-size、
term-depth limits を検査する前に大きな literal / term tree を clone しなくて済むようにする。

## Failure classes

この module は以下の structural rejection detail を生成できる:

- missing または unstable symbol id;
- arity mismatch;
- malformed term encoding;
- noncanonical variable id;
- construction path が duplicate を受け付けない場合の normalization 前 duplicate
  literal;
- ordinary form の empty payload、および non-empty `empty` / `tautology` payload;
- disallowed tautology;
- literal-count、term-size、または term-recursion-depth resource exhaustion。

Caller は、`rejection.md` が存在した後、その detail を stable rejection category へ
map する。それまでは task 3 tests は artifact-facing diagnostic text ではなく、
module-local structural error identity を assert する。

## Planned tests

Task 3 は以下の Rust tests を追加しなければならない:

- valid single-literal / multi-literal clause;
- polarity、symbol id、arity、argument encoding による deterministic ordering;
- それ以外は比較可能な id を持つ異なる symbol kind の symbol-kind precedence。
  id だけで sort する実装は通らないこと;
- duplicate literal removal;
- empty-clause / contradiction form の rendering と hash stability;
- disallowed tautology rejection;
- allowed zero-payload tautology marker normalization;
- malformed arity、malformed term encoding、missing-symbol、unsupported
  symbol-kind rejection;
- explicit clause-local validation context による noncanonical variable-id rejection;
- empty ordinary payload rejection と non-empty `empty` / `tautology` payload rejection
  を含む explicit profile/form payload rule;
- literal-count と term-size resource exhaustion;
- `ClauseValidationContext` の depth limit による term-recursion-depth resource
  exhaustion;
- caller が encoder を重複実装しなくて済むように、canonical byte encoder と同じ長さを
  返す non-allocating canonical length または bounded-writer helper;
- caller が literal vector を clone する前に、budget 超過の canonical clause を拒否する
  borrowed canonical-part validation;
- normalizing constructor を bypass する construction path がある場合の
  duplicate-before-normalization rejection;
- rendering stability;
- input literal order を shuffle しても hash input が安定すること;
- domain separator、schema/profile version、tautology policy、clause form、
  canonical literal bytes が hash input に含まれること;
- display name、source range、file path、timestamp、backend log、allocation order、
  worker completion order が canonical bytes に影響しないことを示す hash exclusion tests。

この module-spec task では `.miz` fixture、expectation sidecar、`doc/spec` change は
不要である。

## Gaps and deferred items

| ID | Class | Evidence | Action |
|---|---|---|---|
| CLAUSE-G001 | `source_drift` / `test_gap` | task 3 前には `src/clause.rs` が存在しない。 | Task 3 がこの spec を focused Rust tests とともに実装する。 |
| CLAUSE-G002 | `external_dependency_gap` | `mizar-atp` には normalized certificate と backend trace producer が存在しない。 | Raw backend format は producer-owned として扱う。この module は normalized clause input だけを受け取る。 |
| CLAUSE-G003 | `deferred` | Artifact-facing clause snapshot と certificate corpus fixture は後続 parser/checker/test harness task を必要とする。 | それらの consumer が存在するまでは、task 3 coverage を crate-local に保つ。 |
