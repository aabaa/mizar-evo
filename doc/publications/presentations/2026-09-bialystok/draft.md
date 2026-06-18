# Mizar Evo: Second Draft For The Bialystok Mizar Team

Status: second content draft before Beamer conversion.

Planned occasion: visit to the Mizar team in Bialystok in September 2026.

Companion Japanese draft: `draft.ja.md`.

## Working Thesis

Mizar Evo should be presented as a continuation of the Mizar tradition, not as a
replacement story. The central claim is:

> Mizar Evo keeps Mizar's readable mathematical vernacular, and rebuilds the
> language boundary, verifier pipeline, artifact model, and publication
> workflow so that large-scale formal mathematics can be maintained with
> predictable automation, AI assistance, and reproducible verification.

The talk should not ask the Bialystok team to accept a finished design. It
should ask them to review the proposed direction against the real constraints
of current Mizar, MML maintenance, and Formalized Mathematics.

## Source Status

### Repository Sources

This draft relies on current repository documents:

- `doc/spec/en/01.introduction.md`
- `doc/spec/en/05.structures.md`
- `doc/spec/en/06.attributes.md`
- `doc/spec/en/12.modules_and_namespaces.md`
- `doc/spec/en/18.templates.md`
- `doc/spec/en/23.package_management_and_build_system.md`
- `doc/spec/en/sample_codes.md`
- `doc/design/architecture/en/00.pipeline_overview.md`
- `doc/design/architecture/en/08.reasoning_boundary.md`
- `doc/design/architecture/en/15.kernel_certificate_format.md`
- `doc/design/architecture/en/20.test_strategy.md`
- `doc/design/architecture/en/21.ai_agent_interface.md`

### External Sources Checked For This Draft

Checked on June 18, 2026:

- Mizar home page, current release metadata: Mizar 8.1.15 and MML 5.94.1493
  dated May 30, 2025.
  <https://mizar.uwb.edu.pl/>
- Mizar System page, article preparation workflow: source editing,
  Accommodator, Verifier, Exporter, and Library Committee submission.
  <https://mizar.uwb.edu.pl/system/index.html>
- Mizar Mathematical Library page, library as verified articles based on
  built-in notions and Tarski-Grothendieck axioms.
  <https://mizar.uwb.edu.pl/library/>
- Formalized Mathematics page, journal metadata, article-in-press entry for
  volume 34 (2026), and volume list.
  <https://mizar.uwb.edu.pl/fm/>
- Current HTML-linked `ALGSTR_0`, used only to check the shape of legacy
  algebraic structure examples.
  <https://mizar.uwb.edu.pl/version/current/html/algstr_0.html>
- Current plain-text MML sources, used for exact short excerpts and line
  numbers:
  - `ALGSTR_0`: <https://mizar.uwb.edu.pl/version/current/mml/algstr_0.miz>
  - `STRUCT_0`: <https://mizar.uwb.edu.pl/version/current/mml/struct_0.miz>

The legacy Mizar source snippets in this draft are intentionally short. The
plain-text files state GPL-3.0-or-later / CC-BY-SA-3.0-or-later distribution
terms; the final deck should preserve article attribution, source URLs, and
line numbers in speaker notes.

## Review Goals For Bialystok

1. Does the project preserve enough of Mizar's identity?
2. Are the proposed language changes justified by scale, tooling, and
   migration needs?
3. Is the trust boundary between Mizar-side semantics, ATP search, and the
   small kernel technically credible?
4. Which MML fragments should become the first migration benchmarks?
5. How should a package-oriented library connect to Formalized Mathematics?

## Main Claims To Defend

| Claim | Why it matters | Evidence to show |
|---|---|---|
| Mizar's readability is the asset to preserve. | It differentiates Mizar from tactic-heavy proof assistants and makes human review possible. | Side-by-side code examples and publication examples. |
| Readability also improves AI operability. | AI agents need stable, searchable, local text patterns. | Safe edit classes, patch flow, citation repair examples. |
| Scale requires module, artifact, and package boundaries. | Monolithic article-level dependency management is hard to evolve. | Import examples, package manifest, dependency fingerprints. |
| ATP search should be powerful but untrusted. | AI and ATP output must not define proof truth. | ATP -> certificate -> kernel flow. |
| The kernel should check evidence, not search. | Smaller trusted base and clearer de Bruijn-style story. | Certificate fields and rejection categories. |
| Formalized Mathematics and the library should be linked but separated. | Articles need exposition; libraries need stable reusable modules. | Article-to-library link model. |

## Recommended Beamer Shape

The second draft is designed for a technical visit, not a conference talk. It
should now become a detailed five-part review deck. Repository/source-status
notes remain useful in this Markdown file, but they should not appear as the
first slides of the Beamer deck.

It can become either:

- a 60-75 frame discussion deck with extensive backup slides; or
- a 120-150 frame full technical deck for a longer workshop-style meeting.

Recommended structure:

| Part | Topic | Main purpose | Approx. frames |
|---|---|---|---:|
| 1 | Concept | readability, AI-readiness, scalability, and non-goals | 15-20 |
| 2 | Language specification | grammar, syntax deltas, migration examples, and review questions | 70+ |
| 3 | Architecture | trust, ATP, SAT kernel, artifacts, LSP, and AI boundary | 25-30 |
| 4 | Roadmap | MML migration phases, compatibility policy, and benchmarks | 10-15 |
| 5 | Formalized Mathematics | library/article split, publication links, and open questions | 8-12 |
| Backup | detailed examples | source, diagrams, policy tables | 20+ |

## Part 0. Opening

### Frame 0.1 - Title

Title:

```text
Mizar Evo
Readable, AI-Ready, Scalable Formal Mathematics
```

Subtitle:

```text
Discussion draft for the Bialystok Mizar team
September 2026
```

Speaker note:

- Begin with gratitude.
- State that this is a design review with the people best positioned to judge
  whether the project is still recognizably Mizar.

### Frame 0.2 - The One-Sentence Proposal

Slide text:

```text
Preserve Mizar's mathematical vernacular.
Modernize the compiler, verifier, artifact, and publication layers.
```

Speaker note:

- The proposal is not "make Mizar look like Lean" or "replace MML".
- It is "make the Mizar style viable under today's scale and automation
  pressures".

### Frame 0.3 - What We Need From This Visit

Bullets:

- identify compatibility constraints that are invisible from outside MML work;
- choose migration examples that are small but representative;
- review the trust boundary for ATP and kernel checking;
- discuss how Formalized Mathematics should link to a package library.

### Frame 0.4 - What This Talk Does Not Claim

Bullets:

- Full MML migration is not complete.
- The final language standard is not frozen.
- The AI-facing protocol is not a substitute for proof checking.
- The current Mizar system's achievements are the baseline, not the problem.

### Frame 0.5 - Running Question

Slide question:

```text
What must Mizar Evo preserve so that the Mizar community still recognizes it
as Mizar?
```

## Part 1. Current Mizar As The Baseline

### Frame 1.1 - Why Start With Current Mizar?

Message:

- The redesign is only meaningful if it starts from Mizar's strengths.
- Mizar already solved a problem many systems still struggle with: readable
  formal mathematical text.

### Frame 1.2 - Current Operational Model

Based on the Mizar System page:

```text
source article
  -> Accommodator builds an article-specific environment
  -> Verifier checks the text
  -> Exporter extracts accepted facts and definitions
  -> Library review and inclusion in MML
```

Interpretation for the talk:

- This model is historically successful.
- It is article-centered and database-centered.
- Mizar Evo keeps verification and review, but makes dependency, artifact, and
  package boundaries more explicit.

### Frame 1.3 - MML As A Verified Article Library

Main points:

- MML consists of Mizar articles.
- Foundational material includes built-in notions and Tarski-Grothendieck set
  theory.
- Other articles are verified consequences and can become available to future
  articles.

Talk angle:

- The library is not only source code. It is a curated mathematical corpus.
- A migration plan must respect article history, theorem identity, and review
  practice.

### Frame 1.4 - Formalized Mathematics As A Publication Layer

Main points:

- Formalized Mathematics is an established publication venue connected to MML.
- It carries scholarly exposition, not only verifier input.
- Mizar Evo should strengthen this bridge rather than replace it with package
  metadata alone.

### Frame 1.5 - Baseline Strengths

Bullets:

- Declarative proof text.
- Soft typing and mathematical modes.
- Attributes, registrations, and clusters.
- A mature curated library.
- A publication culture around formal articles.

### Frame 1.6 - Baseline Pressure Points

Bullets:

- Article environments can hide the exact local dependency surface.
- Large libraries need finer-grained cache and artifact boundaries.
- Modern editor workflows need partial, resilient analysis.
- AI agents need bounded, structured context.
- Publication and reusable library layout have different needs.

### Frame 1.7 - Design Rule

Slide text:

```text
Do not trade away readability to gain automation.
Use automation to protect and extend readability.
```

## Part 2. Design Concept

### Frame 2.1 - Three Pillars

The design is judged by three constraints that must hold at the same time.

| Pillar | What it means | Why it matters |
|---|---|---|
| Readability | Proof scripts stay close to mathematical exposition; local disambiguation is preferred when ambiguity matters. | Human reviewers can inspect the argument without replaying hidden automation. |
| AI-readiness | Verifier context, source spans, obligations, and safe edit classes are exposed through small auditable interfaces. | AI can help with search, explanation, migration, and patch proposals without becoming trusted proof authority. |
| Scalability | Imports, artifacts, packages, caches, and publication links have stable boundaries. | Large-library maintenance becomes predictable without changing theorem meaning. |

The three pillars reinforce each other:

```text
Readable source
  -> better human review
  -> better AI retrieval and edits
  -> clearer artifacts and dependency traces
  -> more reliable large-library maintenance
```

Design test:

- A feature is good only if it improves automation or scale while preserving
  readable mathematics.

### Frame 2.2 - A Design Anti-Goal

Slide text:

```text
Mizar Evo should not make proofs shorter by making the argument invisible.
```

Speaker note:

- This is the key contrast with very tactic-heavy workflows.
- Some automation is valuable, but the accepted proof artifact must remain
  inspectable.

### Frame 2.3 - First-Order Core As A Strategic Choice

Bullets:

- A first-order set-theoretic core matches much ordinary mathematical practice.
- ATP tools are strong in first-order reasoning.
- Soft types, modes, attributes, and registrations can remain Mizar-facing
  structure around that core.
- The kernel can focus on checking evidence rather than performing high-level
  elaboration.

Design reason:

- This choice preserves the current Mizar foundation while letting powerful ATPs
  work at the layer where they are strongest.
- It also avoids making dependent-type elaboration or tactic execution part of
  the trusted explanation of ordinary mathematical text.

### Frame 2.4 - What Modernization Means

Bullets:

