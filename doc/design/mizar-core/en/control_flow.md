# Control-Flow Preparation

> Canonical language: English. Japanese companion:
> [../ja/control_flow.md](../ja/control_flow.md).

This document specifies phase-10 `ControlFlowIr` construction owned by
`mizar-core`. It refines architecture 06 Step 6, architecture 07 Step 1, and
specification chapter 20. It is intentionally a spec-only task: no Rust source
is implemented in task 14.

## Scope

`ControlFlowIr` is the algorithm-specific projection between phase-9 `CoreIr`
algorithm shells and later VC/extraction consumers. It owns:

- deterministic basic blocks and control-flow edges for one core algorithm;
- local binding metadata for parameters, result binders, statement locals, and
  site-local `Pick` binders;
- statement-level assignment and initialization state used by flow diagnostics;
- placement of contracts, assertions, loop invariants, and decreasing terms on
  CFG sites;
- ghost/runtime use classification and ghost-effect metadata;
- source maps from core statements, CFG blocks, loop sites, and local bindings
  back to `CoreSourceRef`.

`ControlFlowIr` does not own:

- proof acceptance, theorem discharge, or kernel checking;
- `VcId` assignment, `ObligationAnchor` construction, canonical VC
  fingerprints, or VC-local dependency slices; those belong to `mizar-vc` by
  internal architecture 07;
- source-to-checker extraction of algorithm payloads;
- code extraction or target-runtime lowering;
- public diagnostic code allocation.

The boundary with `mizar-vc` is therefore one-way: `mizar-core` constructs and
validates source-mapped control-flow facts; `mizar-vc` consumes them, together
with `CoreIr` and obligation seeds, to build canonical `VcIr`.

## Classification

Task 14 classifications:

- `design_drift`: architecture 07's affected-module list mentions a
  `mizar-vc/control_flow.md` module, but internal architecture 07 assigns
  elaboration and control-flow preparation to `mizar-core`. This document
  records the current crate boundary: `mizar-core` owns `ControlFlowIr`;
  `mizar-vc` owns VC generation.
- `design_drift`: architecture 07 also describes phase 11 as preparing
  algorithm control-flow facts. Under the current crate boundary, phase 10 in
  `mizar-core` prepares those facts; phase 11 in `mizar-vc` consumes them when
  assigning `VcId`s and building `VcIr`.
- `external_dependency_gap`: source-to-checker extraction for full algorithm
  payloads and parser task 32-34 source-derived coverage remain outside this
  task. The implementation tasks use explicit core/Rust fixtures until the
  upstream payload bridge exists.
- `external_dependency_gap`: snapshot and claim payloads from specification
  chapter 20 are not present in the task-13 `CoreAlgorithmStmtKind` surface.
  They must not be silently dropped; future checker-owned shells must carry
  explicit snapshot-site metadata before phase 10 can lower them.
- `external_dependency_gap`: `mizar-vc`, `mizar-kernel`, and `mizar-proof`
  are not implemented as downstream consumers in this crate task. Do not
  fabricate APIs for them.
- `deferred`: Rust data structures, CFG construction, contract/ghost/termination
  attachment, diagnostics, and obligation-seed handoff are tasks 15-18.

No `spec_gap` blocks task 14. Chapter 20 defines the language behavior, and
the design documents define the ownership boundary.

## Inputs

Control-flow preparation consumes a valid phase-9 `CoreIr` subset:

- a `CoreAlgorithm` row;
- all `CoreAlgorithmStmt` rows reachable from that algorithm's root
  `statements`, including nested `If`, `While`, and `Match` bodies;
- lowered terms and formulas referenced by statement values, conditions,
  assertions, contracts, invariants, and decreasing measures;
