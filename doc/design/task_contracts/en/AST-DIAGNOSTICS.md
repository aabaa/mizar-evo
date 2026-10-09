# Task AST-DIAGNOSTICS: Reflection diagnostic adoption
Canonical language: English; [Japanese pointer](../ja/AST-DIAGNOSTICS.md).
Status: planned. Tier: full gates. Primary owner: [mizar-diagnostics](../../mizar-diagnostics/en/00.crate_plan.md).
Consumers: parser/checker, VC/proof/MVM, mizar-test and tooling.
Dependencies: source-aware owning-stage failure records and frozen caller scope.
Readiness: reservation/descriptor preparation ready; emission and executable
diagnostic coverage blocked on actual owning-stage producers.
Authority: [§22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics)
and [§22.7](../../../spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference).
Gap: source_drift/test_gap for reserved code adoption and source-derived reports.
Scope: existing E0104/E0111/E0112/E0206/E0332/E0333 reservations and existing
syntax/runtime/verification codes where the formal table assigns them.
Explain declaration/use and source names/positions, missing conditions/mode/C,
candidate kind/exact residual position and first inapplicable rule.
Distinguish unproved from false, fail from incomplete, and actual stage/limit;
import failure after normal return must preserve internal evidence.
Separate legal guarded valid_at statements from unconditionally valid updates.
B1-B8 remain record classes, never diagnostic codes.
Forbidden: new IDs/syntax, inferred codes from message text, registry/emission
credit from specification transfer, silent candidate fragment publication,
or changing protected existing .miz/expectation intent.
Owner-local invariants/tests: [registry.md](../../mizar-diagnostics/en/registry.md#reflection-diagnostic-adoption).
Owner details: [allocation](../../mizar-diagnostics/en/registry.md#allocation-rules),
[retirement](../../mizar-diagnostics/en/registry.md#retirement-rules),
[records](../../mizar-diagnostics/en/failure_record.md).
Tests: use the linked registry adoption matrix.
Existing [registry tests](../../../../crates/mizar-diagnostics/src/registry.rs) retain their infrastructure scope.
Audit: reserved descriptors do not activate producer emission or runner credit.

Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, and source/document consistency under the canonical protocol.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected owner tests and local-link/parity checks.
Exit: frozen scope satisfied by authentic producer inputs and reviewed tests;
no completion or coverage claim for deferred consumers.
