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
| concrete export item node | `SyntaxKind::ExportItem` |
| visibility marker node | `SyntaxKind::VisibilityMarker` |
| visible item wrapper node | `SyntaxKind::VisibleItem` |
| reserve item node | `SyntaxKind::ReserveItem` |
| reserve segment node | `SyntaxKind::ReserveSegment` |
| type expression node | `SyntaxKind::TypeExpression` |
| attribute chain node | `SyntaxKind::AttributeChain` |
| attribute reference node | `SyntaxKind::AttributeRef` |
| parameter prefix node | `SyntaxKind::ParameterPrefix` |
| generic type head node | `SyntaxKind::TypeHead` |
| type arguments node | `SyntaxKind::TypeArguments` |
| legacy bracket term placeholder node | `SyntaxKind::TermPlaceholder` |
| term expression node | `SyntaxKind::TermExpression` |
| term reference node | `SyntaxKind::TermReference` |
| numeral term node | `SyntaxKind::NumeralTerm` |
| `it` term node | `SyntaxKind::ItTerm` |
| parenthesized term node | `SyntaxKind::ParenthesizedTerm` |
| choice term node | `SyntaxKind::ChoiceTerm` |
| application term node | `SyntaxKind::ApplicationTerm` |
| structure constructor node | `SyntaxKind::StructureConstructor` |
| field argument node | `SyntaxKind::FieldArgument` |
| set enumeration node | `SyntaxKind::SetEnumeration` |
| `qua` expression node | `SyntaxKind::QuaExpression` |
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
| 7 | `QualifiedSymbol` | dotted active user symbol node。attribute-ref の structure prefix も含む |
| 8 | `PathSegment` | 単一 identifier または user-symbol segment wrapper |
| 9 | `RelativePrefix` | `.` / `..` import-relative prefix wrapper |
| 10 | `CompilationUnit` | module file skeleton node |
| 11 | `ItemList` | top-level item list node |
| 12 | `PlaceholderItem` | task-5 keyword-dispatched placeholder item node |
| 13 | `ImportItem` | task-6 concrete `import` item node |
| 14 | `ImportAliasDecl` | task-6 simple import または alias declaration node |
| 15 | `ModuleBranchImport` | task-6 branch import declaration node |
| 16 | `ExportItem` | task-7 concrete `export` item node |
| 17 | `VisibilityMarker` | task-7 `private` / `public` token wrapper |
| 18 | `VisibleItem` | task-7 visible top-level item wrapper |
| 19 | `ReserveItem` | task-8 concrete top-level `reserve` host item |
| 20 | `ReserveSegment` | task-8 `identifier_list "for" type_expression` segment |
| 21 | `TypeExpression` | task-8 `attribute_chain type_head` node |
| 22 | `AttributeChain` | task-8 non-empty attribute reference 列 |
| 23 | `AttributeRef` | task-8 任意の `non` を含む syntactic attribute reference |
| 24 | `ParameterPrefix` | task-8 attribute parameter-prefix wrapper |
| 25 | `TypeHead` | task-8 generic radix-or-mode type head |
| 26 | `TypeArguments` | task-8 `of` / `over` / bracket argument wrapper |
| 27 | `TermPlaceholder` | task 8 の legacy bracket `qua_arg` stub。task 11 の parser path では生成しない |
| 28 | `TermExpression` | task-9 current term-expression wrapper |
| 29 | `TermReference` | task-9 identifier または qualified-symbol term reference |
| 30 | `NumeralTerm` | task-9 numeral term |
| 31 | `ItTerm` | task-9 `it` term |
| 32 | `ParenthesizedTerm` | task-9 parenthesized term |
| 33 | `ChoiceTerm` | task-9 `"the" type_expression` term |
| 34 | `ApplicationTerm` | task-9 parenthesized / reserved-bracket application term |
| 35 | `StructureConstructor` | task-9 named-field structure-constructor surface |
| 36 | `FieldArgument` | task-9 structure-constructor field argument |
| 37 | `SetEnumeration` | task-9 set-enumeration term |
| 38 | `SelectorAccess` | task-10 selector postfix / selector-call surface |
| 39 | `StructureUpdate` | task-10 functional structure-update postfix |
| 40 | `FieldUpdate` | task-10 structure-update field assignment |
| 41 | `QuaExpression` | task-11 の `term "qua" type_expression` qualification surface |
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
上に列挙した task S-009 の共有 path node kind、task 5 の module skeleton node kind /
task 6 の import node kind / task 7 の export と visibility node kind、および task
S-010 の reserve / type node kind、さらに上に列挙した task S-011 の term node kind
に対して true であり、`is_token_kind` は token leaf kind に対してのみ true である。
将来の raw value は、既存 snapshot と rowan test が raw 語彙変更時に明確に失敗するよう、
末尾へ追加するか、文書化された予約 range に割り当てるべきである。

