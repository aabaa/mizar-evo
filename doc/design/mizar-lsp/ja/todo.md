# mizar-lsp TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより
前に、専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。例外は
`range` で、コードが仕様に先行しているため task 2 が仕様を後付けする。
この crate は [internal 07](../../internal/ja/07.crate_module_layout.md) に
従い、アーキテクチャ 12、19 と internal 03 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| range | `range.md`（task 2） | `src/range_mapper.rs` | [~] 変換は実装済み、仕様が未着手 |
| server | `server.md`（task 4） | `src/server.rs` | [ ] |
| snapshot | `snapshot.md`（task 6） | `src/snapshot.rs` | [ ] |
| diagnostics | `diagnostics.md`（task 8） | `src/diagnostics.rs` | [ ] |
| build_bridge | `build_bridge.md`（task 10） | `src/build_bridge.rs` | [ ] |
| metadata | `metadata.md`（task 12） | `src/metadata.rs` | [ ] |
| navigation | `navigation.md`（task 14） | `src/navigation.rs` | [ ] |
| code_action | `code_action.md`（task 16） | `src/code_action.rs` | [ ] |
| explain | `explain.md`（task 18） | `src/explain.rs` | [ ] |

`mizar-lsp` はエディタ向けプロトコルブリッジを所有する: open バッファの
ソース snapshot と文書バージョン、`SourceRange` → LSP UTF-16 変換、現在の
`LspSnapshot` からの診断公開、公開済み artifact と snapshot メタデータから
提供される hover/ナビゲーション/セマンティック機能、構造化 fix 提案からの
code action、explanation クエリ、compiler driver を通じたビルドの
スケジューリング。証明の受理、型検査、ATP 推論、artifact の変更は一切
所有しない。3 つの波で構築する: **第 A 波**（range 仕様の後付け）は独立。
**第 B 波**（server、snapshot、診断、build bridge）は
`mizar-diagnostics`、`mizar-ir`、`mizar-driver` とともに着地する。
**第 C 波**（メタデータ機能）はメタデータを生産する意味論層とともに
成長する。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は現在 `mizar-session` と `mizar-lexer`（range 変換）に依存
する。第 B 波で `mizar-diagnostics`（レコードと索引）、`mizar-ir`
（snapshot ハンドル）、`mizar-driver`（ビルドリクエストとイベント）が
加わり、第 C 波で `mizar-artifact`（メタデータリーダー）が加わる。
アーキテクチャ:
[12.diagnostics_and_lsp.md](../../architecture/ja/12.diagnostics_and_lsp.md)、
[19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)。
internal: [03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)。

## 解決済みおよび保留中の決定

- **LSP トランスポート/プロトコルライブラリ: 未解決。task 4 で解決する。**
  既存のプロトコル crate と最小の自前 JSON-RPC 層のどちらを採るかを、
  workspace の外部依存ポリシーを勘案して決め、決定とその信頼への含意を
  `server.md` に記録する。
- **オーバーレイ診断: internal 03 により解決済み。** 未保存テキストの
  open バッファ診断は共有レコードの形を再利用するが、CLI 出力や
  `VerifiedArtifact` には決して公開されない。`diagnostics.md` がこれを
  再掲する。
- **メタデータの供給源: internal 03 により解決済み。** hover/
  ナビゲーション機能は公開済み artifact と `LspSnapshot` メタデータを
  読み、生のコンパイラ IR は決して読まない。鮮度は snapshot ごとに追跡
  され、古いデータは隠されず、マークされる。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-lsp` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 第 A 波: range の基盤

1. **lint 方針のガード。** [ ]
   - `mizar-frontend` のガードに倣った `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。
   - 依存: なし。仕様: リポジトリの慣行。

