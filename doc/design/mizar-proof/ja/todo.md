# mizar-proof TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割
（`policy`、`witness_store`）に、internal 04 の選択/状態射影を加えたものに
従う。この crate はアーキテクチャ 08、15、19 と internal 04 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| policy | `policy.md`（task 2） | `src/policy.rs` | [~] |
| selection | `selection.md`（task 5） | `src/selection.rs` | [~] |
| status | `status.md`（task 8） | `src/status.rs` | [ ] |
| witness_store | `witness_store.md`（task 10） | `src/witness_store.rs` | [ ] |

`mizar-proof` は untrusted な証拠生産（`mizar-atp`、`mizar-vc` の
discharge）と信頼された検証（`mizar-kernel`）の間のポリシー層を所有する:
`ProofPolicyEvaluator`（候補クラス、外部認証の規則、
`require_kernel_certificates`、ビルドモードごとの open 義務の許容）、
portfolio 候補上の決定的な勝者選択、artifact 向けの証明状態射影
（`kernel_verified`、`discharged_builtin`、ポリシー管理の external と
open 状態）、artifact コミットに向けて stage される proof witness ストア。
ポリシーの結果は信頼された証明状態とは常に区別される: この crate は
何かを「より証明済み」にすることは決してない — 何を記録し、選択し、公開
するかを決めるだけである。

依存順序: `policy` → `selection` → `status` → `witness_store`。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session`、`mizar-kernel`（`KernelCheckResult`、
証明書スキーマ）、`mizar-vc`（`VcId`、discharge 証拠）、`mizar-atp`
（portfolio 候補）、`mizar-artifact`（witness 参照スキーマ）に依存する。
kernel がこの crate に依存することは決してない。アーキテクチャ:
[08.reasoning_boundary.md](../../architecture/ja/08.reasoning_boundary.md)、
[15.kernel_certificate_format.md](../../architecture/ja/15.kernel_certificate_format.md)。
internal: [04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)。

## 解決済みおよび保留中の決定

- **ポリシー/信頼の分離: internal 04 により解決済み。** kernel はポリシー
  非依存の検証結果を返し、この crate がその上でポリシーを評価する。外部
  認証された証拠はポリシーで記録される証拠であって信頼された状態では
  決してなく、`require_kernel_certificates` の下では勝てない。
- **discharge 証拠の検証範囲: task 6 で解決済み。**
  `DischargedBuiltin` は `KernelPolicyInput` から作られた
  `TrustedKernelEvidence` を通じてだけ trusted selection に入る。public caller が
  `KernelPolicyInput` を構築できるのは、`KernelCheckResult` と明示的 origin を
  渡す場合だけである。したがって pre-ATP discharge は kernel replay されるか、
  kernel が accept した primitive evidence として表現されなければならない。
  それ以外は deterministic policy evidence のままであり、trusted `used_axioms` を
  publish できない。
- **ポリシー fingerprint の表面: task 2 で解決済み。task 3 で実装する。**
  `policy.md` は `PolicyFingerprint` に入る設定を定義する。将来の cache
  integration は `mizar-cache` task 2 と調整する。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-proof` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### Protocol prerequisite

0. **Crate plan と task ledger。** [x]
   - paired crate plan と ledger を追加した:
     [00.crate_plan.md](./00.crate_plan.md) と
     [task_ledger.md](./task_ledger.md)。
   - 状態: kickoff として完了。この docs-only task を review、verify、commit
     した後に task 1 から実装を開始する。

### ポリシー

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session`、`mizar-kernel`、`mizar-vc`、`mizar-atp`、
     `mizar-artifact` に依存する workspace メンバー `mizar-proof` を追加
     し、`mizar-frontend` のガードに倣った `tests/lint_policy.rs` を追加
     する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-kernel` task 1、`mizar-atp` task 1。仕様: internal 04。
   - 状態: scaffold crate と lint-policy guard を追加した。policy、selection、
     status、witness-store module は、後続 task で paired spec が追加されるまで
     unavailable のままである。

