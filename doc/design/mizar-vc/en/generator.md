# Module: generator

> Canonical language: English. Japanese companion:
> [../ja/generator.md](../ja/generator.md).

## Purpose

This module specifies phase-11 VC generation for `mizar-vc`.

The generator consumes the task-4 `SeedIntakeTable`, immutable `CoreIr`,
immutable `ControlFlowIr`, source/core provenance, proof hints, and verifier
policy inputs. It produces generation candidates that later task 8 normalizes,
classifies, maps back to seeds, and assigns deterministic `VcId`s to. It does
not parse source syntax, reconstruct missing checker payloads, discharge VCs,
call ATP backends, validate proofs, publish artifacts, or accept registrations.

Task 5 is specification-only. Rust source for theorem/definition and
registration-style correctness VCs is task 6; algorithm VC generation is task
7; normalization, classification, and `VcId` assignment are task 8.

## Gap Classification For This Spec

| ID | Class | Evidence | Handling |
|---|---|---|---|
| GEN-G001 | `spec_gap` | `generator.md` did not exist before task 5, while tasks 6-8 need a generation contract. | This task creates the English/Japanese generator spec without Rust source. |
| GEN-G002 | `external_dependency_gap` | `mizar-core` currently carries registration, redefinition, and reduction correctness through available definition/checker seeds and provenance rather than dedicated payloads for every registration-style condition. | Generate registration-style correctness VCs only from explicit core/checker seeds and provenance that are already present. Missing dedicated payloads are `DeferredExternal` or no-VC records, not fabricated obligations. |
| GEN-G003 | `test_gap` / `source_drift` | No `src/generator.rs` or generator tests exist before tasks 6-8. | This spec names the behavior and test obligations for the implementation tasks; task 5 changes no Rust source. |
| GEN-G004 | historical `external_dependency_gap`; exact exception resolved | Before Task 31, active source-derived `proof_verification` `.miz` runner support and extraction seams were unavailable. | Tasks 6-8 used Rust fixtures over explicit core/control-flow payloads; Task 31 adds the exact Task-180 source-derived exception, while broader corpus activation remains deferred under task 15 and the source-VC decomposition. |
| GEN-G005 | `external_dependency_gap` | `ObligationSeed` does not yet expose first-class theorem status dependency metadata or dedicated registration/redefinition/reduction correctness payload fields. | Task 6 preserves only namespaced explicit `CoreProvenance` markers supplied by upstream fixtures. Absent markers produce ordinary candidates or visible no-candidate records according to the seed kind/status; the generator must not infer these semantics from labels, generic paths, or source text. |
| GEN-G006 | `external_dependency_gap` | The current `ObligationSeedHandoff` carries flow-site metadata and goal formulas for contract, assertion, and invariant obligations, but it does not yet expose call-precondition, branch, match, range-loop, collection-loop, term-derived termination schemas, authenticated partial-call termination evidence, or complete ghost-isolation metadata. | Task 7 generates candidates only for explicit flow-derived seed rows that have `ControlFlowObligationSite` metadata and a goal formula. Missing proof families and zero-VC/static integration payloads remain visible no-candidate/deferred records instead of fabricated VCs. |
| GEN-G007 | `source_drift` / `test_gap` | After task 7 the Rust source has pre-normalized task-6/task-7 candidates, but no final `VcSet` normalizer, documented `VcKind` ordering rank, dense `VcId` assignment, or normalizer tests. | Task 8 implements the normalizer, documents the stable classification order below, and adds Rust tests for dense ids, seed accounting, duplicate rejection, deferred status preservation, expanded-mapping validation, and stable rendering inputs. |
| GEN-G008 | `deferred` | Status transitions, deterministic discharge, dependency slices, ATP translation, kernel/proof/cache/corpus integration, and source-derived corpus runner activation depend on later module specs or unavailable external seams. | Task 8 records those paths as out of scope and must preserve existing statuses instead of discharging, transitioning, slicing, translating, publishing, or fabricating external integration records. |

No `doc/spec`, `.miz` fixtures, expectations, or traceability metadata change
in task 5. This document refines existing architecture/spec requirements; it
does not introduce new language semantics.

## Inputs And Outputs

Required inputs:

- a validated `SeedIntakeTable` with every handoff row intake-accounted once;
- immutable `CoreIr` proof skeletons, theorem propositions, definition
  correctness payloads, proof hints, formulas, source maps, and provenance;
