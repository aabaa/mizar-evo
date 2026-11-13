# Task CHECKER279-REDEFINITION-REMAINDER: source redefinition transport
Canonical language: English; [Japanese pointer](../ja/CHECKER279-REDEFINITION-REMAINDER.md).
Status: complete. Tier: full; further Task279 slice fails the new-public-seam pattern criterion.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [runner plan](../../mizar-test/en/00.crate_plan.md), real MT10-AS Task10 Case.
API/invariants: [source redefinition](../../mizar-checker/en/source_redefinition.md); consumer boundary: [harness](../../mizar-test/en/harness.md).
Authority: EN [06.7](../../../spec/en/06.attributes.md#67-redefinition), [09.6–9.7](../../../spec/en/09.predicates.md#96-predicate-redefinition), [10.7–10.8](../../../spec/en/10.functors.md#107-functor-redefinition), [11.1](../../../spec/en/11.symbol_management.md#111-synonyms-and-antonyms), [19.5](../../../spec/en/19.overload_resolution.md#195-symbol-redefinition-redefine).
Dependencies: [278](CHECKER278-OVERLOAD-SELECTION.md), [279A](CHECKER279A-INTRINSIC-REDEFINITION-TARGETS.md), [279B](CHECKER279B-EXPLICIT-LOCAL-TARGETS.md); [acyclic graph](../../mizar-checker/en/payload_family_decomposition.md#producer-contract-shared-by-tasks-248-264-and-269-279).
Gaps: source_drift/test_gap in relation, intent, diagnostic, refinement/coherence-input and exposed-view transport.
## Frozen boundary
Extend source_redefinition and its sealed handoff in both AST containers; no new owner module or consumer.
Retain intrinsic attribute, binary predicate and unary equals-functor restrictions and ordinary-root authentication from Tasks279A/B.
Allow one optional notation alias following an ordinary declaration in its definition block: bare attribute synonym or binary predicate antonym with the same ordered formal loci.
Bind that relation to the actual earlier ordinary declaration in the block; authenticate both patterns, alias kind/site and ordinary root, without implementing alias use or semantics.
Allow a final builtin Object reserve and reflexive equality theorem containing one unary ordinary functor application; its proof remains unchecked.
Retain explicit local `coherence with` label-token intent or omitted-target intent, actual target/coherence/obligation association, and a source-anchored bound-target diagnostic payload.
Expose a source-derived input with ordered target identity, relation, intent, diagnostic, Pending coherence-input and refinement candidate data; fresh equality validation rejects caller substitutions.
Each redefinition binds an already-identified ordinary root; never add or send a new ordinary-root candidate back into Task278.
For the real application, authenticate Pending refinement candidates from the bound target and source guard/result sites in the root normalization domain.
The ordinary-root handoff first seals the actual application graph; Task279 reuses it without rerunning candidate stages. Its selection intake authenticates Pending refinements and supplies an empty active set, with no ordering edges.
Expose the ordinary selected-root result with empty evidence and inserted views; declaration-only inputs retain empty overload stages and no selected-root claim.
Transport the sealed input and selection outputs through existing TypedAst/ResolvedTypedAst slots and fresh replay; reject unsealed exposed views transactionally.
Retain the shipped Task285 AttributeRedefinition/FunctorRedefinition rules and their [family ledger](../../../../tests/coverage/source_element_rule_migration.tsv); exercise both in construction and replay.
Forbidden: target guessing among several eligible roots, coherence proof acceptance, priority-edge or alias-semantics fabrication, accepted refinement, facts, VC or semantic credit.
Deferred: imported/template/nominal/attributed targets, chains, scheme/theorem roles (external S1), Tasks274–276 acceptance/activation, Task280 and Task49 reconciliation.
No spec, fixture-intent, diagnostic-class, trace-status, coverage or protected-surface changes.
## Test matrix (cap 24 categories)
Positive inputs and expected outcomes:
1. Ordinary attribute root plus same-block synonym: authentic relation payload bound to that root.
2. Ordinary binary predicate root plus same-block antonym: authentic relation payload bound to that root.
3. Ordinary functor root plus explicit `coherence with` label: exact written-label intent retained.
4. Omitted `with` target: unique root, Pending request, no accepted coherence.
5. Bound target: diagnostic payload retains actual target identity and source anchor.
6. Bound functor target plus real application: authenticated refinement candidate reaches selection intake; Pending candidate never activates.
7. Accepted-coherence input channel carrying Pending: retained as Pending, no accepted refinement emitted.
8. Sealed exposed view: survives both AST containers and is inspected by a real MT10-AS Task10 Case.
9. AttributeRedefinition and FunctorRedefinition: genuine element rules used by construction and replay.
Refusal inputs and expected outcomes (no partial installation):
10. Several eligible roots without disambiguation: reject, never guess.
11. Missing target/root or source anchor: reject.
12. Coherence-proof, priority-edge, alias-semantics or accepted-refinement fabrication: reject, never emit.
13. Refinement candidate without authenticated bound target: reject.
14. Duplicate/conflicting target identity or out-of-source-order target set: reject.
15. Exposed view without sealed handoff: reject.
Reuse predecessor cases; additions use real frontend/resolver/source rows, source oracles and corruption controls.
## Exit and verification
Freeze before implementation; independent specification/test-intent, sufficiency, precision, pure volume and source/document consistency reviews; nine gates and score at least 90.
Run fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites including links/fragments, source-redefinition/overload/AST owner groups, affected runner tests and advanced-semantics CLI.
Heavy evidence/cache/resolver obligations are unchanged; full cargo test follows cadence. Preserve predecessor oracles.
Update paired owner/harness boundaries and indices; audit changes only if owned coverage/deferral changes.
