# STEP 5 Source-Derived VC Family Decomposition

> Canonical language: English. Japanese companion:
> [../ja/source_vc_decomposition.md](../ja/source_vc_decomposition.md).

This document is the accepted output of mizar-vc Task 30. It freezes the one
exact Task-180 theorem-obligation-to-VC mapping implemented by Task 31 and
decomposes every other source-derived VC family into bounded follow-up tasks.
The decomposition remains task, dependency, and consumer authority; the Task
31 completion record below names its exact implementation and coverage without
broadening any later family.

## Authority And Entry Baseline

The inventory follows the repository authority order:

1. `doc/spec/en/`;
2. existing `.miz` sources;
3. `tests/coverage/spec_trace.toml`;
4. existing expectation sidecars;
5. design documents;
6. source as non-normative inventory evidence.

Canonical Chapters 03-21 apply as cited by each task below. The accepted
upstream task authority is checker Task 247's
[payload-family decomposition](../../mizar-checker/en/payload_family_decomposition.md)
and Core Task 32's
[source-family decomposition](../../mizar-core/en/source_family_decomposition.md).
Core Task 31 is the sole exact Task-180 Core exception. Core Tasks 33-53,
Gate A1/MC-G004, and Gate S1 remain unimplemented dependencies rather than
available payloads.

Task 30 preserves the clean-entry behavioral oracles: active runners parse 96,
declaration 4, and type elaboration 188; plan 403/368; type elaboration
236/224; pass/fail 219/184; warnings/errors 23/0; 272 `mizar-test` unit tests;
and 17 production paths / 19,803 lines. The CLI, test-list, and production
path/content hashes recorded by Core Task 32 must remain unchanged. Task 30
adds no trace requirement; it may change only existing deferred ownership text.

## Contract Shared By Descendant Tasks

Each row below is one nonempty logical task and one commit. Before editing, the
task must freeze the exact canonical sections, source family, syntax-free Core
and control-flow inputs, VC consumer, visibility, generated formula and
context representation, forbidden scope, tests, trace impact, and exit
criteria.

Unless a row narrows the rule, every source-derived VC task must:

- consume validated `CoreIr`, `ControlFlowOutput`, and obligation handoff data;
  never inspect raw syntax or reconstruct checker/Core semantics in `mizar-vc`;
- preserve source/module/owner identity, local path, semantic origin, seed
  status, source maps, context order, and versioned provenance transactionally;
- use a complete, replayable generated formula or substitution representation
  when the canonical obligation is not already a Core formula;
- keep `VcId` snapshot-local and assign dense deterministic ids only after
  validation and normalization;
- keep anchors honest: unavailable canonical formula, context, trace, or
  dependency identity remains explicitly incomplete and proof-reuse-ineligible;
- account for every input seed exactly once as zero, one, or many VCs under the
  existing intake policy, without silently turning incomplete deferred input
  into an open VC;
- fail closed on missing, duplicate, reordered, stale, recovered,
  diagnostic-bearing, cross-owner, cross-module, wrong-kind/status/goal/path,
  partial-context, or corrupt-provenance input;
- add deterministic rerun, immutable whole-`VcSet` comparison, complete
  `VcSet::debug_text()` snapshot, and task-local corruption coverage;
- add executable specification-derived real-source assertions for every newly
  claimed VC family, kind, phase, and semantic branch; one smallest fixture may
  cover several claims only when it asserts each one separately;
- add the smallest real-source zero-VC or near-miss case for every non-generation
  boundary claimed by the row, and the smallest real-source negative case for
  every newly claimed diagnostic family;
- activate only a narrow task-local trace row when its producer, consumer, and
  real baseline land. The broad proof-verification rows remain deferred.

All tasks forbid proof search, deterministic discharge unless separately
authorized, `NeedsAtp` policy execution, ATP/kernel/proof/cache/artifact work,
theorem or algorithm acceptance, fact publication, extraction/MVM execution,
fabricated upstream evidence, expectation rebaselining, and Steps 6/7
promotion.

## Exact Task-180 Mapping For VC Task 31

