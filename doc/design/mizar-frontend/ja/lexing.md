# モジュール: lexing

> 正本は英語です。英語版: [../en/lexing.md](../en/lexing.md)。

状態: task 7〜9 は完了。パーサー支援字句解析のフォローアップは引き続き保留中。

## 目的

このモジュールは、フロントエンドパイプラインの Step 4（字句解析／曖昧性解消）を実装する。task 7/8/9 の実装は、`mizar-lexer` の生スキャナー、スコープスケルトン事前走査、曖昧性解消器を駆動し、`PreprocessedSource` を session スパン付きの parser-facing `TokenStream` へ変え、回復可能な字句解析器診断と `ErrorRecovery` トークンを同じエントリポイントで保持する。

配線とスパン橋渡しを所有するが、最長一致規則・スコープスケルトン構築・パーサー字句文脈の意味は所有しない（それらは `mizar-lexer` にある）。識別子が定義済みか、どのオーバーロードが適用されるかといった意味的判断は行わない。

[architecture/ja/02.source_and_frontend.md](../../architecture/ja/02.source_and_frontend.md) の「ステップ 4: 字句解析」「字句解析は生のスキャンを先に行い、その後で文脈に応じて確定する」「ドットの扱いは字句解析器・構文解析器・リゾルバに分担される」「文字列リテラルは字句解析器が完全にトークン化する」を参照。

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
    pub text: InternedText,
    pub span: SourceRange,
}

pub type InternedText = Arc<str>;

pub struct TokenizeRequest<'a> {
    pub preprocessed: &'a PreprocessedSource,
    pub environment: &'a ActiveLexicalEnvironment,
    pub parser_context: ParserLexContext,
}

impl<'a> TokenizeRequest<'a> {
    pub fn new(
        preprocessed: &'a PreprocessedSource,
        environment: &'a ActiveLexicalEnvironment,
        parser_context: ParserLexContext,
    ) -> Self;
}

pub struct LexingDiagnostic {
    pub kind: LexingDiagnosticKind,
    pub message: InternedText,
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
        rejected_lexeme: InternedText,
        recovery: LexRecoveryHint,
    },
    ParserContextRejectedCandidate {
        mode: ParserLexMode,
        rejected_lexeme: InternedText,
        candidates: Vec<LexingRejectedTokenCandidate>,
        recovery: LexRecoveryHint,
    },
    MalformedStringLiteral {
        opening_quote: char,
        reason: MalformedStringLiteralReason,
        recovery: LexRecoveryHint,
    },
    UnsupportedRawToken {
        raw_kind: RawTokenKind,
        raw_lexeme: InternedText,
        recovery: LexRecoveryHint,
    },
    UnsupportedLexerPayload,
}

pub struct LexingRejectedTokenCandidate {
    pub kind: TokenKind,
    pub text: InternedText,
    pub span: SourceRange,
    pub secondary: Vec<SourceAnchor>,
}

pub struct ScopeView {
    pub source_id: SourceId,
    pub frames: Vec<ScopeFrame>,
    pub blocks: Vec<ScopeBlock>,
    pub statements: Vec<ScopeStatement>,
}

impl TokenStream {
    pub fn tokens(&self) -> &[Token];
    pub fn diagnostics(&self) -> &[LexingDiagnostic];
    pub fn scope_view(&self) -> &ScopeView;
    pub fn into_parts(self) -> (Vec<Token>, ScopeView, Vec<LexingDiagnostic>);
}

