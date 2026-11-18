# Task DOC8-COHERENT-ADOPTION: CommonMark/tag specification and diagnostics
Canonical language: English; [Japanese pointer](../ja/DOC8-COHERENT-ADOPTION.md).
Status: frozen adoption unit. Tier: full; freeze and adoption form one changed task.
Primary owner: [diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md); consumer and parser owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md).
Purpose: adopt approved Doc8 Q9–Q18 and MT-Q1–MT-Q7 candidate ① so subsequent Doc8 implementation is startable.

## Authority And Readiness
Owner approvals: Q9–Q18 all ① (2026-11-18 17:50 JST) and MT-Q1–MT-Q7 all ① (2026-11-18, dispatched ledger approval). Authorities are the [CommonMark proposal](../../mizar-doc/en/commonmark.md), [diagnostic proposal](../../mizar-diagnostics/en/documentation_markdown_tags.md), spec [24.1–3](../../../spec/en/24.documentation_generation.md), spec22 [locations](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display) and [attachment allocation](../../../spec/en/22.error_handling_and_diagnostics.md#2268-documentation-attachment-warnings).
Missing normative definitions/descriptors/detail encoding are approved spec_gap/design_drift repairs; missing parser implementation is deferred source_gap. No new semantic choice is authorized. Doc7 supplies retained source-ordered blocks and full target provenance.

## Scope And Ownership
Adopt paired spec24 recognition, raw fields/duplicates, pinned CommonMark 0.31.2/pulldown-cmark 0.13.6 options, bounded math and warning recovery, including the consecutive-tag example's single field. Keep raw text/maps and lower-reader refusals.
Add spec22 §22.6.9 with seven fresh descriptors W0405–W0411 in proposal order, unversioned semantic names as meaning keys, revision `spec-22-documentation-markdown-tags-v1`, and canonical section URL. W0400–W0404 stay attachment-only.
Explicitly extend §22.1.2 provenance-location applicability to these warnings. Adopt marker/slot/delimiter-first mapped primary, ordered other segments, valid raw-block conversion location and provenance fallback; no invented bridge or source binding.
Use existing DiagnosticDetails strings/lists for full publication/module/optional target identity, lossless decimal block ordinal and logical half-open offsets, reason and raw/subreason fields. Failed blocks retain independently established condition identities/reasons as nested details; no shared schema/API change. Record owner defines exact encoding before tests/registry adoption; aggregation keeps its comparator.
Write set: this contract pair; paired spec22/spec24; paired diagnostics registry/record/aggregation/proposal and Doc comments/CommonMark owners; necessary paired crate-plan/TODO links; registry builtin and comments; existing diagnostics registry/record/aggregation/render tests and mizar-test doc tests. Update existing allocation inventories only where adoption makes them stale.
Forbidden: AGENTS/protocol edits, candidate ②, parser/Cargo dependency implementation, new tags/metadata inference, Doc9/Doc11 choices, HTML/LaTeX/link policy, new primary-location types, warning flags, proof/extraction/publication policy, fixtures/trace/audit credit or milestone closure. Audit ownership/coverage stays unchanged.

## Acceptance Matrix
Six families (cap six; extra cases require an authority citation), adoption seam tests rather than a pretend parser:
1. Seven descriptors exactly match spec22 names/keys/summaries/lifecycle/since/URL; W0400–W0404 unchanged, W0412–W0499 unallocated, existing compatibility checks lock all Documentation identities.
2. Each new code accepts mapped Span and provenance-only locations, with ordered discontiguous secondaries and complete details surviving draft/record/debug; existing metadata/location rejection tests include new codes.
3. At coincident/provenance locations, changes in target, block, reason or logical offsets remain distinct; equal occurrences deduplicate despite message changes; shuffled drafts/batches preserve shared canonical order, numeric provenance block order and freshness.
4. Conversion warning alone represents a failed block with nested retained condition identities/reasons; unaffected-block warnings survive; shared records preserve this payload without implementing emission.
5. Draft-to-index-to-CLI displays a new provenance warning without source lookup/fabricated coordinates; the existing documentation profile promotes supplied new-code warnings only for documentation, preserving Warning descriptors and original inputs.
6. Paired normative/owner definitions cover every approved Q decision, protected math boundaries, raw retention and refusals; proposal history cannot re-gate adoption. EN/JA links, fragments and budgets pass; parser conformance remains subsequent Doc8 acceptance.
Reviews: specification before freeze; independent sufficiency, precision, volume/scope and final source/document consistency with task-scoped scoring.
Verification: CARGO_BUILD_JOBS=4; freeze runs both explicit lint_policy suites and git diff --check. Adoption runs cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; both lint_policy suites; cargo test -p mizar-diagnostics; affected mizar-test doc group and explicit new-code CLI/profile checks; git diff --check. Full cargo test only on a named cadence trigger; unchanged heavy resolver/evidence/cache obligations remain deferred.
Exit: freeze and reviewed coherent adoption committed locally, without push; subsequent Doc8 parsing may start, without implementation/completion credit here.
