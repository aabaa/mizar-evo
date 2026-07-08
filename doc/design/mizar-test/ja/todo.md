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
      （`type-elaboration` external-gap runner）、task 16（source-derived
      builtin type-expression normalization）、task 17（source-derived
      builtin type-expression projection to `ResolvedTypedAst`）、task 18
      （source-derived reserve declaration semantic bridge）、task 19
      （reserve bridge `ResolvedTypedAstSummary::from_ast` readiness と次の
      builtin declaration inventory）、task 20（reserve bridge binder-only
      `CoreContext` readiness）、post-task-20 resolver R-G007 の parser-backed
      same-signature/different-return functor conflict active declaration-symbol seed
      と exact SymbolEnv-derived declaration-symbol pass payload assertion、checker
      task 50 の same-module attributed reserve evidence-query active fail slice、
      checker task 51 の same-module local mode reserve missing-expansion active
      fail slice、checker task 52 の same-module local structure reserve
      evidence-query active fail slice、checker task 53 の attributed local structure
      reserve evidence-query active fail slice、checker task 54 の attributed local mode
      reserve missing-expansion active fail slice、checker task 55 の bare same-module
      local mode expansion active pass slice、checker task 56 の one-edge same-module
      local-mode expansion chain active pass/gap slice、checker task 57 の same-module
      local-mode structure-RHS evidence-query active fail slice、checker task 58 の
      same-module local-mode attributed-builtin-RHS evidence-query active fail slice、checker
      task 59 の same-module attributed local-mode reserve evidence-query active fail slice、checker
      task 60 の same-module attributed local-mode structure-RHS evidence-query active fail slice、checker task 61 の same-module attributed local-mode attributed-builtin-RHS evidence-query active fail slice、checker task 62 の same-module local-mode structure-RHS chain evidence-query active fail slice、checker task 63 の same-module local-mode attributed-RHS chain evidence-query active fail slice、checker task 64 の same-module attributed local-mode bare-builtin chain evidence-query active fail slice、checker task 65 の same-module attributed local-mode structure-RHS chain evidence-query active fail slice、
      checker task 66 の same-module attributed local-mode attributed-builtin-RHS chain evidence-query active fail slice、
      checker task 67 の structure-qualified attribute extraction-gap active boundary slice、
      checker task 68 の argument-bearing local-mode reserve extraction-gap active boundary slice、
      checker task 69 の argument-bearing local-structure reserve extraction-gap active boundary slice、
      checker task 70 の bracket-form local-mode reserve extraction-gap active boundary slice、
      checker task 71 の bracket-form local-structure reserve extraction-gap active boundary slice、
      checker task 72 の two-edge bare local-mode chain active pass slice、checker task 73 の three-edge bare local-mode chain active pass slice、checker task 74 の structural bare local-mode chain active pass slice、checker task 75 の lower-stage forward local-mode active-range boundary、checker task 76 の lower-stage forward local-structure active-range boundary、checker task 77 の lower-stage forward local-attribute active-range boundary、checker task 78 の imported structure reserve extraction-gap boundary、checker task 79 の imported mode reserve extraction-gap boundary、checker task 80 の imported attribute reserve extraction-gap boundary、checker task 81 の argument-bearing local attribute reserve extraction-gap boundary と declaration-symbol suffix projection、checker task 82 の imported mode reserve provenance bridge、checker task 83 の imported structure reserve provenance bridge、checker task 84 の imported attribute reserve provenance bridge とともに
      prepared/implemented increments として記録する。
      checker task 85 の imported non-empty attribute reserve provenance bridge と
      checker task 86 の theorem formula extraction-gap boundary、checker task 106 の
      builtin equality theorem term/formula checker bridge、checker task 110 の imported predicate/functor
      theorem checker bridge、checker task 108 の builtin membership theorem
      checker bridge、checker task 107 の builtin inequality theorem checker
      bridge、checker task 109 の builtin type assertion theorem term/formula/type
      checker bridge、checker task 103 の imported attribute assertion formula
      extraction-gap boundary、checker task 104 の attribute-level non-empty
      imported attribute assertion formula extraction-gap boundary、checker task 111 の exact set-enumeration theorem
      checker bridge、checker task 112 の exact formula connective/quantifier shell
      checker bridge、checker task 88 の proof skeleton
      extraction-gap boundary、checker task 89 の statement proof extraction-gap
      boundary、checker task 90 の predicate/functor definition extraction-gap
      boundary、checker task 91 の attribute definition extraction-gap boundary、
      checker task 92 の mode/structure definition extraction-gap boundary、
      checker task 93 の proof-local declaration extraction-gap boundary、checker task 94 の proof-local inline definition extraction-gap boundary、checker task 95 の registration block extraction-gap boundary、checker task 96 の redefinition/notation extraction-gap boundary も
      prepared/implemented increment として記録する。
      checker task 29、`mizar-vc` task 15、`mizar-atp`
      task 20、`mizar-kernel` task 17 は `paced/open` として記録し、placeholder
      runner や fake active fixture は作らない。
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

