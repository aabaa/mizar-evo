# Module: ids

> Canonical language: English. Japanese companion: [../ja/ids.md](../ja/ids.md).

## Purpose

This module defines opaque identifiers owned by `mizar-session`.

The identifiers let source, snapshot, diagnostic, LSP, cache, and IR-facing crates agree on identity without exposing paths, memory addresses, scheduler counters, or unstable frontend internals. Each id type documents its scope, ordering rules, and serialization boundary.

## Public API

```rust
pub struct BuildSessionId(OpaqueId);
pub struct BuildRequestId(OpaqueId);
pub struct BuildSnapshotId(Hash);
pub struct SourceId(OpaqueId);
pub struct SourceMapId(OpaqueId);
pub struct SnapshotLeaseId(OpaqueId);

pub trait SessionIdAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError>;
    fn next_request_id(&self) -> Result<BuildRequestId, IdError>;
    fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError>;
    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError>;
    fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError>;
}

pub struct InMemorySessionIdAllocator { /* private fields */ }

impl InMemorySessionIdAllocator {
    pub const fn new() -> Self;
}
```

`BuildSnapshotId` is a content-derived fingerprint. Other ids are opaque registry identities whose values are meaningful only through the registry that issued them.

## Dependencies

- Internal: none
- External: hashing and stable encoding utilities

This module is consumed by all other `mizar-session` modules and by downstream crates that store snapshot or source handles.

## Data Structures

### Identifier Scope

| Identifier | Scope | Derived From | May Persist? |
|---|---|---|---|
| `BuildSessionId` | one compiler-driver run | allocator | no |
| `BuildRequestId` | one batch/watch/LSP request generation | allocator | no |
| `BuildSnapshotId` | complete build input state | canonical snapshot hash | yes, as provenance |
| `SourceId` | one `BuildSnapshotId` | snapshot registry assignment | no |
| `SourceMapId` | one retained source map | allocator | no |
| `SnapshotLeaseId` | one live retention lease | allocator | no |

Persisted artifacts must not treat allocator-issued ids as compatibility promises. They may record content-derived ids as provenance when the schema explicitly allows it.

### Ordering

Ids do not define semantic order.

When deterministic ordering is needed, callers must sort by canonical keys:

- source versions by package id, module path, normalized path, and source hash;
- diagnostics by source range and diagnostic identity;
- artifacts by module path and stable artifact ids.

Allocator-issued ids may be ordered only for in-memory maps and debug output.

### Serialization

Content-derived ids may be serialized through canonical lowercase hex encoding.
`BuildSnapshotId` uses the published-schema form
`mizar-session-build-snapshot-v1:<64 lowercase hex digits>`. The prefix is the
serialized id domain; deserialization must reject any other domain and any
non-canonical hex spelling.

Allocator-issued ids may be serialized only in local debug dumps, logs, and development artifacts that are explicitly marked non-portable. Published artifacts and cache keys must use canonical source, dependency, toolchain, and configuration hashes instead.

## Algorithm / Logic

### Content-Derived Id Construction

`BuildSnapshotId` is computed from canonical snapshot encoding. The encoding must:

1. include a domain separator for the id kind;
2. include relevant schema and toolchain identity when interpretation depends on them;
3. sort unordered collections before hashing;
4. exclude session-local ids, task ids, memory addresses, timestamps, and lease ids.

### Allocator-Issued Id Construction

Allocator-issued ids must be unique within the owning registry. They may be monotonic counters, random nonces, or arena indexes, as long as callers cannot infer semantics from the value.

Allocator methods return `IdError::AllocatorOverflow` if the allocator cannot issue another unique id. Source, source-map, and lease id methods take a `BuildSnapshotId` so the snapshot-scoped allocation boundary is visible at the API.

## Error Handling

`IdError` includes:

- malformed serialized content-derived id;
- wrong id domain separator;
- id from an unknown snapshot registry;
- allocator overflow;
- attempt to serialize a non-persistable id into a published schema.

Using a well-formed id in the wrong snapshot is a stale-handle error reported by `snapshot` or `retention`, not by this module.

## Tests

Key scenarios:

- content-derived ids are deterministic for identical canonical input;
- changing the domain separator changes the id;
- unordered source inputs hash identically after canonical sorting;
- session-local ids are absent from snapshot hashes;
- allocator-issued ids are unique within one registry;
- published-schema serialization rejects non-persistable ids.

## Constraints and Assumptions

- Id values are not user-facing names.
- `SourceId` is snapshot-scoped and must not be used as a stable artifact identity.
- `BuildSnapshotId` is an identity and freshness token, not proof authority.
- Debug output may include opaque ids, but reproducible artifacts must not depend on them.
