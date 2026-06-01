# mizar-session TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

| Module | Spec | Source | Status |
|---|---|---|---|
| ids | [en/ids.md](./ids.md) | `src/ids.rs` | [ ] |
| source_map | [en/source_map.md](./source_map.md) | `src/source_map.rs` | [~] |
| snapshot | [en/snapshot.md](./snapshot.md) | `src/snapshot.rs` | [ ] |
| source | [en/source.md](./source.md) | `src/source.rs` | [~] |
| retention | [en/retention.md](./retention.md) | `src/retention.rs` | [ ] |

## Recommended Implementation Order

The crate is the leaf identity/coordinate layer, so build it bottom-up by internal
dependency. `SourceId` is the shared primitive that every other module references.

1. **ids** — `SourceId` / `BuildSnapshotId` / `SourceMapId` / lease ids + `SessionIdAllocator`. No internal dependencies.
2. **source_map** — finish `LoadingMap`, `PreprocessMap`, `SourceAnchor`, generated spans, and `SourceMapService`; integrate `SourceId` into `SourceRange` / `LineMap`.
3. **snapshot** — `SourceVersion`, `BuildSnapshot`, `SnapshotRegistry`, content-derived `BuildSnapshotId`, freshness checks.
4. **source** — `LoadedSource`, `SourceInput`, `SourceLoader` (UTF-8 validation, leading-BOM stripping, CRLF normalization, `source_hash`, `LoadingMap`); ties ids + source_map + snapshot together.
5. **retention** — `RetentionManager`, leases, current marks, collection policy.

## Per-Module Remaining Work

### source_map [~]
Done: `LineMap`, `SourceRange`, `LineColumn`, `LineColumnRange`, `SourceMapError`, coordinate conversion.
Remaining: `LoadingMap` (+ `LoadingOrigin`, `LoadingMapSegment`), `PreprocessMap` (+ `PreprocessSegment`), `SourceAnchor`, generated spans, `SourceMapService` trait, `SourceId` integration, `u32` overflow reporting.

### source [~]
Done: `NormalizedPath`, `normalize_source_path`, `SourcePathError` (path normalization only).
Remaining: `SourceInput` / `SourceOriginInput`, `LoadedSource`, `SourceLoader` trait, disk / open-buffer / generated loading, `source_hash`, BOM + newline normalization, `LoadingMap` emission.

## Cross-Cutting Tasks

- [ ] Introduce the `ids` module as the shared identity primitive (`SourceId`, `BuildSnapshotId`, `SessionIdAllocator`)
- [ ] Integrate `SourceId` into `SourceRange` / `LineMap`
- [ ] Deterministic, content-derived `BuildSnapshotId` hashing (canonical ordering, domain separators)
- [ ] Snapshot freshness + lease / retention lifecycle
- [ ] Property/determinism tests: identical canonical inputs ⇒ identical `BuildSnapshotId`
- [ ] Japanese companion sync for `ids.md`, `snapshot.md`, `retention.md` (currently missing under `ja/`)

## Notes

- `mizar-session` is the leaf identity & coordinate crate; downstream crates consume its handles to agree on source/snapshot state.
- Keep `mizar-lexer` decoupled from this crate; lexer-token span integration is the frontend's responsibility (see [../../mizar-lexer/en/todo.md](../../mizar-lexer/en/todo.md)).
- Confirm the span-bridging decision (lexer stays decoupled vs. lexer adopts session `SourceRange`) before integrating `SourceId` into `source_map`.
- Source maps and snapshot identity are internal compiler data, not stable external schemas.
