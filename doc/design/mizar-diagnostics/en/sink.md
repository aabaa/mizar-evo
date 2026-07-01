# Diagnostic Sink

> Canonical language: English. Japanese companion:
> [../ja/sink.md](../ja/sink.md).

## Purpose

This document specifies the producer-side diagnostic sink owned by
`mizar-diagnostics`. The sink is the narrow boundary through which compiler
phases submit validated `DiagnosticDraft` values for later aggregation. It
collects drafts and preserves producer payloads; it does not render diagnostics,
assign record handles, deduplicate, publish LSP diagnostics, mutate artifacts,
or decide phase/proof/build status.

## Scope

The sink owns:

- accepting already validated `DiagnosticDraft` values;
- grouping drafts by producer scope for later aggregation;
- preserving draft order within that producer scope for debugging and trace
  comparison;
- exposing collected drafts to the aggregator without changing their fields;
- rejecting sink misuse such as emitting drafts after the sink is sealed.

The sink does not own:

- registry code allocation or message identity;
- draft construction rules already owned by `failure_record.md`;
- deterministic build-wide ordering, deduplication, or `BuildDiagnosticIndex`;
- CLI formatting, LSP protocol conversion, JSON-RPC payloads, or code actions;
- proof acceptance, trusted status, kernel acceptance, or phase success;
- cache writes, artifact mutation, or driver session orchestration.

## Producer Scope

A producer receives a sink bound to one producer scope:

```rust
struct DiagnosticProducerScope {
    phase: PipelinePhase,
    source_snapshot: BuildSnapshotId,
    producer_name: &'static str,
}
```

`phase` is provenance metadata and must match each emitted draft's `phase`.
`source_snapshot` is the snapshot observed by the producer and must match each
emitted draft's `source_snapshot`. `producer_name` is stable debug metadata for
tests and trace output; it is not user-facing identity.

Producer scopes are local to one phase invocation. Cross-phase or build-wide
aggregation is task 9 behavior, not sink behavior.

## API Shape

Task 7 implementation should expose this conceptual API:

```rust
trait DiagnosticSink {
    fn scope(&self) -> DiagnosticProducerScope;
    fn emit(&mut self, draft: DiagnosticDraft) -> Result<(), DiagnosticSinkError>;
    fn seal(&mut self) -> DiagnosticBatch;
}

struct DiagnosticBatch {
    scope: DiagnosticProducerScope,
    drafts: Vec<DiagnosticDraft>,
}
```

The concrete Rust API may use owned structs instead of a trait if that better
matches the crate style. The behavior must remain the same: producers append
drafts, then hand one immutable batch to aggregation.

`emit` validates only sink-level invariants: the sink is open, the draft phase
matches the scope, and the draft source snapshot matches the scope. It does not
revalidate registry descriptors beyond what `DiagnosticDraft` construction
already enforced.

Failed `emit` calls are non-mutating. If a draft is rejected for phase or
snapshot mismatch, that draft is not collected, already collected drafts remain
unchanged, and the sink remains open for later valid drafts unless it had
already been sealed.

`seal` consumes or closes the sink and returns collected drafts unchanged. After
sealing, further emission must fail or be impossible by ownership.

## Emission Rules

Producers must:

- construct drafts through the `failure_record` APIs so `DiagnosticCode`,
  spans, details, fixes, explanations, and freshness inputs are validated;
- emit only drafts whose `phase` and `source_snapshot` match the sink scope;
- keep `stable_detail_key` and structured details independent of message text;
- include compact structured references rather than large traces;
- treat sink order as local production order only.

Producers must not:

- attach CLI-rendered strings, terminal styles, line excerpts, or underlines;
- attach LSP ranges, UTF-16 offsets, JSON-RPC payloads, or code-action objects;
- emit artifact mutation instructions, cache writes, driver events, or scheduler
  commands through the sink;
- publish stale diagnostics as current output;
- infer proof acceptance, trusted status, kernel acceptance, or phase success
  from diagnostic severity or category.

## Collection Semantics

`DiagnosticBatch` is immutable once sealed. It preserves drafts byte-for-byte at
the field level. The sink must not:

- sort drafts;
- deduplicate drafts;
- assign `DiagnosticId` or `DiagnosticHandle`;
- rewrite messages, notes, details, fixes, or explanation handles;
- drop drafts because another draft looks similar.

The aggregator may later normalize, deduplicate, sort, assign handles, reject
stale publication, and produce `BuildDiagnosticIndex` values. Tests for task 7
should confirm the sink preserves input drafts exactly and keeps local
production order without treating it as global publication order.

## Error Handling

Sink errors are internal producer-boundary errors. They may be reported in tests
or developer diagnostics, but they are not normal user diagnostics and must not
allocate public `DiagnosticCode` values unless a later registry task explicitly
does so.

Initial error cases:

| Error | Rule |
|---|---|
| `SinkSealed` | Emission after sealing is rejected or impossible. |
| `PhaseMismatch` | Draft `phase` differs from scope `phase`. |
| `SnapshotMismatch` | Draft `source_snapshot` differs from scope `source_snapshot`. |

These errors protect the producer boundary only. They do not decide whether a
phase failed; phase status remains owned by the scheduler/driver layer.
`PhaseMismatch` and `SnapshotMismatch` are recoverable producer-boundary errors:
they must not seal the sink or remove previously collected drafts.

## Determinism

The sink must expose deterministic debug data for tests as a canonical
`DiagnosticBatch::debug_snapshot()` string. The string uses LF line endings, no
color, no localized field names, and this field order:

1. `kind=batch`.
2. `phase`.
3. `source_snapshot`, rendered like `DiagnosticDraft::debug_snapshot()`.
4. `producer_name`, escaped with Rust debug-string escaping.
5. `draft_count`.
6. `draft[0]`, `draft[1]`, ... in local emission order.

Each `draft[n]` entry embeds the corresponding `DiagnosticDraft::debug_snapshot()`
with its trailing newline removed and internal newlines escaped as `\n`.
Empty batches render `draft_count=0` and no `draft[n]` lines.

This debug data is not CLI rendering. It must not include memory addresses,
thread IDs, map iteration order, localized field names, or process-local
ordering other than the explicit local emission ordinal.

## Boundary Rules

- The sink collects drafts only; it does not create `DiagnosticRecord` values.
- The sink's local order is not publication order.
- The sink can reject mismatched producer metadata, but it cannot convert a
  failure into success or success into failure.
- The sink has no dependency on `mizar-driver`, `mizar-lsp`, `mizar-artifact`,
  or existing resolver/frontend diagnostic types.
- Existing lexer/frontend/parser/resolver diagnostic migration remains deferred
  until a real consumer adoption seam exists.
