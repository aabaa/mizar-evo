# Task DOC13-G4-TEMPLATE-DEFINITION: complete-page template definition
Canonical language: English; [Japanese companion](../ja/DOC13-G4-TEMPLATE-DEFINITION.md).

## Status And Ownership
Frozen bounded definition; documentation only, full tier: complete-page/DOM consumer contracts exceed the isolated-fragment pattern. Freeze and definition are one changed task; push:no.
[Doc Render](../../mizar-doc/en/00.crate_plan.md) owns template literals and insertion rules; future page/search consumers use them. Doc11, Q4/MT and Diagnostics retain math, degraded-summary and source/readiness meanings.

## Authorities And Gaps
Authorities: [spec24.1–4](../../../spec/en/24.documentation_generation.md), [architecture13 output](../../architecture/en/13.documentation_and_extraction.md#step-4-emit-static-site), [Render inputs and hrefs](../../mizar-doc/en/render.md#authority-and-fixed-inputs), [isolated URL handoff](../../mizar-doc/en/render.md#adopted-isolated-url-handoff), [opaque transport](../../mizar-doc/en/render.md#adopted-opaque-math-transport), [G4 prerequisites](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates).
Design gap: complete package/module/item literals, insertion/escaping, attribute order, canonical bytes and DOM hooks are undefined. Existing isolated fragments do not supply templates. This is internal presentation definition, not language intent or executable completion; no audit/trace/coverage or owner credit changes.

## Scope And Artifacts
Define the page shell and package/module/item bodies, ordered repeated rows and all required-content slots in paired Render documents. Define context-specific insertion, identity/href preservation, deterministic bytes and search/math DOM hooks. Preserve existing fragment bytes, ordinals, payloads and local-asset contracts.
Math, summary and source presentation not yet defined by their owners remains explicitly owner-supplied; no invented fallback, formula, source route, diagnostic or unavailable-input meaning. A slot is not evidence of producer readiness.
Write set: this pair, paired render.md and render_prerequisites.md, and paired Doc crate-plan navigation only. Render owns literals; prerequisites own remaining readiness gates. No new module document or duplicated literal owner.
Forbidden: code/tests/spec/AGENTS/protocol edits, immutable asset manifests or concrete style/search bytes, renderer implementation/witnesses, coherent Doc13 adoption, remaining diagnostic/source/readiness choices, symbolic math, G5/Doc14–15, schemas/publication, Doc23/24 and credit. A language-intent change or genuinely new semantic choice stops for Muse.

## Acceptance Matrix And Reviews
Six documentation witness families, cap six; extra cases require a specification citation:
1. Package overview/module list; module comments/all exports with mathematical signature/summary/dedicated link; item signature/comment/formal contracts/parameter-returns/Used by/See also/source all have explicit slots and literal placement.
2. Full identities, exact FQN ids/validated aliases and existing percent-UTF8 hrefs survive non-ASCII/metacharacters and nested pages without guessed routing or identity merging.
3. Text/attribute/fragment insertion handles metacharacters and retained payload line endings without double escaping, executable interpolation or raw authored HTML insertion; existing URL/math fragments retain their contracts.
4. Complete literal expansions fix bytes, whitespace, attribute order, list ordering, absent/empty optional version and repeated/empty collections without volatile metadata.
5. Fixed local style/search/math script locations and DOM hooks support nested pages, text-only search results and adopted opaque math without changing fragment selectors or asset validation.
6. Unavailable owner inputs remain gates/slots; known-empty is distinct. EN/JA definitions and links agree; no manifest, executable witness or G4 completion is claimed.
Independent specification review before freeze; then documentation sufficiency, precision, volume/scope and final source/EN-JA consistency reviews over the whole unit. Existing CommonMark/URL/math tests are preservation references, not new executable template witnesses.
Verification: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy and cargo test -p mizar-test --test lint_policy, including documentation ledger and local link/fragment checks, at freeze and final tree; git diff --check. No fmt/Clippy/E2E for documentation only; full cargo test only on a named cadence trigger.
Exit: freeze committed first, then reviewed paired template definition committed locally; implementation, owner readiness and every excluded gate remain separate.
