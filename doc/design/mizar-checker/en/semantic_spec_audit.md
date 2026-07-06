# Semantic Specification Audit: mizar-checker Scope

> Canonical language: English. Japanese companion:
> [../ja/semantic_spec_audit.md](../ja/semantic_spec_audit.md).

This audit reviews the language-specification chapters that the
`mizar-checker` crate (pipeline phases 6-8) must implement — type system,
structures, attributes, modes, type inference, clusters/registrations, and
overload resolution — for contradictions, underspecified cases, room for
nondeterminism, and places where termination cannot be demonstrated. It also
checks consistency with architecture 04/05 and the checker TODO.

Scope of this change:

- **No specification text is edited.** Every finding that needs a spec
  amendment has at least two sound resolutions; the choice changes surface
  language or checker behavior, so each is recorded here with proposed
  resolutions and left to a follow-up spec task.
- An adversarial rejection corpus is fixed test-first under `tests/miz/fail/`
  for the behaviors the specification already states unambiguously (see
  [Adversarial Corpus](#adversarial-corpus)).

Audited sources: `doc/spec/en/` 03, 05, 06, 07, 08, 13, 14, 17, 18, 19;
`doc/design/architecture/en/` 04, 05; `doc/design/mizar-checker/en/todo.md`.

## Severity Legend

| Severity | Meaning |
|---|---|
| critical | A conforming implementation can be made unsound (false becomes provable). |
| high | The checker cannot be implemented deterministically without a decision; or spec and design contradict. |
| medium | Underspecified behavior that will produce divergent implementations or diagnostics. |
| low | Editorial or wording defect with a single obvious repair. |

Classification uses the AGENTS.md taxonomy (`spec_gap`, `design_drift`, ...).

## Findings Index

| Id | Severity | Area | Summary |
|---|---|---|---|
| SSA-001 | critical | 5.5/5.8 | Resolved by task 35: constructor-supplied property values plus fields-only extensionality collapsed the logic |
| SSA-002 | high | 5.3/5.4 | Resolved by task 36: member identity tracks root declaration plus inheritance path/view |
| SSA-003 | high | 19.6.1 | Resolved by task 37: template constraints are not Phase B tie-breakers after expansion |
| SSA-004 | high | 17.5/17.9.3 | Functorial cluster `for T` clause has no semantics in the FOL encoding |
| SSA-005 | high | 7.4.1 | Property implementations lack coherence conditions across overlapping modes |
| SSA-006 | high | 17.1 vs arch 04 | Registration activation timing: spec is item-ordered, design defers to verifier acceptance |
| SSA-007 | medium | 17.10/3.3 | Cluster termination silently relies on the restricted adjective grammar |
| SSA-008 | medium | 17.7.3 | Contradiction detection site is inconsistent (ATP vs closure) |
| SSA-009 | medium | 17.6.4 | Reduction determinism claim conflicts with `such`-condition context dependence |
| SSA-010 | medium | 19.4.3/19.4.4 | Resolved by task 37: ambiguity covers multiple maximal roots, including equivalent roots |
| SSA-011 | medium | 5.4 vs 19.2.2 | Resolved by task 36: implicit upcast path uniqueness is syntactic |
| SSA-012 | medium | 5.3 | Resolved by task 36: inheritance acyclicity is explicit with `structures.inherit.cycle` |
| SSA-013 | medium | 7.8.1 | `sethood` obligation form for dependent (parameterized) modes is not given |
| SSA-014 | medium | 7.8/17.3.4 | Existence requirements for unattributed bases and built-ins are unstated |
| SSA-015 | medium | 8.2 | `reconsider` with omitted justification has no defined discharge path |
| SSA-016 | low | 19.2.3 | Resolved by task 37: specificity is a preorder before quotienting by closure-equivalence |
| SSA-017 | low | 6.7/19.4.1 | `coherence with` omitted with several sharpenable originals: diagnostic unspecified |
| SSA-018 | low | 19.6.4 | Greedy `of`/`over` parse depends on the in-scope arity set |
| SSA-019 | low | 19.6.1 | Resolved by task 37: duplicated introductory sentences removed |
| SSA-020 | medium | 3.3/6.2 | Argument-list attribute form `attr(args)` is usable but never declarable |

## Findings

### SSA-001 (critical, resolved `spec_gap`) — Constructor property arguments vs fields-only extensionality

**Where:** 05.structures.md §5.5.1, §5.8.4, §5.8.5; 07.modes.md §7.4.1.

Before task 35, §5.5.1 let the default constructor supply **property** values
(`OneStr(carrier: A, one: b)`), and §5.8.4 emitted projection axioms for them
(`one(Agg_OneStr(A, b)) = b`). §5.8.5 stated extensionality over **fields
only**. Together these were inconsistent: for any `b1, b2` of the property
type, `Agg_OneStr(A, b1)` and `Agg_OneStr(A, b2)` agreed on all fields, so
extensionality forced `Agg_OneStr(A, b1) = Agg_OneStr(A, b2)`, and the
projection axioms then proved `b1 = b2` — every carrier collapsed to at most
one element, from which false statements became provable. Constructor
property arguments also competed with §7.4.1 `means`/`equals` property
implementations as a second, unreconciled source of the property value.

**Impact:** any implementation that emits the three axiom families as written
is unsound. Blocks the elaboration/VC side of constructor support; the
checker needs to know which obligations a constructor call emits.

**Proposed resolutions (choose one):**

1. Constructors accept **fields only**; property values always come from
   property implementations. §5.5.1's example and §5.8.4's aggregator change.
2. Keep property arguments but (a) include properties in the extensionality
   tuple, and (b) when the property has an implementation, emit a proof
   obligation that the supplied value satisfies the definiens. Note (a)
   contradicts the §7.8.2 uniqueness story and "identity is determined solely
   by fields".

Resolution 1 is recommended; it keeps the many-sorted-set reading intact.

**Disposition:** task 35 adopts resolution 1. Spec 05 now makes default
constructors fields-only and removes property projection axioms; spec 07
states that property implementations are the only source of property values.
The inactive reject-first seed
`fail_structure_constructor_property_arg_001` pins the rejected
constructor-property form until `advanced_semantics` runner and
source-to-checker payload gaps close.

### SSA-002 (high, `spec_gap`) — Diamond member identity under renaming

**Where:** 05.structures.md §5.3.1 (renaming), §5.4.

§5.4 keys the automatic diamond-consistency check on members of the *same
name and type* while also saying the verifier "traces the `from` chains".
§5.3.1 permits renaming (`field derived_field from base_field`), so two
parents can expose one grandparent member under different names, or unrelated
members under one name. Name-based and origin-based identity disagree
exactly in those cases, and the spec does not say which one governs, nor what
the child's joined member type must satisfy when parents disagree (only the
two-parent example with a common subtype is shown; incomparable parent types
are unspecified).

**Resolution:** task 36 records the superior root-plus-path rule in spec 05.
Inherited member identity tracks the root declaration reached by `from`
mappings and the inheritance path/view that reaches it. The root coordinate
identifies the ancestor declaration being covered; the path/view coordinate
preserves reduct terms and prevents evidence from crossing renamed or
multi-path views. A child member may realize several parent members in a
diamond join, but its type must be `⊑` every covered parent member type; the
existing `coherence` block discharges the non-identical type obligations for
that `inherit` declaration. Same-name/same-type joins from distinct roots
remain valid, and the same root reached through distinct renamed paths may
remain exposed as distinct child views/selectors.

**Corpus:** `fail_structure_diamond_member_type_conflict_001`,
`fail_structure_inherit_uncovered_member_001`, and task-36 seed
`fail_structure_inherit_duplicate_member_coverage_001` pin the rejection
cases. Task 36 does not add a renamed-view reject seed because renamed-view
exposure is positive behavior; `fail_template_qua_view_attribute_leak_001`
remains the negative guard against evidence leaking across those views.

### SSA-003 (high, resolved `spec_gap`) — Template inference examples contradicted the selection rule

**Where:** 19.overload_resolution.md §19.6.1 Cases 2-3 vs §19.4.3;
architecture 05 "narrow tie-breakers".

Task 37 chose the conservative spec rule: Phase A produces concrete
template-derived candidates, and Phase B compares those concrete normalized
parameter vectors with the normal `⊑` preorder. Declared template constraint
strictness is not a Phase B tie-breaker. A non-template candidate wins only
when its concrete vector is closure-equivalent to a template-derived vector
and every other allowed tie-breaker is also tied.

The §19.6.1 examples now follow that rule. Case 2, where two template-derived
roots instantiate to the same concrete vector but come from distinct ordinary
roots, is ambiguous. Case 3, where the argument has exact type `C`, selects
the template-derived candidate with concrete parameter `C` over the
non-template `B` candidate because `C ⊏ B`.

Architecture 05 and checker `overload_resolution.md` now carry the same
tie-breaker list. This decision coordinates with the separate Phase A rule
recorded by `mizar-core` task 26 / F7: omitted-template-argument inference is
based on declared argument types and must not be inferred from missing payloads.

### SSA-004 (high, `spec_gap`) — Functorial cluster `for T` has no encoding

**Where:** 17.clusters_and_registrations.md §17.5, §17.9.3.

The syntax is `cluster F(args) -> adjectives for T`, but every FOL encoding
in §17.9.3 drops `T` (`cluster n ! -> positive for Nat` ⟹
`∀n. is_Nat(n) → is_positive(factorial(n))` — the `for Nat` contributes
nothing). Candidate meanings differ observably: (a) consequent constraint
`is_T(F(args))` added to the axiom; (b) applicability guard — the cluster
fires only where the result is already known to be `T`; (c) documentation
only. Trigger indexing and closure results differ under (a) vs (b).

**Proposed resolution:** specify (b) — the registration applies when the
result type's radix is `T` or a subtype (mirroring conditional clusters
§17.7.2) — and additionally emit `is_T(F(args))` premises in the coherence
obligation. Update §17.9.3 tables.

