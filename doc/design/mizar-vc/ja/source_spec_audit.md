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
- `vc_ir`

対応する source path:

- `crates/mizar-vc/src/dependency_slice.rs`
- `crates/mizar-vc/src/discharge.rs`
- `crates/mizar-vc/src/generator.rs`
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
  `VcStatusAction`, `VcIr`, `VcSourceRef`, `SeedVcRef`,
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
| Policy status は明示的に保持され、discharge evidence にはならない。 | `try_discharge` の status-preserving path。 | `policy_and_deferred_statuses_are_preserved_without_discharge_evidence`、determinism suite。 | 実装済み。 |
| Public enum は forward-compatible である。 | discharge public enum は `#[non_exhaustive]`。 | `vc_public_enums_are_forward_compatible_and_documented`。 | task 17 で guard 済み。 |

### `dependency_slice`

Source path: `crates/mizar-vc/src/dependency_slice.rs`.

literal top-level public item:

- `DEPENDENCY_SLICE_SCHEMA_VERSION`
- `DependencySliceInput`, `DependencySliceSet`, `DependencySlice`,
  `DependencySliceFingerprint`, `DependencySliceCompleteness`,
  `DependencyEntry`, `DependencyEntryClass`, `DependencyUnknown`,
  `DependencyUnknownFamily`, `DependencySliceError`,
  `try_compute_dependency_slices`

Correspondence:

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Slice は VC ごとの deterministic record であり、VC order、kind/status boundary、entry、unknown、completeness、fingerprint を保持する。 | `try_compute_dependency_slices`、`DependencySliceSet`、`DependencySlice`、debug/fingerprint helpers。 | dependency ordering/debug/fingerprint tests、status-boundary tests、determinism suite。 | 実装済み。 |
| Dependency entry は local context、generated/core formula、definition、import、trace、policy、anchor、discharge evidence、seed mapping data を収集する。 | `VcSet` と optional `DischargeOutput` を読む slice collector helpers。 | `collects_dependency_classes_from_vc_ir_inputs`、discharge-evidence、pre-existing evidence、unused-context tests。 | 現在の explicit payload 向けに実装済み。 |
| Unknown coverage は fail closed で、uncacheable slice になる。 | `DependencyUnknown`、completeness/cache-miss helpers。 | conservative unknown、incomplete anchor、binder/context cycle、unavailable evidence tests。 | 実装済み。 |
| Reusable fingerprint は snapshot-local `VcId` を除外し、snapshot-local discharge evidence hash を正規化する。 | fingerprint payload helpers。 | `reusable_fingerprint_excludes_snapshot_local_vc_id`、`reusable_fingerprint_normalizes_snapshot_local_discharge_hashes`、anchor hash tests。 | 実装済み。full cross-edit reuse identity は Task 20。 |
| Public enum は forward-compatible である。 | dependency-slice public enum は `#[non_exhaustive]`。 | `vc_public_enums_are_forward_compatible_and_documented`。 | task 17 で guard 済み。 |

## Cross-Module Evidence

| contract | source/test correspondence |
|---|---|
| Crate scaffolding and dependency boundary | `Cargo.toml`、`src/lib.rs`、`tests/lint_policy.rs`; manifest、workspace、dependency、module-export、allow-rationale tests が guard する。 |
| 同一 public input が同一 VC set、id、order、status、discharge evidence、slice を生成すること | `crates/mizar-vc/tests/determinism_suite.rs`; `identical_public_inputs_have_deterministic_pipeline_outputs`。 |
| Public enum forward compatibility | source attribute、EN/JA module policy table、`vc_public_enums_are_forward_compatible_and_documented`。 |
| Active source-derived corpus coverage | active proof-verification corpus は未実装。Task 15 は fake `.miz` fixture ではなく deferred traceability row を記録する。 |

## Remaining Classified Follow-Ups

Task 18 は新しい source/spec correspondence gap を追加しない。既存の分類済み record は残る:

- `external_dependency_gap`: active `proof_verification` runner support と
  source-to-core / source-to-VC extraction seam は `mizar-test` に存在しない。
  Task 15 が concrete deferred corpus obligation を記録済み。
- `external_dependency_gap`: `mizar-atp`、`mizar-kernel`、`mizar-proof`、`mizar-cache`
  は active workspace consumer ではない。そのため ATP translation、certificate acceptance、
  proof policy、cache lookup/reuse、artifact persistence はこの crate の外に残る。
- `external_dependency_gap`: registration/redefinition/reduction details、call-precondition、
  branch、match、range-loop、collection-loop、term-only termination、partial termination、
  Pick non-emptiness、ghost-erasure、complete trace family の一部について、upstream explicit
  payload はまだ不完全である。
- `deferred`: Task 20 は canonical VC/context identity と consumer policy seam が揃った時点で、
  architecture 22 に対する `ObligationAnchor` と cross-edit reuse identity follow-through を所有する。
- `deferred`: Task 21 は architecture-22 follow-up audit、Task 22 は module-boundary refactor
  gate、closeout は final quality review と crate-exit status を記録する。

`repo_metadata_conflict` は観測されなかった。
