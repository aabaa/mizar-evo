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
| 249 | builtin/local/imported mode/structure radix、positional/bracket type argument、term argument、written type-site identityを含むtype head/application payload。Specs 03/05/07/08/12/18とAppendix A、MC-G014/016/020。 | Task 248、resolver symbol/provenance、crate planでfrozen済みexact Task-248 two-row co-consumerとten-reserve-root Task-10 type-elaboration consumer。 | type inputのみ。expansion、inhabitation、subtyping、term/`qua` selection、evidence resultを捏造しない。 |
| 250 | polarity、argument、qualification/owner identity、local/imported provenance、order、attributed type-site linkを含むattribute-chain payload。Specs 03/06/11/12と17 restricted-adjective boundary、MC-G014/020。 | Tasks 248-249。crate planはexact Task-67/81/84/85 consumer、Task-249 4/4/0 co-handoff、Task-250 4-chain/4-attribute/1-qualifier/1-group/1-actual handoff、synthetic prefix/order extraction coverageをfreezeする。 | later canonical equivalence向けにwritten prefix/list formを保持するが、arity/admissibility、owner compatibility、normalized instance、evidence、closure fact、attribute truthをsynthesizeしない。 |
| 251 | mode expansion、structure base shape/constructor witness、attributed-type inhabitation、sethood/non-emptiness、inheritance、coercion viabilityのevidence-query requestとupstream dependency-fact input。opaque `ExistentialGateInput` request identityとdependency-fact referenceを含む。Specs 03/05-08/13/17/19、MC-G016/018/026。 | Tasks 248-250。exact representative selectorはTask-249 broad fixtureとTask-84/85で、mode-expansion 5 + structure-inhabitation 3 + attributed 2 requestを全てmissingとしてemitする。同じproduction Task-10 pathがfinal `TypedAst`/`ResolvedTypedAst`までrequested/missing/rejected/supplied transport stateをinject/区別し、semantic imported summaryを仮定しない。 | request/site/provenance/reference transportだけを所有する。`Supplied`はreference arrivalであり、それだけではaccepted/consumable evidenceではない。later Tasks 252-255/263/271/278がnew source siteをownし、accepted evidence、theorem result、artifact statusは外部input。 |
| 252 | variable/constant reference、`it`、numeral、transparent parenthesisとbinding/role/parent/numeric-type request。Specs 04.1-04.3/04.4.1/04.6、13.1/13.8.1-13.8.2/13.8.8、MC-G017/020。 | Tasks 248-251。exact real Task-10 selectorはnumeral equality、reserved-variable equality、parenthesized reserved-variable equalityでaggregate term/reference/numeric-request oracleは7/4/2。Tasks 260/264/269がreal owner payloadを供給するまでconstant/`it`とeligible nested parenthesis schemaはsynthetic producer/extractor testがownする。referenceはvisibilityだけでなくexact `BindingEnv::lookup` winnerでauthenticateする。 | transport only。parenthesisはsemantic term/type/FOL nodeを追加せず、numeric requestはresult/factを追加しない。arbitrary application、structure/set term、formula、definition/current-result acceptance、local binding productionをownしない。cross-family parent edgeはTasks 253-255を待つ。 |
| 253 | frozen/open: ordinary functor application source shape、Task-253-owned cross-family transparent wrapper/origin、個別認証ordinary candidate reference、Task-252 primary-term / nested-application argument edge、unresolved candidate-signature/application-result request。Inline shapeはsyntheticのみ。Specs 10/13.2/15.2.3/19、MC-G017/020。 | Tasks 248-252、exact Task-10 imported `(1 ++ 2)` caseと、同じdefinition blockで先に完了したfunctorを後続functorのdefiniensから適用する新規spec-derived case。Task 270がinline identity/formal/capture/substitution、Task 277がdirect template transport、Task 278がordinary/template candidate collection/viability/winnerを所有する。 | contract frozen、producer absent。primary重複所有、exhaustive candidate claim、overload winner、semantic signature/result、definition proof、template payload、inline semantic identityなし。 |
| 254 | root/member/view identity、ordered field、inheritance-path request、result-type requestを含むstructure constructor/selector/update term。Specs 05/13.3、MC-G017/018。 | Tasks 249-253、後続Task 263のsource definition payload、Task-10 structure-term consumer。 | constructor property argument、field coverage、upcast winner、structure evidenceを捏造しない。 |
| 255 | generator scope、predicate/body link、sethood request、written target type、explicit conversion intentを含むset enumeration/comprehension、choice、`qua` term。Specs 07.8.1/08.2/13.4-13.6、MC-G017/018。 | Tasks 248-254、Task-10 set/choice/`qua` consumer。 | missing sethood/narrowing proof、implicit widening path、comprehension factを捏造しない。 |
| 256 | completeなterm/type/attribute linkとexpected-input requestを持つpredicate application、equality/inequality、membership、type/attribute assertion。Specs 09/14.2/14.5、MC-G017/020。 | Tasks 249-255、Task-10 exact atomic-formula consumer。 | checker evidenceなしのtruth/theorem acceptance/inequality proof/assertion factなし。 |
| 257 | constant、negation、binary connective、quantified variable、child graph、context、role、source order。Specs 04.5/14.3-14.4、MC-G011/017/020。 | Tasks 248-256、Task-10 connective/quantifier consumer。 | child identityを失うflattening、implicit closure、truth value、theorem statusなし。 |
| 258 | general theorem owner/statement-semantic shell、assumption/conclusion/witness、resolver identityとしてのlabel/citation、local context、visibility-scoped input fact、candidate fact input。Specs 15/16、MC-G019/020。 | Tasks 248-257、resolver label fact、準備済み`MT10-FS` consumer。 | input/candidate assumption/factのみ。verified premise publication、checked theorem fact、discharge、theorem acceptance、proof closureなし。 |
| 259 | parameter、guard、definiens graph、property/correctness-condition identity、`InitialObligationId`、source anchor input、declaration provenanceを持つpredicate definition。Specs 09/16.6。 | separate Task-248 two-parameter profile extension後のexact Tasks 248/249/252/256 handoffとpass Task-10 definition consumer。frozen sourceではTasks 253-255/257/258はabsent。 | recursive unfolding、guard-conditioned FOL property-VC construction、property proof、obligation discharge、`VcId`、accepted obligation、overload selection、axiom publicationなし。future Task 272はunconsumed justification subtreeを保持。 |
| 260 | `equals`/`means`、parameter、guard、result type、definiens、property/correctness-condition identity、`InitialObligationId`、source anchor input、declaration provenanceを持つfunctor definition。Specs 10/16.6。 | Tasks 248-259、Task-10 definition consumer。 | existence/uniqueness proof、obligation discharge、`VcId`、recursive unfolding、accepted result、overload winnerなし。 |
| 261 | subject/parameter、positive/negative definiens、guard、radix/qualification、correctness obligation requestを持つattribute definition。Specs 06/09/16.6。 | Tasks 248-260、Task-10 attribute-definition consumer。 | attribute truth、cluster fact、existential evidence、accepted proof、redefinition selectionなし。 |
| 262 | parameter、mode application、expansion/RHS、definiens、sethood/existence obligation request、declaration contextを持つmode definition。Specs 07/16.6。 | Tasks 248-261、Task-10 mode-definition consumer。 | property implementationはTask 264。accepted existence、expansion fact、registration activationなし。 |
| 263 | exact zero-parameter structure-definition/inheritance payload: 2 declarations、4 typed field/property selectors、1 parent edge、exact root+path/view mapping/coverage、fields-only constructor order、identical `set` typeに対する0 derived coherence requests。Specs 05/13.3/16.6/19.2.2。 | Tasks 249/249S、263R、committed Tasks 248-262 boundary、1 private structure-definition runner consumer。 | fabricated parameter/context、property constructor argument、inferred identity、nonidentical-type goal/guard、accepted coherence、chosen upcast、fact/proof/Core/CFG/VCなし。 |
| 264 | owner/property identity、1 local parameter、`means`/`equals` definiens、declared return association、correctness-condition identity、`InitialObligationId`、existence/uniquenessのsource anchor inputを持つstruct-property implementation。Specs 05、07.4.1/07.8.2/07.10、13.1.2/13.8.2、16.6.1/16.6.2/16.7.2。 | Parser Task 48、Tasks 248-256/263/264R、dedicated property-implementation runner consumer。 | five-table transportとpending initial-obligation intakeのみ。parameter/domain/return-type goal/guard、overlap/coherence detection、property value、`VcId`、discharge、acceptance、fact、proof、Core/CFG/VC payloadなし。 |
| 269 | `let`/`set`/`given`/`consider`/named `take`等のproof-local declaration/bindingとfirst-order local-term abbreviation payload。context transition、source-order closure、definition-site binding/RHS link、later term reference用capture-by-resolved-binding replayを含む。Specs 04/15.2-15.4/16.4。 | Tasks 248-258、準備済み`MT10-FS` consumer。 | `deffunc`/`defpred` closureはTask 270、`reconsider` coercionはTask 271、existential-binder matching/witness type obligation/goal substitutionはTask 272。proof search、accepted witnessなし。 |
| 270 | formal identity、captured free variable、body graph、guard、substitution request、capture-avoidance provenanceを持つproof-local `deffunc`/`defpred` closure。Specs 04.4.3/10.11.3/15.2.3-15.2.4、architecture 16。 | Tasks 248-269、existing advanced-semantics trace row用の`MT10-AS` capture consumer。同producerは`MT10-FS`にもproof-local declaration dataを供給できるが、trace-row ownershipは移らない。 | explicit replay evidenceなしのsubstitution result、runnerでのcapture修復、accepted local theoremなし。 |
| 271 | binding、source/target type、written/omitted justification intent、widening/narrowing request、proof-free evidence queryを持つ`reconsider`。Specs 04.4.2/08.2/15.5.1/19.3.2。 | Parser Task 47、Tasks 248-258/269、proof-local family用`MT10-FS` consumerとexisting omitted-justification advanced-semantics fixture用`MT10-AS` consumer。 | omitted proofをacceptせず、narrowing evidenceを捏造せず、parser expectation driftをここで修復しない。 |
| 272 | non-Task-180 proof skeleton/justification: nested proof node、thesis/terminal goal、citation、local path、case/suppose/now、明示pending/blocked state。`take`ではordered witness-to-existential-binder matching、explicit witness type-obligation request、capture-avoiding goal-substitution trace、remaining goalも所有。Specs 15.4.4/15.6/15.8/15.11.5/16.3-16.5。 | Task-269 named-witness binding/RHS provenanceを含むTasks 248-271、resolver label identity、`MT10-FS` consumerと、explicit pending/blocked intentをassertする`MT10-AS` omitted-`reconsider` negative consumer。 | Task-180 tableはTasks 266-268。authenticated term/binder inputなしのsubstitution、invented type evidence、proof search、implicit closure、acceptance、discharge、Core/VCなし。 |
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

## Task 249 frozen-contract prerequisite completion

paired crate planはTask 249のexact syntax-free table、ten-reserve-root broad fail
consumer、10/13/6 raw cardinalityとdual form histogram、Task-248
two-Bare-builtin-row co-consumer、runner-only dependency status、
corruption/determinism matrix、future trace row 1件、expected count delta、
forbidden scopeをfreezeした。これはcompleteした独立documentation prerequisiteであり、
Task 249 implementationではない。source、test、expectation、trace row/status、
count、hash、coverage credit、Tasks 269+、Steps 6/7は不変である。

