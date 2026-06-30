# Crate Exit Report: mizar-build

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete.

Quality score: 96/100.

Score caps applied: none. The read-only closeout quality review found hard
gates passing and no score-cap condition.

## Scope

Milestone scope: autonomous `mizar-build` crate development through tasks 0-26
and closeout.

Included:

- crate plan, ordered task decomposition, EN/JA module specs, and audits;
- phase-0 planning and module-index source;
- deterministic task graph construction;
- scheduler, resource budget, cancellation, failure propagation, cache-aware
  scheduling seam, and deterministic artifact commit boundary;
- batch integration, cross-boundary determinism, implemented-seam
  architecture-22 equivalence, bilingual/source-spec audits, and module-boundary
  refactor gate.

Excluded:

- real `mizar-driver` build sessions, event streams, phase registry, and
  driver-owned `salsa` cache-query integration;
- real `mizar-ir` sealed output handles, output storage, and snapshot-handle
  rehydration;
- real producer artifact projection and publication tokens;
- `mizar-cache` `CacheKey`, dependency-fingerprint, and proof-reuse validation
  implementation.

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | [00.crate_plan.md](./00.crate_plan.md), module specs, [source_spec_correspondence.md](./source_spec_correspondence.md), and [architecture_22_follow_up_audit.md](./architecture_22_follow_up_audit.md) report no unresolved blocking/high specification inconsistency. |
| Test contract | Pass with deferred low item | Focused unit/integration tests cover planning, module index, task graph, scheduler, resource, cancellation, failure, cache seam, artifact commit, batch integration, determinism, and architecture-22 implemented-seam equivalence. BUILD-G-016 remains a documented non-blocking direct-helper `test_gap`. |
| Traceability | Pass | TODO task records, source/spec audit tables, bilingual audit tables, and task-specific reports trace implemented behavior to specs and tests. No `.miz` tests or expectations were changed for `mizar-build`. |
| Design/source sync | Pass | Task 22/25/26 audits synchronize source, tests, English design docs, and Japanese companion docs. |
| Boundary discipline | Pass | `mizar-build` has no `mizar-driver` dependency; unavailable driver/IR/producer-token integrations are `external_dependency_gap`; cache hits remain execution skips and never proof or semantic authority. |
| Verification | Pass | Closeout verification passed `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`. Task 26 also passed `cargo test -p mizar-build`, `cargo clippy -p mizar-build --all-targets -- -D warnings`, adjacent cache/artifact/VC/proof tests, and `git diff --check`. |
| Residual risk | Pass with classified deferrals | Remaining risks are classified as BUILD-G-016 `test_gap` and external dependency gaps for real driver, IR, producer-token, and full real clean/incremental integration. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 15/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 96/100 |

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| BUILD-G-016 | `sorted_manifest_updates` is covered indirectly through `commit_manifest_updates`, but lacks a direct focused standalone-ordering test. | Future artifact-commit hardening task. | Add a direct focused test before claiming method-level helper coverage. |
| BUILD-G-002 / BUILD-G-011 | `mizar-driver` is absent, so real requests, sessions, event streams, phase registry, cache-query adapter, and driver-owned `salsa` boundary cannot be consumed. | Future `mizar-driver` integration phase. | Add real driver crate/integration; consume it from driver-to-build direction only. |
| BUILD-G-003 / BUILD-G-012 | `mizar-ir` sealed output handles, output storage, and snapshot rehydration are absent. | Future `mizar-ir` integration phase. | Expose real IR output handles and rehydration boundary. |
| BUILD-G-004 / BUILD-G-013 | Real producer artifact publication tokens and full phase-15 emission inputs are unavailable. | Future producer/artifact integration phase. | Provide real producer publication authority; `mizar-build` must consume it without minting tokens. |
| BUILD-G-006 / BUILD-G-015 / BUILD-G-017 | Full real resolver/checker/VC/proof/kernel/driver clean/incremental/parallel equivalence is unavailable until external seams exist. | Future external integration phase. | Wire real driver sessions, IR rehydration, producer projection, and publication tokens. |
| BUILD-G-009 | Driver-owned cache query integration, IR output rehydration, and producer publication tokens remain absent. | Future driver/cache/artifact integration phase. | Driver-owned cache lookup calls `mizar-cache`; `mizar-build` continues to consume decisions only. |

## Human Review Surface

Primary human review should inspect:

- [00.crate_plan.md](./00.crate_plan.md)
- [todo.md](./todo.md)
- module specs under `doc/design/mizar-build/en/`
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
- [architecture_22_follow_up_audit.md](./architecture_22_follow_up_audit.md)
- [module_boundary_refactor_gate.md](./module_boundary_refactor_gate.md)
- `crates/mizar-build/src/`
- `crates/mizar-build/tests/`

## Test Expectation Summary

No `.miz` tests, expectation TOML files, or language-spec files were changed by
the `mizar-build` crate task stream. Rust tests were added or moved only under
`crates/mizar-build`.

## Verification

Commands run:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check
git diff --cached --check
```

Results: all passed.

Additional task-26 regression commands also passed:

```text
cargo test -p mizar-build
cargo clippy -p mizar-build --all-targets -- -D warnings
cargo test -p mizar-cache
cargo test -p mizar-artifact
cargo test -p mizar-vc
cargo test -p mizar-proof
```

`mizar-driver` is absent in this checkout, so `cargo test -p mizar-driver` is
not a runnable verification command and remains an `external_dependency_gap`.

The staged documentation check is run at the closeout commit boundary after
explicitly staging only closeout-related paths.

## Handoff

Next recommended work: begin the external integration phase in a separate task
stream, starting with the driver-owned build request/session/phase-registry and
cache-query boundary. Recommended reasoning: xhigh, because the next phase
crosses crate ownership, incremental verification, artifact publication, cache
reuse, and proof-trust boundaries. Lower reasoning is reasonable for a
docs-only inventory update; keep xhigh for implementation or cross-crate
design changes.
