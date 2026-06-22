# mizar-artifact TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様は専用の仕様タスクが（英語と日本語を同じ変更で）執筆し、
それを引用する実装タスクより前に追加する。完了した仕様タスクは、対応する
source task が始まる前に file を追加する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割
（に加えて resolver と checker が消費する summary スキーマ）に従う。
この crate はアーキテクチャ 11、18 と internal 02、06 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| store | `store.md`（task 2） | `src/store.rs` | [x] |
| module_summary | `module_summary.md`（task 4） | `src/module_summary.rs` | [x] |
| registration_summary | `registration_summary.md`（task 6） | `src/registration_summary.rs` | [x] |
| proof_witness | `proof_witness.md`（task 8） | `src/proof_witness.rs` | [x] |
| verified_artifact | `verified_artifact.md`（task 10） | `src/verified_artifact.rs` | [x] |
| manifest | `manifest.md`（task 12） | `src/manifest.rs` | [~] |

`mizar-artifact` はパイプラインの安定した外部射影を所有する: artifact
スキーマ（`ModuleSummary`、`RegistrationSummary`、`ProofWitnessRef`、
`VerifiedArtifact`）、atomic write を備えた artifact ストア、manifest
トランザクション（phase 15）。意図的に低依存のスキーマ/ストア crate で
あり、生産者（`mizar-resolve`、`mizar-checker`、`mizar-vc`、
`mizar-kernel`、`mizar-proof`）が射影を構築するためにこの crate に依存する
のであって、逆は決してない。2 つの波で構築する: **第 A 波**（スキーマと
リーダー）はクロスモジュール解決が `ModuleSummary` artifact を再利用する
ため早期に着地させる。**第 B 波**（ストア、manifest トランザクション、
完全な emission）は証明出力が存在するようになってから phase 15 を完成
させる。

