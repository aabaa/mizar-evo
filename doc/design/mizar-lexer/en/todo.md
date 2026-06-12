# mizar-lexer TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

This document records follow-up tasks identified during lexer reviews and
language-spec synchronization.

## Ordered Task List

No open ordered tasks.

## Completed Tasks

1. Hardened `SourceLineIndex` offset validation.
   - `location` and `range` now reject byte offsets that are not UTF-8 character boundaries by returning `None`.
   - Unit tests cover multi-byte UTF-8 text while preserving the documented zero-based line and zero-based byte-column convention.

2. Added property-style final token span tests.
   - Crate-local Phase 7 unit tests now cover several `lex` and `disambiguate` seeds.
   - Every final token span is asserted to point back to the token spelling: `source[token.span.start..token.span.end] == token.lexeme`.
   - Seeds include one-to-one raw mappings, `LexemeRun` splits, string-required context, malformed string recovery, parser-context rejection, and unsupported raw-token recovery spans.

3. Pinned unsupported Unicode code-region diagnostics.
   - Crate-local raw lexer and preprocessor tests now cover NBSP, zero-width space, zero-width joiner/non-joiner, full-width punctuation, and `U+FEFF` inside code regions.
   - These cases assert stable `NonAsciiCode` preprocessor diagnostics with byte spans and stable raw-scan hard errors rather than treating the characters as layout or valid token text.

4. Pinned unsupported ASCII control characters.
   - Crate-local tests now cover vertical tab and form feed in code regions.
   - These cases assert that the characters are not layout and produce stable raw-scan hard errors rather than being treated as valid token text.

5. Expanded comment-removal edge tests.
   - Crate-local preprocessing tests now cover adjacent comments, comments at EOF, comments immediately between token-shaped text, and multi-line comments with multiple preserved newlines.
   - Inline comment removal now inserts synthetic layout when needed so lexical text preserves line structure and does not accidentally concatenate tokens.

6. Decided nested multi-line comment policy.
   - Multi-line comments are documented as non-nesting: the first `=::` after a `::=` opener closes the comment.
   - Crate-local preprocessing tests now cover inner `::=` spellings as ordinary comment text, including trivia spans and preserved newlines.

7. Added minimal documentation tests and examples.
   - Crate-level doctests now cover `scan_raw`, `lex`, and `disambiguate`.
   - The examples show that token spans are byte offsets into the scanner input.
   - The examples stay on small, stable parser-facing API surfaces.

8. Decided public API stability policy.
   - Public enums are now marked `#[non_exhaustive]` so downstream crates keep wildcard match arms for categories that may grow.
   - Public data struct fields remain visible because they are parser-facing transfer objects used by corpus and early integration code.
   - Crate-level docs and the raw lexer design notes now state that `0.1` APIs remain provisional until a later stability milestone.

9. Documented source-text normalization policy.
   - Crate-level docs and design notes now state that the lexer does not perform Unicode normalization.
   - Code-region identifiers, numerals, reserved spellings, and user-symbol spellings remain ASCII-only at the lexer boundary.
   - Comment and documentation text remains raw Unicode trivia unless a later documentation/source-loading layer adds warnings.

10. Added fuzz coverage.
   - Added a `cargo-fuzz` target, `lexer_valid_utf8`, for arbitrary valid UTF-8 strings.
   - The target exercises `preprocess_source_for_lexing`, direct `scan_raw`, and `scan_raw` over preprocessed lexical text.
   - Fuzz assertions pin span validity, comment lexeme slicing, raw token lexeme slicing, and full raw-token input coverage for successful scans.
   - Seed corpus entries cover ASCII/layout/annotation text and Unicode in documentation/comment/code regions.

11. Added performance benchmarking.
   - Added a Criterion benchmark for a large `.miz`-like source.
   - The benchmark measures `preprocess_source_for_lexing`, `scan_raw`, and `SourceLineIndex` construction separately.
   - The benchmark stays lexer-local and does not involve module resolution, parser context, or imported symbol loading.

