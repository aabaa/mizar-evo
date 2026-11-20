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
inputs even for VC58 same-package Open-goal generation. This registration
does not decide whether the exporter proves and transports uniqueness or
the importer generates its obligations; see R3-N1 below.

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

**Open decisions (all undecided; no approved format):**
R3-Q1/Q3 ownership follows the owner decision of 2026-11-20 11:00 JST.
- **R3-Q1 — Artifact; blocks R3 implementation freeze and R4 format choice:**
  versioned executable instruction/serialization model and exact semantics
  against checked Core/CFG, including whether to share code-extraction
  [RuntimeIr](../../mizar-doc/en/todo.md) or use a separate format. This is
  the open question transferred from [R1-Q3](../../mizar-vc/en/mvm_readiness.md#open-questions);
  neither representation is selected before the future Artifact-owned freeze.
- **R3-Q2 — RESERVED/undecided owner/specification judgment at the R3 freeze;
  unassigned in this slice; blocks R3 export and the R4 same-module pilot:**
  same-package verification-time evaluation ordering. Determine
  whether a separately authenticated, fully verified staged runtime item
  can be loaded before public manifest commit, or whether evaluation must
  await a published verified dependency. The same-file computation examples
  (§20.9.2/§20.13.4) and file-final Artifact emit (§23.4) constrain this
  choice; defining-file private-entry evaluation also remains undecided.
  Never accept a self-dependent
  proof/verification/publication cycle or unverified staging token.
- **R3-Q3 — Artifact, with Build/Proof agreement; blocks R3 export:**
  which producer-owned evidence, complete dependency closure and reader
  checks demonstrate correspondence, with what format/manifest versioning?
  **R3-N2:** what authenticates imported verification lineage (exporter-side
  E1/E2/applicable E4 acceptance)? Relate the [Proof trust-transfer rule](../../mizar-proof/en/policy.md#trust-transfer-rule)
  and [Artifact transport boundary](./manifest.md#imported-promotion-authority-boundary)
  ([producer transport design](./manifest.md#producer-transport-design))
  without deciding whether that authenticated publication is required for
  runtime artifacts; computation retains separate correspondence checks.
- **R3-N1 — registered in R3; blocks R3 export inputs for R6-C:** does the
  exporter prove and transport closure uniqueness, or does the importer
  generate it, including for private helpers whose contracts are opaque?
- **R3-N3 — registered in R3; blocks R3 computable-public export eligibility:**
  which producer determines that an algorithm is computable (§20.9.1)?
  The producer and its ownership remain undecided. N1–N3 registration follows
  the same owner decision; it does not settle their implementation ownership.

**Exit for design review:** independently review these boundaries and record
the undecided questions, their dispositions and blocked targets. Implementation
still requires the version/correspondence decisions and authentic R2 evidence
before a new Artifact-owned freeze under AST-ARTIFACT. R2 review is closed;
its implementation, R3 export and MVM execution remain blocked.
