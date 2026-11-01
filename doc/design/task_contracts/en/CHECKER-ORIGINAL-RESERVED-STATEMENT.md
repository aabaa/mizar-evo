# Task CHECKER-ORIGINAL-RESERVED-STATEMENT: original-source reserved statement transport
Canonical language: English; [Japanese pointer](../ja/CHECKER-ORIGINAL-RESERVED-STATEMENT.md).
Status: complete. Tier: full. Purpose: retire the production Task258A exact profile.
Owner: [Checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: CHECKER-SOURCE-RESERVED-EQUALITY and QUALITY-EXACT-PROFILE-RETIREMENT.
Readiness: original-source native typing exists; statement installation and final transport are `design_drift`.
Authority: specification chapters [4 §4.3](../../../spec/en/04.variables_and_constants.md#43-reserved-variables-reserve), [14 §14.5.2](../../../spec/en/14.formulas.md#1452-equality-and-inequality), [16 §§16.1–16.2,16.7.1](../../../spec/en/16.theorems_and_proofs.md); layout equivalence: [2 §2.2](../../../spec/en/02.lexical_structure.md#22-whitespace-and-formatting).
Existing source: `tests/miz/pass/types/pass_type_elaboration_reserved_variable_equality_001.miz`; existing Task258A and sibling isolation Rust tests.

## Scope and ownership
The [TypeChecker intake](../../mizar-checker/en/type_checker.md#original-source-reserved-statement-inputs) retains source-syntax interpretation and projects the existing semantic owner input.
The [statement owner](../../mizar-checker/en/source_statement.md#original-source-reserved-statement-construction) owns source authentication and the five existing tables.
The [TypedAst owner](../../mizar-checker/en/typed_ast.md#original-source-reserved-statement-installation) owns atomic installation and complete canonical-base equality.
The [final owner](../../mizar-checker/en/resolved_typed_ast.md#original-source-reserved-statement-transport) owns final syntax-only transport.
Use only the existing native base slice: one reserve binder for bare set, followed by one unmodified default-public theorem with direct equality and omitted justification.
Retire the production Task258A raw profile and runner fallback; keep historical exact-profile coverage private to Checker tests. Other profiles remain unchanged.
The [harness owner](../../mizar-test/en/harness.md#original-source-reserved-statement-tests) owns dispatch isolation and external tests.
Migrate external Task258A tests to the actual public original-source boundary; retain malformed, foreign, rollback and cross-family refusal purposes.
Specification-derived assertion maintenance is limited to legal label/layout equivalence and the native resolver representation. Historical raw-byte identity is not weakened into layout equivalence.
The existing active runner paths, results, detail keys and their reserve/reference-clock coverage remain unchanged; native statement construction is opt-in.

## Boundaries
No added public type, provenance field, duplicated arena, unchecked attachment adapter, language diagnostic, dependency or protected artifact change.
Detached native handoffs cannot be installed through legacy setters, including reconstructed rootless arenas. Cloning the complete installed AST preserves it.
No implicit universal closure, statement semantics, proof bundle, checked fact, obligation discharge, acceptance, active route or coverage promotion.
No claim of raw capture/trivia/currentness identity, other statement-profile retirement or whole Task281 completion.
No `doc/spec`, existing `.miz`, expectation, trace or historical-contract changes. Coverage audit ownership/status is unchanged.

## Review and exit
Independent specification, test-intent/trust, test-sufficiency, code, volume and consistency reviews; reviewed assertion proposal/evidence stays outside `doc/`.
Exercise genuine Disk/OpenBuffer captures, coherent rename/layout, whole-AST mismatch with preserved lower facts, detached replay, extra families, final auxiliary-input refusals and canonical replay.
Require `cargo fmt --check`, all-target/all-feature Clippy with `-D warnings`, affected native/statement/guard/corpus/doc/ledger checks and unfiltered `cargo test`.
Lower the literal-offset ratchet only where the retired production profile removes counted checks. Exit: actual opt-in construction/install/final transport and unchanged active routes, with required reviews/checks clear.
