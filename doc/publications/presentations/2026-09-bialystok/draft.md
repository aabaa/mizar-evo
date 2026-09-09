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

Specification examples re-checked against `doc/spec/en/` on July 10, 2026:
the standard-library namespace root is now `mml` throughout (Chapter 12),
reduction rule selection is pattern subsumption, then guard specificity, then
FQN tie-break (Chapter 17, §17.6.4), and template parameter inference is
defined as unique-declared-type only, with `qua` never inferred (Chapter 18,
§18.2.7).

## Deck Shape

Thirteen sections: opening, motivation, eight stories, architecture overview,
roadmap, closing. Roughly 60-70 frames; the seminar is informal, so we allow more
time for detailed discussion. Every story ends with questions that
only the Bialystok team can answer.

Two-tier pacing: frames marked `[deep dive]` in their headings can be skipped
without losing the main points. The unmarked core path is roughly 48 frames
(about 60-75 minutes plus discussion); we use deep-dive frames when
the audience wants more detail. The generated deck shows a small "deep dive"
tag on those frames.

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

- Thank the team.
- Explain that this is a design review. The team knows best whether the
  project still looks and reads like Mizar.
- The design is still open to change. We want to hear objections.

### Frame 0.2 - A First Look

Legacy Mizar (exact MML excerpt):

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

Mizar Evo (specification example):

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

- Source: current MML `algstr_0.miz`, lines 37-40.
- This slide shows the main idea. It still reads as Mizar, and the
  mathematics is unchanged. The parent link and field mapping were implicit.
  Now they are visible in source text that the verifier can check.

### Frame 0.3 - The Proposal In One Sentence

Slide text:

```text
Preserve Mizar's mathematical vernacular.
Update the compiler, verifier, artifact, and publication layers.
```

What this talk does not claim:

- Full MML migration is not complete.
- The language standard is not final.
- AI assistance is not a substitute for proof checking.
- We build on what the current Mizar system has achieved.

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

### Frame 0.5 - What We Need From This Visit

Bullets:

- find compatibility constraints that are hard to see outside MML work;
- choose migration benchmark articles that are small but representative;
- review the trust boundary for ATP search and kernel checking;
- discuss how Formalized Mathematics should link to a package library;
- collect the objections we have not thought of.

Main question:

```text
What must Mizar Evo preserve so that the Mizar community
still recognizes it as Mizar?
```

## Part 1. Why Now

### Frame 1.1 - What Mizar Got Right

Bullets:

- declarative proof text that reads as mathematics;
- soft types, modes, and adjective-rich vocabulary;
- attributes, registrations, and clusters as reusable automation;
- a well-established, carefully maintained library (MML 5.94.1493: 1493 articles);
- a tradition of publishing formal articles (Formalized Mathematics).

Message:

- We build on these strengths. We judge every proposal by whether it
  protects them.

Speaker note:

- The audience knows its own system. Spend one minute on the strengths
  we agree on.

### Frame 1.2 - Pressure One: Scale

Bullets:

- MML has grown to roughly 1500 articles that depend on each other.
- The unit of dependency, review, and reuse is the whole article.
- Tools resolve article environments. But reviewers cannot see the resolved
  dependencies in the source.
- Whole-library maintenance operations (renames, refactorings, revisions)
  become more risky as the library grows.

Message:

- The problem is not that current Mizar is wrong. It is that article-level
  boundaries were designed for a smaller library.

### Frame 1.3 - Pressure Two: Tooling Expectations

Bullets:

- Editors are expected to give immediate feedback, even on incomplete or broken source.
- Builds are expected to be reproducible from a manifest and lockfile.
- Reuse is expected to work through packages with versions.
- Documentation is expected to be generated, linked, and browsable.

Message:

- Common programming tools already provide these features. New users
  expect formal libraries to provide them too.

### Frame 1.4 - Pressure Three: AI

Bullets:

- AI agents are already useful for search, explanation, and repair.
- They need a limited amount of structured context linked to the source,
  rather than a copy of the whole library.
- Their output must never decide whether a proof is valid. Verification
  must stay independent of how capable the assistant is.
- Readable source helps here. AI editing and retrieval work best with
  stable local text patterns.

Message:

