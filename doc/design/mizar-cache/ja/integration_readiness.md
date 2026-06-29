# Integration Readiness

> 正本は英語です。英語版:
> [../en/integration_readiness.md](../en/integration_readiness.md)。

Status: task 15 の readiness audit。owning downstream seam がまだ ready ではないため、
source integration は実装しない。

## 目的

task 15 は、`mizar-cache` を build scheduler、IR cache adapter、artifact
committed-publication token flow に接続できるかを評価する。これは gatekeeping task であり、
placeholder integration task ではない。

結果は deferred integration である。crate-owned cache surface は将来の consumer に向けて
利用可能だが、この task は scheduler hook、`mizar-ir` API、artifact publication-token
shortcut、proof status projection、publication authority を追加しない。

## 現在の cache surface

`mizar-cache` は現在、次を所有する。

- pure な `CacheKey` construction
- dependency slice と dependency-footprint fingerprint
- fail-closed な cache record / blob storage
- `mizar-proof` metadata に対する proof-reuse validation
- accepted-only cluster-db origin と in-memory import-scoped view

これらの surface は optimization input にすぎない。cache hit は validation predicate が
通った後でのみ work を skip してよい。proof acceptance、trusted `used_axioms`、artifact
publication、proof winner selection、scheduler policy、IR handle reconstruction authority は
提供しない。

## readiness inventory

| Integration | Classification | Evidence | Task-15 handling |
|---|---|---|---|
| `mizar-build` cache-aware scheduler | `external_dependency_gap` | `mizar-build` wave B の task-graph と scheduler task 7-10 は open であり、cache-aware scheduling task 18 も open である。task-18 seam は将来の driver-owned query boundary を通して消費される必要もある。 | `mizar-cache` に scheduler hook や cache scheduling trait を追加しない。将来の integration は `mizar-build` が seam を所有した後で既存 cache API を消費する。 |
| `mizar-ir` cache adapter | `external_dependency_gap` | `crates/mizar-ir` は存在しない。`doc/design/mizar-ir/en/todo.md` では scaffold task 1、cache-adapter spec task 9、cache-adapter implementation task 10 が open のままである。 | placeholder crate、mock adapter、rehydration API を作らない。将来の integration は `mizar-ir` が所有し、handle reconstruction 前に cache record を validate する。 |
| artifact committed publication token linkage | `external_dependency_gap` | `mizar-proof` は、`mizar-artifact` が artifact-owned committed publication proof token を公開するまで `CommittedWitnessPublicationProof` を opaque に保つ。artifact crate はその production token を公開していない。 | cache validation を artifact publication shortcut に接続しない。cache は witness、dependency-artifact、reuse metadata を比較してよいが、publication は artifact/proof owner に残る。 |

## deferred work

downstream owner が ready になった後の task は、cache hit が execution を skip しても clean
build と byte-identical な externally visible result を生むことを示す integration test を追加してよい。
その将来 task は引き続き次を証明しなければならない。

- cache deletion は performance だけを変える。
- cache hit/miss timing は diagnostics、artifact order、proof selection、proof acceptance、
  cache publication に影響しない。
- incomplete dependency footprint、不明な schema/toolchain/policy metadata、
  uncacheable marker、proof-reuse mismatch は fail closed になる。
- externally attested evidence と cache record は kernel-verified status や trusted
  `used_axioms` にならない。

それらの owner seam が存在するまでは、task 15 は `external_dependency_gap` を記録し、
no-stub boundary を保つことで完了する。

## verification

task 15 は documentation-only である。Rust source、manifest、test、`.miz` fixture、
expectation、traceability metadata は変更しない。verification は documentation review と
diff check である。
