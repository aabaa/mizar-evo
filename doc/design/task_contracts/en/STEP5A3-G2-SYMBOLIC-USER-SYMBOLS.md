# Task STEP5A3-G2-SYMBOLIC-USER-SYMBOLS: declaration-site tokenization

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A3-G2-SYMBOLIC-USER-SYMBOLS.md](../ja/STEP5A3-G2-SYMBOLIC-USER-SYMBOLS.md).

Owning plans: [mizar-lexer](../../mizar-lexer/en/00.crate_plan.md) and
[mizar-frontend](../../mizar-frontend/en/00.crate_plan.md).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Frozen; pre-implementation |
| Tier | Full: production tokenization, parser-visible behavior, token-cache identity, and test-first `.miz` |
| Owner / consumers | `mizar-lexer` owns final disambiguation; `mizar-frontend` owns the token-cache namespace; `mizar-parser` consumes the result |
| Dependencies | Step 5A frozen order selects 5A.3 after completed 5A.2; no semantic dependency |
| Authority | Ch. [§2.3](../../../spec/en/02.lexical_structure.md#23-tokens-and-lexicon), [§2.5.1](../../../spec/en/02.lexical_structure.md#251-reserved-special-symbols), and [§2.5.2](../../../spec/en/02.lexical_structure.md#252-user-defined-symbolic-names); Appendix [A.2](../../../spec/en/appendix_a.grammar_summary.md#a2-lexical-structure); and [Step 5A](../../todo.md#step-5a--frontend-gap-closure--) |
| Classification | `source_drift`, `test_gap`; no `spec_gap` |
| Semantic-credit throughput | `0 tasks/week`; G2 closes a syntax blocker and activates no semantic oracle |

The immutable scope oracle is `tests/coverage/audit1_frontend_gaps.tsv`: select
exactly the three rows containing token `G2` (two G1+G2 functor rows and one
G2-only predicate row). G1 is already closed by 5A.2. Only G2 lexer
diagnostics are removed; independently observed G7 parser diagnostics may
remain until 5A.8.

## Frozen behavior and boundary

At the exact source start of a collected local functor or predicate notation
declaration, a punctuation-shaped spelling that has no ordinary token form is
admitted as that declaration's `UserSymbol` token. The already collected
`declared_at` span is the sole authority for this exception; it does not make
the spelling generally active. Normal activation still begins only after the
whole declaration item and its correctness/property trail completes.

The declaration-site candidate participates in existing longest-match and
parser-context filtering. This admits `\+\`, `<<=`, `<+>`, and all other
spec-valid punctuation-shaped functor/predicate spellings, including spellings
that contain a reserved-symbol prefix. Identifier-shaped declarations remain
identifiers at their own declaration site. Constructor kinds, reserved-word or
reserved-symbol collisions, `@`, pre-declaration uses, self-use in a definiens,
and unrelated unknown punctuation remain rejected or tokenized by their
existing rules. Resolver identity, overload choice, types, proofs, and
semantics remain downstream-owned.

No public API shape is added or removed. The existing crate-private
declaration-start query is reused. Because identical source and imported
environment inputs now produce a different token stream, the value of the
existing public frontend `TOKEN_STREAM_CACHE_KEY_VERSION` advances from v1 to
v2; the parser seam version remains v4 because AST reuse is already separated
by the changed token key.

G3/G4/G6/G7/G9, synonym/antonym grammar acceptance, semantic resolution, and
all 5C oracle activation are excluded.

## Test-first contract and protected surfaces

Add exactly
`tests/miz/pass/parser/pass_parser_symbolic_user_symbol_declarations_001.miz`
and `.expect.toml`. Freeze `schema_version = 1`, matching ID and source,
`kind = "pass"`, `stage = "parse_only"`,
`domain = "parser.symbolic_user_symbols"`, `expected_outcome = "pass"`,
`expected_phase = "parse"`, `diagnostic_codes = []`, and
`tags = ["active_parse_only"]`. The source has exactly two symbolic infix
functor declarations (`\+\` and `<+>`) and one symbolic infix predicate
declaration (`<<=`), followed by later uses of all three; correctness
conditions use explicit proof blocks. Its exact `spec_refs` are
`spec.en.09.predicate_definitions.parser` and
`spec.en.10.functor_definitions.parser`; append this backlink to those
already-covered parse-only trace rows without changing status, stage, coverage,
or requirement count. Focused lexer tests, rather than a cross-stage manifest
link, bind the Chapter 2 declaration-site tokenization rule.

Before production edits, the real frontend case must fail with
`NoValidTokenCandidate` at the declaration spellings. Focused Rust tests must
pin exact declaration-start admission, longest matching over reserved prefixes,
identifier-shaped non-reclassification, no forward/self activation, collision
and `@` exclusion, parser-context rejection, token-cache v2, and the exact
three-row G2 ledger selection.

Baseline is 552 cases / 499 requirements / 103 active parse-only cases; the
expected corpus delta is +1 / +0 / +1. Existing `.miz`, expectations, semantic
oracle pairs, trace states, activation map, gap ledger, coverage audit, Cargo
metadata, and `doc/design/archive/` are immutable. Protected activation-map,
gap-ledger, coverage-audit, and archive-manifest hashes are respectively
`e9a1c2e6b0b444a2caed6a081b6c4a2ff780ee39d4e357c0268d3f8d7215a34b`,
`a0d161dedd78450110fe25e11634f9cdde5f5f633d30965b887cd70aee383dd8`,
`9e75f8ea0f7a1ca81a88f47811f21a264803b16fb3101327caaed7c0925af285`,
and `934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.

Implementation scope is limited to lexer disambiguation and focused tests,
the frontend token-cache version and focused test, the new parse-only pair and
count assertions, append-only trace backlinks, paired lexer/frontend owner
docs, this contract, and concise live Step 5 indexes.
`doc/design/spec_coverage_audit.md` has no impact because no coverage status,
owner, or deferred rationale changes.

## Gates and exit

Independent spec/equivalence and boundary reviews precede implementation.
Then run test-sufficiency, implementation, and source/docs/API reviews;
focused lexer/frontend/parse-only checks; metadata/link/ledger lints; format,
warnings-denied workspace Clippy, and full tests. Exit requires 9/9 hard gates,
a valid read-only score of at least 90/100, exact task-only staging, local
commit, clean postcommit proof, and fresh selection of 5A.4/G3.
