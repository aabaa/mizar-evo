# STEP 5 ソースペイロードファミリ分解

> 正本言語: 英語。英語正本:
> [../en/payload_family_decomposition.md](../en/payload_family_decomposition.md)。

本書は checker Task 247 の受理済み成果物である。残るソース由来 checker
ペイロードファミリを棚卸しし、各ファミリを境界付き checker producer task、
準備済み `mizar-test` Task-10 consumer increment、または明示的外部 gate に
割り当てる。本書は task 所有権と依存関係だけの authority であり、言語意味論、
ソースコード、fixture、expectation、trace status、test list、coverage credit は
変更しない。

## Authority と開始baseline

棚卸しは次のrepository authority orderに従う。

1. `doc/spec/en/`;
2. 既存 `.miz` source;
3. `tests/coverage/spec_trace.toml`;
4. 既存 expectation sidecar;
5. checker/consumer design document;
6. 非規範的な棚卸し証拠としての現在の checker/runner source。

Task-247 のread-only開始baselineはcleanな
`b0930a0c44a4f306d1a1ef2f9e66b4a7bd7f5cf6` であった。active runnerは
parse 96、declaration 4、type elaboration 188。repository planは403 cases /
368 requirements、type elaborationは236/224、pass/failは219/184であった。
`mizar-test` はunit test 272、production 17 paths / 19,803 linesであった。
Task 247はこれらと次のoracleを維持しなければならない。

- CLI SHA-256: plan
  `0915fed1465c86f4b4d0420a35703fe93aed0cbb23b7304abff927195b4f5758`,
  parse `57d0fba9be95644890b80bfa4ec2cd992e47bb8ad4b67c130f5194ea73aa0273`,
  declaration
  `08b00a9f6fe70d94fe2c1b2bdebbdb5603bcee39bf3ceb460abe53f403bba7b5`,
  type `1dadbeabb219f5853c713ad53aa1cc7cd720a0e80abd7f882e9e0a5ea7802625`;
- test-list SHA-256: raw
  `5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e`,
  normalized
  `c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`;
- `mizar-test` production SHA-256: path
  `b36d96fed3207b415c95de27be11ade57654c6573a2f0637aa2d0a3d56aca01d`,
  content
  `5f9e716169964a861b71576957c05e2dc2538b5e0ff9d1025ef51a4bea6aa306`。

## Tasks 248-264/269-279 共通producer contract

各producer taskは、1 nonempty logical taskかつ1 commitである。編集前に正確な
spec section、source family、syntax-free input payload、checker API consumer、
`mizar-test` Task-10 consumer、visibility、negative boundary、tests、trace row、
coverage impact、exit criteriaを固定する。

以下の行で狭めない限り、各producer taskは次を満たす。

- 実 `.miz` AST inspectionとsource-role extractionは`mizar-test`に保持する;
- validated、syntax-free、source-orderedなidentity/range/provenance/recovery
  state/semantic inputだけを`mizar-checker`へ渡す;
- runnerでchecker結果を再構築せず、既存checker table/algorithmをconsumerにする;
- 実装familyを適用可能な`TypedAst`/`ResolvedTypedAst` tableまでtransactionally
  projectし、source identity/range/provenance/recovery/predecessor linkを保存して
  Task-10 consumerが最終checker handoffをassertする。未消費input DTOでproducer
  taskをcompleteにしない;
- missing/duplicate/reordered/recovered/cross-module/stale-provenance/
  wrong-role/partial payloadをfail closedにする;
- checker unit/corruption/determinism coverageと、当該familyに必要な最小の
  spec-derived real-source consumer coverageを追加する;
- canonical specificationが明示的に意味変更を許可しない限り既存expectationを
  維持し、新規test-first caseは既存canonical requirementから直接導出する;
- 後続taskが実装した正確なexecutable sliceに限ってdeferred trace rowまたは
  coverage creditを変更し、Task 247がownerを命名しただけでは変更しない。

