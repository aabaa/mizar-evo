# Module: source_map

> Canonical language: English. Japanese companion: [../ja/source_map.md](../ja/source_map.md).

## Purpose

This module defines source coordinate tables and range conversion for `mizar-session`.

It lets frontend, diagnostics, LSP, documentation, extraction, artifacts, and IR side tables agree on source ranges without sharing mutable source text or frontend internals. It covers raw source ranges, line/column conversion, preprocessing maps, generated spans, and degraded mappings for synthesized text.

## Public API

```rust
pub type DocumentUri = String;
pub type LspDocumentVersion = i64;

pub struct LineMap {
    /* private fields */
}

impl LineMap {
    pub fn new(source_id: SourceId, text: &str) -> Self;
    pub fn with_source(source_id: SourceId, text: &str) -> Self;
    pub fn source(&self) -> &str;
    pub fn source_id(&self) -> SourceId;
    pub fn text_hash(&self) -> Hash;
    pub fn line_starts(&self) -> &[usize];
    pub fn line_column_for_source(
        &self,
        source_id: SourceId,
        offset: usize,
    ) -> Result<LineColumn, SourceMapError>;
    pub fn line_column_range(&self, range: SourceRange) -> Result<LineColumnRange, SourceMapError>;
    pub fn validate_range(&self, range: SourceRange) -> Result<(), SourceMapError>;
}

pub struct SourceRange {
    pub source_id: SourceId,
    pub start: usize,
    pub end: usize,
}

pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub const fn new(start: usize, end: usize) -> Self;
    pub const fn try_new(start: usize, end: usize) -> Option<Self>;
    pub const fn len(self) -> usize;
    pub const fn is_empty(self) -> bool;
}

pub struct LineColumn {
    pub line: u32,
    pub column: u32,
}

pub struct LineColumnRange {
    pub start: LineColumn,
    pub end: LineColumn,
}

#[non_exhaustive]
pub enum SourceMapError {
    UnknownSourceId { source_id: SourceId },
    ReversedRange,
    RangeOutsideSourceText { range: SourceRange, source_len: usize },
    OffsetNotUtf8Boundary { source_id: SourceId, offset: usize },
    LineColumnOverflow,
    RangeOutsideLoadedText { source_id: SourceId, range: TextRange, loaded_len: usize },
    MissingLoadingMapSegment { source_id: SourceId, range: TextRange },
    RangeOutsideLexicalText { source_id: SourceId, range: TextRange, lexical_len: usize },
    MissingPreprocessSegment { source_id: SourceId, range: TextRange },
    GeneratedSpanWithoutOriginReason,
}

pub struct LoadingMap {
    pub source_id: SourceId,
    pub loaded_text_hash: Hash,
    pub loaded_text_len: usize,
    pub origin: LoadingOrigin,
    pub segments: Vec<LoadingMapSegment>,
}

impl LoadingMap {
    pub fn new(
        source_id: SourceId,
        loaded_text: &str,
        origin: LoadingOrigin,
        segments: Vec<LoadingMapSegment>,
    ) -> Self;
    pub fn identity(source_id: SourceId, loaded_text: &str, origin: LoadingOrigin) -> Self;
    pub fn source_id(&self) -> SourceId;
    pub fn loaded_text_hash(&self) -> Hash;
    pub fn loaded_len(&self) -> usize;
    pub fn original_offset_for_loaded(
        &self,
        source_id: SourceId,
        offset: usize,
    ) -> Result<usize, SourceMapError>;
    pub fn original_range_for_loaded(
        &self,
        source_id: SourceId,
        loaded: TextRange,
    ) -> Result<LoadedToOriginalRange, SourceMapError>;
}

#[non_exhaustive]
pub enum LoadingOrigin {
    DiskBytes { normalized_path: NormalizedPath },
    OpenBufferText { uri: DocumentUri, version: LspDocumentVersion },
    Generated,
}

#[non_exhaustive]
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

pub struct LoadedToOriginalRange {
    pub original: TextRange,
    pub kind: LoadedToOriginalRangeKind,
}

pub enum LoadedToOriginalRangeKind {
    Exact,
    Degraded,
}

pub struct PreprocessMap {
    pub source_id: SourceId,
    pub lexical_text_hash: Hash,
    pub lexical_text_len: usize,
    pub segments: Vec<PreprocessSegment>,
}

impl PreprocessMap {
    pub fn new(
        source_id: SourceId,
        lexical_text: &str,
        segments: Vec<PreprocessSegment>,
    ) -> Self;
    pub fn identity(source_id: SourceId, lexical_text: &str) -> Self;
    pub fn source_id(&self) -> SourceId;
    pub fn lexical_text_hash(&self) -> Hash;
    pub fn lexical_len(&self) -> usize;
    pub fn source_anchors_for_lexical_offset(
        &self,
        source_id: SourceId,
        offset: usize,
    ) -> Result<Vec<SourceAnchor>, SourceMapError>;
    pub fn source_range_for_lexical(
        &self,
        source_id: SourceId,
        lexical: TextRange,
    ) -> Result<LexicalSourceMapping, SourceMapError>;
}

#[non_exhaustive]
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

#[non_exhaustive]
pub enum SourceAnchor {
    Range(SourceRange),
    Point { source_id: SourceId, offset: usize },
    Generated(GeneratedSpanOrigin),
}

pub struct GeneratedSpanOrigin {
    /* private fields */
}

impl GeneratedSpanOrigin {
    pub fn new(anchor: GeneratedSpanAnchor, reason: impl Into<String>) -> Result<Self, SourceMapError>;
    pub fn anchor(&self) -> GeneratedSpanAnchor;
    pub fn reason(&self) -> &str;
}

#[non_exhaustive]
pub enum GeneratedSpanAnchor {
    Range(SourceRange),
    Point { source_id: SourceId, offset: usize },
}

#[non_exhaustive]
pub enum CommentKind {
    SingleLine,
    MultiLine,
    Documentation,
}

pub struct LexicalSourceMapping {
    pub primary: Option<SourceRange>,
    pub anchors: Vec<SourceAnchor>,
    pub kind: LexicalSourceMappingKind,
}

pub enum LexicalSourceMappingKind {
    Exact,
    Composite,
    Degraded,
}

pub struct MappedSourceRange {
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub original_input: Option<TextRange>,
    pub kind: MappedSourceRangeKind,
}

pub enum MappedSourceRangeKind {
    Exact,
    Composite,
    Degraded,
}

pub trait SourceMapService {
    fn line_column(&self, range: SourceRange) -> Result<(LineColumn, LineColumn), SourceMapError>;
    fn original_range_for_loaded(&self, source_id: SourceId, loaded: TextRange) -> Result<MappedSourceRange, SourceMapError>;
    fn source_range_for_lexical(&self, source_id: SourceId, lexical: TextRange) -> Result<MappedSourceRange, SourceMapError>;
    fn attach_generated_span(&self, origin: GeneratedSpanOrigin) -> Result<SourceAnchor, SourceMapError>;
    fn validate_range(&self, range: SourceRange) -> Result<(), SourceMapError>;
}

pub struct RetainedSourceMapService {
    /* private fields */
}

impl RetainedSourceMapService {
    pub fn new() -> Self;
    pub fn insert_line_map(&mut self, line_map: LineMap);
    pub fn insert_loading_map(&mut self, loading_map: LoadingMap);
    pub fn insert_preprocess_map(&mut self, preprocess_map: PreprocessMap);
    pub fn with_line_map(self, line_map: LineMap) -> Self;
    pub fn with_loading_map(self, loading_map: LoadingMap) -> Self;
    pub fn with_preprocess_map(self, preprocess_map: PreprocessMap) -> Self;
}
```

