# Task DOC-Q1-VALIDATED-PUBLICATION-CONSUMPTION: validated comment inputs
Canonical language: English; [Japanese pointer](../ja/DOC-Q1-VALIDATED-PUBLICATION-CONSUMPTION.md).
Status: frozen implementation scope. Tier: full; freeze and implementation form one changed task.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md); consumer: Doc7, deferred.
Dependencies: [Driver emission](DRIVER-Q1-EMISSION-INTEGRATION.md), [Artifact manifest integration](ARTIFACT-Q1-MANIFEST-INTEGRATION.md); ready for bounded consumption.

## Authority And Gaps
Authority: spec [2.9](../../../spec/en/02.lexical_structure.md#29-comments-and-annotations), [24.1](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax), approved option 1 [Doc6 Q1](../../mizar-doc/en/comments.md#owner-q-decisions), Artifact [projection invariants](../../mizar-artifact/en/comment_projection.md) and [declared profiles](../../mizar-artifact/en/manifest.md#declared-enclosing-profiles).
[Reader](../../mizar-doc/en/artifact_reader.md#published-input-loading) reloads validated references with only legacy codecs; [planning](../../mizar-doc/en/doc_build.md#full-build-and-index-inputs) discards projected inputs: source_gap/test_gap. Neither requires new language semantics or lower-owner changes.

## Scope And Boundary
Add read_published_artifacts_with_comments with the existing reader arguments and Result<(PublishedManifestRead, Vec<(VerifiedArtifact, CanonicalJson)>), ManifestError>. Preserve complete manifest/path/hash and module order. Validate manifest references/witness files first, then reload each entry with its declared family/version/domain/exclusions, actual path and expected artifact/module/interface/implementation hashes. Use the existing commented codec only for its declared profile; preserve normal/Builtin selection and all original error layers. No retry or downgrade after failure.
Keep read_published_artifacts signature and legacy results by delegating and discarding projections. Legacy modules return null projection; commented envelopes retain their validated comments unchanged, including null unavailable versus available-empty objects and present empty doc lines. Existing export/summary/label readers must work with declared envelopes.
Add DocModuleInput.comment_projection: CanonicalJson. build_doc_plans consumes the paired reload output and retains each projection with its original module entry/full identity and selected manifest/publication hashes only after existing package/source/provenance/current-request binding. No separately indexed projection, fabricated availability, partial return or weakened binding.
Writes: Doc artifact_reader.rs/doc_build.rs; appended regressions in mizar-test/src/runner/tests/doc.rs (existing fixture reuse/extension only as needed); this paired contract, paired reader/build owners, paired Artifact projection handoff status and Doc crate-plan backlink. Keep legacy fixtures/expectations intact.
Lower Artifact owns codec/schema/hash/source/map validation and refusals; Session owns snapshot currentness. Doc retains validated data without producer, attachment or proof authority. No lower-stage or schema changes.
Forbidden: AGENTS/protocol/spec/test-intent/trace edits; attachment, tag/Markdown parsing, source fallback, new diagnostics, rendering/extraction/publication, evidence/proof/cache/reuse authority, option-2 alternatives or owner-gated semantics. No audit coverage/ownership change or broad Doc4/5, Doc7/8, Artifact17 or Steps6–8 closure.

## Test Matrix And Exit
Six genuine-publication cases, cap six (subcases share obligations; additions require cited higher authority):
1. Mixed normal/Builtin/commented modules retain complete typed artifacts, exact raw projection/maps/order/full identity, manifest/hash and legacy-reader results; export/summary/label consumers remain usable, with read-only bytes.
2. Legacy unavailable, envelope null, available-empty and present empty doc line stay distinct as defined by Artifact; optional native source is absent and never consulted.
3. Current-request-bound plans retain exact projections/module entries under colliding module paths across package identities, empty modules and canonical replay; no namespace merge or attachment.
4. Rehashed malformed envelope/maps or unsupported/mixed profiles refuse in original lower error layers before any usable result, with no fallback; reuse lower exhaustive matrices.
5. Wrong selected hash, source/package/provenance/request/snapshot binding refuses plans, including a later bad publication after a valid one; no partial result.
6. Comment-only changed publication retains changed projection/source binding and selected hash; obsolete hash/source snapshot refuse; old plans stay unchanged.
Reviews: independent specification before freeze; sufficiency, precision, volume/scope and final consistency over the unit.
Freeze checks: both lint_policy suites including links/fragments and git diff --check. Implementation: fmt, Clippy, both lint suites, mizar-doc tests, explicit new cases and complete doc_* owner tests, Artifact q1_ regressions and representative proof-verification end-to-end; CARGO_BUILD_JOBS=4. Unchanged resolver/evidence/cache heavy obligations defer to cadence. Full cargo test only on a named cadence trigger.
Exit: bounded reader and bound-plan consumption reviewed, checked and committed locally; no push.