全producer taskで、checker内raw-syntax inspection、parser/resolver ownership
takeover、proof search/acceptance、fact/evidenceの捏造、`CoreIr`/
`ControlFlowIr`/VC構築、artifact schemaやpublic diagnostic codeの捏造、広範な
expectation rebaseline、Steps 6/7昇格を禁止する。

## 受理済みproducer graph

以下のtask IDは`mizar-checker`に属する。既存joint Task 265とTasks 266-268は
完了済みの意味を保持し、意図的に再利用しない。

| Task | 境界付きproducerとcanonical authority | 依存と準備済みconsumer | exit boundary |
|---|---|---|---|
| 248 | source item、declaration site、local scope、ordinal、reserve/default、`BindingEnv` context payload。Specs 04/11/12/15、MC-G011/016。 | 既存resolver identityと`mizar-test` Task 10。consumerはsource order、shadowing、recovery、declaration/binding identityを証明する。 | type result、RHS評価、proof context、global name resolution再構築なし。 |
| 249 | builtin/local/imported mode/structure radix、positional/bracket type argument、term argument、written type-site identityを含むtype head/application payload。Specs 03/05/07/08、MC-G014/016。 | Task 248、resolver symbol/provenance、Task-10 type-elaboration consumer。 | type inputのみ。expansion、inhabitation、subtyping、evidence resultを捏造しない。 |
| 250 | polarity、argument、qualification/owner identity、local/imported provenance、order、attributed type-site linkを含むattribute-chain payload。Specs 03/06/17、MC-G014/020。 | Tasks 248-249。argument-bearing/qualified/imported/non-emptyの正確なboundaryをTask-10 consumerが再利用する。 | admissibility、existential evidence、closure fact、attribute truthなし。 |
| 251 | mode expansion、structure base shape/constructor witness、attributed-type inhabitation、sethood/non-emptiness、inheritance、coercion viabilityのevidence-query requestとupstream dependency-fact input。`ExistentialGateInput` request identityとdependency-fact referenceを含む。Specs 03/05-08/13/17/19、MC-G016/018/026。 | Tasks 248-250と利用可能なcanonical imported summary。Task-10 consumerはrequest/missing/rejected/supplied inputを区別する。 | request/site/provenance/reference transportだけを所有する。accepted evidence、theorem result、artifact statusは外部input。 |
| 252 | variable/constant reference、`it`、numeral、transparent parenthesisとbinding/role/numeric-type request。Spec 13.1、MC-G017/020。 | Tasks 248-251、正確なTask-10 term consumer。 | arbitrary application、structure/set term、formula、numeric theorem factなし。 |
| 253 | ordered argument、signature/result request、imported provenance、definition-local actualを含むfunctor/inline-functor application。Specs 10/13.2、MC-G017/020。 | Tasks 249-252、resolver signature candidate、Task-10 imported/local application consumer。 | overload winner、implicit-definition proof、evidenceなしのresult typeなし。 |
| 254 | root/member/view identity、ordered field、inheritance-path request、result-type requestを含むstructure constructor/selector/update term。Specs 05/13.3、MC-G017/018。 | Tasks 249-253、後続Task 263のsource definition payload、Task-10 structure-term consumer。 | constructor property argument、field coverage、upcast winner、structure evidenceを捏造しない。 |
| 255 | generator scope、predicate/body link、sethood request、written target type、explicit conversion intentを含むset enumeration/comprehension、choice、`qua` term。Specs 07.8.1/08.2/13.4-13.6、MC-G017/018。 | Tasks 248-254、Task-10 set/choice/`qua` consumer。 | missing sethood/narrowing proof、implicit widening path、comprehension factを捏造しない。 |
| 256 | completeなterm/type/attribute linkとexpected-input requestを持つpredicate application、equality/inequality、membership、type/attribute assertion。Specs 09/14.2/14.5、MC-G017/020。 | Tasks 249-255、Task-10 exact atomic-formula consumer。 | checker evidenceなしのtruth/theorem acceptance/inequality proof/assertion factなし。 |
| 257 | constant、negation、binary connective、quantified variable、child graph、context、role、source order。Specs 04.5/14.3-14.4、MC-G011/017/020。 | Tasks 248-256、Task-10 connective/quantifier consumer。 | child identityを失うflattening、implicit closure、truth value、theorem statusなし。 |
| 258 | general theorem owner/statement-semantic shell、assumption/conclusion/witness、resolver identityとしてのlabel/citation、local context、visibility-scoped input fact、candidate fact input。Specs 15/16、MC-G019/020。 | Tasks 248-257、resolver label fact、準備済み`MT10-FS` consumer。 | input/candidate assumption/factのみ。verified premise publication、checked theorem fact、discharge、theorem acceptance、proof closureなし。 |
| 259 | parameter、guard、definiens graph、property/correctness-condition identity、`InitialObligationId`、source anchor input、declaration provenanceを持つpredicate definition。Specs 09/16.6。 | Tasks 248-258、Task-10 definition consumer。 | recursive unfolding、property proof、obligation discharge、`VcId`、accepted obligation、overload selection、axiom publicationなし。 |
| 260 | `equals`/`means`、parameter、guard、result type、definiens、property/correctness-condition identity、`InitialObligationId`、source anchor input、declaration provenanceを持つfunctor definition。Specs 10/16.6。 | Tasks 248-259、Task-10 definition consumer。 | existence/uniqueness proof、obligation discharge、`VcId`、recursive unfolding、accepted result、overload winnerなし。 |
| 261 | subject/parameter、positive/negative definiens、guard、radix/qualification、correctness obligation requestを持つattribute definition。Specs 06/09/16.6。 | Tasks 248-260、Task-10 attribute-definition consumer。 | attribute truth、cluster fact、existential evidence、accepted proof、redefinition selectionなし。 |
| 262 | parameter、mode application、expansion/RHS、definiens、sethood/existence obligation request、declaration contextを持つmode definition。Specs 07/16.6。 | Tasks 248-261、Task-10 mode-definition consumer。 | property implementationはTask 264。accepted existence、expansion fact、registration activationなし。 |
| 263 | parameter、parent、root+path/view member identity、field coverage/type、constructor/selector declaration、coherence obligation requestを持つstructure definition/inheritance。Specs 05/13.3/16.6/19.2.2。 | Tasks 248-262、Task-10 structure-definition consumer。 | property-valued constructor argument、member identity、accepted coherence、chosen upcastを捏造しない。 |
| 264 | owner/property identity、local parameter、`means`/`equals` definiens、overlap domain、correctness-condition identity、`InitialObligationId`、existence/uniqueness/coherenceのsource anchor inputを持つmode property implementation。Specs 07.4.1/07.8.2/16.6。 | Parser Task 48、Tasks 248-263、Task-10 property consumer。 | parser grammar ownership、overlap acceptance、constructor property source、`VcId`、obligation discharge、proof acceptanceなし。 |
| 269 | `let`/`set`/`given`/`consider`/`take`等のproof-local declaration/binding、context transition、source-order closure。Specs 04/15.2-15.4/16.4。 | Tasks 248-258、準備済み`MT10-FS` consumer。 | inline abbreviation expansion、reconsider coercion、proof search、accepted witnessなし。 |
| 270 | formal identity、captured free variable、body graph、guard、substitution request、capture-avoidance provenanceを持つproof-local `deffunc`/`defpred` closure。Specs 04.4.3/10.11.3/15.2.3-15.2.4、architecture 16。 | Tasks 248-269、existing advanced-semantics trace row用の`MT10-AS` capture consumer。同producerは`MT10-FS`にもproof-local declaration dataを供給できるが、trace-row ownershipは移らない。 | explicit replay evidenceなしのsubstitution result、runnerでのcapture修復、accepted local theoremなし。 |
| 271 | binding、source/target type、written/omitted justification intent、widening/narrowing request、proof-free evidence queryを持つ`reconsider`。Specs 04.4.2/08.2/15.5.1/19.3.2。 | Parser Task 47、Tasks 248-258/269、proof-local family用`MT10-FS` consumerとexisting omitted-justification advanced-semantics fixture用`MT10-AS` consumer。 | omitted proofをacceptせず、narrowing evidenceを捏造せず、parser expectation driftをここで修復しない。 |
| 272 | non-Task-180 proof skeleton/justification: nested proof node、thesis/terminal goal、citation、local path、case/suppose/now、明示pending/blocked state。Specs 15.6/15.8/16.3-16.5。 | Tasks 248-271、resolver label identity、`MT10-FS` consumerと、explicit pending/blocked intentをassertする`MT10-AS` omitted-`reconsider` negative consumer。 | Task-180 tableはTasks 266-268。proof search、implicit closure、acceptance、discharge、Core/VCなし。 |
| 273 | existential/conditional/functorial/reduction registrationのitem/correctness payload: guard、pattern、consequent、source order、correctness-condition identity、`InitialObligationId`、source anchor input。Specs 07.8/16.6.3/17.2-17.6。 | Tasks 249-272、`MT10-AS` consumer。 | pending registration/obligation intakeのみ。`VcId`、discharge、accepted status、activation、closure、rewrite result、artifact、theorem factなし。 |
| 274 | **blocked-reserved:** canonical accepted verifier/artifact statusをimport/validateし、authenticated source/order/provenanceを持つeligible registrationだけをactivateする。Specs 17.1/17.3.4/17.8.4と既存checker policy。 | Task 273と将来のcanonical verifier/artifact owner/schema。upstream ownerは現在未命名で、authorityが命名するまでTask 274は実行不能。 | source order、local check、obligation request、pending registrationから`Accepted`を生成しない。このgate命名は実装authorityを与えない。 |
| 275 | applicable registration identity、normalized input/output、ordered firing、bound/loop/contradiction、完全provenanceを持つsource-derived cluster closure trace。Spec 17.7/17.9、MC-G021/023。 | Tasks 251/256-257/273-274、`MT10-AS` consumer。 | unaccepted registration、unrecorded fact、arbitrary theorem reasoning、cache/artifact result、runnerでのtrace再構築なし。 |
| 276 | accepted reduction identity、guard evidence、orientation/termination check、normalization step、result dependence、loop/bound/failure、provenanceを持つreduction trace。Spec 17.6/17.9.4、MC-G023。 | Tasks 251-257/273-275、`MT10-AS` consumer。 | `such`はapplicabilityのみ。unaccepted rewrite、hidden normalization、artifact/cache捏造、proof dischargeなし。 |
| 277 | parser/syntaxが既に公開するdirect template-role declaration、formal/actual、constraint/guard、substitution request、provenance。Spec 18、MC-G027。 | Tasks 248-264、`MT10-AS` consumer。 | Task 277は実行可能でdirect template roleだけをcloseする。external Gate S1のmissing scheme/theorem roleを所有/closeせず、omitted actual/inference/substitution resultを捏造しない。 |
| 278 | 既存collection/expansion/viability/specificity/ordinary-root selection/inserted-view APIへ渡すordinary/template overload site/candidate payload。Specs 08/18/19.1-19.4/19.6、MC-G027。 | Tasks 249-257/259-264/277、`MT10-AS` consumer。resolver Task 31のsame-return declaration conflictはindependent Task-49 prerequisiteで、Task-278 payloadではない。 | evidence/comparison inputはexplicit。return-type tie-break、omitted comparison evidence、hidden `qua`、redefinition refinementを捏造しない。 |
| 279 | bound ordinary target/root、synonym/antonym relation、`coherence with` intent/omission、target diagnostic payload、refinement candidate、accepted-coherence input、exposed viewを持つredefinition/notation producer。Specs 06.7/09.6-9.7/10.7-10.8/11.1/19.5。 | Tasks 259-264とTask 278 ordinary-root output、`MT10-AS` consumer。 | 複数root時のtarget、coherence proof、priority edge、alias semantics、accepted refinementを捏造しない。 |

