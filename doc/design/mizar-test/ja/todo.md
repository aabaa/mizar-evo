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

## Checker Task 251 frozen consumer prerequisite

- [x] Task-249 broad fixture + Task-84/85だけをrepresentative real selectorとして
  freezeし、`.miz`を追加せず全siblingをbyte-identicalに保つ。
- [x] missing request 10件（mode-expansion 5 / structure-inhabitation 3 /
  attributed 2）、combined
  Task-249 12/15/6、Task-250 2/2/0/0/0、dependency reference 0件をfreezeする。
- [x] broadだけをmissing-dependency detailへ進め、Task-84/85 evidence-query
  detailと全outcome/public codeを維持する。
- [x] real ASTとproduction Task-10 consumerからfinal
  `TypedAst`/`ResolvedTypedAst`までrequested/missing/rejected/supplied injectionを
  requireする。supplied inputはaccepted evidenceではなく、corruptionはatomic
  failure。
- [x] Checker Task 251をseparate logical taskとしてimplementする。plan
  411/374、type 240/228、unchanged pass/fail/admission/warning、exact isolation、
  full hash/review、dedicated commit 1件を達成する。

## Checker Task 252 frozen consumer prerequisite

- [x] existing builtin numeral equality、bare reserved-variable equality、
  single-left-parenthesized reserved-variable equalityだけをfreezeし、new
  `.miz`/outcome/detail changeを禁止する。
- [x] aggregate term/reference/numeric-request 7/4/2とsource-only parent edgeを
  freezeし、parenthesisがsemantic type/term/fact/axiom/FOL rowを追加しない。
- [x] `LocalAbbreviation` constant、`it` current-result role、nested
  parenthesisのsame-producer synthetic coverageをfreezeし、Task-269 local
  binding/Tasks-260/264 definition ownershipを取らない。
- [x] transactional final ownership、full corruption/determinism/isolation、
  bounded 3-sidecar trace reference、no-new-case implementation oracle
  plan 411/375/type 241/229を要求する。
- [x] post-freeze ordinal contractを、先行して完了したbinding rowを数えるruleへ
  correctし、reachable `Ambiguous` rejection用duplicate-priority groupを保持し、
  `Resolver`をstructurally unreachableと記録する。
- [x] Checker Task 252をseparate logical taskとして実装する。pass/fail
  224/187、admission 101/5/190/1、warnings/errors 23/0を維持し、library test
  291件とverified 23-path/24,120-line layoutへ進みfresh hashを記録する。
  Tasks 253+/260/264/269とSteps 6/7はpromoteしない。

## Checker Task 253 frozen consumer prerequisite

- [x] Checker Task 253を、既に完了した`mizar-test` runner-refactor Tasks
  253A/253Bと区別する。
- [x] 既存imported `1 ++ 2` routeとexact new spec-derived module-local
  second-definiens `task253_local_source(x)` fail routeだけをreal consumerと
  してfreezeする。
- [x] new sourceをreserve `x`と、inner parameter `x`を共有するfunctor
  declaration 2件のdefinitionへfreezeする。Task-248 two-item/two-binding
  shadow handoffをreuseし、actualは`BindingId(1)` / `BindingContextId(1)` /
  `use_ordinal == 2`をreferenceする。
- [x] aggregate Task-253 application/wrapper/candidate/argument/request table
  2/1/2/3/4、参照するTask-252 primary/reference/numeric slice 3/1/2を、
  primary重複所有なしでfreezeする。
- [x] `(1 ++ 2)`のTask-253 transparent-wrapper originと個別認証candidate
  referenceを、completeness/viability/ranking/winner claimなしでfreezeする。
- [x] synthetic ordinary/nested/parenthesized/candidate coverage、inline
  zero/one/two-actual source-schema coverageだけ、template whole-subtree
  exclusionをfreezeする。
- [x] inline identity/formal/capture/substitutionをTask 270、template direct
  role/actual/guard/requestをTask 277、ordinary/template candidate
  collection/selectionをTask 278へ割り当てる。
- [x] imported outcome/detailを維持し、new local sidecarをpublic diagnostic
  なしの`definition_declaration_payload_extraction_gap` /
  `type_elaboration.external_dependency.ast_payload_extraction`へfreezeする。
- [x] Checker Task 253を別の1 logical taskで実装し、exact new fixture/sidecarと
  bounded diagnostic trace rowを追加し、imported outcome/detailを維持し、fresh
  計測で412/376、242/230、224/188、101/5/191/1 oracleとlibrary test
  303件へ到達した。paired completion documentは24-path/25,607-line manifestと
  exact 5 CLI/test-list/production hashを記録する。Tasks 254+とSteps 6/7は
  unpromotedのままである。

## Checker Task 254 frozen consumer prerequisite

- [x] construction、selector access、functional updateを使う3 definiensと
  `Task254Pair` declarationを持つexact new spec-derived local structure-term
  fail source 1件をfreezeする。
- [x] real Task-254 term/wrapper/root/member/field-update/edge/request oracle
  5/0/3/9/2/10/26とcomposed Task-252
  primary/reference/numeric-request slice 8/0/8を、real Task-253 rowなしで
  freezeする。
- [x] raw constructor/selector/update、member、`FieldUpdate`、wrapper、edge
  extractionをprivate runner leaf 1個に限定し、checker handoffをsyntax-freeに
  保つ。
- [x] repeated label/pathをsource orderで保存し、nested pathをmember chainで
  表現し、`FieldUpdate`へ独立term/type/factを割り当てない。
- [x] one-way same-context Task-252 root、別Task-253 argument edgeからtargetに
  されないTask-253 root application、Task-254 child compositionをfreezeし、
  nested Task-253 targetをrejectする。reverse Task-253 application、Task-255
  term、template、initial type-argument-bearing constructorはwhole-subtree
  excludedのままとする。
- [x] sidecarをpublic diagnosticなしの
  `definition_declaration_payload_extraction_gap` /
  `type_elaboration.external_dependency.ast_payload_extraction`へfreezeし、
  structure definition/member/viewと全semantic decisionをTask 263に残す。
- [x] 本prerequisiteをdocumentation-onlyとする。fixture、sidecar、trace
  row/status/count、runner route、test list、production source、executable
  creditを変更せず、412/376、242/230、224/188、101/5/191/1、303-test、
  24-path/25,607-line baselineを維持する。
- [x] Checker Task 254を別logical taskで実装した。exact fixture/sidecar、
  bounded requirement、Chapter-5/13 transport-only widening、reciprocal
  backlink 4件、Task-248 context reuse、complete real/synthetic/exclusion/
  corruption/final-ownership matrix、measured 413/377、243/231、224/189、
  101/5/192/1 oracleはcompleteである。

## Checker Task 255 frozen consumer prerequisite

- [x] enumeration、conditionなしcomprehension、choice、`qua` definiensを持つ
  future local-definition fail source 1件をfreezeする。
- [x] private raw-syntax ownerとpublic 6-table syntax-free boundaryを
  4/0/1/3/4/7 + Task-252 4/0/4、real Task-253/254 target/fingerprintなしで
  freezeする。
- [x] written generator declarationを保持するが`BindingId`/captureを捏造せず、
  binding/captureはTask 257、condition formulaはTasks 256-257に割り当てる。
- [x] bare builtin `set`/`object` target siteだけをadmitし、Task-249
  declaration-application ownershipを不変にする。
- [x] 本prerequisiteをdocumentation-onlyとし、413/377、243/231、224/189、
  101/5/192/1、312 tests、25 paths / 27,317 lines、全hashを不変にする。
  separate implementationはfresh preflight条件で414/378、244/232、224/190、
  active type 193をprojectする。
- [x] separate Task-255 consumerをexact fixture/sidecar、reciprocal trace
  reference 5件、Task-248/252 composition、final 6-table 4/0/1/3/4/7 +
  4/0/4 oracle、active-case isolation、review済みsynthetic/exclusion/
  corruption/install-order coverageとともに実装する。external dependency gapを
  維持し、binder/formula/semantic ownershipはfrozenどおりdeferredに残す。

## Checker Task 256 frozen consumer prerequisite

- [x] 既存active fail consumer 8件をfreezeし、新規/既存`.miz`とcurrent
  outcome/detailを変更しない。
- [x] private raw-syntax ownerとpublic 8-table boundaryをTask-256
  `8/0/1/1/1/2/13/11`、Task-252 `16/0/16`、Task-253
  `1/1/1/2/2`、Task-255 `2/0/0/0/4/2`、real Task-254 targetなしで
  freezeする。
- [x] imported predicate/attribute provenance、formula-owned bare asserted
  type、source-anchored attribute polarity、nearest-family term ownership、
  conditional fingerprint、unresolved request 11件をfreezeする。
- [x] Task-256-owned combined composition順をcomplete Task-252 union、
  same handoff/arena上のTask-253/255 dependencyの順でfreezeし、既存
  lower-family exact selector/allowlistを変更しない。
- [x] edge 13件/request 11件のexact ordered positive vectorにTask-253 outer
  wrapper rangeとattribute target/`non` anchorを含め、standalone selector
  isolation oracle不変も要求する。
- [x] predicate chain/negation、inline/template、general asserted-type graph、
  qualified/argument-bearing attribute、semantic fact/truth、conditioned
  comprehensionをbounded increment外に明記する。
- [x] 本prerequisiteをdocumentation-onlyとし、414/378、244/232、
  224/190、101/5/193/1、320 tests、26 paths / 29,138 lines、全hashを
  保持する。separate implementationはfresh preflight条件でcase count不変の
  414/379、245/233をprojectする。
- [x] separate Task-256 consumer/producer/final handoff、bounded reciprocal
  trace increment、review済みreal/synthetic/exclusion/corruption/install-order
  matrixを実装した。exact 8 existing sourceは既存semantic detail ownerを維持した
  ままsyntax-free checker transactionをexerciseする。

## Checker Task 257A frozen consumer prerequisite

本節のChecker Tasks 257A-Cはchecker producer sliceで、上記の完了済み
mizar-test Tasks 257A-H test-layout系列とは別である。

- [x] unchanged connective/quantifier fail source 1件、exact formula site
  5件、binder segment/identifier/type site、source rangeをfreezeする。
- [x] private ownershipをfreezeする。既存`source_formula.rs` raw extraction
  shapeをextendし、lower-family selector/allowlistを拡張せず専用private
  `source_composite_formula` assemblerを使う。
- [x] public 7-table `5/0/1/1/1/4/6` transactionとexact `2/1/4` binding
  environmentをsingle context transition、resolver-shaped local binder
  identityを含めfreezeする。
- [x] ordered formula/root/binder/type-site/edge/request oracle、不変two-key
  semantic detail vector、all-active isolation、environment/table
  corruption/install/final-ownership coverageをfull literal handoff debug
  snapshot、exact legacy debug bytes、実行可能なpreinstalled-source-context
  rejectionを含めfreezeする。
- [x] broader connective/quantifier、bound use/capture、predicate chain、
  conditioned comprehension、theorem ownership、全semantic answerをChecker Task 257A
  外に保つ。
- [x] 本prerequisiteをdocumentation-onlyとし、414/379、245/233、
  224/190、101/5/193/1、checker/mizar-test 287/328 tests、27 paths /
  30,154 lines、全hashを保持する。separate implementationはcase count不変で
  414/380、246/234をprojectする。
- [x] separate Checker Task 257A selector extension、private assembler、public
  producer/binding prepass/final handoff、bounded reciprocal trace increment、
  review済みreal/synthetic/exclusion/corruption/install matrixを実装した。
  routeは不変two-key semantic detail vectorとcorrected parser range
  `52..113`、`78..89`、`78..79`、`86..89`を保持する。次のdependency-ready
  sliceはChecker Task 257Bである。

## Checker Task 257B1 frozen consumer prerequisite

- [x] exact 79-byte spec-derived pass sourceとuniversal/binder/type/equality/
  two-use rangeをfreezeする。
- [x] same-arena compositionをTask-252 `2/2/0`、Task-256
  `1/0/0/0/0/0/2/2`、Task-257 `1/0/1/1/1/0/2`、Task-257B1 `1/2`で
  freezeする。
- [x] Task-252 reference ownershipを保持し、両referenceがcontext 1でquantifier
  binding 0をselectすることを要求する。captured-free-variable metadataを
  誤用しない。
- [x] Task-257A source-context exclusionを維持する。combined installerは
  `source_context()`をabsentに保ち、preinstalled Task-248 handoffをatomic
  rejectする。
- [x] ownership-partition testをfreezeする。legacy installerはB1、combined
  installerはTask 257Aを持つASTをrejectし、両方partial publicationなしで
  byte-identical rollbackする。
- [x] A-cardinality/B-row hybrid、inverse hybrid、otherwise valid third
  profileのprofile-discriminator testをfreezeする。
- [x] semantic truth、theorem acceptance、broader connective、
  existential/restricted/nested/implicit binder、predicate chain、
  conditioned comprehensionをTask 257B1外に保つ。
- [x] prerequisiteをdocumentation-onlyで414/380、246/234、224/190、
  101/5/193/1、299/333 tests、28 paths / 30,654 linesに保つ。
- [x] exact consumer、第2 composite profile、lower-family composition、
  public `1/2` handoff、trace row、test、final ownershipを実装する。bounded pass
  routeは全semantic deferralを保持し、次はChecker Task 257B2。

## Checker Task 257B2 frozen runner checklist

- [x] exact 166-byte source/SHA-256、parser range、repeated flag/connective
  token、grouping wrapper、private selectorをfreeze。
- [x] same-arena Task-252 `16/0/16`、Task-256
  `8/0/0/0/0/0/16/16`、Task-257B2 `8/6/1/1/1/7/9`、
  composition `8/0`をfreeze。
- [x] selector isolation、mutation/recovery、A/B1 preservation、final ownership、
  trace/count impact、全semantic deferralをfreeze。
- [x] prerequisiteをdocumentation-onlyとし415/381、247/235、225/190、
  active 101/5/194/1、338 tests、29 paths / 31,374 linesを不変にする。
