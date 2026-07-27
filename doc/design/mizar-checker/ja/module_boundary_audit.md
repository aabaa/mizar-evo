# Module Boundary Audit: mizar-checker

> 正本は英語です。英語版:
> [../en/module_boundary_audit.md](../en/module_boundary_audit.md)。

task 34 は、downstream crate が checker boundary を消費する前に、現在の
`mizar-checker` source layout を分割すべきか監査する。これは layout gate
だけであり、checker source behavior、public API、diagnostic、deterministic
rendering、artifact-facing schema、`.miz` fixture、expectation は変更しない。

## Split Gate

behavior-neutral private module split が必要になるのは、checker-owned file が
すでに所有済みの module boundary 内で具体的な layout / review bottleneck を
作っている場合だけである。crate ownership violation、undocumented public API、
behavior drift、API exposure、diagnostic change、schema change は layout fix ではない。
それらは autonomous crate protocol 上の hard-gate finding であり、修正するか、
owner 付きで defer するか、独立した specification task へ移す必要がある。task 34
は file move によってそれらを隠してはならない。

大きくても cohesive な file は、public surface、diagnostic、deterministic rendering、
module ownership が owning specification と揃っている場合、monitored ergonomics
note として記録する。

## Source Layout Inventory

| Path | Lines | Boundary label | Owning specification | Split required | Hard-gate finding | Decision |
|---|---:|---|---|---|---|---|
| `src/lib.rs` | 43 | crate boundary and public module exports | `00.crate_plan.md` and `source_spec_audit.md` | no | no | Task 257B1 documented syntax-free formula-composition moduleをexportするcrate rootとして維持。 |
| `src/typed_ast.rs` | 4117 | typed AST data model | `typed_ast.md` | no | no | Task-253/254/255/256 bidirectional install、Task-257A one-shot install、Task-257B1/B2/B3 combined installを含むcohesive owner。 |
| `src/binding_env.rs` | 3143 | binding environment and resolver shell boundary | `binding_env.md` | no | no | source-formula context identityを含むcohesiveなbinding/context data layer。behavior-neutral splitは不要。 |
| `src/source_context.rs` | 1150 | syntax-free source-item / binding-context producer | `source_context.md` | no | no | cohesive な Task-248 validation、table construction、recovery、handoff、boundary test。split不要。 |
| `src/source_atomic_formula.rs` | 7428 | syntax-free source atomic-formula producer | `source_atomic_formula.md` | no | no | cohesiveなTask-256/257C1 nine-table association、resolver provenance、predicate-segment/shared-boundary validation、cross-family ownership/fingerprint validation、deterministic rendering、install check、compatibility literal。split不要。 |
| `src/source_composite_formula.rs` | 4700 | syntax-free source composite-formula/binder producer | `source_composite_formula.md` | no | no | exact Task-257A/B1/B2/B3 profiles、binding extension、wrapper/tree validation、rendering/install/corruption/profile testsを持つcohesive owner。 |
| `src/source_formula_composition.rs` | 3117 | syntax-free cross-family formula composition producer | `source_formula_composition.md` | no | no | Task-257B1/B2/B3 atomic-edge/bound-use associationとTask-257C1 empty-segment compatibility、dependency fingerprint、rendering/install/corruption testsを持つcohesive owner。 |
| `src/source_attribute.rs` | 3074 | syntax-free source-attribute producer | `source_attribute.md` | no | no | cohesiveなTask-250 flat table、environment/parent/arena/provenance validation、deterministic rendering、corruption test。split不要。 |
| `src/source_evidence.rs` | 2413 | syntax-free source-evidence request/reference producer | `source_evidence.md` | no | no | cohesiveなTask-251 request/response table、upstream association、catalog/payload validation、deterministic rendering、corruption test。split不要。 |
| `src/source_term.rs` | 2207 | syntax-free source primary-term producer | `source_term.md` | no | no | cohesiveなTask-252 term/reference/request table、binding/parent validation、deterministic rendering、corruption test。split不要。 |
| `src/source_application.rs` | 4001 | syntax-free source functor-application producer | `source_application.md` | no | no | cohesiveなTask-253 application/wrapper/candidate/argument/request table、dependency/provenance validation、deterministic rendering、corruption test。split不要。 |
| `src/source_set_term.rs` | 6806 | syntax-free source set-term producer | `source_set_term.md` | no | no | cohesiveなTask-255/255C1 seven-table association、condition-subtree exclusion、cross-family ownership/fingerprint validation、deterministic rendering、install check、corruption test。split不要。 |
| `src/source_structure.rs` | 5036 | syntax-free source structure-term producer | `source_structure.md` | no | no | cohesiveなTask-254 term/wrapper/root/member/field-update/edge/request table、written-partition/cross-family dependency/provenance validation、deterministic rendering、corruption test。split不要。 |
| `src/source_type.rs` | 3294 | syntax-free source-type application producer | `source_type.md` | no | no | cohesiveなTask-249 flat table、environment/arena/form/graph/provenance validation、deterministic rendering、exhaustive corruption test。split不要。 |
| `src/type_checker.rs` | 13235 | phase-6 type checking over checker-owned payloads | `type_checker.md` | no | no | 最大の file だが phase-6 spec boundary 内にある。normalization、reserve/authenticated exact theorem-owner handoff validation、declaration checking、inference、coercion、fact query、diagnostic、rendering、test は behavior-coupled。 |
| `src/registration_resolution.rs` | 5888 | phase-7 registration validation, activation, and existential gates | `registration_resolution.md` | no | no | cohesive な registration data layer と gate logic。behavior-neutral split は不要。 |
| `src/cluster_trace.rs` | 3948 | cluster closure and reduction trace recording | `cluster_trace.md` | no | no | cohesive な trace/replay module。behavior-neutral split は不要。 |
| `src/overload_resolution.rs` | 8004 | phase-8 overload pipeline | `overload_resolution.md` | no | no | overload collection、template expansion、viability、specificity、selection、rendering、test は大きいが cohesive。downstream 利用後の ergonomics を monitor する。 |
| `src/resolved_typed_ast.rs` | 6950 | final resolved typed AST assembly | `resolved_typed_ast.md` | no | no | Task-251/252/253/254/255/256/257A/257B1/B2/B3 clone-preserving handoffを含むcohesive final projection module。 |
| `src/determinism_suite.rs` | 1101 | test-only cross-module determinism suite | `00.crate_plan.md` and `source_spec_audit.md` | no | no | private `#[cfg(test)]` crate support として維持する。 |
| `tests/lint_policy.rs` | 1846 | cross-cutting policy and audit guards | `source_spec_audit.md`, `bilingual_sync_audit.md`, and `module_boundary_audit.md` | no | no | 大きい support test だが repository-policy guardrail を意図的に集約している。task 34 の split は不要。 |

