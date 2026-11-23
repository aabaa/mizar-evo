# Task VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-KERNEL: bounded caller uniqueness
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-KERNEL.md).
Status: frozen Kernel-only slice. Tier: full; new public semantic rule fails pattern eligibility.
Owner: [Kernel](../../mizar-kernel/en/00.crate_plan.md). Consumer: mizar-test; fresh VC checking remains separate.
Dependencies: [single-valuedness request](VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-REQUEST.md), [caller typing precedent](VC53-UNARY-CALLER-E1-TYPING-KERNEL.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1/E3](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [caller typing](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-typing), [child joins](../../mizar-kernel/en/checker.md#unary-child-existential-instantiation), [separate goals](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design).
Gap: source_drift/test_gap; the bounded caller single-valuedness request has no local Kernel rule. Existing body semantics fix uniqueness; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add only boolean `check_unary_caller_return_single_valuedness(core, request, callee_flow, targets, binders, conclusion, context)` in Kernel checker.rs. Reuse existing relation tuples: two targets, two result variables, and two ordered definition-free equality operands.
Require exact common caller/flow/Return/actual input, result projections matching the two distinct fresh binders, ordered equality and only the original caller object-input guard. Reuse unchanged unary-child validation for each binder with the supplied callee flow, both sole bodies, guards, request/slot/Apply joins and global cycle safety before rebuilding and matching the caller CFG.
Every child return is its immutable formal instantiated by the actual caller parameter, and the caller returns exactly that child's result (§20.1.2, §20.2.6). Thus any two caller returns for the same input agree, independently of typing or E4 existence. Returns remain antecedents; requires, ensures, function equality and child/E4 evidence are not premises.
No new type, holder, field, helper, fixture or parallel representation. Keep direct-return single-valuedness, caller typing and every lower-stage API unchanged.
Exclude schema reporting, fresh-source authentication, VC checking facade, Proof authority, E2, whole acceptance/status, currentness producers, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. No spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: Kernel checker.rs, existing runner tests/algorithm.rs, paired Kernel checker/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful setup before expanding.
1. Independent exact two-target/binder/equality/sole-guard oracle succeeds repeatedly without supplied child/E4 evidence, preserving input/product purity. Equal cloned Core checks independently.
2. Genuine renames/trivia/equal spellings and optional terminating flags preserve exact structural joins and success.
3. Existing request/Core drift matrix refuses invalid structural joins; valid label-only differences remain local successes. Refuse drift of either target's caller/flow/Return/input/result definition, callee flow, duplicate/nonfresh binders, swapped/wrong/defined equality and missing/extra/swapped guard. Both bodies/Apply/slots/guards/freshness/CFG/cycles are exercised; restore and recover. Calibrate negatives with genuine success first.
4. Existing child/E4/inhabitation/typing/direct-return and raw unknown/incomplete/cache-miss boundaries stay unchanged. Caller rule refuses direct-return targets; local success authenticates no source or Proof authority.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/VC/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed structural/uniqueness matrices run immediately; unchanged heavy evidence/cache/policy/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: only local bounded caller E1 single-valuedness becomes checkable, preserving every excluded authority.
