# Module: raw_lexer

> Canonical language: English. Japanese companion: [../ja/raw_lexer.md](../ja/raw_lexer.md).

## Purpose

This module defines the lexer boundary for Mizar Evo.

Mizar lexical classification is context-sensitive: imported modules add user-defined symbols, user symbols may be identifier-shaped, and scoped identifier bindings may override symbols. Therefore the lexer must not be designed as a single context-free pass that permanently decides every `Identifier` vs `UserSymbol` classification.

The current implementation exposes both the low-level raw scanner and higher-level disambiguation entry points. This document keeps the boundary explicit so callers do not accidentally treat the convenience `lex(&str)` shell as the full context-sensitive lexer.

## Public API Stability

`mizar-lexer` is currently a `0.1` crate. Public data structures remain parser-facing transfer objects with visible fields so early parser, corpus, and integration code can inspect and construct them directly.

Public enums are marked `#[non_exhaustive]`. Downstream crates must include wildcard arms when matching token kinds, raw token kinds, diagnostic codes, parser modes, import pre-scan categories, scope-skeleton categories, source-preprocessing categories, and lexical-environment errors. This keeps the crate free to add categories as the parser-facing API matures.

For the next parser-integration `0.x` milestone, downstream crates may rely on the following compatibility boundary:

| Surface | Compatibility promise for the next parser milestone |
|---|---|
| Entry points | `load_source_text_from_bytes`, `preprocess_source_for_lexing`, strict `scan_raw`, recoverable `scan_raw_recoverable`, `scan_import_prelude`, `build_lexical_environment`, `build_scope_skeleton`, `lex`, and `disambiguate` remain the executable lexer handoff path. |
| Coordinates | `SourceSpan` remains a half-open byte range in the exact text passed to the producing stage. `SourceSpan::new`, `try_new`, `len`, `is_empty`, `is_valid`, and `contains` are the preferred stable helpers for downstream code that does not need field construction. `SourceSpan::new` rejects reversed ranges; defensive boundary APIs still reject invalid spans that external callers construct through visible fields. |
| Token access | `RawToken`, `RawTokenStream`, `Token`, `TokenStream`, and `LexDiagnostic` keep parser-facing fields visible for now, but callers should prefer their constructors and accessor methods where possible. `TokenStream::into_parts` should be used when taking ownership so recoverable diagnostics are not dropped accidentally. This leaves room to add cached metadata or wrappers later without changing the read-only parser path. |
| IDs and fingerprints | `ModuleId`, `SymbolId`, `ExportRank`, `LexicalSummaryFingerprint`, and `LexicalEnvironmentFingerprint` remain lightweight newtypes. Prefer `new`, `as_str`, or `get` over tuple-field access in new integration code. The exact fingerprint algorithm may still change between `0.x` releases when the lexical summary shape changes. |
| Diagnostics | Diagnostic codes and byte spans are compatibility-facing. Human-readable `message` text is provisional and should not be snapshot-tested by downstream tools unless a fixture explicitly owns that wording. |
| Visible transfer fields | Existing visible fields are retained during this milestone to keep parser and corpus integration simple. They are not a promise that every field will remain constructible forever; fields that need stronger stability should first gain a constructor or accessor. |

Until a later explicit stability milestone, `0.1` minor releases may still make breaking changes to public fields, constructors, and helper functions when that is needed to keep the lexer boundary coherent.

## Source Preconditions

Input to `mizar-lexer` is not raw file bytes.

The source-loading layer outside this crate owns:

- reading files;
- validating UTF-8;
- accepting and stripping a leading UTF-8 BOM from package-authored source text before lexer entry;
- normalizing platform newlines to LF-only text before scanner entry points are used;
- preserving a source map back to original file offsets when needed;
- deciding how source files are located in packages.

`mizar-lexer` provides source-preprocessing helpers for the lexical boundary:

- validating UTF-8 source bytes with the crate-local `load_source_text_from_bytes` helper when tests or early integration code need an executable source-loading boundary;
- stripping one leading UTF-8 BOM in that helper and recording a minimal loading map from post-strip loaded-text offsets back to original byte offsets;
- normalizing CRLF newline pairs to LF in that helper before lexer entry and recording normalized-newline map segments back to original byte ranges;
- removing ordinary, documentation, and multi-line comments from lexical input;
- preserving comment trivia with source spans;
- preserving newline characters from comment text so line structure is not collapsed;
- inserting synthetic layout when comment removal would otherwise concatenate adjacent token-shaped text;
- reporting carriage returns, non-ASCII code-region text, and unterminated multi-line comments as preprocessing diagnostics;
- validating package-rooted `.miz` source names when requested.

