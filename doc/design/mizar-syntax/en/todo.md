# mizar-syntax TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

| Module | Spec | Source | Status |
|---|---|---|---|
| ast | [ast.md](./ast.md) | `src/ast.rs` | [~] rowan storage boundary in place; vocabulary still growing |
| trivia | [trivia.md](./trivia.md) | `src/trivia.rs` | [x] task-4 model implemented; task-5 item attachment fixture landed |
| recovery | [recovery.md](./recovery.md) | `src/recovery.rs` | [x] task-5 recovery vocabulary implemented; parser producers remain incremental |

`mizar-syntax` is a data-definition crate: it owns the `SurfaceAst` shape shared
by `mizar-parser`, `mizar-frontend`, and future resolver/LSP/formatter
consumers, and it owns no parsing logic and no semantics. The target syntax-tree
backend is `rowan`: the early representation work must adopt a rowan-backed
green tree and may keep only compatibility wrappers needed to preserve the
current minimal boundary during the migration. It is built in two waves: first
the representation foundation (rowan storage boundary, rendering, trivia,
recovery vocabulary), then the node vocabulary, which grows in lockstep with
`mizar-parser` grammar tasks.

Dependency order: `ast` foundation → `trivia` / `recovery` → grammar
consistency gate → node vocabulary paired with `mizar-parser`.

## Crate Prerequisites

The crate depends on `mizar-session` (for `SourceId`, `SourceRange`,
`SourceAnchor`) and `rowan` (for immutable green-tree storage). The task-11/12
`SurfaceNode`-style data remains privately stored inside `SurfaceAst`, with a
temporary public compatibility API that includes exported compatibility types,
read-only accessors, typed views, and `SurfaceNode` constructors/fields over the
rowan-backed representation. Do not add
`salsa` here: the query engine belongs to the
frontend/build/resolver/checker layers, while this crate must stay an
immutable, query-friendly syntax data boundary. The task-11/12 minimal boundary
(`SurfaceAst`, `SurfaceNode`, recovery kinds, `SyntaxDiagnostic`) is already
consumed by `mizar-parser` and `mizar-frontend::parsing::MizarParserSeam`;
every change here must keep `cargo test -p mizar-parser` and
`cargo test -p mizar-frontend` green in the same change.

## Resolved And Open Decisions

- **Syntax-tree backend: resolved.** `SurfaceAst` owns a rowan-backed green
  tree. Compatibility wrappers for existing task-11/12 names remain temporarily
  public as exported types, read-only accessors, typed views, and
  `SurfaceNode` constructors/fields over privately stored data, but parser tasks
  5-7 must grow against the `SurfaceAstBuilder` and typed accessor boundary
  rather than a custom arena backend.
- **Trivia ownership: resolved.** `mizar-frontend` owns comment/doc-comment
  extraction, raw doc-comment bodies, lexical text, and preprocess maps.
  `SurfaceAst` carries syntax-owned trivia side tables that reference that
  frontend-owned data by `SourceRange` plus syntactic attachment hints.
- **Salsa integration: deferred from this crate, required later.** `salsa` is
  required in the compiler's query and cache layers, not in `mizar-syntax`.
  Keep `SurfaceAst` immutable,
  deterministic, and cheap to share so later `salsa` queries can use it as an
  output value without changing this crate's semantic-free boundary.
