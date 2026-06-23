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
  `CoercionRequestKind`, `CoercionEvidence`, `CoercionDeferredReason`,
  `InitialObligationInput`, `InitialRequirementKind`,
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
  `CheckedDeclaration`, `DeclarationStatus`, `TypeExpressionInput`,
  `TypeHeadInput`, `AttributeInput`, `AttributePolarity`, `ModeExpansion`,
  `NormalizedTypeTable`, `NormalizedType`, `TypeHeadRef`,
  `TypeHeadErrorKind`, `AttributeSet`, `AttributeInstance`, `TypeSource`,
  `NormalizedTypeStatus`, `SourceRangeKey`

対応:

| 仕様上の約束 | source 根拠 | test 根拠 | 状態 |
|---|---|---|---|
| Type-expression normalization は mode、attribute、arity、unsupported input の degraded state を canonicalize し、cluster repair を行わない。 | `TypeNormalizer`, `TypeNormalizationOutput`, `TypeExpressionInput`, `ModeExpansion`, normalized type tables. | attribute/order/builtin/mode-expansion/degraded-head tests. | explicit payload について実装済み。MC-G014 は残る。 |
| Declaration checking は explicit declaration と binding context を消費し、partial output を保持する。 | `DeclarationChecker`, `DeclarationCheckingOutput`, declaration input/status tables. | declaration deterministic/invalid/constrained/set/attributed/reconsider tests. | explicit payload について実装済み。MC-G016 は残る。 |
| Term/formula inference は checked table、expected constraint、open candidate、fact、recovery を記録する。 | `TermFormulaChecker`, term/formula input and checked tables. | term/formula/recovery tests. | explicit payload について実装済み。MC-G017/MC-G019 は残る。 |
| Coercion と initial obligation は `VcId` や fabricated evidence なしで記録される。 | `CoercionObligationChecker`, `CoercionInput`, `InitialObligationInput`, evidence/deferred enum. | coercion deterministic/missing evidence/alternate candidate tests. | explicit payload について実装済み。MC-G018 は残る。 |
| Fact query は deterministic、visibility-scoped、non-mutating。 | `TypeFactQueryEngine`, `TypeFactQueryOutput`, `TypeFactQueryStatus`. | deterministic/provenance/visibility/contradiction tests. | 実装済み。 |
| public enum は forward-compatible。 | public enum の `#[non_exhaustive]`。 | `checker_public_enums_are_forward_compatible_and_documented`. | task 31 で guard 済み。 |

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
  `ExistentialGateCandidate`, `ExistentialGateGuardEvidence`,
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
| Existential gate は accepted activation、visible guard、exact pattern、deterministic recovery を要求する。 | `ExistentialGateInput`, candidates, guard evidence, output/result/status types. | missing/inactive/pending/unaccepted/accepted/rejected/degraded existential tests. | explicit payload について実装済み。MC-G026 は残る。 |
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
| Saturation bound、loop、explicit contradiction は silent truncation でなく visible failure。 | `ClusterTraversalConfig`, `ClusterTraversalProfile`, `ClusterClosureStatus`, diagnostics. | loop/bound/zero-antecedent/contradiction tests. | 実装済み。 |
| Replay は active registration fingerprint を再検証する。 | `ResolutionTrace::replay`, `ClusterReplayReport`, `ClusterReplayStatus`. | `replay_revalidates_active_registration_fingerprint`, `active_pattern_fallback_must_match_rule_fingerprint`. | 実装済み。 |
| Reduction step は architecture 17 provenance、guard evidence、strategy audit key を保持する。 | `ReductionTraceBuilder`, `ReductionTraceOutput`, `ReductionStep`, reduction input/guard types. | reduction provenance/inactive/rejected/invalid/`such` guard tests. | explicit payload について実装済み。source-derived rewrite extraction は MC-G023。 |
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
| Specificity は explicit comparison から per-site graph を作り、ordinary root ordering に return type を使わない。 | `SpecificityGraphOutput::build`, graph/node/comparison/edge types. | graph/equivalence/return-type/empty/blocked/missing tests. | 実装済み。 |
| Selection は unique maximal ordinary root を選択し、accepted refinement を join し、widening/source-`qua` view を記録し、failure を保持する。 | `OverloadSelectionOutput::resolve`, result/view/refinement/exposed-result types. | selection/no-match/ambiguity/missing/redefinition/refinement/invalid/deterministic tests. | explicit payload について実装済み。 |
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
| `crates/mizar-checker/src/determinism_suite.rs` | type normalization、fact query、cluster closure、overload pipeline、final `ResolvedTypedAst` projection の cross-module deterministic rerun と equivalent-order permutation。 |
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
| MC-G020 | `external_dependency_gap` / `deferred` | task 7-11 と後続 consumer の semantic pass fixture を妨げる source-to-checker extraction blocker として active。 |
| MC-G021 | `external_dependency_gap` / `deferred` | registration payload、accepted-status、source extraction blocker として active。registration code は explicit payload seam のみ消費する。 |
| MC-G023 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived cluster/reduction fixture、artifact/cache integration、real trace extraction について active。 |
| MC-G025 | `external_dependency_gap` / `deferred` | accepted registration status の proof/artifact production または import について active。 |
| MC-G026 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived existential gate case、artifact reuse、accepted-status integration について active。 |
| MC-G027 | `test_gap` / `external_dependency_gap` / `deferred` | source-derived overload payload、diagnostic code allocation、artifact emission/reuse、semantic fixture について active。 |
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
