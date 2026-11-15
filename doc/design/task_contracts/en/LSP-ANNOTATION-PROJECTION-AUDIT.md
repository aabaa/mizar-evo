# Task LSP-ANNOTATION-PROJECTION-AUDIT: annotation output boundaries
Canonical language: English; [Japanese pointer](../ja/LSP-ANNOTATION-PROJECTION-AUDIT.md).
Status: specified. Tier: full; annotation projection audit without semantic credit.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); consumer: [task24](../../mizar-lsp/en/todo.md).

## Authority and readiness

Dependencies: tasks13, 18 and 23 are complete. Authority: [chapter21](../../../spec/en/21.source_code_annotation_and_atp.md#215-aiide-display-hints), including §21.6, and [architecture12](../../architecture/en/12.diagnostics_and_lsp.md#lsp-model). The [coverage audit](../../spec_coverage_audit.md) owns chapter coverage and SCA-003.
Relevant evidence: [parser annotation pass](../../../../tests/miz/pass/parser/pass_parser_annotations_001.miz), [recovery fail](../../../../tests/miz/fail/parser/fail_parser_annotations_recovery_001.miz), parser/syntax annotation unit tests; LSP diagnostics, metadata_tests, hover_tests, explain_tests, snapshot/server and framed-stdio tests indexed by the [correspondence owner](../../mizar-lsp/en/source_spec_correspondence.md).
Classify missing source-to-output links as source_drift/test_gap or deferred integration, not as implemented semantics. Source observations cannot change normative annotation behavior.

## Scope and ownership

Create one English annotation projection audit under the LSP design tree with a Japanese pointer. Trace each of `@show_type`, `@show_resolution`, `@show_thesis` and `@eval` from parser/syntax through semantic producers, diagnostic codes/severity, metadata artifacts and LSP presentation. Separate required output from currently available diagnostic, inline hint, explicit information response and unavailable/deferred paths; identify exact symbols and named test limitations.
Record current snapshot admission, stale/pending behavior and response invalidation by linking existing owners; do not invent a new freshness policy. The LSP consumes upstream results and never evaluates terms.
Each missing path gets an actionable owner/follow-up using existing tasks/contracts where sufficient; otherwise append a bounded audit follow-up without changing tasks25 onward. Owner-gated semantic decisions remain deferred; stop if completing this audit requires a new semantic choice.
Writes: contract/pointer, audit/pointer, paired LSP plan/TODO indexes, coverage audit, and only stale annotation-audit status sentences in existing LSP owners/companions. Close SCA-003 as an audit obligation only when all four paths and deferrals are owned; chapters21/23 remain partial and trace/coverage credit unchanged.
Forbidden: production/test changes, public API or wire additions, annotation execution or emulation, registry/artifact/schema changes, language/specification decisions, protected spec/tests/expectations/trace edits, AGENTS/protocol edits, and task25/later implementation.

## Evidence matrix and exit

Cap: six grouped audit cases. (1) `@show_type(expr)` standalone input, including before a following item without attachment: syntax, intended `info.type`, producer/artifact and presentation boundary. (2) Attached `@show_resolution`: syntax, intended `info.resolution` plain-text diagnostic with FQN/template/coercion/refinement result and LSP hover boundary. (3) Proof `@show_thesis`: syntax, intended `info.thesis`, active obligation and missing-facts boundary. (4) `@eval(expr)` module/proof/algorithm inputs: syntax, value/type-only/non-computable outcomes, intended `info.eval`/warning and inline display boundary, without server execution. (5) Current/stale/pending/replaced or unavailable snapshots: only existing admitted data, explicit stale/refusal behavior and named evidence limits. (6) Every missing path has an owner, all links resolve, audit closure grants no semantic coverage.
Independent specification/test-intent review precedes freeze. Sufficiency, precision, volume/scope, consistency and final assessment cover the whole unit. Reuse existing tests; missing end-to-end evidence is recorded rather than fabricated.
Documentation-only verification for freeze and final tree: both checker/test lint_policy suites and affected link/fragment checks. No fmt, Clippy, owner runtime/E2E or full cargo test absent a cadence trigger; no executable or test-selection changes. Push: no.
Exit: all four annotation routes classified with owned deferrals; task24 checked in canonical TODO. Freeze and audit count as one changed task.
