# Binder Normalization

> Canonical language: English. Japanese companion:
> [../ja/binder_normalization.md](../ja/binder_normalization.md).

Status: task 4 module specification for `mizar-core`. This document is
normative for tasks 5 and 6. It refines
[architecture 16](../../architecture/en/16.substitution_and_binding.md) for
core terms and formulas and uses [core_ir.md](./core_ir.md) as its data-shape
input.

## Scope

`binder_normalization` owns deterministic binding behavior over `CoreIr`
terms, formulas, binders, proof skeleton fragments, and algorithm contract
formulas. It provides the representation and operations later replayed by VC
generation and kernel substitution checking.

Owned behavior:

- canonical binder representation for normalized core terms and formulas;
- alpha-equivalence by canonical form, not display names;
- free-variable computation and side-condition extraction;
- capture-avoiding substitution over normalized core terms and formulas;
- definition-time closure representation and substitution mechanics for `set`,
  `deffunc`, and `defpred`;
- deterministic fresh variable allocation when a normalized result needs a
  source-facing binder again;
- structured rejection of malformed substitution evidence.

Out of scope:

- name resolution, type checking, overload resolution, registration
  activation, or proof search;
- source-to-checker payload extraction;
- VC id assignment, kernel certificate format emission, or proof acceptance;
- raw source rescanning or text-macro expansion.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| BIND-G001 | `spec_gap` | `binder_normalization.md` did not exist before task 4. | This document closes the module-spec gap for tasks 5 and 6. |
| BIND-G002 | `test_gap` | No binder-normalization source or tests exist before task 5. | Task 5 adds substitution tests; task 6 adds alpha-equivalence and normalization tests. |
| BIND-G003 | `external_dependency_gap` | `mizar-kernel` now exists, but binder-normalized replay/consumer integration is still not wired through the evidence pipeline. | Specify replayable shapes only; do not fabricate kernel APIs or proof-consumer shortcuts. |
| BIND-G004 | `external_dependency_gap` | Source-derived checker payload extraction remains deferred upstream. | Use explicit Rust fixtures until real source-to-core fixtures can be produced. |
| BIND-G005 | `deferred` | Cross-edit proof reuse anchors and `VcId`s are downstream concerns. | Binder paths must be deterministic but must not allocate downstream ids. |

## Representation Decision

`mizar-core` uses a locally nameless representation for normalized terms and
formulas:

- bound variables are represented by de Bruijn indices relative to the current
  binder stack;
- free variables keep stable `CoreVarId`s from the surrounding core context;
- schematic variables and generated fresh variables keep explicit stable ids
  with a variable class;
- source display names are retained only in binder frames and diagnostics.

This is the chosen representation for tasks 5 and 6.

Rationale:

- Alpha-equivalent inputs normalize to identical structural encodings without
  carrying a name-renaming map.
- Free variables captured by definition-time closures remain stable ids, so
  later shadowing of a display name cannot change meaning.
- Substitution replay can run as a pure linear walk over the normalized tree
  and binder stack.
- The kernel can re-check side conditions without consulting resolver,
  checker, parser, cluster, overload, or ATP state.
- Pure de Bruijn would obscure captured free variables needed for closures and
  diagnostics; named-with-alpha would require replaying a renaming map and is
  too easy to make display-name dependent.

## Core Shapes

Task 5 may implement these as concrete Rust types or equivalent private
structures, but it must preserve the invariants below.

