# mizar-parser: Pratt Parsing

> 正本は英語です。英語版: [../en/pratt.md](../en/pratt.md)。

状態: task 12 の項 Pratt 解析は、active lexicon 由来の prefix、postfix、infix
演算子向けに実装済みである。task 13 の atomic formula は項 Pratt boundary を使う。
task 14 の fixed formula connective precedence は実装済みである。task 15 の
set-comprehension primary は実装済みである。

## 目的

このモジュールは、項と論理式の優先順位 parser を定義する。

## 責務

- 項レベルの prefix、postfix、infix 形式には、アクティブ語彙の演算子メタデータを用いる。
- 論理式レベルの優先順位には、固定された論理結合子テーブルを用いる。
- オーバーロード解決を行わず、構文上の形だけを解析する。
- 非結合演算子の連鎖や意外な優先順位について、ソース局所的な診断を出す。

## 項 Pratt 契約

`ParseRequest::operator_fixity` は、`ParserInputs` から導出された source-position-aware `ParserOperatorView` の現在の転送形である。内容は spelling、fixity kind、precedence、source-coordinate の `active_from`、そして infix operator については associativity である。Pratt lookup は operator token span で active な operator metadata を問い合わせなければならない。parser はこの metadata を文法設定としてのみ消費する。overload root の選択、result type 推論、cluster fact の評価、proof obligation の生成、`ObligationAnchor` の構築、`DependencySlice` の生成、selector-versus-namespace role の解決は行わない。

`infix_operator`、`prefix_operator`、`postfix_operator` の宣言は後続のトークンにだけ
影響するため、Pratt の検索は演算子トークンの source span で metadata を filter する。
`active_from > token.span.start` の entry は無視する。operator metadata は spelling
単位である。ある token spelling について、Pratt lookup は fixity に関係なく最も新しく
アクティブな metadata entry を先に選び、その entry が現在の prefix/postfix/infix
context で有効かを後で確認する。そのため、同じ spelling に対する新しい prefix 宣言は、
後続 token について古い infix 宣言を shadow し、古い infix を引き続き利用可能にはしない。
同じ activation point の tie は決定的に保ち、この parser stage の前に link-time conflict
が検査されていると仮定する。

項 parsing は次の順序を使う。

1. Prefix operator は null denotation である。operand はその operator precedence で parse する。
2. Primary term と固定の selector/update postfix chain を次に parse する。そのため
   selector/update/application syntax は user operator より強く bind する。task 15 で
   追加する set comprehension は primary term であり、その mapper child は同じ term
   Pratt boundary、任意の condition child は固定 formula Pratt boundary で parse する。
3. User postfix / infix operator は Pratt binding power で畳み込む。
4. `qua` は Pratt の後、固定の最も低い term-level operator として module grammar が parse し、
   left-associative のままにする。

atomic-formula parsing は項 Pratt boundary の後から始まる。Task 13 はすでに parse 済みの
term operand の周辺で、built-in predicate、`is` assertion、inline predicate call、
syntax-only user predicate segment を消費する。

Infix term operator の binding power は Appendix B と一致する。

| Associativity | Left binding power | Right minimum binding power |
|---|---:|---:|
| Left | `N` | `N + 1` |
| Right | `N` | `N` |
| None | `N` | `N + 1`、加えて同一 operator chain を診断する |

Prefix / postfix operator は、供給された precedence を binding power として使う。この
metadata は明示宣言または spec-defaulted summary-side metadata から来てよい。最も新しく
アクティブな spelling entry を選んだ後、parser はその fixity が構文上 eligible な context
でだけそれを使う。prefix entry は term operand の開始位置、postfix / infix entry は left
operand を parse した後だけである。同一 spelling operator の incompatible metadata
conflict は lexical-environment または link stage の error である。この parser stage は、
`ParserInputs` が parse 中のトークン位置に対する決定的な visible entry order をすでに
選んでいると仮定する。

`ParserInputsHash` は range-aware operator view またはその canonical fingerprint を含めなければならない。parser node order、`SourceRange`、`VcId` の一致を proof-reuse authority として扱ってはならない。

## Formula Pratt Contract

Task 14 は import-dependent operator metadata ではなく fixed formula parser を使う。
atomic formula、parenthesized formula、`thesis`、`contradiction` は primary formula
operand である。prefix `not` は prefix binding level で formula operand を 1 つ parse する。
binary connective は Appendix B の fixed hierarchy を使う。

| Operator | Associativity | Relative binding |
|---|---|---:|
| `&` | left | 50 |
| `or` | left | 40 |
| `implies` | right | 30 |
| `iff` | none | 20 |

numeric binding power は parser-local constant であり、relative order が contract である。
`iff` は同じ unparenthesized chain の top-level `iff` が続く場合に
`NonAssociativeOperatorChain` を送出する。repetition form の `& ... &` と
`or ... or` は non-repetition connective と同じ precedence / associativity を持ち、
追加の `...` token を保持した同じ binary formula node で表す。

Quantifier は user operator ではない。`for` と `ex` は Pratt binary table より前に
outermost formula form として parse する。Chapter 14 が
`( ... | quantified_formula )` を許す right operand では quantified formula が現れてよく、
`P implies for x being T holds Q` は `P implies (for x being T holds Q)` と group する。
quantifier variable typing は syntactic only であり、`reserve` 由来の implicit variable
typing は後段の resolution に属する。

## 公開 enum の互換性

`OperatorAssociativity` は意図的に exhaustive のままとする。これは
`mizar-syntax::SurfaceOperatorAssociativity` と同じ三つの意味、つまり左結合、
右結合、非結合を持つ、閉じた parser-facing fixity property である。将来の operator
model 変更で別の associativity category が必要になった場合は、この design note、
parser match、syntax payload mapping、lint-policy expectation を同じ変更で更新する。

`OperatorFixity` も現在の term operator model では意図的に exhaustive とする。
variant は prefix、infix、postfix である。Bracket delimiter-pair functor は、
parser input が bracket-pair metadata を得るまでこの enum では表さない。
