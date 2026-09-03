# Task STEP5A6-G6-DEPENDENT-MODE-USE: local dependent-mode type arguments

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A6-G6-DEPENDENT-MODE-USE.md](../ja/STEP5A6-G6-DEPENDENT-MODE-USE.md).

Owning plans: [mizar-parser](../../mizar-parser/en/00.crate_plan.md) for the
type-argument follow boundary and
[mizar-frontend](../../mizar-frontend/en/00.crate_plan.md) for real-parser
cache identity.

## Frozen assignment

| Field | Value |
|---|---|
| Status | Frozen for test-first implementation; no production edit yet |
| Tier | Full: parser-visible AST/diagnostic behavior, parser-cache identity, and test-first `.miz` |
| Owner / consumers | `mizar-parser` owns unbracketed type-argument parsing; `mizar-frontend` owns the real-parser cache namespace; resolver/checker consume the unchanged AST vocabulary |
| Dependencies | Completed 5A.5; the Step 5A frozen order selects 5A.6 next |
| Authority | Chapter [§3.3](../../../spec/en/03.type_system.md#33-type-expressions), [§7.2](../../../spec/en/07.modes.md#72-syntax-for-declaring-and-using-modes), [§7.7](../../../spec/en/07.modes.md#77-dependent-modes), and [§7.10](../../../spec/en/07.modes.md#710-complete-syntax-ebnf); Appendix [A.3](../../../spec/en/appendix_a.grammar_summary.md#a3-type-expressions), [A.10](../../../spec/en/appendix_a.grammar_summary.md#a10-functors-and-operator-declarations), and [A.15](../../../spec/en/appendix_a.grammar_summary.md#a15-statements-proofs-and-references); [Step 5A](../../todo.md#step-5a--frontend-gap-closure--) |
| Classification | `source_drift`, `design_drift`, `test_gap`; no `spec_gap` |
| Semantic-credit throughput | `0 tasks/week`; G6 closes a syntax blocker and activates no semantic oracle |

The immutable scope oracle is `tests/coverage/audit1_frontend_gaps.tsv`:
select exactly its two G6 rows,
`pass_type_elaboration_mode_dependent_of_params_001.miz` and
`pass_type_elaboration_func_dependent_return_type_001.miz`. Their
expectations, trace states, and respective 5C.4/5C.5 activation-map rows remain
unchanged.

## Frozen behavior and boundary

The canonical type grammar permits an active local dependent-mode head followed
by `of` or `over` and one or more comma-separated term arguments wherever a
`type_expression` occurs. The frontend lexical environment classifies the
active local mode, but its parser transfer is the intentionally generic
`UserSymbol`; the parser remains mode-identity agnostic. It already builds the
correct `TypeHead` and parses the first argument. It must finish the unbracketed
`TypeArguments` node without consuming or diagnosing the outer grammar token
that follows that completed argument.

Add one private unbracketed-type-argument follow helper, used both at the list
loop entry and after a completed argument. Its follow set is the existing
term-argument-list boundary plus outer `by`, `proof`, `equals`, `means`, `st`,
`holds`, `->`, and formula-connective tokens, all of which cannot belong to an
unparenthesized term argument. Do not widen the generic term-list helper.
Comma remains an argument separator rather than a follow token. The existing
comprehension-generator comma rule and structure-constructor field-list rule
remain their more specific callers. The exact G6 sources must therefore retain
`holds` and `means` for their enclosing productions and parse without
diagnostics.

Malformed argument lists remain fail-closed: missing first/trailing arguments,
missing commas, unmatched bracket forms, and malformed terms keep their
existing diagnostics and recovery nodes. Multi-argument `of`/`over`, bracket
type arguments, local mode activation, qualified names, `qua`, attributes,
term precedence, and all non-type term lists remain unchanged. Any imported
head already transferred as syntactic `UserSymbol` necessarily shares this
parser follow rule. Task 68's same-module argument-bearing local-mode reserve
provenance and resolver/checker payload-extraction boundary remains unchanged;
imported mode provenance remains separately owned by Tasks 79/82. Neither earns
credit here. The parser does not validate type-argument arity, resolve a mode
identity, type-check arguments, expand a mode, or grant semantic credit.

No syntax kind, AST/public API shape, or diagnostic vocabulary is added. Since
identical tokens and parser inputs now produce a different AST/diagnostic
result, `MIZAR_PARSER_CACHE_KEY_VERSION` advances from v5 to v6. The v3 token
key and cache-key shapes/storage policy remain unchanged.

## Test-first contract and protected surfaces

Add exactly
`tests/miz/pass/parser/pass_parser_local_dependent_mode_use_001.miz` and
`.expect.toml`. Freeze `schema_version = 1`, matching ID and source,
`kind = "pass"`, `stage = "parse_only"`,
`domain = "parser.dependent_mode"`, `expected_outcome = "pass"`,
`expected_phase = "parse"`, `diagnostic_codes = []`, and
`tags = ["active_parse_only"]`. The source declares one local dependent mode,
uses it in a quantified binder and proof-local `let`, and uses it as a
functor return type before `means`; explicit equality bodies and the already
accepted empty existence/uniqueness forms avoid unrelated gaps. Its sole
`spec_refs` entry is `spec.en.03.type_expressions.parser`; append only this
parse-only backlink to that trace row without changing its status, stage,
coverage, or requirement count.

Before production edits, the new real-frontend case and both byte-exact G6
sources must fail on the outer `holds`/`means` boundaries. Focused parser tests
must prove exact `TypeArguments` ownership and preservation of the outer token
for valid enclosing contexts using each frozen follow class: `by`, `proof`,
`->`, `equals`, `means`, `st`, `holds`, and each formula-connective spelling.
They also prove unchanged multi-argument parsing, comprehension/field special
cases, and fail-closed missing-comma/missing-argument recovery. Frontend tests
must prove v6 and reject the historical v5 namespace.

Baseline is 555 cases / 499 requirements / 106 active parse-only cases; expected
delta is +1 / +0 / +1, with pass/fail 313/243. Existing `.miz`, expectations,
semantic oracle pairs, trace states, activation map, gap ledger, coverage audit,
Cargo metadata, and `doc/design/archive/` are immutable. Protected activation,
gap-ledger, coverage-audit, and archive hashes are respectively
`e9a1c2e6b0b444a2caed6a081b6c4a2ff780ee39d4e357c0268d3f8d7215a34b`,
`a0d161dedd78450110fe25e11634f9cdde5f5f633d30965b887cd70aee383dd8`,
`9e75f8ea0f7a1ca81a88f47811f21a264803b16fb3101327caaed7c0925af285`,
and `934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.

Implementation scope is limited to the parser's unbracketed type-argument
follow rule and focused tests; the frontend parser-cache constant and focused
tests; the new parse-only pair and count assertions; one append-only trace
backlink; paired parser/frontend owner docs; the G6 inventory status; this
contract; and concise live Step 5 indexes. The coverage audit has no impact
because coverage status, owner, and deferred rationale do not change.

## Gates and exit

Independent specification/equivalence and boundary reviews precede
implementation. Then run test-sufficiency, implementation, and source/docs/API
reviews; focused parser/frontend/parse-only checks; metadata/link/ledger lints;
format, warnings-denied workspace Clippy, and full tests. Exit requires 9/9
hard gates, a valid read-only score of at least 90/100, exact task-only staging,
local commit, clean postcommit proof, and fresh selection of 5A.7/G9.
