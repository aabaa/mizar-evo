# Architecture-22 follow-up audit

> Canonical language: English. Japanese companion:
> [../ja/architecture_22_follow_up_audit.md](../ja/architecture_22_follow_up_audit.md).

Status: completed by task D-020.

## Scope

This audit re-checks the implemented `mizar-driver` seams against
[architecture 22](../../architecture/en/22.incremental_verification_contract.md)
after the D-016 determinism suite, D-018 source/spec correspondence audit, and
D-019 bilingual documentation sync audit.

The focused contract points are:

- driver-owned query boundary and cache-key purity;
- snapshot-scoped result freshness and obsolete-output suppression;
- diagnostics publication through `mizar-diagnostics` records instead of
  message text identity;
- cache and proof authority remaining outside the driver;
- artifact-publication serialization remaining outside the driver unless a real
  owner seam supplies a current artifact boundary record;
- deterministic local event, CLI, diagnostics-gap, and watch replay behavior
  across worker-count controls for implemented seams.

This audit does not claim full clean/incremental/parallel equivalence for real
semantic, proof, cache, or artifact producers. Those properties require owner
seams that are still classified below.

## Result

- No unresolved blocking or high architecture-22 drift was found for the
  implemented driver seams.
- The driver preserves snapshot-scoped publication: obsolete watch/LSP sessions
  cannot publish current diagnostics or artifact-boundary events.
- The phase registry remains the driver-owned query boundary. It observes pure
  `PhaseService::cache_key` projections but does not own cache compatibility,
  proof reuse, or artifact freshness decisions.
- Diagnostics are carried as `mizar-diagnostics` records/indexes or explicit
  owner-gap events; message text is not treated as diagnostic identity.
- Artifact publication remains a non-driver authority. The driver can carry a
  protocol-agnostic artifact-boundary event only when the producing owner
  supplies a current record; it does not mint publication tokens or serialize
  artifacts.
- D-016 supplies crate-local determinism coverage for implemented seams.
  Full clean/incremental/parallel equivalence with real producer/cache/artifact
  and proof reuse remains `deferred` until the corresponding owner seams exist.
- The D-020 audit document is paired in English and Japanese; the bilingual
  sync audit was refreshed to include this file pair.

## Architecture-22 Contract Matrix

| Contract point | Driver status | Evidence | Remaining classification |
|---|---|---|---|
| Clean-build equivalence | Implemented only for driver-local projections: request/session identity, event ordering, CLI output, diagnostics owner-gap output, unavailable-owner output, and watch replay suppression are deterministic across repeated runs and worker controls. | `tests/determinism.rs`; D-016 records in `todo.md` and `00.crate_plan.md`. | Full real clean/incremental/parallel equivalence remains `deferred` under `DRIVER-G-007` until real cache, producer, artifact, proof, and multi-task dispatch owner seams exist. |
| Cache-miss fallback and cache authority | `registry.rs` asks phase services for pure cache-key intent and reports cache observations. It does not decide cache compatibility or upgrade cache hits to proof authority. | `tests/registry.rs` cache-key purity and query-boundary tests; owner-boundary source guards in `tests/lint_policy.rs`, `tests/driver.rs`, and `tests/cli.rs`. | Real cache lookup/reuse validation remains outside the driver; unavailable semantic/proof/artifact adapters remain `external_dependency_gap` under `DRIVER-G-013`. |
| Snapshot-scoped results | Request/session lane and generation metadata determine currentness. Events validate session/snapshot identity and suppress obsolete publications. Watch replay mutates superseded sessions to suppressed publications. | `tests/request.rs`, `tests/events.rs`, `tests/watch.rs`, `tests/driver.rs`, and `tests/determinism.rs`. | No new gap for implemented driver seams. Real LSP bridge publication remains `external_dependency_gap` / `deferred` under `DRIVER-G-012`. |
| Query boundary | `PhaseRegistry`, `DriverQueryBoundary`, and `DriverQueryDatabase` own the driver-local salsa/query-compatible boundary. Syntax/parser/semantic phase crates do not gain a driver or query-engine dependency. | `tests/registry.rs` query-boundary coverage and `tests/lint_policy.rs` dependency guards. | No new gap for implemented driver seams. Scheduler-selected real phase dispatch remains `external_dependency_gap` under `DRIVER-G-011`. |
| Diagnostics freshness and identity | Driver events carry diagnostics owner records only for current sessions. CLI renders records through `mizar-diagnostics` when available and otherwise emits an explicit diagnostics bridge gap. Message text is not a diagnostic identity key. | `tests/events.rs`, `tests/cli.rs`, `tests/determinism.rs`; `source_spec_correspondence.md` behavior table. | Real frontend/module-index-to-diagnostic bridge gaps remain classified as `external_dependency_gap` under `DRIVER-G-010` and `DRIVER-G-013`. |
| Artifact publication boundary | Driver never commits artifacts in completion order, never serializes artifacts, and never creates a publication token. Artifact-boundary events require current session identity and owner-provided records. | `tests/events.rs`, `tests/watch.rs`, `tests/driver.rs`, `tests/cli.rs`, `tests/determinism.rs` guards against fake artifact output. | Artifact publication token and full phase-15 producer emission remain `external_dependency_gap` under `DRIVER-G-005` and `DRIVER-G-013`; missing `mizar-artifact` closeout remains report-only `repo_metadata_conflict` under `DRIVER-G-001`. |
| Parallel compatibility | Implemented driver projections sort events by deterministic order keys and keep CLI output stable across `--jobs` controls. Scheduler semantics are consumed from `mizar-build`; driver does not duplicate scheduler readiness or completion-order semantics. | `tests/events.rs`, `tests/driver.rs`, `tests/determinism.rs`; D-016 adjacent `mizar-build` verification. | Real scheduler callback dispatch for non-phase-zero work remains `external_dependency_gap` under `DRIVER-G-011`; full worker-race equivalence remains deferred under `DRIVER-G-007`. |
| LSP freshness | Driver request/session/event semantics include LSP lane currentness and obsolete-publication suppression, but event payloads remain protocol-agnostic. | `tests/request.rs`, `tests/events.rs`, `tests/watch.rs`, `tests/determinism.rs`; D-018 source/spec audit. | Real LSP protocol bridge remains outside the driver under `DRIVER-G-012`. |

## Follow-up Records

This audit does not introduce new blocking/high tasks.

Existing records remain the active follow-ups:

- `DRIVER-G-005`: artifact publication tokens and full producer emission are an
  `external_dependency_gap`.
- `DRIVER-G-007`: full clean/incremental/parallel equivalence with real cache,
  producer, artifact, proof, and worker-race seams is `deferred`.
- `DRIVER-G-010`: frontend canonical producer payload and diagnostics bridge
  readiness remains an `external_dependency_gap`.
- `DRIVER-G-011`: scheduler-selected real phase dispatch callback remains an
  `external_dependency_gap`.
- `DRIVER-G-012`: real file-watcher/coalescing owner and LSP bridge remain
  `external_dependency_gap` / `deferred`.
- `DRIVER-G-013`: semantic/proof/artifact phase adapters remain
  `external_dependency_gap`.
- `DRIVER-G-001` and `DRIVER-G-009`: artifact metadata conflicts remain
  report-only `repo_metadata_conflict` items for this task stream.

## Verification

D-020 is documentation-only. Required local checks are:

- `git diff --check`
- `git diff --cached --check` after staging the task-related paths

Rust verification is not required for this task unless review finds source
changes are needed. The audit evidence relies on the already-implemented
crate-local tests named above; final crate closeout will run the full workspace
verification suite.
