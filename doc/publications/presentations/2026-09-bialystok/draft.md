# Mizar Evo: Third Draft For The Bialystok Mizar Team

Status: third content draft, rewritten around problems and solutions.

Planned occasion: visit to the Mizar team in Bialystok in September 2026.

Companion Japanese draft: `draft.ja.md`.

## Main Idea

Present Mizar Evo as a project that continues the Mizar tradition, not as a
replacement. The main idea is:

> Mizar Evo keeps Mizar's readable mathematical vernacular. It rebuilds the
> language boundary, verifier pipeline, artifact model, and publication
> workflow. These changes help us maintain large-scale formal mathematics
> with predictable automation, AI assistance, and reproducible verification.

The talk has eight stories about problems and solutions. Each starts with
a real cost in current Mizar work. It shows the Mizar Evo answer in code and
states what we keep. We show the language through examples, without grammar
notation (EBNF). The specification files under `doc/spec/en/` remain the
authority for edge cases.

## Code Status Convention

Every code example carries one of three status labels:

- "exact MML excerpt": exact text from the current MML, with article name
  and line numbers; attribution and license notes stay in speaker notes.
- "specification example": taken from or directly adapted from the Mizar Evo
  language specification under `doc/spec/en/`.
- "sketch": an example that is not yet fixed by the specification.

## Source Status

### Repository Sources

- `doc/spec/en/05.structures.md`, `06.attributes.md`, `07.modes.md`
- `doc/spec/en/12.modules_and_namespaces.md`
- `doc/spec/en/17.clusters_and_registrations.md`
- `doc/spec/en/18.templates.md`
- `doc/spec/en/20.algorithm_and_verification.md`
- `doc/spec/en/21.source_code_annotation_and_atp.md`
- `doc/spec/en/23.package_management_and_build_system.md`
- `doc/spec/en/sample_codes.md`
- `doc/design/architecture/en/00.pipeline_overview.md`
- `doc/design/architecture/en/08.reasoning_boundary.md`
- `doc/design/architecture/en/15.kernel_certificate_format.md`
- `doc/design/architecture/en/21.ai_agent_interface.md`

### External Sources

Checked on June 18, 2026 unless noted:

- Mizar home page: Mizar 8.1.15 and MML 5.94.1493, dated May 30, 2025.
  <https://mizar.uwb.edu.pl/>
- `ALGSTR_0` plain text (exact excerpts, lines 15-25, 37-40, 104-109):
  <https://mizar.uwb.edu.pl/version/current/mml/algstr_0.miz>
- `STRUCT_0` plain text (exact excerpt, lines 637-643):
  <https://mizar.uwb.edu.pl/version/current/mml/struct_0.miz>
- `NAT_1` plain text (scheme `NatInd`, near line 90; checked July 2, 2026):
  <https://mizar.uwb.edu.pl/version/current/mml/nat_1.miz>

The MML plain-text files state GPL-3.0-or-later / CC-BY-SA-3.0-or-later
distribution terms; the final deck must keep article attribution, source URLs,
and line numbers in speaker notes.

Specification examples follow `doc/spec/en/`. They assume the required
imports and earlier declarations; `...` marks omitted proof text.
Kernel evidence follows Chapter 21, §21.7, and the current evidence design
in `doc/design/architecture/en/15.kernel_certificate_format.md`.

## Deck Shape

Thirteen sections: opening, motivation, eight stories, architecture overview,
roadmap, closing. Keep the detailed deck for readers to review after the talk.
For the 45-minute visit, explain selected examples and cover the other slides
briefly. Whether the slot includes questions is still to be confirmed.

Frames marked `[deep dive]` can be skipped without losing the main points.
Keep each story's questions for later review. Choose two or three questions
for discussion at the end.

For the 45-minute overview, read the sentences marked `**...**` from top to
bottom. They appear in dark-blue bold text in both PDFs. Start with the title-page
notes; after that, slides without highlighted prose can be skipped. Other text,
code, tables, and "for later review" sections remain available in the handout.
Code syntax colors do not mark talk priorities. The notes keep the same emphasis
under "Read aloud". See README.md, Two-Tier Pacing, for the provisional
40-minute explanation plus five-minute question/extra-explanation budget.

## Part 0. Opening

### Frame 0.1 - Title

Title:

```text
Mizar Evo
Readable, AI-Ready, Scalable Formal Mathematics
```

Subtitle:

```text
Eight problems, eight proposals
A discussion with the Bialystok Mizar team, September 2026
```

Speaker note:

- **Thank you for inviting me to Bialystok. Today I will introduce Mizar Evo.**
- **Mizar is a system for writing mathematical proofs and checking them by computer.
  Evo is a project to update its language and tools while keeping proofs readable.**
- **I will give an overview. The handout keeps the details for later review.**
- The design is still open. I would welcome your comments and objections.

### Frame 0.2 - A First Look

Current Mizar names the parent structure (exact MML excerpt):

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

Evo also maps the fields to a common Magma view (specification example):

```mizar
definition
  struct AddMagma where
    field carrier -> set;
    field add -> BinOp of carrier;
  end;

  inherit AddMagma extends Magma where
    field carrier from carrier;
    field add from binop;    :: renamed view
  end;
end;
```

Speaker note:

- Both examples describe a carrier and a binary operation.
  Current Mizar already names the parent, `1-sorted`.
  The Evo example adds a view of the same fields as a `Magma`.
- Source: current MML `algstr_0.miz`, lines 37-40.

### Frame 0.3 - The Proposal In One Sentence

Slide text:

```text
**Preserve Mizar's mathematical vernacular.**
**Update the compiler, verifier, artifact, and publication layers.**
```

