# mizar-artifact ProofWitnessRef Schema

> Canonical language: English. Japanese companion:
> [../ja/proof_witness.md](../ja/proof_witness.md).

`ProofWitnessRef` is the stable artifact projection for proof witness files. It
lets published verified artifacts name a witness by path and hash without
embedding the witness payload in the main artifact.

This document defines the `ProofWitnessRef` schema. Task 9 introduced the
reference schema, canonical value writer, validating reader, and tests at the
`CanonicalJson` boundary. Task 23 revises the trusted witness projection from
legacy certificate acceptance to formula/substitution kernel evidence.

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
Version `1.0` was the initial certificate-reference version. Version `2.0` is
the current supported version and is the first version for formula/substitution
kernel evidence references.

## Conceptual Shape

Task 23 uses this canonical JSON field shape:

```text
proof_witness_ref = {
  "schema_version": "2.0",
  "obligation_id": string,
  "obligation_fingerprint": interface_hash_string,
  "proof_status": "kernel_verified",
  "evidence_kind": "formula_substitution_kernel_evidence",
  "witness_path": string,
  "witness_artifact_hash": artifact_hash_string,
  "kernel_acceptance": kernel_acceptance_metadata
}

kernel_acceptance_metadata = {
  "kernel_profile_fingerprint": interface_hash_string,
  "verifier_policy_fingerprint": interface_hash_string,
  "checker_schema_version": schema_version,
  "evidence_schema_version": schema_version,
  "target_binding_hash": interface_hash_string,
  "formula_evidence_hash": interface_hash_string,
  "substitution_evidence_hash": interface_hash_string,
  "provenance_hash": interface_hash_string,
  "formula_context_hash": interface_hash_string | null,
  "accepted_result_hash": interface_hash_string
}
```

`proof_status` records the accepted status already produced by the proof and
kernel phases. It does not cause acceptance. In schema version `2.0`, the only
trusted accepted status is `kernel_verified`, meaning that the kernel checked
formula/substitution evidence against its own deterministic instantiation and
SAT encoding. Externally attested, open, pending, rejected, legacy certificate,
or backend-only obligations do not produce trusted `ProofWitnessRef` values;
development-only evidence references must use a separate diagnostic or
development schema.

`evidence_kind` records the accepted evidence class. In schema version `2.0`,
the only trusted value is `formula_substitution_kernel_evidence`. Readers reject
legacy `atp_certificate`, `builtin_certificate`, `kernel_primitive`, resolution
trace, SMT proof-object, backend log, backend method, or status/evidence
combinations not listed above.

`kernel_acceptance` is a stable projection of the kernel evidence handoff and
accepted result. It records only formula/substitution/provenance/target-binding
hashes, the optional imported formula-context hash, schema versions, and policy
fingerprints. Instantiated formulas and SAT problems are not caller-supplied
trusted payloads and are not stored here. The kernel derives them from the
formula evidence and substitution evidence when it checks acceptance.

Backend proof methods, portfolio names, solver logs, resolution traces, and SMT
proof objects are not trusted witness content. If preserved, they must live in a
diagnostic or provenance attachment outside `kernel_acceptance` and outside the
accepted witness identity.

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
family and version. The task 23 fixture family is
`mizar-kernel/formula-evidence-witness`; concrete producer publication remains
owned by the future proof/producer integration.

`obligation_fingerprint`, `kernel_profile_fingerprint`,
`verifier_policy_fingerprint`, `target_binding_hash`,
`formula_evidence_hash`, `substitution_evidence_hash`, `provenance_hash`,
`formula_context_hash`, and `accepted_result_hash` use class `interface`. They
preserve the producer-owned schema family and version instead of being rewritten
into the `mizar-artifact/proof-witness-ref` domain.

No trusted hash field records a backend proof method, resolution trace,
certificate format, solver log, or used-axiom diagnostic. Those materials are
not proof authority for this artifact schema.

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

