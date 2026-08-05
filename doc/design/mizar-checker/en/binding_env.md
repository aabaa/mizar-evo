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

## Task 248 Source-Context Producer Integration

Task 248 supplies the first bounded real source walk without moving syntax
ownership into the checker. `mizar-test` matches one reserve shell and one
definition-block shell against the resolver's `DeclarationShellSet`, then
passes only opaque shell ids, ordered item/binding records, ranges, typed sites,
`LocalTermScope`, and `LocalTermBinding` to `SourceBindingContextProducer`.
The producer constructs one module context plus one declaration context,
retains distinct same-spelling reserve/parameter identities, and records the
parameter's structural shadow link to the visible reserve.

The complete transaction is retained in `SourceBindingContextHandoff` and
paired with `LocalTypeContextTable`. Unsupported visibility, stale or reordered
identity/provenance, duplicate or partial rows, and recovered shells that claim
bindings fail before publication. A supported recovered shell with no binding
produces an explicit empty recovered context and one internal diagnostic, but
remains incomplete and cannot enter `TypedAst`. This closes only the exact
Task-248 MC-G011/MC-G016 slice; term-use lookup and later proof/closure contexts
remain owned by Tasks 252/257/258/269/270/272.

## Task 257A Source-Formula Context Addendum

Task 257A adds `BindingContextOwner::SourceFormula { source_range }` for the
one syntax-free quantified-formula body context. `BindingEnv::try_new`
authenticates that the owner range is nonempty and belongs to the environment
source. The bounded producer extends the exact normal `1/0/4` module shell to
`2/1/4`: context 1 is an expression child of context 0, owns one
resolver-shaped `QuantifierBinder`, and exposes that binding as visible.
The context owner records source provenance only; it does not publish a
semantic formula result, accepted fact, or theorem owner.

## Task 258A Reserved-Theorem Consumer

Task 258A consumes one Task-48-derived normal module environment with context
0 and one active, visible `ReservedVariable` binding 0 for `x`. The
identifier declaration is `8..9` within reserve item `0..18`, the written
type site is `14..17`, and the theorem's first use has ordinal 1. The
statement context copies only the authenticated
visible binding ID and reserved-type-guard association; it does not mutate
the environment, add a theorem/proof context, publish a fact, or extend this
module's public API.

## Task 258B1 Source-Statement Proof Contexts

Task 258B1 adds the exact non-exhaustive owner variant:

```rust
BindingContextOwner::SourceStatement { source_range: SourceRange }
```

`BindingEnv::try_new` requires every such range to be nonempty and to belong
to the environment source. Deterministic debug renders it exactly as
`source-statement(<start>..<end>)`; every existing owner and pre-B1
environment retains byte-identical debug.

The only admitted Task-258B1 environment is `3/1/0`. Context 0 is the
unchanged normal module context, has no lexical scope, owns reserved binding
0, and exposes `[0]`. Context 1 is the normal proof child of context 0,
owner range `69..137`, lexical scope `[0]`, owns no binding, and exposes
`[0]`. Context 2 is the normal proof child of context 1, owner range
`86..113`, lexical scope `[0,0]`, owns no binding, and exposes `[0]`.
Binding 0 and every Task-48 reserve identity/range/type/visibility field
remain unchanged.

The variant records source topology only. It adds no proof-local binding,
capture, substitution, fact, diagnostic, goal, or proof meaning. The
Task-258B1 checker matrix covers exact owner/range/scope/debug, wrong-source
and empty ranges, parent/layer/scope/visibility mutation, fingerprint
propagation into Tasks 252/258, rollback, and Task-258A byte compatibility.
This documentation prerequisite changes no source or test.

### Task 258B1 Implementation Status

The frozen `BindingContextOwner::SourceStatement { source_range }` variant is
implemented. Exact outer/nested proof contexts at `69..137` and `86..113`
extend the module base to `3/1/0`, preserve the reserved binding, scope paths
`[0]` and `[0, 0]`, and deterministic debug/fingerprint bytes. Invalid empty
or foreign ranges and profile substitutions fail before publication.
Task-258A retains its original one-context bytes.

### Task 258B2 Frozen Assumption-Context Extension

