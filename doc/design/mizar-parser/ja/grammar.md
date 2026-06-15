# mizar-parser: Grammar

> 正本は英語です。英語版: [../en/grammar.md](../en/grammar.md)。

状態: task 5 までの module skeleton と top-level placeholder dispatch は実装済み。
具体的な import / export / item 文法は計画中。

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
`ItemList` を 1 つ持たせる。`ItemList` には、認識した top-level start に対応する
source order の `PlaceholderItem` node と、skip された top-level input の
`SkippedToken` recovery node が入る。認識する start は `import`、`export`、
`definition`、`reserve`、`registration`、`claim`、`theorem`、`lemma`、
theorem-status prefix の `open` / `assumed` / `conditional`、visibility prefix の
`private` / `public`、notation start の `infix_operator`、`prefix_operator`、
`postfix_operator`、`synonym`、`antonym` である。

`@[` で始まる連続した library annotation prefix は、認識済み top-level start が
後続する場合、同じ placeholder に保持する。malformed annotation parsing と
concrete annotation node は annotation grammar task まで延期する。セミコロン型
placeholder は nested `proof ... end` と文脈付き algorithm/proof block をまたいで
scan するため、proof body 内のセミコロンで theorem / lemma item を分割しない。
式レベルの `if` や `otherwise` のような文脈依存 formula keyword は placeholder の
block depth に影響しない。

この task は import alias、export path、theorem formula、visibility semantics、
item validity、symbol identity を parse しない。`import` と `export` は task 6 と
7 が concrete item node に置き換えるまで placeholder である。認識可能な top-level
item start を含まない token stream は module skeleton に関して task 3 の互換
behavior を保つ。つまり token は保持され、item list は空になる。このような stream
が diagnostic-free のままになるのは、legacy minimal token-stream corpus case のように、
先行する recovery pass でも指摘がない場合に限られる。最初の認識済み item keyword が先行する recovery
block opener の内側にある合成 block-recovery stream も、この互換 behavior を保つ。
一方、theorem item の前に裸の reserved word があるような通常の malformed prefix は
`UnexpectedTopLevelToken` recovery を生成する。

## 公開 enum の互換性

`ParserTokenKind` は downstream crate 向けに `#[non_exhaustive]` とする。parser-facing
lexing context が追加の token class を得るにつれて、parser token transfer vocabulary
は成長し得るため、downstream consumer は wildcard fallback arm を持つ必要がある。
`mizar-parser` 内部の match は exhaustive のままにし、新しい token kind が追加された
ときに parser 側の更新がローカルに強制されるようにする。
