# Module Specifications: mizar-session

> Canonical language: English. Japanese companion: [../ja/README.md](../ja/README.md).

`mizar-session` owns source identity, build snapshots, source versions, source maps, and snapshot retention contracts used by batch, watch, and LSP builds.

It does not own task scheduling, IR storage, artifact publication, or diagnostic aggregation. Those crates consume `mizar-session` handles to agree on the exact source/dependency/configuration state they are observing.

## Context

- [doc/design/architecture/en/00.pipeline_overview.md](../../architecture/en/00.pipeline_overview.md) - phase boundaries and build snapshots
- [doc/design/architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) - source loading, line maps, preprocessing maps, comments, and source spans
- [doc/design/architecture/en/11.artifact_and_incremental_build.md](../../architecture/en/11.artifact_and_incremental_build.md) - source hashes, dependency artifact hashes, and incremental reuse
- [doc/design/architecture/en/12.diagnostics_and_lsp.md](../../architecture/en/12.diagnostics_and_lsp.md) - LSP snapshots, source ranges, and freshness
- [doc/design/internal/en/01.compiler_driver_and_pipeline_scheduler.md](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md) - `BuildSnapshot`, task graph input identity, and cancellation
- [doc/design/internal/en/03.diagnostics_model_and_lsp_bridge.md](../../internal/en/03.diagnostics_model_and_lsp_bridge.md) - diagnostic indexing and open-buffer overlay
- [doc/design/internal/en/06.ir_storage_and_snapshot_handles.md](../../internal/en/06.ir_storage_and_snapshot_handles.md) - `PhaseOutputRef<T>`, side tables, and retained snapshot handles

## Index

| Document | Maps To | Description | Status |
|---|---|---|---|
| [00.crate_plan.md](./00.crate_plan.md) | `doc/design/autonomous_crate_development.md` | Retrospective autonomous crate plan, gap classification, task decomposition, and exit criteria | Complete |
| [ids.md](./ids.md) | `crates/mizar-session/src/ids.rs` | Opaque session identifiers, ordering, serialization boundaries, and compatibility rules | Implemented |
| [source.md](./source.md) | `crates/mizar-session/src/source.rs` | Source loading records, normalized paths, source hashes, and open-buffer source text | Implemented |
| [snapshot.md](./snapshot.md) | `crates/mizar-session/src/snapshot.rs` | `BuildSnapshot`, `SourceVersion`, snapshot identity, retention, and freshness contracts | Implemented |
| [source_map.md](./source_map.md) | `crates/mizar-session/src/source_map.rs` | `LineMap`, source ranges, preprocessing maps, generated spans, and coordinate conversion | Implemented |
| [retention.md](./retention.md) | `crates/mizar-session/src/retention.rs` | Snapshot leases, LSP/watch retention, and garbage-collection policy | Implemented |
| [crate_exit_report.md](./crate_exit_report.md) | `doc/design/autonomous_crate_development.md` | Crate exit gates, quality score, deferred items, review surface, verification, and handoff | Complete |
| [todo.md](./todo.md) | `crates/mizar-session` | Module implementation order, status, completed follow-ups, and verification expectations | Living |

## Crate Boundary

`mizar-session` provides immutable identity and coordinate services:

- source file identity and source hashes;
- build snapshot identity across source, dependency, lockfile, toolchain, and verifier configuration state;
- source-version records used by cache keys, artifacts, diagnostics, and LSP publication;
- source-range conversion between raw source, preprocessed lexical text, and generated internal fragments;
- snapshot retention handles used while diagnostics, LSP views, or phase outputs still reference a snapshot.

It must not:

- schedule phase tasks;
- store typed IR phase outputs;
- decide cache compatibility;
- aggregate diagnostics;
- publish artifacts;
- evaluate proof policy.
