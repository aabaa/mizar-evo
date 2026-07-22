# mizar-parser: Grammar

> 正本は英語です。英語版: [../en/grammar.md](../en/grammar.md)。

状態: module skeleton、top-level placeholder dispatch、concrete import item、
export item、visibility wrapper、reserve-hosted type expression、set comprehension
を含む task 15 term surface、task 14 formula surface、S-013 statement node、
task-22 theorem/proof item、task 23〜30 の definition block / attribute /
predicate / functor / mode / redefinition / notation alias / property /
structure / registration 増分、task 31 template surface、task 32 の
basic algorithm / claim surface、task 33 の algorithm control-flow surface、
task 34 の algorithm verification clause、task 35 の annotation surface は実装済み。
package-oriented item grammar は引き続き計画中。

## 目的

このモジュールは、Mizar Evo のパーサー入口とモジュール／項目文法を定義する。

## 責務

- parser-facing token transfer object を消費し、`mizar-syntax::SurfaceAst` を生成する。
- モジュール、import、定義、registration、文、証明、アルゴリズム、アノテーション、項、論理式を構文解析する。
- 名前解決、型推論、オーバーロード選択、証明義務生成を行わず、構文解析を意味論から分離する。

parser output は後段の `ObligationAnchor` construction に十分な source-shaped syntax を保持しなければならないが、parser は owner origin id、`ObligationAnchor`、`DependencySlice`、proof obligation、overload root selection、type inference、cluster fact を計算してはならない。parser node order、`SourceRange`、`VcId` の一致を proof-reuse authority として扱ってはならない。

現在の挙動:

- crate root の公開 API（`parse`、`ParseRequest`、`ParserToken`、
  `ParseOutput`、および関連する転送 enum / entry）は、従来の
  `mizar_parser::...` path から到達可能なまま保つ。
- `grammar` は現在の parser orchestration と syntax-event sink への受け渡しを
  所有する。Pratt expression parsing と recovery policy は、後続タスクで完全な
  文法を成長させるまで、兄弟の実装 module に置く。
- grammar code は token、通常 node、recovery node を private な syntax-event sink
  と文書化された `mizar-syntax` builder / accessor API を通じて送出し、rowan の
  storage layout や密な arena index に依存しない。
- top-level `reserve` item は、syntax-only `TypeExpression` tree を host できる
  ところまで concrete である。attribute chain、generic type head、`of` / `over`
  `TermExpression` argument、bracket nested type argument、bracket `qua_arg` entry を
  `TermExpression` / `QuaExpression` surface として保持する。その他の non-module item
  grammar は placeholder のままである。

## Task 4: 共有 path

Production inventory:

```ebnf
module_path       ::= [ relative_prefix ] module_identifier
                      { "." module_identifier } ;
relative_prefix   ::= "." | ".." ;
module_identifier ::= identifier ;

namespace_path    ::= identifier { "." identifier } ;

qualified_symbol            ::= { namespace_segment "." } user_symbol ;
qualified_constructor_name  ::= { namespace_segment "." } constructor_name ;
namespace_segment           ::= identifier ;
```

`module_path` は第 12 章の import / export path 形である。共有 path helper の中で
`relative_prefix` を受け入れるのは `module_path` だけであり、citation / reference
prefix 用の `namespace_path` は相対 import prefix を受け入れてはならない。
`qualified_symbol` は、functor / 述語の記法用にアクティブなレキシコンから渡される
パーサー向けの `user_symbol` トークンで終わる。type head と属性参照は
`qualified_constructor_name` を使い、その最後の構成要素は、通常の識別子または
読みやすいハイフン区切りのコンストラクタ名である。

parser task 4 は共有 helper method と unit coverage だけを提供する。これらの path
形は後続の消費側文法タスク、つまり import item（task 6）、型 head（task 8）、
項 / 論理式、citation（task 17）を通じて frontend から到達可能になるため、この
task 単独の corpus position は導入しない。helper は syntax-event sink を通じて
`mizar-syntax` task S-009 の path node を送出し、dot separator を構文として保持する。
module resolution、namespace shadowing、symbol identity 割り当て、citation lookup、
validity checking は行わない。

## Task 5: module skeleton と top-level dispatch

Production inventory:

```ebnf
compilation_unit   ::= import_prelude export_prelude { annotated_declaration } ;
import_prelude     ::= { import_stmt } ;
export_prelude     ::= { export_stmt } ;
declaration        ::= definition_block
                     | reserve_decl
                     | registration_block
                     | claim_block
                     | [ visibility ] theorem_item
                     | [ visibility ] notation_decl ;
visibility         ::= "private" | "public" ;
theorem_status     ::= "open" | "assumed" | "conditional" ;
theorem_role       ::= "theorem" | "lemma" ;
notation_decl      ::= operator_decl | synonym_def | antonym_def ;
```

task 5 は、後続の item parser が concrete node に置き換える安定した surface
skeleton を構築する。parser は `CompilationUnit` node を送出し、その child として
`ItemList` を 1 つ持たせる。`ItemList` には、source order の concrete item node、
まだ concrete でない認識済み top-level start に対応する `PlaceholderItem` node、
skip された top-level input の `SkippedToken` recovery node が入る。認識する start は `import`、`export`、
`definition`、`reserve`、`registration`、`claim`、`theorem`、`lemma`、
theorem-status prefix の `open` / `assumed` / `conditional`、visibility prefix の
`private` / `public`、notation start の `infix_operator`、`prefix_operator`、
`postfix_operator`、`synonym`、`antonym` である。task 6 以降、`import` は import
prelude がまだ開いている場合だけ concrete item になり、それより後の `import`
token は位置が不正な top-level input として回復する。

`@[` または `@identifier` で始まる連続した annotation prefix は task 35 以後
parser-visible である。表現済み declaration と top-level block、statement、
algorithm statement、definition content、registration content、claim-local item の
eligible start は、placeholder-only prefix token ではなく concrete `Annotation` /
`LibraryAnnotation` child を所有する。import/export prelude item は
`annotated_declaration` の外側に残り、annotation prefix を受け付けない。malformed annotation argument、option、delimiter
surface は `MalformedAnnotation` で recover し、可能な限り後続の eligible item を保持する。
セミコロン型 placeholder は nested `proof ... end` と文脈付き
algorithm/proof block をまたいで scan するため、proof body 内のセミコロンで theorem /
lemma item を分割しない。
式レベルの `if` や `otherwise` のような文脈依存 formula keyword は placeholder の
block depth に影響しない。

この task は theorem formula、visibility semantics、item validity、symbol identity を
parse しない。task 7 以降、`export` と visibility prefix は concrete な syntax
wrapper であり、非 module declaration は所有する文法 task が
着地するまで placeholder item のままである。認識可能な top-level item start を
含まない token stream は module skeleton に関して task 3 の互換 behavior を保つ。
つまり token は保持され、item list は空になる。このような stream が diagnostic-free
のままになるのは、legacy minimal token-stream corpus case のように、先行する
recovery pass でも指摘がない場合に限られる。最初の認識済み item keyword が先行する
recovery block opener の内側にある合成 block-recovery stream も、この互換 behavior
を保つ。一方、theorem item の前に裸の reserved word があるような通常の malformed
prefix は `UnexpectedTopLevelToken` recovery を生成する。

## Task 6: import item

Production inventory:

```ebnf
import_stmt          ::= "import" module_alias_decl
                         { "," module_alias_decl } ";" ;
module_alias_decl    ::= module_path [ "as" module_identifier ]
                       | module_branch_import ;
module_branch_import ::= module_path ".{"
                         module_identifier { "," module_identifier } "}" ;
```

parser は import prelude が開いている間、`import_stmt` ごとに `ImportItem` を
1 つ送出する。well-formed import では、item の child は `import` token、comma token
で区切られた 1 個以上の import declaration node、終端 semicolon token である。
単純 import と alias は `ModulePath` child、任意の `as` token、任意の alias
`PathSegment` を持つ `ImportAliasDecl` を送出する。branch import は base
`ModulePath`、`.{` token、comma token で区切られた branch identifier `PathSegment`、
`}` を持つ `ModuleBranchImport` を送出する。

import path は task 4 の共有 `ModulePath`、`RelativePrefix`、`PathSegment` node を
使う。parser は relative prefix と branch component を構文的に保持するが、module
resolution、alias collision 検査、export の検査、symbol identity 割り当て、
visibility の判断は行わない。

非 import の top-level item が parse されると import prelude は閉じる。それ以降の
`import` token は `UnexpectedTopLevelToken`、`SkippedToken` recovery、skipped-range
trivia により、semicolon または次の top-level boundary まで回復される。import の
semicolon 欠落は `MissingSemicolon` を使う。`as` の後に alias がない、または branch
import の `}` がないなど、現在の statement boundary で継続できる import 内部の不正
構文は `MalformedImport` を使う。semicolon の前にある malformed source を消費する
場合、parser は import item または不正 declaration の内部に `SkippedToken` recovery
node と skipped-range trivia を持たせて所有する。そのため recovery shape では、
`import` の後に declaration を持たない `ImportItem`、後続 declaration のない trailing
comma、alias segment のない `ImportAliasDecl`、branch segment または `}` のない
`ModuleBranchImport` が現れ得る。

## Task 7: export と visibility item

Production inventory:

```ebnf
export_stmt ::= "export" module_path { "," module_path } ";" ;
visibility  ::= "private" | "public" ;
```

parser は export prelude が開いている間、`export_stmt` ごとに `ExportItem` を
1 つ送出する。import prelude は引き続き最初に来る。最初の非 import item を見た時点で
import は閉じる。import prelude の直後に連続する `export` statement が export
prelude を構成する。最初の通常 declaration が export prelude を閉じ、それ以降の
`export` token は `UnexpectedTopLevelToken`、`SkippedToken` recovery、skipped-range
trivia により、予期しない top-level input として回復される。それ以降の `import`
token は late-import recovery のままである。

well-formed export では、`ExportItem` の child は `export` token、comma token で
区切られた 1 個以上の `ModulePath` node、終端 semicolon token である。export path は
task 4 の `ModulePath`、`RelativePrefix`、`PathSegment` node を使う。parser は
relative prefix と comma list を構文的に保持するが、module resolution、import された
export の検査、facade summary 構築、visibility 検証は行わない。

現在の statement boundary で継続できる export 内部の不正構文は `MalformedExport` を
使う。例は `export` の後または comma の後の path 欠落である。semicolon の前にある
malformed source は、`ExportItem` 内部の nested `SkippedToken` recovery node と
skipped-range trivia が所有する。export の semicolon 欠落は `MissingSemicolon` を使う。

top-level visibility は、Chapter 12 が許す theorem item と notation declaration に
だけ表現する。parser は `VisibleItem` wrapper を送出する。child は source order で、
parsed `Annotation` prefix node があればそれら、`private` または `public` token を包む
`VisibilityMarker` 1 個、後続 target item node である。
表現済み theorem / lemma target は concrete な `TheoremItem` / `LemmaItem` node を使う。
notation target と短い legacy theorem fragment は `PlaceholderItem` target のままである。
template predicate argument を含む theorem payload は task 31 以後、周辺 theorem shape が
表現可能な場合には concrete theorem target として parse される。合法 target start は
`theorem`、`lemma`、theorem status と theorem role の組（`open`、`assumed`、
`conditional` の後に `theorem` または `lemma`）、および notation start の
`infix_operator`、`prefix_operator`、`postfix_operator`、`synonym`、`antonym` である。
他の top-level declaration への visibility、duplicate visibility marker、dangling
marker は `MalformedVisibility` を使い、statement semicolon より前に malformed tail
token があれば単一の `VisibleItem` 内部で skip する。semicolon だけの dangling
marker では、空の recovery node を作らず semicolon を `VisibleItem` の直接 child として
残す。不正な target が block-like な top-level declaration（`definition`、
`registration`、`claim`）である場合、同じ recovery が matching `end` までを
malformed target として所有する。後続 semicolon があれば wrapper の statement
terminator のままにし、追加の top-level recovery node へ cascade しないようにする。

## Task 8: type expression

Production inventory:

```ebnf
reserve_decl      ::= "reserve" reserve_segment ";" ;
reserve_segment   ::= identifier_list "for" type_expression ;
identifier_list   ::= identifier { "," identifier } ;

type_expression   ::= attribute_chain type_head ;
type_head         ::= radix_type | mode_type ;

attribute_chain   ::= { [ "non" ] attribute_ref } ;
attribute_ref     ::= [ param_prefix ] [ struct_ref_name "." ] attribute_ref_name
                      [ "(" argument_list ")" ] ;
param_prefix      ::= parameter "-" | "(" parameter_list ")" "-" ;

radix_type        ::= builtin_type | struct_ref_name [ type_args ] ;
mode_type         ::= mode_ref_name [ type_args ] ;
type_args         ::= ( "of" | "over" ) argument_list
                    | "[" type_arg_list "]" ;
type_arg_list     ::= type_arg { "," type_arg } ;
type_arg          ::= type_expression | qua_arg ;
qua_arg           ::= identifier { "qua" radix_type } ;
argument_list     ::= term_expression { "," term_expression } ;

builtin_type      ::= "object" | "set" ;
attribute_ref_name ::= qualified_constructor_name ;
mode_ref_name     ::= qualified_constructor_name ;
struct_ref_name   ::= qualified_constructor_name ;
```

task 8 は top-level `reserve` declaration を通じて type expression を実行可能にする。
parser が送出する `ReserveItem` と `ReserveSegment` は、現在の `TypeExpression` host
に限る。local statement-level `reserve` behavior は言語の一部ではなく、Chapter 4 は
block-local `reserve` shaped statement を syntax error として分類する。well-formed
`ReserveItem` は `reserve` token、1 個の `ReserveSegment`、終端 semicolon を所有する。
`ReserveSegment` は source order の identifier / comma token、`for` token、
`TypeExpression` を所有する。

parser は、任意の non-empty `AttributeChain` と必須の generic `TypeHead` を持つ
`TypeExpression` を送出する。syntactic head が radix type、structure、mode のいずれか、
また dotted attribute spelling が structure qualifier を含むか namespace segment だけを
含むかは判断しない。コンストラクタ名形の参照の列を複数の方法で分割できる場合、
parser は右端に残る syntactic type-head candidate を `TypeHead` として確保し、それより
前の reference を attribute として扱う。これは syntax-only の boundary rule であり、
semantic classification ではない。

