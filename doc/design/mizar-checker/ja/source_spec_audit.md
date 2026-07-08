# Source/Spec 対応監査: mizar-checker

> Canonical language: English:
> [../en/source_spec_audit.md](../en/source_spec_audit.md).

task 32 は task 31 後の checker public surface と仕様上の約束を監査する。
source behavior、`.miz` fixture、expectation、public API は変更しない。
未接続の挙動は現在の実装都合を正本化せず、明示的な
`external_dependency_gap`、`test_gap`、または `deferred` として分類する。

## 範囲と方法

inventory は `crates/mizar-checker/src/lib.rs` の現在の `pub mod` export、
`crates/mizar-checker/src/*.rs` の top-level public item、そして crate-local
`dense_id!` / `string_key!` macro で生成される public newtype をすべて含む。
public method は、module spec が table、builder、output API として記述している
ため、所有する public type の下にまとめる。

監査対象の module specification:

- [typed_ast.md](./typed_ast.md)
- [binding_env.md](./binding_env.md)
- [type_checker.md](./type_checker.md)
- [registration_resolution.md](./registration_resolution.md)
- [cluster_trace.md](./cluster_trace.md)
- [overload_resolution.md](./overload_resolution.md)
- [resolved_typed_ast.md](./resolved_typed_ast.md)

結果: 実装済み explicit-payload API について、blocking な `source_drift`、
`design_drift`、`source_undocumented_behavior` は観測していない。残る
source-derived behavior の未 coverage は、下記の gap row により意図的に
defer されている。

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
task 67 は checker payload が real qualifier と owner provenance をまだ持たないため、
structure-qualified attribute reference を external extraction-gap diagnostic へ運ぶ。
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

## Crate Module Exports

`src/lib.rs` は次の checker-owned module だけを export する:

- `binding_env`
- `cluster_trace`
- `overload_resolution`
- `registration_resolution`
- `resolved_typed_ast`
- `type_checker`
- `typed_ast`

根拠: `tests/lint_policy.rs` の
`checker_public_semantic_api_matches_documented_modules` がこの list を検査し、
crate が direct `mizar-syntax` import を持たないこと、resolver/session dependency
boundary を保つことも guard している。

## Public Surface Inventory

### `typed_ast`

生成 public newtype:

- `TypedNodeId`, `LocalTypeContextId`, `TypeEntryId`, `NormalizedTypeId`,
  `OpenCandidateSetId`, `TypeFactId`, `CoercionId`, `InitialObligationId`,
  `TypeDiagnosticId`
- `TypedNodeKind`, `TypeRole`, `TypePredicateRef`, `TypeRuleId`,
  `TypeAssumptionId`, `BuiltinRuleId`, `ResolutionStepId`,
  `InitialObligationGoal`, `InitialObligationProvenance`

literal top-level public item:

- `TypedAst`, `TypedAstParts`, `TypedNode`, `TypedNodeLinks`, `TypingState`,
  `NodeRecoveryState`, `TypedArena`, `TypedArenaBuilder`,
  `TypedArenaError`, `TypedSiteRef`, `TypedSubjectRef`, `LocalTypeContext`,
  `LocalTypeContextDraft`, `LocalTypeContextTable`, `TypeContextLayer`,
  `BindingTypeRef`, `ContextRecoveryState`, `TypeEntry`, `TypeEntryDraft`,
  `TypeTable`, `TypeStatus`, `TypeEntryActual`, `TypeProvenance`,
  `TypeFact`, `TypeFactDraft`, `TypeFactTable`, `Polarity`,
  `FactProvenance`, `FactStatus`, `CoercionEntry`, `CoercionDraft`,
  `CoercionTable`, `CoercionKind`, `CoercionStatus`, `CoercionProvenance`,
  `InitialObligation`, `InitialObligationDraft`, `InitialObligationTable`,
  `InitialObligationKind`, `InitialObligationStatus`, `TypeDiagnostic`,
  `TypeDiagnosticDraft`, `TypeDiagnosticTable`, `TypeDiagnosticClass`,
  `TypeDiagnosticSeverity`, `DiagnosticRecoveryState`, `SourceRangeKey`,
  `TypedAstError`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| source-shaped typed AST root が node、context、type、fact、coercion、obligation、diagnostic side table を保持する。 | `TypedAst`, `TypedAstParts`, accessor/debug method in `src/typed_ast.rs`. | `arena_ids_are_dense_and_debug_rendering_is_stable`, `public_data_shapes_do_not_expose_proof_or_final_overload_fields`. | checker-owned data shape として実装済み。 |
| dense local id、arena validation、parent/child integrity。 | `TypedArena`, `TypedArenaBuilder`, `TypedArenaError`, generated dense ids. | `arena_validation_rejects_invalid_references_and_cycles`. | 実装済み。 |
| immutable local context snapshot と visible fact discipline。 | `LocalTypeContextTable`, `LocalTypeContextDraft`, `BindingTypeRef`, context recovery/status enum. | `local_context_snapshots_validate_parent_chain_and_visibility`, `context_validation_enforces_assumed_fact_visibility`. | 実装済み。 |
| type/fact/coercion/initial-obligation/diagnostic table は id と deterministic order を保持する。 | `TypeTable`, `TypeFactTable`, `CoercionTable`, `InitialObligationTable`, `TypeDiagnosticTable`. | `tables_round_trip_ids_and_deduplicate_facts`, `canonical_queries_are_deterministic`, `coercion_ordering_and_provenance_are_stable`. | 実装済み。 |
| partial typing と handoff boundary が明示される。 | `TypeStatus`, `FactStatus`, `CoercionStatus`, `InitialObligationStatus`, recovery enum. | `status_variants_preserve_partial_typing_and_handoff_boundaries`, validation rejection tests. | 実装済み。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `binding_env`

生成 public newtype:

- `BindingContextId`, `BindingId`, `BindingDiagnosticId`

literal top-level public item:

- `BindingEnv`, `BindingEnvParts`, `BindingContextTable`,
  `BindingContext`, `BindingContextDraft`, `BindingContextOwner`,
  `BindingContextLayer`, `BindingContextRecovery`, `BindingTable`,
  `BindingEntry`, `BindingDraft`, `BindingKind`, `BinderIdentity`,
  `BindingTypeSite`, `BindingStatus`, `BindingRecoveryState`,
  `CapturedFreeVariables`, `BindingDiagnosticTable`, `BindingDiagnostic`,
  `BindingDiagnosticDraft`, `BindingDiagnosticClass`,
  `BindingDiagnosticSeverity`, `BindingDiagnosticRecovery`,
  `BindingLookupSite`, `BindingLookupResult`, `BindingEnvError`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Binding environment は resolver-boundary data layer であり、raw syntax walker ではない。 | `BindingEnv::try_new`, `BindingEnvParts`, resolver `SymbolEnv` inputs. | `public_module_shell_signature_stays_on_resolver_shell_boundary`, `module_shell_records_external_gaps_and_debug_rendering_is_stable`. | 実装済み。missing extraction は MC-G011。 |
