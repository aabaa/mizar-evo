# Source/Spec 対応監査: mizar-checker

> Canonical language: English:
> [../en/source_spec_audit.md](../en/source_spec_audit.md).
> 2026-09-02 圧縮（batch CPT-08、規則は
> [../../documentation_compaction_rules.md](../../documentation_compaction_rules.md)）:
> ステータス文書の言語方針（2026-09-01 承認）に基づき、タスク別監査
> セクション本文の正本は英語版および英語アーカイブ
> [../../archive/checker_source_spec_audit_sections.md](../../archive/checker_source_spec_audit_sections.md)
> に一本化した。以下には全 H2 見出しと登録済み redirect 行が残る。
> 各タスクの詳細の正本は [../../task_contracts/ja/](../../task_contracts/ja/)
> 配下の対応契約文書。

task 32 は task 31 後の checker public surface と仕様上の約束を監査する。
source behavior、`.miz` fixture、expectation、public API は変更しない。
未接続の挙動は現在の実装都合を正本化せず、明示的な
`external_dependency_gap`、`test_gap`、または `deferred` として分類する。

## Task 257C4A Fraenkel generator source-spec audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 257C4B Fraenkel generator bound-use classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 257C4C7 Two-capture Test Intent

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 257C4C8 Normalized Graph Completed Zero-Credit Mapping

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 33C Frozen Zero-Credit Graph-Owner Correspondence

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269SDU Implemented Zero-Credit Mapping

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## 範囲と方法

inventory は `crates/mizar-checker/src/lib.rs` の現在の `pub mod` export、
`crates/mizar-checker/src/*.rs` の top-level public item、そして crate-local
`dense_id!` / `string_key!` macro で生成される public newtype をすべて含む。
public method は、module spec が table、builder、output API として記述している
ため、所有する public type の下にまとめる。

監査対象の module specification:

- [typed_ast.md](./typed_ast.md)
- [binding_env.md](./binding_env.md)
- [source_context.md](./source_context.md)
- [source_atomic_formula.md](./source_atomic_formula.md)
- [source_attribute_definition.md](./source_attribute_definition.md)
- [source_functor_definition.md](./source_functor_definition.md)
- [source_property_implementation.md](./source_property_implementation.md)
- [source_mode_definition.md](./source_mode_definition.md)
- [source_predicate_definition.md](./source_predicate_definition.md)
- [source_structure_definition.md](./source_structure_definition.md)
- [source_attribute.md](./source_attribute.md)
- [source_evidence.md](./source_evidence.md)
- [source_application.md](./source_application.md)
- [source_set_term.md](./source_set_term.md)
- [source_structure.md](./source_structure.md)
- [source_statement.md](./source_statement.md)
- [source_proof_local_declaration.md](./source_proof_local_declaration.md)
- [source_template.md](./source_template.md)
- [source_term.md](./source_term.md)
- [source_type.md](./source_type.md)
- [type_checker.md](./type_checker.md)
- [registration_resolution.md](./registration_resolution.md)
- [cluster_trace.md](./cluster_trace.md)
- [overload_resolution.md](./overload_resolution.md)
- [resolved_typed_ast.md](./resolved_typed_ast.md)

implemented Task 258A contract
[source_statement.md](./source_statement.md)はbounded syntax-free theorem
transactionをownし、broader statement semanticsはdeferredのまま。