Task 258B2 is limited to the exact 113-byte source frozen in
`source_statement.md`: one module reserve and one theorem whose proof contains
the unlabeled assumption `assume x = x;` followed by `thus x = x;`. Its
Task-48 environment profile is exactly `2/1/0`. Context 0 is the unchanged
module context and owns reserved binding 0. Context 1 is its normal proof
child, has owner `SourceStatement { source_range: 72..111 }`, lexical scope
`[0]`, owns no binding, and exposes `[0]`. There is no nested proof context.

The extension records source topology only. It does not turn the assumption
into a binding, premise, fact, checked formula, goal, accepted theorem, or
proof result. Empty/foreign ranges, a non-proof owner, a different parent or
scope, another binding count, or any profile other than `2/1/0` must fail
before publication. The documentation prerequisite changes no source or test;
the existing Task-258A and Task-258B1 profiles and bytes remain unchanged.

### Task 258B2 Implementation Closure

The implementation reuses the existing `SourceStatement` owner exactly:
module context 0 plus proof context 1 at `72..111`, parent 0, proof layer,
scope `[0]`, no local bindings, and visible reserved binding `[0]`.
`binding_env.rs` remains byte-for-byte outside this task; mutation tests
authenticate the exact `2/1/0` fingerprint and reject cross-profile lower
environments atomically.

## Task 258B3 Frozen Proof Context

The witness profile reuses the public Task-48 model without changing
`binding_env.rs`. It requires module context 0 plus proof context 1 owned by
source range `69..102`, parent 0, proof layer, scope `[0]`, no local binding,
visible binding `[0]`, and normal recovery. Reserved binding 0 remains `x`
at `8..9` with source type site `14..17`.

Task-252 terms use contexts `0,0,1,1,1`; the witness row stores direct
`BindingContextId(1)`, never a theorem/conclusion
`SourceStatementContextId`. Witness validation reauthenticates binding 0,
scope `[0]`, and stored use ordinal 1 through primary term/reference 2.
Foreign context, scope, binding, or B1/B2 binding fingerprint fails as a
dependency before witness-row validation.

The implemented B3 route constructs exactly this two-context environment and
the checker revalidates the direct proof context through witness primary term
2. No binding row or binding-environment API changed.

## Task 258B3N Named-Witness Boundary

B3N retains exactly two contexts, one reserved binding, and no diagnostic.
Proof context 1 covers `68..105`, has lexical scope `[0]`, an empty owned
binding list, and visible binding `[0]`. Named token `y` is transported by
the new witness-name table and is not a `BindingId`. Task 269 alone retains
the later local binding, RHS link, capture-by-resolved-binding abbreviation
replay, and context transition. Task 270 remains limited to
`deffunc`/`defpred` closure.

## Task 258B3N Implementation Result

The implemented route revalidates exactly two contexts and the sole reserved
binding. The `y` token exists only in `SourceStatementWitnessNameTable`; no
new `BindingId`, owned binding, visible binding, capture, or context
transition is published. Task 269 retains those effects.

## Task 258B3M1 Mixed-Witness Boundary

B3M1 keeps exactly module/proof contexts `0/1`, reserved binding 0, and no
diagnostic. Both witness primary terms resolve to binding 0 in proof scope
`[0]`; token `y` remains only witness-name syntax and never enters the
binding environment. The second unnamed row creates no binding either.
Task 269 retains the future `y` binding, RHS link, abbreviation replay, and
context transition. No binding API or fingerprint grammar changes.

## Task 258B3M1 Implementation Result

The implementation keeps exactly module/proof contexts `0/1`, reserved
binding 0, visible `[0]`, and no diagnostic. Token `y` and the second
unnamed witness create no binding or resolver-owned symbol. Task 269 still
owns future witness-name binding and abbreviation replay.

## Task 258B3M2A Unnamed-Numeral Boundary

B3M2A retains exactly module/proof contexts `0/1`, reserved binding 0,
visible scope `[0]`, and no diagnostic. Numeral primary term 2 has no
reference row and witness row 0 has no name row, owned binding, or
resolver symbol. It creates no binding, abbreviation, capture, or context
transition. Task 269 therefore receives no B3M2A work; Task 272 retains
later witness typing and existential effects. No binding API or fingerprint
grammar changes.

## Task 258B3M2A Implementation Result

The implemented profile revalidates exactly module/proof contexts `0/1`,
reserved binding 0, visible `[0]`, and zero diagnostics. Numeral term 2 and
its unnamed witness create no reference, binding, resolver-owned symbol,
capture, abbreviation, or context transition. Task 269 therefore remains a
no-op for this slice, and no binding API or fingerprint changed.

