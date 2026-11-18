# Task DOC6-COMMENTS-DEFINITION: comment definition proposal
Canonical language: English; [Japanese pointer](../ja/DOC6-COMMENTS-DEFINITION.md).
Status: frozen residual-definition continuation within approved Q1–Q4 option ①. Tier: full, documentation only.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [task6](../../mizar-doc/en/todo.md#documentation-strand); consumers: Doc7 attachment, Doc8 Markdown, later links/math/rendering. Dependencies: Doc4 definitions, Doc5 planning and owner approval of Q1–Q4 option ①; amendment freeze plus paired proposal is one changed task.
Purpose: define comment inputs, verified attachment targets, Markdown and presentation diagnostics without implementing or deciding unowned semantics.

## Authority And Readiness
Authority: spec [2.9/2.10](../../../spec/en/02.lexical_structure.md#29-comments-and-annotations), [24.1–3](../../../spec/en/24.documentation_generation.md); architecture02 [source metadata](../../architecture/en/02.source_and_frontend.md#comments-and-doc-comments-are-source-metadata), architecture13 [attachment](../../architecture/en/13.documentation_and_extraction.md#step-2-attach-doc-comments); internal05 [inputs](../../internal/en/05.documentation_extraction.md#documentation-inputs), [diagnostics](../../internal/en/05.documentation_extraction.md#diagnostics).
Existing [lexical input](../../mizar-doc/en/artifact_reader.md#optional-native-lexical-source) preserves raw comments/maps, not grouping or attachment; [planning](../../mizar-doc/en/doc_build.md#full-build-and-index-inputs) preserves unresolved identities. Missing comments definitions are design_drift; absent attachment/Markdown implementations are deferred source_gap, not work in this unit.
Owner approval selects [comments Q1–Q4 option ①](../../mizar-doc/en/comments.md#owner-q-decisions). Q1 projection remains with Artifact/Driver; this continuation excludes parked Driver emission work and Doc consumption. Residual association/tag definitions are design_drift; genuinely new semantic choices remain proposed Q-decisions.

## Scope And Ownership
Write only this paired contract and paired mizar-doc/comments.md. The Japanese contract remains a pointer stub. The comments owner holds definitions and Q-decisions; this contract holds acceptance criteria.
Define Q2 module-leading trivia/multiple blocks and attachment diagnostics, and Q4 required arguments, duplicates, available-metadata mismatches and unavailable-parameter behavior. Preserve full provenance, raw text/maps, verified identity and existing Q1/Q3, CommonMark and lower-stage boundaries; never infer metadata from source text.
Consequential adaptations of approved ① choices may be defined; language-intent changes or genuinely new semantics must expose proposal alternatives and affected gates without selecting one. Doc7/8 still wait for remaining prerequisites/approvals; no runtime API, schema or diagnostic code is frozen here.
Forbidden: Rust/Cargo/test/fixture/expectation/trace/spec changes, protocol/AGENTS edits, deciding language intent or doc-comment source, new producer/proof/reuse/cache authority, attachment implementation, link/UsedBy/math/rendering/extraction/publication completion or claiming broader Doc4/5 closure.
Audit impact: coverage, trace status and follow-up ownership remain unchanged; no coverage-audit edit.

## Review And Verification Matrix
No executable tests are added or changed. Existing native lexical/published-input regressions remain raw-input evidence, not attachment coverage. Documentation acceptance has six cases (cap six):
1. Leading whitespace/blank lines, ordinary line/block comments and standalone versus attached annotations: distinguish module eligibility from item adjacency; expose any unowned boundary choice.
2. Multiple leading blocks, first named item, imports and later blocks: preserve separate raw blocks/order and module-only precedence without guessing verified targets or crossing interruptions.
3. Orphan, ambiguous, unsupported/unverified targets and missing/conflicting identity/maps: define presentation outcomes/ranges while preserving lower-reader integrity refusals and private-doc boundaries.
4. All seven tags with missing arguments/bodies, repeated names/targets and malformed/unknown forms: preserve text and specify warning versus deferred-policy boundaries without new version grammar or link resolution.
5. Available complete, unavailable or partial parameter/return metadata: define justified mismatch checks without signature parsing, inferred absence or proof effects.
6. EN/JA parity, links/fragments, budgets and handoff gates: retain approved ① choices, identify new Q-decisions if needed, leave Doc7/8/CommonMark, Q1 and parked emission work open.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency. Repeat specification only if frozen scope changes.
Verification for both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (links/fragments/pairing/volume included); git diff --check. Documentation only: no fmt/Clippy/owner-runtime checks. Full cargo test runs only on a protocol cadence trigger; this local documentation continuation is below the tenth outstanding task.
Exit: amendment freeze then reviewed paired residual definitions/proposed Q-decisions committed locally as one changed task. New Q-decisions require owner approval; no implementation or push.
