# mizar-proof Selection Spec

> Canonical language: English. Japanese companion:
> [../ja/selection.md](../ja/selection.md).

## Purpose

The selection module chooses one proof evidence winner from a normalized set of
policy-classified candidates for a single obligation. It owns deterministic
winner ordering and the stable reuse metadata exported by selection.

The module does not accept proofs. It does not run ATP backends, kernel checks,
SAT solving, proof search, premise selection, substitution invention, status
projection, witness staging, cache lookup, or artifact commit.

## Inputs

Selection consumes immutable records already produced by earlier stages:

- normalized obligation identity and encoded problem hash;
- active `VerifierPolicy` and `PolicyFingerprint`;
- policy decisions from `ProofPolicyEvaluator`;
- accepted kernel results paired with explicit kernel evidence origin and the
  accepted kernel evidence hash;
- deterministic built-in discharge hashes;
- externally attested admission records;
- open-obligation explanations;
- backend profile metadata and evidence-format metadata supplied by the
  portfolio owner.

Inputs must be complete before selection begins. Arrival order, backend
completion order, runtime duration, wall-clock time, worker id, process id, and
temporary path are not inputs to winner identity or tie-breaking.

## Winner Classes

Selection compares candidates by winner class before applying tie-break keys.

| Winner class | Eligible candidate | Trusted? | Notes |
|---|---|---:|---|
| `KernelVerified` | Accepted kernel result for formula/substitution evidence satisfying the active policy. | yes | Highest class. Trusted `used_axioms` may be copied only from the accepted kernel result. |
| `DischargedBuiltin` | Accepted `KernelCheckResult` for deterministic built-in evidence, including kernel-owned primitive evidence accepted by the kernel. | yes | Distinct from ATP `KernelVerified`; exports a deterministic discharge hash for reuse validation. |
| `PolicyPermittedExternal` | External admission with `may_win_selection = true` and `require_kernel_certificates = false`. | no | Never trusted and never supplies trusted `used_axioms`. |
| `PolicyAssumed` | Explicit policy assumption admitted by the active policy and not blocked by `require_kernel_certificates`. | no | Non-trusted policy status only; never supplies trusted witness material or `used_axioms`. |
| `PolicyOpen` | Open obligation allowed by active policy after no schedulable/running better candidate remains. | no | Carries stable explanation refs only. |
| `Rejected` | Kernel rejection, evidence-format rejection, policy rejection, or invalid selection input. | no | Not a proof winner; selected only as the primary diagnostic result when no eligible non-rejected winner exists. |
| `DiagnosticOnly` | Backend diagnostics, backend proof payloads, cache records, timings, counterexamples, unsupported payloads. | no | Never a winner. |

When `require_kernel_certificates = true`, `PolicyPermittedExternal` is not an
eligible winner even if an external record is present. External evidence may
still be recorded according to `ExternalEvidenceAdmission`, but winner
selection treats it as below every kernel-checkable path and below release
publication acceptance. `PolicyAssumed` likewise cannot win under
`require_kernel_certificates`.

## Deterministic Ordering

The winner ordering is:

1. `KernelVerified` satisfying the active policy.
2. `DischargedBuiltin` satisfying the active policy.
3. `PolicyPermittedExternal`, only when external admission permits winning and
   kernel certificates are not required.
4. `PolicyAssumed`, only when policy assumption recording is enabled and
   kernel certificates are not required.
5. `PolicyOpen`, only when policy allows open output and no better candidate
   can still be produced.
6. Best diagnostic rejection.

`DiagnosticOnly` candidates are excluded from winner selection, but their
diagnostics may be attached to the selected result.

## Tie-Break Keys

Within the same winner class, candidates are sorted by this stable key tuple:

1. winner-class rank;
2. backend profile priority, lower numeric value first;
3. certificate/evidence-format priority, lower numeric value first;
4. encoded problem hash bytes;
5. policy profile id;
6. evidence payload hash or deterministic discharge hash;
7. provenance hash;
8. stable candidate source id.

