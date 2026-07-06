# mizar-proof Source/Spec Correspondence Audit

> 正本は英語です。英語版:
> [../en/source_spec_audit.md](../en/source_spec_audit.md)。

## Scope

task 15 は、task 14 までに実装された public source surface を paired module
spec と照合する:

- [`policy.md`](./policy.md) と `crates/mizar-proof/src/policy.rs`;
- [`selection.md`](./selection.md) と `crates/mizar-proof/src/selection.rs`;
- [`status.md`](./status.md) と `crates/mizar-proof/src/status.rs`;
- [`witness_store.md`](./witness_store.md) と
  `crates/mizar-proof/src/witness_store.rs`;
- `crates/mizar-proof/src/lib.rs` の public module exposure;
- `crates/mizar-proof/tests/` の integration / lint coverage。

この audit では blocking な `spec_gap`、`test_gap`、`design_drift`、
`source_drift`、`boundary_violation`、`repo_metadata_conflict` は見つからなかった。
task 15 は未 exercise だった 3 つの validation path に focused unit coverage を
追加するが、production behavior は変更しない。

## Trust Boundary Restatement

各 module spec は policy/trust split を再掲しており、各 source module にも
対応する crate/module-level boundary comment がある:

| Module | Spec statement | Source statement | Result |
|---|---|---|---|
| crate root | `todo.md` は `mizar-proof` を policy/selection/status/witness owner と定義し、proof acceptor とはしない。 | `src/lib.rs` は trusted acceptance が accepted proof-obligation `mizar-kernel` `KernelCheckResult` だけに由来し、backend/external/cache/consistency-check material は昇格しないと記す。 | consistent |
| policy | `policy.md` は evaluator が proof acceptor ではなく、trusted `used_axioms` は accepted proof-obligation kernel result だけに由来すると記す。Accepted consistency check は diagnostic-only である。 | `src/policy.rs` は normalized input の分類、kernel-checkable evidence の scheduling、non-trusted evidence の記録、accepted consistency check の `DiagnosticOnly` 扱い、policy-tainted proof-obligation kernel output の non-trusted external-policy 扱いだけを行う。 | consistent |
| selection | `selection.md` は selection が分類済み evidence を順序付けるだけで、backend/kernel/status/witness/cache/artifact commit を実行しないと記す。 | `src/selection.rs` は deterministic selection と merge だけを公開し、trusted winner は proof-obligation `KernelPolicyInput` 由来の `TrustedKernelEvidence` を要求し、accepted consistency check は non-trusted に保つ。 | consistent |
| status | `status.md` は projection が proof acceptance ではなく、trusted `used_axioms` は trusted class として選択された accepted proof-obligation kernel result だけに由来すると記す。 | `src/status.rs` は selected class を projection し、policy fingerprint を検証し、accepted goal polarity を trusted proof-obligation selection からだけ記録し、`TrustedUsedAxiomsRef` を proof-obligation `KernelCheckResult` からだけ導出する。 | consistent |
| witness store | `witness_store.md` は staging/publication が proof authority ではなく、accepted proof-obligation kernel metadata と committed artifact reachability を要求すると記す。 | `src/witness_store.rs` は trusted metadata と committed publication proof の production constructor を非公開にし、unsupported/non-trusted publication path を reject する。 | consistent |

## Public API Trace

### Policy

| Public API group | Spec coverage | Test coverage | Result |
|---|---|---|---|
| `POLICY_SCHEMA_VERSION`、`DEFAULT_CHECKER_SCHEMA_VERSION`、`VerifierPolicy` constructors/accessors/builders/`policy_fingerprint` | `policy.md` の verifier policy setting、build mode、fingerprint surface。 | `fingerprint_changes_for_each_policy_relevant_setting`、`fingerprint_sorts_kernel_evidence_formats_and_ignores_candidate_order`、lint package metadata guard。 | consistent |
| `ProofPolicyEvaluator::{new, policy, policy_fingerprint, candidate_class, can_schedule_kernel_check, evaluate_candidate, external_evidence_admission}` | `policy.md` の candidate class、kernel scheduling policy、external admission matrix、open obligation、policy assumption、policy diagnostic。 | `src/policy.rs` の policy classification、scheduling、external admission matrix、rejection diagnostic、policy-tainted kernel result、negative scheduling tests。 | consistent |
| `ProofPolicyEvaluator::{best_possible_early_stop_class, portfolio_early_stop_decision}`、`PortfolioEarlyStopInput`、`PortfolioEarlyStopDecision`、`PortfolioEarlyStopClass`、`PortfolioEarlyStopReason` | `policy.md` の early-stop policy query section。 | early-stop equivalence、equal/higher pending block、external/kernel-certificate、policy-tainted normalization、public API tests。 | consistent |
| `PolicyFingerprint` | `policy.md` の stable policy fingerprint section。 | fingerprint unit tests と determinism suite。 | consistent |
| `BuildMode`、`ExternalEvidenceMode`、`OpenObligationMode`、`PolicyAssumptionMode`、`KernelEvidenceFormat`、`CandidatePolicyClass`、`KernelEvidenceOrigin`、`AcceptedGoalPolarity`、`PolicyCandidate`、`BackendProofPayloadKind`、`ExternalEvidencePublicationStatus`、`PolicyDiagnosticCategory`、`PolicyReasonCode` | `policy.md` の candidate class、input、diagnostic、accepted-goal-polarity、public enum policy section。 | policy unit tests と `public_enums_are_forward_compatible_and_documented`。 | consistent |
| `KernelPolicyInput::from_kernel_result` と accessors | `policy.md` の explicit kernel-origin wrapper と accepted-goal-polarity requirement。 | kernel-origin classification、policy-taint、accepted proof-obligation polarity、consistency-check exclusion、downstream trusted-marker tests。 | consistent |
| `PolicyDecision`、`ExternalEvidenceAdmission`、`PolicyDiagnostic` | `policy.md` の external evidence admission、kernel-evidence check-kind preservation、diagnostics section。 | external admission matrix、rejected evidence check-kind routing、diagnostic stability tests。 | consistent |

