# Module: source

> Canonical language: English. Japanese companion: [../ja/source.md](../ja/source.md).

Status: implemented.

## Purpose

This module implements the frontend pipeline Step 1 (source loading) and
produces the `SourceUnit` consumed by preprocessing and every later step.

It bridges `mizar-session` source identity into a frontend-local loaded record:
the wrapped session loader reads source bytes, validates UTF-8, derives source
metadata, builds the `LineMap`, and emits the optional `LoadingMap` from
loaded-text offsets back to the original input; this module preserves those
loaded values in `SourceUnit`. It does not preprocess comments, tokenize, parse,
resolve imports, or assign `SourceId` / `SourceVersion` identity by itself.

`mizar-session` owns source identity, source hashes, and snapshot membership.
`mizar-frontend` consumes a `mizar_session::LoadedSource` and reshapes it into
the `SourceUnit` defined by
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Step 1: Load SourceUnit". The loader bridge never re-hashes or re-normalizes text
that `mizar-session` already loaded.

## Public API

```rust
pub struct SourceUnit {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub edition: Edition,
    pub file_path: PathBuf,
    pub source_text: Arc<str>,
    pub source_hash: Hash,
    pub line_map: LineMap,
    pub loading_map: Option<LoadingMap>,
    pub origin: SourceOrigin,
    pub generated_anchor: Option<SourceAnchor>,
}

impl SourceUnit {
    pub fn canonical_disk_bytes(&self) -> Option<Vec<u8>>;
    pub fn from_canonical_disk_bytes(
        bytes: &[u8], source_id: SourceId, input: &SourceInput,
    ) -> Option<Self>;
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

pub fn source_unit_from_loaded(
    loaded: LoadedSource,
    file_path: PathBuf,
) -> SourceUnit;

pub fn register_source_unit(
    bridge: &mut SpanBridge,
    source: &SourceUnit,
) -> Result<(), SpanBridgeError>;
```

`SourceUnit` mirrors the architecture interface and adds the session identity
metadata later frontend phases need without re-reading session records:
`normalized_path`, `edition`, `origin`, and `generated_anchor`.

`FrontendSourceLoader` wraps any `mizar_session::SourceLoader` (for example
`DiskSourceLoader`). It forwards the request to the session loader, then calls
`source_unit_from_loaded` to project the resulting `LoadedSource` into a
`SourceUnit`. The frontend does not define its own path normalization, hashing,
or BOM/newline rules; those remain in `mizar-session`.

`LoadedSource` does not store a filesystem path. The caller supplies `file_path`
as local diagnostic metadata: disk and open-buffer loaders derive it from the
request/origin URI when one exists, while generated sources may use the
`normalized_path` or a synthetic display path derived from `generated_anchor`.
For `file://` open-buffer URIs, the frontend decodes the URI path for this
display path; if the URI cannot be decoded as a local file path, it falls back to
the request's `normalized_path`. This value is never part of published source
identity or cache keys.

`register_source_unit` records the source's `LineMap` and optional `LoadingMap`
with the mutable `SpanBridge`. Loading itself stays independent of bridge
registration so tests and callers can project a `LoadedSource` without also
mutating source-map state.

## Dependencies

- Internal: `span_bridge` (registers the `LineMap` / `LoadingMap` for later
  coordinate conversion), `preprocess` (consumes `SourceUnit`).
- External: `mizar-session` (`SourceLoader`, `LoadedSource`, `SourceInput`,
  `SourceId`, `LineMap`, `LoadingMap`, `SourceOrigin`, `SourceAnchor`,
  `NormalizedPath`, `Edition`, `SessionIdAllocator`, `BuildSnapshotId`),
  filesystem and package metadata via the session loader.

This module is consumed by the orchestration coordinator and by LSP/document
consumers that need a `SourceUnit` for a single file.

## Data Structures

### SourceUnit

`SourceUnit` is a source-faithful loaded record for one `.miz` file or
generated source fragment. Callers treat a constructed `SourceUnit` as immutable
pipeline input. `source_text` is the validated,
source-loading-normalized text exactly as `mizar-session` produced it.
On loading, `source_hash`, `line_map`, `loading_map`, `normalized_path`,
`edition`, `origin`, and `generated_anchor` are the session values, copied
without recomputation. `file_path` is a local display path for diagnostics; published
identity uses `normalized_path`, not `file_path`.

`SourceUnit` is the loaded content anchor for Step 1 in
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Incrementality": its content identity is package/module identity, normalized
path, `source_hash`, and edition, matching the session source-version summary.
`origin` and open-buffer versions are retained as freshness and diagnostic
metadata, but they are not part of the published source-version content identity.

### Loading Map Preservation

`loading_map` is `Some` only when source loading changed offsets (leading-BOM
strip or CRLF→LF normalization for disk/open-buffer text). It is `None` for
identity loads and for generated text. The frontend never edits the map; it
forwards the session map so `span_bridge` can expose source-loading input byte
offsets through `MappedSourceRange.original_input` when diagnostics or LSP
adapters need that optional view. `SourceRange` values themselves remain
loaded-text coordinates.

## Algorithm / Logic

### Load a single SourceUnit

1. Build a `SourceUnitRequest` for the target `BuildSnapshotId` and `SourceInput`
   (package id, module path, normalized path, edition, origin input).
2. Delegate to the wrapped `mizar_session::SourceLoader::load`, which performs
   path normalization, package-root enforcement, byte reading, UTF-8 validation,
   leading-BOM stripping, CRLF→LF normalization, source hashing, `LineMap`
   construction, and `LoadingMap` emission.
