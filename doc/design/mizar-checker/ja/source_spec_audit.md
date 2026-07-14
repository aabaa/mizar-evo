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
| reserve-only source-derived producer seam は syntax-free な reserve payload を消費し、raw syntax を import せず checker-owned binding/declaration handoff data を構築する。 | `SourceReserveDeclarationBridge`, `SourceReserveBindingInput`, `SourceReserveDeclarationHandoff`. | `source_reserve_declaration_bridge_builds_checker_owned_handoff`, `source_reserve_declaration_bridge_validates_local_symbol_heads_and_mismatched_inputs`, active `mizar-test` type-elaboration runner regressions. | successful bare builtin `set` / `object` reserve declaration、builtin `set` / `object` に終端する accepted same-module bare local-mode chain、および MC-G020 に記録された active diagnostic reserve/type-head boundary について実装済み。task 82 は imported-mode slice を拡張し、documented `parser.type_fixtures` の imported `TypeCaseMode` head が `ImportedSource` provenance 付き checker type-head payload になり、その後 missing imported `ModeExpansion` payload だけで fail closed する。task 83 は imported-structure slice を documented structure `R` に限って拡張し、imported `SymbolKind::Structure` head が checker type-head payload になり、その後 missing structure-evidence query だけで fail closed する。task 97 は同じ slice を documented structure `TypeCaseStruct` に適用する。task 84 は imported-attribute slice を documented `TypeCaseAttr` に限って拡張し、imported `SymbolKind::Attribute` が builtin `set` 上の checker `AttributeInput` payload になり、その後 missing attributed-type evidence query だけで fail closed する。task 85 は imported-attribute slice を documented negative `empty` over builtin `set` に限って拡張し、imported `SymbolKind::Attribute` が negative checker `AttributeInput` payload になり、その後 missing attributed-type evidence query だけで fail closed する。task 116 は documented positive `empty` over builtin `set` に同じ slice を適用し、positive checker `AttributeInput` payload として同じ missing attributed-type evidence query で fail closed する。`R` / `TypeCaseStruct` を超える imported structure、`TypeCaseAttr` と task-85/task-116 `empty`/builtin-`set` を超える imported attribute、imported mode expansion、imported module AST extraction、より広い AST-wide extraction は MC-G020 のまま。 |
| Term/formula inference は checked table、expected constraint、open candidate、fact、recovery を記録し、type assertion は normalized identity を受理して non-reflexive reachability payload 欠落で fail closed する。 | `TermFormulaChecker`, term/formula input and checked tables, `FormulaDeferredReason::MissingTypeAssertionReachabilityPayload`. | term/formula/recovery tests、`type_assertion_requires_reflexive_or_external_reachability_payload`。 | explicit payload は reflexive type-assertion admissibility まで実装済み。general widening/`qua` reachability は MC-G017/MC-G019/external のまま。 |
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

Task 80 addendum: task 84 / task 85 / task 116 / task 171 より前の active `mizar-test`
type-elaboration runner は、documented `parser.type_fixtures` import summary
由来の attribute symbol を持つ source-derived reserve type について
`type_elaboration.external_dependency.ast_payload_extraction` を観測した。task 84
は `TypeCaseAttr` imported provenance と `AttributeInput` payload coverage だけ、
task 85 は negative `empty`/builtin-`set` fixture だけについてこの boundary を
上書きし、task 116 は positive `empty`/builtin-`set` fixture だけについてこの
boundary を上書きし、task 171 は negative `empty`/builtin-`object` fixture だけに
ついてこの boundary を上書きする。task-84 / task-85 / task-116 / task-171 bridge 外の imported attribute は、source-derived
fixture と payload producer が存在するまで deferred とする。これは
import summary を real imported module AST extraction と扱わず、attributed-type
evidence、positive attributed type elaboration、CoreIr、ControlFlowIr、VC、
proof payload coverage を主張しない。positive `empty object` と symbol head 上の
imported attribute は active fixture credit のない deferred extraction gap に残る。

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
attributed-type existential/evidence payload、positive `empty object`、symbol head
上の imported attribute、broader imported attribute、structure-qualified attribute
owner provenance、attribute argument、CoreIr、ControlFlowIr、VC、proof payload
coverage は credit しない。
Task 116 addendum: active `mizar-test` type-elaboration runner は documented
`parser.type_fixtures` imported attribute symbol `empty` を、既存 `empty set`
fixture について builtin `set` 上の real `ImportedSource` positive checker
`AttributeInput` として観測し、
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を報告する。
これはその fixture の imported attribute provenance と argument-free positive
`AttributeInput` payload extraction だけを credit し、imported module AST extraction、
attributed-type existential/evidence payload、positive attributed-type acceptance、
positive `empty object`、symbol head 上の imported attribute、broader imported attribute、
structure-qualified attribute owner provenance、attribute argument、CoreIr、
ControlFlowIr、VC、proof payload coverage は credit しない。
Task 171 addendum: active runner は同じ imported `empty` symbol を、既存
`non empty object` fixture について builtin `object` 上の real
`ImportedSource` negative checker `AttributeInput` として観測し、同じ evidence-
query diagnostic を報告する。credit するのは exact imported provenance、negative
polarity、argument-free checker handoff だけである。positive `empty object`、
symbol head 上の imported attribute、imported module AST extraction、attributed-
type admissibility/evidence または acceptance、downstream payload は credit しない。

Task 181 current-state repair: tasks 84、85、116、171 が共有する source route は、
credit 済み source shape 5 件、すなわち exact single-binding fixture 4 件と exact
ordered mixed reserve fixture だけを受理する。real `parser.type_fixtures`
imported attribute は exact import/item layout、spelling、head、polarity、argument
なしを要求し、missing/wrong/duplicate import、duplicate/mixed attribute、broader
binding/item/order shape、definition、recovery node は extraction gap に残す。
既存 expectation と runner count 129 は変えない。この repair は
`source_undocumented_behavior` を閉じるが、positive `empty object`、attribute
evidence/acceptance、imported module AST extraction、downstream payload は昇格しない。

Task 86 / task 115 / task 117 addendum: task 86 は
`theorem FormulaPayloadBoundary: thesis;` という formula-only theorem source を
parser / resolver 実行後の source-derived theorem/formula boundary として記録する。
task 115 はこの exact source だけを supersede し、source-derived `thesis` formula
constant site/range を checker recovery `FormulaInput` として渡す。task 117 は
同じ exact source を real `FormulaKind::Thesis` payload に進め、missing formula
payload で fail closed する。これは exact formula-constant kind handoff だけを
credit し、formula constant semantic checking、child-formula graph payload、
local proof context、recorded fact、theorem acceptance、dedicated
`formula_statement` runner、CoreIr、ControlFlowIr、VC、proof payload coverage は
主張しない。

Task 119 MC-G020 current-state override: exact active source
`reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`
は real reserve declaration handoff と source-derived identifier-term / equality
payload を組み合わせる。2 つの use ordinal は reserve binding range と両
identifier range の順序から独立に導出し、checker-owned `BindingEnv` は各 use を
別々に解決する。記述された builtin `set` type は distinct source-anchored role
owner を通じて 2 つの term result type と 2 つの equality expected-type
constraint を供給する。production runner は 2 つの `Inferred` variable term、
type/well-formedness だけを表す 1 つの `Checked` equality、exact normalized type
source range/spelling/head、empty candidate/fact/deferred/diagnostic output を要求し、
payload drift は stable
`type_elaboration.checker.reserved_variable_equality.invalid_payload` key を報告する。
unit test は active sidecar の real frontend/resolver AST に対して同じ assertion を
反復する。これは MC-G020 の generic term/formula gap wording をその exact slice
だけについて override する。implicit universal-closure node、equality truth/fact、
theorem acceptance、`formula_statement` runner、proof skeleton、CoreIr、
ControlFlowIr、VC、broader source extraction は credit しない。

Task 120 MC-G020 current-state override: exact active membership source
`reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`
は task 119 の real binding/use event ordinal と independent lookup を再利用する。
既存 API が要求する checker-owned membership payload、すなわち 2 つの known
source-derived `set` variable result、右 operand の 1 つの expected-`set` role、
1 つの no-fact `Checked` membership formula だけを追加する。production
invariant、task-specific invalid-payload key、real frontend/resolver sidecar test が
slice を guard する。membership truth/fact、implicit closure、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC、broader source extraction は credit しない。

Task 121 MC-G020 current-state override: exact reserved-variable inequality
source は real binding/use producer を共有し、2 linked result role と 2 linked
expected role を fact-free pre-desugaring `Checked` inequality に渡す。production
invariant と real sidecar payload test が guard する。expected type shape の根拠は
checker-owned API coverage と task 119 であり、task 107 の partial numeral bridge
ではない。desugaring/truth/fact、implicit closure、theorem acceptance、proof、
CoreIr、ControlFlowIr、VC、broader extraction は open のままである。

Task 122 MC-G020 current-state override: exact reserved-variable type assertion
source は task 119 の real reserve lookup/result producer と task 109 の
formula-side asserted builtin-`set` AST producer を結合する。checker は exactly
one ready subject と one asserted type を要求し、normalized reflexive identity
だけで `Checked` を維持し、known non-identical type は widening を捏造せず
`checker.formula.external.type_assertion_reachability_payload` で fail closed する。
source runner は normalization 前の 2 input と distinct source anchor を独立検証し、
1 `Inferred` variable と 1 fact-free `Checked` type assertion を要求する。general
reachability/widening/`qua`、attribute、truth/fact、implicit closure、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC、broader extraction は open のままである。

Task 123 MC-G020 current-state override: exact distinct reserved-variable
equality source は、既存の real multi-reserve declaration producer と task 119 の
equality consumer を結合する。記述された builtin `set` range 1 個が 2 個の
source binding を裏付けるが、source-order lookup は operand ごとの
result/expected role が fact-free `Checked` equality に到達する前に `x` と `y`
を distinct checker binding id に解決する。production invariant、near-miss
matrix、real frontend/resolver sidecar が exact bridge を guard する。これは
classified `test_gap` / `source_drift` / `design_drift` を exact distinct-binding
type/well-formedness についてだけ閉じる。implicit closure/order node、equality
truth/fact、theorem acceptance、broader source shape、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 124 MC-G020 current-state override: exact multiple-reserve-declaration
equality source は同じ real declaration、binding、lookup、term、formula consumer
を再利用しつつ、2 個の記述上の builtin `set` range を operand ごとの 4 個すべての
pre-normalization result/expected input に distinct に保持する。checker はこれらの
semantically equal input を、deterministic な最初の source representative を持つ
1 `NormalizedTypeId` に intern するが、その representative は original input の
provenance を置き換えない。production invariant、exact near-miss matrix、real
frontend/resolver sidecar がこの `test_gap` / `source_drift` / `design_drift` repair
を guard する。implicit closure/order、truth/fact、theorem acceptance、broader
source、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 125 MC-G020 current-state override: exact heterogeneous-reserve membership
source は既存の real mixed-builtin reserve producer と task 120 の right-expected-type
membership consumer を結合する。runner は 2 distinct declaration range と binding
id にわたり、左 `object` result input と右 `set` result/expected input を保持する。
checker は 2 semantic identity を生成し、右の両 role は `set` を共有する。
production invariant は identity partition と deterministic な type ごとの source
representative を検証する。これは classified `test_gap` / `source_drift` /
`design_drift` を exact type/well-formedness slice についてだけ閉じる。membership
truth/fact、object/set coercion evidence、closure/order、theorem acceptance、broader
source、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 126 MC-G020 current-state override: exact direct-local-mode equality source
は task 55 の real AST-derived bare-set expansion と task 119 の equality consumer
を結合する。4 raw input は local-mode provenance を保持し、checker は expansion を
消費して 1 builtin-set identity、2 `Inferred` term、1 fact-free `Checked` equality
を記録する。mode declaration checking/acceptance、inhabitation evidence、broader
mode、closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、
VC は open のままである。

