# Task STEP6A3-IR-STORAGE-PAYLOAD: separate semantic and storage bytes

Canonical language: English; [Japanese pointer](../ja/STEP6A3-IR-STORAGE-PAYLOAD.md).

Status: implemented; full tier. Primary owner: mizar-ir.
Purpose: publish lossless storage payloads without hashing maps as semantics.
Consumers: real producer services, including the disk SourceUnit codec.
Dependencies: IR publisher/storage/cache and frontend storage codec; ready.
Owner plan: [IR](../../mizar-ir/en/00.crate_plan.md).

## Authority and inventory

- [IR immutable snapshots](../../architecture/en/01.ir_layers.md#irs-are-immutable-snapshots).
- [Publisher hashes](../../mizar-ir/en/publisher.md#canonical-hashing),
  [storage](../../mizar-ir/en/storage.md),
  [cache adapter](../../mizar-ir/en/cache_adapter.md).
- `design_drift`: one canonical stream serves semantic hashing and lossless
  storage, although source maps must retain separate side-table identity.
- `source_drift`: resident canonical seals discard byte fingerprints; cache
  encoding cannot bind a separate supplied storage stream to the sealed handle.
- Existing tests: IR inline publisher/storage/cache tests and lifetime tests;
  driver registry/dispatch/watch fixtures consume the affected input structs.
- No semantic/diagnostic policy, coverage-audit, or .miz expectation changes.

## Frozen scope

- Keep canonical_payload as semantic bytes; add explicit storage_payload to
  publisher/cache encoding requests. Existing equal-stream producers supply both.
- Publisher hashes semantic bytes and seals/decodes storage bytes; reject either
  missing stream before registration. Producer owns payload/stream coherence.
- Preserve the existing content_blob_id fingerprint for resident and blob seals;
  cache encoding must authenticate supplied storage bytes against sealed storage.
  Resident-only seals lacking storage bytes remain usable but are not encodable.
- Encode both streams and storage fingerprint in a versioned cache envelope;
  reject old format, malformed lengths, semantic/storage hash mismatch before
  decoding or registering output. Validated cache ownership remains mizar-cache.
- Rehydration decodes storage bytes and republishes both streams in current
  snapshot; preserve parent/freshness, side-table and fail-closed checks.
- Update IR source/tests, affected existing driver/IR fixtures, paired owning
  docs/API inventories, plan index and top-level Step 6 sequencing only.
- No new producer adapter, semantic projector, diagnostics, source identity
  persistence, cache-key/proof policy, artifact token, Step 7 or MVM execution.

## Acceptance

Test semantic-equal/storage-different publication with corresponding side tables
and distinct identities, and placement from storage length rather than semantic length;
resident/blob binding, foreign/collected/noncanonical handle rejection, valid
cache roundtrip and independently tampered streams/fingerprint/legacy format.
Independent specification, test-sufficiency, implementation, volume/scope and
source/documentation reviews; protocol quality evaluation before commit.
Run IR and driver tests, `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
and `git diff --check`. Exit: both streams retain their distinct identities
through sealing/cache without conferring producer or artifact readiness.
