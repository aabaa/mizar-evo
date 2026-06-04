# mizar-frontend TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

| Module | Spec | Source | Status |
|---|---|---|---|
| span_bridge | [span_bridge.md](./span_bridge.md) | `src/span_bridge.rs` | [ ] |
| source | [source.md](./source.md) | `src/source.rs` | [ ] |
| preprocess | [preprocess.md](./preprocess.md) | `src/preprocess.rs` | [ ] |
| lexical_env | [lexical_env.md](./lexical_env.md) | `src/lexical_env.rs` | [ ] |
| lexing | [lexing.md](./lexing.md) | `src/lexing.rs` | [ ] |
| parsing | [parsing.md](./parsing.md) | `src/parsing.rs` | [ ] |
| orchestration | [orchestration.md](./orchestration.md) | `src/orchestration.rs` | [ ] |

`mizar-frontend` is an orchestration crate, so it is built bottom-up by phase:
the coordinate bridge first, then pipeline Steps 1-5 in order, then the
end-to-end coordinator. `span_bridge` is the shared primitive every later step
references; `orchestration` is the only module that wires the full pipeline.

Dependency order: `span_bridge` → `source` → `preprocess` → `lexical_env` →
`lexing` → `parsing` → `orchestration`.

## Crate Prerequisites

The frontend foundation depends on `mizar-session` and `mizar-lexer`. It should
not add hard dependencies on `mizar-syntax` or `mizar-parser` until the real
parser-seam tasks land, because those crates are not yet implemented (top-level
[../../todo.md](../../todo.md) lists both as not started). Tasks 1-10 and the
stubbed coordinator portions of tasks 13-14 can be implemented against
`mizar-session` and `mizar-lexer` alone. The real parser invocation and
syntax-AST assertions in tasks 11-12, and any real-parser assertions in tasks
13-14, are gated on a minimal `mizar-parser` entry point and
`mizar-syntax::SurfaceAst`.

## Open Decisions To Resolve First

These gate the public API and are tracked at the top level in
[../../todo.md](../../todo.md) "Open Decisions":

- **Lexer span bridging.** This crate adopts the decoupled option: `mizar-lexer`
  keeps its byte-offset spans and `span_bridge` (task 1) maps them onto
  `mizar-session` `SourceRange`. Confirm this against the `mizar-session`
  `source_map` integration before freezing the bridge API.
- **Parser-assisted lexing contract.** The current lexer exposes a uniform
  `ParserLexContext`, not position-sensitive string-required spans. Position-
  sensitive string literal recognition, Unicode inside annotation string
  arguments, and parser-driven symbol-kind filters remain gated on a narrow
  `ParserLexContext` / `ParserInputs` contract that never exposes arbitrary
  parser state.

## Ordered Task List

