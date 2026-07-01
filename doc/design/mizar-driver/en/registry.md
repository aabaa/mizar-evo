# Module: registry

> Canonical language: English. Japanese companion:
> [../ja/registry.md](../ja/registry.md).

## Purpose

This module defines the driver-owned phase service registry and salsa query
boundary.

`mizar-driver` assembles phase services behind one deterministic front door,
maps scheduler work into service calls, and owns the query/database layer that
keeps phase crates free of a direct salsa dependency. The registry wires real
owner seams only. If a phase adapter, producer output, artifact publication
token, or LSP bridge is unavailable, the registry records an
`external_dependency_gap` or `deferred` item instead of registering a fake
adapter, stub API, provisional token, or temporary wiring.

## Ownership Boundary

`registry` owns:

- deterministic registration, lookup, and duplicate-phase rejection for phase
  services;
- the service table for architecture phases 0 through 16;
- the driver-owned salsa database and query adapters that call registered
  services;
- the `PhaseService` contract, including `phase`, `cache_key`, and `execute`;
- `PhaseContext` and `PhaseResult` shapes as protocol-agnostic driver/service
  boundaries;
- purity checks for service cache-key projections;
- conversion from driver query outcomes into scheduler/cache-seam inputs that
  `mizar-build` can consume.

`registry` does not own:

- source loading, parsing, name resolution, type checking, VC generation,
  proof acceptance, trusted status, kernel acceptance, ATP winner policy, or
  artifact semantics;
- `mizar-cache` cache compatibility decisions, dependency-fingerprint
  construction, proof-reuse validation, cache-store lookup, or cache promotion;
- `mizar-build` task graph readiness, worker scheduling, resource admission,
  cancellation-state propagation, or scheduler result collation;
- `mizar-ir` sealed storage internals, IR identity assignment, or artifact
  projection schemas;
- `mizar-artifact` manifest transactions, artifact serialization, or
  publication-token issuance;
- `mizar-diagnostics` diagnostic-code allocation, diagnostic identity, record
  aggregation, rendering, or explanation/fix resolution;
- `mizar-lsp` protocol conversion, document-version handling, editor command
  shaping, or LSP diagnostic/code-action payloads.

## Public Enum Compatibility

All public enums in this module are downstream-facing registry boundary types
and are marked `#[non_exhaustive]`. D-017 records no exhaustive exceptions for:

- `PhaseOwner`;
- `PhaseServiceAvailability`;
- `PhaseStatus`;
- `PhaseCacheIntent`;
- `PhaseRegistryError`.

Downstream crates must use wildcard arms when matching these enums. Future
phase owners, status values, cache intents, or registry errors may be added
without transferring phase semantics, cache compatibility, proof acceptance,
artifact publication, or LSP authority into the driver.

## Gap Classification

| Gap | Classification | Registry disposition |
|---|---|---|
| Real semantic phase adapters for phases 4-14 are not all available. | `external_dependency_gap` | Leave those phases unregistered until the owning crates expose real service surfaces. |
| Real phase-15 producer outputs and artifact publication tokens are not available to the driver. | `external_dependency_gap` | Accept only owner-provided artifact/proof seams later; do not mint publication authority. |
| Real LSP protocol bridge and event consumption are owned by `mizar-lsp`. | `external_dependency_gap` | Keep registry outputs protocol-agnostic; event/LSP conversion belongs outside this module. |
| Durable cache lookup and compatibility are owned by `mizar-cache`. | `external_dependency_gap` until a later cache integration task wires the real owner API | Task 5 records cache-key intent through the driver query boundary only. Compatibility, lookup, and proof-reuse decisions stay in `mizar-cache`. |
| The requested `mizar-artifact` closeout report is absent in this checkout. | `repo_metadata_conflict` | Report only; do not repair artifact metadata in the driver task stream. |
| Test-only registry fixture services are needed for deterministic registration and cache-key purity tests. | allowed test fixture | Fixtures must live only in tests and must not be exported or presented as real phase adapters. |

