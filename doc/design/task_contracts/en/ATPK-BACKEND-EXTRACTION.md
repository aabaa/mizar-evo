# Task ATPK-BACKEND-EXTRACTION: guarded Vampire extraction
Canonical language: English; [Japanese pointer](../ja/ATPK-BACKEND-EXTRACTION.md).
Status: design frozen under owner decision DR-1 (2026-11-14), first backend Vampire; implementation deferred to a separate dispatch after provisioning. Tier: full (new adapter owner, candidate-extraction seam, kernel-evidence trust boundary).
Owner: [atp plan](../../mizar-atp/en/00.crate_plan.md). Consumers: [kernel plan](../../mizar-kernel/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md).
Dependencies: [instance finder](ATPK-INSTANCE-FINDER.md), [equality instances](ATPK-KERNEL-EQUALITY-INSTANCES.md), PROOF-INSTANCE-SEARCH-LIMITS, existing translator/FOF encoder and VC/kernel handoff.
Authority: [specification §21.7.1–§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#217-atp-integration), [architecture 10](../../architecture/en/10.atp_backend_integration.md#evidence-extraction-by-instance-finding), [task-15 decision](../../mizar-atp/en/todo.md#resolved-and-open-decisions), ATP-G-004/014/015/016 in the owner plan, and [Step 7 item 1](../../todo.md#step-7--evidence-pipeline-integration--).
Gap/readiness: `external_dependency_gap` / `deferred` implementation; paired extraction specification and guarded-route documentary preconditions are met. No adapter, output parser or new candidate schema is implemented. Provisioning remains required. No specification-coverage credit changes.

## Frozen scope

F1–F6: [extraction owner](../../mizar-atp/en/evidence.md) fixes the existing generic runner, deterministic encoded FOF input only, stdout SZS/advisory/reproducibility mapping, finder-only explicit kernel-schema payload/ref production, independent kernel acceptance, task-29 non-inference and deterministic fail-closed behavior. Reuse [classification](../../mizar-atp/en/backend.md#result-classification), [candidate boundary](../../mizar-atp/en/backend.md#candidate-evidence-boundary), [failure semantics](../../mizar-atp/en/backend.md#failure-semantics) and [finder API/limits](../../mizar-atp/en/instance_finder.md#api-and-boundary); metadata alone is never evidence.
F7: orchestrator provisions official open-source Vampire 4.9 outside the sandbox at `.tools/provers/vampire-4.9/vampire` before implementation; `.tools/` stays ignored. Record binary SHA-256 only in run records. Probe identity/version before solving; mismatch refuses the route without solve or fallback.
F8/deferred: portfolio, CVC5/Z3/E, SMT-LIB extraction, TFF/typed output, proof policy, witness/cache/artifact integration and externally attested handling. Step 7 item 1 remains open.
Affected: paired evidence specification, owner task-15/ATP-G-015 status, roadmap row and required pairing registrations; later implementation owns ATP adapter/extraction and integration tests. Freeze changes no source, executable behavior, language specification or test cases.
Forbidden: backend proof/TSTP/substitution payload translation, trusted backend reports, premise widening, new process model, invented polarity/context/imported projection, lower-stage schema or policy changes.

## Future implementation matrix and exit

F9: twelve categories, cap 24; real encoded-problem fixtures and the provisioned guarded backend are mandatory. Existing finder and generic-runner tests remain dependencies; no tests are added in this freeze.
- P1: success + complete advisory report narrows finder premises; candidates are emitted and every candidate passes independent kernel checking.
- P2: success without a report searches all encoded premises.
- P3: equality-axiom advisory cases use the equality-instance dependency.
- P4: pinned version probe succeeds; metadata records identity/version and command fingerprint.
- R1: missing/malformed SZS status yields no extraction/candidates.
- R2: unknown label discards the whole report with a diagnostic; no partial narrowing, finder uses all encoded premises.
- R3: timeout/crash/nonzero exit follows existing classification, produces no candidates and cleans temporary files.
- R4: mismatched probe version prevents solving and backend fallback.
- R5: proof object/TSTP offered as payload is refused; only finder payloads qualify.
- R6: output implying changed polarity/context/imported projection supplies no inferred binding or candidate.
- R7: repeated identical problem/stdout/version extraction yields identical advisory sets, candidate ids and hashes under fixed handoff/limits.
- R8: advisory narrowing never widens premises; PROOF-INSTANCE-SEARCH-LIMITS still governs search.

Reviews: independent specification/test-intent now; sufficiency, implementation/precision, volume/scope and consistency for implementation; freeze also receives volume/scope and consistency review.
Freeze verification: both checker/test `lint_policy` suites, local links/fragments, pairing registrations, scope and contract caps; no fmt/Clippy/full test trigger.
Implementation verification: fmt, workspace all-target/all-feature Clippy with warnings denied, both lint suites, ATP owner/integration matrix, representative end-to-end check and full `cargo test` under its dispatch cadence.
Exit: green matrix including kernel acceptance of every emitted candidate; unchanged mock behavior; guarded provisioned route permits ATP-G-015/016 reopening/resolution; implementation full suite passes. This freeze only makes that implementation dispatchable.