Task 127 MC-G020 current-state override: exact one-edge local-mode-chain equality
source は task 56 の real AST-derived expansion 2 個と task 126 の equality consumer
を結合する。4 raw input は outer-mode provenance を保持し、checker は両 link を
recursively 消費して real terminal `set` RHS に anchor された 1 builtin-set
identity、2 `Inferred` term、1 fact-free `Checked` equality を記録する。mode
declaration checking/acceptance、inhabitation evidence、object terminal、
longer-chain formula、closure/order、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 128 MC-G020 current-state override: exact direct local-object-mode equality
source は task 55 の real AST-derived `LocalObjectMode -> object` expansion と task
126 の equality consumer を結合する。4 raw input は object-mode provenance を保持し、
checker は real expansion を消費して real `object` RHS に anchor された 1
builtin-object identity、2 `Inferred` term、1 fact-free `Checked` equality を記録する。
exact production guard、invalid-expansion corruption、withheld-family near miss、
real sidecar が route を guard する。mode declaration checking/acceptance、
inhabitation evidence、broader object-mode formula、closure/order、truth/fact、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 130 MC-G020 current-state override: exact direct local-mode inequality は
task 55 の real `LocalModeInequality -> set` expansion と task 121 の
pre-desugaring consumer を結合する。4 raw input は mode provenance を保持し、
real RHS 起点の builtin-set identity 1 個、2 `Inferred` term、1 fact-free
`Checked` inequality を得る。declaration acceptance/inhabitation、desugaring、
closure/order、truth/fact、theorem acceptance、proof/Core/VC は open のままである。

Task 131 MC-G020 current-state override: exact direct local-object-mode
inequality は task 55 の real `LocalObjectModeInequality -> object` expansion と
task 121/130 の pre-desugaring consumer を結合する。4 raw input は object-mode
provenance を保持し、real RHS 起点の builtin-object identity 1 個、2
`Inferred` term、1 fact-free `Checked` inequality を得る。mode declaration
acceptance/inhabitation、desugaring、closure/order、truth/fact、theorem
acceptance、proof/Core/VC は open のままである。

Task 132 MC-G020 current-state override: exact one-edge set-terminal mode-chain
inequality は task 56/127 の real AST-derived expansion 2 個と task 121/130 の
pre-desugaring inequality consumer を結合する。4 raw input は outer-mode
provenance を保持し、recursive normalization は terminal RHS に 1 builtin-set
identity を anchor して 2 `Inferred` term と 1 fact-free `Checked` inequality を
生成する。declaration acceptance/inhabitation、object terminal、direct/longer
chain、desugaring、closure/order、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 133 MC-G020 current-state override: exact one-edge object-terminal
mode-chain inequality は task 129 の real AST-derived expansion 2 個と task 131
の pre-desugaring builtin-object inequality consumer を結合する。4 raw input は
outer-mode provenance を保持し、recursive normalization は terminal RHS に 1
builtin-object identity を anchor して 2 `Inferred` term と 1 fact-free
`Checked` inequality を生成する。declaration acceptance/inhabitation、
set-terminal、direct/longer chain、desugaring、closure/order、truth/fact、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 134 MC-G020 current-state override: exact two-edge set-terminal mode-chain
equality は task 72 の real AST-derived expansion 3 個と task 127 の equality
consumer を結合する。4 raw input は outer-mode provenance を保持し、recursive
normalization は terminal RHS に 1 builtin-set identity を anchor して 2
`Inferred` term と 1 fact-free `Checked` equality を生成する。declaration
acceptance/inhabitation、object terminal、direct/one-edge/longer chain、implicit
closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 135 MC-G020 current-state override: exact two-edge object-terminal
mode-chain equality は task 72 の real AST-derived expansion 3 個と task 134 の
equality consumer および builtin-object terminal support を結合する。4 raw input
は outer-mode provenance を保持し、recursive normalization は terminal RHS に 1
builtin-object identity を anchor して 2 `Inferred` term と 1 fact-free `Checked`
equality を生成する。declaration acceptance/inhabitation、task 134 を超える
set-terminal sibling semantics、direct/one-edge/longer chain、implicit
closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 136 MC-G020 current-state override: exact two-edge set-terminal
mode-chain inequality は task 72 の real AST-derived expansion 3 個と task 132 の
pre-desugaring inequality consumer を結合する。4 raw input は outer-mode
provenance を保持し、recursive normalization は terminal RHS に 1 builtin-set
identity を anchor して 2 `Inferred` term と 1 fact-free pre-desugaring `Checked`
inequality を生成する。mode declaration acceptance/inhabitation、object
terminal、direct/one-edge/longer chain、inequality desugaring、implicit
closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 137 MC-G020 current-state override: exact two-edge object-terminal
mode-chain inequality は task 72 の real AST-derived expansion 3 個と task 133 の
builtin-object pre-desugaring inequality consumer を結合する。4 raw input は
outer-mode provenance を保持し、recursive normalization は terminal RHS に 1
builtin-object identity を anchor して 2 `Inferred` term と 1 fact-free
pre-desugaring `Checked` inequality を生成する。declaration
acceptance/inhabitation、set terminal、direct/one-edge/longer chain、inequality
desugaring、implicit closure/order、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 138 MC-G020 current-state override: exact direct set-terminal local-mode
reserved-variable type assertion は task 55 の real AST-derived
`LocalModeTypeAssertion -> set` expansion と task 122 の normalized-reflexive
type-assertion consumer を結合する。raw subject は local-mode provenance、asserted
builtin `set` は独立した formula source を保持し、real expansion 1 本が両 type を
terminal-RHS builtin-set identity 1 個へ normalize してから、1 `Inferred` term と
1 fact-free `Checked` type assertion を記録する。mode declaration
acceptance/inhabitation、formula-side local-mode asserted head、general
reachability/widening/`qua`、truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 139 MC-G020 current-state override: exact direct set-terminal local-mode
left membership は task 55 の real AST-derived
`LocalModeMembership -> set` expansion、task 120 の right-only expected-set
membership consumer、task 125 の two-binding distinct-source form を結合する。
raw left result は local-mode provenance、独立した right result と sole
expected-set input は explicit reserve provenance を保持する。real expansion 1 本が
left role を normalize し、right builtin-set role は直接 normalize され、3 role
すべてが terminal-RHS builtin-set identity 1 個へ intern してから、2 `Inferred`
term と 1 fact-free `Checked` membership を記録する。mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 140 MC-G020 current-state override: exact direct object-terminal
local-mode left membership は task 55 の real AST-derived
`LocalObjectModeMembership -> object` expansion と task 125 の right-only
expected-set / two-binding distinct-source membership consumer を結合する。raw
left result は local object-mode provenance、独立した right result と sole
expected-set input は explicit reserve provenance を保持する。real expansion 1
本が left role を terminal-RHS builtin-object identity へ normalize し、right
builtin-set role は distinct explicit-reserve-anchored identity へ直接 normalize
する。2 `Inferred` term と 1 fact-free `Checked` membership を、right-owned
constraint 1 個だけ、left expected type なしで記録する。mode declaration
acceptance/inhabitation、membership truth/fact、object/set coercion、implicit
closure/order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open の
ままである。

Task 141 MC-G020 current-state override: exact one-edge set-terminal
local-mode-chain left membership は task 56 の real AST-derived
`ChainModeMembership -> BaseModeMembership -> set` expansion 2 本と task 139
の right-only expected-set / two-binding membership consumer を結合する。raw
left result は outer-mode provenance、独立した right result と sole expected-set
input は explicit reserve provenance を保持する。real expansion 2 本が left role
を recursive に normalize し、right builtin-set role は直接 normalize され、3
role すべてが terminal-RHS builtin-set identity 1 個へ intern する。right-owned
constraint 1 個だけ、left expected type なしで 2 `Inferred` term と 1 fact-free
`Checked` membership を記録する。mode declaration acceptance/inhabitation、
membership truth/fact、implicit closure/order、theorem acceptance、proof、
CoreIr、ControlFlowIr、VC は open のままである。

Task 142 MC-G020 current-state override: exact one-edge object-terminal
local-mode-chain left membership は task 56 の real AST-derived
`ChainObjectModeMembership -> BaseObjectModeMembership -> object` expansion 2
本と task 125 / 140 / 141 の right-only expected-set / two-binding membership
consumer を結合する。raw left result は outer-mode provenance、独立した right
result と sole expected-set input は explicit reserve provenance を保持する。real
expansion 2 本が left role を terminal-RHS builtin-object identity 1 個へ recursive
に normalize し、right builtin-set role は distinct explicit-reserve-anchored
identity 1 個へ直接 normalize される。right-owned constraint 1 個だけ、left
expected type なしで 2 `Inferred` term と 1 fact-free `Checked` membership を
記録する。mode declaration acceptance/inhabitation、membership truth/fact、
object/set coercion、implicit closure/order、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 129 MC-G020 current-state override: exact one-edge object-terminal
mode-chain equality は task 56 の real AST-derived expansion 2 個と task 127/128 の
equality / builtin-object consumer を結合する。4 raw input は outer-mode provenance
を保持し、recursive normalization は terminal RHS に 1 builtin-object identity を
anchor して 2 `Inferred` term と 1 fact-free `Checked` equality を生成する。
declaration acceptance/inhabitation、longer chain、closure/order、truth/fact、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 143 MC-G020 current-state override: exact two-edge set-terminal
local-mode-chain left membership は task 72 の real AST-derived expansion 3 本
`OuterTwoEdgeModeMembership -> MiddleTwoEdgeModeMembership -> BaseTwoEdgeModeMembership -> set`
と、task 125 / 139 / 141 の right-only expected-set two-binding membership
consumer を compose する。raw left result は outer-mode provenance、独立した
right result と sole expected-set input は explicit reserve provenance を保持する。
real expansion 3 本が left を再帰的に normalize し、right builtin-set role は
直接 normalize され、3 role すべてが terminal-RHS builtin-set identity 1 個へ
intern する。right-owned constraint 1 個だけ、left expected type なしで 2
`Inferred` term と 1 fact-free `Checked` membership を記録する。mode
declaration acceptance/inhabitation、membership truth/fact、implicit
closure/order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open の
ままである。

