# Task STEP6A5-SOURCE-SERVICE: real disk source service

Canonical language: English; [Japanese pointer](../ja/STEP6A5-SOURCE-SERVICE.md).
Status: implemented. Tier: full. Owner: [driver plan](../../mizar-driver/en/00.crate_plan.md).
Dependencies: STEP6A1–A4. Consumer: later real frontend service and Step 6 orchestration.

## Authority and scope

[Spec 22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md) owns source-load failures;
[spec 23.4](../../../spec/en/23.package_management_and_build_system.md) and
[architecture 22](../../architecture/en/22.incremental_verification_contract.md) own input identity and freshness.
Resolve the SourceLoad portion of DRIVER-G-010, leaving Frontend's producer/diagnostic gaps explicit.
Register a real disk-only SourceLoad service that calls the existing frontend/session loader and publishes its actual SourceUnit through the supplied IR publisher.
Use borrowed submission snapshot, build plan, module index, and caller allocator; wire their transport in the existing scheduler dispatch path.
A5 execution acceptance is direct registry execution: default submit remains blocked before scheduling until real later services exist; add no source-only graph/profile.
Require the existing owner-supplied dispatch identity to equal the source semantic key; missing identities remain a dispatch gap.
Keep all later producers, artifact task 17, cache-hit scheduling, full-graph equivalence, Step 7, and MVM outside this task.
Do not fabricate source requests, payloads, parents, spans, phase services, or cache hits; do not change source acceptance or existing corpus expectations.

## Owner contracts and artifacts

[Source adapter](../../mizar-driver/en/frontend_adapter.md) owns binding, status, diagnostic conversion, and tests.
[Registry](../../mizar-driver/en/registry.md) owns execution resources and service availability.
[Source payload](../../mizar-frontend/en/source.md) owns semantic/storage and source-map identities.
[Publisher](../../mizar-ir/en/publisher.md) and [storage](../../mizar-ir/en/storage.md) retain publication, sealing, and placement authority.
Update paired affected owner docs, driver source/tests/dependencies, and registry consumers for borrowed execution context.
Update the coverage audit only for real disk source producer adoption; later frontend/language coverage remains partial.
No SourceLoad cache reuse: normalized text hashes alone do not prove current raw source maps.

## Tests and exit

Use real files, planner/index/snapshot values, loader, shared diagnostics, and resident/blob IR storage.
Cover exact current metadata/SourceId/maps, semantic stability with distinct raw maps, typed blob decoding, wrong metadata/hash, and post-capture mutation.
Cover invalid UTF-8, unreadable source, outside-root, generic allocation failure, request-location diagnostics, missing/wrong context, cancellation, stale publisher, and disallowed work units.
Reject missing, duplicate, foreign, non-module, non-disk, dependency-summary, and mismatched plan/index inputs before source loading.
Keep normal full submission honestly blocked by unavailable later services; direct service execution earns only SourceLoad credit.
Independent specification, test-sufficiency, implementation, volume/scope, and consistency reviews are required.
Run narrow driver tests, `cargo fmt --check`, all-target/all-feature clippy with `-D warnings`, and `cargo test`.
Exit: registered real SourceLoad publishes only validated current complete source output, emits allocated source diagnostics on real load errors, and preserves missing-owner blocking.