Offsets are byte offsets into validated UTF-8 `LoadedSource.text`. For disk and open-buffer inputs this is the text after source-loading normalization. For generated inputs this is the accepted generated text byte-for-byte, because generated loading does not strip a leading `U+FEFF` or normalize CRLF pairs. User-facing columns use the Unicode scalar column rule defined by the frontend architecture and must be converted through `LineMap` rather than computed by consumers ad hoc.

`DocumentUri` and `LspDocumentVersion` are intentionally public aliases at this layer because source loading, snapshot origins, retention owners, and LSP range mapping need a shared boundary type without depending on an LSP crate. `TextRange` convenience helpers are public so callers can construct loaded-text and lexical-text ranges explicitly: `TextRange::new` asserts `start <= end`, `TextRange::try_new` returns `None` for reversed bounds, and `len`/`is_empty` assume the range invariant.

`LineColumn` intentionally uses `u32` rather than `usize`. These values are presentation and protocol-adjacent coordinates, not raw memory indexes, and keeping them bounded matches diagnostics, artifact metadata, and LSP adapters. If a loaded source has more than `u32::MAX` user-facing lines or a line with more than `u32::MAX` Unicode scalar columns, `LineMap` returns `SourceMapError::LineColumnOverflow` instead of saturating, wrapping, or silently narrowing. LSP positions remain `u32`; the `mizar-lsp` bridge performs its own explicit checked narrowing for UTF-16 columns.

