# Task LSP-RAW-OPEN-BUFFER-RANGE: raw editor range arithmetic
Canonical language: English; [Japanese pointer](../ja/LSP-RAW-OPEN-BUFFER-RANGE.md).
Status: implemented. Tier: full. Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md).
Scope: LSP 30, a coordinate prerequisite for tasks 6–11; those tasks remain open.
Authority: [range](../../mizar-lsp/en/range.md), [Session maps](../../mizar-session/en/source_map.md#loading-map), internal 03 Range Conversion and spec §23.6.
Dependencies: real native acquisition and shared Session normalization are implemented.
Gap: loaded-only UTF-16 arithmetic does not recover pre-normalization editor offsets.

Add lsp_range_from_open_buffer(line_map: &LineMap, loading_map: Option<&LoadingMap>,
origin: &SourceOriginInput, range: SourceRange) -> Result<LspRange, RangeMapError>,
re-exported from the existing LSP lib. Add existing-enum InvalidOpenBufferContext.
Validate the loaded range through LineMap first, preserving native SourceMap errors.
Require OpenBuffer origin with matching nonnegative expected/actual versions.
Use the existing pure normalize_source_text on its exact raw editor text;
require the loaded LineMap text and optional entire LoadingMap to equal that
canonical normalization, source id, hash/length, OpenBuffer URI/version and segments.
Do not accept a missing transformed map or an invented identity map.
Map loaded bytes through the validated LoadingMap, or retain identity bytes when
normalization emitted no map, then reuse UTF-16 counting/bounds on exact raw text.
Raw editor lines split at LF, lone CR and CRLF (one break); points inside EOL clamp
to that line's content end. Session normalization and old loaded-only line rules stay unchanged.
Normalized-newline ranges use Session's enclosing raw range; empty/EOF ranges stay valid.
Preserve old loaded-only APIs, error meaning, UTF-8 checks and uinteger bounds.

This is arithmetic consistency only. Public descriptors/maps are not validation tokens;
caller binds actual source/snapshot/URI/version and currentness before publication or edits.
No filesystem reads, loader/parser call, source-ID allocation, text mutation, protocol change,
freshness/current diagnostic/semantic snapshot/artifact/cache/proof or Step completion claim.
No new Rust file/type/field/adapter/dependency or lower-stage/spec/.miz/expectation/trace change.
Source scope: existing LSP range_mapper.rs, inline tests and lib.rs re-export only.
Paired range owners/plans/TODOs, canonical contract and partial audit owner pointer only.

Existing inline tests acquire real OpenBuffer sources and test unchanged text, BOM,
CRLF/lone CR/mixed line endings, supplementary/combining scalars, newline boundaries, empty/BOM-only EOF,
and exact raw UTF-16 endpoints. Corrupt/mismatched maps, URI/version/text/source id,
wrong origin, invalid UTF-8 offsets/reversed/bounds fail closed; old arithmetic tests remain.
Require independent spec/test/implementation/volume/consistency/bounded-gate reviews
and parent cargo fmt --check, full Clippy -D warnings and cargo test.
Exit: actual native loaded ranges map to exact raw editor coordinates; LSP consumers remain open.
