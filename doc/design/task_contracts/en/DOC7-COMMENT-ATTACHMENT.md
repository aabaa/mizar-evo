# Task DOC7-COMMENT-ATTACHMENT: validated comment attachment
Canonical language: English; [Japanese pointer](../ja/DOC7-COMMENT-ATTACHMENT.md).
Status: frozen implementation scope. Tier: full; freeze and implementation form one changed task.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md); consumers: later Doc index/render stages.
Dependencies: [Q1 consumption](DOC-Q1-VALIDATED-PUBLICATION-CONSUMPTION.md), [Documentation diagnostics](DOC-DIAGNOSTICS-ADOPTION.md); ready under approved Q1–Q8 option 1.

## Authority And Gaps
Authority: spec [2.9](../../../spec/en/02.lexical_structure.md#29-comments-and-annotations), [24.1–2](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax), [22.6.8](../../../spec/en/22.error_handling_and_diagnostics.md#2268-documentation-attachment-warnings); [comments](../../mizar-doc/en/comments.md), [projection](../../mizar-artifact/en/comment_projection.md), [bound plans](../../mizar-doc/en/doc_build.md#full-build-and-index-inputs).
Dispatch adopts existing owner approvals Q1–Q4 option 1 and Q5–Q8 option 1; stale unselected/gated owner wording is design_drift. Missing grouping/attachment and regressions are source_gap/test_gap. No new language decision or lower-owner change is authorized.

## Scope And Boundary
Add the comments module's read-only attachment API over validated DocBuildPlan inputs. Preserve complete publication/module and original export origin/FQN identity; return source-ordered blocks with raw lines/ranges, original projection/maps, candidate provenance, attachment or shared Documentation warnings. Borrow original inputs rather than duplicate semantic representations.
Group maximal consecutive documentation lines, preserving indentation and blank documentation lines; ordinary comments/physical blank lines separate blocks. Approved Q5/Q6 retain every eligible module-prefix block separately in order, module-only. Imports, source items and standalone annotations end that prefix. Item attachment requires uninterrupted adjacency, supported named kind, exact verifier identity/label and compatible mapped target boundary; item-attached annotations preserve only their owning item's adjacency.
Missing/unsupported, conflicting, invalid/unverified or unmappable targets stay unattached and retain text/provenance under W0400–W0404 as defined by spec22. Never choose by name, range alone or enumeration. Preserve unavailable versus available-empty inputs and deliberate public/private omission rules; private-doc options grant no metadata. Warning promotion affects documentation outcome only.
Use genuinely source-bound block spans when provided by the consumer; otherwise retain portable mapped ranges and use the shared full-provenance Documentation non-range location, never reconstructed SourceIds. Preserve lower-reader refusal layers, source order and immutable artifact/projection bytes. Coordinate comparison composes retained maps rather than equating lexical and loaded offsets.
Writes: mizar-doc comments.rs/lib.rs/Cargo.toml and necessary lock metadata; appended attachment tests in mizar-test runner/tests/doc.rs; this contract pair; paired comments owner, Doc plan/TODO and Artifact projection handoff wording. No existing fixture/expectation intent changes.
Forbidden: AGENTS/protocol/spec edits, new codes/schema/producer/lower-stage authority, native-source fallback, semantic reanalysis, Markdown/tag parsing, links/math/rendering/extraction/publication or option-2 alternatives. No broad Doc4/5, Doc8, crate or Steps6–8 closure. Coverage audit changes only if attachment ownership/coverage actually changes; other deferred coverage remains open.

## Test Matrix And Exit
Six attachment test families, cap six (subcases share obligations; extras require a cited authority clause):
1. Module prefix with whitespace, ordinary comments, blank lines and separated blocks: every eligible block remains distinct, source ordered and module-only.
2. Named definition/theorem/registration/algorithm with verified identity, stored labels and mapped ranges, including attached annotations: only the exact supported item attaches.
3. Physical blank line, ordinary comment, import, unrelated item, standalone annotation and trailing block interruptions: no skipping, orphan warnings and retained text.
4. Missing identity/label, unsupported/nested target, recovered/unverified target and contradictory candidates/ranges: unresolved/ambiguous/unverified warnings, no guessed winner.
5. Unicode/BOM/newline/preprocessing maps and absent/incompatible association coordinates: exact composition or W0404; preserved raw maps; genuine spans or provenance fallback; colliding names retain complete identities.
6. Replay/input permutation, unchanged bytes, unavailable/empty inputs, private-doc request and warning promotion: deterministic results without verification changes; existing Q1 lower-error regressions remain authoritative.
Reviews: independent specification before freeze; test sufficiency, implementation precision, volume/scope, final consistency over the whole unit.
Freeze: both lint_policy suites including link/fragment checks and git diff --check. Implementation: fmt, all-target/all-feature Clippy -D warnings, both lint suites, mizar-doc tests, explicit six families, complete doc_* owner tests, Artifact q1_ tests and representative proof-verification end-to-end; CARGO_BUILD_JOBS=4. Unchanged resolver/evidence/cache heavy obligations defer to cadence. Full cargo test only on a named trigger.
Exit: frozen scope implemented, independently reviewed, focused checks passed and committed locally; no push.