Task 144 MC-G020 current-state override: exact two-edge object-terminal
local-mode-chain left membership は task 72 の real AST-derived expansion 3 本
`OuterTwoEdgeObjectModeMembership -> MiddleTwoEdgeObjectModeMembership -> BaseTwoEdgeObjectModeMembership -> object`
と、tasks 125 / 140 / 142 / 143 の right-only expected-set two-binding
membership consumer を compose する。raw left result は outer-mode provenance、
独立した right result と sole expected-set input は explicit reserve provenance
を保持する。real expansion 3 本が left を terminal-RHS builtin-object identity
へ再帰的に normalize し、right role は distinct explicit-reserve builtin-set
identity へ直接 normalize される。right-owned constraint 1 個だけ、left
expected type なし、object/set coercion なしで 2 `Inferred` term と 1
fact-free `Checked` membership を記録する。mode declaration acceptance/
inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 145 MC-G020 current-state override: exact direct object-terminal
local-mode reserved-variable type assertion は task 55 の real AST-derived
`LocalObjectModeTypeAssertion -> object` expansion と tasks 122 / 138 の
identifier-result / 独立した formula-anchored asserted-type consumer を compose
する。raw subject result は written local-mode provenance、asserted builtin
`object` は独立した formula source node を保持する。real expansion 1 本が両
input を definition RHS を canonical source とする builtin-object identity 1
個へ normalize してから、1 `Inferred` term と 1 fact-free `Checked` type
assertion を記録する。この exact slice は `BindingId(0)` と source-order use
ordinal 1 を要求し、non-exact definition、reserve、formula、expansion payload
は fail closed する。mode declaration acceptance/inhabitation、formula-side
local-mode asserted-head extraction、general reachability/widening/`qua`、
object/set coercion、truth/fact、implicit closure/order、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は open のままである。

Task 146 MC-G020 current-state override: exact one-edge set-terminal
local-mode-chain reserved-variable type assertion は task 56 の real
AST-derived `ChainModeTypeAssertion -> BaseModeTypeAssertion -> set` expansion
2 本と、tasks 122 / 138 の identifier-result / 独立した formula-anchored
asserted-type consumer を compose する。raw subject result は written outer-mode
provenance、asserted builtin `set` は独立した formula source node を保持する。
real expansion 2 本が subject と asserted input を terminal definition RHS を
canonical source とする builtin-set identity 1 個へ再帰的に normalize して
から、1 `Inferred` term と 1 fact-free `Checked` type assertion を記録する。
この exact slice は `BindingId(0)` と source-order use ordinal 1 を要求し、
non-exact definition、reserve、formula、各 expansion payload は fail closed
する。mode declaration acceptance/inhabitation、formula-side local-mode
asserted-head extraction、general reachability/widening/`qua`、truth/fact、
implicit closure/order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 147 MC-G020 current-state override: exact one-edge object-terminal local-
mode-chain reserved-variable type assertion は task 56 の real AST-derived
`ChainObjectModeTypeAssertion -> BaseObjectModeTypeAssertion -> object`
expansion 2 本と、tasks 122 / 145 / 146 の identifier-result / 独立した formula-
anchored asserted-type consumer を compose する。raw subject result は written
outer-mode provenance、asserted builtin `object` は独立した formula source node
を保持する。real expansion 2 本が subject と asserted input を terminal
definition RHS を canonical source とする builtin-object identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
を記録する。この exact slice は `BindingId(0)` と source-order use ordinal 1
を要求し、non-exact definition、reserve、formula、各 expansion payload は fail
closed する。mode declaration acceptance/inhabitation、formula-side local-mode
asserted-head extraction、general reachability/widening/`qua`、object/set
coercion、truth/fact、implicit closure/order、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 148 MC-G020 current-state override: exact two-edge set-terminal local-mode-chain
reserved-variable type assertion は task 72 の real AST-derived
`OuterTwoEdgeModeTypeAssertion -> MiddleTwoEdgeModeTypeAssertion -> BaseTwoEdgeModeTypeAssertion -> set`
expansion 3 本と、tasks 122 / 146 / 147 の identifier-result / 独立した formula-
anchored asserted-type consumer を compose する。raw subject result は written
outer-mode provenance、asserted builtin `set` は独立した formula source node
を保持する。real expansion 3 本が subject と asserted input
を terminal definition RHS を canonical source とする builtin-set identity 1
個へ再帰的に normalize してから、1 `Inferred` term と 1 fact-free `Checked`
type assertion を記録する。この exact slice は `BindingId(0)` と source-order
use ordinal 1 を要求し、non-exact definition、reserve、formula、各 expansion
payload は fail closed する。mode declaration acceptance/inhabitation、formula-
side local-mode asserted-head extraction、general reachability/widening/`qua`、
truth/fact、implicit closure/order、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は open のままである。

Task 149 MC-G020 current-state override: exact two-edge object-terminal
local-mode-chain reserved-variable type assertion は task 72 の real AST-derived
`OuterTwoEdgeObjectModeTypeAssertion -> MiddleTwoEdgeObjectModeTypeAssertion -> BaseTwoEdgeObjectModeTypeAssertion -> object`
expansion 3 本と、tasks 122 / 145 / 147 / 148 の identifier-result / 独立した
formula-anchored asserted-type consumer を compose する。raw subject
result は written outer-mode provenance、asserted builtin `object` は独立した
formula source node を保持する。real expansion 3 本が両 input を terminal
definition RHS を canonical source とする builtin-object identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
を記録する。この exact slice は `BindingId(0)` と source-order use ordinal 1 を
要求し、non-exact definition、reserve、formula、各 expansion payload は fail
closed する。分類は `test_gap`、`source_drift`、`design_drift` であり、
`spec_gap` ではない。mode declaration acceptance/inhabitation、formula-side
local-mode asserted-head extraction、general reachability/widening/`qua`、
object/set coercion、truth/fact、implicit closure/order、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は open のままである。production route と
real frontend/resolver sidecar が exact active slice を guard する。

Task 150 MC-G020 current-state override: exact three-edge set-terminal local-mode-
chain reserved-variable type assertion は task 73 の real AST-derived
`OuterThreeEdgeModeTypeAssertion -> MiddleThreeEdgeModeTypeAssertion -> InnerThreeEdgeModeTypeAssertion -> BaseThreeEdgeModeTypeAssertion -> set`
expansion 4 本と、tasks 122 / 148 / 149 の identifier-result / 独立した formula-
anchored asserted-type consumer を compose する。raw subject result は written
outer-mode provenance、asserted builtin `set` は独立した formula source node
を保持しなければならない。real expansion 4 本が両 input を terminal
definition RHS を canonical source とする builtin-set identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
を記録する。この exact slice は `BindingId(0)` と source-order use ordinal 1
を要求し、non-exact definition、reserve、formula、各 expansion payload は
fail closed しなければならない。分類は `test_gap`、`source_drift`、
`design_drift` であり、`spec_gap` ではない。mode declaration acceptance/
inhabitation、formula-side local-mode asserted-head extraction、general
reachability/widening/`qua`、truth/fact、implicit closure/order、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。
production route と real frontend/resolver sidecar が exact active slice を
guard する。

Task 151 MC-G020 current-state override: exact three-edge object-terminal local-mode-
chain reserved-variable type assertion は task 73 の real AST-derived
`OuterThreeEdgeObjectModeTypeAssertion -> MiddleThreeEdgeObjectModeTypeAssertion -> InnerThreeEdgeObjectModeTypeAssertion -> BaseThreeEdgeObjectModeTypeAssertion -> object`
expansion 4 本と、tasks 122 / 149 / 150 の identifier-result / 独立した formula-
anchored asserted-type consumer を compose する。raw subject result は written
outer-mode provenance、asserted builtin `object` は独立した formula source node
を保持しなければならない。real expansion 4 本が両 input を terminal
definition RHS を canonical source とする builtin-object identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
を記録する。この exact slice は `BindingId(0)` と source-order use ordinal 1
を要求し、non-exact definition、reserve、formula、各 expansion payload は fail
closed しなければならない。分類は `test_gap`、`source_drift`、
`design_drift` であり、`spec_gap` ではない。mode declaration acceptance/
inhabitation、formula-side local-mode asserted-head extraction、general
reachability/widening/`qua`、object/set coercion、truth/fact、implicit closure/
order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open のままである。
production route と real frontend/resolver sidecar が exact active slice を
guard する。

Task 152 MC-G020 current-state override: exact four-edge set-terminal local-mode-chain
reserved-variable type assertion は task 74 の real AST-derived
`TooDeepFourEdgeModeTypeAssertion -> OuterFourEdgeModeTypeAssertion -> MiddleFourEdgeModeTypeAssertion -> InnerFourEdgeModeTypeAssertion -> BaseFourEdgeModeTypeAssertion -> set`
expansion 5 本と、tasks 122 / 150 / 151 の identifier-result および独立した
formula-anchored asserted-type consumer を composition する。raw subject result
は written outermost-mode provenance、asserted builtin `set` は独立した formula
source node を保持する。expansion 5 本が両 input を terminal definition RHS を
canonical source とする builtin-set identity 1 個へ再帰的に normalize してから
1 `Inferred` term と 1 fact-free `Checked` type assertion を記録する。この exact
slice は `BindingId(0)` と source-order use ordinal 1 を要求し、non-exact
definition、reserve、formula、各 expansion payload は fail closed する。分類は
`test_gap`、`source_drift`、`design_drift` であり、`spec_gap` ではない。mode
declaration acceptance/inhabitation、formula-side local-mode asserted-head
extraction、general reachability/widening/`qua`、truth/fact、implicit closure/
order、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は open のままで
ある。production route と real frontend/resolver sidecar が exact active slice
を guard する。

Task 153 MC-G020 current-state override: exact four-edge object-terminal local-mode-
chain reserved-variable type assertion は task 74 の real AST-derived
`TooDeepFourEdgeObjectModeTypeAssertion -> OuterFourEdgeObjectModeTypeAssertion -> MiddleFourEdgeObjectModeTypeAssertion -> InnerFourEdgeObjectModeTypeAssertion -> BaseFourEdgeObjectModeTypeAssertion -> object`
expansion 5 本と、tasks 122/151/152 の identifier-result および独立した formula-
side asserted-type consumer を合成する。raw subject result は written
outermost-mode provenance を保持し、asserted builtin `object` は独立した formula
source node を保持する。全 expansion は両 input を terminal definition RHS に
canonical anchor された builtin-object identity 1 個へ再帰的に normalize し、1
`Inferred` term と 1 fact-free `Checked` type assertion を記録する。この exact
slice は `BindingId(0)` と source-order use ordinal 1 を要求し、non-exact
definition/reserve/formula または expansion payload 5 本のいずれかで fail closed
とする。分類は `test_gap`、`source_drift`、`design_drift` で `spec_gap` ではない。
declaration acceptance/inhabitation、formula-side local asserted-head、general
reachability/widening/`qua`、object/set coercion、truth/fact、closure/order、
theorem acceptance、proof/Core/ControlFlow/VC は open のままである。production
route と real sidecar が exact active slice を guard し、active count は 104 件で
ある。