`AttributeRef` は、任意の `non`、任意の parameter prefix、コンストラクタで修飾された
参照の表層、任意の parenthesized term argument を保持する。`TypeHead` は、builtin
`object` / `set` token、またはコンストラクタで修飾された head と任意の `TypeArguments` を
保持する。既存の syntax-node name は storage shape として `QualifiedSymbol` を使い続けても
よいが、type / attribute の綴りクラスは、任意の述語 / functor の `user_symbol` ではなく
`qualified_constructor_name` である。task 8 は、
incoming token が attribute reference の前で局所的な prefix split をすでに露出している場合に
限って `ParameterPrefix` を保持する。つまり identifier または numeral と `-`、または
parenthesized identifier / numeral list と `-` である。template-parameter scope は検証せず、
`n-dimensional` のような 1 個の user-symbol spelling を split しない。この点は
parameter-scope facts と active attribute suffix を扱える後続 parser / lexer task まで
source drift として分類する。

`TypeArguments` は `of` / `over` argument list を保持する。task 9 以降、これらの list は
一時的な term-entry placeholder ではなく primary term の `TermExpression` child を所有する。
bracket type argument は可能な範囲で nested `TypeExpression` child として再帰的に parse する。
task 11 以降、bracket argument が代わりに Appendix A の `qua_arg` に一致する場合、parser は
identifier の `TermReference` または left-nested `QuaExpression` chain を term-shape に持つ
`TermExpression` child として保持する。この bracket fallback は通常の term parsing より狭く、
identifier-shaped `qua_arg` からだけ始まり、各 `qua` target は radix-type 形の
`TypeExpression` として parse する。`]` 欠落は `MalformedTypeExpression` と
`UnmatchedOpeningDelimiter` recovery として保持する。task 8 の `TermPlaceholder` node は
task 11 以降 legacy vocabulary としてのみ残り、term classification、operator fact、
name resolution、overload selection を encode してはならない。

現在の reserve statement boundary で継続できる malformed type syntax は
`MalformedTypeExpression` を使う。`reserve ... for` の後、または bracket
`type_arg_list` 内で純粋に type expression が欠落した場合は `MissingTypeExpression`
recovery を挿入してよい。`of` / `over` の term argument 欠落は task 9 の term recovery
（`MalformedTermExpression` と `MissingTerm`）であり、missing type expression として
報告してはならない。`;`、`,`、`]`、`)` の前にある malformed tail は、最も近い
reserve/type node が所有する nested `SkippedToken` recovery と skipped-range trivia を使ってよい。

active parse-only corpus は、identifier-shaped mode / attribute / structure symbol を
可視にするため syntax-only の `parser.type_fixtures` module を import する。
`mizar-test` の parse-only provider は、その fixture module に限って task 8 fixture 用の
小さな固定 symbol set を export する。これらの symbol は test harness input に限られ、
resolver semantics や built-in library content を意味しない。task 8 の test は少なくとも、
連続する fixture symbol に対する右端の attribute / type-head split、positive `non`
attribute chain、`of` / `over` argument list、bracket nested `TypeExpression`
argument、bracket `qua_arg` placeholder、token が split を露出する場合の局所
`ParameterPrefix` 保持（parser unit test）、`]` 欠落 diagnostic、`reserve ... for` 後の
malformed type-expression insertion を pin する。

## Task 9: Primary Terms

Production inventory:

```ebnf
term_expression      ::= operator_expression { "qua" type_expression } ;
operator_expression  ::= postfix_expression | functor_application ;
postfix_expression   ::= term_primary { term_postfix } ;

term_primary         ::= variable_identifier
                       | "it"
                       | numeral
                       | "(" term_expression ")"
                       | struct_constructor
                       | set_enumeration
                       | choice_expression
                       | inline_functor_application
                       | bracket_functor_application ;
variable_identifier ::= identifier ;
numeral             ::= digit+ ;

choice_expression   ::= "the" type_expression ;
struct_constructor  ::= struct_ref_name [ type_args ]
                         "(" [ named_arg { "," named_arg } ] ")" ;
named_arg           ::= identifier ":" term_expression ;
set_enumeration     ::= "{" [ term_list ] "}" ;
term_list           ::= term_expression { "," term_expression } ;
inline_functor_application ::= inline_func_name "(" [ term_list ] ")" ;
bracket_functor_application ::= "[" term_list "]" ;
```

Task 9 は syntax-only の primary term node を導入し、grammar が
`argument_list` または `term_list` としている場所で task 8 の type parser に接続する。
そのため `of` / `over` の `TypeArguments` と parenthesized `AttributeRef` argument は、
task 8 の `TermPlaceholder` ではなく `TermExpression` child を所有する。実際に
bracket `type_arg_list` behavior は task 8 から deterministic のまま維持する。
`type_expression` として parse できる argument は nested `TypeExpression` node のままにし、
`qua_arg` に一致する entry は task 11 まで `TermPlaceholder` として残す。task 9 は
bracket type argument を term expression として再解釈しない。
`template_functor_application` は normative な term primary だが、parser task 31 /
mizar-syntax S-016 が所有する template argument surface を必要とする。task 9 ではこれを
deferred `source_drift` として記録し、template functor application は parse しない。

parser は現在の term wrapper として `TermExpression` を出力する。task 9 では
selector/update postfix、`qua`、active operator parsing が後続 task であるため、
`TermExpression` は primary-term child をちょうど 1 つ含む。`TermReference` は term
position の identifier token または共有 `QualifiedSymbol` を包み、それが variable、
inline functor、structure name、その他の semantic entity のどれであるかは判断しない。
`NumeralTerm`、`ItTerm`、`ParenthesizedTerm`、`ChoiceTerm`、`ApplicationTerm`、
`StructureConstructor`、`FieldArgument`、`SetEnumeration` は対応する source delimiter と
source-order child を保持する。

parenthesized application syntax は、argument list に目に見える
`identifier ":" term_expression` field assignment が含まれる場合だけ
`StructureConstructor` として parse し、それ以外は `ApplicationTerm` として保持する。
これは syntax-only split である。`S()` のような zero-field constructor は、将来の semantic
boundary が structure fact を渡すまで generic application のまま残す。reserved `[` と `]`
による built-in bracket functor notation は `ApplicationTerm` として保持する。
`user_symbol term_list user_symbol` の active user-symbol delimiter pair は、task 12 の
prefix/postfix/infix `OperatorFixity` entry を超える bracket-pair metadata を必要とするため、
その metadata が存在するまで deferred のままとする。

Task 9 は term list 内の欠落または malformed primary term に
`MalformedTermExpression` diagnostic を出し、pure insertion point では `MissingTerm`
recovery を挿入してよい。`)` / `}` / `]` delimiter が欠ける場合は、nearest term node の下で
`MalformedTermExpression` と `UnmatchedOpeningDelimiter` recovery を使う。`,`, `;`,
`)`, `]`, `}`、または top-level item boundary で同期できる malformed tail は、
`SkippedToken` recovery と skipped-range trivia を使ってよい。

active parse-only corpus は、statement/formula host が着地するまで reserve-hosted type
argument list と attribute argument list から task 9 term に到達させる。テストは少なくとも
term position の identifier と numeral、parenthesized term、`it`、`the type_expression`
を使う choice term、通常の parenthesized application、named-field structure-constructor
syntax、set enumeration literal、reserved bracket functor application、missing term
argument、missing term delimiter を pin する。

結果: task 9 は実装済みである。`of` / `over` と parenthesized `AttributeRef`
argument list は、primary term の syntax-only `TermExpression` child を所有する。
bracket `type_arg_list` は引き続き nested `TypeExpression` child と `qua_arg`
`TermPlaceholder` child を保持する。parser unit test と active parse-only pass/fail
corpus case は、上記の primary-term 形と recovery 挙動を cover する。

Task 10 は semantic lookup を追加せずに parser/syntax の dot-role surface 形状を
解決する。incoming token kind が qualified symbol をすでに露出している dotted
qualified-name head は `QualifiedSymbol` のまま保持する。すでに parse 済みの term の後に
reserved `.` が来る場合は postfix selector surface とする。parser は scope を使って
spelling が namespace segment なのか selected field なのかを判断しない。その分類は
resolver phase の責務である。

task 10 でも `TermExpression` は current term-shape child を 1 個所有するが、その child は
primary term または nested postfix chain になり得る。`SelectorAccess` は base term-shape
child、`.` token、identifier field token、任意の parenthesized argument list を保持する。
selector chain は left-associative に nest するため、`line.finish.y` は base が
`line.finish` selector の selector である。`StructureUpdate` は base term-shape child、
`with` token、delimiter、source-order `FieldUpdate` children、comma token を保持する。
`FieldUpdate` は identifier selector path、`:=` token、value の `TermExpression` または
`MissingTerm` recovery を所有する。selector path は `field_name` に合わせて identifier
だけを使い、example と test は `end` のような reserved-word field name を避ける。

Task 10 は task 9 の term parser に現在到達できる位置で `p with (x := y)` のような
functional update term を parse する。`p.x := t` のような standalone in-place selector
assignment は statement / algorithm host が後続 parser task の責務であるため parse しない。
leading `with (...)` は `with` が `term_primary` ではなく postfix なので malformed のままにする。
malformed selector / update syntax は `MalformedTermExpression` を使う。update value 欠落は
`MissingTerm` を挿入し、`)` delimiter 欠落は nearest selector/update term node の下で
`UnmatchedOpeningDelimiter` を使う。

Task 10 result: selector/update postfix parsing は module grammar に実装済みである。
unit coverage は selector chain と call、functional update list、update value 欠落、
update delimiter 欠落、selector argument 後の structure-constructor field-list boundary を
pin する。active parse-only pass/fail fixture は frontend seam を cover し、§2.5.3 と
§13.3.2-13.3.3 に trace back する。

## Task 11: `qua` Qualification

Production inventory:

```ebnf
term_expression      ::= operator_expression { "qua" type_expression } ;
qua_expression       ::= operator_expression "qua" type_expression
                         { "qua" type_expression } ;
```

Task 11 は、現在実装されている term-level operator の中で最も低い precedence として
`qua` を parse する。parser はまず primary term と task 10 の selector/update postfix
chain を形成し、その後 `qua` suffix を left-nested `QuaExpression` node に畳み込む。
そのため `p.x qua T` は selector result を修飾する。qualified term の後の selector は、
selector/update postfix が `qua` より強く bind するため括弧が必要である。
`(p qua T).x` は parenthesized qualified term への selector として parse される。
一方、`p qua T.x` の dot は type parser がその dotted type surface を形成できる場合だけ
target type の中に残る。

通常の term-level `qua` の target は `TypeExpression` である。その type が `of` / `over`
term argument を含む場合、outer `qua` chain が続く前に、それらの argument は full term parser
で parse される。したがって `x qua Element of S qua Magma` は
`x qua Element of (S qua Magma)` として表し、outer result を再度修飾するには
`(x qua Element of S) qua Magma` と書く必要がある。

Appendix A の `qua_arg` に一致する bracket `type_arg_list` entry は、task 8 の
`TermPlaceholder` node としては保存しない。identifier の `TermReference` base と任意の
left-nested `QuaExpression` suffix を持つ `TermExpression` child として parse する。
この fallback は通常の term parsing より意図的に狭く、`qua_arg ::= identifier { "qua"
radix_type }` に合わせて identifier から始まり、各 target は radix-type syntax を使う。

通常の `qua` target type 欠落時は `MalformedTypeExpression` を出し、`QuaExpression` の下に
`MissingTypeExpression` recovery child を挿入する。malformed target tail は周囲の term parser
が続く前に type-expression recovery boundary で同期する。bracket `qua_arg` recovery は
`]` 欠落に対して引き続き `TypeArguments` bracket diagnostics を使う。bracket `qua` 後の
type 欠落は、同じく `QuaExpression` の下の `MissingTypeExpression` child を使う。

Task 11 の test は少なくとも、left-associative `qua` chain、selector / application precedence
（`p.x qua T`、`f(a) qua T`、`(p qua T).x`）、`Element of S qua Magma` の target-type argument
binding、bracket `V qua R` surface が `TermPlaceholder` から移行したこと、target 欠落
および malformed target diagnostic、Chapter 13 へ trace する active parse-only pass/fail
coverage を pin する。

Task 11 result: `qua` qualification parsing は module grammar に実装済みである。
unit coverage は left-associative chain、selector / application precedence、
parenthesized selector-after-`qua`、target-type argument binding、bracket `qua_arg` が
`TermPlaceholder` から移行したこと、target 欠落 recovery、malformed target-tail recovery を
pin する。active parse-only pass/fail fixture は frontend seam を cover し、§13.6 に
trace back する。

## Task 12: Operator Expressions

Production inventory:

```ebnf
operator_expression ::= prefix_expression
                      | postfix_expression
                      | infix_expression
                      | selector_or_primary_term ;
prefix_expression   ::= prefix_operator operator_expression ;
postfix_expression  ::= operator_expression postfix_operator ;
infix_expression    ::= operator_expression infix_operator operator_expression ;
term_expression     ::= operator_expression { "qua" type_expression } ;
```

実 parser は、上の模式的な production を直接再帰するのではなく、
[pratt.md](./pratt.md) の Pratt 契約を使う。Operator metadata は frontend が
`ParserInputs` から導出した `ParseRequest::operator_fixity` としてこの crate に届く。
各 entry は source spelling、fixity kind、precedence、source-coordinate activation offset、
infix の場合の associativity を記録する。parser は各 operator token span でこの metadata を
filter し、その後 source token を `PrefixExpression`、`PostfixExpression`,
`InfixExpression` syntax node に group 化する。overload 解決、result type validation、
table に存在しない visible symbol の default fixity 創作は行わない。Chapter 10 /
Appendix B の default precedence / associativity は、frontend が parser `ParseRequest` を
組み立てる前に lexical-summary producer が materialize していることを期待する。

Task 12 は、legacy token-only Pratt entry point ではなく module term parser を拡張する。
各 Pratt operand は既存の primary term と固定 selector/update postfix chain であるため、
selector、selector call、ordinary application、structure update、parenthesized term は
user operator より強く bind する。`qua` は Pratt の外側に残り、固定の最も低い
term-level operator である。たとえば `p.x ++ y qua T` は selector を left operand 内で
group 化した後、infix expression 全体を修飾する。`(p qua T).x ++ y` のような
`qua` 後の selector には括弧が必要である。
left operand の後で同じ source spelling が postfix と infix の両方として visible な場合、
その infix entry が eligible で、かつ後続 token が right operand を開始できるなら parser は
infix form を選ぶ。そうでなければ、現在の binding power で eligible な postfix entry を選ぶ。

Postfix operator は `[base, operator_token]` の 2 child node を使う。Prefix operator は
`[operator_token, operand]` を使う。Infix operator は既存の 3 child order
`[left, operator_token, right]` を保ち、spelling、precedence、associativity payload を
保持する。同じ infix operator の non-associative chain は 2 個目の operator range に
`NonAssociativeOperatorChain` を出す。infix の right operand 欠落時は dangling operator
range に `DanglingOperator` を出し、partial left expression は表現したままにする。
prefix operand 欠落時は prefix operator range に `DanglingOperator` を出し、
`MissingTerm` operand を挿入して recoverable な `PrefixExpression` を保持する。