### 現在の surface 語彙

現在実装済みの surface node 語彙は意図的に小さい。

| 公開 surface kind | payload | raw rowan node kind | 注記 |
|---|---|---|---|
| `SurfaceNodeKind::Root` | なし | `SyntaxKind::Root` | top-level 互換 root |
| `SurfaceNodeKind::Token(SurfaceToken)` | token kind と interned text | token raw kind の token leaf を 1 つ持つ `SyntaxKind::Token` | rowan token leaf の互換 wrapper |
| `SurfaceNodeKind::CompilationUnit` | なし | `SyntaxKind::CompilationUnit` | parser task 5 の module file skeleton。`ItemList` child を 1 つ持ち、semantic module identity は持たない |
| `SurfaceNodeKind::ItemList` | なし | `SyntaxKind::ItemList` | top-level item placeholder と item-level recovery marker の source-order list |
| `SurfaceNodeKind::PlaceholderItem` | なし | `SyntaxKind::PlaceholderItem` | 後続 task が concrete item node に置き換えるまで使う、keyword-dispatched top-level item placeholder |
| `SurfaceNodeKind::ImportItem` | なし | `SyntaxKind::ImportItem` | parser task 6 の concrete `import_stmt`。`import` token、comma token で区切られた import declaration node、任意の malformed-tail recovery、任意の semicolon token を所有する |
| `SurfaceNodeKind::ImportAliasDecl` | なし | `SyntaxKind::ImportAliasDecl` | parser task 6 の `module_path ["as" module_identifier]`。`ModulePath`、任意の `as` token、任意の alias `PathSegment`、任意の malformed-tail recovery を所有する |
| `SurfaceNodeKind::ModuleBranchImport` | なし | `SyntaxKind::ModuleBranchImport` | parser task 6 の `module_path ".{" module_identifier { "," module_identifier } "}"`。base `ModulePath`、`.{` token、comma token で区切られた branch `PathSegment`、任意の malformed-tail recovery、任意の `}` を所有する |
| `SurfaceNodeKind::ExportItem` | なし | `SyntaxKind::ExportItem` | parser task 7 の concrete `export_stmt`。`export` token、comma token で区切られた exported `ModulePath` node、任意の malformed-tail recovery、任意の semicolon token を所有する |
| `SurfaceNodeKind::VisibilityMarker` | なし | `SyntaxKind::VisibilityMarker` | parser task 7 の `private` または `public` token 1 個だけを包む wrapper |
| `SurfaceNodeKind::VisibleItem` | なし | `SyntaxKind::VisibleItem` | parser task 7 の top-level visibility wrapper。annotation-prefix token があればそれら、1 個の `VisibilityMarker`、現在の target item node を所有する |
| `SurfaceNodeKind::ReserveItem` | なし | `SyntaxKind::ReserveItem` | parser task 8 の concrete top-level `reserve_decl` host item。`reserve` token、1 個の `ReserveSegment`、任意の malformed-tail recovery、任意の semicolon token を所有する |
| `SurfaceNodeKind::ReserveSegment` | なし | `SyntaxKind::ReserveSegment` | parser task 8 の `identifier_list "for" type_expression`。comma token で区切られた identifier token、`for` token、`TypeExpression` または missing-type recovery を所有する |
| `SurfaceNodeKind::TypeExpression` | なし | `SyntaxKind::TypeExpression` | parser task 8 の `attribute_chain type_head`。任意の non-empty `AttributeChain` と generic `TypeHead` を所有する |
| `SurfaceNodeKind::AttributeChain` | なし | `SyntaxKind::AttributeChain` | parser task 8 の non-empty `AttributeRef` node 列 |
| `SurfaceNodeKind::AttributeRef` | なし | `SyntaxKind::AttributeRef` | parser task 8 の任意の `non` token、任意の `ParameterPrefix`、syntactic `QualifiedSymbol`、任意の parenthesized term argument |
| `SurfaceNodeKind::ParameterPrefix` | なし | `SyntaxKind::ParameterPrefix` | parser task 8 の attribute parameter prefix。`parameter "-"` または `"(" parameter_list ")" "-"` |
| `SurfaceNodeKind::TypeHead` | なし | `SyntaxKind::TypeHead` | parser task 8 の generic radix-or-mode head。builtin `object`/`set` token または `QualifiedSymbol` と、任意の `TypeArguments` を所有する |
| `SurfaceNodeKind::TypeArguments` | なし | `SyntaxKind::TypeArguments` | parser task 8 の `of`、`over`、または bracket 構文の type argument wrapper。task 9 は `of`/`over` placeholder を `TermExpression` argument に置き換え、task 11 は bracket `qua_arg` placeholder を `TermExpression` / `QuaExpression` surface に置き換える |
| `SurfaceNodeKind::TermPlaceholder` | なし | `SyntaxKind::TermPlaceholder` | raw-kind 互換性のために残る parser task 8 の legacy syntax-only term-entry stub。task 11 の parser は bracket `qua_arg` 形に対してこれを生成しない |
| `SurfaceNodeKind::TermExpression` | なし | `SyntaxKind::TermExpression` | parser task 9 の current term-expression wrapper。primary term、postfix chain、`QuaExpression`、後続 operator expression のいずれか 1 つの current term-shape child を所有する |
| `SurfaceNodeKind::TermReference` | なし | `SyntaxKind::TermReference` | parser task 9 の term position の identifier token または共有 `QualifiedSymbol`。semantic classification は持たない |
| `SurfaceNodeKind::NumeralTerm` | なし | `SyntaxKind::NumeralTerm` | parser task 9 の numeral term wrapper |
| `SurfaceNodeKind::ItTerm` | なし | `SyntaxKind::ItTerm` | parser task 9 の `it` keyword term wrapper |
| `SurfaceNodeKind::ParenthesizedTerm` | なし | `SyntaxKind::ParenthesizedTerm` | parser task 9 の parenthesized term。`(`、`TermExpression` または `MissingTerm`、任意の `)` を所有する |
| `SurfaceNodeKind::ChoiceTerm` | なし | `SyntaxKind::ChoiceTerm` | parser task 9 の `"the" TypeExpression` choice term。type operand 欠落時は `MissingTypeExpression` recovery を使う |
| `SurfaceNodeKind::ApplicationTerm` | なし | `SyntaxKind::ApplicationTerm` | parser task 9 の ordinary parenthesized application または reserved-bracket functor application。delimiter と source-order term argument を所有する |
| `SurfaceNodeKind::StructureConstructor` | なし | `SyntaxKind::StructureConstructor` | parser task 9 の、named field argument が見える場合の syntax-only structure-constructor surface |
| `SurfaceNodeKind::FieldArgument` | なし | `SyntaxKind::FieldArgument` | parser task 9 の `identifier ":" term_expression` field argument |
| `SurfaceNodeKind::SetEnumeration` | なし | `SyntaxKind::SetEnumeration` | parser task 9 の set-enumeration term。Fraenkel/comprehension form は parser task 15 |
| `SurfaceNodeKind::SelectorAccess` | なし | `SyntaxKind::SelectorAccess` | parser task 10 の postfix selector access または selector-call surface。syntax-only dot role を保持する |
| `SurfaceNodeKind::StructureUpdate` | なし | `SyntaxKind::StructureUpdate` | parser task 10 の functional `term "with" "(" field_update_list ")"` update surface |
| `SurfaceNodeKind::FieldUpdate` | なし | `SyntaxKind::FieldUpdate` | parser task 10 の、`StructureUpdate` 内の `selector ":=" term_expression` field update |
| `SurfaceNodeKind::QuaExpression` | なし | `SyntaxKind::QuaExpression` | parser task 11 の type qualification。child order は base term-shape、`qua` token、`TypeExpression` または `MissingTypeExpression` recovery |
| `SurfaceNodeKind::ModulePath` | なし | `SyntaxKind::ModulePath` | `module_path`。任意の `RelativePrefix`、最初の `PathSegment`、続く `.` token + `PathSegment` の反復。この path 形だけが `RelativePrefix` を持てる |
| `SurfaceNodeKind::NamespacePath` | なし | `SyntaxKind::NamespacePath` | `namespace_path`。最初の `PathSegment`、続く `.` token + identifier `PathSegment` の反復。相対 prefix は許さない |
| `SurfaceNodeKind::QualifiedSymbol` | なし | `SyntaxKind::QualifiedSymbol` | `qualified_symbol`。0 個以上の namespace identifier `PathSegment` + `.` token の組に最後の user-symbol `PathSegment` が続く形、または task 8 の attribute-ref flattening として、最後の user-symbol の前に user-symbol token の dotted prefix `PathSegment` も許す形 |
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
所有する。`ItemList` の child は source order の concrete item node、
`PlaceholderItem` node、`SkippedToken` のような item-level recovery node である。
`PlaceholderItem` は top-level item boundary 1 つとして消費された source token を包み、
annotation prefix や終端セミコロンを欠いた回復済み item も含める。parser はこれらの
node に import resolution、visibility semantics、theorem validity、symbol identity を
encode してはならない。
`SurfaceNodeView` は `as_compilation_unit`、`as_item_list`、
`as_placeholder_item` の typed helper を公開する。後続 item への leading
doc-comment attachment は、comment text を item node にコピーせず、`SurfaceTrivia`
で表現する。

