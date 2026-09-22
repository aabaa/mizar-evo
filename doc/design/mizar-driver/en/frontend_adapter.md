# Source Services

> Canonical language: English. [Japanese companion](../ja/frontend_adapter.md).

## Disk SourceLoad service

`PhaseRegistryBuilder::register_source_load()` registers the real disk-only service named `SourceLoad`, owned by `mizar-frontend`, covering only `PipelinePhase::SourceLoad`. The zero-argument registration method fixes the single-phase descriptor/catalog identity without exposing a second public service type. Frontend remains an external dependency gap: its complete payload and shared diagnostic mapping are not supplied by SourceLoad.
`SourceLoadInputs<'a>` borrows the captured `BuildSnapshot`, `BuildPlan`, `ModuleIndex`, and caller `SessionIdAllocator`. `PhaseExecutionResources<'a>` and `PhaseExecutionContext<'a>` carry optional source inputs; real submission wires them into the scheduler dispatcher. This transport remains dormant under the default graph while later services are missing; A5 validates execution directly through the registry, without a new graph/profile. No allocator or payload is captured in the registered service.

## Binding and execution

Require a module work unit and exactly one matching disk SourceVersion, workspace package plan, package index entry, and workspace module entry. Match package/module/path/edition, package root/source root/manifest, and snapshot/workspace root across owners; reject duplicate or conflicting entries. The module source-relative path must agree with its package-relative path and source root. Reject unsafe workspace-relative plan roots rather than resolving them outside the captured workspace.
Canonical package roots must remain inside the canonical captured workspace; use the bound canonical root for loading. Unavailable roots still reach the loader for its real diagnostics, but successful publication requires proven root containment.
Derive a Disk SourceInput from the captured typed normalized path and source metadata. Resolve the package root from the captured workspace and workspace package plan; let the session loader own filesystem canonicalization and source acceptance.
The dispatch input hash must equal the SourceUnitCacheKey for the captured version; SourceLoad consumes no output parents or dependency hashes. Missing owner dispatch identities remain blocked; this service does not invent planner output handles.
Require a current supplied publisher and an empty, unsealed diagnostic sink scoped to SourceLoad and the same snapshot. Missing/mismatched resources, invalid bindings/identities, publication failure, or source changes after capture return Blocking without output. A successful load whose hash differs from the captured version is rejected as an obsolete dispatch result under architecture 22, not classified as a loader failure or assigned a new E0600 cause. A matching cancellation token returns Cancelled; a foreign token returns Blocking.
Call `FrontendSourceLoader<DiskSourceLoader>::load_source_unit` with the caller allocator. Validate loaded metadata, Disk origin and normalized text hash against the captured version. Use the disk codec to rebind the actual loaded payload/maps to the captured SourceId, then publish it through the supplied publisher. The service never registers/revives publisher snapshots or grants work-unit permission.
Use the [frontend-owned publication representation](../../mizar-frontend/en/source.md#sourceunit-publication). Decoder closures capture current request metadata, SourceId and expected source hash only; they decode bytes, never retain the produced payload or reload/reparse a file.
Return Complete with the actual sealed output only after successful publication. The full driver submission still refuses graphs with missing later services. Direct SourceLoad execution is not full build completion.

## Diagnostics and cache boundary

Real loader errors emit validated shared drafts with the request's package/path location, SourceLoad phase/category, current snapshot, and no fabricated range, then return Fatal without output. Map InvalidUtf8 to E0601, UnreadableSourceFile to E0602, and SourcePathOutsidePackageRoot to E0603. For E0601–E0603, use the allocated semantic name as the stable detail key. Other variants map to E0600, with stable detail keys `source.` plus the variant name in snake_case; unknown future variants remain an unsupported integration gap and return Blocking without a fabricated diagnostic. Message text is display-only. Failure to construct/emit a draft returns Blocking.
`cache_key` returns NoKey: normalized source hashes cannot prove that cached raw loading maps match the current file. Every execution loads the real source. The codec supports storage placement, not cache-hit scheduling or cross-snapshot reuse credit.

## Tests and remaining ownership

Use real temporary source files and actual planner/index/snapshot outputs. Exercise valid resident/blob publication, current SourceId/map rebinding, equal normalized text with distinct raw maps, changed text after capture, failed request/resource binding, cancellation, stale publisher and work-unit denial.
Exercise real invalid UTF-8, deleted/unreadable files, symlink escape where supported, and allocator failure through the shared diagnostic sink. Preserve absent later-service blocking and existing registry tests.
Preprocessing, lexing, parsing, recovery, full Frontend serialization and its diagnostic conversion remain frontend-owner integration work. Cache compatibility, LSP conversion, artifact publication and later semantic/proof phases stay with their existing owners.

## Dependency lexical provider

The provider adds the `mizar-resolve` production dependency; `mizar-artifact` is the sole dev dependency for real publication fixtures. The exact dependency-boundary lint admits these owner seams only.

`SourceLoadInputs::dependency_lexical_provider(artifact_roots)` returns a private
implementation of the actual frontend `LexicalSummaryProvider` contract. Roots
are borrowed `(PackageId, PathBuf)` bindings supplied by the caller; no registry
installation path is guessed and no root is retained in a new public resource.
The caller supplies a build-validated module index and stubs from the source
version represented by the request SourceId; the request carries no source hash.
Require exactly one captured source for that SourceId, matching request and
indexed module editions, and matching indexed package/module metadata.
Map stubs with the resolver's provisional frontend mapper and resolve paths
through its existing index-backed import resolver. This is not AST import validation.
For each resolved dependency target, require matching module/reference artifact
path and hash plus a unique explicit root. Read through the build-owned indexed
summary reader, then the resolver's canonical lexical-summary consumer.
Successful entries retain the summary's full artifact module identity, with the
original stub ordinal/span. Each successful `ResolvedImportEntry.import.module_id`
must equal the A15 `ModuleLexicalSummary.module_id` exactly. Duplicate imports remain in provenance; frontend
canonicalization owns their active-environment deduplication.
A file-read rejection retains a resolved entry but no summary, so the frontend
owns its existing `MissingSummary` recovery. Its diagnostic-only module ID is
`format!("{:?}:{:?}", package_id, module_path)` over the indexed string parts;
it asserts only known package/path identity and never reaches the lexer.
Unresolved paths remain absent for existing `UnresolvedImport` recovery.
Malformed request/index/root bindings, unsupported source-backed targets, or a
rejected lexical payload return `ProviderUnavailable`, without partial output.
No empty source summary is fabricated. Existing frontend environment assembly
owns conflict recovery; shared diagnostic conversion, full Frontend publication,
complete source export producers and current-build lock/cache/proof acceptance
remain outside this provider.
