# Module-boundary refactor gate

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_refactor_gate.md](../ja/module_boundary_refactor_gate.md).

Status: completed by task D-021.

## Scope

This audit checks the completed `mizar-driver` source layout against
[internal 07](../../internal/en/07.crate_module_layout.md), the module specs,
and the source/spec correspondence records. The goal is to remove review
bottlenecks without changing public APIs, deterministic output schemas,
diagnostic records, artifact-boundary schemas, or consumer-visible behavior.

## Result

- No unresolved blocking/high module-boundary drift remains for the completed
  driver surface.
- The public module surface remains `cli`, `driver`, `events`, `registry`, and
  `request` from `src/lib.rs`.
- Only private helper modules were added. No phase semantics, proof/cache
  authority, artifact serialization, publication token, LSP conversion, or fake
  producer output moved into `mizar-driver`.
- Public API ownership remains documented by the module specs and
  `source_spec_correspondence.md`; moved code is private helper code or private
  unit-test code.

## Split Summary

| Area | Before | After | Boundary result |
|---|---|---|---|
| `cli` | `src/cli.rs` mixed argument parsing, request preparation, driver invocation, output rendering, JSON escaping, and exit-code rendering. | `src/cli.rs` keeps the public CLI request/invocation surface and driver submission preparation. `src/cli/output.rs` owns private rendering, JSON escaping, owner-gap output, and exit-code projection helpers. | Public `CliInvocation`, `CliBatchInput`, `CliOutput`, and `run_*` functions remain in the `cli` module. Output bytes are covered by existing CLI and determinism tests. |
| `driver` | `src/driver.rs` mixed public driver/session/watch types, submit/cancel orchestration, event construction, scheduler helper logic, watch helper logic, and unit tests. | `src/driver.rs` keeps public driver data types and submit/cancel orchestration. `src/driver/event_log.rs`, `src/driver/scheduler.rs`, and `src/driver/watch.rs` hold private helper logic. `src/driver/tests.rs` holds the private unit tests. | Public `CompilerDriver`, `BuildSubmission`, watch structs, and submit/cancel APIs remain in the `driver` module. Event ordering, cancellation, scheduler consumption, and watch freshness are still tested. |
| `registry` | `src/registry.rs` mixed public registry/query-boundary types with phase catalog, phase ranking, and stable query fingerprint helpers. | `src/registry.rs` keeps public registry/query-boundary types and service execution methods. `src/registry/catalog.rs` holds private phase requirements, ranking, owner/availability lookup, and stable fingerprint helpers. | Public `PhaseRegistry`, `DriverQueryBoundary`, `PhaseService`, and `required_phase_services` remain in the `registry` module. Cache-key purity and deterministic registration remain tested. |

## Source Size Check

This gate treats a file as a review bottleneck when it combines unrelated
private helper families or keeps a public facade above roughly 1,000 lines
after helper extraction. The split reduced the largest mixed-responsibility
files while keeping helper files single-purpose:

| File | Lines after split | Role |
|---|---|
| `src/request.rs` | 413 | request/session boundary |
| `src/registry.rs` | 602 | public registry and query-boundary API; was 808 lines before catalog extraction |
| `src/registry/catalog.rs` | 216 | private phase catalog and fingerprint helpers |
| `src/driver.rs` | 835 | public driver front door and submit/cancel orchestration; was 1,344 lines before helper/test extraction |
| `src/driver/event_log.rs` | 198 | private driver event construction |
| `src/driver/scheduler.rs` | 54 | private scheduler result projections |
| `src/driver/watch.rs` | 118 | private watch helper logic |
| `src/driver/tests.rs` | 182 | private driver unit tests |
| `src/events.rs` | 499 | protocol-agnostic event stream |
| `src/cli.rs` | 694 | public CLI request mapping and batch entry points; was 1,275 lines before output extraction |
| `src/cli/output.rs` | 594 | private CLI rendering and JSON helpers |

## Follow-up Records

No new blocking/high task is introduced by this gate.

Existing non-driver owner gaps remain unchanged:

- semantic/proof/artifact adapters remain `external_dependency_gap`;
- real LSP bridge and file-watcher/coalescing owner seams remain
  `external_dependency_gap` / `deferred`;
- full real clean/incremental/parallel equivalence remains `deferred`;
- missing `mizar-artifact` closeout metadata remains report-only
  `repo_metadata_conflict`.

## Verification

D-021 changes Rust source and design documentation. Per-task checks are:

- `cargo fmt --check`
- `cargo test -p mizar-driver`
- `cargo clippy -p mizar-driver --all-targets -- -D warnings`
- `git diff --check`
- `git diff --cached --check` after staging task-related paths

Adjacent crate tests are not required for this private module split unless
review finds cross-crate behavior drift. Because this is still a Rust source
change, final crate closeout must run the repository hard-gate commands:
`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
and `cargo test`.
