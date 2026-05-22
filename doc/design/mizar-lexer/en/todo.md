# mizar-lexer TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

This document records follow-up tasks identified during the lexer quality review after adding final-token source spans.

## Ordered Task List

1. Document source-text normalization policy.
   - State that the lexer does not perform Unicode normalization and that code-region identifiers/symbols remain ASCII-only.
   - Keep comments and documentation text as raw Unicode unless a later documentation/source-loading layer adds warnings.

2. Add fuzz coverage.
   - Add a `cargo-fuzz` target for `scan_raw` over arbitrary byte input or valid UTF-8 strings.
   - Include arbitrary valid UTF-8 input for `preprocess_source_for_lexing` and `scan_raw`.
   - Minimize any discovered failures and promote stable cases into `tests/lexical` before committing them as corpus regressions.

3. Add performance benchmarking.
   - Benchmark `scan_raw` throughput on a large `.miz`-like source.
   - Measure raw scanning, preprocessing, and `SourceLineIndex` construction separately.
   - Keep benchmarks independent of module resolution, parser context, and imported symbol loading.

4. Decide UTF-8 BOM handling policy at the source-loading boundary.
   - Prefer accepting a leading UTF-8 BOM in raw file input and stripping it before `mizar-lexer` entry points receive `&str`.
   - Keep direct lexer helper calls strict: a `U+FEFF` that reaches `preprocess_source_for_lexing` or `scan_raw` should remain a malformed source precondition rather than silently disappearing.
   - Document whether token spans after BOM stripping are measured in loaded text offsets and how the source map relates them back to original file byte offsets.
   - Add source-loading tests once the frontend/session source loader exists; avoid changing lexer behavior until that boundary is implemented.

5. Specify and test UTF-8 file loading.
   - Reject invalid UTF-8 before lexer entry and avoid lossy decoding into `U+FFFD`.
   - Decide and test leading UTF-8 BOM stripping, including original-byte-offset source-map behavior.

6. Specify and test newline normalization.
   - Define CRLF-to-LF behavior before lexer entry.
   - Ensure the source map can relate normalized lexical/source text offsets back to original file byte offsets.

7. Implement preprocess source-map tests.
   - Cover ordinary comment removal, documentation comment retention, synthetic whitespace/newline segments, and lexical ranges spanning removed comments.
   - Ensure diagnostics from lexer/preprocessor helpers can be mapped back to original source ranges.

8. Keep user-facing column conversion outside lexer.
    - Test Unicode scalar columns in the source-map/session layer.
    - Test LSP UTF-16 conversion in the LSP bridge, not in `mizar-lexer`.