Task 154 MC-G020 implementation specification: exact three-edge set-terminal
local-mode-chain reserved-variable equality は task 73 の real AST-derived
`OuterThreeEdgeModeEquality -> MiddleThreeEdgeModeEquality -> InnerThreeEdgeModeEquality -> BaseThreeEdgeModeEquality -> set`
expansion 4 本と task 134 の equality consumer を合成する。raw result/expected
input 4 個は written outer-mode provenance を保持し、両 operand は ordinal 1、2
で `BindingId(0)` へ解決し、全 role は terminal-RHS builtin-set identity 1 個へ
normalize されてから 2 `Inferred` term と 1 fact/deferred-free `Checked`
equality を記録する。exact definition/radix/expansion corruption、withheld-
family near miss、real frontend/resolver sidecar を必須とする。分類は
`test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。mode
declaration acceptance/inhabitation、equality truth/fact、closure/order、theorem
acceptance、proof/Core/ControlFlow/VC は open のままである。fixture、expectation、
trace row、production route、full near-miss/corruption matrix、real frontend/
resolver sidecar が exact slice を guard し、active count は 105 件である。

Task 155 MC-G020 implementation specification: exact three-edge object-terminal
local-mode-chain reserved-variable equality は task 73 の real AST-derived
`OuterThreeEdgeObjectModeEquality -> MiddleThreeEdgeObjectModeEquality -> InnerThreeEdgeObjectModeEquality -> BaseThreeEdgeObjectModeEquality -> object`
expansion 4 本と task 135 の equality consumer を合成する。raw result/expected
input 4 個は written outer-mode provenance を保持し、両 operand は ordinal 1、2
で `BindingId(0)` へ解決し、全 role は terminal-RHS builtin-object identity 1 個
へ normalize されてから 2 `Inferred` term と 1 fact/deferred-free `Checked`
equality を記録する。exact definition/radix/expansion corruption、withheld-
family near miss、real frontend/resolver sidecar を必須とする。分類は
`test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。mode
declaration acceptance/inhabitation、object/set coercion、equality truth/fact、
closure/order、theorem acceptance、proof/Core/ControlFlow/VC は open のままで
ある。fixture、expectation、trace row、production route、full near-miss/
corruption matrix、real frontend/resolver sidecar が exact slice を guard し、
active count は 106 件である。

Task 156 MC-G020 implementation specification: exact three-edge set-terminal
local-mode-chain reserved-variable inequality は task 73 の real AST-derived
`OuterThreeEdgeModeInequality -> MiddleThreeEdgeModeInequality -> InnerThreeEdgeModeInequality -> BaseThreeEdgeModeInequality -> set`
expansion 4 本と task 136 の pre-desugaring inequality consumer を合成する。
raw result/expected input 4 個は written outer-mode provenance を保持し、両
operand は ordinal 1、2 で `BindingId(0)` へ解決し、全 role は terminal-RHS
builtin-set identity 1 個へ normalize されてから 2 `Inferred` term と 1 fact/
deferred-free pre-desugaring `Checked` inequality を記録する。exact definition/
radix/expansion corruption、withheld-family near miss、real frontend/resolver
sidecar を必須とする。分類は `test_gap`、`source_drift`、`design_drift` であり
`spec_gap` ではない。mode declaration acceptance/inhabitation、inequality
desugaring、truth/fact、closure/order、theorem acceptance、proof/Core/
ControlFlow/VC は open のままである。fixture、expectation、trace row、
production route、full near-miss/corruption matrix、real frontend/resolver
sidecar が exact slice を guard し、active count は 107 件である。

Task 157 MC-G020 implementation specification: exact three-edge object-terminal
local-mode-chain reserved-variable inequality は task 73 の real AST-derived
`OuterThreeEdgeObjectModeInequality -> MiddleThreeEdgeObjectModeInequality -> InnerThreeEdgeObjectModeInequality -> BaseThreeEdgeObjectModeInequality -> object`
expansion 4 本と task 137 の builtin-object pre-desugaring inequality consumer
を合成する。raw result/expected input 4 個は written outer-mode provenance を
保持し、両 operand は ordinal 1、2 で `BindingId(0)` へ解決し、全 role は
terminal-RHS builtin-object identity 1 個へ normalize されてから 2 `Inferred`
term と 1 fact/deferred-free pre-desugaring `Checked` inequality を記録する。
exact definition/radix/expansion corruption、withheld-family near miss、real
frontend/resolver sidecar を必須とする。分類は `test_gap`、`source_drift`、
`design_drift` であり `spec_gap` ではない。mode declaration acceptance/
inhabitation、object/set coercion、inequality desugaring、truth/fact、closure/
order、theorem acceptance、proof/Core/ControlFlow/VC は open のままである。
fixture、expectation、trace row、production route、full near-miss/corruption
matrix、real frontend/resolver sidecar が exact slice を guard し、active count
は 108 件である。

Task 158 MC-G020 implementation specification: exact three-edge set-terminal
local-mode-chain left reserved-variable membership は task 73 の real AST-
derived
`OuterThreeEdgeModeMembership -> MiddleThreeEdgeModeMembership -> InnerThreeEdgeModeMembership -> BaseThreeEdgeModeMembership -> set`
expansion 4 本と task 143 の two-binding right-only expected-set membership
consumer を compose する。raw left result は outer-mode provenance、独立した
right result と sole expected input は explicit-set reserve provenance を保持し、
left expected input は持たない。operand は source-order ordinal 2/3 で
`BindingId(0/1)` へ解決し、全 4 expansion は 3 role を terminal-RHS builtin-
set identity 1 個へ normalize してから 2 `Inferred` term と exactly one right-
owned constraint を持つ 1 fact/deferred-free `Checked` membership を記録する。
independent definition/radix/expansion corruption、withheld-family near miss、
real frontend/resolver sidecar を必須とする。分類は `test_gap`、
`source_drift`、`design_drift` であり `spec_gap` ではない。mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof/Core/ControlFlow/VC、object-terminal behavior、broader depth
は open のままである。fixture、expectation、trace row、production route、
full near-miss/corruption matrix、real frontend/resolver sidecar が exact slice
を guard し、active count は 109 件である。

Task 159 MC-G020 current-state override: exact distinct-binding shared-
reserve membership は task 123 の one-item/two-binding/shared-range producer と
tasks 120/125 の right-only expected-set membership consumer を compose する。
`x/y` は ordinal 2/3 で `BindingId(0/1)` へ解決され、left result、right result、
sole right expected input は written set range 1 個を保持し、left expected input
はなく、3 role は builtin-set identity 1 個へ normalize してから 2 `Inferred`
term と right-owned constraint 1 個を持つ 1 fact/deferred-free `Checked`
membership を記録する。分類は `test_gap`、`source_drift`、`design_drift` で
`spec_gap` ではない。truth/fact、closure/order、theorem acceptance、proof/Core/
ControlFlow/VC、separate declaration、broader shape は open のままである。
fixture/expectation/trace row、production route、full near-miss/corruption
matrix、real sidecar が exact slice を guard するため active count は 110 件である。

Task 160 MC-G020 current-state override: exact distinct-binding shared-reserve
inequality は task 123 の one-item/two-binding/shared-range producer と task 121 の
pre-desugaring inequality consumer を compose する。`x/y` は ordinal 2/3 で
`BindingId(0/1)` へ解決され、両 binding と left/right result/expected role 4 個は
written set range 1 個を保持して shared-source builtin-set identity 1 個へ
normalize してから、2 `Inferred` term と 2 ordered operand-owned constraint を
持つ 1 fact/deferred-free `Checked` inequality を記録しなければならない。分類は
`test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。
desugaring/truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、
separate declaration、broader shape は open のままである。test-first fixture/
expectation/trace contract、production route、near-miss/corruption guard、real
sidecar が exact slice を保護するため active count は 111 件である。

Task 161 MC-G020 current-state override: exact multiple-reserve-declaration
inequality は task 124 の two-item/two-binding/distinct-written-range producer と
task 160 の pre-desugaring inequality consumer を compose する。`x/y` は ordinal
2/3 で `BindingId(0/1)` へ解決され、各 operand の result/expected role は固有の
written set range を保持しながら全 4 role を earlier `x` range に canonical anchor
された builtin-set identity 1 個へ normalize し、2 `Inferred` term と 2 ordered
constraint を持つ 1 fact/deferred-free `Checked` inequality を記録しなければならない。
分類は `test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。
desugaring/truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、
shared range、broader shape は open のままである。source/trace contract、production
route、near-miss/corruption guard、real sidecar が exact slice を保護するため active
count は 112 件である。

Task 162 MC-G020 current-state override: exact multiple-reserve-declaration
membership は task 124 の two-item/two-binding/distinct-written-range producer と
task 120/159 の right-only expected-set membership consumer を compose する。
`x/y` は ordinal 2/3 で `BindingId(0/1)` へ解決され、left result は first written
set range、right result と sole right expected input は second range を保持し、left
expected input は持たない。3 role は earlier `x` range に canonical anchor された
builtin-set identity 1 個へ normalize してから、2 `Inferred` term と exactly one
right-owned constraint を持つ 1 fact/deferred-free `Checked` membership を記録する。
分類は `test_gap`、`source_drift`、`design_drift` であり `spec_gap` ではない。
membership truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/
VC、shared range、broader shape は open のままである。fixture/expectation/trace
contract、production routing、near-miss/corruption guard、real sidecar が exact
slice を保護するため active count は 113 件である。

Task 163 MC-G020 current-state override: exact three-edge local-object-mode-chain
left membership を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.5.3、16 は test-first
source を直接支える。task 73/151/155/157 は real four-expansion object-terminal
producer、task 144 は real object-left/set-right membership consumer を提供する。
実装は ordinal 2/3 の `BindingId(0/1)`、raw outer-mode left
result、独立した explicit-set right result / sole right expected input、no left
expected input、distinct terminal-object-RHS / explicit-set identity、2
`Inferred` term、exactly one right-owned constraint を持つ 1 fact/deferred-free
`Checked` membership を保持しなければならない。truth/fact、object/set
coercion、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、他の
depth、broader shape は open のままである。production routing、near-miss/
corruption guard、real frontend/resolver sidecar が active runner 114 を保護する。

Task 164 MC-G020 current-state override: exact four-edge set-terminal
local-mode-chain left membership を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 4、7、13、14.5.3、16
が test-first source を直接支え、task 74/152 が real five-expansion producer、
task 158 が real set-left/set-right membership consumer を提供する。contract
は ordinal 2/3 の `BindingId(0/1)`、raw outermost-mode left provenance、独立
した explicit-set right result/sole expected provenance、no left expected
input、source-derived expansion 5 本、terminal-set-RHS identity 1 個、2
`Inferred` term、exactly one right-owned constraint を持つ 1 fact/deferred-free
`Checked` membership を要求する。truth/fact、closure/order、theorem
acceptance、proof/Core/ControlFlow/VC、object-terminal behavior、他 depth、
broader shape は open のままである。fixture/expectation、6 trace backlink、
exact route、corruption/near-miss coverage、real sidecar が active runner 115
を保護する。

