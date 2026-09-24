# Module: orchestration

> Canonical language: English. Japanese companion: [../ja/orchestration.md](../ja/orchestration.md).

Status: implemented through task 20, including unrecoverable-failure coverage,
end-to-end failure assertions, cache-key output wiring, parser lexing-plan
wiring, and source-position-aware operator metadata assembly.

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

`run_loaded` consumes an already loaded `SourceUnit` without filesystem access or source-id allocation; `run` delegates to it after normal loading. The caller validates snapshot binding. Both entries share preprocessing, provider/parser invocation, diagnostics and cache keys; tests cover output parity (including recovery), deleted files and unchanged failures.

## Public API

```rust
pub struct FrontendOutput<A> {
    pub source: SourceUnit,
    pub preprocessed: PreprocessedSource,
    pub tokens: TokenStream,
    pub ast: Option<A>,
    pub diagnostics: Vec<FrontendDiagnostic>,
    pub cache_keys: FrontendCacheKeys,
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
    pub fn run_loaded(&self, source: SourceUnit) -> Result<FrontendOutput<PS::Ast>, FrontendError>;
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

#[non_exhaustive]
pub enum SourceLoadLocation {
    Path { path: PathBuf },
    NormalizedPath { path: NormalizedPath },
    OpenBuffer { uri: DocumentUri },
    Generated { anchor: Option<SourceAnchor> },
    Unknown,
}

#[non_exhaustive]
pub enum DiagnosticCode {
    SourceLoad,
    Preprocess(PreprocessDiagnosticKind),
    LexicalEnvironment(LexicalEnvironmentDiagnosticCode),
    Lexing(LexingDiagnosticKind),
    Syntax(Arc<str>),
}

#[non_exhaustive]
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

#[non_exhaustive]
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
parser seam, `A` is `mizar_syntax::SurfaceAst`. `cache_keys` is the
frontend-computed content-key bundle from [cache_key.md](./cache_key.md);
drivers decide how to persist cache records and compose these content keys with
snapshot/task identity. `FrontendDiagnostic` is the
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
the real parser seam is enabled; current `mizar-syntax` codes map to stable
snake-case keys, and future non-exhaustive syntax codes fall back to
`syntax_diagnostic` until frontend-owned handling is added. With
`StubParserSeam` no syntax diagnostics are emitted.
`FrontendParserDiagnostic` is the narrow adapter that lets the coordinator map
the configured parser seam's diagnostic type into the unified frontend
diagnostic stream; it is implemented for `mizar_syntax::SyntaxDiagnostic` and
for the stub seam's unit diagnostic type.

`SourceLoadLocation`, `DiagnosticCode`, `DiagnosticClass`, and `FrontendError`
are `#[non_exhaustive]` for downstream crates so future source, diagnostic, and
unrecoverable frontend surfaces can be added without breaking external matches.
Matches on these frontend-owned enums inside `mizar-frontend` remain
exhaustive; non-exhaustive enums owned by upstream crates use explicit frontend
fallbacks such as the syntax diagnostic key above.

With the stub parser seam, `ast = None` is the expected placeholder result. The
real parser seam returns a minimal `SurfaceAst` for recovered token streams and
may return `ast = None` when parsing cannot recover enough structure for
downstream phases. The lexical, preprocessing, and syntax diagnostics are still
returned.

## Dependencies

- Internal: `source`, `preprocess`, `lexical_env`, `lexing`, `parsing`,
  `cache_key`, `span_bridge` (constructed once and threaded through the phases).
- External: `mizar-session` (`SourceId`, `SourceRange`, `SourceAnchor`,
  `NormalizedPath`, `DocumentUri`, `SessionIdAllocator`, `BuildSnapshotId`),
  `mizar-lexer`, `mizar-syntax`, `mizar-parser`, and `std::path::PathBuf`.

This module is the public entry point of the crate; it is consumed by the
compiler driver, LSP, the formatter, and tests.

## Data Structures

### Frontend Output

`FrontendOutput` bundles each phase artifact plus the merged diagnostics and
frontend content cache keys. It is the unit later phases (module/name
resolution) consume: they read `ast` and `tokens`, and they read
`source`/`preprocessed` for spans, comments, and import stubs. Each artifact's
content key is exposed in `cache_keys` per
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Incrementality", so a comment-only edit can reuse semantic outputs while a
dependency export change invalidates tokenization. Cache storage and scheduler
task keys remain outside this crate.

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
5. Derive a pre-tokenization parser-input policy from the active lexical
   environment and source edition, then derive the position-sensitive
   `ParserLexingPlan` from the preprocessed lexical text.
