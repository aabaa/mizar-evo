# mizar-test TODO

> 2026-09-02 圧縮（batch CPT-10、規則は
> [../../documentation_compaction_rules.md](../../documentation_compaction_rules.md)）:
> ステータス文書の言語方針（2026-09-01 承認）に基づき、完了タスク本文の正本は
> 英語版および英語アーカイブ
> [../../archive/test_todo_sections.md](../../archive/test_todo_sections.md)
> に一本化した。以下には全見出し・登録済み redirect 行・未完了作業のみ残る。
> 各タスクの詳細の正本は [../../task_contracts/ja/](../../task_contracts/ja/)
> 配下の対応契約文書。

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
2. **ソース/仕様ギャップ監査と状態の同期。** [x]
3. **ランナーモードと CLI の完成。** [x]

### snapshot 対応

4. **snapshot モジュール: API と正準化。** [x]
5. **snapshot の更新ポリシーと決定性チェック。** [x]

### カバレッジと健全性の契約

6. **カバレッジと pass/fail 比率の報告。** [x]
7. **stage 前提条件の検証。** [x]
8. **fail/健全性契約の対応。** [x]
9. **コーパスサイズとレビュー規則の検証。** [x]

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
      [Step 5C.1](../../task_contracts/ja/STEP5C1-VARIABLE-SEMANTICS.md)はexact
      audit-1 variable 12 pairとfirst runner incrementだけを所有する。上記のdistinct
      reserved-variable/capture-shadowing/omitted-`reconsider` fixtureは
      `MT10-FS`/`MT10-AS`所有のまま。
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
12. **公開 enum の前方互換性ポリシー。** [x]
13. **二言語ドキュメント同期監査。** [x]
14. **増分/並列検証 regression matrix。** [x]
15. **architecture-22 フォローアップ監査。** [x]
16. **Source-derived builtin type-expression bridge。** [x]
17. **Source-derived builtin `ResolvedTypedAst` bridge。** [x]
18. **Source-derived reserve declaration semantic bridge。** [x]
19. **Reserve bridge core summary readiness and builtin declaration
    inventory。** [x]
20. **Reserve bridge core context readiness。** [x]

### kernel 健全性監査フォローアップ(2026-07-03)

kernel 受理境界の監査
([soundness_argument.md](../../mizar-kernel/en/soundness_argument.md))は
harness 所有の所見 F7 と F8 を報告した。以下は監査由来の最小限の追加で
あり、より広い runner 成長は引き続き task 10 のペース配分に従う。

21. **必須ケース registry への訂正後 soundness 語彙(kernel F7)。** [x]
22. **certificate corpus ルート命名の調停(kernel F8)。** [x]

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

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 242 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 243 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 244 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 245 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 246 Active Addendum

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Runner Module-Boundary Refactor Backlog

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## VC Task 30 / Task-10 consumer ownership

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## VC Task 31 / Task-10 consumer completion

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Resolver Task 31 / declaration-symbol completion

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Parser Task 47 / parse-only completion

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Parser Task 48 / property-implementation parse-only completion

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 248 / Task-10 consumer completion

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 249 frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 250 frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 251 frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 252 frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 253 frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 254 frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 255 frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 256 frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257A frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257B1 frozen consumer prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257B2 frozen runner checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257B3 frozen runner checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257C1 frozen runner checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 255C1 frozen runner checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257C2 frozen runner checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 256C1 frozen runner checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257C2 implementation checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257C3 frozen runner checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257C3 implementation checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258A frozen consumer checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B1 frozen consumer checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B2 frozen consumer checklist

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

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

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2A runner documentation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2A runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B1 runner prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B1 runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2A runner prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2A runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1P runner prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1P runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1A runner prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1A runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1B1P runner prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1B1P runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1B1 runner prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1B1 runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2P runner prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2P runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2A runner frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2A runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2BP runner frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2BP runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2B runner frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2B runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2CP runner frozen-prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2C runner frozen-contract ledger

Completion evidence: [central Task-258B3M2B2B2C historical contract](../../task_contracts/ja/258B3M2B2B2C.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3P runner frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3P runner implementation-closure ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3A runner frozen-contract ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3A runner implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3B runner ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3B implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3C documentation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3C implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3D documentation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

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

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3E implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B4A documentation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B4A implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B4B documentation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B4B implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B4C documentation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 257B3 private double-LF selector prerequisite ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B4C implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B5A frozen-contract documentation prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B5A implementation ledger

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B5B frozen-contract documentation prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B5B lower-stage prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B5B upper implementation

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 258B5C frozen-contract documentation prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## B5C R-032A preflight overlay

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

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

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 248 Two-Parameter Runner Prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 259 Frozen-Consumer Correction Prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 259 active consumer implementation

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 260 Frozen Consumer Prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 260 active consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 261 Frozen Attribute-Definition Consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 262 frozen mode-definition consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 263 preflight resolver gate

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 249S no-runner prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 263 frozen runner consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 264R no-runner prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 248P no-runner prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 249PI no-runner prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269A dormant consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269B dormant B3M1 increment

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269CP dormant proof-`let` lower projection

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269C dormant binding-only consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269CT dormant runner increment

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269GP dormant lower increment

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269GS canonical scope reconciliation

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269G dormant binding consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269GT dormant source-type consumer

Completion evidence: [central Task-269GT historical contract](../../task_contracts/ja/269GT.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269GUP dormant use-profile binding prerequisite

Completion evidence: [central Task-269GUP historical contract](../../task_contracts/ja/269GUP.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269GUPT dormant source-type consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269GU dormant term/reference consumer

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 269GCP Given-condition lower route

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 269GC Given-condition binding route

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 269GCT Given-condition source-type route

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/ja/269GCT.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 269GCU given-condition term/reference route

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/ja/269GCU.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 269SDP dormant runner handoff

- [x] exact lower source/Surface/shell/resolver/debugとzero-credit境界を凍結。
- [x] docs commit `f468b0163bb00726dca9b356f48790c73bb1fe98`後、private selector/facadeと4 testsだけを追加。
- [x] fixture/sidecar/trace/metadata/dispatch/active resultを不変に保つ。
- [x] runner library `592`、production `37/79025`、test-list hashを再現し、
  focused/crate/test/implementation reviewをPASS。
- [x] source/docs/full verification/final qualityを全9 hard gates PASS、
  score capなしの`100/100`で完了する。
- [ ] commit後、descendant context/bindingへhandoffする。

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/ja/269SDP.md#completion-evidence)。

## Checker Task 269SDC dormant consumer handoff

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/ja/269SDC.md#completion-evidence)。
本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 269SDT

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 269SDU Private Runner

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 277A Direct Parser-Origin Template Transport

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Resolver Task 277R1 test-only fixture probe

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 277B-L Private Association Probe

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 277C private structural composition probe

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Resolver Task 277R2 test-only fixture probe

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257C4A private binding-context probe

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257C4B private bound-use probe

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 257C4C0 inactive capture oracle

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 257C4C1 lexical-admission prerequisite

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Task 257C4C7 two-capture inactive oracle

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## Checker Task 257C4C8 normalized capture graph

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).

## 2026年9月 監査1 意味論ブリッジ・オラクルコーパス増分

本文は英語正本へ移管: [../en/todo.md](../en/todo.md) / [archive](../../archive/test_todo_sections.md).