impl ScopeView {
    pub fn empty(source_id: SourceId) -> Self;
    pub fn binding_overrides_symbol(&self, spelling: &str, position: usize) -> bool;
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

`TokenKind`、`ParserLexContext`、`ParserLexMode`、`LexRecoveryHint`、`MalformedStringLiteralReason`、`RawTokenKind`、`BindingShapeKind`、`LexicalBlockKind`、`LexicalStatementKind`、`ScopeSkeletonDiagnosticCode`、`LexDiagnosticCode` は `mizar-lexer` から再エクスポートされる。`InternedText` は、最初の実装ではフロントエンドローカルな `Arc<str>` のつづりハンドルであり、グローバルなインターナは不要である。字句解析器の `String` lexeme は `Arc::<str>::from` で変換する。生の字句解析器診断構造体は入力としてのみ消費する。フロントエンドはそれらを即座に `LexingDiagnostic` へ変換し、公開診断は session 範囲を持ち、生の字句解析器バイトスパンを再公開しない。構造化された字句解析器ペイロード variant はスパン以外のデータをコピーし、棄却候補は session スパンと副次アンカーを持つフロントエンド所有の `LexingRejectedTokenCandidate` として表す。この frontend がまだ写像方法を知らない将来の lexer payload variant は、`None` に暗黙変換せず、明示的に `UnsupportedLexerPayload` として表す。

`parser_context` は `mizar_lexer::disambiguate` へ渡され、同じ frontend request から general、namespace-path、string-required、recovery-mode の token stream を作れるようにする。`environment` は有効なユーザー記号 index を提供する。インポート済み記号は token 分類に参加するが、公開用の字句的な `ScopeView` には混ざらない。

## 依存関係

- 内部: `preprocess`（字句テキストとマップ）、`lexical_env`（`ActiveLexicalEnvironment`）、`span_bridge`（字句バイトから `SourceRange`）、`parsing`（`TokenStream` を利用）。
- 外部: `mizar-lexer`（`scan_raw`、`build_scope_skeleton`、`disambiguate`、`TokenKind`、`ParserLexContext`、`LexDiagnostic`、`LexDiagnosticCode`、構造化された字句解析器ペイロード enum、`ScopeSkeletonDiagnostic`、`ScopeSkeletonDiagnosticCode`、スコープのブロック／文／束縛形状 enum）、`mizar-session`（`SourceId`、`SourceRange`、`SourceAnchor`）。

このモジュールは構文解析が利用し、診断を通じて統制統合も利用する。

## データ構造

### トークンストリーム

`TokenStream` は、1 ファイルのソースに忠実なトークン列であり、その実行で使った parser lexing context の下にある。成功した tokenization では、最終的な parser-facing の `UserSymbol`、`ReservedWord`、`ReservedSymbol`、`Identifier`、`Numeral`、`StringLiteral` 分類を公開し、回復可能な字句解析器または曖昧性解消器診断が入力を消費した位置には `TokenKind::ErrorRecovery` トークンを挟む。厳密な生スキャン失敗では代わりに字句テキスト全体に対する粗い `TokenKind::ErrorRecovery` を 1 つ出力する。各 `Token` は、元のつづり（`text`）と session の `SourceRange` を保持する。

`LexingDiagnostic` は、Step 4 のフロントエンド側の写像済み診断ペイロードである。生スキャン失敗、スコープスケルトン診断、字句解析器の曖昧性解消診断を表す。統制層のために必ず session 座標の primary 範囲と副次アンカーを持つ。副次は `SourceAnchor` なので、複合／縮退した preprocess マッピングの点、生成、隣接コメントのアンカーを保持できる。生の字句解析器診断オブジェクトには字句解析器のバイトスパンが含まれるため、`LexingDiagnostic` は写像済みの code/message とフロントエンド所有の構造化 payload を持ち、生オブジェクト自体は保持しない。

### スコープ字句ビュー

スコープスケルトン事前走査（`build_scope_skeleton`）は、読み取り専用の `ScopeLexView` を生成し、曖昧性解消器がスコープ付き識別子の上書き規則に用いる。フロントエンドは、まず初回の曖昧性解消のために raw skeleton を作り、その後、文字列と非識別子形のユーザー記号が公開 scope 診断では不活性になるように、最終 token 形状から文脈化した skeleton を作り直す。フロントエンドは、その contextual skeleton を公開用の session スパン付き `ScopeView` へ写像し、後続の検査と診断に使えるようにする。スコープ自体は構築せず、ビューは字句的な形のみを記録し、解決済みの束縛は決して記録しない。

## アルゴリズム / ロジック

### 前処理済みソースのトークン化

1. 字句テキストを生スキャン（`scan_raw`）し、スパンを保持した生単位にする。厳密な生スキャンが失敗した場合は、字句テキスト全体（空ならソース先頭のゼロ長範囲）を覆う粗い `ErrorRecovery` トークン 1 つと、写像済みの `LexingDiagnosticKind::RawScan` 診断 1 つを出力し、その実行ではスコープスケルトン構築と曖昧性解消をスキップする。現在の `mizar_lexer::LexError` はスパンや部分トークンのペイロードを持たないため、より細かい回復はフォローアップ契約として追跡する。
2. 生トークンから初回の `ScopeSkeleton` / `ScopeLexView` を構築し、生 token stream、アクティブ字句環境、現在の `ParserLexContext` を渡して `disambiguate` を一度実行する。
3. 初回の最終 token 形状から scope skeleton を作り直す。このとき `StringLiteral`、`ErrorRecovery`、数値、非識別子形のユーザー記号は scope に対して不活性として扱う。その contextual skeleton を使ってもう一度 `disambiguate` を実行し、文字列内容やユーザー記号の綴りを scope 構文として扱わずに、スコープ付き識別子の上書きを利用できるようにする。公開用の scope skeleton は、最終 token 形状から構築する。
4. 曖昧性解消器は、アクティブなユーザー記号、予約特殊記号、予約語、識別子／数値規則、そして言語が要求する場合のパーサー期待／スコープ上書きの順で最長一致を適用する。
5. 最終的な各字句解析器トークンを `span_bridge.lexical_span` で写像し、session スパン付きの frontend `Token` にする。スコープスケルトンの frame、block、statement、束縛形状も公開 `ScopeView` へ写像する。
6. 現在のパーサー字句文脈が文字列を要求するときだけ `StringLiteral` トークンを認識する。選択された byte span だけに対する位置依存文脈は、パーサー支援字句解析の契約まで延期する。
7. 字句解析器の診断を `LexingDiagnostic` として集め、スパン以外のペイロードデータをコピーし、入れ子の棄却候補スパンをフロントエンド側の payload 構造へ写像する。

複合予約トークン（`.{`、`.*`、`.=`、`...`）は字句解析器が認識する。`.` のセレクタ／名前空間としての役割は、パーサーと解決器に委ねる。字句解析器は、定義済み性・適用可能性・オーバーロード選択を決して判断しない。

## エラー処理

lexing wrapper は、回復可能な生スキャン、スコープスケルトン、および現在の曖昧性解消器の問題を保持する。

- 厳密な生スキャンの致命的失敗は、利用できる最善のソース範囲を持つ粗い `TokenKind::ErrorRecovery` 1 つと、`RawScan` 診断を出力する。
- `ScopeSkeletonDiagnostic` は、スパンを持つ生の診断構造体を `TokenStream` に保持せず、フロントエンド診断へ写像する。
- 現在の `LexDiagnostic` は、フロントエンド所有の payload と写像済みの入れ子の棄却候補スパンを持つ `LexingDiagnosticKind::Lexer` へ写像する。

ユーザー入力に由来する回復可能な字句問題に対して、`tokenize` は `Ok(TokenStream)` を返す。厳密な生スキャンの失敗は、粗い回復トークンと写像済み診断へ縮退し、スコープスケルトンと曖昧性解消器の問題は、回復可能なトークンを落とさずに写像される。`Err(SpanBridgeError)` は、トークン、スコープ形状、診断スパンを登録済みの bridge で写像できない場合に限る。task 9 では同じ境界を保ち、残りの回復パススルーの網羅とフォローアップケースを追加する。

## テスト

実装済み task 7/8/9 のシナリオ:

- 生スキャンと曖昧性解消が最終 token のスパンを保持する。コメント除去などの preprocess 写像を通じたスパンも含む。
- 公開 `ScopeView` が、解決済み／インポート済み束縛なしで、字句ブロック／文の形とローカル束縛形状を反映する。
- スコープスケルトン診断が、写像済みの `LexingDiagnostic` として保持される。
- 送出された各トークンの `span` は、`source_id` に対する妥当な `SourceRange` である。
- アクティブなユーザー記号とスコープ付き識別子の上書きが、最終 token 分類に反映される。
- 複合予約トークンが単一の最終 token のまま保持される。
- 一様な `StringRequired` 文脈では `StringLiteral` を出力し、general 文脈では棄却された文字列候補に対する写像済み字句解析器診断を出力する。
- 字句解析器診断 payload が、スパン以外のデータと写像済みの入れ子候補スパンを保持する。
- 不正 lexeme と未対応 raw-token ケースが、後続トークンの送出を妨げず、写像済みの `ErrorRecovery` トークンを出力する。
- 専用の不正数値 lexer 診断が追加されるまで、parser 文脈で棄却された数値が写像済みの棄却候補 payload を保持する。
- 回復可能な字句解析器診断が併存する場合でも、スコープ診断が曖昧性解消後に写像済みスパン付きで保持される。

## 制約と前提

- このモジュールは字句解析器を統制する。最長一致・スコープスケルトン・曖昧性解消の規則は所有しない。
- 字句解析器は、複合予約トークンとアクティブなユーザー記号を認識するが、意味的なセレクタ／名前空間としての役割は認識しない。
- `StringLiteral` トークンは、パーサー字句文脈が文字列を要求するときだけ送出される。正確な文法位置の認識は、パーサー支援字句解析の契約で確定する。
- トークンストリームをキャッシュする場合、キャッシュキーは `PreprocessedSource.lexical_hash`、アクティブ字句環境のフィンガープリント、およびその実行で使った `ParserLexContext` ／パーサー支援字句解析プランの安定したエンコードである。
- すべてのトークンスパンは、`span_bridge` を通じて生成された session の `SourceRange` 値である。
