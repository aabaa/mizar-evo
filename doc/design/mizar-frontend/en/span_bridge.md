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
spans; this module converts them into source-id-scoped `SourceRange` values and
registers the per-source maps with the `mizar-session` `SourceMapService` so
diagnostics and LSP positions resolve consistently. It performs no I/O,
tokenization, parsing, or semantic work.

## Public API

```rust
pub struct SpanBridge { /* registered per-source maps */ }

impl SpanBridge {
    pub fn new(service: Arc<dyn SourceMapService>) -> Self;

    pub fn register_source(
        &self,
        source_id: SourceId,
        line_map: LineMap,
        loading_map: Option<LoadingMap>,
    );

    pub fn register_preprocess_map(
        &self,
        source_id: SourceId,
        preprocess_map: PreprocessMap,
    );

    pub fn loaded_span(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<SourceRange, SpanBridgeError>;

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
and `SourceMapService` are owned by `mizar-session`; `span_bridge` adapts
`mizar-lexer` byte spans onto them. `loaded_span` maps a span in loaded text
(Step 1 coordinates); `lexical_span` maps a span in comment-stripped lexical
text (Step 2+ coordinates) and returns a `MappedSourceRange` with the primary
range plus secondary anchors for spans that cross a removed comment.

## Dependencies

- Internal: consumed by `source`, `preprocess`, `lexing`, and `parsing`; it is
  the lowest-level frontend coordination module.
- External: `mizar-session` (`SourceMapService`, `SourceRange`,
  `MappedSourceRange`, `LineMap`, `LoadingMap`, `PreprocessMap`, `SourceId`),
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

`loaded_span` uses the loading map and line map only. `lexical_span` additionally
applies the preprocess map, returning composite adjacent anchors at zero-length
boundaries (for example a lexical range whose interior was a removed comment).
The bridge derives the session-side `LoadingMap` / `PreprocessMap` from the
lexer's `SourceLoadingMap` / `SourcePreprocessMap` (or reuses the session
`LoadingMap` already attached to the `SourceUnit`), so there is exactly one
canonical map per `SourceId`.

### Registry

The bridge holds a per-`SourceId` registry of the maps registered during phases
1-2. Registration is idempotent for a given `SourceId`; re-registering with a
different map for an already-registered source is a programming error surfaced as
`SpanBridgeError`.

## Algorithm / Logic

### Map a loaded-text span (Step 1 diagnostics)

1. Validate that `span` lies within the loaded text for `source_id`.
2. Map start/end loaded offsets to original input offsets through the
   `LoadingMap` (identity when no transform changed offsets).
3. Build a `SourceRange` scoped by `source_id` and return it.

### Map a lexical-text span (Step 2+ tokens and diagnostics)

1. Validate that `span` lies within the lexical text for `source_id`.
2. Map the lexical offsets to loaded offsets through the `PreprocessMap`,
   producing primary plus secondary anchors when the span crosses a removed
   comment.
3. Continue through the `LoadingMap` to original input offsets.
4. Return a `MappedSourceRange` (primary `SourceRange`, secondary anchors, and
   the loaded-to-original `original_input` bytes) using the session
   `SourceMapService`.

All conversions delegate the arithmetic to `mizar-session`; this module only
selects the right map layer and the right `SourceId`.

## Error Handling

`SpanBridgeError` wraps the failures the session `SourceMapService` reports —
unknown source id, range outside source/lexical text, offset not on a UTF-8
boundary, missing loading-map or preprocess-map segment, line/column overflow —
plus a frontend-local "source not registered" / "conflicting map registration"
case. A bridge failure is an internal invariant violation (a span that does not
belong to its declared source), not a user diagnostic; orchestration treats it
as a bug surface rather than a recoverable lexical/syntax diagnostic.

## Tests

Key scenarios:

- a loaded-text span over BOM-stripped text maps to the correct original byte
  offsets through the loading map;
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
- All coordinate arithmetic is delegated to the `mizar-session`
  `SourceMapService`; the bridge never reimplements offset math.
- Bridge failures are internal invariant violations, not user-facing diagnostics.