- explicit module imports;
- templates for parameterized definitions and theorem schemas;
- package manifests and lockfiles;
- stable fully-qualified names;
- deterministic build artifacts;
- incremental and parallel verification;
- LSP for editors;
- planned MCP-style context surfaces for AI agents;
- versioned links from publications to library objects.

### Frame 2.5 - What Modernization Does Not Mean

Bullets:

- not abandoning declarative proof style;
- not making ATP success trusted by default;
- not replacing human mathematical review with AI;
- not forcing every old article into a new style at once.

## Part 3. Lean As A Design Contrast

### Frame 3.1 - Why Compare With Lean?

Message:

- Lean is one of the most successful modern proof assistant ecosystems.
- Its success clarifies the design space.
- Mizar Evo should learn from Lean without copying Lean's core trade-offs.

### Frame 3.2 - Careful Framing

Slide text:

```text
This is not "Lean is wrong".
It is "Mizar Evo chooses different constraints".
```

### Frame 3.3 - Trade-Off Table

| Topic | Lean direction | Mizar Evo direction |
|---|---|---|
| Foundation | dependent type theory | first-order set-theoretic core with soft types |
| Proof style | tactics and terms | declarative Mizar-style text |
| Syntax | expressive macros and notation | disciplined, stable surface syntax |
| Automation | elaboration, type classes, tactics, simplifiers | Mizar-side semantics plus ATP and checked certificates |
| AI context | proof states and tactic traces | source text, obligations, artifacts, safe patches |

### Frame 3.4 - Readability Contrast

Careful claim:

- Tactics are powerful and often essential in Lean practice.
- A tactic script can also hide the final mathematical argument behind
  procedure.
- Mizar Evo should optimize for proof text that remains useful as mathematical
  prose and as publication source.

### Frame 3.5 - Scalability Contrast

Careful claim:

- Lean's elaboration and type-class ecosystem supports very expressive
  libraries.
- Mizar Evo should instead keep name resolution, cluster resolution, overload
  resolution, and ATP dispatch separated and traceable.
- The point is predictable local context and reproducible artifacts.

### Frame 3.6 - AI Contrast

Careful claim:

- Lean agents often operate by generating tactics or terms against proof states.
- Mizar Evo agents should operate by small source edits:
  citations, explicit coercions, imports, local assertions, and explanations.
- The verifier independently accepts or rejects every edit.

### Frame 3.7 - Discussion Question

Question:

```text
Which Lean-style conveniences would be valuable in Mizar Evo, and which would
we intentionally reject to preserve readability?
```

## Part 4. Language Changes And Migration Examples

### Frame 4.1 - Language Change Strategy

Bullets:

- Preserve the recognizable mathematical surface.
- Make dependencies and names more explicit.
- Separate structure definition, inheritance mapping, and template
  parameterization.
- Give tools stable identities and source spans.
- Keep compatibility metadata for migrated MML material.

### Frame 4.1a - Mizar To Evo Language Delta Inventory

This is a checklist, not a claim that every item must be finalized before the
visit. Artifact and workflow deltas that affect migration semantics follow in
the next inventory slide.

| Area | Current Mizar baseline | Evo delta | Why it changes |
|---|---|---|---|
| Article environment | `environ` roles such as vocabularies, notations, constructors, registrations, requirements | explicit `import` prelude and module/package namespace | make the dependency surface reproducible and reviewable |
| Module interfaces | article export is mediated by library tooling, with fewer source-level interface boundaries | `export`, `public`/`private`, opaque imports, and separate compilation define the public API surface | keep reusable interfaces stable while proof bodies and helpers stay private |
| Symbol identity | article-local names and MML article labels | path-derived FQNs and module-qualified labels | support migration, renaming, API compatibility, and citation repair |
| Symbolic notation and aliases | broad symbolic constructor vocabulary, with synonyms and antonyms as mathematical aliases | arbitrary symbols concentrated in `func` and `pred`; type constructors stay identifier-like; synonym/antonym equivalence is preserved | stabilize lexing, parsing, editor tooling, and AI edits without losing alternative notation |
| Operator precedence and parse domains | notation is powerful but precedence choices can be implicit in parser/verifier behavior | explicit operator metadata, separate term/formula precedence domains, and parse-before-overload discipline | make grouping deterministic, reviewable, and independent of overload choice |
| Lexical activation | vocabulary and notation availability is article-environment driven | import-prelude pre-scan, source-position active lexicon, no forward references, operator metadata after declaration | make tokenization and dot/operator disambiguation deterministic |
| Soft type foundation | Mizar soft types over set-theoretic objects | preserve soft typing while making radix/mode heads, `object`/`set`, type erasure, widening/narrowing, and `reconsider` obligations explicit | keep Mizar's foundation recognizable while exposing verifier obligations |
| Structures | parent structure, fields, and selectors are tightly coupled in one declaration | `struct`, `field`, `property`, and `inherit` are separate concepts | distinguish layout, derived canonical values, and inherited views |
| Inheritance | inherited fields are mostly implicit in structure syntax | `inherit Child extends Parent where ...` records mapping, renaming, narrowing, and coherence | make multiple inheritance and diamond cases auditable |
| Modes and attributes | soft type vocabulary with adjectives and clusters | modes remain type abbreviations/refinements; attributes are type-refining predicates | preserve Mizar style while separating classification from data layout |
| `sethood` and comprehensions | Fraenkel-style set formation relies on set-theoretic side conditions | mode `sethood` and comprehension checks are explicit type-checker obligations | preserve set-theoretic foundations before ATP search starts |
| Definitions and correctness | `func`/`pred` definitions use familiar `equals`/`means` styles | preserve `equals`, `means`, `existence`, `uniqueness`, and `assume` side conditions as explicit obligations | keep Mizar's definitional idiom while making obligations auditable |
| Predicate and functor properties | algebraic properties support concise automatic reasoning | proof-backed property declarations such as `commutativity`, `symmetry`, and `irreflexivity` | keep compact mathematical notation while recording exactly what automation may use |
| Overload and `redefine` | overloaded symbols and redefinitions are central Mizar idioms | root/family overload resolution, `coherence with`, and refinement joins are recorded and explainable | avoid source-order ambiguity and make refined result facts stable |
| Registrations, clusters, and reductions | powerful automatic propagation and simplification, often hard to explain locally | labeled registration items, import-filtered resolution graph, reduction index, trace artifacts | keep automation but make inference and rewriting explainable |
| Schemes and generics | classical `scheme`, `of`/`over` parameterization | first-class templates; bracket form is canonical, constraints and inference are explicit, `of`/`over` are shorthands | unify parameterized definitions and theorem schemas without hiding instantiation choices |
| Declarative proof skeleton | Jaśkowski-style `proof ... end` text with `let`, `assume`, `thus`, `hence`, `thesis`, and diffuse reasoning | preserve the readable proof skeleton and emit extracted obligations, thesis states, and source spans as metadata | protect Mizar's proof prose while making proof state toolable |
| Proof status | verified theorem items as the normal publication unit | `open`, `assumed`, and `conditional` theorem statuses | separate unsettled work, assumptions, and clean verified results |
| Proof citations | `by` references to visible facts | grouped/bulk citations, used-axiom recording, citation refinement | support small verifier-checked repairs and AI-assisted edits |
| Type views | existing `qua`, implicit widening, and local disambiguation idioms | keep source-level `qua`, and emit resolved view/coercion metadata artifacts | make overload and inherited-view choices inspectable |
| Algorithms | mathematical proof language, not a general verified programming surface | `algorithm`, contracts, invariants, termination, MVM, `by computation` | support verified computation without changing theorem truth |

Speaker note:

- The important review question is not "how many syntactic changes exist?" but
  "which Mizar-facing idea is being preserved, and which hidden dependency is
  being made explicit?"

### Frame 4.1b - Mizar To Evo Artifact And Workflow Delta Inventory

These rows are not purely syntactic, but they affect what migration tools,
reviewers, editors, and packages can trust.

| Area | Current Mizar baseline | Evo delta | Why it changes |
|---|---|---|---|
| Annotations | comments and informal tool hints | semantic-neutral annotations such as `@proof_hint`, `@show_type`, `@show_resolution`, `@latex` | guide tools while keeping logical meaning unchanged |
| Diagnostics and LSP | verifier messages are primarily human-facing output | stable diagnostic codes, primary/secondary spans, fix suggestions, lazy explanations, and LSP records | make failures actionable for humans, editors, and AI tools |
| ATP boundary | proof search and verifier behavior are not surfaced as a stable artifact boundary | ATP dispatch plus independently checked certificates and rejection categories | keep a small trusted base |
| Packages and dependency resolution | article/MML workflow | `mizar.pkg`, lockfile, SemVer, features, compatibility checks, cached verifier artifacts | enable reproducible, package-oriented development |
| Incremental verification | accepted article output is the main reuse boundary | dependency slices, VC anchors, witness hashes, and cache keys; cache reuse is never proof authority | scale verification while preserving clean-build equivalence |
| Documentation generation | source comments and publication pages are separate from reusable API documentation | `mizar doc`, `:::` doc comments, label/FQN cross-links, `@latex`, docs from verified artifacts | produce browsable API documentation without making docs a proof gate |
| Code extraction | no general verified extraction workflow in the source language | terminating algorithms, ghost erasure, target-neutral runtime IR, extractor configuration | keep executable output downstream of verified computation |
| Formalized Mathematics link | article and library identity are closely coupled | publication metadata links to reusable library modules | preserve citation value while supporting package reuse |

Speaker note:

- These are artifact deltas, but they belong near the language discussion
  because they determine whether a migrated source change remains explainable.

### Frame 4.1b.01 - Language Specification Review Contract

Message:

- This chapter is the part we should not compress.
- The Bialystok review should treat the language surface as a specification,
  not only as a design story.
- Each syntax change should be reviewed for three things: readability for Mizar
  authors, migration cost for MML, and determinism for parsers, verifiers, and
  tools.
- The examples below are not final tutorial material. They are review targets:
  if a construct looks too unlike Mizar, or if a hidden Mizar idiom is missing,
  that is exactly the feedback we need.

### Frame 4.1b.02 - Grammar Sources For This Review

Canonical specification sources:

All paths in this table are under `doc/spec/en/`.

| Topic | Source |
|---|---|
| Cross-chapter grammar | `appendix_a.grammar_summary.md` |
| Operator precedence | `appendix_b.operator_precedence.md`, `10.functors.md` |
| Lexical structure | `02.lexical_structure.md` |
| Structures | `05.structures.md` |
| Attributes and modes | `06.attributes.md`, `07.modes.md` |
| Terms and formulas | `13.term_expression.md`, `14.formulas.md` |
| Statements and proofs | `15.statements.md`, `16.theorems_and_proofs.md` |
| Registrations and templates | `17.clusters_and_registrations.md`, `18.templates.md` |

