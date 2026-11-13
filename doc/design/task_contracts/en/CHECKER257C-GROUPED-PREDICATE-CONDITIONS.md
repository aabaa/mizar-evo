# Task CHECKER257C-GROUPED-PREDICATE-CONDITIONS: transparent predicate-condition parentheses
Canonical language: English; [Japanese pointer](../ja/CHECKER257C-GROUPED-PREDICATE-CONDITIONS.md).
Status: defined.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [runner plan](../../mizar-test/en/00.crate_plan.md); [family ledger](../../../../tests/coverage/predicate_chain_composition.tsv).
Pattern tier: follows full-tier [whole-chain C2](CHECKER257C-SCOPED-PREDICATE-CHAIN-CONDITIONS.md), [grouped builtin conditions](CHECKER257C-NATIVE-CONDITION-COMPOSITION.md) and implemented [native co-installation](CHECKER257C-NATIVE-C2-C3-COINSTALLATION.md). Same owners, consumers, wrapper tables and refusal classes; no public seam or protected-surface change.
Authority: [EN14.1 parentheses](../../../spec/en/14.formulas.md#141-overview), [EN14.2.1 chains/polarity](../../../spec/en/14.formulas.md#1421-predicate-applications), [EN13.4.2 comprehensions](../../../spec/en/13.term_expression.md#1342-set-comprehension-fraenkel-operator), [EN4.6 scopes](../../../spec/en/04.variables_and_constants.md#46-scoping-and-shadowing). Classification: bounded source_drift/test_gap; paired predicate authentication and native intake currently require bare conditions.

## Intended boundary
Grouping means one or more transparent parentheses around the entire existing singleton or whole-chain predicate condition, never regrouping chain segments or combining formulas. Preserve the predecessor's earlier local binary definition, bare variable operands/mapper, one bare Set generator, independent unwrapped Comprehension, outer Membership and authenticated theorem/proof scopes.
Use genuine source `definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z likes A) };`. Let G(P,d) enclose condition P in d parentheses; d=0 is the existing bare control.
Authenticate every actual Normal parenthesis/FormulaExpression bridge, punctuation, ordered child, range and lexical context through the existing [Atomic owner](../../mizar-checker/en/source_atomic_formula.md#validation-and-ownership). Associate the condition's effective outer occurrence with its unique core atom; wrappers add no formula identity, C2 association or C3 row. Preserve complete scoped Primary/Set/Atomic/empty Application products, candidates and unresolved requests; C2 maps once to the whole chain, C3 retains its exact C2 receipt and complete adjacency/negative-segment census through Typed/final replay.
Production scope: existing checker source_atomic_formula.rs condition authentication, necessary source_set_term.rs/source_formula_composition.rs plumbing, and existing runner/type_elaboration atomic/set/composition intake. Tests stay in existing checker source_term.rs/source_atomic_formula.rs and runner composition/Step5C7 tests. No new API/type/field/carrier/walker or fallback.
Exclude compound conditions, formula-level negation, grouped predicate operands/heads, wrapped/nested Comprehensions and broader profiles. No specification, MIZ, expectation, trace, diagnostics, proof selection or coverage-credit changes; inference, applicability, sethood/result typing, facts/truth, acceptance and Core/VC remain deferred.

## Intended test matrix
Cap 24 categories; the following 18 are required. Successful cases require genuine Frontend/Resolver acquisition, literal source/child-derived complete lower/wrapper/C2/C3 censuses and deterministic Typed/final replay, without answering requests or adding semantic tables.
1. G(`z likes A`,1): singleton succeeds, one C2 association, no C3.
2. Same at depths 2 and 4: all wrappers retained, unchanged association counts.
3. G(`z does not likes A`,1) and G(`z do not likes A`,2): singleton polarity retained, no C3.
4. G(`z likes A likes z`,1): one C2 association and one C3 adjacency.
5. Three-segment chains with first/middle/last negative segments, both verbs, depths 1/2: two adjacencies and the exact negative census.
6. Header plus proof (`let A,Q be set; thus A in { w where w is set : (w likes A likes w) };`): shadowing/unused declarations retain actual contexts.
7. Disjoint grouped chain and bare negative-singleton siblings: complete associations, C3 only for chain.
8. Alpha-renaming, whitespace/comments and depth variants of cases 1/4: deterministic source-derived replay.
9. Bare singleton/chain and grouped builtin controls: existing results unchanged.
10. Wrapped compound `z likes A & A likes z` or formula-level `not`: refuse this intake profile.
11. Wrapped owning Comprehension or grouped predicate operand: retain predecessor refusal.
12. Missing/extra/duplicate/reordered wrapper rows: refuse.
13. Disconnected/substituted/extra-child/nontransparent/recovered bridges: refuse.
14. Wrong punctuation/range/spelling/context or foreign/stale arena: refuse.
15. Cross-condition, per-segment or missing C2 association: refuse.
16. Conditioned C3 with absent/different C2 receipt: refuse without partial installation.
17. Duplicate C2/C3 installation: refuse; pristine retry and input remain unchanged.
18. Valid grouped transactions in both supported lower installation orders, then C2 before C3: build, rollback after faults and immutable final clones preserve ownership.

## Exit
Freeze this pair before production; establish genuine source/lower success and intended paired-Atomic/native runtime failures first. Reuse existing lower corruption matrices, not implementation-derived expectations. Review authority/test intent, combined implementation/sufficiency/consistency and independent removable-only volume. Run fmt, Clippy, both lint_policy suites (links/fragments), scoped oracle, Atomic/C2/C3/Typed-final, native composition and Step5C7 groups, Driver and representative type/formula/metadata CLI; full cargo test only on a cadence trigger. Update owner boundaries on implementation; Task 257C remains open.
