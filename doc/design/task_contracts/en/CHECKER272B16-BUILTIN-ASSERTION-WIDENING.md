# Task CHECKER272B16-BUILTIN-ASSERTION-WIDENING: intrinsic normalized Set-to-Object assertions
Canonical language: English; [Japanese pointer](../ja/CHECKER272B16-BUILTIN-ASSERTION-WIDENING.md).
Status: complete for this intrinsic relation; broader Task272 stays open. Tier: full; specification-derived old-test maintenance before edits.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: existing reconsider_builtin_widening FS case.
Depends on B2 conversion checkpoints and B14 assertion projection; no new public API or source carrier.
Authority: EN03 §§3.4–5 builtin hierarchy/widening, EN14 §14.2.3 admissibility, EN08 §8.2.2 and EN15/16 source conversion/skeleton.
Gap: source_drift/test_gap; the specified intrinsic relation lacks generic inference, derived formation and native goal consumption.
## Exact intrinsic relation
Keep the existing normalized-reflexive admissibility path unchanged.
Additionally admit actual Known normalized BuiltinSet -> asserted Known normalized BuiltinObject ONLY with empty args/attributes.
Use authentic normalized inputs; keep actual Set and written asserted Object types/IDs distinct, with original target provenance.
Existing authenticated normalization may yield plain builtin Set; do not invent a source-spelling exclusion or a nominal solver.
All other pairs retain existing reflexive/error/defer behavior; no new reverse, nonbuiltin, attributed or applied reachability is added.
No new general reachability, structure/mode graph, attribute/view/qua semantics, caller flag, fact payload or coercion evidence.
Recheck actual derived subject after substitution under the same relation; Object does not become Set.
Retain existing fact policy: caller-supplied facts follow Checked/Known versus Partial/Degraded; unrelated defers still block Known.
Native FormulaInputs have no facts; admissibility never proves the goal, refines a subject or discharges a Pending guard.
Keep SourceProofTerm/Step/five-tuple, source slots, native builder, alias/scalar/B2/frontier/citation and B9/B10/B11 caps fixed.
Inline the small normalized relation; no adapter, duplicated AST, fabricated raw SourceVariableType or unnecessary shared helper.
## Specification-derived test maintenance
Before edits, independently review the existing unit type_assertion_requires_reflexive_or_external_reachability_payload.
Only its actual Set/asserted Object case becomes Checked without reachability defer; the same pair with another defer stays Partial.
Remove only those two reachability diagnostics; retain missing asserted type, zero/multiple subjects, reflexive/fact and other-defer checks.
Preserve the external-reachability purpose with genuine Object-to-Set/nonbuiltin inputs and independently normalized types.
Any necessary unit/case naming update stays within that reviewed unit; never rebaseline from observed output.
Move ONLY B14's unchanged pure SetObject refusal literal into strong actual-Set/asserted-Object positive coverage.
Keep TargetChanged, UnusedBadAssertion, ObjectSet, Pending scalar Set-target, nominal/attribute and every unrelated refusal fixed.
Historical B14 contract is frozen; maintain only current paired boundaries and the new contract.
## Real source and independently replayed oracle
Keep the existing reconsider_builtin_widening source/sidecar/outcome/tags/refs/counts/trace/selection and Task49 fixed.
After its current strict conversion checks, consume native goals on the same authentic handoff; add no redundant builder or route refactor.
Three sealed bindings H/P/Y have references[H,P,P]; unused Reconsider Y has no invented goal value or declaration input.
FIVE original variable inputs, TWO original Is assertions with genuine Object targets, FOUR theorem/Let/ReconsiderItem/conclusion events.
Initial forall H:Set Is(H,Object); Let(H,P)->Is(P,Object); checkpoint and terminal retain that whole unproved goal.
Free[]/[P]/[P]/[P], avoided[], matched only at Let; authenticate actual owner, contexts, original sites and six Pending A intents.
Header uses Expression and proof uses Proof; request actual types remain Known Set, asserted types Known Object with distinct IDs.
Original H/P type sites remain Missing; unused Y retains Source(actual written Object target); installed type/fact tables stay empty.
SEPARATE conversion output retains one genuine Widening Candidate, Known builtin-widening fact and Satisfied query at the actual item.
Goal five-tuple checked output is witness guards, empty for this literal; never merge the conversion fact into it.
## Test-first, controls and exit
Compile full primary on OLD API: actual FS/legacy/source/conversion/native-A success then genuine goal DependencyMismatch RED.
Root reads the compiled primary/RED and independent PRE/test-intent/trust before production GO; no absent-API failure counts.
Cover normalized pair identity/provenance, reverse/plain/unsupported/other-defer and caller-fact policy without fake normalized IDs.
Reuse genuine private assertion fixtures for source replay, coherent all-Inferred/Checked false retyping and foreign/trace/source controls.
Cover authentic conversion/empty-guard separation, unused Y, prior-Let frontier and existing alias/checkpoint exclusions.
An Object witness may retain a real Pending Set guard under an admissible Object assertion; preserve actual Object and nonconsumable Pending.
Actual FS bypass needs genuine legacy/conversion/A success before native refusal and real FS Failed; no parser failure is causal evidence.
Run fmt, workspace Clippy, BOTH lint_policy, verifier standard, new/prior/native/protected/type-assertion groups and actual corpus CLI.
Run metadata/metamorphic and affected publication/refusal/fact/evidence checks; bound global Checked/Known effects before deferring heavy groups.
Full follows named cadence; unbounded Checked/Known-fact publication requires full, not an API/tier/task-boundary shortcut.
Update minimal paired type-checker/goal/harness boundaries, permitted plan/TODO/audit links; no trace or proof-acceptance promotion.
Independent PRE, genuine RED, POST implementation/trust/sufficiency, pure volume and consistency precede nine gates/score and commit.
