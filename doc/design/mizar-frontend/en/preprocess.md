# Module: preprocess

> Canonical language: English. Japanese companion: [../ja/preprocess.md](../ja/preprocess.md).

Status: planned.

## Purpose

This module implements the frontend pipeline Step 2 (preprocessing) and produces
the `PreprocessedSource` consumed by lexical-environment construction and lexing.

It coordinates the `mizar-lexer` source-preprocessing helpers over a
`SourceUnit`: code-region ASCII validation, comment and doc-comment separation,
annotation preservation in lexical text, and the shallow top-level import
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

pub fn preprocess(source: &SourceUnit, bridge: &SpanBridge) -> PreprocessedSource;
```

`PreprocessedSource` mirrors the architecture interface. `diagnostics` is added
so Step-2 lexical-precondition and comment-structure diagnostics travel with
the output and are merged later in deterministic order by the orchestration
layer.

`preprocess` always returns a `PreprocessedSource`: comment-structure and
ASCII-region errors are recorded as diagnostics with recovered lexical text
rather than aborting, so the lexer can still run and report further problems.

## Dependencies

- Internal: `source` (provides `SourceUnit`), `span_bridge` (converts lexer
  preprocess-map offsets to `mizar-session` `SourceRange`), `lexical_env` and
  `lexing` (consume `PreprocessedSource`).
- External: `mizar-lexer` (`preprocess_source_for_lexing`,
  `PreprocessedLexicalSource`, `SourcePreprocessMap`, `CommentTrivia`,
  `SourcePreprocessDiagnostic`, `scan_import_prelude`, `ImportPrelude`,
  `ImportStub`, `ImportPrescanDiagnostic`, `scan_raw`), `mizar-session`
  (`SourceId`, `SourceRange`).

## Data Structures

### Lexical Text and Source Map

`LexicalText` is the comment-stripped, annotation-preserving text the lexer will
scan. `LexicalSourceMap` wraps the lexer's `SourcePreprocessMap` together with
the `SourceUnit` `LineMap` / `LoadingMap`, so that any lexical-text byte offset
can be mapped through `span_bridge` to a primary `SourceRange` (and, across a
removed comment, to composite adjacent anchors). Synthetic whitespace inserted
where comments were removed is never reported as a primary user range.

### Comments and Doc Comments

Ordinary `Comment`s are retained for formatting and debugging only and are not
fed to the lexer. `DocComment`s keep their source range and raw body so the
parser can later attach them to a documentable item; attachment stays a parser
concern, and structured tag parsing is deferred. Both carry `SourceRange`
values already mapped to source coordinates.

### Import Stubs

`ImportStub` (re-exported from `mizar-lexer`) is the shallow result of scanning
the top-level import prelude: import kind, raw dotted module path, and source
span. It is not a resolved import — it is only enough to request an active
lexical environment and to produce good diagnostics if lexicon loading fails.
Package/module existence, visibility, and export checks are deferred to module
resolution.

## Algorithm / Logic

### Preprocess a SourceUnit

1. Call `mizar_lexer::preprocess_source_for_lexing` over `SourceUnit.source_text`
   to validate code-region ASCII (allowing Unicode in comments/annotations),
   strip ordinary comments, retain doc comments, preserve annotation syntax in
   lexical text, and produce a `SourcePreprocessMap`.
2. Map every retained comment, doc comment, and preprocess diagnostic span from
   lexical/source offsets to `mizar-session` `SourceRange` through `span_bridge`.
3. Raw-scan the lexical text (`scan_raw`) and run `scan_import_prelude` to
   extract `ImportStub`s and import-prescan diagnostics; map their spans to
   `SourceRange`.
4. Collect comment-structure, ASCII-precondition, and import-prescan diagnostics
   into `diagnostics`, preserving source order.
5. Assemble `LexicalSourceMap` from the preprocess map plus the line/loading maps
   and return `PreprocessedSource`.

The import pre-scan consumes raw lexer output; raw scanning itself does not
interpret imports. Only malformed import syntax that prevents lexicon loading is
reported here.

## Error Handling

Step-2 diagnostics are carried in `PreprocessedSource.diagnostics`, not raised
as a hard error:

- code-region non-ASCII characters and other lexical preconditions
  (`SourcePreprocessDiagnostic`);
- unterminated block comment and other comment-structure problems;
- import pre-scan failures that prevent active lexical environment construction
  (`ImportPrescanDiagnostic`).

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
- a removed comment yields a composite mapping for a lexical range that spans it;
- top-level `import` forms produce `ImportStub`s with correct kind, raw path, and
  span; a malformed import yields an `ImportPrescanDiagnostic` without aborting;
- a non-ASCII character in a code region is reported as a lexical-precondition
  diagnostic while preprocessing still returns recovered lexical text;
- an unterminated block comment is reported and recovered.

## Constraints and Assumptions

- This module does not tokenize, parse, or resolve imports.
- Comment-stripping, ASCII validation, and import-prescan algorithms belong to
  `mizar-lexer`; this module orchestrates them and owns span bridging.
- Annotations remain in lexical text for parser ownership; preprocessing does
  not collect annotations into a separate metadata channel.
- Synthetic whitespace is never a primary user-facing source range.
- `PreprocessedSource` is keyed by `source_hash` plus frontend version for
  incremental reuse.
