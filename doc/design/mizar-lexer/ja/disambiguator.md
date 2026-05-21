# Module: disambiguator

> Canonical language: English. English canonical version: [../en/disambiguator.md](../en/disambiguator.md).

## Purpose

This module converts raw lexer output into final tokens.

It owns context-sensitive longest-match over `LexemeRun` content. It consumes the active lexical environment, parser lexical context, and read-only scope view. It does not build those inputs.

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

現在の `disambiguate` は raw tokens を順に処理し、final tokens と recoverable diagnostics を持つ `TokenStream` を返します。

1. layout は捨てます。
2. `NumeralLike` は parser context が numeral を許可する場合だけ `Numeral` になります。許可されない場合は元の spelling を `ErrorRecovery` として出力し、`ParserContextRejectedCandidate` を報告します。
3. annotation marker `@[` は parser context が symbol を許可する場合だけ reserved symbol になります。それ以外の annotation marker や raw error token は `UnsupportedRawToken` diagnostic 付きの `ErrorRecovery` になります。
4. `LexemeRun` は内部 byte cursor で走査します。string-required context で cursor 位置が quote から始まる場合、通常の candidate selection より先に string literal を scan します。literal は同じ quote で閉じなければならず、escape できるのは `"`, `'`, `\` だけです。malformed string は run の残りを 1 個の recovery token として消費します。
5. それ以外の cursor 位置では、reserved compound symbols、active user symbols、reserved words、identifier syntax、numeral syntax、fallback recovery から candidates を集めます。string-literal candidates は string-required context でだけ許可されるため、この通常 candidate set より先に特別扱いします。

選択される candidate は、parser expectation と scope override rule を適用した後に残る、最長の valid candidate です。

candidate はまず length で比較し、同じ length の場合は priority で比較します。namespace-path context の `.` reserved symbol が最も高い priority を持ちます。次に identifier-shaped user symbol に対する scoped identifier override、active user symbols、reserved symbols、reserved words、identifiers、numerals の順です。parser context に許可された candidate がないものの、raw spelling としては候補が存在した場合は `ParserContextRejectedCandidate`、候補形そのものがなければ `NoValidTokenCandidate` を出します。

現在の実装には将来用の `AmbiguousUserSymbol` diagnostic code がありますが、同一 import 内の same-spelling overloads は lexical environment 内の deterministic candidate set として保持し、最終的な解決は後続 phase に委ねます。

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

- identifier-required position;
- symbol/operator-admitting expression position;
- string-literal-required argument position;
- namespace-path position;
- recovery mode after a syntax error.

Disambiguator は tokens と diagnostics を返すだけで、parser state を変更してはいけません。

Quote characters は raw scanning 中は `LexemeRun` の一部として残ります。Disambiguator は、`ParserLexContext` が current position を string-required と示す場合にのみ string literals を認識します。それ以外の位置では、quotes は ordinary symbol characters として user-symbol matching に参加します。

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

Disambiguation diagnostics include:

- no valid token candidate in a raw run;
- parser context rejects every candidate;
- malformed string literal in a string-required position;
- raw scanning から渡ってきた malformed annotation marker などの unsupported raw token.

可能な場合、disambiguator は元の spelling を持つ `ErrorRecovery` token を emit し、次の recoverable byte boundary から処理を再開します。

## Tests

Tests should cover:

- longest-match over punctuation-shaped user symbols;
- identifier-shaped user symbol vs scoped identifier override;
- reserved word emission;
- namespace-path context;
- string literal only in string-required positions;
- equal-length import tie breaking through lexical environment;
- recovery emits stable `Error` tokens and diagnostics.
