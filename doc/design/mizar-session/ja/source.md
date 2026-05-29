# Module: source

> Canonical language: English. English canonical version: [../en/source.md](../en/source.md).

## Purpose

この module は `SourceVersion` values を作るための source records を定義する。

Normalized source paths、validated source text handles、source hashes、LSP requests が提供する open-buffer source text を所有する。Snapshots 用の source identity を準備するが、comments の preprocess、tokenize、parse、import resolution は行わない。

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
    fn load(&self, input: SourceInput, ids: &dyn SessionIdAllocator) -> Result<LoadedSource, SourceLoadError>;
    fn normalize_path(&self, package_root: &Path, path: &Path) -> Result<NormalizedPath, SourceLoadError>;
    fn hash_text(&self, text: &str) -> Hash;
}
```

`LoadedSource` は immutable source-text handle である。Snapshot creation は loaded sources を consume し、その `SourceVersion` summaries を record する。

## Dependencies

- Internal: `ids`, `source_map`, `snapshot`
- External: filesystem、package metadata、hashing、UTF-8 validation、LSP document synchronization types

この module は snapshot creation、frontend source loading、LSP open-buffer overlay construction、diagnostics、documentation/extraction source consumers から consume される。

## Data Structures

### Normalized Path

`NormalizedPath` は normalized separators を持ち、`.` or `..` components を含まない workspace- or package-relative path である。

含んではならないもの:

- absolute path prefixes
- local-only と明示されない symlink-expanded host-specific paths
- platform-specific separator differences
- package-managed source paths に対する non-canonical case variants

Local diagnostics は absolute display path を別に持ってよい。Published artifacts は normalized paths を使う。

### Loaded Source

`LoadedSource` は source-loading normalization 後の validated UTF-8 text と、その exact text 用の `LineMap` を含む。Construction 後 immutable であり、snapshot leases、LSP snapshots、diagnostic indexes、source-map handles によって retain され得る。

`source_hash` は UTF-8 validation と、先頭 BOM stripping や newline normalization などの source-loading normalization 後の `LoadedSource.text` から計算される。Open buffers では on-disk file ではなく normalized editor-provided text である。Packaging or diagnostics のために byte-exact provenance が必要な場合は、`source_hash` を再定義せず、origin metadata または separate raw-content hash を使う。

`loading_map` は、`LoadedSource.text` が作られる前に source loading が offset を変更した場合に存在する。Normalized loaded-text ranges を source-loading input へ戻す map であり、disk sources では original file byte offsets、open buffers では editor-provided text byte offsets、generated inputs では generated-source anchor を指す。Source-loading transform が offset を変えなかった場合、mapping は identity なので省略してよい。

### Source Origin

`SourceOrigin` は text の由来を区別する。

- `Disk`: package tree から読まれた source files
- `OpenBuffer`: unsaved editor text
- `Generated`: compiler-created or tool-provided source fragments

Open-buffer sources は targeted LSP request or watch generation に限って disk sources を override できる。Normal artifact output には書き込まない。

## Algorithm / Logic

### Disk Source Loading

1. Package source root からの relative path に normalize する。
2. Package source tree の外側の path を reject する。
3. Disk から bytes を read する。
4. UTF-8 を validate する。Invalid bytes は line-map construction 前に reject し、lossy decode によって `U+FFFD` にしてはいけない。
5. Validated text が UTF-8 BOM signature で始まる場合、先頭 `U+FEFF` を strip する。
6. CRLF pairs を LF 一つに置き換えて source-loading newlines を normalize する。Lone `\r` は platform newline として扱わず、preprocessing に届いた場合は malformed lexer-boundary input のままにする。
7. BOM stripping or newline normalization が offset を変更した場合、normalized loaded-text offsets から original file byte offsets への `LoadingMap` を記録する。
8. `LoadedSource.text` から source hash を compute する。
9. `LoadedSource.text` 上に `LineMap` を build する。
10. `LoadedSource` を返す。

Encoding signature として扱うのは先頭 UTF-8 BOM だけです。それ以外の位置にある `U+FEFF` は loaded text に保持され、code に現れた場合は malformed lexer-boundary character のままです。

Code-region ASCII validation は preprocessing に属する。この module は text encoding and source identity のみ validate する。

### Open-Buffer Source Loading

1. LSP bridge が提供した document version を validate する。
2. Document URI を package source path に normalize する。
3. その request では editor-provided text を authoritative として使う。
4. BOM-prefixed disk file の editor view が disk source loading と一致するように、package-authored open-buffer text から先頭 `U+FEFF` を一つ strip する。
5. CRLF pairs を LF 一つに置き換えて source-loading newlines を normalize する。Lone `\r` は frontend/lexer diagnostics が一貫して reject できるよう preserve する。
6. Stripping or newline normalization が offset を変更した場合、normalized loaded-text offsets から editor-provided text byte offsets への `LoadingMap` を記録する。
7. `LoadedSource.text` から source hash and `LineMap` を compute する。
8. Origin を `OpenBuffer` として mark する。

Open-buffer text は last verified artifact より fresh な場合がある。Consumers は artifact data を暗黙に current として扱わず freshness metadata を carry しなければならない。LSP diagnostics and edits は editor document に対する protocol UTF-16 position rules を適用する前に、`LoadedSource.text` offsets から `loading_map` を通して変換しなければならない。

### Generated Source Loading

Generated sources は generator kind と、可能な場合は original source への anchor を必要とする。Generated source text は diagnostics、documentation、extraction に使ってよいが、package-authored `.miz` source と取り違えてはならない。

## Error Handling

`SourceLoadError` includes:

- source path outside package root
- unsupported file extension
- invalid UTF-8
- unreadable source file
- duplicate module path supplied by the build plan
- stale LSP document version
- open-buffer URI that cannot be mapped to a package source
- generated source without required generator metadata

User-facing read and encoding failures は frontend/build diagnostics として emitted される。Internal callers は structured errors も受け取り、snapshot creation が build request を fatal or recoverable と判断できるようにする。

## Tests

Key scenarios:

- disk and open-buffer sources with identical text produce the same source hash but different origins
- open-buffer source overrides disk text only for the matching document version
- invalid UTF-8 is rejected before line-map construction and is not turned into replacement characters by lossy decoding
- leading UTF-8 BOM is accepted and stripped before line-map construction
- non-leading `U+FEFF` is not stripped by source loading
- open-buffer BOM stripping and newline normalization preserve a loading map back to editor-provided text offsets
- path normalization rejects paths outside the package root
- CRLF and LF handling matches `LineMap` expectations
- generated sources preserve generator metadata and anchors
- source hashes do not include absolute paths or document versions

## Constraints and Assumptions

- This module does not parse, preprocess, or tokenize source text.
- Source hashes are content hashes, not freshness decisions by themselves.
- Absolute paths are local diagnostic metadata and are excluded from published source identity.
- Source text is retained only while snapshots, source maps, diagnostics, LSP views, or downstream consumers hold leases.