- This is the main proposal. We build on what current Mizar has achieved.
- **The language standard is still a draft. Full MML migration is not complete.**
- AI assistance must never replace proof checking.
- **The Evo examples follow the specification. Some are not yet implemented.
  I will explain the current implementation scope near the end.**

### Frame 0.4 - How To Read The Examples [deep dive]

Every code example is labeled:

| Label | Meaning |
|---|---|
| exact MML excerpt | exact current MML text, with article and line numbers |
| specification example | from the Mizar Evo language specification |
| sketch | an example; not yet fixed by the specification |

Reading rule:

- No EBNF appears in this talk. The language is shown through examples only;
  `doc/spec/en/` remains the authority for grammar and edge cases.
- Examples assume required imports and earlier declarations. `...` marks
  omitted proof text.

### Frame 0.5 - What We Need From This Visit

- **The purpose of this visit is to learn from your experience with Mizar.**
- I would like to identify compatibility needs and choose small MML articles
  for migration experiments.
- **I also hope to discuss how we check results from automated theorem provers, or ATPs.
  Another topic is how we link our work to Formalized Mathematics.**
- My main question is this.

Main question:

```text
**What must Mizar Evo preserve so that the Mizar community**
**still recognizes it as Mizar?**
```

Speaker note:

- There may be constraints that are hard to see without daily MML work.
  Please tell me about objections I have missed.

## Part 1. Why Now

### Frame 1.1 - What Mizar Got Right

- **First, let me explain what I want to keep.**
- **Mizar proofs are declarative and read as mathematics.**
- Soft types, modes, and attributes give us a rich mathematical vocabulary.
- **Registrations and clusters provide automation that other proofs can reuse.**
- **The Mizar Mathematical Library, or MML, is large and carefully maintained.
  Formalized Mathematics
  provides a way to publish this work.**
- I want to judge each Evo proposal by how well it protects these strengths.

Speaker note:

- The library version cited here is MML 5.94.1493, with 1493 articles.

### Frame 1.2 - Pressure One: Scale

- **The first reason for this work is library size.**
- MML has roughly 1500 articles that depend on each other.
- **An article is the unit of dependency, review, and reuse.**
- Tools resolve article environments, but the resolved dependencies are
  not visible in the source.
- As the library grows, renaming and revising articles can affect more work.
  Evo asks whether smaller boundaries would help us maintain the library.

### Frame 1.3 - Pressure Two: Tooling Expectations

- **The second reason is the way people now expect tools to work.**
- **We expect editors to give quick feedback, even while the source is incomplete.**
- **We expect a manifest and lockfile to make builds reproducible.**
- We expect packages with versions and documentation that we can browse.
- These features are common in programming tools. I want them to support
  work on formal libraries too.

### Frame 1.4 - Pressure Three: AI

- **The third reason is AI assistance.**
- **AI tools can help us search, explain, and edit proofs.**
- They need a small amount of structured context linked to the source.
  They should not need a copy of the whole library for each task.
- **Their output must still be checked. Proof validity must not depend on
  how capable the AI tool is.**
- Mizar's readable source helps here. Stable local text patterns make
  editing and retrieval easier.

### Frame 1.5 - Three Pressures, One Design

![Three pressures on a proven design](figures/three_pressures.pdf)

- This diagram brings the three reasons together: library size, tools, and AI.
- They do not mean that Mizar's design was wrong.
- They explain why I want to update some boundaries while keeping
  Mizar's readable mathematical language.

### Frame 1.6 - The Design Rule

Slide text:

```text
Do not make proofs harder to read to add automation.
Use automation to protect and extend readability.
```

- This gives us three questions. Can people still read the proof?
  Can tools inspect its context? Will the design work as the library grows?
- The next eight stories apply these questions to specific problems.

Details for later review:

| Goal | Test for every feature |
|---|---|
| Readability | does proof text still read as mathematics? |
| AI-readiness | can a tool see a limited context that we can audit? |
| Scalability | do boundaries stay stable as the library grows? |

## Part 2. Story 1: Dependencies You Can See

### Frame 2.1 - The Problem

This environment lists dependencies (exact MML excerpt):

```mizar
environ

 vocabularies XBOOLE_0, SUBSET_1, BINOP_1, ZFMISC_1, STRUCT_0, ARYTM_3,
      FUNCT_1, FUNCT_5, SUPINF_2, ARYTM_1, RELAT_1, MESFUNC1, ALGSTR_0, CARD_1;
 notations TARSKI, XBOOLE_0, SUBSET_1, ZFMISC_1, BINOP_1, FUNCT_5, ORDINAL1,
      CARD_1, STRUCT_0;
 constructors BINOP_1, STRUCT_0, ZFMISC_1, FUNCT_5;
 registrations ZFMISC_1, CARD_1, STRUCT_0;
 theorems STRUCT_0;

begin :: Additive structures
```

- **Our first story is about dependencies. These lists tell Mizar which
  library material an article needs.**
- **This form has supported library growth for decades. The question is how
  to make dependencies easier to review in a larger library.**

Speaker note:

- Source: current MML `algstr_0.miz`, lines 15-25.

### Frame 2.2 - Why This Is Difficult [deep dive]

Bullets:

- Information about a symbol's origin is spread across several role lists.
  A reviewer cannot see which article provides which notation, constructor,
  or cluster.
- The Accommodator resolves the environment. But the resolved dependencies
  are not source text that a person can review.
- Tools cannot cache or invalidate units smaller than an article.
- Moving a theorem between articles risks breaking unknown dependents.

Message:

- Implicit dependencies add work to every edit, review, and tool.
  That cost grows with the library.

### Frame 2.3 - The Evo Answer: Import Prelude

**Evo puts imports before definitions** (specification example):

