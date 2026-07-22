# STEP 5 Source-Derived Core/Control-Flow Family Decomposition

> Canonical language: English. Japanese companion:
> [../ja/source_family_decomposition.md](../ja/source_family_decomposition.md).

This document is the accepted output of mizar-core Task 32. It decomposes every
remaining source-derived `CoreIr` and `ControlFlowIr` family into bounded
follow-up tasks. It is task and dependency authority only: it changes no
language semantics, Rust behavior, `.miz` source, expectation, trace status,
test list, or coverage credit.

## Authority And Baseline

The inventory follows the repository authority order:

1. `doc/spec/en/`;
2. existing `.miz` sources;
3. `tests/coverage/spec_trace.toml`;
4. existing expectation sidecars;
5. design documents;
6. source as non-normative inventory evidence.

The accepted checker input is Task 247's
[payload-family decomposition](../../mizar-checker/en/payload_family_decomposition.md).
Core Task 31 remains the sole exact Task-180 exception. The broad non-Task-180
`CoreIr` row and every `ControlFlowIr` row remain deferred.

Task 32 preserves the clean-entry behavioral oracles: active runners parse 96,
declaration 4, and type elaboration 188; plan 403/368; type elaboration 236/224;
pass/fail 219/184; 272 `mizar-test` unit tests; and 17 production paths. The
Task-247 production layout was 19,803 lines with path hash
`b36d96fed3207b415c95de27be11ade57654c6573a2f0637aa2d0a3d56aca01d`
and content hash
`5f9e716169964a861b71576957c05e2dc2538b5e0ff9d1025ef51a4bea6aa306`.
Task 32 may change only documentation and deferred-reason text, so runner,
test-list, and production-source oracles must remain unchanged.

## Contract Shared By Descendant Tasks

Each row below is one nonempty logical task and one commit. Before editing, its
implementation specification must freeze the exact canonical sections, source
family, upstream syntax-free payload, consumer, visibility, forbidden scope,
tests, coverage impact, and exit criteria.

Unless a row narrows the rule, every source-derived Core task must:

- leave `.miz` AST inspection in `mizar-test`;
- consume a checker-owned, syntax-free, source-ordered final projection rather
  than raw syntax or resolver reconstruction in `mizar-core`;
- preserve owner identity, dense order, source ranges, recovery state,
  predecessor links, and versioned provenance transactionally;
- use existing generic Core lowering APIs where they already express the
  accepted payload, without generalizing Task 31's exact adapter;
- fail closed on missing, duplicate, reordered, recovered, cross-owner,
  cross-module, stale-provenance, wrong-role, partial, or orphan payloads;
- include deterministic rerun and a complete immutable IR comparison in the
  prepared consumer owned by that task;
- add the smallest specification-derived real-source positive case and
  corruption matrix needed for the slice, without changing an existing
  expectation merely to match implementation;
- when a task owns rejection or diagnostics, add the smallest
  specification-derived real-source negative case for every newly claimed
  diagnostic family; payload corruption is additional boundary coverage and
  never substitutes for executing the semantic rejection through the runner;
- activate trace credit only for the exact executable slice that lands.

The algorithm tasks additionally own, in the same logical task, the narrow
joint path from `mizar-test` AST extraction through a syntax-free
`mizar-checker` final projection to `mizar-core` lowering. Core Task 32 is the
canonical authority for that joint producer/lowerer split; no new checker task
number is implied. If a fresh task inventory finds a concrete missing parser or
resolver semantic identity, that slice stops for a specific authority decision
instead of guessing the identity or creating a general unnamed gate.

All tasks forbid proof search, theorem or algorithm acceptance, registration
status fabrication, downstream artifact/schema invention, public diagnostic
code allocation, `VcId` or `ObligationAnchor` assignment, VC generation,
kernel checking, MVM execution, code extraction, and Steps 6/7 promotion.

## Accepted CoreIr Task Graph

Task identifiers in this table belong to `mizar-core`. The checker task numbers
name prerequisites; they are not redefined here.

