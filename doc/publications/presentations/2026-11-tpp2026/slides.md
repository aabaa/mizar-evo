# Mizar Evo: Design Principles

Status: first deck draft for TPP 2026, designed from the Japanese narrative
draft `draft.ja.md`, which remains the canonical content document.

Planned occasion: TPP 2026 (the 22nd Theorem Proving and Provers meeting),
RIKEN AIP Tokyo Office, November 16-17, 2026. Thirty-minute slot.

Companion files: `script.ja.md` (Japanese spoken script, frame by frame),
`references.bib` (seed bibliography), `../2026-09-bialystok/` (visual and
reference deck; its figures are reused by relative path and its detailed
feature frames serve as backup material instead of being repeated here).

## Main Idea

Start with six challenges in modernizing current Mizar, then map each to a
design principle in the new specification. Preserve the logical foundation and
readable mathematics while rebuilding generic mechanisms, verified computation,
development tools, and evidence checking.

Benchmarks and HOL/FOL comparisons are supplementary context. Their results
do not establish a first-order performance advantage.

## Language Level

Slide prose is plain English at CEFR B1 to easy B2: short sentences, common
words, one idea per sentence. Technical terms (premise selection, hammer,
reconstruction, soft type, kernel) stay, and the first use of a term explains
it in simple words. Code, exact MML excerpts, and figure labels are unchanged.
The Japanese companion deck `slides.ja.md` follows the same frames.

## Claim Levels

Statements in this deck fall into three levels. Untagged statements are facts
about existing systems, published benchmarks, or the Mizar Evo specification
and main branch. The two other levels are tagged in the text:

- "research hypothesis": a claim Mizar Evo is built to test, not a result;
- "future direction": a target with no committed date or design.

## Code Status Convention

Every code example carries one of three status labels, as in the Bialystok deck:

- "exact MML excerpt": exact text from the current MML, with article name and
  line numbers; attribution and license notes stay in speaker notes;
- "specification example": taken from or directly adapted from the Mizar Evo
  language specification under `doc/spec/en/`;
- "sketch": schematic text that no specification fixes (used for the HOL and
  encoding illustrations and for ordinary Mizar usage).

## Source Status

### Repository Sources

- `doc/spec/en/01.introduction.md` (first-order foundation, kernel principle)
- `doc/spec/en/18.templates.md` (templates, schemes)
- `doc/spec/en/20.algorithm_and_verification.md` (algorithms, MVM)
- `doc/spec/en/21.source_code_annotation_and_atp.md` (ATP integration,
  axiom set assembly: cited premises only, no global premise selection)
- `doc/design/architecture/en/08.reasoning_boundary.md`,
  `09.atp_interface_protocol.md`, `10.atp_backend_integration.md`
- `doc/design/todo.md`, Crate Status (implementation scope, October 2026)

### External Sources

Checked on October 7, 2026:

- Jakubův et al., MizAR 60 for Mizar 50, ITP 2023 (arXiv 2303.06686):
  MML 1147 exported by MPTP as 57,897 theorems including unnamed top-level
  lemmas; 58.4% proved in hammering mode by a portfolio limited to 420 CPU s;
  over 75% with premises selected from the library by a human or a machine;
  strongest single method 40% in 30 s (hammering) and 60% in 120 s
  (human premises).
- Blanchette, Haslbeck, Matichuk, Nipkow, Mining the Archive of Formal Proofs,
  CICM 2015: 6,934 goals from 128 randomly selected AFP theories (up to 100
  goals each), Isabelle2014, MePo, 30 s per prover (E, SPASS, Vampire, Z3);
  one-line reconstruction 49.4-49.7% per prover; 60.7% of goals proved when
  the provers are combined and trusted as oracles.
- Böhme and Nipkow, Sledgehammer: Judgement Day, IJCAR 2010 (as summarized in
  the CICM 2015 paper): 1,240 subgoals from seven theories, 46% with E, SPASS,
  and Vampire in parallel for 30 s; a 2015 preliminary evaluation reached 75%
  on the same benchmarks with six provers.
