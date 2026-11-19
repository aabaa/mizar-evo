# Task DOC13-Q3-SOURCE-ROUTING-DEFINITION: source-view and line routing
Canonical language: English; [Japanese companion](../ja/DOC13-Q3-SOURCE-ROUTING-DEFINITION.md).

## Status And Ownership
Frozen bounded definition; documentation only, full tier: the original-source presentation/routing boundary has no reviewed source-view predecessor. Freeze and definition are one changed task; push:no.
[Doc Render](../../mizar-doc/en/00.crate_plan.md) owns source-view allocation, line anchors and source-slot presentation; later rendering consumes the adopted G3 original-coordinate handoff. Session/Artifact retain source/map authority; Diagnostics retains Q12 definitions.

## Authorities And Gaps
Authorities: [spec24.2 hrefs](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec24.4 source link](../../../spec/en/24.documentation_generation.md#244-html-output), [architecture source availability](../../architecture/en/13.documentation_and_extraction.md#documentation-is-not-a-publication-gate-for-proof-validity), [G3 coordinates](../../mizar-doc/en/render_prerequisites.md#adopted-g3-source-coordinates), [Q3 inputs](../../mizar-doc/en/render.md#candidate-destinations-and-page-content), [template slots](../../mizar-doc/en/render.md#owner-slots-and-dom-hooks), [Session original/loaded boundary](../../mizar-session/en/source_map.md).
Design gap: authenticated original byte segments do not yet define source-view paths, line anchors, range-to-line routing or source presentation. Existing coordinate witnesses in mizar-test runner/tests/doc.rs and mizar-driver tests/request.rs are preservation references, not source-rendering witnesses. No language intent, coverage/audit/trace or owner-credit change.

## Scope And Artifacts
Define version-local source-view allocation and collision binding, original-text line calculation, relative Q3 hrefs, escaped source-view/source-slot representation, ordered discontiguous/partial/empty segments and unavailable inputs. Preserve exact original provenance and existing missing-source warning policy without allocating diagnostics.
Write set: this contract pair, paired render.md and render_prerequisites.md, and paired Doc crate-plan navigation only. Render owns routing/presentation; prerequisites own outstanding readiness. No new source/map authority or serialized schema.
Forbidden: code, tests, executable witnesses, spec/AGENTS/protocol edits, consumer source fetch, guessed offsets, source normalization, semantic reclassification, diagnostic descriptors/profiles, joint Q4/MT summary definition, Q12 completion, coherent Doc13/G4 closure, G5/Doc14–15, schemas/publication, Doc23/24 or credit. Language-specification intent changes or genuinely new semantic choices stop for Muse; presentation, naming and internal structure are autonomous design.

## Acceptance Matrix And Reviews
Six documentation witness families, cap six; extra cases require an authority citation:
1. Same/different full module/publication/path/original bindings allocate authenticated version-local views or refuse collisions; no FQN/path/hash guessing, absolute routes or source fetch.
2. Original UTF-8, LF, CRLF, lone CR, empty text, trailing LF and EOF points have explicit one-based line anchors without loaded/normalized offset reuse.
3. Ordered contiguous/discontiguous, repeated, partial and unavailable segments retain exact provenance; no bridging gaps or turning incomplete maps into complete locations.
4. Nested pages and non-ASCII/metacharacter paths use existing Q3 encoding and attribute escaping once; source text cannot become HTML or executable interpolation.
5. Available source/view without maps, absent source and contradictory bindings remain distinct; visible source-slot behavior preserves missing-source warning and lower-reader refusal without invented diagnostic meaning.
6. EN/JA routes, literals and links agree; definition supplies no implementation/readiness witness and preserves every excluded gate.
Independent specification review before freeze, then documentation sufficiency, precision, volume/scope and final source/EN-JA consistency reviews over the whole unit.
Verification: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy and cargo test -p mizar-test --test lint_policy (documentation ledger and local links/fragments) at freeze and final tree; git diff --check. No fmt/Clippy/E2E for documentation only; full cargo test only on a named cadence trigger.
Exit: commit freeze first, then the reviewed paired definition locally; implementation and remaining gates stay separate.
