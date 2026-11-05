# Task CHECKER271-NATIVE-CONVERSIONS: native reconsider and qua requests
Canonical language: English; [Japanese pointer](../ja/CHECKER271-NATIVE-CONVERSIONS.md).
Status: complete (producer/classification only). Tier: full; plan first for the existing-handoff method and production scope.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); lower: [resolver plan](../../mizar-resolve/en/00.crate_plan.md); [lexer plan](../../mizar-lexer/en/00.crate_plan.md).
Consumer: [test plan](../../mizar-test/en/00.crate_plan.md); parser47/native269/270 ready; proof272 remains separate.
Authority: EN04 §4.4.2, EN08 §8.2, EN15 §15.5.1, EN16 §16.1/16.8, EN19 §19.3.2.
Gap: source_drift/test_gap: native conversion roles and source-authenticated proof-free classification are missing.
API owner: [native binding boundary](../../mizar-checker/en/source_proof_local_declaration.md#native-proof-local-binding-boundary).
Consumer owner: [harness](../../mizar-test/en/harness.md); audit impact: additive request/classification metadata only.
## Intake and roles
Reuse existing original/native/symbol/sealed-scope handoff, BindingEnv and fresh Typed/Resolved slot.
Ship distinct QuaExpression, ReconsiderItem and ReconsiderStatement original-row rules together.
Rules return original base/RHS/name/shared-target/tail sites and order through existing row helpers.
Preserve exact omitted/by/proof intent; syntax and citations supply no accepted evidence.
Named reconsider introduces one binding/RHS link; bare items retain an existing sealed binding ID.
Qua introduces no binding. Preserve all three forms' original target and source associations.
Opt-in proof_bindings adds named-Reconsider own-RHS exclusion and bare-token Term references.
Preserve resolver legacy outputs; check bare-token/reference totality/non-self RHS; lexer bare views declare no name.
Retain whole originals/symbols, authenticate namespace declaration shells/contexts, exclude only their local names.
Unowned conversion subtrees fail closed; no theorem-only surrogate, blanket skip or registration-as-evidence.
## Checking API and source authority
Add parameterless check_conversions() returning Result<(Vec<CoercionInput>, CoercionCheckingOutput,
Vec<TypeFactQueryOutput>), SourceProofLocalDeclarationError>; no caller facts/flags, stored DTO/table/field/slot.
Reconstruct native authority before deriving transient requests and running the real checker/query engine.
Use original explicit plain Set/Object declarations and earlier qualified builtin views; refuse unsupported dependencies.
Keep original types alongside view history keyed by binding ID, original item order and lexical ancestry.
Only independently qualified three-pair builtin relations with real Candidate/known types create usable views.
Qualified named aliases retain actual RHS source-type knowledge; pending/rejected aliases gain none.
Pending/rejected/RequiresObligation written targets never seed source types; child views do not escape.
Prefer latest eligible view supporting the target, falling back to retained original type; no retroactive retyping.
Qua uses its latest valid apparent/expression view regardless of target; no original fallback or binding update.
Qualify BuiltinRadix only for plain Set identity, Object identity or Set-to-Object; no arbitrary marker trust.
Authenticate local attribute target SymbolIds/roles from real source symbols; declaration is not attribute truth.
Allocate from/to roles conversion.<request-node-id>.from/to on original type nodes via actual resolved links.
Shared targets get distinct request-local sites; never equate numeric Surface/Resolved/Typed IDs.
Preserve TypeNormalizer duplicate-site policy. Scope-check every attribute symbol rather than trusting its kind.
Join real checker outputs by unique site; core sorting is not request order.
Request/query ranges match the original item/qua site; reconsider query uses positive coercion.widening.builtin_radix;
target is joined through that unique immutable request's to_type site and authentic SymbolIds.
Query only actual matching current-request builtin facts; other supported evidence remains unavailable/Missing.
SourceQua uses the existing actual StaticUpcast path, without inventing a builtin query fact.
Return actual Candidate/RequiresObligation/Blocked/Rejected results; no accepted proof or view-dispatch claim.
Omitted missing evidence is Rejected/type.narrowing_requires_proof with no implicit obligation.
## Executable intent and exit
Keep the existing active FS builtin-widening fixture and semantic check; additionally consume the native pipeline.
Add a distinct non-Task49 AS classification source/sidecar; pass means observed requests/query/results only.
Retain the existing omitted AS seed's inactivity, stable adapter key and semantic trace credit.
Whole-source negative regression proves its unavailable attribute evidence/core rejection without activation.
Correct only the omitted seed's mandatory theorem header to named closed Set reflexivity (EN16); body/oracle fixed.
Manual original-role/query/status oracles cover named/bare/mixed lists, shared targets, by/proof/omission and qua.
Cover self/forward/shadow/restoration, old-scope bypass, source-point/view lifetime and pending-target laundering.
Cover malformed/foreign/coherent original/type/target/tail/query associations, empty-semantic installation and replay.
Preserve old269/270, semantic reconsider/qua, parser tails, all old expectations and trace statuses.
Producer exit requires all roles plus real FS/AS classified request/query consumption; stronger MT10-AS stays open.
Task272 retains general proof-skeleton pending/blocked state and the existing seed's final semantic consumer.
No inferred arbitrary term types, imported/cluster evidence, proof search, discharge, Core/VC or Task49 activation.
Run fmt, Clippy, both mandatory lint_policy suites, owner/query/coercion/resolver and protected regressions.
New active admission requires metadata/global-tally checks and the whole active-semantic metamorphic guard.
Run actual FS/AS/declaration/type corpora and affected Driver/heavy checks; full at selection change/producer exit.
Require independent PRE authority/test-intent, sufficiency, implementation, volume and consistency reviews.
