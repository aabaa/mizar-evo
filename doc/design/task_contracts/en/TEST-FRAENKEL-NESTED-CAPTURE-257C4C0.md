# Task TEST-FRAENKEL-NESTED-CAPTURE-257C4C0: Nested Fraenkel Capture Inactive Test Intent

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

**Status:** artifact, owner-document, and private count-guard implementation is
complete; test/implementation and source-documentation/bilingual/boundary
reviews are **NO FINDINGS**. Final quality is **NO FINDINGS**, `9/9` hard gates
pass, and the valid uncapped score is `100/100`. Lifecycle closeout is complete.
The exact 24-path completion adds the frozen `.miz`, inactive sidecar, sole
trace row, Chapter-13 coverage-audit
delta, 16 synchronized owner records, and four private global-count test
maintenance edits. It creates no active route,
executable stage, capture semantics, diagnostic, or Task-277B credit. The four
crate-plan rows remain unchanged.

Authority is, in order:

1. canonical [Chapter 13 §13.4.4](../../../spec/en/13.term_expression.md#1344-nested-comprehensions),
   with §§13.4.2 and 13.8.6;
2. the implemented test-first `.miz` recorded below;
3. its implemented trace requirement and inactive sidecar;
4. completed R2, C4A, and C4B contracts and derived owner documents;
5. current parser/resolver observations, which are non-normative.

Chapter 13 requires an inner comprehension's reference to an outer generator
to capture the resolved binder identity, not a display spelling. The completed
C4B inventory records that immutable F5 has no such nested occurrence. This
documentation prerequisite closed the missing derived test-intent contract
(`design_drift`); this artifact completion closes the existing `test_gap` with
the spec-derived inactive oracle. Current lexical/import admission remains
`source_drift`: the exact implemented source still emits six parser diagnostics,
first at `Element` range `67..74`. No capture implementation is made
dependency-ready by this inactive artifact.

Task 277B remains not ready and receives zero credit. Its `MC-G020` and
`MC-G021` blockers, template type/sethood interpretation, diagnostic, and
semantic verdict remain unchanged.

## Implemented source and artifact paths

The canonical artifact surface contains exactly these two files plus the trace
and coverage-audit deltas recorded below:

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

## Exact implemented sidecar

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

## Exact implemented trace row and coverage impact

The artifact adds one requirement:

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

The row has only that dependency and sidecar. Its notes say that coverage
is inactive, spec-derived test intent only; current execution, capture
semantics, Task 277B, and every semantic verdict receive zero credit. Because
the artifact changes mapping, traceability, and follow-up ownership, it updates
`doc/design/spec_coverage_audit.md`. Only the Chapter-13 row changes: its
mapping names the new inactive positive capture oracle, its status remains
`partial`, and its follow-up retains zero executable capture credit while
assigning lexical/import admission and later resolver/checker capture transport
as separate work. No other audit row or coverage status changes.

At clean baseline HEAD `e0b86bc4ce9ba4adaedab3962057d5f28e368ad6`, the
corpus has `343/343` `.miz`/sidecar pairs and the contract trees have `89/89`
EN/JA files. The implemented corpus is `344/344`; the prerequisite brought the
contract trees to `90/90`. Baseline `tests/coverage/spec_trace.toml` was
`5908` lines with SHA-256
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
Baseline `doc/design/spec_coverage_audit.md` was `7005` lines with SHA-256
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`.
Final measurements are: source `124` bytes with SHA-256
`f1a35d2d7f6cb4a57ece3b1143a68c1a01ab9ac478960862057025cc9838cea7`;
sidecar SHA-256
`2c7d987baa988b9ea1ae179d6ed1a3b9c8df334694cdd9d43626342647d59701`;
trace `5924` lines with SHA-256
`d1df314665998fe5271a73d7102b6e6d6098fd6636d78e2a6ded779d5f44cbae`;
and coverage audit `7005` lines with SHA-256
`99720173f84f1713ed2bf63e9806566b2aa6a904d18d6855b20544bab96928a5`.
Metadata passes `137/137` with cases/requirements `429/396`, pass/fail
`236/193`, active routes `101/7/205/1`, and aggregate warnings/errors `23/0`.
The first metadata run found requirement-ID ordering; moving `nested_capture`
before `parser` repaired it, and the final run passed. CLI stdout SHA-256 values
are plan `2d2accef2c6fc32c1b3372530f6136af1299ac6ae7db6a0158798336b779c7e7`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.
These unchanged active route and aggregate warning/error counts grant no
execution or semantic credit and do not replace command-specific development
output.

The first full `cargo test` reached the four stale count guards and failed
`610/614`; after the exact tuple repairs, `cargo test -q -p mizar-test --lib`
passes `614/614`. Final private-test file measurements are:

| path | lines | SHA-256 |
|---|---:|---|
| `source_attribute_definition.rs` | `1113` | `ae59a65e2b899471967e37d597273d1705344ac17ba9d688003f549afb35968a` |
| `source_functor_definition.rs` | `1674` | `d97abf2bd83e9af4e5c64b84bd8b05045b1df257bf3c56dad7bf7f7876a3b715` |
| `source_mode_definition.rs` | `1242` | `701fca1a591973e54ffe121599d1e7de7596b3e968f3180d2bc120fa8aabee25` |
| `source_property_implementation.rs` | `236` | `15db079c61dcfbde48b2922eaebb321ea126163e6368fdfa9e218395a6ebed83` |

Post-repair `cargo fmt --all -- --check`, checker and mizar-test lint policy
`15/15` each, full `cargo test`, full workspace all-target/all-feature Clippy
with `-D warnings`, metadata `137/137`, all five CLIs, and `git diff --check`
pass. The exact24 sorted-path SHA-256 is
`085737bdf48261cd81b84101aa89e84c9b7444b3c38b66c9fd98fb849d154a4e`.
Independent artifact/test-sufficiency and implementation review ended with
**NO FINDINGS**. Independent source-documentation, EN/JA, and boundary review
also ended with **NO FINDINGS**; the exact24 measurements and zero-credit
deferrals were reproduced. Final-quality scoring and commit lifecycle are
complete, with the commit lifecycle recorded in the historical checkpoint below.

## Scope, prohibitions, and deferrals

The historical documentation prerequisite changed exactly these 20 Markdown
paths:

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

The artifact-and-owner completion changes exactly 24 paths:
the two new corpus files, `tests/coverage/spec_trace.toml`,
`doc/design/spec_coverage_audit.md`, these 16 completion records, and four
private count-guard test paths:

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

```text
crates/mizar-test/src/runner/tests/type_elaboration/source_attribute_definition.rs
crates/mizar-test/src/runner/tests/type_elaboration/source_functor_definition.rs
crates/mizar-test/src/runner/tests/type_elaboration/source_mode_definition.rs
crates/mizar-test/src/runner/tests/type_elaboration/source_property_implementation.rs
```

The first full `cargo test` exposed the prerequisite's exact20 scope as
`design_drift`: those four existing private tests intentionally pin the global
metadata tuple. The only authorized Rust changes update cases/requirements
`(428, 395) -> (429, 396)` and pass/fail `(235, 193) -> (236, 193)` once in
each named file. Active `[205; 6]` and `(101, 7, 205, 1)` route assertions stay
unchanged. This is test-maintenance only; production code, routing, capture,
and semantic behavior remain byte-unchanged.

The parent exclusively owns the `.miz`, sidecar, trace, and coverage-audit
paths; owner integration changes only the 16 Markdown records listed above.
The four plan rows remain unchanged. These updates close their future corpus,
trace, audit, gap, parity, and lifecycle claims.

It changes no production Rust, Cargo, canonical specification, existing
fixture/expectation beyond the exact new pair, trace/audit state beyond the
exact recorded row deltas, protected artifact, route, or active metadata.

Protected and forbidden:

- do not edit or reinterpret F5, R2, C4A, C4B, Task 252, or
  `CapturedFreeVariables`;
- do not add any artifact beyond the exact implemented source, sidecar, sole
  trace row, and Chapter-13 audit delta;
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

Review the authority/test intent, exact 24-path artifact boundary, EN/JA parity,
corpus and trace schema, links/fragments, protected no-op claims, and truthful
inactive status independently. Run `git diff --check` plus checker and
mizar-test `lint_policy` suites. Artifact completion exits only with the exact
four artifact/audit paths, 16 owner paths, and four private count-guard test
paths, synchronized EN/JA owners, passing checks, and all nine autonomous hard
gates valid. Source/documentation and bilingual/boundary reviews are complete;
final quality is **NO FINDINGS**, all `9/9` hard gates pass, and the valid
uncapped score is `100/100` (`20/20/15/15/10/10/5/5`). Current lifecycle
status is recorded below.

Historical pre-commit staging/cached review is complete: the cache contained only these 24
paths, including the two new artifacts and four private count guards, had zero
unstaged paths at review time, had sorted path SHA-256
`085737bdf48261cd81b84101aa89e84c9b7444b3c38b66c9fd98fb849d154a4e`,
final stat `378/191`, and passed `git diff --cached --check`.

Historical prerequisite pre-staging evidence is closed. Independent authority/test-
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
closed. Fresh inventory accepted the then-frozen exact20 artifact-and-owner
completion as the next task; the current artifact review repaired that scope
to exact24 for four global count guards. The selection did not authorize
capture implementation or a lexical/import implementation choice. Task 277B
remains not ready with zero credit.

## Historical immediate post-artifact checkpoint

Immediately after task-only artifact commit
`eb2ff9d40427797d1946dc140c7ba9c3a83d4b90` with parent
`4c3d012d7f330474b72d733bc05f405a00bf9cec`, read-only inventory observed
`HEAD=eb2ff9d40427797d1946dc140c7ba9c3a83d4b90`, a clean worktree,
`origin/main...HEAD=0/26`, and unchanged protected
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`. The commit contains exactly
24 paths, `378` insertions and `191` deletions, with sorted path SHA-256
`085737bdf48261cd81b84101aa89e84c9b7444b3c38b66c9fd98fb849d154a4e`;
`git show --check HEAD` passed. This is a historical immediate-postimplementation,
pre-closure observation, not a claim about the documentation-closure commit's
current `HEAD` or worktree. It is distinct from the historical pre-commit cached
review above even though their path inventory, stat, and hash are equal.

The task-only artifact commit, immediate post-commit proof, and fresh successor
inventory are closed. Fresh inventory concludes **protocol STOP** on a blocking,
human-owned `spec_gap`: canonical authority names a built-in prelude but does
not define its contents, lexical-seeding relationship, or the `Element`/`NAT`
provider and module/export identities. Canonical
[§2.10](../../../spec/en/02.lexical_structure.md#210-lexical-preprocessing) and
[§12.3](../../../spec/en/12.modules_and_namespaces.md#123-import-statements)
make imported lexical summaries source-import-prelude-driven, while
[§11.2.4](../../../spec/en/11.symbol_management.md#1124-precedence-rules)
separately includes a built-in prelude in semantic lookup and
[§3.3](../../../spec/en/03.type_system.md#33-type-expressions) makes only
`object` and `set` built-in type heads. The exact 124-byte source has no import,
no canonical membership or module/export identity for `Element` or `NAT` in
that prelude is frozen, and the frontend requires every resolved import to
correspond to a source import stub.
That frontend constraint is a non-normative observation, not authority for the
missing rule. Implicit injection would cross the unresolved language and
provider-provenance boundary; adding an explicit import would change the frozen
source and test intent. This checkpoint selects no lower task, owner, API,
module, or capture implementation. Task 277B remains not ready with zero credit.

Resume only after human authority either separately approves reopening the test
intent with exact replacement source/hash, explicit import, and canonical
`Element`/`NAT` module/export identities, or specifies the canonical built-in-
prelude contents, lexical seeding, and provider provenance. Sol may
interpret that authority after it exists but must not invent either rule. Do not
jump directly to capture implementation. Keep Sol at
`xhigh` for authority, public-owner, or acceptance decisions. Terra `xhigh` is
eligible for the bounded artifact implementation and independent reviews after
the contract is frozen; return to Sol on ambiguity or scope expansion.
