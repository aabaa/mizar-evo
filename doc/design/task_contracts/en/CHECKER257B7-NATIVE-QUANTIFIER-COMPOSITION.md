# Task CHECKER257B7-NATIVE-QUANTIFIER-COMPOSITION: native explicit quantifiers
Canonical language: English; [Japanese pointer](../ja/CHECKER257B7-NATIVE-QUANTIFIER-COMPOSITION.md).
Status: bounded implementation complete; broader Task257/258 remains open. Tier: full.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [test plan](../../mizar-test/en/00.crate_plan.md).
Predecessors: CHECKER257B6-NATIVE-CONNECTIVE-COMPOSITION and CHECKER258-NATIVE-LET-STATEMENTS.
Authority: EN04 §§4.5/4.6; EN14 §§14.3/14.4.1–4/14.5.2/14.7.3/14.7.5; EN15 §§15.2.1/15.4.1; EN16 §§16.1/16.2/16.4.
Gap: source_drift/test_gap; genuine lower/context and original composition exclude explicit existential and restricted universal structure.
## Frozen source family
Retain ordinary genuine theorem/lemma owners, explicit builtin-Set declarations, actual proof Lets and direct equality conclusions.
Admit universal or existential header roots and finite nesting of both explicit quantifier kinds with the existing fixed connectives and transparent grouping.
Header roots remain direct QuantifiedFormula children; transparent grouping applies below them.
Universal tails have an optional st condition and an actual holds body or nested quantifier body; existential st introduces its body.
Conditions and bodies may be atomic equality or supported composite formulas; every variable reference selects its actual lexical declaration.
Include grouped, multiple and unused explicit declarations, shadowing and omitted holds only where the existing parser grammar permits them.
Keep numeric/constant/other term or predicate leaves, repetition, implicit/reserved binders, imports, modified status/visibility and function Binder outside this family.
Native258 statement intake remains its single ordinary universal-root theorem with proof Lets and one direct conclusion; existential-root statement intake still refuses.
Restrictions add no assumption/fact, existential bodies add no witness, and no checked composite, accepted theorem/proof, Core/VC output or requirement activation follows.
## Existing representation and boundaries
Reuse the existing seven composite and two composition tables, atomic/composite wrappers, source context/type tables and complete native metadata.
Append only SourceFormulaEdgeRole::UniversalRestriction and SourceFormulaAtomicEdgeRole::ExistentialBody with their stable formatter keys.
Keep old role/key meanings, raw profiles and goldens; add no public type, field, method, table, slot, DTO, helper or parallel walker.
TypeChecker owns original grammar and interpretation; SourceContext joins actual universal and existential expression boundaries to their unchanged genuine contexts.
Typed context compatibility must retain faithful quantifier-kind/opening-token correspondence; preserve the existing kind-only universal-to-existential forgery refusal unchanged.
Use existing retained inputs and final original replay; no new kind carrier, syntax authority or accepted semantic receipt is introduced.
Preserve every primitive input, authentic declaration/type application and typed-node association through scoped inference, canonical installation and fresh original replay.
Quantifier formulas/type sites use their incoming context; condition/body use the actual quantifier expression context, with nested quantifiers retaining fresh contexts.
Connectives and transparent groups add no binding scope; each wrapper retains its actual owner/context and recursively parenthesized operator descriptor.
Operand ordinals follow written child order: restricted universal condition then body; unrestricted universal and existential have one body operand.
Composite and atomic children share those operand ordinals through their actual roles; st is a universal restriction marker or existential body marker according to its parent kind.
Derive bound uses in actual header-reference order independently of edge allocation, preserving per-binder ordinals, shadow winners and separate proof-local identities.
Retain complete rooted unique graph, token/child ownership, full lower/context/type/composition fingerprints and unrelated occupancy/external-input refusals.
## Frozen test-first stages
Add one private native quantifier MIZ shared by at least three actual owner tests and the complete external public oracle; no corpus discovery, sidecar or activation.
Freeze complete source-derived lower/context/type expectations and full Typed/Resolved success/refusal oracles before semantic implementation.
Prepare all genuine Frontend/Resolver inputs and unchanged B6 positive prerequisites before RED-L; compiler/setup failures are not RED.
Implement lower/scoped/SourceContext/Typed compatibility alone, require every new lower prerequisite GREEN, then witness RED-U at unchanged original composition.
Only after that actual RED-U implement composition and execute complete graph/wrapper/request/use-clock, native258 coexistence and fresh final replay oracles.
Cover atom/composite restriction and existential bodies, nested/sibling quantifiers, compound conditions, grouped declarations, alpha/layout and source-clock versus edge order.
From installed nonempty positives exercise causal condition/body, kind/context/type/wrapper/lexical-use swaps with dependent fingerprints refreshed where needed.
Correct only the two old B6 lower exclusions for nested explicit existential and restricted universal, after independent specification/test-intent/trust PRE.
Their source-derived primitive success checks preserve every unrelated B6 positive, rejection and recovered-input control; no output-derived rebaseline is authorized.
Preserve other old native/raw/proof/Core/VC/Driver tests and existing MIZ/expectation/spec/trace metadata; separately review any additional discovered maintenance before editing.
## Review, volume and verification
Present the public-role/large-change plan first; ceilings: production 900 including embedded private tests and documentation 120 added lines.
Independent specification/test-intent/trust PRE, staged actual RED/GREEN, POST correctness/sufficiency, pure volume and source/document consistency precede nine gates and score.
Run fmt, workspace Clippy, both lint_policy, affected checker/context/harness/metadata/Core owners, verifier/resolver standard and short formula/type/CLI checks.
Run changed graph/replay/context obligations immediately; unchanged heavy resolver/evidence groups defer only to the named shared-push integration full.
Full cargo test must pass before shared push; full failure stops feature work until repair and full success.
Exit: complete genuine explicit-quantifier lower and original composition/final replay pass with native258 and all protected boundaries preserved.
