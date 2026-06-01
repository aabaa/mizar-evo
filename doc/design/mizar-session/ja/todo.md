# mizar-session TODO

> 正規言語: 英語。英語版が正典です: [../en/todo.md](../en/todo.md)。

## ステータス凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | ステータス |
|---|---|---|---|
| ids | [en/ids.md](../en/ids.md) | `src/ids.rs` | [ ] |
| source_map | [en/source_map.md](../en/source_map.md) | `src/source_map.rs` | [~] |
| snapshot | [en/snapshot.md](../en/snapshot.md) | `src/snapshot.rs` | [ ] |
| source | [en/source.md](../en/source.md) | `src/source.rs` | [~] |
| retention | [en/retention.md](../en/retention.md) | `src/retention.rs` | [ ] |

## 推奨実装順序

本クレートは識別子・座標を提供する最下層（leaf）なので、内部依存に沿ってボトムアップで実装する。
`SourceId` は他の全モジュールが参照する共有プリミティブ。

1. **ids** — `SourceId` / `BuildSnapshotId` / `SourceMapId` / lease id 群 + `SessionIdAllocator`。内部依存なし。
2. **source_map** — `LoadingMap` / `PreprocessMap` / `SourceAnchor` / generated span / `SourceMapService` を追加し、`SourceRange` / `LineMap` へ `SourceId` を統合。
3. **snapshot** — `SourceVersion` / `BuildSnapshot` / `SnapshotRegistry`、内容由来の `BuildSnapshotId`、freshnessチェック。
4. **source** — `LoadedSource` / `SourceInput` / `SourceLoader`（UTF-8検証・先頭BOM除去・CRLF正規化・`source_hash`・`LoadingMap`）。ids + source_map + snapshot を結合する。
5. **retention** — `RetentionManager`、lease、current mark、回収ポリシー。

## モジュール別の残作業

### source_map [~]
完了: `LineMap`、`SourceRange`、`LineColumn`、`LineColumnRange`、`SourceMapError`、座標変換。
残り: `LoadingMap`（+ `LoadingOrigin`、`LoadingMapSegment`）、`PreprocessMap`（+ `PreprocessSegment`）、`SourceAnchor`、generated span、`SourceMapService` trait、`SourceId` 統合、`u32` オーバーフロー報告。

### source [~]
完了: `NormalizedPath`、`normalize_source_path`、`SourcePathError`（パス正規化のみ）。
残り: `SourceInput` / `SourceOriginInput`、`LoadedSource`、`SourceLoader` trait、disk / open-buffer / generated のロード、`source_hash`、BOM + 改行正規化、`LoadingMap` 生成。

## 横断的タスク

- [ ] 共有識別子プリミティブとして `ids` モジュールを導入（`SourceId`、`BuildSnapshotId`、`SessionIdAllocator`）
- [ ] `SourceRange` / `LineMap` へ `SourceId` を統合
- [ ] 決定的かつ内容由来の `BuildSnapshotId` ハッシュ（正規順序・ドメインセパレータ）
- [ ] snapshot の freshness + lease / retention ライフサイクル
- [ ] プロパティ/決定性テスト: 同一の正規入力 ⇒ 同一の `BuildSnapshotId`
- [ ] `ids.md` / `snapshot.md` / `retention.md` の日本語版同期（現状 `ja/` に未整備）

## 注記

- `mizar-session` は識別子・座標の最下層クレート。下流クレートはそのハンドルを consume して source/snapshot 状態を一致させる。
- `mizar-lexer` は本クレートから疎結合に保つこと。lexerトークンの span 統合はフロントエンドの責務（[../../mizar-lexer/ja/todo.md](../../mizar-lexer/ja/todo.md) 参照）。
- `source_map` へ `SourceId` を統合する前に、span橋渡しの方針（lexerは疎結合のまま vs. lexerがsessionの `SourceRange` を採用）を確定すること。
- source map と snapshot identity は内部コンパイラデータであり、外部公開スキーマではない。
