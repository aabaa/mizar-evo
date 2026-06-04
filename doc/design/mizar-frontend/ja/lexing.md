# モジュール: lexing

> 正本は英語です。英語版: [../en/lexing.md](../en/lexing.md)。

状態: 計画中。

## 目的

このモジュールは、フロントエンドパイプラインの Step 4（字句解析／曖昧性解消）を実装する。`mizar-lexer` の生スキャナー、スコープスケルトン事前走査、文脈依存の曖昧性解消器を駆動し、`PreprocessedSource` と `ActiveLexicalEnvironment` を、スパンが `mizar-session` の `SourceRange` 値である `TokenStream` へ変える。

配線とスパン橋渡しを所有するが、最長一致規則・スコープスケルトン構築・パーサー字句文脈の意味は所有しない（それらは `mizar-lexer` にある）。識別子が定義済みか、どのオーバーロードが適用されるかといった意味的判断は行わない。

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
    NoValidTokenCandidate {
        rejected_lexeme: Arc<str>,
        recovery: LexRecoveryHint,
    },
    ParserContextRejectedCandidate {
        mode: ParserLexMode,
        rejected_lexeme: Arc<str>,
        candidates: Vec<MappedRejectedTokenCandidate>,
        recovery: LexRecoveryHint,
    },
    MalformedStringLiteral {
        opening_quote: char,
        reason: MalformedStringLiteralReason,
        recovery: LexRecoveryHint,
    },
    UnsupportedRawToken {
        raw_kind: RawTokenKind,
        raw_lexeme: Arc<str>,
        recovery: LexRecoveryHint,
    },
}

pub struct MappedRejectedTokenCandidate {
    pub kind: TokenKind,
    pub lexeme: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
}

