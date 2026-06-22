# mizar-checker: Type Checker

> Canonical language: English. Japanese companion:
> [../ja/type_checker.md](../ja/type_checker.md).

## Purpose

`type_checker` specifies phase-6 type checking and inference after resolver name
resolution and binding-environment construction. It refines:

- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  Step 2, "Normalize Type Expressions";
- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  Step 3, "Check Declarations and Local Bindings";
- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  Step 4, "Infer Terms and Formulas";
- [spec chapter 03](../../../spec/en/03.type_system.md) soft types, radix
  types, attributes, and widening;
- [spec chapter 08](../../../spec/en/08.type_inference.md) conversion,
  `qua`, and `reconsider`;
- [spec chapter 13](../../../spec/en/13.term_expression.md) term expressions,
  set enumeration, set comprehension, and choice terms;
- [`typed_ast.md`](./typed_ast.md) type, fact, coercion, obligation, and
  diagnostic tables;
- [`binding_env.md`](./binding_env.md) local binding and context construction.

Task 6 is a specification-only task. It adds no Rust source, no active checker
runner, no language behavior change, no registration closure, no final overload
root selection, no VC generation, and no proof acceptance. Tasks 7-11 implement
the named sections below.

## Boundary

`type_checker` owns:

- normalization of source type expressions into deterministic normalized type
  keys;
- declaration, local-binding, and binding-type checks over `BindingEnv`;
- term and formula type inference before final overload root selection;
- expected-type constraints and unresolved typed candidate groups;
- widening, source-written `qua`, and narrowing coercion candidates;
- checker-local `InitialObligation`s for sethood and narrowing claims;
- type facts, local assumptions, and deterministic fact queries;
- partial typing and diagnostic recovery for recoverable semantic errors.

`type_checker` does not own:

- resolver name lookup, label lookup, import/export validation, or symbol
  allocation;
- binding-context construction or binder identity rules already owned by
  `binding_env`;
- cluster saturation, registration activation, reduction normalization, or the
  canonical `ResolutionTrace` schema;
- final ordinary overload root selection, active refinement joining, or inserted
  overload-disambiguating `qua` views;
- expansion replay for `set`, `deffunc`, or `defpred` bodies beyond preserving
  the definition-time closure metadata supplied by `binding_env`;
- `VcId`, `ObligationAnchor`, proof witness, prover result, accepted verifier
  status, or kernel replay;
- public diagnostic-code allocation while the checker diagnostic code-space
  remains an external planning gate.

## Inputs And Outputs

The type checker consumes:

- one resolver `ResolvedAst`;
- the matching resolver `SymbolEnv`;
- a validated `BindingEnv`;
- checker-local configuration controlling recovery and feature gates;
- dependency summaries and activated registration summaries when later tasks
  expose them through a task-scoped seam.

The type checker produces a checker-local `TypedAst` snapshot:

```rust
struct TypeCheckOutput {
    typed_ast: TypedAst,
    diagnostics: TypeDiagnosticTable,
}
```

The logical implementation may use a mutable `TypeCheckState` while checking,
but accepted output is expressed through `TypedAst` tables:

- `LocalTypeContextTable`;
- `TypeTable`;
- `TypeFactTable`;
- `CoercionTable`;
- `InitialObligationTable`;
- `TypeDiagnosticTable`.

No phase-6 output is a stable artifact schema. Later artifact tasks may project
stable summaries from `TypedAst` only through their own specifications.

## Normalized Type Model

Type checking treats Mizar types as soft predicates over untyped objects. A
normalized type is a radix/type-head key plus a canonical attribute set:

```rust
struct NormalizedType {
    id: NormalizedTypeId,
    head: TypeHeadRef,
    args: Vec<NormalizedTypeId>,
    attributes: AttributeSet,
    source: TypeSource,
    status: NormalizedTypeStatus,
}

enum TypeHeadRef {
    BuiltinObject,
    BuiltinSet,
    Mode(SymbolId),
    Structure(SymbolId),
    Error(TypeHeadErrorKind),
}

enum TypeHeadErrorKind {
    Unknown,
    WrongKind,
    Ambiguous,
    Unsupported,
    Recovery,
}

struct AttributeSet {
    positive: Vec<AttributeInstance>,
    negative: Vec<AttributeInstance>,
}

struct AttributeInstance {
    symbol: SymbolId,
    args: Vec<NormalizedTypeId>,
    source_range: SourceRangeKey,
    spelling: String,
}
```

Canonical type keys are ordered by head kind, canonical `SymbolId`, normalized
argument keys, then attribute keys. Source ranges are retained for diagnostics;
source spelling and ranges are retained on normalized records for debug
rendering, but neither defines semantic equality.

Required invariants:

- built-in `object` and `set` heads are canonical singleton heads;
- mode heads unfold to their declared radix and attribute chain when the resolver
  exposes enough signature payload;
- structure heads remain radix heads and are not silently treated as `set`;
- duplicate attributes with identical polarity and arguments collapse to one
  canonical instance;
- positive and negative occurrences of the same attribute key create a
  diagnostic and degraded normalized type rather than being silently removed;
- type arguments are normalized before they participate in the parent type key;
- missing signature, mode-definition, structure, or attribute payload is an
  `external_dependency_gap`, not an invitation to inspect raw parser syntax.

## Task 7: Type-Expression Normalization

Task 7 implements this section.

Inputs:

- checker-owned resolved type-expression payloads that identify typed sites,
  source ranges, type-head symbols, type arguments, and attribute occurrences;
- type-head, attribute, mode, and structure identities validated through
  `SymbolEnv`;
- an explicit mode-expansion provider for radix/attribute payloads when later
  resolver or artifact tasks expose those payloads.

Outputs:

- a task-local `TypeNormalizationOutput` owning `NormalizedTypeTable` entries in
  deterministic key order;
- `TypeEntry`s for type-expression sites;
- diagnostics for unknown heads, wrong arity, wrong symbol kind,
  contradictory attributes, unsupported checker-owned payloads, and missing
  explicit mode-expansion provider payloads.

Task 7 does not walk `ResolvedAst` directly because the current resolver does
not expose a typed site table for type expressions. The integration seam is the
checker-owned payload listed above. A later resolver/source-walk task may fill
that payload from `ResolvedAst`, but task 7 must not infer it from raw surface
node kinds.

Rules:

1. Resolve the type head only through resolver/symbol payloads. The checker must
   not redo name lookup or inspect raw syntax node kinds.
2. Normalize type arguments left-to-right, then use canonical type keys in the
   parent type.
3. Normalize attribute polarity by storing `non A` in the negative set and `A`
   in the positive set.
4. Sort attributes by canonical symbol id, normalized arguments, polarity, and
   source range; this order is for storage and rendering only.
5. Unfold mode definitions only through the explicit mode-expansion provider
   when it supplies defining radix and attribute payload. If that payload is
   absent, record an `external_dependency_gap` and keep a degraded type headed
   by the mode symbol.
6. Preserve source ranges for every diagnostic. Preserve source spelling and
   range on normalized type and attribute records for debug rendering, using a
   deterministic representative source for semantically equivalent type keys.
7. Never use cluster closure to "fix" a normalized type during task 7. Cluster
   closure is phase 7 and belongs to later registration tasks.

## Task 8: Declaration And Local-Binding Checking

Task 8 implements this section.

Declaration checking attaches normalized types to bindings and introduces local
facts into the appropriate `LocalTypeContext` snapshot.

Required behavior:

- `let`, definition parameters, quantified variables, `given`, `consider`, and
  `take` binders receive `TypeEntry`s linked to their `BindingId` and
  `TypedSiteRef`;
- reserved variables supply default type sites only when the occurrence is not
  shadowed by a local binding;
- `set` declarations infer the right-hand side type and store an abbreviation
  binding with definition-time closure metadata from `BindingEnv`;
- `deffunc` and `defpred` formals are checked as local definition parameters and
  their bodies are checked under the definition-time context;
- `reconsider x as T` updates the type view of an existing binding at the
  current site, while `reconsider y = t as T` introduces a new local binding;
- declarations with attributed or constrained types emit sethood or existence
  `InitialObligation`s when the required evidence is not already a known fact;
- `such that`, `given`, and assumption-like clauses add `Assumed` facts only to
  the context that introduced them.

Invalid declarations must produce explicit diagnostics and partial entries. They
must not fabricate known facts, silently activate registrations, or drop the
source-shaped typed site.

## Task 9: Term And Formula Type Inference

Task 9 implements this section.

Term inference records a `TypeEntry` for each typed term site. Formula inference
records well-formedness and any type facts introduced by formula structure.

Term rules:

- variable references consume `BindingEnv` lookup results and attach the
  selected binding or resolver symbol to the typed site;
- `it` is valid only in definition/property contexts that provide a current
  result type;
- numerals receive the built-in numeric type payload exposed by resolver or a
  degraded external-gap type when that payload is absent;
- functor applications may keep candidate groups when final overload root
  selection is not phase-6-deterministic;
- selector access checks that the selected field or property is visible on the
  current type view and records a candidate group if overload resolution must
  finish the choice;
- structure constructors check field coverage and field value types against
  resolver-exposed structure signatures;
- set enumeration and set comprehension produce set-like types and sethood
  obligations for generator domains when required by spec chapter 13;
