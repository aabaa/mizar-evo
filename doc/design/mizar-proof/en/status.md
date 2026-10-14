# mizar-proof Status Projection

> Canonical language: English. Japanese companion: [../ja/status.md](../ja/status.md).

## Purpose

`mizar-proof` owns the projection from deterministic proof selections into
artifact-facing and diagnostics-facing proof status records.

Status projection is not proof acceptance. Trusted acceptance and trusted
`used_axioms` originate only from `mizar-kernel` `KernelCheckResult` values
whose status is `KernelCheckStatus::Accepted`, whose evidence check kind is
`ProofObligation`, and that are selected as `KernelVerified` or
`DischargedBuiltin`. Externally attested evidence, backend diagnostics,
backend-reported axiom lists, proof cache records, witness metadata, policy
assumptions, and accepted consistency checks never become kernel-verified
status or trusted `used_axioms`.

## Inputs

Projection consumes:

- the `ArtifactProofSelection` stream produced by deterministic selection
  merge;
- the active `VerifierPolicy` or an equivalent publication profile for the
  current build;
- stable obligation identity supplied by the VC/artifact producer:
  `VcId`, obligation id, `ObligationAnchor`, obligation fingerprint, canonical
  VC fingerprint, canonical local-context fingerprint, and dependency-slice
  fingerprint;
- optional trusted kernel result references for selected trusted classes,
  bound to the selected accepted evidence hash;
- optional diagnostic or explanation references produced by the policy,
  kernel, ATP, or diagnostics owner.

Projection must reject stale or mismatched inputs. A `VcId`,
`ObligationAnchor`, arrival order, completion time, runtime duration, backend
profile runtime, or cache hit alone is never proof identity and is never
sufficient for status reuse.

## Status Model

The internal projection result keeps the selection winner class and the
projected obligation status separate:

| Winner class | Projected obligation status | Trusted | Notes |
|---|---|---:|---|
| `KernelVerified` | `accepted` | yes | Requires a matching accepted proof-obligation kernel result. Projecting the accepted artifact status as publishable also requires matching selected kernel-witness metadata; committed witness refs are published separately by `witness_store::publish_ref`. |
| `DischargedBuiltin` | `accepted` | yes | Requires a matching accepted proof-obligation kernel result. Final artifact witness publication remains an `external_dependency_gap` until artifact schema support exists. |
| `PolicyPermittedExternal` | `externally_attested` | no | Policy-controlled evidence only. It carries no trusted `used_axioms`. |
| `PolicyAssumed` | `policy_assumed` internal status | no | Must remain distinct from accepted and externally attested status. Current artifact schema lacks this public obligation status, so artifact publication is `external_dependency_gap` unless a later schema adds it. |
| `PolicyOpen` | `open` | no | Carries an explanation reference when available. |
| `Rejected` | `rejected` | no | Carries the selected rejection or policy/kernel diagnostic reference. |
| `NoSelectableEvidence` | `open` or `rejected` | no | `open` only when the active policy permits publishing open obligations through `AllowPolicyOpen`; `RecordDiagnostic` is diagnostics-only and projects to `rejected`. It carries a no-selectable-evidence explanation. |

`not_required` is reserved for producer-owned obligations that do not require a
proof selection. It is not emitted from `ArtifactProofSelection`.

The projection must never collapse `DischargedBuiltin` into `KernelVerified`,
`PolicyAssumed` into `externally_attested`, or any non-trusted status into
`accepted`.

## Trusted `used_axioms`

Trusted `used_axioms` projection is allowed only when all of the following hold:

1. the selected winner class is `KernelVerified` or `DischargedBuiltin`;
2. the selected proof selection reports trusted `used_axioms` availability;
3. the projection input includes the accepted proof-obligation kernel result
   or a trusted kernel-owned reference derived from it;
4. the accepted proof-obligation kernel evidence hash matches the selection's
   selected evidence hash;
5. the kernel result is not policy-tainted, has status `Accepted`, and has
   evidence check kind `ProofObligation`.

When these conditions hold, projection may expose a trusted used-axiom
reference or ordered axiom list exactly as supplied by the kernel-owned
accepted result. Projection must not merge in backend-reported axiom lists,
externally attested citations, cache dependency records, diagnostic hints, or
policy-assumption dependencies.

All non-trusted statuses set trusted `used_axioms` to absent. They may carry
untrusted diagnostic suggestions only through diagnostics-owned references that
are not accepted dependency facts.

## Diagnostics And Explanations

Every non-accepted projection has a stable explanation surface made from
metadata and diagnostics-owned references:

- `externally_attested` records the external admission status, policy
  fingerprint, selected class, selection reason, and optional explanation ref;
