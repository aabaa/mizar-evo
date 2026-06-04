# Module: source

> Canonical language: English. Japanese companion: [../ja/source.md](../ja/source.md).

## Purpose

This module defines source records used to create `SourceVersion` values.

It owns normalized source paths, validated source text handles, source hashes, and open-buffer source text supplied by LSP requests. It prepares source identity for snapshots, but it does not preprocess comments, tokenize, parse, or resolve imports.

## Public API

```rust
pub struct NormalizedPath(String);

impl NormalizedPath {
    pub fn as_str(&self) -> &str;
}

pub struct SourceInput {
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub edition: Edition,
    pub origin: SourceOriginInput,
}

pub struct DiskSourceLoader { /* package root */ }

impl DiskSourceLoader {
    pub fn new(package_root: impl Into<PathBuf>) -> Self;
    pub fn package_root(&self) -> &Path;
}

#[non_exhaustive]
pub enum SourceOriginInput {
    Disk { path: PathBuf },
    OpenBuffer {
        uri: DocumentUri,
        expected_version: LspDocumentVersion,
        actual_version: LspDocumentVersion,
        text: Arc<str>,
    },
    Generated { generator: GeneratedSourceKind, text: Arc<str>, anchor: Option<SourceAnchor> },
}

pub struct LoadedSource {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub text: Arc<str>,
    pub source_hash: Hash,
    pub edition: Edition,
    pub origin: SourceOrigin,
    pub line_map: LineMap,
    pub loading_map: Option<LoadingMap>,
    pub generated_anchor: Option<SourceAnchor>,
}

pub trait SourceLoader {
    fn load(
        &self,
        snapshot: BuildSnapshotId,
        input: SourceInput,
        ids: &dyn SessionIdAllocator,
    ) -> Result<LoadedSource, SourceLoadError>;
    fn normalize_path(&self, package_root: &Path, path: &Path) -> Result<NormalizedPath, SourceLoadError>;
    fn hash_text(&self, text: &str) -> Hash;
}

pub fn normalize_path(package_root: &Path, path: &Path) -> Result<NormalizedPath, SourceLoadError>;
pub fn hash_text(text: &str) -> Hash;
pub fn normalize_source_path(package_root: &Path, path: &Path) -> Result<NormalizedPath, SourcePathError>;

#[non_exhaustive]
pub enum SourcePathError {
    UnsupportedPathEncoding { path: PathBuf },
    PackageRootUnavailable { path: PathBuf, kind: io::ErrorKind },
    SourcePathUnavailable { path: PathBuf, kind: io::ErrorKind },
    OutsidePackageRoot { package_root: PathBuf, path: PathBuf },
    NonCanonicalPathAlias { requested: PathBuf, canonical: PathBuf },
    NonCanonicalPathSpelling { requested: PathBuf, canonical: PathBuf },
    InvalidNamespaceComponent { component: String },
    MissingSourceRoot { path: PathBuf },
    UnsupportedExtension { path: PathBuf },
}

#[non_exhaustive]
pub enum SourceLoadError {
    SourcePathOutsidePackageRoot { package_root: PathBuf, path: PathBuf },
    UnsupportedFileExtension { path: PathBuf },
    InvalidUtf8 { path: Option<PathBuf> },
    UnreadableSourceFile { path: PathBuf, kind: io::ErrorKind },
    DuplicateModulePath { package_id: PackageId, module_path: ModulePath },
    StaleLspDocumentVersion { expected: LspDocumentVersion, actual: LspDocumentVersion },
    UnmappedOpenBufferUri { uri: DocumentUri },
    GeneratedSourceWithoutMetadata { module_path: ModulePath },
    SourceIdAllocation { error: IdError },
    UnsupportedSourceOrigin { origin: SourceOriginKind },
    InvalidSourcePath { error: SourcePathError },
}

#[non_exhaustive]
pub enum SourceOriginKind {
    Disk,
    OpenBuffer,
    Generated,
}
```

