# mizar-build TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

完全なモジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。`planner` と
`module_index` source は第 A 波の phase-0 planning と module-index provider work を
実装済みであり、残りの module は引き続き専用の仕様先行 task に従う。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md)（最小:
`task_graph`、`scheduler`、`failure_state`）に、アーキテクチャ 00/03 の
phase 0 計画モジュールを加えたものに従う。この crate はアーキテクチャ 14、
19 と internal 01 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| planner | `planner.md`（task 2） | `src/planner.rs` | [x] |
| module_index | `module_index.md`（task 5） | `src/module_index.rs` | [x] |
| task_graph | `task_graph.md`（task 7） | `src/task_graph.rs` | [ ] |
| scheduler | `scheduler.md`（task 9） | `src/scheduler.rs` | [ ] |
| resource | `resource.md`（task 11） | `src/resource.rs` | [ ] |
| cancel | `cancel.md`（task 13） | `src/cancel.rs` | [ ] |
| failure_state | `failure_state.md`（task 15） | `src/failure_state.rs` | [ ] |

`mizar-build` はパイプライン phase 0（workspace 計画: manifest、lockfile、
依存グラフ、`BuildPlan`、モジュール索引）と、並列検証の機構（タスク
グラフ、スケジューラ、リソース予算、キャンセル、blocked タスク状態）を
実装する。スケジューリングは意味論から分離される: 並列性はレイテンシを
変えてよいが、診断順、artifact 順、証明の受理、再現性を決して変えない。
ビルドリクエストと phase サービスレジストリは `mizar-driver` に属し、
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
スケジューリングは `mizar-cache` ができたときに統合する。アーキテクチャ:
[14.parallel_verification_and_scheduling.md](../../architecture/ja/14.parallel_verification_and_scheduling.md)、
[19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)。
internal: [01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)。
仕様: [23.package_management_and_build_system.md](../../../spec/ja/23.package_management_and_build_system.md)。

## 解決済みおよび保留中の決定

- **driver の分離: internal 00/01 により解決済み。** `mizar-driver` が
  ビルドリクエスト、CLI/watch/LSP のエントリポイント、phase レジストリを
  所有する。`mizar-build` は計画とスケジューリングを所有し、エントリ
  ポイントに依存しないままにする。
- **初期タスク粒度: 未解決。task 8 で解決する。** スケジューラは
  アーキテクチャの依存境界が保たれる限り、最初は粗いタスク粒度を選んで
  よい（アーキテクチャ 14 の制約）。最初の粒度（既定候補: モジュール
  ごとの意味論タスク、proof タスクは後で VC ごと）を決め、
  `task_graph.md` に記録する。
- **cache-aware スケジューリングの時期: 未解決。task 18 で解決する。**
  タスク実行前のキャッシュ参照は `mizar-cache` を必要とし、driver が所有する
  必須の `salsa` query 境界（`mizar-driver` task 4〜5）から呼び出せなければ
  ならない。それまでは seam を用意した上で非キャッシュ実行とする。

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

7. **仕様: `task_graph.md`。** [ ]
   - タスクグラフの仕様を執筆する（英語と日本語、コードなし）: タスク
     種別、バージョン付きタスク識別、依存辺（モジュール依存が意味論
     phase をゲートする。VC は細粒度タスク）、初期粒度の決定。
   - 依存: 2。仕様: アーキテクチャ 14「Task Graph」「VCs Are Fine-Grained
     Tasks」、[internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)。

8. **タスクグラフの構築。** [ ]
   - `BuildPlan` をバージョン付きタスクグラフへ展開する。粒度の決定を
     解決し記録する。
   - テスト: グラフ展開のフィクスチャ。依存辺がアーキテクチャ境界と
     一致する。決定的な展開。
   - 依存: 7。仕様: `task_graph.md`。