- [x] documentation commit/fresh parser/resolver preflight後、exact
  route/sidecar/covered trace/testを実装する。
- [x] corpus `416/382`、type `248/236`、pass/fail `226/190`、active
  `101/5/195/1`、343 library tests、semantic output absenceをverifyする。
- [x] own EN/JA frozen contract準備中はTask 257B3をunselectedに保つ。contractは
  completeで、implementationはseparateのまま。

## Checker Task 257B3 frozen runner checklist

- [x] exact 138-byte source/hash、Task-48 reserve extraction、mandatory
  preflight factとしてのparser/resolver node/scope/range、exact private
  selector boundaryをfreeze。
- [x] same-arena Task-252 `6/6/0`、Task-256
  `3/0/0/0/0/0/6/6`、Task-257B3 `3/0/1/3/3/2/6`、
  composition `3/6`とsource-order lookup/owning-edge associationをfreeze。
- [x] Task-248 source contextをabsentに保ち、reserve-derived base validation、
  isolation/mutation/previous-route/final-ownership/semantic-output testを
  freeze。
- [x] prerequisiteをdocumentation-onlyで`416/382`、`248/236`、
  `226/190`、active `101/5/195/1`、343 tests、29 paths /
  32,064 linesに保つ。
- [x] documentation commit/fresh preflight後にexact route、sidecar、covered
  trace row、testsを実装する。
- [x] corpus `417/383`、type `249/237`、active type `196`、library 349
  tests、exact selector isolation、semantic output不在をverifyする。

## Checker Task 257C1 frozen runner checklist

- [x] exact 107-byte source/hash、segment/head range 2件、negative token
  range、同じimported provenance、loaded-source/final-LF guardをfreeze。
- [x] same-arena Task-252 `3/0/3`とextended Task-256
  `1/0/2/2/2/0/0/3/2`（shared middle boundary edge 1件）をfreeze。
- [x] exact source near miss、recovery/mixed-chain exclusion、corruption/
  isolation/install/final matrix、empty semantic outputをfreeze。
- [x] prerequisiteを`417/383`、`249/237`、`227/190`、active
  `101/5/196/1`、349 tests、29 paths / 32,809 linesに保つ。
- [x] documentation commit/fresh preflight後にexact route、fixture/sidecar
  1件、covered trace row、testsを実装する。
- [x] `418/384`、`250/238`、`228/190`、active `101/5/197/1`、353
  tests、exact selector isolation、fail-closed corruption、empty semantic
  outputをverifyする。

## Checker Task 255C1 frozen runner checklist

- [x] valid 191-byte source/hash、parser range、imported `++` provenance、
  loaded-source/final-LF selector、future fail detailをfreeze。
- [x] same-arena Task-252 `4/0/4`、Task-253 `1/0/1/2/2`、
  Task-255 `1/0/1/1/1/1/2`をdirect condition-wrapper anchorと
  untargeted condition operand込みでfreeze。
- [x] reusable Task-253 ownership、near-miss/corruption/isolation、atomic
  install/clone、empty semantic output、prior route不変をfreeze。
- [x] prerequisiteをdocumentation-only `418/384`、`250/238`、
  `228/190`、active `101/5/197/1`、353 tests、29 paths / 33,184 linesに保つ。
- [x] documentation commit/fresh preflight後にexact route、fail sidecar、
  covered trace row、testsをimplement。runnerは`419/385`、`251/239`、
  `228/191`、active `101/5/198/1`、library tests 357件を実測し、empty
  semantic outputと全prior routeを保持する。

## Checker Task 257C2 frozen runner checklist

- [x] unchanged 191-byte fixture/hash/parser range/imported mapper
  provenance/exact Task-252/253/255 profileのreuseをfreeze。
- [x] reusable Task-256 equality builder、exact
  `1/0/0/0/0/0/0/2/2` profile、direct wrapper/equality ownership split、
  same-arena associationをfreeze。
- [x] dedicated one-edge Task-257C2 handoff、route order、
  mutation/near-miss/isolation/final-clone test、bidirectional A/B/C2
  installer exclusion、semantic exclusionをfreeze。
- [x] existing sidecar reuse、future trace row 1件、unchanged 419-case/
  pass-fail/active count、projected plan `419/386`/type `252/240`、
  unchanged diagnostic intent、本prerequisiteのexecutable artifact変更ゼロを
  freeze。
- [x] 本runner slice edit前にseparate Task-256C1 frozen-contract/
  checker-only implementationを完了。runner editなしで両lower install
  orderがpassする。
- [x] Task-256C1とfresh preflight後、separate Task-257C2 implementation
  commitでこのfrozen runner sliceだけをimplement。

## Checker Task 256C1 frozen runner checklist

- [x] runner non-ownershipをfreeze。source/test、fixture、sidecar、
  expectation、trace、production manifest、CLI変更ゼロ。
- [x] 191-byte fixtureはauthority/future Task-257C2 consumerとしてだけ保持し、
  checker-local testsが両install order/corruption coverageをown。
- [x] `419/385`、`251/239`、`228/191`、active `101/5/198/1`、357 tests、
  29 paths / 33,725 lines、全runner hashを保持。
- [x] Task 256C1 implementation中runnerを不変に保ちchecker-only lower gateを
  verify。C1 exit時点ではfrozen Task-257C2 routeのfresh preflightが次logical
  taskだった。

## Checker Task 257C2 implementation checklist

- [x] exact five-profile same-arena routeをlower diagnostic-only routeより
  先にpublishし、existing extraction-gap detailを保持。
- [x] exact profile/provenance/ownership、dependency/arena mutation、
  loaded-source/named near miss、active isolation、sidecar stability、
  replay/final cloneをrunner tests 4件で固定。
- [x] fixtureを追加せず、existing sidecar reference/noteとcovered trace row
  1件だけを更新。
- [x] `419/386`、`252/240`、`228/191`、active `101/5/198/1`、
  361 tests、production 29 paths / 34,064 linesを測定。

## Checker Task 257C3 frozen runner checklist

- [x] unchanged 107-byte Task-257C1 pass fixture/hash、exact range、
  final-LF guard、imported `divides` provenanceをreuse。
- [x] one arenaでTask-252 `3/0/3`、Task-256
  `1/0/2/2/2/0/0/3/2`、Task-257C3 `1/1`をfreeze。
- [x] Task-257C1よりcomplete routeを優先し、named near miss、active
  isolation、corruption/arena rollback、replay、final clone、empty semantic
  outputをfreeze。
- [x] existing sidecarへfuture reference/note 1件とfuture covered trace row
  1件だけを許可し、fixture/semantic expectationを追加しない。
- [x] prerequisite baseline `419/386`、`252/240`、`228/191`、active
  `101/5/198/1`、361 tests、29 paths / 34,064 linesを保持。
- [x] documentation commitとfresh parser/resolver/lower-stage/count/hash
  preflight後だけfrozen routeをimplement。

## Checker Task 257C3 implementation checklist

- [x] existing fixture/semantic detailを変更せず、lower C1 routeより先に
  exact complete routeをpublish。
- [x] route/provenance/debug、near-miss isolation、全dependency/arena
  mutation、rollback/replay、cloneをexactly runner tests 4件でcover。
- [x] existing sidecar reference/noteとcovered trace row 1件だけを更新。
- [x] 365 tests、production 29 paths / 34,290 linesを測定。

## Checker Task 258A frozen consumer checklist

- [x] exact 81-byte final-LF source/hashとreal frontend/resolver library-test
  pathをfreezeし、future corpus fixtureは追加しない。
- [x] Task-48 binding、Task-252 `2/2/0`、Task-256
  `1/0/0/0/0/0/2/2`、Task-258A `1/1/1/1/1` compositionをfreeze。
- [x] exact owner/label provenance、selector/subtree exclusion、
  owned BindingEnv/fingerprint、absent Task-248 owner、typed/resolved
  equality、empty semantics、active-route isolationをfreeze。
- [x] fixture/sidecar/trace metadata/status/count/CLI count/hash、365-test list、
  29-path / 34,290-line production manifestをpreserve。
- [x] checker documentation commit/fresh preflight後、dormant production
  routeとexactly library tests 4件だけを追加し、corpus activationは
  `MT10-FS`に残す。
- [x] runner 369 tests、production 30 paths / 34,955 linesを測定し、
  plan/type/active countとtrace metadataを不変に保つ。

## Checker Task 258B1 frozen consumer checklist

- [x] old Task-258B runner umbrellaを分解し、139-byte nested equality/
  conclusion/local-citation sourceだけをselectしてfinal LF/hash/parser
  range/resolver theorem-local-label provenanceをfreeze。
- [x] accepted fact/proof semanticsなしで、exact shared Task-48 `3/1/0`、
  Task-252 `8/8/0`、Task-256 `4/0/0/0/0/0/0/8/8`、Task-258B1
  `1/4/4/4/4`、reference `1/1` transactionをfreeze。
- [x] private raw-syntax ownership、corpus-dormant precedence、replayable
  resolver projection/reference/result、sole resolved/keyed node 68を持つ
  two-pass 77-node/root-76 resolver AST、selector/subtree/provenance
  mutation、active-route isolation、final clone/replay、exactly future
  library tests 5件をfreeze。
- [x] 本prerequisiteでは全fixture/sidecar/expectation/trace
  row/status/count/active route/executable count/Task-258A hashをpreserve。
- [x] checker documentation commitとfresh parser/resolver/lower preflight後、
  dormant Task-258B1 route/tests 5件だけをimplementし、Task 258B2+と
  Tasks 269–272をdefer。
- [x] runner 374 testsと、topology不変のproduction 30 paths /
  35,854 linesを測定し、corpus/trace/CLI count/hashをすべてpreserve。

## Checker Task 258B2 frozen consumer checklist

- [x] Task 258B2+をdecomposeし、final-LF 113-byte single-assumption theorem、
  hash、exact parser range、theorem-only resolver provenanceだけをfreeze。
- [x] reference association/semantic outputなしのTask-48 `2/1/0`、
  Task-252 `6/6/0`、Task-256 `3/0/0/0/0/0/0/6/6`、Task-258B2
  `1/3/3/3/3` transactionをfreeze。
- [x] private raw-syntax ownership、corpus-dormant precedence、
  selector/subtree/provenance mutation、Task-258A/B1/active-route isolation、
  typed/final clone、future runner tests exactly 5本をfreeze。
- [x] 本documentation prerequisiteでは全fixture、sidecar、expectation、
  trace row/status/count、active route、source、374-test list、30-path /
  35,854-line manifest、既存count/hash baselineをpreserve。
- [x] checker documentation commitとfresh parser/resolver/lower/API/count/
  hash preflight後、dormant Task-258B2 route/tests 5本だけをimplementした。
  B3–B5とTasks 269–272はdeferを維持。

## Checker Task 258B3 frozen consumer checklist

- [x] exact 104-byte final-LF source/hash、49-node/root-48 tree、全range、
  theorem-only resolver provenance、label-reference bundleなしをfreeze。
- [x] one arenaでTask-48 `2/1/0`、Task-252 `5/5/0`、Task-256
  `2/0/0/0/0/0/0/4/4`、base `1/2/2/2/2`、witness row 1件、
  `[0,1,2]` orderをfreeze。
- [x] private selector precedence、all-index parity、complete
  mutation/replay、named/multiple/missing/extra/subtree near miss、全
  A/B1/B2/active isolation order、clone/debug、rollback、empty semanticsを
  freeze。
- [x] 本prerequisiteでは全corpus artifact、trace row/status/count、active
  route、source、379-test list、30-path / 36,479-line manifest、hashを
  preserve。
- [x] documentation commit/fresh preflight後、exactly runner tests 5本で
  dormant B3 routeだけをimplementした。runnerは384 tests、production/
  test hashはimplementation resultで再測定済み。
- [x] Task 258B3N exact named-primary dormant runner consumer、51-node
  identity、witness/name `1/1`、five-test matrix、no semantics、unchanged
  runner baselineをfreeze。
- [x] documentation commitとfresh preflight後、frozen B3N consumerだけを
  implement。compound tests 5本がpassし、runnerは389 tests、
  production 30 paths / 37,555 lines。active-corpus changeはない。
- [x] broad Task 258B3Mをexact B3M1 reserved-variable mixed
  multiple-witness transportとB3M2 other witness-term shapeへ分解。
- [x] 113-byte/56-node B3M1 dormant consumer、lower/base profile、
  witness/name `2/1`、shared/dense ordinals、exact tests 5本、
  no semantics、unchanged runner baselineだけをfreeze。
- [x] docs commitとfresh preflight後にB3M1だけをimplementし、runner
  394 testsをproject。
- [x] B3M2をexact unnamed-numeral B3M2Aとremaining other-term B3M2Bへ
  decompose。
- [x] 107-byte/49-node B3M2A dormant consumer、Task-252 `5/4/1` +
  numeric request 0、witness/name `1/0`、exact tests 5本、
  no public/active/semantic route、unchanged runner baselineだけをfreeze。
- [x] docs commitとfresh preflight後にB3M2Aだけをimplementし、runner
  399 testsをproject。
- [x] B3M2Bをexact parenthesized B3M2B1 / remaining authority-valid
  B3M2B2へdecompose。
- [x] B3M2B1だけをfreezeし、implementation/B3M2B2をseparateに維持。
- [x] docs commit/fresh parser/resolver/lower/count/hash preflight後に
  frozen B3M2B1だけをimplement。
- [x] B3M2B2をexact nested-parenthesized B3M2B2Aとremaining
  authority-valid B3M2B2Bへdecompose。
- [x] B3M2B2Aだけをfreezeし、implementation/B3M2B2Bをseparateに維持。
- [ ] docs commit/fresh parser/resolver/lower/count/hash preflight後に
  B3M2B2Aをimplement。
- [ ] Task 258B4前にB3M2B2Bをfreeze/implement。

## Checker Task 258B3M1 runner implementation ledger

