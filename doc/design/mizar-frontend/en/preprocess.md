# Module: preprocess

> Canonical language: English. Japanese companion: [../ja/preprocess.md](../ja/preprocess.md).

Status: implemented for frontend pipeline Step 2 preprocessing: comment and
doc-comment preprocessing plus shallow import pre-scan integration are wired.

## Purpose

This module implements the frontend pipeline Step 2 (preprocessing) and produces
the `PreprocessedSource` consumed by lexical-environment construction and lexing.

It coordinates the `mizar-lexer` source-preprocessing helpers over a
`SourceUnit`: code-region ASCII validation, comment and doc-comment separation,
annotation syntax preservation in lexical text, and shallow top-level import
pre-scan. It owns the orchestration and the span bridging back to
`mizar-session` `SourceRange`; it does not own the comment-stripping or
import-scan algorithms (those live in `mizar-lexer`), and it does not tokenize,
parse, or resolve imports.

See
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Step 2: Preprocess Source", "Comments and Doc Comments Are Source Metadata",
"Import Pre-Scan Is Shallow", and "Annotations Are Parser-Owned Syntax".

## Public API

```rust
pub struct PreprocessedSource {
    pub source_id: SourceId,
    pub lexical_text: LexicalText,
    pub lexical_hash: Hash,
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub import_stubs: Vec<ImportStub>,
    pub source_map: LexicalSourceMap,
    pub diagnostics: Vec<PreprocessDiagnostic>,
}

pub struct LexicalText {
    pub text: Arc<str>,
}

pub struct Comment {
    pub kind: CommentKind,
    pub source_range: SourceRange,
}

pub struct DocComment {
    pub source_range: SourceRange,
    pub raw_body: Arc<str>,
}

pub struct LexicalSourceMap { /* lexical-text offsets -> SourceRange */ }

pub struct ImportStub {
    pub path: ImportStubPath,
    pub alias: Option<ImportStubAlias>,
    pub span: SourceRange,
}

pub struct ImportStubPath {
    pub spelling: Arc<str>,
    pub relative: Option<ImportStubRelativePrefix>,
    pub components: Vec<Arc<str>>,
    pub source_segments: Vec<SourceRange>,
    pub span: SourceRange,
}

pub enum ImportStubRelativePrefix {
    Current,
    Parent,
}

pub struct ImportStubAlias {
    pub spelling: Arc<str>,
    pub span: SourceRange,
}

pub struct PreprocessDiagnostic {
    pub kind: PreprocessDiagnosticKind,
    pub message: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
}

#[non_exhaustive]
pub enum PreprocessDiagnosticKind {
    SourcePrecondition(SourcePreprocessDiagnosticCode),
    ImportPrescan(ImportPrescanDiagnosticCode),
    RawImportScan,
}

pub fn preprocess(
    source: &SourceUnit,
    bridge: &mut SpanBridge,
) -> Result<PreprocessedSource, SpanBridgeError>;
```

`PreprocessedSource` mirrors the architecture interface. `diagnostics` is added
so Step-2 lexical-precondition and comment-structure diagnostics travel with
the output and are merged later in deterministic order by the orchestration
layer.

`PreprocessDiagnosticKind` is `#[non_exhaustive]` for downstream crates so
future recoverable preprocessing surfaces can be added without breaking
external matches. Matches inside `mizar-frontend` remain exhaustive.

For user-recoverable input problems, `preprocess` returns `Ok(PreprocessedSource)`:
comment-structure and ASCII-region errors are recorded as diagnostics with
recovered lexical text rather than aborting, so the lexer can still run and
report further problems. Import-pre-scan diagnostics use the same channel. It
returns `Err(SpanBridgeError)` only for internal coordinate-bridge or
frontend/lexer integration invariant failures, such as an unmappable span, a
conflicting map registration, or an unsupported lexer-owned metadata variant.

## Dependencies

- Internal: `source` (provides `SourceUnit`), `span_bridge` (registers the
  source's preprocess map and converts lexer preprocess-map offsets to
  `mizar-session` `SourceRange`), `lexical_env` and `lexing` (consume
  `PreprocessedSource`).
