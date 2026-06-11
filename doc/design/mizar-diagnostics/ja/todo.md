# mizar-diagnostics TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割
（`failure_record`、`aggregator`）に、アーキテクチャ 12 と internal 03 の
registry/render/fix/explain モジュールを加えたものに従う。この crate は
アーキテクチャ 12、19 と internal 03 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| registry | `registry.md`（task 2） | `src/registry.rs` | [ ] |
| failure_record | `failure_record.md`（task 4） | `src/failure_record.rs` | [ ] |
| sink | `sink.md`（task 6） | `src/sink.rs` | [ ] |
| aggregator | `aggregator.md`（task 8） | `src/aggregator.rs` | [ ] |
| render | `render.md`（task 10） | `src/render.rs` | [ ] |
| fix | `fix.md`（task 12） | `src/fix.rs` | [ ] |
| explain | `explain.md`（task 14） | `src/explain.rs` | [ ] |

`mizar-diagnostics` はすべての phase が共有する正準 diagnostic レコードを
所有する: 安定した diagnostic コードのレジストリ、構造化された failure
レコード（アーキテクチャ 19）、生産者向け sink API、決定的順序を持つ
ビルドレベルの集約、CLI レンダリング、構造化された fix 提案、遅延
explanation ハンドル。ツールは `DiagnosticCode` をキーにし、メッセージ
テキストには決して依存しない。メッセージはバージョン間で改善してよいが、
コードは再利用してはならない。

依存順序: `registry` → `failure_record` → `sink` → `aggregator` →
`render` / `fix` / `explain`。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session`（ソース範囲と snapshot id）にのみ依存する。
最初の消費者は `mizar-resolve`（同 crate task 13 のゲートにある採用時期の
決定）であり、LSP ブリッジは `mizar-lsp` から同じレコードを消費する。
アーキテクチャ:
[12.diagnostics_and_lsp.md](../../architecture/ja/12.diagnostics_and_lsp.md)、
[19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)。
internal: [03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)。
仕様: [22.error_handling_and_diagnostics.md](../../../spec/ja/22.error_handling_and_diagnostics.md)。

## 解決済みおよび保留中の決定

- **採用時期: 未解決。`mizar-resolve` task 13 のゲートが所有する。** この
  crate はいずれにせよ目標配置に存在する。決定は、resolver が最初の診断
  から採用するか、1 層遅らせるかである。トップレベルに登録済み。
- **既存の crate ごとの診断の移行: 未解決。task 16 で解決する。**
  `mizar-lexer`/`mizar-frontend`/`mizar-parser` の診断はこの crate より
  古い。共有レコードへ移行する（その順序も）か、変換アダプターの背後で
  ローカル型を維持するかを決め、決定とそのトリガーをここととトップ
  レベルに記録する。
- **コード空間の割り当て: 未解決。task 2 で解決する。** 仕様第 22 章に
  従い、phase ファミリーごとの数値コード範囲と retirement ポリシーを
  決める。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-diagnostics` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### レコードとレジストリ

1. **crate の足場と lint 方針のガード。** [ ]
   - `mizar-session` に依存する workspace メンバー `mizar-diagnostics` を
     追加し、`mizar-frontend` のガードに倣った `tests/lint_policy.rs` を
     追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: なし。仕様: アーキテクチャ 12。

2. **仕様: `registry.md`。** [ ]
   - レジストリの仕様を執筆する（英語と日本語、コードなし）: 恒久的な
     `DiagnosticCode` の割り当て、phase ファミリーごとのコード空間、
     retirement 規則、互換性検証、参照メタデータ（意味論名、既定の
     重大度、ドキュメント URL）。
   - 依存: 1。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「Diagnostic Registry」、
     [22.error_handling_and_diagnostics.md](../../../spec/ja/22.error_handling_and_diagnostics.md)。

3. **レジストリの実装。** [ ]
   - 互換性検証（コードは別の意味で決して再利用されない）と、割り当て
     済みコードを固定するレジストリ整合性テストを備えたコードレジストリを
     実装する。
   - テスト: 割り当て/retirement のフィクスチャ。再利用の試みの失敗。
     参照メタデータのラウンドトリップ。
   - 依存: 2。仕様: `registry.md`。

4. **仕様: `failure_record.md`。** [ ]
   - レコードの仕様を執筆する（英語と日本語、コードなし）:
     `DiagnosticDraft` と `DiagnosticRecord` の形、アーキテクチャ 19 に
     従う安定した failure カテゴリ、primary/secondary スパン、構造化
     詳細、機械可読ペイロードの規則。
   - 依存: 2。仕様: [19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)、
     [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)。

5. **レコードとドラフトの実装。** [ ]
   - スパン・詳細テーブルと決定的 debug レンダリングを備えたドラフトと
     レコードを実装する。
   - テスト: レコードのラウンドトリップ。スパンは必ず `SourceId` を参照
     する。レンダリングの安定性。
   - 依存: 3、4。仕様: `failure_record.md`。

### 生産と集約

