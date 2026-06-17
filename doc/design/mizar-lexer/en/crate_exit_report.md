# Crate Exit Report: mizar-lexer

> Canonical language: English. Japanese companion: [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the `mizar-lexer` crate-wide autonomous-development
milestone started from commit `b9f2482` (`docs: refine lexical symbol naming
rules`).

Quality score: reviewed 94/100.

Score caps applied: none. The required read-only crate-exit review confirmed
that all Crate Exit Gates pass. The only remaining gaps, `MLX-GAP-001` and
`MLX-GAP-005`, are explicitly deferred and outside lexer-owned implementation
scope. `MLX-GAP-007` and `MLX-GAP-008` are resolved.

## Scope

Milestone scope:

- apply the lexical-symbol naming updates from `b9f2482` to `mizar-lexer`;
- complete every non-deferred item in the `mizar-lexer` Ordered Task List;
- resolve `MLX-GAP-007` and `MLX-GAP-008`;
- perform paired `mizar-frontend` and `mizar-parser` work required for the
  lexer handoff;
- keep English and Japanese design documents synchronized.

Included:

- range-aware current-module lexical declaration support;
- separation of notation symbols from readable constructor names;
- source-position-aware parser-facing operator metadata;
- `TokenizeRequest.current_module`, parser-input local declaration forwarding,
  parser cache-key updates, and Pratt lookup filtering by token source position;
- lexer, frontend, parser, and cache tests for the above behavior;
- bilingual design updates for the crate plan, TODO, lexer/frontend/parser
  handoff documents, and this exit report.

Excluded or deferred:

- adding `.miz` fixtures for pre-parser lexer phases (`MLX-GAP-001`);
- authoritative selector-vs-namespace semantic resolution (`MLX-GAP-005`);
- module resolution, type checking, overload resolution, proof checking, and
  LSP/user-facing diagnostic rendering;
- changes to `doc/spec/en`, existing `.miz` tests, or test expectations merely
  to match implementation behavior.

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | `doc/spec/en` and the `b9f2482` bilingual spec/design updates were treated as authoritative. No spec gap or contradiction blocked implementation. Review-only agents reported no remaining specification or documentation findings for the implemented tasks. |
| Source behavior documented and tested | Pass | Local declaration ranges, constructor-name restrictions, selector exclusion, non-introducing `deffunc`/`defpred`/`algorithm`, imported/local overload preservation, and source-position-aware operator metadata are documented in the lexer/frontend/parser design notes and covered by Rust tests. |
| Milestone-owned tests | Pass with deferred `.miz` rationale | Crate-local lexer tests, parser tests, frontend tests, cache tests, and `mizar-test` verification cover the implemented contracts. `.miz` additions remain deferred only for lexer phases that operate before complete parser source is meaningful. |
| Test expectation discipline | Pass | No existing `.miz` test or `.expect.toml` file was changed to follow current implementation behavior. |
| Design/source sync | Pass | `doc/design/mizar-lexer`, `doc/design/mizar-frontend`, and `doc/design/mizar-parser` describe the implemented handoff. The source/documentation consistency reviews ended with no findings. |
| Boundary discipline | Pass | Lexer changes remain lexical and handoff-oriented. `private`/`public` visibility is not interpreted by lexing, selector semantics remain downstream-owned, and parser/frontend changes only consume lexer metadata. |
| Verification | Pass | Narrow crate tests, full workspace formatting, Clippy, full tests, and traceability planning passed. Details are listed below. |
| Residual risk | Pass | Remaining risks are classified as deferred or downstream-owned. No unresolved `source_undocumented_behavior`, `test_expectation_drift`, `boundary_violation`, or `repo_metadata_conflict` remains for this milestone. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

The reviewed score keeps deductions for the intentional `.miz` fixture
deferral, the downstream selector-vs-namespace handoff, and residual review
risk at the lexer/parser/frontend boundary. The score is valid because the
read-only crate-exit review confirmed that all hard gates pass.

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MLX-GAP-001 | Lexer-owned tests still use `.src` fixtures where sub-file lexical behavior is the reviewable contract. | Parser/frontend milestone | Add `.miz` tests when complete-source lexing/parser integration can assert the same behavior without losing lexer phase isolation. |
| MLX-GAP-005 | `spec.en.02.lexical.dot_disambiguation.selector_shadowing_handoff` remains downstream semantic work. | Parser/resolver integration | Add complete-source tests and downstream implementation for authoritative selector-vs-namespace semantic resolution. |

Resolved during this milestone:

- `MLX-GAP-007`: current-module declarations and operator metadata now become
  active only after the declaring item is complete, with source-position-aware
  lexer/frontend/parser lookup.
- `MLX-GAP-008`: lexer metadata now separates arbitrary notation symbols from
  constructor-name spellings, and selector/generic constructor summary entries
  are rejected at the lexer summary boundary.
- Earlier documented gaps `MLX-GAP-002`, `MLX-GAP-003`, `MLX-GAP-004`, and
  `MLX-GAP-006` remain resolved by the crate plan, README links, traceability
  classification, and ownership-boundary documentation.

## Human Review Surface

Primary documents:

- [00.crate_plan.md](./00.crate_plan.md)
- [todo.md](./todo.md)
- [lexical_environment.md](./lexical_environment.md)
- [../ja/00.crate_plan.md](../ja/00.crate_plan.md)
- [../ja/todo.md](../ja/todo.md)
- [../ja/lexical_environment.md](../ja/lexical_environment.md)
- [../../mizar-frontend/en/lexing.md](../../mizar-frontend/en/lexing.md)
- [../../mizar-frontend/en/parsing.md](../../mizar-frontend/en/parsing.md)
- [../../mizar-parser/en/expression_parser.md](../../mizar-parser/en/expression_parser.md)
- [../../../spec/en/02.lexical_structure.md](../../../spec/en/02.lexical_structure.md)
- [../../../spec/en/11.symbol_management.md](../../../spec/en/11.symbol_management.md)
- [../../../spec/en/12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md)

Representative source and tests:

- `crates/mizar-lexer/src/lexical_environment.rs`
- `crates/mizar-lexer/src/disambiguator.rs`
- `crates/mizar-lexer/src/tests/lexical_environment.rs`
- `crates/mizar-frontend/src/lexing.rs`
- `crates/mizar-frontend/src/parsing.rs`
- `crates/mizar-frontend/src/orchestration.rs`
- `crates/mizar-frontend/src/cache_key.rs`
- `crates/mizar-parser/src/lib.rs`

Implementation commits:

- `c038b1c` `feat: add range-aware lexer declarations`
- `d133ad0` `feat: split lexer notation metadata`
- `6279950` `feat: make operator metadata source-position aware`

## Test Expectation Summary

No `.miz` source fixture, `.expect.toml` sidecar, or canonical specification file
was changed during the implementation commits.

Added or updated Rust coverage includes:

- lexer tests for declaration-before-use, declaration-after-use, declaration
  self-non-activation, local/import overload preservation, constructor-name
  restrictions, selector exclusion, aliases, visibility no-op behavior,
  operator metadata recording, and non-introducing local forms;
- frontend tests for current-module tokenization, parser-input forwarding,
  preprocessing coordinate mapping, orchestration, and cache-key sensitivity;
- parser tests for source-position-aware operator metadata and latest-active
  operator selection.

## Verification

Narrow verification was run for the task commits before each commit:

```sh
cargo test -p mizar-lexer
cargo test -p mizar-parser
cargo test -p mizar-frontend
cargo test -p mizar-test
```

Full verification run for the milestone:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

Results:

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test`: passed.
- `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml`: passed with 162 test cases, 90 requirements, 0 errors, and 4 warnings for existing planned requirements outside `mizar-lexer`:
  - `spec.en.algorithm.vc.assignment_loop_exits`
  - `spec.en.binding.substitution.capture_avoidance`
  - `spec.en.elaboration.choice_comprehension.lowering`
  - `spec.en.type_soundness.escape_and_guard_failures`

## Handoff

Next recommended work:

- Start parser/resolver/frontend work for the selector-vs-namespace semantic
  handoff and decide where complete-source `.miz` tests should complement the
  lexer `.src` fixture corpus.

Known constraints:

- Do not rebaseline lexical expectations to match implementation behavior
  without a spec-derived or explicitly approved test-intent change.
- Keep design-derived traceability ids subordinate to canonical `doc/spec/en`
  requirements.
- Keep selector-vs-namespace semantic resolution outside `mizar-lexer`.
- Preserve the source-position-aware operator metadata contract when changing
  parser expression handling.

Open questions:

- Which parser/resolver milestone should turn
  `spec.en.02.lexical.dot_disambiguation.selector_shadowing_handoff` from
  `partial` into covered complete-source behavior?
- Which lexer `.src` fixture families can safely gain `.miz` companions without
  weakening phase-isolated lexer review?

Recommended reasoning setting for the next task:

- `high`, because the next useful work crosses lexer, parser, frontend, and
  resolver ownership boundaries and may need careful test-authority decisions.
  Lower to `medium` for documentation-only traceability cleanup, and raise
  above `high` if the task changes canonical syntax, resolver semantics, or
  proof/type behavior.
