# mizar-lexer TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

This document records follow-up tasks identified during the lexer quality review after adding final-token source spans.

## Priority Tasks

1. Harden `SourceLineIndex` offset validation.
   - Decide whether `location` and `range` should reject byte offsets that are not UTF-8 character boundaries.
   - If rejected, return `None` for non-boundary offsets and add unit tests using multi-byte UTF-8 text.
   - Keep the documented convention as zero-based line and zero-based byte column.

2. Add property-style final token span tests.
   - Cover several `lex` and `disambiguate` seeds.
   - Assert that every final token span points back to the token spelling: `source[token.span.start..token.span.end] == token.lexeme`.
   - Include one-to-one raw mappings, `LexemeRun` splits, string-required context, and error recovery.

3. Decide public API stability policy.
   - Consider `#[non_exhaustive]` for public enums such as `TokenKind`, `RawTokenKind`, `LexDiagnosticCode`, `ImportPrescanDiagnosticCode`, `ScopeSkeletonDiagnosticCode`, `SourcePreprocessDiagnosticCode`, and `LexicalEnvironmentError`.
   - Alternatively document that the `0.1` API may make breaking changes without compatibility guarantees.
   - Review whether any currently public fields should be narrowed to `pub(crate)` before downstream crates depend on them.

4. Add minimal documentation tests or examples.
   - Add examples for `scan_raw`, `lex`, and `disambiguate`.
   - Show that token spans are byte offsets into the scanner input.
   - Keep examples small enough that they stay stable as the parser-facing API evolves.

5. Add fuzz coverage.
   - Add a `cargo-fuzz` target for `scan_raw` over arbitrary byte input or valid UTF-8 strings.
   - Minimize any discovered failures and promote stable cases into `tests/lexical` before committing them as corpus regressions.

6. Add performance benchmarking.
   - Benchmark `scan_raw` throughput on a large `.miz`-like source.
   - Keep the benchmark independent of module resolution, parser context, and imported symbol loading.

## Suggested Verification

After each task, run:

```text
cargo test -p mizar-lexer
cargo test -p mizar-test
```

For API stability or fuzz/benchmark work, also update this TODO file or remove completed items as appropriate.
