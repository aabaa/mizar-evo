# Task CI-RECEIPT-DESIGN: Design signed local-full receipt adoption
Canonical language: English; [Japanese pointer](../ja/CI-RECEIPT-DESIGN.md).
Status: design frozen; successor implemented for bootstrap/shadow only. Design tier: light; implementation: full.
Owner: orchestrator via [process priority](../../todo.md#development-process-improvement-priority).
Test-gate owner: [mizar-test plan](../../mizar-test/en/00.crate_plan.md).
Consumers: local integration and CI maintainers. Dependencies: none for design; activation prerequisites below.
Purpose: adopt OEIS D9 / FCL D1 custody and exact sealed-full binding without duplicate ordinary CI tests.
Authority: owner dispatch CI-RECEIPT-DESIGN, registration c513717c5, and [protocol](../../autonomous_crate_development.md#verification-cadence).
Design owner: [receipt model](../../ci_receipt_model.md); existing behavior: [CI](../../../../.github/workflows/ci.yml), [test selector](../../../../tests/check.sh).
Classification: design gap only; current full CI is intentional, not implementation drift.
Current scope: adoption design, frozen separate-unit scope and roadmap link only.
Forbidden now: CI/workflow/policy/code/tests changes, AGENTS/protocol edits, implementation work,
language/test-intent changes, Open Question/G-class decisions and semantic/coverage credit.

## Frozen successor: CI-RECEIPT-IMPLEMENTATION

Requires a separate full-tier dispatch with explicit custody paths and external-control authority.
Implement the [local gate](../../ci_receipt_model.md#local-gate-and-custody),
[receipt/transport](../../ci_receipt_model.md#receipt-and-transport),
[routing](../../ci_receipt_model.md#ci-routing-and-trust-exceptions) and
[activation boundary](../../ci_receipt_model.md#activation-boundary) exactly; no shortcut-region expansion.
Allowed repository surface: `.github/workflows/ci.yml`, one verifier and one pinned policy under
`.github/ci-receipts/`, dedicated verifier tests/fixtures under `tests/ci_receipts/`, and necessary owner-doc updates.
The generator/key/publication ledger belong to orchestrator storage outside the target repository.
Owner policy reconciliation and protected-check configuration are activation prerequisites,
not permission for this unit or its successor to edit AGENTS/protocol autonomously.
Preserve exhaustive `cargo test`, focused task checks, existing non-test CI gates and language/proof policy.

Test matrix cap: eight groups; subcases exercise each mandatory refusal within its group.
1. Genuine isolated full/pass capture and signature/tag roundtrip => eligible docs accepted once.
2. Failed/timed/interrupted/dirty/mutated execution or filtered/empty/omitted/ignored inventory drift => no success receipt.
3. Wrong repository/head/tree/base, moved head/base, partial branch delta or replay => no shortcut.
4. Forgery, tamper, unknown/revoked signer, stale/future times, changed plan/generator/environment => refusal.
5. Duplicate/unknown keys, unsupported schema, malformed encoding, missing/oversized/nonregular evidence or log mismatch => refusal.
6. Docs versus every trust override, code/tests/config, rename/mode/unknown/empty/ambiguous comparisons => exact routing.
7. Candidate verifier/workflow/key substitution, unavailable trusted anchor, fork/PR/merge tree => full CI or failed required check.
8. Bootstrap/shadow, exact required-check source, retained non-test gates, activation/rollback and live canaries => owner-observed fail-closed deployment.
Beyond-cap findings name a design requirement. Disposable fixture keys never use production custody.

## Review, verification and exit

Design: independent equivalence and removable-only volume/scope reviews; both lint_policy suites,
documentation-volume and local links/fragments; no executable gates or full-test trigger for this design alone.
Exit: reviewed adoption design plus frozen successor scope; no implementation or CI activation credit.
Successor: intent/trust, sufficiency, precision, volume and final consistency reviews; fmt, AGENTS Clippy,
both lint suites, receipt attack matrix and representative E2E, plus full `cargo test` for policy/CI adoption.
