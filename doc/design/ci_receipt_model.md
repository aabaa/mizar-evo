# CI receipt-model adoption design

## Status and authority

Activation is authorized; deployed controls remain bootstrap/shadow until the
[activation boundary](#activation-boundary) is satisfied.
The 2026-11-22 owner decision abolishes shadow operation at activation;
its 19:10 JST clarification authorizes production signer provisioning and live
canary receipt-tag publication within the activation unit.
Owner authority is the CI policy recorded by the orchestrator on 2026-11-21
13:07 JST, supplied through the CI-RECEIPT-DESIGN dispatch and registration.
Owner decisions `MIZAR-CI-RECEIPT-PROTOCOL-689` (2026-11-22 17:59:54 JST) and
`MIZAR-CI-RECEIPT-MODEL-V1-SCOPE` (2026-11-22, option 1) authorize all-change
receipts, including code, replacing per-push/PR hosted full tests with signed
local-full verification and retaining scheduled full hosted reconciliation.
These decisions supersede the earlier successor's docs-only and shadow restrictions;
integrity requirements remain.
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
Existing focused task checks and full-test cadence remain; this replaces a duplicate hosted
full execution only when eligible. A failed or invalidated gate needs a new successful gate.

The plan binds exact commands, Rust/Cargo versions, host/target, relevant environment,
dependency-lock identity and generator source identity. Expected environment and test-inventory
rules are custody-selected, versioned inputs, never learned from the candidate receipt.
Toolchain/dependency drift or removed/ignored tests require reviewed baseline renewal
and full hosted reconciliation before the renewed baseline can authorize receipts.
Local Linux execution is a signer observation, not independent Ubuntu-host equivalence.

## Receipt and transport

Use versioned strict canonical UTF-8 JSON: fixed keys/types, no duplicate/unknown keys,
floats or trailing bytes; deterministic sorted compact encoding with a terminal newline.
Bind repository identity, S/tree, comparison base, trusted policy/plan and generator hashes,
environment/inventory identities, ordered steps/commands, UTC start/end, durations, actual
exit statuses, pass/full scope, counts/ignored names and log SHA-256/size.
All mandatory steps must succeed; an empty or partial test inventory cannot qualify.
Sign exact manifest bytes with OpenSSH Ed25519 namespace `mizar-local-full-v1`.
The private key and generator stay outside Git, worker access and Actions; CI pins the
authorized identity/public key and generator/plan identities from trusted policy.
Freshness is at most 24 hours from execution start, with ordered times and no future times.
Key rotation/revocation and plan renewal are owner-controlled trust changes;
renewed pins require successful full hosted reconciliation before authorizing receipts.

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

Preserve [current CI](../../.github/workflows/ci.yml) formatting, dependency fetch,
Clippy and corpus-plan validation. Replace its per-push/PR `cargo test` with
enforcing receipt sanity; retain scheduled unfiltered hosted `cargo test` reconciliation.
Keep AGENTS' stronger local all-feature Clippy requirement; do not silently narrow it.
Sanity runs a trusted default-branch verifier/policy pinned independently of candidate code,
with read-only credentials, no signer secrets and no candidate/evidence checkout or imports.
Existing candidate-code gates remain separate from sanity. Receipt-tag events carry evidence
only; source branch pushes and PRs still receive their required checks.

V1 covers all changes, including Rust, tests/fixtures, scripts, configuration,
workflows, governance and language/proof documentation. Path classification never
substitutes for successful local exhaustive tests or confers semantic credit.
Trust-control changes require independent owner review and renewed trusted pins;
candidate-edited policy, signer, generator or workflow cannot authorize itself.
Main admission combines `receipt-sanity` from the expected GitHub Actions App
with independently enforced owner review of `.github/workflows/`,
`.github/ci-receipts/` and `.github/CODEOWNERS`, using base-branch CODEOWNERS.
The owner-review rule dismisses stale approvals and permits custody-owned
administrative bypass; the separate receipt requirement has no bypass.
Candidate status alone grants no authorization. Trust-control receipts use
prior-main/default controls; repository rules enforce independent approval.

Classify the entire main push `before..S` with ancestry checked, or the entire branch
merge-base-to-S delta against the trusted default branch; never just the latest lane slice.
Treat renames as delete/add and inspect both modes using NUL-delimited raw Git records.
Missing/ambiguous bases and unsupported comparisons fail enforcing sanity.
PRs, forks and synthetic merge revisions require receipts bound to the exact tested
event revision/tree; never reuse a head receipt for a merge tree. If custody cannot
test and attest that exact revision, the required check fails. A changed live
head/base invalidates an in-flight decision. No path or event silently falls back
to per-push hosted full tests; scheduled reconciliation remains independent.

## Activation boundary

Activate in one enforcement change after the canaries; no shadow or transition period
is required by the revised owner decision. Before activation, the owner must
approve environment/baseline and signer pins, deploy custody, and enforce the trusted required
check source through repository rules; fixtures cannot establish these external controls.
Activation requires live canaries for ordinary docs, trust changes, forgery and moved heads.
Until these pass, existing full CI remains effective. Rollback restores unconditional hosted testing
and revokes shortcut authorization without changing local exhaustive tests or acceptance policy.
Receipts attest a trusted signer's observation, not proof certificates or independent execution;
host/signer compromise and unobserved inputs remain outside their assurance.

## Bootstrap operation

This section describes the existing bootstrap deployment, not the authorized enforcing state.

The orchestrator stores `generate.py` and its unprovisioned `custody.json` at
`~/orchestration/ci-receipts/` on dev-linux; keys and the publication ledger stay there.
Only the owner provisions custody and reviewed tool, environment and inventory pins.
The fixed generator requires Linux namespaces, read-only Git metadata and toolchain,
an isolated writable checkout/cache, offline dependencies, and a parent-owned bounded log pipe.
It rejects source mutations through inotify as well as pre/postflight checks. Harness-qualified
names and ignored names must match the reviewed baseline, including individual summary counts.
The owner-only publication mode atomically pushes the sealed source and create-only evidence
tag, then appends its publication observation; bootstrap never invokes publication.
Repository policy starts disabled with no signer or baseline; it cannot grant a shortcut.
The [verifier](../../.github/ci-receipts/verify.py) consumes only exact Git blobs and
the [trusted policy](../../.github/ci-receipts/policy.json); no candidate modules are imported.
The `receipt-shadow` job extracts them from a fixed default-branch commit into isolated
storage: the event's prior default head for a default-branch push, otherwise the live default
head, always distinct from the candidate. Missing prior code stays bootstrap/full.
It checks live refs and reports observations while `mizar-test` runs unconditionally.
Remote evidence is fetched as commit metadata, one root tree, and three fixed blobs; reads
have size limits and Git operations time out. Native Git cannot prebound a single object's
wire transfer before its size is available; shadow failure always retains full execution.
Receipt tags trigger no source workflow; branch pushes and PRs retain their checks.
Run the disposable attack matrix with `python3 -m unittest discover -s tests/ci_receipts -v`;
generator tests use the external custody source, or `CI_RECEIPT_CUSTODY` for a disposable copy.
Required-check source enforcement, live canaries, baseline approval and production custody
deployment remain owner work. Rollback removes shadow observation and preserves full testing.
