# Task DOC10-ARTIFACT-DIRECT-BY: Artifact-owned citation projection and coverage
Canonical language: English; [Japanese pointer](../ja/DOC10-ARTIFACT-DIRECT-BY.md).
Status: frozen implementation scope. Tier: full; freeze and implementation are one changed task.
Purpose: supply only gate 2 of the adopted [Doc10 prerequisites](../../mizar-doc/en/link_prerequisites.md#ordered-readiness-gates-and-decisions).
Owners: [Artifact plan](../../mizar-artifact/en/00.crate_plan.md), [Doc plan](../../mizar-doc/en/00.crate_plan.md), [Driver plan](../../mizar-driver/en/00.crate_plan.md). Artifact owns projection; Driver retains original compilation inputs; Doc validates the finite scope for later consumers.

## Authority And Scope
Authority: [spec24.2 automatic back-references](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format), [architecture13 Step3](../../architecture/en/13.documentation_and_extraction.md#step-3-resolve-links-and-render-math).
Interface owner: [Artifact-owned producer](../../mizar-doc/en/link_prerequisites.md#artifact-owned-direct-by-producer); reuse [authenticated finite inputs](../../mizar-doc/en/link_prerequisites.md#authenticated-ordered-imports-handoff) and existing validating publication readers.
Classified source_gap: artifacts expose exports and evidence, but no authoritative direct syntactic by projection or examined-body/proof scope. Existing resolver label candidates cover a bounded syntax subset, not every citation endpoint.
Supply an immutable in-memory Artifact-owned handoff from original compilation syntax and compiler-supplied endpoint bindings, captured at the genuine Driver publication boundary. Do not reparse source in Doc or infer endpoints from spelling, exports alone or used_axioms.
Retain citing/cited full module, publication and export identities, original by and target ordinals, and available authentic source provenance. Repeated and self edges remain separate; cross-package endpoints bind to the selected finite inventory. Preserve an unmapped target occurrence and mark coverage incomplete.
Explicitly enumerate examined theorem bodies/proofs. Validate source/publication binding, theorem export endpoints and the whole declared finite scope against gate-1 publications/exports; missing producers, theorem coverage, packages or endpoints never mean absence or complete-empty.
Complete-empty requires authoritative examination of the required scope with no direct edges. Unavailable, incomplete and complete remain distinct; citation coverage and finite import closure coverage remain separate.
Internal API shape, traversal and validation ordering are derived implementation decisions. Reuse native identities and provenance; introduce no serialized artifact/DocIndex schema or proof-acceptance changes.
Forbidden: language-specification intent changes, new semantic choices, ② alternatives, UsedBy presentation/reversal or link resolution, validated allocations, diagnostic adoption/allocation, resolver integration, rendering/publication schemas, owner-gated scope, coverage/owner/milestone credit. All readiness gates after gate 2 remain open.

## Artifacts And Review
Write set: this contract pair; paired Artifact/Doc/Driver crate-plan links; paired Doc link_prerequisites and existing Artifact/Driver owner documents whose boundaries change; necessary Artifact/Doc source and Cargo metadata, bounded Driver publication capture; tests in existing Artifact/Doc/Driver owners. A new private source/test module is permitted only when module ownership requires it. No AGENTS/protocol/spec/coverage-audit/trace edits.
Test owners: crates/mizar-artifact/src/verified_artifact/tests.rs or a private producer test module; crates/mizar-test/src/runner/tests/doc.rs; crates/mizar-driver/src/frontend_adapter/artifact.rs publication tests. Existing .miz intent and expectations stay unchanged.
Reviews: independent specification before freeze; test sufficiency, implementation precision, volume/scope, final source/documentation consistency and scoped quality assessment over the entire unit. Repeat specification only if the contract changes.

## Acceptance Matrix
Six families, cap six; an extra family requires a cited authority:
1. Original syntax projection: repeated by/target occurrences, self and cross-package theorem endpoints retain original ordinals and provenance, independent of storage/inventory order; genuine publication-time capture witnesses the producer path.
2. Authoritative complete-empty body/proof examination differs from absent producer and incomplete examination; empty records alone grant no completeness.
3. Missing/unexamined theorem, module/package, producer or endpoint and unsupported/unresolved syntax preserve known occurrences with unavailable/incomplete coverage; no evidence/name/source fallback.
4. Scope validation: exact finite closure and theorem body/proof inventory, duplicate/foreign examination and occurrence identities, out-of-scope citing/cited identities and non-theorem endpoints cannot manufacture complete coverage.
5. Wrong source/module/package/publication/export binding refuses authentication; existing reader schema/integrity/currentness refusals remain errors. Publication changes cannot reuse stale projections.
6. Compatibility: raw Doc4/gate-1 behavior, artifact bytes/proof acceptance and later gates remain unchanged; evidence-only, @see, Markdown and transitive relationships produce no direct edges.
Verification: CARGO_BUILD_JOBS=4; freeze runs both lint_policy suites and local links/fragments. Implementation runs fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites, Artifact/Doc owner groups, changed Driver publication tests and a short genuine publication end-to-end witness. Run changed publication-binding obligations explicitly; unrelated heavy cache/evidence and resolver edit-versus-clean matrices remain deferred to cadence/integration. Full cargo test only on a named cadence trigger.
Exit: reviewed gate-2 projection and finite scope/endpoint validation committed locally after freeze, push:no; later gates remain open.
