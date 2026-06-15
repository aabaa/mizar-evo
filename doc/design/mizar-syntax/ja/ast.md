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

task 12 の互換 data（`SurfaceNode`、`SurfaceNodeId`、`token_nodes`、`root`、
`expression_root`）は、`SurfaceAst` 内部の private field に backed される。ただし
その surface の一部は移行中の公開 API として残る。互換型、read-only accessor、
`SurfaceNode` の constructor / field は、`mizar-parser`、`mizar-frontend`、既存
test が現在の最小形状を検査し続けられるように export されている。これは公開
互換 API であり、storage backend でも安定 artifact schema でもない。新しい
consumer は `SurfaceNodeView` と typed accessor を優先するべきである。互換 id と
node は cross-run identity として serialize してはならず、consumer が green tree
と独立に mutation することもできない。

### Syntax kind mapping

`SyntaxKind` は rowan の raw kind 語彙である。現在の node kind mapping は以下:

| surface role | raw kind |
|---|---|
| root node | `SyntaxKind::Root` |
| 互換 token node | `SyntaxKind::Token` |
| compilation unit node | `SyntaxKind::CompilationUnit` |
| top-level item list node | `SyntaxKind::ItemList` |
| parser task-5 placeholder item node | `SyntaxKind::PlaceholderItem` |
| module path node | `SyntaxKind::ModulePath` |
| namespace path node | `SyntaxKind::NamespacePath` |
| qualified symbol node | `SyntaxKind::QualifiedSymbol` |
| path segment node | `SyntaxKind::PathSegment` |
| relative import prefix node | `SyntaxKind::RelativePrefix` |
| infix expression node | `SyntaxKind::InfixExpression` |
| recovery node | `SyntaxKind::ErrorRecovery` |

token role は別の raw kind として、identifier、reserved word、reserved symbol、
numeral、lexeme run、user symbol、string literal、error-recovery token、
unknown token を持つ。rowan tree は source-shaped であり、各 token はソース順に
一度だけ rowan token leaf として現れる。互換 side table は task 12 API のために
token payload を保持してよいが、それによって rowan tree 内の token leaf や text
を重複させてはならない。

現在の raw discriminant は、この段階の rowan 境界の一部である。

| raw value | `SyntaxKind` | role |
|---:|---|---|
| 0 | `Unknown` | 認識できない raw rowan kind の fallback |
| 1 | `Root` | root node |
| 2 | `Token` | 互換 token wrapper node |
| 3 | `InfixExpression` | infix expression node |
| 4 | `ErrorRecovery` | recovery node |
| 5 | `ModulePath` | module import/export path node |
| 6 | `NamespacePath` | citation/reference namespace path node |
| 7 | `QualifiedSymbol` | namespace-qualified active user symbol node |
| 8 | `PathSegment` | 単一 identifier または user-symbol segment wrapper |
| 9 | `RelativePrefix` | `.` / `..` import-relative prefix wrapper |
| 10 | `CompilationUnit` | module file skeleton node |
| 11 | `ItemList` | top-level item list node |
| 12 | `PlaceholderItem` | task-5 keyword-dispatched placeholder item node |
| 100 | `TokenIdentifier` | identifier token leaf |
| 101 | `TokenReservedWord` | reserved-word token leaf |
| 102 | `TokenReservedSymbol` | reserved-symbol token leaf |
| 103 | `TokenNumeral` | numeral token leaf |
| 104 | `TokenLexemeRun` | lexeme-run token leaf |
| 105 | `TokenUserSymbol` | user-symbol token leaf |
| 106 | `TokenStringLiteral` | string-literal token leaf |
| 107 | `TokenErrorRecovery` | lexer recovery token leaf |
| 108 | `TokenUnknown` | unknown token leaf |

`SyntaxKind::from_raw` は未知の raw value をすべて `Unknown` に写像する。
`SyntaxKind::is_node_kind` は `Root`、`Token`、`InfixExpression`、`ErrorRecovery`、
上に列挙した task S-009 の共有 path node kind、および task 5 の module skeleton
node kind に対してのみ true であり、`is_token_kind` は token leaf kind に対してのみ
true である。将来の raw value は、既存 snapshot と rowan test が raw 語彙変更時に
明確に失敗するよう、末尾へ追加するか、文書化された予約 range に割り当てるべきである。

### 現在の surface 語彙

現在実装済みの surface node 語彙は意図的に小さい。