- Mizar's readability makes safe AI assistance possible today.

### Frame 1.5 - Three Pressures, One Design

![Three pressures on a proven design](figures/three_pressures.pdf)

Message:

- These three needs do not mean that Mizar's design was wrong.
  Together, they give us reasons to change its boundaries.

### Frame 1.6 - The Design Rule

Slide text:

```text
Do not make proofs harder to read to add automation.
Use automation to protect and extend readability.
```

Three goals, one test:

| Goal | Test for every feature |
|---|---|
| Readability | does proof text still read as mathematics? |
| AI-readiness | can a tool see a limited context that we can audit? |
| Scalability | do boundaries stay stable as the library grows? |

Speaker note:

- The eight stories that follow each apply this rule to one specific problem.

## Part 2. Story 1: Dependencies You Can See

### Frame 2.1 - The Problem

Legacy Mizar (exact MML excerpt):

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

Speaker note:

- Source: current MML `algstr_0.miz`, lines 15-25.
- Everyone here has edited these blocks, trying changes until they work.
- `environ` has supported library growth for decades. We need to consider
  the cost of using it with today's larger library.

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

Mizar Evo (specification example):

```mizar
import .function;
import mml.algebra.structure.sorted;

definition
  let S be 1-sorted;
  mode BinOpDef: BinOp of S is
    Function of [: S.carrier, S.carrier :], S.carrier;
end;
```

Rules that make this deterministic:

- all imports appear before the first item; no mid-file environment changes;
- imports provide the initial active lexicon. Every symbol, notation,
  registration, and theorem is traceable to exactly one import;
- stable fully-qualified names are derived from package and module paths.

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

- a manifest plus a lockfile makes every build reproducible;
- versioned reuse (SemVer) replaces manual copying between article sets.

### Frame 2.5 - Migrating The Environment [deep dive]

![Environment-to-import migration map](figures/environ_migration.pdf)

Message:

- migration needs more than renaming. Current environments mix roles for
  semantics, syntax, and automation. Migration reports must explain what
  each imported module provides.

### Frame 2.6 - What Is Preserved, What We Ask

Preserved:

- the mathematics and theorem identities are untouched;
- article-style authorship remains; a module is still a readable text;
- migration keeps origin metadata (story 8 returns to this).

Questions for Bialystok:

- Which `environ` roles must remain visually familiar during migration?
- Which current article dependencies are hardest to explain, and would make
  the best test cases for generated dependency reports?

## Part 3. Story 2: Structures Without Hidden Merges

### Frame 3.1 - The Problem

Legacy Mizar (exact MML excerpt):

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

Bullets:

- parent link, fields, and selector layout are one compact declaration;
- with multiple parents, the merge of inherited fields is implicit;
- renamed views (additive vs multiplicative) depend on naming conventions;
- stored data and canonical values (a zero, a unit) are not distinguished.

Speaker note:

- Source: current MML `algstr_0.miz`, lines 37-40.
- Explain why this short form was useful: structures stayed close to
  informal mathematical writing.

### Frame 3.2 - Why This Is Difficult [deep dive]

Bullets:

- Diamond inheritance (one structure reachable through two parent paths) is
  resolved by convention and declaration order, not by checkable source.
- The syntax does not tell a migration tool whether a selector is intrinsic
  data, a canonical value with obligations, or an inherited view.
- Errors appear far from their cause, as type mismatches in later articles.

Message:

- At MML scale, structure inheritance is a graph maintenance problem, and the
  graph needs explicit edges that the verifier can check.

### Frame 3.3 - The Evo Answer: Field, Property, Attribute

Mizar Evo (specification example):

```mizar
definition
  struct AddLoopStr where
    field carrier -> set;
    field add -> BinOp of carrier;
    property zero -> Element of carrier;
  end;
end;
```

Three distinct concepts:

| Concept | Meaning | Consequence |
|---|---|---|
| `field` | intrinsic data supplied by the value | constructor shape, equality |
| `property` | uniquely determined canonical value | existence/uniqueness obligations |
| `attribute` | predicate-style refinement | cluster propagation, not layout |

### Frame 3.4 - The Evo Answer: Explicit Inheritance

Mizar Evo (specification example):

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

Bullets:

- one `inherit` statement per parent; mapping and renaming are source text;
- narrowing and type conversions carry `coherence` proof obligations;
- the additive/multiplicative naming convention becomes a checked view.

### Frame 3.5 - The Evo Answer: Diamonds Become Checkable

Mizar Evo (specification example):

```mizar
struct DoubleLoopStr where
  field carrier -> set;
  field add -> BinOp of carrier;
  field mul -> BinOp of carrier;
  property zero -> Element of carrier;
  property one -> Element of carrier;
end;

inherit DoubleLoopStr extends AddLoopStr;
inherit DoubleLoopStr extends MulLoopStr;
```

![The diamond, with checkable joins](figures/diamond_inheritance.pdf)

Message:

- the verifier traces `from` chains to root declarations; both paths must
  introduce the same components (the shared `carrier` above);
- diamond inheritance becomes a diagnostic with source spans, not a silent
  merge decided by declaration order.

### Frame 3.6 - What Is Preserved, What We Ask

Preserved:

- structures remain Mizar structures: carriers, selectors, `Element of`;
- aggregates become longer only where we need to make a hidden decision explicit.

Questions for Bialystok:

- Is the `field` / `property` / `attribute` split readable in real algebraic
  articles, or does it require too many annotations in simple cases?
- Is one-parent-per-`inherit` acceptable for parts of MML with much
  inheritance, such as the `ALGSTR` and topology hierarchies?
- Which MML structures would be the best diamond test cases?

## Part 4. Story 3: Automation You Can Audit

### Frame 4.1 - The Problem

Legacy Mizar (exact MML excerpt):

```mizar
registration
  let M be addMagma;
  cluster right_add-cancelable left_add-cancelable -> add-cancelable for
Element
    of M;
  coherence;
end;
```

Speaker note:

- Source: current MML `algstr_0.miz`, lines 104-109.
- Registrations are one of Mizar's best ideas. Adjectives propagate
  automatically, so proofs stay short. But it is hard to see how this works.

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

Mizar Evo (specification example):

```mizar
registration
  cluster EmptyImpliesFinite: empty -> finite for set;
  coherence proof ... end;

  cluster FiniteImpliesCountable: finite -> countable for set;
  coherence proof ... end;
end;
```

Bullets:

- every registration item carries a required label: citable in `by`,
  reported in diagnostics, part of the module interface;
- the verifier stores an import-filtered cluster resolution graph;
- `@show_resolution` and explanation artifacts answer "why (not)?" with the
  actual chain, e.g. empty -> finite -> countable.

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

Preserved:

- registrations and clusters remain first-class; proofs stay short;
- no new proof text is required at use sites - only traces are added.

Questions for Bialystok:

- Which cluster explanations would help most with daily work: failure
  explanations, firing traces, or difference reports between environments?
- Which MML article families make the most complex use of registrations and should become
  migration benchmarks for the cluster graph?

## Part 5. Story 4: Powerful Search, Small Trust

### Frame 5.1 - The Problem

Bullets:

- Users want stronger automation: bigger `by` steps, hammer-style search.
- But in a monolithic verifier, every gain in search power grows the code
  that must be trusted.
- External provers (ATPs) are strong exactly where Mizar's core is
  first-order - and they are the least auditable component of all.
- MizAR and MPTP research already showed that ATP search is powerful
  on MML premises; the open question is trust, not power.

Slide text:

```text
How do we get modern proof search
without trusting the searcher?
```

Speaker note:

- Mention MizAR, MPTP, and hammer research by name. They showed how well
  search works on MML. Mizar Evo adds a boundary that allows us to use
  that search without making the trusted base larger.

### Frame 5.2 - The Evo Answer: A Reasoning Boundary

![The reasoning boundary: semantics, untrusted search, trusted checking](figures/reasoning_boundary.pdf)

Key rules:

- ATPs never resolve names, infer types, expand clusters, or pick overloads;
- the kernel never searches for proofs;
- deterministic pre-ATP discharge needs replayable evidence too - nothing is
  accepted because an earlier phase said "done".

### Frame 5.3 - Certificates, Not Success Bits

![A certificate and its kernel replay](figures/certificate_replay.pdf)

Bullets:

- accepted evidence is normalized into replayable certificate data;
- the kernel checks imported facts, substitutions, clause well-formedness,
  and the resolution/SAT trace. It does not trust a solver's exit code.
  Unsoundness in search cannot cause unsoundness in accepted results;
- hashes link an accepted proof to its dependency slice. The proof cannot
  be reused with another slice without checking.

### Frame 5.4 - The Same Boundary Controls AI

Legacy Mizar (exact MML excerpt):

```mizar
theorem
  for F being non degenerated ZeroOneStr holds 1.F in NonZero F
proof
  let F be non degenerated ZeroOneStr;
  not 1.F in {0.F} by TARSKI:def 1;
  hence thesis by XBOOLE_0:def 5;
end;
```

Message:

- Citation repair is a standard safe AI edit. It proposes a missing or
  more precise `by` reference. The edit is local to the source and keeps
  the meaning. The verifier checks it just like a human edit.

Speaker note:

- Source: current MML `struct_0.miz`, lines 637-643.
- The agent proposes; the verifier and kernel decide. The assistant's
  strength never enters the trusted base.

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

Preserved:

- the de Bruijn discipline: a small checker judges everything;
- proof text stays declarative and readable - automation maintains the
  argument, it does not replace it.

Questions for Bialystok:

- Is the SAT/resolution certificate approach convincing for Mizar-style
  obligations, including clusters and definitional expansions?
- Which evidence format would the team be most willing to audit?

## Part 6. Story 5: Verification That Scales

### Frame 6.1 - The Problem

Bullets:

- Verifying the whole MML is a batch operation measured in hours.
- The reuse boundary is the accepted article: a small change re-verifies
  more than it should.
- Memory follows the article environment, not the actually used interface.
- These costs follow from using the article as the unit in a large
  library. They are not errors in the current design.

### Frame 6.2 - The Evo Answer: Fingerprints And Incrementality

Bullets:

- hash the source, the dependency interfaces, the generated obligations, and
  the proof witnesses;
- reuse a result only when fingerprints and verifier policy match;
- a proof-body-only change does not rebuild importers, because public
  statements and statuses are unchanged;
- independent modules, obligations, ATP runs, and kernel checks run in
  parallel; results are published in canonical order.

![The fingerprint graph: what a change re-verifies](figures/fingerprint_graph.pdf)

```text
Cache reuse is never proof authority.
A clean build must reproduce every acceptance.
```

### Frame 6.3 - The Evo Answer: A Memory Contract [deep dive]

```text
resident memory should scale with:
  active source
  imported public interfaces
  import-filtered indexes
  active module obligations

not with:
  imported proof bodies
  private lemmas outside the interface
  registration data outside the import closure
```

Message:

- Interfaces are loaded; proof bodies are not. This is what makes whole-MML
  editing sessions possible on ordinary hardware.

### Frame 6.4 - What Is Preserved, What We Ask

Preserved:

- clean-build semantics: caching and parallelism change speed, never truth;
- article-style review: what a human reads is still complete source text.

Questions for Bialystok:

- What clean-build equivalence tests would make incremental verification
  trustworthy to the team that maintains MML today?
- Which current maintenance operations (revisions, renamings) should we
  benchmark for incremental cost?

## Part 7. Story 6: Templates For Generic Mathematics

### Frame 7.1 - The Problem, Part One: Schemes Use Separate Rules

Legacy Mizar (exact MML excerpt):

```mizar
scheme
  NatInd { P[Nat] } : for k being Nat holds P[k]
provided
A1: P[0] and
A2: for k be Nat st P[k] holds P[k + 1]
```

Bullets:

- schemes support second-order patterns (induction, separation, replacement).
  They work, but they use a separate mechanism with separate rules;
- schemes can parameterize theorems, but not structures, modes, or functors.

Speaker note:

- Source: current MML `nat_1.miz`, near line 90 (checked July 2, 2026).

### Frame 7.2 - The Problem, Part Two: Copy-Paste Algebra

Bullets:

- `addMagma` and `multMagma` are the same mathematics twice, related only by
  naming convention (we saw this in story 2);
- polynomial rings, vector spaces, and matrix theories are written again for each
  carrier because there is no parameterized construction;
- a theorem proved for one commutative operation is re-proved for `+`
  and `*` separately.