- algorithm diagnostics and source maps from phase 9;
- generated origins and obligation seeds already present in `CoreIr`, without
  assigning new `VcId`s`.

The constructor may rely on raw `CoreIr` validation that every reachable nested
statement has `owner` equal to the containing algorithm. If this invariant is
violated, construction must fail with a structured diagnostic instead of
guessing ownership.

## Data Model

The implementation may refine names, but it must preserve this semantic shape.

```rust
struct ControlFlowIr {
    algorithm: CoreAlgorithmId,
    item: CoreItemId,
    symbol: SymbolId,
    entry: BasicBlockId,
    blocks: ControlFlowBlockTable,
    locals: ControlFlowLocalTable,
    contexts: ControlFlowContextTable,
    contracts: ControlFlowContractSet,
    loops: ControlFlowLoopTable,
    exits: Vec<ControlFlowExit>,
    ghost_effects: ControlFlowGhostTable,
    termination: ControlFlowTerminationPlan,
    source_map: ControlFlowSourceMap,
    diagnostics: ControlFlowDiagnosticTable,
}
```

A module-level output contains one `ControlFlowIr` per successfully lowered
algorithm and diagnostic records for skipped/error algorithms. Phase-9
diagnostics carried by `CoreAlgorithmStmtKind::Error` are preserved by
reference, while phase-10-only diagnostics use control-flow diagnostic rows so
the builder never mutates `CoreIr`.

### Basic Blocks

```rust
struct ControlFlowBlock {
    id: BasicBlockId,
    algorithm: CoreAlgorithmId,
    statements: Vec<CoreAlgorithmStmtId>,
    terminator: ControlFlowTerminator,
    context_in: ProgramContextId,
    context_out: Vec<ProgramContextId>,
    reachable: Reachability,
    source: CoreSourceRef,
}

enum ControlFlowTerminator {
    Goto(BasicBlockId),
    Branch {
        condition: CoreFormulaId,
        then_block: BasicBlockId,
        else_block: BasicBlockId,
    },
    Switch {
        scrutinee: CoreTermId,
        arms: Vec<ControlFlowSwitchArm>,
        join: Option<BasicBlockId>,
    },
    Return(Option<CoreTermId>),
    Break { loop_id: LoopId, target: BasicBlockId },
    Continue { loop_id: LoopId, target: BasicBlockId },
    Unreachable,
    Error(ControlFlowDiagnosticId),
}
```

Blocks are allocated in deterministic source traversal order. Synthetic join,
loop-header, and exit blocks are ordered immediately after the construct that
introduces them, using stable local counters rather than hash iteration.

`statements` keeps the original source-order statement-shell ids whose effects
occur inside the block before the terminator. Structured statements such as
`If`, `While`, and `Match` normally become terminators or synthetic loop/switch
metadata instead of remaining as opaque block statements.

### Locals

```rust
struct ControlFlowLocal {
    id: LocalId,
    algorithm: CoreAlgorithmId,
    binder: CoreBinder,
    kind: LocalKind,
    declaration: LocalDeclaration,
    mutability: LocalMutability,
    ghost: bool,
    initialized_at: Option<CoreAlgorithmStmtId>,
    source: CoreSourceRef,
}

enum LocalKind {
    Parameter,
    Result,
    Let,
    Pick { witness_ty: Option<CoreFormulaId> },
    HiddenLoopValue,
}

enum LocalDeclaration {
    Parameter,
    Result,
    Var,
    Const,
    GhostVar,
    GhostConst,
    PickRuntime,
    PickGhost,
    HiddenLoopValue,
    Unsupported(CoreVarRole),
}