## Dependencies

- Internal: `snapshot` for `SourceId` and source-version identity
- External: hashing, UTF-8 text utilities, LSP range types

This module is consumed by frontend phases, `mizar-ir` side tables, `mizar-diagnostics`, `mizar-lsp`, `mizar-artifact`, `mizar-doc`, and `mizar-extract`.

## Data Structures

### Line Map

`LineMap` records source identity, text hash, and line starts for the exact source text represented by a `SourceVersion`.

It is immutable after construction. The stored source id, text hash, source text, and line starts are observable through accessors, not by mutable field access. `LineMap::source` exposes the retained loaded text for consumers that need the exact normalized text behind the line map; it is not raw file or editor text when source loading changed offsets. Consumers must validate that the `source_id` belongs to the snapshot they are reporting against before converting offsets to user-facing line/column values.

If disk source loading stripped a leading UTF-8 BOM, byte offset `0` in the `LineMap` is the first byte after that BOM in the original file. Raw-file byte positions are recovered through `LoadingMap`, not by changing `SourceRange` or lexer span coordinates. For generated source, byte offset `0` is the first byte of the generated text supplied by the generator, even when that text starts with `U+FEFF`, and CRLF remains two bytes when the generator supplied CRLF.

### Source Range

`SourceRange` and `TextRange` are half-open: `start <= offset < end`.

Ranges must:

- reference one `SourceId`;
- use byte offsets aligned to UTF-8 scalar boundaries;
- remain inside the source text length;
- preserve zero-length ranges only for insertion points and synthesized anchors.

Reversed ranges are always invalid. APIs that receive an explicitly constructed reversed `SourceRange` or `TextRange` return `SourceMapError::ReversedRange`; callers can use `TextRange::try_new` when constructing a range from unchecked bounds.

### Loading Map

`LoadingMap` relates normalized `LoadedSource.text` to the source-loading input before BOM stripping or newline normalization. For disk sources, `original` ranges are byte offsets into the original file bytes after UTF-8 validation. For open buffers, `original` ranges are byte offsets into the editor-provided UTF-8 text; the LSP bridge then converts those byte offsets to protocol UTF-16 positions. The default generated-source loader preserves generated text byte-for-byte and emits no `LoadingMap`; generated source locations are represented through the optional `LoadedSource.generated_anchor`, `SourceAnchor::Generated`, and `GeneratedSpanOrigin`, which preserve the best available source anchor and reason.

When a leading UTF-8 BOM is stripped, the map records a `RemovedLeadingBom` segment for original byte range `[0, 3)` and the first `Original` loaded segment starts at loaded offset `0` and original byte offset `3`. Source loading may omit `LoadingMap` only when the loaded text is offset-identical to the source-loading input; generated loading is such an identity case because it performs no BOM or CRLF transform. The retained `SourceMapService` requires a retained map for loaded-to-original conversion; when callers want service-level conversion for offset-identical text, including custom generated-source flows, `LoadingMap::identity` represents the relation with one `Original` segment.

