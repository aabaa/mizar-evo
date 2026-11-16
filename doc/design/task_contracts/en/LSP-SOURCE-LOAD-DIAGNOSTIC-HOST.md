# Task LSP-SOURCE-LOAD-DIAGNOSTIC-HOST: genuine failed-source retention
Canonical language: English; [Japanese pointer](../ja/LSP-SOURCE-LOAD-DIAGNOSTIC-HOST.md).
Status: frozen implementation contract. Tier: full; source-load admission adds a public seam and fails the pattern criterion.
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md), [LSP todo](../../mizar-lsp/en/todo.md). Consumer: separately dispatched non-range presentation, then Task41.

## Authority and readiness

Authority: spec [§22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display), [source-loading locations](../../mizar-diagnostics/en/failure_record.md#source-loading-primary-locations), [native capture](../../mizar-driver/en/request.md#native-source-capture), [host ownership](../../mizar-lsp/en/ranged_diagnostic_host.md#admission-and-ownership), [currentness](../../mizar-lsp/en/snapshot.md#currentness-and-retention).
[Presentation handoff](../../mizar-lsp/en/non_range_diagnostics.md#conditional-validation-and-implementation-handoff) requires genuine source-load capture/retention/host admission independently of wire decisions. This dispatch authorizes the prerequisite; approved Q1–Q5 A belongs to later presentation.
Gap: `source_drift` at all-or-fail capture and ranged-only admission; existing producer, identity and guard remain authoritative. No language/spec/expectation or §23.6/SCA-003 credit change.

## Frozen boundary

Extend an original protected native capture with additional genuine Disk source-load failures. Attempt each supplied request once through the real native loader; retain its exact SourceInput and SourceLoadError with the original session. Require the same package/loader root as an original captured source; validate original state and reject unsupported inputs, duplicate requests and unexpected successful loads. No caller-supplied draft/index/error is evidence.
The successful original native cohort remains unchanged, snapshot, SourceVersions, maps and lease. Supplemental failed requests create no SourceId, hash, version or source text; replay and submission never reopen them. The existing nonempty-cohort native currentness guard stays unchanged. Entirely failed initial captures remain unavailable; defining their currentness is outside this unit.
Submit retained failures only through a terminal Failed diagnostic route, producing canonical existing E0600–E0603 drafts and aggregation under the original snapshot context. No source output, scheduler work, manifest, full SnapshotPublished event or semantic/artifact authority is synthesized. Preserve lane/request generation, cancellation, publisher and stale-replay guards; reject reuse/foreign/obsolete inputs as existing ownership requires.
The originating LSP ticket consumes that session and admits only its authentic returned immutable index while its original capture Arc/editor incarnation and native guard are current. SourceLoad package/path records need no failed-source text. Real secondary anchors and ranged records retain original-source validation; unknown or invalid records refuse the entire admission and preserve the prior selection.
Retain the original capture, exact canonical index, distinct activation identity and producer store with the diagnostic selection/readers. Existing retained stale reads, replacement and equal-snapshot isolation remain unchanged. Full snapshot accessors retain their success-only meaning.
Writes: Driver capture/submission/native source producer and tests; LSP originating capture/admission and tests; paired owner boundaries. Lower-stage Session/Build loading, diagnostics identity/aggregation and language behavior remain unchanged.
Forbidden: wire presentation/conversion/capability changes, relaxing whole-batch refusal, owner B alternatives, Task41 completion, AGENTS/protocol edits, spec/fixtures/expectation/coverage changes. Approved presentation is a separate unit.

## Test matrix and exit

Cap six cases; subassertions and existing controls allowed:
1. Real missing/unreadable and invalid-UTF8 Disk attempts, plus supported generic/outside-root failures: exact canonical code/detail/package/path survives capture, deletion/recreation, submission and retained reads without failed text or fabricated coordinates.
2. Same path in two captured packages and multiple failures: preserve identities, canonical order/deduplication and exact payload; real retained anchors remain validated. No synthetic anchor producer required.
3. Unexpected success, duplicate/foreign/unsupported descriptors, ordinary capture failure and caller-built indexes cannot create an admitted failed-source publication; prior selection remains intact.
4. Genuine terminal failure retains original native guard; nonterminal/reused/obsolete sessions, publisher replacement, stale request and terminal Server lifecycle cannot grant current admission. No outputs, full publication or metadata authority.
5. Version-only/normalization-equivalent edit, identical-tuple reopen and equal-snapshot replacement retain distinct original associations; old readers retain original records/maps/leases and explicit stale status until release.
6. Native non-range-only host admission is positive evidence; existing ranged delivery, full snapshot, CLI/artifact, cancellation and whole-batch SourceLoad conversion refusal remain controls. No wire delivery credit is claimed.
Reviews: independent specification/test intent before freeze; sufficiency, precision, volume/scope and consistency/final across both commits. Extra cases require authority.
Verification: freeze runs both lint_policy suites and local links/fragments. Implementation runs fmt, all-target/all-feature Clippy with warnings denied, both lint_policy suites, Driver source-load/request/lifecycle and complete LSP owners including native stdio E2E. Changed snapshot/replay obligations run in those groups; unrelated cache/proof/kernel heavy matrices defer to periodic/integration cadence. Full cargo test follows cadence, with no push in this unit.
Exit: original native guard admits genuine retained source-load diagnostics independently of unchanged wire refusal; reviews/checks pass; commits remain local.
