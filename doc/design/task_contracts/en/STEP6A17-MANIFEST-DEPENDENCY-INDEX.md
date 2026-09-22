# Task STEP6A17-MANIFEST-DEPENDENCY-INDEX: manifest dependency metadata
Canonical language: English; [Japanese pointer](../ja/STEP6A17-MANIFEST-DEPENDENCY-INDEX.md).
Status: implemented. Tier: full. Owner: [build plan](../../mizar-build/en/00.crate_plan.md).
Authority: [architecture 3](../../architecture/en/03.module_and_symbol_resolution.md), [build metadata contract](../../mizar-build/en/module_index.md#manifest-metadata-projection), [manifest sidecars](../../mizar-artifact/en/manifest.md#module-entries).
Gap: external dependency/design gap; the real driver consumes DependencyArtifactIndex but no manifest metadata producer supplies it.
Scope: one method on existing DependencyArtifactIndex, existing module_index tests, paired owner/plan indexes, contracts and global todo. No new type, field, file or dependency.
Contract: validate canonical manifest JSON with the artifact reader, bind known package-plan identity, preserve supplied namespace metadata and project present current-schema summary references using their store-level artifact hash. No synthesized missing sidecar.
Tests: actual manifest storage/read to production index construction; present/absent sidecars, exact artifact-versus-interface digest, identity/schema/published-path/group/order failures, supplied namespace validation through existing index.
Forbidden: filesystem reference validation claims, current-build lockfile binding or cache/proof credit, new diagnostics, source export/summary stand-ins, provider/service publication, spec/corpus/expectation/trace/audit changes, artifact task17 and Step7/MVM.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews; cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: canonical manifest metadata produces faithful package-bound dependency indexes or fails closed; root resolution, referenced-file validation and complete source producers remain subsequent work.
