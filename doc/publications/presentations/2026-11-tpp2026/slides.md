# Mizar Evo: Design Principles

Status: English version aligned with the reviewed Japanese deck. Section 0 introduces the history and development; sections 1–6 keep the six design principles; section 7 covers implementation and plans.

Occasion: TPP 2026 (the 22nd Theorem Proving and Provers meeting), RIKEN AIP Tokyo Office, November 16–17, 2026. Assumed 45-minute slot: 36:30 for the talk, seven minutes for questions, and 1:30 of buffer. Confirm the official slot.

Content authority: `draft.ja.md`. This deck follows `slides.ja.md`; the English spoken script is `script.md`. Shared figures keep their English labels.

## Conventions

- Keep the structural labels `Title:`, `Subtitle:`, `Speaker note:`, and `Source:`, and code-status labels `exact MML excerpt`, `specification example`, and `sketch` for the generator.
- Bold text gives the main spoken points. Tables have titles and continuous numbers across the main talk and backups.
- Distinguish published history and the author's account. Technical examples describe existing Mizar, the Evo specification, or a sketch. Implementation claims use the fixed snapshot in 7.1. Mark research hypotheses and future directions explicitly.
- Use plain English (CEFR B1 to easy B2). Explain technical terms at first use. Build with `python3 build_beamer.py en`, then pdflatex; see README.

## Part 0. Introduction

### Frame 0.1 - Title

Title:

```text
Mizar Evo: Design Principles
Reconnecting automated proof, readable mathematics, and verified computation
```

Speaker note:

- **Mizar is a proof checker. It combines first-order set theory, readable mathematical language, and the MML library.**
- **Mizar Evolution, or Mizar Evo, redesigns the language and its tools. We begin with history and development, then discuss six design principles, implementation limits, and plans.**
- Distinguish facts, research hypotheses, and future directions throughout.

### Frame 0.2 - Mizar implementation history: from early systems to Mizar-rs and Evo

### Table 1. Early Mizar systems and the start of MML

| Period | Main milestones |
|---|---|
| 1973–75 | Mathematical documents → propositional Mizar-PC (Algol / ODRA) |
| 1977 | Mizar-QC: quantifiers (Pascal / CDC 6000) |
| 1982–84 | MSE: a smaller language. HPF: mathematical notation and implicit arguments |
| 1986 | Mizar-4: multiple passes; ported to PC / DOS at the end of the year |
| 1988–89 | PC-Mizar, references between articles; MML began on January 1, 1989 |

### Table 2. PC-Mizar, Mizar-rs, and Mizar Evo

| System | Overview |
|---|---|
| PC-Mizar (Pascal) | Parser → MSM → Analyzer → Checker |
| Mizar-rs (Rust, ITP 2023) | Reimplements the checker / analyzer, aiming at language compatibility |
| | **All MML checked:** replaced the checker and checked the full MML of that time |
| | **Speed:** about 5–6× faster (8 threads, elapsed time) |
| | **Bug reports:** four soundness bugs in the existing system; no false MML proofs using them |
| Mizar Evo (Rust) | Keeps mathematical writing and the logical foundation; builds a new language and processor |

Source: Matuszewski & Rudnicki (2005); Carneiro (ITP 2023), Table 1.

Speaker note:

- Mizar-PC was propositional; it differs from the later PC-Mizar. Readable notation, checking, and library reuse grew through several experiments and redesigns. These are selected milestones, not a single line of successors. Source details and URLs: `draft.ja.md`, history and development.
- Carneiro 2023 §1, footnote 2 dates the PC port to 1986; PC-Mizar dates to 1988. MSM means More Strict Mizar.
- Mizar-rs elapsed times: checker only, 57.37→11.33 minutes; analyzer plus checker, 71.38→11.71 minutes. Both affect soundness. Compatibility-related issues remain in §5.4 of the paper.

### Frame 0.3 - 2023–2026: from an idea to language design

### Table 3. Mizar Evo development

| Period | Development |
|---|---|
| August 2023 | First README: mizar-wishlist |
| September–October 2024 | Imports, namespaces, structure inheritance, type system |
| May–June 2025 | Modes, type inference, redefinition; dependent structures / attributes removed, then reintroduced |
| January–April 2026 | EBNF, logical encoding, registration, template and algorithm semantics / contracts |
| May 2026 | Implementation design → test harness → minimal lexer |

- **The idea began as an extension of Mizar notation. Preparing a 2023 grant proposal led to a redesign of the language and processor.**
- At ITP 2023, Mario Carneiro responded positively to a request for implementation help. At CICM 2023, I discussed the wishlist with Adam Naumowicz.
- Early priorities: generic types, namespaces / packages, ATP / traceability. The prototype was to use Mizar-rs.

JSPS KAKENHI, Scientific Research (C), awarded in 2024: 24K14897, “Renewing the language specification of a set-theory-based proof assistant” (title translated).

Speaker note:

- Source: August 2023 grant proposal and Git diffs. Details: `draft.ja.md`, history and development.
- The exact order of the consultations within 2023 and any current participation commitment are unconfirmed. Timeline dates are commit dates.

### Frame 0.4 - Design development and implementation with AI agents

### Developing the language design

- **template:** from generics to a common theorem form for schemes, with structure views and type-argument inference.
- **algorithm:** cryptographers in the Japanese Mizar community wanted proofs of algorithms from the start. Proving elliptic-curve associativity also required difficult, large symbolic calculations.

### Implementation and discussions since May 2026

- **May: started substantial Rust implementation with AI coding agents. June–July: crate development, autonomous workflows, SAT-backed checking and audits.**
- **Design → agent implementation → tests and audits → specification revision.**
- September: seminar and discussions with the Mizar team in Białystok.
- September–October: more source-derived VCs; revisions to algorithm partiality and contracts.

Speaker note:

- Source: specification chapters 18 and 20, Git diffs, and the author's additional account on October 15, 2026. Details: `draft.ja.md`.

