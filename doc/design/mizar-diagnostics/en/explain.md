# Lazy Explanation Handles

> Canonical language: English. Japanese companion:
> [../ja/explain.md](../ja/explain.md).

## Purpose

This document specifies lazy explanation handles owned by
`mizar-diagnostics`. Explanation data gives users bounded context for type
inference, cluster resolution, overload resolution, proof failure, verification
conditions, and similar diagnostics without embedding large traces in normal
diagnostic records.

An explanation handle is a compact, deterministic reference attached to a
`DiagnosticRecord`. The handle may include a bounded preview. Full explanation
traces stay in artifacts, cache-backed query data, dedicated explanation files,
or a later query service.

Human preview text is presentation. Tools key on `DiagnosticCode`,
structured diagnostic fields, and stable explanation handle identity, not on
preview wording.

## Scope

The explain module owns:

- stable `ExplanationHandleId` values referenced by diagnostic records;
- explanation kind and subject metadata;
- bounded previews suitable for CLI and editor display;
- source, snapshot, artifact/hash, and query preconditions needed to resolve a
  handle later;
- deterministic ordering and debug snapshots for explanation handles;
- lazy resolution status values such as available, missing, stale, and
  truncated.

The explain module does not own:

- proof acceptance, trusted status, kernel acceptance, or phase success;
- generation of type/proof/VC traces by phase services;
- artifact creation, artifact mutation, cache validity, or cache eviction;
- LSP `mizar/explain` request routing, JSON-RPC payloads, or editor protocol
  conversion;
- driver session orchestration, build scheduling, or snapshot publication;
- source loading, path normalization, line maps, or UTF-16 conversion.

## Data Model

Task 15 implements structured explanation payloads equivalent to:

```rust
struct ExplanationHandle {
    id: ExplanationHandleId,
    kind: ExplanationKind,
    subject: ExplanationSubject,
    source: ExplanationSourceRef,
    required_snapshot: Option<BuildSnapshotId>,
    required_artifact_hash: Option<Hash>,
    summary_hash: Option<Hash>,
    preview: Option<ExplanationPreview>,
}

enum ExplanationSubject {
    Diagnostic { code: DiagnosticCode, stable_detail_key: String },
    Expression(String),
    VerificationCondition(String),
    SourceRange(SourceRange),
    PhaseLocal { phase: PipelinePhase, key: String },
}

enum ExplanationSourceRef {
    PreviewOnly,
    Artifact { path: String, content_hash: Hash },
    CacheRecord { cache_key: String, content_hash: Option<Hash> },
    QueryService { service_key: String, query_key: String },
}

struct ExplanationPreview {
    format: ExplanationPreviewFormat,
    text: String,
    truncated: bool,
    byte_len: usize,
    line_count: usize,
}
```

`Diagnostic` subjects use pre-publication stable diagnostic fields, not
`DiagnosticHandle`, because handles are assigned only after aggregation.
`Expression` and `VerificationCondition` subjects are opaque structured keys at
this crate boundary. The owning phase or later query service defines their
meaning. `mizar-diagnostics` stores and compares the keys but does not interpret
them as semantic authority.
When a `Diagnostic` subject is attached to a diagnostic draft, its code and
stable detail key must match the containing draft.

`ExplanationHandleId` is assigned before publication from an explicit producer
key or deterministic producer-local ordinal. It must not be derived from the
human preview or localized wording. Published records may project the containing
`DiagnosticHandle` next to the explanation handle for convenience, but that
back-reference is not the explanation handle identity.

`summary_hash` is a structured integrity and identity field for the canonical
bounded summary or backing explanation descriptor. It is not a hash of localized
preview text. If present, task 15 must include it in canonical explanation
identity and use it to detect mismatched backing explanation data.

## Explanation Kinds

Initial explanation kinds:

| Kind | Meaning |
|---|---|
| `TypeInference` | Type, mode, attribute, or registration reasoning. |
| `ClusterResolution` | Attribute/cluster search or loop explanation. |
| `OverloadResolution` | Candidate set, selected policy, or ambiguity detail. |
| `ProofFailure` | Proof obligation, ATP, or evidence rejection context. |
| `VerificationCondition` | VC generation/checking context. |
| `AlgorithmTrace` | Algorithm-contract checking context. |
| `DiagnosticContext` | General diagnostic context not classified above. |
| `Internal` | Developer-only internal explanation data. |

