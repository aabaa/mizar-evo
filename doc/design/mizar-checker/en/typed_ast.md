# mizar-checker: TypedAst

> Canonical language: English. Japanese companion:
> [../ja/typed_ast.md](../ja/typed_ast.md).

## Purpose

`TypedAst` is the checker-owned, source-shaped semantic snapshot produced by
type checking before registration closure and final overload resolution finish.
It refines:

- [architecture 01](../../architecture/en/01.ir_layers.md) `TypedAst`
  ownership;
- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  phase 6 and the `Typed AST` interface;
- checker [todo.md](./todo.md) task 2.

This document specifies the logical data shape used by checker implementation
tasks. Task 3 records the physical arena representation decision and implements
these structures without adding type inference, registration firing, overload
selection, language semantics, or proof behavior.

## Boundary

`TypedAst` owns:

- the typed node arena for one resolved source module;
- source-shaped links back to resolver nodes and checker recovery state;
- an optional immutable source-item/declaration/`BindingEnv` handoff produced
  from syntax-free resolver-shell projections;
- immutable local type context snapshots needed to interpret typed sites;
- `TypeTable` entries for expressions, formulas, declarations, and binding
  sites that receive checker type information;
- `TypeFactTable` entries for declared, assumed, inferred, built-in, and
  obligation-backed type facts;
- `CoercionTable` entries for widening, narrowing, and source-written `qua`
  coercion candidates;
- checker-local `InitialObligation`s identified by `InitialObligationId`;
- deterministic diagnostics and debug rendering for the typed snapshot.

`TypedAst` does not own:

- name lookup, label lookup, import/export validation, or resolver symbol
  allocation;
- final ordinary overload root selection, active refinement joining, or
  inserted overload-disambiguating `qua` views;
- registration activation, cluster closure, reduction normalization, or the
  canonical `ResolutionTrace` schema;
- `VcId`, `ObligationAnchor`, VC generation, ATP search, proof acceptance, or
  kernel replay;
- stable artifact schema publication or cache storage.

`InitialObligationId` is the only obligation identity allowed in this layer.
The checker must never assign, store, or derive a `VcId` while constructing
`TypedAst`.

## Top-Level Shape

The logical top-level shape is:

```rust
struct TypedAst {
    source_id: SourceId,
    module_id: ModuleId,
    resolved_root: Option<ResolvedNodeId>,
    source_context: Option<SourceBindingContextHandoff>,
    source_type: Option<SourceTypeApplicationHandoff>,
    source_attribute: Option<SourceAttributeHandoff>,
    nodes: TypedNodeArena,
    root: Option<TypedNodeId>,
    contexts: LocalTypeContextTable,
    types: TypeTable,
    facts: TypeFactTable,
    coercions: CoercionTable,
    initial_obligations: InitialObligationTable,
    diagnostics: TypeDiagnosticTable,
}
```

`source_id` and `module_id` come from `ResolvedAst`; they are stored for
source-map and module-boundary checks, not as proof or artifact identities.
`resolved_root` and `root` may be absent when prerequisite resolution failed
before a source-shaped checker shell can be built. When recoverable resolver
or type errors leave enough source shape, the checker should still allocate
recovered typed shells instead of dropping subtrees silently.

All ids inside `TypedAst` are local to the typed snapshot. They must be
deterministic for equivalent `ResolvedAst`, `SymbolEnv`, dependency summaries,
and checker configuration, but they are not stable public artifact identities
and are not proof-reuse identities.

Task 248 adds `source_context` as the sole owner of the complete source-item and
binding-context handoff. It is installed only together with its matching
`LocalTypeContextTable`; source/module, typed-site, binding, and context links
are validated transactionally. A recovered-empty producer result is incomplete
and cannot be installed. When the field is absent, deterministic debug output
remains byte-identical to the pre-Task-248 format.

Task 249 adds `source_type` as the sole owner of the validated flat
type-head/application/argument handoff. Installation revalidates every
expression, head, term, and `qua` typed site against the attached arena,
including same-source range containment and exact recovery. The producer has
already authenticated binding and symbol environments; `TypedAst` cannot
replace or reconstruct them. When this field is absent, conditional rendering
preserves the existing debug bytes.

Task 250 adds `source_attribute` as the sole owner of the validated raw
chain/attribute/qualifier/argument-group/actual handoff. Installation
revalidates source/module identity, its exact Task-249 expression association,
and every attached arena site's range and recovery state. Dense parent/order
links, polarity, qualifier and symbol provenance, punctuation, actual
kinds/origins, and compositional spelling consistency are producer-time
invariants preserved by the immutable handoff; installation does not
reconstruct or re-authenticate them. No partial or recovered bundle is
installed. When this field is absent, conditional rendering preserves the
existing debug bytes.

## Node Arena

`TypedNodeArena` stores source-shaped `TypedNode`s with stable local
`TypedNodeId`s.

Required node data:

- a source-shaped kind corresponding to the originating resolved node shape;
- source range or generated/recovered anchor;
- zero or more child `TypedNodeId`s in source order;
- a required link to the originating `ResolvedNodeId` when the node came from
  resolver syntax;
- optional table keys for node-local type, fact, coercion, diagnostic, and
  initial-obligation entries;
- a `TypingState` that distinguishes successful, assumed, unknown, error, and
  skipped typing;
- recovery metadata when the typed node is a degraded shell.

Arena invariants:

- every child id refers to a node allocated in the same arena;
- parent/child edges are acyclic;
- child order is deterministic and source-shaped;
- repeated checking of equivalent inputs produces the same ids and ordering;
- unsupported but recoverable source shapes are represented as degraded typed
  shells when `ResolvedAst` preserved enough shape;
- arena ids must not be used as `VcId`s, `ObligationAnchor`s, artifact ids, or
  cross-edit proof-reuse identities.

Task 3 decision: `TypedAst` uses a homogeneous arena of `TypedNode` records
whose source-shaped role is carried by `TypedNodeKind`. The arena allocates
dense local `TypedNodeId`s in insertion order and validates child links plus
acyclicity before a `TypedAst` is accepted. This mirrors the current
`mizar-syntax` compatibility view and `mizar-resolve` arena style, where a
shared id abstraction owns source-shaped traversal and node-specific meaning is
kept in node kind payloads or side tables.

`TypedNodeKind` is a checker-local source-shape projection. Task 3 must not add
a direct `mizar-syntax` dependency merely to store parser node kinds. When a
typed node mirrors a resolved source node, it records a stable checker-local
kind name plus the originating `ResolvedNodeId`; later tasks may add a
resolver-provided projection if `mizar-resolve` exposes one. Unsupported or
generated checker shells use explicit checker-local kind names instead of raw
parser vocabulary.

Typed node structs remain a possible future refactor only if a later task shows
that they remove concrete complexity without changing id stability,
side-table ownership, or deterministic debug rendering.

## LocalTypeContextTable

`LocalTypeContextTable` stores immutable snapshots of checker-local context
visibility for typed sites. It reconciles architecture 01's statement that
`TypedAst` owns local type context with the task split in which
`binding_env.md` specifies context construction.

```rust
struct LocalTypeContext {
    id: LocalTypeContextId,
    owner: TypedSiteRef,
    parent: Option<LocalTypeContextId>,
    layer: TypeContextLayer,
    bindings: Vec<BindingTypeRef>,
    introduced_assumptions: Vec<TypeFactId>,
    visible_facts: Vec<TypeFactId>,
    recovery: ContextRecoveryState,
}
```

Required invariants:

- context entries are immutable snapshots, not the mutable checker
  `TypeContext`;
- parent links form an acyclic layer chain;
- bindings refer to resolver-owned symbols or typed binding sites without
  redoing name lookup;
- `introduced_assumptions` records the `FactStatus::Assumed` facts introduced
  by this context layer;
- visible fact lists are sorted deterministically and may include only facts
  whose status is consumable in that context;
- an `Assumed` fact is consumable only when it appears in the current context's
  `introduced_assumptions` or in an ancestor context that remains visible;
- recovered contexts are explicit so later phases can avoid treating degraded
  assumptions as verified evidence.

The detailed lookup, layer-building, and binder-identity rules are specified by
`binding_env.md` in tasks 4 and 5. Task 2 reserves the storage shape only.

## TypeTable

`TypeTable` is the canonical checker table for type information attached to
typed sites.

```rust
struct TypeEntry {
    id: TypeEntryId,
    owner: TypedSiteRef,
    expected: Option<NormalizedTypeId>,
    actual: TypeEntryActual,
    status: TypeStatus,
    provenance: TypeProvenance,
}

enum TypeStatus {
    Known,
    Assumed,
    Unknown,
    Error,
    Skipped,
}
```

Task 3 exposes `TypeStatus::is_available_for_handoff()` only as a status
predicate: `Known` and `Assumed` may be forwarded with their provenance, while
`Unknown`, `Error`, and `Skipped` remain explicit partial-typing records.

`TypedSiteRef` is a source-local reference to a typed node or a stable
sub-node role, such as a binding site, expression result, formula result, type
expression, or candidate argument. It must not point back to raw surface
syntax. Resolver-owned ids may be referenced only through the owning typed
node's resolver link.
Typed site order sorts by the owning `TypedNodeId`, then whole-node entries
before role entries, then the stable role key.

`TypeEntryActual` records the normalized type known for the site, a candidate
set whose final overload root remains open, or the absence of a type after an
error. A table entry with `Error`, `Unknown`, or `Skipped` status is explicit
state, not a fabricated successful type. A handoff-available `Known` or
`Assumed` entry must carry either a known normalized type or a candidate set;
`Absent` is reserved for partial, error, or skipped typing state. Recovery
provenance must reference an existing `TypeDiagnosticId`.

Required invariants:

- each typed site has at most one primary `TypeEntry`;
- auxiliary expected-type constraints are linked from the primary entry rather
  than stored in source traversal order only;
- normalized type ids are allocated deterministically from canonical type keys;
- unresolved overload candidates may be filtered for impossible arity, kind, or
  mandatory type constraints, but final root selection is not represented as
  complete in `TypedAst`;
- query and debug rendering order is by typed site order, then table id.

## TypeFactTable

`TypeFactTable` stores the facts that phase 6 and later registration/overload
work consume.

```rust
struct TypeFact {
    id: TypeFactId,
    subject: TypedSubjectRef,
    predicate: TypePredicateRef,
    polarity: Polarity,
    provenance: FactProvenance,
    status: FactStatus,
}

enum FactProvenance {
    Declared(SourceRange),
    Assumed(TypeAssumptionId),
    Inferred(TypeRuleId),
    Obligation(InitialObligationId),
    Builtin(BuiltinRuleId),
    Registration(ResolutionStepId),
}

enum FactStatus {
    Known,
    Assumed,
    PendingObligation,
    Degraded,
    Rejected,
}
```

`Registration` provenance is reserved for the enriched fact table produced
after registration closure. Phase 6 may define the variant so table shape is
shared, but it must not invent cluster-derived facts before phase 7 records the
corresponding `ResolutionTrace` step.

`FactStatus` controls consumption:

- `Known` facts may be consumed as active checker evidence;
- `Assumed` facts may be consumed only in the local context that introduced the
  assumption and must remain marked as assumptions;