### Frame 0.5 - Current Mizar: six challenges and design principles

### Table 4. Six challenges and design principles

| Area | Challenge in current Mizar | Design principle |
|---|---|---|
| **§1 Foundation** | Keep the logic and MML while replacing the processor | Keep FOL / set theory and a small kernel |
| **§2 Writing** | Trace implicit type, registration, and overload choices | Keep mathematical abstractions; explain implicit choices |
| **§3 Generics** | Separate mechanisms for definitions, theorems, and schemes | Unify generic mechanisms with templates |
| **§4 Computation** | Connect proofs to executable procedures | Check algorithm contracts, invariants, and termination |
| **§5 Tools** | Dependencies, distribution, incremental checking, IDE | Modules, namespaces, packages, incremental builds, LSP |
| **§6 Checking** | Connect external search to checking evidence | Use ATPs; instantiate evidence and check with SAT |

Speaker note:

- Each row connects a challenge to a principle; sections 1–6 follow this order. First-order foundations and readable mathematics are strengths to preserve.
- The closing status section distinguishes the specification from implemented behavior.
- Source: specification chapters 01, 18, 20, 21, 23; the problem-driven stories in the Białystok deck.

## Part 1. Logical Foundation

### Frame 1.1 - Keep the foundation, rebuild the tools

```text
**Keep the mathematical language.**
**Rebuild the processor and development tools.**
```

- **Keep first-order logic (FOL) and Tarski–Grothendieck set theory.**
- Keep soft types, modes, attributes, registrations, structures, and declarative proofs.
- **Separate ATP search from checking: a small kernel independently checks the results. Details in §6.**

Speaker note:

- Mathematical language and proof support can be built over a first-order foundation. Similar mechanisms do not imply identical logics or representations.
- Success rates and reconstruction failures do not establish a first-order performance advantage. Evaluate the design through mathematical writing, automation, and proof checking.

### Frame 1.2 - From sets to topological spaces

Example 1: Cantor's theorem in current Mizar (sketch, proof omitted):

```mizar
theorem Cantor:
  for X being set holds card X in card (bool X)
proof ... end;
```

Example 2: a continuous bijection from a compact space to a Hausdorff space (sketch, proof omitted):

```mizar
theorem CompactBijection:
  for S being compact TopSpace,
      T being non empty T_2 TopSpace,
      f being Function of S,T
  st f is continuous one-to-one & rng f = [#] T
  holds f is being_homeomorphism
proof ... end;
```

- **Write mathematics from sets, power sets, and cardinals to topological spaces over first-order set theory.**

Speaker note:

- Source: CARD_1:14 and COMPTS_1:17 (`draft.ja.md` §1). Spend about 30 seconds on each example, pointing out the meaning and language features. Proofs are omitted; these are explanatory sketches, not execution results.
- Cantor: bool X is the power set. Cardinals are initial ordinals, so in expresses a strict cardinal inequality here. Topology: T_2 means Hausdorff; rng f = [#] T means surjective; the conclusion states that f is a homeomorphism. Search and checking are separated in §6.

## Part 2. Readable Mathematics

### Frame 2.1 - Trace type, registration, and overload choices

Readable mathematics relies on many automatic decisions.

- **type: which types can this object be used as?**
  Follow attributes and inheritance. Example: Group → Magma, with a unique path.
- **registration: which rules supplied the required attributes?**
  Example: the chain empty → finite → countable.
- **overload: which definition and view were selected?**
  Distinguish definitions with the same notation, or addition and multiplication in a ring.

**Keep natural writing and automatic rules, while tracing the reasons for implicit choices.**

Speaker note:

- Source: `doc/spec/en/08.type_inference.md` §8.1–8.2; `17.clusters_and_registrations.md` §17.7; `19.overload_resolution.md` §19.1–19.3. New specification: type in 2.2–2.4, registration in 2.5, overload and explicit views in 2.6.

### Frame 2.2 - type: declare fields and properties

Structure declarations (specification example, inside a definition block):

```mizar
struct LoopStr where
  field carrier -> set;
  field binop -> BinOp of carrier;
  property unit -> Element of carrier;
end;

struct AddLoopStr where
  field carrier -> set;
  field add -> BinOp of carrier;
  property zero -> Element of carrier;
end;
```

**A field stores data. A property declares a name and type; its implementation is given separately.**

Speaker note:

- Source: `doc/spec/en/05.structures.md` §5.2, 5.5.1; `sample_codes.md`, LoopStr and AddLoopStr. These declarations are excerpts inside a definition block. Only fields are constructor arguments. Inheritance follows in 2.3, then the mode and unit implementation in 2.4.

### Frame 2.3 - type: declare inheritance after the structure

Add a parent mapping after declaring AddLoopStr (specification example):

```mizar
definition
  inherit AddLoopStr extends LoopStr where
    field carrier from carrier;
    field add from binop;
    property zero from unit;
  end;
end;
```

- **Separate structure declarations from inheritance. This resembles implementing a Rust trait after defining a type.**
- `from` renames a parent's role: `binop` becomes `add`, and `unit` becomes `zero`.
- One declaration per parent. Equal member types need no proof; narrower types require `coherence`.

Speaker note:

- Source: `doc/spec/en/05.structures.md` §5.3; `sample_codes.md`, AddLoopStr. The Rust analogy concerns the separation of declarations. In LoopStr, binop is a field and unit is a property.

### Frame 2.4 - type: implement unit for Unital, reuse it as zero

has_unit states that a two-sided identity exists; it does not depend on unit (sketch):

```mizar
definition
  mode UnitalDef: Unital is has_unit LoopStr;
  let M be Unital;
  property M.unit means
    for x being Element of M.carrier holds
      M.binop(it,x) = x & M.binop(x,it) = x;
  existence proof ... end;
  uniqueness proof ... end;
end;
```

- **If A's selected LoopStr view is Unital, reuse its unit implementation as zero.**
- `property zero from unit` gives `A.zero = (A qua LoopStr).unit`. No new zero implementation is needed.

Speaker note:

- Source: `doc/spec/en/07.modes.md` §7.4.1, 7.8.2; `05.structures.md` §5.3, 5.8.3. Sketch adapting the implementation example in chapter 7 to LoopStr. The has_unit definition is in `draft.ja.md`; proof bodies are omitted. Existence follows from the mode assumption, uniqueness by applying the operation to two identities.
- A is AddLoopStr, and the selected LoopStr view must satisfy Unital. Do not mix facts from different views. This example has no overlapping implementations; overlaps require coherence.

### Frame 2.5 - registration: automatic chains hide dependencies

**Automatic rules can form chains. The source at the point of use does not show all rules involved.**

```mizar
registration
  cluster EmptyImpliesFinite: empty -> finite for set;
  coherence proof ... end;
  cluster FiniteImpliesCountable: finite -> countable for set;
  coherence proof ... end;
end;
```

### Table 5. Registration paths recorded by the new specification

| Starting fact | Derived fact | Recorded rule |
|---|---|---|
| S is empty | S is finite | EmptyImpliesFinite |
| S is finite | S is countable | FiniteImpliesCountable |

- Declaring `empty` gives `finite` and then `countable` implicitly. No rule citation with `by` is needed.
- **New specification: keep automatic application, and record rule labels and application paths.**

Speaker note:

- Source: `doc/spec/en/17.clusters_and_registrations.md` §17.2, 17.4.1, 17.7; `23.package_management_and_build_system.md` §23.7.7. Labeled registration example from the new specification; the table is an explanatory trace for an empty set S.
- Emphasize that rules are not cited at the point of use and can apply in several stages. Attribute resolution takes place before ATP search.

### Frame 2.6 - overload: two identities in the same ring

**In the same ring R, the additive identity is 0 and the multiplicative identity is 1.**

- Additive AddMagma: `0 + x = x + 0 = x`
- Multiplicative MulMagma: `1 * x = x * 1 = x`

id. M returns the identity of a unital Magma M (sketch, definition omitted):

```mizar
let R be Ring;
id. R;                   :: ambiguous Magma view
id. (R qua AddMagma);     :: R.zero
id. (R qua MulMagma);     :: R.one
```

- Even with one definition of id., the Magma view of R is ambiguous.
- **Use qua to select the additive or multiplicative view and distinguish 0 from 1.**

Speaker note:

- Source: `doc/spec/en/sample_codes.md`, IdentityDef, LoopUnitalDef, RingLikeDef; `19.overload_resolution.md` §19.2.2–19.3.1; `05.structures.md` §5.8.3.
- Sketch limited to the identity functor id. Assume both views are unital, their identities correspond to R.zero / R.one, and each path to Magma is unique. Definitions: `draft.ja.md`.

## Part 3. Generic Mathematics

### Frame 3.1 - Current MML: more types, more redefinitions and registrations

**The sum of functions has one body, but preserving result types needs work for each type.**

### Table 6. Per-type declarations for function addition in VALUED_1

| Value type | Result-type redefinition | Attribute / totality registration |
|---|---|---|
| Complex | `PartFunc of C,COMPLEX` | `complex-valued` + `total` |
| Real | `PartFunc of C,REAL` | `real-valued` + `total` |
| Rational | `PartFunc of C,RAT` | `RAT-valued` + `total` |
| Integer | `PartFunc of C,INT` | `INT-valued` + `total` |
| Natural | `PartFunc of C,NAT` | `natural-valued` + `total` |

- **Each new type adds redefinitions, registrations, and coherence proofs.**
- We want one template for function addition and its result type.

Speaker note:

- Source: MML [VALUED_1](https://mizar.uwb.edu.pl/version/current/html/valued_1.html), def 1 and the following five groups of attribute registrations, PartFunc redefinitions, and total registrations. The table summarizes them; it is not a quotation. C is a common domain. Each row assumes the corresponding input value types; total registrations assume total functions and the required nonempty ranges. Each redefinition / registration has a coherence proof.
- The body of VALUED_1:def 1 is shared and uses the intersection of domains. The next template sketch is restricted to a common nonempty domain I.

### Frame 3.2 - template: share the body and result type

Generic functor (sketch, existence and uniqueness proofs omitted):

```mizar
definition
  let T be type extends non empty AddMagma;
  let I be non empty set;
  let f, g be Function of I, T.carrier;
  func AddDef: Add[T,I](f,g) -> Function of I,T.carrier means
    for i being Element of I holds it.i = T.add(f.i,g.i);
  synonym f +[T,I] g for Add[T,I](f,g);
end;
```

### Table 7. Template instances and result types

| Call (real R / complex C)[^1] | Result type |
|---|---|
| `f + g` | `Function of I,R.carrier` |
| `u + v` | `Function of I,C.carrier` |

- **Use the same Add[T,I] for rational, integer, and natural values. Share the body and result-type declaration.**

[^1]: Explicit forms: `f +[R,I] g`, `u +[C,I] v`. Omit arguments only when declared types determine them uniquely. Use `qua` for ambiguous inheritance paths.

Speaker note:

- Assume `let R be RealAdd; let C be ComplexAdd;` and the needed registrations. Each has a unique path to non empty AddMagma. f,g have type Function of I,R.carrier; u,v have type Function of I,C.carrier. Declared types determine T and I uniquely. Other number types need their additive structures, closure, and registrations. synonym supplies infix +.
- Source: `doc/spec/en/11.symbol_management.md` §11.1.2; `18.templates.md` §18.2.2, 18.2.7, 18.7; `19.overload_resolution.md` §19.6.2. A range set alone does not select the addition structure. qua views are not inferred automatically.

### Frame 3.3 - Unify schemes with template theorems [deep dive]

Induction as a theorem with a predicate parameter (specification example):

```mizar
definition
  let P be pred(Nat);
  theorem NatInduction[P]:
    P(0) & (for n being Nat st P(n) holds P(n+1))
    implies for n being Nat holds P(n)
  proof ... end;
end;
```

- A predicate parameter describes a family of first-order theorems, one per substituted predicate.[^1]
- Explicit instantiation: `defpred`, followed by `by NatInduction[P], Base, Step`.
- Functor parameters are schema-level symbols, not sets. The logic stays first-order.

[^1]: Details: Białystok deck, Story 6 (`PermProduct[T]` and views with `qua`); also Backup 1 here.

Speaker note:

- Source: `doc/spec/en/18.templates.md` §18.2.3, 18.8.2, 18.9. Frame 3.4 shows type / value parameters for structures and attributes.

### Frame 3.4 - template: parameterize structures and attributes too

struct: vector-space data over a field K (sketch, inside a definition block):

```mizar
let K be Field;
struct VectorSpace[K] where
  field carrier -> non empty set;
  field add -> BinOp of carrier;
  field smul -> Function of [:K.carrier, carrier:], carrier;
end;
```

`of` / `over` are syntax sugar: `Vector of K` → `Vector[K]` (a mode example).

attr: a subset S is dense in a space M (sketch, condition omitted):

```mizar
let M be MetricSpace;
let S be Subset of M.carrier;
attr DenseDef: S is dense[M] means ...;
```

Prefix attribute notation is syntax sugar too: `M-dense` → `dense[M]`.

**Type / value parameters are shared across struct, attr, mode, pred, and func.**

Speaker note:

- Source: `doc/spec/en/18.templates.md` §18.1, 18.2.6, 18.3, 18.5. Excerpts inside a definition block; vector-space laws and the density condition are omitted. Assume a mode Vector taking K and its of synonym. VectorSpace over K is also a synonym for VectorSpace[K]. Predicate / functor parameters are restricted to theorem and algorithm.
- In the 35-minute version, spend 15 seconds pointing to structure and attribute parameters.

## Part 4. Verified Computation

### Frame 4.1 - Computation: verified algorithms

Use Hoare rules to turn a procedure into proof obligations:

```text
{ requires }  algorithm body  { ensures }
```

P / Q: precondition / postcondition; B: condition; I: invariant; S: statement.

### Table 8. Basic Hoare rules

| Syntax | Conditions to check (partial correctness)[^1] |
|---|---|
| Assignment `x := e` | `P ⇒ Q[x ← e]` (replace free x in Q with e) |
| `if B do S1 else S2 end` | `{P ∧ B} S1 {Q}` and `{P ∧ ¬B} S2 {Q}` |
| `while B do S end` | Establish: `P ⇒ I`; preserve: `{I ∧ B} S {I}`; exit: `I ∧ ¬B ⇒ Q` |
| `return e` | `P ⇒ Q[result ← e]` |

**Apply these rules recursively to generate first-order proof obligations.**

[^1]: Basic forms for simple variables and pure expressions. Early return / break / continue and type / domain conditions are omitted.

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md` §20.1.4, 20.4.3, 20.5.1, 20.13.3. The table gives backward rules with capture-avoiding substitution. Forward generation uses old-state names for assignments and havocs modified locations at loop heads. Details: `draft.ja.md`.
- Partial correctness is the default. terminating requires separate termination checks. Verified procedures can also support proofs.

### Frame 4.2 - algorithm: contracts, proofs, computation

**Euclid's algorithm with a contract** (specification example):

```mizar
terminating algorithm EuclidGcdDef: euclid_gcd(a, b) -> Nat
  requires a >= 1 & b >= 1
  ensures result = Gcd(a, b)
do
  var x := a;  var y := b;
  while y <> 0 do
    invariant x >= 1 & y >= 0 & Gcd(a, b) = Gcd(x, y);  decreasing y;
    const r := x mod y;  x := y;  y := r;
  end;
  return x;
end;
```

- **`terminating` requires termination for every input satisfying requires. Check the invariant and strict decrease of y.**[^1]
- **After verification, promote it to a mathematical functor for formulas and proofs.**

[^1]: Loop annotations resemble Dafny's `invariant` / `decreases`. Evo uses `invariant` / `decreasing`.

Speaker note:

- Source: specification chapter 20, §20.1.1, 20.5, 20.12. Outer definition / let context omitted.
- Comparison: [Dafny §8.15](https://dafny.org/latest/DafnyRef/DafnyRef.html#sec-loop-specifications).

### Frame 4.3 - Euclid: code and proof obligations

The body again (sketch, a,b >= 1; outer context omitted):

```mizar
var x := a; var y := b; :: (1)
while y <> 0 do
  invariant x >= 1 & y >= 0 & Gcd(a,b) = Gcd(x,y); :: (2)
  decreasing y; :: (3)
  const r := x mod y; x := y; y := r;
end;
return x; :: (4)
```

### Table 9. Proof obligations for Euclid's algorithm

| Point | Proof obligation | Mathematical reason |
|---|---|---|
| (1) | Establish the invariant | `x=a`, `y=b`; positivity from the precondition |
| (2) | Preserve the invariant | Old `(x,y)` becomes `(y,r)`; `Gcd(x,y) = Gcd(y,r)`, old `y >= 1`, `r >= 0` |
| (3) | Decrease the Nat measure | `y' = r`, `0 <= r < y` |
| (4) | Postcondition at exit | `y=0`; `Gcd(a,b) = Gcd(x,0) = x` |

**Check each point using library lemmas for GCD and remainder.**

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md` §20.5, 20.12, 20.13.3. Numbers connect the repeated body to the table. Compute r=x mod y in the old state, then update to (y,r). SAT alone does not know arithmetic. Changing y:=r to y:=x loses the decrease. Specification-based explanation.
- Future direction: number theory, combinatorics, symbolic computation, optimization, cryptography, and quantum algorithms. MVM execution and code extraction remain future work.

### Frame 4.4 - terminating and functor promotion

### Table 10. Algorithm termination and guarantees

| Algorithm form | Meaning under requires |
|---|---|
| Without terminating | The contract holds if the call returns |
| Verified terminating | Total under requires; usable as a functor |

Reason with Euclid's contract (sketch):

```mizar
let a, b be Nat;
assume a >= 1 & b >= 1;
thus euclid_gcd(a,b) = Gcd(a,b) by EuclidGcdDef;
```

Use a computed result in a proof (sketch, MVM not implemented):

```mizar
thus euclid_gcd(12,8) = 4 by computation;
```

Run a verified algorithm on MVM, the virtual machine, and use its result in a proof.

**The default `computed` path also trusts MVM. It differs from kernel / SAT rechecking.**

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md` §20.7.2–20.7.3, 20.9.1–20.9.2, 20.13.2; `16.theorems_and_proofs.md` §16.5.1. MVM execution and code extraction are future work.
- Requires the precondition and a verified executable artifact. Replay requires defining equations and kernel checks; it does not apply to this loop-based Euclid example. Details: `draft.ja.md`.

### Frame 4.5 - Verified expression rewriting: lambda and differentiation

Write a function directly (sketch, u is Real):

```mizar
u |-> (sin u)*u
```

`differentiate` recursively processes expression forms such as products and sin.

Use it in a derivative proof (sketch, x is Real; MVM not implemented):

```mizar
thus diff((u |-> (sin u)*u), x) = (cos x)*x + sin x
  by computation(differentiate(u |-> (sin u)*u));
```

- **`|->` denotes a set-theoretic function. An `expr` syntax tree keeps expression structure and variable binding.**
- The transformation algorithm's contract verifies differentiability and the correct derivative.[^1]

[^1]: Assumes differentiation, semantic conversion, and simplification lemmas. This is a specification sketch, not an execution result.

Speaker note:

- Source: `doc/spec/en/13.term_expression.md` §13.10; `20.algorithm_and_verification.md` §20.5.3, 20.9.2, 20.12.
- differentiate is a partial algorithm. Variable declarations, set-theoretic meaning, and a factorization application idea are in `draft.ja.md`.

## Part 5. Development Infrastructure

### Frame 5.1 - Environment: what must an author import?

Current Mizar environment, shortened from ALGSTR_0 (sketch):

```mizar
environ
 vocabularies ... STRUCT_0 ...;
 notations ... STRUCT_0;
 constructors ... STRUCT_0 ...;
 registrations ... STRUCT_0;
 theorems STRUCT_0;
```

### Table 11. The ten environment directives

| Directives | What they import |
|---|---|
| vocabularies / notations | Symbols / notation |
| constructors / registrations | Constructors / automatic type facts |
| requirements / definitions | Special automatic processing / definition information |
| equalities / expansions | Equality / definition-expansion information |
| theorems / schemes | Theorems / schemes |

- `notations` / `definitions`: article order determines notation / definition priority.
- **The hardest part of article authoring: managing environment overrides and order dependence.**

Speaker note:

- Source: Białystok `draft.md`, Frames 2.1–2.2; ALGSTR_0 excerpt. The code shows only five directive types. Authors select resources by role. requirements names special automatic processing to enable.
- Source: Adam Naumowicz, Towards Standardized Mizar Environments, CICM 2017, slides 7, 13, 19–24: <https://mizar.uwb.edu.pl/~softadm/imports/slides.pdf>. Overrides mean priority when selecting notation and redefinitions. Reordering can change existing notation resolution and proofs. Details: `draft.ja.md`.

### Frame 5.2 - Imports and dependency tracking

Module imports in the new specification (specification example):

```mizar
import .function;
import mml.algebra.structure.sorted;
```

### Table 12. Imports and dependency tracking

| Stage | What authors can inspect |
|---|---|
| Import | Public definitions, theorems, and registrations |
| Resolution | Original fully qualified names and registration traces |
| Reuse | Checked dependency fingerprints for incremental builds |

- **Import public items together, and trace what was actually used.**
- The design fixes dependency versions with packages and lockfiles. IDE diagnostics expose resolved names and traces.

Speaker note:

- Source: `doc/spec/en/12.modules_and_namespaces.md` §12.3; `17.clusters_and_registrations.md`, Traceability; `23.package_management_and_build_system.md`. These imports are not a one-to-one mechanical translation of the old environment. Details: Backups 2–3.

### Frame 5.3 - The whole picture [deep dive]

![Mizar Evo in one picture](figures/layer_stack.pdf)

- **Support rich mathematical writing with a small first-order foundation and modern development tools.**

Speaker note:

- Elaboration turns mathematical language into a first-order representation. The kernel checks candidate evidence from external ATPs. Checked artifacts and libraries support IDE / LSP, AI, and publication.

## Part 6. Checking And Automation

### Frame 6.1 - Automation: find lemmas and suggest a proof

Isabelle/HOL: Sledgehammer suggests a proof (sketch):

```text
have "Q a"
  sledgehammer
  by (metis allPQ pa)
```

Mizar Evo: request lemma search with `by atp` (provisional syntax).

**Open goal → library lemma candidates → ATP trials → a `by` suggestion.**

Request and suggestion (sketch, planned; Pa is the local assumption P(a)):

```mizar
thus Q(a) by atp;
:: Suggested proof:
:: library.AllPQ: for x holds P(x) implies Q(x)
thus Q(a) by library.AllPQ, Pa;
```

- **Reduce manual lemma search. Keep the selected premises explicit in the source.**
- Apply the suggestion and verify again. The kernel / SAT checks formula and substitution evidence before acceptance.

Speaker note:

- `by atp` is provisional presentation syntax, not settled specification syntax. Future arguments are expected.
- Source: specification chapter 22, §22.4.4; chapter 23, §23.5.2. Details and comparison sources: `draft.ja.md`.

### Frame 6.2 - Using ATPs: search, check, and reuse

![ATP use with checking, storage, and retry](figures/llm_atp_loop.pdf)

- **Goal and premises → ATP search → kernel checking → library storage and reuse.**
- If unresolved, revise lemmas or strategy and retry. The figure shows design intent.

Speaker note:

- LLMs can help suggest and repair proofs. Acceptance depends on evidence checking, explained next.
- Normal verification uses source citations and local hypotheses. Assistance selects library candidates, tries them, and suggests citations to apply before re-verification. Native-hammer and retry evaluation remains future work.
- Source: `doc/spec/en/21.source_code_annotation_and_atp.md` §21.7.2; `doc/design/architecture/en/21.ai_agent_interface.md`.

### Frame 6.3 - Resolution tree and extracted substitutions

Candidate extraction from a resolution log (sketch):

![Resolution tree with composed substitutions and extracted evidence](figures/resolution_evidence.pdf)

- **An untrusted extractor composes substitutions along branches: save `x := a` for F1 and `y := a` for F2.**

Speaker note:

- Clause variables x/y have distinct names. Unifying Q gives x:=y; unifying P gives y:=a. Compose them to recover F1[x:=a] and F2[y:=a].
- This is a log-based candidate-generation idea. The independent instance finder in architecture 10 is unimplemented (ATP TODO 30). The kernel does not replay each log inference.
- Source: architecture chapters 08, 10, 15, 16. This is a sketch, not an external-prover execution example.

### Frame 6.4 - From saved evidence to concrete SAT clauses

From evidence checks to SAT-based acceptance (sketch):

![Evidence checks, ground instances and a five-variable ten-clause SAT refutation](figures/sat_evidence_check.pdf)

- **Check evidence, regenerate CNF, confirm UNSAT with SAT, and accept R(a).**

Speaker note:

- Save original formulas, composed substitutions, binding context, target VC, and refutation polarity. Regenerate instances and SAT clauses; details in `draft.ja.md` §6.
- Each blue box defines implication variable s/t and gives four clauses. With unit clauses p and not r, there are five variables and ten clauses. DIMACS is retained in `draft.ja.md` §6. This is not an execution dump.
- The kernel neither searches for substitutions nor replays external resolution steps. Source: architecture chapters 08, 15, 16.

## Part 7. Status And Roadmap

### Frame 7.1 - Implementation status and size

**Rust product code: 271.1 kLoC now; 360–540 kLoC estimated at completion (provisional).**

### Table 13. Implementation status and code size (kLoC)

| Area | Implemented | Completion estimate | Limited implementation and remaining work |
|---|---:|---:|---|
| lexer / parser / frontend | 34.4 | 36–43 | Includes syntax; source loading, lexing, parsing connected |
| resolver / checker | 122.9 | 148–193 | Limited source-derived semantics; generalization / service connections remain |
| Core / VC | 49.7 | 65–95 | Includes IR / proof; limited source to verification conditions |
| ATP | 9.6 | 16–28 | Problem encoding / candidate generation; real backend integration remains |
| kernel | 12.8 | 15–20 | Formula, substitution, provenance checks; includes 1.9 for SAT encoding / connection |
| build / driver / artifact | 41.3 | 56–85 | Includes cache / shared support; full-pipeline publication not connected |
| LSP | 0.5 | 6–19 | Lifecycle / document sync base; semantic connections remain |
| MVM / extraction / doc | 0 | 17–57 | Runtime, extraction, documentation generation unimplemented |

**Checking, excluding ATP: kernel plus SAT dependencies = 17.6 kLoC.**

Core, session, and shared support are counted separately. The table excludes dependencies, tests, blank lines, and comments. Estimates include current code; values are rounded.

Source: fixed snapshot `08861708`. Counting and estimate basis: `draft.ja.md` §7.

Speaker note:

- 1 kLoC = 1,000 lines. Completion estimates are rough and not calibrated against delivery history. MML porting is excluded.
- Kernel 12.751, including 1.947 for SAT encoding / connection. SAT dependencies batsat 0.6.0 and bit-vec 0.5.1 add 4.879, totaling 17.630 kLoC. This does not measure the full minimum trusted computing base (TCB), which also involves Core, session, and other support.
- Limited implementation at the fixed snapshot. Whole-code count: October 15; kernel recount and SAT dependencies: October 16, 2026.

### Frame 7.2 - Next steps and summary

### Next steps

- **First priority: connect source-to-semantics → VC → evidence → kernel → artifact → build.**
- Align algorithm semantics and proof obligations; expand the working paths.
- Develop MVM runtime and extraction separately. Porting all MML is a separate task from completing the processor.
- Learned premise selection, LLM repair, cryptography, and quantum algorithms are long-term directions.

### Summary

- **Keep FOL, set theory, and mathematical writing. Redesign the language and development tools.**
- **Separate ATP search from trusted checking. The specification is still evolving.**
- **I welcome ITP users' requests for notation, proof assistance, and development tools.**

Speaker note:

- Source: `doc/design/todo.md`, integration order and MML-port boundary.

## Backup 1. MizAR 60 In Detail

Source: Jakubův et al., MizAR 60 for Mizar 50, ITP 2023.

- Dataset: MML 1147 exported by MPTP, 57,897 theorems including unnamed top-level lemmas; the same version as the MizAR 40 evaluation, so the results can be compared.
- Holdout: 1,690 / 2,896 (58.4%), without user premise help, 420 CPU s. Training/development/holdout: 90:5:5 (MizAR 40: about 40.6%).
- Over 75% proved when the premises can be chosen from the library by a human or a machine (MizAR 40: 56%).
- Strongest single method: 40% in 30 s in hammering mode; 60% in 120 s with human premises.
- Transfer: the strongest method also works on 13,370 new theorems from 242 new articles in MML 1382.
- Methods: E and Vampire with ENIGMA and Deepire guidance, learned premise selection, and a loop that trains on millions of ATP proofs.

## Backup 2. Sledgehammer Evaluations In Detail

| Evaluation | Dataset and method | Success |
|---|---|---|
| ITP 2022 | 5,000 AFP goals, 16 greedy configurations | 68.8% |
| Magnushammer (preprint 2023) | 1,000 PISA theorems, Sledgehammer | 38.3% |
| Same PISA evaluation | Magnushammer with learned premise selection | 59.5% |

- ITP 2022: 50 entries, 100 goals each; MePo, 512 facts in the base setting, 30 CPU seconds per configuration. Configurations are selected using the evaluation set. Reconstruction is not evaluated.
- PISA: Isabelle2021-1, Sledgehammer timeout 30 seconds, five local provers, union over several settings. Success requires a proof checked by Isabelle.
- Close dates do not make local goals and whole theorems, or search success and checked proofs, the same measure.

Speaker note:

- Source: Desharnais et al., Seventeen Provers Under the Hammer, ITP 2022, sections 5, 5.6, Table 10; Mikuła et al., Magnushammer, arXiv:2303.04488 (preprint 2023), Table 2, Appendix A.4.

## Backup 3. HOL-To-FOL Encodings And Reconstruction

- Function application: a variable function applied to an argument becomes `app(F, X)`; applications of constants may stay curried or be flattened.
- Lambda abstraction: lambda lifting introduces fresh constants with defining equations; combinator translation is the alternative.
- Types: polymorphic HOL types are encoded by guards, tags, or monomorphization; the choice affects soundness, completeness, and prover performance.
- Booleans: Boolean-valued terms inside formulas need a separate encoding.
- Reconstruction: `metis` with the used lemmas, `smt` replay of SMT proofs, or generated Isar text; evaluations count reconstruction failures separately.

Source: Meng and Paulson 2008; Blanchette, Böhme, Popescu, Smallbone 2016; Blanchette, Kaliszyk, Paulson, Urban 2016; Schurr, Fleury, Desharnais 2021.

## Backup 4. Template Instantiation: One Proof, Many Views

Bounded type parameter and generic theorem (specification example, proof omitted):

```mizar
definition
  let T be type extends commutative associative unital Magma;
  theorem PermProduct[T]:
    for s being FinSequence of T,
        p being Permutation of dom s
    holds Product[T](s) = Product[T](s * p)
  proof ... end;
end;
```

Instantiation (specification example, with the required registrations):

```mizar
PermProduct[commutative associative unital AddMagma]
PermProduct[commutative associative unital MulMagma]
let R be commutative Ring;  PermProduct[R qua AddMagma]  :: additive view
```

- A ring reaches Magma along two paths; `qua` selects the view, and the view also sets the notation.

Speaker note:

- Source: `doc/spec/en/18.templates.md`, section 18.2.2.

## Backup 5. Kernel Evidence And The Trusted SAT Check

![KernelEvidence and the kernel's SAT check](../2026-09-bialystok/figures/certificate_replay.pdf)

- The evidence holds source formulas, explicit substitutions, provenance, and target and goal bindings. The kernel checks them, builds a deterministic SAT problem, and requires UNSAT from its trusted in-process SAT checker.
- Backend proof traces, SMT proof objects, logs, and exit codes are for diagnosis only.

Speaker note:

- Source: `doc/design/architecture/en/08.reasoning_boundary.md`, `15.kernel_certificate_format.md`.

## Backup 6. The Core ATP Path

![The core ATP path, with responsibility groups](../2026-09-bialystok/figures/pipeline.pdf)

- Only obligations that are still open after deterministic discharge go to ATPs. The earlier discharge also needs replayable evidence.
- Each boundary states who owns a fact, which artifact records it, and what must be checked again after a change.

Speaker note:

- Source: `doc/design/architecture/en/00.pipeline_overview.md`; Bialystok deck, Part 10.

## Backup 7. Incremental Verification

![The fingerprint graph: what a change re-verifies](../2026-09-bialystok/figures/fingerprint_graph.pdf)

- A proof-body edit does not rebuild importers when the exported statement and the accepted status stay the same. An interface change re-verifies the dependency cone.
- All relevant cache keys must match; missing data is a cache miss. Cache reuse is never proof authority: a clean build must reproduce every acceptance.

Speaker note:

- Source: `doc/design/architecture/en/11.artifact_and_incremental_build.md`, `18.dependency_fingerprint.md`; Bialystok deck, Story 5.

## Backup 8. Package Manifest

Mizar Evo (specification example):

```toml
[package]
name    = "algebra"
version = "2.3.1"
edition = "2025"

[dependencies]
mml_core = "^1.0"
topology = { version = "^0.9", features = ["metric"] }
```

- Reproducible builds need fixed source, lockfile, toolchain, and verifier settings, including deterministic ATP evidence.
- Versioned reuse replaces manual copying between article sets; an aggregate module can export a whole topic through one import.

Source: `doc/spec/en/23.package_management_and_build_system.md`; Bialystok deck, Story 1.

## Backup 9. Forty-Five-Minute Additions

Add selected examples where they fit in the story:

1. HOL-to-FOL function encoding worked through on one goal (`app`, partial application, lambda lifting).
2. Mizar function representation: `Function of X,Y`, `f.x`, soft type as a set-theoretic object.
3. Template: `PermProduct[T]` with additive and multiplicative views (Backup 4).
4. Algorithm: Euclidean GCD plus `by computation` and the promotion to a functor.
5. Structures and views: `AddMagma`, `MulMagma`, `Magma` (Bialystok Story 2).
6. Modern infrastructure: `mizar.pkg`, namespaces, the fingerprint graph (Backups 7-8).
7. Checking: evidence instantiation and the SAT check (Frame 6.3; details in Backup 5).

## Backup 10. Sources And Attribution

Exact MML excerpts used in this talk:

| Purpose | Source | Lines | Used in |
|---|---|---:|---|
| Function application `f.x` | `funct_1.miz` | 138-140 | Frame 1.2 |
| `Function of X,Y` as a soft type | `funct_2.miz` | 87-90 | Frame 1.2 |

Source URLs:

- `FUNCT_1`: <https://mizar.uwb.edu.pl/version/current/mml/funct_1.miz>
- `FUNCT_2`: <https://mizar.uwb.edu.pl/version/current/mml/funct_2.miz>

Attribution note:

- MML plain-text files state GPL-3.0-or-later / CC-BY-SA-3.0-or-later terms; keep article attribution, URLs, and line numbers in speaker notes.
- Benchmark figures: see Backups 1-2 and `references.bib`. Verify bibliographic metadata against publishers before the final deck.

## Backup 11. Comparing Evaluation Conditions

| | MizAR 60 (ITP 2023) | Sledgehammer / AFP (ITP 2022) |
|---|---|---|
| success | 58.4% (1,690 / 2,896) | 68.8% (3,440 / 5,000) |
| counts | MML 1147: 2,896 holdout theorems/lemmas (57,897 total) | 5,000 local goals; 50 AFP entries |
| premises | learned selection from the whole library | MePo; base 512 facts, varied by configuration |
| budget | portfolio: 420 CPU s | 16 greedy configurations, 30 CPU s each: 480 CPU s |
| status | ATP proof found (hammering) | external proof found; no Isabelle reconstruction |

- **Different units, premises, budgets, and configurations; the rates do not rank the designs.**

Speaker note:

- A "hammer" is a tool that sends a goal to automatic provers. "Premises" are the facts the prover may use.
- Source: Jakubův et al. 2023, sections 6.2, 6.5; Desharnais et al. 2022, sections 5, 5.6, Table 10. The greedy configurations were selected using the evaluation set itself.
- Publication dates differ from experiment dates. MizAR's main experiments ran in 2020-2021; the AFP study uses Isabelle from January 2022 and AFP from December 2021.
- MizAR's 75% figure uses premises chosen from the library by a human or a machine. Keep it for questions.

## Backup 12. What Each Number Counts

![What each benchmark counts](figures/evaluation_units.pdf)

- **MizAR asks: can the machine prove this whole theorem from the library, with no help from the author?**
- **The AFP study asks: can the machine close this goal where it appears, often inside a proof that a person has already written?**
- Both are fair questions. They are not the same question.

## Backup 13. Two Paths To A First-Order Prover

![Two paths from an interactive prover to a first-order ATP](figures/two_paths.pdf)

- **In Sledgehammer's first-order and SMT path, a higher-order goal is translated, and the proof is rebuilt inside Isabelle.**
- **MizAR gives the prover a problem that is already first-order. The generated problem is first-order.**

Speaker note:

- Both paths describe existing systems (Blanchette, Kaliszyk, Paulson, Urban 2016; Jakubův et al. 2023). The orange boxes are the translation and reconstruction layers.
- The figure shows the first-order and SMT path. The ITP 2022 evaluation also includes provers using native higher-order formats.
- MizAR's headline percentages count ATP proofs. The Mizar checker re-checks the resulting `by` step when the inference is within its strength.

## Backup 14. Functions In Higher-Order Logic

Schematic HOL statement (sketch, Isabelle/HOL-style notation):

```hol
f :: 'a => 'b        x :: 'a

P (f x)
```

- **In HOL, function types and function application are part of the logic itself.**
- Functions as arguments, functions that return functions, partial application, lambda: all can be written directly.
- **This is very convenient for the person who writes mathematics.**
- **But E and Vampire are first-order provers. The higher-order structure must be encoded before they can see it.**

## Backup 15. HOL Encoding And Reconstruction

Encoding steps, schematic (sketch):

```fol
F X                   ->  app(F, X)
(%x. t) ...           ->  fresh constant + defining axioms   (lambda lifting)
polymorphic types     ->  type guards or type tags
Boolean-valued terms  ->  extra encoding
```

- **Applying a variable function becomes an explicit `app` symbol. Lambdas are lifted out or turned into combinators. Types become guards or tags.**
- **After the prover succeeds, the proof must come back into Isabelle: a `metis` call, an `smt` replay, or generated Isar text. This step is called reconstruction.**
- These mechanisms connect HOL goals to first-order provers.
- This describes the connection, not a measured performance ranking.

Speaker note:

- Source: Meng and Paulson 2008 (translating higher-order clauses to first-order clauses); Blanchette, Böhme, Popescu, Smallbone 2016 (type encodings); Blanchette, Kaliszyk, Paulson, Urban 2016 (survey); Schurr, Fleury, Desharnais 2021 (reconstruction).
- The exact encodings that Sledgehammer uses depend on the prover and the options. This frame is schematic.

## Backup 16. Where Complexity Lives

![Where HOL and FOL systems pay for complexity](figures/where_you_pay.pdf)

- **HOL encodes higher-order structures for ATPs. Mizar's language hides set-theoretic details.**
- **Mizar Evo keeps the first-order foundation and modernizes mathematical writing.**

Speaker note:

- HOL and FOL put complexity in different layers. HOL has function types and lambdas in its logic; its first-order ATP path uses encoding and reconstruction.
- Mizar's soft types, modes, attributes, registrations, and schemes hide details such as pairs and domains that appear in raw set-theoretic writing.

## Backup 17. Design Trade-Offs

| | HOL ITP + ATP | FOL ITP + ATP |
|---|---|---|
| writing | higher-order features, short text | long text if written raw |
| functions | basic higher-order objects | first-order set-theoretic objects |
| ATP connection | HOL-to-FOL / SMT encoding | generation of a first-order problem |
| reconstruction | reconstruction in Isabelle | rechecking Mizar steps where supported |
| the language must provide | HOL abstraction | abstraction that hides first-order details |

- This table compares representations and checking paths. It gives no measured performance advantage.
