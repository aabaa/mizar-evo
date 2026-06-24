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
intake by task 4; generation, normalization, and `VcId` assignment by later
generator tasks.

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
| VCIR-G003 | `external_dependency_gap` | `mizar-atp`, `mizar-kernel`, `mizar-proof`, and `mizar-cache` are not active workspace consumers. | Specify only prover-independent IR, untrusted deterministic evidence references, status states, and reuse-candidate data. |
| VCIR-G004 | `test_gap` | No `vc_ir` Rust data shapes or tests exist before task 3. | Planned tests below become task 3 and task 8 obligations. |

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
    module: ModuleId,
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
- `Skipped`, `Deferred`, and `Error` seeds may produce no concrete VC when they
  carry a diagnostic or provenance reason;
- disabled or policy-open seeds must be represented by a visible mapping and
  status, not by silent omission;
- multi-VC expansion is allowed only through an explicit `Expanded` mapping
  that records a stable `expansion_index` for each generated VC;
- ordinary theorem, definition, checker-initial, generated type, and current
  flow-derived milestone seeds use `One` unless their owning generator spec
  records a specific expansion schema;
- duplicate handoff entries with the same canonical seed key and origin must be
  rejected or represented as deterministic diagnostics before `VcId`
  assignment.

This resolves the task-0 drift: "exactly once" means every seed is accounted for
exactly once, while concrete VC cardinality is explicit and auditable.

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
rejection, seed-to-VC mapping, and `VcId` assignment.

No active `.miz` fixture is added by task 2 because the `proof_verification`
runner and source-derived payload seams remain external gaps.

## Public Enum Policy

Task 17 owns the final public enum forward-compatibility audit. Until then,
implementation tasks must default public `vc_ir` enums to downstream
forward-compatible `#[non_exhaustive]` surfaces unless a later spec explicitly
documents an exhaustive exception.
