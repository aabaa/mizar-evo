# Range conversion
Canonical language: English; [Japanese companion](../ja/range.md).

[Session](../../mizar-session/en/source_map.md#source-range) owns half-open byte ranges and exact text.
`lsp_range_from_source_range` uses LineMap to validate source identity/order/bounds/UTF-8,
then converts lines to zero-based LSP and counts UTF-16 from each line start: supplementary
scalars count two units, combining marks independently; EOF/empty ranges remain valid.
Native errors stay `RangeMapError::SourceMap`; failed narrowing or line/character values
above LSP uinteger (2³¹−1) produce `PositionOverflow`.

`source_range_from_lexer_span` copies loaded byte coordinates and supplied SourceId;
`lsp_range_from_lexer_span` applies the same validation. Acquisition owns BOM/newlines:
disk/open-buffer loading normalizes CRLF; generated text is retained. The mapper changes no text.
Callers bind the exact snapshot/map before conversion; loaded-only APIs establish no
raw editor coordinates, document freshness or edit applicability.

## Raw OpenBuffer coordinates

`lsp_range_from_open_buffer` first validates the loaded range through its LineMap.
An OpenBuffer descriptor must have matching nonnegative expected/actual versions.
Session's pure normalizer checks its exact raw text against the loaded LineMap and
the entire optional native LoadingMap (source id, hash/length, URI/version, segments).
Mismatch yields `RangeMapError::InvalidOpenBufferContext`; native range/map errors
remain `SourceMap`. An omitted map is valid only for offset-identical input.
LoadingMap recovers raw byte endpoints before UTF-16 counting/bounds on raw text.
Raw editor lines treat LF, lone CR and CRLF as breaks (CRLF is one); an endpoint
inside EOL clamps to the line content end. The existing loaded-only line rules
and Session normalization remain unchanged. Normalized-newline spans use the
map's enclosing raw range. Empty and EOF
ranges, including BOM-only text, follow the native map. No text or IDs are changed.
These public inputs establish arithmetic consistency only. The snapshot bridge
still binds actual source, URI/version and currentness, and validates edit applicability.
