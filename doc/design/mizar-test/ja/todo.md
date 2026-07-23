# mizar-test TODO

## Parser Task 46 / operator-declaration parse-only completion

- [x] exact active pass/fail pairをadmitし、sidecarをpinする。
- [x] `spec.en.10.operator_declarations.parser`をexact backlink付きcoveredとし、
  parse-only deferred-reserved-word setから今回exerciseしたoperator word 6個だけを除く。
- [x] existing corpus/expectation、production runner layout、semantic operator behavior、
  Task 49、Steps 6/7をunchangedに保つ。

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
| layout | [layout.md](./layout.md) | `src/layout.rs`、`src/path_rules.rs` | [x] discovery、missing-sidecar diagnostic、unknown-root inventory は実装済み。task 238 で Public API / ownership wording を同期し、task 239 で到達不能な sidecar-name diagnostic を削除し、task 240 で direct raw-order/missing-root/unknown-root coverage を追加済み |
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

- `layout`: task 238-240 で documented discovery API と harness/expectation
  ownership を同期し、到達不能な sidecar-name diagnostic を削除し、
  MT-AUDIT-020 を direct raw-order、missing-root、複数 unknown-root coverage で
  解消済みである。新しい root が入るたび coverage を同期する。
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
    - Core Task 31はこのopen task内のcompleted consumer-paced incrementであり、
      new mizar-test task idではない。existing Task-180 active type-elaboration caseは
      complete checker bundleをvalidateし、exact CoreIrを2回lowerしてcommitted
      full-byte baselineをverify-compareする。covered requirement 1件追加によりplan
      403/368、type 236/224となるが、active case 188、pass/fail 219/184、`.miz`、
      phase、diagnosticは不変。broader CoreIr/ControlFlowIr/proof-verificationはCore
      Task 32とprerequisiteにpacedされる。
    - Checker Task 247は同じopen task内の将来のnon-placeholder consumer
      increment 2件を命名した。`MT10-FS`は`formula-statement` stage/tag/reportを
      所有し、distinct `pass_formula_statement_reserved_variable_equality_smoke_001`
      sourceとsingular formula-statement sidecarを追加し、同checker bundleの
      corruptionをnegative runner coverageにする。existing type-elaboration
      fixtureと唯一のsidecarは不変。`MT10-AS`は`advanced-semantics`を所有し、新しいspec-derived
      non-Task-49 single-ordinary-functor/single-candidate reflexive-equality
      smokeをreal definition/application/candidate/ordinary-root producerへ通し、
      display-name shadowing越しにouter resolved identityを保持するdistinct Task-270
      definition-time capture smokeも実行する。さらにparser Task 47とchecker Tasks
      251/271-272後のexisting advanced-semantics omitted-`reconsider` caseを所有し、
      explicit non-accepting pending/blocked intentとproof searchなしをassertする。
      空/placeholder runnerや24-fixture
      Task-49 reconciliation setの早期activationは禁止。完全な
      dependency/blocked gateはchecker
      [payload_family_decomposition.md](../../mizar-checker/ja/payload_family_decomposition.md)
      をcanonicalとする。
    - Core Task 32はこのopen task内にさらに5個のnon-placeholder increment、
      `MT10-CIR-TE`、`MT10-CIR-FS`、`MT10-CIR-AS`、`MT10-CIR-ALG`、
      `MT10-CFG-PV`を命名した。Exact stage/tag/phase/artifact dependencyと
      corruption boundaryはCore
      [source_family_decomposition.md](../../mizar-core/ja/source_family_decomposition.md)
      をcanonicalとする。最初のgeneral Core snapshot integrationと最初の
      `SnapshotKind::ControlFlowIr` changeは各々最初のreal baselineと同時にlandし、
      empty infrastructureにしない。Consumer名の追加はcurrent runner/sidecar/
      trace status/coverageを変更しない。
    - historical selected task-10 ledger は、`mizar-parser` task 3
      （`parse-only`）、
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
      checker task 85 の imported non-empty attribute reserve provenance bridge、
      checker task 116 の imported positive empty attribute reserve provenance bridge と
      checker task 86 の theorem formula extraction-gap boundary、checker task 106 の
      builtin equality theorem term/formula checker bridge、checker task 110 の imported predicate/functor
      theorem checker bridge、checker task 108 の builtin membership theorem
      checker bridge、checker task 107 の builtin inequality theorem checker
      bridge、checker task 109 の builtin type assertion theorem term/formula/type
      checker bridge、checker task 113 の imported attribute assertion theorem
      checker bridge、checker task 114 の exact attribute-level non-empty
      imported attribute assertion theorem checker bridge、checker task 111 の exact set-enumeration theorem
      checker bridge、checker task 112 の exact formula connective/quantifier shell
      checker bridge、checker task 117 の exact formula constant kind checker
      bridge、checker task 118 の builtin-binary exact-token guard、checker task
      119 の exact reserved-variable equality active pass bridge、checker task 120 の
      exact reserved-variable membership active pass bridge、checker task 121 の exact
      reserved-variable inequality active pass bridge、checker task 122 の reflexive
      type-assertion gate と exact reserved-variable type-assertion active pass bridge、
      checker task 123 の exact distinct reserved-variable equality active pass bridge、
      checker task 124 の distinct pre-normalization source range と 1 semantic
      normalized type を持つ exact multiple-reserve-declaration equality active
      pass bridge、
      checker task 125 の left `object`、right/expected `set`、2 normalized
      semantic identity を持つ exact heterogeneous-reserve membership active
      pass bridge、
      checker task 126 の 4 raw local-mode result/expected input と real
      expansion RHS から normalized された 1 builtin-`set` identity を持つ exact
      direct-local-mode reserved-variable equality active pass bridge、
      checker task 127 の 4 raw outer-mode input、2 real expansion link、terminal
      RHS normalized provenance を持つ exact one-edge local-mode-chain
      reserved-variable equality active pass bridge、
      checker task 128 の 4 raw object-mode input と real expansion RHS から
      normalized された 1 builtin-`object` identity を持つ exact direct
      local-object-mode reserved-variable equality active pass bridge、
      checker task 129 の 4 raw outer-mode input、2 real expansion、terminal
      object-RHS normalized provenance を持つ exact one-edge
      local-object-mode-chain equality active pass bridge、
      checker task 130 の 4 raw mode input、1 real expansion、terminal set-RHS
      provenance、fact-free pre-desugaring checked inequality を持つ exact
      direct-local-mode inequality active pass bridge、
      checker task 131 の 4 raw object-mode input、1 real expansion、terminal
      object-RHS provenance、fact-free pre-desugaring checked inequality を持つ
      exact direct-local-object-mode inequality active pass bridge、
      checker task 132 の 4 raw outer-mode input、2 real expansion、terminal
      set-RHS provenance、fact-free pre-desugaring checked inequality を持つ exact
      one-edge local-mode-chain inequality active pass bridge、
      checker task 133 の 4 raw outer-mode input、2 real expansion、terminal
      object-RHS provenance、fact-free pre-desugaring checked inequality を持つ
      exact one-edge local-object-mode-chain inequality active pass bridge、
      checker task 134 の 4 raw outer-mode input、3 real expansion、terminal
      set-RHS provenance、fact-free checked equality を持つ exact two-edge
      local-mode-chain equality active pass bridge、
      checker task 135 の 4 raw outer-mode input、3 real expansion、terminal
      object-RHS provenance、fact-free checked equality を持つ exact two-edge
      local-object-mode-chain equality active pass bridge、
      checker task 136 の 4 raw outer-mode input、3 real expansion、terminal
      set-RHS provenance、fact-free pre-desugaring checked inequality を持つ exact
      two-edge local-mode-chain inequality active pass bridge、
      checker task 137 の 4 raw outer-mode input、3 real expansion、terminal
      object-RHS provenance、fact-free pre-desugaring checked inequality を持つ
      exact two-edge local-object-mode-chain inequality active pass bridge、
      checker task 138 の raw local-mode subject、独立した builtin-set asserted
      type、1 real expansion、terminal set-RHS provenance、fact-free checked type
      assertion を持つ exact direct local-mode reserved-variable type-assertion
      active pass bridge、
      checker task 88 の proof skeleton
      extraction-gap boundary、checker task 89 の statement proof extraction-gap
      boundary、checker task 90 の predicate/functor definition extraction-gap
      boundary、checker task 91 の attribute definition extraction-gap boundary、
      checker task 92 の mode/structure definition extraction-gap boundary、
      checker task 93 の proof-local declaration extraction-gap boundary、checker task 94 の proof-local inline definition extraction-gap boundary、checker task 95 の registration block extraction-gap boundary、checker task 96 の redefinition/notation extraction-gap boundary も
      prepared/implemented increment として記録する。
      この historical inline selection の latest-numbered checker entry は
      task 138 である。checker tasks 139-236 の詳細 lifecycle は paired
      [crate plan](./00.crate_plan.md)、[harness](./harness.md)、
      [traceability](./traceability.md) で管理する。active Task 233 corpus は
      395 cases / 359 requirements 内の type-elaboration case 180 件、
      type-elaboration coverage 227/215、pass/fail 211/184 を持ち、Step 5 は
      active、Steps 6/7 は deferred である。checker task 233 は既存 expectation を
      rebaseline しない最新の active exact parenthesized builtin-object equality row
      を供給する。
      checker task 234 は最新の active exact seven-expansion set-terminal
      full-distance six-hop asserted-head row を供給する。fixture と backlink 6 件は
      既存 expectation を rebaseline せず 396 cases / 360 requirements、
      type-elaboration 228/216、pass/fail 212/184、active runner 181 を計上する。
      checker task 236 は six link を直接検証し、object edge 1 本を terminal-only
      とする最新の active exact object-terminal full-distance six-hop sibling を
      供給する。backlink 6 件と先行 owner 57 件を持ち、既存 expectation を
      rebaseline せず 397 cases / 361 requirements、type-elaboration 229/217、
      pass/fail 213/184、active runner 182 を計上する。
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
      structure base-shape payload、task-92 extraction-gap boundary を超える definition payload、task-93 extraction-gap boundary を超える proof-local declaration payload、task-94 extraction-gap boundary を超える inline definition payload、task-95 extraction-gap boundary を超える registration payload / activation / correctness payload、task-96 extraction-gap boundary を超える redefinition/notation payload、task-106/task-107/task-108/task-109/task-110/task-111/task-113/task-114 を超える numeric/signature/result-type payload と equality/inequality/membership/type-assertion/imported predicate-functor/set-enumeration semantic checking、task-112 を超える formula child/binder semantics、および task-86/task-105/task-88/task-89/task-93/task-94/task-95/task-96 extraction-gap boundary と task-112 / task-113 / task-114 checker bridge を超える
      imported predicate/functor semantic payload、membership operand expected-type construction/checking、inequality desugaring または equality semantic checking、broader type-assertion type payload extraction、type-assertion semantic checking、imported attribute assertion attribute-chain/provenance payload extraction、imported attribute-level non-empty assertion attribute-chain/provenance payload extraction、set-enumeration result-type payload extraction beyond task 111、negated attribute admissibility/semantic checking、attribute admissibility/semantic checking、quantifier binder/context payload、term / formula / theorem / proof payload、coercion、overload payload、fact、
      CoreIr、ControlFlowIr、VC payload、proof
      evidence は明示的な `type_elaboration.external_dependency.ast_payload_extraction`
      gap のままにする。separately traced exact Task-180 CoreIr snapshotはCore Task
      31でpromote済みである。対応するreal source-derived payloadがdownstream
      consumerへlowerされていないため、broader CoreIr / ControlFlowIr / VC / proof
      rowはdeferredのまま。
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

