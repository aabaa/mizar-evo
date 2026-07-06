# mizar-proof Policy Spec

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