checker境界でgraphはacyclicである。Task 278がordinary/template root結果を先に
生成し、Task 279は既に同定済みordinary rootへredefinitionをbindして、
authenticated refinement dataだけを既存selection layerへ渡す。Task 279は新たな
ordinary-root candidateをTask 278へ戻さない。

## 準備済み`mizar-test` Task-10 runner increment

これらはopenな`mizar-test` Task 10内のconsumer incrementであり、新しいchecker
task番号でも、新しいtop-level mizar-test taskでもない。

| Increment | scope | 依存とexit criteria |
|---|---|---|
| `MT10-FS` | formula-statement stage/tag admission、plan/report、deterministic rerun、expectation validation、formula/statement/proof-local familyのsource-to-checker execution。distinct future fixtureとsingular sidecar `pass_formula_statement_reserved_variable_equality_smoke_001.miz`を追加し、sidecar stageを`formula_statement`とする。active type-elaboration fixtureをreclassifyせずsidecarも追加しない。exact sourceは`reserve x for set;`に続く`theorem FormulaStatementReservedVariableEqualitySmoke: x = x;`。reserve/two terms/equality/theorem owner/statement shell/explicit non-accepting omitted-justification stateを`ResolvedTypedAst`まで保存する。 | Tasks 248-272。新しい実sourceがpositive case。同bundleのmissing/duplicate/reordered/cross-owner corruptionがsemantic `.miz` failを捏造せずnegative runner testになる。既存`pass_type_elaboration_reserved_variable_equality_001`と唯一のsidecarは変更せず現在のcreditを維持する。planned seedをexecuted計上せず、truth/acceptance/Core/VC/Steps 6/7 creditなし。 |
| `MT10-AS` | advanced-semantics stage/tag admission、plan/report、deterministic rerun、expectation validation、definition/registration/cluster/reduction/template/overload/redefinition/reconsider-conversion/definition-time capture-avoidance familyのsource-to-checker execution。ordinary-root non-Task-49 smokeは、1個の`set` typed argument/resultを持つlocal ordinary functor root、1個の`set` reserve、そのrootを1回applyするreflexive equality theoremで、template/redefinition/registration/cluster/reduction/proof-acceptance inputを持たない。Task 278は編集前にSpecs 10/13/14/19に対してparser-valid spellingを固定する。distinct capture smokeはfuture `pass_advanced_semantics_definition_time_capture_avoidance_001.miz`で、exact semantic fragmentはouter `m`をbindし、`defpred P(n be Nat) means n < m;`を定義し、display name `m`をshadowしてから`P`をapplyする。runnerはclosureがouter resolved `m` identityを保持し、formal substitutionがそれをcapture/rewriteしないことを証明する。Task 270は編集前にparser-valid enclosing proof shellを固定する。existing `fail_types_reconsider_omitted_justification_001` sidecarは`advanced_semantics`のまま。parser Task 47とTasks 251/271-272後、runnerはexplicit omitted intent、unavailable proof-free narrowing evidence、non-accepting pending/blocked result 1件、proof searchなしの`type.narrowing_requires_proof`をassertする。 | 当該consumerはTasks 249-264/270-273/277-279。missing/duplicate/reordered/cross-root candidate corruption、captured-identity/formal/substitution-request corruption、missing/wrong reconsider intent/evidence/status corruptionをnegative runner testにする。accepted registrationを要するcaseはTasks 274-276にも依存。smokeとmapped fail caseはTask-49 reconciliation setの他fixtureをactivateせずreal applicable producerを実行する。substitution result/omitted proofをaccepted creditしない。 |

