# Module: dependency_slice

> Canonical language: English. Japanese companion:
> [../ja/dependency_slice.md](../ja/dependency_slice.md).

## Purpose

This module specifies conservative dependency slices for canonical `VcSet` data
after generation, normalization, status policy, and discharge. A dependency
slice records which local facts, generated formulas, imported facts,
definitions, registrations, cluster/reduction traces, policy inputs, and
evidence inputs a VC depends on for deterministic diagnostics, cache decisions,
artifact metadata, and later prover/proof consumers.

Task 26 extends the slice and proof-reuse identity boundary with the canonical
kernel evidence handoff hash produced by
[kernel_evidence_handoff.md](./kernel_evidence_handoff.md). The hash is a
reuse invalidation input only. It does not make `mizar-vc` a kernel caller and
does not promote a handoff package to proof acceptance.

Task 13 is specification-only. It refines
[architecture 18](../../architecture/en/18.dependency_fingerprint.md) and the
phase-12 boundary; it does not change language semantics, `.miz` fixtures,
expectations, `doc/spec`, traceability metadata, or Rust source.

## Responsibility

Owned by this module:

- stable per-VC dependency-slice data shape;
- dependency classification for context entries, premise refs, proof hints,
  discharge evidence, anchors, and generated formulas;
- conservative unknown-coverage markers that force cache misses or downstream
  recomputation instead of pretending dependencies are absent;
- stable dependency-slice fingerprints suitable for artifact and reuse keys.
- optional kernel evidence handoff identity entries used to invalidate
  proof-reuse candidates after task 25.

Out of scope:

- computing the slice in Rust before task 14;
- ATP translation, proof search, kernel proof acceptance, and certificate
  validation;
- proof/cache artifact persistence or corpus-runner integration;
- kernel checking, SAT solving, ATP backend execution, proof-witness
  persistence, or cache lookup;
- new source extraction, new VC generation, or new semantic payload families;
- treating unknown dependency coverage as an empty dependency set.

## Gap Classification For This Spec

| ID | Class | Evidence | Handling |
|---|---|---|---|
| DEP-G001 | `spec_gap` | `dependency_slice.md` did not exist before task 13, while tasks 14 and 20 need a dependency/fingerprint contract. | Task 13 adds only the English/Japanese spec. |
| DEP-G002 | `source_drift` / `test_gap` | `src/dependency_slice.rs`, slice data shapes, fingerprint helpers, and focused slice tests do not exist yet. | Task 14 implements source/tests against this spec. |
| DEP-G003 | `external_dependency_gap` | Complete registration, cluster, reduction, import, proof/cache, corpus, and artifact consumers are not all wired to `mizar-vc`. | The spec requires conservative markers for unavailable coverage and defers consumer integration. |
| DEP-G004 | `deferred` | Cross-edit reuse identity, canonical VC/context fingerprints, and [architecture 22](../../architecture/en/22.incremental_verification_contract.md) proof-reuse gates are task-20+ work. | Task 13 defines how dependency slices must feed those later fingerprints without implementing them. |

## Inputs And Outputs

Required input:

- a validated `VcSet`;
- generated formulas, local contexts, premises, proof hints, anchors, statuses,
  seed accounting, and discharge output evidence/explanations when available;
- optional task-25 `VcKernelEvidenceHandoff` values keyed by `VcId`, when a
  caller wants dependency slices and proof-reuse keys to include kernel
  evidence identity;
- explicit upstream identifiers already present in `VcIr`, such as
  `CoreFormulaId`, `ContextEntryId`, `CoreDefinitionId`, premise refs,
  trace refs, policy keys, and evidence hashes. These identifiers may appear in
  diagnostic dependency entries, but they are not reusable cross-edit
  fingerprint payloads unless the producing crate also exposes stable payload
  content.

Required output for each concrete VC:

- the snapshot-local VC id, VC kind, and status observed when the slice was
  computed;
- sorted dependency entries with a stable class and source reference;
- conservative unknown markers for any dependency family whose coverage is
  incomplete or unavailable;
- a stable cross-edit dependency-slice fingerprint over the normalized entries,
  schema version, policy inputs, status/evidence boundary, and unknown markers.

The slice must preserve VC order in any batch output. It must not remove VCs,
rewrite goals, mutate statuses, infer hidden facts, or treat a missing upstream
payload as proof that no dependency exists.

