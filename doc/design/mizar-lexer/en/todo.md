# mizar-lexer TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

This document records follow-up tasks identified during the lexer quality review after adding final-token source spans.

## Priority Tasks

1. Add property-style final token span tests.
   - Cover several `lex` and `disambiguate` seeds.
   - Assert that every final token span points back to the token spelling: `source[token.span.start..token.span.end] == token.lexeme`.
   - Include one-to-one raw mappings, `LexemeRun` splits, string-required context, and error recovery.

2. Decide public API stability policy.
   - Consider `#[non_exhaustive]` for public enums such as `TokenKind`, `RawTokenKind`, `LexDiagnosticCode`, `ImportPrescanDiagnosticCode`, `ScopeSkeletonDiagnosticCode`, `SourcePreprocessDiagnosticCode`, and `LexicalEnvironmentError`.
   - Alternatively document that the `0.1` API may make breaking changes without compatibility guarantees.
   - Review whether any currently public fields should be narrowed to `pub(crate)` before downstream crates depend on them.

3. Add minimal documentation tests or examples.
   - Add examples for `scan_raw`, `lex`, and `disambiguate`.
   - Show that token spans are byte offsets into the scanner input.
   - Keep examples small enough that they stay stable as the parser-facing API evolves.

4. Add fuzz coverage.
   - Add a `cargo-fuzz` target for `scan_raw` over arbitrary byte input or valid UTF-8 strings.
   - Minimize any discovered failures and promote stable cases into `tests/lexical` before committing them as corpus regressions.

5. Add performance benchmarking.
   - Benchmark `scan_raw` throughput on a large `.miz`-like source.
   - Keep the benchmark independent of module resolution, parser context, and imported symbol loading.

6. Decide UTF-8 BOM handling policy at the source-loading boundary.
   - Prefer accepting a leading UTF-8 BOM in raw file input and stripping it before `mizar-lexer` entry points receive `&str`.
   - Keep direct lexer helper calls strict: a `U+FEFF` that reaches `preprocess_source_for_lexing` or `scan_raw` should remain a malformed source precondition rather than silently disappearing.
   - Document whether token spans after BOM stripping are measured in loaded text offsets and how the source map relates them back to original file byte offsets.
   - Add source-loading tests once the frontend/session source loader exists; avoid changing lexer behavior until that boundary is implemented.

## Completed Tasks

1. Hardened `SourceLineIndex` offset validation.
   - `location` and `range` now reject byte offsets that are not UTF-8 character boundaries by returning `None`.
   - Unit tests cover multi-byte UTF-8 text while preserving the documented zero-based line and zero-based byte-column convention.

## Suggested Verification

After each task, run:

```text
cargo test -p mizar-lexer
cargo test -p mizar-test
```

For API stability or fuzz/benchmark work, also update this TODO file or remove completed items as appropriate.
