# mizar-checker: Binding Environment

> Canonical language: English. Japanese companion:
> [../ja/binding_env.md](../ja/binding_env.md).

## Purpose

`binding_env` specifies the checker-owned binding and local-context layer used
between resolver output and type checking. It refines:

- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  Step 1, "Build the Type Checking Context";
- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  "Local Type Contexts Are Layered";
- [architecture 16](../../architecture/en/16.substitution_and_binding.md)
  binder identity, alpha-equivalence, capture avoidance, and definition-time
  closures;
- [`typed_ast.md`](./typed_ast.md) `LocalTypeContextTable`.

Task 4 is a specification-only task. It adds no Rust source, executable tests,
language semantics, type inference, registration activation, overload
selection, or proof behavior. Task 5 implements the data structures and context
builder described here.

## Boundary

`binding_env` owns:

- checker-local binding identities for local variables, binders, reserved
  variables, local abbreviations, and generated recovery bindings;
- deterministic layered context construction over `ResolvedAst` and
  resolver-owned `SymbolEnv`;
- local lookup order and shadowing rules used before type checking attaches
  normalized types;
- the bridge from resolver lexical scopes to `TypedAst` local context
  snapshots;
- binding/context diagnostics and deterministic debug rendering for task 5.

`binding_env` does not own:

- parser or resolver name lookup, label lookup, import/export validation, or
  symbol allocation;
- type-expression normalization or existence checks for attributed types;
- final type facts, coercions, registration closure, overload root selection,
  or inserted views;
- substitution execution, abbreviation expansion replay, VC generation, proof
  search, proof acceptance, or kernel replay;
- public diagnostic-code allocation while the checker diagnostic code-space is
  still an external planning gate.

## Inputs And Outputs

Task 5 constructs a `BindingEnv` from the resolver payload that is available
at the time of the task:

- one `ResolvedAst` source-shaped snapshot;
- its resolver-owned `SymbolEnv`;
- explicit local binding records supplied by resolver/source-walk payloads
  when those payloads exist;
- dependency module summaries as read-only inputs when available;
- checker configuration that controls recovery, but not semantic inference.

The current resolver surface exposes `LocalTermScope`, `LocalTermBinding`,
`NameRefEntry::resolution()`, definition shell binders, and `SymbolEnv`, but it
does not expose a complete AST-wide table of local binding declarations,
use-site scopes, use-site ordinals, reserve payloads, or captured-free-variable
payloads for closure replay. Task 5 must therefore implement the binding-env
data layer, validation, deterministic rendering, and module-level shell over the
available payloads. Missing local source-walk or closure payload is recorded as
an `external_dependency_gap` diagnostic instead of being reconstructed from raw
syntax.

The output is a checker-local snapshot:

```rust
struct BindingEnv {
    source_id: SourceId,
    module_id: ModuleId,
    contexts: BindingContextTable,
    bindings: BindingTable,
    diagnostics: BindingDiagnosticTable,
}
```

`BindingEnv` is not a serialized artifact. Later type-checking tasks consume it
to populate `TypedAst::contexts()` and to attach `BindingTypeRef` entries:

- global declarations and imported symbols are referenced by resolver
  `SymbolId`;
- local typed sites are mapped to `TypedSiteRef` only after the corresponding
  typed node or role exists;
- facts and assumptions are inserted by later type-checking tasks, not by the
  binding builder itself.

Task 5 must not add a direct `mizar-syntax` dependency or inspect
`ResolvedNode::kind()` to reverse-engineer binding constructs. Source-shape
roles needed for bindings must arrive through resolver-owned projections or be
reported as external dependency gaps.

## Context Graph

`BindingContextTable` is a deterministic forest rooted at the module context.
Each context is immutable after construction.

```rust
struct BindingContext {
    id: BindingContextId,
    owner: BindingContextOwner,
    parent: Option<BindingContextId>,
    layer: BindingContextLayer,
    lexical_scope: Option<LocalTermScope>,
    bindings: Vec<BindingId>,
    visible_bindings: Vec<BindingId>,
    recovery: BindingContextRecovery,
}

enum BindingContextLayer {
    Module,
    Declaration,
    Proof,
    Block,
    Expression,
}
```

