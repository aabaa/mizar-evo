# mizar-parser Design

Canonical language: English. Japanese companion: [ja/README.md](./ja/README.md).

This directory contains implementation-facing design notes for the `mizar-parser` crate.

`mizar-parser` consumes the frontend-adapted parser token transfer object, applies the Mizar Evo grammar, and produces `mizar-syntax::SurfaceAst`. It owns grammar logic and syntax recovery, but it must not perform semantic name resolution, type inference, overload selection, cluster registration, elaboration, or proof-obligation generation.

Status: the current parser crate milestone is complete through tasks 1-45 and
47-48 with an independently reviewed score of 94/100. Task 46 remains
trigger-deferred for future concrete operator declarations, and P-265-47D is a
nonblocking human-owned wording gap. This closeout does not close global Step 5
or authorize a successor parser task.

## Expected Module Specs And Audits

- [en/00.crate_plan.md](./en/00.crate_plan.md) - crate responsibility, current milestone, deferred work, and closeout gates
- [en/grammar.md](./en/grammar.md) - parser entry points and module/item grammar
- [en/pratt.md](./en/pratt.md) - term and formula precedence parsing
- [en/recovery.md](./en/recovery.md) - synchronization, error nodes, and skipped-token handling
- [en/source_spec_audit.md](./en/source_spec_audit.md) - task 43 source/spec and reserved-word coverage audit
- [en/bilingual_documentation_synchronization.md](./en/bilingual_documentation_synchronization.md) - task 44 bilingual documentation synchronization audit
- [en/crate_exit_report.md](./en/crate_exit_report.md) - current parser milestone closeout, hard gates, score, verification, and handoff