Task 31 owns a narrow Core-aware adapter. It borrows and validates the exact
Core Task-31 `CoreIr`; it does not mutate Core data or insert a synthetic
`vc-proof-goal:terminal` provenance marker.

The accepted input has exactly one public structurally valid theorem item, one
`False` formula, one `PendingAutomaticProof`, one direct `TerminalGoal`, and one
Active `TheoremProof` seed at `proof/0`. Context, label, citations, diagnostics,
terms, definitions, algorithms, and generated rows are empty. Source maps,
Core references, semantic origin, and checker/proof-skeleton provenance must
match the committed Core Task-31 contract.

Task 31 must build and validate an empty `ControlFlowOutput`, then validate one
`ExistingCore { seed: 0 }` handoff/source-map row and a freshly recomputed
`EligibleOneVc { goal: CoreFormulaId(0) }` intake row. The authenticated direct
proof-node-to-obligation-to-goal relation, not marker text, classifies the
candidate as:

- exactly one dense `VcId(0)` with seed handoff 0;
- `VcKind::TerminalProofGoal` and `VcStatus::Open`;
- owner `AnchorOwner::Theorem(CoreItemId(0))`;
- goal `VcFormulaRef::Core(CoreFormulaId(0))`;
- primary source, local path `proof/0`, and semantic origin preserved;
- empty local context, premises, proof hint, related-source list, and generated
  formula table;
- the existing Core-handoff, generator, and normalization provenance chain;
- one Active/ExistingCore seed-accounting row mapped to that VC.

The source-shape and empty-context anchor hashes are available. Because the
current VC payload transports only the Core formula id, the canonical-goal hash
remains conservatively unavailable and the anchor remains
`Incomplete { missing: [CanonicalGoalHash] }`. Prevalidating this exact formula
as `False` does not authorize a fabricated hash or proof reuse.

The adapter rejects every extra, missing, duplicate, reordered, stale,
cross-owner, wrong-status/kind/goal/path/source/provenance/Core-reference, or
proof-link row. `PendingAutomaticProof` to `Open` is only the phase mapping for
an undischarged obligation. Task 31 performs no discharge, `NeedsAtp`
transition, proof verification, theorem acceptance, or fact publication.

## VC Task 31; Exact Task-180 Open VcIr Proof-Verification Snapshot

Task 31 implements the preceding mapping through public borrowed adapter
`generate_exact_task180_vc`. The adapter accepts the exact CoreIr, snapshot,
and explicit generation/VC schema versions
`mizar-vc-generation-task31-v1` / `mizar-vc-vcset-task31-v1`. It encodes Core
package/module identity as a length-framed `VcModuleRef`, constructs the empty
CFG, handoff, and intake internally, and returns only the fully validated
`VcSet` or a typed atomic error.

The distinct
`pass_proof_verification_contradiction_formula_constant_001.miz` sidecar is the
only `active_proof_verification` admission. The runner executes the complete
source-to-checker-to-Core-to-VC path twice and verify-compares the complete
`VcSet::debug_text()` bytes with the committed VcIr baseline. Admission tests
cover wrong stage, missing/duplicate/wrong tag, wrong phase, absent snapshot,
and exclusion of the unchanged type-elaboration Task-180 sidecar. Snapshot and
report tests cover absent/missing/unreadable/mismatched baselines, failure
diagnostics, deterministic report bytes, and the CLI summary.

The VC corruption matrix covers the exact Core table allowlist and every
owner/formula/proof/terminal/seed identity, kind, status, path, source,
source-map, Core-reference, provenance, handoff, flow-site, and fresh-intake
boundary. It also proves that classification is structural and marker-free.
Exactly one covered trace requirement is added for this snapshot; broad
proof-verification rows remain deferred. The existing type-elaboration source,
sidecar, Core snapshot, and expectation bytes are unchanged. No downstream
verification or acceptance is claimed.

## Prepared mizar-test Consumers

These are increments within open mizar-test Task 10, not new top-level
mizar-test tasks. Naming an increment grants no coverage before its real
producer, source, sidecar, trace row, and baseline execute.

