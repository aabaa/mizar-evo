# STEP 5 Source-Payload Family Decomposition

> Canonical language: English. Japanese companion:
> [../ja/payload_family_decomposition.md](../ja/payload_family_decomposition.md).

This document is the accepted output of checker Task 247. It inventories the
remaining source-derived checker payload families and assigns each family to a
bounded checker producer task, a prepared `mizar-test` Task-10 consumer
increment, or an explicit external gate. It is authority for task ownership
and dependencies only. It does not change language semantics, source code,
fixtures, expectations, trace status, test lists, or coverage credit.

## Authority And Entry Baseline

The inventory follows the repository authority order:

1. `doc/spec/en/`;
2. existing `.miz` sources;
3. `tests/coverage/spec_trace.toml`;
4. existing expectation sidecars;
5. checker and consumer design documents;
6. current checker and runner source as non-normative inventory evidence.

The read-only Task-247 entry baseline was clean at
`b0930a0c44a4f306d1a1ef2f9e66b4a7bd7f5cf6`. The active runner counts were
parse 96, declaration 4, and type elaboration 188. The repository plan was
403 cases / 368 requirements, type elaboration was 236/224, and pass/fail was
219/184. `mizar-test` had 272 unit tests and 17 production paths / 19,803
lines. Task 247 must preserve these values and the following oracles:

- CLI SHA-256: plan
  `0915fed1465c86f4b4d0420a35703fe93aed0cbb23b7304abff927195b4f5758`,
  parse `57d0fba9be95644890b80bfa4ec2cd992e47bb8ad4b67c130f5194ea73aa0273`,
  declaration
  `08b00a9f6fe70d94fe2c1b2bdebbdb5603bcee39bf3ceb460abe53f403bba7b5`,
  and type
  `1dadbeabb219f5853c713ad53aa1cc7cd720a0e80abd7f882e9e0a5ea7802625`;
- test-list SHA-256: raw
  `5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`
  and normalized
  `c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`;
- `mizar-test` production SHA-256: path
  `b36d96fed3207b415c95de27be11ade57654c6573a2f0637aa2d0a3d56aca01d`
  and content
  `5f9e716169964a861b71576957c05e2dc2538b5e0ff9d1025ef51a4bea6aa306`.

## Producer Contract Shared By Tasks 248-264 And 269-279

Each producer task is one nonempty logical task and one commit. Before editing,
the task must freeze its exact spec sections, source family, syntax-free input
payload, checker API consumer, `mizar-test` Task-10 consumer, visibility,
negative boundaries, tests, trace rows, coverage impact, and exit criteria.

Unless a row below narrows this contract, every producer task must:

- keep real `.miz` AST inspection and source-role extraction in `mizar-test`;
- pass only validated, syntax-free, source-ordered identities, ranges,
  provenance, recovery state, and semantic input payloads into
  `mizar-checker`;
- consume existing checker tables and algorithms instead of reconstructing
  checker results in the runner;
- transactionally project the implemented family through its applicable
  `TypedAst` and `ResolvedTypedAst` tables, preserving source identity, range,
  provenance, recovery, and predecessor links, and make the Task-10 consumer
  assert that final checker handoff; a producer task is not complete at an
  unconsumed input DTO;
- fail closed on missing, duplicate, reordered, recovered, cross-module,
  stale-provenance, wrong-role, and partial payloads;
- add checker unit/corruption/determinism coverage and the smallest
  spec-derived real-source consumer coverage needed for the family;
- preserve existing expectations unless the canonical specification itself
  explicitly authorizes a semantic change; new test-first cases must be
  directly derived from existing canonical requirements;
- change a deferred trace row or coverage credit only for the exact executable
  slice implemented by that later task, never merely because Task 247 named an
  owner.

All producer tasks forbid raw-syntax inspection in checker, parser or resolver
ownership takeover, proof search or proof acceptance, fabricated facts or
evidence, `CoreIr`/`ControlFlowIr`/VC construction, artifact schema invention,
public diagnostic-code invention, broad expectation rebaselining, and Steps
6/7 promotion.

## Accepted Producer Graph

The task identifiers below belong to `mizar-checker`. Existing joint Task 265
and Tasks 266-268 retain their already completed meanings and are deliberately
not reused.

| Task | Bounded producer and canonical authority | Dependencies and prepared consumer | Exit boundary |
|---|---|---|---|
| 248 | Source item, declaration-site, local-scope, ordinal, reserve/default, and `BindingEnv` context payloads. Specs 04, 11, 12, and 15; MC-G011/MC-G016. | Existing resolver identities and `mizar-test` Task 10. The consumer proves source order, shadowing, recovery, and declaration-to-binding identity. | No type result, RHS evaluation, proof context, or global name-resolution reconstruction. |
| 249 | Type head/application payloads: builtin, local/imported mode and structure radix, positional/bracket type arguments, term arguments, and written type-site identity. Specs 03, 05, 07, 08, 12, and 18 plus Appendix A; MC-G014/MC-G016/MC-G020. | Task 248; resolver symbol/provenance data; exact Task-248 two-row co-consumer and ten-reserve-root Task-10 type-elaboration consumer frozen in the crate plan. | Produces type inputs only; no expansion, inhabitation, subtyping, term/`qua` selection, or evidence result is fabricated. |
| 250 | Attribute-chain payloads: polarity, arguments, qualification/owner identity, local/imported provenance, order, and attributed type-site association. Specs 03, 06, 11, 12, and the 17 restricted-adjective boundary; MC-G014/MC-G020. | Tasks 248-249; the crate plan freezes exact Task-67/81/84/85 consumers, a 4/4/0 Task-249 co-handoff, a 4-chain/4-attribute/1-qualifier/1-group/1-actual Task-250 handoff, and synthetic prefix/order extraction coverage. | Preserves written prefix/list forms for later canonical equivalence, but synthesizes no arity/admissibility, owner compatibility, normalized instance, evidence, closure fact, or attribute truth. |
| 251 | Evidence-query requests and upstream dependency-fact inputs for mode expansion, structure base shape/constructor witness, attributed-type inhabitation, sethood/non-emptiness, inheritance, and coercion viability, including an opaque `ExistentialGateInput` request identity and dependency-fact references. Specs 03, 05-08, 13, 17, and 19; MC-G016/MC-G018/MC-G026. | Tasks 248-250; the exact representative selector is the Task-249 broad fixture plus Task-84/85. It emits 5 mode-expansion + 3 structure-inhabitation + 2 attributed requests, all missing. The same production Task-10 path injects and distinguishes requested/missing/rejected/supplied transport states through final `TypedAst`/`ResolvedTypedAst`; no semantic imported summary is assumed. | It owns request/site/provenance/reference transport only. `Supplied` is reference arrival, not accepted or consumable evidence by itself. Later Tasks 252-255/263/271/278 own new source sites; accepted evidence, theorem results, and artifact status remain external inputs. |
| 252 | Primary terms: variable/constant references, `it`, numerals, and transparent parentheses with binding, role, parent, and numeric-type request payloads. Specs 04.1-04.3/04.4.1/04.6 and 13.1/13.8.1-13.8.2/13.8.8; MC-G017/MC-G020. | Tasks 248-251; the exact real Task-10 selector is numeral equality, reserved-variable equality, and parenthesized reserved-variable equality with aggregate term/reference/numeric-request oracle 7/4/2. Synthetic producer/extractor tests own constant/`it` and eligible nested-parenthesis schema coverage until Tasks 260/264/269 provide their real owner payloads. References are authenticated by exact `BindingEnv::lookup` winner, not visibility alone. | Transport only: parentheses add no semantic term/type/FOL node, numeric requests add no result/fact, and no arbitrary application, structure/set term, formula, definition/current-result acceptance, or local-binding production is owned. Cross-family parent edges await Tasks 253-255. |
| 253 | Frozen/open: ordinary functor application source shape, Task-253-owned cross-family transparent wrappers/origins, individually authenticated ordinary candidate references, Task-252 primary-term / nested-application argument edges, and unresolved candidate-signature/application-result requests. Inline shape is synthetic only. Specs 10, 13.2, 15.2.3, and 19; MC-G017/MC-G020. | Tasks 248-252; exact Task-10 imported `(1 ++ 2)` case plus one new spec-derived same-module later-definiens application with a `DefinitionParameter` actual. Task 270 owns inline identity/formals/capture/substitution, Task 277 direct template transport, and Task 278 ordinary/template candidate collection/viability/winner. | Contract frozen; producer absent. No duplicate primary ownership, exhaustive candidate claim, overload winner, semantic signature/result, definition proof, template payload, or inline semantic identity. |
| 254 | Structure constructor, selector, and update terms with root/member/view identity, ordered fields, inheritance-path requests, and result-type requests. Specs 05 and 13.3; MC-G017/MC-G018. | Tasks 249-253; later Task 263 supplies source definition payloads; Task-10 structure-term consumer. | No constructor property arguments, invented field coverage, upcast winner, or structure evidence. |
| 255 | Set enumeration/comprehension, choice, and `qua` terms with generator scope, predicate/body links, sethood requests, written target type, and explicit conversion intent. Specs 07.8.1, 08.2, and 13.4-13.6; MC-G017/MC-G018. | Tasks 248-254; Task-10 set/choice/`qua` consumers. | No missing sethood proof, narrowing proof, implicit widening path, or comprehension fact is fabricated. |
| 256 | Atomic formulas: predicate applications, equality/inequality, membership, type assertions, and attribute assertions with complete term/type/attribute links and expected-input requests. Specs 09 and 14.2/14.5; MC-G017/MC-G020. | Tasks 249-255; Task-10 exact atomic-formula consumers. | No truth, theorem acceptance, inequality proof, or assertion fact without checker evidence. |
| 257 | Composite formulas and binders: constants, negation, binary connectives, quantified variables, child graph, contexts, roles, and source order. Specs 04.5 and 14.3-14.4; MC-G011/MC-G017/MC-G020. | Tasks 248-256; Task-10 connective/quantifier consumer. | No flattening that loses child identity, implicit closure, truth value, or theorem status. |
| 258 | General theorem-owner and statement-semantic shells, assumptions, conclusions, witnesses, labels/citations as resolver identities, local contexts, visibility-scoped input facts, and candidate fact inputs. Specs 15 and 16; MC-G019/MC-G020. | Tasks 248-257; resolver label facts; prepared `MT10-FS` consumer. | Records input/candidate assumptions and facts only; no verified premise publication, checked theorem fact, discharge, theorem acceptance, or proof closure is inferred. |
| 259 | Predicate-definition payloads: parameters, guards, definiens graph, properties/correctness-condition identity, `InitialObligationId`, source anchor input, and declaration provenance. Specs 09 and 16.6. | Exact Tasks 248/249/252/256 handoffs after a separate Task-248 two-parameter profile extension; pass Task-10 definition consumer. Tasks 253-255/257/258 are absent for the frozen source. | No recursive unfolding, guard-conditioned FOL property-VC construction, property proof, obligation discharge, `VcId`, accepted obligation, overload selection, or axiom publication. Future Task 272 retains the unconsumed justification subtree. |
| 260 | Functor-definition payloads: `equals`/`means`, parameters, guards, result type, definiens, properties/correctness-condition identity, `InitialObligationId`, source anchor input, and declaration provenance. Specs 10 and 16.6. | Tasks 248-259; Task-10 definition consumer. | No existence/uniqueness proof, obligation discharge, `VcId`, recursive unfolding, accepted result, or overload winner. |
| 261 | Attribute-definition payloads: subject/parameters, positive or negative definiens, guards, radix/qualification, and correctness obligation requests. Specs 06, 09, and 16.6. | Tasks 248-260; Task-10 attribute-definition consumer. | No attribute truth, cluster fact, existential evidence, accepted proof, or redefinition selection. |
| 262 | Mode-definition payloads: parameters, mode application, expansion/RHS, definiens, sethood/existence obligation requests, and declaration context. Specs 07 and 16.6. | Tasks 248-261; Task-10 mode-definition consumer. | Property implementations are Task 264; no accepted existence, expansion fact, or registration activation. |
| 263 | Exact zero-parameter structure-definition/inheritance payload: two declarations, four typed field/property selectors, one parent edge, exact root+path/view mappings and coverage, fields-only constructor order, and zero derived coherence requests for identical `set` types. Specs 05, 13.3, 16.6, and 19.2.2. | Tasks 249/249S, 263R, and the committed Tasks 248-262 boundary; one private structure-definition runner consumer. | No fabricated parameter/context, property constructor argument, inferred member identity, nonidentical-type goal/guard, accepted coherence, chosen upcast, fact, proof, or Core/CFG/VC payload. |
| 264 | Struct-property implementation payloads: owner/property identity, one local parameter, `means`/`equals` definiens, declared return association, correctness-condition identity, `InitialObligationId`, and source anchor input for existence/uniqueness. Specs 05, 07.4.1/07.8.2/07.10, 13.1.2/13.8.2, and 16.6.1/16.6.2/16.7.2. | Parser Task 48, Tasks 248-256/263/264R, and the dedicated property-implementation runner consumer. | Five-table transport plus pending initial-obligation intake only; no parameter/domain/return-type goal or guard, overlap/coherence detection, property value, `VcId`, discharge, acceptance, fact, proof, or Core/CFG/VC payload. |
| 269 | Proof-local declaration/binding and first-order local-term abbreviation payloads for `let`, `set`, `given`, `consider`, named `take`, and other local introductions, including context transitions, source-order closure, definition-site binding/RHS links, and capture-by-resolved-binding replay for later term references. Specs 04, 15.2-15.4, and 16.4. | Tasks 248-258; prepared `MT10-FS` consumer. | `deffunc`/`defpred` closure remains Task 270, `reconsider` coercion remains Task 271, and existential-binder matching, witness type obligations, and goal substitution remain Task 272. No proof search or accepted witness. |
| 270 | Proof-local `deffunc`/`defpred` closure payloads: formal identities, captured free variables, body graph, guard, substitution request, and capture-avoidance provenance. Specs 04.4.3, 10.11.3, 15.2.3-15.2.4; architecture 16. | Tasks 248-269; prepared `MT10-AS` capture consumer for the existing advanced-semantics trace row. The same producer may also supply proof-local declaration data to `MT10-FS`, but that does not transfer the trace-row ownership. | No substitution result without explicit replay evidence, no capture repair in the runner, and no accepted local theorem. |
| 271 | `reconsider` payloads: bindings, source/target types, written-or-omitted justification intent, widening/narrowing request, and proof-free evidence query. Specs 04.4.2, 08.2, 15.5.1, and 19.3.2. | Parser Task 47, Tasks 248-258 and 269; prepared `MT10-FS` consumer for proof-local families and `MT10-AS` consumer for the existing omitted-justification advanced-semantics fixture. | No omitted proof is accepted, no narrowing evidence is invented, and parser expectation drift is not repaired here. |
| 272 | Non-Task-180 proof skeleton and justification payloads: nested proof nodes, thesis/terminal goals, citations, local paths, case/suppose/now structure, and explicit pending/blocked states. For `take`, it also owns ordered witness-to-existential-binder matching, explicit witness type-obligation requests, the capture-avoiding goal-substitution trace, and the remaining goal. Specs 15.4.4/15.6/15.8/15.11.5 and 16.3-16.5. | Tasks 248-271, including Task-269 named-witness binding/RHS provenance; resolver label identities; prepared `MT10-FS` consumer plus the `MT10-AS` omitted-`reconsider` negative consumer where explicit pending/blocked intent must be asserted. | Task-180 exact tables remain Tasks 266-268; no substitution without authenticated term/binder inputs, invented type evidence, proof search, implicit closure, theorem acceptance, discharge, Core, or VC. |
| 273 | Registration-item and correctness payloads for existential, conditional, functorial, and reduction registrations, including guards, patterns, consequents, source order, correctness-condition identity, `InitialObligationId`, and source anchor input. Specs 07.8, 16.6.3, and 17.2-17.6. | Tasks 249-272; prepared `MT10-AS` consumer. | Produces pending registrations and obligation intake only; no `VcId`, discharge, accepted status, activation, closure, rewrite result, artifact, or theorem fact. |
| 274 | **Blocked-reserved:** import and validate canonical accepted verifier/artifact status, then activate only eligible registration rows with authenticated source/order/provenance. Specs 17.1/17.3.4/17.8.4 and existing checker activation policy. | Task 273 plus a future canonical verifier/artifact owner and schema. That upstream owner is currently unnamed; Task 274 is not executable until authority names it. | Never manufacture `Accepted` from source order, local checking, an obligation request, or a pending registration. Naming this gate grants no implementation authority. |
| 275 | Source-derived cluster closure trace: applicable registration identity, normalized input/output, ordered rule firing, bounds/loop/contradiction, and complete provenance. Spec 17.7/17.9; MC-G021/MC-G023. | Tasks 251, 256-257, 273-274; prepared `MT10-AS` consumer. | No unaccepted registration fires; no unrecorded fact, arbitrary theorem reasoning, cache/artifact result, or trace reconstruction in the runner. |
| 276 | Source-derived reduction trace: accepted reduction identity, guard evidence, orientation/termination checks, normalization steps, result dependence, loop/bound/failure, and provenance. Spec 17.6/17.9.4; MC-G023. | Tasks 251-257 and 273-275; prepared `MT10-AS` consumer. | `such` is applicability only; no unaccepted rewrite, hidden normalization, artifact/cache fabrication, or proof discharge. |
| 277 | Direct template-role declaration, formal/actual, constraint/guard, substitution-request, and provenance payloads already exposed by parser/syntax. Spec 18; MC-G027. | Tasks 248-264; prepared `MT10-AS` consumer. | Task 277 is executable and closes only direct template roles. It does not own or close the missing scheme/theorem roles in external Gate S1, and it invents no omitted actual, inference result, or substitution result. |
| 278 | Ordinary/template overload site and candidate payloads through existing collection, expansion, viability, specificity, ordinary-root selection, and inserted-view APIs. Specs 08, 18, and 19.1-19.4/19.6; MC-G027. | Tasks 249-257, 259-264, and 277; prepared `MT10-AS` consumer. Resolver Task 31's same-return declaration conflict is an independent Task-49 prerequisite and is not a Task-278 payload. | Evidence/comparison inputs must be explicit. No return-type tie-break, omitted comparison evidence, hidden `qua`, or redefinition refinement is invented. |
| 279 | Redefinition/notation producer: bound ordinary target/root, synonym/antonym relation, `coherence with` intent or omission, target-diagnostic payload, refinement candidate, accepted-coherence input, and exposed view. Specs 06.7, 09.6-9.7, 10.7-10.8, 11.1, and 19.5. | Tasks 259-264 and Task 278 ordinary-root output; prepared `MT10-AS` consumer. | No target is guessed when several roots apply; no coherence proof, priority edge, alias semantics, or accepted refinement is fabricated. |

