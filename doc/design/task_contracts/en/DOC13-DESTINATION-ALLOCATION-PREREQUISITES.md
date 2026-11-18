# Task DOC13-DESTINATION-ALLOCATION-PREREQUISITES: bounded G2 destinations
Canonical language: English; [Japanese companion](../ja/DOC13-DESTINATION-ALLOCATION-PREREQUISITES.md).

## Status And Ownership
Frozen bounded prerequisite, full tier: a new G1 allocation consumer and fail-closed refusal classes fail the pattern criteria. Freeze and implementation are one changed task.
Purpose: close only the declared-namespace G2 destination allocation remainder; coherent Doc13 adoption and all other gates remain separate.
Primary owner: [mizar-doc plan](../../mizar-doc/en/00.crate_plan.md); consumers: future render/search and existing Doc9 module-entry links.
Dependencies: [G1 identities](./DOC13-IDENTITY-BINDING-PREREQUISITES.md), [native provenance](./DOC4-NATIVE-SELECTED-ROOT-PROVENANCE.md), and [Doc10 allocations](./DOC10-VALIDATED-ALLOCATIONS.md).

## Authority And Gaps
Authority: [spec24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec24.4](../../../spec/en/24.documentation_generation.md#244-html-output), [Q1/Q2](../../mizar-doc/en/render.md#candidate-destinations-and-page-content), [G2](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates), and [allocation invariants](../../mizar-doc/en/link_prerequisites.md#validated-page-anchor-and-section-allocations).
Existing tests: `published_inputs::allocations` and `published_inputs::doc13` in `crates/mizar-test/src/runner/tests/doc.rs`; genuine native G1 handoffs in Driver integration tests.
Classified source/test gaps: no Q1 producer, paired dedicated-item binding or exact expected coverage check; supplied-table validation lacks reserved/ASCII-case namespace policy. Existing Doc9 table semantics and tests remain intact.

## Frozen Scope And Decisions
Consume immutable authenticated G1 identities and Doc10 inputs; require exact full module/publication/export correspondence across the complete finite inventory, including empty modules. No FQN-derived components, guessed labels or identity merging.
Produce Q1 version root from the selected stored version, module paths from G1 components, and dedicated paths from unique authoritative export labels. Apply Q1 UTF-8 byte spelling, including literal tilde, dot-only and trailing-dot rules; empty components refuse. Preserve exact FQN fragments and original identity bindings.
Return an immutable in-memory paired handoff reusing validated Doc9 module entries and safe optional aliases; Doc9 references remain module-page-plus-full-FQN. Missing/ambiguous labels, duplicate full ids and incomplete inputs refuse the bounded product as a whole.
Accept an explicit complete auxiliary page/section namespace bound to that version output root; auxiliary pages cannot impersonate generated module/item pages. Validate section/version bindings through the existing owner. Unavailable/incomplete section coverage remains independent and cannot fabricate sections.
Validate every generated/auxiliary path and implicit directory together with reserved `index.html`, `search.json`, `cross-references.json` files and the reserved `assets/` subtree. Reject exact duplicates, file/directory conflicts and ASCII-case-folded path/component spelling conflicts, independent of enumeration order. Reserved files have one producer-owned slot; auxiliary/generated pages cannot claim them. No suffixes or identity winners.
Internal representation, API/refusal names and validation order are autonomous. No new semantic choice, diagnostic code, language behavior, hash domain or artifact/schema change.
Forbidden: renderer/HTML/JSON emission, coherent Doc13 adoption, G3–G5, source routes/coordinates, degraded summaries, math/assets/diagnostic adoption, Doc14/15, schema/publication, Doc23/24, owner-gated work and coverage/owner/milestone credit. No spec/audit/trace/AGENTS/protocol edits.

## Artifacts And Review
Write set: this pair, paired Doc plan links and render prerequisite boundary; existing `crates/mizar-doc/src/allocations.rs` and a narrow `doc_build.rs` binding accessor if needed; existing Doc/Driver integration tests only.
Reviews: independent specification before freeze; sufficiency, precision, volume/scope, final source/EN-JA consistency and scoped quality assessment over the whole unit. Specification repeats only if the contract changes.

## Acceptance Matrix
Six families, cap six; extra families require cited authority:
1. Q1 spelling: stored version, one/nested modules and labels produce exact paths; UTF-8, `%`, `#`, tilde, dot-only/trailing-dot and empty components exercise spelling/refusal independently of Q3 href encoding.
2. Exact coverage: complete-empty modules pass; missing/foreign/mismatched module/publication/export inventories and partial auxiliary namespaces refuse without partial results.
3. Paired bindings: each exact export has module-entry and dedicated destinations, exact FQN and validated alias; absent/ambiguous labels, duplicate FQNs/origins and conflicting page ids refuse; existing Doc9 destination remains unchanged.
4. Reserved/file-directory: generated and auxiliary paths collide against all reserved slots/subtree, each other and implicit directories; refusal includes nested-module `algebra`/`algebra.index` and cross-reference reservations.
5. ASCII case: file and directory spelling variants, folded prefix conflicts and same-label distinct identities refuse; distinct non-ASCII bytes remain exact; permuted valid inventories yield equal products.
6. Integration/boundary: genuine Driver publication through G1 to paired G2 positive/refusal, replay/read-only input preservation and lower-reader integrity/currentness refusal; existing Doc10 section/alias behavior remains unchanged.
Verification: CARGO_BUILD_JOBS=4; freeze runs both lint_policy suites and links/fragments. Implementation runs fmt, all-target/all-feature Clippy -D warnings, both policy suites, Doc owner/corpus and affected genuine Driver integration including short end-to-end witnesses. Full cargo test only on a named cadence trigger; unrelated heavy proof/cache/evidence groups defer to cadence/integration.
Exit: reviewed bounded G2 allocation handoff committed locally after freeze, push:no; all excluded gates remain open.