Review rule:

- The slides summarize the surface grammar. The spec files remain the
  authority for edge cases and well-formedness rules.

### Frame 4.1b.03 - Surface Syntax Review Map

The review should cover this grammar stack in order:

1. lexical classes and active lexicon;
2. module/import prelude and item activation;
3. definition blocks and visibility;
4. type expressions, attributes, modes, and structures;
5. functors, predicates, notation, properties, and overload families;
6. operator precedence and term/formula boundary;
7. terms, formulas, comprehensions, `the`, `sethood`, and `qua`;
8. theorem/proof statements and citations;
9. registrations, reductions, templates, annotations, and algorithms.

Why this order matters:

- Mizar-style readability is not only local syntax. It depends on when names
  become visible, which notation is active, and which automation is allowed to
  fire.

### Frame 4.1b.04 - Compilation Unit Syntax

Grammar sketch:

```ebnf
compilation_unit ::= import_prelude export_prelude
                     { annotated_declaration } ;

import_prelude   ::= { import_stmt } ;
export_prelude   ::= { export_stmt } ;

declaration      ::= definition_block
                   | reserve_decl
                   | registration_block
                   | claim_block
                   | [ visibility ] theorem_item
                   | [ visibility ] notation_decl ;

visibility       ::= "private" | "public" ;
```

Example:

```mizar
import algebra.group as group;
export algebra.group;

public theorem left_cancel:
  for G being Group, a, b, c being Element of G
  st a * b = a * c holds b = c;
```

Review point:

- The file begins with a reproducible dependency surface, then ordinary Mizar
  items. This replaces article-environment roles without turning the language
  into a generic programming module system.

### Frame 4.1b.05 - Import Prelude And Item Activation

Grammar sketch:

```ebnf
import_stmt ::= "import" module_alias_decl
                { "," module_alias_decl } ";" ;
export_stmt ::= "export" module_path { "," module_path } ";" ;

module_alias_decl ::= module_path [ "as" module_identifier ]
                    | module_branch_import ;
```

Example:

```mizar
import algebra.group as group;
import algebra.ring.{Ring, Ideal};
export algebra.group;
```

Semantic rule:

- All `import` statements must appear before the first non-import item.
- Imported symbols seed the active lexicon for the body.
- Declarations in the current module become active only after their item is
  complete.
- A later import-shaped statement is a syntax error, not a dynamic environment
  update.

Why:

- This makes lexing, parsing, and AI context extraction deterministic.

### Frame 4.1b.06 - Lexical Classes

Grammar sketch:

```ebnf
identifier       ::= ( letter | "_" )
                     { letter | digit | "_" | "'" } ;
constructor_name ::= identifier | readable_constructor_name ;
user_symbol      ::= symbol_char { symbol_char } ;
```

Example:

```mizar
struct ZeroStr where
  field carrier -> set;
end;

func + (x, y: Integer) -> Integer equals int.add(x, y);
```

Policy:

- `mode`, `struct`, and `attr` names use constructor names.
- `field` and `property` names are identifiers.
- Arbitrary symbolic notation is concentrated in `func` and `pred`.
- Tokenization uses the longest active match at the source position.

Review point:

- This is one of the largest visible syntax differences from legacy Mizar; it
  should be reviewed with real algebraic and set-theoretic examples.

### Frame 4.1b.07 - Reserved Symbols And Dot Disambiguation

Reserved punctuation includes:

```text
, . .. ; : := ( ) [ ] { } .{ = <> & -> .= .* @[ ...
```

Dot policy:

- `.` can be selector syntax, namespace qualification, or a user functor
  symbol, depending on grammar position and the active lexicon.
- The parser preserves enough surface form for the resolver to classify it.

Bracket policy:

- `[` and `]` are reserved for template arguments and built-in bracket functor
  syntax.
- Postfix indexing such as `x[y]` is not silently admitted.

Example:

```mizar
let R be Ring;
set U = R.carrier;
set e = algebra.group.identity(R qua Group);
```

Why:

- Ambiguous punctuation must not make migration depend on parser guesses.

### Frame 4.1b.08 - Type Expression Syntax

Grammar sketch:

```ebnf
type_expression ::= attribute_chain type_head ;
type_head       ::= radix_type | mode_type ;

attribute_chain ::= { [ "non" ] attribute_ref } ;
attribute_ref   ::= [ param_prefix ]
                    [ struct_ref_name "." ] attribute_name
                    [ "(" argument_list ")" ] ;

radix_type      ::= "object" | "set"
                  | struct_ref_name [ type_args ] ;
mode_type       ::= mode_ref_name [ type_args ] ;
```

Example:

```mizar
reserve G for non empty Group;
let x be Element of G;
let H be commutative subgroup of G;
```

Review point:

- Mizar's soft type style is preserved, but the grammar makes the split between
  radix type, mode type, and attribute chain explicit.

### Frame 4.1b.09 - Definition Block Syntax

Grammar sketch:

```ebnf
definition_block   ::= "definition"
                         { definition_content }
                       "end" ";" ;

definition_content ::= { annotation }
                       ( definition_parameter_decl
                       | assumption
                       | correctness_condition
                       | property_item
                       | [ visibility ] definitional_item
                       | [ visibility ] theorem_item
                       | [ visibility ] registration_item ) ;
```

Example:

```mizar
definition
  let G be Group;
  func UnitDef: unit(G) -> Element of G
    means for x being Element of G holds it * x = x;
  existence by group.unit_exists;
  uniqueness by group.unit_unique;
end;
```

Review point:

- The `definition ... end` envelope remains familiar.
- New surface elements should be judged by whether they clarify obligations
  without breaking Mizar's definition-oriented reading style.

### Frame 4.1b.10 - Structure Declaration Syntax

Grammar sketch:

```ebnf
struct_def    ::= "struct" struct_def_name [ type_params ]
                  "where" struct_member { struct_member }
                  "end" ";" ;

struct_member ::= field_decl | property_decl ;
field_decl    ::= "field" identifier "->" type_expression
                  [ ":=" term_expression ] ";" ;
property_decl ::= "property" identifier "->" type_expression ";" ;
```

Example:

```mizar
struct UnitalMagmaStr where
  field carrier -> non empty set;
  field mult -> BinOp of carrier;
  property unit -> Element of carrier;
end;
```

Review point:

- `field` declares intrinsic data of the structure value.
- `property` declares a canonical derived slot or obligation.
- The syntax deliberately separates layout from mathematically determined
  values.

### Frame 4.1b.11 - Property Implementation Syntax

Grammar sketch:

```ebnf
property_means_impl  ::= "property" identifier "." identifier
                         "means" formula_definiens ";"
                         existence_block uniqueness_block ;
property_equals_impl ::= "property" identifier "." identifier
                         "equals" term_definiens ";" ;
```

Example:

```mizar
definition
  let M be UnitalMagma;
  property M.unit means
    for x being Element of M.carrier holds
      M.mult(it, x) = x & M.mult(x, it) = x;
  existence by magma.unit_exists;
  uniqueness by magma.unit_unique;
end;
```

Review point:

- A `property` in a `struct` is a virtual slot, not stored field data.
- The implementation is supplied later for a mode whose attributes guarantee
  the required assumptions.
- `means` carries `existence` and `uniqueness`; `equals` directly supplies the
  value.

### Frame 4.1b.12 - Inheritance Declaration Syntax

Grammar sketch:

```ebnf
inherit_def ::= "inherit" inherit_child "extends" parent_type
                [ "where" inherit_member { inherit_member }
                  [ coherence_block ] "end" ] ";" ;

field_redef    ::= "field" identifier [ "->" type_expression ]
                   "from" ( identifier | "it" ) ";" ;
property_redef ::= "property" identifier [ "->" type_expression ]
                   "from" identifier ";" ;
```

Example:

```mizar
inherit GroupStr extends UnitalMagmaStr where
  field carrier from carrier;
  field mult from mult;
  property unit from unit;
  coherence by group.inherit_unital_magma;
end;
```

Review point:

- Each `inherit` statement names exactly one parent.
- Renaming, narrowing, and coherence are visible source items.
- Diamond inheritance becomes a checked mapping problem rather than a hidden
  merge.

### Frame 4.1b.13 - Attribute And Mode Syntax

Grammar sketch:

```ebnf
attr_def ::= "attr" label ":" subject "is" attr_pattern
             "means" formula_definiens ";" ;

mode_def ::= "mode" label ":" mode_def_name [ type_params ]
             "is" attribute_chain radix_type ";"
             [ mode_property ] ;
```

Example:

```mizar
attr AssocDef:
  G is associative means
    for x, y, z being Element of G holds (x * y) * z = x * (y * z);

mode GroupDef: Group is associative invertible UnitalMagmaStr;
```

Review point:

- Attributes are unary type-refining predicates.
- Modes name reusable type classifications.
- This keeps Mizar's adjective-rich prose while making cluster participation
  explicit.

### Frame 4.1b.14 - Predicate And Functor Syntax

Grammar sketch:

```ebnf
pred_def ::= "pred" label ":" pred_pattern
             "means" formula_definiens ";" ;

func_def ::= "func" label ":" func_pattern "->" type_expression
             ( "means" formula_definiens
             | "equals" term_definiens ) ";" ;
```

Example:

```mizar
pred DividesDef:
  x divides y means ex z being Integer st y = x * z;

func InvDef: inverse(G, x) -> Element of G
  means x * it = unit(G) and it * x = unit(G);
```

Definition patterns:

- Predicate and functor patterns may use symbolic or phrase notation.
- Their symbols become active only after the declaration item is complete.
- Overloaded definitions are resolved by roots, argument types, and refinement
  rules, not by accidental source-order behavior.

### Frame 4.1b.15 - Synonyms, Antonyms, And `redefine`

Grammar sketch:

```ebnf
synonym_def  ::= "synonym" alt_pattern "for" original_pattern ";" ;
antonym_def  ::= "antonym" alt_pattern "for" original_pattern ";" ;

redefine_attr ::= "redefine" "attr" label ":" subject "is" attr_pattern
                  "means" formula_definiens ";"
                  "coherence" [ "with" label ] justification ";" ;
redefine_pred ::= "redefine" "pred" label ":" pred_pattern
                  "means" formula_definiens ";"
                  "coherence" [ "with" label ] justification ";" ;
redefine_func ::= "redefine" "func" label ":" func_pattern
                  "->" type_expression
                  ( "means" formula_definiens | "equals" term_definiens ) ";"
                  "coherence" [ "with" label ] justification ";" ;
```

Example:

```mizar
definition
  let a, b be Real;
  pred LessDef: a < b means real.lt(a, b);
  synonym b > a for a < b;
  antonym a >= b for a < b;
end;

definition
  let x be non negative Real;
  redefine func AbsNonNeg: |.x.| -> non negative Real equals x;
  coherence with AbsGeneral by real.abs_nonneg;
end;
```