The graph is acyclic at the checker boundary: Task 278 first produces ordinary
and template root results; Task 279 may then bind a redefinition to an already
identified ordinary root and feed only authenticated refinement data to the
existing selection layer. Task 279 does not feed a new ordinary-root candidate
back into Task 278.

## Prepared `mizar-test` Task-10 Runner Increments

These are consumer increments inside the already open `mizar-test` Task 10,
not new checker task numbers and not new top-level mizar-test tasks.

| Increment | Scope | Dependencies and exit criteria |
|---|---|---|
| `MT10-FS` | Add `formula-statement` stage/tag admission, plan/report output, deterministic rerun, expectation validation, and source-to-checker execution for formula/statement/proof-local families. Add a distinct future fixture and singular sidecar named `pass_formula_statement_reserved_variable_equality_smoke_001.miz`, with sidecar stage `formula_statement`; do not reclassify or add another sidecar to the active type-elaboration fixture. Its exact source is `reserve x for set;` followed by `theorem FormulaStatementReservedVariableEqualitySmoke: x = x;`. The producer must preserve the reserve, two terms, equality, theorem owner, statement shell, and explicit non-accepting omitted-justification state through `ResolvedTypedAst`. | Tasks 248-272. The new real source is the positive runner case; missing/duplicate/reordered/cross-owner corruption of the same bundle supplies fail-closed negative runner tests without inventing a semantic `.miz` failure. The existing `pass_type_elaboration_reserved_variable_equality_001` case and its sole sidecar remain unchanged and keep their current credit. Planned seeds are never counted as executed. It grants no truth, theorem acceptance, Core, VC, or Steps 6/7 credit. |
| `MT10-AS` | Add `advanced-semantics` stage/tag admission, plan/report output, deterministic rerun, expectation validation, and source-to-checker execution for definition, registration, cluster/reduction, template, overload, redefinition, reconsider/conversion, and definition-time capture-avoidance families. Its ordinary-root non-Task-49 smoke is a single local ordinary functor root with one `set`-typed argument and result, one `set` reserve, and one reflexive equality theorem containing a single application of that root; it has no template, redefinition, registration, cluster/reduction, or proof-acceptance input. Task 278 must freeze the parser-valid spelling against Specs 10/13/14/19 before editing. Its distinct capture smoke is the future `pass_advanced_semantics_definition_time_capture_avoidance_001.miz`, whose exact semantic fragment binds outer `m`, defines `defpred P(n be Nat) means n < m;`, then shadows the display name `m` before applying `P`; the runner must prove that the closure retains the outer resolved `m` identity and that formal substitution neither captures nor rewrites it. Task 270 must freeze the parser-valid enclosing proof shell before editing. The existing `fail_types_reconsider_omitted_justification_001` sidecar remains `advanced_semantics`; after parser Task 47 and Tasks 251/271-272, this runner must assert explicit omitted intent, the unavailable proof-free narrowing evidence, one pending/blocked non-accepting result, and `type.narrowing_requires_proof` without proof search. | Tasks 249-264, 270-273, and 277-279 for these consumers; missing/duplicate/reordered/cross-root candidate corruption, captured-identity/formal/substitution-request corruption, and missing/wrong reconsider intent/evidence/status corruption supply negative runner tests. Cases requiring accepted registrations additionally depend on Tasks 274-276. The smokes and mapped fail case must execute the real applicable producers without activating any other fixture in the Task-49 reconciliation set. No substitution result or omitted proof is credited as accepted. |

## Existing Boundary And Trace Ownership

Task 247 changes ownership notes only. The current umbrella extraction row and
all exact active diagnostic rows retain their status, tests, and coverage.

| Existing boundary family | Assigned owner |
|---|---|
| generic declaration/binding and non-builtin type payload extraction | Tasks 248-251 |
| argument-bearing/bracket mode or structure heads; imported structures; mode expansion/evidence requests | Tasks 249 and 251 |
| argument-bearing, qualified, imported, positive/negative attribute payloads | Task 250, with evidence requests in Task 251 |
| primary, imported-application, set-enumeration, structure, comprehension, choice, and `qua` terms | Tasks 252-255 |
| builtin/imported atomic formula and assertion boundaries | Task 256 |
| connective, constant, child-graph, and quantifier/binder boundaries | Task 257 |
| formula-statement, statement-proof, assumption, conclusion, and fact boundaries | Tasks 258, 269-272, and `MT10-FS` |
| predicate/functor/attribute definition boundaries | Tasks 259-261 |
| mode/structure/property/inheritance/constructor boundaries | Tasks 262-264, with parser Task 48 for property syntax |
| proof-local declaration, inline definition/capture, reconsider, and proof-skeleton boundaries | Tasks 269-272, with parser Task 47 for reconsider syntax |
| registration block/correctness and accepted activation | Task 273 plus blocked-reserved Task 274 |
| cluster and reduction source traces | Tasks 275-276 |
| direct template roles, overload, redefinition, and notation payloads | Tasks 277-279; missing scheme/theorem roles remain external Gate S1 |
| deferred `formula_statement` runner row | `MT10-FS` |
| deferred registration/cluster/reduction and overload `advanced_semantics` rows | `MT10-AS`, Tasks 273-279, and the stated external Gates A1 and S1 |
| deferred definition-time capture-avoidance row | Task 270 and `MT10-AS`; Task 270 may also supply proof-local payloads to `MT10-FS`, but the existing advanced-semantics trace row remains with `MT10-AS` |
| deferred type-soundness escape/guard row: witness leakage, local definition guards, sethood, and invalid `qua` | Tasks 258/272, Task 270, Tasks 251/255/271, and the applicable `MT10-FS` or `MT10-AS` increment; these cases are not part of the Task-49 24-fixture bundle |

The broad imported-attribute and imported-structure deferred rows remain
deferred. Their already active exact slices keep their current credit; Tasks
249-251 own only future broader source families.

## Task-49 Corpus Mapping

The semantic audit lists 25 adversarial fixtures. The
same-signature/different-return resolver fixture is already active and remains
an unchanged control outside the set below. At Task-247 entry the other 24
fixtures are inactive and form the exact **24-fixture reconciliation set**.
Resolver Task 31 is the sole activation owner for the same-return member of
that set and its `declaration_symbol` consumer. Task 49 owns activation of the
other 23 members after every mapped producer, runner, and gate is complete,
then reconciles and deduplicates all 24 without reactivating the resolver-owned
member.

| # | Literal fixture ID | Activation owner and required owners/gates |
|---:|---|---|
| 1 | `fail_cluster_reduce_cycle_orientation_001` | Task 49 after Tasks 273-274/276 and `MT10-AS` |
| 2 | `fail_cluster_reduce_commutative_orientation_001` | Task 49 after Tasks 273-274/276 and `MT10-AS` |
| 3 | `fail_cluster_reduce_fresh_variable_001` | Task 49 after Tasks 273-274/276 and `MT10-AS` |
| 4 | `fail_cluster_reduce_duplicating_variable_001` | Task 49 after Tasks 273-274/276 and `MT10-AS` |
| 5 | `fail_cluster_contradictory_consequent_001` | Task 49 after Tasks 250-251/256-257/273-275 and `MT10-AS` |
| 6 | `fail_cluster_functorial_for_guard_001` | Task 49 after Tasks 250-251/256-257/273-275 and `MT10-AS` |
| 7 | `fail_mode_missing_existential_001` | Task 49 after Tasks 251/262/273-275, Gate A1 where accepted status is required, and `MT10-AS` |
| 8 | `fail_mode_existential_after_declaration_001` | Task 49 after Tasks 251/262/273-275, Gate A1 where accepted status is required, and `MT10-AS` |
| 9 | `fail_structure_diamond_member_type_conflict_001` | Task 49 after Task 263 and `MT10-AS` |
| 10 | `fail_structure_inherit_duplicate_member_coverage_001` | Task 49 after Task 263 and `MT10-AS` |
| 11 | `fail_structure_inherit_cycle_001` | Task 49 after Task 263 and `MT10-AS` |
| 12 | `fail_structure_inherit_uncovered_member_001` | Task 49 after Task 263 and `MT10-AS` |
| 13 | `fail_structure_constructor_property_arg_001` | Task 49 after Tasks 254/263-264, parser Task 48, and `MT10-AS` |
| 14 | `fail_overload_incomparable_roots_001` | Task 49 after Tasks 255/263/277-278, Gate S1 where the missing role is required, and `MT10-AS` |
| 15 | `fail_overload_equivalent_roots_ambiguity_001` | Task 49 after Tasks 255/263/277-278, Gate S1 where the missing role is required, and `MT10-AS` |
| 16 | `fail_overload_template_equivalent_roots_ambiguity_001` | Task 49 after Tasks 255/263/277-278, Gate S1, and `MT10-AS` |
| 17 | `fail_overload_inheritance_path_ambiguity_001` | Task 49 after Tasks 255/263/277-278, Gate S1 where the missing role is required, and `MT10-AS` |
| 18 | `fail_resolve_same_signature_same_return_conflict_001` | **Resolver Task 31 sole activation owner**, using `declaration_symbol`; Task 49 reconciles/deduplicates only |
| 19 | `fail_types_qua_narrowing_001` | Task 49 after Tasks 255/263/278 and `MT10-AS` |
| 20 | `fail_types_qua_unrelated_struct_001` | Task 49 after Tasks 255/263/278 and `MT10-AS` |
| 21 | `fail_types_comprehension_missing_sethood_001` | Task 49 after Tasks 251/255 and `MT10-AS` |
| 22 | `fail_types_reconsider_omitted_justification_001` | Task 49 after parser Task 47, Tasks 251/271-272, and `MT10-AS`; preserve its existing advanced-semantics sidecar stage |
| 23 | `fail_mode_property_overlap_missing_coherence_001` | Task 49 after parser Task 48, Tasks 262-264, and `MT10-AS` |
| 24 | `fail_overload_redefine_ambiguous_target_001` | Task 49 after Tasks 278-279 and `MT10-AS` |

