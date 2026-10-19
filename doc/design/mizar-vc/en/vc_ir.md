# Module: vc_ir

> Canonical language: English. Japanese companion:
> [../ja/vc_ir.md](../ja/vc_ir.md).

## Purpose

This module specifies the prover-independent verification-condition IR owned by
`mizar-vc`.

`VcIr` is the phase-11 output and phase-12 input. It records local contexts,
symbolic premises, goals, proof hints, source/core provenance, policy-visible
status, and cross-edit anchor ingredients without committing to TPTP, SMT-LIB,
backend process settings, proof witnesses, kernel acceptance, or artifact
publication.

Task 2 is specification-only. Rust data shapes are implemented by task 3; seed
intake by task 4; generation, normalization, and `VcId` assignment by generator
tasks 6-8; status and policy projection by task 9.

## Responsibility

Owned by this module:

- `VcId`, a deterministic id within one build snapshot;
- `VcKind`, the verifier-facing category of the obligation;
- `LocalContext`, the explicit formulas, binders, path conditions, and policy
  inputs available at the obligation site;
- symbolic `PremiseRef`s encoded by ATP translation; budget selection belongs
  to the VC-owned pretranslation assembly below;
- goal formulas as `VcFormulaRef`s into core or VC-local formula tables, not
  backend text;
- `ProofHint` records that preserve user hints, citations, unfolding requests,
  and computation requests without choosing an ATP backend;
- `VcStatus`, including `NeedsAtp` and policy statuses;
- seed intake and seed-to-VC accounting records;
- `ObligationAnchor` ingredients required by architecture 22.

Out of scope:

- parsing source syntax or recovering missing checker/core payloads;
- name resolution, type inference, registration activation, overload
  resolution, elaboration, or control-flow construction;
- concrete ATP translation, backend configuration, certificate validation,
  kernel replay, proof acceptance, cache lookup, and artifact publication;
- accepting unverified registrations or proof results merely because their
  seed, VC, or anchor exists.

## Gap Classification For This Spec

| ID | Class | Evidence | Handling |
|---|---|---|---|
| VCIR-G001 | `design_drift` | Architecture 07 says some seeds may generate zero or multiple VCs, while task 0 classified stale planning language that implied each seed always becomes one concrete VC. | This spec separates seed intake from VC expansion: every handoff entry is intake-tracked exactly once; concrete VC generation must record an explicit seed-to-VC mapping, with zero-VC skipped/deferred/error cases and any future multi-VC expansion made visible. |
| VCIR-G002 | `external_dependency_gap` | `mizar-core` currently carries registration/redefinition/reduction correctness through `DefinitionCorrectness`, `CheckerInitial`, provenance, and non-exhaustive seed kinds rather than dedicated VC seed kinds. | Preserve the available explicit seed/provenance data, classify unavailable payloads as external/deferred, and keep `VcKind` forward-compatible. |
| VCIR-G003 | `external_dependency_gap` / `deferred` | Task 2 originally treated `mizar-atp`, `mizar-kernel`, `mizar-proof`, and `mizar-cache` as inactive consumers. `mizar-kernel` tasks 23-31 now provide the checker-side formula/substitution evidence path, goal-polarity binding, and context-identity verification, and post-closeout VC tasks 24-28 provide the producer-side handoff builder, explicit goal-polarity declaration/rejection, reuse identity integration, and context-identity payload, but ATP candidate production, proof/cache consumers, and artifact witness consumers remain later work. | Keep `VcIr` prover-independent and record only untrusted deterministic evidence references, status states, reuse-candidate data, and task-24/28 handoff rules. Do not add SAT clauses, backend traces, or trusted acceptance fields. |
| VCIR-G004 | `test_gap` | No `vc_ir` Rust data shapes or tests exist before task 3. | Planned tests below become task 3 and task 8 obligations. |
| VCIR-G005 | `source_drift` / `test_gap` | After task 8, final `VcSet` normalization exists, but there is no deterministic API or focused test suite for projecting verifier policy into `NeedsAtp`, `PolicyOpen`, or `AssumedByPolicy` statuses. | Task 9 adds status-policy projection over immutable `VcSet` data, plus tests that status changes preserve context, proof hints, anchors, seed accounting, and ATP-bound obligations. |
| VCIR-G006 | `deferred` | Discharge evidence and dependency slices are now implemented by later VC tasks, post-closeout VC tasks 24-28 implement the producer-side kernel-evidence handoff, explicit goal-polarity declaration/rejection, reuse identity integration, and context-identity payload, and `mizar-kernel` tasks 30-31 implement the trusted goal-polarity and F2 membership checks. ATP translation, downstream proof/cache/artifact or corpus consumers, and source-derived corpus runner activation remain later tasks or external seams. | Task 9 must not generate discharge evidence, compute dependency slices, translate ATP problems, invoke kernel/proof/cache/corpus paths, or fabricate downstream integration records. |

No `doc/spec` or `.miz` tests change in task 2. This spec refines existing
architecture and core handoff contracts; it does not introduce new language
semantics.

## Conceptual Package Shape

The implementation may choose exact Rust field names, but it must preserve this
semantic shape:

```rust
struct VcSet {
    schema_version: VcSchemaVersion,
    snapshot: BuildSnapshotId,
    source: SourceId,
    module: VcModuleRef,
    generated_formulas: Vec<VcGeneratedFormula>,
    vcs: Vec<VcIr>,
    seed_accounting: SeedAccountingTable,
}

struct VcIr {
    id: VcId,
    kind: VcKind,
    source: VcSourceRef,
    seed: SeedVcRef,
    anchor: ObligationAnchor,
    local_context: LocalContext,
    premises: Vec<PremiseRef>,
    goal: VcFormulaRef,
    proof_hint: Option<ProofHint>,
    status: VcStatus,
    provenance: Vec<VcProvenance>,
}
```

`VcSet` is an immutable snapshot. Later phases may produce side tables or
status projections, but they must not mutate `CoreIr`, `ControlFlowIr`, or the
seed handoff.

