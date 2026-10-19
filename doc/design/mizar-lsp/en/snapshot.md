# Editor snapshot and freshness boundary
Canonical language: English; [Japanese companion](../ja/snapshot.md).

## Responsibility

LSP owns immutable editor reads and freshness under [architecture12](../../architecture/en/12.diagnostics_and_lsp.md#lsp-reads-stable-snapshots) and [internal03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot). Session owns source acquisition/maps/leases; IR owns sealed output validity and retention; Driver owns build scheduling/currentness; Artifact owns manifests and published metadata. No compiler or proof authority moves into LSP.

## Source association

Use the actual [originating capture](server.md#originating-native-open-buffer-capture), preserving its original source/map objects and URI/version/raw-text/open-incarnation relations. A BuildSnapshotId identifies normalized build inputs; it does not identify a document version, raw loading maps, open incarnation or publication generation. Accepted version-only and normalization-equivalent raw edits must invalidate the old editor association even when the build id stays equal. An identical-tuple reopen is a new incarnation.

A read retains one immutable association for its whole duration. Replacement makes later reads select the new association; existing reads keep their original values and retention until finished. Never mix maps, diagnostics, versions or metadata from old/new reads. Validate exact current document tuples and originating incarnations before current delivery, using the existing native guards as well; a stored id or retained handle cannot replace those checks.

## Freshness and replacement

Use the existing current/stale_artifact/pending meanings in [architecture12](../../architecture/en/12.diagnostics_and_lsp.md#open-buffers-override-disk-source). Current describes a response computed for the current editor source; it is not proof acceptance. Pending requires an actually scheduled/running verification. Missing producer inputs remain unavailable, rather than implying scheduled work or an empty successful snapshot.

Current syntax-only overlays may exist before full artifacts, but never supply semantic metadata or a coherent full LspSnapshot. On change/close/reopen, withhold old overlay delivery immediately; close removes its open association. Shutdown/exit allow no new current delivery. Do not relabel an old overlay by querying a new lifetime handle.

A retained artifact read may remain stale_artifact after an edit. Preserve its original source association and explicit stale status; suppress unsafe edits and current-buffer delivery whose target cannot be validated. Read-only stale ranges remain associated with their original retained source and explicit stale status; they are never relabeled as current-buffer ranges. Never clear diagnostics merely because producer work or a current map is absent. Task8 owns publication/conversion details; task10 owns scheduling/cancellation.

A full LspSnapshot requires the coherent completed build boundary and original manifest-backed metadata described by [internal03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot-publication). Read the manifest first, retain one complete immutable set, then atomically replace the handler-visible snapshot. ArtifactCommitted alone, Frontend completion and a diagnostic index do not establish that boundary. Driver's current event enum has no coherent SnapshotPublished producer; full snapshot construction/activation stays deferred until that owner supplies the genuine boundary and payloads.

## Currentness and retention

Keep [Driver lane/request publication](../../mizar-driver/en/request.md) separate from [IR publisher currentness](../../mizar-ir/en/publisher.md#snapshot-and-work-unit-validation). SnapshotHandleRegistry registers identity/lineage, not currentness. When build ids differ, an owner-authorized publisher replacement can obsolete the prior id. Equal-id editor edits still invalidate the editor association; do not call IR replace_current_snapshot with equal ids or revive an obsolete id.

Acquire and release LSP-owned Session leases through SnapshotRegistry with the actual PublishedLspSnapshot/OpenBufferOverlay retention reason. An ActiveBuild lease belongs to the captured build owner; cloning its value does not transfer cleanup authority. Retain only actual sealed IR outputs needed by a read using IrStorageService retain/release; a Session lease alone does not pin IR storage. Keep retention while a published/overlay association or any in-flight reader needs its data; release each owned retention only after both associations and readers are finished; collection may follow the upstream storage rules. Retention allows stale reading, never current publication or artifact authority.

## Implementation boundary

Task6 fixes this design only. Task7 implements the buffer/version registry over these associations; tasks8–11 bind diagnostics, scheduling and lifecycle cleanup. Their contracts must select concrete APIs from actual upstream inputs without creating placeholder full snapshots, metadata indexes, manifests, event producers or runtime identities. No new public type/signature is selected here.

## Test design

Use genuine accepted open/change/close/reopen/terminal messages and protected native capture. Check a version-only edit and a normalization-equivalent raw edit that share a build id, plus an identical-tuple reopen and another Server; each must suppress the old current association while preserving valid retained reads. A reader spanning replacement must observe one complete old/new association. Exercise actual Session lease and IR retain/release/collection separately, including stale-readable versus publisher-obsolete outputs and equal-id replacement refusal. Syntax-only success must not create full metadata/current-build completion; absent/partial producer work must not clear diagnostics or create pending work. Existing language, producer, mapper and lifecycle matrices remain unchanged.
