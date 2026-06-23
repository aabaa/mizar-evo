# Source/Spec Correspondence Audit: mizar-core

> 正本は英語です。英語版:
> [../en/source_spec_audit.md](../en/source_spec_audit.md)。

Task 22 は task 21 後の public `mizar-core` surface を監査する。この task は
source behavior、public API、`.miz` fixture、expectation、runtime semantics を
変更しない。実装済み public surface がどの仕様、test、guard、明示 deferred row
に対応しているかを記録する。

## Scope And Method

inventory は `crates/mizar-core/src/lib.rs` の現在の `pub mod` export、
`crates/mizar-core/src/*.rs` の top-level public item、crate-local
`dense_id!` / `string_key!` / `table!` macro が生成する public newtype/table
を対象にする。public method は owning public type の下にまとめる。module spec
が builder、table、accessor、lowering、rendering API をその単位で記述している
ためである。

監査対象の module specification:

- [core_ir.md](./core_ir.md)
- [binder_normalization.md](./binder_normalization.md)
- [elaborator.md](./elaborator.md)
- [control_flow.md](./control_flow.md)

Result: 実装済み explicit-payload API について No source/spec drift。public
surface に blocking な `source_undocumented_behavior`、`design_drift`、
`boundary_violation` は残っていない。利用不能な coverage は実装済み挙動として
扱わず、下記の `external_dependency_gap` または `deferred` として分類する。

## Crate Module Exports

`src/lib.rs` は次の public module だけを export する:

- `binder_normalization`
- `control_flow`
- `core_ir`
- `elaborator`

Evidence: `crates/mizar-core/tests/lint_policy.rs` の
`public_semantic_modules_have_owning_specs` がこの list を検査し、crate が frontend
や downstream VC/proof/kernel boundary を越えないことを保つ。さらに本 audit が
各 public module/source/spec path を記録していることも検査する。

## Public Surface Inventory

### `core_ir`

Source: `crates/mizar-core/src/core_ir.rs`。owning spec: `core_ir.md`。

生成される public dense id:

- `CoreItemId`, `CoreTermId`, `CoreFormulaId`, `CoreDefinitionId`,
  `CoreProofId`, `CoreProofNodeId`, `CoreAlgorithmId`,
  `CoreAlgorithmStmtId`, `GeneratedOriginId`, `ObligationSeedId`,
  `CoreDiagnosticId`, `CoreVarId`

生成される public string key:

- `CoreTypePredicate`, `CoreVisibility`, `CoreVarRole`,
  `GeneratedOriginKey`, `CoreProvenanceKey`, `CoreDiagnosticMessageKey`,
  `LocalProofOrProgramPath`, `NormalizedSemanticOrigin`, `CoreLabelRef`,
  `CorePlace`, `GhostEffectKey`

生成される public table:

- `CoreItemTable`, `CoreTermTable`, `CoreFormulaTable`,
  `CoreDefinitionTable`, `CoreProofTable`, `CoreProofNodeTable`,
  `CoreAlgorithmTable`, `CoreAlgorithmStmtTable`, `GeneratedOriginTable`,
  `CoreDiagnosticTable`

literal top-level public item:

- `CoreIr`, `CoreIrParts`, `CoreItem`, `CoreItemKind`, `CoreItemStatus`,
  `CoreTerm`, `CoreTermKind`, `CoreFormula`, `CoreFormulaKind`,
  `CoreBinder`, `CoreDefinition`, `DefinitionBody`,
  `GuardedDefinitionBranch`, `DefinitionBranchBody`, `ExpansionPolicy`,
  `CoreProof`, `CoreProofStatus`, `CoreProofNode`, `CoreProofNodeKind`,
  `CoreJustification`, `CoreCitation`, `ProofBranchKind`, `CoreAlgorithm`,
  `CoreContractSet`, `CoreAlgorithmStmt`, `CoreAlgorithmStmtKind`,
  `CoreAlgorithmMatchArm`, `GeneratedOrigin`, `GeneratedOriginKind`,
  `ObligationSeed`, `ObligationSeedCanonicalKey`, `ObligationSeedKind`,
  `ObligationSeedStatus`, `ObligationSeedTable`, `CoreSourceMap`,
  `CoreSourceRef`, `CoreSourceAnchor`, `GeneratedFrom`, `CoreProvenance`,
  `CoreProvenancePhase`, `CoreDiagnostic`, `CoreDiagnosticClass`,
  `CoreDiagnosticSeverity`, `CoreDiagnosticRecovery`, `CoreNodeRef`,
  `CoreIrError`

