# Architecture: ATP Interface Protocol

> Canonical language: English. Japanese companion: [../ja/atp_interface_protocol.md](../ja/atp_interface_protocol.md).

## Purpose

This document defines how Mizar Evo translates prover-independent verification conditions into backend-neutral ATP problems and then into concrete problem formats such as TPTP and SMT-LIB.

It refines pipeline phase 13, specifically the problem translation part. Backend process execution is covered by [atp_backend_integration.md](./atp_backend_integration.md).

## Context

- [00.pipeline_overview.md](./00.pipeline_overview.md) — overall pipeline
- [reasoning_boundary.md](./reasoning_boundary.md) — reasoning responsibility split
- [doc/spec/en/21.source_code_annotation_and_atp.md](../../../spec/en/21.source_code_annotation_and_atp.md) — ATP translation, property encoding, and proof certificates
- [doc/spec/en/23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md) — verifier settings, ATP logs, and used-axiom recording

## Design Decisions

### Supported Formats

| Format | Target Backends | Primary Use |
|---|---|---|
| TPTP FOF/TFF | Vampire, E | First-order reasoning, equational reasoning, larger premise sets |
| SMT-LIB 2 | CVC5, Z3 | Arithmetic, SMT theories, equality mixed with theory reasoning |

### Backend-Neutral Problem Layer

`AtpProblem` is not concrete TPTP or SMT-LIB text. It is a backend-neutral representation produced from `VcIr`.

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

One `VcIr` may produce multiple `AtpProblem`s when different backend profiles need different encodings.

### Encoding Strategy

- Soft types are encoded as sorts when the backend profile supports them, otherwise as guard predicates.
- Mizar functors, predicates, modes, and attributes are mapped to backend-safe symbol names.
- Premises come from local hypotheses, explicitly cited `by` references, and Mizar-side resolved facts.
- Properties such as commutativity, symmetry, reflexivity, and idempotence are encoded either as backend-native declarations or as axioms.
- Used-axiom metadata must be preserved when the backend reports it.

### Property Encoding

| Property | TPTP Strategy | SMT-LIB Strategy |
|---|---|---|
| `commutativity` | native AC support when available, otherwise axiom | quantified axiom |
| `symmetry` | implication axiom | quantified axiom |
| `reflexivity` | universal axiom | quantified axiom |
| `idempotence` | equality axiom | equality axiom |
| `involutiveness` | equality axiom | equality axiom |
| `projectivity` | equality axiom | equality axiom |
| `asymmetry` | implication plus negation axiom | implication axiom |
| `connectedness` | implication axiom | implication axiom |
| `irreflexivity` | negated reflexive axiom | negated reflexive axiom |

## Alternatives Considered

1. **TPTP only**: simpler implementation, but loses SMT arithmetic and theory support.
2. **SMT-LIB only**: strong theory support, but weaker fit for general FOL and large equational searches.
3. **Dual format support**: more implementation work, but allows backend selection by obligation shape.

## Adopted Approach

Mizar Evo uses a backend-neutral `AtpProblem` layer and supports both TPTP and SMT-LIB concrete encoders.

## Interface Definitions

Concrete encoders consume `AtpProblem` and produce backend input:

- TPTP encoder: emits `fof(...)` / `tff(...)` declarations.
- SMT-LIB encoder: emits `(declare-...)`, `(assert ...)`, and `(check-sat)` forms.

## Affected Modules

- `doc/design/mizar-atp/problem.md` — backend-neutral problem model
- `doc/design/mizar-atp/tptp_encoder.md` — TPTP encoding
- `doc/design/mizar-atp/smtlib_encoder.md` — SMT-LIB encoding
- `doc/design/mizar-atp/property_encoding.md` — property encoding rules
- [reasoning_boundary.md](./reasoning_boundary.md)
- [atp_backend_integration.md](./atp_backend_integration.md)

## Constraints and Assumptions

- Encoding does not need to be reversible.
- Certificate validation is performed by the kernel, not by trusting encoder output.
- Backend-specific extensions may be used only when the selected backend profile records them.