- immutable `ControlFlowIr` algorithm contexts, structured exits, contract
  sites, loop metadata, ghost/runtime facts, and termination metadata;
- explicit registration, redefinition, reduction, checker, cluster, and
  reduction-trace references when the upstream payload contains them;
- verifier policy inputs that affect status, premise restrictions, local
  unfolding, computation requests, or open/assumed handling.

Generator output is a pre-normalized candidate set:

- candidate VC records with stable seed references, owner references,
  generated formula references, local contexts, symbolic premise refs, proof
  hints, source/core provenance, and anchor ingredients;
- a VC-local generated formula table for split, instantiated, or schema-created
  formulas;
- seed-to-candidate bookkeeping with explicit zero, one, or expanded candidate
  cardinality;
- deterministic sort keys and classification ingredients consumed by task 8.

The candidate set is not final `VcIr`. It may not expose concrete `VcId`s, final
canonical ordering, final `SeedAccountingTable`, discharge evidence, dependency
slices, or ATP/backend text.

## Global Generation Rules

The generator must:

- consume only explicit `CoreIr`, `ControlFlowIr`, seed, provenance, and policy
  payloads;
- preserve the task-4 handoff order as input evidence while providing stable
  canonical sort keys for task 8;
- fail closed when required formula, source, owner, context, or provenance data
  is missing;
- represent skipped, invalid, deferred, policy-open, and missing-payload cases
  as visible candidate/status/accounting records;
- keep `NeedsAtp`-bound obligations concrete with full local context and proof
  hints;
- keep generated formulas in the VC-local table with provenance and generation
  schema versions;
- avoid mutating `CoreIr`, `ControlFlowIr`, the seed handoff, or upstream source
  maps;
- avoid any dependence on hash-map iteration, worker completion order, backend
  availability, local absolute paths, or timings.

The generator must not:

- infer a theorem, definition, registration, reduction, or algorithm obligation
  from raw syntax when the core/control-flow payload is absent;
- turn registration activation into proof acceptance;
- make an unavailable registration, redefinition, or reduction correctness
  condition available to downstream resolution as a fallback;
- silently drop local context, disabled seeds, policy-open obligations, or
  deferred external gaps;
- choose ATP encodings, backend process options, proof certificates, cache hits,
  kernel acceptance, or artifact schemas.

## Local-Context Construction

Every candidate VC gets a self-contained local context. ATP translation must be
able to encode the candidate without reading source syntax or reconstructing
semantic context.

Required context ingredients, when explicitly available at the obligation site:

- binder declarations, normalized binder roles, type predicates, sethood facts,
  and non-emptiness facts;
- proof assumptions, current thesis facts, local labels, diffuse-proof facts,
  contradiction assumptions, and scoped witnesses;
- symbolic citations from `by` justifications, grouped citations, bulk
  citations, and local proof labels;
- checker facts, generated facts, `qua` evidence, registration traces, cluster
  traces, and reduction traces;
- algorithm path conditions, branch facts, loop invariants, hidden immutable
  loop metadata, post-havoc facts, and structured exit contexts from
  `ControlFlowIr`;
- definition opacity inputs, permitted local unfolding requests, reduction
  policy inputs, and `by computation` requests;
- verifier policy inputs that affect status, dispatch, premise limits, or
  computation limits.

Context entries carry source/core provenance and a canonical context sort key.
The key is independent of insertion order and must be stable enough for task 8
to compute local-context fingerprints. Unknown or incomplete context is never
treated as an empty context: it must either make the candidate conservative,
deferred, or erroneous with diagnostics.

## Step 3: Theorem And Definition VCs

Theorem, lemma, proof-step, and terminal-proof candidates are generated from
explicit proof skeletons and terminal goals in `CoreIr`.

Generation rules:

- instantiate theorem binders and type predicates from the core proof context;
- preserve the current `thesis` as the goal for terminal proof obligations;
- attach proof assumptions and scoped witnesses only inside their owning block;
- preserve cited premises from `by` justifications as symbolic `PremiseRef`s;
- preserve theorem status dependencies for clean, non-clean, open, assumed, and
  conditional theorem checks;
- record proof hints, local unfold requests, and computation requests as
  symbolic hints, not backend dispatch decisions.

