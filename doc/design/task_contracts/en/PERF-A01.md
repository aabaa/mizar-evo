# Task PERF-A01: Measure development and product performance baseline
Canonical language: English; [Japanese pointer](../ja/PERF-A01.md).
Status: draft; proposed full-tier contract, review before dispatch. Dependencies: VC53-E1 chain complete and sealed; measure that revision and coordinate with custody.
Owner: [test plan](../../mizar-test/en/00.crate_plan.md).
Authority: [test strategy](../../architecture/en/20.test_strategy.md), [development protocol](../../autonomous_crate_development.md), [local verification](../../ci_receipt_model.md).
Gap: `test_gap` / development velocity.

## Scope

Measure clean, unchanged, isolated test edit and representative owner edit Cargo builds (timings, compile/link/run, Clippy, RSS); use disposable checkout for edits. Inventory targets and test durations. Add lightweight opt-in phase counters and geometric microbenchmarks only where genuine routes exist; no missing-semantics implementation. Report raw logs and method outside doc. Rank measured bottlenecks and recommend the next package; A02 is a candidate only if the ranking supports it. Supply the measurement method to PERF-A06 (component/end-to-end baselines) and PERF-C05 (actual incremental/parallel benchmarks); neither requires A02 completion. Do not wait for full feature coverage or large workloads.

## Verification and exit

Existing test obligations unchanged; no proof, expectation, coverage, policy or receipt change. Verify measurement tooling on a small fixed case and counters do not change canonical output. Run task-appropriate local checks per AGENTS.md. Exit: reproducible baseline and ranked bottlenecks, unavailable measurements marked.