### Selection

| Public API group | Spec coverage | Test coverage | Result |
|---|---|---|---|
| `CandidateSourceId`、`SelectionInputError`、`DiagnosticRef` | `selection.md` の stable candidate id と diagnostic-ref requirement。 | empty id、duplicate/conflicting id、diagnostic ordering、lint public-enum coverage。 | consistent |
| `TrustedKernelEvidence` | `selection.md` の accepted proof-obligation kernel evidence hash と accepted goal polarity に bind された trusted marker。 | trusted marker spoofing、policy-tainted rejection、accepted-consistency diagnostic-only rejection、accepted-polarity propagation、trusted class tests。 | consistent |
| `ProofEvidenceCandidate`、`ProofEvidenceSet` | `selection.md` の normalized selection input、tie-break input、stable candidate id、optional proof witness/discharge hash。 | winner-order、tie-break、duplicate input、pending kernel-checkable、shuffled arrival tests。 | consistent |
| `ProofWinnerClass`、`ProofSelection`、`ProofSelector`、`select_winner`、`SelectedReuseMetadata`、`ProofWitnessPublication` | `selection.md` の winner class、deterministic ordering、accepted goal polarity を含む selected reuse metadata、no-selectable result shape、witness-publication gap rule。 | invalid SAT refutation、proof-obligation goal-polarity context mismatch、missing provenance に対する real-payload terminal kernel rejection fallback coverage、negative target-context-mismatch、consistency-check goal-polarity mismatch、non-open winner coverage、selector equivalence、no-selectable diagnostics、reuse metadata、determinism suite。 | consistent |
| `VcProofSelection`、`ProofSelectionSource`、`ArtifactProofSelection`、`ArtifactProofSelectionError`、`merge_artifact_proof_selections` | `selection.md` の artifact proof selection merge section。 | merge canonical order、source/class compatibility、duplicate source、trusted precedence、same-class tie-break、non-trusted preservation tests。 | consistent |

### Status

| Public API group | Spec coverage | Test coverage | Result |
|---|---|---|---|
| `ObligationAnchor`、`ProofObligationIdentity`、`ExplanationRef` | `status.md` の stable obligation identity と explanation reference。 | empty identity checks、projection fixtures、reuse metadata tests。 | consistent |
| `TrustedUsedAxiomsRef`、`TrustedUsedAxiomsError` | `status.md` の trusted `used_axioms` boundary。 | accepted proof-obligation kernel result constructor、non-accepted/policy-tainted/consistency-check rejection、missing/mismatched evidence hash、non-trusted status rejection tests。 | consistent |
| `ProofStatusProjectionInput`、`project_status`、`ProofStatusProjection`、`StatusProjectionError` | `status.md` の projection input、stale/mismatched input rejection、final projection shape。 | projection class fixtures、policy-fingerprint mismatch、trusted-axiom validation、explanation/diagnostic metadata tests。 | consistent |
| `ProjectedProofStatus`、`CurrentArtifactObligationStatus`、`ArtifactPublicationGap`、`ArtifactStatusPublication` | `status.md` の status model、artifact projection limits、public enum policy。 | status mapping、current artifact publication gaps、`KernelVerified` witness requirement、`DischargedBuiltin`/`PolicyAssumed` external dependency gaps、lint public-enum coverage。 | consistent |
| `StatusReuseMetadata` | `status.md` の proof reuse metadata section。accepted goal polarity を proof-evidence identity に含む。 | architecture-22 identity と reuse metadata coverage tests、accepted-polarity hash invalidation、terminal rejection projection、determinism suite。 | current task scope では consistent。downstream cache consumer はこの crate の外側に残る。 |

