# Task DOC5-DOC-BUILD-PLANNING-INDEX: published documentation planning
Canonical language: English; [Japanese pointer](../ja/DOC5-DOC-BUILD-PLANNING-INDEX.md).
Status: frozen implementation scope. Tier: full; freeze and implementation are one changed task.
Owner: [Doc plan](../../mizar-doc/en/00.crate_plan.md), [task5](../../mizar-doc/en/todo.md#inputs); consumers: Doc6–15, Doc23/24. Dependencies: tasks3/4 and published readers.

## Authority And Decisions
Authority: [approved Doc4 definitions and six-case matrix](../../mizar-doc/en/doc_build.md#full-build-and-index-inputs), spec [23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format), [24](../../../spec/en/24.documentation_generation.md), architecture13 [interfaces](../../architecture/en/13.documentation_and_extraction.md#interface-definitions), internal05 [requests](../../internal/en/05.documentation_extraction.md#phase-16-request).
Owner approval selects Q1–Q3 option ①: new plan/index version is required, source_file and item range are required, and caller expected publication hash plus build snapshot authenticate owning-package/current-workspace binding; mismatch returns no plan. Stored optional identities remain unchanged; no synthesized version/path/range.
Missing planning/index APIs are source_gap; the resolved optionality conflicts are design_drift repaired in the paired build owner. Q4 comment sourcing remains Doc6-owned.

## Scope And Ownership
Implement in `crates/mizar-doc/src/doc_build.rs`; append genuine-publication regressions in `crates/mizar-test/src/runner/tests/doc.rs` (shared fixture extension only if needed). Update this paired contract, paired doc_build owner, crate-plan backlinks and task4/5 TODO boundaries only.
Add request/profile, per-publication plan and unresolved documentation index inputs. Retain complete published manifest/path/hash, module entries, exports, optional summaries and explicit dependency index provenance. Nest the index in its plan to avoid duplicate item storage. Required versions use existing stored strings, never parsed version ordering.
Authenticate registry-retained current snapshot/request/root; bind selected package metadata to manifest and each module package tuple; bind module path/edition/source path/hash and artifact toolchain/lock/config provenance to snapshot. Use existing reader validation first; preserve its errors under a wrapper. No partial return on any refusal.
Canonical package ordering uses full package tuple/publication hash; module and export order use existing full identity and exact FQN/origin. Dependency references sort by complete provenance; caller-supplied semantic import vectors retain order and missing state. Label queries retain exact-record None, Some(empty), and every candidate through the index.
Keep documentation/extraction switches and warning policies independent; disabled documentation produces no documentation plans, extraction remains a delegated request. Output/source-bundle paths are retained inputs, not written or proof of currentness.
Forbidden: spec/test-intent/trace changes, protocol/AGENTS edits, scheduler/service/frontend producer changes, source fallback, comment sourcing/attachment, rendering/extraction/publication, anchor/link/UsedBy completion, evidence acceptance or proof/cache/reuse authority. Broader Doc4/5 and crate exit remain open; no audit ownership/coverage change.

## Review And Verification Matrix
Six genuine-publication cases (cap six; subcases exercise the same obligations):
1. All four consumer switches preserve independent policies; disabled consumers do no work and verification/publication bytes remain unchanged.
2. Two packages with colliding module paths/FQNs, an empty module and dependency references retain complete typed metadata and full publication identity without merging.
3. Reversed package/module/dependency enumeration and different worker execution yield equal canonical plans/index inputs while import vectors retain semantic order.
4. Missing package/module version or source metadata refuses without defaults; absent versus empty summary survives successful planning. Required ranges retain exact stored values.
5. Bad manifest/hash/schema/path/stale artifact and package/root/snapshot/source/provenance mismatches propagate refusals and yield no plan or partial index. Reuse lower exhaustive reader matrices; representative wrapper cases suffice.
6. Missing/forged labels, zero/multiple candidates and missing dependency/import/citation/comment metadata retain all candidate fields and absence distinctions without fabricated downstream results.
Reviews: independent specification before freeze, test sufficiency, implementation precision, volume/scope and final consistency under the full gates.
Freeze checks: both AGENTS lint_policy suites (including links/fragments/ratchet) and git diff --check. Implementation checks: AGENTS fmt/Clippy/both lint suites, mizar-doc tests, complete doc_* owner tests and representative proof-verification end-to-end. Run full cargo test only on a named cadence trigger; unaffected Resolver/cache/evidence heavy matrices defer to periodic/integration cadence.
Exit: approved bounded planning/index APIs and six-case regression matrix reviewed, checked and committed locally; no push.
