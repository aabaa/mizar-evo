# Task EDITOR-COMPUTED-RESULTS: computed result display and repair actions
Canonical language: English; [Japanese pointer](../ja/EDITOR-COMPUTED-RESULTS.md).
Status: dependency-blocked. Tier: full gates. Owner: mizar-lsp; consumers: driver and test.
Authority: [§23.5.2 and §23.6](../../../spec/en/23.package_management_and_build_system.md), [§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step), [§22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics).
Classification: source_drift/test_gap; generic metadata/actions do not provide this computation-specific consumer.
Dependencies: LSP snapshot/metadata/action tasks6–17; AST-COMPUTE authentic trust/replay results, AST-ARTIFACT publication and AST-DIAGNOSTICS structured Binder/ghost/import failures; real driver replay request route.
Readiness: consumer design can start; implementation waits for those published inputs and request transport.
Scope: display computed distinctly from kernel-accepted/replayed results; offer explicit replay or computation repair from authentic structured inputs, including Binder, ghost and import diagnostics.
Boundary: Proof certifies status/provenance; Artifact publishes; driver executes requests; LSP only projects and requests. No semantic reconstruction or certification in the editor.
Invariants: snapshot/version and executable/dependency/input/result/trust provenance stay associated; stale metadata is marked and never authorizes a current action. VM reports never become kernel witnesses or kernel-only used_axioms.
Failure: missing replay/repair inputs yield unavailable actions; missing/stale provenance cannot relabel a result or authorize an edit. Suppression/freshness follows existing LSP ownership.
Forbidden: executing algorithms, automatic edits, invented fixes or kernel evidence, artifact mutation, proof acceptance, presenting computed as replayed.
Owner details/tests: [LSP metadata/action tasks](../../mizar-lsp/en/todo.md#wave-c-metadata-features); [Proof publication](../../mizar-proof/en/policy.md#computation-publication-boundary).
Tests: computed versus replayed display; explicit replay request; authentic Binder/ghost/import repairs; missing replay inputs, stale snapshot/source/dependency provenance and invalid fixes; no change to proof status from display/actions alone.
Acceptance: only authentic current inputs enable the specified presentation/actions; computation/proof authority remains outside LSP.
Audit: chapter23 consumer ownership only.
Reviews: independent specification, test sufficiency, implementation, volume/scope and source/EN-JA consistency; full protocol gates.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test.
Owner index: [driver](../../mizar-driver/en/00.crate_plan.md).
