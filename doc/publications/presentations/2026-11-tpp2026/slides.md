# Mizar Evolution: Why First-Order Logic, Now

Status: first deck draft for TPP 2026, designed from the Japanese narrative
draft `draft.ja.md`, which remains the canonical content document.

Planned occasion: TPP 2026 (the 22nd Theorem Proving and Provers meeting),
RIKEN AIP Tokyo Office, November 16-17, 2026. Thirty-minute slot.

Companion files: `script.ja.md` (Japanese spoken script, frame by frame),
`references.bib` (seed bibliography), `../2026-09-bialystok/` (visual and
reference deck; its figures are reused by relative path and its detailed
feature frames serve as backup material instead of being repeated here).

## Main Idea

Do not list the features of Mizar Evo. Start from a puzzle the audience can
feel: two hammer benchmarks, two numbers that look alike, counting different
things. Use the puzzle to make one idea intuitive:

> Higher-order and first-order systems do not remove complexity. They put it
> in different places. Mizar has spent fifty years hiding the first-order cost
> in its language. Mizar Evo keeps that mathematical identity and rebuilds
> everything else: generic mechanisms, verified computation, and fifty years of
> software infrastructure.

Every frame carries one of three claim levels (see Claim Levels). The deck
never turns a benchmark difference into a verdict, never presents a
specification as an implementation, and never presents a future direction as a
plan with a date.

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

Six parts and ten backup frames. The thirty-minute talk reads the dark-blue
bold sentences in order; frames marked `[deep dive]` can be skipped without
breaking the story. Backups hold the detailed benchmark conditions, the
encoding details, and the Bialystok feature material.

| Part | Minutes | Story beat |
|---|---:|---|
| 1. Two hammers, two numbers | 0-6 | the puzzle, and why the numbers must not be compared |
| 2. Where complexity lives | 6-14 | functions in HOL and in set theory; Mizar's fifty-year answer |
| 3. Modernizing Mizar's answer | 14-24 | templates, algorithms, trust boundary, fifty years of infrastructure |
| 4. The AI era | 24-28 | the whole picture, LLM plus ATP, honest status |
| 5. Roadmap and closing | 28-30 | 2026, 2027, 2028+, back to the two paths |

For a 45-minute slot, add the examples listed in `README.md` rather than new
story beats.

## Part 0. Opening

### Frame 0.1 - Title

Title:

```text
Mizar Evolution: Why First-Order Logic, Now
Connecting automatic proof, readable mathematics, and checked computation
```

Subtitle:

```text
TPP 2026, RIKEN AIP Tokyo Office, November 2026
```

Speaker note:

- **Mizar is a proof checker. It has a fifty-year-old library, written in a readable mathematical language over first-order set theory.**
- **Mizar Evolution, or Mizar Evo, is a redesign of its language and tools. Today I will not list features. I will start from a puzzle about automatic provers, and use it to explain why we keep the logic first-order.**
- Everything I say has a label: fact, research hypothesis, or future direction.

### Frame 0.2 - How To Read The Claims [deep dive]

| Level | Meaning | Marked how |
|---|---|---|
| fact | existing systems, published benchmarks, the Mizar Evo specification and main branch | no tag |
| research hypothesis | a claim that Mizar Evo is built to test | tagged in the text |
| future direction | a goal with no fixed date or design yet | tagged in the text |

Code labels follow the Bialystok deck: exact MML excerpt, specification example, sketch.

- The specification and the implementation are different things. One frame near the end separates them.

## Part 1. Two Hammers, Two Numbers

### Frame 1.1 - Comparing Evaluation Conditions

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

### Frame 1.2 - What Each Number Counts

![What each benchmark counts](figures/evaluation_units.pdf)

- **MizAR asks: can the machine prove this whole theorem from the library, with no help from the author?**
- **The AFP study asks: can the machine close this goal where it appears, often inside a proof that a person has already written?**
- Both are fair questions. They are not the same question.

### Frame 1.3 - Two Paths To A First-Order Prover

![Two paths from an interactive prover to a first-order ATP](figures/two_paths.pdf)

- **In Sledgehammer's first-order and SMT path, a higher-order goal is translated, and the proof is rebuilt inside Isabelle.**
- **MizAR gives the prover a problem that is already first-order. The gap between the checker and the ATP problem is short.**

Speaker note:

- Both paths describe existing systems (Blanchette, Kaliszyk, Paulson, Urban 2016; Jakubův et al. 2023). The orange boxes are the translation and reconstruction layers.
- The figure shows the first-order and SMT path. The ITP 2022 evaluation also includes provers using native higher-order formats.
- MizAR's headline percentages count ATP proofs. The Mizar checker re-checks the resulting `by` step when the inference is within its strength.

