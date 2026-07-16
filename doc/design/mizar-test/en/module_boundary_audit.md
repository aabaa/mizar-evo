# Module-Boundary Audit: mizar-test Runner

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_audit.md](../ja/module_boundary_audit.md).

## Task 248 Gate

Task 248 audits the active-runner implementation before any source move. The
maintenance series repairs a `design_drift` in source layout and reviewability;
it does not change Mizar language behavior, runner admission, public APIs,
diagnostics, detail keys, payloads, ordering, expectation meaning, or
traceability credit.

The authority order remains `doc/spec/en` > `.miz` tests > `spec_trace.toml` >
expectations > design > source. Chapters 03, 04, 07, 13, 14, and 16 and their
existing executable intent remain inputs to the runner, not targets of this
refactor. [harness.md](./harness.md), [minimal_crate.md](./minimal_crate.md),
[expectation_schema.md](./expectation_schema.md), and
[internal 07](../../internal/en/07.crate_module_layout.md) define the derived
harness and ownership boundaries.

## Baseline

At Task 248 inventory:

- `src/runner.rs` has 111,262 lines;
- the pre-test prefix ends at line 17,142 and contains the public runner
  facade, private phase helpers, and 137 `#[cfg(test)]` helper attributes;
- one private `mod tests` begins at line 17,143 and occupies about 94,120
  lines;
- the private module contains 272 `#[test]` attributes: 244 at its direct
  scope and 28 in existing nested task modules;
- its direct tests comprise one parse-only import-provider test and
  type-elaboration source-extraction, payload, fixture, corruption, and
  cross-owner isolation families;
- declaration-symbol runner tests are integration-owned by
  `tests/metadata.rs`; no private declaration-symbol test exists to move;
- the active type-elaboration runner has 188 cases, the metadata plan has
  403 cases / 367 requirements, type-elaboration coverage is 235 / 223,
  pass/fail is 219 / 184, and the unit-test count is 272.

## Task 249 Move Result