対応:

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| item、term、formula、definition、proof skeleton、algorithm shell、generated origin、obligation seed、source map、diagnostic、validation、deterministic debug text の dense table を持つ backend-neutral `CoreIr` root。 | `src/core_ir.rs` の `CoreIr`, `CoreIrParts`, generated id/table, `ObligationSeedTable`, `CoreIr::try_new`, `CoreIr::debug_text`, validation error。 | `src/core_ir.rs` の module-local unit test、`crates/mizar-core/tests/determinism_suite.rs`。 | explicit public-API fixture について実装済み。 |
| Core term/formula/proof/algorithm shape は failed semantic site を explicit に保持し、proof search、VC generation、kernel checking を行わない。 | `CoreTermKind::Error`, `CoreFormulaKind::Error`, proof status / terminal seed data, algorithm statement shell, `CoreDiagnostic` class/recovery。 | invalid reference、error node、source map、proof/algorithm shell invariant、deterministic rendering の unit test。 | core data layer 内で実装済み。 |
| Definition boundary、generated origin、source/core provenance、obligation seed は explicit かつ deterministic。 | `CoreDefinition`, `GeneratedOrigin`, `ObligationSeed`, `ObligationSeed::canonical_key`, `CoreSourceMap`, `CoreSourceRef`, `CoreProvenance`。 | `src/core_ir.rs` の unit test、`src/control_flow.rs` の obligation-handoff test、determinism integration suite。 | 実装済み。concrete `VcId` と `ObligationAnchor` は downstream。 |
| Public enum は downstream forward-compatible。 | `src/core_ir.rs` の public enum 上の `#[non_exhaustive]`。 | `tests/lint_policy.rs` の `public_core_enums_are_forward_compatible_and_documented`。 | task 21 で guard 済み。 |

### `binder_normalization`

Source: `crates/mizar-core/src/binder_normalization.rs`。owning spec:
`binder_normalization.md`。

top-level public item:

- `BinderResult`, `BoundVar`, `NormalizedVar`, `NormalizedVarClass`,
  `NormalizedVarSort`, `BinderFrame`, `NormalizedBinderEntry`,
  `BinderContext`, `NormalizedTerm`, `NormalizedTermKind`,
  `NormalizedFormula`, `NormalizedFormulaKind`, `GeneratedOriginRecord`,
  `CanonicalTerm`, `CanonicalTermKind`, `CanonicalFormula`,
  `CanonicalFormulaKind`, `CanonicalVar`, `CanonicalBinderEntry`,
  `CanonicalGeneratedOrigin`, `NormalizedTermOrFormula`,
  `SubstitutionTarget`, `SubstitutionReplacement`, `CapturePolicy`,
  `SubstitutionSideConditions`, `FreshnessConfig`,
  `NormalizedTermOrFormulaPath`, `Substitution`, `SubstitutionResult`,
  `SubstitutionOutput`, `FreshnessWitness`, `BinderDiagnostic`,
  `BinderDiagnosticClass`, `DefinitionClosure`, `DefinitionExpansion`

public function:

- `recompute_fresh_id`, `shift_term`, `shift_formula`, `open_rec_term`,
  `open_rec_formula_with_term`, `open_rec_formula_with_formula`,
  `close_rec_term`, `close_rec_formula`, `subst_bound_term`,
  `subst_bound_formula`, `apply_substitution_to_term`,
  `apply_substitution_to_formula`, `expand_definition_closure`,
  `normalize_core_term`, `normalize_core_formula`,
  `validate_normalized_term`, `validate_normalized_formula`,
  `canonical_term`, `canonical_formula`, `alpha_equivalent_terms`,
  `alpha_equivalent_formulas`

