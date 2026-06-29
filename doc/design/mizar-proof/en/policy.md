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
status is `Accepted`.

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
| `KernelVerified` | Accepted kernel result for ATP formula/substitution evidence. | yes | May propagate trusted `used_axioms` from the kernel result. |
| `DischargedBuiltin` | Accepted kernel result for built-in discharge evidence or allowed kernel primitive evidence. | yes | Projected separately from `KernelVerified`; must not be collapsed into ATP kernel verification. |
| `KernelRejected` | Rejected kernel result. | no | Carries structured rejection reason for diagnostics. |
| `KernelCheckable` | Unchecked formula/substitution candidate or built-in discharge evidence that policy may send to the kernel. | no | Schedulable evidence only; not a winner until the kernel accepts it. Built-in discharge evidence without a stable kernel representation is not in this class. |
| `ExternallyAttested` | Policy-admitted external attestation. | no | May be recordable or policy-selectable only when allowed and `require_kernel_certificates` is false. |
| `OpenAllowed` | Open obligation allowed by build mode. | no | Diagnostic/status projection only. |
| `AssumedByPolicy` | Explicit assumption permitted by the active policy. | no | Non-trusted policy status only; cannot satisfy `require_kernel_certificates` or synthesize trusted dependencies. |
| `RejectedByPolicy` | Evidence or open status rejected by active policy. | no | Stable policy rejection diagnostic required. |
| `DiagnosticOnly` | Backend logs, backend diagnostics, counterexamples, cache records, timings, and unsupported proof payloads. | no | Never a proof winner and never trusted material. |

Forward-compatible implementations may add classes only under a new schema
version and must preserve the rule that non-kernel material cannot become
trusted acceptance.

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

- a kernel-verified winner may stop remaining backend work when no higher
  class exists under the active policy;
- external evidence cannot justify stopping when `require_kernel_certificates`
  is true;
- diagnostic-only candidates never justify semantic early stop;
- open results may stop only when policy proves no schedulable or running
  evidence can produce a better allowed class.

Runtime duration and first-completion order are never early-stop authority.

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
- Trusted `used_axioms` are copied only from accepted kernel results.
- Built-in discharge becomes trusted `discharged_builtin` only after accepted
  kernel checking or an explicitly allowed kernel primitive result. Otherwise
  it remains deterministic policy evidence.
- Current artifact witness references support formula/substitution
  `kernel_verified` publication but not `discharged_builtin` publication.
  Until that artifact schema gap is closed, policy and selection may keep
  `discharged_builtin` distinct, but witness publication for that class is an
  `external_dependency_gap` and must not be collapsed into `kernel_verified`.