```rust
enum NormalizedVar {
    Bound(BoundVar),
    Free(CoreVarId),
    Schematic(CoreVarId),
    Generated(CoreVarId),
}

enum NormalizedVarClass {
    Free,
    Schematic,
    Generated,
}

enum NormalizedVarSort {
    Term,
    Formula,
}

struct BoundVar {
    index_from_innermost: u32,
}

struct BinderContext {
    frames: Vec<BinderFrame>,
    free_variables: BTreeSet<CoreVarId>,
    variable_classes: BTreeMap<CoreVarId, NormalizedVarClass>,
    variable_roles: BTreeMap<CoreVarId, CoreVarRole>,
    variable_sorts: BTreeMap<CoreVarId, NormalizedVarSort>,
}

struct BinderFrame {
    canonical_index: u32,
    original_var: CoreVarId,
    role: CoreVarRole,
    source_name: Option<String>,
    source: CoreSourceRef,
}

struct NormalizedBinderEntry {
    frame: BinderFrame,
    ty_guard: Option<Box<NormalizedFormula>>,
}

struct GeneratedOriginRecord {
    owner: CoreItemId,
    kind: GeneratedOriginKind,
    key: GeneratedOriginKey,
    params: Vec<NormalizedVar>,
}

enum NormalizedTermKind {
    Var(NormalizedVar),
    Const(SymbolId),
    Apply { functor: SymbolId, args: Vec<NormalizedTerm> },
    Select { selector: SymbolId, base: Box<NormalizedTerm> },
    Tuple(Vec<NormalizedTerm>),
    SetEnum(Vec<NormalizedTerm>),
    Generated { origin: GeneratedOriginRecord, args: Vec<NormalizedTerm> },
    Error(CoreDiagnosticId),
}

struct NormalizedTerm {
    kind: NormalizedTermKind,
    free_variables: BTreeSet<CoreVarId>,
}

enum NormalizedFormulaKind {
    // Non-binder logical nodes omitted here follow `CoreFormulaKind`.
    Forall { binders: Vec<NormalizedBinderEntry>, body: Box<NormalizedFormula> },
    Exists { binders: Vec<NormalizedBinderEntry>, body: Box<NormalizedFormula> },
    Error(CoreDiagnosticId),
}

struct NormalizedFormula {
    kind: NormalizedFormulaKind,
    free_variables: BTreeSet<CoreVarId>,
}
```

`BoundVar.index_from_innermost = 0` names the nearest binder. Entering a
binder increases the index of every outer bound variable by one. Leaving a
binder decreases only variables whose index is outside the closed body. Indices
are semantic; source names are not.

`CoreVarId` is stable only inside one `CoreIr` snapshot. It must not be used as
a cross-snapshot proof reuse anchor.

`variable_classes`, `variable_roles`, and `variable_sorts` are the required
source of truth for `Free`, `Schematic`, and `Generated` classification,
role-sensitive checks, and term/formula compatibility. Task 5 must either
implement these maps or an equivalent context object. A
`CoreTermKind::Var(CoreVarId)` occurrence not in the binder stack is classified
by this context; absent class entries default to `Free` only for ordinary local
core variables during raw normalization. Schematic or generated ids must be
explicit context entries. Validation and canonicalization require explicit
class/role/sort metadata for every non-bound normalized variable; the
normalization-only default is not reused after a normalized object has been
constructed. Binder frames carry their own role and source, so a freshened
binder frame remains valid when no context metadata exists for its fresh id; if
context metadata is present, it must be complete and agree with the frame.

## Binder Normalization

Normalization walks a `CoreTerm` or `CoreFormula` with a binder stack:

1. When entering a binder, push a `BinderFrame` in lexical binder order.
2. The binder receives the next `canonical_index` in the current normalized
   object.
3. Occurrences of the binder's `original_var` inside its scope become
   `NormalizedVar::Bound(BoundVar { index_from_innermost })`.
4. Occurrences not found in the binder stack remain `Free`, `Schematic`, or
   `Generated` according to the surrounding core context.
5. Source names and source ranges are preserved in frames/provenance but are
   excluded from semantic equality and hashing.

For multi-binder constructs, binders are processed left to right:

- the guard of binder `i` is normalized after prior binders plus binder `i`
  have been pushed, and before binders `i+1..` are visible;
- therefore a binder's own guard may refer to that binder when the surface form
  permits self-typed predicates, may refer to prior binders, and must not refer
  to later binders;
- the body is normalized only after every binder in the construct has been
  pushed.

This rule applies to quantifiers, definition formals, scheme/template formals,
proof-introduction binders, and algorithm parameter/result binder lists. It
prevents equivalent typed binder lists from diverging between alpha-equivalence
and substitution. Type guards must never rely on display-name lookup.

Normalization output must be deterministic across platforms, worker counts,
hash-map iteration orders, and diagnostic display names.

The public task-6 normalization API is:

```rust
fn normalize_core_term(
    core: &CoreIr,
    term_id: CoreTermId,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm>;

fn normalize_core_formula(
    core: &CoreIr,
    formula_id: CoreFormulaId,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula>;
```

Missing dense-id rows, generated-origin rows, or diagnostic rows are malformed
binding evidence and must return `BinderDiagnosticClass::MalformedEvidence`.
The functions do not call resolver, checker, VC, proof, or kernel services.

## Alpha-Equivalence

Two normalized terms or formulas are alpha-equivalent exactly when their
semantic normalized encodings are structurally equal after excluding:

- source display names;
- source ranges;
- diagnostics and recovery metadata;
- provenance that is not part of the logical node.

Alpha-equivalence must include:

- bound-variable indices;
- free/schematic/generated stable ids and classes;
- functor and predicate `SymbolId`s;
- `CoreTermKind::Generated` by the generated-origin semantic record: owner,
  kind, semantic key, alpha-normalized payloads, and normalized argument order.
  Dense `GeneratedOriginId`s may be used only as table lookups within one
  `CoreIr` snapshot; stable-choice terms must already be ordinary `Apply`
  nodes;
- recovery/error nodes by diagnostic id only when both sides are explicitly
  recovery data. Error nodes are not valid logical terms for proof acceptance;
- binder roles and type guards;
- logical structure and child order.

Task 6 must expose an alpha-equivalence API that is a pure comparison of
normalized structures. It must not re-run name resolution or consult source
text.

Task 6 also exposes:

```rust
fn validate_normalized_term(
    term: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()>;

fn validate_normalized_formula(
    formula: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()>;

fn canonical_term(
    term: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<CanonicalTerm>;

fn canonical_formula(
    formula: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<CanonicalFormula>;

fn alpha_equivalent_terms(
    left: &NormalizedTerm,
    right: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<bool>;

fn alpha_equivalent_formulas(
    left: &NormalizedFormula,
    right: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<bool>;
```

Validation starts at `context.frames.len()` so terms and formulas normalized
under an ambient binder context remain valid. It must reject invalid de Bruijn
indices, malformed binder metadata, later-binder references in guards, and any
public `free_variables` cache that does not match the normalized structure.

Canonical keys exclude source names, source ranges, provenance, and diagnostic
display text; they include bound indices, free/schematic/generated ids and
classes, symbol ids, binder roles and guards, generated semantic origin
records, child order, and recovery diagnostic ids. The key shape is equivalent
to:

```rust
enum CanonicalVar {
    Bound(u32),
    Free(CoreVarId),
    Schematic(CoreVarId),
    Generated(CoreVarId),
}

struct CanonicalBinderEntry {
    role: CoreVarRole,
    ty_guard: Option<Box<CanonicalFormula>>,
}

struct CanonicalGeneratedOrigin {
    owner: CoreItemId,
    kind: GeneratedOriginKind,
    key: GeneratedOriginKey,
    params: Vec<CanonicalVar>,
}
```

## Free Variables

Free-variable computation returns a sorted set of stable ids. Entering a binder
removes that binder's `original_var` from the body's free-variable set. Shadowed
source names remain distinct because their `CoreVarId`s are distinct.

Free-variable constraints have this shape:

```rust
struct FreeVariableConstraint {
    variable: CoreVarId,
    must_remain_free_in: NormalizedTermOrFormulaPath,
}
```

Constraints are emitted when substitution replay or later kernel checking must
prove that a variable was not captured.

## Capture-Avoiding Substitution

Substitution is defined over normalized terms and formulas:

```rust
struct Substitution {
    target: SubstitutionTarget,
    replacement: SubstitutionReplacement,
    side_conditions: Vec<FreeVariableConstraint>,
}

enum SubstitutionResult<T> {
    Applied(T),
    Rejected(BinderDiagnostic),
}

enum SubstitutionTarget {
    TermVar(NormalizedVar),
    FormulaVar(NormalizedVar),
}

enum SubstitutionReplacement {
    Term(NormalizedTerm),
    Formula(NormalizedFormula),
}

struct BinderDiagnostic {
    class: BinderDiagnosticClass,
    source: CoreSourceRef,
    owner: Option<CoreNodeRef>,
    message_key: CoreDiagnosticMessageKey,
}
```

Rules:

- Substitution is a pure function of normalized source, normalized target,
  binder context, and side conditions.
- Substitution is sort-safe. A `TermVar` target accepts only a term
  replacement; a `FormulaVar` target accepts only a formula replacement.
  Variable class, role, and sort constraints come from the target
  `BinderContext`. If a free, schematic, or generated variable lacks required
  class/role/sort metadata, the operation is rejected or deferred with a
  diagnostic payload instead of guessing.
- Bound substitutions use de Bruijn shifting when crossing binders.
- Free-variable substitutions preserve `CoreVarId`s and must not be captured
  by a binder in the target context.
- If a replacement would be captured, the implementation must use the
  deterministic freshness strategy before converting the result back to a
  binder-bearing core shape.
