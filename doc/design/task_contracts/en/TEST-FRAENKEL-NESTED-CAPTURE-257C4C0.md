# Task TEST-FRAENKEL-NESTED-CAPTURE-257C4C0: Nested Fraenkel Capture Test-Intent Prerequisite

> Canonical language: English. Japanese companion: [../ja/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md](../ja/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

Stable owner sections: checker [source/spec classification](../../mizar-checker/en/source_spec_audit.md#task-257c4c0-nested-fraenkel-capture-test-intent),
[TODO](../../mizar-checker/en/todo.md#task-257c4c0-nested-fraenkel-capture-test-intent),
and [bilingual record](../../mizar-checker/en/bilingual_sync_audit.md#task-257c4c0-frozen-contract-parity);
mizar-test [corpus](../../mizar-test/en/miz_corpus.md#task-257c4c0-frozen-corpus-increment),
[traceability](../../mizar-test/en/traceability.md#task-257c4c0-frozen-traceability-increment),
[TODO](../../mizar-test/en/todo.md#task-257c4c0-inactive-capture-oracle), and
[bilingual record](../../mizar-test/en/bilingual_sync_audit.md#task-257c4c0-frozen-contract-parity).

## Status, authority, and readiness

**Status:** documentation-only prerequisite. This exact 20-Markdown-path
change freezes a later test-artifact task; it creates no `.miz`, sidecar, trace
row, coverage-audit edit, source, route, stage, or semantic credit.

Authority is, in order:

1. canonical [Chapter 13 §13.4.4](../../../spec/en/13.term_expression.md#1344-nested-comprehensions),
   with §§13.4.2 and 13.8.6;
2. the future test-first `.miz` frozen below;
3. its future trace requirement and sidecar;
4. completed R2, C4A, and C4B contracts and derived owner documents;
5. current parser/resolver observations, which are non-normative.

Chapter 13 requires an inner comprehension's reference to an outer generator
to capture the resolved binder identity, not a display spelling. The completed
C4B inventory records that immutable F5 has no such nested occurrence. This
task therefore closes only the missing derived test-intent contract
(`design_drift`) and freezes a spec-derived future oracle for the existing
`test_gap`. Current lexical/import admission remains `source_drift`: the exact
future source currently emits six parser diagnostics, first at `Element`
range `67..74`. No capture implementation is dependency-ready from this
documentation prerequisite.

Task 277B remains not ready and receives zero credit. Its `MC-G020` and
`MC-G021` blockers, template type/sethood interpretation, diagnostic, and
semantic verdict remain unchanged.

## Frozen future source and artifact paths

The later artifact task's canonical artifact surface contains exactly these
two files plus the trace and coverage-audit deltas frozen below:

- `tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz`
- `tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml`

The source bytes are exactly:

```mizar
definition
  func NestedCapture -> set equals
    { { x where y is Element of NAT }
      where x is Element of NAT };
end;
```

The file has a final LF, is exactly `124` bytes, and has SHA-256
`f1a35d2d7f6cb4a57ece3b1143a68c1a01ab9ac478960862057025cc9838cea7`.
Its intended relation is the inner mapper `x` selecting the outer generator
`x` by resolved binder identity; inner generator `y` is distinct and unused.
Both domains are the canonical `Element of NAT` form. A builtin-`set` rewrite,
local `NAT`/`Element` lookalikes, added condition, or renamed/reformatted source
is outside the frozen oracle.

## Exact future sidecar

The sidecar is schema version 1 with these exact fields and values:

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

`tags`, `failure_category`, `rejection_reason`, `stable_detail_key`, and every
failure-only field are absent. The case is inactive test intent: no active tag
or runner is authorized, and current parser diagnostics are not rebaselined
into the semantic expectation.

## Exact future trace row and coverage impact

The later artifact task adds one requirement:

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

The row has only that dependency and sidecar. Its notes must say that coverage
is inactive, spec-derived test intent only; current execution, capture
semantics, Task 277B, and every semantic verdict receive zero credit. Because
the later task changes mapping, traceability, and follow-up ownership, it must
update `doc/design/spec_coverage_audit.md`. This prerequisite leaves that file
unchanged. The artifact task changes only the Chapter-13 row: its mapping must
name the new inactive positive capture oracle, its status remains `partial`,
and its follow-up must retain zero executable capture credit while assigning
the lexical/import admission and later resolver/checker capture transport as
separate work. No other audit row or coverage status changes.

At clean baseline HEAD `e0b86bc4ce9ba4adaedab3962057d5f28e368ad6`, the
corpus has `343/343` `.miz`/sidecar pairs and the contract trees have `89/89`
EN/JA files. The artifact task projects `344/344`; this prerequisite projects
the contract trees to `90/90`. Baseline `tests/coverage/spec_trace.toml` is
`5908` lines with SHA-256
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
Baseline `doc/design/spec_coverage_audit.md` is `7005` lines with SHA-256
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`.
The artifact task projects metadata cases/requirements `428/395 -> 429/396`
and pass/fail `235/193 -> 236/193`; active route counts `101/7/205/1` and
established aggregate CLI warnings/errors `23/0` receive no credit and must
remain unchanged. This does not replace any command-specific development
output. The artifact task must
rerun metadata and all five plan/parse/declaration/type/proof CLIs, recording
their complete measured counts and hashes rather than assuming the projection.

## Scope, prohibitions, and deferrals

This prerequisite changes exactly these 20 Markdown paths:

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

The later artifact-and-owner completion task also changes exactly 20 paths:
the two new corpus files, `tests/coverage/spec_trace.toml`,
`doc/design/spec_coverage_audit.md`, and these 16 completion records:

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

The four plan rows remain unchanged during artifact completion. These owner
updates close their future corpus, trace, audit, gap, parity, and lifecycle
claims; omitting them would leave the derived documentation stale.

It changes no Rust, Cargo, canonical
specification, existing fixture/expectation, trace, coverage audit, protected
artifact, route, or active metadata.

Protected and forbidden:

- do not edit or reinterpret F5, R2, C4A, C4B, Task 252, or
  `CapturedFreeVariables`;
- do not add the future artifacts during this prerequisite;
- do not invent a builtin-set positive oracle or local symbols named `NAT` or
  `Element`;
- do not implement capture, term/reference ownership, type/sethood evidence,
  requests, verdicts, diagnostics, Typed/Resolved installation, routing,
  trace credit, or Task-277B activation;
- do not select a concrete lexical/import prelude implementation before fresh
  inventory after the artifact commit.

The six current parser diagnostics are retained as lower lexical/import
`source_drift`, not expectation drift. The exact import/prelude owner and
module identity are deferred to a separately frozen successor.

## Reviews, verification, exit, and handoff

Review the authority/test intent, exact 20-path boundary, EN/JA parity, corpus
and trace schema, links/fragments, protected no-op claims, and future-status
wording independently. Run `git diff --check` plus checker and mizar-test
`lint_policy` suites. The documentation prerequisite exits only with exactly
20 Markdown paths, synchronized EN/JA owners, no artifact/source delta, passing
checks, and all nine autonomous hard gates valid. Staging, commit, post-commit
proof, and fresh inventory were separate lifecycle steps and are closed in the
historical checkpoint below.

Pre-staging completion evidence is now closed. Independent authority/test-
intent and bilingual/boundary reviews ended with **NO FINDINGS** after the
future owner-completion scope and literal EN/JA `notes` repairs. The exact20
path inventory has sorted SHA-256
`9826854d25bdd239f1a0e568e4bf27bd8a06d23e2731aeb9222b9738f99e935d`;
contract trees are `90/90`; `git diff --check`, checker lint policy `15/15`,
mizar-test lint policy `15/15`, metadata `137/137`, and all five frozen CLI
replays pass. Final quality is **NO FINDINGS**, all `9/9` hard gates pass, and
the uncapped valid score is `100/100` (`20/20/15/15/10/10/5/5`). Exact
staging/cached review is complete: the cache contains only these 20 paths,
has sorted path SHA-256 `9826854d25bdd239f1a0e568e4bf27bd8a06d23e2731aeb9222b9738f99e935d`,
has `633` insertions and zero deletions, had zero unstaged paths at review
time, and passes `git diff --cached --check`.

## Historical immediate post-prerequisite checkpoint

Immediately after the task-only documentation commit
`8e42d5d40a1524639ab13e5462eaf3f646705618`, read-only inventory observed
`HEAD=8e42d5d40a1524639ab13e5462eaf3f646705618`, a clean worktree,
`origin/main...HEAD=0/24`, and unchanged protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`. The commit contains exactly
20 Markdown paths, `633` insertions and zero deletions, with sorted path
SHA-256 `9826854d25bdd239f1a0e568e4bf27bd8a06d23e2731aeb9222b9738f99e935d`;
`git show --check HEAD` passed. This is a historical immediate observation,
not a claim about the later closeout commit's current `HEAD` or worktree.

The task-only commit, post-commit proof, and fresh successor inventory are
closed. Fresh inventory accepted the already frozen exact20 artifact-and-owner
completion as the next task; it did not authorize capture implementation or a
lexical/import implementation choice. Task 277B remains not ready with zero
credit.

The next task after this prerequisite is the exact artifact-and-owner
completion task frozen above. After that task's commit, run fresh inventory
and freeze the lower lexical/import prelude prerequisite; do not jump directly
to capture implementation. Keep Sol at
`xhigh` for authority, public-owner, or acceptance decisions. Terra `xhigh` is
eligible for the bounded artifact implementation and independent reviews after
the contract is frozen; return to Sol on ambiguity or scope expansion.
