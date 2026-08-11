# Task TEST-FRAENKEL-NESTED-CAPTURE-257C4C0: Nested Fraenkel Capture Test-Intent Prerequisite

> canonical English: [../en/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md](../en/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md)。正本は英語です。

Owning plans: [mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index) と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。

Stable owner sections: checker [source/spec classification](../../mizar-checker/ja/source_spec_audit.md#task-257c4c0-nested-fraenkel-capture-test-intent)、
[TODO](../../mizar-checker/ja/todo.md#task-257c4c0-nested-fraenkel-capture-test-intent)、
[bilingual record](../../mizar-checker/ja/bilingual_sync_audit.md#task-257c4c0-frozen-contract-parity)、
mizar-test [corpus](../../mizar-test/ja/miz_corpus.md#task-257c4c0-frozen-corpus-increment)、
[traceability](../../mizar-test/ja/traceability.md#task-257c4c0-frozen-traceability-increment)、
[TODO](../../mizar-test/ja/todo.md#task-257c4c0-inactive-capture-oracle)、
[bilingual record](../../mizar-test/ja/bilingual_sync_audit.md#task-257c4c0-frozen-contract-parity)。

## Status, authority, readiness

**Status:** docs-only prerequisite。このexact 20-Markdown-path changeはlater
test-artifact taskだけをfreezeし、`.miz`/sidecar/trace row/coverage-audit/source/
route/stage/semantic creditを追加しない。

Authority orderはcanonical Chapter 13 §13.4.4（§§13.4.2/13.8.6を併読）、
future test-first `.miz`、future trace/sidecar、completed R2/C4A/C4B derived
owners、最後にnon-normative current source observationである。Inner
comprehensionのouter generator referenceはdisplay spellingではなくresolved
binder identityをcaptureしなければならない。F5にはnested occurrenceがないため、
本taskはmissing derived test-intent `design_drift`をcloseし、existing `test_gap`
用future oracleだけをfreezeする。exact sourceは現在parser diagnostic 6件、first
`Element` range `67..74`であり、lower lexical/import `source_drift`はopen。
capture implementationはreadyでない。Task277Bはnot-ready/zero credit、
`MC-G020`/`MC-G021`も不変。

## Frozen future source/artifacts

Future pathsはexactly次の2件である。

- `tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz`
- `tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml`

Source bytesはexactly次の通りである。

```mizar
definition
  func NestedCapture -> set equals
    { { x where y is Element of NAT }
      where x is Element of NAT };
end;
```

final LF、`124` bytes、SHA-256
`f1a35d2d7f6cb4a57ece3b1143a68c1a01ab9ac478960862057025cc9838cea7`。
inner mapper `x`はresolved binder identityでouter generator `x`をselectし、inner
generator `y`はdistinct/unused。builtin-`set` rewrite、local `NAT`/`Element`
lookalike、condition追加、rename/reformatは禁止。

## Exact future sidecar

```toml
schema_version = 1
id = "pass_types_nested_comprehension_outer_generator_capture_001"
kind = "pass"
stage = "advanced_semantics"
domain = "set_expressions.nested_capture"
source = "pass_types_nested_comprehension_outer_generator_capture_001.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
spec_refs = [
  "spec.en.13.set_expressions.nested_capture.semantic",
]
notes = "Inactive advanced_semantics pass oracle derived from Chapter 13 sections 13.4.2, 13.4.4, and 13.8.6: the inner mapper x must capture the resolved outer generator x while inner y remains distinct. This fixes test intent only; current lexical/import admission, capture transport, execution, and Task 277B remain deferred."
```

`tags`と全failure-only fieldはabsent。inactive test intentであり、active tag/
runnerなし。current parser diagnosticをsemantic expectationへrebaselineしない。

## Exact future trace/coverage impact

```toml
[[requirement]]
id = "spec.en.13.set_expressions.nested_capture.semantic"
source = "doc/spec/en/13.term_expression.md"
section = "13.4.4 Nested Comprehensions; with 13.8.6 Set Expression Encoding"
stage = "advanced_semantics"
status = "covered"
required = true
coverage = "pass"
depends_on = [
  "spec.en.13.set_expressions.parser",
]
tests = [
  "tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml",
]
notes = "Spec-derived positive nested-capture seed. The sidecar remains inactive until canonical Element/NAT lexical/import admission, resolver/checker capture transport, and an advanced_semantics runner exist. The covered status records test intent only and grants no execution, diagnostic, sethood, Task-252, C4A/C4B capture-table, or Task-277B credit."
```

dependency/sidecarは各1件だけ。notesはinactive/spec-derived test-intent coverage
only、current execution/capture semantics/Task277B/verdict credit 0を明記する。
Later artifact taskはmapping/trace/follow-upを変更するため
`doc/design/spec_coverage_audit.md`を更新する。本prerequisiteではunchanged。
Artifact taskが変更するのはChapter-13 rowだけであり、mappingにinactive positive
capture oracleを追加し、statusは`partial`のまま、follow-upはexecutable capture
credit 0とlexical/import admissionおよびlater resolver/checker capture transportの
separate ownershipを保持する。他row/statusは変更しない。

Clean baseline HEADは`e0b86bc4ce9ba4adaedab3962057d5f28e368ad6`。
corpus pair `343/343 -> 344/344`、contract trees `89/89 -> 90/90`。
trace baselineは`5908` lines、SHA-256
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`。
coverage audit baselineは`7005` lines、parent inventory frozen abbreviation
ではなくfull SHA-256
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`。
Artifact taskのmetadata projectionはcases/requirements `428/395 -> 429/396`、
pass/fail `235/193 -> 236/193`。Active route counts `101/7/205/1`とestablished
aggregate CLI warnings/errors `23/0`はcreditなし/unchangedで、command-specific
development outputとは区別する。Metadataとplan/parse/declaration/
type/proof全5 CLIをrerunし、full measured counts/hashesをrecordする。

## Scope, prohibitions, deferrals

本prerequisiteは次のexact 20 Markdown pathsだけを変更する。

```text
doc/design/task_contracts/en/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md
doc/design/task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md
doc/design/mizar-checker/en/00.crate_plan.md
doc/design/mizar-checker/ja/00.crate_plan.md
doc/design/mizar-checker/en/source_spec_audit.md
doc/design/mizar-checker/ja/source_spec_audit.md
doc/design/mizar-checker/en/todo.md
doc/design/mizar-checker/ja/todo.md
doc/design/mizar-checker/en/bilingual_sync_audit.md
doc/design/mizar-checker/ja/bilingual_sync_audit.md
doc/design/mizar-test/en/00.crate_plan.md
doc/design/mizar-test/ja/00.crate_plan.md
doc/design/mizar-test/en/miz_corpus.md
doc/design/mizar-test/ja/miz_corpus.md
doc/design/mizar-test/en/traceability.md
doc/design/mizar-test/ja/traceability.md
doc/design/mizar-test/en/todo.md
doc/design/mizar-test/ja/todo.md
doc/design/mizar-test/en/bilingual_sync_audit.md
doc/design/mizar-test/ja/bilingual_sync_audit.md
```

Later artifact-and-owner completion taskもexact 20 pathsだけを変更する。すなわち
上記new corpus 2 files、`tests/coverage/spec_trace.toml`、
`doc/design/spec_coverage_audit.md`、および次の16 completion recordsである。

```text
doc/design/task_contracts/en/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md
doc/design/task_contracts/ja/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md
doc/design/mizar-checker/en/source_spec_audit.md
doc/design/mizar-checker/ja/source_spec_audit.md
doc/design/mizar-checker/en/todo.md
doc/design/mizar-checker/ja/todo.md
doc/design/mizar-checker/en/bilingual_sync_audit.md
doc/design/mizar-checker/ja/bilingual_sync_audit.md
doc/design/mizar-test/en/miz_corpus.md
doc/design/mizar-test/ja/miz_corpus.md
doc/design/mizar-test/en/traceability.md
doc/design/mizar-test/ja/traceability.md
doc/design/mizar-test/en/todo.md
doc/design/mizar-test/ja/todo.md
doc/design/mizar-test/en/bilingual_sync_audit.md
doc/design/mizar-test/ja/bilingual_sync_audit.md
```

Artifact completionでは4 plan rowsを変更しない。このowner updateでfuture corpus、
trace、audit、gap、parity、lifecycle claimをcloseし、omissionによるderived-doc
stalenessを防ぐ。

Rust/Cargo/spec/existing fixture/expectation/
trace/audit/protected artifact/route/active metadataはunchanged。

F5/R2/C4A/C4B/Task252/`CapturedFreeVariables`をedit/reinterpretしない。Future
artifactを本taskで追加しない。builtin-set positive oracle/local lookalikeを作らない。
capture、term/reference ownership、type/sethood/evidence/request/verdict/diagnostic、
Typed/Resolved install、route、trace credit、Task277B activationを実装しない。
parser diagnostic 6件はlower lexical/import `source_drift`でありexpectation driftでは
ない。exact import/prelude owner/module identityはseparate successorへdeferする。

## Reviews, verification, exit, handoff

Authority/test intent、exact20、EN/JA parity、corpus/trace schema、links/fragments、
protected no-op、future-status wordingをindependent reviewする。`git diff --check`と
checker/mizar-test `lint_policy`を実行する。exact20、parity、zero artifact/source
delta、checks PASS、全9 hard gate validでdocs prerequisiteをexitする。stage/commit/
postcommit/fresh inventoryはseparate lifecycleであり、下記historical checkpointで
closeした。

Pre-staging completion evidenceは完了した。Future owner-completion scopeとliteral
EN/JA `notes`をrepair後、independent authority/test-intentおよびbilingual/boundary
reviewは**NO FINDINGS**。Exact20 sorted-path SHA-256は
`9826854d25bdd239f1a0e568e4bf27bd8a06d23e2731aeb9222b9738f99e935d`、contract
treesは`90/90`。`git diff --check`、checker lint policy `15/15`、mizar-test lint
policy `15/15`、metadata `137/137`、frozen 5 CLI replayは全PASS。Final qualityは
**NO FINDINGS**、全`9/9` hard gates PASS、valid uncapped scoreは`100/100`
（`20/20/15/15/10/10/5/5`）。Exact staging/cached reviewは完了した。cacheは
この20 pathsだけ、sorted path SHA-256は
`9826854d25bdd239f1a0e568e4bf27bd8a06d23e2731aeb9222b9738f99e935d`、
`633` insertions/zero deletions、review時unstaged paths 0で、
`git diff --cached --check`をPASSした。

## Historical immediate post-prerequisite checkpoint

Task-only documentation commit
`8e42d5d40a1524639ab13e5462eaf3f646705618`直後のread-only inventoryは、
`HEAD=8e42d5d40a1524639ab13e5462eaf3f646705618`、clean worktree、
`origin/main...HEAD=0/24`、protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`不変を観測した。Commitは
exact 20 Markdown paths、`633` insertions/zero deletions、sorted path SHA-256
`9826854d25bdd239f1a0e568e4bf27bd8a06d23e2731aeb9222b9738f99e935d`であり、
`git show --check HEAD`をPASSした。これはhistorical immediate observationであり、
later closeout commitのcurrent `HEAD`/worktree claimではない。

Task-only commit、postcommit proof、fresh successor inventoryはclosed。Fresh
inventoryは既にfreeze済みのexact20 artifact-and-owner completionだけをnext task
としてacceptし、capture implementationまたはlexical/import implementation choiceを
authorizeしない。Task277Bはnot-ready/zero creditのままである。

次は上記exact artifact-and-owner completion task。そのcommit後fresh inventoryで
lower lexical/import prelude prerequisiteをfreezeし、capture implementationへjump
しない。Authority/public owner/acceptanceはSol `xhigh`、frozen bounded artifact/reviewは
Terra `xhigh`。ambiguity/scope expansionはSolへ戻す。
