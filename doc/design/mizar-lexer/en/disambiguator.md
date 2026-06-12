# Module: disambiguator

> Canonical language: English. Japanese companion: [../ja/disambiguator.md](../ja/disambiguator.md).

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
```

## Candidate Selection

The implemented `disambiguate` algorithm processes raw tokens in order and emits a `TokenStream` containing final tokens plus recoverable diagnostics.

1. Layout is skipped.
2. `NumeralLike` becomes `Numeral` only when the parser context admits numerals; otherwise the original spelling becomes `ErrorRecovery` with `ParserContextRejectedCandidate`.
3. Annotation marker `@[` becomes a reserved symbol only when the parser context admits symbols. Other annotation markers and raw error tokens become `ErrorRecovery` with `UnsupportedRawToken`.
4. Each `LexemeRun` is scanned with an internal byte cursor. In string-required context, a leading quote starts string-literal scanning before normal candidate selection. The literal must close with the same quote and may only escape `"`, `'`, and `\`; malformed strings consume the rest of the run as one recovery token.
5. At every other cursor position, the disambiguator gathers candidates from reserved compound symbols, active user symbols, reserved words, identifier syntax, numeral syntax, and fallback recovery. String-literal candidates are intentionally handled before this normal candidate set because they are admitted only in string-required context.

The selected candidate is the longest valid candidate after parser expectation and scope override rules are applied.

Candidate priority breaks ties between candidates of equal length. Namespace-path `.` as a reserved symbol has the highest priority, scoped identifier override of an identifier-shaped user symbol comes next, then active user symbols, reserved symbols, reserved words, identifiers, and numerals. If no admitted candidate exists but at least one raw candidate shape was present, the diagnostic is `ParserContextRejectedCandidate`; otherwise it is `NoValidTokenCandidate`.

The current implementation has an `AmbiguousUserSymbol` diagnostic code reserved for future cases, but equal-spelling same-import overloads remain a deterministic candidate set in the lexical environment and are intentionally left for later resolution phases.

## Final Token Spans

Every final `Token` must carry the byte span of the source spelling that produced it. Raw scanning already preserves spans on `RawToken`; disambiguation must not discard that information.

`Token` stores only the byte span, not line and column coordinates. Line and column are derived later from the same text addressed by the span through `SourceLineIndex`, or from the original loaded source through the session layer's `LineMap` plus source-map information when preprocessing changed offsets.

The span invariant is:

- `token.lexeme == source[token.span.start..token.span.end]` for tokens emitted from contiguous source text;
- `token.span.start < token.span.end` for every emitted token;
- tokens emitted from a single `RawToken` are ordered and non-overlapping within the raw token span;
- layout raw tokens are skipped and therefore do not produce final token spans;
- `ErrorRecovery` tokens use the same span as the diagnostic that caused them whenever the recovery token covers the malformed spelling.

For raw tokens that are mapped one-to-one, such as `NumeralLike` to `Numeral` or `@[` to `ReservedSymbol`, the final token span is copied directly from the raw token.

For tokens split out of a `LexemeRun`, the disambiguator computes spans from the raw token's start offset plus the internal byte cursor:

```rust
SourceSpan {
    start: raw_token.span.start + cursor,
    end: raw_token.span.start + cursor + candidate_len,
}
```

String literals recognized in `StringRequired` context use the same rule. A malformed string literal consumes the rest of the raw run as one `ErrorRecovery` token whose span starts at the opening quote and ends at the raw token end.

The convenience `lex` and `disambiguate_reserved_shell` APIs also return span-bearing `Token` values. They are context-free, but they still preserve the source locations produced by `scan_raw`.

## Identifier and Symbol Override

When a spelling is both identifier-shaped and an active user symbol:

1. Ask `ScopeLexView` whether a scoped identifier binding overrides the symbol at this source position.
2. Ask `ParserLexContext` whether an identifier is legal at this grammar position.
3. Emit `Identifier` only when the override and parser context allow it.
4. Otherwise, emit the active `UserSymbol` candidate.

Undefined identifiers are still emitted as `Identifier`; name resolution reports undefined-name diagnostics later.

## Parser Context

`ParserLexContext` is a narrow input that expresses lexical expectations, not a parser callback into arbitrary syntax logic.

Examples:

- identifier-required position;
- symbol/operator-admitting expression position;
- symbol/operator position restricted to a subset of active user-symbol kinds;
- string-literal-required argument position;
- namespace-path position;
- recovery mode after a syntax error.

The disambiguator must not mutate parser state except by returning tokens and diagnostics.

Quote characters remain part of `LexemeRun` during raw scanning. The disambiguator is responsible for recognizing string literals only when `ParserLexContext` marks the current position as string-required. Outside those positions, quotes are ordinary symbol characters and participate in user-symbol matching.

`ParserLexContext` may also carry a `UserSymbolKindSet`. General, symbolic, and recovery modes admit a user-symbol spelling only when at least one active candidate for that spelling has an admitted kind. This lets parser integration ask for predicate-only, functor-only, mode-only, or other category-specific lexical views without rebuilding the active lexical environment. The final `UserSymbol` token still stores only the spelling and span; downstream phases recover the candidate list from the active lexical environment when they need kind, arity, overload, or provenance details.

The implemented modes admit candidates as follows:

| Mode | Identifiers | Reserved Words | Reserved Symbols | User Symbols | Numerals | Strings |
|---|---|---|---|---|---|---|
| `General` | yes | yes | yes | yes | yes | no |
| `IdentifierRequired` | yes | no | no | no | no | no |
| `Symbolic` | yes | yes | yes | yes | yes | no |
| `StringRequired` | no | no | no | no | no | yes |
| `NamespacePath` | yes | no | only `.` / `..` | no | no | no |
| `Recovery` | yes | yes | yes | yes | yes | no |

## Error Handling

Disambiguation diagnostics include:

- no valid token candidate in a raw run;
- parser context rejects every candidate;
- malformed string literal in a string-required position;
- unsupported raw tokens such as malformed annotation markers passed through from raw scanning.

Whenever possible, the disambiguator emits an `ErrorRecovery` token with the original spelling and resumes at the next recoverable byte boundary.

Diagnostic codes and byte spans are stable. Human-facing `message` text remains provisional. Tooling should use `LexDiagnosticPayload` for machine-readable details:

| Code | Payload |
|---|---|
| `NoValidTokenCandidate` | rejected lexeme plus an `EmitErrorRecoveryToken` recovery hint |
| `ParserContextRejectedCandidate` | parser mode, rejected lexeme, rejected candidate token kinds/spellings/spans, and an `EmitErrorRecoveryToken` recovery hint |
| `MalformedStringLiteral` | opening quote, malformed-string reason, and recovery hint |
| `UnsupportedRawToken` | raw token kind, raw spelling, and recovery hint |

Fixtures may optionally assert `diagnostic_payloads` summaries when a stable structured payload matters. They must not assert human-facing diagnostic messages.

## Tests

Tests should cover:

- longest-match over punctuation-shaped user symbols;
- identifier-shaped user symbol vs scoped identifier override;
- final token spans for one-to-one raw token mapping and for tokens split out of a `LexemeRun`;
- `ErrorRecovery` token spans matching the associated diagnostic span;
- reserved word emission;
- namespace-path context;
- parser contexts that admit only a subset of active user-symbol kinds;
- string literal only in string-required positions;
- equal-length import tie breaking through lexical environment;
- recovery emits stable `ErrorRecovery` tokens and diagnostics.
