# Source/Spec Correspondence Audit: mizar-vc

> 正本言語: 英語。英語正本:
> [../en/source_spec_audit.md](../en/source_spec_audit.md)。

Task 18 は public-enum policy gate 後の現在の `mizar-vc` public surface と
仕様が約束する挙動を監査する。この task は source behavior、`.miz` fixture、
expectation、public API、language specification を変更しない。まだ利用できない挙動は、
現在の実装都合を normative とせず、明示的な `external_dependency_gap`、`test_gap`、
または `deferred` work として記録する。

## Scope And Method

この inventory は `crates/mizar-vc/src/lib.rs` が export する public module、各 module
の top-level public item、public data type の public method を対象にする。crate-local
`dense_id!` / `string_key!` macro が生成する public newtype は、その constructor と accessor
を所有する `vc_ir` に含める。

監査した module specification:

- [vc_ir.md](./vc_ir.md)
- [generator.md](./generator.md)
- [discharge.md](./discharge.md)
- [dependency_slice.md](./dependency_slice.md)
- [kernel_evidence_handoff.md](./kernel_evidence_handoff.md)
- [todo.md](./todo.md)

結果: 現在の explicit-payload implementation について、未分類の `source_drift`、
`design_drift`、`source_undocumented_behavior`、`test_expectation_drift`、
`boundary_violation`、`repo_metadata_conflict` は観測されない。残る gap は下に列挙する
既知の deferred / external seam である。

## Crate Module Exports

`src/lib.rs` は次の mizar-vc-owned module だけを export する:

- `dependency_slice`
- `discharge`
- `generator`
- `kernel_evidence_handoff`
- `vc_ir`

対応する source path:

- `crates/mizar-vc/src/dependency_slice.rs`
- `crates/mizar-vc/src/discharge.rs`
- `crates/mizar-vc/src/generator.rs`
- `crates/mizar-vc/src/kernel_evidence_handoff.rs`
- `crates/mizar-vc/src/vc_ir.rs`

Evidence: `crates/mizar-vc/tests/lint_policy.rs` の
`vc_lib_exposes_only_current_spec_backed_modules` がこの list と対応する EN/JA module spec を
確認し、crate root に re-export や boundary-crossing import を置かないことを guard する。

## Public Surface Inventory

### `vc_ir`

生成 public newtype:

- `VcId`, `VcGeneratedFormulaId`, `ContextEntryId`
- `VcSchemaVersion`, `GenerationSchemaVersion`, `ExpansionSchemaVersion`,
  `VcModuleRef`, `CanonicalSortKey`, `PolicyKey`, `PolicyValue`,
  `ProofHintKey`, `AnchorUnavailableReason`, `VcText`

literal top-level public item:

- `VcSet`, `VcSetParts`, `VcStatusPlan`, `VcStatusOverride`,
  `VcStatusAction`, `CanonicalVcFingerprint`, `LocalContextFingerprint`,
  `VcIr`, `VcSourceRef`, `SeedVcRef`,
  `VcGeneratedFormula`, `VcGeneratedFormulaKind`, `VcGeneratedFormulaShape`,
  `QuantifierKind`, `VcFormulaRef`, `VcKind`,
  `RegistrationCorrectnessKind`, `LoopInvariantPhase`, `RangeLoopObligation`,
  `CollectionLoopObligation`, `LocalContext`, `ContextEntry`,
  `ContextEntryKind`, `VerifierPolicyInput`, `PremiseRef`, `ProofHint`,
  `DefinitionUnfoldRequest`, `DefinitionOpacityOverride`,
  `PremiseRestriction`, `ComputationHint`, `VcStatus`,
  `DischargeEvidenceRef`, `SeedAccounting`, `SeedIntakeTable`,
  `SeedIntakeRow`, `SeedIntakeMapping`, `SeedOriginRef`, `SeedVcMapping`,
  `SeedNoVcReason`, `ExpandedVcRef`, `ObligationAnchor`, `AnchorOwner`,
  `AnchorLabel`, `AnchorLabelRole`, `AnchorCompleteness`,
  `AnchorIngredient`, `VcProvenance`, `VcProvenancePhase`, `HashMarker`,
  `VcIrError`