## Task 258B3M2B1 Frozen Binding Boundary

The exact parenthesized witness reuses module/proof contexts `0/1`,
reserved binding 0, proof lexical scope `[0]`, and zero diagnostics. Outer
term 2 has no reference; only child term 3 resolves binding 0. Dense
reference IDs `0/1/2/3/4` target terms `0/1/3/4/5`, all with use ordinal
1 and scopes `[]/[]/[0]/[0]/[0]`. The unnamed witness introduces no
binding, capture, abbreviation, resolver symbol, or context transition, so
Task 269 remains a no-op. Task 272 retains witness typing, existential
matching/substitution, remaining-goal, and proof effects. No binding API or
fingerprint grammar changes.

## Task 258B3M2B1 Implementation Result

The implemented profile revalidates exactly module/proof contexts `0/1`,
reserved binding 0, visible proof scope `[0]`, and zero diagnostics. Outer
parenthesized term 2 creates no reference; child term 3 alone resolves
binding 0 at use ordinal 1. The unnamed witness creates no binding,
resolver-owned symbol, capture, abbreviation, or context transition. Task
269 therefore remains a no-op, and no binding API or fingerprint changed.

## Task 258B3M2B2A Frozen Binding Boundary

The nested-parentheses prerequisite reuses exactly module/proof contexts
`0/1`, reserved binding 0, proof scope `[0]`, and zero diagnostics. Outer
wrapper term 2 and inner wrapper term 3 create no references; only variable
term 4 resolves binding 0 at use ordinal 1. The unnamed outer witness adds
no binding, capture, abbreviation, symbol, or context transition. Task 269
remains a no-op; no binding API, table, or fingerprint may change.

## Task 258B3M2B2A Implementation Result

The implemented profile reuses the frozen `2/1/0` environment byte-for-byte:
module context 0, proof context 1 over `82..119`, one reserved binding 0,
proof scope `[0]`, and no diagnostic. Only leaf term 4 has the proof-local
reference; both wrappers remain reference-free, and the unnamed witness adds
no binding or symbol. No binding source or public API changed.

## Task 258B5A Frozen Nested-Proof Binding Boundary

The private route reuses one reserved binding and exactly four normal
contexts with zero diagnostics. Context 0 is the module. Context 1 is outer
proof `87..183`, parent 0, lexical scope `[0]`; context 2 is first descendant
proof `104..131`, parent 1, scope `[0,0]`; context 3 is second descendant
proof `146..178`, parent 1, scope `[0,1]`. Every context exposes only binding
0.

Task-252 term-reference contexts are
`0,0,1,1,2,2,1,1,3,3`; all ten uses select binding 0 at producer-stored use
ordinal 1. Proof labels are resolver provenance, not BindingEnv bindings.
B5A changes no binding producer, row, fingerprint, scope rule, diagnostic,
source file, or public API.

## Task 258B5B Frozen Import-Proof Binding Boundary

The imported-citation route reuses one reserved binding and exactly two
normal contexts with zero diagnostics. Context 0 is the module; context 1 is
proof `114..144`, parent 0, lexical scope `[0]`. Both expose only binding 0.
Task-252 term-reference contexts are `0,0,1,1`; all four uses select binding
0 at the existing producer-stored use ordinal.

The imported theorem `Ref` is resolver label provenance, not a BindingEnv
binding or statement fact. The separate import-summary prerequisite and the
later upper implementation change no binding source, row, fingerprint,
scope rule, diagnostic, or BindingEnv API.

## Task 258B5C Frozen Unresolved-Label Binding Boundary

The two B5C negatives stop in resolver declaration-symbol handling before
checker binding transport. Their raw resolver environments each remain
`1/0/1/1/0`; that count is not a BindingEnv profile. R-032A first provides
the validated structural arena, and R-032B derives proof scopes `[0]`,
`[0,0]`, and `[0,1]` and returns one
`UnresolvedLabelRef`, while no `BindingContextId`, `BindingId`, visible
binding, statement fact, or checker binding fingerprint is constructed.

The active runner must consume this resolver-owned failure and may not
synthesize a binding context to route it through the checker. B5C therefore
changes no BindingEnv source, row, public API, diagnostic, scope rule, or
test. Tasks 252/253 and B1/B5A/B5B remain exact and disjoint.