Layer meanings follow architecture 04:

| Layer | Contains | Lifetime |
|---|---|---|
| `Module` | imported signatures, exported declarations, built-ins, top-level reserved variables | entire module |
| `Declaration` | declaration parameters, definition-local binders, declaration assumptions | current item |
| `Proof` | thesis-local binders, assumptions, proof-local declarations, label-related facts | current proof block |
| `Block` | `let`, `given`, `consider`, `reconsider`, statement-local bindings, local abbreviations | lexical block or statement frame |
| `Expression` | expected-type or expected-formula mode, coercion context, temporary generated binders | current expression/formula |

Required invariants:

- context ids are dense and deterministic for equivalent resolver inputs;
- `context#0` is the single module root context with `BindingContextOwner::Module`;
  every other context must have a parent;
- parent links form an acyclic chain;
- child contexts may read visible outer bindings but may write only to their
  own `bindings`;
- `visible_bindings` is sorted by deterministic `BindingId`; semantic lookup
  priority is computed during lookup from scope depth, visibility ordinal, and
  declaration range;
- leaving a context freezes only the bindings and later facts that are allowed
  to escape under the source construct that introduced them;
- recovered contexts are explicit and must not fabricate missing source
  binders.

## Binding Table

`BindingTable` stores local checker bindings. Resolver symbols remain in
`SymbolEnv`; they are not copied into this table unless a source construct
introduces a local checker binding.

```rust
struct BindingEntry {
    id: BindingId,
    spelling: String,
    kind: BindingKind,
    identity: BinderIdentity,
    owner_context: BindingContextId,
    declaration_range: SourceRange,
    visible_after_ordinal: usize,
    type_site: BindingTypeSite,
    status: BindingStatus,
    captured: CapturedFreeVariables,
    diagnostics: Vec<BindingDiagnosticId>,
    recovery: BindingRecoveryState,
}

enum BindingKind {
    QuantifierBinder,
    DefinitionParameter,
    LocalAbbreviation,
    ReservedVariable,
    LetBinding,
    Generated,
}
```

`spelling` is the source key used to prefilter candidate bindings and to render
diagnostics. Once candidates are selected, semantic equality, alpha-equivalence,
and capture checks use `BinderIdentity`.

`type_site` records where a later type-checking task should attach or discover
the binding's type. It may point to resolver syntax or to a future typed site,
but task 5 must not normalize that type.

`CapturedFreeVariables` is used for `set`, `deffunc`, and `defpred` closures.
Task 5 records the captured ids that the resolver exposes. If the resolver
does not yet expose enough closure payload, the builder records an
`external_dependency_gap` diagnostic and keeps the abbreviation non-expandable
instead of inventing captured variables.

## Binder Identity

Architecture 16 is authoritative: source display names are never enough to
decide equality, alpha-equivalence, or capture.

```rust
enum BinderIdentity {
    ResolverLocal {
        scope: LocalTermScope,
        ordinal: usize,
        declaration_range: SourceRange,
    },
    DefinitionShell {
        symbol: SymbolId,
        shell: ResolverShellId,
    },
    ReservedVariable {
        spelling: String,
        declaration_range: SourceRange,
    },
    Generated {
        context: BindingContextId,
        counter: u32,
    },
}
```

Required invariants:

- two source variables with the same display spelling but different scopes or
  ordinals have different identities;
- `LocalTermScope` is the resolver-owned lexical scope key exposed by
  `mizar-resolve`; task 5 may wrap it internally, but the boundary value must
  remain traceable to the resolver scope and visibility ordinal;
- shadowing creates a new binding id and never mutates the shadowed binding;
- generated identities are deterministic from the owning context and counter;
- alpha-equivalence and capture checks use `BinderIdentity`, not `spelling`;
- missing resolver identity payload is reported as an external dependency gap,
  not repaired by textual matching.

## Lookup Rules

Local lookup is deterministic:

1. Search only the active context's `visible_bindings` snapshot. That snapshot
   is the under-approximation boundary selected by the builder; lookup must not
   recover omitted ancestor bindings by walking parents.