Correspondence:

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| `VcSet` が schema、snapshot/source/module identity、generated formula、VC、seed accounting、validation、deterministic debug rendering、lookup helper、immutable status projection を所有する。 | `crates/mizar-vc/src/vc_ir.rs` の `VcSet`、`VcSetParts`、`VcStatusPlan`、`VcStatusAction`、`VcStatusOverride`、validation helper。 | `constructs_minimal_vc_set_with_symbolic_refs`、`rendering_is_byte_identical_and_marks_incomplete_anchor`、status-plan tests、generated formula / validation rejection tests。 | explicit payload 向けに実装済み。 |
| Seed intake は各 handoff entry を正確に 1 回 tracking し、skipped/deferred/error/missing-goal row の no-VC reason を記録する。 | `SeedIntakeTable`、`SeedIntakeRow`、`SeedIntakeMapping`、seed-origin/mapping helpers。 | `seed_intake_preserves_handoff_order_and_debug_rendering`、`seed_intake_records_visible_no_vc_reasons`、duplicate/missing-source and distinct-origin tests。 | 実装済み。 |
| `VcIr` は obligation を証明・消去せず、source ref、seed ref、anchor、local context、premise、goal ref、proof hint、status、provenance を保持する。 | `VcIr`、`VcSourceRef`、`LocalContext`、`ProofHint`、`VcStatus`、`ObligationAnchor`、validation helpers。 | context/status/anchor/generated-goal validation tests と downstream generator/discharge/dependency tests。 | 実装済み。source-derived extraction は external のまま。 |
| Canonical VC / local-context fingerprint は stable generated formula / context payload を解決し、raw upstream row id、quantified binder gap、unresolved payload では fail closed する。 | `CanonicalVcFingerprint`、`LocalContextFingerprint`、`VcSet::canonical_vc_fingerprint`、`VcSet::local_context_fingerprint`、fingerprint payload helpers。 | Task 20 determinism と unresolved-payload dependency tests が public reuse path と fail-closed boundary を exercise する。 | stable generated payload 向けに実装済み。upstream core/definition/binder payload は external のまま。 |
| Public enum は forward-compatible である。 | すべての public enum の `#[non_exhaustive]`。 | `vc_public_enums_are_forward_compatible_and_documented`。 | task 17 で guard 済み。 |

### `generator`

Source path: `crates/mizar-vc/src/generator.rs`.

literal top-level public item:

- `CoreGenerationInput`, `VcNormalizationInput`,
  `CoreGenerationCandidateSet`, `CoreGenerationCandidate`,
  `CoreGenerationNoCandidate`, `GeneratorError`

Correspondence:

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Candidate generation は fresh な `SeedIntakeTable` と matching `ObligationSeedHandoff` を消費し、stale/partial intake を拒否する。 | `CoreGenerationCandidateSet::try_from_seed_intake`。 | `rejects_stale_intake_handoff_mismatch`、`rejects_partial_intake_when_handoff_adds_obligations`。 | 実装済み。 |
| Task-6 theorem、definition、generated-core、explicit registration-style candidate は source、context、proof hint、status を保持し、unavailable payload は visible no-candidate record にする。 | `build_candidate`、task-six generation helpers、no-candidate helpers。 | task-six candidate tests、theorem-status、registration-style、local-context、deferred-row tests。 | explicit upstream payload 向けに実装済み。dedicated missing payload は external のまま。 |
| Task-7 algorithm candidate は explicit flow-site metadata と goal formula からだけ生成し、unsupported family は visible no-candidate/deferred row に残す。 | flow-derived generation and no-candidate helpers。 | flow-site generation tests、flow mismatch tests、unavailable algorithm family tests。 | explicit flow payload 向けに実装済み。call-precondition、branch、match、range-loop、collection-loop、term-only termination、partial termination、Pick、ghost-erasure payload family は external/deferred のまま。 |
| Normalization は documented kind rank、candidate sort key、handoff id で dense `VcId` を割り当て、seed accounting と status を保持する。 | `CoreGenerationCandidateSet::try_normalize`。 | normalization id/order、duplicate、deferred status、expanded mapping、debug rendering tests。 | 実装済み。 |
| Generated anchor は stable source-shaped provenance が存在するとき source-shape hash を持つが、stable formula/context payload がない場合 canonical goal/context hash marker は fail closed する。 | `anchor_for_seed`、source-shape / canonical marker helpers。 | task-six / algorithm candidate tests が source-shape availability と raw-core canonical-goal incompleteness を assert する。 | 現在の candidate 向けに実装済み。 |
| Public enum は forward-compatible である。 | `GeneratorError` は `#[non_exhaustive]`。 | `vc_public_enums_are_forward_compatible_and_documented`。 | task 17 で guard 済み。 |

