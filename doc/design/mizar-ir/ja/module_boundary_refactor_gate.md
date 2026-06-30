# mizar-ir Module-Boundary Refactor Gate

> 正本は英語です。英語版:
> [../en/module_boundary_refactor_gate.md](../en/module_boundary_refactor_gate.md)。

## Scope

この task-19 gate は、downstream consumer が crate に依存する前に、
`crates/mizar-ir/src` に oversized implementation file、混在した責務、module table と
module specification に沿って分割すべき private helper が残っていないかを監査する。

この audit では、この task で必要な source split は見つからなかった。各 implementation
module の production file は 1 つの public module と 1 つの owning module specification に
対応している。`src/lib.rs` は crate boundary のままである。total line count が大きい file は
test-heavy であり、その test は exercise する module に attached されたままでよい。

## Layout Inventory

| Source file | Production lines | Test lines | Total lines | Module boundary result |
|---|---:|---:|---:|---|
| `src/lib.rs` | 32 | 0 | 32 | crate boundary のみ。 |
| `src/identity.rs` | 1090 | 642 | 1732 | `identity.md` に対応する。helper は identity canonicalization / registry helper である。 |
| `src/storage.rs` | 1459 | 903 | 2362 | `storage.md` に対応する。slot lifecycle、typed handle、blob placement、side table、retain/release、collection はすべて storage responsibility である。 |
| `src/publisher.rs` | 874 | 1009 | 1883 | `publisher.md` に対応する。helper は currentness、allowed work unit、parent validation、canonical hash、publish rollback を扱う。 |
| `src/cache_adapter.rs` | 689 | 905 | 1594 | `cache_adapter.md` に対応する。private payload codec helper は cache-adapter boundary 内に残す。 |
| `src/projection.rs` | 786 | 844 | 1630 | `projection.md` に対応する。sort key と leakage scanner は projection-local である。 |

## Refactor Decision

現在、ownership boundary をまたいでいる file はない:

- `identity.rs` は `BuildSnapshotId` construction、dependency fingerprint、
  proof reuse、proof authority を所有しない。
- `storage.rs` は production file として最大だが、private helper はすべて
  `storage.md` が述べる storage slot / handle / blob / collection behavior を詳細化する。
  Storage は current publication、cache validation、artifact projection、proof status、
  driver/diagnostics integration を所有しない。
- `publisher.rs` は publication validation と deterministic hashing を所有し、cache-key
  construction や proof acceptance は所有しない。
- `cache_adapter.rs` は `mizar-cache` lookup / record outcome を消費し、private payload
  encoding を adapter 内に保つ。`CacheKey`、dependency fingerprint、proof-reuse policy を
  再実装しない。
- `projection.rs` は stable artifact projection boundary と leakage guard を所有する。
  manifest publication を行わず、raw IR / kernel / storage state を公開しない。

この task では source split を行わないため、Rust formatting、Clippy、crate test は
crate-closeout verification まで不要である。将来の module-boundary audit が Rust code または
test を移動する場合、その task では commit 前に `cargo fmt --check`、
`cargo test -p mizar-ir`、`cargo clippy -p mizar-ir --all-targets -- -D warnings`、
および moved API の source/spec audit と bilingual audit を再実行しなければならない。

## Classified Gaps

`mizar-ir` に current な source-layout 固有の `source_drift`、`design_drift`、
`boundary_violation`、`test_expectation_drift`、`repo_metadata_conflict` は見つからなかった。

system-level downstream integration は既存の `external_dependency_gap` risk のままである:
この gate は placeholder の `mizar-driver`、`mizar-diagnostics`、producer-token、
artifact-publication-token、cache-key、dependency-fingerprint、proof-policy、
kernel-acceptance API を追加しない。

## Audit Result

Task 19 は source move なしで close する。module table は internal 07 と `mizar-ir`
TODO が指定する public module level のままであり、5 つの implementation module は
crate closeout verification に進める状態である。