enum LocalMutability {
    Immutable,
    Mutable,
    Unknown,
}
```

Parameters are immutable. `CoreAlgorithm.result` creates a result local when
present. `Let` and `Pick` binders create locals in statement order. A `Pick`
local is site-local even when another pick has the same witness type; it never
reuses stable-choice generated origins. Hidden loop values may represent
old-state values such as loop measures, range bounds, or collection domains
when later statement forms require them.

`LocalDeclaration` is derived only from checker-owned declaration metadata.
`CoreBinder.role` may carry that semantic role in explicit fixtures, and
statement `ghost` flags refine runtime versus ghost state, but phase 10 must
not parse source spelling such as `var` or `ghost const`. If a local declaration
does not identify mutability and ghostness, construction records the original
role as `LocalDeclaration::Unsupported`, sets mutability to `Unknown`, and emits
an unsupported local-declaration diagnostic instead of defaulting to mutable
runtime state. This diagnostic is a structural construction diagnostic owned by
Task 15, not a replacement for the later data-flow diagnostics in Task 17.
Parameters and result locals are immutable runtime locals unless checker-owned
metadata explicitly marks a later extension as ghost-only.

Assignments to `CorePlace` are resolved through checker-owned place metadata
when available. Until source-to-checker extraction provides richer lvalue
identity, explicit Rust fixtures may use canonical `CorePlace` keys. The CFG
must not infer aliasing from source spelling.

### Context and Assignment State

Flow construction records, but does not prove, per-block context summaries:

- definitely initialized locals at block entry and exit;
- possibly assigned mutable places;
- active loop stack for `break` and `continue` resolution;
- path conditions introduced by branches and match arms;
- active invariant facts available at loop headers and exits;
- ghost-only state visible only to specification contexts.

```rust
struct ProgramContext {
    id: ProgramContextId,
    definitely_initialized: Vec<LocalId>,
    maybe_assigned: Vec<CorePlace>,
    available_facts: Vec<ContextFactId>,
    assignment_effects: Vec<AssignmentEffectId>,
    call_effects: Vec<CallSiteId>,
    path_conditions: Vec<CoreFormulaId>,
    active_invariants: Vec<CoreFormulaId>,
    loop_stack: Vec<LoopId>,
    ghost_visible: Vec<LocalId>,
}
```

Every block references its input context and all normal/exit output contexts.
Context rows are source-independent semantic summaries; source links live on
the block, statement placement, local, loop, and exit records that caused the
context transition.
`available_facts` records entry `requires`, local equalities from `let`/`const`
initializers, asserted formulas after their checkpoint, loop facts, and
known-terminating call `ensures` facts. `assignment_effects` records old-state
assignment transformers and hidden values needed for later Hoare-style VCs.
`call_effects` references call-site rows so phase 11 can distinguish
unconditional facts from conditional partial-call metadata without
reconstructing calls from source syntax.

Task 15 records the minimum context needed for CFG construction: parameters are
definitely initialized at entry, result locals are present but not definitely
initialized, initialized `Let` and all `Pick` locals are added to successor
contexts, `Assign` statements record a place write, branch/loop conditions are
kept as path conditions on the corresponding successor contexts, and assertion,
call facts remain empty or placed-only until checker-owned call payloads exist.
Task 16 attaches contract, ghost, assertion, invariant, and termination
metadata. Task 17 adds flow diagnostics for invalid use of those contexts.

Join points intersect definitely initialized sets and join path conditions
symbolically. If a later task cannot represent a precise join, it must keep a
conservative context and emit diagnostics for unsupported precision instead of
claiming facts that are not guaranteed on every path.

### Contracts and Assertions

```rust
struct ControlFlowContractSet {
    requires: Vec<ContractSite>,
    ensures: Vec<ContractSite>,
    calls: Vec<CallSiteId>,
    assertions: Vec<AssertionSite>,
    loop_invariants: Vec<LoopInvariantSite>,
    decreasing: Vec<TerminationMeasureSite>,
}
```

`requires` clauses become entry assumptions and caller-side obligations for
later `mizar-vc` generation. `ensures` clauses attach to every `Return`
terminator. `Assert` statements become checkpoint sites and then facts in the
normal successor context. Loop invariants attach to loop headers, normal back
edges, `break`, and `continue` exits as required by specification 20.5.1.

`ContractSite` rows carry a kind (`Requires` or `Ensures`), the lowered formula,
the exact source, and an entry or return placement. `AssertionSite` rows carry
the formula, source, and a placement for either an aggregate algorithm-contract
payload or a statement checkpoint with its block and successor context.
`LoopInvariantSite` rows carry the formula, source, and a placement for either
an aggregate algorithm-contract payload or a loop header, normal backedge, break
exit, or continue exit; loop placements carry the owning `LoopId`.
`TerminationMeasureSite` rows carry the term, source, and a placement for the
algorithm header, loop header, or loop continue edge.

`CoreContractSet.assertions` and `CoreContractSet.invariants` are preserved as
aggregate `AlgorithmContract` placements. They are metadata anchors for later
obligation generation and do not by themselves add unconditional successor
facts; statement `Assert` and loop `While` annotations provide the flow-local
facts and edge sites.

`CallSiteId` links to the flow-level call-site table. In the current Task 16
surface, `CallSite` only records the statement and source of an explicit
checker-owned call payload. Resolved callees, argument terms, instantiated
`requires`/`ensures`, and termination availability remain deferred until the
checker payload exposes that data. Phase 10 must not infer call contracts from
spelling; without an explicit payload, the call-site table stays empty. When
that payload exists in a later task, partial or unknown call `ensures` must
remain conditional metadata for later `mizar-vc` obligation generation rather
than unconditional successor facts, and phase 11 assigns concrete `VcId`s`.

