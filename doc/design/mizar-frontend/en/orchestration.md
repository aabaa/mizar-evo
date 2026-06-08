# Module: orchestration

> Canonical language: English. Japanese companion: [../ja/orchestration.md](../ja/orchestration.md).

Status: implemented through task 14, including unrecoverable-failure coverage
and end-to-end failure assertions.

## Purpose

This module implements the phase 1-3 coordinator (source_and_frontend pipeline Steps 1-5) that produces `FrontendOutput`.
It wires `source` → `preprocess` → `lexical_env` → `lexing` → `parsing`, merges
diagnostics from every phase into one deterministically ordered list, and exposes
the combined frontend output.

It is the only module that owns the end-to-end pipeline. It does not own source
identity, comment stripping, lexical environment assembly, longest-match,
grammar, or AST node definitions; those belong to `mizar-session`, `mizar-lexer`,
`mizar-syntax`, and `mizar-parser`. It does not
perform semantic name resolution or type checking.

See
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Frontend Pipeline", "Error Recovery", "Diagnostics", and "FrontendOutput".

## Public API

```rust
pub struct FrontendOutput<A> {
    pub source: SourceUnit,
    pub preprocessed: PreprocessedSource,
    pub tokens: TokenStream,
    pub ast: Option<A>,
    pub diagnostics: Vec<FrontendDiagnostic>,
}

pub struct Frontend<L, P, PS>
where
    L: SourceUnitLoader,
    P: LexicalSummaryProvider,
    PS: ParserSeam,
{ /* loader, lexical-summary provider, parser seam */ }

impl<L, P, PS> Frontend<L, P, PS>
where
    L: SourceUnitLoader,
    P: LexicalSummaryProvider,
    PS: ParserSeam,
    PS::Diagnostic: FrontendParserDiagnostic,
{
    pub fn new(loader: L, provider: P, parser: PS) -> Self;

    pub fn run(
        &self,
        request: SourceUnitRequest,
        ids: &dyn SessionIdAllocator,
    ) -> Result<FrontendOutput<PS::Ast>, FrontendError>;
}

pub struct FrontendDiagnostic {
    pub code: DiagnosticCode,
    pub message: Arc<str>,
    pub class: DiagnosticClass,
    pub location: DiagnosticLocation,
    pub secondary: Vec<SourceAnchor>,
    pub recovery_note: Option<String>,
}

pub enum DiagnosticLocation {
    SourceRange(SourceRange),
    SourceLoad(SourceLoadLocation),
}

pub enum SourceLoadLocation {
    Path { path: PathBuf },
    NormalizedPath { path: NormalizedPath },
    OpenBuffer { uri: DocumentUri },
    Generated { anchor: Option<SourceAnchor> },
    Unknown,
}

pub enum DiagnosticCode {
    SourceLoad,
    Preprocess(PreprocessDiagnosticKind),
    LexicalEnvironment(LexicalEnvironmentDiagnosticCode),
    Lexing(LexingDiagnosticKind),
    Syntax(Arc<str>),
}

pub enum DiagnosticClass {
    SourceLoad,
    LexicalPrecondition,
    CommentStructure,
    ImportPrescan,
    LexicalEnvironment,
    ScopeSkeleton,
    Tokenization,
    Syntax,
    AnnotationSyntax,
}

pub enum FrontendError {
    SourceLoad {
        source: Box<SourceLoadError>,
        diagnostic: Box<FrontendDiagnostic>,
    },
    SpanBridge {
        source: SpanBridgeError,
    },
    LexicalEnvironment {
        source: FrontendLexicalEnvironmentError,
    },
}

pub trait FrontendParserDiagnostic {
    fn into_frontend_diagnostic(self) -> Option<FrontendDiagnostic>;
}
```