6. Tokenize (`lexing`) into a `TokenStream` with `TokenizeRequest::with_plan`
   and `TokenizeRequest::with_current_module`, retaining the plan and current
   module local declarations on the token stream. Propagate a `SpanBridgeError`
   as `FrontendError`.
7. Build the final `ParserInputs` from the active lexical environment and the
   token stream's local declarations. Local operator metadata activation offsets
   are mapped through `SpanBridge` so the parser entries use the same source byte
   coordinate space as token spans.
8. Parse (`parsing`) through the configured `ParserSeam` into an optional AST.
9. Compute the frontend content cache-key bundle for the phase artifacts.
10. Map every phase diagnostic into `FrontendDiagnostic`, merge in the
   deterministic order above, and assemble `FrontendOutput`.

The cache-key bundle is the frontend dependency footprint for source,
preprocessing, lexing, and parsing. It is not a semantic `DependencySlice` and
does not record theorem, definition, registration, or proof facts.

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
`SourceLoadLocation::NormalizedPath` and `SourceLoadLocation::Unknown` are
reserved fallback locations: the former is for future non-exhaustive
`SourceOriginInput` variants that still carry a normalized input path, and the
latter is for future source-load diagnostics that cannot be tied to even that
path. They are public and sorted deterministically, but they have no current
runtime producer.
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
- local operator declarations collected during tokenization are included in the
  final parser inputs with source-coordinate activation offsets;
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
- reserved source-load fallback locations and the reserved `AnnotationSyntax`
  class have deterministic local ordering even without current producers;
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

## `PARSER-RECOVERY-B1B1P-P1-FE` Merge-Order Assessment

The parser adds no diagnostic variant or frontend class. Existing
orchestration sorting is independent of the parser recovery shape, so no
production merge logic changes. The exact nine-case real-pipeline regression
must compare full diagnostic vectors and recovery-bearing AST output across
two runs, proving the newly returning outputs enter the existing total order
deterministically.

Implementation result: the real-provider regression compares the full
diagnostic vectors and AST/cache outputs for every included mutation. No merge
logic or diagnostic class/code changed.

## `PARSER-TEMPLATE-TYPEHEAD-277P1` Replay Contract

`task277p1_template_typehead_changes_ast_cache_namespace` now runs the real
parser seam twice for the frozen source, observes the exact complete
zero-diagnostic 57-node AST and v3 cache key, and pins the unchanged control's
seven-node shape/ranges across replay. The test passes. Orchestration remains
payload-agnostic: no merge-order algorithm, diagnostic class, resolver input,
or semantic result changes.

## Disk FrontendOutput storage

`FrontendOutput<SurfaceAst>::canonical_disk_bytes() -> Option<Vec<u8>>` and
`from_canonical_disk_bytes(bytes: &[u8], source_id: SourceId, input: &SourceInput) -> Option<Self>`
retain the complete disk-source aggregate. This concrete specialization adds no generic codec
trait. SourceInput is caller-validated current session metadata, as required by the
[source codec](./source.md); open buffers and generated sources are outside this format.
No loader, provider, lexer, parser, source-map registration or publication runs on decode.

