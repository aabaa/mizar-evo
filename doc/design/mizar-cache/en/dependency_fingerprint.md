# Module: dependency_fingerprint

> Canonical language: English. Japanese companion:
> [../ja/dependency_fingerprint.md](../ja/dependency_fingerprint.md).

Status: specified by task 4. Source implementation begins in task 5.

## Purpose

`dependency_fingerprint` owns the cache-side dependency fingerprint and
dependency-footprint contract for `mizar-cache`.

Dependency fingerprints are internal optimization inputs. They decide whether a
cached phase output is a candidate for reuse; they do not decide proof
acceptance, artifact publication, or trusted `used_axioms`. Deleting the cache
or recomputing every fingerprint must change only performance.

This module refines
[architecture 18](../../architecture/en/18.dependency_fingerprint.md) and the
cache validation parts of
[architecture 22](../../architecture/en/22.incremental_verification_contract.md).
It consumes producer-owned hashes and slices from `mizar-artifact` and
`mizar-vc`; it does not invent placeholder integration for `mizar-build`,
`mizar-ir`, proof witness publication tokens, or artifact commit scheduling.

## Responsibility

Owned by this module:

- cache-side fingerprint target names, domains, schema versions, and stable
  ordering;
- dependency footprint records that say whether a phase has complete,
  conservative, or uncacheable coverage;
- the initial slice granularity used by `mizar-cache`;
- rebuild-trigger classification over fingerprint changes;
- compatibility-diff inputs used by cache invalidation and by the semver-check
  handoff described in architecture 18;
- the conservative `uncacheable` marker used when a dependency footprint cannot
  be trusted.

Out of scope:

- proof status projection, deterministic proof winner selection, ATP backend
  policy, kernel acceptance, or trusted proof evidence;
- reading or writing cache records, blob storage, or cluster-db indexes;
- computing producer-side interface hashes, registration summaries,
  dependency slices, VC fingerprints, or proof-reuse metadata;
- scheduler integration, IR adapter integration, artifact committed
  publication-token integration, and end-to-end cache hit scheduling.

## Public Conceptual API

Task 5 may refine exact Rust names, but the semantic surface is:

```rust
pub const DEPENDENCY_FINGERPRINT_SCHEMA_VERSION: &str;
pub const DEPENDENCY_FINGERPRINT_HASH_DOMAIN: &str;

pub struct DependencyFingerprint {
    pub target: FingerprintTarget,
    pub identity: FingerprintIdentity,
    pub value_hash: Hash,
    pub schema_version: SchemaVersion,
}

pub struct DependencyFootprint {
    pub owner: FootprintOwner,
    pub phase: PipelinePhase,
    pub fingerprints: Vec<DependencyFingerprint>,
    pub slices: Vec<DependencySliceFingerprint>,
    pub completeness: FootprintCompleteness,
    pub uncacheable: bool,
}

pub enum FootprintCompleteness {
    Complete,
    ConservativeComplete,
    IncompleteUncacheable,
}

pub enum RebuildTrigger {
    ReuseAllowed,
    RebuildPhase,
    RebuildDependents,
    UncacheableMiss,
}
```

Task 5 may define shared trigger data shapes such as `RebuildTrigger` when they
are needed by the fingerprint API, but it does not implement trigger
evaluation. Mapping fingerprint deltas to phase invalidation and the
source/import/registration/policy/toolchain trigger fixtures are task 6.

`Complete` means all required dependencies for the phase are known at the
current granularity. `ConservativeComplete` means the footprint may be coarser
than ideal but contains every dependency family required for sound reuse;
false-positive rebuilds are allowed. `IncompleteUncacheable` means at least
one required dependency family is missing, unstable, unsupported, or available
only through an opaque local id. It must set `uncacheable = true` and force a
cache miss.

## Initial Slice Granularity

The ideal architecture names theorem, definition, cluster, notation, mode, and
attribute slices. The initial `mizar-cache` implementation uses a conservative
two-level granularity:

