# mizar-artifact ProofWitnessRef Schema

> Canonical language: English. Japanese companion:
> [../ja/proof_witness.md](../ja/proof_witness.md).

`ProofWitnessRef` is the stable artifact projection for proof witness files. It
lets published verified artifacts name a witness by path and hash without
embedding the witness payload in the main artifact.

This document defines the `ProofWitnessRef` schema. Task 9 implements the
schema, canonical value writer, validating reader, and tests at the
`CanonicalJson` boundary.

## Ownership

The proof-witness reference schema owns:

- the stable reference shape used by `VerifiedArtifact` proof obligations;
- package-artifact-root-relative witness paths and path validation rules;
- artifact-framed hashes for witness files and proof-related fingerprints;
- projected kernel-acceptance metadata needed by artifact readers and cache
  validators;
- the resident-set rule that witness payloads are loaded on demand.

It does not own:

- ATP search, portfolio winner selection, or backend certificate parsing;
- kernel replay, kernel acceptance, or proof authority;
- the byte schema of witness payload files;
- internal cache records or proof-reuse decisions;
- manifest transactions or artifact-store file I/O.

The schema family for references is `mizar-artifact/proof-witness-ref`.
Version `1.0` is the first supported version.

## Conceptual Shape

Task 9 uses this canonical JSON field shape:

```text
proof_witness_ref = {
  "schema_version": "1.0",
  "obligation_id": string,
  "obligation_fingerprint": interface_hash_string,
  "proof_status": "kernel_verified" | "discharged_builtin",
  "evidence_kind": "atp_certificate" | "builtin_certificate" | "kernel_primitive",
  "witness_path": string,
  "witness_artifact_hash": artifact_hash_string,
  "kernel_acceptance": kernel_acceptance_metadata
}

kernel_acceptance_metadata = {
  "kernel_profile_fingerprint": interface_hash_string,
  "verifier_policy_fingerprint": interface_hash_string,
  "checker_schema_version": schema_version,
  "certificate_format": string | null,
  "accepted_result_hash": interface_hash_string,
  "used_axioms_hash": diagnostic_hash_string | null
}
```

`proof_status` records the accepted status already produced by the proof and
kernel phases. It does not cause acceptance. `kernel_verified` requires an ATP
certificate accepted by the minimum kernel. `discharged_builtin` requires an
accepted built-in certificate or allowed kernel primitive. Externally attested,
open, pending, or rejected obligations do not produce trusted
`ProofWitnessRef` values; future development-only evidence references must use a
separate diagnostic or development schema.

`evidence_kind` records the accepted evidence class. `atp_certificate` is valid
only with `proof_status = "kernel_verified"`. `builtin_certificate` and
`kernel_primitive` are valid only with `proof_status = "discharged_builtin"`.
Readers reject other combinations.

`certificate_format` is present as a non-empty string when the witness payload is
format-specific, for example an ATP certificate format. It is JSON `null` for
allowed kernel primitives that do not have a certificate file format. The exact
format vocabulary is owned by the proof and kernel crates, not by
`mizar-artifact`. Task 9 readers require a non-empty `certificate_format` for
`atp_certificate` and `builtin_certificate`, and require JSON `null` for
`kernel_primitive`.

## Hash String Domains

Task 9 uses the task-3 artifact-framed hash string form:

```text
mizar-artifact/artifact-framed-hash-text/v1:<class>:<schema_family>:<schema_version>:<digest>
```

`<digest>` is a 64-character lowercase hexadecimal digest. Artifact-framed
`schema_family` strings follow the same grammar as `registration_summary.md`:
they are non-empty, colon-free, slash-separated identifiers whose segments use
only ASCII letters, ASCII digits, hyphen, underscore, or dot.

`witness_artifact_hash` uses class `artifact` and the witness payload schema
family and version. The task 8 reference spec does not define the witness payload
schema family; that is an `external_dependency_gap` for the proof/kernel witness
producer work.