2. Within that snapshot, first restrict candidates to bindings whose resolver
   local-binding key matches the use-site key. For source local terms, this
   includes the use-site spelling exposed by resolver scope data.
3. Among matching candidates, consider only bindings whose
   `visible_after_ordinal` is strictly before the use-site ordinal.
4. Partition visible bindings by semantic priority: deepest lexical scope
   containing the use-site scope, then greatest visibility ordinal, then source
   range.
5. If a same-spelling resolver-local candidate is visible but the use site does
   not carry enough lexical payload to compare its scope, do not select another
   textual candidate. Consume an extracted resolver `NameResolution` when one is
   available; otherwise return an `external_dependency_gap` missing-payload
   result.
6. If the best partition has more than one binding, return a degraded
   ambiguity result with an `AmbiguousLocalBinding` diagnostic draft and do not
   choose one arbitrarily.
7. Otherwise select the only binding in the best partition.
8. If no local binding matches and the use site has a resolver
   `NameRefEntry`, consume that entry's `NameResolution`.
   `BindingLookupSite` stores this already extracted `NameResolution`; the
   checker does not construct or persist resolver-owned `ReferenceSite` or
   `ResolvedNodeId` values.
9. Use `SymbolEnv` only to inspect `SymbolId`s already referenced by resolver
   outcomes. The checker must not call symbol indexes to redo or widen global
   name lookup.
10. If lexical payload is sufficient to decide that no visible local binding
   matches and no resolver outcome is supplied, return `Unresolved`.
11. If neither local binding payload nor a resolver name-reference outcome is
   available, return a degraded result carrying an `external_dependency_gap`
   diagnostic draft instead of fabricating a fallback.

Task 5 keeps lookup pure: `BindingEnv::lookup()` returns local, resolver,
ambiguous, forward-reference, missing-payload, or unresolved result states.
Ambiguity, forward-reference, and missing-payload results carry diagnostic
drafts. Builders or later semantic tasks record those drafts in
`BindingDiagnosticTable` when they materialize the affected site.

`BindingId` is never semantic lookup priority. It may be used only as a
deterministic storage, iteration, or rendering tie-breaker after ambiguity has
already been rejected.

The name/key filter is a lookup precondition, not semantic equality. After a
binding has been selected, equality, alpha-equivalence, and capture checks use
`BinderIdentity`; display spelling is diagnostic metadata.

These rules intentionally mirror the semantic part of the current resolver
local-binding ordering: scope depth, visibility ordinal, and declaration range.
Resolver spelling and stable-id order may inform deterministic storage order,
but they must not silently resolve semantic ambiguity.

Forward references are invalid for local bindings. A binding occurrence does
not resolve to itself while its declaration is still being parsed or typed.

## Reserved Variables

Top-level `reserve` declarations introduce `ReservedVariable` bindings in the
module context. They are visible only after the declaration ordinal and provide
default type sites for later occurrences of the reserved spelling.

Reserved-variable rules:

- task 5 records reserved bindings only from explicit resolver/source-walk
  payloads; current `SymbolEnv` does not expose reserve payloads;
- task 5 validation rejects `ReservedVariable` bindings owned by non-module
  contexts;
- nested `reserve` declarations are recovery cases until resolver/source
  support proves a narrower legal scope;
- a reserved variable is not a witness and does not create a type fact by
  itself;
- a local binder with the same spelling shadows the reserved variable through a
  distinct `BinderIdentity`;
- reserved type expressions are normalized by later type-checking tasks.

## Binder And Closure Rules

Quantifiers, `for`, `ex`, `given`, definition parameters, and source constructs
that introduce binders create `QuantifierBinder` or `DefinitionParameter`
entries. Their body contexts include those bindings and remove them from the
body's free-variable set for later substitution work.

Local abbreviations (`set`, `deffunc`, `defpred`) create
`LocalAbbreviation` entries with definition-time closure metadata:

- captured free variables are stored as `BinderIdentity`s;
- shadowing after the definition does not change the closure;
- expansion and capture-avoiding substitution are deferred to later semantic
  tasks, but task 5 must preserve enough identity metadata for them;
