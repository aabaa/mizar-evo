# mizar-lexer TODO

> Canonical language: English. English canonical version: [../en/todo.md](../en/todo.md).

この文書は、final-token source span 追加後の lexer quality review で見つかった follow-up tasks を記録します。

## Ordered Task List

1. final token span の property-style test を追加する。
   - 複数の `lex` / `disambiguate` seed を cover する。
   - すべての final token について `source[token.span.start..token.span.end] == token.lexeme` を確認する。
   - one-to-one raw mapping、`LexemeRun` split、string-required context、malformed recovery cases、error recovery spans を含める。

2. unsupported Unicode code-region diagnostics を固定する。
   - NBSP、zero-width space、zero-width joiner/non-joiner、full-width punctuation、code region 内の `U+FEFF` について lexer/preprocessor fixtures を追加する。
   - これらが layout や valid token text として扱われず、stable な `NonAsciiCode` diagnostics または raw-scan errors になることを確認する。

3. unsupported ASCII control characters を固定する。
   - code region 内の vertical tab と form feed の tests を追加する。
   - これらが layout ではなく、deterministic diagnostics または raw-scan errors になることを確認する。

4. comment-removal edge tests を拡充する。
   - adjacent comments、EOF comments、token-shaped text の間にある comments、source loading 後に複数の preserved newlines を含む multi-line comments を cover する。
   - lexical text が line structure を保ち、token を accidental に concat しないことを確認する。

5. nested multi-line comment policy を決める。
   - nested `::=` ... `=::` comments を unsupported とするか、認識するかを文書化する。
   - 選んだ挙動について spans と preserved newlines を含む tests を追加する。

6. 最小限の documentation tests / examples を追加する。
   - `scan_raw`, `lex`, `disambiguate` の examples を追加する。
   - token span が scanner input への byte offset であることを示す。
   - parser-facing API が進化しても安定する小さな examples に留める。

7. public API stability policy を決める。
   - `TokenKind`, `RawTokenKind`, `LexDiagnosticCode`, `ImportPrescanDiagnosticCode`, `ScopeSkeletonDiagnosticCode`, `SourcePreprocessDiagnosticCode`, `LexicalEnvironmentError` などの public enum に `#[non_exhaustive]` を付けるか検討する。
   - 代替案として、`0.1` API では compatibility guarantee なしで breaking change を行う可能性があることを明記する。
   - downstream crates が依存する前に、現在 public な fields の一部を `pub(crate)` に絞るべきか review する。

8. source-text normalization policy を文書化する。
   - lexer は Unicode normalization を行わず、code-region identifiers/symbols は ASCII-only のままであることを明記する。
   - comments と documentation text は、後続の documentation/source-loading layer が warning を追加しない限り raw Unicode のまま保持する。

9. fuzz coverage を追加する。
   - arbitrary byte input または valid UTF-8 strings を対象にした `scan_raw` 用 `cargo-fuzz` target を追加する。
   - `preprocess_source_for_lexing` と `scan_raw` に arbitrary valid UTF-8 input を与える。
   - 見つかった failure は minimize し、corpus regression として commit する前に stable case として `tests/lexical` に昇格する。

10. performance benchmarking を追加する。
   - large `.miz`-like source に対する `scan_raw` throughput を benchmark する。
   - raw scanning、preprocessing、`SourceLineIndex` construction を分けて測定する。
   - module resolution、parser context、imported symbol loading から独立した benchmarks にする。

11. source-loading boundary における UTF-8 BOM handling policy を決める。
   - raw file input の先頭 UTF-8 BOM は受け入れ、`mizar-lexer` entry point が `&str` を受け取る前に source-loading 側で取り除く方針を優先する。
   - direct lexer helper calls は strict のままにする。`preprocess_source_for_lexing` や `scan_raw` に届いた `U+FEFF` は silently disappear させず、malformed source precondition として扱う。
   - BOM stripping 後の token span が loaded text offsets で測られること、および source map が original file byte offsets にどう対応するかを文書化する。
   - frontend/session source loader ができた段階で source-loading tests を追加する。それまでは lexer behavior は変更しない。

12. UTF-8 file loading を仕様化して test する。
   - invalid UTF-8 を lexer entry 前に reject し、lossy decode で `U+FFFD` にしない。
   - 先頭 UTF-8 BOM stripping を決めて test し、original-byte-offset source-map behavior も確認する。

13. newline normalization を仕様化して test する。
   - lexer entry 前の CRLF-to-LF behavior を定義する。
   - source map が normalized lexical/source text offsets を original file byte offsets に対応付けられることを確認する。

14. preprocess source-map tests を実装する。
   - ordinary comment removal、documentation comment retention、synthetic whitespace/newline segments、removed comments をまたぐ lexical ranges を cover する。
   - lexer/preprocessor helpers 由来の diagnostics を original source ranges に map できることを確認する。

15. user-facing column conversion は lexer 外に保つ。
    - Unicode scalar columns は source-map/session layer で test する。
    - LSP UTF-16 conversion は `mizar-lexer` ではなく LSP bridge で test する。

16. source path normalization は lexer 外で cover する。
    - `.`/`..`、symlinks、case policy、package-root escape attempts、platform-specific separators を source-loading/path layer で test する。

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

## Text-Processing Audit Notes

この first-pass audit は、text-processing で一般的に問題になりやすい点について、現在の lexer crate が解決しているか、別 layer に委譲しているか、policy/test が残っているかを記録する。これは review note であり、単独の implementation plan ではない。

