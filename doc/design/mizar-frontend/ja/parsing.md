# モジュール: parsing

> 正本は英語です。英語版: [../en/parsing.md](../en/parsing.md)。

状態: planned。

## 目的

このモジュールはフロントエンドパイプラインの Step 5（パーサー呼び出し）を実装する。`mizar-parser` を呼び出して `TokenStream` を `mizar-syntax::SurfaceAst` へ変え、アクティブ字句環境から導出したパーサー入力（演算子の結合性、文字列必須の文法文脈）を供給し、統合がパーサー駆動の曖昧性解消を用いる場合にはパーサーが要求する字句文脈を Step 4 へ戻す。

`SurfaceAst` ノード定義（`mizar-syntax` にある）も、文法・Pratt 優先順位・回復・注釈付与ロジック（`mizar-parser` にある）も所有しない。crate 境界で入出力を適合させるだけである。

[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Step 5: Parse」「Syntax and Parser Are Separate Crate Boundaries」「Annotations Are Parser-Owned Syntax」を参照。

## 公開 API

```rust
pub struct ParseRequest<'a> {
    pub tokens: &'a TokenStream,
    pub parser_inputs: ParserInputs,
}

pub struct ParserInputs {
    pub edition: Edition,
    pub operator_fixity: OperatorFixityTable,
    pub string_required_positions: StringRequiredContext,
}

pub struct ParseOutput {
    pub ast: Option<SurfaceAst>,
    pub diagnostics: Vec<SyntaxDiagnostic>,
}

pub fn parse(request: ParseRequest<'_>) -> ParseOutput;
```

`SurfaceAst` と `SyntaxDiagnostic` は `mizar-syntax` が所有する。`parse` は `mizar-parser` のエントリポイントに委譲する。`ParserInputs` はフロントエンドがアクティブ字句環境とエディションから導出する。`operator_fixity` と `string_required_positions` はパーサーが必要とする唯一の字句関連信号であり、字句解析器側の狭い `ParserLexContext` 契約と対応する。

`ast = None` は、パーサーが以降のフェーズに十分な構造を回復できなかったことを意味する。字句診断と構文診断は依然として返される。

## 依存関係

- 内部: `lexing`（`TokenStream` を提供）、`lexical_env`（`ParserInputs` 用のデータを提供）、`orchestration`（`ParseOutput` を消費）。
- 外部: `mizar-parser`（文法エントリポイント、Pratt 解析、回復、注釈／ドキュメントコメント付与）、`mizar-syntax`（`SurfaceAst`、`SyntaxDiagnostic`）、`mizar-session`（`Edition`、AST ノード内に運ばれる `SourceRange`）。

このモジュールは統制層が消費して `FrontendOutput` を組み立てる。

## データ構造

### パーサー入力

`ParserInputs` は文法に影響する設定をフロントエンドが組み立てた束である。言語エディション、アクティブ字句環境から導出した演算子結合性、文字列必須引数位置のレジストリである。字句解析器の `ParserLexContext` に対応するパーサー向けのものであり、任意のスコープや解決器の状態を運ぶことは決してない。

### Surface AST の受け渡し

`SurfaceAst`（`mizar-syntax` 由来）はソース形である。ソース順と `SourceRange` を保持し、注釈を構文ノードとして付与し、ドキュメントコメントを近接するドキュメント化可能項目に付与し、回復ノードを明示的に印付ける。フロントエンドはそれをそのまま通す。ノードの書き換え・剪定・解釈は行わない。

## アルゴリズム / ロジック

### トークンストリームの構文解析

1. アクティブ字句環境とエディションから `ParserInputs` を構築する。
2. `TokenStream` と入力を与えて `mizar-parser` のエントリポイントを呼ぶ。
3. パーサーはモジュール、定義、登録、文、項、論理式、定理、証明、アルゴリズムを解析し、項・論理式の式に Pratt／優先順位解析を用い、注釈引数リストを解析して注釈を付与し、ドキュメントコメントを近接するドキュメント化可能項目に付与し、同期点（`;`、`end`、トップレベル項目キーワード、EOF）で回復する。
4. `SurfaceAst`（回復が失敗した場合は `None`）と構文診断を返す。

統合がパーサー駆動の曖昧性解消を用いる場合、パーサーは狭い文脈オブジェクトを通じて文字列必須位置を Step 4 へ供給する。パーサーが字句解析器のカーソルを直接駆動することは決してない。

## エラー処理

構文診断は `mizar-parser` から来る（予期しないトークン、不整合な区切り、`end` の欠落、文字列必須位置での文字列リテラルトークンの欠落、不正な注釈引数リスト）。フロントエンドは構文エラーカテゴリを追加しない。使用可能な木を回復できない構文解析は、後のフェーズが優雅にスキップ／縮退できるよう、診断とともに `ast = None` を返す。返された `SurfaceAst` 内の回復ノードはパーサーが印付ける。フロントエンドはその印を保持する。

## テスト

主要シナリオ:

- 整形式のトークンストリームが、ソース順と `SourceRange` を保持した `SurfaceAst` へ解析される。
- アクティブ字句環境からの演算子結合性が、ユーザー定義中置演算子に対する正しい Pratt 優先順位を駆動する。
- 注釈引数位置の `StringLiteral` トークンが注釈ノードへ解析され、文字列必須位置での文字列リテラルの欠落が期待される構文診断を生む。
- `end` の欠落が同期点で回復し、明示的なエラーノードを生み、`ast` は `Some` のままである。
- 回復不能なトークンストリームが診断とともに `ast = None` を返す。
- ドキュメントコメントは可能なら後続のドキュメント化可能項目に付与され、それ以外ではソース位置の近くに保たれる。

## 制約と前提

- このモジュールは AST ノード定義や文法／回復ロジックを所有しない。
- パーサーは既にトークン化された `StringLiteral` トークンを消費し、字句解析器のカーソルを直接駆動しない。
- `ParserInputs` は文法に影響する設定のみを運び、解決器の状態は運ばない。
- `SurfaceAst` は回復ノードを含みうる。後のフェーズはそれらを明示的に許容または拒否しなければならない。
- `SurfaceAst` キャッシュキーはトークンストリームハッシュ、パーサーバージョン、エディションである。
