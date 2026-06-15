# mizar-parser: Pratt Parsing

> 正本は英語です。英語版: [../en/pratt.md](../en/pratt.md)。

状態: task 11 の明示的な infix Pratt 解析は最小実装済みで、task 1 の module
split により内部 `pratt` module として配置済み。完全な項／論理式の優先順位解析は
計画中。

## 目的

このモジュールは、項と論理式の優先順位 parser を定義する。

## 責務

- 項レベルの prefix、postfix、infix 形式には、アクティブ語彙の演算子メタデータを用いる。
- 論理式レベルの優先順位には、固定された論理結合子テーブルを用いる。
- オーバーロード解決を行わず、構文上の形だけを解析する。
- 非結合演算子の連鎖や意外な優先順位について、ソース局所的な診断を出す。

## 公開 enum の互換性

`OperatorAssociativity` は意図的に exhaustive のままとする。これは
`mizar-syntax::SurfaceOperatorAssociativity` と同じ三つの意味、つまり左結合、
右結合、非結合を持つ、閉じた parser-facing fixity property である。将来の operator
model 変更で別の associativity category が必要になった場合は、この design note、
parser match、syntax payload mapping、lint-policy expectation を同じ変更で更新する。