## Task 241 Active Addendum

- [x] exact test-first builtin-set `(x) <> x` fixture、Chapter 04/13/14/16 から
  導く新規 expectation、shared backlink 4 件、dedicated checker row 1 件を追加
  する。既存 fixture/expectation は変更も rebaseline もしない。
- [x] active metadata/CLI の runner 183 assertion を追加し、real frontend/
  resolver/checker payload を exact/negative/corruption/immutable/focused
  regression/先行 owner 54 件との bidirectional test で保護する。
- [x] 398 cases / 362 requirements、type-elaboration 230/218、pass/fail 214/184
  を同期する。parenthesized membership、imported/other parenthesized variant、
  proof/IR/VC、broader semantics は Task 241 の credit 外。Step 5 は active、
  Steps 6/7 は deferred。

## Task 242 Active Addendum

- [x] exact test-first builtin-object `(x) <> x` fixture、Chapter 03/04/13/14/16
  から導く expectation、shared backlink 5 件、dedicated checker row 1 件を
  追加する。既存 expectation は変更も rebaseline もしない。
- [x] active metadata/CLI の runner 184 assertion を追加し、real frontend/
  resolver/checker payload を exact/negative/corruption/immutable/focused
  regression/先行 owner 55 件との bidirectional test で保護する。
- [x] 399 cases / 363 requirements、type-elaboration 231/219、pass/fail 215/184
  を同期する。parenthesized membership と active imported provenance は Task
  242 の credit 外。未成立 imported expansion/evidence/signature payload と
  proof/CoreIr/ControlFlowIr/VC は deferred。Step 5 は active、Steps 6/7 は
  deferred。

## Task 243 Active Addendum

- [x] exact test-first builtin-set `(x) in x` fixture、Chapter 04/13/14/16 から
  導く expectation、shared backlink 4 件、dedicated checker row 1 件を追加
  する。既存 expectation は変更も rebaseline もしない。
- [x] active metadata/CLI の runner 185 assertion を追加し、real frontend/
  resolver/checker payload を exact/negative/corruption/immutable/focused
  regression/先行 owner 56 件との bidirectional test で保護する。left expected
  input 0 個と unexpected-left/wrong-right/missing-right corruption を検証する。
- [x] 400 cases / 364 requirements、type-elaboration 232/220、pass/fail 216/184
  を同期する。extraction gap の解除は exact source だけ。object-left/set-
  right parenthesized membership と active imported provenance は Task 243 の
  credit 外。未成立 imported expansion/evidence/signature payload と proof/
  CoreIr/ControlFlowIr/VC は deferred。Step 5 は active、Steps 6/7 は deferred。

## Task 244 Active Addendum

- [x] exact two-reserve source `reserve x for object; reserve y for set; theorem
  ParenthesizedHeterogeneousReserveMembershipPayloadBoundary: (x) in y;` の
  test-first `.miz` / expectation pair を追加。
- [x] ordered distinct binding、ordinal 2/3、`BindingId(0/1)`、written range に
  anchor された object/set identity 2件、inferred term 2件、type entry 5件、
  right-only expected-set input、wrapper semantics/coercion なしの checked
  membership を real frontend/resolver runner で active 化。
- [x] finite exact/near-miss/provenance/corruption、immutable output、既存 binary
  owner 57件、Tasks 120/125/223/233/241/242/243、real imported-mode-gap
  diagnostic fixture、real active sidecar を cover。
- [x] shared backlink 5件 + dedicated requirement 1件を追加し、active runner
  186、cases/requirements 401/365、type 233/221、pass/fail 217/184 を同期。
- [x] extraction gap の解除は exact source だけ。その他 parenthesized shape
  と imported-positive provenance は Task 244 credit 外。未成立 imported
  expansion/evidence/signature payload と proof/CoreIr/ControlFlowIr/VC は
  deferred。Step 5 は active、Steps 6/7 は deferred。

## Task 245 Active Addendum

- [x] exact test-first `x in (x)` fixture/expectation、Chapters 04/13/14/16 の
  shared backlink 4件、dedicated checker row 1件を追加。
- [x] explicit `Right` wrapper side と Task-245-only key/config/role を active
  化し、従来6 `Left` route を維持。
- [x] real frontend/resolver/checker payload、right-inner expected ownership、
  side/config/range/constraint corruption、Task-243 cross-route、immutable/
  module、既存 owner 58件の双方向を検証。
- [x] runner 187、plan 402/366、type 234/222、pass/fail 218/184 を同期。
  その他 shape/imported-positive は credit 外、未成立 imported/proof/
  downstream payload は deferred。Step 5 active、Steps 6/7 deferred。

## Task 246 Active Addendum

- [x] exact 3-mode set-terminal `(z) = z` fixture と trace 6件を既存
  expectation 変更なしで追加。
- [x] conditional mode-node admission、expansion 3件、raw Outer input 4件、
  ordinal 1/2 の `BindingId(0)`、terminal set identity 1件、term 2件、entry
  6件、constraint 2件、checked equality 1件、wrapper ownership なしを要求。
- [x] 全5 nonidentity order、finite structure/provenance/corruption、Tasks
  134/223、immutable/module、既存 owner 59件、real sidecar を cover。