- [x] exact 113-byte/56-node sourceだけをB3N/B3/B2/B1/Aより先にselect。
- [x] corpus/semantic routeをactivateせず、6 terms、2 atomic formulas、
  2 statements、2 witnesses、1 nameをassemble。
- [x] identity、mutation/replay、byte/subtree near miss、both ownership
  orders、active isolation、empty final semanticsのcompound testsをexactly
  5本追加。
- [x] 394 tests、leaf/facade/root/test sizes
  `3724/688/2501/7246`、production 30 paths / 38,103 linesを再測定。

## Checker Task 258B3M2A runner documentation ledger

- [x] exact 107-byte/hash selector、49-node/root-48 arena、
  zero-diagnostic frontend、theorem-only resolver provenance、
  B3M1/B3N/B3/B2/B1/A precedenceをfreeze。
- [x] Task-48/252/256/base + witness/name `1/0`、numeric request ownership、
  complete mutation/replay、全near miss、both ownership orders、final
  clone/debug/rollback、empty semanticsをfreeze。
- [x] future runner tests exactly 5本をfreezeし、394 tests、sizes
  `3724/688/2501/7246`、production 30 paths / 38,103 lines、全
  corpus/expectation/sidecar/trace/active/list/hash baselineを維持。

## Checker Task 258B3M2A runner implementation ledger

- [x] exact 107-byte/49-node sourceだけをB3M1/B3N/B3/B2/B1/Aより先に
  selectし、全near missをdormantに維持。
- [x] authenticated lower/base handoffとone unnamed numeral witness/no
  namesをassembleし、public/active/binding/semantic ownershipを追加しない。
- [x] identity、precedence、mutation/replay、全near miss、both
  family/active order、rollback、detail projection、empty final semanticsの
  compound testsをexactly 5本追加。
- [x] 399 tests、sizes `4185/691/2505/8611`、production 30 paths /
  38,571 linesを実測し、B3M2B-before-B4を保持。

## Checker Task 258B3M2B1 runner prerequisite ledger

- [x] final-LF 113-byte/hash、53 nodes/root 52、diagnostics 0、one local
  exported theorem owner/labelをfreeze。
- [x] five roots / Task-252 `6/5/0`、wrapper term 2 / child 3、refs
  `0/1/2/3/4 -> 0/1/3/4/5`、atomic `[0,4]`、input refs `[0,3]`。
- [x] one unnamed outer-term witness/no names、no public/active/binding/
  fixture/trace credit/semantic outputをfreeze。
- [x] future tests exactly 5本、unchanged 399 tests、30 paths /
  38,571 lines、B3M2B2-before-B4をfreeze。
- [x] docs commit/fresh preflight後にB3M2B1だけをimplementし、
  404 runner testsを実測。

## Checker Task 258B3M2B1 runner implementation ledger

- [x] exact dormant 113-byte/53-node selectorだけを追加し、frontend
  diagnostics 0をassert。
- [x] five roots / six Task-252 primariesを分離し、wrapper term 2 /
  child 3をTask-256 equalitiesからexclude。
- [x] paired base + `1 witness / 0 names`だけをpublishし、detail key、
  active route、binding、semantic outputなし。
- [x] exactly five testsを維持し、lower fail-closeを弱めずTasks 253–255
  both ownership ordersをcover。
- [x] 404 tests、sizes `4676/695/2508/9902`、30 production paths /
  39,069 linesを実測し、B3M2B2をB4前に保持。

## Checker Task 258B3M2B2A runner prerequisite ledger

- [x] final-LF 121-byte `take ((x));`、SHA-256
  `35396db1f7e22abfbe94861709b2ab9bca38d4464712dfbce114533d2ab4d71d`、
  57 nodes/root 56、frontend diagnostics 0だけをfreeze。
- [x] five roots / seven primaries、wrapper chain `2 -> 3 -> 4`、
  refs `0/1/4/5/6`、equalities `[0,1]` / `[5,6]`をfreeze。
- [x] paired base + `1 witness / 0 names`、detail key/active/binding/
  semantics/fixture/sidecar/trace changeなしをfreeze。
- [x] exactly five future compound tests、unchanged 404-test /
  30-path/39,069-line baseline、B3M2B2B-before-B4をfreeze。

## Checker Task 258B3M2B2A runner implementation ledger

- [x] exact dormant 121-byte/57-node selectorだけを追加し、frontend
  diagnostics 0をrequire。
- [x] five roots/seven primaries、chain `2 -> 3 -> 4`、five refs、two
  equalities、paired base + `1 witness / 0 names`をcompose。
- [x] lower-producer-first failure、`Some(Vec::new())`、lookups `1/1`、
  uses `[1; 5]`、empty binding/semantic outputをpreserve。
- [x] exactly five compound testsでidentity、corruption、near miss、
  family/active isolation、replay、rollback、final cloneをcover。
- [x] 409 tests、sizes `5188/699/2513/11234`、30 production paths /
  39,590 linesを実測し、B3M2B2BをB4前に保持。

## Checker Task 258B3M2B2B1P runner prerequisite ledger

- [x] private explicit-context Task-253 unwrapped imported application seamを
  freezeし、legacy context-0 entry pointを維持。
- [x] 143-byte/63-node motivating sourceとproof-context-1 Task-253
  `1/0/1/2/2` profileをstatement consumer追加なしでfreeze。
- [x] identity、context/provenance/form corruption、replay、legacy byte
  compatibilityのcompound testsちょうど2件をfreeze。
- [x] 409 tests、sizes `5188/699/2513/11234`、production 30 paths /
  39,590 lines、全active/fixture/expectation/sidecar/trace artifacts、
  B1P-before-B1A orderを維持。

## Checker Task 258B3M2B2B1P runner implementation ledger

- [x] private explicit-context helperを追加し、context-0 delegationで
  compatibilityを維持。
- [x] exact Task-252 roots/public Task-253 producerをproof context 1でreuse
  し、statement consumer/lower row duplicateを追加しない。
- [x] identity、complete fail-close、replay/rollback、fixed legacy bytes、
  empty downstream ownershipのcompound testsちょうど2件をpass。
- [x] 411 tests、Task-253 sizes `1782/701/2514/2799`、production
  30 paths / 39,857 linesを実測。
- [x] exact B3M2B2B1A application-witness contractをseparate
  documentation commitでfresh-inventory/freeze。

## Checker Task 258B3M2B2B1A runner prerequisite ledger

- [x] final-LF 143-byte/hash、diagnostics 0、63 nodes/root 62、
  theorem/import provenance、proof context 1をfreeze。
- [x] Task-252 `6/4/2`、Task-253 `1/0/1/2/2` reuse、equality exclusion、
  base `1/2/2/2/2`、unnamed `Application(0)` witnessをfreeze。
- [x] owned take/witness nodes 49/48、unowned traversal node 47、
  Task-253 target node 46をlower-row duplicateなしでfreezeし、atomic
  checker three-handoff installerをrequire。
- [x] compound tests 5件、empty semantic/proof/goal output、no active/
  fixture/expectation/sidecar/trace changeをfreeze。
- [x] 411 tests、sizes `5188/701/2514/11234`、30 paths/39,857 linesを
  preserveし、implementation 416 testsをproject。
- [x] docs commit/fresh preflight後、exact B1A dormant consumerだけを実装。

## Checker Task 258B3M2B2B1A runner implementation ledger

- [x] exact 143-byte/63-node sourceだけをselectし、theoremとimported
  `parser.type_fixtures::++` local/FQN resolver provenanceをauthenticate。
- [x] Task-252 `6/4/2`、Task-253 `1/0/1/2/2`、Task-256 equality exclusionを
  reuseし、base `1/2/2/2/2` + unnamed `Application(0)` witness 1件を
  atomic checker installerでpublish。
- [x] exactly five compound testsで全loaded-source byte mutation、reparsed
  near miss、provenance/dependency/precedence corruption、family/active-route
  isolation、rollback/replay、final clone、empty semantic/proof/goal tablesを
  cover。
- [x] runner 416 tests、sizes `5618/706/2520/11945`、production
  30 paths / 40,298 linesを実測し、active cases、fixtures、expectations、
  sidecars、trace metadataを不変に維持。

## Checker Task 258B3M2B2B1B1P runner prerequisite ledger

- [x] diagnostics 0のexact final-LF 158-byte/67-node parenthesized
  imported application sourceをfreeze。
- [x] shared Task-252 `6/4/2`、Task-253 `1/1/1/2/2`、proof context 1、
  wrapper node 50、application node 48、imported `++` provenanceをfreeze。
- [x] implementationをprivate wrapper-aware Task-253 reuse sibling 1件に
  限定し、全unwrapped context-0/context-1 bytesをpreserve。
- [x] wrapper/context/provenance corruption、stale replay、clean replay、
  compatibilityのrunner compound testsちょうど2件をfreeze。
- [x] 全158 source-byte/67-node mutations、全success fields、
  dormant-route exclusion、lower-stage precedence、atomic rollback、
  separate context-0/context-1 debug hashesをfreeze。
- [x] 416 tests、Task-253 sizes `1782/706/2520/2799`、30 paths /
  40,298 lines、全active/fixture/expectation/sidecar/trace artifactsを
  preserve。
- [x] docs commitとfresh preflight後、B1B1Pだけをimplementし、B1B1
  statement consumerはlater logical taskでfreeze。

## Checker Task 258B3M2B2B1B1P runner implementation ledger

- [x] private exact wrapped-imported-application reuse seamだけを追加し、
  legacy unwrapped contexts 2件をpreserve。
- [x] complete imported `++` resolver provenanceをauthenticateし、
  same-source identity/path/signature/export/contribution substitutions
  5件をreject。
- [x] exactly two compound testsで全source bytes、全AST fields、exact
  eight-entry reparsed near-miss matrix、empty downstream tables、atomic
  failure、replay、compatibilityをcover。
- [x] 418 tests、sizes `2652/708/2523/3727`、production 30 paths /
  41,173 linesとrecorded production/test-list hashesを実測。
- [x] public/active/statement routes、fixtures、expectations、sidecars、
  trace status/count、semantic/proof/goal ownershipを不変にし、B1B1
  documentationはimplementation commit後にのみselect。

## Checker Task 258B3M2B2B1B1 runner prerequisite ledger

- [x] exact final-LF 158-byte/67-node selectorとcomplete local
  theorem/imported `++` resolver provenanceをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `6/4/2`、wrapped Task-253
  `1/1/1/2/2`、equality-only Task-256、base `1/2/2/2/2`、one unnamed
  `Application(0)` witness/no namesをfreeze。
- [x] B1B1P wrapped seam/existing B1A checker API/atomic installerをone
  explicit private B1B1 profile経由でreuse。
- [x] exact runner tests 5件で全bytes/nodes、resolver substitutions、
  near-miss matrix、precedence、B1A/family/active isolation、rollback/
  replay/clone、empty upper tablesをfreeze。
- [x] tests `374/418`、全measured sizes/counts/hashes、public/active
  routes、fixtures、expectations、sidecars、trace status/count、
  semantic/proof/goal ownershipをpreserve。
- [x] documentationだけをcommitし、fresh-inventory後B1B1をimplement。

## Checker Task 258B3M2B2B1B1 runner implementation ledger

- [x] B1A/active dispatchをbroadenせず、exact private wrapped selectorと
  B1B1 routeをimplement。
- [x] frozen runner tests 5件/checker tests 4件を全てpass。librariesは
  `378/423`。
- [x] `source_drift`、`test_gap`、completion `design_drift`をcloseし、
  test-sufficiency/implementation reviewsはfindingsなし。
- [x] fixtures、expectations、sidecars、trace status/count、public APIs、
  semantic/proof/goal/type-substitution deferralsをpreserve。
- [x] commit前のfinal read-only quality reviewは全hard gate PASS、
  valid score `98/100`。

## Checker Task 258B3M2B2B2P runner prerequisite ledger

- [x] B2Aより先に、exact final-LF 172-byte/hash、diagnostics 0、
  76 nodes/root 75のimported structure-constructor sourceをselect。
- [x] Task-48 `2/1/0`、Task-252 `6/4/2`、Task-254
  `1/0/1/2/0/2/6`をTask-252 row duplicateなしでproof context 1にfreeze。
- [x] exact imported `TypeCaseStruct#5` contribution/origin/export
  provenanceと、constructor 59/members 20/24だけのowned kindsをfreeze。
- [x] exactly two runner compound testsで全bytes/nodes、lower rows、
  provenance、substitutions、precedence、atomic replay、existing Task-254
  outputをcoverするとfreeze。
- [x] checker source/API/tests、statement consumers、active routes、
  fixtures、expectations、sidecars、trace status/count、全semantic/proof/
  goal/IR ownersをpreserve。
- [x] documentation baselines `378/423`、sizes
  `1689/713/2528/1716`、30 paths / 41,513 lines、全measured hashesを
  preserve。
- [x] dedicated documentation commit/fresh preflight後、B2Pだけをimplementし、
  runner tests 425をproject、全changed counts/hashesを再実測してからB2A
  documentationをfresh-inventory。

## Checker Task 258B3M2B2B2P runner implementation ledger

- [x] frozen 4 files内でexact private owned-kind selectorと
  existing-context/shared-Task-252 Task-254 seamをimplement。
- [x] frozen compound tests 2件とrunner library 425 testsをpass。
- [x] `source_drift`、`test_gap`、completion `design_drift`をclose。
- [x] checker/public/active/fixture/expectation/sidecar/trace/semantic
  boundariesをpreserveし、Task-258 rowはpublishしない。
- [x] sizes、production manifest、test-list hashesを再測定し、B2Aを次に維持。
- [x] final read-only quality reviewはfindingsなし、全hard gate PASS、
  valid score `98/100`。
- [x] commit後にfresh-inventoryし、B2A documentationをseparately freeze。

## Checker Task 258B3M2B2B2A runner frozen-contract ledger

