# Task DRIVER-NATIVE-CONFIG-CONTENT-BINDING: explicit native package configuration association
Canonical language: English; [Japanese pointer](../ja/DRIVER-NATIVE-CONFIG-CONTENT-BINDING.md).
Status: implemented for explicit caller package-content association. Tier: full. Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md), task 34.
Authority: spec §23.1/23.4 and architecture22 captured-input boundary; native runtime caller prerequisite.
Dependencies: protected native capture, SourceLoadInputs and existing Build workspace_config_content_hash; full active runtime context remains absent.
Gap: source_drift/test_gap; supplied config identity has no explicit typed-plan aggregate content comparison.
Scope: SourceLoadInputs::validate_native_config_content(self) -> Result<(), String> in existing frontend_adapter.rs; [sole owner](../../mizar-driver/en/frontend_adapter.md#native-config-content-association) defines guards and existing content profile.
This caller-only check elects the existing package-config profile explicitly; no new carrier/profile/validation/currentness authority or implicit native/cache/submit guard.
Tests: append actual acquired configuration with Disk/Open resident/blob SourceLoad/Frontend/own-parent/import-discovery consumers, preserving real outputs/maps/diagnostics/NoKey and inputs.
Controls: coherent plan-only verifier/build config changes and independently captured changed digests mismatch; missing/non-native/foreign capture, borrowed snapshot and toolchain guards reject; independent aggregate reorder agrees. Legacy arbitrary supplied hash semantics and old tests remain unchanged.
Forbidden: new type/field/file/helper/adapter/hash primitive, metadata/source reread, blanket legacy comparison, aggregate validation or full config/pin/toolchain/currentness claims, fake Accepted/Hit, protected spec/.miz/expectation/trace/policy/ledger edits.
Audit: chapter23 remains partial; full policy/backend/seed/budget/invocation/replay context, registry metadata provenance, freshness/root/lane/lease/retention/scheduler/host, later services/artifacts/cache and STEP6C/Steps6–8 remain open.
Reviews: independent specification/tests/code/volume/consistency, scoped gates/quality; parent fmt, full Clippy -D warnings and cargo test.
Exit: explicit callers compare genuine protected captured context with existing typed aggregate content or receive String Err without mutation/I/O/output/publication; equality authenticates no files or runtime.
