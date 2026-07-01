# CLI Diagnostic Rendering

> 正本は英語です。英語版:
> [../en/render.md](../en/render.md)。

## 目的

この文書は `mizar-diagnostics` が所有する deterministic CLI rendering を定義する。
rendering は immutable `DiagnosticRecord` と caller-provided source context を
human-facing terminal text へ project する。これは diagnostic identity でも aggregation
でも LSP protocol conversion でもない。

`DiagnosticCode` と structured record field が tools の authority のままである。message
text、localized text、terminal color、excerpt、underline layout は code identity を変えずに
進化してよい。

## Scope

CLI rendering が所有するもの:

- 1 つ以上の current `DiagnosticRecord` の deterministic text layout。
- severity/code/semantic-name header。
- caller-provided path と line-map data から derived される source-location header。
- source excerpt、primary underline、secondary underline、label。
- record note と後続 structured fix data から projection される note/help line。
- byte-stable output のために disable できる optional color/style token。
- rendered text の golden-test snapshot。

CLI rendering が所有しないもの:

- `DiagnosticCode` meaning の allocation、retirement、interpretation。
- deduplication、sorting、stale filtering、handle assignment。
- record 作成や record severity/category/freshness の変更。
- LSP UTF-16 range、JSON-RPC payload、diagnostics publication、code action。
- proof acceptance、trusted status、kernel acceptance、phase success、build exit status。
- artifact write、cache mutation、source loading、driver session orchestration。

## Inputs

rendering は record と source context を consume する。

```rust
struct DiagnosticRenderInput<'a> {
    records: &'a [DiagnosticRecord],
    source_context: &'a dyn DiagnosticSourceContext,
    options: RenderOptions,
}

trait DiagnosticSourceContext {
    fn path_for(&self, source: SourceId) -> Option<&str>;
    fn source_key_for(&self, source: SourceId) -> String;
    fn line_text(&self, source: SourceId, line: u32) -> Option<&str>;
    fn line_column(&self, range: SourceRange) -> Option<LineColumnRange>;
}
```

具体的な task 11 API は trait ではなく struct を使ってよい。ただし ownership は同じで
なければならない。source text、path normalization、file length validation、line-map
construction は caller または `mizar-session` から来る。rendering は supplied view を使うだけ
である。

`source_key_for` は `DiagnosticSourceKey` と同じ deterministic rule を使わなければならない。
利用できる場合は published-schema string を使い、そうでなければ `Debug` rendering を使うか、
caller-owned の同等に stable な key を提供する。この key は fallback/debug display key であり、
durable artifact path ではない。

source context が欠けている場合でも、rendering は code、message、semantic name、source key、
byte range を使って deterministic diagnostic を生成しなければならない。line/column data を
捏造してはならない。

## Header Layout

canonical plain-text header は次の形式である。

```text
severity[CODE]: message (semantic.name)
```

`severity` は record descriptor severity から derived される。`CODE` は stable
`DiagnosticCode` である。`message` は human-facing record text であり identity ではない。
`semantic.name` は現在の registry semantic name であり、registry compatibility rule の下でのみ
rename されてよい。

rendered ordering は input record order に従う。`BuildDiagnosticIndex` の場合、caller は
`index.records()` を渡す。これにより aggregation が publication ordering の single source の
ままである。複数 diagnostic は diagnostic block 間に exactly one blank line を入れて render
し、最後の diagnostic の後に余分な blank line は入れない。

## Source Blocks

record の単一 primary span について、rendering は次を emit する。

```text
  --> path/to/file.miz:line:column
   |
LL | source text
   | ^^^^^ label
```

Rules:

- path は `DiagnosticSourceContext` から提供され、通常は workspace-relative である。
  rendering は path の normalize や resolve を行わない。
- line と column は 1-based であり supplied line map から derived される。
- column は CLI display 用の Unicode scalar count であり、byte でも LSP UTF-16 code unit でも
  ない。
- primary span は `^` underline を使う。secondary span は `-` underline を使う。
- zero-width span は insertion point の 1 caret として render し、必要なら `eof` または
  `insertion_point` label を含めてよい。
- multiline span は、span 全体が configured context limit を超える場合 first/last line と
  ellipsis separator を表示する。
- source text が欠けている場合、panic ではなく `source <key>:<start>..<end>` のような
  deterministic fallback を render する。

rendering は record order で primary block の後に secondary span を group してよい。secondary
span を primary status に昇格してはならない。

## Notes, Fixes, And Explanations

`DiagnosticNote` は span block の後に render される。

```text
   = note: message
   = help: message
```

`DiagnosticNoteKind::Help` は `help` を使う。`Note`、`Cause`、`Related` はそれぞれの kind と
一致する stable label を使う。note text は human-facing であり identity ではない。
note が optional source span を持つ場合、rendering は note text の直前にその span の
secondary-style source block を emit する。underline は `-` を使い、span が既に label を持つ
場合を除き note message を label とする。rendering は note source context を silent に drop
してはならない。

task 11 は、task 13 が structured fix payload を定義するまで、既存 `FixSuggestionRef` を
opaque で bounded な help reference としてのみ render してよい。text edit、code action、
automatic application behavior を invent してはならない。

task 11 は `ExplanationRef` を bounded `explain:` reference または documentation hint として
render してよいが、large trace を resolve してはならない。explanation storage と lazy
resolution は task 15 の behavior である。

## Styling And Determinism

rendering option は少なくとも次を support しなければならない。

- tests と non-terminal consumer のための ANSI color なし plain output。
- terminal output のための optional ANSI styling。
- multiline span 用 context line limit。
- caller が supplied する stable path display mode。

plain output は byte-stable でなければならない。LF line ending、trailing whitespace なし、
localized field name なし、memory address なし、map iteration order なし、debug output で明示
された `DiagnosticHandle` 以外の process-local id なし。

ANSI styling は presentation layer にすぎない。tests はまず plain mode を cover し、必要なら
style token placement を別途 cover してよい。

## Boundary Rules

- rendering は record を read するだけで、mutate や reclassify はしない。
- rendering は header に `DiagnosticCode` を含めなければならず、tool behavior を message text
  に key してはならない。
- rendering は stale diagnostic を current output として publish できない。caller が
  non-current view のために stale/historical record を明示的に渡した場合だけ stale marker を
  表示してよい。
- rendering は process exit code、phase status、proof acceptance、kernel acceptance を決定
  できない。
- rendering は LSP diagnostics や code action を作れない。LSP conversion は `mizar-lsp` が
  所有する。
