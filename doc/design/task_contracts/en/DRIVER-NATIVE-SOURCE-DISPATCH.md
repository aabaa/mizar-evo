# Task DRIVER-NATIVE-SOURCE-DISPATCH: captured native source services
Canonical language: English; [Japanese pointer](../ja/DRIVER-NATIVE-SOURCE-DISPATCH.md).
Status: implemented. Tier: full. Owners: [driver plan](../../mizar-driver/en/00.crate_plan.md), [frontend plan](../../mizar-frontend/en/00.crate_plan.md).
Scope: driver 27 / frontend 31; native SourceLoad/Frontend prerequisite for LSP wave B.
Authority: [capture](../../mizar-driver/en/request.md#native-source-capture), [services](../../mizar-driver/en/frontend_adapter.md#native-captured-sources), [submission](../../mizar-driver/en/driver.md#captured-native-submission); architecture 02/12/22, IV-007 and spec 22.3.5/23.6.
Dependencies: SESSION-SOURCE-CAPTURE, FRONTEND-OPEN-BUFFER-STORAGE and IR-OPEN-BUFFER-DISPATCH IR21.
No language, proof acceptance, cache policy or artifact-publication decision is authorized.

Add BuildRequestDraft::capture_sources<A>(self, allocator: &A, registry: &SnapshotRegistry<A>,
sources: Vec<(DiskSourceLoader, SourceInput)>) -> Result<BuildSession, DriverSubmitError>.
Reject populated source_inputs.versions before IDs through existing-enum
NativeSourceVersionsProvided { request: Box<BuildRequestDraft> }; never discard expectations.
Allocate request/session normally; call real load_snapshot once. Failure preserves original Pending
through existing capture error; caller retains descriptors for retry and counters do not roll back.
Success fills request summaries from the actual snapshot and privately retains original loader/root,
LoadedSource/SourceInput triples in existing CapturedSnapshot with the real ActiveBuild lease.
BuildSession id/request/captured/state and CapturedSnapshot snapshot/lease become crate-visible only;
public read-only id()/request()/captured()/state(), snapshot()/active_snapshot_lease() preserve observation.
Internal lifecycle/forced watch supersession remain unchanged; no automatic lease/raw-resource release.
Add CompilerDriver::submit_captured and submit_captured_with_import_discovery using existing
allocator/registry/DriverSubmitInput parameters and a BuildSession instead of a draft.
Reject non-native/non-SnapshotCaptured, reused session id in this Driver and non-current registry request
before lane mutation via existing-enum InvalidCapturedSession { session: Box<BuildSession> }.
Both new errors yield no accepted watch replacement; CLI uses its existing internal-error envelope.
Caller authorizes actual snapshot/work units through the existing publisher before submit;
Driver/services never register/revive snapshots or grant rights. No second acquisition/ID guess.
Evolve existing submit_inner to consume a captured session; legacy submits allocate/capture as before.
Keep existing concrete generic A restriction; distinct allocator instances remain legal.

SourceLoadInputs optionally borrows genuine CapturedSnapshot at its two Driver constructions.
Three real consumers (discovery, SourceLoad, Frontend) share native binding against the entire protected
snapshot, unique actual source/id/hash/origin/raw URI/version/maps and retained loader root plus plan/index.
Summary metadata alone grants no native authority. Preserve legacy no-capture Disk drift/errors/diagnostics;
forged Open summaries and Generated service inputs remain unsupported. Captured Disk/Open never reload text.
Expose existing frontend diagnostic_file_path(&SourceInput) -> PathBuf for its third actual caller;
reuse exact URI/display policy and actual source projection, without duplicate parsing/normalization.
Preserve provider/dependency/parent/cancellation/source-import correspondence and diagnostic conversion.
Authenticate complete own/supplied workspace-parent source payloads against capture, including raw maps.
Use existing Disk/Open codecs; Open publishes OpenBuffer/InternalOnly, Disk remains current/package.
Only genuine native dispatch adopts published parent/result checks at the four Driver consumers;
legacy dispatch retains all four current/package checks and existing Open-result rejection. Both intents stay NoKey.
No later service, semantic LSP snapshot/current diagnostics, cache hit, proof/artifact/full-build or Step completion credit.

Existing request/driver/source_load fixtures cover actual capture, owner authorization, scheduled Disk/Open,
real diagnostics/NoKey, physical blob retrieval/full payloads, imports, replay/order/workers, same canonical id
with changed raw/version/ids and lane supersession/cancellation; positive raw replays use fresh owner publishers.
Existing IR conflicting-slot rejection is preserved. Typed pre-ID/capture/prepared failures,
preserved pending context, denied rights, foreign/tampered binding, unsupported origins and legacy rejection fail closed.
Preserve five captured-summary tampering controls through separate forged snapshot views; no assertion weakening.
Source scope: Driver request.rs/driver.rs/driver/scheduler.rs/registry.rs/frontend_adapter.rs and existing
driver/watch.rs and cli/output.rs exhaustive error arms; existing Driver integration fixtures
(accessors/native cases); Frontend source.rs display-path visibility only.
No new Rust file/type/DTO/adapter/dependency/identity schema, protected spec/.miz/expectation or trace change.
Paired request/driver/registry/service/source owners, plans/TODOs, roadmap and audit follow-up ownership only.
Require independent spec/test/implementation/volume/consistency/bounded-gate reviews and parent full checks.
Exit: genuine captured native scheduled prefix; LSP and downstream producer gates remain open.