- If freshness or side conditions cannot be satisfied, substitution is
  rejected. It must not silently produce a capture-prone term.
- Substitution never activates registrations, unfolds unrelated definitions,
  runs proof search, or resolves overloads.

`BinderDiagnostic` is an owned diagnostic payload, not an already-inserted
`CoreDiagnosticId`. The caller that integrates the result into `CoreIr` is
responsible for inserting it into the diagnostic table and wiring any error
node or skipped item to the resulting id. This keeps substitution replay pure
and prevents dangling diagnostic ids.

### De Bruijn Operations

Task 5 must implement these operations exactly or expose equivalent helpers:

```text
shift(term_or_formula, cutoff, delta)
open_rec(term_or_formula, depth, replacement)
open(term_or_formula, replacement) = open_rec(term_or_formula, 0, replacement)
close_rec(term_or_formula, depth, variable)
close(term_or_formula, variable) = close_rec(term_or_formula, 0, variable)
subst_bound(term_or_formula, depth, replacement)
```

Definitions:

- `shift(cutoff, delta)` adds `delta` to every bound index `i >= cutoff` and
  leaves `i < cutoff` unchanged. Negative deltas are allowed only when no index
  would become negative; otherwise the operation is rejected.
- `open_rec(body, depth, replacement)` replaces bound index `depth` with
  `shift(replacement, 0, depth)`, decrements indices greater than `depth`, and
  leaves indices below `depth` unchanged. When walking under one binder,
  `depth` increases by one.
- `close_rec(body, depth, variable)` replaces free occurrences of `variable`
  with bound index `depth`, increments indices `>= depth`, and recurses under a
  binder with `depth + 1`.
- `subst_bound(body, depth, replacement)` replaces bound index `depth` with
  `shift(replacement, 0, depth)` and leaves the binder structure in place;
  under a binder it recurses with `depth + 1`. Unlike `open_rec`, it does not
  decrement surrounding indices.

Use-site-normalized actual arguments are stored at depth `0`. They are not
pre-shifted by the use-site binder stack. `open_rec` and `subst_bound` are the
only operations that shift replacements, and only by the depth at the insertion
point under the closure body or target binder.

All operations must preserve sorted free-variable sets and must reject invalid
indices instead of saturating or wrapping.

### Freshness Witnesses

Freshening produces replayable evidence:

```rust
struct FreshnessWitness {
    source_id: SourceId,
    owner: CoreItemId,
    original: CoreVarId,
    fresh: CoreVarId,
    binder_path: NormalizedTermOrFormulaPath,
    role: CoreVarRole,
    counter: u32,
}
```

`mizar-core` owns recording these witnesses as internal replay evidence for
later phases. It does not define a kernel certificate schema, but the witness
or its surrounding replay envelope must provide `source_id`, owner
`CoreItemId`, normalized binder path, role, original/fresh ids, and counter so
a downstream kernel checker can recompute the fresh id from the deterministic
freshness strategy and verify that the renaming avoided capture.

Composition law for task 5 tests:

```text
subst(B, y := U) after subst(A, x := T)
```

must equal the single normalized composition when the side conditions prove
that `x` and `y` do not capture each other. Otherwise the composition must be
rejected with a deterministic diagnostic.

## Definition-Time Closures

Local `set`, `deffunc`, and `defpred` definitions are stored as normalized
closures:

```rust
struct DefinitionClosure {
    formals: Vec<BinderFrame>,
    body: NormalizedTermOrFormula,
    captured_free_variables: BTreeSet<CoreVarId>,
    formal_type_guards: Vec<NormalizedFormula>,
    source: CoreSourceRef,
}
```

The stored body is closed over the closure formals: formal occurrences are
represented as bound indices relative to the `formals` prefix, and captured
definition-site variables remain free `CoreVarId`s in
`captured_free_variables`. The closure body is not open relative to the
use-site binder stack.

At definition time, the closure records every free variable id in the body. At
use time:

1. Actual arguments are normalized in the use-site context.
2. Actuals are kept at depth `0`; they are shifted only by `open_rec` or
   `subst_bound` at the point where they are inserted under closure-body
   binders.
3. Actuals are substituted for formal bound variables from right to left so
   earlier replacements cannot change the remaining formal indices.
4. The expanded body is opened into the use-site context.
5. Captured free variables keep their definition-site `CoreVarId`s.
6. Formal type guards are closed over the same formals as the body, then
   instantiated with the same actual substitutions from right to left. Expansion
   is conditionally valid only with those instantiated guard side conditions
   preserved until they are discharged.