| Core task | Bounded source-derived family | Dependencies and prepared consumer | Exit boundary |
|---|---|---|---|
| 33 | Core context, item shells, binders, declaration order, local scope, visibility, reserve/default context, source map, and checker provenance. | Checker 248; `MT10-CIR-TE`. | Context and item identity only. No type result, RHS, proof, registration activation, or algorithm body. |
| 34 | Type, attribute, evidence, coercion, and view lowering: written heads/arguments, attribute chains, accepted/missing evidence references, normalized types, widening/narrowing request results, sethood/non-emptiness records, and authenticated reduct/view paths. | Checker 249-251; Core 33; existing Core 27-30 explicit-payload APIs; `MT10-CIR-TE`. | Reusable conversion/evidence/view lowering only. Source-level proof-local `reconsider` belongs to Core 37. No evidence, path, or fact is inferred. |
| 35 | Primary/application/structure/set/choice/`qua` terms and atomic/composite formulas, including complete child graphs, binder links, generated choice/comprehension origins, and source identity. | Checker 252-257; Core 33-34; `MT10-CIR-TE`. | No truth, theorem acceptance, proof closure, implicit sethood, invented view, or algorithm `Pick`. |
| 36 | Predicate, functor, attribute, mode, structure, and property-definition shells; parameters/guards/definiens; definition expansion boundary; correctness-condition and initial-obligation references. | Checker 259-264; Core 33-35; parser 48 for property syntax; `MT10-CIR-AS`. | No correctness proof, obligation discharge, accepted property, recursive unfolding, overload winner, or axiom publication. |
| 37 | Statement/theorem shells; assumptions/conclusions; proof-local declarations and closures; source-level `reconsider`; non-Task-180 proof skeletons, citations, cases, pending/blocked state, thesis, and terminal goals. | Checker 258 and 269-272; Core 33-35; Core 34 conversion API; parser 47 for `reconsider`; `MT10-CIR-FS`. | Task-180 remains Core 31. No proof search, implicit closure, theorem fact, acceptance, discharge, or verified premise. |
| 38 | Direct template-role Core metadata, authenticated substitution requests, ordinary/template overload results, redefinition/notation roots, coherence/refinement input, and exposed views. | Checker 277-279; Core 33-36; `MT10-CIR-AS`. | Executable direct roles only. Missing scheme/theorem roles are Core 41/Gate S1. No substitution result, guessed root, accepted coherence, or fresh scheme symbol. |
| 39 | Pending registration shells and correctness/initial-obligation intake for existential, conditional, functorial, and reduction registrations. | Checker 273; Core 33-37; `MT10-CIR-AS`. | Pending intake only. No accepted status, activation, closure, rewrite, trace, artifact, `VcId`, or discharge. |
| 40 | **Blocked-reserved:** authenticated accepted-registration activation plus source-derived cluster and reduction trace lowering. | Checker 274-276; Core 34-35 and 39; Gate A1 and MC-G004; `MT10-CIR-AS`. | Not executable until the accepted verifier/artifact-status producer, schema, authentication, and corruption tests are canonically named. Never derive `Accepted` from order, local checking, or an obligation request. |
| 41 | **Blocked-reserved:** scheme/theorem-role-dependent Core slices not exposed by direct parser/syntax roles. | Checker Gate S1; applicable Core 33-38; `MT10-CIR-AS`. | Not executable until canonical parser/syntax and resolver ownership names the missing roles. Task 277 and Core 38 do not fabricate them. |

Core 39 and 40 remain separate logical tasks: pending registration intake is
executable independently, while accepted activation and traces are gated.
Core 38 and 41 likewise keep direct roles separate from missing roles.

## Accepted Algorithm CoreIr Task Graph

Chapter 20 is sufficient to name the following semantic families and negative
boundaries. Parser Tasks 32-34 provide syntax coverage only; they are not a
semantic payload producer. Each task below therefore implements its own bounded
joint source/checker/Core route and must prove that the needed identities exist
before editing semantic behavior.