```mizar
import .function;
import mml.algebra.structure.sorted;

definition
  let S be 1-sorted;
  mode BinOpDef: BinOp of S is
    Function of [: S.carrier, S.carrier :], S.carrier;
end;
```

- **Here, the imports come before the first non-import item.**
- **They provide the initial active lexicon. Local declarations extend it
  after their declaration points.**
- **Imported items keep their source FQNs. Package and module paths give
  each item a stable fully-qualified name.**

### Frame 2.4 - The Evo Answer: Packages [deep dive]

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

Bullets:

- reproducible builds need fixed source, lockfile, toolchain, and verifier
  settings, including deterministic ATP evidence;
- versioned reuse (SemVer) replaces manual copying between article sets.

### Frame 2.5 - Migrating The Environment [deep dive]

![Environment-to-import migration map](figures/environ_migration.pdf)

Message:

- Migration changes more than names: reports must show each module's
  syntax, semantics, and automation.

### Frame 2.6 - What Is Preserved, What We Ask

- **The aim is to preserve the mathematics and theorem identities during migration.**
- A module is still a readable text, with article-style authorship.
- Origin metadata records where migrated items came from.
  I will return to this when I discuss publication.
- **The questions below are for later review. I will now turn to structures.**

Questions for later review:

- Which `environ` roles must remain visually familiar during migration?
- Which current article dependencies are hardest to explain, and would make
  the best test cases for generated dependency reports?

## Part 3. Story 2: Structures Without Hidden Merges

### Frame 3.1 - The Problem

This structure uses the familiar compact form (exact MML excerpt):

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

- **One declaration contains the parent link, fields, and selectors.**
- With several parents, it also determines which inherited fields are shared.
- Additive and multiplicative views depend on naming conventions.
- **The syntax does not separate stored data from canonical values such as zero.**
- Evo proposes a more explicit way to state these choices.

Speaker note:

- This short form keeps structures close to informal mathematical writing.
- Source: current MML `algstr_0.miz`, lines 37-40.

### Frame 3.2 - Why This Is Difficult [deep dive]

Bullets:

- With diamond inheritance, readers need to trace which inherited selectors
  are shared. Evo proposes explicit member mappings for these paths.
- The syntax does not tell a migration tool whether a selector is intrinsic
  data, a canonical value with obligations, or an inherited view.
- Errors appear far from their cause, as type mismatches in later articles.

Message:

- At MML scale, structure inheritance is a graph maintenance problem, and the
  graph needs explicit edges that the verifier can check.

### Frame 3.3 - The Evo Answer: Field, Property, Attribute

**Evo separates fields, properties, and attributes** (specification example):

```mizar
definition
  struct AddLoopStr where
    field carrier -> set;
    field add -> BinOp of carrier;
    property zero -> Element of carrier;
  end;
end;
```

- **A field stores data, like `carrier` and `add` here. Fields are constructor
  arguments and determine equality of exact instances.**
- **A property, like `zero` here, gets its value from an implementation.
  The declaration alone supplies no value. `means` requires existence and
  uniqueness; `equals` gives a term.**
- **An attribute is a predicate-style refinement. It supports cluster propagation
  and does not change the layout.**

Speaker note:

- Overlapping property implementations need `coherence`.

### Frame 3.4 - The Evo Answer: Explicit Inheritance

**Evo writes each parent mapping explicitly** (specification example):

```mizar
definition
  struct AddMagma where
    field carrier -> set;
    field add -> BinOp of carrier;
  end;

  inherit AddMagma extends Magma where
    field carrier from carrier;
    field add from binop;    :: renamed
  end;
end;
```

- **There is one `inherit` statement for each parent.**
- **Here, `from` maps the child's `add` field to the parent's `binop` field.**
- **Syntactically identical member types need no proof, even if a name changes.
  Other types need `coherence` proofs of subtype inclusion.**

### Frame 3.5 - The Evo Answer: Diamonds Become Checkable

This child has two parent structures (specification example):

```mizar
definition
  struct DoubleLoopStr where
    field carrier -> set;
    field add -> BinOp of carrier;
    field mul -> BinOp of carrier;
    property zero -> Element of carrier;
    property one -> Element of carrier;
  end;

  inherit DoubleLoopStr extends AddLoopStr;
  inherit DoubleLoopStr extends MulLoopStr;
end;
```

The diagram shows the inheritance paths (sketch):

![The diamond, with checkable joins](figures/diamond_inheritance.pdf)

- Same-name, same-type members can join across roots. Different member types need
  `coherence` proofs of subtype inclusion.
- Renamed views stay distinct. Invalid mappings or missing proofs give diagnostics.

Speaker note:

- The diagram assumes the shown mappings from `AddLoopStr` and `MulLoopStr`
  to `Magma`. The code adds their shared child, `DoubleLoopStr`.

### Frame 3.6 - What Is Preserved, What We Ask

- We still use carriers, selectors, and `Element of`, as in Mizar.
- Aggregates become longer only when we need to make a hidden choice explicit.
- **The main question is whether this form is readable in real algebraic articles.**
- **I would like to test it on structures with several inheritance paths.
  Next, I will discuss registrations and clusters.**

Questions for later review:

- Is the `field` / `property` / `attribute` split readable in real algebraic
  articles, or does it require too many annotations in simple cases?
- Is one-parent-per-`inherit` acceptable for parts of MML with much
  inheritance, such as the `ALGSTR` and topology hierarchies?
- Which MML structures would be the best diamond test cases?

## Part 4. Story 3: Automation You Can Audit

### Frame 4.1 - The Problem

This registration lets Mizar reuse a proved fact (exact MML excerpt):

```mizar
registration
  let M be addMagma;
  cluster right_add-cancelable left_add-cancelable -> add-cancelable for
Element
    of M;
  coherence;
end;
```

