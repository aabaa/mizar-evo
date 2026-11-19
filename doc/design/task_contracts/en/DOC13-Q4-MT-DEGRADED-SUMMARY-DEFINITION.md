# Task DOC13-Q4-MT-DEGRADED-SUMMARY-DEFINITION: joint summary/search projection
Canonical language: English; [Japanese companion](../ja/DOC13-Q4-MT-DEGRADED-SUMMARY-DEFINITION.md).

## Status And Ownership
Frozen bounded definition; documentation only, full tier: this is a new joint summary/search consumer boundary, not a Q3 producer-pattern slice. Freeze and definition are one changed task; push:no.
[Doc Render](../../mizar-doc/en/00.crate_plan.md) owns the shared module/search text projection; Doc8 and Diagnostics retain parsing, raw fallback and warning authority. Q3 source-view prerequisites are landed; complete rendering and search production remain separate.

## Authorities And Gaps
Authorities: [spec24.1 recognition](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax), [spec24.4 module/search content](../../../spec/en/24.documentation_generation.md#244-html-output), [spec22 MT recovery](../../../spec/en/22.error_handling_and_diagnostics.md#2269-documentation-markdown-and-tag-warnings), [Q4 selection/flattening](../../mizar-doc/en/render.md#candidate-destinations-and-page-content), [MT precedence](../../mizar-diagnostics/en/documentation_markdown_tags.md#locations-emission-and-profiles), [readiness item 3](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates), [execution order](../../todo.md#sequential-execution-plan).
Design gap: Q4 has no degraded summary/search projection for adopted block-local conversion/map recovery. Existing Doc8 recovery cases in mizar-test runner/tests/doc8_math.rs and mizar-doc markdown tests are preservation references, not executable summary witnesses. No language intent, coverage/audit/trace or credit change.

## Scope And Artifacts
Define one source-ordered successful-view projection for module summaries and search, retaining raw failed blocks separately without inventing their paragraphs. Distinguish known absence, available-empty, unavailable inputs and recovered conversion/map failure; preserve every export, exact successful Q4 flattening and existing diagnostics/refusals.
Write set: this contract pair, paired render.md and render_prerequisites.md, and paired Doc crate-plan navigation only. Render owns the joint definition; prerequisites own remaining readiness. No new schema, producer API or warning definition.
Forbidden: implementation, production code, tests/executable witnesses, spec/AGENTS/protocol edits, new parsing/tag recognition, source/map inference, warning reassignment, complete templates, remaining Q12, G4 closure, G5/Doc14–15, schemas/publication, Doc23/24 or credit. Language-specification intent changes or genuinely new semantic choices stop for Muse; diagnostics, presentation/rendering, naming, provenance and internal structure within existing authority are autonomous design.

## Acceptance Matrix And Reviews
Six documentation witness families, cap six; extra cases require an authority citation:
1. Separately attached blocks and ordinary/tag paragraphs select in source order using existing recognition, retaining every tag and complete comment.
2. Known absence maps to null search/empty visible text; available comments without a selected paragraph yield an empty string; unavailable required inputs remain unavailable.
3. Actual conversion/map failure retains the affected raw block and existing warning/precedence; unaffected neighbors retain their views and conditions.
4. Failed-before/after-valid, multiple failed and all-failed blocks have deterministic selection with no invented failed-view paragraphs.
5. Successful paragraph flattening preserves text/code, link/image text, opaque math, breaks, raw HTML and ASCII-whitespace rules exactly, without normalization or truncation.
6. Module/search consume identical text, preserve every export and existing warning controls/lower-reader refusals; EN/JA agree and definition alone supplies no readiness witness.
Independent specification review before freeze, then documentation sufficiency, precision, volume/scope and final source/EN-JA consistency reviews over the whole unit.
Verification: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy and cargo test -p mizar-test --test lint_policy (ledger and local links/fragments) at freeze and final tree; git diff --check. No fmt/Clippy/E2E for documentation only; full cargo test only on a named cadence trigger.
Exit: commit freeze first, then the reviewed paired definition locally; implementation and remaining gates stay separate.
