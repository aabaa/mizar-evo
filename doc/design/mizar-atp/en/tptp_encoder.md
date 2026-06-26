# Module: tptp_encoder

> Canonical language: English. Japanese companion:
> [../ja/tptp_encoder.md](../ja/tptp_encoder.md).

## Purpose

The `tptp_encoder` module lowers a validated backend-neutral `AtpProblem` to
deterministic TPTP text for candidate-producing ATP backends. It is a concrete
emitter only. It does not search for proofs, run a backend process, call
`mizar-kernel`, interpret backend output, publish artifact witnesses, or create
trusted acceptance material.

The emitted TPTP file is backend input. It is not kernel evidence, not a SAT
problem, not an instantiated-formula payload, and not a proof certificate. A
backend may use the file to produce an untrusted candidate later, but the
kernel remains the only component that may accept formula/substitution evidence
after deriving its own instantiated formulas and deterministic SAT problem.

## Scope

Task 9 is specification-only. It authorizes a future task-10 source module to
emit TPTP text from `AtpProblem`; it does not add Rust source, backend process
execution, portfolio policy, kernel checking, proof-witness publication, cache
promotion, or legacy certificate handling.

The task-10 implementation may support only the FOF dialect described here.
TPTP TFF/TXF/THF, CNF clausification, include files, arithmetic theories, typed
native property declarations, and backend-specific pragmas remain `deferred`
until a paired English/Japanese spec defines exact semantics and tests.

## Inputs And Output

The conceptual task-10 API consumes:

```text
TptpEncodingInput
  problem: AtpProblem
  dialect: Fof
```

and produces:

```text
TptpEncodingOutput
  text: byte-identical TPTP document
  symbol_map: deterministic ATP-symbol to TPTP-name metadata
  formula_labels: deterministic TPTP-label to AtpProblem item metadata
```

The semantic text must contain only TPTP entries. Non-semantic diagnostics,
source payloads, backend configuration, wall-clock data, process ids, random
seeds, backend command lines, backend logs, and proof traces must not be
embedded in the TPTP document. If task 10 exposes diagnostics, they must be
returned outside `text` or in a separately requested deterministic debug
rendering that is explicitly non-semantic.

## Dialect Coverage

Task 10 supports this fail-closed subset:

| Problem profile field | Requirement for task-10 FOF |
|---|---|
| `ConcreteFormat` | `logic_profile.concrete_formats()` contains `ConcreteFormat::Tptp`. |
| `LogicFragment` | `LogicFragment::Fof`. `TffLike` and `SmtLibUninterpreted` are unsupported. |
| `ExpectedBackendResult` | `ExpectedBackendResult::Unsat`. The TPTP file presents the goal as a conjecture; later backend-result classification must map successful refutation/proof output back to the unchanged `Unsat` contract. |
| `EqualitySupport` | `Unsupported` rejects equality formulas; `Supported` emits TPTP `=`. |
| `QuantifierPolicy` | `PropositionalOnly` rejects `Forall` and `Exists`; `FirstOrder` emits TPTP quantifiers. |
| `SoftTypeStrategy` | `GuardPredicates` only. `BackendSorts` and `SortsAndGuards` are unsupported for FOF because task 10 has no typed TPTP semantics and must not erase soft-type facts. |
| `NativePropertySupport` | Does not authorize native declarations. `EncodedProperty::native_declaration` is unsupported until a concrete native TPTP spec exists. |

FOF emission is unsorted first-order emission. Any `AtpBinder` with a sort,
any formula/profile combination that would require backend sorts, or any
rendered use of an `AtpDeclarationKind::Sort` fails closed for this dialect.
The mere presence of an unused sort declaration does not fail by itself; FOF
does not render declarations, and only sorted binders or sort-dependent
formula/type-guard uses are unsupported. Guard predicates and explicit
type-guard formulas may still be emitted as ordinary FOF axioms when their
declarations are predicates/functions supported by FOF.

## Entry Roles And Ordering

The encoder emits entries in this deterministic order:

1. `AtpProblem.axioms()` as `fof(ax_<id>, axiom, <formula>).`
2. `AtpProblem.type_context().guards()` as
   `fof(tg_<id>, axiom, <formula>).`
3. `EncodedProperty::axiom` rows as
   `fof(prop_<id>, axiom, <formula>).`
4. `AtpProblem.conjecture()` as
   `fof(conj_<id>, conjecture, <formula>).`

