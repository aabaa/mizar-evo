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
| `src/lib.rs` | 37 | crate boundary and public module exports | `00.crate_plan.md` and `source_spec_audit.md` | no | no | crate root として維持する。documented module と test-only determinism support だけを公開する。 |
| `src/typed_ast.rs` | 3805 | typed AST data model | `typed_ast.md` | no | no | typed-AST table、id、validation、rendering、test は大きいが cohesive。downstream 利用後の ergonomics を monitor する。 |
| `src/binding_env.rs` | 3095 | binding environment and resolver shell boundary | `binding_env.md` | no | no | cohesive な binding/context data layer。behavior-neutral split は不要。 |
| `src/source_context.rs` | 1063 | syntax-free source-item / binding-context producer | `source_context.md` | no | no | cohesive な Task-248 validation、table construction、recovery、handoff、boundary test。split不要。 |
| `src/source_attribute.rs` | 3074 | syntax-free source-attribute producer | `source_attribute.md` | no | no | cohesiveなTask-250 flat table、environment/parent/arena/provenance validation、deterministic rendering、corruption test。split不要。 |
| `src/source_evidence.rs` | 2413 | syntax-free source-evidence request/reference producer | `source_evidence.md` | no | no | cohesiveなTask-251 request/response table、upstream association、catalog/payload validation、deterministic rendering、corruption test。split不要。 |
| `src/source_term.rs` | 2207 | syntax-free source primary-term producer | `source_term.md` | no | no | cohesiveなTask-252 term/reference/request table、binding/parent validation、deterministic rendering、corruption test。split不要。 |
| `src/source_type.rs` | 3294 | syntax-free source-type application producer | `source_type.md` | no | no | cohesiveなTask-249 flat table、environment/arena/form/graph/provenance validation、deterministic rendering、exhaustive corruption test。split不要。 |
| `src/type_checker.rs` | 13235 | phase-6 type checking over checker-owned payloads | `type_checker.md` | no | no | 最大の file だが phase-6 spec boundary 内にある。normalization、reserve/authenticated exact theorem-owner handoff validation、declaration checking、inference、coercion、fact query、diagnostic、rendering、test は behavior-coupled。 |
| `src/registration_resolution.rs` | 5888 | phase-7 registration validation, activation, and existential gates | `registration_resolution.md` | no | no | cohesive な registration data layer と gate logic。behavior-neutral split は不要。 |
| `src/cluster_trace.rs` | 3948 | cluster closure and reduction trace recording | `cluster_trace.md` | no | no | cohesive な trace/replay module。behavior-neutral split は不要。 |
| `src/overload_resolution.rs` | 8004 | phase-8 overload pipeline | `overload_resolution.md` | no | no | overload collection、template expansion、viability、specificity、selection、rendering、test は大きいが cohesive。downstream 利用後の ergonomics を monitor する。 |
| `src/resolved_typed_ast.rs` | 6766 | final resolved typed AST assembly | `resolved_typed_ast.md` | no | no | exact Task-180 singleton statement/proof/direct-terminal validationとTask-251/252 clone-only handoffを含むcohesiveなfinal projection module。behavior-neutral splitは不要。 |
| `src/determinism_suite.rs` | 1101 | test-only cross-module determinism suite | `00.crate_plan.md` and `source_spec_audit.md` | no | no | private `#[cfg(test)]` crate support として維持する。 |
| `tests/lint_policy.rs` | 1818 | cross-cutting policy and audit guards | `source_spec_audit.md`, `bilingual_sync_audit.md`, and `module_boundary_audit.md` | no | no | 大きい support test だが repository-policy guardrail を意図的に集約している。task 34 の split は不要。 |

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

## Task 252 current-layout addendum

Task 252はcohesiveなpublic `source_term.rs` ownerを1件追加する。syntax-free
binding/typed-arena inputとcanonical `mizar_lexer::is_identifier` vocabulary
predicateだけをaccept/reuseし、raw syntaxをimportしない。raw `SurfaceAst`
traversalはprivate `mizar-test` leaf 1件に残る。term/reference/request
association、binding lookup、parent closure、numeric cardinality、rendering、
corruption testはbehavior-coupledでありprivate checker splitは不要である。
`TypedAst`がimmutable handoffをownし、`ResolvedTypedAst`はclone-onlyのままである。
