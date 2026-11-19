# MVM R3 — Runtime Artifact and Validating Loader

> Canonical English; [Japanese companion](../ja/mvm_runtime_export.md).
> [Design task](../../task_contracts/en/MVM-R3-ARTIFACT-LOADER-DESIGN.md);
> **proposal only**, not an executable payload schema or implementation.

## Required boundary

[Spec §20.9.1](../../../spec/en/20.algorithm_and_verification.md#2091-the-mizar-virtual-machine-mvm)
requires a verified runtime artifact for each public computable algorithm,
including default-public items, with exact identity, signature, contracts,
accepted verification and executable correspondence, transitive dependencies
and compatible MVM semantics. [§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility)
governs ordered builds and reproducibility. The existing
[AST-ARTIFACT](../../task_contracts/en/AST-ARTIFACT.md) splits *runtime
export* from later *computation-record provenance*. An ordinary
[VerifiedArtifact](./verified_artifact.md) export or a receipt/hash alone
is **not** runnable proof-checked code.

## Proposed ownership and validation sequence (no new schema)

| Phase owner | Input -> output boundary | Fail-closed requirement |
|---|---|---|
| Core / Checker / VC / Proof (R2) | Original selected body, typed signature/call, contracts, CFG and *accepted applicable* verification evidence -> authenticated executable-eligibility input | Missing body, ambiguous call, Open/tainted evidence, changed source/context or incomplete contract/CFG refuses eligibility; Artifact never decides proof validity |
| Artifact producer (R3) | Exact eligible body and dependencies -> immutable candidate executable and authenticated correspondence metadata | No text-reparse, independent unverified source lowering, placeholder bytecode or unproven private helpers; runtime erasure must preserve verified behavior |
| Artifact store / manifest (R3) | Versioned canonical content, verified identity and dependency references -> atomic published package reference | Reuse [store](./store.md) validating reads and [manifest](./manifest.md) transaction semantics; never publish raw CoreIr, proof authority, temporary or orphan files as executable exports |
| Artifact validating reader + Build/Driver (R3) | Manifest-selected executable and dependency closure -> read-only validated loader handle | Reject unknown format/VM version, mismatched hashes or source/algorithm/body/contracts/proof, stale lock/policy/snapshot, missing dependencies and traversal/symlink violations; no path or Git source fallback |
| VC MVM (R4) | Validated handle and authenticated typed concrete call -> execution report | Calls cannot reinterpret proofs, modify Artifact or manufacture E1/E2/E4; E5 only from actual normal return |
| Proof / Cache (R6–R7) | Actual MVM report and separately accepted admission -> computed/replayed status and later provenance | No raw return promoted to proof, weaker-policy cache hit, or unverified transitive dependency |

**Visibility:** only public computable entry points may be imported and
called across packages. Their validated executable may include private
helpers, which remain invisible to importers. Cross-package *transitive*
calls require public targets and validated published artifacts. A public
signature without a verified executable remains usable through contracts,
not by computation; missing or invalid runtime data is never repaired from
dependency source. Preserve module/version/FQN identity at each call edge.

**Order:** R2 accepted body/call -> R3 executable generation and validated
publication -> R4 execution -> R6 admission/R7 conversion/replay -> later
AST-ARTIFACT computation record. Publication of computed results cannot
be required to construct the initial executable, or the graph is circular.
Execution caches depend on exact executable bytes/versions, dependencies,
complete inputs/context/C and effective replay policy as required by §20.9.2;
Cache owns compatibility, not Artifact.

## Tests and decisions for the next implementation freeze

Positive *future* oracle: only an actually accepted matching algorithm with
a validating executable/manifest can be loaded and called. The current
[`idalgo` corpus](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
has an [Open VC expectation](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml);
it is **not** an eligible positive runtime fixture. Compare equivalent
repeated reads and canonical hashes independently of build scheduling.

Negative matrix: mutate body, ordered statement, type/signature, requires/
ensures, selected proof receipt, module/source/lock/version, VM version,
private/public entry, private helper closure and transitive import.
Reject truncated/malformed/duplicate-key/unreachable content, forged
manifest hash, stale publication, missing dependency, policy incompatibility,
or local source fallback. Refusal yields no validated handle, E5 report or
trusted computation status; retain the previous durable manifest on failure.

**Open decisions (no approved format):**
- **R3-Q1:** versioned executable instruction/serialization model and
  validation of its exact semantics against checked Core/CFG; code-extraction
  [RuntimeIr](../../mizar-doc/en/todo.md) is separate and not an implicit
  MVM runtime format.
- **R3-Q2:** same-package verification-time evaluation ordering. Determine
  whether a separately authenticated, fully verified staged runtime item
  can be loaded before public manifest commit, or whether evaluation must
  await a published verified dependency. Never accept a self-dependent
  proof/verification/publication cycle or unverified staging token.
- **R3-Q3:** which producer-owned evidence, complete dependency closure and
  reader checks demonstrate correspondence; precise format/manifest
  integration and versioning must be frozen by Artifact/Build/Proof owners.

**Exit for design review:** agree ownership, acyclic lifecycle, version and
correspondence choices with authentic R2 evidence. Only then freeze a
new Artifact-owned implementation slice under AST-ARTIFACT. R2 remains
review-pending/implementation-blocked; R3 and MVM execution are not ready.
