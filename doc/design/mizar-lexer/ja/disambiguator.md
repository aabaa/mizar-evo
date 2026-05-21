# Module: disambiguator

> Canonical language: English. English canonical version: [../en/disambiguator.md](../en/disambiguator.md).

## Purpose

この module は raw lexer output を final token に変換します。

`LexemeRun` の内容に対する context-sensitive longest-match は、この module の責務です。入力として active lexical environment、parser lexical context、read-only scope view を受け取りますが、それらを構築することはありません。

## Public API

Implemented API:

```rust
pub struct TokenStream {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<LexDiagnostic>,
}

pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: SourceSpan,
}

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
```

## Candidate Selection

現在の `disambiguate` は raw token を順に処理し、final token と recoverable diagnostic を持つ `TokenStream` を返します。

1. layout は捨てます。
2. `NumeralLike` は parser context が numeral を許可する場合だけ `Numeral` になります。許可されない場合は元の spelling を `ErrorRecovery` として出力し、`ParserContextRejectedCandidate` を報告します。
3. annotation marker `@[` は parser context が symbol を許可する場合だけ reserved symbol になります。それ以外の annotation marker や raw error token は `UnsupportedRawToken` diagnostic 付きの `ErrorRecovery` になります。
4. `LexemeRun` は内部 byte cursor で走査します。string-required context で cursor 位置が quote から始まる場合、通常の candidate selection より先に string literal を scan します。literal は同じ quote で閉じなければならず、escape できるのは `"`, `'`, `\` だけです。malformed string は run の残りを 1 個の recovery token として消費します。
5. それ以外の cursor 位置では、reserved compound symbol、active user symbol、reserved word、identifier syntax、numeral syntax、fallback recovery から candidate を集めます。string-literal candidate は string-required context でだけ許可されるため、この通常 candidate set より先に特別扱いします。

選択される candidate は、parser expectation と scope override rule を適用した後に残る、最長の valid candidate です。

candidate はまず length で比較し、同じ length の場合は priority で比較します。namespace-path context の `.` reserved symbol が最も高い priority を持ちます。次に identifier-shaped user symbol に対する scoped identifier override、active user symbol、reserved symbol、reserved word、identifier、numeral の順です。parser context に許可された candidate がないものの、raw spelling としては候補が存在した場合は `ParserContextRejectedCandidate`、候補形そのものがなければ `NoValidTokenCandidate` を出します。

現在の実装には将来用の `AmbiguousUserSymbol` diagnostic code があります。ただし、同一 import 内の same-spelling overload は lexical environment 内の deterministic candidate set として保持し、最終的な解決は後続 phase に委ねます。

## Final Token Spans

すべての final `Token` は、その token の spelling が由来する source byte span を保持しなければなりません。Raw scanning はすでに `RawToken` に span を保持しているため、disambiguation はその情報を落としてはいけません。

`Token` が保存するのは byte span だけで、line/column coordinate は保存しません。Line/column は、span が指す同じ text から `SourceLineIndex` で derive するか、preprocessing により offset が変わった場合は source-map information と session layer の `LineMap` を組み合わせて original loaded source から derive します。

span の不変条件は以下です。

- contiguous な source text から emit された token では `token.lexeme == source[token.span.start..token.span.end]` が成り立つ;
- emit されるすべての token で `token.span.start < token.span.end` が成り立つ;
- 1 個の `RawToken` から emit された tokens は、その raw token span の内側で順序を保ち、互いに重ならない;
- layout raw token は捨てられるため、final token span を生成しない;
- `ErrorRecovery` token は、recovery token が malformed spelling 全体を覆う場合、原因となった diagnostic と同じ span を使う。

`NumeralLike` から `Numeral`、`@[` から `ReservedSymbol` のように raw token と final token が 1 対 1 で対応する場合、final token span は raw token からそのままコピーします。

`LexemeRun` の内部を分割して token を emit する場合、disambiguator は raw token の start offset と内部 byte cursor から span を計算します。

```rust
SourceSpan {
    start: raw_token.span.start + cursor,
    end: raw_token.span.start + cursor + candidate_len,
}
```

`StringRequired` context で認識される string literal も同じ規則を使います。malformed string literal は raw run の残りを 1 個の `ErrorRecovery` token として消費し、その span は opening quote から raw token end までです。

convenience API である `lex` と `disambiguate_reserved_shell` も span 付き `Token` を返します。これらは context-free ですが、`scan_raw` が生成した source location は保持します。

## Identifier and Symbol Override

ある spelling が identifier-shaped であり、同時に active user symbol でもある場合、以下の順に判断します。

1. `ScopeLexView` に、この source position で scoped identifier binding が symbol を override するか問い合わせる。
2. `ParserLexContext` に、この grammar position で identifier が legal か問い合わせる。
3. override があり、かつ parser context が identifier を許す場合だけ `Identifier` を emit する。
4. それ以外の場合は active `UserSymbol` candidate を emit する。

未定義の名前であっても、この段階では `Identifier` として emit します。undefined-name diagnostics は後続の name resolution が報告します。

## Parser Context

`ParserLexContext` は lexical expectation だけを表す狭い input です。任意の syntax logic に callback する parser hook ではありません。

代表的な mode は以下です。

- identifier が必須の位置;
- symbol/operator を許す expression 位置;
- string literal が必須の argument 位置;
- namespace path の位置;
- syntax error 後の recovery mode.

Disambiguator は tokens と diagnostics を返すだけで、parser state を変更してはいけません。

Quote character は raw scanning 中は `LexemeRun` の一部として残ります。Disambiguator は、`ParserLexContext` が current position を string-required と示す場合にのみ string literal を認識します。それ以外の位置では、quote は ordinary symbol character として user-symbol matching に参加します。

実装済み mode ごとの許可関係は以下の通りです。

| Mode | Identifiers | Reserved Words | Reserved Symbols | User Symbols | Numerals | Strings |
|---|---|---|---|---|---|---|
| `General` | yes | yes | yes | yes | yes | no |
| `IdentifierRequired` | yes | no | no | no | no | no |
| `Symbolic` | yes | yes | yes | yes | yes | no |
| `StringRequired` | no | no | no | no | no | yes |
| `NamespacePath` | yes | no | only `.` | no | no | no |
| `Recovery` | yes | yes | yes | yes | yes | no |

## Error Handling

Disambiguation diagnostic には以下があります。

- raw run 内に valid token candidate がない;
- parser context がすべての candidate を拒否した;
- string-required position に malformed string literal がある;
- raw scanning から渡ってきた malformed annotation marker などの unsupported raw token.

可能な場合、disambiguator は元の spelling を持つ `ErrorRecovery` token を emit し、次の recoverable byte boundary から処理を再開します。

## Tests

テストでは以下を確認します。

- punctuation-shaped user symbol の longest-match;
- identifier-shaped user symbol と scoped identifier override の切り替え;
- raw token と 1 対 1 に対応する final token、および `LexemeRun` から分割された final token の span;
- `ErrorRecovery` token の span が対応する diagnostic span と一致すること;
- reserved word emission;
- namespace-path context;
- string literal が string-required position でだけ認識されること;
- lexical environment を通じた equal-length import tie breaking;
- recovery が stable な `ErrorRecovery` token と diagnostic を emit すること。
