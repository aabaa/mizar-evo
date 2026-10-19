# TPP 2026 Spoken Script (English)

> **Status:** English companion to the reviewed Japanese script. Frame order follows `slides.md` and `slides.ja.md`; `draft.ja.md` remains the content authority.
> **Timing:** Headings give start times in the 36:30 allocation: introduction 9 minutes, six principles 23:30, closing 4 minutes. Add seven minutes of questions and 1:30 of buffer for an assumed 45-minute slot. This includes pointing at code and figures; confirm the slot and rehearse the English delivery. Do not exceed 40 minutes.
> **Claim labels:** [Literature], [Git history], [Specification], [Limited implementation], and [Plan / Future direction]. Presenter notes at the end are not spoken.

For 35 minutes, skip 3.3 (40 seconds), reduce 3.4 to 15 seconds (save 20), and reduce the design discussion in 0.4 to 30 seconds (save 30). Keep the timeline in 0.3 and the checking examples in 6.3–6.4. Backups are for questions only. In the full version, start 7.1 at 32:30 and finish at 36:30.

## Part 0. Introduction (0–9 minutes)

### 0.1 Title (0:00)

Mizar is a proof checker based on set theory. It uses readable mathematical language and the Mizar Mathematical Library, or MML.
Mizar Evo redesigns the language and tools. I will cover the history, six design principles, implementation status, and next steps.

### 0.2 Mizar implementation history (0:30)

In 1973, the starting idea was to handle mathematical documents on a computer. The 1975 Mizar-PC handled propositional logic. Here, PC means propositional calculus. It is different from the later PC-Mizar for personal computers. The early version used Algol on ODRA. Mizar-QC added quantifiers in 1977, using Pascal on CDC. [Historical paper]
In the early 1980s, experiments included MSE, with a smaller language, and HPF, with more mathematical notation and implicit arguments. These are some key stages in the history. The systems did not simply replace each other in order.
Mizar-4 in 1986 used several passes linked by files to work with limited memory. It was ported to PCs at the end of that year. PC-Mizar followed in 1988, and MML began the next year. Articles could refer to each other and build mathematics together. [Historical paper]
Readable writing and library reuse grew through these experiments and redesigns.

Mario Carneiro's Mizar-rs reimplemented the analyzer and checker in Rust. Both parts affect soundness. [Carneiro 2023]
There are three main results. First, its checker replaced the existing checker and checked the full MML of that time. Second, it was about five to six times faster. The paper compared elapsed time with eight threads. Third, it found and reported four soundness bugs in the existing implementation. This does not mean those bugs were used to prove false statements in MML. [Table 1, §5]
The reimplementation improved speed and helped improve the existing system. Proof output for a small trusted base remained future work. Mizar Evo redesigns the language specification and builds the processor in Rust. [Literature / Evo design]

### 0.3 From an idea to language design (3:00)

The timeline shows the move from an idea to language design and implementation. [Git history]
The first README, in August 2023, was called mizar-wishlist. By autumn 2024, I was writing concrete designs for imports, namespaces, structure inheritance, and the type system.
In 2025, I worked on mode syntax, type inference and checking, and redefinition of predicates and functors. Features were also removed and reconsidered. Drafts for dependent structures and attributes were removed, then brought back after clarifying their relation to templates. [Git history]
From January to April 2026, the work covered EBNF, logical encoding, registration, templates, and algorithm contracts and semantics. In May, I designed the implementation pipeline, added a test harness on the 17th, and a minimal lexer on the 18th. [Git history]

The original idea was to extend Mizar notation. Preparing a grant proposal in 2023 led me to redesign both the language specification and the processor.
At ITP that year, I asked Mario about helping with implementation after the specification was ready. He responded positively. At CICM 2023, I discussed the wishlist with Adam Naumowicz.
The proposal focused on universal quantification over types, namespaces and packages, automated proof search, and traceability. Traceability means recording how the system makes its choices. The planned prototype used Mizar-rs. “Universal quantification over types” was the proposal's wording; the current specification uses templates over a first-order foundation. [Grant proposal / Specification]

### 0.4 Design development and AI agents (6:30)

Generics were part of the template idea from the start. The design grew to express traditional schemes as theorems inside templates. Structure views and bounds on type parameters let one theorem serve, for example, both addition and multiplication in a ring. Type-argument inference was also developed. The scope grew from replacing types to sharing definitions and proofs. [Specification 18 / Git history]
There was also an early demand for algorithms. Many members of the Japanese Mizar community work in cryptography and wanted to prove algorithms. I also experienced difficult symbolic calculations while proving elliptic-curve associativity with formulas. We had wanted to bring computation into proofs for a long time. This led to the design of contracts, termination, partiality, and conditions for using verified computation as a functor.