## Task 249 implementation completion

frozen producerはpublic syntax-free checker `source_type` boundaryとprivate exact
mizar-test consumerとして実装済みである。broad routeはexact 10/13/6 tableを
publishしてrunner-owned pending detailで停止し、unchanged Task-248 routeはexact
2/2/0 dependency regressionをco-installする。validated immutable handoffは
`TypedAst`が所有し、`ResolvedTypedAst`はcloneだけする。

real resolverはderived frozen scaffolding内のrowをemitしないformal/field
spelling重複を検出した。このtask-local `design_drift`とparse-only preflightの
`test_gap`はdistinct nameだけでrepairし、source-type oracleと言語intentは変更
していない。new bounded diagnostic trace rowはselected Task-249 `test_gap`/
`source_drift`だけをcloseする。Tasks 250+/269+、normalization、term/`qua`
binding selection、later semantic payload、Steps 6/7はdeferredのままである。

## Task 250 frozen-contract prerequisite completion

paired crate planはTask 250のexact syntax-free chain/attribute/polarity/qualifier/
argument-group/actual table、existing real consumer 4件と4/4/0および4/4/1/1/1
cardinality oracle、Task-67/81 runner-only outcome progression、Task-84/85
evidence-query preservation、legacy `AttributeInput` coexistence、synthetic
`SurfaceAst` prefix/order extractor coverage、future trace row 1件と必要なexisting
trace-note update、expected plan 411/373・type 239/227、corruption matrix、
forbidden scopeをfreezeした。これはcompleteした独立documentation prerequisiteで
あり、Task 250 implementationではない。source、test、expectation、trace、count、
hash、coverage credit、Tasks 251+/269+、Steps 6/7は不変である。

## Task 250 source-attribute producer completion

frozen familyはpublic syntax-free `source_attribute` producerとprivate runner
extractor 1件で実装済み。exact Task-81/67/84/85 real routeだけがTask-249
dependency 4/4/0とTask-250 handoff
4-chain/4-attribute/1-qualifier/1-group/1-actualをpublishする。synthetic prefix
probeとchecker corruption/determinism matrixはselected exact `test_gap`とraw
transport `source_drift`をcloseする。bounded trace rowはnew case/admission change
なしでplan 411/373・type 239/227へ到達する。semantic attribute instanceと全
evidence/truth/acceptance/downstream IR、Tasks 251+/269+、Steps 6/7はexisting
later ownerに残る。

## Task 251 current-state addendum

Task 251はsemantic evidence ownerをadvanceせずgraphのrequest/reference
transport nodeを実装する。public checker handoffはexact Task-249 application、
optional Task-250 chain、resolver symbol kind、dependency key、payload
reference、fact、gate associationをatomic publication前にauthenticateする。
private Task-10 consumerはbroad Task-249 fixtureとTask-84/85だけをactivateし、
response row 0件のmissing request 10件を5/3/2 histogramでproduceする。
production four-state testとchecker corruption matrixがbounded
`source_drift`/`test_gap`をcloseする。Tasks 252-264/269-279のdependency edge/
exit boundaryは不変で、Steps 6/7はpromoteしない。

## Task 252 current-state addendum

Task 252はsemantic term/formula ownershipを進めずgraphのprimary-term transport
nodeを実装する。public checker handoffはfive frozen source kind、exact binding
winnerとproducer-derived binding-event ordinal、transparent parent closure、
unresolved numeric requestをatomic publication前にauthenticateする。private
Task-10 consumerはfrozen real route 3件だけをactivateしexact 7/4/2 aggregateを
produceする。synthetic probeはconstant、`it`、nested-parenthesis、mixed-family
boundaryをcoverする。bounded `source_drift`/`test_gap`はclosedである。Tasks
253+と260/264/269はexisting dependency edge/semantic ownerをretainし、Steps
6/7をpromoteしない。

## Task 253 frozen-contract prerequisite

paired crate planはpublic 5-table `source_application` contract、Task-252 primary/
nested-application argument edge、Task-253-owned transparent application-wrapper
relation、complete set/winnerを主張しない個別認証resolver candidate referenceを
freezeした。future real selectorは既存imported `1 ++ 2` routeと、同じdefinition
blockで完了した最初のfunctorを後続functorのdefiniensから適用する新規caseである。
local actualは外側のreserveではなく、再利用するTask-248 source-context handoffが
認証するinner `DefinitionParameter`である。aggregate Task-253
application/wrapper/candidate/argument/request oracleは2/1/2/3/4、参照する
Task-252 term/reference/numeric-request sliceは3/1/2である。

inline zero/one/two-actual shapeはsynthetic source-schema coverageだけで、identity、
formal、capture、substitutionはTask 270に残す。template applicationはwhole-subtree
で除外し、direct role/actual/guard/request transportはTask 277、ordinary/template
candidate collection/viability/winnerはTask 278に残す。この独立documentation
prerequisiteはselected `design_drift`を解消するがTask 253を実装しない。
`source_drift`/`test_gap`はopenで、source/fixture/expectation/trace status/
count/hash/executable credit、Tasks 254+、Steps 6/7は不変である。

## Task 253 current-state addendum

Task 253はsemantic term/definition/formula/overload-selection ownershipを進めず、
graphのfunctor-application transport nodeを実装する。public checker handoffは
five dense table、exact Task-252 debug fingerprint、root-only/nested argument
edge、transparent application wrapper、individual resolver functor referenceを
atomic publication前にauthenticateする。private Task-10 consumerはfrozen real
route exactly 2件だけをactivateし、Task-253 2/1/2/3/4とco-installed Task-252
3/1/2をproduceする。synthetic private-extractor probeはremaining source form
すべて、inline schemaだけ、nesting、wrapper、degraded transport、candidate
subset、whole-template/mixed-family exclusionをcoverする。bounded
`source_drift`/`test_gap`はclosedである。Tasks 254+、260、270、277、278は
existing dependency edge/semantic ownerをretainし、Steps 6/7をpromoteしない。

## Task 254 frozen-contract prerequisite

paired crate planはpublic syntax-freeな`source_structure` handoffをfreezeする。
seven dense immutable tableはstructure-family term、transparent wrapper、
authenticated constructor root、written member segment、parser `FieldUpdate`
container、ordered child edge、unresolved requestである。future exact real
consumerはTask-254 term/wrapper/root/member/field-update/edge/request =
5/0/3/9/2/10/26をpublishし、Task-252 primary/reference/numeric-request =
8/0/8をcomposeする。Task-253 row/fingerprintはない。

resolver `Structure` referenceとしてauthenticateするのはconstructor rootだけで
ある。written constructor label、selector name、update-path segmentはunresolved
member/path requestを伴うsource occurrenceのままで、repeated label/pathを判断・
deduplicateせず保存する。parser `FieldUpdate`はpath 1件とreplacement association
をownするが、独立term/type/factは持たない。Task-254 child edgeはone-wayに
Task-252 root、same-context Task-253 root application、後続same-context Task-254
rowを参照できる。Task-253 rootはどのTask-253 argument edgeからもtargetにされない
applicationで、nested Task-253 applicationはTask 254がmultiply ownせずrejectする。
structure childを含むTask-253 applicationはfrozen target vocabularyをreopen
しないためwhole-subtree excludedのままである。

authenticated structure definition、field/property kind、inheritance view、
coverage/default decision、constructor acceptance、selector result、update-copy
semantics、exact-instance evidenceはTask 263に残す。この独立prerequisiteがclose
するのはselected Task-254 `design_drift`だけである。production source、fixture、
sidecar、trace row/status/count、executable credit、measured 412/376・242/230
baseline、Tasks 255+/263-264、Steps 6/7は不変である。

## Task 254 current-state addendum

Task 254はsemantic member/structure-definition ownershipを進めずgraphの
structure-family source-transport nodeを実装する。public checker handoffはseven
dense table、arena-key class 5個、resolver constructor root、written member path、
`FieldUpdate` association/exact spelling、exhaustive direct written-child
partition、両install順のTask-252/253/254 ownership、conditional fingerprintを
atomic publication前にauthenticateする。private Task-10 consumerはTask-248
contextを再利用し、frozen definiens 3件だけをactivateしてTask-254
5/0/3/9/2/10/26とTask-252 8/0/8をproduceする。bounded `source_drift`、
`test_gap`、implementation時のcontext/cross-family install-order
`boundary_violation`はclosedである。
Tasks 255+/263-264がlater familyと全structure semanticsをretainし、Steps 6/7を
promoteしない。

## Task 255 frozen source-set-term family

Task 255はTasks 248/252-254直後のsource-transport graph nodeとしてfreezeする。
future `source_set_term` handoffはset/choice/`qua` term、transparent wrapper、
written comprehension generator、bare builtin target-type site、ordered child
edge、unresolved requestのdense table 6個を持つ。exact real transactionは
4/0/1/3/4/7、co-installed Task-252 4/0/4で、Task-253/254 target/fingerprintは
ない。

edgeはone-wayにTask-252 primary root、Task-253 root application、Task-254 root
structure term、nested Task-255 rowを参照できる。Task-255 childを含むreverse
Task-253/254 parentはwhole-subtree excludedである。Task-249 declaration-linked
type applicationをTask-255のterm-owned targetにもgenerator-owned targetにも
流用せず、bounded sliceはauthenticated bare `set`/`object` target siteだけを
admitする。

canonical row schemaはmaximal-effective-range partitionを使い、Task-253/254が
既ownするprimaryとTask-254が既ownするapplicationをTask-255が再targetしない。
unrelated optional handoffはrange-disjointで、later Task-253/254 installはinstalled
Task-255をrevalidateする。Task-255 request intentはTask 251のfrozen
type-application evidence originをextendしない。

generator rowはwritten declarationを保持するだけで`BindingId`/captureを作らない。
comprehension binder/context identityはTask 257、condition付きformula ownershipは
Tasks 256-257とのcompositionに残す。semantic result type、sethood、choice
nonemptiness/stability、`qua` widening/reduct、fact/acceptanceはTask 255の外である。
本docs-only prerequisiteは`design_drift`だけをcloseし、implementation
`source_drift`/`test_gap`はopenのままとする。

Task 255はこのfrozen boundary内で実装済みである。public 6-table producer、
private exact consumer、optional Task-253/254 fingerprint、final
`TypedAst`/`ResolvedTypedAst` ownership、bounded fixture/trace row、review済み
test matrixがbounded `source_drift`/`test_gap`をcloseする。generator
binding/captureはTask 257、condition formulaはTasks 256-257が引き続きownし、
semantic set/choice/`qua` creditは追加しない。

## Task 256 frozen atomic-formula family

Task 256はTasks 248/252-255直後のsource-transport graph nodeとしてfreezeする。
future `source_atomic_formula` handoffはformula occurrence、transparent wrapper、
ordinary predicate head、individually authenticated predicate candidate、
formula-owned bare asserted-type site、formula-owned simple attribute、direct term
edge、unresolved expected-input requestのdense table 8個を持つ。

exact real selectorは既存active fail fixture 8件を再利用し、新規`.miz`を追加しない。
independent transaction aggregateはTask-256 formula/wrapper/head/candidate/
type-site/attribute/edge/request `8/0/1/1/1/2/13/11`。direct edgeはTask-252
primary 10、Task-253 root application 1、Task-255 root set term 2である。
complete dependency aggregateはTask-252 `16/0/16`、Task-253
`1/1/1/2/2`、Task-255 `2/0/0/0/4/2`で、real Task-254 targetはない。