| Context graph は parent、layer、ownership、recovery を検証する。 | `BindingContextTable`, `BindingContextDraft`, owner/layer/recovery enum. | `context_layers_and_validation_cover_parent_chain_and_recovery`, `validation_rejects_invalid_ranges_and_diagnostic_links`. | 実装済み。 |
| Lookup は deterministic、visibility-scoped、forward-reference aware。 | `BindingLookupSite`, `BindingLookupResult`, binding lookup methods. | lookup shadowing/ambiguity/tie-break/resolver-fallback tests. | explicit payload について実装済み。 |
| reserved variable、binder identity、closure metadata、diagnostic、rendering は stable。 | `BindingKind`, `BinderIdentity`, `CapturedFreeVariables`, diagnostic table/classes. | reserved/binder identity/diagnostic/canonical rendering tests. | 実装済み。full closure replay は deferred。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `type_checker`

literal top-level public item:

- `TypeNormalizationOutput`, `TypeNormalizer`, `DeclarationCheckingOutput`,
  `DeclarationChecker`, `TermFormulaInferenceOutput`, `TermFormulaChecker`,
  `CoercionCheckingOutput`, `CoercionObligationChecker`, `CoercionInput`,
  `CoercionRequestKind`, `CoercionJustification`, `CoercionEvidence`,
  `CoercionDeferredReason`, `InitialObligationInput`, `InitialRequirementKind`,
  `TypeFactQueryEngine`, `TypeFactQuery`, `TypeFactQueryOutput`,
  `TypeFactQueryStatus`, `TermInput`, `TermKind`, `TermReference`,
  `TermDeferredReason`, `FormulaInput`, `FormulaKind`,
  `FormulaDeferredReason`, `ExpectedTypeInput`, `FormulaFactInput`,
  `OpenCandidateInput`, `CandidateIdentity`, `OpenCandidateSetTable`,
  `OpenCandidateSet`, `OpenCandidate`, `CandidateSetKind`,
  `CandidateSetStatus`, `CandidateStatus`, `CheckedTermTable`,
  `CheckedTermId`, `CheckedTerm`, `TermStatus`, `CheckedFormulaTable`,
  `CheckedFormulaId`, `CheckedFormula`, `FormulaStatus`,
  `ExpectedTypeConstraint`, `DeclarationContextInput`, `DeclarationInput`,
  `DeclarationKind`, `ReservedDefaultPayload`,
  `DeclarationDeferredReason`, `DeclarationAssumptionInput`,
  `CheckedDeclarationTable`, `CheckedDeclarationId`,
  `CheckedDeclaration`, `DeclarationStatus`,
  `SourceReserveDeclarationBridge`, `SourceReserveBindingInput`,
  `SourceReserveDeclarationHandoff`, `TypeExpressionInput`,
  `TypeHeadInput`, `AttributeInput`, `AttributePolarity`, `ModeExpansion`,
  `NormalizedTypeTable`, `NormalizedType`, `TypeHeadRef`,
  `TypeHeadErrorKind`, `AttributeSet`, `AttributeInstance`, `TypeSource`,
  `NormalizedTypeStatus`, `SourceRangeKey`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Type-expression normalization は mode、attribute、arity、unsupported input の degraded state を canonicalize し、cluster repair を行わない。 | `TypeNormalizer`, `TypeNormalizationOutput`, `TypeExpressionInput`, `ModeExpansion`, normalized type tables. | attribute/order/builtin/mode-expansion/degraded-head tests. | explicit payload について実装済み。MC-G014 は残る。 |
| Declaration checking は explicit declaration と binding context を消費し、partial output を保持する。 | `DeclarationChecker`, `DeclarationCheckingOutput`, declaration input/status tables. | declaration deterministic/invalid/constrained/set/attributed/reconsider tests. | explicit payload について実装済み。MC-G016 は残る。 |
| reserve-only source-derived producer seam は syntax-free な reserve payload を消費し、raw syntax を import せず checker-owned binding/declaration handoff data を構築する。 | `SourceReserveDeclarationBridge`, `SourceReserveBindingInput`, `SourceReserveDeclarationHandoff`. | `source_reserve_declaration_bridge_builds_checker_owned_handoff`, `source_reserve_declaration_bridge_validates_local_symbol_heads_and_mismatched_inputs`, active `mizar-test` type-elaboration runner regressions. | successful bare builtin `set` / `object` reserve declaration、builtin `set` / `object` に終端する accepted same-module bare local-mode chain、および MC-G020 に記録された active diagnostic reserve/type-head boundary について実装済み。task 82 は imported-mode slice を拡張し、documented `parser.type_fixtures` の imported `TypeCaseMode` head が `ImportedSource` provenance 付き checker type-head payload になり、その後 missing imported `ModeExpansion` payload だけで fail closed する。task 83 は imported-structure slice を documented structure `R` に限って拡張し、imported `SymbolKind::Structure` head が checker type-head payload になり、その後 missing structure-evidence query だけで fail closed する。task 97 は同じ slice を documented structure `TypeCaseStruct` に適用する。task 84 は imported-attribute slice を documented `TypeCaseAttr` に限って拡張し、imported `SymbolKind::Attribute` が builtin `set` 上の checker `AttributeInput` payload になり、その後 missing attributed-type evidence query だけで fail closed する。task 85 は imported-attribute slice を documented negative `empty` over builtin `set` に限って拡張し、imported `SymbolKind::Attribute` が negative checker `AttributeInput` payload になり、その後 missing attributed-type evidence query だけで fail closed する。`R` / `TypeCaseStruct` を超える imported structure、`TypeCaseAttr` と negative `empty`/builtin-`set` を超える imported attribute、imported mode expansion、imported module AST extraction、より広い AST-wide extraction は MC-G020 のまま。 |
| Term/formula inference は checked table、expected constraint、open candidate、fact、recovery を記録する。 | `TermFormulaChecker`, term/formula input and checked tables. | term/formula/recovery tests. | explicit payload について実装済み。MC-G017/MC-G019 は残る。 |
| Coercion と initial obligation は `VcId` や fabricated evidence なしで記録される。 | `CoercionObligationChecker`, `CoercionInput`, `InitialObligationInput`, justification/evidence/deferred enum. | coercion deterministic/missing evidence/alternate candidate tests; task 47 omitted-`reconsider` proof-free/requires-proof tests。 | explicit payload について実装済み。source-derived coercion/reconsider extraction について MC-G018/MC-G020 は残る。 |
| Fact query は deterministic、visibility-scoped、non-mutating。 | `TypeFactQueryEngine`, `TypeFactQueryOutput`, `TypeFactQueryStatus`. | deterministic/provenance/visibility/contradiction tests. | 実装済み。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

Task 75 addendum: active `mizar-test` type-elaboration runner は、後続
same-module local mode declaration を earlier reserve head から参照する
source-derived case について、
`type_elaboration.lower_stage.frontend:malformed_type_expression` を
観測する。これは Chapter 2/11 の active-range/no-forward-reference 境界だけを
credit し、future declaration から checker `ModeExpansion` payload を作ること、
CoreIr/ControlFlowIr/VC/proof payload coverage を主張することはしない。
forward-reference acceptance は active-range rule で forbidden として
lower-stage rejection だけで cover する。

Task 76 addendum: active `mizar-test` type-elaboration runner は、後続
same-module local structure declaration を earlier reserve head から参照する
source-derived case について、同じ
`type_elaboration.lower_stage.frontend:malformed_type_expression` を観測する。
これは Chapter 2/5/11 の active-range/no-forward-reference 境界と structure
syntax/type-head surface だけを credit し、future declaration から checker
structure type-head payload、base-shape evidence query、constructor-witness
evidence query、CoreIr/ControlFlowIr/VC/proof payload coverage を主張しない。