With AI coding agents, I could begin Rust implementation in May 2026. I no longer needed to wait for the implementation help I had first considered.
In June and July, the work covered crates, agent development procedures, SAT-backed checking, and soundness audits. Implementation and tests revealed unclear parts of the specification and led back to design. [Git history / Experience]
In September, I gave a seminar and discussed the design with the Mizar team in Białystok. Algorithm specification changes continued in autumn. [Git history]

### 0.5 Six challenges and design principles (8:30)

This work led to six design principles: foundation, writing, generics, computation, development tools, and checking and automation.
We keep first-order logic, set theory, and readable mathematics, while redesigning templates, algorithms, tools, and evidence checking. Most examples that follow explain the specification. I will show the implemented scope at the end. [Challenges / Specification]

## Part 1. Logical Foundation (9–11 minutes)

### 1.1 Keep the foundation, rebuild the tools (9:00)

Mizar Evo keeps the mathematical language and rebuilds the tools below and around it.
We keep first-order logic and Tarski–Grothendieck set theory as the proof foundation. We also keep soft types, modes, attributes, registrations, structures, and declarative proofs.
ATP means automated theorem prover. We keep its search separate from checking. A small kernel, the core proof checker, checks the ATP results independently. This preserves the logic while keeping the trusted base small. Section 6 will show how checking works.
Section 2 covers mathematical writing, section 3 generics, section 4 computation, and section 5 development tools. [Specification]

### 1.2 From sets to topological spaces (10:00)

Here are two examples of the mathematical writing we want to keep. Please focus on the assumptions and conclusions.
The first is Cantor's theorem: the power set of X has a larger cardinality than X. Sets, power sets, and cardinals are all objects in the same first-order set theory. Cardinals are represented as ordinals, so in expresses the strict inequality here.
The second theorem says that a continuous bijection from a compact space to a Hausdorff space is a homeomorphism. TopSpace is a structure; compact and T_2 are attributes. The statement gives conditions on the map and its conclusion.
We keep this way of writing mathematics and redesign the processor that supports search and checking. [CARD_1:14 / COMPTS_1:17]

## Part 2. Readable Mathematics (11–16 minutes)

### 2.1 Trace type, registration, and overload choices (11:00)

We follow implicit decisions in this order: type, registration, and overload.
Which types can an object be used as? Which rules supplied the required attributes? Which definition and view were selected? We keep natural writing and automatic processing, while making the reasons visible. First, let us look at structure declarations and inheritance. [Specification]

### 2.2 type: declare fields and properties (11:30)

These declarations define LoopStr and its additive version, AddLoopStr. The carrier and operation are stored fields. They are data passed to the constructor.
The properties unit and zero declare only names and types. Their values are not defined here. We first declare how the two structures are related by inheritance, then implement the property. [Specification]

### 2.3 type: declare inheritance after the structure (12:15)

We can write inherit after declaring the structure. The type definition and inheritance are separate. This is similar to a Rust trait implementation.
The parent's binop maps to the child's add; its property unit maps to the child's zero. There is one inherit declaration per parent. Equal types need no proof; narrowing a type needs a coherence proof.
Next, we use this mapping to implement unit on the parent side. [Specification]

### 2.4 type: implement unit for Unital, reuse it as zero (13:00)

The attribute has_unit says that a two-sided identity exists. It does not refer to unit itself. The mode Unital selects LoopStr objects with this attribute. We then implement unit with means. Existence follows from the mode assumption; uniqueness follows by applying the operation to two identities.
If AddLoopStr's selected LoopStr view is Unital, the zero from unit mapping lets us reuse this implementation as zero. We do not need a separate implementation for zero. [Specification-based sketch, proofs omitted]

### 2.5 registration: automatic chains hide dependencies (13:45)

Next is registration. Declaring a set S to be empty automatically gives finite, then countable. At the point of use, we do not have to cite these rules with by.
This is useful, but when registrations form a chain, it becomes difficult to see which rules produced countable from that source alone. This is the traceability problem.
The new specification keeps automatic application but requires labels and records the path from EmptyImpliesFinite to FiniteImpliesCountable. It can explain the dependency chain, as well as the final attribute. [Specification]

