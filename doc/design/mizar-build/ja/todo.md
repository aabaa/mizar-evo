# mizar-build TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

完全なモジュール仕様は、それを引用する実装タスクより前に、専用の仕様タスクが
（英語と日本語を同じ変更で）執筆する。`planner` と `module_index` source は
第 A 波の phase-0 planning と module-index provider work を実装済みであり、
第 B 波の module specs と source implementation は `cache_seam` まで存在する。
モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md)（最小:
`task_graph`、`scheduler`、`cancel`、`failure_state`）に、アーキテクチャ 00/03 の
phase 0 計画モジュールを加えたものに従う。この crate はアーキテクチャ 14、
19 と internal 01 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| planner | `planner.md`（task 2） | `src/planner.rs` + private `src/planner/tests.rs` | [x] |
| module_index | `module_index.md`（task 5） | `src/module_index.rs` + private `src/module_index/tests.rs` | [x] |
| task_graph | `task_graph.md`（task 7） | `src/task_graph.rs` + private `src/task_graph/tests.rs` | [x] |
| scheduler | `scheduler.md`（task 9） | `src/scheduler.rs` + private `src/scheduler/tests.rs` | [x] |
| resource | `resource.md`（task 11） | `src/resource.rs` | [x] |
| cancel | `cancel.md`（task 13） | `src/cancel.rs` | [x] |
| failure_state | `failure_state.md`（task 15） | `src/failure_state.rs` | [x] |
| artifact_commit | `artifact_commit.md`（task 17） | `src/artifact_commit.rs` | [x] |
| cache_seam | `cache_seam.md`（task 18） | `src/cache_seam.rs` | [x] |

`mizar-build` は現在パイプライン phase 0（workspace 計画: manifest、
lockfile、依存グラフ、`BuildPlan`、モジュール索引）を実装しており、並列検証の
機構（タスクグラフ、スケジューラ、リソース予算、キャンセル、blocked タスク状態）
を所有し実装していく。スケジューリングは意味論から分離される: 並列性は
レイテンシを変えてよいが、診断順、artifact 順、証明の受理、再現性を決して
変えない。ビルドリクエストと phase サービスレジストリは `mizar-driver` に属し、
driver がこの crate に依存する — 逆は決してない。

2 つの波で構築する: **第 A 波**（planner とモジュール索引、phase 0）は、
resolver のモジュール索引入力が workspace スタブを planner 出力から構築した
実際の module-index provider で置き換えるため早期に着地させる。**第 B 波**
（タスクグラフ、スケジューラ、リソース、キャンセル、failure 状態）は
スケジュール対象の検証 phase とともに到来し、合成タスクに対して開発できる。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

第 A 波は `mizar-session` と仕様第 23 章の manifest フォーマットに依存
する。第 B 波は合成タスクでテスト可能であり、そのコミット境界は
`mizar-artifact` の manifest トランザクションと統合し、cache-aware
スケジューリングは cache internals を再実装せず、task 18 の scheduler seam
を通じて `mizar-cache` を消費する。アーキテクチャ:
[14.parallel_verification_and_scheduling.md](../../architecture/ja/14.parallel_verification_and_scheduling.md)、
[19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)。
internal: [01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)。
仕様: [23.package_management_and_build_system.md](../../../spec/ja/23.package_management_and_build_system.md)。

## 解決済みおよび保留中の決定

- **driver の分離: internal 00/01 により解決済み。** `mizar-driver` が
  ビルドリクエスト、CLI/watch/LSP のエントリポイント、phase レジストリを
  所有する。`mizar-build` は計画とスケジューリングを所有し、エントリ
  ポイントに依存しないままにする。
- **初期タスク粒度: task 7 で解決済み。** `task_graph.md` は VC generation
  までは module-level phase tasks を使い、explicit VC descriptors が利用
  可能になった後だけ VC-level proof tasks を作成する方針を選んだ。
