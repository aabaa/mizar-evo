# モジュール: lexing

> 正本は英語です。英語版: [../en/lexing.md](../en/lexing.md)。

状態: planned。

## 目的

このモジュールはフロントエンドパイプラインの Step 4（字句解析／曖昧性解消）を実装する。`mizar-lexer` の生スキャナー、スコープスケルトン事前走査、文脈依存曖昧性解消器を駆動し、`PreprocessedSource` と `ActiveLexicalEnvironment` を、スパンが `mizar-session` `SourceRange` 値である `TokenStream` へ変える。

配線と span 橋渡しを所有するが、最長一致規則・スコープスケルトン構築・パーサー字句文脈の意味は所有しない（それらは `mizar-lexer` にある）。識別子が定義済みか、どのオーバーロードが適用されるかといった意味的判断は行わない。

[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Step 4: Lex」「Lexing Is Raw First, Then Contextually Disambiguated」「Dot Handling Is Split Across Lexer, Parser, and Resolver」「String Literals Are Fully Tokenized by the Lexer」を参照。

## 公開 API

```rust
pub struct TokenStream {
    pub source_id: SourceId,
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<LexingDiagnostic>,
}

pub struct Token {
    pub kind: TokenKind,
    pub span: SourceRange,
    pub text: InternedText,
}

pub struct TokenizeRequest<'a> {
    pub preprocessed: &'a PreprocessedSource,
    pub environment: &'a ActiveLexicalEnvironment,
    pub parser_context: ParserLexContext,
}

pub struct LexingDiagnostic {
    pub kind: LexingDiagnosticKind,
    pub message: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceRange>,
}

pub enum LexingDiagnosticKind {
    RawScan,
    ScopeSkeleton(ScopeSkeletonDiagnosticCode),
    Lexer(LexDiagnosticCode),
}

pub fn tokenize(
    request: TokenizeRequest<'_>,
    bridge: &SpanBridge,
) -> Result<TokenStream, SpanBridgeError>;
```

`TokenKind`、`ParserLexContext`、`ParserLexMode`、`LexError`、`ScopeSkeletonDiagnosticCode`、`LexDiagnosticCode` は `mizar-lexer` から再エクスポートされる。raw lexer diagnostic struct は入力としてだけ消費する。フロントエンドはそれらを即座に `LexingDiagnostic` へ変換し、公開診断は session 範囲を持ち、raw lexer byte span を再公開しない。将来、span を含む入れ子 payload が必要になった場合は、その payload も frontend-mapped representation を持たせる。

`parser_context` は曖昧性解消器が必要とする狭い文法由来の信号を、任意のパーサー状態を晒さずに運ぶ。現在の `mizar-lexer` API は uniform context であり、注釈／演算子宣言引数のような位置別の文字列必須 span はまだ表現しない。source → tokens の基盤では `ParserLexContext::general()` を使い、位置別の `StringLiteral` 認識は parser-assisted lexing contract の確定後に追加する。

## 依存関係

- 内部: `preprocess`（字句テキストとマップ）、`lexical_env`（`ActiveLexicalEnvironment`）、`span_bridge`（字句バイト → `SourceRange`）、`parsing`（`TokenStream` を消費）。
- 外部: `mizar-lexer`（`scan_raw`、`build_scope_skeleton`、`ScopeLexView`、`disambiguate`、`lex`、`Token`、`TokenKind`、`ParserLexContext`、`LexError`、`LexDiagnostic`、`LexDiagnosticCode`、`ScopeSkeletonDiagnostic`、`ScopeSkeletonDiagnosticCode`）、`mizar-session`（`SourceId`、`SourceRange`）。

このモジュールは構文解析が消費し、診断を通じて統制統合も消費する。

## データ構造

### トークンストリーム

`TokenStream` は 1 ファイルの完全かつソース忠実なトークン列である。各 `Token` は元のつづり（`text`）と session `SourceRange` を保持する。`TokenKind` には分割されなかった生単位の `LexemeRun`、`UserSymbol`、`ReservedWord`、`ReservedSymbol`、`Identifier`、`Numeral`、`StringLiteral`、`ErrorRecovery` が含まれる。現在の uniform context API では、明示的な `StringRequired` 実行のときだけ `StringLiteral` が現れる。文法上の位置に基づく string-required registry は parser-assisted lexing contract の範囲である。

`LexingDiagnostic` は Step 4 の frontend 側 mapped diagnostic payload である。raw-scan 失敗、スコープスケルトン診断、曖昧性解消器診断を表し得るが、統制層のために必ず session 座標の第一／副次範囲を持つ。raw lexer diagnostic object には lexer byte span が含まれるため、`LexingDiagnostic` は写像済みの code/message を持ち、raw object 自体は保持しない。

### スコープ字句ビュー

スコープスケルトン事前走査（`build_scope_skeleton`）は、生の字句解析器出力に対する読み取り専用の `ScopeLexView` を生成し、曖昧性解消器がスコープ付き識別子の上書き規則に用いる。フロントエンドはこのビューを構築して曖昧性解消器に渡す。スコープ自体は構築せず、ビューは字句的な形のみを記録し、解決済み束縛は決して記録しない。

## アルゴリズム / ロジック

### 前処理済みソースのトークン化

1. 字句テキストを生スキャン（`scan_raw`）し、スパンを保持した `LexemeRun` にする。strict raw scan が失敗した場合は、字句テキスト全体（空ならソース先頭のゼロ長範囲）を覆う粗い `ErrorRecovery` トークン 1 つと mapped `LexingDiagnosticKind::RawScan` 診断 1 つを出力し、その実行ではスコープスケルトン構築と曖昧性解消をスキップする。現在の `mizar_lexer::LexError` は span や部分 token payload を持たないため、より細かい回復は follow-up contract として追跡する。
2. 生トークンから `ScopeSkeleton` / `ScopeLexView` を構築し、スコープ付き識別子の上書きに用いる。`ScopeSkeletonDiagnostic` は収集して写像する。
3. `disambiguate`（またはパーサー統合の `lex`）を実行し、次の順序で最長一致する。アクティブユーザー記号、予約特殊記号、予約語、識別子／数値規則、そして言語が要求する場合のパーサー期待／スコープ上書き。
4. `StringLiteral` トークンは現在の parser lexing context が文字列を要求するときだけ認識する。それ以外では引用符は通常の記号文字のままである。注釈／演算子位置の string-required 判定は real parser contract まで延期する。
5. 結果の各字句解析器スパンを `span_bridge.lexical_span` を通じて、`source_id` でスコープされた第一 `SourceRange` へ変換し、隣接アンカーは診断用に保持する。
6. raw-scan、スコープスケルトン、字句解析器の診断を `LexingDiagnostic` として集め、`TokenStream` を返す。

複合予約トークン（`.{`、`.*`、`.=`、`...`）は字句解析器が認識する。`.` のセレクタ／名前空間の役割はパーサーと解決器に委ねる。字句解析器は定義済み性・適用可能性・オーバーロード選択を決して判断しない。

## エラー処理

字句解析器の回復はエンドツーエンドで保持される。

- 不正なスパンは診断のために元のソース範囲を保持したまま `TokenKind::ErrorRecovery` を送出する。
- スキャンは空白・予約区切り・行境界で再開する。
- `LexDiagnostic`（未知／不正トークン、不正な数値、文字列必須位置での不正な文字列リテラル）は、回復可能なトークンを落とさずに集める。

ユーザー入力由来の回復可能な字句問題に対して、`tokenize` は `Ok(TokenStream)` を返す。strict raw-scan 失敗は粗い回復トークンと mapped diagnostic へ縮退し、スコープスケルトン問題と曖昧性解消器診断は回復可能トークンを落とさず写像される。`Err(SpanBridgeError)` は、トークンまたは診断 span を登録済み bridge で写像できない場合に限る。

## テスト

主要シナリオ:

- 識別子とつづりを共有するユーザー記号が、アクティブ字句環境に対する最長一致で分類される。
- 複合予約トークン（`.{`、`.*`、`.=`、`...`）が単一トークンとして字句解析される。
- 引用符が general context ではユーザー記号文字として字句解析され、bounded uniform `StringRequired` context では `StringLiteral` を生む。注釈／演算子宣言引数位置の StringLiteral テストは parser-assisted lexing contract まで延期する。
- 不正なトークンが正しい `SourceRange` を持つ `ErrorRecovery` を送出し、スキャンが再開する。
- スコープスケルトン診断が mapped `LexingDiagnostic` として保持される。
- strict raw-scan 失敗は、現在の `LexError` が精密 span を持たないため、粗い `ErrorRecovery` トークン 1 つと `RawScan` 診断 1 つを出力する。
- 送出された各トークンの `span` が `source_id` に対する妥当な `SourceRange` であり、ソースマップを通じて元のつづりを再現する。

## 制約と前提

- このモジュールは字句解析器を統制する。最長一致・スコープスケルトン・曖昧性解消規則は所有しない。
- 字句解析器は複合予約トークンとアクティブユーザー記号を認識するが、意味的なセレクタ／名前空間の役割は認識しない。
- `StringLiteral` トークンは parser lexing context が文字列を要求するときだけ送出される。正確な文法位置認識は parser-assisted lexing contract で確定する。
- `TokenStream` キャッシュキーは前処理済みハッシュ、アクティブ字句環境 fingerprint、およびその実行で使った `ParserLexContext` / parser-assisted lexing plan の安定したエンコードである。
- すべてのトークンスパンは `span_bridge` を通じて生成された session `SourceRange` 値である。
