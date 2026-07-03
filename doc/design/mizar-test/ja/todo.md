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
| layout | [layout.md](./layout.md) | `src/layout.rs`、`src/path_rules.rs` | [~] discovery/pairing と validation-mode unknown-root policy は実装済み。Public API 同期は未完 |
| expectation_schema | [expectation_schema.md](./expectation_schema.md) | `src/expectation.rs` | [~] core schema、profile/provenance metadata retention、fail/soundness rejection gate は実装済み。general snapshot 強化は未完 |
| staged_model | [staged_model.md](./staged_model.md) | `src/staged_model.rs` | [~] stage id と declared prerequisite validation は実装済み。より広い admission policy は未完 |
| traceability | [traceability.md](./traceability.md) | `src/traceability.rs` | [~] syntax/backref、coverage report/status gate、manifest ordering、obsolete-ref check、prerequisite credit gate、architecture-22 matrix summary は実装済み |
| harness | [harness.md](./harness.md) | `src/harness.rs`、`src/main.rs`、`src/runner.rs` | [~] metadata plan、validation-mode CLI、profile filtering、coverage/pass-fail/matrix report、active parse/declaration/type runner |
| miz_corpus | [miz_corpus.md](./miz_corpus.md) | `tests/` 配下のコーパスツリー | [~] root discovery、pass/fail mix reporting、provenance/profile policy rules validation は実装済み。future corpus classes は未完 |
| snapshot | [snapshot.md](./snapshot.md) | `src/snapshot.rs`、`src/expectation.rs`、`src/runner.rs` | [~] general snapshot record API/hash/update/determinism helpers は実装済み。sidecar/runner integration は未完 |
| fail_soundness | [fail_soundness.md](./fail_soundness.md) | `src/expectation.rs`、`src/harness.rs`、将来の runner case | [~] metadata contract gate は実装済み。active proof/certificate/kernel execution は将来の runner が律速 |
| minimal_crate | [minimal_crate.md](./minimal_crate.md) | crate 境界＋CLI | [~] metadata plan、validation mode、CLI fixture、coverage gate、prerequisite gate は実装済み |

`mizar-test` はコーパスとハーネスの crate である: テスト発見、
`.expect.toml` の expectation 構文解析、staged model、仕様カバレッジの
traceability、snapshot 比較、fail/健全性契約。意図的に最小である
（[minimal_crate.md](./minimal_crate.md)）: metadata `plan` mode は payload を
実行せずに検証と計画を所有する。一方、明示的な active runner subcommand は、
その stage に必要な狭い pipeline seam にだけ依存してよい。parse-only runner の
場所は `mizar-parser` task 3 で確定しており、declaration-symbol runner は
`mizar-resolve` task 23 で同じ active-subcommand model に従う。

以下の各タスクは意図的に小さくしてある — 既存仕様に対する 1 挙動
スライス — 。これにより、crate の残りを抱え込まずに 1 タスクを単独で
実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は [minimal_crate.md](./minimal_crate.md) に従って依存集合を
最小に保つ。metadata API は payload-free のままにする。active runner
subcommand だけが、実行する stage に必要な pipeline dependency を追加する。
コーパスとカバレッジの成長は消費側 crate のランナータスク（`mizar-parser` task 3、
`mizar-resolve` task 23、`mizar-checker` task 12/29、`mizar-vc` task 15、
`mizar-atp` task 20、`mizar-kernel` task 17）が律速する。

## 解決済みおよび保留中の決定

- **パイプライン非依存: [minimal_crate.md](./minimal_crate.md) により
  解決済み。** metadata `plan` path は payload を実行しない。明示的な
  active runner subcommand は、対象 stage のために exercise する狭い
  pipeline seam に依存してよい。それらの dependency は metadata validation では
  使わない。
- **コーパスランナーの場所: `mizar-parser` task 3 が所有する**（後続
  stage は対応するタスクが所有する）。`mizar-resolve` task 23 はこの先例を
  `mizar-test` 内の declaration-symbol runner に拡張する。
