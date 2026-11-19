# Task DOC13-Q3-SOURCE-VIEW-PREREQUISITES: bounded source views
Canonical language: English; [Japanese companion](../ja/DOC13-Q3-SOURCE-VIEW-PREREQUISITES.md).

## Status And Ownership
Frozen prerequisite, full tier: a new executable source-view consumer boundary. Freeze and implementation count as one changed task; push:no.
Primary owner: [Doc Render plan](../../mizar-doc/en/00.crate_plan.md); Session/Artifact retain original-text/map authority. Future page consumers use this isolated handoff.
Dependencies: [Q3 definition](../../mizar-doc/en/render.md#q3-source-view-and-line-routing-definition), [G2 allocation](../../mizar-doc/en/render_prerequisites.md#adopted-g2-destination-allocation), [G3 authentication](../../mizar-doc/en/render_prerequisites.md#adopted-g3-source-coordinates), [Session maps](../../mizar-session/en/source_map.md).

## Authority And Gaps
Authority: [spec24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec24.4](../../../spec/en/24.documentation_generation.md#244-html-output), [source availability](../../architecture/en/13.documentation_and_extraction.md#documentation-is-not-a-publication-gate-for-proof-validity), and the frozen [entry270 definition](./DOC13-Q3-SOURCE-ROUTING-DEFINITION.md).
Source/test gap: G3 retains original projection privately; authenticated original-text access, source allocations and rendering witnesses are absent. Existing coordinate and genuine Driver publication tests are preservation references. No coverage/audit/trace or credit change.

## Frozen Scope And Decisions
Expose read-only original text from the same retained authenticated G3 projection, with exact module/publication/path/digest checks; legacy/null stays unavailable. Preserve currentness authentication and lower-reader refusals, without source reread or reconstruction.
Allocate version-local sources/<packaged-path>.html views in G2's complete auxiliary namespace, bound to full module/publication/path/original digest. Identical bindings reuse; distinct bindings, reserved/file-directory/ASCII-case collisions refuse without suffixes.
Render only the fixed [source shell and source slot](../../mizar-doc/en/render.md#source-view-and-source-slot), using original LF-based one-based L<n> anchors and once-escaped relative Q3 hrefs. Preserve original text/maps/provenance; visible controls use exactly the defined display projection.
Item slots consume the authenticated artifact source_range through G3 loaded_ranges, retaining ordered segments, repeated starts and incomplete state. No covering ranges, invented lines, endpoint links or normalization. Source versus map unavailability remain distinct; retain existing missing-packaged-source warning policy without allocating diagnostics.
Immutable handoff/API names and validation order are internal design. Keep source binding and allocation together for later rendering; do not add another source authority.
Forbidden: spec/AGENTS/protocol edits, changed language intent, semantic reclassification, source fetch, Q4/MT degraded summaries, Q12 definitions/completion, complete page producer, coherent Doc13/G4 closure, G5/Doc14–15, schemas/publication, Doc23/24 or credit. New language semantics stop for Muse.

## Artifacts And Reviews
Write set: this pair, paired Doc plan navigation and render/render_prerequisites boundary updates; existing Doc doc_build.rs, allocations.rs and render.rs; existing mizar-test Doc tests and Driver request tests. Narrow existing owner handoffs only if required by the definition.
Reviews: independent specification before freeze; test sufficiency, implementation precision, volume/scope and final source/EN-JA consistency over the whole unit. Specification review repeats only if the contract changes.

## Acceptance Matrix
Six families, cap six; extra families require a cited authority clause:
1. Genuine current publication exposes exact original text; foreign module/publication/path/digest, stale/tampered plan and contradictory lower inputs refuse without partial output.
2. UTF-8, LF, CRLF, lone CR, empty/trailing-LF text and EOF points produce exact rows/anchors; invalid bounds/UTF-8 boundaries refuse.
3. Ordered discontiguous/repeated/empty and partial segments retain start-line links and mapping state, without bridges or exclusive-end links.
4. Identical bindings reuse views; distinct bindings and complete namespace reserved/file-directory/ASCII-case collisions refuse.
5. Nested/non-ASCII/metacharacter paths and hostile source/package text use exact Q3 encoding and single escaping; fixed shell bytes are deterministic.
6. Available text without maps/segments, legacy/absent source and missing packaged source preserve distinct slots and existing warnings; no new diagnostics or readiness claim.
Verification: CARGO_BUILD_JOBS=4. Freeze: both lint_policy suites (ledger/local links/fragments), git diff --check. Implementation: fmt --check; all-target/all-feature Clippy -D warnings; both policies; Doc owner tests and published_inputs corpus; genuine native Driver publication short E2E. Full cargo test only on a named cadence trigger; unaffected heavy proof/evidence/cache/recovery matrices defer to periodic/integration cadence.
Exit: commit freeze first, then reviewed implementation locally; excluded gates stay open.
