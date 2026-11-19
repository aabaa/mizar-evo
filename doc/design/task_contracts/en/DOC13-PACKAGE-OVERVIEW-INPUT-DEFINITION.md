# Task DOC13-PACKAGE-OVERVIEW-INPUT-DEFINITION: package overview input
Canonical language: English; [Japanese companion](../ja/DOC13-PACKAGE-OVERVIEW-INPUT-DEFINITION.md).

## Status And Ownership
Frozen bounded definition; documentation only, full tier: a new package-level owner-input boundary has no adopted predecessor. Freeze and definition form one changed task; push:no.
[Doc Render](../../mizar-doc/en/00.crate_plan.md) owns this input/slot definition; the documentation owner supplies package content and authenticated safe-flow provenance. Future package-page production consumes it. Doc8/9 retain comment interpretation/link meanings; Diagnostics retains remaining Q12 policy.

## Authorities And Gaps
Authorities: [spec24.1–4](../../../spec/en/24.documentation_generation.md), [architecture13 output](../../architecture/en/13.documentation_and_extraction.md#step-4-emit-static-site), [Doc4 selected plans](../../mizar-doc/en/doc_build.md#package-planning), [Doc8 targets](../../mizar-doc/en/comments.md#blocks-and-attachment-targets), [template slots](../../mizar-doc/en/render.md#owner-slots-and-dom-hooks), [ordered gates](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates), [dispatch ordering](../../todo.md#m1-dispatch-policy).
Design gap: the package-overview slot lacks a package-level input/absence definition. Existing authenticated module/item comments and the isolated package Modules expansion supply no overview. Missing producer/adoption is a deferred source gap, not negative content evidence. No language, test intent, audit/trace/coverage or completion-credit change.

## Scope And Artifacts
Define a format-neutral, explicitly selected documentation-owner package overview product: exact selected package/publication and current-request binding, retained owner content/provenance and safe rendering evidence, and distinct available, known-absent and unavailable states. A bare identity/hash or caller HTML cannot authenticate content or its absence.
Define verbatim safe-flow insertion, empty-content behavior and the unchanged package wrapper; reuse adopted escaping, URL/link/math and block-local recovery rules without assigning package text a module/item attachment. No implicit README, manifest, source-header, module aggregation or network fallback.
Write set: this pair, paired render.md and render_prerequisites.md, and paired Doc crate-plan navigation only. Render is the single input-definition owner; prerequisites link the still-required producer/adoption/witness gates.
Forbidden: production/test/fixture/spec/AGENTS/protocol edits, executable witnesses, concrete API/type/schema allocations, new source-discovery or attachment semantics, diagnostic descriptors/refusals, formal slots, complete rendering/search/auxiliary interface adoption, G4/G5 closure, Doc14/15, schemas/publication, Doc23/24 or credit. Language-specification intent changes or genuinely new semantic choices stop for Muse; presentation, naming, provenance and internal structure are autonomous.

## Acceptance Matrix And Reviews
Six documentation witness families, cap six; additions require an authority citation:
1. Explicitly selected owner content with exact root package/version/lock/publication/current-request provenance supplies one complete safe flow fragment; dependency-only, foreign, stale, duplicate/conflicting or altered inputs never satisfy the root slot.
2. Authenticated complete owner selection with no overview yields known absence; available-empty remains available. Both insert zero fragment bytes into the unchanged overview wrapper, without invented text/nodes.
3. Missing owner, selection, content, completeness or rendering/authentication evidence leaves the slot unavailable; absence is never inferred from missing inputs, an empty module set or a hash. Lower integrity/currentness refusals retain their meanings; no new build refusal/diagnostic is allocated.
4. Metacharacters, raw HTML, URLs, semantic references, opaque math and recovered blocks follow their existing owners; safe fragment bytes/line endings are inserted once, never accepted as arbitrary caller HTML or reinterpreted as templates.
5. Module/item attachment and source coordinates stay unchanged; package content has explicit owner provenance/context, without guessed current module, README/source fallback or new serialized fields.
6. EN/JA definitions and links agree; definition-only completion supplies no executable producer, complete-page/search/adoption witness or later-gate/credit change.
Independent specification/EN-JA review before freeze; then documentation sufficiency, precision, volume/scope and final consistency over the whole unit. Existing published-input/comment/render tests are preservation references, not package-overview witnesses; no executable test matrix is added.
Verification at freeze and final tree: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy and cargo test -p mizar-test --test lint_policy (ledger and local links/fragments), plus git diff --check. Documentation-only fmt/Clippy/E2E do not apply; full cargo test only on a named cadence trigger.
Exit: commit freeze first, then the reviewed paired definition locally; implementation and all remaining gates stay separate.
