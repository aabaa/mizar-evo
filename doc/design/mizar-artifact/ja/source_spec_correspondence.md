# mizar-artifact ソース/仕様対応監査

> 正本は英語です。英語版:
> [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md)。

## 範囲

task 20 は task 19 後の crate を監査する。module spec から source と tests へ、
public API と約束された artifact-facing behavior を trace する。この監査は
documentation-only であり、source behavior、schema、diagnostic、public API は
変更しない。

分類結果:

- `spec_gap`: crate-owned surface では見つからない。
- `test_gap`: crate-owned surface では見つからない。
- `design_drift`: crate-owned surface では見つからない。
- `source_drift`: crate-owned surface では見つからない。
- `external_dependency_gap`: 既存の upstream producer、proof、kernel、cache、
  build-integration gap は下に残す。
- `deferred`: task 17 emission integration は deferred のまま。task 22 が
  source-layout refactor decision を所有する。

## Public API Trace

| Spec | Public API surface | Source | Coverage | Finding |
|---|---|---|---|---|
| [todo.md](./todo.md)、[00.crate_plan.md](./00.crate_plan.md) | public module `store`、`module_summary`、`registration_summary`、`proof_witness`、`verified_artifact`、`manifest` | `crates/mizar-artifact/src/lib.rs` | `tests/lint_policy.rs` が crate dependency boundary と workspace lint opt-in を guard する。 | No finding. |
| [store.md](./store.md) | `ARTIFACT_HASH_CONSTRUCTION`; `CanonicalJson`, `CanonicalJsonError`; `SchemaVersion`, `SchemaVersionParseError`, `SchemaVersionSupport`, `SchemaVersionError`, `SchemaVersionErrorContext`, `MinorVersionPolicy`; `HashClass`, `CanonicalHashDomain`; `FieldPath`, `FieldPathError`; `PublishedArtifactPath`, `PublishedArtifactWrite`, `PublishedArtifactRead`, `PublishedArtifactReadOptions`, `ArtifactTextLocation`, `PublishedPathError`, `StoreIoOperation`, `StoreIoError`; constructor/accessor と `artifact_hash_domain`, `write_published_artifact`, `read_published_artifact`, `canonical_json_bytes`, `canonical_json_string` | `crates/mizar-artifact/src/store.rs` | Store unit tests は canonical JSON spelling、duplicate object key、string escaping、schema-version policy、hash-domain framing/exclusion、path validation、atomic write/replace behavior、symlink/root escape rejection、corruption diagnostic、deterministic write、hash mismatch handling を覆う。 | No finding. |
| [module_summary.md](./module_summary.md) | `MODULE_SUMMARY_SCHEMA_FAMILY`, `SOURCE_HASH_CONSTRUCTION`; `ModuleSummary`, `ModuleSummaryIdentity`, `SourceRangeSummary`, `ExportedSymbolSummary`, `ProofStatusSummary`, `ExportedLabelSummary`, `ModuleLexicalSummary`, `LexicalContributionSummary`, `ModuleReexportSummary`, `DependencyInterfaceRef`, `ModuleSummaryReadOptions`, `ModuleSummaryError`; `current_schema_version`, `schema_version_support`, `write_module_summary`, `module_summary_json`, `read_module_summary`, `ModuleSummary::{compute_interface_hash, refresh_interface_hash}` | `crates/mizar-artifact/src/module_summary.rs` | Unit tests は round-trip canonical JSON、deterministic writer/hash behavior、canonical ordering と duplicate rejection、interface-hash stability/exclusion rule、importer-visible hash participation、schema-version rejection、hash-domain validation、module/hash mismatch、missing/unknown field rejection を覆う。 | No finding. |
| [registration_summary.md](./registration_summary.md) | `REGISTRATION_SUMMARY_SCHEMA_FAMILY`; `RegistrationSummary`, `ActivatedRegistrationSummary`, `RegistrationPatternSummary`, `RegistrationContributionSummary`, `RegistrationTraceArtifactRef`, `DependencyRegistrationRef`, `ArtifactHashClass`, `ArtifactHashRef`, `RegistrationKind`, `RegistrationVisibility`, `RegistrationAcceptedStatus`, `RegistrationContributionKind`, `RegistrationTraceKind`, `RegistrationSummaryReadOptions`, `SuppliedTraceArtifactRef`, `RegistrationSummaryError`; `current_schema_version`, `schema_version_support`, `write_registration_summary`, `registration_summary_json`, `read_registration_summary`, `RegistrationSummary::{compute_registration_interface_hash, refresh_registration_interface_hash}`, `ArtifactHashRef::{new, to_artifact_hash_string}` | `crates/mizar-artifact/src/registration_summary.rs` | Unit tests は round-trip canonical JSON、deterministic writer/hash behavior、canonical ordering と duplicate rejection、hash による trace reference、interface-hash stability/exclusion rule、importer-visible hash participation、schema-version rejection、module/hash mismatch、invalid hash domain/digest、broken trace cross-reference、nested missing/unknown field、unaccepted/private payload rejection を覆う。 | No finding. |
| [proof_witness.md](./proof_witness.md) | `PROOF_WITNESS_REF_SCHEMA_FAMILY`; `ProofWitnessRef`, `KernelAcceptanceMetadata`, `ProofStatus`, `EvidenceKind`, `ProofWitnessReadOptions`, `ProofWitnessError`; `current_schema_version`, `schema_version_support`, `write_proof_witness_ref`, `proof_witness_ref_json`, `read_proof_witness_ref` | `crates/mizar-artifact/src/proof_witness.rs` | Unit tests は round-trip canonical JSON、deterministic writer behavior、schema-version rejection、hash-domain/digest validation、missing/unknown/empty field、unsafe witness path、status/evidence/certificate-format validation、supplied witness artifact hash mismatch を覆う。 | No finding. |
| [verified_artifact.md](./verified_artifact.md) | `VERIFIED_ARTIFACT_SCHEMA_FAMILY`; `VerifiedArtifact`, `VerifiedExport`, `ExportVisibility`, `ExportProofStatus`, `ExpressionMetadata`, `OverloadMetadata`, `ObligationMetadata`, `ObligationStatus`, `ArtifactDiagnostic`, `DiagnosticSeverity`, `DiagnosticRelated`, `BuildProvenance`, `DependencyArtifactHash`, `VerifiedArtifactReadOptions`, `VerifiedArtifactError`; `current_schema_version`, `schema_version_support`, `artifact_hash_excluded_paths`, `write_verified_artifact`, `verified_artifact_json`, `read_verified_artifact`, `interface_hash_input_json`, `implementation_hash_input_json`, `VerifiedArtifact::{compute_interface_hash, compute_implementation_hash, refresh_hashes}` | `crates/mizar-artifact/src/verified_artifact.rs`; public helper API guard は `crates/mizar-artifact/tests/public_api.rs` | Unit tests は round-trip canonical JSON、deterministic writer/hash input、public hash helper equivalence/order、schema-version rejection、source-range/path/timestamp validation、hash domain と hash participation、provenance/local-field hash exclusion、witness-reference consistency、proof-authority boundary rejection、raw-IR/cache/scheduler ownership-boundary rejection、canonical ordering、duplicate identity、implementation-only/interface-hash boundary を覆う。 | No finding. |
| [manifest.md](./manifest.md) | `MANIFEST_SCHEMA_FAMILY`, `ARTIFACT_MANIFEST_PATH`; `ArtifactManifest`, `PackageIdentity`, `ManifestProvenance`, `ModuleArtifactEntry`, `ManifestProofWitnessEntry`, `DevelopmentArtifactEntry`, `ArtifactManifestReadOptions`, `ManifestFileReadOptions`, `PublishedManifestRead`, `ManifestFreshnessCheck`, `ManifestCommitOptions`, `ManifestTransaction`, `ManifestCommit`, `ManifestError`; `current_schema_version`, `schema_version_support`, `manifest_hash_domain`, `artifact_manifest_path`, `write_artifact_manifest`, `artifact_manifest_json`, `read_artifact_manifest`, `write_manifest_file`, `read_manifest_file`, `ManifestTransaction::{begin, base_manifest_hash, freshness_guard, stage_module, commit}` | `crates/mizar-artifact/src/manifest.rs` | Unit tests は sorted manifest round-trip、deterministic writer と transaction output、unsorted/duplicate collection rejection、old-or-new manifest visibility、canonical commit ordering、idempotent replay、changed-base/obsolete-freshness rejection、referenced `VerifiedArtifact`/summary/hash validation、exact witness coverage、witness/development artifact reachability、local hash exclusion、deferred payload hash recomputation を覆う。 | No finding. |
| [all module specs](./todo.md#強化と横断フォローアップ) | 現在の全 public enum の forward-compatibility decision | test-only ではない `src/**/*.rs` の source attribute; 各 module の「公開 enum の前方互換性」section | `tests/lint_policy.rs` が、test-only ではない source file の全 public enum が `#[non_exhaustive]` で英語/日本語 module spec の両方に文書化されていることを guard する。 | No finding. |
| [module_boundary_refactor.md](./module_boundary_refactor.md) | inline unit test は private `src/<module>/tests.rs` file へ移り、public module root は変わらない。 | `crates/mizar-artifact/src/*.rs`; private `crates/mizar-artifact/src/*/tests.rs`; `crates/mizar-artifact/tests/lint_policy.rs` | Unit tests は parent module 経由で引き続き実行される。public-enum guard は test-only file を除外しつつ source を再帰 scan する。 | No finding. |

## Promised Behavior Trace

| Specs が約束する behavior | Source implementation | Test coverage | Finding |
|---|---|---|---|
| Canonical UTF-8 JSON、sorted object keys、duplicate-key rejection、stable bytes、hash-domain separation | `src/store.rs` の canonical JSON と `CanonicalHashDomain` helpers | `store.rs` の `canonical_json_*`、`hash_*`、determinism tests | No finding. |
| schema interpretation の前に行う schema-version parsing と compatibility check | shared store helpers と各 schema reader の `schema_version_support` path | store と各 schema の incompatible-version tests | No finding. |
| published path safety、root confinement、symlink rejection、atomic temp-and-rename writes、corruption-detecting reads | `write_published_artifact`、`read_published_artifact`、path validation、temporary-file protocol | store I/O、path、symlink、corruption、interrupted-write、replacement、deterministic-write tests | No finding. |
| `ModuleSummary` は source-only metadata を `interface_hash` から除外し、non-canonical collection を拒否する | `src/module_summary.rs` の projection/hash helpers と reader validation | module-summary hash stability、hash participation、ordering、duplicate、mismatch、reader rejection tests | No finding. |
| `RegistrationSummary` は activated accepted public registration と trace reference を記録するが、trace payload production は所有しない | `src/registration_summary.rs` の schema、hash、validation helpers | registration-summary round-trip、trace-hash、private/unaccepted rejection、ordering、duplicate、mismatch tests | No finding. |
| `ProofWitnessRef` は proof payload を load/accept せず、reference と kernel-acceptance metadata を保存する | `src/proof_witness.rs` の schema と validation helpers | proof-witness round-trip、hash mismatch、status/evidence matrix、path、missing/unknown field、hash-domain tests | No finding. |
| `VerifiedArtifact` は安定した projected data を publish し、raw IR/cache/scheduler/proof-authority ownership leak を拒否し、interface/implementation/artifact hash を分離する | `src/verified_artifact.rs` の schema、projection-input、hash-input、validation helpers | verified-artifact round-trip、raw-IR/boundary rejection、witness consistency、proof-authority rejection、hash participation/exclusion、public helper、deterministic tests | No finding. |
| Manifest publication は manifest-first、deterministic、final manifest path で atomic であり、cache promotion を所有せず referenced artifact を validate する | `src/manifest.rs` の manifest reader/writer と `ManifestTransaction` | manifest round-trip、transaction、replay、freshness/base hash、reference validation、witness coverage、reachability、deterministic tests | No finding. |
| crate-owned artifact-facing surface の determinism | canonical writers、sorted collections、hash helpers、store writes、manifest transactions | store、schema writer、hash input、manifest transaction を横断する task 18 determinism tests | No finding. |
| public enum は forward-compatible API surface だが、serialized unknown value は future schema revision まで拒否される | public enum の `#[non_exhaustive]` と strict string readers | task 19 lint-policy guard と既存 unknown-enum reader rejection tests | No finding. |
| module-boundary refactor は public API path と artifact-facing behavior を保存する | parent module は同じ public root を維持し、`#[cfg(test)]` の private `tests` submodule だけを持つ | task 22 後の既存 crate tests、clippy、formatting | No finding. |

## 残る gap

crate-owned behavior について、この監査は新しい `spec_gap`、`test_gap`、
`source_drift`、`design_drift` follow-up を開かない。残る gap は upstream または
明示的に deferred されたものだけである。

| Gap | Class | Disposition |
|---|---|---|
| ART-G-004 | `external_dependency_gap` | full phase 15 emission はまだ real kernel/proof output と producer projection に依存する。task 17 は deferred のまま。 |
| ART-G-007 | `external_dependency_gap` | resolver/checker producer はまだ real `ModuleSummary` projection を emit する必要がある。この crate は stable schema/writer/reader だけを所有する。 |
| ART-G-008 / ART-G-010 | `external_dependency_gap` | checker/proof/trace producer はまだ real `RegistrationSummary` input と concrete trace artifact を emit する必要がある。この crate は stable schema/writer/reader だけを所有する。 |
| ART-G-012 | `external_dependency_gap` | concrete witness payload schema、proof producer integration、accepted kernel result construction、built-in certificate/primitive encoding は upstream に残る。 |
| ART-G-014 | `external_dependency_gap` | full `VerifiedArtifact` emission のための real resolver/checker/VC/proof/kernel projection input は upstream に残る。 |
| ART-G-015 | `external_dependency_gap` | development artifact payload hash recomputation は producer-owned payload schema に blocked されている。 |
| ART-G-016 | `external_dependency_gap` | clean/incremental/parallel scheduler、cache-race、ATP completion order、real-emission determinism は upstream integration concern のまま。 |

## Verification

この task は documentation-only である。必要な verification は `git diff --check`。
後続 task が Rust source を変更しない限り、Rust verification は不要である。
task 22 は tests 移動後にこの audit scope を再実行する。その Rust verification は
[module_boundary_refactor.md](./module_boundary_refactor.md) に記録する。