Task 12 の test は、parse-only fixture summary 由来の active-lexicon fixity derivation、
prefix/postfix/infix surface node、left/right/non-associative grouping、dangling operator
diagnostic、selector/update、application、parentheses、`qua` との相互作用、および
`spec.en.13.operator_precedence.parser` により Chapter 13 / Appendix B に trace する
active parse-only pass/fail corpus coverage を pin する。

## Task 13: Atomic Formulas

Production inventory:

```ebnf
formula              ::= atomic_formula ;
atomic_formula       ::= predicate_application
                       | inline_predicate_application
                       | is_assertion ;

predicate_application        ::= user_predicate_application
                               | builtin_predicate_application ;
user_predicate_application   ::= predicate_segment { predicate_chain_segment } ;
predicate_segment            ::= [ term_list ] [ negation ] predicate_head
                                  [ term_list ] ;
predicate_chain_segment      ::= [ negation ] predicate_head term_list ;
predicate_head               ::= predicate_symbol [ template_args ]
                               | identifier template_args ;
builtin_predicate_application ::= term_expression builtin_pred term_expression ;
inline_predicate_application ::= inline_pred_name "(" [ term_list ] ")" ;
is_assertion                 ::= term_expression "is" [ "not" ]
                                  is_assertion_body ;
is_assertion_body            ::= type_expression | attribute_test_chain ;
attribute_test_chain         ::= [ "non" ] attribute_ref
                                  { [ "non" ] attribute_ref } ;
negation                     ::= "does" "not" | "do" "not" ;
builtin_pred                 ::= "in" | "=" | "<>" ;
```

Task 13 は atomic-formula boundary だけを実装する。formula connective、quantifier、
parenthesized formula、`thesis`、`contradiction` は task 14 の責務である。Task 13 は当初
`label: formula;` coverage のため theorem/lemma placeholder host を使っていたが、task 22
以後、表現済み formula payload を持つ theorem/lemma item は concrete `TheoremItem` /
`LemmaItem` node である。template predicate argument を含む formula payload は、
task 31 / S-016 まで legacy token-preserving `PlaceholderItem` path に残していたが、
task 31 以後は周辺 theorem shape が concrete なら表現する。

`FormulaExpression` は atomic formula child を 1 つ包む。built-in predicate application は
left `TermExpression`、builtin predicate token、right `TermExpression` を保持する。right
operand 欠落は formula 専用 diagnostic ではなく term recovery を使う。`IsAssertion` は
subject `TermExpression`、`is` token、任意の formula-level `not`、generic body child を
保持する。body は `TypeExpression` または `AttributeTestChain` であり、parser は assertion
が type assertion か attribute assertion かを判断しない。`AttributeTestChain` は task-8
`AttributeRef` surface を再利用し、trailing type head を持たない `non empty` のような
attribute-only body を表現できる。task 13 の active fixture では、`empty` のような bare
lowercase attribute-like body も、trailing type head によって `TypeExpression` を形成できない
場合は `AttributeTestChain` として保持する。`T` のような uppercase body や type argument を
持つ body は `TypeExpression` surface のままである。これは syntactic preservation rule であり、
resolver classification ではない。

user predicate application は syntax-only である。`PredicateApplication` は 1 個以上の
`PredicateSegment` child を所有する。segment は任意の left `TermExpression` list child、
任意の `does not` / `do not` negation token、1 個の `PredicateHead`、任意の right
term-list child を保持する。predicate-chain adjacency と overload validity は resolver の
責務であり、parser は `a < b < c` のような chain が解決可能であることを証明せず、書かれた
segment を保持する。built-in predicate は predicate-chain head ではない。`in`、`=`、`<>`
は単独の `BuiltinPredicateApplication` atom だけを作るため、`a < b = c` のような mixed
chain は user predicate chain として表現せず syntax error のままとする。task 31 以後、
`PredicateHead` は predicate symbol / qualified symbol または template-local identifier と、
任意の `template_args` を所有できる。

theorem/lemma formula host は task-13 shape では exact であり、表現済み
`label: formula;` payload は後続の concrete theorem item 配下に `FormulaExpression` を送出する。
predicate-chain segment の right term 欠落は `MalformedTermExpression` を報告し、
`MissingTerm` を挿入する。

Task 13 の test は built-in `in`、`=`、`<>` atom、attribute-only `non` chain を含む generic
`is` assertion、inline predicate call shape、active-lexicon user predicate segment、
theorem formula host、semantic classification を必要としない malformed atomic
formula recovery を pin する。

## Task 14: Connectives And Quantifiers

Production inventory:

```ebnf
formula              ::= quantified_formula | iff_formula ;
iff_formula          ::= implies_formula
                         [ "iff" ( implies_formula | quantified_formula ) ] ;
implies_formula      ::= or_formula
                         [ "implies" ( implies_formula | quantified_formula ) ] ;
or_formula           ::= and_formula
                         { "or" ( and_formula | quantified_formula )
                         | "or" "..." "or"
                           ( and_formula | quantified_formula ) } ;
and_formula          ::= not_formula
                         { "&" ( not_formula | quantified_formula )
                         | "&" "..." "&"
                           ( not_formula | quantified_formula ) } ;
not_formula          ::= "not" ( not_formula | quantified_formula )
                       | atomic_formula
                       | "(" formula ")"
                       | "contradiction"
                       | "thesis" ;

quantified_formula   ::= universal_formula | existential_formula ;
universal_formula    ::= "for" quantified_vars [ "st" formula ]
                         ( "holds" formula | quantified_formula ) ;
existential_formula  ::= "ex" quantified_vars "st" formula ;

quantified_vars      ::= explicit_vars [ "," implicit_vars ] | implicit_vars ;
explicit_vars        ::= qualified_segment { "," qualified_segment } ;
qualified_segment    ::= var_list ( "being" | "be" ) type_expression ;
implicit_vars        ::= var_list ;
var_list             ::= identifier { "," identifier } ;
```

Task 14 は task-13 の atomic-only formula parser を fixed formula-precedence
parser に置き換え、S-012 formula surface を完了する。formula precedence は
term operator fixity と分離する。`not` は `&` より強く、次に `or`、右結合の
`implies`、非結合の `iff` が続き、quantifier は outermost formula form として
bind する。parentheses なしの `iff` chaining は `NonAssociativeOperatorChain`
を報告する。atomic formula operand 内の user-defined functor precedence は
引き続き term Pratt parser が所有する。

`FormulaExpression` は引き続き formula child を 1 つだけ包む。Task 14 は
`not` 用の `PrefixFormula`、`&` / `or` / `implies` / `iff` 用の
`BinaryFormula`、`ParenthesizedFormula`、`QuantifiedFormula`、
`QuantifierVariableSegment`、および `thesis` / `contradiction` 用の
`FormulaConstant` を追加する。binary formula node は connective token を保持し、
`& ... &` と `or ... or` は 2 つの connective token と `...` token を同じ
binary node 上に保持する。repetition form の展開と alpha-equivalence check は
semantic / checker work に残す。

`QuantifiedFormula` は quantifier token（`for` または `ex`）、comma token で
区切られた 1 個以上の `QuantifierVariableSegment` child、universal
quantification 用の optional `st` condition formula、existential
quantification 用の required `st` body formula、および universal quantification
用の `holds` body formula または nested quantified-formula body を保持する。
`QuantifierVariableSegment` は書かれた variable token list、optional `be` /
`being` token、optional `TypeExpression` を保持し、`reserve` 由来の implicit
variable type は解決しない。

theorem/lemma formula host は atomic formula から task-14 formula 全体へ広がる。
task 22 以後、`by` や `proof` の theorem justification / proof tail が続く prefix も
concrete theorem item であり、template predicate argument は task 31 / S-016 まで
deferred だった。task 31 以後はそれも表現する。formula を term syntax 内へ埋め込む
Fraenkel / set-builder term は task 15 で実装済みである。

`not`、connective、quantifier `st`、`holds` の後で formula operand が malformed
な場合は `MissingFormula` recovery を挿入し、`MalformedFormulaExpression` を
報告する。quantifier header は、少なくとも 1 個の variable segment を表現できる場合に
保持する。`be` または `being` 後の explicit type 欠落は `MissingTypeExpression`
recovery と `MalformedTypeExpression` を再利用する。malformed quantifier-header
separator や tail は `MalformedFormulaExpression` を報告する。quantified variable
list が完全に欠落している場合、task 14 は missing variable segment を合成しない。その入力は
later binder-recovery task が専用 missing-binder vocabulary を追加するまで concrete formula
host の外に残す。

Task 14 の test は connective precedence と associativity、parenthesized formula
grouping、非結合 `iff` rejection、repetition token preservation、`thesis` /
`contradiction`、explicit / implicit variable を持つ universal / existential
quantifier、`holds` を繰り返さない nested universal quantification、
theorem formula hosting、および missing-formula recovery を固定しなければならない。

## Task 15: Fraenkel / set-builder term

Production inventory:

```ebnf
set_expression       ::= set_enumeration | set_comprehension ;
set_enumeration      ::= "{" [ term_list ] "}" ;
set_comprehension    ::= "{" term_expression "where" typed_var_list
                          [ ":" formula ] "}" ;
typed_var_list       ::= typed_var { "," typed_var } ;
typed_var            ::= identifier "is" type_expression ;
```

Task 15 は Chapter 13 の set-comprehension primary term を追加して、S-011
term surface を完了する。`SetEnumeration` は `{}` と `{ term_list }` を表す
task 9 surface のまま残す。`SetComprehension` は、brace 内の先頭が 1 個の
mapper `TermExpression` として parse され、その後に brace が閉じる前の
top-level `where` が続く場合にだけ選択する。mapper term は既に実装済みの
task-12 term surface なので、その内部の selector/update、`qua`、active operator
grouping は保持される。nested comprehension は通常の nested `SetComprehension`
term child として表す。

`SetComprehension` の child order は source order で、`{`、mapper
`TermExpression`、`where`、comma token で区切られた 1 個以上の
`ComprehensionVariableSegment`、任意の `:` と `FormulaExpression`、最後に
`}` または delimiter recovery である。`ComprehensionVariableSegment` は
generator identifier、または identifier 位置の `MissingTerm` recovery、存在する場合の
`is` token、そして `is` token が存在する場合の `TypeExpression` または
`MissingTypeExpression` recovery を所有する。parser は binder identity、implicit
domain、sethood、capture、mapper result type、elaborated Fraenkel symbol を解決しない。

`:` 後の任意 condition は task-14 formula parser を使う。その formula 内の
template predicate argument は task 31 / S-016 まで deferred だったが、task 31 以後は
task-15 comprehension shape を変えずに表現する。
condition omission は `:` と `FormulaExpression` の両方がないこととして表し、synthetic
`thesis` や暗黙 true formula は作らない。

mapper term 欠落、generator identifier 欠落、generator `is` 欠落、malformed
generator separator は `MalformedTermExpression` を使う。純粋な mapper insertion
point では `MissingTerm` を使い、generator segment は将来 binder-specific recovery
vocabulary が追加されるまで identifier 位置に `MissingTerm` を所有してよい。`is` 後の
generator type 欠落は `MalformedTypeExpression` と `MissingTypeExpression` を再利用する。`:` 後の
condition formula 欠落は `MalformedFormulaExpression` を報告し、`MissingFormula`
を挿入する。`}` 欠落は `SetComprehension` node 下の `UnmatchedOpeningDelimiter`
と `MalformedTermExpression` を使う。

Task 15 の test は condition omission を持つ comprehension、conditioned
comprehension、comma preservation を含む multiple generator、mapper term precedence と
nested comprehension structure、active parse-only pass/fail fixture、missing generator
type recovery、missing condition formula recovery、generator `is` 欠落 recovery、
closing brace 欠落 recovery、set enumeration と set comprehension の区別を固定しなければならない。

Task 15 の結果: `SetComprehension` と `ComprehensionVariableSegment` は primary
term surface として実装済みである。parser は最初の top-level separator より前に
top-level `where` が現れる場合だけ comprehension syntax を選択し、enumeration
syntax は既存の `SetEnumeration` path に残す。任意 condition には task 14 の
formula parser を再利用し、仕様化された missing type、missing formula、
missing term、unmatched delimiter recovery node を送出する。active parse-only
pass/fail fixture と `spec.en.13.set_expressions.parser` traceability がこの増分を
覆う。lexer scope skeleton も expression-level の `is set` type word を malformed
`set name =` binder statement ではなく type syntax として扱うため、set-comprehension
fixture は active parse-only corpus で実行できる。

## Task 16: simple statement

Task 16 は、Chapter 15 のうち、この増分では justification clause を持たない statement
形式から S-013 statement syntax を開始する。

```ebnf
statement_item      ::= simple_statement ;
simple_statement   ::= let_statement
                     | assume_statement
                     | take_statement
                     | set_statement
                     | given_statement ;
let_statement      ::= "let" qualified_variable_segment
                       { "," qualified_variable_segment }
                       [ "such" condition_list ] ";" ;
assume_statement   ::= "assume" ( proposition | condition_list ) ";" ;
take_statement     ::= "take" witness { "," witness } ";" ;
set_statement      ::= "set" equating { "," equating } ";" ;
given_statement    ::= "given" qualified_variable_segment
                       { "," qualified_variable_segment }
                       [ "such" condition_list ] ";" ;
condition_list     ::= "that" proposition { "and" proposition } ;
proposition        ::= [ identifier ":" ] formula_expression ;
witness            ::= term_expression | identifier "=" term_expression ;
equating           ::= identifier "=" term_expression ;
```

`StatementItem` は、active parse-only fixture が module-level statement fragment を検査できるようにする
parser-owned の一時 item host である。task 22 の proof block は、同じ concrete statement node を
`StatementItem` wrapper なしで直接所有する。task 35 は attachable annotation prefix
がある ordinary statement position に `AnnotatedStatement` wrapper を追加する。
canonical Chapter 4 specification は
`reserve` を top-level module declaration のみに分類しているため、task 16 は
`reserve` coverage を既存 task-8 `ReserveItem` path の non-regression として扱い、
block-local `ReserveStatement` node を追加しない。

`QualifiedVariableSegment` は、書かれた identifier list、任意の `be` / `being` token、
任意の `TypeExpression` または `MissingTypeExpression` recovery を保持する。module-level
`reserve` からの implicit type は解決しない。`ConditionList` は statement-level の
`that` / `and` separator を保持し、`and` は formula conjunction ではない。`Proposition`
は任意の label token と colon、および 1 個の `FormulaExpression` または `MissingFormula`
recovery を所有する。`Witness` は通常の term witness または `identifier "=" term` の
named witness 形状を保持する。`Equating` は `set` abbreviation 用の
`identifier "=" term_expression` を保持する。

