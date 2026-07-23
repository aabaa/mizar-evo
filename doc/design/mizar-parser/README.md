# mizar-parser Design

Canonical language: English. Japanese companion: [ja/README.md](./ja/README.md).

This directory contains implementation-facing design notes for the `mizar-parser` crate.

`mizar-parser` consumes the frontend-adapted parser token transfer object, applies the Mizar Evo grammar, and produces `mizar-syntax::SurfaceAst`. It owns grammar logic and syntax recovery, but it must not perform semantic name resolution, type inference, overload selection, cluster registration, elaboration, or proof-obligation generation.

Status: Tasks 1-48 are implemented. Task 46 now parses concrete operator
declarations after a fresh audit confirmed that completed frontend Task 20 had
already satisfied its named trigger. The earlier 94/100 parser closeout is
historical and superseded until a separate post-Task-46 closeout reruns every
hard gate. P-265-47D remains a nonblocking human-owned wording gap. This status
does not close global Step 5 or authorize Task 49 or Steps 6/7.

## Expected Module Specs And Audits

- [en/00.crate_plan.md](./en/00.crate_plan.md) - crate responsibility, current milestone, deferred work, and closeout gates
- [en/grammar.md](./en/grammar.md) - parser entry points and module/item grammar
- [en/pratt.md](./en/pratt.md) - term and formula precedence parsing
- [en/recovery.md](./en/recovery.md) - synchronization, error nodes, and skipped-token handling
- [en/source_spec_audit.md](./en/source_spec_audit.md) - task 43 source/spec and reserved-word coverage audit
- [en/bilingual_documentation_synchronization.md](./en/bilingual_documentation_synchronization.md) - task 44 bilingual documentation synchronization audit
- [en/crate_exit_report.md](./en/crate_exit_report.md) - historical pre-Task-46 closeout evidence pending a separate fresh closeout
