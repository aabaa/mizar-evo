# Task DOC13-COMMENT-SAFE-FLOW-PREREQUISITES: comment and tag fragments
Canonical language: English; [Japanese companion](../ja/DOC13-COMMENT-SAFE-FLOW-PREREQUISITES.md).

## Status And Ownership
Frozen bounded implementation, full tier: a new Doc13 consumer beyond summary/isolated-URL patterns. Freeze and implementation count as one changed task; push:no.
[Doc Render](../../mizar-doc/en/00.crate_plan.md) owns safe fragment composition. Doc8 retains parsing/recovery, Doc9 supplied classification/resolution and Diagnostics warning authority. Complete page production stays gated.

## Authorities And Gaps
Authorities: [spec24.1 content/tags](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax), [spec24.2 references](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec24.4 output](../../../spec/en/24.documentation_generation.md#244-html-output), [MT recovery](../../../spec/en/22.error_handling_and_diagnostics.md#2269-documentation-markdown-and-tag-warnings), [owner slots](../../mizar-doc/en/render.md#owner-slots-and-dom-hooks), [Q5 safety](../../mizar-doc/en/render.md#adopted-html-and-url-output-policy), [opaque math](../../mizar-doc/en/render.md#adopted-opaque-math-transport), [readiness](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates).
Source/test gap: isolated URL/math and summary producers do not compose complete comments, tag bodies or See entries. Existing genuine Doc8/Doc9 publication fixtures are authoritative lower-stage witnesses. No coverage/audit/trace or credit change.

## Frozen Scope And Decisions
Consume genuine complete Doc8 views for an exact attached target/publication and supplied Doc9 occurrence classifications; authenticate attachment/order/view and occurrence bindings before composition. Never discover/classify references or parse fallback blocks again.
Produce safe flow complete-comment and param/returns tag-body fragments, and safe phrasing See entries. Preserve original block/tag order, all ordinary/unknown/malformed/duplicate content and raw-block recovery. Every recognized param/returns and See occurrence has its own derived view; no removal, merging or replacement of complete-comment content.
Preserve CommonMark structure and inline formatting. Tag-body ranges use existing Doc8 fields; parameter labels use exact raw slots, returns uses a presentation label. Missing slots/bodies remain visible empty derived views with unchanged MT warnings. See labels preserve the raw reference slot, with body content outside the reference anchor.
Compose semantic code/explicit/See links using supplied Doc9 outcomes; unresolved/ambiguous/incomplete/excluded references stay safe unlinked content with existing warnings and coverage. External URLs/raw HTML follow adopted Q5 once, without duplicated label content. No new warning meaning or policy.
Overlay exact existing opaque-math fragments at original source occurrences, retaining payload/attributes/status/code. Keep display divs in valid flow positions outside paragraphs/anchors; no symbolic math or numbering.
Known absence remains distinguishable from available-empty; unavailable required metadata and foreign/tampered bindings refuse through existing boundaries. Preserve maps, provenance, warnings, promotion and lower-reader refusals. API naming, immutable result shape and presentation markup are internal design choices.
Forbidden: spec/AGENTS/protocol edits, language-intent changes, UsedBy fragments, supplied-fragment template expansion, discovery, new diagnostics, complete-page/search claims, Doc11/math or Q12 completion, Doc13/G4/G5 closure, Doc14–15, schemas/publication, Doc23/24 or credit. New semantic choices stop for Muse.

## Artifacts And Reviews
Write set: this pair, paired Doc plan navigation and render/render_prerequisites boundary updates; existing mizar-doc render.rs/markdown.rs only as required, mizar-test Doc tests and native mizar-driver/tests/request.rs witness. No new parsing or source authority.
Independent specification review before freeze; test sufficiency, precision, volume/scope and final source/EN-JA consistency over the whole unit. Do not repeat passed specification review unless this contract changes.

## Acceptance Matrix
Six families, cap six; extras require a cited authority clause:
1. Complete multi-block/container/inline content preserves order and all tags, unknown/malformed/duplicate text; genuine failed blocks retain safe raw fallback alongside successful neighbors, without mutation.
2. Param/returns/See views preserve every occurrence in block/tag order, including duplicate/missing slots and empty/multiline/markup bodies; complete comments retain those same occurrences.
3. Supplied code/explicit/See classifications preserve linked/unresolved/ambiguous/incomplete/excluded outcomes, exact hrefs, warning drafts and promotion; absent classifications refuse rather than infer.
4. Accepted/rejected URL and raw HTML composition escapes once, preserves formatting safely and emits adopted W0415 without label duplication or active authored HTML.
5. Inline/display opaque math bytes, payload, ordinals and attributes are exact in complete/tag views; display placement is valid and no display math is injected into anchors.
6. Known absence/available-empty/unavailable metadata, foreign target/publication and tampered block/field/math/occurrence bindings remain distinct; original maps/provenance/warnings and lower-reader refusals are unchanged. Include genuine native Driver publication through the new API.
Verification: CARGO_BUILD_JOBS=4. Freeze: both lint_policy suites (ledger/local links/fragments), git diff --check. Implementation: fmt --check; all-target/all-feature Clippy -D warnings; both policies; Doc owner tests and published_inputs corpus; short native Driver publication E2E. Full cargo test only on a named cadence trigger; unrelated heavy proof/evidence/cache/recovery obligations defer to periodic/integration cadence.
Exit: commit freeze first, then reviewed implementation locally; excluded gates remain open.