## Task D-015 Semantic Adapter Readiness

Task D-015 audits the semantic/proof/artifact/doc phase adapters after the
driver core, event stream, CLI batch entry point, and watch orchestration exist.
No production semantic adapter is registered by this task. A real adapter may be
registered only when the owner crate exposes all of these seams at once:

- a driver-callable service input over scheduler work-unit identity and sealed
  parent output handles;
- a canonical producer output that can be stored through `mizar-ir` without a
  synthetic payload;
- diagnostic records or a documented diagnostics bridge through
  `mizar-diagnostics`, not message-text identity;
- proof, cache, artifact, and LSP authority remaining in the owning crates.

| Service | D-015 readiness | Classification |
|---|---|---|
| `ModuleResolver` | `mizar-resolve` exposes resolver-owned data shapes and deterministic internal diagnostics, but public resolver diagnostic codes and artifact-backed `ModuleSummary` reuse remain unavailable; no driver service envelope or sealed producer payload is exposed. | `external_dependency_gap` |
| `SemanticChecker` | `mizar-checker` exposes explicit checker-owned payloads, but source-derived payload extraction, accepted proof/artifact status integration, public diagnostic allocation, artifact emission/reuse, and a unified driver service envelope remain unavailable. | `external_dependency_gap` |
| `Elaborator` | `mizar-core` exposes explicit lowering/control-flow data and local diagnostics, but source-to-checker extraction, concrete downstream identities, artifact schemas, public diagnostic allocation, and downstream VC/kernel/proof/artifact consumers remain unavailable. | `external_dependency_gap` |
| `VcService` | `mizar-vc` exposes untrusted VC candidates, handoff hashes, and reuse inputs over explicit payloads, but source-derived `proof_verification` runners, full upstream payload families, downstream proof/cache/artifact consumers, and accepted discharge paths remain unavailable. | `external_dependency_gap` |
| `AtpService` | `mizar-atp` exposes untrusted backend-neutral problem/candidate machinery and generic runner behavior, but real backend adapters, backend-output extraction to kernel-owned formula/substitution candidates, proof-policy integration, cache reuse, and witness publication remain unavailable. | `external_dependency_gap` |
| `KernelService` | `mizar-kernel` exposes trusted evidence checking for explicit normalized inputs, but source-derived certificates, ATP proof translation, cluster/reduction producer payloads, service-envelope normalization, cancellation plumbing, and downstream proof/cache/artifact consumers remain unavailable. | `external_dependency_gap` |
| `ArtifactService` | `mizar-artifact` exposes artifact schemas/store primitives, but real producer projections and phase-15 emission are still external gaps, and the driver has no publication-token authority to mint. The requested closeout report is absent as a separate report-only `repo_metadata_conflict` recorded in the gap table. | `external_dependency_gap` |
| `DocExtractionService` | `mizar-doc` has design TODOs, but no workspace crate or implemented service-facing extraction owner surface exists. | `deferred` |

Existing registry tests remain the appropriate coverage for D-015 because no
real adapter is registered: they prove missing services report owner gaps,
test-only fixtures stay local to tests, duplicate coverage is rejected, and the
driver/query boundary does not move proof/cache/artifact/LSP authority. Per-
adapter fixture tests must be added in the later task that registers each real
adapter.

## Phase Service Table

The registry records one service descriptor for each architecture phase or
contiguous phase group. A descriptor covers one or more phases; duplicate
coverage is rejected even if service names differ. Services may be absent until
their real owning crate exposes an adapter seam.