- if deterministic closure metadata cannot be collected, the abbreviation is
  retained only as degraded diagnostic state.

`binding_env` may compute and store normalized binder paths, but it must not
perform substitution replay. Replay remains the pure function specified by
architecture 16.

## Diagnostics And Recovery

`BindingDiagnosticTable` records checker-local diagnostics with stable
message keys. The id-order iterator preserves deterministic insertion order;
`canonical_iter()` renders and queries diagnostics sorted by source range,
class, message key, then id.

Required diagnostic classes:

- duplicate local binding in the same lexical scope;
- local binding used before it is visible;
- unsupported or ambiguous binding source shape;
- missing local binding table, use-site scope/ordinal, reserve payload, or
  closure payload from resolver/source-walk integration;
- missing resolver identity or closure payload;
- illegal nested `reserve`;
- recovered context boundary after malformed source.

Recovery must under-approximate. It is better to omit a binding and emit a
diagnostic than to invent an identity that could capture a different variable
or make a later proof obligation unsound.

## Deterministic Debug Rendering

Task 5 must provide deterministic binding-env debug rendering with a versioned
header:

```text
binding-env-debug-v1
```

The rendering must include module id, context graph, binding table, lookup
priority keys, diagnostics, and external dependency gaps in stable order. It
must not include memory addresses, host paths, hash-map iteration order, `VcId`,
proof witnesses, verifier status, or final overload information.

## Public Enum Policy

Task 31 applies the frontend task-25 public-enum decision procedure to this
module. All public checker-owned enums in `binding_env` are forward-compatible
API surfaces and must remain `#[non_exhaustive]`; downstream consumers must
keep wildcard or fallback arms. Checker-internal matches may remain exhaustive
over the currently represented variants when implementing the specified
behavior.

| enum | decision |
|---|---|
| `BindingContextOwner` | Forward-compatible; context owners may grow with richer source-to-checker extraction. |
| `BindingContextLayer` | Forward-compatible; context layer categories may grow with statement, proof, and definition scopes. |
| `BindingContextRecovery` | Forward-compatible; context recovery states may grow with partial binding recovery. |
| `BindingKind` | Forward-compatible; binding forms may grow as more Mizar declarations are extracted. |
| `BinderIdentity` | Forward-compatible; binder identity payloads may grow with closure and substitution evidence. |
| `BindingTypeSite` | Forward-compatible; binding type references may gain additional checker-owned anchors. |
| `BindingStatus` | Forward-compatible; binding status may grow with deferred/external dependency states. |
| `BindingRecoveryState` | Forward-compatible; binding recovery states may grow with richer resolver payloads. |
| `BindingDiagnosticClass` | Forward-compatible; diagnostic classes may grow before public checker diagnostic codes are allocated. |
| `BindingDiagnosticSeverity` | Forward-compatible; diagnostic severity policy may grow with IDE/artifact consumers. |
| `BindingDiagnosticRecovery` | Forward-compatible; diagnostic recovery states may grow with partial binding policy. |
| `BindingLookupResult` | Forward-compatible; lookup results may grow with additional ambiguity and external-gap handling. |
| `BindingEnvError` | Forward-compatible; binding-env construction errors may gain new validation cases. |

No exhaustive public enum exceptions are owned by this module.

## Planned Tests For Task 5

Task 5 must add Rust tests that cover:

- deterministic dense ids for contexts, bindings, diagnostics, and debug text;
- module, declaration, proof, block, and expression layer creation;
- lookup order across nested layers, including shadowing;
- fallback from local lookup to existing `NameRefEntry::resolution()` without
  redoing global `SymbolEnv` lookup;
- no forward local references before `visible_after_ordinal`;
- `reserve` declarations visible after their declaration and shadowed by local
  binders;
- binder identity equality independent of display spelling;
- duplicate same-scope binding diagnostics;
- recovered/unsupported binding shapes under-approximate rather than inventing
  bindings;
- definition-time closure metadata for exposed resolver payloads, plus
  external-gap diagnostics when the payload is missing;
