# ソース overload 選択
正本: [English owner](../en/source_overload.md)。

## 境界
`SourceOverloadInput` は既存 overload engine の入力型を使い、viability・比較・view の evidence を明示的に運ぶ。
ソース抽出では ordinary/template use、候補宣言の identity、signature、引数、anchor、記述順を検証する。
初期 profile は unary builtin set/object signature と direct type template に限定し、未対応形状は拒否する。
Task277 の native parameter/loci/actual node を消費し、独立した固定 profile の 277A handoff は拡張しない。
Task279 は ordinary root 選択後の refinement を所有する。

## Transaction
`SourceOverloadHandoff` は元の TypedAst と collection・expansion・viability・specificity・selection 出力を保持する。
TypedAst への設置と最終 ResolvedTypedAst assembly は同じ handoff と対応する全段階の出力を要求する。
比較の欠落から順序を推定せず、expected/result type で root を選ばない。source qua view は記述された根拠を要求する。
MT10-AS ordinary-root smoke が最終 projection を消費する。
proof、accepted evidence、登録 activation、semantic coverage credit は追加しない。