Task 16 は当初 justification surface を deferred とした。semicolon 前に top-level
`by` tail を持つ `let` statement は、部分的な concrete `LetStatement` ではなく
legacy placeholder のまま残していた。task 17 はこの境界を concrete な
justification-aware 形状に置き換えた。この task-16 parser は label、reference、
witness leakage、type well-formedness、proof obligation も検証しない。

statement recovery は既存の syntax-level diagnostic を再利用する。qualified type の欠落は
`MalformedTypeExpression` と `MissingTypeExpression` を使う。proposition
formula の欠落は `MalformedFormulaExpression` と `MissingFormula` を使う。`take`
witness、`set` equating identifier、`set` 右辺の欠落は、binder-specific recovery kind が
入るまで `MalformedTermExpression` と `MissingTerm` を使う。malformed statement tail は
semicolon、EOF、または次の statement/item boundary で同期し、token を skip する必要が
ある場合は skipped source を `SkippedToken` recovery 配下に保持する。

Task 16 の test は、concrete な `let`、`assume`、`assume that`、`given`、`take`、
`set` statement、direct statement head 用の `StatementItem` wrapping、statement-level
`and` condition splitting、named / unnamed take witness、複数 `set` equating、
既存 `ReserveItem` path による top-level `reserve` non-regression、`let ... by ...` の deferral、missing type /
formula / term / equals / semicolon boundary の recovery を固定する必要がある。
active parse-only corpus coverage は non-`reserve` simple statement には top-level
statement host を使い、top-level `reserve` coverage は既存の `ReserveItem` path に残す。

## Task 17: justification と citation

Task 17 は、既に concrete になっている statement host が消費できる justification clause
から S-014 の proof-support syntax を開始する。canonical Chapter 15 / Chapter 16 は
simple justification を `by references` として定義し、Chapter 20 は computation proof
形を追加する。parser TODO の古い `from` 記述は、この増分では derived documentation
drift と扱い、Chapter 15 / 16 の justification production が許していないため実装しない。

```ebnf
justification_clause     ::= simple_justification | computation_proof ;
simple_justification     ::= "by" references ;
references               ::= reference { "," reference } ;
reference                ::= identifier
                           | qualified_reference
                           | grouped_reference
                           | bulk_reference ;
qualified_reference      ::= namespace_path "." identifier ;
grouped_reference        ::= namespace_path ".{" grouped_reference_item
                              { "," grouped_reference_item } "}" ;
grouped_reference_item   ::= identifier ;
bulk_reference           ::= namespace_path ".*" ;
computation_proof        ::= "by" "computation"
                              [ "(" computation_option
                                  { "," computation_option } ")" ] ;
computation_option       ::= ( "steps" | "timeout" | "nest" )
                              ":" numeral ;
let_statement            ::= "let" qualified_variable_segment
                              { "," qualified_variable_segment }
                              [ "such" condition_list ]
                              [ simple_justification ] ";" ;
compact_statement        ::= proposition justification_clause ";" ;
```

reference と grouped item の template argument は task 31 / S-016 まで deferred
だった。task 31 より前は、次の citation separator までの前に `[` が続く reference は、
部分的な template invocation ではなく recoverable malformed justification syntax として扱っていた。
full `proof ... end` block、theorem / lemma item node、proof-body nesting は task 22
で実装する。

parser-facing な `Numeral` token は、computation option における canonical Chapter 20 の
`nat_literal` の token-level representation として使う。token category を超える数値の
well-formedness は、この syntax 増分の外に残す。
option name は spelling で照合する。現在の lexer table では `steps` と
`timeout` は identifier token として届き得る一方、`nest` は reserved-word token
として届く。

`JustificationClause` は先頭の `by` token と、通常 citation 用の `ReferenceList`
または `by computation(...)` 用の `ComputationJustification` を所有する。
`ReferenceList` は comma token で区切られた source-order の `Reference`、
`QualifiedReference`、`GroupedReference`、`BulkReference` child を所有する。
`Reference` は local identifier token を所有する。`QualifiedReference` は
`NamespacePath`、最後の dot token、最後の identifier token を所有し、既存の namespace
path helper は semantic-free のままである。`GroupedReference` は `NamespacePath`、
compound `.{` token、comma token で区切られた 1 個以上の `GroupedReferenceItem`、
存在する場合の `}` を所有する。`BulkReference` は `NamespacePath` と compound `.*`
token を所有する。`ComputationJustification` は `computation` token と任意の
parenthesized `ComputationOption` list を所有し、各 option は option keyword、colon
token、numeral token を所有する。

Task 17 は、この増分で十分小さく扱える canonical host でだけ justification を消費する。
`let ... by refs;` は task-16 placeholder behavior から `LetStatement` へ更新される。
この host は Chapter 15 が generalization tail を `[ "by" references ]` と定義しているため、
`simple_justification` だけを受け入れる。Task 17 は、共有 justification node を
exercise するために、明示的 justification を持つ最小の `CompactStatement` host も追加する。
これにより `proposition by refs;` と `proposition by computation(...);` を扱える。
明示的 `by` tail を持たない compact statement、compact equality と zero-step iterative
equality の dispatch、conclusion、`consider`、`reconsider` は後続 statement task に残す。
canonical Chapter 15 production がそのような tail を定義していないため、`assume`、
`given`、`take`、`set` はこの task で justification tail を得ない。また reference
resolution、computation option validation、ATP engine selection、computation proof replay
も行わない。

Malformed justification syntax は `SyntaxDiagnosticCode::MalformedJustification` を使う。
欠落した reference、grouped item、computation option operand は、該当する
justification node 配下の `MissingProofStep` recovery node として表す。justification
内部の予期しない top-level token は comma、`}`、`)`、semicolon、次の statement / item
boundary、または EOF へ回復し、skipped source は `SkippedToken` recovery と
skipped-range trivia に保持する。

Task 17 の test は、simple local reference、qualified reference、grouped citation、
bulk citation、comma-separated mixed reference list、明示的 compact statement 上の
option あり / なしの `by computation`、`let ... by ...` の placeholder から concrete
`LetStatement` への更新、non-canonical な `assume` / `given` / `take` / `set`
justification tail の reject または recovery、leading / trailing comma の malformed case、
grouped `}` 欠落、computation-option value 欠落、template argument deferral recovery、
derived documentation drift である `from` tail が task-17 justification node の外に残ること、
および Chapter 15 §15.2.1 / §15.8、Chapter 16 §16.5、Chapter 20 §20.9.2 への traceability
を持つ active parse-only pass/fail corpus coverage を固定する必要がある。

## Task 18: `consider` と `reconsider`

Task 18 は、当時 mandatory な simple justification を持つものとして実装された
Chapter 15 の linkable statement form により、S-013 statement syntax を継続した。
この task は task-17 の `JustificationClause` と `ReferenceList` surface を使うが、
simple citation 形だけに限定する。`by computation` は、より多くの statement kind に
仕様が明示的に許可するまで、task-17 の明示的 `CompactStatement` host だけで受け入れる。

task 18は当初、明示的な`by references` tailを必須とし、欠落したtailをmalformed
justification syntaxとしてrecoverした。checker task 44はその後、Chapter 4、8、15、
Appendix Aのcanonical `reconsider`契約を変更した。justification省略`reconsider`は
構文上受理可能だがsemantic `type.narrowing_requires_proof` gateによって制限され、
proof-block `reconsider`も`reconsider_tail`で明示された。parser task 47はこのhistorical
`reconsider` tail ruleだけをsupersedeし、従来の`source_drift` /
`test_expectation_drift`をcloseした。`consider`はmandatory-simpleのままである。

```ebnf
statement_item       ::= ... | consider_statement | reconsider_statement ;
consider_statement  ::= "consider" qualified_vars
                         "such" condition_list simple_justification ";" ;
reconsider_statement::= "reconsider" reconsider_item
                         { "," reconsider_item }
                         "as" type_expression reconsider_tail ;
reconsider_item     ::= identifier [ "=" term_expression ] ;
reconsider_tail     ::= [ simple_justification ] ";" | proof ";" ;
simple_justification::= "by" references ;
```

`ConsiderStatement` は `consider` token、source-order の `QualifiedVariableSegment`
child と comma token、`such` token、`ConditionList`、simple `JustificationClause`、
存在する場合の semicolon を所有する。task-16 の `qualified_vars` behavior を再利用するため、
1 個の `QualifiedVariableSegment` は `x, y being Real` のような shared-type identifier list
を保持でき、複数の typed segment も source-order のまま残る。task-16 の condition-list
behavior も再利用し、child の `ConditionList` が `that` token と statement-level `and`
separator を所有し、proposition は label を持てる。

`ReconsiderStatement`は`reconsider` token、source-orderの`ReconsiderItem` childとcomma
token、`as` token、1個の`TypeExpression`、optionalなsimple `JustificationClause`または
1個の`ProofBlock`、final semicolonを所有する。`ProofBlock`は`proof`、reasoning body、
`end`を所有し、省略形はjustification childもparser diagnosticも生成しない。
`ReconsiderItem`は既存名を表すidentifier tokenだけ、または新しいnarrowed nameを導入する
identifier、`=`、`TermExpression`を所有する。parserはidentifierが既に束縛済みか、全
reconsidered termがtarget typeを持つか、proof obligationを生成するかを検証しない。

Task 18 の recovery は既存の syntax diagnostic を再利用する。欠落または malformed な
qualified variable と target type は、挿入が必要な場合に `MalformedTypeExpression` と
`MissingTypeExpression` を使う。`consider` condition の欠落は
`MalformedFormulaExpression` と `MissingFormula`、または malformed condition-list
recovery を使う。欠落または malformed な `ReconsiderItem` identifier / 右辺 term は
`MalformedTermExpression` と `MissingTerm` を使う。`consider`のmandatory
`by references` tail欠落と`reconsider`のmalformed explicit `by` tailは
`MalformedJustification`と`MissingProofStep`、またはtask-17のmalformed reference-list
recoveryを使う。omitted `reconsider` tailはvalid syntaxであり、proof-block formは通常の
`MissingEnd` recoveryを再利用する。malformed statement tailは
semicolon、EOF、または次の statement / item boundary で同期し、token を skip する必要が
ある場合は skipped source を `SkippedToken` recovery 配下に保持する。

Task 18 の test は、canonical shared-type 形 `consider x, y being T ... by ...`、
複数の qualified-variable segment を持つ concrete `consider`、label 付き statement-level
`such that` / `and` condition、bare item と equated item を持つ concrete `reconsider`、
共有 target-type ownership、simple citation justification reuse、`qualified_vars` /
`such` / condition / `as` / target type / justification / reconsider item 部品の欠落に対する
reject / recovery、これらの statement host における `by computation` の
recovery、top-level `StatementItem` wrapping、active parse-only pass/fail corpus
coverage、Chapter 15 §15.3.4、§15.5.1、§15.8 への traceability を固定する必要がある。

## Task 19: conclusion、`then`、iterative equality

Task 19 は conclusion statement、sequential `then` modifier、iterative equality
chain により S-013 statement syntax を継続する。この task は、明示的な compact
equality statement と iterative equality chain の parser-owned 境界について
grammar-audit G-AUD-010 も解決する。

```ebnf
statement_item              ::= ... | then_statement | conclusion_statement
                              | iterative_equality_statement ;
then_statement              ::= "then" linkable_statement ;
linkable_statement          ::= compact_statement
                              | conclusion_statement
                              | consider_statement
                              | reconsider_statement
                              | iterative_equality_statement
                              | case_reasoning ;
conclusion_statement        ::= ( "thus" | "hence" ) proposition
                                [ justification_clause ] ";" ;
iterative_equality_statement::= [ label_identifier ":" ]
                                term_expression "=" term_expression
                                [ simple_justification ]
                                iterative_equality_step
                                { iterative_equality_step } ";" ;
iterative_equality_step     ::= ".=" term_expression
                                [ simple_justification ] ;
simple_justification        ::= "by" references ;
```

`ThenStatement` は syntax-only wrapper であり、`then` token と 1 個の
linkable statement child を所有する。parser は `then` を desugar せず、直前の
statement を semantic に接続せず、`hence` を `then thus` に書き換えない。
case reasoning は spec-valid な linkable syntax だが、concrete statement node は
parser task 20 が所有するため、task 19 は `then per cases` を不正な `then`
modifier として reject せず、deferred statement-placeholder path に残す。
`let` のような実装済み standalone statement の前に現れる `then` は、
`ThenStatement` 配下の `MissingStatement` recovery で reject する。後続の
standalone statement は次の statement boundary として残す。

`ConclusionStatement` は `thus` または `hence`、1 個の `Proposition`、任意の明示的
`JustificationClause`、任意の recovery、存在する場合の semicolon を所有する。
Chapter 15 は simple justification を optional として定義しているため、明示的な
`by` tail を持たない conclusion も syntax として受け入れる。明示的な `by` tail が
ある場合、conclusion は task-17 justification surface を使う。`conclusion` は
`simple_justification` ではなく一般の `justification` production を使うため、ここでは
computation justification を受け入れる。full `proof ... end` justification block は
task 22 で実装する。

`IterativeEqualityStatement` は任意の label と colon、最初の left term、`=`、最初の
right term、任意の simple citation `JustificationClause`、1 個以上の
`IterativeEqualityStep` child、任意の recovery、存在する場合の semicolon を所有する。
各 `IterativeEqualityStep` は `.=` token、1 個の term expression、任意の simple
citation `JustificationClause` を所有する。Chapter 15 production は各 step に
`simple_justification` を使うため、iterative equality では computation justification を
受け入れない。

G-AUD-010 の dispatch は次のように解決する。parser は、最初の equality の後に
top-level `.=` continuation が続く場合だけ `IterativeEqualityStatement` を構築する。
`x = y by A;` のような `.=` continuation を持たない justified equality は
`CompactStatement` のままにする。同じ規則は label と `then` variant にも適用する。
つまり `A1: x = y by A;` は compact、`A1: x = y by A .= z by B;` は iterative である。

Task 19 recovery は既存 diagnostic を再利用する。conclusion proposition 欠落や
不正な `then` linkable statement は `MalformedFormulaExpression` と `MissingFormula`
または `MissingStatement` recovery を使う。equality term や `.=` step term の欠落 /
malformed syntax は `MalformedTermExpression` と `MissingTerm` を使う。明示的な
citation tail の malformed syntax は `MalformedJustification` と task-17 justification
recovery を使う。malformed statement tail は semicolon、EOF、または次の
statement/item boundary で同期し、token を skip する必要がある場合は `SkippedToken`
recovery として source を保持する。

