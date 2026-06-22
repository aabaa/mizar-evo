# VerifiedArtifact

> 正本は英語です。英語版: [../en/verified_artifact.md](../en/verified_artifact.md)。

## 目的

`VerifiedArtifact` は 1 つの verified module の安定した公開射影である。下流
packages、documentation、LSP features、AI tooling が消費する。これは serialized
compiler state ではない。

## Schema

Schema version は `mizar-artifact/verified-artifact/v1`。

Fields:

- `schema_version`。
- `source_file`。
- `source_hash`。
- `verified_at`。
- `module_path`。
- `interface_hash`。
- `implementation_hash`。
- `exports`。
- `expressions`。
- `obligations`。
- `proof_witnesses`。
- `diagnostics`。
- `provenance`。

`verified_at` は local metadata である。User-visible だが `interface_hash`、
`implementation_hash`、semantic reproducibility hashes から除外する。

## Projection inputs

Producer crate は stable projection inputs を埋める:

- exported declarations and signatures。
- stable artifact fields として render された expression metadata。
- proof/kernel policy outputs から供給された obligation metadata と proof status。
- proof witness references。
- stable codes/source ordering と optional explanation references を持つ
  projected diagnostics。
- build provenance。

Forbidden inputs:

- raw `SurfaceAst`、`ResolvedAst`、`TypedAst`、`CoreIr`、`ControlFlowIr`、
  `VcIr`、`AtpProblem`、kernel-internal state。
- opaque arena ids、task ids、memory addresses、session-local allocator ids。
- main artifact 内の full ATP stdout/stderr streams。

Reader は unknown raw-IR-shaped fields を reject する。

## Hashes

`interface_hash` は importer-visible exported declarations、exported proof
acceptance status、`provenance.dependency_artifact_hashes` 経由で保持する
dependency-facing notation/registration summary hashes、schema versions を覆う。
Local diagnostics、implementation-only expression metadata、proof witness bytes、
`verified_at` は除外する。

`implementation_hash` は `verified_at` や producer-local run ids などの local
fields を除く stable full verified projection を覆う。Proof witnesses、
obligation metadata、expression metadata、projected diagnostics が変わると変化してよい。

Diagnostics は tooling が diagnostic artifact changes を独立検出できるように
独自の stable hash input を持つ。

## Diagnostic explanation references

Stable diagnostic explanation previews は、各 projected diagnostic 内の optional
field として `VerifiedArtifact` が所有する:

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

Explanation bytes は store が write する development artifacts であり、main
artifact の外に留まる。`explanation_ref` が存在する場合、path は portable で
なければならず、consumer が explanation を使う前に referenced bytes が
`content_hash` と一致しなければならない。Producer-driven explanation file creation
は `mizar-artifact` の外であり、この crate は stable reference schema と hash
validation contract だけを所有する。

## Ordering

Repeated fields は schema-specific canonical keys で sort する:

- exports: kind と FQN。
- expressions: stable expression id。
- obligations: stable obligation id。
- witnesses: obligation id と witness id。
- diagnostics: source order、stable code、message。

## Compatibility policy

Reader は supported `schema_version` values のみ accept する。Public schema enums
は Rust API で forward-compatible / non-exhaustive とする。Optional fields の追加は
reader が受理する前に新しい compatibility rule を要求する。