Task 77 addendum: active `mizar-test` type-elaboration runner は、後続
same-module local attribute declaration を earlier reserve type expression から
使う source-derived case について、同じ
`type_elaboration.lower_stage.frontend:malformed_type_expression` を観測する。
これは Chapter 2/6/11 の active-range/no-forward-reference 境界と attribute
syntax/use surface だけを credit し、future declaration から checker
`AttributeInput` extraction、attributed-type evidence query、CoreIr、
ControlFlowIr、VC、proof payload coverage を主張しない。

Task 78 addendum: task 83 より前の active `mizar-test` type-elaboration runner
は、documented `parser.type_fixtures` import summary 由来の structure symbol を
持つ source-derived reserve head について
`type_elaboration.external_dependency.ast_payload_extraction` を観測した。task 83
は `R` imported provenance/type-head coverage についてこの boundary を上書きし、
task 97 は `TypeCaseStruct` imported provenance/type-head coverage について同じ boundary
を上書きする。task-83 / task-97 bridge 外の imported structure は deferred とする。これは
import summary を real imported module AST extraction と扱わず、base-shape /
constructor-witness evidence、positive structure type elaboration、CoreIr、
ControlFlowIr、VC、proof payload coverage を主張しない。

Task 83/task 97 addendum: active `mizar-test` type-elaboration runner は
documented `parser.type_fixtures` imported structure symbol `R` と
`TypeCaseStruct` を real `ImportedSource` checker type head として観測し、
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を報告する。
これは imported structure provenance と type-head payload extraction だけを
credit し、imported module AST extraction、base-shape / constructor-witness
evidence、positive imported structure elaboration、CoreIr、ControlFlowIr、VC、
proof payload coverage は credit しない。

Task 79 addendum: task 82 より前の active `mizar-test` type-elaboration runner
は、documented `parser.type_fixtures` import summary 由来の mode symbol を持つ
source-derived reserve head について
`type_elaboration.external_dependency.ast_payload_extraction` を観測した。task 82
は `TypeCaseMode` imported provenance/type-head coverage だけについてこの
boundary を上書きする。task-82 bridge 外の imported mode は引き続き Chapter
3/7/11/12 の imported-mode reserve-head diagnostic boundary だけを credit する。
これは import summary を real imported module AST extraction と扱わず、
`ModeExpansion` payload、positive mode elaboration、CoreIr、ControlFlowIr、VC、
proof payload coverage を主張しない。

Task 80 addendum: task 84 / task 85 より前の active `mizar-test`
type-elaboration runner は、documented `parser.type_fixtures` import summary
由来の attribute symbol を持つ source-derived reserve type について
`type_elaboration.external_dependency.ast_payload_extraction` を観測した。task 84
は `TypeCaseAttr` imported provenance と `AttributeInput` payload coverage だけ、
task 85 は negative `empty`/builtin-`set` fixture だけについてこの boundary を
上書きする。task-84 / task-85 bridge 外の imported attribute は、source-derived
fixture と payload producer が存在するまで deferred とする。これは
import summary を real imported module AST extraction と扱わず、attributed-type
evidence、positive attributed type elaboration、CoreIr、ControlFlowIr、VC、
proof payload coverage を主張しない。
task 85 は positive `empty set` と `non empty object` をこの task-80 row 配下の
active external-gap boundary sidecar として保持する。

Task 84 addendum: active `mizar-test` type-elaboration runner は documented
`parser.type_fixtures` imported attribute symbol `TypeCaseAttr` を builtin `set`
上の real `ImportedSource` checker `AttributeInput` として観測し、
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を報告する。
これは imported attribute provenance と argument-free `AttributeInput` payload
extraction だけを credit し、imported module AST extraction、attributed-type
existential/evidence payload、positive imported attributed type elaboration、
`empty` のような generic imported attribute、structure-qualified attribute
owner provenance、attribute argument、CoreIr、ControlFlowIr、VC、proof payload
coverage は credit しない。

Task 85 addendum: active `mizar-test` type-elaboration runner は documented
`parser.type_fixtures` imported attribute symbol `empty` を、既存 `non empty set`
fixture について builtin `set` 上の real `ImportedSource` negative checker
`AttributeInput` として観測し、
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を報告する。
これはその fixture の imported attribute provenance と argument-free negative
`AttributeInput` payload extraction だけを credit し、imported module AST extraction、
attributed-type existential/evidence payload、positive `empty set`、non-`set` head
上の imported `empty`、broader imported attribute、structure-qualified attribute
owner provenance、attribute argument、CoreIr、ControlFlowIr、VC、proof payload
coverage は credit しない。
したがって positive `empty set` と `non empty object` は、この provenance bridge
ではなく active extraction-gap sidecar のままである。

Task 86 addendum: active `mizar-test` type-elaboration runner は
`theorem FormulaPayloadBoundary: thesis;` という formula-only theorem source を
parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。これは
source-derived theorem/formula extraction-gap boundary だけを credit する。
checker theorem/formula payload extraction、local proof context、recorded fact、
theorem acceptance、dedicated `formula_statement` runner、CoreIr、ControlFlowIr、
VC、proof payload coverage は主張しない。

Task 106 addendum: active `mizar-test` type-elaboration runner は task-87 の generic
boundary のうち `theorem TermFormulaPayloadBoundary: 1 = 1;` を supersede する。
unrecovered builtin equality theorem shape かつ structural Chapter 13 numeral
operand が 2 つだけの場合に限り、runner は real module-shell checker binding
context を作り、source-derived checker `TermInput` と equality `FormulaInput`
payload を `TermFormulaChecker` に渡す。その後 missing numeric type payload と
partial formula checking で fail closed する。numeric type payload extraction、
equality semantic checking、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 110 addendum: active `mizar-test` type-elaboration runner は
`theorem ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2);` という
imported predicate/functor theorem formula source について task 98 を supersede
する。parser / resolver 実行後に `parser.type_fixtures` の `divides` / `++`
imported provenance を検証し、source-derived checker term/formula payload を
渡してから、missing numeric type payload、missing functor signature payload、
missing predicate signature payload、partial formula checking を報告する。これは
Chapter 11、12、13、14、16 の exact imported predicate/functor checker bridge
だけを credit する。imported module AST extraction、semantic predicate/functor
signature、term inference、formula checking、recorded fact、theorem acceptance、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload
coverage は主張しない。

Task 100 addendum: active `mizar-test` type-elaboration runner は当初
`theorem BuiltinMembershipPayloadBoundary: 1 in 1;` という builtin membership
theorem formula source を parser / resolver 実行後に
`type_elaboration.external_dependency.ast_payload_extraction` として観測した。
task 108 はこの exact source を supersede し、real source-derived checker
`TermInput` / `FormulaInput` payload を渡して
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` を報告する。これは
Chapter 13、14、16 の narrow source-derived builtin membership term/formula
checker bridge だけを credit する。numeric type payload extraction、membership
operand expected-type construction/checking、recorded fact、theorem acceptance、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload
coverage は主張しない。

Task 107 addendum: active `mizar-test` type-elaboration runner は task-101
generic boundary の exact builtin inequality theorem formula source
`theorem BuiltinInequalityPayloadBoundary: 1 <> 2;` を supersede する。parser /
resolver 実行後、runner は real source-derived checker `TermInput` /
`FormulaInput` payload を渡し、
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` を報告する。これは Chapter
13、14、16 の narrow source-derived builtin inequality term/formula checker
bridge だけを credit する。numeric type payload extraction、inequality
desugaring または equality semantic checking、recorded fact、theorem acceptance、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload
coverage は主張しない。