Review point:

- Synonyms and antonyms preserve mathematical phrasing and natural negation.
- They are semantic aliases, not new roots.
- `redefine` sharpens the same root with `coherence with`; it is not a rival
  overload.

### Frame 4.1b.16 - Predicate And Functor Properties

Grammar sketch:

```ebnf
pred_property ::= ( "symmetry" | "asymmetry" | "connectedness"
                 | "reflexivity" | "irreflexivity" )
                  justification ";" ;

func_property ::= ( "commutativity" | "idempotence"
                 | "involutiveness" | "projectivity" )
                  justification ";" ;
```

Example:

```mizar
definition
  let x, y be Real;
  pred LessDef: x < y means x is_less_than y;
  asymmetry by real.lt_asym;
  irreflexivity by real.lt_irrefl;
end;

definition
  let X, Y be set;
  func UnionDef: X \/ Y -> set means
    for z being object holds z in it iff z in X or z in Y;
  commutativity by set.union_comm;
  idempotence by set.union_idem;
end;
```

Review point:

- Properties are not comments and not trusted hints.
- Each property is accepted only with a proof obligation.
- The design keeps algebraic notation compact while recording exactly which
  automatic matching, rewriting, and ATP facts may be used.

### Frame 4.1b.17 - `equals`, `means`, And Correctness Blocks

Key distinction:

| Form | Meaning | Obligations |
|---|---|---|
| `equals` | gives a direct term definition | usually immediate well-definedness |
| `means` | characterizes a value or relation | may require `existence`, `uniqueness`, `coherence`, or `compatibility` |
| `assume` | adds a local definition-side condition | must be recorded as part of the generated obligation |

Grammar sketch:

```ebnf
existence_block     ::= "existence" justification ";" ;
uniqueness_block    ::= "uniqueness" justification ";" ;
coherence_block     ::= "coherence" justification ";" ;
compatibility_block ::= "compatibility" justification ";" ;
```

Example:

```mizar
func InvDef: inverse(G, x) -> Element of G
  means x * it = unit(G) and it * x = unit(G);
existence by group.inverse_exists;
uniqueness by group.inverse_unique;
```

Review point:

- The syntax should preserve Mizar's definitional idiom while exposing the
  proof obligations that migration tools must not hide.

### Frame 4.1b.18 - Operator Declarations And Precedence

Grammar sketch:

```ebnf
infix_operator_decl   ::= "infix_operator" "(" string_literal ","
                          infix_assoc "," nat_literal ")" ";" ;
prefix_operator_decl  ::= "prefix_operator" "(" string_literal ","
                          nat_literal ")" ";" ;
postfix_operator_decl ::= "postfix_operator" "(" string_literal ","
                          nat_literal ")" ";" ;
infix_assoc           ::= "left" | "right" | "none" ;
```

Example:

```mizar
infix_operator("+", left, 80);
infix_operator("*", left, 90);
infix_operator("^", right, 95);
prefix_operator("-", 85);
postfix_operator("!", 95);

a + b * c       :: a + (b * c)
a ^ b ^ c       :: a ^ (b ^ c)
a %% b %% c     :: error if %% is non-associative
```

Design reason:

- Term precedence and formula precedence are separate domains.
- Precedence values range from `0` to `255`; higher values bind more tightly.
- A symbolic functor with no declaration defaults to precedence `64` and
  non-associative grouping.
- Operator declarations affect only later tokens; they are not forward
  declarations, and the target spelling must already be an active functor.
- Parsing chooses grouping from operator metadata; overload resolution later
  chooses the semantic root.
- Conflicting imported metadata for the same visible user symbol is a
  link-time conflict.
- Built-in predicate symbols such as `=`, `<>`, and `in` come from the core
  module and cannot be overridden.
- `qua` is fixed by the language, has the lowest term precedence, and is
  left-associative.

### Frame 4.1b.19 - Term Expressions, Selectors, And `qua`

Grammar sketch:

```ebnf
term_expression ::= operator_expression
                    { "qua" type_expression } ;

term_postfix    ::= "." field_name [ "(" [ term_list ] ")" ]
                  | "with" "(" field_update_list ")" ;

field_update    ::= selector ":=" term_expression ;
```

Example:

```mizar
let x be object;
reconsider y = x qua Element of G;
set C = G.carrier;
set H = G with (carrier := C);
```

Review point:

- Field/property access remains readable.
- `qua` remains source-level syntax because it records an intended view, not a
  failed inference.
- Selector syntax and namespace syntax must be resolved with source spans and
  explanation artifacts.

### Frame 4.1b.20 - Term Primary Forms: Constructors, Sets, And `the`

Grammar sketch:

```ebnf
term_primary      ::= variable_identifier
                    | "it"
                    | numeral
                    | "(" term_expression ")"
                    | struct_constructor
                    | set_expression
                    | choice_expression
                    | inline_functor_application
                    | template_functor_application
                    | bracket_functor_application ;

struct_constructor ::= struct_name [ type_args ]
                       "(" [ field_assignment_list ] ")" ;
inline_functor_application ::= inline_func_name "(" [ term_list ] ")" ;
template_functor_application ::= functor_symbol template_args
                                 [ "(" [ term_list ] ")" ] ;
bracket_functor_application ::= user_symbol term_list user_symbol
                              | "[" term_list "]" ;
set_expression    ::= "{" [ term_list ] "}"
                    | "{" term_expression "where"
                      typed_var_list [ ":" formula ] "}" ;
choice_expression ::= "the" type_expression ;
```

Example:

```mizar
set p = Point(x: 3, y: 4);
set evens = { n where n is Element of NAT : n mod 2 = 0 };
set witness = the Element of X;
```

Review point:

- These are ordinary mathematical terms, not algorithmic runtime constructs.
- `the T` depends on non-emptiness evidence for `T`.
- Set comprehensions are admitted only when the generator range is known to be
  a set.

### Frame 4.1b.21 - `sethood` And Fraenkel Safety

Grammar sketch:

```ebnf
mode_property    ::= "sethood" justification ";" ;
set_comprehension ::= "{" term_expression "where"
                       typed_var_list [ ":" formula ] "}" ;
```

Example:

```mizar
definition
  mode FiniteOrdinalDef: FiniteOrdinal is ordinal finite set;
  sethood by ordinal.finite_ordinal_sethood;
end;

set S = { f where f is FiniteOrdinal : f is non empty };
```

Review point:

- `sethood` proves that all instances of a mode form a set, not a proper class.
- The type checker rejects `{ f(x) where x is T : P(x) }` when no sethood
  witness or element-of-set range is available.
- This keeps the Tarski-Grothendieck set-theoretic boundary visible before ATP
  proof search begins.

### Frame 4.1b.22 - Formula Syntax

Grammar sketch:

```ebnf
formula           ::= quantified_formula | iff_formula ;
universal_formula ::= "for" quantified_vars [ "st" formula ]
                      ( "holds" formula | quantified_formula ) ;
existential_formula ::= "ex" quantified_vars "st" formula ;

iff_formula       ::= implies_formula
                      [ "iff" ( implies_formula | quantified_formula ) ] ;
implies_formula   ::= or_formula
                      [ "implies" ( implies_formula | quantified_formula ) ] ;
or_formula        ::= and_formula
                      { "or" ( and_formula | quantified_formula )
                      | "or" "..." "or"
                        ( and_formula | quantified_formula ) } ;
and_formula       ::= not_formula
                      { "&" ( not_formula | quantified_formula )
                      | "&" "..." "&"
                        ( not_formula | quantified_formula ) } ;
not_formula       ::= "not" ( not_formula | quantified_formula )
                    | atomic_formula
                    | "(" formula ")"
                    | "contradiction"
                    | "thesis" ;

atomic_formula      ::= predicate_application
                      | inline_predicate_application
                      | is_assertion ;
is_assertion        ::= term_expression "is" [ "not" ]
                        is_assertion_body ;
```

Example:

```mizar
for x being Element of G st x is invertible holds
  ex y being Element of G st x * y = unit(G);
```

Review point:

- Mizar-style quantified prose is preserved.
- Attribute assertions and type assertions intentionally share surface syntax
  until resolution classifies them.
- Repeated connective forms such as `& ... &` and `or ... or` are explicit
  formula syntax, not parser accidents.

### Frame 4.1b.23 - Term/Formula Boundary And Formula Precedence

Precedence domains:

| Domain | Source of precedence | Review point |
|---|---|---|
| Terms | user operator metadata plus fixed term syntax | parse before overload |
| Atomic formulas | predicates, equality, membership, `is` assertions | bridge from terms to formulas |
| Formulas | fixed hierarchy for `not`, `&`, `or`, `implies`, `iff`, quantifiers | no user-defined formula precedence |

Example:

```mizar
a + b = c + d
:: atomic formula over two parsed terms

not x > 0 & y > 0
:: (not (x > 0)) & (y > 0)

a or b implies c
:: (a or b) implies c

a iff b iff c
:: error: iff is non-associative
```

Review point:

- Atomic formulas are the boundary between term parsing and formula parsing.
- Predicate-chain notation such as `a < b < c` is resolved at this boundary,
  not as term-operator associativity.
- This makes the grouping explainable in diagnostics and stable for AI edits.

### Frame 4.1b.24 - Theorem Item And Status Syntax

Grammar sketch:

```ebnf
theorem_item   ::= [ theorem_status ] theorem_role
                   label_identifier ":" formula
                   [ justification ] ";" ;

theorem_status ::= "open" | "assumed" | "conditional" ;
theorem_role   ::= "theorem" | "lemma" ;
```

Example:

```mizar
open theorem cancellation_candidate:
  for a, b, c being Element of G st a * b = a * c holds b = c;

assumed lemma choice_profile:
  for X being non empty set holds ex x being object st x in X;
```

Review point:

- `open`, `assumed`, and `conditional` are source-visible because unfinished or
  policy-controlled material must not look like ordinary verified theorem
  material.
- Publication profiles can restrict which statuses are allowed.

### Frame 4.1b.25 - Proof Statement Syntax

Grammar sketch:

```ebnf
proof      ::= "proof" reasoning "end" ;
reasoning  ::= { annotated_statement } ;
statement  ::= [ "then" ] linkable_statement
             | standalone_statement ;

conclusion ::= ( "thus" | "hence" ) proposition
               justification ";" ;
```

Example:

```mizar
proof
  let x be Element of G;
  assume A1: x = unit(G);
  thus thesis by A1, group.unit_left;
end;
```

Preserved proof vocabulary:

- `let`, `assume`, `given`, `consider`, `take`;
- `thus`, `hence`, `thesis`, `by`;
- `now ... end`, `hereby ... end`, `per cases`, `case`, `suppose`;
- `deffunc`, `defpred`, and `reconsider`.

Review point:

- The readable Jaśkowski skeleton is not optional. The new artifacts support
  tools, but the source proof should still read as Mizar mathematics.

### Frame 4.1b.26 - Proof Statement Details

Grammar sketch:

```ebnf
iterative_equality ::= [ label_identifier ":" ]
                       term_expression "=" term_expression
                       simple_justification
                       ".=" term_expression simple_justification
                       { ".=" term_expression simple_justification } ";" ;

statement ::= [ "then" ] linkable_statement
            | standalone_statement ;
```

Example:

```mizar
A1: f.(x + y) = f.x + f.y by Additive
             .= g.x + f.y by A2
             .= g.x + g.y by A3;

assume A2: f = g;
then f.x + f.y = g.x + f.y by FuncEq;
```

Review point:

- `.=` is a justified equality chain, not assignment.
- `then` records dependence on the immediately preceding statement.
- Statement-level `such that` and `and` create labeled assumptions, while
  formula-level `st` and `&` remain inside one formula.

### Frame 4.1b.27 - Citation And Reference Syntax

Grammar sketch:

```ebnf
justification ::= simple_justification | proof | computation_proof ;
simple_justification ::= [ "by" references ] ;

reference ::= label_identifier [ template_args ]
            | qualified_reference [ template_args ]
            | grouped_reference
            | bulk_reference ;

grouped_reference ::= namespace_path ".{"
                      grouped_item { "," grouped_item } "}" ;
bulk_reference    ::= namespace_path ".*" ;
```

Example:

```mizar
thus thesis by group.unit_left, group.{assoc, inverse_left};
then thesis by algebra.group.*;
```

Review point:

- Citation repair remains a small source edit.
- Bulk/grouped forms must be convenient without making dependencies invisible;
  artifacts record the actually used facts.

### Frame 4.1b.28 - Registrations, Clusters, And Reductions

Grammar sketch:

```ebnf
registration_block ::= "registration"
                         { registration_content }
                       "end" ";" ;

registration_item ::= existential_registration
                    | conditional_registration
                    | functorial_registration
                    | reduction_registration ;

conditional_registration ::= "cluster" label ":"
                             antecedent_adjectives "->"
                             consequent_adjectives
                             "for" type_expression ";"
                             "coherence" justification ";" ;
```

Example:

```mizar
registration
  cluster nonempty_group:
    associative unital -> non empty for Magma;
  coherence by group.nonempty_from_unit;
end;
```

Review point:

- Registrations remain a first-class Mizar strength.
- The syntax adds labels and traceability so automatic propagation is
  reviewable and import-filtered.

### Frame 4.1b.29 - Reduction Registration Semantics

Grammar sketch:

```ebnf
reduction_registration ::= "reduce" label ":"
                           term_expression "to" term_expression ";"
                           "reducibility" justification ";" ;
```

Example:

```mizar
registration
  let n be Nat;
  reduce NatAddZero: n + 0 to n;
  reducibility by nat.add_zero;
end;
```

Design reason:

- A reduction is an oriented simplification rule backed by an equality proof.
- The right-hand side must be strictly smaller under the language's
  simplification order, so imported rules cannot create rewrite cycles.
- Normalization is deterministic: rule selection is specificity-first, with an
  FQN tie-break when needed.
- The classic unoriented `identify` idea is not used here; a simplifying
  equivalence must be written as an auditable `reduce`.

### Frame 4.1b.30 - Template Syntax

Grammar sketch:

```ebnf
template_definition ::= definition_block ;
template_parameter_decl ::= definition_parameter_decl ;

let_type ::= "type" [ "extends" bound_type ]
           | type_expression ;

template_item ::= attr_def | mode_def | struct_def
                | pred_def | func_def | algorithm_def
                | theorem_item | registration_item ;

template_args ::= "[" template_arg { "," template_arg } "]" ;
```

Example:

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;
```

Review point:

- Templates are not a cosmetic replacement for `scheme`.
- They are the common syntax for parameterized definitions, theorem schemas,
  structures, registrations, and algorithms.
- `of` and `over` remain readable shorthands; bracket form is canonical for
  identity and tooling.

### Frame 4.1b.31 - Template Constraints And Inference

Grammar sketch:

```ebnf
template_args ::= "[" template_arg { "," template_arg } "]" ;
let_constraint ::= "such" "that" formula ;
let_type       ::= "type" [ "extends" bound_type ]
                 | type_expression ;
```

Example:

```mizar
definition
  let F be Field;
  let V be VectorSpace[F] such that V is finite_dimensional;
  mode BasisDef: Basis[V] is finite linearly_independent Subset of V.carrier;
end;

Product[Ring](s)       :: explicit template argument
Product(s)             :: valid only when the argument type determines Ring
M of A, B              :: greedy shorthand for M[A, B] when arity matches
```

Review point:

- Bracket arguments are canonical because they preserve identity for tools.
- `of` and `over` are kept as readable shorthands, but greedy parsing must be
  deterministic.
- A `such that` constraint is a use-site obligation, not a hidden global
  assumption.

### Frame 4.1b.32 - Annotation Syntax

Example:

```mizar
@show_type(total + x)

@show_resolution
Product[R](s);

@proof_hint(max_axioms: 32, solver: vampire)
thus result = unit(G) by group.identity_unique;

@latex("\\mathbb{Z}")
func IntSetDef: INT -> set equals IntegerSet;
```

Review point:

- Annotations are source syntax, but they must be semantic-neutral unless a
  specific proof-development profile treats them as hints for search.
- They guide editors, documentation, proof search, and AI agents.
- They do not become proof evidence merely by being present in the source.
- Fixed diagnostic annotations such as `@show_type(...)` and
  `@show_resolution` report state; they do not attach logical content to a
  declaration.

### Frame 4.1b.33 - Algorithm Syntax Boundary

Example:

```mizar
definition
  let x, y be Integer;

  algorithm max2(x, y) -> Integer
    ensures (result = x or result = y) & x <= result & y <= result
  do
    if x <= y do
      return y;
    end;
    return x;
  end;
