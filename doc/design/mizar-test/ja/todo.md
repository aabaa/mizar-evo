# mizar-test TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

パイプライン crate と異なり、この crate のモジュール仕様は既に存在する。
以下のタスクは仕様に対して実装し、ギャップを閉じる。この crate は
[internal 07](../../internal/ja/07.crate_module_layout.md) に従い
[architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)
を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| layout | [layout.md](./layout.md) | `src/layout.rs`、`src/path_rules.rs` | [~] |
| expectation_schema | [expectation_schema.md](./expectation_schema.md) | `src/expectation.rs` | [~] |
| staged_model | [staged_model.md](./staged_model.md) | `src/staged_model.rs` | [~] |
| traceability | [traceability.md](./traceability.md) | `src/traceability.rs` | [~] |
| harness | [harness.md](./harness.md) | `src/harness.rs`、`src/main.rs` | [~] discovery + `plan` モード |
| miz_corpus | [miz_corpus.md](./miz_corpus.md) | `tests/` 配下のコーパスツリー | [~] |
| snapshot | [snapshot.md](./snapshot.md) | `src/snapshot.rs` | [ ] |
| fail_soundness | [fail_soundness.md](./fail_soundness.md) | ハーネス規則＋コーパスケース | [ ] |
| minimal_crate | [minimal_crate.md](./minimal_crate.md) | crate 境界＋CLI | [~] |

`mizar-test` はコーパスとハーネスの crate である: テスト発見、
`.expect.toml` の expectation 構文解析、staged model、仕様カバレッジの
traceability、snapshot 比較、fail/健全性契約。意図的に最小である
（[minimal_crate.md](./minimal_crate.md)）: 所有するのは検証と計画で
あって、パイプラインの実行ではない — stage ランナーは消費側 crate の
統合テストに住む（先例: frontend の lexical コーパスランナー。
parse-only ランナーの場所は `mizar-parser` task 3 が決める）。

以下の各タスクは意図的に小さくしてある — 既存仕様に対する 1 挙動
スライス — 。これにより、crate の残りを抱え込まずに 1 タスクを単独で
実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は [minimal_crate.md](./minimal_crate.md) に従って依存集合を
最小に保つ。パイプライン crate には依存せず、消費側 crate が発見・
expectation・検証のためにこの crate に依存する。コーパスとカバレッジの
成長は消費側 crate のランナータスク（`mizar-parser` task 3、
`mizar-resolve` task 23、`mizar-checker` task 12/29、`mizar-vc` task 15、
`mizar-atp` task 20、`mizar-kernel` task 17）が律速する。

## 解決済みおよび保留中の決定

- **パイプライン非依存: [minimal_crate.md](./minimal_crate.md) により
  解決済み。** stage 実行は消費側 crate に住む。この crate は検証・
  計画・比較・報告を行う。
- **コーパスランナーの場所: `mizar-parser` task 3 が所有する**（後続
  stage は対応するタスクが所有する）。いずれにせよこの crate は発見と
  expectation を提供する。