Task 14 specifies placement. Task 16 attaches the concrete non-call tables and
keeps call rows empty unless an explicit checker-owned call payload exists.
Task 18 defines the final obligation-seed handoff.

### Ghost Effects

The ghost table records local visibility and separates assignment effects and
pick locals into runtime and ghost-only lists. Assertion, contract, invariant,
and decreasing sites are specification-only by construction and are recorded in
the contract/termination tables rather than as separate ghost-table rows. A
ghost value may feed specification contexts (`requires`, `ensures`, `assert`,
`invariant`, `decreasing`) but must not flow into runtime assignments, runtime
return values, or non-ghost calls. Violations are flow diagnostics.

Ghost-only picks still create non-emptiness obligations, but their runtime
effect is erased by later extraction. Runtime picks are represented as local
`Pick` bindings; neither form emits a generated stable-choice origin.
Task 16 records assignment effects and pick locals in separate runtime and
ghost-only lists so later erasure and diagnostics can consume the split without
re-parsing statements.

### Termination

`ControlFlowTerminationPlan` records:

- header-level decreasing terms for recursive or mutually recursive
  algorithms;
- loop-level decreasing terms attached to loop headers;
- `continue` edges that must prove the measure has already decreased;
- sites that are partial because no decreasing measure is present.

When an algorithm has no algorithm-level decreasing term, phase 10 records an
algorithm partial site. When a loop has no loop-level decreasing term, phase 10
records a loop partial site. Decreasing terms with explicit payloads are attached
to the algorithm header, loop header, and reachable continue edges.

The plan is metadata for later VC generation. Phase 10 does not prove
well-foundedness, assign `VcId`s, or promote an algorithm to a terminating
functor.

### Source Map

```rust
struct ControlFlowSourceMap {
    algorithm_sources: Map<CoreAlgorithmId, CoreSourceRef>,
    block_sources: Map<BasicBlockId, CoreSourceRef>,
    statement_placements: Map<CoreAlgorithmStmtId, ControlFlowStatementPlacement>,
    local_sources: Map<LocalId, CoreSourceRef>,
    loop_sources: Map<LoopId, CoreSourceRef>,
    exit_sources: Map<ControlFlowExitId, CoreSourceRef>,
}
```

Every block, local, loop, exit, diagnostic, and contract/termination site must
retain a source reference. Synthetic blocks use the source of the construct
that forced their creation plus generated provenance explaining the synthetic
role. Diagnostics must point at the narrowest responsible source when possible.
`statement_placements` records where each core statement shell contributed to
the CFG: a block statement, terminator, loop header, switch arm, local binding,
contract/checkpoint site, or error site. Structured statements that disappear
into synthetic blocks are therefore still traceable from source statement id to
CFG ownership.

### Diagnostics