Task 109 addendum: active `mizar-test` type-elaboration runner は
`theorem BuiltinTypeAssertionPayloadBoundary: 1 is set;` という exact builtin
type-assertion theorem formula source を parser / resolver 実行後に観測し、
source-derived checker `TermInput`、`FormulaInput`、asserted builtin `set`
`TypeExpressionInput` payload を渡してから missing numeric type payload と
partial formula checking で fail closed する。これは Chapter 3、13、14、16 の
exact source-derived builtin type-assertion bridge だけを credit する。より広い
asserted type payload extraction、type-assertion semantic checking、recorded
fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 113 addendum: active `mizar-test` type-elaboration runner は
`import parser.type_fixtures; theorem ImportedAttributeAssertionPayloadBoundary: 1 is empty;`
という exact imported attribute assertion theorem formula source について task 103 を
supersede する。parser / resolver 実行後に imported `empty` provenance を検証し、
source-derived numeral と attribute-assertion checker payload を渡し、missing
numeric type payload、missing formula / attribute semantic payload、partial formula
checking で fail closed する。これは Chapter 6、11、12、13、14、16 の exact
source-derived imported attribute assertion checker handoff だけを credit する。
imported module AST extraction、theorem formula 向け checker `AttributeInput`
payload extraction、attribute-chain semantic payload extraction、term inference、
attribute admissibility/semantic checking、formula checking、recorded fact、
theorem acceptance、dedicated `formula_statement` runner、CoreIr、ControlFlowIr、
VC、proof payload coverage は主張しない。

Task 114 addendum: active `mizar-test` type-elaboration runner は
`import parser.type_fixtures; theorem ImportedNonEmptyAttributeAssertionPayloadBoundary: 1 is non empty;`
という exact attribute-level `non empty` imported attribute assertion theorem
formula source について task 104 を supersede する。parser / resolver 実行後に
direct `non` surface と imported `empty` provenance を検証し、source-derived
numeral と attribute-assertion checker payload を渡してから missing numeric type
payload、missing formula / attribute semantic payload、partial formula checking
で fail closed する。これは Chapter 6、11、12、13、14、16 の exact
source-derived attribute-level `non empty` imported attribute assertion checker
handoff だけを credit する。imported module AST extraction、theorem formula 向け
checker `AttributeInput` payload extraction、negated attribute-chain semantic
payload extraction、term inference、negated attribute admissibility/semantic
checking、formula checking、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 111 addendum: active `mizar-test` type-elaboration runner は task 105 のうち
exact set-enumeration theorem formula source
`theorem SetEnumerationPayloadBoundary: {1, 2} = {1, 2};` だけを supersede する。
parser / resolver 実行後に、4 つの numeral item term、2 つの
set-enumeration term、builtin equality formula の real source-derived checker
payload を渡し、missing numeric type payload、missing set-enumeration
result-type/sethood payload、partial formula checking を報告する。これは Chapter
13、14、16 の exact checker handoff だけを credit する。broader
set-enumeration term extraction、term inference、equality/formula checking、
recorded fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 112 addendum: active `mizar-test` type-elaboration runner は task 99 のうち
exact formula connective / quantifier theorem source
`theorem FormulaConnectiveQuantifierPayloadBoundary: contradiction implies for x being set holds not contradiction;`
だけを supersede する。parser / resolver 実行後に implication、quantified
formula、negation の real source-derived checker `FormulaInput` shell を渡し、
missing formula payload と missing quantifier payload を報告する。これは Chapter
14 と 16 の exact checker shell handoff だけを credit する。formula constant、
child-formula graph payload、quantifier binder/context payload、formula checking、
recorded fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 88 addendum: active `mizar-test` type-elaboration runner は
`theorem ProofSkeletonPayloadBoundary: thesis proof thus thesis; end;` という
proof-block theorem source を parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。これは
Chapter 15 conclusion statement と Chapter 16 proof block の source-derived
proof-block / proof-skeleton extraction-gap boundary だけを credit する。checker
proof skeleton payload extraction、local proof context、formula payload extraction、
recorded fact、theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 90 addendum: active `mizar-test` type-elaboration runner は predicate
definition と functor definition を含む definition block を parser / resolver
実行後に観測し、`type_elaboration.external_dependency.ast_payload_extraction`
を報告する。これは Chapter 9 predicate definitions と Chapter 10 functor
definitions の source-derived predicate/functor definition extraction-gap
boundary だけを credit する。checker definition declaration payload extraction、
definition-local context、definiens formula/term payload extraction、overload
payload、recorded fact、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 91 addendum: active `mizar-test` type-elaboration runner は attribute
definition を parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。
これは Chapter 6 attribute definitions の source-derived attribute
definition extraction-gap boundary だけを credit する。checker attribute
definition declaration payload extraction、definition-local context、
formula-definiens payload extraction、attributed-type evidence、recorded fact、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof
payload coverage は主張しない。

