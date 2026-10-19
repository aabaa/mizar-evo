# Module: miz_corpus

> Canonical language: English. Japanese companion: [../ja/miz_corpus.md](../ja/miz_corpus.md).

## Purpose

This module defines the strategy for building and maintaining a large `.miz` corpus.

The corpus is an implementation asset. It is used to pin syntax, type behavior, cluster behavior, overload behavior, proof rejection, dependency fingerprints, and deterministic artifact output.

## Corpus Classes

| Class | Purpose | Ownership |
|---|---|---|
| handwritten minimal cases | focused parser/type/cluster/proof behavior | reviewed by developers |
| migrated examples | realistic source patterns from existing Mizar-like material | reviewed before acceptance |
| generated `.miz` | broad coverage of grammar and semantic combinations | generated with stored seeds |
| fuzz-minimized reproducers | permanent regression cases from fuzz failures | committed after minimization |
| bug regressions | protect fixed bugs and soundness failures | linked to issue/PR when possible |
| stress/integration articles | large-file and incremental rebuild behavior | reviewed for stability |

## Growth Targets

| Stage | Target |
|---|---|
| early module development | 100-300 `.miz` files |
| evo2 alpha | 300-1,000 `.miz` files |
| evo2 beta / release candidate | 1,000-5,000 `.miz` files |
| mature ecosystem | 500,000-1,000,000 LOC equivalent including generated/fuzz/property corpora |

Growth should favor high-signal fail/soundness tests near the kernel over large numbers of shallow pass tests.

## File Size Guidelines

| Purpose | Lines |
|---|---:|
| parser test | 5-30 |
| type test | 10-50 |
| cluster test | 20-80 |
| theorem test | 30-150 |
| integration test | 100-300 |
| stress test | 500-1,000 |

Validation treats these ranges as upper-bound review gates. Short minimal
regressions are allowed without diagnostics because many fail/soundness cases
are intentionally tiny. Oversized generated `.miz` files are errors unless they
live under `tests/stress/`; oversized handwritten `.miz` files are warnings.

## Generation Policy

Generated `.miz` files must record:

- generator name and version;
- seed;
- generation profile;
- expected outcome;
- minimization status;
- schema version for metadata.

Generated, fuzz, and property sidecars record this provenance in `[origin]`.
Metadata-only handoff anchors may use the crate-local test family or harness
handoff as the generator, `generator_version = "handoff"`, and the fixture's
stable phase/name as the seed. `origin.expected_outcome` mirrors the harness
sidecar outcome. All fuzz seeds record `origin.original_failure_category`; for
promoted fuzz failures, that original fuzz failure class must match the
executable `failure_category`.

Generated tests are committed only when they add coverage, reproduce a bug, or serve as stable stress cases. Bulk generated corpora may live outside the default fast test set until they are minimized or promoted.

## Review Rules

Corpus additions are reviewed for:

- stable expected outcome;
- deterministic diagnostics and snapshots;
- absence of hidden reliance on test execution order;
- minimality for fail/soundness regressions;
- clear domain placement and naming.

Generated sidecars live under `tests/generated/` unless they are stress cases
under `tests/stress/`. Fuzz seeds live under `tests/fuzz/`; property seeds live
under `tests/property/`. Unminimized generated/fuzz/property seeds stay outside
the default `fast` profile. Metadata-only fuzz handoff seeds use the
`fuzz_regression` profile. Stress cases use `profiles = ["stress"]` and do not
also opt into `fast`.

## Tests

Key scenarios:

- generated tests can be reproduced from stored seed metadata;
- minimized fuzz reproducers preserve the original failure category;
- corpus manifest counts pass/fail ratios by domain;
- stress tests are excluded from default fast runs unless requested.

## Constraints and Assumptions

- `.miz` corpus files are long-lived compatibility inputs.
- Fail tests are not loosened to match current compiler behavior.
- A soundness regression case is never deleted without architecture-level review.

## Checker Task 263 Frozen Corpus Increment

After the separate documentation prerequisite, Task 263 may add exactly one
canonical-derived pass source/sidecar pair named
`pass_type_elaboration_structure_definition_payload_001`. Its source is the
frozen 320-byte/final-LF text with SHA-256
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`.
The sidecar is pass/type_elaboration/type_check with empty public diagnostics
and payloads and one new Task-263 trace reference.

The existing mixed mode/structure-definition failure pair and every parser/
resolver structure fixture remain byte-identical. The docs prerequisite adds
no corpus file; implementation projects cases `425 -> 426` and pass cases
`232 -> 233` only after fresh inventory.

## Checker Task 263 Active Corpus Increment

The sole new pair is
`pass_type_elaboration_structure_definition_payload_001.miz` and its matching
sidecar. The source is exactly 320 bytes/16 lines with SHA-256
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`;
the sidecar hash is
`d82c8d3102ea34fdb4a32792167c4b109b96b9c05265d3f04e6310278178e8ac`.
All existing `.miz` and expectations remain byte-identical. Cases are
`426`, pass/fail is `233/193`, and active type cases are `203`.

## Task 257C4C0 frozen corpus increment

The [C4C0 contract](../../task_contracts/en/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md)
owns one implemented handwritten pass pair under `tests/miz/pass/types/` for
the Chapter 13 nested-comprehension outer-generator capture oracle. The pair
inventory is `344/344`; the new pair remains inactive and earns no
current parser, semantic, route, warning/error, or Task-277B credit. The exact
source bytes/hash and sidecar are owned by the contract; local-lookalike and
builtin-set variants are forbidden.

## Task 257C4C1 explicit-import corpus repair