The goal is never copied into the axiom section. The text contains no
declaration entries for FOF; declarations are consumed for validation, arity,
kind, binder, and name-mangling metadata. `EncodedProperty::native_declaration`
rows are not rendered in task 10 and must return an unsupported/deferred
failure instead.

Labels use stable dense ids from `AtpProblem` rows and must be unique after
mangling. If a later task adds TFF support, typed declarations must receive
their own ordered role and label rules in a paired spec before source changes.

## Formula Rendering

Task 10 renders structured formula trees, not backend text stored in the
problem. The mapping is:

| `AtpFormulaTree` | TPTP FOF rendering |
|---|---|
| `True` | `$true` |
| `False` | `$false` |
| `Atom(P, args)` | `<pred>` or `<pred>(<term1>, <term2>, ...)` |
| `Equality { left, right }` | `(<term> = <term>)` when equality is supported |
| `Not(f)` | `~(<f>)` |
| `And(fs)` | `(<f1> & ... & <fn>)`, rejecting an empty list |
| `Or(fs)` | `(<f1> | ... | <fn>)`, rejecting an empty list |
| `Implies(a, b)` | `(<a> => <b>)` |
| `Forall { binders, body }` | `(! [<vars>] : (<body>))` when quantifiers are first-order |
| `Exists { binders, body }` | `(? [<vars>] : (<body>))` when quantifiers are first-order |

The renderer uses a fixed grammar:

- each TPTP entry is one line:
  `fof(<label>, <role>, <formula>).` followed by `\n`;
- the document ends with exactly one newline after the final entry;
- labels use the section prefix and base-10 dense id described below;
- function and predicate arguments are separated by comma plus one space;
- variable lists are separated by comma plus one space;
- all compound formulas are parenthesized exactly as described by the table:
  equality as `(<left> = <right>)`, negation as `~(<formula>)`,
  n-ary conjunction/disjunction as `(<f1> & <f2> & ... & <fn>)` or
  `(<f1> | <f2> | ... | <fn>)`, implication as `(<left> => <right>)`,
  and quantifiers as `(! [<vars>] : (<body>))` /
  `(? [<vars>] : (<body>))`;
- singleton `And` / `Or` render as `(<f1>)`; empty `And` / `Or` and empty
  quantifier binder lists fail closed.

The renderer must not simplify formulas, clausify, Skolemize, reorder
connective operands, drop duplicate operands, normalize quantifier structure,
inline definitions, invent substitutions, or approximate unsupported
constructs. Empty conjunctions, disjunctions, or quantifier binder lists are
producer errors because their intended logical identity is not encoded in
`AtpProblem`.

Term rendering maps `AtpTerm::Variable` to a TPTP variable name and
`AtpTerm::Function` to a TPTP function/constant name. A zero-arity function is
rendered as a constant. Formula atoms must refer to predicate declarations and
terms must refer to function or generated-binder declarations with matching
arity; a mismatch is a fail-closed encoder error even if `AtpProblem::try_new`
would normally prevent it.

The encoder must track active quantifier scope. `AtpTerm::Variable` may be
rendered only when the variable is bound by the current formula's active
`Forall` or `Exists` binder stack. Free variables are rejected and must not be
left for TPTP implicit universal quantification. Within one emitted formula,
duplicate binder variables in a quantifier and nested binder shadowing are
unsupported and fail closed. A binder variable must resolve to a
`GeneratedBinder` declaration and to a matching `AtpSymbolSource::GeneratedBinder`
symbol-map row; declaration presence alone is not enough.

## Name Mangling

TPTP names are derived deterministically from `AtpProblem` declarations and
symbol-map source identities. The encoder must not reuse raw display spelling
as TPTP syntax.

Task 10 uses these classes:

- predicate/function/constant names: lower-case prefix `m_` followed by a
  deterministic lowercase hexadecimal encoding of a canonical symbol key;
- variables for generated binders: upper-case prefix `V_` followed by a
  deterministic lowercase hexadecimal encoding of a canonical binder key;
- formula labels: lower-case section prefix (`ax_`, `tg_`, `prop_`, `conj_`)
  followed by the stable dense id in base-10 decimal without leading zeroes
  except that id `0` renders as `0`.

