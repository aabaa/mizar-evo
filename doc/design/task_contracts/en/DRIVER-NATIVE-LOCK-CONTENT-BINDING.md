# Task DRIVER-NATIVE-LOCK-CONTENT-BINDING: explicit native lock content association
Canonical language: English; [Japanese pointer](../ja/DRIVER-NATIVE-LOCK-CONTENT-BINDING.md).
Status: implemented for explicit caller content association. Tier: full. Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md), task 33.
Authority: spec §23.1/23.4 and architecture 22 captured-input boundary; consumer native caller integration.
Dependencies: existing protected native capture, SourceLoadInputs and Build lockfile_content_hash; complete runtime context remains absent.
Gap: source_drift/test_gap; supplied captured lock identity has no explicit typed-plan content comparison.
Scope: SourceLoadInputs::validate_native_lock_content(self) -> Result<(), String> in existing frontend_adapter.rs; [sole owner](../../mizar-driver/en/frontend_adapter.md#native-lock-content-association) defines guards and existing hash profile.
This opt-in content check supplies no new carrier, profile, authenticity/currentness receipt or implicit native/cache/submit guard.
Tests: append genuine acquired-lock Disk/Open resident/blob SourceLoad/Frontend and import-discovery consumers; actual content digest, bound inputs, output/maps/diagnostics and NoKey remain intact.
Controls: plan-only typed-lock changes, captured-hash/capture/snapshot/toolchain mismatch fail; reordered typed content agrees. Legacy supplied-hash context variation and existing tests retain their intent.
Forbidden: new type/field/file/helper/adapter/hash primitive, source or metadata reread, blanket legacy-route comparison, pin/release/config completeness claims, cache Hit/accepted evidence, protected spec/.miz/expectation/trace/policy edits.
Audit: chapter23 remains partial; currentness, active policy/root/lane/lease, retention/scheduler, native host, Frontend/later cache/services/artifact and full STEP6C/Steps6–8 remain open.
Reviews: independent specification/tests/code/volume/consistency and scoped gates/quality; parent fmt, full Clippy -D warnings and cargo test.
Exit: explicit callers compare genuine captured context with existing typed-lock content or get String Err without mutation/output/publication; content equality alone authenticates no files or runtime.