The tuple is compared lexicographically. Every selectable or rejected candidate
must carry a stable candidate source id; if a producer cannot provide one, the
selection input is invalid and must be reported as policy diagnostic material
rather than silently falling back to iteration order. Non-backend candidates use
the maximum backend-profile priority sentinel. Candidates with no
certificate/evidence format use the maximum evidence-format priority sentinel.
Missing optional hashes sort after present hashes by using a tagged optional
field (`present(hash)` before `missing`). Collections inside a key are sorted by
stable byte ordering before hashing. Duplicate records with the same stable id
and identical canonical payload may be coalesced; the same id with a different
canonical payload is invalid selection input.

The following values must never appear in the tie-break tuple:

- candidate arrival order;
- backend completion order;
- backend runtime duration;
- wall-clock time;
- process id, worker id, thread id, temporary path, or scheduler priority;
- backend stdout/stderr bytes or diagnostic wording;
- cache hit/miss state or cache lookup time.

## Rejected Candidate Ordering

If no eligible non-rejected winner exists, selection chooses one rejected
candidate only to provide the primary diagnostic result. Rejected candidates use
a separate deterministic diagnostic tuple:

1. rejection source rank: kernel rejection, evidence-format rejection, policy
   rejection, invalid selection input;
2. architecture-19 failure category rank when available;
3. stable severity rank;
4. stable reason code;
5. encoded problem hash bytes;
6. evidence payload hash or deterministic discharge hash;
7. provenance hash;
8. stable candidate source id.

Backend diagnostics, cache records, timings, and unsupported payloads may be
attached as secondary diagnostic refs, sorted by stable diagnostic ref hash, but
they do not outrank kernel or policy rejection records and never become proof
winners.

If the candidate set is empty, or if every input is `DiagnosticOnly`, selection
returns a `NoSelectableEvidence` diagnostic outcome instead of inventing a
proof winner or choosing by input order. That outcome has no selected candidate
id, no trusted `used_axioms`, and a stable diagnostic result id derived from
the obligation identity, encoded problem hash, policy fingerprint, and the
`no_selectable_evidence` reason code. Its diagnostic refs are sorted by stable
diagnostic ref hash.

## Selected Reuse Metadata

Selection exports stable metadata used by proof reuse and later witness
publication:

| Field | Meaning |
|---|---|
| `selected_class` | Winner class selected by the deterministic ordering. |
| `policy_fingerprint` | Active `PolicyFingerprint`. |
| `encoded_problem_hash` | Stable hash of the encoded obligation. |
| `selected_evidence_hash` | Kernel evidence payload hash, external evidence hash, policy-assumption source hash, or open explanation hash, depending on class. |
| `selected_proof_witness_hash` | Witness payload artifact hash (`witness_artifact_hash`) when the selected `KernelVerified` candidate carries witness metadata that the current artifact schema can reference. It is not a hash of the `ProofWitnessRef` metadata object and does not prove committed manifest reachability. |
| `deterministic_discharge_hash` | Deterministic built-in discharge hash for `DischargedBuiltin`. |
| `external_admission_status` | External publication status for `PolicyPermittedExternal`. |
| `proof_witness_publication` | `available`, `external_dependency_gap`, or `not_applicable` for the selected class. |
| `selected_candidate_provenance_hash` | Producer-owned provenance hash from the selected candidate, when present. This is part of reuse identity, not proof authority. |
| `selection_reason` | Stable reason code describing whether the result came from class ordering, a primary rejected diagnostic candidate, or `NoSelectableEvidence`. |
| `tie_break_key_hash` | Stable hash of the actual tie-break tuple. |

For `KernelVerified`, proof reuse may depend on the accepted kernel result and
its trusted `used_axioms`. For `DischargedBuiltin`, reuse depends on the
deterministic discharge hash and the accepted built-in/kernel evidence path.
For `PolicyPermittedExternal`, `PolicyAssumed`, and `PolicyOpen`, reuse
metadata is a validation predicate only and does not become trusted acceptance.
Changing the selected provenance hash, selection reason, tie-break key hash, or
any selected witness/discharge/evidence hash must change downstream
proof-reuse validation identity.

