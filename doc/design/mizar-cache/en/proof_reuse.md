# Module: proof_reuse

> Canonical language: English. Japanese companion:
> [../ja/proof_reuse.md](../ja/proof_reuse.md).

Status: specified by task 10. Implementation begins in task 11.

## Purpose

`proof_reuse` validates whether proof-related cache records may be used as an
optimization for the current build.

The module consumes validation metadata exported by `mizar-proof`. It does not
own proof policy, deterministic winner selection, proof status projection,
witness publication, kernel checking, ATP execution, or artifact manifest
commit. A successful validation result can only say that a cached proof-related
phase output is compatible with the current inputs. It never accepts a proof,
never marks evidence as kernel-verified, and never creates trusted
`used_axioms`.

Trusted proof acceptance still comes only from `mizar-kernel`
`KernelCheckResult` values consumed by the proof/status owner layers.

Historical `mizar-proof` closeout and audit documents written before the
formal `mizar-cache` scaffold may describe cache support as absent or
design-only. Those statements record the proof crate's closeout state. For
current cache ownership and proof-reuse validation, this document and the
`mizar-cache` crate plan are authoritative.

## Inputs

Proof reuse validation consumes an immutable request derived from the current
cache key plus proof metadata exported by `mizar-proof`:

- `ObligationAnchor`;
- obligation fingerprint;
- canonical VC fingerprint;
- canonical local-context fingerprint;
- dependency-slice fingerprints;
- active verifier policy fingerprint;
- cache, proof-reuse, VC, artifact, witness, and checker schema versions;
- selected proof class and proof-evidence identity exported by `mizar-proof`;
- selected proof witness hash for `KernelVerified` selections when available;
- deterministic discharge hash for `DischargedBuiltin` selections;
- proof-reuse validation hash exported by status projection;
- selected evidence hash, selected candidate provenance hash, selection
  reason, and tie-break key hash;
- trusted used-axioms reference hash when `mizar-proof` exports one for a
  trusted class;
- dependency artifact availability/hash metadata supplied by the artifact/cache
  boundary;
- diagnostic/explanation refs used only to explain misses.

The validation request may reference a `CacheRecord`, but the record is not a
proof authority. The record contributes bytes and metadata to compare against
the request; it does not define trusted status.

## Reusable Classes

The validation predicate is class-aware:

| Exported proof class | Reuse requirement | Trusted by cache? |
|---|---|---:|
| `KernelVerified` | The selected proof witness hash, selected evidence hash, proof-reuse validation hash, `ObligationAnchor`, canonical VC/local-context/dependency-slice fingerprints, policy fingerprint, schema versions, and dependency artifacts all match. | no |
| `DischargedBuiltin` | The deterministic discharge hash, selected evidence hash, proof-reuse validation hash, `ObligationAnchor`, canonical VC/local-context/dependency-slice fingerprints, policy fingerprint, schema versions, and dependency artifacts all match. | no |
| `PolicyPermittedExternal` | May be compared or recorded as non-trusted validation metadata only when `mizar-proof` exports it as the selected class for the current policy. It is not a complete proof-reuse hit. | no |
| `PolicyAssumed` | May be compared or recorded as policy metadata only. It is not a complete proof-reuse hit. | no |
| `PolicyOpen`, `Rejected`, `NoSelectableEvidence` | May be compared or recorded for diagnostics only. They are not complete proof-reuse hits. | no |

`proof_reuse` must respect the completeness predicate exported by
`mizar-proof`. In the current upstream contract, only `KernelVerified` and
`DischargedBuiltin` can produce a complete proof-reuse hit that skips proof
recomputation. Non-trusted classes remain metadata for diagnostics or
status-owner recomputation and must be treated as misses for proof-reuse hit
purposes. `proof_reuse` must preserve the exported class unchanged, must not
reinterpret a non-trusted class as accepted, must not choose a different winner,
and must not decide whether a policy permits publication.

## Validation Predicate

Validation succeeds only when all required fields match exactly:

1. cache key schema and proof-reuse schema versions are supported;
2. `ObligationAnchor` and obligation fingerprint match;
3. canonical VC fingerprint matches;
4. canonical local-context fingerprint matches;
5. dependency-slice fingerprints match and are complete;
6. verifier policy fingerprint and policy-compatibility fields match;
7. selected proof class and proof-evidence identity match;
8. selected proof witness hash matches for `KernelVerified`;
9. deterministic discharge hash matches for `DischargedBuiltin`;
10. selected evidence hash, selected provenance hash, selection reason, and
    tie-break key hash match when supplied by `mizar-proof`;
11. trusted used-axioms reference hash matches when `mizar-proof` exports one;
12. proof-reuse validation hash matches;
13. dependency artifact availability and recorded domain/digest checks succeed;
14. no `uncacheable`, incomplete-footprint, unsupported-schema, unknown
    toolchain, or incompatible policy marker is present.

