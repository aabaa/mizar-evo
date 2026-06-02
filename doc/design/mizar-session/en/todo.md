# mizar-session TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

| Module | Spec | Source | Status |
|---|---|---|---|
| ids | [ids.md](./ids.md) | `src/ids.rs` | [x] |
| source_map | [source_map.md](./source_map.md) | `src/source_map.rs` | [~] |
| snapshot | [snapshot.md](./snapshot.md) | `src/snapshot.rs` | [~] |
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

1. **Opaque id primitives and id newtypes.** [x]
   - Add `pub mod ids;` to `lib.rs`; re-export the public id types.
   - Define the internal `OpaqueId` primitive and a `Hash` newtype used by content ids.
   - Define `BuildSessionId`, `BuildRequestId`, `BuildSnapshotId(Hash)`, `SourceId`, `SourceMapId`, `SnapshotLeaseId` with `Debug`/`Clone`/`Copy`/`Eq`/`Hash` as appropriate.
   - Add the `IdError` enum (malformed/ wrong-domain/ unknown-registry/ overflow/ non-persistable serialization).
   - Tests: equality, copy/clone, that ids are opaque (no semantic ordering exposed).
   - Spec: [ids.md](./ids.md) "Public API", "Identifier Scope".

2. **Content-derived id encoding.** [x]
   - Implement canonical lowercase-hex serialization/deserialization for `BuildSnapshotId` with a domain separator; reject malformed/ wrong-domain input via `IdError`.
   - Provide an internal hashing helper (domain separator + schema/toolchain identity hooks + sorted-collection requirement) that the snapshot module will feed (actual snapshot hashing is task 10).
   - Reject serializing allocator-issued ids into a published schema.
   - Tests: round-trip hex encode/decode; domain-separator change changes the id; published-schema serialization rejects non-persistable ids.
   - Spec: [ids.md](./ids.md) "Serialization", "Content-Derived Id Construction".

3. **Session id allocator.** [x]
   - Define the `SessionIdAllocator` trait and a concrete in-memory allocator (monotonic counters or arena indexes) for session/request/source/source-map/lease ids.
   - Tests: ids are unique within one registry; allocator overflow surfaces `IdError`.
   - Spec: [ids.md](./ids.md) "Allocator-Issued Id Construction".

### Module: source_map (`src/source_map.rs`)

4. **Integrate `SourceId` into `SourceRange` and `LineMap`.** [x]
   - Add `source_id: SourceId` to `SourceRange`; add `source_id` + `text_hash: Hash` to `LineMap`.
   - Keep byte-offset semantics; add a `with_source(source_id, text)` constructor and keep/adjust the existing `new` path.
   - Validate that a range/offset belongs to the expected source before conversion.
   - Extend `SourceMapError` toward the full spec variant set, adding each variant as its feature lands: unknown source id, range outside source text, offset not on a UTF-8 boundary, line/column overflow (task 5), lexical range outside preprocessed text (task 7), missing loading-map segment (task 6), missing preprocess segment (task 7), generated span without an origin reason (task 8).
   - Cross-crate impact: update `mizar-lsp::range_mapper` call sites and tests to pass a `SourceId`.
   - Tests: existing line/column tests updated; cross-source range is rejected; unknown source id is rejected.
   - Depends on: 1. Spec: [source_map.md](./source_map.md) "Line Map", "Source Range".
   - Note: this is additive for the lexer — lexer keeps its own `SourceSpan`; the bridge stays in `mizar-lsp`. Confirm the span-bridging decision but do not block on a lexer change.

5. **Line/column overflow policy.** [x]
   - Keep `LineColumn` values `u32`; add `SourceMapError::LineColumnOverflow`.
   - Report overflow instead of saturating/wrapping/narrowing from `usize`.
   - Tests: unrepresentable line/column reports overflow; normal multi-byte conversion still returns one-based Unicode scalar columns.
   - Depends on: 4. Spec: [source_map.md](./source_map.md) "Public API" (`LineColumn` note).

