# Task BUILD-TOOLCHAIN-PIN: manifest constraints and locked toolchain
Canonical language: English; [Japanese pointer](../ja/BUILD-TOOLCHAIN-PIN.md).
Status: planned. Tier: full. Owner: [build plan](../../mizar-build/en/00.crate_plan.md); consumers: [driver plan](../../mizar-driver/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md), the toolchain manager.
Dependencies: current planner and dependency-constraint handling; authenticated manifests for all resolved dependencies and released-toolchain version metadata.
Readiness: optional workspace constraints and recorded lock-version metadata are available; complete validation, selection and mismatch integration await authenticated dependency constraints and release versions.
Authority: [§23.1-23.2](../../../spec/en/23.package_management_and_build_system.md#232-versioning-and-dependency-resolution) and [§22.6.6](../../../spec/en/22.error_handling_and_diagnostics.md#2266-api-compatibility-semver-check).
Gap: `source_drift` and `test_gap`; parsed metadata is not yet integrated with complete constraint satisfaction, authenticated locked selection or running-version mismatch policy. Running ToolchainInfo is provenance, not a resolved pin.
Owner details: [planner boundary and tests](../../mizar-build/en/planner.md#toolchain-constraint-boundary); diagnostics adoption remains producer-owned.

## Scope

Implement the linked planner boundary using existing constraint syntax and
real manifest/release inputs. Build owns selection/lock/mismatch policy;
manager consumers neither select another version nor mint missing metadata.
Freeze representation and diagnostic handoffs in the owner document before
implementation; this contract introduces no API names or lock schema fields.

## Forbidden

Changing specification, existing fixtures or expectations; choosing a version
without every required constraint; silently upgrading a pin; installing or
bundling backends; inventing public diagnostics. Backend redistribution review
and installation remain distribution prerequisites.

## Tests and exit

The linked planner section owns manifest/lock and CLI test cases; absent
authenticated dependency inputs must fail closed.
Run existing [planner tests](../../../../crates/mizar-build/src/planner/tests.rs)
and [CLI tests](../../../../crates/mizar-driver/tests/cli.rs); add focused fixtures without
claiming language-semantic coverage. Require independent specification,
test-sufficiency, implementation, volume/scope and consistency reviews;
`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
`cargo test`. Exit: selection satisfies every constraint, is locked, and its
mismatch policy is projected without installing anything.
Audit impact: §23 toolchain resolution and §22 W0306 retain deferred integration
ownership until this task's producers and consumers are wired.
