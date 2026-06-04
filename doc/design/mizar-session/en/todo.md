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
| source_map | [source_map.md](./source_map.md) | `src/source_map.rs` | [x] |
| snapshot | [snapshot.md](./snapshot.md) | `src/snapshot.rs` | [x] |
| source | [source.md](./source.md) | `src/source.rs` | [x] |
| retention | [retention.md](./retention.md) | `src/retention.rs` | [x] |

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

14. **Loaded-source types and loader surface.** [x]
    - Define `SourceInput`, `SourceOriginInput` (the source-loading input variants carrying `path` / `uri,version,text` / generator text+anchor), `LoadedSource`, and the `SourceLoader` trait; `load` takes the target `BuildSnapshotId` before `SourceInput` so it can request a snapshot-scoped `SourceId`; reuse snapshot's `SourceOrigin` (task 9) for `LoadedSource.origin` instead of redefining it; implement `hash_text` and `normalize_path` (reuse existing `normalize_source_path`).
    - Define `SourceLoadError` with its spec variants: source path outside package root, unsupported file extension, invalid UTF-8, unreadable source file, duplicate module path, stale LSP document version, open-buffer URI that cannot be mapped to a package source, generated source without required generator metadata, source id allocation failure from `SessionIdAllocator`.
    - Tests: `source_hash` excludes absolute paths/document versions; identical text in different origins shares the hash.
    - Depends on: 1, 4, 6, 9. Spec: [source.md](./source.md) "Public API", "Loaded Source".

15. **Disk source loading.** [x]
    - Implement disk loading: path normalization + package-root enforcement, read bytes, UTF-8 validation (no lossy `U+FFFD`), leading-BOM strip, CRLF→LF normalization, `source_hash`, `LineMap`, and `LoadingMap` emission.
    - Only the leading UTF-8 BOM is an encoding signature; a non-leading `U+FEFF` stays in loaded text. Only CRLF pairs normalize to LF; a lone `\r` is preserved (not treated as a platform newline).
    - Tests: invalid UTF-8 rejected before line-map; unsupported extension rejected; leading BOM → loading map `0`↔`3`; non-leading `U+FEFF` preserved in loaded text; CRLF normalized while lone `\r` is preserved; path outside root rejected.
    - Depends on: 14. Spec: [source.md](./source.md) "Disk Source Loading".

16. **Open-buffer and generated loading.** [x]
    - Implement open-buffer loading (LSP document-version validation, URI→package path, BOM strip, CRLF normalize, loading map back to editor offsets) and generated-source loading (generator metadata + anchor).
    - Tests: open-buffer overrides disk only for the matching version; stale version rejected; the open-buffer loading map relates loaded-text offsets back to editor-provided text byte offsets (before LSP UTF-16 conversion); unmappable open-buffer URI rejected; generated source without metadata rejected.
    - Depends on: 15. Spec: [source.md](./source.md) "Open-Buffer Source Loading", "Generated Source Loading".

### Module: retention (`src/retention.rs`)

17. **Retention manager and leases.** [x]
    - Add `pub mod retention;`. Define `RetentionManager`, `RetainSnapshotInput`, `RetainGuard`, `RetainOwner`, and `retain_snapshot`/`release` with reference counting; reuse `RetentionReason` (defined in task 11) rather than redefining it.
    - Bridge existing `SnapshotLease` handles from the snapshot registry into retention accounting with `retain_existing_lease`, so the active-build lease returned by `create_snapshot` can block retention collection eligibility without allocating a duplicate lease id.
    - Define `RetentionError` with its spec variants: unknown snapshot id, unknown or already-released lease id, lease snapshot mismatch, invalid owner/reason combination, attempt to mark a missing snapshot as current, collection blocked by inconsistent retention state.
    - Retaining a stale snapshot is allowed for diagnostic / explanation / LSP stale-display / IR-output reasons, but must not make the snapshot current.
    - Tests: active lease prevents collection eligibility; duplicate release is reported without underflow; an invalid owner/reason combination is rejected; a stale-snapshot retain succeeds without marking it current.
    - Depends on: 13. Spec: [retention.md](./retention.md) "Retain", "Release", "Error Handling".

