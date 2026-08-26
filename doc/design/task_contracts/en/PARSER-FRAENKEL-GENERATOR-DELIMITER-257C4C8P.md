# Task PARSER-FRAENKEL-GENERATOR-DELIMITER-257C4C8P: Parameterized generator delimiter prerequisite

> Canonical language: English. Japanese companion:
> [../ja/PARSER-FRAENKEL-GENERATOR-DELIMITER-257C4C8P.md](../ja/PARSER-FRAENKEL-GENERATOR-DELIMITER-257C4C8P.md).

Owning plan: [mizar-parser](../../mizar-parser/en/00.crate_plan.md#task-index).
Durable owner section:
[grammar](../../mizar-parser/en/grammar.md#task-257c4c8p-parameterized-comprehension-generator-delimiter).
Blocked consumer: resolver
[C4C8R](RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R.md).

## Status, purpose, and readiness

**Status:** implementation complete and task-only commit pending. The parser
prerequisite is satisfied by the commit containing this completion record;
resolver C4C8R then requires one fresh unrecovered-AST preflight before source
work starts.

Fresh C4C8R implementation preflight found that the exact C4C7 source is
currently parsed with one recovered outer segment. The generic `of`/`over`
type-argument parser consumes the comma before outer `y` as another type
argument, then reports `MalformedTermExpression` at `is@167..169` and recovers
through `167..184`. This task repairs only that parser `source_drift` and its
Rust `test_gap`. Resolver C4C8R remains blocked until this task commits.

The owner and result are unique. Chapter 13 and Appendix A define
`typed_var_list ::= typed_var { "," typed_var }`; in comprehension-generator
context the comma before `identifier is` is therefore the next generator
separator. Generic `T of a, b` and `T over c, d` remain valid outside that
context. Reconstructing the AST in resolver, admitting recovery, or changing
the frozen `.miz` is a `boundary_violation`. There is no `spec_gap` or
`repo_metadata_conflict`.

## Authority and exact behavior

Authority is, in order:

1. Chapter 13 [§13.4 grammar and multiple generators](../../../spec/en/13.term_expression.md#134-set-expressions)
   and [§13.8.6](../../../spec/en/13.term_expression.md#1386-set-expression-encoding),
   plus [Appendix A §A.13](../../../spec/en/appendix_a.grammar_summary.md#a13-term-expressions);
2. exact existing C4C7
   [`pass_types_nested_comprehension_two_outer_generator_captures_001.miz`](../../../../tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.miz);
3. existing parser set-comprehension and `of`/`over` argument Rust tests;
4. completed C4C7 and frozen C4C8R contracts;
5. current parser source observation, which is non-normative.

The parser must propagate a private comprehension-generator type context far
enough that an unbracketed `of`/`over` argument list stops before a comma whose
next tokens are an identifier and reserved `is`. The comma remains unconsumed
for `parse_set_comprehension_at`, which emits it between two
`ComprehensionVariableSegment` children. This preference is limited to the
`RequiredTypePolicy::ComprehensionGenerator` path.

The exact C4C7 term must become diagnostics-free and unrecovered with two
`SetComprehension` nodes, three generator segments, and one bracket
`ApplicationTerm` containing the `x` and `y` mapper arguments. The outer
segments remain `x is Element of NAT` and `y is Element of NAT`.

Generic type parsing retains multi-term `of`/`over` argument lists. Within a
comprehension generator, a comma not followed by `identifier is` remains a
type-argument comma. Missing/invalid `is`, missing type, malformed separators,
and all existing recovery diagnostics remain fail-closed and byte-compatible.
No public API, AST kind, diagnostic code/message, lexical rule, resolver or
checker identity, semantic result, active route, or ordering semantics is
added.

## Scope, tests, and audit impact

The documentation prerequisite changes exactly nine paths: this paired
contract, paired parser plan/grammar records, the paired C4C8R dependency-
status amendment, and `doc/design/spec_coverage_audit.md`. The audit adds one
planned zero-credit parser mapping; Chapter-13 summary and trace metadata stay
unchanged.

Implementation changes exactly two Rust paths:

1. `crates/mizar-parser/src/module.rs`; and
2. `crates/mizar-parser/src/module/tests.rs`.

It adds exactly two tests:

1. `parser_parses_parameterized_multiple_comprehension_generators`; and
2. `parser_keeps_non_generator_comma_in_comprehension_type_arguments`.

The first freezes zero diagnostics/recovery and the exact nested 2-comprehension,
3-segment, bracket-mapper shape. The second freezes the negative lookahead:
`Element of NAT, y` remains a two-argument type when `y` is not followed by
`is`. Existing `of`/`over` multi-argument and malformed-comprehension tests are
mandatory compatibility coverage.

Completion may additionally update only this paired contract, the paired
C4C8R status/dependency evidence, and the dedicated coverage-audit paragraph.
The final implementation commit therefore changes exactly seven paths. Other
owner documentation is completion-neutral.

## Baseline and protected state

Clean baseline HEAD is documentation commit
`5b165dd38e5f1a560eeaff80ef65aa8e5eab0539`, origin/main is
`ffc882675141a3e25bc78a47affc018bfe3685e1`, and divergence is `0/5`.
Protected stash remains
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

| Path | Baseline lines / bytes | Baseline SHA-256 |
|---|---:|---|
| `crates/mizar-parser/src/module.rs` | `16811 / 629108` | `de648c5e1a81e6d26b2cf94fbcac85fdcae125f4bfcf2ec749c9c8cd0b2de96e` |
| `crates/mizar-parser/src/module/tests.rs` | `18924 / 723828` | `20ed0a5346888e3eab6837fa61220df75fca745ce5def411d46ec61cef0d325b` |

Parser library tests project `229 -> 231`; the sorted baseline list SHA-256 is
`9463c31776de0e7b5647a538968ae9fff318964fcaa458ffbf930d0450ebb8e1`.
Contract trees project `103/103 -> 104/104`. C4C7 source/sidecar and trace
hashes remain respectively
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`,
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`,
and `17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`.

No `doc/spec`, `.miz`, sidecar, trace/expectation, metadata count, public API,
lexer, resolver/checker/Core source, C4C4 captured state, diagnostic contract,
active result, or Task-277B change is authorized.

## Reviews, verification, exit, and handoff

Before source edits, independent specification/equivalence and bilingual/
boundary reviews must report **NO FINDINGS**. After implementation,
independent test-sufficiency, implementation, source/documentation/API, and
final-quality reviews must report **NO FINDINGS**, with finding-specific
re-review after fixes.

Verification includes both new tests, existing set-comprehension recovery and
`of`/`over` argument tests, parser library and lint, mizar-test metadata/lint,
formatting, offline Cargo metadata, full-workspace all-target/all-feature
warnings-denied Clippy and tests, `git diff --check`, exact hashes/counts, and
protected-surface checks.

Exit requires `9/9` hard gates and a valid score of at least `90/100`, exact
task-only commit, clean postcommit proof, and fresh inventory. Successful
completion unblocks the already-frozen resolver C4C8R implementation; it does
not make checker C4C8, Core 33/35, or Task 277B ready.

## Precommit implementation completion evidence

The documentation prerequisite is commit
`7e8e2abba8ee05e60247958545b47fbaf189b92e`. The implementation changes only
the two frozen Rust paths. A private `RequiredTypePolicy::ComprehensionGenerator`
context now reaches unbracketed `of`/`over` parsing and leaves only a comma
followed by identifier plus reserved `is` for the comprehension generator
loop. All other callers retain the generic path, and no public API, AST kind,
diagnostic, lexer, resolver/checker/Core identity, or semantic route changes.

The two exact test functions cover the nested `x`/`y` mapper arguments, all
three generator segments, the outer `Element of NAT` type trees, contextual
non-generator commas for both `of` and `over`, and a contextual missing-type
case with exact fail-closed diagnostic and recovery evidence. Final source
measurements are:

| Path | Lines / bytes | SHA-256 |
|---|---:|---|
| `crates/mizar-parser/src/module.rs` | `16846 / 630350` | `18694e3942668b6694d3dee62417d79b841023e70e304cd7d56af6b6db7d845b` |
| `crates/mizar-parser/src/module/tests.rs` | `19241 / 735658` | `2c43239857b0dac74f6dd5d5b8fbe04b8e12867f29707f16cfe6e4de8bf4b133` |

Parser library state is exactly `231` tests with sorted raw-list SHA-256
`d8dd5c1c7c1508794fd662a70ffdd2bb5dbf81ff56a329a8e789f4a995e5981e`.
The focused tests, parser library `231/231`, parser lint `14/14`, mizar-test
metadata `137/137`, and mizar-test lint `15/15` pass. Workspace all-target,
all-feature warnings-denied Clippy and full tests, including benchmark targets,
formatting, offline Cargo metadata, and `git diff --check` also pass. Initial independent
implementation review reported **NO FINDINGS**. Independent test-sufficiency
review found three bounded oracle gaps; after strengthening the same two test
functions, finding-specific re-review reported **NO FINDINGS**. Broad
source/documentation/API review found one stale source measurement after the
Clippy repair; the synchronized evidence was refreshed and finding-specific
re-review reported **NO FINDINGS**. Independent bilingual/boundary review also
reported **NO FINDINGS**. Independent final-quality review reported
**NO FINDINGS**, all `9/9` hard gates pass, and the valid uncapped score is
`100/100` (`20/20/15/15/10/10/5/5`). Exact staging and the task-only commit
remain the precommit exit steps.

The first warnings-denied Clippy run found adjacent identical `break` arms.
Their already-frozen predicates were combined without changing behavior;
focused tests, formatting, and workspace Clippy then pass. The final source
measurement above is after that repair.

The protected C4C7 source, sidecar, and trace hashes reproduce the frozen
values. Contract trees are `104/104`. `doc/spec`, every `.miz` and expectation,
trace metadata, C4C4 captured state, active results, and Task 277B remain
unchanged; this task grants zero semantic or coverage credit.
