# mizar-ir Dispatch Input

> Canonical language: English. Japanese companion:
> [../ja/dispatch_input.md](../ja/dispatch_input.md).

## Purpose

This document specifies the `mizar-ir` boundary consumed by
scheduler-selected phase dispatch.

`mizar-build` selects ready tasks and calls a dispatcher. `mizar-driver`
adapts that callback to the phase registry. `mizar-ir` owns the immutable
phase input identity bundle and the sealed parent output handles that a real
phase service may receive once the scheduler has selected a task.

This boundary closes the IR-owned part of the previous build dispatch seam. It
does not move scheduling semantics into `mizar-ir`, and it does not move phase
semantics, type checking, proof acceptance, cache compatibility decisions,
artifact publication tokens, or LSP protocol conversion into `mizar-ir`.

## Owned Boundary

`mizar-ir` owns:

- `PhaseInputIdentities`, the deterministic identity summary used by a phase
  registry query boundary;
- `PhaseDispatchInputBundle`, the immutable dispatch input package containing
  identities plus sealed parent output handles;
- `SealedParentOutputHandle`, a wrapper around a sealed `AnyPhaseOutputRef`
  that has passed snapshot/currentness validation before dispatch;
- a generic `PhaseDispatchInputProvider<Task>` trait so downstream front doors
  can supply IR-owned bundles for scheduler-selected task types without making
  `mizar-ir` depend on `mizar-build` or `mizar-driver`;
- snapshot and current-output validation over `PhaseOutputPublisher` and
  sealed-handle validation over `IrStorageService`;
- canonical ordering of dependency hashes and parent output identities.

Out of scope:

- choosing ready tasks, dependency ordering, cancellation checkpoints,
  resource admission, cache-hit scheduling, or result collation;
- deriving phase semantics, module dependencies, type-checking inputs,
  proof obligations, proof acceptance, or trusted proof status;
- constructing `mizar-cache` `CacheKey`s, dependency fingerprints, cache
  compatibility decisions, or proof-reuse validation;
- producer payload schemas, fake producer output, semantic adapters, proof
  adapters, artifact publication tokens, manifest commits, diagnostics
  rendering, or LSP protocol payloads.

## Data Model

```rust
pub struct PhaseInputIdentities {
    input_hash: Hash,
    dependency_hashes: Vec<Hash>,
    parent_output_hashes: Vec<Hash>,
}

pub struct SealedParentOutputHandle {
    handle: AnyPhaseOutputRef,
}

pub struct PhaseDispatchInputBundle {
    snapshot: BuildSnapshotId,
    identities: PhaseInputIdentities,
    parent_outputs: Vec<SealedParentOutputHandle>,
}

pub struct PhaseDispatchInputRequest<'a, Task: ?Sized> {
    task: &'a Task,
    snapshot: BuildSnapshotId,
}

pub trait PhaseDispatchInputProvider<Task: ?Sized> {
    fn dispatch_input_for_task(
        &self,
        request: PhaseDispatchInputRequest<'_, Task>,
    ) -> Result<Option<PhaseDispatchInputBundle>, DispatchInputError>;
}
```

`snapshot` binds the bundle to the scheduler-selected dispatch snapshot.
Downstream registry/front-door code must validate the returned bundle against
the `PhaseDispatchInputRequest` snapshot before executing a phase. This keeps a
provider from accidentally returning parent handles from a different snapshot.

`input_hash` is a stable owner-supplied identity for non-parent phase inputs
such as source text, selected module identity, typed signatures, normalized VC
descriptors, or toolchain/configuration summaries. `mizar-ir` records and
canonicalizes this hash; it does not derive semantic meaning from it.

`dependency_hashes` are stable hashes supplied by owner seams for non-output
dependencies, such as dependency artifact summaries. Missing dependency data
must be reported by the caller as an owner gap; it must not be replaced with
an empty vector unless the owning seam has actually proven that there are no
dependencies.

