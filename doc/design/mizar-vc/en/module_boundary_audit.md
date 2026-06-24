# Module-Boundary Refactor Gate: mizar-vc

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_audit.md](../ja/module_boundary_audit.md).

Task 22 audits the `mizar-vc` source layout before crate closeout. It is an
audit-only task unless a required move-only split is found. This task does not
change Rust source, public APIs, diagnostics, deterministic renderings,
artifact-facing schemas, `.miz` fixtures, expectations, `doc/spec`,
traceability metadata, runner support, or downstream ATP/kernel/proof/cache
integration.

## Scope And Inputs

The audit follows [internal 07](../../internal/en/07.crate_module_layout.md),
the mizar-vc crate plan, and the current module specifications:

- [vc_ir.md](./vc_ir.md)
- [generator.md](./generator.md)
- [discharge.md](./discharge.md)
- [dependency_slice.md](./dependency_slice.md)
- [source_spec_audit.md](./source_spec_audit.md)
- [architecture_22_audit.md](./architecture_22_audit.md)

Current source line counts:

| Path | Lines | Primary responsibility |
|---|---:|---|
| `crates/mizar-vc/src/lib.rs` | 10 | Public module export boundary. |
| `crates/mizar-vc/src/vc_ir.rs` | 3517 | `VcSet`, VC IR data shapes, validation, status projection, anchors, fingerprints, rendering, and tests. |
| `crates/mizar-vc/src/generator.rs` | 3368 | Seed-intake candidate generation, flow-derived candidates, normalization, anchor construction, and tests. |
| `crates/mizar-vc/src/discharge.rs` | 2113 | Deterministic pre-ATP discharge, evidence/explanations, evidence hashing, and tests. |
| `crates/mizar-vc/src/dependency_slice.rs` | 2573 | Dependency-slice collection, reusable fingerprints, proof-reuse candidate keys, and tests. |
| `crates/mizar-vc/tests/determinism_suite.rs` | 676 | Cross-module deterministic pipeline/reuse coverage. |
| `crates/mizar-vc/tests/lint_policy.rs` | 849 | Manifest, public-module, lint-policy, audit, and public-enum guards. |

## Boundary Review

| Boundary | Finding | Decision |
|---|---|---|
| Public module exports | `lib.rs` exports exactly `vc_ir`, `generator`, `discharge`, and `dependency_slice`; this matches the paired module specs and lint-policy guard. | No split or re-export change required. |
| `vc_ir.rs` | Large file, but cohesive around owned data shapes, validation, deterministic rendering, status projection, anchors, and canonical fingerprints. Splitting validation/rendering/fingerprint helpers could reduce review size, but no mixed public ownership or behavior-boundary violation is present. | Watchlist only; no required move-only split before closeout. |
| `generator.rs` | Large file, but cohesive around seed-derived candidate production and normalization. Algorithm fixture helpers dominate test size. Private submodules for flow candidates or tests could be maintenance work, but current public API and spec boundary are aligned. | Watchlist only; no required move-only split before closeout. |
| `discharge.rs` | Medium-large file with a single deterministic discharge responsibility. Evidence records, rule selection, and tests are coupled by the discharge spec. | No split required. |
| `dependency_slice.rs` | Large file, but cohesive around dependency collection, unknown coverage, reusable fingerprinting, and proof-reuse candidate keys. Private helper split for fingerprint payloads or tests could reduce review friction, but source/spec ownership remains one module. | Watchlist only; no required move-only split before closeout. |
| Integration tests | `determinism_suite.rs` and `lint_policy.rs` are long but intentionally cover cross-module behavior and policy guards. | No split required for crate completion. |

## Classification

- `design_drift`: none. Public module boundaries still match the module specs
  and internal ownership map.
- `source_drift`: none. Source files are large but do not cross public
  ownership boundaries or contradict the documented module responsibilities.
- `source_undocumented_behavior`: none observed in this audit.
- `test_gap`: none for module-boundary gating. Task 22 is docs-only and uses
  diff checks; source-moving tasks would require Rust verification.
- `repo_metadata_conflict`: none observed.
- `deferred`: optional maintenance refactors may later split private helper
  clusters or tests inside `vc_ir`, `generator`, and `dependency_slice`, but
  those are not required for the crate exit gate and must be separate
  move-only tasks if pursued.

## Gate Decision

No move-only split is required before closeout. The crate remains ready for the
closeout quality review with the current public module boundaries. The line
counts are a maintenance watchlist, not a hard gate failure, because the files
remain aligned with their documented module responsibilities and all public
surfaces stay guarded by lint-policy and source/spec audits.
