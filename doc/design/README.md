# Internal Design Specifications

> Implementation order across crates: [todo.md](./todo.md).
> Crate-wide autonomous development protocol:
> [autonomous_crate_development.md](./autonomous_crate_development.md).

This directory contains internal implementation specifications organized at two levels:

## 1. Architecture Specifications (`architecture/`)

Cross-cutting design documents that span multiple modules or crates.
These define **subsystem boundaries, protocols, and design decisions** that cannot be captured at the individual module level.

```
doc/design/architecture/<topic>.md
```

Typical topics include:
- Reasoning boundaries (what the kernel handles vs. what ATPs handle)
- Inter-subsystem communication protocols (TPTP, SMT-LIB)
- Integration strategies for external tools
- Proof certificate formats

See [`architecture/README.md`](./architecture/README.md) for details and the document template.

## 2. Internal Design Specifications (`internal/`)

Subsystem implementation designs that refine architecture specifications into compiler services, APIs, data structures, and execution contracts.

```
doc/design/internal/<language>/<topic>.md
```

Typical topics include:
- Compiler driver and pipeline scheduler
- Artifact store, cache keys, and manifest transactions
- Diagnostics model and LSP bridge
- ATP portfolio and kernel check integration
- Documentation extraction

See [`internal/README.md`](./internal/README.md) for details and the document template.

## 3. Crate Design Specifications (`<crate-name>/`)

Crate design documents describe crate responsibilities, APIs, data structures,
algorithms, and implementation boundaries. They are derived from `doc/spec/en/`
and tests; they are not independent language authority.

```
doc/design/<crate-name>/<language>/<topic>.md  relates to  crates/<crate-name>/src/...
```

Some focused design documents may map directly to a Rust source module, but
crate-wide autonomous work should be decomposed by specification requirement or
test obligation first, then mapped to design and source files through the Crate
Plan.

For crate design specifications, English documents are canonical and Japanese documents are companions when both are present.

Current crate design roots:

- [`mizar-session/`](./mizar-session/README.md) - source identity, build snapshots, source maps, and snapshot retention
- [`mizar-test/`](./mizar-test/README.md) - test corpus layout, `.miz` corpus strategy, fail/soundness tests, snapshots, and harness behavior
- [`mizar-lexer/`](./mizar-lexer/README.md) - raw lexical scanning, lexeme runs, and context-sensitive token disambiguation boundaries
- [`mizar-syntax/`](./mizar-syntax/README.md) - source-shaped syntax nodes, trivia, and recovery markers
- [`mizar-parser/`](./mizar-parser/README.md) - grammar implementation, Pratt parsing, and syntax recovery
- [`mizar-frontend/`](./mizar-frontend/README.md) - source loading and phase 1-3 orchestration across lexer and parser services

### Focused Design Document Template

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

```
doc/idea/                          Immature ideas, brainstorming
   ↓  (matured)
doc/design/architecture/           Confirmed cross-cutting design decisions
   ↓  (refined)
doc/design/internal/               Subsystem APIs, data structures, execution contracts
   ↓  (refined into crate design)
doc/design/<crate>/<language>/<topic>.md
                                    Crate and focused implementation design
   ↓  (mapped through crate plan)
crates/<crate>/src/...             Rust source code
```

## Workflow

For crate-wide autonomous work, follow
[autonomous_crate_development.md](./autonomous_crate_development.md) first. It
defines the authority order for language behavior, the required Crate Plan, and
crate exit gates.

1. Start with an idea in `doc/idea/`
2. When a design decision is confirmed, promote it to `doc/design/architecture/`
3. Refine cross-cutting designs into internal subsystem designs in `doc/design/internal/`
4. Refine into crate or focused design documents in `doc/design/<crate>/<language>/`
5. Implement (or ask AI to implement) the corresponding Rust source changes
6. Run tests to verify the implementation matches the spec
7. Keep design specs and code in sync within the authority order above
