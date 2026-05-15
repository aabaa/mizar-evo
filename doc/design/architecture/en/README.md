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

| Document | Pipeline Phase | Description | Status |
|---|---:|---|---|
| [00.pipeline_overview.md](./00.pipeline_overview.md) | All | End-to-end pipeline from source files to verified artifacts | Draft |
| [ir_layers.md](./ir_layers.md) | All | IR ownership boundaries and stability rules across pipeline phases | Draft |
| [source_and_frontend.md](./source_and_frontend.md) | 1-3 | Source loading, preprocessing, lexing, and parsing boundaries | Draft |
| [module_and_symbol_resolution.md](./module_and_symbol_resolution.md) | 0, 4-5 | Package, module, namespace, label, and symbol table resolution | Draft |
| [reasoning_boundary.md](./reasoning_boundary.md) | 12-14 | Reasoning responsibility split between Mizar, ATP backends, and the kernel | Draft |
| [atp_interface_protocol.md](./atp_interface_protocol.md) | 13 | ATP problem formats and encoding strategy | Draft |
| [atp_backend_integration.md](./atp_backend_integration.md) | 13 | External ATP process execution, timeout handling, and certificate collection | Draft |

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
