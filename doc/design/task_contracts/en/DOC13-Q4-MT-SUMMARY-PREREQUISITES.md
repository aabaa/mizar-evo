# Task DOC13-Q4-MT-SUMMARY-PREREQUISITES: shared summary projection
Canonical language: English; [Japanese companion](../ja/DOC13-Q4-MT-SUMMARY-PREREQUISITES.md).

## Status And Ownership
Frozen bounded implementation, full tier: the shared selector and module/search consumers introduce an executable pattern distinct from Q3. Freeze and implementation count as one changed task; push:no.
[Doc Render](../../mizar-doc/en/00.crate_plan.md) owns the projection; Doc8 and Diagnostics retain parsing, raw fallback and warning authority. Complete rendering and search serialization remain separate.

## Authorities And Gaps
Authorities: [spec24.1 recognition](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax), [spec24.4 module/search content](../../../spec/en/24.documentation_generation.md#244-html-output), [spec22 MT recovery](../../../spec/en/22.error_handling_and_diagnostics.md#2269-documentation-markdown-and-tag-warnings), [joint Q4/MT definition](../../mizar-doc/en/render.md#joint-q4mt-summary-definition), [Doc8 recovery](../../mizar-doc/en/commonmark.md#degradation-and-adoption-boundary), [MT precedence](../../mizar-diagnostics/en/documentation_markdown_tags.md#locations-emission-and-profiles), [readiness item 3](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates).
Source/test gap: no executable shared projection or module/search preservation witness. Existing mizar-doc markdown views and mizar-test runner/tests/doc8_math.rs recovery witnesses are the lower-stage inputs. No language intent, coverage/audit/trace or credit change.

## Frozen Scope And Decisions
Consume complete source-ordered attachments for the same full-provenance target/publication. Select the first eligible successful-view paragraph, including structural container paragraphs; use existing Doc8 tag recognition and exclude recognized tags and non-paragraph constructs. Select before flattening, including an empty result.
Skip the whole failed view without reparsing raw fallback. Preserve raw blocks, identities, maps, provenance, warnings, promotion and lower-reader refusals. All-failed or no eligible paragraph yields an available empty string, never absence/null.
Flatten text/code, link/image text, unchanged opaque math, literal HTML and breaks exactly as Q4 defines; collapse ASCII whitespace and trim, with no Unicode normalization or truncation. Escape only at final sinks.
Known absence is null for search and empty visible module text; unavailable required metadata remains unavailable. Module/search consumers share exact text and preserve all exports in existing full-identity order, including undocumented and reexported items.
API names, immutable handoff structure and internal paragraph reuse are autonomous design within existing authorities. No new warning, status prefix or serialized field.
Forbidden: spec/AGENTS/protocol edits, changed language intent, new parsing/tag recognition or recovery semantics, complete pages/index serialization, Q12 completion, publication, Doc13/G4 closure, G5/Doc14–15, schemas, Doc23/24 or credit. New semantic choices stop for Muse.

## Artifacts And Reviews
Write set: this pair, paired Doc plan navigation and render/render_prerequisites boundary updates; existing mizar-doc markdown.rs/render.rs, mizar-test Doc tests and mizar-driver/tests/request.rs native publication witness. Narrow existing owner handoffs only when required by the definition; no new source authority.
Independent specification review before freeze; test sufficiency, implementation precision, volume/scope and final source/EN-JA consistency over the whole unit. Specification review repeats only if the contract changes.

## Acceptance Matrix
Six families, cap six; extra families require a cited authority clause:
1. Separate attached blocks, ordinary/tag paragraphs, tight/loose lists and quotes select the first eligible paragraph in source order; known missing/duplicate tags are excluded while unknown/lookalike tags remain text. Headings, code/HTML blocks, definitions and display math are not candidates.
2. Known absence, available empty/tag-only/non-paragraph-only, unavailable metadata and selected-empty followed by nonempty remain distinct; selected-empty stops selection.
3. Genuine conversion/map failure preserves complete raw blocks, original identities/maps/provenance, W0411 and retained conditions/MT precedence; successful neighbors remain usable.
4. Failed-before/after-valid, multiple failed and all-failed attachments yield deterministic selection; no failed-view text is invented.
5. Exact flattening covers text/code, links/images, opaque math, literal HTML, soft/hard breaks, ASCII whitespace, Unicode and long text, without normalization/truncation or double escaping.
6. Module/search exact text agrees for every export with full identity/order preserved; absence/reexports/undocumented items, warning controls and lower-reader refusals retain existing meaning, using genuine publication inputs.
Verification: CARGO_BUILD_JOBS=4. Freeze: both lint_policy suites (ledger/local links/fragments), git diff --check. Implementation: fmt --check; all-target/all-feature Clippy -D warnings; both policies; Doc owner tests and published_inputs corpus; short genuine native Driver publication E2E through the projection. Full cargo test only on a named cadence trigger; unaffected heavy proof/evidence/cache/recovery obligations defer to periodic/integration cadence.
Exit: commit freeze first, then reviewed implementation locally; excluded gates stay open.
