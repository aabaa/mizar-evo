# mizar-parser: Grammar

> 正本は英語です。英語版: [../en/grammar.md](../en/grammar.md)。

状態: task 9 までの module skeleton、top-level placeholder dispatch、concrete
import item、export item、visibility wrapper、reserve-hosted type expression、および
reserve-hosted primary term は実装済み。残りの具体的な非 module item 文法は計画中。

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
  `TermExpression` argument、bracket nested type argument、bracket `qua_arg` placeholder を
  保持する。その他の non-module item grammar は placeholder のままである。

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
に限る。local statement-level `reserve` behavior は後続 statement task が所有する。
well-formed `ReserveItem` は `reserve` token、1 個の `ReserveSegment`、終端 semicolon
を所有する。`ReserveSegment` は source order の identifier / comma token、`for`
token、`TypeExpression` を所有する。

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
bracket argument が代わりに
`qua_arg` に一致する場合、task 8 は identifier と任意の `qua` radix-type tail token を所有する
一時的な `TermPlaceholder` child として保持する。`]` 欠落は `MalformedTypeExpression` と
`UnmatchedOpeningDelimiter` recovery として保持する。task 8 の `TermPlaceholder` node は
task 9 以降は bracket `qua_arg` argument の浅い token owner にすぎず、term classification、
operator fact、name resolution、overload selection を encode してはならない。

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
`user_symbol term_list user_symbol` の active user-symbol delimiter pair は、まだ
`ParserInputs` に含まれていない bracket-pair metadata を必要とするため、その
active-operator extension は parser task 12 が所有する。

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

## 公開 enum の互換性

`ParserTokenKind` は downstream crate 向けに `#[non_exhaustive]` とする。parser-facing
lexing context が追加の token class を得るにつれて、parser token transfer vocabulary
は成長し得るため、downstream consumer は wildcard fallback arm を持つ必要がある。
`mizar-parser` 内部の match は exhaustive のままにし、新しい token kind が追加された
ときに parser 側の更新がローカルに強制されるようにする。
