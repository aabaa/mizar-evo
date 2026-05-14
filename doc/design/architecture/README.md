# Architecture Design Specifications

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
| [00.pipeline_overview.md](./00.pipeline_overview.md) | All | ソースから検証済み成果物までの全体処理パイプライン | Draft |
| [reasoning_boundary.md](./reasoning_boundary.md) | 12-14 | Mizar側 / ATP側 / kernel側 の推論責務境界 | Draft |
| [atp_interface_protocol.md](./atp_interface_protocol.md) | 13 | 外部ATPとの通信言語（TPTP/SMT-LIB）の設計 | Draft |
| [atp_backend_integration.md](./atp_backend_integration.md) | 13 | 外部ATPとの接続方式（プロセス管理等） | Draft |

`00.pipeline_overview.md` is the parent document for this directory. Other architecture documents should state which pipeline phase(s) they refine and should link back to the overview in their Context section.

## Document Template

Each architecture document should follow this structure:

```markdown
# Architecture: <Title>

## Purpose
The architectural problem this document addresses.

## Context
References to related external specs (doc/spec/) and ideas (doc/idea/).

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
| External Spec | `doc/spec/` | Language features | Users |
| Ideas | `doc/idea/` | Immature concepts | Brainstorming |
| **Architecture** | **`doc/design/architecture/`** | **Cross-cutting subsystems** | **Developers** |
| Module Spec | `doc/design/<crate>/` | Individual files (1:1) | Developers |
