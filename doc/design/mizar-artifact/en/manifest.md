# mizar-artifact Artifact Manifest Schema

> Canonical language: English. Japanese companion:
> [../ja/manifest.md](../ja/manifest.md).

## Purpose

The package artifact manifest is the publication index for `mizar-artifact`.
Downstream builds, LSP, documentation, extraction, and audit tools read the
manifest before reading any module artifact or proof witness file. They must not
discover published artifacts by scanning arbitrary files in the artifact root.

This document refines [store.md](./store.md),
[architecture 11](../../architecture/en/11.artifact_and_incremental_build.md),
and [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
for task 12. It specifies the manifest file schema and the transaction protocol
that later tasks implement.

## Scope

The manifest owns:

- package-level publication metadata;
- module artifact entries and their store-level artifact hashes;
- proof witness file reachability and producer-owned artifact hashes;
- optional development artifact reachability when those files are deliberately
  exposed through the artifact root;
- manifest transaction visibility, validation, and recovery rules.

The manifest does not own:

- the `VerifiedArtifact`, `ModuleSummary`, `RegistrationSummary`, or
  `ProofWitnessRef` object schemas;
- raw compiler IR dumps, scheduler state, internal cache records, or
  `mizar-cache` content-addressed blobs;
- proof authority, witness replay, kernel acceptance, or verifier policy
  decisions;
- the store file I/O implementation, which remains task 13 work;
- the Rust manifest reader/writer and transaction manager, which remain task 14
  work.

The manifest is integrity and reachability metadata. A manifest entry can prove
that a referenced artifact has the publication-equivalent content named by its
store-level artifact hash, but it cannot turn externally attested evidence into
kernel-accepted proof evidence.

Manifest transactions may carry caller-supplied build snapshot freshness state
to reject obsolete writers. That state is transaction control data, not stable
artifact content, and `mizar-artifact` does not define scheduler ordering.

## File Name And Version

The standard package artifact manifest path is:

```text
artifact-manifest.json
```

The manifest path is relative to the package artifact root. Temporary manifest
paths are not published paths and must not be read by consumers.

The schema family is `mizar-artifact/manifest`. Version `1.0` is the initial
version. Task 14 readers support `1.0..=1.0` and reject missing, malformed,
different-major, and newer-minor versions unless a later spec explicitly
declares forward-compatible minor-version handling.

The manifest file's own artifact hash is computed by the store or manifest
manager over canonical manifest JSON using the task-3 artifact-framed hash
construction with class `artifact`, schema family `mizar-artifact/manifest`, and
the manifest `schema_version`. It follows the store hash-exclusion model, not a
raw filesystem-byte hash; schema version `1.0` declares no hash-excluded manifest
fields. The manifest does not store its own hash inside the JSON file.

## Top-Level Shape

Task 12 specifies this canonical JSON field shape:

```text
artifact_manifest = {
  "schema_version": "1.0",
  "package": package_identity,
  "artifact_root": string,
  "lockfile_hash": artifact_hash_string,
  "toolchain": string,
  "language_edition": string,
  "verifier_config_hash": interface_hash_string,
  "modules": [module_artifact_entry, ...],
  "development_artifacts": [development_artifact_entry, ...],
  "provenance": manifest_provenance
}

package_identity = {
  "package_id": string,
  "package_version": string | null,
  "lockfile_identity": string | null
}

manifest_provenance = {
  "generated_by": string,
  "manifest_policy": string,
  "transaction_format": string
}
```

Every listed field is required. Fields whose absent value is meaningful are
encoded as JSON `null`; readers reject omitted required fields and unknown fields
at every schema object.

`artifact_root` is a normalized package-relative path for the artifact root as
seen from the package root. It must not be absolute, must not contain empty,
`.` or `..` path segments, and must use `/` separators.

`toolchain`, `language_edition`, `generated_by`, `manifest_policy`, and
`transaction_format` are stable non-empty strings. They are manifest metadata,
not proof authority.

## Module Entries

`modules` contains one entry for each published module artifact reachable from
this manifest.

```text
module_artifact_entry = {
  "module": module,
  "source_file": string,
  "source_hash": source_hash_string,
  "artifact_file": string,
  "artifact_hash": artifact_hash_string,
  "interface_hash": interface_hash_string,
  "implementation_hash": implementation_hash_string,
  "module_summary_file": string | null,
  "module_summary_hash": artifact_hash_string | null,
  "module_summary_interface_hash": interface_hash_string | null,
  "registration_summary_file": string | null,
  "registration_summary_hash": artifact_hash_string | null,
  "registration_interface_hash": interface_hash_string | null,
  "proof_witnesses": [manifest_proof_witness_entry, ...],
  "diagnostics_hash": diagnostic_hash_string | null
}
```

`module` uses the same identity shape as `ModuleSummary` and `VerifiedArtifact`.
`source_file` follows the same portable path rules as `VerifiedArtifact`.
`artifact_file`, summary file paths, and proof witness paths are package artifact
root-relative paths. They must remain under the artifact root after
normalization and must not be absolute.

`artifact_file` points to the canonical `VerifiedArtifact` file for the module.
`artifact_hash` is the referenced file's store-level artifact hash: canonical
JSON after the referenced schema's declared hash exclusions, framed with class
`artifact`. It is not a raw filesystem-byte hash. `interface_hash` and
`implementation_hash` must equal the top-level hashes inside the referenced
`VerifiedArtifact`.

Summary file fields are optional because task staging may publish the verified
artifact before every sidecar summary is emitted. The three `module_summary_*`
fields are all `null` together or all non-null together. The
`registration_summary_*` fields follow the same rule.

`diagnostics_hash` is an optional diagnostic-payload hash for local diagnostic
indexes or explanation bundles. It is not proof authority and does not replace
the diagnostics embedded in `VerifiedArtifact`.

## Proof Witness Entries

Manifest proof witness entries make externally stored witness files reachable
from a module artifact entry.

```text
manifest_proof_witness_entry = {
  "obligation_id": string,
  "obligation_fingerprint": interface_hash_string,
  "witness_path": string,
  "witness_artifact_hash": artifact_hash_string
}
```

Each entry must match exactly one `ProofWitnessRef` in the referenced
`VerifiedArtifact`:

- `obligation_id` matches `ProofWitnessRef.obligation_id`;
- `obligation_fingerprint` matches `ProofWitnessRef.obligation_fingerprint`;
- `witness_path` matches `ProofWitnessRef.witness_path`;
- `witness_artifact_hash` matches `ProofWitnessRef.witness_artifact_hash`.

The `proof_witnesses` array in a module manifest entry must exactly cover the
referenced `VerifiedArtifact.proof_witnesses` collection. Readers reject missing
manifest witness entries, extra manifest witness entries, and entries whose
identity or hash tuple differs from the referenced artifact. This guarantees
that every accepted obligation requiring a trusted witness remains reachable
through the manifest.

The manifest transaction writes witness files and records producer-owned
`witness_artifact_hash` values before the manifest is committed. A committed
manifest entry that names an accepted witness must make the witness file
reachable by `witness_path`, and readers reject missing witness files or witness
hash mismatches when witness validation is requested. The concrete witness
payload hash construction remains producer-owned as described by
[proof_witness.md](./proof_witness.md).

## Development Artifact Entries

Development artifacts are optional files for diagnostics, missing-facts
workflows, ATP logs, explanation previews, and debug/audit tooling.

```text
development_artifact_entry = {
  "kind": string,
  "path": string,
  "artifact_hash": artifact_hash_string | null,
  "diagnostic_hash": diagnostic_hash_string | null,
  "related_module": module | null
}
```

Development entries are published only when deliberately listed in the manifest.
They must not contain raw compiler IR unless a later development-artifact schema
explicitly marks the file as local/debug-only and excludes it from semantic
artifact hashes. Development artifacts never satisfy proof witness requirements.

Exactly one of `artifact_hash` or `diagnostic_hash` must be non-null unless a
later schema defines a multi-hash development file. Entries with both hash
fields null, or with both fields non-null, are invalid. `related_module` is
optional navigation metadata.

## Hash String Domains

`source_hash` uses the source text hash string from `ModuleSummary` and
`VerifiedArtifact`:

```text
mizar-session/hash-text/v1:<digest>
```

All other hash fields use the task-3 artifact-framed form:

```text
mizar-artifact/artifact-framed-hash-text/v1:<class>:<schema_family>:<schema_version>:<digest>
```

Task 14 validates at least these domains:

| Field | Required class | Required schema family/version | Notes |
|---|---|---|---|
| manifest file hash | `artifact` | `mizar-artifact/manifest`, manifest `schema_version` | Computed externally from canonical manifest JSON; not stored in the file. |
| `lockfile_hash` | `artifact` | producer-owned, valid grammar | Lockfile or lock projection hash. |
| `verifier_config_hash` | `interface` | producer-owned, valid grammar | Verifier configuration fingerprint. |
| `module_artifact_entry.source_hash` | `mizar-session/hash-text/v1` | none | Exact source text hash. |
| `module_artifact_entry.artifact_hash` | `artifact` | `mizar-artifact/verified-artifact`, referenced artifact schema version | Published `VerifiedArtifact` store-level artifact hash. |
| `module_artifact_entry.interface_hash` | `interface` | `mizar-artifact/verified-artifact`, referenced artifact schema version | Must match referenced artifact. |
| `module_artifact_entry.implementation_hash` | `implementation` | `mizar-artifact/verified-artifact`, referenced artifact schema version | Must match referenced artifact. |
| `module_summary_hash` | `artifact` | `mizar-artifact/module-summary`, referenced summary schema version | Optional `ModuleSummary` store-level artifact hash. |
| `module_summary_interface_hash` | `interface` | `mizar-artifact/module-summary`, referenced summary schema version | Optional summary interface hash. |
| `registration_summary_hash` | `artifact` | `mizar-artifact/registration-summary`, referenced summary schema version | Optional `RegistrationSummary` store-level artifact hash. |
| `registration_interface_hash` | `interface` | `mizar-artifact/registration-summary`, referenced summary schema version | Optional registration interface hash. |
| `manifest_proof_witness_entry.obligation_fingerprint` | `interface` | producer-owned, valid grammar | Must match the `ProofWitnessRef`. |
| `manifest_proof_witness_entry.witness_artifact_hash` | `artifact` | producer-owned, valid grammar | Must match the witness payload hash recorded by `ProofWitnessRef`. |
| `diagnostics_hash` | `diagnostic` | producer-owned, valid grammar | Optional diagnostics/explanation hash. |
| `development_artifact_entry.artifact_hash` | `artifact` | producer-owned, valid grammar | Optional development artifact hash. |
| `development_artifact_entry.diagnostic_hash` | `diagnostic` | producer-owned, valid grammar | Optional diagnostic payload hash. |

When a field references another mizar-artifact schema, readers validate both the
class and the schema family/version recorded by the referenced file. Producer
owned hashes keep their own family and version but must use valid artifact-framed
spelling and the required class.

## Canonical Ordering

Writers sort collections before serialization. Readers reject unsorted
collections and duplicate identity keys.

Ordering keys:

- `modules`: module identity;
- `module_artifact_entry.proof_witnesses`: `obligation_id`,
  `obligation_fingerprint`, `witness_path`;
- `development_artifacts`: `kind`, `path`, `related_module`;

For nullable `related_module`, `null` sorts before any module identity; non-null
values sort by the same module identity ordering used by `modules`.

Duplicate identity keys:

- `modules`: module identity;
- `module_artifact_entry.proof_witnesses`: `obligation_id`;
- `development_artifacts`: `kind`, `path`.

Source traversal order, worker completion order, filesystem order, and manifest
transaction staging order must not affect canonical manifest bytes.

## Reader Requirements

Manifest readers:

- read only `artifact-manifest.json`, not temporary manifest paths;
- check schema-version compatibility before interpreting fields;
- reject unknown fields, missing required fields, malformed paths, malformed hash
  strings, wrong hash classes, duplicate identities, and unsorted collections;
- reject paths that leave the artifact root after normalization;
- load referenced module artifacts only through manifest entries;
- validate referenced artifact hashes when requested by the consuming command,
  publication policy, or transaction commit path;
- when validating a module entry, require the referenced `VerifiedArtifact`
  module identity, source hash, interface hash, implementation hash, and full
  proof witness reference set to agree exactly with the manifest entry;
- never fall back to internal cache records to repair or trust a bad manifest.

Reader failures are artifact diagnostics. A missing or corrupt referenced file is
not proof evidence and is not silently replaced by a cache hit.

## Manifest Transaction Protocol

The manifest manager is serialized per package. Parallel module artifact writes
produce independent module updates; the manifest manager folds those updates
into one package-level publication transaction.

### Begin

Beginning a transaction records:

- the package identity and artifact root;
- the current manifest hash, or `null` when no manifest exists;
- the caller-supplied build snapshot freshness guard for obsolete-writer
  rejection;
- local session/snapshot/task identifiers used for diagnostics and temporary
  file names.

Local session and snapshot identifiers, including the freshness guard, are
transaction state only. They are not serialized into the published manifest and
do not participate in manifest canonical bytes. The freshness guard is opaque to
the manifest schema; it is supplied by the build coordinator that owns snapshot
ordering.

### Stage

Before commit, every staged module update must have already written, flushed,
and hash-validated its referenced files. Staged updates record final paths and
hashes only. Temporary paths must not appear in the candidate manifest.

The manager rejects a staged update if:

- a path is outside the artifact root;
- a required referenced file is missing;
- a recorded hash does not match the referenced canonical artifact, producer-owned
  payload, or referenced artifact field;
- two staged module updates have the same module identity but different
  canonical content or hashes;
- an update attempts to publish raw IR, scheduler state, internal cache records,
  or proof authority state as a manifest entry.

### Commit

Commit performs these steps:

1. Re-read the current manifest and compare its hash with the transaction's
   `base_manifest_hash`.
2. Check the transaction's snapshot freshness guard against the currently active
   package snapshot.
3. Abort if the transaction is obsolete. Abort or rebase if the base hash
   changed; a rebase must refresh staged content against the active snapshot and
   repeat the freshness check.
4. Merge unchanged current entries with staged updates, then sort all entries by
   canonical keys.
5. Serialize the candidate manifest using canonical JSON.
6. Validate every entry path and referenced hash from disk.
7. Write candidate bytes to a temporary manifest path in the artifact root.
8. Flush the temporary manifest file.
9. Atomically rename the temporary manifest over `artifact-manifest.json`.
10. Flush the containing directory when the platform supports it.
11. Publish build events only after the final manifest path names the new
    entries.

Commit success is reached only after the required rename and supported directory
flush complete. If any step before the atomic rename fails, the previous
manifest remains authoritative. If the atomic rename succeeds but the containing
directory flush fails on a platform that supports directory flushing, the commit
returns an artifact I/O diagnostic, build events are not published, cache
promotion is not allowed, and callers treat the previous manifest hash as the
last durable authority. Recovery is still manifest-first: after a crash or
reopen, consumers read and validate only the final `artifact-manifest.json` that
survived, never temporary or orphaned files. If rename and required flush
succeed, the new manifest is authoritative even if cleanup of temporary or
orphaned files fails.

Successful manifest commit is the earliest point at which cache records or cache
indexes that depend on the new publication may be promoted. `mizar-artifact`
reports manifest commit success or failure; `mizar-cache` owns cache promotion
and discard policy.

## Recovery From Interrupted Commits

Recovery is manifest-first:

- an interrupted transaction before manifest rename leaves the previous manifest
  authoritative;
- a supported directory-flush failure after manifest rename is a failed commit
  for producer, event, and cache-promotion semantics; recovery still reads only
  the surviving final manifest path and validates it;
- completed artifact or witness files that are not reachable from the committed
  manifest are ignored by readers and may be garbage-collected later;
- temporary manifest files are never publication indexes and may be removed when
  no active transaction owns them;
- if the final manifest file is missing, unreadable, or has an unsupported
  schema, consumers report an artifact diagnostic instead of scanning the
  artifact root;
- if the final manifest references missing or hash-mismatched files, consumers
  report artifact integrity diagnostics for those entries;
- cache records or cache indexes staged for a failed manifest commit remain
  staged or are discarded by `mizar-cache`; they must not become proof authority.

This recovery rule means a package is observed through exactly one complete
manifest version at a time: the previous committed manifest or the new committed
manifest.

## Deferred Implementation

Task 12 adds this specification only. Task 13 implements artifact-store writes
and corruption-detecting file reads. Task 14 implements the manifest schema,
validating reader, writer, and transaction manager. Task 17 connects real
producer projections to store and manifest publication. Broader determinism
coverage remains task 18 work.