- [x] runner 188、plan 403/367、type 235/223、pass/fail 219/184 を同期。
  Step 5 active、Steps 6/7 deferred。

## Runner Module-Boundary Refactor Backlog

優先度: 次の Step 5 semantic bridge を追加する前に、この maintenance series
を完了する。新しい language/runner coverage ではなく、source layout と
reviewability の behavior-preserving `design_drift` と分類する。Task 246
closeout 時点の `src/runner.rs` は 111,262 行で、`#[cfg(test)]` helper 137件を
含む pre-test-module prefix 17,142行の後に、`#[test]` attribute 272件を含む
単一の test module 約94,120行が続く。

- [x] Task 248 で runner boundary を監査し、paired EN/JA module-boundary 文書を
  追加する。
  orchestration、parse-only、declaration-symbol、type-elaboration、source
  extraction、payload validation、fixture builder、corruption test の ownership
  を inventory し、dependency map、target source layout、move order、exit
  criteria を記録する。source move 前に paired `00.crate_plan.md` へ task ID、
  affected files/tests、coverage-audit impact、completion conditions、forbidden
  behavior を記録する。audit/docs-only task として commit する。
- [x] Task 249 で monolithic private `mod tests` を `runner.rs` から
  `src/runner/tests.rs` へ機械的に移動した。
  module privacy、test name/discovery、helper behavior、全 public API を保持し、
  rename、deduplication、generalization、semantic cleanup と混ぜない。move
  だけを 1 task/commit とする。
- [x] private tests を shared support、parse-only、declaration-symbol、
  type-elaboration owner に分割する。必要なら type-elaboration を cohesive な
  source-bridge family ごとに追加分割し、family ごとに bounded move-only
  task/commit として cross-owner isolation test を保持する。
  Tasks 250-252、253A、254、253B で shared-support、parse-only、baseline
  type-elaboration source-extraction/handoff、先頭 reserved/binary、
  non-long-chain mode、direct reserved fragment は完了。Tasks 253/253B は完了し、
  Tasks 255A-255E で先頭/four-edge/three-edge object/two-edge object
  asserted-head fragment と最後の type-assertion asserted-head fragment は完了。
  parent Task 255 と Task 256 は完了。Task 257A で先頭 binary-route
  fixture/isolation family は完了。fresh authority review により Task 180
  formula-constant fixture を分離した。Tasks 257A-257H と parent Task 257 は
  完了し、private test layout は安定した。
