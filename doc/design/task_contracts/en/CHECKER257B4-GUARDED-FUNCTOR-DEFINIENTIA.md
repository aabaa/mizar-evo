# Task CHECKER257B4-GUARDED-FUNCTOR-DEFINIENTIA: guarded functor source transport
Canonical language: English; [Japanese pointer](../ja/CHECKER257B4-GUARDED-FUNCTOR-DEFINIENTIA.md).
Status: bounded implementation complete; broader Task257, function Binder formation and Task49 remain open. Tier: full.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); API: [functor definition](../../mizar-checker/en/source_functor_definition.md).
Authority: EN02 §§2.2/2.10, EN10 §§10.1/10.3.1/10.6/10.12.3 and EN16 §§16.6.1/16.6.4/16.9; existing source functor/lower transports.
## Frozen input and meaning
Admit one ordinary binary functor with two separately declared builtin-Set parameters, both actual written loci, and declared Set result.
Its equals definiens contains exactly two explicit variable-valued, own-formal primitive-equality guards followed by a required variable-valued otherwise.
Retain the actual ordered value/if/formula cases and fallback; order records source provenance and never chooses a branch.
The otherwise branch has the residual guard of all explicit conditions; do not synthesize an original negation/formula node or prove coverage.
Require one actual written consistency Proof with one unlabelled Thus thesis; retain a Pending request without checking that proof.
Guard overlap requires consistency even with otherwise; FunctorExistence/Uniqueness remain means-only and cannot stand in for equals consistency.
Case guards are not enclosing assumptions; keep SourceFunctorGuardInput for its existing actual declaration assumptions.
No branch value, defining equation/axiom, fact, accepted definition/proof, canonical VC or Core/ControlFlow lowering is produced.
This is conditional functor definiens transport, not function-Binder certification or reflection pattern capture.
Imports/templates/nominal/attributed inputs, implicit captured parameters, partial coverage, means cases, nested conditionals and accepted correctness are deferred.
## Existing-table representation and public seam
Add Copy SourceFunctorDefiniensTarget::TermCase(SourcePrimaryTermId, SourceAtomicFormulaId) and Otherwise(SourcePrimaryTermId).
Reuse existing definientia rows for each actual TermCase and the final fallback term, with existing owner/site/range/context/ordinal fields.
Definition.definiens names the first row of its complete contiguous ordered owner group; reject orphans, duplicates and extra rows.
Add truthful SourceFunctorCorrectnessKind::Consistency and InitialObligationKind::FunctorConsistency at the actual CorrectnessCondition owner/proof anchor.
Keep existing handoff/projection/TypedAst/ResolvedTypedAst slots, row fields and getters; no new DTO, adapter, AST, slot or native carrier.
Construction, typing, installation, replay and real Case inspection consume the same actual source-row associations.
Authenticate original declaration/pattern/arity/symbol/contribution/visibility, formal identities and all parent/child/token/source-order relations.
Extend existing native formal lookup to the actual binary prefix pattern with authenticated punctuation and distinct loci; preserve other roles and refusals.
Use actual source context/type/primary/atomic lower results; no fake lower handoff, SignatureShell parsing or copied fixture-sized validator.
Extend actual single-owner Set return transport through its existing API; preserve the old truncated two-owner return refusal unchanged.
Derive each value type from its real formal and check the declared result; never assign the return type to manufacture branch compatibility.
Compatibility is structural source-type validation; no installed semantic typing or TypeTable replay credit is claimed.
Retain complete immutable lower/source fingerprints, source type universes and full value associations; stage and replay atomically.
Ship the blocked TermCase element rule with this producer; BinderExpression remains blocked on its real certification/UID prerequisites.
Preserve every old exact260 scalar/means, constructor, getter, lower/type, ownership and Core refusal/acceptance boundary.
## Test-first and trust
Add a small real active type-elaboration source/sidecar before semantic logic; existing mixed imported parser fixtures remain unchanged and parser-only.
Use reciprocal stage-compatible trace metadata without changing prior status/coverage/required evidence or activating old Fail seeds.
Independently review any source-derived inventory/count correction before edits; preserve unrelated semantic assertions and seeded identities.
Freeze a complete manually source-derived primary and success tail before logic; a safe unsupported interface-only scaffold may make new variants compile.
Require real frontend/resolver/original/lower/context/type/term/atomic prerequisites before genuine executable producer RED; compiler/setup failures are not RED.
Assert both loci, exact value/guard operand/source/context/type joins, ordered three-row group, fallback, Pending owner and both installed containers.
Cover alpha/layout/order, missing/extra/duplicate/foreign values/guards/fallback, wrong if/token/formal/context/type, source drift and coherent association swaps.
Require valid installation/Resolved baselines before forged replay refusals; preserve repeated/overlap/foreign atomicity and old protected consumers.
## Verification and exit
Present the public-seam/large-change plan first and keep the smallest source-driven extension of the existing tables.
Run fmt, workspace Clippy, BOTH lint_policy, complete checker/mizar-test owners, metadata, verifier standard, protected Core and short CLI groups.
New selected MIZ requires mandatory full cargo test before commit; run both evidence-heavy matrices and resolver edit-clean under owner/full selections.
Independent PRE/spec/test-intent/trust, complete-primary genuine RED, POST/sufficiency, pure volume and consistency precede final nine gates/score>=90.