## 既存boundary/trace所有権

Task 247はownership noteだけを変更する。現在のumbrella extraction rowと全exact
active diagnostic rowはstatus/tests/coverageを維持する。

| 既存boundary family | owner |
|---|---|
| generic declaration/binding、non-builtin type extraction | Tasks 248-251 |
| argument-bearing/bracket mode/structure、imported structure、mode expansion/evidence request | Tasks 249/251 |
| argument-bearing/qualified/imported/positive-negative attribute | Task 250、evidence requestはTask 251 |
| primary/imported-application/set-enumeration/structure/comprehension/choice/`qua` term | Tasks 252-255 |
| builtin/imported atomic formula/assertion | Task 256 |
| connective/constant/child-graph/quantifier-binder | Task 257 |
| formula-statement/statement-proof/assumption/conclusion/fact | Tasks 258/269-272と`MT10-FS` |
| predicate/functor/attribute definition | Tasks 259-261 |
| mode/structure/property/inheritance/constructor | Tasks 262-264、property syntaxはparser Task 48 |
| proof-local declaration/inline definition/capture/reconsider/proof skeleton | Tasks 269-272、reconsider syntaxはparser Task 47 |
| registration block/correctness/accepted activation | Task 273とblocked-reserved Task 274 |
| cluster/reduction source trace | Tasks 275-276 |
| direct template role/overload/redefinition/notation | Tasks 277-279。missing scheme/theorem roleはexternal Gate S1 |
| deferred `formula_statement` runner row | `MT10-FS` |
| deferred registration/cluster/reduction/overload `advanced_semantics` row | `MT10-AS`、Tasks 273-279、明記されたexternal Gates A1/S1 |
| deferred definition-time capture-avoidance row | Task 270と`MT10-AS`。Task 270はproof-local payloadを`MT10-FS`にも供給できるが、existing advanced-semantics trace rowは`MT10-AS` ownership |
| deferred type-soundness escape/guard row: witness leakage、local-definition guard、sethood、invalid `qua` | Tasks 258/272、Task 270、Tasks 251/255/271と該当`MT10-FS`/`MT10-AS`。これらはTask-49 24-fixture bundle外 |

