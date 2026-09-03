# Task STEP5A5-G4-NOTATION-ALIASES: local notation alias activation

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A5-G4-NOTATION-ALIASES.md](../ja/STEP5A5-G4-NOTATION-ALIASES.md).

Owning plans: [mizar-lexer](../../mizar-lexer/en/00.crate_plan.md) for the
local declaration producer, [mizar-parser](../../mizar-parser/en/00.crate_plan.md)
for the existing notation grammar, and
[mizar-frontend](../../mizar-frontend/en/00.crate_plan.md) for token-cache
identity.

## Frozen assignment

| Field | Value |
|---|---|
| Status | Frozen for test-first implementation; no production edit yet |
| Tier | Full: parser-visible tokenization, AST/diagnostic output, token-cache identity, and test-first `.miz` |
| Owner / consumers | `mizar-lexer` owns local alias spelling, kind, arity, fixity, and activation; `mizar-parser` consumes the tokens through its existing `NotationAlias` grammar; `mizar-frontend` owns token-cache identity |
| Dependencies | Completed 5A.3/G2; Step 5A frozen order selects 5A.5 after completed 5A.4 |
| Authority | Chapter [§11.1](../../../spec/en/11.symbol_management.md#111-synonyms-and-antonyms), [§11.2](../../../spec/en/11.symbol_management.md#112-scope-and-visibility), and [§11.6](../../../spec/en/11.symbol_management.md#116-complete-syntax-ebnf); Appendix [A.11](../../../spec/en/appendix_a.grammar_summary.md#a11-symbol-management) and [A.12](../../../spec/en/appendix_a.grammar_summary.md#a12-modules-and-namespaces); [Step 5A](../../todo.md#step-5a--frontend-gap-closure--) |
| Classification | `source_drift`, `design_drift`, `test_gap`; no `spec_gap` |
| Semantic-credit throughput | `0 tasks/week`; G4 closes a syntax blocker and activates no semantic oracle |

The immutable scope oracle is `tests/coverage/audit1_frontend_gaps.tsv`: select
exactly its three G4 rows,
`fail_type_elaboration_synonym_loci_mismatch_001.miz`,
`pass_type_elaboration_antonym_predicate_001.miz`, and
`pass_type_elaboration_synonym_functor_001.miz`. Their expectations, trace
states, and 5C.6 activation-map rows remain unchanged.

## Frozen behavior and boundary

The normative grammar accepts predicate and functor `synonym`/`antonym`
patterns with arbitrary user notation. A current-module alias item may refer
to an earlier active current-module original notation. The lexer must exclude
the enclosing definition parameters from both pattern spelling selections,
then match the selected original spelling and original-pattern arity to local
declarations whose `activation_start` is no later than the alias keyword.
Every distinct alias-capable kind among exact matches is inherited as a
separate lexical candidate; multiple same-kind overload shapes collapse only
for lexical classification. Thus a same-spelling/arity predicate/functor
overload preserves both kinds and defers selection. This does not select or
merge semantic overload identities.

When there is no exact active local match, the collector preserves its existing
syntactic fallback: operator-like notation in either pattern classifies the
alias as a functor, otherwise it remains constructor-mode shaped. The collector
does not consult imported summaries, so word-only imported predicate/functor
originals remain unsupported; punctuation-shaped imported originals retain
only the pre-existing syntactic fallback. No imported alias behavior gains
credit in this task.

Only the alternative spelling is registered. Its arity comes from the
alternative pattern. For functors, the existing default-fixity rule counts
top-level definition parameters or locus-shaped operand words on each side of
the alternative spelling: exactly `(0,1)` is prefix, `(1,0)` postfix, and
`(1,1)` non-associative infix; every other shape has no default operator
metadata. The fixed default precedence remains `64`.

This admits identifier-shaped predicate patterns such as
`X closeto Y for X apartfrom Y` and identifier- or punctuation-shaped functor
patterns such as `X synalt Y for X synbase Y` and
`X <+> Y for X \+\ Y`. It never registers loci or the original pattern as a
new alias. `activation_start` is exactly the alias semicolon's source-span end;
the G2 declaration-site exception may classify the alternative's own
`declared_at` span but does not activate it elsewhere before that boundary.
Existing mode/attribute constructor aliases, reserved-word rejection, direct
declaration activation, imported lexical summaries, lexical/source coordinate
mapping, and forward-reference rejection remain unchanged.

The parser's existing raw `NotationPattern`/`NotationAlias` tree is the correct
consumer and needs no grammar, syntax-kind, recovery, or public API change.
The Step 5 roadmap's parser-only owner label is therefore `design_drift`: the
observed producer defect belongs to `mizar-lexer`. Semantic alias identity,
equivalence/negation, loci compatibility, overload selection, export/import
propagation, diagnostics, and all checking remain excluded.

Because the same source and imported environment can now produce different
tokens after a local alias item, `TOKEN_STREAM_CACHE_KEY_VERSION` advances
from v2 to v3. `MIZAR_PARSER_CACHE_KEY_VERSION` remains v5 because identical
tokens and parser inputs retain the same parser behavior. Cache-key shapes and
storage policy do not change.

## Test-first contract and protected surfaces

Add exactly
`tests/miz/pass/parser/pass_parser_notation_alias_activation_001.miz` and
`.expect.toml`. Freeze `schema_version = 1`, matching ID and source,
`kind = "pass"`, `stage = "parse_only"`,
`domain = "parser.notation_alias"`, `expected_outcome = "pass"`,
`expected_phase = "parse"`, `diagnostic_codes = []`, and
`tags = ["active_parse_only"]`. The source defines an identifier-shaped
predicate and a punctuation-shaped functor, aliases them in a later definition
using respectively identifier and punctuation spellings on both sides, and
uses both aliases after activation. Explicit proof blocks/justifications avoid
G7. Its sole `spec_refs` entry is
`spec.en.syntax.redefinition_notation.parser`; append only this parse-only
backlink to that existing trace row without changing its status, stage,
coverage, or requirement count.

Before production edits, the exact frontend case must fail because the aliases
are absent after their declarations. Focused lexer tests must prove exact
alternative spelling/kind/arity/fixity, spelling-plus-original-arity matching,
same-kind collapse, cross-kind candidate preservation, no locus/original
registration, G2 declaration-site admission, and post-semicolon-only activation
for identifier and symbolic patterns. Frontend tests must prove the v3 token
key and unchanged v5 parser key.

Baseline is 554 cases / 499 requirements / 105 active parse-only cases;
expected corpus delta is +1 / +0 / +1, with pass/fail 312/243. Existing `.miz`,
expectations, semantic oracle pairs, trace states, activation map, gap ledger,
coverage audit, Cargo metadata, and `doc/design/archive/` are immutable.
Protected activation-map, gap-ledger, coverage-audit, and archive-manifest
hashes are respectively
`e9a1c2e6b0b444a2caed6a081b6c4a2ff780ee39d4e357c0268d3f8d7215a34b`,
`a0d161dedd78450110fe25e11634f9cdde5f5f633d30965b887cd70aee383dd8`,
`9e75f8ea0f7a1ca81a88f47811f21a264803b16fb3101327caaed7c0925af285`,
and `934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.

Implementation scope is limited to the lexer collector and focused tests; the
frontend token-cache constant and focused tests; the new parse-only pair and
count assertions; one append-only trace backlink; paired
`mizar-lexer/lexical_environment.md`, `mizar-parser/grammar.md`, and
`mizar-frontend/cache_key.md`; this contract; and concise live Step 5 indexes.
The contract alone owns status, frozen scope, review, verification, and
completion evidence. The lexer document owns the durable collection and
activation algorithm, parser grammar owns the unchanged raw-pattern consumer
boundary, frontend cache docs own namespace invalidation, crate plans only link
the contract, and `todo.md` owns only frozen execution order.
`doc/design/spec_coverage_audit.md` has no impact because coverage status,
owner, and deferred rationale do not change.

## Gates and exit

Independent specification/equivalence and boundary reviews precede
implementation. Then run test-sufficiency, implementation, and source/docs/API
reviews; focused lexer/frontend/parser/parse-only checks; metadata/link/ledger
lints; format, warnings-denied workspace Clippy, and full tests. Exit requires
9/9 hard gates, a valid read-only score of at least 90/100, exact task-only
staging, local commit, clean postcommit proof, and fresh selection of 5A.6/G6.
