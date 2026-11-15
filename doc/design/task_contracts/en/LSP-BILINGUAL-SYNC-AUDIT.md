# Task LSP-BILINGUAL-SYNC-AUDIT: bilingual documentation sync
Canonical language: English; [Japanese pointer](../ja/LSP-BILINGUAL-SYNC-AUDIT.md).
Status: specified. Tier: full; documentation boundary reconciliation without semantic credit.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); consumer: [task23](../../mizar-lsp/en/todo.md).

## Authority and readiness

Dependency: task22 is complete. Authority: [documentation policy](../../../../.claude/CLAUDE.md#documentation-maintenance), [language scope](../../documentation_compaction_rules.md#rule-4--language-scope-for-status-and-audit-documents), and the protocol's [single-owner rule](../../autonomous_crate_development.md#single-owner-documentation-rule).
English is canonical. Module designs retain translated Japanese companions; status, audit, process and roadmap companions point to English, including the crate plan and TODO; retain plan contract backlinks required by repository lint. Existing approved module boundaries and [source/spec correspondence](../../mizar-lsp/en/source_spec_correspondence.md) constrain reconciliation; this task cannot create language or protocol behavior.
Readiness: G7 is bounded design_drift assigned to this task. Audit every document under `doc/design/mizar-lsp/en/` and its corresponding Japanese path, including links and explicit implemented/deferred distinctions.

## Scope and ownership

Synchronize missing or divergent Japanese module content to canonical English. Reconcile stale English/Japanese boundary prose identified by G7 against existing owner decisions, preserving every deferred feature and evidence limitation. Convert status/audit companions to pointers where policy requires it; preserve externally referenced fragments or redirect them to their live owner.
Use `doc/design/mizar-lsp/en/bilingual_sync_audit.md` with the same-named Japanese pointer as the parity owner for document groups and unresolved parity deferrals; keep per-run evidence outside `doc/`. Add the audit link to the English plan and task23 TODO entry, and contract links to the English plan/TODO and Japanese plan pointer; check only task23 on completion.
Writes: this contract pair and `doc/design/mizar-lsp/{en,ja}/` documents whose owned state changes. Module documents retain API/invariant/test-design ownership; correspondence retains source/test evidence and follow-up ownership. Coverage and SCA-003 remain unchanged; parity grants no specification credit.
Forbidden: production/test/fixture changes, protected spec/trace/expectation edits, new semantics, changed diagnostics or acceptance, producer/registry/store implementations, AGENTS/protocol edits, owner-gated decisions, and task24 or later implementation. An unresolved language-intent contradiction is a stop condition, not a translation choice.
Existing evidence: module Test design sections and named source/tests in the correspondence owner. No new runtime tests are required for a documentation-only reconciliation.

## Evidence matrix and exit

Cap: three grouped audit cases. (1) Every canonical document has a same-path Japanese translation or policy-compliant pointer; module API, invariants, refusals, freshness, test design and deferred boundaries agree across languages. (2) G7 stale boundary prose agrees with existing approved implementation-boundary owners, without closing implementation gaps or strengthening typed/native/stdio evidence. (3) Local links/fragments, contract pairing/backlinks, documentation volume and task23 sequencing status remain valid; later tasks and coverage are unchanged.
Independent specification/test-intent review precedes freeze. Sufficiency, precision, volume/scope, consistency and final full-tier assessment review the entire unit, including the freeze.
Verification for freeze and implementation: `cargo test -p mizar-checker --test lint_policy`, `cargo test -p mizar-test --test lint_policy`, and affected local Markdown links/fragments. Documentation-only: fmt, Clippy, owner runtime and end-to-end checks do not apply. Full `cargo test` runs only on a cadence trigger. Push: no.
Exit: complete policy-compliant parity inventory and bounded G7 reconciliation, with no unresolved blocking/high finding; task23 checked. Freeze and implementation count as one changed task.