Task 19 の test は、label と明示 reference を持つ `thus`、明示的な `by` を持たない
`hence`、compact / conclusion / 現在実装済み introduction statement を wrap する
`then`、standalone statement 前の `then` rejection、1 個または複数の `.=` step を持つ
iterative equality、`x = y by A;` と `x = y by A .= z by B;` の compact-versus-
iterative boundary、その label / `then` variant、malformed conclusion proposition、
iterative-equality term 欠落、iterative equality 内の disallowed computation
justification、active parse-only pass/fail corpus coverage、Chapter 15 §15.4.1、
§15.4.2、§15.7、§15.8、§15.9.1 への traceability を固定する必要がある。

## Task 20: Block statement

Parser task 20 は、Chapter 15 の reasoning block に concrete syntax node を追加して
mizar-syntax S-013 を継続する。task 19 で deferred だった `then per cases`
placeholder path は、case-reasoning body が parse 可能な場合、`CaseReasoningStatement`
を wrap する `ThenStatement` に昇格する。

```ebnf
statement_item              ::= ... | now_statement | hereby_statement
                              | case_reasoning_statement ;
linkable_statement          ::= ... | case_reasoning_statement ;
standalone_statement        ::= ... | now_statement | hereby_statement ;
now_statement               ::= [ label_identifier ":" ] "now"
                                reasoning_body "end" ";" ;
hereby_statement            ::= "hereby" reasoning_body "end" ";" ;
case_reasoning_statement    ::= "per" "cases"
                                [ simple_justification ] ";"
                                ( case_list | suppose_list | empty_branch_list ) ;
case_list                   ::= case_item { case_item } ;
suppose_list                ::= suppose_item { suppose_item } ;
empty_branch_list           ::= /* fragment recovery 専用で受け入れる */ ;
case_item                   ::= "case" ( proposition | conditions ) ";"
                                reasoning_body "end" ";" ;
suppose_item                ::= "suppose" ( proposition | conditions ) ";"
                                reasoning_body "end" ";" ;
reasoning_body              ::= { statement } ;
```

parser は block reasoning を syntax-only として保持する。`NowStatement` は任意の
label と colon、`now` token、0 個以上の nested statement node、closing `end`、任意の
recovery、存在する場合の closing semicolon を所有する。`HerebyStatement` は label
を持たない同じ block-body 形状である。`CaseItem` と `SupposeItem` は branch keyword、
leading `that` により選ばれる `Proposition` または `ConditionList`、header
semicolon、0 個以上の nested statement node、branch-closing `end`、任意の recovery、
存在する場合の closing semicolon を所有する。`CaseReasoningStatement` は `per`、
`cases`、任意の simple citation `JustificationClause`、header semicolon、source order の
`CaseItem` / `SupposeItem` child を所有する。
branch child は homogeneous であり、source order の `CaseItem` だけ、または
source order の `SupposeItem` だけで構成する。最初の branch kind が見えた後、もう一方の
branch keyword は現在の case-reasoning node の外にある statement boundary とする。parser は
`case` と `suppose` list を silent に混在させてはならない。

Chapter 15 の prose と例は `per cases;` を含む一方、complete EBNF summary は
bracket なしの `simple_justification` を表示する。grammar audit G-AUD-011 はこの
nonblocking inconsistency を記録する。parser surface は `per cases;` と
`per cases by A;` の両方を受け入れる。active parse-only fixture は complete proof の外で
statement fragment を exercise し得るため、branch を持たない `per cases;` fragment も
diagnose しない。ただし後続する `case` または `suppose` branch がある場合は保持する。

Task 20 recovery は既存 diagnostic を再利用する。欠落 block `end` token は block opener
を secondary anchor とする `MissingEnd` diagnostic と `MissingEnd` recovery を使う。
block `end` 後または case header 後の semicolon 欠落は `MissingSemicolon` を使う。
case / suppose proposition 欠落は `MalformedFormulaExpression` と `MissingFormula` を使う。
malformed block tail は semicolon、`end`、EOF、または次の statement/item boundary で同期し、
token を skip する必要がある場合は `SkippedToken` recovery として source を保持する。

Task 20 の test は、labelled `now` block、`hereby` block、block body 内の nested
statement、`case` branch を持つ `per cases`、`suppose` branch を持つ `per cases`、
mixed branch-list keyword の rejection / recovery、`then per cases`、`then now` /
`then hereby` の rejection、`per cases` 後の optional simple `by`、`per cases` 後の
`by computation(...)` rejection、proposition 形と condition-list 形の branch header、
branch/body `end` 欠落 recovery、branch-header semicolon 欠落 recovery、active
parse-only pass/fail corpus coverage、Chapter 15 §15.4.3、§15.6.1、§15.6.2、§15.6.3、
§15.8、§15.9.1 への traceability を固定する必要がある。

### Task 21: ローカル定義

Task 21 は Chapter 15 の inline definition を concrete statement node にして
S-013 statement-node bucket を完了させる。parser は standalone statement 形だけを
受け付ける。Chapter 15 は inline definition を `linkable_statement` に含めないため、
`then deffunc` と `then defpred` は不正なままにする。

```ebnf
standalone_statement        ::= ... | inline_functor_definition
                                   | inline_predicate_definition ;
inline_functor_definition   ::= "deffunc" identifier "(" [ typed_parameters ] ")"
                                "->" type_expression "equals"
                                term_expression ";" ;
inline_predicate_definition ::= "defpred" identifier "(" [ typed_parameters ] ")"
                                "means" formula ";" ;
typed_parameters            ::= typed_parameter { "," typed_parameter } ;
typed_parameter             ::= identifier ( "being" | "be" ) type_expression ;
```

`InlineFunctorDefinition` は `deffunc` keyword、definition-name slot、parameter
parentheses、comma token で区切られた source-order の `TypedParameter` child 0 個以上、
`->` token、1 個の return `TypeExpression` または `MissingTypeExpression`
recovery、`equals` keyword、1 個の `TermExpression` または `MissingTerm`
recovery、任意の malformed-tail recovery、存在する場合の final semicolon を所有する。
`InlinePredicateDefinition` は同じ head 形状に加えて、`defpred` keyword、`means`
keyword、1 個の `FormulaExpression` または `MissingFormula` recovery を所有する。
definition-name slot は、書かれた identifier token または `MissingTerm` recovery である。
`TypedParameter` は存在する場合の parameter identifier token、書かれた場合の optional
`be` または `being`、`TypeExpression` または `MissingTypeExpression` recovery を所有する。
binder keyword が欠落しているが parameter-list delimiter の前で type を parse できる場合、
parser はその type を `TypedParameter` 配下に保持し、binder 欠落を診断する。それ以外では
delimiter 位置に `MissingTypeExpression` を挿入する。

parser は `->`、`equals`、`means` を inline-definition delimiter として扱う。
これらは top-level の type / term / formula expression parsing と recovery を止めるが、
expression operator にはならない。inline definition parsing は純粋に syntax-only
である。definition expansion、captured variable validation、parameter guard
satisfaction、scope binding の導入、後続の `deffunc` / `defpred` name application の
分類は行わない。

Task 21 recovery は既存 diagnostic を再利用する。definition name 欠落は
`MalformedTermExpression` と `MissingTerm` recovery を使う。semicolon 欠落は
`MissingSemicolon` を使う。`(`、`)`、`->`、`equals`、`means` delimiter 欠落は、
recovery が継続できる場合に inline-definition node を保持しながら、最も近い既存の
malformed type / term / formula diagnostic を使う。parameter / return type 欠落は
`MalformedTypeExpression` と `MissingTypeExpression`、
functor body 欠落は `MalformedTermExpression` と `MissingTerm`、predicate body 欠落は
`MalformedFormulaExpression` と `MissingFormula` を使う。malformed parameter list と
definition tail は `,`、`)`、`->`、`equals`、`means`、semicolon、`end`、次の
statement boundary、次の item boundary、EOF で同期する。

Task 21 tests は、typed parameter 付き `deffunc`、zero-argument `deffunc`、typed
parameter 付き `defpred`、zero-argument `defpred`、reasoning body 内での使用、
`then deffunc` / `then defpred` の rejection、definition name 欠落、parameter type
binder または type 欠落、`)` 欠落、`->` 欠落、return type 欠落、`equals` 欠落、
functor body 欠落、`means` 欠落、predicate body 欠落、semicolon 欠落、active
parse-only pass/fail corpus coverage、Chapter 15 §15.2.3、§15.2.4、§15.9.1 への
traceability を固定する必要がある。

### Task 22: 定理と証明

Task 22 は、表現済みの theorem/lemma formula と proof tail を concrete item node に
置き換えて S-014 theorem/proof increment を完了する。parser は canonical Chapter 16 の
theorem item 形を受け入れるが syntax-only のままである。status token は保存するだけで
validity は検証せず、reference resolution、proof obligation、theorem validity も扱わない。

```ebnf
theorem_item     ::= [ theorem_status ] theorem_role label_identifier ":"
                     formula [ justification ] ";" ;
theorem_status   ::= "open" | "assumed" | "conditional" ;
theorem_role     ::= "theorem" | "lemma" ;
justification    ::= justification_clause | proof_block ;
proof_block      ::= "proof" reasoning "end" ;
reasoning        ::= { statement } ;
```

`TheoremItem` と `LemmaItem` は optional status token、role token、label identifier または
`MissingTerm`、存在する場合の colon token、`FormulaExpression` または `MissingFormula`、
任意の `JustificationClause` または `ProofBlock`、任意の recovery、存在する場合の
final semicolon を所有する。visibility wrapper（`public` / `private`）は
`VisibilityMarker` と concrete theorem / lemma target を所有する。notation target は、
対応する item grammar が入るまで既存 placeholder path を使い続ける。

`ProofBlock` は `proof`、reasoning-body parser が parse した nested concrete statement
node、`MissingEnd` を含む任意の recovery、存在する場合の `end` を所有する。後続
semicolon は enclosing theorem item または statement が所有する。Task 22 は
`ProofBlock` を theorem/lemma item と、canonical grammar が `justification` を使う
既存 concrete statement host（`ConclusionStatement` と `CompactStatement`）に受け入れる。
`simple_justification`を使うhost（`let`、`consider`、iterative equality step、`per
cases`）は引き続きtask-17の`by` clauseだけを受け入れる。Task 47は別途
`reconsider_tail`を通して`ProofBlock`を受け入れる。

concrete theorem path は、`theorem T;` のような短い legacy fragment を意図的に
token-preserving placeholder として残す。これは earlier parser skeleton tests が generic item
boundary として使うためである。表現される theorem shape は colon、label-colon pair、
または label の後に formula start が見える missing-colon form から始まる。predicate
template argument を含む formula payload は task 31 / S-016 まで placeholder に残していたが、
task 31 以後は theorem host が concrete なら表現する。

Task 22 recovery は既存 diagnostic を再利用する。theorem label 欠落は
`MalformedTermExpression` と `MissingTerm` を使う。colon 欠落と formula 欠落は
`MalformedFormulaExpression` を使い、formula 欠落は `MissingFormula` を挿入する。
proof `end` 欠落は `MissingEnd` diagnostic と `MissingEnd` recovery を使う。parser は
theorem/proof tail を semicolon、`end`、次の statement / item boundary、case/suppose branch
keyword、EOF で同期し、proof `end` 欠落後の次の theorem item を飲み込んではならない。

Task 22 tests は、theorem / lemma item、status token、theorem target の visibility wrapper、
`by` と `by computation` の theorem justification、full theorem proof block、proof-body
statement wiring、conclusion と compact statement 上の statement-level proof justification、
label / colon / formula / proof-end 欠落 recovery、active parse-only pass/fail corpus coverage、
Chapter 16 §16.2、§16.4.1、§16.5、Chapter 20 §20.9.2 への traceability を固定する必要がある。

### Task 23: definition block と属性定義

Task 23 は、共有 `definition ... end;` container を concrete にし、最初の concrete
definition content を追加することで S-015 を開始する。parser は syntax-only のままであり、
symbol 導入、attribute 解決、correctness obligation 検査、template declaration の意味的分類、
既存の statement/proof 文法を超える proof body 検証は行わない。

```ebnf
definition_block       ::= "definition" { definition_content } "end" ";" ;
definition_content     ::= definition_parameter_decl
                         | assumption
                         | correctness_condition
                         | attr_def
                         | [ visibility ] theorem_item
                         | placeholder_definition_content ;

definition_parameter_decl ::= "let" definition_qualified_vars
                              [ definition_parameter_constraint ] ";" ;
definition_parameter_constraint ::= "such" conditions
                                  | "such" "that" formula
                                    ( "by" references | proof_block )
                                  | "by" references ;

attr_def               ::= "attr" label ":" subject "is" attr_pattern
                           "means" formula_definiens ";" ;
attr_pattern           ::= [ param_prefix ] attribute_def_name ;
formula_definiens      ::= formula
                         | formula_case { "," formula_case }
                           [ "otherwise" formula ] ;
formula_case           ::= formula "if" formula ;

correctness_condition  ::= ( "existence" | "uniqueness" | "coherence"
                           | "compatibility" | "consistency"
                           | "reducibility" )
                           [ justification ] ";" ;
```

`DefinitionBlockItem` は `definition` token、source order の definition content node、
closing `end` token または `MissingEnd`、存在する場合の final semicolon を所有する。
concrete content は paired parser task を通じて拡張済みである:
`definition_qualified_vars` を持つ通常の `let` parameter、assumption statement、
correctness-condition clause、attribute / predicate / functor / mode definition、
redefinition、property clause、structure、registration、notation alias、theorem / lemma
item、visibility-wrapped theorem target、task 35 の annotation wrapper を所有できる。
`let T be type;` のような template-like parameter は、該当する場合 task 31 の template
surface を使う。

`DefinitionParameter` は、仕様が許す場所で `QualifiedVariableSegment`、
`ConditionList`、`JustificationClause`、`ProofBlock` を再利用する。parser は
未解決の definition/template ambiguity に対して AST 形を創作せず、template-ambiguous
binder を placeholder として保持する。

`AttributeDefinition` は `attr` keyword、label、colon、subject token、`is`、
`AttributePattern`、`means`、`FormulaDefiniens`、recovery node、terminating semicolon を
所有する。`AttributePattern` は任意の task-8 `ParameterPrefix` と、
コンストラクタ名形の属性名を保持する。任意の演算子風のユーザーシンボル綴りは
属性名ではない。`FormulaDefiniens` は単一の
`FormulaExpression`、または comma で区切られた `FormulaCase` 列と任意の
`otherwise` formula を所有する。`FormulaCase` は value formula、`if`、condition
formula を所有する。

