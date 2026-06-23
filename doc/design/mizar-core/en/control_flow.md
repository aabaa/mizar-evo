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
contract, ghost, call, and termination facts remain empty or placed-only until
Tasks 16 and 17 attach their semantics.

Join points intersect definitely initialized sets and join path conditions
symbolically. If a later task cannot represent a precise join, it must keep a
conservative context and emit diagnostics for unsupported precision instead of
claiming facts that are not guaranteed on every path.

### Contracts and Assertions

```rust
struct ControlFlowContractSet {
    requires: Vec<ContractSite>,
    ensures: Vec<ContractSite>,
    calls: Vec<CallSite>,
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

`CallSite` records a resolved algorithm/functor call, its argument terms, the
instantiated `requires` formulas that later become caller-side obligations, and
the instantiated `ensures` facts that may become normal successor facts only
when termination is known. A call-site row therefore records termination
availability metadata such as `KnownTerminating`, `KnownPartial`, or
`UnknownTermination`, plus any checker-owned evidence reference. For partial or
unknown calls, `ensures` remains conditional metadata for later `mizar-vc`
obligation generation; phase 10 must not add it to the unconditional context.
The site is control-flow metadata only: phase 10 records the source-mapped
instantiation, while phase 11 assigns concrete `VcId`s. If the checker payload
does not identify a call target and contract instantiation, phase 10 must not
infer it from spelling; it records an unsupported-call diagnostic or leaves the
call-site table empty for explicit fixtures.

Task 14 only specifies placement. Task 16 attaches the concrete tables. Task 18
defines the final obligation-seed handoff.

### Ghost Effects

The ghost table records whether each local, place write, pick, assertion, and
contract site is runtime-visible or specification-only. A ghost value may feed
specification contexts (`requires`, `ensures`, `assert`, `invariant`,
`decreasing`) but must not flow into runtime assignments, runtime return values,
or non-ghost calls. Violations are flow diagnostics.

Ghost-only picks still create non-emptiness obligations, but their runtime
effect is erased by later extraction. Runtime picks are represented as local
`Pick` bindings; neither form emits a generated stable-choice origin.

### Termination

`ControlFlowTerminationPlan` records:

- header-level decreasing terms for recursive or mutually recursive
  algorithms;
- loop-level decreasing terms attached to loop headers;
- `continue` edges that must prove the measure has already decreased;
- sites that are partial because no decreasing measure is present.

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
- `Assert` creates an assertion site and adds the asserted formula to the
  normal successor context after the later VC is generated.
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

Task 17 implements flow diagnostics, but the spec fixes the diagnostic classes
and ordering:

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

Diagnostics are sorted by source order, then algorithm id, then block id, then
diagnostic class. A diagnostic may mark an algorithm as partial/error for
downstream consumers, but it must not pretend the algorithm is verified.

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
- call-site contract instantiation when checker-owned call payloads are
  available, or an explicit deferred/unsupported diagnostic fixture otherwise;
- conditional call-site `ensures` availability for partial or unknown
  termination;
- assertion and invariant sites;
- decreasing measures at headers and loop `continue` edges;
- exact source/provenance preservation for contract, call, assertion,
  invariant, decreasing, and statement-placement sites;
- future range/collection `for` metadata only when explicit core shells exist;
- ghost-only state not appearing in runtime effect tables;
- runtime and ghost `Pick` distinction.

Task 17 tests must cover:

- unreachable statements;
- use before assignment;
- immutable-local assignment;
- ghost-to-runtime leakage;
- unsupported call/contract payloads;
- unsupported match/pattern payloads;
- unsupported snapshot/claim payloads;
- malformed phase-9 algorithm statements;
- unsupported alias/lvalue metadata;
- stable diagnostic ordering.

Spec-only task 14 is verified by bilingual documentation review and diff
checks. Rust implementation and tests are deferred to tasks 15-17.
