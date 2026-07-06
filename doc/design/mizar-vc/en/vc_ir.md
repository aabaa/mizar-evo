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
- symbolic `PremiseRef`s that later ATP translation may encode or prune;
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
- loop invariant entry, preservation, break, continue, or exit obligation;
- range-loop positive-step, range-bound, and hidden-index obligations;
- collection-loop finiteness and order-independence obligations;
- termination and partial-termination obligation;
- ghost-erasure safety;
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