12. Decided UTF-8 BOM handling policy at the source-loading boundary.
   - Package-authored source loading accepts one leading UTF-8 BOM and strips it before constructing `LoadedSource.text` or calling `mizar-lexer`.
   - Direct lexer helper calls stay strict: `U+FEFF` reaching `preprocess_source_for_lexing` or `scan_raw` remains malformed lexer-boundary input.
   - Lexer spans after BOM stripping are byte offsets into post-strip loaded text; `LoadingMap` relates loaded offset `0` to original file byte offset `3` for BOM-prefixed disk files.

13. Specified and tested UTF-8 file loading boundary behavior.
   - Added `load_source_text_from_bytes` as a crate-local executable boundary for UTF-8 validation and leading BOM stripping.
   - Invalid UTF-8 now returns `SourceLoadError::InvalidUtf8` before lexer entry and is not decoded lossily into `U+FFFD`.
   - Leading UTF-8 BOM stripping is tested with `LoadedSourceText.loading_map`; loaded offset `0` maps back to original byte offset `3`.
   - Non-leading `U+FEFF` remains in loaded text and is still rejected by lexer-boundary preprocessing/raw scanning when it appears in code.

14. Specified and tested newline normalization.
   - `load_source_text_from_bytes` now normalizes CRLF pairs to LF before lexer entry.
   - `SourceLoadingMapSegment::NormalizedNewline` records each normalized LF against the original two-byte CRLF range.
   - Tests cover plain CRLF input, CRLF after leading BOM stripping, and preserving lone `\r` as malformed lexer-boundary input.

15. Implemented preprocess source-map tests.
   - `PreprocessedLexicalSource` now carries a lightweight `SourcePreprocessMap` with original, removed-comment, and synthetic-whitespace segments.
   - Tests cover ordinary comment removal, documentation comment retention, synthetic spaces/newlines, and lexical ranges spanning removed comments.
   - Tests also pin mapping for lexer/preprocessor diagnostic byte ranges back to original source ranges.

16. Returned composite anchors for zero-length preprocess-map boundaries.
   - Lexical insertion points on boundaries between original text, removed comments, and synthetic whitespace now return all adjacent source anchors.
   - Tests cover removed-comment boundaries with and without inserted synthetic layout.

17. Kept user-facing column conversion outside lexer.
   - Added a minimal `mizar-session` crate with `LineMap` tests for one-based Unicode scalar line/column conversion.
   - Added a minimal `mizar-lsp` crate with range-mapper tests for zero-based LSP UTF-16 positions.
   - `mizar-lexer` remains byte-span oriented and does not perform user-facing or LSP column conversion.

18. Covered source path normalization outside lexer.
   - Added `mizar-session::normalize_source_path` and `NormalizedPath` for package-relative `.miz` source identities.
   - Tests cover `.`/`..`, symlink alias/escape rejection, canonical case spelling, package-root escape attempts, source-root enforcement, extension validation, ASCII identifier-shaped namespace components, and platform-specific separator normalization.
   - The lexer-local `module_source_name_from_path` remains only a boundary naming helper; filesystem-aware source identity now lives in the session source layer.

19. Decided shared source-span ownership across lexer, session, and LSP crates.
   - Kept lexer `SourceSpan` and session `SourceRange` crate-local instead of adding a common source-coordinate crate at this stage.
   - Added explicit LSP bridge conversion APIs: `source_range_from_lexer_span` and `lsp_range_from_lexer_span`.
   - Added LSP bridge tests for lexer-token spans, UTF-16 column conversion, pure field-copy conversion, and error propagation for invalid lexer spans, keeping coordinate-space conversion visible at the boundary.

20. Decided line/column overflow policy for session source maps.
   - Kept `mizar-session::LineColumn` values as `u32` because they are presentation and protocol-adjacent coordinates, not raw memory indexes.
   - Added `SourceMapError::LineColumnOverflow` so `LineMap` reports unrepresentable line or Unicode scalar column values instead of saturating, wrapping, or silently narrowing from `usize`.
   - Kept LSP protocol positions as `u32` and added explicit checked narrowing for UTF-16 columns in the LSP bridge.

