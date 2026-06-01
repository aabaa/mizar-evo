# mizar-session TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

| Module | Spec | Source | Status |
|---|---|---|---|
| ids | [ids.md](./ids.md) | `src/ids.rs` | [ ] |
| source_map | [source_map.md](./source_map.md) | `src/source_map.rs` | [~] |
| snapshot | [snapshot.md](./snapshot.md) | `src/snapshot.rs` | [ ] |
| source | [source.md](./source.md) | `src/source.rs` | [~] |
| retention | [retention.md](./retention.md) | `src/retention.rs` | [ ] |

The crate is the leaf identity/coordinate layer, so it is built bottom-up by
internal dependency: `ids` → `source_map` → `snapshot` → `source` → `retention`.
`SourceId` is the shared primitive that every other module references.

## Ordered Task List

Each task is sized to be implemented, tested, and committed on its own. Tasks are
in dependency order; a later task assumes the earlier ones are merged. Every task
should keep `cargo test -p mizar-session` green (see [Suggested Verification](#suggested-verification)).

### Module: ids (`src/ids.rs`)

1. **Opaque id primitives and id newtypes.** [ ]
   - Add `pub mod ids;` to `lib.rs`; re-export the public id types.
   - Define the internal `OpaqueId` primitive and a `Hash` newtype used by content ids.
   - Define `BuildSessionId`, `BuildRequestId`, `BuildSnapshotId(Hash)`, `SourceId`, `SourceMapId`, `SnapshotLeaseId` with `Debug`/`Clone`/`Copy`/`Eq`/`Hash` as appropriate.
   - Add the `IdError` enum (malformed/ wrong-domain/ unknown-registry/ overflow/ non-persistable serialization).
   - Tests: equality, copy/clone, that ids are opaque (no semantic ordering exposed).
   - Spec: [ids.md](./ids.md) "Public API", "Identifier Scope".

2. **Content-derived id encoding.** [ ]
   - Implement canonical lowercase-hex serialization/deserialization for `BuildSnapshotId` with a domain separator; reject malformed/ wrong-domain input via `IdError`.
   - Provide an internal hashing helper (domain separator + schema/toolchain identity hooks + sorted-collection requirement) that the snapshot module will feed (actual snapshot hashing is task 10).
   - Reject serializing allocator-issued ids into a published schema.
   - Tests: round-trip hex encode/decode; domain-separator change changes the id; published-schema serialization rejects non-persistable ids.
   - Spec: [ids.md](./ids.md) "Serialization", "Content-Derived Id Construction".

3. **Session id allocator.** [ ]
   - Define the `SessionIdAllocator` trait and a concrete in-memory allocator (monotonic counters or arena indexes) for session/request/source/source-map/lease ids.
   - Tests: ids are unique within one registry; allocator overflow surfaces `IdError`.
   - Spec: [ids.md](./ids.md) "Allocator-Issued Id Construction".

### Module: source_map (`src/source_map.rs`)

4. **Integrate `SourceId` into `SourceRange` and `LineMap`.** [ ]
   - Add `source_id: SourceId` to `SourceRange`; add `source_id` + `text_hash: Hash` to `LineMap`.
   - Keep byte-offset semantics; add a `with_source(source_id, text)` constructor and keep/adjust the existing `new` path.
   - Validate that a range/offset belongs to the expected source before conversion.
   - Cross-crate impact: update `mizar-lsp::range_mapper` call sites and tests to pass a `SourceId`.
   - Tests: existing line/column tests updated; cross-source range is rejected.
   - Depends on: 1. Spec: [source_map.md](./source_map.md) "Line Map", "Source Range".
   - Note: this is additive for the lexer — lexer keeps its own `SourceSpan`; the bridge stays in `mizar-lsp`. Confirm the span-bridging decision but do not block on a lexer change.

5. **Line/column overflow policy.** [ ]
   - Keep `LineColumn` values `u32`; add `SourceMapError::LineColumnOverflow`.
   - Report overflow instead of saturating/wrapping/narrowing from `usize`.
   - Tests: unrepresentable line/column reports overflow; normal multi-byte conversion still returns one-based Unicode scalar columns.
   - Depends on: 4. Spec: [source_map.md](./source_map.md) "Public API" (`LineColumn` note).

6. **Loading map.** [ ]
   - Add `LoadingMap`, `LoadingOrigin`, `LoadingMapSegment` (`Original` / `RemovedLeadingBom` / `NormalizedNewline`).
   - Implement loaded-text → original mapping, including identity when no transform changed offsets.
   - Tests: leading BOM maps loaded `0` → original byte `3`; CRLF→LF segments; composite mapping across a normalized segment.
   - Depends on: 4. Spec: [source_map.md](./source_map.md) "Loading Map", "Loaded-to-Original Mapping".

7. **Preprocess map and anchors.** [ ]
   - Add `PreprocessMap`, `PreprocessSegment` (`Original` / `RemovedComment` / `SyntheticWhitespace`) and `SourceAnchor`.
   - Implement lexical → source mapping, returning composite adjacent anchors at zero-length boundaries.
   - Tests: removed comments map to preserved ranges; lexical range spanning a removed comment yields a composite mapping; synthetic whitespace is not a primary user range.
   - Depends on: 6. Spec: [source_map.md](./source_map.md) "Preprocess Map", "Lexical-to-Source Mapping".

8. **`SourceMapService` and generated spans.** [ ]
   - Define the `SourceMapService` trait (`line_column`, `original_range_for_loaded`, `source_range_for_lexical`, `attach_generated_span`, `validate_range`) and a concrete implementation over the retained maps.
   - Add generated-span origins (`GeneratedSpanOrigin`) with a required reason.
   - Tests: each trait method on representative inputs; generated span without an origin is rejected.
   - Depends on: 5, 7. Spec: [source_map.md](./source_map.md) "Public API", "Generated Spans".

### Module: snapshot (`src/snapshot.rs`)

9. **Source-version record.** [ ]
   - Add `pub mod snapshot;`. Define `SourceVersion` and `SourceOrigin` (`Disk` / `OpenBuffer{version}` / `Generated{generator}`).
   - Provide the canonical sort key (package id, module path, normalized path, source hash).
   - Tests: deterministic ordering by canonical key independent of insertion order.
   - Depends on: 1, 4. Spec: [snapshot.md](./snapshot.md) "Source Version".

10. **Build snapshot identity.** [ ]
    - Define `BuildSnapshot` and `SnapshotInput`; compute content-derived `BuildSnapshotId` from canonical input (sorted source/dependency summaries, lockfile hash, toolchain, verifier-config hash), excluding session-local ids/timestamps.
    - Tests: identical canonical inputs ⇒ identical id; source/dependency/config change ⇒ different id; session-local ids absent from the hash.
    - Depends on: 2, 9. Spec: [snapshot.md](./snapshot.md) "Snapshot Identity".

11. **Snapshot registry and freshness.** [ ]
    - Define `SnapshotRegistry` with `create_snapshot`, `get`, and `is_current_for_request`.
    - Tests: created snapshot is retrievable; stale id is rejected by freshness; older snapshot is not reported as current.
    - Depends on: 3, 10. Spec: [snapshot.md](./snapshot.md) "Snapshot Creation", "Freshness Check".

12. **Snapshot leases (basic).** [ ]
    - Add `SnapshotLease` + `acquire_lease`/`release_lease` on the registry, tracking lease counts; no collection policy yet.
    - Tests: acquire/release adjusts counts; release mismatch surfaces `SnapshotError`.
    - Depends on: 11. Spec: [snapshot.md](./snapshot.md) "Snapshot Lease".

### Module: source (`src/source.rs`)

13. **Loaded-source types and loader surface.** [ ]
    - Define `SourceInput`, `SourceOriginInput`, `SourceOrigin`, `LoadedSource`, and the `SourceLoader` trait; implement `hash_text` and `normalize_path` (reuse existing `normalize_source_path`).
    - Tests: `source_hash` excludes absolute paths/document versions; identical text in different origins shares the hash.
    - Depends on: 1, 4, 6. Spec: [source.md](./source.md) "Public API", "Loaded Source".

14. **Disk source loading.** [ ]
    - Implement disk loading: path normalization + package-root enforcement, read bytes, UTF-8 validation (no lossy `U+FFFD`), leading-BOM strip, CRLF→LF normalization, `source_hash`, `LineMap`, and `LoadingMap` emission.
    - Tests: invalid UTF-8 rejected before line-map; leading BOM → loading map `0`↔`3`; CRLF handling; path outside root rejected.
    - Depends on: 13. Spec: [source.md](./source.md) "Disk Source Loading".

15. **Open-buffer and generated loading.** [ ]
    - Implement open-buffer loading (LSP document-version validation, URI→package path, BOM strip, CRLF normalize, loading map back to editor offsets) and generated-source loading (generator metadata + anchor).
    - Tests: open-buffer overrides disk only for the matching version; stale version rejected; generated source without metadata rejected.
    - Depends on: 14. Spec: [source.md](./source.md) "Open-Buffer Source Loading", "Generated Source Loading".

### Module: retention (`src/retention.rs`)

16. **Retention manager and leases.** [ ]
    - Add `pub mod retention;`. Define `RetentionManager`, `RetainSnapshotInput`, `RetainGuard`, `RetainOwner`, `RetentionReason`, and `retain_snapshot`/`release` with reference counting.
    - Tests: active lease prevents collection eligibility; duplicate release is reported without underflow.
    - Depends on: 12. Spec: [retention.md](./retention.md) "Retain", "Release".

17. **Current marks and collection.** [ ]
    - Add `mark_current`/`unmark_current`, `collect`, and `CollectionSummary`; implement the collection policy (no lease, no current mark, no retained map/explanation, IR phase-output lease released).
    - Tests: current mark prevents collection without other leases; releasing the final lease collects; a phase-output lease blocks collection until released; collection does not delete artifacts/cache.
    - Depends on: 16. Spec: [retention.md](./retention.md) "Collection", "Current Marks".

## Cross-Cutting Follow-ups

- [ ] Keep `ja/` module specs in sync if any API changes during implementation.
- [ ] Determinism property tests across the crate: identical canonical inputs ⇒ identical `BuildSnapshotId` and identical source-range conversions, independent of scheduling order.

## Suggested Verification

After each task, run:

```text
cargo test -p mizar-session
cargo test -p mizar-test
```

Task 4 changes the `LineMap` / `SourceRange` surface, so also run:

```text
cargo test -p mizar-lsp
```

Check off the task here (or move it to a "Completed" section) once its tests pass.

## Notes

- `mizar-session` is the leaf identity & coordinate crate; downstream crates consume its handles to agree on source/snapshot state.
- Keep `mizar-lexer` decoupled from this crate; lexer-token span integration is the frontend's responsibility (see [../../mizar-lexer/en/todo.md](../../mizar-lexer/en/todo.md)).
- Source maps and snapshot identity are internal compiler data, not stable external schemas.