### `discharge`

Source path: `crates/mizar-vc/src/discharge.rs`.

literal top-level public item:

- `DEFAULT_COMPUTATION_STEP_LIMIT`, `DEFAULT_COMPUTATION_LIMIT_POLICY`,
  `DEFINITIONAL_REDUCTION_POLICY`, `DEFINITIONAL_REDUCTION_ALLOW`
- `DischargeInput`, `DischargePolicy`, `ComputationLimit`,
  `DischargeOutput`, `DischargeEvidenceRecord`, `DischargeEvidenceSource`,
  `DischargeEvidenceInputs`, `DischargePolicyEvidence`,
  `DischargeComputationEvidence`, `DischargeEvidenceReplay`,
  `DischargeExplanation`, `DischargeExplanationCategory`, `DischargeRule`,
  `try_discharge`

Correspondence:

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Deterministic pre-ATP discharge は supported explicit rule だけを適用し、unsupported VC を ATP 用に保持する。 | `try_discharge`、rule selection / decision application helpers。 | generated tautology、reflexivity、contradiction/direct fact、trace/definition/computation、unsupported and limit-exceeded tests。 | explicit `VcIr` data 向けに実装済み。 |
| Evidence と explanation は deterministic で in-memory replayable だが、trusted kernel proof ではない。 | `DischargeOutput`、`DischargeEvidenceRecord`、`DischargeExplanation`、`debug_text`。 | evidence-record、preserved-status、policy/deferred、multi-output order、deterministic repeat tests。 | in-memory evidence として実装済み。artifact serialization と kernel/proof/cache validation は external/deferred。 |
| Newly produced deterministic discharge evidence hash は canonical VC/context fingerprint が利用可能な場合だけ cross-edit stable である。 | `evidence_hash`、canonical fingerprint lookup、conservative-unknown marker。 | Task 20 reuse tests が stable deterministic discharge evidence と fail-closed evidence boundary を cover する。 | deterministic-discharge branch 向けに実装済み。 |
| Policy status は明示的に保持され、discharge evidence にはならない。 | `try_discharge` の status-preserving path。 | `policy_and_deferred_statuses_are_preserved_without_discharge_evidence`、determinism suite。 | 実装済み。 |
| Public enum は forward-compatible である。 | discharge public enum は `#[non_exhaustive]`。 | `vc_public_enums_are_forward_compatible_and_documented`。 | task 17 で guard 済み。 |

### `dependency_slice`

Source path: `crates/mizar-vc/src/dependency_slice.rs`.

literal top-level public item:

- `DEPENDENCY_SLICE_SCHEMA_VERSION`
- `DependencySliceInput`, `DependencySliceSet`, `DependencySlice`,
  `DependencySliceFingerprint`, `DependencySliceCompleteness`,
  `DependencyEntry`, `DependencyEntryClass`, `DependencyUnknown`,
  `DependencyUnknownFamily`, `ProofReuseCandidateKey`,
  `KernelEvidenceDependencyInput`,
  `DependencySliceError`,
  `try_compute_dependency_slices`,
  `try_compute_dependency_slices_with_kernel_evidence`