`CorrectnessCondition` は correctness keyword のいずれかと、任意の general justification
を所有する。`existence;` のような空の simple justification は有効であり、recovery node
を作らない。仕様の `correctness_condition` tail は `justification` を使うため、
通常の `by` reference、`by computation(...)`、full proof block を受け入れる。

Task 23 recovery は既存 diagnostic を再利用する。attribute label、subject、pattern の
欠落は `MalformedTermExpression` と `MissingTerm` を使う。`means` または formula
definiens の欠落は `MalformedFormulaExpression` と `MissingFormula` を使う。malformed
correctness-condition tail は `MalformedJustification` と skipped-token recovery を使う。
definition `end` 欠落は `MissingEnd` を使う。concrete parser が対応する recovery node を
挿入するとき、pre-pass の missing-end diagnostic との重複は抑制する。definition content
recovery は semicolon、`end`、次の認識済み definition-content start、EOF で同期し、
未対応 content の placeholder は top-level placeholder と同じ文脈付き block rule で
nested block-like construct を scan する。

Task 23 tests は、concrete definition block、ordinary definition parameter、
template-ambiguous content の placeholder preservation、attribute definition、
single-formula body と `otherwise` 付き formula-definiens case、空 / reference /
computation / proof justification を持つ correctness condition、assumption content、
direct theorem / lemma content、definition 内の visible theorem / lemma content、
malformed attribute / correctness recovery、active parse-only pass/fail corpus coverage、
Chapter 6 §6.2 / Appendix A.6、Chapter 16 §16.2 / §16.6 / Appendix A.16、
Chapter 20 §20.9.2 への traceability を固定する必要がある。

### Task 24: 述語定義

Task 24 は、task-23 の `DefinitionBlockItem` container 内に
`pred ... means ...;` definition form を追加する。parser は syntax-only のままで、
predicate symbol の導入、overload 解決、phrase-pattern role の決定、parameter typing、
predicate property の証明、template definition の分類は行わない。

```ebnf
definition_content     ::= ... | pred_def | [ visibility ] pred_def ;

pred_def               ::= "pred" label ":" pred_pattern
                           "means" formula_definiens ";" ;
pred_pattern           ::= [ loci ] def_predicate_symbol
                           [ template_loci ] [ loci ] ;
loci                   ::= locus_list | "(" locus_list ")" ;
locus_list             ::= locus { "," locus } ;
locus                  ::= identifier ;
template_loci          ::= "[" locus_list "]" ;
def_predicate_symbol   ::= identifier | user_symbol ;
```

`PredicateDefinition` は `pred` keyword、label identifier または `MissingTerm`、
colon、`PredicatePattern`、`means`、task-23 の `FormulaDefiniens`、任意の
recovery、存在する場合の semicolon を所有する。definition-local な
`public pred` と `private pred` は、既存の `VisibleItem` と `VisibilityMarker`
wrapper で concrete predicate definition を包んで表す。他の visible definition kind は、
それぞれの owning task に残す。

`PredicatePattern` は、left-loci / predicate-symbol / right-loci role を記録せず、
source-order の raw pattern token を保持する。任意の balanced token を受理しないため、
parser は raw span が少なくとも 1 通りの syntactic split で `pred_pattern` に
一致できる場合だけ受理する。`loci` は non-empty identifier comma-list であり、
parenthesized でもよい。`template_loci` は最大 1 個の bracketed non-empty
identifier comma-list である。`def_predicate_symbol` はちょうど 1 個の identifier、
active user-symbol、または lexeme-run token である。active parse-only source fixture は
imported symbolic predicate token を扱う。lexeme-run case は、frontend が
definition-symbol lexing context を持った時に fresh symbolic predicate definition を
parser-token boundary で受け取れるようにするためのものである。空 group、dangling comma、
隣接 loci group、複数 bracket group、
unsupported token は malformed predicate pattern として recover する。primitive
built-in predicate token の `in`、`=`、`<>` は definition symbol ではないため、
predicate definition pattern を形成せず recover する。

template-loci token は `PredicatePattern` に保持してよいが、task 24 は
template-definition fixture を有効化せず、template-specific AST node を追加せず、
`definition ... end;` block を template definition として分類しない。
`let T be type;` のような template-ambiguous parameter の後では、definition block は
G-AUD-006 のもとで後続 content も placeholder として保持し続ける。

Task 24 recovery は task-23 の definition-content synchronization を再利用する。
predicate label 欠落と malformed predicate pattern は `MalformedTermExpression` と
`MissingTerm` を使う。`means` 欠落、formula-definiens body 欠落、formula case 欠落、
`otherwise` body 欠落は `MalformedFormulaExpression` と `MissingFormula` を使う。
malformed predicate definition tail は semicolon、`end`、次の definition-content
start、または EOF まで skip してよい。

Task 24 tests は、通常 predicate definition、raw phrase / infix / multi-loci pattern、
imported symbolic predicate token、parser-token lexeme-run symbolic predicate pattern、
formula definiens case、definition-local visibility、template definition 分類なしの
template-loci token preservation、template-ambiguous parameter 後の placeholder
preservation、built-in predicate-symbol rejection、malformed pattern recovery、active
parse-only pass/fail corpus coverage、Chapter 9 §9.1 / §9.3 / §9.4 / §9.5 / §9.10 への
traceability を固定する必要がある。

### Task 25: ファンクタ定義

Task 25 は、task-23 の `DefinitionBlockItem` container 内に
`func ... -> ... means|equals ...;` definition form を追加する。parser は
syntax-only のままで、functor symbol の導入、overload 解決、return-type subtype
checking、existence / uniqueness 証明、`it` の評価、template definition の分類は行わない。

```ebnf
definition_content     ::= ... | func_def | [ visibility ] func_def ;

func_def               ::= "func" label ":" func_pattern
                           "->" type_expression
                           ( "means" formula_definiens
                           | "equals" term_definiens ) ";" ;
func_pattern           ::= [ loci ] functor_symbol
                           [ template_loci ] [ loci ] ;
loci                   ::= locus_list | "(" locus_list ")" ;
locus_list             ::= locus { "," locus } ;
locus                  ::= identifier ;
template_loci          ::= "[" locus_list "]" ;
functor_symbol         ::= identifier | user_symbol ;
term_definiens         ::= term_expression
                         | term_case { "," term_case }
                           [ "otherwise" term_expression ] ;
term_case              ::= term_expression "if" formula ;
```

`FunctorDefinition` は `func` keyword、label identifier または `MissingTerm`、
colon、`FunctorPattern`、`->`、return `TypeExpression` または
`MissingTypeExpression`、body keyword（`means` または `equals`）、task-23 の
`FormulaDefiniens` または task-25 の `TermDefiniens`、任意の recovery、存在する場合の
semicolon を所有する。definition-local な `public func` と `private func` は、
既存の `VisibleItem` / `VisibilityMarker` wrapper で concrete functor definition を
包んで表す。functor の後に続く correctness-condition clause は task 23 で確立した通り、
別個の definition-content node のままである。

`FunctorPattern` は left-loci / functor-symbol / right-loci role を記録せず、
source-order の raw pattern token を保持する。parser は raw span が canonical な
single-symbol `func_pattern` に少なくとも 1 通りの syntactic split で一致できる場合に
受理する。さらに Chapter 10 で文書化されている circumfix surface shape、つまり
2 個の functor-symbol token が non-empty loci list を挟む形も受理し、semantic role を
割り当てず raw token として保持する。`loci` は non-empty identifier comma-list であり、
parenthesized でもよい。`template_loci` は最大 1 個の bracketed non-empty
identifier comma-list である。functor-symbol token はちょうど 1 個の identifier、
active user-symbol、または lexeme-run token である。active parse-only source fixture は
imported symbolic functor token を扱う。lexeme-run case は、frontend が
definition-symbol lexing context を持った時に fresh symbolic functor definition を
parser-token boundary で受け取れるようにするためのものである。空 group、dangling comma、
隣接 loci group、複数 bracket group、unsupported token は malformed functor pattern として
recover する。

`TermDefiniens` は `equals` body 用に `FormulaDefiniens` と対応する。1 個の
`TermExpression`、または comma token で区切られた source-order `TermCase` child と
任意の `otherwise TermExpression` を所有する。`TermCase` は value `TermExpression`、
`if`、condition `FormulaExpression` を所有する。

template-loci token は `FunctorPattern` に保持してよいが、task 25 は
template-definition fixture を有効化せず、template-specific AST node を追加せず、
schema functor parameter を解析せず、`definition ... end;` block を template definition
として分類しない。`let F be func(T) -> S;` のような canonical schema-functor parameter
の後では、definition block は G-AUD-006 のもとで後続 content も placeholder として
保持し続ける。

Task 25 recovery は task-23 の definition-content synchronization を再利用する。
functor label 欠落、malformed functor pattern、`equals` term body 欠落、
term case 欠落、`equals ... otherwise` term body 欠落は `MalformedTermExpression` と
`MissingTerm` を使う。return type 欠落は `MalformedTypeExpression` と
`MissingTypeExpression` を使う。colon 欠落、`->` delimiter 欠落、body keyword 欠落、
`means` formula body 欠落、formula case 欠落、`means ... otherwise` formula body 欠落、
`TermCase` condition formula 欠落は、formula child を挿入する必要がある場合に
`MalformedFormulaExpression` と `MissingFormula` を使う。body keyword が欠落している場合、
parser は現在 token が formula を開始できれば `FormulaDefiniens`、そうでなければ現在 token が
term を開始できる場合に `TermDefiniens` を選んで、次の parseable branch を保持する。
どちらも開始できない場合は canonical recovery child として missing formula body を挿入して
同期する。malformed functor definition tail は semicolon、`end`、次の
definition-content start、または EOF まで skip してよい。

Task 25 tests は、通常の `means` / `equals` functor definition、raw identifier、
prefix、postfix、infix、parenthesized-argument、circumfix、imported symbolic、
parser-token lexeme-run symbolic functor pattern、`otherwise` 付き term definiens case、
`means` 用の formula definiens 再利用、definition-local visibility、template definition
分類なしの template-loci token preservation、canonical schema-functor parameter 後の
placeholder preservation、malformed pattern / colon / arrow / return / body-keyword /
body recovery、active parse-only pass/fail corpus coverage、Chapter 10 §10.1 / §10.3 /
§10.5 / §10.6 / §10.8 / §10.13 への traceability を固定する必要がある。

### Task 26: mode 定義

Task 26 は、task-23 の `DefinitionBlockItem` container 内に canonical な
`mode ... is ...;` definition を追加する。parser は syntax-only のままであり、
mode symbol の導入、semantic な radix type と mode / structure type の区別、
existence 証明、sethood 証明、dependent-mode parameter の妥当性確認、legacy な
`means` mode-definition body の受理は行わない。

```ebnf
definition_content     ::= ... | mode_def | [ visibility ] mode_def ;

mode_def               ::= "mode" label ":" mode_pattern
                           "is" type_expression ";"
                           [ mode_property ] ;
mode_pattern           ::= mode_def_name [ type_params ] ;
mode_def_name          ::= constructor_name ;
type_params            ::= ( "of" | "over" ) type_parameter_list
                         | "[" type_parameter_list "]" ;
type_parameter_list    ::= identifier { "," identifier } ;
mode_property          ::= "sethood" justification ";" ;
```

`ModeDefinition` は `mode` keyword、label identifier または `MissingTerm`、
colon、`ModePattern`、`is`、body `TypeExpression` または
`MissingTypeExpression`、存在する場合の最初の semicolon、直後の `sethood`
property が mode definition に属する場合の任意の `ModeProperty` を所有する。
definition-local な `public mode` と `private mode` は、concrete mode definition を
既存の `VisibleItem` / `VisibilityMarker` wrapper で包んで表す。

`ModePattern` は `mode_def_name [ type_params ]` span の source-order raw token を
保持する。mode definition name はちょうど 1 個の通常の識別子、または読みやすい
ハイフン区切りのコンストラクタ名トークンでなければならない。type parameter は、
`of` または `over` が導入する non-empty identifier comma-list、または bracketed
non-empty identifier comma-list のいずれか 1 個だけを任意で持てる。空の parameter
list、dangling comma、複数の parameter group、任意のユーザーシンボル / lexeme-run
token、unsupported token は malformed mode pattern として recover する。AST は
parameter list が意味的に dependent か、structure 上のものか、その他に妥当かを記録しない。

mode body は、Chapter 7 の attribute-chain plus radix-type surface に task-8 の
`TypeExpression` を再利用する。この表現は attribute chain と type head を syntactic に
保持する。radix / mode / structure head の区別は resolver と semantic phase が所有する。
`mode_property` は `sethood`、必須の general justification（`by`、
`by computation(...)`、または `proof ... end`）、任意の recovery、property semicolon
を所有する。mode definition の直後にない standalone `sethood` や他の property clause は
この task の対象外であり、後続の property-content shape として保持される。

Task 26 recovery は task-23 の definition-content synchronization を再利用する。
mode label 欠落と malformed mode pattern は `MalformedTermExpression` と
`MissingTerm` を使う。`is` 後の body type 欠落は `MalformedTypeExpression` と
`MissingTypeExpression` を使う。colon 欠落、`is` delimiter 欠落、malformed
definition tail は、delimiter または tail preservation 用の既存 formula/term recovery
diagnostic を使う。semicolon 欠落時は、`sethood`、次の definition-content start、
`end`、または EOF で継続する。`by` / `proof` を持たない `sethood` property は
`MalformedJustification` を出す。malformed property tail は property semicolon、
次の definition-content start、`end`、または EOF まで skip してよい。

Task 26 tests は、通常の canonical `is` mode definition、読みやすいハイフン区切りの
コンストラクタ名による mode definition、mode body 内の attribute chain、`of` / `over` /
bracketed type-parameter list、definition-local visibility、citation / computation / proof justification を伴う
`sethood` clause、legacy `means` mode body を recovered syntax として拒否すること、
malformed label / colon / pattern / `is` / body / semicolon /
property-justification recovery、active parse-only pass/fail corpus coverage、
Chapter 7 §7.2 / §7.6 / §7.7 / §7.8 / §7.8.1 への traceability を固定する必要がある。

### Task 27: redefinition と notation alias

Task 27 は、仕様で定義された redefinition form と symbol-management alias form を
追加する。parser は syntax-only のままであり、redefine 対象の previous definition
の解決、coherence 証明、overload membership の決定、alias pattern の active symbol kind
による分類、synonym / antonym の semantic fact 作成は行わない。