| Core task | Bounded source-derived family | Dependencies and prepared consumer | Exit boundary |
|---|---|---|---|
| 42 | Algorithm declaration/header, parameters/result, visibility, runtime/ghost `var` and `const`, mutability, resolved places/lvalues, assignment, and executable `Pick` shells. | Specs 20.1 and 20.3; parser 32-34 as syntax-only evidence; Core 33-35; direct template roles from Checker 277/Core 38 where applicable; `MT10-CIR-ALG`. | Core algorithm shells only. No CFG, hidden loop state, call substitution, Pick non-emptiness VC, promotion, execution, or extraction. |
| 43 | Structured `if`, `while`, `return`, `break`, and `continue` statement shells with owners, child order, conditions/values, and source/provenance. | Spec 20.2.1-20.2.2/20.2.6; Core 42; `MT10-CIR-ALG`. | No CFG edge, path fact, exit proof, invariant attachment, or reachability result. |
| 44 | Range and collection loop shells, separately validating direction, bounds/collection, explicit step, binder, loop body, hidden immutable bound/step values, and hidden processed-set state. | Spec 20.2.3-20.2.4; Core 42-43; `MT10-CIR-ALG`. | One task for the common `for` shell, but separate range/collection contracts and negative cases. No CFG, finiteness/order-independence fact, termination proof, or fabricated hidden value. |
| 45 | `match` subject, ordered cases, resolved pattern graph, captures, guard/body ownership, and explicit exhaustiveness/unsupported state. | Spec 20.2.5; Core 42-43; `MT10-CIR-ALG`. | No inferred pattern identity, implicit exhaustiveness, CFG branch fact, or proof acceptance. |
| 46 | `requires`/`ensures`/`assert`, loop invariants/decreasing terms, call target/actual/result-binder and substitution-request metadata, recursive-group membership, declared terminating intent, and termination-measure availability. | Specs 20.4-20.5, 20.7-20.8, and 20.13; Core 34-35 and 42-45; `MT10-CIR-ALG`. | Transport requests and metadata only. Actual call/result substitution generation, application, and validation belong to VC Task-30 descendants. No contract axiom, VC, termination proof, promotion, or recursive encoding is emitted. |
| 47 | Snapshot and claim shells, snapshot identity, captured algorithm context request, visible runtime/ghost locals, hidden-loop-value references, claim body/order, and missing/unsupported state. | Spec 20.6; Core 37 theorem shells and Core 42-46; `MT10-CIR-ALG`. | Claim links only to already-lowered Core-37 theorem/statement shells. No source-text reconstruction, context fact invention, old-state substitution, claim proof, CFG capture, VC, or acceptance. |

Chapter 20.9 MVM execution, `by computation`, runtime failures/gas, and 20.10
code extraction remain parked downstream work. Chapter 20.13 formulas are VC
authority; Core 46 records only authenticated metadata needed by later owners.

## Accepted Phase-10 ControlFlowIr Task Graph