### 2.6 overload: two identities in the same ring (14:45)

Finally, overload. Let us use the identity element for addition and multiplication in a ring.
The additive identity is zero, and the multiplicative identity is one. Both are identities, but their values differ.
A functor is a function symbol in Mizar. Here, id. M returns the identity of a unital Magma. Writing only id. R leaves a choice: the additive or multiplicative Magma view?
R qua AddMagma gives R.zero. R qua MulMagma gives R.one. Each selected view must have an identity.
The view lets us select zero or one within the same ring. Types and attributes decide which definitions apply; the chosen operation view must also be clear. [Specification-based sketch]

## Part 3. Generic Mathematics (16–19 minutes)

### 3.1 Current MML: more types, more declarations (16:00)

Next, templates. Adding functions means adding their values at each point. Current MML already defines this body once.
But preserving the result type requires separate redefinitions and registrations for complex, real, rational, integer, and natural values. Each also needs coherence proofs.
The sum is the same, but each new type adds surrounding declarations and proofs. The next example uses a template to share this work. [VALUED_1]

### 3.2 template: share the body and result type (16:45)

We take an additive structure T and a common nonempty domain I as parameters. One definition gives both the body and the result type of function addition.
From real to natural values, we can use the same Add with the corresponding additive structure. Type-specific closure facts and registrations are still needed, but we do not repeat the sum and its result type.
The synonym supplies + notation. Here, declared types determine T and I uniquely, so we can write f+g or u+v without arguments. Ambiguous inheritance paths need qua.
Current MML also handles intersections of different domains. This example is restricted to a common nonempty domain. [Specification-based sketch, proofs omitted]

### 3.3 Unify schemes with template theorems [deep dive] (17:45)

Traditional schemes also become theorems. Induction becomes NatInduction[P], with let P be pred(Nat). The predicate parameter gives a family of first-order theorems, one for each substituted predicate.
After defpred, we explicitly apply by NatInduction[P], Base, Step. Functor parameters are schema-level symbols, so the logic stays first-order. [Specification]

### 3.4 template: structures and attributes too (18:25)

The upper example gives vector-space data over a field K. The lower example describes density in a space M.
Familiar of notation and prefix attributes are syntactic sugar for templates: shorter ways to write the same expressions. Vector of K corresponds to Vector[K]; M-dense corresponds to dense[M]. We can also write VectorSpace over K.
Structures, attributes, modes, and other declarations share this parameter mechanism. [Specification-based sketch]

## Part 4. Verified Computation (19:00–24:15)

### 4.1 Computation: verified algorithms (19:00)

P is the precondition; Q is the postcondition. Hoare logic checks that if P holds before a statement, Q holds when it returns.
Assignment replaces x in Q with e. If splits into B and not B. For while, check three things: I holds on entry; the body preserves I; and I with not B gives Q on exit.
Applied recursively, these rules give first-order proof obligations, called verification conditions or VCs. Termination is checked separately. [Basic specification rules]

### 4.2 algorithm: contracts, proofs, computation (19:45)

This is Euclid's algorithm. Requires states what must hold before the call. Ensures states what must hold after it returns. Together they form the contract. The loop has invariant and decreasing annotations. Gcd is the mathematical functor.
EuclidGcdDef is the definition label; euclid_gcd is the callable name. Terminating claims termination for every input satisfying requires. We check proof obligations generated from the invariant and measure.
After verification, it can be used as a mathematical functor in formulas and proofs. [Specification]

### 4.3 Euclid: code and proof obligations (21:00)

Here is the body again. Match the numbered code points to the table.
At one, initializing x=a and y=b establishes the invariant from the precondition. At two, the invariant must survive computing r=x mod y and replacing x,y by the old y,r. We use the lemma Gcd(x,y)=Gcd(y,r).
At three, the measure is y. It is positive when we enter the loop. The remainder is nonnegative and smaller than the old y, so the measure strictly decreases. At four, y is zero. The invariant and Gcd(x,0)=x show that the returned value is the GCD of the inputs.
We check these obligations with library lemmas. [Specification-based explanation]

### 4.4 terminating and functor promotion (22:15)

Partial correctness is the default. Once terminating is verified, the algorithm returns for every input that meets requires. It can then be used as a functor.
The upper example cites the verified contract for positive natural numbers a and b. In the lower example, by computation runs the algorithm on 12 and 8 using MVM, the virtual machine. It uses the result, four, in the proof.
This default path also trusts MVM's execution result. It differs from kernel and SAT rechecking. MVM execution is not implemented; the examples explain the specification. [Specification / Implementation status]

