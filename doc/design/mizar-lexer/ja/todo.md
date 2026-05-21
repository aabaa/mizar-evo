# mizar-lexer TODO

> Canonical language: English. English canonical version: [../en/todo.md](../en/todo.md).

この文書は、final-token source span 追加後の lexer quality review で見つかった follow-up tasks を記録します。

## Priority Tasks

1. `SourceLineIndex` の offset validation を強化する。
   - `location` と `range` が UTF-8 character boundary ではない byte offset を reject するか決める。
   - reject する場合、non-boundary offset では `None` を返し、multi-byte UTF-8 text を使った unit test を追加する。
   - documented convention は zero-based line と zero-based byte column のままにする。

2. final token span の property-style test を追加する。
   - 複数の `lex` / `disambiguate` seed を cover する。
   - すべての final token について `source[token.span.start..token.span.end] == token.lexeme` を確認する。
   - one-to-one raw mapping、`LexemeRun` split、string-required context、error recovery を含める。

3. public API stability policy を決める。
   - `TokenKind`, `RawTokenKind`, `LexDiagnosticCode`, `ImportPrescanDiagnosticCode`, `ScopeSkeletonDiagnosticCode`, `SourcePreprocessDiagnosticCode`, `LexicalEnvironmentError` などの public enum に `#[non_exhaustive]` を付けるか検討する。
   - 代替案として、`0.1` API では compatibility guarantee なしで breaking change を行う可能性があることを明記する。
   - downstream crates が依存する前に、現在 public な fields の一部を `pub(crate)` に絞るべきか review する。

4. 最小限の documentation tests / examples を追加する。
   - `scan_raw`, `lex`, `disambiguate` の examples を追加する。
   - token span が scanner input への byte offset であることを示す。
   - parser-facing API が進化しても安定する小さな examples に留める。

5. fuzz coverage を追加する。
   - arbitrary byte input または valid UTF-8 strings を対象にした `scan_raw` 用 `cargo-fuzz` target を追加する。
   - 見つかった failure は minimize し、corpus regression として commit する前に stable case として `tests/lexical` に昇格する。

6. performance benchmarking を追加する。
   - large `.miz`-like source に対する `scan_raw` throughput を benchmark する。
   - module resolution、parser context、imported symbol loading から独立した benchmark にする。

## Suggested Verification

各 task の後に以下を実行します。

```text
cargo test -p mizar-lexer
cargo test -p mizar-test
```

API stability、fuzz、benchmark 作業では、この TODO file の更新または完了項目の削除も行ってください。
