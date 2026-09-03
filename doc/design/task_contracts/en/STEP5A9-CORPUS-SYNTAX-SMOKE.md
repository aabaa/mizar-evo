# Task STEP5A9-CORPUS-SYNTAX-SMOKE: corpus-wide syntax guard

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A9-CORPUS-SYNTAX-SMOKE.md](../ja/STEP5A9-CORPUS-SYNTAX-SMOKE.md).

Owning plan: [mizar-test](../../mizar-test/en/00.crate_plan.md).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Complete; full gates 9/9 and quality 100/100, pending local implementation commit |
| Tier | Full: production runner, CLI, and public Rust API |
| Owner / consumers | `mizar-test` owns corpus selection, the exact exception ledger, frontend execution, and the report; `mizar-frontend` and `mizar-parser` remain unchanged providers |
| Dependencies | Step 5A.1-5A.8 complete; all G1-G9 frontend gaps are closed |
| Authority | [Step 5A.9](../../todo.md#step-5a--frontend-gap-closure--), audit-1 [corpus map](../../mizar-test/en/semantic_bridge_corpus_map.md#follow-ups) / [frontend method](../../mizar-test/en/semantic_bridge_frontend_gaps.md#verification-method), Chapters [2](../../../spec/en/02.lexical_structure.md#24-reserved-words), [16](../../../spec/en/16.theorems_and_proofs.md#162-syntax-of-theorem-declarations), [18](../../../spec/en/18.templates.md#1811-complete-syntax-ebnf), and Appendix [A.15](../../../spec/en/appendix_a.grammar_summary.md#a15-statements-proofs-and-references) |
| Classification | `test_gap` and bounded `design_drift`; four pre-existing template sources expose a deferred internal `spec_gap`/`test_drift`, but this task changes neither side |
| Semantic-credit throughput | `0 tasks/week`; this guard activates no semantic oracle |

## Selection and acceptance

`run_syntax_smoke_corpus` shall clone the supplied `DiscoveryConfig`, force
`TestProfile::Full`, and build the ordinary fail-closed plan. It selects every
case whose source extension is exactly `.miz` and whose expectation stage is
not `parse_only`. It does not inspect active tags, expected outcome, expected
phase, diagnostic expectations, or activation-map membership. Canonical plan
order is retained. It also loads
`tests/coverage/syntax_smoke_expected_rejections.tsv`; a missing, malformed,
extra, duplicate, reordered, stale, or code/path-mismatched row fails closed.
Validation errors carried by a successfully built plan return no case results
and retain the plan diagnostics. An infrastructure `HarnessError` from plan
construction or ledger file I/O remains the public `Err` and CLI exit 2.

The ledger uses LF-delimited UTF-8 with the exact header
`case_id<TAB>source<TAB>syntax_diagnostic_codes<TAB>owner`. Data rows have
exactly four non-empty, unescaped tab-separated fields. `source` is the clean
forward-slash workspace-relative path; codes are non-empty stable syntax keys
joined by commas in emitted order, with no whitespace or empty member; `owner`
is the existing task/spec-decision owner token. No field may contain a tab,
newline, or comma except separators in the codes field. Duplicate ids or
sources are errors. Every row must resolve to exactly one selected plan case,
and row ordinals must be strictly increasing in canonical selected-plan order;
the runner never sorts or repairs them. Header/content violations are plan-like
validation errors with no results and CLI exit 1.

The checked-in ledger is the exact row/path/code authority for this task. Its
frozen SHA-256 is
`54bd225e86fffde5c3b114dcfd66bb5bfd18683cc96477078486ddfd9496b019`.

Each selected source runs once through the existing real `Frontend` with
`MizarParserSeam` and the existing syntax-only import provider. As in the
audit-1 harness, each call is panic-isolated so one source cannot abort the
remaining inventory. A case passes the syntax predicate only if execution does
not panic, execution succeeds, a `SurfaceAst` exists, and no
`DiagnosticCode::Syntax` is present. Other frontend diagnostics remain in the
ordered public report but do not fail this syntax-only guard. Panics, frontend
errors, and missing ASTs always fail. Syntax diagnostics pass only when their
complete ordered code list and source identity consume the next exact ledger
row; such a result is `ExpectedSyntaxRejection`, not silently `Passed`.
Unledgered or mismatched syntax diagnostics fail. The report uses stable
`frontend_panic`, `missing_ast`, and existing `frontend_error:<message>` forms
and never consults or changes the case's semantic oracle.

At execution, a ledger row is consumed only by an exact case id, normalized
source path, and complete syntax-code-vector match. A selected case with syntax
diagnostics and no matching next row is missing/mismatched; a row whose selected
case has no syntax diagnostics or is not consumed exactly once is stale/extra.
Each produces a validation error and `Failed` result where a case exists.

At freeze time the repository has 473 `.miz` cases: 113 `parse_only` and 360
eligible smoke cases. All currently use the default fast profile, but forcing
`Full` prevents later non-fast corpus additions from silently escaping the
guard. The test-first probe found 353 syntax-clean cases, seven exact expected
syntax rejections, and no unexpected failure once the ledger is applied.

The seven ledger rows preserve three specified active-range rejections owned by
Tasks 75/76/77 and four pre-existing template sources. The template sources use
reserved `left`/`right` as `field_name`, or append template loci to a theorem
label although the complete Chapter 16/18 and Appendix A `theorem_item` grammar
permits only `label_identifier ":"`. Chapter 18 examples conflict with that
complete production. This task makes the authorized no-language-change
disposition: do not loosen the parser and do not rewrite the sources or their
semantic expectations. The ledger records the exact blocker until its existing
template owner resolves the specification/test intent.

## API, tests, and documentation

Add public `SyntaxSmokeRunReport`, `SyntaxSmokeCaseResult`, and non-exhaustive
`SyntaxSmokeCaseStatus::{Passed, ExpectedSyntaxRejection, Failed}`, including
an expected-rejection count, plus `syntax_smoke_cases` and
`run_syntax_smoke_corpus`. Add the `syntax-smoke` CLI command with the same
summary/exit convention as existing runners. Keep execution in a private
`runner/syntax_smoke.rs` leaf and reuse the shared frontend path; do not change
the parser or its cache and do not add an import provider, expectation field,
tag, stage, or compatibility adapter.

Before production changes, add failing tests that require: exact selection of
non-`parse_only` `.miz` across profiles without active tags; exclusion of
`parse_only` and non-`.miz` sources; deterministic order/replay; pass only with
successful execution, an AST, and zero syntax diagnostics; rejection of
malformed syntax even when its sidecar is a later semantic failure; per-case
panic isolation; a non-syntax-only case that remains `Passed` while retaining
its complete ordered diagnostics without the expectation-aware filtering used
by the parse-only runner; and the exact repository/CLI totals. The exact
test-first paths are
`crates/mizar-test/src/runner/tests/syntax_smoke.rs` and
`crates/mizar-test/tests/metadata.rs`. Synthetic fixtures remain test-local and
do not change the committed corpus. They also prove exact ledger admission and
missing/extra/duplicate/reordered/stale/code-mismatch rejection. Repository and
CLI totals are 360 total / 353 passed / 7 expected / 0 failed.

Synchronize the existing paired `mizar-test` harness module design for the new
[public surface](../../mizar-test/en/harness.md#public-api),
[runner mode](../../mizar-test/en/harness.md#runner-modes),
[algorithm](../../mizar-test/en/harness.md#algorithm--logic), and
[tests](../../mizar-test/en/harness.md#tests). The crate-plan pair gains only
this contract link. `semantic_bridge_corpus_map.md` replaces its recommendation
with a link after completion; no status narrative is copied. The coverage audit
has no impact because requirement coverage, owner, and deferred status do not
change.

## Protected scope and exit

Do not edit `doc/spec`, any `.miz`, expectation sidecar, snapshot, trace row or
state, `tests/coverage/step5_activation_map.tsv`,
`tests/coverage/audit1_frontend_gaps.tsv`, or `doc/design/archive/`.
The activation-map, audit-1 gap ledger, coverage-audit, and 13-file archive
baselines are
`e9a1c2e6b0b444a2caed6a081b6c4a2ff780ee39d4e357c0268d3f8d7215a34b`,
`a0d161dedd78450110fe25e11634f9cdde5f5f633d30965b887cd70aee383dd8`,
`9e75f8ea0f7a1ca81a88f47811f21a264803b16fb3101327caaed7c0925af285`,
and `934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.
Task 277B remains not-ready/zero-credit; the 23 certificate rejection cases
and all semantic activation states remain unchanged.

Independent specification/equivalence and boundary reviews precede production
edits. Test-sufficiency, implementation, and source/docs/API reviews follow.
Verification requires focused runner/metadata/CLI tests, link/ledger lint,
`cargo fmt --all --check`, warnings-denied workspace Clippy, and full workspace
tests. Exit requires hard gates 9/9, a valid score of at least 90/100, exact
task-only staging, a local commit, clean postcommit proof, and fresh selection
of Step 5B.1.

## Completion evidence

- Outcome: public runner/API/CLI cover all 360 eligible cases as 353 `Passed`,
  seven exact `ExpectedSyntaxRejection`, and zero `Failed`; semantic credit is
  unchanged at `0 tasks/week`.
- Reviews: specification/equivalence, boundary, test sufficiency,
  implementation, and source/docs/API reviews ended with no findings after
  finding-specific re-review.
- Verification: plan `558/499/315/243/23/0`, parse-only `109/109`, focused
  syntax-smoke unit `2/2`, metadata `12/12`, lint `16/16`, format, warnings-
  denied workspace Clippy, and full workspace tests all passed.
- Hard gates: parent Sol judged 9/9 passed and quality 100/100; coverage-audit impact remains none.
  Protected hashes and the 13-file archive aggregate match their frozen values;
  no oracle, expectation, trace state, parser/cache, or specification changed.
- Deferred: four template rows remain with their existing spec-decision owner;
  Tasks 75/76/77 retain their active-range rows. Task 277B remains not-ready and
  zero-credit.
- Handoff: select Step 5B.1 next with Luna xhigh for the frozen light-tier
  structural documentation transport; return any authority, semantic-credit,
  ownership, or one-way-promotion issue to the parent Sol xhigh.