`VcModuleRef` preserves the canonical module identity supplied by the producing
core snapshot without adding a new task-3 dependency on `mizar-resolve`. A later
boundary task may replace it with a direct resolved module id only if the
workspace dependency boundary and lint guard are updated in the same task.

`VcFormulaRef` identifies either a borrowed core formula or a VC-local generated
formula:

```rust
enum VcFormulaRef {
    Core(CoreFormulaId),
    Generated(VcGeneratedFormulaId),
}

struct VcGeneratedFormula {
    kind: VcGeneratedFormulaKind,
    formula: CoreFormulaShape,
    provenance: Vec<VcProvenance>,
}
```

The implementation may reuse existing `mizar-core` formula constructors or
normalization helpers, but VC-local generated formulas live in the `VcSet`, not
inside the upstream `CoreIr`. Generated formulas therefore get their own
snapshot-local ids and must carry enough provenance to reconstruct the seed,
expansion index, and generator rule that produced them.

## VcId

`VcId` is:

- deterministic within one `BuildSnapshot`;
- dense in canonical VC order;
- used for diagnostics, artifact ordering, result collation, and stable debug
  rendering inside the snapshot;
- never a cross-edit proof-reuse identity.

`VcId` assignment belongs to generator task 8. Earlier tasks may construct
placeholder or builder records only when they cannot escape as concrete `VcIr`.

Across edits, none of the following authorizes proof reuse by itself:

- matching `VcId`;
- matching source range;
- matching surface/syntax/arena id;
- matching `ObligationAnchor`.

Cross-edit reuse additionally requires the architecture-22 validation keys:
matching canonical VC fingerprint, local-context fingerprint, dependency-slice
fingerprints, compatible verifier policy, and matching proof witness or
deterministic discharge hash.

## VcKind

`VcKind` classifies the obligation for ordering, diagnostics, policy, and later
consumer filtering. Required categories are:

- theorem or lemma proof step;
- terminal proof goal;
- definition correctness;
- registration-style correctness when explicit core/checker seeds represent
  registration, redefinition, or reduction correctness;
- checker-initial carried obligation;
- generated non-emptiness or sethood obligation;
- generated Fraenkel membership axiom;
- algorithm precondition or postcondition;
- call precondition;
- algorithm assertion;
- loop invariant entry, preservation, break, or continue obligation; the existing
  exit shape is not source-generated merely because normal exit exists;
- range-loop positive-step obligation; existing range-bound and hidden-index
  shapes are not source-generated without explicit canonical formulas;
- collection-loop finiteness and order-independence obligations;
- termination obligation; the existing partial-termination shape is not
  source-generated by the current canonical specification;
- the existing ghost-erasure-safety shape as a validation/sort shape only;
  ghost isolation is a static check and not a source-generated VC;
- policy/deferred traceability record.

The Rust enum is downstream forward-compatible. Task 17 records the final public
enum policy, but implementation tasks must not expose an exhaustive public enum
without an owning spec update.

Ordering by kind must be stable and documented by task 8. Ordering must not
depend on hash-map iteration, worker completion order, or backend availability.

## Seed Accounting

The input is `mizar_core::control_flow::ObligationSeedHandoff`, which contains
existing core seeds and flow-derived seeds. `mizar-vc` must not inspect raw
syntax to reconstruct missing seed data.

Every handoff entry is intake-tracked exactly once:

```rust
struct SeedAccounting {
    handoff: ObligationHandoffId,
    origin: SeedOriginRef,
    seed_status: ObligationSeedStatus,
    mapping: SeedVcMapping,
}

enum SeedVcMapping {
    NoConcreteVc { reason: SeedNoVcReason },
    One { vc: VcId },
    Expanded { vcs: Vec<ExpandedVcRef>, expansion_schema: ExpansionSchemaVersion },
}

struct ExpandedVcRef {
    expansion_index: usize,
    vc: VcId,
}
```

Rules:

- no concrete `VcIr` exists without a seed accounting row;
- `Active` seeds with a goal are eligible for concrete VC generation;
- `Deferred` flow-derived `AlgorithmContract` seeds with a goal and supported
  explicit `ControlFlowObligationSite` metadata are also eligible for concrete
  VC generation, because `mizar-core` marks those rows deferred until
  `mizar-vc` applies the task-7 algorithm schema; the original deferred
  `seed_status` remains recorded for task-8 accounting;
- `Skipped`, `Deferred`, and `Error` seeds may produce no concrete VC when they
  carry a diagnostic or provenance reason;
- disabled seeds may use a visible no-VC mapping, while policy-open obligations
  must remain concrete VCs with `PolicyOpen` status; neither case may be a
  silent omission;
- multi-VC expansion is allowed only through an explicit `Expanded` mapping
  that records a stable zero-based dense `expansion_index` for each generated
  VC;
- ordinary theorem, definition, checker-initial, generated type, and eligible
  goal-bearing flow-derived milestone seeds use `One` unless their owning
  generator spec records a specific expansion schema;
- duplicate handoff entries with the same canonical seed key and origin must be
  rejected or represented as deterministic diagnostics before `VcId`
  assignment.

This resolves the task-0 drift: "exactly once" means every seed is accounted for
exactly once, while concrete VC cardinality is explicit and auditable.

### Task 4 Intake Table

Task 4 implements a pre-`VcId` intake table:

```rust
struct SeedIntakeTable {
    rows: Vec<SeedIntakeRow>,
}

struct SeedIntakeRow {
    handoff: ObligationHandoffId,
    origin: SeedOriginRef,
    seed_status: ObligationSeedStatus,
    canonical_key: ObligationSeedCanonicalKey,
    source: CoreSourceRef,
    mapping: SeedIntakeMapping,
}

enum SeedIntakeMapping {
    EligibleOneVc { goal: CoreFormulaId },
    NoConcreteVc { reason: SeedNoVcReason },
}
```