R-032B's module-global one-based statement counter, completion maximum, and
canonical `proof-step-v1` origin are resolver label data, never BindingEnv
ordinals or fingerprints. Source-byte runner selection and private
`proof_scope_input`/`proof_scope_confinement` details likewise add no
BindingEnv consumer. The current documentation transaction spans 48 design
files only.

The R-032B default-deny edge table begins with exact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock`: Root/CompilationUnit each require exact-one normal structural
children, and ItemList scans only direct normal theorem children. It assigns
no ordinal and performs no descent for other item children,
formula/token/wrapper, unsupported/recovered/malformed,
qualified/grouped/bulk, or template forms; none can create a BindingEnv
context. Likewise, env/module, projection namespace/module/contribution, and
exact id-0 LocalSource record/source-id mutations are runner input
authentication only. Their sole `proof_scope_input` output does not enter
this crate.

## Task 248 Two-Parameter Profile-B Binding Boundary

Profile B adds no binding API. The existing producer receives one normal
definition item and two ordered `DefinitionParameter` inputs, then publishes
binding ids 0/1 in definition context 1. Module context 0 is empty;
definition context 1 has parent 0, scope `[0]`, bindings and visible bindings
`[0,1]`. Both rows are active resolver-local identities with ordinals 0/1,
exact declaration/type ranges, empty captures, and no shadow. Declaration 1's
predecessor 0 records source order only.

Same-scope duplicates, recovery, a reserve hybrid, extra items/bindings, and
stale scope/ordinal/range/site provenance remain fail-closed. Profile A's
reserve/local shadow and recovered-empty branch are unchanged. Task 259 may
only consume this handoff and may not reconstruct it.

## Task 248P Property Context Binding Reuse

Profile C adds no `BindingEnv` enum or constructor. One normal property
parameter reuses `BindingKind::DefinitionParameter`, resolver-local identity,
active status, source written-type site, empty capture/diagnostic lists, and a
property-shell-owned declaration context whose parent is the empty module
context. Its visible/binding lists both contain only binding zero and it has no
shadow. The recovered zero-binding branch reuses the existing recovered
context plus one recovery diagnostic. Profile A/B rows and error precedence
remain byte-identical; Task 264 may consume this context but cannot reconstruct
or add property semantics inside `BindingEnv`.

## Task 248P Active Property Context Binding

The implemented Profile C now publishes exactly the frozen active
definition-parameter binding and declaration context, or the recovered empty
context plus one recovery diagnostic. No `BindingEnv` type, constructor,
lookup, or semantic payload changed, and Profiles A/B retain their prior bytes
and validation precedence.

## Task 269A Frozen Named-Witness Binding Transition

The exact Task-258B3N base environment `2/1/0` is immutable input. Task 269A
reconstructs context 1 with `bindings=[1]` and `visible_bindings=[0,1]`, then
appends exact `y` binding 1 as `LocalAbbreviation` with resolver-local scope
`[0]`, declaration `81..82`, visible-after 1, missing type site, active status,
empty captures/diagnostics, and normal recovery. Context 0 and binding 0 remain
byte-identical; diagnostics remain empty.

Lookup at ordinal 1 cannot see `y`; a later same-scope lookup sees binding 1.
This transition records definition-site identity only. Later-use/capture replay
is Task 269B+, and Task 272 retains witness typing and goal effects.

## Task 269A Active Named-Witness Binding Transition

The implemented producer reconstructs exactly `2/2/0` and replays both
ordinal lookups during construction and installation. Checker and runner
corruption matrices confirm that local provenance, row links, all 51 nodes,
and final fingerprints fail closed. A cfg(test)-only mutable-row seam exercises
installed spelling/scope/range/ordinal precedence without changing the
production API. No later-use/capture or typing behavior is added.

## Task 269B frozen B3M1 transition

The same exact transition is admitted for Task-258B3M1 with resolver-local
`y`, scope `[0]`, range `84..85`, and visible-after 1. Only named witness 0 is
associated with binding 1; unnamed witness 1 does not create a binding. The
base/final profile stays `2/1/0 -> 2/2/0`, definition-site lookup is forward,
and later same-scope lookup resolves binding 1. Later-use/capture execution and
all type/goal/proof effects remain deferred.

## Task 269B active B3M1 transition

The implemented transition reproduces exactly `2/1/0 -> 2/2/0` at range
`84..85`. Context 1 owns only binding 1 and sees `[0,1]`; binding 2 does not
exist, proving the unnamed sibling has no binding effect. Ordinal-1 lookup is
forward and ordinal-2 lookup resolves binding 1. The row retains
`BindingTypeSite::Missing`; no type is inferred, captures remain empty, and
facts, obligations, and proof/goal effects remain absent.

## Task 269CP no-binding lower boundary

The isolated proof-`let` prerequisite authenticates a resolver-shaped local
`y@[0],71..72,visible-after=1` but deliberately does not mutate a
`BindingEnv`, allocate `BindingKind::LetBinding`, choose a type site, or
publish a proof/block context. Those checker-owned decisions remain Task
269C. Treating the private lower projection as an active binding would be a
boundary violation.

## Task 269C frozen `LetBinding` transaction

The checker consumes the Task-269CP syntax-free projection plus the existing
reserve bridge's exact base `1/1/0` environment. It validates reserved `x`
binding 0 and appends proof context 1 plus binding 1 only. The result is exact
`2/2/0`: `y`, `LetBinding`, resolver-local scope `[0]`, range `71..72`,
visible-after 1, missing type site, active, uncaptured, diagnostic-free, and
normal. Context 1 is `SourceStatement(59..98)`, proof-layer, parent 0, owned
`[1]`, visible `[0,1]`. Definition-site ordinal 1 remains forward; synthetic
ordinal 2 resolves binding 1. No actual use/capture row or source type is
claimed. Every base/final row and debug fingerprint fails closed.

## Task 269C Active `LetBinding` Transaction

The implemented producer and both installers now enforce this exact
transition and both lookup oracles. Corruption of either environment, context,
binding, declaration link, or fingerprint fails transactionally; the final
binding still has a missing type and no real use/capture or semantic effect.

## Task 269CT Typed Binding Overlay

Task 269C remains the immutable missing-type dependency. Task 269CT constructs
a separate exact `2/2/0` typed overlay: binding 0 remains source `14..17` and
only proof-local `LetBinding` 1 becomes source `76..79`. Contexts, identities,
lookup, captures, diagnostics, and all non-type fields are unchanged. The
overlay is accessible only through the new composite; no use/capture, guard,
fact, goal, proof, or obligation is published.

## Task 269CT Implemented Overlay

The implementation reconstructs the exact dependency environment without
sorting or repair. Contexts, both bindings, identities, lookup fields,
captures, diagnostics, and recovery compare equal to Task 269C except solely
for binding 1's `Missing -> Source(76..79)` type-site overlay. Validation and
tests authenticate the complete `2/2/0` payload and fail binding corruption
before source-type or availability errors.

## Task 269GP No-Binding Lower Boundary

The private lower output carries only the `y` token spelling/range and no
resolver-shaped local identity; it neither creates a `BindingEnv` row nor
performs lookup. The Chapter-4/16 `given` scope contradiction blocks future
Task 269G/269GT pending human canonical reconciliation.

Implemented 269GP preserves this boundary; focused tests reject every
binding-shaped publication and leave the existing binding environment APIs
unchanged.

## Task 269GS Resolved Scope Input

Canonical scope is now sufficient for a later binding consumer: the `given`
binding covers its declaration's `such that` conditions and subsequent
visibility ends with the innermost enclosing proof or reasoning block, is
inherited by nested children unless shadowed, and is absent from parent and
sibling blocks. Task 269GS does not modify
`BindingEnv`; Task 269G must separately freeze exact scope IDs, ordinals,
lookup/replay, restoration, and tests without adding condition or proof facts.

## Task 269G Frozen `GivenWitness` Environment

Task 269G adds `BindingKind::GivenWitness` immediately after `LetBinding` and
before `Generated`, with stable debug key `given_witness`, and one exact proof-context binding
at context/binding `1/1`, scope `[0]`, source/visible-after ordinal `1/1`, and
missing type. Lookups prove forward-before, same-condition/later/child
visibility, parent/sibling exclusion, child shadowing, and outer restoration.
Only real contexts 0/1 enter the handoff. Test-derived contexts 2/3/4 are
normal `Block` rows with owner keys `task269g-unshadowed-child`,
`task269g-shadow-child`, and `task269g-sibling`; binding 2 is the exact
test-only `y`/`GivenWitness` shadow at scope `[0,1]`, ordinal/range
`2/109..110`, owner 3, missing type, active status, and empty capture/
diagnostics. No condition, fact, capture, type, or proof row is created. Task 269GT
owns the missing source type.

## Task 269G Active `GivenWitness` Transaction

The producer and both installers now enforce the frozen transition and the
exact declaration-context forward/local lookups. A separate checker-only
synthetic matrix proves that the declared witness remains visible through its
corresponding block and nested children unless shadowed, is restored after a
shadow ends, and is absent from parent and sibling blocks; those synthetic
contexts are not production handoff rows. Environment, declaration, binding,
and fingerprint corruption fails transactionally. The installed row keeps a
missing type and creates no condition, fact, capture, obligation, or proof
effect.

## Task 269GT Frozen Type Overlay

The new composite must preserve the Task-269G environment byte-for-byte except
for binding 1's `BindingTypeSite::Missing -> Source(84..87)` overlay. Binding
0 stays `Source(14..17)`; contexts, identities, lookup, scope, status, capture,
diagnostics, and `2/2/0` cardinality do not change. The composite owns this
typed snapshot without mutating the immutable Task-269G dependency.

### Task 269GT implemented overlay

The immutable Task-269G environment remains `2/2/0` with the Given row missing its type. Task 269GT copies it without sorting or repair and changes only binding 1 to `Source(84..87)`; binding 0, both contexts, resolver identity, block-local inheritance/shadowing/restoration behavior, capture, diagnostics, and all other fields remain exact.

## Task 269GUP New-source Binding Profile

The sibling transaction independently builds `1/1/0 -> 2/2/0`. Its context 1
owner is `62..126`; binding 1 is `GivenWitness`, scope `[0]`, ordinal 1,
range `76..77`, and type `Missing`. Ordinal-2 lookup selects this environment's
own `BindingId(1)`. Capture/diagnostics stay empty; parent/sibling exclusion
and child inheritance/shadow restoration are the required test matrix.
### Task 269GUP implemented binding profile

The frozen six-file transaction and its exact four checker/four runner tests are implemented. Libraries measure `502/564`; checker/runner production is `30/172531` and `37/74826`, with unchanged path hashes and content hashes `e0342952a01a0b379cf7b06ad243cd40a1656e940480196323cf43fbe7d8f7c5` / `8fe7c8c0b7e855e5113f3830873e133f42c8048a3272055e2fddd5ebd9cbb1bc`.

This closes only dormant private lexical-binding evidence and grants zero active corpus, trace, type, term/use, condition/fact, goal/proof, obligation, diagnostic, or CLI credit. Task 269GUPT is next; Task 269GU, capture, and Task 270 remain deferred.

## Task 269GUPT Frozen Binding Overlay Boundary

GUPT consumes the immutable GUP handoff and constructs a copy of its exact `2/2/0` environment. Contexts, identities, lookup lifetime, capture, diagnostics, and every non-type field are unchanged; only copied binding 1 changes from `Missing` to `Source(84..87)`, while binding 0 stays `Source(14..17)`. The GUP dependency and old G/GT environments are never mutated. No occurrence, guard, fact, capture, or semantic binding state is added.

### Task 269GUPT implemented overlay

The exact copied overlay is implemented and its corruption, replay, ownership,
and same-identity cross-family matrix passes. The immutable GUP dependency and
all non-type binding state remain unchanged; no semantic binding credit is
claimed.

## Task 269GU Frozen Lookup Consumer

GU reuses the immutable GUPT `2/2/0` environment. Both later `y` rows query
context 1, lexical scope `[0]`, and derived use ordinal 2 and must resolve to
`BindingId(1)` of kind `GivenWitness`. A private exact source-term profile
admits that winner as `Variable`; generic role admission, contexts, capture,
diagnostics, binding types, scope lifetime, and all GUP/GUPT bytes are
unchanged. Parent/sibling visibility remains excluded.

### Task 269GU implemented binding use

The term producer consumes the immutable GUPT environment and authenticates
both later references to binding 1 at use ordinal 2. It adds no binding row or
context, and does not yet transport condition/descendant occurrences, capture,
or export behavior.

## Task 269GCP No-binding Lower Boundary

The exact condition source proves only that a future binding consumer is
required. GCP creates no context, `BindingId`, lookup, captured identity, or
diagnostic and leaves GUP/GUPT/GU byte-identical. Task 269GC must construct a
new exact environment in which the witness is available to condition
occurrences without weakening generic source-order lookup.

### Task 269GCP implemented no-binding boundary

The implemented lower row retains the witness declaration sites but creates no
binding context, ID, lookup, lifetime, or diagnostic. GUP/GUPT/GU remain
byte-identical; the user-confirmed innermost-block lifetime is still owned by
the distinct next Task 269GC binding handoff.

## Task 269GC Frozen Binding Environment

GC reuses unchanged `BindingKind::GivenWitness` and common Given row types; it
adds no `binding_env.rs` source. The authenticated reserve base changes only
`1/1/0 -> 2/2/0`, adding proof context `SourceStatement(68..132)` at scope
`[0]` and one normal active missing-type witness `y@82..83`, visible after
ordinal 1. Lookup freezes own-condition and subsequent visibility, descendant
inheritance, shadow/restoration, and parent/sibling exclusion. No occurrence,
fact, condition lifetime, capture, diagnostic, or type row is created.

### Task 269GC implemented binding environment

The exact `1/1/0 -> 2/2/0` transaction and complete lexical lookup matrix are
implemented in the frozen checker owner without changing `binding_env.rs`.
All four checker and four runner tests pass the own-condition, subsequent,
inheritance, shadow/restoration, and parent/sibling boundaries. Type,
occurrence, fact, capture, diagnostic, and semantic-credit ownership remains
deferred exactly as frozen.

## Task 269GCT Frozen Type Overlay

GCT clones the validated GC `2/2/0` environment and changes only binding 1's
type site from `Missing` to `Source(90..93)`. Contexts, visible-binding order,
lookup behavior, binder identity/kind/status, declaration range, ordinal,
scope, capture set, recovery, and empty diagnostics remain byte-for-byte
equivalent; binding 0 retains `Source(14..17)`. No new binding or context is
inserted. GCT adds no `binding_env.rs` source and grants no fact, condition,
guard, capture, or obligation semantics.

### Task 269GCT implementation status

After documentation prerequisite `b43081161b31fcc4bc23ac2fd42c5c42e772ab78`,
the exact seven-file implementation and four checker/four private runner tests
are present. The new public checker family is
`SourceProofLocalGivenConditionType{Handoff,Producer,Error}`; Typed and
Resolved own the same boxed composite atomically. Libraries are `518/584`.
Checker production is `30/179612`, with unchanged path hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`
and content hash
`8078ee6235c8ca52ce8cdba0be9a347231260d3421c54625a3fc96cf395c9718`.
Runner production is `37/77159`, with unchanged path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
and content hash
`5b0e68f35d37fcf843f7cb64885f09bfa9dd5423c17506713e096811a5ddf689`.
Raw/normalized test-list hashes are checker
`6d10b524115a209f198bc5085a726bc1fcc6f92dc3e25a8056e29975b708b656` /
`502f7535a34b9d2224c67e6db15f4eaf45f05eec2a2fe4c914704ecf162d89b2`
and runner
`d599bd69654d000f44858942cec771742d8c3c9e0d2ca459d7fecc84d76752c9` /
`bc3cdabbc6424b0f01d817ed323dd823ff57d1d8d4261220dc3d9c37d9004a61`.