`mizar-parser` task 6 で追加された import node は syntax-only の形である。
`ImportItem` は import prelude が開いている間の `import_stmt` 1 つを表す。well-formed
input では、child は source order で、`import` token、comma token で区切られた
1 個以上の `ImportAliasDecl` または `ModuleBranchImport` node、semicolon token である。
malformed recovery では、`import` の後に declaration を持たない `ImportItem`、
後続 declaration のない trailing comma、または semicolon の前に消費した malformed
source に対する `SkippedToken` recovery child が現れ得る。`ImportAliasDecl` は
import される `ModulePath`、任意の `as` token、任意の alias `PathSegment` を所有する。
不正 alias では `MalformedImport` diagnostic を伴い、alias segment が欠けてよく、
nested `SkippedToken` recovery を含み得る。`ModuleBranchImport` は well-formed input
では base `ModulePath`、`.{` token、comma token で区切られた branch `PathSegment`
child、`}` を所有する。不正 branch import では `MalformedImport` を伴い、branch
segment または close token が欠けてよく、nested `SkippedToken` recovery を含み得る。
これらの node は relative `ModulePath` prefix を含み得るが、module resolution、
branch import の semantic import への分割、export availability 検査、alias 割り当ては
行わない。`SurfaceNodeView` は `as_import_item`、`as_import_alias_decl`、
`as_module_branch_import` の typed helper を公開する。