Task 49 remains one later 23-member activation plus 24-member reconciliation/
deduplication task. It may update the Task-29 deferred rows only after each
fixture really executes through its owning runner. It must keep the already
active different-return control, the resolver-owned same-return member, and
all independently covered rows from being counted twice.

## Disagreement Classification

| Protocol class | Task-247 finding and disposition |
|---|---|
| `spec_gap` | The pre-existing MC-G005 public-diagnostic-code allocation gap remains a nonblocking external registry/consumer-adoption gate. No new payload-family specification gap was found; the canonical English chapters are sufficient to name these families and negative boundaries. |
| `test_gap` | The 24 inactive Task-49 fixtures, broader source-derived family cases, formula-statement/advanced runners, and exact positive/negative semantic slices are not executable. Assigned to the graph above without changing status. |
| `design_drift` | The remaining families previously had only umbrella ownership. Closed by this accepted decomposition and paired trace/plan/TODO ownership updates. |
| `source_drift` | Checker APIs consume explicit payloads, but AST-wide real-source producers and several semantic consumers are absent. Assigned to Tasks 248-264 and 269-279. Parser Task 47 is a separate exact source drift. |
| `source_undocumented_behavior` | None found. Current exact source bridges remain narrower than the canonical requirements and already document their credit limits. |
| `test_expectation_drift` | The existing omitted-`reconsider` parser expectation conflicts with the canonical optional-justification syntax. Parser Task 47 owns it; Task 247 does not repair or rebaseline it. |
| `boundary_violation` | No current violation found. Reconstructing AST payloads in checker/core, fabricating evidence/acceptance, or making the runner compute checker results would create one and is explicitly forbidden. |
| `repo_metadata_conflict` | None found. No automatic metadata repair is authorized. |

## External Gates And Deferred Authority

- **Gate A1 — accepted registration status:** Task 274 is blocked-reserved
  because no canonical verifier/artifact owner or
  accepted-status schema is named. Task 247 does not invent that owner. A
  future canonical authority decision must name the producer, schema,
  authentication rules, and negative tests before Task 274 becomes executable.
- **Gate S1 — scheme/theorem source roles:** any missing module scheme
  declaration shell and scheme/theorem role payload remain gated on a future
  named canonical parser/syntax and resolver owner. This gate is not part of
  executable Task 277, and checker must not synthesize it.
- MC-G004 artifact/schema integration remains an unnamed external gate; no
  checker payload task may invent an artifact schema or reuse contract.
- MC-G005 public checker diagnostic allocation remains the existing nonblocking
  `spec_gap` and unnamed registry/consumer-adoption gate. Later tasks may
  preserve stable internal detail keys but may not allocate public numeric
  codes or aliases.
- Parser Tasks 47-48 and resolver Task 31 remain independently authorized
  prerequisites. They are not dependencies of completed Tasks 266-268 or Core
  Task 31.
- Steps 6/7 remain deferred. This graph does not authorize their promotion.

## Task-247 Exit Criteria

Task 247 is complete only when:

- every remaining family, MC-G owner, boundary fixture group, deferred runner
  row, and inactive Task-49 fixture has exactly one producer/consumer owner or
  explicit gate;
- English canonical and Japanese companion documents, checker plan/TODO/audits,
  mizar-test Task-10 documents, trace ownership notes, and the specification
  coverage audit agree;
- `spec_trace.toml` changes only deferred owner/reason wording and preserves
  every status, test list, and coverage class;
- no source, fixture, expectation, runner count, test list, or coverage credit
  changes;
- review-only specification, test-sufficiency, implementation-scope, and
  source/documentation consistency reviews end with no findings;
- the full baseline verification and count/hash oracles remain green;
- the Task-247 changes are committed as one docs/traceability logical task.

After that commit, Core Task 32 may consume this accepted graph immediately for
its own docs/traceability-only remaining-family decomposition. Core Task 32
does not need to wait for Tasks 248-264 and 269-279 to be implemented, but it must preserve
every gate and forbidden boundary recorded here.

Core Task 32 has now accepted
[source_family_decomposition.md](../../mizar-core/en/source_family_decomposition.md).
The absence of Chapter-20 algorithm rows from checker Tasks 248-279 is
intentional scope, not authority for a new checker task id: Core Tasks 42-47
are separate joint vertical tasks in which `mizar-test` owns AST extraction,
checker owns the syntax-free final projection, and Core owns lowering. They
preserve Gates A1/S1 for exact dependent slices. This ownership note changes
no checker source, task status, fixture, expectation, or coverage.

## Task 248 Completion

Task 248 is complete for exactly its bounded row. The implemented
`SourceBindingContextHandoff` preserves source-item/declaration order, resolver
shell and local-binding provenance, module/declaration context links, and the
structural local-to-reserve shadow relation through `TypedAst` and
`ResolvedTypedAst`. Its single active Task-10 fixture has no term-use lookup
site and produces no type result, RHS/formula/proof payload, fact, or
obligation. Task 249 is the next dependency-authorized producer; Tasks 269+
and Steps 6/7 are not promoted.

## Task 249 Frozen-Contract Prerequisite Completion

The paired crate plan now freezes Task 249's exact syntax-free tables,
ten-reserve-root broad fail consumer, 10/13/6 raw cardinalities and dual form
histograms, Task-248 two-Bare-builtin-row co-consumer, runner-only dependency
status, corruption/determinism matrix, one future trace row, expected count
deltas, and forbidden scope. This is a completed independent documentation
prerequisite, not Task 249 implementation. Source, tests, expectations, trace
rows/status, counts, hashes, coverage credit, Tasks 269+, and Steps 6/7 remain
unchanged.

## Task 249 Implementation Completion

The frozen producer is now implemented as the public syntax-free checker
`source_type` boundary plus the private exact mizar-test consumer. The broad
route publishes the exact 10/13/6 tables and stops at its runner-owned pending
detail; the unchanged Task-248 route co-installs its exact 2/2/0 dependency
regression. `TypedAst` owns the validated immutable handoff and
`ResolvedTypedAst` only clones it.

The real resolver exposed repeated non-emitting formal/field spellings in the
derived frozen scaffolding. This task-local `design_drift` and parse-only
preflight `test_gap` were repaired with distinct names only; no source-type
oracle or language intent changed. The new bounded diagnostic trace row closes
only the selected Task-249 `test_gap` and `source_drift`. Tasks 250+, 269+,
normalization, term/`qua` binding selection, later semantic payloads, and Steps
6/7 remain deferred.

## Task 250 Frozen-Contract Prerequisite Completion

The paired crate plan now freezes Task 250's exact syntax-free chain,
attribute, polarity, qualifier, argument-group, and actual tables; four
existing real consumers and their 4/4/0 plus 4/4/1/1/1 cardinality oracles;
Task-67/81 runner-only outcome progression; Task-84/85 evidence-query
preservation; legacy `AttributeInput` coexistence; synthetic `SurfaceAst`
prefix/order extractor coverage; one future trace row and required existing
trace-note updates; expected plan 411/373 and type 239/227; the corruption
matrix; and forbidden scope. This is a completed independent documentation
prerequisite, not Task 250 implementation. Source, tests, expectations, trace,
counts, hashes, coverage credit, Tasks 251+/269+, and Steps 6/7 remain
unchanged.

## Task 250 Source-Attribute Producer Completion

The frozen family is now implemented by the public syntax-free
`source_attribute` producer and one private runner extractor. Exactly the
Task-81/67/84/85 real routes publish the 4/4/0 Task-249 dependency and the
4-chain/4-attribute/1-qualifier/1-group/1-actual Task-250 handoff. The
synthetic prefix probe and checker corruption/determinism matrix close the
selected exact `test_gap` and raw transport `source_drift`. The bounded trace
row reaches plan 411/373 and type 239/227 with no new case or changed
admission. Semantic attribute instances and all evidence, truth, acceptance,
downstream IR, Tasks 251+/269+, and Steps 6/7 remain assigned to their
existing later owners.

## Task 251 Current-State Addendum

Task 251 implements the graph's request/reference transport node without
advancing any semantic evidence owner. The public checker handoff
authenticates exact Task-249 applications, optional Task-250 chains,
resolver symbol kinds, dependency keys, payload references, facts, and gate
associations before atomic publication. The private Task-10 consumer activates
only the broad Task-249 fixture and Task-84/85, producing ten missing requests
with histogram 5/3/2 and no response rows. Production four-state tests and the
checker corruption matrix close the bounded `source_drift` and `test_gap`.
Tasks 252-264/269-279 retain their existing dependency edges and exit
boundaries; Steps 6/7 are not promoted.

## Task 252 Current-State Addendum

Task 252 implements the graph's primary-term transport node without advancing
semantic term or formula ownership. The public checker handoff authenticates
five frozen source kinds, exact binding winners and producer-derived
binding-event ordinals, transparent parent closure, and unresolved numeric
requests before atomic publication. The private Task-10 consumer activates
only the three frozen real routes, producing the exact 7/4/2 aggregate;
synthetic probes cover constant, `it`, nested-parenthesis, and mixed-family
boundaries. The bounded `source_drift` and `test_gap` are closed. Tasks 253+
and 260/264/269 retain their existing dependency edges and semantic owners;
Steps 6/7 are not promoted.

## Task 253 Frozen-Contract Prerequisite

The paired crate plan now freezes the public five-table `source_application`
contract, Task-252 primary and nested-application argument edges, the
Task-253-owned transparent application-wrapper relation, and individually
authenticated resolver candidate references without claiming completeness or
a winner. Its exact future real selector is the existing imported
`1 ++ 2` route plus one new same-definition-block application of a completed
first functor from a later functor's definiens. That local actual is the inner
`DefinitionParameter`, authenticated by a reused Task-248 source-context
handoff, not the outer reserve. The aggregate Task-253
applications/wrappers/candidates/arguments/requests oracle is 2/1/2/3/4 and
the referenced Task-252 terms/references/numeric-requests slice is 3/1/2.

Inline zero/one/two-actual shapes are synthetic source-schema coverage only;
identity, formals, capture, and substitution remain Task 270. Template
applications are excluded whole-subtree: direct role/actual/guard/request
transport remains Task 277, while ordinary/template candidate
collection/viability/winner remains Task 278. This independent documentation
prerequisite resolves the selected
`design_drift` but does not implement Task 253. Its `source_drift` and
`test_gap` remain open; source, fixtures, expectations, trace status, counts,
hashes, executable credit, Tasks 254+, and Steps 6/7 are unchanged.

## Task 253 Current-State Addendum

Task 253 now implements the graph's functor-application transport node without
advancing semantic term, definition, formula, or overload-selection
ownership. The public checker handoff authenticates the five dense tables,
exact Task-252 debug fingerprint, root-only and nested argument edges,
transparent application wrappers, and individual resolver functor references
before atomic publication. The private Task-10 consumer activates exactly the
two frozen real routes and produces 2/1/2/3/4 with co-installed Task-252
3/1/2. Synthetic private-extractor probes cover all remaining source forms,
inline schema only, nesting, wrappers, degraded transport, candidate subsets,
and whole-template/mixed-family exclusion. The bounded `source_drift` and
`test_gap` are closed. Tasks 254+, 260, 270, 277, and 278 retain their
existing dependency edges and semantic owners; Steps 6/7 are not promoted.

## Task 254 Frozen-Contract Prerequisite

The paired crate plan freezes a public syntax-free `source_structure`
handoff with seven dense immutable tables: structure-family terms,
transparent wrappers, authenticated constructor roots, written member
segments, parser `FieldUpdate` containers, ordered child edges, and unresolved
requests. The exact future real consumer publishes Task-254
term/wrapper/root/member/field-update/edge/request counts
5/0/3/9/2/10/26 and composes the Task-252 primary/reference/numeric-request
slice 8/0/8. It has no Task-253 row or fingerprint.

Only constructor roots are authenticated resolver `Structure` references.
Written constructor labels, selector names, and update-path segments remain
source occurrences with unresolved member/path requests; repeated labels and
paths are preserved rather than decided or deduplicated. A parser
`FieldUpdate` owns one path and replacement association but no independent
term/type/fact. Task-254 child edges may point one-way to Task-252 roots,
same-context Task-253 root applications, or later same-context Task-254 rows.
A Task-253 root is not targeted by any Task-253 argument edge; a nested
Task-253 application is rejected rather than multiply owned by Task 254.
Task-253 applications containing structure children remain whole-subtree
excluded because the frozen Task-253 target vocabulary is not reopened.

Task 263 retains authenticated structure definitions, field/property kinds,
inheritance views, coverage/default decisions, constructor acceptance,
selector results, update-copy semantics, and exact-instance evidence. This
independent prerequisite closes the selected Task-254 `design_drift` only.
Production source, fixtures, sidecars, trace rows/status/counts, executable
credit, the measured 412/376 and 242/230 baseline, Tasks 255+/263-264, and
Steps 6/7 remain unchanged.

## Task 254 Current-State Addendum

Task 254 now implements the graph's structure-family source-transport node
without advancing semantic member or structure-definition ownership. The
public checker handoff authenticates seven dense tables, five arena-key
classes, resolver constructor roots, written member paths, `FieldUpdate`
associations and exact spellings, exhaustive direct written-child partitions,
Task-252/253/254 ownership in either install order, and conditional
fingerprints before atomic publication. The private Task-10 consumer reuses Task-248 contexts and
activates only the frozen three definientia, producing 5/0/3/9/2/10/26 with
Task-252 8/0/8. The bounded `source_drift`, `test_gap`, and implementation-time
context and cross-family installation-order `boundary_violation` are closed.
Tasks 255+ and 263-264 retain later
families and all structure semantics; Steps 6/7 are not promoted.

## Task 255 Frozen Source-Set-Term Family

Task 255 is frozen as a source-transport graph node after Tasks 248 and
252-254. Its future `source_set_term` handoff contains six dense tables for
set/choice/`qua` terms, transparent wrappers, written comprehension
generators, bare builtin target-type sites, ordered child edges, and unresolved
requests. Its exact real transaction is 4/0/1/3/4/7 with co-installed
Task-252 4/0/4 and no Task-253/254 target or fingerprint.