Correspondence:

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Slice は VC ごとの deterministic record であり、VC order、kind/status boundary、entry、unknown、completeness、fingerprint を保持する。 | `try_compute_dependency_slices`、`DependencySliceSet`、`DependencySlice`、debug/fingerprint helpers。 | dependency ordering/debug/fingerprint tests、status-boundary tests、determinism suite。 | 実装済み。 |
| Dependency entry は local context、generated/core formula、definition、import、trace、policy、anchor、discharge evidence、seed mapping data を収集する。 | `VcSet` と optional `DischargeOutput` を読む slice collector helpers。 | `collects_dependency_classes_from_vc_ir_inputs`、discharge-evidence、pre-existing evidence、unused-context tests。 | 現在の explicit payload 向けに実装済み。 |
| Unknown coverage は fail closed で、uncacheable slice になる。 | `DependencyUnknown`、completeness/cache-miss helpers。 | conservative unknown、incomplete anchor、binder/context cycle、unavailable evidence tests。 | 実装済み。 |
| Reusable fingerprint は snapshot-local id を除外し、snapshot-local discharge evidence hash を正規化する。 | diagnostic local key ではなく stable payload を hash する fingerprint payload helpers。 | `reusable_fingerprint_excludes_snapshot_local_vc_id`、`reusable_fingerprint_normalizes_snapshot_local_discharge_hashes`、generated-formula-id shift と unresolved-payload tests。 | 現在の deterministic-discharge reuse candidate 向けに実装済み。 |
| Proof-reuse candidate key は complete anchor/slice、current matching slice computation、canonical VC/context fingerprint、compatible policy fingerprint、newly produced replayable deterministic discharge evidence、current kernel evidence handoff identity を要求する。 | `DependencySliceSet::proof_reuse_key_for_kernel_handoff`、fail-closed fallback の `DependencySliceSet::proof_reuse_key_for`、`KernelEvidenceDependencyInput`、`ProofReuseCandidateKey`、`proof_reuse_key`。 | `kernel_evidence_reuse_key_requires_handoff_and_tracks_kernel_hash`、`kernel_evidence_identity_participates_in_slice_and_reuse_key`、dependency-slice fail-closed tests。 | kernel-evidence invalidation と deterministic-discharge branch 向けに実装済み。proof-witness/cache/kernel acceptance consumer は external/deferred のまま。 |
| Public enum は forward-compatible である。 | dependency-slice public enum は `#[non_exhaustive]`。 | `vc_public_enums_are_forward_compatible_and_documented`。 | task 17 で guard 済み。 |

### `kernel_evidence_handoff`

Source path: `crates/mizar-vc/src/kernel_evidence_handoff.rs`.

literal top-level public item:

- `KERNEL_EVIDENCE_SCHEMA_VERSION`, `KERNEL_EVIDENCE_ENCODING_VERSION`,
  `VC_KERNEL_HANDOFF_SCHEMA`, `VC_TARGET_FINGERPRINT_ALGORITHM_ID`,
  `KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID`
- `KernelEvidenceHandoffInput`, `VcKernelEvidenceHandoff`,
  `KernelEvidenceEnvelope`, `KernelEvidenceProfile`,
  `KernelClauseTautologyPolicy`, `KernelCertificateHashInputAlgorithm`,
  `KernelEvidenceFingerprint`, `KernelManifestEntry`,
  `KernelFormulaPayload`, `KernelFormulaProjection`,
  `KernelImportedFormulaPayload`, `KernelImportedFormulaClass`,
  `KernelImportedFactRequirement`, `KernelRequiredProofStatus`,
  `KernelFormulaContextRequirements`, `KernelSubstitutionPayload`,
  `KernelFormulaEvidenceEntry`, `KernelFormulaSource`,
  `KernelSubstitutionEvidence`, `KernelEvidenceProvenance`,
  `KernelFinalGoalEvidence`, `KernelGoalPolarity`,
  `KernelEvidenceDiagnosticInputs`, `KernelDischargeDiagnostic`,
  `KernelEvidenceHandoffError`, `KernelEvidenceRole`,
  `build_kernel_evidence_handoff`

