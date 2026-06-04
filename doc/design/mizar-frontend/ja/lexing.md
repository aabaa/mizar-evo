# モジュール: lexing

> 正本は英語です。英語版: [../en/lexing.md](../en/lexing.md)。

状態: 進行中（task 7 の生スキャン／スコープスケルトン骨格は完了。task 8
の曖昧性解消と task 9 の回復パススルーは保留中）。

## 目的

このモジュールは、フロントエンドパイプラインの Step 4（字句解析／曖昧性解消）を実装する。task 7 の実装は、`mizar-lexer` の生スキャナーとスコープスケルトン事前走査を駆動し、`PreprocessedSource` を session スパン付きの生 `TokenStream` 骨格へ変える。task 8 と task 9 で、同じエントリポイントに文脈依存の曖昧性解消、最終的な parser-facing token kind、回復可能な字句解析器診断ペイロードの写像を追加する。

配線とスパン橋渡しを所有するが、最長一致規則・スコープスケルトン構築・パーサー字句文脈の意味は所有しない（それらは `mizar-lexer` にある）。識別子が定義済みか、どのオーバーロードが適用されるかといった意味的判断は行わない。

[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Step 4: Lex」「Lexing Is Raw First, Then Contextually Disambiguated」「Dot Handling Is Split Across Lexer, Parser, and Resolver」「String Literals Are Fully Tokenized by the Lexer」を参照。

## 公開 API

```rust
pub struct TokenStream {
    pub source_id: SourceId,
    pub parser_context: ParserLexContext,
    pub tokens: Vec<Token>,
    pub scope_view: ScopeView,
    pub diagnostics: Vec<LexingDiagnostic>,
}

pub struct Token {
    pub kind: TokenKind,
    pub span: SourceRange,
    pub text: InternedText,
}

pub type InternedText = Arc<str>;

pub struct TokenizeRequest<'a> {
    pub preprocessed: &'a PreprocessedSource,
    pub environment: &'a ActiveLexicalEnvironment,
    pub parser_context: ParserLexContext,
}

pub struct LexingDiagnostic {
    pub kind: LexingDiagnosticKind,
    pub message: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub payload: LexingDiagnosticPayload,
}

pub enum LexingDiagnosticKind {
    RawScan,
    ScopeSkeleton(ScopeSkeletonDiagnosticCode),
    Lexer(LexDiagnosticCode),
}

pub enum LexingDiagnosticPayload {
    None,
}

pub struct ScopeView {
    pub source_id: SourceId,
    pub frames: Vec<ScopeFrame>,
    pub blocks: Vec<ScopeBlock>,
    pub statements: Vec<ScopeStatement>,
}

pub struct ScopeFrame {
    pub range: SourceRange,
    pub bindings: Vec<ScopedBinding>,
}

pub struct ScopedBinding {
    pub spelling: InternedText,
    pub introduced_at: SourceRange,
    pub kind: BindingShapeKind,
}

pub struct ScopeBlock {
    pub kind: LexicalBlockKind,
    pub range: SourceRange,
}

pub struct ScopeStatement {
    pub kind: LexicalStatementKind,
    pub range: SourceRange,
}

pub fn tokenize(
    request: TokenizeRequest<'_>,
    bridge: &SpanBridge,
) -> Result<TokenStream, SpanBridgeError>;
```

`TokenKind`、`ParserLexContext`、`BindingShapeKind`、`LexicalBlockKind`、`LexicalStatementKind`、`ScopeSkeletonDiagnosticCode`、`LexDiagnosticCode` は `mizar-lexer` から再エクスポートされる。`InternedText` は、最初の実装ではフロントエンドローカルな `Arc<str>` のつづりハンドルであり、グローバルなインターナは不要である。字句解析器の `String` lexeme は `Arc::<str>::from` で変換する。生の字句解析器診断構造体は入力としてのみ消費する。フロントエンドはそれらを即座に `LexingDiagnostic` へ変換し、公開診断は session 範囲を持ち、生の字句解析器バイトスパンを再公開しない。構造化された字句解析器ペイロード variant と写像済みの棄却候補は、task 8 と task 9 で追加する。

`parser_context` は、task 8 で同じ request を曖昧性解消器へ渡せるように、現時点から保持する。task 7 の骨格はこれを記録するが、まだ token 分類には使わない。`environment` も task 8 のための request 境界であり、task 7 はインポート済み記号が字句的な `ScopeView` に混ざらないことを確認する。

## 依存関係

- 内部: `preprocess`（字句テキストとマップ）、`lexical_env`（`ActiveLexicalEnvironment`）、`span_bridge`（字句バイトから `SourceRange`）、`parsing`（`TokenStream` を利用）。
- 外部: `mizar-lexer`（`scan_raw`、`build_scope_skeleton`、`TokenKind`、`ParserLexContext`、`LexDiagnosticCode`、`ScopeSkeletonDiagnostic`、`ScopeSkeletonDiagnosticCode`、スコープのブロック／文／束縛形状 enum）、`mizar-session`（`SourceId`、`SourceRange`、`SourceAnchor`）。task 8 と task 9 で、`disambiguate`、`LexDiagnostic`、構造化された字句解析器ペイロード写像を追加する。

このモジュールは構文解析が利用し、診断を通じて統制統合も利用する。

## データ構造

### トークンストリーム

`TokenStream` は、1 ファイルのソースに忠実なトークン列であり、その実行で使った parser lexing context の下にある。task 7 の骨格では、layout 以外の生トークンを session スパン付きの `TokenKind::LexemeRun` として公開し、厳密な生スキャン失敗では粗い `TokenKind::ErrorRecovery` を 1 つ出力する。task 8 で、これらの生 run を最終的な parser-facing の `UserSymbol`、`ReservedWord`、`ReservedSymbol`、`Identifier`、`Numeral`、`StringLiteral` 分類に置き換える。各 `Token` は、元のつづり（`text`）と session の `SourceRange` を保持する。

`LexingDiagnostic` は、Step 4 のフロントエンド側の写像済み診断ペイロードである。task 7 では、生スキャン失敗とスコープスケルトン診断を表す。task 8 と task 9 で、曖昧性解消器診断と構造化ペイロード variant を追加する。統制層のために必ず session 座標の primary 範囲と副次アンカーを持つ。副次は `SourceAnchor` なので、複合／縮退した preprocess マッピングの点、生成、隣接コメントのアンカーを保持できる。生の字句解析器診断オブジェクトには字句解析器のバイトスパンが含まれるため、`LexingDiagnostic` は写像済みの code/message を持ち、生オブジェクト自体は保持しない。

### スコープ字句ビュー

スコープスケルトン事前走査（`build_scope_skeleton`）は、生の字句解析器出力に対する読み取り専用の `ScopeLexView` を生成し、曖昧性解消器がスコープ付き識別子の上書き規則に用いる。フロントエンドは、その skeleton を公開用の session スパン付き `ScopeView` へ写像し、後続の検査と診断に使えるようにする。スコープ自体は構築せず、ビューは字句的な形のみを記録し、解決済みの束縛は決して記録しない。task 8 で、同じ lexing pass の中で raw scope view を使って曖昧性解消器を実行する。

## アルゴリズム / ロジック

### 前処理済みソースのトークン化

1. 字句テキストを生スキャン（`scan_raw`）し、スパンを保持した生単位にする。厳密な生スキャンが失敗した場合は、字句テキスト全体（空ならソース先頭のゼロ長範囲）を覆う粗い `ErrorRecovery` トークン 1 つと、写像済みの `LexingDiagnosticKind::RawScan` 診断 1 つを出力し、その実行ではスコープスケルトン構築と曖昧性解消をスキップする。現在の `mizar_lexer::LexError` はスパンや部分トークンのペイロードを持たないため、より細かい回復はフォローアップ契約として追跡する。
2. 生トークンから `ScopeSkeleton` / `ScopeLexView` を構築し、スコープ付き識別子の上書きに用いる。`ScopeSkeletonDiagnostic` は収集して写像する。
3. layout 以外の生単位を `span_bridge.lexical_span` で写像し、session スパン付きの `TokenKind::LexemeRun` 骨格トークンにする。スコープスケルトンの frame、block、statement、束縛形状も公開 `ScopeView` へ写像する。
4. task 8 で、`disambiguate`（またはパーサー統合の `lex`）を実行し、次の順序で最長一致する。アクティブなユーザー記号、予約特殊記号、予約語、識別子／数値規則、そして言語が要求する場合のパーサー期待／スコープ上書きである。
5. task 8 で、現在のパーサー字句文脈が文字列を要求するときだけ `StringLiteral` トークンも認識する。選択された byte span だけに対する位置依存文脈は、パーサー支援字句解析の契約まで延期する。
6. task 8 と task 9 で、字句解析器の診断を `LexingDiagnostic` として集め、スパン以外のペイロードデータをコピーし、入れ子の候補スパンを写像済みペイロード構造へ写像する。

複合予約トークン（`.{`、`.*`、`.=`、`...`）は字句解析器が認識する。`.` のセレクタ／名前空間としての役割は、パーサーと解決器に委ねる。字句解析器は、定義済み性・適用可能性・オーバーロード選択を決して判断しない。

## エラー処理

task 7 の骨格は、回復可能な生スキャンとスコープスケルトンの問題を保持する。

- 厳密な生スキャンの致命的失敗は、利用できる最善のソース範囲を持つ粗い `TokenKind::ErrorRecovery` 1 つと、`RawScan` 診断を出力する。
- `ScopeSkeletonDiagnostic` は、スパンを持つ生の診断構造体を `TokenStream` に保持せず、フロントエンド診断へ写像する。

ユーザー入力に由来する回復可能な字句問題に対して、`tokenize` は `Ok(TokenStream)` を返す。厳密な生スキャンの失敗は、粗い回復トークンと写像済み診断へ縮退し、スコープスケルトンの問題は、回復可能な生トークンを落とさずに写像される。`Err(SpanBridgeError)` は、トークン、スコープ形状、診断スパンを登録済みの bridge で写像できない場合に限る。曖昧性解消器の回復パススルー（不正トークン、不正な数値、不正な文字列リテラルを含む）は task 9 で扱う。

## テスト

task 7 のシナリオ:

- 生スキャンが `LexemeRun` のスパンを保持する。コメント除去などの preprocess 写像を通じたスパンも含む。
- 公開 `ScopeView` が、解決済み／インポート済み束縛なしで、字句ブロック／文の形とローカル束縛形状を反映する。
- スコープスケルトン診断が、写像済みの `LexingDiagnostic` として保持される。
- 送出された各トークンの `span` は、`source_id` に対する妥当な `SourceRange` である。

task 8/9 のシナリオでは、ユーザー記号の最長一致、複合予約トークン、`StringRequired` 実行、回復可能な不正トークン、不正な数値、不正な文字列リテラル、構造化された字句解析器ペイロード写像を追加する。

## 制約と前提

- このモジュールは字句解析器を統制する。最長一致・スコープスケルトン・曖昧性解消の規則は所有しない。
- 字句解析器は、複合予約トークンとアクティブなユーザー記号を認識するが、意味的なセレクタ／名前空間としての役割は認識しない。
- `StringLiteral` トークンは、パーサー字句文脈が文字列を要求するときだけ送出される。正確な文法位置の認識は、パーサー支援字句解析の契約で確定する。
- トークンストリームをキャッシュする場合、キャッシュキーは `PreprocessedSource.lexical_hash`、アクティブ字句環境のフィンガープリント、およびその実行で使った `ParserLexContext` ／パーサー支援字句解析プランの安定したエンコードである。
- すべてのトークンスパンは、`span_bridge` を通じて生成された session の `SourceRange` 値である。