`mizar-lexer` may assume layout uses only:

```text
space, tab, newline
```

Carriage return is not layout at this layer. Source loading normalizes CRLF pairs to LF before scanner entry; a lone `\r` reaching the lexer is either a source-loading bug, malformed input that is not a platform newline, or an intentionally malformed test fixture.

A leading UTF-8 BOM is also a source-loading concern, not a lexer feature. Disk input may contain the byte sequence `EF BB BF`, and package-authored open-buffer text may contain the corresponding leading `U+FEFF`; the source loader strips exactly that leading signature before constructing `LoadedSource.text` or calling lexer entry points. Any `U+FEFF` that reaches `preprocess_source_for_lexing` or `scan_raw`, including a non-leading one, remains malformed lexer-boundary input and must not be silently discarded here.

## Source-Text Normalization Policy

`mizar-lexer` does not perform Unicode normalization. It neither canonicalizes nor compatibility-normalizes code text before applying lexical spelling rules.

Code-region identifiers, numerals, reserved words, reserved symbols, and user-symbol spellings are ASCII-only at this layer. Recognized parser-assisted string argument spans are the narrow exception: preprocessing preserves single-line quoted spans before comment stripping and ASCII diagnostics so Unicode and comment markers inside annotation-style arguments remain source text for later plan-sensitive tokenization. The recognition uses the lexical text accumulated so far, so removed comment contents cannot create a string-argument context. Outside comments, documentation comments, and those recognized single-line string argument spans, non-ASCII text that reaches a code region is malformed input for the lexer boundary: preprocessing reports it as `NonAsciiCode`, and direct raw scanning rejects unsupported characters instead of converting them into ASCII spellings.

Comments and documentation comments are different. Their text is preserved as raw Unicode trivia with source spans, except that newline structure is retained in `lexical_text` according to the comment-stripping rules above. The lexer does not normalize, warn about, or reject Unicode in comment/documentation text. A later documentation, source-loading, or diagnostic policy may add warnings for suspicious Unicode, confusables, or normalization-sensitive text without changing lexer tokenization.

## Core Design

Lexing is split into two conceptual stages.

## Implemented Algorithm Flow

The current crate separates source preparation, raw scanning, and final disambiguation even when a caller uses the convenience `lex` entry point.

1. `preprocess_source_for_lexing` walks the input byte span in source order. It preserves recognized single-line parser-assisted string argument spans verbatim before comment stripping, removes comments from lexical text, preserves newline characters from comment bodies, inserts synthetic layout when removal would concatenate adjacent token-shaped text, stores comment trivia with source spans, records a lightweight preprocess map from lexical ranges back to loaded-source ranges, and reports carriage returns, non-ASCII code characters outside recognized string argument spans, or unterminated multi-line comments as preprocessing diagnostics. Multi-line comments are non-nesting: the first `=::` after a `::=` opener closes the comment, and any inner `::=` spelling is ordinary comment text. This helper does not read files or normalize platform-specific paths.
2. `scan_raw` consumes LF-only lexical text with a `char_indices` cursor. It coalesces adjacent layout into one `Layout`, recognizes annotation markers beginning with `@`, coalesces ASCII graphic non-`@` characters into either `NumeralLike` when all characters are digits or `LexemeRun` otherwise, and rejects unsupported characters with `LexError`. `scan_raw_recoverable` uses the same tokenization rules but records `RawScanDiagnostic`s with precise offending spans, emits matching `RawTokenKind::Error` sentinels for malformed spellings, and resumes so callers can keep usable partial raw tokens without joining text across recovery boundaries.
3. `disambiguate_reserved_shell` is the context-free shell used by `lex`. It drops layout, maps `NumeralLike` to `Numeral`, maps `@[` to a reserved symbol, and classifies whole `LexemeRun` values as reserved symbols, reserved words, identifiers, or opaque `LexemeRun` values.
4. Context-sensitive callers use `disambiguate` instead. That path keeps raw scanning deliberately coarse and lets the disambiguator split each `LexemeRun` using reserved tables, the active lexical environment, parser lexical context, and `ScopeLexView`.
5. `module_source_name_from_path` is a source-boundary helper rather than a scanner. It validates the package name, requires a `.miz` file under a `src` root, checks that the source root matches the package name, normalizes path separators, and emits namespace components that are all identifier-shaped.