- [x] test layout 安定後、production helper を監査済み phase/ownership boundary
  で分割する。`runner.rs` は public facade と top-level orchestration owner に
  限定する。internal visibility を最小に保ち、detail key、diagnostic、payload
  contract、fixture ownership、ordering、fail-closed behavior を変更しない。
  Tasks 258-259 で private shared frontend と parse-only owner、Tasks 260A-260B
  で shared resolver leaf と declaration-symbol owner、Task 261 で
  fixture/import-summary owner、Tasks 262A-262B で common source-AST leaf、
  Task 262C で reserve type-expression/symbol-projection leaf、Task 262D で shared
  exact fixture-import AST projection、Task 262E で reserve declaration/local-mode
  extraction family、Task 262F で standalone formula-constant source leaf だけを
  移動し、Task 262G で残る formula extractor が共有する exact numeral
  AST-projection prerequisite を移動した。Task 262H0 で bounded builtin
  equality/inequality/membership family の test-only preservation prerequisite
  は完了し、Task 262H でその後の move も完了した。Task 262I0 で bounded
  builtin type-assertion family の test-only preservation prerequisite は完了し、
  Task 262I で I0 後にその family だけを移動した。Task 262J0 で imported
  predicate/functor family の test-only preservation prerequisite は完了した。
  shared symbol projection は Task 262J1、exact imported predicate/functor family は
  Task 262J2 で移動した。fresh inventory は exact imported attribute assertion
  family を完了済み test-only preservation Task 262K0、その後の move-only Task
  262K に分割し、両方とも完了した。次の fresh inventory は set-enumeration
  family を test-only preservation Task 262L0、その後の move-only Task 262L に
  分割し、両方とも完了した。次の fresh inventory は connective/quantifier family
  を test-only preservation Task 262M0、その後の move-only Task 262M に分割し、
  両方とも完了した。fresh inventory は残る reserved-variable formula work を
  test-only preservation Task 262N0、shared source-substrate Task 262N、direct-
  binary Task 262O、parenthesized-binary Task 262P、type-assertion Task 262Q に
  分割する。Tasks 262N0/262N/262O/262P は完了した。fresh review により move-only
  Task 262Q の前へ test-only preservation Task 262Q0 を追加し、両方と parent Task
  262 は完了した。fresh dependency inventory は Task 263 を分割し、bounded
  checker-handoff substrate Task 263A を最初に選び、Task 263A は完了した。fresh
  inventory は common frontend diagnostic projection Task 263B を次に選び、これも
  完了した。fresh inventory が選んだ expected-result/failure-projection Task 263C も
  exact-body/byte-stability を維持して完了した。fresh Task 263 inventory は次の
  正確な50行 type active-admission gate Task 263D を選び、exact-body/byte-stability を
  維持して完了した。fresh Task 263 inventory が選んだ正確な33行 checker-output
  transport substrate Task 263E も exact-body/byte-stability を維持して完了した。
  fresh Task 263 inventory が選んだ正確な277行 checker-output builder family Task
  263F も exact-body/byte-stability を維持して完了した。fresh inventory は正確な229行
  type-assertion validator/shared normalized-type predicate family Task 263G を選び、
  exact-body/byte-stability を維持して完了した。fresh inventory は正確な380行
  binary-formula validator/helper family Task 263H を選び、exact-body/byte-stability を
  維持して完了した。fresh inventory は正確な67行 config-independent parenthesized-
  validator core Task 263I を選び、exact-body/byte-stability を維持して完了した。
  fresh inventory は正確な46行 type-assertion result/detail core Task 263J を選び、
  exact-body/byte-stability を維持して完了した。次の fresh inventory は正確な36行
  binary-formula result/detail core を Task 263K として選び、exact-body/byte-stability
  を維持して完了した。fresh inventory は正確な16行 parenthesized-binary output-
  detail core を Task 263L として選び、exact-body/byte-stability を維持して完了した。
  fresh inventory は正確な17行 parenthesized-binary payload-detail wrapper を Task 263M
  として選び、exact-body/byte-stability を維持して完了した。fresh inventory は正確な
  7 fragment/720行の cohesive parenthesized config/named-route owner を Task 263N として
  選び、exact-body/byte-stability を維持して完了した。fresh inventory は正確な
  8 fragment/546行のleading direct-binary route ownerをTask 263Oとして選び、
  token-identical body/byte-stabilityを維持して完了した。fresh inventoryは訂正済み
  正確な5 fragment/313行multiple-reserve declaration binary route familyをTask 263P
  として選び、token-identical body/byte-stabilityを維持して完了した。fresh inventory
  は正確な5 fragment/116行base reserved-variable membership/inequality route familyを
  Task 263Qとして選び、token-identical body/byte-stabilityを維持して完了した。fresh
  inventoryは正確な10 fragment/183行direct local-mode membership/equality/inequality
  route familyをTask 263Rとして選び、token-identical body/byte-stabilityを維持して完了した。
  fresh inventoryは正確な10 fragment/190行direct local-object-mode membership/equality/
  inequality route familyをTask 263Sとして選び、token-identical body/byte-stabilityを維持
  して完了した。fresh inventoryは正確な14 fragment/207行chained local-mode membership/
  equality/inequality route familyをTask 263Tとして選び、token-identical body/byte-
  stabilityを維持して完了した。fresh inventoryは正確な9 fragment/229行chained local-
  object-mode membership/equality/inequality route familyをTask 263Uとして選び、token-
  identical body/byte-stabilityを維持して完了した。fresh inventoryは正確な15 fragment/
  222行two-edge local-mode membership/equality/inequality route familyをTask 263Vとして
  選び、token-identical body/byte-stabilityを維持して完了した。fresh inventoryは
  正確な11 fragment/241行two-edge local-object-mode membership/equality/inequality route
  familyをTask 263Wとして選び、token-identical body/byte-stabilityを維持して完了した。
  fresh inventoryは正確な15 fragment/242行three-edge local-mode membership/equality/
  inequality route familyをTask 263Xとして選び、token-identical body/byte-stabilityを
  維持して完了した。fresh inventoryは正確な11 fragment/258行three-edge local-object-
  mode membership/equality/inequality route familyをTask 263Yとして選び、token-
  identical body/byte-stabilityを維持して完了した。fresh inventoryは正確な15 fragment/
  252行four-edge local-mode membership/equality/inequality route familyをTask 263Zとして
  選び、token-identical body/byte-stabilityを維持して完了した。fresh inventoryは
  正確な11 fragment/273行four-edge local-object-mode membership/equality/inequality route
  familyをTask 263ZAとして選び、token-identical body/byte-stabilityを維持して完了した。
  fresh dependency inventoryは正確な2 fragment/74行の共有long-chain seven-expansion
  definition tableをTask 263ZBとして選び、token-identical body/byte-stabilityを維持して
  完了した。fresh inventoryはparent dependencyやconsumer-family混在なしでlocal-mode/
  local-object-mode long-chain binary route familyを分離でき、正確な15 fragment/176行の
  local-mode long-chain membership/equality/inequality binary route familyをTask 263ZCとして
  選び、token-identical body/byte-stabilityを維持して完了した。fresh inventoryは
  正確な15 fragment/186行local-object-mode long-chain membership/equality/inequality
  binary route siblingをTask 263ZDとして選び、token-identical body/byte-stabilityを
  維持して完了した。fresh inventoryは正確な5 fragment/52行local-mode long-chain
  reserved-variable type-assertion routeをTask 263ZEおよびprivate
  `type_assertion_routes.rs`の最初のnonempty ownerとして選び、token-identical body/
  byte-stabilityを維持して完了した。fresh inventoryは正確な5 fragment/48行local-mode
  long-chain same-mode asserted-head routeを同じownerのTask 263ZFとして選び、token-
  identical body/byte-stabilityを維持して完了した。fresh inventoryは正確な5 fragment/
  50行local-mode long-chain immediate-radix asserted-head routeを同じownerのTask 263ZG
  として選び、token-identical body/byte-stabilityを維持して完了した。fresh inventoryは
  正確な5 fragment/51行local-mode long-chain two-hop asserted-head routeを同じownerの
  Task 263ZHとして選び、token-identical body/byte-stabilityを維持して完了した。fresh
  inventoryは正確な5 fragment/54行local-mode long-chain three-hop asserted-head routeを
  同じownerのTask 263ZIとして選び、token-identical body/byte-stabilityを維持して完了した。
  fresh inventoryは正確な5 fragment/55行local-mode long-chain four-hop asserted-head routeを
  同じownerのTask 263ZJとして選び、token-identical body/byte-stabilityを維持して完了した。
  fresh inventoryは正確な5 fragment/56行local-mode long-chain five-hop asserted-head
  routeを同じownerのTask 263ZKとして選び、token-identical body/byte-stabilityを維持して
  完了した。fresh inventoryは正確な5 fragment/55行local-mode long-chain six-hop
  asserted-head routeを同じownerのTask 263ZLとして選び、token-identical body/byte-
  stabilityとstale local-table runner exposure除去を維持して完了した。fresh inventoryは
  正確な5 fragment/58行local-object-mode long-chain six-hop asserted-head routeを同じ
  ownerのTask 263ZMとして選び、token-identical body/byte-stability/object-terminal
  fail-closed preservationを維持して完了した。fresh inventoryは正確な5 fragment/57行
  local-object-mode long-chain five-hop asserted-head routeを同じownerのTask 263ZNとして
  選び、token-identical body/byte-stability/object-terminal fail-closed preservationを
  維持して完了した。fresh inventoryは正確な5 fragment/56行local-object-mode long-chain
  four-hop asserted-head routeを同じownerのTask 263ZOとして選び、token-identical body/
  byte-stability/object-terminal fail-closed preservationを維持して完了した。fresh
  inventoryは正確な5 fragment/55行local-object-mode long-chain three-hop asserted-head
  routeを同じownerのTask 263ZPとして選び、token-identical body/byte-stability/object-
  terminal fail-closed preservationを維持して完了した。fresh
  inventoryは正確な5 fragment/54行local-object-mode long-chain two-hop asserted-head
  routeを同じownerのTask 263ZQとして選び、token-identical body/byte-stability/object-
  terminal fail-closed preservationを維持して完了した。fresh
  inventoryは正確な5 fragment/52行local-object-mode long-chain immediate-radix
  asserted-head routeを同じownerのTask 263ZRとして選び、token-identical body/byte-
  stability/object-terminal fail-closed preservationを維持して完了した。fresh
  inventoryは正確な5 fragment/50行local-object-mode long-chain same-mode asserted-head
  routeを同じownerのTask 263ZSとして選び、token-identical body/byte-stability/object-
  terminal fail-closed preservationを維持して完了した。fresh
  inventoryは正確な5 fragment/52行local-object-mode long-chain reserved-variable builtin
  type-assertion routeを同じownerのTask 263ZTとして選び、token-identical body/byte-
  stability/direct sibling-table ownership/object-terminal fail-closed preservationを維持して
  完了した。fresh
  inventoryは正確な5 fragment/53行direct local-object-mode reserved-variable builtin type-
  assertion routeを同じownerのTask 263ZUとして選び、token-identical body/byte-stability/
  object-terminal fail-closed preservationを維持して完了した。fresh
  inventoryは正確な5 fragment/67行chained local-object-mode reserved-variable builtin
  type-assertion routeを同じownerのTask 263ZVとして選び、token-identical body/byte-
  stability/two-expansion object-terminal chain/fail-closed preservationを維持して完了した。
  fresh inventoryは残るlocal-object-mode type-assertion/asserted-head routeへ戻るため、
  正確な5 fragment/71行two-edge local-object-mode reserved-variable builtin type-
  assertion routeを同じownerのTask 263ZWとして選び、token-identical body/byte-stability/
  three-expansion object-terminal chain/fail-closed behaviorを維持して完了した。fresh
  inventoryは残るlocal-object-mode type-assertion/asserted-head routeへ戻るため、Task 263
  parent itemはopenのまま。正確な5 fragment/82行three-edge local-object-mode reserved-
  variable builtin type-assertion routeを同じownerのTask 263ZXとして選び、token-identical
  body/byte-stability/four-expansion object-terminal chain/fail-closed behaviorを維持して
  完了した。fresh inventoryは残るlocal-object-mode type-assertion/asserted-head routeへ戻り、
  正確な5 fragment/81行four-edge local-object-mode reserved-variable builtin type-assertion
  routeを同じownerのTask 263ZYとして選び、token-identical body/byte-stability/five-
  expansion object-terminal chain/fail-closed behaviorを維持して完了した。fresh inventoryは
  残るlocal-object-mode asserted-head routeへ戻り、正確な5 fragment/55行direct local-
  object-mode same-mode asserted-head routeを同じownerのTask 263ZZとして選び、token-
  identical body/byte-stability/one-expansion object-terminal same-mode behavior/fail-closed
  behaviorを維持して完了した。fresh inventoryは残るlocal-object-mode asserted-head routeへ
  戻り、正確な5 fragment/63行chained local-object-mode same-mode asserted-head routeを
  同じownerのTask 263ZZAとして選ぶ。token-identical body、byte stability、two-expansion
  object-terminal same-mode behavior、fail-closed behaviorを保持して完了した。fresh
  inventoryは残るlocal-object-mode asserted-head routeへ戻り、正確な5 fragment/65行
  chained local-object-mode immediate-radix asserted-head routeを同じownerのTask 263ZZB
  として選ぶ。token-identical body、byte stability、two-expansion object-terminal
  immediate-radix behavior、fail-closed behaviorを保持して完了した。fresh inventoryは
  残るlocal-object-mode asserted-head routeへ戻り、正確な5 fragment/68行two-edge
  local-object-mode same-mode asserted-head routeを同じownerのTask 263ZZCとして選ぶ。
  token-identical body、byte stability、three-expansion object-terminal same-mode
  behavior、fail-closed behaviorを維持して完了した。fresh inventoryは残るlocal-object-
  mode asserted-head routeへ戻り、正確な5 fragment/72行two-edge local-object-mode
  immediate-radix asserted-head routeを同じownerのTask 263ZZDとして選ぶ。token-
  identical body、byte stability、three-expansion object-terminal immediate-radix behavior、
  fail-closed behaviorを維持して完了した。fresh inventoryは残るlocal-object-mode
  asserted-head routeへ戻り、正確な5 fragment/71行two-edge local-object-mode two-hop
  asserted-head routeを同じownerのTask 263ZZEとして選ぶ。token-identical body、byte
  stability、three-expansion object-terminal two-hop behavior、fail-closed behaviorを
  維持して完了した。fresh inventoryは残るlocal-object-mode asserted-head routeへ戻り、
  正確な5 fragment/83行three-edge local-object-mode two-hop asserted-head routeを同じ
  ownerのTask 263ZZFとして選ぶ。token-identical body、byte stability、four-expansion
  object-terminal two-hop behavior、fail-closed behaviorを維持して完了した。fresh
  inventoryは残るlocal-object-mode asserted-head routeへ戻り、正確な5 fragment/89行
  four-edge local-object-mode two-hop asserted-head routeを同じownerのTask 263ZZGとして
  選ぶ。token-identical body、byte stability、five-expansion object-terminal two-hop
  behavior、fail-closed behaviorを維持して完了した。fresh inventoryは残るlocal-
  object-mode asserted-head routeへ戻り、正確な5 fragment/84行three-edge local-
  object-mode three-hop asserted-head routeを同じownerのTask 263ZZHとして選ぶ。
  token-identical body、byte stability、four-expansion object-terminal three-hop
  behavior、fail-closed behaviorを維持して完了した。fresh inventoryは正確な5 fragment/
  91行four-edge local-object-mode three-hop asserted-head routeを同じownerのTask 263ZZI
  として選ぶ。token-identical body、byte stability、five-expansion object-terminal
  three-hop behavior、fail-closed behaviorを維持して完了した。fresh inventoryは残る
  local-object-mode asserted-head routeへ戻り、正確な5 fragment/92行four-edge local-
  object-mode four-hop asserted-head routeを同じownerのTask 263ZZJとして選ぶ。token-
  identical body、byte stability、five-expansion object-terminal four-hop behavior、
  fail-closed behaviorを維持して完了した。fresh inventoryは残るlocal-object-mode
  asserted-head routeへ戻り、正確な5 fragment/81行three-edge local-object-mode
  immediate-radix asserted-head routeを同じownerのTask 263ZZKとして選ぶ。token-
  identical body、byte stability、four-expansion object-terminal immediate-radix
  behavior、fail-closed behaviorを維持して完了した。fresh inventoryは残るlocal-
  object-mode asserted-head routeへ戻り、正確な5 fragment/86行four-edge local-
  object-mode immediate-radix asserted-head routeを同じownerのTask 263ZZLとして
  選ぶ。token-identical body、byte stability、five-expansion object-terminal
  immediate-radix behavior、fail-closed behaviorを維持して完了した。fresh inventoryは
  残るlocal-object-mode asserted-head routeへ戻り、正確な5 fragment/78行four-edge
  local-object-mode same-mode asserted-head routeを同じownerのTask 263ZZMとして
  選ぶ。token-identical body、byte stability、five-expansion object-terminal same-
  mode behavior、fail-closed behaviorを維持して完了した。fresh inventoryは残る
  local-object-mode asserted-head routeへ戻り、正確な5 fragment/73行three-edge
  local-object-mode same-mode asserted-head routeを同じownerのTask 263ZZNとして
  選ぶ。token-identical body、byte stability、four-expansion object-terminal same-
  mode behavior、fail-closed behaviorを維持して完了した。fresh inventoryでは
  `runner.rs`に物理的local-object-mode asserted-head routeは残らず、残るproduction-
  helper familyへ戻る。同じownerのTask 263ZZOとして、正確な5 fragment/53行direct
  local-mode same-mode asserted-head routeを選ぶ。token-identical body、byte
  stability、one-expansion set-terminal same-mode behavior、fail-closed behaviorを
  維持して完了した。fresh inventoryは残るproduction-helper familyへ戻り、同じ
  ownerのTask 263ZZPとして正確な5 fragment/62行chained local-mode same-mode
  asserted-head routeを選ぶ。token-identical body、byte stability、two-expansion
  set-terminal same-mode behavior、fail-closed behaviorを維持し、immediate-radix
  siblingは移動せず完了した。fresh inventoryは残るproduction-helper familyへ
  戻り、同じownerのTask 263ZZQとして正確な5 fragment/61行chained local-mode
  immediate-radix asserted-head routeを選ぶ。token-identical body、byte stability、
  two-expansion set-terminal immediate-radix behavior、fail-closed behaviorを維持し、
  two-edge siblingは移動せず完了した。fresh inventoryは残るproduction-helper
  familyへ戻り、同じownerのTask 263ZZRとして正確な5 fragment/66行two-edge
  local-mode immediate-radix asserted-head routeを選ぶ。token-identical body、byte
  stability、three-expansion set-terminal immediate-radix behavior、fail-closed
  behaviorを維持し、two-hop siblingは移動せず完了した。fresh inventoryは残る
  production-helper familyへ戻り、同じownerのTask 263ZZSとして正確な5 fragment/
  67行two-edge local-mode two-hop asserted-head routeを選ぶ。token-identical body、
  byte stability、three-expansion set-terminal two-hop behavior、fail-closed behavior
  を維持し、three-edge siblingは移動せず完了した。fresh inventoryは残るproduction-
  helper familyへ戻り、同じownerのTask 263ZZTとして正確な5 fragment/72行three-
  edge local-mode two-hop asserted-head routeを選ぶ。token-identical body、byte
  stability、four-expansion set-terminal two-hop behavior、fail-closed behaviorを
  維持し、four-edge siblingは移動せず完了した。fresh inventoryは残るproduction-
  helper familyへ戻り、同じownerのTask 263ZZUとして正確な5 fragment/77行four-
  edge local-mode two-hop asserted-head routeを選ぶ。token-identical body、byte
  stability、five-expansion set-terminal two-hop behavior、fail-closed behaviorを
  維持し、three-hop/他routeは移動せず完了した。fresh inventoryは残るproduction-
  helper familyへ戻り、同じownerのTask 263ZZVとして正確な5 fragment/75行three-
  edge local-mode three-hop asserted-head routeを選ぶ。token-identical body、byte
  stability、four-expansion set-terminal three-hop behavior、fail-closed behaviorを
  維持し、four-edge/他siblingは移動せず完了した。fresh inventoryは残る
  production-helper familyへ戻り、同じownerのTask 263ZZWとして正確な5 fragment/
  80行four-edge local-mode three-hop asserted-head routeを選ぶ。token-identical body、
  byte stability、five-expansion set-terminal three-hop behavior、fail-closed behaviorを
  維持し、four-hop/他siblingは移動せず完了した。fresh inventoryは残るproduction-
  helper familyへ戻り、同じownerのTask 263ZZXとして正確な5 fragment/79行four-
  edge local-mode four-hop asserted-head routeを選ぶ。token-identical body、byte
  stability、five-expansion set-terminal four-hop behavior、fail-closed behaviorを
  維持し、他routeは移動せず完了した。fresh inventoryは残るproduction-helper
  familyへ戻り、同じownerのTask 263ZZYとして正確な5 fragment/47行direct
  builtin-set reserved-variable type-assertion routeを選ぶ。token-identical body、
  byte stability、独立したreserve/formula-side source provenance、normalized-
  reflexive builtin-set behavior、fail-closed behaviorを維持し、builtin-object、
  local-mode、他siblingは移動せず完了した。fresh inventoryは残るproduction-
  helper familyへ戻り、既存private `type_elaboration/output.rs`の正確な10行
  shared term/formula diagnostic-key projectionをTask 263ZZZとして選ぶ。
  token-identical body、canonical diagnostic traversal、prefix、sort、dedup、
  byte stability、既存parent consumer 9個を維持し、wrapperを移動せず、key/
  diagnostic/payload/fail-closed behaviorを変えず完了した。fresh inventoryは
  残るproduction-helper familyへ戻り、既存private
  `type_elaboration/type_assertion_routes.rs`の正確な5 fragment/47行direct
  builtin-object reserved-variable type-assertion routeをTask 263ZZZAとして選ぶ。
  token-identical body、byte stability、独立したreserve/formula-side source
  provenance、normalized-reflexive builtin-object behavior、fail-closed behavior
  を維持し、builtin-set/local-mode/chained/他siblingを移動せず完了した。fresh
  inventoryは残るproduction-helper familyへ戻り、既存private
  `type_elaboration/output.rs`の正確な2 fragment/28行standalone contradiction
  formula output/detail familyをTask 263ZZZBとして選ぶ。token-identical body、
  byte stability、正確なchecked contradiction payload、空diagnostic/deferred/
  fact、normal detail consumer 1個、test-only output consumerを維持し、他formula
  family/routeを移動せず完了した。fresh inventoryは残るproduction-helper
  familyへ戻り、既存private `type_elaboration/output.rs`の正確な2 fragment/30行
  formula-statement output/detail familyをTask 263ZZZCとして選ぶ。token-
  identical body、byte stability、partial thesis payload、missing-formula deferred
  reason/diagnostic 1個、normal detail consumer、test-only output/extractor consumer
  を維持し、他formula family/routeは移動せず、全preservation gateをpassして
  完了した。Task 263 parent itemはopenを維持する。fresh inventoryは既存private
  `type_elaboration/output.rs`の正確な35行inline builtin-binary term/formula
  checker/detail producerをTask 263ZZZDとして選ぶ。token-identical body、byte
  stability、ordered numeral term 2個、source-selected equality/inequality/
  membership formula、ordered/deduplicated diagnostic、normal detail consumer、
  test-only extractor consumerを維持し、他formula family/routeは移動しない。
  全preservation gateをpassして完了した。Task 263 parent itemは次のfresh bounded
  selectionまでopenを維持する。fresh inventoryは既存private
  `type_elaboration/output.rs`の正確な2 fragment/43行builtin type-assertion formula
  output/detail familyをTask 263ZZZEとして選ぶ。token-identical body、byte
  stability、source-derived numeral/formula/asserted-type payload、type-entry ownership、
  normalized builtin-set type、diagnostic ordering、normal detail consumer、test-only
  output/extractor consumerを維持し、全preservation gateをpassして完了した。Task
  263 parent itemはopenを維持する。fresh inventoryは既存private
  `type_elaboration/type_assertion_routes.rs`の正確な5 fragment/52行direct local-
  mode reserved-variable type-assertion routeをTask 263ZZZFとして選ぶ。token-
  identical body、key/test alias、real expansion 1個、normalized-reflexive Task138
  output、normal detail、test-only config/output/extractor、全fail-closed/isolation
  boundaryを維持し、全preservation gateをpassして完了した。Task 263 parent
  itemはopenを維持する。修正したfresh inventoryは既存private
  `type_elaboration/output.rs`の正確な29行shared imported-attribute assertion
  checker-output coreをTask 263ZZZGとして選ぶ。token-identical body、shared
  Task113/114 numeral/attribute-assertion payload、context、deferred reason、
  diagnostic、保持するwrapper 2個、normal parent-only visibility、全fail-closed/
  isolation boundaryを維持し、全preservation gateをpassして完了した。Task 263
  parent itemはopenを維持する。fresh inventoryは既存private
  `type_elaboration/output.rs`の正確な8行positive imported-attribute assertion
  output wrapperをTask 263ZZZHとして選ぶ。token-identicalなpositive extractor
  selectionと移動済みshared coreへのpayload forwarding、normal parent-only
  visibility、保持するdetail/non-empty wrapper、正確なdiagnostic、全fail-closed/
  isolation boundaryを維持し、全preservation gateをpassして完了した。Task 263
  parent itemはopenを維持する。fresh inventoryは既存private
  `type_elaboration/output.rs`の正確な8行positive imported-attribute assertion
  detail wrapperをTask 263ZZZIとして選ぶ。token-identicalなoutput-to-canonical-
  key projection、normal detail visibility、test-only output/extractor crossing、
  保持するnon-empty family、正確なdiagnostic、全fail-closed/isolation boundaryを
  維持し、全preservation gateをpassして完了した。Task 263 parent itemはfresh
  inventoryまでopenを維持する。
  fresh inventoryは既存private `type_elaboration/output.rs`の正確な9行の
  attribute-level-negative imported-attribute assertion output wrapperをTask
  263ZZZJとして選ぶ。token-identicalなdirect-`non` extractor selectionとshared
  coreへのpayload forwarding、normal parent-only visibility、保持するdetail
  wrapper、正確なdiagnostic、全fail-closed/isolation boundaryを保存し、全
  preservation gateをpassして完了した。Task 263 parent itemはfresh inventory
  までopenを維持する。
  fresh inventoryは既存private `type_elaboration/output.rs`の正確な9行の
  attribute-level-negative imported-attribute assertion detail wrapperをTask
  263ZZZKとして選ぶ。token-identicalなoutput-to-canonical-key projection、normal
  detail visibility、test-only output/extractor crossing、正確なdiagnostic、全
  fail-closed/isolation boundaryを保存し、全preservation gateをpassして完了した。
  Task 263 parent itemはfresh inventoryまでopenを維持する。
  修正済みfresh inventoryは既存private `type_elaboration/output.rs`の正確な43行
  set-enumeration checker-output producerをTask 263ZZZLとして選ぶ。token-
  identicalなordered numeral item 4個、ordered set-enumeration term 2個、
  equality formula、context、
  payload/status/diagnostic、normal parent-only visibility、保持するdetail wrapper、
  全fail-closed/isolation boundaryを保存し、全preservation gateをpassして完了した。
  Task 263 parent itemはfresh inventoryまでopenを維持する。
  Fresh inventoryは既存private `type_elaboration/output.rs`の正確な8行set-
  enumeration formula detail wrapperをTask 263ZZZMとして選ぶ。token-identicalな
  output-to-canonical-key projection、normal detail visibility、test-only output/
  extractor crossing、正確なdiagnostic、全fail-closed/isolation
  boundaryを保存し、全preservation gateをpassして完了した。Task 263 parent itemは
  fresh inventoryまでopenを維持する。
  Fresh inventoryは既存private `type_elaboration/output.rs`の正確な49行imported
  predicate/functor checker-output producerをTask 263ZZZNとして選ぶ。token-
  identicalなordered input、imported functor referenceとsymbol provenance 2個、
  predicate formula、context、payload/status/diagnostic、normal producer visibility、
  test-only extractor crossing、保持するdetail/connective family、全fail-closed/
  isolation boundaryを保存し、全preservation gateをpassして完了した。Task 263
  parent itemはfresh inventoryまでopenを維持する。
  Fresh inventoryは既存private `type_elaboration/output.rs`の正確な8行imported
  predicate/functor formula detail wrapperをTask 263ZZZOとして選ぶ。token-
  identicalなoutput-to-canonical-key projection、normal detail visibility、test-
  only output/extractor crossing、正確なdiagnostic、全fail-closed/isolation boundary
  を保存し、全preservation gateをpassして完了した。Task 263 parent itemはfresh
  inventoryまでopenを維持する。
  Fresh inventoryは既存private `type_elaboration/output.rs`の正確な52行formula
  connective/quantifier checker-output producerをTask 263ZZZPとして選ぶ。token-
  identicalなordered formula shell 5個、context、deferred reason、payload/status/
  diagnostic、normal producer visibility、test-only extractor crossing、保持する
  detail、全fail-closed/isolation boundaryを保存し、全preservation gateをpassして
  完了した。Task 263 parent itemはfresh inventoryまでopenを維持する。
  Fresh inventoryはprivate `output.rs`の正確な8行formula connective/quantifier
  detail wrapperをTask 263ZZZQとして選ぶ。exact key projection、normal detail
  visibility、test-only output/extractor crossing、diagnostic、fail-closed/isolation
  behaviorを保存し、全preservation gateをpassして完了。Task 263はfresh inventory
  までopen。
  fresh inventoryは正確な5 fragment/62行chained local-mode reserved-variable
  type-assertion routeをTask 263ZZZRとして既存private
  `type_elaboration/type_assertion_routes.rs`に選ぶ。leaf-private key、config-derived
  test alias、normal detail route、test-only config/output/extractor crossing、正確な
  Task 146 normalization/provenance、全fail-closed/isolation boundaryを保存する。
  全preservation gate通過で完了し、Task 263はfresh inventory待ちでopenのままである。
  fresh inventoryは正確な5 fragment/67行two-edge local-mode reserved-variable builtin
  type-assertion routeをTask 263ZZZSとして既存private
  `type_elaboration/type_assertion_routes.rs`に
  選ぶ。leaf-private key、normal detail、test-only config/output/extractor、正確な
  Task 148 provenance/normalization、全fail-closed/isolation boundaryを保存する。
  全preservation gate通過で完了し、Task 263はfresh inventory待ちでopenのままである。
  fresh inventoryは正確な5 fragment/67行Task 186 two-edge local-mode same-mode
  asserted-head routeをTask 263ZZZTとして既存private
  `type_elaboration/type_assertion_routes.rs`に選ぶ。leaf-private key、config-derived
  test alias、normal detail route、test-only config/output/extractor crossing、正確な
  same-Outer relationとnormalization/provenance、全fail-closed/isolation boundaryを
  保存する。全preservation gate通過で完了し、Task 263はfresh inventory待ちで
  openのままである。
  fresh inventoryは正確な5 fragment/71行Task 205 three-edge local-mode
  immediate-radix asserted-head routeをTask 263ZZZUとして既存private
  `type_elaboration/type_assertion_routes.rs`に選ぶ。leaf-private key、config-derived
  test alias、normal detail route、test-only config/output/extractor crossing、正確な
  immediate-radix relationとnormalization/provenance、全fail-closed/isolation
  boundaryを保存する。全preservation gate通過で完了し、Task 263はfresh inventory
  待ちでopenのままである。
  fresh inventoryは正確な5 fragment/73行Task 150 three-edge local-mode
  reserved-variable builtin type-assertion routeをTask 263ZZZVとして既存private
  `type_elaboration/type_assertion_routes.rs`に選ぶ。73行同率ではconsumer surfaceが
  小さい方である。leaf-private key、config-derived test alias、normal detail route、
  test-only config/output/extractor crossing、正確なbuiltin relationとnormalization/
  provenance、全fail-closed/isolation boundaryを保存する。全preservation gate通過で
  完了し、Task 263はfresh inventory待ちでopenのままである。
  fresh inventoryは正確な5 fragment/73行Task 195 three-edge local-mode
  same-mode asserted-head routeをTask 263ZZZWとして既存private
  `type_elaboration/type_assertion_routes.rs`に選ぶ。leaf-private key、
  config-derived test alias、normal detail route、test-only config/output/
  extractor crossing、正確なsame-Outer relationとnormalization/provenance、全
  fail-closed/isolation boundaryを保存する。全preservation gate通過で完了し、
  Task 263はfresh inventory待ちでopenのままである。
  fresh inventoryは正確な5 fragment/76行Task 207 four-edge local-mode
  immediate-radix asserted-head routeをTask 263ZZZXとして既存private
  `type_elaboration/type_assertion_routes.rs`に選ぶ。leaf-private key、
  config-derived test alias、normal detail route、test-only config/output/
  extractor crossing、正確なimmediate-radix relationとnormalization/provenance、全
  fail-closed/isolation boundaryを保存する。全preservation gate通過で完了し、
  Task 263はfresh inventory待ちでopenのままである。
  修正済みfresh inventoryは正確な5 fragment/76行Task 152 four-edge local-mode
  reserved-variable builtin type-assertion routeをTask 263ZZZYとして既存private
  `type_elaboration/type_assertion_routes.rs`に選ぶ。leaf-private key、
  config-derived test alias、normal detail route、test-only config/output/
  extractor crossing、正確なbuiltin relationと5 expansionのnormalization/
  provenance、全fail-closed/isolation boundaryを保存する。全preservation gate
  通過で完了し、Task 263はfresh inventory待ちでopenのままである。
  fresh inventoryは正確な5 fragment/78行Task 197 four-edge local-mode
  same-mode asserted-head routeをsole remaining production-helper familyとして
  確認し、Task 263ZZZZとして既存private
  `type_elaboration/type_assertion_routes.rs`に選ぶ。leaf-private key、
  config-derived test alias、normal detail route、test-only config/output/
  extractor crossing、正確なsame-TooDeep relationと5 expansionのnormalization/
  provenance、全fail-closed/isolation boundaryを保存する。全preservation gate
  通過で完了した。fresh production-helper inventoryは`runner.rs`にtop-level
  dispatch/orchestrationだけを確認し、Task 263は完了、Task 264は独立closeout
  へ進む。
