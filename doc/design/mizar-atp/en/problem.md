# Module: problem

> Canonical language: English. Japanese companion:
> [../ja/problem.md](../ja/problem.md).

## Purpose

The `problem` module owns the backend-neutral ATP problem data model for
pipeline phase 13. It describes the immutable `AtpProblem` shape produced from
validated VC inputs before any concrete TPTP, SMT-LIB, backend process, or
portfolio policy is involved.

`AtpProblem` is untrusted candidate-production input. It is not kernel evidence,
not a proof witness, not a SAT problem, and not accepted proof material. The
kernel remains the only component that can check formula/substitution evidence,
derive instantiated formulas, build deterministic SAT material, and accept a VC.

## Boundary Rules

The problem layer may describe formulas, declarations, type context, encoded
properties, provenance, and target binding needed by ATP backends. It must not:

- run ATP backends, SAT solvers, or `mizar-kernel`;
- select additional premises, invent substitutions, repair binders, resolve
  overloads, search clusters, insert implicit coercions, or perform fallback
  inference;
- contain TPTP text, SMT-LIB text, DIMACS, SAT clauses, caller-supplied
  instantiated formulas, backend proof methods, backend logs, backend
  `used_axioms`, SMT proof objects, MiniSAT-compatible resolution traces, or
  legacy certificates as trusted fields;
- classify a backend result as accepted proof status;
- publish proof witnesses, cache entries, or artifact proof status.

Backend proof traces and backend success reports are diagnostic or extraction
inputs for later untrusted candidate production only. They do not become
trusted acceptance material through `AtpProblem`.

## Conceptual Shape

Task 3 exposes concrete Rust names for the backend-neutral data model, and the
module represents this conceptual shape:

```text
AtpProblem
  problem_id
  vc_id
  target_binding
  logic_profile
  expected_result
  declarations
  axioms
  conjecture
  type_context
  properties
  symbol_map
  provenance
  diagnostics?
```

`diagnostics` are optional producer-side notes and never participate in proof
acceptance. They may be included in deterministic debug rendering only when the
rendering explicitly labels them non-semantic.

## Field Requirements

| Field | Requirement |
|---|---|
| `problem_id` | Deterministic identity for the problem content, selected backend-neutral profile, and snapshot-local `vc_id` collation component. It is not a proof-reuse identity across source edits and is not kernel evidence. |
| `vc_id` | Snapshot-local VC ordering and collation id. Task-3 semantic identity includes it to distinguish same-content problems within a snapshot, but it must not be used as a stable target binding by itself. |
| `target_binding` | Stable target fingerprint and producer binding derived from validated `VcIr` / VC handoff inputs. Task 3 rejects missing or structurally invalid target bindings; mismatch comparison against a VC handoff input belongs to the translator/handoff task that has both sides of the binding. |
| `logic_profile` | Backend-neutral capability profile: first-order fragment, equality support, quantifier policy, sort/type strategy, property-native capability, and concrete-format eligibility. It must not record backend proof methods. |
| `expected_result` | Validity-checking polarity. The current trusted success contract is `Unsat`: premises plus the negated goal are unsatisfiable under the chosen encoding. `Sat`, `Unknown`, timeout, crash, or backend error does not prove the VC. |
| `declarations` | Symbol, sort, function, predicate, and generated-binder declarations needed by the formula layer. Each backend-visible declaration must have provenance, a unique symbol, and the kind/arity required by every formula, property, or type-guard reference that can affect candidate evidence. Each such reference must resolve through both a declaration and a `symbol_map` row. `symbol_map` bindings support encoding and diagnostics, but they are not a provenance substitute. |
| `axioms` | Deterministically ordered premise formulas already materialized by prior VC phases. Axioms are candidate-search input, not trusted `used_axioms`. |
| `conjecture` | The target goal formula. It is not an axiom and never becomes a `used_axioms` source. |
| `type_context` | Soft-type and sort context needed for sound backend encoding. Sort encoding must not erase required mode, attribute, subtype, coercion, guard, or intersection-like facts. |
| `properties` | Encoded definitional properties, either as explicit axiom formulas or as backend-native declarations selected by `logic_profile`. Native property declarations still need encoded ids and provenance. |
| `symbol_map` | Deterministic map between backend-safe symbols and canonical Mizar/core/generated identities. It is for encoding and diagnostics, not proof acceptance. |
| `provenance` | Complete source binding for every backend-visible formula, native property declaration, type guard, and generated declaration that can affect candidate evidence. |

## Logic Profile

`logic_profile` is selected before concrete encoding and records only Mizar-side
translation capabilities. It may state:

- whether the problem uses FOF, TFF-like typed first-order structure, SMT-LIB
  with uninterpreted symbols, or a later explicitly specified fragment;
- whether equality, quantifiers, finite sort encodings, or native property
  declarations are allowed;
- whether soft types are represented as backend sorts, explicit guard
  predicates, or both;
- which concrete encoders may consume the problem.

It must not encode backend success policy, portfolio priority, proof method,
solver seed, timeout, command-line flags, or process environment. Those belong
to backend and portfolio specs.

## Formula And Declaration Model

The problem model is backend-neutral. Formulas must be represented as structured
terms, atoms, equality, connectives, and quantifiers within the selected
`logic_profile`, not as backend text. A concrete encoder may later lower the
same problem to TPTP or SMT-LIB without changing the source problem identity.

Declarations are ordered by deterministic keys derived from canonical source
identity and generated-binder identity, never by map iteration, pointer address,
source range alone, display spelling alone, or backend output order.