### Frame 1.4 - Design Choice: Keep The First-Order Foundation

```text
**A first-order foundation supports abstraction and modern proof tools.**
```

- **Mathematical writing: represent functions and structures as sets; use types and language features to support abstraction.**
- Proof tools share the same broad steps: select premises, search with ATPs, and check proofs.
- **Mizar Evo keeps its first-order foundation and library, and rebuilds the language and development tools.**

Speaker note:

- The shared structure concerns mathematical writing and proof tools, not identical logics or representations.
- Neither success rates nor reconstruction failures show a first-order advantage. Evaluate ease of writing, automation, and proof checking.

## Part 2. Where Complexity Lives

### Frame 2.1 - Functions In Higher-Order Logic

Schematic HOL statement (sketch, Isabelle/HOL-style notation):

```hol
f :: 'a => 'b        x :: 'a

P (f x)
```

- **In HOL, function types and function application are part of the logic itself.**
- Functions as arguments, functions that return functions, partial application, lambda: all can be written directly.
- **This is very convenient for the person who writes mathematics.**
- **But E and Vampire are first-order provers. The higher-order structure must be encoded before they can see it.**

### Frame 2.2 - The Cost Appears At The ATP Boundary

Encoding steps, schematic (sketch):

```fol
F X                   ->  app(F, X)
(%x. t) ...           ->  fresh constant + defining axioms   (lambda lifting)
polymorphic types     ->  type guards or type tags
Boolean-valued terms  ->  extra encoding
```

- **Applying a variable function becomes an explicit `app` symbol. Lambdas are lifted out or turned into combinators. Types become guards or tags.**
- **After the prover succeeds, the proof must come back into Isabelle: a `metis` call, an `smt` replay, or generated Isar text. This step is called reconstruction.**
- This is a great piece of engineering. It is also a layer that a first-order system does not need.
- HOL gets the convenience first and pays at the ATP boundary.

Speaker note:

- Source: Meng and Paulson 2008 (translating higher-order clauses to first-order clauses); Blanchette, Böhme, Popescu, Smallbone 2016 (type encodings); Blanchette, Kaliszyk, Paulson, Urban 2016 (survey); Schurr, Fleury, Desharnais 2021 (reconstruction).
- The exact encodings that Sledgehammer uses depend on the prover and the options. This frame is schematic.

### Frame 2.3 - Functions In Set Theory: Mizar's Foundation

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

### Frame 2.4 - Mizar's Fifty-Year Answer: Hide The Details In The Language

What the author actually writes (sketch, valid in current Mizar and Mizar Evo):

```mizar
let X, Y be set;
let f be Function of X, Y;
let x be Element of X;
...  f.x  ...
```

- **It is the same set-theoretic function. But the author writes `Function of X,Y` and `f.x`, not pairs and domains.**
- **Soft types, modes, attributes, registrations, schemes, and declarative proofs are not just a nicer way to write the same thing. They are the language design that lifts first-order set theory up to readable mathematics.**
- **Mizar has walked in this direction for fifty years. Mizar Evo keeps walking.**

### Frame 2.5 - Where You Pay

![Where HOL and FOL systems pay for complexity](figures/where_you_pay.pdf)

```text
**HOL and FOL do not remove complexity.**
**They put it in different places.**
```

- **HOL pays at the ATP boundary. FOL pays at the keyboard, and Mizar's language takes on that payment.**
- **Mizar Evo's choice: keep the base logic first-order, and hide the human-facing complexity in the language.**

### Frame 2.6 - The Trade-Off In One Table [deep dive]

| | HOL ITP + ATP | FOL ITP + ATP |
|---|---|---|
| writing | higher-order features, short text | long text if written raw |
| functions | basic higher-order objects | first-order set-theoretic objects |
| ATP connection | encoding needed | short distance |
| reconstruction | must cross the logic gap back | simpler in principle |
| the language must provide | HOL abstraction | abstraction that hides first-order details |

- This table is an interpretation, not a measurement. The measurement is what Mizar Evo must produce in 2027.

## Part 3. Modernizing Mizar's Answer

### Frame 3.1 - Keep The Logic, Modernize The Language

```text
**Keep the mathematical layer.**
**Rebuild the tools under it and around it.**
```

- **Mizar Evo keeps first-order logic and Tarski-Grothendieck set theory as the base logic.**
- It keeps soft types, modes, attributes, registrations, structures, and declarative proofs.
- **It makes hidden choices explicit, unifies the generic mechanisms, makes automation traceable, and puts everything on a modern compiler architecture.**
- The next frames show what "modernize the language" means: templates, algorithms, the trust boundary, and the infrastructure.

