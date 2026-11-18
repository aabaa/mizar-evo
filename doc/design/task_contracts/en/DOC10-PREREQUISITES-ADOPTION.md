# Task DOC10-PREREQUISITES-ADOPTION: bounded interface adoption
Canonical language: English; [Japanese pointer](../ja/DOC10-PREREQUISITES-ADOPTION.md).
Status: frozen adoption scope. Tier: full, documentation only; freeze and adoption are one changed task.
Purpose: adopt entry212's completed Doc10 prerequisite interfaces without supplying their inputs or implementation readiness.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md); consumers: Doc10 and Doc13. Dependencies: adopted Doc9 and Doc13 href interface; producer, allocation and diagnostic owners retain their gates.

## Authority And Readiness
Authority: [spec24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format), [architecture13 Step3](../../architecture/en/13.documentation_and_extraction.md#step-3-resolve-links-and-render-math), [internal05 index](../../internal/en/05.documentation_extraction.md#documentation-index).
Definitions: [prerequisite interfaces and ordered gates](../../mizar-doc/en/link_prerequisites.md), [Doc9 boundary](../../mizar-doc/en/links.md#authority-and-fixed-boundary), [Doc4 inputs](../../mizar-doc/en/doc_build.md#index-inputs-and-later-completion), [Doc13 hrefs](../../mizar-doc/en/render.md#candidate-destinations-and-page-content).
Deferred diagnostic owner: [exact shared link diagnostics](../../mizar-diagnostics/en/documentation_links.md); neither their adoption nor allocation is in scope.
Gap classification: design_drift is the unadopted interface status; source_gap remains authenticated ordered imports/finite closure, Artifact direct-by projection/coverage and validated page/anchor/alias/section allocations. Adoption closes only the status gap.
No language-specification intent change or new semantic choice is authorized; either requires stopping for owner input.

## Scope And Ownership
Adopt the existing format-neutral authenticated import/dependency, Artifact-owned direct-by and validated allocation definitions as the Doc10 prerequisite handoff contract, preserving their contents and lower-reader refusals.
Keep readiness order: interface adoption, authenticated imports/finite closure, Artifact projection/coverage, validated allocations, separate diagnostic adoption/allocation, then separately dispatched resolver integration.
Write set: this contract pair; paired mizar-doc/link_prerequisites.md status/gates and necessary paired links.md, 00.crate_plan.md and todo.md pointers only. Keep definitions single-owned; no copied payload model.
Forbidden: spec, code, tests/fixtures/expectations/trace, schema/API/numeric diagnostic allocation, producers/authentication/allocation implementation, Doc10 integration, ② alternatives, other Doc13/Doc23 choices, AGENTS/protocol changes and coverage/owner/milestone credit.
No specification-coverage audit impact: no behavior, implementation coverage, ownership or deferral status changes; existing producer/diagnostic deferrals remain open.

## Documentation Acceptance Matrix
No executable test intent changes. Six families, cap six; additions require an authority citation:
1. Interface status and paired pointers: completed definitions are adopted only as prerequisite interfaces; no claim that inputs, validators or resolver exist.
2. Missing/incomplete/complete-empty imports, cycles and wrong bindings: retain semantic order, finite authenticated closure, full identities, no discovery/fallback or refusal downgrade.
3. Repeated/self/cross-package direct-by occurrences and missing/unmapped/complete-empty coverage: retain Artifact authority, occurrence provenance, explicit scope and no source/exports/used_axioms reconstruction.
4. Page/full-id/alias collisions, unavailable labels, version-bound sections and non-ASCII/percent/hash hrefs: retain validation, exact ids and adopted encoding; no guessed paths, winner or routing choice.
5. Ordered readiness and diagnostics: every producer/authentication/allocation/diagnostic/implementation gate survives; shared diagnostics remain unadopted/unallocated and ② alternatives stay inactive.
6. EN/JA parity, local links/fragments, budgets and frozen write set: no spec/source/test/audit/protected-policy edits or completion credit.
Reviews: independent specification before freeze; sufficiency, precision, volume/scope and final consistency over the whole unit. Repeat specification only if the contract changes.
Verification: CARGO_BUILD_JOBS=4 both lint_policy suites, local links/fragments and git diff --check for each commit. Docs-only fmt/Clippy/runtime/e2e inapplicable; full cargo test only on a cadence trigger.
Exit: freeze then reviewed paired interface adoption committed locally, push:no; all later readiness gates remain open.
