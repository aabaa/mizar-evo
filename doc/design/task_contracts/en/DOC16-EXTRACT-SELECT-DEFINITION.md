# Task DOC16-EXTRACT-SELECT-DEFINITION: extraction selection proposal
Canonical language: English; [Japanese pointer](../ja/DOC16-EXTRACT-SELECT-DEFINITION.md).
Status: frozen definition unit; new owner decisions remain unselected. Tier: full, documentation only.
Primary owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [Doc16](../../mizar-doc/en/todo.md#extraction-strand); consumers: Doc17 selection, Doc18 runtime IR and Doc21 backend. Freeze plus paired proposal is one changed task.
Purpose: define a non-normative proposal for the verified executable subset, selection rules and explicit rejection of unsupported runtime constructs, without graceful degradation.

## Authority And Readiness
Authorities: spec20 [ghost separation](../../../spec/en/20.algorithm_and_verification.md#2013-variable-declarations-var-const), [Pick](../../../spec/en/20.algorithm_and_verification.md#203-non-determinism-pick), [extraction](../../../spec/en/20.algorithm_and_verification.md#2010-code-extraction); architecture13 [subset](../../architecture/en/13.documentation_and_extraction.md#extraction-uses-verified-executable-subset), [selection](../../architecture/en/13.documentation_and_extraction.md#step-1-select-extractable-items); internal05 [builder](../../internal/en/05.documentation_extraction.md#extraction-builder), [pipeline](../../internal/en/05.documentation_extraction.md#extraction-pipeline) and [errors](../../internal/en/05.documentation_extraction.md#extraction-errors).
Completed Doc2's [reader](../../mizar-doc/en/artifact_reader.md) defines stable published-input loading, not executable projection or extraction readiness. These missing definitions are spec_gap prerequisites for later implementation, not permission to infer semantics from source or raw IR.

## Scope And Ownership
Write only this paired contract, paired Doc 00.crate_plan.md task-index links and `doc/design/mizar-doc/{en,ja}/extract_select.md`. Japanese contract is a pointer; proposal has full EN/JA parity.
Separate inherited constraints from candidate refinements. Preserve terminating-algorithm scope, proof/artifact authority, ghost isolation, target support, source/projection availability, deterministic identity and the five existing classification meanings.
Expose genuinely new choices as unselected owner Q-decisions: root combination/visibility, dependency closure, status overlap/reporting, policy/capability evidence and failure profiles. Do not choose a first backend or define a runtime representation.
Preserve internal05's affected-item refusal and all-roots profile boundary: no partial code for a refused item; request/output policy details remain owner-gated. No proof rechecking, new acceptance policy, source fallback or fabricated missing metadata.
Forbidden: normative/spec adoption, allocation of diagnostics or APIs, Rust/Cargo/test/fixture/expectation/trace edits, implementation, AGENTS/protocol edits, approved comments Q1–Q8 option-2 alternatives or broader milestone closure. Doc7 remains parked; Doc8 remains dependent.
Runtime IR/erasure implementation, backend capability catalog, first target, generated output and atomic publication belong to later owners. No audit/coverage change or implementation credit.

## Documentation Acceptance Matrix
No executable tests change; six cases (cap six) are reviewed as documentation obligations, not runtime coverage:
1. Export/config/CLI roots and non-algorithm or non-terminating candidates preserve higher-authority limits; unresolved combination and visibility rules stay owner questions.
2. Verified executable eligibility requires matching source/dependency artifacts, accepted required obligations, permitted external attestations and erasable ghost/proof state; reader success alone grants none of these.
3. Missing executable/source-map/dependency/mapping inputs and stale/schema failures preserve lower ownership; classification conflicts and missing representation remain explicit prerequisites.
4. Runtime operations, non-ghost Pick and ghost escape get explicit rejection boundaries, with no guessed value, skipped operation or partial emitted item; ghost-only Pick retains its proof obligation before erasure.
5. Dependency selection, target/configuration determinism, diagnostics and item-versus-all-roots behavior separate fixed rules from unselected policy choices; later RuntimeIr/backend/publisher work stays deferred.
6. Paired proposal, Q-decision/adoption gates, links/fragments and budgets hold; no normative adoption, allocation, source/test change or extraction completion is claimed.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope and final source/document consistency. Repeat specification only if frozen scope changes.
Verification for both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (links/fragments/pairing/volume); git diff --check. Documentation only: fmt/Clippy/runtime/e2e unrun. Full cargo test is required on the final proposal tree by the tenth-changed-task cadence; no push.
Exit: reviewed freeze and paired proposal committed locally, with new choices unselected and later adoption/implementation owner-gated.