assertion type/attributeはoccurrence-specific Task-256 rowで、Task-249 declaration
applicationやTask-250 chainを捏造しない。initial sliceはbare builtin
`set`/`object`とsimple unqualified argument-free attributeだけをadmitする。
requestはoperand expected type、candidate signature、type reachability、attribute
admissibilityのunresolved intentだけを運び、Task 251をextendせず、answer/fact/
winner/truth/accepted formulaを作らない。

single-segment ordinary predicateだけをadmitし、chain conjunction、segment
negation、inline substitution、template argumentを作らない。predicate chainと
formula operator/binderはTask 257、inline closure/substitutionはTask 270、
template roleはTask 277、overload collection/selectionはTask 278がownする。
condition付きcomprehensionはTask-255をreopenせずTask-255/256/257 joint
follow-upに残す。

本documentation-only prerequisiteはTask-256 `design_drift`をcloseする。
public producer/final handoffはbounded `source_drift`、real/synthetic/corruption/
install/exclusion matrixは`test_gap`のままである。source、fixture、
expectation、trace row/status/count、count/hashを変更しない。

Task 256はこのfrozen boundary内で実装済みである。public 8-table producer、
private exact consumer、same-arena Task-252/253/255 composition、optional
Task-253/254/255 fingerprint、unresolved request 11件、immutable final handoff、
bounded reciprocal trace row、review済みreal/synthetic/corruption/install/
exclusion matrixがbounded `source_drift`と`test_gap`をcloseする。既存8 semantic
routeはoutcome/detail ownershipを維持する。predicate chain、formula
operator/binder、conditioned-comprehension compositionはTask 257、inline
closure、template role、overload selectionはTasks 270/277/278のままである。

## Task 257A frozen composite-formula/binder core

fresh inventoryはimplementation前にTask-257 umbrellaを分割する。Task 257Aは
dependency-readyなexact implication/universal/negation/contradiction treeと、
unused explicit universal binder 1件である。broader connective/quantifier、
implicit binder、bound use/captureはTask 257B、predicate-chainとconditioned-
comprehension compositionは必要なTask-256/255 contract extensionを別途freeze
した後のTask 257Cに残す。

public source familyはformula occurrence、transparent wrapper、unassigned root、
quantified binder、binder-owned type site、child edge、unresolved requestのdense
table 7個を持つ。唯一のunchanged real connective/quantifier fail sourceのexact
aggregateは`5/0/1/1/1/4/6`である。extended Task-248-era `BindingEnv`
schemaは`2/1/4`、すなわちnormal module-shell prefix、expression body context
1件、source-derived quantifier binding `x`、不変module-shell diagnostic 4件で
ある。Task-248 source-context handoffは作らない。

formula rowはparent-before-child preorderである。source-role edge 4件は
implication left/right、universal body、negated-formulaを形成し、
universal-body edgeだけがmodule context 0からexpression context 1へ遷移する。
request 6件はconnective、constant、quantifier、binder-type、negationのinput
intentだけを保持し、semantic answer/fact/truth/theorem owner/proof/acceptanceを
publishしない。

bounded binder typeはoccurrence-specific bare builtin `set` siteで、Task-249
declaration applicationではない。resolver-shaped local binder identityはwritten
declaration rangeとstable local scopeを使い、symbol/contribution/declaration
shell/opaque id/generated counterを捏造しない。

Task 257Aはこのexact sliceを実装した。public transport、binding extension、
private consumer、one-shot `TypedAst` install、final `ResolvedTypedAst`
clone preservation、bounded corruption/context/install/exclusion matrixが記録済み
`source_drift`/`test_gap`をcloseする。implementationは既存sidecar上のcovered
reciprocal trace requirementだけを追加し、canonical sourceと既存semantic
outcome/detail intentは不変である。broader shape、bound use/capture、
executable wrapper、predicate chain、conditioned comprehensionはTasks 257B-Cに残る。

### Task 257B dependency refinement

Task-257 authority/exit boundaryを変えずTask 257Bを分割する。Task 257B1は
explicit universal/binder profileをTask-256 equality 1件とTask-252 binding
reference 2件へ最初にcomposeする。Task 257B2はbroader binary/repeated
connective/grouping、Task 257B3はexistential/restricted/nested/
implicit-reserve binder formを追加する。binding context→primary term→atomic
formula→formula compositionの順でgraphはacyclic。

Task-257B1 `bound_uses` rowはformula-side associationだけである。Task 252は
lookup-winner/source-reference owner、Task 256はequality/operand ownerのまま。
`BindingEntry::captured`はdirect quantified occurrenceではなく
free-variable capture用に保持する。

Task 257B1はこのboundaryで実装済みである。exact pass routeはoccurrenceを
duplicateせずownerも移動せず、3 predecessor familyと`1/2` handoffをcomposeする。
bounded `source_drift`/`test_gap`はcloseし、次のgraph nodeはTask 257B2である。

### Task 257B2 connective/grouping node

次nodeは変更しないTask-257 explicit-binder environmentへTask-252 numeral、
Task-256 equality、第3 exact Task-257 composite profile、既存Task-257B1
cross-family table shapeをcomposeする。graphは`Task252 16/0/16 -> Task256
8/.../16/16 -> Task257B2 8/6/1/1/1/7/9 -> composition 8/0`。
`ParenthesizedFormula` 6件はtransparent wrapper rowのまま。fixed/repeated
conjunction/disjunction、`iff`、groupingだけをownし、Task 257B3 binder
extension、Task 257C predicate/comprehension、全semantic result familyは
downstreamに残す。

### Task 257B2 implemented node

frozen nodeは`Task252 16/0/16 -> Task256 8/0/0/0/0/0/16/16 ->
Task257B2 8/6/1/1/1/7/9 -> composition 8/0`として実行可能になった。
fixed/repeated connective tree/wrapperだけをtransportし、Task 257B3/257C、
connective truth、repetition expansion、theorem ownershipはdownstreamに残す。

### Task 257B3 frozen nested-binder node

次のgraph nodeはTask-48 one-binding bare-set reserve baseを4-binding nested
environmentへ先にextendし、その後Task-252 `6/6/0`、Task-256
`3/0/0/0/0/0/6/6`、fourth Task-257
`3/0/1/3/3/2/6` profile、formula composition `3/6`をcomposeする。
restricted explicit universal 1件、explicit existential 1件、nested
implicit-reserve universal 1件、same-family child edge 2件、atomic-parent
association 3件、formula-side bound-use association 6件だけを所有する。

Task 48はwritten reserve/default owner、Task 252はoccurrence/referenceと
lookup-winner owner、Task 256はequality/operand ownerのままである。
Task 257B3はreserve bindingをimplicit binder-type sourceとしてauthenticateし
shadow relationを保持できるが、predecessor rowをcopy/reinterpretしない。
quantified truth、witness construction、restriction discharge、implicit
theorem closure、capture result、Task 257C、theorem ownership、later semantic
stageはdownstreamのままである。

Task 257B3 implementationがcloseするのはfrozen composition transportの
`source_drift`とexact-consumerの`test_gap`だけで、predecessor row
ownership/downstream semantic responsibilityは移動しない。

## Task 257C1 frozen decomposition

Task 257C1はformula compositionではなくlower-family Task-256 extensionである。
Task 252はexact `3/0/3` numeral occurrence/requestを所有する。Task 256は
exact `1/0/2/2/2/0/0/3/2`のroot、segment/head/candidate各2 row、
polarity-token provenance、global argument/boundary edge 3件、
candidate-signature request 2件を所有する。middle primaryはedge idで共有し、
copyしない。Task 257は後でimplicit conjunction/semantic segment negation、
Task 278は後でoverload selectionを所有する。

別Task-255 condition-bearing comprehension extensionはこのimplementation
prerequisiteの後に続く。conditioned-comprehension/predicate-chain compositionは
別々のfuture Task-257C sliceなので、本contractはどちらにもsemantic creditを
与えない。

Task 257C1 transportはこのlower-family boundaryで実装済み。predicate-chain
compositionは未実装で、次のprerequisiteは別Task-255
condition-bearing-comprehension transportのまま。

## Task 255C1 frozen condition node

次のgraph nodeは
`Task252 4/0/4 -> Task253 1/0/1/2/2 -> Task255C1
1/0/1/1/1/1/2`である。Task 252はmapper/condition numeral、Task 253は
imported mapper、Task 255はcomprehension、generator、bare type、colon
association、direct condition-wrapper association、mapper edge、unresolved
set requestだけをownする。condition内のlower-family rowはcopy/targetせず、
Task-255 child discoveryから除外する。

Task 256がlater inner equality node/operand edge、Task 257Cがlater condition
compositionをownする。generator binding/captureと全semantic resultはdownstream
のままである。このprerequisiteはlater nodeがconsumeすべきimmutable objectを
凍結する。

## Task 255C1 implemented boundary

frozen dependency chain
`Task252 4/0/4 -> Task253 1/0/1/2/2 -> Task255C1
1/0/1/1/1/1/2`はexecutableになった。condition wrapperをrecursive exclusion
boundaryとし、inner equalityはdownstream Task-256/257 consumerに残す。semantic
familyはpromoteしていない。

## Task 257C2 frozen condition-formula edge

次のgraph nodeは
`Task252 4/0/4 -> Task253 1/0/1/2/2 -> Task255
1/0/1/1/1/1/2 -> Task256 1/0/0/0/0/0/0/2/2 -> Task257C2 1`。
Task 256はinner equalityとTask-252 operand edge 2件だけを取り、Task 257C2は
immutable condition-0-to-formula-0 associationだけを追加する。Task-255
wrapper ownershipと全existing dense IDは不変。frozen pre-Task-256C1
baselineでは、このtarget edgeはseparate lower taskがunrelated overlap
rejectionを保持しながらauthenticated Task-255 condition containmentだけを
set/atomic両installation orderでexecutableにするまでgateされていた。Task
256C1は両orderをpassし、completed Task-257C2 implementationはfresh
preflight後にtarget edgeを現在publishする。

edgeは`source_formula_composition`のdedicated cross-family handoffで、
composite-formula placeholderではない。generator binding/capture、
predicate-chain composition、equality truth、formula result、definition
acceptanceはdownstreamに残す。

## Task 256C1 frozen compatibility edge

Task 256C1はgraph node/published edgeを追加しない。Task-256 validatorによる
already authenticated containment 1件の解釈だけを変更する。Task-255 term 0/
condition 0がsame owner-term/formula contextでTask-256 equality 0をencloseし
direct parentとなる。immutable
graphは`Task252 4/0/4 -> Task253 1/0/1/2/2 -> Task255
1/0/1/1/1/1/2`にdependency-neutral Task-256
`1/0/0/0/0/0/0/2/2`を加えたままで、Task-257C2がlater sole association
edgeを追加する。family ownership、ID、fingerprint、semantic boundaryは不変。

## Task 257C3 frozen cross-family edge

next graph sliceは
`Task252 3/0/3 -> Task256 1/0/2/2/2/0/0/3/2 -> Task257C3 1/1`。
Task 257C3はpre-existing boundary edge 1を介したsegments 0/1 association 1件と
negative segment 1 association 1件だけをownする。Task 252はprimary 1、
Task 256はshared edge/polarity/candidate/resolver provenanceを保持する。new
composite node/semantic formula ownerは導入しない。