21. Revisited long-term LSP/lexer dependency layering.
   - Kept the direct `mizar-lsp` dependency on `mizar-lexer` for the explicit `SourceSpan` bridge conversion because no frontend or diagnostic adapter crate exists yet to own that boundary.
   - Kept `source_range_from_lexer_span` and `lsp_range_from_lexer_span` in `mizar-lsp::range_mapper` so coordinate-space conversion remains visible at the protocol boundary.
   - Avoided adding a shared source-coordinate crate solely for this bridge; the bridge should move only when a concrete adapter crate has a broader reason to own lexer-to-session conversion.

22. Replaced linear active-symbol lookup with a trie-backed lexicon.
   - `UserSymbolIndex` now keeps a canonical spelling map for exact lookup and deterministic diagnostics plus an internal ASCII byte trie for longest-prefix lookup.
   - `longest_user_symbol_at` walks the trie from the requested byte offset and returns the deepest terminal candidate set, preserving existing longest-match, parser-context, scope-override, and candidate-order behavior.
   - Regression coverage includes overlapping spellings, identifier-shaped symbols, punctuation-shaped symbols, invalid offsets, scope overrides, and a large imported-symbol lexicon.
   - The Criterion benchmark now includes `active_user_symbol_lexicon/disambiguate_many_imported_symbols` for disambiguation against thousands of imported symbols.

23. Carried symbol kind and arity metadata in lexical summaries and active-symbol candidates.
   - `ExportedSymbolShape` and `UserSymbolCandidate` now carry `UserSymbolKind` and `UserSymbolArity`.
   - Same-spelling overload candidates retain kind and arity metadata for downstream parser and resolver phases.
   - `ParserLexContext` can restrict admitted active user symbols with a `UserSymbolKindSet` without rebuilding the active lexical environment.
   - Final `UserSymbol` tokens still keep only spelling and span; downstream phases recover candidate metadata from the active lexical environment.
   - Environment fingerprints include symbol kind and arity metadata, and tests cover same-spelling metadata preservation, kind-filtered parser contexts, invalid arity shapes, and metadata-sensitive fingerprints.

24. Split crate-local lexer tests by concern.
   - Replaced the large inline `src/lib.rs` unit-test module with `src/tests/mod.rs` and focused test modules for shell lexing, raw lexing, source loading/preprocessing/maps, import pre-scan, lexical environments, disambiguation, Phase 7 invariants, and scope skeletons.
   - Moved shared fixtures and assertion helpers into `src/tests/common.rs`.
   - Kept the existing corpus fixture test as the end-to-end lexical regression layer.

25. Reconfirmed responsibility boundaries between lexer, session/source, parser, diagnostics, and LSP crates.
   - Added an explicit responsibility-boundary table to the lexer design README covering source loading, preprocessing maps, raw scanning, import pre-scan, lexical environments, scope skeletons, parser lexical context, diagnostics, LSP coordinate conversion, and semantic phases.
   - Kept source-loading helpers and module-name helpers as executable boundary contracts for tests and early integration, while assigning production file I/O, source identity, snapshots, and rich retained maps to session/source or frontend services.
   - Recorded the intended dependency direction: session/source/frontend feed byte-oriented source text into `mizar-lexer`; parser and later semantic phases consume lexer handoff data; diagnostics/LSP adapters render and bridge coordinate spaces without making `mizar-lexer` depend on parser, resolver, session snapshot, diagnostic rendering, or protocol crates.

26. Revisited the public API stability boundary before parser integration.
   - Audited the public lexer transfer structs and kept fields visible for the next parser-integration milestone, while documenting those fields as provisional transfer-object escape hatches rather than the preferred stable read path.
   - Added constructors/accessors for the parser-facing stable surfaces: `SourceSpan`, raw/final tokens and token streams, `LexDiagnostic`, and lightweight module/symbol/fingerprint newtypes. `SourceSpan::new` rejects reversed ranges, while `SourceSpan::try_new` lets callers handle invalid external ranges without panicking.
   - Reconfirmed that externally matched public enums stay `#[non_exhaustive]`.
   - Documented the concrete next-`0.x` compatibility promise in the raw lexer design notes, including stable entry points, byte-span coordinates, diagnostic code/span stability, provisional diagnostic text, and preferred use of constructors/accessors.