| Topic | Current status | Owner / evidence | Follow-up |
|---|---|---|---|
| UTF-8 validation | design 上は委譲済み | `mizar-lexer` は `&str` を受け取る。file bytes の validation は lexer entry 前の source loading が担当する。`raw_lexer.md` Source Preconditions と architecture source loading step を参照。 | source-loader crate ができた段階で tests を実装する。 |
| UTF-8 BOM | deferred | 現在の lexer helpers は `U+FEFF` を malformed/non-ASCII として扱う。Priority task 6 に intended source-loading policy を記録済み。 | source-loading boundary で BOM stripping policy を決定し test する。 |
| replacement character `U+FFFD` | code region では概ね reject | `preprocess_source_for_lexing` は code-region の non-ASCII char を `NonAsciiCode` として報告する。comments は Unicode allowed。 | invalid UTF-8 を lossy decode して `U+FFFD` にしないことを source-loading policy として明記する。 |
| LF / CRLF / CR handling | 委譲 + guard 済み | source loading が platform newline を normalize する想定。lexer helper は `CarriageReturn` を報告し、raw scanner は `\r` を reject する。 | source loader と source-map tests で CRLF-to-LF mapping を cover する。 |
| missing final newline / empty file / trailing newlines | lexer level では covered | empty raw stream の test があり、`SourceLineIndex` は EOF を受け入れる。scanner は final newline を要求しない。 | parser/import prelude が final newline に依存する場合のみ corpus cases を追加する。 |
| byte offset vs character column | partially resolved | lexer spans は byte offsets。`SourceLineIndex` は zero-based byte columns を使い、non-UTF-8-boundary offsets を reject する。session source map は user-facing Unicode scalar columns を規定する。 | byte-column helper docs と implementation を同期し続ける。human-facing conversion は source-map layer が担当する。 |
| LSP UTF-16 columns | design 上は委譲済み | `raw_lexer.md` と `source_map.md` は LSP UTF-16 conversion を lexer 外に置いている。 | LSP bridge ができた段階で tests を追加する。 |
| preprocessed text vs original source spans | designed, not implemented here | `raw_lexer.md` は lexer spans が scanner input を指すと明記し、`source_map.md` は `PreprocessMap` ownership を lexer 外に定義している。 | comment removal と synthetic whitespace mapping の frontend/source-map tests を追加する。 |
| Unicode in code vs comments | partially covered | preprocessor は comments 内の Unicode を許し、code regions の non-ASCII を報告する。Greek comment text と code-region rejection の tests がある。 | zero-width chars、NBSP、BOM-in-code、full-width punctuation の diagnostics tests を追加する。 |
| Unicode normalization and confusables | lexer は normalize しない | identifier/numeral/user-symbol helpers は ASCII-only なので、non-ASCII code identifiers は normalize せず reject される。comments は raw Unicode のまま保持される。 | lexer が source text を normalize しないことを明記する。comments/doc text の suspicious Unicode warning は将来 source-loading/docs 側で検討する。 |
| Unicode whitespace | comments 外では strict reject | raw layout は space, tab, LF のみ。preprocessor は non-ASCII code chars を報告する。 | NBSP、zero-width space、vertical tab、form feed の diagnostics を fixtures で固定する。 |
| tab display width | delegated | lexer は byte spans のみを保持し、tab は layout として扱う。diagnostics/source map が display columns を担当する。 | diagnostic renderer で tab expansion policy を test する。 |
| comment stripping and newline preservation | partially covered | ordinary/doc/multi-line comment removal、trivia retention、newline preservation の tests がある。 | source loading 後の comments 内 CRLF、adjacent comments、EOF comments、token boundary 付近の comment removal の edge tests を追加する。 |
| nested multi-line comments | policy unclear | 現在の preprocessor は最初の `=::` を探す。nested-comment policy は lexer TODO では未文書化。 | nested block comments を unsupported とするか決め、文書化して tests を追加する。 |
| unterminated comments | covered | preprocessor は `UnterminatedMultiLineComment` を emit し、line structure を保持する。unit test あり。 | source-map implementation 後に frontend が diagnostic を original source に map することを確認する。 |
| annotation visibility | lexer boundary では covered | annotation syntax が parser ownership のため lexical text に残ることを tests が確認している。 | annotation argument validation は parser tests が担当する。 |
| string literal escapes and recovery | partially covered | helper と disambiguator tests は supported escapes、invalid escapes、malformed strings、context-required string positions を cover している。 | property/fuzz tests に string-required context と malformed recovery spans を含める。 |
| quote characters as user symbols | partially covered | design は strings を contextual と定め、disambiguator tests は string-required context と string outside context を cover している。 | grammar integration が変わるときに parser-context API tests を広げる。 |
| raw/final token span integrity | partially covered | 既存 tests は raw span contiguity と final token span preservation を cover する。Priority task 1 は property-style span tests を要求している。 | `lex` と `disambiguate` を横断する property-style final token span tests を実装する。 |
| error-recovery token spans | partially covered | disambiguator には recovery diagnostics/tokens があるが、広い span invariant tests は pending。 | Priority task 1 に recovery cases を含める。 |
| huge files / long lines / memory | not covered | `SourceLineIndex` は全 char boundaries を保持し、raw scanning は token lexemes を clone する。benchmark は未実施。 | benchmark task で large `.miz`-like input と long-line behavior を測定する。 |
| panic safety under arbitrary text | partially covered | scanner は `char_indices` を使う。fuzz task は pending。 | valid UTF-8 strings 用 fuzz target を追加し、regressions を corpus に昇格する。 |
| file/path text normalization | lexing 外で partially covered | `module_source_name_from_path` は separators を normalize し、identifier-shaped components を validate する。 | symlink、case、`.`/`..`、platform path edge cases は source-loading/path layer が担当する。 |