Task 249 replaced the inline module with private `#[cfg(test)]`,
`#[rustfmt::skip] mod tests;` and moved its body byte-for-byte to
`src/runner/tests.rs`. The formatter guard prevents the newly top-level test
imports and body from being reordered or reflowed during this move-only task.
The runner file is now 17,144 lines and the test module is 94,118 lines. The
exact extracted body hash is
`ab658ad10bcbb2d415778f6289cbb9ae2bed48e21c19b5496fa8f676309d3b69`;
the sorted 272-test list remained
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`.
Module privacy, qualified test names, public API, active-runner counts,
diagnostics, payloads, ordering, and fail-closed behavior are unchanged.

## Task 250 Move Result

Task 250 root-included `src/runner/tests/support.rs` without a wrapper module.
The 6,546-line fragment contains the 17 import groups and the contiguous shared
environment, fixture-specification, AST-builder, corruption, range, and id
support: 201 non-test functions and 24 type/constant items. Its exact moved
hash is
`b880b4605345b1156f125292134d62aff91a32799b5f5834fe7d2a1e5de068a8`.
The retained 87,572 lines remained byte-identical with hash
`197f2d6dc31da2130674954667383bb9aec502a613f3e5b1c33bf0299ea2959b`;
the resulting 87,574-line `tests.rs` hash is
`7d85a8ecd4dffcb0475afc53693e581af661ccbb01b44eab974e030abb046a66`.
All 272 tests remain in `runner::tests` with the same sorted-name hash.

## Task 251 Move Result

Task 251 root-included `src/runner/tests/parse_only.rs` without a wrapper
module. The byte-identical 111-line fragment contains only
`parse_only_provider_resolves_every_stub_and_deduplicates_fixture_summaries`
and has hash
`3cddce85155b72597cfc4c2ea5841dbf3fe5f88d0c8123d98ba9cb958f90a3a8`.
The retained 87,463 lines, including the separator blank, remained
byte-identical with hash
`010f86378bca27c0620998c0de0242d6376fb8b3c37c002d0ca430fb01f7e35c`;
the resulting 87,464-line `tests.rs` hash is
`16480c65416a611c732153360775f10180f609b012027b0a0a970cff1f5a3d84`.
The fully qualified test name and sorted 272-test list are unchanged.

## Task 252 Move Result

Task 252 root-included
`src/runner/tests/type_elaboration/source_extraction.rs` without a wrapper
module. The byte-identical 3,680-line fragment contains the three baseline
reserve extraction, local-mode expansion-chain extraction, and real
declaration-checked `ResolvedTypedAst` handoff tests. Its hash is
`aa9a16c3ed36439ac8c5a4756e3818d6e5f0abd7e076e2e2df8b46487e88c358`.
The retained 83,784 lines, including the separator before Task 253, remained
byte-identical with hash
`2d9ef7d8369c4d654af3bd91598d306c8a9777c9d0981454ce9396095c8a6d79`;
the resulting 83,785-line `tests.rs` hash is
`16f3d6ceb1e75655ea39825f0294896393e676d0a7391bb2a409e14b3b904d22`.
All three fully qualified names and the sorted 272-test list are unchanged.

## Task 253A Move Result

Task 253A root-included
`src/runner/tests/type_elaboration/reserved_binary.rs` without a wrapper
module. The byte-identical 9,982-line fragment contains the leading 23
baseline reserved-variable and binary-formula bridge tests, including the
audited Task 189 and Task 246 ownership exceptions. Its hash is
`88f1a793e139ea808c823fd68956d0dc9863735905ae3fb34e214efa86a11d8e`.
The retained 73,803 lines, including the separator before Task 254, remained
byte-identical with hash
`faf592952a4c871b840b6a1cbbb977ca3f1bbddc98def4f99d54c1a900fdcb06`;
the resulting 73,804-line `tests.rs` hash is
`97d05a3dc35774246af301ad7b4dc6601d2ab85ca669bebfdbcfa140767d150f`.
All 23 fully qualified names, their original order position, and both the
canonical raw and secondary normalized 272-test list hashes are unchanged.
Task 253 remains pending until Task 253B.

## Task 254 Move Result

Task 254 root-included
`src/runner/tests/type_elaboration/mode_chain.rs` without a wrapper module.
The byte-identical 10,232-line fragment contains the 26 audited non-long-chain
local-mode/object-mode tests. Its hash is
`2989031d64871c726f325a5d5bd2ebb4ed4b9a078b83bab3c4f04f456cf3225f`.
The retained 63,572 lines, including the separator before Task 253B, remained
byte-identical with hash
`6725980d7842af5c398f58139ce371ac64d8912ba744f4417ac20c88165d5d81`;
the resulting 63,573-line `tests.rs` hash is
`7e5d0f5735c551be19ac13b2dc96732bf4a9f3cd7088317beb22c760e0d03b68`.
All 26 fully qualified names, their original order position, the Task 253B
boundary, and both 272-test list hashes are unchanged.

## Task 253B Move Result

Task 253B root-included
`src/runner/tests/type_elaboration/reserved_direct.rs` without a wrapper
module. The byte-identical 284-line fragment contains the two audited direct
reserved-variable membership and inequality tests. Its hash is
`c65a5f27463950979368bc702e36f42fa0398884029cff450b54b31095f30e4e`.
The retained 63,289 lines, including the separator before Task 255, remained
byte-identical with hash
`fffe06106cca615e370bb4c2da222da5a4bc21a264cadb5ae8c2d79ed7fdbcce`;
the resulting 63,290-line `tests.rs` hash is
`c90905d94abd1a43c0d65d4abffe8bc970262eee2d64e22da1db4024d614bbf4`.
Both fully qualified names, their original order position, the Task 255
boundary, and both 272-test list hashes are unchanged. Parent Task 253 is
complete.

## Current Ownership

| Current area | Responsibility | Dependency direction | Audit decision |
|---|---|---|---|
| public report/result/status types and `run_*_corpus` functions | Stable public runner facade and corpus-level orchestration | plan/discovery to phase execution | Keep in `runner.rs`. |
| active-case admission and source/frontend staging | Tag/phase gates, source package preparation, frontend execution, stable failure assembly | shared by parse, declaration-symbol, and type-elaboration | Move only after tests stabilize, into private shared helpers. |
| parse-only execution and fixture import provider | Surface-AST snapshots and parser fixture lexical summaries | shared frontend plus parser/frontend seams | Private parse-only owner; preserve the provider's use by later phases. |
| declaration-symbol observation | Resolver shell/projection/symbol collection and deterministic payload keys | frontend AST to resolver output | Private declaration-symbol owner; existing integration tests remain in `tests/metadata.rs`. |
| type-elaboration admission/execution | Lower-stage fail-closed gates and checker/core handoff dispatch | resolver output to source bridge | Private type-elaboration owner. |
| source extraction | Exact source-shape recognition and real AST/resolver payload construction | syntax/resolver inputs to checker inputs | Private type-elaboration leaf owner, moved before its callers. |
| payload validation and detail-key rendering | Exact checker/core output validation, expected/actual matching, deterministic keys, diagnostics | source bridge output to runner result | Private type-elaboration leaf owner; no key or ordering edits. |
| fixture builders and corruption probes | AST/env/sidecar builders and finite negative matrices | test support to private production seams | Private test support/fragments only. |
| cross-owner isolation tests | Bidirectional route rejection and immutable/module guards | all supported source-bridge owners | Keep intact and move as a cohesive fragment. |

## Dependency Map

The permitted dependency direction is:

```text
public runner facade
  -> parse-only owner
     -> shared plan/admission/source/frontend staging
  -> declaration-symbol owner
     -> shared plan/admission/source/frontend staging
  -> type-elaboration owner
     -> shared plan/admission/source/frontend staging
     -> fixture/import-summary adapter
     -> source extraction
     -> checker/core payload validation
     -> deterministic detail keys and failure diagnostics