対応:

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| de Bruijn bound index、stable free/schematic/generated id、metadata validation を持つ locally nameless representation。display name は semantics から除外される。 | `NormalizedVar`, `BinderContext`, `BinderFrame`, `NormalizedTerm`, `NormalizedFormula`, validation function。 | task 5-6 の module-local test、determinism suite の guarded-quantifier case。 | 実装済み。 |
| capture-avoiding substitution、shift/open/close、replayable freshness witness、malformed evidence rejection。 | shift/open/close/substitution function、`Substitution`, `FreshnessWitness`, `BinderDiagnostic`。 | `src/binder_normalization.rs` の shadowing、capture、malformed evidence、composition、closure test。 | core public fixture について実装済み。 |
| alpha-equivalence と canonical form は deterministic で source-name independent。 | `CanonicalTerm`, `CanonicalFormula`, canonical / alpha-equivalence function。 | reflexive/symmetric/transitive、idempotence、canonical iff alpha-equivalent test、determinism suite。 | 実装済み。 |
| Kernel replay shape を保つが kernel API は仮造しない。 | freshness witness、definition closure、side-condition data。 | Rust unit test が replayable metadata を検査する。kernel crate 依存はない。 | core-owned shape として実装済み。kernel checking は external。 |
| Public enum は downstream forward-compatible。 | `src/binder_normalization.rs` の public enum 上の `#[non_exhaustive]`。 | `public_core_enums_are_forward_compatible_and_documented`。 | task 21 で guard 済み。 |

### `elaborator`

Source: `crates/mizar-core/src/elaborator.rs`。owning spec: `elaborator.md`。

top-level public API group:

- Context preparation: `CoreContextResult`, `CoreContextError`,
  `CheckerOwnedProvenance`, `CoreItemSeed`, `CoreDependencySummary`,
  `CoreVariableSeed`, `CoreBinderSeed`, `GeneratedOriginSeed`,
  `DefinitionBoundaryKind`, `DefinitionBoundaryStatus`,
  `DefinitionBoundary`, `DefinitionBoundaryRegistry`,
  `GeneratedOriginRegistry`, `CoreItemRegistry`,
  `CoreDependencyResolution`, `BinderSourceRecord`,
  `BinderSourceRegistry`, `ResolvedTypedAstSummary`, `CheckerSiteSummary`,
  `CheckerSiteKind`, `CheckerSiteSeverity`, `CoreContextInput`,
  `CoreContext`, `ElaborationWorklist`, `ElaborationWorkItem`,
  `ElaborationWorkItemKind`, `ElaborationWorkStatus`,
  `prepare_core_context`
- Type and fact lowering: `TypeAndFactResult`, `TypeAndFactLoweringError`,
  `TypeAndFactLoweringInput`, `TypePredicateSeed`,
  `DeclaredBinderTypeSeed`, `AttributeChainSeed`, `ModeExpansionSeed`,
  `ClusterFactSeed`, `ViewExplanationKind`, `ViewExplanationSeed`,
  `ReconsideringSeed`, `ObligationFormulaSeed`,
  `CarriedInitialObligationSeed`, `MissingEvidenceKind`,
  `MissingEvidenceSeed`, `TypeAndFactLoweringOutput`,
  `LoweredBinderGuard`, `LoweredModeExpansion`, `LoweredClusterFact`,
  `ViewExplanation`, `ReconsideredBinding`, `MissingEvidenceRecord`,
  `lower_type_and_fact_inputs`
- Term/formula lowering: `TermAndFormulaResult`,
  `TermAndFormulaLoweringError`, `CoreTermSeedId`, `CoreFormulaSeedId`,
  `TermAndFormulaLoweringInput`, `CoreTermSeed`, `CoreTermSeedKind`,
  `CoreFormulaSeed`, `CoreFormulaSeedKind`, `QuantifierBinderSeed`,
  `FailedSemanticSiteSeed`, `CoreObligationSeed`,
  `FraenkelMembershipObligationSeed`,
  `AlreadyCarriedFraenkelMembershipSeed`,
  `TermAndFormulaLoweringOutput`, `GeneratedOriginUse`,
  `GeneratedOriginReuseSource`, `LoweredGeneratedObligation`,
  `AlreadyCarriedGeneratedObligation`, `lower_term_and_formula_inputs`
