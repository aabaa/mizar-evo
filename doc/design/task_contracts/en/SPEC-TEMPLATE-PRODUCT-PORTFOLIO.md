# Task SPEC-TEMPLATE-PRODUCT-PORTFOLIO: specification consistency

> Canonical language: English. Japanese pointer:
> [../ja/SPEC-TEMPLATE-PRODUCT-PORTFOLIO.md](../ja/SPEC-TEMPLATE-PRODUCT-PORTFOLIO.md).

- Status: complete; user-approved specification correction, full gates.
- Purpose: reconcile template parameter scope, finite-product assumptions,
  and deterministic portfolio selection across the English/Japanese specification.
- Primary derived owner: [mizar-proof](../../mizar-proof/en/00.crate_plan.md);
  consumers: mizar-atp and mizar-test.
- Authority: spec §§18.2.2–18.2.6, 18.8.4, 20.1.1, 21.4.2, 23.4.
- Gaps: theorem-only parameter declarations conflict with algorithm scope;
  `PermProduct` omits associativity and the identity; first-completion ATP
  selection conflicts with reproducible artifacts.
- Scope: paired specification corrections and affected examples; the
  incremental verification contract; the invalid predicate-parameter oracle
  and its trace metadata; corresponding Bialystok examples and generated decks.
- Dependencies and boundaries: [selection](../../mizar-proof/en/selection.md),
  [policy](../../mizar-proof/en/policy.md), and
  [portfolio](../../mizar-atp/en/portfolio.md) retain implementation ownership.
- Deferred: activating template schema semantic oracles and integrating live
  ATP runtime selection. No production API or diagnostic changes are included.
- Tests: `pass_type_elaboration_template_pred_param_001`, proof `early_stop_`
  unit tests, and ATP `determinism_suite`; inactive oracles earn no execution credit.
- Required reviews: specification parity, test sufficiency, implementation and
  source consistency, and independent volume/scope review.
- Verification: `cargo fmt --check`, strict all-target/all-feature Clippy,
  `cargo test`; regenerate both decks, complete LaTeX builds, inspect PDFs.
- Exit: approved rules agree across affected owners and examples; checks pass;
  exact quoted MML and unrelated code remain unchanged.