Current artifact witness references do not yet support `discharged_builtin`
publication. Until the artifact schema gap is closed, selection may export a
deterministic discharge hash but must not export a selected proof witness hash
for that class; it must mark proof-witness publication as
`external_dependency_gap`. Committed witness publication remains a separate
`witness_store::publish_ref` concern after artifact-manifest reachability is
proved.

## Result Shape

The task-6 implementation must expose a stable result with:

- selected candidate id when a winner or primary rejected diagnostic candidate
  exists;
- selected winner class, or `NoSelectableEvidence` when there is no selectable
  or rejected candidate;
- selected reuse metadata;
- ordered diagnostic refs for non-winning candidates;
- a flag stating whether trusted `used_axioms` are available from an accepted
  kernel result.

The selected result must not synthesize trusted `used_axioms`. It may carry
trusted `used_axioms` only by referencing the accepted kernel result selected
as `KernelVerified` or `DischargedBuiltin`. The trusted marker used by
selection must be bound to the accepted kernel evidence hash and must match the
selected candidate's evidence hash.

## Artifact Proof Selection Merge

Artifact merge consumes portfolio selections and phase-12 built-in discharge
selections keyed by `VcId`. It emits one `ArtifactProofSelection` per `VcId` in
canonical `VcId` order.

Source/class compatibility is part of merge validation. Portfolio input may
publish `KernelVerified`, policy-permitted external, policy-assumed,
policy-open, rejected, and `NoSelectableEvidence` outcomes, but it must not
publish `DischargedBuiltin`. Built-in discharge input may publish
`DischargedBuiltin` or diagnostic outcomes (`Rejected` and
`NoSelectableEvidence`) for failed or absent built-in discharge evidence, but it
must not publish `KernelVerified`, policy-permitted external, policy-assumed, or
policy-open outcomes. Invalid source/class pairs are rejected before artifact
status projection.

Merge ordering uses the same winner-class rank as ordinary selection:
`KernelVerified`, `DischargedBuiltin`, policy-permitted external, policy
assumption, policy-open, rejected, then `NoSelectableEvidence`. Within the same
class, merge compares the selected result's `tie_break_key_hash` and then a
stable source rank (`Portfolio` before `BuiltinDischarge`). Duplicate inputs
from the same source for one `VcId` are invalid merge input.

The merge layer does not project final artifact status, stage witnesses, write
artifact manifests, run ATP, run kernel checks, or trust cache records. It
preserves `KernelVerified` and `DischargedBuiltin` as distinct trusted classes
and preserves policy-permitted external, policy-assumed, open, rejected, and
no-selectable outcomes as non-trusted classes for status projection.

## Public Enum Policy

Task 14 applies the public-enum forward-compatibility procedure to this
module. All public selection enums are downstream-facing API surfaces and must
remain `#[non_exhaustive]`; downstream consumers must keep wildcard match arms.
Adding or removing variants requires paired spec/test review because these
enums affect deterministic winner identity, diagnostics, artifact merge
compatibility, and proof-reuse metadata.

| Enum | Compatibility decision |
|---|---|
| `SelectionInputError` | forward-compatible |
| `ProofWinnerClass` | forward-compatible |
| `ProofWitnessPublication` | forward-compatible |
| `ProofSelectionSource` | forward-compatible |
| `ArtifactProofSelectionError` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Deferred Integrations

- Artifact witness publication for `DischargedBuiltin` remains
  `external_dependency_gap` until `mizar-artifact` supports that witness class.
- Cache lookup and proof reuse validation remain downstream integration work.
  Selection exports stable metadata only; it does not query or trust a cache.
- ATP early-stop may consume winner-class information later, but selection does
  not stop backend processes itself.
