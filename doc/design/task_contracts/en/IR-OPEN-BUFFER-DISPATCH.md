# Task IR-OPEN-BUFFER-DISPATCH: genuine internal output authorization
Canonical language: English; [Japanese pointer](../ja/IR-OPEN-BUFFER-DISPATCH.md).
Status: implemented for IR task 21 only. Tier: full. Owners: [IR plan](../../mizar-ir/en/00.crate_plan.md), [driver plan](../../mizar-driver/en/00.crate_plan.md).
Scope: IR task 21, prerequisite for driver task 27, native editor services and LSP 6–7.
Authority: [publisher authorization](../../mizar-ir/en/publisher.md#published-output-authorization), [dispatch validation](../../mizar-ir/en/dispatch_input.md#published-parents), [driver routing](../../mizar-driver/en/driver.md#published-output-authorization); architecture 12, internal 03/06 and IV-007.
Dependencies: existing genuine publisher/storage/dispatch, native SESSION-SOURCE-CAPTURE and FRONTEND-OPEN-BUFFER-STORAGE codecs.
Gap: successful OpenBuffer/InternalOnly outputs lack publisher authorization usable by same-run dispatch; storage/lineage alone cannot establish publication authority.
No language, diagnostic, proof, cache-acceptance or artifact-policy decision is authorized.

Add `PhaseOutputPublisher::validate_published_output(snapshot: BuildSnapshotId,
output: &AnyPhaseOutputRef) -> Result<(), PublishError>`.
Add `SealedParentOutputHandle::from_published_output(publisher: &PhaseOutputPublisher,
snapshot: BuildSnapshotId, handle: AnyPhaseOutputRef) -> Result<Self, DispatchInputError>`.
Add existing-enum `PublishError::OutputNotPublished { snapshot: BuildSnapshotId, output: PhaseOutputId }`.
Record only actual successful OpenBuffer/InternalOnly sealing, reusing the existing
private output/generation key; do not create current/package roots for these outputs.
Validate current publisher snapshot, matching output snapshot and exact recorded
output/generation: old CurrentPackage successes or genuine OpenBuffer/InternalOnly successes.
Ordinary InternalOnly, retained-stale origins, direct storage sealing, foreign publisher,
failed sealing and unrecorded generations gain no authority. Storage still validates physical handles.
Preserve old current-output/current-parent and cache-owner rehydrated APIs, errors/order/meaning;
reuse their validation sequence with the new constructor at three actual caller sites.
Preserve all four Driver current/package consumer checks and Disk-only source binding.
Published Open/internal provenance alone cannot authenticate a summary-only/Disk dispatch result;
Driver adoption waits for genuine native capture and input binding under driver task 27.
No new public type, DTO, adapter, source file, dependency or origin/target hash/schema change.
No raw transport/acquisition, registered Open service, source/URI validity token, editor freshness,
cache hit, proof acceptance, artifact publication or whole-milestone credit. Caller source/version/lane
binding remains necessary even when canonical snapshot ids repeat across raw/version changes.

Tests extend existing IR publisher/dispatch and Driver source_load fixtures:
genuine native capture/projection/real Frontend output publishes OpenBuffer/InternalOnly;
actual source/cache-key semantic bytes, existing side-table hashes and genuine SourceLoad parent;
force physical blobs and count actual decoders, retaining raw descriptors rather than typed payloads
in decoder closures, compare complete returned payloads after backing-file changes/deletion.
Require current/package rejection, old current-constructor rejection and new published-parent success;
wrong snapshot/publisher/storage, obsolete/superseded, collected/resealed generation, unrecorded
ordinary/retained internal and failed/direct sealing fail closed. Existing Disk/cancellation/provider
and rehydration controls remain unchanged; direct owner publication is not a Driver service claim.
Source scope: IR publisher.rs/dispatch_input.rs and existing Driver tests/source_load.rs; inline IR tests.
Paired owner plans/TODOs/publisher/dispatch/driver/frontend-adapter; audit follow-up ownership only.
Coverage: no language trace, active corpus or acceptance change; editor/service gates remain open.
Require independent specification, test-sufficiency, implementation, volume/scope, consistency and
bounded gate reviews; parent full fmt, Clippy all-targets/all-features -D warnings and cargo test.
Exit: real same-run internal publication authorization; native capture, input binding and Driver adoption follow separately.
