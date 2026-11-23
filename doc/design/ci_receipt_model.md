# CI receipt-model adoption design

## Status and authority

Owner decisions `MIZAR-CI-ABOLISH` and `MIZAR-CI-FULLRUN-REMOVAL`
(2026-11-23, including the weekly full-test amendment) replace all hosted CI
with local verification. Repository Actions are disabled; no standing GitHub
workflow remains. These decisions supersede earlier hosted routing and reconciliation.
The signed local-full receipt remains the admission gate for every change.
This design neither changes language intent nor settles Open Questions or G-class items.
The orchestrator owns custody, sealed integration and activation; workers own task feedback.
References: [OEIS D9 receipt](https://github.com/aabaa/oeis-semantic-atlas/blob/39d91933161509fbe95312efd017bdcf38f39621/docs/D9_LOCAL_TEST_RECEIPT.md),
[D9 sanity](https://github.com/aabaa/oeis-semantic-atlas/blob/39d91933161509fbe95312efd017bdcf38f39621/docs/D9_CI_RECEIPT_SANITY.md),
[FCL D1](https://github.com/aabaa/formal-cad-language/blob/939342784b0efac84c51035c401029e32869be4a/docs/core-local-receipts.md).

## Local gate and custody

After lane integration, the orchestrator seals source commit S and its exact Git tree.
Its fixed, reviewed generator runs outside worker checkouts from orchestrator-owned storage;
workers cannot select its implementation, plan, environment, signer or success values.
Use a dedicated clean checkout, exclusive execution and a controlled environment; disable
ambient Cargo/Rust configuration and unreviewed wrappers, and bind dependencies to Cargo.lock.
Preflight and postflight verify HEAD, tracked/untracked cleanliness and unchanged inputs.
The generator itself launches one unfiltered `cargo test`, captures complete stdout/stderr,
waits for its actual exit status and records test summaries, including ignored tests.
No filters, extra arguments, skips, synthetic counts or worker-provided result files qualify.
Failure, timeout, concurrent mutation or incomplete capture cannot produce a passing receipt.
Existing focused task checks and full-test cadence remain. A failed or invalidated gate needs a new successful gate.

The plan binds exact commands, Rust/Cargo versions, host/target, relevant environment,
dependency-lock identity and generator source identity. Expected environment and test-inventory
rules are custody-selected, versioned inputs, never learned from the candidate receipt.
Toolchain/dependency drift or removed/ignored tests require reviewed baseline renewal
and full local reconciliation before the renewed baseline can authorize receipts.
Local Linux execution is a signer observation, not independent Ubuntu-host equivalence.

## Receipt and transport

Use versioned strict canonical UTF-8 JSON: fixed keys/types, no duplicate/unknown keys,
floats or trailing bytes; deterministic sorted compact encoding with a terminal newline.
Bind repository identity, S/tree, comparison base, trusted policy/plan and generator hashes,
environment/inventory identities, ordered steps/commands, UTC start/end, durations, actual
exit statuses, pass/full scope, counts/ignored names and log SHA-256/size.
All mandatory steps must succeed; an empty or partial test inventory cannot qualify.
Sign exact manifest bytes with OpenSSH Ed25519 namespace `mizar-local-full-v1`.
The private key and generator stay outside Git and worker access; custody pins the
authorized identity/public key and generator/plan identities from trusted policy.
Freshness is at most 24 hours from execution start, with ordered times and no future times.
Key rotation/revocation and plan renewal are owner-controlled trust changes;
renewed pins require successful full local reconciliation before authorizing receipts.

Keep evidence outside S: create one evidence commit with fixed regular blobs
`manifest.json`, `manifest.sig`, `cargo-test.log`, referenced by create-only tag
`ci-receipts/<S>`. The source commit never acquires its own receipt or a post-test amendment.
Publish evidence and S together; retain full logs and an append-only custody publication ledger.
The verifier derives S from the event, fetches only the exact tag and bounded blobs, verifies
signature before interpreting claims, then schema/identities/log hash/size/freshness.
Never execute evidence or select a source revision/key/plan from its claims.
Missing, malformed, forged or mismatched evidence fails enforcing sanity,
with no older-receipt fallback. Revoked or expired evidence cannot authorize a shortcut.

## CI routing and trust exceptions

Formatting, all-feature Clippy, dependency and corpus-plan validation run locally.
The installed [admission runner](../../.github/ci-receipts/admit.py) is custody-owned,
not executed from candidate code. It loads the verifier and policy from the exact
prior-main base into isolated storage and verifies the staged signed evidence before
publication. Missing independent controls, invalid evidence or a changed base refuses.
Publication checks fast-forward ancestry, uses an exact-base lease and atomically
creates the evidence tag with the sealed source. Legacy generator publication is disabled.

V1 covers all changes; path classification confers no semantic credit.
Trust-control changes still require independent review; candidate policy, signer,
generator or runner cannot authorize itself. GitHub requires no Actions status checks.
A separate main-update restriction permits only custody-owned administration; the
independent owner-review rule, stale-approval dismissal, force-push and deletion guards
remain. Administrators and their credentials are trusted: GitHub does not independently
verify receipt signatures. Administrative compromise remains outside this assurance.

Classify the entire prior-main-to-source delta with ancestry checked, never just its
latest slice. The exact final integration commit/tree requires its own receipt;
PR/head receipts cannot authorize a different merge tree. Moved refs invalidate admission.
No event falls back to hosted tests. Weekly exhaustive local reconciliation is separate
from per-publication receipts and retains its source identity, complete logs and failure.

## Activation boundary

The receipt activation canaries remain the prerequisite. The local-only transition
requires reviewed custody admission, live protection readback and a local weekly
registration. Fixtures cannot establish external controls. Rollback halts publication
and repairs/reconciles locally; it does not re-enable GitHub CI.
Receipts attest a trusted signer's observation, not proof certificates or independent
execution; host/signer compromise and unobserved inputs remain outside their assurance.

## Bootstrap operation

The orchestrator stores `generate.py`, `admit.py`, `weekly.sh` and `custody.json`
at `~/orchestration/ci-receipts/`; keys and the append-only publication ledger stay there.
Only the owner provisions custody and reviewed tool, environment and inventory pins.
The generator uses Linux namespaces, read-only Git/toolchain inputs, an isolated
checkout/cache, offline dependencies and a bounded log pipe; inotify and pre/postflight
checks reject source mutation. Exact test names, ignored names and counts must match
the reviewed baseline. The admission runner checks signatures and publishes only a
pending custody receipt; custody configuration disables the former `--publish` route.
The weekly local timer invokes `weekly.sh` under the same exclusive custody lock,
with a clean source checkout and retained local logs; it creates no substitute receipt.
Run the disposable matrix with `python3 -m unittest discover -s tests/ci_receipts -v`;
generator tests use external custody, or `CI_RECEIPT_CUSTODY` for a disposable copy.
