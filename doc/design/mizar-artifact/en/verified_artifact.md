# VerifiedArtifact

> Canonical language: English. Japanese companion: [../ja/verified_artifact.md](../ja/verified_artifact.md).

## Purpose

`VerifiedArtifact` is the stable published projection for one verified module.
It is consumed by downstream packages, documentation, LSP features, and AI
tooling. It is not a serialized compiler state.

## Schema

The schema version is `mizar-artifact/verified-artifact/v1`.

Fields:

- `schema_version`;
- `source_file`;
- `source_hash`;
- `verified_at`;
- `module_path`;
- `interface_hash`;
- `implementation_hash`;
- `exports`;
- `expressions`;
- `obligations`;
- `proof_witnesses`;
- `diagnostics`;
- `provenance`.

`verified_at` is local metadata. It is user-visible but excluded from
`interface_hash`, `implementation_hash`, and semantic reproducibility hashes.

## Projection Inputs

Producer crates fill stable projection inputs:

- exported declarations and signatures;
- expression metadata rendered as stable artifact fields;
- obligation metadata and proof status supplied by proof/kernel policy outputs;
- proof witness references;
- projected diagnostics with stable codes, source ordering, and optional
  explanation references;
- build provenance.

Forbidden inputs:

- raw `SurfaceAst`, `ResolvedAst`, `TypedAst`, `CoreIr`, `ControlFlowIr`,
  `VcIr`, `AtpProblem`, or kernel-internal state;
- opaque arena ids, task ids, memory addresses, or session-local allocator ids;
- full ATP stdout/stderr streams in the main artifact.

Readers reject unknown raw-IR-shaped fields.

## Hashes

`interface_hash` covers importer-visible exported declarations, exported proof
acceptance status, dependency-facing notation/registration summary hashes
carried through `provenance.dependency_artifact_hashes`, and schema versions.
It excludes local diagnostics, implementation-only expression metadata, proof
witness bytes, and `verified_at`.

`implementation_hash` covers the stable full verified projection except local
fields such as `verified_at` and producer-local run ids. It may change when
proof witnesses, obligation metadata, expression metadata, or projected
diagnostics change.

Diagnostics have their own stable hash input so tooling can detect diagnostic
artifact changes independently.

## Diagnostic Explanation References

Stable diagnostic explanation previews are owned by `VerifiedArtifact` as an
optional field inside each projected diagnostic:

```text
DiagnosticProjection
  code
  severity
  message
  source_order
  explanation_ref?

ArtifactRef
  path
  content_hash
  schema_version
```

The explanation bytes are development artifacts written by the store and remain
outside the main artifact. When `explanation_ref` is present, the path must be
portable and the referenced bytes must match `content_hash` before a consumer
uses the explanation. Producer-driven creation of those explanation files is
outside `mizar-artifact`; this crate owns only the stable reference schema and
hash validation contract.

## Ordering

All repeated fields are sorted by schema-specific canonical keys:

- exports by kind and FQN;
- expressions by stable expression id;
- obligations by stable obligation id;
- witnesses by obligation id and witness id;
- diagnostics by source order, stable code, and message.

## Compatibility Policy

Readers accept only supported `schema_version` values. Public schema enums are
forward-compatible and non-exhaustive in Rust APIs. Adding optional fields
requires a new compatibility rule before readers accept them.
