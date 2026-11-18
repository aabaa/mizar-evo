# Task DOC10-SHARED-DIAGNOSTICS: shared link diagnostics
Canonical language: English; [Japanese pointer](../ja/DOC10-SHARED-DIAGNOSTICS.md).
Status: frozen implementation scope. Tier: full; freeze and implementation are one changed task.
Purpose: supply only gate 4 of the adopted [Doc10 prerequisites](../../mizar-doc/en/link_prerequisites.md#ordered-readiness-gates-and-decisions).
Owners: [Diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md) for shared support; [Doc plan](../../mizar-doc/en/00.crate_plan.md) for the later consumer.

## Authority And Scope
Authority: [spec24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec22 locations](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display) and existing warning controls.
Adopt exactly the selected [shared diagnostic design](../../mizar-diagnostics/en/documentation_links.md): meanings, summaries, structured identity/details, locations, precedence, deduplication, ordering and profile boundaries. Gates 1–3 remain supplied independently.
Classified source_gap: link descriptors remain unallocated; block provenance exists but module-scope provenance does not. Existing String/List details supply transport; the existing Doc profile can fail documentation quality without changing diagnostic severity.
Allocate fresh W0412 unresolved, W0413 ambiguous and W0414 incomplete_input, with meaning_key = semantic_name, active lifecycle and empty aliases/retirement/replacements. Use since `DOC10-SHARED-DIAGNOSTICS` and the canonical spec24.2 URL; these metadata choices preserve language intent.
Record this exact adoption in paired spec22 diagnostic/location documentation preserving spec24.2 intent. Keep detailed producer conventions solely in the shared diagnostic owner.
Extend DiagnosticPrimaryLocation inline with module-scope Documentation provenance retaining authenticated publication and full module identity, with no invented block or source coordinates. Validate required identity and Documentation phase/category; render scope and verified item identity from the adopted subject details.
Group scope and block primaries under the same Documentation source key; order scope before every block including block zero, preserving numeric u64 block order. Dedup identity distinguishes scope and block kinds; retain existing freshness, mapped tie, detail and presentation comparators.
Use existing details/aggregation APIs; do not add a link resolver, detail schema validator, new wire format, semantic lookup adapter or diagnostic factory. Tests construct explicit drafts without lookup claims.
Forbidden: language intent changes, ② alternatives, link lookup/UsedBy/resolver integration, Doc13 rendering/source routes, Doc23 publication, serialized schema, owner-gated scope, coverage/owner/milestone credit. Gate 5 and every later gate remain open.

## Artifacts And Review
Write set: this contract pair; paired owner-plan links; paired spec22; shared diagnostic proposal and paired registry/failure-record/aggregator/render boundary documents; Doc prerequisite boundary document; necessary mizar-diagnostics source and existing tests; existing crates/mizar-test/src/runner/tests/doc.rs for the Doc profile witness. No AGENTS/protocol/spec24/audit/trace edits.
Reviews: independent specification before freeze; test sufficiency, implementation precision, volume/scope and final source/documentation consistency with scoped quality assessment over the whole unit. Repeat specification only if the contract changes.

## Acceptance Matrix
Six families, cap six; an extra family requires a cited authority:
1. Registry: exact three descriptors, codes, meanings, summaries, family/default severity, lifecycle, since and canonical URL; old allocations and unallocated remainder preserved.
2. Locations: valid block and module-scope provenance, exact optional identities, no invented block/source; reject blank required fields, wrong family and phase/category; draft/record round trips and deterministic debug/CLI rendering retain identity.
3. Reference drafts: adopted details retain full subject, raw/interpreted text, block/logical offsets/form, section version, candidates, causes and routes; duplicate drafts collapse while distinct coincident, discontiguous and unmapped occurrences survive without bridged spans. Complete miss/ambiguity and incomplete known candidates remain distinct explicit outcomes.
4. Scope drafts: module-page, section-index and UsedBy subjects carry all distinct causes and independent closure/citation unavailable/incomplete/complete-empty states, authenticated inventories and examined parts; duplicate reports collapse and no citing occurrence or block is invented.
5. Aggregation: worker/order permutations, duplicate routes versus distinct import positions, same-module scope-before-block-zero and maximum-u64 block ordering, full package/version/publication distinctions and stale snapshots preserve the adopted identity and existing comparator.
6. Presentation/profile: short draft-to-aggregation-to-CLI witness includes scope value/verified item with no source lookup; default Warning/code identity survive the existing Doc fail_on_doc_warnings quality policy without proof/publication/extraction effects. No effective-severity/suppression API; warning controls, emission precedence, success/no-warning and lower-refusal suppression remain gate-5 consumer obligations.
Verification: CARGO_BUILD_JOBS=4; freeze runs both lint_policy suites and local links/fragments. Implementation runs fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites, Diagnostics owner tests and Doc owner/corpus regression checks, including the short diagnostic handoff witness. Existing lower-reader refusal checks run with Doc owner/corpus. Full cargo test only on a named cadence trigger; unchanged heavy proof/cache/evidence obligations defer to cadence/integration.
Exit: reviewed shared diagnostic support committed locally after freeze, push:no; only readiness gate 4 is discharged.