- **cache-aware スケジューリングの時期: task 18 で解決済み。**
  タスク実行前の cache lookup は build-side seam を通じて外部で検証済みの
  cache decision を消費し、将来は driver が所有する必須の `salsa` query 境界
  （`mizar-driver` task 4〜5）から呼び出せなければならない。その外部境界が
  できるまでは、tests は synthetic decisions を使い、`mizar-build` は default で
  uncached のままである。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-build` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 第 A 波: workspace 計画（phase 0）

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session` に依存する workspace メンバー `mizar-build` を追加
     し、`mizar-frontend` のガードに倣った `tests/lint_policy.rs` を追加
     する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: なし。仕様: アーキテクチャ 14、internal 01。

2. **仕様: `planner.md`。** [x]
   - 計画の仕様を執筆する（英語と日本語、コードなし）: 仕様第 23 章に
     従う `mizar.pkg`、`mizar.workspace`、`mizar.lock` のモデル、
     `BuildPlan`（パッケージ、依存グラフ、ツールチェーン、verifier と
     ビルドの設定）、決定的な計画規則。
   - 依存: 1。仕様:
     [23.package_management_and_build_system.md](../../../spec/ja/23.package_management_and_build_system.md)、
     アーキテクチャ 00「Interface Definitions」。

3. **manifest と lockfile の構文解析。** [x]
   - パッケージ/workspace の manifest と lockfile を構文解析・検証し、
     manifest エラー診断を出す。
   - package manifest の `name` spelling 検証は最初の限定スライスとして着地済みで、
     完全な parsing/validation slice も完了した: package id は小文字の `snake_case`
     (`[a-z][a-z0-9]*(?:_[a-z0-9]+)*`) でなければならず、hyphenated spelling
     は拒否され、hyphen から underscore への正規化は行わない。TOML package/workspace
     manifest を検証し、既存 lockfile を parse して package/version/source の
     consistency を検査する。
   - テスト: 有効/無効な manifest のフィクスチャ。lockfile 不一致の
     診断。決定的なエラー順。
   - 依存: 2。仕様: `planner.md`。

4. **依存グラフ解決と `BuildPlan` の生成。** [x]
   - パッケージ依存グラフを解決し（循環の拒否、バージョンと edition の
     検査）、決定的な `BuildPlan` を生成する。
   - テスト: 循環とバージョン競合を含むグラフのフィクスチャ。同一入力が
     同一の計画を生む。
   - 依存: 3。仕様: `planner.md`。

5. **仕様: `module_index.md`。** [x]
   - モジュール索引の仕様を執筆する（英語と日本語、コードなし）:
     アーキテクチャ 03 Step 1 に従うパッケージ → モジュール識別の写像と、
     resolver が消費するプロバイダー契約。
   - 依存: 2。仕様: アーキテクチャ 03「Step 1」。

6. **モジュール索引の構築。** [x]
   - `BuildPlan` とソースレイアウトからモジュール索引を構築し、`module_index.md`
     で定義した build-side provider/accessor contract を公開する。
     resolver stub replacement を試みる前に `mizar-resolve` task 7 を確認する。
     その task がまだ open なら、resolver parity を外部 dependency gap として分類し、
     resolver-owned fixture や compatibility shim を `mizar-build` 内で創作しない。
   - 完了した build-side slice: `ModuleIndex`、package/namespace/module entry、
     dependency-summary-backed module entry、static source layout provider、決定的 diagnostics、
     provider accessor。2026-06-18 の履歴確認では `mizar-resolve` task 7 は open
     だったが、R-007 で resolver-owned seam と workspace stub provider が着地済みである。
     そのため build-side external dependency gap は、resolver fixture や compatibility shim を
     `mizar-build` に追加せずに解消済みである。
   - テスト: 複数パッケージのフィクスチャ。別名に依存しないモジュール
     識別。決定的な source discovery order。resolver stub fixture との
     provider parity は `mizar-resolve` task 7 がその seam を供給した後のみ。
   - 依存: 4、5。resolver stub replacement は追加で `mizar-resolve` task 7 に
     依存する。仕様: `module_index.md`。

### 第 B 波: タスクグラフとスケジューリング

