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