### 4.5 Verified expression rewriting: lambda and differentiation (23:15)

Lambda notation writes a function directly. Here, u maps to sin u times u, for real u.
When passed as expr, a syntax tree, the algorithm can process products and sin while preserving variable binding. We prove differentiate's contract using differentiation rules. If it returns, the input is differentiable and the result gives its derivative. We then use that result in a derivative proof. The displayed conclusion also uses semantic conversion and simplification lemmas.
This connects function notation, expression transformation, and its correctness proof. It is a specification sketch; MVM execution remains future work. [Specification-based sketch]

## Part 5. Development Infrastructure (24:15–27:15)

### 5.1 Environment: what must an author import? (24:15)

The table shows all ten environment directives. The code above is shortened from ALGSTR_0. Even for the same article STRUCT_0, authors select resources by role: symbols, notation, constructors, registrations, and so on.
Notations and definitions are especially difficult. Overlapping notation and redefinitions have priorities. The order of articles can change which notation or definition is selected. Adding or reordering articles can affect existing text and proofs.
Authors manage both the imported resources and these priorities. Managing overrides and order dependence is the hardest part of writing a Mizar article. [Current environment]

### 5.2 Imports and dependency tracking (25:15)

The new specification imports a module's public items with import. The two example modules follow the specification. Original fully qualified names and registration traces are intended to show which items were actually used.
Dependency fingerprints record which dependencies were checked. They allow us to reuse checked results. Packages and lockfiles fix versions. The editor is designed to show resolved names and traces.
This is not a one-to-one translation of the old environment lists. We keep mathematical writing while building tools that explain dependencies and implicit processing. [Specification]

### 5.3 The whole picture [deep dive] (26:15)

Here is the whole design in one picture.
At the top is rich mathematics: structures, templates, algorithms, and modules. Below is a small first-order foundation. Around it are modern tools: the ATP layer, kernel, artifacts, versioned libraries, IDE, AI, and publication.
The design keeps mathematical writing and a small core, and rebuilds the tools. The features in this talk belong to these layers. [Specification]

## Part 6. Checking And Automation (27:15–32:30)

### 6.1 Automation: find lemmas and suggest a proof (27:15)

Isabelle's Sledgehammer finds lemmas and suggests proofs. We plan similar help in Mizar Evo.
Here, by atp is proposed syntax for requesting search. The exact syntax is not yet specified. Arguments may be added later.
For the goal Q(a) and local assumption P(a), the planned tool finds AllPQ. It tries the lemma with an ATP and suggests the citation below. We apply the citations in the source and verify again.
Search and suggestions help authors. Acceptance depends on kernel checks of formula and substitution evidence. [Specification / Plan]

### 6.2 Using ATPs: search, check, and reuse (28:15)

The flowchart shows where ATPs are used.
We send the goal and available premises to an ATP. The kernel checks the candidate evidence, and accepted results are stored in the library for reuse.
If the goal remains open, we revise lemmas or strategy and retry. An LLM can help with suggestions and repairs. The figure shows design intent, not a measured result about the cost and benefit of retries.
Normal verification uses source citations and local hypotheses. Lemmas found during assistance must also be written into citations and verified again. The next slide shows the evidence that supports acceptance. [Specification / Design intent]

### 6.3 Resolution tree and extracted substitutions (29:15)

F1 says P implies Q; F2 says Q implies R; H is P(a); the goal is R(a). Add the negated goal and derive a contradiction.
In the left tree, unifying Q(x) and Q(y) gives x:=y and the clause not P(y) or R(y). Then unifying P(y) and P(a) gives y:=a and R(a). Together with not R(a), this gives the empty clause.
On the right, combine x:=y with y:=a along the branch. This gives x:=a for the original F1 and y:=a for F2. Record which formulas and goal these substitutions belong to.
This is a proposal for getting candidate evidence from an external log. We do not trust the extractor; the kernel checks its output. The design also plans a separate instance finder that produces the same evidence format. It is not implemented. The kernel does not replay each inference in the log. [Design sketch / Unimplemented]

### 6.4 From saved evidence to concrete SAT clauses (30:45)