## Task 34 Classification

| Class | Evidence | Action |
|---|---|---|
| `spec_gap` | この audit は language specification behavior を変更しない。 | spec edit なし。 |
| `test_gap` | task は source-layout gate である。実行可能 coverage は本 audit table と既存 source/spec・bilingual guard を検査する lint-policy guard。 | `.miz` fixture は追加しない。 |
| `design_drift` | crate plan、TODO、source/spec audit、bilingual audit、本 layout audit は現在の source file について同期済み。 | task 34 completion を記録し、future audit drift を guard する。 |
| `source_drift` | Source behavior は変更しない。現在の evidence では file move や private split は不要。 | lint-policy test 以外の source/API edit はない。 |
| `source_undocumented_behavior` | task 32 の guard が public source/spec correspondence を引き続き cover する。task 34 は新しい undocumented public API を見つけていない。 | future public surface drift は hard gate のままで、split trigger ではない。 |
| `boundary_violation` | 現在の public module は internal 07 と module spec が述べる checker ownership boundary 内にある。 | boundary repair / deferral なし。 |
| `external_dependency_gap` | 新規なし。既存 checker external gap は crate plan と source/spec audit に記録済み。 | 新規 deferral なし。 |
| `deferred` | task 34 では必須の behavior-neutral module split を defer しない。大きい cohesive file は monitored ergonomics note のみ。 | future split work は、独立 review / commit を持つ behavior-neutral private-layout task とする。 |

## Completion Decision

task 34 は、この English audit と Japanese companion、crate plan / todo update、
source/spec audit と bilingual audit の更新、lint-policy module-boundary guard が
同じ commit に含まれた時点で完了する。task 34 単体では crate completion を主張しない。
closeout task は crate exit report をすでに記録しており、その report が read-only
quality review result を記録している。

## Task 266 current-layout addendum

Task 266 は既存 checker ownership boundary 内に留まり module split は不要。
resolver-global owner validation は `type_checker.rs` に置き、
`resolved_typed_ast.rs` は checker-owned owner/binding/inference/typed-AST
payload だけを消費する。boundary lint は final projection module の
`SymbolEnv` / `mizar_resolve::env` scan を禁止し、通過する。

