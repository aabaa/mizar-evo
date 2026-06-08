# モジュール: parsing

> 正本は英語です。英語版: [../en/parsing.md](../en/parsing.md)。

状態: パーサー入力の組み立て、スタブ parser seam、最小限の実
`mizar-parser` seam、task 12 の parser recovery passthrough は実装済み。完全な文法回復は引き続き保留中。

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
    pub precedence: u8,
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

pub struct MizarParserSeam;
```

`SurfaceAst` と `SyntaxDiagnostic` は `mizar-syntax` が所有する。実 `MizarParserSeam` は `mizar-parser` のエントリポイントへ委譲し、それらの出力をそのまま返す。`StubParserSeam` は source-to-token coordinator 経路のために残り、`ast = None` と空の診断リストを返す。`ParserInputs` は、Step 3 の後にフロントエンドがアクティブ字句環境とエディションから導出する。トップレベルのコーディネータの呼び出し側は、これを渡さない。

`operator_fixity` は、依存字句サマリが公開しているデータからのみ埋める。現在のサマリの形がまだ fixity を公開していない場合、通常の source-to-token 経路では `OperatorFixityTable { entries: Vec::new() }` を使う。一方で、明示的な合成 parser 入力により、実 Pratt/fixity seam は検証できる。`StringRequiredContext::None` は通常の source-to-token 基盤モードであり、`UniformForTest` は、字句解析器全体を意図的に `ParserLexContext::string_required()` で実行する有界なテスト専用である。位置別の文字列必須スパンとパーサー駆動の記号種別フィルタは、この初期型では表現しない。実ソース入力で文法位置の文字列リテラルが必要になる前に、パーサー支援字句解析の契約で追加する。

stub parser seam では、`ast = None` は期待されるプレースホルダ結果である。実 parser seam は、`end` token が存在しない場合の `end` 欠落と、文字列リテラル必須位置の task 12 回復ノードを含め、回復済みトークン列に対して最小の `SurfaceAst` を返す。パーサーが以降のフェーズに十分な構造を回復できない場合は `ast = None` を返せる。字句診断と構文診断は依然として返される。

## 依存関係

- 内部: `lexing`（`TokenStream` を提供）、`lexical_env`（`ParserInputs` 用のデータを提供）、`orchestration`（`ParseOutput` を利用）。
- 外部: `mizar-session`（`Edition`、AST ノード内に運ばれる `SourceRange`）、`mizar-syntax`（`SurfaceAst`、`SyntaxDiagnostic`）、`mizar-parser`（文法エントリポイントと最小限の Pratt 解析）。

このモジュールは、統制層が利用して `FrontendOutput` を組み立てる。

## データ構造

### パーサー入力

`ParserInputs` は、文法に影響する設定をフロントエンドが組み立てた束である。言語エディション、アクティブ字句環境から導出した演算子の結合性、文字列必須引数位置のレジストリからなる。字句解析器の `ParserLexContext` に対応するパーサー向けのものであり、任意のスコープや解決器の状態を運ぶことは決してない。

### Parser Seam と Surface AST の受け渡し

parser seam により、フロントエンドは stubbed source-to-token pipeline と実 parser 境界のどちらもコンパイル・テストできる。実 seam は、`SurfaceAst` の token node でソース順と `SourceRange` を保持し、明示的な infix fixity を小さな Pratt parser で扱い、task 12 の回復マーカーをそのまま通す。後続の parser タスクで、同じ境界に完全なモジュール／項目ノード、注釈付与、ドキュメントコメント付与、より広い回復マーカーを追加する。フロントエンドは parser 出力をそのまま通す。ノードの書き換え・剪定・解釈は行わない。

## アルゴリズム / ロジック

### トークンストリームの構文解析

1. アクティブ字句環境とエディションから `ParserInputs` を構築する。
2. `TokenStream` と入力を与えて、設定済みの `ParserSeam` を呼ぶ。スタブの seam は、`ast = None` と構文診断なしを返す。
3. parser は、token node をソース順に保持し、明示的な演算子 fixity が与えられた場合に最小限の infix expression node を構築し、`end` token が存在しない場合の `end` 欠落と、文字列リテラル必須位置の task 12 回復マーカーを保持する。後続の parser タスクで、完全なモジュール／項目構文解析、注釈とドキュメントコメントの付与、より広い同期回復を追加する。
4. `SurfaceAst` と構文診断を返す。parser が回復不能な入力を報告した場合は `ast = None` を返す。

統合がパーサー駆動の曖昧性解消を用いる場合、パーサーは狭い文脈オブジェクトを通じて、文字列必須位置または記号種別フィルタを Step 4 へ供給する。その契約が導入されるまでは、実 parser 統合は、ソースレベルの注釈／演算子文字列リテラルのトークン化を要求してはならない。それらのテストは、合成トークンストリームを使うか、保留のままにする。パーサーが字句解析器のカーソルを直接駆動することは決してない。

## エラー処理

構文診断は `mizar-parser` から来る。フロントエンドは構文エラーのカテゴリを追加しない。parser は、`UnexpectedErrorToken`、`DanglingOperator`、`NonAssociativeOperatorChain`、`MissingEnd`、`MissingStringLiteral`、`UnrecoverableInput` を送出できる。回復可能な欠落構文要素は、`recovered` が付いた明示的な `SurfaceNodeKind::ErrorRecovery` node で表す。回復不能な入力では、診断とともに `ast = None` を返す。一般的な予期しないトークン、不整合な区切り、不正な注釈引数リストのような広い診断は、後続の parser/recovery 作業で扱う。スタブの seam は構文診断を送出せず、`ast = None` を返す。

## テスト

主要シナリオ:

- スタブの seam は、`ast = None` と空の診断リストを返す。
- スタブの source-to-token 経路では、`ParserInputs` が空の `OperatorFixityTable` と `StringRequiredContext::None` を使う。
- 実 seam は、整形式のトークンストリームを、ソース順と `SourceRange` を保持した `SurfaceAst` へ解析する。
- 実 seam は token-kind adaptation を保持し、parser 診断をそのまま返す。
- 明示的な演算子 fixity が、対応する infix operator の Pratt 優先順位と結合性を駆動する。字句サマリが fixity を公開したら、アクティブ字句環境は同じ parser 入力を埋める。
- 回復済み token と unknown token は保持されるが、infix operator にはならない。
- 同じ非結合演算子の連鎖は診断され、同じ precedence の別演算子は区別される。
- `end` token が存在しない場合、`end` の欠落は保守的に EOF で回復し、明示的な回復ノードを返す。
- 回復不能な 1 トークンの `end` 入力では、構文診断とともに `ast = None` が保持される。
- 文字列リテラル必須位置での欠落は、task 20 まで実ソーステキストではなく合成トークンストリームで診断を検証する。
- 注釈ノード、不正な注釈の回復、ドキュメントコメント付与は、後続の parser/recovery coverage として残る。

## 制約と前提

- このモジュールは、AST ノード定義や文法／回復のロジックを所有しない。
- パーサーは、フロントエンドが生成したトークンストリームを消費する。文字列リテラルにパーサー支援字句解析が必要な場合でも、パーサーは合意済みの狭い文脈／プランオブジェクトを通じてのみ通信し、任意のパーサー状態を字句解析器に晒さない。
- `ParserInputs` は文法に影響する設定のみを運び、解決器の状態は運ばない。
- `SurfaceAst` は回復ノードを含みうる。後のフェーズは、それらを明示的に許容または拒否しなければならない。
- `SurfaceAst` のキャッシュキーは、トークンストリームハッシュ、パーサーバージョン、エディションである。