| Core task | Bounded phase-10 family | Dependencies and prepared consumer | Exit boundary |
|---|---|---|---|
| 48 | Basic CFGs: deterministic blocks/edges, locals, statement placements, program contexts, source maps, fallthrough, structured exits, and source-derived structural diagnostics for illegal break/continue and unsupported local declarations. | Core 42-43; `MT10-CFG-PV`. The task that first lands a real CFG baseline adds `SnapshotKind::ControlFlowIr` and its schema/guard in the same commit; real-source negative cases cover both structural diagnostic families. | No empty snapshot-infrastructure prerequisite, semantic contract attachment, VC, proof status, or artifact. |
| 49 | Range/collection-loop CFG attachment: hidden immutable bound/step/domain values, loop/exit metadata, processed binding, and normal/early-exit state. | Core 44 and 48; `MT10-CFG-PV`. | No finiteness/order-independence theorem or VC, and no hidden value reconstructed from spelling. |
| 50 | Match CFG attachment: ordered arm edges/path-condition metadata, capture initialization, joins, and explicit exhaustiveness/unsupported metadata. | Core 45 and 48; `MT10-CFG-PV`. | No semantic algebraic matching, invented capture, exhaustiveness proof, or VC. |
| 51 | Snapshot/claim flow state: snapshot program-point ownership, captured-local identity/context, hidden-loop-value references, and claim-to-snapshot links. | Core 47-50 as nesting requires; `MT10-CFG-PV`. | No state theorem acceptance, old-state substitution, artifact/extraction schema, or VC. |
| 52 | Contract, call, assertion, invariant, decreasing, ghost-effect, recursive-group, and termination metadata attachment to CFG sites and contexts. | Core 46 and 48-51; `MT10-CFG-PV`. | Carries callee/actual/result-binder/substitution-request metadata only. Concrete substitution and VC context are VC-owned. No discharge, promotion, or ghost-erasure proof. |
| 53 | Complete source-derived semantic flow diagnostics: use-before-assignment, unreachable statements, immutable assignment, ghost leakage, malformed/missing call-contract payload, pattern/capture, snapshot/claim, and alias/lvalue-precision failures, with stable internal detail and source ordering. | Core 48-52; diagnostics registry for public codes only if separately authorized; `MT10-CFG-PV`. Add a smallest real-source fail consumer for every newly claimed diagnostic family, plus corruption tests for malformed handoffs. | Structured local diagnostics only. Core 48 owns illegal break/continue and unsupported-local structural negatives. No public code invention, proof result, VC, or acceptance. |

## Prepared mizar-test Consumers

These are consumer increments within open mizar-test Task 10, not new
top-level mizar-test tasks. A prepared name grants no coverage before its first
real producer and baseline execute.

| Increment | Stage/tag, phase, and artifact | Dependencies and corruption boundary |
|---|---|---|
| `MT10-CIR-TE` | `type_elaboration` / `active_type_elaboration`, `expected_phase = "elaboration"`; pipeline output phase 9; `SnapshotKind::CoreIr` whose body is the complete deterministic `CoreIr::debug_text()` bytes for Core 33-35. | Exact checker prerequisites and owning Core task. Reject wrong stage/tag/phase/domain/kind/hash, missing/duplicate/reordered/cross-owner/cross-module/recovered/stale/partial input, and nondeterministic bytes. The broad Task-19 row remains deferred until its bounded slices really execute. |
| `MT10-CIR-FS` | `formula_statement` / `active_formula_statement`, `expected_phase = "elaboration"`; pipeline output phase 9; `SnapshotKind::CoreIr` whose body is the complete deterministic `CoreIr::debug_text()` bytes for Core 37. | `MT10-FS`, Checker 258/269-272, Core 33-35/37. Same corruption boundary; no truth, theorem acceptance, proof verification, or VC credit. |
| `MT10-CIR-AS` | `advanced_semantics` / `active_advanced_semantics`, `expected_phase = "elaboration"`; pipeline output phase 9; `SnapshotKind::CoreIr` whose body is the complete deterministic `CoreIr::debug_text()` bytes for Core 36 and 38-41. | `MT10-AS`, applicable checker/Core tasks, Gate A1 for Core 40 and Gate S1 for Core 41. Same corruption boundary; blocked families remain unexecuted and uncredited. |
| `MT10-CIR-ALG` | `proof_verification` / `active_proof_verification`, `expected_phase = "elaboration"`; pipeline output phase 9; `SnapshotKind::CoreIr` whose body is the complete deterministic algorithm-bearing `CoreIr::debug_text()` bytes for Core 42-47. | Each joint algorithm producer/lowerer. Same corruption boundary plus owner/nesting/local-role/contract/snapshot-link checks. It grants no CFG, VC, proof-verification result, MVM, or extraction credit merely because the stage is named. |
| `MT10-CFG-PV` | `proof_verification` / `active_proof_verification`, `expected_phase = "elaboration"`; pipeline output phase 10; `SnapshotKind::ControlFlowIr` whose body is the complete deterministic module-level `ControlFlowOutput::debug_text()` bytes, including every `ControlFlowIr`, for Core 48-53. | `MT10-CIR-ALG` and the owning CFG task. Same corruption boundary plus block/edge/local/context/site/source-map referential integrity. It excludes obligation handoff and `VcIr` and grants no VC, proof acceptance, kernel, artifact, MVM, or extraction credit. |