- `PendingObligation` facts explain a claim whose proof handoff is represented
  by `InitialObligationId`, but they are not verified evidence;
- `Degraded` facts are diagnostic or recovery metadata only;
- `Rejected` facts are retained only to explain diagnostics and cannot be
  consumed or exported.

Task 3 exposes `FactStatus::is_unconditionally_consumable()` for the `Known`
case only. Assumed facts still require local-context introduction before they
can be visible.

Required invariants:

- facts are deduplicated by canonical subject, predicate, polarity, and
  provenance key;
- `Obligation` provenance must reference an existing `InitialObligationId`;
- contradictory facts are recorded through diagnostics and status rather than
  resolved by hash or traversal accidents;
- invalid facts derived from errored nodes may remain as local degraded
  metadata, but they must not be exported as verified metadata or consumed as
  active evidence;
- facts produced under recoverable assumptions are distinguishable from fully
  known facts;
- deterministic queries sort by canonical fact key and then `TypeFactId`.

## CoercionTable

`CoercionTable` records checker-discovered coercion candidates. It does not
mean that a final implicit view has been inserted into `ResolvedTypedAst`.

```rust
struct CoercionEntry {
    id: CoercionId,
    site: TypedSiteRef,
    from: Option<NormalizedTypeId>,
    to: NormalizedTypeId,
    kind: CoercionKind,
    status: CoercionStatus,
    supporting_facts: Vec<TypeFactId>,
    obligation: Option<InitialObligationId>,
    provenance: CoercionProvenance,
}

enum CoercionKind {
    Widening,
    Narrowing,
    SourceQua,
}

enum CoercionStatus {
    Candidate,
    RequiresObligation,
    Blocked,
    Rejected,
}

enum CoercionProvenance {
    WideningRule(TypeRuleId),
    NarrowingClaim(SourceRange),
    SourceQua(SourceRange),
    Recovery(TypeDiagnosticId),
}
```

Task 3 exposes `CoercionStatus::is_available_for_handoff()` so later phases
can distinguish `Candidate` and `RequiresObligation` entries from `Blocked` and
`Rejected` entries without inferring that from renderer text. Recovery
provenance must reference an existing `TypeDiagnosticId`.

Required behavior:

- widening candidates must be proof-free semantic views justified by recorded
  type facts stored in `supporting_facts`;
- narrowing candidates require an `InitialObligationId` unless task-10
  known-fact support or a later spec proves they are locally discharged without
  VC generation;
- `Candidate` entries are available to later phases subject to the status of
  their referenced facts, types, and provenance;
- `RequiresObligation` entries carry an `InitialObligationId` and are not
  verified coercions;
- `Blocked` and `Rejected` entries are diagnostic/recovery records only;
- source-written `qua` expressions are preserved as source views and may
  contribute candidate constraints, but task 2 does not specify overload-root
  disambiguation;
- final overload-driven inserted `qua` views belong to `ResolvedTypedAst`, not
  to `TypedAst`;
- candidate ordering is deterministic by site order, kind, target type, and
  provenance. When provenance keys tie, `supporting_facts` order breaks the
  tie. If those keys are also identical, source type and `CoercionId` are used
  only as deterministic final tie-breakers.

## InitialObligation

`InitialObligationTable` stores checker-local obligations created before VC
generation.

```rust
struct InitialObligation {
    id: InitialObligationId,
    kind: InitialObligationKind,
    owner: TypedSiteRef,
    source_range: SourceRange,
    assumptions: Vec<TypeFactId>,
    goal: InitialObligationGoal,
    provenance: InitialObligationProvenance,
    status: InitialObligationStatus,
}

enum InitialObligationStatus {
    Pending,
    Blocked,
    Invalidated,
}
```

Task 3 exposes `InitialObligationStatus::is_available_for_handoff()` for
`Pending` obligations only. `Blocked` and `Invalidated` obligations remain
diagnostic state until the owning later task changes them.

Required obligation kinds include:

- sethood obligations for type expressions and constructs that introduce
  witnesses;
- non-emptiness obligations for choice terms such as `the T`;
- narrowing obligations for `reconsider` and invalid or non-trivial narrowing
  claims;
- registration correctness obligations once registration validation tasks
  refine the table.

Required invariants:

- `InitialObligationId` is deterministic within the `TypedAst` snapshot;
- ids are allocated in source order with a deterministic tie-breaker for
  multiple obligations at the same site;
- the table stores enough assumptions and source provenance for later
  conversion to VC generation inputs;
- `Pending` obligations are ready for later proof-owned VC generation;
- `Blocked` obligations are kept for diagnostics when prerequisite type or
  resolver data is degraded;
- `Invalidated` obligations cannot be handed off and are retained only to
  explain local errors;
- no field stores `VcId`, `ObligationAnchor`, prover result, proof witness, or
  accepted verifier status;
- later VC generation maps initial obligations to `VcId`s exactly at the
  proof-owned boundary.

## TypeDiagnosticTable

`TypeDiagnosticTable` stores checker-local diagnostic records for type data
shapes and recovery. It does not allocate public diagnostic codes while the
dedicated diagnostic code-space remains an external planning gate.

```rust
struct TypeDiagnostic {
    id: TypeDiagnosticId,
    owner: Option<TypedSiteRef>,
    source_range: SourceRange,
    class: TypeDiagnosticClass,
    severity: TypeDiagnosticSeverity,
    message_key: String,
    recovery: DiagnosticRecoveryState,
}
```

Required invariants:

- `TypeDiagnosticId` is local to the `TypedAst` snapshot;
- `message_key` is a stable crate-internal key, not a public diagnostic code;
- diagnostics sort by source range, class, message key, then id;
- diagnostic records may explain degraded types, facts, coercions, contexts,
  and initial obligations, but they are not proof evidence;
- no diagnostic field stores verifier status, proof witness, or `VcId`.

## Partial Typing After Errors

Type checking should continue after recoverable resolver or type errors when
enough source shape remains.

Recovery contract:

- unresolved names, ambiguous names, failed type expressions, impossible
  overload candidates, and invalid coercions produce explicit degraded table
  entries;
- `Known` entries are never fabricated to keep later phases running;
- `Assumed` entries must record the assumption that made recovery possible;
- `Unknown`, `Error`, and `Skipped` entries are visible to registration,
  overload, diagnostics, and debug rendering;
- facts and coercions attached to degraded sites must carry degraded status or
  diagnostics so they cannot be consumed as verified evidence;
- diagnostics are emitted in deterministic source order with stable secondary
  keys.

Later phases must check status before consuming a type, fact, or coercion.
Registration resolution may not fire registrations from invalid facts.
Overload resolution may preserve failed sites, but it must not elaborate them
as successful core terms.

## Deterministic Debug Rendering

Task 3 must provide `TypedAst::debug_text()` as a deterministic debug rendering
with the exact `typed-ast-debug-v1` header. The rendering contract is:

- render top-level ids, arena nodes, type entries, facts, coercions, initial
  obligations, and diagnostics in stable order;
- render source references as source-local ranges or resolver/typed ids, not
  memory addresses or host paths;
- render maps and sets in canonical key order;
- include degraded statuses explicitly;
- never rely on hash-map iteration order or allocation addresses.

The debug format is a test and review aid, not a stable public artifact
schema.

## Public Enum Policy

Task 31 applies the frontend task-25 public-enum decision procedure to this
module. All public checker-owned enums in `typed_ast` are forward-compatible API
surfaces and must remain `#[non_exhaustive]`; downstream consumers must keep
wildcard or fallback arms. Checker-internal matches may remain exhaustive over
the currently represented variants when implementing the specified behavior.

| enum | decision |
|---|---|
| `TypingState` | Forward-compatible; phase-6 node typing states may grow as recovery and handoff states are refined. |
| `NodeRecoveryState` | Forward-compatible; node recovery categories may grow with parser/checker recovery integration. |
| `TypedArenaError` | Forward-compatible; arena validation failures may add new structural checks. |
| `TypedSiteRef` | Forward-compatible; typed-site ownership may gain additional checker-owned roles. |
| `TypeContextLayer` | Forward-compatible; local context layers may grow as statement/proof extraction lands. |
| `BindingTypeRef` | Forward-compatible; binding type references may gain additional checker-owned anchors. |
| `ContextRecoveryState` | Forward-compatible; context recovery categories may grow with richer partial checking. |
| `TypeStatus` | Forward-compatible; type availability states may grow as downstream handoff policy is refined. |
| `TypeEntryActual` | Forward-compatible; type-entry actual payloads may grow with later checker phases. |
| `TypeProvenance` | Forward-compatible; type provenance may gain additional checker-owned evidence classes. |
| `Polarity` | Forward-compatible; predicate polarity may grow if the checker records richer logical qualifiers. |
| `FactProvenance` | Forward-compatible; fact provenance may grow with proof, registration, and artifact inputs. |
| `FactStatus` | Forward-compatible; fact consumption states may grow as obligation and artifact flows mature. |
| `CoercionKind` | Forward-compatible; coercion categories may grow with source and inserted-view handling. |
| `CoercionStatus` | Forward-compatible; coercion state may grow as proof/artifact validation is connected. |
| `CoercionProvenance` | Forward-compatible; coercion provenance may gain additional evidence sources. |
| `InitialObligationKind` | Forward-compatible; initial obligation categories may grow with VC and proof integration. |
| `InitialObligationStatus` | Forward-compatible; obligation status may grow when proof/artifact handoff is connected. |
| `TypeDiagnosticClass` | Forward-compatible; diagnostic classes may grow before public checker diagnostic codes are allocated. |
| `TypeDiagnosticSeverity` | Forward-compatible; diagnostic severity policy may grow with IDE/artifact consumers. |
| `DiagnosticRecoveryState` | Forward-compatible; diagnostic recovery states may grow with partial-checking policy. |
| `TypedAstError` | Forward-compatible; top-level typed-AST validation failures may gain new variants. |

No exhaustive public enum exceptions are owned by this module.

## Planned Tests For Task 3

Task 3 must add Rust tests that cover:

- deterministic `TypedNodeId`, `TypeEntryId`, `TypeFactId`, `CoercionId`, and
  `InitialObligationId` allocation for equivalent inputs;
- table insertion and query round-trips;
- local context snapshot insertion and query, deterministic context ordering,
  parent-chain validity, visible-fact filtering by consumable status, and
  recovered-context marking;
- fact deduplication and deterministic query ordering;
- status consumption rules for `Known` and `Assumed` type entries, consumable
  versus pending/degraded/rejected facts, blocked/rejected coercions, and
  blocked/invalidated obligations that must not be handed off;
- coercion candidate ordering and obligation links;
- partial typing entries for `Unknown`, `Error`, and `Skipped` statuses;
- boundary guards that no `TypedAst` data shape stores `VcId`,
  `ObligationAnchor`, proof witness, prover result, or accepted verifier
  status;
- boundary guards that final overload roots, active refinements, and inserted
  overload-disambiguating `qua` views are absent from `TypedAst`;
- deterministic debug rendering.

No `.miz` checker-stage fixtures are required by task 2 because no executable
checker semantics exist yet. Task 12 owns the first active `type_elaboration`
corpus runner and traceability entries.

