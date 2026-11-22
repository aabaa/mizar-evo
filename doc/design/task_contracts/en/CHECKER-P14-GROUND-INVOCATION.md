# Task CHECKER-P14-GROUND-INVOCATION: Typed ground invocation
Canonical language: English; [Japanese pointer](../ja/CHECKER-P14-GROUND-INVOCATION.md).
Status: implemented for the bounded ground profile. Tier: full (new authenticated invocation seam).
Owner: [Checker plan](../../mizar-checker/en/00.crate_plan.md); [TODO](../../mizar-checker/en/todo.md).
Consumer: [R4 loader join](../../mizar-vc/en/mvm_executor.md#proposed-entry-output-and-owner-boundary-no-new-public-api).
Dependency: [P7 checked selection](CHECKER-P7-SELECTED-CALLS.md).
Authority: [builtin object](../../../spec/en/03.type_system.md#34-built-in-types), [viability](../../../spec/en/19.overload_resolution.md#1942-viable-candidates), [algorithm calls](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability).
Owner decision P14-NUMERAL (2026-11-22): numerals initially have type object and are admitted to object arguments; no additional language-intent change is authorized.
Classification: source_drift/test_gap; genuine same-module idalgo(0) lookup exists, but no complete typed ground invocation producer exists.

## Scope and ownership
Extend the existing checked selection path for direct same-module algorithm-body calls with nonempty ordered numeral actuals and explicit plain builtin object formal/result types; retain the existing unary variable-actual path.
Use the existing normalization, collection, expansion, viability, specificity and selection engines; a singleton Resolver candidate is insufficient.
Return an opaque borrowed SourceGroundInvocationCheck from check_source_ground_invocations(source, symbols, typed), retaining immutable original source/typed association, exact application/target/caller identities, complete ordered numeral sites, and existing BindingEnv/inference tables.
Numeral payloads remain in the original source arena; no new value/term representation, evaluator, serialized schema or invented numeric subtype.
Authenticate the complete neutral source/environment/typed correspondence before any output. Invalid associations fail atomically; unsupported sites yield no invocation row.
Retain source ranges and a caller-body context for each actual; this ground-only context grants no variable lookup, captured UID or C evidence. Caller owns physical source currentness and later loader join.
Refuse ambiguity, arity/type mismatch, imported/qualified/alias/template/attributed signatures, local shadowing, recovered input, non-ground actuals and header calls. Preserve existing unsupported-input refusals.
No SourceAlgorithmCheck expansion, Core lowering, runtime artifact/loader/execution, requires discharge, termination/computability/promotion/Proof acceptance, diagnostic activation, AST-TYPE/AST-LOWER or owner-gated credit.
API/invariants: [type checker owner](../../mizar-checker/en/type_checker.md#typed-ground-invocations); lower-stage authority remains [Resolver](../../mizar-resolve/en/symbols.md#planned-algorithm-call-dependencies).
Affected: type_checker.rs, necessary private source_overload selection adaptation, checker tests, test-first numeral-call .miz fixture and EN/JA type-checker docs; spec EN/JA records only P14-NUMERAL.
Coverage audit remains partial: this producer does not implement R4 execution or full algorithm typing. Traceability records parse-only fixture credit separately from Rust producer checks.

## Test matrix and exit
Cap: six families, at most six new Rust test functions; existing genuine frontend SourceFixture establishes shared setup before the remaining cases.
1. Genuine idalgo(0): exact target/declaration/application/caller/ranges, Known object actual, original numeral payload; setup passes first.
2. Multi-actual and repeated calls: complete formal order, distinct occurrences/payloads, same-value repetitions and renamed targets; deterministic immutable originals.
3. Arity and type mismatch, ambiguous/overloaded roots: no invocation, no result-type tie breaking.
4. Variables/nested applications, qualified/imported/alias/template/local-shadowing inputs and header calls: no ground invocation; existing selected variable edges unchanged.
5. Foreign source/symbol/typed, changed projection/range/children/kind or recovered data: atomic refusal, no partially exposed rows.
6. Genuine numeral payload and context checks across distinct call sites/sources: exact ordered actual-to-inference/context correspondence, no facts, coercions, execution or acceptance output.
Fixture: tests/miz/pass/parser/pass_parser_typed_ground_invocation_001.miz; parse-only corpus expectation grants no type/execution credit.
Reviews: independent specification/test-intent/trust before freeze; sufficiency, implementation precision, removable-only volume, source/document/EN-JA consistency and final full-tier quality.
Verification: fmt; all-target/all-feature Clippy -D warnings; both lint_policy suites and links/fragments; checker selection/ground-invocation owner tests; Resolver selected-edge tests; genuine parse/producer end-to-end check; full cargo test for dispatched push and inherited cadence.
Exit: frozen matrix and required gates pass, authenticated producer is reviewable, and the implementation is committed and pushed without higher-stage credit.