- [x] `258B3M2B2B2A`をhistorical `258B3M2B2A`と区別し、exact
  final-LF 172-byte/76-node structure-constructor witnessだけをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `6/4/2`、Task-254
  `1/0/1/2/0/2/6`、direct structure edge/fingerprintなしのequality-only
  Task-256 `2/0/0/0/0/0/0/4/4`、Task-258 base `1/2/2/2/2`、
  witness/name `1/0`のreuseをfreeze。
- [x] complete current/imported provenance、62/61だけのownership、retained
  Task-254 59/20/24とTask-252 45/47/53/56/63/65 ownership、unowned
  52/54/57/60をfreeze。
- [x] sole target `Structure(0)`、additive checker target/fingerprint/
  builder/atomic installer seams、lower rows/parser-resolver projectionsを
  duplicateしないことをfreeze。
- [x] runner 5/checker 4 testsで全bytes/nodes/dependencies/precedence/
  isolation/rollback/replay/final clone/compatibility/malformed recovery/
  empty semanticsをfreeze。
- [x] source、fixtures、expectations、sidecars、trace status/count、active
  routes、APIs、diagnostics、test baselines `378/425`、runner sizes
  `5962/2857/715/2531/13381/2991`、counts/exact hashesをpreserve。
- [x] no-findings specification reviewと全documentation hard gatesを
  valid final quality `98/100`で完了。
- [x] dedicated commit後、fresh-inventoryしてB2Aだけをimplement。

## Checker Task 258B3M2B2B2A runner implementation ledger

- [x] exact private selectorをimplementし、lower rowsをduplicateせずB2Pと
  Task-48/252/254/256/baseをreuse。
- [x] checker-owned APIs経由で`Structure(0)` witness 1件をpublishし、
  runner 5/checker 4 exact testsをpass。
- [x] bounded B2A `source_drift`/`test_gap`をcloseし、active/fixture/
  expectation/sidecar/trace/semantic boundariesをpreserve。
- [x] runner tests 430、sizes/manifests、test-list hashesをrecord。
- [x] no-findings test/implementation/docs consistency reviewsをcomplete。
- [x] focused checker/runner `4/4`/`5/5`、full format、all-target/all-feature
  Clippy warnings denied、libraries `382/430`/lint policies `15/14`を含む
  `cargo test -q`をPASS。
- [x] five CLIs exit 0、warnings 23/errors 0、counts/hashes不変をverifyし、
  manifests/test lists/forbidden artifactsはunchanged、`stash@{0}`は
  untouched。
- [x] 全9 hard gates PASS、valid score `98/100`でfinal quality reviewを
  complete。
- [x] `7613d50d`でcommitし、clean metadata/stash invariantsをverifyして
  next dependencyをfresh-inventory。

## Checker Task 258B3M2B2B2BP runner frozen-contract ledger

- [x] B2Bとdistinctなprivate selector proof-context lower prerequisite
  としてB2BPを171-byte/79-node exact source上でfreeze。
- [x] Task-48 `2/1/0`、Task-252 `6/4/2`、Task-254
  `2/0/1/3/0/3/9`、provenance/ownership/edge/request/malformed/exclusion
  matricesをfreeze。
- [x] existing source-structure ownersのprivate selector site/owned-kind/
  context-handoff siblingsだけをfreezeしchecker/public/active surfaceなし。
- [x] exact runner tests 2件/checker 0件とcorruption/replay/constructor seam
  compatibilityをfreeze。
- [x] fixtures、expectations、sidecars、trace/credit、diagnostics、active
  cases、baseline `382/430`、sizes/manifests/hashesをpreserve。
- [x] no-findings specification/source-documentation reviewsと全verification
  をcomplete。
- [x] external docs commit `6f84d4eb`をreport-only metadata conflictとして
  recordし、docs-only BPC1 imported-provenance correctionをfreeze。
- [x] BPC1後のtest/implementation-boundary/source-documentation reviewsを
  findingsなしまでrepeat。
- [x] BPC1 final qualityをfindingsなし、全9 hard gates、valid `98/100`
  でPASS。
- [x] correctionだけをcommit後、B2BP implementationだけを
  fresh-inventory。
- [x] separate implementation後にB2B frozen consumer docsへ戻る。

## Checker Task 258B3M2B2B2BP runner implementation ledger

- [x] frozen 4 filesへexact private selector site/owned-kind/context
  handoffを実装し、public/active surfaceを追加しない。
- [x] exact tests 2件、全mutation/precedence/replay gates、exact malformed
  diagnostic、B2P/B2A/legacy compatibilityをPASS。
- [x] bounded `source_drift`/`test_gap`をcloseし、test-sufficiency/
  implementation reviewsはfindingsなし。
- [x] fixtures、expectations、sidecars、trace/credit、diagnostics、active
  cases、checker APIs、semantic boundariesをpreserve。
- [x] runner tests `432`、sizes、production/test-list hashes、unchanged
  CLI counts/hashesを記録。
- [x] source/documentation consistencyをfindingsなしで完了し、全9 hard
  gatesとvalid `98/100`でfinal quality reviewをPASS。
- [x] commit 1件を作成後、B2B documentationをfresh-inventory。

## Checker Task 258B3M2B2B2B runner frozen-contract ledger

- [x] exact source/parser/malformed profileと全Task-48/252/254/256/258
  rows、provenance、ownership、exclusionsをfreeze。
- [x] production-private B2BP consumer、B2A/B2B family boundary、exact
  runner implementation owners 5 files、no-public/no-active routeをfreeze。
- [x] exact runner tests 5件、precedence/replay matrices、subtree near
  misses、final clone、rollback、semantic emptinessをfreeze。
- [x] `382/432` baseline、`386/437` projection、module/manifest/test-list/
  CLI hashes、unchanged fixture/trace/coverage impactを記録。
- [x] reviews 4系統をfindingsなしで完了し、final quality全9 hard gatesを
  valid `98/100`でPASS。
- [x] paired documentation prerequisiteだけを`4d2fb2b6`でcommit。
- [x] docs commit後にfresh-inventoryし、B2Bだけをimplement。

## Checker Task 258B3M2B2B2B runner implementation ledger

- [x] exact frozen eight-file transactionを実装し、private B2BP
  owned-kind/proof-context handoff seamsだけをconsume。
- [x] exact source、lower/base/witness rows、ownership exclusions、
  transitive surface validation、no-public/no-active/no-semantic boundaryを
  preserve。
- [x] exact frozen runner tests 5件をPASSし、checker/runner library counts
  `386/437`をrecord。
- [x] final runner sizes `6826/4506/728/2543/17120/4315`、30-path /
  45,224-line production manifest、production/test-list hashesをrecord。
- [x] bounded `design_drift`、`source_drift`、`test_gap`をclose。
- [x] specification/dependency reviewをfindingsなしでcomplete。
- [x] test-sufficiency reviewをfindingsなしでcomplete。
- [x] implementation reviewをfindingsなしでcomplete。
- [x] source/documentation consistency reviewをfindingsなしでcomplete。
- [x] focused/full verification、lint、count、hash gatesを全てcomplete。
- [x] 全hard gatesとvalid score `90/100`以上でfinal read-only quality
  reviewをPASS。
- [x] implementation commit `8311502c`を作成し、clean worktree、
  ahead-three origin metadata、untouched stashをverifyしてからB2Cより
  先のB2CP prerequisiteをfresh-inventory。

## Checker Task 258B3M2B2B2CP runner frozen-prerequisite ledger

- [x] B2Cより先のprivate update/`FieldUpdate` reuse seamをestablish。
- [x] 181-byte/86-node exact source、180-byte malformed profile、
  Task-48/252/254 rows、provenance、ownership、edges、exclusionsをfreeze。
- [x] exact four-file runner boundary、tests 2件、checker tests 0件、
  no-statement/no-public/no-active/no-semantic outputをfreeze。
- [x] private `ImportedStructureUpdateSite`、owned-kind、context-handoff
  siblingsとB2P-constructor/B2BP-selector compatibilityをfreeze。
- [x] second testのexact nameを
  `task258b3m2b2b2cp_structure_update_corruption_replay_and_prior_sibling_compatibility_fail_closed`
  に固定。
- [x] functional-copy semantics、type/result identity、witness
  obligations、theorem/proof acceptance、goals、IRをdeferし、`x = x`
  goal内の`take`にsemantic acceptance claimを与えない。
- [x] libraries `386/437`、projection `386/439`、current module、
  manifest、test-list、count、CLI hashesをrecord。
- [x] skipped prerequisiteを`design_drift`、future seamをbounded
  `source_drift`、testsを`test_gap`と分類し、blocking/nonblocking
  いずれの`spec_gap`もなし。
- [x] specification/dependency、test-sufficiency、
  implementation-boundary、source/documentation reviewsをfindingsなしで
  completeし、documentation verificationと全hard gatesをPASS。
- [x] concurrent docs commit `817bb92b`をreport-only
  `repo_metadata_conflict`としてrecordし、restored `spec_gap` labelが
  hard-gate/`98/100` claimsをinvalidateしたことを記録。
- [x] CPC1 repeated no-findings reviewsをcompleteし、全9 hard gatesを
  PASS、valid final quality `98/100`を取得。unrelated incomplete source
  diffでblockされるlive broad rerunを明示的にjustify。
- [x] docs-only correction `258B3M2B2B2CPC1`を`ee267d9c`として
  separate commit。
- [x] fresh-inventory後private dormant B2CP runner seamだけをimplementし、
  frozen tests exactly 2件をPASS、`design_drift`、`source_drift`、
  `test_gap`をclose。
- [x] final test-sufficiency/implementation re-reviewsをfindingsなしで
  complete。
- [x] focused/workspace fmt、Clippy、tests、全count/hash gatesをPASS。
- [x] specification/corpus/trace-creditを変更せずfinal runner metricsと
  narrative-only audit impactを同期。
- [x] final source/documentation reviewをfindingsなしでcomplete。
- [x] independent final qualityをfindingsなし、全9 hard gates PASS、
  valid `98/100`でcomplete。
- [x] staged-diff auditとdedicated B2CP implementation commit
  `b146f0f72dceac2233c9d679b7820e264974b227`をcomplete。
- [x] B2CP commit後B2Cをfresh-inventory。

## Checker Task 258B3M2B2B2C runner frozen-contract ledger

- [x] B2CP commit `b146f0f72dceac2233c9d679b7820e264974b227`
  completeとclean fresh B2C selectionをrecord。
- [x] exact 181-byte/86-node source、180-byte malformed profile、five
  valid-excluded byte/hash/node profilesをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `7/4/3`、Task-254
  `2/0/1/3/1/4/9`、Task-256 `2/0/0/0/0/0/0/4/4`、Task-258 base
  `1/2/2/2/2`、witness `1/0`をfreeze。
- [x] resolver provenance、cross-family edges、ownership/exclusions、
  B2C only `72/71` plus witness-to-`Structure(0)`をfreeze。
- [x] existing eight files、unchanged B2CP seam、no-public/no-active、
  checker tests 4/runner tests 5 exact namesをfreeze。
- [x] baseline/projection `386/439` -> `390/444`、sizes/manifests/hashes、
  unchanged corpus/CLI gates、narrative audit impactをfreeze。
- [x] stale selectionを`design_drift`、future codeを`source_drift`、
  nine testsを`test_gap`にclassifyし、normative `spec_gap`、
  expectation drift、current boundary violationなしをrecord。
- [x] executable/canonical artifactsをimmutableとし、`x = x`下の`take`
  はsource transport onlyとrecord。
- [x] specification reviewをfindingsなしでcomplete。
- [x] test-sufficiency reviewをfindingsなしでcomplete。
- [x] implementation-boundary reviewをfindingsなしでcomplete。
- [x] source/documentation consistency reviewをfindingsなしでcomplete。
- [x] documentation verificationと全required count/hash gatesをPASS。
- [x] 全hard gates PASSかつvalid score `98/100`のfinal read-only quality
  reviewをcomplete。
- [x] cached-diff auditをPASSし、dedicated B2C frozen-contract
  documentation commit
  `d6076cc757ce675d1b46a720b4f00805923d3c70`を作成。
- [x] fresh-inventory後scoped B2C eight-file transactionだけをimplement。

## Checker Task 258B3M2B2B2C runner implementation ledger

- [x] runner implementationをfrozen statement/structure/facade/root/test
  filesに限定しprivate B2CP seamをunchanged consume。
- [x] exact source、malformed profile、valid excluded 5 profilesをauthenticateし、
  active fixture/public routeを追加しない。
- [x] Tasks48/252/254/256/base rowsをpreserveし、witness-to-
  `Structure(0)` edgeだけをpublish。
- [x] runner tests exactly 5件とpaired checker tests 4件を追加しPASS。
- [x] runner library `444`+policy suitesとchecker library `390`をPASS。
- [x] final test-sufficiency/implementation reviewsをfindingsなしでcomplete。
- [x] EN/JA plan/ledger/harness/module auditsとnarrative-only coverage auditを
  final sizes/hashesでsync。
- [x] spec、`.miz`、fixtures、expectations、sidecars、trace status/tests、
  coverage credit、active corpus、public API、semanticsをunchangedに保つ。
- [x] broad workspace fmt/Clippy/testsをPASSし、focused `4/4`/`5/5`、
  sibling `12/12`/`21/21` suitesもPASS。
- [x] final source/docs consistency re-reviewを**NO FINDINGS**でcomplete。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  valid `98/100`でcomplete。
- [x] cached implementation diffをauditしB2Cを
  `e8373c683448e524cb98edde83fdf8de83a125cd`としてcommit。
- [x] clean ahead 8/behind 0 post-commit repo state、unchanged stashを
  verifyしB3Pをfresh inventory。

## Checker Task 258B3M2B2B3P runner frozen-contract ledger

- [x] exact 117-byte/hash、57-node/root-56 parser profileをfreeze。
- [x] proof context 1、local resolver record、Task48 `2/1/0`、Task252
  `6/4/2`、Task255 `1/0/0/0/0/2/1`、ownership/exclusionsをfreeze。