Message:

- Without a generics mechanism, the library needs duplicate articles.
  Each copy needs maintenance.

### Frame 7.3 - The Evo Answer: Templates

Mizar Evo (specification example):

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

Bullets:

- a template is an ordinary `definition` block whose leading `let` binds
  parameters: types, values, predicates, or functors;
- one mechanism covers structures, modes, functors, predicates, theorems,
  registrations, and algorithms;
- readable short forms remain: `Module over R` is an automatic synonym for
  `Module[R]`, `Subset of X` for `Subset[X]`.

### Frame 7.4 - Bounded Parameters And Generic Theorems [deep dive]

Mizar Evo (specification example):

```mizar
definition
  let T be type extends commutative Magma;
  theorem PermProduct[T]:
    for s being FinSequence of T,
        p being Permutation of dom s
    holds Product[T](s) = Product[T](s * p)
  proof ... end;
end;
```

Message:

- `type extends commutative Magma` states what the parameter must provide
  before any proof search begins;
- the theorem is proved once, for every commutative operation.

### Frame 7.5 - One Proof, Many Instantiations [deep dive]

Instantiation (specification example):

```mizar
PermProduct[AddMagma]              :: additive form
PermProduct[commutative MulMagma]  :: multiplicative form

let R be commutative Ring;
PermProduct[R qua AddMagma]        :: R's additive view
PermProduct[R qua MulMagma]        :: R's multiplicative view
```

Message:

- `qua` selects the intended view when a ring reaches Magma along two
  paths - and notation follows the view: the generic `*` displays as `+`
  under the additive instantiation.

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
- instantiation is explicit bracket syntax, so tools and artifacts see
  exactly which instance a proof uses.

### Frame 7.7 - What Is Preserved, What We Ask

Preserved:

- scheme-style reasoning keeps the same power;
- `of` / `over` phrasing keeps mathematical prose readable;
- first-order discipline: templates are checked instantiation, not a new
  logic.

Questions for Bialystok:

- Which MML schemes should be the first migration targets?
- Are brackets acceptable as the canonical identity form, with `of`/`over`
  as display forms?
- The specification limits inference. A parameter is inferred only when
  the declared argument types determine it uniquely. `qua` views are never
  inferred. Does this rule suit real MML idioms, or does it require too many
  explicit `[T]`?

## Part 8. Story 7: Verified Computation With Algorithms

### Frame 8.1 - The Problem

Bullets:

- Current Mizar can define `Gcd` and prove its theory - but cannot compute
  `gcd(48, 18)`; every numeric fact needs a hand-written proof chain.
- There is no checked connection between MML mathematics and executable
  code; verified-algorithm work must leave the system entirely.
- This is a boundary of the design, not a defect: Mizar chose to be a proof
  language. The question is whether that boundary still meets our needs.

Slide text:

```text
The library describes computation.
It cannot perform or export it.
```

### Frame 8.2 - The Evo Answer: Algorithms With Contracts

Mizar Evo (specification example, condensed from spec section 20.12):

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

- `ensures` and the invariant cite the mathematical `Gcd`: no circularity.

Speaker note:

- The contract uses mathematics. The algorithm computes the result, and
  the library functor `Gcd` specifies it. The `decreasing` measure on `y`
  proves termination.

### Frame 8.3 - The Evo Answer: Proof By Computation

Mizar Evo (specification example):

```mizar
theorem EuclidGcd12_8:  euclid_gcd(12, 8)  = 4  by computation;
theorem EuclidGcd100_75: euclid_gcd(100, 75) = 25 by computation;

theorem Fact10: factorial(10) = 3628800
proof
  thus thesis by computation(steps: 100000);
end;
```

Bullets:

- the Mizar Virtual Machine (MVM) evaluates ground goals during
  verification, under explicit step, time, and depth budgets;
- only ground equalities and ground predicates qualify; everything else
  still requires a classical proof;
- imported algorithms are opaque: downstream proofs may use only their
  `ensures` contract, never their body.

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
- once its termination obligations are proved, a `terminating`
  algorithm is promoted to a functor, usable in any proof;
- this is the only way to add recursion to the mathematical layer.
  It requires a proof.

### Frame 8.5 - Computation Never Redefines Truth [deep dive]