The [C4C1 contract](../../task_contracts/en/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md)
has modified that existing oracle in place to the exact 164-byte explicit-import
source and updates only its inactive sidecar note. One separate 140-byte module
at `crates/mizar-test/tests/testdata/parser/nested_capture_fixtures.miz`
defines the ordered `Element`/`NAT` lexical shapes in separate definition
blocks. It is crate-local testdata, not an ordinary corpus payload, and has no
sidecar. Corpus pairs therefore remain `344/344`; active cases, outcomes,
routes, warnings/errors, capture credit, and Task-277B credit do not change.
The measured file hashes and unchanged inventories are recorded once in the
contract's [precommit completion
checkpoint](../../task_contracts/en/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md#precommit-implementation-completion-checkpoint).

Independent source-documentation consistency and bilingual/boundary reviews
ended with **NO FINDINGS**.
Independent final-quality review also ended with **NO FINDINGS** and is linked
through the canonical [precommit completion
checkpoint](../../task_contracts/en/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md#precommit-implementation-completion-checkpoint).
The exact staging/cached review recorded at that historical review-time
checkpoint also ended with **NO FINDINGS**.
The task-only commit, post-commit proof, and accepted fresh-inventory STOP are
closed at the language-local [historical postimplementation
checkpoint](../../task_contracts/en/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md#historical-immediate-postimplementation-pre-closure-checkpoint).
The inactive, zero-credit corpus disposition remains unchanged.

## Task 257C4C7 Frozen Corpus Increment

The [C4C7 contract](../../task_contracts/en/TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7.md)
owns one exact new inactive pair under `tests/miz/pass/types/`. Its 193-byte
source combines the independently specified multiple-generator and nested-
capture forms: inner mapper `[x, y]` references exactly two resolved outer
generators, while inner `z` is local. The sidecar asserts membership/cardinality
only and explicitly withholds ordering, execution, Core, and Task-277B credit.
The completed increment projects corpus pairs `344/344 -> 345/345`; the
existing one-capture pair remains byte-identical. Exact completion evidence is
owned by the linked contract.

## September 2026 Audit-1 Semantic-Bridge Oracle Increment

The September 2026 audit wave added 120 handwritten inactive oracle pairs
(60 pass / 60 fail) spanning spec chapters 3-20, with 103 traceability
requirements, under the chapter-aligned pass/fail subdirectories listed in
[layout.md](./layout.md). All pairs follow the Task 257C4C0 inactive-seed
convention: no `active_*` tag, no current runner credit. Coverage,
conventions, and spec findings are owned by
[semantic_bridge_corpus_map.md](./semantic_bridge_corpus_map.md); the 29
sources currently blocked by frontend gaps are inventoried in
[semantic_bridge_frontend_gaps.md](./semantic_bridge_frontend_gaps.md) and
`tests/coverage/audit1_frontend_gaps.tsv` (note: one blocked source
reproduces a parser panic, so corpus-wide tooling must not assume every
committed `.miz` parses until gap G5 closes). Corpus pairs move
`345/345 -> 465/465`; discovered cases move `430 -> 550`.

## Multiple-argument Binder Acceptance Tests

Test design for [§13.10](../../../spec/en/13.term_expression.md#1310-function-binder--) and [§20.5.3](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax).

These requirements fix future test intent, not current implementation success. Stage-specific fixtures require authentic producers; existing tests/expectations are unchanged.

| ID / stage | Required positive and rejection observations |
|---|---|
| MB1 / syntax | Unary, binary, ternary and four-head forms; weakest precedence, right nesting, body versus whole-Binder qua, application/restriction and use inside a comma-separated call. Reject empty, repeated, trailing-comma, unparenthesized multi-head, typed and destructuring heads. |
| MB2 / formation | Infer REAL×REAL and NAT×REAL; explicit certified product expectation selects its factors, including narrower factors with all-input type proofs. Reject missing carriers, incompatible factors and non-product expectations without fallback or factor guessing from equalities. |
| MB3 / empty domains | An explicit empty factor produces an empty function, including empty codomain; no values/environments are manufactured. Missing factors remain errors even if another is empty; unsupported body forms still reject; nonempty domain with empty codomain rejects. |
| MB4 / scope and obligations | Fresh UIDs ignore outer values/individual assumptions; retain outer type dependencies and free identities. Reject unavailable declarations, fresh-head-dependent domains/types and unproved body/application/range conditions; restriction cannot rescue failed formation. |
| MB5 / meaning | Binary f.[a,b], ternary f.[[a,b],c] and four-coordinate packing follow the pointwise equation; all coordinates update simultaneously. Distinguish curried functions, unary pair arguments and multiple heads without implicit conversion. |
| MB6 / capture | Match exact arity independently of display names; preserve original coordinate types/order, actual domain and external/extended contexts, even for empty domains. Mismatched arity does not match. |
| MB7 / reconstruction | Unchanged round trip is structurally equal; explicit codomain changes retain range proofs. Function requires saved-domain equality, PartFunc inclusion without enlargement. Reject reordered/incomplete/mixed capture groups or missing C, including equal UIDs with different default codomains. |
| MB8 / structure and R2 | Exact child order/duplicates, size recurrence, free-UID removal for every head, coordinate-valued pair versus separate coordinates, deterministic inner-collision renaming and unavailable-free-declaration rejection follow §20.5.3. |
| MB9 / certification | Certified packing/decomposition and function construction prove the ordinary/reflected meaning correspondence; missing library/per-use proofs or same-spelling substitutes supply no axiom or acceptance. |
| MB10 / later differentiation | For (x,y) ↦ x*y+sin y on REAL×REAL, coordinate 1 yields y and coordinate 2 yields x+cos y. Require all-real update admissibility in every other-coordinate environment and preserve all head identities/order; this is a future library test, not a partial-differentiation API definition. |