| Increment | Stage, phase, artifact, and ownership |
|---|---|
| `MT10-VC-T180` | Reserved solely for VC Task 31. Add the distinct `tests/miz/pass/theorems/pass_proof_verification_contradiction_formula_constant_001.miz` source and matching expectation sidecar under `proof_verification` / `active_proof_verification`, `expected_phase = "vc_generation"`, pipeline phase 11. Compare two complete `VcSet`s and the complete `SnapshotKind::VcIr` / `VcSet::debug_text()` bytes. Admission tests reject the wrong stage, a missing/duplicate/wrong active tag, the wrong `expected_phase`, and the unchanged type-elaboration sidecar. The existing type-elaboration Task-180 source, sidecar, Core snapshot, outcome, and trace backlink remain unchanged. Task 31 adds and activates exactly one trace row: id `spec.en.mizar_vc.vc_ir.task180_proof_verification_snapshot`, source `doc/design/mizar-vc/en/source_vc_decomposition.md`, section `VC Task 31; exact Task-180 open VcIr proof-verification snapshot`, stage `proof_verification`, status `covered`, `required = true`, coverage `snapshot`, and sole test backlink `tests/miz/pass/theorems/pass_proof_verification_contradiction_formula_constant_001.expect.toml`. Task 30 adds no row. |
| `MT10-VC-PV` | Shared contract for VC Tasks 32-55. Each task owns a distinct slice `MT10-VC-PV/VC<n>` with its smallest real source, sidecar, task-local trace row, full `VcIr` baseline (including an empty `VcSet` for an owned zero-VC boundary), and corruption/negative coverage. Stage/tag are `proof_verification` / `active_proof_verification`, `expected_phase = "vc_generation"`, pipeline phase 11, and the artifact is complete deterministic `VcSet::debug_text()` bytes. VC 40 remains unexecuted behind VC 37/39 plus Core 40/A1; VC 53 remains unexecuted behind its bounded canonical-authority gap. Missing scheme/theorem roles remain outside direct VC 41 behind Gate S1. |

The first runner/tag/guard change must land with Task 31's first real baseline.
There is no empty runner or snapshot-infrastructure prerequisite. Existing
parser-only theorem, definition, registration, and algorithm fixtures must not
be reclassified as semantic VC baselines.

## Accepted Definition And Proof VC Task Graph