| 公開 surface kind | payload | raw rowan node kind | 注記 |
|---|---|---|---|
| `SurfaceNodeKind::Root` | なし | `SyntaxKind::Root` | top-level 互換 root |
| `SurfaceNodeKind::Token(SurfaceToken)` | token kind と interned text | token raw kind の token leaf を 1 つ持つ `SyntaxKind::Token` | rowan token leaf の互換 wrapper |
| `SurfaceNodeKind::CompilationUnit` | なし | `SyntaxKind::CompilationUnit` | parser task 5 の module file skeleton。`ItemList` child を 1 つ持ち、semantic module identity は持たない |
| `SurfaceNodeKind::ItemList` | なし | `SyntaxKind::ItemList` | top-level item placeholder と item-level recovery marker の source-order list |
| `SurfaceNodeKind::PlaceholderItem` | なし | `SyntaxKind::PlaceholderItem` | 後続 task が concrete item node に置き換えるまで使う、keyword-dispatched top-level item placeholder |
| `SurfaceNodeKind::ModulePath` | なし | `SyntaxKind::ModulePath` | `module_path`。任意の `RelativePrefix`、最初の `PathSegment`、続く `.` token + `PathSegment` の反復。この path 形だけが `RelativePrefix` を持てる |
| `SurfaceNodeKind::NamespacePath` | なし | `SyntaxKind::NamespacePath` | `namespace_path`。最初の `PathSegment`、続く `.` token + identifier `PathSegment` の反復。相対 prefix は許さない |
| `SurfaceNodeKind::QualifiedSymbol` | なし | `SyntaxKind::QualifiedSymbol` | `qualified_symbol`。0 個以上の namespace identifier `PathSegment` + `.` token の組に、最後の user-symbol `PathSegment` が続く |
| `SurfaceNodeKind::PathSegment` | なし | `SyntaxKind::PathSegment` | identifier または user-symbol token を 1 つだけ包む。役割は親と token kind で決まる |
| `SurfaceNodeKind::RelativePrefix` | なし | `SyntaxKind::RelativePrefix` | `ModulePath` 先頭の `.` または `..` token を 1 つだけ包む |
| `SurfaceNodeKind::InfixExpression(SurfaceInfixOperator)` | spelling、precedence、associativity | `SyntaxKind::InfixExpression` | task 12 の Pratt expression 形状 |
| `SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind)` | recovery kind | `SyntaxKind::ErrorRecovery` | builder が作る recovery node は recovered |

`SurfaceTokenKind` は、上に挙げた token raw kind に対応する現在の語彙として
`Identifier`、`ReservedWord`、`ReservedSymbol`、`Numeral`、`LexemeRun`、
`UserSymbol`、`StringLiteral`、`ErrorRecovery`、`Unknown` を持つ。
`SurfaceOperatorAssociativity` は現在 `Left`、`Right`、`NonAssociative` を持つ。

`mizar-parser` task 4 のために追加された共有 path node は syntax-only の形である。
node range は、その path または wrapper が所有する最初の token から最後の token
までとする。親 path node は子を source order で列挙する。segment 間の separator
`.` token は `PathSegment` で包まず、親 path node の直接 child とする。これらの
node 自体は recovery node や trivia entry を生成しない。missing path diagnostic、
skipped-token trivia、doc-comment attachment は消費側の文法タスクが所有する。
`SurfaceNodeView` は `as_module_path`、`as_namespace_path`、`as_qualified_symbol`、
`as_path_segment`、`as_relative_prefix` の typed helper を公開し、consumer がこれらの
共有 path 形のために生の rowan traversal を使わずに済むようにする。

`mizar-parser` task 5 で追加された module skeleton node は syntax-only の形である。
`CompilationUnit` は source file surface を表し、`ItemList` child をちょうど 1 つ
所有する。`ItemList` の child は source order の `PlaceholderItem` node と、
`SkippedToken` のような item-level recovery node である。`PlaceholderItem` は
top-level item boundary 1 つとして消費された source token を包み、annotation
prefix や終端セミコロンを欠いた回復済み item も含める。parser はこれらの node に
import resolution、visibility semantics、theorem validity、symbol identity を
encode してはならない。
`SurfaceNodeView` は `as_compilation_unit`、`as_item_list`、
`as_placeholder_item` の typed helper を公開する。後続 item への leading
doc-comment attachment は、comment text を item node にコピーせず、`SurfaceTrivia`
で表現する。

### 語彙増分の契約

node 語彙は、その形を構築する `mizar-parser` 文法タスクと同じ変更でのみ増やす。
各増分では、実装と同時または先行して、追加する各公開 syntax kind について
この仕様に次の契約を書く。

- `SurfaceNodeKind` variant 名と raw `SyntaxKind` mapping。
- payload field がある場合、その内容と、それが parser fact なのか互換 data なのか。
- child role と child order。optional / repeated role も含める。
- node と child の range rule。文書化された recovery 例外も含める。
- 生の rowan traversal ではなく consumer が使うべき typed accessor / view helper。
- 新しい kind の snapshot rendering text と、escaping / sorting rule。
- skipped token、欠落 construct、doc-comment attachment、空白依存 hint を所有する場合の
  recovery / trivia との相互作用。

`doc/spec/ja/` 配下の言語文法は、どの構文要素が存在するかを定義する。この
モジュール仕様は、それらを `SurfaceAst` でどう表現するかを定義する。

### Builder 境界