Correspondence:

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| builder は producer-side formula、substitution、provenance、target-binding evidence だけを package し、kernel、SAT solving、ATP backend を呼び出さない。 | `build_kernel_evidence_handoff` は `VcSet`、explicit payload slice、imported context requirement、optional discharge output だけを消費する。 | deterministic handoff、proof-hint exclusion、target-binding、prohibited-backend-material、discharge-diagnostic tests。 | explicit payload 向けに実装済み。 |
| proof-obligation handoff は explicit refutation polarity を宣言し、canonical package assembly 前に consistency polarity を拒否する。 | `KernelEvidenceHandoffInput::goal_polarity`、現在の `VcKind` 全列挙の `required_goal_polarity`、`KernelEvidenceHandoffError::GoalPolarityMismatch`、`KernelFinalGoalEvidence::polarity`。 | `consistency_goal_polarity_for_proof_obligation_fails_closed` と deterministic handoff polarity assertion。 | producer-side の現在 VC kind 向けに実装済み。checker-side acceptance binding は `mizar-kernel` task 30 に残る。 |
| canonical evidence は schema/encoding version、target VC、kernel profile、manifest、formula evidence、substitution、provenance、final goal を含む。imported context requirement と diagnostics は canonical hash input の外に残る。 | `KernelEvidenceEnvelope`、`target_fingerprint`、`VcKernelEvidenceHandoff::canonical_hash_input`、`canonical_hash_input`。 | deterministic hash/debug tests、proof-hint metadata target-binding test、imported-context tests。 | 実装済み。 |
| imported context / payload data は、empty context provenance、imported statement/formula fingerprint mismatch、unsupported formula fingerprint algorithm、missing context/payload data で fail closed する。返される context requirement は canonical sorted/deduplicated である。 | `validate_import_requirement`、`imported_payload_map`、`canonical_context_requirements`。 | imported context missing/mismatch、empty context provenance、unsupported algorithm、fingerprint mismatch、duplicate-context tests。 | 現在の explicit imported payload 向けに実装済み。 |
| formula、substitution source、provenance、manifest、side-condition、unsupported premise data の不足は fabrication ではなく fail closed する。 | `KernelEvidenceHandoffError` と validation helpers。 | missing payload、invalid projection、substitution missing-source/empty/side-condition、manifest tests。 | 現在の fail-closed case 向けに実装済み。upstream full formula/binder payload production は external のまま。 |
| substitution record は instantiated formula と target formula field を含めない。side-condition record は opaque deterministic kernel-compatible encoding であり、sort され、empty または duplicate の場合は拒否される。 | `KernelSubstitutionPayload`、`KernelSubstitutionEvidence`、`canonical_side_conditions`。 | `substitutions_reference_source_formula_without_instantiated_fields`、substitution side-condition fail-closed tests、input-order canonicalization tests。 | 実装済み。 |
| Public enum は forward-compatible である。 | handoff public enum は `#[non_exhaustive]`。 | `vc_public_enums_are_forward_compatible_and_documented`。 | task 25 で guard 済み。 |

## Cross-Module Evidence

| contract | source/test correspondence |
|---|---|
| Crate scaffolding and dependency boundary | `Cargo.toml`、`src/lib.rs`、`tests/lint_policy.rs`; manifest、workspace、dependency、module-export、allow-rationale tests が guard する。 |
| 同一 public input が同一 VC set、id、order、status、discharge evidence、slice を生成すること | `crates/mizar-vc/tests/determinism_suite.rs`; `identical_public_inputs_have_deterministic_pipeline_outputs`。 |
| deterministic discharge candidate 向け architecture-22 cross-edit reuse identity | `crates/mizar-vc/tests/determinism_suite.rs`; shifted `VcId`、shifted generated-formula id、stale slice、policy/context/goal change、pre-existing evidence、incomplete anchor、unresolved-payload checks。 |
| Kernel evidence handoff が producer-side かつ prover-independent に残ること | `crates/mizar-vc/src/kernel_evidence_handoff.rs`; deterministic handoff tests、fail-closed missing-payload tests、prohibited backend/legacy material tests。 |
| Public enum forward compatibility | source attribute、EN/JA module policy table、`vc_public_enums_are_forward_compatible_and_documented`。 |
| Active source-derived corpus coverage | active proof-verification corpus は未実装。Task 15 は fake `.miz` fixture ではなく deferred traceability row を記録する。 |

## Task 21 architecture-22 follow-up

Task 21 は Task 20 後にこの source/spec audit を再実行し、新しい未分類の source/spec drift は
記録しない。architecture-22 identity contract の deterministic-discharge branch は stable
generated payload 向けに実装済みである。paired
[architecture_22_audit.md](./architecture_22_audit.md) document が focused Task 21 artifact である。

## Task 22 module-boundary follow-up

Task 22 は module specification と repository crate-layout guidance に照らして source layout
を監査した。paired [module_boundary_audit.md](./module_boundary_audit.md) document は、
`vc_ir`、`generator`、`discharge`、`dependency_slice` が現在も public module responsibility
に一致することを記録する。いくつかの file は line count の面で maintenance watchlist に残るが、
closeout 前に必須の move-only split、public API move、source/spec drift は見つからなかった。

## Closeout follow-up

Closeout task は paired [crate_exit_report.md](./crate_exit_report.md) document を追加し、
final hard gate、quality score、verification、deferred item、handoff を記録する。Rust
source、public API、`.miz` fixture、expectation、`doc/spec`、traceability metadata、
runner behavior、downstream consumer は変更せず、新しい source/spec drift も記録しない。

## Task 24 kernel evidence handoff follow-up

Task 24 は、`mizar-kernel` task 23-29 が checker-side formula/substitution evidence
path、trusted SAT checker wrapper、SAT-backed check service、legacy-certificate
audit gate を導入した後、paired
[kernel_evidence_handoff.md](./kernel_evidence_handoff.md) document を追加する。この
task は `mizar-vc` では specification-only であり、Rust API、source module、`.miz`
fixture、expectation、`doc/spec` edit、SAT solving、kernel call、ATP backend
integration、proof policy、cache storage、artifact witness publication を追加しない。

