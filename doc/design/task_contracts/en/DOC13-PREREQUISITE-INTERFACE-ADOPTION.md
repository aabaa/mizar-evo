# Task DOC13-PREREQUISITE-INTERFACE-ADOPTION: bounded interface adoption
Canonical language: English; [Japanese companion](../ja/DOC13-PREREQUISITE-INTERFACE-ADOPTION.md).

## Status And Ownership
Frozen documentation-only adoption, full tier for newly adopted interface definitions, not a repeated producer slice. Freeze and adoption form one changed task; push:no.
[Doc Render](../../mizar-doc/en/00.crate_plan.md) owns these internal contracts; Artifact, Driver, Doc8/9/10 and Diagnostics retain their producer, authenticity and diagnostic responsibilities.

## Authorities And Gaps
Authorities: [spec24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec24.4](../../../spec/en/24.documentation_generation.md#244-html-output), [architecture13](../../architecture/en/13.documentation_and_extraction.md), [selected publications](../../mizar-doc/en/doc_build.md#index-inputs-and-later-completion), [identity/allocation](../../mizar-doc/en/render_prerequisites.md#identity-and-allocation-handoffs), [search encoding](../../mizar-doc/en/render_prerequisites.md#proposed-search-encoding-and-compatibility), [auxiliary serialization](../../mizar-doc/en/render_prerequisites.md#proposed-cross-reference-and-usedby-serialization), [readiness gates](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates), [sequential plan](../../todo.md#sequential-execution-plan).
Design gap: existing M/I/D identity/destination and search/auxiliary wire definitions remain unadopted. Source gaps: complete search composition and authenticated original reference encounter ordinals remain missing; prose adoption supplies neither. No audit/trace/coverage change.

## Frozen Scope
Adopt the existing identity/allocation and search/cross-reference/UsedBy definitions as bounded internal prerequisite interfaces, preserving all fields, encodings, orders, authenticity, compatibility and refusal/state distinctions. Update EN/JA status and gates; use minimal navigation to this contract and the single live definition owner.
Preserve full identities, every export and occurrence, complete comment content, Doc8 recovery, Doc9 outcomes, G1/G2/G3 evidence and lower-reader refusals. Unavailable, incomplete, known absence and available-empty never collapse; no inferred ordinals or fabricated producer evidence.
Write set: this pair; paired mizar-doc render_prerequisites.md, render.md, todo.md and 00.crate_plan.md. Existing heading anchors may remain for navigation stability; adopted status overrides historical proposal wording only within this scope.
Forbidden: producer/code/API/schema/spec/test changes, template adoption, Q8 complete-build outcomes, Q12 policy/allocation/emission, package-overview adoption/production, complete rendering/search, G4/G5 closure, schemas/publication, Doc23/24 and completion credit. Preserve the conditional package-overview producer-readiness review. New language intent or genuinely new semantics stop for Muse; naming, presentation, provenance and internal placement within existing definitions are autonomous.

## Acceptance Matrix And Reviews
Six documentation witness families, cap six; additions require an authority citation:
1. M/I/D, owner spellings, null meanings, kind/version binding and destination validation are adopted without changing values or Artifact storage.
2. Search retains exact envelope/record shapes, all selected exports, summary/signature distinctions and canonical bytes/version compatibility; complete production stays missing.
3. Auxiliary records retain reference/citation/coverage/UsedBy shapes, order, repeated occurrences, authenticated provenance and independent completeness; original encounter ordinals stay a producer gate.
4. Available-empty, known absence, unavailable/incomplete and lower refusals retain their meanings; Doc8 block recovery never becomes a Q8 build refusal.
5. EN/JA status, owner boundaries and navigation agree; Doc13 remains open and templates, Q8/Q12, package overview and later gates gain no adoption or credit.
6. Authentic producer requirements remain prerequisites under the sequential plan; conditional package-overview readiness remains separate.
Independent specification/EN-JA review before freeze; then documentation sufficiency, precision, volume/scope and final consistency over the whole unit. Existing identity/destination, summary/search-reader and reference/direct-by tests are preservation references, not complete wire-producer witnesses; no executable matrix is added.
Verification at freeze and final tree: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy and cargo test -p mizar-test --test lint_policy; local links/fragments and git diff --check. Documentation-only fmt/Clippy/E2E do not apply; full cargo test only on a named cadence trigger.
Exit: commit freeze first, then reviewed bounded adoption locally; complete producer/readiness gates remain open.
