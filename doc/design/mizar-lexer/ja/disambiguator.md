# Module: disambiguator

> Canonical language: English. English canonical version: [../en/disambiguator.md](../en/disambiguator.md).

## Purpose

このモジュールは、生の字句解析器(lexer)の出力を最終トークンに変換します。

`LexemeRun` の内容に対する文脈依存(context-sensitive)・最長一致(longest-match)の処理は、このモジュールの責務です。入力としてアクティブ字句環境(active lexical environment)、パーサーの字句コンテキスト(parser lexical context)、読み取り専用のスコープビュー(scope view)を受け取りますが、それらを構築することはありません。

## Public API

実装済み API:

```rust
pub struct TokenStream {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<LexDiagnostic>,
}

pub struct LexDiagnostic {
    pub code: LexDiagnosticCode,
    pub message: String,
    pub span: SourceSpan,
    pub payload: LexDiagnosticPayload,
}

pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: SourceSpan,
}

#[non_exhaustive]
pub enum TokenKind {
    Identifier,
    ReservedWord,
    ReservedSymbol,
    Numeral,
    LexemeRun,
    UserSymbol,
    StringLiteral,
    ErrorRecovery,
}

pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream;

pub fn disambiguate_with_local_declarations(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    local_declarations: &LocalLexicalDeclarations,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream;
```

## Candidate Selection

現在の `disambiguate` は生トークンを順に処理し、最終トークンと回復可能(recoverable)な診断(diagnostic)を持つ `TokenStream` を返します。`disambiguate_with_local_declarations` は同じアルゴリズムを、ソース位置対応の現在モジュール宣言レイヤーとともに使います。`disambiguate` は、ローカル宣言イベントを持たない import-seeded environment 向けの互換 entry point です。