Definition correctness candidates are generated only from explicit core
definition-correctness payloads. Supported correctness families include
existence, uniqueness, coherence, compatibility, consistency, reducibility, and
definition-specific sethood or non-emptiness obligations when the upstream seed
names such a condition. Generated formulas must carry the definition owner,
correctness kind, source/core provenance, seed reference, and generation schema.

Each ordinary theorem, proof-step, definition-correctness, or checker-initial
eligible seed generates one candidate unless this spec or a later owned spec
defines an explicit expansion schema. Any split of a conjunction, schema
instantiation, or generated helper formula must be represented in the VC-local
generated formula table and in task-8 seed mapping.

## Generated Core Obligation VCs

Generated core obligation candidates are generated from explicit core seed
kinds only. The required families are:

- generated non-emptiness obligations;
- generated sethood obligations carried by an explicit upstream handoff. This
  is compatibility for the existing task-6/task-8 payload/kind, not authority
  for a Task-30 source-derived generator;
- generated Fraenkel membership axioms.

These candidates use the core formula named by the seed when one exists, or a
VC-local generated formula when the generator must instantiate a schema from
explicit core payloads. The candidate must preserve the seed kind, owner,
source/core provenance, local proof path, semantic origin, and generated-formula
schema. Missing generated-obligation payloads are `DeferredExternal` or visible
no-VC mappings; the generator must not reconstruct them from source syntax.

Each generated core obligation seed generates one candidate unless a later
owned spec defines an explicit expansion schema. Task 8 records the concrete
seed-to-VC mapping and rejects duplicate ownership deterministically.

## Registration-Style Correctness VCs

Registration-style correctness covers registration, redefinition, and reduction
conditions only when explicit core/checker payloads are present. The generator
must not synthesize missing correctness obligations from registration
activation, inferred attributes, or raw source syntax.

When explicit payloads exist, candidate kind and provenance must distinguish:

- existential cluster existence;
- conditional or functorial cluster coherence;
- redefinition compatibility or coherence carried by the core/checker seed;
- reduction `reducibility` equality only. Simplification-order, size, and
  variable-occurrence rules are structural registration-time checks that reject
  invalid registrations and never become proof VCs;
- checker-initial carried obligations whose provenance identifies a
  registration-style owner but whose exact future seed kind is not yet stable.

For registration-style candidates:

- the registration label/FQN, owner item, activation boundary, and correctness
  condition are provenance, not proof acceptance;
- pending or failed correctness must remain unavailable to downstream
  resolution;
- unavailable dedicated registration/redefinition/reduction payloads are
  recorded as `DeferredExternal` or visible no-VC mappings with concrete
  reasons;
- reduction traces may be referenced as context or premise evidence only when
  the upstream payload explicitly records the applied rule.

## Step 4: Algorithm VCs

Algorithm candidates are generated from `ControlFlowIr` and the algorithm seed
rows prepared from it. The generator follows the structured Hoare-style schema
from the language spec and must not reconstruct control flow from source text.

The complete algorithm VC model includes these candidate families:

- call preconditions for each algorithm call;
- postconditions for every return edge and algorithm contract exit;
- assertion obligations;
- branch and match path contexts, plus an explicit match-exhaustiveness proof
  obligation only when the source requests `exhaustive`;
- while-loop invariant entry, preservation, break, continue, and
  decreasing-measure obligations; normal exit is successor context, not an
  exit VC;
- range-loop positive-step, invariant-entry, and invariant-preservation
  obligations for both `to` and `downto`; bounds and hidden indices are context,
  not standalone VCs;
- collection-loop finiteness, processed-set, invariant-establishment,
  invariant-maintenance, and order-independence obligations;
- recursive and mutually recursive termination obligations grouped by call
  graph component and decreasing measure;
- partial-call postcondition admission only when the exact call cites or has in
  context verified termination evidence; no `PartialTermination` VC is emitted;
- Pick non-emptiness obligations for every set-based or type-based Pick site,
  including ghost-only Pick sites;
- ghost-isolation static checking and ghost-only `Pick` erasure records;
  isolation produces no `GhostErasureSafety` proof VC.

Algorithm contexts must preserve:

- old-state assignment facts from the pre-state of an assignment;
- field-update alias identity from resolved lvalue paths;
- post-havoc loop contexts that forget or freshen may-write locations while
  preserving immutable hidden loop values;