| Service | Phases | Owner seam | Registry status |
|---|---:|---|---|
| `WorkspacePlanner` | 0 | `mizar-build` planner | Real bootstrap owner exists; driver task 8 wires it without duplicating planner semantics. |
| `SourceFrontend` | 1-3 | `mizar-frontend` | D-006 records the adapter as `external_dependency_gap`; a real adapter requires future producer/diagnostic/input seams. |
| `ModuleResolver` | 4-5 | `mizar-resolve` | `external_dependency_gap` until the resolver exposes a service surface. |
| `SemanticChecker` | 6-8 | `mizar-checker` | `external_dependency_gap` until checker services expose real typed outputs. |
| `Elaborator` | 9-10 | `mizar-core` | `external_dependency_gap` until core/elaboration services land. |
| `VcService` | 11-12 | `mizar-vc` | `external_dependency_gap` until VC generation and deterministic discharge services land. |
| `AtpService` | 13 | `mizar-atp` | `external_dependency_gap`; ATP policy and backend evidence remain outside the registry. |
| `KernelService` | 14 | `mizar-kernel`/`mizar-proof` | `external_dependency_gap`; kernel acceptance and proof-status projection remain owner decisions. |
| `ArtifactService` | 15 | `mizar-artifact` plus producer seams | `external_dependency_gap`; no publication token is invented by the registry. |
| `DocExtractionService` | 16 | `mizar-doc` | `deferred` until documentation/extraction owner surfaces land. |

The Rust registry uses `mizar-build::task_graph::PipelinePhase` as its concrete
phase key. Some `PipelinePhase` variants intentionally group architecture
subphases: `Frontend` covers lexing/parsing phases 2-3, and `BackendRun` is
the backend execution subphase of ATP dispatch. The registry's internal rank is
therefore a deterministic ordering over current `PipelinePhase` variants, not a
replacement for the architecture phase numbers in the table above.

The registry may expose missing-service diagnostics for a submitted task, but
the diagnostic must identify the missing owner seam. It must not emulate the
phase, fabricate output handles, or mark a phase as complete.

## Data Model

### PhaseService

The registry-level trait is conceptual; task 5 may choose concrete Rust type
parameters or object-safe adapters that fit the available owner APIs.

```rust
trait PhaseService {
    fn phase(&self) -> PhaseDescriptor;
    fn cache_key(&self, input: &PhaseInput, context: &PhaseCacheContext) -> PhaseCacheIntent;
    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext) -> PhaseResult;
}
```

`phase` returns a descriptor containing:

- the service name;
- the covered architecture phase or contiguous phase range;
- the service schema/version used for cache and query identity;
- the owning crate or adapter owner;
- the output kind family expected from `execute`.

The descriptor is registration identity, not semantic authority. Phase meaning
comes from the architecture specs and owning phase crates.

`cache_key` is a pure projection from supplied input identities and context
identity fields into a cache-key intent or owner-provided cache-key request. It
does not perform cache lookup and does not decide cache compatibility. When the
real cache seam is wired, the driver-owned query boundary calls
`mizar-cache` to construct/validate the canonical `CacheKey` and lookup result.

`execute` runs the owner service for one immutable input and returns structured
outputs. It must not mutate scheduler state, publish events directly, write
artifacts, convert to LSP payloads, or promote proof/cache/artifact records to
trusted status.

### PhaseContext

```rust
struct PhaseContext {
    snapshot: BuildSnapshotId,
    work_unit: WorkUnit,
}

struct PhaseCacheContext {
    common: PhaseContext,
    input_identities: PhaseInputIdentities,
}

struct PhaseExecutionContext {
    common: PhaseContext,
    cancellation: Option<CancellationToken>,
    diagnostics: Option<DiagnosticSink>,
    output_publisher: Option<PhaseOutputPublisher>,
}
```

`PhaseContext` is the immutable common driver-service identity boundary.
`PhaseCacheContext` is the narrowed view passed to `cache_key`; it contains
only immutable identity data allowed by the purity contract. `PhaseExecutionContext`
is passed to `execute` and contains the owner handles needed for execution.
These fields are references to owner seams:

- snapshot ids come from `request` and `mizar-session`;
- session id and request generation stay in the driver query/event/publication
  layer and are excluded from all `PhaseService` contexts;