18. **Current marks and collection.** [x]
    - Add `mark_current`/`unmark_current`, `record_retained_resources`, `collect`, and `CollectionSummary`; implement the collection policy (no lease, no current mark, no retained map/explanation/LSP/IR lease).
    - `CollectionSummary` reports counts for snapshots scanned/collected, sources and maps released, snapshots skipped for current marks, snapshots skipped for live leases, and stale/mismatched-lease diagnostics.
    - Tests: current mark prevents collection without other leases; releasing the final lease collects; a phase-output lease blocks collection until released; marking a missing snapshot as current surfaces `RetentionError`; `CollectionSummary` reports skipped-for-current and skipped-for-lease counters and stale-lease diagnostics; collection does not delete artifacts/cache.
    - Depends on: 17. Spec: [retention.md](./retention.md) "Collection", "Current Marks", "Collection Summary".

### Module-wide maintenance before cross-cutting follow-ups

19. **Implementation refactoring pass.** [x]
    - Review `ids`, `source_map`, `snapshot`, `source`, and `retention` now that the first implementation pass is complete.
    - Keep public APIs and behavior stable unless the refactor exposes a clear bug or spec mismatch.
    - Prefer small local extractions, shared test fixtures, and naming cleanup over broad rewrites.
    - Remove task-era duplication only when it simplifies the code without obscuring the spec mapping.
    - Tests: update only where behavior-preserving refactors need better assertions; keep existing module tests green.
    - Depends on: 18. Spec: all mizar-session module specs.

20. **Source/spec correspondence audit.** [x]
    - Build a lightweight traceability check from each public API, error variant, and task requirement in `ids.md`, `source_map.md`, `snapshot.md`, `source.md`, and `retention.md` to the implementing source/tests.
    - Record any missing implementation, stale spec text, underspecified behavior, or missing tests as follow-up tasks rather than mixing broad changes into the audit.
    - Check the English canonical specs first, then verify that Japanese companion specs carry the same API and behavioral commitments.
    - Tests: no new product tests expected unless the audit finds a small, safe gap; run the standard verification commands after any edits.
    - Depends on: 19. Spec: all mizar-session module specs and this TODO.
    - Audit result: implementation and unit-test coverage are broadly traceable for
      tasks 1-19; remaining gaps were recorded as follow-up tasks 25-28.

## Cross-Cutting Follow-up Tasks

21. **Bilingual documentation synchronization audit.** [x]
    - Compare every English canonical document under `doc/design/mizar-session/en/` with its Japanese companion under `doc/design/mizar-session/ja/`.
    - Synchronize API lists, task statuses, terminology, and links introduced during tasks 1-20.
    - If a Japanese companion cannot be fully synchronized in the same change, mark the gap explicitly and link back to the canonical English section.
    - Tests: documentation-only; run formatting or link checks if the repository has an established command.
    - Depends on: 20. Spec: repository documentation policy.
    - Audit result: English canonical documents and Japanese companions are
      synchronized for module statuses, public API/error lists, task statuses,
      terminology introduced by tasks 1-20, and companion-local links. No
      unsynchronized Japanese companion gap remains.

22. **Determinism property tests.** [x]
    - Add crate-level determinism coverage for identical canonical inputs producing identical `BuildSnapshotId` values independent of insertion order or scheduling-like construction order.
    - Add source-range conversion determinism checks for equivalent retained line/loading/preprocess maps.
    - Keep the tests focused on deterministic public behavior rather than implementation details.
    - Tests: add the property/regression tests and run `cargo test -p mizar-session`.
    - Depends on: 20. Spec: [ids.md](./ids.md), [snapshot.md](./snapshot.md), [source_map.md](./source_map.md).
    - Result: added public integration coverage in
      `crates/mizar-session/tests/determinism.rs` for registry-created
      `BuildSnapshotId` equality across source/dependency insertion
      permutations, scheduling-like source-id allocation orders, and equivalent
      retained source-map conversion orders.