Each task is sized to be implemented, tested, and committed on its own. The
listed dependency lines are authoritative: when `mizar-parser` / `mizar-syntax`
are unavailable, skip the gated real-parser tasks and continue with the stubbed
source → tokens coordinator tasks that do not depend on them. Every task should
keep `cargo test -p mizar-frontend` green (see
[Suggested Verification](#suggested-verification)).

### Crate scaffolding

1. **Crate skeleton and coordinate bridge.** [ ]
   - Add the `mizar-frontend` crate to the workspace with dependencies on
     `mizar-session` and `mizar-lexer`; opt in to the workspace
     `[workspace.lints]` table via `lints.workspace = true` (matching
     `mizar-session`).
   - Add `pub mod span_bridge;` and define `SpanBridge`, `LexerByteSpan`, and
     `SpanBridgeError`; let `SpanBridge` own a retained session source-map
     service, implement fallible `register_source` / `register_preprocess_map`,
     and provide `loaded_span`, `loaded_mapping`, and `lexical_span`
     conversions over the retained `mizar-session` maps.
   - Derive the session-side `LoadingMap` / `PreprocessMap` from the lexer
     `SourceLoadingMap` / `SourcePreprocessMap` (or reuse the session
     `LoadingMap` attached to the `SourceUnit`) so there is one canonical map per
     `SourceId`.
   - Tests: `loaded_span` over BOM-stripped text stays in loaded-text
     coordinates, while `loaded_mapping` reports original input offsets through
     `MappedSourceRange.original_input`; lexical span maps through preprocess +
     loading maps; a span crossing a removed comment yields primary plus
     secondary anchors; non-UTF-8-boundary and out-of-range spans are rejected;
     conflicting map registration is reported.
   - Spec: [span_bridge.md](./span_bridge.md) "Public API", "Algorithm / Logic".

### Module: source (`src/source.rs`)

2. **`SourceUnit` and the loader bridge.** [ ]
   - Add `pub mod source;`. Define `SourceUnit`, `SourceUnitRequest`, the
     `SourceUnitLoader` trait, and `FrontendSourceLoader<L: SourceLoader>`;
     implement `source_unit_from_loaded` projecting a `mizar_session::LoadedSource`
     into a `SourceUnit` without recomputing hash, line map, loading map,
     normalized path, edition, origin, or generated anchor.
   - Provide a helper that registers the loaded `LineMap` / `LoadingMap` with a
     mutable `SpanBridge` under the `SourceId`.
   - Tests: disk `LoadedSource` projects with identical id/hash/line-map/loading-map;
     BOM/CRLF-normalized source carries `Some(loading_map)`; identity load carries
     `None`; normalized path and edition are preserved; open-buffer origin and
     version are preserved; generated sources preserve `generated_anchor`; a
     session `SourceLoadError` is propagated unchanged.
   - Depends on: 1. Spec: [source.md](./source.md) "Public API",
     "Algorithm / Logic".

### Module: preprocess (`src/preprocess.rs`)

3. **Comment and doc-comment preprocessing.** [ ]
   - Add `pub mod preprocess;`. Define `PreprocessedSource`, `LexicalText`,
     `Comment`, `DocComment`, `LexicalSourceMap`, and `PreprocessDiagnostic`.
   - Drive `mizar_lexer::preprocess_source_for_lexing` over `SourceUnit.source_text`,
     map comment / doc-comment / preprocess-diagnostic spans through the
     `SpanBridge`, and assemble the `LexicalSourceMap`.
   - Tests: ordinary comments removed from lexical text but retained as `Comment`s
     with correct ranges; doc comment preserved with raw body and range; annotation
     syntax stays in lexical text; a lexical range crossing a removed comment yields
     a composite mapping; a code-region non-ASCII char and an unterminated block
     comment are reported and recovered.
   - Depends on: 2. Spec: [preprocess.md](./preprocess.md) "Comments and Doc
     Comments", "Algorithm / Logic".

4. **Shallow import pre-scan integration.** [ ]
   - Raw-scan lexical text (`scan_raw`) and run `mizar_lexer::scan_import_prelude`;
     populate `import_stubs` with mapped `SourceRange`s and collect
     `ImportPrescanDiagnostic`s into `diagnostics`.
   - Tests: top-level `import` forms produce `ImportStub`s with raw path, optional
     alias, source segments, and span; a malformed import yields an import-prescan
     diagnostic without aborting; import order is preserved for provenance and
     deterministic fingerprints.
   - Depends on: 3. Spec: [preprocess.md](./preprocess.md) "Import Stubs",
     "Error Handling".

### Module: lexical_env (`src/lexical_env.rs`)

5. **Lexical environment request and provider seam.** [ ]
   - Add `pub mod lexical_env;`. Define `LexicalEnvironmentRequest`,
     `LexicalSummaryProvider`, `ResolvedImports`, `ActiveLexicalEnvironmentResult`,
     and `LexicalEnvironmentDiagnostic`; re-export the `mizar-lexer` environment
     types.
   - Tests: a fake provider that returns resolved imports + summaries produces a
     `UserSymbolIndex` with correct provenance and import ordinal; reserved tables
     are always present.
   - Depends on: 4. Spec: [lexical_env.md](./lexical_env.md) "Public API".

6. **Active lexical environment construction.** [ ]
   - Implement `build_active_lexical_environment` calling
     `mizar_lexer::build_lexical_environment`; merge provider diagnostics; compute
     and surface the `LexicalEnvironmentFingerprint`.
   - Tests: equal-spelling user symbols imported from different modules produce a
     deterministic lexical-environment conflict or provider diagnostic according
     to the lexer/spec contract; an unresolved import degrades to a smaller
     environment with a diagnostic while remaining symbols load; the fingerprint
     changes when a dependency summary changes and is stable for comment-only
     local edits.
   - Depends on: 5. Spec: [lexical_env.md](./lexical_env.md) "Algorithm / Logic",
     "Error Handling".

### Module: lexing (`src/lexing.rs`)

7. **Raw scan and scope skeleton wiring.** [ ]
   - Add `pub mod lexing;`. Define `TokenizeRequest`, the frontend `Token` /
     `TokenStream` (session-spanned), and re-export `TokenKind` / `LexDiagnostic`.
   - Build the `ScopeSkeleton` / `ScopeLexView` from raw tokens and prepare the
     disambiguator inputs.
   - Tests: raw scan preserves `LexemeRun` spans; the scope view reflects lexical
     block/statement shape without resolved bindings.
   - Depends on: 6. Spec: [lexing.md](./lexing.md) "Scope Lex View",
     "Algorithm / Logic".

8. **Context-sensitive disambiguation to `TokenStream`.** [ ]
   - Run `disambiguate` (or parser-integrated `lex`) with the active lexical
     environment, scope view, and the current `ParserLexContext` (general/stub
     context until the parser-assisted contract is finalized); map every lexer
     span through the `SpanBridge` to a session `SourceRange`.
   - Tests: a user symbol sharing spelling with an identifier is classified by
     longest-match; compound reserved tokens (`.{`, `.*`, `.=`, `...`) lex as
     single tokens; a quote lexes as a symbol char under the general context; a
     bounded uniform `StringRequired` context produces a `StringLiteral`; every
     emitted token span resolves to a valid primary `SourceRange` while secondary
     anchors are preserved for diagnostics. Position-specific annotation/operator
     string-literal tests are deferred to the parser-assisted lexing contract.
   - Depends on: 7. Spec: [lexing.md](./lexing.md) "Token Stream",
     "Algorithm / Logic".

9. **Lexer recovery passthrough.** [ ]
   - Preserve `TokenKind::ErrorRecovery` spans and `LexDiagnostic`s end to end;
     adapt raw-scan hard errors into recovery tokens/diagnostics so the frontend
     `tokenize` wrapper always returns a `TokenStream`.
   - Tests: a malformed token emits `ErrorRecovery` with the correct `SourceRange`
     and scanning resumes; an invalid numeral and a malformed string literal in a
     string-required position are reported without dropping recoverable tokens.
   - Depends on: 8. Spec: [lexing.md](./lexing.md) "Error Handling".

### Module: parsing (`src/parsing.rs`)

10. **Parser-input assembly and parser seam.** [ ]
    - Add `pub mod parsing;`. Define `ParseRequest`, `ParserInputs`, `ParseOutput`,
      `ParserSeam`, and `StubParserSeam`; derive `ParserInputs` after the active
      lexical environment is built, using the source edition plus only the data
      currently exposed by lexical summaries.
    - Until `mizar-parser` exists, implement the seam against a stub that returns
      `ast = None` plus an empty diagnostic list, so the source → tokens pipeline
      is exercisable.
    - Tests: `ParserInputs` carries the edition, uses an empty operator-fixity
      table when summaries do not expose fixity, carries no resolver state, and
      the stub seam returns `ast = None`.
    - Depends on: 8. Spec: [parsing.md](./parsing.md) "Parser Inputs",
      "Public API".

11. **`mizar-parser` invocation.** [ ]
    - Replace the stub seam with the real `mizar-parser` entry point; return the
      `mizar-syntax::SurfaceAst` and syntax diagnostics unchanged.
    - Requires a minimal `mizar-parser` / `mizar-syntax` (top-level
      [../../todo.md](../../todo.md)). Gate behind their availability.
    - Tests: a well-formed token stream parses to a `SurfaceAst` with preserved
      source order and ranges; operator fixity drives correct Pratt precedence for
      a user infix operator once summaries expose fixity; a `StringLiteral` at an
      annotation argument parses to an annotation node, using either a synthetic
      parser test token stream or the finalized parser-assisted lexing contract.
    - Depends on: 10, plus `mizar-parser`/`mizar-syntax`. Spec:
      [parsing.md](./parsing.md) "Algorithm / Logic".

12. **Parser recovery passthrough.** [ ]
    - Preserve `ast = None` on unrecoverable input and explicit recovery-node
      markers inside a returned `SurfaceAst`; carry syntax diagnostics through.
    - Tests: a missing `end` recovers at a synchronization point with `ast = Some`
      and an explicit error node; an unrecoverable stream returns `ast = None` with
      diagnostics; a missing string literal at a string-required position yields the
      expected syntax diagnostic.
    - Depends on: 11. Spec: [parsing.md](./parsing.md) "Error Handling".

### Module: orchestration (`src/orchestration.rs`)

13. **Frontend coordinator and diagnostic merge.** [ ]
    - Add `pub mod orchestration;`. Define `FrontendOutput`, `Frontend`,
      `FrontendDiagnostic`, `DiagnosticClass`, and `FrontendError`; wire
      `source` → `preprocess` → `lexical_env` → `lexing` → `parsing` and merge all
      phase diagnostics into the deterministic order in
      [orchestration.md](./orchestration.md) "Diagnostic Merge Order".
    - Tests: with `StubParserSeam`, a well-formed source returns source,
      preprocessing output, tokens, `ast = None`, and no parser diagnostics; merge
      order is identical across repeated runs. With the real parser seam, add the
      `ast = Some` and syntax-diagnostic ordering assertions.
    - Depends on: 9, 10. Real-parser assertions depend on 12. Spec:
      [orchestration.md](./orchestration.md)
      "Algorithm / Logic", "Diagnostic Merge Order".

14. **Unrecoverable-failure handling and end-to-end output.** [ ]
    - Return `FrontendError` only for Step 1 load failures and `SpanBridgeError`
      invariant violations; keep recoverable problems as diagnostics inside
      `FrontendOutput`.
    - Tests: a Step 1 load failure returns `FrontendError` with the file-level
      diagnostic and no output; a parser seam that returns `ast = None` preserves
      earlier diagnostics; merged diagnostics carry valid `SourceRange`s.
    - Depends on: 13. Spec: [orchestration.md](./orchestration.md) "Error
      Handling".

### Module-wide maintenance before cross-cutting follow-ups

15. **Implementation refactoring pass.** [ ]
    - Review `span_bridge`, `source`, `preprocess`, `lexical_env`, `lexing`,
      `parsing`, and `orchestration` once the first implementation pass is complete.
    - Keep public APIs and behavior stable unless the refactor exposes a clear bug
      or spec mismatch; prefer small local extractions and shared test fixtures.
    - Tests: keep all module tests green.
    - Depends on: 14. Spec: all mizar-frontend module specs.

16. **Source/spec correspondence audit.** [ ]
    - Build a lightweight traceability check from each public API, error variant,
      and task requirement in the frontend specs to the implementing source/tests.
    - Record any missing implementation, stale spec text, or missing tests as
      follow-up tasks rather than mixing broad changes into the audit.
    - Check the English canonical specs first, then verify Japanese companions
      carry the same API and behavioral commitments.
    - Depends on: 15. Spec: all mizar-frontend module specs and this TODO.

## Cross-Cutting Follow-up Tasks

17. **Bilingual documentation synchronization audit.** [ ]
    - Compare every English canonical document under
      `doc/design/mizar-frontend/en/` with its Japanese companion under
      `doc/design/mizar-frontend/ja/`; synchronize API lists, task statuses,
      terminology, and links.
    - Depends on: 16. Spec: repository documentation policy.

18. **Frontend determinism property tests.** [ ]
    - Add crate-level coverage that identical inputs produce identical
      `FrontendOutput` diagnostics order and identical token spans independent of
      internal scheduling, and that the `LexicalEnvironmentFingerprint` and cache
      keys are stable for equivalent inputs.
    - Depends on: 16. Spec: [orchestration.md](./orchestration.md),
      [lexical_env.md](./lexical_env.md).

19. **Incremental cache-key wiring.** [ ]
    - Decide where the layered frontend cache keys from
      [architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
      "Incrementality" are computed and stored (this crate vs. the driver/artifact
      layer), and expose the per-artifact keys (`SourceUnit`, `PreprocessedSource`,
      `ActiveLexicalEnvironment`, `TokenStream`, `SurfaceAst`) accordingly.
    - Verify comment-only edits can reuse semantic outputs while import / dependency
      export edits invalidate tokenization and downstream layers.
    - Depends on: 16. Spec: architecture incrementality table.

20. **Parser-assisted lexing contract finalization.** [ ]
    - Finalize whether disambiguation runs in one pass with a precomputed
      position-sensitive `ParserLexContext` or interleaves with parsing through
      the narrow context object, and document the chosen integration in
      [lexing.md](./lexing.md) and [parsing.md](./parsing.md).
    - Keep the lexer free of arbitrary parser state under either choice. This
      task blocks position-specific `StringLiteral` tests and Unicode acceptance
      inside annotation string arguments.
    - Depends on: 10; real-parser validation also depends on 11. Spec: top-level
      [../../todo.md](../../todo.md) "Open Decisions", [lexing.md](./lexing.md),
      [parsing.md](./parsing.md).

21. **Durable lint enforcement.** [ ]
    - Confirm `crates/mizar-frontend/Cargo.toml` opts into the workspace
      `[workspace.lints]` table so `cargo build`/`cargo test` surface the same
      denials as the standalone clippy gate (matching the `mizar-session` policy).
    - Record any intentional `allow` exceptions with a rationale next to the `allow`.
    - Tests: `cargo clippy -p mizar-frontend --all-targets -- -D warnings` passes.
    - Depends on: 16. Spec: this TODO "Suggested Verification".

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
  `mizar-lexer`, and, once the real parser seam is enabled, `mizar-syntax` /
  `mizar-parser`, but owns none of their core algorithms or data definitions.
- Keep `mizar-lexer` decoupled from `mizar-session`; the lexer-span → session
  `SourceRange` bridge lives only in `span_bridge`.
- The frontend produces syntax, not semantics; no name resolution, type checking,
  overload selection, or proof obligations belong here.
- Frontend artifacts (`SourceUnit`, `PreprocessedSource`, `TokenStream`,
  `SurfaceAst`, `FrontendOutput`) are internal compiler data, not stable external
  schemas.