Kinds classify explanation payloads only. They do not decide whether a proof,
kernel replay, or phase succeeded.

## Preview Bounds

Previews are optional and bounded. The implementation enforces constants for:

- maximum preview bytes;
- maximum preview lines;
- deterministic truncation markers.

When a preview exceeds a bound, the stored preview must be truncated
deterministically and marked `truncated=true`. Large traces must not be embedded
in `DiagnosticRecord`, `BuildDiagnosticIndex`, CLI output, or normal artifact
diagnostic projections.

## Lazy Resolution

Task 15 exposes a store that resolves handles lazily and returns bounded results
equivalent to:

```rust
enum ExplanationResolution {
    Available(ExplanationPayload),
    Missing { reason: ExplanationMissingReason },
    Stale { source_snapshot: BuildSnapshotId, current_snapshot: BuildSnapshotId },
    Unavailable { reason: String },
}
```

Resolution checks the handle preconditions that this crate can validate without
owning backing storage. A snapshot-bound handle is stale outside
`required_snapshot`. Artifact hash and cache/query preconditions are preserved
in the handle identity for the backing owner to validate; when no bounded
payload has been registered for that canonical handle, resolution degrades to
`Missing`. A cache-backed handle may accelerate a query, but missing cache data
must degrade to `Missing` or `Unavailable`; it must not change diagnostic
identity or proof acceptance. When `summary_hash` is present and registered
backing data exposes a comparable summary hash, a mismatch degrades to
`Unavailable` rather than silently returning different explanation data.

The store returns bounded explanation payloads only. It does not publish LSP
responses, schedule builds, mutate artifacts, or validate proof status.

## Attachment And Deduplication

Explanation handles attach to diagnostic records as compact structured payloads
or stable handles. They must preserve:

- the diagnostic code they belong to;
- the containing published diagnostic handle as a projection when one exists;
- stable explanation handle id;
- kind and subject;
- source reference;
- snapshot/artifact/hash preconditions;
- summary hash, when present;
- optional bounded preview metadata.

Draft construction rejects explanation handles whose snapshot precondition
points at a different snapshot from the draft source snapshot, or whose
diagnostic subject names a different code or stable detail key. This prevents
stale or foreign diagnostic explanations from being published as current
explanation data.

Aggregation deduplication includes canonical explanation identity in diagnostic
identity. Canonical explanation identity consists of handle id, kind, subject,
source reference, snapshot/artifact/hash preconditions, and `summary_hash` when
present. Preview text, localized text, rendered `explain:` lines, and full
explanation payloads must not be deduplication keys.

## Debug Snapshot

Task 15 exposes deterministic explanation debug snapshots with:

1. `kind=explanation`.
2. `id`.
3. `diagnostic`.
4. `explanation_kind`.
5. `subject`.
6. `source`.
7. snapshot/artifact/hash preconditions.
8. `summary_hash`.
9. bounded preview metadata.

Snapshots are test/debug data, not CLI rendering and not LSP responses. They
must not include memory addresses, map iteration order, localized field names,
process-local ordering, or full unbounded traces.

## Public Enum Compatibility

Task 18 marks explain-owned public enums as `#[non_exhaustive]` for downstream
forward compatibility:

- `ExplanationKind`;
- `ExplanationSubject`;
- `ExplanationSourceRef`;
- `ExplanationPreviewFormat`;
- `ExplanationResolution`;
- `ExplanationMissingReason`;
- `ExplanationError`.

Future variants must preserve the lazy-reference boundary: explanations may add
new handle kinds, subjects, sources, or resolution states, but they must not move
proof status, artifact mutation, LSP routing, or driver orchestration into this
crate.

## Boundary Rules

- Explanation handles are references, not embedded proof/type/VC traces.
- A missing explanation payload must not suppress the diagnostic record.
- A stale handle must not be published as current explanation data.
- Cache-backed explanations are optional and never establish trusted status.
- CLI rendering may show bounded previews or `explain:` references, but it does
  not resolve large traces.
- LSP request conversion, stale-handle retry policy, and JSON response shaping
  belong to `mizar-lsp` or the driver layer.
- Artifact-backed explanation files are owned by artifact/cache components;
  `mizar-diagnostics` stores references and validates shape only.
