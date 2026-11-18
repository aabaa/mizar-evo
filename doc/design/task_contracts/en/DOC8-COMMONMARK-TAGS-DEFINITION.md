# Task DOC8-COMMONMARK-TAGS-DEFINITION: CommonMark and tag-field proposal
Canonical language: English; [Japanese pointer](../ja/DOC8-COMMONMARK-TAGS-DEFINITION.md).
Status: frozen definition-proposal scope. Tier: full, documentation only; freeze and paired proposal form one changed task.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [task8](../../mizar-doc/en/todo.md#documentation-strand); consumers: Doc8 parsing, later links/math/rendering. Dependencies: Doc7 attachment and approved [Q1–Q8 option ①](../../mizar-doc/en/comments.md#owner-q-decisions).
Purpose: supply a non-normative CommonMark/tag-field definition proposal; expose new observable choices as unselected owner Q-decisions, without implementation or adoption.

## Authority And Readiness
Authority: spec [2.9.2](../../../spec/en/02.lexical_structure.md#292-annotations), [24.1–3](../../../spec/en/24.documentation_generation.md); architecture13 [attachment](../../architecture/en/13.documentation_and_extraction.md#step-2-attach-doc-comments); internal05 [inputs](../../internal/en/05.documentation_extraction.md#documentation-inputs), [diagnostics](../../internal/en/05.documentation_extraction.md#diagnostics).
[Comments definitions](../../mizar-doc/en/comments.md#markdown-and-structured-tags) own approved extraction, tag validation and handoffs. Exact CommonMark/tag-field recognition is missing design_drift; absent Markdown parsing is deferred source_gap. Spec24.1's adjacent tag-line example and paragraph-first-token rule need explicit owner clarification, not silent reinterpretation.
Doc7 supplies attached raw blocks/maps, not Markdown products. No parser dependency is currently adopted; propose an exact parser/version and CommonMark revision, with extension interactions preserving the complete required surface. Genuine language-intent changes stop for owner decision.

## Scope And Ownership
Write only this contract pair, paired mizar-doc/commonmark.md proposal, and necessary links in paired comments.md, crate plan and TODO. The Japanese contract is a pointer; the proposal keeps full EN/JA parity. Comments retains approved policy ownership; commonmark.md owns only the unadopted proposal.
Define candidate paragraph/tag boundaries, marker eligibility, name/ref/version slots, body extent/nonemptiness, raw duplicate comparison/scope, source mappings, code/link/math/HTML interactions and graceful degradation. Cover all seven tags under approved Q7/Q8; preserve unknown tags and existing metadata-availability limits.
Every genuinely new semantic choice remains explicitly unselected, including alternatives and affected gates. No option-2 alternative to approved Q1–Q8 may be adopted or built. Doc8 implementation remains gated on owner clarification/adoption and diagnostic prerequisites.
Forbidden: Rust/Cargo/test/fixture/expectation/trace/spec changes, AGENTS/protocol edits, normative adoption, new diagnostic codes/schema, lower-stage or proof authority changes, source fallback, link resolution, math/HTML rendering, extraction/publication or broader Doc4/5/crate closure.
Audit impact: no coverage, trace status or follow-up ownership changes; leave the coverage audit untouched.

## Review And Verification Matrix
No executable tests change. Six documentation acceptance families (cap six; extra cases require an authority citation):
1. Full CommonMark paragraphs/headings/lists/quotes/code/links/images/HTML and proposed exact parser/version/options: no reduced grammar or accidental optional extensions.
2. Blank-separated versus adjacent tag lines, continuations, tight/loose lists, quotes and first-token lookalikes: exact candidate recognition and explicit unselected ambiguity decisions.
3. All seven known tags and unknown/escaped/code/formatted forms; missing or multiline name/ref/version/body fields: raw boundaries, retention and approved Q8 warnings without new field grammars.
4. Parameter/singleton/repeatable duplicates, whitespace/markup differences and separate blocks for one target: Q7-compatible keys/scope, retained order, available/partial/absent metadata limits.
5. Code, standard/cross-reference links, inline/display math, HTML, escapes and malformed-looking Markdown: explicit precedence/degradation candidates, mapped ranges and later-owner handoffs without resolution/rendering or new codes.
6. EN/JA parity, links/fragments, budgets and non-normative status: preserve Q1–Q8, attachment/proof/lower-reader boundaries and implementation gates.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency over the entire unit. Repeat specification only if scope changes.
Verification for both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (including links/fragments/pairing/volume); git diff --check. Documentation only: fmt/Clippy/runtime/e2e unrun. Full cargo test only on a named cadence trigger.
Exit: freeze then independently reviewed paired proposal committed locally; new Q-decisions unselected, Doc8 parsing still open, no push.
