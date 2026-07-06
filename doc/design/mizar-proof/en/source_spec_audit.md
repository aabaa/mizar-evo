# mizar-proof Source/Spec Correspondence Audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_audit.md](../ja/source_spec_audit.md).

## Scope

Task 15 audits the public source surface implemented through task 14 against
the paired module specifications:

- [`policy.md`](./policy.md) and `crates/mizar-proof/src/policy.rs`;
- [`selection.md`](./selection.md) and `crates/mizar-proof/src/selection.rs`;
- [`status.md`](./status.md) and `crates/mizar-proof/src/status.rs`;
- [`witness_store.md`](./witness_store.md) and
  `crates/mizar-proof/src/witness_store.rs`;
- public module exposure in `crates/mizar-proof/src/lib.rs`;
- integration and lint coverage in `crates/mizar-proof/tests/`.

The audit found no blocking `spec_gap`, `test_gap`, `design_drift`,
`source_drift`, `boundary_violation`, or `repo_metadata_conflict`. Task 15 adds
focused unit coverage for three previously unexercised validation paths, but
does not change production behavior.

## Trust Boundary Restatement

Every module spec restates the policy/trust split, and each source module has a
matching crate or module-level boundary comment:

| Module | Spec statement | Source statement | Result |
|---|---|---|---|
| crate root | `todo.md` defines `mizar-proof` as policy/selection/status/witness owner, not a proof acceptor. | `src/lib.rs` states that trusted acceptance comes only from accepted proof-obligation `mizar-kernel` `KernelCheckResult` values and that backend/external/cache/consistency-check material is not promoted. | consistent |
| policy | `policy.md` says the evaluator is not a proof acceptor and trusted `used_axioms` come only from accepted proof-obligation kernel results. Accepted consistency checks are diagnostic-only. | `src/policy.rs` only classifies normalized inputs, schedules kernel-checkable evidence, records non-trusted evidence, treats accepted consistency checks as `DiagnosticOnly`, and handles policy-tainted proof-obligation kernel output as non-trusted external policy evidence. | consistent |
| selection | `selection.md` says selection orders already classified evidence and does not run backends, kernel checks, status projection, witness staging, cache lookup, or artifact commit. | `src/selection.rs` exposes deterministic selection and merge only; trusted winners require `TrustedKernelEvidence` built from proof-obligation `KernelPolicyInput`, while accepted consistency checks stay non-trusted. | consistent |
| status | `status.md` says projection is not proof acceptance and trusted `used_axioms` originate only from accepted proof-obligation kernel results selected as trusted classes. | `src/status.rs` projects selected classes, validates policy fingerprint, records accepted goal polarity only from trusted proof-obligation selections, and derives `TrustedUsedAxiomsRef` only from proof-obligation `KernelCheckResult` values. | consistent |
| witness store | `witness_store.md` says staging/publishing is not proof authority and requires accepted proof-obligation kernel metadata plus committed artifact reachability. | `src/witness_store.rs` keeps trusted metadata and committed publication proof constructors non-public in production and rejects unsupported/non-trusted publication paths. | consistent |

## Public API Trace

### Policy

| Public API group | Spec coverage | Test coverage | Result |
|---|---|---|---|
| `POLICY_SCHEMA_VERSION`, `DEFAULT_CHECKER_SCHEMA_VERSION`, `VerifierPolicy` constructors, accessors, builders, and `policy_fingerprint` | Verifier policy settings, build modes, and fingerprint surface in `policy.md`. | `fingerprint_changes_for_each_policy_relevant_setting`, `fingerprint_sorts_kernel_evidence_formats_and_ignores_candidate_order`, lint package metadata guard. | consistent |
| `ProofPolicyEvaluator::{new, policy, policy_fingerprint, candidate_class, can_schedule_kernel_check, evaluate_candidate, external_evidence_admission}` | Candidate classes, kernel scheduling policy, external admission matrix, open obligations, policy assumptions, and policy diagnostics in `policy.md`. | Policy classification, scheduling, external admission matrix, rejection diagnostics, policy-tainted kernel result, and negative scheduling tests in `src/policy.rs`. | consistent |
| `ProofPolicyEvaluator::{best_possible_early_stop_class, portfolio_early_stop_decision}`, `PortfolioEarlyStopInput`, `PortfolioEarlyStopDecision`, `PortfolioEarlyStopClass`, `PortfolioEarlyStopReason` | Early-stop policy query section in `policy.md`. | Early-stop equivalence, equal/higher pending block, external/kernel-certificate, policy-tainted normalization, and public API tests. | consistent |
| `PolicyFingerprint` | Stable policy fingerprint section in `policy.md`. | Fingerprint unit tests and determinism suite. | consistent |
| `BuildMode`, `ExternalEvidenceMode`, `OpenObligationMode`, `PolicyAssumptionMode`, `KernelEvidenceFormat`, `CandidatePolicyClass`, `KernelEvidenceOrigin`, `AcceptedGoalPolarity`, `PolicyCandidate`, `BackendProofPayloadKind`, `ExternalEvidencePublicationStatus`, `PolicyDiagnosticCategory`, `PolicyReasonCode` | Candidate class, input, diagnostic, accepted-goal-polarity, and public enum policy sections in `policy.md`. | Policy unit tests plus `public_enums_are_forward_compatible_and_documented`. | consistent |
| `KernelPolicyInput::from_kernel_result` and accessors | Explicit kernel-origin wrapper and accepted-goal-polarity requirements in `policy.md`. | Kernel-origin classification, policy-taint, accepted proof-obligation polarity, consistency-check exclusion, and downstream trusted-marker tests. | consistent |
| `PolicyDecision`, `ExternalEvidenceAdmission`, `PolicyDiagnostic` | External evidence admission, kernel-evidence check-kind preservation, and diagnostics sections in `policy.md`. | External admission matrix, rejected evidence check-kind routing, and diagnostic stability tests. | consistent |