7. Instantiated formal type guards are returned with the expansion result as
   normalized guard facts.
8. The elaborator, not `binder_normalization`, decides whether those guard
   facts become explicit predicates, assumptions, obligation seeds, or
   diagnostics, but it must not drop them silently.

Required regression:

```mizar
defpred P(n be Nat) means n < m;
for m being Nat holds P(m)
```

The quantified `m` used as the actual argument and the captured outer `m` in
the definition body must remain distinct variable ids after expansion.

## Deterministic Freshness

Fresh variable ids are generated from:

- `SourceId`;
- owner `CoreItemId`;
- normalized binder path;
- variable role;
- deterministic counter scoped to the normalized object.

Generated display names are diagnostic-only. They must not affect equality,
hashes, proof acceptance, or generated-origin keys.

## Diagnostics

Malformed binding evidence produces structured core diagnostics. Examples:

- unbound variable id in a supposedly closed normalized term;
- de Bruijn index outside the binder stack;
- free-variable side condition violation;
- substitution that would capture a free variable;
- deterministic freshness exhaustion or collision;
- closure actuals that do not match formals.

Diagnostics preserve source/core provenance, but a diagnostic never repairs the
term heuristically. Invalid normalized fragments remain explicit error nodes or
cause the owning operation to return `Rejected`.

## Test Obligations

Task 5 must add Rust tests for:

- capture-avoiding substitution under nested binders;
- shadowing by display name with distinct stable ids;
- deterministic freshening when substitution crosses binders, including the
  emitted `FreshnessWitness` `source_id`, owner, original id, fresh id, binder
  path, role, and counter used to recompute the fresh id;
- substitution composition laws for accepted cases;
- rejected substitution composition when side conditions or capture checks
  fail;
- rejected malformed substitutions, including capture-producing evidence,
  explicit side-condition violations, deterministic freshness collision or
  exhaustion, term/formula sort mismatches, role mismatches, and missing
  class/role/sort metadata for free, schematic, or generated variables;
- definition-time closure expansion, including the `defpred P(n be Nat) means
  n < m` shadowing regression;
- closure arity/formal mismatch rejection and preservation of actual-instantiated
  formal type guards in expansion results.

Task 6 must add Rust tests for:

- alpha-equivalence reflexivity, symmetry, and transitivity;
- normalization idempotence;
- identical canonical forms if and only if terms/formulas are
  alpha-equivalent;
- deterministic canonical forms across repeated runs;
- free-variable set ordering and binder removal;
- rejection of invalid de Bruijn indices and malformed binder contexts;
- `shift`, `open_rec`, `open`, `close_rec`, `close`, and `subst_bound` edge
  cases under nested binders, including nonzero-depth `open_rec`/`close_rec`
  behavior and the contrast between `open_rec` decrementing surrounding indices
  and `subst_bound` preserving binder structure;
- alpha-equivalence for generated terms, including distinct dense ids with
  equal semantic origin records, differing owner/kind/key/alpha-normalized
  payloads, differing argument order, and explicit recovery/error nodes.

No `.miz` fixture is required for task 4. Source-derived substitution coverage
remains deferred until checker payload extraction and mizar-test stage support
can feed `mizar-core` without fabricated inputs.

## Public Enum Policy

Task 21 classifies every `binder_normalization` public enum as a downstream
forward-compatible API surface. Each enum must remain `#[non_exhaustive]` so
future binder, substitution, canonicalization, and diagnostic categories can be
added without breaking downstream exhaustive matches.

| Public enum | Decision |
|---|---|
| `NormalizedVar` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `NormalizedVarClass` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `NormalizedVarSort` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `NormalizedTermKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `NormalizedFormulaKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CanonicalTermKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CanonicalFormulaKind` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CanonicalVar` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `NormalizedTermOrFormula` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `SubstitutionTarget` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `SubstitutionReplacement` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CapturePolicy` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `SubstitutionResult` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `BinderDiagnosticClass` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module. Internal
`mizar-core` matches may remain exhaustive where they deliberately enumerate the
current variants.

## Forbidden Behavior

`binder_normalization` must not:

- compare source display names for semantic equality;
- perform raw source lookup or text macro expansion;
- repair malformed substitution evidence heuristically;
- inspect active resolver, checker, parser, cluster, overload, ATP, or kernel
  state during replay;
- allocate `VcId`s, proof acceptance status, or cross-edit reuse anchors.
