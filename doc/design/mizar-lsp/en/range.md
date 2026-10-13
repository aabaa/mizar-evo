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
Callers bind the exact snapshot/map before conversion. Raw editor coordinates
after acquisition changes and edit applicability belong to the later snapshot
bridge; this arithmetic API establishes neither document freshness nor an inverse map.
