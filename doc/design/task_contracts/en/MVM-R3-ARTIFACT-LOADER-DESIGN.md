# Task MVM-R3-ARTIFACT-LOADER-DESIGN: Verified runtime export and loader boundary

Canonical English; [Japanese pointer](../ja/MVM-R3-ARTIFACT-LOADER-DESIGN.md).
Status: design draft; independent review and documentation checks pending.
**Documentation only**; not an executable-format freeze or R3 implementation.
Sole owner: [mizar-artifact](../../mizar-artifact/en/00.crate_plan.md).
Design: [Artifact owner document](../../mizar-artifact/en/mvm_runtime_export.md).
Consumers: Build/Driver loader, VC-owned MVM, Proof computation import, Cache.
Authority: [spec §20.9.1](../../../spec/en/20.algorithm_and_verification.md#2091-the-mizar-virtual-machine-mvm),
[§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step),
[§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility);
[AST-ARTIFACT](AST-ARTIFACT.md),
[R2 correspondence design](MVM-R2-CORRESPONDENCE-DESIGN.md),
[existing manifest](../../mizar-artifact/en/manifest.md) and
[store](../../mizar-artifact/en/store.md).
Readiness: design can be reviewed now; implementation/export blocked by
R2 accepted **definition/body** correspondence (not a concrete ground call)
and an approved executable format/loader.

## Scope

- Describe a reusable, owner-verified immutable algorithm executable,
  independent of any concrete caller, bound to the accepted source body,
  public signature and contracts, precise
  proof/verification lineage, compatible VM version, and complete transitive
  executable dependencies. Do not equate a VerifiedArtifact metadata row
  with runnable bytes.
- Separate producer, manifest/store publisher, reader, Build/Driver loader,
  MVM executor, Proof acceptor and Cache policy boundaries. Determine
  supported public entry/private-helper and imported public-call rules,
  source-fallback refusal, reproducibility and fail-closed validation.
- Identify R3 freeze decisions: executable representation/version; trust/
  correspondence evidence and mandatory reader recomputation; intra-package
  staging/verification order; reuse versus separation from code-extraction
  RuntimeIr. Propose tests without assigning an unreviewed schema or API.
- Preserve the acyclic order: R2 verified body -> R3 runtime export/loader
  -> R4 real execution -> R6 accepted computation admission/R7 import
  -> later AST-ARTIFACT computation-record publication. No stage may
  consume evidence from its own future stage.

## Acceptance and exclusions

Deliver a compact EN/JA design with an owner/dependency table,
negative controls, honest unavailable states, and specific release
conditions for the next Artifact-owned implementation freeze. Reject
forged/missing/stale bytes, wrong body/contracts, visibility, version,
dependencies, snapshot/manifest and policy, with no unverified fallback.
Neither a proof status alone nor hashing unchecked code grants validity.
Existing .miz/expectations, source, schema, completion/coverage and
trust authority remain unchanged; no Rust, new crate, bytecode, backend,
new diagnostic codes, extra manifest entry or MVM execution.
Independent specification, test-sufficiency, boundary, volume, EN/JA
reviews and required documentation lint/link checks precede completion.
