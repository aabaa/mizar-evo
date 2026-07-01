# Failure Records

> Canonical language: English. Japanese companion:
> [../ja/failure_record.md](../ja/failure_record.md).

## Purpose

This document specifies the diagnostic draft and diagnostic record model owned by
`mizar-diagnostics`. Records carry stable diagnostic identity, source identity,
failure classification, structured details, and compact projection payloads. They
do not decide proof acceptance, trusted status, kernel acceptance, cache reuse,
artifact publication, LSP protocol shapes, CLI formatting, or driver session
orchestration.

The registry owns `DiagnosticCode` identity. A record references a code and
registry descriptor metadata; it does not infer identity from message text.

## Scope And Lifecycle

There are two record stages:

| Stage | Owner | Meaning |
|---|---|---|
| `DiagnosticDraft` | Phase producer through `DiagnosticSink` | Producer-owned draft before snapshot-wide deduplication, ordering, and handle assignment. |
| `DiagnosticRecord` | Aggregator | Immutable published record with a deterministic handle, freshness, and normalized metadata. |

Drafts may be collected before the build has a coherent publication boundary.
Records are published only after aggregation has validated source snapshot
freshness and assigned deterministic identifiers. Obsolete snapshot diagnostics
must not be published as current records.

## Shared Fields

Both drafts and records carry these fields:

| Field | Type | Required | Rule |
|---|---|---:|---|
| `code` | `DiagnosticCode` | yes | Stable identity. Must exist in the registry and must not be retired for new current diagnostics. |
| `phase` | `PipelinePhase` | yes | Phase that produced the diagnostic. This is ordering/provenance metadata, not phase-status authority. |
| `category` | `FailureCategory` | yes | Stable machine-readable failure class. |
| `stable_detail_key` | `String` | yes | Deterministic key for deduplication and sorting. It must not contain localized text. |
| `message` | `String` | yes | Human-facing primary message. It may change across versions and is never identity. |
| `primary_span` | `DiagnosticSpan` | yes | Main location. Must reference a `SourceId`. |
| `secondary_spans` | `Vec<DiagnosticSpan>` | yes, may be empty | Supporting locations, sorted by producer when naturally ordered and normalized by the aggregator. |
| `notes` | `Vec<DiagnosticNote>` | yes, may be empty | Human-facing note/help text plus optional source anchors. |
| `details` | `DiagnosticDetails` | yes, may be empty | Machine-readable payload map. |
| `fixes` | `Vec<FixSuggestion>` | yes, may be empty | Structured advisory fix suggestions. They are normalized by canonical payload and never apply edits. |
| `explanation` | `Option<ExplanationHandle>` | optional | Structured lazy explanation handle with bounded preview metadata. |

The message, notes, summaries, rendered labels, and localized text are
presentation payloads. Tools and consumers must key on `DiagnosticCode` and
structured fields, not on those strings.

## DiagnosticDraft

```rust
struct DiagnosticDraft {
    source_snapshot: BuildSnapshotId,
    code: DiagnosticCode,
    phase: PipelinePhase,
    category: FailureCategory,
    stable_detail_key: String,
    message: String,
    primary_span: DiagnosticSpan,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    details: DiagnosticDetails,
    fixes: Vec<FixSuggestion>,
    explanation: Option<ExplanationHandle>,
}
```

Draft rules:

- Producers must already ground every span in a `SourceId`.
- `source_snapshot` is the build snapshot observed by the producer. Producers
  must provide it, but they do not decide whether the draft is current for
  publication.
- Span freshness on a draft is relative to `source_snapshot`, not to the future
  publication snapshot. Draft spans normally use `SpanFreshness::Current`.
  Producers may mark related spans stale or historical only when the span was
  copied from older context; the aggregator still owns publication freshness.
- Producers must not attach CLI-rendered strings, LSP ranges, JSON-RPC payloads,
  artifact mutation instructions, or driver events.
- Producers may attach structured details only when each key has stable meaning
  independent of human wording.
- If a producer attaches an `ExplanationHandle` with `required_snapshot`, that
  snapshot must match the draft `source_snapshot`.
- If an attached `ExplanationHandle` uses a diagnostic subject, its
  `DiagnosticCode` and stable detail key must match the containing diagnostic.

## DiagnosticRecord

```rust
struct DiagnosticRecord {
    handle: DiagnosticHandle,
    code: DiagnosticCode,
    semantic_name: String,
    severity: DiagnosticSeverity,
    phase: PipelinePhase,
    category: FailureCategory,
    stable_detail_key: String,
    message: String,
    primary_span: DiagnosticSpan,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    details: DiagnosticDetails,
    fixes: Vec<FixSuggestion>,
    explanation: Option<ExplanationHandle>,
    related: Vec<DiagnosticHandle>,
    freshness: DiagnosticFreshness,
}
```

Record rules:

- `semantic_name` and `severity` are copied from the validated registry
  descriptor unless a later spec explicitly defines a safe severity override.
- `handle` is assigned by the aggregator and is meaningful only within its
  snapshot.
- `freshness` records whether the source snapshot used by the diagnostic matches
  the publication boundary.
- Published current records must not use obsolete source snapshots.
- `related` links records within the same build snapshot. Cross-snapshot
  relationships must be represented as explanation or artifact references, not
  as direct handles.

## Construction And Round Trips

Task 5 record round-trips are in-memory structural round-trips, not durable
serialization. A draft constructor must validate its code against a
`DiagnosticRegistry` and reject unknown or retired codes. A record constructor
must look up the draft code in a validated registry, preserve every shared draft
field exactly, copy `semantic_name` and `severity` from the descriptor found by
that lookup, and expose the same values through cloning, equality, accessors,
and deterministic debug snapshots. `SourceId` and `BuildSnapshotId` remain
session-local identities for Task 5 record APIs; no Task 5 API promises JSON,
LSP, artifact, or cross-session serialization.

The aggregator supplies the publication snapshot when converting drafts to
records. A draft whose `source_snapshot` equals the publication snapshot may
produce `DiagnosticFreshness::Current`. A draft whose `source_snapshot` differs
must produce `Stale` or be withheld from current publication by the aggregator.
Task 5 constructors may represent stale and historical records for tests and
future consumers, but they must not publish them as current build output.

## Handles And Freshness

```rust
struct DiagnosticHandle {
    snapshot: BuildSnapshotId,
    id: DiagnosticId,
}

struct DiagnosticId(u64);

enum DiagnosticFreshness {
    Current {
        source_snapshot: BuildSnapshotId,
    },
    Stale {
        source_snapshot: BuildSnapshotId,
        current_snapshot: BuildSnapshotId,
        reason: StaleDiagnosticReason,
    },
    Historical {
        source_snapshot: BuildSnapshotId,
        artifact_hash: Option<String>,
    },
}

enum StaleDiagnosticReason {
    SourceEdited,
    SourceRemoved,
    SnapshotSuperseded,
    ProducerCacheObsolete,
    HistoricalReplay,
}
```

`DiagnosticId` is deterministic within one `BuildSnapshotId`; it is not globally
meaningful. The aggregator derives it from source identity, diagnostic code,
phase, primary span, stable detail key, normalized details, canonical fix
payloads, explanation handle identity, and the deduplicated ordinal.

`Current` records are eligible for CLI output, artifact projection, and semantic
LSP publication by the owning consumer. `Stale` records may be shown by an editor
overlay only when the LSP layer marks them stale and suppresses unsafe edits.
`Historical` records are for artifact/cache/log reads and must not be treated as
current build output.

`Current { source_snapshot }` is valid only when `source_snapshot` equals the
publication snapshot used for the record handle. `Stale { source_snapshot,
current_snapshot, .. }` requires those two snapshots to differ. `Historical`
records do not have a current publication snapshot and must not be included in a
current `BuildDiagnosticIndex`.

## Source Spans

```rust
struct DiagnosticSpan {
    range: SourceRange,
    role: DiagnosticSpanRole,
    label: Option<String>,
    freshness: SpanFreshness,
    zero_width: Option<ZeroWidthSpanIntent>,
}

enum DiagnosticSpanRole {
    Primary,
    Secondary,
    DefinitionSite,
    Related,
}

enum SpanFreshness {
    Current,
    Stale { reason: StaleDiagnosticReason },
    Historical,
}

enum ZeroWidthSpanIntent {
    Eof,
    InsertionPoint,
}
```

`SourceRange` is the `mizar-session` byte range with `SourceId`, `start`, and
`end`. Every diagnostic span must contain a `SourceId`. Line/column conversion,
UTF-16 offsets, context snippets, and rendered underlines are projections owned
by render or LSP consumers.

Task 5 span constructors must enforce `start <= end`, `primary_span.role ==
Primary`, and no `secondary_spans` entry with `role == Primary`. They do not
validate file length or line-map membership; that remains a source-map consumer
responsibility. Zero-width ranges are allowed only when `zero_width` is
`Some(Eof)` or `Some(InsertionPoint)`. Non-zero ranges must use `zero_width ==
None`.

## FailureCategory

`FailureCategory` is stable, machine-readable classification for propagation,
ordering, and regression tests. Initial categories are:

| Category | Meaning |
|---|---|
| `parse_error` | Lexical, syntactic, or parse-recovery failure. |
| `resolve_error` | Name, import, namespace, or symbol resolution failure. |
| `type_error` | Type, attribute, mode, or registration mismatch. |
| `overload_ambiguity` | Overload/template ambiguity or no viable overload. |
| `cluster_loop` | Cluster, attribute, or registration cycle. |
| `atp_timeout` | ATP timeout or resource exhaustion that leaves an obligation unresolved. |
| `certificate_rejection` | Malformed, unsupported, or policy-rejected evidence envelope. |
| `kernel_rejection` | Kernel-level evidence or replay rejection. |
| `logic_failure` | Logical inconsistency or VC failure not classified above. |
| `compatibility_warning` | Compatibility or packaging warning. |
| `informational` | Informational display diagnostic after `I` codes are allocated. |
| `internal_invariant` | Developer-mode internal diagnostic that is not normal user output. |

