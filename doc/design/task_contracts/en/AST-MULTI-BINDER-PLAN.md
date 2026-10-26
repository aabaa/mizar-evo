# Task AST-MULTI-BINDER-PLAN: Multiple-argument Binder implementation decomposition
Canonical language: English; [Japanese pointer](../ja/AST-MULTI-BINDER-PLAN.md).
Status: partial; MBP-01 root syntax implemented, all other implementation slices remain unimplemented. Tier: full gates.
Owners: [Checker plan](../../mizar-checker/en/00.crate_plan.md) (coordination/typing), [Resolver plan](../../mizar-resolve/en/00.crate_plan.md) (MBP-02 source identities).
Consumers: Parser, Core, VC, Proof, Diagnostics, Artifact and Test through their existing AST tasks.
Authority: [§13.10](../../../spec/en/13.term_expression.md#1310-function-binder--), [§20.5.3](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax), [Appendix A](../../../spec/en/appendix_a.grammar_summary.md#a13-term-expressions).
Test intent stays in the [MB1–MB10 corpus plan](../../mizar-test/en/miz_corpus.md#multiple-argument-binder-acceptance-tests); this contract owns sequencing, not duplicate acceptance text.
Scope: decompose the agreed unary/multiple-head implementation into bounded owner slices, dependencies and exits; no source, specification, fixture, expectation or trace edits in this planning task.
Classification: source_drift/test_gap for Binder syntax, source identities, certified formation and reflected execution; existing generic binding/normalization infrastructure is not their producer.

## Readiness and boundaries

MBP-01 has a bounded set/grouped/body-root syntax increment; remaining host/pattern preparation stays open. MBP-03 can inventory pinned library inputs and MBP-08 can prepare descriptors independently; their acceptance/emission still waits for authentic producers.
Before implementing a row, freeze its source forms, real producer/consumer route and stage-specific tests under the named existing task; these are slices, not replacement owners or new public APIs.
MBP-02 is the explicit Resolver prerequisite missing from AST-TYPE's generic declaration dependency; algorithm synonym/citation task RESOLVE33 does not supply Binder scope records.
Retain AST-PARSE's protected legacy exhaustive fixture blocker; additive Binder work neither requires nor authorizes its retirement.

| Slice | Owner / existing task | Required predecessor or external input | Deliverable / exit and test mapping |
|---|---|---|---|
| MBP-01 Syntax | Parser with Lexer/Frontend/Syntax / AST-PARSE | Existing lexical/surface pipeline; frozen Binder syntax | Source-ordered unary/multi-head syntax, nesting/qua and pattern candidates; preserve arity without tuple-variable desugaring. MB1 through the real parser route, with malformed-head controls; no typing credit. |
| MBP-02 Source resolution | Resolver and Checker binding consumer / AST-TYPE prerequisite | MBP-01, authentic outer declarations and construction-position context | Resolver supplies ordered head/body candidates and outer declaration/type-source correspondence. Checker selects the mode before head identity commitment, then establishes simultaneous body or branch-local capture scope with preserved free identities and no mode fallback. MB4 scope, MB6 names and MB8 scope controls; type/C acceptance stays in MBP-04–06. |
| MBP-03 Library certification | Checker/Proof with Resolver/Artifact inputs / AST-LIBRARY-CERTIFICATION | Genuine pinned definitions, proof material and existing proof-acceptance route | Validated pair/product packing and decomposition laws plus required carrier/function construction correspondences. MB9 identity/version/premise controls; missing exact library/proof inputs remain blockers. |
| MBP-04 Fresh formation requests | Checker and Core request transport / AST-TYPE, request-only AST-LOWER | MBP-02, MBP-03 and authentic type/signature inputs | Provisional product-domain/codomain/context records and all-input typing/application/range/construction requests, including empty domains. MB2–MB4 request observations only; no checked Binder or runtime output. |
| MBP-05 Formation proof and adoption | VC/Proof/Checker / AST-FORMATION-PROOFS | MBP-04 requests and genuine validation inputs | Generate goals, validate proofs and re-enter Checker with matching evidence. MB2–MB5/MB9 accepted versus Open/rejected/stale cases and ordinary function meaning; no new kernel lambda or inference from request presence. |
| MBP-06 Capture, reconstruction and R2 | Checker / AST-TYPE | MBP-05 checked originals, MBP-01/02 pattern provenance | Exact-arity capture, complete ordered C, saved domain/default codomain and hygienic reconstruction. MB6–MB8; reconstructed results use the MBP-04/05 proof/adoption route before checked output. |
| MBP-07 Checked ordinary/reflected handoff | Core ordinary-term lowering; final AST-LOWER for reflection | MBP-05 checked records; MBP-06 additionally for captured/reconstructed syntax | Lower ordinary Binder to a certified first-order function term with accepted construction/uniqueness premises; require a real ordinary source→Core/VC MB5/MB9 exit without MVM. Reflected lowering preserves ordered Abs, child duplicates, FV/size rules and outer/extended contexts, with MB5/MB8 structural and corruption controls; operation execution remains MBP-09. |
| MBP-08 Diagnostics | Diagnostics with failing-stage owners / AST-DIAGNOSTICS | Descriptor design may start; emission needs each relevant source failure producer | Existing reserved diagnostics explain head/use positions, domain/type/C and failed obligations. Negative controls for MB1–MB9 run with their owner stage; no new diagnostic numbers or synthetic emission. |
| MBP-09 Reflected execution and publication | Existing AST-CONTRACT, AST-ARTIFACT, ALG-MVM-EVALUATION, AST-COMPUTE owners | MBP-07, applicable MBP-08 emission and every existing runtime/proof/build-policy dependency | Execute supported Binder capture/reconstruction/structure operations and publish only certified complete candidates. Runtime/import/replay instances of MB5–MB9 retain complete input/C; no differentiation API. |

## Execution order and cross-cutting work

The initial path is MBP-01 → MBP-02 → MBP-04 → MBP-05, with MBP-03 feeding MBP-04. MBP-05 → MBP-07 supplies ordinary lowering; captured/reconstructed syntax follows MBP-05 → MBP-06 → MBP-07. MBP-08 attaches to each source stage.
MBP-05's initial fresh-Binder proof route does not depend on MBP-06; later reconstruction reuses that route with its own premises. Request-only Core transport precedes adoption; final checked handoff follows it.
MBP-09 retains separate existing tasks in order: AST-CONTRACT verification → AST-ARTIFACT runtime export → ALG-MVM-EVALUATION → AST-COMPUTE → AST-ARTIFACT computation provenance. Export cannot depend on later computation publication; ordinary mathematical formation does not require MVM execution.
AST-CORPUS accompanies every row: add the applicable new .miz/sidecar tests before implementation, then activate only the genuine stage route. Parser success does not activate typing/proof/runtime rows; existing fixtures, expectations and trace statuses are not changed by this plan.
MB10 remains a separate future differentiation specification/library task: define its API and verified coordinate-update contract before implementation. Its execution would need MBP-09; it is not a Binder-completion prerequisite.
Deferred throughout: dependent heads, direct arbitrary subset-domain multi-head formation, implicit currying, public UID/environment/substitution APIs, unverified certification and changes to protected legacy tests.

## Ownership and completion

Owner boundaries: [parser](../../mizar-parser/en/grammar.md#reflected-syntax-input-boundary), [resolution](../../mizar-resolve/en/names.md#boundary), [typing](../../mizar-checker/en/typed_ast.md#reflected-syntax-formation), [ordinary lowering](../../mizar-core/en/elaborator.md#step-3-term-and-formula-lowering), [reflected Core](../../mizar-core/en/core_ir.md#reflected-syntax-handoff), [formation proofs](../../mizar-vc/en/generator.md#reflected-formation-requests), [corpus routes](../../mizar-test/en/harness.md#reflection-corpus-boundary).
Audit impact: clarify dependency/owner sequencing only; retain all absent executable coverage and external producer blockers.
Reviews: independent plan/specification consistency and acceptance-mapping review, then volume/scope review; no implementation acceptance from this document.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; local links, dependency order and unchanged authority/test surfaces.
Planning exit: each MB1–MB9 has an owning slice and prerequisite; MB10 is explicitly deferred; ready preparation and blocked implementation are distinguished. Implementation completion belongs to the corresponding owner task and real stage tests.