Current source-derived runner note: the `mizar-test` type-elaboration runner may
construct explicit checker-owned `TypedAst` nodes for the bounded reserve-only
bare-builtin declaration pass bridge. Each reserve binding gets a declaration
node and a binding-specific type-expression node; multiple bindings may share
the same source type range while still using distinct `TypedSiteRef` owners.
Same-module attributed builtin and local-mode reserve heads are active fail
slices only; the active runner may use the same checker-owned assembly helper
to collect stable diagnostic keys, but those slices are not credited as
successful `TypedAst` readiness payloads. This keeps `TypedAst` a checker-owned
payload surface and does not authorize raw syntax walking, general declaration
extraction, CoreIr, ControlFlowIr, VC payloads, or proof evidence in
`mizar-checker`.

## Task 2 Classification

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | None found for the `TypedAst` data-shape boundary. Architecture 01 and 04 provide enough authority for this docs-only task. | Continue to task 3 after this spec is reviewed and committed. |
| `test_gap` | Checker semantic fixture directories and the `type_elaboration` runner are still absent. Task 3 also needs explicit boundary guards for proof-owned ids and final overload/view fields. | Task 3 adds Rust data-shape and boundary tests; task 12 adds active corpus coverage. |
| `design_drift` | Architecture 01 says `TypedAst` owns local type context while `todo.md` assigns context construction to `binding_env.md`; architecture 01 also names the coercion side table `CoercionTable`, while architecture 04's example uses `CoercionCandidateTable`. | This spec resolves the context split by reserving `LocalTypeContextTable` storage while deferring construction rules to tasks 4-5. It standardizes the checker module name as `CoercionTable` and states that it stores candidate entries only. No architecture rename is performed in task 2. |
| `source_drift` | None. Task 1 introduced only crate scaffolding and no checker semantic source. | No source repair is needed for task 2. |
| `external_dependency_gap` | None blocking task 2. Later tasks still depend on resolver payloads, diagnostic code ownership, artifact summaries, and proof acceptance inputs. | Re-evaluate in the owning implementation tasks; do not fabricate missing external data. |
| `deferred` | Resolved by task 3 for the typed arena: use a homogeneous `TypedNodeKind` arena with dense local ids. Later semantic tasks still own their external dependency gates. | Keep any future representation refactor behavior-preserving and task-scoped. |

## Task 3 Classification

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | None blocking the data-shape implementation after task 3 adds the checker-local node-kind projection, diagnostic table shape, and context assumption links. | Implement only the documented data shapes and deterministic rendering. |
| `test_gap` | Task 2 documented the missing Rust coverage for ids, tables, contexts, statuses, proof-boundary guards, final-overload-field absence, and rendering. | Resolved by task 3 Rust unit tests. `.miz` semantic fixtures remain task 12. |
| `design_drift` | The task-1 lint guard described the crate as exposing no public semantic API, and the TODO decision described the arena representation as open before this task. | Resolved by task 3: the guard allows only the documented `typed_ast` API and the TODO decision text records the arena decision. |
| `source_drift` | Before task 3, source had no `typed_ast` module while task 2 specified it. | Resolved by task 3: `src/typed_ast.rs` is added and only this documented module is exposed from `lib.rs`. |
| `external_dependency_gap` | Public checker diagnostic code ownership remains absent; resolver may later expose a richer source-kind projection. Neither blocks task 3. | Keep diagnostics crate-internal with stable `message_key`s. Do not add a direct `mizar-syntax` dependency for node-kind storage. |
| `deferred` | No physical typed-arena deferral remains after the task-3 decision. Type inference, binding construction, registration firing, overload resolution, public diagnostics, artifacts, and proof acceptance remain owned by later tasks. | Keep task 3 data-only. |

## Task 251 Ownership Addendum

`TypedAst` now owns an optional immutable `SourceEvidenceHandoff`.
`with_source_evidence` rejects replacement and authenticates the handoff
source/module plus referenced facts against the existing typed payload before
installation. The addition is syntax-free and does not add evidence truth,
accepted facts, proof status, or downstream IR to the typed arena.

## Task 252 Ownership Addendum

`TypedAst` now owns an optional immutable `SourcePrimaryTermHandoff`.
`with_source_term` rejects replacement and authenticates the handoff
source/module plus every referenced arena node before installation. The
addition is syntax-free and does not add a normalized type, semantic term or
formula, accepted fact, proof status, or downstream IR to the typed arena.

## Task 253 Ownership Addendum

`TypedAst` now owns an optional immutable `SourceFunctorApplicationHandoff`.
`with_source_application` is one-shot, requires the Task-252 handoff already
installed, compares its exact deterministic debug fingerprint, and
revalidates every referenced primary root before installation. When a
Task-254 handoff is already present it also revalidates that handoff against
the new Task-253 ownership graph before committing the field, so install order
cannot introduce a shared primary, reverse containment, partial overlap, or
an unowned contained application. An equivalent Task-252 clone is accepted;
replacement and non-equivalent same-source/module substitution fail
atomically. This adds no signature, result type, candidate selection,
definition behavior, semantic term/formula, fact, proof, or downstream IR.

## Task 254 Ownership Addendum

`TypedAst` now owns an optional immutable `SourceStructureHandoff`.
`with_source_structure` is one-shot, requires Task 252 and any targeted
Task-253 dependency already installed, compares the exact deterministic
fingerprints, and revalidates every Task-252/253/254 target plus all
term/member/FieldUpdate/wrapper arena sites and cross-family ownership before
installation, preserving the producer-validated direct written partitions.
An unrelated installed Task-253 handoff may coexist when
the Task-254 application fingerprint is absent only if its targets and ranges
are disjoint from Task 254. Replacement, wrong-key input, non-root or reverse
Task-253 ownership, shared Task-253 argument primaries, and non-equivalent
dependency substitution fail atomically. This adds no structure signature,
member/view identity, result type, semantic constructor/selector/update,
fact, proof, or downstream IR.

## Task 255 Ownership Addendum

`TypedAst` now owns an optional immutable `SourceSetTermHandoff`.
`with_source_set_term` is one-shot, requires Task 252 and every targeted
Task-253/254 dependency already installed, compares exact deterministic
fingerprints, and revalidates all Task-252/253/254/255 targets, arena sites,
canonical spellings, and nearest-family ownership before installation.
`with_source_application` and `with_source_structure` revalidate an already
installed Task-255 handoff before committing their fields, so either install
order preserves the same partition. Unrelated optional handoffs may coexist
with absent fingerprints only when their occurrences are range-disjoint.
Replacement, missing dependency, non-root/reverse ownership, overlap, and
non-equivalent dependency substitution fail atomically. This adds no
comprehension binding/capture, formula, sethood/nonemptiness/widening result,
semantic term/type, fact, proof, or downstream IR.

## Task 256 Ownership Addendum

`TypedAst` now owns an optional immutable `SourceAtomicFormulaHandoff`.
`with_source_atomic_formula` is one-shot, requires Task 252 and every targeted
Task-253/254/255 handoff already installed, compares exact deterministic
fingerprints, and revalidates formula sites, provenance, requests, and the
nearest-family target partition. Later Task-253/254/255 installers revalidate
an already installed Task-256 handoff before committing their fields, so
installation order cannot bypass an ownership or fingerprint check.
Replacement, missing or non-equivalent dependencies, non-root targets,
overlap, and arena/provenance drift fail atomically. This adds no candidate
selection, expected-type answer, assertion fact or truth, formula result,
theorem acceptance, proof, or downstream IR.

## Task 257A Ownership Addendum

`TypedAst` now owns one optional immutable
`SourceCompositeFormulaHandoff`. `with_source_composite_formula` is one-shot,
rejects coexistence with the Task-248 source-context handoff, and revalidates
source/module identity, the complete typed arena, the exact extended
`BindingEnv`, and all seven dense tables before publication. The borrowed
getter exposes the syntax-free transport. Legacy ASTs retain their exact debug
bytes when the field is absent. The handoff contains unresolved source intent
only and creates no formula truth, type answer, fact, theorem owner, proof, or
acceptance.

## Task 257B1 Ownership Addendum

`TypedAst` now also owns an optional immutable
`SourceFormulaCompositionHandoff`. The combined
`with_source_formula_composition` installer requires preinstalled Task-252
primary terms and Task-256 atomic formulas, then validates and publishes the
second composite profile and composition together. It rejects Task-248
source-context coexistence, an already installed Task-257A profile, all
dependency drift, and partial publication. The legacy composite installer
continues to reject the second profile.

Task 257B2 extends the same combined installer to the exact third composite
profile plus `8/0` composition. It requires the exact installed Task-252/256
dependencies and rejects missing/stale fingerprints, Task-248 coexistence,
existing A/B1/B2 ownership, and partial publication. The legacy composite-only
installer remains Task-257A-only.

## Task 257B3 Frozen Ownership Addendum

The combined installer now admits only the exact fourth composite profile and
`3/6` composition over the Task-48-derived one-reserve base plus exact
Task-252/256 dependencies. `source_context()` remains absent because the
Task-248 reserve-plus-definition profile is not this consumer. Installation
authenticates reserve-default provenance, nested contexts, shadowing, owning
atomic edges, and lookup replay, and rejects existing A/B1/B2/B3 ownership or
partial publication. The legacy composite-only installer remains A-only.

The B3 combined installation and duplicate/collision rollback paths are now
executable; the Task-248 exclusion and legacy installer remain unchanged.

## Task 257C1 Frozen Ownership Addendum

The caller/pipeline installs Task 252 first. It then invokes
`TypedAst::with_source_atomic_formula`, which atomically validates and
publishes only the extended Task-256 handoff. The exact chain transaction must
authenticate `3/0/3` and `1/0/2/2/2/0/0/3/2`, including one shared
boundary edge, two imported candidates, and both candidate requests.
Missing/stale dependency fingerprints, old-family collision, partial
publication, or segment corruption fail closed. Existing Task-256 and
Task-257A/B1/B2/B3 installers and bytes remain exclusive and unchanged.

Task 257C1 is implemented through the existing one-shot
`with_source_atomic_formula` path. Successful installation and subsequent
clone revalidation preserve all nine tables; every tested partial or
cross-profile mutation remains atomic and fail-closed.

## Task 255C1 Frozen Ownership Addendum

The existing one-shot `with_source_set_term` path will admit the exact
seven-table Task-255C1 profile only after the complete Task-252 `4/0/4` and
Task-253 `1/0/1/2/2` dependencies are installed. It revalidates the colon
and direct condition-wrapper anchors, condition-contained lower-family
exclusion, and both fingerprints. Condition operands stay in the immutable
Task-252 handoff without a Task-255 edge. Failure publishes no condition row
and preserves all previous fields and debug bytes.

## Task 255C1 Installation Result

`with_source_set_term` now installs the authenticated seven-table handoff
after Task 252 and Task 253. Failed condition or dependency revalidation
publishes nothing; a subsequent valid install succeeds from the unchanged
base object. Legacy condition-empty objects remain byte-identical.

## Task 257C2 Frozen Ownership Addendum

`TypedAst` will expose one optional
`SourceConditionFormulaCompositionHandoff` after Task-252/253/255/256
installation through this exact surface:

```rust
pub const fn source_condition_formula_composition(
    &self,
) -> Option<&SourceConditionFormulaCompositionHandoff>;

pub fn with_source_condition_formula_composition(
    self,
    composition: SourceConditionFormulaCompositionHandoff,
) -> Result<Self, TypedAstError>;

// New #[non_exhaustive] TypedAstError variant:
InvalidSourceConditionFormulaComposition,
```

The installer reauthenticates the four lower fingerprints, direct
condition-wrapper/equality relation, exact operand ownership, and one
association row. It rejects missing or substituted dependencies, an existing
Task-257 composite/Task-257B composition, or a second
condition-composition handoff atomically through the dedicated error variant.
The existing Task-257A and combined Task-257B installer signatures and
successful legacy behavior remain unchanged, but both add a reciprocal
fail-closed check for an already installed C2 handoff through their existing
error variants. Tests cover A/B-before-C2 and C2-before-A/B with rollback.
At the frozen pre-Task-256C1 baseline, the C2 installer was not implementable
until the separate lower prerequisite made the authenticated Task-255
condition containment pass both set/atomic installation orders without
weakening unrelated overlap guards. Task 256C1 now passes both orders; the C2
installer is now implemented and passes both lower installation orders plus
all four reciprocal A/B/C2 exclusion orders with byte-identical rollback.
Absent-handoff debug bytes remain unchanged; no semantic table is populated.

## Task 256C1 Frozen Installation Revalidation

No `TypedAst` API or production implementation changes. Existing symmetric
revalidation is the contract: Task 255 then Task 256 validates the atomic
handoff against the installed set handoff, while Task 256 then Task 255
revalidates the installed atomic handoff against the incoming set handoff.
After the private Task-256 validator fix, both orders accept only the
authenticated equality-condition container and produce byte-identical
immutable handoffs and full debug output.

Every invalid overlap still fails atomically through the existing
`InvalidSourceAtomicFormula` or `InvalidSourceSetTerm` variant. No field is
published, and valid replay from the unchanged base succeeds. Final resolved
revalidation and clone ownership are unchanged.

## Task 256C1 Implementation Result

The existing symmetric installers now pass the exact authenticated
condition/equality relation in both orders. Substituted validation contexts
still fail through the existing order-specific error, publish no field, and
allow replay from the unchanged base. Equal full debug output confirms that
installation order adds no state. No `TypedAst` source or API changed.

## Task 257C3 Frozen Ownership

The later Task-257C3 implementation adds an optional
`SourcePredicateChainCompositionHandoff`, accessor, one-shot installer, debug
projection, and `InvalidSourcePredicateChainComposition`. It requires the
exact Task-252 and Task-256 handoffs and is reciprocally exclusive with
Task-257A, Task-257B, and Task-257C2 ownership in all installation orders.
Failure publishes nothing and preserves byte-identical replay. This
documentation prerequisite changes no `TypedAst` source or executable API.

```rust
pub const fn source_predicate_chain_composition(
    &self,
) -> Option<&SourcePredicateChainCompositionHandoff>;

pub fn with_source_predicate_chain_composition(
    self,
    composition: SourcePredicateChainCompositionHandoff,
) -> Result<Self, TypedAstError>;
```

C3-after-A/B/C2 fails with
`InvalidSourcePredicateChainComposition`. A/B/C2-after-C3 fails with,
respectively, `InvalidSourceCompositeFormula`,
`InvalidSourceFormulaComposition`, or
`InvalidSourceConditionFormulaComposition`. All six directional paths are
atomic and replayable. The optional C3 debug chunk is after Task-252
source-term, Task-256 source-atomic-formula, and the mutually exclusive
A/B/C2 slots, immediately before the node/table section.

## Task 257C3 Implementation Result

The optional field, accessor, one-shot installer, dedicated error, and debug
projection are implemented. The installer requires exact Task-252 and
Task-256 dependencies and rejects duplicate or A/B/C2 occupancy before
publication. Reciprocal test-only occupancy mutations exercise each of the
six directional guards with an otherwise valid attempted install, so lower
dependency mismatch cannot mask the ownership contract. Failure preserves
the base debug bytes and valid replay.

## Task 258A Frozen Source-Statement Ownership

The later Task-258A implementation adds one optional
`SourceStatementHandoff`, read-only accessor, one-shot installer, debug
projection, and `InvalidSourceStatement`. Exact Task-252 and Task-256
handoffs must already be installed; every other lower and Task-257 owner is
absent in the frozen `MT10-FS` smoke profile.

```rust
pub const fn source_statement(&self) -> Option<&SourceStatementHandoff>;

pub fn with_source_statement(
    self,
    statement: SourceStatementHandoff,
) -> Result<Self, TypedAstError>;
```

The installer revalidates source/module, both lower fingerprints, resolver-
authenticated owner data already frozen in the handoff, all five
`1/1/1/1/1` rows, arena sites/ranges, the handoff-owned exact `BindingEnv`
and its fingerprint, binding visibility, reference uses, and formula target
before publication. Duplicate, missing lower, stale, substituted binding, or
corrupt input fails atomically, preserves byte-identical state, and permits
valid replay.

Task 248 and Task 258A are exclusive. Production exposes only the
Task-248-constructor-first direction: `with_source_statement` after
`source_context` fails with `InvalidSourceStatement`. Task 248 has no
post-construction installer and this task adds none. The exact reverse test
oracle uses:

```rust
#[cfg(test)]
pub(crate) fn with_source_context_for_test(
    self,
    source_context: SourceBindingContextHandoff,
) -> Result<Self, TypedAstError>;
```

It invokes the same private validation and fails with
`InvalidSourceContext`. Each rejection preserves the first owner's exact
debug and supports valid replay. A separate
`inject_source_statement_for_test(&mut self, SourceStatementHandoff)` bypass
exists only to prepare the final-assembly coexistence rejection; it is not a
production construction path. The debug chunk follows every Task-257 owner
slot and precedes the node/table section. `facts` and all existing semantic
tables remain empty. This documentation commit changes no `TypedAst` source
or API.

### Task 258A Implementation Result

The optional handoff, read-only accessor, one-shot installer, debug chunk,
and dedicated `InvalidSourceStatement` path are implemented. Installation
requires an empty generic typed projection (`resolved_root`, contexts, types,
facts, coercions, initial obligations, and diagnostics) in addition to the
frozen source-family exclusions and Task-252/256 dependencies. Failed
coexistence does not mutate the prior value and valid replay remains
deterministic.

## Task 258B1 Frozen Combined Statement Ownership

Task 258B1 keeps the Task-258A field/accessor/installer unchanged and adds a
second optional field plus these exact public APIs:

```rust
pub const fn source_statement_references(
    &self,
) -> Option<&SourceStatementReferenceHandoff>;

pub fn with_source_statement_references(
    self,
    statements: SourceStatementHandoff,
    references: SourceStatementReferenceHandoff,
) -> Result<Self, TypedAstError>;
```

The existing `with_source_statement` remains Task-258A-only. The combined
installer alone admits the B1 `1/4/4/4/4 + 1/1` pair. It requires fresh
statement/reference slots, exact installed Task-252/256 handoffs, the
handoff-owned `3/1/0` environment, one arena, matching statement
fingerprint, retained 77-node/root-76 resolver AST with sole resolved
`Label(0)` node 68 and table/site parity, and replay-authenticated resolver
projection/reference/result. Any failure is
`TypedAstError::InvalidSourceStatement`; validation completes before either
field is published.

Task-248, every Task-257 owner, Task-258A, any generic typed semantic table,
a base without references, references without the matching base, duplicate
installation, and every opposite install order fail atomically. In debug,
the base statement chunk is followed immediately by the reference chunk
before nodes/tables. Task-258A has no second chunk and keeps identical bytes.
No checked formula, fact, statement semantic, proof, goal, diagnostic, or
accepted theorem is created. This prerequisite changes no `TypedAst` source.

### Task 258B1 Implementation Status

`TypedAst::with_source_statement_references` now validates and installs the
exact B1 base/reference pair atomically. The legacy statement installer still
accepts only Task 258A, all existing payload owners reject the pair, and
failed validation leaves the original value reusable. Accessors, cloning,
and debug ordering preserve the frozen contract without semantic output.

### Task 258B2 Frozen Typed Ownership

Task 258B2 reuses the base-only
`TypedAst::with_source_statement(SourceStatementHandoff)` path. That
installer may admit either the exact Task-258A profile or the exact Task-258B2
profile; Task 258B1 continues to require
`with_source_statement_references`. For B2 it must revalidate the frozen
113-byte source identity, Task-48 `2/1/0`, Task-252 `6/6/0`, Task-256
`3/0/0/0/0/0/0/6/6`, and statement profile `1/3/3/3/3`, including the sole
proof context and the theorem, assumption, and conclusion rows.

The B2 handoff contains no reference association. A reference handoff, a
Task-248 or Task-257 payload, duplicate installation, a mismatched source or
profile, or any semantic-stage input must fail atomically as
`TypedAstError::InvalidSourceStatement`. Successful installation owns only
the syntax-free source table and its lower-stage provenance. It creates no
fact, accepted premise, checked formula, statement semantic, proof, goal,
diagnostic, or theorem result. This prerequisite changes no `TypedAst`
source, tests, or existing debug bytes.

### Task 258B2 Implementation Closure

`TypedAst::with_source_statement` now admits only the exact Task-258A or
Task-258B2 base profile. Task-258B1 remains pair-only, and Task-248,
Task-257A/B/C2/C3, Task-258 cross-profile hybrids, occupied semantic tables,
and either foreign-first or statement-first ownership order fail without
partial mutation. Clone and debug order remain stable.

### Task 258B3 Frozen Paired Installation

`TypedAst` adds
`source_statement_witnesses: Option<SourceStatementWitnessHandoff>`,
`source_statement_witnesses()`, and
`with_source_statement_witnesses(statements, witnesses)`. The paired
installer requires installed Task-252/256 lower values, exact B3 base and
witness fingerprints, the shared 49-node arena, and empty references,
foreign source families, and semantic tables. It publishes both halves only
after all validation succeeds.

`with_source_statement` remains A/B2-only and
`with_source_statement_references` remains B1-only. A B3 base may exist as a
producer result but cannot install alone. Orphan, stale, cross-profile,
Task-248/257, B1-reference, and both-order ownership conflicts roll back
without mutation. Debug appends the stable witness chunk immediately after
the base chunk; prior profiles remain byte-identical.

### Task 258B3 Paired Installation Result

`source_statement_witnesses()` and
`with_source_statement_witnesses(statements, witnesses)` are implemented.
The base-only and reference-paired installers reject B3, while the B3
installer validates both halves before atomic publication. Cross-family and
both-order rollback tests pass; debug emits base then witness then nodes.

## Task 258B3N Planned Paired Ownership

The existing paired installer remains the only B3N publication path. It will
validate base, witness, dense name table, exact 51-node arena, and B3N
profile before atomic installation. Existing B3 remains valid with an empty
name table; every B3/B3N hybrid, repeated install, cross-family order, and
semantic coexistence must roll back without partial publication.

## Task 258B3N Implementation Result