```rust
struct ControlFlowDiagnosticId(usize);

struct ControlFlowDiagnostic {
    kind: ControlFlowDiagnosticKind,
    algorithm: CoreAlgorithmId,
    statement: Option<CoreAlgorithmStmtId>,
    source: CoreSourceRef,
    carried_core_diagnostic: Option<CoreDiagnosticId>,
}

enum ControlFlowDiagnosticKind {
    UnsupportedLocalDeclaration,
    IllegalBreak,
    IllegalContinue,
    Phase9Error,
    FlowDiagnostic,
}
```

`ControlFlowTerminator::Error` references a control-flow diagnostic row. For a
phase-9 error statement, that row carries the original `CoreDiagnosticId`.
Illegal `break` / `continue` and unsupported local declarations are produced
during Task 15 because CFG construction cannot represent them as ordinary valid
edges or locals. Broader flow diagnostics remain Task 17 work.

## Core-to-CFG Construction

Construction traverses `CoreAlgorithm.statements` in source order. A statement
builder returns exit contexts: `Normal`, `Return`, `Break(loop)`, and
`Continue(loop)`. Sequential composition continues only from `Normal`.

Rules:

- `Let` creates a local and, when an initializer exists, records a normal
  assignment effect. Unsupported declaration metadata is preserved on the local
  and reported with a construction diagnostic.
- `Pick` creates a site-local local with `Pick` metadata, records the witness
  type when present, classifies it as ghost/runtime, and records a
  non-emptiness obligation site for later handoff.
- `Assign` records a write to `CorePlace`; use-before-assignment and alias
  precision are diagnostics, not source-string guesses.
- `Assert` creates an assertion site and records the asserted formula as a
  normal successor-context fact for later VC generation.
- `If` creates a condition block, then/else blocks, and a deterministic join
  for normal exits. The absent branch is represented by the false path.
- `While` creates a loop header, body entry, normal exit, and back edge. It
  attaches invariants and decreasing terms to the loop record. `Break` exits
  join at the loop exit without gaining the negated loop condition.
- `Match` creates a deterministic switch over the scrutinee, one arm entry per
  source-order arm, and a join for normal arm exits. Pattern exhaustiveness and
  capture-variable binding metadata are external until checker payloads provide
  explicit pattern semantics; unsupported arms are diagnostics.
- `For` forms from specification chapter 20 are not present in the task-13
  `CoreAlgorithmStmtKind` surface. Until checker payload extraction provides
  explicit range/collection loop shells, phase 10 must not synthesize `for`
  semantics from source text. If a checker-owned fixture supplies a future
  `ForRange` or `ForEach` shell, it is represented as a loop record with hidden
  immutable range, step, collection, and exit values, a processed-set ghost
  local when present, finiteness or order-independence contract sites, and the
  same `Break`/`Continue` exit discipline as `While`. `ForRange` metadata must
  include the direction (`to` or `downto`), the hidden positive-`Nat` step value
  obligation, direction-specific `next(i)` expression, and `past_end(i_exit)`
  exit predicate required by specification 20.13.3.
- `Snapshot` and claim-related algorithm statements from specification chapter
  20 are not present in the current core statement shell. Until checker payload
  extraction provides explicit snapshot shells, phase 10 must not reconstruct
  snapshots from source text. A future `Snapshot` shell records a
  source-mapped snapshot site that captures the current `ProgramContextId`, all
  visible runtime and ghost locals, and any hidden loop values needed by claim
  blocks. Missing snapshot payloads are diagnostics, not silently erased.
- `Return` closes the current block with a return terminator and attaches
  postcondition sites.
- `Break` and `Continue` resolve to the innermost active loop. Outside a loop
  they are diagnostics and produce an error terminator.
- `Error` statements preserve the phase-9 diagnostic, create an error
  terminator or error statement site, and do not fabricate executable flow.

Unreachable statements after a terminating `Return`, `Break`, `Continue`, or
`Error` are represented with unreachable blocks and diagnostics rather than
being silently dropped.

## Diagnostics

Task 17 implements the flow diagnostics that can be decided from the current
phase-10 core surface:

