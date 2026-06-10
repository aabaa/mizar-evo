# Source/spec correspondence audit

> Canonical language: English. Japanese companion: [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

Status: completed through task 22.

## Scope

This audit checks the task-22 `mizar-frontend` implementation against the
English canonical module specs under `doc/design/mizar-frontend/en/`, then checks
that the Japanese companion specs carry the same public API names, error and
diagnostic variants, and behavior promises.

This is a lightweight source/spec/test correspondence map. It is not a release
coverage gate and does not replace executable tests. If this audit finds missing
implementation, stale spec text, or missing tests, the item is recorded as a
follow-up task instead of being mixed into the audit.

## Result

- No missing implementation was found for the public APIs and error/diagnostic
  variants promised by tasks 1-20, the task-21 lint policy guard, or the task-22
  precise raw-scan recovery contract.
- The task-2 source requirement text now names the open-buffer `file://`
  diagnostic-path decode/fallback tests that were added before this audit.
- No remaining stale English canonical spec text was found for tasks 1-22.
- Japanese companion specs were checked for matching API names and behavior
  promises. No remaining API or behavior drift was found.
- Broader bilingual wording/terminology review was completed by task 17 in
  [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md).
  Task 19 incremental cache-key wiring, task 20 parser-assisted lexing,
  task 21 durable lint enforcement, and task 22 precise raw-scan recovery are
  now complete; other deferred implementation or coverage work remains tasks
  23-24.

## Public API Correspondence

| Spec | Public API checked | Source | Test evidence |
|---|---|---|---|
| [span_bridge.md](./span_bridge.md) | `SpanBridge`, `LexerByteSpan`, `SpanBridgeError`; `SpanBridge::{new, source_map_service, register_source, register_preprocess_map, loaded_span, loaded_mapping, lexical_span}`; crate-visible `whole_lexical_text_mapping` | `crates/mizar-frontend/src/span_bridge.rs` | Inline tests cover loaded and lexical mappings, loading-map identity behavior, composite/degraded preprocess mappings, whole-lexical-text recovery mapping, invalid spans, missing registrations, and conflicting registrations. Unsupported lexer preprocess metadata is represented by a defensive guard; no unsupported map variant is currently constructible from `mizar-lexer`. |
| [source.md](./source.md) | `SourceUnit`, `SourceUnitRequest`, `SourceUnitLoader`, `SourceUnitLoader::load_source_unit`, `FrontendSourceLoader`, `FrontendSourceLoader::new`, `source_unit_from_loaded`, `register_source_unit` | `crates/mizar-frontend/src/source.rs` | Inline tests cover projection without recomputation, loader forwarding, BOM/CRLF loading maps, identity loads, open-buffer origin/version, open-buffer `file://` path decode/fallback, generated sources, bridge registration, and unchanged `SourceLoadError` propagation. |
| [preprocess.md](./preprocess.md) | `PreprocessedSource`, `LexicalText`, `LexicalText::as_str`, `Comment`, `DocComment`, `LexicalSourceMap`, `LexicalSourceMap::{lexical_span, lexical_len, is_empty}`, `ImportStub`, `ImportStubPath`, `ImportStubRelativePrefix::{Current, Parent}`, `ImportStubAlias`, `PreprocessDiagnostic`, `PreprocessDiagnosticKind`, `preprocess`, `lexical_hash`; re-exported `ImportPrescanDiagnosticCode`, `SourcePreprocessDiagnosticCode`, `CommentKind` | `crates/mizar-frontend/src/preprocess.rs` | Inline tests cover comment/doc-comment separation, annotation preservation, import stubs and relative prefixes, malformed import recovery, precise recoverable raw import scan diagnostics with preserved partial imports and error-sentinel path boundaries, mapped diagnostics, composite/degraded lexical mappings, stable lexical hashes, non-ASCII code diagnostics, and unterminated block comments. |
| [lexical_env.md](./lexical_env.md) | `LexicalEnvironmentRequest`, `LexicalSummaryProvider`, `LexicalSummaryProvider::resolve_imports`, `ResolvedImports`, `ResolvedImportEntry`, `ActiveLexicalEnvironmentResult`, `FrontendLexicalEnvironmentError`, `LexicalEnvironmentDiagnostic`, `LexicalEnvironmentDiagnosticCode`, `build_active_lexical_environment`; re-exported lexer environment types | `crates/mizar-frontend/src/lexical_env.rs` | Inline tests cover provider seam output, import deduplication, reserved tables, provider infrastructure failure, provider provenance hard failures, unresolved imports, missing summaries, conflict retry, non-conflict malformed summaries, and fingerprint stability/change behavior. |
| [lexing.md](./lexing.md) | `InternedText`, `TokenizeRequest`, `TokenizeRequest::{new, with_plan}`, `ParserLexingPlan`, `ParserLexingPlan::{uniform, new, for_lexical_text, context_at, is_uniform}`, `ParserLexingPlanContext`, `ParserLexingPlanContext::new`, `LexicalByteRange`, `LexicalByteRange::{new, contains}`, `TokenStream`, `TokenStream::{tokens, diagnostics, scope_view, into_parts}`, `Token`, `ScopeView`, `ScopeView::{empty, binding_overrides_symbol}`, `ScopeFrame`, `ScopedBinding`, `ScopeBlock`, `ScopeStatement`, `LexingDiagnostic`, `LexingDiagnosticKind`, `LexingDiagnosticPayload`, `LexingRejectedTokenCandidate`, `tokenize`; re-exported lexer token/context/scope enums | `crates/mizar-frontend/src/lexing.rs` | Inline tests cover raw-span preservation, preprocess mapping, longest-match user symbols, scoped identifier overrides, compound reserved tokens, parser context/string behavior, position-sensitive annotation string arguments with Unicode/comment-marker contents, line-boundary rejection for planned string ranges, range-specific user-symbol kind filters, payload mapping, recoverable error tokens, unsupported raw-token recovery, rejected candidates, secondary anchors, scope view contents, scope diagnostics, and precise recoverable raw-scan diagnostics with partial token continuation. |
| [parsing.md](./parsing.md) | `DEFAULT_PARSER_CACHE_KEY_VERSION`, `STUB_PARSER_CACHE_KEY_VERSION`, `MIZAR_PARSER_CACHE_KEY_VERSION`, `ParseRequest`, `ParseRequest::new`, `ParserInputs`, `ParserInputs::{new, from_active_environment}`, `OperatorFixityTable`, `OperatorFixityTable::{empty, is_empty}`, `OperatorFixityEntry`, `OperatorAssociativity::{Left, Right, NonAssociative}`, `StringRequiredContext::{None, PositionSensitive, UniformForTest}`, `StringRequiredContext::{parser_lex_context, parser_lexing_plan}`, `ParserCacheKeyVersion`, `ParserCacheKeyVersion::new`, `ParseOutput`, `ParseOutput::new`, `ParserSeam`, `ParserSeam::{cache_key_version, parse}`, `StubParserSeam`, `MizarParserSeam` | `crates/mizar-frontend/src/parsing.rs` | Inline tests cover parser inputs, absence of resolver state, string-required context mapping, position-sensitive plan construction, stub seam output, real parser AST handoff, token-kind adaptation, error-recovery tokens, missing-`end` recovery, unrecoverable `ast = None`, string-required forwarding, Pratt fixity and associativity, and syntax diagnostic passthrough. Cache-key version use is covered through `cache_key` and frontend determinism tests. |
| [cache_key.md](./cache_key.md) | `SOURCE_UNIT_CACHE_KEY_VERSION`, `PREPROCESSED_SOURCE_CACHE_KEY_VERSION`, `ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION`, `PARSER_LEXING_PLAN_CACHE_KEY_VERSION`, `TOKEN_STREAM_CACHE_KEY_VERSION`, `SURFACE_AST_CACHE_KEY_VERSION`, `FrontendCacheKeys`, `SourceUnitCacheKey`, `SourceUnitCacheKey::{from_source, stable_hash}`, `PreprocessedSourceCacheKey`, `PreprocessedSourceCacheKey::{from_source, stable_hash}`, `ActiveLexicalEnvironmentCacheKey`, `ActiveLexicalEnvironmentCacheKey::{new, stable_hash}`, `ParserLexingPlanCacheKey`, `ParserLexingPlanContextCacheKey`, `ParserLexingPlanCacheKey::{current, from_plan}`, `TokenStreamCacheKey`, `TokenStreamCacheKey::{new, stable_hash}`, `SurfaceAstCacheKey`, `SurfaceAstCacheKey::{new, stable_hash}`, `parser_inputs_hash` | `crates/mizar-frontend/src/cache_key.rs` | Inline tests cover source-key freshness exclusions and content identity changes, comment-only preprocessing invalidation with token/AST reuse, import/environment/parser-context/parser-plan token invalidation including same-version position-sensitive plan content, and token-stream/parser-version/parser-input/edition AST invalidation. Crate-level determinism tests assert `FrontendOutput.cache_keys` for comment-equivalent runs and end-to-end import/dependency invalidation. |
| [orchestration.md](./orchestration.md) | `FrontendOutput` including `cache_keys`, `Frontend`, `Frontend::{new, run}`, `FrontendDiagnostic`, `DiagnosticLocation`, `SourceLoadLocation`, `DiagnosticCode`, `DiagnosticClass`, `FrontendError`, `FrontendParserDiagnostic`, `FrontendParserDiagnostic::into_frontend_diagnostic` | `crates/mizar-frontend/src/orchestration.rs` | Inline tests cover stub and real parser coordinator output, syntax diagnostic merge order, repeated-run determinism for current coordinator paths, same-class sorting, source-load diagnostics without fabricated ranges, open-buffer/generated load locations, span-bridge hard failures, lexical-environment hard failures, `ast = None` parser seams, and valid range-backed merged diagnostics. Crate-level determinism tests cover `FrontendOutput.cache_keys`. |

The Japanese companion files under `doc/design/mizar-frontend/ja/` carry the
same API names, variants, and behavior boundaries for every row above. Broader
language synchronization was completed by task 17 in
[bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md).

## Error And Diagnostic Variant Correspondence

| Surface | Variants | Source/test status |
|---|---|---|
| `SpanBridgeError` | `SourceNotRegistered`, `PreprocessMapNotRegistered`, `ConflictingSourceRegistration`, `ConflictingPreprocessMapRegistration`, `UnsupportedLexerPreprocessMap`, `SourceMap` | Implemented in `span_bridge.rs`. Missing/conflict variants and `SourceMap` wrapping have tests. `UnsupportedLexerPreprocessMap` is the defensive conversion guard for future unsupported lexer preprocess metadata; it has no current producer in `mizar-lexer`. |
| `PreprocessDiagnosticKind` | `SourcePrecondition`, `ImportPrescan`, `RawImportScan` | Implemented in `preprocess.rs` and covered by source precondition, import pre-scan, and raw-scan recovery tests. |
| `FrontendLexicalEnvironmentError` | `ProviderUnavailable`, `MalformedProviderProvenance`, `MalformedSummary` | Implemented in `lexical_env.rs` and covered by provider infrastructure, provenance hard failure, and malformed-summary tests. |
| `LexicalEnvironmentDiagnosticCode` | `UnresolvedImport`, `MissingSummary`, `UserSymbolImportConflict`, `InvalidUserSymbolSpelling`, `InvalidUserSymbolArity`, `ReservedWordCollision`, `ReservedSymbolCollision` | Implemented in `lexical_env.rs`. The first three are emitted by current frontend recovery paths and covered directly. The last four are currently unproduced by frontend recovery; lexer-owned invalid spelling/arity and reserved collisions remain `MalformedSummary` hard failures by spec. Variant-level provider pass-through policy and coverage are recorded as task 24. |
| `LexingDiagnosticKind` | `RawScan`, `ScopeSkeleton`, `Lexer` | Implemented in `lexing.rs` and covered by raw-scan recovery, scope-skeleton diagnostic, and lexer diagnostic tests. |
| `LexingDiagnosticPayload` | `None`, `NoValidTokenCandidate`, `ParserContextRejectedCandidate`, `MalformedStringLiteral`, `UnsupportedRawToken`, `UnsupportedLexerPayload` | Implemented in `lexing.rs`. Current lexer payloads are covered; `UnsupportedLexerPayload` is the explicit fallback for future payload variants. |
| `mizar_syntax::SyntaxDiagnosticCode` through `DiagnosticCode::Syntax` | `UnexpectedErrorToken`, `DanglingOperator`, `NonAssociativeOperatorChain`, `MissingEnd`, `MissingStringLiteral`, `UnrecoverableInput` | Owned by `mizar-syntax` / `mizar-parser` and passed through by `MizarParserSeam` and `FrontendParserDiagnostic`. Frontend/parser tests cover each current parser code and verify syntax diagnostic passthrough. |
| `DiagnosticLocation` / `SourceLoadLocation` | `SourceRange`, `SourceLoad`; `Path`, `NormalizedPath`, `OpenBuffer`, `Generated`, `Unknown` | Implemented in `orchestration.rs`. Current disk, open-buffer, generated, and range-backed locations are covered. `NormalizedPath` fallback is reserved for future non-exhaustive source origins, and `Unknown` has no current producer because `SourceInput` carries a `normalized_path`; coverage/policy for these reserved fallback locations is recorded as task 24. |
| `DiagnosticCode` / `DiagnosticClass` | `SourceLoad`, `Preprocess`, `LexicalEnvironment`, `Lexing`, `Syntax`; `SourceLoad`, `LexicalPrecondition`, `CommentStructure`, `ImportPrescan`, `LexicalEnvironment`, `ScopeSkeleton`, `Tokenization`, `Syntax`, `AnnotationSyntax` | Implemented in `orchestration.rs` and covered through merge-order and class sorting tests for currently emitted frontend diagnostics. `AnnotationSyntax` is a reserved class with no current producer; its future producer/test coverage remains part of task 24 reserved frontend diagnostic surface coverage. |
| `FrontendError` | `SourceLoad`, `SpanBridge`, `LexicalEnvironment` | Implemented in `orchestration.rs` and covered by hard-failure path tests. |

## Task Requirement Correspondence

| Task | Status | Source/test correspondence |
|---|---|---|
| 1 | Complete | `span_bridge` public API and source-map behavior are implemented in `src/span_bridge.rs` with mapping and conflict tests. |
| 2 | Complete | `source` loader bridge is implemented in `src/source.rs` with projection, loading-map, open-buffer URI path, generated-source, registration, and load-error tests. |
| 3 | Complete | Comment/doc-comment preprocessing is implemented in `src/preprocess.rs` with comment, doc body, annotation, mapping, hash, non-ASCII, and unterminated-comment tests. |
| 4 | Complete | Shallow import pre-scan is implemented in `src/preprocess.rs` with import stub, relative prefix, malformed import, raw-scan failure, source-order, and mapping tests. |
| 5 | Complete | Provider seam and provenance API are implemented in `src/lexical_env.rs` with provider, deduplication, diagnostic, and reserved-table tests. |
| 6 | Complete | Active lexical environment recovery is implemented in `src/lexical_env.rs` with unresolved import, missing summary, conflict retry, malformed summary, and fingerprint tests. |
| 7 | Complete | Raw scan and scope skeleton wiring are implemented in `src/lexing.rs` with raw span and scope-view tests. |
| 8 | Complete | Context-sensitive disambiguation is implemented in `src/lexing.rs` with user symbol, compound token, string context, token span, and payload mapping tests. |
| 9 | Complete | Lexer recovery passthrough is implemented in `src/lexing.rs` with error-recovery, unsupported raw token, rejected candidate, and scope diagnostic preservation tests. |
| 10 | Complete | Parser input assembly and stub seam are implemented in `src/parsing.rs` with edition/fixity/string context/no-resolver-state/stub tests. |
| 11 | Complete | Real parser seam is implemented in `src/parsing.rs` with AST handoff, token adaptation, syntax diagnostics, and Pratt fixity tests. |
| 12 | Complete | Parser recovery passthrough is implemented in `src/parsing.rs` with missing-`end`, unrecoverable `ast = None`, string-required context, and diagnostic passthrough tests. |
| 13 | Complete | Coordinator and deterministic diagnostic merge are implemented in `src/orchestration.rs` with stub/real parser output and merge-order tests. |
| 14 | Complete | Unrecoverable failure handling is implemented in `src/orchestration.rs` with source-load, span-bridge, lexical-environment, `ast = None`, and valid range-backed diagnostic tests. |
| 15 | Complete | The refactoring pass is reflected in shared whole-lexical-text mapping, source URI boundary tests, lexical-env provenance hard failures, and synchronized module specs. |
| 16 | Complete | This audit and its Japanese companion record the source/spec/test correspondence and follow-up status, including new task 24 for reserved diagnostic/fallback surfaces. |
| 17 | Complete | The bilingual documentation synchronization audit records synchronized public API/status/terminology/link/behavior commitments. |
| 18 | Complete | Crate-level determinism property tests cover provider scheduling permutations and comment-equivalent cache-key stability. |
| 19 | Complete | Incremental cache-key wiring is implemented in `src/cache_key.rs`, exposed through `FrontendOutput.cache_keys`, and documented in `cache_key.md`; tests cover source/preprocess/environment/token/AST invalidation boundaries. |
| 20 | Complete | Parser-assisted lexing uses precomputed `ParserLexingPlan`s in `src/lexing.rs` / `src/parsing.rs` / `src/orchestration.rs`; tests cover annotation string arguments with Unicode/comment-marker contents, single-line range guards, range-specific user-symbol kind filters, cache-key plan invalidation, and real source-to-token-to-parser handoff. |
| 21 | Complete | Durable lint enforcement is guarded by `crates/mizar-frontend/tests/lint_policy.rs`, which checks the frontend manifest opt-in to workspace lints, the shared rustc/clippy denial baseline, and adjacent rationale for any future frontend `allow` attributes. |
| 22 | Complete | Precise raw-scan recovery is implemented with `mizar_lexer::scan_raw_recoverable`; frontend import pre-scan and tokenization map `RawScanDiagnostic` spans precisely, preserve usable partial imports/tokens, and keep whole-text fallback only for internal parser-plan defects. |

## Follow-up Records

This audit added task 24 for reserved or currently unproduced diagnostic/fallback
surface coverage. Tasks 18, 19, 20, 21, and 22 have since been completed; the
remaining cross-cutting items are:

- Task 23: resident-set contract guard for the lexical environment.
- Task 24: reserved frontend diagnostic surface coverage.
