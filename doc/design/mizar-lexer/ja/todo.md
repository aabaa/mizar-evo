# mizar-lexer TODO

> Canonical language: English. English canonical version: [../en/todo.md](../en/todo.md).

この文書は、final-token source span 追加後の lexer quality review で見つかった follow-up tasks を記録します。

## Ordered Task List

1. source path normalization は lexer 外で cover する。
    - `.`/`..`、symlinks、case policy、package-root escape attempts、platform-specific separators を source-loading/path layer で test する。

2. lexer、session、LSP crates 間の shared source-span ownership を決める。
    - `SourceRange` を crate-local のまま explicit conversion APIs でつなぐか、common source-coordinate crate に移すかを再検討する。
    - lexer token spans を session/LSP diagnostics へ接続し、実際の integration pressure が見えてから判断する。

3. session source maps の line/column overflow policy を決める。
    - `mizar-session::LineMap` は現在 user-facing line/column values を `u32` として返す。extremely large files または long lines に対して、session APIs が `usize` を使うべきか overflow error を返すべきかを review する。
    - LSP protocol positions は `u32` のままにしつつ、session coordinates から narrow する箇所は explicit かつ tested にする。

## Completed Tasks

1. `SourceLineIndex` の offset validation を強化した。
   - `location` と `range` は UTF-8 character boundary ではない byte offset に対して `None` を返す。
   - documented convention の zero-based line と zero-based byte column を保ったまま、multi-byte UTF-8 text の unit test を追加した。

2. final token span の property-style tests を追加した。
   - crate-local Phase 7 unit tests で複数の `lex` / `disambiguate` seed を cover するようにした。
   - すべての final token について `source[token.span.start..token.span.end] == token.lexeme` を確認する。
   - one-to-one raw mapping、`LexemeRun` split、string-required context、malformed string recovery、parser-context rejection、unsupported raw-token recovery spans を seed に含めた。

3. unsupported Unicode code-region diagnostics を固定した。
   - crate-local raw lexer / preprocessor tests で NBSP、zero-width space、zero-width joiner/non-joiner、full-width punctuation、code region 内の `U+FEFF` を cover するようにした。
   - これらが layout や valid token text として扱われず、byte span 付きの stable な `NonAsciiCode` preprocessor diagnostics および stable な raw-scan hard errors になることを確認する。

4. unsupported ASCII control characters を固定した。
   - crate-local tests で code region 内の vertical tab と form feed を cover するようにした。
   - これらが layout ではなく、valid token text として扱われずに stable な raw-scan hard errors になることを確認する。

5. comment-removal edge tests を拡充した。
   - crate-local preprocessing tests で adjacent comments、EOF comments、token-shaped text の間にある comments、複数の preserved newlines を含む multi-line comments を cover するようにした。
   - inline comment removal では、lexical text が line structure を保ち token を accidental に concat しないよう、必要に応じて synthetic layout を挿入する。

6. nested multi-line comment policy を決定した。
   - multi-line comments は non-nesting と文書化した。`::=` opener の後に最初に現れる `=::` が comment を閉じる。
   - crate-local preprocessing tests で内部の `::=` spelling を通常の comment text として扱うことを、trivia spans と preserved newlines を含めて cover するようにした。

7. 最小限の documentation tests / examples を追加した。
   - crate-level doctests で `scan_raw`, `lex`, `disambiguate` を cover するようにした。
   - examples で token span が scanner input への byte offset であることを示した。
   - parser-facing API の小さく安定した surface に留めた。

8. public API stability policy を決定した。
   - public enums に `#[non_exhaustive]` を付け、今後増える可能性のある categories に対して downstream crates が wildcard match arms を保つようにした。
   - public data struct fields は corpus と初期 integration code が使う parser-facing transfer objects なので visible のままにした。
   - crate-level docs と raw lexer design notes に、`0.1` APIs は後続の stability milestone までは provisional であることを明記した。

