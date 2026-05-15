# Module: source_map

> Canonical language: English. English canonical version: [../en/source_map.md](../en/source_map.md).

## Purpose

この module は `mizar-session` の source coordinate tables and range conversion を定義する。

Frontend、diagnostics、LSP、documentation、extraction、artifacts、IR side tables が mutable source text や frontend internals を共有せずに source ranges について合意できるようにする。Raw source ranges、line/column conversion、preprocessing maps、generated spans、synthesized text 用の degraded mappings を扱う。

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

Offsets は validated UTF-8 source text への byte offsets である。User-facing columns は frontend architecture が定義する Unicode scalar column rule を使うため、consumers が ad hoc に計算せず `LineMap` を通して変換しなければならない。

## Dependencies

- Internal: `SourceId` and source-version identity のための `snapshot`
- External: hashing、UTF-8 text utilities、LSP range types

この module は frontend phases、`mizar-ir` side tables、`mizar-diagnostics`、`mizar-lsp`、`mizar-artifact`、`mizar-doc`、`mizar-extract` から consume される。

## Data Structures

### Line Map

`LineMap` は `SourceVersion` が表す exact source text の line starts を記録する。

Construction 後 immutable である。Consumers は offsets を user-facing line/column values に変換する前に、`source_id` が report 対象 snapshot に属することを検証しなければならない。

### Source Range

`SourceRange` は half-open である: `start <= offset < end`。

Ranges must:

- 一つの `SourceId` を reference する
- UTF-8 scalar boundaries に aligned した byte offsets を使う
- source text length の内側に残る
- zero-length ranges は insertion points and synthesized anchors に限って preserve する

### Preprocess Map

`PreprocessMap` は lexer が consume する lexical text と original source を関連付ける。

Original segments は lexical ranges を source ranges に戻す。Removed comment segments は comments が lexical input から消えていても ordinary and doc-comment locations を preserve する。Synthetic whitespace segments は comment removal or recovery の後に token separation を保つため inserted された text を表す。

Frontend がこの map の construction を所有する。Later phases は original source locations に diagnostics and syntax nodes を attach するために consume する。

### Generated Spans

Generated spans は compiler-created item に exact source range がない場合に使う。Examples:

- implicit obligations
- inserted coercions or `qua` nodes
- generated proof replay steps
- multiple inputs から derived された documentation or extraction records

Generated spans は best available source anchor and reason を指す origin を含めなければならない。Diagnostics は generated spans を secondary information として表示してよいが、primary diagnostics は available な場合 original source ranges を prefer するべきである。

## Algorithm / Logic

### Line/Column Conversion

1. `LineMap` source text length に対して range を validate する。
2. `line_starts` を binary search して start and end lines を locate する。
3. Line start から各 offset まで Unicode scalar values を count する。
4. Diagnostics、artifacts、CLI formatting のため one-based lines and one-based columns を返す。

LSP conversion は protocol の UTF-16 position rules を `mizar-lsp` bridge で apply しなければならない。この module は source-stable coordinates を expose する。

### Lexical-to-Source Mapping

1. Lexical range と intersect する preprocess segments をすべて見つける。
2. Range が一つの contiguous original source range に map できる場合、その range を返す。
3. Removed or synthetic segments をまたぐ場合、primary and secondary anchors を持つ composite mapping を返す。
4. Original source が存在しない場合、generated anchor を返す。

Composite mappings は diagnostics、documentation attachment、explanation metadata に許可される。Cache keys and artifact hashes は source hashes and stable ids を使わなければならず、composite mappings の serialized pretty forms を使ってはならない。

### Source Map Retention

Source maps は、snapshot lease、diagnostic index、LSP publication、IR side table のいずれかが参照している間、owning snapshot と一緒に retain される。Owning snapshot が collected された後に drop してよい。

## Error Handling

`SourceMapError` includes:

- unknown source id
- range outside source text
- offset not aligned to a UTF-8 boundary
- lexical range outside preprocessed text
- missing preprocess segment
- generated span without an origin reason

Malformed user source は frontend diagnostics によって reported される。Source-map errors は、explicitly stale LSP request に由来する場合を除き compiler bugs or stale handles を示す。

## Tests

Key scenarios:

- line maps convert byte offsets to Unicode scalar columns
- CRLF and LF inputs produce deterministic line starts according to source-loading rules
- removed comments map to preserved comment source ranges
- lexical ranges spanning comment removal produce composite mappings
- synthetic whitespace does not become a primary user source range
- generated anchors preserve their origin reason
- invalid byte offsets and cross-source ranges are rejected
- LSP UTF-16 conversion remains outside this module

## Constraints and Assumptions

- Source maps are internal compiler data and are not published as a stable schema by this crate.
- Published artifacts may include projected source ranges, but not the full preprocessing map.
- Source range conversion must be deterministic and independent of scheduling order.
- The module must not read files directly; it works from source text and identity supplied by source loading and snapshot creation.
