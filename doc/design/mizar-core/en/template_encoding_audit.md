# Template Logic Encoding Soundness Audit (Spec Chapter 18, §18.10)

> Canonical language: English. Japanese companion:
> [../ja/template_encoding_audit.md](../ja/template_encoding_audit.md).

## Purpose

This document is a logic-level soundness audit of the template mechanism's
first-order encoding, as specified in
[doc/spec/en/18.templates.md](../../../spec/en/18.templates.md), with §18.10
(Logic Encoding Details) as the primary target. Templates admit second-order
patterns (schemes, type parameters, predicate/functor parameters) and lower
them to first-order axiom generation at use sites. The audit question is:

> Does there exist an instantiation that passes the specified encoding rules
> but generates a first-order axiom that is false in the intended set-theoretic
> semantics?

For each encoding rule the audit (a) reconstructs the argument for why the
rule is sound, and (b) where the argument fails or the rule is ambiguous,
gives a concrete counterexample candidate as a `.miz` fragment. Interactions
audited, per the task scope:

- Chapter 3 type erasure and widening;
- Chapter 5 structure encoding (reached as the root cause of a critical
  finding);
- Chapter 7 `sethood` obligations for parameterized modes;
- Chapter 13 choice (`the T`), Fraenkel comprehension, and `qua` encoding;
- §18.9 scheme application and §18.2.7 parameter inference determinism;
- Chapter 20 algorithm templates (§18.8.4).

This is an audit artifact. Spec amendments made in the same change are listed
per finding under **Disposition**; everything else is recorded for follow-up
tasks (`mizar-core` elaborator todo.md revision is deliberately deferred).

## Method and Soundness Baseline

The template system's global soundness argument has the following shape
(reconstructed from §18.10.1):

1. **Symbolic verification.** A template body is verified once, at declaration
   time, in a context Γ_schema containing: uninterpreted symbols for each
   schema parameter (`is_T/1`, `P/n`, `F/n`), declared guards, and `such that`
   constraints as hypotheses.
2. **Instantiation as substitution.** A use site substitutes actuals for the
   uninterpreted symbols and emits the substituted axioms/theorem instances.
3. **Substitution lemma.** If Γ_schema ⊢ φ in classical FOL with equality,
   and σ maps each uninterpreted symbol to a definable interpretation such
   that σ(Γ_schema) holds, then σ(φ) holds.

Step 3 is a standard metatheorem, so the encoding is sound **iff** every
assumption present in Γ_schema is actually guaranteed (proved, checked, or
carried as an antecedent) at every instantiation, and the substitution is a
genuine capture-avoiding first-order substitution. Every finding below is a
place where the specification either (i) lets Γ_schema silently contain an
assumption that instantiation does not guarantee, (ii) leaves the substitution
itself ill-defined, or (iii) makes two instantiations indistinguishable in the
generated FOL although they are mathematically distinct.

Severity scale: **Critical** = a false axiom is derivable under a natural
reading of the current text; **Major** = the rule is ill-defined and at least
one plausible implementation reading is unsound; **Medium** = missing rule
with soundness-relevant consequences in composition; **Minor** = defect in
examples or non-normative text.

## Per-Rule Soundness Reconstruction

### §18.10.1 Templates as axiom schemas

**Why sound (as intended):** verify-once + substitution lemma, as above. All
proof obligations inside the body (existence/uniqueness for `func` items, mode
existence, sethood, correctness conditions of registrations) are discharged
relative to the parameter guards; instances inherit them under substitution.
**Holds only conditionally:** see F1–F5 — several Γ_schema assumptions are not
tracked to instantiation by the current text.

### §18.10.2 Type parameter encoding

**Why sound (bare `let T be type`):** `is_T/1` uninterpreted, nothing assumed
about it; any type expression σ(is_T) := λx. is_σT(x) is a definable predicate
and the substitution lemma applies. This is sound **provided** Γ_schema does
not assume inhabitation or sethood of `T` (F2, F5) and provided value-level
actuals are excluded for bare type parameters (F3).

