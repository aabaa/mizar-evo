# mizar-frontend TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

| Module | Spec | Source | Status |
|---|---|---|---|
| span_bridge | [span_bridge.md](./span_bridge.md) | `src/span_bridge.rs` | [x] |
| source | [source.md](./source.md) | `src/source.rs` | [x] |
| preprocess | [preprocess.md](./preprocess.md) | `src/preprocess.rs` | [x] |
| lexical_env | [lexical_env.md](./lexical_env.md) | `src/lexical_env.rs` | [x] |
| lexing | [lexing.md](./lexing.md) | `src/lexing.rs` | [x] |
| parsing | [parsing.md](./parsing.md) | `src/parsing.rs` | [x] implemented through task 28 current parser growth |
| cache_key | [cache_key.md](./cache_key.md) | `src/cache_key.rs` | [x] |
| orchestration | [orchestration.md](./orchestration.md) | `src/orchestration.rs` | [x] implemented through task 28 current parser growth |

`mizar-frontend` is an orchestration crate, so it is built bottom-up by phase:
the coordinate bridge first, then pipeline Steps 1-5 in order, then the
end-to-end coordinator. `span_bridge` is the shared primitive every later step
references; `orchestration` is the only module that wires the full pipeline.

Dependency order: `span_bridge` → `source` → `preprocess` → `lexical_env` →
`lexing` → `parsing` → `cache_key` → `orchestration`.

## Crate Prerequisites

The frontend foundation began with `mizar-session` and `mizar-lexer` only. Task
11 added hard dependencies on the minimal `mizar-syntax::SurfaceAst` boundary
and `mizar-parser` entry point needed by the real parser seam. Task 12 added the
minimal parser recovery passthrough on that boundary. Tasks 1-10 and the task
13-14 coordinator paths remain valid with `StubParserSeam`; task 14 failure
assertions and real-parser assertions build on the task-12 parser/syntax
boundary and the task-13 coordinator.

## Resolved And Gated Decisions

These public API decisions are tracked at the top level in
[../../todo.md](../../todo.md) "Resolved And Open Decisions":

- **Lexer span bridging: resolved.** This crate adopts the decoupled option: `mizar-lexer`
  keeps its byte-offset spans and `span_bridge` (task 1) maps them onto
  `mizar-session` `SourceRange`.
- **Parser-assisted lexing contract: resolved.** The frontend precomputes a
  position-sensitive `ParserLexingPlan` over lexical byte ranges and passes only
  narrow `ParserLexContext` values to the lexer. The parser and lexer do not
  interleave, and the lexer never receives arbitrary parser state. The plan now
  covers grammar-position string literals, Unicode inside annotation string
  arguments, and parser-driven user-symbol kind filters.
- **Quality bar before the next crate: resolved.** Task 25 is complete, so
  there are no remaining frontend-side gates before development moves to the
  next crate. Tasks 27 and 28 are complete for the current parser/recovery
  surface; future `mizar-parser` grammar/recovery growth should open a new
  frontend follow-up rather than leaving hidden work in this crate.

## Ordered Task List

