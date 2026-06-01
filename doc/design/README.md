# Internal Design Specifications

> Implementation order across crates: [todo.md](./todo.md).

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

## 3. Module Specifications (`<crate-name>/`)

Detailed specifications that map **1:1** to Rust source files.

```
doc/design/<crate-name>/<language>/<module>.md  →  crates/<crate-name>/src/<module>.rs
```

For module specifications, English documents are canonical and Japanese documents are companions when both are present.

Current module specification roots:

- [`mizar-session/`](./mizar-session/README.md) - source identity, build snapshots, source maps, and snapshot retention
- [`mizar-test/`](./mizar-test/README.md) - test corpus layout, `.miz` corpus strategy, fail/soundness tests, snapshots, and harness behavior
- [`mizar-lexer/`](./mizar-lexer/README.md) - raw lexical scanning, lexeme runs, and context-sensitive token disambiguation boundaries
- [`mizar-syntax/`](./mizar-syntax/README.md) - source-shaped syntax nodes, trivia, and recovery markers
- [`mizar-parser/`](./mizar-parser/README.md) - grammar implementation, Pratt parsing, and syntax recovery
- [`mizar-frontend/`](./mizar-frontend/README.md) - source loading and phase 1-3 orchestration across lexer and parser services

### Module Specification Template

Each module spec should follow this structure:

```markdown
# Module: <module_name>

## Purpose
Brief description of what this module does.

## Public API
List of public functions, structs, traits, and their signatures.

## Dependencies
- Internal: which other modules this depends on
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
   ↓  (decomposed into modules)
doc/design/<crate>/<language>/<module>.md
                                    Per-file implementation specifications
   ↓  (implemented)
crates/<crate>/src/<module>.rs     Rust source code
```

## Workflow

1. Start with an idea in `doc/idea/`
2. When a design decision is confirmed, promote it to `doc/design/architecture/`
3. Refine cross-cutting designs into internal subsystem designs in `doc/design/internal/`
4. Decompose into module-level specs in `doc/design/<crate>/<language>/`
5. Implement (or ask AI to implement) the corresponding Rust source
6. Run tests to verify the implementation matches the spec
7. Keep specs and code in sync — update both together