```ebnf
definition_content     ::= ... | [ visibility ] redefine_attr
                         | [ visibility ] redefine_pred
                         | [ visibility ] redefine_func
                         | [ visibility ] notation_alias_decl ;
declaration            ::= ... | [ visibility ] notation_decl ;

redefine_attr          ::= "redefine" "attr" label ":" subject "is"
                           attr_pattern "means" formula_definiens ";"
                           coherence_tail ;
redefine_pred          ::= "redefine" "pred" label ":" pred_pattern
                           "means" formula_definiens ";"
                           coherence_tail ;
redefine_func          ::= "redefine" "func" label ":" func_pattern
                           "->" type_expression
                           ( "means" formula_definiens
                           | "equals" term_definiens ) ";"
                           coherence_tail ;
coherence_tail         ::= "coherence" [ "with" label ] justification ";" ;

notation_decl          ::= operator_decl | notation_alias_decl ;
notation_alias_decl    ::= synonym_def | antonym_def ;
synonym_def            ::= "synonym" notation_pattern
                           "for" notation_pattern ";" ;
antonym_def            ::= "antonym" notation_pattern
                           "for" notation_pattern ";" ;
notation_pattern       ::= top-level の "for"、semicolon、definition boundary、
                           item boundary まで受理される raw token ;
```

Status note: parser task 36 は、実装と active corpus を上の修正済み Chapter 9 /
Appendix A `redefine_pred` production に同期済みである。task 36 以前の label なし
surface は architecture contract ではない。

`AttributeRedefinition`、`PredicateRedefinition`、`FunctorRedefinition` は、grammar
が同一の箇所で task 23〜25 の pattern parser と definiens parser を再利用し、その後に
必須 `CoherenceCondition` tail を所有する。`AttributeRedefinition` は `redefine`、
`attr`、label、`:`、subject、`is`、`AttributePattern`、`means`、
`FormulaDefiniens`、存在する場合の最初の semicolon、`CoherenceCondition` を所有する。
`PredicateRedefinition` は `redefine`、`pred`、label、`:`、`PredicatePattern`、
`means`、`FormulaDefiniens`、最初の semicolon、`CoherenceCondition` を所有する。
`FunctorRedefinition` は `redefine`、`func`、label、`:`、`FunctorPattern`、`->`、return
`TypeExpression`、選択された `means` / `equals` definiens branch、最初の semicolon、
`CoherenceCondition` を所有する。
definition-local な `public` / `private` redefinition は、Appendix A の
`[ visibility ] definitional_item` shape に合わせ、concrete redefinition node を既存の
`VisibleItem` / `VisibilityMarker` wrapper で包んで表す。

`CoherenceCondition` は `coherence`、任意の `with` と label identifier、必須の
general justification（`by` references、`by computation(...)`、または
`proof ... end`）、任意の recovery、coherence semicolon を所有する。これは standalone
`CorrectnessCondition` として emit せず、redefinition の下に nest する。

canonical spec には `redefine_attr`、`redefine_pred`、`redefine_func` production が
存在するが、`redefine_mode` production は存在しない。Task 27 はこの不在を局所的な
仕様境界として扱い、`redefine mode` concrete node を創作しない。mode syntax は
`synonym` / `antonym` の raw notation pattern として参加し、`redefine mode` source は
後続の human-reviewed spec change が production を追加しない限り placeholder /
recovery のままにする。

`NotationAlias` は top level と definition block 内の `synonym` と `antonym` の両方を
表す。operator declaration は canonical `notation_decl` の deferred branch のままであり、
この task は `notation_alias_decl` だけを実装する。`NotationAlias` は alias keyword、
alternate `NotationPattern`、`for` token、original `NotationPattern`、任意の recovery、
terminating semicolon を所有する。
definition-local と top-level の `public` / `private` alias は既存の `VisibleItem`
wrapper を使う。`NotationPattern` は、`alt_pattern` / `original_pattern` の predicate、
functor、mode、attribute branch を選ばず、各側を source-order raw token として保持する。
その branch は symbol table に依存するため resolver-owned のまま残る。この生の
保持は言語上の制約を緩和しない。述語 / functor の alias branch は任意のユーザー
シンボル記法を含められるが、モード / 属性の alias branch はコンストラクタ名の
綴りを使う。

Task 27 recovery は task 23〜26 の definition-content synchronization を再利用する。
redefinition label、subject、malformed raw pattern、`equals` term body、
notation-pattern placeholder は、挿入 child が必要な場合に `MalformedTermExpression`
と `MissingTerm` を使う。`redefine func` return type 欠落は
`MalformedTypeExpression` と `MissingTypeExpression` を使う。colon、`is`、`->`、
body keyword、`means` formula body、formula case、term-case condition、notation の
`for`、必須 `coherence` keyword 欠落は、formula child が必要な場合は対応する
inserted formula とともに `MalformedFormulaExpression` を使う。coherence justification
の欠落または malformed syntax、`coherence with` 後の label 欠落は、
placeholder proof step が必要な場合に `MalformedJustification` と `MissingProofStep` を
使う。malformed tail は semicolon、`end`、次の definition-content start、top-level
item boundary、または EOF まで skip してよい。

Task 27 tests は、`coherence by ...;`、`coherence with Label by ...;`、
proof-block coherence を伴う attribute / predicate / functor redefinition、必須 label slot
と missing-label recovery を伴う predicate redefinition、`means` と `equals` の両方の functor redefinition、
mode-like / attribute-like raw pattern を含む top-level と definition-local の
`synonym` / `antonym` alias、visibility-wrapped redefinition と alias、concrete
`redefine mode` が存在しないこと、malformed pattern / body / coherence / alias
recovery、active parse-only pass/fail corpus coverage、Chapter 6 §6.7、Chapter 9
§9.6、Chapter 10 §10.7、Chapter 11 §11.1 / §11.6、Appendix A.11 への traceability を
固定する必要がある。

### Task 28: Property Clauses

Task 28 は syntax-only の definition-content property 句を追加する。parser は canonical
grammar に列挙される property keyword だけを受理する。Chapter 9 の predicate property、
Chapter 10 の functor property、Chapter 7 / Appendix A の standalone mode `sethood` である。
現在の `doc/spec/en` の property production には `transitivity` がないため、これを property
句として創作しない。また、形が曖昧な `property_impl` block surface もこの task では
実装しない。

```ebnf
definition_content     ::= ... | property_item ;
property_item          ::= pred_property | func_property | mode_property ;
pred_property          ::= ( "symmetry" | "asymmetry" | "connectedness"
                           | "reflexivity" | "irreflexivity" )
                           justification ";" ;
func_property          ::= ( "commutativity" | "idempotence"
                           | "involutiveness" | "projectivity" )
                           justification ";" ;
mode_property          ::= "sethood" justification ";" ;
```

`PropertyClause` は property keyword、存在する場合の必須 general justification
（`by` references、`by computation(...)`、または `proof ... end`）、任意の recovery、
存在する場合の property semicolon を所有する。`mode` definition 直後の `sethood` 句は
引き続き task-26 の `ModeProperty` として `ModeDefinition` に所有される。standalone の
`sethood` property item は `PropertyClause` を使う。

Task 28 recovery は definition-content synchronization を再利用する。property
justification の欠落または malformed syntax は、proof placeholder が必要な場合に
`MalformedJustification` と `MissingProofStep` を使う。malformed property tail は
semicolon、`end`、次の definition-content start、top-level item boundary、または EOF まで
skip してよい。property semicolon 欠落は `MissingSemicolon` を使い、別の property 句を
含む後続 definition item を消費せずに継続する。

Task 28 tests は、canonical predicate / functor property keyword 一式、standalone
`sethood`、citation / computation / proof justification、task-26 の mode-attached
`ModeProperty` の保持、missing / malformed justification recovery、別 property item
直前の missing semicolon recovery、active parse-only pass/fail corpus coverage、Chapter 7
§7.8.1、Chapter 9 §9.5.1、Chapter 10 §10.6.1、Appendix A.12 への traceability を
固定する必要がある。

### Task 29: Structures

Task 29 は definition block 内の syntax-only な structure definition と inheritance
definition を追加する。parser は structure name、type parameter、field / property
declaration、inheritance target、explicit な field / property mapping を保持するが、
structure identity の解決、inheritance coherence の証明、parent coverage の検査、
type narrowing の妥当性確認、selector fact の作成、constructor 導出は行わない。

```ebnf
definition_content     ::= ... | [ visibility ] struct_def
                         | [ visibility ] inherit_def ;

struct_def             ::= "struct" struct_pattern "where"
                           struct_member { struct_member } "end" ";" ;
struct_pattern         ::= struct_def_name [ type_params ] ;
struct_member          ::= field_decl | property_decl ;
field_decl             ::= "field" identifier "->" type_expression
                           [ ":=" term_expression ] ";" ;
property_decl          ::= "property" identifier "->" type_expression ";" ;

inherit_def            ::= "inherit" inherit_child "extends" parent_type
                           ( ";"
                           | "where" inherit_member { inherit_member }
                             [ inheritance_coherence ] "end" ";" ) ;
inherit_child          ::= struct_name [ type_args ] ;
parent_type            ::= struct_name [ type_args ] | "set" ;
inherit_member         ::= field_redef | property_redef ;
field_redef            ::= "field" identifier [ "->" type_expression ]
                           "from" ( identifier | "it" ) ";" ;
property_redef         ::= "property" identifier [ "->" type_expression ]
                           "from" identifier ";" ;
inheritance_coherence  ::= "coherence" justification ";" ;
```

`StructureDefinition` は `struct`、raw `StructurePattern`、`where`、1 個以上の
`StructureField` / `StructureProperty` member、`end`、存在する場合の final semicolon
を所有する。`StructurePattern` は `of`、`over`、bracket parameter を含む
source-order の structure definition name / parameter token を所有する。structure name は
通常の識別子、または読みやすいハイフン区切りのコンストラクタ名に制限され、任意の
ユーザーシンボルトークンではない。definition-local な `public` / `private` structure definition は
既存の `VisibleItem` / `VisibilityMarker` wrapper を再利用する。

`StructureField` は `field`、field identifier、`->`、`TypeExpression`、`:=` で始まる
任意の `TermExpression` initializer、member semicolon を所有する。
`StructureProperty` は initializer を持たない同じ member skeleton を所有する。parser は
grammar shape だけを確認し、selector declaration の妥当性と field / property の一意性は
後続 phase に残す。

`InheritanceDefinition` は `inherit`、child `InheritanceTarget`、`extends`、parent
`InheritanceTarget`、shorthand semicolon または explicit `where ... end;` block、および
nested redefinition / coherence node を所有する。explicit `where` block には少なくとも
1 個の `FieldRedefinition` または `PropertyRedefinition` が必要である。shorthand
inheritance では synthetic mapping node を作らない。`InheritanceTarget` は child /
parent の structure-like reference と任意の raw type argument、または parent 側の `set`
token を保持し、structure / type identity は解決しない。`FieldRedefinition` と
`PropertyRedefinition` は child member name、任意の narrowed `TypeExpression`、必須
`from`、source member name（field だけ `from it` を許す）、member semicolon を所有する。
任意の inheritance `coherence` は必須の general justification を所有し、task-27 の
redefinition 専用 `with` label は受理しない。

Task 29 recovery は `struct` と explicit `inherit` block 内の local member
synchronization と、境界での definition-content synchronization を使う。空または
malformed な structure pattern、field / property name、inheritance target、field /
property redefinition name、malformed member tail は、inserted raw surface placeholder が
必要な場合に `MalformedTermExpression` と `MissingTerm` を使う。member または
redefinition type の欠落は `MalformedTypeExpression` と `MissingTypeExpression` を使う。
inheritance coherence justification の欠落または malformed syntax は
`MalformedJustification` と `MissingProofStep` を使う。`coherence with ...` は
inheritance では受理せず recovered syntax として扱う。member semicolon と外側
semicolon の欠落は `MissingSemicolon` を使い、block closer 欠落は `MissingEnd` を使う。
malformed member tail は semicolon、`field`、`property`、`coherence`、`end`、次の
definition-content start、top-level item boundary、EOF まで skip してよい。frontend の
scope skeleton も nested `struct ... end` と explicit `inherit ... where ... end` range を
認識し、parse-only fixture が parsing 前に spurious unmatched `end` diagnostic を出さない
ようにする。

Task 29 tests は、structure field / property、`of` / `over` / bracket parameter、
field initializer、shorthand inheritance、`extends set` を含む explicit inheritance、
field / property redefinition、citation と proof justification を伴う coherence、
definition-local visibility wrapper、name / type / semicolon 欠落、空の explicit
`where` recovery、malformed coherence recovery、active parse-only pass/fail corpus
coverage、Chapter 5 §5.2、§5.3、§5.3.1、§5.3.2、§5.6、Appendix A.5 / A.12 への
traceability を固定する必要がある。

### Task 30: Registration と Cluster

Task 30 は Chapter 17 の registration block と definition-local registration item を
syntax-only に追加する。parser は registration-local parameter、existential /
conditional / functorial cluster registration、reduction registration、および
syntax-level correctness condition を保持する。cluster closure、reduced normal form の
推論、reducibility validation、proof obligation check は行わない。

Production inventory:

```ebnf
declaration             ::= ... | registration_block ;
definition_content      ::= ... | [ visibility ] registration_item ;

registration_block      ::= "registration" { registration_content } "end" ";" ;
registration_content    ::= registration_parameter | registration_item ;
registration_parameter  ::= "let" qualified_variable_segments
                             [ "such" condition_list ]
                             [ "by" references ] ";" ;
registration_item       ::= cluster_registration | reduction_registration ;

cluster_registration    ::= "cluster" label ":"
                             ( existential_cluster
                             | conditional_cluster
                             | functorial_cluster ) ;
existential_cluster     ::= attributed_type ";" "existence" justification ";" ;
conditional_cluster     ::= registration_adjectives "->"
                             registration_consequent ";"
                             "coherence" justification ";" ;
functorial_cluster      ::= functorial_payload "->"
                             registration_consequent ";"
                             "coherence" justification ";" ;
registration_consequent ::= registration_adjectives "for" type_expression ;
registration_adjectives ::= registration_adjective { registration_adjective } ;
registration_adjective  ::= [ "non" ] [ param_prefix ] attribute_ref_name ;
functorial_payload      ::= application_term | operator_term
                           | bracket_functor_application ;

reduction_registration  ::= "reduce" label ":" term_expression "to"
                             term_expression ";"
                             "reducibility" justification ";" ;
```

`RegistrationBlockItem` は `registration`、source order の
`RegistrationParameter` / registration-item child、任意の recovery、存在する場合の
closing `end` と semicolon を所有する。registration parameter は通常の
qualified-variable segment と condition-list surface を再利用するが、この位置で
syntax-local な `by` reference だけを受け付ける。definition-only の proof-bearing
constraint は malformed として扱う。

