# TPP 2026 Spoken Script (English)

> **Status:** English companion to the reviewed Japanese script. Frame order follows `slides.md` and `slides.ja.md`; `draft.ja.md` remains the content authority.
> **Timing:** Headings give start times in the 36:30 allocation: introduction 9 minutes, six principles 23:30, closing 4 minutes. Add seven minutes of questions and 1:30 of buffer for an assumed 45-minute slot. This includes pointing at code and figures; confirm the slot and rehearse the English delivery. Do not exceed 40 minutes.
> **Claim labels:** [Literature], [Git history], [Specification], [Limited implementation], and [Plan / Future direction]. Presenter notes at the end are not spoken.

For 35 minutes, skip 3.3 (40 seconds), reduce 3.4 to 15 seconds (save 20), and reduce the design discussion in 0.4 to 30 seconds (save 30). Keep the timeline in 0.3 and the checking examples in 6.3–6.4. Backups are for questions only. In the full version, start 7.1 at 32:30 and finish at 36:30.

## Part 0. Introduction (0–9 minutes)

### 0.1 Title (0:00)

Mizar is a proof checker based on set theory. We write proofs in a language close to readable mathematics.
Mizar Evo redesigns this language and its tools. I will begin with the history and development, then show six design principles, the implementation status, and our next steps.

### 0.2 Mizar implementation history (0:30)

In 1973, the starting idea was to handle mathematical documents on a computer. The 1975 Mizar-PC handled propositional logic. Here, PC means propositional calculus. It is different from the later PC-Mizar for personal computers. The early version used Algol on ODRA. Mizar-QC added quantifiers in 1977, using Pascal on CDC. [Historical paper]
In the early 1980s, experiments included MSE, with a smaller language, and HPF, with more mathematical notation and implicit arguments. These are selected milestones, not a single line of successors.
Mizar-4 in 1986 used several passes linked by files to work with limited memory. It was ported to PCs at the end of that year. PC-Mizar followed in 1988, and MML began the next year. Articles could refer to each other and build mathematics together. [Historical paper]
Readable writing and library reuse grew through these experiments and redesigns.

Mario Carneiro's Mizar-rs reimplemented the analyzer and checker in Rust. Both parts affect soundness. [Carneiro 2023]
There are three main results. First, its checker replaced the existing checker and checked the full MML of that time. Second, it was about five to six times faster in the paper's eight-thread elapsed-time comparison. Third, it found and reported four soundness bugs in the existing implementation. This does not mean those bugs were used to prove false statements in MML. [Table 1, §5]
The reimplementation improved speed and helped improve the existing system. Proof output for a small trusted base remained future work. Against this background, Mizar Evo redesigns the language specification and builds the processor in Rust. [Literature / Evo design]

### 0.3 From an idea to language design (3:00)

The timeline shows the move from an idea to language design and implementation. [Git history]
The first README, in August 2023, was called mizar-wishlist. By autumn 2024, I was writing concrete designs for imports, namespaces, structure inheritance, and the type system.
In 2025, I worked on mode syntax, type inference and checking, and redefinition of predicates and functors. Features were also removed and reconsidered. Drafts for dependent structures and attributes were removed, then brought back after clarifying their relation to templates. [Git history]
From January to April 2026, the work covered EBNF, logical encoding, registration, templates, and algorithm contracts and semantics. In May, I designed the implementation pipeline, added a test harness on the 17th, and a minimal lexer on the 18th. [Git history]

The original idea was to extend Mizar notation. Preparing a grant proposal in 2023 led me to redesign both the language specification and the processor.
At ITP that year, I asked Mario about helping with implementation after the specification was ready. He responded positively. At CICM 2023, I discussed the wishlist with Adam Naumowicz.
The proposal focused on universal quantification over types, namespaces and packages, ATP, and traceability. The planned prototype used Mizar-rs. “Universal quantification over types” was the proposal's wording; the current specification uses templates over a first-order foundation. [Grant proposal / Specification]

### 0.4 Design development and AI agents (6:30)

Generics were part of the template idea from the start. The design grew to express traditional schemes as theorems inside templates. Structure views and bounds on type parameters let one theorem serve, for example, both addition and multiplication in a ring. Type-argument inference was also developed. The scope grew from replacing types to sharing definitions and proofs. [Specification 18 / Git history]
There was also an early demand for algorithms. Many members of the Japanese Mizar community work in cryptography and wanted to prove algorithms. I also experienced difficult symbolic calculations while proving elliptic-curve associativity with formulas. We had wanted to bring computation into proofs for a long time. This led to the design of contracts, termination, partiality, and conditions for using verified computation as a functor.

Progress in AI coding agents made it possible to start substantial Rust implementation in May 2026 without waiting for the implementation help first considered.
In June and July, the work covered crates, autonomous development procedures, SAT-backed checking, and soundness audits. Implementation and tests revealed unclear parts of the specification and led back to design. [Git history / Experience]
In September, I gave a seminar and discussed the design with the Mizar team in Białystok. Algorithm specification changes continued in autumn. [Git history]

### 0.5 Six challenges and design principles (8:30)

This work led to six design principles: foundation, writing, generics, computation, development tools, and checking and automation.
We keep first-order logic, set theory, and readable mathematics, while redesigning templates, algorithms, tools, and evidence checking. Most examples that follow explain the specification. I will show the implemented scope at the end. [Challenges / Specification]

## Part 1. Logical Foundation (9–11 minutes)

### 1.1 Keep the foundation, rebuild the tools (9:00)

Mizar Evo keeps the mathematical language and rebuilds the tools below and around it.
We keep first-order logic and Tarski–Grothendieck set theory as the proof foundation. We also keep soft types, modes, attributes, registrations, structures, and declarative proofs.
ATP means automated theorem prover. We separate its search from independent checking of its results by a small kernel. This preserves the logic while keeping the trusted base small. Section 6 will show how checking works.
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

We can write inherit after declaring the structure. Separating the type definition from the implementation of a relation resembles a Rust trait implementation.
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
The specification's functor id. M returns the identity of a unital Magma. Writing only id. R leaves a choice: the additive or multiplicative Magma view?
R qua AddMagma gives R.zero. R qua MulMagma gives R.one. Each selected view must have an identity.
The view lets us select zero or one within the same ring. Types and attributes decide which definitions apply; the chosen operation view must also be clear. [Specification-based sketch]
