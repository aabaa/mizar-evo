# Source/Spec Correspondence: mizar-artifact

> 正本は英語です。英語版: [../en/source_spec_correspondence.md](../en/source_spec_correspondence.md)。

## 範囲

この監査は [todo.md](./todo.md) の task 1-16 と 18-22 を対象にする。Task 17 は
real `mizar-kernel` / `mizar-proof` producer outputs が存在するまで
`external_dependency_gap` として deferred である。

## Public API trace

| Source/API | Design authority | Test evidence |
|---|---|---|
| `src/store.rs`: `CanonicalJson`, hash helpers, schema/version helpers, `ArtifactStore`, `ArtifactError` | [store.md](./store.md) | `src/store.rs` unit tests; `tests/determinism.rs`; `tests/lint_policy.rs` |
| `src/module_summary.rs`: `ModuleSummary`, `ExportedItem`, `ExportedLabel`, `ImportedInterface` | [module_summary.md](./module_summary.md) | `src/module_summary.rs` unit tests; `tests/determinism.rs` |
| `src/registration_summary.rs`: `RegistrationSummary`, `ExportedRegistration`, `ClusterTraceRef` | [registration_summary.md](./registration_summary.md) | `src/registration_summary.rs` unit tests; `tests/determinism.rs` |
| `src/proof_witness.rs`: `ProofWitnessRef` | [proof_witness.md](./proof_witness.md) | `src/proof_witness.rs` unit tests; `src/verified_artifact.rs` tests |
| `src/verified_artifact.rs`: `VerifiedArtifact`, projection metadata, diagnostics, explanation refs, provenance | [verified_artifact.md](./verified_artifact.md) | `src/verified_artifact.rs` unit tests; `tests/determinism.rs` |
| `src/manifest.rs`: `ArtifactManifest`, `ModuleArtifactEntry`, `ProofWitnessEntry`, `ManifestTransaction` | [manifest.md](./manifest.md) | `src/manifest.rs` unit tests; `tests/determinism.rs` |
| `src/lib.rs` public module boundaries and curated re-exports | [00.crate_plan.md](./00.crate_plan.md), all module specs | `tests/lint_policy.rs` |

## Gap classification

| ID | Class | Evidence | Disposition |
|---|---|---|---|
| MA-SC-001 | `boundary_violation` risk | Full phase-15 emission は real kernel/proof outputs に依存する。 | task 17 deferred。fake producer / placeholder proof acceptance は実装していない。 |
| MA-SC-002 | `test_gap` | この crate の `.miz` tests はない。 | 現 scope では source-language behavior ではなく infrastructure schemas/storage を所有するため対象外。Rust tests が executable contract。 |
| MA-SC-003 | `source_undocumented_behavior` | Public schema constructors は field sets に対応し、理由付き clippy allow を使う。 | Module specs と `tests/lint_policy.rs` で文書化・guard 済み。undocumented public behavior は残らない。 |
| MA-SC-004 | `design_drift` | Architecture/internal docs は `emit.md`、`commit.md`、`projection.md`、`provenance.md`、`development_artifact.md` に言及する。 | `store.md`、`verified_artifact.md`、`manifest.md`、`proof_witness.md`、`crate_exit_report.md` に明示的に fold 済み。 |

## Module-boundary gate

Source layout は TODO module table と一致する:

- `src/store.rs` は canonical JSON、hash helpers、schema reader helpers、store I/O
  を所有する。共有 parser/helpers を持つため最大の file だが、public boundary は
  cohesive である。
- `src/module_summary.rs`、`src/registration_summary.rs`、
  `src/proof_witness.rs`、`src/verified_artifact.rs`、`src/manifest.rs` はそれぞれ
  1 つの published schema family を所有する。
- Producer integration、cache trust logic、proof acceptance、cluster-db storage は
  この crate にはない。

Downstream consumers が使い始める前に private module split は不要である。将来
`store.rs` が canonical JSON/store helpers を越えて成長する場合、public APIs や
canonical rendering を変えずに parser helpers を private module へ分割する。

## Result

Task 1-16 と 18-22 について unresolved source/spec mismatch は残らない。Task 17
は明示的に deferred のままである。
