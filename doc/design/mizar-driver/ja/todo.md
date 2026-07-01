# mizar-driver TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様は段階的に追加する。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。この crate は
[internal 07](../../internal/ja/07.crate_module_layout.md) の所有権マップに
従い、[internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| request | `request.md`（task 2） | `src/request.rs` | [x] |
| registry | `registry.md`（task 4） | `src/registry.rs` | [x] |
| driver | `driver.md`（task 7） | `src/driver.rs` | [ ] |
| events | `events.md`（task 9） | `src/events.rs` | [ ] |
| cli | `cli.md`（task 12） | `src/cli.rs` | [ ] |

Task D-006 は `SourceFrontend` adapter readiness decision を
[frontend_adapter.md](frontend_adapter.md) に記録する。これは module source surface
ではない。owner seam が存在するまで、registry はその real adapter を external
dependency gap と分類し続ける。

`mizar-driver` はすべてのビルドモードの正面玄関である: CLI/watch/LSP の
リクエストを `BuildRequest` へ構文解析し、`mizar-build` の planner を
通じて phase 0 をブートストラップし、ソースと依存の snapshot を持つ
`BuildSession` を作成し、`PhaseService` trait の背後に phase サービス実装を
登録し、初期タスクグラフをスケジューラへ投入し、ビルドイベントを進捗
レポーターと LSP ブリッジへ公開する。phase の意味論、キャッシュ互換性の
決定、artifact のシリアライゼーション、エディタプロトコル変換は一切
所有しない — 部品を結線する薄い層に留まる。

依存順序: `request` → `registry` → `driver` → `events` → `cli` / watch
モード。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session`、`mizar-build`（planner、タスクグラフ、
スケジューラ）、`mizar-ir`（出力ストレージと snapshot ハンドル）、
`mizar-diagnostics`（sink と集約）に依存し、バイナリレベルで登録される
phase サービスアダプターを通じて、着地し次第パイプライン crate
（最初は `mizar-frontend`）に依存する。最後に組み立てるサブシステムで
あり、`mizar-build` 第 B 波とともに開始する。internal:
[01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)。
仕様: [23.package_management_and_build_system.md](../../../spec/ja/23.package_management_and_build_system.md)。

`salsa` は最終的な driver / phase-service orchestration 層の必須 query engine
である。syntax と parser の crate は `salsa` に依存しない。この crate が、
phase service を包み、build scheduler と cache seam へ純粋な入力 / 出力を
公開する database / query 境界を所有する。

## 解決済みおよび保留中の決定

- **driver/build の分離: internal 00/01 により解決済み。** 計画と
  スケジューリングは `mizar-build` にあり、この crate はリクエストの
  ライフサイクル、サービスレジストリ、エントリポイントを所有する。
- **CLI の表面: 未解決。task 12 で解決する。** 仕様第 23 章のビルド
  ライフサイクルに対してバイナリ名とサブコマンド集合を決める（既定候補:
  単一の `mizar` バイナリに `build`/`check`/`doc` サブコマンドを増分で
  追加）。決定を `cli.md` に記録する。
- **`cache_key` の純粋性: internal 01 により解決済み。**
  `PhaseService::cache_key` は入力識別、設定、schema version、依存
  ハッシュからの純粋な射影である。レジストリがこの契約を強制し、
  テストする。
- **salsa query 境界: task 4 で仕様化済み。初期 registry seam は task 5 で実装済み。**
  `salsa` は最終的な query / cache 層に必須である。`registry.md` は `PhaseService` の
  入力、出力、cancellation / versioning、cache-key intent を driver-owned salsa database
  へどう対応付けるかを定義し、phase crate には driver / query-engine への直接依存を
  持ち込まない。real cache lookup と real phase adapter は後続の owner seam work に残る。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-driver` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### リクエストとサービス

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session`、`mizar-build`、`mizar-ir`、`mizar-diagnostics` に
     依存する workspace メンバー `mizar-driver` を追加し、
     `mizar-frontend` のガードに倣った `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-build` task 10、`mizar-ir` task 8、
     `mizar-diagnostics` task 9。仕様: internal 01。
   - task D-001 で完了: workspace member、最小 crate manifest/library scaffold、
     driver lint-policy guard、driver scaffold のための diagnostics
     reverse-dependency guard 例外を追加した。request/session/registry/event/CLI/watch
     behavior や placeholder seam は導入していない。

2. **仕様: `request.md`。** [x]
   - リクエストの仕様を執筆する（英語と日本語、コードなし）:
     batch/watch/LSP の `BuildRequest` の形、`BuildSession` とその
     ソース/依存 snapshot、セッションのライフサイクル状態、置き換えられた
     watch/LSP セッションからの obsolete publication を拒否する、driver
     lane-current-session と request-generation snapshot guard の combined 境界。
   - 依存: 1。仕様: [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
     「Build Session」。
   - task D-002 で完了: [request.md](request.md) は request origin、
     driver-owned currentness lane、session lifecycle state、`mizar-session` を通じた
     snapshot capture、superseded watch/LSP session の obsolete-publication
     suppression を combined publication guard で定義する。source implementation は
     追加していない。

3. **`BuildRequest` と `BuildSession`。** [x]
   - `mizar-session`/`mizar-ir` の識別を通じた snapshot 捕捉を備えた
     リクエストとセッションを実装する。置き換えられた watch/LSP
     セッションからの obsolete publication を拒否する lane-current-session と
     request-generation snapshot guard の combined 境界を含める。
   - テスト: セッションのラウンドトリップ。同一の workspace 状態が同一の
     snapshot id を生む。fresh request id が watch/LSP generation 間で driver lane
     を共有する。superseded session は lane-current session と同じ snapshot id を持つ
     場合も拒否される。
   - 依存: 2。仕様: `request.md`。
   - task D-003 で完了: `src/request.rs` は request draft、allocated pending
     request、captured session、lifecycle transition、driver lane-current-session
     tracking、publication decision を実装する。request tests は `mizar-session` を
     通じた snapshot capture、同一 canonical snapshot id と異なる allocator-issued
     id、same-snapshot supersession rejection、stale request-generation snapshot、
     stale lane update rejection、snapshot-creation failure context、suppressed
     publication decision、idempotent cancellation を cover する。LSP
     protocol conversion、scheduler semantics、artifact publication、cache/proof
     authority、phase semantics は追加していない。

4. **仕様: `registry.md`。** [x]
   - レジストリの仕様を執筆する（英語と日本語、コードなし）:
     `PhaseService` trait（`phase`、`cache_key`、`execute`）、
     `PhaseContext`/`PhaseResult`、phase 0-16 のサービス表、cache_key の
     純粋性契約。
   - 必須の `salsa` 統合境界を定義する: database の lifetime、input query、
     派生 phase-output query、cancellation / snapshot-version との相互作用、
     phase crate は `salsa` に直接依存せず純粋な service を公開するという規則。
   - 依存: 2。仕様: [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
     「Phase Services」「Phase Service API」。
   - task D-004 で完了: [registry.md](registry.md) は deterministic phase-service
     registry、phase 0-16 service table、driver-owned salsa query boundary、
     cache-key purity contract、scheduler/cache seam handoff、diagnostics/artifact/LSP
     non-ownership rule を定義する。未準備の real phase adapter、artifact publication
     token、producer output、LSP bridge は fake adapter や provisional API ではなく
     classified gap として残す。

5. **phase サービスレジストリ。** [x]
   - 重複 phase の拒否と `cache_key` の純粋性テストハーネスを備えた
     phase サービスの登録と参照を実装する。
   - `registry.md` の salsa-backed な初期レジストリ seam を追加する。service は
     決定的 query adapter 経由で実行し、registry が database handle を所有し、
     すべての入力 / 出力は後続の実 phase service でも使う同じ query-compatible
     境界を通過しなければならない。
   - テスト: 登録のフィクスチャ。重複の拒否。test-only fixture service による
     `cache_key` 決定性ハーネス。real owner seam の欠落が synthetic output なしで
     classified gap として報告されること。driver-owned salsa/query boundary の
     positive guard。syntax/parser/phase owner crate が driver または salsa dependency を
     得ていないことを示す dependency scan。cache compatibility、proof acceptance、
     artifact publication-token、LSP payload、scheduler-readiness ownership を拒否する
     boundary guard。
   - 依存: 4。仕様: `registry.md`。
   - task D-005 で完了: `src/registry.rs` は deterministic phase-service
     registration、duplicate coverage rejection、phase 0-16 requirement table、
     missing-service gap reporting、`PhaseService` cache/execute adapter、
     driver-local salsa database/query observation boundary、focused registry/lint test を
     実装する。使う service は test-local fixture だけである。real semantic adapter、
     cache compatibility decision、proof acceptance、artifact publication token、LSP
     payload、scheduler readiness logic は導入していない。

6. **`SourceFrontend` サービスアダプター。** [x]
   - `mizar-frontend` の phase 1-3 を最初の実 `PhaseService` として包む
     （入力: 計画のスライス。出力: `mizar-ir` を通じて seal された
     frontend 出力）。
   - テスト: フィクスチャモジュール上のアダプターのラウンドトリップ。
     診断が sink へ流れる。
   - 依存: 5、`mizar-ir` task 8。仕様: `registry.md`、
     [mizar-frontend todo](../../mizar-frontend/ja/todo.md)。
   - task D-006 で source implementation ではなく分類済み
     `external_dependency_gap` として完了:
     [frontend_adapter.md](frontend_adapter.md) は、`mizar-frontend` には real
     in-memory `FrontendOutput` がある一方、必要な canonical `mizar-ir`
     producer payload、diagnostics-draft bridge、driver build-plan-to-source-request
     mapping がまだ real seam ではないことを記録する。fake adapter、synthetic IR
     payload、message-text keyed diagnostic bridge は追加していない。registry の
     `SourceFrontend` requirement は `external_dependency_gap` のままである。

### オーケストレーション

7. **仕様: `driver.md`。** [ ]
   - driver の仕様を執筆する（英語と日本語、コードなし）:
     `CompilerDriver` API（`submit`、`cancel`、`events`）、phase 0 の
     ブートストラップ、タスクグラフの投入、artifact コミット境界への
     受け渡し。
   - 依存: 4。仕様: [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
     「Driver API」「Control Flow」。

8. **driver のコア。** [ ]
   - `submit` を実装する: `mizar-build` planner による phase 0 ブート
     ストラップ、セッション作成、タスクグラフの展開と投入、登録された
     サービスのスケジューラ経由の駆動。
   - テスト: frontend サービスを使ったフィクスチャ workspace 上の
     batch 実行。決定的な phase 順序。
   - 依存: 3、5、6、7、`mizar-build` task 8。仕様: `driver.md`。

9. **仕様: `events.md`。** [ ]
   - イベントの仕様を執筆する（英語と日本語、コードなし）:
     `BuildEventStream`（進捗、phase 完了、診断準備完了、コミット）、
     決定的なイベント順序、CLI と LSP の消費規則。
   - 依存: 7。仕様: [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
     「Build Events」。

10. **ビルドイベントストリーム。** [ ]
    - ワーカーの完了順に依存しない決定的順序でのイベント公開を実装する。
    - テスト: 完了順をシャッフルしても同一のイベント列。イベントが有効な
      セッションを参照する。
    - 依存: 8、9。仕様: `events.md`。

11. **キャンセルのフロー。** [ ]
    - `cancel` を実装する: `mizar-build` のキャンセルトークンへ伝播し、
      終端のセッション状態を報告する。置き換えられた watch セッションは
      クリーンにキャンセルされる。
    - テスト: ビルド途中のキャンセルが部分公開なしに終端状態へ達する。
      二重キャンセルの冪等性。
    - 依存: 8、`mizar-build` task 14。仕様: `driver.md`、
      [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
      「Cancellation」。

### エントリポイント

12. **仕様: `cli.md` と CLI 表面の決定。** [ ]
    - 仕様第 23 章のビルドライフサイクルに対して CLI 表面の決定を解決
      する。CLI の仕様（バイナリ、サブコマンド、終了コード、
      `mizar-diagnostics` 経由の進捗レンダリング）を執筆する。
    - 依存: 7。仕様:
      [23.package_management_and_build_system.md](../../../spec/ja/23.package_management_and_build_system.md)。

13. **CLI の batch エントリポイント。** [ ]
    - batch サブコマンドを実装する: 引数を `BuildRequest` へ構文解析し、
      driver を実行し、診断と進捗をレンダリングし、結果を終了コードへ
      写像する。
    - テスト: フィクスチャ workspace 上のエンドツーエンド CLI 実行。
      安定した終了コード。golden ファイル出力。
    - 依存: 10、12。仕様: `cli.md`。

14. **watch モード。** [ ]
    - watch ループを実装する: ファイル変更検出、`mizar-ir` を通じた
      snapshot 置換、置き換えられたセッションのキャンセル、増分の
      再投入。
    - テスト: 変更 → 再ビルドのフィクスチャ。古いセッションは決して
      公開しない。置換は retain された出力を生かしておく。
    - 依存: 11、13、`mizar-ir` task 13。仕様:
      [internal 01](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
      「Watch and LSP Build」。

15. **意味論 phase のサービスアダプター。** [ ] — パイプライン crate が
    律速。
    - `ModuleResolver`、`SemanticChecker`、`Elaborator`、`VcService`、
      `AtpService`、`KernelService`、`ArtifactService`、
      `DocExtractionService` のアダプターを、各 crate のサービス向け表面が
      着地し次第、1 変更につき 1 アダプターで登録する。最後のアダプターが
      着地した時点でチェックを付ける。
    - アダプターごとのテスト: driver を通したフィクスチャ実行。診断と
      出力がエンドツーエンドで流れる。
    - 依存: 8。各 crate の統合タスクと対になる。仕様: `registry.md`。

### 強化と横断フォローアップ

16. **エンドツーエンド決定性スイート。** [ ]
    - 同一の workspace が、ワーカー数と実行をまたいで同一のイベント
      ストリーム、診断、終了コードを生むことのプロパティ的検証。cache /
      scheduler seam が接続された後は、driver-owned query boundary を通る
      clean versus incremental と cache hit/miss timing の architecture-22
      equivalence case も含める。stale または obsolete snapshot の出力は、
      current diagnostics や artifact として公開される前に拒否する。
    - 依存: 13、14、`mizar-build` task 24。仕様:
      [20.test_strategy.md](../../architecture/ja/20.test_strategy.md),
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md)。

17. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。
    - 依存: 13。仕様: 全モジュール仕様。

18. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 17。仕様: 全モジュール仕様と本 TODO。

19. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-driver/en/` の各英語正本と日本語版を比較し、
      内容を同期する。
    - 依存: 18。仕様: リポジトリのドキュメント方針。

20. **architecture-22 フォローアップ監査。** [ ]
    - task 16 の driver query-boundary、stale-output rejection、diagnostics、
      artifact-publication 契約について、ソース/仕様対応監査と二言語
      ドキュメント同期監査を再実行する。残る architecture-22 gap を
      フォローアップタスクとして記録する。
    - 依存: 16、19。仕様: 全モジュール仕様、本 TODO、リポジトリの
      ドキュメント方針。

21. **module 境界リファクタリング gate。** [ ]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 20。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-driver
cargo clippy -p mizar-driver --all-targets -- -D warnings
```

オーケストレーションのタスクでは追加で実行する:

```text
cargo test -p mizar-build
cargo test -p mizar-ir
cargo test -p mizar-frontend
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- driver が所有するのは結線であって意味論ではない: 型検査、オーバー
  ロード解決、VC 生成、証明の受理、キャッシュ互換性の決定、artifact の
  シリアライゼーション、LSP 範囲変換は行わない。
- `PhaseService::cache_key` は純粋な射影に留まらなければならない。
  レジストリの純粋性ハーネスが強制点である。
- `salsa` は driver / registry 層で導入し、syntax、parser、semantic phase
  crate には入れない。phase adapter が純粋で query-compatible な境界を提供する。
- 古い snapshot 由来の診断を現在のものとして公開しない。artifact の
  コミットは決して完了順では行われない。
- LSP のエントリポイントは `mizar-lsp` を通じて同じ driver API を再利用
  する。この crate はプロトコル非依存に留まる。