23. **Snapshot lease allocation mutex hardening.** [x]
    - Decide whether `SnapshotRegistry::acquire_lease` should allocate lease ids outside the registry mutex, matching `create_snapshot`.
    - If changed, preserve existing lease-count behavior and make allocator failure leave registry state untouched.
    - If not changed, document the rationale in [snapshot.md](./snapshot.md) or this TODO.
    - Decision: `acquire_lease` now rejects unknown snapshots before allocation, allocates lease ids outside the registry mutex for known snapshots, and records the lease under the mutex without adding duplicate-id defense here.
    - Tests: allocator failure does not change registry state; repeated lease acquisition remains unique and counted by reason.
    - Depends on: 20. Spec: [snapshot.md](./snapshot.md) "Snapshot Lease".

24. **Snapshot lease duplicate-id defense.** [x]
    - Decide whether to add a defensive duplicate-lease-id check or debug assertion in `SnapshotRegistry` state, even though `SessionIdAllocator` is expected to issue unique lease ids.
    - If implemented, surface duplicate allocation as an internal allocation/registry error without corrupting lease counts or current snapshot state.
    - If only a debug assertion is kept, document why that is sufficient for the allocator contract.
    - Tests: custom allocator duplicate id scenario if the behavior is observable outside debug assertions.
    - Decision: add an observable defensive duplicate-live-lease-id check in `SnapshotRegistryState`, because `SessionIdAllocator` is a public trait and a release-build registry must not let a custom allocator overwrite the live lease map while incrementing counts. Duplicate allocation is reported as `SnapshotError::DuplicateLeaseIdAllocation` before snapshot records, current marks, live leases, or lease counts are mutated.
    - Tests: duplicate ids returned during `acquire_lease` and `create_snapshot` leave lease counts, live leases, snapshot records, and current request state unchanged.
    - Depends on: 23. Spec: [snapshot.md](./snapshot.md) "Snapshot Lease", [ids.md](./ids.md) "Allocator-Issued Id Construction".

25. **Public API blocks and source-map error-surface spec sync.** [x]
    - Decide whether implemented public helpers and aliases that are absent from
      the current public API blocks are intentional public API or should be
      narrowed. Audit at least `Hash::{from_bytes, as_bytes}`,
      `LineMap::source`, `TextRange::{new, try_new, len, is_empty}`,
      `DocumentUri`, `LspDocumentVersion`, and `NormalizedPath::as_str`.
    - Synchronize the English and Japanese public API blocks and error lists with
      the decision, including the implemented `SourceMapError::ReversedRange`
      variant when reversed ranges remain a public error.
    - Keep the existing validation behavior stable unless the API decision
      explicitly narrows it.
    - Tests: documentation-only if the surface is documented as-is; otherwise
      adjust unit or compile-fail coverage for the changed public surface.
    - Depends on: 20. Spec: [ids.md](./ids.md), [source.md](./source.md), [source_map.md](./source_map.md).
    - Decision: keep the existing helpers and aliases public and document them as
      intentional API. No validation behavior was narrowed. `Hash` byte helpers
      remain low-level canonical-byte accessors rather than a standalone
      published serialization format; `LineMap::source`, `TextRange` helpers,
      `DocumentUri`, `LspDocumentVersion`, and `NormalizedPath::as_str` remain
      public. `SourceMapError::ReversedRange` is part of the public error
      surface for manually constructed reversed `SourceRange`/`TextRange`
      values.
    - Test policy: documentation-only because the public Rust surface and
      validation behavior were preserved.

