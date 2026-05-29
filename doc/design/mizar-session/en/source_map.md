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

pub struct LoadingMap {
    pub source_id: SourceId,
    pub loaded_text_hash: Hash,
    pub origin: LoadingOrigin,
    pub segments: Vec<LoadingMapSegment>,
}

pub enum LoadingOrigin {
    DiskBytes { normalized_path: NormalizedPath },
    OpenBufferText { uri: DocumentUri, version: LspDocumentVersion },
    Generated { anchor: Option<SourceAnchor> },
}

pub enum LoadingMapSegment {
    Original {
        loaded: TextRange,
        original: TextRange,
    },
    RemovedLeadingBom {
        original: TextRange,
    },
    NormalizedNewline {
        loaded: TextRange,
        original: TextRange,
    },
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
    fn original_range_for_loaded(&self, source_id: SourceId, loaded: TextRange) -> Result<MappedSourceRange, SourceMapError>;
    fn source_range_for_lexical(&self, source_id: SourceId, lexical: TextRange) -> Result<MappedSourceRange, SourceMapError>;
    fn attach_generated_span(&self, origin: GeneratedSpanOrigin) -> SourceAnchor;
    fn validate_range(&self, range: SourceRange) -> Result<(), SourceMapError>;
}
```

Offsets are byte offsets into validated UTF-8 `LoadedSource.text` after source-loading normalization. User-facing columns use the Unicode scalar column rule defined by the frontend architecture and must be converted through `LineMap` rather than computed by consumers ad hoc.

## Dependencies

- Internal: `snapshot` for `SourceId` and source-version identity
- External: hashing, UTF-8 text utilities, LSP range types

This module is consumed by frontend phases, `mizar-ir` side tables, `mizar-diagnostics`, `mizar-lsp`, `mizar-artifact`, `mizar-doc`, and `mizar-extract`.

## Data Structures

### Line Map

`LineMap` records line starts for the exact source text represented by a `SourceVersion`.

It is immutable after construction. Consumers must validate that the `source_id` belongs to the snapshot they are reporting against before converting offsets to user-facing line/column values.

If disk source loading stripped a leading UTF-8 BOM, byte offset `0` in the `LineMap` is the first byte after that BOM in the original file. Raw-file byte positions are recovered through `LoadingMap`, not by changing `SourceRange` or lexer span coordinates.

### Source Range

`SourceRange` is half-open: `start <= offset < end`.

Ranges must:

- reference one `SourceId`;
- use byte offsets aligned to UTF-8 scalar boundaries;
- remain inside the source text length;
- preserve zero-length ranges only for insertion points and synthesized anchors.

### Loading Map

`LoadingMap` relates normalized `LoadedSource.text` to the source-loading input before BOM stripping or newline normalization. For disk sources, `original` ranges are byte offsets into the original file bytes after UTF-8 validation. For open buffers, `original` ranges are byte offsets into the editor-provided UTF-8 text; the LSP bridge then converts those byte offsets to protocol UTF-16 positions. Generated sources use anchors when no original text range exists.

When a leading UTF-8 BOM is stripped, the map records a `RemovedLeadingBom` segment for original byte range `[0, 3)` and the first `Original` loaded segment starts at loaded offset `0` and original byte offset `3`. Source loading may omit `LoadingMap` only when the loaded text is offset-identical to the source-loading input.

### Preprocess Map

`PreprocessMap` relates the lexical text consumed by the lexer to the original source.

Original segments map lexical ranges back to source ranges. Removed comment segments preserve ordinary and doc-comment locations even when comments are absent from lexical input. Synthetic whitespace segments represent text inserted to keep token separation after comment removal or recovery.

The frontend owns snapshot retention and service access for this map. It may reuse or mirror the lightweight preprocess map produced by the lexer helper when constructing the retained session `PreprocessMap`. Later phases consume it to attach diagnostics and syntax nodes to original source locations.

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
4. Return one-based lines and one-based columns for diagnostics, artifacts, and CLI formatting.

LSP conversion must apply the protocol's UTF-16 position rules in the `mizar-lsp` bridge, not inside this module. This module exposes source-stable coordinates.

### Loaded-to-Original Mapping

1. Use the `LoadingMap` for the `SourceId` when one exists.
2. If there is no `LoadingMap`, treat loaded-text offsets as identity offsets into the source-loading input.
3. If a loaded range crosses removed or normalized segments, return a composite mapping with primary loaded text and secondary original anchors.
4. For open buffers, return editor-text byte offsets; the LSP bridge performs the final UTF-16 conversion.

### Lexical-to-Source Mapping

1. Find all preprocess segments intersecting the lexical range.
2. If the range maps to one contiguous loaded source range, map that range through loaded-to-original mapping when diagnostics need source-loading input coordinates.
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
- missing loading map segment;
- missing preprocess segment;
- generated span without an origin reason.

Malformed user source is reported by frontend diagnostics. Source-map errors indicate compiler bugs or stale handles unless they originate from an explicitly stale LSP request.

## Tests

Key scenarios:

- line maps convert byte offsets to Unicode scalar columns;
- line maps for BOM-prefixed disk files start at loaded-text offset `0` after the stripped BOM;
- `LoadingMap` relates loaded-text offset `0` to original file byte offset `3` when a leading BOM was stripped;
- `LoadingMap` for open buffers relates loaded-text offsets back to editor-provided text byte offsets before LSP UTF-16 conversion;
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
