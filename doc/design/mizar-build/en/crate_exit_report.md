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

- real driver-owned build sessions, event streams, phase registry semantics,
  and driver-owned `salsa` cache-query integration;
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
| Boundary discipline | Pass | `mizar-build` has no `mizar-driver` dependency; unavailable driver-owned session/query inputs, IR handles, producer-token integrations, and owner-provided dispatch inputs are `external_dependency_gap`; cache hits remain execution skips and never proof or semantic authority. |
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

This table preserves the original closeout result and adds post-closeout task-27
annotations where later driver availability changes the wording. It does not
re-score the closeout gates above.

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| BUILD-G-016 | `sorted_manifest_updates` is covered indirectly through `commit_manifest_updates`, but lacks a direct focused standalone-ordering test. | Future artifact-commit hardening task. | Add a direct focused test before claiming method-level helper coverage. |
| BUILD-G-002 / BUILD-G-011 | Closeout originally recorded `mizar-driver` absence. Task 27 implements the remaining build-owned portion as the scheduler-selected dispatch callback; driver-owned requests, sessions, event streams, phase registry semantics, cache-query adapter, and `salsa` boundary remain outside `mizar-build`. | Completed task-27 dispatch seam plus future driver-owned integration phases. | The scheduler-selected callback is exposed from `mizar-build`; remaining owner inputs, producer outputs, publisher handles, and artifact tokens must come from their owning crates without adding a `mizar-driver` dependency to `mizar-build`. |
| BUILD-G-003 / BUILD-G-012 | Real IR sealed output handles, output storage, and snapshot rehydration are not available through a build-owned seam. | Future `mizar-ir` integration phase. | Expose real IR output handles and rehydration boundary from the owning crate without placeholder handles in `mizar-build`. |
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

At the original closeout, `mizar-driver` was absent and `cargo test -p
mizar-driver` was not runnable. Task 27 ran the now-available driver checks and
full workspace verification for the dispatch-seam contract while keeping
driver-owned runtime/session authority outside `mizar-build`.

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