## Task 250 current-layout addendum

Task 250はcohesive public `source_attribute.rs` ownerを1件追加する。raw syntaxは
private `mizar-test` leafに残り、checker moduleはsyntax-free Task-249、
binding、symbol、typed-arena dependencyだけを受け取る。five-table data model、
validation、construction、rendering、corruption testはbehavior-coupledなので
private splitは不要。`TypedAst`がimmutable handoffをownし、
`ResolvedTypedAst`はclone-onlyのままである。

## Task 251 current-layout addendum

Task 251はcohesiveなpublic `source_evidence.rs` ownerを1件追加する。syntax-free
Task-249/250 handoff、resolver identity、checker fact/gate、dependency recordだけを
acceptし、raw syntaxは`mizar-test`に残る。request/response association、
state/cardinality validation、catalog/payload authentication、deterministic
rendering、corruption matrixはbehavior-coupledでありprivate splitは不要である。
`TypedAst`がimmutable handoffをownし、`ResolvedTypedAst`はclone-onlyのままである。

## Task 253 current-layout addendum

Task 253はcohesiveなpublic `source_application.rs` ownerを1件追加する。syntax-free
resolver/binding/Task-252/typed-arena inputだけをacceptし、raw syntaxはprivate
`mizar-test` leafに残る。five-table association、application/wrapper geometry、
root-only/cross-application ownership、candidate provenance、unresolved request、
exact dependency fingerprint、rendering、corruption testはbehavior-coupledであり、
private checker splitは不要である。`TypedAst`がone-shot immutable handoffをownし、
`ResolvedTypedAst`はrevalidate後にclone-preserveする。

## Task 252 current-layout addendum

Task 252はcohesiveなpublic `source_term.rs` ownerを1件追加する。syntax-free
binding/typed-arena inputとcanonical `mizar_lexer::is_identifier` vocabulary
predicateだけをaccept/reuseし、raw syntaxをimportしない。raw `SurfaceAst`
traversalはprivate `mizar-test` leaf 1件に残る。term/reference/request
association、binding lookup、parent closure、numeric cardinality、rendering、
corruption testはbehavior-coupledでありprivate checker splitは不要である。
`TypedAst`がimmutable handoffをownし、`ResolvedTypedAst`はclone-onlyのままである。

## Task 254 current-layout addendum

Task 254はcohesiveな5,036-line public `source_structure.rs` ownerを1件追加する。
syntax-free resolver/binding/Task-252/253/typed-arena inputだけをacceptし、raw
syntaxとTask-248 source-context extractionはprivate `mizar-test` leafに残る。
seven-table association、constructor provenance、member/`FieldUpdate` geometry、
cross-family root ownership、conditional fingerprint、rendering、corruption testは
behavior-coupledでありprivate checker splitは不要である。`TypedAst`がone-shot
immutable handoffをownし、`ResolvedTypedAst`はrevalidate後にclone-preserveする。

## Task 256C1 frozen boundary recheck

fresh inventoryでは`source_atomic_formula.rs` 7,428 lines、
`source_set_term.rs` 6,806、`typed_ast.rs` 4,117、
`resolved_typed_ast.rs` 6,950。Task 256C1が変更するのは最初のcohesive
ownerだけで、private range predicate/checker-local testsはexisting nine-table
validation matrixとbehavior-coupledである。new module、public API、runner
owner、cross-crate dependencyは不要。`TypedAst`、`source_set_term`、
resolved ownership、全mizar-test production pathは不変。

## Checker Task 257C2 frozen boundary

Task 257C2はexisting `source_formula_composition.rs` owner内に留めるが、
Task-257B composite/bound-use tableへ混在させずseparate condition-formula
transactionをfreezeする。checker inputはsyntax-free Task-252/253/255/256
handoffと`TypedArena`だけに限定し、raw AST selection、loaded-source guard、
parser/resolver inspectionはprivate `mizar-test` leafに残す。new associationは
site/semantic resultをownしない。本prerequisiteはproduction不変なので、
measured 3,117-line moduleとcurrent boundary-table countはすべて不変。
implementationはediting前に再測定し、その後本auditを再実行する。fresh
preflightで`source_atomic_formula.rs`内にseparate Task-256
condition-container compatibility `source_drift`を確認した。そのdedicated
Task-256C1 documentation/implementation commitはlower module ownershipを保持し、
両lower-handoff installation orderをpassさせた後に本moduleをeditする。