The canonical symbol key is a length-delimited sequence of UTF-8 fields based
on declaration kind, declaration arity, the matching
`AtpSymbolMapEntry.source`, and the backend-neutral `AtpDeclaration.symbol()`
as a tie-breaker. It must not be based on map iteration, source range alone,
display spelling alone, backend output order, process ids, random state, or
wall-clock time. A declaration without a matching symbol-map row is rejected.

The canonical binder key is based on the generated-binder symbol-map source,
declaration id, declaration arity `0`, and binder position within its
quantifier. Variables must have `AtpDeclarationKind::GeneratedBinder` and a
matching `AtpSymbolSource::GeneratedBinder` row.

Even with an injective encoding, the implementation must check for duplicate
TPTP names, reserved-word collisions, illegal initial characters, and formula
label collisions before returning output. Any collision fails closed; the
encoder must not append traversal-order suffixes to recover.

## Provenance And Metadata

Every emitted formula label must be traceable to the original `AtpProblem`
row and provenance id:

- `ax_<id>` maps to `AtpFormulaId` and its `AtpProvenanceId`;
- `tg_<id>` maps to `AtpTypeGuardId` and its `AtpProvenanceId`;
- `prop_<id>` maps to `AtpPropertyId`, target symbol, and its
  `AtpProvenanceId`;
- `conj_<id>` maps to the conjecture formula id and its `AtpProvenanceId`.

This metadata helps later untrusted candidate extraction explain which problem
items a backend mentioned. It does not turn backend-reported used axioms,
backend proof methods, traces, logs, SMT proof objects, resolution traces, or
legacy certificates into trusted acceptance material.

## Determinism

Equivalent validated `AtpProblem` inputs and the same dialect must produce
byte-identical TPTP text and byte-identical side metadata. Determinism covers:

- entry ordering;
- formula labels;
- symbol and variable mangling;
- connective and quantifier rendering;
- newline style, which is `\n`;
- absence of timestamps, process ids, randomized suffixes, backend version
  strings, environment paths, and diagnostic order from semantic text.

The encoder must not mutate the `AtpProblem` or recompute its semantic
identity. Formatting differences are observable API behavior and need golden
tests.

## Failure Semantics

Task-10 FOF encoding fails closed for malformed producer input and returns an
unsupported/deferred outcome for valid problems outside the supported FOF
dialect. No failure mode creates proof acceptance.

- `deferred`: TFF-like sorted output, THF/TXF, CNF, include files, arithmetic
  theories, native property declarations, and backend-specific pragmas.
- `source_drift`: source support for a TPTP dialect or formula construct not
  specified here requires a paired spec update before implementation.
- `external_dependency_gap`: backend result parsing, evidence-candidate
  extraction, portfolio execution, proof-witness publication, and cache
  promotion belong to later tasks or other crates.
- `boundary_violation`: treating TPTP text, backend success, backend proof
  methods, backend logs, resolution traces, SMT proof objects, or legacy
  certificates as trusted acceptance material is forbidden.

## Task-10 Test Expectations

Task 10 must add focused Rust coverage for:

- golden FOF output for axioms, type guards, encoded property axioms, and a
  conjecture;
- byte-identical rendering across repeated runs and shuffled-but-equivalent
  validated inputs;
- formula rendering for atoms, equality, negation, conjunction, disjunction,
  implication, universal and existential quantifiers, variables, constants, and
  function applications, including exact separators, parenthesization, labels,
  and final-newline behavior;
- profile rejection for missing `ConcreteFormat::Tptp`, non-FOF fragments,
  backend-sort strategies, unsupported equality, unsupported quantifiers, and
  sorted binders;
- fail-closed rejection of native property declarations, empty `And`/`Or`, and
  empty quantifier binder lists;
- fail-closed rejection of free variables, duplicate binders, nested binder
  shadowing, declaration/symbol-map/kind/arity mismatches, and name-mangling
  collisions;
- raw-name injection coverage using source/backend-neutral spellings that look
  like TPTP keywords, reserved `$` names, uppercase/lowercase edge cases,
  punctuation, whitespace, and newline-like payloads, with assertions that
  all symbol and variable positions use only the deterministic `m_<hex>` and
  `V_<hex>` mangled names and that the raw spellings are absent as emitted
  symbol or variable names in semantic `text`;
- provenance side metadata for every emitted entry;
- lint/API guards confirming no backend runner, kernel/SAT checking, proof
  policy, witness/cache integration, backend proof methods, backend logs,
  resolution traces, instantiated formulas, or SAT problems are added.