`LoadedSource` is an immutable source-text handle. Snapshot creation consumes loaded sources and records their `SourceVersion` summaries.
`load` takes the target `BuildSnapshotId` so it can request a snapshot-scoped `SourceId` from `SessionIdAllocator`.
`LoadedSource.origin` uses the snapshot module's `SourceOrigin`; the source module does not define a duplicate origin enum for loaded records.
`SourceLoader` helper methods delegate to the public `normalize_path` and `hash_text` helpers. `normalize_path` reuses `normalize_source_path`, while `hash_text` hashes only the normalized text content.
`DiskSourceLoader` owns the package root used for path and URI normalization. It implements `SourceLoader` for disk files, open-buffer overlays mapped from `file://` document URIs, and generated source fragments.
`NormalizedPath::as_str` is intentionally public so snapshot identity, diagnostics, and downstream metadata can read the canonical path spelling without exposing a mutable path representation. `DocumentUri` and `LspDocumentVersion` are the crate-level public aliases defined with the source-map coordinate types and used here for open-buffer source loading.
`PackageId`, `ModulePath`, and `Edition` values are supplied by upstream package
planning and module resolution. Single-source loading preserves those identity
values unchanged while validating source paths, text, open-buffer freshness, and
generated-source metadata. The future source-loading aggregator may reject
duplicate module paths across a build plan with
`SourceLoadError::DuplicateModulePath`, but a single `SourceLoader::load` call
does not have enough context to emit that error. `SnapshotRegistry::create_snapshot`
remains the final pre-hash validation boundary for registry snapshots.

## Dependencies

- Internal: `ids`, `source_map`, `snapshot`
- External: filesystem, package metadata, hashing, UTF-8 validation, LSP document synchronization types

This module is consumed by snapshot creation, frontend source loading, LSP open-buffer overlay construction, diagnostics, and documentation/extraction source consumers.

## Data Structures

### Normalized Path

`NormalizedPath` is a workspace- or package-relative path with normalized separators and no `.` or `..` components.

It must not contain:

- absolute path prefixes;
- symlink-expanded host-specific paths unless explicitly marked local-only;
- platform-specific separator differences;
- non-canonical case variants for package-managed source paths.
- namespace components that are not language identifiers, including reserved words.

Local diagnostics may keep an absolute display path separately. Published artifacts use normalized paths.

### Loaded Source

`LoadedSource` contains validated UTF-8 text and a `LineMap` for that exact text. Disk and open-buffer inputs store the text after source-loading normalization. Generated inputs store the accepted generated text byte-for-byte, with no source-loading BOM or newline normalization. It is immutable after construction and may be retained by snapshot leases, LSP snapshots, diagnostic indexes, or source-map handles.

`source_hash` is computed from `LoadedSource.text`. For disk and open-buffer inputs, that means after UTF-8 validation and source-loading normalization such as leading BOM stripping and newline normalization. For open buffers, it is the normalized editor-provided text, not the on-disk file. For generated inputs, it is the hash of the exact generated text accepted from the generator. Byte-exact provenance, if needed for packaging or diagnostics, must use origin metadata or a separate raw-content hash rather than redefining `source_hash`.

`loading_map` is present when source loading changed offsets before `LoadedSource.text` was created. It maps normalized loaded-text ranges back to the source-loading input: original file byte offsets for disk sources or editor-provided text byte offsets for open buffers. Generated inputs carry an optional `SourceAnchor` on `SourceOriginInput`; `LoadedSource.generated_anchor` preserves that anchor. Generated loading performs no byte-offset transforms and emits no `LoadingMap`; generated locations are recovered from the optional anchor and generated-span metadata instead. When no source-loading transform changed offsets, the mapping is identity and may be omitted.

### Source Origin

`SourceOrigin` distinguishes where the text came from:

- `Disk` for source files read from the package tree;
- `OpenBuffer` for unsaved editor text;
- `Generated` for compiler-created or tool-provided source fragments.

Open-buffer sources can override disk sources only for the targeted LSP request or watch generation. They are not written into normal artifact output.

## Algorithm / Logic

### Disk Source Loading

1. Normalize the path relative to the package root.
2. Reject paths outside the package root or outside its required `src/` source tree.
   Paths inside the package root but outside `src/` report through
   `SourceLoadError::InvalidSourcePath` carrying
   `SourcePathError::MissingSourceRoot`, not through the package-root-boundary
   category.
3. Read bytes from disk.
4. Validate UTF-8. Invalid bytes are rejected before line-map construction and must not be decoded lossily into `U+FFFD`.
5. If the validated text starts with a UTF-8 BOM signature, strip that leading `U+FEFF`.
6. Normalize source-loading newlines by replacing each CRLF pair with one LF. Lone `\r` is not treated as a platform newline and remains malformed lexer-boundary input if it reaches preprocessing.
7. Record a `LoadingMap` from normalized loaded-text offsets back to original file byte offsets when BOM stripping or newline normalization changed offsets.
8. Compute the source hash from `LoadedSource.text`.
9. Build the `LineMap` over `LoadedSource.text`.
10. Return `LoadedSource`.

Only the leading UTF-8 BOM is treated as an encoding signature. A `U+FEFF` anywhere else is preserved in loaded text and remains a malformed lexer-boundary character if it appears in code.

Code-region ASCII validation belongs to preprocessing. This module only validates the text encoding and source identity.

### Open-Buffer Source Loading

1. Compare the request's expected LSP document version with the actual editor-buffer version supplied by the LSP bridge, and reject stale or structurally invalid versions before allocating a `SourceId`.
2. Normalize the document URI to a package source path. A URI that cannot become
   a package-relative path at all, such as a non-`file://` URI, an undecodable
   percent-encoded URI, or a file URI outside the package root, reports
   `SourceLoadError::UnmappedOpenBufferUri`.
3. Use the editor-provided text as authoritative for the request.
4. Strip one leading `U+FEFF` from package-authored open-buffer text so editor views of a BOM-prefixed disk file match disk source loading.
5. Normalize source-loading newlines by replacing each CRLF pair with one LF. Lone `\r` is preserved so frontend/lexer diagnostics can reject it consistently.
6. If stripping or newline normalization changed offsets, record a `LoadingMap` from normalized loaded-text offsets back to editor-provided text byte offsets.
7. Compute source hash and `LineMap` from `LoadedSource.text`.
8. Mark the origin as `OpenBuffer` with the validated actual document version.

Once an open-buffer URI has been mapped to a package-relative path, source-path
validation uses the same `normalize_source_path` classification as disk loading.
For example, a package `src/` file with a non-`.miz` extension reports
`SourceLoadError::UnsupportedFileExtension`, and missing `src/` roots,
non-canonical paths, or invalid namespace components report through
`SourceLoadError::InvalidSourcePath`.

Open-buffer text may be newer than the last verified artifact. Consumers must carry freshness metadata rather than silently treating artifact data as current. LSP diagnostics and edits must convert from `LoadedSource.text` offsets through `loading_map` before applying protocol UTF-16 position rules against the editor document.

### Generated Source Loading

Generated sources require a non-empty generator kind and, when available, an anchor to original source. Loading rejects blank generator metadata before allocating a `SourceId`, preserves accepted generator metadata in `LoadedSource.origin`, and preserves the optional anchor in `LoadedSource.generated_anchor`.

Generated source text is already UTF-8 because it enters the API as `Arc<str>`. The source loader preserves that text byte-for-byte as `LoadedSource.text`: a leading `U+FEFF` is not treated as an encoding signature, CRLF pairs are not converted to LF, and lone `\r` remains unchanged. The `source_hash` and `LineMap` are computed over this exact generated text, and `loading_map` is `None` because generated loading performs no source-loading offset transform. A generator that wants package-authored source normalization must normalize its own output before constructing `SourceOriginInput::Generated`, while still recording generator metadata and any source anchor.

