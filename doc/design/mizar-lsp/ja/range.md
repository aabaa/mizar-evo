# Range conversion
正本: [英語版](../en/range.md)。

[Session](../../mizar-session/ja/source_map.md#ソース範囲) が半開 byte range と正確な text を所有する。
`lsp_range_from_source_range` は LineMap で source identity、
順序・範囲・UTF-8 境界を検査した後、その行構造と行頭からの UTF-16 unit 数を使う。
LSP 座標は0起点で、補助平面の scalar は2 unit、結合文字も独立に数える。
native validation が許す EOF と空 range は有効。native error は `RangeMapError::SourceMap`、
UTF-16 narrowing 失敗と LSP uinteger (2³¹−1) を超える行/文字値は `PositionOverflow` とする。

`source_range_from_lexer_span` は読み込み済み byte 座標と SourceId のコピー、`lsp_range_from_lexer_span` は同じ検証である。
BOM・改行は取得側が所有し、
disk/open-buffer の CRLF は正規化、generated text は保持される。mapper は text を変えない。
caller が正確な snapshot/map を対応付ける。取得後の raw editor 座標と edit applicability は
後続 snapshot bridge の責務であり、この算術 API は freshness や逆変換を保証しない。
