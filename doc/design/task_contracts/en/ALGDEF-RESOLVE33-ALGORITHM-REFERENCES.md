# Task ALGDEF-RESOLVE33-ALGORITHM-REFERENCES: algorithm synonyms and citations
Canonical language: English; [Japanese pointer](../ja/ALGDEF-RESOLVE33-ALGORITHM-REFERENCES.md).
Status: frozen for definition-label identity projection; later synonym/citation adoption remains deferred. Tier: full. Owner: [resolve plan](../../mizar-resolve/en/00.crate_plan.md); consumers: [checker plan](../../mizar-checker/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: authenticated Algorithm declaration shells and preserved optional label tokens ([symbols](../../mizar-resolve/en/symbols.md#terminating-algorithm-label-boundary)); [label owner](../../mizar-resolve/en/labels.md#purpose).
Readiness: the bounded source identity prerequisite is ready; synonym targets, citation adoption and verified promotion premises remain later work.
Authority: [§16.5.1](../../../spec/en/16.theorems_and_proofs.md#1651-simple-citation), [architecture 03](../../architecture/en/03.module_and_symbol_resolution.md#label-resolution-is-scoped-separately-from-item-resolution). Later synonym/replay work follows §11.1.2, §11.5.1 and §20.1.1.
Gap: `source_drift` and `test_gap`: syntax retains optional terminating-algorithm labels, but no authenticated owner/token identity producer exists.

## Scope

Add `source_algorithm_definition_labels(&SurfaceResolvedArena, &SymbolEnv)` in symbols.rs, returning existing borrowed Algorithm entries paired with original label `SurfaceNodeId`s, in declaration source order. First run unchanged `validated_source_symbol_ast` and preserve its exact error. Recollect actual AlgorithmDefinition shells under DefinitionBlock owners and join each to exactly one Algorithm entry by the complete existing shell origin; never use spelling as ownership. Retain the owner's signature, contribution, visibility and original declaration provenance unchanged.

Project only the exact direct `terminating algorithm Label: name` prefix: original Identifier label token, colon, and actual algorithm name. Check prefix range containment, source association and order. The token id retains original spelling, SourceId and range through the source arena. Unlabelled terminating/ordinary algorithms and non-algorithm labels produce no row; empty authenticated inputs return an empty vector. Do not require body-call inventory or impose its unrelated body profile. Refuse malformed/ambiguous ownership or labelled prefixes atomically with an internal String; no partial result. Caller retains source association and owns physical raw-source currentness.

The tuple is an internal identity projection: it introduces no new label path, visibility clock, public type, semantic node, label index/ref-table entry or consumer wiring. Document this boundary in paired label/symbol owners. Existing collector profiles and resolver outputs remain unchanged.

## Forbidden

Citation adoption, automatic algorithm-synonym targets or synonym replay, Checker replay, typed target selection, promotion acceptance/facts, ordinary name changes, parser changes and specification/corpus/expectation changes are excluded. Algorithm names and `by alg(args)` gain no citation status. Definition-label identity establishes no promotion or proof evidence.

## Tests

Append at most six regression families to existing labels/tests.rs using genuine Frontend/collectors and the existing [label syntax fixture](../../../../tests/miz/pass/parser/pass_parser_terminating_labels_001.miz): (1) public/private/default/template labelled owners retain exact borrowed identities and token/range provenance; (2) empty, non-algorithm and unlabelled controls yield no rows; (3) repeated labels/names, multiple blocks and ordering never substitute spelling for ownership; (4) genuine source relocation/trivia/body changes preserve current source associations, replay and input purity; (5) foreign/augmented environments and real parser recovery preserve upstream refusal atomically; (6) unchanged collectors/citation outcomes and labelled bodies outside call-inventory shape remain independent. Additional cases require a named authority clause.

## Coverage, reviews, and exit

Coverage audit ownership/deferrals remain unchanged: this unwired prerequisite grants no chapter 11/16 acceptance credit. Require independent specification/test-intent, sufficiency, precision, volume/scope and source/EN-JA consistency reviews. Run fmt, Clippy, both lint_policy suites, scoped links/fragments, `bash tests/check.sh resolver standard` and full `cargo test` on the tenth-task cadence. Resolver edit/clean and evidence matrices defer to their periodic/integration deadline because their consumers are untouched.
Exit: authenticated owner/token projection and its tests pass; later synonym/citation and promotion obligations remain deferred.
