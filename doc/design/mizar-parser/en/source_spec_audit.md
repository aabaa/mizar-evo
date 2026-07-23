# mizar-parser: Source/Spec Correspondence Audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_audit.md](../ja/source_spec_audit.md).

Status: task 43 audit complete; Task 265 ownership refreshed through completed
Tasks 46-48. P-043-01/P-046 is closed by Task 46.

## Task 46 Source/Specification Recheck

Appendix A is the canonical parser-normalization authority; Chapter 10 supplies
the exact three forms and semantic boundary, and Chapter 12 supplies visible
top-level notation placement. Completed frontend Task 20 already provides the
named position-sensitive string context and local operator metadata handoff,
so the former trigger deferral was `design_drift`, not missing authority.

`src/module.rs` now emits `OperatorDeclaration` for the exact infix/prefix/
postfix forms at annotated/visible top-level and definition-local notation
positions. The active pass/fail pair and exact trace row close the selected
`source_drift` and `test_gap`. There is no selected-slice `spec_gap`,
`source_undocumented_behavior`, `boundary_violation`,
`test_expectation_drift`, or `repo_metadata_conflict`; existing `.miz` and
expectation files remain unchanged.

## Scope

This audit traces the public API and implementation-facing behavior promised by
[grammar.md](./grammar.md), [pratt.md](./pratt.md), and
[recovery.md](./recovery.md) to source and tests. It also records the Appendix A
reserved-word coverage check required by task 43.

Authority order for this audit remains `doc/spec/en/`, executable `.miz`
tests, `tests/coverage/spec_trace.toml`, expectation sidecars, design docs, and
then source. No language specification or existing `.miz` source was changed.
Task 47 changed one existing expectation only to remove a diagnostic that
contradicted the canonical grammar, not to rebaseline implementation behavior.

## Result

No blocking non-deferred `spec_gap`, `boundary_violation`, or
`repo_metadata_conflict` was found. Implemented parser behavior through task 42
is represented by source, unit tests, active parse-only corpus cases, and
traceability metadata. The original task-43 audit found one deferred operator-
declaration gap, recorded below; Task 265 adds two later, canonically grounded
current-state rows. Tasks 46-48 now close all three bounded parser slices.

The only task-43 follow-up is now closed:

| ID | Classification | Follow-up | Status |
|---|---|---|---|
| P-043-01 / P-046 | `source_drift` / `test_gap` (closed) | Concrete `operator_decl` parsing and active parser corpus coverage for `infix_operator`, `prefix_operator`, `postfix_operator`, and `left`/`right`/`none`. The parser emits exact syntax nodes and preserves Pratt metadata unchanged. | Completed parser Task 46. Semantic activation, resolution, and precedence validation remain downstream. |

Task 265 identified the following classified execution ownership. Tasks 47 and
48 have now closed their bounded parser/test/trace slices:

| ID | Classification | Evidence and exact scope | Current owner |
|---|---|---|---|
| P-265-47 | `source_drift` / `test_expectation_drift` / `test_gap` (closed) | The parser now accepts omitted, explicit-`by`, and proof-block `reconsider_tail`; the active corpus covers the two exact rows and the unchanged mixed recovery `.miz` no longer expects an error for the omitted form. | Completed parser Task 47. Semantic reconsider remains owned by checker authority and receives no parser credit. |
| P-265-48 | `source_drift` / `test_gap` (closed) | The parser now exposes the exact top-level Chapter-7 means/equals property-implementation form, append-only typed syntax, bounded recovery, and active pass/fail parse-only evidence. | Completed parser Task 48. Task-39 semantic activation remains gated and receives no parser credit. |

`transitivity` remains explicitly future-reserved for parser purposes. Task 28
already records the old TODO wording as design drift against the canonical
property productions; there is no current parser grammar position for a
`transitivity` property clause.

## Public API Trace

