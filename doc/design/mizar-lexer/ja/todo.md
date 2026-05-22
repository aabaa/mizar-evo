# mizar-lexer TODO

> Canonical language: English. English canonical version: [../en/todo.md](../en/todo.md).

この文書は、final-token source span 追加後の lexer quality review で見つかった follow-up tasks を記録します。

## Priority Tasks

1. final token span の property-style test を追加する。
   - 複数の `lex` / `disambiguate` seed を cover する。
   - すべての final token について `source[token.span.start..token.span.end] == token.lexeme` を確認する。
   - one-to-one raw mapping、`LexemeRun` split、string-required context、error recovery を含める。

2. public API stability policy を決める。
   - `TokenKind`, `RawTokenKind`, `LexDiagnosticCode`, `ImportPrescanDiagnosticCode`, `ScopeSkeletonDiagnosticCode`, `SourcePreprocessDiagnosticCode`, `LexicalEnvironmentError` などの public enum に `#[non_exhaustive]` を付けるか検討する。
   - 代替案として、`0.1` API では compatibility guarantee なしで breaking change を行う可能性があることを明記する。
   - downstream crates が依存する前に、現在 public な fields の一部を `pub(crate)` に絞るべきか review する。

3. 最小限の documentation tests / examples を追加する。
   - `scan_raw`, `lex`, `disambiguate` の examples を追加する。
   - token span が scanner input への byte offset であることを示す。
   - parser-facing API が進化しても安定する小さな examples に留める。

4. fuzz coverage を追加する。
   - arbitrary byte input または valid UTF-8 strings を対象にした `scan_raw` 用 `cargo-fuzz` target を追加する。
   - 見つかった failure は minimize し、corpus regression として commit する前に stable case として `tests/lexical` に昇格する。

5. performance benchmarking を追加する。
   - large `.miz`-like source に対する `scan_raw` throughput を benchmark する。
   - module resolution、parser context、imported symbol loading から独立した benchmark にする。

6. source-loading boundary における UTF-8 BOM handling policy を決める。
   - raw file input の先頭 UTF-8 BOM は受け入れ、`mizar-lexer` entry point が `&str` を受け取る前に source-loading 側で取り除く方針を優先する。
   - direct lexer helper calls は strict のままにする。`preprocess_source_for_lexing` や `scan_raw` に届いた `U+FEFF` は silently disappear させず、malformed source precondition として扱う。
   - BOM stripping 後の token span が loaded text offsets で測られること、および source map が original file byte offsets にどう対応するかを文書化する。
   - frontend/session source loader ができた段階で source-loading tests を追加する。それまでは lexer behavior は変更しない。

## Completed Tasks

1. `SourceLineIndex` の offset validation を強化した。
   - `location` と `range` は UTF-8 character boundary ではない byte offset に対して `None` を返す。
   - documented convention の zero-based line と zero-based byte column を保ったまま、multi-byte UTF-8 text の unit test を追加した。

## Suggested Verification

各 task の後に以下を実行します。

```text
cargo test -p mizar-lexer
cargo test -p mizar-test
```

API stability、fuzz、benchmark 作業では、この TODO file の更新または完了項目の削除も行ってください。