`FrontendOutput<A>` matches the architecture interface while keeping the parser
AST type abstract. With `StubParserSeam`, `ast` is always `None`; with the real
parser seam, `A` is `mizar_syntax::SurfaceAst`. `FrontendDiagnostic` is the
unified diagnostic that all phase-specific diagnostics
(`SourcePreprocessDiagnostic`, `ImportPrescanDiagnostic`,
`LexicalEnvironmentDiagnostic`, `LexingDiagnostic` including raw-scan /
scope-skeleton / lexer diagnostics, and `SyntaxDiagnostic`) are mapped into, so
consumers see one ordered list. Range-backed diagnostics use
`DiagnosticLocation::SourceRange`; source-load failures that occur before a
`SourceId` / `LineMap` exists use `DiagnosticLocation::SourceLoad` with the best
available path, normalized-path, open-buffer URI, generated anchor, or `Unknown`
location.
`DiagnosticCode::Syntax` stores the parser-owned syntax diagnostic code key once
the real parser seam is enabled; with `StubParserSeam` no syntax diagnostics are
emitted.
`FrontendParserDiagnostic` is the narrow adapter that lets the coordinator map
the configured parser seam's diagnostic type into the unified frontend
diagnostic stream; it is implemented for `mizar_syntax::SyntaxDiagnostic` and
for the stub seam's unit diagnostic type.

With the stub parser seam, `ast = None` is the expected placeholder result. The
real parser seam returns a minimal `SurfaceAst` for recovered token streams and
may return `ast = None` when parsing cannot recover enough structure for
downstream phases. The lexical, preprocessing, and syntax diagnostics are still
returned.

## Dependencies

- Internal: `source`, `preprocess`, `lexical_env`, `lexing`, `parsing`,
  `span_bridge` (constructed once and threaded through the phases).
- External: `mizar-session` (`SourceId`, `SourceRange`, `SourceAnchor`,
  `NormalizedPath`, `DocumentUri`, `SessionIdAllocator`, `BuildSnapshotId`),
  `mizar-lexer`, `mizar-syntax`, `mizar-parser`, and `std::path::PathBuf`.

This module is the public entry point of the crate; it is consumed by the
compiler driver, LSP, the formatter, and tests.

## Data Structures

### Frontend Output

`FrontendOutput` bundles each phase artifact plus the merged diagnostics. It is
the unit later phases (module/name resolution) consume: they read `ast` and
`tokens`, and they read `source`/`preprocessed` for spans, comments, and import
stubs. Each artifact carries its own cache key per
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Incrementality", so a comment-only edit can reuse semantic outputs while a
dependency export change invalidates tokenization.

### Diagnostic Merge Order

Range-backed diagnostics merge by phase precedence and then by primary
`SourceRange` start, so the order is stable across runs and independent of
internal scheduling:

1. lexical precondition (Steps 1-2);
2. comment structure (Step 2);
3. import pre-scan (Step 2);
4. lexical environment (Step 3);
5. scope skeleton (Step 4 pre-disambiguation);
6. tokenization (Step 4);
7. syntax and annotation syntax (Step 5).

Within a class, order range-backed diagnostics by a total stable key:
`source_id`, primary span start, primary span end, diagnostic-code stable key
(including the syntax code string), message, secondary-anchor stable keys,
recovery-note text, and finally the phase-local emission ordinal assigned while
collecting diagnostics from that deterministic phase output. Source-load
diagnostics do not participate in the returned `FrontendOutput` merge because a
source-load failure returns `FrontendError` before any phase artifact exists. If a
caller displays several source-load failures from a batch, order them by the
stable source-load location key and then by diagnostic code. Secondary
`SourceAnchor`s are preserved for display and explanation and are part of the
tie-breaker key, but not the primary ordering criterion. A recovery note is
attached when a later diagnostic may be affected by an earlier recovery.

## Algorithm / Logic

### Run the frontend for one source

1. Construct a fresh mutable `SpanBridge` with its owned retained source-map
   service.
2. Load the `SourceUnit` (`source`); on a load error, return a `FrontendError`
   carrying a file-level diagnostic whose location is
   `DiagnosticLocation::SourceLoad`, and stop.