`mizar-parser` task 7 で追加された export / visibility node は syntax-only の形である。
`ExportItem` は export prelude が開いている間の `export_stmt` 1 つを表す。well-formed
input では、child は source order で、`export` token、comma token で区切られた
1 個以上の `ModulePath` node、semicolon token である。malformed recovery では、
`export` の後に path を持たない `ExportItem`、後続 path のない trailing comma、
または semicolon の前に消費した malformed source に対する nested `SkippedToken`
recovery child が現れ得る。`VisibilityMarker` は `private` または `public` token を
1 つだけ包む。`VisibleItem` は Chapter 12 が許す theorem / notation form 上の
top-level visibility prefix を表す。それらの target item grammar がまだ placeholder
である間、child は source order で、annotation-prefix token node があればそれら、
1 個の `VisibilityMarker`、target `PlaceholderItem` である。duplicate visibility
marker、dangling marker、非 theorem/notation top-level declaration の前の visibility では、
`MalformedVisibility` を伴って nested `SkippedToken` recovery child と任意の semicolon
token を含み得る。これらの node は public/private semantics、export availability、
symbol identity、theorem validity、notation validity を判断しない。`SurfaceNodeView` は
`as_export_item`、`as_visibility_marker`、`as_visible_item` の typed helper を公開する。

