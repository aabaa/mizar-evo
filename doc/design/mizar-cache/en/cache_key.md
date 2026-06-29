# Module: cache_key

> Canonical language: English. Japanese companion: [../ja/cache_key.md](../ja/cache_key.md).

Status: specified by task 2. Source implementation begins in task 3.

## Purpose

`cache_key` owns canonical internal build cache keys for `mizar-cache`.

A `CacheKey` is a pure content identity for a requested phase output. It
records every identity, hash, schema version, dependency slice, and policy
fingerprint that can affect whether a cached output is reusable. Key
construction is not a trust decision: a key match only makes a cache record a
candidate for later compatibility checks, dependency-footprint validation, and
proof-reuse validation.

Cache keys are internal optimization data. They are never proof authority and
must not promote cache records, externally attested evidence, backend
diagnostics, backend logs, timing metadata, or cache hit/miss state into
kernel-verified status or trusted `used_axioms`.

## Public API

Task 3 should expose the following conceptual API. Exact Rust field wrappers
may use newtypes, but the semantic fields must remain visible to tests and
audit code.

```rust
pub const CACHE_KEY_SCHEMA_VERSION: &str;
pub const CACHE_KEY_HASH_DOMAIN: &str;

pub struct CacheKey {
    pub cache_schema_version: SchemaVersion,
    pub phase: PipelinePhase,
    pub work_unit: WorkUnit,
    pub source_identity: Option<SourceIdentity>,
    pub input_hashes: Vec<NamedHash>,
    pub dependency_hashes: Vec<DependencyHash>,
    pub dependency_slices: Vec<DependencySliceHash>,
    pub config_hash: Hash,
    pub schema_versions: Vec<NamedSchemaVersion>,
    pub policy_fingerprint: PolicyFingerprint,
    pub validation_inputs: CacheValidationInputs,
    pub final_hash: Hash,
}

pub struct CacheKeyBuilder { ... }

impl CacheKeyBuilder {
    pub fn new(request: CacheKeyRequest) -> Self;
    pub fn build(self) -> CacheKeyBuildOutcome;
}

pub enum CacheKeyBuildOutcome {
    Cacheable(CacheKey),
    Uncacheable(CacheKey),
    NoKey(CacheKeyBuildRejection),
}
```

`CacheKeyBuilder` is a pure projection from `CacheKeyRequest` to
`CacheKeyBuildOutcome`. It must not read mutable scheduler state, wall-clock
time, cache contents, record arrival order, write order, backend runtime,
diagnostics, or filesystem freshness. It must not inspect proof status beyond
already supplied policy and proof-reuse validation fingerprints.

The outcome is explicit so key construction can fail closed without panics or
impossible placeholder keys:

- `Cacheable` means every required field and validation input is present and no
  `uncacheable` marker applies;
- `Uncacheable` carries a canonical key for diagnostics and deterministic miss
  accounting, but cache lookup must treat it as a miss;
- `NoKey` means the request is structurally invalid or unsupported, such as an
  unknown cache-key schema, a conflicting duplicate canonical key, or a missing
  required identity that prevents even deterministic miss accounting.

## Data Structures

### CacheKey

| Field | Meaning |
|---|---|
| `cache_schema_version` | Schema version for the cache key and cache record compatibility rules. Unknown versions miss. |
| `phase` | Pipeline phase whose output is being requested, such as frontend, resolve, checker, VC, ATP, proof, artifact, or cluster-db. |
| `work_unit` | Phase-local unit identity, for example package, module, item, VC, obligation, or cluster-view unit. |
| `source_identity` | Optional source package/module/path/hash/edition identity. It is absent for package-global or dependency-only work units. |
| `input_hashes` | Named direct phase input hashes, including source, IR, side-table, or producer output hashes that affect this phase. |
| `dependency_hashes` | Referenced published dependency artifact hashes, manifest hashes, interface hashes, implementation hashes, and lockfile hashes. |
| `dependency_slices` | Explicit dependency-slice fingerprints used by the cached output. Proof/VC reuse must include VC, local-context, and dependency-slice fingerprints. |
| `config_hash` | Verifier/build configuration hash, including relevant computation limits and cache-affecting build settings. |
| `schema_versions` | Named schema versions for phase outputs, artifacts, proof-reuse metadata, dependency footprints, backend encodings, and canonical serialization that affect meaning. |
| `policy_fingerprint` | Active verifier/proof policy fingerprint. Policy compatibility is checked later and cannot be inferred from the final hash alone. |
| `validation_inputs` | Structured architecture-22 validation fields that compatibility checks and proof reuse must compare after lookup. |
| `final_hash` | Domain-separated hash over the canonical encoding of every field above except `final_hash` itself. |

### SourceIdentity

`SourceIdentity` records:

- `package_id`;
- `module_path`;
- normalized source path relative to the package or workspace policy;
- source content hash;
- language edition.

Display paths, absolute local paths, open-buffer version numbers, source map
allocation ids, diagnostics-only source origins, and local filesystem metadata
are excluded.

### Validation Inputs

`CacheValidationInputs` records the values that a cache lookup must validate
after an exact key match:

- cache schema compatibility;
- producing toolchain compatibility;
- all dependency artifact hashes and availability;
- complete dependency footprint status;
- `uncacheable` marker state;
- verifier policy compatibility;
- canonical VC fingerprint when the output is proof/VC-related;
- local-context fingerprint when the output is proof/VC-related;
- dependency-slice fingerprint set;
- `ObligationAnchor` fingerprint when proof reuse is obligation-scoped;
- selected proof witness hash or deterministic discharge hash when proof
  reuse can be trusted as a recomputation skip;
- proof-reuse metadata schema version and validation hash exported by
  `mizar-proof`.

