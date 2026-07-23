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
| 249 | Type head/application payloads: builtin, local/imported mode and structure radix, positional/bracket type arguments, term arguments, and written type-site identity. Specs 03, 05, 07, and 08; MC-G014/MC-G016. | Task 248; resolver symbol/provenance data; Task-10 type-elaboration consumer. | Produces type inputs only; no expansion, inhabitation, subtyping, or evidence result is fabricated. |
| 250 | Attribute-chain payloads: polarity, arguments, qualification/owner identity, local/imported provenance, order, and attributed type-site association. Specs 03, 06, and 17; MC-G014/MC-G020. | Tasks 248-249; Task-10 consumer reuses exact argument-bearing, qualified, imported, and non-empty boundaries. | No admissibility, existential evidence, closure fact, or attribute truth is synthesized. |
| 251 | Evidence-query requests and upstream dependency-fact inputs for mode expansion, structure base shape/constructor witness, attributed-type inhabitation, sethood/non-emptiness, inheritance, and coercion viability, including `ExistentialGateInput` request identity and dependency-fact references. Specs 03, 05-08, 13, 17, and 19; MC-G016/MC-G018/MC-G026. | Tasks 248-250 and canonical imported summaries when available; Task-10 consumer distinguishes request, missing, rejected, and supplied input. | It owns request/site/provenance/reference transport only. Accepted evidence, theorem results, and artifact status remain external inputs. |
| 252 | Primary terms: variable/constant references, `it`, numerals, and transparent parentheses with binding, role, and numeric-type request payloads. Spec 13.1; MC-G017/MC-G020. | Tasks 248-251; exact Task-10 term consumers. | No arbitrary application, structure/set term, formula, or numeric theorem fact. |
| 253 | Functor and inline-functor applications, including ordered arguments, signature/result requests, imported provenance, and definition-local actuals. Specs 10 and 13.2; MC-G017/MC-G020. | Tasks 249-252 and resolver signature candidates; Task-10 imported and local application consumers. | No overload winner, implicit definition proof, or result type without supplied evidence. |
| 254 | Structure constructor, selector, and update terms with root/member/view identity, ordered fields, inheritance-path requests, and result-type requests. Specs 05 and 13.3; MC-G017/MC-G018. | Tasks 249-253; later Task 263 supplies source definition payloads; Task-10 structure-term consumer. | No constructor property arguments, invented field coverage, upcast winner, or structure evidence. |
| 255 | Set enumeration/comprehension, choice, and `qua` terms with generator scope, predicate/body links, sethood requests, written target type, and explicit conversion intent. Specs 07.8.1, 08.2, and 13.4-13.6; MC-G017/MC-G018. | Tasks 248-254; Task-10 set/choice/`qua` consumers. | No missing sethood proof, narrowing proof, implicit widening path, or comprehension fact is fabricated. |
| 256 | Atomic formulas: predicate applications, equality/inequality, membership, type assertions, and attribute assertions with complete term/type/attribute links and expected-input requests. Specs 09 and 14.2/14.5; MC-G017/MC-G020. | Tasks 249-255; Task-10 exact atomic-formula consumers. | No truth, theorem acceptance, inequality proof, or assertion fact without checker evidence. |
| 257 | Composite formulas and binders: constants, negation, binary connectives, quantified variables, child graph, contexts, roles, and source order. Specs 04.5 and 14.3-14.4; MC-G011/MC-G017/MC-G020. | Tasks 248-256; Task-10 connective/quantifier consumer. | No flattening that loses child identity, implicit closure, truth value, or theorem status. |
| 258 | General theorem-owner and statement-semantic shells, assumptions, conclusions, witnesses, labels/citations as resolver identities, local contexts, visibility-scoped input facts, and candidate fact inputs. Specs 15 and 16; MC-G019/MC-G020. | Tasks 248-257; resolver label facts; prepared `MT10-FS` consumer. | Records input/candidate assumptions and facts only; no verified premise publication, checked theorem fact, discharge, theorem acceptance, or proof closure is inferred. |
| 259 | Predicate-definition payloads: parameters, guards, definiens graph, properties/correctness-condition identity, `InitialObligationId`, source anchor input, and declaration provenance. Specs 09 and 16.6. | Tasks 248-258; Task-10 definition consumer. | No recursive unfolding, property proof, obligation discharge, `VcId`, accepted obligation, overload selection, or axiom publication. |
| 260 | Functor-definition payloads: `equals`/`means`, parameters, guards, result type, definiens, properties/correctness-condition identity, `InitialObligationId`, source anchor input, and declaration provenance. Specs 10 and 16.6. | Tasks 248-259; Task-10 definition consumer. | No existence/uniqueness proof, obligation discharge, `VcId`, recursive unfolding, accepted result, or overload winner. |
| 261 | Attribute-definition payloads: subject/parameters, positive or negative definiens, guards, radix/qualification, and correctness obligation requests. Specs 06, 09, and 16.6. | Tasks 248-260; Task-10 attribute-definition consumer. | No attribute truth, cluster fact, existential evidence, accepted proof, or redefinition selection. |
| 262 | Mode-definition payloads: parameters, mode application, expansion/RHS, definiens, sethood/existence obligation requests, and declaration context. Specs 07 and 16.6. | Tasks 248-261; Task-10 mode-definition consumer. | Property implementations are Task 264; no accepted existence, expansion fact, or registration activation. |
| 263 | Structure-definition/inheritance payloads: parameters, parents, root+path/view member identity, field coverage/types, constructor/selector declarations, and coherence obligation requests. Specs 05, 13.3, 16.6, and 19.2.2. | Tasks 248-262; Task-10 structure-definition consumer. | No property-valued constructor argument, inferred member identity, accepted coherence, or chosen upcast. |
| 264 | Mode property-implementation payloads: owner/property identity, local parameters, `means`/`equals` definiens, overlap domain, correctness-condition identity, `InitialObligationId`, and source anchor input for existence/uniqueness/coherence. Specs 07.4.1/07.8.2 and 16.6. | Parser Task 48, Tasks 248-263, and Task-10 property consumer. | No parser grammar ownership, overlap acceptance, constructor property source, `VcId`, obligation discharge, or proof acceptance. |
| 269 | Proof-local declaration/binding payloads for `let`, `set`, `given`, `consider`, `take`, and other local introductions, including context transitions and source-order closure. Specs 04, 15.2-15.4, and 16.4. | Tasks 248-258; prepared `MT10-FS` consumer. | No inline abbreviation expansion, reconsider coercion, proof search, or accepted witness. |
| 270 | Proof-local `deffunc`/`defpred` closure payloads: formal identities, captured free variables, body graph, guard, substitution request, and capture-avoidance provenance. Specs 04.4.3, 10.11.3, 15.2.3-15.2.4; architecture 16. | Tasks 248-269; prepared `MT10-AS` capture consumer for the existing advanced-semantics trace row. The same producer may also supply proof-local declaration data to `MT10-FS`, but that does not transfer the trace-row ownership. | No substitution result without explicit replay evidence, no capture repair in the runner, and no accepted local theorem. |
| 271 | `reconsider` payloads: bindings, source/target types, written-or-omitted justification intent, widening/narrowing request, and proof-free evidence query. Specs 04.4.2, 08.2, 15.5.1, and 19.3.2. | Parser Task 47, Tasks 248-258 and 269; prepared `MT10-FS` consumer for proof-local families and `MT10-AS` consumer for the existing omitted-justification advanced-semantics fixture. | No omitted proof is accepted, no narrowing evidence is invented, and parser expectation drift is not repaired here. |
| 272 | Non-Task-180 proof skeleton and justification payloads: nested proof nodes, thesis/terminal goals, citations, local paths, case/suppose/now structure, and explicit pending/blocked states. Specs 15.6/15.8 and 16.3-16.5. | Tasks 248-271; resolver label identities; prepared `MT10-FS` consumer plus the `MT10-AS` omitted-`reconsider` negative consumer where explicit pending/blocked intent must be asserted. | Task-180 exact tables remain Tasks 266-268; no proof search, implicit closure, theorem acceptance, discharge, Core, or VC. |
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
