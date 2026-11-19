# Task DOC13-Q12-MISSING-PACKAGED-SOURCE-DEFINITION: missing-packaged-source warning
Canonical language: English; [Japanese companion](../ja/DOC13-Q12-MISSING-PACKAGED-SOURCE-DEFINITION.md).

## Status And Ownership
Frozen bounded definition; documentation only, full tier for a distinct diagnostic meaning, not a repeated producer pattern. Freeze and definition form one changed task; push:no.
[Diagnostics](../../mizar-diagnostics/en/00.crate_plan.md) owns the warning definition; [Doc Render](../../mizar-doc/en/00.crate_plan.md) consumes it. Artifact/Session retain source/publication authentication; no producer is supplied here.

## Authorities And Gaps
Authorities: [architecture13 warning intent](../../architecture/en/13.documentation_and_extraction.md#documentation-is-not-a-publication-gate-for-proof-validity), [spec22 locations](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display), [registry allocation](../../mizar-diagnostics/en/registry.md#allocation-rules), [link scope/precedence](../../mizar-diagnostics/en/documentation_links.md#locations-precedence-deduplication-and-ordering), [Q3 source slots](../../mizar-doc/en/render.md#q3-source-view-and-line-routing-definition), [selected plans](../../mizar-doc/en/doc_build.md#package-planning), [Gate 4](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates), [sequential plan](../../todo.md#sequential-execution-plan).
Design gap: missing packaged source has warning intent but no descriptor or descriptor-specific scope applicability. Source gap: unavailable original projections/maps and optional caller bundles do not establish non-packaging; authenticated trigger evidence and later adoption/emission witnesses remain required. No audit/trace/coverage impact.

## Frozen Scope
Define a nonallocated descriptor proposal, trigger evidence, authenticated module-scope location applicability, identity/granularity, overlap/precedence and existing profiles in paired Diagnostics registry.md; Gate 4 links that single owner.
Require positive source-owner evidence that the affected module's source is not packaged in the exact selected publication/current request. Missing evidence, legacy/null projections, absent caller bundles and map gaps alone never warn. Preserve source-slot fallback and lower integrity/binding/currentness refusals.
One warning subject per affected authenticated publication/full module identity, independent of item/block/route counts; reuse shared details, scope location, deduplication/order/freshness without new API/schema. Keep W0412–W0415 meanings and Doc8 block-local recovery unchanged.
Write set: this pair; paired Diagnostics registry.md; paired Doc render_prerequisites.md; paired Diagnostics and Doc crate-plan navigation only.
Forbidden: production/test/fixture/spec/AGENTS/protocol edits, allocation/adoption/emission, producer/API/type/schema changes, source discovery, general readiness triggers/build outcomes or package-level diagnostic provenance, complete rendering/search/interface adoption, G4/G5 closure, schemas/publication, Doc23/24 or credit. New language intent or genuinely new semantics stop for Muse; naming, descriptor shape, provenance, presentation and internal placement within existing owners are autonomous.

## Acceptance Matrix And Reviews
Six documentation witness families, cap six; additions require an authority citation:
1. Authenticated selected module/publication/current-request evidence establishes source not packaged and unavailable source links; proposal distinguishes this from unknown availability, legacy/null projections, absent caller bundles and map-only gaps.
2. Candidate code/name/meaning/summary/family/default/phase/category/lifecycle/metadata are explicit but unallocated; existing descriptors and unused slots stay unchanged.
3. Authenticated full module/publication scope has no fabricated block/span/EOF/source lookup; foreign/stale/conflicting or incomplete provenance cannot create a warning.
4. Multiple affected item links/blocks/routes collapse to one module subject; different publications/full module identities remain distinct under existing deduplication/order/freshness.
5. Lower refusals suppress dependent drafts; W0412–W0415 and Doc8 retain independent meanings/recovery without remapping source absence to link incompleteness or failed conversion.
6. Existing warning suppression/promotion remains documentation-only; source presentation and proof/extraction acceptance stay unchanged. Adoption/emission, general readiness, interface review and conditional package-overview producer readiness remain separate.
Independent specification/EN-JA review before freeze; then documentation sufficiency, precision, volume/scope and final consistency over the whole unit. Existing source-slot, shared diagnostic and profile tests are preservation references, not missing-source emission witnesses; no executable test matrix is added.
Verification at freeze and final tree: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy and cargo test -p mizar-test --test lint_policy, local links/fragments and git diff --check. Documentation-only fmt/Clippy/E2E do not apply; full cargo test only on a named cadence trigger.
Exit: commit freeze first, then reviewed definition locally; no remaining readiness/credit gate is discharged.