広範なimported-attribute/imported-structure deferred rowはdeferredのままである。
既にactiveなexact sliceは現在のcreditを維持し、Tasks 249-251は将来の広範なsource
familyだけを所有する。

## Task-49 corpus mapping

semantic auditはadversarial fixture 25件を列挙する。same-signature/
different-return resolver fixtureは既にactiveで、下記set外のunchanged controlで
ある。Task-247 entryでは他の24件がinactiveで、exact **24-fixture reconciliation
set**を構成する。このうちsame-return memberのsole activation ownerとconsumerは
resolver Task 31の`declaration_symbol`である。Task 49は全mapped producer/runner/
gate完了後に他の23件をactivateし、resolver-owned memberを再activateせず24件全体を
reconcile/deduplicateする。

| # | literal fixture ID | activation ownerと必須owner/gate |
|---:|---|---|
| 1 | `fail_cluster_reduce_cycle_orientation_001` | Tasks 273-274/276と`MT10-AS`後にTask 49 |
| 2 | `fail_cluster_reduce_commutative_orientation_001` | Tasks 273-274/276と`MT10-AS`後にTask 49 |
| 3 | `fail_cluster_reduce_fresh_variable_001` | Tasks 273-274/276と`MT10-AS`後にTask 49 |
| 4 | `fail_cluster_reduce_duplicating_variable_001` | Tasks 273-274/276と`MT10-AS`後にTask 49 |
| 5 | `fail_cluster_contradictory_consequent_001` | Tasks 250-251/256-257/273-275と`MT10-AS`後にTask 49 |
| 6 | `fail_cluster_functorial_for_guard_001` | Tasks 250-251/256-257/273-275と`MT10-AS`後にTask 49 |
| 7 | `fail_mode_missing_existential_001` | Tasks 251/262/273-275、accepted statusが必要な場合Gate A1、`MT10-AS`後にTask 49 |
| 8 | `fail_mode_existential_after_declaration_001` | Tasks 251/262/273-275、accepted statusが必要な場合Gate A1、`MT10-AS`後にTask 49 |
| 9 | `fail_structure_diamond_member_type_conflict_001` | Task 263と`MT10-AS`後にTask 49 |
| 10 | `fail_structure_inherit_duplicate_member_coverage_001` | Task 263と`MT10-AS`後にTask 49 |
| 11 | `fail_structure_inherit_cycle_001` | Task 263と`MT10-AS`後にTask 49 |
| 12 | `fail_structure_inherit_uncovered_member_001` | Task 263と`MT10-AS`後にTask 49 |
| 13 | `fail_structure_constructor_property_arg_001` | Tasks 254/263-264、parser Task 48、`MT10-AS`後にTask 49 |
| 14 | `fail_overload_incomparable_roots_001` | Tasks 255/263/277-278、missing roleを要する場合Gate S1、`MT10-AS`後にTask 49 |
| 15 | `fail_overload_equivalent_roots_ambiguity_001` | Tasks 255/263/277-278、missing roleを要する場合Gate S1、`MT10-AS`後にTask 49 |
| 16 | `fail_overload_template_equivalent_roots_ambiguity_001` | Tasks 255/263/277-278、Gate S1、`MT10-AS`後にTask 49 |
| 17 | `fail_overload_inheritance_path_ambiguity_001` | Tasks 255/263/277-278、missing roleを要する場合Gate S1、`MT10-AS`後にTask 49 |
| 18 | `fail_resolve_same_signature_same_return_conflict_001` | **resolver Task 31がsole activation owner**、consumerは`declaration_symbol`。Task 49はreconcile/deduplicateのみ |
| 19 | `fail_types_qua_narrowing_001` | Tasks 255/263/278と`MT10-AS`後にTask 49 |
| 20 | `fail_types_qua_unrelated_struct_001` | Tasks 255/263/278と`MT10-AS`後にTask 49 |
| 21 | `fail_types_comprehension_missing_sethood_001` | Tasks 251/255と`MT10-AS`後にTask 49 |
| 22 | `fail_types_reconsider_omitted_justification_001` | parser Task 47、Tasks 251/271-272、`MT10-AS`後にTask 49。existing advanced-semantics sidecar stageを維持 |
| 23 | `fail_mode_property_overlap_missing_coherence_001` | parser Task 48、Tasks 262-264、`MT10-AS`後にTask 49 |
| 24 | `fail_overload_redefine_ambiguous_target_001` | Tasks 278-279と`MT10-AS`後にTask 49 |

