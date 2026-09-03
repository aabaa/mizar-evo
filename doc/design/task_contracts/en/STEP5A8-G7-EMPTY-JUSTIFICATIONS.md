# Task STEP5A8-G7-EMPTY-JUSTIFICATIONS: uniform omitted justifications

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A8-G7-EMPTY-JUSTIFICATIONS.md](../ja/STEP5A8-G7-EMPTY-JUSTIFICATIONS.md).

Owning plan: [mizar-parser](../../mizar-parser/en/00.crate_plan.md).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Complete; hard gates 9/9 pass; final quality score 100/100 |
| Tier | Full: specification, `.miz`, expectations, traceability, parser behavior, and cache identity |
| Owner / consumers | `mizar-parser` owns syntax admission and recovery; `mizar-frontend` owns parser-cache invalidation; `mizar-test` owns corpus execution |
| Dependencies | Step 5A.1-5A.7 complete; no semantic dependency |
| Authority | [Chapter 15 §15.8](../../../spec/en/15.statements.md#158-justification-forms), [Chapter 16 §§16.5-16.6](../../../spec/en/16.theorems_and_proofs.md#165-proof-justification-by), [Appendix A.15-A.16](../../../spec/en/appendix_a.grammar_summary.md#a15-statements-proofs-and-references), and the approved 2026-09-03 decision |
| Classification | Human-resolved `spec_gap`; bounded `source_drift`, `test_gap`, `test_expectation_drift`, and `design_drift` |
| Semantic-credit throughput | `0 tasks/week`; this is syntax/trace coverage and grants no proof acceptance |

## Frozen language decision and boundary

The empty branch of `justification ::= [ simple_justification ] | proof |
computation_proof` is intentional. In every grammar production that requires a
`justification`, the obligation keyword may therefore be followed immediately
by `;`. Omission supplies no citation or proof block: the verifier must still
discharge the unchanged obligation using its no-hint policy, and failure is a
semantic proof failure rather than a syntax error. The paired Chapter 15/16,
Appendix A, and sample-code edits record this approved intent; they do not make
an obligation true or accepted.

At selection HEAD `89881f3d`, generic definition correctness conditions already
implement this rule. Five private parser paths instead emit
`malformed_justification`: registration correctness, mode `sethood`, predicate/
functor property clauses, inheritance coherence, and redefinition coherence.
Each path shall accept omission only when its cursor is already at `;`; it emits
no fabricated `JustificationClause` or `MissingProofStep`. Explicit `by`, proof,
and computation forms remain unchanged. Missing `coherence with` labels,
malformed `by` clauses, non-semicolon junk, wrong correctness keywords/order,
missing mandatory property-implementation existence/uniqueness blocks, and
all synchronization boundaries retain existing fail-closed diagnostics.

No syntax kind or public API changes. Because identical tokens and parser
inputs can now yield a different AST/diagnostic result, the real frontend
parser cache namespace changes from `mizar-parser/surface-ast-v6` to v7.
The paired parser grammar updates Tasks 26-30 in place; the paired frontend
`parsing.md` and `cache_key.md` documents own the targeted v7 seam/cache delta.
Proof construction, automatic discharge, correctness acceptance, facts,
registration activation, sethood evidence, redefinition resolution, and
checker diagnostics remain downstream and unchanged.

## Test-first and artifact contract

Before Rust changes, add exactly one active parse-only pair:
`tests/miz/pass/parser/pass_parser_empty_justifications_001.{miz,expect.toml}`.
The source covers empty generic conditions, property clauses, mode `sethood`,
redefinition coherence with and without `with`, property-implementation and
inheritance coherence, and registration existence/coherence/reducibility. Its
metadata is schema 1, matching id/source, pass/parse-only, domain
`parser.empty_justification`, empty diagnostics, and `active_parse_only`. Its
exact `spec_refs`, each receiving the sole new backlink, are
`spec.en.05.structures.parser`, `spec.en.07.mode_definitions.parser`,
`spec.en.07.modes.property_implementation.parser`,
`spec.en.16.correctness_conditions.parser`,
`spec.en.17.clusters_and_registrations.parser`,
`spec.en.syntax.property_clauses.parser`, and
`spec.en.syntax.redefinition_notation.parser`. Do not change their status,
stage, coverage, or the 499-requirement count.

Update only the diagnostic lists and notes of the four existing recovery
expectations whose sources intentionally contain now-valid bare forms:
mode definitions (`sethood`), property clauses (`symmetry`), redefinition
notation (`coherence`), and registrations (`reducibility`). Their `.miz`, kind,
stage, outcome, reason, detail key, tags, and trace state remain unchanged; all
other malformed constructs must still fail in the same order. Matching exact
metadata assertions change with those lists.

Add focused parser tests for all five corrected private paths, absence of
fabricated justification/recovery nodes, and unchanged malformed `by`/junk
rejection. Add one real-frontend v7 test and deterministic replay. Baseline is
557 cases / 499 requirements / 314 pass / 243 fail / 108 active parse-only;
expected delta is +1 case / +0 requirements / +1 pass / +1 parse-only.

## Protected scope, reviews, and exit

Existing `.miz` sources, non-G7 expectations, oracle activation, trace states,
`tests/coverage/step5_activation_map.tsv`, and `doc/design/archive/` are
immutable. The archive baseline is 13 files; hashing the sorted per-file
SHA-256 manifest yields
`934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.
The four expectation edits follow the approved specification, not observed
implementation. No semantic oracle or 5C activation-map row may be activated.
`doc/design/spec_coverage_audit.md` has no impact because coverage status,
owner, and deferred rationale stay unchanged. Task 277B remains not-ready and
zero-credit.

Independent specification/equivalence and boundary reviews precede test or
Rust edits. Test-sufficiency, implementation, source/docs/API, and final score
reviews follow. Verification requires focused parser/frontend/metadata and
parse-only checks, link/ledger lint, `cargo fmt --all --check`, warnings-denied
workspace Clippy, and full workspace tests. Exit requires hard gates 9/9, a
valid score of at least 90/100, exact task-only staging, a local commit, clean
postcommit proof, and fresh dependency-minimal selection of Step 5A.9.

## Completion evidence

The test-first source initially failed with exactly eight
`malformed_justification` diagnostics. The five private paths now accept only
an immediate semicolon and create no justification or recovery node. Malformed
`by`, junk, and missing `coherence with` labels remain rejected. The frontend
uses parser cache v7 and replays the corrected AST deterministically.

Independent specification, boundary, test-sufficiency, implementation, and
source/docs/API reviews ended with no findings after targeted repairs. The
four existing expectations changed only by removal of their now-valid bare-form
diagnostic and a matching note; their sources and rejection identities remain
unchanged. The coverage audit has the frozen no-impact disposition.

Verification passed: focused parser/frontend tests; all 141 metadata tests;
plan 558 cases / 499 requirements / 315 pass / 243 fail / 0 errors / 23
pre-existing warnings; parse-only 109/109; `cargo fmt --all --check`;
warnings-denied workspace Clippy; and full workspace tests plus doctests.

| Hard gate | Result |
|---|---|
| 1. specification consistency | Pass: approved EN/JA intent and reviews agree |
| 2. documented/tested behavior | Pass: five bounded paths and real frontend evidence |
| 3. milestone `.miz` coverage | Pass: one test-first active pair |
| 4. expectation integrity | Pass: four authority-driven diagnostic removals only |
| 5. design/source synchronization | Pass: paired parser/frontend owners agree |
| 6. responsibility boundaries | Pass: no proof acceptance or public API change |
| 7. coverage-audit disposition | Pass: explicit no-impact; audit byte-identical |
| 8. verification | Pass: every required command completed |
| 9. residual-risk classification | Pass: proof discharge and 5C activation deferred |

Activation map, coverage audit, and the 13-file archive retain their frozen
hashes; oracle activation and trace states are unchanged. Task 277B remains
not-ready/zero-credit, and semantic-credit throughput remains `0 tasks/week`.

Next handoff: select Step 5A.9 by fresh inventory. Keep Sol xhigh for authority,
boundary, and final gates; use Luna xhigh only for frozen inventory, localized
implementation, focused verification, and first-pass independent reviews.

The read-only advisory score was 100/100 with no cap; parent Sol independently
confirmed hard gates 9/9 and the final 100/100 score.