3. Project the returned `LoadedSource` into a `SourceUnit`, preserving
   `source_id`, `package_id`, `module_path`, normalized path, edition, text,
   hash, line map, loading map, origin, and generated anchor unchanged.
4. Return the `SourceUnit`.

Orchestration calls `register_source_unit` immediately after loading and before
preprocessing to record the loaded `LineMap` / `LoadingMap` with the mutable
`SpanBridge` registry. Source loading itself does not mutate bridge state.

Source loading performs no encoding work of its own here. Code-region ASCII
validation is deferred to preprocessing; this module only carries the
session-validated encoding and identity forward.

## Canonical Disk Payload

`canonical_disk_bytes` encodes disk sources only. The version domain is
`mizar-frontend-source-disk-v1`; fields are u64 little-endian length followed by
bytes: package, module, normalized path, edition, UTF-8 text, source hash, and
loading map. The hash is exactly 32 raw bytes. The map starts with presence byte 0/1; present segments use a tag
(0 original, 1 removed BOM, 2 normalized newline) and four u64 little-endian
loaded/original range bounds; BOM loaded bounds are zero. Only complete 33-byte
records are allowed through the map field end. Order is preserved.
Allocator-local source ids and diagnostic filesystem paths are never encoded.
These storage bytes include maps and must not be used directly as IR semantic
content-hash input; publication must preserve the separate source-map side-table
hash required by the IR publisher, as defined below.

`from_canonical_disk_bytes` accepts the current session SourceId and validated
disk SourceInput. SourceInput is not a validation token: the caller must validate
its metadata through the session loader. The codec performs no filesystem
validation, path normalization or loader calls; it checks Disk origin and stable
field equality only. Encoding requires Disk origin, no generated anchor and a
DiskBytes map origin matching normalized_path; decoding reconstructs that origin
from the request. It reconstructs maps through session
constructors, and uses the input's local path. Neither codec loads or parses
source. Invalid schema, UTF-8, lengths, trailing bytes, mismatched hashes or map
metadata/ranges, unsupported origins and generated anchors return `None`.
This is local payload rejection, not a language diagnostic or cache acceptance.
Loading projection remains unchanged.

## Error Handling

Loading surfaces `mizar_session::SourceLoadError` unchanged. Examples include
source path outside package root, unsupported extension, invalid UTF-8,
unreadable file, stale open-buffer version, unmappable open-buffer URI,
generated source without metadata, duplicate module paths, unsupported origins,
source-id allocation failure, and the path-normalization variants; this list is
non-exhaustive because wrapped session loaders may return any current
`SourceLoadError` variant.
The frontend converts these into file-level frontend diagnostics in the
orchestration layer; it does not invent new error categories for conditions that
`mizar-session` already classifies.

A load failure produces no `SourceUnit`; orchestration reports the diagnostic
and stops the pipeline for that file before preprocessing.

## Tests

Key scenarios:

- real disk codec roundtrips preserve BOM/CRLF maps, rebind local identity/path,
  retain canonical bytes and reject corrupt/incompatible/unsupported inputs;

- a disk `LoadedSource` projects to a `SourceUnit` with identical
  `source_id`, `normalized_path`, `edition`, `source_hash`, `line_map`, and
  `loading_map` (no recomputation);
- a BOM-stripped / CRLF-normalized disk source carries a `Some(loading_map)`
  into the `SourceUnit`;
- an identity load (no offset change) carries `loading_map = None`;
- an open-buffer `SourceUnit` records `SourceOrigin::OpenBuffer` and the
  validated document version;
- open-buffer diagnostic paths decode local `file://` URI paths and fall back to
  `normalized_path` when the URI path is not usable;
- a generated `SourceUnit` preserves `SourceOrigin::Generated` and
  `generated_anchor`;
- `register_source_unit` records the `LineMap` / `LoadingMap` with the bridge and
  reports `SpanBridgeError` for duplicate conflicting registrations;
- a session `SourceLoadError` (invalid UTF-8, path outside root) is propagated
  without being reclassified.

## Constraints and Assumptions

- This module does not read or normalize bytes itself; it delegates to
  `mizar-session` and only reshapes the result.
- Loading preserves `source_hash`, `line_map`, and `loading_map`; blob decoding
  reconstructs maps through session constructors and checks the stored hash.
- `normalized_path` and `edition` are retained from `LoadedSource` because
  parser inputs, lexical-environment requests, cache keys, and diagnostics need
  them later.
- `file_path` is local diagnostic metadata, excluded from published identity.
- A `SourceUnit` is treated as immutable after construction and may be retained
  by snapshot leases, LSP views, or downstream phase outputs.

## SourceUnit Publication

The output kind is `SourceUnit`, descriptor schema `mizar-frontend/source-unit-publication/v1`, IR SchemaVersion 1, phase `SourceLoad`. Semantic bytes are the 32 bytes of `SourceUnitCacheKey::stable_hash()`; storage bytes are `canonical_disk_bytes()`. Neither current SourceId nor absolute diagnostic path enters either hash stream.
The IR work-unit label is `format!("{:?}:{:?}", package_id.as_str(), module_path.as_str())`. The sole named input is `source`, domain `SOURCE_UNIT_CACHE_KEY_VERSION`, with that source key digest; no parent or dependency hashes are required for source loading.
The source-map side table has one record: kind `source-storage`, key normalized package-relative path, digest BLAKE3 derive-key with context `mizar-frontend/source-unit-storage-side-table/v1` over the full storage bytes. This conservative side identity includes the actual loading maps while semantic content remains map-independent. Other side tables are empty.

Disk publication uses `OutputOrigin::PackageSource` and `PublicationTarget::CurrentPackage`.
