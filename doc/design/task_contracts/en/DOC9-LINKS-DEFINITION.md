# Task DOC9-LINKS-DEFINITION: link-model proposal
Canonical language: English; [Japanese pointer](../ja/DOC9-LINKS-DEFINITION.md).
Status: frozen definition-proposal scope. Tier: full, documentation only; freeze and paired proposal form one changed task.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [task9](../../mizar-doc/en/todo.md#documentation-strand); consumers: Doc10 resolution and later index/render consumers. Dependency: bounded approved [Doc4 build inputs](../../mizar-doc/en/doc_build.md#full-build-and-index-inputs).
Purpose: define a non-normative link-model proposal before implementation; genuinely new semantic choices remain unselected owner Q-decisions.

## Authority And Readiness
Authority: spec [24.1–2](../../../spec/en/24.documentation_generation.md), [23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format); architecture13 [shared identity](../../architecture/en/13.documentation_and_extraction.md#documentation-and-extraction-share-symbol-identity), [Step3](../../architecture/en/13.documentation_and_extraction.md#step-3-resolve-links-and-render-math); internal05 [index](../../internal/en/05.documentation_extraction.md#documentation-index), [diagnostics](../../internal/en/05.documentation_extraction.md#diagnostics).
The [build-input boundary](../../mizar-doc/en/doc_build.md#index-inputs-and-later-completion) retains candidates only: import/dependency completeness, anchor collisions and UsedBy citation inputs remain undefined design_drift; absent resolution is deferred source_gap. No executable link test is adopted by this unit.
Doc4's approved requirements remain fixed. Doc9 definitions depend on Doc4, not Doc8 parsing or any Q9–Q18 outcome. A language-specification intent change stops for owner decision; alternatives here cannot silently change spec24.

## Scope And Ownership
Write only this contract pair, paired mizar-doc/links.md, and necessary paired crate-plan/TODO links. The Japanese contract is a pointer; links.md keeps full EN/JA parity and owns only the unadopted proposal.
Define candidate reference forms and parser-neutral handoff, complete symbol/publication provenance, current-module/import-order and FQN lookup, missing/ambiguous/unresolved states, import/dependency validation/completeness, page/anchor identity and collisions, section/URL routing, UsedBy citation inputs, deterministic ordering and presentation diagnostics.
Expose alternatives and affected adoption gates for every new observable choice. Preserve exact exported-label authority, lower-reader refusals, source-map limits, profile-only warning promotion and proof independence. No fallback or invented metadata.
Forbidden: Rust/Cargo/tests/fixtures/expectations/trace/spec edits, AGENTS/protocol edits, normative adoption, diagnostic codes/schema, lower-stage authority, dependency/import selection, Markdown parsing, math/rendering, extraction/publication or broader Doc4/5/crate closure. Do not decide Doc8 Q9–Q18.
Audit impact: no coverage, trace status or follow-up ownership changes; leave the coverage audit untouched.

## Review And Verification Matrix
No executable tests change. Six documentation acceptance families (cap six; additions require an authority citation):
1. Exact label/FQN inline code, expressions, explicit links and @see: all spec24 forms covered without selecting Doc8 recognition or new syntax.
2. Current-module hit/miss, ordered imports, duplicate labels/FQNs and cross-package/version collisions: preserve candidates and authority; ambiguous versus unresolved outcomes exposed.
3. Missing/empty/partial import vectors and dependency indexes, wrong publication identities or integrity failures: explicit completeness/adoption questions; no fabricated graph or source fallback.
4. Full/short item anchors, duplicate labels, page collisions and specification section/URL links: spec24 names retained; unresolved allocation/encoding choices unselected.
5. Direct by citations, used_axioms-only data, repeated/self/cross-package citations and unavailable metadata: exact proposed UsedBy evidence, provenance, ordering and completeness gates without proof credit.
6. EN/JA parity, deterministic diagnostics/maps, links/fragments, budgets and non-normative boundaries: no implementation, adoption, new diagnostics or Doc8 decisions.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency over the whole unit. Repeat specification only if the contract changes.
Verification for both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (links/fragments/pairing/volume included); git diff --check. Documentation only: fmt/Clippy/runtime/e2e unrun; full cargo test only on a named cadence trigger.
Exit: freeze then reviewed paired proposal committed locally; all new Q-decisions unselected, Doc10 and Doc8 implementation gates retained; no push.