`mizar-parser` task 8 で追加する type-expression node は syntax-only の形である。
`ReserveItem` は、現在 frontend から到達可能な `TypeExpression` の host である。
これは top-level `reserve_decl` だけを表し、local statement-level `reserve` semantics は
実装しない。`ReserveSegment` は identifier-list の comma、`for` token、後続の
type expression を保持する。`TypeExpression` は、任意の non-empty `AttributeChain` と
必須の `TypeHead` という surface split を保持する。mode / radix / attribute の分類は
active environment に依存するため、`TypeHead` は意図的に generic である。builtin
`object` / `set` または `QualifiedSymbol` と任意の `TypeArguments` を所有するが、それが
mode、structure、radix type のいずれかは記録しない。parser は semantic lookup ではなく、
右端に残る syntactic type-head candidate を `TypeHead` として確保することで
attribute/head boundary を決める。

`AttributeRef` は 1 個の attribute occurrence の source-order syntax を所有する。
任意の `non`、任意の `ParameterPrefix`、1 個の syntactic `QualifiedSymbol`、任意の
parenthesized term argument を含む。struct-qualified attribute spelling は
同じ dotted `QualifiedSymbol` surface として保持する。その attribute-ref context では、
prefix `PathSegment` が namespace identifier だけでなく user-symbol token を包んでもよい。
どの prefix segment が structure であるかは AST では判断しない。`ParameterPrefix` は task 8 が attribute reference の前で
局所的に見える token split だけを保持する。つまり、1 個の identifier または numeral と
`-`、または parenthesized identifier / numeral list と `-` である。template-parameter scope の
妥当性は検証せず、`n-dimensional` のような名前に対する完全な contextual whole-spelling split
も行わない。この source drift は parameter-scope facts を lexing / parsing に渡す将来 task の
責務として残す。

`TypeArguments` は `of` / `over` token と comma-separated term arguments、または
`[`、comma-separated type-template arguments、任意の `]` を所有する。parser task 9 以降、
`of`/`over` と attribute argument list は task 8 の `TermPlaceholder` child ではなく、
concrete `TermExpression` child を使う。type expression として parse できる bracket
argument は nested `TypeExpression` として表す。parser task 11 以降、Appendix A の
`qua_arg` syntax を使う bracket argument は `TermExpression` child として表し、その
term-shape は identifier の `TermReference` または left-nested `QuaExpression` chain である。
この fallback は通常の term parser より狭く、identifier-shaped `qua_arg` からだけ始まり、
各 `qua` target は radix-type 形の `TypeExpression` として parse する。
bracket close が欠ける場合は
`TypeArguments` node の下で
`MalformedTypeExpression` と `UnmatchedOpeningDelimiter` recovery を使う。
`TermPlaceholder` は task 8 の legacy vocabulary としてのみ残り、task 11 の
bracket `qua_arg` parser path では生成しない。term classification、operator fact、
name resolution を encode してはならない。
`SurfaceNodeView` は
`as_reserve_item`、`as_reserve_segment`、`as_type_expression`、`as_attribute_chain`、
`as_attribute_ref`、`as_parameter_prefix`、`as_type_head`、`as_type_arguments`、
`as_term_placeholder` の typed helper を公開する。

`mizar-parser` task 9 で追加される primary term node は syntax-only の形である。
`TermExpression` は現在、1 個の term-shape child を包む wrapper である。parser task 9 と
10 では primary term または postfix chain を置くことができ、parser task 11 では
`QuaExpression` を置くことができる。後続 operator task が operator expression を置いても
wrapper role は変わらない。
`TermReference` は identifier token 1 個または共有 `QualifiedSymbol` 1 個を所有し、
term-position reference を name resolution や functor classification なしで保持する。
`NumeralTerm` と `ItTerm` は対応する token 1 個を包む。`ParenthesizedTerm` は delimiter
token と nested `TermExpression` または `MissingTerm` recovery を所有する。`ChoiceTerm` は
`the` token と nested `TypeExpression`、または type operand 欠落時の
`MissingTypeExpression` recovery を所有する。