2. **仕様: `range.md`（後付け）。** [ ]
   - 既存の `range_mapper` に対する range 変換の仕様を執筆する（英語と
     日本語）: UTF-16 コードユニットの規則、サロゲートペア、改行、行
     マップとの相互作用、エラーケース。コードと仕様のギャップを
     フォローアップとして記録する。
   - 依存: 1。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「Range Conversion」。

3. **range 変換の強化。** [ ]
   - task 2 が見つけたギャップを閉じ、プロパティテスト（定義される範囲の
     ラウンドトリップ、サロゲートペア境界、CRLF/LF）を追加する。
   - テスト: プロパティスイート。フィクスチャコーパス上の変換の全域性。
   - 依存: 2。仕様: `range.md`。

### 第 B 波: server、snapshot、診断、build bridge

4. **仕様: `server.md`。** [ ]
   - server ライフサイクルの仕様を執筆する（英語と日本語、コードなし）:
     initialize/shutdown、capability ネゴシエーション、文書同期、
     リクエストルーティング、トランスポートライブラリの決定とその根拠。
   - 依存: 2。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「LSP Bridge」。

5. **server ライフサイクル。** [ ]
   - 選択したトランスポート上で initialize/shutdown、capability
     ネゴシエーション、文書同期を実装する。
   - テスト: インメモリトランスポート上のライフサイクルフィクスチャ。
     不正なリクエストにはプロトコルに従って応答し、決してクラッシュ
     しない。
   - 依存: 4。仕様: `server.md`。

6. **仕様: `snapshot.md`。** [ ]
   - snapshot/鮮度の仕様を執筆する（英語と日本語、コードなし）: open
     バッファの snapshot、文書バージョン、`mizar-ir` ハンドル上の
     current/stale な `LspSnapshot` モデル、staleness のマーク規則。
   - 依存: 4。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「LSP Snapshot」。

7. **open バッファの snapshot とバージョン。** [ ]
   - バージョン追跡付きのバッファ snapshot と current/stale snapshot
     レジストリを実装する。
   - テスト: 編集列でバージョンが単調に保たれる。stale な snapshot は
     マークされ、current として黙って提供されない。
   - 依存: 5、6、`mizar-ir` task 13。仕様: `snapshot.md`。

8. **仕様: `diagnostics.md`。** [ ]
   - 診断公開の仕様を執筆する（英語と日本語、コードなし）:
     `BuildDiagnosticIndex` からの公開、range 変換、未保存テキストの
     オーバーレイ診断（CLI/artifact へ決して出さない規則の再掲）、
     順序の保証。
   - 依存: 6。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「Diagnostic Aggregator」「Lightweight Open-Buffer Diagnostics」。

9. **診断の公開。** [ ]
   - 現在の snapshot の索引から変換済み range 付きで診断を公開する。
     古い snapshot の診断は current として決して公開されない。
   - テスト: 公開のフィクスチャ。stale 索引の抑制。決定的な順序。
   - 依存: 7、8、`mizar-diagnostics` task 9。仕様: `diagnostics.md`。

10. **仕様: `build_bridge.md`。** [ ]
    - build bridge の仕様を執筆する（英語と日本語、コードなし）:
      `CompilerDriver` API を通じたビルドのスケジューリング、ビルド
      イベントの消費、編集時のデバウンスと置き換え、watch モードとの
      相互作用。
    - 依存: 6。仕様: [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
      「Watch and LSP Build」。

11. **build bridge。** [ ]
    - `mizar-driver` を通じたビルドのスケジューリングとイベント消費を
      実装する。置き換えられたセッションはクリーンにキャンセルされる。
    - テスト: 編集バーストのフィクスチャがデバウンスされる。イベントが
      正しい snapshot に対応付く。置き換え時のキャンセル。
    - 依存: 9、10、`mizar-driver` task 8 と 11。仕様: `build_bridge.md`。

### 第 C 波: メタデータ機能

12. **仕様: `metadata.md`。** [ ]
    - メタデータリーダーの仕様を執筆する（英語と日本語、コードなし）:
      IDE 機能のための `VerifiedArtifact` 式メタデータとモジュール
      summary の読み込み、鮮度の規則、生 IR 禁止の境界。
    - 依存: 6。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)、
      [internal 06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)
      「Artifact Projection Boundary」。

