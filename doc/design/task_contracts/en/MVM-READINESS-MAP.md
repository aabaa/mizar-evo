# Task MVM-READINESS-MAP: Authentic MVM first-slice readiness inventory

Canonical language: English; [Japanese pointer](../ja/MVM-READINESS-MAP.md).
Status: planned, documentation-only; ready for Muse dispatch under the MVM
preparation fallback in the [roadmap](../../todo.md#mvm-readiness-decomposition--).
Tier: full documentation/review gates. Sole task owner:
[mizar-vc plan](../../mizar-vc/en/00.crate_plan.md). The inventory coordinates
Core, Checker, Artifact, Build and Test without assuming their authority.
Dependencies: existing specification, currently committed owner contracts and
source/tests; no unimplemented runtime producer is a prerequisite for this
read-only inventory.
Authority: [specification §20.9](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability),
[§20.13.1 E1-E5](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms),
[§20.13.7](../../../spec/en/20.algorithm_and_verification.md#20137-certified-computation-import-t1t8),
[AST-LOWER](AST-LOWER.md), [AST-CONTRACT](AST-CONTRACT.md),
[AST-ARTIFACT](AST-ARTIFACT.md), [ALG-MVM-EVALUATION](ALG-MVM-EVALUATION.md),
[AST-COMPUTE](AST-COMPUTE.md), [ALGDEF-COMPUTATION-ADMISSION](ALGDEF-COMPUTATION-ADMISSION.md),
and [autonomous protocol](../../autonomous_crate_development.md).

## Scope and deliverable

1. Inventory the actual implemented and missing source-owned path from one
   smallest spec-supported ground/direct algorithm call: original Frontend /
   Resolver / Checker identity and types, CoreIr/ControlFlowIr verified body,
   source-derived contract/normal-return evidence, executable correspondence,
   published executable, validated loader and actual evaluation result. Cite
   exact existing source, test, specification, and owner files for each edge.
   Do not infer that a declaration, open computation request, synthetic fixture,
   or existing theorem-reflexivity artifact is an executable algorithm.
2. Choose the smallest honest pilot case if an existing source fixture and
   acceptance oracle support it; otherwise record the exact missing producer
   and a proposed test-first case derived from the specification, not from
   implementation output. Distinguish plain ordinary ground values from
   reflected syntax/Binder/context-C input; do not exempt a case from an
   existing AST-TYPE/AST-LOWER or certification gate without a reviewed,
   spec-grounded owner decision.
3. Map exact prerequisites and owners for the roadmap's MVM-R2 through MVM-R8
   slices: checked body/call association (Core), executable export/loader
   (Artifact), evaluator (VC), failure/budgets and calls (VC), full computation
   closure (VC/Proof), certified import/replay (Proof), and stage-specific
   corpus (Test). Confirm that runtime export precedes MVM execution, whereas
   computation-record publication follows AST-COMPUTE; never invert these
   dependencies. Keep algorithm-execution and proof-acceptance claims distinct.
4. Record the owner-resolved matrix in
   `doc/design/mizar-vc/en/mvm_readiness.md`, with a synchronized Japanese
   companion or pointer under `ja/`. Each row must say implemented,
   ready-for-contract, dependency-blocked, or open-question, its exact
   prerequisite, producer, consuming owner, and negative control. Add
   separate paired task contracts and crate-plan entries **only** for a
   genuinely ready new implementation unit, never for a fabricated API.

## Non-goals and safety

No Rust, existing `.miz`/expectation/trace status, coverage promotion,
public API, kernel change, MVM implementation, new crate, source fallback,
unverified executable, invented foundational library proof or change to
`doc/spec`. This task does not implement AST formation, normal-return
verification, MVM or `by computation`. Missing premises remain blocked.

## Acceptance

A cross-owner dependency DAG with a supported pilot or exact blocker;
stage-specific positive, rejection and provenance/corruption tests identified
without changing protected fixtures; no circular export/compute dependency;
distinct trust/visibility/cache boundaries; each immediate successor has
an unambiguous single owner and a freeze trigger. Independent specification,
boundary, test-sufficiency, volume/scope and EN/JA documentation review
reports no unresolved blocking finding. Run both repository `lint_policy`
suites, local link/fragment checks and `git diff --check` for this
documentation-only task. Record verification evidence in the commit/handoff,
not in the roadmap. No runtime coverage or feature-completion credit.