| VC task | Bounded family and canonical authority | Required Core dependency and consumer | Exit boundary |
|---|---|---|---|
| 32 | General theorem proof-step and terminal-goal VCs, stable formula payloads, quantified binders, and ordered local contexts. Specs 04.5, 14, 15, and 16.1-16.5/16.7. | Core 33-35/37; `MT10-VC-PV/VC32`. | Task 180 remains VC 31. No proof search, closure, acceptance, or verified-premise publication. |
| 33 | Functor-definition correctness for both `equals` and `means`: result-type/type-correctness assertions, guarded-branch consistency, and missing-`otherwise` coverage; plus existence and uniqueness for `means` only. Specs 10.3-10.6/10.12.2-10.12.6 and 16.6.1/16.6.4. | Core 33-36; `MT10-VC-PV/VC33`. | Result-type assertions are VCs for both accepted styles. Unconditional `equals` has no existence, uniqueness, guarded-consistency, or coverage VC, and ill-formed/type-invalid bodies are rejected before VC generation. Predicate, attribute, mode, and structure declarations do not acquire invented existence/uniqueness VCs; no correctness acceptance. |
| 34 | Predicate and functor declared algebraic-property obligations. Specs 09.5, 10.6, and 16.6. | Core 33-36; `MT10-VC-PV/VC34`. | Exact declared property and guard only; no theorem fact or inferred property. |
| 35 | Explicit mode-declaration `sethood`, structure inheritance/type-inclusion coherence, and property-implementation `means` existence/uniqueness and overlap-coherence obligations. Specs 05.3/05.8, 07.8.1, 16.6, and 19.2.2. | Core 33-36; parser 48 where required; `MT10-VC-PV/VC35`. | Mode RHS inhabitation remains a mandatory checker evidence lookup/hard error, not a VC. No accepted constructor, inheritance, property, or mode evidence. |
| 36 | Term-derived choice non-emptiness, generated Fraenkel membership formulas, and per-occurrence non-template `qua` inheritance/cluster-widening validity obligations. Specs 13.4-13.6/13.8.6-13.8.7 and 14. | Core 34-36; `MT10-VC-PV/VC36`. | Template view actuals belong to VC 41. Fraenkel sethood is validated before VC generation and only its accepted evidence may enter context. No use-site/generated sethood VC, implicit choice evidence, invented view, or proof-free narrowing. |
| 37 | Existential, conditional, and functorial registration correctness. Specs 07.8, 16.6.3, and 17.2-17.5/17.8.3/17.9. | Core 39; `MT10-VC-PV/VC37`. | Pending correctness VCs only; no activation, closure, or registered fact. |
| 38 | Predicate/functor/attribute redefinition compatibility and coherence. Specs 06.7, 09.6-9.7, 10.7-10.8, 11.1, 16.6, and 19.5. | Core 38; direct roles only; `MT10-VC-PV/VC38`. | No guessed root, accepted coherence, refinement winner, or missing scheme role. |
| 39 | Reduction `reducibility` universal-equality obligation. Specs 17.6 and 17.9.4. | Core 39; `MT10-VC-PV/VC39`. | Simplification-order/size/variable checks are structural registration-time rejection rules and never VCs. No rewrite activation or formula synthesized from labels/order alone. |
| 40 | **Blocked-reserved:** authenticated registration/cluster/reduction trace context and fingerprints. Specs 17.1/17.3.4/17.6-17.9. | Completed VC 37 and VC 39 outputs plus Core 40 and Gate A1/MC-G004; unexecuted `MT10-VC-PV/VC40`. | After all dependencies exist, attach authenticated trace context/fingerprints to the real VC-37 registration/cluster correctness VCs and VC-39 reduction-equality VCs and their full snapshots. Never create a trace-derived goal or a standalone trace-only candidate. |
| 41 | Dependency-ready direct template use-site constraints, signature compatibility, and view-actual validity obligations. Specs 18.2/18.10.2/18.10.4-18.10.5. | Core 34-38; `MT10-VC-PV/VC41`. | Only authenticated direct roles and substitution requests. Missing scheme/theorem roles remain blocked by Core 41/Gate S1 outside this executable slice and are never fabricated. |

Definition and property tasks use `DefinitionCorrectness` with exact
correctness-family provenance where that existing kind is semantically honest.
Task 36 uses the specific generated non-emptiness and Fraenkel kinds. The
existing `GeneratedSethood` kind is explicit-handoff compatibility from the
earlier generator contract, not a Task-30 source-derived family: explicit mode
declaration sethood belongs to VC 35 as `DefinitionCorrectness`, while Fraenkel
sethood is prerequisite evidence and emits no sethood VC. No descendant may
source-generate `GeneratedSethood` without future canonical authority and its
first real source. Registration, redefinition, and reduction tasks use
`RegistrationStyleCorrectness` plus exact family/style provenance. A future
task may add a narrower kind only with its first real source and canonical
formula; Task 30 authorizes no empty enum expansion.

## Accepted Algorithm VC Task Graph

Every algorithm task rejects diagnostic-bearing or incomplete Core/CFG input.
Branching and hidden-state facts are ordered context/provenance unless an
explicit canonical formula makes them a proof obligation.