7. **仕様: `task_graph.md`。** [x]
   - タスクグラフの仕様を執筆した（英語と日本語、コードなし）: タスク
     種別、バージョン付きタスク識別、依存辺（モジュール依存が意味論
     phase をゲートする。VC は細粒度タスク）、dependency-coverage handling、
     初期粒度の決定。
   - 依存: 2。仕様: アーキテクチャ 14「Task Graph」「VCs Are Fine-Grained
     Tasks」、[internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)。

8. **タスクグラフの構築。** [x]
   - `BuildPlan`、`ModuleIndex`、dependency overlay、explicit VC descriptors
     をバージョン付きタスクグラフへ展開する。
   - テスト: グラフ展開のフィクスチャ。依存辺がアーキテクチャ境界と
     一致する。決定的な展開。
   - 結果: 決定的な task ID、dependency coverage diagnostics、explicit
     VC/backend/kernel subgraphs、artifact/documentation scheduling edges、
     focused unit tests を備えた `src/task_graph.rs` を実装した。
   - 依存: 7。仕様: `task_graph.md`。

9. **仕様: `scheduler.md`。** [x]
   - スケジューラの仕様を執筆する（英語と日本語、コードなし）: ワーク
     キュー、優先度ポリシー、batch 対 watch/LSP モード、ビルドイベント、
     決定的結果順序の規則（完了順は決して意味論順・artifact 順では
     ない）。
   - 結果: task states、ready queues、priority/collation、scheduler events、
     cache-aware seam boundaries、deferred resource/cancel/failure/commit seams
     を扱う同期済み `scheduler.md` specs を追加した。
   - 依存: 7。仕様: アーキテクチャ 14「Deterministic Result Ordering」、
     [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
     「Pipeline Scheduler」。

10. **スケジューラのコア。** [x]
    - タスクグラフ上の deterministic dispatch batches とキュー実行を、
      任意の完了順の下で決定的な結果順序を保って実装する。テストは
      合成タスクを使う。resource-budgeted worker pools は tasks 11-12 に残す。
    - テスト: 完了順のシャッフルと worker-count variation が同一の結果順・
      イベント順を生成すること。公開された出力の不変性。
    - 結果: 決定的な synthetic scheduling、queue routing、terminal/blocked
      states、canonical event/result collation、disabled cache seam behavior、
      synthetic cancellation、focused unit tests を備えた `src/scheduler.rs`
      を実装した。
    - 依存: 8、9。仕様: `scheduler.md`。

11. **仕様: `resource.md`。** [x]
    - リソース予算の仕様を執筆する（英語と日本語、コードなし）: 階層的
      予算（ビルド → パッケージ → タスク）、ワーカープールのサイズ、
      ATP runner へ渡す外部プロセス制限。
    - 結果: hierarchical budgets、deterministic queue admission、worker pools、
      ATP/backend limits、release accounting、telemetry、non-authority rules を
      扱う同期済み `resource.md` specs を追加した。
    - 依存: 9。仕様: アーキテクチャ 14「Resource Budgets Are
      Hierarchical」。

12. **リソース予算。** [x]
    - スケジューラにおける予算の計上と強制を実装する。
    - テスト: 予算枯渇時はオーバーコミットせずキューに入る。予算が
      workspace/package/module/obligation/backend/commit scopes を通じて
      階層的に合成される。terminal states が正確に一度だけ release する。
      ATP portfolio work が backend process slots を消費しない。backend fanout が
      obligation と global process limits を守る。worker-count changes が canonical
      result/event collation を保つ。shuffled ready/completion order の下でも admission が
      deterministic である。impossible requests が stable diagnostics を生成する。
      telemetry と I/O commit permits が proof、cache、artifact publication、
      trusted-status authority を作らない。`mizar-driver`、`mizar-cache`、
      ATP OS-process、artifact publication token、proof-authority placeholder を
      導入しない。
    - 結果: modeled hierarchical budget accounting、deterministic
      admission/release telemetry、per-pool と per-scope limits、ATP
      portfolio/process separation、backend fanout、commit permits を備えた
      `src/resource.rs` を実装し、driver、cache、OS-process、publication-token、
      proof-authority boundaries を追加せず scheduler admission に統合した。
    - 依存: 10、11。仕様: `resource.md`。

13. **仕様: `cancel.md`。** [x]
    - キャンセルの仕様を執筆する（英語と日本語、コードなし）: 協調的で
      バージョン付きのキャンセルトークン、watch/LSP のための snapshot
      無効化、部分 artifact 禁止の規則。
    - 結果: versioned cancellation、snapshot freshness check、cooperative
      checkpoint、cancelled/obsolete work を current として publish しない規則、
      resource-release handoff、明示的な non-authority/cache/artifact/driver 境界を
      扱う同期済み `cancel.md` specs を追加した。
    - 依存: 9。仕様: アーキテクチャ 14「Cancellation Is Cooperative and
      Versioned」。

14. **キャンセル。** [x]
    - キャンセルトークンと snapshot バージョンの無効化を実装する。
      キャンセルされた作業は決して出力を公開しない。
    - テスト: pending/ready の開始前 cancellation。safe checkpoint での
      running cancellation。monotonic generation / token propagation。canonical
      cancellation-decision ordering。publication 前の stale completed-result
      discard。cancelled work から current diagnostics、cache records、artifact
      commit attempts が出ないこと。exactly-once resource release。modeled
      atomic transaction 開始前/後の commit-boundary 挙動。deterministic /
      idempotent cancellation。driver/cache/IR/process/artifact-token/
      proof-authority placeholder がないこと。
    - 結果: versioned cancellation policy、monotonic generation、token、
      canonical decision、snapshot freshness check、commit-started decision、
      pre-start / checkpoint / obsolete-completed cancellation の scheduler
      integration を備えた `src/cancel.rs` を実装した。driver/cache/IR/process/
      artifact-token/proof-authority placeholder は追加していない。
    - 依存: 10、13。仕様: `cancel.md`。

15. **仕様: `failure_state.md`。** [x]
    - failure 状態の仕様を執筆する（英語と日本語、コードなし）: blocked
      タスク状態、有界の failure 伝播、アーキテクチャ 19 に従う安定した
      failure カテゴリ。
    - 結果: direct failure record、blocked-work record、有界 propagation、
      stable category、deterministic ordering、publication / authority boundary、
      task 16 coverage を扱う同期済み `failure_state.md` specs を追加した。
    - 依存: 9。仕様: アーキテクチャ 14「Failure Propagation Is Bounded」、
      [19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)。

16. **failure の伝播。** [x]
    - 有界伝播と決定的な failure 報告を備えた blocked/failed タスク状態を
      実装する。
    - テスト: 1 つの失敗タスクが正確にその依存先のみをブロックする。
      failure 順が決定的である。
    - 結果: direct failure record、blocked-task record、stable block reason、
      deterministic ordering、synthetic task-category projection を備えた
      `src/failure_state.rs` を追加した。scheduler run は `failure_records` と
      `blocked_records` を emit し、nearest blocker を保持し、direct scheduler block
      reason を保存し、failed / blocked / cancelled work が output を publish
      しないことを保つ。
    - 依存: 10、15。仕様: `failure_state.md`。

17. **決定的コミット境界。** [x]
    - `mizar-artifact` の manifest トランザクションを通じた artifact
      コミットを統合する: コミットは完了順に関係なく正準順で直列化
      される。
    - テスト: 完了順をシャッフルしても同一の manifest をコミットする。
      中断されたコミットは旧状態を可視のまま残す。
    - 結果: deterministic `ModuleArtifactEntry` staging、`mizar-artifact`
      `ManifestTransaction` consumption、freshness-check forwarding、
      deterministic commit records を備えた `src/artifact_commit.rs` を追加した。
      focused tests は shuffled ordering、obsolete freshness rejection、conflict
      propagation、boundary placeholder absence を覆う。
    - 依存: 10、`mizar-artifact` task 14。仕様:
      [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
      「Artifact Commit Boundary」、`artifact_commit.md`。

18. **cache-aware スケジューリングの seam。** [x]
    - タスク実行前キャッシュ参照の seam（internal 02 の制御フロー）を
      インターフェースの背後に追加し、`mizar-cache` が接続できるように
      する。それまでは非キャッシュ実行が既定である。
    - driver が所有する `salsa` query 境界（`mizar-driver` task 4〜5）が消費する
      scheduler / cache seam を提供する: driver はこの seam を通じて work を
      skip、reuse、enqueue できるが、結果順序と artifact commit は決定的なまま
      保つ。`mizar-build` は引き続き `mizar-driver` に依存しない。
    - テスト: synthetic caller-supplied cache scheduling decisions による seam
      fixtures。ヒット時は実行をスキップしつつ外部から見える結果は同一。
    - 依存: 10。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
      「Cache Lookup Before Task Execution」; `cache_seam.md`。
    - task 18 で完了: `cache_seam.md` と `src/cache_seam.rs` が validated-hit
      decision boundary を追加する。`mizar-driver`、`mizar-ir`、real producer
      publication-token integration は `external_dependency_gap` のままである。

### 強化と横断フォローアップ

19. **batch ビルド統合スイート。** [x]
    - 小さな workspace 上で plan → graph → schedule → commit をエンド
      ツーエンドで実行する。その時点で利用可能な build-side boundaries を
      使う。real driver-owned phase-service boundary が消費可能になるまで、
      frontend-shaped tasks は synthetic outcomes で schedule する。
    - task 19 scope: fixture は現時点で利用可能な public `mizar-build`
      boundaries を覆い、不在の driver、IR、producer-token integration を
      `external_dependency_gap` として記録する。fake driver APIs、IR handles、
      producer publication tokens、proof authority は追加してはならない。
    - 依存: 6、17。仕様: [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
      「Batch Build」; `batch_integration.md`。
    - task 19 で完了: `tests/batch_integration.rs` が plan、module index、
      task graph、batch scheduling、`mizar-artifact` 経由の deterministic
      manifest commit を検証する。real driver、IR、producer-token integration は
      `external_dependency_gap` のままである。明示的 placeholder guards と
      validated-cache-hit non-authority check も Task 19 boundary を覆う。

20. **決定性スイート。** [x]
    - 同一入力に対して、ワーカー数をまたいで計画、グラフ、スケジュール、
      イベント、コミットが同一であることのプロパティ的検証。
    - task 20 scope: implemented `mizar-build` seams を table-driven fixtures
      で覆う。不在の driver、IR、producer-token clean/incremental integration は
      `external_dependency_gap` として記録し、placeholders は追加しない。
    - 依存: 17、18、19。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md);
      `determinism_suite.md`。
    - task 20 で完了: `tests/determinism_suite.rs` が deterministic
      plan/index/graph projections、worker と priority variants をまたぐ
      scheduler results/events、cache hit/miss commit projections、shuffled
      manifest commits、明示的 external-gap placeholder guards を比較する。

21. **公開 enum の前方互換性ポリシー。** [x]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。
    - task 21 で完了: 現在のすべての `mizar-build` public enum は
      `#[non_exhaustive]` である。所有する英語版と日本語版の各 module spec は
      `Public Enum Policy` table にその決定を記録し、exhaustive public enum
      exception がないことを明記する。`tests/lint_policy.rs` が今後の
      source/spec drift を guard する。
    - 依存: 16。仕様: 全モジュール仕様。

22. **ソース/仕様対応監査。** [x]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - task 22 で完了: `source_spec_correspondence.md` が、実装済みの public
      API family と behavior promise について source/spec/test map を記録する。
      新しい blocking/high drift は見つからず、BUILD-G-016 が non-blocking
      public-helper `test_gap` を 1 件記録し、既存の driver、IR、producer-token、
      full real clean/incremental integration gaps は `external_dependency_gap` のまま残る。
    - 依存: 21。仕様: 全モジュール仕様と本 TODO。

23. **二言語ドキュメント同期監査。** [x]
    - `doc/design/mizar-build/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - task 23 で完了: `bilingual_documentation_synchronization.md` が
      paired-file audit を記録する。すべての英語正本 design docs は同名の
      日本語 companion を持つ。deferred された companion update は残らず、
      BUILD-G-016 は non-blocking `test_gap` のままであり、既存の driver、
      IR、producer-token、full real clean/incremental integration gaps は
      `external_dependency_gap` のまま残る。
    - 依存: 22。仕様: リポジトリのドキュメント方針。

24. **増分/並列同値性 gate。** [x]
    - architecture 22 の scheduler-level regression gate を追加する。同じ
      `BuildSnapshot` と verifier policy に対して、clean sequential、clean
      parallel、incremental sequential、incremental parallel execution が
      同一の published artifact、interface hash、依存側に見える summary、
      proof acceptance、canonical diagnostics を commit しなければならない。
      cache hit が作業を skip する場合、progress や build event の timing は
      異なり得るが、event consumer が stale publication を current result として
      観測してはならない。
    - テスト: randomized ready-task scheduling と worker count。synthetic cache
      decision の hit/miss timing。cancellation / supersession が部分 publication を
      残さないこと。cache miss は work を enqueue するだけで deterministic commit
      boundary を変えないこと。
    - 依存: 14、18、20。仕様:
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md),
      [14.parallel_verification_and_scheduling.md](../../architecture/ja/14.parallel_verification_and_scheduling.md),
      [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。
    - task 24 で完了: `incremental_parallel_equivalence.md` が implemented-seam
      scope と BUILD-G-017 `external_dependency_gap` を記録する。
      `tests/determinism_suite.rs` は同じ snapshot 上の clean sequential、clean
      parallel、incremental sequential、incremental parallel scheduler runs について
      externally visible projection を比較し、stale または superseded incremental
      results が current manifest updates を publish しないことを確認する。

25. **architecture-22 フォローアップ監査。** [x]
    - task 24 の scheduler equivalence、cancellation、cache seam 契約について、
      ソース/仕様対応監査と二言語ドキュメント同期監査を再実行する。残る
      stale-publication または deterministic commit-boundary gap を
      フォローアップタスクとして記録する。
    - task 25 で完了: `architecture_22_follow_up_audit.md` が source/spec と
      bilingual audit re-run を記録する。`source_spec_correspondence.md` と
      `bilingual_documentation_synchronization.md` は task-24 gate、BUILD-G-017、
      同期済み EN/JA status を含む。stale-publication または deterministic
      commit-boundary gap は low severity 超で unresolved のまま残っていない。
    - 依存: 24。仕様: 全モジュール仕様、本 TODO、リポジトリの
      ドキュメント方針。

26. **module 境界リファクタリング gate。** [x]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 25。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。
    - task 26 で完了: `module_boundary_refactor_gate.md` は BUILD-G-018 を
      解決済み layout-only `source_drift` として記録する。`planner`、
      `module_index`、`task_graph`、`scheduler` の inline unit-test bodies は
      private child modules へ移動した。public exports、diagnostics、
      deterministic renderings、schemas、behavior は変更していない。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-build
cargo clippy -p mizar-build --all-targets -- -D warnings
```

resolver のプロバイダーやコミット境界に触れるタスクでは、利用可能な隣接
crate checks を追加で実行する:

```text
cargo test -p mizar-resolve
cargo test -p mizar-artifact
```

architecture-22 の equivalence gate では、利用可能な隣接 crate checks を追加で
実行し、未作成の `mizar-driver` など欠けている crate は明示的に正当化する:

```text
cargo test -p mizar-cache
cargo test -p mizar-artifact
cargo test -p mizar-vc
cargo test -p mizar-proof
cargo test -p mizar-driver
cargo test -p mizar-test
```

テストが通ったらここでタスクにチェックを付ける。

Closeout は [crate_exit_report.md](./crate_exit_report.md) に記録する。この report は
hard gates、deferred items、verification、human review surface、next-phase handoff を
要約する。

## 備考

- スケジューリングは検証された意味論、診断順、artifact 順、証明の受理を
  変えてはならない。完了順は決して意味論順・artifact 順として使われ
  ない。
- すべてのタスク出力は依存先へ公開された時点で不変である。キャンセル
  された作業や失敗した作業は決して公開しない。
- `mizar-driver` が batch/watch/LSP のエントリポイントのためにこの crate
  を消費する。この crate を CLI とプロトコルの関心事から自由に保つ。
- キャッシュヒットはクリーンビルドと同じ検証規則を満たさなければなら
  ない（キャッシュは最適化であって権威ではない — `mizar-cache` の契約）。