- break exits without adding the loop's normal-exit `not C` fact;
- continue exits with invariant and decreasing checks when a measure exists;
- range-loop hidden values `a0`, `b0`, `s0`, `S0`, and hidden exit indices;
- Pick bindings as fresh site-local logical values with explicit
  non-emptiness/type-inhabitation obligations;
- ghost facts only in logical verification contexts, never as runtime
  dependencies.

Task 7 owns the concrete generator implementation only for the subset whose
flow-derived handoff rows currently carry explicit `ControlFlowObligationSite`
metadata and goal formulas: requires, ensures, assertions, and supported
loop-invariant phases. The remaining listed families stay visible
no-candidate/deferred records until upstream payloads and generated formula
schemas expose them explicitly.

## Controlled Definition Unfolding

The generator may unfold or simplify definitions only through explicit policy
from `CoreIr` and proof hints.

Allowed inputs:

- definition opacity or transparency metadata;
- local unfold requests from proof hints;
- reduction registrations explicitly present in resolution traces or checker
  payloads;
- computation requests that remain symbolic until the discharge/computation
  policy fixes limits.

Unfolding output must be represented as generated formulas or premise refs with
source/core provenance. The generator must preserve opaque boundaries when no
explicit policy permits unfolding. It must not use traversal order, global
availability of a definition body, or ATP convenience to strengthen a goal.

## Step 5: Normalization And Classification Handoff

Task 8 owns final canonicalization, classification, and `VcId` assignment. The
generator supplies the inputs required for that task:

- stable candidate sort keys based on module, owner, source/core provenance,
  seed canonical key, generation schema, and expansion index;
- normalized binder and local-context ingredients where upstream APIs provide
  them;
- explicit `VcKind` classification ingredients and priority hints;
- symbolic premises sorted by stable references or marked conservative when
  sorting inputs are incomplete;
- generated formula provenance and schema versions;
- seed-to-candidate cardinality: no candidate, one candidate, or expanded
  candidates with dense zero-based expansion indices;
- incomplete-anchor markers when required anchor ingredients are unavailable.

Task 8 converts these candidates into canonical `VcIr`, assigns dense
within-snapshot `VcId`s, creates the final `SeedAccountingTable`, rejects
duplicates deterministically, and prepares fingerprint inputs. A matching
candidate key, source range, or future `VcId` is not cross-edit proof reuse.

## Task 6 Implementation Slice

Task 6 implements the first source module, `src/generator.rs`, for explicit core
seed families only. The public surface is a pre-normalized
`CoreGenerationCandidateSet` built from a validated `SeedIntakeTable` plus the
matching `ObligationSeedHandoff`.

Task 6 candidates preserve:

- the handoff id, seed origin, seed status, stable candidate sort key, and
  schema version;
- the selected `VcKind`, source reference, owner, local proof path, label,
  semantic origin, local context, symbolic premises, proof hint, goal formula,
  open status, provenance, and incomplete anchor;
- no-candidate records for skipped, deferred, error, missing-goal, and later
  generator-task seed kinds.

Before generating candidates, Task 6 recomputes the seed-intake table from the
same `ObligationSeedHandoff` and rejects the request unless the supplied
`SeedIntakeTable` matches it exactly. This rejects stale, partial, reordered, or
otherwise mismatched intake rather than silently omitting or resurrecting
obligations.

Task 6 supports these active seed kinds:

- `TheoremProof`;
- `DefinitionCorrectness`;
- `CheckerInitial`;
- `GeneratedNonEmptiness`;
- `GeneratedSethood`;
- `FraenkelMembershipAxiom`.

`AlgorithmContract`, `AlgorithmTermination`, and `GhostErasure` active seeds
are represented as visible `DeferredExternal` no-candidate records until task
7. Task 6 does not assign final `VcId`s, build a final `VcSet`, generate
algorithm VCs, transition status beyond `Open` / visible no-candidate records,
compute dependency slices, discharge obligations, call ATP, or activate corpus
fixtures.

Task 6 stable candidate sort keys are built from the module, generation schema,
owner, seed canonical key, source, core provenance, dense expansion index,
handoff id, and candidate kind. Context entries are sorted by core formula id
before dense context ids are assigned; original context insertion order is used
only as a tie-breaker for duplicate formula references.