| VC task | Bounded family and canonical authority | Required Core dependency and consumer | Exit boundary |
|---|---|---|---|
| 42 | Local `as T`/narrowing and field-update type obligations. Specs 05.7-05.8, 08.2, 13.3, 19.3, and 20.1.3-20.1.4. | Core 42/46/48/52-53; `MT10-VC-PV/VC42`. | Add an honest narrow VC kind with the first real source if required; do not squeeze these obligations into an unrelated existing kind. |
| 43 | `requires` as callee-body context; return-result substitution, postcondition, and assertion VCs. Specs 20.4-20.5 and 20.13.1/20.13.3. | Core 42-43/46/48/52-53; `MT10-VC-PV/VC43`. | A declared entry `requires` is not an `AlgorithmPrecondition` proof VC. No return/postcondition fact without its generated goal. |
| 44 | Call actual/result substitution, call-precondition VC, and successor postcondition fact for a verified `terminating` callee. Specs 20.4.1, 20.8, and 20.13.1. | Core 46/48/52-53; concrete substitution is VC-owned; `MT10-VC-PV/VC44`. | Substitution is replayable, capture-safe, and provenance-versioned; no label-text inference. A partial-call postcondition cannot be admitted until Task 53's bounded transport/authentication-authority gap is resolved and exact evidence is available. |
| 45 | `if` path contexts exercised through a nested real VC. Specs 20.2.1 and 20.13.3. | Core 43/48/52-53; `MT10-VC-PV/VC45`. | `if` alone creates no standalone VC; then/else conditions remain ordered context. |
| 46 | Ordered `match` capture/nonmatch contexts and an explicit `exhaustive` proof only. Specs 20.2.5 and 20.13.3. | Core 45/50/52-53; `MT10-VC-PV/VC46`. | No implicit exhaustiveness VC. Add an honest kind only with an explicit canonical goal. |
| 47 | While invariant establishment/preservation/break/continue with old-state, havoc, and alias context. Specs 20.2.2, 20.5, and 20.13.3. | Core 43/46/48/52-53; `MT10-VC-PV/VC47`. | Normal exit is successor context, not `LoopInvariantPhase::Exit` merely because an exit exists. |
| 48 | Range positive-step and invariant/break obligations, with evaluated-once `a0`/`b0`/`s0`/`i_exit` context for `to` and `downto`. Specs 20.2.3, 20.5, and 20.13.3. | Core 44/46/49/52-53; `MT10-VC-PV/VC48`. | `RangeBound`/`HiddenIndex` are not standalone VCs without explicit canonical formulas. |
| 49 | Collection finiteness, invariant establishment/maintenance/break, and processed-set context. Specs 20.2.4, 20.5, and 20.13.3. | Core 44/46/49/52-53; `MT10-VC-PV/VC49`. | Standalone order independence requires an explicit formula; otherwise maintenance carries its meaning. |
| 50 | Set/type `Pick` non-emptiness with distinct runtime/ghost provenance. Specs 20.3 and 20.13.3. | Core 42/46/48/52-53; `MT10-VC-PV/VC50`. | Use a non-emptiness kind with exact Pick origin; no chosen witness or execution result. |
| 51 | Term-derived Nat-valued and lexicographic loop/continue measure formulas. Specs 20.5.2-20.5.3, 20.7, and 20.13.3. | Core 46/48-49/52-53; `MT10-VC-PV/VC51`. | Generate only from explicit measure terms and binder data; no text marker or automatic totality. |
| 52 | Recursive and mutually recursive component-decrease obligations. Specs 20.7-20.8 and 20.13.4. | Core 46/52-53; `MT10-VC-PV/VC52`. | No promotion to `func`, call-graph acceptance, or termination fact. |
| 53 | **Blocked-reserved non-VC admission boundary:** partial-call successor-postcondition admission from an explicit cited or in-context verified termination theorem for that exact call. Specs 20.7-20.8 and 20.13.1 define the admission condition, not its transport/authentication implementation. | Core 46/52-53; unexecuted `MT10-VC-PV/VC53`. Current canonical authority names no authenticated termination-evidence reference payload, producer, schema, or authentication contract. | No `PartialTermination` VC exists. Without exact verified evidence, the call contributes only type facts and zero successor-postcondition facts; missing `decreasing` metadata or a vague caller request never creates an obligation. VC 53 remains blocked until future canonical authority names the evidence producer, reference identity/schema, authentication rules, and owning tests. |
| 54 | Snapshot/claim theorem obligations with captured program context and universal valid-execution quantification. Specs 20.6 and 20.13. | Core 47/51-53; `MT10-VC-PV/VC54`. | No state theorem, old-state substitution, or claim acceptance without explicit captured payload. |
| 55 | Non-VC ghost-isolation integration and zero-VC accounting. Specs 20.1.3, 20.3, and 20.13.5. | Core 46/52-53; `MT10-VC-PV/VC55`. | Diagnostic-free explicit isolation emits no `GhostErasureSafety` VC; ghost-leakage or corrupt/diagnostic-bearing flow is rejected through Core 53. No extraction, runtime erasure, execution, or accepted isolation fact. |