- [x] runner files exactly 4件とcompound tests 2件をfreezeし、existing
  context-0 helper bytesをpreserve。
- [x] 同じ2 testsに全source bytes/LF variants、node/lower-table fields、
  resolver substitutions、owner partitions、precedence/replay/rollback/
  clones、family/semantic emptiness、Task111 handoff/typed/resolved literal
  debug hashesをexhaustさせる。
- [x] checker source/test/API、upper B3A statement-witness edgeを追加しない。
- [x] canonical/executable/trace artifactsとsemantic deferralsをpreserveし、
  baseline `390/444`、projection `390/446`、exact counts/hashesをrecord。
- [x] specification reviewをfindingsなしでcomplete。
- [x] documentation review/repeatをfindingsなしでcomplete。
- [x] test-sufficiency reviewをfindingsなしでcomplete。
- [x] implementation-boundary reviewをfindingsなしでcomplete。
- [x] source/documentation consistency reviewをfindingsなしでcomplete。
- [x] source/hash、lint `15/14`、libraries `390/444`、production/test-list、
  5 CLI hashes、exact scope、diff、trace no-op verificationをPASS。
- [x] final qualityをfindingsなし、全9 hard gates PASS、valid
  `98/100`（`20/20/15/14/10/10/5/4`）でcomplete。
- [x] task-only docsをaudit/stageしfrozen contractを
  `285a1f11c310bb313c4c6b4feae914eb11f74754`としてcommit。
- [x] clean post-commit invariantsとunchanged stashをverify後、private
  B3P runner seamをfresh inventory。

## Checker Task 258B3M2B2B3P runner implementation-closure ledger

- [x] prerequisite commit
  `285a1f11c310bb313c4c6b4feae914eb11f74754`をrecord。
- [x] exact 4 existing runner filesに`pub(super)` explicit-context sibling/
  context-0 delegateをimplementしpublic/active changeなし。
- [x] exact 2 testsでbytes/LF、57 nodes、resolver `63`、binding `39`、
  Task-252/255、fingerprint-only absence、precedence/replay/clones、
  literal hashes/isolationをcover。
- [x] test-sufficiency/implementation reviewsを**NO FINDINGS**でcomplete。
- [x] focused `2/2`、runner library `446/446`、format、package Clippy
  `-D warnings`、diff checkをPASS。
- [x] sizes `7240/4517/740/2557/19275/2528`、production `30/49472`、
  current production/test-list hashesをrecord。
- [x] canonical/fixture/expectation/sidecar/trace/checker/public/activeを
  unchangedに維持。
- [x] source/documentation consistencyとdocumentation/boundaryのrepeat
  reviewを**NO FINDINGS**でcomplete。
- [x] lint-policy `15/14`、metadata `137`、focused `2/2`、runner
  library `446/446`、format、workspace-wide warnings-denied Clippy/tests、
  5 CLI/count/hash、current manifest/test-list hash、exact 30-file scope、
  diff-check gatesをPASS。
- [x] final read-only quality reviewを**NO FINDINGS**、全9 hard gates
  PASS、valid `98/100`（`20/20/15/14/10/10/5/4`）でcomplete。
- [x] B3P implementation closureを
  `abbfedfc2cdbaa97d8294893859da8cd350ad9a8`としてaudit/stage/commit。
- [x] clean post-commit HEAD、ahead-10 origin metadata、untouched stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`をverifyしupper B3Aを
  fresh inventory。

## Checker Task 258B3M2B2B3A runner frozen-contract ledger

- [x] B3P commit `abbfedfc2cdbaa97d8294893859da8cd350ad9a8`と
  fresh B3Aをclean/ahead-10/untouched-stash evidenceでclose。
- [x] authority、source/resolver label、Tasks48/252/255/256/258、
  witness1/names0、partition/graph、non-existential intentをfreeze。
- [x] exact7 files、unchanged B3P set-term consumer、additive API/debug、
  runner5+checker4 tests、matrices、precedence、deferralsをfreeze。
- [x] `design_drift`/`source_drift`/`test_gap`、blocking disagreementなし、
  baselines/projections/hashes、trace no-op、exact32 scopeをrecord。
- [x] specification/documentation、test-sufficiency、implementation/API
  boundary reviewsを**NO FINDINGS**でcomplete。
- [x] source/count/hash、lint/library、CLI、scope、diff、trace no-opをPASS。
- [x] documentation/boundary/source-docs reviewsを**NO FINDINGS**でcomplete。
- [x] final qualityを**NO FINDINGS**、全9 hard gates PASS、valid
  `98/100`（`20/20/15/14/10/10/5/4`）でcomplete。
- [x] dedicated documentation-only commit
  `f4ff45964d97b31b6c328381120ba8ede080a2b1`をcreate。
- [x] clean ahead-11/behind-0 postcommit state、unchanged stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`、fresh implementation
  inventoryをverify。

## Checker Task 258B3M2B2B3A runner implementation ledger

- [x] prerequisite commit/postcommit/fresh-inventory gatesをclose。
- [x] exact runner4+checker3 source filesだけをimplementし、両set-term
  source ownerと全authority artifactをpreserve。
- [x] exact runner5+checker4 tests、additive API、set-only tuple、atomic
  installation/final clone、全frozen matricesを追加。
- [x] specification/test-sufficiency/implementation reviewsを
  **NO FINDINGS**でcomplete。
- [x] focused/package tests、fmt、targeted Clippy、5 CLI、final
  count/hash manifests、diff checksをPASS。
- [x] 2回目のsource/documentation consistency repeatを
  **NO FINDINGS**でcomplete。
- [x] final documentation/boundary rereadを**NO FINDINGS**でcomplete。
- [x] parent final verificationをPASS：focused checker `4` + runner `5`、
  checker package `394` + lint-policy `15`、mizar-test package `451` +
  layout `3` + lint-policy `14` + metadata `137` + public-enum `2` +
  snapshot `21`、fmt、workspace Clippy/tests、5 CLI counts/hashes、
  production manifests/test lists、diff check、exact `39`-file scope。
- [x] independent final read-only quality reviewを**NO FINDINGS**、全9
  hard gates PASS、score capなし、valid `98/100`
  （`20/20/15/14/10/10/5/4`）、記載済みresidual deferrals unchangedで
  complete。
- [x] dedicated B3A implementation commit
  `a147bad88f1963c504f796051ba0b855eca71d07`をcreate。
- [x] clean ahead-12/behind-0 postcommit stateとunchanged stashをverify。
- [x] fresh inventoryでB3B empty-enumeration documentationをselect。

## Checker Task 258B3M2B2B3B runner ledger

- [x] exact 118-byte/hash source、diagnostics 0、50 nodes/root 49、
  resolver label、complete Task-48/252/255/256/258 handoffsをfreeze。
- [x] zero-edge Enumeration target 1件とunnamed witness 1件をfreezeし、
  new lower helper/public APIは追加しない。
- [x] exhaustive dormant runner tests exactly 5件とmatching checker tests
  4件をfreezeし、libraries `394/451`から`398/456`をproject。
- [x] `.miz`、expectations、sidecars、trace status/count、active route、
  CLI output、semanticsをunchangedに保持。
