# mizar-checker: Source / Binding Context Projection

> Canonical language: English. English canonical:
> [../en/source_context.md](../en/source_context.md).

## 目的と authority

`source_context` は [`00.crate_plan.md`](./00.crate_plan.md) で固定した
Task 248 source/binding-context producer を実装する。language authority は
Chapters 04 §4.3, §4.6、11 §11.2、12 §12.3, §12.7、15 §15.10 に限定する。
source-item order、resolver shell provenance、reserve と definition parameter の
distinct identity、local shadow、checker context link を保持する。

## Boundary

module は syntax-free projection のみ受け取る。opaque `DeclarationShellId` は
resolver の実 `DeclarationShellSet` から来なければならず、checker は shell
identity を生成せず `mizar-syntax` を import しない。`mizar-test` が bounded
`SurfaceAst` walk を所有し、source range、typed site、lexical scope、source
order、resolver-shaped `LocalTermBinding` provenance を供給する。

Task 248 が受理するのは named real-consumer transaction、すなわち module-level
`reserve x for set;` 1 item と、それに続く `x` という `set` local parameter
1件を持つ `definition` block 1件だけである。Vec-based input/table shape は
order と将来の extension seam を保持するが、この task は他の cardinality / role
combination を受理しない。canonicalなdistinct-name multiple-reserve inputを含む
additional reserve itemはvalid language shapeだが、このexact transaction外なので
`UnsupportedTaskShape`として拒否する。引用canonical specで未定義なのはsame
identifier再reserve時のreplacement/duplicate ruleだけであり、このnonblocking
`spec_gap`はhuman-reviewed authorityなしに意味を与えない。

type normalization、use-site resolution、RHS evaluation、fact/obligation 構築、
formula/proof verification、Tasks 249+/269+ は所有しない。Steps 6/7 は deferred
のままである。

## Projection model

- `SourceBindingContextInput` は source/module identity、module typed site、ordered
  item shell、ordered binding site を運ぶ。
- complete construction は checker-owned source-item/declaration table、1つの
  `BindingEnv`、exact binding/local-context link、local-context table を所有する
  immutable `SourceBindingContextHandoff` を生成する。
- `TypedAst` は source/module identity、local-context table 全体、item/declaration
  site、context link、module root owner が一致する場合だけ handoff を install する。
  `ResolvedTypedAst` は install 済み handoff を clone する経路しか持たない。
- reserve と local parameter は distinct checker id を保持し、local row は module
  reserve を structural shadow predecessor として記録する。

## Validation、recovery、atomicity

missing/duplicate/reordered row、stale ordinal、source/module/range mismatch、invalid
parent/context/site link、unsupported visibility、stale local provenance、wrong role、
duplicate local binder、partial payload は complete handoff 公開前に拒否する。
exact transactionでは両itemをtop levelに固定し、definition parameterにreserveと
同じspellingを要求するため、structural shadow linkは欠落しない。

recovered definition shell は binding をclaimしない場合だけ supported である。この
場合 producer は empty recovered context と deterministic internal diagnostic 1件を
持つ `SourceBindingContextIncomplete` を返す。binding を持つ recovered shell は
拒否する。incomplete/inconsistent data は `TypedAst` / `ResolvedTypedAst` に table
を一切 install しない。

## Determinism と coverage

dense id は validated source order に従う。同一 input は equal table と byte-identical
な nonempty debug text を生成し、reordered input は sort せず拒否する。
source-context handoff を持たない legacy `TypedAst` path は exact full-string debug
oracle を維持する。

real fixture `pass_type_elaboration_source_binding_context_shadowing_001.miz` は frontend、
resolver shell、producer、`TypedAst`、`ResolvedTypedAst` を通る。runner test はその
実 opaque shell id からだけ corruption input を再構築し、frozen
corruption/recovery/atomicity matrix をcoverする。later type/fact/obligation/formula/
statement/proof payload はすべて empty のままである。

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceItemRole` | `#[non_exhaustive]`; 後続 source-item role を許容する。 |
| `SourceItemVisibility` | `#[non_exhaustive]`; Task 248 は `Unspecified` だけ受理する。 |
| `SourceItemRecovery` | `#[non_exhaustive]`; 後続 recovery state を許容する。 |
| `SourceBindingContextOwner` | `#[non_exhaustive]`; 後続 owner form を許容する。 |
| `SourceBindingSiteRole` | `#[non_exhaustive]`; 後続 binding role を許容する。 |
| `SourceBindingContextBuild` | `#[non_exhaustive]`; complete/incomplete result を区別する。 |
| `SourceContextError` | `#[non_exhaustive]`; validation failure を exhaustive match しない。 |

この module が所有する exhaustive public enum exception はない。

## Task 248 classification

| Class | Result |
|---|---|
| `test_gap` | exact named source/binding-context slice についてだけ closed。broader canonical producer shapeはexisting MC-G011/MC-G016 follow-up ownerに残る。 |
| `source_drift` | real shell-to-checker handoff と immutable final projection でrepair。 |
| `design_drift` | module spec、paired audit、plan、todo、harness record を同期。 |
| `boundary_violation` | current violation なし。shell fabrication と syntax import は禁止。 |
| `spec_gap` | same-identifier re-reservationのreplacement/duplicate semanticsだけが未定義。このnonblocking gapは実装authorityを与えない。 |
| `repo_metadata_conflict` | 未検出。 |