`LoadingMap::new` records caller-supplied segments without full structural validation. Source loaders that construct these maps are responsible for preserving the segment invariants: loaded ranges are ordered and non-overlapping; `Original` segments have equal loaded/original byte lengths; `NormalizedNewline` segments represent CRLF-to-LF normalization, normally loaded length 1 and original length 2; `RemovedLeadingBom` represents only the leading UTF-8 BOM original range `[0, 3)`; and every mapped loaded byte range is covered by a segment. Disk and open-buffer source loading preserve these invariants when emitting BOM/CRLF loading maps for original file bytes or editor-provided text.

### Preprocess Map

`PreprocessMap` relates the lexical text consumed by the lexer to the original source.

Original segments map lexical ranges back to source ranges. Removed comment segments preserve ordinary and doc-comment locations even when comments are absent from lexical input. Synthetic whitespace segments represent text inserted to keep token separation after comment removal or recovery.

`PreprocessMap::new` records caller-supplied segments without full structural validation, mirroring the loading-map policy. Mapping APIs still validate the requested `SourceId`, lexical bounds, and any segment or anchor source ids they touch. `LexicalSourceMapping` is the lower-level mapping result: `primary` is the best user source range when one exists, `anchors` preserve adjacent, comment, or generated anchors, and `kind` distinguishes exact, composite, and degraded mappings. `SourceMapService` converts retained loaded and lexical maps into `MappedSourceRange`, preserving the same exact/composite/degraded distinction while separating the primary range from secondary anchors. For loaded-to-original mapping, `primary` stays a validated range in loaded source text and `original_input` carries the corresponding source-loading input byte range.

The frontend owns snapshot retention and service access for this map. It may reuse or mirror the lightweight preprocess map produced by the lexer helper when constructing the retained session `PreprocessMap`. Later phases consume it to attach diagnostics and syntax nodes to original source locations.

### Generated Spans

Generated spans are used when a compiler-created item has no exact source range, such as:

- implicit obligations;
- inserted coercions or `qua` nodes;
- generated proof replay steps;
- documentation or extraction records derived from multiple inputs.

Generated spans must include an origin that points to the best available source anchor and a non-empty reason. `GeneratedSpanOrigin::new` and `SourceMapService::attach_generated_span` reject generated spans without that reason. Diagnostics may display generated spans as secondary information, but primary diagnostics should prefer original source ranges when available.

## Algorithm / Logic

### Line/Column Conversion

1. Validate the range against the `LineMap` source text length.
2. Locate start and end lines by binary searching `line_starts`.
3. Count Unicode scalar values from the line start to each offset.
4. Return one-based lines and one-based columns for diagnostics, artifacts, and CLI formatting.

LSP conversion must apply the protocol's UTF-16 position rules in the `mizar-lsp` bridge, not inside this module. This module exposes source-stable coordinates.

### Loaded-to-Original Mapping

1. Use the retained `LoadingMap` for the `SourceId`.
2. If the retained service has no `LoadingMap` for that `SourceId`, return `SourceMapError::MissingLoadingMapSegment`. Callers that need identity conversion should retain a `LoadingMap::identity`.
3. If a loaded range crosses a normalized segment, return a degraded `LoadedToOriginalRange` over the enclosing original byte range. The retained `SourceMapService` exposes this as a degraded `MappedSourceRange` whose `primary` remains the loaded source range and whose `original_input` records the original input byte range.
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
- reversed range bounds;
- range outside source text;
- offset not aligned to a UTF-8 boundary;
- line or column coordinate not representable as `u32`;
- loaded range outside loaded text;
- lexical range outside preprocessed text;
- missing loading map segment;
- missing preprocess segment;
- generated span without an origin reason.

Malformed user source is reported by frontend diagnostics. Source-map errors indicate compiler bugs or stale handles unless they originate from an explicitly stale LSP request.

## Tests

Key scenarios:

- line maps convert byte offsets to Unicode scalar columns;
- line maps report overflow instead of silently narrowing unrepresentable line or column values;
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
- LSP UTF-16 narrowing is explicit and reports overflow rather than using unchecked casts.

## Constraints and Assumptions

- Source maps are internal compiler data and are not published as a stable schema by this crate.
- Published artifacts may include projected source ranges, but not the full preprocessing map.
- Source range conversion must be deterministic and independent of scheduling order.
- The module must not read files directly; it works from source text and identity supplied by source loading and snapshot creation.