Task 165 MC-G020 current-state override: exact four-edge object-terminal local-
mode-chain left membership を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.5.3、16 が test-first
source を直接支える。task 74/153 が real five-expansion object-terminal
producer、task 163 が real object-left/set-right membership consumer を提供
する。exact contract は ordinal 2/3 の `BindingId(0/1)`、raw outermost-mode
left provenance、独立した explicit-set right result / sole expected provenance、
no left expected input、source-derived expansion 5 本、distinct terminal-
object-RHS / explicit-set identity、2 `Inferred` term、exactly one right-owned
constraint を持つ 1 fact/deferred-free `Checked` membership を保持する。
truth/fact、object/set coercion、closure/order、theorem acceptance、proof/Core/
ControlFlow/VC、他 depth、broader shape は open のままとする。fixture、
expectation、trace backlink 6 件、exact production route、corruption/near-miss
coverage、real frontend/resolver sidecar が active runner 116 を保護する。

Task 166 MC-G020 current-state override: exact four-edge set-terminal
local-mode-chain equality を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 4、7、13、14.5.2、16 が test-first source
を支え、task 74/152 が real five-expansion producer、task 154 が equality
consumer を提供する。exact contract は ordinal 1/2 の `BindingId(0)`、raw
outermost-mode result/expected input 4 個、source-derived expansion 5 本、
terminal-set-RHS identity 1 個、2 `Inferred` term、1 fact/deferred-free
`Checked` equality、ordered operand-owned expected constraint 2 個を保持する。
fixture、trace backlink 6 件、exact production route、corruption matrix、real
frontend/resolver sidecar が active runner 117 を保護する。declaration acceptance/inhabitation、
truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、
object-terminal behavior、他 depth、broader shape は open のままである。

Task 167 MC-G020 current-state override: exact four-edge object-terminal
local-mode-chain equality を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.5.2、16 が test-first
source を支え、tasks 74/153 が real five-expansion producer、task 155 が
equality consumer を提供する。exact contract は ordinal 1/2 の
`BindingId(0)`、raw outermost-mode result/expected input 4 個、source-derived
expansion 5 本、terminal-object-RHS identity 1 個、2 `Inferred` term、1 fact/
deferred-free `Checked` equality、ordered operand-owned expected constraint 2
個を object/set coercion なしで保持する。fixture、trace backlink 6 件、
production route、full corruption matrix、real frontend/resolver sidecar が
active runner 118 を保護する。declaration
acceptance/inhabitation、truth/fact、closure/order、theorem acceptance、proof/
Core/ControlFlow/VC、set-terminal behavior、他 depth、broader shape は open の
ままである。

Task 168 MC-G020 current-state override: exact four-edge set-terminal local-mode-chain
inequality を `spec_gap` ではなく `test_gap`、`source_drift`、`design_drift`
と分類する。Chapter 4、7、13、14.5.2、16 が test-first source を支え、
tasks 74/152 が real five-expansion producer、task 156 が pre-desugaring
inequality consumer を提供する。exact contract は ordinal 1/2 の
`BindingId(0)`、raw outermost-mode result/expected input 4 個、source-derived
expansion 5 本、terminal-set-RHS identity 1 個、2 `Inferred` term、1 fact/
deferred-free pre-desugaring `Checked` inequality、ordered operand-owned
expected constraint 2 個を保持する。fixture、trace backlink 6 件、production
routing、full corruption coverage、real sidecar が active runner 119 を保護する。
declaration acceptance/inhabitation、desugaring/truth/
fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC、object-
terminal behavior、他 depth、broader shape は open のままである。

Task 169 MC-G020 current-state override: exact four-edge object-terminal local-mode-chain
inequality を `spec_gap` ではなく `test_gap`、`source_drift`、`design_drift`
と分類する。Chapter 3、4、7、13、14.5.2、16 が test-first source を支え、
tasks 74/153 が real five-expansion producer、task 157 が pre-desugaring
inequality consumer を提供する。exact contract は ordinal 1/2 の
`BindingId(0)`、raw outermost-mode result/expected input 4 個、source-derived
expansion 5 本、terminal-object-RHS identity 1 個、2 `Inferred` term、1 fact/
deferred-free pre-desugaring `Checked` inequality、ordered operand-owned
expected constraint 2 個を object/set coercion なしで保持する。fixture、trace
backlink 6 件、production routing、full corruption coverage、real sidecar が
active runner 120 を保護する。declaration acceptance/
inhabitation、desugaring/truth/fact、closure/order、theorem acceptance、proof/
Core/ControlFlow/VC、set-terminal behavior、他 depth、broader shape は open の
ままである。

Task 172 MC-G020 current-state override: exact set-terminal local-mode long-chain
reserved-variable equality を `spec_gap` ではなく `test_gap`、`source_drift`、
`design_drift` と分類する。Chapter 4、7、13、14.5.2、16 が test-first source
を支え、task 74 が active long-chain fixture で既に実行する real seven-
expansion producer、task 166 が equality consumer を提供する。contract は
ordinal 1/2 の `BindingId(0)`、raw `ChainMode6` result/expected input 4 個、
source-derived expansion 7 本、terminal `BaseMode` RHS の builtin-set identity
1 個、2 `Inferred` term、1 fact/deferred-free `Checked` equality、ordered
operand-owned expected constraint 2 個を保持する。exact source routing、full
near-miss / corruption coverage、real frontend/resolver sidecar が active
runner 121 を保護する。
declaration acceptance/inhabitation、truth/fact、closure/order、theorem
acceptance、proof/Core/ControlFlow/VC payload、imported/attributed/argument-
bearing または別 chain shape、general unbounded semantics は open のままで
ある。

Task 173 MC-G020 current-state override: exact set-terminal local-mode long-chain
inequality を `spec_gap` ではなく `test_gap`、`source_drift`、`design_drift`
と分類する。Chapter 4、7、13、14.5.2、16 が source を支え、task 74 が real
expansion 7 本、task 168 が real pre-desugaring inequality consumer を提供する。
raw `ChainMode6` role 4 個、ordinal 1/2 の `BindingId(0)`、terminal `BaseMode`
RHS identity 1 個、2 `Inferred` term、1 fact/deferred-free checked inequality、
ordered constraint 2 個を保持する。exact routing、full guard、real sidecar が
active runner 122 を保護する。desugaring/truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 174 MC-G020 current-state override: exact set-terminal local-mode long-chain left
reserved-variable membership を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 4、7、13、14.5.3、16 が
test-first source を支え、task 74 が real seven-expansion producer、task 164 が
right-only expected-set membership consumer を提供する。raw `ChainMode6` left
result、独立した explicit-set right result と sole right expected input、ordinal
2/3 の `BindingId(0/1)`、terminal `BaseMode` RHS identity 1 個、left expected
input なし、2 `Inferred` term、1 fact/deferred-free checked membership、right-
owned constraint 1 個を保持する。exact routing、membership-specific corruption
test、shared full structural guard、real sidecar が active runner 123 を保護する。
membership truth/fact、acceptance、closure/order、theorem/proof/Core/
ControlFlow/VC、別 chain、general semantics は open のままである。

Task 175 MC-G020 current-state override: exact set-terminal local-mode long-chain
reserved-variable type assertion を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 3、4、7、13、14.2.3、16 が
test-first source を支え、task 74 が real seven-expansion producer、task 152 が
normalized-reflexive type-assertion consumer を提供する。raw `ChainMode6`
subject result、独立した formula-side builtin-set asserted input、ordinal 1 の
`BindingId(0)`、terminal `BaseMode` RHS identity 1 個、1 `Inferred` term、general
reachability を用いない 1 fact/deferred-free checked type assertion を保持する。
exact production route、task 172 の shared structural guard、type-assertion-
specific corruption test、real frontend/resolver sidecar が active runner 124 を
保護する。widening/`qua`、assertion truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 176 MC-G020 current-state override: exact builtin-object-terminal local-mode
long-chain reserved-variable equality を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 3、4、7、13、14.5.2、16 が
test-first source を支え、task 74 が real AST-bounded object-terminal chain
producer、task 167 が object-normalizing equality consumer を提供する。raw
`ChainObjectMode6` result/expected input 4 個、ordinal 1/2 の `BindingId(0)`、
real expansion 7 本、terminal `BaseObjectMode` RHS identity 1 個、2 `Inferred`
term、1 fact/deferred-free checked equality、ordered constraint 2 個を object/set
coercion なしで保持する。exact production route、task 172 の shared structural
guard、object-specific
corruption test、real frontend/resolver sidecar が active runner 125 を保護する。
truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 177 MC-G020 current-state override: exact builtin-object-terminal local-mode
long-chain reserved-variable inequality を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 3、4、7、13、14.5.2、16 が
test-first source を支え、task 74 が real AST-bounded object-terminal chain
producer、task 169 が object-normalizing pre-desugaring inequality consumer を
提供する。raw `ChainObjectMode6` result/expected input 4 個、ordinal 1/2 の
`BindingId(0)`、real expansion 7 本、terminal `BaseObjectMode` RHS identity 1 個、
2 `Inferred` term、1 fact/deferred-free checked inequality、ordered constraint 2
個を object/set coercion なしで保持する。exact production route、task 172 の
shared structural guard、object-specific corruption test、real frontend/resolver
sidecar が active runner 126 を保護する。desugaring、truth/fact、
acceptance、closure/order、theorem/proof/Core/ControlFlow/VC、別 chain、general
semantics は open のままである。

Task 178 MC-G020 current-state override: exact builtin-object-terminal local-mode
long-chain left reserved-variable membership を `spec_gap` ではなく `test_gap`、
`source_drift`、`design_drift` と分類する。Chapter 3、4、7、13、14.5.3、16 が
test-first source を支え、task 74 が real AST-bounded object-terminal chain
producer、task 165 が object-left/set-right membership consumer を提供する。raw
`ChainObjectMode6` left result、独立した explicit-set right result/sole expected
input、ordinal 2/3 の `BindingId(0/1)`、real expansion 7 本、distinct terminal-
object-RHS と explicit-set identity、left expected input なし、2 `Inferred`
term、1 fact/deferred-free checked membership、right-owned constraint 1 個を
object/set coercion なしで保持する。exact route、task 172 shared guard、
membership/object-specific corruption test、real sidecar が active runner 127 を
保護する。truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 179 MC-G020 current-state override: exact builtin-object-terminal local-mode
long-chain reserved-variable normalized-reflexive type assertion を `spec_gap`
ではなく `test_gap`、`source_drift`、`design_drift` と分類する。Chapter 3、4、
7、13、14.2.3、16 が test-first source を支え、task 74 が real AST-bounded
object-terminal chain producer、task 153 が object-normalizing type-assertion
consumer、task 175 が seven-expansion sibling guard pattern を提供する。raw
`ChainObjectMode6` subject result、独立した formula-side builtin-object asserted
input、ordinal 1 の `BindingId(0)`、real expansion 7 本、terminal-object-RHS
identity 1 個、1 `Inferred` term、general reachability と object/set coercion を
用いない 1 fact/deferred-free normalized-reflexive checked type assertion を
保持する。exact route、task 172 shared guard、task 153 の real object consumer/
source near miss、builtin-set asserted head または raw subject provenance
corruption を reject するよう task 175 から適応した matched-output guard、real
sidecar が active runner 128 を保護する。widening/`qua`、truth/fact、acceptance、closure/order、theorem/proof/
Core/ControlFlow/VC、別 chain、general semantics は open のままである。

