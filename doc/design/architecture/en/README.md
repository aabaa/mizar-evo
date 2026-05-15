# Architecture Design Specifications

> Canonical language: English. Japanese companion: [../ja/README.md](../ja/README.md).

This directory contains **cross-cutting internal design documents** that define the boundaries, protocols, and design decisions spanning multiple modules or crates.

## Purpose

While module-level specs (`doc/design/<crate>/<module>.md`) describe individual Rust source files, architecture specs address questions that cannot be answered at the single-module level:

- **Where** is the boundary between subsystems? (e.g., kernel vs. ATP)
- **What** protocols connect subsystems? (e.g., TPTP, SMT-LIB)
- **How** are external tools integrated? (e.g., process management, timeouts)
- **Why** was a particular design chosen over alternatives?

## Index

Architecture documents are numbered by reading order and design dependency, not strictly by pipeline phase. Missing numbered documents are planned slots.

| Document | Pipeline Phase | Description | Status |
|---|---:|---|---|
| [00.pipeline_overview.md](./00.pipeline_overview.md) | All | End-to-end pipeline from source files to verified artifacts | Draft |
| [01.ir_layers.md](./01.ir_layers.md) | All | IR ownership boundaries and stability rules across pipeline phases | Draft |
| [02.source_and_frontend.md](./02.source_and_frontend.md) | 1-3 | Source loading, preprocessing, lexing, and parsing boundaries | Draft |
| [03.module_and_symbol_resolution.md](./03.module_and_symbol_resolution.md) | 0, 4-5 | Package, module, namespace, label, and symbol table resolution | Draft |
| `04.type_and_registration_resolution.md` | 6-7 | Type checking, cluster database, and resolution traces | Planned |
| `05.overload_resolution.md` | 8 | Candidate selection, subsumption DAG, and `qua` insertion | Planned |
| `06.elaboration_and_core_ir.md` | 9 | Lowering surface language into core logic | Planned |
| `07.vc_generation.md` | 10-12 | Algorithm verification preparation and obligation generation | Planned |
| [08.reasoning_boundary.md](./08.reasoning_boundary.md) | 12-14 | Reasoning responsibility split between Mizar, ATP backends, and the kernel | Draft |
| [09.atp_interface_protocol.md](./09.atp_interface_protocol.md) | 13 | ATP problem formats and encoding strategy | Draft |
| [10.atp_backend_integration.md](./10.atp_backend_integration.md) | 13 | External ATP process execution, timeout handling, and certificate collection | Draft |
| `11.artifact_and_incremental_build.md` | 15 | Artifact schema, cache update, and reproducibility | Planned |
| `12.diagnostics_and_lsp.md` | All, 15 | Diagnostics, metadata, and IDE integration | Planned |
| `13.documentation_and_extraction.md` | 16 | Documentation generation and code extraction | Planned |

`00.pipeline_overview.md` is the parent document for this directory. Other architecture documents should state which pipeline phase(s) they refine and should link back to the overview in their Context section.

## Document Template

Each architecture document should follow this structure:

```markdown
# Architecture: <Title>

## Purpose
The architectural problem this document addresses.

## Context
References to related external specs and architecture documents.

## Design Decisions

### Alternatives Considered
Comparison of approaches and their trade-offs.

### Adopted Approach
The chosen design and its rationale.

## Interface Definitions
Boundaries, APIs, and data formats between subsystems.

## Affected Modules
List of module-level specs and source files that implement this design.
- `doc/design/<crate>/<module>.md` → `crates/<crate>/src/<module>.rs`

## Constraints and Assumptions
Performance requirements, security considerations, compatibility, etc.
```

## Relationship to Other Documentation

| Layer | Directory | Granularity | Audience |
|---|---|---|---|
| External Spec | `doc/spec/en/` | Language features | Users |
| **Architecture** | **`doc/design/architecture/`** | **Cross-cutting subsystems** | **Developers** |
| Module Spec | `doc/design/<crate>/` | Individual files (1:1) | Developers |
