# Task STEP5A9-CORPUS-SYNTAX-SMOKE: corpus-wide syntax guard

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A9-CORPUS-SYNTAX-SMOKE.md](../ja/STEP5A9-CORPUS-SYNTAX-SMOKE.md).

Owning plan: [mizar-test](../../mizar-test/en/00.crate_plan.md).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Frozen for test-first implementation |
| Tier | Full: production runner, CLI, and public Rust API |
| Owner / consumers | `mizar-test` owns corpus selection, frontend execution, and the report; `mizar-frontend` and `mizar-parser` remain unchanged providers |
| Dependencies | Step 5A.1-5A.8 complete; all G1-G9 frontend gaps are closed |
| Authority | [Step 5A.9](../../todo.md#step-5a--frontend-gap-closure--) and the audit-1 [corpus map](../../mizar-test/en/semantic_bridge_corpus_map.md#follow-ups) / [frontend method](../../mizar-test/en/semantic_bridge_frontend_gaps.md#verification-method) |
| Classification | `test_gap` and bounded `design_drift`; no specification gap |
| Semantic-credit throughput | `0 tasks/week`; this guard activates no semantic oracle |

## Selection and acceptance

`run_syntax_smoke_corpus` shall clone the supplied `DiscoveryConfig`, force
`TestProfile::Full`, and build the ordinary fail-closed plan. It selects every
case whose source extension is exactly `.miz` and whose expectation stage is
not `parse_only`. It does not inspect active tags, expected outcome, expected
phase, diagnostic expectations, or activation-map membership. Canonical plan
order is retained. Validation errors carried by a successfully built plan
return no case results and retain the plan diagnostics. An infrastructure
`HarnessError` from plan construction remains the public `Err` and CLI exit 2.

Each selected source runs once through the existing real `Frontend` with
`MizarParserSeam` and the existing syntax-only import provider. As in the
audit-1 harness, each call is panic-isolated so one source cannot abort the
remaining inventory. A case passes only if execution does not panic, execution
succeeds, a `SurfaceAst` exists, and the complete frontend diagnostic list is
empty. Panics, frontend errors, missing ASTs, syntax diagnostics, and other
frontend diagnostics fail closed. The report records stable diagnostic-code
strings, using `frontend_panic`, `missing_ast`, or the existing
`frontend_error:<message>` form where applicable; it never consults or changes
the case's semantic oracle.

At freeze time the repository has 473 `.miz` cases: 113 `parse_only` and 360
eligible smoke cases. All currently use the default fast profile, but forcing
`Full` prevents later non-fast corpus additions from silently escaping the
guard. Successful repository execution is exactly 360 passed / 0 failed.

## API, tests, and documentation

Add public `SyntaxSmokeRunReport`, `SyntaxSmokeCaseResult`, and non-exhaustive
`SyntaxSmokeCaseStatus`, plus `syntax_smoke_cases` and
`run_syntax_smoke_corpus`. Add the `syntax-smoke` CLI command with the same
summary/exit convention as existing runners. Keep execution in a private
`runner/syntax_smoke.rs` leaf and reuse the shared frontend path; do not add a
new parser, import provider, expectation field, tag, stage, or compatibility
adapter.

Before production changes, add failing tests that require: exact selection of
non-`parse_only` `.miz` across profiles without active tags; exclusion of
`parse_only` and non-`.miz` sources; deterministic order/replay; pass only with
an AST and zero diagnostics; rejection of malformed syntax even when its
sidecar is a later semantic failure; per-case panic isolation; and
complete ordered projection of non-syntax frontend diagnostics without the
expectation-aware filtering used by the parse-only runner; and repository/CLI
totals of 360/360. The exact test-first paths are
`crates/mizar-test/src/runner/tests/syntax_smoke.rs` and
`crates/mizar-test/tests/metadata.rs`. Synthetic fixtures remain test-local and
do not change the committed corpus.

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

Pending implementation and full-tier verification.