2. **仕様: `policy.md`。** [x]
   - ポリシーの仕様を執筆する（英語と日本語、コードなし）: verifier
     ポリシー設定、`CandidatePolicyClass`、外部認証の許可規則、
     `require_kernel_certificates`、ビルドモードごとの open 義務の許容、
     「ポリシーの結果は信頼された状態と区別される」規則。
   - 依存: 1。仕様: [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
     「Proof Policy Evaluator」、アーキテクチャ 08。
   - 状態: paired spec を追加した。実装は task 3 で開始する。

3. **ポリシー評価器。** [x]
   - `ProofPolicyEvaluator` を実装する: 候補分類、
     `can_schedule_kernel_check`、`policy.md` が定義する policy fingerprint の射影。
     将来の cache integration は `mizar-cache` task 2 と調整したままにする。
   - kernel result と明示的な evidence origin を対にする、この crate ローカルの
     normalized policy input wrapper を定義する。`KernelCheckResult` だけから
     origin を推測してはならない。
   - この task の external evidence support は base classifier shape のみにする。
     完全な admission matrix と安定した rejection diagnostic は task 4 に残す。
   - テスト: 証拠種別ごとの分類フィクスチャ。fingerprint はポリシーに
     関係する設定が変わったときだけ変わる。
   - 依存: 2。仕様: `policy.md`。
   - 状態: `src/policy.rs` に normalized kernel-origin input、base external
     classifier shape、schedulability check、deterministic policy fingerprint、
     focused test を実装した。

4. **外部認証された証拠の扱い。** [x]
   - 外部認証された証拠の許可とラベル付けを実装する: プロファイルが
     許せば開発証拠として記録可能、`require_kernel_certificates` の下では
     決して勝てない、信頼された `used_axioms` を決して生まない。
   - テスト: プロファイルごとの許可マトリクス。安定した拒否診断。
   - 依存: 3。仕様: `policy.md`（外部認証された証拠の節）。
   - 状態: `ExternalEvidenceAdmission`、具体的な publication label、安定した
     policy diagnostic、policy-tainted kernel result routing、profile/requirement
     matrix test を実装した。

### 選択と状態

5. **仕様: `selection.md`。** [x]
   - 勝者選択の仕様を執筆する（英語と日本語、コードなし）: 決定的な
     順序クラス（active policy を満たす kernel 検証済み → discharged
     built-in → ポリシーが許す external → policy assumption → 最良の説明を持つ
     open）、タイブレークキー（backend プロファイル優先度、証明書
     フォーマット優先度、エンコード済み problem ハッシュ、プロファイル id、
     安定した candidate id）、reuse validation へ export
     される選択済み proof witness hash または deterministic discharge hash、
     完了時刻の使用禁止。
   - 依存: 2。仕様: [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
     「Winner Selection」。
   - 状態: paired spec を追加した。実装は task 6 で開始する。

6. **勝者選択。** [x]
   - `ProofEvidenceSet` 上の決定的な勝者選択を実装する。built-in
     discharge 証拠がどのクラスに入るかを定める discharge 証拠の検証範囲
     決定を（`mizar-kernel` とともに）解決し記録する。
   - テスト: クラスとタイブレークをまたぐ順序フィクスチャ。候補到着を
     シャッフルしても勝者は変わらない。
   - 依存: 3、5、`mizar-kernel` task 16。仕様: `selection.md`。
   - 状態: `src/selection.rs` に required stable candidate id、trusted-kernel
     evidence marker、deterministic winner/rejection ordering、no-selectable
     diagnostic outcome、reuse metadata、focused test を実装した。

7. **artifact proof selection のマージ。** [x]
   - portfolio の結果と phase 12 の built-in discharge の結果を `VcId`
     ごとにマージし、`kernel_verified` / `discharged_builtin` の選択に
     する。external と open の状態は区別可能なまま保つ。
   - テスト: 組み合わせごとのマージフィクスチャ。どの状態も別の状態に
     潰れない。
   - 依存: 6、`mizar-vc` task 12。仕様: `selection.md`（マージの節）、
     [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
     「Artifact Proof Selection」。
   - 状態: `merge_artifact_proof_selections` に canonical `VcId` ordering、
     duplicate-source rejection、trusted class precedence、source/class compatibility
     validation、後続 status projection のための non-trusted outcome preservation
     を実装した。

8. **仕様: `status.md`。** [x]
   - 状態射影の仕様を執筆する（英語と日本語、コードなし）: artifact と
     診断に向けた証明状態モデル、信頼された `used_axioms` の伝播
     （kernel が受理した証拠からのみ）、open/拒否された義務の
     explanation 参照。
   - 依存: 5。仕様: [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
     「Proof Witness and Artifact Flow」、アーキテクチャ 19、アーキテクチャ 22。
   - 状態: paired `status.md` spec を追加し、projection input、selection から status
     への mapping、trusted `used_axioms` boundary、diagnostic/explanation reference、
     artifact projection limit、proof reuse metadata、deferred/external dependency gap
     を定義した。

9. **証明状態の射影。** [ ]
   - artifact と診断のための状態射影を実装する。信頼された
     `used_axioms` の抽出境界を含む。
   - テスト: 選択結果ごとの射影フィクスチャ。`used_axioms` は kernel が
     受理した証拠からのみ。
   - 依存: 7、8。仕様: `status.md`。

### witness ストア

10. **仕様: `witness_store.md`。** [ ]
    - witness ストアの仕様を執筆する（英語と日本語、コードなし）:
      stage/publish のフロー（コミット前に `stage`、artifact manifest が
      witness を参照した後にのみ `publish_ref`）、proof witness hash として
      使う安定した内容ハッシュ、来歴メタデータ。
    - 依存: 2。仕様: [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
      「Proof Witness Store」。

11. **witness ストアの実装。** [ ]
    - `mizar-artifact` の witness 参照スキーマに対する
      `ProofWitnessDraft` の stage と公開を実装する。
    - テスト: stage/publish のラウンドトリップ。manifest 参照前の公開の
      失敗。コミット前のハッシュ記録。
    - 依存: 9、10、`mizar-artifact` task 9。仕様: `witness_store.md`。

12. **portfolio early-stop のポリシーフック。** [ ]
    - ATP portfolio が early stop に使うポリシークエリ（これ以上良い
      クラスが不可能かの判定）を提供する。終了の決定はポリシー駆動で
      あり、時間駆動ではない。
    - テスト: ポリシーごとの early-stop フィクスチャ。停止しても完走
      した場合と選択される勝者が変わらない。
    - 依存: 6、`mizar-atp` task 18。仕様: `policy.md`、
      [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
      「Early Stop and Cancellation」。

### 強化と横断フォローアップ

13. **決定性スイート。** [ ]
    - 同一の証拠集合が、到着順をシャッフルしても同一の分類、勝者、状態、
      witness 参照を生むことのプロパティ的検証。
    - 依存: 11、12。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

14. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。状態
      enum はさらに artifact 互換性ポリシーに従う。
    - 依存: 11。仕様: 全モジュール仕様。

15. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースする。すべてのモジュール仕様がポリシー/信頼の分離を再掲
      していることを検証する。
    - 依存: 14。仕様: 全モジュール仕様と本 TODO。

16. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-proof/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 15。仕様: リポジトリのドキュメント方針。

17. **proof-reuse metadata export 契約。** [ ]
    - `mizar-cache` が消費する proof-reuse metadata を公開する:
      compatible verifier-policy fingerprint、`ObligationAnchor`、canonical VC、
      local-context、dependency-slice fingerprint、選択された proof witness hash
      または deterministic discharge hash、matching proof-evidence identity、
      dependency artifact/schema compatibility、evidence class、selected-candidate
      provenance、selection reason。この metadata は `status.md` と architecture 22
      に従う reuse validation predicate であり、trusted proof status ではない。
    - テスト: export された reuse component のいずれかが変わると reuse
      predicate が変化または無効化される。candidate arrival をシャッフルしても
      同じ metadata が export される。外部認証された証拠は外部認証された
      証拠のままであり、metadata reuse によって昇格しない。
    - 依存: 6、7、9、11、13。仕様:
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md),
      [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md),
      [11.artifact_and_incremental_build.md](../../architecture/ja/11.artifact_and_incremental_build.md)。

18. **architecture-22 フォローアップ監査。** [ ]
    - task 17 の reuse-metadata export 契約について、ソース/仕様対応監査と
      二言語ドキュメント同期監査を再実行する。残る trust-boundary、
      witness-hash、deterministic discharge、policy-selection gap を
      フォローアップタスクとして記録する。
    - 依存: 17。仕様: 全モジュール仕様、本 TODO、リポジトリの
      ドキュメント方針。

19. **module 境界リファクタリング gate。** [ ]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 18。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-proof
cargo clippy -p mizar-proof --all-targets -- -D warnings
```

kernel、ATP、artifact の境界に触れるタスクでは追加で実行する:

```text
cargo test -p mizar-kernel
cargo test -p mizar-atp
cargo test -p mizar-artifact
```

proof-reuse metadata タスクでは追加で実行する:

```text
cargo test -p mizar-cache
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- ポリシーの結果は信頼された証明状態と区別される。この crate が証拠を
  昇格させることは決してない — 信頼の唯一の源は kernel の肯定的結果で
  ある。
- 勝者選択は決定的かつポリシー駆動である。生の完了時刻は記録されるが、
  何かを決めることは決してない。
- 外部認証された証拠は信頼された `used_axioms` を決して生まず、
  `require_kernel_certificates` の下では勝てない。
- witness は artifact manifest が参照して初めて公開到達可能になる。
  stage だけでは何も公開されない。
