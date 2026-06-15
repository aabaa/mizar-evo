# mizar-parser: Pratt Parsing

> 正本は英語です。英語版: [../en/pratt.md](../en/pratt.md)。

状態: task 12 の項 Pratt 解析は、active lexicon 由来の prefix、postfix、infix
演算子向けに実装済みである。task 13 の atomic formula は項 Pratt boundary を使う。
formula connective の優先順位解析は引き続き計画中である。

## 目的

このモジュールは、項と論理式の優先順位 parser を定義する。

## 責務

- 項レベルの prefix、postfix、infix 形式には、アクティブ語彙の演算子メタデータを用いる。
- 論理式レベルの優先順位には、固定された論理結合子テーブルを用いる。
- オーバーロード解決を行わず、構文上の形だけを解析する。
- 非結合演算子の連鎖や意外な優先順位について、ソース局所的な診断を出す。

## 項 Pratt 契約

`ParseRequest::operator_fixity` は、`ParserInputs` から導出された parser-facing
operator metadata を運ぶ。内容は spelling、fixity kind、precedence、そして infix
operator については associativity である。parser はこの metadata を文法設定としてのみ
消費する。overload root の選択、result type 推論、cluster fact の評価、
selector-versus-namespace role の解決は行わない。

項 parsing は次の順序を使う。

1. Prefix operator は null denotation である。operand はその operator precedence で parse する。
2. Primary term と固定の selector/update postfix chain を次に parse する。そのため
   selector/update/application syntax は user operator より強く bind する。
3. User postfix / infix operator は Pratt binding power で畳み込む。
4. `qua` は Pratt の後、固定の最も低い term-level operator として module grammar が parse し、
   left-associative のままにする。

atomic-formula parsing は項 Pratt boundary の後から始まる。Task 13 はすでに parse 済みの
term operand の周辺で、built-in predicate、`is` assertion、inline predicate call、
syntax-only user predicate segment を消費する。固定 formula connective Pratt table は task 14 が所有する。

Infix term operator の binding power は Appendix B と一致する。

| Associativity | Left binding power | Right minimum binding power |
|---|---:|---:|
| Left | `N` | `N + 1` |
| Right | `N` | `N` |
| None | `N` | `N + 1`、加えて同一 operator chain を診断する |

Prefix / postfix operator は、供給された precedence を binding power として使う。この
metadata は明示宣言または spec-defaulted summary-side metadata から来てよい。1 つの
token に異なる fixity の visible operator entry が複数ある場合、parser は term operand
開始位置では prefix entry だけを使い、left operand を parse した後は postfix / infix
entry だけを使う。left operand の後で postfix entry と infix entry が同じ spelling を
共有する場合、eligible な infix entry は後続 token が right operand を開始できるなら
優先される。そうでなければ eligible な postfix entry が優先される。同一 spelling
operator の incompatible metadata conflict は lexical-environment または link stage の
error である。この parser stage は `ParserInputs` が deterministic な visible table を
すでに選んでいると仮定する。

## 公開 enum の互換性

`OperatorAssociativity` は意図的に exhaustive のままとする。これは
`mizar-syntax::SurfaceOperatorAssociativity` と同じ三つの意味、つまり左結合、
右結合、非結合を持つ、閉じた parser-facing fixity property である。将来の operator
model 変更で別の associativity category が必要になった場合は、この design note、
parser match、syntax payload mapping、lint-policy expectation を同じ変更で更新する。

`OperatorFixity` も現在の term operator model では意図的に exhaustive とする。
variant は prefix、infix、postfix である。Bracket delimiter-pair functor は、
parser input が bracket-pair metadata を得るまでこの enum では表さない。