- **Registrations let attributes propagate automatically, so proofs stay short.**
- **I want to keep this idea and make the steps of the automation easier to see.**

Speaker note:

- Source: current MML `algstr_0.miz`, lines 104-109.

### Frame 4.2 - Why This Is Difficult [deep dive]

Bullets:

- When a proof fails, "why does the checker not see that this is a Group?"
  has no local answer. The cause is somewhere in the environment.
- Users cannot see which registrations fired or in which order.
  The automation is powerful, but explaining it becomes harder as it does more.
- For an AI assistant the situation is worse: it must guess the cluster
  state instead of reading it.

Message:

- Automation without explanations makes a large library harder to
  maintain, even when it is sound.

### Frame 4.3 - The Evo Answer: Labeled, Traceable Registrations

**Evo gives every registration item a label** (specification example):

```mizar
registration
  cluster EmptyImpliesFinite: empty -> finite for set;
  coherence proof ... end;

  cluster FiniteImpliesCountable: finite -> countable for set;
  coherence proof ... end;
end;
```

- **We can cite the label in `by`. It also appears in diagnostics and the module interface.**
- The verifier uses an import-filtered view of the global cluster graph.
- **`explain-attribute` and resolution traces explain success or failure.
  For example, they can show the steps from empty to finite to countable.**

### Frame 4.4 - The Evo Answer: Oriented Reductions [deep dive]

Mizar Evo (specification example):

```mizar
registration
  let n be Nat;
  reduce NatAddZero: n + 0 to n;
  reducibility
  proof
    let n be Nat;
    thus n + 0 = n by mml.number.natural.Nat_add_zero;
  end;
end;
```

Bullets:

- a reduction is an oriented simplification backed by an equality proof;
- the right side must be strictly smaller, so imported rules cannot loop,
  and rule selection is deterministic (pattern subsumption, then guard
  specificity, then FQN tie-break);
- unoriented identification idioms become auditable `reduce` items.

### Frame 4.5 - What Is Preserved, What We Ask

- Registrations and clusters remain part of the language, and proofs stay short.
- Applying a cluster does not require us to repeat its proof.
- A reduction simplifies a term using an equality. It may need local guard
  evidence or an explicit equality citation.
- **I would like to learn which explanations would help most in daily MML work.
  This leads to the next story: stronger proof search.**

Questions for later review:

- Which cluster explanations would help most with daily work: failure
  explanations, firing traces, or difference reports between environments?
- Which MML article families make the most complex use of registrations and should become
  migration benchmarks for the cluster graph?

## Part 5. Story 4: Powerful Search, Small Trust

### Frame 5.1 - The Problem

- Users want stronger automation, including larger `by` steps and hammer-style search.
- **MizAR and MPTP already use ATP search with MML premises.**
- **Evo's goal is to keep proof checking small as search becomes stronger.**
- **A search result alone is not a kernel-verified proof. We need checkable evidence.**
- So the question is this.

Slide text:

```text
How do we get modern proof search
without trusting the searcher?
```

Speaker note:

- This builds on MizAR, MPTP, and hammer research.
  I am describing Evo's evidence contract, not making a claim that earlier
  tools must trust their search backends.

### Frame 5.2 - The Evo Answer: A Reasoning Boundary

![The reasoning boundary: semantics, untrusted search, trusted checking](figures/reasoning_boundary.pdf)

- **We can read this diagram from left to right.**
- **Mizar-side phases resolve names, infer types, expand clusters, and pick overloads.
  ATPs do none of these tasks.**
- **The kernel checks the supplied formulas and substitutions, then runs its trusted SAT check.
  It does not select premises or invent substitutions.**
- **Earlier deterministic discharge also needs replayable evidence.**

Speaker note:

- Development policy may record `externally_attested` results.
  These are separate from kernel-verified proofs.

### Frame 5.3 - Formula And Substitution Evidence