The reference and validation contract is implemented at the canonical-value
boundary. Atomic writes, manifest visibility, byte-level corruption diagnostics,
and filesystem escape checks belong to the artifact store and manifest layers.

## Resident-Set Discipline

Verified artifacts may keep `ProofWitnessRef` values resident, but they must not
inline witness payloads. Downstream builds, LSP features, documentation tools,
and cache validators may inspect resident references, hashes, proof status, and
kernel-acceptance metadata without loading proof bodies.

Consumers load the external witness file only when they explicitly audit,
diagnose, or validate a proof witness. A cache hit, manifest entry, or reference
hash is never proof authority by itself. `mizar-artifact` records that the
kernel accepted the referenced evidence and later validates that referenced
bytes still match the recorded hash; it does not recompute kernel acceptance or
run a prover.

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

Writers use the canonical UTF-8 JSON rules from `store.md` and emit the current
schema version. Readers operate over `CanonicalJson` values
produced at the store boundary; file parsing and duplicate object-key detection
belong to artifact-store I/O. Readers:

- require every field listed above, including fields whose absent value is JSON
  `null`;
- reject unknown fields at every schema object;
- check schema-version compatibility before interpreting fields;
- reject empty strings in ids, paths, and hash strings;
- reject non-publication-safe witness paths;
- validate artifact-framed hash construction labels, classes, schema-family
  grammar, schema-version grammar, and digest spelling;
- reject unsupported `proof_status`, `evidence_kind`, or status/evidence
  combinations;
- reject legacy certificate fields such as `certificate_format`, resolution
  traces, backend logs, and any caller-supplied instantiated-formula or SAT
  problem payload;
- verify `witness_artifact_hash` against a supplied witness file hash when the
  caller supplies witness bytes or a manifest hash;
- never replay proof payloads, accept proofs, run ATP, or fall back to internal
  cache records.

Reader failures are artifact diagnostics. They do not establish proof authority
and do not silently downgrade to externally attested evidence.

## Public Enum Forward Compatibility

Task 19 applies the frontend task-25 public-enum procedure to proof-witness
reference APIs. Every public enum owned by this module is a forward-compatible
API surface and must remain `#[non_exhaustive]`; downstream consumers must keep
wildcard fallback arms when matching them.

This is an API compatibility decision, not a reader leniency rule. Artifact
schema readers still reject unknown serialized enum values unless a later schema
revision and version policy explicitly document how to accept them.

Task 23 retains legacy public enum variants from schema version `1.0` for
source compatibility, but treats them as unsupported in the current trusted
schema. `DischargedBuiltin`, `AtpCertificate`, `BuiltinCertificate`, and
`KernelPrimitive` may still appear in downstream source matches, but current
writers/readers reject them through status/evidence validation.

| Enum | Forward-compatibility decision |
|---|---|
| `ProofStatus` | Non-exhaustive so accepted proof-status categories can grow under documented proof/kernel producer policy. Current trusted schema accepts only `KernelVerified`; legacy `DischargedBuiltin` is source-compatible but unsupported. |
| `EvidenceKind` | Non-exhaustive so accepted evidence classes can grow under documented proof/kernel producer policy. Current trusted schema accepts only `FormulaSubstitutionKernelEvidence`; legacy certificate/primitive variants are source-compatible but unsupported. |
| `ProofWitnessError` | Non-exhaustive so proof-witness reference validation diagnostics can grow. |

This module has no exhaustive public enum exceptions.

## Implementation Boundary

Task 23 implements schema version `2.0`, the canonical value writer, validating
`CanonicalJson` reader, and tests for round-trips, deterministic writer output,
version mismatch rejection, hash-domain validation, legacy certificate-field
rejection, and witness hash mismatch detection.

Concrete witness payload publication, proof producer integration, and full phase
15 emission remain `external_dependency_gap` items until real producer outputs
exist. Compatibility with schema version `1.0` legacy certificate references is
not retained in the normal trusted reader; any migration reader must be
explicitly audit-only and outside trusted proof acceptance.