Edges may point one-way to Task-252 primary roots, Task-253 root
applications, Task-254 root structure terms, or nested Task-255 rows. Reverse
Task-253/254 parents containing Task-255 children remain whole-subtree
excluded. Task-249 declaration-linked type applications are not reused for
either term-owned or generator-owned Task-255 targets; the bounded slice
admits only authenticated bare `set`/`object` target sites.

The canonical row schemas use a maximal-effective-range partition: a primary
already owned by Task 253/254 and an application already owned by Task 254
cannot become a second Task-255 target. Unrelated optional handoffs are
range-disjoint, and later Task-253/254 installation revalidates any installed
Task-255 handoff. Task-255 request intents do not extend Task 251's frozen
type-application evidence origin.

Generator rows preserve written declarations but create no `BindingId` or
capture. Task 257 retains comprehension binder/context identity and
conditioned-comprehension formula ownership composes only after Tasks
256-257. Semantic result typing, sethood, choice nonemptiness/stability,
`qua` widening/reducts, facts, and acceptance remain outside Task 255. This
documentation-only prerequisite closes `design_drift`; the implementation
`source_drift` and `test_gap` remain open.

Task 255 is now implemented within this frozen boundary. The public
six-table producer, private exact consumer, optional Task-253/254
fingerprints, final `TypedAst`/`ResolvedTypedAst` ownership, bounded
fixture/trace row, and reviewed test matrix close the bounded
`source_drift` and `test_gap`. Task 257 still owns generator binding/capture,
Tasks 256-257 still own condition formulas, and no semantic set/choice/`qua`
credit is added.

## Task 256 Frozen Atomic-Formula Family

Task 256 is frozen as the next source-transport graph node after Tasks
248/252-255. Its future `source_atomic_formula` handoff contains eight dense
tables for formula occurrences, transparent wrappers, ordinary predicate
heads, individually authenticated predicate candidates, formula-owned bare
asserted-type sites, formula-owned simple attributes, direct term edges, and
unresolved expected-input requests.

The exact real selector reuses eight existing active fail fixtures and adds no
new `.miz`. Across those independent transactions the Task-256
formula/wrapper/head/candidate/type-site/attribute/edge/request aggregate is
`8/0/1/1/1/2/13/11`. Direct edges target ten Task-252 primaries, one
Task-253 root application, and two Task-255 root set terms. The complete
dependency aggregate is Task-252 `16/0/16`, Task-253 `1/1/1/2/2`, and
Task-255 `2/0/0/0/4/2`; there is no real Task-254 target.

The bounded assertion type and attributes are occurrence-specific Task-256
rows, not fabricated Task-249 declaration applications or Task-250 chains.
The initial slice admits only bare builtin `set`/`object` and simple
unqualified argument-free attributes. Requests transport operand expected
types, candidate signatures, type reachability, and attribute admissibility
as unresolved intent; they do not extend Task 251 or provide an answer, fact,
winner, truth, or accepted formula.

Single-segment ordinary predicates are admitted without chain conjunction,
segment negation, inline substitution, or template arguments. Task 257 owns
predicate chains and formula operators/binders, Task 270 owns inline closure
and substitution, Task 277 owns template roles, and Task 278 owns overload
collection and selection. Conditioned comprehensions remain a joint
Task-255/256/257 follow-up rather than reopening Task 255.

This documentation-only prerequisite closes Task-256 `design_drift`.
The public producer/final handoff remains bounded `source_drift`, and the
real/synthetic/corruption/install/exclusion matrix remains `test_gap`.
It changes no source, fixture, expectation, trace row/status/count, count, or
hash.

Task 256 is now implemented within that frozen boundary. The public
eight-table producer, private exact consumer, same-arena Task-252/253/255
composition, optional Task-253/254/255 fingerprints, eleven unresolved
requests, immutable final handoff, bounded reciprocal trace row, and reviewed
real/synthetic/corruption/install/exclusion matrix close the bounded
`source_drift` and `test_gap`. All eight prior semantic routes retain their
outcome and detail ownership. Task 257 still owns predicate chains, formula
operators/binders, and conditioned-comprehension composition; Tasks 270,
277, and 278 retain inline closure, template roles, and overload selection.

## Task 257A Frozen Composite-Formula/Binder Core

Fresh inventory decomposes the Task-257 umbrella before implementation.
Task 257A is the dependency-ready exact implication/universal/negation/
contradiction tree plus one explicit unused universal binder. Task 257B
retains broader connectives and quantifiers, implicit binders, bound use, and
capture. Task 257C retains predicate-chain and conditioned-comprehension
composition after any Task-256/255 contract extensions are separately
frozen.

The public source family has seven dense tables for formula occurrences,
transparent wrappers, unassigned roots, quantified binders, binder-owned type
sites, child edges, and unresolved requests. The one unchanged real
connective/quantifier fail source has the exact aggregate
`5/0/1/1/1/4/6`. Its extended Task-248-era `BindingEnv` schema is `2/1/4`:
the normal module-shell prefix, one expression body context, the
source-derived quantifier binding `x`, and the four unchanged module-shell
diagnostics. It does not create a Task-248 source-context handoff.

Formula rows are parent-before-child preorder. Four source-role edges form
implication left/right, universal body, and negated-formula relationships;
only the universal-body edge crosses from module context 0 to expression
context 1. Six requests preserve connective, constant, quantifier,
binder-type, and negation input intent without publishing any semantic
answer, fact, truth, theorem owner, proof, or acceptance.

The bounded binder type is an occurrence-specific bare builtin `set` site,
not a Task-249 declaration application. The resolver-shaped local binder
identity uses its written declaration range and stable local scope; no symbol,
contribution, declaration shell, opaque id, or generated counter is
fabricated.

Task 257A now implements this exact slice. The public transport, binding
extension, private consumer, one-shot `TypedAst` installation, final
`ResolvedTypedAst` clone preservation, and bounded corruption/context/install/
exclusion matrix close the recorded `source_drift` and `test_gap`. The
implementation adds only its covered reciprocal trace requirement over the
existing sidecar; the canonical source and existing semantic outcome/detail
intent are unchanged. Broader shapes, bound use/capture, executable wrappers,
predicate chains, and conditioned comprehensions remain Tasks 257B-C.

### Task 257B Dependency Refinement

Task 257B is split without changing the Task-257 authority or exit boundary.
Task 257B1 first composes the explicit universal/binder profile with one
Task-256 equality and two Task-252 binding references. Task 257B2 then adds
broader binary/repeated connective and grouping occurrences. Task 257B3 adds
existential, restricted/nested, and implicit-reserve binder forms. The graph
remains acyclic: binding context precedes primary terms, primary terms precede
atomic formulas, and all three precede formula composition.

Task-257B1 `bound_uses` rows are formula-side associations only. Task 252
remains the lookup-winner and source-reference owner, Task 256 remains the
equality/operand owner, and `BindingEntry::captured` remains reserved for
free-variable capture rather than direct quantified occurrences.

Task 257B1 is now implemented at that boundary. The exact pass route composes
all three predecessor families plus the `1/2` handoff without duplicating an
occurrence or moving an owner. The bounded `source_drift` and `test_gap` are
closed; Task 257B2 remains the next graph node.

### Task 257B2 connective/grouping node

The next node composes the unchanged Task-257 explicit-binder environment with
Task-252 numeral transport, Task-256 equality transport, a third exact
Task-257 composite profile, and the existing Task-257B1 cross-family table
shape. Its graph is `Task252 16/0/16 -> Task256 8/.../16/16 ->
Task257B2 8/6/1/1/1/7/9 -> composition 8/0`. Six
`ParenthesizedFormula` occurrences remain transparent wrapper rows. The node
owns fixed/repeated conjunction/disjunction, `iff`, and grouping only; Task
257B3 binder expansion, Task 257C predicate/comprehension composition, and
all semantic result families remain downstream.

### Task 257B2 implemented node

The frozen node is now executable as
`Task252 16/0/16 -> Task256 8/0/0/0/0/0/16/16 -> Task257B2
8/6/1/1/1/7/9 -> composition 8/0`. It transports the fixed/repeated
connective tree and wrappers only. Task 257B3, Task 257C, connective truth,
repetition expansion, and theorem ownership remain separate downstream nodes.

### Task 257B3 frozen nested-binder node

The next graph node first extends the Task-48 one-binding bare-set reserve
base to the four-binding nested environment, then builds Task-252 `6/6/0`,
Task-256 `3/0/0/0/0/0/6/6`, the fourth Task-257
`3/0/1/3/3/2/6` table profile, and formula composition `3/6`. It owns one
restricted explicit universal, one explicit existential, one nested
implicit-reserve universal, their two same-family child edges, three
atomic-parent associations, and six formula-side bound-use associations.

Task 48 remains the written reserve/default owner, Task 252 remains the
occurrence/reference and lookup-winner owner, and Task 256 remains the
equality/operand owner. Task 257B3 may authenticate the reserve binding as the
implicit binder-type source and preserve its shadow relation, but it may not
copy or reinterpret those predecessor rows. Quantified truth, witness
construction, restriction discharge, implicit theorem closure, capture
results, Task 257C, theorem ownership, and later semantic stages remain
downstream.

Task 257B3 implementation closes only the frozen composition-transport
`source_drift` and exact-consumer `test_gap`; predecessor row ownership and
downstream semantic responsibility do not move.

## Task 257C1 Frozen Decomposition

Task 257C1 is a lower-family Task-256 extension, not formula composition.
Task 252 owns the exact `3/0/3` numeral occurrences and requests. Task 256
owns the root, two segment/head/candidate rows, polarity-token provenance,
three global argument/boundary edges, and two candidate-signature requests in
exact profile `1/0/2/2/2/0/0/3/2`. The middle primary is shared by edge id,
never copied. Task 257 later owns implicit conjunction and semantic segment
negation; Task 278 later owns overload selection.

The separate Task-255 condition-bearing comprehension extension follows this
implementation prerequisite. Conditioned-comprehension and predicate-chain
composition remain separate future Task-257C slices, so this contract grants
neither family semantic credit.

Task 257C1 transport is now implemented at this lower-family boundary.
Predicate-chain composition remains unimplemented, and the next prerequisite
is still the separate Task-255 condition-bearing-comprehension transport.

## Task 255C1 Frozen Condition Node

The next graph node is
`Task252 4/0/4 -> Task253 1/0/1/2/2 -> Task255C1
1/0/1/1/1/1/2`. Task 252 owns mapper and condition numerals, Task 253 owns
the imported mapper, and Task 255 owns only the comprehension, generator,
bare type, colon association, direct condition-wrapper association, mapper
edge, and unresolved set requests.
Condition-contained lower-family rows are excluded from Task-255 child
discovery rather than copied or targeted.

Task 256 later owns the inner equality node/operand edges and Task 257C later
owns condition composition. Generator binding/capture and every semantic
result remain downstream. This prerequisite freezes the immutable objects
those later nodes must consume.

## Task 255C1 Implemented Boundary

The frozen dependency chain is now executable:
`Task252 4/0/4 -> Task253 1/0/1/2/2 -> Task255C1
1/0/1/1/1/1/2`. The condition wrapper is the recursive exclusion boundary;
the inner equality remains a downstream Task-256/257 consumer. No semantic
family was promoted.

## Task 257C2 Frozen Condition-Formula Edge

The next graph node is
`Task252 4/0/4 -> Task253 1/0/1/2/2 -> Task255
1/0/1/1/1/1/2 -> Task256 1/0/0/0/0/0/0/2/2 -> Task257C2 1`.
Task 256 takes only the inner equality and its two Task-252 operand edges;
Task 257C2 adds only the immutable condition-0-to-formula-0 association.
Task-255 wrapper ownership and every existing dense ID remain unchanged.
At the frozen pre-Task-256C1 baseline, this target edge was gated on the
separate lower task making only the authenticated Task-255 condition
containment executable in both set/atomic installation orders while
preserving unrelated overlap rejection. Task 256C1 now passes both orders;
the completed Task-257C2 implementation now publishes the target edge after
fresh preflight.

The edge is a dedicated cross-family handoff in
`source_formula_composition`, not a composite-formula placeholder. Generator
binding/capture, predicate-chain composition, equality truth, formula
results, and definition acceptance stay downstream.

## Task 256C1 Frozen Compatibility Edge

Task 256C1 adds no graph node or published edge. It changes only the
Task-256 validator's interpretation of one already authenticated containment:
Task-255 term 0/condition 0 encloses and directly parents Task-256 equality 0
in the same owner-term/formula context.
The immutable graph remains
`Task252 4/0/4 -> Task253 1/0/1/2/2 -> Task255
1/0/1/1/1/1/2` plus the dependency-neutral Task-256
`1/0/0/0/0/0/0/2/2`; Task-257C2 later adds the sole association edge.
No family ownership, ID, fingerprint, or semantic boundary changes.

## Task 257C3 Frozen Cross-Family Edge

The next graph slice is
`Task252 3/0/3 -> Task256 1/0/2/2/2/0/0/3/2 -> Task257C3 1/1`.
Task 257C3 owns only one association of segments 0/1 through the pre-existing
boundary edge 1 and one association of negative segment 1. Task 252 retains
primary 1; Task 256 retains the shared edge, segment polarity, candidates,
and resolver provenance. No new composite node or semantic formula owner is
introduced.

## Task 257C3 Implemented Cross-Family Edge

The frozen graph slice is now executable with the same ownership:
Task 252 retains all primary rows, Task 256 retains all segment/head/
candidate/edge/request and resolver-provenance rows, and Task 257C3 adds only
the two syntax-free associations. Typed/resolved ownership is mutually
exclusive with A/B/C2, and no semantic formula node is introduced.

## Tasks 258A/258B1 Frozen Statement Edges

Task 258 is an umbrella. Task 258A owns only the exact 81-byte
`FormulaStatementReservedVariableEqualitySmoke` transaction:
one resolver-authenticated theorem/label owner, one theorem-proposition
shell, one statement context, one visible reserved-type-guard input, and one
unverified atomic-equality candidate (`1/1/1/1/1`). It fingerprints exact
Task-252 `2/2/0` and Task-256 `1/0/0/0/0/0/2/2` handoffs and owns an
exact clone/fingerprint of the validated Task-48 `BindingEnv`. Task 248 and
Task 258A typed owners are mutually exclusive across the production
Task-248-first path, named reverse checker-test seam, and final assembly. It
creates no checked formula,
`statement_semantics` row, proof intent, accepted fact, or runner coverage.