6. **Loading map.** [x]
   - Introduce `TextRange` (a byte range into loaded or lexical text, kept distinct from `SourceRange` which is source-id-scoped).
   - Add `LoadingMap`, `LoadingOrigin`, `LoadingMapSegment` (`Original` / `RemovedLeadingBom` / `NormalizedNewline`).
   - Implement loaded-text → original mapping, including identity when no transform changed offsets.
   - Tests: leading BOM maps loaded `0` → original byte `3`; CRLF→LF segments; composite mapping across a normalized segment.
   - Depends on: 4. Spec: [source_map.md](./source_map.md) "Loading Map", "Loaded-to-Original Mapping".

7. **Preprocess map and anchors.** [x]
   - Add `PreprocessMap`, `PreprocessSegment` (`Original` / `RemovedComment` / `SyntheticWhitespace`) and `SourceAnchor`.
   - Implement lexical → source mapping, returning composite adjacent anchors at zero-length boundaries.
   - Tests: removed comments map to preserved ranges; lexical range spanning a removed comment yields a composite mapping; synthetic whitespace is not a primary user range.
   - Depends on: 6. Spec: [source_map.md](./source_map.md) "Preprocess Map", "Lexical-to-Source Mapping".

8. **`SourceMapService` and generated spans.** [x]
   - Define `MappedSourceRange` (a primary `SourceRange`, secondary anchors, and loaded-to-original `original_input` bytes) as the composite return type for loaded/lexical mapping.
   - Define the `SourceMapService` trait (`line_column`, `original_range_for_loaded`, `source_range_for_lexical`, `attach_generated_span`, `validate_range`) and a concrete implementation over the retained maps.
   - Add generated-span origins (`GeneratedSpanOrigin`) with a required reason.
   - Tests: each trait method on representative inputs; composite mapping returns primary plus secondary anchors; generated span without an origin is rejected.
   - Depends on: 5, 7. Spec: [source_map.md](./source_map.md) "Public API", "Generated Spans".

### Module: snapshot (`src/snapshot.rs`)

9. **Source-version record.** [x]
   - Add `pub mod snapshot;`. Define `SourceVersion` and `SourceOrigin` (`Disk` / `OpenBuffer{version}` / `Generated{generator}`).
   - Define `SnapshotError` with its spec variants (added as later tasks need them): invalid or non-normalizable source path, duplicate module path, missing dependency artifact, unsupported lockfile or toolchain metadata, stale open-buffer version, unknown snapshot id, lease release mismatch.
   - Provide the canonical sort key (package id, module path, normalized path, source hash).
   - Tests: deterministic ordering by canonical key independent of insertion order.
   - Depends on: 1, 4. Spec: [snapshot.md](./snapshot.md) "Source Version".

10. **Build snapshot identity.** [x]
    - Define `BuildSnapshot` and `SnapshotInput`; compute content-derived `BuildSnapshotId` from canonical input (sorted source/dependency summaries, lockfile hash, toolchain, verifier-config hash), excluding session-local ids/timestamps.
    - Handoff from task 9: source-version entries that compare equal by the canonical key (package id, module path, normalized path, source hash) must not make snapshot hashing insertion-order dependent. If such duplicates can reach hashing, encode them deterministically; preferably have task 11 validation reject duplicate source-version identities before hashing.
    - Tests: identical canonical inputs ⇒ identical id; source/dependency/config change ⇒ different id; session-local ids absent from the hash.
    - Depends on: 2, 9. Spec: [snapshot.md](./snapshot.md) "Snapshot Identity".