- Definition/proof/algorithm lowering: `DefinitionLoweringResult`,
  `DefinitionLoweringError`, `DefinitionLoweringInput`, `DefinitionSeed`,
  `DefinitionBodySeed`, `GuardedDefinitionBranchSeed`,
  `DefinitionGuardSeed`, `DefinitionCorrectnessSeed`,
  `DefinitionObligationSeed`, `DefinitionLoweringOutput`,
  `DefinitionItemStatusUpdate`, `DefinitionCorrectnessRecord`,
  `DefinitionGeneratedDependencyRecord`, `OtherwiseGuardRecord`,
  `lower_definition_inputs`, `ProofLoweringResult`, `ProofLoweringError`,
  `ProofLoweringInput`, `ProofSeed`, `ProofSkeletonSeed`, `ProofNodeSeed`,
  `ProofFormulaRef`, `ProofJustificationSeed`, `ProofTerminalGoalSeed`,
  `MalformedProofSkeletonSeed`, `ProofLoweringOutput`,
  `ProofStatusRecord`, `ProofTerminalObligationRecord`,
  `ProofTerminalCitationRecord`, `lower_proof_inputs`,
  `AlgorithmLoweringResult`, `AlgorithmLoweringError`,
  `AlgorithmLoweringInput`, `AlgorithmSeed`, `AlgorithmPayloadSeed`,
  `AlgorithmStmtSeed`, `AlgorithmMatchArmSeed`,
  `AlgorithmLoweringOutput`, `lower_algorithm_inputs`

対応:

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Step 1 は explicit checker/resolver/session payload 上で deterministic core context を準備し、raw syntax を scan しない。 | context seed/input/output type、registry、summary、worklist、`prepare_core_context`、lint boundary guard。 | task 8 の `src/elaborator.rs` test、`core_source_stays_off_frontend_and_downstream_boundaries`。 | explicit checker-owned payload summary について実装済み。 |
| Step 2 は soft type/fact data を explicit predicate、assumption、view provenance、carried obligation、diagnostic、deferred seed へ lower し、registration activation はしない。 | type/fact seed/output type と `lower_type_and_fact_inputs`。 | declared binder、attribute、mode、cluster fact、`qua`、reconsidering、carried/missing evidence の task 9 test。 | 実装済み。 |
| Step 3 は term/formula、inserted/source `qua`、stable choice、Fraenkel comprehension、failed site、generated origin、generated obligation を lower する。 | term/formula seed/output type と `lower_term_and_formula_inputs`。 | surface form、generated origin reuse/delta、sethood evidence、failed error node、quantifier guard の task 10 test。 | explicit seed について実装済み。 |
| Step 4 は definition expansion boundary を explicit に保ち、correctness obligation と generated dependency を記録する。 | definition seed/output type と `lower_definition_inputs`。 | boundary、correctness seed、generated dependency、skipped/error status の task 11 test。 | 実装済み。 |
| Step 5 は proof skeleton、thesis tracking、label、citation、malformed root、terminal obligation seed を lower し、proof acceptance はしない。 | proof seed/output type と `lower_proof_inputs`。 | proof form、citation、terminal goal、label、malformed/error case、durable terminal citation の task 12 test。 | 実装済み。 |
| Step 6 は contract、ghost/runtime metadata、local `Pick` binder、source/provenance preservation、diagnostic を持つ algorithm shell を lower し、CFG construction はしない。 | algorithm seed/output type と `lower_algorithm_inputs`。 | shell form、malformed statement、source/provenance、status、diagnostic aggregation の task 13 test。 | 実装済み。 |
| Public enum は downstream forward-compatible。 | `src/elaborator.rs` の public enum 上の `#[non_exhaustive]`。 | `public_core_enums_are_forward_compatible_and_documented`。 | task 21 で guard 済み。 |

### `control_flow`

Source: `crates/mizar-core/src/control_flow.rs`。owning spec:
`control_flow.md`。

生成される public dense id:

- `ControlFlowId`, `BasicBlockId`, `LocalId`, `LoopId`,
  `ControlFlowExitId`, `ProgramContextId`, `ContextFactId`,
  `AssignmentEffectId`, `CallSiteId`, `ControlFlowDiagnosticId`,
  `ObligationHandoffId`

