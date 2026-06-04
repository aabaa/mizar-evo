# Module: span_bridge

> Canonical language: English. Japanese companion: [../ja/span_bridge.md](../ja/span_bridge.md).

Status: planned.

## Purpose

This module owns the coordinate bridge between `mizar-lexer` byte spans and
`mizar-session` `SourceRange` values. It is the single place where the frontend
resolves the top-level open decision recorded in
[../../todo.md](../../todo.md) "Lexer span bridging": `mizar-lexer` stays
decoupled and keeps its own byte-offset `SourceSpan`, while the frontend maps
those spans onto session source coordinates.

Every later module (preprocessing, lexing, parsing) produces lexer-relative byte
spans; this module converts them into source-id-scoped `SourceRange` /
`MappedSourceRange` values and owns the retained session source-map service for
the maps registered by the frontend run. It performs no I/O, tokenization,
parsing, or semantic work.

## Public API

```rust
pub struct SpanBridge { /* registered per-source maps */ }

impl SpanBridge {
    pub fn new() -> Self;

    pub fn source_map_service(&self) -> &dyn SourceMapService;

    pub fn register_source(
        &mut self,
        source_id: SourceId,
        line_map: LineMap,
        loading_map: Option<LoadingMap>,
    ) -> Result<(), SpanBridgeError>;

    pub fn register_preprocess_map(
        &mut self,
        source_id: SourceId,
        preprocess_map: PreprocessMap,
    ) -> Result<(), SpanBridgeError>;

    pub fn loaded_span(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<SourceRange, SpanBridgeError>;

    pub fn loaded_mapping(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<MappedSourceRange, SpanBridgeError>;

    pub fn lexical_span(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<MappedSourceRange, SpanBridgeError>;
}

pub struct LexerByteSpan {
    pub start: usize,
    pub end: usize,
}
```

`SourceRange`, `MappedSourceRange`, `LineMap`, `LoadingMap`, `PreprocessMap`,
`RetainedSourceMapService`, and `SourceMapService` are owned by
`mizar-session`; `span_bridge` adapts
`mizar-lexer` byte spans onto them. `loaded_span` maps a span in loaded text
(Step 1 coordinates) into a validated loaded-text `SourceRange`. It does not
rewrite the `SourceRange` into raw file/editor byte offsets. Callers that need
source-loading input byte offsets use `loaded_mapping`, whose `original_input`
field is populated when a `LoadingMap` exists. `lexical_span` maps a span in
comment-stripped lexical text (Step 2+ coordinates) and returns a
`MappedSourceRange` with the primary loaded-source range plus secondary anchors
for spans that cross a removed comment or synthetic whitespace.

## Dependencies

- Internal: consumed by `source`, `preprocess`, `lexing`, and `parsing`; it is
  the lowest-level frontend coordination module.
- External: `mizar-session` (`RetainedSourceMapService`, `SourceMapService`,
  `SourceRange`, `MappedSourceRange`, `LineMap`, `LoadingMap`,
  `PreprocessMap`, `SourceId`),
  `mizar-lexer` (byte-offset span types from `mizar_lexer::source`, converted at
  this boundary only).

## Data Structures

### Translation Stages

The bridge composes three `mizar-session` map layers per `SourceId`:

| Stage | From | To | Owner |
|---|---|---|---|
| lexical → loaded | lexical-text byte offset | loaded-text byte offset | `PreprocessMap` |
| loaded → original | loaded-text byte offset | original input byte offset | `LoadingMap` |
| offset → line/column | loaded-text byte offset | one-based Unicode column | `LineMap` |

`loaded_span` validates the byte range against the registered line map and
returns a loaded-text `SourceRange`. `loaded_mapping` additionally composes the
registered `LoadingMap` when present and returns a `MappedSourceRange` whose
primary range is still in loaded-text coordinates and whose `original_input`
contains the source-loading input bytes. `lexical_span` applies the preprocess
map, returning composite adjacent anchors at zero-length boundaries (for example
a lexical range whose interior was a removed comment). The bridge derives the
session-side `PreprocessMap` from the lexer's `SourcePreprocessMap` and reuses
the session `LoadingMap` attached to the `SourceUnit`; there is exactly one
canonical map per `SourceId`.

### Registry

The bridge holds a per-`SourceId` registry of the maps registered during phases
1-2 and inserts those maps into its owned `RetainedSourceMapService`.
Registration is idempotent for a given `SourceId`; re-registering with a
different map for an already-registered source is a programming error surfaced as
`SpanBridgeError`. The session retained service itself overwrites maps on
`insert_*`, so conflict detection is a frontend bridge responsibility before the
insert happens.

## Algorithm / Logic

### Map a loaded-text span (Step 1 diagnostics)

1. Validate that `span` lies within the loaded text for `source_id`.
2. Build a `SourceRange` scoped by `source_id` in loaded-text coordinates.
3. Validate the range through the retained session source-map service and return
   it.

`SourceRange` never stores raw file/editor offsets. When the caller needs those
bytes for LSP or source-loading diagnostics, `loaded_mapping` uses the retained
`LoadingMap` and returns them in `MappedSourceRange.original_input`. If no
loading map was emitted because source loading was offset-identical,
`loaded_mapping` returns an exact mapping with `original_input = None` rather
than inventing a second identity map.

### Map a lexical-text span (Step 2+ tokens and diagnostics)

1. Validate that `span` lies within the lexical text for `source_id`.
2. Map the lexical offsets to loaded offsets through the `PreprocessMap`,
   producing primary plus secondary anchors when the span crosses a removed
   comment.
3. Return a `MappedSourceRange` using the retained session `SourceMapService`.
   The primary `SourceRange` and secondary anchors are loaded-source
   coordinates. Source-loading input bytes remain an optional view obtained from
   `loaded_mapping` / the loading map when consumers need them.

All conversions delegate the arithmetic to `mizar-session`; this module only
selects the right map layer, enforces per-`SourceId` registration invariants, and
chooses whether the caller needs a plain loaded `SourceRange` or a richer
`MappedSourceRange`.

## Error Handling

`SpanBridgeError` wraps the failures the retained session `SourceMapService`
reports —
unknown source id, range outside source/lexical text, offset not on a UTF-8
boundary, missing preprocess-map segment, line/column overflow — plus
frontend-local "source not registered" / "conflicting map registration" cases.
A bridge failure is an internal invariant violation (a span that does not belong
to its declared source), not a user diagnostic; orchestration treats it as a bug
surface rather than a recoverable lexical/syntax diagnostic.

## Tests

Key scenarios:

- a loaded-text span over BOM-stripped text remains a valid loaded-source
  `SourceRange`, and `loaded_mapping` reports the correct original byte offsets
  through the loading map;
- a lexical-text span maps through both preprocess and loading maps to the
  expected original `SourceRange`;
- a lexical span that crosses a removed comment yields a primary range plus
  secondary anchors;
- an offset not on a UTF-8 boundary is rejected rather than silently truncated;
- a span outside the registered text length is rejected with the session error;
- registering two different maps for the same `SourceId` is reported as a
  conflicting registration.

## Constraints and Assumptions

- `mizar-lexer` stays decoupled from `mizar-session`; this module is the only
  place that converts lexer byte spans into session `SourceRange` values.
- There is exactly one canonical line/loading/preprocess map per `SourceId`.
- All coordinate arithmetic is delegated to `mizar-session`; the bridge never
  reimplements offset math beyond constructing checked `TextRange` /
  `SourceRange` request objects and detecting duplicate registration.
- Bridge failures are internal invariant violations, not user-facing diagnostics.