Task 49は後続の1個の23-member activation兼24-member reconciliation/
deduplication taskのままである。各fixtureがowning runnerを通って実行された後に限り
Task-29 deferred rowを更新できる。既にactiveなdifferent-return control、resolver-
owned same-return member、独立にcoveredなrowを二重計上してはならない。

## disagreement分類

| protocol class | Task-247 findingと処置 |
|---|---|
| `spec_gap` | 既存MC-G005 public-diagnostic-code allocation gapはnonblocking external registry/consumer-adoption gateとして残る。新しいpayload-family spec gapはなく、英語canonical chapterはfamilyとnegative boundary命名に十分。 |
| `test_gap` | 24 inactive Task-49 fixture、広範なsource-derived family、formula-statement/advanced runner、exact positive/negative sliceが未実行。statusを変えずgraphへ割当。 |
| `design_drift` | 残familyがumbrella ownerだけだった。本decompositionとpaired ownership更新で解消。 |
| `source_drift` | checker APIはexplicit payloadをconsumeするがAST-wide producerと複数semantic consumerがない。Tasks 248-264/269-279へ割当。parser Task 47は別のexact source drift。 |
| `source_undocumented_behavior` | なし。現在のexact bridgeはcanonical requirementより狭く、credit limitを既に文書化。 |
| `test_expectation_drift` | omitted-`reconsider` parser expectationがcanonical optional-justification syntaxと不一致。parser Task 47がownerで、Task 247は修復/rebaselineしない。 |
| `boundary_violation` | 現在なし。checker/coreでのAST再構築、evidence/acceptance捏造、runnerによるchecker結果計算はviolationとなるため明示禁止。 |
| `repo_metadata_conflict` | なし。自動metadata修復authorityなし。 |