- External: `mizar-lexer` (`preprocess_source_for_lexing`,
  `PreprocessedLexicalSource`, `SourcePreprocessMap`, `CommentTrivia`,
  `SourcePreprocessDiagnostic`, `scan_import_prelude`, `ImportPrelude`,
  `mizar_lexer::ImportStub`, `RawModuleRelativePrefix`,
  `ImportPrescanDiagnostic`, `SourcePreprocessDiagnosticCode`,
  `ImportPrescanDiagnosticCode`, `scan_raw_recoverable`,
  `RawScanDiagnostic`),
  `mizar-session`
  (`SourceId`, `SourceRange`, `SourceAnchor`).

## Data Structures

### Lexical Text and Source Map

`LexicalText` is the comment-stripped, annotation-preserving text the lexer will
scan. `LexicalSourceMap` carries the source id, lexical length, and retained
lexer `SourcePreprocessMap`; the mutable `SpanBridge` owns the registered
session `LineMap`, optional `LoadingMap`, and converted `PreprocessMap` used for
actual coordinate conversion. Together they let callers map lexical-text byte
offsets to a primary `SourceRange` (and, across a removed comment, to composite
adjacent anchors). Synthetic whitespace inserted where comments were removed has
no exact user-authored range; if a session `MappedSourceRange` needs a primary,
it is a degraded anchor fallback and must not be treated as exact source text.
`lexical_hash` is computed from the lexical text plus the frontend
preprocessing-version domain and is the downstream token/AST reuse key when
comment-only edits leave lexical text unchanged.

### Comments and Doc Comments

Ordinary `Comment`s are retained for formatting and debugging only and are not
fed to the lexer. `DocComment`s keep their source range and raw body so the
parser can later attach them to a documentable item; attachment stays a parser
concern, and structured tag parsing is deferred. Both carry `SourceRange`
values already mapped to source coordinates.

### Import Stubs

`ImportStub` is the mapped frontend counterpart of the `mizar-lexer`
import-pre-scan stub. It mirrors the lexer `RawModulePath` / `RawModuleAlias`
shape, but every span has already been converted to a session `SourceRange`.
The raw dotted module path, relative prefix (`.` for current vs `..` for
parent), and split source coverage for branch imports live on `path.spelling`,
`path.relative`, `path.components`, and `path.source_segments`. It is not a
resolved import — it is only enough to request an active lexical environment and
to produce good diagnostics if lexicon loading fails. Package/module existence,
visibility, export checks, and re-export semantics are deferred to module
resolution. `preprocess` fills `import_stubs` from the shallow raw import
pre-scan over the recoverable raw token stream, preserving imports that remain
readable without joining module-path text across raw-scan recovery boundaries.

`PreprocessDiagnostic` is the frontend-mapped diagnostic form for
`SourcePreprocessDiagnostic`, `ImportPrescanDiagnostic`, and frontend-local raw
import pre-scan failures. Raw lexer diagnostic structs are consumed as inputs and
converted immediately; public diagnostics keep mapped session ranges plus
secondary `SourceAnchor`s when a preprocess mapping is composite or degraded.
`SourcePrecondition`, `ImportPrescan`, and `RawImportScan` diagnostics are all
emitted by this module when their corresponding recoverable input problem is
encountered.

## Algorithm / Logic

### Preprocess a SourceUnit

1. Call `mizar_lexer::preprocess_source_for_lexing` over `SourceUnit.source_text`
   to validate code-region ASCII while allowing Unicode in comments and
   recognized single-line parser-assisted string argument spans, strip ordinary
   comments, retain doc comments, preserve annotation syntax in lexical text, and
   produce a `SourcePreprocessMap`.
2. Convert the lexer `SourcePreprocessMap` to the session `PreprocessMap` and
   register it on the mutable `SpanBridge` for the `SourceId`.
3. Map every retained comment, doc comment, and preprocess diagnostic span from
   lexical/source offsets to `mizar-session` `SourceRange` through `span_bridge`.
4. Recoverably raw-scan the lexical text for import pre-scan, preserving
   recognized single-line string argument spans before scanning ordinary
   segments. `mizar_lexer::scan_raw_recoverable` reports precise raw-scan
   diagnostics and returns usable partial raw tokens plus `RawTokenKind::Error`
   sentinels for recovery boundaries.