- [x] 全repeat reviewsとverificationを**NO FINDINGS**でcomplete。
- [x] documentation prerequisiteを
  `080e6824d843655986079f5d5fc41abe06b0fbd6`としてcommitし、clean
  ahead-13/behind-0 stateとunchanged stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`をverify後、separate B3B
  implementationをfresh inventory。

## Checker Task 258B3M2B2B3B implementation ledger

- [x] prerequisite commit/post-commit/fresh-inventory gatesをclose。
- [x] exact runner owners 4件とpaired checker owners 3件を実装し、
  public API/diagnostics/dependencies/routesをpreserve。
- [x] frozen runner tests 5件とchecker tests 4件を実装。
- [x] initial test-sufficiency findings 3件をexisting 9 tests内で
  remediate。
- [x] additional B3B-specific Task-48/252/255 lower-field mutation findingを
  exact `32/55/23` matricesでremediate。
- [x] bounded test-only follow-up前のindependent implementation repeatを
  **NO FINDINGS**でcomplete。
- [x] post-auth injectionとstage-prefix/non-generic-guard assertionsを追加し、
  全test-sufficiency repeatsを**NO FINDINGS**でcomplete。
- [x] final implementation repeatを**NO FINDINGS**でcomplete。
- [x] follow-up後のfocused testsとformat/diffをrerun。
- [x] final runner count/hash measurementsをcomplete。
- [x] libraries `398/456`、workspace Clippy/tests、5 CLI invariantsを
  rerun。
- [x] medium `design_drift` wording fixes 2件後のsource/documentation
  consistency repeatを**NO FINDINGS**でcomplete。
- [x] independent final documentation/boundary reviewを
  **NO FINDINGS**でcomplete。
- [x] independent final read-only quality reviewを**NO FINDINGS**、全9
  hard gates PASS、score capなし、valid `98/100`
  （`20/20/15/14/10/10/5/4`）でcomplete。
- [x] exact `39` synchronized task filesをstageし、cached diffをinspect。
- [x] implementation commit
  `dbbf5f6a2b0bd58d8434fb4687f7bfad398ca4bc`をcreate。
- [x] clean ahead-14/behind-0 post-commit/stash unchangedをverify。
- [x] fresh inventoryでB3C choice witnessをselect。

## Checker Task 258B3M2B2B3C documentation ledger

- [x] exact `110`-byte/hash、52-node/root-51 choice sourceとlocal resolver
  provenanceをfreeze。
- [x] lower profiles `2/1/0`、`4/4/0`、Tasks 253/254 empty、
  `1/0/0/1/0/0/2`、`2/0/0/0/0/0/0/4/4`、base `1/2/2/2/2`、
  witness `1/0`をfreeze。
- [x] owner/unowned graphとTask-255 child edge 0のSetTerm witness edgeを
  freeze。
- [x] exact checker 4/runner 5 namesと`32/55/39/72/62/21`、
  byte/node/resolver/family/replay matricesをfreeze。
- [x] future runner scopeをexact 4 filesに固定し両`source_set_term.rs`は
  unchanged。
- [x] fixture/expectation/sidecar/trace status/count/active route/CLI/
  semantics/coverage creditをpreserve。
- [x] initial medium ownership/matrix findingsをfixし、repeat specification
  review **NO FINDINGS**。
- [x] consistency/boundary reviewを**NO FINDINGS**でcomplete。
- [x] independent final quality reviewを**NO FINDINGS**、全9 hard gates
  PASS、score capなし、valid `98/100`でcomplete。
- [x] exact docs-only scope、crate/workspace checks、5 CLIs、全count/hash/
  no-op gatesをverify。
- [x] dedicated B3C documentation commit
  `ea48ffc4fa586ac6d0813cd23a6b1d9b571087b2`を作りclean post-commit/stashを
  verify。
- [x] B3C implementationをfresh inventory。

## Checker Task 258B3M2B2B3C implementation ledger

- [x] prerequisiteをclean ahead-15/behind-0、stash unchangedでcloseし、
  lower-stage prerequisite不要をconfirm。
- [x] frozen runner 4 + checker 3 source filesだけをimplementし、両
  `source_set_term.rs` ownersをpreserve。
- [x] exact runner 5/checker 4 testsと`32/55/39/72/62/21` field
  matricesをimplement。
- [x] resolver replayとupper-family prefix/non-generic `test_gap`を
  remediate。
- [x] enumeration siblingを変えずB3A-hard-coded B3C
  `source_drift`/`test_gap`をremediate。
- [x] repeat test-sufficiency/implementation reviewsを**NO FINDINGS**で
  complete。
- [x] focused `5/5 + 4/4`、runner package
  `461+3/14/137/2/21`、formatをPASS。
- [x] final sizes、production/test hashes、unchanged CLI hashes、
  trace/authority no-opをrecord。
- [x] workspace Clippy/testsとfinal measurementsをcomplete。
- [x] final source/docs consistencyとquality reviewsをcomplete。
- [x] exact 39 synchronized task filesをstageしimplementation commit
  `7988a50934656ff90b31e06b883225f86196103b`をcreate。
- [x] clean ahead-1/behind-0 post-commit state、unchanged stashをverifyし、
  external origin movementをreport-onlyとする。
- [x] fresh inventoryでB3D qua witnessをselect。

## Checker Task 258B3M2B2B3D documentation ledger

- [x] exact 109-byte/hash、24-token、54-node/root-53 qua sourceとlocal
  resolver owner/label provenanceをfreeze。
- [x] lower profiles `2/1/0`、`5/4/1`、Tasks 253/254 empty、
  `1/0/0/1/0/1/2`、`2/0/0/0/0/0/0/4/4`、base
  `1/2/2/2/2`、witness `1/0`をfreeze。
- [x] owner/unowned graph、`QuaBase -> Primary(2)`、witness-to-SetTerm
  edgeをfreeze。
- [x] exact checker 4/runner 5 namesと`32/70/44/72/62/21`に加え、
  byte/node/resolver/family/replay matricesをfreeze。
- [x] future runner scopeを4 filesに固定し、両`source_set_term.rs` ownersを
  unchangedに保つ。
- [x] authority、fixtures、expectations、sidecars、trace
  status/count/tests、active behavior、semantics、coverage creditをpreserve。
- [x] 全repeated reviewsを**NO FINDINGS**でcomplete。
- [x] exact docs-only scope、crate/workspace checks、5 CLIs、全
  count/hash/no-op gatesをPASS。
- [x] 全9 hard gatesとvalid score `>=90/100`でindependent final qualityを
  complete。
- [x] dedicated B3D documentation commit
  `43af562c2cb84e72658cee059abbe7543ee73fe7`をcreate。
- [x] clean ahead-2/behind-0 post-commit stateとunchanged stashをverify。
- [x] B3D implementationをfresh inventoryし、lower-stage prerequisite
  不要をconfirm。

## Checker Task 258B3M2B2B3D implementation ledger

- [x] frozen runner 4 + checker 3 Rust consumersだけをimplementし、generic
  Task-255/source-set ownersをpreserve。
- [x] exact runner 5 + checker 4 testsと
  `32/70/44/72/62/21` matricesをimplement。
- [x] bytes/LF、nodes/root、resolver、owner/graph/subtree、family orders、
  replay/rollback/clone/debug、empty semanticsをcover。
- [x] test-sufficiency reviewを**NO FINDINGS**でcomplete。
- [x] focused `5/5 + 4/4`、runner/checker packages
  `466+3/14/137/2/21` / `406+15`、format、full ClippyをPASS。
- [x] final module sizes、production/test hashes、unchanged 5 CLI hashes、
  authority/trace/active/semantic no-opをrecord。
- [x] independent implementation reviewを**NO FINDINGS**でcomplete。
- [x] stale implementation-review stateのMedium `design_drift`、
  24-order wordingのLow、EN qua-edge table wordingのLowをfix。
- [x] final source/docs consistency、bilingual、boundary repeatsを
  **NO FINDINGS**でcomplete。
- [x] runner/checker packages、format、full Clippy、full workspace tests、
  5 CLI/count/hash final rerunsをPASS。
- [x] independent final read-only quality reviewを**NO FINDINGS**、全9
  hard gates PASS、score capなし、valid `100/100`
  （`20/20/15/15/10/10/5/5`）でcomplete。
- [x] metadata CLI warnings/errors `23/0`とlarge repeated-test diff review
  volumeをnonblocking residualとしてrecord。
- [ ] exact synchronized implementation scopeだけをstageしcached diffを
  inspect。
- [ ] implementation commit 1件をcreate。
- [ ] clean post-commit/stash invariantsをverifyしnext taskをfresh inventory。

## Checker Task 258B3M2B2B3E documentation ledger

- [x] B3D implementation commit
  `08a7d1e3d8c4b3b439325a16e1e139df4a1c18ed`、clean
  `origin/main...HEAD = 0/3` snapshot、unchanged stashをrecord。
- [x] exact final-LF 139-byte/hash、28-token、60-node/root-59 source、
  resolver provenanceをfreeze。
- [x] exact lower/upper profiles
  `2/1/0`、`5/4/1`、empty Tasks 253/254、
  `1/0/1/1/0/1/2`、`2/0/0/0/0/0/0/4/4`、
  `1/2/2/2/2`、witness/names `1/0`をfreeze。
- [x] exact ownership、generator segment `42` unowned、
  `ComprehensionMapper -> Primary(2)`、ordered requests、upper witness
  edgeをfreeze。
- [x] runner 5 + checker 4 future test namesと
  `32/70/53/72/62/21` matrices、five-family `120` ordersをfreeze。
- [x] future runner 4 + checker 3 consumersをfixし、両
  `source_set_term.rs`とactive/authority/trace/semantic ownersをpreserve。
- [x] complete semantic deferrals、trace/coverage no-op、lower
  prerequisite不要をrecord。
- [x] specification/test/boundary reviewsを実行し、findingをremediateして
  repeat。
- [x] source/documentationとbilingual consistency reviewsを実行。
- [x] relevant verificationとforbidden-path/no-op checksをcomplete。
- [x] independent final quality reviewを全hard gatesとvalid score
  `>=90/100`でcomplete。
- [x] B3E documentation commit
  `8075000bf79be3fdea6b22f366fb6d9e59781fe7`をcreateしfresh inventory。

## Checker Task 258B3M2B2B3E implementation ledger

- [x] frozen runner 4＋checker 3 consumersだけをimplement。
- [x] runner 5/checker 4 testsとmatrices/ordersを追加。
- [x] same-provenance successful coherent Task-255 negativesを使用。
- [x] test/implementation reviewを**NO FINDINGS**で完了。
- [x] focused `5/5 + 4/4`、libraries `471/410`をPASS。
- [x] final size/hash、CLI、all no-op boundariesを記録。
- [x] 3件の`design_drift`修正後、source/docs、bilingual、boundary
  consistencyを**NO FINDINGS**で完了。
- [x] independent qualityを**NO FINDINGS**、全9 gates PASS、capなし、
  valid `100/100`で完了。
- [x] focused/package、fmt、full Clippy、root workspace、5 CLI、
  count/hash/scope/forbidden/stash gatesをPASS。
- [x] exact B3E scopeをstageしcached diffをinspect。
- [x] B3E implementation commit
  `e4479691db3b0a8785bb16e94d386bd71a394274`をcreate。
- [x] post-commit/stash invariantsをverifyし、B4Aをfresh inventory。

## Checker Task 258B4A documentation ledger

- [x] distinct 80-byte/double-LF source/hash、26-node/root-25 parser
  surface、resolver owner provenanceをfreeze。
- [x] exact Task-252/256/257/B1/binding profilesとTask-258
  `1/1/1/0/1` `Composite(0)` associationをfreeze。
- [x] active 79-byte caseをlower-only route isolationとして保持し、全
  fixture/expectation/sidecar/trace editをforbid。
- [x] single crate-private Task-257B1 helper visibility seamを含むfuture
  runner consumer 5件とexact tests 5件をlower/upper mutation、replay、
  family-order、clone/debug、empty-semantic coverage付きでfreeze。
- [x] truth、acceptance、proof、facts、active behavior、public runner
  schemas、formula-statement coverage creditをpreserve。
- [x] repeated documentation reviewを**NO FINDINGS**でcomplete。
- [x] docs-only verificationと全no-op/count/hash/stash gatesをPASS。
- [x] final qualityを全hard gatesとscore `>=90/100`でcomplete。
- [x] dedicated B4A documentation commitをstage/inspect/createし、
  `9da1ac13e811c78359d8d64e740832b2a30dae24`としてcommit。
- [x] clean ahead-6/behind-0 post-commit state、unchanged stashをverifyし、
  B4A implementationをfresh inventory。

## Checker Task 258B4A implementation ledger

- [x] frozen runner 5/checker 3 consumersだけをimplement。
- [x] 全80 bytes、26 Surface rows/root 25、resolver provenance、lower
  profiles/owned sites、upper `1/1/1/0/1`をauthenticate。
- [x] exact mutation、coherent-near-miss、route-isolation、family-order、
  rollback、clone、semantic-empty coverageを持つrunner 5/checker 4 testsを
  追加。
- [x] test-sufficiency/implementation reviewsを**NO FINDINGS**でcomplete。
- [x] focused runner `5/5` / checker `4/4`をPASSし、libraries
  `476/414`、runner production 30 paths/55,109 linesをmeasure。
- [x] corpus/expectation/sidecar/trace/active/public-runner/semantic no-op
  boundariesをpreserve。
- [x] Low `design_drift` 3件のcorrection後、final
  source/documentation/bilingual consistencyを**NO FINDINGS**でcomplete。
- [x] package/workspace/Clippy/fmt/CLI/count/hash/stash gatesをPASS。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`でcomplete。
- [x] exact B4A scopeをstageし、cached diffをinspect。
- [x] dedicated B4A implementation commit
  `662adbde71e665ab37504ac476e94c935c493535`をcreate。
- [x] clean ahead-7/behind-0 post-commit state、unchanged stashをverifyし、
  B4Bをfresh inventory。

## Checker Task 258B4B documentation ledger

- [x] private 167-byte/double-LF source/hash、124 nodes/root 123、exact
  local theorem resolver provenanceをfreeze。
- [x] lower Task-252/256/257/B2/binding profiles、rootless arena、42/1/81
  ownership、upper `1/1/1/0/1` `Composite(0)` linksをfreeze。
- [x] active 166-byte fixtureをlower-onlyとしてpreserveし、全
  corpus/expectation/sidecar/trace/active-route editをforbid。
- [x] exactly runner consumers 4件、runner tests 5件、complete
  mutation/isolation/replay/final matrices、semantic/coverage deferralsを
  freeze。
- [x] baseline runner `476`、projection `481`、production `30/55109`、
  unchanged manifests/CLIs、narrative audit impact、exit gatesをrecord。
- [x] repeated documentation reviewを**NO FINDINGS**でcomplete。
- [x] docs-only verificationと全no-op/count/hash/stash gatesをPASS。
- [x] independent final qualityを全hard gates、score `>=90/100`でPASS。
- [x] dedicated B4B documentation commitをstage/inspectし、
  `b8a7b8257a682f7c88de943ceaa35b67c0585bc4`としてcreate。
- [x] clean ahead-8/behind-0 post-commit stateとunchanged stash
  fingerprintをverifyし、B4B implementationをfresh
  inventory。

## Checker Task 258B4B implementation ledger

- [x] frozen runner 4/checker 3、合計7 consumersだけをimplement。
- [x] private 167-byte/double-LF source、124 Surface nodes/root 123、
  raw label-free resolverからenriched `1/1/1/1/0`へのtransitionを
  authenticate。
- [x] Task-257B2 lower profiles、rootless 124-node arena、
  `42/1/81` ownership、upper `1/1/1/0/1`と両`Composite(0)`をpublish。
- [x] B1/B4A対B2/B4B pairingとprivate telemetry `0/0/[]`を
  profile-aware guardでisolateし、B4A `1/1/[1,1]`をpreserve。
- [x] exact runner `5/5` / checker `4/4` testsとactive 166-byte
  lower-only negativeをPASS。
- [x] separate test-sufficiency/implementation reviewsを
  **NO FINDINGS**でcomplete。
- [x] public runner API、semantics、corpus、expectation、sidecar、traceを
  no-opのまま保持。
- [x] final source/documentation、bilingual、boundary consistency reviewを
  **NO FINDINGS**でclose。
- [x] focused/package、full workspace、fmt、Clippy、5 CLI、
  count/hash/scope/audit-no-op/stash verificationをPASS。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  score capなし、valid `100/100`
  （`20/20/15/15/10/10/5/5`）でclose。
- [x] exact B4B implementation scopeをstage/inspectし、dedicated commit
  `752c17ae7d552d5268d1028612b8174e480b6f3e`をcreate。
- [x] clean ahead-1/behind-0 post-commit state、unchanged stashをverifyし、
  B4Cをfresh inventory。

## Checker Task 258B4C documentation ledger

- [x] private 139-byte/two-LF source hash
  `36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1`
  をfreezeし、active 138-byte lower-only hash
  `cbfd7077713e8e9630900e349d5f579251c19fba55434acb62170ea1dd940237`
  をpreserve。
- [x] Surface `66/root65`、theorem `62` `19..137`、label token `6`
  `27..65`、outer formula `60` `67..136`、raw resolver `1/0/1/1/0`、
  owner path `[2,1]`、contribution `0` anchor `0..18`、enriched resolver
  `1/1/1/1/0`をfreeze。
- [x] mandatory independent lower prerequisiteを`source_formula.rs`と
  runner `source_formula_composition.rs` test leafだけに限定し、exact
  138/139-byte one-/two-LF variantsをaccept、zero/triple LFをreject、
  production `source_formula_composition.rs`をunchangedとしてfreeze。
- [x] lower prerequisite test countをfresh-inventory-measuredとし、
  independent review、verification、quality gate、commit、post-commit
  inventoryをrequire。
- [x] lower profiles binding `4/4/0`、primary `6/6/0`、atomic
  `3/0/0/0/0/0/0/6/6`、composite `3/0/1/3/3/2/6`、
  composition `3/6`をfreeze。
- [x] upper `1/1/1/0/1`、context visible `[0]`、input facts 0件、両
  `Composite(0)`、ownership `24/1/41`、telemetry
  `2/2/[2,2,4,4,4,4]`をfreeze。
- [x] B4Bと同じeventual upper consumers 7件をfreezeし、complete exact/
  mutation/isolation/order/replay/final-empty coverage付きfocused checker
  4/runner 5 testsをproject。
- [x] existing spec、fixture、expectation、sidecar、trace status/count、
  active route、public schema、semantic/proof output、coverage-audit
  statusをpreserveし、audit impactをnarrative only、B5をdeferred。
- [x] baseline checker/runner `418/481`、checker production
  `23/140821`、runner production `30/56007`、exact production/test-list/
  five CLI hashes、unchanged stashをrecord。
- [x] repeated documentation reviewを**NO FINDINGS**でcomplete。
- [x] docs-only verificationと全no-op/count/hash/stash gatesをPASS。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）でPASS。
- [x] dedicated B4C documentation commitをstage/inspectし、
  `3c723316ae632a867d29e8f4fc36348be30df202`としてcreate。
- [x] clean post-commit/stash invariantsをverifyし、mandatory lower-stage
  prerequisiteをfresh inventory。

## Task 257B3 private double-LF selector prerequisite ledger

- [x] prerequisiteを`runner/type_elaboration/source_formula.rs`とrunner
  `source_formula_composition` test ownerだけに限定。
- [x] exact active 138-byte/private 139-byte identitiesだけをadmitし、
  zero/triple LFとsource/AST identity spoofingをreject。
- [x] identical Task-257B3 lower tables/fingerprints、production
  `source_formula_composition.rs`、active CLI/trace behavior、全upper
  ownerをpreserve。
- [x] independent reviews、focused/broad verification、final qualityを
  **NO FINDINGS**、全hard gates PASSでcomplete。
- [x] dedicated lower-stage prerequisiteをstage/inspectし、
  `42356f38ed0e679d7b878caf0e647c6aa8148d82`としてcommit。
- [x] clean post-commit/stash invariantsをverifyし、B4C implementationを
  fresh inventory。

## Checker Task 258B4C implementation ledger

- [x] exact runner 4/checker 3 filesだけをchangeし、全lower production
  owner、fixture、sidecar、expectation、trace row、specificationを
  unchangedで保持。
- [x] private 139-byte route、exact Surface/raw/enriched resolver profiles、
  Task-257B3 lower transaction、rootless 66-node `24/1/41` arena、upper
  `1/1/1/0/1`と両`Composite(0)`をauthenticate。
