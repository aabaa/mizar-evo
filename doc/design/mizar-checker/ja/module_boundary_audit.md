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
| `src/lib.rs` | 32 | crate boundary and public module exports | `00.crate_plan.md` and `source_spec_audit.md` | no | no | crate root として維持する。documented module と test-only determinism support だけを公開する。 |
| `src/typed_ast.rs` | 3527 | typed AST data model | `typed_ast.md` | no | no | typed-AST table、id、validation、rendering、test は大きいが cohesive。downstream 利用後の ergonomics を monitor する。 |
| `src/binding_env.rs` | 3090 | binding environment and resolver shell boundary | `binding_env.md` | no | no | cohesive な binding/context data layer。behavior-neutral split は不要。 |
| `src/type_checker.rs` | 10238 | phase-6 type checking over checker-owned payloads | `type_checker.md` | no | no | 最大の file だが phase-6 spec boundary 内にある。normalization、reserve source handoff production、declaration checking、inference、coercion、fact query、diagnostic、rendering、test は behavior-coupled なので、review friction が具体化した場合だけ focused private-layout task で後続 split する。 |
| `src/registration_resolution.rs` | 5888 | phase-7 registration validation, activation, and existential gates | `registration_resolution.md` | no | no | cohesive な registration data layer と gate logic。behavior-neutral split は不要。 |
| `src/cluster_trace.rs` | 3948 | cluster closure and reduction trace recording | `cluster_trace.md` | no | no | cohesive な trace/replay module。behavior-neutral split は不要。 |
| `src/overload_resolution.rs` | 8004 | phase-8 overload pipeline | `overload_resolution.md` | no | no | overload collection、template expansion、viability、specificity、selection、rendering、test は大きいが cohesive。downstream 利用後の ergonomics を monitor する。 |
| `src/resolved_typed_ast.rs` | 3728 | final resolved typed AST assembly | `resolved_typed_ast.md` | no | no | cohesive な final projection module。behavior-neutral split は不要。 |
| `src/determinism_suite.rs` | 1096 | test-only cross-module determinism suite | `00.crate_plan.md` and `source_spec_audit.md` | no | no | private `#[cfg(test)]` crate support として維持する。 |
| `tests/lint_policy.rs` | 1786 | cross-cutting policy and audit guards | `source_spec_audit.md`, `bilingual_sync_audit.md`, and `module_boundary_audit.md` | no | no | 大きい support test だが repository-policy guardrail を意図的に集約している。task 34 の split は不要。 |

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