The paired installer now accepts authenticated B3 or B3N bundles, including
the exact B3N dense name table and 51-node arena. Base-only, repeated,
B3/B3N hybrid, reference, Task-248/257, and semantic-table orders fail
atomically; successful debug ordering remains base, witness/name, then
nodes.

## Task 258B3M1 Planned Paired Ownership

The existing paired installer remains the only publication path. It will
accept only the exact B3M1 base plus `2 witnesses / 1 name`, verify the
56-node arena and both witness/name links, and publish both halves
atomically. B3/B3N bytes remain unchanged. Repeated installation,
cross-profile halves, reference/Task-248/257/other-258 families in either
order, and semantic coexistence roll back without partial ownership.

## Task 258B3M1 Implementation Result

The existing paired installer now recognizes only the exact authenticated
B3M1 base plus `2 witnesses / 1 name`. It revalidates the six-term lower
profile, 56-node arena, statement/primary fingerprints, dense ordinals, and
name link before publishing both halves. Every cross-family or repeated
order still returns `InvalidSourceStatement` without partial ownership.

## Task 258B3M2A Planned Paired Ownership

The existing paired installer remains the sole publication path. It may
accept only the exact B3M2A base plus `1 witness / 0 names`, authenticate
the 49-node arena, five Task-252 terms, four references, numeric request 0,
both equality exclusions, fingerprints, and `[0,1,2]` source order, then
publish both halves atomically. B3/B3N/B3M1 bytes remain unchanged.
Standalone/repeated installation, profile hybrids, reference or numeric
request corruption, Task-248/257/other-258 families in either order, and
semantic coexistence roll back without partial ownership.

## Task 258B3M2A Implementation Result

The existing paired installer now accepts the exact B3M2A base and
`1 witness / 0 names` transaction, then publishes both tables atomically.
All standalone, repeated, cross-profile, Task-248/257/other-258, corrupted
dependency, and reversed-order attempts still fail without partial ownership.
No public typed-AST method, field, or debug grammar changed.

## Task 258B3M2B1 Frozen Typed Ownership

The existing paired installer is sufficient for one exact B3M2B1 base and
`1 witness / 0 names` transaction. It must revalidate the 53-node arena,
five-root/six-primary mapping, parenthesized term 2 with child 3, and
child-only reference before atomic publication. It authenticates all six
Task-252 terms, five references, the outer/inner parent edge, both equality
exclusions, fingerprints, and source order `[0,1,2]`; witness 0 targets
outer term 2 while reference 2 targets inner term 3. Standalone, repeated,
prior-profile, B3M2A, Tasks-248/253–257/other-258, corrupted dependency,
reference/parent-corrupt, semantic-coexisting, and reverse-order attempts
must fail without partial ownership. No public method, field, enum, or debug
grammar is added.

## Task 258B3M2B1 Implementation Result

The existing paired installer now accepts the exact B3M2B1 base and
`1 witness / 0 names` transaction and publishes both tables atomically.
It revalidates the 53-node arena, five-root/six-primary map,
parenthesized-wrapper/child edge, all five references, both equality
exclusions, fingerprints, and `[0,1,2]` source order. Standalone, repeated,
cross-profile, corrupted-dependency, semantic-coexisting, and reversed-order
attempts still fail without partial ownership. No public typed-AST API or
debug grammar changed.

## Task 258B3M2B2A Frozen Typed Ownership

The future paired installer may accept only one authenticated B3M2B2A base
plus `1 witness / 0 names` transaction. It must revalidate the 57-node
arena, five-root/seven-primary mapping, parent chain `2 -> 3 -> 4`, five
references, Task-256 exclusion of all three witness-subtree terms,
fingerprints, and source order `[0,1,2]` before atomic publication.
Standalone, hybrid, repeated, stale, cross-family, reversed-order, and
semantic-coexisting states remain rejected. No public typed-AST API or
debug grammar changes in the prerequisite.

## Task 258B3M2B2A Implementation Result

The paired installer now accepts the exact B3M2B2A base/witness profile and
atomically revalidates all dependencies, both wrapper links, five
references, complete Task-256 subtree exclusion, fingerprints, and source
order `[0,1,2]`. Standalone, hybrid, repeated, stale, cross-family,
reversed-order, and semantic-coexisting states remain rejected. No public
typed-AST API or debug grammar changed.

## Task 258B3M2B2B1A Atomic Typed Ownership Result

`TypedAst::with_source_application_statement_witnesses` is the sole B1A
publication path. It accepts the exact authenticated Task-253 application,
Task-258 base statement, and one unnamed `Application(0)` witness as one
three-handoff transaction. The installer revalidates the complete 63-node
arena, Task-252 `6/4/2`, Task-253 `1/0/1/2/2`, Task-256 equality-only
exclusion, resolver-owner fingerprints, statement/witness source order, and
the witness-to-application fingerprint before publishing any table.

The pre-existing standalone Task-253 application remains valid. An
application-first state followed by a separate B1A statement-witness
installer, application plus only a statement or witness, statement-first,
witness-only, hybrid, stale, substituted, repeated, reverse-order,
Tasks-253/254/255 coexisting, and semantic-coexisting B1A publication attempts
all fail with the original `TypedAst` unchanged. Legacy application-free
statement profiles and debug bytes remain valid. The successful B1A
installation adds no type, expression-semantic, proof, or goal ownership.

## Task 258B3M2B2B1B1 Frozen Atomic Typed Ownership

The existing
`TypedAst::with_source_application_statement_witnesses` entry point must
enumerate B1A and B1B1 as two exact profiles. B1B1 is the 67-node wrapped
profile with Task-252 `6/4/2`, Task-253 `1/1/1/2/2`, Task-256 equality-only
edges `[0,1]` / `[4,5]`, base statement `1/2/2/2/2`, and one unnamed
`Application(0)` witness/no names. Wrapper 0 is authenticated Task-253
containment, never a witness target.

The installer revalidates complete source/module/arena identity, local
theorem and imported application provenance, both lower fingerprints,
base/witness rows, wrapper-to-application containment, and the
witness-to-application edge before publishing all three handoffs. B1A
remains the separate 63-node unwrapped profile with byte-identical API/debug
behavior; neither profile may be inferred by broadening the other.

Application plus only a statement or witness, an orphan statement/witness
pair, B1A/B1B1 hybrids, wrapper/application substitution, stale
fingerprints, partial/reverse/repeated installation, another Task-258 family,
Tasks-254/255 coexistence, or semantic coexistence rejects with the original
`TypedAst` unchanged. No new public typed-AST API, debug grammar, type,
semantic, proof, or goal owner is authorized.

## Task 258B3M2B2B1B1 Typed Installation Result

Typed installation now enumerates B1B1 beside, not inside, B1A and atomically
publishes the exact application/statement/witness bundle. The frozen partial,
hybrid, substituted, stale, reverse, repeated, and coexistence failures leave
the original AST unchanged. `typed_ast.rs` is 4,743 lines; no public installer
or semantic/type owner changed.

## Task 258B3M2B2B2A Frozen Atomic Installer

`with_source_structure_statement_witnesses` is the sole future atomic entry
point for this profile. It validates exact structure, statement, and witness
transactions against already installed Task-252/256 data, including
Task-256 revalidation with `Some(&structure)` while its direct Structure
target and structure fingerprint remain absent, before publishing all three
together. Existing `with_source_structure` continues to support its
already-authorized pre-statement structure/atomic coexistence; it and
`with_source_statement_witnesses` cannot partially assemble the exact B2A
statement bundle. Application installers reject structure targets and the
new installer rejects application/legacy targets. Every failure leaves the
original AST unchanged.

## Task 258B3M2B2B2A Atomic Installation Result

`with_source_structure_statement_witnesses` now atomically installs only the
authenticated B2A structure/statement/witness triple. It revalidates
Task-252/254/256 and all statement/witness fingerprints before mutation,
including Task 256 with `Some(&structure)` but no direct structure target or
fingerprint. Application, legacy, partial, repeated, stale, reverse, and
family-coexisting bundles reject with the original AST unchanged.

`typed_ast.rs` is 4,829 lines. Existing installers remain compatible; no
active route, semantic/type/proof owner, or coverage credit was added.

## Task 258B3M2B2B2B Frozen Atomic Sibling

The existing combined structure-statement installer must enumerate B2A and
B2B as two exact siblings; it must not admit a generic
`application = None`/`structure = Some` statement bundle. B2A is the
constructor-witness bundle already frozen above. B2B is the 79-node selector
bundle with Task-254 terms `0/1`, witness target `Structure(0)`, and selector
base `Structure(1)`. It revalidates the exact Task-252 roots, Task-254 rows,
Task-256 equality rows, Task-258 base rows, and all fingerprints before
publishing the structure, statement, and witness tables together.

Task 256 owns `BuiltinPredicateApplication` nodes `51/70`; nodes `52/71`
are unowned formula containers and cannot be substituted into the ownership
map. B2A/B2B row, target, ownership, or fingerprint hybrids reject with the
original AST unchanged. This prerequisite adds no public API, installer,
debug surface, active route, or semantic table.

## Task 258B3M2B2B2B Atomic Installation Result

`with_source_structure_statement_witnesses` now enumerates B2B beside B2A
and accepts only the exact authenticated 79-node selector bundle. Before
publication it revalidates Task-252/254/256, Task-258 base/witness rows, all
fingerprints, witness target `Structure(0)`, selector base `Structure(1)`,
Task-256 ownership at `51/70`, and unowned containers `52/71`.

Generic structure admission, B2A/B2B hybrids, stale or swapped
fingerprints/targets, application coexistence, and partial/reverse/repeated
installation leave the original AST unchanged. `typed_ast.rs` is 4,830
lines. No public installer, debug surface, semantic/type/proof/goal owner,
corpus active route, or coverage credit changed.

## Task 258B3M2B2B2C Frozen Atomic Sibling

`with_source_structure_statement_witnesses` remains the sole atomic entry
point. It must enumerate B2C beside B2A/B2B, selected by the full exact
source/arena/provenance/profile rather than the shared option shape. Before
publication it revalidates Task-252 `7/4/3`, Task-254
`2/0/1/3/1/4/9`, Task-256 equality operands `Primary(0/1)` and
`Primary(5/6)`, Task-258 base `1/2/2/2/2`, witness `1/0`, structure
fingerprint, and target `Structure(0)`.

Structure, statement, and witness handoffs publish together only after all
checks pass. B2A/B2B/B2C row, ownership, fingerprint, or target hybrids,
application coexistence, stale/reverse/partial/repeated installation, and
subtree-container substitution leave the original AST unchanged. No public
installer, debug schema, type/semantic/proof/goal owner, fixture, trace
credit, or active root dispatch is added. Implementation and atomicity tests
remain open.

## Task 258B3M2B2B2C Implemented Atomic Sibling

`with_source_structure_statement_witnesses` now admits the exact B2C profile
beside B2A/B2B and reuses the existing structure-aware validation and
publication path. The profile cannot be selected by the common option shape:
all source, arena, lower-table, ownership, fingerprint, statement, and witness
fields are revalidated first. Failure leaves the original TypedAst unchanged.

The frozen atomicity, hybrid/order, replay, and rollback matrices pass. No
public installer, field, schema, or semantic payload was added; final
source/documentation and quality reviews remain pending.