- [x] B1/A対B2/B対B3/Cをexactにpairし、active 138-byte routeを
  lower-onlyとして保持し、telemetry `2/2/[2,2,4,4,4,4]`をpublish。
- [x] focused runner `5/5` / checker `4/4` testsをPASS。
- [x] test-sufficiency/implementation reviewsを**NO FINDINGS**で
  complete。
- [x] runner library `488`、production `30/56872`、checker library
  `422`、checker production `23/141952`、exact production/test-list
  hashesをmeasure。
- [x] public schemas、active behavior、semantics、corpus、expectation、
  sidecar、trace status/count/backlinks、specificationをpreserve。
- [x] Medium `design_drift` 1件をcorrect後、final
  source/documentation、bilingual、boundary consistency reviewsを
  **NO FINDINGS**でcomplete。
- [x] complete crate/workspace、fmt、Clippy、CLI、count/hash/scope/stash
  verificationをrunし、全frozen count/hashをreproduce。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）でcomplete。
- [x] exact B4C implementation scopeだけをstage/inspectし、
  `50ab1ebc747e912fff1f0cf111832e3c2c81ba01`としてcommit。
- [x] clean post-commit state、unchanged protected stashをverifyし、次の
  dependency-ready logical taskをfresh inventory。

## Checker Task 258B5A frozen-contract documentation prerequisite

- [x] exact source/hash、Surface/resolver/lower/base/reference rows、
  proof-label `[0]` -> descendant-citation `[0,1]`、20/73 ownershipをfreeze。
- [x] private route telemetry `1/1/[1,1,1,1,1,1,1,1,1,1]`、runner five
  tests、exact consumers、atomic B1/B5A pairingをfreeze。
- [x] B5B import、B5C active negative、public/corpus/trace change、全
  semantic outputをexclude。
- [x] independent specification、test-sufficiency、source/documentation
  boundary、bilingual reviewsを**NO FINDINGS**でcomplete。
- [x] crate/workspace、fmt、Clippy、CLI、exact scope/count/hash、authority
  no-op、repository-state、stash gatesをreproduce。
- [x] repeated independent final qualityを**NO FINDINGS**、全9 hard gates
  PASS、capなし、valid `100/100`
  （`20/20/15/15/10/10/5/5`）でcomplete。
- [x] synchronized documentationだけを
  `59021f764f146d669f84877042f0512882c9c5ff`としてcommitし、
  post-commit invariantsをverifyしてB5A implementationをfresh inventory。

## Checker Task 258B5A implementation ledger

- [x] exact four runner/three checker consumersを変更し、全parser/resolver/
  lower production ownerとpublic harness schemaをunchangedに維持。
- [x] 185-byte source、exact Surface/raw/enriched resolver profile、lower
  handoff、Task-258 base/reference row、`20/73` ownership、label `[0]`、
  citation `[0,1]`、resolver node 82をauthenticate。
- [x] exact telemetry `1/1/[1,1,1,1,1,1,1,1,1,1]`、B1/B5A atomic
  installation、selector isolation、replay、clone、empty semanticsをpreserve。
- [x] B5B/B5C、active fixture、specification、expectation、sidecar、
  trace status/count/backlink/credit、public result、diagnostic、semantic
  outputをunchangedに維持。
- [x] frozen focused runner `5/5`、checker `4/4`、preserved B1 runner
  `6/6` testsをrun。
- [x] separate test-sufficiency/implementation reviewを**NO FINDINGS**で
  complete。
- [x] final source-documentation consistency reviewを**NO FINDINGS**で
  complete。
- [x] checker `426/426`、runner `493/493`、full workspace test、formatting、
  exact Clippy、five CLI、count/hash、diff gateをPASS。
- [x] final scope/forbidden-artifact、repository-state、stash gateをcomplete。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）でcomplete。
- [x] B5Aだけをstage/inspectしimplementation commit
  `4a79116c1a6f71155e4f366950fee8335b4dc8f1`を作り、post-commit
  invariantをverifyしてnext dependency-ready taskをfresh inventory。

## Checker Task 258B5B frozen-contract documentation prerequisite

- [x] B5A commit `4a79116c1a6f71155e4f366950fee8335b4dc8f1`がhistorical
  pending lineをsupersedeするとrecordし、B5Bをfresh inventory。
- [x] unfrozen API=`design_drift`、opt-in imported-label=`source_drift`、
  active coverage=`test_gap`とclassifyし、blocking gapなしを確認。
- [x] 146-byte source/hash、57-node frontend/resolver、raw/opt-in env、
  lower profiles、Task-258 `1/2/2/2/2 + 0/1`、`8/49`をfreeze。
- [x] two-file lower task/2 tests、exact `Ref` provenance、citation API、
  telemetry、7 consumers、checker 4/runner 5 tests、B1/B5A debugをfreeze。
- [x] authority/test/trace/public/semantic no-opとB5C deferralをpreserve。
- [x] specification reviewをblocking findingなしでcompleteし、crate/
  workspace、format、Clippy、five CLIをPASS。
- [x] test-contract、source/documentation、bilingual reviewを
  **NO FINDINGS**でcomplete。
- [x] final scope/repository/stash gateをcomplete。
- [x] final qualityを**NO FINDINGS**、全9 hard gates PASS、capなし、valid
  `100/100`（`20/20/15/15/10/10/5/5`）でcomplete。
- [x] synchronized docsだけを`141dc44a`としてcommitし、mandatory lower
  taskをfresh inventory。

## Checker Task 258B5B lower-stage prerequisite

- [x] `runner/import_fixtures.rs`とstatement test leafだけを変更し、opt-in
  `Ref` helperと2 testsをseparate commit `46dd9db5`として完成後、upperを
  fresh inventory。

## Checker Task 258B5B upper implementation

- [x] frozen checker 3/runner 4 Rust consumersだけを変更し、同logical taskの
  synchronized design outputがcode boundaryをexpandしないことを維持。
- [x] exact source 146 bytes、57 nodes/root 56、raw/enriched resolver
  `1/0/1/1/0`/`8/1/1/3/1`、Binding `2/1/0`、Task-252 `4/4/0`、
  Task-256 two formulas/four edges/four requests、Task-258
  `1/2/2/2/2 + 0/1`、`8/49`をauthenticate。
- [x] exact-source opt-inだけでimported target/kindとexact import/
  projection/reference provenanceをinstallし、B1/B5A debug bytes、pairing、
  replay、empty semanticsをpreserve。
- [x] focused checker `4/4`、runner `7/7`（upper five/lower two）、full
  checker `430/430`、runner `500/500`をPASSし、current production/
  test-list/CLI hash、owner countをrecord。
- [x] test-sufficiency/repeated implementation reviewを**NO FINDINGS**で
  completeし、hard-gate documentation mismatchを`design_drift`として同期。
- [x] spec-coverage変更をnarrative onlyとし、requirement `tests = []`、
  trace status/count/backlink/owner/credit、corpus、expectation、sidecar、
  public runner schema、B5C、semantic deferralをpreserve。
- [x] final source/documentation consistencyを**NO FINDINGS**でcompleteし、
  workspace formatting、exact Clippy、full tests、five CLIs、final
  count/hash/scope gateをPASS。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）でcomplete。
- [x] B5B upper logical taskだけをstage/inspectし、implementation commit
  `f27d2c9169b08078f00b75c4a57f94e30fa28f59`を作り、clean
  post-commit/stash invariantをverifyしてnext taskをfresh inventory。

## Checker Task 258B5C frozen-contract documentation prerequisite

- [x] clean post-B5B inventoryからB5Cをselectし、Chapter 15 §15.10と
  Chapter 16 §§16.4.2/16.5.1をlabel-scope authority、Chapter 11 §11.2を
  contextual onlyとしてfreeze。
- [x] normal-source proof-label projection path欠如をpotential
  `boundary_violation`を伴うMedium `source_drift`、stale derived ownershipを
  `design_drift`、active confinement case欠如を`test_gap`、public resolver
  code未規定をLow deferred/nonblocking `spec_gap`とclassify。
- [x] exact 173-byte inner-to-outer/197-byte sibling source、hash、normal
  frontend identity、proof scope、statement ordinal、declaration/reference
  node/range、expected unresolved resolver outcomeをfreeze。
- [x] structurally validated Surface-to-resolved providerをarchitecture
  authority十分なknown resolver prerequisiteとしてrecord。
- [x] dependency orderをdocumentation-only commit、resolver R-032A
  structural arena/map、resolver R-032B proof-label source collector、
  active B5C declaration-symbol fixture/sidecar/trace/runner commitとして
  freeze。
- [x] 両exact `Result`-returning API/error、narrow R-032B
  inclusion/exclusion、theorem-root path、completion visibility ordinal 3、
  exact label/semantic origin、lower positive/negative/provenance testをfreeze。
- [x] same-`'a` ast/resolved storage、validation-only module、owned
  namespace/contribution、`Self` return、exact `SurfaceNodeId` error payload、
  global one-based ordinal、`ConclusionStatement`/reference chain、
  canonical `proof-step-v1` identityをfreeze。
- [x] active runnerにはvalidated resolver-owned projection/candidateの
  consumeを要求し、checker installationとrunner-fabricated id/semantic
  proof scope/ordinal/originを禁止。
- [x] source-byte+normal-AST-only selection、shared env/moduleとexact
  local-source contribution-0 authentication、separate private
  input/confinement detail、expectation-copy guard、exact 48-file docs
  scopeをfreeze。
- [x] future fail fixture 2件、sidecar stage/domain/phase/category、empty
  public diagnostic codes、private detail key、trace requirement ID 2件、
  projected count change、exact consumers/tests/exclusion/audit impact/exit
  criteriaをfreeze。
- [x] repeated specification、test-contract、source/documentation、
  bilingual、final-quality reviewをno findings、全hard gates PASS、valid
  quality `90/100`以上でcomplete。
- [x] production/corpus/expectation/sidecar/trace status/countを変更せず、
  current crate/workspace、format、Clippy、five CLI、count/hash/scope、
  authority no-op、repository-state、protected-stash gateをreproduce。
- [x] synchronized B5C frozen-contract documentationだけをstage/commitし、
  clean post-commit/stash invariantをverifyしてresolver R-032Aをfresh
  inventory。
- [x] resolver exact
  `Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
  ProofBlock` upper chain、exact-one normal upper child、direct-normal theorem
  scan、remaining default-deny no-ordinal/no-descent form、positive-edge/
  missing/additional/wrong/direct-relocation/`VisibleItem`/mixed-list testを
  freeze。
- [x] environment module、projection module/namespace/contribution、
  contribution zero/multiple cardinality、id、全non-local kind、record
  module、LocalSource source idのindependent mutationをfreezeし、すべて
  `proof_scope_input`だけへmap。
- [x] source-bytes-plus-normal-AST selection、expectation non-selection、
  empty public code、exact 48-file scopeをunchangedに維持。

## B5C R-032A preflight overlay

- [x] separate mizar-syntax S-026 frozen-documentation commit を完了。
- [x] separate S-026 implementation と review/verification gate を完了。
- [x] active B5C artifact追加前にdedicated S-026、R-032A、R-032B
  prerequisite commitを完了。R-032Bは
  `b3a7e79a6b60db2974e911c69bb56ff5f4609064`。
- [x] exact source-only selector、provenance authentication、private detail、
  empty public code、projected active count impact を維持。

## Checker Task 258B5C active implementation

- [x] frozen fail fixture/sidecar 2組とcovered trace row 2件だけを追加し、
  `421/389`、`228/193`、`101/7/198/1`、`23/0`をreproduce。
- [x] unchanged R-032A/R-032B API上のprivate declaration-symbol consumerだけを
  implementし、runner-derived semantic identityを追加しない。
- [x] exact dense profile、全frozen provenance/result corruption、
  expectation non-selection、replay/order、existing-case isolationをcover。
- [x] omitted metadata count consumerのexact four `5 -> 7` assertionを修正し、
  test intentを変えず`test_expectation_drift`とwrite-scope
  `design_drift`に分類。
- [x] findings-free test/implementation/source-documentation reviewと
  focused/crate/workspace/count/hash verification gateを完了。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  score capなし、valid `100/100`
  （`20/20/15/15/10/10/5/5`）で完了。
- [ ] task fileだけをstageしdedicated commitを作成、repository/stash
  invariantをverifyして次のexecutable taskをfresh-inventory。

## Checker Task 259 Frozen Consumer Prerequisite

- [x] Task 259選択前にB5C commit
  `33ac57e96f048dc40559565f54369cac854409a7`とsuccessful post-commit
  invariantを記録する。
- [x] future exact pass fixture/sidecar name、source bytes/hash、
  pass/type-check/empty-diagnostic expectation、one-row trace intentを凍結。
- [x] lower build order/exact profileを凍結: extended Task 248、Task 249
  `2/2/0`、Task 252 `4/4/0`、Task 256 `2/0/0/0/0/0/0/4/4`、Task 259
  `1/2/1/1/1` plus one pending obligation。
- [x] exact-source/normal-AST selection、resolver predicate authentication、
  lower-handoff corruption、all-or-nothing install、deterministic
  replay/order、expectation non-selection、family isolation testを凍結。
- [x] mixed predicate-plus-functor fixture/sidecar/trace row、generic gap
  detail、全Task-260 ownershipをunchangedに保つ。
- [x] このprerequisiteをdocumentation-onlyの`421/389`、`228/193`、
  `101/7/198/1`、type `253/241`、warnings/errors `23/0`に保つ。
- [x] 4つすべてのfindings-free review、full verificationとcount/hash
  reproduction、全9 hard gate PASSのindependent final quality
  `100/100`をcompleteする。
- [x] task-only documentation commit
  `d5294b8f4be46a420bbdfa2fc4062384be983ce0`とpost-commit fresh
  inventoryをcompleteする。
- [x] separate Task-248 extension documentation prerequisiteをfresh-inventory
  する。

## Checker Task 248 Two-Parameter Runner Prerequisite