- [x] paired source-layout inventory、crate plan、todo、harness/source-path table、
  ownership guard を同期して series を closeout する。fresh inventory が Step 5
  を再開する前に、active runner 188、plan 403/367、type-elaboration 235/223、
  pass/fail 219/184、discovered unit test 272件、expectation/trace credit、既存
  `.miz` intent が不変であることを確認する。Task 264は完了し、paired final
  inventoryはproduction runner path 17個/18,952行、path/content manifest hash
  `b36d96fe...`/`62d30627...`、private type-elaboration leaf 11個、facade/top-
  level-orchestration-only `runner.rs`を記録する。全preservation count、4 CLI
  hash、raw/normalized test-list hashは不変で、`spec_coverage_audit.md`も変更しない。
  fresh canonical Step 5 inventoryには次のnonempty unchecked taskがなく、Steps
  6/7はdeferredのままである。
- [x] **Task 265: Step 5 execution authorityを明文化する。** fresh canonical
  inventoryを行い、残る全familyをconcrete owner taskまたはnonemptyなowner-owned
  decomposition taskへ割り当てる。top-level roadmap、paired owner plan/TODOと
  current-state audit、traceability deferred ownership、specification coverage
  auditを同期する。source、language semantics、`.miz` fixture、expectation、trace
  status/test list、runner count、coverage creditは変更しない。dependency graphは
  Task 266 -> Task 267 -> Task 268、Tasks 266 + 268 -> mizar-core Task 31、checker
  Task 247 -> core Task 32、Core Tasks 31 + 32 -> mizar-vc Task 30 -> VC 31とする。
  Parser Tasks 47-48とresolver Task 31は
  Task-266 dependencyではなく、独立にauthorizedされたchecker Task-49 prerequisite
  である。Checker Task 247、core Task 32、VC Task 30はpayloadを捏造せず残る
  checker、CoreIr/ControlFlowIr、VC/obligation family decompositionをexhaustiveに
  所有する。Steps 6/7はdeferredのままとする。Inventory
  分類: executable decompositionの欠落は`design_drift`、exact Task-180 final
  handoff、property implementation、same-return conflict、Core、VCのgapは
  `source_drift`と`test_gap`、Task-47 recoveryは`test_expectation_drift`と
  `source_drift`である。downstream crateが他crateのraw syntaxを再構築することは
  `boundary_violation`であり禁止する。Task 265のselected execution-authority sliceに
  新規またはblocking `spec_gap`はなかったが、既存MC-G005 public-code allocation
  `spec_gap`はexplicitに残った。`source_undocumented_behavior`、
  `repo_metadata_conflict`は検出していない。
  Checker Task 247はauthorized docs/traceability splitを完了し、Tasks
  248-264/269-279、Task-10 increments `MT10-FS`/`MT10-AS`、既存Task 49が
  remaining familyを所有する。same-return memberはresolver Task 31が
  `declaration_symbol`でsole activationし、Task 49が他23件をactivateしてexact
  24-fixture setをreconcile/deduplicateする。Task 274とexternal scheme/theorem-role Gate S1は
  explicit blocked gateのためTask 49はまだ実行不能。Core Task 32はdocs-decomposition-
  authorizedとなった。