The first general snapshot-registry/schema change must be paired with the first
real non-Task-180 source-derived baseline that needs it. An empty infrastructure
commit or a synthetic Core/CFG snapshot is forbidden.

## Gates And Cross-Crate Boundary

- Gate A1 and MC-G004 continue to block Core 40.
- Gate S1 continues to block Core 41 and any algorithm template slice that
  actually requires a missing scheme/theorem role.
- MC-G005 and CORE-AUDIT-G006 keep public diagnostic codes outside these tasks.
- VC Task 30 becomes dependency-authorized after this Task-32 documentation
  commit because Core Tasks 31 and 32 are then complete. It may specify
  mappings and descendants, but must preserve every unimplemented Core task and
  gate and may not generate or accept a VC in its docs-only decomposition.
- Steps 6/7 remain deferred; none of these names promotes them.

## Disagreement Classification

| Protocol class | Task-32 inventory and disposition |
|---|---|
| `spec_gap` | No new blocking Task-32 specification gap. Chapter 20 and the other English chapters define the families and negative semantic boundaries. Existing MC-G005 remains a separate nonblocking public-code gap. |
| `test_gap` | Every non-Task-180 Core snapshot, all source-derived algorithm/CFG baselines, their real-source negative diagnostic consumers, and their corruption matrices remain absent. Assigned to Core 33-53 and the five prepared consumers without changing status. |
| `design_drift` | Remaining Core/CFG families previously had umbrella ownership. Closed by this task graph. Task 247's lack of an algorithm producer number is resolved by Task 32's explicitly authorized joint task contract, not by inventing a checker ID or a new unnamed general gate. |
| `source_drift` | Real source-to-checker-to-Core routes are absent beyond Task 180, and all source-derived CFG routes are absent. Assigned to the descendant graph. |
| `source_undocumented_behavior` | None newly found. Current explicit-payload APIs and the Task-180 adapter remain narrower than the specification and document their boundaries. |
| `test_expectation_drift` | Parser Task 47 retains the existing omitted-`reconsider` drift. Task 32 does not repair it. |
| `boundary_violation` | None current. The review prevented overlap between Core 34 conversion APIs and Core 37 source `reconsider`, and kept concrete call/result substitution out of Core 46/52. Raw-syntax inspection in Core, evidence/identity fabrication, or downstream substitution/VC work here would create a violation. |
| `repo_metadata_conflict` | None found. The stale Core Task-30 ledger hash was separately repaired as `CORE-LEDGER-001`; Task 32 does not auto-repair repository metadata. |

## Task-32 Exit Criteria

Task 32 is complete only when:

- every remaining source-derived Core/CFG family has one bounded task and one
  prepared consumer or an explicit preserved gate;
- English canonical and Japanese companion Core, checker, mizar-test, VC, and
  top-level ownership documents agree;
- trace changes alter only deferred ownership/reason wording and keep all
  statuses, coverage classes, and test lists unchanged;
- `doc/design/spec_coverage_audit.md` records ownership only, with no new
  coverage credit;
- no source, fixture, expectation, runner count, test list, or production
  layout/hash changes;
- review-only specification, test-sufficiency, implementation-scope, and
  source/documentation consistency reviews end with no findings;
- focused metadata/lint checks and the complete baseline verification pass;
- the result is one docs/traceability Task-32 commit.

After this commit, fresh inventory selects the smallest dependency-ready task.
Checker 248 is the first upstream producer for Core 33; VC Task 30 is also
dependency-authorized as a docs-only decomposition. Selection must follow the
canonical top-level sequence and may not bypass a missing prerequisite.
