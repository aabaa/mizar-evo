# Task CI-RECEIPT-ACTIVATION: Enforce signed local-full receipts
Canonical language: English; [Japanese pointer](../ja/CI-RECEIPT-ACTIVATION.md).
Status: frozen scope; activation requires the live boundary below. Gate tier: full.
Owner: orchestrator via [process priority](../../todo.md#development-process-improvement-priority).
Test-gate owner: [mizar-test plan](../../mizar-test/en/00.crate_plan.md).
Consumers: sealed integration, hosted CI and repository administrators.
Authority: RESUME 2 dispatch and owner decisions `MIZAR-CI-RECEIPT-PROTOCOL-689`
and `MIZAR-CI-RECEIPT-MODEL-V1-SCOPE` (2026-11-22, option 1).
Owner requirements: [receipt model](../../ci_receipt_model.md), especially
[custody](../../ci_receipt_model.md#local-gate-and-custody),
[transport](../../ci_receipt_model.md#receipt-and-transport),
[routing](../../ci_receipt_model.md#ci-routing-and-trust-exceptions) and
[activation](../../ci_receipt_model.md#activation-boundary).
Supersedes the [earlier successor scope](./CI-RECEIPT-DESIGN.md#frozen-successor-ci-receipt-implementation)
only for all-change eligibility and direct enforcement; retain its integrity requirements.
Classification: authorized CI policy expansion and deployment gap; no language-intent change.

## Scope and order

Revise the receipt model for all changes and only the protocol's every-push full-CI clause.
Then execute live docs, trust-change, forgery and moved-head canaries before enforcement.
Generate the production signer in `~/orchestration/ci-receipts/`, private mode 600;
register only public key, generator/key identities and reviewed public policy pins.
Switch [CI](../../../../.github/workflows/ci.yml) once: enforcing `receipt-sanity`,
remove observe-only shadow and per-push/PR full testing, retain scheduled full reconciliation.
Configure required-check protection and verify it by readback. The orchestrator lands;
this unit commits locally without pushing. Stop at any uncompletable step with evidence.
Allowed implementation: existing verifier/policy, receipt tests and external custody tooling.
Preserve non-test CI gates, exhaustive local tests, signing, exact-head binding,
trusted independent controls, capture/inventory checks and bounded evidence verification.
Forbidden: language-spec/test-intent changes, other protocol or AGENTS edits, signer
secrets in Git, weakened integrity, self-authorizing controls or bypassed live canaries.
No semantic/coverage credit or spec-coverage-audit ownership changes.

## Test matrix and exit

Cap: eight groups, retaining the earlier successor's refusal subcases.
1. Genuine isolated full/pass capture, signing and exact-tag roundtrip for docs and code.
2. Failed, timed, interrupted, dirty or mutated gate; filtered/empty/omitted/ignored drift refuses.
3. Wrong repository/head/tree/base, replay, partial delta and moved live refs refuse.
4. Forgery/tamper, revoked signer, stale/future times and changed trust pins refuse.
5. Noncanonical/schema/blob/log-size or hash failures refuse; evidence is never executed.
6. All-change eligibility retains exact comparison checks and independent trust-control review.
7. Candidate control substitution refuses; PR/fork/merge needs an exact event receipt.
8. Live canaries, required-check source/readback, one-switch enforcement, schedule and rollback.
Fixtures cannot establish external controls. New cases beyond the cap name an owner requirement.
Reviews: specification/trust, sufficiency, precision, removable volume, consistency and final quality.
Verification: fmt, all-target/all-feature Clippy, both lint_policy suites, receipt attack matrix,
representative end-to-end receipt check and full `cargo test` for the CI-policy change.
Exit: all scoped steps verified locally and live, local commit ready for orchestrator landing/T2 audit.