- `policy_assumed` records the selected class, selection reason, policy
  fingerprint, and optional explanation ref;
- `open` records the selected class, selection reason, ordered diagnostic refs,
  and optional explanation ref;
- `rejected` records the selected class, selection reason, ordered diagnostic
  refs, and optional explanation ref;
- `NoSelectableEvidence` records the selected class, selection reason, ordered
  diagnostic refs, optional explanation ref, and generated diagnostic result
  id.

Structured policy-assumption, open-obligation, rejection-layer, and backend
exhaustion details are diagnostics-owned payloads referenced through stable
diagnostic or explanation refs. Status projection does not embed those payloads
directly.

Diagnostic ordering follows architecture 19: source identity, source range,
phase order, severity, diagnostic code, and stable detail key. Parallel
completion order, backend runtime, and cache lookup timing do not participate
in diagnostic ordering.

## Artifact Projection

Projection may populate artifact obligation fields only from stable projection
data:

- `status`;
- `accepted_witness_obligation_id` when a trusted accepted witness is
  publishable;
- `diagnostic_ref` or explanation reference for non-accepted outcomes;
- policy and obligation fingerprints used by artifact consistency checks.

Current `mizar-artifact` `ProofWitnessRef` schema version `2.0` accepts
trusted `ProofWitnessRef` values for `kernel_verified` formula/substitution
evidence only. Therefore:

- `KernelVerified` can project an accepted artifact status as publishable only
  when matching selected witness metadata is available; status projection does
  not itself publish a committed witness ref;
- `DischargedBuiltin` remains an accepted internal projection but cannot publish
  a trusted artifact witness ref yet; its deterministic discharge hash remains
  internal projection and proof-reuse metadata under the current artifact schema
  and must not be written as an accepted artifact obligation field;
- `PolicyAssumed` cannot be losslessly represented in current
  `ObligationStatus`.

These are integration gaps, not permission to emit placeholder witnesses or to
rename statuses.

## Proof Reuse Metadata

Status projection exports validation metadata for proof reuse:

- selected winner class;
- projected obligation status;
- `ObligationAnchor`;
- obligation fingerprint;
- canonical VC fingerprint;
- canonical local-context fingerprint;
- dependency-slice fingerprint;
- policy fingerprint;
- selected evidence hash;
- accepted proof-obligation goal polarity for trusted kernel selections, when
  present;
- selected proof witness payload artifact hash (`witness_artifact_hash`) when
  available for a `KernelVerified` candidate; this is not a committed
  `ProofWitnessPublishedRef`;
- deterministic discharge hash, when present;
- trusted used-axiom reference hash, when present;
- external admission status, when present;
- matching proof-evidence identity, including selected candidate id,
  selected-candidate provenance hash, selection reason, and tie-break key hash;
- dependency artifact fingerprint and compatible dependency/proof-reuse schema
  versions, when supplied by the artifact/cache boundary;
- diagnostic or explanation reference hashes.

This metadata is a cache validation predicate. It is never proof authority.
Reuse additionally requires matching `ObligationAnchor`, canonical VC
fingerprint, canonical local-context fingerprint, dependency-slice fingerprint,
compatible verifier policy, matching proof evidence, and compatible referenced
dependency artifacts and schemas. Cache records cannot promote externally
attested, assumed, open, rejected, or no-selectable outcomes to trusted status,
and cannot synthesize trusted `used_axioms`.

Status projection also exports a stable proof-reuse validation hash over every
field above. `mizar-cache` may compare that hash and the structured fields as a
future reuse predicate, but a match only avoids recomputation after cache
validation. Missing dependency artifact/schema compatibility, policy
incompatibility, witness hash mismatch, deterministic discharge mismatch, or
proof-evidence identity mismatch, including an accepted-goal-polarity mismatch,
is a miss. A match never upgrades
`ExternallyAttested`, `PolicyAssumed`, `Open`, `Rejected`, or
`NoSelectableEvidence` to `Accepted`, and never creates trusted
`used_axioms`.

A complete proof-reuse predicate is class-aware: `KernelVerified` requires the
selected proof witness hash, `DischargedBuiltin` requires the deterministic
discharge hash, and non-trusted classes remain exported metadata only rather
than complete proof-reuse hits.

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `STATUS8-G001` | `external_dependency_gap` | Current artifact obligation status lacks a distinct public `policy_assumed` value. Projection must keep it distinct internally and defer artifact publication rather than collapse it. |
| `STATUS8-G002` | `external_dependency_gap` | Current `ProofWitnessRef` trusted reader rejects `DischargedBuiltin` witness publication. Projection may record the trusted status and deterministic discharge hash, but witness publication stays deferred. |
| `STATUS8-G003` | `deferred` | Diagnostics, artifact emission, manifest commit, cache lookup, and ATP early-stop integration consume this projection in later tasks. This spec defines stable metadata only. |

