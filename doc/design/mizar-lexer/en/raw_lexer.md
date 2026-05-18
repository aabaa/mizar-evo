# Module: raw_lexer

> Canonical language: English. Japanese companion: [../ja/raw_lexer.md](../ja/raw_lexer.md).

## Purpose

This module defines the lexer boundary for Mizar Evo.

Mizar lexical classification is context-sensitive: imported modules add user-defined symbols, user symbols may be identifier-shaped, and scoped identifier bindings may override symbols. Therefore the lexer must not be designed as a single context-free pass that permanently decides every `Identifier` vs `UserSymbol` classification.

The first implementation is intentionally smaller: it exposes `Token`, `TokenKind::Identifier`, `LexError`, and `lex(&str)` for initial identifier tests. This document defines how that minimal API grows without painting the lexer/parser boundary into a corner.

## Source Preconditions

Input to `mizar-lexer` is not raw file bytes.

The source-loading layer owns:

- reading files;
- validating UTF-8;
- normalizing platform newlines to LF-only text;
- removing ordinary comments from lexical input while preserving comment metadata elsewhere;
- preserving documentation comments as trivia metadata for later attachment;
- preserving a source map back to original file offsets when needed;
- validating code-region ASCII rules before lexing.

`mizar-lexer` may assume layout uses only:

```text
space, tab, newline
```

Carriage return is not layout at this layer. A `\r` reaching the lexer is either a source-loading bug or an intentionally malformed test fixture.

## Core Design

Lexing is split into two conceptual stages.

### Stage 1: Raw Scan

The raw scanner reads LF-only source text and produces source-span-preserving raw units.

Raw units are not final language tokens. In particular, `LexemeRun` is a graphic run that may later become one or more final tokens.

```rust
pub enum RawTokenKind {
    LexemeRun,
    NumeralLike,
    AnnotationMarker,
    Layout,
    Error,
}
```

`LexemeRun` is the central raw unit. It covers both identifier-shaped and punctuation-shaped spelling:

```text
alpha
succ
foo'
+
*+
|.
x*+y
```

The raw scanner must preserve spans, spelling, and enough structure for later longest-match disambiguation. It must not split too early in ways that prevent later recognition of active user symbols.

`LexemeRun` is deliberately coarse. Reserved punctuation such as `.`, `..`, `,`, `;`, quotes, and operator characters may appear inside a run. Later modules may inspect and split a run internally, but they must preserve source spans and must not require the raw scanner to know grammar context.

Comments and documentation comments are not raw tokens. The source-loading and preprocessing layers remove ordinary comments from lexical input and retain documentation comments as trivia metadata. Import pre-scan and scope skeleton construction may skip that trivia via the preprocessed source metadata, but they do not receive comments as `RawTokenKind` values.

### Import Pre-Scan and Active Lexical Environment

The raw scanner does not interpret imports and does not know the module system. It only produces raw units.

Active user symbols are assembled by a separate import pre-scan and environment-building path:

```text
LF-only source text
  -> raw scan
       RawTokenStream with LexemeRun spans
  -> import pre-scan
       top-level ImportStub values with raw module path spellings
  -> module resolver / build planner
       module ids and imported module lexical summaries
  -> lexical environment builder
       ActiveLexicalEnvironment
```

The import pre-scan reads raw lexer output using a restricted syntax mode. It is allowed to inspect and split inside `LexemeRun` spans for import syntax such as `.`, `..`, `,`, and `;`. It recognizes only enough top-level import structure to extract module path spellings and source spans. It must not resolve package/module existence, visibility, re-export legality, or imported symbol identity.

The active lexical environment is the input consumed by the disambiguator. It contains built-in reserved tables and exported user-symbol shapes from imported module lexical summaries. Constructing it is outside raw scanning.

### Stage 2: Disambiguation

The disambiguator turns raw units into final tokens using:

- reserved words;
- reserved special symbols;
- active user symbols from imported module interface summaries;
- parser expectation at the current grammar position;
- a read-only scope view when symbol/identifier override rules require it.

The disambiguator owns longest-match across `LexemeRun` content. A single raw run may produce multiple final tokens.

Example:

```text
raw:   LexemeRun("x*+y")
final: Identifier("x"), UserSymbol("*+"), Identifier("y")
```

If an active user symbol covers the full spelling and is not overridden by a scoped identifier rule, the same raw run may become:

```text
raw:   LexemeRun("x*+y")
final: UserSymbol("x*+y")
```

The disambiguator consumes scope information but does not build it. The scope view is produced before full parsing by a dedicated scope-skeleton pre-scan.

## Scope Skeleton Pre-Scan

Parser construction depends on token disambiguation, but token disambiguation may need to know whether a scoped identifier binding overrides an active user symbol. To avoid a parser/lexer cycle, Mizar Evo uses a dedicated scope-skeleton pre-scan.

The scope skeleton pre-scan reads raw lexer output and recognizes only the reserved-keyword-shaped structure needed to approximate lexical binding ranges. It does not produce a `SurfaceAst`, does not resolve names semantically, and does not decide whether an identifier is defined.

It may recognize constructs such as:

- block delimiters that affect lexical scope, such as `definition`, `proof`, `now`, and `end`;
- binder-introducing reserved words and forms, such as `let`, `for`, `reserve`, and `given`;
- comma-separated binding lists where their shape is recoverable from reserved syntax;
- labels or local names whose binding range can be approximated without full expression parsing.