The strict raw scanner's main invariant is span contiguity: every emitted raw token points back to the exact byte slice it came from, and the concatenation of raw token lexemes reconstructs the raw scanner input. The recoverable scanner preserves that invariant by inserting `RawTokenKind::Error` sentinels for diagnostic spans; callers that want only usable raw units can filter those sentinels while still using them to avoid joining text across recovery boundaries.

### Source Coordinates

`SourceSpan` is the canonical coordinate type inside `mizar-lexer`. It stores byte offsets into the exact text that produced the token or diagnostic and represents a half-open range `[start, end)`.

Callers must keep the coordinate space explicit. Raw tokens and final tokens produced from `scan_raw` and `disambiguate` point into the scanner input passed to `scan_raw`. When that input is `PreprocessedLexicalSource.lexical_text`, the spans are lexical-text offsets, not necessarily offsets into the original loaded `.miz` text. A `SourceLineIndex` must always be built from the same text that the spans address.

Mapping lexical-text offsets back to original loaded-source offsets is explicit. `PreprocessedLexicalSource.preprocess_map` provides a lightweight lexer-local map for comment stripping and synthetic layout; richer snapshot/source-map ownership remains in the session layer. The lexer must not silently treat spans from preprocessed text as original file coordinates.

When source loading strips a leading BOM or normalizes CRLF newline pairs, lexer spans are measured in the normalized loaded text, not in raw file bytes. For disk sources, `LoadingMap` relates those loaded-text offsets back to original file byte offsets: offset `0` in loaded text maps to byte offset `3` in the original file when a BOM was stripped, and each normalized LF maps to the original two-byte CRLF range. The stripped BOM has no lexer `SourceSpan`.

The crate-local `load_source_text_from_bytes` helper implements the UTF-8, leading-BOM, and CRLF-to-LF parts of that contract. It rejects invalid UTF-8 with `SourceLoadError::InvalidUtf8` and never decodes bytes lossily into `U+FFFD`. When it strips a leading BOM, it returns `LoadedSourceText.loading_map` with `RemovedLeadingBom { original: [0, 3) }` and following segments that map loaded offsets to original byte offsets. When it normalizes CRLF, it records `NormalizedNewline` segments whose loaded range is the single LF byte and whose original range is the two-byte CRLF spelling. Lone `\r` is not normalized by this helper and remains malformed lexer-boundary input. Full file I/O, path normalization, hashes, and rich session `LineMap` ownership remain outside `mizar-lexer`.

The lexer must not store line and column numbers on every raw or final token. Line and column positions are derived views computed from the source text when diagnostics, debug output, snapshots, or LSP bridges need human-readable coordinates. This avoids duplicating location data and avoids mixing multiple coordinate systems in token values.

`mizar-lexer` provides a lightweight line-index helper for lexer-local use:

```rust
pub struct SourceLineIndex {
    line_starts: Vec<usize>,
    char_boundaries: Vec<usize>,
    source_len: usize,
}

pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

pub struct SourceLocationRange {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceLineIndex {
    pub fn new(source: &str) -> Self;
    pub fn location(&self, offset: usize) -> Option<SourceLocation>;
    pub fn range(&self, span: SourceSpan) -> Option<SourceLocationRange>;
}
```

The internal convention is zero-based line and zero-based byte column. `location` and `range` return `None` when the requested offset or span is outside the indexed source text or does not land on a UTF-8 character boundary. Human-facing diagnostics can convert to one-based display numbers at formatting time. LSP-specific UTF-16 positions are not stored in tokens; they are computed by the LSP bridge or a dedicated adapter from the same byte offsets.

This helper is intentionally not a source-loading abstraction. The session layer may keep a richer `LineMap` on `LoadedSource` for open buffers, snapshots, source maps, and LSP integration. `mizar-lexer` only needs enough coordinate conversion to make lexer diagnostics and tests readable from a `&str`.