### Selection

| Public API group | Spec coverage | Test coverage | Result |
|---|---|---|---|
| `CandidateSourceId`, `SelectionInputError`, `DiagnosticRef` | Stable candidate id and diagnostic-ref requirements in `selection.md`. | Empty id, duplicate/conflicting id, diagnostic ordering, and lint public-enum coverage. | consistent |
| `TrustedKernelEvidence` | Trusted marker bound to accepted proof-obligation kernel evidence hash and accepted goal polarity in `selection.md`. | Trusted marker spoofing, policy-tainted rejection, accepted-consistency diagnostic-only rejection, accepted-polarity propagation, and trusted class tests. | consistent |
| `ProofEvidenceCandidate`, `ProofEvidenceSet` | Normalized selection inputs, tie-break inputs, stable candidate ids, optional proof witness and discharge hashes in `selection.md`. | Winner-order, tie-break, duplicate input, pending kernel-checkable, and shuffled arrival tests. | consistent |
| `ProofWinnerClass`, `ProofSelection`, `ProofSelector`, `select_winner`, `SelectedReuseMetadata`, `ProofWitnessPublication` | Winner classes, deterministic ordering, selected reuse metadata including accepted goal polarity, no-selectable result shape, and witness-publication gap rules in `selection.md`. | Ordering fixtures, real-payload terminal kernel rejection fallback coverage for invalid SAT refutation, proof-obligation goal-polarity context mismatch, and missing provenance; negative target-context-mismatch, consistency-check goal-polarity mismatch, and non-open winner coverage; selector equivalence; no-selectable diagnostics; reuse metadata; determinism suite. | consistent |
| `VcProofSelection`, `ProofSelectionSource`, `ArtifactProofSelection`, `ArtifactProofSelectionError`, `merge_artifact_proof_selections` | Artifact proof selection merge section in `selection.md`. | Merge canonical order, source/class compatibility, duplicate source, trusted precedence, same-class tie-break, and non-trusted preservation tests. | consistent |

### Status

| Public API group | Spec coverage | Test coverage | Result |
|---|---|---|---|
| `ObligationAnchor`, `ProofObligationIdentity`, `ExplanationRef` | Stable obligation identity and explanation references in `status.md`. | Empty identity checks, projection fixtures, and reuse metadata tests. | consistent |
| `TrustedUsedAxiomsRef`, `TrustedUsedAxiomsError` | Trusted `used_axioms` boundary in `status.md`. | Accepted proof-obligation kernel result constructor, non-accepted/policy-tainted/consistency-check rejection, missing/mismatched evidence hash, and non-trusted status rejection tests. | consistent |
| `ProofStatusProjectionInput`, `project_status`, `ProofStatusProjection`, `StatusProjectionError` | Projection inputs, stale/mismatched input rejection, and final projection shape in `status.md`. | Projection class fixtures, policy-fingerprint mismatch, trusted-axiom validation, and explanation/diagnostic metadata tests. | consistent |
| `ProjectedProofStatus`, `CurrentArtifactObligationStatus`, `ArtifactPublicationGap`, `ArtifactStatusPublication` | Status model, artifact projection limits, and public enum policy in `status.md`. | Status mapping, current artifact publication gaps, `KernelVerified` witness requirement, `DischargedBuiltin`/`PolicyAssumed` external dependency gaps, and lint public-enum coverage. | consistent |
| `StatusReuseMetadata` | Proof reuse metadata section in `status.md`, including accepted goal polarity in proof-evidence identity. | Architecture-22 identity and reuse metadata coverage tests, accepted-polarity hash invalidation, terminal rejection projection, plus determinism suite. | consistent for the current task scope; downstream cache consumers remain outside this crate. |