## Task 257C3 implemented cross-family edge

frozen graph sliceは同じownershipでexecutableになった。Task 252は全primary
row、Task 256は全segment/head/candidate/edge/request/resolver-provenance
rowを保持し、Task 257C3はsyntax-free association 2件だけを追加する。
typed/resolved ownershipはA/B/C2とmutually exclusiveで、semantic formula
nodeは導入しない。

## Tasks 258A/258B1 frozen statement edges

Task 258はumbrella。Task 258Aはexact 81-byte
`FormulaStatementReservedVariableEqualitySmoke` transactionだけをownする。
resolver-authenticated theorem/label owner、theorem-proposition shell、
statement context、visible reserved-type-guard input、unverified
atomic-equality candidateが各1件 (`1/1/1/1/1`)。exact Task-252 `2/2/0`と
Task-256 `1/0/0/0/0/0/2/2`をfingerprintし、validated Task-48
`BindingEnv`のexact clone/fingerprintをownする。Task 248/Task 258A typed
ownerはproduction Task-248-first path、named reverse checker-test seam、
final assemblyでmutually exclusive。
checked formula、`statement_semantics` row、proof intent、accepted fact、
runner coverageは生成しない。

old Task-258B umbrellaはdecompose済み。Task 258B1はexact 139-byte nested
equality sliceだけをownする: Task-48 `3/1/0` proof-context extension、
Task-252 `8/8/0`、Task-256 `4/0/0/0/0/0/0/8/8`、statement/context/
guard/candidate 4 rows (`1/4/4/4/4`)、replay-authenticated proof-step
label/local citation association 1件 (`1/1`)をownし、node 68だけが
resolved/keyed reference siteとなるtwo-pass 77-node/root-76 resolver ASTで
backする。inner/outer conclusion shellとnested contextをtransportするが
fact/proof resultはpublishしない。

Task 258B2+はexplicit assumption/witness、composite root、broader
imported/outer/inner visibilityを保持する。Tasks 269-272がproof-local
declarationとproof/justification ownershipを保持する。complete dependency
chain/runnerがexecutableになるまでdeferred `MT10-FS` rowはdeferredのまま。

### Task 258B1 implementation closure

B1 statement/reference edgeはfrozenどおりtransport-only familyとして実装した。
次のstatement workはfresh contract review後のTask 258B2+であり、Tasks
269–272がlocal declaration、closure、coercion intent、proof skeleton、
justification meaning、goal、acceptanceを所有し続ける。本implementationで
family edgeをreclassifyしない。

### Task 258B2 frozen family edge

Task 258B2は次のminimal transport-only edgeである:
Task 48 `2/1/0` → Task 252 `6/6/0` → Task 256
`3/0/0/0/0/0/0/6/6` → profile `1/3/3/3/3`のsource-statement handoff
1件。single proof context下のunlabeled single assumptionとdirect conclusionを
transportする。Task-258B1 reference edge、およびfact、premise acceptance、
checked formula、statement semantic、proof、goal、theorem-result edgeは
意図的に存在しない。

Task 258B3はwitness transport、Task 258B4はcomposite theorem root、Task
258B5はbroader imported/outer/inner visibilityを保持する。Tasks 269–272は
proof-local declarationとproof/justification semanticsを保持する。したがって
deferred `MT10-FS` rowはdeferredのまま。本documentation prerequisiteは
executable coverage creditを得ず、source、fixture、sidecar、expectation、
trace status/count、既存test list/hashを変更しない。

### Task 258B2 implemented family edge

frozen single-assumption edgeはexecutableになったがtransport-onlyのまま。
direct equality theorem 1件、unlabeled assumption 1件、direct conclusion
1件だけで、reference/semantic handoffはない。B2 source/test gapをcloseするが、
B3 witness、B4 composite root、B5 broader visibility、Tasks 269–272
proof-semantic ownershipを消費しない。

### Task 258B3 witness companion

Task-258B3 familyはstatement transportを維持するが、異種payloadを分離する。
existing baseはsource ordinals 0/2のtheorem formula 1件/conclusion formula
1件をownする。new `SourceStatementWitnessHandoff`はその間のunnamed
primary-term witnessだけをsource ordinal 1/within-take ordinal 0でownする。
base/Task-252 fingerprintにdependし、authenticated pairとしてだけinstall
する。

このsplitによりterm-only `take` itemへfabricated formula、statement
context、guard、candidate fact、resolver bundleを与えない。Task 252が
witness term/referenceをownし、Task 256は明示的にexcludeする。Tasks
269–272はexistential matching、obligation、substitution、abbreviation、
proof stateを保持する。Tasks 258B3N/MはB4前のnamed/multiple/other
witness-term transport、B4/B5はcomposite-root/visibility familyを保持する。

Task 258B3はfrozen unnamed-primary witness companionだけをimplementした。
family partitionとB3N/M、B4/B5、269–272 ownershipはimplementation後も不変。

### Task 258B3N named-witness edge

B3Nはsyntax-only edgeをfreezeする。named primary-term witness 1件とdense
name row 1件である。bindingやabbreviationを作らずB3 transportをextendする。
Task 269だけがlater local binding、RHS link、capture-by-resolved-binding
abbreviation replay、context transitionを所有する。Task 272だけが
existential-binder matching、witness type obligation、capture-avoiding goal
substitution、remaining goalを所有する。Task 270は`deffunc`/`defpred`
closure、Task 271は`reconsider`のまま。B3Mはmultiple/other witness term、
B4/B5はroot/visibilityを保持する。

### Task 258B3N named-witness結果

named-primary edgeを1 witness row / 1 name rowのsyntax-only transportとして
実装した。binding/semantic edgeを追加せず、B3M、B4/B5、Tasks 269–272の
ownershipをconsumeしない。Task 258B3Mがnext dependency-ready documentation
prerequisiteである。

### Task 258B3M1 mixed multiple-witness edge

former B3M umbrellaをsplitする。B3M1はnamed primary term 2の後にunnamed
primary term 3が続くtwo-row syntax edge、one shared `take`、one dense name
row、shared source ordinal 1、within-`take` ordinals 0/1だけをownする。
Task 252は両reserved-variable referenceをownし、Task 256は両方をexclude
する。Task 269はname binding/abbreviation、Task 272はordered existential
goal effect、B3M2はnon-reserved-variable/other witness-term shapeを保持する。
B4/B5はcomposite root/visibilityを保持する。

### Task 258B3M1 implementation closure

exact reserved-variable mixed edgeはcomplete: named witness 0、unnamed
witness 1、name row 0はsyntax-only/dense。resolver-owned `y`、binding、
abbreviation、ordered goal effect、other witness-term shapesはすべてexclude
する。B3M2がB4前のnext dependencyとなる。

### Task 258B3M2A numeral-witness edge

B3M2をB3M2A/B3M2Bへsplitする。B3M2Aはexisting primary term 2がkind
`Numeral`、spelling `101`、Task-252 numeric request 0を持つone unnamed
witnessだけをownする。syntax-only witness row 1件を追加し、name row、
binding、atomic edge、semantic edgeは追加しない。numeral/requestはTask 252、
term 2 exclusionはTask 256、bindingなしはTask 269、typing/existential
matching/substitution/goal/proof effectはTask 272が保持する。B3M2B1は
exact parenthesized wrapperとreserved-variable childを保持し、B3M2B2は
compound、application、selector、update、set、choice、other
authority-valid witness shapeを保持する。`it`はChapter-13-valid `means`
contextだけ。B4/B5はB3M2B2までblocked。

### Task 258B3M2A implementation closure

private B3M2A profileはこのsyntax-only edgeだけをrealizeした。Task-252
numeral term 2 / numeric request 0をreuseし、Task-256 edges/requestsは
terms `0/1/3/4`だけをcoverし、one unnamed witness/no namesをbase
statement handoffとatomicにpublishする。binding、semantic edge、active
route、public schema、neighbor familyは変更していない。B3M2BがB4/B5前の
next unimplemented edge。

### Task 258B3M2B1 parenthesized-witness edge

B3M2B1はTask-252 parenthesized term 2 / child variable term 3上のone
syntax-only witness targetをownする。Task 252はparent edge/child-only
reference、Task 256はterms 2/3をexcludeして`[0,1]` / `[4,5]`だけ、
Task 258はwitness/take/base rows `1 witness / 0 names`だけをown。
Tasks 253–255にapplication/structure/selector/update/set/choice
payload/wrapper/edgeなし。Task 269はbindingなし、Task 272がall semantic
effect。B3M2B2はnested parentheses、application、structure
constructor/selector/update、set、choice、その他のauthority-valid witness
termを保持し、`it`はChapter-13-valid `means` definition/property
contextだけでeligible。B3M2B2がB4/B5前のnext。

### Task 258B3M2B1 implementation closure

private B3M2B1 profileはこのsyntax-only edgeだけを実装した。Task-252は
parenthesized wrapper、child reference、parent linkを保持し、Task-256は
`[0,1]` / `[4,5]`だけを保持する。Task 258はone unnamed outer-term
witness / zero namesをbase statementとatomicにpublishする。application、
structure、selector、update、set、choice、binding、semantic edge、active
route、public schema、neighbor familyは不変。B3M2B2がB4/B5前のnext。

### Task 258B3M2B2A nested-parenthesized witness edge

B3M2B2をsplitする。B3M2B2Aはtwo-level Task-252 parenthesized chain上の
one syntax-only witness targetだけをownする: outer term 2がinner term 3を
parentし、term 3がreserved-variable term 4をparentする。Task 252は三rowsと
child-only reference、Task 256はcomplete `2/3/4` subtreeをexcludeして
equalities `[0,1]` / `[5,6]`をownする。Task 258はone unnamed outer-term
witness/no namesとbase rowsだけをownする。Tasks 253–255にapplication、
structure、selector、update、set、choice、wrapper、cross-family edgeを
追加しない。Task 269はbindingなし、Task 272が全semantic effectを保持。
B3M2B2Bはapplication、structure constructor/selector/update、set、
choice、compound、other authority-valid witness termsを保持し、B4/B5は
B3M2B2B後。

### Task 258B3M2B2A implementation closure

private statement familyはexact two-level parenthesized witnessだけをownする。
outer/inner/leaf primary chainをone witness subtreeとしてauthenticateし、
全atomic edge/requestからexcludeする。application、structure constructor/
selector/update、set、choice、compound、deeper parenthesesはB3M2B2Bに
残り、cross-family edge/semantic ownerを追加しない。

### Task 258B3M2B2B1P lower application seam

B3M2B2Bをdependency-firstでsplitする。B1Pは既存Task-253 unwrapped
imported applicationを明示的proof contextでrebuildするprivate runner
capabilityだけをownする。Task-258 witness row、新payload family、public
schema、semantic edgeはownしない。B1Aはexact application-witness
cross-family edge、B1B+は他Task-253 forms、B2+/B3+はそれぞれTask-254/
Task-255 witness formsを保持する。

### Task 258B3M2B2B1P completion boundary

B1Pはprivate proof-context Task-253 reuse seamだけを提供する。new checker
family、Task-258 row、cross-family edgeはpublishしない。application-to-
witness edgeは次のB1A frozen contract/implementationが全てownし、B1B+/
B2+/B3+ ownershipはdeferredのまま。

