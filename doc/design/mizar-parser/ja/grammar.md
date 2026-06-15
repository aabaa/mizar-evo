# mizar-parser: Grammar

> 正本は英語です。英語版: [../en/grammar.md](../en/grammar.md)。

状態: module skeleton、top-level placeholder dispatch、concrete import item、
export item、visibility wrapper、reserve-hosted type expression、set comprehension
を含む task 15 term surface、および task 14 formula surface は実装済み。残りの
具体的な statement / proof item 文法は計画中。

## 目的

このモジュールは、Mizar Evo のパーサー入口とモジュール／項目文法を定義する。

## 責務

- parser-facing token transfer object を消費し、`mizar-syntax::SurfaceAst` を生成する。
- モジュール、import、定義、registration、文、証明、アルゴリズム、アノテーション、項、論理式を構文解析する。
- 名前解決、型推論、オーバーロード選択、証明義務生成を行わず、構文解析を意味論から分離する。

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

qualified_symbol  ::= { namespace_segment "." } user_symbol ;
namespace_segment ::= identifier ;
```

`module_path` は第 12 章の import / export path 形である。共有 path helper の中で
`relative_prefix` を受け入れるのは `module_path` だけであり、citation / reference
prefix 用の `namespace_path` は相対 import prefix を受け入れてはならない。
`qualified_symbol` は、active lexicon から渡された parser-facing `user_symbol` token
で終わり、先行する namespace segment は identifier segment として表現する。

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

`@[` で始まる連続した library annotation prefix は、認識済み
annotated-declaration start が後続する場合、同じ placeholder に保持する。annotation
prefix により `import` や `export` が annotation 可能になるわけではないため、
import/export prelude item の前に annotation prefix がある場合は、その statement
全体とともに予期しない top-level input として回復する。malformed annotation parsing
と concrete annotation node は annotation grammar task まで延期する。セミコロン型
placeholder は nested `proof ... end` と文脈付き algorithm/proof block をまたいで
scan するため、proof body 内のセミコロンで theorem / lemma item を分割しない。
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
だけ表現する。それらの concrete item grammar がまだ延期中である間、parser は
`VisibleItem` wrapper を送出する。child は source order で、既に skip 済みの library
annotation prefix token があればそれら、`private` または `public` token を包む
`VisibilityMarker` 1 個、後続 target `PlaceholderItem` である。合法 target start は
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
attribute_ref_name ::= qualified_symbol ;
mode_ref_name     ::= qualified_symbol ;
struct_ref_name   ::= qualified_symbol ;
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
含むかは判断しない。user-symbol-shaped reference の列を複数の方法で分割できる場合、
parser は右端に残る syntactic type-head candidate を `TypeHead` として確保し、それより
前の reference を attribute として扱う。これは syntax-only の boundary rule であり、
semantic classification ではない。

`AttributeRef` は任意の `non`、任意の parameter prefix、`QualifiedSymbol`、任意の
parenthesized term argument を保持する。`TypeHead` は builtin `object` / `set` token
または `QualifiedSymbol` head と任意の `TypeArguments` を保持する。task 8 は、
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
各 entry は source spelling、fixity kind、precedence、infix の場合の associativity を記録する。
parser はこの table を使って source token を `PrefixExpression`、`PostfixExpression`,
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
predicate_head               ::= predicate_symbol ;
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
parenthesized formula、`thesis`、`contradiction` は、後続 task の host が placeholder として
保持する場合を除き task 14 の責務である。現在 frontend から到達できる formula host は
`theorem label: formula;` または `lemma label: formula;` 形の theorem/lemma placeholder
item である。item 自体は task 22 が theorem/proof item node を追加するまで
`PlaceholderItem` のままだが、formula payload は task-13 coverage のため concrete formula
child として parse する。それ以外の theorem/lemma placeholder tail は既存の
token-preserving placeholder behavior を保ち、この task では theorem status、proof nesting、
label、validity を固定しない。

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
chain は user predicate chain として表現せず syntax error のままとする。template predicate
argument は `template_args` がまだ表現されていないため task 31 / S-016 に延期する。

theorem/lemma placeholder formula host は正確に `label: formula;` だけである。
`label: x = y by A;` や `label: x = y proof ... end;` のように parse 可能な atomic formula
prefix の後に theorem justification / proof tail が続く場合は、task 22 が theorem/proof item を
所有するまで legacy token-preserving `PlaceholderItem` behavior を保つ。predicate-chain
segment の right term 欠落は `MalformedTermExpression` を報告し、`MissingTerm` を挿入する。

Task 13 の test は built-in `in`、`=`、`<>` atom、attribute-only `non` chain を含む generic
`is` assertion、inline predicate call shape、active-lexicon user predicate segment、
theorem-placeholder formula host、semantic classification を必要としない malformed atomic
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

theorem/lemma placeholder formula host は atomic formula から task-14 formula
全体へ広がるが、引き続き exact である。`label: formula;` だけが
`FormulaExpression` を送出する。`by` や `proof` の theorem justification /
proof tail が続く prefix は、task 22 が theorem/proof item node を所有するまで
legacy token-preserving `PlaceholderItem` payload のままにする。template
predicate argument は task 31 / S-016 に deferred のままである。formula を
term syntax 内へ埋め込む Fraenkel / set-builder term は task 15 で実装済みである。

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
theorem-placeholder formula hosting、および missing-formula recovery を固定しなければならない。

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
template predicate argument は task 31 / S-016 まで deferred のままである。
theorem / lemma formula host がそのような deferred predicate template surface を含む
comprehension payload を持つ場合、host は task-15 syntax を部分 parse せず
legacy placeholder のままにする。
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

`StatementItem` は、task 22 で theorem/proof block item が入る前に active parse-only
fixture が statement node を検査できるようにする parser-owned の一時 item host
である。後続の proof / block parser は、同じ concrete statement node を
`StatementItem` wrapper なしで直接所有してよい。statement-level annotation はこの
task では parse せず、annotation 付き statement source は task 35 / S-016 まで
legacy placeholder または recovery input のまま残す。canonical Chapter 4 specification は
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

task-17 justification surface はまだ deferred である。semicolon 前に top-level `by`
tail を持つ `let` statement は、部分的な concrete `LetStatement` ではなく legacy
placeholder のまま残す。task 17 がこの境界を concrete な justification-aware 形状に
置き換える。この task-16 parser は label、reference、witness leakage、type
well-formedness、proof obligation も検証しない。

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

## 公開 enum の互換性

`ParserTokenKind` は downstream crate 向けに `#[non_exhaustive]` とする。parser-facing
lexing context が追加の token class を得るにつれて、parser token transfer vocabulary
は成長し得るため、downstream consumer は wildcard fallback arm を持つ必要がある。
`mizar-parser` 内部の match は exhaustive のままにし、新しい token kind が追加された
ときに parser 側の更新がローカルに強制されるようにする。
