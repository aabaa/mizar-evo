# Task STEP5C11-CLUSTER-REGISTRATIONS: cluster source bridge

Canonical language: English; [Japanese pointer](../ja/STEP5C11-CLUSTER-REGISTRATIONS.md).
Owning plans: [checker](../../mizar-checker/en/00.crate_plan.md#task-index),
[test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: partial; full tier; checker owns registration semantics; test owns admission.
- Dependencies: 5A.2, 5B.2, 5C.3 and 5C.10 complete.
- Authority: [Chapter 17](../../../spec/en/17.clusters_and_registrations.md)
  §§17.1, 17.3–7, 17.10; the seven 5C.11 rows of the
  [activation map](../../../../tests/coverage/step5_activation_map.tsv).
- Current increment: activate only `fail_parse_only_cluster_adjective_argument_list_001`.
  Classification: executable test gap; parser already rejects argument-bearing adjectives.
- Remaining registration sources need source-derived validation and correctness
  obligations. The four positive rows stop at cluster resolution and may remain
  pending; actual registration effects require Task 274's accepted-status authority.
  Negative correctness checks still require source-derived obligations; the reduction
  negative also needs reconciliation of its proof-phase expectation with §17.6.4.

## Scope and boundaries

The current increment adds the sole `active_parse_only` tag without changing
the source, expected phase/outcome, empty public codes or stable detail key.
Authenticate source/sidecar identity, stage, phase, outcome, detail key and tag.
Accept the frozen failure only from the real parser diagnostic at an argument-bearing
attribute within a registration header; unrelated errors and repaired syntax fail
this negative oracle. Reject malformed admission and cross-stage fallback.
Add focused source/metadata/diagnostic mutations, including alpha-renaming.
Reuse the existing parse runner; add no public types, semantic adapter or parser policy.

Task 274 remains blocked-reserved: pending correctness obligations never become
accepted status, active registrations, verified closure or reduction evidence.
Do not change `.miz`, existing expectation intent, trace status, activation-map
ownership, diagnostics, specification, archive or volume ratchet.
Preserve completed profiles and Task 277B's not-ready/zero-credit boundary.
Owner details: [registration](../../mizar-checker/en/registration_resolution.md),
[harness](../../mizar-test/en/harness.md). Audit impact: Chapter 17 parser coverage
and explicit remaining registration deferral only.

## Exit

Review specification/docs, test sufficiency, implementation, volume and consistency.
Run focused parse tests/corpus, fmt, warnings-denied Clippy and cargo test.
Commit the bounded increment; keep the parent task open until all seven mapped
outcomes reach their unchanged stages through real producer/consumer seams.
