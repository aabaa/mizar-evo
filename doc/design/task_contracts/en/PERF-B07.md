# Task PERF-B07: Integrated persistent knowledge and retrieval architecture
Canonical language: English; [Japanese pointer](../ja/PERF-B07.md).
Status: draft, design-only full-tier proposal; review before dispatch. Dependencies: VC53-E1 chain complete and sealed, PERF-A01 ranking and development-efficiency priority; coordinate A07 baselines and active lanes.
Owner: [Artifact plan](../../mizar-artifact/en/00.crate_plan.md).
Consumers: Core, Checker, IR, Resolve, Cache, Driver, Build, VC, Proof, Kernel, ATP, LSP, Doc, Test.
Authority: [package specification](../../../spec/en/23.package_management_and_build_system.md), [artifact architecture](../../architecture/en/11.artifact_and_incremental_build.md), [store design](../../mizar-artifact/en/store.md), [performance roadmap](../../performance_optimization_roadmap.md).
Gap: performance/design; static evidence in roadmap source anchors; no semantic or coverage credit.

## Scope

Inventory genuine readers/producers, representation identities and existing indexes. Investigate LEO-II/Leo-III, E, Vampire, Lean, Isabelle, Agda and relevant Zipperposition sources/literature; compare retrieval relations, sharing, index sizes and update patterns. Define candidate module-local typed DAG and optional session/package sharing, immutable binary section layout, derived persistent indexes and local overlays. Specify owner boundaries, exact candidate validation, binder/type/visibility semantics, deterministic identities, partial integrity verification and JSON compatibility/migration. Use A07 measurement design to distinguish codec, sharing, index and compression effects. Record design alternatives and open decisions in paired owner docs; no new crate without demonstrated need. Coordinate B01/B02/B04/B06 before their affected representations freeze; preserve existing A01 ranking, Step 6/7 and MVM gates.

## Forbidden and exit

No production code, published schema/spec change, acceptance change, source replay removal, benchmark claims, new coverage, receipt policy change or forced adoption of a specific codec/index in this design unit. No weakening of manifest validation or independent Kernel evidence. Later implementation requires separately reviewed owner contracts and approved specification migration.
Review specification correspondence, ownership/dependency direction, trust and performance methodology; inspect existing genuine fixtures and readers. Design test matrix (future owner contracts, not executable additions here): identical semantic data round-trip across formats; corrupt/stale/wrong-context data rejects or safely rebuilds; exhaustive reference retrieval equivalence including binder/type/visibility restrictions; cold/warm/edit workload comparison including index bytes. Local both lint_policy suites and link/fragment checks per AGENTS.md; other gates by protocol. Exit: reviewed paired owner design, documented candidate choices and benchmark plan, dependency-ready successor scopes; unresolved choices explicit.
