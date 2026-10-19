# Task VC-SOURCE-THEOREM-LOCAL-HANDOFF: ordinary theorem local candidate
Canonical language: English; [Japanese pointer](../ja/VC-SOURCE-THEOREM-LOCAL-HANDOFF.md).
Status: implemented as a bounded original-source handoff; full extraction/acceptance remains open. Tier: full. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md#task-index), task 69. Consumers: later ATP equality/source evidence and Proof checking.
Purpose: a first genuine ordinary source-theorem Kernel-input producer, without acceptance or broader Step 7 closure.
Authority: [spec 16](../../../spec/en/16.theorems_and_proofs.md) §§16.1/16.3.1/16.3.3/16.5; [spec 14](../../../spec/en/14.formulas.md) §§14.4.1/14.5.2; [spec 4](../../../spec/en/04.variables_and_constants.md) §4.6; [spec 21](../../../spec/en/21.source_code_annotation_and_atp.md) §§21.7.1/21.7.5.
Dependencies/readiness: existing theorem Checker/Core skeleton, generic source VC generation and local evidence writer are available. Classification: source_drift; existing handoff producers omit ordinary source-theorem candidates. General ATP equality/acceptance remain deferred.
Sole durable API/profile/replay/manifest/wire/provenance/test owner: [source theorem local handoff](../../mizar-vc/en/kernel_evidence_handoff.md#source-theorem-local-handoff).
Scope: add that exact producer inline in existing crates/mizar-vc/src/kernel_evidence_handoff.rs; append its three genuine-source families within the current private test module in existing crates/mizar-test/src/runner/proof_verification.rs. Preserve existing tests/helpers except the sole fallback explicitly permitted by the owner; retain the final module brace.
Doc scope: paired handoff owners, VC plan indexes/TODOs and this contract; root Step 7/Chapter 21 partial mapping only if integration status changes. No whole-gate or semantic-credit closure.
No new type/field/helper/adapter/source file/dependency or lower-stage change. No spec, existing .miz/expectation, trace/ledger or Cargo edit. Keep old APIs/registration outputs unchanged.
Checker owns source semantics and its seal; Core owns lowering and variable identity; VC owns untrusted candidate projection; Kernel/Proof own fresh acceptance. No source authority, proof acceptance, publication or cache authority is introduced.
Before Rust: freeze owner/test design and resolve independent specification review findings. Parent owns integration, final verification, staging and committing.
Require independent specification, test-sufficiency, implementation, removable-volume, consistency and bounded final reviews; fix findings before exit.
Run narrow relevant tests, cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test and git diff --check; keep logs outside doc/.
Exit: exact supported source produces only the owner's untrusted local candidate, unsupported/drifted inputs refuse, old source intent remains, and actual Kernel parsing/checking does not accept the incomplete v1 evidence.
