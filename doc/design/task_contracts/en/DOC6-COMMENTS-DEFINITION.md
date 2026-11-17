# Task DOC6-COMMENTS-DEFINITION: comment definition proposal
Canonical language: English; [Japanese pointer](../ja/DOC6-COMMENTS-DEFINITION.md).
Status: frozen definition scope; proposal awaits owner decisions. Tier: full, documentation only.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [task6](../../mizar-doc/en/todo.md#documentation-strand); consumers: Doc7 attachment, Doc8 Markdown, later links/math/rendering. Dependencies: Doc4 definitions and Doc5 planning; freeze plus paired proposal is one changed task.
Purpose: define comment inputs, verified attachment targets, Markdown and presentation diagnostics without implementing or deciding unowned semantics.

## Authority And Readiness
Authority: spec [2.9/2.10](../../../spec/en/02.lexical_structure.md#29-comments-and-annotations), [24.1–3](../../../spec/en/24.documentation_generation.md); architecture02 [source metadata](../../architecture/en/02.source_and_frontend.md#comments-and-doc-comments-are-source-metadata), architecture13 [attachment](../../architecture/en/13.documentation_and_extraction.md#step-2-attach-doc-comments); internal05 [inputs](../../internal/en/05.documentation_extraction.md#documentation-inputs), [diagnostics](../../internal/en/05.documentation_extraction.md#diagnostics).
Existing [lexical input](../../mizar-doc/en/artifact_reader.md#optional-native-lexical-source) preserves raw comments/maps, not grouping or attachment; [planning](../../mizar-doc/en/doc_build.md#full-build-and-index-inputs) preserves unresolved identities. Missing comments definitions are design_drift; absent attachment/Markdown implementations are deferred source_gap, not work in this unit.
Doc4 [Q4](../../mizar-doc/en/doc_build.md#owner-q-decisions) remains open: artifact projection versus source preprocessing/frontend transport. This unit frames that and other genuinely unowned choices as owner Q-decisions; it selects no alternative or new dependency.

## Scope And Ownership
Write only this paired contract, paired mizar-doc/comments.md and necessary paired Doc crate-plan/task6–8 TODO links/status or gating text. The comments owner holds definitions and Q-decisions; this contract holds acceptance criteria.
Define source/provenance and coordinate requirements, module/item targets and adjacency, Markdown/CommonMark features and structured tags, graceful diagnostics, deterministic ordering, and lower-stage/consumer boundaries. Preserve complete publication/module/item identity and unavailable-versus-empty inputs; never infer semantic identity from source text or FQN parsing.
Any unresolved source transport, association precedence, normalization or presentation semantics must expose alternatives and affected gates for owner approval. Doc7/8 implementation waits for completed definitions and the relevant owner decisions; no runtime API or serialized schema is frozen here.
Forbidden: Rust/Cargo/test/fixture/expectation/trace/spec changes, protocol/AGENTS edits, deciding language intent or doc-comment source, new producer/proof/reuse/cache authority, attachment implementation, link/UsedBy/math/rendering/extraction/publication completion or claiming broader Doc4/5 closure.
Audit impact: coverage, trace status and follow-up ownership remain unchanged; no coverage-audit edit.

## Review And Verification Matrix
No executable tests are added or changed. Existing native lexical/published-input regressions remain raw-input evidence, not attachment coverage. Documentation acceptance has six cases (cap six):
1. Artifact-only, hash-bound native input and unavailable/bad source: distinguish transport candidates, empty versus unavailable comments and integrity refusals; Q4 stays owner-gated without fallback.
2. Contiguous/blank/interrupted blocks, file-leading comments and each named item kind: state spec association rules and expose unresolved precedence or boundary choices, with no syntactic candidate promoted to verified identity.
3. Colliding module/FQN/item candidates, non-exported/invalid targets and source-coordinate mismatch: retain full identities, require verified association, preserve diagnostics and private-doc boundaries without invented labels.
4. Paragraphs, lists, fenced code, block quotes, inline code, inline/display math, explicit links and all seven structured tags: preserve raw text/ranges; separate parsing from later resolution/rendering and preserve unknown tags verbatim.
5. Malformed Markdown/tags, unattached comments, missing source and promoted warnings: deterministic presentation diagnostics and graceful degradation cannot change proof validity; unowned policies are Q-decisions.
6. EN/JA parity, links/fragments, budgets and handoff gates: paired proposal states the same choices; Doc7/8 and broader consumers remain unimplemented.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency. Repeat specification only if frozen scope changes.
Verification for both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (links/fragments/pairing/volume included); git diff --check. Documentation only: no fmt/Clippy/owner-runtime checks. Full cargo test is required on the final proposal tree by the tenth-outstanding-task cadence trigger.
Exit: reviewed paired proposal and owner Q-decision handoff committed locally; approval remains prerequisite to implementation. No push.
