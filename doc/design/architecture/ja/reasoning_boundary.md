# Architecture: Reasoning Boundary

> Canonical language: English. English canonical version: [../en/reasoning_boundary.md](../en/reasoning_boundary.md).

## 目的

この文書は、Mizar Evo における reasoning responsibility boundary を定義する。すなわち、どの check を Mizar 側 verifier が実行し、どの obligation を外部 ATP/SMT backend に委譲し、どの result を trusted kernel が受理しなければならないかを定める。

## Context

- [00.pipeline_overview.md](./00.pipeline_overview.md) — overall pipeline。この文書は phases 12-14 を詳細化する
- [doc/spec/en/16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md) — proof syntax and `by` justification
- [doc/spec/en/17.clusters_and_registrations.md](../../../spec/en/17.clusters_and_registrations.md) — clusters and registrations
- [doc/spec/en/20.algorithm_and_verification.md](../../../spec/en/20.algorithm_and_verification.md) — algorithm verification and verification conditions
- [doc/spec/en/21.source_code_annotation_and_atp.md](../../../spec/en/21.source_code_annotation_and_atp.md) — annotations and ATP integration
- [doc/spec/en/23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md) — build lifecycle、cluster-db、artifact output

### Pipeline Position

| Phase | Responsibility |
|---|---|
| 12. Pre-ATP Discharge | Mizar 側の deterministic machinery で解ける obligations を discharge する |
| 13. ATP Translation / Dispatch | open VCs を ATP problems へ translate し、external provers へ dispatch する |
| 14. Kernel Certificate Check | ATP evidence を独立に validate し、proof status を finalize する |

## Design Decisions

### Mizar-Side Reasoning

Mizar-side verifier は deterministic semantic processing を所有する。

- type checking、subtype checks、coercion checks
- Fraenkel expressions の sethood checks
- cluster and registration resolution
- overload resolution
- surface syntax から core logical IR への elaboration
- definitional expansion boundaries
- trivial VC discharge、computation-based discharge、type-derived facts

これらの責務は ATP translation の前に完了していなければならない。

### ATP-Side Reasoning

External provers は search-heavy logical reasoning に使う。

- `by` justification steps の first-order reasoning
- cited premises と local hypotheses に基づく equational reasoning
- commutativity、symmetry、reflexivity などの property-based reasoning
- backend が support する場合の refutation-style proof search

ATP backends は type inference、overload resolution、cluster resolution、namespace resolution を実行しない。

### Kernel Responsibility

kernel は independently checkable evidence だけを受理する。

- ATP results は evidence であり、trusted proof status ではない。
- Proof certificates または replayable witnesses は受理前に check されなければならない。
- `externally_attested` proofs は policy-controlled exceptions として記録してもよいが、kernel-verified proofs と同等ではない。

## Alternatives Considered

1. **All reasoning inside Mizar**: trust story は単純になるが、automation が弱く verifier が大きくなる。
2. **All reasoning delegated to ATPs**: search は強力だが、language-specific semantics と trust の control が弱い。
3. **Hybrid boundary**: Mizar が semantic processing を所有し、ATP が logical search を所有し、kernel が acceptance を所有する。

## Adopted Approach

Mizar Evo は hybrid boundary を採用する。これにより trusted base を小さく保ち、language-specific checks を deterministic にしつつ、ATP/SMT backend が強い領域を活用する。

## Interface Definitions

```text
Typed and elaborated Mizar context
  -> local VC + cited premises
  -> ATP translation
  -> backend certificate or witness
  -> kernel certificate check
  -> verified proof status
```

## Affected Modules

- `doc/design/mizar-checker/types.md` — type checking and type-derived facts
- `doc/design/mizar-checker/registrations.md` — cluster and registration resolution
- `doc/design/mizar-vc/generator.md` — verification condition generation
- `doc/design/mizar-atp/translator.md` — ATP problem translation
- `doc/design/mizar-atp/backend.md` — ATP backend invocation
- `doc/design/mizar-kernel/certificate.md` — proof certificate validation
- [atp_interface_protocol.md](./atp_interface_protocol.md)
- [atp_backend_integration.md](./atp_backend_integration.md)

## Constraints and Assumptions

- kernel は ATP output を盲目的に信頼してはならない。
- ATP が利用できなくても parsing、name resolution、type checking、cluster resolution は妨げられてはならない。
- backend が success を報告したとしても、certificate validation failure は proof error である。
