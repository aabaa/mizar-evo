# Task ALGDEF-RESOLVE-CALL-EDGES: source algorithm calls and definition dependencies
Canonical language: English; [Japanese pointer](../ja/ALGDEF-RESOLVE-CALL-EDGES.md).
Status: planned. Tier: full gates. Owner: Resolver; consumers: Checker280, Core46 and VC call/group owners.
Authority: [§10.3.3](../../../spec/en/10.functors.md), [§20.8](../../../spec/en/20.algorithm_and_verification.md#208-recursive-definitions), [§22.5.2](../../../spec/en/22.error_handling_and_diagnostics.md#2252-circular-definitions), [architecture03](../../architecture/en/03.module_and_symbol_resolution.md).
Classification: source_drift/test_gap; signature dependency indexes and Resolve33 label/synonym rules do not produce body-call/definiens edges.
Readiness: authentic source/signature inventory and bounded body-call resolution can start; each consumer requires its complete real declaration/body producers.
Dependencies: parser algorithm/definiens source records, resolved declarations/signatures and lexical scopes; preserve existing ordinary visibility behavior outside algorithm recursion.
Scope: resolve source algorithm call targets and definition-block self/mutual algorithm scope; produce authenticated definiens/body-call edges and complete recursive-group membership with source provenance for Checker280/Core46.
Boundary: identity/scope/edge versus checked-semantic responsibilities follow the linked owners. Overloaded uses retain authenticated candidate identities for Checker; Core consumes the checked selected call, never a provisional candidate as winner.
Invariants: exact declaration/use-site/body/edge/group identity, source order and signatures remain bound; no dependency edges inferred from synonyms or algorithm contracts as if they were definiens/body calls.
Failure: missing/stale/recovered/ambiguous identities remain unresolved and block checked consumers; never resolve from spelling or manufacture a winner. Existing diagnostics/recovery remain owned, with no new numbers.
Forbidden: ordinary forward-reference relaxation, typed-overload decisions in Resolver, termination/proof acceptance, executable loading or guessing call/group provenance.
Owner details/tests: [scope boundary](../../mizar-resolve/en/names.md#scope-model), [source edges](../../mizar-resolve/en/symbols.md#planned-algorithm-call-dependencies).
Tests: legal self/mutual body calls, ordinary illegal forward uses, direct/mutual/mixed definiens cycles, contracts/synonyms excluded as graph edges, scopes/shadowing/overload candidates; stale/forged target, missing group member and cross-body/range provenance rejection.
Acceptance: complete authentic edge/target/group inputs reach Checker/Core without granting type, cycle, termination or promotion acceptance.
Audit: chapter10/20/22 producer ownership only; no implemented cycle coverage.
Reviews: independent specification, test sufficiency, implementation, volume/scope and source/EN-JA consistency; full protocol gates.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test.
Owner indexes: [mizar-resolve index](../../mizar-resolve/en/00.crate_plan.md), [mizar-checker index](../../mizar-checker/en/00.crate_plan.md), [mizar-core index](../../mizar-core/en/00.crate_plan.md).
Existing foundations: [name tests](../../../../crates/mizar-resolve/src/names/tests.rs), [signature dependency tests](../../../../crates/mizar-resolve/src/symbols/tests.rs) and [resolved source tests](../../../../crates/mizar-resolve/src/resolved_ast/tests.rs); none establish source algorithm call/group acceptance.