15. **architecture-22 フォローアップ監査。** [x]
    - ソース/仕様ギャップ監査と二言語ドキュメント同期監査を再実行し、
      task 14 の scenario id、equivalence class、active/planned gating、
      traceability record を architecture 22 に照らしてレビューする。残る
      matrix gap をフォローアップタスクとして記録する。
    - 完了: task 15 は [bilingual_sync_audit.md](./bilingual_sync_audit.md) と
      [00.crate_plan.md](./00.crate_plan.md) に post-task-14 audit を記録した。
      18 個の scenario id/class と metadata-only trace anchor は architecture 20/22
      に一致する。新しく準備済みの consumer runner increment は確認できていないため、
      すべての row は `planned` のままである。残る active matrix execution は
      MT-AUDIT-014 として consumer-paced `test_gap` に分類する。`spec_gap`、
      `repo_metadata_conflict`、language behavior change、既存 expectation の
      semantic change は不要である。
    - 依存: 14。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md),
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md),
      リポジトリのドキュメント方針。

16. **Source-derived builtin type-expression bridge。** [x]
    - 完了: active `type_elaboration` の最初の real source-to-checker extraction
      slice を追加する。frontend parsing と resolver symbol collection が pass した後、
      reserve-only の unrecovered な builtin `set` / `object` `TypeExpression` node を
      checker-owned `TypeExpressionInput` payload に抽出し、`mizar-checker` で
      normalize し、最小の `TypedAst` shell を組み立てる。
    - 未対応の declaration、term、formula、coercion、attribute、mode /
      structure、overload、fact、CoreIr、ControlFlowIr、VC、proof seed payload は
      explicit external gap のままにする。既存 `.miz` や expectation semantics を
      rebaseline せず、prepared consumer execution なしに Architecture-22 row を
      昇格しない。
    - 依存: 10、`mizar-checker` task 12。仕様: [harness.md](./harness.md),
      [expectation_schema.md](./expectation_schema.md),
      [traceability.md](./traceability.md)、checker MC-G020。

17. **Source-derived builtin `ResolvedTypedAst` bridge。** [x]
    - 完了: task 16 の active `type_elaboration` source bridge を拡張し、
      normalized builtin `set` / `object` type-expression payload を `TypedAst` に
      組み立てた後、real checker-owned expression metadata、source-preserved node hint、
      empty cluster/overload predecessor output により `ResolvedTypedAst::assemble` へ
      投影する。runner は対応済み source type site がすべて resolved node、
      expression metadata、final type に diagnostic なしで到達することを確認する。
    - declaration extraction、non-builtin type head、attribute、term、formula、
      overload candidate、cluster fact、proof evidence、CoreIr、ControlFlowIr、
      VC seed、`proof_verification` row は producer/consumer seam が実行可能になるまで
      deferred のままにする。fake active fixture、public checker diagnostic code、
      CoreIr / ControlFlowIr / VC payload を追加しない。
    - 依存: 16、`mizar-checker` task 28。仕様: [harness.md](./harness.md)、
      checker `resolved_typed_ast.md`、checker MC-G020/MC-G027。

