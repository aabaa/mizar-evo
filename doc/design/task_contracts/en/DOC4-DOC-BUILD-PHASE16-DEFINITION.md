# Task DOC4-DOC-BUILD-PHASE16-DEFINITION: planning proposal
Canonical language: English; [Japanese pointer](../ja/DOC4-DOC-BUILD-PHASE16-DEFINITION.md).
Status: frozen definition scope; proposal awaits owner decisions. Tier: full, documentation only.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [task4](../../mizar-doc/en/todo.md#inputs); consumers: Doc5 planning/index, Doc6 comment definitions, Doc7 attachment after Doc5/6.
Purpose: extend the paired doc_build owner with phase-16 request, package planning, index-input, determinism and consumer-boundary definitions; freeze plus proposal is one changed task.

## Authority And Readiness
Authority: spec [23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format), [24.1–4](../../../spec/en/24.documentation_generation.md); architecture13 [consumer rules](../../architecture/en/13.documentation_and_extraction.md#design-decisions), [interfaces](../../architecture/en/13.documentation_and_extraction.md#interface-definitions); internal05 [data model](../../internal/en/05.documentation_extraction.md#data-model), [control flow](../../internal/en/05.documentation_extraction.md#control-flow).
Existing [published reader](../../mizar-doc/en/artifact_reader.md) and [build inputs](../../mizar-doc/en/doc_build.md) supply structural metadata only. Missing full definitions are design_drift; architecture/internal version and source optionality conflicts are unresolved owner choices, not permission to choose semantics.
Doc6 retains comment-source/attachment decisions. General Artifact emission, current-build binding, downstream producers/publication and broader caching remain gated by their owners.

## Scope And Ownership
Write only this EN/JA contract, paired doc_build.md, and necessary Doc crate-plan/task4 TODO links/status. Keep module definitions solely in doc_build.md; contract indexes them.
Proposal covers independent documentation/extraction enablement, selected publication inputs, per-package planning and package-wide index inputs, identity/order preservation, source availability, validation/error ownership and later consumers.
Explicit Q-decisions expose version/source optionality conflicts and any unowned semantic choices; alternatives and affected consumer gates remain visible until owner approval. Do not implement a preferred alternative.
Forbidden: Rust/Cargo/test/fixture/expectation/trace/spec changes, protocol/AGENTS edits, new producer or proof/reuse/cache authority, deciding Doc6's source, inventing missing labels/metadata or claiming complete Doc4/5/rendering coverage.
Audit impact: no coverage, trace status or follow-up ownership change; no coverage-audit edit.

## Review And Verification Matrix
No executable tests are added; existing published-input/index/native-source tests remain unchanged. Documentation acceptance has six cases (cap six):
1. Request switches and selected package publications: definitions distinguish enabled consumers and retain publication identity without source/acceptance inference.
2. Multiple packages, equal module paths/FQNs and reordered inputs: complete identities and stable ordering remain explicit, with no winner/deduplication policy invented.
3. Absent version/source/summary and empty exports: absences remain distinguishable; conflicting requirements are Q-decisions and Doc6 remains open.
4. Invalid/stale inputs: lower reader refusals are preserved; unsupported binding/metadata is not silently treated as valid semantic input.
5. Index consumers: stored candidates are distinguished from resolved links, comments, anchors, UsedBy, rendering, extraction and publication; unowned semantics stay gated.
6. EN/JA parity, links/fragments and documentation budget: both owner documents express the same definitions and decisions within protocol limits.
Reviews: independent specification review before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency review. Specification review is repeated only if frozen scope changes.
Verification for freeze and final proposal: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (recursive contract pairing, links/fragments and volume included); git diff --check. No fmt/Clippy/runtime gates for documentation only; full cargo test only on a cadence trigger.
Exit: reviewed paired proposal and Q-decision handoff committed locally; Doc5 implementation waits for owner approval, and Doc4 remains pending that approval. No push.
