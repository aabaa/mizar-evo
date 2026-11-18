# Task DOC8-PRESENTATION-DIAGNOSTICS-DEFINITION: Markdown/tag warning proposal
Canonical language: English; [Japanese pointer](../ja/DOC8-PRESENTATION-DIAGNOSTICS-DEFINITION.md).
Status: frozen definition-proposal scope. Tier: full, documentation only; freeze and paired proposal form one changed task.
Primary owner: [diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md); consumer: [Doc plan](../../mizar-doc/en/00.crate_plan.md), Doc8 parsing and coherent adoption.
Purpose: define non-normative Markdown/tag presentation diagnostic candidates required before Doc8 adoption; every new semantic choice remains an unselected owner Q-decision.

## Authority And Readiness
Authority: spec [24.1](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax), [22.1.1–2](../../../spec/en/22.error_handling_and_diagnostics.md#2211-error-code-system-e-series), [22.6.8](../../../spec/en/22.error_handling_and_diagnostics.md#2268-documentation-attachment-warnings); internal05 [diagnostics](../../internal/en/05.documentation_extraction.md#diagnostics); comments [tag validation](../../mizar-doc/en/comments.md#residual-tag-validation) and [diagnostics](../../mizar-doc/en/comments.md#diagnostics-and-determinism).
Fixed input: owner approved [Doc8 Q9–Q18 candidate ①](../../mizar-doc/en/commonmark.md#unselected-owner-q-decisions) on 2026-11-18; approval resolves those questions but is not coherent adoption. Comments Q1–Q8 and adopted attachment diagnostics remain binding. The proposal's historical unselected wording is not a new decision gate here.
Missing Markdown/tag descriptors and emission/location details are design_drift; [unallocated Documentation codes](../../mizar-diagnostics/en/registry.md#documentation-attachment-allocation) require owner definition before allocation. Absent Doc8 implementation is deferred source_gap. Language-specification intent changes stop for owner decision.

## Scope And Ownership
Write only this contract pair, paired diagnostics/documentation_markdown_tags.md, and necessary task-index links in paired diagnostics crate plans. Japanese contract is a pointer; proposal keeps full EN/JA parity and owns only unadopted definitions.
Define warning meanings, descriptor metadata, locations, multiplicity/order and profile behavior for approved Doc8 ①: known-tag slots/bodies, duplicates, established metadata mismatch, bounded math-extension failures and actual conversion/map failures. Use stable owner links for recognition and raw comparison, without copying the parser definition.
Expose alternatives for genuinely new descriptor granularity, code allocation, emission and location choices; do not select or reserve any code. W0400–W0404 retain attachment meanings. Preserve mapped/provenance locations, full identities, text retention and lower-reader refusal boundaries.
Forbidden: implementation, Rust/Cargo/tests/fixtures/expectations/trace/spec or registry edits, normative adoption, AGENTS/protocol edits, Doc8 ② alternatives, new tag grammar, link/math-rendering policy, Doc9 Q1–Q10 or Doc11 Q1–Q12 decisions, proof/extraction acceptance changes or broader crate closure.
Audit impact: no coverage, trace status or follow-up ownership change; leave the coverage audit untouched. Lower-stage and later diagnostic ownership stays unchanged.

## Documentation Acceptance Matrix
No executable tests change. Six acceptance families (cap six; additions require an authority citation):
1. Seven known tags with absent slots/bodies, malformed-looking raw tokens and unknown/lookalike tags: exact inherited warning conditions and explicit exclusions, no added grammar.
2. Parameter, singleton and repeatable duplicates across same-target blocks; complete/partial/absent metadata: retained raw comparisons, source order, no inferred absence or link resolution.
3. Protected code/HTML/link boundaries, unmatched eligible dollars, W-only display payloads, valid CommonMark recovery and conversion/map failure: approved warning/degradation boundary retained.
4. Candidate descriptors, permanent identities, summaries and allocation alternatives: full metadata policy, no reuse of attachment codes or actual allocation.
5. Bound mapped ranges, discontiguous or missing maps, overlapping conditions and reordered inputs: location/multiplicity/order candidates without fabricated spans or new shared deduplication policy; profile promotion affects documentation only.
6. EN/JA parity, links/fragments, budgets and non-normative status: new Q-decisions unselected; diagnostics definition precedes separate coherent adoption and implementation; Doc9/Doc11 decisions stay unselected.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency over the whole unit. Repeat specification only if the contract changes.
Verification for both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (links/fragments/pairing/volume included); git diff --check. Documentation only: fmt/Clippy/runtime/e2e unrun; full cargo test only on a named cadence trigger.
Exit: freeze then reviewed paired proposal committed locally; new Q-decisions unselected, adoption and implementation remain gated; no push.
