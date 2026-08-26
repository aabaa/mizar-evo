# Task PARSER-FRAENKEL-GENERATOR-DELIMITER-257C4C8P: parameterized generator delimiter prerequisite

> 正本言語は英語。canonical English:
> [../en/PARSER-FRAENKEL-GENERATOR-DELIMITER-257C4C8P.md](../en/PARSER-FRAENKEL-GENERATOR-DELIMITER-257C4C8P.md)。

Owner planは[mizar-parser](../../mizar-parser/ja/00.crate_plan.md#task-index)、durable ownerは
[grammar](../../mizar-parser/ja/grammar.md#task-257c4c8p-parameterized-comprehension-generator-delimiter)。
Blocked consumerはresolver [C4C8R](RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R.md)。

## Status、目的、readiness

**Status:** implementation complete、task-only commit pending。本completion recordを含む
commitでparser prerequisiteを満たし、その後resolver C4C8Rはsource work前にfreshな
unrecovered-AST preflightを1回必要とする。

C4C8R preflightでexact C4C7 sourceのouter second generatorがrecoveryになることを発見した。
Generic `of`/`over` type-argument parserがouter `y`前のcommaをadditional argumentとして
consumeし、`is@167..169`で`MalformedTermExpression`、`167..184`でrecoveryを作る。
本taskはparser `source_drift`とRust `test_gap`だけをrepairし、C4C8Rはcommitまでblocked。

Chapter 13/Appendix Aの`typed_var_list ::= typed_var { "," typed_var }`により、
comprehension-generator contextの`identifier is`前commaはnext generator separator。
Generic `T of a, b`/`T over c, d`は他contextで保持する。Resolver AST reconstruction、
recovery admission、frozen `.miz`変更は`boundary_violation`。`spec_gap`/
`repo_metadata_conflict`はない。

## Authorityとexact behavior

Authority順はChapter 13 §§13.4/13.8.6、Appendix A、exact C4C7 `.miz`、existing parser
comprehension/`of`/`over` Rust tests、completed C4C7/frozen C4C8R、最後にnon-normative source。

Private comprehension-generator type contextを伝播し、unbracketed `of`/`over` argument listを
comma + identifier + reserved `is`の直前でstopする。Commaはunconsumedのまま
`parse_set_comprehension_at`が2 `ComprehensionVariableSegment`間へemitする。この選択は
`RequiredTypePolicy::ComprehensionGenerator` path限定。

Exact C4C7 termはzero diagnostic/recovery、2 `SetComprehension`、3 generator segments、
`x`/`y` mapper argumentsを含むbracket `ApplicationTerm`となる。Outer segmentsは
`x is Element of NAT`と`y is Element of NAT`。

Generic multi-term `of`/`over`は保持する。Generator contextでもcomma後が`identifier is`
でなければtype-argument commaのまま。Missing/invalid `is`/type/separatorとexisting recovery
diagnosticはfail-closed/byte-compatible。Public API/AST kind/diagnostic code-message/lexer/
resolver/checker identity/semantics/active route/order semanticsは追加しない。

## Scope、test、baseline

Docs prerequisiteはexact 9 paths: paired contract、parser plan/grammar pair、C4C8R
dependency-status pair、coverage audit。Auditはplanned zero-credit mappingだけを追加し、
Chapter-13 summary/traceは不変。

Implementationはexact `crates/mizar-parser/src/module.rs`と`module/tests.rs`。New testは
exact `parser_parses_parameterized_multiple_comprehension_generators`と
`parser_keeps_non_generator_comma_in_comprehension_type_arguments`。前者はexact nested
2-comprehension/3-segment/bracket-mapperをzero diagnostic/recoveryでfreezeし、後者は
`is`が続かない`Element of NAT, y`をtwo-argument typeのまま保持する。Existing `of`/`over`
multi-argumentとmalformed comprehension testはmandatory compatibility coverage。

Completionではpaired contract、paired C4C8R status/evidence、dedicated audit paragraphだけを
追加更新でき、final implementation commitはexact 7 paths。Other owner docsはcompletion-neutral。

Baseline HEADは`5b165dd38e5f1a560eeaff80ef65aa8e5eab0539`、origin/main
`ffc882675141a3e25bc78a47affc018bfe3685e1`、divergence `0/5`、stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。Parser sourceは`16811/629108` SHA
`de648c5e1a81e6d26b2cf94fbcac85fdcae125f4bfcf2ec749c9c8cd0b2de96e`、testsは
`18924/723828` SHA `20ed0a5346888e3eab6837fa61220df75fca745ce5def411d46ec61cef0d325b`。
Parser libは`229->231`、baseline list hash
`9463c31776de0e7b5647a538968ae9fff318964fcaa458ffbf930d0450ebb8e1`、contract tree
`103/103->104/104`。

C4C7 source/sidecar/trace hashは
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`、
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`、
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`で不変。
`doc/spec`/`.miz`/sidecar/trace/expectation/metadata/public API/lexer/resolver/checker/Core/
C4C4 captured/diagnostic contract/active/Task277Bはprotected。

## Review、verification、exit、handoff

Pre-source spec/equivalence/bilingual/boundary、post-source test-sufficiency/implementation/
source-docs-API/final-qualityを独立reviewし、finding-specific re-reviewする。New/existing
focused tests、parser lib/lint、mizar-test metadata/lint、fmt、offline metadata、workspace
warnings-denied Clippy/full tests、diff/hash/count/protected checksを実行。Exitは9/9、valid
90/100以上、exact commit、clean postcommit、fresh inventory。Completionはfrozen resolver
C4C8Rだけをunblockし、checker C4C8/Core 33/35/Task277Bはreadyにしない。

## Precommit implementation completion evidence

Documentation prerequisiteはcommit
`7e8e2abba8ee05e60247958545b47fbaf189b92e`。Implementationはfrozen Rust 2 pathだけを
変更した。Private `RequiredTypePolicy::ComprehensionGenerator` contextをunbracketed
`of`/`over` parsingまで伝播し、comma + identifier + reserved `is`だけをcomprehension
generator loopへ残す。他callerはgeneric pathを保持し、public API/AST kind/diagnostic/
lexer/resolver/checker/Core identity/semantic routeは変更しない。

Exact 2 test functionはnested `x`/`y` mapper arguments、3 generator segments、outer
`Element of NAT` type tree、`of`/`over`両方のcontextual non-generator comma、exact
fail-closed diagnostic/recovery付きcontextual missing-type caseをcoverする。Final sourceは
`module.rs`が`16846 / 630350`、SHA
`18694e3942668b6694d3dee62417d79b841023e70e304cd7d56af6b6db7d845b`、
`module/tests.rs`が`19241 / 735658`、SHA
`2c43239857b0dac74f6dd5d5b8fbe04b8e12867f29707f16cfe6e4de8bf4b133`。

Parser libraryはexact `231` tests、sorted raw-list SHA
`d8dd5c1c7c1508794fd662a70ffdd2bb5dbf81ff56a329a8e789f4a995e5981e`。
Focused、parser lib `231/231`、parser lint `14/14`、mizar-test metadata `137/137`、
mizar-test lint `15/15`はPASS。Workspace all-target/all-feature warnings-denied Clippyと
benchmark targetを含むfull tests、fmt、offline Cargo metadata、`git diff --check`もPASS。
Initial independent implementation reviewは
**NO FINDINGS**。Test-sufficiency reviewのbounded oracle gap 3件はsame 2 test functionを
strengthenして解消し、finding-specific re-reviewは**NO FINDINGS**。Broad source/docs/API
reviewのClippy repair後source measurement 1件は同期evidenceをrefreshし、finding-specific
re-reviewは**NO FINDINGS**。Independent bilingual/boundary reviewも**NO FINDINGS**。
Independent final-quality reviewも**NO FINDINGS**、`9/9` hard gates PASS、valid uncapped
scoreは`100/100`（`20/20/15/15/10/10/5/5`）。Exact stagingとtask-only commitは残る
precommit exit step。

最初のwarnings-denied Clippyはadjacentな同一`break` armを1件検出した。Frozen predicateを
behavior不変のままOR統合し、focused tests/fmt/workspace Clippyはその後PASS。上記source
measurementはrepair後のfinal値。

Protected C4C7 source/sidecar/trace hashはfrozen値を再現し、contract treeは`104/104`。
`doc/spec`、全`.miz`/expectation、trace metadata、C4C4 captured、active result、Task277Bは
不変。本taskはsemantic/coverage creditを付与しない。