Bullets:

- contracts, invariants, and termination measures generate verification
  conditions that pass through the same ATP-plus-kernel boundary as
  ordinary theorems (story 4);
- `by computation` is MVM replay under budgets, not solver trust;
- code extraction (to runtime targets) is strictly downstream of verified
  artifacts and can never affect acceptance.

Message:

- Algorithms let the library express more. They do not change what it
  means for a theorem to be accepted.

### Frame 8.6 - What Is Preserved, What We Ask

Preserved:

- the theorem language is unchanged; algorithms live in `definition` blocks
  and interact with proofs only through contracts and verified promotion;
- first-order set-theoretic foundations stay intact.

Questions for Bialystok:

- Which computational examples would show clear benefits to mathematicians
  without shifting the culture toward programming?
- Are there MML areas (number theory, combinatorics, finite structures)
  where `by computation` would immediately shorten real proofs?
- Which extraction targets matter first, if any?

## Part 9. Story 8: A Library You Can Cite

### Frame 9.1 - The Problem

Bullets:

- Formalized Mathematics gives Mizar a rare feature: a research journal
  linked to a formal library, with articles that people can cite.
- But article identity is closely tied to library organization. Refactoring
  the library can conflict with the structure of published articles.
  Package reuse has no identity for journal citations.
- A written explanation needs an order that helps readers. Reuse needs
  dependency order. One structure cannot serve both as well as possible.

### Frame 9.2 - The Evo Answer: Linked, Not Merged

![The article-to-library link model](figures/fm_links.pdf)

Message:

- each layer answers a different question: research citation, current
  location, semantic drift detection, reproducible verification, and
  links to MML and past Formalized Mathematics volumes.

### Frame 9.3 - Who Gains What [deep dive]

| Audience | Gain |
|---|---|
| readers | written explanations come first; formal source is one click away |
| maintainers | refactoring no longer rewrites published explanations |
| authors | articles cite stable identities, not file layouts |
| AI tools | prose for retrieval, fingerprints for exact context |

### Frame 9.4 - What Is Preserved, What We Ask

Preserved:

- Formalized Mathematics remains a real journal with review and written explanations;
- origin metadata keeps the link to every existing MML citation.

Questions for Bialystok:

- Which identity should be primary in user-facing citations: article label,
  library FQN, or origin id?
- How should existing Formalized Mathematics articles link to migrated
  modules - by updating old links now, when articles are revised, or not at all?

## Part 10. Architecture In One Picture

### Frame 10.1 - The Pipeline

![The pipeline, with responsibility groups](figures/pipeline.pdf)

Message:

- every boundary states who owns a fact, which artifact records it, and
  what must be recomputed when it changes. That is the main purpose.

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

| Story | Pipeline stage |
|---|---|
| dependencies | resolver, package manager |
| structures and automation | checker (inheritance and cluster graphs, traces) |
| search vs trust | ATP layer, kernel, certificates |
| scale | artifacts, fingerprints, scheduler |
| templates | elaborator (checked instantiation) |
| algorithms | VC generator, MVM |
| publication | artifact emitter, doc generation |

Message:

- The eight stories describe one pipeline. Each starts with a different
  problem that users face.

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

Bullets:

- bilingual language specification: 24 chapters plus appendices, English
  canonical with Japanese companions (`doc/spec/`);
- architecture specification: 24 documents covering the pipeline, kernel,
  certificate format, and AI agent interface (`doc/design/architecture/`);
- a Rust workspace of 20 crates - lexer, parser, resolver, checker, VC
  generator, ATP bridge, kernel, build system - roughly 400k lines including
  tests;
- focused audits completed in 2026: kernel soundness, template logic
  encoding, SAT solver dependency;
- the roadmap is split into small tasks that can be verified separately.

Message:

- The planned end-of-2026 alpha is a milestone in work already under way.

### Frame 11.1 - Migration Needs Research

![Roadmap timeline](figures/roadmap_timeline.pdf)

1. End of 2026, alpha: core-subset frontend and parser, import and module
   resolution prototype, structured diagnostics, early artifacts.
2. 2027, migration laboratory: translate 3-5 representative MML articles by
   hand and by script. Classify and record every mismatch.
