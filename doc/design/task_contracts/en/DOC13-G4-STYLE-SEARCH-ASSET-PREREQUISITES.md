# Task DOC13-G4-STYLE-SEARCH-ASSET-PREREQUISITES: local search consumer
Canonical language: English; [Japanese companion](../ja/DOC13-G4-STYLE-SEARCH-ASSET-PREREQUISITES.md).

## Status And Ownership
Frozen bounded prerequisite, full tier: first executable search compatibility/DOM consumer. Freeze plus implementation is one changed task.
Primary owner: [mizar-doc](../../mizar-doc/en/00.crate_plan.md); Render owns local assets and browser presentation. Diagnostics retains build diagnostics/readiness; summary/math/source owners retain their projections.
Purpose: adopt only the supplied-index reader and local style/search asset boundary within [G4](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates).
Dependencies: [complete-page DOM hooks](../../mizar-doc/en/render.md#owner-slots-and-dom-hooks), adopted Q3 hrefs/G1 vocabulary and existing pinned browser harness.

## Authority And Gaps
Authority: [spec24.4](../../../spec/en/24.documentation_generation.md#244-html-output), architecture13 [Step4](../../architecture/en/13.documentation_and_extraction.md#step-4-emit-static-site) and [determinism](../../architecture/en/13.documentation_and_extraction.md#outputs-are-deterministic), internal05 [index](../../internal/en/05.documentation_extraction.md#documentation-index).
Design/source/test gaps: supplied search encoding and Q9 matching are proposals; fixed DOM hooks exist but style/search bytes, pinned manifests and executable witnesses are absent.
Existing references: `crates/mizar-doc/tests/opaque_math.rs`, `tests/doc_math_browser.py`; no existing search producer or complete renderer is presumed.

## Frozen Scope
Adopt only the [reader contract](../../mizar-doc/en/render_prerequisites.md#proposed-search-encoding-and-compatibility) and Q9 query/DOM behavior for supplied complete version-1 indexes; preserve exact display strings, known-absent null versus empty summary, complete-empty items and supplied order.
Require exact envelope/record fields and tuple shapes, owner kind vocabulary, lowercase hexadecimal publication hash, strict UTF-8/JSON and rejection of duplicate/unknown keys, wrong types and unsupported versions before consuming any records. No completeness/authentication claim is inferred from supplied bytes.
Accept only Q3-encoded local relative dedicated-page hrefs with full FQN fragments; forbid root/foreign URLs, traversal and malformed encoding. Rebase from the index directory without double encoding or rewriting stored escapes/fragments.
Bind each search control independently through the fixed hooks, use text nodes/validated href attributes, accessible status and result rows. Load/parse/compatibility failure is search unavailable, never zero matches; neighboring controls continue.
Provide fixed local CSS/search JS, immutable manifest and exact-set asset handoff/validation using existing Render ownership; bytes contain no remote dependencies or volatile metadata. Internal API, CSS, parser structure and status wording are autonomous presentation choices.
Unavailable/degraded summary inputs remain producer gates; no null synthesis, producer sorting/export enumeration, index generation, complete-page rendering or template completion witness.
Forbidden: language/spec/test-intent changes, diagnostic/readiness definitions, summary/math/source projections, remaining G4/G5, coherent Doc13, Doc14/15, schemas/publication/Doc23/24, credit/audit/trace changes, AGENTS/protocol edits.

## Artifacts And Reviews
Write set: this pair and paired Render/prerequisite boundaries plus crate-plan links; `crates/mizar-doc/src/render.rs`, local style/search assets/manifest, Doc asset tests and `tests/doc_search_browser.py`. Existing math assets/semantics stay unchanged.
Reviews: independent specification before freeze; test sufficiency, precision, volume/scope, final source/EN-JA consistency and scoped quality assessment.

## Acceptance Matrix
Six witness families, cap six; extra families require cited authority:
1. Exact bundled paths/bytes/digests, repeated byte equality and refusal of missing/extra/duplicate/changed assets or manifest.
2. Fixed scoped DOM hooks, accessible label/status, exact result-node shape and local CSS presentation, with no complete renderer claim.
3. Compatible populated and complete-empty indexes succeed; malformed UTF-8/JSON, duplicate/unknown keys, wrong shapes/types/version/kind/hash/hrefs refuse atomically, including bad later records.
4. ASCII-whitespace tokenization, ASCII-only folding, AND matching across all four fields, empty query, non-ASCII preservation, duplicate FQNs and stable supplied result order.
5. Hostile display text remains inert; nested-page href rebasing preserves percent escapes and full fragment with local targets.
6. Missing/failed/redirected/foreign index loads expose unavailable, distinguish zero matches and complete-empty, and isolate malformed/missing controls from working neighbors.
Verification: CARGO_BUILD_JOBS=4; freeze runs both lint_policy suites including links/fragments. Implementation runs fmt, all-target/all-feature Clippy -D warnings, both policies, Doc owner/corpus, pinned real-browser search witnesses and short native publication E2E. Unchanged heavy proof/evidence/cache/recovery matrices defer to cadence/integration; full cargo test runs only on a named trigger.
Exit: reviewed bounded reader/assets committed after freeze; complete rendering and all excluded gates remain open.