| Public surface | Spec promise | Source | Test evidence | Finding |
|---|---|---|---|---|
| `parse(ParseRequest) -> ParseOutput` | Crate-root parser entry point remains reachable and deterministic. | `crates/mizar-parser/src/lib.rs`, `src/grammar.rs` | `crates/mizar-parser/tests/lint_policy.rs`, `crates/mizar-parser/tests/determinism.rs`, active `mizar-test parse-only` corpus | No finding |
| `ParseRequest` | Carries `source_id`, `edition`, frontend-adapted tokens, source-position-aware operator fixity, and string-required context without resolver/build-system state. | `src/lib.rs`, `src/grammar.rs`, `src/recovery.rs` | parser unit tests, frontend seam tests, `mizar-test` parse-only runner | No finding |
| `ParserToken` / `ParserTokenKind` | Parser consumes frontend token transfer objects and preserves token kind/text/range into syntax output. | `src/lib.rs`, `src/grammar.rs`, `src/event.rs` | parser token-preservation tests, parse-only snapshots, lint public API reachability test | No finding |
| `OperatorFixityEntry`, `OperatorFixity`, `OperatorAssociativity` | Term Pratt parsing uses active spelling-level fixity metadata; associativity/fixity enums are deliberately exhaustive. | `src/lib.rs`, `src/grammar.rs`, `src/module.rs`, `src/pratt.rs` | parser operator tests, determinism tests, `pass_parser_operator_terms_001`, operator fail corpus | No finding |
| `StringRequiredContext` | String-required parser contexts are forward-compatible and currently support the synthetic test context. | `src/lib.rs`, `src/recovery.rs` | missing-string parser unit test, lint enum policy tests | No finding |
| `ParseOutput` | Returns optional `SurfaceAst` plus syntax diagnostics; unrecoverable stray `end` may return `ast = None`. | `src/lib.rs`, `src/recovery.rs`, `src/module.rs` | parser recovery unit tests, `fail_parser_stray_end_001`, active parse-only runner | No finding |

## Behavior Trace

| Design promise | Source | Test/corpus evidence | Finding |
|---|---|---|---|
| Parsing is syntax-only: no name resolution, type inference, overload selection, proof obligations, cache authority, or owner-origin ids. | `crates/mizar-parser/src/` has no resolver/build/cache dependencies; semantic facts are not produced by parser nodes. | parser crate tests and active corpus assert syntax shapes/diagnostics only. | No finding |
| Grammar code emits through the parser event sink and documented `mizar-syntax` builder/accessor boundary, not raw rowan layout. | `src/event.rs`, `src/grammar.rs`, `src/module.rs`, `src/module/annotations.rs` | parser unit tests plus `mizar-syntax` builder/view tests | No finding |
| Module skeleton, imports, exports, visibility wrappers, and placeholder top-level dispatch keep source order and recovery ownership. | `src/module.rs`, `src/sync.rs` | `pass_parser_module_skeleton_001`, `pass_parser_import_items_001`, `pass_parser_export_visibility_001`, late import/export fail cases | No finding |
| Type, term, formula, statement, proof, definition, operator declaration, property implementation, structure, registration, template, algorithm, claim, verification-clause, annotation, and predicate redefinition-label surfaces are implemented through task 36 plus Tasks 46-48. | `src/module.rs`, `src/module/annotations.rs`, `src/path.rs` | active parser pass/fail corpus requirements in `tests/coverage/spec_trace.toml`, parser unit tests, parse-only snapshots | No finding; P-043-01/P-046, P-265-47, and P-265-48 are closed. |
| Term Pratt parsing respects `active_from`, newest active same-spelling metadata, prefix/postfix/infix binding powers, `qua` as the fixed lowest term operator, and non-associative diagnostics. | `src/grammar.rs`, `src/module.rs`, `src/pratt.rs` | parser operator unit tests, `crates/mizar-parser/tests/determinism.rs`, `pass_parser_operator_terms_001`, operator fail cases | No finding |
| Formula Pratt parsing uses fixed connective precedence and outer quantifier parsing. | `src/module.rs` | `pass_parser_formula_connectives_001` and formula fail corpus | No finding |
| Recovery synchronizes at semicolons, `end`, top-level item starts, category-local starts, and EOF; recoverable syntax emits recovery nodes and diagnostics. | `src/recovery.rs`, `src/sync.rs`, `src/module.rs`, `src/module/annotations.rs` | fail parser corpus, parser recovery unit tests, task-37 consolidation cases | No finding |
| Stray unmatched `end` is intentionally unrecoverable and returns diagnostics with `ast = None`. | `src/recovery.rs` | `fail_parser_stray_end_001`, parser recovery unit tests | No finding |
| Parser determinism and frontend cache readiness are preserved: no global state, hidden caches, or salsa dependency. | `src/lib.rs`, `src/grammar.rs`, `src/module.rs` | `crates/mizar-parser/tests/determinism.rs`, parser fuzz target, frontend passthrough audit from task 41 | No finding |