26. **Source and snapshot source-identity validation boundary.** [x]
    - Decide where blank or otherwise invalid `WorkspaceRoot`, `PackageId`,
      `ModulePath`, `Edition`, and generated-source metadata are rejected:
      constructors, source loading, snapshot creation, or an upstream build-plan
      layer.
    - Clarify whether `SourceLoadError::DuplicateModulePath` is emitted by a
      future source-loading aggregator or whether duplicate module paths are
      solely a `SnapshotRegistry::create_snapshot` validation responsibility.
    - If source or snapshot validation is strengthened, preserve deterministic
      snapshot hashing by rejecting invalid source identities before hashing.
    - Tests: add focused cases for the chosen boundary, especially blank
      package/module/edition values and duplicate module paths at the layer that
      owns them.
    - Depends on: 20. Spec: [source.md](./source.md), [snapshot.md](./snapshot.md).
    - Decision: constructors for `WorkspaceRoot`, `PackageId`, `ModulePath`,
      `Edition`, and `GeneratedSourceKind` stay infallible string wrappers.
      Upstream build planning owns normalized workspace roots, package ids,
      editions, and canonical module discovery; source loading owns source-path,
      text, open-buffer freshness, and generated-source metadata validation for
      loaded inputs; `SnapshotRegistry::create_snapshot` is the final pre-hash
      registry boundary and rejects blank workspace roots, blank or
      whitespace-containing package ids, invalid module path components
      (including reserved words), blank editions, blank generated-source metadata
      in direct `SourceVersion` input, duplicate source-version identities, and
      duplicate module paths before hashing, lease allocation, or registry
      insertion. Nonblank package-id spelling stays an upstream build-plan
      concern until the package-name specs are aligned.
    - Decision: `SourceLoadError::DuplicateModulePath` remains reserved for a
      future source-loading aggregator over a build plan. A single
      `SourceLoader::load` call does not emit it, and snapshot creation keeps the
      required whole-snapshot duplicate module path check.
    - Test policy: focused snapshot unit tests cover blank workspace root,
      package id, module path, edition, direct generated-source metadata, and the
      existing duplicate module path/source identity pre-hash checks; source tests
      keep the generated-source metadata rejection before source-id allocation
      and reject reserved-word namespace components.
    - Follow-up: resolve the English canonical package-name spelling conflict
      between `doc/spec/en/23.package_management_and_build_system.md`
      (`[a-z][a-z0-9-]*`) and `doc/spec/en/12.modules_and_namespaces.md`
      (`snake_case`), then sync the Japanese companions, including the malformed
      Japanese package-name table row in `doc/spec/ja/23.package_management_and_build_system.md`.

27. **Generated-source normalization policy.** [x]
    - Decide and document whether generated source text is preserved byte-for-byte
      after UTF-8 validation, or whether source-loading normalization such as
      leading-BOM stripping and CRLF-to-LF conversion applies to generated input.
    - The current implementation preserves generated text exactly and emits no
      `LoadingMap`; if that remains the policy, make it explicit in the English
      and Japanese `source.md` generated-source sections.
    - If generated input should instead be normalized like disk/open-buffer text,
      update implementation, source-map expectations, and source-hash tests
      together.
    - Tests: add or update a focused generated-source BOM/CRLF case for the chosen
      behavior.
    - Depends on: 20. Spec: [source.md](./source.md), [source_map.md](./source_map.md).
    - Decision: keep generated-source loading byte-for-byte. Generated input
      enters the API as already-valid UTF-8 `Arc<str>`, and the loader preserves
      accepted text exactly: a leading `U+FEFF` is not treated as an encoding
      signature, CRLF pairs are not converted to LF, and lone `\r` is unchanged.
      `source_hash` and `LineMap` are computed over that exact generated text.
      Generated loading emits no `LoadingMap`; generated locations are recovered
      through `LoadedSource.generated_anchor`, `SourceAnchor::Generated`, and
      `GeneratedSpanOrigin`. Generators that need package-authored text
      normalization must normalize their own output before constructing
      `SourceOriginInput::Generated`.
    - Test policy: `source.rs` includes a focused generated-source leading
      `U+FEFF` plus CRLF case that preserves the text byte-for-byte, hashes the
      exact text rather than the normalized spelling, and emits no `LoadingMap`.
    - Follow-up: none for this policy. `LoadingOrigin::Generated` remains usable
      for custom identity maps when a caller needs retained service-level
      loaded-to-original conversion for generated text, but the default loader
      does not create one.