3. Register the loaded source maps on the bridge, then preprocess
   (`preprocess`): register the preprocess map, produce `PreprocessedSource`,
   and collect Step-2 diagnostics. Propagate a `SpanBridgeError` as
   `FrontendError`.
4. Build the `ActiveLexicalEnvironment` (`lexical_env`) from the import stubs.
   Recoverable import/provider issues become `LexicalEnvironmentDiagnostic`s;
   `FrontendLexicalEnvironmentError` becomes `FrontendError`.
5. Derive `ParserInputs` from the active lexical environment and source edition.
6. Tokenize (`lexing`) into a `TokenStream` using the current parser lexing
   context or the stub/general context when the real parser contract is not yet
   available. Propagate a `SpanBridgeError` as `FrontendError`.
7. Parse (`parsing`) through the configured `ParserSeam` into an optional AST.
8. Map every phase diagnostic into `FrontendDiagnostic`, merge in the
   deterministic order above, and assemble `FrontendOutput`.

Phases 2-5 do not abort on recoverable problems: they record diagnostics and
carry recovered artifacts forward, so one run can report lexical, tokenization,
and syntax diagnostics together.

## Error Handling

`FrontendError` is reserved for failures that prevent producing any
`FrontendOutput` — primarily source-load failures from Step 1 (unreadable file,
invalid UTF-8, path outside root) and internal `SpanBridgeError` invariant
violations. `FrontendLexicalEnvironmentError` from lexical-environment
construction also becomes a `FrontendError` when the active lexical environment
cannot be degraded safely.
Source-load errors are allowed to lack a `SourceRange` because most of them occur
before `SourceId` allocation, UTF-8 validation, or `LineMap` construction. They
must be reported with `DiagnosticLocation::SourceLoad`, never with a fabricated
zero-length source range.
Recoverable lexical precondition, comment, import pre-scan, lexical-environment,
scope-skeleton, tokenization, and syntax problems are not `FrontendError`s; they
are `FrontendDiagnostic`s inside a returned `FrontendOutput`. The stub parser
seam produces no syntax diagnostics and returns `ast = None`; syntax diagnostics
are expected only when the real parser seam is configured. Frontend diagnostics
never claim semantic facts such as "undefined symbol" or "ambiguous overload";
those belong to later phases.

## Tests

Key scenarios:

- with `StubParserSeam`, a well-formed source runs source → tokens and returns
  `FrontendOutput` with `ast = None` and no parser diagnostics;
- with the real parser seam, a well-formed source runs all phases and returns
  `FrontendOutput` with `ast = Some` and no diagnostics;
- with the real parser seam, a source with lexical-precondition, import-pre-scan,
  lexical-environment, scope-skeleton, tokenization, and syntax errors reports
  them in the deterministic merge order;
- later unrecoverable parser recovery returns `ast = None` while preserving
  preprocessing and tokenization diagnostics;
- a Step 1 load failure returns `FrontendError` with the file-level diagnostic
  and no `FrontendOutput`;
- diagnostic order is identical across repeated runs regardless of internal
  scheduling;
- same-class diagnostics with the same start and code still sort deterministically
  by the complete stable tie-breaker key;
- the merged diagnostics carry valid `SourceRange`s resolved through the span
  bridge when they are range-backed, while source-load failures carry non-range
  `SourceLoadLocation`s.

## Constraints and Assumptions

- This module owns orchestration only; phase logic stays in the per-phase modules
  and the upstream crates.
- The frontend produces syntax, not semantics.
- Returned-output diagnostic merge order is deterministic and span-keyed for
  range-backed diagnostics.
- `FrontendError` is for unrecoverable failures; recoverable problems are mapped
  diagnostics inside `FrontendOutput`.
- Source-load diagnostics must not fabricate source ranges; they use
  `DiagnosticLocation::SourceLoad`.
- Frontend artifacts are internal compiler data, not stable public build outputs.
