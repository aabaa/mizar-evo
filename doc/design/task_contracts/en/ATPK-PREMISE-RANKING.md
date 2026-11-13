# Task ATPK-PREMISE-RANKING: deterministic premise ranking
Canonical language: English; [Japanese pointer](../ja/ATPK-PREMISE-RANKING.md).
Status: S1–S3 implemented within authenticated inputs; S4 and D1–D4 deferred. Tier: full. Owner: [ATP plan](../../mizar-atp/en/00.crate_plan.md), task 31; prerequisite: [ATP37](ATP-STRUCTURED-LOCAL-RANKING.md).
Consumers: [VC](../../mizar-vc/en/00.crate_plan.md), [resolve](../../mizar-resolve/en/00.crate_plan.md), [artifact](../../mizar-artifact/en/00.crate_plan.md), [test](../../mizar-test/en/00.crate_plan.md).
Authority: specification §22.4.4, §21.4.1 (`max_axioms`), §22.4.5 (E0321), §23.5.2 (suggestion result shape); [architecture](../../architecture/en/09.atp_interface_protocol.md).
Gap: source_drift/test_gap; bounded ranking can start; complete integration remains blocked on visible-fact inputs, VC assembly and artifacts.

## Definitions
Authenticated inputs: existing producers' immutable normalized AtpProblem, structured source projections/registrations, and policy/configuration records carrying original identities and provenance for goals, facts, budgets and configuration; never reconstruct payloads from text, hashes or opaque bytes.
Ranking: deterministic integer `symbol-overlap-v1`, sharing more goal symbols raises relevance, rare symbols weigh more relative to supplied candidates, one total deterministic tie-break; record configuration with every result, reuse ATP37 scoring.
Bounded slice: S1–S3 only over authenticated inputs; S4 is deferred.
Deferred integration: D1–D4 are out of scope; no placeholders or bounded-exit dependencies.

## Scope
S1: ranking core shared by S2–S3; reuse AtpProblem::rank_axioms() wherever it supplies the order.
S2: prune assembled facts to exactly the top authenticated `max_axioms` budget when exceeded; otherwise unchanged; caller-supplied authenticated failed pruned attempt returns E0321 outcome value.
S3: ranked suggestions for failed/open obligations; optional authenticated retry failure leaves hints, success classifies a suggestion with the resolving candidate first; classification never changes proof acceptance.
S4: deferred by the 2026-11-14 scope revision: no authenticated producer joins existing labels to normalized candidate formulas and an unresolved-reference goal; supplying one requires D1/D2. S4 returns only in a future unit after D1/D2 exist.
D1: visible-fact index/producer from summaries/imports deferred.
D2: VC selection assembly and live verifier/ATP wiring deferred.
D3: missing_facts.json, diagnostic note/help/hint text and IDE fixes deferred.
D4: complete-integration exit deferred, not done.
Forbidden: uncited source-proof acceptance, learned configurations, nondeterministic scores/orders, proof-acceptance changes, doc/spec or .miz expectation edits.
Stop: if S1–S3 requires D1–D4, report the exact missing piece; if a required term lacks a dispatch/contract/spec definition, report it rather than inventing one.

## Tests and exit
P1 rare-symbol order; P2 total tie-break/repeatability; P3 over-budget pruning; P4 unchanged within budget; P5 failed pruning E0321.
P6 suggestions without retry; P7 failed retry hints; P8 successful retry/resolving-first/unchanged-proof refusal. P9 (S4 unresolved citations): out of scope under the 2026-11-14 revision.
R1 corrupt visible-fact identity; R2 corrupt source provenance; R3 unknown/corrupt configuration; R4 missing goal/candidate payload; R5 uncited-proof refusal.
Binding matrix: P1–P8 and R1–R5 (13 categories), fixed by the revised 2026-11-14 dispatch; cap 24, no splits.
E1 bounded exit: S1–S3 are implemented and verified against the binding matrix; every result records `symbol-overlap-v1`; corrupt identities fail closed without ranking; no accepted proof gains an uncited fact; S4 is excluded from this unit’s completion; S4 and D1–D4 remain deferred, not done.
Deferred complete-integration exit: every ranking use in §22.4.4 is served by the recorded configuration, and no accepted source proof gains an uncited fact.
Verification/reviews: follow full-tier AGENTS.md and protocol cadence, including specification, sufficiency, implementation, volume and consistency reviews.
