# Task DRIVER-GUARDED-VAMPIRE-BACKEND: scheduled genuine Vampire candidates
Canonical language: English; [Japanese pointer](../ja/DRIVER-GUARDED-VAMPIRE-BACKEND.md).
Status: frozen. Tier: full. Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index); consumer: existing Kernel evidence checking.
Dependencies: [P2](STEP7-P2-OWNER-SYNC.md), [scoped finder](ATPK-THEOREM-FINDER.md), existing scheduled theorem ATP route.
Authority: spec [21.7](../../../spec/en/21.source_code_annotation_and_atp.md#217-atp-integration), [guarded extraction](../../mizar-atp/en/evidence.md#guarded-route-and-input), [process model](../../mizar-atp/en/backend.md#process-model), [scheduled ATP](../../mizar-driver/en/frontend_adapter.md#scheduled-source-theorem-atp-solve).
Readiness: DR-1's existing one-child guarded runner covers a Driver-scheduled external process; scheduling changes neither process authority nor backend trust. The scoped finder prerequisite supplies genuine candidates for the zero-axiom, one-local-guard quantified conjecture.
Gap: source_drift/test_gap — BackendRun lacks a registered service and the existing local candidate-check service regenerates evidence instead of consuming backend output.

## Scope and boundary

Add an explicit opt-in Driver BackendRun service using only the pinned guarded
`run_vampire_backend`, genuine source Core/VC/Discharge and original handoff.
Authenticate retained parents and exact encoding/configuration before execution;
retain actual run metadata and returned candidate, with original source/module/
snapshot/target bindings. Never replace missing evidence with a local candidate.
Add a corresponding bounded KernelCheck service consuming actual returned bytes
against the original handoff through normal Kernel checking and the existing
Proof policy evaluator. This is Driver composition, not a new Proof acceptance
API; existing local candidate, witness and Artifact routes stay unchanged.
Use existing immutable publication, parent validation and currentness guards,
NoKey and explicit singleton backend/evidence tasks. Failures, missing candidate,
cancellation and stale/mismatched inputs cannot publish accepted Kernel results.
Use the existing guarded process limits/probe/capture behavior; no shell,
fallback, synthetic backend or candidate, portfolio or dynamic task growth.
No language/specification intent, public diagnostic, proof policy, lower-owner
implementation, source activation, cache/reuse, witness/Artifact acceptance,
trace/audit/coverage-credit or owner-gated scope changes. General backend
integration and overall Step 7 completion remain deferred.

## Artifacts and tests

Source: Driver frontend_adapter candidate services, registry, driver preflight
and scheduler parent bridge. Documentation: this paired contract and paired
Driver frontend_adapter/registry/live-prefix boundaries and plan links only.
Tests: existing Driver source_load/driver tests and the existing
`scheduled_theorem_resolver_edits_match_independent_clean_modules` gate.
Establish the genuine shared scheduled fixture and pass one setup case before
expanding its matrix. Matrix cap: six families, variants share their oracle:
1. Genuine Disk/OpenBuffer schedule returns actual Vampire candidate; original-handoff normal Kernel accepts its actual payload.
2. Wrong/missing/duplicate/stale parents and source/module/snapshot/VC/target bindings fail closed; altered returned payload under valid bindings is rejected by normal Kernel checking, proving no local-evidence substitution.
3. Unsupported configuration, unavailable executable/version and process failures produce no accepted result.
4. Missing candidates and exhausted finder limits remain unaccepted; exact sufficient limits succeed.
5. Explicit registration/config descriptors, cancellation/currentness and NoKey preserve existing profiles.
6. Extend the same edit/clean gate through new services with independent stores, all five edits, worker/order variants and whole deterministic payload/lineage checks; retain old profile assertions.

## Reviews and exit

Independent specification/test-intent, sufficiency, implementation, volume/scope
and consistency/quality reviews. Run fmt, all-target/all-feature Clippy with
-D warnings, both lint_policy suites, Driver owner tests, the genuine finder
regressions and scheduled real-process E2E. Run changed evidence obligations;
defer unchanged heavy groups under cadence. Full cargo test only on a named
cadence trigger; no push or tenth-task trigger applies to this lane unit.
Exit: genuine scheduled process/candidate/normal-Kernel dataflow, refusals and
existing profile regressions pass applicable gates; local commit, no push.