## Public Enum Policy

Task 14 applies the public-enum forward-compatibility procedure to this
module. All public status-projection enums are downstream-facing API surfaces
and must remain `#[non_exhaustive]`; downstream consumers must keep wildcard
match arms. Artifact-facing status enums additionally require artifact schema
compatibility review before new variants are published or mapped to current
artifact fields.

| Enum | Compatibility decision |
|---|---|
| `TrustedUsedAxiomsError` | forward-compatible |
| `ProjectedProofStatus` | forward-compatible |
| `CurrentArtifactObligationStatus` | forward-compatible with artifact compatibility review |
| `ArtifactPublicationGap` | forward-compatible with artifact compatibility review |
| `ArtifactStatusPublication` | forward-compatible with artifact compatibility review |
| `StatusProjectionError` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Non-Goals

Status projection does not run ATP backends, perform SAT solving, call the
kernel, invent substitutions, select premises, query proof caches, stage or
publish witnesses, write artifact manifests, or accept proofs.

## Source candidate checking

`check_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` owns a fresh single-leaf check, not registration activation or publication. The encoding tuple groups the existing explicit profile/format; it adds no configuration type. Re-run source checker and Core lowering, generate the original graph with existing source/VC schemas, then apply actual default Discharge. Use the [actual source candidate factory](../../mizar-atp/en/backend.md#source-registration-candidate-inputs) for the selected NeedsAtp leaf. Unsupported/foreign inputs, wrong or Discharged targets and unschedulable policy return errors without a checked result or fallback.
Compare the candidate's payload kind and complete target, input/provenance hashes, labels and symbol mappings against the locally produced encoded container. Derive UncheckedFormulaSubstitution's encoded_problem_matches from those comparisons; require existing FormulaSubstitution scheduling permission. Caller match booleans, encoded/candidate pairs, contexts, Accepted results or hashes are not inputs. No backend observation is constructed.
Independently rebuild the same authentic source handoff from the fresh Core/whole default-discharge graph. Derive expected target/profile, parse actual returned v2 bytes with schema 2 explicitly allowed, and construct the ordinary [Kernel context](../../mizar-kernel/en/checker.md) from this separate handoff's complete identity/producer entries. Reject imported/unsupported context classes rather than invent empty context. Invoke normal check_kernel_evidence as ProofObligation with existing default policy/limits; never enable legacy audit or reuse an earlier result.
Wrap the actual result with KernelPolicyInput::from_kernel_result and explicit AtpFormulaSubstitution origin, then call the existing evaluator. Return its actual PolicyDecision and KernelCheckResult, including actual rejection detail for false goals. Accepted/untainted normal proof checks receive KernelVerified under existing policy; rejection is never manually upgraded or replaced by Open/Builtin/external evidence. Preserve all policy criteria and fingerprints.
Return values are this call's checked output, not transferable authorization for selection, source activation, witness publication, artifact status or cache reuse. The caller owns current source/snapshot freshness. Preserve the original Open/Builtin facade’s public behavior. Generic by/built-in integration, search budgets, imported context, witness/artifact/cache publication and full milestone closure remain deferred.

## Source candidate selection

`select_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<ProofSelection, String>` owns bounded same-source orchestration. Call check_source_registration_candidate freshly with these immutable inputs and the supplied policy; never accept a caller check tuple, accepted flag or evidence pair. Preserve the original checking and Builtin facade public behavior.
Independently re-run the existing source checker/Core/original VC/default Discharge producers on the same inputs and obtain the actual source candidate factory's encoded container and candidate. Reuse existing schemas and owner APIs, with no new hash/codec/parser or check implementation. Require the fresh Kernel target algorithm/digest to equal the actual encoded target fingerprint before selection. Caller still owns current source/snapshot freshness; this local association creates no runtime lease or reusable source receipt.
Use the actual candidate_id with CandidateSourceId::new. Only the actual current decision's KernelVerified class may use ProofEvidenceCandidate::from_trusted_kernel_input from this fresh result with AtpFormulaSubstitution origin; failure to derive trusted evidence is an error. Otherwise preserve the actual decision with ProofEvidenceCandidate::new, without a trusted marker or invented evidence hash. In either case carry the actual candidate provenance hash. No backend/profile/format priorities or witness hashes are invented.
Construct ProofEvidenceSet with the fresh target's existing TargetVcFingerprint::sort_bytes identity, the actual encoded input_hash and a clone of the supplied immutable policy; select_winner receives this one bound candidate. Existing selection owns ordering, rejection diagnostics, accepted evidence identity/polarity and unpublished reuse metadata. Binding/producer/check/scheduling errors return Err without a selection; a genuinely rejected check yields the existing non-trusted rejected selection, never sibling acceptance or an Open/Builtin fallback.
Tests compare genuine original/renamed single and three-registration sources and both encodings against independently assembled existing-selector results from actual producers. False definitions must reject their own dependent leaves while preserving independent leaf identity. Fresh-source/policy/target/format failures return no trusted selection; FormulaSubstitution and Builtin permissions stay distinct. Preserve the checking tests and original Builtin behavior.
Return only this call's existing ProofSelection, with no selected witness or committed publication. It is not transferable authority for activation, artifact projection, trusted cache reuse or current source validity. Generic by/built-in acceptance, finder/backend execution, matched witness/obligation metadata, Artifact publication and full Steps 6–8 remain deferred.

## Source discharge checking

`check_source_registration_discharge_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` freshly reconstructs the original source graph and actual default Discharge, then obtains the selected original direct-premise handoff through VC's source association. Encode normal local evidence, require existing BuiltinKernelEvidence scheduling permission, parse with the original target/profile and invoke ordinary proof-obligation checking. Classify the actual result with BuiltinDischarge origin under the supplied policy; no caller evidence/result or receipt is accepted.
Only context construction and the normal Kernel call are shared privately with the existing ATP check and original built-in activation path. Their parser/schema/instance selection, errors, scheduling, origin, policy and activation behavior remain unchanged. Unsupported or foreign inputs and denied formats fail closed; false-definition direct leaves still prove their unchanged premise/goal. General acceptance, selection, trusted witness metadata and publication remain deferred; caller-owned source/snapshot freshness is not a reuse receipt.

## Source existential-registration proof

`prove_source_existential_registration(&SurfaceResolvedArena, &TypedArena, &SymbolEnv,
BuildSnapshotId, &VerifierPolicy) -> Result<RegistrationDatabase, String>` freshly invokes
[checker](../../mizar-checker/en/registration_resolution.md#source-existential-registration-proof),
Core, VC and both source handoffs. The [VC local transport](../../mizar-vc/en/kernel_evidence_handoff.md#local-binary-transport) frames their actual fields; Proof first parses six-section
Formula v1, including full terms, substitutions, provenance, context identity and
AssertFalseForRefutation, without checking acceptance. ATP's [untrusted name writer](../../mizar-atp/en/backend.md#closed-choice-reflexivity-names) supplies Formula v2 names from that parsed input. Proof runs the parser and check_kernel_evidence
itself for both leaves; caller Accepted inputs, kernel-result summaries, cached handoffs and
diagnostic/hash text cannot substitute.

Require existing UncheckedBuiltinDischarge scheduling permission before checking, then use
KernelEvidenceOrigin::BuiltinDischarge. Both fresh Accepted, untainted ProofObligation results must
pass existing KernelPolicyInput/ProofPolicyEvaluator as genuine DischargedBuiltin. Construct
existing ActivationInput and RegistrationDatabase inside this call with the same authenticated
SymbolEnv and checker validation, binding the full pattern, correctness owner, original
parent/nonempty/leaf association, source/module/snapshot, handoff hashes and current policy. Failure
produces no activation or mutation of the pending input; no new kernel receipt or trust policy is
introduced.

The returned database is this call's module-local output, not a transferable proof receipt. Later
consumers must use this producer within the same authenticated source orchestration, never accept an
arbitrary returned/public database as proof authority. Preserve item-order visibility and no
retroactive facts. The bounded C3 widening runner may consume this fresh local output immediately in that orchestration;
the bounded C13 consumer belongs to [harness orchestration](../../mizar-test/en/harness.md#attributed-ambiguity-admission); the bounded C5 narrower-loci consumer may likewise immediately consume the single-registration output with the same immutable inputs; broader Task274, artifact publication and cluster/reduction effects remain deferred.

## Three-registration source-proof extension

The implemented additional source profile uses the unchanged prove_source_existential_registration signature and freshly reconstructs the complete checker/Core/VC graph for all three authored C13 registrations.
All six ordinary Formula-v2 leaf checks must succeed under the current policy before constructing one same-environment database containing exactly the three full patterns. Failure returns no partial activation or mutation of earlier pending inputs.
Each association binds its own registration, parent, independent nonempty seed, both leaves, actual definition dependencies, handoff hashes and source/module/snapshot. No callback, caller receipt or another registration's acceptance substitutes.
Retain the single-registration behavior and all non-transferable-database restrictions. This prerequisite provides no C13 overload/inhabitation-consumer credit, C3 widening, general Task274/import, cluster closure or publication.
