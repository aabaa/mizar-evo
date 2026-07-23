# mizar-parser Design

Canonical language: English. Japanese companion: [ja/README.md](./ja/README.md).

This directory contains implementation-facing design notes for the `mizar-parser` crate.

`mizar-parser` consumes the frontend-adapted parser token transfer object, applies the Mizar Evo grammar, and produces `mizar-syntax::SurfaceAst`. It owns grammar logic and syntax recovery, but it must not perform semantic name resolution, type inference, overload selection, cluster registration, elaboration, or proof-obligation generation.

Status: parser task 48 is complete. The parser now emits a dedicated top-level
`PropertyImplementation` for the canonical Chapter-7 means/equals forms and
runs their pass/fail corpus through the active parse-only path. This is
syntax-only coverage; semantic Task 39 remains deferred. Task 46 remains
deferred for future concrete operator declarations.

## Expected Module Specs And Audits

- [en/00.crate_plan.md](./en/00.crate_plan.md) - crate responsibility, active Step-5 tasks, and closeout gates
- [en/grammar.md](./en/grammar.md) - parser entry points and module/item grammar
- [en/pratt.md](./en/pratt.md) - term and formula precedence parsing
- [en/recovery.md](./en/recovery.md) - synchronization, error nodes, and skipped-token handling
- [en/source_spec_audit.md](./en/source_spec_audit.md) - task 43 source/spec and reserved-word coverage audit
- [en/bilingual_documentation_synchronization.md](./en/bilingual_documentation_synchronization.md) - task 44 bilingual documentation synchronization audit
