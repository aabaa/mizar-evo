# mizar-artifact Store And Canonical Form

> Canonical language: English. Japanese companion:
> [../ja/store.md](../ja/store.md).

## Purpose

This document specifies the store-level rules shared by published
`mizar-artifact` schemas before the source implementation lands.

It refines phase 15 artifact storage from
[architecture 11](../../architecture/en/11.artifact_and_incremental_build.md)
and [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md):
published artifacts are stable UTF-8 JSON projections, written atomically,
indexed by a manifest, and hashed from deterministic canonical bytes.

## Scope

The store spec owns:

- package artifact root layout and path policy;
- canonical UTF-8 JSON serialization requirements;
- top-level `schema_version` compatibility checks;
- semantic, implementation, diagnostic, and local-metadata hash separation;
- atomic write and validating read requirements for published files.

The store spec does not own:

- manifest transaction semantics beyond the store requirements that make
  manifest publication safe;
- internal cache records, cache-key lookup, cluster-db storage, or proof reuse;
- raw compiler IR dumps or scheduler state;
- proof authority, kernel acceptance, proof search, or policy decisions.

## Store Layout

Each package has one artifact root, normally the package `artifact_dir`
(`build/` by default). Portable published paths are package-relative or
workspace-relative. Absolute paths may appear only in explicitly local
diagnostic or debug payloads, and those fields are excluded from semantic and
publication-equivalence hashes.

The standard package manifest file is:

```text
artifact-manifest.json
```

The manifest is the only publication index. Readers must not discover
published artifacts by scanning arbitrary files in the artifact root.

The store recognizes the following root-level areas from internal 02:

```text
build/
  artifact-manifest.json
  <module>.mizir.json
  proof-witnesses/
    <module>/
      <witness-file>
  logs/
    <module>.atp-log
  missing_facts.json
  explanations/
    <module>/
      <diagnostic-id>.json
```

Schema-specific specs may refine file names for module summaries,
registration summaries, witness references, and verified artifacts, but every
published artifact path must be manifest-reachable and must remain under the
package artifact root after path normalization. `..` traversal, symlink escape,
and drive-root changes are rejected for portable published paths.

Content-addressed blobs under internal cache directories such as
`.mizar-cache/blobs/` belong to `mizar-cache`, not `mizar-artifact`.
`mizar-artifact` may write hash-named published files only when a
schema-specific artifact or witness spec requires that published path shape.

Unreferenced files in the artifact root are not published artifacts. They may
be left by failed writes or interrupted sessions, but readers ignore them.

## Canonical UTF-8 JSON

Published artifacts are canonical UTF-8 JSON unless a later published schema
explicitly defines another format.

Canonical JSON bytes obey these rules:

- the byte stream is UTF-8 without a byte order mark;
- each artifact is one JSON value followed by one `\n`;
- object member names are unique and sorted by Unicode scalar value;
- schema fields are emitted in canonical field order, which is the same order
  as object-member sorting unless the schema defines a stricter stable order;
- maps and sets are serialized as arrays sorted by their schema-defined
  canonical keys;
- arrays that represent source order, proof order, diagnostic order, or other
  semantically ordered sequences preserve that sequence;
- optional fields are omitted when absent unless the schema explicitly requires
  a `null`;
- strings preserve source text exactly except for required JSON escaping;
- string escaping uses `\"` for quotation mark, `\\` for reverse solidus, `\b`,
  `\t`, `\n`, `\f`, and `\r` for those control characters, and lowercase
  `\u00xx` escapes for the remaining U+0000 through U+001F control characters;
  non-control Unicode scalar values remain UTF-8, not `\u`-escaped;
- paths are normalized before serialization and use `/` separators in portable
  fields;
- hash strings include their algorithm or domain prefix when the schema
  exposes them to readers;
- no insertion order, map iteration order, filesystem directory order, wall
  clock order, or worker completion order may affect canonical bytes.

The implementation must reject duplicate JSON object keys in published
artifacts read from disk. A non-canonical but parseable artifact is not silently
rewritten by a reader; it is either rejected or treated as non-authoritative by
the consuming command.

## Schema Versions

Every published top-level JSON schema has a required `schema_version` string.
The schema version is part of the canonical bytes and part of every hash domain
whose meaning changes when the schema changes.

Readers perform compatibility checks before trusting an artifact:

- missing `schema_version` is incompatible;
- an unknown schema name or schema family is incompatible;
- a newer major version is incompatible;
- an older major version is incompatible unless a schema-specific migration is
  explicitly implemented;
- a newer minor version is readable only when the schema declares forward
  compatibility for all added fields that the reader may ignore;
- malformed version strings are incompatible.