## external gateとdeferred authority

- **Gate A1 — accepted registration status:** Task 274はcanonical verifier/
  artifact owner/accepted-status schemaが未命名のため
  blocked-reservedである。Task 247はownerを捏造しない。将来のcanonical authority
  がproducer/schema/authentication rule/negative testを命名して初めて実行可能。
- **Gate S1 — scheme/theorem source role:** 欠落module scheme declaration shellと
  scheme/theorem role payloadは将来のnamed canonical parser/syntax/resolver
  owner待ち。このgateはexecutable Task 277に含めず、checkerは合成しない。
- MC-G004 artifact/schema integrationは未命名external gateのまま。checker
  payload taskはartifact schema/reuse contractを捏造しない。
- MC-G005 public checker diagnostic allocationは既存のnonblocking `spec_gap`かつ
  未命名registry/consumer-adoption gateのまま。後続taskはstable internal detail
  keyを保存できるがpublic numeric code/aliasを割り当てない。
- Parser Tasks 47-48とresolver Task 31は独立にauthorizedされたprerequisiteで、
  completed Tasks 266-268/Core Task 31のdependencyではない。
- Steps 6/7はdeferredのまま。本graphは昇格authorityを与えない。

## Task-247 exit criteria

Task 247は次を全て満たした場合だけcompleteである。

