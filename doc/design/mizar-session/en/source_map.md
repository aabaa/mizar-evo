# Module: source_map

> Canonical language: English. Japanese companion: [../ja/source_map.md](../ja/source_map.md).

## Purpose

This module defines source coordinate tables and range conversion for `mizar-session`.

It lets frontend, diagnostics, LSP, documentation, extraction, artifacts, and IR side tables agree on source ranges without sharing mutable source text or frontend internals. It covers raw source ranges, line/column conversion, preprocessing maps, generated spans, and degraded mappings for synthesized text.

## Public API

```rust
pub struct LineMap {
    pub source_id: SourceId,
    pub line_starts: Vec<ByteOffset>,
    pub text_hash: Hash,
}

pub struct SourceRange {
    pub source_id: SourceId,
    pub start: ByteOffset,
    pub end: ByteOffset,
}

pub struct LineColumn {
    pub line: u32,
    pub column: u32,
}

pub struct PreprocessMap {
    pub source_id: SourceId,
    pub lexical_text_hash: Hash,
    pub segments: Vec<PreprocessSegment>,
}

pub enum PreprocessSegment {
    Original {
        lexical: TextRange,
        source: SourceRange,
    },
    RemovedComment {
        source: SourceRange,
        kind: CommentKind,
    },
    SyntheticWhitespace {
        lexical: TextRange,
        anchor: SourceAnchor,
    },
}

pub enum SourceAnchor {
    Range(SourceRange),
    Point { source_id: SourceId, offset: ByteOffset },
    Generated { origin: GeneratedSpanOrigin },
}

pub trait SourceMapService {
    fn line_column(&self, range: SourceRange) -> Result<(LineColumn, LineColumn), SourceMapError>;
    fn source_range_for_lexical(&self, source_id: SourceId, lexical: TextRange) -> Result<MappedSourceRange, SourceMapError>;
    fn attach_generated_span(&self, origin: GeneratedSpanOrigin) -> SourceAnchor;
    fn validate_range(&self, range: SourceRange) -> Result<(), SourceMapError>;
}
```

Offsets are byte offsets into validated UTF-8 source text. User-facing columns use the Unicode scalar column rule defined by the frontend architecture and must be converted through `LineMap` rather than computed by consumers ad hoc.

## Dependencies

- Internal: `snapshot` for `SourceId` and source-version identity
- External: hashing, UTF-8 text utilities, LSP range types

This module is consumed by frontend phases, `mizar-ir` side tables, `mizar-diagnostics`, `mizar-lsp`, `mizar-artifact`, `mizar-doc`, and `mizar-extract`.

## Data Structures

### Line Map

`LineMap` records line starts for the exact source text represented by a `SourceVersion`.

It is immutable after construction. Consumers must validate that the `source_id` belongs to the snapshot they are reporting against before converting offsets to user-facing line/column values.

### Source Range

`SourceRange` is half-open: `start <= offset < end`.

Ranges must:

- reference one `SourceId`;
- use byte offsets aligned to UTF-8 scalar boundaries;
- remain inside the source text length;
- preserve zero-length ranges only for insertion points and synthesized anchors.

### Preprocess Map

`PreprocessMap` relates the lexical text consumed by the lexer to the original source.

Original segments map lexical ranges back to source ranges. Removed comment segments preserve ordinary and doc-comment locations even when comments are absent from lexical input. Synthetic whitespace segments represent text inserted to keep token separation after comment removal or recovery.

The frontend owns construction of this map. Later phases consume it to attach diagnostics and syntax nodes to original source locations.

### Generated Spans

Generated spans are used when a compiler-created item has no exact source range, such as:

- implicit obligations;
- inserted coercions or `qua` nodes;
- generated proof replay steps;
- documentation or extraction records derived from multiple inputs.

Generated spans must include an origin that points to the best available source anchor and a reason. Diagnostics may display generated spans as secondary information, but primary diagnostics should prefer original source ranges when available.

## Algorithm / Logic

### Line/Column Conversion

1. Validate the range against the `LineMap` source text length.
2. Locate start and end lines by binary searching `line_starts`.
3. Count Unicode scalar values from the line start to each offset.
4. Return one-based lines and zero-based columns for diagnostics/LSP adapters to format as required by their protocol.

LSP conversion must apply the protocol's UTF-16 position rules in the `mizar-lsp` bridge, not inside this module. This module exposes source-stable coordinates.

### Lexical-to-Source Mapping

1. Find all preprocess segments intersecting the lexical range.
2. If the range maps to one contiguous original source range, return that range.
3. If it spans removed or synthetic segments, return a composite mapping with primary and secondary anchors.
4. If no original source exists, return a generated anchor.

Composite mappings are allowed for diagnostics, documentation attachment, and explanation metadata. Cache keys and artifact hashes must use source hashes and stable ids, not serialized pretty forms of composite mappings.

### Source Map Retention

Source maps are retained with the owning snapshot while any snapshot lease, diagnostic index, LSP publication, or IR side table references them. They may be dropped after the owning snapshot is collected.

## Error Handling

`SourceMapError` includes:

- unknown source id;
- range outside source text;
- offset not aligned to a UTF-8 boundary;
- lexical range outside preprocessed text;
- missing preprocess segment;
- generated span without an origin reason.

Malformed user source is reported by frontend diagnostics. Source-map errors indicate compiler bugs or stale handles unless they originate from an explicitly stale LSP request.

## Tests

Key scenarios:

- line maps convert byte offsets to Unicode scalar columns;
- CRLF and LF inputs produce deterministic line starts according to source-loading rules;
- removed comments map to preserved comment source ranges;
- lexical ranges spanning comment removal produce composite mappings;
- synthetic whitespace does not become a primary user source range;
- generated anchors preserve their origin reason;
- invalid byte offsets and cross-source ranges are rejected;
- LSP UTF-16 conversion remains outside this module.

## Constraints and Assumptions

- Source maps are internal compiler data and are not published as a stable schema by this crate.
- Published artifacts may include projected source ranges, but not the full preprocessing map.
- Source range conversion must be deterministic and independent of scheduling order.
- The module must not read files directly; it works from source text and identity supplied by source loading and snapshot creation.