- [x] future Task-259 sourceからexact shell/direct-parameter/range/type/
  source-order extractionをfreezeし、active routeをselectしない。
- [x] caller-owned site plus shared-arena anchor/context validationをfreezeし、
  existing Task-248 projectionだけを返す。
- [x] Profile Aをpreserveし、全guard/predicate/property/justification
  descendantとTask 249+/259 semanticsをexcludeする。
- [x] five-file Rust scope、four-test matrix、runner `504 -> 508`
  projection、no fixture/sidecar/trace/count deltaをfreezeする。
- [x] findings-free reviewとfull docs-only focused/crate/workspace/count/hash
  verificationをcompleteする。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gate PASS、
  score capなし、有効な `100/100`
  (`20/20/15/15/10/10/5/5`)でcompleteする。
- [x] dedicated documentation commit
  `f9b47375acc18acebf56a69f5d8a7edec539c2be`とclean post-commit
  inventoryをcompleteする。
- [x] separate Task-248 extension commit
  `ca54135f36c9fecfc02c2b8120ec4e63e8c6ca36`をimplementし、Checker
  Task-259 consumerへ戻る。

## Checker Task 259 Frozen-Consumer Correction Prerequisite

- [x] completed Task-248 Profile Bとcurrent runner `508` baselineを記録する。
- [x] new private route leaf、parent facade/re-export、test include/leaf、
  mechanical metadata assertion 4件、pass fixture/sidecar 1件、trace row
  1件をexact future consumerとしてfreezeする。
- [x] `BindingEnv`、全lower handoff、Task-259 tableをchecker-ownedに保ち、
  raw AST/sibling/subtree authenticationをrunner privateに保つ。
- [x] specification/consumer reviewをfindingsなしまでrepeatし、全executable
  artifact byte-unchangedでdocs-only verificationをcompleteする。
- [x] correction documentだけを
  `e202dd70bf4e97ddb53c1275b49e667b6a77f7a0`としてcommitし、
  clean/stash-invariant stateをverifyし、Task-259 implementationを
  fresh-inventoryする。

## Checker Task 259 active consumer implementation

- [x] exact 165-byte pass fixture/sidecarとsole covered trace backlinkを追加し、
  existing mixed Task-260 fixture familyは変更しない。
- [x] exact lower profileを持つprivate Task 248 -> 249 -> 252 -> 256 ->
  259 routeと、final `1/2/1/1/1` handoff plus one `Pending`
  `PredicatePropertyCorrectness` obligationをimplementする。
- [x] frozen runner test 4件を追加し、raw AST、sibling、resolver、
  subtree exclusion、exact-source authenticationをrunner-privateに保つ。
- [x] stale source-statement active-type count consumer 2件だけを
  `198`から`199`へindependently review/updateし、empty selection assertionを
  preserveして他のmechanical metadata assertion 2件もalignする。
- [x] executable metadata `422/390`、`229/193`、`101/7/199/1`、type
  `254/242`、warnings/errors `23/0`、metadata `137`、checker `435`、
  runner `512`、resolver `144`、syntax `59`をreproduceする。
- [x] final independent test-sufficiency、implementation、
  source/documentation-consistency reviewを全件no findingsで完了し、全9 hard
  gateをPASS、score capなしのfinal quality score `100/100`を得る。
- [x] affected production/test-support inventoryをfresh final measurementし、checker
  `24/147030`、runner `31/63248`、checker producer/support `1794/1974`、
  runner production/test leaf `1233/517`をrecordする。
- [x] Task-259 fileだけをstageしimplementation commit
  `b61be7e567b92d31b3544b86e5c7a68537625743`を作成、repository/stash
  invariantをverifyしてTask 260をfresh inventoryする。

## Checker Task 260 Frozen Consumer Prerequisite

- [x] active consumer edit前にexact source/hash/AST/resolverとlower
  Task-248/249/252/256 bundleをfreezeする。
- [x] `2/2/1/2/2`、Pending obligation 2件、exact selection、lower corruption、
  atomic install、replay、Task-259/mixed isolationをfreezeする。
- [x] future pass pair/trace row 1、runner test 4、mechanical active-count assertion
  6、count projection、empty semantic outputをfreezeする。
- [x] prerequisite中current fixture/sidecar/expectation/trace row/status/
  production/Cargoをbyte-unchangedに保つ。
- [x] all four reviewをno findingsまでrepeatし、docs-only verificationを
  all nine gates PASS、score capなしquality `100/100`でcompleteする。
- [x] exact staging、documentation commit
  `b587038f12f84a77720f6441a000ddb84c7b996f`、post-commit gateをcompleteする。
- [x] Task 249Rをchecker-onlyとしてrecordし、runner source/library count、
  fixture/sidecar/trace、corpus/metadata/CLI countを不変に保ち、impossible lower
  `4/4/0`を`2/4/0/2`へcorrectする。
- [x] separate Task-249R documentation/implementation commitがgateをpassした後に
  frozen Task-260 consumerをresumeする。implementation commitは
  `c233bfdff8317a1f4ffdd5750e62a29ee6e69b2f`。

## Checker Task 260 active consumer

- [x] private exact-source/resolver/lower routeを追加し、raw syntax/resolver
  mutation ownershipを`mizar-test`内部に保つ。
- [x] literal 108-row Surface oracleとindependent environment/projection/symbol/
  definition/contribution plus every-lower-association corruptionを含むfrozen
  runner test exactly 4件を追加する。
- [x] pass pair 1組/sole covered trace backlinkを追加し、six mechanical active-type
  assertionを`200`へ更新し、existing expectationをpreserveする。
- [x] runner `516`、metadata `137`、`423/391`、`230/193`、
  `101/7/200/1`、type `255/243`、warnings/errors `23/0`、exact CLI/
  test-list/production hashを再現する。
- [x] proof/fact/acceptance/VC payloadをpublishせずrepeated test-sufficiency/full
  implementation reviewを**NO FINDINGS**で終了する。
- [x] source/docs consistencyを**NO FINDINGS**で完了し、full shared
  verification matrixをPASSする。
- [x] final hard gate 9件をquality `100/100`、score capなしでPASSする。
- [x] shared Task-260 staging/commit/post-commit gateを
  `c83e424a485a24dd0f00ddea687903a235d85850`で完了する。

## Checker Task 261 Frozen Attribute-Definition Consumer

- [x] implementation edit前にexact 116-byte source/hash、45-row Surface
  oracle、resolver provenance、lower Task-248/249/252/256 association、
  Task-259/260 isolationをfreezeする。
- [x] private runner/public checker ownership splitとchecker table cardinality
  `1/2/1/1`をfreezeし、ordinary initial obligationを追加しない。
- [x] future pass pair/trace row exactly 1件、checker test 5件、runner test 4件、
  projected count、再計測するhash、exclusion、exit gateをfreezeする。
- [x] 本documentation prerequisiteではproduction、fixture、sidecar、
  expectation、trace row/status/count、Cargo metadataをbyte-unchangedに保つ。
- [x] findings-free review、全9 documentation gate、exact staging、
  prerequisite `209c32fc2ec547ceedd32f1052345ae2fc5b0451`、clean
  post-commit inventoryを完了する。
- [x] fresh-inventoryしてfrozen Task 261 source/runner/fixture/trace/count
  scopeだけをimplementし、focused/repeated test/implementation reviewを
  **NO FINDINGS**で閉じる。
- [x] source/documentation consistencyを**NO FINDINGS**で完了し、exact
  count/hash reproductionを含むfull verificationをPASSする。
- [x] final hard gate 9件を**NO FINDINGS**、score capなし、quality
  `100/100`でPASSする。
- [x] exact stage/commit/post-commitを
  `b1782bfc06388410229f07ee193a5febe0bf525e`として完了し、Task 262をselectする。

## Checker Task 262 frozen mode-definition consumer

- [x] exact 141-byte source/hash、54-row Surface oracle、two-shell resolver
  identity、lower Task-248/249 association、sibling isolationをfreezeする。
- [x] private runner/public checker boundary、`1/2/1/1/1/1` table 6個、
  unresolved RHS request、semantic acceptanceなしのpending existing-kind
  `Sethood` row 1個をfreezeする。
- [x] future pass pair/sole trace row 1個、checker 5/runner 4 test、projected
  count、remeasure対象hash、exclusion、exit gateをfreezeする。
- [x] documentation prerequisiteでproduction、fixture、sidecar、expectation、
  trace row/status/count、Cargo metadataをunchangedに保つ。
- [x] findings-free review、全9 docs gateをscore capなし`100/100`、exact
  staging、prerequisite commit
  `8c3fa20acef42477d38a66ddddec42dacced0863`、clean post-commit inventoryで
  完了する。
- [x] mandatory checker Task 249Mをrunner/corpus changeなしでfresh-inventory/
  freezeする。
- [x] Task-249M docsをreview/separate commit後、checker test 4件をimplementし、
  checker review/verification/separate commitを完了する。
- [x] Task 262だけへ戻り、exact consumer、test 4件、pass pair、reciprocal trace
  row、active count、measured hashを実装した。
- [x] repeated reviewを**NO FINDINGS**で完了し、full verification、全9 hard
  gateをscore capなしのquality `100/100`で通過してexact Task-262
  commit-readyとする。その後mixed-gap semanticsを広げずfresh-inventory
  Task 263+へ進む。

## Checker Task 263 preflight resolver gate

- [x] exact 320-byte future sourceをChapter-5-derived test intentとしてfreezeし、
  lower prerequisiteではcorpusへ追加しない。
- [x] Task 263Rで全runner route、fixture、sidecar、expectation、trace row/
  status/count、active case、metadata assertion、CLI hashを不変にする。
- [x] separate Task-263R documentation commitとlower implementation fresh
  inventoryを`mizar-test`/corpus deltaなしで完了。
- [x] implemented resolver two-file correctionがresolver test inventory
  `144 -> 146`だけを変更し、exact probe/reviewをPASSすることを確認。
- [x] 全full/final gateを**NO FINDINGS**、全9 hard gate PASS、score capなし、
  valid `100/100`で完了。
- [x] implementation commit
  `997457dd3189030aa3b137b568ce82fed456fe1e`後にTask-263 boundaryをfresh
  inventoryし、direct consumer freezeに先立つremaining lower prerequisiteをchecker
  Task 249Sへ更新する。

## Checker Task 249S no-runner prerequisite

- [x] exact checker-only `0/4/0/0/0/4` member-type intake/checker local test
  4件をfreezeし、runner/corpus/trace/metadata/CLI artifactを変更しない。
- [x] separate implementationのno-op boundaryを保ち、その後Task 263
  だけがexact private consumer/pass+trace pairを追加する。

## Checker Task 263 frozen runner consumer

- [x] exact source/hash、75 Surface rows、resolver `10/8/8/8/0`、lower
  `0/4/0/0/0/4` fingerprint、subtree exclusionsをfreezeする。
- [x] private pre-gap route 1件、exact runner tests 4件、canonical-derived pass
  pair 1件、reciprocal covered trace row 1件をassignする。
- [x] transport-only credit、existing mixed-gap byte stability、projected runner/
  metadata/coverage counts、hash remeasurement、scope、semantic non-publicationを
  freezeする。
- [x] docs-only commit readinessまでno-op boundaryを保つ。repeated reviewsは
  **NO FINDINGS**、all nine hard gatesはuncapped `100/100`でPASS。
- [x] docs commit後のfresh inventoryからexact route/tests/pass/traceだけを実装し、
  active audit/count/hashを同期する。
- [x] 全review/verificationを**NO FINDINGS**、全9 hard gatesをscore capなしの
  `100/100`でPASSし、Checker Task 263 implementation commit
  `f11a517e91433b461447522eff06cd85e6187063`とclean fresh inventoryを完了する。

## Checker Task 264R no-runner prerequisite

- [x] runner/corpus/sidecar/expectation/trace/metadata/CLI影響ゼロをfreeze。
- [x] Parser Task 48 fixture 2件をread-only lower probeとし、inactive coherence seedを
  byte-identical/inactiveに保存。
- [x] docs/lower implementationを通じpost-Task-263 count/hashを再確認し、Checker Task 264前に
  runner consumerを追加しない。dedicated implementation commit
  `db8c39e31678d6b8a1f0900a5368c3b95c7162b5`とclean post-commit inventoryは完了。

## Checker Task 248P no-runner prerequisite

- [x] runner/corpus/sidecar/expectation/trace/metadata/CLI impact zeroをfreezeし、runner
  `528`、production `35/67939`、全hashを保存。
- [x] Profile-C runner helperを追加せずchecker-only docs commit
  `1e3fa789ce335b900fca4ac6ef5ad56b40cb5f24`とimplementationを完了。
- [x] Task 264 bounded consumerだけをfresh-inventory/freezeする。
- [x] Task264 means/equals texts/hashes、85/56 AST、resolver/lower order、two
  sidecars、one requirement、four runner tests、counts/isolation/no semantic
  publicationをfreezeする。
- [x] Docsではrunner/corpus/trace/count/hash zero changeを保存し、consumer前に
  Task249PIをselectする。
- [x] Task264 docs reviewを**NO FINDINGS**、all nine gates PASS、score capなし
  `100/100`でcompleteし、runner pathを追加しない。
- [x] Exact synchronized docsだけを
  `4c3f74b053d31cae45b8af3fc478498b4a112768`としてcommitし、Task249PIをfresh
  inventoryする。
- [ ] Task249PI後frozen routeだけをimplement/review/verify/commitする。

## Checker Task 249PI no-runner prerequisite

- [x] checker-only exact `1/3/0/0/0/2`、test 4件、checker `469 -> 473`、
  runner/corpus/trace/metadata/CLI zero impactを記録する。
- [x] docs/implementation中current runner route/fixture/sidecar/expectation/trace
  row/status/count/hashを全てpreserveする。
- [ ] `mizar-test` source/artifact deltaなしでTask249PI docs/implementationをcommitする。
- [ ] fresh inventory後already-frozen Task264 routeだけをimplementする。