`SurfaceAstBuilder` は parser 向けの構築境界である。parser code は builder
method 経由で token、通常 node、recovery node を追加し、root と任意の
expression root を指定して finish する。parser grammar code は private arena
へ直接 push したり、rowan node を直接確保したり、生の rowan traversal に依存
したりしてはならない。文法拡張で新しい tree 操作が必要になった場合は、まず
ここに typed builder または accessor として追加する。

builder id は 1 つの builder instance に局所的である。別 builder 由来の child、
root、expression-root id は無効である。`add_node` は通常の structural node だけを
作る。token node は `add_token` または `add_recovered_token`、recovery node は
`add_recovery` で作らなければならない。`finish` は、任意の root と expression
root が存在すること、また non-root の structural parent が child subtree を共有
していないことを検証する。

構築中、parser 基盤は `node_kind` や `node_range` のような typed builder accessor
を通じて、すでに送出した builder node を検査してよい。これらの accessor は parser
composition に必要な surface kind と source range だけを公開し、private builder
arena を storage contract として露出しない。

互換 root は、task 12 の consumer が両方の view を検査し続けられるよう、ソース順
の token node と、それらの token を含む structural node の両方を列挙してよい。
rowan green tree は source-shaped のままである。structural child が source token
を所有する場合、builder は互換 root listing から token leaf を重複させず、その
structural rowan node の下に一度だけ出力しなければならない。Recovery node は自身
の insertion range の外にある context child を互換 view に保持してよいが、その
out-of-range context child は recovery rowan node の下には出力しない。

現在の rowan construction は、root に列挙された token node が non-recovery の
structural root child の descendant でもある場合にのみ deduplicate する。root の
互換 listing にもあり、かつ recovery node の in-range child としても含まれる token
は deduplicate しない。将来の builder check または rowan emission rule がこの
case を扱うまでは、parser producer は、root token listing にも現れる in-range
token child を包む recovery node を作ってはならない。missing-construct recovery
には out-of-range context child を使い、skip された source span は重複 token leaf
を包むのではなく trivia に記録する。

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

`SurfaceAst::snapshot_text_with_trivia` は、[trivia.md](./trivia.md) で定義する
決定的な trivia side table を追加して描画する。既定の syntax snapshot はその
section を省略し、既存の syntax-only baseline を安定させる。

現在の syntax snapshot format は次のとおり。

```text
surface-ast-snapshot-v1
root:
  <node-or-none>
expression_root:
  <node-or-none>
token_nodes:
  <node-or-none>
```

node 行は depth ごとに 2 space で indent し、現在は次の形を使う。

```text
Root range=<start>..<end> recovered=<bool>
Token kind=<SurfaceTokenKind> text="<escaped-text>" range=<start>..<end> recovered=<bool>
CompilationUnit range=<start>..<end> recovered=<bool>
ItemList range=<start>..<end> recovered=<bool>
PlaceholderItem range=<start>..<end> recovered=<bool>
ModulePath range=<start>..<end> recovered=<bool>
NamespacePath range=<start>..<end> recovered=<bool>
QualifiedSymbol range=<start>..<end> recovered=<bool>
PathSegment range=<start>..<end> recovered=<bool>
RelativePrefix range=<start>..<end> recovered=<bool>
InfixExpression spelling="<escaped-text>" precedence=<u8> associativity=<SurfaceOperatorAssociativity> range=<start>..<end> recovered=<bool>
ErrorRecovery kind=<SyntaxRecoveryKind> range=<start>..<end> recovered=<bool>
```

`<escaped-text>` は Rust の default character escaping を使うため、制御文字、
quote、backslash、非表示 character は決定的に描画される。snapshot format を変更する
場合は、新しい header version に加え、この仕様、日本語 companion、影響を受ける
baseline snapshot の更新が必要である。外側の snapshot envelope または update policy
が変わる場合にのみ、`mizar-test` snapshot documentation を更新する。

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

### 公開 enum の互換性

現在の公開 syntax enum は、まだ長命な resolver / LSP surface ではない。parser
task 5〜7 により downstream input として現実的になる前に、[todo.md](./todo.md)
の consumer 前ゲートを適用する。将来の語彙増加を約束する enum
（`SyntaxKind`、`SurfaceNodeKind`、`SurfaceTokenKind`）は、下流 crate 向けに
`#[non_exhaustive]` とし、lint-policy gate がこれらの属性を固定する。
`MizarLanguage` は downstream の syntax category ではなく空の rowan marker enum
であるため、意図的に exhaustive のままとする。`SurfaceOperatorAssociativity` は現在、
閉じた三分の operator property（`Left`、`Right`、`NonAssociative`）であり、後続の
operator-model task が新しい associativity category を設計しない限り、意図的に
exhaustive のままとする。この crate 内部の match は exhaustive のままにし、新しい
variant 追加時にローカル更新がコンパイル時に促されるようにする。下流 crate は
`#[non_exhaustive]` により必要になる箇所で wildcard fallback arm を含めなければならない。