Each task is sized to be implemented, tested, and committed on its own. The
listed dependency lines are authoritative. The early stubbed source → tokens
coordinator tasks were allowed to proceed before `mizar-parser` / `mizar-syntax`
existed; task 11 now provides minimal crates for the real parser seam. Every task
should keep `cargo test -p mizar-frontend` green (see
[Suggested Verification](#suggested-verification)).

### Crate scaffolding

1. **Crate skeleton and coordinate bridge.** [x]
   - Add the `mizar-frontend` crate to the workspace with dependencies on
     `mizar-session` and `mizar-lexer`; opt in to the workspace
     `[workspace.lints]` table via `lints.workspace = true` (matching
     `mizar-session`).
   - Add `pub mod span_bridge;` and define `SpanBridge`, `LexerByteSpan`, and
     `SpanBridgeError`; let `SpanBridge` own a retained session source-map
     service, implement fallible `register_source` / `register_preprocess_map`,
     and provide `loaded_span`, `loaded_mapping`, and `lexical_span`
     conversions over the retained `mizar-session` maps.
   - Make `SpanBridgeError` distinguish frontend-local registration invariants
     (`SourceNotRegistered`, `PreprocessMapNotRegistered`, conflicting source /
     preprocess registrations) from wrapped `mizar-session::SourceMapError`.
   - Reuse the optional session `LoadingMap` attached to the `SourceUnit` and
     derive the session-side `PreprocessMap` from the lexer `SourcePreprocessMap`;
     do not synthesize or retain `LoadingMap::identity` for identity loads. When
     `loaded_mapping` sees no loading map, it returns an exact loaded-coordinate
     mapping with `original_input = None` after line-map validation.
   - Tests: `loaded_span` over BOM-stripped text stays in loaded-text
     coordinates, while `loaded_mapping` reports original input offsets through
     `MappedSourceRange.original_input`; lexical span maps through the preprocess
     map to loaded-source coordinates; an identity load without `LoadingMap`
     returns exact `loaded_mapping` with `original_input = None`; a span crossing
     a removed comment yields primary plus secondary anchors; a synthetic-only
     lexical span yields a degraded anchor-backed mapping, not an exact primary
     source range; non-UTF-8-boundary and out-of-range spans are rejected;
     conflicting map registration is reported.
   - Spec: [span_bridge.md](./span_bridge.md) "Public API", "Algorithm / Logic".

### Module: source (`src/source.rs`)

2. **`SourceUnit` and the loader bridge.** [x]
   - Add `pub mod source;`. Define `SourceUnit`, `SourceUnitRequest`, the
     `SourceUnitLoader` trait, and `FrontendSourceLoader<L: SourceLoader>`;
     implement `source_unit_from_loaded` projecting a `mizar_session::LoadedSource`
     into a `SourceUnit` without recomputing hash, line map, loading map,
     normalized path, edition, origin, or generated anchor.
   - Treat `file_path` as caller-provided diagnostic metadata because
     `LoadedSource` does not store a filesystem path; derive it from the
     request/origin for disk or open-buffer sources and from the normalized path or
     generated anchor for generated sources.
   - Provide a helper that registers the loaded `LineMap` / `LoadingMap` with a
     mutable `SpanBridge` under the `SourceId`; orchestration calls this helper
     after loading and before preprocessing, while `load_source_unit` itself does
     not mutate bridge state.
   - Tests: disk `LoadedSource` projects with identical id/hash/line-map/loading-map;
     BOM/CRLF-normalized source carries `Some(loading_map)`; identity load carries
     `None`; normalized path and edition are preserved; open-buffer origin and
     version are preserved; open-buffer diagnostic paths decode local `file://`
     URI paths and fall back to `normalized_path` when the URI path is unusable;
     generated sources preserve `generated_anchor`; `register_source_unit` records
     line/loading maps and reports conflicting duplicate registrations; a session
     `SourceLoadError` is propagated unchanged.
   - Depends on: 1. Spec: [source.md](./source.md) "Public API",
     "Algorithm / Logic".

### Module: preprocess (`src/preprocess.rs`)

3. **Comment and doc-comment preprocessing.** [x]
   - Add `pub mod preprocess;`. Define `PreprocessedSource`, `LexicalText`,
     `Comment`, `DocComment`, `LexicalSourceMap`, `lexical_hash`, mapped `ImportStub` /
     `ImportStubPath` / `ImportStubRelativePrefix` / `ImportStubAlias`, and
     `PreprocessDiagnostic` with code/message, primary `SourceRange`, and
     secondary `SourceAnchor`s.
   - Drive `mizar_lexer::preprocess_source_for_lexing` over `SourceUnit.source_text`,
     map comment / doc-comment / preprocess-diagnostic spans through the
     `SpanBridge`, and assemble the `LexicalSourceMap`.
   - Tests: ordinary comments removed from lexical text but retained as `Comment`s
     with correct ranges; doc comment preserved with raw body and range; annotation
     syntax stays in lexical text; a lexical range crossing a removed comment yields
     a composite mapping; synthetic whitespace is represented only as a degraded
     anchor-backed mapping; `lexical_hash` is stable when comment-only edits leave
     lexical text unchanged; a code-region non-ASCII char and an unterminated block
     comment are reported and recovered.
   - Depends on: 2. Spec: [preprocess.md](./preprocess.md) "Comments and Doc
     Comments", "Algorithm / Logic".

4. **Shallow import pre-scan integration.** [x]
   - Recoverably raw-scan lexical text (`scan_raw_recoverable`) and run
     `mizar_lexer::scan_import_prelude`; populate `import_stubs` with mapped
     `SourceRange`s and collect `ImportPrescanDiagnostic`s plus precise
     `RawImportScan` diagnostics into `diagnostics`.
   - If recoverable raw scanning reports diagnostics, map each offending span
     precisely and continue import pre-scan over the usable partial raw tokens.
     Whole-lexical-text fallback is reserved for internal scanner/plan invariant
     failures rather than user-authored malformed raw input.
   - Tests: top-level `import` forms produce `ImportStub`s with raw path, optional
     alias, `path.relative`, `path.source_segments`, and span; `.` and `..`
     relative prefixes remain distinguishable as current and parent imports; a
     malformed import yields an import-prescan diagnostic without aborting;
     raw-scan recovery during import pre-scan yields precise diagnostics while
     preserving usable partial `import_stubs`; import order is preserved for
     provenance and deterministic fingerprints.
   - Depends on: 3. Spec: [preprocess.md](./preprocess.md) "Import Stubs",
     "Error Handling".

### Module: lexical_env (`src/lexical_env.rs`)

5. **Lexical environment request and provider seam.** [x]
   - Add `pub mod lexical_env;`. Define `LexicalEnvironmentRequest`,
     `LexicalSummaryProvider`, `ResolvedImports`, `ResolvedImportEntry`,
     `ActiveLexicalEnvironmentResult`, `LexicalEnvironmentDiagnostic`,
     `LexicalEnvironmentDiagnosticCode`, and `FrontendLexicalEnvironmentError`;
     re-export the `mizar-lexer` environment types.
   - Preserve each resolved import's originating `ImportStub` ordinal and span in
     `ResolvedImportEntry`; pass only the ordered lexer `ResolvedImport`s to
     `mizar_lexer::build_lexical_environment`.
   - Canonicalize resolved imports by `ModuleId` before the lexer call, keeping the
     first stub in import order as the active provenance entry and preserving
     duplicate-stub provenance for provider diagnostics. Do not pass duplicate
     module ids to the current lexer, because conflict errors identify modules but
     not frontend import ordinals.
   - Use `FrontendLexicalEnvironmentError`, not the lexer-owned
     `LexicalEnvironmentError`, for provider infrastructure failures.
   - Tests: a fake provider that returns resolved imports + summaries produces a
     `UserSymbolIndex` with correct canonical provenance and import ordinal;
     duplicate stubs resolving to the same module are deduplicated before the
     lexer call and provider diagnostics still point at the correct originating
     import span; reserved tables are always present.
   - Depends on: 4. Spec: [lexical_env.md](./lexical_env.md) "Public API".

6. **Active lexical environment construction.** [x]
   - Extend the task-5 `build_active_lexical_environment` entry point with the
     remaining recovery behavior while preserving its call to
     `mizar_lexer::build_lexical_environment`, provider diagnostic merge, and
     surfaced `LexicalEnvironmentFingerprint`.
   - Omit unresolved imports and imports whose dependency lexical summary is
     unavailable before calling the lexer; represent them as
     `LexicalEnvironmentDiagnostic`s so the lexer is not asked to build with
     missing summaries.
   - Handle `LexicalEnvironmentError::UserSymbolImportConflict` with a bounded
     deterministic retry: diagnose the later conflicting import using
     canonical `ResolvedImportEntry` provenance, add the earlier import as
     secondary context when known, remove the later conflicting module, and retry
     at most once per original canonical import. Treat every other lexer
     `LexicalEnvironmentError` as
     `FrontendLexicalEnvironmentError::MalformedSummary`.
   - Tests: equal-spelling user symbols imported from different modules produce a
     deterministic lexical-environment conflict diagnostic and retry by dropping
     the later conflicting module; duplicate imports of the same module do not
     create spurious conflicts; an unresolved import degrades to a smaller
     environment with a diagnostic while remaining symbols load; a missing summary
     is omitted before the lexer call and diagnosed; non-conflict lexer
     environment errors become `MalformedSummary`; the fingerprint changes when a
     dependency summary changes and is stable for comment-only local edits.
   - Depends on: 5. Spec: [lexical_env.md](./lexical_env.md) "Algorithm / Logic",
     "Error Handling".

### Module: lexing (`src/lexing.rs`)

7. **Raw scan and scope skeleton wiring.** [x]
   - Add `pub mod lexing;`. Define `TokenizeRequest`, `InternedText = Arc<str>`,
     the frontend `Token` /
     `TokenStream` (session-spanned), `LexingDiagnostic`, and
     `LexingDiagnosticKind` / `LexingDiagnosticPayload`; re-export `TokenKind` and
     raw lexer diagnostic code enums such as `LexDiagnosticCode` /
     `ScopeSkeletonDiagnosticCode`.
   - Build the `ScopeSkeleton` / `ScopeLexView` from raw tokens and prepare the
     disambiguator inputs; map `ScopeSkeletonDiagnostic`s into `LexingDiagnostic`s
     without storing raw span-bearing diagnostic structs in the public
     `TokenStream`.
   - Tests: raw scan preserves `LexemeRun` spans; the scope view reflects lexical
     block/statement shape without resolved bindings.
   - Depends on: 6. Spec: [lexing.md](./lexing.md) "Scope Lex View",
     "Algorithm / Logic".

8. **Context-sensitive disambiguation to `TokenStream`.** [x]
   - Run `disambiguate` (or parser-integrated `lex`) with the active lexical
     environment, an initial raw `ScopeSkeleton` / `ScopeLexView`, a contextual
     scope skeleton rebuilt from final token shapes, and the current
     `ParserLexContext` selected by the parser-assisted lexing plan; map every
     lexer token and diagnostic span through the
     `SpanBridge` to session `SourceRange`s. Convert raw `LexDiagnostic`s to
     frontend `LexingDiagnostic`s
     by copying code/message and preserving structured payloads in mapped form;
     map nested rejected-candidate spans to session ranges and preserve secondary
     `SourceAnchor`s from composite/degraded mappings. Return
     `Err(SpanBridgeError)` only for internal mapping invariant
     failures.
   - Tests: a user symbol sharing spelling with an identifier is classified by
     longest-match; compound reserved tokens (`..`, `.{`, `.*`, `.=`, `...`) lex as
     single tokens; quote-delimited spelling is rejected with mapped lexer
     diagnostics under the general context unless supplied by the active lexicon,
     while a bounded uniform `StringRequired` context produces a `StringLiteral`;
     every emitted token span resolves to a valid primary `SourceRange` while
     secondary anchors are preserved for diagnostics; lexer payloads with
     rejected token candidates preserve non-span payload data and mapped nested
     spans. Task 20 adds position-specific annotation string-literal coverage.
   - Depends on: 7. Spec: [lexing.md](./lexing.md) "Token Stream",
     "Algorithm / Logic".

9. **Lexer recovery passthrough.** [x]
   - Preserve `TokenKind::ErrorRecovery` spans and lexer diagnostics end to end as
     mapped `LexingDiagnostic`s; add recoverable disambiguator / lexer
     diagnostics so the frontend `tokenize` wrapper returns `Ok(TokenStream)` for
     recoverable input problems.
   - Tests: a malformed token emits `ErrorRecovery` with the correct `SourceRange`
     and scanning resumes; invalid-numeral diagnostics, when the lexer exposes
     them, and unsupported raw-token cases are reported without dropping
     recoverable tokens; scope-skeleton diagnostics remain preserved with mapped
     spans after disambiguation.
   - Depends on: 8. Spec: [lexing.md](./lexing.md) "Error Handling".

### Module: parsing (`src/parsing.rs`)

10. **Parser-input assembly and parser seam.** [x]
    - Add `pub mod parsing;`. Define `ParseRequest`, `ParserInputs`,
      `OperatorFixityTable`, `OperatorFixityEntry`, `OperatorFixity`,
      `OperatorAssociativity`, `StringRequiredContext`, `ParseOutput`,
      `ParserSeam`, and `StubParserSeam`;
      derive `ParserInputs` after the active lexical environment is built, using
      the source edition plus only the data currently exposed by lexical
      summaries.
    - Until `mizar-parser` exists, implement the seam against a stub that returns
      `ast = None` plus an empty diagnostic list, so the source → tokens pipeline
      is exercisable.
    - Tests: `ParserInputs` carries the edition, derives operator-fixity
      entries only when summaries expose fixity, uses
      `StringRequiredContext::PositionSensitive` for normal source-to-token
      paths, carries no resolver state, and the stub seam returns `ast = None`.
    - Depends on: 8. Spec: [parsing.md](./parsing.md) "Parser Inputs",
      "Public API".

11. **`mizar-parser` invocation.** [x]
    - Added minimal `mizar-syntax` and `mizar-parser` crates, plus
      `MizarParserSeam`, which adapts the frontend `TokenStream` and
      `ParserInputs` into the parser entry point and returns
      `mizar_syntax::SurfaceAst` plus syntax diagnostics unchanged.
    - `StubParserSeam` remains available for stubbed coordinator paths.
    - Tests: a well-formed token stream parses to a `SurfaceAst` with preserved
      source order and ranges; explicit operator fixity drives Pratt precedence
      for user operators. Task 12 added summary-derived prefix/postfix/infix
      fixity coverage through the active source path. Task 20 adds real
      source-text coverage for annotation string literals.
    - Depends on: 10, plus `mizar-parser`/`mizar-syntax`. Spec:
      [parsing.md](./parsing.md) "Algorithm / Logic".

12. **Parser recovery passthrough.** [x]
    - Preserve `ast = None` on unrecoverable input and explicit recovery-node
      markers inside a returned `SurfaceAst`; carry syntax diagnostics through.
    - Tests: a missing `end` recovers conservatively at EOF when block-stack
      matching leaves an opener unclosed, with `ast = Some` and an explicit error node; an unrecoverable
      one-token `end` stream returns `ast = None` with diagnostics; a missing
      string literal at a uniform string-required position yields the expected
      syntax diagnostic using a synthetic token stream.
    - Depends on: 11. Spec: [parsing.md](./parsing.md) "Error Handling".

### Module: orchestration (`src/orchestration.rs`)

13. **Frontend coordinator and diagnostic merge.** [x]
    - Add `pub mod orchestration;`. Define `FrontendOutput`, `Frontend`,
      `FrontendDiagnostic`, `DiagnosticLocation`, `SourceLoadLocation`,
      `DiagnosticCode`, `DiagnosticClass`, and `FrontendError`; wire
      `source` → `preprocess` → `lexical_env` → `lexing` → `parsing` and merge all
      phase diagnostics, including import-pre-scan, lexical-environment,
      scope-skeleton, and tokenization diagnostics, into the deterministic order in
      [orchestration.md](./orchestration.md) "Diagnostic Merge Order".
    - `FrontendDiagnostic` carries code, message, class, `DiagnosticLocation`,
      secondary `SourceAnchor`s, and optional recovery note. Range-backed
      diagnostics use `DiagnosticLocation::SourceRange`; source-load diagnostics
      use `DiagnosticLocation::SourceLoad` with the best available path,
      normalized path, open-buffer URI, generated anchor, or `Unknown`.
      `FrontendError` distinguishes source-load, span-bridge, and
      lexical-environment hard failures.
    - Tests: with `StubParserSeam`, a well-formed source returns source,
      preprocessing output, tokens, `ast = None`, and no parser diagnostics; merge
      order is identical across repeated runs, including same-class diagnostics
      with the same start and diagnostic code. With the real parser seam, add the
      `ast = Some` and syntax-diagnostic ordering assertions.
    - Depends on: 9, 10. Real-parser assertions depend on 12. Spec:
      [orchestration.md](./orchestration.md)
      "Algorithm / Logic", "Diagnostic Merge Order".

14. **Unrecoverable-failure handling and end-to-end output.** [x]
    - Broaden coverage around the already wired `FrontendError` paths for Step 1
      load failures, `SpanBridgeError` invariant violations from source
      registration / preprocessing / lexing, and
      `FrontendLexicalEnvironmentError` from lexical-environment construction;
      keep recoverable problems as diagnostics inside `FrontendOutput`.
    - Tests: complete assertions that Step 1 load failures return
      `FrontendError` with a file-level `DiagnosticLocation::SourceLoad`
      diagnostic and no output; source-load diagnostics do not fabricate
      zero-length `SourceRange`s; a parser seam that returns `ast = None`
      preserves earlier diagnostics; range-backed merged diagnostics carry valid
      `SourceRange`s; span-bridge and lexical-environment hard-failure fixtures
      return the matching `FrontendError` variants.
    - Depends on: 13. Spec: [orchestration.md](./orchestration.md) "Error
      Handling".

### Module-wide maintenance before cross-cutting follow-ups

15. **Implementation refactoring pass.** [x]
    - Review `span_bridge`, `source`, `preprocess`, `lexical_env`, `lexing`,
      `parsing`, and `orchestration` once the first implementation pass is complete.
    - Keep public APIs and behavior stable unless the refactor exposes a clear bug
      or spec mismatch; prefer small local extractions and shared test fixtures.
    - Tests: keep all module tests green.
    - Depends on: 14. Spec: all mizar-frontend module specs.

16. **Source/spec correspondence audit.** [x]
    - Build a lightweight traceability check from each public API, error variant,
      and task requirement in the frontend specs to the implementing source/tests.
    - Record any missing implementation, stale spec text, or missing tests as
      follow-up tasks rather than mixing broad changes into the audit.
    - Check the English canonical specs first, then verify Japanese companions
      carry the same API and behavioral commitments.
    - Result: [source_spec_correspondence.md](./source_spec_correspondence.md)
      records the audit. The audit added task 24 for reserved or currently
      unproduced diagnostic/fallback surface coverage.
    - Depends on: 15. Spec: all mizar-frontend module specs and this TODO.

## Cross-Cutting Follow-up Tasks

17. **Bilingual documentation synchronization audit.** [x]
    - Compare every English canonical document under
      `doc/design/mizar-frontend/en/` with its Japanese companion under
      `doc/design/mizar-frontend/ja/`; synchronize API lists, task statuses,
      terminology, links, and behavior commitments.
    - Result: [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
      records the audit. English canonical documents and Japanese companions are
      synchronized for public API/error lists, module and task statuses,
      terminology, companion-local links, and behavior commitments. No
      unsynchronized Japanese companion gap remains.
    - Depends on: 16. Spec: repository documentation policy.

18. **Frontend determinism property tests.** [x]
    - Add crate-level coverage that identical inputs produce identical
      `FrontendOutput` diagnostics order and identical token spans independent of
      internal scheduling, and that the `LexicalEnvironmentFingerprint` and cache
      keys are stable for equivalent inputs.
    - Result: `crates/mizar-frontend/tests/determinism.rs` covers provider
      scheduling permutations for frontend diagnostic order and token spans, plus
      comment-equivalent cache-key stability for `lexical_hash`,
      `LexicalEnvironmentFingerprint`, and parser context.
    - Depends on: 16. Spec: [orchestration.md](./orchestration.md),
      [lexical_env.md](./lexical_env.md).

19. **Incremental cache-key wiring.** [x]
    - Decide where the layered frontend cache keys from
      [architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
      "Incrementality" are computed and stored (this crate vs. the driver/artifact
      layer), and expose the per-artifact keys (`SourceUnit`, `PreprocessedSource`,
      `ActiveLexicalEnvironment`, `TokenStream`, `SurfaceAst`) accordingly.
    - Verify comment-only edits can reuse semantic outputs while import / dependency
      export edits and parser lexing context / parser-assisted lexing-plan changes
      invalidate tokenization and downstream layers.
    - Result: [cache_key.md](./cache_key.md) documents the split: this crate
      computes deterministic frontend content keys and returns them through
      `FrontendOutput.cache_keys`, while the driver/artifact layer owns cache
      storage, validation, and task-key composition. Unit tests cover source,
      preprocessing, lexical-environment, token-stream, and AST key invalidation;
      `tests/determinism.rs` now asserts the crate-level frontend cache keys for
      comment-equivalent runs and end-to-end import/dependency invalidation.
    - Depends on: 16. Spec: architecture incrementality table.

20. **Parser-assisted lexing contract finalization.** [x]
    - Finalize whether disambiguation runs in one pass with a precomputed
      position-sensitive `ParserLexContext` or interleaves with parsing through
      the narrow context object, and document the chosen integration in
      [lexing.md](./lexing.md) and [parsing.md](./parsing.md).
    - Keep the lexer free of arbitrary parser state under either choice.
    - Result: the contract chooses precomputed, position-sensitive lexing plans.
      `ParserInputs` uses `StringRequiredContext::PositionSensitive` for normal
      source runs, orchestration derives one `ParserLexingPlan` after
      preprocessing, `TokenizeRequest::with_plan` uses it for tokenization,
      `TokenStream` retains it, and token cache keys hash its actual range and
      context content. Preprocessing and import pre-scan use matching recognized
      string-argument protection before recoverable raw scanning. Tests cover
      single-line annotation string arguments with Unicode/comment-marker
      contents, line-boundary guards, range-specific user-symbol kind filters,
      and real source-to-token-to-parser
      handoff through `MizarParserSeam`.
    - Depends on: 10; real-parser validation also depends on 11. Spec: top-level
     [../../todo.md](../../todo.md) "Resolved And Open Decisions", [lexing.md](./lexing.md),
     [parsing.md](./parsing.md).

21. **Durable lint enforcement.** [x]
    - Confirm `crates/mizar-frontend/Cargo.toml` opts into the workspace
      `[workspace.lints]` table so `cargo build`/`cargo test` surface the same
      denials as the standalone clippy gate (matching the `mizar-session` policy).
    - Record any intentional `allow` exceptions with a rationale next to the `allow`.
    - Tests: `cargo clippy -p mizar-frontend --all-targets -- -D warnings` passes.
    - Depends on: 16. Spec: this TODO "Suggested Verification".
    - Result: `crates/mizar-frontend/Cargo.toml` already opts into the shared
      lint policy with `[lints] workspace = true`; `tests/lint_policy.rs` now
      guards that opt-in, the workspace `warnings = "deny"` and
      `clippy::all = "deny"` baseline, and the requirement that any future
      frontend `allow` attribute carries an adjacent reason. No intentional
      `allow` exceptions are currently present.

22. **Precise raw-scan recovery contract.** [x]
    - Decide whether `mizar-lexer` should expose a recoverable raw scanner that
      returns failure spans and partial raw tokens, or whether `mizar-frontend`
      keeps only coarse full-lexical-text recovery for strict `scan_raw` failures.
    - If a recoverable raw scanner is added, update [preprocess.md](./preprocess.md)
      and [lexing.md](./lexing.md) to replace the coarse diagnostics/recovery token
      with precise failure spans and synchronization-boundary continuation.
    - Tests: strict `scan_raw` failure remains coarse until this contract lands;
      after it lands, import pre-scan and tokenization report the precise
      offending span, preserve usable partial raw tokens, and keep error
      sentinels at recovery boundaries so callers do not join malformed text.
    - Depends on: 9. Spec: [preprocess.md](./preprocess.md), [lexing.md](./lexing.md).
    - Result: `mizar-lexer` now exposes strict `scan_raw` plus
      `scan_raw_recoverable`, which returns `RecoverableRawTokenStream` with
      usable partial raw tokens, error sentinels, and precise
      `RawScanDiagnostic`s. `mizar-frontend`
      uses the recoverable path for import pre-scan and tokenization, maps
      offending spans through `SpanBridge`, preserves import stubs and tokens
      found in the usable partial stream, and keeps the older whole-text fallback
      only for internal parser-plan range defects. Tests cover lexer-side
      recovery continuation, precise import pre-scan diagnostics with preserved
      imports, and tokenization that emits precise `ErrorRecovery` tokens while
      continuing with later source tokens.

23. **Resident-set contract guard for the lexical environment.** [x]
    - Add coverage that locks the resident-set contract now stated in
      [lexical_env.md](./lexical_env.md) "Constraints and Assumptions": the active
      lexical environment holds only compact `ModuleLexicalSummary` projections of
      imported modules, never their definitions or full module IR, and the
      `LexicalSummaryProvider` is queried only for the current file's resolved
      imports rather than an eagerly expanded import closure.
    - Tests: a recording fake `LexicalSummaryProvider` receives exactly one
      `resolve_imports` call scoped to the request's `ImportStub`s and is never
      asked to expand to transitive imports; the resulting `ActiveLexicalEnvironment`
      exposes only summary-derived lexical shape and provenance (e.g., symbol
      spelling, kind, arity, symbol id, source/imported module, import ordinal,
      export rank — all lightweight `ModuleLexicalSummary`-derived data), with no
      API path requiring full dependency IR.
    - Depends on: 6. Spec: [lexical_env.md](./lexical_env.md) "Constraints and
      Assumptions"; resident-set memory model spec
      [§12.6.3](../../../spec/en/12.modules_and_namespaces.md#1263-memory-model).
    - Result: `tests/lexical_env_resident_set.rs` adds a recording
      `LexicalSummaryProvider` that proves `build_active_lexical_environment`
      asks exactly once for the current request's direct `ImportStub`s and never
      expands the import closure. The test also checks that the resulting
      `ActiveLexicalEnvironment` exposes only summary-derived lexical shape and
      provenance fields, and that a transitive fixture symbol is absent unless it
      appears in a direct `ModuleLexicalSummary`.

24. **Reserved frontend diagnostic surface coverage.** [x]
    - Review public variants that are reserved or have no current producer:
      `SpanBridgeError::UnsupportedLexerPreprocessMap`,
      `LexicalEnvironmentDiagnosticCode::{InvalidUserSymbolSpelling,
      InvalidUserSymbolArity, ReservedWordCollision, ReservedSymbolCollision}`,
      `SourceLoadLocation::{NormalizedPath, Unknown}`, and
      `DiagnosticClass::AnnotationSyntax`, plus
      `LexingDiagnosticPayload::UnsupportedLexerPayload`.
    - Keep lexer-owned malformed dependency summaries as
      `FrontendLexicalEnvironmentError::MalformedSummary` unless a provider-owned
      recoverable diagnostic contract is explicitly specified.
    - Decide for each reserved surface whether to keep it public without a
      producer, add direct coverage with a constructible fixture, or defer it
      until the producer exists. Update
      [source_spec_correspondence.md](./source_spec_correspondence.md) and the
      relevant module specs after the decision.
    - Tests: add coverage for constructible fallback/reserved variants, and add
      producer-backed tests when future lexer/session/parser contracts expose the
      remaining surfaces.
    - Result: constructible reserved surfaces are covered directly:
      `SpanBridgeError::UnsupportedLexerPreprocessMap` display/construction,
      provider-owned pass-through for the four reserved
      `LexicalEnvironmentDiagnosticCode`s, deterministic ordering for
      `SourceLoadLocation::{NormalizedPath, Unknown}` and
      `DiagnosticClass::AnnotationSyntax`, and the no-recovery-note policy for
      `LexingDiagnosticPayload::UnsupportedLexerPayload`. Producer-backed coverage
      remains deferred for future non-exhaustive lexer/session/parser variants.
    - Depends on: 16. Spec: [source_spec_correspondence.md](./source_spec_correspondence.md),
      [span_bridge.md](./span_bridge.md), [lexical_env.md](./lexical_env.md),
      [orchestration.md](./orchestration.md), [lexing.md](./lexing.md),
      [parsing.md](./parsing.md).

### Quality bar before the next crate

Task 25 was the only gate before next-crate development started. Tasks 26, 27,
and 28 are now complete for the current frontend/parser surface. Future parser
grammar growth should add a new follow-up task with the same passthrough,
diagnostic ordering, and cache-key checks.

25. **Public enum forward-compatibility decision.** [x]
    - For each public frontend enum whose spec already promises future variants
      or reserved surfaces — `SpanBridgeError`,
      `LexicalEnvironmentDiagnosticCode`, `LexingDiagnosticKind`,
      `LexingDiagnosticPayload`, `PreprocessDiagnosticKind`, `DiagnosticCode`,
      `DiagnosticClass`, `SourceLoadLocation`, and `FrontendError` — decide
      whether to mark it `#[non_exhaustive]` so future variants do not break
      downstream crates, or to deliberately keep exhaustive matching so a new
      variant is a compile-time signal inside the workspace.
    - Record the per-enum decision next to the enum in the owning module spec
      and in [source_spec_correspondence.md](./source_spec_correspondence.md);
      apply the chosen attributes in the same change.
    - This is the cheapest moment for the decision: no crate outside the
      frontend pipeline consumes these enums yet, so either choice is still
      free to apply.
    - Tests: `cargo test -p mizar-frontend` and the clippy gate stay green;
      matches on the listed frontend-owned enums inside this crate remain
      exhaustive either way.
    - Result: the listed public frontend enums are `#[non_exhaustive]` for
      downstream crates, while `mizar-frontend` keeps its internal matches on
      those frontend-owned enums exhaustive. The per-enum decision is recorded
      in the owning module specs and in
      [source_spec_correspondence.md](./source_spec_correspondence.md).
    - Depends on: 24. Spec: all module specs,
      [source_spec_correspondence.md](./source_spec_correspondence.md).

26. **Public API rustdoc summaries.** [x]
    - Original deferral rationale: before this task was reopened, no crate in
      the workspace carried rustdoc and the canonical API contracts lived in
      the `doc/design` specs. Adding `///` summaries only to `mizar-frontend`
      was treated as a workspace-level documentation decision rather than a
      frontend gap.
    - Re-entry trigger, now satisfied by this task: before the first long-lived consumer outside the
      frontend pipeline (the driver or `mizar-lsp`) starts coding against the
      `mizar-frontend` public API, or when the workspace adopts a rustdoc
      policy — whichever comes first.
    - Completed content: transcribe one-line summaries from each spec's
      "Public API" section onto the public items, link each module header to
      its spec, and keep `doc/design` canonical for behavior promises.
    - Result: `mizar-frontend` public modules and public API items now carry
      short rustdoc summaries derived from the canonical design specs; module
      headers point back to the owning `doc/design/mizar-frontend/en/` spec.
    - Depends on: 16. Spec: repository documentation policy.

27. **Frontend pipeline fuzz target and performance baselines.** [x] Complete.
    - Before this task, the workspace fuzz harness covered only
      `lexer_valid_utf8`. Add a frontend target that drives preprocess → import
      pre-scan → tokenize over arbitrary UTF-8 input with a stub summary
      provider, asserting no panics and recoverable-diagnostics-only outcomes
      on the recovery paths that tasks 9 and 22 promise.
    - Re-entry trigger (fuzz): task 28 satisfied the parser-recovery growth
      trigger, and task 29 completed the real-parser fuzz follow-up.
      Re-entry trigger (performance): when the driver's incremental loop exists
      and consumes `FrontendOutput.cache_keys`, extend the current
      full-pipeline baselines with true incremental timing for comment-only
      versus import-edit reruns.
    - Depends on: 22. Spec: [preprocess.md](./preprocess.md),
      [lexing.md](./lexing.md), [cache_key.md](./cache_key.md).
    - Completed in task 27: added `frontend_valid_utf8` under `fuzz/` using an
      empty summary provider and the stub parser seam, so arbitrary valid UTF-8
      exercises source loading, preprocessing/import pre-scan, active lexical
      environment recovery, tokenization, and diagnostic merging without hard
      frontend errors. Added Criterion baselines in
      `crates/mizar-frontend/benches/frontend_pipeline.rs` for cold full
      pipeline runs plus comment-only and import-edit full-pipeline edited
      fixtures that consume `FrontendOutput.cache_keys`; true driver
      incremental rerun timing remains reserved for the performance re-entry
      trigger above.

28. **Grammar-recovery follow-through with `mizar-parser` growth.** [x] Complete.
    - Completed follow-through for the current parser growth: nested block-end
      recovery now matches available `end` tokens before diagnosing still-open
      block starts, frontend seam tests preserve the new recovery node shape,
      orchestration tests merge the resulting syntax diagnostic, and
      `MIZAR_PARSER_CACHE_KEY_VERSION` invalidates `SurfaceAstCacheKey`s for
      the changed parser output semantics.
    - Future grammar/recovery expansion should open a new task that repeats the
      same checklist: recovery-node passthrough, syntax-diagnostic merge
      ordering, and `SurfaceAstCacheKey` invalidation for the new grammar
      shapes.
    - Depends on: 12, 13. Spec: [parsing.md](./parsing.md),
      [orchestration.md](./orchestration.md).

29. **Real-parser frontend fuzz follow-up.** [x] Complete.
    - The `frontend_valid_utf8` fuzz target now uses `MizarParserSeam` instead
      of the stub seam, so arbitrary valid UTF-8 flows through preprocessing,
      tokenization, the real parser seam, syntax diagnostic merging, and
      `SurfaceAstCacheKey` construction when the parser returns an AST.
      Parser diagnostics remain recoverable frontend output rather than hard
      frontend errors.
    - This lands the frontend-owned half coordinated with `mizar-parser` task
      39; the parser-owned fuzz target remains tracked there.
    - Depends on: 27, 28. Spec: [parsing.md](./parsing.md),
      [orchestration.md](./orchestration.md), [cache_key.md](./cache_key.md).

## Suggested Verification

After each task, run:

```text
cargo test -p mizar-frontend
cargo clippy -p mizar-frontend --all-targets -- -D warnings
```

Tasks that touch the lexer/session boundary should also run:

```text
cargo test -p mizar-session
cargo test -p mizar-lexer
```

Check off the task here once its tests pass.

## Notes

- `mizar-frontend` is an orchestration crate: it coordinates `mizar-session`,
  `mizar-lexer`, `mizar-syntax`, and `mizar-parser`, but owns none of their core
  algorithms or data definitions.
- Keep `mizar-lexer` decoupled from `mizar-session`; the lexer-span → session
  `SourceRange` bridge lives only in `span_bridge`.
- The frontend produces syntax, not semantics; no name resolution, type checking,
  overload selection, or proof obligations belong here.
- Frontend artifacts (`SourceUnit`, `PreprocessedSource`, `TokenStream`,
  `SurfaceAst`, `FrontendOutput`) are internal compiler data, not stable external
  schemas.
