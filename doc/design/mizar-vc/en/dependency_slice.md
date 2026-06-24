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

Out of scope:

- computing the slice in Rust before task 14;
- ATP translation, proof search, kernel proof acceptance, and certificate
  validation;
- proof/cache artifact persistence or corpus-runner integration;
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
- explicit upstream identifiers already present in `VcIr`, such as
  `CoreFormulaId`, `ContextEntryId`, `CoreDefinitionId`, premise refs,
  trace refs, policy keys, and evidence hashes.

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

`VcId` is ownership and ordering metadata for one `BuildSnapshot`; it is not an
input to the reusable cross-edit dependency-slice fingerprint. Artifact records
may store the current `VcId` next to the fingerprint for diagnostics and result
collation, but proof/cache reuse must validate the fingerprint content through
the `ObligationAnchor` and canonical VC/context keys described by
[architecture 22](../../architecture/en/22.incremental_verification_contract.md).

## Dependency Entry Classes

Task 14 may introduce a structured Rust enum, but the semantic classes are:

- `local_context`: a `ContextEntryId` and its formula, kind, provenance, and
  policy-input relationship;
- `generated_formula`: a `VcGeneratedFormulaId` and every generated formula it
  transitively references;
- `core_formula`: a `CoreFormulaId` used as a goal, context formula, checker
  fact, type predicate, generated formula leaf, or premise target;
- `definition`: a `CoreDefinitionId` referenced by definition boundaries,
  permitted unfoldings, unfold requests, or definitional discharge evidence;
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
- `seed`: seed handoff ids and seed mapping rows needed to keep concrete-VC
  cardinality stable.

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
exclude snapshot-local `VcId` and include:

- dependency-slice schema version;
- `VcKind`, status boundary, and evidence boundary;
- ordered dependency entries and conservative unknown markers;
- relevant policy keys/values;
- generated formula references and discharge evidence boundaries;
- stable anchor and context hash markers when available, or conservative
  unknown markers when unavailable.

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