### Task 258B3M2B2B1A frozen cross-family edge

B1Aは`SourceStatementWitness(0) -> SourceFunctorApplication(0)`のdirected
edge 1件だけを追加する。Task 252はnumeral argument primaries、Task 253は
imported infix application、Task 258はtake/witness associationをownする。
Task 256はtheorem/conclusion equalitiesだけをownしapplication fingerprint
から独立する。atomic TypedAst bundleがpartial/reverse edgeを防ぐ。
structure、set/choice/qualification、semantic term/formula/proof/goal
familiesは除外する。

### Task 258B3M2B2B1A implemented cross-family edge

frozen directed edge
`SourceStatementWitness(0) -> SourceFunctorApplication(0)`だけを実装した。
witnessは`Application(0)`とmatching optional application fingerprintを
保持し、applicationはTask 253、numeral argumentsはTask 252 ownershipの
まま。one atomic installerがapplication/statement/witness bundleを全て
publishするか何もpublishせず、final assemblyも同じvalidationをrepeat
する。reverse edge、lower row duplicate、wrapper ownership、Task-254/255
ownership、structure/set/choice/qualification edge、semantic/proof/goal
familyは追加していない。

### Task 258B3M2B2B1B1P wrapped Task-253 seam

B1B1Pは完全にTask-253 runner producer boundary内に留まる。payload familyも
cross-family edgeも追加しない。Task 252はnumeral primaries 2/3、Task
253はapplication 0とwrapper 0、Task 258はまだ何もownしない。future
B1B1 edgeは引き続き
`SourceStatementWitness(0) -> SourceFunctorApplication(0)`。wrapper 0は
authenticated containment metadataでありwitness targetではない。
Task-254/255と全semantic/proof/goal familiesは除外したまま。

### Task 258B3M2B2B1B1P completion boundary

implementationはprivate extraction/reuse seamだけを追加し、checker payload
family/cross-family edgeを追加しない。Task 252はprimaries 2/3、Task 253は
application/wrapper 0を引き続きownし、exact resolver authenticationは
別rowをpublishせずadmissionだけを狭める。Task 258はまだ何もownせず、
全statement/witness/semantic/proof/goal familiesはB1B1へdeferする。

### Task 258B3M2B2B1B1 frozen cross-family edge

B1B1はwrapped sourceにexisting edge shape
`SourceStatementWitness(0) -> SourceFunctorApplication(0)`だけを追加する。
Task 252はprimaries 2/3、Task 253はapplication 0、wrapper 0、
candidate、arguments、requests、Task 258はtake/witness pair 1件をown。
wrapper 0はauthenticated containmentだけで、reverse edge/witness target
ではない。Task 256はequality edges `[0,1]`、`[4,5]`だけを保持する。

public B1A schema/atomic three-handoff installerはB1Aをbroadenせずreuseする。
new profileはcrate-private。Task-254/255、structure/set/choice/
qualification、semantic term、proof、goal、Core/CFG/VC、全other familiesは
excludeする。

### Task 258B3M2B2B1B1 implemented cross-family edge

frozen witness-to-application edgeをprivate B1B1 profileへatomically
installした。ownershipは不変で、Task 252はprimaries、Task 253は
application/wrapper/candidate/requests、Task 258はtake/witness pairだけを
retainする。reverse wrapper edge、new payload family、semantic/proof/goal
edgeは追加していない。

### Task 258B3M2B2B2P frozen lower-family boundary

B2Pはpayload family/cross-family edgeを追加しない。existing Task-254 family
だけのprivate runner reuse seamをfreezeする。Task 254はconstructor 59と
assignment members 20/24をownし、qualified root 52はunowned provenance
traversalのまま。Task 252は54/57をprivate extraction rootsとしてのみ使用し、
numeral rowsを53/56でpublishして53/56を`source.term.numeral`としてownし、
54/57はarena-unownedのまま。B2Pでは
Task 258は何もownしない。future B2A witness-to-structure edge、§5.7配下の
future B2B selector family、B2C update/`FieldUpdate` familiesはseparate。
semantic term、proof、fact、goal、Core/CFG/VC、inheritance、typing、
defaults、coverage edgesはabsent。

### Task 258B3M2B2B2P implemented lower-family boundary

private B2P selector/reuse seamは、shared Task-252 primariesとfrozen
Task-254 constructor profileをinstallするが、payload family/cross-family
edgeは追加しない。Task 254はconstructor 59/members 20/24だけ、Task 252は
published numeral sites 53/56だけをownし、Task 258は何もownしない。
B2A witness-to-structure edgeが次で、B2B/B2Cと全semantic/proof/goal
edgesはdeferred。

### Task 258B3M2B2B2A frozen witness-to-structure edge

B2Aがfutureに追加するdirected edgeは
`SourceStatementWitness(0) -> SourceStructureTerm(0)`だけ。Task-258 base
transactionはtheorem/conclusion statement rows、B2A extensionは
take/witness occurrenceとedgeだけをownする。Task 254はconstructor/member/
request rowsをretainし、structure-root row 0はarena-unowned traversal
node 52をauthenticateする。Task 252はprimary children、Task 256はdirect
`Structure` target/structure fingerprintなしのequality formulas 2件だけを
retainする。reverse edge、wrapper target、field/member identity、
selector/update family、semantic/proof/goal edge、coverage creditは
authorizeしない。B2B/B2Cはseparate。

### Task 258B3M2B2B2A implemented witness-to-structure edge

exact directed `SourceStatementWitness(0) -> SourceStructureTerm(0)` edgeを
installした。Task 258はtheorem/conclusion base rowsとtake/witness 62/61
だけをownし、Task 254はconstructor/member/request、Task 252はprimary
childrenをretainする。Task 256はdirect structure target/fingerprintなしの
equality-onlyで、atomic typed/final boundaryだけでrevalidateする。

reverse edge、field/member identity、selector/update payload、semantic/
proof/goal edge、active route、coverage creditは追加していない。B2B
selectorとB2C update/`FieldUpdate`はseparate deferred families。

### Task 258B3M2B2B2BP frozen private selector lower edge

B2BPはTask-254 chain
`Structure(0 selector) -> Structure(1 constructor) -> Primary(2/3)`の
runner-private reuse pathだけをownする。Task 254はselector/constructor/
member/edge/request、Task 252はprimary valuesをretainし、Task 258はrow/
edgeを追加しない。

future seamはexact selector profileをexisting proof-context extractionへ
routeできるが、cross-family witness edge、public API、semantic edge、
coverage creditは追加しない。B2Bがlater direct-selector witness consumer、
B2Cがupdate/`FieldUpdate` owner。

private lower edgeはTask-258 rowを追加せずimplemented/testedとなった。
Task 254はselector/constructor/member/request rowsを、Task 252はprimary
valuesを引き続きownし、exact chainは
`Structure(0) -> Structure(1) -> Primary(2/3)`のまま。future witness
consumerはB2Bだけ。

### Task 258B3M2B2B2B frozen witness-to-selector edge

B2Bはdirected cross-family edge
`SourceStatementWitness(0) -> SourceStructureTerm(0)`をexact 1件だけ
追加する。Structure term 0はselector。Task 254はlower chain
`Structure(0 selector) -> Structure(1 constructor) -> Primary(2/3)`を
retainする。Task 258はtheorem/conclusion base rows、take/witness 65/64、
new directed edgeだけをownし、structure terms、root、member、primary
child、reverse edgeをownしない。

Task 252はprimaries、Task 254はselector/constructor/member/request rows、
Task 256はdirect structure target/fingerprintなしのequality-only
`BuiltinPredicateApplication` nodes 51/70をretainし、
`FormulaExpression` containers 52/71はunowned。witness handoffのexisting
fingerprintはdependency authenticationでsemantic edgeではない。B2C
update/`FieldUpdate`、selector identity/type/call/chain、semantic term、
proof、goal、Core/CFG/VC、coverage creditはabsentのまま。

### Task 258B3M2B2B2B implemented witness-to-selector edge

exact directed
`SourceStatementWitness(0) -> SourceStructureTerm(0)` edgeをprivate B2B
profileへinstallした。Task 258はtheorem/conclusion nodes `75/73`、
take/witness nodes `65/64`、そのedgeだけをownする。Task 254は
selector/constructor/member/request ownershipと
`Structure(0) -> Structure(1) -> Primary(2/3)`をretainし、Task 252は
primary rowsをretainする。

Task 256はequality-onlyでnodes `51/70`をownし、containers `52/71`は
arena-unownedのまま。B2A/B2Bはseparately authenticated atomic siblings
なので、target/fingerprint/ownership/lower-family hybridはpublication
なしでrejectする。reverse、selector-semantic、update/`FieldUpdate`、
proof、goal、Core/CFG/VC、active-route、coverage edgeは追加していない。

### Task 258B3M2B2B2CP frozen lower-family boundary

B2CPはTask-258 edgeを追加しない。existing Task-254 lower graphの
runner-private reuseだけをfreezeする:

```text
Structure(0 functional-update)
  -> UpdateBase -> Structure(1 constructor)
  -> UpdateValue(member 0) -> Primary(4)
Structure(1 constructor)
  -> ConstructorValue(member 1/2) -> Primary(2/3)
FieldUpdate(0) -> member 0
```

Task 252はprimary rows 7件、Task 254はupdate/constructor、members 3件、
non-term `FieldUpdate`、directed child edges 4件、unresolved requests
9件をretainする。B2CPはtheorem、statement、take、witness、formula、
reverse edge、typed/final rowをownしない。later B2C consumerだけがB2CP
implementation後にtake/witness nodes 72/71をownして
witness-to-`Structure(0)` edgeを追加できる。Task 256がlater ownするのは
equality nodes 55/77だけでupdate subtreeをexcludeし、formula
containers 56/78はunowned。functional-copy meaning、member identity、
replacement/result typing、proof/goal semantics、active routes、
coverage creditはabsent。

### Task 258B3M2B2B2CP implemented lower-family seam

CPC1 correction commit `ee267d9c`はcomplete。B2CPはexisting proof
context内でfrozen Task-254 functional-update/constructor/member/
`FieldUpdate` graphだけをprivately authenticate/re-publishし、
payload-family rowやupper edgeを追加しない。exact runner tests 2件が
PASSしたためprerequisite `design_drift`、bounded `source_drift`、
`test_gap`はclose。final test-sufficiency/implementation re-reviewsは
findingsなし。

Task 252/254 ownershipはunchanged。Task 256/258、B2C witness ownership、
public/active route、functional-copy/type/result meaning、proof/goal/theorem、
IRはdeferred。specification、corpus、fixture、expectation、sidecar、
trace status/count/backlink/creditは変更せず、formula rowは`deferred`、
`tests = []`、coverage audit impactはnarrative-only。concurrent ownershipは
report-only `repo_metadata_conflict`でmetadata repairなし。
fmt、Clippy、tests、全count/hash gatesはPASS。final source/documentation
re-reviewはfindingsなし。independent final qualityはfindingsなし、
全9 hard gates PASS、valid `98/100`。dedicated B2CP commit
`b146f0f72dceac2233c9d679b7820e264974b227`はcomplete。以下のB2C edgeが
post-commit next owner。

### Task 258B3M2B2B2C frozen witness-to-update edge

