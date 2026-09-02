# Task STEP5A2-G1-LOCAL-NOTATION: same-module notation activation

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A2-G1-LOCAL-NOTATION.md](../ja/STEP5A2-G1-LOCAL-NOTATION.md).

Owning plans: [mizar-frontend](../../mizar-frontend/en/00.crate_plan.md) and
[mizar-lexer](../../mizar-lexer/en/00.crate_plan.md).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Completed; implementation and full-gate evidence recorded below |
| Tier | Full: production lexing/parser-input behavior, public lexer DTO, parser AST/cache semantics, and test-first `.miz` |
| Owner / consumers | `mizar-lexer` local declaration prepass and `mizar-frontend::ParserInputs` / `mizar-parser`, registrations, later semantic stages |
| Dependencies | None; Step 5A frozen order selects 5A.2 after completed 5A.1 |
| Authority | Ch. [2](../../../spec/en/02.lexical_structure.md), [§9.3](../../../spec/en/09.predicates.md#93-definition-styles-symbolic-vs-phrase), [§10.8–10.9](../../../spec/en/10.functors.md#108-notation-styles), [§13.2](../../../spec/en/13.term_expression.md#132-functor-applications), [§15.2.3–15.2.4](../../../spec/en/15.statements.md), [§17.5–17.6](../../../spec/en/17.clusters_and_registrations.md), [§18.2](../../../spec/en/18.templates.md#182-template-declarations)/[§18.7](../../../spec/en/18.templates.md#187-template-for-functors-func), Appendix [A.13](../../../spec/en/appendix_a.grammar_summary.md#a13-term-expressions)/[B.2](../../../spec/en/appendix_b.operator_precedence.md#b2-term-operators), and [Step 5A](../../todo.md#step-5a--frontend-gap-closure--) |
| Classification | `source_drift`, `test_gap`, bounded `design_drift`; no `spec_gap` |
| Semantic-credit throughput | `0 tasks/week`; 5A.2 closes syntax blockers but activates no semantic oracle |

The immutable scope oracle is `tests/coverage/audit1_frontend_gaps.tsv`: select
exactly the 20 rows whose comma-delimited gaps field contains token `G1` (17
G1-only, two G1+G2, one G1+G6). The roadmap's count 19 is `design_drift`, not
permission to delete, reorder, infer, or repair a row. G2/G6 ownership stays
deferred; G7 is not recorded as an overlap in this ledger and its independently
observed empty-justification question stays reserved for 5A.8. Only G1 failures
are removed; mixed rows need not become wholly diagnostic-free here.

## Frozen behavior and boundary

The lexer selects identifier notation loci from preceding definition `let`
bindings, including uppercase one-letter loci, before using its existing
conservative fallback. A completed local `func` declaration publishes its
parser-facing default shape at the same activation point as its user-symbol
candidate: prefix, postfix, or non-associative infix at precedence 64. Template
loci do not count as term operands, and each occupied side must contain exactly
one term locus; the existing one-character lowercase fallback applies when no
preceding `let` inventory is available. Functional, nullary, multi-locus, and
circumfix shapes do not synthesize a Pratt entry. A local `pred` publishes only
the correctly selected predicate spelling. A later explicit operator
declaration continues to supersede an earlier default by source-position
ordering.

`LocalUserSymbolDeclaration` gains exactly
`operator: Option<ExportedOperatorMetadata>`; its candidate and range-aware
operator queries preserve the value. No resolver identity, type, overload-root,
proof, registration, or semantic activation enters lexer/frontend state.
`ParserInputs` remains the sole frontend adapter and adds no API. The parser
continues to invent no fixity: when a supplied prefix entry is followed by
template arguments, it preserves `TemplateArguments` between the operator token
and its term operand. A bracket group is treated as those arguments only when a
term operand follows it; otherwise it remains the built-in bracket-functor
operand. Within a parser-recognized template definition only, a
grammar-required quantified-binder type may preserve one bare `Identifier`
`TypeHead` after strict type parsing fails; this is structural preservation,
not binding or symbol classification. These changes affect real AST output, so
`MIZAR_PARSER_CACHE_KEY_VERSION` advances from v3 to v4.

Inline `deffunc`/`defpred` names remain ordinary proof-local identifiers and use
the existing parenthesized inline-call grammar; they are regression-covered but
do not become lexer user symbols. Empty compact/correctness justifications are
out of scope and remain owned by 5A.8. G2 symbolic token formation, G3/G4/G6/G9,
semantic resolution, and all 5C oracle activation are excluded.

## Test-first contract and protected surfaces

Add exactly `tests/miz/pass/parser/pass_parser_local_notation_activation_001.miz`
and `.expect.toml`. The sidecar freezes `schema_version = 1`, the matching ID
and source, `kind = "pass"`, `stage = "parse_only"`,
`domain = "parser.local_notation"`, `expected_outcome = "pass"`,
`expected_phase = "parse"`, `diagnostic_codes = []`, and
`tags = ["active_parse_only"]`. It covers uppercase-locus phrase predicates,
identifier prefix/infix functors, a prefix template functor with a trailing
term locus, proof-local `deffunc`/`defpred` calls, and prefix functor heads in
functorial/reduce registrations; all correctness conditions use explicit proof
blocks so 5A.8 is not pre-decided. Before production edits, the focused real
frontend run must fail at later local uses; afterward it must have no syntax
diagnostic. Its exact `spec_refs` are `spec.en.09.predicate_definitions.parser`,
`spec.en.10.functor_definitions.parser`, `spec.en.13.functor_applications.parser`,
`spec.en.13.operator_precedence.parser`, `spec.en.14.atomic_formula.parser`,
`spec.en.15.statements.syntax`, `spec.en.17.clusters_and_registrations.parser`,
and `spec.en.syntax.template_arguments.parser`. Trace edits append only this
sidecar to those eight already-covered parse-only rows; status, stage, coverage,
and requirement count stay unchanged.

Focused Rust tests must pin binder-aware spelling selection, default metadata
activation/no-forward-reference, explicit-metadata precedence, template-prefix
AST children, bounded template-local binder type fallback, frontend
source-coordinate mapping, cache-key versioning, and the 20-row G1 inventory
classification. Existing `.miz`, expectations, semantic
oracle pairs, trace states, activation map, gap ledger, coverage audit, Cargo
metadata, and `doc/design/archive/` are immutable.

Implementation scope is limited to the lexer local-declaration producer,
frontend parser-input/cache-version adapter, parser prefix and template-local
required-binder-type consumers, their focused tests, the new parse-only pair
and count assertions, trace backlinks, the paired owner documents, this
contract, and concise live Step 5 indexes.
`doc/design/spec_coverage_audit.md` has no impact because no requirement's
coverage status, owner, or deferred rationale changes.

## Gates and exit

Baseline is 551 cases / 499 requirements / 102 active parse-only cases; the
expected corpus delta is +1 / +0 / +1. Protected activation-map, gap-ledger,
coverage-audit, and archive-manifest hashes are respectively
`e9a1c2e6b0b444a2caed6a081b6c4a2ff780ee39d4e357c0268d3f8d7215a34b`,
`a0d161dedd78450110fe25e11634f9cdde5f5f633d30965b887cd70aee383dd8`,
`9e75f8ea0f7a1ca81a88f47811f21a264803b16fb3101327caaed7c0925af285`,
and `934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.

Independent spec/equivalence and API/boundary reviews precede implementation.
Then run test-sufficiency, implementation, and source/docs/API reviews; focused
lexer/parser/frontend/parse-only checks; metadata/link/ledger lints; fmt,
warnings-denied workspace Clippy, and full tests. Exit requires 9/9 hard gates,
a valid read-only score of at least 90/100, exact task-only staging, local
commit, clean postcommit proof, and fresh selection of 5A.3/G2.

## Completion evidence

The exact 20-row read-only probe reports no remaining G1 diagnostic; the two
G1+G2 rows retain only G2, the G1+G6 row retains only G6, and independently
observed G7 diagnostics remain with 5A.8. The new corpus case passes and raises
the totals to 552 cases / 499 requirements / 103 active parse-only cases.
Independent spec/equivalence, boundary, test-sufficiency, implementation, and
source/docs/API reviews ended with no findings after finding-specific repairs.

Focused crate and corpus checks, metadata/link/ledger lints, `cargo fmt --check`,
warnings-denied workspace Clippy, full `cargo test`, and the read-only corpus
plan pass; the plan retains the expected 23 baseline warnings. The parent
records 9/9 hard gates passed. No semantic oracle or trace state was activated,
and all four protected hashes above remain unchanged. The final independent
read-only review applies no cap and assigns `100/100`
(`20/20/15/15/10/10/5/5`). The local commit identity remains owned by Git
history rather than duplicated in another status owner.
