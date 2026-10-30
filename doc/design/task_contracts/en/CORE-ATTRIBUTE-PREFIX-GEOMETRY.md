# Task CORE-ATTRIBUTE-PREFIX-GEOMETRY: transport uniform Attribute coordinates
Canonical language: English; [Japanese pointer](../ja/CORE-ATTRIBUTE-PREFIX-GEOMETRY.md).
Status: complete. Tier: full. Owner: [Core plan](../../mizar-core/en/00.crate_plan.md); consumer: existing private [Attribute probe](../../mizar-test/en/harness.md#core-task-33i261-private-item-context-probe).
Dependencies: completed Core 33I261/33LB and native Attribute checker transport; prerequisite to checker Task 281.
Readiness: existing native layouts produce complete public checker handoffs and range-derived Core seeds/33LB; actual uniform geometry must be witnessed before production.
Authority: lexical §§2.2/2.9.1 and Attributes §§6.1/6.2/6.8.1/6.9 in `doc/spec/en/`; unchanged `tests/miz/pass/types/pass_type_elaboration_attribute_definition_payload_001.miz`.
Gap: `source_drift` in Core's canonical-only Attribute coordinate profile.

## Scope and boundaries
The [Core owner](../../mizar-core/en/elaborator.md#core-33-task261-attribute-item-context) owns current geometry; preserve the completed historical contract.
In `validate_attribute_checker_owner`, derive d by checked source-item end minus 115, require start<=d and explicit current-source equality, and translate existing inner profile ranges by d.
Underflow/incoherence remains static InvalidCheckerOwner; preserve environment→33LB→owner→Core shape→association precedence. Newly admitted layouts may reach existing later errors.
Keep every non-coordinate profile, whole-symbol join, fingerprint, provenance, normality, pending state and checker ownership check; add no fallback/helper/public seam.
Change only `crates/mizar-core/src/elaborator.rs`, `crates/mizar-test/src/runner/tests/type_elaboration/source_attribute_definition.rs`, necessary paired owner/index metadata and measured ratchet decreases.
No arbitrary nonuniform layout, native selection, lower/upper API, specification, corpus, expectation, trace/coverage, CoreIr/production installation, semantics or acceptance change; broader retirement remains deferred.

## Tests and exit
Before production, confirm actual uniform native geometry and collect public Core RED for distinct nonzero displacements, covering start=d and start<d, using fresh public checker/seed/33LB inputs; require GREEN afterward.
Verify literal independent ranges, whole-symbol association, source map/boundary/worklist provenance, pending states, retained-input equality and fresh repeat; no semantic outputs.
Retain canonical/default-deny oracles; add shifted wrong-source/provenance and matching same-source stale Context/owner refusals, plus genuine native-supported nonuniform refusal. Do not mutate sealed positives to obtain success.
Require independent SPEC/intent-trust, TEST, CODE, VOLUME/repeat and consistency reviews, scoped gates and score.
Run fmt, all-target/all-feature Clippy-Dwarnings, affected Core/33LB/replay/default-deny, Task261/native Attribute layouts/refusals, type corpus/whole metamorphic guard and docs/link/ledger checks; apply inherited full cadence and expand immediately for changed/unbounded obligations.
Exit: bounded public Core coordinate transport verified; no Core/crate/quality milestone completion claim.