Registration-style correctness is detected only from namespaced explicit
`CoreProvenance` markers such as `vc-registration-style:registration`,
`vc-registration-style:redefinition`,
`vc-registration-style:reduction`, or
`vc-registration-style:explicit-core-seed`. Labels, generic local paths, and
semantic-origin text alone do not classify a seed as registration-style
correctness. Otherwise `DefinitionCorrectness` and `CheckerInitial` remain
ordinary core/checker candidates or visible no-candidate records.

Explicit theorem status dependency markers such as `vc-theorem-status:clean`,
`vc-theorem-status:non-clean`, `vc-theorem-status:open`,
`vc-theorem-status:assumed`, and `vc-theorem-status:conditional` are preserved
as `VerifierPolicyInput` entries in the candidate local context. Absent
theorem-status payloads are not invented from labels, paths, semantic origins,
or source text.

Task 6 emits terminal proof goal candidates only when the upstream provenance
includes an explicit `vc-proof-goal:terminal` marker; otherwise `TheoremProof`
seeds remain proof-step candidates. Task 6 creates local definition unfold
requests only when an explicit `vc-unfold:*` provenance marker permits local
unfolding.

## Task 7 Implementation Slice

Task 7 extends `src/generator.rs` to generate algorithm candidates from
explicit flow-derived handoff rows. The input may include immutable
`ControlFlowOutput` so the generator can validate that a flow-derived seed's
`ControlFlowObligationSite` belongs to the referenced `ControlFlowIr` and can
classify loop-invariant phases from explicit CFG metadata. Raw source text,
labels, and generic semantic-origin strings are not algorithm payloads.

Task 7 may generate an open candidate for a deferred flow-derived seed when the
task-7 seed-intake rule has marked the row eligible and all of these conditions
hold:

- the row origin is `FlowDerived`;
- the entry has a `ControlFlowObligationSite`;
- the referenced `ControlFlowIr` exists in the supplied `ControlFlowOutput`;
- the seed kind and site kind form an owned task-7 algorithm family;
- the seed carries an explicit goal formula.

Seed intake preserves the original deferred seed status in bookkeeping while
using an eligible one-candidate mapping for goal-bearing flow rows. The
generated candidate uses `VcStatus::Open`. Task 8 owns the final seed-to-VC
mapping and must make this status transition auditable.

Task 7 supports these explicit goal-bearing site mappings:

- `Requires` -> `AlgorithmPrecondition`;
- `Ensures` -> `AlgorithmPostcondition`;
- `AlgorithmAssertion` and `StatementAssertion` -> `AlgorithmAssertion`;
- `AlgorithmInvariant` -> `LoopInvariant { phase: Entry }`;
- `LoopInvariant` -> `LoopInvariant` with phase `Entry`, `Preservation`,
  `Break`, or `Continue` when the supplied `ControlFlowIr` exposes the loop
  header or exit kind required to distinguish the phase.

Task 7 records these cases as visible no-candidate records:

- flow-derived algorithm rows without a `ControlFlowObligationSite`;
- rows whose referenced `ControlFlowIr` is missing from the input;
- algorithm rows with no explicit goal formula, including current
  `TerminationMeasure`, `PartialTermination`, `GhostPick`, and
  `GhostAssignment` rows;
- call-precondition, branch, match, range-loop, collection-loop, and Pick
  non-emptiness proof families not yet present as explicit handoff payloads;
- historical ghost-erasure rows as zero-VC/static-check integration data, not
  as a specification-backed proof family.

Task 7 algorithm context entries are symbolic and conservative. They may record
explicit site metadata such as site kind, ordinal, statement, block, loop id,
exit id, local id, assignment-effect id, and the matching flow id as
`VerifierPolicyInput` records or metadata-only local-context entries. They must
not invent path conditions, old-state assignment facts, alias identities,
post-havoc facts, hidden range values, branch facts, or ghost runtime facts when
those formulas are absent from the handoff or `ControlFlowIr`.

Task 7 may add a test-only `mizar-resolve` dev-dependency so Rust fixtures can
construct the `SymbolId` values required by `ControlFlowIr`. Production
`mizar-vc` code remains limited to `mizar-core` and `mizar-session` inputs.

## Task 8 Implementation Slice

