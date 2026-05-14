# Architecture: Source and Frontend

> Canonical language: English. English canonical version: [../en/source_and_frontend.md](../en/source_and_frontend.md).

## 目的

`.miz` source file を読み込み、`SourceUnit -> PreprocessedSource -> TokenStream -> SurfaceAst` へ変換する frontend の設計を定義する。
frontend は syntax を作るが、semantic name resolution や type checking は行わない。

## 対象フェーズ

| Phase | Output | Responsibility |
|---|---|---|
| 1. Source Loading / Preprocessing | `SourceUnit`, `PreprocessedSource` | file validation, line map, comments/doc comments, import pre-scan |
| 2. Lexing | `TokenStream` | active lexicon に基づく tokenization |
| 3. Parsing | `SurfaceAst` | source-shaped AST, annotation attachment, syntax recovery |

## 設計判断

### Import pre-scan は浅く行う

import は active lexicon に影響するため、tokenization 前に最低限の import path を読む必要がある。
ただし package/module の存在確認、visibility、export check は module resolver の責務とする。

### Lexing は active lexicon を使う

lexer は reserved words、reserved symbols、active user symbols、identifier、numeral を longest-match で token 化する。
identifier 形状の token が active user symbol と一致する場合は symbol token とする。

### `.` の解釈は分担する

- `.{`, `.*`, `.=`, `...` は lexer が compound reserved token として扱う。
- `.` を含む user symbol は active lexicon と longest-match により lexer が扱う。
- selector access / namespace separator の最終判断は parser + resolver が行う。
- namespace path と local variable の shadowing は resolver が判断する。

### String literal は lexer が完全 token 化する

parser が lexer cursor を動かす方式にはしない。
lexer は grammar-derived `StringPositionRecognizer` を使い、string が許される位置だけで `StringLiteral` token を生成する。
それ以外の場所では quote は user symbol の一部として扱う。

### Annotation は parser が所有する

preprocessing は annotation を別 metadata channel に切り出さない。
annotation token は lexical text に残し、parser が `SurfaceAst` の annotation node として attach する。

## 主なデータ構造

- `SourceUnit`: source text, file path, module path, source hash, line map
- `PreprocessedSource`: lexical text, comments, doc comments, import stubs, lexical source map
- `ImportStub`: shallow import pre-scan の結果
- `ActiveLexiconSeed`: reserved tables と imported user symbols
- `TokenStream`: source span 付き token 列
- `SurfaceAst`: source-shaped syntax tree

## Error Recovery

lexer は malformed span に `TokenKind::Error` を出し、可能なら tokenization を継続する。
parser は `;`, `end`, top-level item keyword などで同期し、error node を作る。
semantic fact を捏造せず、後続 phase が明示的に skip / reject できる形にする。

## Incrementality

- comment-only edit は documentation metadata を無効化するが、lexical text が同じなら semantic output を再利用できる可能性がある。
- import edit は active lexicon, token stream, AST, semantic layers を無効化する。
- dependency export の変更は local source が同じでも tokenization を無効化し得る。
- parser version と language edition は AST cache key に含める。
