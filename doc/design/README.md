# Internal Design Specifications

> Implementation order across crates: [todo.md](./todo.md).
> Crate-wide autonomous development protocol:
> [autonomous_crate_development.md](./autonomous_crate_development.md).

This directory contains implementation-facing design documents. English
documents are canonical; Japanese companions are provided where a language-pair
directory exists. For language behavior, `doc/spec/en/` and executable `.miz`
tests remain authoritative. Design documents refine that authority into crate
boundaries, APIs, data structures, algorithms, and task plans.

## 1. Architecture Specifications (`architecture/`)

Cross-cutting design documents that span crates or compiler subsystems. They
define subsystem boundaries, protocols, artifact contracts, scheduling rules,
and design decisions that cannot be owned by one module.

```text
doc/design/architecture/<language>/<topic>.md
```

Typical topics include:

- pipeline phases and source/frontend boundaries
- IR layering, module/symbol/type resolution, and elaboration
- ATP, kernel, proof evidence, cache, artifact, and build protocols
- diagnostics, LSP, documentation extraction, scheduling, and failure semantics

See [architecture/README.md](./architecture/README.md) and
[architecture/en/README.md](./architecture/en/README.md) for the index and
template.

## 2. Internal Design Specifications (`internal/`)

Subsystem implementation designs that refine architecture specifications into
compiler services, APIs, data structures, and execution contracts.

```text
doc/design/internal/<language>/<topic>.md
```

Typical topics include:

- compiler driver and pipeline scheduler
- artifact store, cache keys, and manifest transactions
- diagnostics model and LSP bridge
- ATP portfolio and kernel check integration
- documentation extraction
- crate/module ownership layout

See [internal/README.md](./internal/README.md) and
[internal/en/README.md](./internal/en/README.md) for the index and template.

## 3. Crate Design Specifications (`<crate-name>/`)

Crate design documents describe crate responsibilities, APIs, data structures,
algorithms, task order, and implementation boundaries. They are derived from
`doc/spec/en/`, executable tests, architecture docs, and internal subsystem
docs; they are not independent language authority.

```text
doc/design/<crate-name>/<language>/<topic>.md  relates to  crates/<crate-name>/src/...
```

Every crate root listed below has at least an English TODO. Some completed or
active crates also have root README files, module specs, crate plans, audit
notes, and exit reports. English documents are canonical and Japanese documents
are companions when both are present.

### Active Workspace Crates

- [mizar-session](./mizar-session/README.md) - source identity, source maps,
  build snapshots, source loading, and snapshot retention; current milestone
  complete.
- [mizar-lexer](./mizar-lexer/README.md) - raw lexical scanning, lexeme runs,
  scope skeletons, lexical environments, and token disambiguation boundaries;
  current milestone complete.
- [mizar-syntax](./mizar-syntax/README.md) - rowan-backed `SurfaceAst`, syntax
  trivia, recovery vocabulary, typed views, and current parser-facing syntax
  vocabulary; current milestone complete with only deferred rustdoc summaries
  remaining.
- [mizar-parser](./mizar-parser/README.md) - grammar implementation, Pratt
  parsing, syntax recovery, and parse-only corpus execution; grammar coverage
  has grown through task 38, with hardening work from task 39 onward pending.
- [mizar-frontend](./mizar-frontend/README.md) - source loading and phase 1-3
  orchestration across session, lexer, syntax, and parser services; current
  milestone complete.
- [mizar-test](./mizar-test/README.md) - corpus layout, expectation sidecars,
  staged model, traceability, snapshots, and harness behavior; implementation
  exists and the TODO tracks formal gap-closing work.
- [mizar-build](./mizar-build/en/todo.md) - workspace planning and scheduling;
  crate scaffold and the package-name validation slice exist, while full
  planner/module-index/scheduler specs and implementation remain pending.
- [mizar-lsp](./mizar-lsp/en/todo.md) - editor-facing range mapping and future
  server, snapshot, diagnostics, metadata, navigation, code-action, and
  explanation features; range conversion exists and the broader design remains
  planned.

### Planned Crate Roots

These directories currently carry crate-level TODOs and design intent before
their Rust crates are added to the workspace:

- [mizar-resolve](./mizar-resolve/en/todo.md) - module graph, namespaces,
  symbols, labels, signatures, and resolver diagnostics.
- [mizar-checker](./mizar-checker/en/todo.md) - type checking,
  cluster/registration resolution, and overload resolution.
- [mizar-core](./mizar-core/en/todo.md) - elaboration, binder-normalized core
  logic, and control-flow preparation.
- [mizar-vc](./mizar-vc/en/todo.md) - VC IR, VC generation, deterministic
  pre-ATP discharge, and dependency slices.
- [mizar-kernel](./mizar-kernel/en/todo.md) - trusted certificate parsing and
  checking.
- [mizar-atp](./mizar-atp/en/todo.md) - ATP encoding, backend execution, and
  portfolio candidates.
- [mizar-artifact](./mizar-artifact/en/todo.md) - artifact schemas, store,
  summaries, and manifest transactions.
- [mizar-doc](./mizar-doc/en/todo.md) - documentation rendering and extraction.
- [mizar-driver](./mizar-driver/en/todo.md) - build requests, phase registry,
  CLI/watch/LSP entry points, and query orchestration.
- [mizar-ir](./mizar-ir/en/todo.md) - IR storage, snapshot handles, publishers,
  cache adapters, and artifact projections.
- [mizar-proof](./mizar-proof/en/todo.md) - proof policy evaluation, status
  projection, witness selection, and evidence reuse metadata.
- [mizar-cache](./mizar-cache/en/todo.md) - cache keys, fingerprints, proof
  reuse, and cluster-database storage.
- [mizar-diagnostics](./mizar-diagnostics/en/todo.md) - diagnostic registry,
  failure records, deterministic ordering, and rendering.

## Focused Design Document Template

For a focused design document, use this structure when it fits:

```markdown
# Design: <topic>

## Purpose
Brief description of the responsibility this design covers.

## Specification And Test Inputs
- Relevant `doc/spec/en/` requirements
- Relevant `.miz` tests and traceability records

## Public API
List public functions, structs, traits, and signatures when the design owns an
API surface.

## Dependencies
- Internal: which crates, modules, or design topics this depends on
- External: crate dependencies

## Data Structures
Detailed description of key types.

## Algorithm / Logic
Step-by-step description of the core logic.

## Error Handling
Expected error conditions and how they are handled.

## Tests
Key test scenarios that must pass.
```

## Relationship Between Layers

```text
doc/idea/                          Immature ideas, brainstorming
   v  (matured)
doc/spec/en/ and tests/            Language authority and executable intent
   v  (refined)
doc/design/architecture/<lang>/    Cross-cutting design decisions
   v  (refined)
doc/design/internal/<lang>/        Subsystem APIs, data structures, contracts
   v  (refined into crate design)
doc/design/<crate>/<lang>/...      Crate and focused implementation design
   v  (mapped through crate plan or TODO)
crates/<crate>/src/...             Rust source code
```

## Workflow

For crate-wide autonomous work, follow
[autonomous_crate_development.md](./autonomous_crate_development.md) first. It
defines the authority order for language behavior, the required crate plan, gap
classification, review gates, and crate exit criteria.

For ordinary focused changes:

1. Identify the canonical spec/test/design inputs.
2. Update the relevant focused design or TODO when behavior, ownership, or task
   order changes.
3. Implement the corresponding Rust source changes.
4. Run focused tests first, then broader workspace verification when the change
   crosses module or crate boundaries.
5. Keep design docs, source behavior, tests, and traceability metadata in sync
   within the authority order above.
