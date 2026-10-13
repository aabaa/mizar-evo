# Task AST-LOWER: Immutable reflected syntax handoff
Canonical language: English; [Japanese pointer](../ja/AST-LOWER.md).
Status: planned. Tier: full gates. Primary owner: [mizar-core](../../mizar-core/en/00.crate_plan.md).
Consumers: mizar-vc, the MVM producer and mizar-proof.
Dependencies: request-only formation transport needs authentic AST-TYPE provisional structure/formulas; final checked handoff needs accepted AST-TYPE structure, conditions, UID/context and complete ordinary/syntax/reconstruction inputs.
Readiness: request-only transport in AST-FORMATION-PROOFS waits for authentic provisional records, not accepted formation; final checked/runtime handoff waits for AST-TYPE acceptance. SurfaceAst storage/logical normalization supplies neither meaning nor proof.
Authority: [§20.5.3](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax),
[§13.10.1](../../../spec/en/13.term_expression.md#13101-capture-reconstruction-and-r2),
[architecture 06](../../architecture/en/06.elaboration_and_core_ir.md#constraints-and-assumptions),
and [architecture 16](../../architecture/en/16.substitution_and_binding.md#constraints-and-assumptions).
Gap: source_drift/test_gap for immutable reflected syntax and complete handoff.
Scope: preserve certified Var/node/opaque/Binder structure and finite context
records through core lowering; keep runtime/contract/returns/function input
correspondence, including C, and source provenance.
Lower structural equality, ordered-child/variable primitives and exact ghost
size without conflating DAG storage with unfolded occurrence count; evaluation
remains with the execution/proof producers, never Core acceptance.
Opaque originals/FV and Binder outer/extended contexts remain intact;
retrieval never alpha-normalizes or renames stored syntax.
Forbidden: synthesized checker records, missing declarations/conditions,
approximate size, ghost runtime flow, new meaning axioms, proof discharge
or moving kernel/MVM/artifact responsibilities into core.
Owner-local invariants/tests: [core_ir.md](../../mizar-core/en/core_ir.md#reflected-syntax-handoff).
Owner details: [terms](../../mizar-core/en/core_ir.md#core-terms),
[binding](../../mizar-core/en/core_ir.md#binders-and-variables),
[lowering](../../mizar-core/en/elaborator.md#step-3-term-and-formula-lowering).
Tests: use the linked owner-local matrix; evaluation/proof acceptance remains downstream.
Existing [logical normalization tests](../../../../crates/mizar-core/src/binder_normalization.rs) retain their separate certificate boundary.
Audit: structural handoff credit grants no meaning/execution/proof acceptance.

Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, and source/document consistency under the canonical protocol.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected owner tests and local-link/parity checks.
Exit: frozen scope satisfied by authentic producer inputs and reviewed tests;
no completion or coverage claim for deferred consumers.
