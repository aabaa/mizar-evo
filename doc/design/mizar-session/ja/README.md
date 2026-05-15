# Module Specifications: mizar-session

> Canonical language: English. English canonical version: [../en/README.md](../en/README.md).

`mizar-session` は batch、watch、LSP builds が使う source identity、build snapshots、source versions、source maps、snapshot retention contracts を所有する。

Task scheduling、IR storage、artifact publication、diagnostic aggregation は所有しない。それらの crates は `mizar-session` の handles を consume し、自分たちが観測している exact source/dependency/configuration state について合意する。

## Context

- [doc/design/architecture/ja/00.pipeline_overview.md](../../architecture/ja/00.pipeline_overview.md) - phase boundaries and build snapshots
- [doc/design/architecture/ja/02.source_and_frontend.md](../../architecture/ja/02.source_and_frontend.md) - source loading、line maps、preprocessing maps、comments、source spans
- [doc/design/architecture/ja/11.artifact_and_incremental_build.md](../../architecture/ja/11.artifact_and_incremental_build.md) - source hashes、dependency artifact hashes、incremental reuse
- [doc/design/architecture/ja/12.diagnostics_and_lsp.md](../../architecture/ja/12.diagnostics_and_lsp.md) - LSP snapshots、source ranges、freshness
- [doc/design/internal/ja/01.compiler_driver_and_pipeline_scheduler.md](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md) - `BuildSnapshot`、task graph input identity、cancellation
- [doc/design/internal/ja/03.diagnostics_model_and_lsp_bridge.md](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md) - diagnostic indexing and open-buffer overlay
- [doc/design/internal/ja/06.ir_storage_and_snapshot_handles.md](../../internal/ja/06.ir_storage_and_snapshot_handles.md) - `PhaseOutputRef<T>`、side tables、retained snapshot handles

## Index

| Document | Maps To | Description | Status |
|---|---|---|---|
| [snapshot.md](./snapshot.md) | `crates/mizar-session/src/snapshot.rs` | `BuildSnapshot`、`SourceVersion`、snapshot identity、retention、freshness contracts | Draft |
| [source_map.md](./source_map.md) | `crates/mizar-session/src/source_map.rs` | `LineMap`、source ranges、preprocessing maps、generated spans、coordinate conversion | Draft |

## Planned Documents

| Document | Maps To | Description |
|---|---|---|
| `source.rs` | `crates/mizar-session/src/source.rs` | Source loading records、normalized paths、source hashes、open-buffer source text |
| `retention.rs` | `crates/mizar-session/src/retention.rs` | Snapshot leases、LSP/watch retention、garbage-collection policy |
| `ids.rs` | `crates/mizar-session/src/ids.rs` | Downstream crates が共有する opaque session identifiers |

## Crate Boundary

`mizar-session` は immutable identity and coordinate services を提供する。

- source file identity and source hashes
- source、dependency、lockfile、toolchain、verifier configuration state にまたがる build snapshot identity
- cache keys、artifacts、diagnostics、LSP publication が使う source-version records
- raw source、preprocessed lexical text、generated internal fragments 間の source-range conversion
- diagnostics、LSP views、phase outputs が snapshot を参照している間に使う snapshot retention handles

次のものは所有してはならない。

- phase tasks の scheduling
- typed IR phase outputs の storage
- cache compatibility decisions
- diagnostics の aggregation
- artifacts の publication
- proof policy の evaluation
