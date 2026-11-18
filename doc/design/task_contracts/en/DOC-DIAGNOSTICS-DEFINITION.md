# Task DOC-DIAGNOSTICS-DEFINITION: Documentation diagnostics proposal
Canonical language: English; [Japanese pointer](../ja/DOC-DIAGNOSTICS-DEFINITION.md).
Status: frozen definition unit; owner decisions remain unselected. Tier: full, documentation only.
Primary owner: [diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md); consumer: [Doc plan](../../mizar-doc/en/00.crate_plan.md), Doc7 attachment and later Doc8. Freeze plus paired proposal is one changed task.
Purpose: propose the canonical Documentation family, code range and Doc7 attachment descriptors that a later approved specification update could adopt.

## Authority And Readiness
Authorities: [comments attachment outcomes](../../mizar-doc/en/comments.md#residual-association-definitions) and [diagnostics](../../mizar-doc/en/comments.md#diagnostics-and-determinism); registry [gaps](../../mizar-diagnostics/en/registry.md#deferred-code-space-gaps), [allocation](../../mizar-diagnostics/en/registry.md#allocation-rules), [retirement](../../mizar-diagnostics/en/registry.md#retirement-rules) and [metadata](../../mizar-diagnostics/en/registry.md#descriptor-metadata); internal05 [diagnostics](../../internal/en/05.documentation_extraction.md#diagnostics); spec22 [codes](../../../spec/en/22.error_handling_and_diagnostics.md#2211-error-code-system-e-series), [locations](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display) and [reference](../../../spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference).
The registry's prohibition on allocating/publishing Documentation codes is a spec_gap, not permission to invent local codes. Existing [PhaseFamily and descriptor validation](../../../../crates/mizar-diagnostics/src/registry.rs) expose the representation boundary; the proposal cannot relax it.
Preserve approved comments Q1–Q4 and Q5–Q8 option ①. Approval of those comment choices does not allocate codes. Doc7 remains parked until this prerequisite receives owner approval and subsequent specification/registry prerequisites are complete; Doc8 remains dependent.

## Scope And Ownership
Write only this paired contract, paired diagnostics/00.crate_plan.md task-index links, and `doc/design/mizar-diagnostics/{en,ja}/documentation_definition.md`. Japanese contract is a pointer; proposal has full EN/JA parity.
The proposal owns candidate definitions and explicit alternative Q-decisions for family/PhaseFamily shape, numeric range, descriptor granularity, stable meaning keys, metadata and normative spec22-versus-registry ownership. Derive default warnings from approved Q4; expose any genuinely new severity/promotion choice without adopting it. Address source-range availability and deterministic/profile boundaries without inventing locations or warning deduplication.
The five Doc7 attachment conditions are the entire descriptor scope. Missing-source/comment lints, Markdown/tags, links, rendering and Extraction allocations remain deferred. No placeholders or reservation becomes normative here.
Forbidden: Rust/Cargo/test/fixture/expectation/trace/spec edits, registry allocation or adoption, Doc7/Doc8 implementation, AGENTS/protocol edits, approved option-2 alternatives, lower-reader reinterpretation, proof/publication authority changes or broader milestone closure.
Audit/coverage and lower-stage ownership remain unchanged; no audit edit or implementation credit.

## Documentation Acceptance Matrix
No executable tests change. Existing [registry tests](../../../../crates/mizar-diagnostics/tests/registry.rs) and [record tests](../../../../crates/mizar-diagnostics/tests/failure_record.rs) constrain future adoption, not current Doc7 coverage. Six cases (cap six):
1. Each of orphan, unresolved-target, ambiguous-attachment, unverified-target and unresolved-attachment has a proposed descriptor mapping and retained meaning; granularity alternatives stay unselected.
2. Candidate ranges do not overlap current allocations; code/family/default-severity, permanent identity and retirement constraints remain explicit.
3. Complete descriptor metadata and meaning-key alternatives support later registry validation; no current allocation or publication is claimed.
4. Available block/candidate ranges versus unavailable association coordinates expose representability prerequisites without fabricated spans or reclassified lower-reader errors.
5. Profile promotion, private-doc omission, ordering/provenance and approved Q1–Q8 option ① remain intact; later diagnostics and Extraction stay deferred.
6. EN/JA parity, links/fragments and budgets hold; Q-decisions identify the normative update owner and later adoption gate, leaving Doc7/Doc8 blocked.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency. Repeat specification only if frozen scope changes.
Verification for both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (including links/fragments/pairing/volume); git diff --check. Documentation only: fmt/Clippy/runtime/e2e unrun; full cargo test only on a cadence trigger, absent for this local task below ten outstanding tasks.
Exit: reviewed freeze and paired proposal committed locally; all new semantic choices remain unselected for owner approval. No implementation or push.
