# Task CHECKER258-NATIVE-LET-STATEMENTS: native quantified statements and Let guards
Canonical language: English; [Japanese pointer](../ja/CHECKER258-NATIVE-LET-STATEMENTS.md).
Status: bounded implementation complete; broader Task258 remains open. Tier: full.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: CHECKER257B5-NATIVE-UNIVERSAL-COMPOSITION and the genuine native theorem-equality context/type lower.
Authority: EN04 §§4.2/4.5/4.6/4.7.1, EN14 §§14.4.1/14.5.2/14.7.5, EN15 §§15.2.1/15.4.1/15.10/15.11.1, EN16 §§16.1/16.2/16.4.
Gap: source_drift/test_gap; reserved-only original statement construction excludes genuine explicit quantifier/proof-Let contexts.
## Frozen input and meaning
Admit one default-public ordinary unmodified theorem with an explicit builtin-Set universal header and a genuine proof.
The proof has one or more explicit Set Let declarations followed by exactly one direct equality Conclusion, without citations or other statements.
Grouped/separate declarations, unused names and nested explicit universals use only shapes authenticated by the unchanged B5/lower.
Keep the single CheckedStatementOwner seal; lemmas, multiple owners, visibility/status modifiers, imports and reference/witness companions are excluded.
The header targets its actual Composite root in the incoming module context, with no external header binder/type-guard input fact.
Quantifier type guards remain inside composite binder/type-site data; header and proof-local identities remain distinct even with identical spelling.
The conclusion targets its actual Atomic equality and proof context; visible actual Let bindings supply LetTypeGuard input intent.
Retain each Let declaration/type application and its actual per-conclusion reference uses; an unused declared binding may have no uses.
Use one existing unverified candidate per proposition; source ordinals count actual theorem/Let/Conclusion events, while Let adds no proposition row or SourceStatementKind.
No accepted premise/fact/theorem/proof, substitution, logical closure, normalized-type certificate, Core/ControlFlow/VC or coverage credit is added.
## Representation and authentication
Add only SourceStatementInputFactKind::LetTypeGuard; preserve ReservedTypeGuard meaning, spelling and every old raw refusal/golden.
Reuse [statement tables](../../mizar-checker/en/source_statement.md), complete BindingEnv and primary/atomic/composite/composition fingerprints.
Extend the existing with_source_statement_from_original entrance on the exact genuine B5-installed receiver, preserving primitive tables/metadata.
Keep the reserved constructor's canonical-base semantics and all detached/raw guards; no added public method/type/field/table/slot/DTO or retained parallel AST.
TypeChecker interprets original syntax; the existing statement owner constructs and validates its immutable five-table transaction.
Authenticate owner label/symbol/contribution/source, statement order/sites/targets/contexts, declared type applications and all guard/candidate associations.
TypeChecker may transiently rebuild the existing original SurfaceAst and rerun actual Resolver collectors for fresh owner authentication; never copy or parse opaque identity algorithms.
Fresh native replay validates the same original/lower graph and complete statement alongside the expected composite/composition pair.
Only a fully validated native Let statement may coexist with B5 transport; every unrelated occupancy and external semantic-input refusal remains.
Retain genuine TypeTable, local contexts and expression metadata; never clear or replace them to satisfy legacy syntax-only validation.
Reject wrong/foreign/repeated/overlapping canonical installation and partial/forged/extra statement, guard, candidate and lower associations.
Function Binder, general proof/statement semantics and multi-owner/reference transport remain separate owners.
## Test-first and fixture ownership
Add crates/mizar-checker/tests/testdata/native_universal_statement.miz from the cited specification before semantic logic.
It is a private source oracle shared by at least three actual owner regressions plus the external public primary; corpus discovery does not select it.
Its Rust assertions are the expectation oracle; add no sidecar, trace activation, selected corpus input or replacement of an existing MIZ.
Freeze a complete manually source-derived public primary and installed Typed/Resolved success tail before implementation.
All admitted alpha/layout/grouped/nested/unused controls must pass real frontend/resolver/SourceType/B5 and baseline Resolved prerequisites before producer RED.
Compiler/setup/lower failure is not RED; only the unsupported existing statement entrance after those prerequisites establishes RED.
Require valid nonempty installed baselines before coherent header/Let/type/context/use/target/owner swaps and missing/extra/corrupt replay controls.
Preserve the unchanged two-owner lemma source as a B5 success and native-statement refusal, old reserved/witness/reference routes and protected Core/VC/Driver outputs.
Coverage audit records the source-only statement/input-intent owner extension without promoting requirement status or acceptance.
## Review and verification
Present the public-enum/large-change plan first; ceilings: production 1400 including embedded private tests, documentation 140 added lines.
Independent PRE/spec/test-intent/trust, complete-primary genuine RED, POST implementation/sufficiency, pure volume and consistency precede final gates/score.
Run fmt, workspace Clippy, both lint_policy, checker/harness/metadata/Core owners, verifier/resolver standard and short formula/type/CLI checks.
Run all affected association/replay regressions immediately; unaffected resolver/evidence heavy obligations defer only to the next periodic/integration full.
The reviewed original-AST builder import updates only the canonical TypeChecker import and exact lint literal; this policy change requires full before commit and shared push.
Exit: canonical installed/fresh final native statement intent, protected consumers unchanged, required reviews/checks clear; broader Task258 stays open.