1. レイアウト(空白類)は捨てます。
2. `NumeralLike` は、パーサーコンテキストが数字を許可する場合だけ `Numeral` になります。許可されない場合は元の綴り(spelling)を `ErrorRecovery` として出力し、`ParserContextRejectedCandidate` を報告します。
3. 注釈マーカー(annotation marker) `@[` は、パーサーコンテキストが記号を許可する場合だけ予約記号になります。それ以外の注釈マーカーや生のエラートークンは `UnsupportedRawToken` 診断付きの `ErrorRecovery` になります。
4. `LexemeRun` は内部のバイトカーソルで走査します。文字列必須(string-required)コンテキストでカーソル位置が引用符(quote)から始まる場合、通常の候補選択より先に文字列リテラルを走査します。リテラルは同じ引用符で閉じなければならず、エスケープできるのは `"`, `'`, `\` だけです。不正な文字列は、ランの残りを 1 個の回復トークンとして消費します。
5. それ以外のカーソル位置では、予約複合記号、アクティブなユーザーシンボル、予約語、識別子構文、数字構文、フォールバックの回復処理から候補を集めます。文字列リテラル候補は文字列必須コンテキストでだけ許可されるため、この通常の候補集合より先に特別扱いします。

選択される候補は、パーサーの期待(parser expectation)とスコープによる上書き(override)規則を適用した後に残る、最長の有効な候補です。

候補はまず長さで比較し、長さが同じ場合は優先度で比較します。名前空間パス(namespace-path)コンテキストの `.` 予約記号が最も高い優先度を持ちます。次に、識別子の形をしたユーザーシンボルに対するスコープ付き識別子の上書き、アクティブなユーザーシンボル、予約記号、予約語、識別子、数字の順です。パーサーコンテキストに許可された候補がないものの、生の綴りとしては候補が存在した場合は `ParserContextRejectedCandidate` を、候補の形そのものがなければ `NoValidTokenCandidate` を出します。

現在の実装には将来用の `AmbiguousUserSymbol` 診断コードがあります。ただし、同一インポート内の同綴りオーバーロード(overload)は字句環境内の決定的な候補集合として保持し、最終的な解決は後続フェーズに委ねます。

## Final Token Spans

すべての最終 `Token` は、そのトークンの綴りが由来するソースのバイトスパン(byte span)を保持しなければなりません。生スキャンはすでに `RawToken` にスパンを保持しているため、曖昧性解消はその情報を落としてはいけません。

`Token` が保存するのはバイトスパンだけで、行/列の座標は保存しません。行/列は、スパンが指す同じテキストから `SourceLineIndex` で導出するか、前処理によってオフセットが変わった場合はソースマップ情報とセッション層(session layer)の `LineMap` を組み合わせて、元の読み込み済みソースから導出します。

スパンの不変条件(invariant)は以下のとおりです。

- 連続したソーステキストから出力されたトークンでは `token.lexeme == source[token.span.start..token.span.end]` が成り立つ;
- 出力されるすべてのトークンで `token.span.start < token.span.end` が成り立つ;
- 1 個の `RawToken` から出力されたトークン群は、その生トークンのスパン内で順序を保ち、互いに重ならない;
- レイアウトの生トークンは捨てられるため、最終トークンのスパンを生成しない;
- `ErrorRecovery` トークンは、回復トークンが不正な綴り全体を覆う場合、原因となった診断と同じスパンを使う。

`NumeralLike` から `Numeral` へ、`@[` から `ReservedSymbol` へのように、生トークンと最終トークンが 1 対 1 に対応する場合、最終トークンのスパンは生トークンからそのままコピーします。

`LexemeRun` の内部を分割してトークンを出力する場合、曖昧性解消器は生トークンの開始オフセットと内部のバイトカーソルからスパンを計算します。

```rust
SourceSpan {
    start: raw_token.span.start + cursor,
    end: raw_token.span.start + cursor + candidate_len,
}
```

`StringRequired` コンテキストで認識される文字列リテラルも同じ規則を使います。不正な文字列リテラルは生ランの残りを 1 個の `ErrorRecovery` トークンとして消費し、そのスパンは開き引用符から生トークンの末尾までです。

利便用 API である `lex` と `disambiguate_reserved_shell` も、スパン付きの `Token` を返します。これらは文脈自由(context-free)ですが、`scan_raw` が生成したソース位置は保持します。

## Identifier and Symbol Override

ある綴りが識別子の形をしており、同時にアクティブなユーザーシンボルでもある場合、以下の順に判断します。

1. `ScopeLexView` に、このソース位置でスコープ付き識別子束縛が記号を上書きするかを問い合わせる。
2. `ParserLexContext` に、この文法位置で識別子が合法かを問い合わせる。
3. 上書きがあり、かつパーサーコンテキストが識別子を許す場合だけ `Identifier` を出力する。
4. それ以外の場合はアクティブな `UserSymbol` 候補を出力する。

未定義の名前であっても、この段階では `Identifier` として出力します。未定義名の診断は、後続の名前解決(name resolution)が報告します。

## Parser Context

`ParserLexContext` は字句上の期待だけを表す狭い入力です。任意の構文ロジックへコールバックするパーサーフックではありません。

代表的なモードは以下のとおりです。

- 識別子が必須の位置;
- 記号/演算子を許す式の位置;
- アクティブなユーザーシンボル種別(kind)の部分集合に制限された記号/演算子の位置;
- 文字列リテラルが必須の引数位置;
- 名前空間パスの位置;
- 構文エラー後の回復モード.

曖昧性解消器はトークンと診断を返すだけで、パーサーの状態を変更してはいけません。

引用符は、生スキャン中は `LexemeRun` の一部として残ります。曖昧性解消器は、`ParserLexContext` が現在位置を文字列必須と示す場合にのみ文字列リテラルを認識します。それ以外の位置では、引用符は通常の記号文字としてユーザーシンボルの照合に参加します。

`ParserLexContext` は `UserSymbolKindSet` も保持できます。`General`、`Symbolic`、`Recovery` の各モードは、その綴りのアクティブ候補のうち少なくとも一つが許可された種別を持つ場合にだけ、ユーザーシンボルを許可します。これにより、パーサー側の統合は、アクティブ字句環境を再構築せずに、述語のみ・functorのみ・モードのみといった、カテゴリ別の字句ビューを要求できます。最終 `UserSymbol` トークンは引き続き綴りとスパンだけを保持します。種別・アリティ(arity)・オーバーロード・由来(provenance)の詳細が必要な下流フェーズは、アクティブ字句環境から候補リストを復元します。

disambiguation が消費するのは lexical context と parser-facing context だけです。semantic resolution、overload root selection、type fact、registration firing、proof information を token classification や diagnostics に埋め込んではなりません。parser context filter は tokenization に影響するため frontend cache footprint に参加しますが、semantic dependency slice ではありません。

実装済みのモードごとの許可関係は以下のとおりです。

| Mode | Identifiers | Reserved Words | Reserved Symbols | User Symbols | Numerals | Strings |
|---|---|---|---|---|---|---|
| `General` | yes | yes | yes | yes | yes | no |
| `IdentifierRequired` | yes | no | no | no | no | no |
| `Symbolic` | yes | yes | yes | yes | yes | no |
| `StringRequired` | no | no | no | no | no | yes |
| `NamespacePath` | yes | no | only `.` / `..` | no | no | no |
| `Recovery` | yes | yes | yes | yes | yes | no |

## Error Handling

曖昧性解消の診断には以下があります。

- 生ラン内に有効なトークン候補がない;
- パーサーコンテキストがすべての候補を拒否した;
- 文字列必須位置に不正な文字列リテラルがある;
- 生スキャンから渡ってきた、不正な注釈マーカーなどの未対応の生トークン.

可能な場合、曖昧性解消器は元の綴りを持つ `ErrorRecovery` トークンを出力し、次の回復可能なバイト境界から処理を再開します。

診断コードとバイトスパンは安定した契約です。人間向けの `message` テキストは暫定のままです。ツールは機械可読な詳細として `LexDiagnosticPayload` を使います。

| Code | Payload |
|---|---|
| `NoValidTokenCandidate` | 拒否された綴りと `EmitErrorRecoveryToken` の回復ヒント |
| `ParserContextRejectedCandidate` | パーサーモード、拒否された綴り、拒否された候補のトークン種別/綴り/スパン、`EmitErrorRecoveryToken` の回復ヒント |
| `MalformedStringLiteral` | 開き引用符、不正文字列の理由、回復ヒント |
| `UnsupportedRawToken` | 生トークン種別、生の綴り、回復ヒント |

安定した構造化ペイロードが重要な場合、フィクスチャは任意で `diagnostic_payloads` の要約を表明できます。人間向けの診断メッセージは表明してはいけません。

## Tests

テストでは以下を確認します。

- 記号の形をしたユーザーシンボルの最長一致;
- 識別子の形をしたユーザーシンボルと、スコープ付き識別子の上書きの切り替え;
- 生トークンと 1 対 1 に対応する最終トークン、および `LexemeRun` から分割された最終トークンのスパン;
- `ErrorRecovery` トークンのスパンが、対応する診断のスパンと一致すること;
- 予約語の出力;
- 名前空間パスのコンテキスト;
- アクティブなユーザーシンボル種別の部分集合だけを許すパーサーコンテキスト;
- 文字列リテラルが文字列必須位置でだけ認識されること;
- 字句環境を通じた、同じ長さのインポート間のタイブレーク;
- 回復処理が、安定した `ErrorRecovery` トークンと診断を出力すること。