The old Task-258B umbrella is now decomposed. Task 258B1 owns only the exact
139-byte nested equality slice: the Task-48 `3/1/0` proof-context extension,
Task-252 `8/8/0`, Task-256 `4/0/0/0/0/0/0/8/8`, four statement/context/
guard/candidate rows (`1/4/4/4/4`), and one replay-authenticated proof-step
label/local citation association (`1/1`) backed by a two-pass
77-node/root-76 resolver AST whose node 68 is the sole resolved/keyed
reference site. It transports the inner/outer conclusion shells and nested
contexts without publishing a fact or proof result.

Task 258B2+ retains explicit assumptions and witnesses, composite roots, and
broader imported/outer/inner visibility. Tasks 269-272 retain proof-local
declaration and proof/justification ownership. The deferred `MT10-FS` row
stays deferred until the complete dependency chain and runner are
executable.

### Task 258B1 Implementation Closure

The B1 statement/reference edge is implemented as frozen and remains a
transport-only family. The next statement work is Task 258B2+ after fresh
contract review; Tasks 269–272 still own local declarations, closures,
coercion intent, proof skeletons, justification meaning, goals, and
acceptance. No family edge is reclassified by the implementation.

### Task 258B2 Frozen Family Edge

Task 258B2 is the next minimal transport-only edge:
Task 48 `2/1/0` → Task 252 `6/6/0` → Task 256
`3/0/0/0/0/0/0/6/6` → one source-statement handoff with profile
`1/3/3/3/3`. It carries one unlabeled single assumption and the direct
conclusion under one proof context. There is deliberately no Task-258B1
reference edge and no fact, premise-acceptance, checked-formula, statement
semantic, proof, goal, or theorem-result edge.

Task 258B3 retains witness transport, Task 258B4 composite theorem roots,
and Task 258B5 broader imported/outer/inner visibility. Tasks 269–272 retain
proof-local declarations and proof/justification semantics. The deferred
`MT10-FS` row therefore remains deferred; this documentation prerequisite
earns no executable coverage credit and changes no source, fixture,
sidecar, expectation, trace status/count, or existing test list/hash.

### Task 258B2 Implemented Family Edge

The frozen single-assumption edge is now executable but remains
transport-only: one direct equality theorem, one unlabeled assumption, one
direct conclusion, and no reference or semantic handoff. This closes the B2
source/test gaps without consuming B3 witness, B4 composite-root, B5 broader
visibility, or Tasks 269–272 proof-semantic ownership.

### Task 258B3 Witness Companion

The Task-258B3 family remains statement transport but separates unlike
payloads. The existing base owns one theorem formula and one conclusion
formula at source ordinals 0/2. The new
`SourceStatementWitnessHandoff` owns only the unnamed primary-term witness
between them at source ordinal 1 and within-take ordinal 0. It depends on the
base and Task-252 fingerprints and installs only as their authenticated pair.

This split prevents a term-only `take` item from gaining a fabricated
formula, statement context, guard, candidate fact, or resolver bundle.
Task 252 owns the witness term/reference; Task 256 explicitly excludes it.
Tasks 269–272 still own existential matching, obligations, substitution,
abbreviation, and proof state. Tasks 258B3N/M retain named, multiple, and
other witness-term transport before B4; B4/B5 retain composite-root and
visibility families.

Task 258B3 now implements only the frozen unnamed-primary witness companion.
The family partition and all B3N/M, B4/B5, and 269–272 ownership remain
unchanged after implementation.

### Task 258B3N Named-Witness Edge

B3N now has a frozen syntax-only edge: one named primary-term witness and
one dense name row. It extends B3 transport without creating a binding or
abbreviation. Task 269 exclusively owns the later local binding, RHS link,
capture-by-resolved-binding abbreviation replay, and context transition.
Task 272 exclusively owns existential-binder matching, witness type
obligations, capture-avoiding goal substitution, and the remaining goal;
Task 270 remains `deffunc`/`defpred` closure and Task 271 remains
`reconsider`. B3M retains multiple/other witness terms, and B4/B5 retain
roots and visibility.

### Task 258B3N Named-Witness Result

The named-primary edge is now implemented as syntax-only transport with one
witness row and one name row. It adds no binding or semantic edge and does
not consume B3M, B4/B5, or Tasks 269–272 ownership. Task 258B3M is the next
dependency-ready documentation prerequisite.

### Task 258B3M1 Mixed Multiple-Witness Edge

The former B3M umbrella is split. B3M1 owns only a two-row syntax edge:
named primary term 2 then unnamed primary term 3, one shared `take`, one
dense name row, shared source ordinal 1, and within-`take` ordinals 0/1.
Task 252 owns both reserved-variable references; Task 256 excludes them.
Task 269 retains name binding/abbreviation, Task 272 retains ordered
existential goal effects, and B3M2 retains every non-reserved-variable or
other witness-term shape. B4/B5 still own composite roots and visibility.

### Task 258B3M1 Implementation Closure

The exact reserved-variable mixed edge is complete: named witness 0,
unnamed witness 1, and name row 0 are syntax-only and dense. Resolver-owned
`y`, binding, abbreviation, ordered goal effect, and every other witness
term shape remain excluded. B3M2 is now the next dependency before B4.

### Task 258B3M2A Numeral-Witness Edge

B3M2 is split into B3M2A and B3M2B. B3M2A owns only one unnamed witness
whose existing primary term 2 has kind `Numeral`, spelling `101`, and
Task-252 numeric request 0. It adds one syntax-only witness row and no name
row, binding, atomic edge, or semantic edge. Task 252 retains the numeral
and request; Task 256 excludes term 2; Task 269 receives no binding; Task
272 retains typing, existential matching, substitution, goal, and proof
effects. B3M2B1 retains the exact parenthesized wrapper plus
reserved-variable child; B3M2B2 retains compound, application, selector,
update, set, choice, and other authority-valid witness shapes. `it` remains
eligible only in a Chapter-13-valid `means` context. B4/B5 remain blocked
behind B3M2B2.

### Task 258B3M2A Implementation Closure

The private B3M2A profile now realizes exactly this syntax-only edge:
Task-252 numeral term 2 and numeric request 0 are reused, Task-256 edges and
requests cover only terms `0/1/3/4`, and one unnamed witness/no names is
published atomically with the base statement handoff. No binding, semantic
edge, active route, public schema, or neighboring family changed. B3M2B
remains the next unimplemented edge before B4/B5.

### Task 258B3M2B1 Parenthesized-Witness Edge

B3M2B1 owns one syntax-only witness target over Task-252 parenthesized term
2 and its child variable term 3. Task 252 owns the parent edge and the
child-only reference; Task 256 excludes both terms from atomic edges and
uses only `[0,1]` / `[4,5]`. Task 258 owns only the witness/take and base
statement rows, with `1 witness / 0 names`. Tasks 253–255 receive no
application, structure, selector, update, set, choice, wrapper, or edge.
Task 269 adds no binding; Task 272 retains every semantic effect. B3M2B2
retains nested parentheses, application, structure constructor/selector/
update, set, choice, and every other authority-valid witness term. `it`
remains eligible only in a Chapter-13-valid `means` definition or property
context. B3M2B2 is next before B4/B5.

### Task 258B3M2B1 Implementation Closure

The private B3M2B1 profile now realizes exactly this syntax-only edge.
Task-252 keeps the parenthesized wrapper, child reference, and parent link;
Task-256 keeps only equality pairs `[0,1]` / `[4,5]`; Task 258 publishes
one unnamed outer-term witness/no names atomically with the base statement.
No application, structure, selector, update, set, choice, binding, semantic
edge, active route, public schema, or neighboring family changed. B3M2B2
remains the next unimplemented edge before B4/B5.

### Task 258B3M2B2A Nested-Parenthesized Witness Edge

B3M2B2 is now split. B3M2B2A owns only one syntax-only witness target over
a two-level Task-252 parenthesized chain: outer term 2 parents inner term 3,
which parents reserved-variable term 4. Task 252 owns all three rows and
the child-only reference; Task 256 excludes the complete `2/3/4` subtree
and keeps equality pairs `[0,1]` / `[5,6]`. Task 258 owns only one unnamed
outer-term witness/no names and the base statement rows. Tasks 253–255
receive no application, structure, selector, update, set, choice, wrapper,
or cross-family edge. Task 269 adds no binding and Task 272 retains every
semantic effect. B3M2B2B retains application, structure constructor/
selector/update, set, choice, compound, and every other authority-valid
witness term. B4/B5 remain blocked behind B3M2B2B.

### Task 258B3M2B2A Implementation Closure

The private statement family now owns only the exact two-level
parenthesized witness. Its outer/inner/leaf primary chain is authenticated
as one witness subtree and excluded from every atomic edge/request.
Application, structure constructor/selector/update, set, choice, compound,
and deeper-parentheses families remain in B3M2B2B; no cross-family edge or
semantic owner was added.

### Task 258B3M2B2B1P Lower Application Seam

B3M2B2B is split dependency-first. B1P owns only the private runner
capability to rebuild the existing Task-253 unwrapped imported application
in an explicitly supplied proof context. It owns no Task-258 witness row,
new payload family, public schema, or semantic edge. B1A retains the exact
application-witness cross-family edge; B1B+ retains other Task-253 forms;
B2+ and B3+ retain Task-254 and Task-255 witness forms respectively.

### Task 258B3M2B2B1P Completion Boundary

B1P now supplies only the private proof-context Task-253 reuse seam. It
publishes no new checker family or Task-258 row and adds no cross-family
edge. The application-to-witness edge remains wholly owned by the next
B1A frozen contract and implementation; all B1B+/B2+/B3+ ownership remains
deferred.

## Task 258B5 Decomposition And B5A Frozen Profile

The B5 umbrella is decomposed before implementation:

| task | owned edge | prerequisite still required |
| --- | --- | --- |
| `258B5A` | local proof-step label at `[0]` cited by a later descendant conclusion at `[0,1]` | this frozen contract |
| `258B5B` | imported public theorem visibility | imported-summary/public provenance contract |
| `258B5C` | active inner-to-outer and sibling rejection | test-first negative route and diagnostic contract |

B5A uses one theorem owner, five statement/context/input/candidate rows, one
private/local-only proof-step label, and one simple-local citation. All five
formulas are Task-256 `Atomic(0..4)` values and every candidate remains
`UnverifiedProposition`; no accepted fact or proof result is created.
Cross-family installation is legal only for the exact B1 base/reference pair
or the exact B5A base/reference pair. B5B/B5C, qualified/grouped/bulk
citations, facts, proof progress, theorem acceptance, and IR remain deferred.

## Task 258B5B Imported-Citation Decomposition

B5B is the positive imported-public-theorem profile only. It follows B5A but
is split into three commits: this frozen documentation prerequisite, a
two-file lower opt-in imported-label prerequisite, and a seven-consumer upper
implementation. B5C active confinement negatives remain separate.

The upper profile is one theorem owner, two statement/context/input/candidate
rows, zero local-label rows, and one imported citation. Task-256 formulas are
`Atomic(0..1)` and both candidates remain `UnverifiedProposition`. Exact
ownership is four terms, two formulas, and two statements (`8/49`).
Cross-family installation admits only a matched B1, B5A, or B5B
base/reference pair; a B5A local-label profile cannot pair with B5B imported
provenance.

Qualified/grouped/bulk imports, private-import diagnostics, facts, proof
progress, truth, theorem acceptance/publication, status propagation, ATP,
Core, CFG, and VC remain deferred.

### Task 258B3M2B2B1A Frozen Cross-Family Edge

B1A adds one directed ownership edge only:
`SourceStatementWitness(0) -> SourceFunctorApplication(0)`. Task 252 owns
the two numeral argument primaries, Task 253 owns the imported infix
application, and Task 258 owns the take/witness association. Task 256 owns
only the theorem and conclusion equality formulas and remains independent
of the application fingerprint. The atomic TypedAst bundle prevents any
partially published or reverse edge. Structure, set/choice/qualification,
semantic term, formula, proof, and goal families are excluded.

### Task 258B3M2B2B1A Implemented Cross-Family Edge

The frozen directed edge is now implemented exactly:
`SourceStatementWitness(0) -> SourceFunctorApplication(0)`. The witness
stores `Application(0)` and a matching optional application fingerprint;
the application remains owned by Task 253 and its numeral arguments remain
owned by Task 252. One atomic installer publishes the application, statement,
and witness bundle or publishes nothing, while final assembly repeats the
same validation. No reverse edge, duplicate lower row, wrapper ownership,
Task-254/255 ownership, structure/set/choice/qualification edge, or
semantic/proof/goal family was added.

### Task 258B3M2B2B1B1P Wrapped Task-253 Seam

B1B1P remains wholly inside the Task-253 runner producer boundary. It adds no
payload family or cross-family edge: Task 252 owns numeral primaries 2/3,
Task 253 owns application 0 plus wrapper 0, and Task 258 owns nothing yet.
The future B1B1 edge will still be
`SourceStatementWitness(0) -> SourceFunctorApplication(0)`; wrapper 0 is
authenticated containment metadata, not a witness target. Task-254/255 and
all semantic/proof/goal families remain excluded.

### Task 258B3M2B2B1B1P Completion Boundary

Implementation adds only the private extraction/reuse seam and no checker
payload family or cross-family edge. Task 252 still owns primaries 2/3 and
Task 253 owns application/wrapper 0; exact resolver authentication narrows
admission without publishing another row. Task 258 still owns nothing, and
all statement/witness/semantic/proof/goal families remain deferred to B1B1.

### Task 258B3M2B2B1B1 Frozen Cross-Family Edge

B1B1 adds exactly the existing edge shape
`SourceStatementWitness(0) -> SourceFunctorApplication(0)` for the wrapped
source. Task 252 owns primaries 2/3; Task 253 owns application 0, wrapper 0,
candidate, arguments, and requests; Task 258 owns one take/witness pair.
Wrapper 0 is authenticated containment only and is never a reverse edge or
witness target. Task 256 retains only equality edges `[0,1]` and `[4,5]`.

The public B1A schema and atomic three-handoff installer are reused without
broadening B1A. The new profile is crate-private. Task-254/255,
structure/set/choice/qualification, semantic term, proof, goal, Core/CFG/VC,
and every other family remain excluded.

