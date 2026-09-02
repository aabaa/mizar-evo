# Task STEP5A4-G3-THEN-LINKING: omitted-justification compact statements

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A4-G3-THEN-LINKING.md](../ja/STEP5A4-G3-THEN-LINKING.md).

Owning plans: [mizar-parser](../../mizar-parser/en/00.crate_plan.md) for grammar
and [mizar-frontend](../../mizar-frontend/en/00.crate_plan.md) for the real-parser
cache namespace.

## Frozen assignment

| Field | Value |
|---|---|
| Status | Frozen; pre-implementation |
| Tier | Full: parser-visible grammar, AST/diagnostic output, parser-cache identity, and test-first `.miz` |
| Owner / consumers | `mizar-parser` owns statement recognition and `ThenStatement`/`CompactStatement`; `mizar-frontend` owns the real-parser cache-key version; later statement checking consumes the AST |
| Dependencies | Step 5A frozen order selects 5A.4 after completed 5A.3; no semantic dependency |
| Authority | Chapter [§15.9.1](../../../spec/en/15.statements.md#1591-sequential-modifier-then) and [§15.12](../../../spec/en/15.statements.md#1512-complete-syntax-ebnf); Appendix [A.15](../../../spec/en/appendix_a.grammar_summary.md#a15-statements-proofs-and-references); [Step 5A](../../todo.md#step-5a--frontend-gap-closure--) |
| Classification | `source_drift`, `design_drift`, `test_gap`; no `spec_gap` |
| Semantic-credit throughput | `0 tasks/week`; G3 closes a syntax blocker and activates no semantic oracle |

The immutable scope oracle is `tests/coverage/audit1_frontend_gaps.tsv`: select
exactly its single G3 row,
`pass_formula_statement_then_hence_linking_001.miz`. Its expectation, trace
state, and 5C.9 activation-map row remain unchanged.

## Frozen behavior and boundary

The grammar already defines `statement ::= [ "then" ] linkable_statement`,
includes `compact_statement` among linkable statements, and defines its
`justification` as an optional simple justification, a proof, or a computation
proof. The
parser therefore accepts both labelled and unlabelled compact propositions
whose justification is omitted, with or without the `then` prefix. A
`ThenStatement` owns only `then` plus one `CompactStatement` child; a
`CompactStatement` owns its proposition and semicolon and has no fabricated
justification child when the source omits one.

Existing explicit `by`, proof-block, conclusion, iterative-equality,
`then per cases`, and `then` recovery behavior remains unchanged. `then` still
cannot prefix standalone statements such as `let`, `assume`, `given`, `take`,
`set`, `deffunc`, `defpred`, `now`, or `hereby`. Missing formulas and missing
semicolons remain diagnosed and recovered locally. This task does not attach
the previous statement, desugar `hence`, check labels, verify proofs, or grant
any statement-semantic credit.

No parser or syntax public API shape, syntax kind, or diagnostic code changes.
Because identical token and parser-input bytes now produce a different AST and
diagnostic result, the existing public frontend
`MIZAR_PARSER_CACHE_KEY_VERSION` advances from v4 to v5. The token-stream key
remains v2 and the cache-key shapes/storage policy do not change.

G4/G6/G7/G9, all semantic resolution/checking, and every 5C oracle activation
are excluded.

## Test-first contract and protected surfaces

Add exactly
`tests/miz/pass/parser/pass_parser_then_linkable_omitted_justification_001.miz`
and `.expect.toml`. Freeze `schema_version = 1`, matching ID and source,
`kind = "pass"`, `stage = "parse_only"`,
`domain = "parser.then_linking"`, `expected_outcome = "pass"`,
`expected_phase = "parse"`, `diagnostic_codes = []`, and
`tags = ["active_parse_only"]`. The source contains a theorem proof with an
assumption followed by an omitted-justification labelled compact statement,
an omitted-justification labelled `then` compact statement, an explicit-`by`
`then` control, and a `hence` control. Its exact `spec_refs` are
`spec.en.15.statements.syntax`; append the sidecar backlink only to that
already-covered parse-only trace row without changing status, stage, coverage,
or requirement count.

Before production edits, the real frontend case must fail only on the omitted
compact/`then` forms. Focused Rust tests must pin AST child ownership and
ranges for labelled/unlabelled omitted forms, the explicit-justification
control, and unchanged rejection/recovery for `then let`, `then ;`, and a
missing semicolon.

Baseline is 553 cases / 499 requirements / 104 active parse-only cases; the
expected corpus delta is +1 / +0 / +1, with pass/fail 311/243. Existing `.miz`,
expectations, semantic oracle pairs, trace states, activation map, gap ledger,
coverage audit, Cargo metadata, and `doc/design/archive/` are immutable.
Protected activation-map, gap-ledger, coverage-audit, and archive-manifest
hashes are respectively
`e9a1c2e6b0b444a2caed6a081b6c4a2ff780ee39d4e357c0268d3f8d7215a34b`,
`a0d161dedd78450110fe25e11634f9cdde5f5f633d30965b887cd70aee383dd8`,
`9e75f8ea0f7a1ca81a88f47811f21a264803b16fb3101327caaed7c0925af285`,
and `934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.

Implementation scope is limited to parser statement recognition and focused
tests; the frontend parser-cache version and focused tests; the new parse-only
pair and count assertions; one append-only trace backlink; paired parser and
frontend owner docs; this contract; and concise live Step 5 indexes.
`doc/design/spec_coverage_audit.md` has no impact because its coverage status,
owner, and deferred rationale do not change.

## Gates and exit

Independent specification/equivalence and boundary reviews precede
implementation. Then run test-sufficiency, implementation, and source/docs/API
reviews; focused parser/frontend/parse-only checks; metadata/link/ledger lints;
format, warnings-denied workspace Clippy, and full tests. Exit requires 9/9
hard gates, a valid read-only score of at least 90/100, exact task-only staging,
local commit, clean postcommit proof, and fresh selection of 5A.5/G4.