This table is not the final `SeedAccountingTable` stored in `VcSet`: it does
not assign `VcId`s and does not construct concrete `VcIr`s. It preserves the
handoff order and seed origin, rejects duplicate `(canonical_key, origin)` rows
deterministically, rejects any handoff row that lacks a matching `source_map`
entry before later `VcId` assignment, and records every skipped/deferred/error/
missing-goal row as a visible no-VC mapping except for task-7-eligible
goal-bearing deferred `FlowDerived` `AlgorithmContract` rows with supported
explicit flow-site metadata. Task 8 consumes eligible rows when deterministic
`VcId`s are assigned.

## LocalContext

`LocalContext` is self-contained. ATP translation must not reconstruct semantic
context from source text.

Required context components:

- binder declarations and their normalized roles;
- type predicates and sethood/non-emptiness facts that are available at the
  obligation site;
- proof assumptions, current thesis facts, and local labels in scope;
- cited premises from `by` justifications as symbolic references;
- algorithm path conditions, loop invariant availability, and post-havoc facts
  prepared from `ControlFlowIr`;
- checker facts, registration/cluster/reduction trace references, and inserted
  `qua` evidence when explicitly available;
- definition unfolding policy and selected local unfold requests;
- verifier policy inputs that affect status, dispatch, or computation limits.

Context entries carry stable source/core provenance and a canonical sort key.
They must be sorted deterministically before fingerprinting or debug rendering.

## PremiseRef

`PremiseRef` is symbolic. It never stores prover syntax, selected ATP encoding,
or backend-local axiom names.

Required premise reference classes:

- local context formula;
- local proof label or citation;
- theorem/lemma fact imported from accepted dependency artifacts;
- definition boundary or permitted unfolding;
- checker fact or type predicate;
- registration, cluster, or reduction trace reference;
- generated fact from core elaboration;
- policy-provided assumption marker when the active verifier policy allows it.

Unknown, incomplete, or unavailable premise data must not be interpreted as an
empty premise set. It must either make the dependency slice conservative or keep
the VC in a deferred/error status with diagnostics.

## Planned Pretranslation Selection Assembly