```text
formula(0) -> Primary(0/1)
formula(1) -> Primary(5/6)
witness(0) -> Structure(0 functional-update)
Structure(0) -> Structure(1 constructor)
Structure(0) -> Primary(4)
Structure(1) -> Primary(2/3)
```

Task252はsites `51/53/59/62/66/73/75`、Task254はupdate69、
constructor65、members30/20/24、`FieldUpdate`68と全lower edges/requests、
Task256はequalities55/77、Task-258 baseは82/80をretain。B2Cは72/71と
directed witness edgeのみをown。root58、private roots60/63/67、
containers56/78、transparent70、その他containersはunowned。

structure fingerprintはlower dependency authenticationでsemantic edgeでは
ない。reverse、identity/type、functional-copy、witness obligation、
proof/goal/theorem、active credit、IR edgeなし。checker tests 4件/runner
tests 5件はfuture `test_gap`; implementationはopenだが、4つの
documentation-prerequisite reviewsはすべてfindingsなし。

### Task 258B3M2B2B2C implemented witness-to-update edge

B2Cはfrozen family decompositionをchangeせずbounded source/test gapsを
closeした。new cross-family edgeは
`SourceStatementWitness(0) -> Structure(0)`だけ。Task254はupdate/
constructor/member/field-update graphを、Task256はequality 2 nodesだけを
引き続きownし、listed subtree containersはunownedのまま。reverse/
semantic edgeはない。

checker 4件/runner 5件のexact testsはhybrid/order、ownership、replay、
final clone、near miss、empty semanticsを含めPASS。final
test-sufficiency/implementation reviewsはfindingsなし。trace creditは
deferredのままで、broad verification、final consistency/quality、
commit gatesはpending。

### Task 258B3M2B2B2C broad family verification

broad fmt/Clippy/crate/workspace gates、focused `4/4`/`5/5`、sibling
`12/12`/`21/21` suitesはPASS。fresh counts/hashesはimplemented inventoryと
一致するため、sole B2C witness edgeとretain/excludeした全family boundaryは
exactのまま。trace creditはdeferredで、independent final consistency/
quality、commit/post-commit gatesはpending。

### Task 258B3M2B2B2C final family review status

independent final source/docs consistency/final qualityは**NO FINDINGS**。
全9 hard gates PASS、valid `98/100`。frozen family decomposition、evidence、
deferred trace statusはunchanged。pendingはcached-diff/staging audit、
implementation commit、post-commit inventory/fresh-next-task gatesだけ。

### Task 258B3M2B2B3P frozen lower set-term reuse

B2Cはimplementation commit
`e8373c683448e524cb98edde83fdf8de83a125cd`でcloseし、post-commit
worktree clean、ahead 8/behind 0、recorded stash unchanged。B3Pはupper
payload-family edgeを追加せず、proof context 1のlower graphだけをfreeze:

```text
SetTerm(0 Enumeration, node 40, 90..96)
  -> ordered Primary(2, node 36, 91..92)
  -> ordered Primary(3, node 38, 94..95)
  -> ResultType request(0)
```

Task 252が6 primaries、Task 255がset term 0だけをown。Tasks
253/254/256/258はemptyで、statement/witness/proof/theorem containersは
unowned。B3Pに`SourceStatementWitness -> SetTerm(0)` edgeはなく、upper
B3A ownership。result/sethood/element、existential、proof、goal、theorem、
Core/CFG/VC edgeもない。

missing contractはclosed `design_drift`、future private explicit-context
runner reuseは`source_drift`、compound runner tests 2件は`test_gap`。
public schema、active route、checker source/test、trace creditは変更しない。

上記2 arrowsはordinals 0/1の`EnumerationElement` edgesで、generic member/
expansion edgeではない。term/target fields、`ResultType` request、
Task252 primary fingerprint、absent application/structure fingerprintsを
field-for-fieldでfreeze。同じ2 testsがgraphをexhaustiveにauthenticateし、
Task111 literal hashes 3件を使う。

### Task 258B3M2B2B3P reviewed family status

documentation phaseの4 review tracksはすべて**NO FINDINGS**。
117-byte/hash、lint `15/14`、libraries `390/444`、source/test/CLI hashes、
exact scope、diff、trace no-op checksはPASS。prerequisite family/test
oracleはfrozenで、future private implementationはbounded
`source_drift`/`test_gap`をownする。final quality、commit、post-commit、
fresh implementation inventoryはpending。

### Task 258B3M2B2B3P final family quality

final qualityは**NO FINDINGS**、全9 hard gates PASS、valid `98/100`
（`20/20/15/14/10/10/5/4`）。family evidenceはunchanged。pendingは
stage/commit、post-commit、fresh implementation inventoryだけ。

### Task 258B3M2B2B3P implemented lower-family reuse

prerequisite `285a1f11c310bb313c4c6b4feae914eb11f74754`はprivate
explicit-context Task-255 seamとしてimplemented。proof context 1のexact
Task-252 numeral rootsがordered `EnumerationElement` 2 edgesへ流れ、
Tasks 253/254/256/258はempty。application/structure absenceはmissing
edgeの推測ではなくshared fingerprint-only subprofileでauthenticateする。

upper payload familyはunchangedで、`SourceStatementWitness -> SetTerm(0)`
edgeもstatement/proof/goal/semantic rowもない。test-sufficiency/
implementation reviewsは**NO FINDINGS**、B3P `source_drift`/`test_gap`
はclosed。次ownerはupper B3A。source/docs consistencyとdocumentation/
boundary repeatsは**NO FINDINGS**。lint-policy `15/14`、metadata `137`、
focused/library/fmt、workspace Clippy/tests、CLI/manifests/test-list hashes、
diff、exact30 scopeはPASS。independent final qualityは**NO FINDINGS**、
全9 hard gates PASS、valid `98/100`（`20/20/15/14/10/10/5/4`）。
pendingはcommit、post-commit、fresh B3A inventoryだけ。

### Task 258B3M2B2B3A frozen upper-family edge

B3Aはnodes `{42,43}`とsole directed
`SourceStatementWitness(0) -> SetTerm(0)` edgeをown。lowerはTask252
`{30,32,36,38,44,46}`、Task255 `{40}`、Task256 `{34,48}`、Task258
base `{51,53}`。unownedは
`0..29,31,33,35,37,39,41,45,47,49,50,52,54..56`。

full graphはformula `0 -> Primary(0/1)`、formula
`1 -> Primary(4/5)`、witness `0 -> SetTerm(0)`、
`SetTerm(0) -> Primary(2/3)`。reverse/cross-owner/semantic edgeなし。
set-shape/label/family hybrid/order near missesはpartial publicationなしで
fail closed。B4/B5とsemantic expansionはdeferred。

### Task 258B3M2B2B3A implemented upper-family edge

implementationはfrozen partitionとsole
`SourceStatementWitness(0) -> SetTerm(0)` edgeを実現し、Task-255
productionは変更しない。set-only fingerprint tuple、exact label/lower
provenance、atomic typed installation、final revalidation/cloneはfrozen
checker4+runner5 testsでcoverする。application/structure/multi-family
hybridは引き続きfail closedし、B4/B5と全semantic expansionはdeferred。
specification/test-sufficiency/implementation reviewsは**NO FINDINGS**。
2回目のsource/documentation consistency repeatとfinal documentation/
boundary rereadも**NO FINDINGS**で、crate plans記載のparent final
verificationはexact `39`-file scopeを含めPASS。independent final
read-only quality reviewは**NO FINDINGS**。全9 hard gates PASS、score
capなし、valid `98/100`（`20/20/15/14/10/10/5/4`）。記載済み
semantic/coverage deferralsはunchanged residual risk。pendingは
dedicated implementation commit、postcommit invariant verification、
fresh next-task inventoryだけ。

### Task 258B3M2B2B3B frozen zero-edge family boundary

post-B3A inventoryはlower Task-255 zero-edge capabilityとupper statement
acceptanceを区別する。B3Bはnode/range `33/95..97`にexactly 1件の
`Enumeration` SetTerm、wrapper/generator/type-site/condition/edge 0件、
proof context 1の`ResultType` request 1件をfreezeする。upper familyは
witness/take nodes `{35,36}`と
`Witness(0) -> SetTerm(0)`だけを追加する。

Task 252は`{27,29,37,39}`、Task 256は`{31,41}`、Task-258 baseは
`{44,46}`、Task 255は`{33}`をownする。empty enumerationはprimary childを
持たないため、directed graphはformula-to-primary edgesと1件の
witness-to-set edge以外を含まない。choice、comprehension、`qua`、
other enumeration cardinalities、semantic expansion、B4、B5はseparateの
ままである。missing upper contractは`design_drift`、future code/testsは
bounded `source_drift`/`test_gap`で、blocking authority gapはない。

## Task 258B3M2B2B3B implemented cross-family closure

B3B exact profileはB3Aのset target/fingerprint/APIをreuseし、Task-255の
zero-edge enumeration、Task-252/256/258 base rows、unnamed witness 1件
だけをcomposeする。initial reviewで不足したbidirectional family orders
とnon-vacuous zero-edge corruptionはexisting frozen tests内に追加し、
B3A/B3B、legacy/application/structure、other B3 profilesとのhybridを
fail closedにした。semantic expansion、remaining B3、B4/B5は引き続き
deferredである。

post-auth injectionとstage-prefix/non-generic-guard assertionsがlast
matrix gapsをcloseした。family ownershipまたはsemantic creditを変えず、
全test-sufficiency repeatsとfinal implementation repeatは
**NO FINDINGS**である。

## Task 258B3M2B2B3C choice-witness family

B3CはB3A/B3B enumeration後のdistinct Task-255 choice siblingである。
B3C ownershipはtake/witness `{38,37}`と
`Witness(0) -> SetTerm(0)` edgeだけ。Task 255はchoice/type nodes
`{35,34,33}`、`ChoiceTarget` type site 1件、ordered
`ChoiceNonempty`/`ResultType` requests、child edge 0をretainする。
Task 252は`{27,29,39,41}`、Task 256は`{31,43}`、Task 258 baseは
`{46,48}`をretainする。comprehension、`qua`、nonemptiness discharge、
generated choice semantics、B4/B5、proof acceptanceはseparate families。

### Task 258B3M2B2B3C implemented choice-witness edge

implementationはownershipをexactにrealizeする: Task-252
`{27,29,39,41}`、Task-255 `{33,34,35}`、Task-256 `{31,43}`、
Task-258 `{46,48}`、B3C `{37,38}`。choiceのTask-255 child edgeは0で、
upper edgeは`Witness(0) -> SetTerm(0)`だけ。全6 B3A/B3B/B3C
installation ordersはindependent exact familyとしてのみacceptし、
application/structure hybridとgeneric fallbackはatomicにfailする。choice
semantics、comprehension、`qua`、B4/B5、proof acceptanceはdeferred。
bounded replay/prefixとB3C-only route correction後のrepeat test/
implementation reviewsは**NO FINDINGS**。

### Task 258B3M2B2B3D frozen qua-witness edge

B3Dはremaining Task-255 set-family witnessで最小のものである:
`Qua` term 1件、term-owned `QuaTarget` builtin-set site 1件、
`QuaBase -> Primary(2)` edge 1件、ordered unresolved
`QuaWidening`/`ResultType`、upper witness-to-SetTerm edge 1件。
condition-free comprehensionはgenerator/sethood rowを追加するため後続と
する。B3D edgeはtransportのみであり、inheritance/cluster widening、
overload/coercion、result typing、proof acceptance、comprehension、
B4/B5、active creditはseparate ownersのままである。

