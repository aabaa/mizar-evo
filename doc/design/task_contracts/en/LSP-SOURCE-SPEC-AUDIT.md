# Task LSP-SOURCE-SPEC-AUDIT: source/spec correspondence
Canonical language: English; [Japanese pointer](../ja/LSP-SOURCE-SPEC-AUDIT.md).
Status: specified. Tier: full; crate-wide correspondence audit, without semantic credit.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); consumer: [task22](../../mizar-lsp/en/todo.md).

## Authority and readiness

Dependency: task21 is complete. Authority: all nine [module specifications](../../mizar-lsp/en/todo.md#module-implementation) and their linked upstream boundaries; source and tests are observations, not new normative behavior.
Audit the actual public surface, including reexports from private modules, public members and trait implementations, and every promised module behavior, including not-yet-implemented features. Existing test evidence is reused; missing evidence is classified rather than fabricated.
Existing evidence: inline tests in [range_mapper.rs](../../../../crates/mizar-lsp/src/range_mapper.rs), [server.rs](../../../../crates/mizar-lsp/src/server.rs) and [diagnostics.rs](../../../../crates/mizar-lsp/src/diagnostics.rs); sibling build_bridge_timing_tests, build_bridge_event_tests, metadata_tests, hover_tests, semantic_token_tests and explain_tests modules; [public API policy](../../../../crates/mizar-lsp/tests/lint_policy.rs) and [framed stdio](../../../../crates/mizar-lsp/tests/stdio.rs). Module Test design sections own the intended behavioral scope.

## Scope and ownership

Create one English correspondence owner under the LSP design tree with a Japanese pointer. Map specification sections and public API groups to exact source symbols and named tests; state whether evidence is genuine publication/transport, typed fixtures or a deferred boundary. Cover range, server, snapshot, diagnostics, build bridge, metadata, navigation, code actions and explanations.
Record design_drift, source_drift, test_gap, spec_gap or source_undocumented_behavior where warranted. Each gap must have evidence, an owner and an actionable follow-up: reuse existing tasks when sufficient, otherwise append a bounded audit follow-up to both TODOs without changing later tasks. Do not implement any discovered gap.
Correct only audit-owned status/link drift in the module table and task22 entries; link the audit and contract from the plan/TODO. Other module prose and bilingual synchronization remain task23's work.
Writes: this contract pair, correspondence/pointer pair, paired LSP plans and TODOs. No production/test changes, public API additions, language choices, producer/registry/storage changes, protected spec/test/trace edits, AGENTS/protocol changes or task23/later implementation. Owner-gated semantics remain deferred and must be escalated if this audit itself requires choosing them.
Coverage and SCA-003 status remain unchanged: correspondence and explicitly owned gaps grant no additional specification coverage.

## Evidence matrix and exit

Cap: three grouped audit cases. (1) Enumerate every exported declaration, member/method and relevant trait implementation from lib/server/range_mapper plus private-module reexports; each has a specification owner, implementation symbol and named test, or an explicit owned gap. (2) Enumerate every module promise by section, including freshness/retention, refusal/error paths, bounds, scheduling, transport and missing producer behavior; map existing evidence with limitations, or an owned gap. (3) Check every deferred/missing/partial row has an actionable TODO owner, links resolve, and no source observation becomes normative or grants coverage credit.
Independent specification/test-intent review precedes freeze. Sufficiency, precision, volume/scope, consistency and final full-tier assessment review the whole unit; reviewers may inspect source/tests but do not implement follow-ups.
Documentation-only verification: both checker/test lint_policy suites, including documentation ledger, links/fragments and pairing checks, for freeze and final tree. Rust formatting, Clippy, owner runtime tests and full cargo test are unrun unless an applicable cadence trigger arises; no executable or test-selection behavior changes. Push: no.
Exit: complete public/behavior correspondence with owned gaps; task22 checked in both TODOs. Freeze and audit implementation count as one changed task.