### Task 258B3M2B2B1B1 Implemented Cross-Family Edge

The frozen witness-to-application edge is now installed atomically for the
private B1B1 profile. Ownership is unchanged: Task 252 retains primaries,
Task 253 retains application/wrapper/candidate/requests, and Task 258 owns
only the take/witness pair. No reverse wrapper edge, new payload family, or
semantic/proof/goal edge was introduced.

### Task 258B3M2B2B2P Frozen Lower-Family Boundary

B2P adds no payload family or cross-family edge. It freezes a private runner
reuse seam for the existing Task-254 family only: Task 254 owns constructor
59 and assignment members 20/24, while qualified root 52 remains unowned
provenance traversal. Task 252 uses 54/57 only as private extraction roots,
publishes numeral rows at 53/56, and owns 53/56 as
`source.term.numeral` while 54/57 remain arena-unowned.
Task 258 owns nothing in B2P. The future B2A witness-to-structure edge,
future B2B selector family under §5.7, and B2C update/`FieldUpdate` families
remain separate. Semantic term, proof, fact, goal, Core/CFG/VC, inheritance,
typing, defaults, and coverage edges remain absent.

### Task 258B3M2B2B2P Implemented Lower-Family Boundary

The private B2P selector and reuse seam now install the frozen Task-254
constructor profile with shared Task-252 primaries. They add no payload
family or cross-family edge: Task 254 still owns only constructor 59 and
members 20/24, Task 252 owns published numeral sites 53/56, and Task 258
owns nothing. The B2A witness-to-structure edge remains next; B2B/B2C and
all semantic/proof/goal edges remain deferred.

### Task 258B3M2B2B2A Frozen Witness-to-Structure Edge

B2A adds exactly one future directed edge:
`SourceStatementWitness(0) -> SourceStructureTerm(0)`. The Task-258 base
transaction owns its theorem/conclusion statement rows; the B2A extension
owns only the take/witness occurrence and edge. Task 254 retains its
constructor/member/request rows; structure-root row 0 authenticates the
arena-unowned traversal node 52. Task 252 retains primary children. Task 256
retains only the two equality formulas, with zero direct `Structure` targets
and no structure fingerprint. No reverse edge, wrapper target, field/member
identity, selector/update family, semantic/proof/goal edge, or coverage
credit is authorized. B2B/B2C remain separate.

### Task 258B3M2B2B2A Implemented Witness-to-Structure Edge

The exact directed `SourceStatementWitness(0) ->
SourceStructureTerm(0)` edge is installed. Task 258 owns only the
theorem/conclusion base rows and take/witness 62/61; Task 254 retains
constructor/member/request ownership and Task 252 retains the primary
children. Task 256 remains equality-only with no direct structure target or
fingerprint and is revalidated only at the atomic typed/final boundary.

No reverse edge, field/member identity, selector/update payload,
semantic/proof/goal edge, active route, or coverage credit was added. B2B
selector and B2C update/`FieldUpdate` remain separate deferred families.

### Task 258B3M2B2B2BP Frozen Private Selector Lower Edge

B2BP owns only a runner-private reuse path for the Task-254 chain
`Structure(0 selector) -> Structure(1 constructor) -> Primary(2/3)`.
Task 254 retains selector/constructor/member/edge/request ownership; Task
252 retains primary values. Task 258 contributes no row or edge.

The future seam may route the exact selector profile through existing
proof-context extraction but adds no cross-family witness edge, public API,
semantic edge, or coverage credit. B2B is the later direct-selector witness
consumer; B2C remains the update/`FieldUpdate` owner.

The private lower edge is now implemented and tested without adding a
Task-258 row. Task 254 still owns the selector/constructor/member/request
rows, Task 252 owns the primary values, and the exact chain remains
`Structure(0) -> Structure(1) -> Primary(2/3)`. B2B remains the only future
witness consumer.

### Task 258B3M2B2B2B Frozen Witness-to-Selector Edge

B2B adds exactly one directed cross-family edge:
`SourceStatementWitness(0) -> SourceStructureTerm(0)`. Structure term 0 is
the selector; Task 254 retains the lower chain
`Structure(0 selector) -> Structure(1 constructor) -> Primary(2/3)`.
Task 258 owns only theorem/conclusion base rows, take/witness 65/64, and the
new directed edge. It does not own either structure term, root, member,
primary child, or reverse edge.

Task 252 retains primaries; Task 254 retains selector/constructor/member/
request rows; Task 256 retains equality-only `BuiltinPredicateApplication`
nodes 51/70 while `FormulaExpression` containers 52/71 remain unowned, with
no direct structure target or fingerprint. The existing structure fingerprint in the
witness handoff is dependency authentication, not a semantic edge.
B2C update/`FieldUpdate`, selector identity/type/call/chain, semantic term,
proof, goal, Core/CFG/VC, and coverage credit remain absent.

### Task 258B3M2B2B2B Implemented Witness-to-Selector Edge

The exact directed
`SourceStatementWitness(0) -> SourceStructureTerm(0)` edge is now installed
for the private B2B profile. Task 258 owns only theorem/conclusion nodes
`75/73`, take/witness nodes `65/64`, and that edge. Task 254 retains
selector/constructor/member/request ownership and
`Structure(0) -> Structure(1) -> Primary(2/3)`; Task 252 retains its
primary rows.

Task 256 remains equality-only and owns nodes `51/70`; containers `52/71`
remain arena-unowned. B2A and B2B are separately authenticated atomic
siblings, so target, fingerprint, ownership, or lower-family hybrids reject
without publication. No reverse, selector-semantic, update/`FieldUpdate`,
proof, goal, Core/CFG/VC, active-route, or coverage edge was added.

### Task 258B3M2B2B2CP Frozen Lower-Family Boundary

B2CP adds no Task-258 edge. It freezes only a runner-private reuse of the
existing Task-254 lower graph for one functional update:

```text
Structure(0 functional-update)
  -> UpdateBase -> Structure(1 constructor)
  -> UpdateValue(member 0) -> Primary(4)
Structure(1 constructor)
  -> ConstructorValue(member 1/2) -> Primary(2/3)
FieldUpdate(0) -> member 0
```

Task 252 retains all seven primary rows. Task 254 retains update,
constructor, three member rows, the non-term `FieldUpdate`, four directed
child edges, and nine unresolved requests. B2CP owns no theorem, statement,
take, witness, formula, reverse edge, or typed/final row. The later B2C
consumer alone may own take/witness nodes 72/71 and add a
witness-to-`Structure(0)` edge after B2CP is implemented. Task 256 later
owns only equality nodes 55/77 and excludes the update subtree; formula
containers 56/78 remain unowned. Functional-copy meaning, member identity,
replacement/result typing, proof/goal semantics, active routes, and
coverage credit remain absent.

### Task 258B3M2B2B2CP Implemented Lower-Family Seam

CPC1 correction commit `ee267d9c` is complete. B2CP now privately
authenticates and republishes exactly the frozen Task-254
functional-update/constructor/member/`FieldUpdate` graph in the existing
proof context; it adds no payload-family row or upper edge. The two exact
runner tests pass, so the prerequisite `design_drift`, bounded
`source_drift`, and `test_gap` are closed. Final test-sufficiency and
implementation re-reviews have no findings.

Task 252/254 ownership is unchanged. Task 256/258, B2C witness ownership,
public/active routes, functional-copy and type/result meaning, proof/goal/
theorem behavior, and IR remain deferred. No specification, corpus,
fixture, expectation, sidecar, or trace status/count/backlink/credit
changed; the formula row stays `deferred`, `tests = []`, and coverage audit
impact is narrative-only. Concurrent ownership is report-only
`repo_metadata_conflict` with no metadata repair. Broad formatting, Clippy,
tests, and all count/hash gates pass. The final source/documentation
re-review has no findings. Independent final quality has no findings, all
nine hard gates PASS, and valid `98/100`. Dedicated B2CP commit
`b146f0f72dceac2233c9d679b7820e264974b227` is complete; the frozen B2C
edge below is the post-commit next owner.

### Task 258B3M2B2B2C Frozen Witness-to-Update Edge

B2C adds exactly one upper edge to the completed lower graph:

```text
formula(0) -> Primary(0/1)
formula(1) -> Primary(5/6)
witness(0) -> Structure(0 functional-update)
Structure(0) -> Structure(1 constructor)
Structure(0) -> Primary(4)
Structure(1) -> Primary(2/3)
```

Task 252 retains sites `51/53/59/62/66/73/75`; Task 254 retains update 69,
constructor 65, members 30/20/24, `FieldUpdate` 68, and all lower
edges/requests; Task 256 owns only equality nodes 55/77; Task-258 base owns
theorem/conclusion 82/80. B2C owns only take/witness 72/71 and the directed
witness edge. Root 58, private roots 60/63/67, containers 56/78,
transparent 70, and all other containers remain unowned.

The structure fingerprint authenticates the lower dependency but is not a
semantic edge. There is no reverse edge, update/member identity, replacement
or result type, functional-copy meaning, witness obligation, proof, goal,
theorem acceptance, active-route credit, or IR edge. The four checker and
five runner tests are a future `test_gap`; implementation remains open while
all four documentation-prerequisite reviews have no findings.

### Task 258B3M2B2B2C Implemented Witness-to-Update Edge

B2C now closes the bounded source/test gaps without changing the frozen family
decomposition. The sole new cross-family edge is
`SourceStatementWitness(0) -> Structure(0)`; Task 254 continues to own the
update/constructor/member/field-update graph, Task 256 continues to own only
the two equality nodes, and all listed subtree containers remain unowned.
There is still no reverse or semantic edge.

The exact four checker and five runner tests pass, including hybrid/order,
ownership, replay, final-clone, near-miss, and empty-semantic matrices. Final
test-sufficiency and implementation reviews have no findings. Trace credit
remains deferred, and broad verification plus final consistency/quality and
commit gates remain pending.

### Task 258B3M2B2B2C Broad Family Verification

Broad format, Clippy, crate, and workspace gates, focused `4/4` and `5/5`,
and sibling `12/12` and `21/21` suites now pass. Fresh counts and hashes match
the implemented inventory, so the sole B2C witness edge and every retained/
excluded family boundary above remain exact. Trace credit is still deferred;
independent final consistency/quality, commit, and post-commit gates remain
pending.

### Task 258B3M2B2B2C Final Family Review Status

Independent final source/documentation consistency and final quality report
**NO FINDINGS**; all nine hard gates PASS with a valid `98/100`. The frozen
family decomposition, evidence, and deferred trace status remain unchanged.
Only cached-diff/staging audit, implementation commit, and post-commit
inventory/fresh-next-task gates remain pending.

### Task 258B3M2B2B3P Frozen Lower Set-Term Reuse

B2C closed at implementation commit
`e8373c683448e524cb98edde83fdf8de83a125cd`; its post-commit worktree is
clean, branch relation is ahead-eight/behind-zero, and the recorded stash is
unchanged. B3P adds no upper payload-family edge. It freezes only this lower
graph in proof context 1:

```text
SetTerm(0 Enumeration, node 40, 90..96)
  -> ordered Primary(2, node 36, 91..92)
  -> ordered Primary(3, node 38, 94..95)
  -> ResultType request(0)
```

Task 252 owns all six primaries and Task 255 owns only set term 0. Tasks
253/254/256/258 are empty and all statement/witness/proof/theorem containers
are unowned. There is no `SourceStatementWitness -> SetTerm(0)` edge in
B3P; that is upper B3A ownership. Likewise there is no result/sethood/
element, existential, proof, goal, theorem, Core, CFG, or VC edge.

The missing contract is closed `design_drift`; future private
explicit-context runner reuse is `source_drift`; two compound runner tests
are `test_gap`. No public schema, active route, checker source/test, or trace
credit changes.

The two arrows above are specifically `EnumerationElement` edges with
ordinals 0 and 1; neither is a generic member or expansion edge. Their
term/target fields, the `ResultType` request, Task-252 primary fingerprint,
and absent application/structure fingerprints are frozen field-for-field.
The two tests authenticate this graph exhaustively and use the three literal
Task-111 legacy hashes.

### Task 258B3M2B2B3P Reviewed Family Status

All four documentation-phase review tracks report **NO FINDINGS**, and the
117-byte/hash, lint `15/14`, library `390/444`, source/test/CLI hash,
exact-scope, diff, and trace-no-op checks pass. The prerequisite family
description and exhaustive test oracle are now frozen; the future private
implementation still owns the bounded `source_drift`/`test_gap`. Final
quality, commit, post-commit, and fresh implementation inventory are pending.

### Task 258B3M2B2B3P Final Family Quality

Final quality reports **NO FINDINGS**, all nine hard gates PASS, and valid
`98/100` (`20/20/15/14/10/10/5/4`). Family evidence is unchanged. Only
stage/commit, post-commit, and fresh implementation inventory are pending.

### Task 258B3M2B2B3P Implemented Lower-Family Reuse

Prerequisite commit `285a1f11c310bb313c4c6b4feae914eb11f74754`
is now implemented by the private explicit-context Task-255 seam. The exact
Task-252 numeral roots feed the two ordered `EnumerationElement` edges in
proof context 1, while Tasks 253/254/256/258 remain empty. The implementation
also authenticates absent application/structure dependencies through a
shared fingerprint-only subprofile, rather than inferring absence from a
missing edge.

This changes no upper payload family: there is still no
`SourceStatementWitness -> SetTerm(0)` edge and no statement, proof, goal,
or semantic row. Test-sufficiency and implementation reviews are
**NO FINDINGS**; B3P `source_drift`/`test_gap` are closed. Upper B3A remains
the next dependency owner. Source/documentation consistency and
documentation/boundary repeats are **NO FINDINGS**; lint-policy `15/14`,
metadata `137`, focused/library/fmt, workspace Clippy/tests, CLI/manifests/
test-list hashes, diff check, and exact 30-file scope PASS. Independent
final quality reports **NO FINDINGS**; all nine hard gates PASS with valid
`98/100` (`20/20/15/14/10/10/5/4`). Only commit, post-commit, and fresh B3A
inventory remain pending.

### Task 258B3M2B2B3A Frozen Upper-Family Edge

