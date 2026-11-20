# mizar-proof Policy Spec

Source open/assumed registration may be observed through the existing development
policy's `OpenObligation` and `PolicyAssumption` candidates. These distinct policy
classes are not kernel verification; strict release rejection remains unchanged.

> Canonical language: English. Japanese companion:
> [../ja/policy.md](../ja/policy.md).

## Purpose

The policy module defines verifier-policy decisions that sit above evidence
production and below artifact publication. It classifies proof evidence,
decides whether kernel checking may be scheduled, admits or rejects
externally attested evidence as policy evidence, computes the stable
`PolicyFingerprint`, and answers policy-driven early-stop queries.

The module is not a proof acceptor. Trusted acceptance and trusted
`used_axioms` come only from `mizar-kernel` `KernelCheckResult` values whose
status is `Accepted` and whose kernel evidence check kind is
`ProofObligation`.

## Inputs

The policy evaluator consumes normalized, immutable records from other crates:

- accepted or rejected `mizar-kernel` results paired with a normalized
  evidence-origin wrapper owned by this crate;
- untrusted `mizar-atp` portfolio candidates and backend diagnostics;
- deterministic `mizar-vc` built-in discharge records and discharge hashes;
- externally attested evidence records;
- open-obligation explanations;
- explicit policy-assumption records and source refs;
- active verifier policy settings.

It may observe cache metadata only as a validation input supplied by the cache
owner. Cache hits are not proof authority and do not change a candidate class.

`KernelCheckResult` does not by itself identify whether the checked evidence
came from ATP formula/substitution evidence, built-in discharge evidence, or a
kernel primitive. The policy module must receive that kernel-checkable origin
as explicit normalized input and must not infer it from arrival order, backend
identity, checked import contents, diagnostics, or used-axiom lists.
Externally attested evidence remains a separate policy input, not a
kernel-result origin.

## Verifier Policy Settings

The active policy is a stable record with these policy-relevant fields:

| Field | Meaning |
|---|---|
| `schema_version` | Policy schema version included in fingerprints and diagnostics. |
| `profile_id` | Stable policy profile id, such as `release`, `development`, or `interactive`. |
| `build_mode` | Release, development, or interactive/open-editor mode. |
| `require_kernel_certificates` | Requires kernel-accepted formula/substitution evidence or accepted built-in kernel evidence for accepted publication. |
| `external_evidence` | Whether externally attested evidence is rejected, recorded as development evidence, or policy-permitted as a non-trusted winner. |
| `open_obligation` | Whether open obligations are rejected, recorded for diagnostics only, or allowed as policy-open output in the current build mode. |
| `policy_assumption` | Whether explicit policy assumptions are rejected or recordable as non-trusted policy status. |
| `kernel_evidence_formats` | Kernel-checkable evidence formats that may be scheduled, currently formula/substitution evidence and built-in kernel evidence representations. |
| `checker_schema_version` | Kernel/proof checker schema version required by the policy. |
| `instance_search_limits` | Optional explicit finder work-count bounds; [configuration-only carrier](#instance-search-limits). |

### Instance Search Limits

`VerifierPolicy::instance_search_limits(&self) -> Option<[u64; 4]>` and `with_instance_search_limits(self, Option<[u64; 4]>) -> Self` expose one private field using existing types. Both are const; the getter copies the value and the builder changes only this field. Release, development, interactive and Default retain None; None clears the setting and denotes unconfigured finder limits, never unlimited search. Preserve literal zero and u64::MAX without conversion or numerical defaults.
Array coordinates are maximum distinct ground terms in the finite input/generated pool, premise-instantiation candidates, named equality-instance candidates, and untrusted SAT search checks, in that order. These are deterministic work-count bounds for the future finder, distinct from timeout, premise pruning, backend-run limits and Kernel parsing/encoding limits. Counting/enforcement, term generation, exhaustion returning no evidence and actual finder invocation remain separate ATP work; storing a setting performs none of them.
For None, policy_fingerprint emits exactly its old fields/bytes. For Some, after the existing checker_schema_version field and before finalization, append field_u16 `instance_search_limits_version` = 1, then field_u64 `max_ground_terms`, `max_premise_instances`, `max_equality_instances`, `max_sat_checks` from the four coordinates. Reuse the existing hash/domain/framing, with no schema-version or old-field change. Some zero is distinct from None; clearing restores the old fingerprint. Existing classification, scheduling, admission, selection and acceptance remain unchanged; configured identity is validation data, not proof authority.
Two appended unit families fix actual pre-change legacy fingerprints/defaults/clear behavior and explicit zero/max/each-coordinate distinction/determinism. One appended genuine-source family reuses original/false aggregate registrations, both formats and existing policies; compare entire fresh Kernel result/PolicyDecision tuples with their unconfigured baselines and preserve immutable actual inputs. These prebuilt source candidates are not finder runs; no test-only Accepted constructor or duplicate existing policy/producer matrix is added.
This carrier does not complete ATP30 or supply term-wire correspondence, search/exhaustion evidence, runtime currentness, witness/publication/reuse or any Step6–8 gate.

### Build Modes

Release mode is strict by default:

- `require_kernel_certificates = true`;
- externally attested evidence is not a winner;
- open obligations are rejected for publication.

Development mode may record externally attested evidence and open obligations
as policy/development evidence, but those statuses remain non-trusted.

Interactive mode may keep obligations open for diagnostics and LSP feedback.
It still must not project open or external evidence as trusted proof status.

## Candidate Policy Classes

`CandidatePolicyClass` is the policy-facing class used before winner
selection. Classes are ordered semantically by selection tasks, but policy
classification itself must be deterministic and independent of arrival order.

| Class | Source | Trusted? | Notes |
|---|---|---:|---|
| `KernelVerified` | Accepted proof-obligation kernel result for ATP formula/substitution evidence. | yes | May propagate trusted `used_axioms` from the kernel result. Accepted consistency checks are diagnostic-only and never enter this class. |
| `DischargedBuiltin` | Accepted proof-obligation kernel result for built-in discharge evidence, including kernel-owned primitive evidence accepted by the kernel. | yes | Projected separately from `KernelVerified`; must not be collapsed into ATP kernel verification. Accepted consistency checks are diagnostic-only and never enter this class. |
| `KernelRejected` | Rejected kernel result. | no | Carries structured rejection reason for diagnostics. |
| `KernelCheckable` | Unchecked formula/substitution candidate or built-in discharge evidence that policy may send to the kernel. | no | Schedulable evidence only; not a winner until the kernel accepts it. Built-in discharge evidence without a stable kernel representation is not in this class. |
| `ExternallyAttested` | Policy-admitted external attestation. | no | May be recordable according to `ExternalEvidenceAdmission`; may be policy-selectable only when allowed and `require_kernel_certificates` is false. |
| `OpenAllowed` | Open obligation allowed by build mode. | no | Diagnostic/status projection only. |
| `AssumedByPolicy` | Explicit assumption permitted by the active policy. | no | Non-trusted policy status only; cannot satisfy `require_kernel_certificates` or synthesize trusted dependencies. |
| `RejectedByPolicy` | Evidence or open status rejected by active policy. | no | Stable policy rejection diagnostic required. |
| `DiagnosticOnly` | Backend logs, backend diagnostics, counterexamples, cache records, timings, and unsupported proof payloads. | no | Never a proof winner and never trusted material. |

Forward-compatible implementations may add classes only under a new schema
version and must preserve the rule that non-kernel material cannot become
trusted acceptance.

Corrected kernel proof-obligation rejections remain terminal policy inputs.
`invalid_sat_refutation`, kernel-evidence goal-polarity `context_mismatch`,
`missing_provenance`, and real legacy unsupported-certificate gate records
stay `KernelRejected` diagnostics and must not be retried into acceptance,
reclassified as policy-open, or upgraded by external admission. Selection may
prefer one of these rejections over an otherwise-open fallback so status
projection records the failed proof result, but accepted trusted kernel
evidence still outranks rejected diagnostics. The proof crate must not
fabricate legacy certificate payloads to test that gate; real-payload coverage
waits on a public kernel legacy-certificate fixture or runner.

For accepted proof-obligation kernel inputs, `KernelPolicyInput` also exposes
the kernel-validated accepted goal polarity used by proof-reuse metadata. The
current public value is `AcceptedGoalPolarity::AssertFalseForRefutation`.
Accepted consistency checks and all non-accepted or policy-tainted inputs
expose no accepted goal polarity.

For rejected kernel-evidence inputs, `PolicyDecision` preserves the kernel
evidence check kind when the kernel result provides it. Selection uses that
explicit kind to distinguish proof-obligation goal-polarity failures from
consistency-check diagnostics; it must not infer the distinction from rejection
detail text alone.

## Public Enum Policy

Task 14 applies the public-enum forward-compatibility procedure to this
module. All public policy enums are downstream-facing API surfaces and must
remain `#[non_exhaustive]`; downstream consumers must keep wildcard match arms
and must treat new variants as requiring policy/spec review rather than
silently promoting evidence.

| Enum | Compatibility decision |
|---|---|
| `BuildMode` | forward-compatible |
| `ExternalEvidenceMode` | forward-compatible |
| `OpenObligationMode` | forward-compatible |
| `PolicyAssumptionMode` | forward-compatible |
| `KernelEvidenceFormat` | forward-compatible |
| `CandidatePolicyClass` | forward-compatible |
| `PortfolioEarlyStopClass` | forward-compatible |
| `PortfolioEarlyStopReason` | forward-compatible |
| `KernelEvidenceOrigin` | forward-compatible |
| `AcceptedGoalPolarity` | forward-compatible |
| `PolicyCandidate` | forward-compatible |
| `BackendProofPayloadKind` | forward-compatible |
| `ExternalEvidencePublicationStatus` | forward-compatible |
| `PolicyDiagnosticCategory` | forward-compatible |
| `PolicyReasonCode` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Kernel Scheduling Policy

`can_schedule_kernel_check` returns true only for evidence that is intended for
the kernel evidence boundary:

- ATP formula/substitution evidence payloads or refs whose target binding,
  encoded problem hash, formula labels, symbol bindings, and provenance hashes
  match the encoded problem;
- deterministic built-in evidence that already has a stable kernel evidence
  representation;
- explicitly allowed kernel primitive evidence.

It returns false for:

- externally attested evidence;
- backend proof methods, backend logs, SMT proof objects, TSTP traces,
  resolution traces, unsat cores, backend-reported used axioms, and backend
  diagnostics;
- cache records and cache hit metadata;
- open obligations;
- legacy certificate/replay material unless a separate migration/audit policy
  explicitly routes it to the kernel's audit-only rejection surface. Audit
  replay still cannot create trusted acceptance in this crate.

Scheduling permission is not acceptance. A scheduled candidate becomes trusted
only if the kernel returns `Accepted`.

## Bounded native positive-result policy

The crate-private `consume_object_inhabitation` consumes an actual Kernel `ObjectInhabitationCheck`, whose opaque type fixes positive existential introduction and rule origin. Require enabled `BuiltinKernelEvidence` and exact agreement between its Kernel-owned checker schema and the supplied policy's requested schema; otherwise refuse. Return that policy's fingerprint only after these checks. This post-check primitive eligibility path does not create a generic `KernelPolicyInput`, selectable class, accepted-evidence hash or refutation polarity. Existing generic classification and trusted selection remain unchanged.

## Externally Attested Evidence

Externally attested evidence is policy evidence. It is never kernel-verified
evidence and never trusted used-axiom material.

Admission matrix:

| Policy state | Record as development evidence | May win selection | Publication status |
|---|---:|---:|---|
| `require_kernel_certificates = true` | only if `external_evidence` permits recording | no | rejected-by-policy or open, never accepted |
| release mode without external permission | no | no | rejected-by-policy |
| development mode with record permission | yes | no unless explicitly policy-permitted and kernel certificates are not required | externally attested / development evidence |
| interactive mode with record permission | yes | no unless explicitly policy-permitted and kernel certificates are not required | externally attested / open diagnostic |

`ExternalEvidenceAdmission` is the task-owned API surface for this matrix. It
contains:

- `record_as_development_evidence`;
- `may_win_selection`;
- `publication_status`;
- `diagnostic`;
- `trusted_used_axioms_allowed`, always `false`.

`publication_status` is one of:

- `RejectedByPolicy`;
- `ExternallyAttestedDevelopment`;
- `ExternallyAttestedOpenDiagnostic`;
- `ExternallyAttestedPolicyPermitted`.

The exhaustive mapping is:

| `external_evidence` | `require_kernel_certificates` | `build_mode` | Record? | May win? | Publication status | `PolicyDecision.class` | Diagnostic |
|---|---:|---|---:|---:|---|---|---|
| `Reject` | any | any | no | no | `RejectedByPolicy` | `RejectedByPolicy` | `policy_rejection/external_evidence_rejected` |
| `RecordDevelopment` | true | `Interactive` | yes | no | `ExternallyAttestedOpenDiagnostic` | `ExternallyAttested` | `policy_open/external_evidence_recorded` |
| `RecordDevelopment` | true | `Release` or `Development` | yes | no | `RejectedByPolicy` | `RejectedByPolicy` | `policy_rejection/external_evidence_requires_kernel_certificate` |
| `RecordDevelopment` | false | `Interactive` | yes | no | `ExternallyAttestedOpenDiagnostic` | `ExternallyAttested` | `policy_open/external_evidence_recorded` |
| `RecordDevelopment` | false | `Release` or `Development` | yes | no | `ExternallyAttestedDevelopment` | `ExternallyAttested` | `policy_open/external_evidence_recorded` |
| `PermitNonTrustedWinner` | true | `Interactive` | yes | no | `ExternallyAttestedOpenDiagnostic` | `ExternallyAttested` | `policy_open/external_evidence_recorded` |
| `PermitNonTrustedWinner` | true | `Release` or `Development` | yes | no | `RejectedByPolicy` | `RejectedByPolicy` | `policy_rejection/external_evidence_requires_kernel_certificate` |
| `PermitNonTrustedWinner` | false | any | yes | yes | `ExternallyAttestedPolicyPermitted` | `ExternallyAttested` | `policy_open/external_evidence_policy_permitted` |

Policy-tainted accepted proof-obligation kernel inputs use the same external
admission mapping; the original kernel result still remains the only source of
kernel status, but the policy-tainted result is not projected as trusted
`KernelVerified` or `DischargedBuiltin`. Accepted consistency checks remain
diagnostic-only before external admission mapping.

For external decisions, `PolicyDecision.diagnostic` mirrors
`ExternalEvidenceAdmission.diagnostic`; both are either absent or structurally
equal. Kernel rejection records are not stored in external admission.

Rejection diagnostics must distinguish policy rejection from kernel rejection.
External records must not synthesize `used_axioms`; backend-reported axiom
lists remain diagnostic-only unless the kernel independently accepts evidence
that produces trusted `used_axioms`.

## Open Obligations

Open obligations are classified by build mode and active policy:

- release publication rejects open obligations;
- development mode may record open obligations as policy-open results;
- interactive mode may keep open obligations current for diagnostics and LSP
  feedback.

Open obligations are never trusted proof status. They may carry stable
explanation refs used by status projection and diagnostics.

## Policy Assumptions

`AssumedByPolicy` is a policy status, not trusted acceptance. It requires an
explicit assumption source and an active policy field that allows recording
assumptions. Release publication rejects policy assumptions unless a future
non-trusted publication profile explicitly permits them.

Policy assumptions cannot satisfy `require_kernel_certificates`, cannot
publish trusted witness material, cannot create trusted `used_axioms`, and
cannot create accepted dependency facts for proof reuse. Status projection
must keep them distinct from externally attested, open, and kernel-verified
evidence.

## Policy Fingerprint

`PolicyFingerprint` is a canonical hash over only policy-relevant settings:

- policy schema version;
- profile id;
- build mode;
- `require_kernel_certificates`;
- externally attested admission mode;
- open-obligation mode;
- policy-assumption mode;
- schedulable kernel evidence representation set;
- checker schema version;
- future forward-compatible policy fields that affect classification,
  scheduling, selection, status projection, witness publication, or proof reuse.

The fingerprint must not include:

- candidate arrival order;
- backend completion time, runtime duration, process id, worker id, temporary
  path, local wall-clock time, or scheduling priority;
- backend stdout/stderr bytes or diagnostic wording;
- cache hit/miss timing or cache record presence;
- artifact output path outside stable published reference fields.

Fingerprint serialization uses stable field names, sorted collections, explicit
schema versions, and deterministic byte encoding. A cache may use the
fingerprint only as a validation predicate. It is not proof authority.

## Early-Stop Policy Queries

The policy module may answer whether no better acceptable class is still
possible for an ATP portfolio. The query is policy-driven:

- a kernel-verified winner may stop remaining backend work only when no
  pending selectable class, including another `KernelVerified` candidate, can
  displace it under the active policy;
- external evidence cannot justify stopping when `require_kernel_certificates`
  is true;
- diagnostic-only candidates never justify semantic early stop;
- open results may stop only when policy proves no schedulable or running
  evidence can produce a better allowed class.

Runtime duration and first-completion order are never early-stop authority.

Task 12 exposes this as a class-level finality query for the ATP portfolio:

```text
PortfolioEarlyStopInput
  observed_best_class?
  pending_best_possible_classes[]

PortfolioEarlyStopDecision
  may_stop
  reason
  observed_best_class?
  blocking_pending_class?
```

`observed_best_class` is the best policy-normalized class already observed
after any required kernel result is available. `pending_best_possible_classes`
is the portfolio owner's conservative summary of classes that pending or
running work can still produce. These classes are semantic possibilities, not
backend completion events.

The decision is `may_stop = true` only when all of the following hold:

- an observed class exists and is selectable under the active policy;
- every pending class is either not selectable under the active policy or is
  strictly lower than the observed class in the deterministic winner order;
- no pending class is equal to the observed class, because an equal-class
  candidate can still displace the observed candidate through deterministic
  tie-break keys unless a later API supplies an explicit dominance proof.

`reason` is a stable enum, not a free-form string:

| Reason | Meaning |
|---|---|
| `NoObservedCandidate` | No observed class is available yet. |
| `ObservedClassNotSelectable` | The observed class cannot be a selectable winner under the active policy. |
| `BlockedByHigherClass` | A pending class can produce a strictly better selectable winner. |
| `BlockedByEqualClass` | A pending class equals the observed class and may still win by deterministic tie-break. |
| `NoDisplacingPendingClass` | No pending selectable class can displace the observed class. |

The query also provides a policy normalization helper that maps a
`PolicyCandidate` to its best possible early-stop class:

- accepted proof-obligation ATP formula/substitution kernel results and
  still-kernel-checkable formula/substitution evidence map to
  `KernelVerified`;
- accepted proof-obligation built-in/kernel-primitive evidence and
  still-kernel-checkable built-in evidence map to `DischargedBuiltin`;
- accepted consistency-check kernel results are non-selectable
  `DiagnosticOnly` evidence and have no early-stop class;
- externally attested evidence maps to `PolicyPermittedExternal` only when the
  active policy permits it to win and kernel certificates are not required;
- policy assumptions and open obligations map to their non-trusted classes only
  when the active policy allows those classes to win;
- rejected and diagnostic-only material cannot justify stopping.

The blocking pending class, when present, is selected by class rank, not by
input order. The query does not run ATP backends, cancel processes, schedule
kernel checks, inspect backend timing, choose the final winner within a class,
or publish proof status. It is an advisory finality predicate consumed by the
portfolio/scheduler owner.

Task 12 defines the owner contract in `mizar-proof`. Downstream `mizar-atp`
consumption of this contract, process cancellation wiring, and runtime
portfolio integration remain deferred/external-dependency work until the ATP
side adopts the API; `mizar-proof` does not add an ATP adapter or scheduler
stub.

## Diagnostics

Policy diagnostics are stable records with:

- `policy_rejection` or `policy_open` category;
- policy profile id and fingerprint;
- candidate class;
- stable reason code;
- optional explanation or evidence ref.

Policy diagnostics must not reuse kernel rejection codes for policy failures.
When both kernel and policy failures exist, status projection keeps both layers
distinguishable.

## Boundary Rules

- The policy module never calls ATP backends, runs SAT solving, searches for
  proofs, selects premises, invents substitutions, performs overload
  resolution, searches clusters, inserts coercions, or consults mutable global
  compiler state.
- Backend success, backend diagnostics, externally attested evidence, cache
  metadata, and open obligations cannot produce trusted proof status.
- Trusted `used_axioms` are copied only from accepted proof-obligation kernel
  results.
- Built-in discharge becomes trusted `discharged_builtin` only after an
  accepted proof-obligation kernel result, including kernel-owned primitive
  evidence accepted by the kernel. Otherwise it remains deterministic policy
  evidence.
- Current artifact witness references support formula/substitution
  `kernel_verified` publication but not `discharged_builtin` publication.
  Until that artifact schema gap is closed, policy and selection may keep
  `discharged_builtin` distinct, but witness publication for that class is an
  `external_dependency_gap` and must not be collapsed into `kernel_verified`.

## Computation Publication Boundary

§§20.9.2/20.13.7 govern validated E5 input/result, finite certified conversion and whole-candidate inspection.
Consume real MVM/artifact/contract/proof records; term/truth, UID substitution, guards and Binder construction precede complete residual rejection.
Import failure preserves internal return, never publishes an ensures fragment; missing producers or incomplete stages produce no unchecked facts.
Tests cover term/truth/update/opaque conversion and terminal UID substitution; reject residual whole ensures/equalities, quantified types, annotations and containers, circular requires and pre-update mistakes, while distinguishing each actual incomplete stage.
Explicit reference replay overrides default-false manifest computation_replay for all its calls. Default computed, all-call equation/SAT replay, exact cache-mode identity and strict transitive computed dependencies remain distinct.
Tests exercise explicit replay true/false over opposite manifest defaults through direct/transitive calls, plus all-call replay rejection, strict computed taint and weaker-mode cache corruption.

Boolean conversion tests cover B=true/B≠false, B=false/B≠true and B=C/B≠C polarities with certified two-valuedness, guarded val_at and terminal ordinary b=true without repeated normalization. Truth equivalences cannot fill boolean term-argument or Binder-body slots; uncertified terms stay rejected.

## Formation Evidence Boundary

Validate actual pinned library and per-use formation evidence with its precise conditional premises and request identity; no accepted status comes from spelling, Open goals or algorithm contract evidence. Reject stale version/content/UID/context/C and wrong-premise substitutions. Accepted instances re-enter Checker, keeping finite KΓ/Adm equivalence and Binder all-input construction obligations distinct from E1/E2/E4 and execution.

## Imported Promotion Authority Boundary

Portable promoted premises require genuine validated applicable E1/E2/E4 and complete-group evidence, bound to algorithm/contracts/equations and dependency/version/policy provenance. Module-local status and runtime/hash metadata cannot supply this authority.

### Accepted evidence and validators

| Existing producer/validator | Exact authority and limit |
|---|---|
| [KernelPolicyInput::from_kernel_result and accepted_evidence_hash_from_kernel_result](../../../../crates/mizar-proof/src/policy.rs), [TrustedKernelEvidence::from_policy_input](../../../../crates/mizar-proof/src/selection.rs) | Actual Accepted, untainted ProofObligation kernel result, target and checked evidence; external/open/audit/cache metadata cannot acquire this authority. It is local per-obligation acceptance. |
| [ProofObligationIdentity and project_status](../../../../crates/mizar-proof/src/status.rs) | Obligation/VC/context/dependency identity and active selection-policy agreement; ordinary kernel-verified publication requires a witness. Algorithm return-reflexivity/state-transitivity, theorem and registration checks are genuine bounded producers, not aggregate E1/E2/E4/group acceptance. |
| [TrustedKernelWitnessMetadata and CommittedWitnessPublicationProof::from_source_theorem_manifest_commit](../../../../crates/mizar-proof/src/witness_store.rs) | Private accepted-result binding and committed source-theorem publication validation. Witness-file readers recover untrusted wire evidence; neither they nor the theorem publication proof validate imported promotion. |
| [FormulaImportedFactEvidence, validate_formula_import_projection and validate_formula_import_status](../../../../crates/mizar-kernel/src/checker.rs) | Caller-supplied immutable imported context is checked against package/module/export/statement identity, exact statement-to-formula projection and permitted status. Legacy identity/status checks have the same origin limitation: none authenticates the exporting producer or a promotion group. |

Required logical content follows [§20.13.1–6](../../../spec/en/20.algorithm_and_verification.md#2013-logical-encoding): E1 justifies result typing, single-valued execution and any asserted all-typed-input value existence; E2 verifies body partial correctness; E4 independently verifies normal return under the exact guards. Declaration, stopping alone, a continuation's assumed return or E5 for one ground call cannot substitute for E4. Retain full inputs, type/requires guards and applicable syntax environment/InputOK/context/C bindings.
Ordinary promotion exposes guarded totality and contract; only definitional algorithms expose the §20.13.6 guarded return equations, with textual-order path guards and repeated const substitution, and no new coverage/consistency obligations. Mutual promotion withholds every member until all applicable prerequisites and complete membership are accepted; equations additionally require every member to be definitional.
There is no aggregate promotion producer for these obligations or portable acceptance validator in the current sources. VC57's module-local acceptance dependency is absent and its contract forbids transferable receipts. [Artifact bindings](../../mizar-artifact/en/manifest.md#imported-promotion-authority-boundary) identify available transport foundations, not a replacement proof path.

### Trust-transfer rule

The trusted source is a committed publication of the actual aggregate acceptance result from an **importer-authorized Proof execution authority**. The importer establishes that authority, its approved Proof/kernel implementation and version, and permitted policy/profile/checker-schema scope through importer-controlled trust configuration independent of the imported artifact. It trusts the authority to execute those checks faithfully and to publish only their genuine results. An artifact, caller, package registry or dependency cannot authorize itself or enlarge that trust scope; absent an independently established authority, import grants no promoted premise.

The authority may publish promotion acceptance only after genuine accepted, untainted kernel/Proof evidence establishes every applicable prerequisite described above for every member of the complete recursive group. Publication must originate from that accepted aggregate result through authority-controlled execution and commit, without a caller being able to manufacture or replace the accepted result. This rule transfers an existing acceptance decision; it never upgrades Open, assumed, externally attested, failed or unchecked evidence to kernel acceptance.

Artifact transport must preserve an authenticated binding from that authorized execution and committed publication to the exact package/module/export and algorithm identity, contracts, eligible equations, complete group membership and per-member prerequisites. The binding includes the full guarded input/context identities above, source and pinned library content, dependency versions/lockfile, and the actual proof-policy/profile/checker-schema provenance. Integrity hashes identify bound content but do not establish who accepted it. A new receipt, schema, public summary or caller-supplied accepted status supplies no authority by itself.

Proof import validation must establish that authenticated origin and complete binding, compare them with the importer-resolved objects/dependencies and current trust configuration, and require the exporting policy/profile/checker schema to satisfy the importer's active acceptance requirements. A matching policy label alone is insufficient; no weaker or unknown policy acquires stronger status through import. Missing, unauthenticated, forged, stale, partial, mismatched or unsupported authority/binding/policy data exposes no member premise. Validating this provenance and projection does not re-prove the exporting module or require its opaque body to be loaded.

On successful validation, only the ordinary/definitional premises specified above may be exposed; the imported provenance remains attached to that acceptance and does not become a fresh local kernel result. Complete-group acceptance precedes exposing even one exported member. Genuine non-computable or contract-only promotion uses the same logical trust path without runtime artifacts; computation retains its separate artifact/correspondence checks.

This is the approved trust-transfer specification decision. The [Artifact producer transport design](../../mizar-artifact/en/manifest.md#producer-transport-design) selects authentication and representation; the [import-validation design](#proof-import-validation-design) below owns its acceptance handoff. Executable transport and Proof import validators remain later slices. This rule changes no language rule or kernel inference.

### Proof import-validation design

This future Proof validator consumes Artifact's structurally checked but untrusted immutable publication closure, exact signed bytes/signature and selected module/summary objects. Independently supplied importer inputs are the resolved logical objects/dependency snapshot, current trust configuration and active acceptance requirements. Validate one complete group privately through the stages below; no intermediate success exposes a member premise.

1. **Authenticate and authorize.** Require the canonical, supported descriptor/version/suite from the Artifact design and verify its framed Ed25519 signature over the exact retained bytes. Resolve the key through current importer-controlled trust, then check the authority's approved Proof/kernel implementation/version and permitted policy/profile/checker-schema scope against the authenticated descriptor. Bundled keys, key identifiers, registry origin and caller status are hints or data, never trust roots. The authorized signature attests the committed aggregate acceptance through the restricted producer path; a local manifest's visibility or an unsigned descriptor cannot establish that commitment. Do not require an authenticated second exporter commit, which the transport does not promise.
2. **Match the resolved binding.** Reconstruct the complete authenticated tuple required by the trust-transfer rule from that descriptor and its mandatory object closure. Compare it with the exact importer-selected module/summary bytes and resolved package/module/export/algorithm, logical statements, guarded inputs/context, source/pinned-library identities, dependency versions/lockfile and complete group. Compare content commitments as well as names/versions; semantic-version compatibility or an interface hash cannot authorize different bytes. Private source/proof content is compared by authenticated identity, without loading opaque bodies. Reject absent, substituted, mixed, duplicate, partial or mismatched bindings rather than resolving another object to fit the publication.
3. **Validate acceptance and projection.** Require the authenticated committed acceptance to cover every group member and every applicable prerequisite under the supported checker schema and bound promotion class/input/context. A schema unable to express that complete coverage is unsupported. Missing, Open, failed, tainted, assumed or externally attested records cannot satisfy it; neither can a bare accepted flag detached from the authority-controlled aggregate publication. Validate authenticated acceptance records without replaying exporter proofs. Require the actual exporting policy/profile/checker-schema semantics to satisfy every active importer acceptance requirement using only explicit, supported importer-owned compatibility checks within the authorized scope. Labels, fingerprints or increasing version numbers alone do not establish compatibility; unknown or weaker combinations fail closed. Check each public statement's exact projection to the imported formula and its ordinary/definitional eligibility under the rules above; extra equations or weakened guards are refused. Complete-group validation includes unselected members even when the importer requests one export.
4. **Expose with imported provenance.** Only after all stages succeed, make the eligible selected projections available together for that validated group. Retain their association with the authenticated descriptor/signature, exporter authority/implementation, complete binding and the importer resolution/trust/acceptance snapshots used for validation. This is imported acceptance; never construct a fresh local kernel result, local accepted witness or stronger status from it. Existing kernel identity/projection/status checks remain necessary consumer checks, but cannot replace these gates.

**Currentness and reuse.** Checking and exposure use the same immutable publication and importer snapshots. If resolution, trust or active requirements change before exposure, discard the pending result and validate against the new snapshots. Any reuse must establish that all those inputs still match the current importer state; changed authorization, implementation scope, policy/schema requirements, objects or dependencies invalidate the prior decision. Neither cached success nor a still-valid signature bypasses this check. Currentness here adds no wall-clock expiry or independent trust-bootstrap protocol.

**Refusals and limits.** Artifact retains structural/content/transaction refusal ownership; Proof owns signature, authority, resolved-binding, acceptance/policy and projection refusals. Any failure withholds every promoted premise in the affected group, without a status/hash/cache fallback or partial publication. Legacy opaque signatures may remain available through their existing route without promotion authority. Diagnostic naming and API shape stay with implementation; this design adds no public diagnostic code. Genuine non-computable/contract-only acceptance follows these same logical gates without runtime loading; computation retains separate artifact/correspondence checks.

### VC59 connection design

The future VC59 route consumes only the completed group validation from [Proof import validation](#proof-import-validation-design), for the exact importer-resolved export and snapshots used by that validation. Artifact loading, a public summary, local VC57 acceptance or a caller-supplied status cannot enter this route as validated imported acceptance. Proof owns authentication/acceptance; VC owns selection and incorporation of the validated projections into premises. This handoff requires no new public receipt or serialized schema.

- **Select eligible premises.** Resolve the imported definition label through existing import/name visibility and apply the same definition-expansion policy as VC57: a later VC receives the selected algorithm's promotion axioms when it cites that label or policy makes them available. An algorithm name or call is not a citation ([§16.5.1](../../../spec/en/16.theorems_and_proofs.md#1651-simple-citation)). Preserve item order and never add facts retroactively. Ordinary promotion supplies only validated guarded totality and contract; definitional promotion may additionally supply only the eligible equations described above. Use the exact validated formulas and full guarded input/context bindings; neither an extra equation nor a stronger unguarded statement may be synthesized by the consumer.
- **Keep provenance through consumption.** Associate each premise with its exact resolved export/algorithm and statement projection and the completed validation's authenticated imported provenance, including its complete-group and importer-snapshot association. Preserve that association through VC construction and kernel imported-context/projection/status checks; those checks remain necessary and cannot replace Proof validation. Do not flatten the result into an unqualified formula/status, convert it to a local accepted witness/kernel result, or treat it as authority for a different export, group or context. Reuse follows the validator's currentness rules.
- **Publish and use atomically.** Validate the whole group even for a single selected export; selected premises become available together only after complete validation and successful consumer mapping. Use the same immutable publication and current importer snapshots through premise construction and consumption. A changed resolution, authorization or acceptance requirement invalidates pending or reused premises: discard them and rebuild only after validation against the current inputs. A missing/mismatched projection or provenance association withholds every promoted premise of the affected group; no partially assembled set may reach a VC. Import visibility, cached premises and earlier successful checks cannot bypass these conditions.

The [six portable validation rows](#portable-validation-design-obligations) also govern this connection's future consumer checks: successful ordinary/definitional and logical-only imports preserve exact eligible formulas and imported provenance through consumption; every refusal contributes no group premise. Actual connection must wait for genuine local aggregate producers, Artifact transport and Proof import-validation implementations, genuine VC57 local prerequisite/group acceptance and all six executable success/refusal rows; documenting this route satisfies none of those gates.

### Portable validation design obligations

These six rows specify future producer/consumer checks; none is executable portable-promotion coverage. A positive outcome requires exporter acceptance authenticated under the trust-transfer rule above.

| Input shape | Required outcome; absent route |
|---|---|
| Authorized, authentically published ordinary and definitional accepted exports with exact algorithm/contracts, applicable E1/E2/E4 and complete-group evidence | Only validated totality/contract, plus eligible defining equations, reach VC59; aggregate producer, transport and import acceptance validator absent. |
| Missing, Open, failed, forged or invalid applicable E1/E2/E4 or any member evidence | No member premise; authenticated prerequisite/group validation absent. |
| Stale algorithm/contract/equations, source/library binding, dependency/version or group membership | No premise; exact portable binding/currentness validation absent. |
| Evidence accepted under an incompatible proof policy/profile/checker schema | No premise; existing kernel status checks cannot authenticate portable policy provenance. |
| Genuine accepted non-computable or contract-only export | Logical import succeeds without loading runtime artifacts; authenticated logical-only path absent. Computation separately rejects missing/invalid execution artifacts. |
| Untrusted/self-authorized origin, hash/status/public-summary-only or runtime-only input | No promoted premise; integrity, visibility and normal-return metadata never establish promotion acceptance. |

Existing [status tests](../../../../crates/mizar-proof/src/status/tests.rs) cover local classes, witness publication, policy mismatch and reuse invalidation; [kernel tests](../../../../crates/mizar-kernel/src/checker/tests.rs) cover imported context/projection/policy checks. They do not implement the positive portable rows.
Ordered slices: genuine VC57 local prerequisite/group acceptance remains a prerequisite; the Artifact producer transport design precedes Proof import validation, then VC59 connection against this rule. Implement genuine producers/validators and all six refusal/success rows before connecting VC59 to authenticated outputs. Opaque imports never re-prove the exporting module ([§12.6.1](../../../spec/en/12.modules_and_namespaces.md#1261-opaque-imports)); runtime correspondence stays with execution owners.
`external_dependency_gap`: genuine local aggregate E1/E2/E4/group outputs, Artifact producer transport implementation, Proof import-validation implementation and VC59 connection remain absent. The trust-transfer definition resolves the former authority-definition gap only; it grants no executable acceptance or coverage credit and does not unblock VC59.

## Computation Admission Boundary

Before any MVM evaluation discharges an obligation, validate the complete authenticated computation closure and each required ensures uniqueness proof or exact syntactic exemption. E1 execution single-valuedness and requested all-call equation/SAT replay are separate gates. Open/failed/stale/forged or missing requirements yield no computation fact; Proof owns the specified E0330 projection when real producer/diagnostic integration exists. Tests distinguish genuine admission, raw VM return, replay-only evidence, incomplete imported closure and wrong-call/contract/exemption provenance; strict/default/replay policy remains unchanged.