**Why unsound as written (`let T be type extends M`):** the table encodes the
parameter as a schema predicate `is_T/1` with guard `∀x. is_T(x) → is_M(x)`,
but §18.2.2 instantiates the same parameter with an **object-level instance
view** (`PermProduct[R qua AddMagma]`). Substituting a term for a unary
predicate symbol is not defined, and the body's uses of `T` as a structure
(`T.binop`, remapped notation `*`, `Element of T`, `FinSequence of T`) have no
FOL rule at all in the table. See F3; view identity issues are F1.

### §18.10.3 Predicate parameter encoding

**Why sound:** `P/n` uninterpreted; instances substitute a `defpred` closure
by capture-avoiding expansion (§15.11.3), which is first-order definable, so
each instance is an ordinary FOL formula and ZF separation/replacement cover
any comprehension the body performs over `P`. The ban on inline λ arguments
(§18.9) keeps actuals resolvable and definable. Residual gaps: signature
compatibility of the actual (F4/F6 companion rule) and schemas applied inside
template bodies with the enclosing schema's parameters as actuals (F6).

### §18.10.4 Functor parameter encoding

**Why sound (intended reading):** `F/n` uninterpreted with typing guard
`∀x. is_T(x) → is_S(F(x))` **assumed** during symbolic verification and
**re-established** at instantiation from the actual's declared signature.
**Why ambiguous as written:** the table calls the guard an axiom attached to
`F`, and §18.10.1 says instantiation "generates axioms"; under that reading
the instantiated guard is asserted, not proved — a false axiom for any actual
whose result type does not land in `S` over all of `T`. See F4.

### §18.10.5 Constraint encoding