- external-gap diagnostics and deterministic module-shell output when current
  resolver payload lacks local binding/use-site/reserve/closure extraction data;
- the public `module_shell(&ResolvedAst, &SymbolEnv)` signature and its
  syntax-free module-match seam;
- deterministic iteration and rendering;
- boundary guards that no binding-env data shape stores `VcId`, proof witness,
  verifier status, active registration state, final overload roots, or inserted
  overload-disambiguating `qua` views, resolver-owned `ReferenceSite` values, or
  resolver-owned `ResolvedNodeId` values.

No `.miz` checker-stage fixtures are required by task 5 because task-local
Rust tests cover its executable scope. Task 12 still owns the first active
`type_elaboration` corpus runner.

## Task 4 Classification

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | None blocking the task-4 specification. Architecture 04 gives the layered-context responsibility and architecture 16 gives binder/capture authority. | Continue with a docs-only spec task. |
| `test_gap` | No `binding_env` Rust tests exist yet because task 5 owns implementation. | This spec records the required task-5 tests; no executable test is added in task 4. |
| `design_drift` | Architecture 04 names a checker `TypeContext`, while `typed_ast.md` stores immutable `LocalTypeContextTable` snapshots. | This spec separates mutable/context-building `BindingEnv` from later `TypedAst` snapshots and defines the bridge. |
| `source_drift` | No `src/binding_env.rs` source exists yet. | Expected before task 5; no source repair belongs to task 4. |
| `external_dependency_gap` | Current resolver data exposes `LocalTermScope`, `LocalTermBinding` as a type, `NameRefEntry::resolution()`, definition shell binders, and `SymbolEnv`, but it does not expose a complete AST-wide local binding table, use-site scope/ordinal table, reserve payload, or captured-free-variable payload for full substitution replay. | Task 5 may implement the available binding-env data layer and module shell. Missing local extraction, reserve payload, closure payload, or binder payload must be recorded as external dependency gaps; do not add a direct `mizar-syntax` dependency or reconstruct bindings from raw syntax. |
| `deferred` | Type normalization, local type facts, registration activation, overload resolution, abbreviation expansion, substitution replay, and proof/VC behavior are outside task 4. | Keep task 4 and task 5 focused on binding/context construction only. |

## task 5 implementation classification

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | No task-5 blocking spec gap remains for the data layer, explicit-payload lookup, module shell, diagnostics, or deterministic rendering. | Continue to task 6 after task-5 review, verification, and commit. |
| `test_gap` | Task 5 adds Rust unit tests for context layers, lookup priority, forward-reference handling, reserved-variable shadowing, resolver-resolution fallback, closure identity metadata, diagnostics, deterministic ordering, module shell gaps, the public module-shell signature, and boundary guards. Active `.miz` checker-stage coverage still does not exist. | Rust tests cover the task-5 executable scope. A fully constructed `ResolvedAst` fixture remains external to checker until resolver exposes a syntax-free fixture; task 12 still owns active `type_elaboration` corpus coverage. |
| `design_drift` | Architecture 04 names a checker `TypeContext`; the implementation keeps this task as `BindingEnv` and later bridges into `TypedAst::contexts()`. | No code drift remains for task 5; keep the bridge deferred to type-checking tasks. |
| `source_drift` | `src/binding_env.rs` now exists and is exposed through the documented `binding_env` module. | Resolved for task 5. |
| `external_dependency_gap` | The resolver still does not expose a complete AST-wide local binding table, use-site scope/ordinal table, reserve payload, captured-free-variable payload, or syntax-free empty `ResolvedAst` fixture for checker-owned tests. | Task 5 records module-shell external-gap diagnostics, accepts explicit binding payloads when available, and type-checks the public module-shell signature without adding a direct `mizar-syntax` dependency. Later resolver/source-walk integration must provide the missing payload and fixture before full source extraction and closure replay. |
| `deferred` | Type normalization, local type facts, registration activation, overload resolution, abbreviation expansion, substitution replay, VC generation, proof acceptance, and kernel replay remain outside task 5. | Covered by later checker tasks and downstream crates. |