### SSA-005 (high, `spec_gap`) — Overlapping property implementations lack coherence

**Where:** 07.modes.md §7.4.1, §7.8.2.

Two `property S.p means/equals` blocks parameterized over different modes
(e.g. `let M be UnitalMagma` and `let M be Group`) can both apply to one
value. Each carries existence/uniqueness relative to its own mode, but
nothing relates the two definientia; if they disagree on a shared instance,
their uniqueness axioms prove a contradiction. `redefine` solved the same
problem with a mandatory coherence obligation (§19.5); property
implementations have no analogue.

**Proposed resolution:** require any two property implementations of the
same struct property with overlapping domains to be related by a coherence
obligation, or restrict each property to at most one implementation per
`inherit`-connected mode family.

### SSA-006 (high, `design_drift`) — Registration activation timing

**Where:** 17.clusters_and_registrations.md §17.1 vs architecture 04
"Registration Databases Separate Pending and Activated Registrations";
todo.md task 19.

The spec promises item-ordered activation: a registration is usable by
subsequent items of the same module once "its own correctness condition has
been accepted". Architecture 04 (and the implemented task-19 interim policy)
says a local registration must **not** be used later in the same unchecked
pass and activates only after the configured verifier policy accepts the
obligations — which happens in phases that do not exist yet. Under the spec,
`fail_mode_existential_after_declaration_001` is the only ordering error a
user sees; under the interim policy, even a *preceding* local registration
does not license a mode declaration within the same pass, so currently-legal
modules would be rejected.

