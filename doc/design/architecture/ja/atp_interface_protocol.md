# Architecture: ATP Interface Protocol

> Canonical language: English. English canonical version: [../en/atp_interface_protocol.md](../en/atp_interface_protocol.md).

## 目的

この文書は、Mizar Evo が prover-independent verification conditions を backend-neutral ATP problems へ変換し、さらに TPTP や SMT-LIB のような concrete problem format へ変換する方法を定義する。

これは pipeline phase 13 のうち、problem translation 部分を詳細化する。Backend process execution は [atp_backend_integration.md](./atp_backend_integration.md) が扱う。

## Context

- [00.pipeline_overview.md](./00.pipeline_overview.md) — overall pipeline
- [reasoning_boundary.md](./reasoning_boundary.md) — reasoning responsibility split
- [doc/spec/21.source_code_annotation_and_atp.md](../../../spec/21.source_code_annotation_and_atp.md) — ATP translation、property encoding、proof certificates
- [doc/spec/23.package_management_and_build_system.md](../../../spec/23.package_management_and_build_system.md) — verifier settings、ATP logs、used-axiom recording

## Design Decisions

### Supported Formats

| Format | Target Backends | Primary Use |
|---|---|---|
| TPTP FOF/TFF | Vampire, E | First-order reasoning、equational reasoning、大きな premise sets |
| SMT-LIB 2 | CVC5, Z3 | Arithmetic、SMT theories、equality と theory reasoning の混在 |

### Backend-Neutral Problem Layer

`AtpProblem` は concrete TPTP text や SMT-LIB text ではない。`VcIr` から生成される backend-neutral representation である。

```rust
struct AtpProblem {
    problem_id: AtpProblemId,
    vc_id: VcId,
    logic_profile: LogicProfile,
    declarations: Vec<AtpDeclaration>,
    axioms: Vec<AtpFormula>,
    conjecture: AtpFormula,
    type_context: AtpTypeContext,
    properties: Vec<EncodedProperty>,
}
```

異なる backend profile が異なる encoding を必要とする場合、1 つの `VcIr` から複数の `AtpProblem` が生成されることがある。

### Encoding Strategy

- Soft types は backend profile が support する場合 sort として encode し、それ以外では guard predicates として encode する。
- Mizar functors、predicates、modes、attributes は backend-safe symbol names へ map する。
- Premises は local hypotheses、明示的に cited された `by` references、Mizar 側で resolved された facts から得る。
- commutativity、symmetry、reflexivity、idempotence などの properties は、backend-native declarations または axioms として encode する。
- backend が used-axiom metadata を報告する場合、それを保持しなければならない。

### Property Encoding

| Property | TPTP Strategy | SMT-LIB Strategy |
|---|---|---|
| `commutativity` | native AC support が利用可能なら使い、それ以外では axiom | quantified axiom |
| `symmetry` | implication axiom | quantified axiom |
| `reflexivity` | universal axiom | quantified axiom |
| `idempotence` | equality axiom | equality axiom |
| `involutiveness` | equality axiom | equality axiom |
| `projectivity` | equality axiom | equality axiom |
| `asymmetry` | implication plus negation axiom | implication axiom |
| `connectedness` | implication axiom | implication axiom |
| `irreflexivity` | negated reflexive axiom | negated reflexive axiom |

## Alternatives Considered

1. **TPTP only**: 実装は単純だが、SMT arithmetic と theory support を失う。
2. **SMT-LIB only**: theory support は強いが、general FOL や大きな equational search には適合しにくい。
3. **Dual format support**: 実装量は増えるが、obligation の形に応じて backend を選べる。

## Adopted Approach

Mizar Evo は backend-neutral `AtpProblem` layer を使い、TPTP と SMT-LIB の concrete encoders の両方を support する。

## Interface Definitions

Concrete encoders は `AtpProblem` を消費して backend input を生成する。

- TPTP encoder: `fof(...)` / `tff(...)` declarations を emit する。
- SMT-LIB encoder: `(declare-...)`, `(assert ...)`, `(check-sat)` forms を emit する。

## Affected Modules

- `doc/design/mizar-atp/problem.md` — backend-neutral problem model
- `doc/design/mizar-atp/tptp_encoder.md` — TPTP encoding
- `doc/design/mizar-atp/smtlib_encoder.md` — SMT-LIB encoding
- `doc/design/mizar-atp/property_encoding.md` — property encoding rules
- [reasoning_boundary.md](./reasoning_boundary.md)
- [atp_backend_integration.md](./atp_backend_integration.md)

## Constraints and Assumptions

- Encoding は reversible である必要はない。
- Certificate validation は kernel が実行し、encoder output を信頼して行うものではない。
- Backend-specific extensions は、selected backend profile がそれを記録する場合にのみ使用できる。