`obligation_fingerprint`, `kernel_profile_fingerprint`,
`verifier_policy_fingerprint`, and `accepted_result_hash` use class
`interface`. They preserve the producer-owned schema family and version instead
of being rewritten into the `mizar-artifact/proof-witness-ref` domain.

`used_axioms_hash` uses class `diagnostic` because it supports citation
refinement and diagnostics. It is not proof authority and does not replace
kernel acceptance metadata.

## Witness Paths And Publication

`witness_path` is relative to the package artifact root and must normalize under
`proof-witnesses/`. Task 9 performs lexical path validation at the
`CanonicalJson` boundary: the path must use `/` separators, start with
`proof-witnesses/`, contain at least one non-empty child segment after that
prefix, and contain no empty, `.`, or `..` segments. It must not be absolute.
Symlink and artifact-root escape checks require filesystem access and remain
part of artifact-store I/O. The manifest is the publication index: a witness
file is not published merely because it exists under `proof-witnesses/`.

The manifest transaction writes and hashes witness files before the manifest is
committed. A `ProofWitnessRef` becomes publication-reachable only when a
committed manifest entry references both the main artifact and the witness file.
If publication policy requires a witness, a missing file or hash mismatch rejects
publication for the affected artifact.

Task 9 implements the reference and validation contract at the canonical-value
boundary. Atomic writes, manifest visibility, byte-level corruption diagnostics,
and filesystem escape checks remain deferred to the artifact store and manifest
tasks.

## Resident-Set Discipline

Verified artifacts may keep `ProofWitnessRef` values resident, but they must not
inline witness payloads. Downstream builds, LSP features, documentation tools,
and cache validators may inspect resident references, hashes, proof status, and
kernel-acceptance metadata without loading proof bodies.

Consumers load the external witness file only when they explicitly replay,
audit, diagnose, or validate a proof witness. A cache hit, manifest entry, or
reference hash is never proof authority by itself. Trust comes only from the
proof/kernel phases that produced the accepted status and from later validation
that the referenced bytes still match the recorded hash.

## Canonical Ordering

When a `VerifiedArtifact` contains more than one `ProofWitnessRef`, the
collection sorts by:

1. `obligation_id`;
2. `obligation_fingerprint`;
3. `proof_status`;
4. `evidence_kind`;
5. `witness_path`;
6. `witness_artifact_hash`.

Readers reject duplicate `obligation_id` values within one verified artifact
unless a later schema explicitly supports multiple accepted witnesses for a
single obligation. Source traversal order, ATP completion order, backend runtime,
and filesystem order must not affect serialized bytes.

## Reader And Writer Requirements

Task 9 writers use the canonical UTF-8 JSON rules from `store.md` and emit the
current schema version. Task 9 readers operate over `CanonicalJson` values
produced at the store boundary; file parsing and duplicate object-key detection
belong to artifact-store I/O. Readers:

- require every field listed above, including fields whose absent value is JSON
  `null`;
- reject unknown fields at every schema object;
- check schema-version compatibility before interpreting fields;
- reject empty strings in ids, paths, certificate formats, and hash strings;
- reject non-publication-safe witness paths;
- validate artifact-framed hash construction labels, classes, schema-family
  grammar, schema-version grammar, and digest spelling;
- reject unsupported `proof_status`, `evidence_kind`, or status/evidence
  combinations;
- verify `witness_artifact_hash` against a supplied witness file hash when the
  caller supplies witness bytes or a manifest hash;
- never replay proof payloads, accept proofs, run ATP, or fall back to internal
  cache records.

Reader failures are artifact diagnostics. They do not establish proof authority
and do not silently downgrade to externally attested evidence.

## Implementation Boundary

Task 9 implements the `ProofWitnessRef` schema, canonical value writer,
validating `CanonicalJson` reader, and tests for round-trips and hash mismatch
detection.

Concrete witness payload schemas, proof producer integration, accepted kernel
result construction, and built-in certificate/primitive encodings remain
`external_dependency_gap` items until the proof and kernel crates expose stable
producer outputs. Manifest/file I/O remains deferred to the artifact-store and
manifest tasks.
