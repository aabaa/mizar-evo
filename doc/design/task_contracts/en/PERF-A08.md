# Task PERF-A08: Statement-level declarative inference benchmark
Canonical language: English; [Japanese pointer](../ja/PERF-A08.md).
Status: draft, full-tier benchmark proposal; review before dispatch. Dependencies: VC53-E1 chain complete/sealed, A01 ranking and A06 measurement method as available; coordinate active lanes. Baseline does not depend on new inference engines.
Owner: [Test plan](../../mizar-test/en/00.crate_plan.md).
Consumers: VC, Proof, ATP, Kernel, Core, Cache, Driver, Build.
Authority: [VC architecture](../../architecture/en/07.vc_generation.md), [ATP architecture](../../architecture/en/10.atp_backend_integration.md), [test strategy](../../architecture/en/20.test_strategy.md), [performance roadmap](../../performance_optimization_roadmap.md).
Gap: test/performance; statement distribution and relative inference costs unmeasured.

## Scope

Inventory actual supported statement/VC routes and current local discharge, finder, backend and independent checking. Measure matched premise/context/budget workloads: preparation, dispatch/IPC, search (including failures), extraction, kernel and replay, accepted success, latency percentiles and total throughput, RSS and cold/warm/edit. Use native theorem/algorithm corpus first, stratified by inference class; investigate faithful MML context/citation extraction as separate comparative input, flag incomplete mappings. Reuse A06 and existing ATPK-BENCHMARK harness where suitable; theorem re-proving remains separate. Compare current isolated route against available local/persistent/embedded prototypes on identical inputs; unsupported candidates remain unmeasured. Baseline and ranking precede D08 selection, extend later as producers become genuine. Store data outside doc.

## Verification and exit

No production inference or policy change, expectation/coverage change, invented accepted outcome, default strategy switch or fixed performance threshold. Test matrix (cap four families): known genuine accepted/rejected/open outcomes and timing attribution; repeatable workload selection and context identity; backend failure/timeout and acceptance distinction; counter instrumentation preserving canonical results. Review methodology, fidelity, test sufficiency, implementation/scope/consistency; local fmt, Clippy, both lint_policy suites, owner tests and full test cadence per AGENTS.md. Exit: reproducible initial measurements, ranked bottlenecks, success/latency distributions and explicit unavailable paths; recommend D08 candidate scope. No benchmark credit for semantic coverage.