9. source-text normalization policy を文書化した。
   - crate-level docs と design notes に、lexer は Unicode normalization を行わないことを明記した。
   - code-region identifiers、numerals、reserved spellings、user-symbol spellings は lexer boundary で ASCII-only のままにした。
   - comments と documentation text は、後続の documentation/source-loading layer が warning を追加しない限り raw Unicode trivia として保持する方針を明記した。

10. fuzz coverage を追加した。
   - arbitrary valid UTF-8 strings 用の `cargo-fuzz` target `lexer_valid_utf8` を追加した。
   - target は `preprocess_source_for_lexing`、direct `scan_raw`、preprocessed lexical text に対する `scan_raw` を exercise する。
   - fuzz assertion で span validity、comment lexeme slicing、raw token lexeme slicing、成功した raw scan の full input coverage を固定した。
   - seed corpus entry は ASCII/layout/annotation text と documentation/comment/code region 内 Unicode を cover する。

11. performance benchmarking を追加した。
   - large `.miz`-like source 用の Criterion benchmark を追加した。
   - benchmark は `preprocess_source_for_lexing`、`scan_raw`、`SourceLineIndex` construction を分けて測定する。
   - module resolution、parser context、imported symbol loading を含めず、lexer-local に保つ。

12. source-loading boundary における UTF-8 BOM handling policy を決定した。
   - Package-authored source loading は先頭 UTF-8 BOM を一つ受け入れ、`LoadedSource.text` を構築する前、または `mizar-lexer` を呼ぶ前に strip する。
   - Direct lexer helper calls は strict のままにする。`preprocess_source_for_lexing` や `scan_raw` に届いた `U+FEFF` は malformed lexer-boundary input のままである。
   - BOM stripping 後の lexer span は post-strip loaded text への byte offset であり、BOM-prefixed disk file では `LoadingMap` が loaded offset `0` を original file byte offset `3` へ対応付ける。

13. UTF-8 file loading boundary behavior を仕様化して test した。
   - UTF-8 validation と leading BOM stripping の executable boundary として crate-local `load_source_text_from_bytes` を追加した。
   - Invalid UTF-8 は lexer entry 前に `SourceLoadError::InvalidUtf8` を返し、lossy decode で `U+FFFD` にしない。
   - 先頭 UTF-8 BOM stripping は `LoadedSourceText.loading_map` で test し、loaded offset `0` が original byte offset `3` に map されることを確認した。
   - Non-leading `U+FEFF` は loaded text に残り、code に現れた場合は lexer-boundary preprocessing/raw scanning で reject されるままにした。

14. newline normalization を仕様化して test した。
   - `load_source_text_from_bytes` は lexer entry 前に CRLF pairs を LF に normalize する。
   - `SourceLoadingMapSegment::NormalizedNewline` は normalized LF と original two-byte CRLF range の対応を記録する。
   - plain CRLF input、leading BOM stripping 後の CRLF、lone `\r` を malformed lexer-boundary input として保持する case を test した。

15. preprocess source-map tests を実装した。
   - `PreprocessedLexicalSource` は original、removed-comment、synthetic-whitespace segments を持つ lightweight `SourcePreprocessMap` を保持するようになった。
   - ordinary comment removal、documentation comment retention、synthetic spaces/newlines、removed comments をまたぐ lexical ranges を tests で cover した。
   - lexer/preprocessor diagnostic byte ranges を original source ranges に map できることも tests で固定した。

16. zero-length preprocess-map boundaries で composite anchors を返すようにした。
   - original text、removed comments、synthetic whitespace の境界にある lexical insertion points は、隣接する source anchors をすべて返す。
   - inserted synthetic layout がある場合とない場合の removed-comment boundaries を tests で cover した。