`VcId`, `ContextEntryId`, `VcGeneratedFormulaId`, `CoreFormulaId`,
`CoreDefinitionId`, seed handoff ids, candidate sort keys, source ids/ranges,
and dense owner ids are ownership and ordering metadata for one build snapshot;
they are not reusable cross-edit dependency-slice fingerprint inputs. Artifact
records may store the current ids next to the fingerprint for diagnostics and
result collation, but proof/cache reuse must validate fingerprint content
through the `ObligationAnchor` and canonical VC/context keys described by
[architecture 22](../../architecture/en/22.incremental_verification_contract.md).

## Dependency Entry Classes

Task 14 may introduce a structured Rust enum, but the semantic classes are:

- `local_context`: a context entry diagnostic id plus stable sort key, kind,
  provenance, policy-input relationship, and resolved formula payload when
  available;
- `generated_formula`: a generated-formula diagnostic id plus formula
  kind/shape/provenance payload and every generated formula it transitively
  references, with generated ids resolved before fingerprinting;
- `core_formula`: a diagnostic `CoreFormulaId` used as a goal, context formula,
  checker fact, type predicate, generated formula leaf, or premise target. A
  raw core row id is not reusable payload; if stable core formula content is
  unavailable, the slice must add conservative unknown coverage and use only an
  unresolved marker in the fingerprint payload;
- `definition`: a diagnostic `CoreDefinitionId` referenced by definition
  boundaries, permitted unfoldings, unfold requests, or definitional discharge
  evidence. A raw definition row id is not reusable payload; if stable
  definition content is unavailable, the slice must add conservative unknown
  coverage and use only an unresolved marker in the fingerprint payload;
- `imported_fact`: imported symbols and cited premises already present as
  `PremiseRef`;
- `trace`: explicit registration, cluster, reduction, and conservative-unknown
  trace refs;
- `policy`: policy keys/values that affected status, discharge, unfolding,
  computation limits, or ATP dispatch;
- `anchor`: complete and incomplete anchor ingredients that affect cache/reuse
  eligibility;
- `discharge_evidence`: rule names, evidence hashes, evidence inputs, and
  preserved-evidence markers from `DischargeOutput`;
- `kernel_evidence`: task-25 canonical kernel evidence handoff hash, target
  binding, schema/encoding/profile identity, and imported formula context
  requirements when supplied by the caller;
- `seed`: seed handoff ids and seed mapping rows needed to keep concrete-VC
  cardinality stable for diagnostics, while reusable fingerprint payloads use
  the current-obligation mapping shape rather than handoff ids.

Entries must be ordered by VC id, class rank, stable local key, then debug-stable
payload. Hash-map iteration order, absolute paths, wall-clock time, backend
availability, and worker scheduling must not influence the order.

## Conservative Unknown Coverage

Unknown coverage is a first-class dependency result, not an error by itself.
Use conservative markers when:

- a premise is `ConservativeUnknown`;
- an anchor is incomplete or a required anchor ingredient is unavailable;
- a registration, cluster, reduction, import, definition, or computation trace
  is only known by an opaque textual marker;
- discharge preserved pre-existing status evidence without replay data;
- upstream crates have not exposed enough payload to enumerate a dependency
  family completely.

Any slice containing unknown coverage must force cache misses or downstream
revalidation for consumers that need complete dependency precision. It must
still be deterministic and explain which family is incomplete. Unknown coverage
must not be silently dropped from fingerprints.

## Fingerprint Contract

The dependency-slice fingerprint is not a proof certificate. It is an
untrusted, deterministic reuse input. The reusable cross-edit fingerprint must
exclude snapshot-local ids (`VcId`, context/generated-formula/core/definition
row ids, handoff ids, source ids/ranges, candidate sort keys, and dense owner
ids) and include:

- dependency-slice schema version;
- `VcKind`, status boundary, and evidence boundary;
- ordered dependency entries and conservative unknown markers;
- relevant policy keys/values;
- generated formula references and discharge evidence boundaries;
- kernel evidence handoff hash and imported formula context requirements when a
  handoff is supplied;
- stable anchor and context hash markers when available, or conservative
  unknown markers when unavailable.

Diagnostic entry local keys may contain snapshot-local ids so that a developer
can inspect the slice, but fingerprint construction must not hash those local
keys. It must hash stable entry payloads and conservative-unknown family/reason
markers. When the only available payload for a dependency family is an opaque
row id, the slice is incomplete/uncacheable and the Task 20 proof-reuse
candidate key must not be produced.