**Why sound:** `such that φ` is assumed in Γ_schema, proved at each use site,
and carried as an antecedent in every generated axiom, so instances where the
caller's proof is wrong are unprovable rather than unsound. Theorem hypotheses
encode as ordinary antecedents. No counterexample found. Note that all
correctness obligations discharged inside the body (mode existence, functor
existence/uniqueness, sethood) must inherit the constraint antecedent in their
exported instances; the text says this ("acts as antecedent in all generated
axioms") and the audit confirms it suffices.

### Interaction reconstructions with no finding

- **Choice stability (§13.5):** `the T` lowers to `choice_T(params)` per
  owning core item **after template instantiation**, so distinct instantiations
  get distinct choice terms (params include the instantiated parameters) and
  repeated occurrences within one instance share one term. Sound as a Hilbert
  ε with per-instance obligations — provided the non-emptiness obligation
  itself is handled per F2.
- **Registrations in template blocks:** conditional clusters are guarded
  universal theorems; instances follow by substitution. Cluster-consistency
  and inherit-acyclicity checks (§13.8.7) are checks, not axioms, so template
  instances cannot smuggle in contradictions through them.
- **Algorithm templates (§18.8.4, Ch.20):** the VC schema (§20.13.3) is
  discharged symbolically with `F`, `P` uninterpreted; every VC provable with
  uninterpreted symbols is provable for every instance (substitution lemma),
  so symbolic verification is conservative. Termination measures must be
  `Nat`-valued via the functor guard, and a measure depending on `F` is only
  accepted if provable from guards — conservative, hence sound. Promotion
  axioms (§20.13.2) are per-instance substitutions of verified contracts.
  Runtime verification failure for non-computable actuals (§20.9.3) is
  execution-side only and does not weaken verification-side soundness.
- **Type erasure (§3.7.3):** template instances are generated in already
  guard-based form, so ATP export erasure introduces no additional template
  hazard beyond F1.

## Findings

### F1 (Critical) — Structure-view collapse: diamond inheritance with field remapping is inconsistent under the flattened encoding, and template `qua` instantiation sits on it

**Rules involved:** §5.8.3, §5.8.5, §13.8.7 ("the term `x` itself is unchanged
in the FOL encoding"), §18.2.2 (notation remapping, `qua` actuals), §18.10.2.

**Defect.** §5.8.3 encodes `inherit D extends B where field d from b` as
subsumption plus **global selector equalities**:
`∀x. is_D(x) → is_B(x)` and `∀x. is_D(x) → d(x) = b(x)`. With the
diamond that Chapter 18 itself uses as its running example (Ring reaches Magma
via AddGroup and via MulMonoid, with `field add from binop` on one path and
`field mul from binop` on the other), any Ring `R` yields:

```
add(R) = binop(R)        (via is_AddGroup(R))
mul(R) = binop(R)        (via is_MulMonoid(R))
⟹ add(R) = mul(R)
```

For any concrete ring where `1+1 ≠ 1*1`, this derives ⊥ inside the kernel
theory — no template needed. Independently, §5.8.5 extensionality is stated
over `is_S`, which by subsumption holds for richer descendants: two
`ZeroOneStr` values agreeing on `carrier` and `zero` but differing on `one`
are forced equal by `ZeroStr`'s extensionality — ⊥ again.

**Template-level counterexample (the encoding-passes-but-unsound instance
this audit was asked to hunt).** Because §13.8.7 erases `qua` ("x unchanged"),
attribute atoms on different views of the same term are the **same FOL atom**:
`R qua AddMagma is commutative` and `R qua MulMagma is commutative` both
lower to `is_commutative(R)`. Take `R` a matrix ring (commutative under `+`,
not under `*`):

```mizar
Comm: R qua AddMagma is commutative by mml.algebra.matrix.add_comm;
:: Bound check for `let T be type extends commutative Magma`
:: is discharged from the atom is_commutative(R) — proved for the ADD view:
thus Product[R qua MulMagma](s) = Product[R qua MulMagma](s * p)
  by PermProduct[R qua MulMagma], Comm;   :: MUST BE REJECTED
```

The generated instance asserts permutation invariance of **matrix products**
— false. The instantiation passed every rule in the current text: the bound
`inherit*` obligation holds, and the `commutative` attribute obligation is
discharged by an atom that the encoding cannot distinguish per view.

**Why the rule cannot be repaired locally.** Field renaming makes "one term,
many views" unrepresentable with shared global selectors: the views differ in
which selector realizes `binop`, so the view must be a **term**, not a
meta-annotation.

**Required encoding (adopted in the spec patch).** Reduct (view) terms:

- An `inherit` edge whose field mapping is the identity and whose ancestor is
  reached by exactly one path keeps subsumption encoding.
- Any edge with renaming, and any ancestor reachable via multiple paths,
  encodes as an explicit reduct function `view_{D→B}` with
  `∀x. is_D(x) → is_B(view_{D→B}(x))` and
  `∀x. is_D(x) → b(view_{D→B}(x)) = d(x)` per mapped field; **no**
  `is_D → is_B` subsumption is emitted on such edges.
- `x qua B` along such a path lowers to the (composed) reduct term, so
  attribute atoms become `is_commutative(view_add(R))` vs
  `is_commutative(view_mul(R))` — distinct, as required.
- Extensionality is restricted to **exact** instances (aggregate-typed
  values), not `is_S` at large.
- Bounded-type-parameter instantiation `PermProduct[R qua AddMagma]`
  instantiates the parameter with the reduct term; notation remapping
  (§18.2.2) is then a theorem (`binop(view(R)) = add(R)`), not a convention.

**Disposition.** Spec patched in this change: §5.8.3, §5.8.5 (encoding),
§13.8.7 (qua lowering), §3.7.2 (subtyping-note), §18.10.2 (instantiation via
reducts). Reject-first test added:
`tests/miz/fail/templates/fail_template_qua_view_attribute_leak_001.miz`.
Elaborator/kernel impact recorded in "Impact on mizar-core" below.

### F2 (Critical) — Inhabitation of type parameters vs. empty type-expression actuals

**Rules involved:** §18.10.2, §18.2.2 (actual classification), §7.8 ("all
types in the system are non-empty"), §17.3.4 (existential gating table),
§13.5 (`the T`).

**Defect.** The system maintains a global invariant that every type in use is
inhabited (mode existence is a hard error otherwise, §7.8; attributed types
are gated on existential registrations, §17.3.4). Inside a template body this
invariant is what licenses `the T`, `consider`/`take` over `T`, and mode
existence proofs that use a parameter's inhabitant. But:

1. §18.10.2 does not say whether Γ_schema contains `∃x. is_T(x)` for a type
   parameter; and
2. §17.3.4's gating table does **not** include "type expression used as a
   template actual", and §18.2.2 accepts any `type_expression` as an actual.

If an implementation honors the §7.8 invariant symbolically (the natural
reading — otherwise `the T` never verifies in any template), then the
following passes every stated rule:

```mizar
definition
  let x be set;
  attr HollowDef: x is hollow means not x = x;   :: unsatisfiable, never registered
end;

definition
  let T be type;
  theorem Inhab[T]: ex y being object st y is T
  proof
    take the T;      :: discharged from the types-are-inhabited invariant
    thus thesis;
  end;
end;

theorem Boom: ex y being object st y is hollow set
  by Inhab[hollow set];   :: MUST BE REJECTED — generates ∃y. is_set(y) ∧ y ≠ y
```

`hollow set` never appears in a variable declaration, mode base, or functor
return, so no rule in §17.3.4 fires; the instance is a false axiom.

**Sound design (adopted).** Keep the invariant and close the gate:
Γ_schema **may** assume `∃x. is_T(x)` for each type parameter, **because**
instantiation is required to present inhabitation evidence for every type
actual — the same evidence §17.3.4 already demands elsewhere (mode existence
per §7.8, existential registration for attribute chains, bound-mode existence
for `type extends M`). This matches classical Mizar's non-empty-type regime
and keeps `the T` usable in template bodies.

**Disposition.** Spec patched in this change: §17.3.4 (new gating row),
§18.10.2 (normative inhabitation paragraph), §18.2.2 (actual-side sentence).
Reject-first test added:
`tests/miz/fail/templates/fail_template_type_actual_missing_existential_001.miz`.

### F3 (Major) — `type extends M` conflates schema-predicate and object-level encodings; body-level structure operations have no FOL rule

**Rules involved:** §18.10.2 (table), §18.2.2.

**Defect.** As reconstructed above, the `is_T/1`-with-guard encoding cannot
absorb instance-view actuals (`R qua AddMagma`) and gives no meaning to
`T.binop`, remapped notation, `Element of T`, or `FinSequence of T` in the
body. An implementation following the table literally would either reject the
chapter's own examples or improvise an unsound bridge (e.g., substituting the
carrier predicate of *some* view — precisely the F1 leak).

**Required rule (adopted).** A structure-bounded parameter
`let T be type extends M` is an **object-level schema constant** `t`:

- Γ_schema guard: `is_M(t)` (plus the bound's attribute atoms on `t`), with
  `M`'s fields available as `field(t)`;
- body occurrences of `T` in type positions mean `Element of t` (encoded via
  the carrier of `t`), and `T.f`/remapped notation means `f(t)`;
- actual = mode/struct name `N`: the instance is the **universal closure**
  `∀t. is_N(t) ∧ bound-attributes(t) → φ(t)` (after checking `N`'s instances
  reach `M`, i.e. `inherit*(N, M)`);
- actual = `v qua N…` view: the instance is `φ(viewpath(v))` with the F1
  reduct term, after the §13.8.7 validity obligations and the bound's
  attribute obligations **stated on the view term**;
- bare `let T be type` parameters remain schema predicate variables and
  accept only `type_expression` actuals — `qua_arg` actuals are only valid
  for structure-bounded parameters.

**Disposition.** Spec patched in this change: §18.10.2 table and notes.
Covered jointly by the F1 test (view obligations) and F2 test (actual
classification); no separate test.

### F4 (Major) — Functor-parameter guard: axiom vs. obligation, and missing signature-compatibility rule for `defpred`/`deffunc` actuals

**Rules involved:** §18.10.4, §18.9, §15.11.3.

**Defect.** If the instantiated guard `∀x. is_T(x) → is_S(F(x))` is emitted
as an axiom (the table's reading), this passes:

```mizar
deffunc shrink(x be Nat) -> Integer equals x - 5;
:: schema expects func(Nat) -> Nat
thus ... by IterBound[shrink], ...;
:: instantiated "guard axiom": ∀x. is_Nat(x) → is_Nat(x - 5)   — FALSE at x = 0
```

Similarly nothing requires a `defpred` actual's parameter types to widen from
the schema's declared `pred(T₁,…,Tₙ)` domain; a narrower actual
(`defpred P(p be Prime)` for `pred(Nat)`) makes the instance apply the
expansion outside its checked domain (§15.11.3's per-application check fires
only if the elaborator re-checks every occurrence post-substitution, which
§18.10 does not require).

**Required rule (adopted).** At instantiation, for each functor parameter
`func(T₁,…,Tₙ) -> S` with actual `f`, and each predicate parameter
`pred(T₁,…,Tₙ)` with actual `p`: every schema domain type `Tᵢ` must **widen
to** the actual's declared parameter type (contravariant), and the actual's
declared result type must **widen to** `S` (covariant). The guard is thereby
a discharged proof obligation, never an assumed axiom; a template that needs
a stronger property of `F` must state it as a hypothesis (as the §18.10.4
example already does).

**Disposition.** Spec patched in this change: §18.10.4, §18.9 note.
Reject-first test added:
`tests/miz/fail/templates/fail_template_func_actual_result_widening_001.miz`.

### F5 (Major) — Sethood of type parameters unspecified; Fraenkel comprehension in template bodies can reach Russell

**Rules involved:** §18.10.2, §13.4.2, §7.8.1.

**Defect.** No rule assigns or denies `sethood` evidence to a type parameter.
§13.4.2 requires proved sethood for a comprehension generator; if an
implementation treats "T is a type parameter" as sufficient (e.g. by analogy
with the inhabitation invariant), then:

```mizar
definition
  let T be type;
  func para[T] -> set equals { x where x is T : not x in x };
end;
set r = para[set];    :: Russell: r in r iff not r in r
```

`set` itself has no sethood (the class of all sets is proper), so the
symbolic check must fail — the spec just never says with respect to what
evidence.

**Required rule (adopted).** A bare type parameter carries **no** sethood
evidence. A bounded parameter `type extends M` inherits sethood evidence iff
the bound `M` has a proved `sethood` (sound: `is_T ⊆ is_M ⊆ S`). Additional
sethood may be demanded as a `such that` constraint and is then discharged at
use sites like any constraint. Comprehension over a parameter is rejected
symbolically unless one of these sources applies.

**Disposition.** Spec patched in this change: §18.10.2 sethood paragraph.
Reject-first test added:
`tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz`.

### F6 (Medium) — Schemes applied inside template bodies with the enclosing template's parameters as actuals

**Rules involved:** §18.10.3 example (NatInduction's body cites
`mml.number.natural.Nat_induction` with the enclosing `P` in scope), §18.9.

**Defect.** The chapter's own examples pass an **uninterpreted** schema
parameter as (implicit) actual of another scheme, but no rule defines this:
scheme actuals are specified as `defpred`/`deffunc` identifiers only. The
sound semantics is substitution composition — instantiate the inner scheme
with the outer schema's uninterpreted symbol, symbolic obligations checked in
Γ_schema; final instances then compose substitutions. Without a stated rule,
an implementation might reject the chapter's examples or, worse, freshly
skolemize the parameter (breaking the binding between inner and outer `P`).

**Disposition.** Spec patched in this change: one normative paragraph in
§18.10.3. No test (pass-side behavior; reject corpus is out of scope for it).

### F7 (Medium) — §18.2.7 "uniquely inferred" is underdefined over the widening lattice

**Defect.** Inference of omitted `[T]` from argument types does not say
whether candidates are compared at the arguments' declared types or at any
widening (e.g. `Product(s)` for `s : FinSequence of Prime` admits `T := Prime`,
`Integer`, …). Any well-formed instance is *sound* (post-F1–F5), so this is
not a soundness hole, but the generated axiom differs per choice, making
verification outcomes implementation-defined. `qua` views are never inferred
(§18.2.7), which — importantly — keeps F1's view choice out of inference.

**Disposition.** Not patched here (needs an inference-algorithm decision owned
by the elaborator design); recorded for the todo.md revision task. Recommend:
infer at declared types after mode unfolding; residual multiple candidates are
an ambiguity error even when instances would be logically equivalent.

### F8 (Minor) — §18.8.4 example passes a schema functor as a first-order argument; partial algorithms as functor actuals unaddressed

**Defect.** `sigma[F]`'s contract reads `ensures result = Sigma(F, lo, hi)`,
passing the meta-level symbol `F` as a term argument to a first-order functor
— violating §18.2.4's own rule that functor parameters are not domain
elements. It must be a template instantiation `Sigma[F](lo, hi)`. Separately,
only `deffunc`, template functors, and **promoted `terminating` algorithms**
denote FOL function symbols; a partial (unpromoted) algorithm has no FOL
symbol and must be rejected as a `func(...)` actual — §18.8.4 only covers the
converse direction (non-computable but valid instantiations).

**Disposition.** Spec patched in this change: §18.8.4 example and a sentence
on valid functor actuals.

## Interaction Summary Against the Task Checklist

| Interaction | Verdict |
|---|---|
| Ch.3 erasure/widening | Sound given F1's reduct encoding; §3.7.2 note added. Erasure itself adds no template hazard. |
| Ch.7 sethood / proper classes from parameterized modes | Sound **iff** F5's evidence rule is adopted; template-declared `sethood` proved symbolically transfers to instances by substitution. |
| Ch.13 `the T` / non-emptiness | Sound **iff** F2's gate is adopted; choice-term identity per instantiation confirmed sound. |
| §18.9 scheme application / §18.2.7 inference | Sound modulo F4 (signature rule) and F6; F7 is determinism, not soundness. |
| Ch.20 algorithm templates | Symbolic VC discharge is conservative, hence sound; promotion per instance sound by substitution; F8 example defect. |

## Impact on mizar-core (elaborator) — input for the todo.md revision task

The following implementation obligations follow from the findings. **todo.md
itself is deliberately not revised in this change.**

1. **Reduct/view lowering (F1, F3).** Core IR needs view terms for `qua` on
   renamed/multi-path inherit edges and for bounded-type-parameter
   instantiation; attribute atoms and field selections must be emitted against
   view terms. Extensionality emission must switch to exact-instance guards.
   This touches the builtin type bridge and the typed-AST elaboration recently
   landed (`feat: bridge type elaboration to resolved typed ast`).
2. **Inhabitation gating of template actuals (F2).** The existential-gating
   check must run for template `type_expression` actuals; schema contexts get
   a per-parameter inhabitation fact.
3. **Signature-compatibility check for scheme actuals (F4/F6).** Contravariant
   domain / covariant codomain widening checks for `defpred`/`deffunc`
   actuals; guard obligations discharged, never asserted.
4. **Sethood evidence plumbing for type parameters (F5).** Fraenkel gating in
   template bodies keyed to bound-inherited or constraint-supplied sethood.
5. **Inference determinism decision (F7)** and rejection of partial-algorithm
   functor actuals (F8).

Kernel-side (from the Jul 3 kernel audit's perspective): the reduct encoding
changes the shape of certificates that mention structure widening; the
soundness-argument document's assumptions about atomic attribute predicates
should be revisited once F1's encoding lands.

## Test Deliverables (reject-first)

All under `tests/miz/fail/templates/`, as inactive `advanced_semantics` seeds
following the existing corpus conventions; no existing test or expectation was
modified:

| Test | Finding | Rejection |
|---|---|---|
| `fail_template_qua_view_attribute_leak_001` | F1 | attribute evidence for one structure view must not discharge a bound on another view |
| `fail_template_type_actual_missing_existential_001` | F2 | attributed type actual without existential registration |
| `fail_template_fraenkel_over_type_param_001` | F5 | comprehension generator ranging over a bare type parameter |
| `fail_template_func_actual_result_widening_001` | F4 | `deffunc` actual whose result type does not widen to the schema codomain |