B3A owns nodes `{42,43}` and the sole directed
`SourceStatementWitness(0) -> SetTerm(0)` edge. Lower ownership remains
Task 252 `{30,32,36,38,44,46}`, Task 255 `{40}`, Task 256 `{34,48}`, and
Task 258 base `{51,53}`. Nodes
`0..29,31,33,35,37,39,41,45,47,49,50,52,54..56` are unowned.

The full graph is formula `0 -> Primary(0/1)`, formula
`1 -> Primary(4/5)`, witness `0 -> SetTerm(0)`, and
`SetTerm(0) -> Primary(2/3)`. No reverse/cross-owner/semantic edge exists.
All set-shape, label, family-hybrid, and family-order near misses fail closed
without partial publication. B4/B5 and semantic expansion remain deferred.

### Task 258B3M2B2B3A Implemented Upper-Family Edge

The implementation realizes the frozen partition and sole
`SourceStatementWitness(0) -> SetTerm(0)` edge without changing Task-255
production. The set-only fingerprint tuple, exact label/lower provenance,
atomic typed installation, and final revalidation/clone are covered by the
frozen four checker plus five runner tests. Application, structure, and
multi-family hybrids still fail closed; B4/B5 and every semantic expansion
remain deferred. Specification, test-sufficiency, and implementation
reviews report **NO FINDINGS**. The second source/documentation consistency
repeat and final documentation/boundary reread also report
**NO FINDINGS**; parent final verification listed in the crate plans
passes, including exact `39`-file scope. Independent final read-only quality
review reports **NO FINDINGS**. All nine hard gates PASS with no score cap;
the valid score is `98/100` (`20/20/15/14/10/10/5/4`). The stated semantic
and coverage deferrals remain unchanged as residual risk. Only the
dedicated implementation commit, post-commit invariant verification, and
fresh next-task inventory remain pending.

### Task 258B3M2B2B3B Frozen Zero-Edge Family Boundary

Post-B3A inventory distinguishes lower Task-255 zero-edge capability from
upper statement acceptance. B3B freezes exactly one `Enumeration` SetTerm
at node/range `33/95..97`, zero wrappers/generators/type-sites/conditions/
edges, and one `ResultType` request in proof context 1. The upper family adds
witness/take nodes `{35,36}` and only `Witness(0) -> SetTerm(0)`.

Task 252 owns `{27,29,37,39}`, Task 256 `{31,41}`, Task-258 base `{44,46}`,
and Task 255 `{33}`. The empty enumeration contributes no primary child, so
the directed graph contains formula-to-primary edges plus the one
witness-to-set edge and nothing else. Choice, comprehension, `qua`, other
enumeration cardinalities, semantic expansion, B4, and B5 remain separate.
The missing upper contract is `design_drift`; future code/tests are bounded
`source_drift`/`test_gap`, with no blocking authority gap.

### Task 258B3M2B2B3B Implemented Zero-Edge Upper-Family Edge

The implementation realizes Task-252 ownership `{27,29,37,39}`, Task-255
ownership `{33}`, Task-256 ownership `{31,41}`, Task-258 base ownership
`{44,46}`, and B3B ownership `{35,36}`. The complete graph is the two
formula-to-primary pairs plus `Witness(0) -> SetTerm(0)`; the empty
enumeration has no child edge. Only the set-only fingerprint tuple is
accepted, while application/structure hybrids and both B3A/B3B family
orders fail atomically. Choice, comprehension, `qua`, semantic expansion,
remaining B3, B4, and B5 stay deferred.

Post-auth injection and stage-prefix/non-generic-guard assertions close the
last matrix gaps. All test-sufficiency repeats and the final implementation
repeat report **NO FINDINGS** without changing family ownership or semantic
credit.

## Task 258B3M2B2B3C Choice-Witness Family

B3C is the distinct Task-255 choice sibling after B3A/B3B enumerations. It
owns only take/witness nodes `{38,37}` and the edge
`Witness(0) -> SetTerm(0)`. Task 255 retains choice/type nodes `{35,34,33}`,
one `ChoiceTarget` type site, and ordered `ChoiceNonempty`/`ResultType`
requests with zero child edges. Task 252 retains `{27,29,39,41}`, Task 256
`{31,43}`, and Task 258 base `{46,48}`. Comprehension, `qua`, nonemptiness
discharge, generated choice semantics, B4/B5, and proof acceptance remain
separate families.

### Task 258B3M2B2B3C Implemented Choice-Witness Edge

The implementation realizes the frozen ownership exactly: Task-252
`{27,29,39,41}`, Task-255 `{33,34,35}`, Task-256 `{31,43}`, Task-258
`{46,48}`, and B3C `{37,38}`. The choice contributes zero Task-255 child
edges; the only upper edge is `Witness(0) -> SetTerm(0)`. All six
B3A/B3B/B3C installation orders are accepted only as independent exact
families, while application/structure hybrids and generic fallbacks fail
atomically. Choice semantics, comprehension, `qua`, B4/B5, and proof
acceptance remain deferred. Repeated test-sufficiency and implementation
reviews report **NO FINDINGS** after the bounded replay/prefix and B3C-only
route corrections.

### Task 258B3M2B2B3D Frozen Qua-Witness Edge

B3D is the smallest remaining Task-255 set-family witness: one `Qua` term,
one term-owned `QuaTarget` builtin-set site, one
`QuaBase -> Primary(2)` edge, ordered unresolved `QuaWidening`/`ResultType`,
and one upper witness-to-SetTerm edge. Condition-free comprehension follows
because it adds a generator/sethood row. The B3D edge is transport only;
inheritance/cluster widening, overload/coercion, result typing, proof
acceptance, comprehension, B4/B5, and active credit remain separate owners.

### Task 258B3M2B2B3D Implemented Qua-Witness Edge

The private exact route now realizes the frozen graph: Task-252 owns
`{28,30,34,41,43}`, Task-255 `{35,36,37}`, Task-256 `{32,45}`,
Task-258 `{48,50}`, and B3D `{39,40}`. Task 255 retains the sole
`QuaBase -> Primary(2)` lower edge and unresolved
`QuaWidening`/`ResultType` requests; B3D adds only
`Witness(0) -> SetTerm(0)`.

All B3A/B3B/B3C/B3D pairings and 24 family orders, subtree exclusions, and
the exact `32/70/44/72/62/21` matrices are covered by the frozen tests.
Test-sufficiency and independent implementation reviews report
**NO FINDINGS**. No widening, type, proof/fact, Core/CFG/VC,
comprehension, B4/B5, or active-credit edge is introduced;
source/documentation consistency and boundary review also report
**NO FINDINGS** after the family-order/qua-edge wording corrections. Full
package/workspace/formatting/Clippy/CLI/count-hash verification passes.
Independent final read-only quality review reports **NO FINDINGS**; all nine
hard gates PASS with no cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). Only exact staging/cached-diff review,
implementation commit, and post-commit/fresh-next-task gates remain pending.

### Task 258B3M2B2B3E Frozen Comprehension-Witness Edge

B3E is the last uncovered exact Task-255 set-family witness after
enumeration, choice, and `qua`. It freezes one condition-free independent
comprehension with one generator/type site, one
`ComprehensionMapper -> Primary(2)` edge, ordered
`GeneratorSethood`/`ResultType` requests, and one
`witness -> SetTerm(0)` edge. Task-255 owns `{16,40,41,43}`; generator
segment `42` remains unowned. All five B3A-E families remain independent
across 120 orders. Binding/capture, sethood discharge, conditions, semantics,
B4/B5, and coverage credit remain deferred.

### Task 258B3M2B2B3E Implemented Comprehension-Witness Edge

The private exact route realizes the frozen graph without changing the
Task-255 producer. Task-252 owns `{32,34,38,47,49}`, Task-255
`{16,40,41,43}`, Task-256 `{36,51}`, Task-258 `{54,56}`, and B3E
`{45,46}`; generator segment `42` remains unowned. Task 255 retains
`ComprehensionMapper -> Primary(2)` and unresolved
`GeneratorSethood`/`ResultType`; B3E adds only
`Witness(0) -> SetTerm(0)`.

All five-family pairings and 120 orders, complete-subtree exclusions,
same-provenance coherent Task-255 near misses, and exact
`32/70/53/72/62/21` matrices are covered. Independent test-sufficiency and
implementation reviews report **NO FINDINGS**. Binding/capture,
conditioned/multiple/nested/generator-reference semantics, sethood/type/
proof/fact/Core/CFG/VC behavior, B4/B5, and coverage credit remain deferred.

Final source/documentation consistency reports **NO FINDINGS** after the
three bounded design corrections. Complete verification PASSes, and
independent final quality reports **NO FINDINGS**, all nine hard gates PASS,
valid `100/100` with no cap. Staging and post-commit gates subsequently
closed in implementation commit
`e4479691db3b0a8785bb16e94d386bd71a394274`; fresh inventory selected
Task 258B4A.

## Task 258B4 Composite-Root Decomposition

The composite-root umbrella is split by already public lower consumer:

1. B4A consumes Task-257B1 explicit-universal composition.
2. B4B retains Task-257B2 connective/grouping roots.
3. B4C retains Task-257B3 restricted, existential, and nested roots.
4. B5 retains broader imported/outer/inner visibility.

B4A adds only the upper `Composite(0)` statement/candidate association over
the private 80-byte/double-LF route. Its zero input-fact profile keeps
explicit binder/type/use transport in Task 257. It neither copies lower rows
nor converts lower `UnassignedStatement` ownership into semantic acceptance.

Repeated read-only documentation review reports **NO FINDINGS**. Independent
final quality passes all nine hard gates with no cap at valid `100/100`;
only staging, commit, and post-commit inventory remain.

## Task 258B4A Implemented Composite-Root Edge

B4A closes only the syntax-free upper edge from theorem statement 0 and
candidate 0 to the existing Task-257B1 `Composite(0)`. Task 252 retains
primary/reference ownership, Task 256 retains atomic equality leaves, Task
257 retains the explicit binder and composite/composition graph, and Task
258 retains statement ownership. Exact lower owned-site/range checks and a
rootless lower typed arena prevent coherent cross-family substitution
without transferring ownership. Truth, binder guard discharge, facts,
acceptance, proof semantics, B4B/B4C, and B5 remain deferred.

## Task 258B4B Frozen Composite-Root Edge

B4B is the second B4 node and consumes only Task-257B2. The unchanged lower
graph is Task 252 `16/0/16`, Task 256
`8/0/0/0/0/0/0/16/16`, Task 257 `8/6/1/1/1/7/9`, and
Task-257B2 `8/0`; its explicit binder is unused and its one root remains
`UnassignedStatement`. Task 258 adds only statement 0 and candidate 0 to
`Composite(0)`, using one owner/context, zero input facts, and no edge to an
inner connective, wrapper, equality, or numeral.

The private 167-byte route is isolated from the active 166-byte lower-only
fixture and B4A's 80-byte route. B4A/B4B profile hybrids fail atomically.
B4C continues to own Task-257B3 restricted/existential/nested roots, and B5
continues to own broader visibility. Connective/repetition semantics, truth,
facts, acceptance, and proof remain deferred.

## Task 258B4C Frozen Composite-Root Edge

B4C consumes only the existing Task-257B3 restricted-universal,
existential, nested-quantifier, and implicit-reserve graph. Its exact lower
profiles are binding `4/4/0`, Task 252 `6/6/0`, Task 256
`3/0/0/0/0/0/0/6/6`, Task 257 `3/0/1/3/3/2/6`, and Task-257B3
composition `3/6`. The 24 lower-owned Surface sites are
`{9,17,22,32,33,36,37,38,39,41,43,44,45,46,47,48,50,52,53,55,57,58,59,60}`;
composite root 60 remains `UnassignedStatement`.

Task 258 adds only theorem node 62 and the statement/candidate edges to
`Composite(0)`. Context 0 sees reserved binding `[0]`, but reserve supplies
a binder/type default rather than a prior statement fact, so the input-fact
table remains empty. No edge targets an inner equality, binder segment, or
reference. The remaining 41 Surface nodes stay unowned.

The private 139-byte/double-LF route is distinct from the active
138-byte/lower-only Task-257B3 source. A separate lower selector prerequisite
must admit exactly those one- and two-LF forms and reject zero or three LF
bytes before B4C implementation. Upper dispatch is matched only as
B1/B4A, B2/B4B, and B3/B4C; every hybrid fails atomically. Quantifier
truth, restriction discharge, witness semantics, capture, implicit theorem
closure, facts, theorem acceptance, proof, Core/CFG/VC, and B5 remain
deferred.

## Task 258B5C Active Negative Decomposition

B5C is not a fourth checker reference profile. It consists of two resolver
failures only: a label declared at proof scope `[0,0]` is not visible from
the enclosing `[0]` scope and is not visible from sibling `[0,1]`. Each
route contains one private/local-only proof-step projection and one
unqualified proof-or-theorem candidate, then terminates with an exact
`UnresolvedLabelRef`.

The work is decomposed into documentation, resolver R-032A structural arena,
resolver R-032B proof-label collection, and active declaration-symbol
fixtures/runner/trace, each in its own commit. R-032A establishes only
same-index structural provenance; R-032B alone establishes proof scopes,
module-global one-based completion ordinals, canonical `proof-step-v1`
origins, and candidates from exact `CompactStatement`/
`ConclusionStatement` plus justification/reference-chain forms. There is no checker
base/reference pair, local or imported citation target, label/citation row,
typed installation, or final clone. Cross-family edges to structure
construction, selector access, functional/field update, Tasks 252/253,
ancestor B5A, imported B5B, and B1 are all empty.
Qualified/grouped/bulk citations, public diagnostic codes, proof discharge,
facts, acceptance, and downstream IR remain deferred.