- [x] **Task 266: exact Task-180 checked contradictionをfinal checker handoffへ
  保存する。** checker-owned syntax-free `ResolvedTypedAst` dataを拡張し、
  `SourceDerivedContradictionConstantBoundary`のresolver theorem owner 1件を、既存の
  checked `FormulaKind::Contradiction` result 1件へlinkする。owner/formula identity、
  source range、state、provenanceを保存する。`mizar-test`はreal AST extractionと
  exact active-runner assertionを所有し続け、checkerはfinal semantic identityと
  validationを所有する。missing/duplicate/reordered/recovered/mismatched rowをreject
  する。既存`.miz`/expectationは不変で再利用し、checker/runner unit、corruption、
  determinism testを追加して4 CLI outputをbyte-stableに保つ。falsehood/fact
  publication、theorem acceptance、proof status/skeleton/terminal goal、Core/CFG/VC、
  broader formula、runner stage promotionは禁止する。依存: Task 265とchecker Task
  180。仕様: 14、16。
- [x] **Task 267: omitted-justification theorem handoff contractを決定する。** paired
  checker/core design docで、written justificationのないordinary theoremに対する
  checker-owned pending-auto-proof status、proof skeleton、explicit terminal-goal
  payload、source/provenance link、malformed/missing behavior、core typeへのexact mapping
  を定義する。docs-only taskであり、omitted justificationをaccepted proofと同一視、
  core内でraw syntaxからterminal goalを推論、proof search実行、fixture/expectation/
  trace status編集をしてはならない。依存: Task 266。仕様: 15、16、architecture 06。
  完了: explicit `Unmodified`/`Omitted` intentをdistinct
  `PendingAutomaticProof` 1件、direct terminal goal 1件、future exact
  `False`/Active `TheoremProof` core seed (`proof/0`)へ写像し、corrupt inputは
  atomic fail、acceptance creditは付与しない。
