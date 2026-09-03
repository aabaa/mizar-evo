# Task STEP5A7-G9-ACTIVE-PATTERN-SPELLINGS: active notation in patterns

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A7-G9-ACTIVE-PATTERN-SPELLINGS.md](../ja/STEP5A7-G9-ACTIVE-PATTERN-SPELLINGS.md).

Owning plan: [mizar-parser](../../mizar-parser/en/00.crate_plan.md).

## Frozen assignment

| Field | Value |
|---|---|
| Status | In progress; contract frozen before test edits |
| Tier | Full: test-first `.miz`, expectation metadata, and trace backlink |
| Owner / consumers | `mizar-parser` owns syntactic pattern admission; `mizar-frontend` supplies position-sensitive local `UserSymbol` tokens; `mizar-test` runs the parse-only regression |
| Dependencies | Completed 5A.2/G1 and 5A.5/G4; selected after completed 5A.6 by the frozen order |
| Authority | Chapter [§9.1](../../../spec/en/09.predicates.md#91-overview-and-syntax), [§9.6](../../../spec/en/09.predicates.md#96-predicate-redefinition), and [§9.10](../../../spec/en/09.predicates.md#910-complete-syntax-ebnf); Chapter [§11.1](../../../spec/en/11.symbol_management.md#111-synonyms-and-antonyms), [§11.2](../../../spec/en/11.symbol_management.md#112-scope-and-visibility), and [§11.6](../../../spec/en/11.symbol_management.md#116-complete-syntax-ebnf); Appendix [A.9](../../../spec/en/appendix_a.grammar_summary.md#a9-predicates) and [A.11](../../../spec/en/appendix_a.grammar_summary.md#a11-symbol-management); [Step 5A](../../todo.md#step-5a--frontend-gap-closure--) |
| Classification | Historical `source_drift` already closed by dependencies; remaining `test_gap` and roadmap `design_drift`; no `spec_gap` |
| Semantic-credit throughput | `0 tasks/week`; G9 closes syntax evidence and activates no semantic oracle |

The immutable scope oracle is `tests/coverage/audit1_frontend_gaps.tsv`:
select exactly its three G9 rows,
`fail_type_elaboration_pred_duplicate_same_signature_001.miz`,
`fail_type_elaboration_synonym_loci_mismatch_001.miz`, and
`pass_type_elaboration_pred_redefine_narrower_loci_001.miz`. Their source,
expectations, trace states, and respective 5C.5/5C.6 activation-map rows remain
unchanged.

## Frozen behavior and boundary

An earlier completed declaration makes its notation spelling lexically active
only after the declaring item. When that spelling occurs in a later predicate
definition or predicate redefinition pattern, the frontend transfers it as the
existing generic `UserSymbol`; `pred_pattern` accepts that token in its existing
single predicate-symbol slot. A notation alias preserves its original pattern
as raw source-order `NotationPattern` tokens, including an earlier active
functor or predicate spelling. The parser does not require an `Identifier` at
any of these already-active positions.

Fresh read-only execution at selection HEAD `7c644c5f` proves all three exact
G9 sources already produce an AST with no syntax diagnostics. The later
`duppred`, `eqv`, and `synbase2` spellings are `UserSymbol` tokens. The parser's
predicate matcher has admitted `Identifier | UserSymbol | LexemeRun` since its
existing pattern implementation, raw notation patterns are already
token-kind-neutral, 5A.2 supplies direct same-module activation, and 5A.5
supplies original-pattern alias classification. Therefore no production Rust,
AST kind, diagnostic, recovery, public API, token-cache, or parser-cache change
is authorized. A newly failing test would reopen the contract for parent-owned
classification rather than justify widening this frozen scope.

Resolution of duplicate definitions, redefinition target/coherence checking,
alias kind or loci compatibility, overload selection, and all semantic
diagnostics remain with their 5C owners. This task neither accepts nor rejects
those meanings and grants no semantic credit.

## Test-first contract and protected surfaces

Add exactly
`tests/miz/pass/parser/pass_parser_active_pattern_spellings_001.miz` and its
`.expect.toml`. Freeze `schema_version = 1`, matching ID/source,
`kind = "pass"`, `stage = "parse_only"`,
`domain = "parser.active_pattern_spelling"`, `expected_outcome = "pass"`,
`expected_phase = "parse"`, `diagnostic_codes = []`, and
`tags = ["active_parse_only"]`. The source covers a second predicate definition
of the same spelling, a predicate redefinition using that spelling, and a
synonym whose original functor pattern uses an earlier spelling. Explicit proof
coherence avoids G7. Its sole `spec_refs` entry is
`spec.en.syntax.redefinition_notation.parser`; append only this backlink to the
existing trace row without changing status, stage, coverage, or requirement
count.

Add one real-frontend regression over the three byte-exact immutable G9 sources.
It must assert AST presence, empty diagnostics, v6 parser identity, deterministic
replay, and the expected later `UserSymbol` spelling. Existing parser unit tests
remain the lower-level evidence for active `UserSymbol` predicate patterns and
raw `NotationPattern` preservation; do not duplicate their matrices.

Baseline is 556 cases / 499 requirements / 107 active parse-only cases;
expected corpus delta is +1 / +0 / +1, with pass/fail 314/243. Existing `.miz`,
expectations, semantic oracle pairs, trace states, activation map, gap ledger,
coverage audit, Cargo metadata, and `doc/design/archive/` are immutable. The
protected activation-map, gap-ledger, coverage-audit, and 13-file archive hashes
are respectively
`e9a1c2e6b0b444a2caed6a081b6c4a2ff780ee39d4e357c0268d3f8d7215a34b`,
`a0d161dedd78450110fe25e11634f9cdde5f5f633d30965b887cd70aee383dd8`,
`9e75f8ea0f7a1ca81a88f47811f21a264803b16fb3101327caaed7c0925af285`,
and `934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.

Implementation scope is limited to the new parse-only pair, its count and exact
manifest assertions, one append-only trace backlink, the byte-exact frontend
regression, this contract, and concise Step 5 indexes. The existing parser
grammar document already owns the durable `UserSymbol` and raw-pattern rules,
so it remains unchanged. `doc/design/spec_coverage_audit.md` has no impact
because coverage status, owner, and deferred rationale do not change.

## Gates and exit

Independent specification/equivalence and boundary reviews precede test edits.
Then run test-sufficiency, implementation, and source/docs/API reviews; focused
frontend/parser/parse-only checks; metadata/link/ledger lints; format,
warnings-denied workspace Clippy, and full tests. Exit requires 9/9 hard gates,
a valid read-only score of at least 90/100, exact task-only staging, local
commit, clean postcommit proof, and the required 5A.8 specification-decision
stop with analysis, options, and a parent recommendation.

## Completion evidence

Pending.
