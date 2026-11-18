# Task DOC8-PARSING-CONFORMANCE: documentation parsing
Canonical language: English; [Japanese pointer](../ja/DOC8-PARSING-CONFORMANCE.md).
Status: frozen implementation unit. Tier: full; freeze and implementation form one changed task.
Primary owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md); consumer: documentation builds.
Purpose: implement entry 209 using the [coherent adoption](./DOC8-COHERENT-ADOPTION.md), Q9–Q18 and MT-Q1–Q7 ALL ①.

## Authority And Readiness
[Spec24.1/3](../../../spec/en/24.documentation_generation.md), [spec22 locations](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display), [W0405–W0411](../../../spec/en/22.error_handling_and_diagnostics.md#2269-documentation-markdown-and-tag-warnings), [comments](../../mizar-doc/en/comments.md#markdown-and-structured-tags) and [detail encoding](../../mizar-diagnostics/en/failure_record.md#markdowntag-occurrence-details) govern behavior.
Doc7 already supplies verified attachments and retained raw projection/maps; missing parsing/conformance is source_gap/test_gap. Parser discrepancies against pinned CommonMark examples return to the owner. No new language choice is authorized.

## Scope And Design
Consume Doc7 without reads, source mutation or lower-reader reinterpretation. Pin CommonMark 0.31.2 and pulldown-cmark =0.13.6, default-features=false, Options::empty(). Preserve raw spelling/maps independently of borrowed parser events; add no serialized schema or replacement semantic AST.
Derive Q3 text and segmented mappings, protected CommonMark blocks/containers, display math, logical paragraph fields, then bounded inline math. Retain fields/math as raw ranges and derived views. Tight/loose lists and quotes use structural paragraphs. Duplicate state spans separate blocks only on the same complete target.
Accept optional authoritative target metadata for complete/partial parameter membership and return availability; existing exports have none, so never infer it from rendered signatures. Keep absent metadata inactive.
Emit adopted diagnostics with full occurrence/raw/subreason details, mapped marker/slot/delimiter primary and ordered secondaries or authenticated provenance. Complete a block before committing its drafts; actual conversion/map failures retain raw fallback and ordered independent conditions. Existing aggregation and documentation-only promotion remain authoritative.
Write set: this contract pair; crates/mizar-doc/src/{lib,comments,markdown}.rs and Cargo.toml; Cargo.lock; focused Doc tests under crates/mizar-doc/tests and crates/mizar-test/src/runner/tests/doc.rs; CommonMark fixtures under tests/fixtures; necessary paired comments/plan links. Reuse existing types where practical, with a parsing entry point owned by mizar-doc.
Forbidden: AGENTS/protocol/spec changes; candidate ②; new tags or inferred metadata; Doc9/Doc11 semantics, link resolution, HTML/LaTeX output policy, extraction/publication/proof changes, shared diagnostic schema/comparator changes, trace/audit credit or milestone closure. Coverage ownership stays unchanged.

## Acceptance Matrix
Six families, cap six families; additional cases must cite an authority clause:
1. All pinned CommonMark 0.31.2 examples match the exact parser baseline; disabled extension syntax stays CommonMark. Retain the official corpus attribution/license.
2. Paragraph/tag frontiers: consecutive lines make one field; nested tight/loose list/quote paragraphs qualify; headings/code/HTML/reference definitions and literal lookalikes do not; escapes/entities/Unicode W distinctions and break-spanning raw slots/bodies follow spec24.1.
3. Every known slot/body requirement and exact-raw duplicate policy across same-target blocks; different full targets remain distinct. Complete/partial/unavailable parameter metadata and explicit no-return metadata follow spec24.1, with no inferred checks.
4. Display/inline math: escaping/parity, dollar runs, same-paragraph bounds, opaque slots, code/HTML/reference/link/image/autolink barriers, label-local math, container identity, W-only display and unmatched recovery preserve raw text and standard CommonMark.
5. Q3/container/preprocess maps preserve real segmented positions, UTF-8 and line endings; each condition uses adopted identity/details/locations. Mapping failure emits only W0411 for that block, retains established conditions and leaves other blocks unaffected.
6. Genuine Doc7-to-parser-to-shared-record/CLI/profile flow preserves source order/raw blocks, deterministic occurrence identity/order, unavailable versus empty inputs and documentation-only warning promotion. Existing lower-reader refusals remain tested unchanged.
Tests may use private fault injection for otherwise unreachable genuine view/map failure paths; it must not become a public failure-control API.
Reviews: specification/test intent before freeze; independent sufficiency, precision, volume/scope and final consistency with task-scoped scoring.
Verification: CARGO_BUILD_JOBS=4; freeze runs both explicit lint_policy suites and local links/fragments. Implementation runs fmt --check; all-targets/all-features Clippy -D warnings; both lint_policy suites; mizar-doc tests; mizar-test Doc group; affected diagnostics checks and a short end-to-end parser diagnostic/profile check; git diff --check. Full cargo test only on a named cadence trigger. Unchanged heavy resolver/evidence/cache obligations defer to cadence/integration.
Exit: reviewed parser/conformance and freeze committed locally, push no; no crate or milestone completion claim.