生成される public table:

- `ControlFlowTable`, `ControlFlowBlockTable`, `ControlFlowLocalTable`,
  `ProgramContextTable`, `ContextFactTable`, `AssignmentEffectTable`,
  `CallSiteTable`, `ControlFlowLoopTable`, `ControlFlowExitTable`,
  `ControlFlowDiagnosticTable`, `ObligationHandoffTable`

literal top-level public item:

- `ControlFlowOutput`, `ObligationSeedHandoff`, `ObligationHandoffEntry`,
  `ObligationHandoffOrigin`, `ControlFlowObligationSite`,
  `ControlFlowObligationSiteKind`, `ControlFlowIr`, `ControlFlowBlock`,
  `ControlFlowTerminator`, `ControlFlowSwitchArm`, `Reachability`,
  `ControlFlowLocal`, `LocalKind`, `LocalDeclaration`, `LocalMutability`,
  `ProgramContext`, `ContextFact`, `ContextFactKind`, `AssignmentEffect`,
  `AssignmentEffectTarget`, `CallSite`, `ControlFlowContractSet`,
  `ContractSite`, `ContractSiteKind`, `ContractSitePlacement`,
  `AssertionSite`, `AssertionPlacement`, `LoopInvariantSite`,
  `LoopInvariantPlacement`, `TerminationMeasureSite`,
  `TerminationMeasurePlacement`, `ControlFlowLoop`, `ControlFlowExit`,
  `ControlFlowExitKind`, `ControlFlowGhostTable`, `GhostVisibility`,
  `ControlFlowTerminationPlan`, `TerminationSite`, `TerminationSiteKind`,
  `ControlFlowSourceMap`, `ControlFlowStatementPlacement`,
  `ControlFlowDiagnostic`, `ControlFlowDiagnosticKind`,
  `build_control_flow_ir`, `build_obligation_seed_handoff`

対応:

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Phase-10 CFG construction は core algorithm shell から deterministic block、edge、local、context、statement placement、source map、structural diagnostic、debug rendering を作る。 | generated id/table、`ControlFlowOutput`, `ControlFlowIr`, block/local/context/source-map/diagnostic shape、`build_control_flow_ir`, `debug_text`。 | task 15 の `src/control_flow.rs` test、determinism integration suite。 | explicit core fixture について実装済み。 |
| contract、assertion、invariant、ghost-effect、termination、context-fact attachment は source-mapped で、VC は生成しない。 | `ControlFlowContractSet`, `ContractSite`, `AssertionSite`, `LoopInvariantSite`, `TerminationMeasureSite`, `ControlFlowGhostTable`, `ControlFlowTerminationPlan`, context fact/effect。 | placement、ghost isolation、context fact、decreasing site の task 16 test。 | 実装済み。 |
| Flow diagnostic は use-before-assignment と unreachable statement を stable ordering と local structured class のみで cover する。 | `ControlFlowDiagnostic`, `ControlFlowDiagnosticKind`, diagnostic source-map row、builder code の stable sorting/remapping。 | task 17 diagnostic test。 | 実装済み。public diagnostic code-space は external。 |
| Obligation handoff は existing core seed を clone し、flow-derived deferred seed を emit するが、`VcId` 割り当てや `ObligationAnchor` 構築はしない。 | `ObligationSeedHandoff`, `ObligationHandoffEntry`, `ObligationHandoffOrigin`, `ControlFlowObligationSite`, `build_obligation_seed_handoff`。 | exact source/provenance/site/order と deferred flow seed の task 18 test、determinism suite。 | seed のみとして実装済み。VC/proof/kernel schema は external。 |
| Public enum は downstream forward-compatible。 | `src/control_flow.rs` の public enum 上の `#[non_exhaustive]`。 | `public_core_enums_are_forward_compatible_and_documented`。 | task 21 で guard 済み。 |

## Cross-Cutting Test And Guard Evidence

