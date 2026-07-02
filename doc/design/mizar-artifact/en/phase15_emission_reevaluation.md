# mizar-artifact Phase 15 Emission Reevaluation

> Canonical language: English. Japanese companion:
> [../ja/phase15_emission_reevaluation.md](../ja/phase15_emission_reevaluation.md).

## Scope

Task 17 was reevaluated after task 23 completed the artifact-side
formula/substitution proof-witness schema. This reevaluation decides whether
`mizar-artifact` can now wire full `VerifiedArtifact` emission from real
producer projections.

Classification result:

- `external_dependency_gap`: phase 15 emission still lacks real producer-owned
  projection outputs and proof witness publication.
- `deferred`: task 17 implementation remains deferred.
- `spec_gap`: none found in the crate-owned artifact schema.
- `test_gap`: none opened for crate-owned behavior.

## Findings

Task 23 supplies the artifact-side witness projection needed by future emission:

- `ProofWitnessRef` schema version `2.0`;
- `formula_substitution_kernel_evidence` as the only trusted current evidence
  kind;
- target binding, formula evidence, substitution evidence, provenance, optional
  formula context, and accepted result hashes;
- normal trusted-reader rejection for legacy certificate, resolution-trace,
  backend log, backend method, SMT proof-object, instantiated-formula, and SAT
  problem payloads.

The remaining task 17 blockers are outside `mizar-artifact`:

- `mizar-proof` now exists and owns witness staging/publication metadata, but
  there is still no integrated producer-owned witness publication output to
  feed the artifact store and manifest;
- no real producer output exists for full `VerifiedArtifact` emission;
- no producer-owned publication hook exists to connect checked kernel evidence,
  selected proof status, witness files, and manifest entries.

Implementing task 17 now would require placeholder proof authority, invented
producer projections, or fake witness publication. Those are forbidden by the
artifact boundary and by the evidence-pipeline correction.

## Disposition

Task 17 remains deferred as `external_dependency_gap`. No source stub,
placeholder crate, fake witness schema, fake producer projection, or artifact
publication shim is added.

The next valid task 17 attempt requires all of the following to exist first:

- real producer-owned `VerifiedArtifact` projection inputs;
- stable proof/witness staging and manifest publication outputs from the
  owning proof/artifact integration;
- integration tests that exercise real emission without changing proof
  authority inside `mizar-artifact`.

Until those dependencies exist, `mizar-artifact` owns only the stable schemas,
canonical writers/readers, hash validation, store primitives, and manifest
transactions.

## Verification

This task is documentation-only. Required verification is:

```text
git diff --check
git diff --cached --check
```

Rust verification is not required for this task because no Rust source changes
are made.
