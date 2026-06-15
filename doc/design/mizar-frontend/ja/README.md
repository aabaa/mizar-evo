# モジュール仕様: mizar-frontend

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-frontend` は、フェーズ 1〜3 の統制モジュール（source_and_frontend パイプラインの Step 1〜5）を所有する。すなわち、ソースの読み込み、ソースマップ、前処理の統制、アクティブ字句環境の構築、字句解析器の呼び出し、parser seam の呼び出し、そして統合されたフロントエンド出力である。

この crate は、次を所有しない。ソース同一性・ソースハッシュ・スナップショット（`mizar-session`）、生スキャン・コメント除去・字句環境の組み立て・トークン曖昧性解消規則（`mizar-lexer`）、`SurfaceAst` ノード定義（`mizar-syntax`）や文法・Pratt 優先順位・回復（`mizar-parser`）である。これらの crate が提供するプリミティブを、フロントエンドが `FrontendOutput` へと統制する。`StubParserSeam` は source-to-token coordinator 経路のために残り、`ast = None` を返す。

## コンテキスト

- [doc/design/architecture/ja/00.pipeline_overview.md](../../architecture/ja/00.pipeline_overview.md) - フェーズ境界とビルドスナップショット
- [doc/design/architecture/ja/02.source_and_frontend.md](../../architecture/ja/02.source_and_frontend.md) - フロントエンドパイプライン、インターフェース定義、回復、診断、インクリメンタリティ
- [doc/design/architecture/ja/01.ir_layers.md](../../architecture/ja/01.ir_layers.md) - `SourceUnit`、`PreprocessedSource`、`TokenStream`、`SurfaceAst`
- [doc/design/mizar-session/ja/README.md](../../mizar-session/ja/README.md) - ここで利用するソース同一性・ソースマップ・スナップショット
- [doc/design/mizar-lexer/ja/README.md](../../mizar-lexer/ja/README.md) - 前処理ヘルパー、生スキャン、インポート事前走査、字句環境、スコープスケルトン、曖昧性解消器
- [doc/design/mizar-syntax/ja/README.md](../../mizar-syntax/ja/README.md) - ここで利用する `SurfaceAst` ノード定義
- [doc/design/mizar-parser/ja/README.md](../../mizar-parser/ja/README.md) - ここで呼び出す文法、Pratt 解析、回復

## 索引

| ドキュメント | 対応先 | 説明 | 状態 |
|---|---|---|---|
| [00.crate_plan.md](./00.crate_plan.md) | `doc/design/autonomous_crate_development.md` protocol evidence | retrospective autonomous crate-development plan、責務境界、gap 分類、task decomposition、exit criteria | Implemented |
| [source.md](./source.md) | `crates/mizar-frontend/src/source.rs` | Step 1: `mizar-session` のソース同一性・line map・loading map を橋渡しする `SourceUnit` の読み込み | Implemented |
| [preprocess.md](./preprocess.md) | `crates/mizar-frontend/src/preprocess.rs` | Step 2: `PreprocessedSource`、コメント／ドキュメントコメントの分離、注釈構文の保持、浅いインポート事前走査の統制 | Implemented |
| [lexical_env.md](./lexical_env.md) | `crates/mizar-frontend/src/lexical_env.rs` | Step 3: インポートスタブと依存字句サマリからのアクティブ字句環境の構築 | Implemented through task 6 |
| [lexing.md](./lexing.md) | `crates/mizar-frontend/src/lexing.rs` | Step 4: 回復可能な生スキャン・スコープスケルトン・位置別 parser lexing plan・文脈依存の曖昧性解消による `TokenStream` | Implemented through task 22 |
| [parsing.md](./parsing.md) | `crates/mizar-frontend/src/parsing.rs` | Step 5: parser seam の呼び出し、パーサー入力の組み立て、位置別 string context planning、`SurfaceAst` の受け渡し | Implemented through task 28 current parser growth |
| [cache_key.md](./cache_key.md) | `crates/mizar-frontend/src/cache_key.rs` | parser lexing-plan content key を含め、`FrontendOutput.cache_keys` で公開する層状 frontend content cache keys | Implemented through task 20 |
| [span_bridge.md](./span_bridge.md) | `crates/mizar-frontend/src/span_bridge.rs` | 字句解析器のバイトスパンから `mizar-session` の `SourceRange` への座標橋渡し | Implemented for task 1 |
| [orchestration.md](./orchestration.md) | `crates/mizar-frontend/src/orchestration.rs` | フェーズ 1〜3 のエンドツーエンド統制（Step 1〜5）、parser lexing-plan wiring、診断統合、`FrontendOutput` | Implemented through task 28 current parser growth |
| [source_spec_correspondence.md](./source_spec_correspondence.md) | `crates/mizar-frontend` の仕様、ソース、ユニットテスト | task 16 の公開 API／エラー variant／タスク要件対応監査 | Implemented |
| [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | `doc/design/mizar-frontend/en/` と `doc/design/mizar-frontend/ja/` | task 17 の二言語 API／状態／用語／リンク／挙動同期監査 | Implemented |
| [crate_exit_report.md](./crate_exit_report.md) | `doc/design/autonomous_crate_development.md` protocol evidence | retrospective hard-gate status、quality score、deferred items、verification results、next-task handoff | Implemented |
| [todo.md](./todo.md) | `crates/mizar-frontend` | モジュール実装順序、状態、残作業 | Living |

## crate 境界

`mizar-frontend` は、ソースから構文への統制を提供する。

- `mizar-session` の `LoadedSource` から射影した単一ファイルのソース読み込み（`SourceUnit`）。
- `PreprocessedSource` を生成する前処理の統制（コメント、ドキュメントコメント、字句テキスト内の注釈構文保持、浅いインポートスタブ）。
- 浅いインポートと依存字句サマリからのアクティブ字句環境の構築。
- session の `SourceRange` スパンを持つ `TokenStream` を生成する文脈依存トークン化。
- 任意の AST を生成する parser seam の呼び出し（スタブの seam または実 parser の回復不能入力では `ast = None`、回復可能な実 parser 入力では `SurfaceAst`）。
- `SourceUnit`、`PreprocessedSource`、`ActiveLexicalEnvironment`、
  `TokenStream`、`SurfaceAst` の層状 frontend content cache keys。
- 字句解析器スパンから session の `SourceRange` への座標橋渡し。
- 単一の `FrontendOutput` への決定的な診断統合。

次を行ってはならない。

- ソース同一性・ソースハッシュ・スナップショットの所有。
- 生スキャン・コメント除去・トークン曖昧性解消規則の所有。
- `SurfaceAst` ノード定義やパーサーの文法／回復ロジックの所有。
- cache storage、cache hit 検証、scheduler task-key composition の所有。
- 意味的な名前解決、型検査、オーバーロード選択、クラスタ登録、証明義務生成。
