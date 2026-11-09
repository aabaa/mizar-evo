# Task CHECKER257-NATIVE-IMPLICIT-QUANTIFIERS: reserved Set quantifier types
Canonical language: English; [Japanese pointer](../ja/CHECKER257-NATIVE-IMPLICIT-QUANTIFIERS.md).
Status: bounded implementation complete; broader Tasks257/258 remain open. Tier: full.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [test plan](../../mizar-test/en/00.crate_plan.md).
Dependency: completed CHECKER257-NATIVE-CONNECTIVE-ROOTS; broader Tasks257/258 remain open.
Authority: EN04 §§4.3/4.5/4.6; EN14 §§14.1/14.3/14.4.1–4/14.5.2/14.8; EN15 §15.2.1; EN16 §§16.1/16.2/16.4.
Gap: source_drift/test_gap; native lower/context/type/replay reject omitted quantifier types and their real Reserve provenance despite existing raw B3 transport.
## Frozen source family
Admit leading top-level bare builtin-Set Reserve declarations with distinct identifier spellings, followed by ordinary theorem/lemma owners with the existing quantifier/fixed-connective/grouping headers.
Require at least one actual omitted-type quantifier declaration; each omitted type uses the unique preceding same-spelling Reserve default authenticated from original source and Resolver inputs.
Support grouped/distinct reserve names, multiple reserves, mixed explicit/implicit quantifier segments, both quantifier kinds, restrictions, nesting, shadowing and existing fixed connectives as one structural rule.
Every header term reference must select an actual quantifier binder; proof bodies retain explicit Set Lets and direct equality conclusions under the existing grammar.
Keep explicit-only modules with unused reserves and all reserve-only atomic/assumption/witness/nested-proof profiles on their existing routes.
Reject repeated reserve spellings without choosing replacement semantics; the existing re-reservation spec_gap remains deferred. Interleaved/nonleading reserves, free reserved theorem uses/implicit closure, implicit proof Lets, other type heads and function Binder remain outside scope.
## Existing ownership and provenance
Reuse [TypeChecker intake](../../mizar-checker/en/type_checker.md#original-source-universal-composition-intake), [SourceContext](../../mizar-checker/en/source_context.md), [SourceType](../../mizar-checker/en/source_type.md), [TypedAst](../../mizar-checker/en/typed_ast.md), [composite](../../mizar-checker/en/source_composite_formula.md#original-source-universal-composition) and [harness](../../mizar-test/en/harness.md) owners.
Use existing Reserve/ReserveDefault roles and reserved identities, preserving genuine projected status; reuse Source(range) binding type sites, declaration/type applications and BuiltinSet binder heads without a new public type, field, API, adapter, parallel representation or syntax owner.
Authenticate omitted segment to original Reserve identifier/segment/TypeExpression using actual spelling, declaration order, module/source and recovery; Resolver has no default_from field and no origin may be fabricated from type equality alone.
Keep reserved defaults as source/type metadata, actual quantifier bindings fresh and active, and proof Let identities separate; no free reserved term winner is admitted by this family.
Retain every real shell/item/declaration/context link; module Reserve declarations share the existing module context and add no lexical scope.
Keep shadowed_binding as its actual lexical predecessor, which may be an outer quantifier; authenticate default type origin independently from the unique leading Reserve.
SourceType applications share the real Reserve expression and head by original site; neither a synthetic quantifier TypeExpression nor a cloned physical expression is allowed.
SourceType application/expression rows add no context field. Typed expression/head nodes retain their actual module context; allow the shared-type validation exception only through authenticated implicit Reserve provenance.
Composite quantified formula contexts remain incoming and binder/body contexts remain actual quantifier scopes. Explicit composite type sites keep existing context rules; implicit composite type sites use the actual Reserve module context, as raw B3 does.
Preserve complete original graph, native metadata, type universe, scopes, clocks, fingerprints, canonical installation and fresh joint replay; narrow Reserve guard changes to this family rather than globally relaxing old routes.
Native258, higher Proof/scoped skeleton, theorem/fact acceptance, Core/VC output and requirement/trace activation remain deferred; test those real refusals after successful new lower/composition.
## Test-first stages and maintenance
Add one private native implicit-quantifier MIZ only if shared by at least three actual owner tests; use genuine Frontend/Resolver input and existing public lower/upper oracle machinery without corpus discovery or activation.
Before semantic code freeze complete source-derived lower/context/type and full Typed/Resolved oracles, including every default/type application, original node, wrapper, edge, request and reference clock.
Primary intent is the existing B7 reserved x/implicit for x/explicit proof Let x case: reservation, fresh quantifier and proof binding stay distinct; reserve and quantifier applications share the written Reserve type.
Correct only that B7 implicit-source exclusion after independent specification/test-intent/trust PRE; preserve every other old assertion/exclusion and every existing MIZ/expectation/trace/hash artifact.
In particular retain reserve-only refusal controls in source_theorem_equality.rs and source_nested.rs, object/foreign/recovered controls, raw goldens and error identity.
First witness compiled lower RED on genuine new inputs; implement lower/scoped/context/Typed type compatibility only, require all new primitive/Typed/Resolved prerequisites GREEN, then witness RED at unchanged original composite intake.
Only after genuine upper RED implement implicit binder/default replay and run complete upper/final oracles, real Native258/Proof refusal controls and source/argument purity.
Cover shared/distinct defaults, mixed/multiple/unused binders, both quantifier kinds, nested shadowing, fixed roots, alpha/layout, default physical expression sharing and independent reference/per-binder use clocks.
From installed nonempty positives exercise coherent default origin/type-site, module/quantifier/proof context, shadow winner, declaration order, shared-expression and bound-use corruption with all dependent fingerprints refreshed and pristine replay controls.
## Reviews, budgets and verification
Independent specification/test-intent/trust PRE, POST sufficiency/implementation/trust, pure volume and source/document consistency precede the full gates and score.
The presented plan permits at most 500 added production lines; inline first and introduce a private helper only for at least three real calls. Documentation stays within 200 added lines and 2:1; each contract stays within 60 lines.
Run fmt, workspace all-target/all-feature Clippy, both lint_policy suites, affected checker/context/type/Typed/scoped/Core owners, verifier/resolver standard and short formula/type/metadata CLI checks.
Run changed default/identity/context/replay obligations immediately; unchanged heavy resolver/evidence matrices defer only to shared-push full cargo test. Full failure stops feature work until repair and full success.
Coverage audit ownership/credit is unchanged; update live owner documents only for accepted-input/type-context boundaries, with paired translations and no per-run evidence.
Exit: complete genuine reserved-type lower and original/final composition pass with distinct lexical/default provenance and all protected upper/legacy boundaries preserved.
