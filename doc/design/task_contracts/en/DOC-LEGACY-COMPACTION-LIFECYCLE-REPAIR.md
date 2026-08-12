# Task DOC-LEGACY-COMPACTION-LIFECYCLE-REPAIR: Registered Batch Lifecycle Repair

> Canonical language: English. Japanese companion:
> [../ja/DOC-LEGACY-COMPACTION-LIFECYCLE-REPAIR.md](../ja/DOC-LEGACY-COMPACTION-LIFECYCLE-REPAIR.md).

This is a derived documentation-maintenance contract. It cannot introduce or
override language behavior, test intent, diagnostics, public API, active
behavior, or semantic and coverage credit.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-LEGACY-COMPACTION-LIFECYCLE-REPAIR` |
| Status | Implementation, independent reviews, and required verification complete. Exact task-only commit and clean postcommit proof remain. |
| Purpose | Reconcile the live status field of every schema-2-registered compaction batch with its already committed migration while preserving all historical checkpoints. |
| Primary owners | Repository documentation policy and the checker/test temporary consolidation gate |
| Consumers | The paired batch contracts, checker/test crate plans, the schema-2 ledger consumer, and successor inventory agents |
| Dependencies | Current clean HEAD `9e40f3cfa2d0a0bbd50784efffb71e61aeee4293`; all 32 registered migration histories; schema-2 ledger/lint support; C4C4 closeout `7b53784a6f2525ebb35ce8d59230f07d1c9041bf` |
| Readiness | Unique. Every registered EN/JA batch pair has one stale top-level live `Status`; the committed migration, redirects, ledger rows, owner links, and historical evidence are otherwise consistent. |

The [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index) and
[runner plan](../../mizar-test/en/00.crate_plan.md#task-index) index this
contract.

## Authority And Classification

The ordering authority is the
[temporary consolidation gate](../../todo.md),
under [`AGENTS.md`](../../../../AGENTS.md) and the
[autonomous crate protocol](../../autonomous_crate_development.md#migration-policy).
No language authority is reinterpreted.

| Class | Decision |
|---|---|
| `design_drift` | All 32 registered EN/JA batch pairs retain a live status that says staging, commit, or clean replay remains although their migration is committed and registered. |
| `repo_metadata_conflict` | None in the current checkout. HEAD, remote `origin/main`, and the local tracking ref are the same commit. Historical remote movement remains time-local evidence and is not repaired. |
| `spec_gap`, `test_gap`, `source_drift`, `source_undocumented_behavior`, `test_expectation_drift`, `boundary_violation` | None introduced or repaired by this lifecycle-only task. |

## Read-Only Reconciliation Baseline

At selection, the worktree is clean at
`9e40f3cfa2d0a0bbd50784efffb71e61aeee4293`; remote `origin/main` resolves to
the same commit and `origin/main...HEAD` is `0/0`. Protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` is present and must remain
unchanged.

The schema-2 ledger has 1,024 physical lines, SHA-256
`2a66d200a1976861600bcf7686388faa3efb19b2b42a43c756c9e689d7f27359`,
and exact cardinalities `32/44/4/638/304` for
batch/canonical-task/task-reference/redirect/index rows. The recursive paired
contract, ledger, link, fragment, anchor, count, and expanded-hash lint passes.
Every migration or registration commit below is an ancestor of selection HEAD.

