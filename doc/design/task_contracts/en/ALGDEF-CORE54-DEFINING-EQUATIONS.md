# Task ALGDEF-CORE54-DEFINING-EQUATIONS: definitional-algorithm defining equations
Canonical language: English; [Japanese pointer](../ja/ALGDEF-CORE54-DEFINING-EQUATIONS.md).
Status: planned. Tier: full. Owner: [Core plan](../../mizar-core/en/00.crate_plan.md); consumers: [VC plan](../../mizar-vc/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: Core 42, 43, and 46 in the [algorithm task graph](../../mizar-core/en/source_family_decomposition.md#accepted-algorithm-coreir-task-graph). The fragment has no loops, so this task checks exhaustiveness on the statement tree itself; Core 48 keeps the general missing-`return` diagnostic for other bodies.
Authority: specification §20.7.3 definitional fragment, §20.13.6 equation construction, §20.2.6 every path returns, §20.8.2 recursive groups, §10.12.2 guarded definiens shape; [architecture 06](../../architecture/en/06.elaboration_and_core_ir.md#algorithms-are-lowered-to-core-items-not-yet-cfg).
Gap: `test_gap` and `source_drift`; the specification amendment adds equation construction that no Core family owned.

## Scope

Classify each `terminating` algorithm body against the definitional fragment.
For a fragment body, check that every path returns, derive local guards, path
conditions, and `const` substitution, and attach one guarded equation per
`return` to the Core algorithm item with source provenance. A fragment body
that can end without `return` receives no equation. A mutually recursive group receives
equations only when every member is in the fragment. Template algorithms keep
schema parameters abstract. Boolean conditions and formula-valued boolean
results follow §20.13.6.

A body outside the fragment (`var` or assignment, loops, `break`/`continue`,
`match`, non-ghost Pick, calls to unpromoted algorithms) records a stable
classification and no equation.

## Forbidden

Acceptance or availability of the equations as facts (VC 57), VC generation,
termination proof, MVM execution, extraction, public diagnostic codes,
synthetic Core input, and specification, corpus, or expectation changes made
to match behavior.

## Tests

Test-first real sources: `factorial`, `fibonacci`, and a mutually recursive
group whose equations match the §20.13.2 and §20.13.6 examples in the
`MT10-CIR-ALG` CoreIr baseline; one near-miss per excluded construct showing no
equation; a group with one non-fragment member; a fragment body that can end
without `return`. Corruption tests cover missing, duplicate, and
reordered returns and stale branch ownership.

## Coverage, reviews, and exit

Coverage audit: the chapter 20 row records this ownership of §20.7.3/§20.13.6.
Require independent specification, test-sufficiency, implementation,
volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: every fragment body yields exactly its §20.13.6 equations in deterministic
order, every other body yields none, and no downstream fact becomes available.
