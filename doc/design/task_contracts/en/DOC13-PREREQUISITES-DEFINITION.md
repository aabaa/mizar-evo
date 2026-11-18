# Task DOC13-PREREQUISITES-DEFINITION: render prerequisite completion
Canonical language: English; [Japanese pointer](../ja/DOC13-PREREQUISITES-DEFINITION.md).
Status: frozen definition scope. Tier: full, documentation only; freeze and paired completion form one changed task.
Purpose: complete bounded non-normative Doc13 prerequisite definitions, without adoption or implementation.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [task13](../../mizar-doc/en/todo.md#documentation-strand); consumers: Doc14/15. Dependencies: adopted Doc8/9 and Q3 href interface, Doc10 producer/allocation proposals; Doc11/MT readiness remains separate.

## Authority And Readiness
Authority: [spec24.1–4](../../../spec/en/24.documentation_generation.md), architecture13 [Step4](../../architecture/en/13.documentation_and_extraction.md#step-4-emit-static-site) and [determinism](../../architecture/en/13.documentation_and_extraction.md#outputs-are-deterministic), internal05 [index](../../internal/en/05.documentation_extraction.md#documentation-index).
Owners: [Doc13 proposal](../../mizar-doc/en/render.md), [Doc4 inputs](../../mizar-doc/en/doc_build.md#index-inputs-and-later-completion), [Doc10 prerequisites](../../mizar-doc/en/link_prerequisites.md), [shared link diagnostics](../../mizar-diagnostics/en/documentation_links.md).
Gaps: unfinished identity/allocation handoffs, search encodings and auxiliary serialization are design_drift; absent producers/renderers are deferred source_gap. Definitions never establish producer readiness or normative adoption.
Preserve adopted Q3 percent-UTF8 hrefs and exact FQN DOM ids. Doc13 Q1–Q12 ① were screened as internal/presentation choices; define only this completion scope. Language-specification intent changes or genuinely new semantic choices stop for owner input; leave other owner gates explicit.

## Scope And Ownership
Write set: this contract pair, paired mizar-doc/render_prerequisites.md, narrow paired render.md completion pointers and necessary paired Doc plan/TODO navigation only.
The new owner completes proposal-only identity projection and allocation handoffs, exact search value encodings/compatibility, cross-reference/UsedBy fields, occurrence ordering and completeness serialization, and independently checkable ordered readiness gates. Link existing producer/diagnostic invariants instead of duplicating them.
Preserve full identities, unavailable/incomplete/complete-empty distinctions, authoritative strings, direct-only citation provenance, lower-reader refusals, safe unlinked degradation and source-map limits.
Forbidden: normative adoption, spec/code/Cargo/schema/numeric allocations, renderer/producer/publication implementation, ② alternatives, new language behavior, source/evidence fallback, Doc11/MT decisions, diagnostic-owner definitions, AGENTS/protocol edits, tests/fixtures/expectations/trace/coverage changes or milestone closure.
Doc23 owns atomic publication, Doc24 executable determinism; optional DocIndex schema remains separate. No coverage or follow-up ownership changes; leave the coverage audit untouched.

## Documentation Acceptance Matrix
No executable test intent changes. Six families, cap six; additions require an authority citation:
1. Distinct package/version/lock/module/edition/origin/publication identities and duplicate FQNs: lossless candidate encoding and authenticated handoff, never guessed components or identity winners.
2. Complete/missing/colliding page/full-anchor/alias/section/source allocations: existing validation ownership, adopted href behavior and explicit source-routing gate.
3. Export/reexport, opaque kinds/signatures, Unicode/control data, absent/empty/unavailable summaries and incompatible formats: exact search projection, deterministic bytes and refusal without omitted exports.
4. Resolved/unresolved/ambiguous/incomplete references, repeated/discontiguous/unmapped occurrences: identity-bound auxiliary records, explicit provenance and stable source order without fabricated spans.
5. Repeated/self/cross-package direct-by edges, unavailable/partial/complete-empty coverage: lossless occurrence records and separately ordered/deduplicated UsedBy views without evidence inference.
6. EN/JA parity, links/fragments, budgets and ordered readiness: adoption, diagnostic/math/summary/template/asset/producer implementation gates remain explicit.
Reviews: independent specification before freeze; sufficiency, precision, volume/scope and final consistency over the whole unit. Repeat specification only if the contract changes.
Verification for each commit: CARGO_BUILD_JOBS=4 both lint_policy suites, local links/fragments and git diff --check. Docs-only fmt/Clippy/runtime/e2e inapplicable. Final-tree full cargo test required for the released push and inherited outstanding-task cadence.
Exit: freeze then reviewed paired completion committed; push only after final-tree full success. No Doc13 adoption, Doc14/15 readiness or completion credit.
