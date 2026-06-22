# ProofWitnessRef

> Canonical language: English. Japanese companion: [../ja/proof_witness.md](../ja/proof_witness.md).

## Purpose

`ProofWitnessRef` is a stable reference from published artifacts to external
witness files. It records enough metadata to find and hash-check accepted proof
evidence without embedding witness bytes in the main artifact.

## Schema

The reference schema version is `mizar-artifact/proof-witness-ref/v1`.

Fields:

- `schema_version`;
- `witness_id`;
- `obligation_id`;
- `path`;
- `content_hash`;
- `evidence_kind`;
- `kernel_status`;
- `kernel_profile`;
- `accepted_by`;
- `used_axioms`;

The referenced witness file has its own producer-owned schema. This crate
records and validates its bytes by hash but does not invent kernel certificate
semantics.

## Evidence Boundary

`kernel_status` may record only projected acceptance metadata supplied by
`mizar-kernel`/`mizar-proof`. `mizar-artifact` must not convert ATP success,
cache hits, or synthetic test producers into trusted proof status.

Witness bytes remain outside `VerifiedArtifact`. The main artifact stores only
`ProofWitnessRef` values and the manifest stores witness entries.

## Hash Validation

When witness bytes are available, `ProofWitnessRef` validates that the canonical
content hash matches `content_hash`. A mismatch is a hard artifact integrity
error for publication or read validation.

## Ordering

Witness references in an artifact are sorted by obligation id, witness id, path,
and content hash.