### Witness Store

| Public API group | Spec coverage | Test coverage | Result |
|---|---|---|---|
| `ProofWitnessPayloadSchema` | Payload schema identity and canonical-byte policy in `witness_store.md`. | Malformed schema and canonical-byte rejection tests. | consistent |
| `ProofWitnessProvenance` | Provenance metadata and cache/reuse boundary in `witness_store.md`. | Provenance/status consistency, trusted used-axiom provenance, dependency fingerprint preservation, and advisory backend ref tests. | consistent |
| `TrustedKernelWitnessMetadata` | Opaque trusted kernel metadata token in `witness_store.md`. | Test-only constructor coverage; production has no public constructor. | consistent; `WITNESS11-G002` remains `external_dependency_gap`. |
| `ProofWitnessDraft`, `stage`, `ProofWitnessStagedRef` | Draft and staging states, stable hashing, path/schema validation, and unpublished `ProofWitnessRef` candidate in `witness_store.md`. | Stage round-trip, stable hash, selected witness hash validation, path/schema/hash-class rejection, non-trusted rejection, and `DischargedBuiltin` internal-staging tests. | consistent |
| `CommittedWitnessPublicationProof`, `publish_ref`, `ProofWitnessPublishedRef` | Committed artifact reachability proof and publication state in `witness_store.md`. | Publish-before-proof failure, stale build snapshot, missing/duplicate manifest/artifact references, exact coverage, and published-ref determinism tests. | consistent; `WITNESS11-G001` remains `external_dependency_gap`. |
| `ProofWitnessStoreError` | Failure semantics and public enum policy in `witness_store.md`. | Store error fixtures and `public_enums_are_forward_compatible_and_documented`. | consistent |

## Gap Classification

The audit preserves these remaining gaps without introducing placeholder
integration:

| ID | Class | Owner / trigger | Current handling |
|---|---|---|---|
| `PROOF15-G001` | `deferred` | Task 17 / future `mizar-cache` consumer | Selection and status expose current reuse metadata as validation data only. The broader cache-facing export contract remains task 17; no cache lookup or proof-status promotion exists. |
| `PROOF15-G002` | `external_dependency_gap` | `mizar-artifact` witness schema | `DischargedBuiltin` remains a trusted internal selection/status class but cannot publish a `ProofWitnessRef`; selection/status/witness-store keep the gap explicit. |
| `PROOF15-G003` | `external_dependency_gap` | Artifact committed-publication token | `CommittedWitnessPublicationProof` has no production constructor in `mizar-proof`; publication waits for artifact-owned reachability proof. |
| `PROOF15-G004` | `external_dependency_gap` | Kernel/artifact copied acceptance metadata | `TrustedKernelWitnessMetadata` has no production constructor in `mizar-proof`; trusted witness drafts cannot be built from caller-synthesized metadata. |
| `PROOF15-G005` | `deferred` | Concrete witness payload producers | The witness store hashes schema identity and exact bytes and rejects empty canonical-byte payloads, but byte-level canonicality validation remains producer-owned. |
| `PROOF15-G006` | `external_dependency_gap` | Downstream ATP portfolio integration | Early-stop API is stable and policy-owned; live backend cancellation/adoption wiring remains in `mizar-atp` and is not stubbed here. |
| `PROOF15-G007` | `external_dependency_gap` | Public legacy-certificate proof payload/runner | `mizar-proof` recognizes real `unsupported_certificate_format` kernel rejection records as terminal against policy-open fallback, but does not fabricate a legacy certificate payload in proof tests. Real-payload coverage waits on a public kernel fixture or source runner. |

No `repo_metadata_conflict` was observed during the task-15 source/spec audit.
The later ATP closeout metadata conflict was recorded by tasks 18-20 and
resolved by focused correction commit `36d1a9c`.

Task 21 extends this audit for the corrected kernel rejection taxonomy.
Terminal kernel/certificate rejections stay `KernelRejected`, policy-open
fallback does not mask the source-backed corrected rejections, and accepted
goal polarity is exported only for trusted accepted proof-obligation
selections. The remaining real-payload coverage gap for the legacy
unsupported-certificate gate is classified above.

## Conclusion

Task 15 adds paired audit documentation and focused validation tests. The
implemented public API surface through task 14 matches the module
specifications and test coverage after those additions. Remaining incomplete
integration points are classified above as `deferred` or
`external_dependency_gap`; none permits externally attested evidence, backend
diagnostics, cache records, witness metadata, or policy assumptions to become
trusted proof acceptance or trusted `used_axioms`.
