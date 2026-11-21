# Task P25-NATIVE-RUNTIME-CONTEXT: native runtime-context definition
Canonical language: English; [Japanese pointer](../ja/P25-NATIVE-RUNTIME-CONTEXT.md).
Status: frozen definition unit; G14 approval remains required before code. Tier: full, documentation only.
Owners: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index), [Build plan](../../mizar-build/en/00.crate_plan.md#task-index). Consumers: later staged producer, native LSP host and build bootstrap.
Purpose: define native toolchain identity, package configuration content identity, default LSP profile and workspace-only dependency inputs. Definition has no implementation dependency.

## Authority and scope

Authority: spec23 [toolchain and configuration](../../../spec/en/23.package_management_and_build_system.md#231-manifest-file-format-mizarpkg), [reproducibility](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility), [LSP](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support); architecture22 [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence).
Existing owners: Driver [snapshot capture](../../mizar-driver/en/request.md#snapshot-capture), [running-image acquisition](../../mizar-driver/en/frontend_adapter.md#native-disk-cache-preparation), [active-run binding](../../mizar-driver/en/driver.md#active-run-sourceload-binding); Build [configuration content](../../mizar-build/en/planner.md#current-package-configuration-content-fingerprint), [toolchain boundary](../../mizar-build/en/planner.md#toolchain-constraint-boundary), [computation policy](../../mizar-build/en/planner.md#computation-policy-boundary).
Gap: `design_gap` in native request-context selection; existing carriers and hashes do not select authentic inputs. Producer and executable-host work remain `external_dependency_gap`/`test_gap` for later units.
Write scope: this contract pair and paired owner-plan index links, then paired Driver request.md and Build planner.md definition sections. Driver owns runtime/profile selection and same-request association; Build owns validated workspace projection and existing content codecs.
Define a bounded default without new language, proof, diagnostic, release-selection or compatibility policy. Distinguish package-content identity from effective Proof policy and runtime inputs; unavailable provenance stays unavailable. Preserve existing approved active-run choices and package-local computation settings.
Record new internal design Q-decisions with alternatives and recommendations for G14; the automatic definition-unit rule permits completing this definition without selecting owner-gated code scope.
Forbidden: code, tests, fixtures, expectations, spec, trace/coverage/audit changes; producer/API implementation, cache or host activation, language-intent changes, release/backend authentication, policy mapping/precedence, AGENTS/protocol edits or unrelated tasks. Existing coverage and deferrals remain unchanged.

## Documentation acceptance matrix

Six cases (cap six); no executable cases change:
1. Actual running-image provenance versus caller label, recorded version and authenticated release: define exact bounded identity source/encoding and unavailable behavior without pin/W0306 claims.
2. Actual validated plan's independent package verifier/build aggregates: use workspace_config_content_hash unchanged; preserve package association, defaults and computation_replay, separate lock/toolchain/policy/invocation identities.
3. Default LSP profile: name the existing stage composition and dependency selection; no implicit proof, replay, artifact or cache activation and no verified-build claim from frontend output.
4. Workspace-only normal dependency closure: validated members, exact lock assignments and source inputs; distinguish genuine empty artifact references from missing registry/artifact metadata, including transitive and dev-only cases.
5. Reconfiguration, foreign plan/capture and unavailable inputs: reconstruct immutable same-request context; no fresh-file, currentness, compatibility or publication authority from equal digests.
6. G14 and Wave 4 producer/host prerequisites remain explicit; paired owners, links, budgets and existing specification/test intent agree.
Existing test anchors: Build planner/tests.rs configuration-content and toolchain metadata matrices; Driver tests/source_load/{explicit,native_toolchain}.rs native association; LSP tests/stdio.rs unavailable executable bootstrap. These constrain future implementation, not native-host coverage today.
Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and final source/EN-JA consistency over the whole unit. Extra cases require an authority clause or .miz test.
Each commit: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (including local links/fragments and documentation ledger); git diff --check.
Documentation only: fmt/Clippy/runtime/E2E unrun; full cargo test only on a cadence trigger. Contract cap: 60 lines/660 words; whole-unit documentation cap: 200 added lines.
Exit: reviewed freeze then owner definition committed locally as one changed task; pending G14 choices handed to Muse. No implementation or push.
