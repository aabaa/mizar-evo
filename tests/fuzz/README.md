# Fuzz Regressions

Reserved for minimized fuzz reproducers promoted into the committed corpus.
Each promoted case must preserve the original failure category and seed
metadata in its expectation sidecar.

Active cargo-fuzz targets live under `fuzz/`. Run the lexer or frontend
valid-UTF-8 targets with:

```text
cargo fuzz run lexer_valid_utf8
cargo fuzz run frontend_valid_utf8
```