Current generated-formula and local-context builders do not yet express every
dependent binder, guarded schema, concrete substitution, old-state/havoc,
match nonmatch, or term-derived goal. This is owned source/design drift in the
first real task that needs each representation. There is no standalone empty
formula-infrastructure task. Deferred flow rows become eligible only when the
owning task supplies the exact specification-backed goal; there is no general
Deferred-to-Open promotion.

## Disagreement Classification

| Class | Task-30 result |
|---|---|
| `spec_gap` | No gap blocks dependency-ready Tasks 31-52/54-55. The user-facing “no proof” wording for widening is narrowed by Spec 13.8.7's explicit automatically discharged FOL obligations. A bounded gap does block VC 53: current canonical authority requires exact verified termination evidence but does not name its producer, reference identity/schema, authentication rules, gate, or owning tests. Task 30 reports that gap and keeps VC 53 reserved rather than inventing a payload. |
| `test_gap` | The Task-180 VC runner/admission/baseline/corruption matrix, per-family positive assertions, and every owned zero-VC/near-miss or diagnostic negative remain absent and are assigned to Tasks 31-55. |
| `design_drift` | Task 30 closes umbrella ownership drift. It assigns structural terminal classification, honest kind/context/formula gaps, non-template `qua`, direct-template use sites, entry-`requires`, and range/exit boundaries. Draft simplification-order, `PartialTermination`, and `GhostErasureSafety` generation proposals were removed because canonical authority makes them structural/evidence/static boundaries rather than VCs. |
| `source_drift` | The exact Core terminal relation is not yet consumed correctly; the current entry-`requires` mapping and existing exit/range/partial/ghost data shapes do not authorize canonical source generation; all broader formula/substitution/context routes remain unimplemented. |
| `source_undocumented_behavior` | None found in this inventory. |
| `test_expectation_drift` | None in Task-30 scope. Existing type-elaboration and parser expectations remain unchanged. |
| `boundary_violation` | None remains in the accepted contract. Marker injection, raw-source reconstruction, fake anchors/statuses, acceptance, unrelated-kind coercion, or generating the three rejected static/evidence families would violate the boundary. |
| `repo_metadata_conflict` | None found. Task 30 adds no requirement row and preserves the count/hash oracles. |

## Coverage And Exit Boundary

Task 30 changes `spec_coverage_audit.md` only to replace umbrella follow-up
ownership with the accepted VC 31-55 graph. Existing trace rows may receive
deferred-reason ownership text only; their status, required flag, coverage
class, and tests remain unchanged. Chapter ratings, test counts, runner counts,
and all behavioral hashes remain unchanged.

Task 30 is complete only when the exact Task-31 mapping and both consumer
contracts are frozen, every source-derived family has a bounded owner or
explicit blocked authority gap, all available Core dependencies and canonical citations are
present, English and Japanese documents agree, and the full preservation
oracles pass. At the Task-30 boundary, VC Task 31 was the next sequential STEP
5 task. Steps 6/7 remained deferred, and naming Tasks 31-55 granted no proof-
verification or acceptance credit.

VC Task 31 subsequently implements and executes only that exact slice. Its
covered snapshot row closes the Task-180 `test_gap` and `source_drift` without
crediting the broad proof-verification row. Tasks 32-55 remain dependency-
paced. For the Core-33-to-VC-32 dependency chain, checker Task 248 is the next
prerequisite and recommended continuation, followed by Core 33 and its
descendants. The top-level TODO independently authorizes parser Tasks 47-48
and resolver Task 31, so every continuation still requires a fresh global
inventory rather than a unique-priority claim. Steps 6/7 remain deferred.
