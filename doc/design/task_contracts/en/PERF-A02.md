# Task PERF-A02: Split the measured library-test compilation target
Canonical language: English; [Japanese pointer](../ja/PERF-A02.md).
Status: draft candidate, not a fixed successor; proposed full-tier contract, review before dispatch. Dependencies: PERF-A01 ranking selects A02 after VC53-E1 completion, and reconciliation of active test edits.
Owner: [test plan](../../mizar-test/en/00.crate_plan.md).
Authority: [test strategy](../../architecture/en/20.test_strategy.md), [development protocol](../../autonomous_crate_development.md), [local verification](../../ci_receipt_model.md).
Gap: `test_gap` / development velocity.

## Scope

Split one measured high-cost family into separately selectable Cargo test target; shared test support where genuinely shared, private checks remain with owners. Preserve test obligations and names or provide reviewed old-to-new inventory bijection. No public production APIs solely for testing. Compare focused/full compile/run/link/RSS cost and unaffected target reuse.

## Verification and exit

No deletion, ignoring or weakening tests, expectation/coverage changes or unreviewed receipt inventory renewal. Run affected targets and full unfiltered suite according to custody policy; compare inventory and outcomes, owner-approved baseline renewal only. Exit: independent target with measured improvement or documented tradeoff.
