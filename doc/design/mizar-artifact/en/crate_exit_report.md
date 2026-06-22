# Crate Exit Report: mizar-artifact

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: conditionally complete.

Quality score: 95/100.

Score caps applied: none. Task 17 is explicitly deferred as an
`external_dependency_gap`, so it is a classified residual risk rather than a
failed hard gate.

## Scope

Milestone scope: `doc/design/mizar-artifact/ja/todo.md` tasks 1-22, executed in
order under the autonomous crate development protocol.

Included:

- tasks 1-16 and 18-22;
- English and Japanese crate plan, module specs, audits, and this exit report;
- the new `mizar-artifact` workspace crate and Rust tests;
- final source/spec, bilingual documentation, implementation, and test
  sufficiency reviews.

Excluded:

- task 17 end-to-end phase-15 emission integration, deferred until real
  `mizar-kernel` task 16 outputs and `mizar-proof` task 11 witness
  staging/publication exist;
- language-spec, `.miz`, and expectation changes, because this crate owns
  artifact infrastructure schemas and storage rather than source-language
  behavior.

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | `00.crate_plan.md` and module specs classify known drift and keep task 17 deferred. |
| Test contract | passed | Rust unit/integration tests cover canonical JSON, hashes, schemas, store atomicity, manifest validation, determinism, and lint policy. No `.miz` intent changed. |
| Traceability | passed | `source_spec_correspondence.md` maps public APIs to specs and tests. |
| Design/source sync | passed | TODO module table, module specs, source layout, and bilingual docs are synchronized. |
| Boundary discipline | passed | The crate depends only on `mizar-session` and does not implement producer integration, cache trust, proof acceptance, or kernel behavior. |
| Verification | passed | Required Rust verification passed; `mizar-checker` could not be run because the workspace has no such package. |
| Residual risk | passed | Task 17 is the only deferred item and is classified as `external_dependency_gap`. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 95/100 |

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MA-017 | `external_dependency_gap` | Future phase-15 integration task spanning producer crates and `mizar-artifact` | Real `mizar-kernel` task 16 outputs and `mizar-proof` task 11 witness staging/publication exist, so `mizar-artifact` can receive producer projections without faking proof status or kernel acceptance. |

## Human Review Surface

The human reviewer should primarily inspect:

- [00.crate_plan.md](./00.crate_plan.md)
- [todo.md](./todo.md)
- [store.md](./store.md)
- [module_summary.md](./module_summary.md)
- [registration_summary.md](./registration_summary.md)
- [proof_witness.md](./proof_witness.md)
- [verified_artifact.md](./verified_artifact.md)
- [manifest.md](./manifest.md)
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
- this report and the paired Japanese documents
- `crates/mizar-artifact/src/`
- `crates/mizar-artifact/tests/`
- workspace membership and lockfile changes

## Test Expectation Summary

No `.miz` tests, traceability metadata, or `.expect.toml` files were changed.

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| N/A | This milestone added Rust schema/storage tests only. | N/A | N/A | N/A | N/A |

## Verification

Commands run:

```sh
cargo fmt --check
cargo clippy -p mizar-artifact --all-targets -- -D warnings
cargo test -p mizar-artifact
cargo test -p mizar-resolve
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

Results:

- `cargo fmt --check`: passed.
- `cargo clippy -p mizar-artifact --all-targets -- -D warnings`: passed.
- `cargo test -p mizar-artifact`: passed.
- `cargo test -p mizar-resolve`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test`: passed.
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: passed with four existing planned-requirement warnings and zero errors.
- `cargo test -p mizar-checker`: not run because no workspace package named `mizar-checker` exists.

## Handoff

Next recommended work: implement task 17 only after real kernel/proof producer
outputs exist, then connect `VerifiedArtifact` emission through the store and
manifest transaction APIs without moving proof acceptance or witness production
into `mizar-artifact`.

Recommended reasoning setting: high.

Rationale: the next task crosses crate boundaries and proof-output ownership,
so it needs careful boundary and determinism review.

Lower to medium for a narrow downstream consumer that only reads one schema.
Raise to xhigh if the task changes kernel acceptance, proof witness ownership,
or artifact compatibility policy.
