# Internal Design Specifications

> Canonical language: English. Japanese companion: [../ja/README.md](../ja/README.md).

This directory contains **internal subsystem design documents** that refine the cross-cutting architecture specifications into compiler modules, APIs, data structures, and execution contracts.

## Purpose

Architecture documents in [`doc/design/architecture/en/`](../../architecture/en/README.md) define phase boundaries and subsystem responsibilities. Internal design documents describe how those boundaries are implemented without becoming 1:1 module specifications.

Internal design documents answer questions such as:

- Which service owns a pipeline phase transition?
- What data structures are shared between compiler subsystems?
- Which APIs are synchronous, asynchronous, or event-based?
- Where are cache, diagnostics, cancellation, and artifact commit decisions enforced?
- How do batch, watch, and LSP builds share the same compiler driver?

## Index

Documents are numbered by implementation dependency order. Missing numbered documents are planned slots.

| Document | Pipeline Phase | Description | Status |
|---|---:|---|---|
| [00.internal_overview.md](./00.internal_overview.md) | All | Internal design scope, crate/service boundaries, and relationship to architecture documents | Draft |
| [01.compiler_driver_and_pipeline_scheduler.md](./01.compiler_driver_and_pipeline_scheduler.md) | 0-16 | Compiler driver, task graph scheduler, phase services, cancellation, cache lookup, diagnostics, and artifact commit orchestration | Draft |
| [02.artifact_store_cache_key_and_manifest.md](./02.artifact_store_cache_key_and_manifest.md) | 15 | Artifact store, cache key construction, manifest transactions, and reproducible write protocol | Draft |
| [03.diagnostics_model_and_lsp_bridge.md](./03.diagnostics_model_and_lsp_bridge.md) | All, 15 | Diagnostic registry, aggregation, explanation handles, LSP snapshot bridge, and editor freshness model | Draft |
| [04.atp_portfolio_and_kernel_check_integration.md](./04.atp_portfolio_and_kernel_check_integration.md) | 13-14 | ATP portfolio execution, backend evidence selection, proof witness storage, and kernel check scheduling | Draft |
| `05.documentation_extraction.md` | 16 | Documentation extraction inputs, render model, code extraction boundary, and artifact consumers | Planned |

## Document Template

Each internal design document should follow this structure:

```markdown
# Internal Design: <Title>

## Purpose
The implementation problem this document addresses.

## Context
References to architecture and spec documents refined by this design.

## Responsibilities
Concrete ownership boundaries between internal services, modules, and data structures.

## Data Model
Important internal types, identity rules, and invariants.

## Control Flow
How requests, phase outputs, diagnostics, cache records, and artifacts move through the subsystem.

## API Sketch
Traits, structs, events, and service calls used between modules.

## Error Handling
Failure, cancellation, recovery, and diagnostic publication rules.

## Affected Modules
Expected module-level specs and source files that will implement the design.

## Constraints and Assumptions
Performance, reproducibility, compatibility, and trust-boundary constraints.
```

## Relationship to Other Documentation

| Layer | Directory | Granularity | Audience |
|---|---|---|---|
| External Spec | `doc/spec/en/` | Language features and user-visible behavior | Users |
| Architecture | `doc/design/architecture/en/` | Cross-cutting subsystem boundaries | Developers |
| **Internal Design** | **`doc/design/internal/en/`** | **Subsystem APIs, data structures, and execution contracts** | **Compiler developers** |
| Module Spec | `doc/design/<crate>/` | Individual files (1:1) | Developers |