- MML plain text for exact excerpts: `funct_1.miz` lines 138-140 and
  `funct_2.miz` lines 87-90 at <https://mizar.uwb.edu.pl/version/current/mml/>.
  The files state GPL-3.0-or-later / CC-BY-SA-3.0-or-later terms; keep
  attribution, URLs, and line numbers in speaker notes.
- TPP 2026 announcement: November 16-17, 2026, RIKEN AIP Tokyo Office
  (<https://auto-res.github.io/tpp2026/>).

## Deck Shape

Section 0 gives one table of challenges and design principles.
Sections 1–6 each explain the corresponding principle. Unnumbered closing
frames cover implementation status, the roadmap, and discussion.
Seventeen backups keep benchmark and HOL/FOL comparison details.

| Section | Minutes | Story beat |
|---|---:|---|
| 0. Introduction | 0-2 | challenges and design principles in one table |
| 1. Logical foundation | 2-4 | retain first-order logic, set theory, and MML |
| 2. Readable mathematics | 4-10 | named registration chains, inheritance, and theorem reuse |
| 3. Generic mathematics | 10-14 | templates, infix notation, and type-argument inference |
| 4. Verified computation | 14-20 | Hoare logic, termination, promotion, and Euclid obligations |
| 5. Development infrastructure | 20-23 | environment roles and ordering, imports |
| 6. Checking and automation | 23-28 | Sledgehammer analogy, refutation, CNF, and SAT |
| Closing | 28-30 | status, roadmap, and discussion |

For a 45-minute slot, add the examples listed in §README.md`.

## Part 0. Introduction

### Frame 0.1 - Title

Title:

```text
Mizar Evo: Design Principles
Connecting automatic proof, readable mathematics, and checked computation
```

Subtitle:

```text
TPP 2026, RIKEN AIP Tokyo Office, November 2026
```

Speaker note:

- **Mizar is a proof checker. It has a fifty-year-old library, written in a readable mathematical language over first-order set theory.**
- **Mizar Evolution, or Mizar Evo, is a redesign of its language and tools. I will start with current Mizar's modernization challenges, then show how the new specification addresses them.**
- Everything I say has a label: fact, research hypothesis, or future direction.

### Frame 0.2 - How To Read The Claims [deep dive]

| Level | Meaning | Marked how |
|---|---|---|
| fact | existing systems, published benchmarks, the Mizar Evo specification and main branch | no tag |
| research hypothesis | a claim that Mizar Evo is built to test | tagged in the text |
| future direction | a goal with no fixed date or design yet | tagged in the text |

Code labels follow the Bialystok deck: exact MML excerpt, specification example, sketch.

- The specification and the implementation are different things. One frame near the end separates them.

### Frame 0.3 - Current Mizar: Six Challenges And Design Principles

| Challenge in modernizing Mizar | New design principle |
|---|---|
| **§1 Foundation:** retain logic and MML; renew tools | First-order logic, set theory, and a small kernel |
| **§2 Writing:** implicit types, registrations, overloads | Mathematical abstraction; explicit, traceable choices |
| **§3 Generics:** separate definition and scheme mechanisms | Templates for definitions, theorems, and schemes |
| **§4 Computation:** connect proofs and procedures | Contracts, invariants, and termination checks |
| **§5 Development:** tools for a large library | Modules, namespaces, packages, incremental builds, LSP |
| **§6 Checking:** external search and evidence | ATP flow; derive formula instances; trusted SAT check |

Speaker note:

- Each row pairs a challenge with its design principle. Sections 1–6 follow these rows. First-order logic and readable mathematics are assets to preserve.
- The closing status frame separates the specification from implementation.
- Source: specification 01, 18, 20, 21, 23; Bialystok problem-driven stories.

## Part 1. Logical Foundation

### Frame 1.1 - Keep The Foundation, Rebuild The Tools

```text
**Keep the mathematical layer.**
**Rebuild the tools under it and around it.**
```

- **Mizar Evo keeps first-order logic and Tarski-Grothendieck set theory as the base logic.**
- It keeps soft types, modes, attributes, registrations, structures, and declarative proofs.
- **Separate proof search from checking; a small kernel decides acceptance. Details in section 6.**

Speaker note:

- The shared structure concerns mathematical writing and proof tools, not identical logics or representations.
- Neither success rates nor reconstruction failures show a first-order advantage. Evaluate ease of writing, automation, and proof checking.

### Frame 1.2 - Current Mizar: Functions In Set Theory

Function application is a defined set-theoretic relation (exact MML excerpt):

```mizar
  func f.x -> set means
  :Def2:
  [x,it] in f if x in dom f otherwise it = {};
```

A typed function is a soft type over partial functions (exact MML excerpt):

```mizar
definition
  let X,Y;
  mode Function of X,Y is quasi_total PartFunc of X,Y;
end;
```

- **Here a function is an ordinary first-order object: a set of pairs. Application is defined, not built into the logic.**
- **Nothing in the base logic has to be higher-order.**
- The price: if you write raw set theory, the domain, the graph, functionality, and membership all appear in the text.

Speaker note:

- Source: current MML `funct_1.miz`, lines 138-140 (`FUNCT_1:def 2`), and `funct_2.miz`, lines 87-90 (checked October 7, 2026). URLs: <https://mizar.uwb.edu.pl/version/current/mml/funct_1.miz>, <https://mizar.uwb.edu.pl/version/current/mml/funct_2.miz>. GPL-3.0-or-later / CC-BY-SA-3.0-or-later.
- `quasi_total` (FUNCT_2, lines 36-44) says the domain is all of X unless Y is empty.

## Part 2. Readable Mathematics

### Frame 2.1 - Preserve And Extend Mathematical Writing

What the author actually writes (sketch, valid in current Mizar and Mizar Evo):

```mizar
let X, Y be set;
let f be Function of X, Y;
let x be Element of X;
...  f.x  ...
```

- **It is the same set-theoretic function. But the author writes `Function of X,Y` and `f.x`, not pairs and domains.**
- **Soft types, modes, attributes, registrations, schemes, and declarative proofs are not just a nicer way to write the same thing. They are the language design that lifts first-order set theory up to readable mathematics.**
- **New specification: preserve and extend mathematical writing; make implicit type, registration, and overload choices explicit and traceable.**

### Frame 2.2 - Registrations: Give Automatic Chains Names

Named automatic rules (specification example):

```mizar
registration
  cluster EmptyImpliesFinite: empty -> finite for set;
  coherence proof ... end;
  cluster FiniteImpliesCountable: finite -> countable for set;
  coherence proof ... end;
end;
```

| Starting fact | Automatically derived fact | Recorded rule |
|---|---|---|
| S is empty | S is finite | EmptyImpliesFinite |
| S is finite | S is countable | FiniteImpliesCountable |

- **Keep automatic chaining; require a label on every registration item and record the applied path.**
- Labels explain which rule a proof depends on. Explicit `by` citations remain optional for automatic application.

Speaker note:

- Source: `doc/spec/en/17.clusters_and_registrations.md` sections 17.2, 17.7; `23.package_management_and_build_system.md` section 23.7.7. The trace above is illustrative for an empty set S. Attribute resolution precedes ATP search.

### Frame 2.3 - Structures: Stored Fields And Derived Properties

Data and canonical values have different roles (specification example):

```mizar
definition
  struct AddLoopStr where
    field carrier -> set;
    field add -> BinOp of carrier;
    property zero -> Element of carrier;
  end;
end;
```

- **Fields store data and are constructor arguments. Properties get a uniquely determined value from a separate implementation.**
- The declaration of `zero` supplies its type. A `means` implementation proves existence and uniqueness; an `equals` implementation supplies a term.

Speaker note:

- Source: `doc/spec/en/05.structures.md` section 5.2; `07.modes.md` sections 7.4.1, 7.8.2; `sample_codes.md`, AddLoopStr. Overlapping property implementations require coherence.

### Frame 2.3a - Inheritance Can Be Declared Later

After declaring AddLoopStr, add a parent mapping (specification example):

```mizar
definition
  inherit AddLoopStr extends LoopStr where
    field carrier from carrier;
    field add from binop;
    property zero from unit;
  end;
end;
```

- **Separate the structure declaration from later inheritance declarations, much like implementing a Rust trait after defining a type.**
- `from` renames the parent roles: `binop` becomes `add`, and `unit` becomes `zero`.
- Each declaration has one parent. Identical member types need no proof; narrower types need a `coherence` proof.

Speaker note:

- Source: `doc/spec/en/05.structures.md` section 5.3; `sample_codes.md`, AddLoopStr. The Rust comparison concerns separate declarations, not identical semantics. LoopStr declares binop as a field and unit as a property.

### Frame 2.3b - Checked Diamonds And Reused Group Theorems

Two paths in the same hierarchy (sketch):

```text
AddLoopStr -> LoopStr -> Magma
AddLoopStr -> AddMagma -> Magma
```

A Group theorem applied to a ring's additive view (sketch):

```mizar
definition
  let T be type extends Group;
  theorem RightUnit[T]:
    for x being Element of T.carrier holds T.binop(x,T.unit) = x
  proof ... end;
end;
RightUnit[R qua AddLoopStr]  :: gives R.add(x,R.zero) = x
```

- **Track member roots and inheritance paths; check shared members while keeping the operation views distinct.**
- Ring's additive view is a Group. Reuse its theorem through renamed members; multiplication is only required to be a monoid.

Speaker note:

- Source: `doc/spec/en/05.structures.md` section 5.4; `sample_codes.md`, Group and Ring; `18.templates.md` sections 18.2.2, 18.10.2. Assume R is Ring, x is in its carrier, and the needed Group/property definitions and registrations. Equal member types join automatically; other types need coherence. The selected view must satisfy the theorem's assumptions.

## Part 3. Generic Mathematics

### Frame 3.1 - Current MML: Result Types For The Sum Of Functions

**Example: the sum of functions. Add their values at each point.**

Result-type registrations (sketch, separate complex-valued and real-valued input blocks):

```mizar
cluster f1 + f2 -> complex-valued;
cluster f1 + f2 -> real-valued;
```

| Step in VALUED_1 | Type information |
|---|---|
| Define pointwise addition once | The result is a Function |
| Refine the codomain | Separate complex and real PartFunc redefinitions |
| Recover the full input domain | Separate totality registrations |

- **The mathematical operation is already shared. The result-type refinements still follow the value types.**

Speaker note:

- Source: MML [VALUED_1](https://mizar.uwb.edu.pl/version/current/html/valued_1.html), def 1 and the following result-type refinements. Each displayed line is from a separate registration block; declarations and coherence proofs are omitted. GPL-3.0-or-later / CC-BY-SA-3.0-or-later.
- VALUED_1:def 1 uses the intersection of the input domains. The next sketch is limited to a common nonempty domain I; it does not replace the full construction.

### Frame 3.1a - Template: One Body And Concrete Result Types

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

| Call (real R, complex C)[^1] | Result type |
|---|---|
| `f + g` | `Function of I,R.carrier` |
| `u + v` | `Function of I,C.carrier` |

- **A synonym gives infix `+`; infer template arguments from declared types.**

[^1]: Explicit: `f +[R,I] g`, `u +[C,I] v`. Omit arguments only when declared types determine them uniquely. Write `qua` for ambiguous inheritance paths.

Speaker note:

- Assume `let R be RealAdd; let C be ComplexAdd;`, with unique inheritance to nonempty AddMagma and the needed registrations. f,g have declared type Function of I,R.carrier; u,v have Function of I,C.carrier. Assume their normalized declared types uniquely determine T and I.
- Source: `doc/spec/en/11.symbol_management.md` section 11.1.2; `18.templates.md` sections 18.2.2, 18.2.7, 18.7; `19.overload_resolution.md` section 19.6.2. A codomain set alone does not select its addition structure. `qua` views are never inferred.

### Frame 3.2 - Schemes Become Ordinary Templates [deep dive]

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

- A predicate parameter gives a family of first-order theorems, one for each predicate you put in.
- Instantiation is explicit: `by NatInduction[P], Base, Step` after a `defpred`.
- Functor parameters are schema-level symbols, not sets. This keeps the logic first-order.
- Details: Bialystok deck, Story 6 (`PermProduct[T]`, `qua` views), and Backup 4 here.

## Part 4. Verified Computation

### Frame 4.1 - Computation: Verified Algorithms

Hoare-style checking of readable procedures:

```text
{ requires }  algorithm body  { ensures }
```

- **Use familiar pseudocode: assignment, `if/else`, `while`, `for`, and `return`. Contracts and loop invariants state what must hold.**
- **Generate first-order verification conditions using Hoare logic. Verify correctness and, when required, termination.**
- Verify algorithms themselves and use verified procedures to support proofs.

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md` sections 20.2, 20.3, 20.13.3. Correctness is partial by default; `terminating` adds a termination obligation. This is pseudocode-like procedural notation, rather than a separate proof-state tactic language.

### Frame 4.2 - Algorithms: Contracts, Proofs, Computation

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

- **`terminating` requires termination for every input satisfying `requires`. The verifier checks the invariant and strict decrease of y.**[^1]
- **After verification, the algorithm is promoted to a mathematical functor, usable in formulas and proofs.**

[^1]: Loop annotations resemble Dafny's `invariant` / `decreases`; Evo uses `invariant` / `decreasing`.

Speaker note:

- Source: spec 20, sections 20.1.1, 20.5, 20.12; outer definition/let omitted.
- Compare [Dafny §8.15](https://dafny.org/latest/DafnyRef/DafnyRef.html#sec-loop-specifications).

### Frame 4.2a - Termination And Functor Promotion

| Algorithm form | Meaning under requires |
|---|---|
| Without terminating | If the call returns, its contract holds |
| Verified terminating | Totality under requires; usable as a functor |

Reason about Euclid through its contract (sketch):

```mizar
let a, b be Nat;
assume a >= 1 & b >= 1;
thus euclid_gcd(a,b) = Gcd(a,b) by EuclidGcdDef;
```

- **Verified definitional algorithms also give defining equations. Euclid has a loop: use only its totality and contract axioms.**
- `by computation` is specified separately; MVM execution and code extraction remain future work.

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md` sections 20.7.2-20.7.3, 20.13.2; `16.theorems_and_proofs.md` section 16.5.1. `by EuclidGcdDef` cites verified promotion axioms. The callable name alone is not a citation. The fragment assumes the surrounding proof and Gcd definition; the guarantee is under requires.

### Frame 4.3 - Euclid: What Must Be Proved?

One loop step: old state `(x,y)` with `y > 0`; set `r = x mod y`; new state `(x',y') = (y,r)`.

| Obligation | Mathematical reason |
|---|---|
| Establish the invariant | `x=a`, `y=b`; the precondition gives positivity |
| Preserve the invariant | `Gcd(x,y) = Gcd(y,r)`, `y >= 1`, `r >= 0` |
| Decrease the Nat measure | `y' = r`, and `0 <= r < y` |
| Establish the postcondition on exit | `y=0`; `Gcd(a,b) = Gcd(x,0) = x` |

- **The library supplies the GCD and remainder lemmas. These obligations go through theorem checking; SAT alone does not know arithmetic.**
- Replacing the final `y := r` with `y := x` loses strict decrease. This is a specification walkthrough.

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md` sections 20.5, 20.12, 20.13.3. This illustrates obligations, not an execution result.
- Longer-term targets remain future directions: number theory, combinatorics, symbolic computation, optimization, cryptography, and quantum algorithms. MVM execution and extraction remain future work.

## Part 5. Development Infrastructure

### Frame 5.1 - Environment: What Must An Author Import?

Current Mizar environment, shortened from ALGSTR_0 (sketch):

```mizar
environ
 vocabularies ... STRUCT_0 ...;
 notations ... STRUCT_0;
 constructors ... STRUCT_0 ...;
 registrations ... STRUCT_0;
 theorems STRUCT_0;
```

| List | What it supplies |
|---|---|
| vocabularies / notations | Symbols / notation |
| constructors | Constructors |
| registrations / theorems | Automatic type facts / cited theorems |

- **Choose articles by role; `notations` and `definitions` also depend on article order.**

Speaker note:

- Source: Bialystok `draft.md` frames 2.1-2.2, quoting ALGSTR_0. Ellipses shorten the lists. Imported registrations can affect implicit type inference; a symbol alone does not supply its notation or type facts.
- Source on ordering: Adam Naumowicz, Towards Standardized Mizar Environments, CICM 2017, slide 13: <https://mizar.uwb.edu.pl/~softadm/imports/slides.pdf>. The claim concerns those current-environment lists, not every import directive.

### Frame 5.1a - Imports And Dependency Tracking

New specification: module imports (specification example):

```mizar
import .function;
import mml.algebra.structure.sorted;
```

| Stage | What the author can inspect |
|---|---|
| Import | The module's public definitions, theorems, and registrations |
| Resolve | Source fully-qualified names and registration traces |
| Reuse | Validated dependency fingerprints for incremental builds |

- **Bring public items in together, then track what was actually used.**
- Packages and lock files fix dependency versions; IDE diagnostics can expose the resolved names and traces.

Speaker note:

- Source: `doc/spec/en/12.modules_and_namespaces.md` section 12.3; `17.clusters_and_registrations.md`, Traceability; `23.package_management_and_build_system.md`. The import example is not a mechanical one-to-one migration. Details: Backups 7-8.

### Frame 5.2 - The Whole Picture

![Mizar Evo in one picture](figures/layer_stack.pdf)

- **Rich mathematics, a small first-order core, and modern infrastructure.**

Speaker note:

- Elaboration maps the mathematical language to a first-order representation. The kernel checks evidence proposed by external ATPs. Verified artifacts and the library support IDE / LSP, AI, and publication.

## Part 6. Checking And Automation

### Frame 6.1 - Automation: Separate Search From Checking

Isabelle/HOL: Sledgehammer suggests a locally checked proof (sketch):

```text
have "Q a"
  sledgehammer
  by (metis allPQ pa)
```

Mizar Evo: cited facts justify a declarative step (sketch):

```mizar
assume AllPQ: for x being object holds P(x) implies Q(x);
assume Pa: P(a);
thus Q(a) by AllPQ, Pa;
```

- **Shared pattern: goal and facts → external search → local trusted acceptance. A prover's success report alone is insufficient.**
- Isabelle reconstructs a proof, often by local Metis reproof. Mizar Evo checks formula/substitution evidence through instantiation and SAT.

Speaker note:

- Source: [Sledgehammer guide](https://isabelle.in.tum.de/doc/sledgehammer.pdf), sections 1, 5.2; spec 16; architecture 08, 10, 15. The sketches assume corresponding hypotheses.
- Evo uses cited/local facts; Sledgehammer selects theory facts. Their shared search/check pattern does not imply identical evidence or measured results.

### Frame 6.2 - Using ATPs: Search, Check, And Reuse

![ATP use with checking, storage, and retry](figures/llm_atp_loop.pdf)

- **Goal and facts → ATP search → kernel check → store in the library and reuse.**
- For open goals, revise lemmas or the plan and retry. The figure shows design intent.

Speaker note:

- LLMs can help propose and repair. Acceptance depends on the evidence check, explained on the next frame.
- Current ATP input is cited premises and local hypotheses. Whole-library premise selection and the loop's cost-effectiveness remain evaluation targets.
- Source: `doc/spec/en/21.source_code_annotation_and_atp.md` section 21.7.2; `doc/design/architecture/en/21.ai_agent_interface.md`.

### Frame 6.3 - Resolution Tree And Extracted Substitutions

Candidate extraction from a Resolution log (sketch):

```text
F1: forall x. (P(x) implies Q(x)); F2: forall y. (Q(y) implies R(y))
H: P(a); goal: R(a); refute: F1 & F2 & H & not R(a)

{not P(x), Q(x)}             {not Q(y), R(y)}
          \                 /
           sigma1 = {x := y}       :: unify Q(x), Q(y)
           {not P(y), R(y)}        {P(a)}
                    \             /
                     sigma2 = {y := a} :: unify P(y), P(a)
                     {R(a)}        {not R(a)}
                          \        /
                           sigma3 = {}
                                {}
```

- **The untrusted extractor composes substitutions along the branches: save `x := a` for F1 and `y := a` for F2.**

Speaker note:

- Variables x/y are renamed apart. Q unifies with x:=y; P with y:=a. Compose to obtain F1[x:=a] and F2[y:=a].
- This log-assisted producer is a proposal; architecture 10 uses independent instance finding. The kernel checks evidence, not log steps.
- Source: architecture 08, 10, 15, 16. Sketch, not an external-prover execution.

### Frame 6.3a - Saved Evidence To Concrete SAT Clauses

Saved candidate, kernel preprocessing, and SAT input (sketch):

```text
save: F1,F2,H (source bindings); F1[x:=a], F2[y:=a]; goal R(a), refute
check: source/context, binders, capture avoidance
instances: P(a) implies Q(a); Q(a) implies R(a); P(a); not R(a)
atoms: p=P(a)=1, q=Q(a)=2, r=R(a)=3
logical CNF: (not p or q) & (not q or r) & p & not r
Tseitin: s=4 iff (not p or q); t=5 iff (not q or r)
SAT clauses (DIMACS example; 0 ends each clause):
p cnf 5 10
1 0       -3 0
1 4 0     -2 4 0     -1 2 -4 0     4 0
2 5 0     -3 5 0     -2 3 -5 0     5 0
```

- **The small kernel derives instances and CNF from checked evidence. Its SAT checker finds UNSAT, so accept R(a); external Resolution steps are not replayed.**

Speaker note:

- Save source formulas, composed substitutions, binder contexts, target VC, and refutation polarity. Derived instances and SAT clauses are recomputed (Backup 5).
- DIMACS: 5 variables, 10 clauses; negative means negation, 0 ends a clause. s/t encode the implications. This encoder-style construction is not a runtime dump.
- The kernel searches no substitutions. Source: architecture 08, 15, 16.

## Part Closing. Status And Roadmap

### Frame Where The Project Stands (October 2026)

- **Specification: 24 chapters plus appendices. English is the canonical language.**
- **Implemented on the main branch: the Rust frontend (lexer, parser, syntax tree); name resolution and type checking on an alpha corpus; proof-obligation generation with deterministic discharge; ATP problem encoding and candidate evidence; SAT-backed kernel evidence checking; cache, fingerprint, and build-scheduling milestones.**
- **In progress: end-to-end integration from source to verified artifact; the LSP server; the documentation generator.**
- **Planned for later: MVM execution, code extraction, library-wide premise selection, MML migration.**
- This talk does not claim any end-to-end results with external provers.

Speaker note:

- Source: `doc/design/todo.md`, Crate Status, as of October 7, 2026. Re-check before the talk.

### Frame Roadmap

![Roadmap](figures/roadmap_tpp.pdf)

- **2026: finish the specification, the Rust pipeline through the kernel, template processing, and an alpha end-to-end run.**
- **2027: migrate representative MML articles, build a native hammer baseline, and benchmark on top-level theorems, in the same units MizAR uses.**
- 2028 and later: scale up the migration; learned premise selection; LLM-assisted failure recovery; MVM and extraction. Cryptographic and quantum targets stay future directions.

### Frame Closing

```text
**Keep the foundation small.**
**Keep the mathematics readable.**
**Modernize everything else.**
```

- **Thank you. I welcome your questions, especially on MML migration, usability, and how to evaluate automation.**

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
