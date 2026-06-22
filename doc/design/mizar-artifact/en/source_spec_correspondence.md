# Source/Spec Correspondence: mizar-artifact

> Canonical language: English. Japanese companion: [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

## Scope

This audit covers tasks 1-16 and 18-22 of [todo.md](./todo.md). Task 17 is
deferred as `external_dependency_gap` until real `mizar-kernel` and
`mizar-proof` producer outputs exist.

## Public API Trace

| Source/API | Design authority | Test evidence |
|---|---|---|
| `src/store.rs`: `CanonicalJson`, hash helpers, schema/version helpers, `ArtifactStore`, `ArtifactError` | [store.md](./store.md) | `src/store.rs` unit tests; `tests/determinism.rs`; `tests/lint_policy.rs` |
| `src/module_summary.rs`: `ModuleSummary`, `ExportedItem`, `ExportedLabel`, `ImportedInterface` | [module_summary.md](./module_summary.md) | `src/module_summary.rs` unit tests; `tests/determinism.rs` |
| `src/registration_summary.rs`: `RegistrationSummary`, `ExportedRegistration`, `ClusterTraceRef` | [registration_summary.md](./registration_summary.md) | `src/registration_summary.rs` unit tests; `tests/determinism.rs` |
| `src/proof_witness.rs`: `ProofWitnessRef` | [proof_witness.md](./proof_witness.md) | `src/proof_witness.rs` unit tests; `src/verified_artifact.rs` tests |
| `src/verified_artifact.rs`: `VerifiedArtifact`, projection metadata, diagnostics, explanation refs, provenance | [verified_artifact.md](./verified_artifact.md) | `src/verified_artifact.rs` unit tests; `tests/determinism.rs` |
| `src/manifest.rs`: `ArtifactManifest`, `ModuleArtifactEntry`, `ProofWitnessEntry`, `ManifestTransaction` | [manifest.md](./manifest.md) | `src/manifest.rs` unit tests; `tests/determinism.rs` |
| `src/lib.rs` public module boundaries and curated re-exports | [00.crate_plan.md](./00.crate_plan.md), all module specs | `tests/lint_policy.rs` |

## Gap Classification

| ID | Class | Evidence | Disposition |
|---|---|---|---|
| MA-SC-001 | `boundary_violation` risk | Full phase-15 emission depends on real kernel/proof outputs. | Deferred task 17; no fake producer or placeholder proof acceptance was implemented. |
| MA-SC-002 | `test_gap` | No `.miz` tests exist for this crate. | Not applicable for current scope because `mizar-artifact` owns infrastructure schemas/storage, not source-language behavior. Rust tests are the executable contract. |
| MA-SC-003 | `source_undocumented_behavior` | Public schema constructors mirror field sets and use documented clippy allow exceptions. | Documented in module specs and guarded by `tests/lint_policy.rs`; no undocumented public behavior remains. |
| MA-SC-004 | `design_drift` | Architecture/internal docs mention `emit.md`, `commit.md`, `projection.md`, `provenance.md`, and `development_artifact.md`. | Folded explicitly into `store.md`, `verified_artifact.md`, `manifest.md`, `proof_witness.md`, and `crate_exit_report.md`. |

## Module-Boundary Gate

The source layout matches the TODO module table:

- `src/store.rs` owns canonical JSON, hash helpers, schema reader helpers, and
  store I/O. It is the largest file because downstream schema modules share its
  parser/helpers, but the public boundary remains cohesive.
- `src/module_summary.rs`, `src/registration_summary.rs`,
  `src/proof_witness.rs`, `src/verified_artifact.rs`, and `src/manifest.rs`
  each own one published schema family.
- No producer integration, cache trust logic, proof acceptance, or cluster-db
  storage appears in this crate.

No private module split is required before downstream consumers start using the
crate. If `store.rs` grows beyond canonical JSON/store helpers in a future
task, split parser helpers into a private module without changing public APIs
or canonical rendering.

## Result

No unresolved source/spec mismatch remains for tasks 1-16 and 18-22. Task 17
remains explicitly deferred.