A missing required validation input is not interpreted as "no dependency". It
marks the key or record uncacheable and forces a miss.

## Canonical Ordering

Every vector in `CacheKey` is sorted before hashing:

- `input_hashes` by `(name, domain, digest)`;
- `dependency_hashes` by `(dependency_kind, package_id, module_path, name,
  domain, digest)`;
- `dependency_slices` by `(slice_kind, owner, name, domain, digest)`;
- `schema_versions` by `(schema_family, name, version)`;
- validation-input collections by the same canonical keys as their
  corresponding top-level field, or by the explicit keys below when no
  top-level field exists.

Validation-input collection ordering is:

- dependency artifact availability by `(package_id, module_path, artifact_kind,
  artifact_path, domain, digest)`;
- dependency-slice validation fingerprints by
  `(slice_kind, owner, name, domain, digest)`;
- policy compatibility fields by `(policy_family, field_name)`;
- toolchain compatibility fields by `(toolchain_component, field_name)`;
- proof-reuse schema versions by `(schema_family, name, version)`;
- proof-reuse evidence identities by `(obligation_anchor_fingerprint,
  evidence_kind, witness_or_discharge_domain, witness_or_discharge_digest)`;
- diagnostic-only refs, when present only for miss explanations, by
  `(diagnostic_ref_kind, diagnostic_ref_hash)`.

Duplicate canonical keys with identical payloads are coalesced. Duplicate
canonical keys with different payloads are invalid key requests and must be
rejected before a `CacheKey` is produced.

Ordering must not depend on `HashMap` iteration, worker completion order,
cache record arrival order, record write order, diagnostics order, ATP runtime,
wall-clock time, process id, thread id, temporary file names, or filesystem
directory order.

## Stable Hashing

The final key hash uses an explicit domain:

```text
mizar-cache/cache-key/v1
```

The canonical encoding is length-prefixed and typed. Each field includes its
field tag, field value, and schema/version tag where applicable. Hash values
are encoded with their domain and digest bytes, not only display strings.

The `phase` and `work_unit` fields are always included in the hash, so two
phases cannot collide semantically even when all direct input hashes match.
`cache_schema_version`, `schema_versions`, and `policy_fingerprint` are also
hash inputs because they affect compatibility and interpretation.

`final_hash` is derived only from the canonical key fields. It excludes:

- cache hit/miss timing;
- record write order;
- record file path;
- temporary file path;
- backend runtime duration;
- backend stdout/stderr bytes unless those bytes are explicitly declared as
  semantic output inputs by the owning phase;
- diagnostics wording or explanation previews unless the owning phase declares
  them semantic;
- local absolute paths except explicitly local diagnostic-only paths, which
  are excluded from cache-key hashes.

## Fail-Closed Rules

Key construction and later cache reuse must fail closed:

- unsupported or unknown cache key schema means no key or miss;
- unsupported or unknown cache record schema means miss;
- unknown toolchain compatibility means miss;
- incomplete dependency footprint means uncacheable and miss;
- explicit `uncacheable` marker always means miss;
- missing dependency artifact hash means miss;
- policy incompatibility means miss;
- proof witness hash mismatch means miss;
- deterministic discharge hash mismatch means miss;
- proof-reuse metadata schema mismatch means miss.

`CacheKeyBuilder` may produce `CacheKeyBuildOutcome::Uncacheable` for
uncacheable work units only when it also sets the uncacheable validation input.
Such a key must not produce a reusable hit. Structurally invalid requests,
unsupported cache-key schemas, or conflicting duplicates produce
`CacheKeyBuildOutcome::NoKey`.

## Proof-Reuse Boundary

Proof reuse is not owned by `cache_key`; this module records only identity and
validation inputs. Task 11 consumes `mizar-proof` metadata to decide whether a
cached proof-related output may skip recomputation.

The key must include enough data for that later validation:

- `ObligationAnchor` fingerprint;
- canonical VC fingerprint;
- canonical local-context fingerprint;
- dependency-slice fingerprints;
- dependency artifact hashes;
- verifier policy fingerprint;
- proof-reuse metadata schema version;
- selected proof witness hash for `KernelVerified` reuse;
- deterministic discharge hash for `DischargedBuiltin` reuse.

Non-trusted proof classes, externally attested records, policy assumptions,
open/rejected outcomes, backend diagnostics, backend logs, and cache records
may be present as diagnostics or metadata but must never become trusted
acceptance by key construction or cache lookup.

## Tests

Task 3 must cover at least:

- deterministic keys for identical requests;
- canonical vector sorting independent of input order;
- any semantic field change changes `final_hash`;
- duplicate identical entries coalesce and conflicting duplicates are rejected;
- phase and work-unit domain separation;
- policy/schema/toolchain validation fields participate in the final key;
- proof-related validation inputs participate in the final key;
- missing complete-footprint markers force `Uncacheable` or `NoKey` outcomes;
- timing, arrival order, write order, and diagnostics-only fields are excluded;
- `CacheKeyBuilder` does not read cache contents or mutable scheduler state.

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `CACHEKEY2-G001` | `external_dependency_gap` | `mizar-build` cache-aware scheduler integration is not ready. This spec defines keys only; scheduler hit/miss behavior remains task 15. |
| `CACHEKEY2-G002` | `external_dependency_gap` | `mizar-ir` cache adapters are absent. This spec defines key inputs for adapters but does not create placeholder adapter APIs. |
| `CACHEKEY2-G003` | `external_dependency_gap` | Artifact committed-publication tokens remain artifact/proof-owned. Cache keys may carry witness/publication hashes as validation inputs only. |

## Non-Goals

This module does not perform cache lookup, read or write cache records, choose
proof winners, project proof status, validate proof reuse, run ATP, call the
kernel, publish artifacts, or decide scheduler readiness.
