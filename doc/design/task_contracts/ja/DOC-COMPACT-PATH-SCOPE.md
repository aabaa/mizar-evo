# Task DOC-COMPACT-PATH-SCOPE: path-scoped legacy heading enforcement

> canonical English: [../en/DOC-COMPACT-PATH-SCOPE.md](../en/DOC-COMPACT-PATH-SCOPE.md)。

## Identity と status

| Field | Frozen value |
|---|---|
| Task | `DOC-COMPACT-PATH-SCOPE` |
| Status | documentation contract frozen、implementation/separate commit待ち |
| Purpose | schema-v1 forbidden-heading enforcementを各redirectのdeclared source pathへscopeする。 |
| Authority | `AGENTS.md` exact-source-path rule、schema-v1 manifest field、B4A duplicate-heading `test_gap` |
| Consumer | mizar-test lint policyのgeneric legacy-compaction validation |
| Plan indexes | [checker plan](../../mizar-checker/ja/00.crate_plan.md#task-index) / [runner plan](../../mizar-test/ja/00.crate_plan.md#task-index) |

## Frozen scope と behavior

implementationは`crates/mizar-test/tests/lint_policy.rs`だけを変更する。
forbidden-heading checkはrepository-global heading setではなく各manifest
redirectのexact `(source_path, legacy_heading)` pairを使う。declared sourceに
legacy headingが残ればrejectし、unrelated unselected Markdown documentの同一text
は許可する。両caseのdeterministic regressionを追加する。

既存schema、batch/task relation、redirect grammar、language-local target、fragment、
neighbor anchor、source cardinality、sorted order、inventory hash checkを全て保持する。
lint correctness repairでありschema extensionではない。

`AGENTS.md`、protocol、documentation、manifest data、source inventory、production
Rust、spec、corpus fixture、expectation、trace、coverage、Cargo、language behavior、
test intent、registered compaction resultは変更禁止。Rustへtask ID/path/headingの
special caseを追加しない。

## Review、verification、exit

spec/design、test-sufficiency、implementation、source/document reviewを
**NO FINDINGS**まで行い、hard gate 9件PASS、capなし、`>=90/100`。focused/full
mizar-test lint、format、warnings-denied Clippy、workspace、`git diff --check`を
実行する。commit済みcontract/index prerequisiteは不変とし、別review済み
implementation commitでは`crates/mizar-test/tests/lint_policy.rs`だけをstageする。
push/fetch/reset/stash mutationは禁止。

clean commit後、DOC-258B4A-COMPACT sourceをfresh replayしてfrozen migrationを
継続する。