9. Cover source path normalization outside lexer.
    - Test `.`/`..`, symlinks, case policy, package-root escape attempts, and platform-specific separators in the source-loading/path layer.

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
| UTF-8 validation | Delegated by design | `mizar-lexer` receives `&str`; source loading validates file bytes before lexer entry. See `raw_lexer.md` Source Preconditions and architecture source loading step. | Implement source-loader tests when that crate exists. |
| UTF-8 BOM | Deferred | Current lexer helpers treat `U+FEFF` as malformed/non-ASCII. Priority task 6 records the intended source-loading policy. | Decide and test BOM stripping at source-loading boundary. |
| Replacement character `U+FFFD` | Mostly rejected in code regions | `preprocess_source_for_lexing` reports any non-ASCII code-region char as `NonAsciiCode`; comments may contain Unicode. | Add explicit source-loading policy that invalid UTF-8 must not be decoded lossy into `U+FFFD`. |
| LF / CRLF / CR handling | Delegated plus guarded | Source loading is expected to normalize platform newlines; lexer helper reports `CarriageReturn`; raw scanner rejects `\r`. | Source loader and source-map tests must cover CRLF-to-LF mapping. |
| Missing final newline / empty file / trailing newlines | Covered at lexer level | Empty raw stream is tested; `SourceLineIndex` accepts EOF; scanner does not require final newline. | Add corpus cases only if later parser/import prelude behavior depends on final newline. |
| Byte offset vs character column | Lexer-local policy covered | Lexer spans are byte offsets; `SourceLineIndex` uses zero-based byte columns and rejects non-UTF-8-boundary offsets. Session source map specifies user-facing Unicode scalar columns. | Source-map layer must own human-facing conversion. |
| LSP UTF-16 columns | Delegated by design | `raw_lexer.md` and `source_map.md` keep LSP UTF-16 conversion outside lexer. | Add LSP bridge tests when available. |
| Preprocessed text vs original source spans | Designed, not implemented here | `raw_lexer.md` states lexer spans point into scanner input; `source_map.md` defines `PreprocessMap` ownership outside lexer. | Add frontend/source-map tests for comment removal and synthetic whitespace mapping. |
| Unicode in code vs comments | Lexer boundary covered for known edge cases | Preprocessor allows Unicode inside comments and reports non-ASCII in code regions; tests cover Greek comment text, NBSP, zero-width chars, BOM-in-code, and full-width punctuation. | Source-loading may optionally warn on suspicious Unicode in comments/doc text later. |
| Unicode normalization and confusables | Not normalized by lexer | Identifier, numeral, and user-symbol helpers are ASCII-only, so non-ASCII code identifiers are rejected rather than normalized. Comments remain raw Unicode. | Document that lexer does not normalize source text; source-loading may optionally warn on suspicious Unicode in comments/doc text later. |
| Unicode whitespace and ASCII controls | Strictly rejected outside comments | Raw layout is exactly space, tab, LF; preprocessor reports non-ASCII code chars, and raw scanner hard-errors on unsupported ASCII controls such as vertical tab and form feed. | Keep source-loading and diagnostic-renderer policy aligned with this strict lexer boundary. |
| Tab display width | Delegated | Lexer stores byte spans only and treats tab as layout; diagnostics/source map own display columns. | Diagnostic renderer should test tab expansion policy. |
| Comment stripping and newline preservation | Lexer boundary covered for known edge cases | Tests cover ordinary/doc/multi-line comment removal, trivia retention, newline preservation, adjacent comments, EOF comments, and synthetic layout around token boundaries. | Source-map tests should cover mapping synthetic whitespace/newline segments back to original source ranges. |
| Nested multi-line comments | Non-nesting policy covered | Multi-line comments close at the first `=::`; inner `::=` spellings are ordinary comment text. Tests cover spans and preserved newlines. | Source-map tests should preserve the same non-nesting interpretation when mapping comment trivia. |
| Unterminated comments | Covered | Preprocessor emits `UnterminatedMultiLineComment` and preserves line structure; unit test exists. | Ensure frontend maps the diagnostic to original source after source-map implementation. |
| Annotation visibility | Covered at lexer boundary | Tests confirm annotation syntax remains in lexical text for parser ownership. | Parser tests should own annotation argument validation. |
| String literal escapes and recovery | Partially covered | Helper and disambiguator tests cover supported escapes, invalid escapes, malformed strings, and context-required string positions. | Property/fuzz tests should include string-required context and malformed recovery spans. |
| Quote characters as user symbols | Partially covered | Design states strings are contextual; disambiguator tests cover string-required context and string outside context. | Keep parser-context API tests broad enough when grammar integration changes. |
| Raw/final token span integrity | Covered by crate-local seeds | Tests cover raw span contiguity, final token span preservation, and property-style final token span checks across `lex` and `disambiguate`. | Broaden with fuzz coverage later. |
| Error-recovery token spans | Covered by crate-local seeds | Disambiguator recovery diagnostics/tokens are included in final-token span invariant seeds. | Broaden with fuzz coverage later. |
| Huge files / long lines / memory | Not covered | `SourceLineIndex` stores all character boundaries; raw scanning clones token lexemes. No benchmark yet. | Use benchmark task to measure large `.miz`-like input and long-line behavior. |
| Panic safety under arbitrary text | Partially covered | Scanner uses `char_indices`; fuzz task is still pending. | Add fuzz target for valid UTF-8 strings and promote regressions into corpus. |
| File/path text normalization | Partially covered outside lexing | `module_source_name_from_path` normalizes separators and validates identifier-shaped components. | Source-loading/path layer should own symlink, case, `.`/`..`, and platform path edge cases. |