### Witness Store

| Public API group | Spec coverage | Test coverage | Result |
|---|---|---|---|
| `ProofWitnessPayloadSchema` | `witness_store.md` の payload schema identity と canonical-byte policy。 | malformed schema と canonical-byte rejection tests。 | consistent |
| `ProofWitnessProvenance` | `witness_store.md` の provenance metadata と cache/reuse boundary。 | provenance/status consistency、trusted used-axiom provenance、dependency fingerprint preservation、advisory backend ref tests。 | consistent |
| `TrustedKernelWitnessMetadata` | `witness_store.md` の opaque trusted kernel metadata token。 | test-only constructor coverage。production には public constructor がない。 | consistent。`WITNESS11-G002` は `external_dependency_gap` のまま。 |
| `ProofWitnessDraft`、`stage`、`ProofWitnessStagedRef` | `witness_store.md` の draft/staging state、stable hashing、path/schema validation、unpublished `ProofWitnessRef` candidate。 | stage round-trip、stable hash、selected witness hash validation、path/schema/hash-class rejection、non-trusted rejection、`DischargedBuiltin` internal-staging tests。 | consistent |
| `CommittedWitnessPublicationProof`、`publish_ref`、`ProofWitnessPublishedRef` | `witness_store.md` の committed artifact reachability proof と publication state。 | publish-before-proof failure、stale build snapshot、missing/duplicate manifest/artifact references、exact coverage、published-ref determinism tests。 | consistent。`WITNESS11-G001` は `external_dependency_gap` のまま。 |
| `ProofWitnessStoreError` | `witness_store.md` の failure semantics と public enum policy。 | store error fixtures と `public_enums_are_forward_compatible_and_documented`。 | consistent |

## Gap Classification

この audit は次の remaining gap を placeholder integration なしに保持する:

| ID | Class | Owner / trigger | Current handling |
|---|---|---|---|
| `PROOF15-G001` | `deferred` | task 17 / future `mizar-cache` consumer | Selection と status は現在の reuse metadata を validation data としてだけ公開する。より広い cache-facing export contract は task 17 に残り、cache lookup や proof-status promotion は存在しない。 |
| `PROOF15-G002` | `external_dependency_gap` | `mizar-artifact` witness schema | `DischargedBuiltin` は trusted internal selection/status class のままだが、`ProofWitnessRef` は publish できない。selection/status/witness-store は gap を明示したままにする。 |
| `PROOF15-G003` | `external_dependency_gap` | Artifact committed-publication token | `CommittedWitnessPublicationProof` は `mizar-proof` 内に production constructor を持たない。publication は artifact-owned reachability proof を待つ。 |
| `PROOF15-G004` | `external_dependency_gap` | Kernel/artifact copied acceptance metadata | `TrustedKernelWitnessMetadata` は `mizar-proof` 内に production constructor を持たない。trusted witness draft は caller-synthesized metadata から作れない。 |
| `PROOF15-G005` | `deferred` | Concrete witness payload producers | witness store は schema identity と exact bytes を hash し、canonical-byte payload の empty bytes を reject するが、byte-level canonicality validation は producer-owned のまま。 |
| `PROOF15-G006` | `external_dependency_gap` | Downstream ATP portfolio integration | Early-stop API は stable かつ policy-owned。live backend cancellation/adoption wiring は `mizar-atp` 側に残り、ここでは stub しない。 |
| `PROOF15-G007` | `external_dependency_gap` | public legacy-certificate proof payload / runner | `mizar-proof` は real `unsupported_certificate_format` kernel rejection record を policy-open fallback に対して terminal として扱うが、proof test で legacy certificate payload は fabricate しない。real-payload coverage は public kernel fixture または source runner を待つ。 |

task-15 source/spec audit 中に `repo_metadata_conflict` は観測されなかった。
後続の ATP closeout metadata conflict は tasks 18-20 で記録され、focused correction
commit `36d1a9c` で解消済みである。

task 21 はこの audit を訂正済み kernel rejection taxonomy へ拡張する。terminal
kernel/certificate rejection は `KernelRejected` のまま、policy-open fallback は
source-backed corrected rejection を隠さず、accepted goal polarity は trusted accepted
proof-obligation selection に対してだけ export される。legacy
unsupported-certificate gate の remaining real-payload coverage gap は上で分類する。

## Conclusion

task 15 は paired audit documentation と focused validation tests を追加する。task 14
までの implemented public API surface は、この追加後の module specification と test
coverage に対応している。未完成 integration point は上で `deferred` または
`external_dependency_gap` と分類済みであり、いずれも externally attested evidence、
backend diagnostics、cache records、witness metadata、policy assumptions を trusted
proof acceptance または trusted `used_axioms` に昇格させない。