| Batch | Committed migration or registration evidence |
|---|---|
| `DOC-247-COMPLETION-COMPACT` | `75d8af2d5e071f415d1cada9e1a8981aaef2d3b2` |
| `DOC-248P-DOC-REVIEW-COMPACT` | `bee5a905c3e0b291018a33165b382d14bb5eb9fd` |
| `DOC-249M-ACTIVE-EVIDENCE-COMPACT` | `331fdc055d9416225ccc6e2acb22d199c17cb8ee` |
| `DOC-249PI-DOC-REVIEW-COMPACT` | `6b139bf1ab37cdc6c0d7239d202802db1efe113f` |
| `DOC-249S-ACTIVE-EVIDENCE-COMPACT` | `cbacea8efa0c7ac60f16636c2932c49b877e3eae` |
| `DOC-258AB-COMPACT` | `5a83db6f82aa789e31b00601e66d57fe4cda2601` |
| `DOC-258B3M1-M2A-IMPLEMENTATION-LEDGER-COMPACT` | `a9435046608eeb69c8ac284c65b069729d62cab2` |
| `DOC-258B3M2B1-B2A-IMPLEMENTATION-LEDGER-COMPACT` | `e9465ba0ffabf78544cc9ad5663c2d999b6898bf` |
| `DOC-258B3M2B2B1B1P-B1B1-IMPLEMENTATION-LEDGER-COMPACT` | `7467fdc1601479d62002a4e16ee7a07a368519ad` |
| `DOC-258B3M2B2B1P-B1A-IMPLEMENTATION-LEDGER-COMPACT` | `4c030c9d66245439c28ec7659d624aefe414494f` |
| `DOC-258B3M2B2B2C-FINAL-REVIEW-COMPACT` | `9b356722d29c26ffc1ba5e927112555ead51babb` |
| `DOC-258B3M2B2B2C-IMPLEMENTATION-LEDGER-COMPACT` | `b91ca9cfe9eb4789045eda271db8160c226e3133` |
| `DOC-258B3M2B2B2C-RUNNER-IMPLEMENTATION-COMPACT` | `7f771af69cb2ffed9d9c7f784c5b723c7f22b977` |
| `DOC-258B3M2B2B2CP-IMPLEMENTATION-COMPACT` | `0343f8e7ef47d6b24a64e8b14b3a85f600a95380` |
| `DOC-258B3M2B2B3-ACDE-COMPACT` | `9c31231eae4a0bb1cff9d6bb037ab030eb2d5fef` |
| `DOC-258B3M2B2B3P-REVIEW-COMPACT` | `787c16fb682db58f2a9fddc0d3f9aee1f9fd22bf` |
| `DOC-258B3N-IMPLEMENTATION-LEDGER-COMPACT` | `b4f97b2ea5f9bba17bf084929214b749389b08b9` |
| `DOC-258B4A-COMPACT` | `fee14f18c2301b1523250f25843d96b91f759b8e` |
| `DOC-258B4B-COMPACT` | `1d32ed06cc110ed98e9116dd59af82e9ef724b15` |
| `DOC-258B4C-DOC-REVIEW-COMPACT` | `d94dfd6330c1dd067be8b26c814ac95e077b2639` |
| `DOC-258B4C-IMPLEMENTATION-LEDGER-COMPACT` | `71edf3400bd8da556322c0510d6824bb62302c60` |
| `DOC-258B5A-COMPACT` | `95b4ce9801bc0b5ec85dbdba30d40ec26d44d3d7` |
| `DOC-258B5A-IMPLEMENTATION-LEDGER-COMPACT` | `ada9f5a3c773dc59687462dbd2a0be72bee03157` |
| `DOC-258B5B-UPPER-IMPLEMENTATION-LEDGER-COMPACT` | `440d27ae6e42f0aef6a58578a643ec5461763af3` |
| `DOC-260-DOC-REVIEW-COMPACT` | `9451e57df52dc105a3faa2348432e3d81642519a` |
| `DOC-269A-DOC-REVIEW-COMPACT` | `a9d5f40650d2ed694ba9304e2448fbd95e272406` |
| `DOC-269B-DOC-REVIEW-COMPACT` | `1ad52ed39cfa98d9a9b08f639e2d75f123de80cf` |
| `DOC-269CTGP-COMPACT` | `f77f68f9b0bd48c681396afb4125cba343a294a8` |
| `DOC-269G-COMPACT` | `34b42908fcc3a7734200e962878dca02b6dafe8f` |
| `DOC-269G-INTERMEDIATE-COMPACT` | `f3dd80bc396d17a76d8bf127f34b2e9f519999c7` |
| `DOC-269GT-COMPACT` | `a1bf34e86b42b19a81cf7ca07bb1e420a266637f` |
| `DOC-269SD-COMPACT` | Migration `5080d3fddaad6e9683e5eecc5e497b4b16908e8a`; later ledger registration `0ec5fce293a6105e04761c5298b605d3f4ff60ca` |