28. **Reserved and diagnostic-only error variant traceability.** [x]
    - Classify public error variants that are currently reserved, custom-loader
      only, or diagnostic-summary-only, including
      `IdError::UnknownSnapshotRegistry`,
      `SnapshotError::InvalidSourcePath`,
      `SourceLoadError::UnsupportedSourceOrigin`, and the retention inconsistent
      state surfaces.
    - For each variant, either add a public observable path with focused tests or
      document that it is reserved/internal and why it remains in the public
      non-exhaustive enum.
    - Clarify in `retention.md` that collection-time stale/mismatched lease state
      is reported through `CollectionSummary::lease_diagnostics`, while
      `RetentionError::CollectionBlockedByInconsistentRetentionState` is used by
      retain/release/allocation paths.
    - Tests: add only the cases needed to make newly observable paths explicit;
      documentation-only if the variants are kept as reserved/internal.
    - Depends on: 20. Spec: [ids.md](./ids.md), [snapshot.md](./snapshot.md), [source.md](./source.md), [retention.md](./retention.md).
    - Decision: no new public observable paths were added. `IdError::UnknownSnapshotRegistry`
      remains reserved for registry-aware custom allocators and is not emitted
      by `InMemorySessionIdAllocator`. `SnapshotError::InvalidSourcePath`
      remains reserved/internal for future snapshot construction or
      revalidation paths because public `create_snapshot` consumes
      already-normalized `SourceVersion` records.
      `SourceLoadError::UnsupportedSourceOrigin` is custom-loader-only because
      `DiskSourceLoader` supports every current `SourceOriginInput` variant.
      Retention collection-time stale/mismatched lease state is
      diagnostic-summary-only via `CollectionSummary::lease_diagnostics`, while
      `RetentionError::CollectionBlockedByInconsistentRetentionState` is kept
      for retain/release/allocation inconsistencies. Existing allocation-side
      tests already covered that error surface; a focused release-time missing
      live-lease-count test was added for the documented release path.