9. **仕様: `scheduler.md`。** [ ]
   - スケジューラの仕様を執筆する（英語と日本語、コードなし）: ワーク
     キュー、優先度ポリシー、batch 対 watch/LSP モード、ビルドイベント、
     決定的結果順序の規則（完了順は決して意味論順・artifact 順では
     ない）。
   - 依存: 7。仕様: アーキテクチャ 14「Deterministic Result Ordering」、
     [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
     「Pipeline Scheduler」。

10. **スケジューラのコア。** [ ]
    - タスクグラフ上のワーカープールとキュー実行を、任意の完了順の下で
      決定的な結果順序を保って実装する。テストは合成タスクを使う。
    - テスト: 完了順をシャッフルしても同一の結果順・イベント順。公開
      された出力の不変性。
    - 依存: 8、9。仕様: `scheduler.md`。

11. **仕様: `resource.md`。** [ ]
    - リソース予算の仕様を執筆する（英語と日本語、コードなし）: 階層的
      予算（ビルド → パッケージ → タスク）、ワーカープールのサイズ、
      ATP runner へ渡す外部プロセス制限。
    - 依存: 9。仕様: アーキテクチャ 14「Resource Budgets Are
      Hierarchical」。

12. **リソース予算。** [ ]
    - スケジューラにおける予算の計上と強制を実装する。
    - テスト: 予算枯渇時はオーバーコミットせずキューに入る。予算が
      階層的に合成される。
    - 依存: 10、11。仕様: `resource.md`。

13. **仕様: `cancel.md`。** [ ]
    - キャンセルの仕様を執筆する（英語と日本語、コードなし）: 協調的で
      バージョン付きのキャンセルトークン、watch/LSP のための snapshot
      無効化、部分 artifact 禁止の規則。
    - 依存: 9。仕様: アーキテクチャ 14「Cancellation Is Cooperative and
      Versioned」。

14. **キャンセル。** [ ]
    - キャンセルトークンと snapshot バージョンの無効化を実装する。
      キャンセルされた作業は決して出力を公開しない。
    - テスト: グラフ途中のキャンセルが公開済みの部分状態を残さない。
      古い snapshot バージョンは決して公開しない。
    - 依存: 10、13。仕様: `cancel.md`。

15. **仕様: `failure_state.md`。** [ ]
    - failure 状態の仕様を執筆する（英語と日本語、コードなし）: blocked
      タスク状態、有界の failure 伝播、アーキテクチャ 19 に従う安定した
      failure カテゴリ。
    - 依存: 9。仕様: アーキテクチャ 14「Failure Propagation Is Bounded」、
      [19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)。

16. **failure の伝播。** [ ]
    - 有界伝播と決定的な failure 報告を備えた blocked/failed タスク状態を
      実装する。
    - テスト: 1 つの失敗タスクが正確にその依存先のみをブロックする。
      failure 順が決定的である。
    - 依存: 10、15。仕様: `failure_state.md`。

17. **決定的コミット境界。** [ ]
    - `mizar-artifact` の manifest トランザクションを通じた artifact
      コミットを統合する: コミットは完了順に関係なく正準順で直列化
      される。
    - テスト: 完了順をシャッフルしても同一の manifest をコミットする。
      中断されたコミットは旧状態を可視のまま残す。
    - 依存: 10、`mizar-artifact` task 14。仕様:
      [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
      「Artifact Commit Boundary」。

18. **cache-aware スケジューリングの seam。** [ ]
    - タスク実行前キャッシュ参照の seam（internal 02 の制御フロー）を
      インターフェースの背後に追加し、`mizar-cache` が接続できるように
      する。それまでは非キャッシュ実行が既定である。
    - driver が所有する `salsa` query 境界（`mizar-driver` task 4〜5）が消費する
      scheduler / cache seam を提供する: driver はこの seam を通じて work を
      skip、reuse、enqueue できるが、結果順序と artifact commit は決定的なまま
      保つ。`mizar-build` は引き続き `mizar-driver` に依存しない。
    - テスト: モックキャッシュによる seam のフィクスチャ。ヒット時は
      実行をスキップしつつ外部から見える結果は同一。
    - 依存: 10。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
      「Cache Lookup Before Task Execution」。

### 強化と横断フォローアップ

19. **batch ビルド統合スイート。** [ ]
    - 小さな workspace 上で plan → graph → schedule → commit をエンド
      ツーエンドで実行する。その時点で利用可能な phase サービス
      （現状は frontend、意味論 phase は着地し次第）を使う。
    - 依存: 6、17。仕様: [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
      「Batch Build」。

20. **決定性スイート。** [ ]
    - 同一入力に対して、ワーカー数をまたいで計画、グラフ、スケジュール、
      イベント、コミットが同一であることのプロパティ的検証。
    - 依存: 17。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

21. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。
    - 依存: 16。仕様: 全モジュール仕様。

22. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 21。仕様: 全モジュール仕様と本 TODO。

23. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-build/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 22。仕様: リポジトリのドキュメント方針。

24. **増分/並列同値性 gate。** [ ]
    - architecture 22 の scheduler-level regression gate を追加する。同じ
      `BuildSnapshot` と verifier policy に対して、clean sequential、clean
      parallel、incremental sequential、incremental parallel execution が
      同一の published artifact、interface hash、依存側に見える summary、
      proof acceptance、canonical diagnostics を commit しなければならない。
      cache hit が作業を skip する場合、progress や build event の timing は
      異なり得るが、event consumer が stale publication を current result として
      観測してはならない。
    - テスト: randomized ready-task scheduling と worker count。mock cache の
      hit/miss timing。cancellation / supersession が部分 publication を残さない
      こと。cache miss は work を enqueue するだけで deterministic commit
      boundary を変えないこと。
    - 依存: 14、18、20。仕様:
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md),
      [14.parallel_verification_and_scheduling.md](../../architecture/ja/14.parallel_verification_and_scheduling.md),
      [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

25. **architecture-22 フォローアップ監査。** [ ]
    - task 24 の scheduler equivalence、cancellation、cache seam 契約について、
      ソース/仕様対応監査と二言語ドキュメント同期監査を再実行する。残る
      stale-publication または deterministic commit-boundary gap を
      フォローアップタスクとして記録する。
    - 依存: 24。仕様: 全モジュール仕様、本 TODO、リポジトリの
      ドキュメント方針。

26. **module 境界リファクタリング gate。** [ ]
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

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-build
cargo clippy -p mizar-build --all-targets -- -D warnings
```

resolver のプロバイダーやコミット境界に触れるタスクでは追加で実行する:

```text
cargo test -p mizar-resolve
cargo test -p mizar-artifact
```

architecture-22 の equivalence gate では追加で実行する:

```text
cargo test -p mizar-cache
cargo test -p mizar-driver
cargo test -p mizar-test
```

テストが通ったらここでタスクにチェックを付ける。

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