- 全remaining family、MC-G owner、boundary fixture group、deferred runner row、
  inactive Task-49 fixtureにexactly one producer/consumer ownerまたはexplicit gate;
- 英語正本/日本語companion、checker plan/TODO/audit、mizar-test Task-10文書、
  trace ownership note、spec coverage auditが一致;
- `spec_trace.toml`はdeferred owner/reason wordingだけを変更し、status/test list/
  coverage classを維持;
- source/fixture/expectation/runner count/test list/coverage credit変更なし;
- review-only specification/test-sufficiency/implementation-scope/
  source-document consistency reviewがfindingなし;
- full baseline verificationとcount/hash oracleがgreen;
- Task-247変更を1 docs/traceability logical taskとしてcommit。

そのcommit後、Core Task 32はTasks 248-264/269-279の実装完了を待たず、本accepted graphを
自身のdocs/traceability-only remaining-family decompositionのinputにできる。ただし
ここに記録した全gateとforbidden boundaryを維持しなければならない。

Core Task 32は
[source_family_decomposition.md](../../mizar-core/ja/source_family_decomposition.md)
をacceptedした。Checker Tasks 248-279にChapter-20 algorithm rowがない点は意図的な
scopeであり、新checker task IDのauthorityではない。Core Tasks 42-47は別々のjoint
vertical taskで、`mizar-test`がAST extraction、checkerがsyntax-free final projection、
Coreがloweringを所有する。Exact dependent sliceではGates A1/S1を維持する。この
ownership noteはchecker source/task status/fixture/expectation/coverageを変更しない。

## Task 248 completion

Task 248はexact bounded rowについてcompleteである。実装済み
`SourceBindingContextHandoff`はsource-item/declaration order、resolver shell/
local-binding provenance、module/declaration context link、structural
local-to-reserve shadow relationを`TypedAst`と`ResolvedTypedAst`まで保持する。
single active Task-10 fixtureはterm-use lookup siteを持たず、type result、
RHS/formula/proof payload、fact、obligationを生成しない。次の
dependency-authorized producerはTask 249である。Tasks 269+とSteps 6/7はpromoteしない。