1. **Published dependency projection granularity.** Imported module and
   registration dependencies are keyed by `mizar-artifact` dependency-facing
   hashes:
   - module summary `interface_hash`;
   - registration summary `registration_interface_hash`;
   - manifest and lockfile identity hashes;
   - implementation/artifact hashes only for local refresh or phase outputs
     whose semantics depend on implementation bodies.
2. **Per-VC dependency-slice granularity.** Proof/VC-related cache entries
   consume `mizar-vc` per-VC dependency-slice fingerprints, canonical VC
   fingerprints, local-context fingerprints, `ObligationAnchor` fingerprints,
   and available deterministic-discharge or witness validation hashes.

Task 5 may compute coarser importer-visible fingerprints when producers do not
yet expose theorem/definition/cluster/notation/mode/attribute sub-slices. Such
coarsening is valid only when it is conservative: a changed coarse dependency
may rebuild more work than necessary, but an unchanged coarse dependency must
not hide a changed dependency used by the cached output.

The theorem/definition/cluster/notation/mode/attribute families remain the
semantic target taxonomy and must be preserved in target names and
diagnostics. When fine-grained producer hashes land, the schema may add
finer target entries without changing the cache-authority boundary.

## Fingerprint Targets

The cache-side target taxonomy is:

| Target | Required inputs |
|---|---|
| `source` | source content hash, package/module identity, language edition, source-affecting schema versions |
| `lexical_parse` | token/AST-affecting source hash, imported lexical summary fingerprints, parser/lexer schema, active notation/operator view hash |
| `module_interface` | module summary identity and `interface_hash` from `mizar-artifact` |
| `module_implementation` | implementation/artifact hash for local body refreshes, never importer-visible proof authority |
| `registration_interface` | registration summary identity and `registration_interface_hash` |
| `cluster_trace` | accepted registration/cluster/reduction trace replay hash and visible origin identity |
| `definition` | definition identity, statement/signature fingerprint, transparency/opacity policy, unfolding boundary |
| `theorem_statement` | theorem origin id, exported statement fingerprint, accepted-status visibility boundary |
| `proof_body` | local proof-body or witness-producing implementation hash for local refresh only |
| `vc_slice` | `mizar-vc` dependency-slice fingerprint, canonical VC/context fingerprints, status/evidence boundary |
| `proof_reuse_identity` | proof-reuse validation hash, witness/discharge hash identity, policy fingerprint, proof metadata schema |
| `policy_toolchain` | verifier policy, backend profile, toolchain/schema compatibility, language edition |
| `lockfile_manifest` | package graph, manifest hash, dependency artifact availability, dependency publication identity |

Every target has an identity key and a value hash. The identity pairs the same
semantic dependency across builds; the value hash decides whether it changed.
Identity and value must not be conflated.

## Stable Inputs And Exclusions

Fingerprints include:

- schema family and schema version;
- cache-key schema version when it affects interpretation;
- package id, module path, normalized public origin id, language edition, and
  lockfile identity where applicable;
- producer-owned interface, implementation, registration-interface, trace,
  witness, discharge, VC, local-context, and dependency-slice hashes;
- verifier policy and toolchain compatibility fields that affect output
  meaning or reuse eligibility;
- conservative unknown markers when dependency coverage is incomplete;
- accepted proof or registration status only when importer visibility depends
  on that projected status.

Fingerprints exclude:

- wall-clock time, backend runtime duration, cache hit/miss timing, scheduler
  priority, worker id, process id, thread id, record arrival order, write
  order, temporary path, and local absolute path;
- backend logs, backend diagnostics, diagnostic wording, and explanation
  previews unless an owning phase explicitly declares a diagnostic artifact as
  semantic output;
- raw snapshot-local ids such as `VcId`, dense generated-formula ids,
  source-map ids, arena ids, row ids, or source ranges when computing a
  cross-edit reusable fingerprint;
- unaccepted, recovered, open, rejected, externally attested, or backend-only
  proof material as a source of trusted status.

