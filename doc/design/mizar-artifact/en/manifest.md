# Artifact Manifest

> Canonical language: English. Japanese companion: [../ja/manifest.md](../ja/manifest.md).

## Purpose

The package artifact manifest is the publication index for artifacts emitted
under one package artifact directory. Readers consult the manifest first and do
not discover published artifacts by scanning arbitrary files.

## Schema

The schema version is `mizar-artifact/manifest/v1`.

Manifest fields:

- `schema_version`;
- `package_id`;
- `package_version`;
- `lockfile_hash`;
- `toolchain`;
- `verifier_config_hash`;
- `modules`.

Module entry fields:

- `module_path`;
- `source_file`;
- `source_hash`;
- `artifact_file`;
- `artifact_hash`;
- `interface_hash`;
- `implementation_hash`;
- `proof_witnesses`;
- `diagnostics_hash`.

Entries are sorted by `module_path`, then `artifact_file`.

## Transactions

A manifest transaction:

1. starts from the currently visible manifest, if any;
2. stages module updates whose referenced artifact and witness files already
   exist and match recorded hashes;
3. validates each referenced `VerifiedArtifact` and checks that manifest
   `module_path`, `source_file`, `source_hash`, `interface_hash`,
   `implementation_hash`, `diagnostics_hash`, and proof witness entries agree
   with the artifact;
4. writes a fully serialized temporary manifest;
5. flushes it;
6. renames it over `artifact-manifest.json`;
7. flushes the containing directory when supported.

Readers therefore see either the old complete manifest or the new complete
manifest. They must never merge entries from temporary files or unreferenced
artifacts.

## Recovery

Interrupted commits may leave temporary files or unreferenced artifacts. Readers
ignore those files. Replaying the same transaction with byte-identical staged
entries is idempotent.

If a referenced file is missing, malformed, or hash-mismatched, the transaction
fails and the previous manifest remains authoritative.

## Reader Validation

Readers validate schema version, deterministic entry order, artifact hashes,
witness hashes when available, portable paths, and agreement between manifest
hash fields and the referenced `VerifiedArtifact`. A manifest with duplicate
module entries or unsupported schema version is rejected.