The implementation changes no canonical specification, `.miz`, fixture,
sidecar, expectation, trace row/status/backlink, metadata, diagnostic, public
dispatch, CLI byte, active result, or semantic credit. GCU still owns both
condition occurrences and every wider semantic effect. Independent test-sufficiency, implementation, source/documentation, and
final-quality reviews report **NO FINDINGS**. All nine hard gates PASS with no
score cap at `100/100`; focused and crate suites, lint policies, formatting,
Clippy, workspace tests, metadata, all five CLIs, count/hash oracles, and diff
checks pass. Dedicated implementation commit
`d6fb0ed28ced4d4706a1793b3aedd2a20eea0749` is complete.

## Task 269GCU Frozen Reference Lookup

GCU does not mutate the GCT `2/2/0` environment. Both term rows use context 1
and producer-derived use ordinal 2; lookup must return binding 1 uniquely as
the block-local `GivenWitness`, scope `[0]`, declaration `82..83`, type site
`Source(90..93)`. The private GCU primary-term profile alone admits that
binding kind as a variable reference. No context, binding, scope, capture,
recovery, diagnostic, or environment fingerprint changes.

### Task 269GCU implementation status

After documentation prerequisite `15f47a837bc2f52d4cd30e8a4dcb86c16f2961d3`,
the seven frozen implementation files, one `cfg(test)`-only predecessor
ownership-sentinel support file, and four checker/four private runner tests are
present. The support seam closes the review-discovered Task-269A both-order
`test_gap` without changing production API or behavior. The public family is
`SourceProofLocalGivenConditionUseTerm{Handoff,Producer,Error}`; Typed and
Resolved own the same boxed composite atomically. Libraries are `522/588`.
Checker production is `30/181154`, with unchanged path hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`
and content hash
`f9901821c2242bfe66321c57982b54b78425c7940c5a7c47c93c43a8c2c035dc`.
Runner production is `37/77435`, with unchanged path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
and content hash
`0651af8339c147d04f88be237f8f49fc716b7da3ff90238be50a9527e89992b7`.
Raw/normalized test-list hashes are checker
`d453ca1e8a7cf9870f14a0f933451ca201c19cc8c8367d51767c40a941766f82` /
`7cd84f6cd8e6d1070b39be9e5f1031512cc2c1b664829f10d337f1b67bcb74b3`
and runner
`7a99bcbb35838b6c1df31dec7b7c70d9c569df86bdc6f5c68d72f41578be2a9e` /
`e49dac17564f330ad5c73018538bf5736720e47f4833709c1b9d36622208888a`.

The implementation closes only the two frozen own-condition `y` term/reference
occurrences. The authoritative block-scope decision makes a `given` binding
visible through the remainder of its innermost block and descendant blocks,
subject to inner shadowing, but descendant-use/capture implementation remains
a separate successor. No canonical specification, `.miz`, fixture, sidecar,
expectation, trace row/status/backlink, metadata, diagnostic, public dispatch,
CLI byte, active result, or semantic credit changed. Equality/formula/fact,
guard, goal, proof/obligation/acceptance, export/capture enforcement,
downstream IR, and Task 270 remain deferred. Independent test-sufficiency,
implementation, and source/documentation reviews report **NO FINDINGS**.
Final read-only quality reports **NO FINDINGS**: all nine hard gates PASS
without a score cap at `100/100`. Focused and full measured gates pass.
Exact staging and implementation commit f984ae683419944493c07723e9950a9101a46502 are complete.

## Task 269SDP Binding Deferral

SDP installs no `BindingEnv` and derives no local winner from spelling. The
outer Given row, descendant child context, inherited visibility, descendant
`y` reference, and `z`/`q` LocalAbbreviation rows are all absent. A separate
next consumer owns only Given-plus-child context; capture remains blocked by
the canonical `set` `spec_gap`.

## Task 269SDP Implementation Status

Documentation prerequisite `f468b0163bb00726dca9b356f48790c73bb1fe98` is
complete. The frozen four Rust files and four exact tests now implement only
the dormant lower projection. Focused `4/4` and runner-library `592/592`
tests pass; independent test-sufficiency and implementation reviews report
**NO FINDINGS**.

Checker/runner libraries are `522/592`. Checker production remains
`30/181154`, path/content SHA-256
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`f9901821c2242bfe66321c57982b54b78425c7940c5a7c47c93c43a8c2c035dc`.
Runner production is `37/79025`, path/content
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`313843b1f4f2e210588410de2e1440f1263711fc6cad4085a943d467d5c6ba5a`.
Runner module sizes are source-statement `23936`, facade `959`, root
`2792`, and proof-local test leaf `9648`. Raw/normalized test-list hashes are
checker `d453ca1e8a7cf9870f14a0f933451ca201c19cc8c8367d51767c40a941766f82` /
`7cd84f6cd8e6d1070b39be9e5f1031512cc2c1b664829f10d337f1b67bcb74b3`
and runner `40f4271712d7fed6ed238a2e03b61511fc26914af52333b12732824e740ead4a` /
`e9e4f359a571a1aa383168ff6950568788ecffcea2c4eb5d85934fd4ee15e147`.

Corpus/requirements `428/395`, pass/fail `235/193`, warnings/errors `23/0`,
stages `101/7/205/1`, type `259=247+12`, trace hash
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`,
fixtures, sidecars, expectations, metadata, five CLI bytes, diagnostics,
dispatch, and active results remain unchanged. SDP publishes zero checker,
`BindingEnv`, type, term/reference, capture/closure, fact, proof, obligation,
or coverage credit. The next task is the separate Given-plus-descendant
context/binding consumer; occurrence remains later, and `z`/`q` capture stays
blocked by the Chapter-4/15 `set` `spec_gap`. The implementation self-hash is
pending its task-only commit.