## Canonical Ordering And Hashing

All collections are canonicalized before hashing:

- fingerprint targets by `(target_kind, owner_package, owner_module,
  origin_id, target_name, schema_family)`;
- dependency slices by `(slice_kind, owner, name, domain)`;
- artifact availability entries by `(package_id, module_path, artifact_kind,
  artifact_path, domain)`;
- compatibility fields by `(family, field_name)`;
- unknown markers by `(family, reason, owner)`;
- rebuild-trigger rows by `(change_kind, target_kind, dependent_phase)`.

Identical duplicate identity keys with identical payloads are coalesced.
Duplicate identity keys with different payloads are structurally invalid and
must yield `IncompleteUncacheable` when a diagnostic footprint can still be
produced, or a no-footprint/no-key rejection when schema or identity data is too
malformed to build a canonical footprint.

Hashing is length-prefixed, field-tagged, and domain-separated with:

```text
mizar-cache/dependency-fingerprint/v1
```

Producer-owned hash domains are preserved. `mizar-cache` may wrap them in a
cache-side field tag, but it must not reinterpret an artifact hash, proof
witness hash, deterministic discharge hash, or kernel evidence hash as proof
authority.

## Dependency Footprint Completeness

A reusable footprint must prove that every required dependency family for the
phase is represented at the chosen granularity.

Completeness requirements:

- source-backed phases require source content, language edition, phase schema,
  and relevant direct input hashes;
- import-aware phases require manifest/lockfile identity and all visible
  module summary and registration summary hashes;
- registration, cluster, and reduction phases require accepted visible origins
  and replay/trace hashes for every visible contribution used;
- VC/proof phases require canonical VC fingerprint, local-context fingerprint,
  dependency-slice fingerprint, `ObligationAnchor` fingerprint, policy
  fingerprint, proof-reuse metadata schema, and witness or deterministic
  discharge validation hash when reuse would skip recomputation;
- artifact-facing phases require dependency artifact availability hashes and
  publication-equivalent artifact hashes, but not artifact commit timing.

Use `IncompleteUncacheable` and set `uncacheable = true` when:

- a required producer hash is absent;
- a dependency family is known only by an opaque local id or diagnostic string;
- a schema or toolchain compatibility field is unknown;
- a dependency-slice producer reports unknown coverage;
- a proof/VC reuse input is missing, mismatched, externally attested only, or
  belongs to an unsupported evidence kind;
- a downstream owner seam has not landed and the missing seam is required to
  establish clean-build equivalence.

Missing data must never be interpreted as an empty dependency set.
Task 5 therefore treats missing compatibility fields and empty compatibility
field values, or values such as `unknown`, `unsupported`, `incompatible`,
`missing`, or `opaque`, as fail-closed unknown markers. VC/proof-phase
footprints with no per-VC slice fingerprint or no proof-reuse validation
metadata also become
`IncompleteUncacheable`.

## Rebuild Triggers

The trigger result answers "what must rerun before a cache hit is accepted?"
It is not proof acceptance and not semver classification.

| Change | Trigger |
|---|---|
| comment-only or diagnostic-wording-only change excluded from fingerprints | `ReuseAllowed` for semantic outputs |
| source content change that changes token/AST shape | rebuild lexical/parse and every dependent phase |
| module `interface_hash` change | rebuild importers that depend on that module summary |
| implementation/artifact hash change without interface change | refresh local implementation outputs; do not rebuild importers solely for exported semantics |
| registration interface or accepted visible origin change | rebuild visible registration, cluster, reduction, resolve/type, VC, proof, and cluster-db views that can see that origin |
| proof body change with unchanged theorem statement and accepted-status boundary | refresh local proof witness/implementation outputs; do not rebuild importers |
| theorem statement, definition signature, mode, attribute, notation, cluster, or exported algorithm contract change | rebuild every dependent footprint whose slice can see that target |
| verifier policy, toolchain compatibility, schema version, lockfile, or manifest identity change | miss and rebuild affected phases |
| incomplete footprint, unknown schema, unknown toolchain, uncacheable marker, or missing proof-reuse validation input | `UncacheableMiss` |