The result is a scope skeleton that can answer only lexical override questions:

```rust
pub struct ScopeSkeleton {
    pub frames: Vec<LexicalScopeFrame>,
}

pub struct LexicalScopeFrame {
    pub range: SourceRange,
    pub bindings: Vec<ScopedBindingShape>,
}

pub struct ScopedBindingShape {
    pub spelling: String,
    pub introduced_at: SourceRange,
}
```

The skeleton is allowed to under-approximate bindings in malformed or unsupported source. It must be deterministic and source-span preserving. It must not accept or reject programs semantically.

The disambiguator receives only a narrow projection:

```rust
pub trait ScopeLexView {
    fn binding_overrides_symbol(&self, spelling: &str, position: SourcePos) -> bool;
}
```

`ScopeLexView` is implemented from the scope skeleton and resolver-provided module-scope data where needed. It must not expose full resolver state, type information, overload candidates, or proof semantics to the lexer.

## Symbol and Identifier Boundary

`Identifier` is a final token class for identifier-shaped source text. It does not mean that the name is defined.

Undefined-name diagnostics belong to name resolution, not raw lexing.

However, final classification between identifier-shaped user symbols and identifiers may require scope information. If the language rule says that a scoped identifier binding overrides an active symbol, the disambiguator must consult the scoped binding environment before committing to `UserSymbol`.

The boundary is:

| Question | Owner |
|---|---|
| Does this spelling match identifier syntax? | raw lexer helper |
| Is this spelling an active imported user symbol? | lexical environment |
| Can a scoped identifier binding override the symbol here? | scope skeleton / `ScopeLexView` |
| Which candidate should be selected after scope override is considered? | disambiguator |
| Is the resulting identifier defined and legal in this grammar construct? | name resolution / later semantic phases |
| Which overload does a symbol or identifier denote? | overload/type checking |

The raw lexer must not collapse these questions into one decision.

## Longest-Match Rules

Longest-match is applied by the disambiguator, not by early raw token splitting.

At each position inside a `LexemeRun`, the disambiguator considers candidates from:

1. reserved compound symbols;
2. active user symbols;
3. reserved words;
4. identifier syntax;
5. numeral syntax when the raw unit starts with a digit;
6. fallback error recovery.

The selected candidate is the longest valid candidate for the current parser expectation and override environment. If equal-length user symbols are active, import-order shadowing rules from the lexical environment decide the winner.

Parser expectation may rule out otherwise valid candidates. For example, a grammar position expecting a binder identifier may prefer an identifier interpretation, while an expression position may admit symbol interpretations.

## Imported Symbol Data

The lexer must not load full IR for imported `.miz` files.

Imports provide a lightweight module interface summary containing exported lexical symbols and enough provenance for diagnostics:

```rust
pub struct ModuleLexicalSummary {
    pub module_id: ModuleId,
    pub exported_symbols: Vec<ExportedSymbolShape>,
    pub fingerprint: LexicalSummaryFingerprint,
}
```

The active lexical environment is assembled from these summaries and from built-in reserved tables.

Full module IR is read only by later phases that need syntax, resolution, verification, or artifact data.

## Minimal Public API

The current crate-local API is:

```rust
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
}

pub enum TokenKind {
    Identifier,
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError>;
```

This API is a bootstrap surface for lexical tests. It represents only the first implemented subset:

- ASCII identifier start: `A-Z`, `a-z`, `_`;
- ASCII identifier continuation: `A-Z`, `a-z`, `0-9`, `_`, `'`;
- layout skipping for space, tab, and LF;
- `LexError` for unsupported token classes.

It must not be treated as the final context-sensitive lexer interface.

## Future Public API Direction

The crate should grow toward explicit raw scanning and disambiguation APIs:

```rust
pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError>;

pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &LexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &ScopeLexView,
) -> Result<TokenStream, LexError>;
```

`ScopeLexView` must be a narrow read-only view produced outside the disambiguator. It must answer only questions needed for lexical disambiguation, such as whether a scoped identifier binding overrides an active symbol at a source position. It must not expose the full resolver or type checker to the lexer.

## Error Handling

Raw scanning errors are for malformed source shapes at the lexical layer:

- non-LF carriage returns after source loading;
- unsupported non-ASCII code characters if source loading did not reject them;
- impossible annotation markers.

Disambiguation errors are for tokenization failures after context is considered:

- no valid token candidate at a source position;
- ambiguous equal-length candidates without a deterministic shadowing rule;
- grammar context forbids all candidates in a raw run.

Undefined identifiers are not lexing errors.

## Tests

The minimal crate tests cover:

- `alpha` lexes as one identifier;
- identifier body characters include digits, `_`, and apostrophe after the first character;
- space, tab, and LF separate identifiers;
- unsupported numerals currently return `LexError` until numeral tokens exist;
- carriage return is rejected because source loading must normalize LF-only text.

Future tests should be added before implementing:

- `scan_raw` preserves `LexemeRun` spans without premature splitting;
- a scope skeleton can be built from reserved-keyword-shaped binding structure before full parsing;
- longest-match chooses the longest active user symbol;
- identifier-shaped user symbols are disambiguated with lexical environment and scope override rules;
- imported symbol summaries are enough for lexical disambiguation without loading full IR;
- unresolved identifiers remain tokens and are rejected later by name resolution diagnostics.
