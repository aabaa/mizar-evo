# Task STEP5C9-PROOF-ORGANIZATION: bounded statement organization

Canonical language: English; [Japanese pointer](../ja/STEP5C9-PROOF-ORGANIZATION.md).
Owning plans: [checker](../../mizar-checker/en/00.crate_plan.md#task-index),
[resolve](../../mizar-resolve/en/00.crate_plan.md#task-index), [test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: complete; full tier; primary owner: checker; dependencies: 5A.4, 5B.2 complete.
- Reuse the completed occurrence and formula bridges without changing their profiles.
- Authority: [Chapter 15](../../../spec/en/15.statements.md) §§15.2.1, 15.3.3–4,
  15.4.1, 15.4.3–4, 15.6–8, 15.10–11; [Chapter 4](../../../spec/en/04.variables_and_constants.md)
  §4.6; [Chapter 14](../../../spec/en/14.formulas.md) quantification and connectives;
  [Chapter 13](../../../spec/en/13.term_expression.md) set enumeration.
- Classification: executable `test_gap` / `source_drift`; no specification change.
- Exact sources and expectations: the seven non-G3 Step 5C.9 rows in the
  [activation map](../../../../tests/coverage/step5_activation_map.tsv).

## Scope and boundaries

Activate the mapped `consider_choice`, `now_diffuse_statement`,
`given_existential_assumption`, `hereby_diffuse_conclusion`, `iterative_equality`,
and `per_cases_suppose` pairs at `formula_statement` / `statement_check`.
Activate only the mapped `per_cases_incomplete` failure at `proof_verification`
/ `verification`, key `theorems.per_cases.incomplete_case_split`.
All public code lists remain empty; add only each stage's sole active tag.
Authenticate ids, source/sidecar paths, stage, phase, outcome, key and tag;
malformed candidates must not fall through to a legacy route.

Resolver owns Given/Consider binding identities, block visibility, shadowing,
and label/citation site and origin; new opt-in entry points preserve older routes.
Checker owns builtin atom/term typing, ordered generalization and witnesses,
existential hypothesis/citation instantiation, block conclusions and references,
iterative equality endpoint/step agreement, and branch goal/assumption isolation.
Use the existing typed arena, primary/atomic/set handoffs and sealed scope/labels;
do not extend or duplicate term, formula or statement representations.
The harness only assembles those inputs and dispatches checker results.
Case completeness is checked from the actual branch propositions, separately
from branch typing/goal matching; the frozen negative must reach verification.
Bounded reflexive equality and complementary branch conditions suffice here;
unsupported obligations fail closed and never earn the frozen negative key.
No case-id, spelling-only, source-text equality or arbitrary-error acceptance.

General proof search, imported facts, richer terms/types, arbitrary block closure,
and theorem/certificate acceptance are deferred. Preserve G3 then/hence inactivity,
existing `.miz`/expectation semantics, trace/map/ratchet, archive, completed tasks,
kernel/certificate boundaries and Task 277B not-ready/zero-credit.
Owner details: [names](../../mizar-resolve/en/names.md), [labels](../../mizar-resolve/en/labels.md),
[checker](../../mizar-checker/en/type_checker.md), [harness](../../mizar-test/en/harness.md).
Audit impact: bounded Chapters 4/15 coverage, not general proof acceptance.

## Exit

Review specification/docs, tests, implementation, removable volume and consistency.
Test all seven outcomes plus binding/label provenance, witness type/substitution,
scope leaks, endpoint drift, branch/citation mismatch and completeness mutations.
Run focused tests/corpora, fmt, warnings-denied Clippy and cargo test.
One local commit; production ≤1,500 added lines, documentation ≤200, no new public types.