18. **Source-derived reserve declaration semantic bridge。** [x]
    - 完了: active `type_elaboration` source bridge を builtin type-expression
      site から reserve-only builtin declaration payload へ拡張した。runner は
      bare builtin `set` / `object` head を持つ unrecovered top-level `reserve`
      item を syntax-free source reserve payload へ抽出する。checker task 48 は
      その payload を checker-owned module `BindingEnv`、binding ごとの
      `DeclarationInput`、binding 固有の `TypeExpressionInput` site、
      `DeclarationChecker` output へ変換する producer seam を所有し、runner は
      その handoff を `TypedAst`、`ResolvedTypedAst` へ継続する。
      `reserve x, y for set` のように source type range を共有する場合も、binding
      ごとに distinct typed site を持つ。
    - 未対応の non-builtin declaration（task 96 の redefinition/notation extraction-gap boundary、task 95 の registration block extraction-gap boundary、task 94 の proof-local inline definition boundary、task 93 の proof-local declaration boundary、task 92 の mode/structure definition boundary を超えるもの）、task-84 `TypeCaseAttr` bridge、task-85
      negative `empty`/builtin-`set` bridge、task-80 boundary を超える imported attribute provenance、task-83 `R` bridge、task-97 `TypeCaseStruct` bridge、task-78 boundary を超える
      imported structure provenance、task 82 の provenance bridge を超える imported mode expansion payload、
      task-81 boundary を超える attribute argument payload、attributed / argument-bearing mode / structure head、
      structure base-shape payload、task-92 extraction-gap boundary を超える definition payload、task-93 extraction-gap boundary を超える proof-local declaration payload、task-94 extraction-gap boundary を超える inline definition payload、task-95 extraction-gap boundary を超える registration payload / activation / correctness payload、task-96 extraction-gap boundary を超える redefinition/notation payload、task-106/task-107/task-108/task-109/task-110/task-111 を超える numeric/signature/result-type payload と equality/inequality/membership/type-assertion/imported predicate-functor/set-enumeration semantic checking、task-112 を超える formula child/binder semantics、および task-86/task-103/task-104/task-105/task-88/task-89/task-93/task-94/task-95/task-96 extraction-gap boundary と task-112 checker bridge を超える
      imported predicate/functor semantic payload、membership operand expected-type construction/checking、inequality desugaring または equality semantic checking、broader type-assertion type payload extraction、type-assertion semantic checking、imported attribute assertion attribute-chain/provenance payload extraction、imported attribute-level non-empty assertion attribute-chain/provenance payload extraction、set-enumeration result-type/sethood payload extraction beyond task 111、negated attribute admissibility/semantic checking、attribute admissibility/semantic checking、quantifier binder/context payload、term / formula / theorem / proof payload、coercion、overload payload、fact、
      CoreIr、ControlFlowIr、VC payload、proof
      evidence は明示的な `type_elaboration.external_dependency.ast_payload_extraction`
      gap のままにする。real source-derived payload がまだ downstream consumer へ
      lower されていないため、CoreIr / ControlFlowIr / VC / proof row は昇格しない。
    - 依存: 16、17、checker MC-G011/MC-G016/MC-G020。仕様:
      [harness.md](./harness.md), [expectation_schema.md](./expectation_schema.md),
      [traceability.md](./traceability.md)。

19. **Reserve bridge core summary readiness and builtin declaration
    inventory。** [x]
    - 完了: active reserve-only builtin declaration bridge を拡張し、real
      checker-owned `ResolvedTypedAst` payload を `mizar-core` の
      `ResolvedTypedAstSummary::from_ast` に渡す。runner は successful active
      reserve pass case について、summary が source/module identity を保ち、checker
      recovery/diagnostic site を持たないことを確認する。
    - inventory 結果: この task では次の builtin declaration family を昇格しない。
      `let`、`given`、`consider`、quantified declaration は local scope、assumption、
      formula、constraint-discharge payload を必要とする。`set` は RHS term inference
      payload を必要とし、`reconsider` は coercion / obligation evidence を必要とし、
      `deffunc` / `defpred` は body / formal payload を必要とする。これらは、raw
      reconstruction や fake evidence なしに実行できる prepared active runner seam が
      存在するまで source-to-checker extraction gap に残す。
    - `ResolvedTypedAstSummary` read は summary-only であり、`CoreIr`、
      `ControlFlowIr`、VC seed、proof row、public checker diagnostic code は build /
      publish しない。
    - 依存: 18、`mizar-core` elaborator summary API。仕様:
      [harness.md](./harness.md), [expectation_schema.md](./expectation_schema.md),
      [traceability.md](./traceability.md)、core `elaborator.md`。