private runner::tests
  -> shared test support and fixture builders
  -> the same private phase seams
```

Leaf helpers move before their callers. Phase modules may depend on shared
staging, but parse-only and declaration-symbol must not depend on checker/core
payload validation. Metadata `plan` remains payload-free.

## Target Source Layout

The exact leaf split may be made smaller when fresh inventory proves a family
is still too large, but no empty or synthetic owner module is permitted.

| Target path | Ownership |
|---|---|
| `src/runner.rs` | Public facade, public report/result/status types, public active-case iterators, and top-level corpus orchestration only. |
| `src/runner/shared.rs` | Private source package preparation, frontend execution, admission support, and genuinely cross-phase helpers. |
| `src/runner/parse_only.rs` | Parse-only case execution, snapshots, and parse-only failure projection. |
| `src/runner/declaration_symbol.rs` | Declaration-symbol case execution, resolver observation, payload keys, and failure projection. |
| `src/runner/import_fixtures.rs` | Existing parser fixture summaries/adapters used by active phases. |
| `src/runner/type_elaboration.rs` and `src/runner/type_elaboration/` | Type-elaboration orchestration plus private source-extraction and payload-validation/detail/diagnostic leaves. |
| `src/runner/tests.rs` | The single private `runner::tests` module and root-level `include!` declarations. |
| `src/runner/tests/support.rs` | Shared test imports, builders, environments, ids, and corruption helpers. |
| `src/runner/tests/parse_only.rs` | The nonempty parse-only private test family. |
| `src/runner/tests/type_elaboration/*.rs` | Nonempty cohesive source-extraction, reserved/binary, mode-chain, asserted-head, long-chain, and isolation families. |
| `tests/metadata.rs` | Existing declaration-symbol integration-test owner; unchanged unless a later independent nonempty move is justified. |

Test fragments are included directly at the root of `runner::tests`, without a
new wrapper module. This preserves existing qualified test names, including the
already nested Task 216-222 module names. A child-module split is forbidden
when it would change the discovered test list.

Fresh Task 253 inventory splits the reserved/binary owner into two physical,
order-preserving fragments around the Task 254 mode-chain block. Task 253A is
the leading 23-test baseline reserve/binary block. It retains Task 189's
reserved-object type-assertion test because that test is embedded in and owns
the baseline reserved-object bridge boundary; it also retains Task 246's
parenthesized two-edge local-mode equality because that test belongs to the
parenthesized binary route. These classifications do not transfer either test
to the broader asserted-head or mode-chain families. Task 253 remains pending
after 253A: Task 254 moves the intervening local-mode/object-mode chain block,
then Task 253B moves the following direct reserved-variable membership and
inequality tests. Task 255 begins with the direct reserved-variable
type-assertion test. This sequence preserves source and discovery order.

Fresh Task 254 inventory fixes that intervening block at 26 complete tests:
the non-long-chain direct-through-four-edge set/object membership, equality,
and inequality families. It contains no long-chain test or non-test helper/item.
The separator after the block is retained, and Task 253B begins immediately
after it. Task 254 moves only this contiguous block to
`src/runner/tests/type_elaboration/mode_chain.rs`.

Fresh Task 253B inventory fixes the next block at two complete direct
reserved-variable tests: membership and inequality. It contains no non-test
helper/item. The following separator is retained, and Task 255 begins with the
direct reserved-variable type-assertion test immediately after it. Task 253B
moves only this 284-line block to
`src/runner/tests/type_elaboration/reserved_direct.rs`; completing that move
completes parent Task 253.

## Ordered Move Tasks

| Task | Bounded action |
|---|---|
| 248 | Add this paired audit, update the paired crate plan, and establish the preservation matrix. No source move. |
| 249 | Complete: mechanically moved the complete inline private `mod tests` body to `src/runner/tests.rs`. |
| 250 | Complete: moved nonempty shared test support into a root-included support fragment. |
| 251 | Complete: moved the nonempty parse-only private test family into a root-included fragment. |
| 252 | Complete: moved the baseline type-elaboration source-extraction and real handoff tests. |
| 253A | Complete: moved the leading 23-test baseline reserved-variable/binary-formula block; Task 253 remains pending. |
| 254 | Complete: moved the 26-test non-long-chain local-mode/object-mode chain bridge block, retaining the following Task 253B boundary. |
| 253B | Complete: moved the two direct reserved-variable membership and inequality tests to `reserved_direct.rs`, retained the following Task 255 boundary, and completed Task 253. |
| 255 | Move type-assertion and asserted-head bridge tests. |
| 256 | Move long-chain bridge tests. |
| 257 | Move corruption and cross-owner isolation tests while retaining existing nested modules. |
| 258 | Move shared source/frontend staging helpers after the test layout is stable. |
| 259 | Move parse-only production helpers. |
| 260 | Move existing declaration-symbol production helpers; this is not a test move. |
| 261 | Move fixture/import-summary production helpers. |
| 262 | Move type-elaboration source-extraction leaves. |
| 263 | Move payload validation, detail-key, expected-output, and failure-diagnostic leaves. |
| 264 | Close out paired source-layout inventories, path tables, todo/plan state, and ownership guards. |

Every listed source-moving task must be nonempty. If fresh inventory requires a
smaller family, add a bounded subtask before editing; never create a no-op
commit.

## Preservation Matrix

| Surface | Required invariant |
|---|---|
| public API | `mizar_test::runner` re-exports, signatures, enum attributes, and CLI behavior are unchanged. |
| tests | Function names, fully qualified discovered names, nested module names, discovery order/set, and all 272 tests are unchanged. |
| corpus/trace | Active runner 188, plan 403/367, type 235/223, pass/fail 219/184, backlinks, requirements, and expectation meaning are unchanged. |
| diagnostics | Codes, stable detail keys, fallback keys, text, source identity, and ordering are byte-for-byte unchanged. |
| payloads | Keys, values, shapes, provenance, source ranges, binding identities, deterministic ordering, and immutable outputs are unchanged. |
| fail-closed behavior | Unsupported, malformed, ambiguous, imported-gap, evidence-gap, and lower-stage cases continue to reject at the same boundary. |
| authority | No `doc/spec`, `.miz`, expectation, or traceability edit is allowed merely to accommodate a move. |

Before and after each move, capture and compare the exact sorted test lines
from `cargo test -p mizar-test --lib -- --list` in addition to running the
tests. The canonical raw-list oracle, including the `: test` suffix, has 272
lines and hash
`5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`.
The suffix-stripped normalized-name list is only a secondary oracle; its hash
before Task 253A is
`c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`.

## Classification And Coverage-Audit Impact

| Class | Result |
|---|---|
| `design_drift` | Active: source layout obscures phase and ownership review boundaries. Tasks 249-264 repair it without changing behavior. |
| `spec_gap`, `test_gap`, `source_drift`, `test_expectation_drift` | None introduced or repaired by this series. |
| `source_undocumented_behavior`, `boundary_violation` | No new finding; existing runner behavior remains governed by the paired harness plan and higher authorities. |
| `repo_metadata_conflict` | None found. |

`doc/design/spec_coverage_audit.md` remains unchanged. The series changes no
specification chapter coverage, design mapping, traceability status, owner
crate, follow-up ownership, or deferred rationale.

## Per-Task Review And Verification

Each source move requires review-only checks for visibility drift,
test-discovery drift, owner-boundary drift, source/documentation inconsistency,
and accidental behavior change. Required commands are:

```text
cargo test -p mizar-test
cargo run -q -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -q -p mizar-test -- parse-only --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -q -p mizar-test -- declaration-symbol --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo run -q -p mizar-test -- type-elaboration --tests-root tests --manifest tests/coverage/spec_trace.toml
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check
```

The active CLI preservation counts are parse-only 96, declaration-symbol 4,
and type-elaboration 188.

## Exit Criteria

The series is complete only when `runner.rs` is limited to the public facade
and top-level orchestration; every private owner has minimal visibility; the
preservation matrix passes; paired source-layout, crate-plan, todo, harness
path-table, bilingual, and ownership-guard documentation is synchronized; and
all verification is green. Fresh Step 5 inventory resumes only after Task 264.
Steps 6 and 7 remain deferred until their existing dependency gates are met.