Task 180 MC-G020 current-state override: exact standalone
`SourceDerivedContradictionConstantBoundary: contradiction` leaf を `spec_gap`
ではなく `test_gap`、`source_drift`、`design_drift` と分類する。Chapter 14、16
は leaf と theorem slot を直接支える。新規 exact standalone extractor は既存
contradiction-kind mapping と theorem-shape validation pattern を再利用し、real
leaf site/range を module-root context で deferred reason なしに既存 checker
consumer へ渡す。term/type/constraint/candidate/fact/deferred/diagnostic payload
が空の 1 checked formula に type/well-formedness credit を与える。exact/near-
miss/corruption と real-sidecar guard が active runner 129 を保護する。
falsehood/fact publication、theorem acceptance、proof-goal closure、implicit
closure/child graph、`formula_statement`、proof、CoreIr、ControlFlowIr、VC は
open のままである。

Task 182 MC-G020 current-state override: exact direct formula-side local-mode
asserted head は `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` を閉じる。Chapter 3、4、7、13、14.2.3、16 は `mode
LocalModeAssertedHeadDef: LocalModeAssertedHead is set;` を含む exact definition
block と test-first source を直接支える。task 55 は
real direct bare-mode expansion producer、tasks 122/138 は normalized-reflexive
type-assertion consumer と direct local-mode subject route を提供する。exact source
は同じ resolved mode 向けの独立した reserve-subject と formula-side asserted raw
input、real expansion 1 個、ordinal 1 の `BindingId(0)`、terminal-RHS builtin-set
identity 1 個へ intern する known type entry 3 個、1 inferred term、general
reachability を用いない 1 fact/deferred-free checked formula を保持する。
exact/near-miss/corruption、production-route、real sidecar guard が active runner
130 を保護する。mode declaration acceptance/
inhabitation、widening/`qua`、truth/fact、theorem/proof/Core/ControlFlow/VC、
他 asserted head/chain、general semantics は open のままである。

Task 183 MC-G020 current-state override: exact direct object-terminal formula-
side local-mode asserted head は `spec_gap` ではなく `test_gap`、narrow
`source_drift`、`design_drift` を閉じる。Chapter 3、4、7、13、14.2.3、16 は `mode
LocalObjectModeAssertedHeadDef: LocalObjectModeAssertedHead is object;` を持つ
exact source を直接支える。task 55 は real bare-object mode expansion、tasks
145/182 は normalized object consumer と same-symbol formula-side asserted-head
producer を提供する。exact route は独立した raw reserve-subject/formula-side
asserted input、real expansion 1 個、ordinal 1 の `BindingId(0)`、terminal-RHS
builtin-object identity 1 個へ intern する known type entry 3 個、1 inferred term、
general reachability と object/set coercion を用いない 1 fact/deferred-free checked
formula を保持する。exact/near-miss/corruption、production-route、real sidecar
guard が active runner 131 を保護する。
declaration acceptance/inhabitation、truth/fact、theorem/proof/Core/ControlFlow/
VC、他 asserted head/chain、general semantics は open のままである。

Task 184 MC-G020 current-state override: exact one-edge set-terminal same-outer-mode
asserted head は `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` である。Chapter 3、4、7、13、14.2.3、16 は
`mode BaseModeAssertedHeadDef: BaseModeAssertedHead is set;` と `mode
ChainModeAssertedHeadDef: ChainModeAssertedHead is BaseModeAssertedHead;` を
含む ordered definition block 2 個、outer-mode reserve 1 個、
`ChainedLocalModeAssertedHeadPayloadBoundary: x is ChainModeAssertedHead;`
から成る test-first source を直接支える。task 56 は real one-edge expansion
producer、tasks 146/182 は normalized set consumer と same-symbol formula-side
asserted-head producer を提供する。exact route は同じ outer symbol 向けの独立した
raw reserve-subject/formula-side asserted input、real expansion 2 個、ordinal 1 の
`BindingId(0)`、terminal base-definition-RHS builtin-set identity 1 個へ intern
する known type entry 3 個、1 inferred term、general reachability を用いない
1 fact/deferred-free checked formula を保持する。exact/near-miss/corruption、
production-route、real sidecar guard が active runner 132 を保護する。
declaration acceptance/inhabitation、widening/`qua`、truth/fact、closure/order、
theorem/proof/Core/ControlFlow/VC、object/deeper/他 asserted head、general chain
semantics は open のままである。

Task 185 MC-G020 current-state override: exact one-edge object-terminal same-outer-mode
asserted head を `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.2.3、16 は `mode
BaseObjectModeAssertedHeadDef: BaseObjectModeAssertedHead is object;` と `mode
ChainObjectModeAssertedHeadDef: ChainObjectModeAssertedHead is
BaseObjectModeAssertedHead;` を含む ordered definition block 2 個、outer-mode
reserve 1 個、`ChainedLocalObjectModeAssertedHeadPayloadBoundary: x is
ChainObjectModeAssertedHead;` から成る test-first source を直接支える。task 56
は real one-edge expansion producer、tasks 147/183/184 は normalized object
consumer と same-symbol recursive formula-side asserted-head seam を提供する。
exact route は同じ outer symbol 向けの独立した raw reserve-subject/formula-side
asserted input、real expansion 2 個、ordinal 1 の `BindingId(0)`、terminal base-
definition-RHS builtin-object identity 1 個へ intern する known type entry 3 個、
1 inferred term、general reachability、widening、`qua`、object/set coercion を
用いない 1 fact/deferred-free checked formula を保持する。exact/near-miss/
corruption、production-route、real sidecar guard が active runner 133 を保護する。
shared trace backlink 5 個と dedicated row 1 個が exact credit を担う。imported
provenance、declaration/attribute acceptance、
broader term/formula、child graph、theorem/proof/Core/ControlFlow/VC、deeper/他
asserted head、general chain semantics は open のままである。module layout 更新は
不要である。

Task 186 MC-G020 current-state override: exact two-edge set-terminal same-outer-
mode asserted head を `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.2.3、16 は `mode
BaseTwoEdgeModeAssertedHeadDef: BaseTwoEdgeModeAssertedHead is set;`、`mode
MiddleTwoEdgeModeAssertedHeadDef: MiddleTwoEdgeModeAssertedHead is
BaseTwoEdgeModeAssertedHead;`、`mode OuterTwoEdgeModeAssertedHeadDef:
OuterTwoEdgeModeAssertedHead is MiddleTwoEdgeModeAssertedHead;` を含む ordered
definition block 3 個、`OuterTwoEdgeModeAssertedHead` の reserve 1 個、
`TwoEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeModeAssertedHead;` を直接支える。task 72 は real two-edge expansion
producer、tasks 148/184 は normalized set consumer と same-symbol formula-side
asserted-head seam、task 185 は object-terminal sibling guard を提供する。exact
route は同じ resolved outer symbol 向けの distinct raw reserve-subject/formula-
side asserted site/range、real expansion 3 個、ordinal 1 の `BindingId(0)`、terminal
`BaseTwoEdgeModeAssertedHead` definition RHS builtin-set identity 1 個へ intern
する known type entry 3 個、reachability、widening、`qua` を用いない 1
`Inferred` term と 1 fact/deferred-free normalized-reflexive `Checked` formula を
保持しなければならない。exact/near-miss/corruption、production-route、real
frontend/resolver-sidecar guard は各 missing expansion、wrong link/terminal/order/
depth、duplicate、forward/recovered/contextual/parameterized/argument-bearing/
attributed definition、direct/one-edge/deeper/object-terminal shape、non-exact
reserve/formula、wrong subject、builtin/base/middle/other/attributed/argument-
bearing asserted head、imported/ambiguous provenance、recovery、extra item、
collapsed provenance、builtin-object output corruption を reject する。shared
trace backlink 5 個と dedicated row 1 個が existing expectation を rebaseline
せず active runner 134 を保護する。object-terminal/deeper/
imported asserted head、declaration/attribute acceptance、broader term/formula/
child graph、truth/fact、theorem/proof/Core/ControlFlow/VC、general chain
semantics は open のままである。module layout 更新は不要である。

Task 187 MC-G020 current-state override: exact two-edge object-terminal same-outer-
mode asserted head を `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` と分類する。Chapter 3、4、7、13、14.2.3、16 は ordered
definition `mode BaseTwoEdgeObjectModeAssertedHeadDef:
BaseTwoEdgeObjectModeAssertedHead is object;`、`mode
MiddleTwoEdgeObjectModeAssertedHeadDef: MiddleTwoEdgeObjectModeAssertedHead is
BaseTwoEdgeObjectModeAssertedHead;`、`mode OuterTwoEdgeObjectModeAssertedHeadDef:
OuterTwoEdgeObjectModeAssertedHead is MiddleTwoEdgeObjectModeAssertedHead;`、
outer-mode reserve 1 個、`TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeObjectModeAssertedHead;` を支える。tasks 72/149 は real object-
terminal producer/consumer、tasks 185/186 は same-symbol asserted-head route
pattern を提供する。exact route は同じ local outer symbol 向けの distinct raw
subject/asserted site/range、real expansion 3 個、ordinal 1 の `BindingId(0)`、
terminal base-definition-RHS builtin-object identity 1 個へ intern する known
entry 3 個、expected constraint、reachability、widening、`qua`、object/set
coercion を持たない 1 `Inferred` term と 1 fact/deferred-free normalized-
reflexive `Checked` assertion を保持する。exact/near-miss/
corruption、production-route、real frontend/resolver-sidecar guard は missing/
wrong/duplicate/reordered/forward/recovered/contextual/parameterized/argument-
bearing/attributed link または wrong label、set-terminal/direct/one-edge/deeper
shape、builtin/base/middle/other asserted head および attributed/argument-bearing
formula-side asserted head、imported Base/Middle/Outer と imported/ambiguous
asserted provenance、extra/collapsed provenance、`BuiltinSet` output corruption
を reject する。shared trace backlink 5 個と dedicated row 1 個により active
runner 135 を保護する。positive imported semantics、declaration/attribute
acceptance、broader term/formula/child graph、truth/fact、implicit closure/order、
theorem acceptance、proof/Core/ControlFlow/VC、general chain semantics は open
のままである。Step 5 は active、
Steps 6/7 は deferred のまま。module layout 更新は不要である。