Task 8 normalizes the task-6/task-7 pre-normalized candidate set into final
`VcSet` data. It assigns dense `VcId`s only within the current build snapshot,
orders concrete VCs by stable classification rank, candidate sort key, and
handoff id, and builds final seed accounting rows sorted by handoff id.

The task-8 `VcKind` classification rank is:

1. `TheoremProofStep`
2. `TerminalProofGoal`
3. `DefinitionCorrectness`
4. `RegistrationStyleCorrectness`, ordered by
   `Registration`, `Redefinition`, `Reduction`, `ExplicitCoreSeed`
5. `CheckerInitial`
6. `GeneratedNonEmptiness`
7. `GeneratedSethood`
8. `FraenkelMembershipAxiom`
9. `AlgorithmPrecondition`
10. `AlgorithmPostcondition`
11. `CallPrecondition`
12. `AlgorithmAssertion`
13. `LoopInvariant`, ordered by `Entry`, `Preservation`, `Break`,
    `Continue`, `Exit`
14. `RangeLoop`, ordered by `PositiveStep`, `RangeBound`, `HiddenIndex`
15. `CollectionLoop`, ordered by `Finiteness`, `OrderIndependence`
16. `Termination`
17. `PartialTermination` (existing validation/sort shape only; not
    source-generated by the current canonical specification)
18. `GhostErasureSafety` (existing validation/sort shape only; ghost isolation
    is a static check and not a source-generated VC)
19. `PolicyDeferredTraceability`

Future `VcKind` variants must be appended by an owning spec task before the
generator may emit them. Task 8 treats the existing candidate sort key as a
stable tie-breaker and duplicate-detection key, not as the sole source of kind
classification semantics.

Task 8 supports:

- `One` mappings for every concrete task-6/task-7 candidate;
- `NoConcreteVc` mappings for every visible no-candidate record;
- validation of the existing `Expanded` mapping shape in `VcSet`, while no
  task-8 generator family creates expanded candidates until an explicit
  expansion schema is introduced.

Normalization must reject duplicate candidate sort keys and duplicate seed
ownership deterministically. It must preserve source references, local
contexts, symbolic premises, proof hints, status, provenance, incomplete
anchors, and the original `seed_status` recorded by intake/generation. It may
append normalization provenance, but it must not discharge VCs, transition
policy status, compute dependency slices, call ATP, activate corpus fixtures,
or add new algorithm payload families.

## Task 20 Reuse-Identity Wiring

Task 20 completes the milestone's generated-obligation reuse identity wiring.
For every concrete generated candidate, the generator must build an
`ObligationAnchor` with stable source-shape, canonical-goal, and
canonical-context hash markers. The generator-owned source-shape hash is
available whenever source-shaped provenance is available, uses stable
source-shaped ingredients, and excludes `VcId`, source range, `SourceId`,
handoff id, candidate sort key, and dense owner row id. Canonical goal/context
hash markers are available only when `mizar-vc` has stable payloads for the
referenced formulas and context entries. Current CoreFormulaId-only generator
payloads therefore remain incomplete/conservative-unknown rather than hashing
raw upstream row ids.

Task 20 does not authorize cache reuse by itself. It only supplies stable
candidate ingredients to later consumers: complete `ObligationAnchor`,
canonical VC fingerprint, local-context fingerprint, dependency-slice
fingerprint, compatible verifier-policy fingerprint, and selected
proof-evidence hash. Matching `VcId`, source range, or anchor alone remains
insufficient.

## Task 30 Source-Derived Integration Contract

The exhaustive contract is canonical in
[source_vc_decomposition.md](./source_vc_decomposition.md). VC Task 31 must use
a narrow Core-aware adapter to validate the exact Task-180 direct terminal
proof-node relationship before candidate classification. The generic generator
currently sees only the seed handoff and would classify the real marker-free
seed as `TheoremProofStep`; Task 31 must instead classify that authenticated
structure as `TerminalProofGoal` without inserting or mutating provenance.