### Frame 3.2 - Templates: Generic Mathematics Without Higher-Order Logic

**A template is a definition block with parameters** (specification example):

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

- **Parameters can be types, values, predicates, or functors. Classical Mizar schemes become theorems with a predicate parameter, in the same system.**
- **Templates add no unrestricted second-order quantification to the logic. Each instantiation is checked and produces first-order obligations.**
- Generic mathematics lives in the surface language, not in the base logic.

Speaker note:

- Source: `doc/spec/en/18.templates.md`, sections 18.1-18.2 and 18.8.

### Frame 3.3 - Schemes Become Ordinary Templates [deep dive]

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

### Frame 3.4 - Algorithms Are A Second Pillar

- **Algorithms in Mizar Evo are not a replacement for higher-order functions. They answer a different need: reasoning about algorithms, and computation you can check.**
- A historical tendency, not a necessity: first-order logic has complete calculi, and a culture of automatic search grew around them, from resolution to superposition and saturation.
- The LCF and HOL family grew a culture of programmable proof construction: tactics and tacticals.
- **Mizar used declarative proofs and the automation built into the system. It never had a tactic language that users could program.**
- Mizar Evo's algorithm is not a tactic language added on top. It is a procedure with a contract, and the system verifies it.

### Frame 3.5 - Algorithms: Contracts, Proofs, Computation

**Euclid's algorithm with a contract** (specification example):