- `the T` records a choice-like typed term and a non-emptiness obligation for
  `T` without assigning a proof-owned id;
- source-written `qua` creates a `SourceQua` coercion candidate and changes only
  the type view used by later checking.

Formula rules:

- predicate applications check candidate argument types but keep unresolved
  candidate groups for phase 8 when final root selection is ambiguous;
- built-in `=`, `<>`, and `in` forms check term well-formedness and add
  appropriate expected-type constraints;
- type and attribute assertions check admissibility of the subject term against
  the normalized asserted type or attribute chain;
- logical connectives preserve formula type/well-formedness state and combine
  facts only through explicit assumption/conclusion rules owned by statements;
- quantified formulas create binder contexts through `BindingEnv` and check the
  body under those contexts.

If enough local payload is present to know that a site has no matching typed
candidate, the checker records `Unknown` or `Error` status instead of inventing a
successful type.

## Task 10: Coercion Candidates And Initial Obligations

Task 10 implements this section.

Coercion entries are checker-discovered candidates, not final inserted views.

Required behavior:

- widening candidates are proof-free only when supported by known type facts,
  built-in radix widening, structure inheritance payload, or already activated
  dependency summaries available through a task-scoped seam;
- source-written `qua` is valid only for statically checkable upcasts or
  compatible views; it must not be used as narrowing proof;
- narrowing to a more specific type creates an `InitialObligation` unless a
  later task explicitly specifies a local discharge rule;
- `reconsider` creates narrowing obligations for both existing-binding and
  new-binding forms when the target type is not already supported by known facts;
- sethood and non-emptiness requirements create `InitialObligation`s with source
  assumptions and deterministic local ids;
- failed or unsupported coercions remain as `Blocked` or `Rejected` entries with
  diagnostics.

`InitialObligationId` is the phase-6 boundary. Task 10 must not assign `VcId`,
`ObligationAnchor`, prover status, proof witness, or accepted verifier status.

## Task 11: Type Facts And Queries

Task 11 implements this section.

Type facts are the local currency shared by declaration checking, inference,
coercion checking, and later registration/overload phases.

Fact sources:

- `Declared`: binding declarations and type-expression sites;
- `Assumed`: local assumptions, `such that`, `given`, and proof-context
  assumptions;
- `Inferred`: direct checker rules such as built-in widening or selector result
  typing;
- `Obligation`: facts whose claim is represented by an `InitialObligationId`;
- `Builtin`: built-in facts about `object`, `set`, equality, and membership;
- `Registration`: reserved for later phase-7 closure with a `ResolutionTrace`
  step.

Query rules:

- only `Known` facts are unconditionally consumable;
- `Assumed` facts are consumable only in the introducing context or visible
  descendants recorded in `LocalTypeContextTable`;
- `PendingObligation`, `Degraded`, and `Rejected` facts are not active evidence;
- fact keys include subject, predicate, polarity, provenance class, and the
  context that controls assumption visibility;
- contradictory facts produce diagnostics and explicit statuses instead of being
  resolved by insertion order.

Phase 6 may record facts needed by later registration resolution, but it must not
create `Registration` provenance or trace steps before phase 7 owns the
corresponding derivation.

## Partial Typing And Recovery

Recoverable errors must leave explicit partial state:

- unresolved or ambiguous type heads create degraded `NormalizedType`s and
  `TypeEntry` diagnostics;
- unresolved terms, missing binding payloads, missing signature payloads, and
  impossible candidate groups produce `Unknown`, `Error`, or `Skipped` entries;
- facts and coercions derived from degraded sites carry `Degraded`, `Blocked`,
  or `Rejected` status;
- diagnostics retain primary source ranges and stable secondary keys;
- later phases must check status predicates before consuming type, fact,
  coercion, or obligation entries.

Recovery is an under-approximation policy. The checker may omit facts and emit
diagnostics, but it must not invent verified facts, activate registrations, or
mark an obligation as accepted.

## Diagnostics

Task-local diagnostics use stable message keys until the public checker
diagnostic code-space is allocated.

Required diagnostic classes include:

- unknown or ambiguous type head;
- unsupported or missing resolver signature payload;
- wrong type-argument arity or kind;
- contradictory attributes;
- uninhabited or unsupported attributed declaration;
- illegal declaration or local-binding type;
- invalid `qua` target or narrowing without an obligation;
- failed sethood or non-emptiness requirement;
- term/formula kind mismatch;
- ambiguous or impossible candidate group;
- partial-typing recovery boundary.

Diagnostics are not proof evidence. They may explain degraded table entries, but
they must not be used as supporting facts.

## Determinism

Task 7-11 implementations must preserve deterministic output:

- normalized type ids are allocated by canonical type key;
- declaration, term, formula, coercion, obligation, fact, and diagnostic
  iteration order is independent of hash-map iteration;
- candidate groups are sorted by resolver candidate identity, mandatory type
  constraints, source range, then stable local id;
- debug rendering extends `typed-ast-debug-v1` without host paths, memory
  addresses, or nondeterministic map order;
- equivalent `ResolvedAst`, `SymbolEnv`, `BindingEnv`, dependency summaries, and
  checker configuration produce equivalent `TypedAst` tables.

## Planned Tests For Tasks 7-11

Task 7 must add Rust tests for:

- attribute sorting, deduplication, polarity, and contradiction diagnostics;
- built-in singleton heads, structure heads that remain radix heads, and
  recursive type-argument normalization;
- mode unfolding idempotence when the explicit mode-expansion provider supplies
  payload;
- degraded mode/type entries when signature payload is missing;
- unknown or ambiguous heads, wrong arity/kind diagnostics, and source-range
  preservation;
- deterministic normalized type ids;
- the guard that type normalization does not use cluster closure to repair
  degraded types.

Task 8 must add Rust tests for:

- binding type attachment for `let`, quantified binders, definition parameters,
  reserved variables, `set`, `deffunc`, `defpred`, and `reconsider`;
- reserved-variable shadowing and definition-time closure metadata for `set`,
  `deffunc`, and `defpred`;
- both `reconsider` forms and obligation emission for constrained
  declarations;
- local assumption visibility and context snapshot updates;
- partial entries after invalid declarations;
- deterministic local-context, type-entry, diagnostic, and debug-rendering
  order.

Task 9 must add Rust tests for:

- variable, numeral, selector, structure, set-expression, choice, `qua`, and
  parenthesized term sites;
- `it` validity and built-in `=`, `<>`, and `in` expected-type constraints;
- predicate applications, type assertions, attribute assertions, connectives,
  and quantified formulas;
- candidate groups that remain open for overload resolution;
- sorted candidate groups and deterministic term/formula/diagnostic rendering;
- unknown/error/skipped partial typing, including the rule that recovery does
  not fabricate successful types.

Task 10 must add Rust tests for:

- widening, source `qua`, and narrowing coercion candidates;
- invalid `qua` targets that do not become narrowing proof;
- failed or unsupported coercions that remain `Blocked` or `Rejected`;
- sethood and non-emptiness initial obligations;
- `reconsider` obligation source ranges and assumption lists;
- deterministic coercion, obligation, diagnostic, and debug-rendering order;
- boundary guards that no `VcId` or proof-owned status is assigned.

Task 11 must add Rust tests for:

- fact deduplication and canonical query order;
- consumability rules for `Known`, `Assumed`, `PendingObligation`, `Degraded`,
  and `Rejected`;
- contradiction diagnostics;
- absence of `Registration` provenance before phase 7 trace ownership.

No `.miz` checker-stage fixtures are required by task 6 because it is
documentation-only. Task 12 still owns the first active `type_elaboration`
corpus runner and traceability metadata.

## Task 6 Classification

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | No task-6 blocking spec gap remains for the named phase-6 sections. Chapters 03, 08, and 13 plus architecture 04 provide enough authority for normalization, declaration checking, term-expression inference, coercion candidates, facts, and recovery. | Continue to task 7 after task-6 review, verification, and commit. |
| `test_gap` | Active checker-stage `.miz` coverage and `type_elaboration` runner remain absent. | Tasks 7-11 add task-local Rust tests. Task 12 owns active corpus coverage and traceability metadata. |
| `design_drift` | Architecture 04 examples use broad `TypeContext` and `CoercionCandidateTable` names, while existing checker docs use `BindingEnv`, immutable `LocalTypeContextTable`, and `CoercionTable`. | This spec preserves the refined checker module split and treats `CoercionTable` entries as candidates until later phases resolve them. |
| `source_drift` | At task 6 time `src/type_checker.rs` did not exist yet. | Resolved by task 7, which creates the module and exports it from `lib.rs`; no source repair belonged to task 6. |
| `external_dependency_gap` | Several implementation seams depend on resolver-exposed signature payloads for mode unfolding, structure fields, attributes, functor/predicate candidates, built-ins, and dependency activated summaries. Public checker diagnostic codes are also not allocated. | Implementation tasks must consume only exposed resolver/artifact payloads. Missing payloads become external dependency gaps or degraded diagnostics; do not add direct raw-syntax reconstruction. |
| `deferred` | Registration closure, reduction normalization, final overload selection, inserted overload-disambiguating `qua` views, VC generation, proof acceptance, kernel replay, and artifact publication remain outside task 6 and phase 6. | Later checker and downstream crate tasks own these boundaries. |