11. **Snapshot registry, creation, and freshness.** [x]
    - Define `SnapshotRegistry` with `create_snapshot`, `get`, and `is_current_for_request`.
    - Follow the task-11 boundary: `create_snapshot` accepts already-loaded `SourceVersion` records from the source-loading layer, validates the creation input, hashes the id, inserts the snapshot, and returns it together with an active-build `SnapshotLease`. The spec signature is updated to `Result<(BuildSnapshot, SnapshotLease), SnapshotError>`. (This resolves the lease-at-creation question in favor of the spec: the registry returns the active-build lease rather than relying on the caller to acquire one.)
    - Introduce here the minimal `SnapshotLease` handle and the dependency-free `RetentionReason` enum that it carries (shared with the `retention` module; the active-build lease uses `RetentionReason::ActiveBuild`). Full lease accounting lands in task 12; retention (task 17) reuses this `RetentionReason` rather than redefining it.
    - Handoff from task 9/10: reject duplicate source-version identities whose canonical keys are equal before snapshot hashing, so creation cannot accept insertion-order-sensitive duplicate records.
    - Tests: created snapshot is retrievable and returns an active-build lease; stale id is rejected by freshness; older snapshot is not reported as current; duplicate module path is rejected; path-normalized duplicate source identities are rejected through the source-version canonical key; missing dependency artifact/content fingerprint is rejected; unsupported lockfile/toolchain metadata is rejected; structurally invalid open-buffer versions are rejected. True expected-vs-actual open-buffer staleness is checked by the source-loading task that has request metadata.
    - Depends on: 3, 10. Spec: [snapshot.md](./snapshot.md) "Snapshot Creation", "Freshness Check", "Error Handling".

12. **Snapshot lease accounting.** [x]
    - Complete `SnapshotLease` with `acquire_lease`/`release_lease` on the registry, tracking lease counts per `RetentionReason` (from task 11); still no collection policy (that is retention, task 17-18).
    - Tests: acquire/release adjusts counts; releasing the active-build lease from task 11 is accounted; unknown snapshot id and lease release mismatch surface `SnapshotError`.
    - Depends on: 11. Spec: [snapshot.md](./snapshot.md) "Snapshot Lease".

13. **Snapshot construction API hardening.** [x]
    - Decide whether direct unchecked constructors (`BuildSnapshot::from_input`, `SnapshotInput::build_snapshot`, and `SnapshotInput::build_snapshot_id`) should remain public, become crate-private, or be renamed/documented as identity-only unchecked helpers.
    - Keep `SnapshotRegistry::create_snapshot` as the validated public creation path for registry snapshots.
    - If direct construction remains useful for identity tests or tooling, make the unchecked semantics explicit so downstream crates do not bypass creation validation by accident.
    - Tests: invalid `SnapshotInput` cannot produce a published/registry snapshot through the validated API; direct unchecked construction is either unavailable publicly or explicitly documented and tested as identity-only.
    - Depends on: 12. Spec: [snapshot.md](./snapshot.md) "Snapshot Creation", "Error Handling".

### Module: source (`src/source.rs`)

14. **Loaded-source types and loader surface.** [ ]
    - Define `SourceInput`, `SourceOriginInput` (the source-loading input variants carrying `path` / `uri,version,text` / generator text+anchor), `LoadedSource`, and the `SourceLoader` trait; `load` takes the target `BuildSnapshotId` before `SourceInput` so it can request a snapshot-scoped `SourceId`; reuse snapshot's `SourceOrigin` (task 9) for `LoadedSource.origin` instead of redefining it; implement `hash_text` and `normalize_path` (reuse existing `normalize_source_path`).
    - Define `SourceLoadError` with its spec variants: source path outside package root, unsupported file extension, invalid UTF-8, unreadable source file, duplicate module path, stale LSP document version, open-buffer URI that cannot be mapped to a package source, generated source without required generator metadata, source id allocation failure from `SessionIdAllocator`.
    - Tests: `source_hash` excludes absolute paths/document versions; identical text in different origins shares the hash.
    - Depends on: 1, 4, 6, 9. Spec: [source.md](./source.md) "Public API", "Loaded Source".