27. Structured lexer diagnostics for downstream tooling.
   - Added machine-readable `LexDiagnosticPayload` values for no-candidate recovery, parser-context rejection with rejected candidate details, malformed string literals, and unsupported raw tokens.
   - Added machine-readable `SourcePreprocessDiagnosticPayload` values for carriage returns, non-ASCII code-region characters, and unterminated multi-line comments.
   - Kept stable diagnostic codes and byte spans while leaving human-facing message text provisional.
   - Added optional `diagnostic_payloads` fixture expectations so corpus tests can assert structured payload summaries without matching diagnostic wording.

28. Synchronized reserved-token tables after the Task 6 grammar audit.
   - Added `step` to the `mizar-lexer` reserved word table and the `mizar-session` reserved-word mirror.
   - Added `..` to the `mizar-lexer` reserved symbol table and admitted it in `NamespacePath` parser lexical context for relative-import parent prefixes.
   - Updated lexical fixtures, dot-disambiguation coverage, and traceability entries so the affected requirements are covered again.
   - Kept standalone `@` out of the reserved-symbol table; `@` remains reserved only as the raw annotation-name marker.

## Suggested Verification

After each task, run:

```text
cargo test -p mizar-lexer
cargo test -p mizar-test
```

For API stability or fuzz/benchmark work, also update this TODO file or remove completed items as appropriate.

## Text-Processing Audit Notes

This first-pass audit records common text-processing pitfalls and whether the current lexer crate resolves them, delegates them to another layer, or still needs a policy/test. It is intentionally a review note, not an implementation plan by itself.