pub fn tokenize(
    request: TokenizeRequest<'_>,
    bridge: &SpanBridge,
) -> Result<TokenStream, SpanBridgeError>;
```

`TokenKind`、`ParserLexContext`、`ParserLexMode`、`LexError`、`LexRecoveryHint`、`MalformedStringLiteralReason`、`RawTokenKind`、`ScopeSkeletonDiagnosticCode`、`LexDiagnosticCode` は `mizar-lexer` から再エクスポートされる。`InternedText` は、最初の実装ではフロントエンドローカルな `Arc<str>` のつづりハンドルであり、グローバルなインターナは不要である。字句解析器の `String` lexeme は `Arc::<str>::from` で変換する。生の字句解析器診断構造体は入力としてのみ消費する。フロントエンドはそれらを即座に `LexingDiagnostic` へ変換し、公開診断は session 範囲を持ち、生の字句解析器バイトスパンを再公開しない。既存の構造化された字句解析器ペイロードは `LexingDiagnosticPayload` に保持し、入れ子スパンはすべて `span_bridge` を通じて `MappedRejectedTokenCandidate` へ写像する。

`parser_context` は、曖昧性解消器が必要とする狭い文法由来の信号を、任意のパーサー状態を晒さずに運ぶ。現在の `mizar-lexer` API は一様な文脈であり、注釈／演算子宣言引数のような、位置ごとに異なる文字列必須スパンはまだ表現しない。source → tokens の基盤では `ParserLexContext::general()` を使い、位置別の `StringLiteral` 認識は、パーサー支援字句解析の契約の確定後に追加する。

## 依存関係

- 内部: `preprocess`（字句テキストとマップ）、`lexical_env`（`ActiveLexicalEnvironment`）、`span_bridge`（字句バイトから `SourceRange`）、`parsing`（`TokenStream` を利用）。
- 外部: `mizar-lexer`（`scan_raw`、`build_scope_skeleton`、`ScopeLexView`、`disambiguate`、`lex`、`Token`、`TokenKind`、`ParserLexContext`、`LexError`、`LexDiagnostic`、`LexDiagnosticCode`、`ScopeSkeletonDiagnostic`、`ScopeSkeletonDiagnosticCode`）、`mizar-session`（`SourceId`、`SourceRange`、`SourceAnchor`）。

このモジュールは構文解析が利用し、診断を通じて統制統合も利用する。

## データ構造

### トークンストリーム

`TokenStream` は、1 ファイルの完全かつソースに忠実なトークン列である。各 `Token` は、元のつづり（`text`）と session の `SourceRange` を保持する。`TokenKind` には、分割されなかった生単位の `LexemeRun`、`UserSymbol`、`ReservedWord`、`ReservedSymbol`、`Identifier`、`Numeral`、`StringLiteral`、`ErrorRecovery` が含まれる。現在の一様な文脈 API では、明示的な `StringRequired` 実行のときだけ `StringLiteral` が現れる。文法上の位置に基づく文字列必須レジストリは、パーサー支援字句解析の契約の範囲である。

`LexingDiagnostic` は、Step 4 のフロントエンド側の写像済み診断ペイロードである。生スキャン失敗、スコープスケルトン診断、曖昧性解消器診断を表しうるが、統制層のために必ず session 座標の primary 範囲と副次アンカーを持つ。副次は `SourceAnchor` なので、複合／縮退した preprocess マッピングの点、生成、隣接コメントのアンカーを保持できる。生の字句解析器診断オブジェクトには字句解析器のバイトスパンが含まれるため、`LexingDiagnostic` は写像済みの code/message を持ち、生オブジェクト自体は保持しない。構造化ペイロードは、入れ子スパンを session 座標へ写像した後にだけ再公開する。

### スコープ字句ビュー

スコープスケルトン事前走査（`build_scope_skeleton`）は、生の字句解析器出力に対する読み取り専用の `ScopeLexView` を生成し、曖昧性解消器がスコープ付き識別子の上書き規則に用いる。フロントエンドはこのビューを構築して曖昧性解消器に渡す。スコープ自体は構築せず、ビューは字句的な形のみを記録し、解決済みの束縛は決して記録しない。

## アルゴリズム / ロジック

### 前処理済みソースのトークン化

1. 字句テキストを生スキャン（`scan_raw`）し、スパンを保持した `LexemeRun` にする。厳密な生スキャンが失敗した場合は、字句テキスト全体（空ならソース先頭のゼロ長範囲）を覆う粗い `ErrorRecovery` トークン 1 つと、写像済みの `LexingDiagnosticKind::RawScan` 診断 1 つを出力し、その実行ではスコープスケルトン構築と曖昧性解消をスキップする。現在の `mizar_lexer::LexError` はスパンや部分トークンのペイロードを持たないため、より細かい回復はフォローアップ契約として追跡する。
2. 生トークンから `ScopeSkeleton` / `ScopeLexView` を構築し、スコープ付き識別子の上書きに用いる。`ScopeSkeletonDiagnostic` は収集して写像する。
3. `disambiguate`（またはパーサー統合の `lex`）を実行し、次の順序で最長一致する。アクティブなユーザー記号、予約特殊記号、予約語、識別子／数値規則、そして言語が要求する場合のパーサー期待／スコープ上書きである。
4. `StringLiteral` トークンは、現在のパーサー字句文脈が文字列を要求するときだけ認識する。それ以外では、引用符は通常の記号文字のままである。注釈／演算子位置の文字列必須判定は、実 parser 契約まで延期する。
5. 結果の各字句解析器スパンを `span_bridge.lexical_span` を通じて、`source_id` でスコープされた第一の `SourceRange` へ変換し、副次の `SourceAnchor` は診断用に保持する。
6. 生スキャン、スコープスケルトン、字句解析器の診断を `LexingDiagnostic` として集める。スパン以外のペイロードデータをコピーし、入れ子の候補スパンを `MappedRejectedTokenCandidate` へ写像して、`TokenStream` を返す。

複合予約トークン（`.{`、`.*`、`.=`、`...`）は字句解析器が認識する。`.` のセレクタ／名前空間としての役割は、パーサーと解決器に委ねる。字句解析器は、定義済み性・適用可能性・オーバーロード選択を決して判断しない。

## エラー処理

字句解析器の回復は、エンドツーエンドで保持される。

- 不正なスパンは、診断のために元のソース範囲を保持したまま `TokenKind::ErrorRecovery` を送出する。
- スキャンは、空白・予約区切り・行境界で再開する。
- `LexDiagnostic`（未知／不正トークン、不正な数値、文字列必須位置での不正な文字列リテラル）は、回復可能なトークンを落とさずに集める。

ユーザー入力に由来する回復可能な字句問題に対して、`tokenize` は `Ok(TokenStream)` を返す。厳密な生スキャンの失敗は、粗い回復トークンと写像済み診断へ縮退し、スコープスケルトンの問題と曖昧性解消器の診断は、回復可能トークンを落とさずに写像される。`Err(SpanBridgeError)` は、トークンまたは診断スパンを登録済みの bridge で写像できない場合に限る。

## テスト

主要シナリオ:

- 識別子とつづりを共有するユーザー記号が、アクティブ字句環境に対する最長一致で分類される。
- 複合予約トークン（`.{`、`.*`、`.=`、`...`）が、単一トークンとして字句解析される。
- 引用符が、一般文脈ではユーザー記号文字として字句解析され、有界で一様な `StringRequired` 文脈では `StringLiteral` を生む。注釈／演算子宣言引数位置の StringLiteral テストは、パーサー支援字句解析の契約まで延期する。
- 不正なトークンが、正しい `SourceRange` を持つ `ErrorRecovery` を送出し、スキャンが再開する。
- スコープスケルトン診断が、写像済みの `LexingDiagnostic` として保持される。
- 棄却トークン候補を含む字句解析器ペイロードが、スパン以外のデータを保持しつつ、入れ子の候補スパンを session の `SourceRange` へ写像する。
- 厳密な生スキャンの失敗は、現在の `LexError` が精密なスパンを持たないため、粗い `ErrorRecovery` トークン 1 つと `RawScan` 診断 1 つを出力する。
- 送出された各トークンの `span` は、`source_id` に対する妥当な `SourceRange` であり、厳密なマッピングでは元のつづりを復元できる。粗い `ErrorRecovery` トークンと複合／縮退マッピングでは、利用可能な最善のソース範囲と副次アンカーだけを保持する場合がある。

## 制約と前提

- このモジュールは字句解析器を統制する。最長一致・スコープスケルトン・曖昧性解消の規則は所有しない。
- 字句解析器は、複合予約トークンとアクティブなユーザー記号を認識するが、意味的なセレクタ／名前空間としての役割は認識しない。
- `StringLiteral` トークンは、パーサー字句文脈が文字列を要求するときだけ送出される。正確な文法位置の認識は、パーサー支援字句解析の契約で確定する。
- `TokenStream` のキャッシュキーは、`PreprocessedSource.lexical_hash`、アクティブ字句環境のフィンガープリント、およびその実行で使った `ParserLexContext` ／パーサー支援字句解析プランの安定したエンコードである。
- すべてのトークンスパンは、`span_bridge` を通じて生成された session の `SourceRange` 値である。
