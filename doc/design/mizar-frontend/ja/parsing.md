# モジュール: parsing

> 正本は英語です。英語版: [../en/parsing.md](../en/parsing.md)。

状態: パーサー入力の組み立て、ソース位置対応の演算子メタデータ、スタブ parser seam、
最小限の実 `mizar-parser` seam、task 12 の parser recovery passthrough、task 20 の位置別
parser lexing-plan integration、task 28 の入れ子 block-end recovery follow-through は実装済み。

## 目的

このモジュールは、フロントエンドパイプラインの Step 5（パーサー呼び出し）を実装する。parser seam を通じて `TokenStream` を AST へ変え、アクティブ字句環境から導出したパーサー入力（演算子 fixity、すなわち優先順位と結合性、文字列必須の文法文脈）を供給し、orchestration が Step 4 の tokenization 前に parser lexing plan を事前計算するための狭い parser-facing contract を定義する。

`SurfaceAst` ノード定義（実 parser seam では `mizar-syntax` にある）も、文法・Pratt 優先順位・回復・注釈付与のロジック（実 parser seam では `mizar-parser` にある）も所有しない。crate 境界で入出力を適合させるだけである。

[architecture/ja/02.source_and_frontend.md](../../architecture/ja/02.source_and_frontend.md) の「ステップ 5: 構文解析」「構文とパーサーは独立した crate 境界にする」「アノテーションは構文解析器が所有する構文である」を参照。

## 公開 API

