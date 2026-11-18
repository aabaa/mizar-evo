# Task DOC-DIAGNOSTICS-ADOPTION: Documentation diagnostics adoption
Canonical language: English; [Japanese pointer](../ja/DOC-DIAGNOSTICS-ADOPTION.md).
Status: frozen adoption unit. Tier: full.
Primary owner: [diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md); consumer: [Doc plan](../../mizar-doc/en/00.crate_plan.md), Doc7 and dependent Doc8.
Purpose: adopt owner-approved DD-Q1–DD-Q7 option ①; define the non-range location and record owner first, then implement/test it before adopting the registry descriptors. Freeze and adoption are one changed task.

## Authority And Readiness
Authority: owner approval of all seven option-① decisions on 2026-11-18; [proposal](../../mizar-diagnostics/en/documentation_definition.md), [attachment outcomes](../../mizar-doc/en/comments.md#residual-association-definitions), spec22 [codes](../../../spec/en/22.error_handling_and_diagnostics.md#2211-error-code-system-e-series) and [locations](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display).
The span-only non-SourceLoad restriction is an approved `spec_gap` repair. Registry [allocation/compatibility](../../mizar-diagnostics/en/registry.md#allocation-rules), record [construction](../../mizar-diagnostics/en/failure_record.md#construction-and-round-trips), [aggregation](../../mizar-diagnostics/en/aggregator.md#deduplication-identity) and [rendering](../../mizar-diagnostics/en/render.md#source-loading-locations) retain their ownership.
No new semantic choice is authorized. Any language intent beyond DD-Q1–DD-Q7 stops this unit. Doc7 implementation and broader coverage remain separate; Doc8 depends on Doc7.

## Scope And Ownership
Freeze paired spec22 Documentation non-range location and paired failure_record owner definition with this contract. The location preserves publication hash, the complete five-field module identity and zero-based block ordinal in source order, without a fabricated source binding. No Artifact dependency or new wrapper type.
Then implement shared location validation, exact round trips, aggregation identity/order/freshness and CLI display; add Documentation phase/category provenance. Preserve SourceLoad restrictions and explicit unsupported consumer refusals.
Adopt Documentation with Warning as its single family default, W0400–W0499 and exactly W0400–W0404; spec22 owns mappings/summaries and revision token `spec-22-documentation-v1`, registry derives metadata with the attachment-section target. Meaning keys equal semantic names; unused slots remain unallocated.
Write set: this paired contract; paired spec22; diagnostics paired failure_record, registry, aggregator, render and proposal status; paired diagnostics/Doc plan task-index links; diagnostics Rust registry, records, aggregation/rendering and focused existing test files. Necessary existing test expectation inventories may update to the approved allocation.
Forbidden: AGENTS/protocol edits, option-② alternatives, Doc7/8 attachment/Markdown implementation, missing-comment/tag/Extraction codes, source-reader reinterpretation, proof/acceptance/publication policy changes, LSP wire expansion, fixture/trace/audit coverage credit or milestone closure.

## Acceptance Matrix
Six cases (cap six), implemented in existing diagnostics test files; no new `.miz` behavior:
1. Exact five descriptors, names/keys/summaries/family/severity/lifecycle/since/URL match spec22; family endpoints map correctly, unused W0405–W0499 remain unallocated and other ranges stay unchanged.
2. Existing registry compatibility checks reject mutation/removal/reuse of the new permanent identities; no mixed-severity family redesign.
3. Documentation Span and non-range locations round-trip through draft/record/debug with complete provenance and real secondary spans; publication/module/block distinctions survive.
4. Reject incomplete required provenance, mismatched code/location/phase/category, and non-Documentation use of its location; existing SourceLoad/span validation remains intact.
5. Aggregation deduplicates only equal provenance/code/detail identities, orders deterministically across batch permutations and block ordinals, and preserves stale/current snapshot rules.
6. Draft-to-index-to-CLI check renders a non-range warning without source lookup, fabricated coordinates/underline/excerpt; real related spans remain supported. EN/JA parity, links and unsupported range-only consumer refusal remain intact.
Reviews: independent specification before freeze; sufficiency, precision, volume/scope, final source/document consistency and score. Coverage audit unchanged: this prerequisite adds no Doc7 implementation credit.
Verification: CARGO_BUILD_JOBS=4; freeze runs both explicit lint_policy suites (including links/fragments/pairing/volume) and git diff --check. Adoption runs cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; both lint_policy suites; cargo test -p mizar-diagnostics; explicit representative draft-to-CLI test; full cargo test (requested push/inherited cadence); git diff --check. Heavy resolver/evidence/cache obligations are unchanged and covered by the push full run.
Exit: reviewed freeze, location implementation/tests and normative/registry adoption committed; full tests pass on pushed tree; push requested by dispatch. Doc7 remains a separately dispatched implementation, Doc8 dependent.