## Task 258B3M2B2B2C Broad Atomic-Install Verification

The broad format, Clippy, crate, and workspace gates, focused `4/4` and
`5/5`, and sibling `12/12` and `21/21` suites pass with unchanged counts and
hashes. Atomic publication and rollback remain exactly bounded to the
implemented private sibling, with no public or semantic expansion. Independent
final source/documentation and quality reviews, commit, and post-commit
inventory remain pending.

## Task 258B3M2B2B2C Final Atomic-Install Review Status

Independent final source/documentation consistency and final quality report
**NO FINDINGS**; all nine hard gates PASS with a valid `98/100`. Atomicity
evidence, counts, hashes, and public/semantic boundaries remain unchanged.
Only cached-diff/staging audit, implementation commit, and post-commit
inventory/fresh-next-task gates remain pending.

## Task 258B3M2B2B3A Frozen Typed-AST Installer

`TypedAst` adds exactly:

```rust
pub fn with_source_set_term_statement_witnesses(
    mut self,
    set_terms: SourceSetTermHandoff,
    statements: SourceStatementHandoff,
    witnesses: SourceStatementWitnessHandoff,
) -> Result<Self, TypedAstError>;
```

It atomically authenticates the set-term table, Task-258 base owners/
contexts, and one B3A witness before publication. The tuple is application
`None`, structure `None`, set `Some`. Legacy `None/None/None`, application
`Some/None/None`, and structure `None/Some/None` remain accepted only by
their existing installers; every other tuple and all hybrids, reorders,
duplicates, stale dependencies, partition violations, and invalid targets
fail closed with `TypedAstError::InvalidSourceStatement` and immediate replay.
The installer adds no error variant or display change; lower handoff
`SourceStatementWitnessError` values never escape this public layer.

The installer publishes one witness/zero names and only
witness-to-`SetTerm(0)`. Existing installers, family composition, literal
debug bytes, semantics, and routes stay unchanged. Final-clone revalidation
belongs to `ResolvedTypedAst`; no set-term producer edit is permitted.

## Task 258B3M2B2B3A Implemented Typed Installation Closure

`with_source_set_term_statement_witnesses` now validates the exact empty
semantic/competing-family precondition, Task-255 set handoff, set-aware
Task-256 atomic handoff, Task-258 statement profile, and B3A witness before
publishing any field. It then publishes set, statement, and witnesses
atomically. Every mutation and family-order failure leaves the prior
`TypedAst` unchanged and permits exact replay; all errors map to the frozen
`InvalidSourceStatement` boundary. The second source/documentation
consistency repeat and final documentation/boundary reread report
**NO FINDINGS**; parent final verification listed in the crate plans
passes, including exact `39`-file scope. Independent final read-only quality
review reports **NO FINDINGS**. All nine hard gates PASS with no score cap;
the valid score is `98/100` (`20/20/15/14/10/10/5/4`). The stated semantic
and coverage deferrals remain unchanged as residual risk. Only the
dedicated implementation commit, post-commit invariant verification, and
fresh next-task inventory remain pending.

## Task 258B3M2B2B3B Frozen Atomic Installation Boundary

B3B reuses `with_source_set_term_statement_witnesses` for the exact
118-byte empty-enumeration profile. Typed installation may publish only the
authenticated Task-252 references, zero-edge Task-255 handoff, Task-256
formula rows, Task-258 base, and one unnamed SetTerm witness atomically.
Lower-stage failure precedence and all legacy/application/structure/B3A
tuples remain unchanged. Partial state, stale fingerprints, family hybrids,
or any nonempty semantic/proof/goal table fail closed. No public API or error
variant is added.

## Task 258B3M2B2B3B Implemented Atomic Installation Closure

`with_source_set_term_statement_witnesses` now recognizes the exact B3B
statement profile after revalidating Task-252, zero-edge Task-255,
set-aware Task-256, Task-258 base, and the one SetTerm witness. Set,
statement, and witness fields publish only after every check succeeds.
Stale dependencies, semantic-table coexistence, family hybrids, and both
B3A/B3B installation orders fail without partial mutation and permit exact
replay. The existing `InvalidSourceStatement` boundary and public API are
unchanged.

## Task 269CT Typed Composite Ownership

`TypedAst` adds one privately boxed optional
`SourceProofLocalLetTypeHandoff`, a const getter, and the by-value one-shot
`with_source_proof_local_let_type` installer. The exact owner requires the
three-node arena and otherwise empty sibling/semantic state. Legacy direct
`source_type` and `source_proof_local_let_binding` fields remain empty. Wrong
dependency, fingerprint, arena, duplicate owner, occupied sibling, or nonempty
semantic state yields `InvalidSourceProofLocalLetType` atomically.

## Task 249PI Implemented Typed Ownership

The existing one-shot source-type installation owns and revalidates the exact
combined handoff, including fail-closed orphan-member rejection. No typed
field, installer, getter, fact, obligation, diagnostic, or semantic result was added.

## Task 249PI Typed Ownership Boundary

Task 249PI adds no `TypedAstParts` field, installer, getter, or serializer. Its
exact combined source-type handoff uses the existing optional `source_type`
owner and one-shot installation; installation revalidates both profile shape
and all parameter/member/expression/head arena identities. Types, local
contexts, facts, coercions, initial obligations, diagnostics, proof,
acceptance, and IR remain unchanged. Task 264 later consumes the lower
fingerprint and member ID 1 through its separately frozen handoff.

## Task 264 Frozen Typed Ownership

Task 264 adds one private optional `SourcePropertyImplementationHandoff` to
`TypedAst`, a read-only getter, and one consuming
`with_source_property_implementation(projection)` transaction. It checks the
current initial-obligation table against the projection baseline, installs the
handoff and exact suffix together, rejects replacement/half-publication and a
Task-259 handoff, and reports `InvalidSourcePropertyImplementation` atomically.

Means installs exactly `PropertyImplementationExistence` then
`PropertyImplementationUniqueness`; equals installs no row. Existing unrelated
baseline rows retain bytes and IDs. `TypedAstParts` has no new public field.
The complete handoff debug appears once; facts, coercions, diagnostics, types,
proofs, acceptance, and IR remain unchanged. The docs prerequisite and lower
Task 249PI add no Task-264 typed field.

## Task 248P Active Typed Ownership

Complete Profile C now installs through the existing one-shot source-context
field and replays deterministically. Exact item, declaration, binding,
binding-context, local-context, context-link, and provenance rows are
revalidated; every type, fact, diagnostic, and initial-obligation table stays
empty in the frozen test. No typed owner field or installation API was added.

## Task 248P Property Context Typed Ownership

Task 248P adds no typed owner field or installation method. A complete Profile
C handoff uses the existing one-shot source-context installation, including
source/module/root, local-context, item/declaration site, and context-link
revalidation. The recovered incomplete branch remains un-installable. No
property payload, type result, fact, obligation, or diagnostic table is
created, and Profile A/B debug/installation bytes remain unchanged.

Post-auth injection and stage-prefix/non-generic-guard assertions preserve
atomic failure and replay. All test-sufficiency repeats and the final
implementation repeat report **NO FINDINGS**.

## Task 258B3M2B2B3C Frozen Atomic Installation

B3C requires the existing `build_with_set_term` and
`validate_installation_with_set_term` path to install Task-48/252/255/256/258
plus one SetTerm witness atomically. Choice target/type-site/request
authentication precedes witness publication; failed mutation, stale replay,
or family hybrid leaves the typed AST unchanged. This prerequisite changes no
typed-AST source, public schema, semantic table, or debug bytes.

## Task 258B3M2B2B3C Implemented Typed-AST Installation

`TypedAst` installs only the authenticated B3C source-set/statement/witness
bundle through the existing atomic transaction. It preserves the exact
choice set fingerprint, validates the complete dependency tuple again, and
rolls back on stale, reordered, hybrid, or generic-guard state. B3A and B3B
remain independently installable in either family order. No public schema,
error text, debug grammar, dependency, or semantic table changed; the private
dormant runner selector is outside this typed/final owner and leaves existing
active-corpus outcomes unchanged.

## Task 258B3M2B2B3D Frozen Atomic Installation

The future installer atomically combines the exact Task-255 qua handoff,
statement base, and one unnamed SetTerm witness, validates the existing
set-only fingerprint tuple, and rolls back on stale, reordered, hybrid, or
generic-guard state. B3A/B3B/B3C remain independently installable in every
family order. No public schema, error text, debug grammar, dependency,
semantic table, or active route changes.

## Task 258B3M2B2B3D Implemented Atomic Installation Inventory

`TypedAst` now installs the exact Task-255 qua handoff, Task-258 statement
base, and unnamed `SetTerm(0)` witness through the existing atomic
transaction. It revalidates the complete dependency/fingerprint tuple and
rolls back on stale, reordered, hybrid, generic-guard, or cross-family
state. B3A/B3B/B3C/B3D remain independent across every family order.

The typed owner grows from 4,932 to 4,933 lines only for the private exact
allowlist. No public schema, error/debug text, dependency, semantic table, or
active route changed. Focused/package tests, formatting, and Clippy pass.
Independent implementation review reports **NO FINDINGS**. Repeated
source/documentation and boundary review also reports **NO FINDINGS** after
the three bounded documentation fixes. Full workspace tests, five CLIs, and
count/hash reruns pass. Independent final read-only quality review reports
**NO FINDINGS**; all nine hard gates PASS with no cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). Only exact staging/cached-diff review,
implementation commit, and post-commit/fresh-next-task gates remain pending.

## Task 258B3M2B2B3E Frozen Atomic Installation

The future typed installation reuses
`with_source_set_term_statement_witnesses` and accepts only the coherent
Task-252/255/256/258/B3E tuple. It preserves one comprehension, generator,
type site, mapper edge, two requests, and one unnamed set-target witness.
Every stage mutation, stale replay, partial publication, or B3A-E hybrid must
roll back atomically. No type/fact/coercion/obligation/diagnostic or public
schema is added.

## Task 258B3M2B2B3E Implemented Atomic Installation Inventory

`TypedAst` now admits the exact B3E Task-252/255/256/258/witness tuple only
after all binding, primary, shared-arena, set, atomic, statement, and witness
fingerprints match. One condition-free comprehension, one generator/type
site, zero conditions, and one unnamed set-target witness install atomically.
Partial publication, extra ownership, stale lower state, or sibling hybrids
leave the original AST unchanged.

The owner grows from 4,933 to 4,934 lines for the private allowlist. All five
B3A-E families remain independent across 120 orders. No public typed-AST
API, semantic table, proof state, diagnostic, or debug grammar changes.
Final source/documentation consistency and independent quality report
**NO FINDINGS**; full verification and all nine hard gates PASS at valid
`100/100`. Staging and post-commit gates subsequently closed in
implementation commit `e4479691db3b0a8785bb16e94d386bd71a394274`;
fresh inventory selected Task 258B4A.

## Task 258B4A Frozen Paired Installation