Task 188 MC-G020 current-state override: exact builtin-object same-binding
equality を `spec_gap` ではなく `test_gap`、narrow `source_drift`、
`design_drift` と分類する。Chapter 3、4、13、14.5.2、16 は `reserve x for
object; theorem ReservedObjectVariableEqualityPayloadBoundary: x = x;` を直接
支える。tasks 48/125 は real builtin-object reserve handoff、task 119 は exact
same-binding equality route/builder、task 128 は real builtin-object normalization
consumer を提供する。route は ordinal 1/2 を `BindingId(0)` に解決し、written
`object` range 1 個上の distinct result/expected role site 4 個、canonical
builtin-object identity 1 個、`Inferred` variable 2 個、ordered expected constraint
2 個、fact/deferred-free `Checked` equality 1 個を保持しなければならない。
exact/near-miss、matched-output、canonical-source、`BuiltinSet` corruption、
route-order、real frontend/resolver-sidecar guard を必須とする。shared trace
backlink 5 個 + dedicated row 1 個により既存 expectation を変更せず active runner
136 を保護する。object/set coercion、general/non-reflexive object
equality、truth/fact、implicit closure/order、theorem acceptance、proof/Core/
ControlFlow/VC、broader source shape は open のままである。Step 5 は active、
Steps 6/7 は deferred のまま。module layout 更新は不要である。

Task 189 MC-G020 current-state override: exact builtin-object same-binding
normalized-reflexive type assertion を `spec_gap` ではなく `test_gap`、narrow
`source_drift`、`design_drift` と分類する。Chapter 3、4、13、14.2.3、16 は
`reserve x for object; theorem
ReservedObjectVariableTypeAssertionPayloadBoundary: x is object;` を直接支える。
tasks 48/125/188 は real builtin-object reserve handoff、task 122 は exact one-
subject assertion route/builder、task 145 は real builtin-object normalization
consumer を提供する。route は ordinal 1 を `BindingId(0)` に解決し、distinct
reserve-subject result/formula-side asserted site/range、written reserve type を
anchor とする canonical builtin-object identity 1 個、`Inferred` variable 1 個、
known type entry 3 個、expected constraint 0 個、fact/deferred-free `Checked`
assertion 1 個を保持しなければならない。exact/near-miss、matched-output、
canonical-source、`BuiltinSet` corruption、route-order、real frontend/resolver-
sidecar guard を必須とする。shared trace backlink 5 個 + dedicated row 1 個により
既存 expectation を変更せず active runner 137 を保護する。
reachability/widening/`qua`、object/set coercion、truth/fact、implicit closure/
order、theorem acceptance、proof/Core/ControlFlow/VC、broader source shape は open
のままである。Step 5 は active、Steps 6/7 は deferred のまま。module layout
更新は不要であった。

Task 190 MC-G020 current-state override: exact builtin-object same-binding inequality
は `test_gap`、narrow `source_drift`、`design_drift` であり、`spec_gap` ではない。
Chapters 3、4、13、14.5.2、16 は `reserve x for object; theorem
ReservedObjectVariableInequalityPayloadBoundary: x <> x;` を直接 support する。
Tasks 48/125/188 は real builtin-object reserve handoff と canonical
normalization producer を提供し、Task 121 は real exact same-binding pre-
desugaring inequality consumer を提供する。Task 128 も builtin-object
normalization consumer を独立に実証する。route は ordinal 1/2 を
`BindingId(0)` へ解決し、written `object` range 1 個上の distinct result/
expected role site 4 個を保持し、canonical builtin-object identity 1 個へ
intern し、`Inferred` variable 2 個、known type entry 6 個、ordered expected
constraint 2 個、fact/candidate/diagnostic/deferred-free `Checked` inequality
1 個を記録しなければならない。exact/near-miss、matched-output、canonical-
source、`BuiltinSet` corruption、route-order、real frontend/resolver-sidecar
guard を備える。shared trace backlink 5 個 + dedicated row 1 個により、
既存 expectation を変更せず active runner 138 を保護する。inequality
desugaring/equality truth、object/set coercion、fact、implicit closure/order、
theorem acceptance、proof/Core/ControlFlow/VC、broader source shape は open の
ままである。Step 5 は active、Steps 6/7 は deferred のまま。checker source
または module layout 更新は不要であった。

Task 191 MC-G020 current-state override: exact distinct-binding shared-
builtin-object equality は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、13、14.5.2、
16 は `reserve x, y for object; theorem
DistinctReservedObjectVariableEqualityPayloadBoundary: x = y;` を直接
support する。task 123 は real one-item/two-binding shared-written-range
producer を提供し、tasks 48/125/188 は real builtin-object reserve、
normalization、equality consumer を提供する。route は ordinal 2/3 を
`BindingId(0/1)` に解決し、両 binding と distinct result/expected role site
4 個に shared written `object` range 1 個を保持し、その reserve range を
anchor とする canonical builtin-object identity 1 個、`Inferred` variable
2 個、known type entry 6 個、operand-owned ordered expected constraint 2 個、
fact/candidate/diagnostic/deferred-free `Checked` equality 1 個を記録しなければ
ならない。exact/near-miss、matched-output、canonical-source、`BuiltinSet`
corruption、route-order、real frontend/resolver sidecar を備える。shared
trace backlink 5 個 + dedicated row 1 個により既存 expectation を変更せず
active runner 139 を保護する。equality truth、object/set coercion、
fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC、
broader distinct-object source shape は open のままである。Step 5 は active、
Steps 6/7 は deferred のまま。checker source または module-layout change は
不要であった。

Task 192 MC-G020 current-state override: exact distinct-binding shared-
builtin-object inequality は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、13、14.5.2、
16 は `reserve x, y for object; theorem
DistinctReservedObjectVariableInequalityPayloadBoundary: x <> y;` を直接
support する。tasks 123/191 は real one-item/two-binding shared-written-range
builtin-object producer を提供し、tasks 121/160/190 は real pre-desugaring
inequality consumer を提供する。route は ordinal 2/3 を `BindingId(0/1)` に
解決し、両 binding と distinct result/expected role site 4 個に shared written
`object` range 1 個を保持し、その reserve range を anchor とする canonical
builtin-object identity 1 個、`Inferred` variable 2 個、known type entry 6 個、
operand-owned ordered expected constraint 2 個、fact/candidate/diagnostic/
deferred-free `Checked` inequality 1 個を記録する。shared trace backlink 5 個 +
dedicated row 1 個により既存 expectation を変更せず active runner 140 を
保護する。exact/near-miss、matched-output、canonical-source、`BuiltinSet`
corruption、route-order、real frontend/resolver sidecar が contract を guard する。
inequality desugaring/equality truth、object/set coercion、fact、implicit closure/order、
theorem acceptance、proof/Core/ControlFlow/VC、broader distinct-object source
shape は open のままである。Step 5 は active、Steps 6/7 は deferred のまま。
checker source または module-layout change は不要であった。

Task 193 MC-G020 current-state override: exact multiple-reserve-declaration
builtin-object equality は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、13、14.5.2、
16 は `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationEqualityPayloadBoundary: x = y;` を直接
support する。Task 124 は real two-item/two-binding/distinct-written-range
producer を提供し、tasks 188/191 は real builtin-object equality consumer を
提供する。route は ordinal 2/3 を `BindingId(0/1)` に解決し、distinct written
`object` range 2 個を distinct result/expected role site 4 個に保持し、先行する
`x` reserve range を anchor とする canonical builtin-object identity 1 個、
`Inferred` variable 2 個、known type entry 6 個、operand-owned ordered expected
constraint 2 個、fact/candidate/diagnostic/deferred-free `Checked` equality 1 個を
記録する。shared trace backlink 5 個 + dedicated row 1 個により既存 expectation
を変更せず active runner 141 を保護する。exact structural/provenance near
miss、matched-output、canonical-source、`BuiltinSet` corruption、route
isolation、real frontend/resolver sidecar が contract を guard する。equality
truth、object/set coercion、fact、implicit closure/order、theorem acceptance、
proof/Core/ControlFlow/VC、shared-range shape、broader multiple-reserve object
shape は open のままである。Step 5 は active、Steps 6/7 は deferred のまま。
checker source または module-layout change は不要であった。

Task 194 MC-G020 current-state override: exact multiple-reserve-declaration
builtin-object inequality は `test_gap`、narrow `source_drift`、
`design_drift` であり、`spec_gap` ではない。Chapters 3、4、13、14.5.2、
16 は `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationInequalityPayloadBoundary: x <> y;` を直接
support する。Task 193 は real ordered two-item/two-binding/distinct-written-
object-range producer を提供し、tasks 190/192 は real pre-desugaring builtin-
object inequality consumer を提供する。route は ordinal 2/3 を
`BindingId(0/1)` に解決し、binding ごとの written `object` range 2 個を
distinct raw result/expected role 4 個に保持し、先行する `x` range を anchor
とする canonical builtin-object identity 1 個、`Inferred` variable 2 個、known
type entry 6 個、operand-owned ordered expected constraint 2 個、fact/candidate/
diagnostic/deferred-free `Checked` inequality 1 個を記録する。shared backlink 5
個 + dedicated row 1 個により既存 expectation を変更せず active runner 142 を
保護する。exact structural/provenance near miss、raw/canonical-source その他の
corruption probe、route isolation、immutable-output check、real frontend/
resolver sidecar が contract を guard する。inequality desugaring/equality
truth、object/set coercion、fact、implicit closure/order、theorem acceptance、
proof/Core/ControlFlow/VC、shared-range shape、broader multiple-reserve object
shape は open のままである。Step 5 は active、Steps 6/7 は deferred のまま。
checker source または module-layout change は不要であった。