VC owns the immutable input assembly that consumes ATP's ordered budget
selection (§§21.4.1,22.4.4). Producers must supply an authenticated source
obligation, eligible local/cited facts and formula projections, visible-fact
indexes, the effective package/step budget and named ranking configuration.
The selected translation input must respect effective `max_axioms` across
the assembled local/cited axioms. The original VC, goal and full source
context remain unchanged. Selected rows
retain their formula, source/citation and context bindings; the selected
problem binds back to that source target and budget/configuration identity.
Suggestions are not eligible source-proof premises merely because they rank
well. Imported rows retain the [handoff authority](kernel_evidence_handoff.md#boundary-rules).

Reject over-cap selections, non-member rows, repeated selection of one assembled row, unavailable
or mismatched formula/provenance/context, and stale target/problem/configuration
identity before producing translation input. No partial/empty fallback or
reconstruction from display/backend data. The translator and existing handoff
builder remain unable to select or mutate premises. The current consumer APIs
do not yet provide this selected-input relationship; integration is blocked on
these authentic producers and assembly.

Future source tests compare within-budget and exact-budget inputs with all
eligible local/cited rows retained; an over-budget input retains the deterministic
top budget-sized axiom subset, including a cutoff tie, under input permutation. Exercise package
budget and step override. Compare the unchanged source VC/goal/context and every
selected binding. Corruption cases include an authentic over-cap selection and change target, row membership/repetition,
formula payload, provenance, context, imported authority and problem/configuration
identity one at a time; each rejects before translation. Failed-pruning diagnostics
and diagnostic retries remain ATP-owned, not proof acceptance by this assembly.

## Goal Formula

The goal is a `VcFormulaRef`: either a `CoreFormulaId` into the producing
`CoreIr` snapshot or a VC-local generated formula with explicit generated-origin
provenance.

The goal must not be:

- a source string;
- a parser/syntax id;
- backend text;
- an unchecked theorem statement copied from source;
- a formula invented to make a missing payload pass.

Generated conjunction or split goals must be stored in the `VcSet` generated
formula table and record generated-origin plus seed expansion provenance so
task 8 and task 20 can fingerprint them deterministically without mutating
`CoreIr`.

## ProofHint

`ProofHint` preserves author and policy inputs. It is not a backend dispatch
configuration.

Required hint data:

- explicit `by` citations and local labels;
- local unfold requests and definition opacity overrides;
- `@proof_hint`-style premise restrictions or backend-abstract intent;
- `@proof_hint` `solver`, `max_axioms`, and `timeout` options as symbolic
  policy inputs;
- `by computation` or verification-time computation requests;
- computation-limit policy references, once task 10 fixes the limit model;
- source/core provenance for every hint.

Unsupported hints must be represented as diagnostics or deferred status records.
They must not silently drop context or force ATP dispatch.

## VcStatus

`VcStatus` records the phase-11/12 verifier state visible to later consumers.

Required states:

- `Open`: generated and not yet discharged or classified for ATP;
- `NeedsAtp`: canonical VC must be translated by `mizar-atp`;
- `Discharged`: phase-12 produced deterministic, replayable, untrusted evidence;
- `PolicyOpen`: active policy intentionally leaves the VC open and does not
  dispatch ATP;
- `AssumedByPolicy`: active policy accepts an assumption marker but not proof
  evidence;
- `SkippedDueToInvalidInput`: seed or owner was invalid/skipped and no concrete
  proof obligation is dispatched;
- `DeferredExternal`: a required upstream/downstream seam is unavailable;
- `Error`: deterministic VC generation failed.

Status rules:

- `NeedsAtp` VCs must retain their full local context, premises, source, anchor,
  and proof hints;
- policy states never erase the underlying seed accounting or source
  provenance;
- `Discharged` evidence is not kernel-verified proof status;
- a status change must be deterministic and auditable;
- status transitions belong to task 9 and discharge tasks, not to raw data-shape
  constructors.

## Task 9 Implementation Slice

Task 9 adds a deterministic status-policy projection over an immutable `VcSet`.
The projection returns a new `VcSet` and must not mutate the input set.

Task 9 supports:

- preserving the current status;
- marking concrete VCs as `NeedsAtp`;
- marking concrete VCs as `PolicyOpen` with an explicit policy key;
- marking concrete VCs as `AssumedByPolicy` with an explicit policy key and
  premise marker.

Policy overrides must be sorted by `VcId`, must not duplicate a `VcId`, and
must target an existing VC. Missing targets, duplicate overrides, and unsorted
overrides are deterministic errors. `AssumedByPolicy` markers are validated by
the existing `VcSet` validation path, so invalid context, premise, or generated
formula references fail closed.

A status change must append `StatusPolicy` provenance to the changed VC. A
preserve/no-op action must not add provenance. Projection must preserve
`VcId`, order, kind, source refs, seed refs, anchors, local contexts, premises,
goals, proof hints, generated formula tables, seed accounting, and existing
non-status provenance. It must not create `Discharged` evidence; deterministic
discharge tasks own evidence creation. It must not compute dependency slices,
call ATP, activate corpus fixtures, accept kernel/proof/cache results, or add
new generator payload families.

## ObligationAnchor

`ObligationAnchor` is a best-effort cross-edit candidate identity. It is not
proof evidence and is not trusted by the kernel.

Required ingredients:

- normalized owner identity: theorem, definition, registration, generated
  symbol, algorithm, proof block, or checker-origin row;
- `VcKind`;
- anchor-ready `LocalProofOrProgramPath` from the seed handoff;
- label role and optional label hint, where available;
- normalized semantic origin;
- source/core provenance, including source range when available;
- source-shape hash or conservative unavailable marker;
- canonical goal hash;
- canonical local-context hash;
- generation schema version.

`VcId`, `SourceRange`, `SurfaceNodeId`, rowan identity, arena ids, parser order,
and handoff-local ids may appear as snapshot-local evidence, but they must not
be used as cross-edit proof-reuse identity.

If any required anchor ingredient is unavailable, the anchor must be marked
incomplete and downstream proof/cache reuse must fail closed.

## Task 20 Fingerprints

Task 20 adds deterministic cross-edit fingerprint helpers for generated
obligations:

- `CanonicalVcFingerprint` covers the VC kind, canonical goal payload,
  symbolic premises, proof hints, and generated formula payloads resolved from
  the owning `VcSet` generated-formula table.
- `LocalContextFingerprint` covers local-context entries by stable sort key,
  kind, resolved formula payload, provenance, and explicit verifier-policy
  inputs.

These fingerprints exclude `VcId`, source range, `SourceId`, handoff ids,
candidate sort keys, and row ids that are local to one build snapshot. Generated
formula references are resolved to formula kind/shape/provenance payloads before
hashing; unresolved generated formula references in an invalid set must fail
validation before they can become reuse inputs. Opaque upstream row identifiers
such as `CoreFormulaId`, `CoreDefinitionId`, and dense owner ids are not
cross-edit payloads. If the stable formula, definition, owner, or context
payload is unavailable to `mizar-vc`, the fingerprint helper must return no
fingerprint and downstream reuse must fail closed.
Quantified generated formulas are also fail-closed in Task 20 unless stable
binder-entry payloads are available; binder counts or `ContextEntryId`s alone
are not enough to form a canonical VC fingerprint.

Task 20 also wires generated `ObligationAnchor` values with source-shape,
canonical-goal, and canonical-context hash markers. The source-shape hash is
available when source-shaped provenance is available and is derived from stable
ingredients such as owner class, `VcKind`, local proof/program path, label,
semantic origin, and source/core provenance markers. It must not be derived from
`VcId`, source range, `SourceId`, handoff id, candidate sort key, or dense owner
row id. Canonical goal/context hash markers are available only when the stable
payloads are available; current CoreFormulaId-only goals and context entries
remain incomplete/conservative-unknown reuse inputs.

## Deterministic Rendering

Task 3 implements a deterministic debug rendering for `VcIr` and related
tables. Rendering must include:

- schema version;
- sorted VC rows by `VcId`;
- seed accounting rows;
- source/core provenance summaries;
- local context entries and symbolic premises in canonical order;
- status and policy references;
- anchor ingredients and incomplete-anchor markers.

Rendering must not include nondeterministic addresses, timings, worker ids,
absolute local paths, hash-map iteration order, or backend runtime data.

## Planned Tests

Task 3 must add Rust coverage for:

- constructing a minimal `VcIr` with explicit context, premise refs, goal, hint,
  anchor, status, and seed accounting;
- rejecting duplicate `VcId`s or unsorted/invalid accounting when constructors
  validate complete sets;
- preserving symbolic premise refs without backend text;
- ensuring `NeedsAtp` and policy statuses do not drop context;
- rendering the same fixture byte-identically across runs;
- preserving incomplete anchor/dependency markers as cache-miss data.
- storing generated/split goals in the VC-local generated formula table without
  mutating `CoreIr`.

Task 4 and task 8 extend coverage for deterministic seed intake, duplicate seed
rejection, seed-to-VC mapping, and `VcId` assignment. Task 9 extends coverage
for deterministic status-policy projection, `NeedsAtp` classification,
policy-open and policy-assumed statuses, rejection of invalid overrides, and
preservation of context, proof hints, anchors, generated formulas, and seed
accounting across status changes.

No active `.miz` fixture is added by task 2 because the `proof_verification`
runner and source-derived payload seams remain external gaps.

## Task 30 Source-Derived Identity Contract

[source_vc_decomposition.md](./source_vc_decomposition.md) freezes the exact
Task-180 phase-11 identity. Task 31 produces one snapshot-local `VcId(0)` with
kind `TerminalProofGoal`, status `Open`, Core formula 0, empty context/premises,
the exact theorem owner/path/source/origin, and one Active/ExistingCore seed
mapping. Because the current VC payload carries only a Core formula id, its
canonical-goal hash remains unavailable; source-shape and empty-context hashes
may be available, but the anchor remains explicitly incomplete and cannot
authorize proof reuse.

Tasks 32-55 may extend `VcKind`, generated-formula shapes, and context roles only
inside the first nonempty real-source task that needs an honest canonical
representation. They must not encode algorithm type correctness or explicit
match exhaustiveness as unrelated existing kinds, flatten dependent binder or
path order, synthesize formula identity from labels, or generally promote
Deferred input to Open. Every extension retains `#[non_exhaustive]` policy and
lands with validation, deterministic rendering, corruption tests, and a real
`MT10-VC-PV/VC<n>` baseline.

Existing `PartialTermination`, `GhostErasureSafety`, loop-exit, range-bound,
and hidden-index variants do not themselves authorize source generation.

## Public Enum Policy

Task 17 classifies every `vc_ir` public enum as a downstream
forward-compatible API surface. Each enum must keep `#[non_exhaustive]` so later
VC, policy, evidence, dependency, and diagnostic categories can be added without
breaking downstream exhaustive matches.

| public enum | decision |
|---|---|
| `VcStatusAction` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `VcGeneratedFormulaKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `VcGeneratedFormulaShape` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `QuantifierKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `VcFormulaRef` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `VcKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `RegistrationCorrectnessKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `LoopInvariantPhase` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `RangeLoopObligation` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CollectionLoopObligation` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ContextEntryKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `PremiseRef` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DefinitionOpacityOverride` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `PremiseRestriction` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ComputationHint` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `VcStatus` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `SeedIntakeMapping` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `SeedOriginRef` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `SeedVcMapping` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `SeedNoVcReason` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `AnchorOwner` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `AnchorLabelRole` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `AnchorCompleteness` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `AnchorIngredient` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `VcProvenancePhase` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `HashMarker` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `VcIrError` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module. Internal
`mizar-vc` matches that intentionally enumerate current variants may remain
exhaustive.

## Concrete return equality

`VcGeneratedFormulaKind::AlgorithmPostcondition` with `VcGeneratedFormulaShape::Equals { left: CoreTermId, right: CoreTermId }` stores the actual substituted equality over existing immutable Core terms. Its generator authenticates both references and ownership against Core; no parallel term representation or string formula is introduced. Debug bytes preserve both ordered operands. Without Core payload, canonical identity and dependency/reuse analysis remain unresolved, and discharge returns Unknown; equal numeric ids alone prove nothing.

`VcProgramValue { var: CoreVarId, definition: Option<CoreAlgorithmStmtId> }` identifies an immutable parameter with None or a value after its actual declaration/write or runtime Pick with Some; Some(loop statement) additionally identifies the arbitrary cutpoint value of a validated MayWrite local, without implying an assignment equality; the generator validates owner, destination, source order and definition-before-use against Core.
`ProgramEquals` and `ProgramTypePredicate` store these values and actual checked predicates; AlgorithmStateFact distinguishes declaration/write context from AlgorithmAssertion goals. Core remains immutable, and unknown Core payload keeps truth, fingerprints and reuse conservative.
`ContextEntryKind::PendingAlgorithmAssertion { handoff }` links a later assumption to the exact earlier assertion goal through One seed accounting; validation rejects wrong owners, goals, self-dependencies and cycles. It remains an unresolved proof dependency, never a direct/false accepted premise or reusable evidence.

## Builtin choice accounting

`SeedNoVcReason::BuiltinSetInhabitation { origin: GeneratedOriginId }` is the [source
generator](./generator.md#source-existential-registration-proof) accounting case, not
skipped/deferred work or Discharged evidence. VcSet structural validation requires an Active ExistingCore
seed origin; only full Core regeneration can authenticate the referenced generated origin and real
seed goal. Generic consumers must not derive truth or acceptance from this caller-constructible
value.

## Source registration canonical VC fingerprints

`VcSet::source_registration_canonical_fingerprints(&self, core: &CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>) -> Result<BTreeMap<VcId, CanonicalVcFingerprint>, String>` covers only the existing single/three existential-registration source generator. Regenerate the complete VcSet with its snapshot, VC schema and first leaf's generation schema; accept only exact Open regeneration or exact default try_discharge VcSet. Preserve upstream errors; other unsupported/mismatched/empty inputs return internal unsupported source registration VC fingerprint without a partial map. Existing generic fingerprint methods, generation, statuses and inputs remain unchanged.
The caller must provide genuine checked-lowered Core and unmodified same-source [Core formula payloads](../../mizar-core/en/elaborator.md#source-registration-formula-payloads) for exactly the Core references reached by all goals and generated-fact premises. Require nonempty values and exact key coverage; equal values at different roots are valid. These checks and generator replay do not authenticate opaque formula bytes, the checker seal, source currentness or execution provenance. A future fresh source factory must bind those genuine producers in one call before trusted consumption.
Resolve current generated references recursively through the actual table, rejecting missing/cyclic/unsupported data atomically. This profile uses only kinds GeneratedTypeObligation, NegatedPremise and Conjunction and shapes Ref, Not, And and Or. Whole source regeneration authenticates their existing dense CoreHandoff provenance; normalize it to the fixed rule source-existential-registration-v1 plus encoded kind, ordered shape and genuine Core content rather than debug node/row references. Preserve child order. Only actual empty local entries/policy inputs, absent hint and GeneratedFact premises are supported.
Formula bytes use tag 0 plus framed opaque Core bytes, or tag 1, kind byte 1 GeneratedTypeObligation / 2 NegatedPremise / 3 Conjunction, framed UTF-8 source-existential-registration-v1, and shape byte 1 Ref / 2 Not / 3 And / 4 Or. Ref/Not append one recursively encoded child; And/Or append checked big-endian u32 child count then ordered children. Every frame is checked big-endian u32 byte length plus bytes. No dense/source/snapshot id, range, binder display spelling or debug provenance enters this profile.
Each VC payload begins UTF-8 mizar-vc/source-registration-canonical-vc/v1 and zero; append framed VC schema and generation schema, kind byte 1 for RegistrationStyleCorrectness/Registration, framed goal bytes, checked big-endian u32 premise count and each lexicographically sorted premise byte vector framed separately. Each premise vector is byte 1 GeneratedFact followed by its encoded formula. Preserve duplicate multiplicity; sort resolved bytes, never reference ids. Append three zero bytes for empty context entries, empty policy inputs and absent proof hint. Wrap unchanged stable_fingerprint_hash with domain mizar-vc-source-registration-canonical-vc in the existing CanonicalVcFingerprint; keys remain current VcIds.
This fingerprint identifies bounded VC content under explicit input responsibilities. Status, anchor, source-shaped identity, complete dependency composition, deterministic discharge evidence hash, policy/witness identity and proof/cache/publication eligibility remain separate. Existing incomplete markers and generic methods remain conservative; this API neither mutates them nor makes a reuse key, Complete result or accepted witness.
Tests use genuine single/aggregate source setup and current owner/leaf associations with independent frozen-format hash expectations; Open/default-discharge equality and input immutability; neutral edits, actual allocator relocation, declaration/registration reordering and resolved-premise order, plus body/owner/module/schema participants; genuine foreign graphs and missing/empty/extra actual-root payloads, altered structural VC data and empty-set refusal. Reuse existing Core/owner profile tests without copying their full matrices.

## Source registration canonical goal and context

`VcSet::source_registration_canonical_goal_context(&self, vc: VcId, core: &CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>) -> Result<(Hash, Hash), String>` returns the selected current leaf's goal hash followed by its context hash.
First invoke [source canonical VC validation](#source-registration-canonical-vc-fingerprints) on the entire set and exact root map; preserve every upstream error. Then require the selected VcId to exist, otherwise return the same internal unsupported source registration VC fingerprint error. Return both hashes atomically and leave all inputs unchanged.
The linked profile's genuine checked Core, unchanged same-source opaque Core55 bytes, supported source graph, Open/default-discharge association and caller-owned provenance/currentness duties apply without relaxation. Share its existing frame/formula encoders without changing any old bytes or generic API.

Encode the selected goal and each visible GeneratedFact premise with that formula grammar. Context includes the actual witness-type and expanded-definition premises, their duplicate multiplicity, empty local entries/policy inputs and absent hint. This is a source-specific context profile; existing generic LocalContextFingerprint retains its local-row profile.
Goal payload starts UTF-8 mizar-vc/source-registration-canonical-goal/v1 and zero, then framed VC schema, framed generation schema and framed encoded goal. Hash with unchanged stable_fingerprint_hash domain mizar-vc-source-registration-canonical-goal.
Context payload starts UTF-8 mizar-vc/source-registration-canonical-context/v1 and zero, then framed VC schema and generation schema, checked big-endian u32 premise count, and each lexicographically sorted premise vector framed separately. Each vector is byte 1 GeneratedFact followed by encoded formula; append three zero bytes for empty local entries, empty policy inputs and absent hint. Hash with domain mizar-vc-source-registration-canonical-context. Frames retain checked big-endian u32 byte lengths.
Neither component substitutes the whole VC fingerprint, a Kernel/Artifact hash, source-shaped identity, a complete composed dependency footprint or accepted evidence. Existing anchor markers, dependency completeness, statuses, policy, freshness, reuse eligibility and publication remain unchanged; future source assembly must bind genuine producers before trusted consumption.

Three test families reuse genuine source setup and current owner/leaf associations: independent frozen-format goal/context expectations and Open/default-discharge equality; neutral relocation/reordering, resolved-premise content order, schema/owner/module participants and a definition-body edit that changes affected contexts while preserving goals; whole-input/selected-leaf refusal, actual upstream errors and API input immutability. Preserve existing canonical VC expectations and reuse earlier producer/profile tests without copying their full matrices.

## Source registration source-shape hashes

`VcSet::source_registration_source_shape_hashes(&self, core: &CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>, syntax: &BTreeMap<CoreItemId, Vec<u8>>) -> Result<BTreeMap<VcId, Hash>, String>` first invokes the existing whole [source canonical VC validation](#source-registration-canonical-vc-fingerprints), preserving every upstream error. The caller must associate each current Core Registration item with the unchanged [Resolver declaration syntax payload](../../mizar-resolve/en/symbols.md#source-registration-syntax-payloads) of that item's actual SymbolId. Require exactly all current Registration item keys and nonempty, pairwise-distinct values. Keys are same-graph lookup handles only; never serialize their ids or SymbolId representations. No new Resolver production dependency or source parsing is introduced.
Traverse actual SeedAccounting Expanded rows, ignoring NoConcreteVc rows; refuse other mappings. Require ExistingCore origin, its actual seed and Registration owner item, seed kind CheckerInitial and local_path `registration/existence`, and expansion schema `source-existential-registration-v1`. Use each actual ExpandedVcRef's current target and expansion_index; require exactly the complete current VC set without duplicate targets. Existing whole-source replay supplies the genuine bounded expansion structure. Later profile/association/framing failures return `unsupported source registration source shape` atomically, with no partial map.
For each leaf, encode UTF-8 `mizar-vc/source-registration-source-shape/v1` plus zero, then checked big-endian u32 byte-length frames of VC schema, that leaf's generation schema and its actual expansion schema, in order. Append byte 1 for this validated registration-correctness kind, a frame of the actual validated seed local_path, checked big-endian u32 expansion_index, then one frame of the unchanged syntax bytes. Apply the existing stable_fingerprint_hash with domain `mizar-vc-source-registration-source-shape`. Reuse the existing source_registration_frame; convert its framing refusal to this operation's profile error. No other field is hashed.
This deliberately bounded local syntax profile includes genuine named-owner content already present in Resolver bytes, authored token text/shape, source seed path and expansion role. Corresponding values survive trivia, allocation, declaration reordering and edits outside that declaration, including referenced attribute bodies. Owner/module, represented local tokens/shape, schema or leaf role changes participate. Canonical semantic goal/context values remain separately owned and must still invalidate reuse when external bodies change.
Opaque byte provenance and source/snapshot currentness remain caller responsibilities: structural replay, key coverage and nonempty/distinct values do not authenticate supplied bytes as fresh Resolver output. Future Proof assembly must bind the actual producers in one call. Results are raw content hashes, not an execution receipt or complete normalized anchor/semantic origin, composed dependency/Proof identity, acceptance/reuse eligibility or publication. Do not mutate VcSet/Core, generic source-shape markers, anchor completeness, statuses or legacy fingerprint behavior.
Tests reuse real single/aggregate source pipelines and Resolver syntax production, joining actual Core symbols to current item keys in test plumbing only. Independent manual source grammar and frozen schema/path/leaf/hash framing supply literal expectations before Rust implementation. Three families cover exact current leaves/default discharge; neutral and participating edits, snapshots and actual owner-id relocation, external-body stability and distinct leaves; whole-input/upstream refusal, missing/extra/empty/duplicate syntax maps and unchanged inputs/original anchors. Do not treat opaque-map structural checks as an authenticity proof or fabricate a positive source association.

## Source registration candidate anchor hashes

`VcSet::source_registration_candidate_anchor_hashes(&self, core: &CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>, syntax: &BTreeMap<CoreItemId, Vec<u8>>, owners: &BTreeMap<CoreItemId, Vec<u8>>) -> Result<BTreeMap<VcId, Hash>, String>` first obtains the complete [source-shape map](#source-registration-source-shape-hashes), preserving every upstream error. Require owner keys exactly equal the validated syntax keys, with nonempty pairwise-distinct values; the caller must supply unchanged Resolver registration owner payloads for each actual Core item's SymbolId. Handles are lookup associations only. Structural validation does not authenticate opaque owner/syntax/formula provenance or currentness; never decode or reinterpret Resolver bytes.
For every actual current VC, use its replay-validated Registration owner and anchor local_path, its unchanged syntax-derived shape and its genuine [canonical goal/context](#source-registration-canonical-goal-and-context). Return exactly all current VcIds or an atomic error; later owner/framing/profile refusal is `unsupported source registration candidate anchor`. Reuse source_registration_frame and stable_fingerprint_hash; no new encoder or production dependency.
Encode UTF-8 `mizar-vc/source-registration-candidate-anchor/v1` plus zero; checked big-endian u32 byte-length frames of VC schema, that leaf's generation schema, unchanged registration owner bytes and actual anchor local_path, in order; byte 1 for the validated registration-correctness kind; then exactly 32 raw bytes each of source-shape, canonical goal and canonical context hashes, in that order. Hash with domain `mizar-vc-source-registration-candidate-anchor`. Only those fields participate. The owner payload already contains genuine package/module/registration-label semantic origin; the source-shape profile includes the actual expansion role, distinguishing leaves sharing a seed path. Do not manufacture a label hint where the original anchor has none.
This normalized candidate content excludes dense owner ids, VcId, SourceId, snapshot, ranges and provenance representations. Corresponding values survive supported relocation/reordering/trivia/outer-binder edits. Represented local syntax/owner/schema changes participate; external definition-body edits can preserve local shape while changing semantic context and hence the candidate. This is a raw candidate hash, not a structured Complete anchor, canonical VC substitute, dependency fingerprint, trusted receipt, reuse authorization or accepted status. Preserve every original graph/anchor/marker and generic method.
Three shared integration-test families exercise this operation and genuine same-call Proof assembly: independent frozen literal candidate values for single/aggregate current leaves and lower-producer equality; neutral/participating content and actual id relocation, distinct expansion leaves and external shape/context separation; whole-source upstream/profile failure, missing/extra/empty/duplicate owner maps, absent Proof selection, genuine foreign/malformed/unsupported source and immutable inputs. Retain all old component/profile tests rather than copying their complete matrices.

## Source theorem canonical components

`VcSet::source_theorem_canonical_components(&self, vc: VcId, core: &mizar_core::core_ir::CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>) -> Result<(CanonicalVcFingerprint, Hash, LocalContextFingerprint), String>` returns bounded VC, goal and local-context content in that order, using existing wrappers in `crates/mizar-vc/src/vc_ir.rs`.
First call unchanged [build_source_theorem_kernel_handoff](./kernel_evidence_handoff.md#source-theorem-local-handoff)(core, self, vc), preserve every upstream error and discard the untrusted handoff. Reuse its whole original Open, single uncited Valid Theorem/one proof-let set/reflexive local-goal profile and validated structure; no discharged/projected graph, second broad validator, Kernel/ATP execution or handoff-hash reinterpretation. Later profile/map/framing failures are exactly internal `unsupported source theorem canonical components`; return atomically without mutation, partial results or public diagnostics.
The caller supplies genuine checked-lowered Core and unchanged same-source [Core theorem formula payloads](../../mizar-core/en/elaborator.md#source-theorem-formula-payloads) for exactly the two distinct actual local guard/goal roots, both nonempty. Equal opaque values are allowed; byte provenance remains caller-owned. Replay authenticates structural same-Core VC association, not a Checker seal, supplied bytes or currentness. Never parse, reconstruct or debug-encode content.
Require one actual ProofAssumption local entry with Core guard, actual Core goal, exactly its one LocalContext premise, empty policy inputs and absent hint. Require actual sort_key `core-context-formula-{guard.index():08}-source-ordinal-00000000` and exactly one CoreHandoff provenance with key `context-formula-{guard.index()}`, Some CoreProvenance phase Checker and key `checker/theorem/{actual item.symbol.fqn()}/skeleton/terminal` for the actual sole item. Normalize only this verified ordinal zero and provenance to the roles below; other provenance refuses. Raw FQN contains contribution/structural-path/slot fields: use it only to validate association, never as canonical owner content.
Reuse unchanged `source_registration_frame` and `stable_fingerprint_hash`, mapping framing refusal to this profile error. Let F(b) be checked BEu32 byte length plus unchanged bytes; all u32 below are BEu32. For x = vc, goal, context, payload starts UTF-8 `mizar-vc/source-theorem-canonical-{x}/v1` plus zero, F(actual VC schema UTF-8), F(actual generation schema UTF-8); respective domains are `mizar-vc-source-theorem-canonical-vc`, `mizar-vc-source-theorem-canonical-goal`, `mizar-vc-source-theorem-canonical-context`.
Entry content E is u32(0) normalized source ordinal, byte1 ProofAssumption, F(actual guard bytes), u32(1) provenance count, byte1 CoreHandoff phase, F(UTF-8 `context-formula`) normalized verified key role, byte1 Some core provenance, byte1 Checker phase, F(UTF-8 `source-theorem-local-proof/terminal`) normalized verified Checker role. E is only a content frame, not a new representation.
Goal suffix is F(actual goal bytes). Context suffix is u32(1) entry count, F(E), u32(0) policy-input count. VC suffix is byte1 TheoremProofStep kind, F(actual goal bytes), u32(1) premise count, byte1 LocalContext premise, F(E) actual resolved entry, u32(0) policy-input count, byte0 absent hint. Counts/roles encode actual validated content; nonempty policy inputs refuse before hashing.
Only those fields participate; schemas affect all three. Goal contains no owner/path/hint/context beyond Core payload module/role/formula bytes; context retains verified source-profile phases/roles and normalized guard ordinal. Exclude dense/source/snapshot/handoff/row/arena IDs, ranges, binder display spellings, status and Debug representations. Never substitute source syntax/anchor/dependency or Kernel/Artifact hashes. Same-profile owner rename, binder spelling, trivia and snapshot changes preserve all three; source-shaped owner/syntax remains separately unavailable.
Append exactly two `source_theorem_canonical_components_*` families inside existing `term_proof_tests` in `crates/mizar-test/src/runner/proof_verification.rs`, preserving all old bytes/final module brace. Reuse `with_step5c14_theorem_check`, `lower_source_theorem_skeletons`, `source_theorem_formula_payloads` and `generate_core_vcs`; no helper/extraction change. Positive producer claims require genuine source pipelines, actual Core guard/goal maps and the original VC graph; malformed actual-derived containers/maps are negative-only.
Family one covers independent frozen-format literal VC/goal/context expectations prepared and reviewed before Rust from this grammar, actual module/schema and unchanged hash primitive, with artifacts outside doc/. Compare original T, same-owner distinct bound/local binder spellings, trivia and genuinely renamed owner U; require determinism, input Core/map/VcSet purity, old generic fingerprint None and original incomplete anchors unchanged.
Genuine snapshot changes preserve all three; alternate VC/generation schemas must genuinely regenerate through existing CoreGenerationCandidateSet/intake/normalization and change all three. Inline existing setup if needed; relabeling an old container is no positive producer. Claim allocator relocation only if a real source producer supplies it; no fabricated Accepted/Hit/Complete or successful proofless-Open lowering.
Family two refuses empty/missing/empty-value/extra maps and original-bound-root substitution using actual roots; absent VcId, genuine foreign same-profile Core/VC association, and whole-graph status/context/provenance/sort/policy/premise/hint drift. Include one explicitly malformed/source-unbacked Core negative changing only the sole seed Checker provenance key while preserving valid source entries; genuinely regenerate its generic VC graph and require VC69 replay success but this API's later profile refusal. Original Core56 bytes are negative operands only, not authenticated to mutated Core; assert no source seal, genuine producer or acceptance and change no lowerer/helper. With absent/foreign inputs AND empty map compare the first error directly to actual handoff replay, detecting wrapped/reordered validation. Every case reaches its asserted API/oracle; reuse existing Core56/VC69 broader matrices rather than copying them.
Existing generic fingerprints/reuse, original anchors/incomplete markers, conservative dependencies, source owner/syntax production, same-call Proof identity/classification/witness, freshness/currentness/cache/publication, broader Core/Theorem/ATP profiles and Steps 6–8 remain open. These raw components are neither complete identity nor a trusted producer receipt; Checker/Core/Kernel/Proof ownership and proof acceptance stay unchanged.

## Source theorem candidate anchor hash

`source_theorem_candidate_anchor_hash(owner: &[u8], syntax: &[u8], vc: CanonicalVcFingerprint, goal: Hash, context: LocalContextFingerprint) -> Result<Hash, String>` is an additive free function in `crates/mizar-vc/src/vc_ir.rs` for opaque raw ingredient hashing only.
Caller supplies unchanged real same-profile owner/syntax bytes and canonical components. This function does not replay a graph, validate owner/syntax grammar or authenticate producer, profile, byte provenance or currentness; never parse or re-encode opaque bytes.
Reject empty owner/syntax and checked framing overflow with exactly internal `unsupported source theorem candidate anchor`; return no partial output or mutation. Reuse unchanged source_registration_frame and stable_fingerprint_hash without a new primitive.
Let F(b) be checked big-endian u32 byte length followed by unchanged bytes. Payload is UTF-8 `mizar-vc/source-theorem-candidate-anchor/v1` plus NUL, byte1 TheoremProofStep, F(UTF-8 `source-theorem-local-proof/terminal`), F(owner), F(syntax), then raw 32 bytes each of vc.hash(), goal and context.hash(), in that order.
Hash with domain `mizar-vc-source-theorem-candidate-anchor`. Fixed kind/terminal describe this version's caller-supplied ordinary singleton profile without establishing it; another role/profile requires a separately reviewed version.
Schemas already participate in canonical components; insert no additional standalone dense-id, raw-FQN, range, snapshot, VC-status, policy, evidence or dependency fields; retain declaration status inside opaque owner bytes. Preserve original structured anchors, incomplete markers, generic fingerprints, source/dependency completeness and cache-miss status.
The result confers no trust, completeness, acceptance, reuse or publication authority. [Proof assembly](../../mizar-proof/en/status.md#source-theorem-original-obligation-identity) owns genuine source association, owner JSON, obligation composition and shared test design.