```mizar
terminating algorithm euclid_gcd(a, b) -> Nat
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

- **Contracts, invariants, and termination measures become first-order proof obligations, checked like theorems.**
- **Two roles: a verified automation procedure, and a checked computation run by `by computation`. Status: specified. MVM execution and code extraction come later.**

Speaker note:

- Source: `doc/spec/en/20.algorithm_and_verification.md`, section 20.12 (condensed: the enclosing `definition` block and `let a, b be Nat;` are omitted).

### Frame 3.6 - Future Targets For Algorithms [deep dive]

Future direction, not current capability:

- number-theoretic and combinatorial algorithms; symbolic algorithms; optimization procedures;
- later: cryptographic algorithms and protocols; quantum and hybrid classical-quantum algorithms.

One workflow for all of them:

1. write the procedure; 2. state the contract; 3. give invariants and termination; 4. generate proof obligations; 5. prove them with ATPs and check them in the kernel; 6. run on concrete inputs; 7. extract code later.

- **Algorithms make the gap between tactics and application algorithms smaller. The targets above are future directions.**

### Frame 3.7 - Search Outside, Trust Inside

![The reasoning boundary: semantics, untrusted search, trusted checking](../2026-09-bialystok/figures/reasoning_boundary.pdf)

- **A first-order ATP is a powerful searcher. It is not a trusted checker.**
- **The Mizar side owns names, types, clusters, and overloads. Provers own search. The kernel owns acceptance: it checks the given formulas and substitutions with a small, trusted SAT check.**
- A prover's exit code is never a proof. This is how first-order automation can be a design principle without making the trusted base bigger.

Speaker note:

- Source: `doc/design/architecture/en/08.reasoning_boundary.md`; Bialystok deck, Story 4; Backup 5 here shows what the evidence contains.

### Frame 3.8 - And Mizar Itself Is Fifty Years Old

| Seen after fifty years of MML | Mizar Evo |
|---|---|
| the article as the unit of dependency | modules with explicit imports |
| global name management | namespaces, fully qualified names |
| weak distribution and versioning | packages, SemVer, lock files |
| full rebuilds | incremental builds with dependency fingerprints |
| ATP as an add-on, automation hard to see | first-class ATP pipeline, resolution traces, kernel evidence |
| weak IDE and machine-readable I/O | LSP, structured diagnostics, agent interface |

- **Keeping Mizar's mathematical ideas and keeping 1970s software architecture are two different things.**
- **Of course, we also modernize the whole development infrastructure. The Bialystok deck has the details. Here one table is enough.**

Speaker note:

- This is not a criticism. It is software engineering that was not common fifty years ago, brought into a formal mathematics environment.
- Details: Bialystok deck, Stories 1, 3, 5, 8; Backups 6-8 here.

## Part 4. The AI Era

### Frame 4.1 - The Whole Picture

![Mizar Evo in one picture](figures/layer_stack.pdf)

```text
**Rich mathematics above. A small first-order logic below.**
**Modern infrastructure around.**
```

### Frame 4.2 - LLM Thinks, ATP Proves, Mizar Evo Remembers And Verifies

![The LLM, ATP, and Mizar Evo division of labor](figures/llm_atp_loop.pdf)

- **LLMs: theories, definitions, strategy, lemmas, recovery from failure. ATPs: cheap, repeated first-order search. Mizar Evo: representation, verified library, trusted checking, origin of each fact.**
- Research hypothesis: when machines generate a lot of mathematics, cheap ATP checking becomes more valuable. The cost and benefit of this loop are not yet measured.
- Current specification: ATPs receive only cited premises and local hypotheses. A library-wide hammer is research for 2027.

Speaker note:

- Source: `doc/spec/en/21.source_code_annotation_and_atp.md`, section 21.7.2, item 4 (no automatic premise selection from the global library); `doc/design/architecture/en/21.ai_agent_interface.md` (edit classes).

### Frame 4.3 - Where The Project Stands (October 2026)

- **Specification: 24 chapters plus appendices. English is the canonical language.**
- **Implemented on the main branch: the Rust frontend (lexer, parser, syntax tree); name resolution and type checking on an alpha corpus; proof-obligation generation with deterministic discharge; ATP problem encoding and candidate evidence; SAT-backed kernel evidence checking; cache, fingerprint, and build-scheduling milestones.**
- **In progress: end-to-end integration from source to verified artifact; the LSP server; the documentation generator.**
- **Planned for later: MVM execution, code extraction, library-wide premise selection, MML migration.**
- This talk does not claim any end-to-end results with external provers.

Speaker note:

- Source: `doc/design/todo.md`, Crate Status, as of October 7, 2026. Re-check before the talk.

## Part 5. Roadmap And Closing

### Frame 5.1 - Roadmap

![Roadmap](figures/roadmap_tpp.pdf)

- **2026: finish the specification, the Rust pipeline through the kernel, template processing, and an alpha end-to-end run.**
- **2027: migrate representative MML articles, build a native hammer baseline, and benchmark on top-level theorems, in the same units MizAR uses.**
- 2028 and later: scale up the migration; learned premise selection; LLM-assisted failure recovery; MVM and extraction. Cryptographic and quantum targets stay future directions.

### Frame 5.2 - Back To The Two Paths

```text
**The question is not whether higher-order systems**
**can use first-order ATPs. They clearly can.**
**The question is what becomes possible when automatic first-order**
**reasoning is a design principle from the beginning.**
```

Mizar Evo's answer, in six parts:

1. Keep first-order logic and set theory as the base logic.
2. Hide the first-order details with Mizar's language design.
3. Extend generic mathematics with templates.
4. Add checked computation with algorithms.
5. Rebuild fifty years of software infrastructure.
6. Combine LLMs and ATPs where each is strong.

### Frame 5.3 - Closing

```text
**Keep the foundation small.**
**Keep the mathematics readable.**
**Modernize everything else.**
```

- **Thank you. I welcome your questions, especially on how to design the 2027 benchmark so that it compares like with like.**

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

The story does not change. Add depth with these examples, in this order of priority:

1. HOL-to-FOL function encoding worked through on one goal (`app`, partial application, lambda lifting).
2. Mizar function representation: `Function of X,Y`, `f.x`, soft type as a set-theoretic object.
3. Template: `PermProduct[T]` with additive and multiplicative views (Backup 4).
4. Algorithm: Euclidean GCD plus `by computation` and the promotion to a functor.
5. Structures and views: `AddMagma`, `MulMagma`, `Magma` (Bialystok Story 2).
6. Modern infrastructure: `mizar.pkg`, namespaces, the fingerprint graph (Backups 7-8).
7. Trusted boundary: ATP search, KernelEvidence, checked acceptance (Backup 5).

## Backup 10. Sources And Attribution

Exact MML excerpts used in this talk:

| Purpose | Source | Lines | Used in |
|---|---|---:|---|
| Function application `f.x` | `funct_1.miz` | 138-140 | Frame 2.3 |
| `Function of X,Y` as a soft type | `funct_2.miz` | 87-90 | Frame 2.3 |

Source URLs:

- `FUNCT_1`: <https://mizar.uwb.edu.pl/version/current/mml/funct_1.miz>
- `FUNCT_2`: <https://mizar.uwb.edu.pl/version/current/mml/funct_2.miz>

Attribution note:

- MML plain-text files state GPL-3.0-or-later / CC-BY-SA-3.0-or-later terms; keep article attribution, URLs, and line numbers in speaker notes.
- Benchmark figures: see Backups 1-2 and `references.bib`. Verify bibliographic metadata against publishers before the final deck.
