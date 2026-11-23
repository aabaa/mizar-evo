# Task CI-FULLRUN-REMOVAL: Local-only verification
Canonical language: English; [Japanese pointer](../ja/CI-FULLRUN-REMOVAL.md).
Status: frozen scope. Gate tier: full.
Owner: orchestrator; test owner: [mizar-test plan](../../mizar-test/en/00.crate_plan.md).
Authority: dispatch and owner decisions MIZAR-CI-ABOLISH / MIZAR-CI-FULLRUN-REMOVAL
(2026-11-23); dependency: [receipt activation](CI-RECEIPT-ACTIVATION.md).
Consumers: custody publication, repository administrators and local reconciliation.
Requirements: [receipt model](../../ci_receipt_model.md#local-gate-and-custody).
Classification: authorized validation/trust-boundary change; no language intent change.

## Scope

Remove all GitHub Actions workflows, including scheduled reconciliation.
Keep exhaustive local capture, signed exact-source evidence, prior-main trusted controls,
independent control review and create-only atomic source/evidence publication.
Move receipt admission to custody before publication, checking the signed evidence
locally with trusted prior-main verifier/policy and enforcing the exact remote base.
Make this the sole configured publication route; disable legacy generator publication.
Remove obsolete Actions required checks; restrict main updates to custody-owned
administration while retaining independent owner review and deletion/force-push guards.
Read back permitted administrators and credential custody; record their trust boundary.
Record actual protection settings before/after in the implementation commit body.
Register weekly exhaustive reconciliation on the local host, with retained logs;
renewal and rollback use local full testing rather than hosted execution.
Update the owning receipt design and affected receipt tests; preserve cryptographic
schema, signer, generator and baseline pins. AGENTS/protocol documents are excluded.
No language, proof, diagnostic, semantic-credit or coverage-audit ownership changes.

## Matrix and exit

Cap: six groups; retain existing receipt integrity/refusal subcases.
1. No hosted workflow or scheduled GitHub execution remains.
2. Genuine local receipt passes; missing, forged, stale or mismatched evidence refuses.
3. Prior-main controls govern; candidate substitution and unavailable anchors refuse.
4. Moved source/base, incomplete delta, wrong revision and legacy publication refuse.
5. Required-check removal and main update restriction read back; owner review preserved.
6. Weekly local full-test registration is enabled and its command is executable.
Reviews: specification/trust, sufficiency, precision, removable volume, consistency,
and final quality. Verification: fmt, all-feature Clippy, both lint_policy suites
(including links/fragments), receipt suites/E2E and unfiltered cargo test.
Exit: frozen contract, reviewed implementation, passing local full gate, signed
publication, live protection readback and local weekly registration; T2 is external.
