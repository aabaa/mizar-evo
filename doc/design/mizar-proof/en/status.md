# mizar-proof Status Projection

> Canonical language: English. Japanese companion: [../ja/status.md](../ja/status.md).

## Purpose

`mizar-proof` owns the projection from deterministic proof selections into
artifact-facing and diagnostics-facing proof status records.

Status projection is not proof acceptance. Trusted acceptance and trusted
`used_axioms` originate only from `mizar-kernel` `KernelCheckResult` values
whose status is `KernelCheckStatus::Accepted` and that are selected as
`KernelVerified` or `DischargedBuiltin`. Externally attested evidence, backend
diagnostics, backend-reported axiom lists, proof cache records, witness
metadata, and policy assumptions never become kernel-verified status or trusted
`used_axioms`.

## Inputs

Projection consumes:

- the `ArtifactProofSelection` stream produced by deterministic selection
  merge;
- the active `VerifierPolicy` or an equivalent publication profile for the
  current build;
- stable obligation identity supplied by the VC/artifact producer:
  `VcId`, obligation id, `ObligationAnchor`, obligation fingerprint, source
  range or diagnostic anchor, canonical VC fingerprint, canonical local-context
  fingerprint, dependency-slice fingerprint, and policy fingerprint;
- optional trusted kernel result references for selected trusted classes,
  bound to the selected accepted evidence hash;
- optional diagnostic or explanation references produced by the policy,
  kernel, ATP, or diagnostics owner.

Projection must reject stale or mismatched inputs. A `VcId`,
`ObligationAnchor`, source range, arrival order, completion time, runtime
duration, backend profile runtime, or cache hit alone is never proof identity
and is never sufficient for status reuse.

## Status Model

The internal projection result keeps the selection winner class and the
projected obligation status separate:

| Winner class | Projected obligation status | Trusted | Notes |
|---|---|---:|---|
| `KernelVerified` | `accepted` | yes | Requires a matching accepted kernel result. Final artifact publication as accepted also requires a publishable kernel witness reference. |
| `DischargedBuiltin` | `accepted` | yes | Requires a matching accepted kernel result. Final artifact witness publication remains an `external_dependency_gap` until artifact schema support exists. |
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
3. the projection input includes the accepted kernel result or a trusted
   kernel-owned reference derived from it;
4. the accepted kernel evidence hash matches the selection's selected evidence
   hash;
5. the kernel result is not policy-tainted and has status `Accepted`.

When these conditions hold, projection may expose a trusted used-axiom
reference or ordered axiom list exactly as supplied by the kernel-owned
accepted result. Projection must not merge in backend-reported axiom lists,
externally attested citations, cache dependency records, diagnostic hints, or
policy-assumption dependencies.

All non-trusted statuses set trusted `used_axioms` to absent. They may carry
untrusted diagnostic suggestions only through diagnostics-owned references that
are not accepted dependency facts.

## Diagnostics And Explanations

Every non-accepted projection has a stable explanation surface:

- `externally_attested` records the external admission status and policy
  fingerprint;
- `policy_assumed` records the policy assumption reason and policy fingerprint;
- `open` records the open-obligation reason and any best diagnostic reference;
- `rejected` records the failure layer: policy rejection, evidence rejection,
  kernel rejection, backend exhaustion, or invalid selection input;
- `NoSelectableEvidence` records a generated no-selectable-evidence diagnostic
  result id.

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

- `KernelVerified` can publish an accepted artifact status only when the
  matching witness reference is available;
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
- selected proof witness payload artifact hash (`witness_artifact_hash`), when
  publishable;
- deterministic discharge hash, when present;
- trusted used-axiom reference hash, when present;
- external admission status, when present;
- diagnostic or explanation reference hashes.

This metadata is a cache validation predicate. It is never proof authority.
Reuse additionally requires matching `ObligationAnchor`, canonical VC
fingerprint, canonical local-context fingerprint, dependency-slice fingerprint,
compatible verifier policy, matching proof evidence, and compatible referenced
dependency artifacts and schemas. Cache records cannot promote externally
attested, assumed, open, rejected, or no-selectable outcomes to trusted status,
and cannot synthesize trusted `used_axioms`.

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