Task 92 addendum: active `mizar-test` type-elaboration runner は mode definition
と structure definition を parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。
これは Chapter 5 と Chapter 7 の source-derived mode/structure definition
extraction-gap boundary だけを credit する。checker mode/structure definition
declaration payload extraction、mode expansion、structure base-shape /
constructor / selector evidence、definition-local context、recorded fact、
dedicated `formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload
coverage は主張しない。

Task 93 addendum: active `mizar-test` type-elaboration runner は theorem proof
内の proof-local `let`、`given`、`consider`、`set`、`reconsider` statement を
parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。
これは Chapter 15 と Chapter 16 の source-derived proof-local declaration
extraction-gap boundary だけを credit する。checker proof-local declaration
payload extraction、local proof context、formula / term payload extraction、
RHS term inference、reconsider coercion / obligation evidence、recorded fact、
theorem acceptance、dedicated `formula_statement` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

Task 94 addendum: active `mizar-test` type-elaboration runner は theorem proof
内の proof-local `deffunc` と `defpred` inline definition を parser / resolver
実行後に観測し、`type_elaboration.external_dependency.ast_payload_extraction`
を報告する。これは Chapter 15 の source-derived proof-local inline definition
extraction-gap boundary だけを credit する。checker inline definition
formal/body payload extraction、local abbreviation expansion、term / formula
body payload extraction、guard evidence、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 95 addendum: active `mizar-test` type-elaboration runner は existential
cluster と conditional cluster を含む top-level registration block を parser /
resolver 実行後に観測し、`type_elaboration.external_dependency.ast_payload_extraction`
を報告する。これは Chapter 17 の source-derived registration-block
extraction-gap boundary だけを credit する。checker registration-item payload
extraction、correctness-condition / proof-obligation payload、accepted
activation / evidence status、cluster / reduction semantics、recorded fact、
Chapter 17 semantic cluster / reduction row、dedicated `formula_statement` または
`advanced_semantics` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 96 addendum: active `mizar-test` type-elaboration runner は top-level と
definition-local の synonym / antonym alias、および attribute、predicate、
functor redefinition declaration を parser / resolver 実行後に観測し、
`type_elaboration.external_dependency.ast_payload_extraction` を報告する。これは
Chapter 11 / 19 の source-derived redefinition / notation extraction-gap
boundary だけを credit する。checker redefinition payload extraction、notation
alias relation payload、redefinition target inference、coherence
proof-obligation payload、overload candidate payload、recorded fact、Chapter 11
alias semantic resolution、Chapter 19 overload / redefinition semantics、
dedicated `formula_statement` または `advanced_semantics` runner、CoreIr、
ControlFlowIr、VC、proof payload coverage は主張しない。

### `registration_resolution`

生成 public newtype:

- `CheckerRegistrationId`, `RejectedRegistrationId`,
  `RegistrationDiagnosticId`, `ExistentialGateId`
- `RegistrationTriggerKey`, `RegistrationLabelKey`,
  `RegistrationPatternKey`, `RegistrationParameterKey`,
  `AcceptedCorrectnessKey`, `ActivationEvidenceKey`,
  `RegistrationFingerprint`, `RegistrationTypeKey`,
  `RegistrationAttributeKey`, `RegistrationFunctorKey`,
  `RegistrationTermKey`, `RegistrationVariableKey`,
  `ExistentialGateGuardKey`

literal top-level public item:

- `RegistrationDatabase`, `PendingRegistration`, `PendingRegistrationTable`,
  `RegistrationPatternStatus`, `PendingRegistrationStatus`,
  `ActivatedRegistration`, `ActivatedRegistrationIndex`,
  `RejectedRegistration`, `RejectedRegistrationTable`,
  `RejectedRegistrationReason`, `RegistrationSource`, `ResolverTargetShell`,
  `RegistrationValidationKind`, `RegistrationValidationInput`,
  `RegistrationValidationPattern`, `RegistrationTermPattern`,
  `RegistrationVariableOccurrence`, `RegistrationValidationParameter`,
  `RegistrationReferencedSymbolRole`, `RegistrationReferencedSymbol`,
  `ActivationInput`, `ActivationVerifierStatus`, `ExistentialGateInput`,
  `ExistentialGateCandidate`, `ExistentialGateBaseEvidence`,
  `ExistentialGateBaseEvidenceKind`, `ExistentialGateBaseEvidenceCoverage`,
  `ExistentialGateGuardEvidence`,
  `ExistentialGateRecovery`, `ExistentialGateOutput`,
  `ExistentialGateResult`, `ExistentialGateStatus`,
  `RegistrationDiagnostic`, `RegistrationDiagnosticDraft`,
  `RegistrationDiagnosticTable`, `RegistrationDiagnosticClass`,
  `RegistrationDiagnosticSeverity`, `RegistrationDiagnosticRecovery`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Pending/activated registration database は resolver origin を保持し、不完全 payload を activate しない。 | `RegistrationDatabase`, pending/activated/rejected tables and source records. | pending/activation/source/invalid activation tests. | 実装済み。MC-G021 は残る。 |
| Validation は obligation を emit し、kind-specific payload を検証し、accepted verifier/artifact status で activation を gate する。 | `RegistrationValidationInput`, validation pattern/parameter/reference types, `ActivationInput`. | validation/invalid/routing/reduction-size/accepted-unaccepted activation tests. | explicit payload について実装済み。MC-G025 は残る。 |
| Existential gate は accepted activation、visible guard、exact pattern/base-evidence match、base-shape coverage、deterministic recovery を要求する。 | `ExistentialGateInput`, candidates, base evidence, guard evidence, output/result/status types. | missing/inactive/pending/unaccepted/accepted/rejected/degraded existential tests; task 47 base-object/set、accepted-mode、structure-field、schema-parameter evidence tests。 | explicit payload について実装済み。MC-G026 は残る。 |
| Diagnostic と deterministic rendering は stable。 | `RegistrationDiagnosticTable` and diagnostic classes/recovery. | debug rendering and validation diagnostic tests. | 実装済み。public diagnostic code は MC-G005。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `cluster_trace`

生成 public newtype:

- `ClusterFactId`, `ClusterStepId`, `ReductionStepId`,
  `ClusterDiagnosticId`
- `ClusterFactFingerprint`, `ClusterTypeFingerprint`,
  `ClusterAttributeFingerprint`, `ClusterRuleFingerprint`,
  `ClusterAuditKey`, `ClusterOrderingVersion`, `ClusterTraversalCacheKey`,
  `ReductionFingerprint`, `ReductionRuleFqn`, `ReductionTermFingerprint`,
  `ReductionRedexPath`, `ReductionBindingKey`, `ReductionGuardKey`,
  `ReductionGuardEvidenceKey`, `ReductionRuleViewFingerprint`,
  `ReductionSelectionKey`, `ReductionStrategyAuditKey`

literal top-level public item:

- `ClusterClosureOutput`, `ClusterClosureStatus`, `ReductionTraceOutput`,
  `ClusterTraceBuilder`, `ReductionTraceBuilder`, `ClusterTraversalConfig`,
  `ResolutionTrace`, `ResolutionTraceStep`, `ClusterStep`,
  `ReductionStep`, `ClusterAntecedentRef`, `ClusterTraversalProfile`,
  `ClusterReplayReport`, `ClusterReplayStatus`, `ClusterFactInput`,
  `ClusterRuleInput`, `ClusterRuleDraft`, `ReductionInput`,
  `ReductionDraft`, `ReductionBinding`, `ReductionGuardKind`,
  `ReductionGuardRequirement`, `ReductionGuardEvidenceRef`,
  `ClusterRuleKind`, `ClusterFact`, `ClusterFactDraft`,
  `ClusterFactTable`, `ClusterFactProvenance`, `ClusterDiagnostic`,
  `ClusterDiagnosticDraft`, `ClusterDiagnosticTable`,
  `ClusterDiagnosticClass`, `ClusterDiagnosticSeverity`,
  `ClusterDiagnosticRecovery`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Cluster closure は replayable deterministic cluster step と derived fact を記録する。 | `ClusterTraceBuilder`, `ClusterClosureOutput`, `ResolutionTrace`, `ClusterStep`, `ClusterFactTable`. | closure/inactive/order/conditional/subtype/transitive/mismatch/rejected/duplicate tests. | explicit payload について実装済み。MC-G023 は残る。 |
| Saturation bound、loop、explicit contradiction は silent truncation でなく visible failure。 | `ClusterTraversalConfig`, `ClusterTraversalProfile`, `ClusterClosureStatus`, diagnostics. | loop/bound/zero-antecedent/contradiction tests; task 46 class/severity/recovery assertion。 | explicit payload について実装済み。 |
| Replay は active registration fingerprint を再検証する。 | `ResolutionTrace::replay`, `ClusterReplayReport`, `ClusterReplayStatus`. | `replay_revalidates_active_registration_fingerprint`, `active_pattern_fallback_must_match_rule_fingerprint`. | 実装済み。 |
| Reduction step は architecture 17 provenance、guard evidence、strategy audit key を保持する。 | `ReductionTraceBuilder`, `ReductionTraceOutput`, `ReductionStep`, reduction input/guard types. | reduction provenance/inactive/rejected/invalid/`such` guard tests; task 46 discharged-side-condition trace identity determinism coverage。 | explicit payload について実装済み。source-derived rewrite extraction と normalization-result dependence は MC-G023。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `overload_resolution`

生成 public newtype:

- `OverloadSiteId`, `OverloadCandidateId`, `OverloadDiagnosticId`,
  `TemplateExpansionId`, `CandidateViabilityId`, `SpecificityGraphId`,
  `SpecificityComparisonId`, `SpecificityEdgeId`, `OverloadResultId`,
  `InsertedViewId`
- `OverloadSiteKey`, `OverloadNameKey`,
  `OverloadDiagnosticMessageKey`, `CandidateProvenanceKey`,
  `TemplateInstantiationKey`, `TemplateParameterKey`, `QuaPathKey`,
  `ViabilityEvidenceKey`, `SpecificityReasonKey`, `SelectionReasonKey`,
  `InsertedViewReasonKey`

literal top-level public item:

- `OverloadCollectionOutput`, `TemplateExpansionOutput`,
  `CandidateViabilityOutput`, `SpecificityGraphOutput`,
  `OverloadSelectionOutput`, `OverloadSiteInput`, `SourceQuaView`,
  `OverloadSiteKind`, `UnsupportedOverloadRole`, `OverloadSiteRecovery`,
  `OverloadCandidateInput`, `CandidateDeclarationKind`, `CandidateOrigin`,
  `CoherenceStatus`, `TemplateCandidatePayload`, `TemplateArgument`,
  `TemplateQuaStatus`, `TemplateArgumentInference`,
  `TemplateConstraintEvidence`, `TemplateConstraintEvidenceStatus`,
  `CandidateProvenance`, `CandidateScope`, `OverloadSite`,
  `OverloadSiteStatus`, `OverloadSiteTable`, `OverloadCandidate`,
  `OverloadCandidateStatus`, `OverloadCandidateTable`,
  `TemplateExpansion`, `TemplateExpansionTable`, `TemplateSubstitution`,
  `TemplateSubstitutionSource`, `TemplateExpansionStatus`,
  `TemplateExpansionFailure`, `CandidateViabilityInput`,
  `ArgumentViabilityEvidence`, `ViabilityFactStatus`,
  `ViabilityCoercionKind`, `ViabilityCoercionStatus`,
  `CandidateViability`, `CandidateViabilityTable`,
  `CandidateViabilityStatus`, `ArgumentViewPlan`, `ArgumentViewKind`,
  `CandidateRejection`, `CandidateRejectionReason`,
  `CandidateBlockedReason`, `CandidateBlockedReasonKind`,
  `SpecificityComparisonInput`, `SpecificityComparisonStatus`,
  `SpecificityBlockedReasonKind`, `SpecificityGraph`,
  `SpecificityGraphTable`, `SpecificityNode`, `SpecificityComparison`,
  `SpecificityComparisonOutcome`, `SpecificityEdge`,
  `SpecificityFailureReason`, `OverloadSiteResolutionInput`,
  `RefinementJoinPayload`, `RefinementJoinStatus`,
  `RefinementJoinFailure`, `ExposedResultPayload`,
  `ExposedResultSource`, `InsertedViewInput`, `InsertedViewKind`,
  `InsertedViewStatus`, `OverloadResult`, `OverloadResultTable`,
  `OverloadResultStatus`, `OverloadBlockedReason`, `InsertedView`,
  `InsertedViewTable`, `OverloadDiagnostic`, `OverloadDiagnosticDraft`,
  `OverloadDiagnosticProvenance`, `OverloadDiagnosticTable`,
  `OverloadDiagnosticClass`, `OverloadDiagnosticSeverity`,
  `OverloadDiagnosticRecovery`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Site/candidate collection は resolver-filtered input、source `qua`、provenance、supported role、stable ordering を保持する。 | `OverloadCollectionOutput::collect`, site/candidate input and table types. | collection/filter/order/provenance/template/source-`qua`/unsupported/duplicate tests. | explicit payload について実装済み。MC-G027 は残る。 |
| Template expansion は explicit template/inference/constraint payload を使い、rejected/deferred case を保持する。 | `TemplateExpansionOutput::expand`, template payload/substitution/constraint types. | expansion/omitted/constraint/source-`qua`/diagnostic/deferred tests. | explicit payload について実装済み。MC-G006/MC-G027 は残る。 |
| Viability は recorded evidence を消費し、新規 fact derivation、registration firing、root selection をしない。 | `CandidateViabilityOutput::filter`, viability evidence/status/rejection/block/view-plan types. | exact/source-`qua`/non-consumable/narrowing/missing/deferred/remap tests. | 実装済み。 |
| Specificity は explicit comparison から per-site graph を作り、ordinary root ordering に return type を使わない。 | `SpecificityGraphOutput::build`, graph/node/comparison/edge types. | graph/equivalence/return-type/empty/blocked/missing tests; task 45 encoded non-template/template priority comparison。 | 実装済み。 |
| Selection は unique maximal ordinary root を選択し、accepted refinement を join し、widening/source-`qua` view を記録し、failure を保持する。 | `OverloadSelectionOutput::resolve`, result/view/refinement/exposed-result types. | selection/no-match/ambiguity/missing/redefinition/refinement/invalid/deterministic tests; task 45 equivalent-template ambiguity、encoded priority、unencoded tie、redefinition-metadata tie tests。 | explicit payload について実装済み。`coherence with` 省略 target inference は MC-G027/MC-G030 の下で upstream producer obligation のままであり、rejected omitted-target declaration は active candidate としてこの API に到達してはならない。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

### `resolved_typed_ast`

生成 public newtype:

- `ResolvedTypedNodeId`, `ExpressionMetadataId`, `OverloadResolutionId`,
  `CoercionInsertionId`, `ResolvedTypedDiagnosticId`
- `ExprId`, `SourceNodeRole`

literal top-level public item:

- `ResolvedTypedAst`, `ResolvedTypedAstInputs`, `ExpressionMetadataInput`,
  `ResolvedNodeKindHint`, `ResolvedNodeKindHintKind`, `ResolvedTypedArena`,
  `ResolvedTypedNode`, `ResolvedTypedNodeKind`, `ResolvedNodeRecovery`,
  `ResolvedNodeRecoveryReason`, `ExpressionMetadata`,
  `ExpressionMetadataTable`, `OverloadCandidateSummary`,
  `OverloadCandidateSummaryTable`, `TemplateExpansionSummary`,
  `TemplateExpansionSummaryTable`, `CandidateViabilitySummary`,
  `CandidateViabilitySummaryTable`, `ResolvedSpecificityComparison`,
  `ResolvedSpecificityGraph`, `ResolvedSpecificityGraphTable`,
  `OverloadResolutionRecord`, `OverloadResolutionStatus`,
  `OverloadResolutionTable`, `CoercionInsertion`,
  `CoercionInsertionSource`, `CoercionInsertionTable`,
  `ResolvedTypedDiagnostic`, `ResolvedTypedDiagnosticSource`,
  `ResolvedTypedDiagnosticSeverity`, `ResolvedTypedDiagnosticTable`,
  `CandidateSummaryNamespace`, `ResolvedTypedAstError`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Final source-shaped projection は typed AST node、expression metadata、overload summary、cluster fact、diagnostic を保持する。 | `ResolvedTypedAst::assemble`, `ResolvedTypedAstInputs`, arena/metadata/summary/table types. | assembly/template/candidate/diagnostic remap tests. | explicit predecessor output について実装済み。source extraction/artifact は MC-G027。 |
| Failed overload site と failed node は success に書き換えられず可視のまま残る。 | `OverloadResolutionStatus`, recovery/reason enum, result and diagnostic tables. | failed-site/failed-selection/validation rejection tests. | 実装済み。 |
| Inserted coercion は source と widening/source-`qua` evidence だけを記録する。 | `CoercionInsertion`, `CoercionInsertionSource`, `CoercionInsertionTable`. | assembly and validation tests, upstream invalid-view tests. | explicit input について実装済み。 |
| deterministic debug projection は equivalent input order を canonicalize する。 | deterministic table iteration and `debug_text`. | `deterministic_debug_text_canonicalizes_equivalent_input_orderings`, task-30 determinism suite. | 実装済み。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

## Cross-Cutting Test And Policy Evidence

| 根拠 | coverage |
|---|---|
| `crates/mizar-checker/tests/lint_policy.rs` | workspace lint opt-in、dependency boundary、no direct syntax import、documented public module、explicit overload/resolved-AST boundary guard、public enum forward-compatibility policy、source/spec audit の public-surface と MC-G reconciliation guard、bilingual documentation sync の pair-inventory / companion-link guard、module-boundary source-layout guard、documented `allow` exception。 |
| `crates/mizar-checker/src/determinism_suite.rs` | type normalization、fact query、cluster closure、reduction trace discharged-side-condition identity、overload pipeline、final `ResolvedTypedAst` projection の cross-module deterministic rerun と equivalent-order permutation。 |
| 各 source module の unit tests | 実装済み checker seam に対する task-local behavior tests。source-to-checker extraction と後続 semantic corpus runner が存在するまで、これが active executable coverage。 |
| `tests/coverage/spec_trace.toml` deferred rows | formula/statement、cluster/reduction、overload/refinement、review-audit semantic corpus obligation を active fixture と偽らずに記録する。 |

## Gap Reconciliation

task 32 は implementation と test evidence なしに gap を閉じない。crate-plan の
MC-G row はすべて次のように照合する。

| ID | Classification | task 32 audit disposition |
|---|---|---|
| MC-G002 | `test_gap` | real semantic `.miz` coverage について active。source-to-checker extraction が入るまで Rust tests と deferred traceability で mitigated。 |
| MC-G003 | `design_drift` plus deferred external gate | 歴史的 wording drift は修復済み。accepted proof/artifact status は MC-G025 で表現する。 |
| MC-G004 | `source_drift` / `external_dependency_gap` planning gate | 現在 code における checker-source drift ではない。artifact producer/reuse integration は task-scoped cross-crate dependency のまま。checker は schema を invent しない。 |
| MC-G005 | `spec_gap` / `external_dependency_gap` | active。public checker diagnostic code-space は未割当。module は crate-local diagnostic class と stable message/detail key を使う。 |
| MC-G006 | `source_drift` / `external_dependency_gap` | parser/syntax template/scheme role について active。overload code は unsupported role を fabricate せず defer する。 |
| MC-G007 | `design_drift` | checker crate plan と module spec で解決済み。task 32 で architecture-file rename は不要。 |
| MC-G009 | `repo_metadata_conflict` | report-only sentinel。task 32 で metadata conflict は観測していない。 |
| MC-G011 | `external_dependency_gap` | AST-wide local binding extraction、use-site scope/ordinal payload、reserve payload、closure payload、syntax-free `ResolvedAst` fixture について active。 |
| MC-G014 | `external_dependency_gap` | AST-wide type-expression payload、mode/radix/attribute expansion payload、arity payload について active。 |
| MC-G016 | `external_dependency_gap` | declaration/type-site table、reserve default、RHS/body payload、evidence query について active。 |
| MC-G017 | `external_dependency_gap` | term/formula payload table、built-in numeric payload、candidate signature、structure/selector payload、source `qua` evidence、sethood/non-emptiness evidence について active。 |
| MC-G018 | `external_dependency_gap` | coercion request table、dependency-summary fact、inheritance graph、cluster evidence、sethood/non-emptiness evidence、proof-query result について active。 |
| MC-G019 | `external_dependency_gap` | statement/proof assumption、theorem acceptance payload、phase-7 trace fact payload について active。 |
| MC-G020 | `external_dependency_gap` / `deferred` | task 7-11 と後続 consumer の semantic pass fixture を妨げる source-to-checker extraction blocker として active。task 16 から task 81 までは reserve-only source-derived checker bridge を段階的に構築し、builtin reserve / type-expression payload は `TypedAst` と `ResolvedTypedAst` まで到達し、builtin `set` / `object` に終端する supported same-module bare local-mode expansion family は pass できる一方、local structure / attribute / argument / bracket / forward-reference / historical task-80 imported-attribute boundary は fabricated evidence なしの active diagnostic に留める。task 82 は task-79 imported mode source を一段進め、documented `parser.type_fixtures` 由来の imported `SymbolKind::Mode` を checker type-head payload として渡し、checker が `ImportedSource` provenance を検証してから missing imported `ModeExpansion` payload で fail closed する。task 83 は documented `parser.type_fixtures` imported structure `R` source を一段進め、checker が `ImportedSource` provenance を検証してから missing base-shape / constructor-witness evidence で fail closed する。task 97 は documented `TypeCaseStruct` source を同じ real checker type-head boundary と missing evidence query に進める。task 84 は documented `parser.type_fixtures` imported attribute `TypeCaseAttr` source を一段進め、checker が `ImportedSource` provenance を検証してから missing attributed-type existential/evidence payload で fail closed する。task 85 は既存 `non empty set` source を一段進め、builtin `set` 上の real imported negative `empty` checker `AttributeInput` payload として渡し、missing attributed-type existential/evidence payload で fail closed する一方、positive `empty set` と `non empty object` runner sidecar は extraction-gap boundary に残す。task 86 は parser / resolver 実行後の formula-only theorem source を active extraction-gap boundary として記録し、theorem/formula payload、proof context、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 106 は exact builtin equality theorem source について real source-derived checker `TermInput` と equality `FormulaInput` payload を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、equality semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 110 は task 98 の exact imported predicate/functor theorem source を supersede し、imported `divides`/`++` provenance を検証して real source-derived numeral、imported functor-application、predicate-application checker payload を `TermFormulaChecker` に渡し、missing numeric/signature payload と partial formula checking で fail closed するが、semantic predicate/functor signature、term inference、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 108 は exact builtin membership theorem source について real source-derived checker `TermInput` と membership `FormulaInput` payload を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、membership operand expected-type construction/checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 107 は exact builtin inequality theorem source について real source-derived checker `TermInput` と inequality `FormulaInput` payload を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、inequality desugaring または equality semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 109 は exact builtin type-assertion theorem source について real source-derived checker `TermInput`、type-assertion `FormulaInput`、asserted builtin `set` `TypeExpressionInput` を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、broader asserted type payload、type-assertion semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 113 は exact imported attribute assertion theorem formula について task 103 を supersede し、imported `empty` provenance を検証して source-derived checker term/formula payload を渡し、missing numeric type payload、missing formula / attribute semantic payload、partial formula checking で fail closed するが、imported module AST extraction、theorem formula 向け checker `AttributeInput` payload extraction、attribute-chain semantic payload extraction、term inference、attribute admissibility/semantic checking、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 114 は exact attribute-level non-empty imported attribute assertion theorem formula について task 104 を supersede し、direct `non` surface と imported `empty` provenance を検証して source-derived checker term/formula payload を渡し、missing numeric type payload、missing formula / attribute semantic payload、partial formula checking で fail closed するが、imported module AST extraction、theorem formula 向け checker `AttributeInput` payload extraction、negated attribute-chain semantic payload、term inference、negated attribute admissibility/semantic checking、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 111 は task 105 の exact set-enumeration theorem formula source だけを supersede し、source-derived numeral item term、set-enumeration term、builtin equality formula checker payload を `TermFormulaChecker` に渡して missing numeric/result-type payload と partial formula checking で fail closed するが、broader set-enumeration result-type/sethood payload extraction、term inference、equality/formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 112 は task 99 の exact formula connective/quantifier theorem formula surface だけを supersede し、parser / resolver 実行後に real checker formula shell payload を渡して missing formula/quantifier payload で fail closed するが、formula constant、child-formula graph payload、quantifier binder/context payload、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 88 は parser / resolver 実行後の proof-block theorem source を同じ active extraction-gap boundary として記録し、proof skeleton payload、local proof context、formula payload、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 89 は parser / resolver 実行後の statement-level proof-justification theorem source を同じ active extraction-gap boundary として記録し、statement proof payload、nested proof skeleton payload、local proof context、formula payload、label-reference semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 90 は parser / resolver 実行後の predicate/functor definition source を active extraction-gap boundary として記録し、definition declaration payload、definition-local context、definiens formula/term payload、overload payload、recorded fact、`formula_statement` runner は主張しない。task 91 は parser / resolver 実行後の attribute definition source を active extraction-gap boundary として記録し、attribute definition declaration payload、definition-local context、formula-definiens payload、attributed-type evidence、recorded fact、`formula_statement` runner は主張しない。task 92 は parser / resolver 実行後の mode/structure definition source を同じ active extraction-gap boundary として記録し、mode/structure definition declaration payload、mode expansion、structure base-shape / constructor / selector evidence、definition-local context、recorded fact、`formula_statement` runner は主張しない。task 93 は parser / resolver 実行後の proof-local declaration statement source を同じ active extraction-gap boundary として記録し、proof-local declaration payload、local proof context、formula / term payload、RHS term inference、reconsider coercion / obligation evidence、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 94 は parser / resolver 実行後の proof-local inline definition source を同じ active extraction-gap boundary として記録し、inline definition formal/body payload、local abbreviation expansion、term / formula body payload、guard evidence、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 95 は parser / resolver 実行後の registration block source を同じ active extraction-gap boundary として記録し、registration-item payload、correctness-condition / proof-obligation payload、accepted activation / evidence status、cluster / reduction semantics、recorded fact、Chapter 17 semantic row、`formula_statement` / `advanced_semantics` runner は主張しない。task 96 は parser / resolver 実行後の redefinition/notation source を同じ active extraction-gap boundary として記録し、redefinition payload、notation alias relation payload、target inference、coherence proof-obligation payload、overload candidate payload、Chapter 11 alias semantic resolution、Chapter 19 overload/redefinition semantics、`formula_statement` / `advanced_semantics` runner は主張しない。より広い non-builtin declaration（task 96 の redefinition/notation extraction-gap boundary を超えるもの）、task 84 `TypeCaseAttr` provenance / `AttributeInput` bridge と task 85 negative `empty`/builtin-`set` bridge を超える imported attribute、task 83 `R` と task 97 `TypeCaseStruct` provenance/type-head bridge を超える imported structure、task 82 provenance/type-head bridge を超える imported mode expansion、attribute argument、qualified attribute qualifier / owner provenance、mode / structure argument、bracket `type_arg_list` と `qua`-argument provenance、term-argument provenance、structure base-shape / full attributed-type existential evidence、broader / attributed / argument-bearing / parameterized / contextual / ambiguous / cyclic mode expansion、task-106/task-107/task-108/task-109/task-110/task-111/task-112/task-113/task-114 を超える numeric/signature/result-type payload と equality/inequality/membership/type-assertion/imported predicate-functor/set-enumeration semantic checking および formula child/binder semantics、task-110/task-111/task-112/task-113/task-114 checker bridge と task-86/task-105/task-88/task-89 extraction-gap boundary を超える term / formula / proof skeleton、task-93 extraction-gap boundary を超える proof-local declaration payload、task-94 extraction-gap boundary を超える inline definition payload、task-95 extraction-gap boundary を超える registration payload / correctness-condition / activation payload、task-96 extraction-gap boundary を超える redefinition/notation payload、coercion、overload、recorded fact、CoreIr、ControlFlowIr、VC、proof payload extraction は未解決のまま。 |
| MC-G021 | `external_dependency_gap` / `deferred` | registration payload、accepted-status、source extraction blocker として active。registration code は explicit payload seam のみ消費する。 |
| MC-G023 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived cluster/reduction fixture、artifact/cache integration、source-derived normalization-result dependence、real trace extraction について active。task 46 は explicit-payload fatal contradiction と reduction trace-identity seam だけを cover する。 |
| MC-G025 | `external_dependency_gap` / `deferred` | accepted registration status の proof/artifact production または import について active。 |
| MC-G026 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived existential gate case、artifact reuse、accepted-status integration について active。 |
| MC-G027 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived overload payload、`coherence with` 省略 target diagnostic production、diagnostic code allocation、artifact emission/reuse、semantic fixture について active。task 45 は explicit-payload Rust regression だけを追加し、source-derived seed は inactive のままにする。 |
| MC-G030 | `test_gap` / `external_dependency_gap` / `deferred` | `formula_statement` と `advanced_semantics` runner/tag support、および source payload extraction について active。 |

Resolved setup-history row は closed のまま: MC-G001、MC-G010、MC-G012、
MC-G013、MC-G015、MC-G022、MC-G024、MC-G028、MC-G029 は task commit を持ち、
この audit で再オープンされた source/spec mismatch はない。

## Task 32 Classification

| Class | Evidence | Action |
|---|---|---|
| `spec_gap` | この audit は新しい language behavior や checker diagnostic code allocation を導入しない。MC-G005 が public diagnostic-code gap のまま。 | task 32 では public code を追加しない。 |
| `test_gap` | active `.miz` semantic fixture coverage は MC-G002/MC-G023/MC-G026/MC-G027/MC-G030 により deferred のまま。 | Rust task-local coverage と deferred traceability を維持し、pass fixture を fabricate しない。 |
| `design_drift` | blocking drift は観測していない。歴史的 MC-G003/MC-G007 drift は crate plan/module specs で解決済み。 | この audit record 以外の design repair はない。 |
| `source_drift` | 実装済み explicit-payload API は owning module spec と矛盾しない。 | source 変更なし。 |
| `source_undocumented_behavior` | top-level public item は上記 module inventory 外に存在しない。 | future public item は module spec と本 audit を更新するか、該当箇所では lint policy により捕捉される必要がある。 |
| `external_dependency_gap` | source extraction、accepted proof/artifact status、public diagnostic code、artifact emission/reuse、後続 semantic runner は現在の checker input の外。 | deferred row と follow-up task を維持し、stub は作らない。 |
| `deferred` | formula/statement、cluster/reduction、overload/refinement、audit-negative corpus obligation は記録済みだが inactive。 | owning extraction/runner/artifact task が入った後にだけ再訪する。 |

## Completion Decision

task 32 は、この English audit と Japanese companion、crate plan / todo update、
lint-policy audit guard が同じ commit に含まれた時点で完了する。この audit は
単体では crate completion を主張しない。task 33、task 34、closeout task は
bilingual synchronization audit、module-boundary refactor gate、crate exit report を
すでに記録している。