- **Dot-role surface shape: resolved for parser/syntax by `mizar-parser` task
  10.** The parser cannot fully separate selector access from namespace
  separation without the resolver (spec
  [§A.2.5](../../../spec/en/appendix_a.grammar_summary.md)); the AST therefore
  keeps dotted qualified-name heads as qualified surfaces and parses `.` after
  an already parsed term as a selector/update postfix. Scope-dependent
  selector-versus-namespace classification remains resolver-owned. Also
  registered at the top level ([../../todo.md](../../todo.md) "Resolved And
  Open Decisions"); tracked in
  [../../mizar-parser/en/todo.md](../../mizar-parser/en/todo.md).
- **Public enum forward compatibility: open, initially resolved by the
  pre-consumer gate.** Apply the same per-enum `#[non_exhaustive]`-versus-
  exhaustive decision procedure that `mizar-frontend` task 25 established
  before `mizar-syntax` becomes a resolver/LSP input, and revisit it as new
  vocabulary enums are added.

## Ordered Task List

Representation-foundation tasks are sized to be implemented, tested, and
committed on their own. Later node-vocabulary entries are tracking buckets:
their individual parser-paired increments should land as separate tested
changes, and the bucket is checked off only when the last paired increment
lands. Keep `cargo test -p mizar-syntax` green after each change (see
[Recommended Verification](#recommended-verification)).

### Representation foundation

1. **Module split and lint-policy guard.** [x]
   - Split `src/lib.rs` into `pub mod ast;` and `pub mod recovery;`, moving the
     task-12 types without behavior changes, and re-export everything from the
     crate root so `mizar-parser` and `mizar-frontend` paths stay valid.
   - Add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard: workspace
     lint opt-in, deny baseline, and rationale required next to any future
     `allow`.
   - Tests: existing consumers compile unchanged; lint-policy guard passes.
   - Deps: none. Spec: [ast.md](./ast.md), [recovery.md](./recovery.md).

2. **`rowan` storage boundary and builder/accessor API.** [x]
   - Adopt a rowan-backed `SurfaceAst` green-tree representation and record the
     decision with its rationale in [ast.md](./ast.md). If existing
     `SurfaceNode`/`SurfaceNodeId` names are kept for compatibility, document
     them as wrappers or views over the rowan-backed representation rather than
     as the storage backend.
   - Define the raw `SyntaxKind`/node-kind mapping, node/token role
     conventions, range attachment rules, and typed accessor/view helpers that
     hide rowan internals from resolver/checker consumers unless exposure is
     deliberately documented.
   - Define the `SurfaceAstBuilder` boundary used by `mizar-parser`: parser code
     constructs nodes and recovery markers through builder/event APIs rather
     than pushing into a concrete arena or depending on rowan's raw tree shape.
   - Record identity rules: rowan green-node identity and any dense indices are
     internal cache details, not stable artifact ids; deterministic snapshots
     and content cache keys are the stability surface.
   - Tests: builder round-trip into the rowan-backed tree, typed accessor
     coverage for every current node/token kind, parent ranges contain child
     ranges except for documented recovery cases, and repeated construction
     produces deterministic snapshots.
   - Deps: 1. Spec: [ast.md](./ast.md) "Public API".

3. **Deterministic snapshot rendering.** [x]
   - Add a stable, human-readable text rendering of `SurfaceAst` (kind, range,
     recovered flag, children indented) for corpus snapshot baselines required
     by [architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md)
     "Snapshot Tests".
   - Rendering must be byte-identical across runs and platforms and free of
     hash-map iteration order, addresses, and other nondeterminism.
   - Tests: identical output across repeated renders; representative fixture
     covering every current node kind; recovery nodes visibly marked.
   - Deps: 2. Spec: [ast.md](./ast.md); snapshot envelope/update policy in
     [../../mizar-test/en/snapshot.md](../../mizar-test/en/snapshot.md). The
     concrete `SurfaceAst` body layout is owned by [ast.md](./ast.md)
     "Snapshot Rendering".

4. **Trivia model.** [x]
   - Add `pub mod trivia;`. Decide and record the ownership split with
     `mizar-frontend::PreprocessedSource` (which already owns comment and
     doc-comment extraction), then define trivia attachment: doc-comment
     attachment targets, skipped-token ranges, and whitespace-sensitive hints
     needed by formatter and LSP consumers.
   - Doc-comment attachment stays syntactic; no semantic interpretation.
   - Tests: trivia ownership and attachment hints; skipped ranges preserved with
     source ranges; rendering includes trivia deterministically when requested.
     The concrete "doc comment attaches to the following item node" fixture
     lands with the first item-node increment in task 9 / S-009 with parser
     task 5.
   - Deps: 2, 3. Spec: [trivia.md](./trivia.md).

5. **Recovery vocabulary expansion.** [x]
   - Extend `SyntaxRecoveryKind` beyond the task-12 minimum (`ErrorToken`,
     `MissingEnd`, `MissingStringLiteral`) to the full vocabulary promised by
     [recovery.md](./recovery.md): missing constructs, skipped tokens,
     unmatched delimiters, malformed annotations.
   - Keep the `recovered` flag contract: resolver and checker phases must be
     able to skip or reject recovered subtrees without re-parsing.
   - Tests: each recovery kind constructible with correct ranges; recovered
     subtree query helpers; snapshot rendering marks each kind distinctly.
   - Deps: 2. Spec: [recovery.md](./recovery.md).

### Pre-consumer compatibility gate

**Initial public enum forward-compatibility gate.** [x]
- For each public enum available at the phase-3 boundary (`SyntaxKind`,
  `SurfaceNodeKind`, `SurfaceTokenKind`, `SurfaceOperatorAssociativity`,
  `SyntaxRecoveryKind`, `SyntaxDiagnosticCode`, and trivia kinds introduced by
  task 4), decide `#[non_exhaustive]` versus deliberate exhaustiveness using the
  `mizar-frontend` task-25 procedure.
- Record each decision next to the enum in the owning module spec and apply the
  attributes before parser tasks 5-7 can make resolver/LSP consumers plausible.
- Result: `SyntaxKind`, `SurfaceNodeKind`, `SurfaceTokenKind`,
  `SyntaxRecoveryKind`, `SyntaxDiagnosticCode`, `TriviaAttachmentTarget`,
  `SkippedTokenReason`, and `WhitespaceHintKind` are guarded as
  `#[non_exhaustive]`. `MizarLanguage`, `SurfaceOperatorAssociativity`, and
  `TriviaPlacement` are documented deliberate exhaustive exceptions.
- Deps: 4, 5. Spec: [ast.md](./ast.md), [trivia.md](./trivia.md),
  [recovery.md](./recovery.md).

### Grammar consistency gate

These tasks intentionally interrupt the node-vocabulary track. Do not start
new AST node-kind design beyond the current task-12 compatibility surface until
the grammar audit has either closed its findings or explicitly recorded them
as accepted follow-ups. The purpose is to avoid freezing grammar drift into
`SurfaceAst` node kinds, child roles, or snapshot baselines.

6. **Canonical grammar consistency audit.** [x]
   - Treat [Appendix A](../../../spec/en/appendix_a.grammar_summary.md) as the
     parser-facing canonical grammar summary and review it against the
     chapter-local syntax blocks in Chapters 2-21.
   - Check for undefined nonterminals, duplicate definitions, unreachable
     productions from `compilation_unit`, direct left recursion outside the
     documented precedence parsers, chapter drift, reserved-token mismatches,
     and ambiguous boundaries that would affect AST shape.
   - Record findings in a grammar audit note or in the relevant spec files,
     with concrete references and one of: fix before AST design, accept as a
     semantic-only issue, or defer with an owner and parser task.
   - Result: recorded in [grammar_audit.md](./grammar_audit.md). Appendix A and
     Chapter 2 were synchronized for the concrete parser-facing fixes found in
     the audit: built-in predicates are reachable, annotations attach through
     parser-owned statement/item wrappers, `claim` is top-level rather than an
     algorithm statement, `is` assertions stay generic until resolution,
     `character` is defined, and `step` / `..` token drift is normalized at the
     spec level. Lexer table/test synchronization was completed in the
     `mizar-lexer` track, with affected lexical coverage entries marked
     covered. Remaining pre-AST issues and Task 7 fixture inputs are classified
     in the audit note.
   - Deps: Appendix A normalization. Spec:
     [../../../spec/en/appendix_a.grammar_summary.md](../../../spec/en/appendix_a.grammar_summary.md),
     chapter-local grammar sections under [../../../spec/en/](../../../spec/en/00.index.md).

7. **Parse-only acceptance matrix and fixture plan.** [x]
   - Define a parse-only acceptance matrix before AST snapshots are designed:
     positive, negative, ambiguous, and recovery-required examples for module
     structure, declarations, type expressions, term expressions, formulas,
     statements/proofs, annotations, registrations, templates, and algorithms.
   - Keep expectations independent of final AST shape at this stage. The
     expected result should be syntactic acceptance, rejection, or recovery
     category, plus the grammar rule being exercised.
   - Identify which fixtures belong to `mizar-parser`, which belong to
     `mizar-test`, and which are pure spec examples. Record traceability to
     Appendix A sections so later AST snapshots inherit a stable fixture set.
   - Result: recorded in
     [parse_only_acceptance_matrix.md](./parse_only_acceptance_matrix.md).
     The matrix uses syntax-only outcomes (`accept`, `reject`,
     `ambiguous-preserve-surface`, `recover`), maps ambiguous rows to ordinary
     parse acceptance for executable corpus expectations, classifies fixture
     ownership, and records Appendix A traceability for each covered area.
   - Deps: 6. Spec:
     [../../mizar-test/en/staged_model.md](../../mizar-test/en/staged_model.md),
     [../../mizar-test/en/expectation_schema.md](../../mizar-test/en/expectation_schema.md).

8. **Initial parse-only grammar fixture seed.** [x]
   - Add the first small parse-only fixture seed or, if parser support is not
     ready, a checked-in fixture manifest/design note that can be activated
     without changing the selected cases.
   - Cover at least the high-risk grammar boundaries identified by task 6:
     term-vs-formula boundaries, dot chains, statement reachability, import
     prelude forms, contextual string literals, `qua`, `the`, `reconsider`,
     and recovery around missing delimiters or `end`.
   - Do not require final AST node snapshots yet. AST snapshots are added only
     after the corresponding node vocabulary increment defines node kinds,
     child roles, range rules, and recovery rendering.
   - Result: recorded in
     [parse_only_fixture_seed.md](./parse_only_fixture_seed.md). Current parser
     readiness is not sufficient for default-discovered full-grammar corpus
     execution, so the Task 8 seed is a checked-in fixture manifest that keeps
     the selected case IDs, source shapes, parse-only expectations, and
     activation targets stable. The seed prioritizes the Task 7 Fixture
     Activation Plan and adds supplemental rows for `qua`, `reconsider`, and
     the string-required annotation rejection boundary. No AST snapshots were
     added.
   - Deps: 7. Spec:
     [../../../spec/en/appendix_a.grammar_summary.md](../../../spec/en/appendix_a.grammar_summary.md),
     [../../mizar-test/en/layout.md](../../mizar-test/en/layout.md).

### Node vocabulary (paired with `mizar-parser` grammar tasks)

Node kinds for each area are added **incrementally**: each increment lands in
the same change as the `mizar-parser` grammar task that constructs it (the
parser todo's numbering governs the change granularity), and each increment
extends snapshot rendering. A vocabulary task below is checked off when the
last of its paired parser tasks lands. Do not add node kinds speculatively
ahead of the parser task that constructs them, and do not begin these tasks
until tasks 6-8 have produced the grammar audit and parse-only fixture plan.
Each increment must first extend the vocabulary-increment contract in
[ast.md](./ast.md) with node kinds,
payloads, child roles, range rules, accessors, snapshots, and recovery/trivia
interaction. Spec references are the normative grammar chapters under
[doc/spec/en/](../../../spec/en/00.index.md).

9. **Module, item, and shared path nodes.** [x] — paired with `mizar-parser`
   tasks 4-7.
   - Shared qualified-symbol/namespace-path nodes needed by parser task 4 before
     import parsing; module file shape, top-level item list and item kinds
     dispatchable by keyword (parser task 5); import items with aliases and
     relative prefixes (parser task 6); export and visibility forms (parser task
     7).
   - Progress: parser task 4 landed the shared path-node increment:
     `ModulePath`, `NamespacePath`, `QualifiedSymbol`, `PathSegment`, and
     `RelativePrefix`, plus parser helper unit coverage. Parser task 5 landed
     `CompilationUnit`, `ItemList`, and `PlaceholderItem`, item-level
     skipped-token recovery trivia, active module-skeleton corpus coverage, and
     the first doc-comment-to-item attachment fixture. Parser task 6 landed
     `ImportItem`, `ImportAliasDecl`, and `ModuleBranchImport`, import-specific
     typed accessors and snapshot coverage, `MalformedImport`, active
     import-item corpus coverage, and import-stub parse-only harness support.
     Parser task 7 landed `ExportItem`, `VisibilityMarker`, and `VisibleItem`,
     export/visibility typed accessors and snapshot coverage,
     `MalformedExport` and `MalformedVisibility`, and active export/visibility
     corpus coverage. The S-009 bucket is complete.
   - Spec: [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

10. **Type expression nodes.** [x] — paired with `mizar-parser` task 8.
   - Attribute chains (with `non`), radix/mode type heads, `of`/`over`
     arguments, struct-qualified attribute references.
   - Spec: [03.type_system.md](../../../spec/en/03.type_system.md),
     [§A.3.2](../../../spec/en/appendix_a.grammar_summary.md).
   - Result: added `ReserveItem`, `ReserveSegment`, `TypeExpression`,
     `AttributeChain`, `AttributeRef`, `ParameterPrefix`, `TypeHead`,
     `TypeArguments`, and `TermPlaceholder` syntax vocabulary, rowan/snapshot
     coverage, typed accessors, `MalformedTypeExpression`, and active
     parse-only corpus coverage with paired parser task 8.

11. **Term nodes.** [x] — paired with `mizar-parser` tasks 9-12 and 15.
   - Consume the shared path vocabulary introduced by task 9. Then add primary
     terms (parser task 9), syntax-only dot-role and selector access/update
     surfaces (parser task 10), functional
     structure updates, `qua` (parser task 11), operator-expression nodes
     generalizing the task-12 `InfixExpression` to prefix/postfix forms (parser
     task 12), and Fraenkel/set-builder forms (parser task 15). Primary-term
     coverage includes `it`, choice expressions (`the type_expression`),
     structure constructors, set enumeration literals, and application forms.
   - Progress: parser task 9 has landed the primary-term increment:
     `TermExpression`, `TermReference`, `NumeralTerm`, `ItTerm`,
     `ParenthesizedTerm`, `ChoiceTerm`, `ApplicationTerm`,
     `StructureConstructor`, `FieldArgument`, and `SetEnumeration`, plus
     `MalformedTermExpression`, `MissingTerm` recovery coverage, and active
     parse-only primary-term corpus cases. Parser task 10 has landed the
     syntax-only dot-role increment: `SelectorAccess`, `StructureUpdate`, and
     `FieldUpdate`, selector/update recovery coverage, and active parse-only
     selector/update corpus cases. Parser task 11 has landed the `qua`
     increment: `QuaExpression`, bracket `qua_arg` migration away from
     `TermPlaceholder`, `MissingTypeExpression` target recovery coverage, and
     active parse-only `qua` corpus cases. Parser task 12 has landed the
     operator-expression increment: `PrefixExpression`, `PostfixExpression`,
     active prefix/postfix/infix Pratt grouping, non-associative and dangling
     operator diagnostics, and active parse-only operator corpus cases. Parser
     task 15 has landed the Fraenkel/set-builder increment:
     `SetComprehension`, `ComprehensionVariableSegment`, top-level `where`
     disambiguation from `SetEnumeration`, missing generator/type/condition
     and closing-brace recovery, active parse-only set-comprehension corpus
     cases, and rowan/snapshot/typed-accessor coverage. S-011 is complete for
     the documented parse-only term surface; binder identity, sethood, capture,
     mapper typing, and elaboration remain later semantic work.
   - Spec: [13.term_expression.md](../../../spec/en/13.term_expression.md),
     [appendix_b.operator_precedence.md](../../../spec/en/appendix_b.operator_precedence.md).

12. **Formula nodes.** [x] — paired with `mizar-parser` tasks 13-14.
   - Atomic predicate application and generic `is` assertions that resolution
     later classifies as type or attribute assertions (parser task 13);
     connectives and quantifiers (`for`/`ex`/`st`/`holds`) (parser task 14).
   - Result: parser tasks 13-14 are implemented. Atomic formula nodes,
     generic `is` assertions, formula constants, prefix/binary formula nodes,
     parenthesized formulas, quantifier variable segments, quantified
     formulas, missing-formula recovery, theorem/lemma formula
     hosting, syntax typed accessors, parser unit tests, and active parse-only
     pass/fail corpus coverage are in place. Template predicate arguments
     remain deferred to task 31 / S-016, and formula-embedding
     Fraenkel/set-builder terms are implemented by parser task 15 / S-011.
   - Spec: [14.formulas.md](../../../spec/en/14.formulas.md).

13. **Statement nodes.** [x] — paired with `mizar-parser` tasks 16 and 18-21.
    - Simple statements `let`, `assume`, `take`, `set`, `given`
      (parser task 16); top-level `reserve` remains the existing
      `ReserveItem` path because Chapter 4 forbids block-local
      `reserve`-shaped statements; `consider`/`reconsider` (parser task 18);
      `thus`/`hence`, `then` chains, iterative equality `.=` (parser task 19);
      compact equality statements versus zero-step iterative equality
      dispatch (grammar audit G-AUD-010);
      `now`/`hereby` and `per cases`/`suppose` blocks (parser task 20);
      `deffunc`/`defpred` local definitions (parser task 21).
    - Result: parser tasks 16, 18, 19, 20, and 21 are implemented. `ConsiderStatement`,
      `ReconsiderStatement`, and `ReconsiderItem` now cover shared-type
      `consider` variables, condition lists, mandatory simple justifications,
      reconsider item lists, target types, task-18 recovery, scope-skeleton
      `type_change_list` support, syntax typed accessors, parser unit tests,
      and active parse-only pass/fail corpus coverage. `ConclusionStatement`,
      `ThenStatement`, `IterativeEqualityStatement`, and
      `IterativeEqualityStep` now cover `thus`/`hence`, linkable `then`
      statements, iterative equality `.=` steps, label and `then` variants,
      the G-AUD-010 dispatch boundary, parser unit tests, and active
      parse-only pass/fail corpus coverage. `NowStatement`, `HerebyStatement`,
      `CaseReasoningStatement`, `CaseItem`, and `SupposeItem` now cover
      `now` / `hereby` blocks, `per cases` branch blocks, optional explicit
      `per cases by` justification, homogeneous `case` / `suppose` branches,
      `then per cases`, block-end recovery, parser unit tests, and active
      parse-only pass/fail corpus coverage. `InlineFunctorDefinition`,
      `InlinePredicateDefinition`, and `TypedParameter` now cover local
      `deffunc` / `defpred` definitions, standalone-only dispatch, `be` /
      `being` typed parameters, zero-argument definitions,
      delimiter/name/type/body recovery, parser unit tests, syntax typed
      accessors, and active parse-only pass/fail corpus coverage. S-013 is
      complete.
    - Spec: [15.statements.md](../../../spec/en/15.statements.md).

14. **Theorem, proof, and justification nodes.** [x] — paired with
    `mizar-parser` tasks 17 and 22.
    - Justification clauses (`by`), citation forms including `.{ … }` and
      `.*`, `let ... by references`, a minimal explicit compact-statement host,
      plus `by computation(...)` option nodes (parser task 17);
      `theorem`/`lemma` items, labels, `proof … end` nesting (parser task 22).
      The canonical Chapter 15/16 grammar does not define `from` as a
      justification form, so earlier `from` wording is treated as
      derived-documentation drift rather than implemented syntax.
      Parser task 17's citation/computation subset is complete; parser task 22
      has added `TheoremItem`, `LemmaItem`, and `ProofBlock`, status-token
      preservation, visibility-wrapped theorem targets, proof-body statement
      wiring, statement-level proof justifications on conclusion and compact
      statement hosts, theorem/proof recovery, typed accessors, and active
      parse-only pass/fail corpus coverage. S-014 is complete.
    - Spec: [15.statements.md](../../../spec/en/15.statements.md),
      [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md),
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md)
      §20.9.2.

15. **Definition, structure, and registration nodes.** [ ] — paired with
    `mizar-parser` tasks 23-30.
    - Definition block skeleton, correctness-condition clauses, and `attr`
      definitions (parser task 23); `pred`/`func`/`mode` bodies (parser tasks
      24-26); `redefine`, `synonym`/`antonym` (parser task 27); property
      clauses (parser task 28); `struct` definitions with fields and
      inheritance (parser task 29); registration and cluster forms and
      `reduce` (parser task 30).
      Parser task 23 is complete for the first S-015 increment:
      `DefinitionBlockItem`, `DefinitionParameter`, `AttributeDefinition`,
      `AttributePattern`, `FormulaDefiniens`, `FormulaCase`, and
      `CorrectnessCondition` are implemented with typed accessors, definition
      block recovery, active parse-only pass/fail corpus coverage, and
      traceability metadata for attribute definitions and correctness
      conditions. Parser task 24 is complete for predicate definitions:
      `PredicateDefinition` and raw `PredicatePattern` are implemented with
      typed accessors, definition-local visibility, formula-definiens bodies,
      grammar-shaped pattern validation, active parse-only pass/fail corpus
      coverage, and traceability metadata. S-015 remains open for parser tasks
      25-30.
    - Spec: [06.attributes.md](../../../spec/en/06.attributes.md),
      [07.modes.md](../../../spec/en/07.modes.md),
      [09.predicates.md](../../../spec/en/09.predicates.md),
      [10.functors.md](../../../spec/en/10.functors.md),
      [11.symbol_management.md](../../../spec/en/11.symbol_management.md),
      [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md),
      [05.structures.md](../../../spec/en/05.structures.md),
      [17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md).

16. **Template, algorithm, and annotation nodes.** [ ] — paired with
    `mizar-parser` tasks 31-35.
    - Template parameters and bracket-form type arguments (parser task 31);
      algorithm blocks, assignment, declarations, ghost declarations/assignments,
      snapshots, top-level `claim` blocks, and returns (parser task 32);
      control flow including processed collection loops and match endings
      (parser task 33); verification clauses (parser task 34);
      statement-level annotations, `@[...]` library annotations, and
      string-literal annotation arguments (parser task 35).
    - Spec: [18.templates.md](../../../spec/en/18.templates.md),
      [20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md),
      [21.source_code_annotation_and_atp.md](../../../spec/en/21.source_code_annotation_and_atp.md).

### Cross-cutting follow-ups

17. **Public enum forward-compatibility policy.** [ ]
    - Revisit the initial public-enum gate once the vocabulary is complete and
      decide `#[non_exhaustive]` versus deliberate exhaustiveness for any public
      enums added by later node-vocabulary increments.
    - Record final decisions next to the enum in the owning module spec and
      apply any remaining attributes.
    - Deps: 16. Spec: all module specs.

18. **Incremental syntax reuse audit.** [ ]
    - Audit the completed rowan-backed syntax tree for fine-grained incremental
      parsing and LSP reuse readiness: stable syntax-kind numbering policy,
      trivia/recovery placement, range attachment, node-role accessors, and
      subtree snapshot behavior under localized edits.
    - This task does not introduce `salsa`; it verifies that `SurfaceAst` can
      be produced and cached by later query layers without exposing unstable
      arena ids or parser internals.
    - Deps: 16, 17. Spec: [ast.md](./ast.md), [trivia.md](./trivia.md),
      [recovery.md](./recovery.md).

19. **Source/spec correspondence audit.** [ ]
    - Mirror the `mizar-frontend` task-16 audit: trace every public API and
      promised behavior in [ast.md](./ast.md), [trivia.md](./trivia.md), and
      [recovery.md](./recovery.md) to implementation and tests, and record
      gaps as follow-up tasks.
    - Deps: 18. Spec: all module specs and this TODO.

20. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-syntax/en/` with its Japanese companion and synchronize
      API lists, statuses, terminology, links, and behavior promises.
    - Deps: 19. Spec: repository documentation policy.

21. **Rustdoc summaries.** [ ] Deferred.
    - Same workspace-level deferral as `mizar-frontend` task 26. Re-entry
      trigger: the first long-lived consumer outside the frontend pipeline
      (resolver or `mizar-lsp`) starts coding against `mizar-syntax`, or the
      workspace adopts a rustdoc policy — whichever comes first.
    - Deps: 17. Spec: repository documentation policy.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-syntax
cargo fmt --check
cargo clippy -p mizar-syntax --all-targets --all-features -- -D warnings
```

For tasks that move or extend the shared boundary, also run:

```text
cargo test -p mizar-parser
cargo test -p mizar-frontend
```

Check the task off here once tests pass.

## Notes

- `mizar-syntax` owns syntax data shapes only: no grammar logic, no name
  resolution, no typing, no proof semantics. Resolved symbol ids, inferred
  types, and proof obligations never appear in `SurfaceAst`.
- `rowan` is the syntax-tree backend; parser and consumer code should
  depend on `mizar-syntax` builder/accessor APIs, not on an ad hoc arena layout.
- `salsa` is a later query/cache layer concern. Preserve pure phase boundaries
  and immutable syntax snapshots here so it can be introduced without rewriting
  the syntax crate.
- Vocabulary growth is paced by `mizar-parser` grammar tasks; do not add node
  kinds speculatively ahead of a parser task that constructs them.
- `SurfaceAst` is internal compiler data, not a stable external schema; the
  snapshot rendering (task 3) is the stability surface for corpus baselines.
