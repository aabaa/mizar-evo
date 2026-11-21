# Task P25-NATIVE-RUNTIME-HOST: staged native LSP host
Canonical language: English; [Japanese pointer](../ja/P25-NATIVE-RUNTIME-HOST.md).
Status: frozen host scope. Tier: full; new executable consumer/lifecycle integration departs from the producer pattern.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md#task-index); dependency: [producer](P25-NATIVE-RUNTIME-PRODUCER.md). Consumer: native editors.
Purpose: connect G14 Q1–Q3 runtime inputs: running-image ToolchainInfo, frontend-only normal workspace dependencies, cache disabled.

## Authority and boundaries

Authority: spec23 [configuration](../../../spec/en/23.package_management_and_build_system.md#231-manifest-file-format-mizarpkg), [reproducibility](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility), [LSP](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support); architecture22 [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence).
Owners: Driver [runtime inputs](../../mizar-driver/en/request.md#native-runtime-context-definition), [CLI](../../mizar-driver/en/cli.md#stdio-lifecycle-host); LSP [server](../../mizar-lsp/en/server.md), [bridge](../../mizar-lsp/en/build_bridge.md#admission-and-deadlines), [execution](../../mizar-lsp/en/execution_seam.md#implemented-a-package), [diagnostic wire](../../mizar-lsp/en/server.md#native-frontend-diagnostic-wire).
Gap: executable source_gap/test_gap; runtime selection remains producer-owned. Coverage and semantic/publication deferrals stay unchanged.
Write scope: LSP main.rs/server.rs, necessary existing bridge internals, stdio.rs and ownership-only serve-test adaptations; this pair, paired LSP plan index and paired server/bridge/CLI boundary wording.
Main installs the lane-exclusive nonterminal bridge, preserving admission/debounce/generation/supersede/shutdown/drain rules. Keep one framing parser. Existing serve consumes owned Send + 'static BufRead; bounded reader channel permits deadline progress with stdin open. Unconfigured serve remains protocol-only. No new public API/type/field/dependency.
At flush, decode only accepted explicit initialize rootUri; no cwd/settings fallback. Invoke capture_native_lsp; carry genuine request/plan inputs into existing OpenBuffer capture for every open URI; any unmappable URI fails the admission. Release temporary Disk capture lease on success/error. Reconstruct changed metadata on the next accepted document event; retain original raw/version/incarnation guards.
Compose StageConfig::frontend with empty artifact roots and unavailable cache defaults. Progress existing work; deliver original completed frontend diagnostic indices through shared eligibility/range/wire checks without resubmission. Revalidate currentness/incarnation before each frame. Acquisition/capture failure consumes admission, leaves protocol live and diagnostics unchanged; no fabricated context/clearing.
Internal choices: owned input/channel over duplicate framing; existing overlay composition over a new context carrier.
Forbidden: batch bootstrap, Watch/disk scheduling, settings interpretation, registry fetching, Proof/replay/artifact/cache activation, coherent full snapshot/semantic-feature activation, release/pin/backend authentication, policy mapping, spec/.miz/expectation/trace/audit changes, owner-gated scope, AGENTS/protocol edits.

## Test matrix and gates

Six cases (cap six), genuine executable/native files; establish one positive fixture before expanding:
1. Initialize/open with stdin open: editor-text frontend error conflicting with Disk, independent UTF-16 range/version and framed-only stdout; clean edit clears it.
2. Multiple members/transitive normal dependencies/open documents: per-document diagnostics; dev metadata inactive. Reuse producer identity/hash oracles.
3. Edit burst, ignored/stale/invalid traffic, save, close/reopen and configuration change before the next accepted edit: final-version cohort and reconstructed inputs; save/configuration traffic admits nothing. Reuse deterministic bridge timing/cancellation matrices.
4. Null/invalid/non-file root, unavailable manifest/lock/source, unmappable open URI and registry dependency: no fallback/diagnostics; protocol remains responsive; later viable admission recovers.
5. Shutdown/early exit with pending/running work: terminal suppression, exit/drain; retain genuine bridge cancellation controls.
6. Malformed/truncated/EOF frames, argument/build-unavailable and output failure: retain existing process/serve assertions except input ownership; no panic/nonprotocol stdout.
Reviews: specification/test-intent before freeze; independent sufficiency, precision, volume/scope, source/EN-JA consistency and quality assessment. Extra cases name authority.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; both lint_policy suites (links/fragments/pairing/ledger); cargo test -p mizar-lsp including stdio E2E. CARGO_BUILD_JOBS=4. Unchanged producer/codec/cache/proof heavy matrices defer to integration cadence; full cargo test only on named trigger.
Freeze: both lint_policy suites and git diff --check. Exit: freeze then host implementation locally committed as one changed task; no push, full editor-entry completion or coverage claim.
