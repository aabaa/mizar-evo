# Task VC53-UNARY-CALLER-E1-TYPING-KERNEL: bounded caller universal typing
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E1-TYPING-KERNEL.md).
Status: frozen Kernel-only slice. Tier: full; new public semantic rule fails pattern eligibility.
Owner: [Kernel](../../mizar-kernel/en/00.crate_plan.md). Consumer: mizar-test; fresh VC checking remains separate.
Dependency: [caller typing request](VC53-UNARY-CALLER-E1-TYPING-REQUEST.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Kernel caller typing](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-typing), [child joins](../../mizar-kernel/en/checker.md#unary-child-existential-instantiation), [typing request](../../mizar-vc/en/generator.md#bounded-unary-caller-e1-typing-request).
Gap: source_drift/test_gap; the bounded caller universal typing request has no local Kernel checker. Existing body semantics determine typing; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add only boolean `check_unary_caller_return_typing(core, request, callee_flow, target, binder, conclusion, context)` in Kernel checker.rs; target/conclusion reuse the existing direct-return projections.
Check the exact caller relation, fresh result binder and atomic object conclusion under the sole original caller object-input guard. Reuse unchanged unary-child validation with the supplied callee flow for both sole bodies, original guards, request/slot/Apply joins and global cycle safety before rebuilding the caller CFG. Refuse every mismatched caller flow/Return/input/result, binder, conclusion, guard or child join.
Universal typing follows from operational semantics: every child return is its immutable formal, instantiated by the actual caller parameter; every caller return is exactly that child's returned result. Thus every returned caller result has the original actual's object type. E4 existence or one witness alone cannot establish this universal claim; returns, requires, ensures and function equality add no context premise.
No new type, holder, field, helper, fixture, schema API or parallel representation. Keep the direct-return typing rule and all lower-stage APIs unchanged.
Exclude fresh-source authentication, VC checking facade, Proof authority, single-valuedness checking, E2, whole acceptance, status, currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. No spec/.miz/expectation/trace/audit-status, diagnostic, AGENTS or protocol edits.
Files: Kernel checker.rs, existing runner tests/algorithm.rs, paired Kernel checker/plan documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful setup before expanding.
1. Existing independent four-row typing oracle supplies exact caller target/binder/conclusion/context; Kernel succeeds repeatedly with input/product purity, without externally supplied child/E4 evidence. Equal cloned Core checks independently.
2. Genuine renames/trivia/equal spellings and all existing optional terminating flags preserve successful typing and exact identity joins.
3. Existing complete request/Core drift matrix refuses locally where structural joins are invalid; valid label-only differences remain local structural successes. Add target/flow/input/result definition, binder/conclusion and missing/extra/swapped guard refusals. Mutate both bodies/Apply/slots/guards/freshness/CFG/cycles; restore original and recover. Calibrate negatives with a genuine success first.
4. Existing child/E4/inhabitation/direct-return and raw unknown/incomplete/cache-miss boundaries remain unchanged. The caller rule refuses a direct-return target; local success authenticates no source or Proof authority.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/VC/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed structural/typing matrices run immediately; unchanged heavy evidence/cache/policy/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: only local bounded caller universal result typing becomes checkable, preserving every excluded authority.