Wire bytes start with ASCII `mizar-frontend-output-disk-v1`, followed by exactly six fields,
each prefixed by its byte length as little-endian u64: source, preprocessed, tokens, AST,
merged diagnostics, cache keys. The fields reuse respectively SourceUnit disk storage,
[preprocessing storage](./preprocess.md#preprocessedsource-storage),
[token storage](./lexing.md#tokenstream-storage), SurfaceAst canonical storage, the diagnostic
array below, and [cache-key storage](./cache_key.md#retained-cache-key-storage).
An empty AST field means None; a present AST uses its nonempty canonical bytes.
The entire payload, including framing, is at most 64 MiB. Length arithmetic is checked;
truncation, trailing bytes, unknown prefix, invalid nested payloads and noncanonical bytes
return None. Re-encoding must reproduce the exact complete input.

All typed SourceIds must equal output.source.source_id on encoding and become the caller's
source_id on decoding. Each nested codec applies its own stricter validation. Cache-key path
must equal source.normalized_path; other retained key versions/relationships and AST-key
presence remain opaque. No cross-artifact source-text, bounds, provenance, producer legality,
cache freshness or proof acceptance validation is implied. The driver owns those checks.

Merged diagnostics are canonical UTF-8 JSON: an ordered array preserving duplicates, each
entry `[code, message, class, primary_range, secondary_anchors, recovery_note]`. Message and
optional recovery-note strings retain exact text; None is null. Primary locations must be
SourceRange; SourceLoad locations are rejected because source-load failures have no aggregate.
Ranges/anchors reuse [span storage](./preprocess.md#preprocessedsource-storage), including
ordered usize ranges, current-ID checks and nonblank generated reasons. No sorting occurs.
Diagnostic class tags are zero-based: SourceLoad, LexicalPrecondition, CommentStructure,
ImportPrescan, LexicalEnvironment, ScopeSkeleton, Tokenization, Syntax, AnnotationSyntax.
Code is `[tag]` for the following zero-based ordered vocabulary, or `[28, syntax_string]`:

| Tags | Codes in order |
|---|---|
| 0 | SourceLoad |
| 1–3 | Preprocess.SourcePrecondition: CarriageReturn, NonAsciiCode, UnterminatedMultiLineComment |
| 4–8 | Preprocess.ImportPrescan: MissingModulePath, EmptyModulePathComponent, MissingAlias, MissingSemicolon, UnexpectedToken |
| 9 | Preprocess.RawImportScan |
| 10–16 | LexicalEnvironment: UnresolvedImport, MissingSummary, UserSymbolImportConflict, InvalidUserSymbolSpelling, InvalidUserSymbolArity, ReservedWordCollision, ReservedSymbolCollision |
| 17 | Lexing.RawScan |
| 18–22 | Lexing.ScopeSkeleton: MalformedBinderList, UnsupportedBinderShape, DuplicateBindingName, UnmatchedEnd, MissingEnd |
| 23–27 | Lexing.Lexer: NoValidTokenCandidate, ParserContextRejectedCandidate, AmbiguousUserSymbol, MalformedStringLiteral, UnsupportedRawToken |

Tags are independent of Rust discriminants; unknown tags and wrong shapes/types fail closed.
Code/class combinations are transported without reclassification, including reserved values.
Tests cover real recovered/valid/absent-AST output, fresh IDs and relocated/deleted source
files, nested-byte equality, every diagnostic tag with independent typed oracles, all anchor
forms and text, malformed framing/nested records, foreign IDs, path/origin rejection and
payload limits. The source and phase codecs retain their existing standalone tests.

## Shared diagnostic adoption (specified, not implemented)

The public meanings and reserved codes are owned by [specification 22.2.3](../../../spec/en/22.error_handling_and_diagnostics.md#2223-frontend-diagnostic-adoption); this bridge is not yet active. The exact frontend discriminator mapping is:

| Local discriminator | Public code(s), in listed order |
|---|---|
| Preprocess.SourcePrecondition: CarriageReturn, NonAsciiCode, UnterminatedMultiLineComment | E0013–E0015 |
| Preprocess.ImportPrescan: MissingModulePath, EmptyModulePathComponent, MissingAlias, MissingSemicolon, UnexpectedToken; Preprocess.RawImportScan | E0016–E0021 |
| LexicalEnvironment: UnresolvedImport, MissingSummary, UserSymbolImportConflict, InvalidUserSymbolSpelling, InvalidUserSymbolArity, ReservedWordCollision, ReservedSymbolCollision | E0022–E0028 |
| Lexing.RawScan; Lexing.ScopeSkeleton: MalformedBinderList, UnsupportedBinderShape, DuplicateBindingName, UnmatchedEnd | E0029–E0033 |
| Lexing.ScopeSkeleton.MissingEnd; Lexing.Lexer.MalformedStringLiteral | E0010; E0002 |
| Lexing.Lexer: NoValidTokenCandidate, ParserContextRejectedCandidate, AmbiguousUserSymbol, UnsupportedRawToken | E0034–E0037 |
| Syntax keys unexpected_error_token, dangling_operator, non_associative_operator_chain | E0038–E0040 |
| Syntax key missing_end; keys missing_semicolon, missing_string_literal, malformed_import, malformed_export, malformed_visibility, malformed_type_expression, malformed_term_expression, malformed_formula_expression, malformed_justification, malformed_annotation, unexpected_top_level_token, unrecoverable_input | E0010; E0041–E0052 |

Syntax keys are an exact allowlist; `syntax_diagnostic` and other strings are unsupported, including values accepted by storage codecs. SourceLoad needs the original typed SourceLoadError for the existing E0600–E0603 mapping; its erased aggregate code alone is insufficient. No message-based classification is permitted.
The bridge preserves each diagnostic before shared aggregation; it does not change the coordinator's merge order or shared deduplication. [Shared span adoption](../../mizar-diagnostics/en/failure_record.md#frontend-anchor-adoption) is required before conversion, and successful storage decoding alone does not establish source binding or publication authority.