`DOC-269SD-COMPACT` predates the data-driven ledger. Its later backfill is the
reviewed generic-ledger implementation, so the distinct migration and
registration commits are consistent rather than a metadata conflict. The
other 31 batch rows were added by their listed task-local migration commits.

## Frozen Repair Surface

Implementation changes exactly these current-state owners:

1. the one top-level `Status` field in each of 32 registered English batch
   contracts and its 32 Japanese companions;
2. this paired contract and one compact row in each checker/test EN/JA Task
   Index; and
3. only temporary-gate checklist items 1 and 2 in `doc/design/todo.md`.

The exact English replacement is:

```text
| Status | Complete. The migration is registered in the schema-2 ledger; task-local completion evidence below preserves the committed migration and clean replay. |
```

The synchronized Japanese replacement value is:

```text
完了。migrationはschema-2 ledgerに登録済みであり、task-local completion evidenceがcommitted migrationとclean replayを保存する。
```

The existing field label is preserved. Therefore 31 Japanese paths use
`| Status | <replacement value> |`, while
`doc/design/task_contracts/ja/DOC-258B4B-COMPACT.md` preserves its historical
lowercase label as `| status | <replacement value> |`. No other byte in those
rows is exempt from the frozen replacement.

The task changes 71 paths: 64 registered batch contracts, this EN/JA pair,
four crate plans, and the top-level roadmap. Task-contract counts become
`95/95`, `doc/design` Markdown file count becomes 824, and all 64 stale live
status values become complete while their field-label spelling is preserved.
The ledger receives no row and remains byte
identical. `doc/design/spec_coverage_audit.md` has no mapping, owner,
traceability, deferral, or credit impact and remains unchanged.

## Protected And Forbidden Changes

Historical precommit, implementation, review, migration, postcommit, and
handoff prose is immutable. Existing owner links, redirects, fragments,
headings, anchors, task rows, hashes, and counts are immutable. Do not edit
`doc/spec`, `.miz`, expectation, trace, production or test Rust, Cargo,
diagnostic, active route/result, test intent, or semantic and coverage credit.
Do not select or compact a new family in this repair task.

## Reviews, Verification, And Exit

Before implementation, an independent specification/equivalence reviewer must
end at **NO FINDINGS**. After implementation, independent test-sufficiency,
implementation/equivalence, source/documentation, bilingual/boundary, and
final-quality reviews must end at **NO FINDINGS**. All nine protocol hard
gates must pass without a score cap and final quality must be at least
`90/100`.

Verification includes case-aware exact 64-value replacement with all existing
field labels preserved, absence of stale live status
wording in the registered pairs, byte-identical ledger/hash/count replay,
recursive contract/link/fragment lint, Markdown/count inventory,
`git diff --check`, protected-path review, formatting, warnings-denied Clippy,
and full workspace tests. Exit requires exact task-only staging and commit,
clean postcommit worktree/origin/stash proof, and a fresh schema-2 family
inventory. The successor inventory must not assume that any unregistered
family is safe.

## Handoff

After the clean repair commit, re-run the temporary gate's schema-2-safe family
inventory. If no dependency-ready family exists, freeze a bounded closeout
that records registered totals and residual shape classes without claiming
repository-wide consolidation. Parent authority reasoning remains `xhigh`;
bounded independent review may use `high`.