## Reserved-Word Coverage

Task 43 adds a mechanical guard in
`crates/mizar-test/tests/metadata.rs`:
`repository_parser_reserved_words_are_covered_or_explicitly_deferred`. The test
reads the reserved-word block from
`doc/spec/en/appendix_a.grammar_summary.md`, runs active parser `.miz` corpus
sources through frontend preprocessing and tokenization, and counts only
frontend `ReservedWord` tokens. It fails if a reserved word is neither present
as a `ReservedWord` token in the active parser corpus nor listed as a
parser-deferred reserved word. It also fails if a deferred reserved word
disappears from Appendix A or starts appearing as a `ReservedWord` token in
active parser corpus sources without this audit being refreshed.

Current parser-deferred reserved words:

| Word | Reason |
|---|---|
| `transitivity` | Reserved by the provisional Appendix A word list, but not part of the canonical implemented property productions; task 28 records the design drift and there is no current parser grammar position. |

All other Appendix A reserved words appear in at least one active parser corpus
`.miz` source as frontend `ReservedWord` tokens.

## Task 47 Source/Specification Recheck

Task 47 closes P-265-47A/B/C. Private `parse_reconsider_statement_at` now
matches Chapters 4 §4.4.2, 8 §8.2.2, 15 §§15.5.1/15.8.2/15.12, and Appendix A
§§A.4/A.15 for omitted, simple-justification, and proof-block tails. The active
parse-only corpus supplies exact backlinks for both newly covered requirements,
and the historical omitted-tail expectation drift is removed without changing
an existing `.miz` source.

P-265-47D remains a nonblocking, human-owned `spec_gap`: Chapter 8's compact
EBNF writes one `reconsider_item`, whereas Chapters 4/15 and Appendix A use a
list. Task 47 preserves the already implemented source-ordered list and does
not edit `doc/spec`. P-265-48 is closed by Task 48 and P-046 by Task 46. No
`source_undocumented_behavior`, `boundary_violation`, or
`repo_metadata_conflict` was found.

## Task 48 Source/Specification Recheck

Task 48 closes P-265-48 against Chapters 7 §§7.4.1/7.8.2/7.10, Chapter 12
§12.7, and Appendix A §§A.7/A.12. `src/module.rs` dispatches the top-level
shape before the generic definition producer, emits one
`PropertyImplementation`, reuses the existing parameter/type/definiens/
correctness/justification nodes, and keeps recovery bounded across nested
blocks and following declarations. `mizar-syntax` adds append-only raw kind
192, stable snapshot/raw/rowan mappings, and the typed accessor.

The exact trace row is covered by the new pass/fail sidecars and the active
runner reports 99/99. Existing `.miz` and expectation files, the inactive
Task-39 semantic seed, semantic payload extraction, proof acceptance, and
checker/Core/CFG/VC behavior are unchanged. The closed classifications are
`source_drift`, `test_gap`, paired-document `design_drift`, and two internal
unit `test_expectation_drift` cases. No selected-slice `spec_gap`,
`source_undocumented_behavior`, `boundary_violation`, or
`repo_metadata_conflict` remains.