`parent_output_hashes` are derived by `mizar-ir` from sealed parent output
handles. They are `PhaseOutputId` hashes, not caller-supplied raw hashes.
Dependent tasks receive the corresponding `SealedParentOutputHandle` values
through execution context, so real services can dereference parent outputs
through storage without treating hashes as payloads.

`PhaseDispatchInputRequest` carries the scheduler-selected downstream task and
the `BuildSnapshotId` selected by the scheduler callback. The provider may use
the snapshot to validate or look up current parent handles, but it must not
re-run scheduler readiness, dependency ordering, cache compatibility, or phase
semantics. A missing bundle is represented as `Ok(None)`. A validation failure
from sealed-handle, currentness, or snapshot checks is represented as
`Err(DispatchInputError)` so callers can distinguish invalid owner input from
an unavailable owner bundle.

## Validation

`SealedParentOutputHandle::from_current_output` validates:

1. the supplied handle is still sealed in the publisher's storage service;
2. the handle belongs to the dispatch snapshot;
3. the handle was published as a current/package output for that snapshot.

`SealedParentOutputHandle::from_validated_rehydrated_output` validates:

1. the supplied handle is still sealed in the supplied storage service;
2. the handle belongs to the dispatch snapshot.

The rehydrated constructor is for handles that an owner has already validated
through `mizar-cache` and rehydrated into the current snapshot. It is not a
cache compatibility decision and it is not proof authority. Obsolete or stale
handles from another snapshot must be rehydrated first by the cache adapter or
kept out of the current dispatch input bundle.

All bundle constructors validate that parent handles belong to the bundle
snapshot, canonicalize dependency hashes by byte order, and canonicalize parent
handles by `PhaseOutputId` hash. Parent identity hashes in
`PhaseInputIdentities` are derived from the canonical parent handle list.
Duplicate parent handles are rejected because a phase input graph should not
silently collapse repeated dependency edges. `PhaseDispatchInputBundle`
exposes `validate_snapshot` for scheduler/front-door code to reject a bundle
that does not match the scheduler-selected dispatch snapshot.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| DISPATCH-IR-G001 | `source_drift` / `boundary_violation` risk | Before this task, `mizar-driver` owned `PhaseInputIdentities` and accepted raw parent output hashes. | Move the identity bundle to `mizar-ir`; the driver consumes it as a front door. |
| DISPATCH-IR-G002 | `external_dependency_gap` | Real producer output payloads and downstream semantic/proof adapters are not ready. | The bundle carries only input identities and sealed parent handles; it does not fabricate producer outputs or adapters. |
| DISPATCH-IR-G003 | `external_dependency_gap` | Artifact publication tokens and manifest commits remain outside `mizar-ir`. | Do not add publication token placeholders. |
| DISPATCH-IR-G004 | `deferred` | Full clean/incremental/parallel equivalence over real semantic, proof, artifact, cache, and LSP integrations requires owner seams beyond this task. | Add crate-local and driver-front-door tests; leave system equivalence deferred. |

## Tests

This task adds focused Rust coverage for:

- canonical dependency and parent identity ordering;
- rejection of duplicate parent handles;
- rejection of bundle/scheduler snapshot mismatches;
- rejection of wrong-snapshot parent handles for current dispatch;
- rejection of foreign-storage parent handles before current or rehydrated
  dispatch;
- rejection of obsolete parent handles as current/package dispatch inputs;
- acceptance of sealed, validated rehydrated current-snapshot handles without
  cache/proof authority;
- driver registry query fingerprints consuming `mizar-ir` identities rather
  than driver-owned raw parent hashes;
- scheduler-selected driver dispatch passing IR-owned identity bundles and
  sealed parent handles to phase execution context;
- source guards proving `mizar-build` does not gain IR authority and
  `mizar-ir` does not gain driver, diagnostics, artifact-token, proof, cache
  compatibility, semantic adapter, proof adapter, or LSP authority.

## Public Enum Forward-Compatibility

`DispatchInputError` is `#[non_exhaustive]` for downstream crates so future
fail-closed dispatch-input validation errors can be added without breaking
external exhaustive matches. This module has no intentional exhaustive
public-enum exception.
