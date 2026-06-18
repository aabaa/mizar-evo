# Architecture Design Specifications

> Canonical language: English. Japanese companion: [../ja/README.md](../ja/README.md).

This directory contains **cross-cutting internal design documents** that define the boundaries, protocols, and design decisions spanning multiple modules or crates.

## Purpose

While module-level specs (`doc/design/<crate>/<language>/<module>.md`) describe individual Rust source files, architecture specs address questions that cannot be answered at the single-module level:

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
| [04.type_and_registration_resolution.md](./04.type_and_registration_resolution.md) | 6-7 | Type checking, cluster database, and resolution traces | Draft |
| [05.overload_resolution.md](./05.overload_resolution.md) | 8 | Candidate selection, subsumption DAG, and `qua` insertion | Draft |
| [06.elaboration_and_core_ir.md](./06.elaboration_and_core_ir.md) | 9 | Lowering surface language into core logic | Draft |
| [07.vc_generation.md](./07.vc_generation.md) | 10-12 | Algorithm verification preparation and obligation generation | Draft |
| [08.reasoning_boundary.md](./08.reasoning_boundary.md) | 12-14 | Reasoning responsibility split between Mizar, ATP backends, and the kernel | Draft |
| [09.atp_interface_protocol.md](./09.atp_interface_protocol.md) | 13 | ATP problem formats and encoding strategy | Draft |
| [10.atp_backend_integration.md](./10.atp_backend_integration.md) | 13 | External ATP process execution, timeout handling, and certificate collection | Draft |
| [11.artifact_and_incremental_build.md](./11.artifact_and_incremental_build.md) | 15 | Artifact schema, cache update, and reproducibility | Draft |
| [12.diagnostics_and_lsp.md](./12.diagnostics_and_lsp.md) | All, 15 | Diagnostics, metadata, and IDE integration | Draft |
| [13.documentation_and_extraction.md](./13.documentation_and_extraction.md) | 16 | Documentation generation and code extraction | Draft |
| [14.parallel_verification_and_scheduling.md](./14.parallel_verification_and_scheduling.md) | 0, 10-15 | Verification task graph, parallel scheduling, cancellation, and deterministic result ordering | Draft |
| [15.kernel_certificate_format.md](./15.kernel_certificate_format.md) | 13-14 | Final certificate schema, clause trace checking, and kernel rejection semantics | Draft |
| [16.substitution_and_binding.md](./16.substitution_and_binding.md) | 4, 6, 9, 14 | Bound variables, alpha-equivalence, capture avoidance, and binder normalization | Draft |
| [17.cluster_trace_format.md](./17.cluster_trace_format.md) | 7, 11, 14-15 | Replayable cluster expansion and reduction application traces | Draft |
| [18.dependency_fingerprint.md](./18.dependency_fingerprint.md) | 0, 4-7, 11, 15 | Dependency slices, fingerprints, and incremental rebuild triggers | Draft |
| [19.failure_semantics.md](./19.failure_semantics.md) | All | Stable failure classification, propagation, and deterministic error ordering | Draft |
| [20.test_strategy.md](./20.test_strategy.md) | All | Regression strategy prioritizing fail and soundness tests | Draft |
| [21.ai_agent_interface.md](./21.ai_agent_interface.md) | All, 15 | AI agent operability: safe edit classes, authorization scopes, and the context/patch protocol framework | Draft |
| [22.incremental_verification_contract.md](./22.incremental_verification_contract.md) | All, 0-15 | Soundness contract for partial-edit incremental verification, proof/VC reuse, cache validation, and parallel-compatible scheduling | Draft |

`00.pipeline_overview.md` is the parent document for this directory. Other architecture documents should state which pipeline phase(s) they refine and should link back to the overview in their Context section.

## Cross-Cutting Concerns

### Memory Model

Memory scalability is a cross-cutting design property rather than the responsibility of any single subsystem. The guiding principle — stated normatively for the language in [doc/spec/en/12.6.3](../../../spec/en/12.modules_and_namespaces.md#1263-memory-model) and [doc/spec/en/23.7.9](../../../spec/en/23.package_management_and_build_system.md#2379-memory-design-principles) — is: **keep interfaces and indexes resident; load proof bodies, traces, and detailed AI-facing data lazily, and never duplicate global indexes per import closure.**

Each architecture document owns one facet of this property:

| Facet | Where the resident-set stays bounded | Document |
|---|---|---|
| Imported state is a minimal projection | `ModuleSummary` carries exported symbols/labels and a lexical summary, not proof bodies | [03.module_and_symbol_resolution.md](./03.module_and_symbol_resolution.md) |
| Cluster/registration data is a filtered view | The checker consumes an activated `RegistrationIndex` built from import-scoped registration summaries; the `cluster-db` cache stores import-scoped views rather than a copy per closure | [04.type_and_registration_resolution.md](./04.type_and_registration_resolution.md), [11.artifact_and_incremental_build.md](./11.artifact_and_incremental_build.md) |
| Traces and witnesses are external artifacts | On-disk traces and hash-referenced witness files rather than resident data | [11.artifact_and_incremental_build.md](./11.artifact_and_incremental_build.md), [17.cluster_trace_format.md](./17.cluster_trace_format.md) |
| Verification conditions stay per-module | The whole-module canonical `VcIr` is materialized before discharge; per-obligation ATP work is bounded by hierarchical resource budgets | [07.vc_generation.md](./07.vc_generation.md), [14.parallel_verification_and_scheduling.md](./14.parallel_verification_and_scheduling.md) |
| Only changed work is recomputed | Dependency fingerprints and reverse-dependency-cone rebuilds | [18.dependency_fingerprint.md](./18.dependency_fingerprint.md), [11.artifact_and_incremental_build.md](./11.artifact_and_incremental_build.md) |
| IDE state stays incremental | LSP keeps open-buffer snapshots and answers from indexed artifacts, not reconstructed global state | [12.diagnostics_and_lsp.md](./12.diagnostics_and_lsp.md) |
| Worker budgets bound peak usage | Per-build memory ceiling and per-backend process budgets | [14.parallel_verification_and_scheduling.md](./14.parallel_verification_and_scheduling.md) |
| Agent-facing context stays bounded | Per-obligation context budgets and lazily loaded AI-facing data instead of whole-library dumps | [21.ai_agent_interface.md](./21.ai_agent_interface.md) |

This is a qualitative resident-set model, not a performance guarantee: concrete memory budgets and benchmark metrics belong to the test/evaluation strategy ([20.test_strategy.md](./20.test_strategy.md)), not to the normative specifications.

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
- `doc/design/<crate>/<language>/<module>.md` → `crates/<crate>/src/<module>.rs`

## Constraints and Assumptions
Performance requirements, security considerations, compatibility, etc.
```

## Relationship to Other Documentation

| Layer | Directory | Granularity | Audience |
|---|---|---|---|
| External Spec | `doc/spec/en/` | Language features | Users |
| **Architecture** | **`doc/design/architecture/`** | **Cross-cutting subsystems** | **Developers** |
| Module Spec | `doc/design/<crate>/` | Individual files (1:1) | Developers |
