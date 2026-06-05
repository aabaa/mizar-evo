# モジュール: parsing

> 正本は英語です。英語版: [../en/parsing.md](../en/parsing.md)。

状態: パーサー入力の組み立てとスタブ parser seam は実装済み。実
`mizar-parser` seam は引き続き保留中。

## 目的

このモジュールは、フロントエンドパイプラインの Step 5（パーサー呼び出し）を実装する。parser seam を通じて `TokenStream` を AST へ変え、アクティブ字句環境から導出したパーサー入力（演算子の結合性、文字列必須の文法文脈）を供給し、統合がパーサー駆動の曖昧性解消を用いる場合には、パーサーが要求する字句文脈を Step 4 へ戻す。

`SurfaceAst` ノード定義（実 parser seam では `mizar-syntax` にある）も、文法・Pratt 優先順位・回復・注釈付与のロジック（実 parser seam では `mizar-parser` にある）も所有しない。crate 境界で入出力を適合させるだけである。

[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Step 5: Parse」「Syntax and Parser Are Separate Crate Boundaries」「Annotations Are Parser-Owned Syntax」を参照。

## 公開 API

```rust
pub struct ParseRequest<'a> {
    pub tokens: &'a TokenStream,
    pub parser_inputs: ParserInputs,
}

impl<'a> ParseRequest<'a> {
    pub fn new(tokens: &'a TokenStream, parser_inputs: ParserInputs) -> Self;
}

pub struct ParserInputs {
    pub edition: Edition,
    pub operator_fixity: OperatorFixityTable,
    pub string_required_positions: StringRequiredContext,
}

impl ParserInputs {
    pub fn new(
        edition: Edition,
        operator_fixity: OperatorFixityTable,
        string_required_positions: StringRequiredContext,
    ) -> Self;

    pub fn from_active_environment(
        edition: Edition,
        environment: &ActiveLexicalEnvironment,
    ) -> Self;
}

pub struct OperatorFixityTable {
    pub entries: Vec<OperatorFixityEntry>,
}

impl OperatorFixityTable {
    pub fn empty() -> Self;
    pub fn is_empty(&self) -> bool;
}

pub struct OperatorFixityEntry {
    pub symbol_id: SymbolId,
    pub spelling: Arc<str>,
    pub precedence: u16,
    pub associativity: OperatorAssociativity,
}

pub enum OperatorAssociativity {
    Left,
    Right,
    NonAssociative,
}

pub enum StringRequiredContext {
    None,
    UniformForTest,
}

impl StringRequiredContext {
    pub fn parser_lex_context(self) -> ParserLexContext;
}

pub trait ParserSeam {
    type Ast;
    type Diagnostic;

    fn parse(&self, request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic>;
}

pub struct ParseOutput<A, D> {
    pub ast: Option<A>,
    pub diagnostics: Vec<D>,
}

impl<A, D> ParseOutput<A, D> {
    pub fn new(ast: Option<A>, diagnostics: Vec<D>) -> Self;
}

pub struct StubParserSeam;
```

`SurfaceAst` と `SyntaxDiagnostic` は `mizar-syntax` が所有する。`mizar-parser` が用意されたら、実 parser seam がそのエントリポイントへ委譲する。それまでは `StubParserSeam` が `ast = None` と空の診断リストを返し、未実装の crate への必須依存を導入しない。`ParserInputs` は、Step 3 の後にフロントエンドがアクティブ字句環境とエディションから導出する。トップレベルのコーディネータの呼び出し側は、これを渡さない。

`operator_fixity` は、依存字句サマリが公開しているデータからのみ埋める。現在のサマリの形がまだ fixity を公開していない場合、スタブ経路では `OperatorFixityTable { entries: Vec::new() }` を使い、実 Pratt/fixity テストは `mizar-parser` / `mizar-syntax` が利用可能になってから着手する。`StringRequiredContext::None` は通常の source-to-token 基盤モードであり、`UniformForTest` は、字句解析器全体を意図的に `ParserLexContext::string_required()` で実行する有界なテスト専用である。位置別の文字列必須スパンとパーサー駆動の記号種別フィルタは、この初期型では表現しない。実ソース入力で文法位置の文字列リテラルが必要になる前に、パーサー支援字句解析の契約で追加する。

`ast = None` は、実 parser seam では、パーサーが以降のフェーズに十分な構造を回復できなかったことを意味する。スタブの seam では、これは期待されるプレースホルダ結果である。字句診断と構文診断は依然として返される。

## 依存関係

- 内部: `lexing`（`TokenStream` を提供）、`lexical_env`（`ParserInputs` 用のデータを提供）、`orchestration`（`ParseOutput` を利用）。
- 外部: `mizar-session`（`Edition`、AST ノード内に運ばれる `SourceRange`）。実 parser seam は、それらの crate が用意されてから、追加で `mizar-parser`（文法エントリポイント、Pratt 解析、回復、注釈／ドキュメントコメント付与）と `mizar-syntax`（`SurfaceAst`、`SyntaxDiagnostic`）に依存する。

このモジュールは、統制層が利用して `FrontendOutput` を組み立てる。

## データ構造

### パーサー入力

`ParserInputs` は、文法に影響する設定をフロントエンドが組み立てた束である。言語エディション、アクティブ字句環境から導出した演算子の結合性、文字列必須引数位置のレジストリからなる。字句解析器の `ParserLexContext` に対応するパーサー向けのものであり、任意のスコープや解決器の状態を運ぶことは決してない。

### Parser Seam と Surface AST の受け渡し

parser seam により、`mizar-syntax` と `mizar-parser` が導入される前でも、フロントエンドは source-to-token パイプラインをコンパイル・テストできる。実 seam では、`SurfaceAst`（`mizar-syntax` 由来）はソース形である。ソース順と `SourceRange` を保持し、注釈を構文ノードとして付与し、ドキュメントコメントを近接するドキュメント化可能項目に付与し、回復ノードを明示的に印付ける。フロントエンドはそれをそのまま通す。ノードの書き換え・剪定・解釈は行わない。

## アルゴリズム / ロジック

### トークンストリームの構文解析

1. アクティブ字句環境とエディションから `ParserInputs` を構築する。
2. `TokenStream` と入力を与えて、設定済みの `ParserSeam` を呼ぶ。スタブの seam は、`ast = None` と構文診断なしを返す。
3. パーサーは、モジュール、定義、登録、文、項、論理式、定理、証明、アルゴリズムを解析し、項・論理式の式に Pratt／優先順位解析を用い、注釈引数リストを解析して注釈を付与し、ドキュメントコメントを近接するドキュメント化可能項目に付与し、同期点（`;`、`end`、トップレベル項目キーワード、EOF）で回復する。
4. `SurfaceAst`（回復が失敗した場合は `None`）と構文診断を返す。

統合がパーサー駆動の曖昧性解消を用いる場合、パーサーは狭い文脈オブジェクトを通じて、文字列必須位置または記号種別フィルタを Step 4 へ供給する。その契約が導入されるまでは、実 parser 統合は、ソースレベルの注釈／演算子文字列リテラルのトークン化を要求してはならない。それらのテストは、合成トークンストリームを使うか、保留のままにする。パーサーが字句解析器のカーソルを直接駆動することは決してない。

## エラー処理

構文診断は `mizar-parser` から来る（予期しないトークン、不整合な区切り、`end` の欠落、文字列必須位置での文字列リテラルトークンの欠落、不正な注釈引数リスト）。フロントエンドは構文エラーのカテゴリを追加しない。使用可能な木を回復できない構文解析は、後のフェーズが適切にスキップ／縮退できるよう、診断とともに `ast = None` を返す。返された `SurfaceAst` 内の回復ノードはパーサーが印付ける。フロントエンドはその印を保持する。スタブの seam は構文診断を送出しないので、構文診断を必要とするテストは、実 parser seam が用意されてから着手する。

## テスト

主要シナリオ:

- スタブの seam は、`mizar-syntax` / `mizar-parser` に依存せず、`ast = None` と空の診断リストを返す。
- スタブの source-to-token 経路では、`ParserInputs` が空の `OperatorFixityTable` と `StringRequiredContext::None` を使う。
- 実 seam が利用可能になったら、整形式のトークンストリームが、ソース順と `SourceRange` を保持した `SurfaceAst` へ解析される。
- 字句サマリが fixity を公開したら、アクティブ字句環境からの演算子の結合性が、ユーザー定義中置演算子に対する正しい Pratt 優先順位を駆動する。
- 注釈引数位置の `StringLiteral` トークンが注釈ノードへ解析され、文字列必須位置での文字列リテラルの欠落が、期待される構文診断を生む。実ソーステキストに対するパーサー支援字句解析の契約が確定するまでは、合成トークンストリームを使う。
- `end` の欠落が同期点で回復し、明示的なエラーノードを生み、`ast` は `Some` のままである。
- 回復不能なトークンストリームが、診断とともに `ast = None` を返す。
- ドキュメントコメントは、可能なら後続のドキュメント化可能項目に付与され、それ以外ではソース位置の近くに保たれる。

## 制約と前提

- このモジュールは、AST ノード定義や文法／回復のロジックを所有しない。
- パーサーは、フロントエンドが生成したトークンストリームを消費する。文字列リテラルにパーサー支援字句解析が必要な場合でも、パーサーは合意済みの狭い文脈／プランオブジェクトを通じてのみ通信し、任意のパーサー状態を字句解析器に晒さない。
- `ParserInputs` は文法に影響する設定のみを運び、解決器の状態は運ばない。
- `SurfaceAst` は回復ノードを含みうる。後のフェーズは、それらを明示的に許容または拒否しなければならない。
- `SurfaceAst` のキャッシュキーは、トークンストリームハッシュ、パーサーバージョン、エディションである。