6. **仕様: `sink.md`。** [ ]
   - 生産者 API の仕様を執筆する（英語と日本語、コードなし）:
     `DiagnosticSink`、phase 側のドラフト発行規則、生産者がしてはなら
     ないこと（CLI 整形なし、LSP 形なし）。
   - 依存: 4。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「Diagnostic Producer API」。

7. **sink の実装。** [ ]
   - 集約に備えた phase ごとのドラフト収集を持つ sink を実装する。
   - テスト: 模擬 phase をまたぐ sink のフィクスチャ。ドラフトが無変更で
     保存される。
   - 依存: 5、6。仕様: `sink.md`。

8. **仕様: `aggregator.md`。** [ ]
   - 集約の仕様を執筆する（英語と日本語、コードなし）: 正規化、識別の
     割り当て、重複排除、正準ソート順、`BuildDiagnosticIndex`、古い
     snapshot の規則（古い snapshot 由来の診断を現在のものとして公開
     しない）。
   - 依存: 4。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「Diagnostic Aggregator」、アーキテクチャ 19。

9. **集約の実装。** [ ]
   - 生産順に依存しない決定的順序を持つ不変の `BuildDiagnosticIndex` への
     集約を実装する。
   - テスト: 入力をシャッフルしても同一の索引。重複排除のフィクスチャ。
     古い snapshot の拒否。
   - 依存: 7、8。仕様: `aggregator.md`。

### 表示

10. **仕様: `render.md`。** [ ]
    - CLI レンダリングの仕様を執筆する（英語と日本語、コードなし）:
      メッセージレイアウト、スパン抜粋、重大度のスタイル、レンダリングは
      コードメタデータをキーにするという規則。
    - 依存: 8。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)、
      アーキテクチャ 12。

11. **CLI レンダリング。** [ ]
    - レコードと行マップから決定的な CLI レンダリングを実装する。
    - テスト: golden ファイルのレンダリングフィクスチャ。バイト同一の
      出力。
    - 依存: 9、10。仕様: `render.md`。

12. **仕様: `fix.md`。** [ ]
    - fix 提案の仕様を執筆する（英語と日本語、コードなし）: 構造化された
      編集提案、適用可能性レベル、安全規則（提案は決して自動適用され
      ない）。
    - 依存: 4。仕様: アーキテクチャ 12「Fix Suggestion」。

13. **fix 提案。** [ ]
    - レコードに付く構造化 fix ペイロードを実装する。
    - テスト: fix のラウンドトリップ。編集が有効な範囲を参照する。
    - 依存: 5、12。仕様: `fix.md`。

14. **仕様: `explain.md`。** [ ]
    - explanation の仕様を執筆する（英語と日本語、コードなし）: 遅延
      explanation ハンドル、上限付きプレビュー、「大きな trace は
      artifact または専用ファイルに留まる」という規則。
    - 依存: 4。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
      「Explanation Store」。

15. **explanation ストア。** [ ]
    - 遅延解決と上限付きプレビューを備えた explanation ストアを実装する。
    - テスト: ハンドル解決のフィクスチャ。プレビュー上限の強制。裏付け
      データ欠落時の明確な劣化。
    - 依存: 13、14。仕様: `explain.md`。

### 採用とフォローアップ

16. **消費者の採用と移行の決定。** [ ]
    - 最初の消費者（`mizar-resolve`）を sink と集約に接続する。既存の
      lexer/frontend/parser 診断の移行決定を解決し、ここととトップ
      レベルに記録する。
    - テスト: resolver の診断がエンドツーエンドで流れる。（選んだ場合）
      変換アダプターのラウンドトリップ。
    - 依存: 9、`mizar-resolve` task 15。仕様: `aggregator.md`。

17. **決定性スイート。** [ ]
    - 同一入力が同一のレコード、索引、レンダリング出力、explanation
      プレビューを生むことのプロパティ的検証。
    - 依存: 11、15。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

18. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。重大度と
      カテゴリの enum はさらにアーキテクチャ 19 の互換性ポリシーに従う。
    - 依存: 16。仕様: 全モジュール仕様。

19. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 18。仕様: 全モジュール仕様と本 TODO。

20. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-diagnostics/en/` の各英語正本と日本語版を比較し、
      内容を同期する。
    - 依存: 19。仕様: リポジトリのドキュメント方針。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-diagnostics
cargo clippy -p mizar-diagnostics --all-targets -- -D warnings
```

採用/移行のタスクでは消費側も実行する:

```text
cargo test -p mizar-resolve
cargo test -p mizar-frontend
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- ツールは `DiagnosticCode` をキーにする。メッセージはバージョン間で
  改善してよいが、コードは恒久的で、別の意味での再利用は決してない。
- 集約の出力は不変かつ決定的順序であり、生産順や並列性が透けて見える
  ことはない。
- open バッファ（LSP オーバーレイ）診断はレコードの形を再利用するが、
  CLI 出力や `VerifiedArtifact` には決して公開されない。ブリッジの
  ロジックは `mizar-lsp` にある。
- 大きな trace が診断にインラインで置かれることは決してない — コンパクトな
  参照と上限付きプレビューのみ。