- work-unit identity and cancellation are consumed from `mizar-build`;
- diagnostics flow through `mizar-diagnostics` producer sinks and records;
- output sealing flows through `mizar-ir` publisher/storage handles.

The handles are optional in task 5 because no real phase adapter is registered
yet. A real adapter must use the owner handle supplied by the registry context
or report an `external_dependency_gap`; it must not bypass the context with a
private sink, output store, or publication token.

`PhaseContext` must not expose mutable scheduler internals, wall-clock time,
worker ids, event subscribers, LSP protocol payloads, artifact manifest
transactions, cache-store handles, cache lookup handles, or publication tokens
that are not provided by the owning crate. Cache lookup happens in the
driver-owned query adapter outside `PhaseService` methods, after `cache_key`
returns a pure intent.

### PhaseResult

```rust
struct PhaseResult {
    status: PhaseStatus,
    diagnostics: Vec<DiagnosticBatch>,
    output_refs: Vec<AnyPhaseOutputRef>,
    cache_observation: Option<PhaseCacheObservation>,
}
```

`PhaseStatus` is a driver/scheduler status projection:

- `Complete` means the phase produced all outputs required by dependents;
- `Recoverable` means syntax-only or development features may continue, but
  semantic acceptance and proof status must not use degraded metadata;
- `Blocking` stops dependent work for the affected unit only;
- `Fatal` stops the session because the global build state cannot be trusted;
- `Cancelled` records cooperative cancellation without publishing current
  outputs.

A status is not proof acceptance, trusted status, cache compatibility, or
artifact publication. Diagnostics are structured records/batches; rendered
message text is never identity.

## Registration Rules

Registration happens through a builder before scheduler submission for a
session. The builder produces an immutable `PhaseRegistry`.

1. Normalize each service descriptor.
2. Sort descriptors by first covered phase, covered phase span, service name,
   and schema version.
3. Reject duplicate coverage for any architecture phase 0 through 16.
4. Reject descriptors whose phase span is empty, non-contiguous, outside
   0 through 16, or incompatible with the service table.
5. Reject descriptors that claim a phase owner different from the owning-crate
   seam recorded in this spec unless a later spec update changes ownership.
6. Freeze the table so lookup order is independent of registration order,
   hash-map order, or plugin discovery order.

Lookup by phase returns either the registered service descriptor or a
missing-service error with the gap classification. Missing services never
produce synthetic phase outputs.

## Cache-Key Purity Contract

`PhaseService::cache_key` may depend on:

- `BuildSnapshotId` and immutable snapshot content identities;
- package, module, work-unit, phase, and output-kind identities;
- source hashes, dependency artifact hashes, lockfile hash, toolchain identity,
  language edition, verifier policy/configuration hashes, and owner schema
  versions;
- owner-provided dependency slices or fingerprints;
- deterministic phase input hashes and sealed parent output identities.

It must not depend on:

- `BuildSessionId`, `BuildRequestId`, `BuildLaneId`, request generation, or
  watch/LSP supersession state;
- scheduler worker count, worker id, ready-queue timing, cancellation timing,
  event subscriber state, wall-clock time, random seeds not recorded in the
  input identity, environment variables, or filesystem metadata outside the
  declared source/dependency/artifact identities;
- rendered diagnostic text, CLI progress formatting, LSP document-version
  payloads, JSON-RPC ids, editor ranges after protocol conversion, or code
  actions;
- previous cache hit/miss timing, cache-store directory iteration order, or
  artifact commit order.

The task-5 implementation must include a purity harness. Running `cache_key`
twice with equivalent inputs and context identities must produce identical
intent/hash data. Changing only forbidden runtime/session fields must not
change the cache-key intent. Changing any required content identity must change
the intent or produce an explicit uncacheable/no-key result. The harness may
use test-only fixture services, but production code must not ship fake phase
adapters.

## Salsa Query Boundary