- unreachable statement;
- use before definite assignment.

`UseBeforeAssignment` is checked for reachable statement-owned expressions:
`Let` initializer terms, assignment right-hand sides, `Assert` formulas, `If`
conditions, `While` conditions, `While` invariant formulas, `While` decreasing
terms, `Match` scrutinees, and `Return` values. `Pick` witness formulas are
checked for ambient variables while treating the picked binder as locally bound.
Algorithm-level `requires` / `ensures`, aggregate `CoreContractSet.assertions`
and `CoreContractSet.invariants`, and final obligation-seed metadata are not
checked by Task 17 because they need caller/result substitution or downstream VC
context that phase 10 does not own.

Use-before diagnostics have a concrete class
`UseBeforeAssignment { local, var }`. The diagnostic source is the source of the
term occurrence that mentions the uninitialized local, and `statement` is the
enclosing `CoreAlgorithmStmtId`. `UnreachableStatement { block }` uses the
unreachable statement source and the unreachable block id. Formula traversal
respects quantifier scoping: binders are processed left-to-right; a binder
shadows in its own guard and in later binders and the body, while earlier
binders shadow in later guards and the body. Unresolved/external variables are
ignored instead of guessed from source spelling.

Earlier phase-10 tasks already emit structural diagnostics for illegal `break`
or `continue` outside a loop, unsupported local-declaration metadata, and
malformed or missing algorithm statements carried from phase 9. The broader
diagnostic catalog is:

- illegal `break` or `continue` outside a loop;
- unsupported local-declaration metadata;
- unreachable statement;
- use before definite assignment;
- assignment to immutable parameter or const local;
- ghost value flowing into runtime state;
- unsupported call target or missing contract-instantiation payload;
- unsupported match/pattern payload;
- unsupported snapshot/claim payload;
- malformed or missing algorithm statement carried from phase 9;
- unsupported aliasing/lvalue metadata.

Assignment to immutable parameter/const locals, ghost leakage into runtime
state, call/contract instantiation errors, unsupported pattern payloads,
snapshot/claim payloads, and alias/lvalue precision require checker-owned
target/payload metadata that the current `CoreAlgorithmStmtKind` and `CorePlace`
surface do not expose. They remain deferred and must not be inferred from source
spelling.

Diagnostics are sorted by source order, then algorithm id, then block id, then
diagnostic class. A diagnostic may mark an algorithm as partial/error for
downstream consumers, but it must not pretend the algorithm is verified.

## Obligation Handoff Sites

Task 18 converts explicit `ControlFlowIr` metadata into deferred
flow-derived obligation seeds for the `core_ir.md` handoff view. The conversion
uses only CFG tables and core ids already present in `ControlFlowIr`; it never
walks source text and never invents checker-owned payloads.

Flow-derived handoff sites include:

- each entry `requires` site as a caller-side precondition seed;
- each return `ensures` site as a postcondition seed;
- algorithm-level and statement-level assertion sites;
- algorithm-level and loop-level invariant sites;
- algorithm and loop decreasing measure sites plus partial termination sites;
- ghost-only pick locals and ghost assignment effects as ghost-erasure safety
  seeds.

These seeds are `Deferred` in task 18. `mizar-vc` owns the exact VC context,
call/result substitution, well-foundedness goal schemas, ghost-erasure proof
shape, `VcId` assignment, and cross-edit `ObligationAnchor` construction. The
core handoff records source refs, core refs, CFG-local site refs, normalized
semantic origins, and anchor-ready program paths so those later phases do not
need raw source spelling.

## Determinism

For identical `CoreIr` input, construction must produce byte-stable debug
rendering and table order:

- algorithm order follows `CoreAlgorithmTable`;
- block order follows deterministic traversal;
- locals follow parameter/result/source statement order;
- loop ids follow header order;
- diagnostics follow the ordering above;
- maps are rendered in key order.

No hash-map iteration, filesystem order, or source spelling fallback may affect
semantic ids.

## Public Enum Policy