- **snapshot 更新メカニズム: 未解決。task 5 で解決する。** ベースラインの
  （再）生成方法 — 明示的な update モード対環境フラグ — を
  [snapshot.md](./snapshot.md) の更新ポリシーの範囲内で決め、そこに
  記録する。

## task 2 監査ベースライン

task 2 は crate-wide source/spec audit を
[00.crate_plan.md](./00.crate_plan.md) に記録した。この監査では、blocking な
`spec_gap`、採用すべき `repo_metadata_conflict`、または language behavior
change は見つかっていない。以前の trace manifest ordering conflict は
`897d549` で修復済みである。task 6 で manifest-order validator と
regression test を追加した。

監査からの follow-up ownership:

- `layout`: documented Public API を `DiscoveredLayout` と harness-owned
  `TestCase` に同期する。新しい root が入るたび unknown-root policy を保つ。
- `expectation_schema`: generated origin table、certificate/kernel
  `rejection_reason`、diagnostic ordering、将来の general `[[snapshots]]`
  hash registry を検証する。
- `traceability`: 新しい evidence kind が入るたび coverage/status reporting を同期する。
  Manifest order validation、mode-aware coverage/status computation、
  obsolete-reference checks、declared prerequisite gates、既存 link-validator error fixtures
  は実装済み。
- `harness`: 後続で generic outcome/reporting surface が入るたび、
  runner-specific report docs と exported API の同期を保つ。
- `miz_corpus`: generated/fuzz/stress metadata、corpus-policy profile
  constraints、stress exclusion checks を enforceable にする。Corpus-wide
  pass/fail mix reporting は実装済み。
- `snapshot`: transitional parse-only `SurfaceAst` baseline path を超えて、
  general snapshot module、canonical hashing、explicit update flow、
  determinism checks を実装する。