## Task 255C1 current-layout addendum

checker production pathは追加していない。`source_set_term.rs`は6,806-lineの
cohesive ownerとしてTask-255 table 7件、recursive condition-subtree boundary、
cross-family partition、fingerprint、rendering、installation、focused matrixを
保持する。compatibility literal 6件を加えた`source_atomic_formula.rs`は7,428
linesである。public module splitは適切なままで、raw syntax/semantic formula
ownershipはcheckerへ入らない。

## Checker Task 257C1 boundary recheck

checker production pathは追加していない。7,422-lineとなった
`source_atomic_formula.rs`はextended nine-table transactionのcohesiveな
syntax-free ownerであり続ける。segment/head/candidate/edge/request
association、polarity-token authentication、shared-boundary validation、
dependency fingerprint、rendering、install revalidation、rollbackは
behavior-coupledである。raw parsing/exact-source selectionはprivate
`mizar-test`に留まり、checker splitは不要。`TypedAst`/`ResolvedTypedAst`は
既存publication boundaryを維持する。

## Checker Task 257B3 boundary recheck

checker production pathは追加しない。fourth profileはcohesiveなcomposite/
composition owner内、exact parser/resolver extractionはprivate `mizar-test`
内に残す。debug-oracle text asset 2件はtest-onlyで、このbounded extensionに
新module splitは不要。

## Task 257B2 Boundary Delta

checker moduleは追加しない。`source_composite_formula`が5 frozen connective
kinds、4 same-family roles、6-wrapper validation、third profileをownし、
`source_formula_composition`が4 atomic rolesとexact `8/0` tableをownする。
`TypedAst`/`ResolvedTypedAst`は既存combined publication boundaryを維持し、
runner extractionも既存private leafに留める。

## Task 257B1 current-layout addendum

Task 257B1はcohesiveなpublic `source_formula_composition.rs` owner 1件と、
`source_composite_formula.rs`へのbounded second-profile extensionを追加する。
両者はsyntax-free Task-252/256/257/typed-arena dependencyだけをacceptし、raw
formula syntaxはprivate `mizar-test` leafに残る。atomic-edge/bound-use
association、dependency fingerprint、combined install、deterministic rendering、
corruption testはbehavior-coupledでprivate checker splitは不要である。

final line countは`lib.rs` 43、`typed_ast.rs` 4,110、
`source_composite_formula.rs` 2,913、`source_formula_composition.rs` 1,475、
`resolved_typed_ast.rs` 6,949、`tests/lint_policy.rs` 1,846である。

## Task 257A current-layout addendum

Task 257Aはcohesiveな2,790-line public `source_composite_formula.rs` ownerを
1件追加する。syntax-free binding/typed-arena inputだけをacceptし、raw formula
syntaxはprivate `mizar-test` leafに残る。seven-table association、
source-derived `2/1/4` binding extension、tree/context/binder/type validation、
install revalidation、deterministic rendering、real/synthetic/corruption/
exclusion matrixはbehavior-coupledでありprivate checker splitは不要である。
`TypedAst`がone-shot immutable handoffをownし、`ResolvedTypedAst`はrevalidate後に
clone-preserveする。

## Task 255 current-layout addendum

Task 255はcohesiveな5,547-line public `source_set_term.rs` ownerを1件追加する。
syntax-free binding/Task-252/253/254/typed-arena inputだけをacceptし、raw
syntaxはprivate `mizar-test` leafに残る。six-table association、canonical
spelling/cardinality、nearest-family ownership、conditional fingerprint、
install revalidation、rendering、corruption matrixはbehavior-coupledであり、
private checker splitは不要である。`TypedAst`がone-shot immutable handoffをownし、
`ResolvedTypedAst`はrevalidate後にclone-preserveする。

## Task 256 current-layout addendum

Task 256はcohesiveな6,414-line public `source_atomic_formula.rs` ownerを1件追加
する。syntax-free resolver/binding/Task-252/253/254/255/typed-arena inputだけを
acceptし、raw formula syntaxはprivate `mizar-test` leafに残る。eight-table
association、predicate/attribute provenance、bare asserted-type ownership、
nearest-family cross-family edge、conditional fingerprint、install
revalidation、rendering、real/synthetic/corruption/exclusion matrixは
behavior-coupledでありprivate checker splitは不要である。`TypedAst`がone-shot
immutable handoffをownし、`ResolvedTypedAst`はrevalidate後にclone-preserveする。
