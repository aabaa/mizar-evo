# Task DOC10-VALIDATED-ALLOCATIONS: validated destinations
Canonical language: English; [Japanese pointer](../ja/DOC10-VALIDATED-ALLOCATIONS.md).
Status: frozen implementation scope. Tier: full; freeze and implementation are one changed task.
Purpose: supply only gate 3 of the adopted [Doc10 prerequisites](../../mizar-doc/en/link_prerequisites.md#ordered-readiness-gates-and-decisions).
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md); later render and link consumers use the independent validated handoff.

## Authority And Scope
Authority: [spec24.2 pages, anchors and hrefs](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [architecture13 Step3](../../architecture/en/13.documentation_and_extraction.md#step-3-resolve-links-and-render-math).
Interface owner: [validated allocations](../../mizar-doc/en/link_prerequisites.md#validated-page-anchor-and-section-allocations); dependencies: supplied [gate-1 identities](../../mizar-doc/en/link_prerequisites.md#authenticated-ordered-imports-handoff) and [gate-2 projection](../../mizar-doc/en/link_prerequisites.md#authenticated-direct-by-handoff).
Classified source_gap: authenticated exports and summaries exist, but no output-root-bound destination validator or adopted href encoder exists.
Supply an immutable in-memory handoff validating an explicitly supplied logical destination table against gate-1 full module/publication/export identities and authoritative stored labels. Do not discover paths, split FQNs to invent labels, select identities or re-read source.
Validate the entire supplied page set before exposing destinations: root binding, relative logical paths, duplicate paths and file/directory collisions. Do not choose filesystem case policy, reserved render paths or directory spelling.
Exact full FQN ids remain unchanged. Distinct items sharing a full id block that page; unavailable authoritative labels block those item anchors. Optional short aliases require unique documented labels and no collision with any full, section or other alias id; suppress unsafe aliases without changing full ids.
An explicit generated-specification inventory carries version, logical section ids, destinations and unavailable/incomplete/complete coverage. Validate version binding, unique logical ids and actual declared page/fragment existence. Missing sections in a complete inventory remain unresolved; unavailable/partial coverage never establishes absence or becomes a module anchor.
Retain logical paths/fragments and exact DOM ids. Encode hrefs separately using the adopted UTF-8 unreserved-byte rule per path segment and whole fragment; encoding never supplies allocation readiness.
Internal row/API shape and validation ordering are derived decisions. Page/fragment existence is checked against the explicit allocation table; no rendering, filesystem publication or section grammar is introduced.
Forbidden: language intent or semantic changes, ② alternatives, diagnostic adoption/allocation, resolver/link lookup or UsedBy integration, Doc13 rendering/source routes, Doc23 publication, serialized schema, owner-gated scope or coverage/owner/milestone credit. Every gate after gate 3 remains open.

## Artifacts And Review
Write set: this contract pair; paired Doc crate-plan links and link_prerequisites boundary documentation; necessary mizar-doc source and existing Doc tests under crates/mizar-test/src/runner/tests/doc.rs. A private allocation source module is permitted for this distinct owner boundary. No AGENTS/protocol/spec/audit/trace edits.
Reviews: independent specification before freeze; test sufficiency, implementation precision, volume/scope and final source/documentation consistency with scoped quality assessment over the whole unit. Repeat specification only if the contract changes.

## Acceptance Matrix
Six families, cap six; an extra family requires a cited authority:
1. Bound genuine publication inputs: module/publication/export and exact stored label checks distinguish packages/versions, reject foreign/stale identities and mismatched output root; raw Doc4/gate-1/gate-2 behavior stays unchanged.
2. Complete destination-set validation: duplicate module allocations, same page paths, file/directory collisions and absolute/escaping paths refuse readiness without winner/suffix/guessed path; incomplete tables cannot authenticate destinations.
3. Full anchors: exact FQN id, missing/foreign label and duplicate-FQN distinct items witness item/page blocking while unaffected destinations remain usable; no label inference or normalization.
4. Aliases: safe unique authoritative short label survives; duplicate labels and collisions with full ids, section ids or aliases suppress only optional aliases, independent of input order.
5. Sections: explicit complete-empty versus unavailable/partial index, complete missing lookup, version mismatch, duplicate logical ids and absent page/fragment; unknown sections never become local anchors.
6. Hrefs: non-ASCII/percent/hash path and fragment encoding, separators and unreserved bytes, exact logical ids retained; invalid/missing allocations cannot become linkable through encoding. Short end-to-end witness authenticates publication, validates a supplied destination and encodes its href without resolver/render integration.
Verification: CARGO_BUILD_JOBS=4; freeze runs both lint_policy suites and local links/fragments. Implementation runs fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites, Doc owner/corpus tests and new allocation families including the short handoff witness. Existing publication refusal tests run with the Doc owner group. Unchanged heavy cache/evidence and resolver edit-versus-clean obligations defer to cadence/integration. Full cargo test only on a named cadence trigger.
Exit: reviewed gate-3 handoff committed locally after freeze, push:no; later gates remain open.