- **snapshot 更新メカニズム: 未解決。task 5 で解決する。** ベースラインの
  （再）生成方法 — 明示的な update モード対環境フラグ — を
  [snapshot.md](./snapshot.md) の更新ポリシーの範囲内で決め、そこに
  記録する。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-test` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 基盤

1. **lint 方針のガード。** [ ]
   - `mizar-frontend` のガードに倣った `tests/lint_policy.rs`（workspace
     lint へのオプトイン、deny ベースライン、将来の `allow` の隣に根拠）を
     追加する。
   - テスト: lint 方針ガードが通る。
   - 依存: なし。仕様: リポジトリの慣行。

2. **ソース/仕様ギャップ監査と状態の同期。** [ ]
   - 9 本のモジュール仕様の Public API と Tests の約束を現在の実装へ
     トレースする。ギャップを本 TODO のフォローアップタスクとして記録し、
     モジュール表の状態を実態に合わせる。
   - 依存: 1。仕様: 全モジュール仕様。

3. **ランナーモードと CLI の完成。** [ ]
   - [minimal_crate.md](./minimal_crate.md)「CLI」「Exit Codes」と
     [harness.md](./harness.md)「Runner Modes」に従い、`plan` を超えて
     CLI を完成させる: コーパスツリーとカバレッジマニフェスト上の
     検証モードと、文書化された終了コード。
   - テスト: モードごとの CLI フィクスチャ。終了コードが仕様の表と
     一致する。決定的な出力。
   - 依存: 2。仕様: `minimal_crate.md`、`harness.md`。

### snapshot 対応

4. **snapshot モジュール: API と正準化。** [ ]
   - [snapshot.md](./snapshot.md) の snapshot 種別、公開 API、正準化
     規則（安定パス、改行正規化、非決定的フィールドなし）を実装する
     `src/snapshot.rs` を追加する。
   - テスト: 正準化のフィクスチャ。比較失敗が正確な diff を保持する。
   - 依存: 2。仕様: [snapshot.md](./snapshot.md)「Public API」
     「Canonicalization」。

5. **snapshot の更新ポリシーと決定性チェック。** [ ]
   - ベースライン更新フロー（更新メカニズムの決定を解決する）と
     [snapshot.md](./snapshot.md) の決定性チェック（再レンダリング
     比較）を実装する。
   - テスト: 更新フローのラウンドトリップ。誤更新からの保護。決定性
     チェックが注入された非決定性を捕まえる。
   - 依存: 4。仕様: [snapshot.md](./snapshot.md)「Update Policy」
     「Determinism Checks」。

### カバレッジと健全性の契約

6. **カバレッジと pass/fail 比率の報告。** [ ]
   - 既存の traceability と発見データから、stage ごとの仕様トレース
     カバレッジと、テスト戦略の 40/60 目標に対するコーパスの pass/fail
     比率を報告する。
   - テスト: 合成コーパス上の報告フィクスチャ。決定的な報告バイト列。
   - 依存: 3。仕様: [traceability.md](./traceability.md)、
     [architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

7. **stage 前提条件の検証。** [ ]
   - staged model の規則を強制する: ケースの stage 前提条件がカバー済み
     または built-in 宣言済みになるまで、カバレッジのクレジットを
     与えない。
   - テスト: 前提条件違反のフィクスチャが安定した診断で検証に失敗する。
   - 依存: 6。仕様: [staged_model.md](./staged_model.md)「Stage Rules」。

8. **fail/健全性契約の対応。** [ ]
   - [fail_soundness.md](./fail_soundness.md) の期待失敗契約を実装する:
     ドメインごとの必須ケースの記録、期待失敗のアサーション（diagnostic
     コードと stage）、健全性ケースが黙って削除・弱体化されない
     リグレッション規則。
   - テスト: 契約のフィクスチャ。弱体化の試みの検出。
   - 依存: 6。仕様: [fail_soundness.md](./fail_soundness.md)。

9. **コーパスサイズとレビュー規則の検証。** [ ]
   - [miz_corpus.md](./miz_corpus.md) のコーパス成長規則を検証する:
     ファイルサイズ指針、命名、コーパスクラスの配置、生成ポリシーの
     マーカー。
   - テスト: 規則ごとの違反フィクスチャ。クリーンなコーパスは通る。
   - 依存: 3。仕様: [miz_corpus.md](./miz_corpus.md)。

### 消費側との歩調とフォローアップ

10. **消費側ランナーの支援。** [ ] — 消費側 crate が律速。
    - 各消費側ランナーの着地に合わせて、発見・expectation・stage・
      snapshot・報告を歩調を合わせて維持する（`mizar-parser` task 3、
      `mizar-resolve` task 23、`mizar-checker` task 12/29、`mizar-vc`
      task 15、`mizar-atp` task 20、`mizar-kernel` task 17）。消費側
      1 つにつき 1 増分を独立した変更で行う。最後のランナーが着地した
      時点でチェックを付ける。
    - 所有する pipeline stage がまだ実行できない traceability seed ケースが
      先にコミットされる場合に備え、消費側 runner の active/planned gate を
      明示的に扱う。既定の metadata plan はそのようなケースを発見してよいが、
      消費側 runner は planned seed を実行済み coverage として黙って数えては
      ならない。
    - 依存: 5、8。仕様: [harness.md](./harness.md)。

11. **決定性スイート。** [ ]
    - 発見順、計画、検証診断、報告、snapshot 比較が実行と
      プラットフォームをまたいでバイト安定であることのプロパティ的検証。
    - 依存: 6。仕様: [harness.md](./harness.md)「Determinism
      Requirements」。

12. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum（`Stage`、`ExpectedOutcome`、`ValidationSeverity`、…）に
      `mizar-frontend` task 25 の手続きを適用し、所有モジュール仕様に
      決定を記録する。
    - 依存: 2。仕様: 全モジュール仕様。

13. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-test/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 12。仕様: リポジトリのドキュメント方針。

14. **増分/並列検証 regression matrix。** [ ]
    - architecture 22 の regression matrix のための corpus / harness metadata と
      reporting support を追加する。この crate は pipeline-free のままにする。
      case の実行は consumer crate が所有するが、`mizar-test` は scenario id、
      expected equivalence class、active/planned gating、traceability record を
      所有する。
    - matrix row は次をカバーしなければならない: clean sequential == clean
      parallel、externally visible artifact について clean build == incremental
      build、sequential incremental == parallel incremental、randomized
      ready-task scheduling、randomized ATP backend completion order、cache
      hit/miss timing、`VcId` reorder 時に `ObligationAnchor`、fingerprint、
      policy、witness / discharge hash が一致する場合だけ reuse されること、
      missing dependency slice が cache miss を強制すること、stale snapshot
      diagnostics と obsolete-result non-publication、proof witness mismatch、
      外部認証された証拠の non-upgrade、cache-key race、artifact manifest
      atomicity、registration / cluster invalidation、theorem proof-body と
      theorem-status の invalidation、notation / operator invalidation。
    - 依存: 10、11。仕様:
      [20.test_strategy.md](../../architecture/ja/20.test_strategy.md),
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md)。

15. **architecture-22 フォローアップ監査。** [ ]
    - ソース/仕様ギャップ監査と二言語ドキュメント同期監査を再実行し、
      task 14 の scenario id、equivalence class、active/planned gating、
      traceability record を architecture 22 に照らしてレビューする。残る
      matrix gap をフォローアップタスクとして記録する。
    - 依存: 14。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md),
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md),
      リポジトリのドキュメント方針。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-test
cargo clippy -p mizar-test --all-targets -- -D warnings
```

発見・expectation・stage を変更するタスクでは、コーパスランナーを
組み込む消費側（現状）も実行する:

```text
cargo test -p mizar-frontend
```

architecture-22 regression matrix では、追加する row の active consumer crate
も実行する:

```text
cargo test -p mizar-build
cargo test -p mizar-driver
cargo test -p mizar-cache
cargo test -p mizar-vc
cargo test -p mizar-atp
cargo test -p mizar-proof
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- この crate は最小に保つ: 検証・計画・比較・報告のみ — パイプラインの
  実行もパイプラインへの依存も決して持たない。
- stage id は `.expect.toml`、`spec_trace.toml`、消費側 enum と共有される
  正準値である。表示名はローカライズしてよいが、id はしてはならない。
- kernel の近傍では fail/健全性カバレッジが優先される。40/60 の
  pass/fail 比率はコーパス全体の目標であり、ディレクトリごとではない。
- snapshot ベースラインは内部レンダリングの安定性表面である。
  レンダリング自体は安定 artifact ではない。