The only new installation path is
`with_source_formula_composition_statement(mut self, composite:
SourceCompositeFormulaHandoff, composition:
SourceFormulaCompositionHandoff, statement: SourceStatementHandoff) ->
Result<Self, TypedAstError>`. It consumes the exact Task-257B1 composite and
formula-composition handoffs plus the B4A statement in that order,
revalidates every lower fingerprint and the statement's optional
composite/composition fingerprints, and publishes all three atomically. The
existing lower-only `with_source_formula_composition` behavior and debug
bytes are unchanged.

The transaction accepts only the private double-LF route and exact
`Composite(0)` statement/candidate pair. Atomic Task-258 statement families,
duplicate owners, stale arenas, reordered or hybrid lower handoffs, and
partial tuples fail with `TypedAstError::InvalidSourceStatement`, leave state
byte-identical, and permit replay. No semantic table or lower root-ownership
row changes.

Repeated read-only documentation review reports **NO FINDINGS**. Independent
final quality passes all nine hard gates with no cap at valid `100/100`;
only staging, commit, and post-commit inventory remain.

## Task 258B4A Implemented Paired Installation

`with_source_formula_composition_statement` now revalidates and installs the
exact Task-257B1 composite/composition pair and Task-258 B4A statement as one
transaction. Both optional statement fingerprints, `Composite(0)` links,
source identity, lower fingerprints, table profiles, and family exclusivity
must agree before mutation. Atomic-statement families, duplicate owners,
stale/reordered/partial handoffs, rooted or relocated lower near misses, and
cross-family hybrids return `InvalidSourceStatement`, leave the AST
byte-identical, and permit clean replay. The lower-only installer and every
semantic table remain unchanged.

## Task 258B4B Frozen Paired Installation

`with_source_formula_composition_statement` reuses its public signature.
It may publish B4B only when the exact Task-257B2 composite/composition
handoffs, the B4B statement handoff, Task-252/256 state, source/module
identity, fingerprints, rootless 124-node arena, and empty incompatible
families all agree. B4A remains the only Task-257B1 statement profile.
The existing B4A crate-private predicate must first be narrowed from shared
cardinalities to exact B4A identity; the new exact B4B predicate is not
interchangeable with it.

The installer rejects every B4A/B4B pairing hybrid, duplicate or partial
state, active atomic statement family, Task-248 context, semantic table,
rooted arena, relocated owned site, and stale fingerprint before mutation.
All installation orders are atomic and replayable. The lower-only
`with_source_formula_composition` remains unchanged and still accepts B1,
B2, and B3 only as lower transport.

## Task 258B4C Frozen Paired Installation

`with_source_formula_composition_statement` retains its public signature and
may install B4C only for a matched Task-257B3 composite/composition handoff
and exact B4C statement. The runner selector and
`SourceStatementProducer` own raw source, Surface, and resolver
authentication. Before mutation this installer revalidates the resulting
producer-authenticated statement owner/context/candidate rows and handoff
identity, the matched lower fingerprints, the rootless 66-node lower arena,
binding `4/4/0`, primary `6/6/0`, atomic
`3/0/0/0/0/0/0/6/6`, composite `3/0/1/3/3/2/6`, composition `3/6`,
the 24-site lower ownership partition, and upper `1/1/1/0/1`.

Both statement and candidate must target `Composite(0)`, statement context 0
must expose exactly reserve binding `[0]`, and input facts must be empty.
The installer recognizes only B1/B4A, B2/B4B, and B3/B4C. Any cross-pairing,
duplicate/partial state, stale fingerprint, rooted or relocated arena,
altered ownership, active atomic statement family, semantic-table
coexistence, or lower-selector mismatch fails with the existing
`TypedAstError::InvalidSourceStatement`, leaves the AST byte-identical, and
permits replay.

The mandatory lower-selector compatibility prerequisite is a separate
logical task and commit before this installer changes. The B4C transaction
adds no public installer, error variant, debug grammar, fact, theorem
acceptance, proof, or semantic table; the lower-only installer remains
unchanged.

## Task 258B4C Implemented Paired Installation

The existing paired installer now admits only the exact B3/B4C transaction.
It revalidates binding `4/4/0`, primary `6/6/0`, atomic
`3/0/0/0/0/0/0/6/6`, composite `3/0/1/3/3/2/6`, composition `3/6`,
upper `1/1/1/0/1`, all fingerprints, and the rootless 66-node arena. Every
anchor and recovery state is exact; 24 nodes remain lower-owned, theorem
node 62 is the only statement-owned node, and 41 nodes remain unowned.

Cross-pairings, duplicate/partial or atomic state, Task-248 occupancy, stale
fingerprints, rooted/relocated arenas, and altered ownership fail before
mutation with the existing error and replay deterministically. No public
installer or semantic table was added.

## Task 258B5A Frozen Paired Installation

The existing reference installer may be generalized privately to accept two
and only two authenticated transactions: the unchanged B1 same-scope profile
and the B5A ancestor/descendant profile. B5A requires exact base
`1/5/5/5/5`, reference `1/1`, lower handoffs, all fingerprints, the
93-node/root-92 arena, and 20-owned/73-unowned partition.

The label and citation must retain scopes `[0]` and `[0,1]`, respectively,
with statement ordinals 1 and 4, private/local-only contribution 0, exact
ranges and origins, and the matching resolver key at node 82. Cross-pairing,
partial/duplicate installation, stale replay, relocation/recovery, wrong
scope or provenance, Task-248/other-family occupancy, and altered ownership
fail before mutation. The operation adds no semantic table and does not
change B1 debug output or a public signature.

## Task 258B5A Implemented Paired Installation

The private paired installer now admits exactly the unchanged B1 same-scope
transaction or the exact B5A ancestor/descendant transaction. B5A
installation revalidates base `1/5/5/5/5`, reference `1/1`, all lower
handoffs and fingerprints, every 93-node arena identity, the `20/73`
ownership partition, label scope `[0]`, citation scope `[0,1]`, statement
ordinals `1/4`, contribution 0, and resolver key node 82.

Duplicate or partial state, B1/B5A cross-pairing, stale fingerprints,
relocation or recovery, wrong range/origin/scope/contribution/key,
Task-248 or another family, and altered ownership fail before mutation.
Failed installation leaves the typed AST replayable. The implementation adds
no public installer, error variant, semantic table, or B1 debug change.

## Task 258B5B Frozen Imported-Target Installation

After the separate lower-stage opt-in label prerequisite, paired
installation admits a third exact reference profile: B5B base
`1/2/2/2/2`, local labels/citations `0/1`, 57-node/root-56 arena, and
`8/49` ownership. The citation target is
`SourceStatementCitationTarget::Imported`, its kind is `SimpleImported`,
and the singular projection is an imported/public/exported theorem `Ref`.

The public target enum is necessary because a mandatory local
`SourceStatementLabelId` would fabricate a row that the resolver does not
contain. Existing B1 and B5A citations become `Local(id)` without changing
their debug bytes or behavior. The installer must match B1, B5A, or B5B
base/reference fingerprints exactly and reject every cross-pair, including
B5A-local with B5B-imported state.

Duplicate or partial installation, absent or extra local labels, wrong
import visibility/export/kind/module/namespace/contribution/origin/anchor/
range/path, recovered or relocated rows, wrong node 48 key, Task-248 or
other-family occupancy, and altered ownership fail before mutation and leave
replay available. No semantic table or runner-facing public schema is added.

## Task 258B5B Implemented Imported-Target Installation

Typed installation now admits exactly three mutually exclusive reference
profiles: B1 local, B5A local, or B5B imported. B5B requires the frozen
57-node/root-56 resolver fingerprint, Task-258 `1/2/2/2/2 + 0/1`,
`8/49` ownership, zero local labels, one
`SourceStatementCitationTarget::Imported`, and one `SimpleImported`
citation. B1/B5A construct `Local(id)` and preserve their prior bytes.

Before mutation the installer revalidates every base/reference fingerprint,
resolver node kind and key, import/projection/reference provenance, citation
row, and owned-node partition. It rejects every B1/B5A/B5B cross-pair,
partial or duplicate installation, and occupied semantic state atomically.
No public installer, semantic table, runner-facing schema, or diagnostic is
added.

## Task 258B5C Frozen Typed-Installation Exclusion

B5C ends at an unresolved resolver result and never satisfies the
source-statement reference handoff. `TypedAst` consequently installs no
B5C base/reference profile, label/citation row, binding context, owned
Surface node, checked formula, fact, proof, or semantic table. It must not
interpret an unresolved result as a B1, B5A, or B5B near match.

The later active declaration-symbol runner task consumes validated R-032A
and R-032B output directly. Existing B1/B5A/B5B installation predicates, mutation
atomicity/replay, debug bytes, public installers, and error variants remain
unchanged.

R-032B's closed edge table starts with exact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock`, exact-one normal Root/CompilationUnit structural children, and
direct-normal theorem scanning. Its no-ordinal/no-descent default deny
prevents every excluded, relocated, or mixed form from reaching typed
installation. Positive edge and negative relocation/mixed-list tests remain
lower-stage tests.

The runner's independent env/projection/contribution provenance mutations
all terminate at `proof_scope_input`; even a structurally coherent mutation
cannot become confinement or a `TypedAst` row. Source bytes plus exact normal
AST remain the only selector, and the 48-file scope remains unchanged.

## Task 259 Frozen Typed-AST Transaction

The future Task-259 projection contains a
clone of the authenticated baseline `InitialObligationTable`, a
`SourcePredicateDefinitionHandoff`, and the completed table produced by
preserving that clone and appending exactly one
`PredicatePropertyCorrectness` row. The
handoff stores source/module identity and exact debug fingerprints for
`SourceBindingContextHandoff`, `SourceTypeApplicationHandoff`,
`SourcePrimaryTermHandoff`, and `SourceAtomicFormulaHandoff`.

`TypedAst::with_source_predicate_definition` is one-shot and publishes the
handoff and obligation table atomically. It requires the four lower handoffs,
revalidates every fingerprint, dense target, source site/range/context,
predicate resolver identity, correctness link, obligation owner/kind/range/
assumptions/goal/provenance/status, and rejects every partial or stale
transaction. It additionally requires its current obligation table to equal
the retained baseline exactly; mismatch rejects the whole projection.
Failure leaves neither Task-259 rows nor obligation linkage.
`TypedAstParts` gains no Task-259 field and is not an alternate installation
path. `ResolvedTypedAst` revalidates and clone-preserves the same handoff,
obligations, IDs, order, fingerprints, and debug bytes without
reconstruction.

The frozen exact transaction has table cardinality `1/2/1/1/1` and exactly
one available-for-handoff `Pending` obligation. It adds no type fact,
coercion, diagnostic, `VcId`, proof status, accepted result, axiom, or IR
node. The guard remains a source-formula link rather than an obligation
assumption. Future semantic consumers must not infer a FOL goal from the
opaque strings frozen here.

## Task 248 Two-Parameter Profile-B Typed Installation

The lower implementation does not add an installer. It returns the existing
projection whose local contexts are installed through the current
`TypedAstParts::source_context` path. Before projection, the private runner
validates four caller sites against the one shared `TypedArena`: module site
is the root at context 0; definition and two declaration sites have their
exact ranges and context 1; all nodes are normal and distinct as sites.

