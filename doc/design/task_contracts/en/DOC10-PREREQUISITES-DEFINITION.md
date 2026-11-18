# Task DOC10-PREREQUISITES-DEFINITION: producer, allocation and diagnostic proposal
Canonical language: English; [Japanese pointer](../ja/DOC10-PREREQUISITES-DEFINITION.md).
Status: frozen definition scope. Tier: full, documentation only; freeze and paired proposal are one changed task.
Purpose: define non-normative prerequisites for Doc10 without adopting or implementing them.
Owners: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [Diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md); consumers: Doc10 and Doc13. Dependencies: adopted Doc9 and Doc13 href interface; Artifact-owned producers remain separate work.

## Authority And Readiness
Authority: [spec24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format), [spec22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display) and [22.6.8–9](../../../spec/en/22.error_handling_and_diagnostics.md#2268-documentation-attachment-warnings).
Boundaries: [Doc9](../../mizar-doc/en/links.md), [Doc13 destinations](../../mizar-doc/en/render.md#candidate-destinations-and-page-content), [Doc4 inputs](../../mizar-doc/en/doc_build.md#index-inputs-and-later-completion), [internal05 index](../../internal/en/05.documentation_extraction.md#documentation-index).
Diagnostic owners: [registry](../../mizar-diagnostics/en/registry.md#descriptor-metadata), [record](../../mizar-diagnostics/en/failure_record.md#documentation-primary-locations), [deduplication](../../mizar-diagnostics/en/aggregator.md#deduplication-identity), [ordering](../../mizar-diagnostics/en/aggregator.md#ordering-and-handles).
Gaps: missing authenticated import/dependency and direct-by producers, validated destinations and exact shared link diagnostics are deferred source_gap/design_drift; definition does not establish readiness. Existing lower-reader refusals remain errors.
Language-specification changes stop for owner input. Diagnostics, presentation, naming, provenance and internal structure are selected autonomously as proposal decisions; expose any genuinely unselected language Q without adopting it.

## Scope And Ownership
Write set: this contract pair; new paired mizar-doc/link_prerequisites.md and mizar-diagnostics/documentation_links.md; necessary paired owner-plan/TODO and links readiness pointers only.
Doc owner defines format-neutral producer authentication/completeness and page/anchor/section validation handoffs; Diagnostics owner defines exact meanings, descriptors, details, locations, deduplication, precedence, ordering and profiles. Link to existing invariants rather than duplicate them.
Define dependency order and independently checkable readiness evidence; preserve finite bound closure, identity/version distinctions, direct-only citations, exact FQN ids and percent-UTF8 href interface.
Forbidden: normative adoption, spec/code/schema/numeric allocations, producer or resolver implementation, new language syntax or semantics, other Doc13 choices, AGENTS/protocol edits, tests/fixtures/expectations/trace/coverage changes or milestone closure. Artifact ownership is unchanged; no source/evidence fallback.

## Documentation Acceptance Matrix
No executable test intent changes. Six families, cap six; additions require an authority citation:
1. Missing/partial/complete-empty ordered imports and finite dependency closure, cycles and wrong bindings: authenticate full identities; no inferred order, discovery or refusal downgrade.
2. Absent/partial/complete-empty direct-by coverage, repeated/self/cross-package edges and unmapped endpoints: Artifact ownership, occurrence/coverage provenance, no exports/used_axioms fallback.
3. Module paths, package/version collisions, exact full ids, optional aliases and version-bound sections: validated readiness and adopted href encoding; no directory/source-routing choice.
4. Unresolved/ambiguous/incomplete link and UsedBy conditions: exact distinct diagnostic descriptors, payloads, source/provenance locations and refusal precedence without numeric allocation.
5. Repeated/coincident/discontiguous/unmapped occurrences and input-level failures: deterministic identity/dedup/order, no fabricated spans or block ordinals; profiles affect documentation only.
6. EN/JA parity, links/fragments, budgets, ordered gates and non-normative boundaries: adoption/producer/allocation/diagnostic implementation remain separate.
Reviews: independent specification before freeze; sufficiency, precision, volume/scope and final consistency over the whole unit. Repeat specification only if this contract changes.
Verification: both lint_policy suites with CARGO_BUILD_JOBS=4, local links/fragments and git diff --check for each commit. Docs-only fmt/Clippy/runtime/e2e inapplicable; full cargo test required by tenth-outstanding-task cadence.
Exit: freeze then reviewed paired proposal committed locally, push:no; no Doc10 readiness or completion credit.
