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
    pub diagnostics: Vec<LexDiagnostic>,
}

pub struct Token {
    pub kind: TokenKind,
    pub span: SourceRange,
    pub text: InternedText,
}

pub struct TokenizeRequest<'a> {
    pub preprocessed: &'a PreprocessedSource,
    pub environment: &'a ActiveLexicalEnvironment,
    pub parser_context: Option<&'a ParserLexContext>,
}

pub fn tokenize(request: TokenizeRequest<'_>, bridge: &SpanBridge) -> TokenStream;
```

`TokenKind`、`ParserLexContext`、`ParserLexMode`、`LexDiagnostic` は `mizar-lexer` から再エクスポートされる。フロントエンドの `Token` は、`span` が字句テキストのバイトスパンではなく session `SourceRange` である点のみが字句解析器の内部トークンと異なる。`tokenize` は各字句解析器スパンを `span_bridge` を通じて変換する。

`parser_context` は曖昧性解消器が必要とする狭い文法由来の信号（文字列必須位置と記号種フィルタ）を、任意のパーサー状態を晒さずに運ぶ。`None` のとき字句解析は既定モードで実行され、パーサー統合パスは Step 5 の途中で文脈を逐次供給する。

## 依存関係

- 内部: `preprocess`（字句テキストとマップ）、`lexical_env`（`ActiveLexicalEnvironment`）、`span_bridge`（字句バイト → `SourceRange`）、`parsing`（`TokenStream` を消費）。
- 外部: `mizar-lexer`（`scan_raw`、`build_scope_skeleton`、`ScopeLexView`、`disambiguate`、`lex`、`Token`、`TokenKind`、`ParserLexContext`、`LexDiagnostic`）、`mizar-session`（`SourceId`、`SourceRange`）。

このモジュールは構文解析が消費し、診断を通じて統制統合も消費する。

## データ構造

### トークンストリーム

`TokenStream` は 1 ファイルの完全かつソース忠実なトークン列である。各 `Token` は元のつづり（`text`）と session `SourceRange` を保持する。`TokenKind` には分割されなかった生単位の `LexemeRun`、`UserSymbol`、`ReservedWord`、`ReservedSymbol`、`Identifier`、`Numeral`、`StringLiteral`、`ErrorRecovery` が含まれる。`StringLiteral` は `parser_context` を通じて知らされる文法定義の文字列必須位置にのみ現れる。

### スコープ字句ビュー

スコープスケルトン事前走査（`build_scope_skeleton`）は、生の字句解析器出力に対する読み取り専用の `ScopeLexView` を生成し、曖昧性解消器がスコープ付き識別子の上書き規則に用いる。フロントエンドはこのビューを構築して曖昧性解消器に渡す。スコープ自体は構築せず、ビューは字句的な形のみを記録し、解決済み束縛は決して記録しない。

## アルゴリズム / ロジック

### 前処理済みソースのトークン化

1. 字句テキストを生スキャン（`scan_raw`）し、スパンを保持した `LexemeRun` にする。
2. 生トークンから `ScopeSkeleton` / `ScopeLexView` を構築し、スコープ付き識別子の上書きに用いる。
3. `disambiguate`（またはパーサー統合の `lex`）を実行し、次の順序で最長一致する。アクティブユーザー記号、予約特殊記号、予約語、識別子／数値規則、そして言語が要求する場合のパーサー期待／スコープ上書き。
4. `StringLiteral` トークンは `parser_context` が示す文字列必須位置でのみ認識する。それ以外では引用符は通常の記号文字のままである。
5. 結果の各字句解析器スパンを `span_bridge` を通じて `source_id` でスコープされた `SourceRange` へ変換する。
6. 字句解析器の診断を集め、`TokenStream` を返す。

複合予約トークン（`.{`、`.*`、`.=`、`...`）は字句解析器が認識する。`.` のセレクタ／名前空間の役割はパーサーと解決器に委ねる。字句解析器は定義済み性・適用可能性・オーバーロード選択を決して判断しない。

## エラー処理

字句解析器の回復はエンドツーエンドで保持される。

- 不正なスパンは診断のために元のソース範囲を保持したまま `TokenKind::ErrorRecovery` を送出する。
- スキャンは空白・予約区切り・行境界で再開する。
- `LexDiagnostic`（未知／不正トークン、不正な数値、文字列必須位置での不正な文字列リテラル）は、回復可能なトークンを落とさずに集める。

`tokenize` は常に `TokenStream` を返す。字句失敗は中断ではなく回復トークンと診断へ縮退するので、パーサーは引き続き回復を試みられる。

## テスト

主要シナリオ:

- 識別子とつづりを共有するユーザー記号が、アクティブ字句環境に対する最長一致で分類される。
- インポート順のタイブレークが、同じ長さの一致に対して期待されるユーザー記号を選ぶ。
- 複合予約トークン（`.{`、`.*`、`.=`、`...`）が単一トークンとして字句解析される。
- 引用符が文字列必須位置の外ではユーザー記号文字として、注釈／演算子宣言引数位置では `StringLiteral` として字句解析される。
- 不正なトークンが正しい `SourceRange` を持つ `ErrorRecovery` を送出し、スキャンが再開する。
- 送出された各トークンの `span` が `source_id` に対する妥当な `SourceRange` であり、ソースマップを通じて元のつづりを再現する。

## 制約と前提

- このモジュールは字句解析器を統制する。最長一致・スコープスケルトン・曖昧性解消規則は所有しない。
- 字句解析器は複合予約トークンとアクティブユーザー記号を認識するが、意味的なセレクタ／名前空間の役割は認識しない。
- `StringLiteral` トークンは文法定義の文字列必須位置でのみ送出される。
- `TokenStream` キャッシュキーは前処理済みハッシュとアクティブ字句環境 fingerprint である。
- すべてのトークンスパンは `span_bridge` を通じて生成された session `SourceRange` 値である。