新しい spec は producer-side handoff contract を記録する。`mizar-vc` は既存
`VcSet` / `VcIr` data から formula、substitution、provenance、target-binding
evidence を package してよいが、instantiated formula、SAT clause、backend proof
method、resolution trace、backend log、legacy certificate は trusted evidence の外に
残す。

## Task 25 kernel evidence handoff builder follow-up

Task 25 は `crates/mizar-vc/src/kernel_evidence_handoff.rs` を追加し、`src/lib.rs` に
module を登録し、lint-policy guard を更新し、builder を focused Rust tests で cover
する。この module は producer-side のままであり、explicit formula/substitution/provenance
payload から immutable handoff package を構築し、imported fact context requirement を
canonical envelope の外に運び、diagnostic discharge input を canonical hash input の外に
記録し、payload または context requirement が不足する場合は fail closed する。

この task は SAT solving、kernel call、ATP backend integration、backend proof method、
resolution trace、legacy certificate acceptance、artifact witness publication、cache
promotion、downstream proof policy を追加しない。

## Task 27 explicit goal polarity follow-up

Task 27 は kernel evidence handoff builder に producer-side `goal_polarity` input を追加し、
現在の各 `VcKind` が `AssertFalseForRefutation` を要求することを列挙し、検証済みの explicit value
を `final_goal.polarity` に記録し、proof obligation に対する `AssertTrueForConsistency`
request を `GoalPolarityMismatch` で拒否する。

この task は SAT solving、kernel call、ATP backend integration、checker/core semantic change、
formula/substitution/provenance payload の捏造、proof row、cache promotion、artifact witness
publication、expectation update を追加しない。trusted checker-side F1 binding は
`mizar-kernel` task 30 に割り当てたままである。

## Remaining Classified Follow-Ups

Task 18 は新しい source/spec correspondence gap を追加しなかった。Task 21 は
architecture-22 identity work 後にこの audit を再実行し、新しい未分類の source/spec gap を
記録しない。Task 22 は module-boundary gate を再実行し、closeout 前に必須の split はないと
記録する。Task 24 は downstream kernel classification を更新し、Task 25 は explicit
payload 向け VC producer-side handoff builder を実装し、Task 26 は canonical handoff hash を
dependency-slice / proof-reuse identity に接続する。Task 27 は producer-side F1 polarity
declaration/rejection contract を閉じるが、残る upstream full payload、checker-side F1、F2、
downstream consumer gap は解決しない。既存の分類済み record は残る:

- `external_dependency_gap`: active `proof_verification` runner support と
  source-to-core / source-to-VC extraction seam は `mizar-test` に存在しない。
  Task 15 が concrete deferred corpus obligation を記録済み。
- `external_dependency_gap` / `deferred`: `mizar-kernel` は現在 checker-side
  formula/substitution evidence acceptance path を所有し、`mizar-vc` は explicit payload
  向け producer-side handoff builder と reuse identity integration を所有するが、
  `mizar-atp` candidate evidence
  producer、`mizar-proof` / `mizar-cache` consumer、artifact witness consumer はまだ
  incomplete である。ATP translation、
  proof policy、cache lookup/reuse、artifact persistence はこの crate の外に残る。
- `external_dependency_gap`: registration/redefinition/reduction details、call-precondition、
  branch、match、range-loop、collection-loop、term-only termination、partial termination、
  Pick non-emptiness、ghost-erasure、complete trace family、source-derived core formula
  payload、definition payload、quantified binder payload、source-derived obligation payload
  family について、upstream explicit/stable payload はまだ不完全である。
- `deferred`: proof-witness hash、ATP/proof/cache validation、artifact consumer、
  source-derived runner integration は、architecture-22 reuse を deterministic discharge と
  current kernel-evidence handoff identity candidate key の外で受理する前に
  実装しなければならない。
- `deferred`: 大きい `vc_ir`、`generator`、`dependency_slice` implementation file 内の
  private helper / test split は、将来の任意の move-only maintenance task として実施してよいが、
  crate exit には不要である。

final quality review と crate-exit status は
[crate_exit_report.md](./crate_exit_report.md) に記録済みである。
`repo_metadata_conflict` は観測されなかった。
