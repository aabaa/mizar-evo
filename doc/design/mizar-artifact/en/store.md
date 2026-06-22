# Store And Canonical Form

> Canonical language: English. Japanese companion: [../ja/store.md](../ja/store.md).

## Purpose

This document specifies the shared store and canonical-form rules used by
`mizar-artifact` published schemas.

## Responsibilities

The `store` module owns:

- canonical UTF-8 JSON serialization for artifact schemas;
- schema-version compatibility checks;
- semantic, implementation, diagnostic, and content hash helpers;
- content-addressed blob storage under an artifact root;
- atomic named artifact writes through temp files and rename.

It does not own cache records, cache reuse, proof acceptance, or producer
projection construction.

## Canonical JSON

Published structured artifacts are UTF-8 JSON. Canonical serialization uses:

- object keys sorted lexicographically by Unicode scalar value;
- arrays kept in the already-canonical order required by the owning schema;
- no insignificant whitespace;
- lowercase `true`, `false`, and `null`;
- strings escaped only where JSON requires escaping;
- non-negative integer values only where a schema explicitly uses numbers.

Schema modules must sort sets and maps before constructing canonical arrays.
Hash-map iteration order, filesystem scan order, worker scheduling order, and
temporary file names are forbidden canonical inputs.

## Schema Versions

Every published schema has a `schema_version` string. A reader must reject a
record when the field is missing, not a string, or not equal to one of the
reader-supported versions for that schema.

For the first implementation wave, each schema supports exactly one version:
`mizar-artifact/<schema>/v1`.

## Hash Classes

Hashes use domain-separated canonical bytes. The domain name is part of the
hash input so that identical JSON bytes in different roles do not collide as
equivalent artifact facts.

Semantic hashes cover importer-visible meaning. They exclude local metadata:

- `verified_at`;
- transaction/session/temp file identifiers;
- local run ids;
- wall-clock timings unless the schema explicitly classifies them as diagnostic
  metadata.

Diagnostic hashes cover stable diagnostic/development projections and may
change when diagnostic payloads change without changing semantic hashes.

Implementation hashes cover the full stable verified projection for one module
except explicitly local fields. They may change for proof-body-only or
diagnostic changes that do not affect downstream semantic imports.

## Store Layout

The store root is a package artifact directory. Readers use manifests and
recorded paths; arbitrary scanning is not a publication mechanism.

```text
artifact_dir/
  artifact-manifest.json
  blobs/
    <content-hash>.json
  tmp/
    <writer-local-name>
  <module>.mizir.json
  logs/
  explanations/
  missing_facts.json
```

Blob file names are derived from recorded content hashes. Temporary file names
are local metadata and never enter canonical artifact content or semantic
hashes.

## Development Artifacts

The store may also write development artifacts such as `missing_facts.json`,
ATP logs, diagnostic explanation previews, and optional debug dumps. They are
not published package interface unless a later manifest schema explicitly
references them.

Development artifacts use the same temp-and-rename write discipline and content
hash helpers, but they are not included in `interface_hash`. If a stable
diagnostic or explanation projection is referenced from `VerifiedArtifact`, the
reference records a portable path and content hash; otherwise readers must treat
the file as local development output and ignore it for package compatibility.

This folded surface replaces a separate `development_artifact.md` for the
current task sequence. Full producer-driven explanation emission remains out of
scope until the producing diagnostics/proof services expose concrete projection
inputs.

## Atomic Writes

Named artifact writes must:

1. serialize bytes fully before publication;
2. write to a temporary file in the target directory;
3. flush file contents;
4. rename the temporary file to the final path;
5. flush the containing directory when the platform supports it.

Readers must never treat a temporary file as published. If a final file exists
but does not parse or its content hash mismatches, reading fails with a
structured artifact error instead of silently accepting partial bytes.

## Error Policy

Store errors are structured and stable enough for diagnostics:

- unsupported or missing schema version;
- malformed JSON with byte position;
- unexpected or missing schema field;
- invalid hash spelling;
- content hash mismatch;
- invalid portable path;
- I/O error with path and error kind.

Error variants are forward-compatible and public enums are non-exhaustive.
