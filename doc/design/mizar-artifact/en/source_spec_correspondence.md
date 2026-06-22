# mizar-artifact Source/Spec Correspondence Audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

## Scope

Task 20 audits the crate after task 19. It traces public API and promised
artifact-facing behavior from the module specs to source and tests. The audit is
documentation-only: it does not change source behavior, schemas, diagnostics,
or public APIs.

Classification result:

- `spec_gap`: none found for the crate-owned surface.
- `test_gap`: none found for the crate-owned surface.
- `design_drift`: none found for the crate-owned surface.
- `source_drift`: none found for the crate-owned surface.
- `external_dependency_gap`: the existing upstream producer, proof, kernel,
  cache, and build-integration gaps remain listed below.
- `deferred`: task 17 emission integration remains deferred; task 22 owns any
  source-layout refactor decision.

## Public API Trace

| Spec | Public API surface | Source | Coverage | Finding |
|---|---|---|---|---|
| [todo.md](./todo.md), [00.crate_plan.md](./00.crate_plan.md) | Public modules `store`, `module_summary`, `registration_summary`, `proof_witness`, `verified_artifact`, `manifest` | `crates/mizar-artifact/src/lib.rs` | `tests/lint_policy.rs` guards the crate dependency boundary and workspace lint opt-in. | No finding. |
| [store.md](./store.md) | `ARTIFACT_HASH_CONSTRUCTION`; `CanonicalJson`, `CanonicalJsonError`; `SchemaVersion`, `SchemaVersionParseError`, `SchemaVersionSupport`, `SchemaVersionError`, `SchemaVersionErrorContext`, `MinorVersionPolicy`; `HashClass`, `CanonicalHashDomain`; `FieldPath`, `FieldPathError`; `PublishedArtifactPath`, `PublishedArtifactWrite`, `PublishedArtifactRead`, `PublishedArtifactReadOptions`, `ArtifactTextLocation`, `PublishedPathError`, `StoreIoOperation`, `StoreIoError`; constructors/accessors plus `artifact_hash_domain`, `write_published_artifact`, `read_published_artifact`, `canonical_json_bytes`, `canonical_json_string` | `crates/mizar-artifact/src/store.rs` | Store unit tests cover canonical JSON spelling, duplicate object keys, string escaping, schema-version policies, hash-domain framing/exclusions, path validation, atomic write/replace behavior, symlink/root escape rejection, corruption diagnostics, deterministic writes, and hash mismatch handling. | No finding. |
| [module_summary.md](./module_summary.md) | `MODULE_SUMMARY_SCHEMA_FAMILY`, `SOURCE_HASH_CONSTRUCTION`; `ModuleSummary`, `ModuleSummaryIdentity`, `SourceRangeSummary`, `ExportedSymbolSummary`, `ProofStatusSummary`, `ExportedLabelSummary`, `ModuleLexicalSummary`, `LexicalContributionSummary`, `ModuleReexportSummary`, `DependencyInterfaceRef`, `ModuleSummaryReadOptions`, `ModuleSummaryError`; `current_schema_version`, `schema_version_support`, `write_module_summary`, `module_summary_json`, `read_module_summary`, `ModuleSummary::{compute_interface_hash, refresh_interface_hash}` | `crates/mizar-artifact/src/module_summary.rs` | Unit tests cover round-trip canonical JSON, deterministic writer/hash behavior, canonical ordering and duplicate rejection, interface-hash stability/exclusion rules, importer-visible hash participation, schema-version rejection, hash-domain validation, module/hash mismatches, and missing/unknown field rejection. | No finding. |
| [registration_summary.md](./registration_summary.md) | `REGISTRATION_SUMMARY_SCHEMA_FAMILY`; `RegistrationSummary`, `ActivatedRegistrationSummary`, `RegistrationPatternSummary`, `RegistrationContributionSummary`, `RegistrationTraceArtifactRef`, `DependencyRegistrationRef`, `ArtifactHashClass`, `ArtifactHashRef`, `RegistrationKind`, `RegistrationVisibility`, `RegistrationAcceptedStatus`, `RegistrationContributionKind`, `RegistrationTraceKind`, `RegistrationSummaryReadOptions`, `SuppliedTraceArtifactRef`, `RegistrationSummaryError`; `current_schema_version`, `schema_version_support`, `write_registration_summary`, `registration_summary_json`, `read_registration_summary`, `RegistrationSummary::{compute_registration_interface_hash, refresh_registration_interface_hash}`, `ArtifactHashRef::{new, to_artifact_hash_string}` | `crates/mizar-artifact/src/registration_summary.rs` | Unit tests cover round-trip canonical JSON, deterministic writer/hash behavior, canonical ordering and duplicate rejection, trace references by hash, interface-hash stability/exclusion rules, importer-visible hash participation, schema-version rejection, module/hash mismatches, invalid hash domains/digests, broken trace cross-references, nested missing/unknown fields, and rejection of unaccepted/private payloads. | No finding. |
| [proof_witness.md](./proof_witness.md) | `PROOF_WITNESS_REF_SCHEMA_FAMILY`; `ProofWitnessRef`, `KernelAcceptanceMetadata`, `ProofStatus`, `EvidenceKind`, `ProofWitnessReadOptions`, `ProofWitnessError`; `current_schema_version`, `schema_version_support`, `write_proof_witness_ref`, `proof_witness_ref_json`, `read_proof_witness_ref` | `crates/mizar-artifact/src/proof_witness.rs` | Unit tests cover round-trip canonical JSON, deterministic writer behavior, schema-version rejection, hash-domain/digest validation, missing/unknown/empty fields, unsafe witness paths, status/evidence/certificate-format validation, and supplied witness artifact hash mismatches. | No finding. |
| [verified_artifact.md](./verified_artifact.md) | `VERIFIED_ARTIFACT_SCHEMA_FAMILY`; `VerifiedArtifact`, `VerifiedExport`, `ExportVisibility`, `ExportProofStatus`, `ExpressionMetadata`, `OverloadMetadata`, `ObligationMetadata`, `ObligationStatus`, `ArtifactDiagnostic`, `DiagnosticSeverity`, `DiagnosticRelated`, `BuildProvenance`, `DependencyArtifactHash`, `VerifiedArtifactReadOptions`, `VerifiedArtifactError`; `current_schema_version`, `schema_version_support`, `artifact_hash_excluded_paths`, `write_verified_artifact`, `verified_artifact_json`, `read_verified_artifact`, `interface_hash_input_json`, `implementation_hash_input_json`, `VerifiedArtifact::{compute_interface_hash, compute_implementation_hash, refresh_hashes}` | `crates/mizar-artifact/src/verified_artifact.rs`; public-helper API guard in `crates/mizar-artifact/tests/public_api.rs` | Unit tests cover round-trip canonical JSON, deterministic writer/hash inputs, public hash helper equivalence and ordering, schema-version rejection, source-range/path/timestamp validation, hash domains and hash participation, provenance and local-field hash exclusions, witness-reference consistency, proof-authority boundary rejection, raw-IR/cache/scheduler ownership-boundary rejection, canonical ordering, duplicate identities, and implementation-only/interface-hash boundaries. | No finding. |
| [manifest.md](./manifest.md) | `MANIFEST_SCHEMA_FAMILY`, `ARTIFACT_MANIFEST_PATH`; `ArtifactManifest`, `PackageIdentity`, `ManifestProvenance`, `ModuleArtifactEntry`, `ManifestProofWitnessEntry`, `DevelopmentArtifactEntry`, `ArtifactManifestReadOptions`, `ManifestFileReadOptions`, `PublishedManifestRead`, `ManifestFreshnessCheck`, `ManifestCommitOptions`, `ManifestTransaction`, `ManifestCommit`, `ManifestError`; `current_schema_version`, `schema_version_support`, `manifest_hash_domain`, `artifact_manifest_path`, `write_artifact_manifest`, `artifact_manifest_json`, `read_artifact_manifest`, `write_manifest_file`, `read_manifest_file`, `ManifestTransaction::{begin, base_manifest_hash, freshness_guard, stage_module, commit}` | `crates/mizar-artifact/src/manifest.rs` | Unit tests cover sorted manifest round-trips, deterministic writer and transaction output, rejection of unsorted/duplicate collections, old-or-new manifest visibility, canonical commit ordering, idempotent replay, changed-base and obsolete-freshness rejection, referenced `VerifiedArtifact`/summary/hash validation, exact witness coverage, witness and development artifact reachability, local hash exclusions, and deferred payload hash recomputation. | No finding. |
| [all module specs](./todo.md#hardening-and-cross-cutting-follow-ups) | Public enum forward-compatibility decisions for every current public enum | Source attributes in `src/*.rs`; per-module "Public Enum Forward Compatibility" sections | `tests/lint_policy.rs` guards that every public enum in `src/*.rs` is `#[non_exhaustive]` and documented in both English and Japanese module specs. | No finding. |

## Promised Behavior Trace

| Behavior promised by specs | Source implementation | Test coverage | Finding |
|---|---|---|---|
| Canonical UTF-8 JSON, sorted object keys, duplicate-key rejection, stable bytes, and hash-domain separation | `src/store.rs` canonical JSON and `CanonicalHashDomain` helpers | `canonical_json_*`, `hash_*`, and determinism tests in `store.rs` | No finding. |
| Schema-version parsing and compatibility checks before schema interpretation | Shared store helpers plus every schema reader's `schema_version_support` path | Store and per-schema incompatible-version tests | No finding. |
| Published path safety, root confinement, symlink rejection, atomic temp-and-rename writes, and corruption-detecting reads | `write_published_artifact`, `read_published_artifact`, path validation, temporary-file protocol | Store I/O, path, symlink, corruption, interrupted-write, replacement, and deterministic-write tests | No finding. |
| `ModuleSummary` excludes source-only metadata from `interface_hash` and rejects non-canonical collections | `src/module_summary.rs` projection/hash helpers and reader validation | Module-summary hash stability, hash participation, ordering, duplicate, mismatch, and reader rejection tests | No finding. |
| `RegistrationSummary` records activated accepted public registrations and trace references without owning trace payload production | `src/registration_summary.rs` schema, hash, and validation helpers | Registration-summary round-trip, trace-hash, private/unaccepted rejection, ordering, duplicate, and mismatch tests | No finding. |
| `ProofWitnessRef` stores references and kernel-acceptance metadata without loading or accepting proof payloads | `src/proof_witness.rs` schema and validation helpers | Proof-witness round-trip, hash mismatch, status/evidence matrix, path, missing/unknown field, and hash-domain tests | No finding. |
| `VerifiedArtifact` publishes stable projected data, rejects raw IR/cache/scheduler/proof-authority ownership leaks, and separates interface/implementation/artifact hashes | `src/verified_artifact.rs` schema, projection-input, hash-input, and validation helpers | Verified-artifact round-trip, raw-IR/boundary rejection, witness consistency, proof-authority rejection, hash participation/exclusion, public helper, and deterministic tests | No finding. |
| Manifest publication is manifest-first, deterministic, atomic at the final manifest path, and validates referenced artifacts without owning cache promotion | `src/manifest.rs` manifest reader/writer and `ManifestTransaction` | Manifest round-trip, transaction, replay, freshness/base hash, reference validation, witness coverage, reachability, and deterministic tests | No finding. |
| Determinism for the crate-owned artifact-facing surface | Canonical writers, sorted collections, hash helpers, store writes, manifest transactions | Task 18 determinism tests across store, schema writers, hash inputs, and manifest transactions | No finding. |
| Public enums are forward-compatible API surfaces while serialized unknown values remain rejected until a future schema revision | `#[non_exhaustive]` on public enums and strict string readers | Task 19 lint-policy guard plus existing unknown-enum reader rejection tests | No finding. |

## Remaining Gaps

No new `spec_gap`, `test_gap`, `source_drift`, or `design_drift` follow-up was
opened by this audit for crate-owned behavior. The remaining gaps are upstream
or explicitly deferred:

| Gap | Class | Disposition |
|---|---|---|
| ART-G-004 | `external_dependency_gap` | Full phase 15 emission still depends on real kernel/proof outputs and producer projections. Task 17 remains deferred. |
| ART-G-007 | `external_dependency_gap` | Resolver/checker producers still need to emit real `ModuleSummary` projections. The crate owns the stable schema/writer/reader only. |
| ART-G-008 / ART-G-010 | `external_dependency_gap` | Checker/proof/trace producers still need to emit real `RegistrationSummary` inputs and concrete trace artifacts. The crate owns the stable schema/writer/reader only. |
| ART-G-012 | `external_dependency_gap` | Concrete witness payload schemas, proof producer integration, accepted kernel result construction, and built-in certificate/primitive encodings remain upstream. |
| ART-G-014 | `external_dependency_gap` | Real resolver/checker/VC/proof/kernel projection inputs for full `VerifiedArtifact` emission remain upstream. |
| ART-G-015 | `external_dependency_gap` | Development artifact payload hash recomputation remains blocked on producer-owned payload schemas. |
| ART-G-016 | `external_dependency_gap` | Clean/incremental/parallel scheduler, cache-race, ATP completion order, and real-emission determinism remain upstream integration concerns. |
| Task 22 source layout | `deferred` | Module-boundary refactor decisions are intentionally left to task 22 so this audit does not mix behavior/source movement with correspondence reporting; if task 22 introduces nested private modules, the task-19 public-enum guard should be revisited for that layout. |

## Verification

This task is documentation-only. Required verification is `git diff --check`.
Rust verification is not required unless a later task changes Rust source.
