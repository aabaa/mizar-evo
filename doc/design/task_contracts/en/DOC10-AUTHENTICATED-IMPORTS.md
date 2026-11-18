# Task DOC10-AUTHENTICATED-IMPORTS: authenticated imports and finite closure
Canonical language: English; [Japanese pointer](../ja/DOC10-AUTHENTICATED-IMPORTS.md).
Status: frozen implementation scope. Tier: full; freeze and implementation are one changed task.
Purpose: supply only gate 1 of the adopted [Doc10 prerequisites](../../mizar-doc/en/link_prerequisites.md#ordered-readiness-gates-and-decisions).
Owners: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [Driver plan](../../mizar-driver/en/00.crate_plan.md); producer: Driver publication boundary using Resolver's immutable PublishedImportView; consumers: later Doc10/Doc13.

## Authority And Scope
Authority: [spec24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [architecture13 Step3](../../architecture/en/13.documentation_and_extraction.md#step-3-resolve-links-and-render-math), [internal05 index](../../internal/en/05.documentation_extraction.md#documentation-index).
Interface owner: [authenticated inputs](../../mizar-doc/en/link_prerequisites.md#authenticated-import-and-dependency-inputs); existing [Doc4 inputs](../../mizar-doc/en/doc_build.md#index-inputs-and-later-completion) and [validating reader](../../mizar-doc/en/artifact_reader.md#published-input-loading) retain their contracts.
Producer/API owners: [Driver metadata publication](../../mizar-driver/en/frontend_adapter.md#producer-metadata-publication), [Resolver published view](../../mizar-resolve/en/imports.md#published-import-view). Existing test owners: crates/mizar-test/src/runner/tests/doc.rs publication/DocBuild fixtures; crates/mizar-resolve/src/imports/tests.rs publication tests; crates/mizar-driver/src/frontend_adapter/artifact.rs publication end-to-end tests. These bound the focused commands below.
Classified source_gap: raw Doc4 import vectors/dependency references do not authenticate semantic order, publication endpoints or closure coverage. Driver's publication metadata currently cannot load nonempty import summaries.
Reuse genuine PublishedImportView source order/provenance and existing validating publication export/summary readers; supply a separate in-memory authenticated handoff over an explicitly enumerated finite publication inventory rooted at the selected publication.
Bind source and target full module/package/publication identities and stored summaries/exports; preserve duplicate routes, distinguish distinct full targets and check every required dependency edge. Inventory enumeration order never selects an endpoint or changes import order.
Retain unavailable, incomplete and complete (including authoritative complete-empty) coverage per module/namespace. Missing views, summaries or inventory endpoints cannot establish absence or completeness. Cycles stay inside the finite supplied inventory; no recursive discovery, source or network fallback.
Keep lower-reader schema/integrity/currentness/binding errors as refusals. A producer/view provenance mismatch refuses the handoff; missing coverage stays unavailable/incomplete.
Internal representation, finite lookup and validation ordering are implementation decisions; no serialized DocIndex schema or language semantic choice is authorized. Stop on a new language-specification intent or genuinely new semantic choice.
Forbidden: Artifact direct-by production, allocations, diagnostic adoption/allocation, resolver integration, ② alternatives, Doc13/23 adoption, proof/acceptance changes or coverage/owner/milestone credit. All later readiness gates remain open.

## Artifacts And Review
Write set: this contract pair; paired mizar-doc/link_prerequisites.md and Doc/Driver 00.crate_plan.md; necessary mizar-doc source/Cargo metadata; bounded Driver publication capture and its existing paired owner documentation if its boundary changes; focused tests in existing Doc/Resolver/Driver test owners. No AGENTS/protocol/spec/coverage-audit/trace changes.
Preserve Doc4's raw optional fields and public behavior; the authenticated handoff is separate. Existing validated publication indexes supply index locations and complete module/export inventories without allocating a serialized documentation index.
Reviews: independent specification before freeze; test sufficiency, implementation precision, volume/scope and final consistency over freeze plus implementation. Specification is repeated only if this contract changes.

## Acceptance Matrix
Six families, cap six; an additional family requires a cited authority:
1. Genuine nonempty producer imports: reversed inventory/storage order, repeated routes and cross-package/full-identity targets retain original semantic order and occurrence provenance; Driver can retain nonempty publication metadata.
2. Genuine complete-empty producer and dependency inventory: explicitly complete namespace with no edges, distinct from unavailable and incomplete inputs.
3. Missing producer/summary/endpoint or unresolved semantic import: retain known authenticated inputs and unavailable/incomplete coverage without negative-lookup credit.
4. Finite dependency inventory: all required edges checked, cycles bounded, duplicate identities/routes handled without winner selection; omitted edges and colliding names cannot imply complete coverage.
5. Wrong package/publication/source/summary binding, stale inputs, unsupported schema and integrity failures: refuse with existing lower-reader errors where applicable; no partial authenticated success or fallback.
6. Boundary compatibility: raw Doc4 options unchanged; later gates remain open; no link/UsedBy/href/diagnostic emission or language/proof behavior changes.
Verification: CARGO_BUILD_JOBS=4; both lint_policy suites and local link/fragment checks at freeze; fmt, all-target/all-feature Clippy with -D warnings, both lint_policy suites, affected Doc/Resolver/Driver owner tests and a genuine short publication end-to-end witness for implementation. Run changed identity/publication obligations explicitly; defer unrelated heavy cache/evidence matrices. Full cargo test only on a cadence trigger.
Exit: reviewed gate-1 producer/authentication and witnesses committed locally after the freeze, push:no; later gates remain open.
