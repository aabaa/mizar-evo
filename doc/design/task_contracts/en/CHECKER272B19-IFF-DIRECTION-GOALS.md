# Task CHECKER272B19-IFF-DIRECTION-GOALS: unproved directional subproof goals
Canonical language: English; [Japanese pointer](../ja/CHECKER272B19-IFF-DIRECTION-GOALS.md).
Status: bounded implementation complete; broader Task272 remains open. Tier: full; source-owner and large-change plan first.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: existing native goal API and inactive source oracle.
Authority: EN16 §§16.3.2/16.3.3(6), EN15 proof/Assume/Thus syntax and existing EN14 admissibility; depends on A/B1–B18.
Gap: authentic nested Conclusion/Proof inputs lack two-direction goal association and independent source replay.
## Frozen bounded source and ownership
Admit an outer plain single-variable Set/Object Let followed by exactly two unlabelled Conclusion statements with child Proofs.
The remaining parent goal must be Iff(Phi,Psi); Phi/Psi are already supported atomic Equality/Membership/TypeAssertion goals.
Actual written forward Imp(Phi,Psi) and reverse Imp(Psi,Phi) must independently alpha-match ordered parent sides after Let.
Each real child Proof contains one unlabelled plain Assume of its own antecedent and one unlabelled unjustified Thus of its whole consequent (explicit or thesis).
No earlier Take/alias, child declaration/witness, By/Then/Hence, nesting, extra condition or additional proof statement in this profile.
Use each actual outer Conclusion as a local unproved goal owner and its genuine child Proof context, preserving the parent state/frontier.
Initialize local goals from their ACTUAL written Implication nodes; never synthesize original Imp/FormulaIds or lower the parent Iff into proofs.
Require authentic non-Blocked native A rows, source frames, Normal recovery, direct parent/context/owner/range and complete source census.
Child contexts are Proof, not Hereby/Now Block; inherit only authenticated prior outer Let values, not merely lexical visibility.
Keep the real theorem owner and original parent Iff; no fake return event, discharged direction or accepted equivalence is exposed.
Reuse existing Goal/Step/five-tuple/handoff/slot and per-owner state; no new public model/type/field/status/stack/adapter.
## Authentic typing and replay
Retain every original term/formula/type/reference site and occurrence context, including written implications outside their child Proofs.
Original inferred inputs are not rewritten by projection or Assume reduction; original/written/derived provenance remain distinct.
Each owner initializes once; child Assume reduces only its own written implication and terminal preserves its unproved consequent.
A Conclusion-owner initialization is not a terminal implication; actual child terminals retain the existing residual-Imp refusal.
Independent replay re-derives parent association/order, captured-free/frontier joins, actual child contexts, goals and every source event.
No child's residual may replace the saved parent Iff or initialize its sibling; reject duplicate-forward, swapped/missing/foreign directions.
Keep existing guard, actual Object/Set/Pending and fact policies; the new profile introduces no witness guard or usable hypothesis fact.
Preserve every old Compact/Consider/case/partial/alias/witness/whole-Iff profile and its semantic restrictions.
Broader Phi/Psi decomposition, arbitrary proof nesting/citations, proofs of directions and parent discharge remain deferred.
## Test-first and bookkeeping
Add inactive pass_formula_statement_iff_direction_goals_001 MIZ/sidecar with distinct Eq(x,x)/Is(x,Object) sides.
Link reciprocally only to existing EN16 thesis-tracking requirement; preserve its stage/status/coverage/required/prior tests.
Omit active tags; active FS/PV/AS/type/declaration selectors, Task49 and semantic/proof credit stay unchanged.
Generic discovery and Full syntax-smoke gain one pass source; independently review exact derived inventory/count maintenance before edits.
Do not change any old semantic oracle or expectation without an additional specification-derived pre-edit review.
## Executable oracle and exit
Freeze a complete compiled old-API primary on the ACTUAL new inactive source before production.
Clean frontend/resolver/native-A/base and genuine two child Proof scopes must precede intended goal DependencyMismatch RED.
Manually fix complete original binder/reference/term/formula/type/intent/context census and all per-owner before/after/free/avoid/matched goals.
Assert the saved unproved parent and independent written-Imp owner goals; no facts, witness requests or installed accepted tables.
Cover both direction orientation, distinct/renamed sides, child contexts, captures, source-order/owner/scope/type/goal/census forgeries.
Require actual native-A/base-positive source controls before claiming a direction-association causal refusal.
Preserve old negative intent and source identities; count-only changes follow discovery algorithms, never observed output rebaselines.
Run fmt, workspace Clippy, BOTH lint_policy, verifier STANDARD/PV/Driver, whole checker and unfiltered mizar-test library.
Run metadata/metamorphic, syntax-smoke and AS/declaration/type/FS CLI; full cargo test is mandatory for new syntax-smoke selection.
Update minimal paired owner/harness boundaries and permitted plan/TODO/audit links; overall272 and Task49 remain open.
Refresh only the affected live source-inventory size bucket from its owning generator; no policy or ratchet baseline changes.
Independent PRE/spec/test-intent/trust, genuine RED, POST/sufficiency, pure volume and consistency precede nine gates/score>=90 and commit.