依存順序: `store` 基盤 → summary スキーマ（`module_summary`、
`registration_summary`）→ `proof_witness` / `verified_artifact` →
`manifest` / emission。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session` にのみ依存する。第 A 波には他のゲートが
なく、`mizar-resolve` と並行して開始できる。第 B 波の emission 統合は
kernel/proof の出力の存在にゲートされる。アーキテクチャ:
[11.artifact_and_incremental_build.md](../../architecture/ja/11.artifact_and_incremental_build.md)、
[18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。
internal: [02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)、
[06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)。

## 解決済みおよび保留中の決定

- **依存方向: internal 02/06 により解決済み。** この crate は leaf の
  スキーマ/ストア crate である。生産者 crate がこれに依存し、internal 06
  の射影境界が生産者の構築した射影から `VerifiedArtifact` を組み立てる。
  コンパイラ内部 IR がスキーマに現れることは決してない。
- **キャッシュの所有権: internal 07 により解決済み。** キャッシュキー、
  依存 fingerprint、proof 再利用の検証、cluster-db ストレージは
  `mizar-cache` に属する。この crate は artifact スキーマ、artifact
  ストア、manifest トランザクションを所有する。両者はここで定義する
  正準ハッシュ規則（task 2）を共有する。
- **ハッシュの分離: internal 07 の制約により解決済み。** 意味論ハッシュと
  診断/開発ハッシュは分離され、ローカルに変動するフィールド（`verified_at`
  など）は正準ハッシュから除外される（task 2 が両方を符号化する）。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-artifact` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 第 A 波: 正準形と summary スキーマ

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session` にのみ依存する workspace メンバー `mizar-artifact` を
     追加し、`mizar-frontend` のガードに倣った `tests/lint_policy.rs` を
     追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: なし。仕様: アーキテクチャ 11。

2. **仕様: `store.md`。** [x]
   - ストア/正準形の仕様を執筆する（英語と日本語、コードなし）: internal
     02 に従うストアレイアウト、決定的順序を持つ正準 UTF-8 JSON
     シリアライゼーション、`schema_version` と互換性チェック、意味論
     ハッシュ対診断ハッシュの分離、ハッシュ除外のローカルフィールド、
     atomic write の要件。
   - 依存: 1。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)、
     アーキテクチャ 11「Deterministic Artifact Output」「Atomic Writes」。

3. **正準シリアライゼーションと schema-version チェック。** [x]
   - 全スキーマが共有する正準シリアライゼーション、ハッシュ規則、
     schema-version 互換性チェックを実装する。
   - テスト: 実行/プラットフォームをまたぐバイト同一のシリアライ
     ゼーション。バージョン不一致の検出。除外フィールドがハッシュに
     影響しない。
   - 依存: 2。仕様: `store.md`。

4. **仕様: `module_summary.md`。** [x]
   - `ModuleSummary` スキーマの仕様を執筆する（英語と日本語、コード
     なし）: アーキテクチャ 03「Module Summary」に従う export 済み
     インターフェース射影。`interface_hash` でキー付けされ
     （アーキテクチャ 18）、resident-set 規則に従って本体と証明を
     含まない。
   - 依存: 2。仕様: アーキテクチャ 03、
     [18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。

5. **`ModuleSummary` スキーマ、writer、reader。** [x]
   - writer と検証つき reader を備えたスキーマを実装する。これにより
     `mizar-resolve` task 24（summary 経由の解決）のブロックが外れる。
   - テスト: ラウンドトリップ、deterministic canonical ordering、
     body-only/source-metadata change に対する interface-hash stability、
     exported interface change による interface-hash change、
     incompatible-version read、module/hash mismatch rejection。
   - 依存: 3、4。仕様: `module_summary.md`。

6. **仕様: `registration_summary.md`。** [x]
   - `RegistrationSummary` スキーマの仕様を執筆する（英語と日本語、
     コードなし）: アーキテクチャ 04 と 17 に従う export 済み
     registration と cluster trace artifact への参照。
   - 依存: 2。仕様: アーキテクチャ 04、
     [17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)。

7. **`RegistrationSummary` スキーマ、writer、reader。** [x]
   - checker のクロスモジュール再利用のために、writer と検証つき reader を
     備えたスキーマを実装する。
   - テスト: ラウンドトリップ。trace 参照がハッシュで解決される。決定的な
     順序。
   - 依存: 3、6。仕様: `registration_summary.md`。

### 第 A/B 波の境界: witness と verified artifact

8. **仕様: `proof_witness.md`。** [x]
   - proof witness 参照の仕様を執筆する（英語と日本語、コードなし）:
     ハッシュで参照される witness ファイル、kernel 受理メタデータ、
     「witness は主 artifact の外部に留まる」規則（resident-set 規律）。
   - 依存: 2。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)、
     [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
     「Proof Witness Store」。

9. **`ProofWitnessRef` スキーマ。** [x]
   - ハッシュ検証つきの witness 参照を実装する。
   - テスト: ラウンドトリップ。ハッシュ不一致の検出。
   - 依存: 3、8。仕様: `proof_witness.md`。

10. **仕様: `verified_artifact.md`。** [x]
    - `VerifiedArtifact` スキーマの仕様を執筆する（英語と日本語、コード
      なし）: export、式メタデータ、義務状態、witness 参照、射影された
      診断、互換性ポリシー。
    - 依存: 4、8。仕様: アーキテクチャ 11「Verified Artifact Schema」、
      [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)。

11. **`VerifiedArtifact` スキーマと射影入力。** [x]
    - スキーマと、生産者 crate が埋める射影入力契約を実装する（実際の
      生産者が揃うまではテスト用スタブ生産者を使う）。
    - テスト: ラウンドトリップ。schema-version compatibility。source-range
      validation。hash class/domain validation と hash participation。accepted witness
      reference が composite obligation fingerprint と一致すること。生 IR 形の
      payload と ownership-boundary field の拒否。射影された診断が安定したコード、
      nullable range、related entry、順序を保つ。
    - 依存: 9、10。仕様: `verified_artifact.md`。

### 第 B 波: ストア、manifest、emission

12. **仕様: `manifest.md`。** [x]
    - manifest の仕様を執筆する（英語と日本語、コードなし）: パッケージ
      artifact manifest、manifest トランザクションプロトコル
      （begin/commit、reader の可視性）、中断されたコミットからの回復。
    - 依存: 2。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
      「Manifest Transaction」、アーキテクチャ 11「Artifact Manifest」。

13. **atomic write を備えた artifact ストア。** [x]
    - ストアを実装する: 安定した公開 artifact 書き込み、witness など
      schema が要求する hash-addressed published file、temp-and-rename の
      原子性、読み込み時の破損検出。中断された書き込みが完全な出力に
      見えることは決してない。internal cache blob は `mizar-cache` が
      所有し続ける。
    - テスト: 書き込み中 kill のフィクスチャが可視の部分 artifact を
      残さない。破損した artifact または witness の読み込みが位置付きで
      失敗する。
    - 依存: 3、12。仕様: `store.md`、`manifest.md`。

14. **manifest トランザクション。** [ ]
    - 決定的な項目順と reader 側検証を備えたトランザクショナルな
      manifest 更新を実装する。
    - テスト: 並行 reader は旧か新を見る（混在しない）。コミットの再生は
      冪等である。
    - 依存: 13。仕様: `manifest.md`。

15. **来歴と再現性メタデータ。** [ ]
    - 発行された artifact に付くビルド来歴レコード（ツールチェーン、
      edition、設定、依存ハッシュ）を実装する。ハッシュ除外のローカル
      フィールドは task 2 に従って扱う。
    - テスト: 来歴のラウンドトリップ。ローカルフィールドが正準ハッシュに
      影響しない。
    - 依存: 11、14。仕様: アーキテクチャ 11「VerifiedArtifact Is a
      Projection」、[18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。

16. **interface/implementation ハッシュ入力。** [ ]
    - `mizar-cache` の fingerprint が消費する interface-hash と
      implementation-hash の入力を計算・公開する（アーキテクチャ 18）。
      ハッシュごとに正準順序を文書化する。
    - テスト: implementation のみの編集に対する interface ハッシュの
      安定性。ハッシュ入力順序の決定性。
    - 依存: 15。仕様: アーキテクチャ 11「Interface Hashes and
      Implementation Hashes」。

17. **phase 15 emission 統合。** [ ]
    - kernel/proof の出力が存在するようになったら、実際の生産者射影から
      完全な `VerifiedArtifact` emission を接続する。emission はストアと
      manifest トランザクションのみを通す。
    - テスト: 小さな検証済みモジュール上のエンドツーエンド emission
      フィクスチャ。再 emission がバイト同一である。
    - 依存: 14、15、`mizar-kernel` task 16、`mizar-proof` task 11
      （witness の stage/公開）。仕様: `verified_artifact.md`、
      `manifest.md`。

### 強化と横断フォローアップ

18. **決定性スイート。** [ ]
    - 同一入力が実行/プラットフォームをまたいでバイト同一の artifact、
      manifest、ハッシュを生むことのプロパティ的検証。
    - 依存: 16。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

19. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。
      スキーマ enum はさらに artifact 互換性ポリシーに従う。
    - 依存: 16。仕様: 全モジュール仕様。

20. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 19。仕様: 全モジュール仕様と本 TODO。

21. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-artifact/en/` の各英語正本と日本語版を比較し、
      内容を同期する。
    - 依存: 20。仕様: リポジトリのドキュメント方針。

22. **module 境界リファクタリング gate。** [ ]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 21。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-artifact
cargo clippy -p mizar-artifact --all-targets -- -D warnings
```

他所で消費されるスキーマを変更するタスクでは消費側も実行する:

```text
cargo test -p mizar-resolve
cargo test -p mizar-checker
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- 公開される artifact は安定した射影であり、生 IR ダンプでは決してない。
  内部 IR はバージョン間で変わってよいが、スキーマは（`schema_version`
  処理なしには）変わってはならない。
- ポータブルな artifact 内のパスはすべてパッケージ相対または workspace
  相対であり、artifact はコンパイラ内部のキャッシュレコードなしに読めな
  ければならない。
- 第 A 波はクロスモジュール解決のクリティカルパス上にある。小さく保ち、
  早く着地させる。
- キャッシュレコード、キャッシュキー、proof 再利用の検証は `mizar-cache`
  にあり、ここにはない。共有される正準ハッシュ規則は task 2 のものである。
