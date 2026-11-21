# CI receipt-model adoption design

## Status and authority

Implemented for bootstrap/shadow only; current full CI and verification policy remain effective.
Owner authority is the CI policy recorded by the orchestrator on 2026-11-21
13:07 JST, supplied through the CI-RECEIPT-DESIGN dispatch and registration.
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
Toolchain/dependency drift or removed/ignored tests require reviewed baseline renewal and full CI.
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
Key rotation/revocation and plan renewal are owner-controlled trust changes requiring full CI.

Keep evidence outside S: create one evidence commit with fixed regular blobs
`manifest.json`, `manifest.sig`, `cargo-test.log`, referenced by create-only tag
`ci-receipts/<S>`. The source commit never acquires its own receipt or a post-test amendment.
Publish evidence and S together; retain full logs and an append-only custody publication ledger.
The verifier derives S from the event, fetches only the exact tag and bounded blobs, verifies
signature before interpreting claims, then schema/identities/log hash/size/freshness.
Never execute evidence or select a source revision/key/plan from its claims.
Missing evidence may select full CI; malformed, forged or mismatched evidence fails sanity,
with no older-receipt fallback. Revoked or expired evidence cannot authorize a shortcut.

## CI routing and trust exceptions

Preserve [current CI](../../.github/workflows/ci.yml) formatting, dependency fetch,
Clippy and corpus-plan validation. Only its `cargo test` step is conditionally replaced.
Keep AGENTS' stronger local all-feature Clippy requirement; do not silently narrow it.
Sanity runs a trusted default-branch verifier/policy pinned independently of candidate code,
with read-only credentials, no signer secrets and no candidate/evidence checkout or imports.
Existing candidate-code gates remain separate from sanity. Receipt-tag events carry evidence
only; source branch pushes and PRs still receive their required checks.

V1 is deliberately conservative, matching D9: only nonempty changes consisting entirely of
regular nonexecutable Markdown under `doc/` or exact `README.md` can use receipts, subject
to the overrides below. All Rust, tests/fixtures, scripts, Cargo/toolchain/configuration,
workflows and unknown paths retain full CI. Widening this region is a separate owner decision.
Within `doc/`, full-CI overrides cover `spec/`, `design/architecture/`, `design/task_contracts/`,
and exact `design/{autonomous_crate_development,documentation_compaction_rules,codex_dispatch_template,ci_receipt_model,todo,spec_coverage_audit}.md`.
These include language/proof, governance, verification and receipt controls; they confer no semantic credit.
The trusted policy owns literal prefix/exact matching, not candidate-supplied glob rules.

Classify the entire main push `before..S` with ancestry checked, or the entire branch
merge-base-to-S delta against the trusted default branch; never just the latest lane slice.
Treat renames as delete/add and inspect both modes using NUL-delimited raw Git records.
Missing/ambiguous bases, empty diffs, nonregular modes and unknown paths select full CI.
PRs, forks and synthetic merge revisions retain full CI in V1; never reuse a head receipt
for a merge tree. A changed live head/base invalidates an in-flight shortcut decision.
Trust changes always require hosted full success on that exact candidate, regardless of signature.
No candidate-edited classifier/workflow may authorize its own reduction.

## Activation boundary

Bootstrap the implementation with original full CI, then shadow sanity without skipping tests.
Before activation, the owner must reconcile the protocol's current every-push/PR full-CI rule,
approve environment/baseline and signer pins, deploy custody, and enforce the trusted required
check source through repository rules; fixtures cannot establish these external controls.
Activation requires live canaries for ordinary docs, trust changes, forgery and moved heads.
Until these pass, full CI remains the default. Rollback restores unconditional hosted testing
and revokes shortcut authorization without changing local exhaustive tests or acceptance policy.
Receipts attest a trusted signer's observation, not proof certificates or independent execution;
host/signer compromise and unobserved inputs remain outside their assurance.

## Bootstrap operation

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
