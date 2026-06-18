# mizar-ir TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割
（`storage`、`identity`）に、internal 06 の publisher / cache adapter /
projection サービスを加えたものに従う。この crate はアーキテクチャ 01、
18 と internal 06 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| identity | `identity.md`（task 2） | `src/identity.rs` | [ ] |
| storage | `storage.md`（task 4） | `src/storage.rs` | [ ] |
| publisher | `publisher.md`（task 7） | `src/publisher.rs` | [ ] |
| cache_adapter | `cache_adapter.md`（task 9） | `src/cache_adapter.rs` | [ ] |
| projection | `projection.md`（task 11） | `src/projection.rs` | [ ] |

`mizar-ir` はコンパイラ内部の IR ストレージと snapshot ハンドルを所有
する: phase 出力の不変ストレージスロット、型付き `PhaseOutputRef<T>`
ハンドル、snapshot ハンドルレジストリ（`BuildSnapshotId` と snapshot
ごとの識別割り当て）、出力を seal する phase output publisher、出力を
キャッシュレコードと相互変換する cache adapter、そして seal された内部
IR を `VerifiedArtifactDraft` へ変える artifact 射影境界。resident-set
規律を実装する: インターフェースと索引は常駐に保ち、大きな出力は
content-addressed blob へ退避し、参照されない出力は回収する。

依存順序: `identity` → `storage` → `publisher` → `cache_adapter` /
`projection`。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session`（snapshot とソースの識別）と、射影境界の
ために `mizar-artifact`（ドラフトスキーマ）に依存する。cache adapter は
seam を通じて `mizar-cache` と統合し、モックキャッシュでテスト可能で
ある。`mizar-driver` と `mizar-build` のスケジューリング波とともに
パイプラインへ接続される。phase サービス自体はこの crate から独立を
保つ（ストレージ内部ではなくコンテキストハンドルを受け取る）。
アーキテクチャ: [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)、
[18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。
internal: [06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)。

## 解決済みおよび保留中の決定

- **blob 退避の閾値: 未解決。task 6 で解決する。** phase 出力をメモリに
  置くか content-addressed blob として置くか（サイズ閾値、IR ごとの
  上書き）を決め、ポリシーを `storage.md` に記録する。internal 概要の
  resident-set 規則は制約であり、性能保証ではない。
- **編集下の識別の安定性: アーキテクチャ 01 により解決済み。** id は
  同一入力に対して決定的であり、編集で完全な安定が不可能な場合は予測
  可能に劣化し、arena インデックスは安定 API として決して公開されない。
  `identity.md` がこれを再掲する。
- **キャッシュの権威: internal 06 により解決済み。** キャッシュヒットは
  決して証明の権威ではない。adapter はハンドル再構築の前に検証し、
  証拠クラスの昇格を拒否する。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-ir` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 識別とストレージ