Discharge evidence records may carry raw evidence-hash bytes for diagnostics or
artifact payloads. A reusable cross-edit dependency-slice fingerprint may include
those bytes only when the hash is known to be cross-edit stable. If the current
evidence hash may include snapshot-local ingredients such as `VcId`, the slice
must fingerprint the rule and hash availability/stability boundary instead, and
leave actual witness or deterministic-discharge hash validation to the
consumer-specific proof-evidence gate.

Consumers must not authorize proof/cache reuse from matching `VcId`, source
range, or anchor alone. Later reuse tasks must combine a confident
`ObligationAnchor` match, canonical VC fingerprint, context fingerprint,
dependency-slice fingerprint, policy/evidence hash, and consumer-specific
validation policy.

Task 20 adds a proof-reuse candidate-key helper for the currently available
deterministic discharge evidence boundary. Task 26 tightens that helper so a
candidate key is unavailable unless a current task-25 kernel evidence handoff
is supplied and its canonical hash participates in the slice/key identity. The
helper must return a key only when all of the following hold:

- the queried VC is taken from the same `DischargeOutput::vc_set()` that
  produced the evidence;
- the supplied `DependencySliceSet` matches a freshly computed slice for that
  same `VcSet`, `DischargeOutput`, and kernel evidence handoff by fingerprint,
  completeness, kind, and status;
- the `ObligationAnchor` is complete and the slice is complete;
- canonical VC and local-context fingerprints are available;
- a task-25 `VcKernelEvidenceHandoff` is supplied for the same VC and its
  canonical hash is included in the proof-reuse key payload;
- explicit verifier-policy inputs and status policy are included in a policy
  fingerprint;
- a newly produced replayable deterministic discharge evidence record exists
  for the same VC and matches the VC's `Discharged` status evidence.

The helper must return no key for absent kernel evidence handoff identity,
preserved/pre-existing discharged status, missing replay data, missing or
unstable evidence hashes, incomplete anchors, incomplete slices, stale slice
sets, non-discharged statuses, or policy/evidence mismatches. Proof-witness
hashes, cache lookup, kernel acceptance, ATP certificate validation, and
artifact consumers remain downstream `external_dependency_gap`s.

## Planned Tests

Task 14 must add Rust coverage for:

- local context, generated formula, core goal formula, premise, proof hint,
  policy, anchor, seed, and discharge-evidence dependencies;
- definition and permitted-unfolding dependencies;
- trace refs for registration, cluster, reduction, and conservative unknown
  markers;
- stable ordering and deterministic debug/fingerprint rendering;
- reusable fingerprint boundaries: otherwise identical slices with different
  snapshot-local `VcId`s must keep distinct owner/order metadata but have the
  same reusable dependency-slice fingerprint;
- unused local facts excluded when they are not referenced by the goal, premise,
  proof hint, discharge evidence, or policy boundary;
- missing or incomplete dependency coverage producing conservative unknown
  markers, cache-miss intent, and fingerprint participation;
- preservation of `NeedsAtp`, policy, skipped, deferred, error, and discharged
  status boundaries.

Later tasks must add coverage for cross-edit reuse identity and architecture-22
gates when canonical VC/context fingerprints and artifact consumers exist.

Task 26 adds Rust coverage for:

- kernel evidence handoff hashes participating in dependency-slice fingerprints
  and proof-reuse candidate keys;
- proof-reuse keys being unavailable when no kernel evidence handoff is
  supplied;
- proof-reuse invalidation when the canonical kernel evidence hash changes;
- duplicate, unknown, or selected-VC-mismatched kernel evidence handoff inputs
  failing closed;
- downstream proof/cache/artifact consumers remaining external/deferred.

## Public Enum Policy

Task 17 classifies every `dependency_slice` public enum as a downstream
forward-compatible API surface. Each enum must keep `#[non_exhaustive]` so later
slice completeness states, dependency classes, unknown families, and slice
errors can be added without breaking downstream exhaustive matches.

| public enum | decision |
|---|---|
| `DependencySliceCompleteness` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DependencyEntryClass` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DependencyUnknownFamily` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DependencySliceError` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module. Internal
`mizar-vc` matches that intentionally enumerate current variants may remain
exhaustive.