29. **Durable lint enforcement.** [x]
    - Make the clippy/rustc lint gate from [Suggested Verification](#suggested-verification)
      durable in-tree instead of relying on each contributor running it by hand;
      the workspace currently has no `[workspace.lints]` table and no crate-level
      lint attributes.
    - Prefer a workspace `[workspace.lints]` table (rustc + clippy groups) with
      `lints.workspace = true` in `crates/mizar-session/Cargo.toml`, so plain
      `cargo build`/`cargo test` surface the same denials as the standalone clippy
      command. If a crate-local `#![deny(...)]`/`#![warn(...)]` policy is chosen
      instead, document why.
    - Decide the baseline severity (at least deny `warnings` and `clippy::all`;
      consider enabling `clippy::pedantic` selectively) and record any intentional
      `allow` exceptions with a rationale next to the `allow`.
    - Keep the existing public API and behavior unchanged; this task only adds
      lint configuration plus any mechanical fixes needed to reach a clean gate.
    - Tests: `cargo clippy -p mizar-session --all-targets -- -D warnings` passes
      with the gate active and `cargo test -p mizar-session` stays green.
    - Depends on: 20. Spec: this TODO "Suggested Verification".
    - Decision: added a workspace `[workspace.lints]` baseline that denies rustc
      `warnings` and `clippy::all`, with `crates/mizar-session/Cargo.toml`
      opting in via `lints.workspace = true`. `mizar-lexer`, `mizar-lsp`, and
      `mizar-test` were not opted in so this task does not broaden enforcement
      beyond `mizar-session`; future crates can adopt the same policy by adding
      the manifest opt-in.
    - Decision: `clippy::pedantic` is not part of the baseline. A trial run with
      `-W clippy::pedantic -D warnings` produced broad API-doc/style churn
      (`similar_names`, missing `# Errors`/`# Panics`, `must_use_candidate`, and
      related findings) outside this task's lint-gate scope. No new `allow`
      exceptions were needed; stale `allow(dead_code)` attributes on the
      already-wired snapshot ID derivation helpers were removed instead.

30. **Oversized module file split.** [x]
    - Reduce the largest source files (`snapshot.rs`, `source_map.rs`, and
      `source.rs` are each roughly 2.3k-3.4k lines including tests) by extracting
      cohesive submodules, without changing the public API surface re-exported
      from `lib.rs` or the "Public API" blocks in the module specs.
    - Prefer moving large `#[cfg(test)]` blocks into sibling test modules or
      `tests/`-style files, and separating clearly independent concerns (for
      example snapshot identity vs. lease accounting vs. registry) into child
      modules under the same public module path.
    - Keep `mod` privacy and re-exports stable so downstream crates and the spec
      "Public API" blocks observe no change.
    - Tests: behavior-preserving; keep all module and doctests green and re-run the
      standard verification commands.
    - Depends on: 19, 20. Spec: all mizar-session module specs.
    - Decision: split the large `#[cfg(test)]` blocks for `snapshot`, `source_map`,
      and `source` into private sibling test modules at
      `src/snapshot/tests.rs`, `src/source_map/tests.rs`, and `src/source/tests.rs`.
      This keeps the public module paths and `lib.rs` re-exports unchanged while
      making the implementation files focused on production code.

31. **Open-buffer source-loading error specificity.** [ ]
    - Problem: `DiskSourceLoader::normalize_open_buffer_uri` collapses every
      `normalize_source_path` failure into `SourceLoadError::UnmappedOpenBufferUri`
      (`src/source.rs`), so an open-buffer URI that does resolve to a package
      location but fails a later path check (unsupported extension, non-canonical
      alias/spelling, invalid namespace component) is reported as an unmappable
      URI instead of the specific category disk loading uses.
    - Decide whether open-buffer loading should distinguish "URI cannot be mapped
      to any package source at all" (non-`file://` scheme, undecodable percent
      encoding, path outside the package root) from path errors that disk loading
      already reports specifically through `SourceLoadError::from_source_path_error`
      (`UnsupportedFileExtension`, `InvalidSourcePath`).
    - If reclassified, reuse the existing disk mapping so disk and open-buffer
      origins share consistent error categories, and keep `UnmappedOpenBufferUri`
      only for URIs that cannot become a package-relative path.
    - Keep the existing validation behavior (still reject the same inputs); only
      the reported error variant changes.
    - Synchronize the English and Japanese `source.md` "Open-Buffer Source
      Loading" and "Error Handling" sections with the decision.
    - Tests: an open-buffer URI under the package `src/` root with a non-`.miz`
      extension reports the unsupported-extension category; a non-`file://` or
      undecodable URI still reports `UnmappedOpenBufferUri`.
    - Depends on: 20. Spec: [source.md](./source.md) "Open-Buffer Source Loading",
      "Error Handling".

32. **Missing-`src/`-root path error fidelity.** [ ]
    - Problem: `SourceLoadError::from_source_path_error` maps
      `SourcePathError::MissingSourceRoot` to
      `SourceLoadError::SourcePathOutsidePackageRoot` (`src/source.rs`), so a path
      that is inside the package root but not under the required `src/` source
      tree is reported as "outside package root", which misstates the condition.
    - Decide whether to surface the missing-`src/`-root case distinctly: either
      add a dedicated `SourceLoadError` variant (for example
      `SourcePathOutsideSourceRoot`) or carry `SourcePathError::MissingSourceRoot`
      through `SourceLoadError::InvalidSourcePath` so the message reflects the
      actual condition rather than reusing the package-root-boundary variant.
    - This touches the public non-exhaustive `SourceLoadError` enum and the
      `source.md` error list, which currently folds the `src/` requirement into
      "source path outside package root"; update the list and wording to match the
      decision.
    - Preserve the existing rejection behavior; only the error identity and
      message change.
    - Synchronize the English and Japanese `source.md` "Disk Source Loading"
      (step 2) and "Error Handling" sections with the decision.
    - Tests: a path inside the package root but outside `src/` reports the
      missing-source-root condition distinctly from a path that is genuinely
      outside the package root.
    - Depends on: 20. Spec: [source.md](./source.md) "Disk Source Loading",
      "Error Handling".

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
