# Task DOC13-RENDER-DEFINITION: static-site rendering proposal
Canonical language: English; [Japanese pointer](../ja/DOC13-RENDER-DEFINITION.md).
Status: frozen definition-proposal scope. Tier: full, documentation only; freeze and paired proposal form one changed task.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [task13](../../mizar-doc/en/todo.md#documentation-strand); consumers: Doc14 HTML and Doc15 search emission. Dependency: bounded approved [Doc4 inputs](../../mizar-doc/en/doc_build.md#full-build-and-index-inputs).
Purpose: define a non-normative static HTML layout, search-index format and deterministic-output proposal; new semantic choices remain unselected owner Q-decisions.

## Authority And Readiness
Authority: spec [24.1–4](../../../spec/en/24.documentation_generation.md); architecture13 [Step4](../../architecture/en/13.documentation_and_extraction.md#step-4-emit-static-site), [determinism](../../architecture/en/13.documentation_and_extraction.md#outputs-are-deterministic); internal05 [index](../../internal/en/05.documentation_extraction.md#documentation-index), [diagnostics](../../internal/en/05.documentation_extraction.md#diagnostics).
Missing render definitions are design_drift; absent HTML/search emission is deferred source_gap. Existing raw exports are not a complete render index; [input limits](../../mizar-doc/en/doc_build.md#index-inputs-and-later-completion) remain binding. No executable render test is adopted here.
Fixed inputs: owner approval of Doc8 Q9–Q18 all ① (2026-11-18 17:50 JST) and Doc9 Q1–Q10 all ① (2026-11-18). Their proposal files retain historical unselected wording; this unit neither re-decides nor adopts them. Doc11 Q1–Q12 and Doc8 MT-Q1–Q7 remain unselected, not readiness prerequisites for this proposal; expose affected rendering dependencies. Doc8 coherent adoption and implementation remain separate gates. A language-specification intent change stops for owner decision.

## Scope And Ownership
Write only this contract pair, paired mizar-doc/render.md and necessary paired crate-plan/TODO links. The Japanese contract is a pointer; render.md keeps full EN/JA parity and owns the unadopted proposal.
Define package/module/item destinations and contents, version/source/asset routing, search records and client behavior, cross-reference/UsedBy and optional machine-readable index boundaries, canonical bytes/order, safe HTML/URL/math handoffs, incomplete-input diagnostics and adoption gates. Alternatives must preserve spec24 and approved inputs.
Preserve full publication/module/export identity, absent/empty distinctions, source-map limits, immutable artifact/proof authority, lower-reader refusals and profile-only warning promotion. Doc23 owns output publication/atomic writes; Doc24 owns executable determinism tests.
Forbidden: Rust/Cargo/tests/fixtures/expectations/trace/spec edits, AGENTS/protocol edits, normative adoption, diagnostic code/schema allocation, source re-analysis, dependency selection, Doc8/Doc9 re-decision, Doc11/MT decisions, HTML/search implementation, extraction/publication or broader Doc4/5/crate closure.
Audit impact: no coverage, trace status or follow-up ownership changes; leave the coverage audit untouched.

## Review And Verification Matrix
No executable tests change. Six documentation acceptance families (cap six; additions require an authority citation):
1. Package/module/item paths, nested/index-like names, version and full-identity collisions: deterministic relative destinations, fixed anchors and explicit unselected routing/refusal choices.
2. Complete/absent comments, signatures/contracts/tags/UsedBy/source; raw HTML, URLs and math: required page content and safe handoffs without invented semantic data or decisions for Doc11/MT.
3. Exported/reexported items, duplicate FQNs, absent versus empty summaries/signatures and package version: concrete search format, cross-reference/UsedBy and optional index boundaries, identity traceability and client query candidates without omission or collision winners.
4. Reordered inputs/workers, semantic source/import order, Unicode/escaping/newlines and assets: canonical HTML/JSON/output rules; timestamps, local paths and nondeterministic state excluded.
5. Missing maps/dependency/math projections, malformed presentation, output collisions and failed writes: deterministic diagnostics and readiness boundaries, no proof/extraction effects, atomic publication delegated.
6. EN/JA parity, links/fragments, budgets, Doc14/15 adoption gates and unchanged spec/tests/audit: definition only, all new Q-decisions unselected.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency over the whole unit. Repeat specification only if the contract changes.
Verification for both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (links/fragments/pairing/volume included); git diff --check. Documentation only: fmt/Clippy/runtime/e2e unrun; full cargo test only on a named cadence trigger.
Exit: freeze then reviewed paired proposal committed locally; all new Q-decisions unselected, separate coherent adoption and Doc14/15 prerequisites retained; no push.
