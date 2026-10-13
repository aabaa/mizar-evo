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
caller が正確な snapshot/map を対応付ける。loaded-only API は原文 editor 座標、
document freshness、edit applicability を保証しない。

## Raw OpenBuffer 座標

`lsp_range_from_open_buffer` は最初に loaded range を LineMap で検査する。
OpenBuffer descriptor の expected/actual version は非負で一致する必要がある。
Session の純粋 normalizer により、正確な原文 text と loaded LineMap、任意の native
LoadingMap 全体（source id、hash/length、URI/version、segment）が一致するか検査する。
不一致は `RangeMapError::InvalidOpenBufferContext`、native range/map error は
`SourceMap` のまま。map の省略は offset が変わらない入力だけに許す。
LoadingMap で原文 byte の端点に戻し、その原文の UTF-16 計数と境界検査を行う。
editor 原文の LF・単独 CR・CRLF を改行とし、CRLF は1回と数える。
改行文字内部の端点はその行の本文末尾へ丸める。既存 loaded-only の行規則と
Session 正規化は維持する。
改行正規化を跨ぐ span は native map の包絡範囲、空/EOF range と BOM だけの text
も native map に従う。text と ID は変更しない。
公開入力は算術的整合性のみを表し、実 source・URI/version・currentness の束縛と
edit applicability の検査は引き続き snapshot bridge の責務である。