- [x] **Task 268: accepted Task-267 checker producerを実装する。** exact Task-180
  final handoffだけにTask-267 proof status/skeleton/terminal-goal payloadを追加する。
  missing/duplicate/reordered/corrupt/owner-formula-proof mismatchのfail-closed checker/
  runner testを追加し、3 proof tableのdeterministic nonempty debug renderingと
  empty時のTask-266 debug outputのbyte-identical性をassertする。theorem
  acceptance、discharge、Core/VC generation、broader
  proof form、existing expectation change、Steps 6/7はscope外。依存: Task 267。
  完了: exact extractorはannotation、written justification、proof blockがない
  Task-180 theoremだけにexplicit intentをemitし、checker/runner corruptionと
  immutable output assertionがpassする。existing fixture/expectation/runner
  admission/count/CLI bytesは不変で、次はCore Task 31である。

## VC Task 30 / Task-10 consumer ownership

VC Task 30 は `MT10-VC-T180` を VC Task 31 だけに予約する。distinct な
Task-180-shaped theorem source/sidecar を `proof_verification` /
`active_proof_verification`、`expected_phase = "vc_generation"`、phase 11 で実行し、
complete deterministic `SnapshotKind::VcIr` / `VcSet::debug_text()` bytes を比較する。
既存 type-elaboration Task-180 source/sidecar/Core snapshot は変更しない。最初の
proof-verification runner/tag/guard 変更はこの最初の real baseline と同時に実装し、empty
infrastructure にしない。

