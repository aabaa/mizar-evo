# Task DOC11-MATH-DEFINITION: math-model proposal
Canonical language: English; [Japanese pointer](../ja/DOC11-MATH-DEFINITION.md).
Status: frozen definition-proposal scope. Tier: full, documentation only; freeze and paired proposal form one changed task.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [task11](../../mizar-doc/en/todo.md#documentation-strand); consumer: Doc12 math rendering and later HTML rendering. Dependency: bounded approved [Doc4 inputs](../../mizar-doc/en/doc_build.md#full-build-and-index-inputs).
Purpose: define a non-normative math-rendering proposal; genuinely new semantic choices remain unselected owner Q-decisions.

## Authority And Readiness
Authority: spec [21.3](../../../spec/en/21.source_code_annotation_and_atp.md#213-latex-expression-annotations), [24.3–4](../../../spec/en/24.documentation_generation.md#243-mathematical-formula-rendering); architecture13 [Step3](../../architecture/en/13.documentation_and_extraction.md#step-3-resolve-links-and-render-math), [determinism](../../architecture/en/13.documentation_and_extraction.md#outputs-are-deterministic); internal05 [artifact inputs](../../internal/en/05.documentation_extraction.md#artifact-inputs), [diagnostics](../../internal/en/05.documentation_extraction.md#diagnostics).
The [input boundary](../../mizar-doc/en/doc_build.md#index-inputs-and-later-completion) retains stored signatures/LaTeX only when present. Complete structured formula/annotation projections and their application, numbering and engine policies remain undefined design_drift; absent rendering is deferred source_gap. No executable math test is adopted here.
Preserve no build-time LaTeX syntax validation, client-side KaTeX output and approved Doc4 requirements. This definition depends on Doc4, not Doc8 adoption. Doc8 Q9–Q18 and Doc9 Q1–Q10 remain unselected. A language-specification intent change stops for owner decision.

## Scope And Ownership
Write only this contract pair, paired mizar-doc/math.md and necessary paired crate-plan/TODO links. The Japanese contract is a pointer; math.md keeps full EN/JA parity and owns only the unadopted proposal.
Define parser-neutral math inputs/provenance, @latex priority/application, default symbol/formula rendering, the LaTeX repertoire and engine handoff, numbering/references, deterministic output, graceful degradation and presentation diagnostics. Expose alternatives and adoption gates for every new observable choice.
Retain lower-reader refusals, absent/empty distinctions, source-map limits, immutable artifact/proof authority and profile-only warning promotion. No invented semantic projections or fallback parsing.
Forbidden: Rust/Cargo/tests/fixtures/expectations/trace/spec edits, AGENTS/protocol edits, normative adoption, diagnostic codes/schema, source re-analysis, dependency selection, Markdown/delimiter decisions, link-policy adoption, HTML implementation, extraction/publication or broader Doc4/5/crate closure.
Audit impact: no coverage, trace status or follow-up ownership changes; leave the coverage audit untouched.

## Review And Verification Matrix
No executable tests change. Six documentation acceptance families (cap six; additions require an authority citation):
1. Bound annotation/formula projections versus absent/empty/incomplete/wrong-publication inputs: preserve provenance and refusal/readiness boundaries without source fallback.
2. Annotated functor/predicate/mode/attribute occurrences, parameter renaming and nesting, unannotated symbols: annotation priority and verbatim monospace fallback preserved; new substitution choices unselected.
3. Every spec24.3 default translation, nested/unsupported syntax, signatures/statements/contracts: complete fixed surface, precedence and unsupported-input gates without invented semantics.
4. Parser-supplied inline/display payloads, invalid/unsupported LaTeX and engine errors: no delimiter decision or build-time validation; client-side KaTeX, repertoire/configuration and safe degradation gates explicit.
5. Display equation numbering/references across pages, reordered inputs and collisions: spec24.3 format retained; allocation/reference routing and deterministic engine/output choices unselected.
6. EN/JA parity, deterministic diagnostics/maps, links/fragments, budgets and boundaries: no implementation, normative adoption, new diagnostics, Doc8 or Doc9 decision.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency over the whole unit. Repeat specification only if the contract changes.
Verification for both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (links/fragments/pairing/volume included); git diff --check. Documentation only: fmt/Clippy/runtime/e2e unrun; full cargo test only on a named cadence trigger.
Exit: freeze then reviewed paired proposal committed locally; all new Q-decisions unselected, Doc12/Doc8 implementation and Doc9 adoption gates retained; no push.