![KernelEvidence and the kernel's SAT check](figures/certificate_replay.pdf)

- **Here we can see what the evidence contains: source formulas, substitutions,
  provenance, and target and goal bindings.**
- **The kernel checks it, derives instantiated formulas and SAT clauses,
  and requires UNSAT from its trusted in-process Rust SAT checker.**
- **Backend resolution traces, SMT proof objects, logs, and exit codes
  are not trusted acceptance evidence.**

Speaker note:

- Hashes bind evidence to its dependency context. The kernel also checks
  binder conditions and the goal's refutation polarity.

### Frame 5.4 - The Same Boundary Controls AI

An AI tool could help repair a citation like this one (exact MML excerpt):

```mizar
theorem
  for F being non degenerated ZeroOneStr holds 1.F in NonZero F
proof
  let F be non degenerated ZeroOneStr;
  not 1.F in {0.F} by TARSKI:def 1;
  hence thesis by XBOOLE_0:def 5;
end;
```

- **The AI tool can propose a missing or more precise `by` reference.**
- This is a local edit that keeps the statement's meaning.
- **The verifier checks it just as it would check a human edit.**
- The AI proposes the change. The verifier and kernel decide whether to accept it.

Speaker note:

- The assistant's strength never enters the trusted base.
- Source: current MML `struct_0.miz`, lines 637-643.

### Frame 5.5 - Edit Classes: Green, Yellow, Red [deep dive]

| Class | Examples | Policy |
|---|---|---|
| Green | add citation, insert `qua`, info annotation | can be proposed automatically; still verified |
| Yellow | add import, local lemma, registration | proposed with human review |
| Red | weaken theorem, change definition, add axiom | forbidden to ordinary agents |

Forbidden repair (sketch):

```mizar
theorem
  for x be Nat holds x + 0 = x or x = 0;
```

Message:

- Weakening a statement to make its proof easier is a Red edit; at most an
  agent may flag it for explicit human unsafe-edit review.

### Frame 5.6 - What Is Preserved, What We Ask

- This keeps the de Bruijn discipline: a small trusted core checks acceptance.
  In this design, the core includes its SAT checker.
- Proof text stays declarative and readable. Automation helps maintain the argument.
- I would welcome your views on the evidence format.
- **Now let us look at the cost of checking a large library.**

Questions for later review:

- Is the formula/substitution evidence approach convincing for Mizar-style
  obligations, including clusters and definitional expansions?
- Which evidence format would the team be most willing to audit?

## Part 6. Story 5: Verification That Scales

### Frame 6.1 - The Problem

- **Checking the whole MML takes hours.**
- **The accepted article is the unit of reuse. A small edit can therefore
  cause more checking than we would like.**
- Memory use follows the article environment, including material outside
  the interface actually used.
- These costs follow from the article as a unit. Evo proposes smaller units of reuse.

### Frame 6.2 - The Evo Answer: Fingerprints And Incrementality

- **Evo uses fingerprints to decide when earlier results can be reused.**
- **All relevant cache keys must match. Missing data causes a cache miss.**
- **A proof-body edit does not rebuild importers when the exported statement
  and accepted status stay unchanged.**
- Independent modules, obligations, ATP runs, and kernel checks can run in parallel.
  Results are published in canonical order.

![The fingerprint graph: what a change re-verifies](figures/fingerprint_graph.pdf)

```text
**Cache reuse is never proof authority.**
**A clean build must reproduce every acceptance.**
```

Speaker note:

- The cache keys cover source, dependency slices, package and lockfile,
  toolchain, schema, policy, registrations, obligations, evidence, and witnesses.

### Frame 6.3 - The Evo Answer: A Memory Contract [deep dive]

```text
resident memory should scale with:
  active source and typed AST
  imported public interfaces
  import-filtered indexes
  active module obligations
  in-flight proof checks and ATP runs
  small caches

not with:
  imported proof bodies
  imported proof witnesses
  private lemmas outside the interface
  registration data outside the import closure
```

Message:

- This is a resident-memory model, not a measured performance guarantee.
  Proof bodies and traces load lazily for specific queries.

### Frame 6.4 - What Is Preserved, What We Ask

- Caching and parallel work change checking time. They must not change truth.
- A person still reviews complete source text, in the style of an article.
- **We need tests that compare incremental results with a clean build.**
- **I would like to use real MML maintenance tasks for these tests.
  Next, I will turn to templates.**

Questions for later review:

- What clean-build equivalence tests would make incremental verification
  trustworthy to the team that maintains MML today?
- Which current maintenance operations (revisions, renamings) should we
  benchmark for incremental cost?

## Part 7. Story 6: Templates For Generic Mathematics

### Frame 7.1 - The Problem, Part One: Schemes Use Separate Rules

**This familiar scheme expresses induction** (exact MML excerpt):

```mizar
scheme
  NatInd { P[Nat] } : for k being Nat holds P[k]
provided
A1: P[0] and
A2: for k be Nat st P[k] holds P[k + 1]
```

- Schemes support second-order patterns such as induction, separation, and replacement.
- **Classic `scheme` blocks define theorem schemas. Parameterized definitions
  use other language forms.**
- **Evo proposes a common template system for these tasks.**

Speaker note:

- Source: current MML `nat_1.miz`, near line 90 (checked July 2, 2026).

### Frame 7.2 - The Problem, Part Two: A Common Template System

- **Mizar already has parameterized constructions, such as `Polynom-Ring L`.**
- The aim is to give definitions and theorem schemas a common template system.
- Each kind of parameter still has explicit rules.
- We need migration examples to see whether these rules make generic
  mathematics easier to write and maintain.

Source: [POLYNOM3, definition 10](https://mizar.uwb.edu.pl/version/current/html/polynom3.html).

### Frame 7.3 - The Evo Answer: Templates

**This template begins with a type parameter** (specification example):

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

- **A template is an ordinary `definition` block. Its leading `let` binds the parameters.**
- **Parameters can be types, values, predicates, or functors.**
- **Predicate and functor parameters cannot occur in `attr`, `mode`, `struct`, `func`, or `pred` items.**
- Some templates allow short forms such as `Module over R` and `Subset of X`.
  Constrained templates require brackets.

Speaker note:

- The short forms display `Module[R]` and `Subset[X]`.

### Frame 7.4 - Bounded Parameters And Generic Theorems [deep dive]

Mizar Evo (specification example, §18.2.2; proof omitted):

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

Message:

- The bound requires commutativity, associativity, and a unit.
- `Product[T]` starts with the unit and folds the sequence with the
  selected operation. The library proves these defining equations.

### Frame 7.5 - One Proof, Many Instantiations [deep dive]

Instantiation (specification example, with the required registrations):

```mizar
PermProduct[commutative associative unital AddMagma]
PermProduct[commutative associative unital MulMagma]

let R be commutative Ring;
PermProduct[R qua AddMagma]        :: R's additive view
PermProduct[R qua MulMagma]        :: R's multiplicative view
```

Message:

- A ring reaches Magma along two paths. `qua` selects the view.
  The view also sets the notation: the generic `*` appears as `+`
  for addition.
- The required attributes must hold on the selected view.

### Frame 7.6 - Schemes Become Ordinary Templates [deep dive]

Mizar Evo (specification example):

```mizar
definition
  let P be pred(Nat);
  theorem NatInduction[P]:
    P(0) & (for n being Nat st P(n) holds P(n+1))
    implies for n being Nat holds P(n)
  proof ... end;
end;
```

Bullets:

- predicate parameters follow the familiar `defpred` convention;
- legacy schemes have a direct, mechanical migration target;
- theorem instantiation uses explicit brackets, so tools and artifacts see
  exactly which instance a proof uses.

### Frame 7.7 - What Is Preserved, What We Ask

- Scheme-style reasoning keeps the same power.
- The words `of` and `over` help keep mathematical text readable.
- **Each instantiation is checked. Templates add no new logic.**
- **I would like to try this design on existing MML schemes.
  The next story connects proofs with algorithms.**

Questions for later review:

- Which MML schemes should be the first migration targets?
- Are brackets acceptable as the canonical identity form, with `of`/`over`
  as display forms?
- For `func` and `pred` templates, normalized declared argument types must
  determine each inferred type parameter uniquely. `qua` views are never
  inferred. Does this rule require too many explicit `[T]` arguments?

## Part 8. Story 7: Verified Computation With Algorithms

### Frame 8.1 - The Problem

- **Mizar already provides arithmetic automation and proofs of program correctness.**
- **Evo proposes a language form for algorithms with contracts.**
- The goal is to connect verification, execution, and code extraction
  within one language and toolchain.
- **Execution and code extraction are still planned work.**

Slide text:

```text
Connect mathematical proofs with executable algorithms.
```

Examples in MML: `NUMERALS` for arithmetic requirements;
`FIB_FUSC` for program correctness (see the
[Formalized Mathematics contents](https://mizar.uwb.edu.pl/fm/contents.html)).

### Frame 8.2 - The Evo Answer: Algorithms With Contracts

**This is Euclid's algorithm with a contract** (specification example):

```mizar
definition
  let a, b be Nat;
  terminating algorithm euclid_gcd(a, b) -> Nat
    requires a >= 1 & b >= 1
    ensures result = Gcd(a, b)
  do
    var x := a;  var y := b;
    while y <> 0 do
      invariant x >= 1 & y >= 0 & Gcd(a, b) = Gcd(x, y);
      decreasing y;
      const r := x mod y;  x := y;  y := r;
    end;
    return x;
  end;
end;
```

The contract and invariant use mathematical `Gcd`, so there is no circular definition.
**The loop computes the result. The `decreasing` measure on `y` proves termination.**

Speaker note:

- This example is condensed from specification section 20.12.

### Frame 8.3 - The Evo Answer: Proof By Computation

**The specification allows proofs by computation** (specification example):

```mizar
theorem EuclidGcd12_8:  euclid_gcd(12, 8)  = 4  by computation;
theorem EuclidGcd100_75: euclid_gcd(100, 75) = 25 by computation;

theorem Fact10: factorial(10) = 3628800
proof
  thus thesis by computation(steps: 100000);
end;
```

- **The Mizar Virtual Machine, or MVM, evaluates ground equalities and predicates.
  It uses computable algorithms defined in the current package.
  Non-ground formulas need classical proofs.**
- Step, time, and depth limits are optional. Each defaults to zero, meaning unlimited.
- **Other packages' algorithm bodies cannot run here. Their `ensures` contracts
  can be used when termination is known.**

### Frame 8.4 - The Evo Answer: Termination Allows Recursion [deep dive]

Mizar Evo (specification example):

```mizar
definition
  let n be Nat;
  terminating algorithm factorial(n) -> Nat
    decreasing n
  do
    if n = 0 do return 1; end;
    return n * factorial(n - 1);
  end;
end;
```

Bullets:

- ordinary `func` definitions are definitional extensions: never recursive;
- a `terminating` algorithm becomes a functor after all contract and
  termination obligations are proved for inputs satisfying `requires`;
- this is the only way to add recursion to the mathematical layer.
  It requires a proof.

Speaker note:

- Each call must satisfy `requires`. An ordinary algorithm may prove only
  partial correctness and is not promoted to a functor.

### Frame 8.5 - Computation Never Redefines Truth [deep dive]

Bullets:

- contracts, invariants, and termination measures generate verification
  conditions that pass through the same ATP-plus-kernel boundary as
  ordinary theorems (story 4);
- `by computation` uses MVM replay with optional limits;
- code extraction (to runtime targets) is strictly downstream of verified
  artifacts and can never affect acceptance.

Message:

- Algorithms let the library express more. They do not change what it
  means for a theorem to be accepted.

### Frame 8.6 - What Is Preserved, What We Ask

- Algorithms stay inside `definition` blocks.
- **Verified promotion lets a `terminating` algorithm become a functor
  after its obligations are proved.**
- Proofs can also use `by computation` for computable ground calls in the current package.
- **A partial algorithm's `ensures` requires evidence that the call terminates.**
- The first-order set-theoretic foundations stay the same.
- **Now I will turn to how we publish and cite this work.**

Questions for later review:

- Which computational examples would show clear benefits to mathematicians
  without shifting the culture toward programming?
- Are there MML areas (number theory, combinatorics, finite structures)
  where `by computation` would immediately shorten real proofs?
- Which extraction targets matter first, if any?

## Part 9. Story 8: A Library You Can Cite

### Frame 9.1 - The Problem

- Formalized Mathematics links a research journal to a formal library.
  People can read and cite its articles.
- **A journal article has an order that helps explain the mathematics.
  A reusable library needs an order based on dependencies.**
- **Changes to library organization can therefore affect links to published articles.
  Package reuse also lacks an identity for journal citations.**
- **Evo proposes separate, linked records for journal articles and library items.**

### Frame 9.2 - The Evo Proposal: Linked Records

![A proposed article-to-library link model (sketch)](figures/fm_links.pdf)

- **This sketch links citations, library items, verification artifacts, and MML origins.**
- **It shows links between records, not a derivation.**

### Frame 9.3 - Who Gains What [deep dive]

| Audience | Gain |
|---|---|
| readers | written explanations come first; formal source is one click away |
| maintainers | refactoring no longer rewrites published explanations |
| authors | frozen `pub` article identities; FQNs for library locations |
| planned AI tools | prose for retrieval, fingerprints for exact context |

### Frame 9.4 - What Is Preserved, What We Ask

- Formalized Mathematics would remain a journal with review and written explanations.
- **Origin metadata would link migrated items to their MML citations.
  The migration mapping still needs to be defined.**
- I would like to discuss which identifiers readers should see in citations.
- **That completes the eight stories. Let us now look at the whole design.**

Questions for later review:

- Which identity should be primary in user-facing citations: article label,
  library FQN, or origin id?
- How should existing Formalized Mathematics articles link to migrated
  modules - by updating old links now, when articles are revised, or not at all?

## Part 10. Architecture In One Picture

### Frame 10.1 - The Core ATP Path

![The core ATP path, with responsibility groups](figures/pipeline.pdf)

- **This diagram shows the ATP path through the proposed pipeline.**
- **Only obligations still open after deterministic discharge go to ATP.
  Earlier discharge also needs evidence.**
- Each boundary states who owns a fact, which artifact records it,
  and what must be checked again after a change.

### Frame 10.2 - Responsibility Split [deep dive]

| Layer | Responsibility |
|---|---|
| frontend | lexing, parsing, recovery |
| resolver | imports, names, labels, namespaces |
| checker | soft types, clusters, registrations, overloads |
| elaborator | core logical representation |
| VC generator | proof and algorithm obligations |
| ATP layer plus kernel | untrusted search, then acceptance by checking |
| artifact emitter | stable outputs for tools and dependents |

### Frame 10.3 - Pipeline Stages For The Eight Stories

- These eight stories belong to one pipeline, from name resolution to publication.
- Each stage keeps its own responsibility for meaning and proof checking.

Details for later review:

| Story | Pipeline stage |
|---|---|
| dependencies | resolver, package manager |
| structures and automation | checker (inheritance and cluster graphs, traces) |
| search vs trust | ATP layer, kernel, formula/substitution evidence |
| scale | artifacts, fingerprints, scheduler |
| templates | elaborator (checked instantiation) |
| algorithms | VC generator, MVM |
| publication | artifact emitter, doc generation |

### Frame 10.4 - Testing The Trust Boundary [deep dive]

Slide text:

```text
Reject what must not pass
before
Accept everything that should pass
```

Bullets:

- soundness bugs have higher priority than parser gaps. Tests near the
  kernel first focus on malformed and failing evidence;
- with these tests in place, we add coverage for accepted language forms.

## Part 11. Roadmap And Collaboration

### Frame 11.0 - Where The Project Stands Today

- **Before I finish, let me separate the design from the implementation.**
- The specification has 24 chapters and appendices. English is canonical;
  Japanese companions are available.
- **Implemented parts include frontend processing, selected semantic cases,
  ATP candidate generation, and kernel evidence checking.**
- **The full path from source through ATP and kernel checking to published
  artifacts is still in progress. Tests with real external provers are still needed.**
- **MVM execution, code extraction, and full MML migration remain future work.**

Speaker note:

- This is the September 2026 scope. Component tests do not show that the
  full pipeline works. See `doc/design/todo.md`, Completion Gates.

### Frame 11.1 - Migration Needs Research

![Roadmap timeline](figures/roadmap_timeline.pdf)

- **We plan an alpha at the end of 2026. It covers a core frontend subset,
  import and module resolution prototypes, diagnostics, and early artifacts.**
- **In 2027, we plan to translate three to five representative MML articles
  by hand and by script. We will record every mismatch.**
- In 2027 and 2028, we plan to expand from set and relation fragments
  to algebraic structures and their dependencies.
- **The alpha does not aim at full MML verification, a final compatibility
  layer, or a stable AI protocol.**

### Frame 11.2 - What We Will Measure [deep dive]

Bullets:

- translated articles and lines; accepted parser subset;
- resolved imports versus unresolved dependencies;
- obligations closed deterministically versus by ATP with kernel evidence;
- memory and wall-clock per module, incremental versus clean;
- compatibility decisions that required human judgment.

### Frame 11.3 - Compatibility Policy And Risks [deep dive]

Policy:

- define origin mappings to preserve MML theorem identity;
- keep compatibility aliases where they help migration;
- record every difference from old behavior with a reason and a test.

| Risk | How to reduce it |
|---|---|
| compatibility work takes all our time | small representative parts first; no all-at-once translation |
| registrations behave differently | trace artifacts and comparison reports early |
| package layout breaks journal links | origin metadata, article-to-library identifiers |
| AI edits hide migration mistakes | Red edits stay forbidden to ordinary agents; verifier artifacts required |

### Frame 11.4 - What September 2026 Should Produce

- For this visit, I hope we can choose migration benchmark articles in priority order.
- I would like to agree on the compatibility metadata we need.
- I also hope to collect comments on the eight stories, especially structures and clusters.
- A first paper outline and a shared list of objections would help guide the next steps.

### Frame 11.5 - What We Ask Of You

- I do not expect us to settle every design question today.
- The handout keeps the questions below for later review.
- Your choice of small MML examples would help us test the proposal in practice.

Questions for later review:

- Which MML articles are small but structurally representative?
- Which idioms matter to the Mizar community, beyond their technical use?
- Which of the eight stories is most wrong, and why?
- What migration result would convince the community that Evo is a serious project?

## Part 12. Closing

### Frame 12.1 - The Main Question, Again

- **I would like to finish with the question from the beginning.**

Slide question:

```text
**What must Mizar Evo preserve so that the Mizar community**
**still recognizes it as Mizar?**
```

- **Which small MML articles should we migrate first?**
- **Which proposal needs the most change?**

### Frame 12.2 - Closing

- **Mizar Evo should update the parts that need to support a larger library.**
- **It should keep the parts that define Mizar's mathematical identity.**
- **Thank you for listening. I would welcome your questions and comments.**

## Backup A. Prepared Exact Examples

Exact excerpts used in this talk:

| Purpose | Source | Lines | Used in |
|---|---|---:|---|
| Structure definition | `algstr_0.miz` | 37-40 | Frames 0.2, 3.1 |
| Article environment | `algstr_0.miz` | 15-25 | Frame 2.1 |
| Registration/cluster | `algstr_0.miz` | 104-109 | Frame 4.1 |
| Proof citation | `struct_0.miz` | 637-643 | Frame 5.4 |
| Induction scheme | `nat_1.miz` | near 90 | Frame 7.1 |

Source URLs:

- `ALGSTR_0`: <https://mizar.uwb.edu.pl/version/current/mml/algstr_0.miz>
- `STRUCT_0`: <https://mizar.uwb.edu.pl/version/current/mml/struct_0.miz>
- `NAT_1`: <https://mizar.uwb.edu.pl/version/current/mml/nat_1.miz>

Attribution note:

- MML plain-text files state GPL-3.0-or-later / CC-BY-SA-3.0-or-later terms;
  keep article attribution, URLs, and line numbers in speaker notes.

## Backup B. Specification Reference Map

Specification used: 10 September 2026. For grammar and semantics, see
`doc/spec/en/00.index.md` in the [repository](https://github.com/aabaa/mizar-evo).
Online files may change after this talk.

| Topic | Specification source (under `doc/spec/en/`) |
|---|---|
| Modules and imports | `12.modules_and_namespaces.md` |
| Structures and inheritance | `05.structures.md` |
| Attributes and modes | `06.attributes.md`, `07.modes.md` |
| Registrations and reductions | `17.clusters_and_registrations.md` |
| Templates and schemes | `18.templates.md` |
| Algorithms and MVM | `20.algorithm_and_verification.md` |
| ATP and kernel evidence | `21.source_code_annotation_and_atp.md` |
| Packages and artifacts | `23.package_management_and_build_system.md` |
| Cross-chapter grammar | `appendix_a.grammar_summary.md` |
| Worked library sketches | `sample_codes.md` |

## Backup C. Diagram List

Required diagrams for the final deck (`figures/*.tex`, TikZ standalone;
build each with `pdflatex` inside `figures/`):

1. Three pressures on a proven design (Part 1).
   [done: `figures/three_pressures.pdf`, used in frame 1.5]
2. Environment-to-import migration (story 1).
   [done: `figures/environ_migration.pdf`, used in frame 2.5]
3. Structure inheritance and diamond coherence (story 2).
   [done: `figures/diamond_inheritance.pdf`, used in frame 3.5]
4. Reasoning boundary: semantics / ATP search / kernel checking (story 4).
   [done: `figures/reasoning_boundary.pdf`, used in frame 5.2]
5. KernelEvidence and the trusted SAT check (story 4).
   [done: `figures/certificate_replay.pdf`, used in frame 5.3]
6. Incremental fingerprint graph (story 5).
   [done: `figures/fingerprint_graph.pdf`, used in frame 6.2]
7. Formalized Mathematics article-to-library link model (story 8).
   [done: `figures/fm_links.pdf`, used in frame 9.2]
8. Core ATP path with responsibility groups (Part 10).
   [done: `figures/pipeline.pdf`, used in frame 10.1]
9. Roadmap timeline (Part 11).
   [done: `figures/roadmap_timeline.pdf`, used in frame 11.1]

## Backup D. Possible Paper Outline

Possible paper title:

```text
Mizar Evo: Readable, AI-Ready, and Scalable Formal Mathematics
```

Possible sections:

1. Introduction: why Mizar needs evolution now.
2. Mizar as baseline: readability, MML, Formalized Mathematics.
3. Design principles and the three goals.
4. Language evolution: dependencies, structures, registrations, templates.
5. Verifier architecture, kernel evidence, and the trusted SAT checker.
6. Verified computation and the MVM.
7. AI-safe proof development.
8. Package-based library and publication workflow.
9. Migration plan and evaluation measures.
10. Related work and plans for working together.

## Backup E. Reviewer Checklist

Use this checklist before converting to Beamer:

- Does every story open with a real cost, not a feature announcement?
- Does every criticism explain why the current practice was useful?
- Does every code example carry its status label (exact MML excerpt,
  specification example, or sketch)?
- Is every exact excerpt attributed with article and line numbers?
- Does every story end with questions the audience can actually answer?
- Are Red AI edits clearly forbidden?
- Are migration claims measurable?
- Is EBNF absent from all frames?

## Backup F. Possible Objections

Prepared answers to other possible objections:

| Objection | Prepared answer |
|---|---|
| Why not improve current Mizar incrementally? | Some proposals may also help existing tools. Evo explores changes to modules, proof evidence, and library artifacts together. Migration examples should help us decide which changes are useful. |
| What happens to authorship and credit of MML articles? | We propose origin mappings that preserve identity and credit during migration. The mapping format is still open. The `pub` namespace keeps published articles unchanged. |
| Is this a fork of the community? | It is a proposal to the community; this visit is its first review. The namespace governance model assumes that the Mizar team controls the `mml` root. |
| What about GPL / CC-BY-SA obligations? | Migration preserves license and attribution of MML content; toolchain licensing is open for discussion. |
| Are the claims about AI too strong? | AI assistance is an optional layer that never enters the trusted base; every proposal also works without it. |
| Why a new kernel instead of the existing checker? | Formula/substitution evidence needs a small trusted core, including a SAT checker. Mizar Evo's specification defines the obligations; legacy behavior guides migration comparisons. |

Speaker note:

- Use these answers only if someone asks these questions.
