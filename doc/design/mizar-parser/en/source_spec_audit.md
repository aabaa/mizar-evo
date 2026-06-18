# mizar-parser: Source/Spec Correspondence Audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_audit.md](../ja/source_spec_audit.md).

Status: task 43 audit complete.

## Scope

This audit traces the public API and implementation-facing behavior promised by
[grammar.md](./grammar.md), [pratt.md](./pratt.md), and
[recovery.md](./recovery.md) to source and tests. It also records the Appendix A
reserved-word coverage check required by task 43.

Authority order for this audit remains `doc/spec/en/`, executable `.miz`
tests, `tests/coverage/spec_trace.toml`, expectation sidecars, design docs, and
then source. No language specification, existing `.miz` source, or existing
expectation was changed to match implementation behavior.

## Result

No blocking non-deferred `spec_gap`, `boundary_violation`, or
`repo_metadata_conflict` was found. Implemented parser behavior through task 42
is represented by source, unit tests, active parse-only corpus cases, and
traceability metadata. The audit did find one deferred operator-declaration
gap, recorded below.

The only task-43 follow-up is deferred:

| ID | Classification | Follow-up | Status |
|---|---|---|---|
| P-043-01 | `source_drift` / `test_gap` | Concrete `operator_decl` parsing and active parser corpus coverage for `infix_operator`, `prefix_operator`, `postfix_operator`, and infix associativity words `left`, `right`, `none`. Canonical Appendix A, Chapter 10, and Chapter 13 already give these words grammar positions, but current parser behavior recognizes the declaration keywords only as task-5 top-level notation starts or consumes Pratt metadata supplied through `ParseRequest::operator_fixity`; it does not parse concrete operator declarations from source yet. | Deferred parser task 46 |

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
| Type, term, formula, statement, proof, definition, structure, registration, template, algorithm, claim, verification-clause, annotation, and predicate redefinition-label surfaces are implemented through task 36. | `src/module.rs`, `src/module/annotations.rs`, `src/path.rs` | active parser pass/fail corpus requirements in `tests/coverage/spec_trace.toml`, parser unit tests, parse-only snapshots | No finding |
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
| `infix_operator` | Concrete source-level operator declarations are deferred to P-043-01 / parser task 46. |
| `prefix_operator` | Concrete source-level operator declarations are deferred to P-043-01 / parser task 46. |
| `postfix_operator` | Concrete source-level operator declarations are deferred to P-043-01 / parser task 46. |
| `left` | Infix associativity value used only inside deferred concrete `infix_operator` declarations. |
| `right` | Infix associativity value used only inside deferred concrete `infix_operator` declarations. |
| `none` | Infix associativity value used only inside deferred concrete `infix_operator` declarations. |
| `transitivity` | Reserved by the provisional Appendix A word list, but not part of the canonical implemented property productions; task 28 records the design drift and there is no current parser grammar position. |

All other Appendix A reserved words appear in at least one active parser corpus
`.miz` source as frontend `ReservedWord` tokens.