The trusted used-axioms reference hash is a validation field only. It may be
covered by the proof-reuse validation hash, but cache validation must never
expose trusted `used_axioms` from cache data and must reject any record that
attempts to synthesize them.

Any missing, malformed, unknown, unsupported, or mismatched required input is a
cache miss. A miss degrades to recomputation. It never degrades to proof
acceptance.

## Determinism

The validation predicate must not depend on:

- cache hit/miss timing;
- record arrival order or write order;
- file modification time;
- worker id, thread id, process id, temporary path, or scheduler priority;
- backend runtime duration, stdout/stderr order, or backend log wording;
- diagnostic ordering except through stable diagnostic/explanation refs;
- witness staging time or artifact manifest commit timing.

When multiple cache records or proof metadata candidates exist, `proof_reuse`
does not select a winner. It validates the candidate selected by upstream
`mizar-proof` metadata and treats any ambiguity or conflict as a miss.

## Failure Semantics

Validation returns a miss for:

- missing `ObligationAnchor`;
- stale or mismatched obligation, VC, local-context, or dependency-slice
  fingerprint;
- missing selected proof witness hash for `KernelVerified`;
- missing deterministic discharge hash for `DischargedBuiltin`;
- witness/discharge hash mismatch;
- selected evidence or proof-evidence identity mismatch;
- missing or mismatched trusted used-axioms reference hash when exported;
- proof-reuse validation hash mismatch;
- unsupported proof-reuse schema;
- unknown toolchain/schema compatibility;
- incompatible verifier policy;
- incomplete dependency footprint;
- missing or mismatched dependency artifact;
- explicit uncacheable marker;
- externally attested, assumed, open, rejected, or no-selectable metadata being
  presented as trusted acceptance.

Miss reasons are cache diagnostics only. They do not reorder published
diagnostics and do not affect proof status.

## Output Contract

A successful validation result may expose:

- the validated proof-reuse class exactly as exported by `mizar-proof`;
- the matched witness or deterministic discharge hash;
- matched validation hashes and schema versions;
- diagnostic refs explaining the reuse decision.

It must not expose:

- a new `KernelCheckResult`;
- kernel-verified status created from cache data;
- trusted `used_axioms` created from cache data;
- a selected winner not chosen by `mizar-proof`;
- a committed witness publication reference;
- artifact publication eligibility.

Downstream consumers must still ask the proof/status/artifact owners for proof
status projection and publication decisions.

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `PROOFREUSE-G001` | `external_dependency_gap` | `mizar-build` scheduler integration is not ready. Task 11 validates metadata locally and does not add placeholder scheduling. |
| `PROOFREUSE-G002` | `external_dependency_gap` | `mizar-ir` cache adapter integration is absent. Proof reuse validation must not invent IR adapter APIs. |
| `PROOFREUSE-G003` | `external_dependency_gap` | Artifact committed witness publication tokens remain artifact-owned. `proof_reuse` may compare selected witness hashes but must not synthesize committed publication refs. |
| `PROOFREUSE-G004` | `external_dependency_gap` | `DischargedBuiltin` artifact witness publication remains unsupported until the artifact witness schema grows a distinct trusted class. Reuse uses deterministic discharge hashes only. |
| `PROOFREUSE-G005` | `deferred` | Full clean/incremental equivalence across scheduler, artifact publication, and cache lookup remains the task-20 gate. |

## Tests For Task 11

Task 11 must cover at least:

- matching `KernelVerified` metadata validates;
- matching `DischargedBuiltin` metadata validates through deterministic
  discharge hash;
- each missing or mismatched required component blocks reuse:
  `ObligationAnchor`, obligation
  fingerprint, canonical VC fingerprint, local-context fingerprint,
  dependency-slice fingerprint, selected witness hash, deterministic discharge
  hash, selected proof class, selected proof-evidence identity, selected
  evidence hash, selected candidate provenance hash, selection reason,
  tie-break key hash, trusted used-axioms reference hash when exported,
  proof-reuse validation hash, policy, schema version, and dependency artifact;
- incomplete dependency footprint, unsupported schema, unknown toolchain, and
  explicit uncacheable marker miss;
- externally attested, policy-assumed, open, rejected, and no-selectable
  metadata never becomes kernel-verified or trusted `used_axioms`, and records
  that attempt to synthesize trusted `used_axioms` or trusted used-axioms
  reference hashes are rejected;
- upstream proof-reuse completeness is honored: classes whose exported
  completeness predicate is false, including current non-trusted classes, miss
  for proof-reuse hit purposes even when their metadata matches;
- cache record arrival order, write order, and cache hit/miss timing do not
  affect validation;
- lint or source-surface guards show task 11 does not add `mizar-build`
  scheduler stubs, `mizar-ir` adapter stubs, artifact committed publication
  placeholder APIs, or witness publication shortcuts.

## Non-Goals

This module does not run ATP, call the kernel, evaluate proof policy, select
proof winners, project proof status, publish witnesses, commit artifacts,
update cluster-db, schedule builds, or adapt IR payloads.