Existing typed validation independently rechecks anchors, contexts, root
ownership, item/declaration sites, and links. Any stale or duplicate site
fails without a partial handoff. Profile A installation/debug/recovery and
all type/fact/coercion/obligation/diagnostic tables remain byte-compatible.
Task-259 installation is still a later, separate transaction.

## Task 260 Typed Functor-Definition Installation

`TypedAst` gains one optional `SourceFunctorDefinitionHandoff`. Its one-shot
installer accepts the producer projection only when all lower handoffs are
already present with exact fingerprints and the supplied initial-obligation
table is precisely the caller baseline plus the two means rows. Handoff and
complete obligation table install atomically.

Task-259 and Task-260 handoffs remain separate optionals but are mutually
exclusive in Task 260. The Task-260 installer rejects a current Task-259
handoff or a baseline containing `PredicatePropertyCorrectness`; final
assembly rejects both handoffs together. There is no cross-family install-
order promise and no Task-259 compatibility edit. No Task-260 path may
replace Task-259 state, create facts/types/coercions/diagnostics, or install
proof/acceptance output.

The producer and installer also reject a baseline already containing
`FunctorExistence` or `FunctorUniqueness`. With the handoff installed, exactly
the two final linked functor rows exist; without it, either functor kind is an
orphan rejected by final assembly.

## Task 249R Definition-Return Ownership Addendum

Task 249R adds no `TypedAstParts` field and no installation method. It extends
the existing `SourceTypeApplicationHandoff` before `TypedAst::try_new`;
`validate_source_type` rechecks both definition owners and appended return
expressions against the owned arena. The same optional `source_type` field is
the sole owner. Empty-return legacy debug bytes remain unchanged, while the
Task-260 profile owns one combined `2/4/0/2` handoff.

## Task 249M Mode-RHS Ownership Addendum

Task 249M adds no `TypedAstParts` field or installer. It extends the existing
source-type handoff before `TypedAst::try_new`; source-type validation rechecks
the exact owner, appended expression/head, mutual exclusion from Task 249R,
and arena identity. The same optional field is the sole owner, legacy and
Task-249R debug bytes stay unchanged, and the Task-262 lower profile is one
combined `2/3/0/0/1` handoff with no semantic output.

## Task 249M Active Typed Ownership

The standalone mode-RHS extension is now implemented before installation.
`TypedAst::try_new` revalidates the row and all three source-type expressions
through the existing optional handoff, with no new field or installer. Exact
tests keep type, fact, coercion, obligation, and diagnostic tables empty.

## Task 262 Active Mode-Definition Transaction

`TypedAst` owns one optional `SourceModeDefinitionHandoff` installed only by
`with_source_mode_definition`. The installer requires the committed Task-248
source context and combined Task-249/249M source-type handoff, compares the
producer's retained baseline with the current obligation table, and publishes
the six-table handoff plus its exact one-row `Sethood` suffix atomically.
`TypedAstParts` remains unchanged and is not an alternate installation path.

The transaction rejects prior Task-259/260/261/262 ownership, sibling-only
obligation kinds, stale lower fingerprints, and an orphan goal/provenance in
the `source.definition.mode` domain. Unrelated baseline `Sethood` rows remain
valid and byte-preserved. The pending row and unresolved RHS-inhabitation
request grant no goal/guard composition, proof, discharge, acceptance, fact,
IR, or VC semantics.
## Task 249S Standalone Member-Type Ownership Addendum

Task 249S adds no `TypedAstParts` field or installation path. The existing
optional `source_type: Option<SourceTypeApplicationHandoff>` owns the exact
standalone `0/4/0/0/0/4` value. Installation revalidates all four member owner,
expression, and head sites against `TypedArena`; missing or corrupt values fail
as `InvalidSourceType`. Types, facts, coercions, initial obligations,
diagnostics, context, and every Task-263 upper field remain empty/absent.

## Task 249S Active Typed Ownership Result

The existing optional `source_type` field and one-shot installation path are
the sole owner. Installation revalidates the exact member table, expression
table, source/module identity, all twelve arena sites, and mutual exclusion
with sibling profiles. All semantic tables and Task-263 upper fields remain
empty or absent.

## Task 263 Frozen Typed Ownership

The future `with_source_structure_definition` is a one-shot compare-and-swap
transaction. It requires the exact Task-249S source-type fingerprint, an
unchanged baseline/final obligation pair, valid `2/4/1/2/0` rows, and an empty
derived coherence table before publishing one optional handoff. It adds only
the same-named getter and `TypedAstError::InvalidSourceStructureDefinition`;
`TypedAstParts` has no replacement path.

Install rejects prior Tasks 259--262 definition-family occupancy in either
observable order and changes no types, facts, coercions, diagnostics, or
obligation row. In particular Task 259's correctness transaction and mixed
predicate/functor boundary remain independent.

The handoff privately clone-retains the complete baseline table. Installation
requires current == projection baseline == private snapshot == projection
final, while exact-runner baseline is empty and checker tests also preserve a
nonempty unrelated baseline. Same-length snapshot corruption is transactional
failure; the snapshot has no getter or stable-debug serialization.

## Task 263 Active Typed Ownership

`TypedAst::with_source_structure_definition` now installs the projection once,
requires byte-equal baseline/current/final obligation tables, revalidates every
frozen dependency and row, and rejects all Task-259--262 definition-family
owners in both installation orders. Failure leaves the original typed value
unchanged.

## Task 264 Active Typed Ownership

`TypedAst::with_source_property_implementation` now installs the frozen
handoff and obligation suffix atomically. It revalidates all lower
fingerprints, rows, arena ownership, and baseline bytes; means appends exact
existence then uniqueness rows, while equals appends none. Duplicate, orphan,
extra, stale, half-published, or Task-259--263 sibling transactions fail as
`InvalidSourcePropertyImplementation` without changing the input value. The
private field is observable only through the documented read-only getter.

## Task 269A Frozen Typed Ownership

`TypedAst` will add one private optional
`SourceProofLocalDeclarationHandoff`, read-only getter, and consuming
`with_source_proof_local_declaration` installer. `TypedAstParts` stays
unchanged. Installation is legal only over the exact Task-258B3N lower bundle
and empty semantic tables; it replays all five fingerprints, all 51 existing
nodes, the one declaration row, and the `2/1/0 -> 2/2/0` binding transition.

Success appends the new debug block but changes no arena node. Duplicate,
orphan, stale, same-length-corrupt, lower-hybrid, or semantic-table-bearing
inputs fail transactionally as `InvalidSourceProofLocalDeclaration`. Existing
Task-258B3N construction without the installer remains byte-identical.

## Task 269A Active Typed Ownership

The private optional field, read-only getter, and consuming installer are now
implemented exactly. The installer requires the complete Task-258B3N lower
bundle, replays the frozen handoff, changes no node or semantic table, and
rejects missing, duplicate, stale, same-length-corrupt, and sibling inputs
atomically. Handoff phases 1--6 run before the crate-private phase-7 one-shot
availability guard. The legacy lower debug bytes remain unchanged.

## Task 269B frozen Typed ownership increment

The same optional handoff and one-shot installer accept an exact B3M1
transaction only when the complete statement/witness/primary/56-node bundle
is already installed. No field or method changes. Cross-profile B3N/B3M1
fingerprints, a bound unnamed witness, partial lower bundles, siblings, or
semantic coexistence fail before publication; valid installation preserves
all lower bytes and empty semantic tables.

## Task 269B active Typed ownership increment

The existing installer allowlist now admits exact B3M1 beside B3N and changes
no field or method. It preserves the 56-node arena and empty semantic tables,
publishes the one binding transaction once, and rejects duplicate, partial,
cross-profile, corrupt-arena, stale-fingerprint, or semantic-coexistence input
transactionally.

## Task 269CP typed-owner exclusion

No `TypedAst` field, installer, source type, binding, node link, or semantic
table is added by the lower prerequisite. The exact 51-node Surface profile is
authenticated privately by the runner and is not claimed as typed ownership.
Task 269C must freeze any typed/final projection independently.

## Task 269C frozen typed owner

`TypedAst` adds a sibling optional `source_proof_local_let_binding` handoff,
getter, and one-shot installer. Its admitted base has no node/root, other
source handoff, or semantic table. Installation validates the exact binding
handoff and adds no type, node link, fact, coercion, initial obligation, or
diagnostic. Duplicate, stale, partial, Task-269A/B cross-family, and semantic-
coexistence inputs fail transactionally. The binding remains
`BindingTypeSite::Missing`.

## Task 269C Active Typed Ownership

The frozen field/getter/installer are implemented. The installer validates the
entire handoff before mutation, rejects both installation orders for every
sibling family, and leaves all node, type, fact, proof, obligation, and
diagnostic payloads empty.

The optional handoff uses private boxed storage to keep `TypedAst` stack size
stable; the frozen public by-value installer and `Option<&Handoff>` getter are
unchanged.

## Task 269CT Implemented Typed Ownership

`TypedAst` owns one boxed composite and exact three-node arena. Installation
revalidates dependency, overlay, source type, fingerprints, arena, empty
semantic tables, and all sibling source owners before publication. Duplicate,
direct Task-269C sibling, direct source-type sibling, stale payload, or any
occupied owner fails atomically with `InvalidSourceProofLocalLetType`.

## Task 269GP No-Typed-Owner Boundary

Task 269GP is runner-private and installs no `TypedAst` field. All Task-269A/B
and boxed Task-269C/CT ownership, one-shot validation, and mutual exclusions
remain unchanged.

The implemented runner route has no typed owner or installation call; focused
Task-269C/CT compatibility tests pass unchanged.

## Task 269GS No-Typed-Owner Reconciliation

The scope decision does not install a `TypedAst` field. Task 269G must first
freeze and implement binding-only ownership; Task 269GT remains the separate
type admission. Existing one-shot ownership and semantic exclusions are
unchanged.

## Task 269G Typed Owner

`TypedAst` adds one private optional given-binding handoff, a read-only getter,
and one-shot installer with `InvalidSourceProofLocalGivenBinding`. It accepts
only an otherwise-empty semantic/node profile and publishes no type, fact,
coercion, initial obligation, diagnostic, or coexisting source handoff.

## Task 269G Active Typed Ownership

The frozen field, getter, and one-shot installer are implemented. Installation
validates the entire handoff before mutation, rejects both installation orders
for every sibling family and all nonempty semantic tables, and leaves node,
type, fact, proof, obligation, and diagnostic payloads empty. Private boxed
storage preserves the by-value public installer and `Option<&Handoff>` getter.

## Task 269GT Frozen Typed Owner

`TypedAst` will add one boxed optional Given-type composite, a read-only getter,
and one-shot by-value installer. Only the exact three-node arena and otherwise-
empty profile are accepted. The direct Given binding, generic source type,
every sibling handoff, contexts/types/facts/coercions/initial obligations/
diagnostics, and semantic nodes remain absent; all failures are atomic.

### Task 269GT implemented typed owner

`TypedAst` now owns one boxed optional `source_proof_local_given_type` immediately after the Given-binding slot, exposes the frozen getter and consuming installer, authenticates the exact three-node arena, and rejects duplicate, direct Given-binding, Let, other source-owner, or nonempty semantic state atomically. No `TypedAstParts` field or semantic table is added.
