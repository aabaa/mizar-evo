# Module: orchestration

> Canonical language: English. Japanese companion: [../ja/orchestration.md](../ja/orchestration.md).

Status: planned.

## Purpose

This module implements the phase 1-3 coordinator (source_and_frontend pipeline Steps 1-5) that produces `FrontendOutput`.
It wires `source` → `preprocess` → `lexical_env` → `lexing` → `parsing`, merges
diagnostics from every phase into one deterministically ordered list, and exposes
the combined frontend output.

It is the only module that owns the end-to-end pipeline. It does not own source
identity, comment stripping, lexical environment assembly, longest-match,
grammar, or AST node definitions; those belong to `mizar-session`, `mizar-lexer`,
and, once available, the real `mizar-syntax` / `mizar-parser` seam. It does not
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
    pub class: DiagnosticClass,
    pub primary: SourceRange,
    pub secondary: Vec<SourceRange>,
    pub recovery_note: Option<String>,
}

pub enum DiagnosticClass {
    LexicalPrecondition,
    CommentStructure,
    ImportPrescan,
    LexicalEnvironment,
    ScopeSkeleton,
    Tokenization,
    Syntax,
    AnnotationSyntax,
}
```

`FrontendOutput<A>` matches the architecture interface while keeping the parser
AST type abstract. With `StubParserSeam`, `ast` is always `None`; with the real
parser seam, `A` is `mizar_syntax::SurfaceAst`. `FrontendDiagnostic` is the
unified diagnostic that all phase-specific diagnostics
(`SourcePreprocessDiagnostic`, `ImportPrescanDiagnostic`,
`LexicalEnvironmentDiagnostic`, `LexingDiagnostic` including raw-scan /
scope-skeleton / lexer diagnostics, and `SyntaxDiagnostic`) are mapped into, so
consumers see one ordered list keyed by `SourceRange`.

`ast = None` means parsing could not recover enough structure for later phases;
the lexical, preprocessing, and syntax diagnostics are still returned.

## Dependencies

- Internal: `source`, `preprocess`, `lexical_env`, `lexing`, `parsing`,
  `span_bridge` (constructed once and threaded through the phases).
- External: `mizar-session` (`SourceId`, `SourceRange`,
  `SessionIdAllocator`, `BuildSnapshotId`), `mizar-lexer`. The real parser seam
  additionally depends on `mizar-syntax` and `mizar-parser` once those crates
  exist.

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

Diagnostics merge by phase precedence and then by primary `SourceRange` start,
so the order is stable across runs and independent of internal scheduling:

1. lexical precondition (Steps 1-2);
2. comment structure (Step 2);
3. import pre-scan (Step 2);
4. lexical environment (Step 3);
5. scope skeleton (Step 4 pre-disambiguation);
6. tokenization (Step 4);
7. syntax and annotation syntax (Step 5).

Within a class, order by primary span start, then by diagnostic code. A recovery
note is attached when a later diagnostic may be affected by an earlier recovery.

## Algorithm / Logic

### Run the frontend for one source

1. Construct a fresh mutable `SpanBridge` with its owned retained source-map
   service.
2. Load the `SourceUnit` (`source`); on a load error, return a `FrontendError`
   carrying the file-level diagnostic and stop.
3. Register the loaded source maps on the bridge, then preprocess
   (`preprocess`): register the preprocess map, produce `PreprocessedSource`,
   and collect Step-2 diagnostics. Propagate a `SpanBridgeError` as
   `FrontendError`.
4. Build the `ActiveLexicalEnvironment` (`lexical_env`) from the import stubs.
   Recoverable import/provider issues become `LexicalEnvironmentDiagnostic`s;
   unrecoverable provider or malformed-summary infrastructure failures become
   `FrontendError`.
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
violations. Unrecoverable lexical-summary provider failures or malformed
dependency summary data may also be `FrontendError`s when they cannot be degraded
to a smaller active lexical environment. Recoverable lexical precondition,
comment, import pre-scan, lexical-environment, scope-skeleton, tokenization, and
syntax problems are not `FrontendError`s; they are `FrontendDiagnostic`s inside a
returned `FrontendOutput`. The stub parser seam produces no syntax diagnostics
and returns `ast = None`; syntax diagnostics are expected only when the real
parser seam is configured. Frontend diagnostics never claim semantic facts such
as "undefined symbol" or "ambiguous overload"; those belong to later phases.

## Tests

Key scenarios:

- with `StubParserSeam`, a well-formed source runs source → tokens and returns
  `FrontendOutput` with `ast = None` and no parser diagnostics;
- with the real parser seam, a well-formed source runs all phases and returns
  `FrontendOutput` with `ast = Some` and no diagnostics;
- with the real parser seam, a source with lexical-precondition, import-pre-scan,
  lexical-environment, scope-skeleton, tokenization, and syntax errors reports
  them in the deterministic merge order;
- a parse failure returns `ast = None` while preserving preprocessing and
  tokenization diagnostics;
- a Step 1 load failure returns `FrontendError` with the file-level diagnostic
  and no `FrontendOutput`;
- diagnostic order is identical across repeated runs regardless of internal
  scheduling;
- the merged diagnostics carry valid `SourceRange`s resolved through the span
  bridge.

## Constraints and Assumptions

- This module owns orchestration only; phase logic stays in the per-phase modules
  and the upstream crates.
- The frontend produces syntax, not semantics.
- Diagnostic merge order is deterministic and span-keyed.
- `FrontendError` is for unrecoverable failures; recoverable problems are mapped
  diagnostics inside `FrontendOutput`.
- Frontend artifacts are internal compiler data, not stable public build outputs.
