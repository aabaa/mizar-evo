# アーキテクチャ設計仕様

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

このディレクトリには、複数のモジュールやクレートにまたがる境界・プロトコル・設計判断を定義する**横断的な内部設計文書**を置きます。

## 目的

モジュール単位の仕様（`doc/design/<crate>/<language>/<module>.md`）が個々の Rust ソースファイルを説明するのに対し、アーキテクチャ仕様は単一のモジュールだけでは答えられない問いを扱います。

- **サブシステムの境界はどこか？** 例: カーネルと ATP の境界
- **サブシステム間をつなぐプロトコルは何か？** 例: TPTP, SMT-LIB
- **外部ツールをどう統合するか？** 例: プロセス管理、タイムアウト
- **なぜその設計を代替案より優先して選んだのか？**

## 索引

アーキテクチャ文書には、厳密なパイプラインのフェーズ順ではなく、読み進める順序と設計上の依存関係に沿って番号を付けています。欠番となっている番号は、今後追加する予定の枠です。

| 文書 | パイプライン段階 | 概要 | 状態 |
|---|---:|---|---|
| [00.pipeline_overview.md](./00.pipeline_overview.md) | 全体 | ソースファイルから検証済みアーティファクトに至るエンドツーエンドのパイプライン | Draft |
| [01.ir_layers.md](./01.ir_layers.md) | 全体 | パイプラインのフェーズ間にまたがる IR の所有境界と安定性のルール | Draft |
| [02.source_and_frontend.md](./02.source_and_frontend.md) | 1-3 | ソース読み込み・前処理・字句解析・構文解析の境界 | Draft |
| [03.module_and_symbol_resolution.md](./03.module_and_symbol_resolution.md) | 0, 4-5 | パッケージ・モジュール・名前空間・ラベル・シンボルテーブルの解決 | Draft |
| [04.type_and_registration_resolution.md](./04.type_and_registration_resolution.md) | 6-7 | 型検査、cluster データベース、解決トレース | Draft |
| [05.overload_resolution.md](./05.overload_resolution.md) | 8 | 候補選択、subsumption DAG、`qua` の挿入 | Draft |
| [06.elaboration_and_core_ir.md](./06.elaboration_and_core_ir.md) | 9 | 表層言語から中核論理への低位化 | Draft |
| [07.vc_generation.md](./07.vc_generation.md) | 10-12 | アルゴリズム検証の準備と証明義務の生成 | Draft |
| [08.reasoning_boundary.md](./08.reasoning_boundary.md) | 12-14 | Mizar・ATP バックエンド・カーネルの間での推論責務の分担 | Draft |
| [09.atp_interface_protocol.md](./09.atp_interface_protocol.md) | 13 | ATP 問題の形式とエンコード戦略 | Draft |
| [10.atp_backend_integration.md](./10.atp_backend_integration.md) | 13 | 外部 ATP プロセスの実行、タイムアウト処理、証明書の収集 | Draft |
| [11.artifact_and_incremental_build.md](./11.artifact_and_incremental_build.md) | 15 | アーティファクトのスキーマ、キャッシュ更新、再現性 | Draft |
| [12.diagnostics_and_lsp.md](./12.diagnostics_and_lsp.md) | 全体, 15 | 診断、メタデータ、IDE 統合 | Draft |
| [13.documentation_and_extraction.md](./13.documentation_and_extraction.md) | 16 | ドキュメント生成とコード抽出 | Draft |
| [14.parallel_verification_and_scheduling.md](./14.parallel_verification_and_scheduling.md) | 0, 10-15 | 検証タスクグラフ、並列スケジューリング、キャンセル、決定的な結果順序 | Draft |
| [15.kernel_certificate_format.md](./15.kernel_certificate_format.md) | 13-14 | 最終的な証明書スキーマ、節トレースの検査、カーネルによる棄却の意味論 | Draft |
| [16.substitution_and_binding.md](./16.substitution_and_binding.md) | 4, 6, 9, 14 | 束縛変数、α 同値、捕獲回避、束縛子の正規化 | Draft |
| [17.cluster_trace_format.md](./17.cluster_trace_format.md) | 7, 11, 14-15 | 再生可能な cluster 展開とリダクション適用のトレース | Draft |
| [18.dependency_fingerprint.md](./18.dependency_fingerprint.md) | 0, 4-7, 11, 15 | 依存スライス、フィンガープリント、増分再ビルドのトリガ | Draft |
| [19.failure_semantics.md](./19.failure_semantics.md) | 全体 | 安定した失敗分類、伝播、決定的なエラー順序 | Draft |
| [20.test_strategy.md](./20.test_strategy.md) | 全体 | 失敗系・健全性のテストを優先する回帰テスト戦略 | Draft |
| [21.ai_agent_interface.md](./21.ai_agent_interface.md) | 全体, 15 | AI エージェント操作性: 安全編集クラス、認可スコープ、文脈／パッチプロトコルの枠組み | Draft |

