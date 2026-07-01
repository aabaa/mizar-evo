# CLI Diagnostic Rendering

> Canonical language: English. Japanese companion:
> [../ja/render.md](../ja/render.md).

## Purpose

This document specifies deterministic CLI rendering owned by
`mizar-diagnostics`. Rendering projects immutable `DiagnosticRecord` values and
caller-provided source context into human-facing terminal text. It is not
diagnostic identity, not aggregation, and not an LSP protocol conversion.

`DiagnosticCode` and structured record fields remain the authority for tools.
Message text, localized text, terminal colors, excerpts, and underline layout
may evolve without changing code identity.

## Scope

CLI rendering owns:

- deterministic text layout for one or more current `DiagnosticRecord` values;
- severity/code/semantic-name headers;
- source-location headers derived from caller-provided path and line-map data;
- source excerpts, primary underlines, secondary underlines, and labels;
- note/help lines projected from record notes and later structured fix data;
- optional color/style tokens that can be disabled for byte-stable output;
- golden-test snapshots for rendered text.

CLI rendering does not own:

- allocating, retiring, or interpreting `DiagnosticCode` meanings;
- deduplication, sorting, stale filtering, or handle assignment;
- creating records or changing record severity/category/freshness;
- LSP UTF-16 ranges, JSON-RPC payloads, diagnostics publication, or code
  actions;
- proof acceptance, trusted status, kernel acceptance, phase success, or build
  exit status;
- artifact writes, cache mutation, source loading, or driver session
  orchestration.

## Inputs

Rendering consumes records and source context:

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

The concrete task 11 API may use structs instead of a trait, but the ownership
must stay the same: source text, path normalization, file length validation, and
line-map construction come from the caller or `mizar-session`. Rendering only
uses the supplied view.

`source_key_for` must use the same deterministic rule as
`DiagnosticSourceKey`: use the published-schema string when available and
otherwise use `Debug` rendering, or provide an equally stable caller-owned key.
The key is a fallback/debug display key, not a durable artifact path.

If source context is missing, rendering must still produce a deterministic
diagnostic using the code, message, semantic name, source key, and byte range.
It must not invent line/column data.

## Header Layout

The canonical plain-text header is:

```text
severity[CODE]: message (semantic.name)
```

`severity` is derived from the record descriptor severity. `CODE` is the stable
`DiagnosticCode`. `message` is human-facing record text and is never identity.
`semantic.name` is the current registry semantic name and may be renamed only
under registry compatibility rules.

Rendered ordering follows the input record order. For a `BuildDiagnosticIndex`,
callers pass `index.records()` so aggregation remains the single source of
publication ordering. Multiple diagnostics render with exactly one blank line
between diagnostic blocks and no extra blank line after the final diagnostic.

## Source Blocks

For the record's single primary span, rendering emits:

```text
  --> path/to/file.miz:line:column
   |
LL | source text
   | ^^^^^ label
```

Rules:

- Paths are provided by `DiagnosticSourceContext` and are normally
  workspace-relative. Rendering does not normalize or resolve paths.
- Lines and columns are 1-based and derived from the supplied line map.
- Columns count Unicode scalar values for CLI display, not bytes and not LSP
  UTF-16 code units.
- The primary span uses `^` underlines. Secondary spans use `-` underlines.
- Zero-width spans render as a single caret at the insertion point and may
  include an `eof` or `insertion_point` label when useful.
- Multiline spans show the first and last lines with an ellipsis separator when
  the full span would exceed the configured context limit.
- Missing source text renders a deterministic fallback such as
  `source <key>:<start>..<end>` rather than panicking.

Rendering may group secondary spans after the primary block in record order.
Secondary spans must not be promoted to primary status.

## Notes, Fixes, And Explanations

`DiagnosticNote` values render after span blocks:

```text
   = note: message
   = help: message
```

`DiagnosticNoteKind::Help` uses `help`; `Note`, `Cause`, and `Related` use
stable labels matching their kind. Note text is human-facing and not identity.
When a note carries an optional source span, rendering emits a secondary-style
source block for that span immediately before the note text, using `-`
underlines and the note message as the label unless the span already has a
label. Rendering must not drop note source context silently.

Task 11 may render existing `FixSuggestionRef` values only as opaque, bounded
help references until task 13 defines structured fix payloads. It must not
invent text edits, code actions, or automatic application behavior.

Task 11 may render an `ExplanationRef` as a bounded `explain:` reference or
documentation hint, but it must not resolve large traces. Explanation storage
and lazy resolution are task 15 behavior.

## Styling And Determinism

Rendering options must support at least:

- plain output with no ANSI color for tests and non-terminal consumers;
- optional ANSI styling for terminal output;
- context line limit for multiline spans;
- stable path display through the caller-supplied `DiagnosticSourceContext`.

Plain output must be byte-stable: LF line endings, no trailing whitespace, no
localized field names, no memory addresses, no map iteration order, and no
process-local ids except `DiagnosticHandle` values explicitly requested for
debug output.

ANSI styling is a presentation layer only. Tests must cover the plain mode first
and may separately cover style token placement.

## Boundary Rules

- Rendering reads records; it does not mutate or reclassify them.
- Rendering must include `DiagnosticCode` in the header and must not key any
  tool behavior on message text.
- Rendering cannot publish stale diagnostics as current output. It may display
  a stale marker only when the caller explicitly passes stale/historical records
  for a non-current view.
- Rendering cannot decide process exit code, phase status, proof acceptance, or
  kernel acceptance.
- Rendering cannot create LSP diagnostics or code actions. LSP conversion is
  owned by `mizar-lsp`.