**Proposed resolution:** keep §17.1 as the language contract, and state in
§17.1 explicitly that acceptance of the correctness condition may be
asynchronous: an implementation may hold the module in a pending state, but
it must not *reject* a use site that a completed verification pass would
accept. Record in `registration_resolution.md` that the interim policy is a
conservative approximation to be lifted when `mizar-vc`/`mizar-proof` land.

### SSA-007 (medium, `spec_gap`) — Termination of cluster closure leans on the adjective grammar

**Where:** 17.10 `adjective`, 19.2.1, 3.3 `attribute_ref`.

§19.2.1 argues the closure fixpoint exists "since the attribute set is
finite". That is true only because the cluster grammar restricts adjectives
to `[non] [param_prefix] attribute_name` with `parameter ::= identifier |
numeral` — consequents cannot form new parameter terms (`(n+1)-dim`), and
functorial clusters attach facts to existing terms without creating terms.
The spec never states this restriction is load-bearing. If the argument-list
attribute form of §3.3 (`attribute_name(args)`) were ever admitted into
cluster consequents, the fact space becomes term-indexed and the naive
fixpoint may diverge; architecture 04's "saturation limits" would silently
become semantics.

**Proposed resolution:** state in §17.7.1 that termination follows from the
adjective grammar (finite attribute vocabulary over let-bound parameters),
and that any future extension of adjectives to term arguments requires a new
termination argument. Keep architecture 04's saturation bound as a defensive
diagnostic, not a semantic device.

