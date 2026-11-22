# MVM R3 — Runtime Artifact and Validating Loader

> Canonical English; [Japanese companion](../ja/mvm_runtime_export.md).
> Design task;
> **proposal only**, not an executable payload schema or implementation.

## Required boundary

[Spec §20.9.1](../../../spec/en/20.algorithm_and_verification.md#2091-the-mizar-virtual-machine-mvm)
requires a verified runtime artifact for each public computable algorithm,
including default-public items, with exact identity, signature, contracts,
accepted verification and executable correspondence, transitive dependencies
and compatible MVM semantics. [§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility)
governs ordered builds and reproducibility. The existing
AST-ARTIFACT splits *runtime
export* from later *computation-record provenance*. An ordinary
[VerifiedArtifact](./verified_artifact.md) export or a receipt/hash alone
is **not** runnable proof-checked code.

## Proposed ownership and validation sequence (no new schema)

| Phase owner | Input -> output boundary | Fail-closed requirement |
|---|---|---|
| Core / Checker / VC / Proof (definition-side R2) | Authenticated original algorithm definition, checked signature/body, contracts, CFG and applicable accepted proof evidence -> export-eligibility input **without a concrete ground call** | Missing/ambiguous definition, Open/tainted evidence, changed source or incomplete body/contract/CFG refuses eligibility; Artifact never decides proof validity |
| Artifact producer (R3) | Exact eligible body and dependencies -> immutable candidate executable and authenticated correspondence metadata | No text-reparse, independent unverified source lowering, placeholder bytecode or unproven private helpers; runtime erasure must preserve verified behavior |
| Artifact store / manifest (R3) | Versioned canonical content, verified identity and dependency references -> atomic published package reference | Reuse [store](./store.md) validating reads and [manifest](./manifest.md) transaction semantics; never publish raw CoreIr, proof authority, temporary or orphan files as executable exports |
| Artifact validating reader + Build/Driver (R3) | Manifest-selected executable and dependency closure -> read-only validated loader handle | Reject unknown format/VM version, mismatched hashes or source/algorithm/body/contracts/proof, stale lock/policy/snapshot, missing dependencies and traversal/symlink violations; no path or Git source fallback |
| VC MVM (R4) | Validated handle and authenticated typed concrete call -> execution report | Calls cannot reinterpret proofs, modify Artifact or manufacture E1/E2/E4; E5 only from actual normal return |
| Proof / Cache (R6–R7) | Actual MVM report and separately accepted admission -> computed/replayed status and later provenance | No raw return promoted to proof, weaker-policy cache hit, or unverified transitive dependency |

**R6-C export inputs:** register type guards, `requires`, complete `ensures`
and value-return/non-ghost Pick/set-versus-sequence loop classifications for
every closure callee, including private helpers. R6-C consumes these R3
inputs even for VC58 same-package Open-goal generation. Under the R3-N1
owner decision below, the exporter proves applicable closure uniqueness;
importers still validate its exact evidence binding.

**Visibility:** only public computable entry points may be imported and
called across packages. Their validated executable may include private
helpers, which remain invisible to importers. Cross-package *transitive*
calls require public targets and validated published artifacts. A public
non-computable algorithm remains usable through contracts. Contract-only
importers need not load runtime artifacts, but publishing a public computable
algorithm requires its verified runtime artifact; missing artifacts are an
error, not a contract-only publication mode (§20.9.1). Missing or invalid
runtime data is never repaired from dependency source. Preserve
module/version/FQN identity at each call edge.

**Order:** R2 accepted **definition/body** -> R3 reusable executable generation
and validated publication; separately typed R2 **invocation** + R3 loader ->
R4 execution -> R6 admission/R7 conversion/replay -> later
AST-ARTIFACT computation record. Publication of computed results cannot
be required to construct the initial executable, or the graph is circular.
Execution caches depend on exact executable bytes/versions, dependencies,
complete inputs/context/C and effective replay policy as required by §20.9.2;
Cache owns compatibility, not Artifact.

## Tests and decisions for the next implementation freeze

Positive *future* oracle: a genuinely accepted algorithm **without any
existing invocation** may be exported and validated; a separate authentic
caller can then load and execute that verified export. The current
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

## Freeze-preparation dossier

Q1/Q3 remain Artifact-owned (2026-11-20 11:00 JST), Q3 with Build/Proof
agreement; Q2 remains RESERVED/unassigned. The owner decisions of
2026-11-22 06:43 JST fix **N1: exporter proves** and **N2: extend the
logical-import trust model**, and assign N3 preparation here. G9 mechanism
permission (06:34 JST) authorizes preparation, not the remaining answers.
The recommendations below select no format, schema, API or owner answer.

### R3-Q1 — executable representation (open)

This is the question transferred from [R1-Q3](../../mizar-vc/en/mvm_readiness.md#open-questions).
It blocks the R3 implementation freeze and R4 format choice.

| Option | Consequences |
|---|---|
| Share extraction `RuntimeIr` | One erasure/traceability model could serve both consumers, but [RuntimeIr specification and construction](../../mizar-doc/en/todo.md) are still planned. [Extraction §20.10](../../../spec/en/20.algorithm_and_verification.md#2010-code-extraction) requires terminating algorithms; MVM also permits partial algorithms. Sharing needs explicit profiles and joint semantics/version ownership, adding an extraction dependency to R4. |
| Separate versioned Artifact executable derived from checked Core/CFG | Keeps R3/R4 independent of extraction delivery and supports the MVM profile, but requires its own instruction/value semantics, canonical encoding, erasure correspondence and compatibility checks; future extraction reuse is not automatic. |

**Recommendation for owner consideration:** the separate, initially narrow MVM
format; reuse checked Core/CFG facts rather than inventing another source
lowering. Freeze supported operations, value/call semantics, erasure and VM/
format compatibility together.

### R3-Q3 — correspondence and loader evidence (open)

Blocks R3 export. N2 fixes the imported-lineage trust basis, but does not
choose how executable correspondence is established.

| Option | Consequences |
|---|---|
| Producer establishes correspondence; authenticated publication transports it | Core/VC/Proof establish the accepted definition and extraction/erasure relation; Artifact binds exact executable bytes and dependencies to that evidence. Reader integrity and Proof authority/policy validation remain separate; no importer source-body replay is needed. This depends on genuine correspondence and accepted-result producers. |
| Loader independently reconstructs/checks correspondence | Can reduce reliance on a producer's transformation claim, but requires a checked relation/witness checker or the original checked representation, a larger reader trust surface and version coupling. Opaque source cannot be fetched as fallback; N2 lineage authentication is still required. |

**Recommendation for owner consideration:** producer-established correspondence
with a validating reader, retaining independent negative controls. The future
freeze must specify the accepted definition/body/CFG and contract identities,
erased-to-executable relation, ordered instruction/value semantics, private
helper and imported-public dependency closure, VM/format versions and current
snapshot/lock/policy binding. Validate the same immutable bytes used for
execution; hashes or signatures alone never prove the relation. Artifact owns
canonical transport/manifest versioning; Proof owns acceptance/authentication,
and Build/Driver consume only a validated handle.

### R3-N3 — computability producer (open)

Blocks computable-public export eligibility under §20.9.1.

| Option | Consequences |
|---|---|
| Frontend/Checker alone | Has resolved types, visibility and operations, but lacks the full accepted body/executable dependency relation; a syntax or type flag cannot certify export readiness. |
| Core/VC definition-and-closure assessment, with Proof acceptance; Artifact consumes it | Reuses checked bodies and call/classification facts, including private helpers and imported exports. Requires a complete, version-bound assessment; Checker/Resolver supply facts, Artifact enforces publication. |
| Artifact infers computability from encodable instructions | Close to the format, but confuses backend support with language computability and duplicates semantic analysis; successful serialization is insufficient. |

**Recommendation for owner consideration:** the Core/VC assessment route,
separating semantic computability, accepted verification and current backend
support. Exact producer/API ownership remains for the owner freeze.
[§20.9.3](../../../spec/en/20.algorithm_and_verification.md#2093-computability-and-runtime-failures)
allows input-dependent runtime failures without invalidating a definition;
eligibility promises neither totality nor successful execution on every input.
Unsupported or incomplete evidence gives no executable handle, and must not
silently reclassify a computable public algorithm as contract-only.
The owner must distinguish declaration-wide from input-dependent computability,
including unreachable operations, template implementations and Pick enumerators;
this dossier supplies no new language criterion for those cases.

### R3-Q2 — ordering facts only (RESERVED)

Blocks R3 export and the R4 same-module pilot. Same-file computation examples
in §20.9.2/§20.13.4 coexist with file-final Artifact emit (§23.4).
The owner still must determine whether fully verified, separately authenticated
staged runtime items may load before public manifest commit or evaluation must
await a published verified dependency; defining-file private-entry evaluation
also remains reserved. The [logical transport design](./manifest.md#producer-transport-design)
authenticates committed publication, so it supplies no pre-commit runtime
authority by itself. Required facts for that decision are the earliest complete
R2 acceptance point, dependency/group order, staging lifetime/currentness and
failure rollback, and a non-circular proof of readiness. Neither an unverified
staging token nor a proof that depends on its own future execution is acceptable.

### R3-N1/N2 — decided inputs and remaining evidence

- **N1: exporter proves closure uniqueness.** Exporter evidence must account
  for applicable value-returning callees, including opaque private helpers,
  with exact typed guards, `requires`, complete `ensures`, triggers and
  proof or spec-exact exemption bindings. Importers validate those bindings,
  dependencies and policy; they do not generate substitute private-body proofs.
  This does not make every algorithm require uniqueness: R6's trigger rules
  remain unchanged, and final admission checks the actual evaluated closure.
- **N2: extend the logical-import trust model.** The [Proof trust-transfer rule](../../mizar-proof/en/policy.md#trust-transfer-rule)
  and [Artifact transport boundary](./manifest.md#imported-promotion-authority-boundary)
  supply importer-authorized authority, accepted-result-only committed
  publication and exact object/group/dependency/policy binding. Runtime use
  must additionally bind executable content, VM semantics and correspondence;
  logical-only import still needs no runtime artifact. The existing transport
  is a design, not an implemented runtime validator or accepted R3 fixture.

### Consequences and next freeze prerequisites

| Consumer | Consequence of the proposals and fixed N1/N2 inputs |
|---|---|
| [R4](../../mizar-vc/en/mvm_executor.md) | Needs Q1 semantics, Q3 validated bytes/closure and N3 eligibility plus a separately typed genuine invocation. Q2 continues to block the same-module pilot; actual normal return alone yields E5, not Proof admission. |
| [R6](../../mizar-vc/en/mvm_computation_admission.md) | Local A/B preparation may proceed; C still requires R3 exports. N1 moves uniqueness proof production to the exporter, while D validates complete evaluated closure and exact accepted proof/exemption under N2; neither static computability nor an R4 return bypasses admission. |
| [R7](../../mizar-proof/en/mvm_computation_import.md) | Must preserve N2 lineage and Q1/Q3 exact executable/dependency/VM identities with real E5, R6 and conversion inputs. Replay needs its separate equation/kernel basis; later computation records cannot authorize initial R3 export. |

Before an Artifact-owned implementation freeze under AST-ARTIFACT: obtain
owner Q1/Q2/Q3/N3 dispositions, genuine complete [R2 definition evidence](../../mizar-core/en/mvm_correspondence.md),
exporter N1 evidence and the N2 runtime-extension contract; freeze compatible
format/manifest/loader checks and the future positive/negative matrix above.
R2's current accepted-set preparation retains Open/Deferred residuals and is
not complete export eligibility. This dossier grants no implementation,
runtime export, computation acceptance, new coverage or language-rule change.