### Task 258B3M2B2B3D implemented qua-witness edge

private exact routeはfrozen graphをrealizeする: Task-252
`{28,30,34,41,43}`、Task-255 `{35,36,37}`、Task-256 `{32,45}`、
Task-258 `{48,50}`、B3D `{39,40}`。Task-255
`SetTerm(0) -> Primary(2)` through `QuaBase`と、B3D
`Witness(0) -> SetTerm(0)`だけをpublishする。`QuaTarget`、ordered
unresolved requests、set-only fingerprintはunchanged lower producerと
existing upper APIからcomposeする。B3A/B3B/B3C/B3Dのpairingsと24
family ordersはindependent exact profilesとしてのみacceptし、hybrid、
stale、reordered、generic fallbackはatomicにfailする。

checker 4 + runner 5 testsと`32/70/44/72/62/21` matricesがedge/owner/
family isolationをcoverし、test-sufficiency reviewとindependent
implementation reviewは**NO FINDINGS**。inheritance/cluster widening、
overload/coercion、result typing、proof acceptance、comprehension、B4/B5、
active creditは引き続きdeferred。24-order/qua-edge wordingとreview-state
driftの同期修正後、source/docs consistencyとboundary repeatも
**NO FINDINGS**、final verificationもPASS。

independent final read-only quality reviewも**NO FINDINGS**、全9 hard
gates PASS、score capなし、valid `100/100`
（`20/20/15/15/10/10/5/5`）。CLI warnings `23`/errors `0`とlarge
repeated-test diff review volumeはnonblocking residual。残るのは
staging/cached diff、commit、post-commit/fresh-nextだけ。

### Task 258B3M2B2B3E frozen condition-free-comprehension witness edge

B3EはB3D qua witness後のnext Task-255 set-family siblingである。
Task-255は`Comprehension` term 1件、generator 1件、generator-owned
`BuiltinSet` type-site 1件、condition 0件、
`ComprehensionMapper -> Primary(2)` edge 1件、ordered unresolved
`GeneratorSethood`/`ResultType` requests 2件をretainする。B3Eはupper
`Witness(0) -> SetTerm(0)` edge 1件だけを追加する。

exact owner partitionはTask-252 `{32,34,38,47,49}`、Task-255
`{16,40,41,43}`、Task-256 `{36,51}`、Task-258 `{54,56}`、B3E
`{45,46}`である。generator container
`ComprehensionVariableSegment(42)`はunownedであり、generator identifier
node `16`はTask-255 source siteとしてのみownedとなる。これをTask-48
bindingまたはTask-252 referenceとしてreinterpretしてはならない。

directed graphはTask-256 formula-to-primary edges、
Task-255 `ComprehensionMapper -> Primary(2)`、B3E
`Witness(0) -> SetTerm(0)`だけである。exact matricesは
`32/70/53/72/62/21`。B3A/B3B/B3C/B3D/B3Eは全`120` install ordersで
independent exact profilesとしてのみfuture acceptanceされる。

generator BindingId/capture/name resolution、conditioned/multiple/nested/
generator-referencing comprehension、sethood/result/numeric typing、
existential/proof/goal semantics、B4/B5、active creditはseparate ownersに
deferする。B3Eはtransport-onlyであり、lower-stage prerequisiteまたは
semantic ownership expansionを導入しない。

### Task 258B3M2B2B3E implemented comprehension-witness edge

private exact routeはTask-255 producerを変更せずfrozen graphを実装する。
Task-252 `{32,34,38,47,49}`、Task-255 `{16,40,41,43}`、Task-256
`{36,51}`、Task-258 `{54,56}`、B3E `{45,46}`、segment `42`
unownedである。Task 255は`ComprehensionMapper -> Primary(2)`と
`GeneratorSethood`/`ResultType`を保持し、B3Eは
`Witness(0) -> SetTerm(0)`だけを追加する。

全120 orders、subtree exclusion、coherent same-provenance near misses、
`32/70/53/72/62/21`をcoverし、test/implementation reviewは
**NO FINDINGS**。binding/capture、condition/multiple/nested/
generator-reference semantics、sethood/type/proof/Core/CFG/VC、B4/B5、
coverage creditはdeferredである。

3件のbounded design correction後のfinal source/docs consistencyは
**NO FINDINGS**である。complete verificationはPASSし、independent final
qualityも**NO FINDINGS**、全9 gates PASS、capなし、valid `100/100`。
staging/post-commit gatesはimplementation commit
`e4479691db3b0a8785bb16e94d386bd71a394274`でcloseし、fresh inventoryは
Task 258B4Aをselectした。

### Task 258B4 composite-root decomposition

composite-root umbrellaは既存public lower consumerごとに分ける。

1. B4AはTask-257B1 explicit-universal compositionをconsumeする。
2. B4BはTask-257B2 connective/grouping rootsを保持する。
3. B4CはTask-257B3 restricted/existential/nested rootsを保持する。
4. B5はbroader imported/outer/inner visibilityを保持する。

B4Aが追加するのはprivate 80-byte/double-LF route上のupper
`Composite(0)` statement/candidate associationだけである。zero input-fact
profileによりexplicit binder/type/use transportはTask 257に残る。lower rowを
copyせず、lower `UnassignedStatement` ownershipをsemantic acceptanceへ
変換しない。repeated read-only documentation reviewは
**NO FINDINGS**である。independent final qualityは全9 hard gatesを
capなし、valid `100/100`でPASSした。remainingはstaging、commit、
post-commit inventoryだけである。

## Task 258B4A implemented composite-root edge

B4Aがcloseするのはtheorem statement 0/candidate 0からexisting Task-257B1
`Composite(0)`へのsyntax-free upper edgeだけである。Task 252は
primary/reference ownership、Task 256はatomic equality leaves、Task 257は
explicit binderとcomposite/composition graph、Task 258はstatement
ownershipを保持する。exact lower owned-site/range checksとrootless lower
typed arenaはownership transferなしにcoherent cross-family substitutionを
防ぐ。truth、binder guard discharge、facts、acceptance、proof semantics、
B4B/B4C/B5はdeferredのままである。

## Task 258B4B frozen composite-root edge

B4Bはsecond B4 nodeで、Task-257B2だけをconsumeする。unchanged lower
graphはTask 252 `16/0/16`、Task 256 `8/0/0/0/0/0/0/16/16`、
Task 257 `8/6/1/1/1/7/9`、Task-257B2 `8/0`である。そのexplicit
binderはunusedで、one rootは`UnassignedStatement`のままである。Task 258が
追加するのはstatement 0/candidate 0から`Composite(0)`へのassociation
だけで、owner/contextは1件、input factは0件、inner connective、
wrapper、equality、numeralへのedgeはない。

private 167-byte routeはactive 166-byte lower-only fixtureおよびB4A
80-byte routeからisolateする。B4A/B4B profile hybridはatomically failする。
B4CはTask-257B3 restricted/existential/nested rootsを引き続きownし、B5は
broader visibilityを引き続きownする。connective/repetition semantics、
truth、facts、acceptance、proofはdeferredのままである。

## Task 258B4C frozen composite-root edge

B4Cはexisting Task-257B3 restricted-universal、existential、
nested-quantifier、implicit-reserve graphだけをconsumeする。exact lower
profilesはbinding `4/4/0`、Task 252 `6/6/0`、Task 256
`3/0/0/0/0/0/0/6/6`、Task 257 `3/0/1/3/3/2/6`、Task-257B3
composition `3/6`である。lower-owned Surface sites 24件は
`{9,17,22,32,33,36,37,38,39,41,43,44,45,46,47,48,50,52,53,55,57,58,59,60}`
で、composite root 60は`UnassignedStatement`のままである。

Task 258が追加するのはtheorem node 62とstatement/candidateから
`Composite(0)`へのedgesだけである。context 0はreserved binding `[0]`を
見るが、reserveはprior statement factではなくbinder/type defaultを
供給するため、input-fact tableはemptyのままである。inner equality、
binder segment、referenceをtargetにするedgeはない。remaining Surface
nodes 41件はunownedのままである。

private 139-byte/double-LF routeはactive 138-byte/lower-only Task-257B3
sourceとdistinctである。separate lower selector prerequisiteはB4C
implementation前にexact one-/two-LF formsだけをadmitし、zero/three LFを
rejectする。upper dispatchはB1/B4A、B2/B4B、B3/B4Cだけをmatchし、全
hybridはatomically failする。quantifier truth、restriction discharge、
witness semantics、capture、implicit theorem closure、facts、theorem
acceptance、proof、Core/CFG/VC、B5はdeferredである。

## Task 258B5 decomposition と B5A frozen profile

B5 umbrellaをimplementation前にdecomposeする。

| task | owned edge | remaining prerequisite |
| --- | --- | --- |
| `258B5A` | `[0]`のlocal proof-step labelをlater descendant conclusion `[0,1]`がcite | this frozen contract |
| `258B5B` | imported public theorem visibility | imported-summary/public provenance contract |
| `258B5C` | active inner-to-outer/sibling rejection | test-first negative route/diagnostic contract |

B5Aはone theorem owner、five statement/context/input/candidate rows、one
private/local-only proof-step label、one simple-local citationを使う。five
formulaはTask-256 `Atomic(0..4)`で、全candidateは
`UnverifiedProposition`のまま。accepted fact/proof resultは作らない。
cross-family installはexact B1 base/reference pairまたはexact B5A
base/reference pairだけを認める。B5B/B5C、qualified/grouped/bulk
citation、fact、proof progress、theorem acceptance、IRはdeferredである。

## Task 258B5B imported-citation decomposition

B5Bはpositive imported-public-theorem profileだけをownする。B5Aに続くが、
このfrozen documentation prerequisite、two-file lower opt-in imported-label
prerequisite、seven-consumer upper implementationのthree commitsに分離する。
B5C active confinement negativeはseparateのまま。

upper profileはtheorem owner 1、statement/context/input/candidate rows 2、
local-label row 0、imported citation 1。Task-256 formulasは
`Atomic(0..1)`、both candidateは`UnverifiedProposition`。exact ownershipは
terms 4/formulas 2/statements 2（`8/49`）。cross-family installationはmatched
B1/B5A/B5B base/reference pairだけをadmitし、B5A local-label profileと
B5B imported provenanceはpairできない。

qualified/grouped/bulk import、private-import diagnostic、fact、proof
progress、truth、theorem acceptance/publication、status propagation、ATP、
Core、CFG、VCはdeferred。

## Task 258B5C active negative decomposition

B5Cはfourth checker reference profileではない。resolver failure
two件だけからなる。proof scope `[0,0]`でdeclareしたlabelはenclosing
`[0]` scopeからvisibleでなく、sibling `[0,1]`からもvisibleでない。各routeは
one private/local-only proof-step projectionとone unqualified
proof-or-theorem candidateを含み、exact `UnresolvedLabelRef`でterminateする。

workはdocumentation、resolver R-032A structural arena、resolver R-032B
proof-label collection、active declaration-symbol fixtures/runner/traceへ
decomposeし、各commitをseparateにする。R-032Aはsame-index structural
provenanceだけ、R-032Bだけがproof scope、module-global one-based completion
ordinal、canonical `proof-step-v1` origin、exact `CompactStatement`/
`ConclusionStatement`+justification/reference-chain candidateをestablishする。
checker base/reference pair、local/imported
citation target、label/citation row、typed installation、final cloneはない。
structure construction、selector access、functional/field update、
Tasks 252/253、ancestor B5A、imported B5B、B1へのcross-family edgeは
すべてempty。
qualified/grouped/bulk citation、public diagnostic code、proof discharge、
fact、acceptance、downstream IRはdeferred。