5. Run `scan_import_prelude` over the partial raw token stream to extract
   `ImportStub`s and import-prescan diagnostics, then map their spans to
   `SourceRange`. Raw-scan diagnostics become frontend-local `RawImportScan`
   diagnostics at the offending spans instead of whole-file fallback ranges. The
   error sentinels prevent module-path text from being joined across malformed
   spans.
6. Collect import-prescan diagnostics into `diagnostics` after the comment
   structure and ASCII-precondition diagnostics, preserving source order within
   each phase.
7. Compute `lexical_hash` from the final lexical text and frontend preprocessing
   version.
8. Assemble `LexicalSourceMap` from the retained lexer preprocess map plus the
   registered bridge state and return `PreprocessedSource`.

The import pre-scan consumes raw lexer output; raw scanning itself does not
interpret imports. Recovered lexical text from preprocessing can still contain
raw-scan errors. Those errors no longer disable the whole shallow import
extraction: the recoverable scanner reports the precise offending span and lets
import pre-scan continue from the next raw boundary.

## Error Handling

Step-2 diagnostics are carried in `PreprocessedSource.diagnostics`, not raised
as a hard error. The module records:

- code-region non-ASCII characters and other lexical preconditions
  (`SourcePreprocessDiagnostic`), excluding characters inside recognized
  single-line parser-assisted string argument spans;
- unterminated block comment and other comment-structure problems.
- import pre-scan failures that prevent active lexical environment construction
  (`ImportPrescanDiagnostic`);
- raw-scan failures during import pre-scan, represented as frontend-local
  `PreprocessDiagnostic` values with precise offending spans so preprocessing
  can still return recovered lexical text and any import stubs found in the
  usable partial raw stream.

A pre-scan failure severe enough to block lexicon loading is recorded so the
orchestration layer can decide whether to skip lexical-environment extension for
the affected import while still tokenizing the rest of the file. Preprocessing
never claims semantic facts.

## Tests

Key scenarios:

- ordinary comments are removed from `lexical_text` but retained as `Comment`s
  with correct `SourceRange`s;
- a doc comment is preserved with its raw body and source range and is not fed
  into lexical text;
- annotation syntax (`@latex(...)`, `@[...]`) stays in `lexical_text`;
- Unicode and comment-marker text inside recognized single-line
  annotation/string arguments stays in `lexical_text` and is not reported as a
  code-region non-ASCII precondition;
- a removed comment yields a composite mapping for a lexical range that spans it;
- synthetic whitespace is exposed only through degraded anchor-backed mappings,
  not exact user-authored ranges;
- `lexical_hash` stays stable when comment-only edits leave `lexical_text`
  unchanged;
- a non-ASCII character outside recognized single-line string argument spans is
  reported as a lexical-precondition diagnostic while preprocessing still
  returns recovered lexical text;
- an unterminated block comment is reported and recovered;
- top-level `import` forms produce `ImportStub`s with correct raw path, optional
  alias, `path.relative`, `path.source_segments`, and span; `.` and `..`
  relative prefixes remain distinguishable as current and parent imports; a
  malformed import yields an `ImportPrescanDiagnostic` without aborting;
- a recoverable raw-scan failure during import pre-scan yields a precise
  diagnostic while preserving usable partial import stubs;
- multiple preprocess diagnostics preserve phase order, messages, primary
  ranges, and secondary anchors.

## Constraints and Assumptions

- This module does not tokenize, parse, or resolve imports.
- Comment-stripping, ASCII validation, and import-prescan algorithms belong to
  `mizar-lexer`; this module orchestrates them and owns span bridging.
- Annotation syntax remains in lexical text for parser ownership; preprocessing
  does not collect annotations into a separate metadata channel. Task 20 lets
  preprocessing preserve recognized single-line string argument spans before
  ASCII precondition reporting, so Unicode and comment markers inside those spans
  are accepted while non-ASCII outside comments and string arguments remains a
  lexical precondition diagnostic.
- Synthetic whitespace is never an exact primary user-facing source range;
  degraded anchor fallbacks are allowed only to satisfy the session
  `MappedSourceRange` shape.
- `PreprocessedSource` production is keyed by `source_hash` plus frontend version.
  Downstream tokenization and syntax reuse use `lexical_hash` so comment-only
  edits can preserve later artifacts when the lexical text is unchanged.