Task 195 MC-G020 current-state override: exact three-edge set-terminal same-
outer-mode asserted head は `test_gap`、narrow `source_drift`、`design_drift`
であり、`spec_gap` ではない。Chapters 3、4、7、13、14.2.3、16 は ordered
mode definition 4 個 `Outer -> Middle -> Inner -> Base -> set`、`reserve x for
OuterThreeEdgeModeAssertedHead`、`ThreeEdgeLocalModeAssertedHeadPayloadBoundary:
x is OuterThreeEdgeModeAssertedHead;` を直接 support する。Task 73 は real
four-expansion producer、Task 150 は同じ深さの subject-side normalization、Task
186 は same-symbol formula-side asserted-head consumer を提供する。exact route
は outer symbol の distinct raw subject/asserted site/range を保持し、ordinal 1
を `BindingId(0)` に解決し、AST-derived expansion 4 個を消費し、known type
entry 3 個を base-definition-RHS anchor の `BuiltinSet` identity 1 個へ
normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/
candidate/diagnostic/deferred-free normalized-reflexive `Checked` type assertion
1 個を記録する。shared backlink 5 個 + dedicated row 1 個により既存
expectation を変更せず active runner 143 を保護する。structural/provenance
near miss は unrelated local、imported、ambiguous asserted head を含み、
corruption、immutable-output、route-isolation、real frontend/resolver sidecar
guard が contract を完成させる。object-terminal/deeper/imported/attributed/
argument-bearing/other asserted head、reachability/widening/`qua`、declaration/
theorem acceptance、truth/fact、closure/order、broader term/formula/child-graph
semantics、proof/Core/ControlFlow/VC、general chain semantics は open のままで
ある。Step 5 は active、Steps 6/7 は deferred のまま。checker source または
module-layout change は不要であった。

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
14 と 16 の exact checker shell handoff だけを credit する。task 117 はその
source 内の 2 つの exact source-derived `contradiction` constant だけを
`FormulaKind::Contradiction` payload に進め、同じ missing formula payload
diagnostic に留める。formula constant semantic truth value、child-formula graph
payload、quantifier binder/context payload、formula checking、recorded fact、
theorem acceptance、dedicated `formula_statement` runner、CoreIr、ControlFlowIr、
VC、proof payload coverage は主張しない。

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
| MC-G020 | `external_dependency_gap` / `deferred` | task 7-11 と後続 consumer の semantic pass fixture を妨げる source-to-checker extraction blocker として active。task 16 から task 81 までは reserve-only source-derived checker bridge を段階的に構築し、builtin reserve / type-expression payload は `TypedAst` と `ResolvedTypedAst` まで到達し、builtin `set` / `object` に終端する supported same-module bare local-mode expansion family は pass できる一方、local structure / attribute / argument / bracket / forward-reference / historical task-80 imported-attribute boundary は fabricated evidence なしの active diagnostic に留める。task 82 は task-79 imported mode source を一段進め、documented `parser.type_fixtures` 由来の imported `SymbolKind::Mode` を checker type-head payload として渡し、checker が `ImportedSource` provenance を検証してから missing imported `ModeExpansion` payload で fail closed する。task 83 は documented `parser.type_fixtures` imported structure `R` source を一段進め、checker が `ImportedSource` provenance を検証してから missing base-shape / constructor-witness evidence で fail closed する。task 97 は documented `TypeCaseStruct` source を同じ real checker type-head boundary と missing evidence query に進める。task 84 は documented `parser.type_fixtures` imported attribute `TypeCaseAttr` source を一段進め、checker が `ImportedSource` provenance を検証してから missing attributed-type existential/evidence payload で fail closed する。task 85 は既存 `non empty set` source を一段進め、builtin `set` 上の real imported negative `empty` checker `AttributeInput` payload として渡し、missing attributed-type existential/evidence payload で fail closed する。task 116 は既存 `empty set` source を一段進め、builtin `set` 上の real imported positive `empty` checker `AttributeInput` payload として渡し、同じ missing attributed-type existential/evidence payload で fail closed する一方、`non empty object` runner sidecar は extraction-gap boundary に残す。task 86 は parser / resolver 実行後の formula-only theorem source を active boundary として記録し、task 117 は task 115 を exact `FormulaPayloadBoundary: thesis` source について supersede し、source-derived `thesis` formula constant を real `FormulaKind::Thesis` checker payload として渡して missing formula payload で fail closed するが、formula constant checking、theorem acceptance、recorded fact、proof context、`formula_statement` runner は主張しない。task 106 は exact builtin equality theorem source について real source-derived checker `TermInput` と equality `FormulaInput` payload を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、equality semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 110 は task 98 の exact imported predicate/functor theorem source を supersede し、imported `divides`/`++` provenance を検証して real source-derived numeral、imported functor-application、predicate-application checker payload を `TermFormulaChecker` に渡し、missing numeric/signature payload と partial formula checking で fail closed するが、semantic predicate/functor signature、term inference、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 108 は exact builtin membership theorem source について real source-derived checker `TermInput` と membership `FormulaInput` payload を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、membership operand expected-type construction/checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 107 は exact builtin inequality theorem source について real source-derived checker `TermInput` と inequality `FormulaInput` payload を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、inequality desugaring または equality semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 109 は exact builtin type-assertion theorem source について real source-derived checker `TermInput`、type-assertion `FormulaInput`、asserted builtin `set` `TypeExpressionInput` を `TermFormulaChecker` に渡し、missing numeric type payload と partial formula checking で fail closed するが、broader asserted type payload、type-assertion semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 113 は exact imported attribute assertion theorem formula について task 103 を supersede し、imported `empty` provenance を検証して source-derived checker term/formula payload を渡し、missing numeric type payload、missing formula / attribute semantic payload、partial formula checking で fail closed するが、imported module AST extraction、theorem formula 向け checker `AttributeInput` payload extraction、attribute-chain semantic payload extraction、term inference、attribute admissibility/semantic checking、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 114 は exact attribute-level non-empty imported attribute assertion theorem formula について task 104 を supersede し、direct `non` surface と imported `empty` provenance を検証して source-derived checker term/formula payload を渡し、missing numeric type payload、missing formula / attribute semantic payload、partial formula checking で fail closed するが、imported module AST extraction、theorem formula 向け checker `AttributeInput` payload extraction、negated attribute-chain semantic payload、term inference、negated attribute admissibility/semantic checking、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 111 は task 105 の exact set-enumeration theorem formula source だけを supersede し、source-derived numeral item term、set-enumeration term、builtin equality formula checker payload を `TermFormulaChecker` に渡して missing numeric/result-type payload と partial formula checking で fail closed するが、broader set-enumeration result-type/sethood payload extraction、term inference、equality/formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 112 は task 99 の exact formula connective/quantifier theorem formula surface だけを supersede し、parser / resolver 実行後に real checker formula shell payload を渡して missing formula/quantifier payload で fail closed し、task 117 は同じ exact source の 2 つの `contradiction` constants を real `FormulaKind::Contradiction` payload に進めるが、formula constant semantic truth value、child-formula graph payload、quantifier binder/context payload、formula checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 180 は別に exact standalone `SourceDerivedContradictionConstantBoundary: contradiction` leaf を type/well-formedness だけの 1 `Checked` `FormulaKind::Contradiction` として check し、truth/fact publication または theorem/proof/downstream credit を主張しない。task 88 は parser / resolver 実行後の proof-block theorem source を同じ active extraction-gap boundary として記録し、proof skeleton payload、local proof context、formula payload、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 89 は parser / resolver 実行後の statement-level proof-justification theorem source を同じ active extraction-gap boundary として記録し、statement proof payload、nested proof skeleton payload、local proof context、formula payload、label-reference semantic checking、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 90 は parser / resolver 実行後の predicate/functor definition source を active extraction-gap boundary として記録し、definition declaration payload、definition-local context、definiens formula/term payload、overload payload、recorded fact、`formula_statement` runner は主張しない。task 91 は parser / resolver 実行後の attribute definition source を active extraction-gap boundary として記録し、attribute definition declaration payload、definition-local context、formula-definiens payload、attributed-type evidence、recorded fact、`formula_statement` runner は主張しない。task 92 は parser / resolver 実行後の mode/structure definition source を同じ active extraction-gap boundary として記録し、mode/structure definition declaration payload、mode expansion、structure base-shape / constructor / selector evidence、definition-local context、recorded fact、`formula_statement` runner は主張しない。task 93 は parser / resolver 実行後の proof-local declaration statement source を同じ active extraction-gap boundary として記録し、proof-local declaration payload、local proof context、formula / term payload、RHS term inference、reconsider coercion / obligation evidence、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 94 は parser / resolver 実行後の proof-local inline definition source を同じ active extraction-gap boundary として記録し、inline definition formal/body payload、local abbreviation expansion、term / formula body payload、guard evidence、recorded fact、theorem acceptance、`formula_statement` runner は主張しない。task 95 は parser / resolver 実行後の registration block source を同じ active extraction-gap boundary として記録し、registration-item payload、correctness-condition / proof-obligation payload、accepted activation / evidence status、cluster / reduction semantics、recorded fact、Chapter 17 semantic row、`formula_statement` / `advanced_semantics` runner は主張しない。task 96 は parser / resolver 実行後の redefinition/notation source を同じ active extraction-gap boundary として記録し、redefinition payload、notation alias relation payload、target inference、coherence proof-obligation payload、overload candidate payload、Chapter 11 alias semantic resolution、Chapter 19 overload/redefinition semantics、`formula_statement` / `advanced_semantics` runner は主張しない。より広い non-builtin declaration（task 96 の redefinition/notation extraction-gap boundary を超えるもの）、task 84 `TypeCaseAttr` provenance / `AttributeInput` bridge、task 85 negative `empty`/builtin-`set` bridge、task 116 positive `empty`/builtin-`set` bridge を超える imported attribute、task 83 `R` と task 97 `TypeCaseStruct` provenance/type-head bridge を超える imported structure、task 82 provenance/type-head bridge を超える imported mode expansion、attribute argument、qualified attribute qualifier / owner provenance、mode / structure argument、bracket `type_arg_list` と `qua`-argument provenance、term-argument provenance、structure base-shape / full attributed-type existential evidence、broader / attributed / argument-bearing / parameterized / contextual / ambiguous / cyclic mode expansion、task-106/task-107/task-108/task-109/task-110/task-111/task-112/task-113/task-114/task-117/task-180 exact leaf を超える numeric/signature/result-type payload と equality/inequality/membership/type-assertion/imported predicate-functor/set-enumeration semantic checking および task-112/task-117 を超える formula child/binder semantics、task-110/task-111/task-112/task-113/task-114/task-117 checker bridge、task-180 exact leaf、task-105/task-88/task-89 extraction-gap boundary を超える term / formula / proof skeleton、task-93 extraction-gap boundary を超える proof-local declaration payload、task-94 extraction-gap boundary を超える inline definition payload、task-95 extraction-gap boundary を超える registration payload / correctness-condition / activation payload、task-96 extraction-gap boundary を超える redefinition/notation payload、coercion、overload、recorded fact、CoreIr、ControlFlowIr、VC、proof payload extraction は未解決のまま。 |
| MC-G021 | `external_dependency_gap` / `deferred` | registration payload、accepted-status、source extraction blocker として active。registration code は explicit payload seam のみ消費する。 |
| MC-G023 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived cluster/reduction fixture、artifact/cache integration、source-derived normalization-result dependence、real trace extraction について active。task 46 は explicit-payload fatal contradiction と reduction trace-identity seam だけを cover する。 |
| MC-G025 | `external_dependency_gap` / `deferred` | accepted registration status の proof/artifact production または import について active。 |
| MC-G026 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived existential gate case、artifact reuse、accepted-status integration について active。 |
| MC-G027 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived overload payload、`coherence with` 省略 target diagnostic production、diagnostic code allocation、artifact emission/reuse、semantic fixture について active。task 45 は explicit-payload Rust regression だけを追加し、source-derived seed は inactive のままにする。 |
| MC-G030 | `test_gap` / `external_dependency_gap` / `deferred` | `formula_statement` と `advanced_semantics` runner/tag support、および source payload extraction について active。 |

Resolved setup-history row は closed のまま: MC-G001、MC-G010、MC-G012、
MC-G013、MC-G015、MC-G022、MC-G024、MC-G028、MC-G029 は task commit を持ち、
この audit で再オープンされた source/spec mismatch はない。

Task 171 current-state override: gap row の、`non empty object` sidecar が
extraction gap に残るという旧記述は supersede される。exact negative imported
`empty`/builtin-`object` source は checker evidence-query diagnostic に到達する。
positive `empty object`、symbol head 上の imported attribute、imported module AST
extraction、attribute evidence/acceptance、downstream payload は deferred のまま。

Task 181 current-state override: imported attributed-reserve routing は credit
済み task-84/85/116/171 source shape 5 件だけを許す exact guard を持つ。
corpus、expectation、traceability、runner count、semantic-coverage row は変更せず、
broader imported attributed-reserve shape は記録済み extraction gap のままである。

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
