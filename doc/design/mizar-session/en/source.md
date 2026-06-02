# Module: source

> Canonical language: English. Japanese companion: [../ja/source.md](../ja/source.md).

## Purpose

This module defines source records used to create `SourceVersion` values.

It owns normalized source paths, validated source text handles, source hashes, and open-buffer source text supplied by LSP requests. It prepares source identity for snapshots, but it does not preprocess comments, tokenize, parse, or resolve imports.

## Public API

```rust
pub struct SourceInput {
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub edition: Edition,
    pub origin: SourceOriginInput,
}

pub enum SourceOriginInput {
    Disk { path: PathBuf },
    OpenBuffer { uri: DocumentUri, version: LspDocumentVersion, text: Arc<str> },
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
    InvalidSourcePath { error: SourcePathError },
}
```

`LoadedSource` is an immutable source-text handle. Snapshot creation consumes loaded sources and records their `SourceVersion` summaries.
`load` takes the target `BuildSnapshotId` so it can request a snapshot-scoped `SourceId` from `SessionIdAllocator`.
`LoadedSource.origin` uses the snapshot module's `SourceOrigin`; the source module does not define a duplicate origin enum for loaded records.
`SourceLoader` helper methods delegate to the public `normalize_path` and `hash_text` helpers. `normalize_path` reuses `normalize_source_path`, while `hash_text` hashes only the normalized text content.

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

Local diagnostics may keep an absolute display path separately. Published artifacts use normalized paths.

### Loaded Source

`LoadedSource` contains validated UTF-8 text after source-loading normalization and a `LineMap` for that exact text. It is immutable after construction and may be retained by snapshot leases, LSP snapshots, diagnostic indexes, or source-map handles.

`source_hash` is computed from `LoadedSource.text`, after UTF-8 validation and source-loading normalization such as leading BOM stripping and newline normalization. For open buffers, it is the normalized editor-provided text, not the on-disk file. Byte-exact provenance, if needed for packaging or diagnostics, must use origin metadata or a separate raw-content hash rather than redefining `source_hash`.

`loading_map` is present when source loading changed offsets before `LoadedSource.text` was created. It maps normalized loaded-text ranges back to the source-loading input: original file byte offsets for disk sources or editor-provided text byte offsets for open buffers. Generated inputs carry an optional `SourceAnchor` on `SourceOriginInput`; the task-14 `LoadedSource` surface does not expose that anchor through `loading_map`. When no source-loading transform changed offsets, the mapping is identity and may be omitted.

### Source Origin

`SourceOrigin` distinguishes where the text came from:

- `Disk` for source files read from the package tree;
- `OpenBuffer` for unsaved editor text;
- `Generated` for compiler-created or tool-provided source fragments.

Open-buffer sources can override disk sources only for the targeted LSP request or watch generation. They are not written into normal artifact output.

## Algorithm / Logic

### Disk Source Loading

1. Normalize the path relative to the package source root.
2. Reject paths outside the package source tree.
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

1. Validate the document version supplied by the LSP bridge.
2. Normalize the document URI to a package source path.
3. Use the editor-provided text as authoritative for the request.
4. Strip one leading `U+FEFF` from package-authored open-buffer text so editor views of a BOM-prefixed disk file match disk source loading.
5. Normalize source-loading newlines by replacing each CRLF pair with one LF. Lone `\r` is preserved so frontend/lexer diagnostics can reject it consistently.
6. If stripping or newline normalization changed offsets, record a `LoadingMap` from normalized loaded-text offsets back to editor-provided text byte offsets.
7. Compute source hash and `LineMap` from `LoadedSource.text`.
8. Mark the origin as `OpenBuffer`.

Open-buffer text may be newer than the last verified artifact. Consumers must carry freshness metadata rather than silently treating artifact data as current. LSP diagnostics and edits must convert from `LoadedSource.text` offsets through `loading_map` before applying protocol UTF-16 position rules against the editor document.

### Generated Source Loading

Generated sources require a generator kind and, when available, an anchor to original source. Generated source text may be used for diagnostics, documentation, or extraction, but it must not be mistaken for package-authored `.miz` source.

## Error Handling

`SourceLoadError` includes:

- source path outside package root;
- unsupported file extension;
- invalid UTF-8;
- unreadable source file;
- duplicate module path supplied by the build plan;
- stale LSP document version;
- open-buffer URI that cannot be mapped to a package source;
- generated source without required generator metadata.
- source id allocation failure from `SessionIdAllocator`.
- other normalization-specific path errors from `normalize_source_path` that do not fit the explicit path categories.

User-facing read and encoding failures are emitted as frontend/build diagnostics. Internal callers still receive structured errors so snapshot creation can decide whether the build request is fatal or recoverable.
Allocator failures carry the underlying `IdError`; in particular, allocator overflow is not silently converted into a source identity.

## Tests

Key scenarios:

- disk and open-buffer sources with identical text produce the same source hash but different origins;
- open-buffer source overrides disk text only for the matching document version;
- invalid UTF-8 is rejected before line-map construction and is not turned into replacement characters by lossy decoding;
- a leading UTF-8 BOM is accepted and stripped before line-map construction;
- non-leading `U+FEFF` is not stripped by source loading;
- open-buffer BOM stripping and newline normalization preserve a loading map back to editor-provided text offsets;
- path normalization rejects paths outside the package root;
- CRLF and LF handling matches `LineMap` expectations;
- generated-source inputs carry generator metadata and optional anchors, and loaded generated sources preserve generator metadata in `SourceOrigin`;
- source hashes do not include absolute paths or document versions.

## Constraints and Assumptions

- This module does not parse, preprocess, or tokenize source text.
- Source hashes are content hashes, not freshness decisions by themselves.
- Absolute paths are local diagnostic metadata and are excluded from published source identity.
- Source text is retained only while snapshots, source maps, diagnostics, LSP views, or downstream consumers hold leases.