`ExistentialRegistration` は `cluster` keyword、label、colon、1 個の attributed
`TypeExpression`、header semicolon、`existence` の `CorrectnessCondition` を所有する。
parser は type expression が少なくとも 1 個の Chapter 17 registration adjective で
始まることを要求する。registration adjective は optional `non`、optional parameter
prefix、parenthesized argument を持たない attribute name である。type が inhabitable か
どうかの semantic check は parser の外に残る。

`ConditionalRegistration` は `->` の前の 1 個以上の registration adjective、
consequent の 1 個以上の registration adjective、`for`、target `TypeExpression`、
header semicolon、`coherence` の `CorrectnessCondition` を所有する。`->` の前に
antecedent がない場合は malformed とし、antecedent slot に `MissingTypeExpression`
placeholder を入れた conditional registration として recover する。

`FunctorialRegistration` は `->` の前に syntactically unambiguous な term payload を
所有する。受理するのは application term、operator expression surface、または bracket
functor application である。bare identifier / reference、numeral、`it`、choice term、
set enumeration、structure constructor、selector/update chain、`qua` expression は
functorial registration payload として受理しない。nullary functorial registration は、
syntax だけでは single-adjective conditional registration と判別できないため deferred
のままである。

`ReductionRegistration` は `reduce`、label、colon、left `TermExpression`、`to`、
right `TermExpression`、header semicolon、`reducibility` の
`CorrectnessCondition` を所有する。reducibility proof replay と normal form equality は
semantic / proof work に残る。

definition-local な `public cluster`、`private cluster`、`public reduce`、
`private reduce` は、concrete registration item を既存の `VisibleItem` /
`VisibilityMarker` wrapper で包む。top-level の bare `cluster` / `reduce` item は引き続き
invalid であり、top-level registration は `registration ... end;` block の中に置く。

Task 30 recovery は registration-content synchronization を使う。malformed
registration parameter、label / colon 欠落、antecedent / consequent adjective 欠落、
unsupported functorial payload、argument-bearing registration adjective、target type
欠落、correctness justification 欠落、header semicolon 欠落、registration block `end`
欠落は、可能な限り後続の registration content を保持して recover する。frontend scope
skeleton は `registration ... end` を認識し、target type の `for set` や `for T` を
binder candidate として扱わないため、active parse-only case が spurious lexical scope
diagnostic を出さない。

Task 30 tests は、registration-local `let`、parameterized registered type を含む
existential cluster、conditional cluster、functorial application/operator/bracket
cluster、compound reduction、proof / citation correctness condition、registration item の
definition-local visibility wrapper、malformed な definition-only `let T be type`
parameter、label 欠落、antecedent 欠落、unsupported functorial payload、argument-bearing
registration adjective、reducibility justification 欠落、registration block end 欠落、
active parse-only pass/fail corpus coverage、Chapter 17 §17.2〜17.6 と、annotation
prefix を S-016/parser task 35 で別に覆う Appendix A.17 registration/cluster/reduction
production への traceability を固定する必要がある。

### Task 31: テンプレート

Task 31 は Chapter 18 の template syntax を parser-visible にする。
template-shaped definition block は、先頭の `let` declaration と、theorem item、
registration item、parameterized predicate / functor pattern などの template-only
content から検出する。そのような block では、先頭の `let` declaration は
`TemplateParameter` node になる。parser は ordinary value parameter、optional
`extends` を持つ `type` parameter、`pred(...)` parameter、`func(...) -> ...`
parameter、通常の definition parameter と同じ syntax-level constraint、proof、
`by` tail を保存する。
この section は、template predicate argument、reference template argument、
template-ambiguous definition content に関する以前の task-local deferred note を
supersede する。それらの古い note は task 31 が入る前の状態を記録している。

pattern 側の bracket は call-site actual と分けて表現する。predicate / functor
definition pattern は `TemplateLoci` / `TemplateLocus` を所有し、predicate head、
template functor application、local / qualified reference、grouped-reference item は
`TemplateArguments` / `TemplateArgument` を所有する。template argument は
parser-visible な type expression、term expression、identifier actual、radix-type
target を持つ `qua` argument を受理する。attribute-bearing `qua` target は malformed
のまま、type diagnostic 付きで recover する。

active lexical environment がまだ symbol を export していない場合でも、parser は
`x matches[T]` のような template-local identifier predicate head を受け付ける。
`pick[T](x)` は ordinary application argument の前に template functor application term
として parse する。`Ref[T]`、`mml.foo.Th[T]`、`mml.foo.{G[T]}` のような reference
citation は malformed justification tail ではなく concrete reference node である。
`nest` は task 17 の `ComputationOption` surface で既に表現されており、task 31 の
traceability として残る。

Task 31 は template の instantiate、template parameter binding、predicate / functor
actual kind の検証、template type constraint の検査は行わない。以前 inactive だった
template pass/fail seed fixture は active parse-only coverage になり、chained `iff`
failure は template predicate argument が concrete syntax になった後も formula fixity
diagnostic として残る。

### Task 32: algorithm ブロック・代入・宣言・claim

Task 32 は Chapter 20 の contract を含まない algorithm / claim syntax を
parser-visible にする。definition block は `algorithm` content を受け付け、
`public` / `private` visibility wrapper は `VisibleItem` を通じて表現する。
algorithm definition は source name、任意の identifier-only schema suffix
（`TemplateLoci` / `TemplateLocus`）、`AlgorithmParameters` list、任意の
return `TypeExpression`、`AlgorithmBody`、末尾 semicolon を所有する。この schema
suffix は call-site の `TemplateArguments` を再利用しない。

実装済みの body subset は `do ... end` と、その中の
`AlgorithmStatementList` である。statement list は variable declaration、
assignment、snapshot、return を含む。`VariableDeclaration` は `var`、`const`、
`ghost var`、`ghost const` を表し、`VariableBinding` child、任意の shared
`as TypeExpression`、任意の syntax-level justification を持つ。
`AssignmentStatement` は通常 assignment と `ghost` assignment を syntactic
`Lvalue` へ表現し、selector / namespace role を解決せず dotted target を保持する。
`SnapshotStatement` と `ReturnStatement` はそれぞれ `snapshot` と `return` を覆い、
return は任意で term と syntax-level justification を所有できる。

Top-level `claim name do ... end;` は `ClaimBlockItem` で表され、theorem / lemma item
を直接または task 35 の annotation wrapper 経由で含められる。algorithm header /
loop verification clause は task 34 で実装済みである。statement-level、algorithm-body、
definition-content、registration-content、claim-local annotation prefix は task 35 で
実装済みである。frontend scope skeleton は algorithm header を単一
lexical block として認識し、`ghost target := term;` を assignment として扱うため、
active source-level parse-only fixture は frontend recovery diagnostic なしで task 32
syntax を行使できる。

### Task 33: algorithm control flow

Task 33 は Chapter 20 の control-flow subset を parser-visible にする。

```ebnf
if_stmt ::= "if" formula "do" algo_statement_list if_tail ;
if_tail ::= "end" ";"
          | "else" if_stmt
          | "else" algo_statement_list "end" ";" ;

while_stmt ::= "while" formula "do" algo_statement_list "end" ";" ;

for_range_stmt ::= "for" identifier "=" term_expression
                   ( "to" | "downto" ) term_expression
                   [ "step" term_expression ]
                   "do" algo_statement_list "end" ";" ;

for_collection_stmt ::= "for" identifier "in" term_expression
                        [ "processed" identifier ] "do"
                        algo_statement_list
                        "end" ";" ;

match_stmt ::= "match" term_expression "do"
               match_case+
               ( "otherwise" algo_statement_list "end" ";"
               | "exhaustive" [ justification ] ";" )
               "end" ";" ;
match_case ::= "case" term_expression "do" algo_statement_list "end" ";" ;

break_stmt    ::= "break" ";" ;
continue_stmt ::= "continue" ";" ;
```

`IfStatement`、`WhileStatement`、`ForRangeStatement`、
`ForCollectionStatement`、`MatchStatement`、`MatchCase`、`MatchEnding`、
`BreakStatement`、`ContinueStatement` は control-flow の source shape を保持する。
branch-local `AlgorithmStatementList` と malformed statement tail recovery は周囲の
control-flow boundary（`else`、`case`、`otherwise`、`exhaustive`、`end`）で停止し、
semicolon 欠落 / inner `end` 欠落の recovery が次の branch を飲み込まず所有元の構文へ
戻れるようにする。

Concrete loop verification clause と `assert` statement は task 34 で実装済みである。
task 33 recovery は malformed control-flow statement と statement-list boundary に
限定される。task 35 は algorithm statement 前の annotation prefix を扱う。

frontend scope skeleton は、completed match `case` branch の token shape である
`end` または `end;` の直後に現れる algorithm-scope `otherwise` に対して
conservative な `Do` frame を開き、`match ... otherwise ... end; end;` を対応する
block shape として扱う。definition-side やその他 non-algorithm の通常の `otherwise`
clause は scope frame を開かない。

### Task 34: algorithm verification clause

Task 34 は Chapter 20 の verification clause syntax を parser-visible にする。

```ebnf
algorithm_def ::= [ "terminating" ] "algorithm" identifier
                  [ template_loci ] algorithm_parameters [ "->" type_expression ]
                  [ requires_clause ] [ ensures_clause ] [ decreasing_clause ]
                  algorithm_body ";" ;
requires_clause  ::= "requires" formula ;
ensures_clause   ::= "ensures" formula ;
decreasing_clause ::= "decreasing" term_list ;

while_stmt ::= "while" formula "do"
                 { loop_invariant_clause | loop_decreasing_clause }
                 algo_statement_list
               "end" ";" ;
for_range_stmt ::= "for" identifier "=" term_expression
                   ( "to" | "downto" ) term_expression
                   [ "step" term_expression ]
                   "do" { loop_invariant_clause }
                   algo_statement_list "end" ";" ;
for_collection_stmt ::= "for" identifier "in" term_expression
                        [ "processed" identifier ] "do"
                        { loop_invariant_clause }
                        algo_statement_list "end" ";" ;
loop_invariant_clause  ::= "invariant" formula [ justification ] ";" ;
loop_decreasing_clause ::= "decreasing" term_list [ justification ] ";" ;
assert_stmt ::= "assert" formula [ justification ] ";" ;
term_list ::= term_expression { "," term_expression } ;
```

`AlgorithmTerminationClause`、`AlgorithmRequiresClause`、
`AlgorithmEnsuresClause`、`AlgorithmDecreasingClause`、`LoopInvariantClause`、
`LoopDecreasingClause`、`AssertStatement`、`TermList` は、verification condition の生成や
termination check を行わず、clause の source shape を保持する。header clause は
`requires`、`ensures`、header `decreasing` の固定順で、それぞれ高々 1 回受理する。
重複または順序違反の header verification keyword は診断し、algorithm body boundary まで
recover する。`terminating algorithm` は直接の definition content と `public` /
`private` visible definition wrapper の両方で受理する。

Loop verification clause は loop `do` 直後の leading clause block としてのみ受理する。
`while` は `invariant` と `decreasing` の両方を受け付ける。range / collection `for`
loop は `invariant` のみ受け付けるため、`for ... do decreasing ...;` は syntax recovery
case のままである。通常の body statement の後に現れる `invariant` または `decreasing`
は misplaced algorithm statement として診断し、clause semicolon で recover する。
空または dangling comma を持つ `decreasing` term list は、`TermList` 内に
`MissingTerm` recovery を挿入する。

### Task 35: annotation

Task 35 は Appendix A.21 の annotation syntax を parser-visible にする。

```ebnf
annotation_prefix      ::= library_annotation | fixed_annotation | generic_annotation ;
library_annotation     ::= "@[" annotation_label { "," annotation_label } "]" ;
annotation_label       ::= identifier [ annotation_args ] ;
fixed_annotation       ::= "@latex" "(" string_literal ")"
                         | "@suppress" "(" identifier ")"
                         | "@proof_hint" "(" proof_hint_option { "," proof_hint_option } ")"
                         | "@show_thesis"
                         | "@show_resolution" ;
generic_annotation     ::= "@" identifier [ annotation_args ] ;
annotation_args        ::= "(" annotation_arg { "," annotation_arg } ")" ;
annotation_arg         ::= identifier | numeral | string_literal ;
proof_hint_option      ::= ( "max_axioms" | "timeout" ) ":" numeral
                         | "solver" ":" ( "vampire" | "e" | "cvc5" | "z3" | "auto" ) ;
standalone_diagnostic  ::= ( "@show_type" | "@eval" ) "(" term_expression ")" ;
```

Attachable prefix は `Annotation`、`LibraryAnnotation`、`AnnotationLabelList`、
`AnnotationLabel`、`AnnotationArgumentList`、`AnnotationArgument`、
`ProofHintOptionList`、`ProofHintOption` node で表す。parser は visible / bare
theorem・lemma item、top-level definition / registration / claim block item、
definition content、registration content、ordinary proof statement、algorithm statement、
claim-local theorem・lemma item の前でこれらを受け付ける。
Appendix A.12 は import/export prelude item を `annotated_declaration` の外側に置くため、
それらは annotation host ではない。host が既に concrete syntax node を持つ場合、task 35 は
必要に応じて `AnnotatedStatement`、`AnnotatedAlgorithmStatement`、
`AnnotatedDefinitionContent`、`AnnotatedRegistrationContent` で host を包む。

`@show_type(...)` と `@eval(...)` は standalone diagnostic annotation である。
これらは `StandaloneDiagnosticAnnotation` node を生成し、次の statement には attach
しない。`@show_thesis` と `@show_resolution` は argumentless であり、どちらかに argument
list が付いた場合は source-preserving recovery としてだけ parse し、`MalformedAnnotation`
を報告する。`@latex` は string literal argument、`@suppress` は identifier argument、
`@proof_hint` は固定された `max_axioms`、`timeout`、`solver` option name と Appendix A.21
の value form を要求する。generic annotation name と registry-specific argument meaning は
semantic または tooling concern に残る。

Malformed annotation argument、proof-hint option、empty slot、unmatched annotation
delimiter は `MalformedAnnotation` で recover する。recovery は source-preserving で、
bad annotation prefix が後続の concrete host を飲み込まないように、周囲の item、
statement、block boundary で同期する。

## 公開 enum の互換性

`ParserTokenKind` は downstream crate 向けに `#[non_exhaustive]` とする。parser-facing
lexing context が追加の token class を得るにつれて、parser token transfer vocabulary
は成長し得るため、downstream consumer は wildcard fallback arm を持つ必要がある。
`mizar-parser` 内部の match は exhaustive のままにし、新しい token kind が追加された
ときに parser 側の更新がローカルに強制されるようにする。