Generated source text may be used for diagnostics, documentation, or extraction, but it must not be mistaken for package-authored `.miz` source.

## Error Handling

`SourceLoadError` includes:

- source path outside package root;
- source path inside the package root but outside the required `src/` source
  tree, reported through `InvalidSourcePath` carrying
  `SourcePathError::MissingSourceRoot`;
- unsupported file extension;
- invalid UTF-8;
- unreadable source file;
- duplicate module path supplied by a future source-loading aggregator over a build plan; single-source `DiskSourceLoader::load` does not emit this variant;
- stale or structurally invalid LSP document version;
- open-buffer URI that cannot become a package-relative path;
- generated source without required generator metadata.
- source id allocation failure from `SessionIdAllocator`.
- unsupported source origin for a concrete loader that intentionally implements only part of the `SourceOriginInput` surface.
- other normalization-specific path errors from `normalize_source_path` that do not fit the explicit path categories.

User-facing read and encoding failures are emitted as frontend/build diagnostics. Internal callers still receive structured errors so snapshot creation can decide whether the build request is fatal or recoverable.
Allocator failures carry the underlying `IdError`; in particular, allocator overflow is not silently converted into a source identity.
Disk and open-buffer source-path validation share the same error categories after
URI-to-path mapping succeeds; `UnmappedOpenBufferUri` is reserved for URI mapping
failures such as non-`file://` schemes, undecodable percent encoding, or file
URIs outside the package root.
Traceability for the reserved-looking source-loading variants is:

| Variant | Current classification | Public observable path |
|---|---|---|
| `SourceLoadError::InvalidSourcePath` | public source-loading path normalization surface | `DiskSourceLoader::load` for disk and mapped open-buffer sources, `SourceLoader::normalize_path`, and the public `normalize_path` helper map `normalize_source_path` failures such as missing `src/` roots, non-canonical aliases, non-canonical spelling, or invalid namespace components into this variant. |
| `SourceLoadError::UnsupportedSourceOrigin` | custom-loader-only | The default `DiskSourceLoader` supports all current `SourceOriginInput` variants (`Disk`, `OpenBuffer`, and `Generated`) and does not emit this variant. It remains public because `SourceLoader` is a public trait and downstream concrete loaders may intentionally implement only part of the origin surface. |

## Tests

Key scenarios:

- disk and open-buffer sources with identical text produce the same source hash but different origins;
- open-buffer source overrides disk text only when the expected and actual document versions match;
- invalid UTF-8 is rejected before line-map construction and is not turned into replacement characters by lossy decoding;
- a leading UTF-8 BOM is accepted and stripped before line-map construction;
- non-leading `U+FEFF` is not stripped by source loading;
- open-buffer BOM stripping and newline normalization preserve a loading map back to editor-provided text offsets;
- path normalization rejects paths outside the package root and paths inside the package root but outside `src/`;
- CRLF and LF handling matches `LineMap` expectations;
- generated-source inputs carry non-empty generator metadata and optional anchors, and loaded generated sources preserve generator metadata in `SourceOrigin` plus the optional anchor in `LoadedSource.generated_anchor`;
- generated-source text with a leading `U+FEFF` and CRLF is preserved byte-for-byte, hashes as that exact text, and emits no `LoadingMap`;
- source hashes do not include absolute paths or document versions.

## Constraints and Assumptions

- This module does not parse, preprocess, or tokenize source text.
- Source hashes are content hashes, not freshness decisions by themselves.
- Absolute paths are local diagnostic metadata and are excluded from published source identity.
- Source text is retained only while snapshots, source maps, diagnostics, LSP views, or downstream consumers hold leases.