Task 21 classifies every `control_flow` public enum as a downstream
forward-compatible API surface. Each enum must remain `#[non_exhaustive]` so
future CFG, local-state, contract-site, termination, handoff, and diagnostic
categories can be added without breaking downstream exhaustive matches.

| Public enum | Decision |
|---|---|
| `ObligationHandoffOrigin` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ControlFlowObligationSiteKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ControlFlowTerminator` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `Reachability` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `LocalKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `LocalDeclaration` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `LocalMutability` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ContextFactKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `AssignmentEffectTarget` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ContractSiteKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ContractSitePlacement` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `AssertionPlacement` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `LoopInvariantPlacement` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `TerminationMeasurePlacement` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ControlFlowExitKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `GhostVisibility` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `TerminationSiteKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ControlFlowStatementPlacement` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ControlFlowDiagnosticKind` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module. Internal
`mizar-core` matches may remain exhaustive where they deliberately enumerate the
current variants.

## Task-32 Source-Derived Follow-ups

[source_family_decomposition.md](./source_family_decomposition.md) assigns
separate Core Tasks 48-53 to basic CFGs, range/collection loops, match,
snapshot/claim state, semantic attachment, and diagnostics. The first real CFG
task must add `SnapshotKind::ControlFlowIr` with its first real baseline in the
same commit. Phase 10 carries substitution-request metadata but never creates
or applies call/result substitutions or VCs.

## Validation And Tests

Task 15 tests must cover:

- deterministic block ordering;
- byte-stable control-flow debug rendering and key-ordered source-map output;
- straight-line let/assign/assert/return flow;
- `if` with and without `else`;
- `while` with `break`/`continue`;
- `match` arm ordering;
- local table contents: parameter/result/let/pick/hidden-local kinds, source
  order, mutability, ghost flags, initialization sites, and unsupported
  declaration metadata;
- structural diagnostic rows for unsupported local declarations and illegal
  `break` / `continue`, including source refs, error terminators, carried
  phase-9 diagnostic refs when present, and deterministic diagnostic ordering;
- source-map preservation for blocks, locals, and statement placements.

The current `CoreAlgorithmStmtKind` surface has no checker-owned payload that
requires a source-derived hidden loop local. Task 15 therefore keeps
`HiddenLoopValue` as an explicit representation, tests that current `While`
construction does not fabricate hidden locals, and defers source-derived hidden
locals for future `for`, snapshot, and termination payloads.

Task 16 tests must cover:

- `requires` and `ensures` placement;
- assertion facts and invariant sites in successor/header/exit contexts;
- decreasing measures at algorithm headers, loop headers, and loop `continue`
  edges;
- algorithm and loop partial sites when decreasing payloads are absent;
- exact source/provenance preservation for contract, call, assertion,
  invariant, decreasing, and statement-placement sites;
- empty call-site tables while checker-owned call payloads are unavailable;
- future range/collection `for` metadata only when explicit core shells exist;
- ghost-only state not appearing in runtime effect tables;
- runtime and ghost `Pick` distinction.

Task 17 tests must cover:

- unreachable statements;
- use before assignment;
- stable diagnostic ordering for newly emitted flow diagnostics;
- no assignment-to-immutable, ghost-to-runtime, call/contract, pattern,
  snapshot/claim, or alias/lvalue diagnostics are fabricated while their
  checker-owned payloads remain unavailable.

Task 18 tests must cover:

- existing core theorem, definition correctness, and checker-initial seeds are
  preserved in the handoff with labels, paths, source refs, provenance, and
  original-seed backreferences;
- flow-derived contract, termination, and ghost-erasure sites produce deferred
  seeds with formula/term refs where available, CFG-site backreferences, and
  deterministic handoff ordering;
- no `VcId`, `ObligationAnchor`, source-derived call/pattern/snapshot/claim
  payload, or concrete termination goal schema is fabricated.

Spec-only task 14 is verified by bilingual documentation review and diff
checks. Rust implementation and tests are deferred to tasks 15-18.