lower resolver familyはexact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock -> CompactStatement/ConclusionStatement`
allowlist、compact proposition-label inspection、direct statement proof/
justification child、sole simple-reference identifier chainへexplicitにclose
する。Root/CompilationUnit exact-one structural childとItemList direct-normal
theorem scanはmandatory。その他のsubtreeはno-row/no-ordinal/no-descentで、
positive upper/lower edge、negative missing/additional/wrong、direct Root/
Compilation relocation、`VisibleItem` wrapping、other forbidden-relocation/
mixed-list testはchecker ownershipより下位である。

env/module、derived namespace、exact one id-0 LocalSource record/source id、
全projection fieldのrunner provenance authenticationはseparate input-only
familyのまま。complete independent mutation matrixはchecker payload/
confinement resultを作れない。

## Task 259 Frozen Predicate-Definition Decomposition

Task 259はone predicate-definition row、two ordered parameter rows、one
guard row、one symmetry-property row、one correctness-condition rowへ
decomposeする。exact lower graphはTask 249 `2/2/0`、Task 252 `4/4/0`、
Task 256 `2/0/0/0/0/0/0/4/4`である。existing Task-248 handoffは独立した
one-block/two-parameter profile extension後だけ利用可能になる。
Tasks 253--255、257、258はrow/fingerprintを供給しない。

definitionはequality definiensを指し、各parameterはone `BindingId`とone
`SourceTypeApplicationId`を指し、guardはother equalityを指す。propertyは
owner、source site/order、`Symmetry` kind、explicit justification anchor
だけを指す。correctness rowはexactly one `Pending`
`InitialObligationKind::PredicatePropertyCorrectness`を指す。assumptionsは
emptyで、goal/provenanceはdeterministic opaque identityである。

runnerはpropertyをsame definition block内でpredicateより後にあるdirect
normal siblingとしてauthenticateする。resolver generic
Attribute/Attribute property projectionをsemantic evidenceとしてconsume
しない。Task 259はcomputation justificationをdescend/interpretせず、
Task 272がfuture proof/justification ownershipを保持する。Task 260はmixed
predicate-plus-functor gapをseparately retainする。

## Task 260 Functor-Definition Family

Task 260はdefinition/shared context parameter/shared guard/definiens/
correctness associationのseparate five-table familyです。definiensはexisting
Task-252/253/254/255/256 lower root 1件だけをpointし、rowをcopyしません。active
sourceはdefinition/definiens各2、Primary equals target 1、AtomicFormula means
target 1です。

explicit means clauseだけから`FunctorExistence`/`FunctorUniqueness`をappendし、
semantic goal/proof/acceptance/fact/VCを保存しません。Task 259/260は
cross-fingerprint/reinterpretせずTask 260ではmutually exclusiveです。mixed
coexistenceはseparate deferred ownerに残します。

## Task 249R definition-return lower family

Task 249はTask-260 sourceでbinding-owned application/expression/argument
`2/2/0`のままである。Task 249Rはsame immutable handoff内のdistinct owner-link
familyで、functor-definition owner site 2件がappended bare-set expression root
2/3を指し、combined `2/4/0/2`となる。Task 260だけがreturn ID 0/1をconsume
する。binding/normalized type/semantic association/goal/fact/obligation/proof/VC
rowはこのlower boundaryを越えない。

## Task 249M mode-RHS lower family

Task 249Mはexisting source-type handoff内のdistinct standalone owner-link
familyである。bare-set expression root 2をappendしmode-definition owner node 49を
`SourceTypeModeRhsId(0)`でlinkして、third binding application/definition-return
rowなしに`2/3/0/0/1`を作る。sole consumerはTask 262で、request/evidence/
expansion/acceptance/fact/proof/VC rowはlower boundaryを越えない。

## Task 249M active family inventory

distinct standalone owner-link familyはfrozen contractどおり実装済みである。
active handoffは`SourceTypeModeRhsId(0)` row 1件/root 2を持ち、Task-249R
definition-return familyとmutually exclusiveで、request/evidence/semantic/fact/
proof/IR/VC rowをpublishしない。
## Task 249S structure-member type lower family

Task 249Sはexisting immutable source-type handoff内のdistinct standalone
owner-link familyである。declaration-member owner site 4件がbare-set expression
root 4件を指し、binding application、definition-return row、mode-RHS rowなしに
`0/4/0/0/0/4`を作る。sole consumerはTask 263。member kind/identity、structure
parent/root/path/view、inheritance coverage、constructor/selector declaration、
coherence、request/evidence/semantics/fact/proof/IR/VC rowはこのlower boundaryを
crossしない。

## Task 249S active lower-family result

frozen standalone member-type familyはapplications/expressions/arguments/
definition-returns/mode-RHS/members `0/4/0/0/0/4`としてactiveになった。
所有するのはdeclaration-memberからtype-rootへのlink 4件だけである。
structure/member identity association、classification、inheritance、
coverage、constructor/selector、coherence request、runner/corpus consumerは
Task 263に残り、semantic familyはこのboundaryをcrossしない。

## Task 264 property-implementation family

Five tablesはimplementations/parameters/targets/definientia/correctnessである。
Targetはresolver property provenanceとTask249PI declared-return rowを所有するが
new resolver definitionではない。MeansはTask256+two obligations、Equalsは
Task254+zero obligationsである。Grammarにad-hoc assumeがないためguard tableは
存在しない。Task252/254/256 lower identityをassociationするだけで、coherence/
proof/acceptance/fact/VC familyやTask259 fingerprintとcrossしない。

## Task 249PI lower composition family

Task 249PIはpayload familyを追加しない。existing Task-249 application/expression ownerと
Task-249S member/expression ownerをexact property-source `1/3/0/0/0/2`でcomposeする。
field/propertyをclassifyせず、member row 1とproperty targetをassociateしない。それは
Task264がresolver authorityから行う。definition return、mode RHS、predicate、functor、
property semantics、obligation、fact、proof、acceptance、IR familyはmutually isolated。

## Task 249PI implemented composition boundary

exact application/member compositionはexisting source-type family内だけで実行可能に
なった。family/semantic payload/obligation/cross-family ownershipは追加せず、Task264が
最初のproperty consumerである。

## Task 264 implemented property-implementation family

frozen five-table familyはmeans/equals exact cardinality `1/1/1/1/2` / `1/1/1/1/0`
で実行可能になった。Task248P/249PI/252/254/256 handoffをauthenticateし、resolver
propertyとdeclared return rowをassociateし、meansにpending property existence/
uniqueness rowだけをappendする。Typed/final installationはatomicでTasks 259--263と
mutually exclusiveのまま。goal/guard composition、proof status、acceptance、fact/
property value、overlap/coherence、Core、CFG、VC familyは追加していない。

## Task 269A proof-local binding family

Task 269Aはimmutable Task-258B3N statement/witness/primary-term handoff上へ
declaration-to-binding association familyを1件追加する。sole rowはwitness
0/name 0/RHS primary 2をnew dense binding 1へlinkし、exact base-to-final
`BindingEnv` transitionをownする。name/witness/RHS arena nodeを再所有せず、
lower fingerprintを変更しない。

このfamilyはprimary-term use referenceと分離され、definition siteとfuture
visibilityを記録するだけでlater use/expansionではない。Task-272 witness
typing/goal effectおよびfact/proof/acceptance/IR/VC familyとも分離する。
Task 269B+がlater-use/capture replayを所有する。

## Task 269A active proof-local binding family

one-row definition-site familyを実装しTyped/final ownershipまで保存する。private
dormant consumerとunit test 8件はactive trace credit/later-use edgeを追加しない。
Task 269B+/270/271/272 ownershipは不変。

## Task 269B frozen B3M1 family increment

existing declaration-to-binding familyが2件目のexact lower profileをacceptするだけで
new payload familyではない。single rowはnamed witness0/name0/primary2をbinding1へ
linkし、sibling unnamed witnessはlower witness tableだけに残る。later-use edge、
capture、type/goal/fact/proof family、active coverage ownerは追加しない。

## Task 269B active B3M1 family increment

existing familyはsame one-row shapeで2件目のexact profileをacceptする。direct
final-environment/context assertionによりunnamed siblingがlower-onlyであることを
証明する。payload family、later-use edge、capture、type、goal、fact、proof、
coverage ownerは追加していない。

## Task 269CP isolated proof-`let` lower family

checker payload familyは追加しない。runner-private source/Surface/resolver
projection 1件のselectorがtheorem/proof/let/segment/name/bare-set Surface nodeを
authenticateし、outputはsource/module identity、source/Surface fingerprint、theorem
symbol/definition/contribution、source ordinal、role-specific range、local provenanceを
retainする。future Task 269Cがseparate checker let-binding familyをownする。
named-witness A/B、later-use/capture、source-type admission、goal/proof semantics、
active coverageはdisjoint。

## Task 269C isolated proof-`let` binding family

new checker siblingは`LetBinding` 1行とexact `BindingEnv` transitionだけをownする。
syntaxをimportせずTask-269CP provenanceをconsumeし、missing type siteをretainして
Typed/final ownerがone-shot preserveする。named-witness A/B、source-type application、
actual use/capture、formula/goal/fact/proof/obligation family、active coverageとはdisjoint。

このfamilyはexact declaration row 1件だけでimplementedとなりsibling payloadは0。
Typed/final replayとdormant runnerはdecompositionを保ち、separate source-type
prerequisiteは未実装のまま。

## Task 269CT proof-`let` source-type composition family

separate prerequisiteをunchanged Task-269C binding snapshot、typed `BindingEnv` overlay、
Task-249 source-type familyのexact compositeとしてfreezeする。bare builtin-`set`
application 2件だけをownし、新binding、use/capture、assumption/guard、goal、fact、proof、
obligation、IR、active coverage familyは追加しない。Task 269C/generic Task 249は不変。

## Task 269CT implemented family

composite familyをfrozen syntax-free boundaryで実装した。immutable Task-269C dependency、
separate typed binding overlay、bare builtin-`set` type application 2件、
source-preserved node 3件だけをownする。generic Task 249はproof-local `LetBinding`をrejectし、
semantic family/active coverage ownerは追加しない。

## Task 269GP proof-`given` lower family

Task269は未完了。runner-private 269GPはsyntaxだけをtransportする。canonical
Chapter-4/16 scope矛盾はdirect binding/type consumerの269G/269GTだけをhuman
reconciliationまでblockする。その後の`given` condition/escape semanticsは新しい
blocker classificationなしでseparately deferred。checker Task-269 familyとTask270
dependencyを変更せず、active creditは0。

implemented private lower rowはsyntax projection gapだけをcloseし、checker payload-
family memberを追加しない。

## Task 269GS family readiness

human-approved block-lifetime ruleは`given` binding/type familyの`spec_gap`を除くが、この
documentation taskはpayloadを作らない。Task269Gはexisting 269GP syntax rowのbinding-only
consumerをfreeze可能となり、Task269GTがlater type admissionを保持する。condition、label-
fact、goal、proof、obligation familyはexcludeのまま。