17. user-facing column conversion を lexer 外に保つようにした。
   - one-based Unicode scalar line/column conversion 用の `LineMap` tests を持つ minimal `mizar-session` crate を追加した。
   - zero-based LSP UTF-16 positions 用の range-mapper tests を持つ minimal `mizar-lsp` crate を追加した。
   - `mizar-lexer` は引き続き byte-span oriented であり、user-facing または LSP column conversion を行わない。

## Suggested Verification

各 task の後に以下を実行します。

```text
cargo test -p mizar-lexer
cargo test -p mizar-test
```

API stability、fuzz、benchmark 作業では、この TODO file の更新または完了項目の削除も行ってください。

## Text-Processing Audit Notes

この first-pass audit は、text-processing で一般的に問題になりやすい点について、現在の lexer crate が解決しているか、別 layer に委譲しているか、policy/test が残っているかを記録する。これは review note であり、単独の implementation plan ではない。

| Topic | Current status | Owner / evidence | Follow-up |
|---|---|---|---|
| UTF-8 validation | executable boundary で covered | `load_source_text_from_bytes` は source bytes を validate し、lexer entry 前に `SourceLoadError::InvalidUtf8` を返す。scanner APIs は引き続き `&str` を受け取る。 | frontend/session source loader ができた段階で、この behavior を reuse または mirror する。 |
| UTF-8 BOM | UTF-8 boundary では covered | `load_source_text_from_bytes` は先頭 UTF-8 BOM を一つ strip し、loading map を記録する。lexer helpers は、届いた `U+FEFF` を malformed/non-ASCII として扱い続ける。`raw_lexer.md`、`mizar-session/source.md`、`mizar-session/source_map.md` を参照。 | richer source-map tests は source-loading/session layer に残る。 |
| replacement character `U+FFFD` | invalid bytes と code region で covered | Invalid UTF-8 は lossy decode で `U+FFFD` にしない。`preprocess_source_for_lexing` は valid non-ASCII code-region characters を `NonAsciiCode` として報告する。comments は Unicode allowed。 | comments/doc text の suspicious Unicode warning は将来 source-loading/docs 側で検討する。 |
| LF / CRLF / CR handling | executable boundary で covered | `load_source_text_from_bytes` は CRLF pairs を LF に normalize し、`NormalizedNewline` loading-map segments を記録する。lone `\r` は `CarriageReturn` preprocessing diagnostic と raw-scan error のまま。 | session source-map tests は同じ CRLF-to-LF mapping policy を mirror する。 |
| missing final newline / empty file / trailing newlines | lexer level では covered | empty raw stream の test があり、`SourceLineIndex` は EOF を受け入れる。scanner は final newline を要求しない。 | parser/import prelude が final newline に依存する場合のみ corpus cases を追加する。 |
| byte offset vs character column | lexer 外で covered | lexer spans は byte offsets。`SourceLineIndex` は zero-based byte columns を使い、non-UTF-8-boundary offsets を reject する。`mizar-session::source_map::LineMap` は byte offsets を one-based Unicode scalar columns に変換する。 | human-facing conversion は source-map/session APIs に保つ。 |
| LSP UTF-16 columns | lexer 外で covered | `mizar-lsp::range_mapper` は session `LineMap` を使い、source byte ranges を zero-based LSP UTF-16 positions に変換する。lexer tokens は引き続き byte spans のみを保存する。 | protocol-specific conversion は LSP bridge に保つ。 |
| preprocessed text vs original source spans | executable boundary で covered | `PreprocessedLexicalSource.preprocess_map` は original、removed-comment、synthetic-whitespace segments を記録し、scanner lexical spans を loaded source ranges に戻せる。Zero-length boundary mapping は composite adjacent anchors を返す。Session `PreprocessMap` はより rich な snapshot/service ownership を保持する。 | frontend/session source-map implementation はこの behavior を reuse または mirror する。より rich な composite mapping を導入する段階で、before/removed/synthetic/after などの explicit anchor roles を付けてもよい。 |
| Unicode in code vs comments | known edge cases は lexer boundary で covered | preprocessor は comments 内の Unicode を許し、code regions の non-ASCII を報告する。Greek comment text、NBSP、zero-width chars、BOM-in-code、full-width punctuation の tests がある。 | comments/doc text の suspicious Unicode warning は将来 source-loading/docs 側で検討する。 |
| Unicode normalization and confusables | lexer は normalize しない | identifier/numeral/user-symbol helpers は ASCII-only なので、non-ASCII code identifiers は normalize せず reject される。comments は raw Unicode のまま保持される。 | lexer が source text を normalize しないことを明記する。comments/doc text の suspicious Unicode warning は将来 source-loading/docs 側で検討する。 |
| Unicode whitespace and ASCII controls | comments 外では strict reject | raw layout は space, tab, LF のみ。preprocessor は non-ASCII code chars を報告し、raw scanner は vertical tab や form feed などの unsupported ASCII controls を hard error にする。 | この strict lexer boundary と source-loading / diagnostic-renderer policy を同期し続ける。 |
| tab display width | delegated | lexer は byte spans のみを保持し、tab は layout として扱う。diagnostics/source map が display columns を担当する。 | diagnostic renderer で tab expansion policy を test する。 |
| comment stripping and newline preservation | known edge cases は lexer boundary で covered | ordinary/doc/multi-line comment removal、trivia retention、newline preservation、adjacent comments、EOF comments、token boundary 付近の synthetic layout、synthetic whitespace/newlines の preprocess-map segments の tests がある。 | retained map service ができたら session source-map tests でも同じ cases を mirror する。 |
| nested multi-line comments | non-nesting policy は covered | multi-line comments は最初の `=::` で閉じ、内部の `::=` spelling は通常の comment text として扱う。tests は spans と preserved newlines を cover する。 | source-map tests でも comment trivia mapping 時に同じ non-nesting interpretation を保つ。 |
| unterminated comments | covered | preprocessor は `UnterminatedMultiLineComment` を emit し、line structure を保持する。unit test あり。 | source-map implementation 後に frontend が diagnostic を original source に map することを確認する。 |
| annotation visibility | lexer boundary では covered | annotation syntax が parser ownership のため lexical text に残ることを tests が確認している。 | annotation argument validation は parser tests が担当する。 |
| string literal escapes and recovery | partially covered | helper と disambiguator tests は supported escapes、invalid escapes、malformed strings、context-required string positions を cover している。 | property/fuzz tests に string-required context と malformed recovery spans を含める。 |
| quote characters as user symbols | partially covered | design は strings を contextual と定め、disambiguator tests は string-required context と string outside context を cover している。 | grammar integration が変わるときに parser-context API tests を広げる。 |
| raw/final token span integrity | crate-local seeds で covered | tests は raw span contiguity、final token span preservation、`lex` と `disambiguate` を横断する property-style final token span checks を cover する。 | 後で fuzz coverage により広げる。 |
| error-recovery token spans | crate-local seeds で covered | disambiguator recovery diagnostics/tokens は final-token span invariant seeds に含まれている。 | 後で fuzz coverage により広げる。 |
| huge files / long lines / memory | large `.miz`-like input は benchmark covered | `lexer_pipeline` が preprocessing、raw scanning、`SourceLineIndex` construction を分けて測定する。 | 今後の profiling で line-length sensitivity が見えた場合は specialized long-line benchmarks を追加する。 |
| panic safety under arbitrary text | lexer の valid UTF-8 entry points は covered | `lexer_valid_utf8` cargo-fuzz target が `preprocess_source_for_lexing`、direct `scan_raw`、preprocessed lexical text 上の `scan_raw` を exercise する。 | fuzz failure が見つかった場合は minimize して committed corpus に昇格する。 |
| file/path text normalization | lexing 外で partially covered | `module_source_name_from_path` は separators を normalize し、identifier-shaped components を validate する。 | symlink、case、`.`/`..`、platform path edge cases は source-loading/path layer が担当する。 |