20. **Reserve bridge core context readiness。** [x]
    - 完了: active reserve-only builtin declaration bridge を拡張し、同じ real
      checker-owned `BindingEnv` と `ResolvedTypedAst` handoff を、抽出済み
      reserve binding ごとに 1 個の `CoreVariableSeed` と `CoreBinderSeed` を持つ
      `mizar-core` `CoreContextInput` へ渡し、`CoreItemSeed` は渡さない。runner は
      successful active reserve pass case について、source/module identity、binder
      source range、checker provenance、empty item registry、empty core diagnostics、
      empty core worklist を確認する。
    - これは binder/context readiness check のみである。reserve declaration は owner
      item、term、formula、proof、algorithm、obligation payload をまだ提供しないため、
      この task は `CoreIr`、`ControlFlowIr`、VC seed、proof row、public checker
      diagnostic code、新しい active fixture、expectation semantic change を build /
      publish しない。
    - 依存: 19、`mizar-core` `prepare_core_context`。仕様:
      [harness.md](./harness.md), [expectation_schema.md](./expectation_schema.md),
      [traceability.md](./traceability.md)、core `elaborator.md`。

### kernel 健全性監査フォローアップ(2026-07-03)

kernel 受理境界の監査
([soundness_argument.md](../../mizar-kernel/en/soundness_argument.md))は
harness 所有の所見 F7 と F8 を報告した。以下は監査由来の最小限の追加で
あり、より広い runner 成長は引き続き task 10 のペース配分に従う。

21. **必須ケース registry への訂正後 soundness 語彙(kernel F7)。** [x]
    - `REQUIRED_SOUNDNESS_CASES` と layout/expectation 文書を訂正済み
      kernel 拒否語彙で拡張する: `invalid_sat_refutation`、
      `context_mismatch`、`missing_provenance`、および normal policy 下の
      unsupported-legacy-certificate ケース(architecture 20 の必須
      カバレッジに従う)。現在これらの理由に非 `soundness.` の stable key
      を使っている certificate corpus の sidecar を、同一変更で新しい
      `soundness.certificate.*` key へ付け替える。拒否挙動は一切変えない。
    - 受け入れ条件: registry は従来どおり未知の `soundness.*` key を拒否
      する。23 件の監査 corpus が拡張後 registry を充足する。`mizar-test`
      plan error は 0 のまま。fail-soundness 簿記が訂正後ケースを covered
      と報告する。
    - 完了: task 21 は `invalid_sat_refutation`、`context_mismatch`、
      `missing_provenance`、normal policy 下の unsupported legacy certificate
      に対する訂正後 `soundness.certificate.*` required-case key を追加し、
      legacy `invalid_sat_proof` は保持する。訂正後 reason の既存 certificate
      sidecar は payload や rejection behavior を変えず、`domain = "certificate"` と
      soundness stable key を使うようになった。
    - 検証: `cargo test -p mizar-test`。
    - 依存: 8; corpus は mizar-kernel 監査(`f75af877`)由来。仕様:
      architecture 20; soundness_argument.md F7。

22. **certificate corpus ルート命名の調停(kernel F8)。** [x]
    - architecture 20 の `tests/kernel_evidence/` ディレクトリ一覧と実装済み
      `tests/certificates/` layout を調停する: 片方を rename するか、両者を
      相互参照する(相互参照なら docs のみ)。architecture 20(英日)と
      corpus README を同一変更で更新する。
    - task 22 で完了: architecture 20(英日)、certificate corpus README、
      crate plan、kernel soundness argument は、`tests/certificates/` を
      certificate/kernel-evidence corpus の正準 root として識別する。残る
      `tests/kernel_evidence/` 記述は歴史的な退役済み名称 note であり、
      規範的 corpus root ではない。
    - 検証: `cargo test -p mizar-test`; `git diff --check`。
    - 依存: なし。仕様: architecture 20; soundness_argument.md F8。

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
