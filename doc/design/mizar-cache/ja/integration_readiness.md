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
| `mizar-build` cache-aware scheduler | `external_dependency_gap` | `mizar-build` は現在 task-graph、scheduler、cache-aware decision、scheduler-selected dispatch seam を所有する。end-to-end cache lookup/use は、cache authority を scheduler に移さず `mizar-cache` を呼ぶ owner-scoped build/driver integration task をまだ必要とする。 | `mizar-cache` に scheduler hook や driver-owned cache scheduling trait を追加しない。将来の integration は owning build/driver path から既存 cache API を消費する。 |
| `mizar-ir` cache adapter | `external_dependency_gap` | `mizar-ir` は現在存在し、cache-adapter の validation-before-rehydration boundary を所有する。この cache milestone では build/driver execution を通じた end-to-end rehydration は未配線である。 | IR handle reconstruction や driver orchestration を `mizar-cache` に移さない。将来の integration は owner path が所有し、handle reconstruction 前に cache record を validate する。 |
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

それらの owner integration task が既存 seam を結線するまでは、task 15 は
`external_dependency_gap` を記録し、no-stub boundary を保つことで完了する。

## verification

task 15 は documentation-only である。Rust source、manifest、test、`.miz` fixture、
expectation、traceability metadata は変更しない。verification は documentation review と
diff check である。
