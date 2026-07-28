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