- `fail_soundness`: task 8 は fail/soundness metadata bookkeeping、
  case-level required checks、false-arithmetic stable-key gating、
  weakening/deletion diagnostics を実装した。active proof/certificate/kernel
  execution は将来の consumer runner が律速する。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-test` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 基盤

1. **lint 方針のガード。** [x]
   - `mizar-frontend` のガードに倣った `tests/lint_policy.rs`（workspace
     lint へのオプトイン、deny ベースライン、将来の `allow` の隣に根拠）を
     追加する。
   - テスト: lint 方針ガードが通る。
   - 依存: なし。仕様: リポジトリの慣行。

2. **ソース/仕様ギャップ監査と状態の同期。** [x]
   - 9 本のモジュール仕様の Public API と Tests の約束を現在の実装へ
     トレースする。ギャップを本 TODO のフォローアップタスクとして記録し、
     モジュール表の状態を実態に合わせる。
   - 監査記録: [00.crate_plan.md](./00.crate_plan.md)「既知のギャップと
     drift」および [task 2 監査ベースライン](#task-2-監査ベースライン)。
   - 依存: 1。仕様: 全モジュール仕様。

3. **ランナーモードと CLI の完成。** [x]
   - [minimal_crate.md](./minimal_crate.md)「CLI」「Exit Codes」と
     [harness.md](./harness.md)「Runner Modes」に従い、`plan` を超えて
     CLI を完成させる: コーパスツリーとカバレッジマニフェスト上の
     検証モードと、文書化された終了コード。
   - task 2 gap として、`ValidationMode` の使用、strict/permissive
     unknown-root policy、plan-mode CLI output/exit-code fixture、
     documented/public reporting API shape を閉じる。
   - 現在は型チェック後に捨てている optional sidecar metadata
     （`profiles`、`notes`、`ast_profile`、`snapshot_profiles`）を保持し、
     plan construction に profile filtering を適用する。
   - [harness.md](./harness.md) と `parser.type_fixtures`
     import-summary exception を整合させる: 例外を明示的に文書化するか、
     fixture symbol injection を削除する。
   - unsupported schema version、id/source-stem mismatch、invalid enum/outcome
     pair、duplicate sidecar `spec_refs` の focused expectation-schema
     regression fixture を追加する。
   - テスト: モードごとの CLI フィクスチャ。終了コードが仕様の表と
     一致する。決定的な出力。
   - 依存: 2。仕様: `minimal_crate.md`、`harness.md`。

### snapshot 対応

4. **snapshot モジュール: API と正準化。** [x]
   - [snapshot.md](./snapshot.md) の snapshot 種別、公開 API、正準化
     規則（安定パス、改行正規化、非決定的フィールドなし）を実装する
     `src/snapshot.rs` を追加する。
   - テスト: 正準化のフィクスチャ。比較失敗が正確な diff を保持する。
   - 依存: 2。仕様: [snapshot.md](./snapshot.md)「Public API」
     「Canonicalization」。

5. **snapshot の更新ポリシーと決定性チェック。** [x]
   - ベースライン更新フロー（更新メカニズムの決定を解決する）と
     [snapshot.md](./snapshot.md) の決定性チェック（再レンダリング
     比較）を実装する。
   - テスト: 更新フローのラウンドトリップ。誤更新からの保護。決定性
     チェックが注入された非決定性を捕まえる。
   - 依存: 4。仕様: [snapshot.md](./snapshot.md)「Update Policy」
     「Determinism Checks」。

### カバレッジと健全性の契約

6. **カバレッジと pass/fail 比率の報告。** [x]
   - 既存の traceability と発見データから、stage ごとの仕様トレース
     カバレッジと、テスト戦略の 40/60 目標に対するコーパスの pass/fail
     比率を報告する。
   - task 2 traceability gap として、coverage-shape computation、manifest
     stored-status comparison、manifest order validation、obsolete references、
     missing manifest source files、missing listed tests、既存
     link-validator error-path tests を閉じる。duplicate manifest test paths、
     missing backrefs、unparsed listed tests、deferred required reasons、
     planned-without-tests warnings も含める。
   - テスト: 合成コーパス上の報告フィクスチャ。決定的な報告バイト列。
   - 依存: 3。仕様: [traceability.md](./traceability.md)、
     [architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

7. **stage 前提条件の検証。** [x]
   - staged model の規則を強制する: ケースの stage 前提条件がカバー済み
     または built-in 宣言済みになるまで、カバレッジのクレジットを
     与えない。
   - task 2 gap として、`depends_on` handling、built-in declarations、
     stage mismatch diagnostics、prerequisite が満たされる前の higher-stage
     coverage non-credit を閉じる。
   - テスト: 前提条件違反のフィクスチャが安定した診断で検証に失敗する。
   - 依存: 6。仕様: [staged_model.md](./staged_model.md)「Stage Rules」。

8. **fail/健全性契約の対応。** [x]
   - [fail_soundness.md](./fail_soundness.md) の期待失敗契約を実装する:
     ドメインごとの必須ケースの記録、期待失敗のアサーション（diagnostic
     コードと stage）、健全性ケースが黙って削除・弱体化されない
     リグレッション規則。
   - task 2 gap として、certificate/kernel `rejection_reason`、typed fail
     identity または同等の validation、false-arithmetic coverage、
     domain-required case bookkeeping を閉じる。
   - テスト: 契約のフィクスチャ。弱体化の試みの検出。
   - 完了: certificate/kernel `rejection_reason` validation、認識済み
     `soundness.*` case の shape/profile/phase gate、mode-aware missing-case
     diagnostics、false-arithmetic stable-key gating。所有する consumer runner が
     存在する前に real proof/certificate/kernel execution は捏造しない。
   - 依存: 6。仕様: [fail_soundness.md](./fail_soundness.md)。

9. **コーパスサイズとレビュー規則の検証。** [x]
   - [miz_corpus.md](./miz_corpus.md) のコーパス成長規則を検証する:
     ファイルサイズ指針、命名、コーパスクラスの配置、生成ポリシーの
     マーカー。
   - task 2 gap として、generated/fuzz/property origin metadata、
     reproducibility metadata、corpus policy 側に属する optional metadata
     retention、corpus-policy profile constraints、stress exclusion、fuzz-category
     preservation を閉じる。
   - テスト: 規則ごとの違反フィクスチャ。クリーンなコーパスは通る。
   - 完了: task 9 は `[origin]` provenance parsing/retention、corpus
     placement/profile gates、stress exclusion、fuzz-category preservation、
     upper-bound `.miz` size diagnostics、naming diagnostics、clean / violating
     corpora の metadata fixtures を実装した。
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
    - R-023 の paired work は、`mizar-resolve` task 23 のために
      `declaration-symbol` active runner command、active-tag validation、resolver
      diagnostic range が未仕様の間の public-code gate、summary reporting、
      traceable seed fixture 2 件を追加した。この task は予定されたすべての
      消費側 runner が着地するまで open のままにする。
    - 現在の task-10 ledger は、`mizar-parser` task 3（`parse-only`）、
      `mizar-resolve` task 23（`declaration-symbol`）、`mizar-checker` task 12
      （`type-elaboration`）を prepared/implemented increments として記録する。
      checker task 29、`mizar-vc` task 15、`mizar-atp` task 20、`mizar-kernel`
      task 17 は `paced/open` として記録し、placeholder runner や fake active
      fixture は作らない。
    - 依存: 5、8。仕様: [harness.md](./harness.md)。

11. **決定性スイート。** [x]
    - 発見順、計画、検証診断、報告、snapshot 比較が実行と
      プラットフォームをまたいでバイト安定であることのプロパティ的検証。
    - task 2 gap として、general snapshot hash determinism、
      parallel-equivalence modes、transitional parse-only `SurfaceAst` path
      外の nondeterminism diagnostics を閉じる。
    - 完了: task 11 は metadata plan と active runner report の canonical-byte
      stability tests、`SurfaceAst` 外の generic snapshot nondeterminism diagnostics、
      snapshot-level `verify_snapshot_parallel_equivalence` を追加した。
    - 依存: 6。仕様: [harness.md](./harness.md)「Determinism
      Requirements」。

12. **公開 enum の前方互換性ポリシー。** [x]
    - 各公開 enum（`Stage`、`ExpectedOutcome`、`ValidationSeverity`、…）に
      `mizar-frontend` task 25 の手続きを適用し、所有モジュール仕様に
      決定を記録する。
    - 完了: `crates/mizar-test/src` のすべての public enum は downstream
      `#[non_exhaustive]` であり、所有する EN/JA module spec は inventory と
      decision を記録する。lint coverage は source attributes と EN/JA inventory
      entries を guard する。
    - 依存: 2。仕様: 全モジュール仕様。