3. 2027-2028, expansion: foundational set and relation fragments; then
   algebraic structures and dependency cones around successful fragments.

Non-goals for the alpha:

- full MML verification, final compatibility layer, stable AI protocol.

### Frame 11.2 - What We Will Measure [deep dive]

Bullets:

- translated articles and lines; accepted parser subset;
- resolved imports versus unresolved dependencies;
- obligations closed deterministically versus by ATP-plus-certificate;
- memory and wall-clock per module, incremental versus clean;
- compatibility decisions that required human judgment.

### Frame 11.3 - Compatibility Policy And Risks [deep dive]

Policy:

- preserve theorem identity through origin metadata;
- keep compatibility aliases where they help migration;
- record every difference from old behavior with a reason and a test.

| Risk | How to reduce it |
|---|---|
| compatibility work takes all our time | small representative parts first; no all-at-once translation |
| registrations behave differently | trace artifacts and comparison reports early |
| package layout breaks journal links | origin metadata, article-to-library identifiers |
| AI edits hide migration mistakes | Red edits stay forbidden; verifier artifacts required |

### Frame 11.4 - What September 2026 Should Produce

Bullets:

- a list of migration benchmark articles in priority order;
- agreement on compatibility metadata needs;
- review notes on the eight stories, especially structures and clusters;
- a first paper outline;
- a shared list of objections and risks.

### Frame 11.5 - What We Ask Of You

Questions:

- Which MML articles are small but structurally representative?
- Which idioms matter to the Mizar community, beyond their technical use?
- Which of the eight stories is most wrong, and why?
- What migration result would convince the community that Evo is a serious project?

## Part 12. Closing

### Frame 12.1 - The Main Question, Again

Slide question:

```text
What must Mizar Evo preserve so that the Mizar community
still recognizes it as Mizar?
```

Speaker note:

- Return to the opening example: the structure that still reads as Mizar.
- Ask for objections to each story, as well as to the overall plan.

### Frame 12.2 - Closing

Final slide:

```text
Update Mizar Evo where a larger library needs it.
Keep the current design where it defines Mizar's mathematical identity.
```

## Backup A. Prepared Exact Examples

Prepared short excerpts for Beamer conversion:

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

The slides show examples only.
The specification is the authority for grammar and semantics.
This map replaces the EBNF shown in earlier drafts.

| Topic | Specification source (under `doc/spec/en/`) |
|---|---|
| Modules and imports | `12.modules_and_namespaces.md` |
| Structures and inheritance | `05.structures.md` |
| Attributes and modes | `06.attributes.md`, `07.modes.md` |
| Registrations and reductions | `17.clusters_and_registrations.md` |
| Templates and schemes | `18.templates.md` |
| Algorithms and MVM | `20.algorithm_and_verification.md` |
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
5. Certificate object and replay (story 4).
   [done: `figures/certificate_replay.pdf`, used in frame 5.3]
6. Incremental fingerprint graph (story 5).
   [done: `figures/fingerprint_graph.pdf`, used in frame 6.2]
7. Formalized Mathematics article-to-library link model (story 8).
   [done: `figures/fm_links.pdf`, used in frame 9.2]
8. Full pipeline with story overlay (Part 10).
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
5. Verifier architecture, certificates, and the small kernel.
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
| Why not improve current Mizar incrementally? | The problems concern boundaries: article granularity, monolithic trust, and implicit environments. These boundaries cannot change in small steps. The rest of the design stays close to Mizar. |
| What happens to authorship and credit of MML articles? | Origin metadata keeps article identity and authorship during migration. The `pub` namespace keeps published articles unchanged so they can still be cited. |
| Is this a fork of the community? | It is a proposal to the community; this visit is its first review. The namespace governance model assumes that the Mizar team controls the `mml` root. |
| What about GPL / CC-BY-SA obligations? | Migration preserves license and attribution of MML content; toolchain licensing is open for discussion. |
| Are the claims about AI too strong? | AI assistance is an optional layer that never enters the trusted base; every proposal also works without it. |
| Why a new kernel instead of the existing checker? | Certificate replay needs a small, auditable core. The existing checker's semantics remain the reference for obligation generation. |

Speaker note:

- Use these answers only if someone asks these questions.