| Topic | Current status | Owner / evidence | Follow-up |
|---|---|---|---|
| UTF-8 validation | Covered at the executable boundary | `load_source_text_from_bytes` validates source bytes and returns `SourceLoadError::InvalidUtf8` before lexer entry; `mizar-lexer` scanner APIs still receive `&str`. | Frontend/session source loader should reuse or mirror this behavior when that crate exists. |
| UTF-8 BOM | Covered for the UTF-8 boundary | `load_source_text_from_bytes` strips one leading UTF-8 BOM and records a loading map; lexer helpers still treat any reached `U+FEFF` as malformed/non-ASCII. See `raw_lexer.md`, `mizar-session/source.md`, and `mizar-session/source_map.md`. | Richer source-map tests remain in the source-loading/session layer. |
| Replacement character `U+FFFD` | Covered for invalid bytes and rejected in code regions | Invalid UTF-8 is not decoded lossily into `U+FFFD`; `preprocess_source_for_lexing` reports valid non-ASCII code-region characters as `NonAsciiCode`; comments may contain Unicode. | Source-loading/docs may optionally warn on suspicious Unicode in comments/doc text later. |
| LF / CRLF / CR handling | Covered at the executable boundary | `load_source_text_from_bytes` normalizes CRLF pairs to LF and records `NormalizedNewline` loading-map segments; lone `\r` remains a `CarriageReturn` preprocessing diagnostic and raw-scan error. | Session source-map tests should mirror the same CRLF-to-LF mapping policy. |
| Missing final newline / empty file / trailing newlines | Covered at lexer level | Empty raw stream is tested; `SourceLineIndex` accepts EOF; scanner does not require final newline. | Add corpus cases only if later parser/import prelude behavior depends on final newline. |
| Byte offset vs character column | Covered outside lexer | Lexer spans are byte offsets; `SourceLineIndex` uses zero-based byte columns and rejects non-UTF-8-boundary offsets. `mizar-session::source_map::LineMap` converts byte offsets to one-based Unicode scalar columns. | Keep human-facing conversion in source-map/session APIs. |
| LSP UTF-16 columns | Covered outside lexer | `mizar-lsp::range_mapper` converts source byte ranges to zero-based LSP UTF-16 positions using the session `LineMap`; lexer tokens still store only byte spans. | Keep protocol-specific conversion in the LSP bridge. |
| Preprocessed text vs original source spans | Covered at the executable boundary | `PreprocessedLexicalSource.preprocess_map` records original, removed-comment, and synthetic-whitespace segments so scanner lexical spans can be mapped back to loaded source ranges. Zero-length boundary mapping returns composite adjacent anchors. Session `PreprocessMap` retains richer snapshot/service ownership. | Frontend/session source-map implementation should reuse or mirror this behavior, and may attach explicit anchor roles such as before/removed/synthetic/after when richer composite mappings are introduced. |
| Unicode in code vs comments | Lexer boundary covered for known edge cases | Preprocessor allows Unicode inside comments and reports non-ASCII in code regions; tests cover Greek comment text, NBSP, zero-width chars, BOM-in-code, and full-width punctuation. | Source-loading may optionally warn on suspicious Unicode in comments/doc text later. |
| Unicode normalization and confusables | Not normalized by lexer | Identifier, numeral, and user-symbol helpers are ASCII-only, so non-ASCII code identifiers are rejected rather than normalized. Comments remain raw Unicode. | Document that lexer does not normalize source text; source-loading may optionally warn on suspicious Unicode in comments/doc text later. |
| Unicode whitespace and ASCII controls | Strictly rejected outside comments | Raw layout is exactly space, tab, LF; preprocessor reports non-ASCII code chars, and raw scanner hard-errors on unsupported ASCII controls such as vertical tab and form feed. | Keep source-loading and diagnostic-renderer policy aligned with this strict lexer boundary. |
| Tab display width | Delegated | Lexer stores byte spans only and treats tab as layout; diagnostics/source map own display columns. | Diagnostic renderer should test tab expansion policy. |
| Comment stripping and newline preservation | Lexer boundary covered for known edge cases | Tests cover ordinary/doc/multi-line comment removal, trivia retention, newline preservation, adjacent comments, EOF comments, synthetic layout around token boundaries, and preprocess-map segments for synthetic whitespace/newlines. | Session source-map tests should mirror these cases when the retained map service exists. |
| Nested multi-line comments | Non-nesting policy covered | Multi-line comments close at the first `=::`; inner `::=` spellings are ordinary comment text. Tests cover spans and preserved newlines. | Source-map tests should preserve the same non-nesting interpretation when mapping comment trivia. |
| Unterminated comments | Covered | Preprocessor emits `UnterminatedMultiLineComment` and preserves line structure; unit test exists. | Ensure frontend maps the diagnostic to original source after source-map implementation. |
| Annotation visibility | Covered at lexer boundary | Tests confirm annotation syntax remains in lexical text for parser ownership. | Parser tests should own annotation argument validation. |
| String literal escapes and recovery | Partially covered | Helper and disambiguator tests cover supported escapes, invalid escapes, malformed strings, and context-required string positions. | Property/fuzz tests should include string-required context and malformed recovery spans. |
| Quote characters as user symbols | Partially covered | Design states strings are contextual; disambiguator tests cover string-required context and string outside context. | Keep parser-context API tests broad enough when grammar integration changes. |
| Raw/final token span integrity | Covered by crate-local seeds | Tests cover raw span contiguity, final token span preservation, and property-style final token span checks across `lex` and `disambiguate`. | Broaden with fuzz coverage later. |
| Error-recovery token spans | Covered by crate-local seeds | Disambiguator recovery diagnostics/tokens are included in final-token span invariant seeds. | Broaden with fuzz coverage later. |
| Huge files / long lines / memory | Benchmark covered for large `.miz`-like input | `lexer_pipeline` measures preprocessing, raw scanning, and `SourceLineIndex` construction separately. | Add specialized long-line benchmarks if later profiling shows line-length sensitivity. |
| Panic safety under arbitrary text | Covered for lexer valid UTF-8 entry points | The `lexer_valid_utf8` cargo-fuzz target exercises `preprocess_source_for_lexing`, direct `scan_raw`, and `scan_raw` over preprocessed lexical text. | Promote minimized fuzz failures into the committed corpus when discovered. |
| File/path text normalization | Partially covered outside lexing | `module_source_name_from_path` normalizes separators and validates identifier-shaped components. | Source-loading/path layer should own symlink, case, `.`/`..`, and platform path edge cases. |