13. **hover と型メタデータ。** [ ]
    - 式メタデータから hover（推論された型、解決済みシンボル、
      `@show_*` データ）を提供する。
    - テスト: フィクスチャ artifact 上の hover フィクスチャ。古いデータの
      マーク。
    - 依存: 12、`mizar-artifact` task 11。仕様: `metadata.md`。

14. **仕様: `navigation.md`。** [ ]
    - ナビゲーションの仕様を執筆する（英語と日本語、コードなし）:
      artifact メタデータと snapshot データ上の定義、参照、document
      symbol、semantic token。
    - 依存: 12。仕様: アーキテクチャ 12、
      [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)。

15. **ナビゲーション機能。** [ ] — メタデータの生産者が律速。
    - 定義、参照、document symbol、semantic token を、生産側の層
      （resolver メタデータ、checker の式メタデータ）の着地に合わせて
      増分で実装する。1 変更につき 1 機能増分。最後の増分が着地した
      時点でチェックを付ける。
    - 依存: 13、14。仕様: `navigation.md`。

16. **仕様: `code_action.md`。** [ ]
    - code action の仕様を執筆する（英語と日本語、コードなし）: 構造化
      fix 提案の LSP code action への変換、適用可能性のフィルタリング、
      自動適用禁止の規則。
    - 依存: 8。仕様: アーキテクチャ 12「Fix Suggestion」、
      [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
      「Code Actions」。

17. **code action。** [ ]
    - `mizar-diagnostics` の fix ペイロードから code action を実装する。
    - テスト: fix 種別ごとの action フィクスチャ。編集が range 変換を
      ラウンドトリップする。
    - 依存: 9、16、`mizar-diagnostics` task 13。仕様: `code_action.md`。

18. **仕様: `explain.md`。** [ ]
    - explanation クエリの仕様を執筆する（英語と日本語、コードなし）:
      explanation ハンドルのオンデマンドな上限付きペイロードへの解決、
      レイテンシ/サイズの制限。
    - 依存: 8。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
      「Explanation Queries」。

19. **explanation クエリ。** [ ]
    - `mizar-diagnostics` の explanation ストア上で explanation
      リクエストを実装する。
    - テスト: クエリのフィクスチャ。上限付きペイロード。裏付けデータ
      欠落時の明確な劣化。
    - 依存: 17、18、`mizar-diagnostics` task 15。仕様: `explain.md`。

### 強化と横断フォローアップ

20. **決定性と鮮度のスイート。** [ ]
    - 同一の snapshot が同一の公開診断と機能応答を生み、古いデータが
      常にマークされることのプロパティ的検証。
    - 依存: 11、13。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

21. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。
    - 依存: 11。仕様: 全モジュール仕様。

22. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 21。仕様: 全モジュール仕様と本 TODO。

23. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-lsp/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 22。仕様: リポジトリのドキュメント方針。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-lsp
cargo clippy -p mizar-lsp --all-targets -- -D warnings
```

第 B/C 波のタスクでは統合先も実行する:

```text
cargo test -p mizar-diagnostics
cargo test -p mizar-driver
cargo test -p mizar-artifact
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- ブリッジが所有するのはプロトコル変換と鮮度であって、意味論ではない:
  証明の受理、型検査、ATP 推論、artifact の変更は行わない。
- 未保存テキストのオーバーレイ診断は CLI 出力や `VerifiedArtifact` に
  決して到達しない。
- 機能は公開済み artifact と snapshot メタデータを読む。生のコンパイラ
  IR は決して読まない。古いデータは隠されず、マークされる。
- LSP ワークフローはレイテンシを優先してよいが、batch 検証が意味論の
  ベースラインであり続ける。
