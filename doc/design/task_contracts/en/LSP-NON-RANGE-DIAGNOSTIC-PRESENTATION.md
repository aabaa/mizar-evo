# Task LSP-NON-RANGE-DIAGNOSTIC-PRESENTATION: approved v1 delivery
Canonical language: English; [Japanese pointer](../ja/LSP-NON-RANGE-DIAGNOSTIC-PRESENTATION.md).
Status: frozen implementation contract. Tier: full; new diagnostic wire boundary/public seam.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md). Consumer: Task41 delivery evidence, separately dispatched.

## Authority and readiness

Authority: spec [§22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display), [source-load identity](../../mizar-diagnostics/en/failure_record.md#source-loading-primary-locations), [conversion](../../mizar-lsp/en/diagnostics.md#range-conversion), [currentness](../../mizar-lsp/en/snapshot.md#currentness-and-retention).
Owner approval on 2026-11-16 selects Q1–Q5 option A in the [proposal](../../mizar-lsp/en/non_range_diagnostics.md#open-owner-decisions); B alternatives are forbidden. The [genuine host prerequisite](LSP-SOURCE-LOAD-DIAGNOSTIC-HOST.md) supplies original failed-source records without failed-source text.
Gap: `source_drift` between approved presentation and whole-batch SourceLoad refusal. No language, public diagnostic code, expectation, trace or §23.6/SCA-003 credit changes.

## Frozen boundary

Implement `mizar/publishNonRangeDiagnostics` with the approved compact v1 envelope, exact underlying package/path strings, existing display fields/handle and validated real secondary locations. Keep canonical non-range subsequence and retained payload identity. Unknown variants, invalid/stale records or anchors fail explicitly; no invented coordinates, versions or provenance.
Enable per connection only for `capabilities.experimental.mizarNonRangeDiagnostics: {version: 1}`; advertise supported version 1. Missing/malformed/unsupported capability rejects non-range batches before writes and preserves ranged-only compatibility.
Each supported current publication emits unchanged ranged URI-order frames then one whole-set extension replacement, including clean empty sets. Use the existing non-reused publication identity as opaque selection token, distinguishing equal-snapshot replacements. No new public record codec, query, authority or persistent identity.
Accepted editor invalidation queues a null-selection empty replacement for the serving loop or explicit diagnostic writer; externally discovered guard/selection loss clears on delivery validation. Terminal connections discard delivery state. Clearing must not erase a newer selection. Build/host integration remains explicit; no new scheduling or automatic build route.
Preflight capability, selection, all records/anchors and serialized frames before writes. Recheck original publication/currentness before each current frame; currentness loss stops writes. Propagate conversion/I/O errors; no rollback, acknowledgment, retry or atomic-display promise.
Retain original native guards, admission, source maps and stale-reader behavior. CLI/artifacts, overlay isolation, full-snapshot authority and Task41 completion remain unchanged.

## Artifacts and matrix

Affected: `crates/mizar-lsp/src/{diagnostics,server}.rs`, existing LSP tests, paired presentation/conversion/host owners. No Driver/Diagnostics producer change. Preserve invalid/unknown refusal intent in `conversion_fails_the_whole_batch_for_source_load_unknown_and_invalid_spans`.
The [six approved validation cases](../../mizar-lsp/en/non_range_diagnostics.md#conditional-validation-and-implementation-handoff) are the complete matrix; cap six grouped cases:
1. Genuine native E0600–E0603 failures without primary text → exact package/path/kind, handles and display fields on wire.
2. Equal paths/messages across packages and repeated real anchors → distinct ordered records, original retained payload; invalid anchors fail.
3. Supported/missing/malformed/unknown capabilities → negotiated delivery or zero-frame refusal; ranged-only compatibility.
4. Clean/package replacement, invalidation, equal-snapshot replacement and terminal states → whole-set replacement/guarded clearing, no revived records.
5. Mixed/non-range-only, invalid/unknown/stale records and anchors → whole-batch preflight with zero frames on failure.
6. Genuine framed delivery, mid-batch currentness/replacement and failing writer → exact payload/order, explicit errors, unchanged ranged/retained/CLI/artifact/overlay controls.

## Reviews and verification

Independent specification/test-intent, sufficiency, precision, volume/scope and consistency/final reviews; full-tier gates apply to this unit only.
Run fmt, all-target/all-feature Clippy with warnings denied, both lint_policy suites, complete LSP owners/native stdio E2E and documentation/link checks. Changed wire/currentness/replay obligations run there; unaffected Driver/cache/proof/kernel heavy matrices defer to periodic/integration cadence. Full cargo test only on a cadence trigger; no push.
Exit: approved v1 presentation works through genuine host delivery, reviews/checks pass, freeze then implementation committed; Task41 delivery audit remains separate.