Compatibility errors report the schema family, actual version, supported
version range, and artifact path. They do not trigger cache fallback that would
turn an unsupported artifact into proof authority.

The shared task 3 compatibility helper carries the schema family and supported
range in every error. Artifact readers that have a path use the path-aware
check so reported diagnostics include the artifact path.

## Hash Separation

The store distinguishes four hash classes.

| Hash class | Covers | Excludes | Consumer |
|---|---|---|---|
| `interface_hash` | dependency-facing exported signatures, accepted exported proof status, schema versions that affect importers | implementation bodies, local diagnostics, local metadata | downstream semantic phases |
| `implementation_hash` | full stable published projection for the module | hash-excluded local metadata | local rebuilds, LSP, docs |
| `diagnostic_hash` | projected diagnostics and structured local explanation handles | semantic fields and proof authority | diagnostics, LSP refresh |
| `artifact_hash` | canonical bytes of the published file after applying declared hash exclusions | temporary names, session ids, wall-clock local fields | manifest validation and publication integrity |

Each hash class is domain-separated. A byte string valid for one hash class is
never reused directly as another class. The concrete hash algorithm and domain
tags are implementation constants introduced with the shared canonical hashing
implementation in task 3, but every exposed hash string records enough
algorithm/domain information for readers to reject mismatches.

Task 3 uses the construction label
`mizar-artifact/artifact-framed-hash-text/v1`: the artifact hash class,
schema family, schema version, and filtered canonical JSON bytes are framed as
UTF-8 text and passed through `mizar_session::hash_text`. This preserves the
`mizar-session`-only dependency boundary. It is not an artifact-owned raw
BLAKE3 API, and exposed hash strings must identify this construction label when
they are serialized by later schema tasks.

Diagnostic and development artifacts may have their own hashes, but those
hashes do not decide semantic dependency compatibility. Internal cache keys and
cache records belong to `mizar-cache`, not this crate.

## Hash-Excluded Local Fields

Fields that vary by local build session are allowed only when the schema marks
them as hash-excluded. The initial hash-excluded local fields are:

- `verified_at`;
- temporary file names;
- build session ids and task ids;
- absolute paths present only in local diagnostics or debug payloads;
- wall-clock timings and backend process metadata when the schema classifies
  them as local provenance rather than semantic input.

Hash-excluded fields are still parsed and validated when present. They are
excluded from `interface_hash`, `implementation_hash`, `artifact_hash`, and
publication equivalence unless a later schema explicitly moves a field into a
stable provenance hash domain. A reader must not use a hash-excluded field to
accept a proof result, validate dependency compatibility, or decide package
publication eligibility.

Hash-exclusion paths are object-key paths. An absent path has no effect. A path
to a parent field excludes the whole subtree, making any child exclusion below
that parent redundant. Paths do not address array elements; schemas that need
array-local local metadata must isolate that metadata in an object field whose
whole value can be excluded.

## Atomic Writes

Published files are written with a temp-and-rename protocol in the target
directory or in a store-owned temporary directory on the same filesystem:

1. Serialize canonical bytes to a temporary file whose name is not a published
   path.
2. Flush the file contents.
3. Re-read or hash the temporary file when required by the writer mode.
4. Atomically rename the temporary file to the final artifact path.
5. Flush the containing directory when the platform supports it.
6. Return the final path and hash to the manifest transaction.

Readers either see the previous complete file or the new complete file. A
partially written JSON file is a build error and must not be accepted as a
published artifact. If a write fails before the manifest commits, the previous
manifest remains authoritative; newly written unreferenced files are ignored by
readers and may be cleaned up later.

The manifest transaction protocol is specified by [manifest.md](./manifest.md).
This store spec requires that files referenced by a manifest entry have already
been written, flushed, and hash-validated before the manifest can publish them.

## Validating Reads

Published artifact readers:

- resolve paths through the manifest, not by scanning the artifact root;
- reject paths outside the package artifact root after normalization;
- parse UTF-8 JSON and report parse failures with the artifact path and a
  useful byte, line, or column location when available;
- check `schema_version` before interpreting schema-specific fields;
- reject duplicate object keys;
- validate hashes requested by the manifest entry or consuming command;
- reject missing proof witness files when publication policy requires them.

Read failures are artifact diagnostics. They do not silently fall back to
internal cache records and do not establish proof authority.

## Implementation Staging

Task 2 introduced this specification. Source implementation is staged across
later tasks:

- task 3 implements shared canonical serialization, hash domains, hash
  exclusions, and version checks;
- task 13 implements store writes, atomicity, and corruption-detecting reads;
- task 14 implements manifest transactions;
- schema-specific reader/writer behavior is implemented by each schema task.
