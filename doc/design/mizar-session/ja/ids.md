# Module: ids

> Canonical language: English. English canonical version: [../en/ids.md](../en/ids.md).

## Purpose

この module は `mizar-session` が所有する opaque identifiers を定義する。

Identifiers により、source、snapshot、diagnostic、LSP、cache、IR-facing crates は paths、memory addresses、scheduler counters、unstable frontend internals を expose せずに identity について合意できる。各 id type は scope、ordering rules、serialization boundary を文書化する。

## Public API

```rust
pub struct BuildSessionId(OpaqueId);
pub struct BuildRequestId(OpaqueId);
pub struct BuildSnapshotId(Hash);
pub struct SourceId(OpaqueId);
pub struct SourceMapId(OpaqueId);
pub struct SnapshotLeaseId(OpaqueId);

pub trait SessionIdAllocator {
    fn next_session_id(&self) -> BuildSessionId;
    fn next_request_id(&self) -> BuildRequestId;
    fn next_source_id(&self, snapshot: BuildSnapshotId) -> SourceId;
    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> SourceMapId;
    fn next_lease_id(&self, snapshot: BuildSnapshotId) -> SnapshotLeaseId;
}
```

`BuildSnapshotId` は content-derived fingerprint である。その他の ids は opaque registry identities であり、その値は発行した registry を通してのみ意味を持つ。

## Dependencies

- Internal: none
- External: hashing and stable encoding utilities

この module は他のすべての `mizar-session` modules と、snapshot or source handles を保存する downstream crates から consume される。

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

Persisted artifacts は allocator-issued ids を compatibility promises として扱ってはならない。Schema が明示的に許す場合、content-derived ids を provenance として record してよい。

### Ordering

Ids は semantic order を定義しない。

Deterministic ordering が必要な場合、callers は canonical keys で sort しなければならない。

- source versions by package id, module path, normalized path, and source hash
- diagnostics by source range and diagnostic identity
- artifacts by module path and stable artifact ids

Allocator-issued ids は in-memory maps and debug output のためにのみ order してよい。

### Serialization

Content-derived ids は canonical lowercase hex encoding で serialize してよい。

Allocator-issued ids は、local debug dumps、logs、non-portable と明示された development artifacts に限って serialize してよい。Published artifacts and cache keys は canonical source、dependency、toolchain、configuration hashes を使わなければならない。

## Algorithm / Logic

### Content-Derived Id Construction

`BuildSnapshotId` は canonical snapshot encoding から計算される。Encoding must:

1. id kind の domain separator を含む。
2. interpretation が依存する場合、relevant schema and toolchain identity を含む。
3. unordered collections を hashing 前に sort する。
4. session-local ids、task ids、memory addresses、timestamps、lease ids を除外する。

### Allocator-Issued Id Construction

Allocator-issued ids は owning registry 内で unique でなければならない。Callers が value から semantics を推測できない限り、monotonic counters、random nonces、arena indexes のいずれでもよい。

## Error Handling

`IdError` includes:

- malformed serialized content-derived id
- wrong id domain separator
- id from an unknown snapshot registry
- allocator overflow
- attempt to serialize a non-persistable id into a published schema

Well-formed id を wrong snapshot で使うことは stale-handle error であり、この module ではなく `snapshot` or `retention` が report する。

## Tests

Key scenarios:

- content-derived ids are deterministic for identical canonical input
- changing the domain separator changes the id
- unordered source inputs hash identically after canonical sorting
- session-local ids are absent from snapshot hashes
- allocator-issued ids are unique within one registry
- published-schema serialization rejects non-persistable ids

## Constraints and Assumptions

- Id values are not user-facing names.
- `SourceId` is snapshot-scoped and must not be used as a stable artifact identity.
- `BuildSnapshotId` is an identity and freshness token, not proof authority.
- Debug output may include opaque ids, but reproducible artifacts must not depend on them.