### Stage 1: Raw Scan

The raw scanner reads LF-only source text and produces source-span-preserving raw units.

Raw units are not final language tokens. In particular, `LexemeRun` is a graphic run that may later become one or more final tokens.

```rust
#[non_exhaustive]
pub enum RawTokenKind {
    LexemeRun,
    NumeralLike,
    AnnotationMarker,
    Layout,
    Error,
}
```

`scan_raw` returns `LexError` for unsupported raw input instead of emitting `RawTokenKind::Error`. `scan_raw_recoverable` returns:

```rust
pub struct RecoverableRawTokenStream {
    pub tokens: Vec<RawToken>,
    pub diagnostics: Vec<RawScanDiagnostic>,
}

pub struct RawScanDiagnostic {
    pub code: RawScanDiagnosticCode,
    pub message: String,
    pub span: SourceSpan,
}

#[non_exhaustive]
pub enum RawScanDiagnosticCode {
    UnsupportedAnnotationMarker,
    UnsupportedInput,
}
```

It emits usable raw tokens for valid spans, records malformed raw input as diagnostics, emits an `Error` raw token for each offending span, and resumes at the next character boundary. Malformed annotation markers consume only the offending `@`, so following text such as `@ name` can still produce later raw tokens.

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

Comments and documentation comments are not raw tokens. `preprocess_source_for_lexing` removes them from lexical input, preserves their trivia and spans separately, keeps their newline characters in `lexical_text`, inserts a synthetic space when removing an inline comment would otherwise concatenate adjacent token-shaped text, and records preprocess-map segments for original text, removed comments, and synthetic whitespace/newlines. Multi-line comments do not nest; the first closing `=::` terminates the comment. Import pre-scan and scope skeleton construction operate on the resulting lexical text; they never receive comments as `RawTokenKind` values.

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

The active lexical environment is the input consumed by the disambiguator. It contains built-in reserved tables and exported user-symbol shapes from imported module lexical summaries, including lightweight kind and arity metadata for later parser/resolver filtering. Constructing it is outside raw scanning.

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

- block delimiters that affect lexical scope, such as `definition`, `proof`, `now`, nested `struct`, explicit `inherit ... where`, and `end`;
- binder-introducing reserved words and forms, such as `let`, `for`, `reserve`, and `given`;
- comma-separated binding lists where their shape is recoverable from reserved syntax;
- local names whose binding range can be approximated without full expression parsing.

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

1. active user symbols;
2. reserved compound symbols;
3. reserved words;
4. identifier syntax;
5. numeral syntax when the raw unit starts with a digit;
6. fallback error recovery.

The selected candidate is the longest valid candidate for the current parser expectation and override environment. The lexical environment has already rejected equal-spelling symbols imported from different modules. Same-import overload candidates with the same spelling remain representable, with kind and arity metadata, for later semantic resolution; they do not change the final token spelling chosen by the lexer.

Parser expectation may rule out otherwise valid candidates. For example, a grammar position expecting a binder identifier may prefer an identifier interpretation, while an expression position may admit symbol interpretations.

## Imported Symbol Data

The lexer must not load full IR for imported `.miz` files.

Imports provide a lightweight module interface summary containing exported lexical symbols, symbol kind and arity shape, and enough provenance for diagnostics:

```rust
pub struct ModuleLexicalSummary {
    pub module_id: ModuleId,
    pub exported_symbols: Vec<ExportedSymbolShape>,
    pub fingerprint: LexicalSummaryFingerprint,
}
```

The active lexical environment is assembled from these summaries and from built-in reserved tables.

Full module IR is read only by later phases that need syntax, resolution, verification, or artifact data.

## Current Public API

The current crate-local API has grown beyond the bootstrap identifier lexer:

```rust
pub fn preprocess_source_for_lexing(input: &str) -> PreprocessedLexicalSource;

pub struct PreprocessedLexicalSource {
    pub lexical_text: String,
    pub comments: Vec<CommentTrivia>,
    pub diagnostics: Vec<SourcePreprocessDiagnostic>,
    pub preprocess_map: SourcePreprocessMap,
}

pub enum SourcePreprocessMapSegment {
    Original { lexical: SourceRange, source: SourceRange },
    RemovedComment { source: SourceRange, kind: CommentKind },
    SyntheticWhitespace { lexical: SourceRange, anchor: SourceRange },
}

pub fn module_source_name_from_path(
    package_name: &str,
    path: &str,
) -> Result<ModuleSourceName, ModuleNamingError>;

pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError>;
pub fn scan_raw_recoverable(input: &str) -> RecoverableRawTokenStream;

pub struct RecoverableRawTokenStream {
    pub tokens: Vec<RawToken>,
    pub diagnostics: Vec<RawScanDiagnostic>,
}

pub struct RawScanDiagnostic {
    pub code: RawScanDiagnosticCode,
    pub message: String,
    pub span: SourceSpan,
}

#[non_exhaustive]
pub enum RawScanDiagnosticCode {
    UnsupportedAnnotationMarker,
    UnsupportedInput,
}

pub fn disambiguate_reserved_shell(raw: &RawTokenStream) -> Result<Vec<Token>, LexError>;
pub fn lex(input: &str) -> Result<Vec<Token>, LexError>;

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
```

`lex` remains a convenience wrapper for raw scanning plus reserved-shell disambiguation. It still returns span-bearing final tokens; context-free classification is not allowed to drop source locations. The context-sensitive API lives in `disambiguator.md` and should be used when imports, parser context, or scope override can affect token classification.

Helper predicates define the low-level spelling rules used across modules: layout is exactly space, tab, or LF; identifiers start with ASCII alphabetic or `_`; identifier continuation additionally admits digits and `'`; numerals are ASCII digit runs; user-symbol spellings are non-empty ASCII graphic runs excluding `@`; string-literal spellings must close with the same quote and may only escape `"`, `'`, or `\`.

## Context-Sensitive API Direction

The explicit raw scanning and disambiguation API is now present:

```rust
pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError>;

pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream;
```

`ScopeLexView` must be a narrow read-only view produced outside the disambiguator. It must answer only questions needed for lexical disambiguation, such as whether a scoped identifier binding overrides an active symbol at a source position. It must not expose the full resolver or type checker to the lexer.

## Error Handling

Raw scanning errors are for malformed source shapes at the lexical layer:

- non-LF carriage returns after source loading;
- unsupported non-ASCII code characters if source loading did not reject them;
- unsupported ASCII control characters such as vertical tab or form feed;
- impossible annotation markers.

Source preprocessing diagnostics keep stable codes and byte spans plus structured payloads. `CarriageReturn` carries a newline-normalization recovery hint, `NonAsciiCode` carries the rejected character and UTF-8 length, and `UnterminatedMultiLineComment` carries the opener span plus a comment-recovery hint.

Disambiguation errors are for tokenization failures after context is considered:

- no valid token candidate at a source position;
- grammar context forbids all candidates in a raw run.

Undefined identifiers are not lexing errors.

Final token spans are part of the lexer boundary. `RawToken` spans are copied for one-to-one mappings and subdivided when a `LexemeRun` yields multiple final tokens. Downstream parser, diagnostic, LSP, formatter, and incremental-analysis layers must be able to locate every final token without consulting raw tokens again.

Line and column values are derived from final token spans through `SourceLineIndex` or the session layer's `LineMap`. They are not stored on `Token`.

## Tests

The crate and corpus tests cover:

- identifier, numeral, layout, annotation marker, reserved word, and reserved symbol tables;
- source preprocessing diagnostics and module source naming boundaries;
- unsupported Unicode code-region characters and unsupported ASCII control characters remain diagnostics or stable raw-scan hard errors rather than layout or token text;
- `scan_raw` preserves `LexemeRun` spans without premature splitting;
- a scope skeleton can be built from reserved-keyword-shaped binding structure before full parsing;
- longest-match chooses the longest active user symbol;
- identifier-shaped user symbols are disambiguated with lexical environment and scope override rules;
- imported symbol summaries are enough for lexical disambiguation without loading full IR;
- unresolved identifiers remain tokens and are rejected later by name resolution diagnostics;
- `cargo-fuzz` coverage exercises `preprocess_source_for_lexing`, direct `scan_raw`, and `scan_raw` over preprocessed lexical text for arbitrary valid UTF-8 input;
- Phase 7 regression tests preserve raw/final span coverage, deterministic raw scanning, retokenization, import conflict, recovery spans, and composite disambiguation behavior.