`ApplicationTerm` は task 9 では意図的に generic である。ordinary parenthesized application と
reserved `[` / `]` bracket functor form を保持するが、arity、overload selection、active
user-symbol bracket-pair metadata は encode しない。ordinary application の child order は
callee の `TermReference` または `QualifiedSymbol`、`(` token、comma token で区切られた 0 個以上の
`TermExpression` argument、任意の `)` である。reserved bracket application の child order は
`[`、comma token で区切られた 0 個以上の `TermExpression` argument、任意の `]` であり、
delimiter pair 自体が syntax-only head なので callee child を持たない。`StructureConstructor`
は named field argument が syntactically に見える場合だけ出力し、child order は constructor
`QualifiedSymbol`、任意の `TypeArguments`、`(` token、comma token で区切られた
`FieldArgument` children、任意の `)` である。`S()` のような ambiguous zero-field form は、
将来の semantic boundary が structure fact を渡すまで generic `ApplicationTerm` として残す。
`FieldArgument` は field identifier、`:` token、`TermExpression` または `MissingTerm` を
所有する。`SetEnumeration` は `{`、comma token で区切られた source-order term arguments、
任意の `}` を所有する。Fraenkel / set-comprehension form は parser task 15 の責務である。
`SurfaceNodeView` は `as_term_expression`、`as_term_reference`、`as_numeral_term`、
`as_it_term`、`as_parenthesized_term`、`as_choice_term`、`as_application_term`、
`as_structure_constructor`、`as_field_argument`、`as_set_enumeration` の typed helper を公開する。

Parser task 10 は dot-role surface を syntax-only に保つ。`SelectorAccess` は
base term-shape child、`.` token、identifier field token、任意の call delimiter と
comma token で区切られた source-order `TermExpression` argument を所有する。
selector chain は left-associative に nest する。`StructureUpdate` は base term-shape
child、`with` token、`(`、comma token で区切られた `FieldUpdate` children、任意の `)` を
所有する。`FieldUpdate` は identifier selector path（identifier、反復する `.` token +
identifier）、`:=` token、`TermExpression` または `MissingTerm` recovery を所有する。
これらの node は scope を使って selector-versus-namespace role を判断せず、standalone
statement / algorithm assignment も表現しない。`SurfaceNodeView` は
`as_selector_access`、`as_structure_update`、`as_field_update` の typed helper を公開する。

Parser task 11 は `term qua type_expression` のために `QuaExpression` を追加する。
parser は selector/update postfix chain を `QuaExpression` より先に形成するため、
`p.x qua T` は selector result を修飾する。`qua` は現在の term precedence で最も低いため、
修飾済み term の後に selector を置くには `(p qua T).x` のように括弧が必要である。
`qua` chain は left-associative に nest し、`x qua T qua U` は `x qua T` の
`QuaExpression` を base とする `QuaExpression` として表す。`qua` target は
`TypeExpression` である。target type が term argument を含む場合、その argument 内の
`qua` は outer chain が続く前に argument term に属する。そのため
`x qua Element of S qua Magma` は `x qua Element of (S qua Magma)` として表し、
outer result をさらに修飾するには `(x qua Element of S) qua Magma` と書く。
target type 欠落時は `QuaExpression` の下に `MissingTypeExpression` を挿入し、
`MalformedTypeExpression` を出す。malformed target tail は surrounding term boundary の前で
type-tail `SkippedToken` recovery を使ってよい。`SurfaceNodeView` はこの node のために
`as_qua_expression` を公開する。static validity、type narrowing / widening、
overload selection、proof obligation は resolver / checker の責務である。

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
structural root child の descendant でもある場合に deduplicate する。その structural
subtree は、malformed import-tail recovery のように、in-range token child を持つ
recovery node を内部に含んでよい。この場合 token leaf は structural rowan subtree の
下に一度だけ出力され、互換 root の token pass からは省略される。互換 root に直接
列挙される recovery node は root-listed token の deduplication owner ではないため、
後続の builder check または rowan emission rule がその case を文書化するまでは、
parser producer はそのような root-level recovery node に in-range token child を
持たせてはならない。missing-construct recovery には out-of-range context child を
使うか、skipped-token recovery を non-recovery structural owner の下に nest し、
skip された source span を trivia に記録する。

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