SAT tests whether propositional clauses can all be true at once. UNSAT means they cannot. The top row shows saved evidence, kernel checks, and instantiation. The kernel checks original formulas and composed substitutions against their origins, target, and binding context. It checks that free variables do not become bound by mistake. It then instantiates both implications at a and adds P(a) and the negated goal.
Assign propositional variables one, two, and three to P(a), Q(a), and R(a). Introduce variables four and five for the implications, with unit clauses that say they are true. This Tseitin encoding follows the current encoder's construction.
The two blue boxes below show definitions of s and t. Each gives three defining clauses and one unit clause asserting the implication. With the unit clauses p and not r above, we get five variables and ten clauses. As the green band shows, p and the two implications force r, contradicting not r.
The small kernel does not replay external resolution steps. It generates clauses from checked evidence and asks a trusted SAT checker to confirm UNSAT. The displayed clauses illustrate the encoding; they are not a dump from an execution. [Architecture-based design sketch]

## Part 7. Status And Roadmap (32:30–36:30)

### 7.1 Implementation status and size (32:30)

The unit is kLoC: one thousand lines. At fixed snapshot 08861708, Rust product code is 271.1 kLoC, excluding tests, blank lines, and comments. The completion estimate of 360 to 540 kLoC includes the current code. It is provisional, not a completion percentage or effort forecast. MML porting is separate. [Measurement / Provisional estimate]
The table separates ATP search from the kernel that accepts proofs. The kernel is 12.8 kLoC, including 1.9 for SAT encoding and connection. Adding the SAT solver and its dependency gives 17.6 kLoC for checking. Core, session, and shared support are counted separately, so this is not the size of the full minimum TCB. [Counting boundary]
Components exist, but connecting general source input through acceptance and artifact storage remains unfinished. LSP semantic connections, MVM execution, extraction, and documentation generation also remain future work. [Limited implementation / Plan]

### 7.2 Next steps and summary (34:00)

Our first priority is to complete these connections: source semantics, VC generation, evidence, kernel acceptance, artifacts, then builds. We will expand the working paths in this order. Unsupported inputs must continue to be rejected. We will also keep aligning algorithm semantics and proof obligations. [Plan]
MVM runtime and code extraction are separate development tasks. Beyond them, we are considering learned premise selection, LLM repair, cryptography, and quantum algorithms. These are long-term directions without completion dates. [Future direction]

We keep Mizar's first-order logic, set theory, and ability to read and write mathematics. On this foundation, we redesign templates, algorithms, and development tools, and separate ATP search from trusted checking.
Several years of language design came first. Implementation and tests with AI agents now lead back to specification changes. The specification is still evolving. I would welcome requests from ITP users: what is difficult in your daily work, and what would you like in notation, proof assistance, or development tools?
Thank you. We now have time for questions.

## Questions and backup choices

- **Which is stronger: 58.4% or 68.8%?** The evaluation units, premises, budgets, and success criteria differ. They do not rank the designs. Backup 4.
- **Is first-order logic better for ATP?** This talk does not rely on that claim. Encoding and reconstruction explain the connection; performance needs separate evaluation. Backup 5.
- **Do templates really stay first-order?** Chapter 18 treats predicate and functor parameters at schema level. Each instance gives first-order obligations. Backup 1.
- **Is algorithm just a tactic language?** It is itself subject to contracts and verification. Tactic-like uses are included, but they are specified and verified too. MVM and extraction are unimplemented.
- **How much is rechecked after a change?** Public claims, acceptance status, and dependency fingerprints determine reuse. Backup 2; package and lockfile example in Backup 3.
- **What does the kernel trust?** Main frame 6.4: check evidence correspondence and substitutions, then confirm UNSAT for the kernel-generated SAT problem. Details: `draft.ja.md` §6. Default computation additionally trusts MVM; see 4.4.
- **What works now?** The limited implementation in 7.1. Connection details are in the question-time notes of `draft.ja.md` §7.1. No general source-to-artifact or end-to-end external-prover result is claimed. Recheck the fixed SHA, `doc/design/todo.md`, and source before the talk.

## Presenter notes (do not read aloud)

- 0.3: the 2023 consultations describe events at that time. Do not imply current participation commitments, a joint-design agreement, or a precise order within the year.
- 0.4: AI implementation progress is qualitative experience; no speed multiplier was measured. A causal link between particular Białystok comments and design changes is unconfirmed.
- In the 35-minute version, replace the two design-development paragraphs in 0.4 with the paragraph below. Do not read it in addition to them.

Templates grew from generics to unifying schemes and theorems, with structure views and type-argument inference. Algorithms answer a long-standing demand to bring computation into proofs, including cryptographers' algorithm proofs and symbolic calculations in elliptic-curve associativity.
