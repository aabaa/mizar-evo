# アーキテクチャ設計仕様

> Canonical language: English. English canonical version: [../en/README.md](../en/README.md).

このディレクトリには、`doc/design/architecture/en/` の日本語 companion 文書を置く。
実装判断・正式な用語・API 境界は英語版を正とし、この日本語版はレビューと議論をしやすくするための補助文書として扱う。

## Index

| Document | Pipeline Phase | Description | Status |
|---|---:|---|---|
| [00.pipeline_overview.md](./00.pipeline_overview.md) | All | ソースから検証済み成果物までの全体パイプライン | Draft |
| [ir_layers.md](./ir_layers.md) | All | 各フェーズで受け渡す IR 層の責務と安定境界 | Draft |
| [source_and_frontend.md](./source_and_frontend.md) | 1-3 | source loading, preprocessing, lexing, parsing の境界 | Draft |
| [reasoning_boundary.md](./reasoning_boundary.md) | 12-14 | Mizar 側 / ATP 側 / kernel 側の推論責務境界 | Draft |
| [atp_interface_protocol.md](./atp_interface_protocol.md) | 13 | ATP に渡す問題形式とエンコーディング方針 | Draft |
| [atp_backend_integration.md](./atp_backend_integration.md) | 13 | 外部 ATP process の実行・timeout・certificate 回収 | Draft |

## 運用ルール

- 英語版を先に更新する。
- 日本語版は英語版の設計判断に追従する。
- 逐語訳よりも、日本語で読みやすく設計意図が伝わることを優先する。
- 英語版と日本語版が食い違う場合、英語版を正とする。