VC Tasks 32-55 は `MT10-VC-PV` を共有し、各 task は distinct
`MT10-VC-PV/VC<n>` source/sidecar/trace/baseline slice を所有する。VC 40 は complete 済み
VC 37/39 output と Core 40/A1、VC 53 は canonical authority が evidence producer/
reference schema/authentication contract/test を命名していないことにより未実行のまま。
missing scheme/theorem-role slice は
direct VC 41 の外で S1 の背後に残る。Task 30 は runner/case/expectation/trace status/test/
count/hash/coverage を変更しない。

## VC Task 31 / Task-10 consumer completion

exact `MT10-VC-T180` increment は complete である。distinct
`pass_proof_verification_contradiction_formula_constant_001` sidecar だけが active
proof-verification case で、phase `vc_generation` と complete VcIr snapshot を持つ。
public runner/report/CLI は exact source-to-checker-to-Core-to-VC path を2回実行し、
admission、generation、baseline error を fail closed にし、passing result 1件を報告する。
plan count は404/369、proof-verification coverage は4/1、pass/fail は220/184で、
parse/declaration/type active count は96/4/188のままである。

existing type-elaboration Task-180 source/sidecar/Core snapshot は不変である。broad
proof-verification、VC 32-55、discharge、ATP/kernel/proof acceptance、fact、Steps 6/7
は deferred または dependency-paced のままである。

## Resolver Task 31 / declaration-symbol completion

exact same-return incrementはcompleteである。変更しない
`fail_resolve_same_signature_same_return_conflict_001.miz` sourceとactive化したsidecarは、
既存real frontend/resolver runnerで実行され、
`declaration_symbol.signature.same_signature_definition_conflict`を観測する。
declaration-symbol admissionは5件で、plan 404/369、parse 96、type 188、proof 1、pass/fail
220/184は不変である。different-return sidecarとその既存detail keyはbyte-identicalのまま。
他Task-49 member、semantic overload behavior、public code、Steps 6/7 statusは変更しない。

各 source-moving task で review-only により visibility drift、test-discovery
drift、owner-boundary drift、source/docs inconsistency、意図しない behavior
change を確認する。focused tests、`cargo test -p mizar-test`、
`cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、
workspace `cargo test`、`git diff --check` を実行し、全 command が成功するまで
failure を修正して再実行する。test/verification failure 自体を series の停止
理由にしない。

## Parser Task 47 / parse-only completion

spec-derived pass fixture 1件はomitted-justification/proof-block `reconsider` rowだけを
activateする。existing mixed recovery `.miz`は変更せず、sidecarからobsolete omitted-tail
parser diagnosticだけを削除した。active planは405/369、parse-onlyは97、pass/failは
221/184である。declaration/type/proof admissionは5/188/1のまま。semantic reconsider
acceptanceとE0102 productionはowning checker taskへdeferredのままで、Parser Task 48と
Steps 6/7はpromoteしない。

## Parser Task 48 / property-implementation parse-only completion

authorized nonempty Task-48 sliceはcompleteである。新しいpass/fail corpus pairは
dedicated top-level property-implementation grammarをreal parse-only runnerで実行し、exact
`spec.en.07.modes.property_implementation.parser` rowは`pass_and_fail`付き
`covered`となる。active totalはplan 407/369、parse-only 99/99、pass/fail 222/185、
warnings/errors 23/0で、declaration/type/proof admissionは5/188/1のままである。

このcompletionが与えるのはparser/syntax-only creditだけである。property payload
extraction、semantic overlap/coherence handling、proof acceptance/discharge、inactive
semantic Task 39は変更しない。このincrementはchecker taskもSteps 6/7 authorityもpromoteしない。

## Checker Task 248 / Task-10 consumer completion

- [x] exact active reserve-plus-definition-parameter shadowing fixture、single-reference
  sidecar、bounded covered trace rowを追加する。
- [x] real resolver shell/source walkをmatchし、syntax-free item/declaration/context
  payloadだけを`mizar-checker`へ渡し、complete handoffを`TypedAst`から
  `ResolvedTypedAst`まで保持する。
- [x] same-spelling distinct binding identity、structural shadowing、deterministic debug
  output、後続semantic payload 0件をassertする。term-use selectionとTasks
  249+/269+はこのincrement外に保つ。
- [x] exact executable coverage変更のためchapter-level coverageを更新する。broad
  payload-extraction rowとSteps 6/7は変更しない。

## Checker Task 249 frozen consumer prerequisite

- [x] future exact ten-reserve-root broad fail consumerと10/13/6 handoff oracleを
  freezeする。fixture/sidecar/trace rowはまだ追加しない。
- [x] existing Task-248 pass routeをdependency regressionとしてfreezeする。
  source/sidecar/traceはunchanged、existing binding 2件へlinkした
  `Bare`/builtin-`set` row exact 2件、argument 0件とする。
- [x] sole pending keyをrunner-ownedに保ち、checker semantic result tableを全て
  emptyにする。Tasks 68-71はbyte-for-byteで維持する。
- [x] Checker Task 249をlogical task 1件としてimplementした。exact broad
  10/13/6 routeとunchanged Task-248 2/2/0 co-consumerはimmutable checker
  handoffを通ってexecuteする。plan 411/372、type 238/226、pass/fail
  224/187、active type 190、fresh hashをmandatory completion oracleとする。
  Tasks 250+、269+、Steps 6/7はpromoteしない。

## Checker Task 250 frozen consumer prerequisite

- [x] existing Task-81/67/84/85 active fail fixtureだけをexact real consumerに
  freezeし、routeごとにTask-249 application/root 1件、type argument 0件とした。
- [x] aggregate Task-250 oracleをnonempty chain 4件、attribute 4件、qualifier
  1件、parenthesized argument group 1件、actual 1件にfreezeし、exact polarityと
  local/imported provenanceを保持する。
- [x] Task-81/67 runner-only pending progressionとTask-84/85 existing
  evidence-query preservationをfreezeし、new `.miz`、broad expectation
  rebaseline、semantic result、public diagnosticを追加しない。
- [x] multi-attribute orderとsingle/parenthesized prefix projectionについてprivate
  synthetic-`SurfaceAst` extractor coverageを要求し、checker corruption/
  determinism matrixも要求する。
- [x] Checker Task 250をlogical task 1件でimplementした。private
  `source_attribute` leafはexact real consumer 4件とsynthetic prefix probeを
  public checker handoffまで実行し、plan 411/373・type 239/227をunchanged
  pass/fail/admissionで達成する。Tasks 251+/269+とSteps 6/7はpromoteしない。
