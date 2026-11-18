# Task DOC13-G4-URL-DETECTION-EMISSION: destination detection and safe output
Canonical language: English; [Japanese companion](../ja/DOC13-G4-URL-DETECTION-EMISSION.md).

## Status And Ownership
Frozen bounded prerequisite; full tier: the registry-only predecessor established neither the producer/consumer pattern nor detection activation. Freeze and implementation are one changed task; push:no.
[Doc](../../mizar-doc/en/00.crate_plan.md) owns recognition/handoff and isolated Render output; [Diagnostics](../../mizar-diagnostics/en/00.crate_plan.md) owns W0415 details, locations, order and profiles.

## Authorities And Gaps
Authorities: [spec24.1/2](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax), [spec22.6.11](../../../spec/en/22.error_handling_and_diagnostics.md#22611-documentation-url-output-warning), [Doc8](../../mizar-doc/en/commonmark.md#candidate-extension-interactions), [Doc10 handoff](../../mizar-doc/en/link_prerequisites.md#resolver-handoff), [output predicate](../../mizar-doc/en/render.md#adopted-html-and-url-output-policy), [shared diagnostic](../../mizar-diagnostics/en/documentation_urls.md).
Source gaps: genuine destination metadata and its classified Render handoff, predicate evaluation and emission are absent. Activation-status wording is derived drift; predicate, classification and warning intent remain unchanged. No audit/trace/coverage or owner credit changes.

## Frozen Scope And Internal Design
Derive destination raw ranges only from genuine pinned CommonMark link/image/autolink events, including selected reference definitions; retain event identity, exact interpreted values, source-order block-local ordinals and original attachments. No inferred links or second decoding.
Expose the bounded destination view on MarkdownBlock, retaining the original events. The isolated Render entry accepts one authoritative Doc10 classification per genuine destination, grouped by original block; exact shape binds ordinals even for repeated references sharing a definition range. It builds existing classified resolver handoffs from those genuine views and authenticates/resolves before producing dependent fragments or drafts. Missing/mismatched handoffs refuse; failed blocks supply no destinations; available-empty stays distinct from unavailable.
Only Doc10 External denotes an ordinary URL. Ordinary is intentionally unlinked; Label/Fqn/Section/Unavailable retain Doc10 outcomes and warnings without URL reclassification or W0415. Lower binding/integrity/currentness refusal returns no dependent output.
Render evaluates the adopted predicate on exact interpreted values, emits isolated escaped accepted link/image fragments or visible label/alternative fallback, and escapes genuine raw HTML without warning. Labels retain visible text/code/HTML/break content. No remote fetch or page renderer. Return fragments alongside existing resolved links/warnings so existing documentation profile and sink/aggregation/CLI paths apply.
Emit one W0415 per rejected destination using exact shared details, full target/publication, true ordinal, destination logical range and independent mapped spans; unmapped input uses true block provenance. Failed views keep their existing warning and suppress only that block.
Forbidden: new diagnostic allocations/schema, URL normalization/repair, classification grammar, proof/extraction/publication changes, complete rendering, remaining G4/G5, coherent Doc13, Doc14/15, Doc23/24, source/readiness warnings, symbolic math and credit.

## Artifacts And Reviews
Write set: this pair, paired Render/prerequisite/Doc and Diagnostics plans/URL owner status, paired spec22 activation status only; existing markdown/render/link implementation and focused Doc/Diagnostics corpus tests. No AGENTS/protocol edits or protected test/spec-intent changes.
Independent specification review before freeze; then sufficiency, precision, volume/scope and final source/EN-JA consistency review. New public handoff/internal representation decisions remain subordinate to these authorities.

## Acceptance Matrix
Six families, cap six; additions require an authority citation:
1. Exact accepted/rejected predicate including delimiter precedence, ASCII spaces/case, controls/DEL/backslashes, entities versus percent spelling; raw HTML is visibly escaped without warning.
2. Genuine inline/reference links, images/autolinks including email, empty destinations and protected code/math; exact raw/interpreted values, true ordinals and full subjects survive.
3. Contiguous/discontiguous/partially mapped and unmapped provenance preserves independent segments, true primary/block locations and no fabricated spans.
4. Semantic, Ordinary and Unavailable classifications retain authoritative resolution; no reclassification or duplicate W0415, including repeated references sharing a destination range.
5. Failed-view/lower-refusal suppression, independent neighboring blocks and unavailable versus available-empty; malformed handoff shape refuses before output.
6. Safe isolated output contains no rejected href/src; genuine detector-to-sink/aggregation/CLI/profile flow preserves deduplication, deterministic order, freshness and unchanged proof/extraction/publication inputs.
Verification: CARGO_BUILD_JOBS=4. Freeze both lint_policy suites including links/fragments. Implementation fmt, all-target/all-feature Clippy -D warnings, both policies, Doc CommonMark owner tests, Doc corpus including resolver/render families, Diagnostics owner tests and short genuine publication-to-CLI/profile E2E. Full cargo test only on a named cadence trigger; unaffected proof/cache/recovery matrices defer to periodic/integration checks.
Exit: reviewed bounded activation committed after freeze; all excluded gates remain open.