### SSA-008 (medium, `spec_gap`) — Where are contradictory derived attributes detected?

**Where:** 17.7 (no ATP call) vs 17.7.3 ("detected during ATP resolution");
architecture 04 diagnostics table ("contradictory derived attributes" during
resolution).

§17.7 fixes cluster resolution as an ATP-free graph traversal, then §17.7.3
describes contradictions as something detected "during ATP resolution". The
checker needs a definite answer: closure-time detection (fact set contains
`A` and `non A` for one subject) is decidable and should be the specified
trigger; ATP-time inconsistency is a distinct, later failure.

**Proposed resolution:** specify closure-time detection as a fatal
`cluster` diagnostic (matching §17.7.3's severity), and reword §17.7.3 to
cover the residual ATP-visible inconsistencies separately.
**Corpus:** `fail_cluster_contradictory_consequent_001` pins the static
single-registration case.

### SSA-009 (medium, `spec_gap`) — Reduction determinism vs `such` side conditions

**Where:** 17.6.4 "Deterministic normalization" and "Matching" row.

Normalization is declared "a deterministic function of the term and the set
of in-scope rules", but the matching row makes a rule's applicability depend
on whether a `such` condition "is already available as a recorded local fact
or cited fact" — i.e. on the local proof context. Two sites with the same
term and rules but different local facts normalize differently, and the
stated function signature is wrong. Additionally, "specificity over the
whole matching constraint" combines pattern subsumption with §19.2.3 type
specificity without defining the product order when the two disagree.

**Proposed resolution:** restate determinism as a function of (term,
in-scope rules, **discharged side-condition set**); define the combined
specificity as: pattern subsumption first, then position-wise guard
comparison, all remaining mixed cases incomparable → FQN tie-break.

### SSA-010 (medium, resolved `spec_gap`) — Equally specific distinct roots

**Where:** 19.2.3 note, 19.4.3, 19.4.4, 19.1 restrictions.

Before task 37, two roots whose concrete normalized parameter types had
identical closures were comparable both ways, so the call site had no
unique best root, but §19.4.4 only named ambiguity for **incomparable** roots.
Relatedly, two ordinary definitions with identical argument signatures and
identical return types were neither a stated definition conflict nor a
resolvable overload.

Task 37 extends ambiguity to the case where at least one viable root exists
but the post-tie-break maximal-root set contains two or more distinct
ordinary roots. This covers both incomparable roots and closure-equivalent
roots. It also extends the §19.1 conflict rule: two ordinary definitions with
the same name and identical argument-type signature are a definition conflict
regardless of return type.

**Corpus:** `fail_overload_equivalent_roots_ambiguity_001` is the inactive
advanced-semantics seed for equivalent-root ambiguity.
`fail_resolve_same_signature_return_conflict_001` remains the active
different-return declaration conflict seed, and
`fail_resolve_same_signature_same_return_conflict_001` is an inactive
declaration-symbol seed until the resolver diagnostic covers same-return
duplicates.

### SSA-011 (medium, `spec_gap`) — Unique "path" vs unique "embedding"

**Where:** 5.4 vs 19.2.2/19.6.2.

§5.4 auto-confirms diamond consistency when `from` chains for same-typed
members coincide, but §19.2.2 blocks implicit upcasting whenever **two or
more syntactic paths** exist — even when every member embedding coincides,
so the upcast is semantically unique. The spec should say whether path
identity is syntactic (declaration pairs) or semantic (member embedding).
Task 36 records the syntactic choice in spec 19. A path is unique only when
there is one resolved `inherit` declaration path. Coherent member joins do not
collapse multiple reduct/view paths for overload resolution, so an explicit
`qua` is required when two or more paths exist.

**Corpus:** `fail_overload_inheritance_path_ambiguity_001` pins the syntactic
behavior test-first.

### SSA-012 (medium, `spec_gap`) — Inheritance acyclicity never stated

**Where:** 5.3; 13.8.7 (cycle-freedom axioms assumed).

The `inherit` closure must be well-founded — §13.8.7's qua encoding assumes
"cycle freedom" — but Chapter 5 never forbids `inherit A extends B; inherit
B extends A;` nor names a diagnostic. Task 36 adds an explicit acyclicity
requirement to §5.3 and names diagnostic detail key
`structures.inherit.cycle`.
**Corpus:** `fail_structure_inherit_cycle_001`.

### SSA-013 (medium, `spec_gap`) — `sethood` for dependent modes

**Where:** 7.8.1.

The obligation table gives only the unparameterized form
`∃S. ∀x. (is_T(x) → x ∈ S)`. For `Subset of X`-style modes the intended
obligation is presumably `∀params. ∃S. ∀x. (is_T(x, params) → x ∈ S)`, and
the comprehension gate of §13.4.2 must check sethood *at the instantiated
parameters*. Not stated; neither is whether sethood is part of the exported
module interface.

### SSA-014 (medium, `spec_gap`) — Existence for unattributed bases and built-ins

**Where:** 7.8 vs 17.3.4.

§7.8 says a mode declaration is a hard error "if no existential registration
is found for the base type", while §17.3.4 requires registrations only for
**attributed** types. Which governs `mode M is set` (no attributes)? Are
`object`, `set`, and struct radixes implicitly inhabited (structs are
FOL-consistent to leave uninhabited)? The checker's existential gate
(todo task 20) needs the built-in inhabitation table spelled out.

### SSA-015 (medium, `spec_gap`) — `reconsider` without justification

**Where:** 8.2 EBNF ("can be omitted entirely"), 8.2.2.

If the justification is omitted, the spec does not say whether the narrowing
obligation is discharged by cluster closure only, sent to ATP, or an error.
**Proposed resolution:** omitted justification is legal iff the obligation
is discharged by widening/closure evidence alone; otherwise a diagnostic
requests a justification.

### SSA-016 (low, resolved `spec_gap`) — Antisymmetry wording

Before task 37, §19.2.3 called `⊑` "antisymmetric", but two syntactically
distinct concrete normalized type expressions with equal closures satisfy
`T₁ ⊑ T₂ ⊑ T₁`. Task 37 rewords specificity as a preorder on concrete
normalized type expressions; antisymmetry holds only after quotienting by
closure-equivalence classes. This wording is part of the SSA-010 ambiguity
decision.

### SSA-017 (low, `spec_gap`) — Ambiguous `coherence with` omission

§19.4.1 assigns a `redefine` without `coherence with` to "the unique earlier
definition whose signature it sharpens". When several qualify, no error name
or behavior is given. Specify an "ambiguous redefinition target" diagnostic.

### SSA-018 (low, `design_drift`) — Greedy `of`/`over` parse depends on scope

§19.6.4's longest-match rule makes the parse tree of `M of A, B` depend on
which arities are visible, so adding an import can reparse existing text.
Documented and deterministic, but fragile; recommend a lint when a lower
arity interpretation also exists, and note the parser needs arity data from
the resolver (layering).

### SSA-019 (low, resolved editorial) — Duplicated sentences

Task 37 removes the repeated §19.6.1 introductory sentences while updating
the template tie-break examples.

### SSA-020 (medium, `spec_gap`) — `attr(args)` usable but not declarable

`attribute_ref` (§3.3, §6.9) admits `attribute_name "(" argument_list ")"`,
but §6.2 only defines declarations with hyphen `param_prefix` parameters,
and the cluster `adjective` grammar (§17.10) excludes the argument-list form
entirely. Either define the declaration and registration story for
argument-list attributes or remove the form from `attribute_ref`. Interacts
with SSA-007 (admitting it into clusters breaks the termination argument).

## Adversarial Corpus

The original audit fixed sixteen rejection fixtures test-first (sidecars +
traceability entries); the `advanced_semantics` fixtures remain inactive until
an advanced runner and source-to-checker payload extraction exist —
MC-G020/MC-G021/MC-G023/MC-G027. Task 35 later adds the SSA-001
constructor-property seed under the same inactive advanced-semantics rule.
Task 36 adds the duplicate-member-coverage seed under that rule, while
renamed-view exposure remains positive behavior guarded by the existing
template view-leak seed. Task 37 adds inactive ordinary and template-derived
equivalent-root ambiguity seeds plus one inactive same-return signature-conflict
declaration seed; the latter waits for the matching resolver diagnostic. The
existing different-return signature-conflict declaration seed remains active.
Existing tests and expectations were not rebaselined to match implementation
behavior.

| Fixture | Target behavior | Spec |
|---|---|---|
| `fail/clusters/fail_cluster_reduce_cycle_orientation_001` | reduce cycle unregistrable (size order) | 17.6.4 |
| `fail/clusters/fail_cluster_reduce_commutative_orientation_001` | same-size orientation rejected | 17.6.4 |
| `fail/clusters/fail_cluster_reduce_fresh_variable_001` | fresh RHS variable rejected | 17.6.4 r1 |
| `fail/clusters/fail_cluster_reduce_duplicating_variable_001` | RHS occurrence increase rejected | 17.6.4 r2 |
| `fail/clusters/fail_cluster_contradictory_consequent_001` | contradictory consequent adjectives | 17.4, 17.7.3 |
| `fail/modes/fail_mode_missing_existential_001` | attributed type without existential evidence | 17.3.4, 7.8 |
| `fail/modes/fail_mode_existential_after_declaration_001` | activation is item-ordered, not retroactive | 17.1, 7.8 |
| `fail/structures/fail_structure_diamond_member_type_conflict_001` | incompatible joined member types under root+path/view identity | 5.3.1, 5.4 |
| `fail/structures/fail_structure_inherit_duplicate_member_coverage_001` | duplicate parent member coverage | 5.3.1 |
| `fail/structures/fail_structure_inherit_cycle_001` | inheritance cycle | 5.3, 13.8.7 |
| `fail/structures/fail_structure_inherit_uncovered_member_001` | uncovered base member | 5.3.1 |
| `fail/overload/fail_overload_incomparable_roots_001` | incomparable roots → ambiguity | 19.2.3, 19.4.4 |
| `fail/overload/fail_overload_equivalent_roots_ambiguity_001` | equivalent distinct roots → ambiguity | 19.2.3, 19.4.4 |
| `fail/overload/fail_overload_template_equivalent_roots_ambiguity_001` | equivalent template-derived roots → ambiguity | 19.4.4, 19.6.1 |
| `fail/overload/fail_overload_inheritance_path_ambiguity_001` | multi-path upcast needs `qua` | 19.2.2, 19.6.2 |
| `fail/resolve/fail_resolve_same_signature_return_conflict_001` | same signature, different return | 19.1 |
| `fail/resolve/fail_resolve_same_signature_same_return_conflict_001` | same signature, same return | 19.1 |
| `fail/types/fail_types_qua_narrowing_001` | `qua` narrowing rejected | 13.6.4, 8.2.2 |
| `fail/types/fail_types_qua_unrelated_struct_001` | `qua` to unrelated struct rejected | 13.6.1, 13.6.4 |
| `fail/types/fail_types_comprehension_missing_sethood_001` | Fraenkel without sethood rejected | 13.4.2, 7.8.1 |
| `fail/structures/fail_structure_constructor_property_arg_001` | constructor property argument rejected | 5.5.1, 5.8.4, 7.4.1 |

New traceability requirements: `spec.en.05.structures.constructor_fields_only.semantic`,
`spec.en.05.structures.inheritance.semantic`,
`spec.en.07.modes.property_implementation.not_constructor_source.semantic`,
`spec.en.07.modes.existential_gating.semantic`,
`spec.en.13.qua.widening_only.semantic`,
`spec.en.13.sethood.comprehension.semantic`,
`spec.en.17.clusters.pattern_consistency.semantic`,
`spec.en.17.reductions.termination_order.semantic`,
`spec.en.19.overload.ambiguity.semantic`,
`spec.en.19.overload.definition_conflict.declaration`,
`spec.en.19.overload.definition_conflict.same_return.declaration`.

## Impact on the mizar-checker TODO

This section records how `todo.md` and crate plans should remain aligned as
spec-decision tasks close.

- **Resolved spec tasks:** SSA-001 (constructor/extensionality) is resolved by
  task 35 with synchronized `doc/spec/en/` + `ja/` edits and an inactive
  reject-first corpus seed. SSA-002+SSA-011+SSA-012 are resolved by task 36
  with synchronized spec 05/19 edits, a duplicate-member-coverage seed, and
  traceability note updates; no renamed-view reject seed was required.
  SSA-003+SSA-010+SSA-016+SSA-019 are resolved by task 37 with synchronized
  spec 19 edits, overload design sync, and equivalent-root / same-signature
  corpus seeds.
- **Remaining spec tasks (before further checker semantics):** one task each
  for SSA-004 (functorial `for` semantics) and SSA-005 (property
  implementation coherence), each updating `doc/spec/en/` + `ja/` together.
- **Task 19/20 (registration gating, existential gates):** revisit against
  SSA-006's activation contract and SSA-014's built-in inhabitation table
  once decided; the interim conservative policy should be recorded as such
  in `registration_resolution.md`.
- **Tasks 16-18 (closure, loops, reductions):** encode SSA-007's grammar-
  based termination argument and SSA-008's closure-time contradiction rule in
  `cluster_trace.md`/`registration_resolution.md`; reduction determinism
  needs SSA-009's corrected function signature.
- **Tasks 23-26 (templates, viability, selection):** task 37 records the
  Phase B tie-break and tie-ambiguity rules. Real payload work must still not
  infer missing comparison evidence; `mizar-core` task 26 / F7 records the
  separate Phase A omitted-template-argument inference determinism rule.
- **Task 29 corpus records:** the two deferred advanced_semantics corpus
  requirements now have concrete sibling seeds; when the runner lands, the
  deferred records should be revised to point at (or be superseded by) the
  applicable audit requirement ids above, including the overload ambiguity
  row and the deferred same-return declaration-conflict row added by task 37.