end;
```

Review point:

- Algorithms are part of the language surface, but they are not allowed to
  redefine theorem truth.
- They are declared inside `definition` blocks, and the public interface is the
  signature plus contracts, not the body.
- Contracts, invariants, termination measures, and `by computation` generate
  verification obligations that pass through the same trusted boundary.

### Frame 4.1b.34 - Syntax Migration Review Questions

Questions for Bialystok:

- Which legacy Mizar constructs must keep near-identical syntax for cultural
  and migration reasons?
- Which constructs may become more explicit because the current implicit form
  hides dependencies?
- Are `field` / `property` / `attribute` distinctions readable enough in real
  algebraic examples?
- Is one-parent-per-`inherit` acceptable for multiple-inheritance-heavy MML
  fragments?
- Do the explicit operator-precedence rules match the expectations of existing
  Mizar notation users?
- Are `sethood` checks visible enough for Fraenkel-style set formation without
  making ordinary set notation too heavy?
- Which predicate/functor properties should be migrated first because existing
  proofs rely on them implicitly?
- Is oriented `reduce` a readable replacement for implicit simplification and
  older identification idioms?
- Are template brackets acceptable as canonical identity syntax if `of` and
  `over` remain source shorthands?
- Where should template inference stop and require explicit `[T]` arguments or
  `qua` views?
- Which proof statement forms are missing from this surface review?

### Frame 4.1c.01 - Detail: Article Environment

- Current: `environ` gives each article its vocabulary, notation,
  constructors, registrations, requirements, and visible theorem base.
- Evo: the same roles must become explicit imports, prelude policy, package
  metadata, and reproducible dependency reports.
- Review question: which current `environ` roles must remain visually familiar
  during migration, and which can be replaced by generated reports?

### Frame 4.1c.02 - Detail: Module Interfaces

- Current: article acceptance and MML inclusion determine what later articles
  can use, but the source file has limited public/private interface syntax.
- Evo: `export`, `public`, `private`, opaque imports, and separate compilation
  define the reusable API surface.
- Review question: which legacy article facts should become public module API,
  and which should become private proof support?

### Frame 4.1c.03 - Detail: Symbol Identity

- Current: theorem and constructor identity is tied to article names, labels,
  and MML inclusion history.
- Evo: path-derived FQNs and module-qualified labels make identity independent
  of the local import spelling.
- Review question: what identity metadata is needed to preserve old citations
  while allowing package/module reorganization?

### Frame 4.1c.04 - Detail: Symbolic Notation And Aliases

- Current: Mizar supports rich mathematical notation, including synonyms and
  antonyms for alternative phrasing and natural negation.
- Evo: arbitrary operator symbols are concentrated in `func` and `pred`, while
  mode, attribute, and structure names remain identifier-like; synonym and
  antonym equivalence is preserved.
- Review question: which legacy symbolic constructors need compatibility
  spelling, and which should migrate to clearer constructor names?

### Frame 4.1c.05 - Detail: Lexical Activation

- Current: an article environment determines which symbols and notations are
  available, with many effects hidden from local text.
- Evo: the import prelude is pre-scanned, the active lexicon is source-position
  dependent, and declarations become active only after their item is complete.
- Review question: where will migration need generated diagnostics for
  no-forward-reference or dot/operator disambiguation changes?

### Frame 4.1c.06 - Detail: Soft Type Foundation

- Current: Mizar's soft type system is a central identity feature, layered over
  set-theoretic objects.
- Evo: soft typing is preserved while `object`/`set`, radix and mode heads,
  type erasure, widening/narrowing, and `reconsider` obligations are exposed.
- Review question: which type obligations should be visible in source, and
  which should remain verifier metadata?

### Frame 4.1c.07 - Detail: Structures

- Current: structure declarations combine parent structure, fields, selectors,
  and constructor layout in compact legacy syntax.
- Evo: `struct`, `field`, `property`, and `inherit` separate layout, canonical
  derived data, and inherited views.
- Review question: when migrating a legacy selector, is it intrinsic structure
  data, a derived property, or an inherited view?

### Frame 4.1c.08 - Detail: Inheritance

- Current: inheritance is powerful but inherited field mapping is often read
  through structure syntax and naming convention.
- Evo: each `inherit` statement records one parent relation, including mapping,
  renaming, narrowing, and coherence evidence.
- Review question: which algebraic examples best expose diamond inheritance and
  coherence obligations without overwhelming the audience?

### Frame 4.1c.09 - Detail: Modes And Attributes

- Current: modes and adjectives make Mizar text close to mathematical prose.
- Evo: modes remain type abbreviations/refinements, while attributes are
  explicit type-refining predicates used by clustering.
- Review question: which current adjectives are cultural vocabulary that should
  be preserved exactly?

### Frame 4.1c.10 - Detail: Definitions And Correctness

- Current: `func` and `pred` definitions use familiar `equals` and `means`
  styles with correctness obligations.
- Evo: these styles remain, but existence, uniqueness, coherence,
  compatibility, and `assume` side conditions become explicit obligations and
  artifact entries.
- Review question: which correctness obligations should be source-visible in a
  migration diff?

### Frame 4.1c.11 - Detail: Overload And `redefine`

- Current: overloaded symbols and redefinitions support compact mathematical
  notation over many related types.
- Evo: ordinary overload roots, same-root `redefine` families, `coherence with`,
  and refinement joins are separated and recorded.
- Review question: how should tools explain a changed overload choice to a user
  who only sees familiar mathematical notation?

### Frame 4.1c.12 - Detail: Registrations, Clusters, And Reductions

- Current: registrations and clusters are one of Mizar's major automation
  strengths, but their local effect can be hard to inspect.
- Evo: labeled registration items, import-filtered cluster graphs, reduction
  indexes, and replayable traces make propagation and rewriting explainable.
- Review question: which cluster explanations are essential for trusting a
  migrated article?

### Frame 4.1c.13 - Detail: Schemes And Generics

- Current: classical `scheme` and `of`/`over` parameterization cover many
  reusable reasoning patterns.
- Evo: first-class templates unify parameterized definitions, structures,
  attributes, functors, predicates, algorithms, and theorem schemas.
- Review question: which existing schemes should be first migration targets for
  template-style presentation?

### Frame 4.1c.14 - Detail: Declarative Proof Skeleton

- Current: Jaśkowski-style proof text with `let`, `assume`, `thus`, `hence`,
  `thesis`, and diffuse reasoning is central to Mizar readability.
- Evo: the readable skeleton is preserved while extracted obligations, thesis
  states, and source spans are emitted as metadata.
- Review question: how much proof-state metadata helps users without making the
  proof look tactic-driven?

### Frame 4.1c.15 - Detail: Proof Status

- Current: the normal publication unit is a verified theorem item.
- Evo: `open`, `assumed`, and `conditional` distinguish unsettled propositions,
  deliberate assumptions, and results depending on non-clean material.
- Review question: what policy is acceptable for non-clean items in published
  or package-distributed material?

### Frame 4.1c.16 - Detail: Proof Citations

- Current: `by` citations name visible facts and keep proof steps compact.
- Evo: grouped citations, bulk citations, used-axiom recording, and citation
  refinement support small verifier-checked repairs.
- Review question: when should a migration tool minimize citations, and when
  should it preserve the author's original citation style?

### Frame 4.1c.17 - Detail: Type Views

- Current: `qua`, widening, narrowing, and local disambiguation help users
  select intended type views.
- Evo: source-level `qua` is preserved, while inserted coercions, selected
  views, and overload metadata become inspectable artifacts.
- Review question: which implicit choices should be shown in source during
  migration, and which should remain hover/explanation data?

### Frame 4.1c.18 - Detail: Algorithms

- Current: Mizar is primarily a mathematical proof language, not a general
  verified programming surface.
- Evo: `algorithm`, contracts, invariants, termination measures, MVM execution,
  and `by computation` connect verified computation to proof.
- Review question: which computational examples would demonstrate value without
  distracting from the mathematical proof language?

### Frame 4.1c.19 - Detail: Annotations

- Current: comments and informal tool conventions can help readers but do not
  form a stable verifier-facing interface.
- Evo: semantic-neutral annotations guide proof search, display inferred state,
  and render notation without changing theorem truth.
- Review question: which annotations are useful enough to standardize, and
  which should stay in external tooling?

### Frame 4.1c.20 - Detail: Diagnostics And LSP

- Current: verifier output is primarily human-facing text.
- Evo: stable diagnostic codes, primary and secondary spans, fix suggestions,
  lazy explanation artifacts, and LSP records make errors actionable.
- Review question: which diagnostic explanations would most reduce migration
  friction for current MML maintainers?

### Frame 4.1c.21 - Detail: ATP Boundary

- Current: proof automation is valuable, but the trusted boundary is not
  presented as a portable artifact protocol.
- Evo: ATPs search, certificate artifacts record evidence, and the kernel
  independently accepts or rejects that evidence.
- Review question: which certificate failures should be user-facing, and which
  are implementation/debugging details?

### Frame 4.1c.22 - Detail: Packages And Dependency Resolution

- Current: MML is organized around articles and library inclusion.
- Evo: `mizar.pkg`, lockfiles, SemVer, features, compatibility checks, and
  cached artifacts support reproducible package-oriented development.
- Review question: where should the standard library, published articles,
  third-party packages, and local experiments sit in the namespace policy?

### Frame 4.1c.23 - Detail: Incremental Verification

- Current: accepted article output is the main reuse boundary.
- Evo: dependency slices, VC anchors, witness hashes, and cache keys allow
  reuse, but cache reuse is never proof authority.
- Review question: what clean-build equivalence tests would make incremental
  verification trustworthy?

### Frame 4.1c.24 - Detail: Documentation Generation

- Current: source comments, MML browsing, and Formalized Mathematics pages
  serve related but distinct reading needs.
- Evo: `mizar doc` uses verified artifacts, labels, FQNs, documentation
  comments, and `@latex` annotations to produce API documentation.
- Review question: which documentation warnings should be optional lint, and
  which should block publication profiles?

### Frame 4.1c.25 - Detail: Code Extraction

- Current: there is no general source-level workflow for extracting verified
  executable code from Mizar articles.
- Evo: terminating algorithms, ghost erasure, target-neutral runtime IR, and
  extractor configuration make executable output a downstream artifact.
- Review question: which target languages and mathematical domains are the
  right first extraction benchmarks?

### Frame 4.1c.26 - Detail: Formalized Mathematics Link

- Current: article identity, publication exposition, and library reuse are
  closely connected.
- Evo: publication metadata links articles to reusable library modules,
  package versions, and stable theorem identities.
- Review question: how can Evo preserve citation value while letting the
  reusable library evolve independently?

### Frame 4.2 - Current Article Environment, Exact Excerpt

Exact current MML excerpt:

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
- The point is not that `environ` is bad. The point is that dependency classes
  are article-level and need translation into explicit module/package data.

### Frame 4.3 - Mizar Evo Import Prelude

```mizar
import std.algebra.structure.sorted;
import std.function.binop;
import std.algebra.structure.magma;
```

Main differences:

- file-scoped import prelude;
- package/module namespace;
- import closure as a build-system input;
- stable FQNs derived from path and module.

Design reason:

- The prelude is deliberately file-scoped so lexing, disambiguation, and AI
  context extraction can build one stable imported base environment before
  reading the body.
- This makes dependency reports reproducible: a tool can say which import
  provided each symbol, notation, registration, and theorem.

### Frame 4.4 - Migration Map: Environment To Imports

| Current Mizar role | Evo target |
|---|---|
| vocabularies | exported symbols and lexical metadata |
| notations | imported notation metadata |
| constructors | visible definitions and constructors |
| registrations | import-scoped registration index |
| requirements | package or prelude policy |
| article labels | module-qualified theorem identities |

### Frame 4.5 - Why This Is Not A Mechanical Rename

Bullets:

- Current environments mix several dependency roles.
- Some dependencies are semantic, some syntactic, some automation-facing.
- Migration needs reports explaining what each imported module contributes.
- The purpose of the new boundary is not cosmetic modernization. It is to make
  old implicit roles reviewable before they become package and artifact
  dependencies.

### Frame 4.6 - Legacy Structure Example: Exact Excerpt

Current plain-text `ALGSTR_0` exposes the shape:

```mizar
definition
  struct (1-sorted) addMagma (# carrier -> set, addF -> BinOp of the carrier
  #);
end;
```

Speaker note:

- Source: current MML `algstr_0.miz`, lines 37-40.
- This is a short exact excerpt; the Beamer slide can use the same snippet with
  attribution in speaker notes.

### Frame 4.7 - Evo Structure Example

```mizar
definition
  struct AddMagma where
    field carrier -> set;
    field add -> BinOp of carrier;
  end;

  struct AddLoopStr where
    field carrier -> non empty set;
    field add -> BinOp of carrier;
    property zero -> Element of carrier;
  end;

  inherit AddMagma extends 1-sorted;
end;
```

Claim:

- Evo makes field declarations and inheritance obligations explicit, while
  keeping the concept close to Mizar structures.
- Evo also separates `field` from `property`: fields are independent structure
  components, while properties are uniquely determined mathematical data or
  obligations associated with the structure.

Design reason:

- Structure inheritance remains useful only if users can see which fields are
  shared, renamed, or required by a parent structure.
- Making those obligations explicit turns a migration risk into something the
  verifier, LSP, and reviewers can discuss.
- Separating fields from properties prevents constructor layout, selector
  identity, and mathematical laws from being treated as the same kind of
  dependency.

### Frame 4.7a - Why Separate Field And Property?

Design distinction:

| Concept | What it means | Why it is separate |
|---|---|---|
| `field` | intrinsic component supplied by the structure value | affects constructor shape, selector identity, extensional equality, and storage-like artifacts |
| `property` | uniquely determined value or obligation attached to the structure | affects semantic obligations, coherence, inheritance constraints, and proof/artifact fingerprints |
| `attribute` | predicate-style refinement such as `empty`, `trivial`, or `add-cancelable` | participates in registration and cluster propagation rather than structure layout |

Message:

- A carrier or operation is a field because changing it changes the object.
- A zero, unit, degree, or similar canonical value may be a property when it is
  determined by the mathematical structure and needs coherence evidence.
- An attribute is neither stored data nor a selector; it is a reusable logical
  refinement that automation can propagate.
- This split gives migration tools a better question to ask: "is this legacy
  item part of the structure's data, a canonical value with proof obligations,
  or a propagated predicate?"

Speaker note:

- Source design vocabulary: `doc/spec/en/05.structures.md` distinguishes
  `field` from `property`; `doc/spec/en/06.attributes.md` defines attributes as
  type-refining predicates used by clustering.

### Frame 4.7b - Why Separate `struct` And `inherit`?

Design distinction:

| Declaration | What it owns | Why it is separate |
|---|---|---|
| `struct` | the local type constructor, fields, and properties | defines the object's own layout and selectors |
| `inherit` | the relation to one parent structure | records field/property mapping, renaming, narrowing, and coherence evidence |

Message:

- A structure declaration should answer "what is this object made of?"
- An inheritance declaration should answer "how is this object viewed as that
  parent?"
- Keeping them separate makes multiple inheritance explicit: each parent gets
  one auditable `inherit` statement instead of one large implicit merge.
- Diamond inheritance then becomes a diagnostic problem with source spans and
  coherence obligations, not a hidden side effect of declaration order.
- It also gives artifacts cleaner fingerprints: changing a field layout is not
  the same event as changing an inherited view or its coherence proof.

Speaker note:

- Source design vocabulary: `doc/spec/en/05.structures.md` requires one parent
  per `inherit` statement and uses explicit `where` blocks for renaming,
  narrowing, and coherence.

### Frame 4.8 - Renaming And Views

```mizar
definition
  struct Magma where
    field carrier -> set;
    field binop -> BinOp of carrier;
  end;

  inherit AddMagma extends Magma where
    field carrier from carrier;
    field add from binop;
  end;
end;
```

Purpose:

- Support library organization where additive and multiplicative views share
  structure without relying on hidden informal naming conventions.
- Preserve the mathematical habit of reusing the same carrier through different
  views, while recording the view translation as checked source.

### Frame 4.9 - Diamond Inheritance As A Diagnostic Opportunity

```mizar
inherit DoubleLoopStr extends AddLoopStr;
inherit DoubleLoopStr extends MulLoopStr;
```

Message:

- The system should explain when two inheritance paths agree, conflict, or need
  a proof of coherence.
- This explanation should be available to users, LSP, and AI agents.
- The design goal is to keep multiple-view algebra usable without letting
  hidden inheritance order decide semantics.

### Frame 4.10 - Registrations And Clusters

Exact current MML excerpt:

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
- The excerpt is intentionally short: it shows attribute propagation without
  making the slide depend on a long registration proof.

Evo interpretation:

- registration is still a mathematical mechanism;
- the implementation stores an import-filtered, explainable registration graph;
- every automatic attribute propagation should be traceable.

Design reason:

- Registrations and clusters are one of Mizar's strengths, so Evo should not
  remove them.
- The change is that automatic propagation must leave a trace that can be
  audited, cached, and shown to an AI agent without dumping the whole library.

### Frame 4.11 - Overload Resolution And `qua`

```mizar
consider p be Product(R qua MulStr) such that
  ...
```

Message:

- Explicit disambiguation is not a failure of inference.
- It is a readable record of the intended mathematical view.
- It gives AI agents a small safe repair target.

### Frame 4.12 - Proof Citation Repair

Before:

```mizar
thus thesis by A;
```

After:

```mizar
thus thesis by A, B, C;
```

Exact current MML proof-citation example:

```mizar
theorem
  for F being non degenerated ZeroOneStr holds 1.F in NonZero F
proof
  let F be non degenerated ZeroOneStr;
  not 1.F in {0.F} by TARSKI:def 1;
  hence thesis by XBOOLE_0:def 5;
end;
```

Speaker note:

- Source: current MML `struct_0.miz`, lines 637-643.
- Use this to explain citation repair as a bounded local edit: an agent may
  propose a missing or more precise citation, but the verifier and kernel
  evidence decide acceptance.

Message:

- This is a Green AI edit: source-local, verifier-checked, meaning-preserving if
  accepted with respect to the theorem statement.
- A later refinement pass can minimize citations based on used axioms recorded
  in artifacts.
- The added dependency still matters; artifacts should record it so a later
  `mizar refine`-style pass can reduce noise.

### Frame 4.13 - Forbidden Repair

Bad AI repair:

```mizar
theorem
  for x be Nat holds x + 0 = x or x = 0;
```

Message:

- Weakening the theorem statement is a Red edit.
- It may make the proof easier but destroys the intended result.
- Ordinary AI agents must not be authorized to apply it; at most they may flag
  that such a change would require explicit human unsafe-edit review.

### Frame 4.14 - Annotations

```mizar
@show_type(total + x)

@show_resolution
Product[R](s);

@proof_hint(max_axioms: 32, solver: vampire)
thus result = unit(G) by group.identity_unique;
```

Message:

- Annotations are structured context for humans, tools, and AI.
- Informational annotations should not change logical meaning.
- Hints that influence search must still be treated as proof-development
  metadata, not as accepted evidence.

### Frame 4.14a - Templates

Proposed Evo sketch:

```mizar
definition
  let T be type;
  struct MagmaStr[T] where
    field carrier -> T;
    field binop -> BinOp of T;
  end;
end;

definition
  let M be type extends commutative Magma;
  theorem PermProduct[M]:
    ...
end;
```

Message:

- Templates are the generics mechanism for Mizar Evo.
- They parameterize definitions and theorem schemas over types, predicates, or
  functors.
- Classical scheme-style reasoning becomes part of one template vocabulary
  rather than a separate special case.
- `of` / `over` forms remain readable shorthands; the bracket form records the
  canonical parameterization.

Design reason:

- Templates avoid copying the same algebraic construction for every carrier,
  field, or predicate instance.
- They make generic mathematical patterns explicit enough for dependency
  fingerprints, theorem indexes, and AI retrieval.
- Constraints such as `type extends commutative Magma` say what the
  parameter must provide before ATP or proof search begins.

Speaker note:

- Source design vocabulary: `doc/spec/en/18.templates.md` defines templates as
  generics for parameterized definitions and theorem schemas, including
  structures, attributes, modes, predicates, functors, and schemes.

### Frame 4.15 - Algorithm Verification

Potential Evo sketch:

```mizar
definition
  let x, y be Integer;

  algorithm max2(x, y) -> Integer
    ensures (result = x or result = y) & x <= result & y <= result
  do
    if x <= y do
      return y;
    end;
    return x;
  end;
end;
```

Message:

- Algorithms broaden the scope beyond pure mathematics.
- Contracts and generated VCs must still feed the same verification pipeline.

### Frame 4.16 - Compatibility Metadata

```mizar
@[origin("mizar:ALGSTR_0:addMagma")]
definition
  struct AddMagma where
    ...
  end;
end;
```

Message:

- Migration must preserve historical identity.
- Origin metadata supports Formalized Mathematics links, citation stability,
  and review.

### Frame 4.17 - What Needs Bialystok Input

Questions:

- Which legacy constructs require direct syntax preservation?
- Which constructs should be translated into clearer Evo forms?
- What origin identifiers are stable enough for MML migration?
- Which current environment dependencies are most difficult to explain?

## Part 5. Architecture

### Frame 5.1 - Pipeline Overview

```text
Source
  -> TokenStream
  -> SurfaceAst
  -> ResolvedAst
  -> TypedAst
  -> CoreIr
  -> VcIr
  -> AtpProblem
  -> ProofCertificate
  -> VerifiedArtifact
```

Message:

- Each layer separates a different responsibility.
- The separation is for diagnostics, caching, trust, and reproducibility.
- The point is not a generic compiler diagram. Each boundary says who owns a
  fact, which artifact records it, and what must be recomputed when it changes.

### Frame 5.2 - Responsibility Split

| Layer | Responsibility |
|---|---|
| frontend | source, lexing, parsing, recovery |
| resolver | imports, names, labels, namespaces |
| checker | soft types, clusters, registrations, overloads |
| elaborator | core logical representation |
| VC generator | proof and algorithm obligations |
| ATP | search for evidence |
| kernel | acceptance by replay/checking |
| artifact emitter | stable outputs for tools and dependents |

### Frame 5.3 - Reasoning Boundary

```text
Mizar-side semantics
  -> well-typed, resolved obligations
ATP-side search
  -> candidate proof evidence
kernel-side checking
  -> accepted or rejected proof status
```

Key point:

- ATPs do not resolve names, infer types, expand clusters, or decide overloads.
- The kernel does not search for proofs.
- Deterministic pre-ATP discharge also needs replayable evidence or an explicit
  policy status; it is not accepted just because an earlier phase said "done".

Design reason:

- Putting all reasoning inside Mizar would make the trusted verifier larger.
- Delegating all reasoning to ATPs would lose control of Mizar-specific
  semantics.
- The hybrid boundary keeps semantic processing deterministic, ATP search
  powerful, and final acceptance independently checkable.

Status distinction:

| Status | Meaning |
|---|---|
| `kernel_verified` | replayed or certificate-checked evidence accepted |
| `discharged_builtin` | deterministic discharge accepted by the same evidence discipline |
| `externally_attested` | recorded backend or policy attestation, not equivalent to verified |
| `open` / policy status | unfinished or explicitly policy-controlled, not silently promoted |

### Frame 5.4 - SAT-Based Small Kernel

Explanation:

- High-level proof search may use ATPs.
- Accepted evidence is normalized into certificate data.
- The kernel checks imported facts, substitutions, alpha-conversion, clause
  well-formedness, and resolution/SAT evidence.
- Unsoundness in search should not become unsoundness in accepted results.
- The SAT part is proof evidence checking, not trust in a solver's success bit.
  Backend-specific proofs must be translated into replayable certificate data
  before acceptance.

### Frame 5.5 - Certificate Data

```text
Certificate
  target VC fingerprint
  kernel profile
  imported facts and hashes
  generated clauses
  substitutions
  resolution trace
  final goal
```

Message:

- The certificate binds a proof to a specific obligation and dependency slice.
- It must be deterministic and replayable.
- This is why the certificate carries hashes and a kernel profile: a proof
  accepted under one dependency and policy context should not silently migrate
  to another.

### Frame 5.6 - What The Kernel Must Not Do

Bullets:

- no proof search;
- no premise selection;
- no name resolution;
- no overload resolution;
- no cluster expansion;
- no hidden global state;
- no trust in raw ATP success.

### Frame 5.7 - Cluster And Registration Architecture

Message:

- Mizar's registrations are a strength but need traceability at scale.
- Evo stores registration and cluster data in indexes.
- An active module gets an import-filtered view.
- Explanation artifacts expose why a fact was or was not derived.

### Frame 5.8 - Memory Model

```text
resident memory should scale with:
  active source
  imported public interfaces
  import-filtered indexes
  active module VCs

not with:
  imported proof bodies
  private lemmas outside the interface
  unused registration data outside the import closure
```

### Frame 5.9 - Incremental Verification

Main points:

- Hash source, dependency interfaces, generated VCs, and proof witnesses.
- Reuse only when fingerprints and verifier policy match.
- Proof-body-only changes should not rebuild unrelated importers when public
  statements and statuses remain unchanged.

### Frame 5.10 - Parallel Verification

Main points:

- Parallelize independent modules, obligations, ATP runs, and kernel checks.
- Publish diagnostics, artifacts, and proof statuses in canonical order.
- Runtime completion order must not determine semantics.

### Frame 5.11 - Artifact Model

Artifacts should serve:

- downstream package verification;
- IDE hover, diagnostics, and go-to-definition;
- AI context extraction and patch verification;
- documentation generation;
- Formalized Mathematics article links;
- reproducible build records.

### Frame 5.12 - LSP And MCP

Message:

- LSP gives editor feedback from syntax and semantic artifacts.
- Planned MCP-style resources and tools expose bounded verifier context to AI
  agents.
- Both should consume artifacts rather than reimplementing the verifier.
- The wire protocol is not the design center here; the design center is that
  agents read bounded, typed, auditable context and cannot bypass verification.
- The current fixed policy is the Green/Yellow/Red edit classification and
  authorization scopes; detailed wire schemas remain roadmap material.

### Frame 5.13 - Safe AI Edit Classes

| Class | Examples | Policy |
|---|---|---|
| Green | add citation, insert `qua`, add info annotation | may be auto-proposed, still verified |
| Yellow | add import, local lemma, registration, invariant | proposed with human review |
| Red | add axiom, weaken theorem, change definition, relax kernel | forbidden to ordinary AI agents |

### Frame 5.14 - Test Strategy

Main principle:

```text
Reject what must not pass
before
Accept everything that should pass
```

Talk angle:

- Parser gaps are annoying.
- Soundness bugs are dangerous.
- Kernel-adjacent tests should emphasize malformed and failing inputs.
- The test mix should therefore be intentionally asymmetric near the trust
  boundary: reject invalid evidence first, then broaden accepted-language
  coverage.

### Frame 5.15 - Architecture Questions

Questions:

- Is the SAT certificate story convincing for Mizar-style proofs?
- Which proof evidence format would be easiest for the Mizar team to audit?
- Which artifact should be built first for real migration experiments?

## Part 6. MML Migration Roadmap

### Frame 6.1 - Roadmap Thesis

Slide text:

```text
MML migration is a research program, not a one-shot translation.
```

### Frame 6.2 - September 2026 Goal

Outputs from the visit:

- a prioritized list of migration benchmark articles;
- agreement on compatibility metadata needs;
- review notes on language changes;
- a first paper outline;
- a shared list of objections and risks.

### Frame 6.3 - End Of 2026 Alpha Target

Possible alpha scope:

- frontend and parser for a meaningful core subset;
- import prelude and module resolution prototype;
- structured AST and diagnostics;
- basic type and registration scaffolding;
- test harness and corpus layout;
- early artifact format for parsed/resolved source.

Non-goals:

- full MML verification;
- final compatibility layer;
- stable AI protocol;
- complete Formalized Mathematics workflow.

### Frame 6.4 - 2027 Migration Laboratory

Focus:

- choose 3-5 representative MML articles;
- translate by hand and with scripts;
- record every mismatch as a classified issue;
- build side-by-side reports.

Suggested issue classes:

- syntax translation;
- environment-to-import mapping;
- structure inheritance;
- registration or cluster behavior;
- overload behavior;
- proof citation or ATP behavior;
- documentation and origin metadata.

### Frame 6.5 - 2027-2028 Library Expansion

Candidate path:

1. foundational set and relation fragments;
2. functions and binary operations;
3. algebraic structures;
4. selected theorem-heavy articles;
5. broader dependency cones around successful fragments.

### Frame 6.6 - Migration Metrics

Measure:

- translated LOC and article count;
- accepted parser subset;
- resolved imports versus unresolved dependencies;
- proof obligations generated;
- obligations closed by deterministic reasoning;
- obligations closed by ATP plus kernel certificate;
- memory and build time per module;
- number of compatibility decisions needing human review.

### Frame 6.7 - Compatibility Policy

Possible policy:

- preserve theorem identity through origin metadata;
- keep source-visible compatibility aliases where they help migration;
- avoid preserving legacy forms that block clear module and artifact semantics;
- record every divergence from old behavior with a reason and test.
- Compatibility should be argued from mathematical identity, review value, and
  reproducibility, not from nostalgia for every surface form.

### Frame 6.8 - Migration Risk Table

| Risk | Mitigation |
|---|---|
| syntax compatibility consumes the project | start with representative slices, not full automation |
| registrations behave differently | build trace and explanation artifacts early |
| proof search differs from current verifier | record used facts and compare proof obligations |
| package layout breaks Formalized Mathematics links | origin metadata and article-to-library identifiers |
| AI edits hide migration mistakes | keep Red edits forbidden and require verifier artifacts |

### Frame 6.9 - Bialystok Input Needed

Questions:

- Which MML articles are small but structurally representative?
- Which article families stress registrations and structures?
- Which current Mizar idioms are culturally important, not just technically
  convenient?
- What migration result would convince the community that Evo is serious?

## Part 7. Formalized Mathematics

### Frame 7.1 - Publication Thesis

Slide text:

```text
The verified library and the published article should be linked but not forced
to have the same structure.
```

### Frame 7.2 - Why Separate Library And Article?

Library modules optimize:

- reuse;
- dependency control;
- package versioning;
- stable machine-readable artifacts.

Formalized Mathematics articles optimize:

- exposition;
- narrative;
- citation;
- peer review;
- reader-facing mathematical structure.

### Frame 7.3 - Link Model

```text
Formalized Mathematics theorem
  -> library package
  -> module path
  -> theorem FQN
  -> statement fingerprint
  -> verified artifact hash
  -> rendered documentation page
```

### Frame 7.4 - Example Link Record

```text
article: Formalized Mathematics, future volume
section: Unique identity element
library_object: std.algebra.group.identity_unique
package: std_algebra
version: 0.4.0
statement_hash: sha256:...
artifact_hash: sha256:...
origin: mizar:GROUP_1:...
```

Design reason:

- These identities are deliberately layered.
- The article label supports scholarly citation and narrative structure.
- The origin id preserves continuity with MML and existing Formalized
  Mathematics history.
- The library FQN addresses the current reusable theorem object.
- The statement fingerprint detects semantic drift.
- The artifact hash supports reproducible verification of the exact build.

### Frame 7.5 - Benefits

For readers:

- article prose remains primary;
- formal source is one click away;
- proof status and dependency information are inspectable.

For maintainers:

- library refactoring does not automatically rewrite article exposition;
- versioned links preserve reproducibility;
- origin metadata preserves historical continuity.

For AI:

- article prose provides semantic search text;
- formal links provide exact theorem identities;
- agents can retrieve both explanation and machine-checkable context.

### Frame 7.6 - Open Formalized Mathematics Questions

Questions:

- Should stable identity be based on library FQN, article label, origin id, or
  artifact hash?
- Which of these identities should be primary in user-facing citations, and
  which should remain machine-facing reproducibility checks?
- How should article revisions track library refactoring?
- Should Formalized Mathematics publish generated HTML, PDF, or both?
- How should old Formalized Mathematics articles link to migrated library
  modules?

### Frame 7.7 - Possible Workflow

```text
library theorem verifies
  -> verified artifact emitted
  -> documentation generated
  -> article references formal objects
  -> publication build checks links and versions
  -> article published with formal links
```

## Part 8. Discussion And Requests

### Frame 8.1 - What We Ask The Bialystok Team To Review

Bullets:

- language identity and compatibility;
- MML migration examples;
- registration and cluster semantics;
- proof evidence and kernel boundary;
- Formalized Mathematics link model;
- roadmap realism.

### Frame 8.2 - Concrete Next Steps

Proposed next steps:

1. Select migration benchmark articles.
2. Prepare exact old/new code comparison slides.
3. Write an initial MML migration report template.
4. Refine this detailed Beamer deck after Bialystok review.
5. Start the paper outline using the same main claims and review feedback.

### Frame 8.3 - Closing

Final slide:

```text
Mizar Evo should be modern where scale demands it,
and conservative where Mizar's mathematical identity depends on it.
```

## Backup A. Prepared Exact Examples

Prepared short excerpts for Beamer conversion:

| Purpose | Source | Lines | Migration point | Evo sketch |
|---|---|---:|---|---|
| Article environment | `algstr_0.miz` | 15-25 | Translate article-level dependency classes into explicit import/package roles. | `import std.function.binop; import std.algebra.structure.sorted;` |
| Structure definition | `algstr_0.miz` | 37-40 | Preserve structures while separating intrinsic fields from canonical properties and inheritance obligations. | `field carrier`, `field add`, and, where appropriate, `property zero` / `property unit` |
| Registration/cluster propagation | `algstr_0.miz` | 104-109 | Keep registrations, but make propagation traceable and import-filtered. | registration graph entry plus resolution trace artifact |
| Proof citation | `struct_0.miz` | 637-643 | Treat citation repair as a small verifier-checked edit. | Green edit: add or refine `by` citations; artifact records used facts |
| Publication link | `contents.html` | 516-517 | Link scholarly article metadata to a reusable MML/library identifier. | Formalized Mathematics article label + `origin: mizar:GROUP_1` + theorem FQN/fingerprint |

Source URLs:

- `ALGSTR_0`: <https://mizar.uwb.edu.pl/version/current/mml/algstr_0.miz>
- `STRUCT_0`: <https://mizar.uwb.edu.pl/version/current/mml/struct_0.miz>
- Formalized Mathematics contents:
  <https://mizar.uwb.edu.pl/fm/contents.html>

Remaining Beamer preparation:

- Decide whether the visible slide should show the raw source URL, a shortened
  citation label, or both.
- Add a final attribution note for MML source licensing.
- If space is tight, move exact source URLs to speaker notes and keep only
  article names and line numbers on slides.

## Backup B. Diagram List

Required diagrams:

1. Current Mizar workflow: source -> Accommodator -> Verifier -> Exporter -> MML.
2. Three pillars: readability, AI-readiness, scalability.
3. Environment-to-import migration.
4. Structure inheritance and diamond coherence.
5. Full compiler/verifier pipeline.
6. Reasoning boundary: Mizar semantics / ATP search / kernel checking.
7. Certificate object and SAT/UNSAT checking.
8. Incremental dependency and fingerprint graph.
9. AI patch verification flow.
10. Formalized Mathematics article-to-library link model.
11. Roadmap timeline.

## Backup C. Paper Outline Seed

Possible paper title:

```text
Mizar Evo: Readable, AI-Ready, and Scalable Formal Mathematics
```

Possible sections:

1. Introduction: why Mizar needs evolution now.
2. Mizar as baseline: readability, MML, Formalized Mathematics.
3. Design principles.
4. Language evolution and compatibility.
5. Verifier architecture and small kernel.
6. AI-safe proof development.
7. Package-based library and publication workflow.
8. Migration plan and evaluation metrics.
9. Related work: current Mizar, Lean, Isabelle/Isar, ATP-integrated systems.
10. Conclusion and collaboration agenda.

## Backup D. Reviewer Checklist

Use this checklist before converting to Beamer:

- Does every criticism of current practice also acknowledge why that practice
  was useful?
- Does every Lean comparison avoid sounding dismissive?
- Does every architecture claim map to an existing or planned artifact?
- Does every code example say whether it is exact, schematic, or proposed?
- Do structure slides explain why `field`, `property`, and `attribute` are
  distinct?
- Do inheritance slides explain why `struct` and `inherit` are separate
  declarations?
- Do template slides explain why generic definitions and theorem schemas need a
  first-class mechanism rather than copy-paste or ad hoc schemes?
- Are Red AI edits clearly forbidden?
- Are MML migration claims measurable?
- Does the Formalized Mathematics section preserve the value of publication, not only library
  indexing?