`mizar-driver` owns the salsa database used for orchestration. Phase crates
must expose pure services and must not depend on salsa directly because that
would leak driver/query-engine lifetime and invalidation policy into semantic
owners.

The registry creates or borrows a driver database for one driver process. For
each `BuildSession`, the driver installs immutable input queries for:

- captured `BuildSnapshot` identity and snapshot input summaries;
- the phase registry descriptor table;
- phase 0 `BuildPlan` data from `mizar-build`;
- task graph/work-unit identity supplied by `mizar-build`;
- sealed parent output handles and diagnostic batches required by each phase;
- verifier configuration and build profile identity.

Derived queries are keyed by snapshot id, phase descriptor, work unit, service
schema version, input-handle hashes, and owner dependency fingerprints. They may
compute:

- the service cache-key intent;
- the cache-key intent and query observation used by the later `mizar-cache`
  owner seam;
- the phase execution result when a later cache lookup misses or is disabled;
- the scheduler-visible output references and diagnostic batches for the task.

Cancellation is not a semantic input to cached query identity. A cancellation
token may stop execution at safe checkpoints and return `Cancelled`, but it
must not make a completed semantic output differ. Obsolete query results for an
older snapshot/session may be retained for diagnostics or cache validation, but
they must pass the request module's publication guard before any current
publication and must pass `mizar-cache` validation before reuse in another
snapshot.

## Scheduler And Cache Seams

The registry/driver layer submits work to `mizar-build`; it does not duplicate
scheduler semantics. The scheduler owns readiness, dependency blocking, resource
admission, cancellation propagation, terminal task states, and canonical
collation. The registry provides:

- registered service descriptors that explain which service can satisfy each
  task's phase span;
- query adapters that execute service work only when the scheduler asks for a
  task execution;
- caller-supplied cache decisions in the shape accepted by the
  `mizar-build` cache seam, after `mizar-cache` has validated them.

A validated cache hit is an execution skip and completed scheduler dependency.
It is not proof evidence, semantic acceptance, trusted status, or artifact
publication authority.

## Diagnostics, Artifacts, And LSP Boundaries

Phase services may emit diagnostics through `mizar-diagnostics` producer sinks.
The registry must preserve structured records and producer identity and must
not treat message text as identity.

Phase services may seal outputs through `mizar-ir` handles. The registry must
not expose raw mutable IR values after publication and must not construct
`mizar-ir` lineage, projection, or cache-adapter authority locally.

Artifact commit remains a handoff to the artifact/proof/producer owners. If the
real artifact publication token or producer output is unavailable, the registry
records `external_dependency_gap` and stops before publication.

LSP protocol conversion remains in `mizar-lsp`. The registry may surface
protocol-agnostic phase results and diagnostic readiness to the later event
stream, but it must not create LSP diagnostics, code actions, document edits,
JSON-RPC responses, or editor publication decisions.

## Tests Required By Implementation

Task 5 must add focused tests for:

- deterministic registration independent of input order;
- duplicate coverage rejection for every architecture phase;
- missing real owner seams reported as classified gaps without synthetic
  outputs;
- `PhaseService::cache_key` purity with test-only fixture services;
- a positive guard that the registry creates or owns the driver-local
  salsa/query boundary and that service `cache_key`/`execute` calls are mediated
  by registry query adapters;
- salsa/query-boundary source scans showing no syntax/parser/phase owner crate
  gains a driver or salsa dependency through the registry. The scanned owner set
  is `mizar-lexer`, `mizar-syntax`, `mizar-parser`, `mizar-frontend`,
  `mizar-resolve`, `mizar-checker`, `mizar-core`, `mizar-vc`, `mizar-atp`,
  `mizar-kernel`, `mizar-proof`, `mizar-artifact`, and `mizar-doc`;
- boundary guards proving the registry does not construct cache compatibility,
  proof acceptance, artifact publication tokens, LSP payloads, or scheduler
  readiness semantics.
