# mizar-parser: Grammar

> 正本は英語です。英語版: [../en/grammar.md](../en/grammar.md)。

状態: task 7 までの module skeleton、top-level placeholder dispatch、concrete
import item、export item、および visibility wrapper は実装済み。具体的な非 module
item 文法は計画中。

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

## 公開 enum の互換性

`ParserTokenKind` は downstream crate 向けに `#[non_exhaustive]` とする。parser-facing
lexing context が追加の token class を得るにつれて、parser token transfer vocabulary
は成長し得るため、downstream consumer は wildcard fallback arm を持つ必要がある。
`mizar-parser` 内部の match は exhaustive のままにし、新しい token kind が追加された
ときに parser 側の更新がローカルに強制されるようにする。