13. **二言語ドキュメント同期監査。** [x]
    - `doc/design/mizar-test/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 完了: [bilingual_sync_audit.md](./bilingual_sync_audit.md) は paired-file
      audit を記録した。task 14 の完了は下に記録する。
    - 依存: 12。仕様: リポジトリのドキュメント方針。

14. **増分/並列検証 regression matrix。** [x]
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
    - 完了: task 14 は architecture-22 scenario registry、sidecar metadata
      validation、deterministic plan/report summary、18 個すべての required scenario id
      を `planned` として覆う metadata-only `tests/property/architecture22_matrix_001`
      anchor を追加した。scenario-specific な clean/incremental/parallel/cache-race
      consumer runner はまだ準備されていないため、すべての row は inactive のままで、
      execution を捏造せず `active` gate は reject する。

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
cargo fmt --check
cargo test -p mizar-test
cargo clippy -p mizar-test --all-targets -- -D warnings
```

発見・expectation・stage を変更するタスクでは、コーパスランナーを
組み込む消費側（現状）も実行する:

```text
cargo test -p mizar-frontend
cargo test -p mizar-resolve
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

- この crate は最小に保つ: metadata validation、計画、比較、報告は
  payload-free のままにする。明示的な active runner subcommand だけが
  pipeline seam を実行し、その seam は実行する stage に限定する。
- stage id は `.expect.toml`、`spec_trace.toml`、消費側 enum と共有される
  正準値である。表示名はローカライズしてよいが、id はしてはならない。
- kernel の近傍では fail/健全性カバレッジが優先される。40/60 の
  pass/fail 比率はコーパス全体の目標であり、ディレクトリごとではない。
- snapshot ベースラインは内部レンダリングの安定性表面である。
  レンダリング自体は安定 artifact ではない。