1. **crate の足場と lint 方針のガード。** [ ]
   - `mizar-session` に依存する workspace メンバー `mizar-ir` を追加し、
     `mizar-frontend` のガードに倣った `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: なし。仕様: internal 06。

2. **仕様: `identity.md`。** [ ]
   - 識別の仕様を執筆する（英語と日本語、コードなし）: ソース/依存/
     ツールチェーン/設定の正確な状態からの `BuildSnapshotId` 割り当て、
     snapshot ごとの id 族（`SourceId`、`ModuleId`、`ItemId`、`ExprId`、
     `VcId`）、親/派生の出力関係、非互換 snapshot をまたぐ再利用禁止の
     規則。
   - 依存: 1。仕様: [internal 06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)
     「Snapshot Handle Registry」、アーキテクチャ 01「Cross-Layer
     Identity」。

3. **snapshot ハンドルレジストリ。** [ ]
   - 決定的な id 割り当てと親/派生の追跡を備えたレジストリを実装する。
   - テスト: 同一状態に対する id の決定性。非互換 snapshot の再利用の
     拒否。派生リンクのラウンドトリップ。
   - 依存: 2。仕様: `identity.md`。

4. **仕様: `storage.md`。** [ ]
   - ストレージの仕様を執筆する（英語と日本語、コードなし）: 不変
     ストレージスロット、型付き `PhaseOutputRef<T>`、seal の意味論、
     メモリ対 blob の配置（退避閾値の決定を含む）、batch/watch/LSP を
     またぐ `retain`/`collect` のライフタイム規則。
   - 依存: 2。仕様: [internal 06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)
     「IR Storage Service」。

5. **ストレージスロットと seal。** [ ]
   - スロット割り当て、seal、型付きハンドルの返却を実装する。seal 済み
     出力は不変であり、未 seal の出力は他タスクから見えない。
   - テスト: 二重 seal の拒否。seal 前アクセスの失敗。ハンドル型付けの
     ラウンドトリップ。
   - 依存: 3、4。仕様: `storage.md`。

6. **content-addressed blob と回収。** [ ]
   - 閾値決定に従う blob 退避と、参照追跡（依存タスク、LSP snapshot、
     診断、explanation リクエスト、キャッシュ writer）上の
     `retain`/`collect` を実装する。
   - テスト: ハッシュによる退避のラウンドトリップ。collect が参照されない
     出力だけを正確に落とす。retain された出力はセッション置換を
     生き延びる。
   - 依存: 5。仕様: `storage.md`。

### 公開とアダプター

7. **仕様: `publisher.md`。** [ ]
   - publisher の仕様を執筆する（英語と日本語、コードなし）: snapshot/
     work-unit の検証、正準エンコーディングによる内容ハッシュ、ソース
     マップと診断サイドテーブルの添付、obsolete snapshot の publication
     拒否、open-buffer output の非公開、部分 IR 非公開の規則。
   - 依存: 4。仕様: [internal 06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)
     「Phase Output Publisher」。

8. **phase output publisher。** [ ]
   - phase サービスが使う狭い seal API を実装する。
   - テスト: 誤った snapshot または obsolete snapshot への公開の拒否。
     ハッシュの安定性。ハンドルからサイドテーブルを取得できる。
   - 依存: 5、7。仕様: `publisher.md`。

9. **仕様: `cache_adapter.md`。** [ ]
   - cache adapter の仕様を執筆する（英語と日本語、コードなし）: どの
     出力がキャッシュ可能か、schema version と依存サマリー付きの
     レコードシリアライゼーション、ハンドル再構築前のヒット検証、
     証明の権威としない規則。architecture 22 の規則として、不完全な
     dependency footprint と `uncacheable` marker は `PhaseOutputRef` を
     再構築する前に miss を強制することも含める。
   - 依存: 7。仕様: [internal 06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)
     「IR Cache Adapter」、[internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)。

10. **cache adapter。** [ ]
    - キャッシュ seam（`mizar-cache` 着地まではモックキャッシュ）の背後で
      レコード変換とヒットの再水和を実装する。
    - テスト: モックキャッシュ経由のラウンドトリップ。無効ヒット、不完全な
      dependency footprint、`uncacheable` record の拒否。再水和されたハンドルが
      元と等しい。
    - 依存: 8、9。仕様: `cache_adapter.md`。

11. **仕様: `projection.md`。** [ ]
    - 射影の仕様を執筆する（英語と日本語、コードなし）: artifact 射影
      境界 — export されたシンボル、正規化されたシグネチャ、証明状態と
      witness 参照、診断と explanation 参照 — および生の
      `SurfaceAst`/`TypedAst`/`CoreIr`/`ControlFlowIr`/`VcIr`/`AtpProblem`
      や kernel 内部状態の公開禁止。
    - 依存: 7。仕様: [internal 06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)
      「Artifact Projection Boundary」。

12. **artifact 射影サービス。** [ ]
    - `mizar-artifact` のスキーマを使って、seal 済み出力から
      `VerifiedArtifactDraft` 値への射影を実装する。
    - テスト: スタブ phase 出力による射影フィクスチャ。生 IR の漏出が
      射影を失敗させる。
    - 依存: 8、11、`mizar-artifact` task 11。仕様: `projection.md`。

13. **watch/LSP の snapshot 置換。** [ ]
    - snapshot 置換を実装する: 新しい snapshot が古いものに取って代わり、
      retain された参照は解放まで古い出力を生かしておく。古い出力は
      読み取り可能なまま、または検証済み cache input になり得るが、
      supersession 後に current result として公開してはならない。
    - テスト: 置換のフィクスチャ。古いハンドルは解放まで読め、その後
      回収される。supersede された出力は current publication として拒否される
      が、cache validation には残せる。
    - 依存: 6、8。仕様: [internal 06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)
      「Snapshot Replacement for Watch and LSP」。

### 強化と横断フォローアップ

14. **決定性とライフタイムのプロパティスイート。** [ ]
    - プロパティ的検証: 同一入力が同一の id とハッシュを生む。
      collect 後の使用がない。collect は冪等。ランダム化された
      retain/release 列の下で参照カウントが漏れない。
    - 依存: 13。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

15. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。
    - 依存: 12。仕様: 全モジュール仕様。

16. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 15。仕様: 全モジュール仕様と本 TODO。

17. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-ir/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 16。仕様: リポジトリのドキュメント方針。

18. **architecture-22 フォローアップ監査。** [ ]
    - architecture 22 のために追加した publisher、cache-adapter、
      snapshot-replacement 契約について、ソース/仕様対応監査と二言語
      ドキュメント同期監査を再実行する。obsolete output は current として
      公開できず、open-buffer output は package artifact にならず、古い出力は
      検証済み cache input としてのみ使えることを確認する。
    - 依存: 10、13、14、17。仕様: 全モジュール仕様、本 TODO、リポジトリの
      ドキュメント方針。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-ir
cargo clippy -p mizar-ir --all-targets -- -D warnings
```

射影のタスクでは追加で実行する:

```text
cargo test -p mizar-artifact
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- seal 済み出力は不変である。作りかけの IR が他タスクから見えることは
  決してない。
- キャッシュヒットは最適化の結果であり、決して証明の権威ではない。
  再水和が証拠クラスを昇格させることはない。
- キャッシュレコードは生の内部 IR エンコーディングを含んでよいが、公開
  artifact は含んではならない — 射影境界がこれを強制する。
- resident-set 規律（インターフェースは常駐、本体は遅延、witness は
  外部）は定性的であり、予算とベンチマークはテスト戦略にある。