15. **Disk source loading.** [ ]
    - Implement disk loading: path normalization + package-root enforcement, read bytes, UTF-8 validation (no lossy `U+FFFD`), leading-BOM strip, CRLF→LF normalization, `source_hash`, `LineMap`, and `LoadingMap` emission.
    - Only the leading UTF-8 BOM is an encoding signature; a non-leading `U+FEFF` stays in loaded text. Only CRLF pairs normalize to LF; a lone `\r` is preserved (not treated as a platform newline).
    - Tests: invalid UTF-8 rejected before line-map; unsupported extension rejected; leading BOM → loading map `0`↔`3`; non-leading `U+FEFF` preserved in loaded text; CRLF normalized while lone `\r` is preserved; path outside root rejected.
    - Depends on: 14. Spec: [source.md](./source.md) "Disk Source Loading".

16. **Open-buffer and generated loading.** [ ]
    - Implement open-buffer loading (LSP document-version validation, URI→package path, BOM strip, CRLF normalize, loading map back to editor offsets) and generated-source loading (generator metadata + anchor).
    - Tests: open-buffer overrides disk only for the matching version; stale version rejected; the open-buffer loading map relates loaded-text offsets back to editor-provided text byte offsets (before LSP UTF-16 conversion); unmappable open-buffer URI rejected; generated source without metadata rejected.
    - Depends on: 15. Spec: [source.md](./source.md) "Open-Buffer Source Loading", "Generated Source Loading".

### Module: retention (`src/retention.rs`)

17. **Retention manager and leases.** [ ]
    - Add `pub mod retention;`. Define `RetentionManager`, `RetainSnapshotInput`, `RetainGuard`, `RetainOwner`, and `retain_snapshot`/`release` with reference counting; reuse `RetentionReason` (defined in task 11) rather than redefining it.
    - Define `RetentionError` with its spec variants: unknown snapshot id, unknown or already-released lease id, lease snapshot mismatch, invalid owner/reason combination, attempt to mark a missing snapshot as current, collection blocked by inconsistent retention state.
    - Retaining a stale snapshot is allowed for diagnostic / explanation / LSP stale-display / IR-output reasons, but must not make the snapshot current.
    - Tests: active lease prevents collection eligibility; duplicate release is reported without underflow; an invalid owner/reason combination is rejected; a stale-snapshot retain succeeds without marking it current.
    - Depends on: 13. Spec: [retention.md](./retention.md) "Retain", "Release", "Error Handling".

18. **Current marks and collection.** [ ]
    - Add `mark_current`/`unmark_current`, `collect`, and `CollectionSummary`; implement the collection policy (no lease, no current mark, no retained map/explanation, IR phase-output lease released).
    - `CollectionSummary` reports counts for snapshots scanned/collected, sources and maps released, snapshots skipped for current marks, snapshots skipped for live leases, and stale/mismatched-lease diagnostics.
    - Tests: current mark prevents collection without other leases; releasing the final lease collects; a phase-output lease blocks collection until released; marking a missing snapshot as current surfaces `RetentionError`; `CollectionSummary` reports skipped-for-current and skipped-for-lease counters and stale-lease diagnostics; collection does not delete artifacts/cache.
    - Depends on: 17. Spec: [retention.md](./retention.md) "Collection", "Current Marks", "Collection Summary".

## Cross-Cutting Follow-ups

- [ ] Keep `ja/` module specs in sync if any API changes during implementation.
- [ ] Determinism property tests across the crate: identical canonical inputs ⇒ identical `BuildSnapshotId` and identical source-range conversions, independent of scheduling order.
- [ ] Snapshot lease accounting hardening: decide whether `SnapshotRegistry::acquire_lease` should allocate lease ids outside the registry mutex, matching `create_snapshot` and reducing the blast radius of heavy or custom allocators.
- [ ] Snapshot lease accounting hardening: decide whether to add a defensive duplicate-lease-id check or debug assertion in the registry state, even though `SessionIdAllocator` is expected to issue unique lease ids.

## Suggested Verification

After each task, run:

```text
cargo test -p mizar-session
cargo test -p mizar-test
cargo clippy -p mizar-session --all-targets -- -D warnings
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