The lower resolver family is now explicitly closed to the exact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock -> CompactStatement/ConclusionStatement`
allowlist, compact proposition-label inspection, direct statement proof/
justification children, and the sole simple-reference identifier chain.
Root/CompilationUnit exact-one structural children and ItemList direct-normal
theorem scanning are mandatory. Every other subtree is
no-row/no-ordinal/no-descent, with positive upper/lower edges and negative
missing/additional/wrong, direct Root/Compilation relocation,
`VisibleItem` wrapping, other forbidden-relocation, and mixed-list tests
below checker ownership.

Runner provenance authentication of env/module, derived namespace, exact
one id-0 LocalSource record/source id, and every projection field remains a
separate input-only family. Its complete independent mutation matrix cannot
create a checker payload or confinement result.

## Task 259 Frozen Predicate-Definition Decomposition

Task 259 is decomposed as one predicate-definition row, two ordered parameter
rows, one guard row, one symmetry-property row, and one correctness-condition
row. The exact lower graph is Task 249 `2/2/0`, Task 252 `4/4/0`, and
Task 256 `2/0/0/0/0/0/0/4/4`; the existing Task-248 handoff becomes available
only after the independent one-block/two-parameter profile extension.
Tasks 253--255, 257, and 258 contribute no row or fingerprint.

The definition points to the equality definiens, each parameter points to one
`BindingId` and one `SourceTypeApplicationId`, and the guard points to the
other equality. The property points only to its owner, source site, source
order, `Symmetry` kind, and explicit justification anchor. The correctness
row points to exactly one `Pending`
`InitialObligationKind::PredicatePropertyCorrectness`. Its assumptions are
empty and its goal/provenance are deterministic opaque identities.

The runner authenticates the property as the direct normal later sibling of
the predicate in the same definition block. The resolver's generic
Attribute/Attribute property projection is not consumed as semantic
evidence. Task 259 does not descend into or interpret the computation
justification; Task 272 retains that future proof/justification ownership.
Task 260 separately retains the mixed predicate-plus-functor gap.

## Task 260 Functor-Definition Family

Task 260 is a separate five-table source family: definitions, shared context
parameters, shared guards, definientia, and correctness associations. A
definiens points to exactly one existing Task-252/253/254/255/256 lower root;
it never copies the lower row. The active source installs two definitions and
two definientia, one primary `equals` target and one atomic-formula `means`
target.

The family appends `FunctorExistence` and `FunctorUniqueness` only for the
explicit means correctness clauses. It stores no semantic goal composition,
proof, acceptance, fact, or VC. Task 259 and Task 260 do not cross-fingerprint
or reinterpret one another and are mutually exclusive in Task 260; mixed
coexistence remains a separate deferred owner.

## Task 249R Definition-Return Lower Family

Task 249 remains a binding-owned application/expression/argument family with
profile `2/2/0` for the Task-260 source. Task 249R is a distinct owner-link
family inside the same immutable handoff: two functor-definition owner sites
point to appended bare-set expression roots 2/3, producing combined
`2/4/0/2`. Task 260 alone consumes return IDs 0/1. No binding, normalized type,
semantic association, goal, fact, obligation, proof, or VC row crosses this
lower boundary.

## Task 249M Mode-RHS Lower Family

Task 249M is a distinct standalone owner-link family in the existing source-
type handoff. It appends bare-set expression root 2 and links mode-definition
owner node 49 through `SourceTypeModeRhsId(0)`, producing `2/3/0/0/1` without
a third binding application or a definition-return row. Task 262 is its sole
consumer; request, evidence, expansion, acceptance, fact, proof, and VC rows
do not cross this lower boundary.

## Task 249M Active Family Inventory

The distinct standalone owner-link family is now implemented exactly as
frozen. The active handoff has one `SourceTypeModeRhsId(0)` row and root 2,
remains mutually exclusive with the Task-249R definition-return family, and
still publishes no request, evidence, semantic, fact, proof, IR, or VC row.
## Task 249S Structure-Member Type Lower Family

Task 249S is a distinct standalone owner-link family in the existing immutable
source-type handoff. Four declaration-member owner sites point to four bare-set
expression roots, producing `0/4/0/0/0/4` without a binding application,
definition-return row, or mode-RHS row. Task 263 is its sole consumer. Member
kind and identity, structure parent/root/path/view, inheritance coverage,
constructor/selector declarations, coherence, requests, evidence, semantics,
facts, proofs, IR, and VC rows do not cross this lower boundary.

## Task 249S Active Lower-Family Result

The frozen standalone member-type family is now active exactly as
applications/expressions/arguments/definition-returns/mode-RHS/members
`0/4/0/0/0/4`. It owns only four declaration-member-to-type-root links.
Task 263 still owns structure/member identity association, classification,
inheritance, coverage, constructors/selectors, coherence requests, and the
runner/corpus consumer; no semantic family crossed this boundary.

## Task 264 Property-Implementation Family

Task 264 is one five-table family: implementations, parameters, referenced
property targets, definientia, and correctness. The target row owns resolver
property provenance plus a Task-249PI declared-return row; it does not become a
new resolver definition. Means consumes Task-256 and appends existence plus
uniqueness; equals consumes Task-254 and appends none. There is intentionally
no guard table because the source grammar uses the defining-mode parameter
rather than an ad-hoc `assume`.

Task-252 `it` remains a term occurrence, Task-254 remains the equals selector
owner, Task-256 remains the means formula owner, and Task 264 only associates
those immutable lower IDs. Coherence/overlap, proof, acceptance, fact, and VC
families remain separate. Task 259 and Task 264 do not cross-fingerprint or
co-install in this bounded task.

## Task 249PI Lower Composition Family

Task 249PI does not add a payload family. It composes existing Task-249
application/expression ownership and Task-249S member/expression ownership in
the exact `1/3/0/0/0/2` property-source profile. It neither classifies field
versus property nor associates member row 1 with the property target; Task 264
does that from resolver authority. Definition returns, mode RHS, predicates,
functors, property semantics, obligations, facts, proof, acceptance, and IR
families remain mutually isolated.

## Task 249PI Implemented Composition Boundary

The exact application/member composition is now executable in the existing
source-type family only. It adds no family, semantic payload, obligation, or
cross-family ownership; Task 264 remains the first property consumer.

## Task 264 Implemented Property-Implementation Family

The frozen five-table family is now executable with exact means/equals
cardinalities `1/1/1/1/2` and `1/1/1/1/0`. It authenticates the Task-248P,
249PI, 252, 254, and 256 handoffs, associates the resolver property with the
declared return row, and appends only pending property existence/uniqueness
rows for means. Typed/final installation remains atomic and mutually exclusive
from Tasks 259--263. No goal/guard composition, proof status, acceptance,
fact/property value, overlap/coherence, Core, CFG, or VC family was added.

## Task 269A Proof-Local Binding Family

Task 269A adds one declaration-to-binding association family over immutable
Task-258B3N statement, witness, and primary-term handoffs. Its sole row links
witness 0/name 0/RHS primary 2 to newly dense binding 1 and owns the exact
base-to-final `BindingEnv` transition. It does not re-own the name/witness/RHS
arena nodes and does not mutate any lower fingerprint.

This family is distinct from primary-term use references: it records a
definition site and future visibility, not a later use or expansion. It is
also distinct from Task-272 witness typing/goal effects and all fact, proof,
acceptance, IR, and VC families. Task 269B+ owns later-use and capture replay.

## Task 269A Active Proof-Local Binding Family

The one-row definition-site family is implemented and preserved through typed
and final ownership. Its private dormant consumer and eight unit tests add no
active trace credit or later-use edge. Task 269B+, 270, 271, and 272 ownership
is unchanged.

## Task 269B frozen B3M1 family increment

The existing declaration-to-binding family accepts a second exact lower
profile, not a new payload family. Its single row still links named witness 0,
name 0, and primary 2 to binding 1; the sibling unnamed witness remains solely
in the lower witness table. No later-use edge, capture set, type/goal/fact/
proof family, or active coverage owner is introduced.

## Task 269B active B3M1 family increment

The existing family now accepts the second exact profile with the same one-row
shape. Direct final-environment and context assertions prove that the unnamed
sibling remains lower-only. No payload family, later-use edge, capture, type,
goal, fact, proof, or coverage owner was added.

## Task 269CP isolated proof-`let` lower family

This prerequisite adds no checker payload family. It freezes one
runner-private source/Surface/resolver projection whose selector authenticates
exact theorem, proof, let, segment, name, and bare-set Surface nodes but whose
output retains source/module identities, source and Surface fingerprints,
the theorem symbol/definition/contribution, source ordinal, role-specific
ranges, and local provenance. Future Task 269C owns a separate checker
let-binding family. Named-witness A/B,
later-use/capture, source-type admission, goal/proof semantics, and all active
coverage remain disjoint.

## Task 269C isolated proof-`let` binding family

The new checker sibling owns one `LetBinding` row and the exact `BindingEnv`
transition only. It consumes Task-269CP provenance without importing syntax,
retains a missing type site, and is one-shot preserved by Typed/final owners.
It is disjoint from named-witness A/B, source-type application, actual use/
capture, formula/goal/fact/proof/obligation families, and active coverage.

The family is now implemented with exactly one declaration row and no sibling
payload. Typed/final replay and the dormant runner preserve that decomposition;
the separate source-type prerequisite remains unimplemented.

## Task 269CT Proof-`let` Source-Type Composition Family

The separate prerequisite is now frozen as one exact composite of the
unchanged Task-269C binding snapshot, a typed `BindingEnv` overlay, and the
Task-249 source-type family. It owns two bare builtin-`set` applications and no
new binding, use/capture, assumption/guard, goal, fact, proof, obligation, IR,
or active-coverage family. Task 269C and generic Task 249 remain unchanged.

## Task 269CT Implemented Family

The composite family is now implemented at exactly the frozen syntax-free
boundary: immutable Task-269C dependency, separate typed binding overlay,
two bare builtin-`set` type applications, and three source-preserved nodes.
Generic Task 249 rejects the proof-local `LetBinding`; no semantic family or
active coverage owner was added.

## Task 269GP Proof-`given` Lower Family

Task 269 remains open. Runner-private 269GP transports syntax only. The
canonical Chapter-4/16 scope contradiction blocks only the direct
binding/type consumers 269G and 269GT pending human reconciliation. Later
`given` condition/escape semantics remain separately deferred without a new
blocker classification. This lower slice changes no Task-269 checker family
or Task-270 dependency and grants no active credit.

The implemented private lower row closes only its syntax projection gap and
adds no checker payload-family member.

## Task 269GS Family Readiness

The human-approved block-lifetime rule removes the `given` binding/type family
`spec_gap`. It creates no payload in this documentation task. Task 269G may now
freeze a binding-only consumer of the existing 269GP syntax row; Task 269GT
retains later type admission. Condition, label-fact, goal, proof, and
obligation families remain excluded.

## Task 269G Payload Delta

The only new payload family is one immutable lexical binding transaction:
authenticated lower provenance, base/final `BindingEnv` snapshots and
fingerprints, and one dense binding row. Type, condition/label fact, theorem
fact, goal, proof, obligation, acceptance, IR, and VC payloads remain absent.

The family is now implemented as exactly one dense Given binding row plus the
authenticated lower provenance and base/final environments. Typed/final
replay and the dormant runner preserve that decomposition; Task 269GT remains
the only owner allowed to add the separately frozen source-type payload.

## Task 269GT Payload Delta

The sole new family is one immutable composite containing the unchanged Given
binding dependency, its fingerprint, a typed `2/2/0` binding snapshot, one
`2/2/0/0/0/0` builtin-`set` source-type handoff, and their fingerprints. No
condition, label/fact, type guard/assumption, goal, proof, obligation,
acceptance, Core, CFG, or VC payload is part of this family.

### Task 269GT implemented payload delta

One immutable Given-type composite now owns the Task-269G dependency, exact upgraded binding environment, exact `2/2/0/0/0/0` source-type handoff, three fingerprints, and the separate three-node arena. Direct Given-binding, generic type, Let, condition/fact, assumption/guard, proof, obligation, acceptance, and downstream semantic payload families remain absent.

## Task 269GUP Payload Delta

One sibling-specific Given binding handoff owns exact lower provenance, base
and final fingerprints, one declaration row, and the `2/2/0` environment.
Source type is GUPT; terms/references and final composition are GU. Typed/final,
capture, formula/condition/fact, goals/obligations, proof acceptance, and every
downstream semantic family remain absent.
### Task 269GUP implemented binding profile

The frozen six-file transaction and its exact four checker/four runner tests are implemented. Libraries measure `502/564`; checker/runner production is `30/172531` and `37/74826`, with unchanged path hashes and content hashes `e0342952a01a0b379cf7b06ad243cd40a1656e940480196323cf43fbe7d8f7c5` / `8fe7c8c0b7e855e5113f3830873e133f42c8048a3272055e2fddd5ebd9cbb1bc`.

This closes only dormant private lexical-binding evidence and grants zero active corpus, trace, type, term/use, condition/fact, goal/proof, obligation, diagnostic, or CLI credit. Task 269GUPT is next; Task 269GU, capture, and Task 270 remain deferred.

## Task 269GUPT Frozen Payload Delta

The only new family is one `SourceProofLocalGivenUseTypeHandoff`: unchanged GUP binding dependency, copied typed environment, exact `2/2/0/0/0/0` builtin-`set` source-type handoff, three fingerprints, and a distinct three-node arena. Typed/Resolved own that composite atomically. Term/reference/final-use composition belongs to Task 269GU; condition/fact, capture, goal/obligation, proof/acceptance, and downstream IR remain absent.

### Task 269GUPT implemented payload

The frozen composite is now implemented exactly and remains the sole new
payload family. Typed/Resolved ownership is atomic and semantically empty;
Task 269GU still owns the later term/reference composition.

## Task 269GU Frozen Payload Delta

The only new family is `SourceProofLocalGivenUseTermHandoff`: an owned exact
GUPT dependency and fingerprint plus one exact `2/2/0` primary-term handoff
and fingerprint over a distinct six-node arena. Direct GUPT, binding, type,
or term owners are absent. Formula/equality, statement, condition/fact,
capture, goal/obligation, proof/acceptance, and downstream IR remain absent.

### Task 269GU implemented payload

The frozen GUPT-dependent `2/2/0` term/reference composite is implemented as
the sole new family. Typed/Resolved ownership is atomic and semantically empty;
condition/descendant use, capture/export, formulas, facts, and proof owners stay
deferred.

### Task 269GCP Condition-profile Decomposition

GCP is the source-order-minimal lower prerequisite for the still-open Task-269
condition-use edge. It authenticates syntax and theorem provenance only. GC,
GCT, and GCU separately own binding, written type, and the two condition term
references; descendant use, first-order `set` capture replay, and Task 270
remain later graph nodes.

### Task 269GCP implemented lower payload

The runner-private immutable lower row now authenticates the exact source,
Surface, shells, theorem provenance, declaration sites, and debug replay. It
adds no checker payload family or semantic owner; GC, GCT, and GCU remain the
separate binding, type, and condition-use nodes.
