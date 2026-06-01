# Module Specifications: mizar-session

> Canonical language: English. English canonical version: [../en/README.md](../en/README.md).

`mizar-session` は、batch・watch・LSP の各ビルドが用いるソースの同一性、ビルドスナップショット、ソースバージョン、ソースマップ、スナップショット保持の契約を所有します。

タスクスケジューリング、IR ストレージ、アーティファクトの公開、診断の集約は所有しません。これらの crate は `mizar-session` のハンドルを消費し、自分たちが観測しているソース・依存・構成の状態がまさに何であるかについて合意します。

## Context

- [doc/design/architecture/ja/00.pipeline_overview.md](../../architecture/ja/00.pipeline_overview.md) — フェーズ境界とビルドスナップショット
- [doc/design/architecture/ja/02.source_and_frontend.md](../../architecture/ja/02.source_and_frontend.md) — ソース読み込み、行マップ、前処理マップ、コメント、ソーススパン
- [doc/design/architecture/ja/11.artifact_and_incremental_build.md](../../architecture/ja/11.artifact_and_incremental_build.md) — ソースハッシュ、依存アーティファクトのハッシュ、増分的な再利用
- [doc/design/architecture/ja/12.diagnostics_and_lsp.md](../../architecture/ja/12.diagnostics_and_lsp.md) — LSP スナップショット、ソース範囲、鮮度
- [doc/design/internal/ja/01.compiler_driver_and_pipeline_scheduler.md](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md) — `BuildSnapshot`、タスクグラフの入力同一性、キャンセル
- [doc/design/internal/ja/03.diagnostics_model_and_lsp_bridge.md](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md) — 診断のインデックス化とオープンバッファのオーバーレイ
- [doc/design/internal/ja/06.ir_storage_and_snapshot_handles.md](../../internal/ja/06.ir_storage_and_snapshot_handles.md) — `PhaseOutputRef<T>`、サイドテーブル、保持されるスナップショットハンドル

## Index

| Document | Maps To | Description | Status |
|---|---|---|---|
| [ids.md](./ids.md) | `crates/mizar-session/src/ids.rs` | 不透明なセッション識別子、順序付け、シリアライズ境界、互換性規則 | Draft |
| [source.md](./source.md) | `crates/mizar-session/src/source.rs` | ソース読み込みレコード、正規化パス、ソースハッシュ、オープンバッファのソーステキスト | Draft |
| [snapshot.md](./snapshot.md) | `crates/mizar-session/src/snapshot.rs` | `BuildSnapshot`、`SourceVersion`、スナップショットの同一性、保持、鮮度の契約 | Draft |
| [source_map.md](./source_map.md) | `crates/mizar-session/src/source_map.rs` | `LineMap`、ソース範囲、前処理マップ、生成スパン、座標変換 | Draft |
| [retention.md](./retention.md) | `crates/mizar-session/src/retention.rs` | スナップショットリース、LSP／watch の保持、ガベージコレクション方針 | Draft |
| [todo.md](./todo.md) | `crates/mizar-session` | モジュールの実装順序、ステータス、残作業 | Living |

## Crate Boundary

`mizar-session` は、不変の同一性サービスと座標サービスを提供します。

- ソースファイルの同一性とソースハッシュ
- ソース・依存・ロックファイル・ツールチェイン・検証器構成の各状態にまたがるビルドスナップショットの同一性
- キャッシュキー・アーティファクト・診断・LSP 公開が用いるソースバージョンのレコード
- 生ソース・前処理済みの字句テキスト・生成された内部フラグメントの間でのソース範囲変換
- 診断・LSP ビュー・フェーズ出力がスナップショットを参照している間に用いるスナップショット保持ハンドル

次のことは行ってはなりません。

- フェーズタスクのスケジューリング
- 型付き IR フェーズ出力の保存
- キャッシュ互換性の判断
- 診断の集約
- アーティファクトの公開
- 証明ポリシーの評価