The Rust data model uses caller-supplied dense identifiers for declarations,
formulas, properties, provenance rows, and type guards. These dense ids are
already-canonical keys: producers and later translator tasks must derive them
deterministically from the canonical source identity, generated-binder identity,
or stable generated-fact identity for the item. The problem layer validates
uniqueness and sorts by these ids, but it does not bless traversal-order,
backend-order, map-iteration, or display-spelling ids as canonical.

Formulas that cannot be represented in the selected profile are classified as
unsupported for that profile. The translator may try another explicit profile
in a later task, but the problem layer must not silently approximate the
formula or drop required facts.

## Provenance

Every backend-visible item that can influence a candidate must have provenance.
Allowed source classes are:

- local hypotheses from validated `VcIr`;
- explicitly materialized cited premises;
- generated VC facts with stable payloads;
- imported axioms or theorems only when stable package/module/item identity,
  statement fingerprint, required proof-status requirement, and kernel context
  requirements are available;
- checker-owned facts only when an explicit formula payload and source binding
  already exist;
- type facts and encoded definitional properties.

Trace-only records, backend logs, backend proof traces, backend-reported used
axioms, and legacy certificate objects are not provenance for trusted
acceptance. A Mizar-side trace may explain why a fact exists only after that
fact has already been materialized as explicit formula/provenance input; the
trace itself is not the formula.

## Expected Result And Failure Semantics

All task-2 problem shapes use `ExpectedBackendResult::Unsat` as the successful
validity contract. The problem may be consumed by profiles that present the
goal as a conjecture, a negated conjecture, or an SMT assertion of the negated
goal, but the recorded contract remains that backend success must correspond to
unsatisfiability of premises plus the negated goal.

Construction fails closed when required target binding, formula payload,
provenance, symbol mapping, declaration, type context, or the `logic_profile`
field itself is missing or invalid. Target-binding mismatch against VC handoff
input is deferred to the translator/handoff task because the problem data shape
does not carry a second expected binding to compare. Formula features that are
valid Mizar-side inputs but not supported by the selected profile are classified
as unsupported/open-status outcomes for that profile. ATP unavailability,
unsupported profiles, timeout, unknown, crash, and backend error leave the VC
open or produce diagnostics; they do not create accepted proof status.

## Determinism

Equivalent validated inputs must produce byte-identical deterministic debug
rendering and stable problem identities. Deterministic ordering applies to:

- declarations;
- axioms;
- generated type guards;
- encoded properties;
- symbol-map rows;
- provenance rows;
- diagnostic rows when diagnostics are rendered.

Backend completion order, wall-clock time, random state, process ids, stdout,
stderr, and backend-reported proof order are excluded from semantic identity.

## Public Enum Forward Compatibility

Task 22 applies the frontend task-25 policy to the `problem` module. All public
enums owned here are `#[non_exhaustive]` for downstream crates:
`LogicFragment`, `EqualitySupport`, `QuantifierPolicy`, `SoftTypeStrategy`,
`NativePropertySupport`, `ConcreteFormat`, `ExpectedBackendResult`,
`AtpDeclarationKind`, `AtpFormulaTree`, `AtpTerm`, `PropertyEncoding`,
`AtpSymbolSource`, `AtpSourceRef`, and `AtpProblemError`.

Public enum inventory: `LogicFragment`, `EqualitySupport`, `QuantifierPolicy`, `SoftTypeStrategy`, `NativePropertySupport`, `ConcreteFormat`, `ExpectedBackendResult`, `AtpDeclarationKind`, `AtpFormulaTree`, `AtpTerm`, `PropertyEncoding`, `AtpSymbolSource`, `AtpSourceRef`, `AtpProblemError`.

Future variants must be specified before source uses them. Inside `mizar-atp`,
matches that affect problem identity, evidence provenance, backend-visible
syntax, or proof status must be explicit and fail closed unless a paired spec
documents an intentional fallback. `ExpectedBackendResult` remains externally
non-exhaustive even though `Unsat` is the only task-3 success contract; any
future success contract needs a paired spec, tests, and acceptance-boundary
review.

## Relation To Kernel Evidence

`AtpProblem` can help later ATP tasks produce formula/substitution evidence
candidates, but it is not itself the evidence accepted by the kernel. Candidate
evidence must be built from formula, substitution, provenance, and target
binding records compatible with `mizar-kernel` schema. Instantiated formulas and
SAT problems are derived only by the kernel during checking.

## Gap Classification

- resolved `source_drift`: task 3 implements the Rust data shapes,
  deterministic debug rendering, fail-closed validation, and construction tests
  for this spec.
- `deferred`: translator, property encoding, concrete encoders, backend runner,
  and portfolio behavior require their own module specs.
- `external_dependency_gap`: full imported-fact context and downstream
  proof/cache/artifact policy remain outside this crate until their owner
  crates exist and define stable contracts.

## Task-3 Test Coverage

Task 3 adds Rust coverage for:

- construction of minimal and populated `AtpProblem` values;
- deterministic debug rendering and stable ordering under shuffled inputs;
- provenance completeness for every declaration, axiom, property, type guard,
  and conjecture;
- fail-closed construction rejection for missing target binding, missing formula
  payloads, missing provenance, missing symbol-map rows, missing declarations,
  missing type-context bindings, and duplicate ids;
- unsupported classification for profile limitations without silently
  approximating or dropping required facts;
- `ExpectedBackendResult::Unsat` as the only task-3 polarity, including
  rejection or unrepresentability of non-`Unsat` success contracts and stable
  rendering/identity coverage for the polarity field;
- absence of backend text, SAT clauses, instantiated formulas, proof methods,
  backend logs, legacy certificates, and accepted-proof status fields from the
  public problem API.
