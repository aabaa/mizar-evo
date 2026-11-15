# Task LSP-PRODUCER-METADATA-PUBLICATION: approved producer implementation
Canonical language: English; [Japanese pointer](../ja/LSP-PRODUCER-METADATA-PUBLICATION.md).
Status: frozen implementation unit. Tier: full; new producer publication boundary.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); consumer: [Task25](EDITOR-LSP-COMPLETION.md).
Purpose: implement [entry-85 design](../../mizar-lsp/en/producer_metadata_publication.md), whose Q1–Q5 option A choices were approved by the owner on 2026-11-16.

## Authority and boundaries

Authority: spec [§2.5.2](../../../spec/en/02.lexical_structure.md#252-user-defined-symbolic-names), [§11.1](../../../spec/en/11.symbol_management.md#111-synonyms-and-antonyms), [§12.3–12.5](../../../spec/en/12.modules_and_namespaces.md#123-import-statements), [§23.6](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support).
Owners: [lexical events](../../mizar-lexer/en/lexical_environment.md), [source correspondence](../../mizar-resolve/en/symbols.md#source-lexical-contribution-producer), [imports](../../mizar-resolve/en/imports.md#alias-binding), [artifact summary](../../mizar-artifact/en/module_summary.md), [snapshot association](../../mizar-lsp/en/snapshot.md#source-association).
Classified gaps: producer incompleteness/design drift: source lexical export rejects operators and NotationAlias; bounded summary candidates do not deliver a complete imported/aliased view.
Publish final publicly eligible spelling-level operator metadata on existing shapes; retain imported/earlier events locally. Each authenticated synonym/antonym has its own declaration origin and separate canonical target/relation. Preserve exact pieces, ordered loci, arity, provenance, activation and visibility; unsupported existing semantic profiles remain unavailable.
Publish an immutable resolver-produced local view alongside the selected publication under its existing source/dependency association; no new serialized artifact or process-local identity encoding. Retain canonical dependency summaries and explicit forwarding edges, validating their acyclic public closure. Ordinary imports and local namespace aliases do not become exports.
Withhold each whole covered product if its required inventory, mapping, relation, dependency or encoding is missing, recovered, ambiguous or unsupported; complete-empty remains valid. Existing diagnostics and independent readers are unaffected.
Forbidden: language/specification changes, Task25 consumer implementation, Task39/40/41, native-payload proposals, provenance/storage/SourceId decisions, new serialized schemas, fabricated producer registration, compiler replay in LSP, proof/acceptance or coverage credit, AGENTS.md/protocol changes.

## Changes and test matrix

Writes: resolver source correspondence/import-summary composition and tests; existing lexer/artifact validation only where required; Driver phase/publication handoff and focused tests; LSP publication retention tests if needed; paired owning boundary documentation and this contract pair. Keep existing semantic checker profiles unchanged.
The approved design owns product semantics; reuse existing source maps, relation metadata, import outcomes, summary readers/codecs and snapshot lifecycle. Owner documents record public seams only. Existing §23.6/SCA-003 credit remains unchanged.
Six cases, split into notation and composition/publication test packets:
1. Genuine frontend multi-piece notation, defaults/explicit operators and later overloads preserve all candidates and declaration-boundary events; outgoing metadata follows Q1-A.
2. Supported local aliases retain alias origin and target/relation; imported/unsupported or ambiguous relations refuse the covered product unless authenticated by existing semantics.
3. Private/recovered/missing owners and incompatible encoding refuse incomplete products without exported leakage; complete-empty is distinguishable.
4. Ordinary/aliased/duplicate/conflicting imports and facade forwarding preserve canonical identities, source bindings, explicit outcomes and public closure; reject missing/private/cyclic forwarding.
5. Wrong/missing dependency module/interface/schema, unpaired lexical envelopes and stale mappings refuse publication; consume owner-validated summaries only.
6. Permutations/rebuilds are deterministic; exported semantic changes invalidate interfaces; edit/reopen/replacement retains original coordinates and one immutable source/dependency set, explicitly stale without equal-build-id revival.
Existing anchors: `crates/mizar-resolve/src/symbols/tests.rs::source_lexical_pairing_rejects_incomplete_or_ambiguous_correspondence`, `src/imports/tests.rs::aliases_do_not_change_canonical_targets_or_graph_candidates`, `src/module_summary_reuse.rs::typed_lexical_shapes_bind_to_artifact_exports`; `crates/mizar-artifact/src/module_summary/tests.rs::module_summary_round_trips_through_canonical_json`. Extend `crates/mizar-test/src/runner/tests/parse_only.rs` notation-producer cases and Driver `src/frontend_adapter/artifact.rs` coherent-publication tests; retain LSP `src/metadata_tests.rs` lifecycle checks.

## Reviews and exit

Independent specification/test-intent review before freeze; test sufficiency, precision, volume/scope, consistency and final full-tier assessment over freeze plus implementation.
Before freeze: both lint_policy suites and local link/fragment checks. Implementation: fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites, affected resolver/lexer/artifact/Driver/LSP groups and representative end-to-end publication/refusal. Run changed identity/hash/snapshot obligations immediately; full cargo test only on a named cadence trigger.
Exit: local freeze then implementation commits, one changed task; no push. Complete producer-to-validating-reader tests, whole-product refusal and coherent owner handoff; preserve unrelated edits. Task25 still requires its own redispatch.
