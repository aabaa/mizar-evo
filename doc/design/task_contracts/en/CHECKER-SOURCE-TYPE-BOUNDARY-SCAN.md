# Task CHECKER-SOURCE-TYPE-BOUNDARY-SCAN: complete architecture scan
Canonical language: English; [Japanese pointer](../ja/CHECKER-SOURCE-TYPE-BOUNDARY-SCAN.md).
Status: complete. Tier: full. Owner: [checker](../../mizar-checker/en/00.crate_plan.md).
Purpose: repair a test_gap in the existing Type architecture assertion; no consumer/API change.
Authority: [Type boundary](../../mizar-checker/en/source_type.md#boundary-and-model), its existing architecture test and AGENTS.md test-infrastructure authority.
Dependency: existing SourceType implementation; no unresolved semantic decision.

Change only the delimiter in `production_boundary_stays_syntax_free_and_has_no_semantic_result_payloads`
to the actual test-module marker, with formatting and the mechanical source inventory count.
Retain its source input, forbidden list, assertion, message and name. This restores existing
architecture intent, not a language oracle or prohibited-dependency policy change.
Before editing, independently review this exact correction and its intent/authority.
Use a temporary test-input-only forbidden sentinel before the module to demonstrate the
old missed region and corrected refusal; remove the injection before real-source verification.
Preserve all runtime bytes, semantic tests/oracles, public surfaces, trace and six exceptions.
No helper, allowlist, module split, production import or new architecture policy.
Existing owner prose and coverage audit remain unchanged; link only from checker plan/todo.
Exit: independent TEST/CODE/VOLUME/consistency and scoped gates; exact test, checker library/
inventory lint, fmt/Clippy and documentation/link checks, plus inherited full cadence.
