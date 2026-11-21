# Task P23-ATP-CANDIDATE-PRODUCER: explicit untrusted local claims
Canonical language: English; [Japanese pointer](../ja/P23-ATP-CANDIDATE-PRODUCER.md).
Status: frozen prerequisite for P23; tier: full (new public production seam, not a repeated family slice).
Owner: [ATP plan](../../mizar-atp/en/00.crate_plan.md); lower-stage owner: [VC plan](../../mizar-vc/en/00.crate_plan.md).
Consumer: P23 ATP-G-029 rejecting-corpus execution; this unit grants no coverage credit.
Dependency/readiness: owner decision 2026-11-22 authorizes the producer/extraction prerequisite; P23 resumes only after the landed-unit audit.
Authority: specification [§21.7.1](../../../spec/en/21.source_code_annotation_and_atp.md#2171-minimum-kernel-architecture-and-de-bruijn-criterion), [§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#2175-kernel-evidence-format); existing `task_29_*` translator tests and Kernel polarity/local-context rejection cases.
Gap: source_gap; authenticated VC construction and ATP translation reject invalid bindings, and the finder emits only Kernel-accepted evidence.

## Scope and design
Add a separately named local candidate encoder sharing the existing VC wire writer.
It receives an original immutable handoff plus explicit goal polarity and a complete
list of `(formula_id, KernelFormulaSource)` claims. Validate row coverage/uniqueness
and supported local source classes, but leave claim membership/polarity to Kernel.
Formula contents, manifests, fingerprints, provenance, target, profile and substitutions
remain from the original handoff; never rebuild canonical context from candidate claims.
Keep `encode_local_evidence`, handoff construction, translator and finder semantics unchanged.
An ATP factory binds this fresh wire to a genuinely encoded original problem and
matching original handoff; reuse existing candidate transport and deterministic hashes.
The factory takes explicit claims, never candidate bytes or backend proof/log material.
No test-only mutation or fixture conversion constitutes this producer.
Paired owners: [candidate boundary](../../mizar-atp/en/backend.md#candidate-evidence-boundary), [VC handoff](../../mizar-vc/en/kernel_evidence_handoff.md).
Affected: those paired owners, owner-plan backlinks, VC wire writer, ATP backend factory, focused tests in existing test modules.
Forbidden: language/specification changes, Kernel/schema/acceptance changes, guarded-backend or finder policy changes, imported facts, proof status, publication/cache, corpus expectations/trace edits and P23 completion.
Audit/deferral: ATP-G-029 corpus execution remains P23-owned; no spec-coverage-audit status changes.

## Test matrix and exit
Six cases maximum; additional sufficiency cases must cite authority.
1. Canonical claims: fresh transport parses; bytes equal existing local writer; independent Kernel outcome unchanged.
2. Consistency polarity: fresh candidate parses and Kernel rejects against original proof-obligation context.
3. Unverifiable local source: fresh candidate parses and Kernel rejects against original context; original handoff remains unchanged.
4. Duplicate/missing/unknown claim ids and unsupported sources: encoder refuses structural input before emission.
5. Different original problem/handoff binding: ATP factory refuses, including target/producer mismatch.
6. Identical inputs through both concrete formats: deterministic metadata/candidate identity; claim changes affect identity.
Existing upstream refusal tests remain mandatory. Establish one real producer fixture before expanding the matrix.
Reviews: independent specification/test intent, sufficiency, precision, removable volume/scope, source/docs consistency and scoped evaluation.
Verification: fmt; all-target/all-feature Clippy `-D warnings`; checker/test `lint_policy` (links/fragments); VC/ATP/Kernel owner tests; short proof-verification executable check. Full suite follows dispatch cadence.
Exit: frozen contract committed first; matrix and required checks pass; original trust boundaries preserved; implementation committed locally for orchestrator audit.