Each category is descriptive metadata. It cannot turn a failure into success and
cannot downgrade proof/evidence/kernel rejection. Specific rejection reasons,
such as `malformed_certificate`, `unsupported_certificate_format`,
`invalid_substitution`, or `invalid_sat_refutation`, must be stored in structured
details.

## Structured Details

```rust
struct DiagnosticDetails {
    entries: BTreeMap<String, DiagnosticDetailValue>,
}

enum DiagnosticDetailValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Code(DiagnosticCode),
    Source(SourceRange),
    List(Vec<DiagnosticDetailValue>),
}
```

`stable_detail_key` and detail-map keys must match this ASCII grammar:

```text
detail_key = segment ("." segment)*
segment = [a-z][a-z0-9]* ("_" [a-z0-9]+)*
```

Examples: `proof.rejection_reason`,
`declaration_symbol.symbol.duplicate_declaration`, and
`resolve.candidate_count` are valid. Empty segments, leading/trailing dots,
uppercase, leading/trailing/doubled underscores, hyphens, whitespace, and
non-ASCII characters are invalid.

Detail values must be deterministically comparable and serializable for
debug/test snapshots. Canonical value ordering uses variant order `Boolean`,
`Integer`, `String`, `Code`, `Source`, then `List`, with natural ordering inside
each variant and lexicographic list comparison. `Source` values are ordered by
`SourceId` debug identity, then `start`, then `end`. Details may include compact
previews and references; they must not embed large traces, LSP payloads, terminal
escape sequences, or localized prose.

Large proof traces, solver logs, candidate lists, and explanation bodies belong
in artifacts, cache-backed query data, or explanation payload files. Records may
store handles to them, not copies.

## Notes

```rust
struct DiagnosticNote {
    kind: DiagnosticNoteKind,
    message: String,
    span: Option<DiagnosticSpan>,
}

enum DiagnosticNoteKind {
    Note,
    Help,
    Cause,
    Related,
}
```

Notes are human-facing projections attached to the record. A `help` note may be
rendered as `help:` by CLI output, but the note itself is not a structured edit.
Structured edits belong to `FixSuggestion` payloads and `fix.md`.

## Attachment Slots

`FixSuggestion` values are structured advisory payloads, not placeholder
adapters. They carry stable suggestion identity, optional producer identity,
applicability, safety, edits, command refs, and snapshot/hash preconditions as
specified by `fix.md`; they do not apply edits, create LSP code actions, execute
commands, or mutate artifacts. `ExplanationHandle` values are structured lazy
references with bounded preview metadata. These attachment identities use the
same ASCII grammar as `stable_detail_key` so aggregation and debug snapshots can
compare them deterministically, and they must not reinterpret human-facing text
as identity.

## Deterministic Debug Rendering

Task 5 implementation must provide a deterministic debug rendering for drafts
and records. It is a test/debug format, not CLI rendering. Implementations
should expose it as a canonical debug snapshot string with LF line endings, no
color, no localized field names, and this field order:

1. `kind` (`draft` or `record`).
2. `handle` (`none` for drafts).
3. `code`.
4. `semantic_name` (`none` for drafts).
5. `severity` (`none` for drafts).
6. `phase`.
7. `category`.
8. `stable_detail_key`.
9. `message`.
10. `source_snapshot`.
11. `freshness` (`draft` for drafts).
12. `primary`.
13. `secondary`.
14. `notes`.
15. `details`.
16. `fixes`.
17. `explanation`.
18. `related`.

Strings are escaped with Rust debug-string escaping. Spans render as
`<SourceId debug>:<start>..<end>:<role>:<span freshness>:<zero-width
intent>:<label>`. `SourceId debug` is a session-local test/debug identity, not a
published schema string. Detail entries render in key order, and values render
with the canonical ordering and representation from [Structured Details](#structured-details).
Empty lists render as `[]`, absent optional fields as `none`, and records render
their complete freshness state and related handles.

It must not include nondeterministic map iteration order, localized strings as
identity, memory addresses, or process-local ordering.

## Boundary Rules

- Records may describe proof, kernel, cache, artifact, driver, or LSP facts, but
  they do not decide those facts.
- A proof or kernel rejection record does not decide proof status; the proof or
  kernel component owns that decision.
- A freshness state does not mutate snapshots or artifacts; aggregation and
  consumer layers decide publication.
- Records store `SourceRange`, not LSP UTF-16 positions.
- Records store compact structured details, not artifact manifests or cache
  mutation instructions.