```rust
pub const DEFAULT_PARSER_CACHE_KEY_VERSION: &str;
pub const STUB_PARSER_CACHE_KEY_VERSION: &str;
pub const MIZAR_PARSER_CACHE_KEY_VERSION: &str;

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

    pub fn from_active_environment_and_local_declarations(
        edition: Edition,
        environment: &ActiveLexicalEnvironment,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Self;

    pub fn try_from_active_environment_and_local_declarations<E>(
        edition: Edition,
        environment: &ActiveLexicalEnvironment,
        local_declarations: &LocalLexicalDeclarations,
        map_active_from: impl FnMut(usize) -> Result<usize, E>,
    ) -> Result<Self, E>;
}

pub struct OperatorFixityTable {
    pub entries: Vec<OperatorFixityEntry>,
}

impl OperatorFixityTable {
    pub fn empty() -> Self;
    pub fn is_empty(&self) -> bool;
    pub fn try_from_active_environment_and_local_declarations<E>(
        environment: &ActiveLexicalEnvironment,
        local_declarations: &LocalLexicalDeclarations,
        map_active_from: impl FnMut(usize) -> Result<usize, E>,
    ) -> Result<Self, E>;
}

pub struct OperatorFixityEntry {
    pub spelling: Arc<str>,
    pub fixity: OperatorFixity,
    pub precedence: u8,
    pub active_from: usize,
}

pub enum OperatorFixity {
    Prefix,
    Infix(OperatorAssociativity),
    Postfix,
}

pub enum OperatorAssociativity {
    Left,
    Right,
    NonAssociative,
}

pub enum StringRequiredContext {
    None,
    PositionSensitive,
    UniformForTest,
}

impl StringRequiredContext {
    pub fn parser_lex_context(self) -> ParserLexContext;
    pub fn parser_lexing_plan(self, lexical_text: &str) -> ParserLexingPlan;
}

pub trait ParserSeam {
    type Ast;
    type Diagnostic;

    fn cache_key_version(&self) -> ParserCacheKeyVersion;
    fn parse(&self, request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic>;
}

pub struct ParserCacheKeyVersion {
    pub version: Arc<str>,
}

impl ParserCacheKeyVersion {
    pub fn new(version: impl Into<Arc<str>>) -> Self;
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

`SurfaceAst` と `SyntaxDiagnostic` は `mizar-syntax` が所有する。実 `MizarParserSeam` は `mizar-parser` のエントリポイントへ委譲し、それらの出力をそのまま返す。`StubParserSeam` は source-to-token coordinator 経路のために残り、`ast = None` と空の診断リストを返す。`ParserInputs` は、Step 3 の後にフロントエンドがアクティブ字句環境とエディションから導出する。トップレベルのコーディネータの呼び出し側は、これを渡さない。`ParserSeam::cache_key_version` は `SurfaceAstCacheKey` の parser component を供給する。`MizarParserSeam` と `StubParserSeam` は明示的な version を使い、custom seam は override しない限り保守的な custom-seam version を継承する。

`SurfaceAst` reuse には token stream hash、parser version、parser input hash、range-aware operator view hash、edition の一致が必要である。parser output は syntax のみであり、semantic `DependencySlice` ではなく、proof reuse を認可できない。

`OperatorFixity` と `OperatorAssociativity` は frontend parser-input API で意図的に exhaustive とする。これらは `mizar-parser` が現在受け付ける閉じた parser-facing operator metadata、つまり prefix、postfix、infix fixity と left/right/non-associative の infix associativity を写す。将来の operator model が fixity や associativity の分類を追加する場合は、この API、`mizar-parser`、cache-key hash、公開 enum lint-policy guard をまとめて更新しなければならず、non-exhaustive match に silently fall through させない。

`operator_fixity` は、依存字句サマリと現在のモジュールのローカル宣言から埋める。notation が operator-shaped である visible symbolic functor について、summary producer は宣言済み metadata、または Chapter 10 / Appendix B の default（宣言のない infix form では `64`、non-associative）を公開する責務を持つ。lexer は local operator declaration を lexical text の byte coordinate で公開するため、orchestration は各 activation point を `SpanBridge` で写像し、`OperatorFixityEntry.active_from` が parser token span と同じ source coordinate byte space になるようにする。`ParserInputs` は、文法に影響する spelling 単位の fact だけを決定的な parser entry に写し、summary boundary の後で欠けている notation form や default precedence を推論せず、overload root も選ばない。operator metadata を公開していない symbol は省略し、明示的な合成 parser input は有界な seam test 用に引き続き利用できる。`StringRequiredContext::PositionSensitive` は通常の source-to-token mode であり、tokenization 前に字句バイト範囲上の `ParserLexingPlan` を事前計算するよう frontend に要求する。`StringRequiredContext::None` は parser-assisted lexing を意図的に無効にする合成 parser 入力用に残り、`UniformForTest` は、字句解析器全体を意図的に `ParserLexContext::string_required()` で実行する有界なテスト専用である。位置別の文字列必須スパンとパーサー駆動の記号種別フィルタは `ParserLexingPlan` で表現され、任意の parser state を lexer に公開しない。

stub parser seam では、`ast = None` は期待されるプレースホルダ結果である。実 parser seam は、block stack の対応付け後に残る `end` 欠落と、文字列リテラル必須位置の回復ノードを含め、回復済みトークン列に対して最小の `SurfaceAst` を返す。パーサーが以降のフェーズに十分な構造を回復できない場合は `ast = None` を返せる。字句診断と構文診断は依然として返される。

## 依存関係

- 内部: `lexing`（`TokenStream` を提供）、`lexical_env`（`ParserInputs` 用のデータを提供）、`orchestration`（`ParseOutput` を利用）。
- 外部: `mizar-session`（`Edition`、AST ノード内に運ばれる `SourceRange`）、`mizar-syntax`（`SurfaceAst`、`SyntaxDiagnostic`）、`mizar-parser`（文法エントリポイントと最小限の Pratt 解析）。

このモジュールは、統制層が利用して `FrontendOutput` を組み立てる。

## データ構造

### パーサー入力

`ParserInputs` は、文法に影響する設定をフロントエンドが組み立てた束である。言語エディション、アクティブ字句環境とローカル宣言から導出した演算子 fixity、文字列必須引数位置のレジストリからなる。fixity entry は spelling、fixity kind、precedence、infix の場合の associativity、そしてその metadata が有効になる source byte offset を含む。operator metadata は選択済み overload root ではなく spelling に付くため、symbol id は運ばない。字句解析器の `ParserLexContext` に対応するパーサー向けのものであり、任意のスコープや解決器の状態を運ぶことは決してない。

### Parser Seam と Surface AST の受け渡し

parser seam により、フロントエンドは stubbed source-to-token pipeline と実 parser 境界のどちらもコンパイル・テストできる。実 seam は、`SurfaceAst` の token node でソース順と `SourceRange` を保持し、明示的または summary-derived な prefix/postfix/infix fixity を parser Pratt 境界で扱い、block stack の対応付け後に残る `end` 欠落と文字列リテラル必須位置の回復マーカーをそのまま通す。後続の parser タスクで、同じ境界に完全なモジュール／項目ノード、注釈付与、ドキュメントコメント付与、より広い回復マーカーを追加する。フロントエンドは parser 出力をそのまま通す。ノードの書き換え・剪定・解釈は行わない。

## アルゴリズム / ロジック

### トークンストリームの構文解析

1. tokenization 前に `StringRequiredContext::PositionSensitive` から `ParserLexingPlan` を構築する。
2. tokenization は現在のモジュールの local lexical declaration を収集し、`TokenStream` に保持して返す。
3. アクティブ字句環境、local declaration、エディションから最終的な `ParserInputs` を構築する。local operator の activation offset は parser に届く前に lexical text byte から source byte へ写像する。
4. `TokenStream` と入力を与えて、設定済みの `ParserSeam` を呼ぶ。スタブの seam は、`ast = None` と構文診断なしを返す。
5. parser は、token node をソース順に保持し、演算子 fixity が与えられた場合に operator expression node を構築し、block stack の対応付け後も開いている block の `end` 欠落と、文字列リテラル必須位置の回復マーカーを保持する。後続の parser タスクで、完全なモジュール／項目構文解析、注釈とドキュメントコメントの付与、より広い同期回復を追加する。
6. `SurfaceAst` と構文診断を返す。parser が回復不能な入力を報告した場合は `ast = None` を返す。

パーサー支援の曖昧性解消は、parser と lexer の実行を交錯させるのではなく、事前計算された位置別 plan を使う。parser-facing input は `StringRequiredContext::PositionSensitive` を選び、orchestration は字句テキストから 1 つの `ParserLexingPlan` を導出し、Step 4 はその plan に従って `ParserLexContext` 値だけを lexer に渡す。lexer は任意の parser state を受け取らず、parser が lexer 内部を直接変更することもない。

## エラー処理

構文診断は `mizar-parser` から来る。フロントエンドは構文エラーのカテゴリを追加しない。parser は、`UnexpectedErrorToken`、`DanglingOperator`、`NonAssociativeOperatorChain`、`MissingEnd`、`MissingStringLiteral`、`UnrecoverableInput` を送出できる。回復可能な欠落構文要素は、`recovered` が付いた明示的な `SurfaceNodeKind::ErrorRecovery` node で表す。回復不能な入力では、診断とともに `ast = None` を返す。一般的な予期しないトークン、不整合な区切り、不正な注釈引数リストのような広い診断は、後続の parser/recovery 作業で扱う。スタブの seam は構文診断を送出せず、`ast = None` を返す。そのため、`DiagnosticClass::AnnotationSyntax` はまだ parser-backed producer surface ではなく、orchestration レベルの予約 class である。

## テスト

主要シナリオ:

- スタブの seam は、`ast = None` と空の診断リストを返す。
- `ParserInputs` は、declared または spec-defaulted fixity を公開する active lexical environment summary と local declaration から決定的な operator-fixity entry を導出し、local activation offset を parser source coordinate に写像し、operator metadata のない symbol は省略し、通常の source-to-token 経路では `StringRequiredContext::PositionSensitive` を使う。
- 実 seam は、整形式のトークンストリームを、ソース順と `SourceRange` を保持した `SurfaceAst` へ解析する。
- 実 seam は token-kind adaptation を保持し、parser 診断をそのまま返す。
- 明示的および summary-derived な operator fixity が、対応する prefix、infix、postfix operator の Pratt 優先順位と fixity / associativity を駆動する。
- `active_from` が実 parser seam に届き、未来の operator metadata がそれ以前の token に影響しない。
- 回復済み token と unknown token は保持されるが、infix operator にはならない。
- 同じ非結合演算子の連鎖は診断され、同じ precedence の別演算子は区別される。
- 利用可能な `end` token を対応付けた後も block が開いている場合、`end` の欠落は保守的に EOF で回復し、明示的な回復ノードを返す。
- 回復不能な 1 トークンの `end` 入力では、構文診断とともに `ast = None` が保持される。
- 一様な string-required 位置での文字列リテラル欠落は、合成トークンストリームで診断を検証する。一方、実ソーステキストの annotation string argument は位置別 plan を通じて token 化される。
- 注釈ノード、不正な注釈の回復、ドキュメントコメント付与は、後続の parser/recovery coverage として残る。

## 制約と前提

- このモジュールは、AST ノード定義や文法／回復のロジックを所有しない。
- パーサーは、フロントエンドが生成したトークンストリームを消費する。文字列リテラルにパーサー支援字句解析が必要な場合でも、パーサーは合意済みの狭い文脈／プランオブジェクトを通じてのみ通信し、任意のパーサー状態を字句解析器に晒さない。
- `ParserInputs` は文法に影響する設定のみを運び、解決器の状態は運ばない。
- `SurfaceAst` は回復ノードを含みうる。後のフェーズは、それらを明示的に許容または拒否しなければならない。
- `SurfaceAst` のキャッシュキーは、トークンストリームハッシュ、パーサーバージョン、parser inputs hash、エディションである。