Tasks 32-55 own every broader source-derived family. In particular, a declared
entry `requires` is callee-body context and not a source-generated
`AlgorithmPrecondition` proof VC; loop normal exit and range hidden/bound values
are context unless an explicit canonical formula exists; simplification-order
and ghost isolation are static checks; no source `PartialTermination` or
`GhostErasureSafety` VC exists. A partial call exposes its successor
postcondition only with exact cited/in-context verified termination evidence.
Task 30 assigns no source-derived family to the existing `GeneratedSethood`
compatibility kind: explicit mode-declaration sethood uses
`DefinitionCorrectness`, and Fraenkel sethood is accepted prerequisite evidence
rather than a VC. Source generation of `GeneratedSethood` requires future
canonical authority and its first real source.
Generated formula,
substitution, binder, path, trace, and old-state representations are extended
only with the first real owning family, never by an empty infrastructure task.
Current source behavior that differs from these canonical rules is
`source_drift` assigned to the relevant descendant, not authority to preserve
that behavior.

## Planned Tests

Task 6 must add Rust coverage for:

- theorem/proof-step terminal goals with explicit local contexts and symbolic
  citations;
- clean, non-clean, open, assumed, and conditional theorem status dependency
  preservation;
- definition correctness candidates for available existence, uniqueness,
  coherence, compatibility, consistency, reducibility, sethood, and
  non-emptiness payloads;
- generated core obligation candidates for generated non-emptiness, generated
  sethood, and Fraenkel membership axiom seeds;
- registration-style correctness candidates when explicit checker/core payloads
  are available;
- unavailable registration-style payloads recorded as deferred/no-VC rather
  than fabricated candidates;
- proof hints and local unfold requests preserved symbolically.

Task 7 must add Rust coverage for:

- goal-bearing algorithm precondition, postcondition, assertion, and
  loop-invariant candidates from explicit flow-derived sites;
- visible no-candidate/deferred records for unavailable call-precondition,
  branch, match, range-loop, collection-loop, term-derived termination, Pick
  non-emptiness, plus historical partial/ghost rows that Task 30 classifies as
  evidence-admission or zero-VC/static boundaries rather than source-generated VCs;
- conservative symbolic context preservation for explicit flow-site metadata,
  including loop header/backedge and break/continue exit classification, while
  not inventing old-state assignment facts, field-update alias identity,
  post-havoc facts, range-loop hidden metadata, branch facts, or Pick facts
  that are not present in the payload;
- deterministic ordering of algorithm candidates independent of traversal
  helper map iteration.

Task 8 must add Rust coverage for:

- deterministic candidate normalization and dense `VcId` assignment;
- complete seed-to-VC accounting with no-VC and one-VC mappings, plus
  validation coverage for the existing expanded-mapping contract;
- duplicate candidate or seed ownership rejection;
- stable rendering/fingerprinting inputs for local contexts, generated
  formulas, and incomplete anchors.

No active `.miz` proof-verification fixture is added by task 5 because runner
support and source-derived extraction seams remain external gaps.

## VC Task 31 Exact Task-180 Adapter

Task 31 adds one public borrowed adapter,
`generate_exact_task180_vc(ExactTask180VcInput)`, backed by the private
`generator/task180.rs` leaf. The input carries an immutable `CoreIr`, the
target `BuildSnapshotId`, and explicit generation/VC schema versions. The
adapter derives `VcModuleRef` from the Core package and module using the
length-framed UTF-8 form
`package=<byte-len>:<package>;module=<byte-len>:<module>`; it never derives
module identity from a source path or debug text.

The adapter atomically validates the exact Core Task-31 public/valid theorem,
`False` formula, pending proof, direct terminal node, active theorem seed,
source maps, references, ranges, and provenance. It constructs and validates
the empty control-flow output, singleton `ExistingCore` handoff, and freshly
recomputed `EligibleOneVc` intake internally. Only after those checks does it
use the existing candidate builder and normalizer to return one open
`TerminalProofGoal`. Failure returns `ExactTask180VcError` and exposes no
partial candidate set.

This is an exact structural exception to the generic marker-based terminal
classification and closes that `design_drift` only for Task 180. No terminal
marker is injected. The adapter performs no discharge, `NeedsAtp` transition,
ATP/kernel/proof-policy execution, theorem acceptance, fact publication, or
general theorem/definition/algorithm generation.

## Public Enum Policy

Task 17 classifies every `generator` public enum as a downstream
forward-compatible API surface. Each enum must keep `#[non_exhaustive]` so later
candidate, normalization, and handoff validation errors can be added without
breaking downstream exhaustive matches.

| public enum | decision |
|---|---|
| `ExactTask180VcError` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `GeneratorError` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module. Internal
`mizar-vc` matches that intentionally enumerate current variants may remain
exhaustive.