| Evidence | Coverage | Status |
|---|---|---|
| `src/core_ir.rs`, `src/binder_normalization.rs`, `src/elaborator.rs`, `src/control_flow.rs` の module-local unit test。 | Data validation、substitution/canonicalization、explicit-payload lowering、CFG construction、diagnostic、handoff、deterministic rendering。 | Active Rust coverage。 |
| `crates/mizar-core/tests/determinism_suite.rs`。 | fresh public-API fixture rebuild、structural equality、byte-stable rendering、binder canonicalization、CFG と handoff ordering。 | task 20 の active Rust coverage。 |
| `crates/mizar-core/tests/lint_policy.rs`。 | workspace/crate boundary、public module/spec pairing、frontend/downstream boundary、public enum policy、source/spec audit coverage。 | Active guard coverage。 |
| `tests/coverage/spec_trace.toml`。 | source-derived `type_elaboration` と `proof_verification` snapshot seam の deferred 記録。 | task 19 の metadata-only deferred row。 |

## Source-Undocumented Behavior Pass

この audit は、source behavior を module spec、active Rust test、lint guard、
traceability row、または明示 gap classification に対応づけるまで inventory として
扱う。現在の top-level public item はすべて上の module inventory に分類され、
correspondence row で promised behavior と spec/test evidence に結びつけられている。

現在の public surface に `source_undocumented_behavior` は残っていない。active な
source-derived `.miz` / snapshot test でまだ cover できない item は undocumented
source behavior ではなく、下の follow-up register で分類した unavailable seam である。

## Remaining Gaps

| ID | Class | Evidence | Owner | Unblock condition | Target follow-up / downstream phase |
|---|---|---|---|---|---|
| CORE-AUDIT-G001 | `external_dependency_gap` | source-to-checker extraction が full source-derived `ResolvedTypedAst` payload と production source-to-core fixture をまだ block している。 | Checker extraction / mizar-test integration follow-up。 | `mizar-core` が raw syntax を再 scan せずに checker-ready AST-wide payload extraction を利用できる。 | active source-derived core lowering fixture と snapshot を追加する。 |
| CORE-AUDIT-G002 | `external_dependency_gap` | `mizar-test` は `CoreIr` / `ControlFlowIr` 向けの active source-derived `type_elaboration` / `proof_verification` snapshot runner をまだ提供しない。 | `mizar-test` staged runner follow-up。 | Stage runner が real checker payload 由来の `CoreIr` / `ControlFlowIr` baseline を比較できる。 | task-19 の deferred traceability row を active corpus snapshot に置き換える。 |
| CORE-AUDIT-G003 | `external_dependency_gap` | artifact schema emission、proof acceptance、VC generation、kernel checking は downstream または cross-crate work。 | `mizar-artifact`, `mizar-proof`, `mizar-vc`, `mizar-kernel` phase。 | downstream crate が core/control-flow handoff の accepted schema と consumer を定義する。 | `mizar-core` を proof acceptance / kernel checking に変えず consumer を接続する。 |
| CORE-AUDIT-G004 | `external_dependency_gap` | concrete `VcId`、`ObligationAnchor`、VC fingerprint、proof/cache reuse anchor、downstream artifact identity は `mizar-core` の責務外。 | `mizar-vc` incremental verification / artifact phase。 | downstream identity と anchor contract が存在する。 | current obligation seed と local path を downstream anchor に map する。 |
| CORE-AUDIT-G005 | `external_dependency_gap` | source-derived call/result substitution、pattern、snapshot、claim、より豊かな algorithm payload seam は checker-owned explicit payload を必要とする。 | Checker payload extraction と phase-10/phase-11 integration。 | それらの source form 向け explicit checker payload が存在する。 | 新 payload の lowering / CFG / VC fixture coverage を active に追加する。 |
| CORE-AUDIT-G006 | `deferred` | Public diagnostic code-space はこの crate が割り当てない。 | Diagnostics registry owner。 | shared public diagnostic registry と allocation policy が存在する。 | current local structured class を保ったまま public code を割り当てる。 |

この audit で新しい `mizar-core` implementation task は開かない。残りの item は
upstream extraction、mizar-test、diagnostics registry、downstream VC/proof/kernel/
artifact phase が所有する follow-up record である。Task 23 は bilingual
documentation synchronization に進む。
