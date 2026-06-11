# mizar-syntax: Surface AST

> 正本は英語です。英語版: [../en/ast.md](../en/ast.md)。

状態: rowan-backed storage 境界と task 12 互換 view は実装済み。完全な AST 範囲は計画中。

## 目的

このモジュールは、`mizar-parser` が生成する、ソースの形を保った `SurfaceAst` を定義する。
`SurfaceAst` は immutable な rowan green tree を backend とする。現在の
`SurfaceNode` / `SurfaceNodeId` surface は、parser と frontend が task 12 の
最小 tree 形状から移行する間の互換 view として残す。

## 責務

- `SurfaceAst`、rowan syntax kind、互換用の構文ノード ID、parser 向け構築 API を定義する。
- ソース順、ソース範囲、回復ノードを保持する。
- モジュール、項目、項、論理式、文、証明、アルゴリズム、アノテーションを表現する。
- 解決済みシンボル ID、推論済み型、overload resolution result、cluster fact、証明義務を持たない。

## 公開 API

### Storage 境界

`SurfaceAst` は rowan green tree を所有する。rowan は構文形状と決定的共有の
storage backend であり、compiler の意味的 identity surface ではない。消費者は
storage 境界そのものをテストする場合を除き、`SurfaceAst` と
`SurfaceNodeView` の typed accessor を使う。生の rowan root は
`SurfaceAst::rowan_root` から、green node は `SurfaceAst::green_node` から取得
できるが、用途は infrastructure test と明示的に文書化された統合に限る。

task 12 の `SurfaceNode` vector、`SurfaceNodeId`、`token_nodes`、`root`、
`expression_root` は、typed accessor 経由で公開される private な互換 side
table であり、rowan-backed AST 上の view である。`mizar-parser` と
`mizar-frontend` が現在の最小形状を検査し続けられるように残すが、storage
backend や安定 artifact id として扱ってはならない。また、消費者が green tree
と独立に mutation することはできない。

### Syntax kind mapping

`SyntaxKind` は rowan の raw kind 語彙である。現在の node kind mapping は以下:

| surface role | raw kind |
|---|---|
| root node | `SyntaxKind::Root` |
| 互換 token node | `SyntaxKind::Token` |
| infix expression node | `SyntaxKind::InfixExpression` |
| recovery node | `SyntaxKind::ErrorRecovery` |

token role は別の raw kind として、identifier、reserved word、reserved symbol、
numeral、lexeme run、user symbol、string literal、error-recovery token、
unknown token を持つ。rowan tree は source-shaped であり、各 token はソース順に
一度だけ rowan token leaf として現れる。互換 side table は task 12 API のために
token payload を保持してよいが、それによって rowan tree 内の token leaf や text
を重複させてはならない。

### Builder 境界

`SurfaceAstBuilder` は parser 向けの構築境界である。parser code は builder
method 経由で token、通常 node、recovery node を追加し、root と任意の
expression root を指定して finish する。parser grammar code は private arena
へ直接 push したり、rowan node を直接確保したり、生の rowan traversal に依存
したりしてはならない。文法拡張で新しい tree 操作が必要になった場合は、まず
ここに typed builder または accessor として追加する。

### Accessor 規約

`SurfaceAst::node_view`、`root_view`、`expression_view`、`token_views` は typed
view を返し、rowan traversal を要求せずに kind、range、recovered flag、children、
token payload、infix payload、recovery kind を公開する。互換用の
`SurfaceAst::node` accessor は既存テストと移行コードのために残す。

### Snapshot rendering

`SurfaceAst::snapshot_text` は、syntax test と後続の parser corpus baseline が使う、
決定的で人間可読な surface snapshot format を返す。format は
`surface-ast-snapshot-v1` header で version 付けされ、root view、任意の
expression root、token 互換 view を保存順で安定して描画する。各 node 行には
surface kind、source-local な byte range、`recovered` flag、および現在の構文語彙
を区別するための kind 固有 payload（token kind/text、infix の
spelling/precedence/associativity、または recovery kind）を含める。

snapshot text は、rowan pointer identity、builder id、`SurfaceNodeId` 値、
生の `SourceId` debug 出力、absolute path、実行時間、hash-map iteration order、
その他の非決定的データを意図的に含めない。range は `SurfaceAst` の source 内の
byte offset として描画する。source identity は `mizar-test` が所有する外側の
snapshot/profile record の責務である。

### Range attachment

各 surface node は `mizar-session` の `SourceRange` を持つ。通常 node では親の
range がすべての子の range を包含する。recovery node は、zero-width insertion
node が opener や skipped token を context として保持する場合、この包含関係を
破ってよい。たとえば missing-`end` recovery node は EOF の挿入 range に付き、
子は block opener を指し戻す。

### Identity rules

rowan green-node identity、rowan text range、dense な `SurfaceNodeId` は内部
cache と互換性の詳細である。構築済み `SurfaceAst` の中では決定的だが、安定
artifact id ではなく、cross-run identity として serialize してはならない。
安定した消費者は deterministic snapshot、content cache key、source id/range、
および後段の resolver/checker layer が所有する semantic id を key にする。
