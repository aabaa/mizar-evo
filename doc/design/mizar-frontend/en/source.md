# Module: source

> Canonical language: English. Japanese companion: [../ja/source.md](../ja/source.md).

Status: planned.

## Purpose

This module implements the frontend pipeline Step 1 (source loading) and
produces the `SourceUnit` consumed by preprocessing and every later step.

It bridges `mizar-session` source identity into a frontend-local loaded record:
it reads source bytes, validates UTF-8, derives the module path, builds the
`LineMap`, and preserves the `LoadingMap` from loaded-text offsets back to the
original input. It does not preprocess comments, tokenize, parse, resolve
imports, or assign `SourceId` / `SourceVersion` identity by itself.

`mizar-session` owns source identity, source hashes, and snapshot membership.
`mizar-frontend` consumes a `mizar_session::LoadedSource` and reshapes it into
the `SourceUnit` defined by
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Step 1: Load SourceUnit". This module never re-hashes or re-normalizes text
that `mizar-session` already loaded.

## Public API

```rust
pub struct SourceUnit {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub file_path: PathBuf,
    pub source_text: Arc<str>,
    pub source_hash: Hash,
    pub line_map: LineMap,
    pub loading_map: Option<LoadingMap>,
    pub origin: SourceOrigin,
}

pub struct SourceUnitRequest {
    pub snapshot: BuildSnapshotId,
    pub input: SourceInput,
}

pub trait SourceUnitLoader {
    fn load_source_unit(
        &self,
        request: SourceUnitRequest,
        ids: &dyn SessionIdAllocator,
    ) -> Result<SourceUnit, SourceLoadError>;
}

pub struct FrontendSourceLoader<L: SourceLoader> { /* session loader */ }

impl<L: SourceLoader> FrontendSourceLoader<L> {
    pub fn new(loader: L) -> Self;
}

impl<L: SourceLoader> SourceUnitLoader for FrontendSourceLoader<L> { /* ... */ }

pub fn source_unit_from_loaded(loaded: LoadedSource, file_path: PathBuf) -> SourceUnit;
```

`SourceUnit` mirrors the architecture interface and adds `origin` so later
phases (preprocessing diagnostics, LSP overlays) can distinguish disk,
open-buffer, and generated text without re-reading session records.

`FrontendSourceLoader` wraps any `mizar_session::SourceLoader` (for example
`DiskSourceLoader`). It forwards the request to the session loader, then calls
`source_unit_from_loaded` to project the resulting `LoadedSource` into a
`SourceUnit`. The frontend does not define its own path normalization, hashing,
or BOM/newline rules; those remain in `mizar-session`.

## Dependencies

- Internal: `span_bridge` (registers the `LineMap` / `LoadingMap` for later
  coordinate conversion), `preprocess` (consumes `SourceUnit`).
- External: `mizar-session` (`SourceLoader`, `LoadedSource`, `SourceInput`,
  `SourceId`, `LineMap`, `LoadingMap`, `SourceOrigin`, `SessionIdAllocator`,
  `BuildSnapshotId`), filesystem and package metadata via the session loader.

This module is consumed by the orchestration coordinator and by LSP/document
consumers that need a `SourceUnit` for a single file.

## Data Structures

### SourceUnit

`SourceUnit` is an immutable, source-faithful loaded record for one `.miz`
file. `source_text` is the validated, source-loading-normalized text exactly as
`mizar-session` produced it. `source_hash`, `line_map`, and `loading_map` are
the session values, copied without recomputation. `file_path` is a local
display path for diagnostics; published identity uses the normalized path
carried inside the session records, not `file_path`.

`SourceUnit` is the cache-key anchor for Step 1 in
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Incrementality": its key is file path plus source bytes, both already
captured by the session `SourceVersion`.

### Loading Map Preservation

`loading_map` is `Some` only when source loading changed offsets (leading-BOM
strip or CRLF→LF normalization for disk/open-buffer text). It is `None` for
identity loads and for generated text. The frontend never edits the map; it
forwards the session map so `span_bridge` can translate loaded-text offsets back
to original input offsets for diagnostics and LSP positions.

## Algorithm / Logic

### Load a single SourceUnit

1. Build a `SourceUnitRequest` for the target `BuildSnapshotId` and `SourceInput`
   (package id, module path, normalized path, edition, origin input).
2. Delegate to the wrapped `mizar_session::SourceLoader::load`, which performs
   path normalization, package-root enforcement, byte reading, UTF-8 validation,
   leading-BOM stripping, CRLF→LF normalization, source hashing, `LineMap`
   construction, and `LoadingMap` emission.
3. Project the returned `LoadedSource` into a `SourceUnit`, preserving
   `source_id`, `package_id`, `module_path`, text, hash, line map, loading map,
   and origin unchanged.
4. Record the loaded `LineMap` / `LoadingMap` with the `span_bridge` registry
   under the `SourceId` so later phases can convert spans.
5. Return the `SourceUnit`.

The frontend performs no encoding work of its own here. Code-region ASCII
validation is deferred to preprocessing; this module only carries the
session-validated encoding and identity forward.

## Error Handling

Loading surfaces `mizar_session::SourceLoadError` unchanged (source path outside
package root, unsupported extension, invalid UTF-8, unreadable file, stale
open-buffer version, unmappable open-buffer URI, generated source without
metadata, source-id allocation failure, and the path-normalization variants).
The frontend converts these into file-level frontend diagnostics in the
orchestration layer; it does not invent new error categories for conditions that
`mizar-session` already classifies.

A load failure produces no `SourceUnit`; orchestration reports the diagnostic
and stops the pipeline for that file before preprocessing.

## Tests

Key scenarios:

- a disk `LoadedSource` projects to a `SourceUnit` with identical
  `source_id`, `source_hash`, `line_map`, and `loading_map` (no recomputation);
- a BOM-stripped / CRLF-normalized disk source carries a `Some(loading_map)`
  into the `SourceUnit`;
- an identity load (no offset change) carries `loading_map = None`;
- an open-buffer `SourceUnit` records `SourceOrigin::OpenBuffer` and the
  validated document version;
- a session `SourceLoadError` (invalid UTF-8, path outside root) is propagated
  without being reclassified.

## Constraints and Assumptions

- This module does not read or normalize bytes itself; it delegates to
  `mizar-session` and only reshapes the result.
- `source_hash`, `line_map`, and `loading_map` are never recomputed by the
  frontend.
- `file_path` is local diagnostic metadata, excluded from published identity.
- A `SourceUnit` is immutable after construction and may be retained by snapshot
  leases, LSP views, or downstream phase outputs.