結果: 実装済み explicit-payload API について、blocking な `source_drift`、
`design_drift`、`source_undocumented_behavior` は観測していない。残る
source-derived behavior の未 coverage は、英語正本 crate plan の
[Known Gaps And Drift](../en/00.crate_plan.md#known-gaps-and-drift) が所有する。

Post-audit source-derived bridge note: `mizar-test` は対応済み `.miz` source から
explicit checker-owned `BindingEnv`、`DeclarationInput`、`TypedAst`、
`ResolvedTypedAst` payload を構築する bounded reserve-only declaration bridge を
実行するようになった。successful pass execution は bare builtin reserve head に限られ、
task 55 の same-module no-argument local mode head のうち、unique / preceding /
unrecovered な mode definition が bare builtin RHS を持ち definition-local context を
持たないもの、および dependency mode がその accepted bare builtin RHS expansion を持つ
task 56 の one-edge same-module local mode chain、builtin `set` / `object` で終端する
task 74 の AST-bounded structural bare local-mode chain も successful pass execution に含まれる。
active fail slice は same-module
attributed builtin head を missing-evidence diagnostic へ、task 55/56/57/58/59/60/61/62/63/64/65/66/74 外の same-module local
mode head（mixed attributed/bare local mode source、attributed chain
dependency、task 74 の structural guard を満たさない chain を含む）を missing mode-expansion diagnostic へ、same-module local structure head と attributed local
structure head を missing evidence-query diagnostic へ運ぶ。task 57 はさらに、
RHS が local structure head である real same-module local-mode expansion を、missing
mode-expansion diagnostic ではなく同じ missing evidence-query diagnostic へ運ぶ。
task 58 はさらに、RHS が attributed builtin head である real same-module
local-mode expansion を、missing mode-expansion diagnostic ではなく同じ missing
evidence-query diagnostic へ運ぶ。
task 59 はさらに、同じ mode が bare reserve use と mixed でない場合に、real direct
bare-builtin RHS expansion を持つ attributed same-module local-mode reserve head を同じ
missing evidence-query diagnostic へ運ぶ。task 60 はさらに、同じ mode が bare reserve use と
mixed でない場合に、real direct local-structure RHS expansion を持つ attributed
same-module local-mode reserve head を同じ missing evidence-query diagnostic へ運ぶ。
task 61 はさらに、同じ mode が bare reserve use と mixed でない場合に、real direct
attributed-builtin RHS expansion を持つ attributed same-module local-mode reserve head を
同じ missing evidence-query diagnostic へ運ぶ。task 62 はさらに、same-module local
structure RHS で終端する one-edge bare local-mode chain を、同じ `SurfaceAst` から両方の
real mode-expansion payload を抽出したうえで同じ missing evidence-query diagnostic へ運ぶ。
task 63 も同様に、attributed builtin RHS で終端する one-edge bare local-mode chain を、
同じ `SurfaceAst` から両方の real mode-expansion payload を抽出したうえで missing
attributed-type evidence-query diagnostic へ運ぶ。task 64 は one-edge dependency chain が
bare builtin RHS に終端する attributed same-module local-mode reserve head を、同じ
`SurfaceAst` から両方の real mode-expansion payload と reserve-head attribute を抽出した
うえで同じ missing attributed-type evidence-query diagnostic へ運ぶ。task 65 は one-edge
dependency chain が same-module local structure RHS に終端する attributed same-module
local-mode reserve head を、同じ `SurfaceAst` から両方の real mode-expansion payload と
reserve-head attribute を抽出したうえで missing base-shape / constructor-witness と full
attributed-type evidence-query diagnostic へ運ぶ。task 66 は one-edge dependency chain が
attributed builtin RHS に終端する attributed same-module local-mode reserve head を、同じ
`SurfaceAst` から両方の real mode-expansion payload、reserve-head attribute、terminal
RHS attribute を抽出したうえで missing full attributed-type evidence-query diagnostic へ運ぶ。
Task 250はexact Task-67 structure-qualified extraction gapを、written qualifierと
authenticated structure/attribute provenanceをraw source-attribute handoffへ保持して
supersedeする。owner compatibility、admissibility、evidence、truthはdeferredのまま。
task 68 は checker payload が real type-argument と term-argument provenance をまだ
持たないため、argument-bearing same-module local mode reserve head を external
extraction-gap diagnostic へ運ぶ。
task 69 は checker payload が real type-argument と term-argument provenance をまだ
持たないため、argument-bearing same-module local structure reserve head を external
extraction-gap diagnostic へ運ぶ。
task 70 は checker payload が real bracket type-argument と `qua`-argument provenance を
まだ持たないため、bracket type-argument payload extraction や mode-head resolution の前に
bracket-form same-module local mode reserve head を external extraction-gap diagnostic へ運ぶ。
task 71 は checker payload が real bracket type-argument と `qua`-argument provenance を
まだ持たないため、bracket type-argument payload extraction や structure-head resolution の前に
bracket-form same-module local structure reserve head を external extraction-gap diagnostic へ運ぶ。
task 72 はさらに、builtin `set` / `object` で終端する real AST-derived two-edge bare
local-mode chain を既存の pass readiness path へ通す。task 73 は同じ source-derived
seam を builtin `set` / `object` で終端する three-edge bare local-mode chain へ昇格する。
task 74 は temporary depth cap を AST-bounded structural bare local-mode chain rule に置き換える。
task 75 は、後続 same-module local mode declaration を reserve head が先に名前参照する
case について lower-stage active-range boundary を記録する。frontend/resolver
processing は checker handoff 前に unresolved type expression を拒否するため、
future mode declaration は `ModeExpansion` payload へ変換されない。
task 76 は、後続 same-module local structure declaration を reserve head が先に
名前参照する case について対応する lower-stage active-range boundary を記録する。
frontend/resolver processing は checker handoff 前に unresolved type expression を
拒否するため、future structure declaration は checker structure type-head payload や
base-shape evidence query へ変換されない。
これは checker の新しい raw-syntax dependency ではなく、non-builtin declaration、
imported symbol、attribute / mode / structure argument、qualified attribute provenance、
bracket `type_arg_list` と `qua`-argument provenance、type-argument / term-argument
provenance、structure base-shape / full attributed-type
existential evidence、broader / imported / attributed /
argument-bearing / parameterized / contextual / ambiguous / cyclic
mode expansion、term、formula、overload、CoreIr、
ControlFlowIr、VC payload、proof evidence の AST-wide source-to-checker gap を閉じるものでもない。

## Task 277C frozen source/specification mapping

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Crate Module Exports

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [live ledger](../checker_source_inventory.tsv) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Public Surface Inventory

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [live ledger](../checker_source_inventory.tsv) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Cross-Cutting Test And Policy Evidence

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Gap Reconciliation

current status owner: [Known Gaps And Drift](../en/00.crate_plan.md#known-gaps-and-drift) / historical [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 32 Classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Completion Decision

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 201 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 202 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 203 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 204 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 205 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 206 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 207 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 208 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 209 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 210 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 211 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 212 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 213 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 214 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 215 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 216 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 217 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 218 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 219 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 220 MC-G020 current-state override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 221 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 222 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 223 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## task 224 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 225 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 226 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 227 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 228 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 229 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 230 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 231 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 233 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 234 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 236 MC-G020 active override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 241 MC-G020 Active Override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 242 MC-G020 Active Override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 243 MC-G020 Active Override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 244 MC-G020 Active Override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 245 MC-G020 Active Override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 246 MC-G020 Active Override

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 247 remaining-family ownership reconciliation

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 248 current-state addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 249 frozen-contract audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 249 implementation audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 260 active public-surface audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 249R definition-return audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 260 Pre-Implementation Classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 259 active public-surface result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 250 frozen-contract audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 250 implementation audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 251 frozen-contract audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 252 frozen-contract audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 252 contract-correction audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 252 implementation audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 253 frozen-contract audit addendum

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Step 5 Checker Task 254 frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Step 5 Checker Task 254 implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Step 5 Checker Task 255 frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Step 5 Checker Task 255 implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Step 5 Checker Task 256 frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Step 5 Checker Task 256 implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Step 5 Checker Task 257A frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Step 5 Checker Task 257A implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Checker Task 257B1 frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Checker Task 257B1 Implementation Audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 257C1 predicate-chain segment classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 255C1 condition-bearing-comprehension classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 257C2 condition-formula association classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 256C1 condition-container compatibility classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 257C3 frozen surface audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 257C3 implementation classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 257C4C0 nested Fraenkel capture test intent

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 257C4C1 explicit-import lexical admission

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258A frozen surface audit

Completion evidence: [central Task-258A historical contract](../../task_contracts/ja/258A.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B1 frozen surface audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B1 implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B2 frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B2 implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3 authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3N authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M1 implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3N 実装監査

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M1 authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2A authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2A implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B1 authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B1 implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2A frozen-ownership audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2A implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B1P frozen lower-prerequisite audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B1P implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B1A frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B1A implementation audit result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B1B1P frozen-prerequisite audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B1B1P implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B1B1 frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B1B1 implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2P frozen-prerequisite audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2P implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2A frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2A implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2BP frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2BP implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2B frozen-contract audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2B implementation audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2CP specification audit

Completion evidence: [central Task-258B3M2B2B2CP historical contract](../../task_contracts/ja/258B3M2B2B2CP.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2C specification audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2C implementation audit update

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2C broad verification audit update

Completion evidence: [central Task-258B3M2B2B2C historical contract](../../task_contracts/ja/258B3M2B2B2C.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B2C post-commitとTask 258B3M2B2B3P specification audit

Completion evidence: [central Task-258B3M2B2B3P historical contract](../../task_contracts/ja/258B3M2B2B3P.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3P final quality audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3P implementation source/specification status

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3A frozen source/specification audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3A implementation source/specification closure

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3B source-spec audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3B implementation authority closure

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3C authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3C implementation source/spec closure

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3D authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3D implementation authority closure

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3E authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B3M2B2B3E implementation source/spec inventory

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B4A authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B4A implementation authority result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B4B authority audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B4B implementation authority result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B4C authority audit

Completion evidence: [central Task-258B4C historical contract](../../task_contracts/ja/258B4C.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B4C Implementation Authority Result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B5A frozen authority audit

Completion evidence: [central Task-258B5A historical contract](../../task_contracts/ja/258B5A.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B5A implementation authority result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B5B frozen authority result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B5B implementation authority result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B5C frozen authority result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 258B5C active coverage result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 259 Frozen Authority Audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 248 Two-Parameter Profile-Extension Authority Audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 259 Corrected Future Public-Surface Audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 261 Frozen Future Public-Surface Audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 261 Active Public-Surface Result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 262 frozen future public-surface audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 249M frozen future public-surface audit

Completion evidence: [central Task-249M historical contract](../../task_contracts/ja/249M.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 262 active source audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 249S frozen future public-surface audit

Completion evidence: [central Task-249S historical contract](../../task_contracts/ja/249S.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 263 frozen source/API audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 263 active source/API result

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 264R source/specification audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 264R implemented source/specification status

Completion evidence: [central Task-248P historical contract](../../task_contracts/ja/248P.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 248P implemented source/specification status

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 264 frozen source/specification status

Completion evidence: [central Task-249PI historical contract](../../task_contracts/ja/249PI.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 249PI implemented source/specification audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 264 implemented source/specification audit

Completion evidence: [central Task-269A historical contract](../../task_contracts/ja/269A.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269A implemented source/specification audit

Completion evidence: [central Task-269B historical contract](../../task_contracts/ja/269B.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269CP source/spec classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269C source/spec classification

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269CT source/API audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269CT implemented source/API audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GP source/API audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GS source/API audit

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269G source/spec delta

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 277A Frozen Source/Specification Mapping

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 277B-L Implemented Source/Specification Mapping

本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GT source/API delta

Completion evidence: [central Task-269GT historical contract](../../task_contracts/ja/269GT.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GUP frozen source/API delta

Completion evidence: [central Task-269GUP historical contract](../../task_contracts/ja/269GUP.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GUPT frozen source/spec mapping

Completion evidence: [central Task-269GUPT historical contract](../../task_contracts/ja/269GUPT.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GU source/spec mapping凍結

Completion evidence: [central Task-269GU historical contract](../../task_contracts/ja/269GU.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GCP frozen source/spec mapping

Completion evidence: [central Task-269GCP historical contract](../../task_contracts/ja/269GCP.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GC frozen source/spec mapping

Completion evidence: [central Task-269GC historical contract](../../task_contracts/ja/269GC.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GCT frozen source/spec mapping

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/ja/269GCT.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269GCU frozen source/specification mapping

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/ja/269GCU.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269SDP source/spec mapping

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/ja/269SDP.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).

## Task 269SDC frozen source/specification audit

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/ja/269SDC.md#completion-evidence)。
本文は英語正本へ移管: [../en/source_spec_audit.md](../en/source_spec_audit.md) / [archive](../../archive/checker_source_spec_audit_sections.md).