Coarse slices may over-trigger. They must not under-trigger.

## API Compatibility Diff

The API compatibility diff follows architecture 18:

- identity pairs elements across baseline and candidate builds;
- fingerprint value comparison detects changed semantic content after pairing;
- missing identity means removal or rename;
- new identity means addition;
- same identity with changed theorem statement, definition signature,
  algorithm contract, mode, attribute, notation, registration-visible value,
  export visibility, or language edition means an interface change.

The diff result is an input to semver-check and diagnostics. It is not a cache
reuse predicate. Cache reuse still requires exact dependency fingerprint,
schema, policy, toolchain, artifact availability, and proof/VC validation
matches.

Overload-resolution shift detection may remain heuristic at this layer unless
call-site traces are available. Heuristics may require conservative rebuilds,
but they must not authorize cache reuse.

## Proof And Trust Boundary

`dependency_fingerprint` may carry proof-reuse validation identities, accepted
status projections from producer-owned summaries, witness hashes, deterministic
discharge hashes, and kernel evidence handoff hashes. These are reuse inputs
only.

The module must not:

- create `KernelCheckResult`;
- mark any proof as kernel-verified;
- project trusted `used_axioms`;
- select proof winners;
- convert external attestation, backend success, backend logs, diagnostics, or
  cache records into accepted proof evidence.

Trusted acceptance remains owned by `mizar-kernel` results consumed through the
proof/status layers.

## Planned Tests

Task 5 must add coverage for:

- deterministic fingerprint output for identical inputs;
- canonical ordering and duplicate conflict rejection for every collection;
- stable fingerprints under comment-only, formatting-only, diagnostic-only,
  backend-runtime-only, scheduler-order-only, and cache-order-only changes;
- stable reusable fingerprints when temporary paths, local absolute paths,
  source ranges, `VcId`, dense generated-formula ids, source-map ids, arena
  ids, row ids, or other snapshot-local ids change without changing stable
  dependency payloads;
- interface hash changes invalidating importer-visible fingerprints;
- implementation/proof-body-only changes not invalidating importer-visible
  semantic fingerprints when interface and accepted-status boundaries match;
- per-VC dependency-slice changes flipping only the dependent VC/proof
  fingerprints at the chosen granularity;
- missing producer hashes, unknown schema/toolchain, incomplete dependency
  slices, unknown coverage, and uncacheable markers forcing miss outcomes;
- mismatched proof/VC reuse inputs, externally attested-only evidence,
  unsupported proof evidence kinds, and missing proof-reuse validation inputs
  forcing miss outcomes;
- proof-reuse validation inputs participating as untrusted validation data and
  never becoming proof authority.

Task 6 must add rebuild-trigger fixtures for source, import, registration,
cluster/reduction, policy, toolchain, schema, proof-body, and diagnostic-only
changes, including no false negatives in conservative coarse-slice mode.

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `DEPFPR-G001` | `external_dependency_gap` | `mizar-build` scheduler cache seam is not ready; do not add placeholder scheduler integration. |
| `DEPFPR-G002` | `external_dependency_gap` | `mizar-ir` cache adapter is absent; do not create adapter stubs. |
| `DEPFPR-G003` | `external_dependency_gap` | artifact committed publication token integration is not available; record availability/hash inputs only. |
| `DEPFPR-G004` | `deferred` | finer theorem/definition/cluster/notation/mode/attribute producer slices may be added later; task 5 starts conservatively from artifact summaries and `mizar-vc` per-VC slices. |
| `DEPFPR-G005` | `external_dependency_gap` | downstream proof/cache/artifact consumers of proof-reuse metadata remain owner-gated; this module only records validation identities. |

## Non-Goals

This module does not read cache records, write cache records, choose proof
evidence, call ATP, call the kernel, publish artifacts, schedule build tasks,
or expose an IR cache adapter.
