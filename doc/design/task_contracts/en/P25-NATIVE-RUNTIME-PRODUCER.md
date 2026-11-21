# Task P25-NATIVE-RUNTIME-PRODUCER: staged native context producer
Canonical language: English; [Japanese pointer](../ja/P25-NATIVE-RUNTIME-PRODUCER.md).
Status: frozen producer scope. Tier: full.
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index); dependency: [Build plan](../../mizar-build/en/00.crate_plan.md#task-index). Consumer: later native LSP host.
Purpose: produce genuine runtime-context inputs after G14 approval of Q1–Q3; no host activation.

## Authority and boundaries

Authority: spec23 [configuration](../../../spec/en/23.package_management_and_build_system.md#231-manifest-file-format-mizarpkg), [reproducibility](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility), [LSP](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support); architecture22 [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence).
Definition: [P25](P25-NATIVE-RUNTIME-CONTEXT.md); owner invariants: Driver [runtime context](../../mizar-driver/en/request.md#native-runtime-context-definition), Build [workspace projection](../../mizar-build/en/planner.md#native-workspace-only-context-inputs).
Gap: `design_gap`/`test_gap` for producing native inputs; later host remains an `external_dependency_gap`.
Write scope: Driver request.rs and source_load tests; paired request.md/planner.md boundary wording, this contract pair and paired owner-plan index links. Reuse existing Build acquisition, planning, layout/index and hash APIs; no new Build semantics.
Add one public `BuildSession::capture_native_lsp` constructor accepting root, lane, generation, LspRequest, allocator and snapshot registry; return existing captured session and DriverSubmitInput with native StaticSourceLayout. Acquire runtime/workspace inputs internally; accept no supplied plan, digest, source list or artifact metadata. Error transport is Result with String, preserving existing acquisition/validation messages and no new public diagnostic.
Use versioned running-image ToolchainInfo, normal selection, full lock/config content hashes and all discovered workspace source inputs. Refuse every registry plan row before layout/index construction. Capture once with existing guards; retain genuine tuples, package-local settings and inactive dev metadata. Returned submission defaults leave cache unavailable; frontend-only composition remains StageConfig::frontend with empty artifact roots at the later caller boundary.
No empty-context fallback. Unsupported runtime or unavailable acquisition/validation/layout/capture fails. Reconstruct after changed inputs; equality does not prove currentness. Existing explicit richer profiles and same-request guards remain unchanged.
Forbidden: host/main/stdio changes, cache activation, Proof/replay/artifact execution, release/pin/backend authentication, registry fetching, config-to-policy mapping, language/spec/test-intent/diagnostic/acceptance/trace/audit changes, AGENTS/protocol edits.

## Test matrix and gates

Six cases (cap six), real native files and independent existing parser/hash oracles:
1. Three workspace members with transitive normal edges: all sources captured, exact runtime/plan/lock/config association, genuine empty artifacts and frontend-only E2E with cache disabled.
2. Configuration-only reconstruction: changed content hash/snapshot; package-local defaults and computation_replay preserved without replay activation.
3. Workspace dev-only metadata and valid unused registry lock entries: admitted, dev edges inactive, full lock identity retained.
4. Viable registry dependency, including inactive dev registry row: refuse before capture; never manufacture empty artifacts.
5. Missing/invalid manifest, lock, member/source inputs and unsupported runtime: unavailable via existing errors; no fabricated context. Reuse running-image platform refusal and native acquisition matrices where unchanged.
6. Retained capture versus later disk mutation and foreign plan/runtime/config/lock association: original tuples remain, existing independent guards refuse mismatches; no currentness or publication claim.
Existing matrices: source_load/native_workspace.rs, native_toolchain.rs and explicit.rs; Build planner tests retain codec/validation oracles. No .miz or expectation edits.
Reviews: specification/test-intent before freeze; independent sufficiency, precision, volume/scope and final source/EN-JA consistency; full hard gates and quality scoring.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; both lint_policy suites (links/fragments/pairing/ledger); cargo test -p mizar-driver --lib; cargo test -p mizar-driver --test source_load; cargo test -p mizar-build --lib; producer frontend E2E in source_load. CARGO_BUILD_JOBS=4. Full cargo test only on named cadence trigger.
Freeze runs both lint_policy suites and git diff --check before commit. Exit: freeze then producer implementation locally committed as one changed task; host remains later. No push.