`00.pipeline_overview.md` はこのディレクトリの親文書です。他のアーキテクチャ文書は、自分がどのパイプライン段階を詳細化するのかを明記し、「関連文書」節から概要へリンクを張ってください。

## 横断的な関心事

### メモリモデル

メモリのスケーラビリティは、特定のサブシステムの責務ではなく、横断的な設計特性です。その指針——言語仕様としては [doc/spec/ja/12.6.3](../../../spec/ja/12.modules_and_namespaces.md#1263-メモリモデル) と [doc/spec/ja/23.7.9](../../../spec/ja/23.package_management_and_build_system.md#2379-メモリ設計原則) で規範的に述べています——は次のとおりです。**インターフェイスと index は常駐させ、証明本体・トレース・AI 向けの詳細データは遅延的に読み込み、global な index を import closure ごとに複製しない。**

各アーキテクチャ文書は、この特性の 1 つの側面を担います。

| 側面 | 常駐集合を有界に保つ箇所 | 文書 |
|---|---|---|
| import される状態は最小限の射影 | `ModuleSummary` は exported な symbol/label と lexical summary を持ち、証明本体は持たない | [03.module_and_symbol_resolution.md](./03.module_and_symbol_resolution.md) |
| cluster／登録データはフィルタしたビュー | チェッカーは import スコープの登録 summary から構築した活性化済みの `RegistrationIndex` を利用する。`cluster-db` キャッシュは closure ごとに複製せず import スコープのビューを保持する | [04.type_and_registration_resolution.md](./04.type_and_registration_resolution.md), [11.artifact_and_incremental_build.md](./11.artifact_and_incremental_build.md) |
| トレースと witness は外部 artifact | 常駐データではなく、オンディスクのトレースと hash 参照の witness file | [11.artifact_and_incremental_build.md](./11.artifact_and_incremental_build.md), [17.cluster_trace_format.md](./17.cluster_trace_format.md) |
| 検証条件は module 単位に保つ | module 全体の正準的な `VcIr` を discharge 前に実体化する。証明義務ごとの ATP 作業は階層的なリソース予算で有界化する | [07.vc_generation.md](./07.vc_generation.md), [14.parallel_verification_and_scheduling.md](./14.parallel_verification_and_scheduling.md) |
| 変更分のみ再計算 | 依存フィンガープリントと逆依存コーンの再ビルド | [18.dependency_fingerprint.md](./18.dependency_fingerprint.md), [11.artifact_and_incremental_build.md](./11.artifact_and_incremental_build.md) |
| IDE の状態は増分的に保つ | LSP は未保存バッファのスナップショットを保持し、global な状態を再構築せず index 化された artifact から応答する | [12.diagnostics_and_lsp.md](./12.diagnostics_and_lsp.md) |
| ワーカー予算でピーク使用量を制限 | ビルド単位のメモリ上限とバックエンドごとのプロセス予算 | [14.parallel_verification_and_scheduling.md](./14.parallel_verification_and_scheduling.md) |
| エージェント向け文脈を有界に保つ | ライブラリ全体のダンプではなく、obligation 単位の文脈予算と遅延ロードの AI 向けデータ | [21.ai_agent_interface.md](./21.ai_agent_interface.md) |

これは定性的な常駐集合モデルであり、性能保証ではありません。具体的なメモリ予算やベンチマーク指標は、規範的仕様ではなくテスト／評価戦略（[20.test_strategy.md](./20.test_strategy.md)）に属します。

## 文書テンプレート

各アーキテクチャ文書は次の構成に従います。

```markdown
# アーキテクチャ: <タイトル>

## 目的
この文書が扱うアーキテクチャ上の課題。

## 関連文書
関連する外部仕様やアーキテクチャ文書への参照。

## 設計判断

### 検討した代替案
検討したアプローチとそのトレードオフ。

### 採用方針
採用した設計とその理由。

## インターフェイス定義
サブシステム間の境界、API、データ形式。

## 関連モジュール
この設計を実装するモジュール単位の仕様とソースファイル。
- `doc/design/<crate>/<language>/<module>.md` → `crates/<crate>/src/<module>.rs`

## 制約と前提
性能要件、セキュリティ上の考慮、互換性など。
```

## 他の文書層との関係

| 層 | ディレクトリ | 粒度 | 対象読者 |
|---|---|---|---|
| 外部仕様 | `doc/spec/en/` | 言語機能 | 利用者 |
| **アーキテクチャ** | **`doc/design/architecture/`** | **横断的なサブシステム** | **開発者** |
| モジュール仕様 | `doc/design/<crate>/` | 個々のファイル（1:1） | 開発者 |
