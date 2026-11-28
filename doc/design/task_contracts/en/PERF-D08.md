# Task PERF-D08: Automatic and guided inference architecture design
Canonical language: English; [Japanese pointer](../ja/PERF-D08.md).
Status: draft, design-only full-tier proposal; review before dispatch. Dependencies: initial PERF-A08 results after A01/VC53-E1, development-efficiency ranking; coordinate B07/B08 and existing D03–D07/SMT owner designs.
Owner: [Proof plan](../../mizar-proof/en/00.crate_plan.md).
Consumers: VC, ATP, Kernel, Core, Checker, Cache, Driver, Build, Parser, Test.
Authority: [VC architecture](../../architecture/en/07.vc_generation.md), [ATP architecture](../../architecture/en/10.atp_backend_integration.md), [ATP specification](../../../spec/en/21.source_code_annotation_and_atp.md), [Algorithm specification](../../../spec/en/20.algorithm_and_verification.md), [performance roadmap](../../performance_optimization_roadmap.md).
Gap: architecture/performance; selection tradeoffs not measured.

## Scope

Use A08 data to compare existing local discharge, cheap checked inference, Metis-like bounded first-order search, isolated/resident/embedded ATP and SMT, and Algorithm-based proof automation. Preserve declarative by default; optional manual strategy, strict versus explicitly requested fallback, configuration scope/precedence and explanatory diagnostics. Specify shared evidence candidate boundary, allowed premise/context, routing/cancellation/resource limits and deterministic acceptance/selection policy. Compare sequential cascade against feature-aware or parallel dispatch including unsuccessful overhead. Preserve separate owner semantics: VC local discharge, ATP backend search, Proof routing/acceptance, independent Kernel check; D09–D12 are separately frozen measured successors. No separate tactic language: assess existing Algorithm/reflected syntax and minimal generic proof-facing extension only if required. Distinguish proof discovery from exact-context evidence replay and existing C03/C04 trust rules.

## Verification and exit

Design only: no Rust, grammar, policy, accepted status, new crate, new tactic language, receipt or coverage change. Candidate test matrix (future implementation contracts, cap five families): automatic/manual strict/fallback and unchanged premises; positive checked evidence across supported routes; unsupported/timeout/crash remains nonaccepted; stale/cross-context evidence or solver state fails closed; Algorithm untrusted output rejected and ordinary by unchanged. Review specification correspondence, ownership, benchmark rationale, trust/precision, scope and consistency; both lint_policy suites and links/fragments per AGENTS.md. Exit: reviewed paired owner design and ranked candidate successor contracts; explicit open decisions and separate specification approval before syntax or acceptance changes.
