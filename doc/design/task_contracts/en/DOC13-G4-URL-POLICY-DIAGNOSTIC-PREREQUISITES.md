# Task DOC13-G4-URL-POLICY-DIAGNOSTIC-PREREQUISITES: URL output diagnostic support
Canonical language: English; [Japanese companion](../ja/DOC13-G4-URL-POLICY-DIAGNOSTIC-PREREQUISITES.md).

## Status And Ownership
Frozen bounded prerequisite; full tier because it adds a diagnostic. Freeze and implementation count as one changed task, push:no.
Render owns output policy; [Diagnostics](../../mizar-diagnostics/en/00.crate_plan.md) owns shared allocation/location/order/profile support; [Doc](../../mizar-doc/en/00.crate_plan.md) supplies existing provenance/profile witnesses.

## Authority And Gaps
Authorities: [Q5/Q12](../../mizar-doc/en/render.md#candidate-destinations-and-page-content), [G4](../../mizar-doc/en/render_prerequisites.md#adopted-g4-opaque-client-prerequisite), [registry](../../mizar-diagnostics/en/registry.md#allocation-rules), [spec22](../../../spec/en/22.error_handling_and_diagnostics.md#22610-documentation-link-warnings), [spec24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks).
Design gap: Q5 output safety and Q12 URL descriptor/details/precedence are not adopted. Source gap: rejected-URL registry support is absent. No language-intent change, audit/trace/coverage or owner credit.

## Frozen Scope
Adopt escaped visible raw HTML without warning and the exact ordinary link/image safety predicate/fallback in [Render](../../mizar-doc/en/render.md#adopted-html-and-url-output-policy).
Adopt one W0415 `documentation.render.rejected_url`, summary `Documentation URL is rejected by output policy`, with the exact [shared owner contract](../../mizar-diagnostics/en/documentation_urls.md). Record allocation in paired spec22 without changing language intent.
Use existing String/List details, mapped primary/secondary segments or true block provenance, dedup/order/freshness and Documentation-only profile failure. Implement registry support and explicit draft witnesses only; no diagnostic factory or new public API.
Preserve raw/interpreted destinations, Doc10 classification/resolution and CommonMark recognition. Accepted URLs and escaped HTML create no URL-policy warning; rejected occurrences use safe visible label/alternative content without href/src.
Detection/emission, failed-view/lower-refusal integration witnesses, page rendering, remaining Q12/source/readiness diagnostics, complete templates/attribute order/style/search manifests, symbolic math, remaining G4/G5, coherent Doc13, Doc14–15, schemas/publication, Doc23/24 and all credit remain gated.

## Artifacts And Reviews
Write set: this pair; paired Render/prerequisite/plan and Diagnostics registry/record/plan boundaries; shared URL owner pair; paired spec22; registry source and focused existing Diagnostics tests; existing Doc profile witness if needed. No AGENTS/protocol/spec24/test-intent edits.
Independent specification review before freeze; sufficiency, precision, volume/scope and final source/EN-JA consistency plus scoped hard gates/score after implementation.

## Acceptance Matrix
Six families, cap six; additions require an authority citation:
1. Exact descriptor/code/meaning/summary/family/default/lifecycle/since/doc_url; old allocations unchanged and remainder unallocated.
2. Policy definition review: relative/fragment/empty/network-path and mixed-case permitted schemes; rejected other/malformed/control-obfuscated schemes and backslashes; entity-decoded versus percent-encoded spellings; HTML escaped without warning. Executable detection witnesses remain gated.
3. Explicit drafts retain exact raw/interpreted destination, link/image kind, full target, block/occurrence ordinals and logical offsets; contiguous/discontiguous segments are unbridged and unmapped occurrences use provenance.
4. Repeated/coincident occurrences survive, duplicate reports collapse; permutation-independent order, freshness and full publication/module distinctions use existing machinery.
5. Define overlap precedence: lower refusal suppresses dependent drafts; failed view owns its block; Doc10 classification warnings and URL-policy warnings cannot duplicate one destination. Integration witnesses remain gated.
6. Explicit draft-to-sink/aggregation/CLI plus existing Doc profile witness retains Warning/code and documentation-only failure, unchanged proof/extraction/publication inputs; no detection claims.
Verification: CARGO_BUILD_JOBS=4; freeze both lint_policy suites and local links/fragments. Implementation fmt, all-target/all-feature Clippy -D warnings, both policies, Diagnostics and Doc owner tests, Doc corpus and short native/diagnostic E2E. Full cargo test only on a named cadence trigger.
Exit: reviewed support committed locally after freeze; remaining gates stay open.
