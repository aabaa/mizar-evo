# Module: smtlib_encoder

> Canonical language: English. Japanese companion:
> [../ja/smtlib_encoder.md](../ja/smtlib_encoder.md).

## Purpose

The `smtlib_encoder` module lowers a validated backend-neutral `AtpProblem` to
deterministic SMT-LIB 2 text for candidate-producing SMT backends. It is a
concrete emitter only. It does not search for proofs, run a backend process,
call `mizar-kernel`, interpret backend output, publish artifact witnesses, or
create trusted acceptance material.

The emitted SMT-LIB document is backend input. It is not kernel evidence, not a
SAT problem, not an instantiated-formula payload, not an unsat core, and not an
SMT proof certificate. A backend may use the document to produce an untrusted
candidate later, but the kernel remains the only component that may accept
formula/substitution evidence after deriving its own instantiated formulas and
deterministic SAT problem.

## Scope

Task 11 is specification-only. It authorizes a future task-12 source module to
emit SMT-LIB text from `AtpProblem`; it does not add Rust source, backend
process execution, portfolio policy, kernel checking, proof-witness
publication, cache promotion, legacy certificate handling, or SMT proof-object
handling.

The task-12 implementation may support only the uninterpreted SMT-LIB profile
described here. Arithmetic theories, arrays, datatypes, bit-vectors, native
property declarations, quantifier-instantiation pragmas, solver options,
`get-proof`, `get-unsat-core`, and backend-specific commands remain
`deferred` until a paired English/Japanese spec defines exact semantics and
tests.

## Inputs And Output

The conceptual task-12 API consumes:

```text
SmtLibEncodingInput
  problem: AtpProblem
  dialect: Uninterpreted
```

and produces:

```text
SmtLibEncodingOutput
  text: byte-identical SMT-LIB document
  symbol_map: deterministic ATP-symbol to SMT-LIB-symbol metadata
  assertion_labels: deterministic :named label to AtpProblem item metadata
```

The semantic text must contain only one SMT-LIB `set-logic` command,
declarations, assertions, and one `check-sat` command. Non-semantic
diagnostics, source payloads, backend configuration, wall-clock data, process
ids, random seeds, backend command lines, backend logs, unsat cores, proof
traces, and SMT proof objects must not be embedded in the SMT-LIB document. If
task 12 exposes diagnostics, they must be returned outside `text` or in a
separately requested deterministic debug rendering that is explicitly
non-semantic.

## Dialect Coverage

Task 12 supports this fail-closed subset:

| Problem profile field | Requirement for task-12 uninterpreted SMT-LIB |
|---|---|
| `ConcreteFormat` | `logic_profile.concrete_formats()` contains `ConcreteFormat::SmtLib`. |
| `LogicFragment` | `LogicFragment::SmtLibUninterpreted`. `Fof` and `TffLike` are unsupported. |
| `ExpectedBackendResult` | `ExpectedBackendResult::Unsat`. The SMT-LIB file asserts the premises and the negated goal, then emits `check-sat`; only an `unsat` backend result can match the profile contract in later result classification. |
| `EqualitySupport` | `Unsupported` rejects equality formulas; `Supported` emits SMT-LIB `=`. |
| `QuantifierPolicy` | `PropositionalOnly` rejects `Forall` and `Exists` and selects `QF_UF`; `FirstOrder` emits SMT-LIB quantifiers and selects `UF`. Logic selection is profile-driven, not based on backend output or formula simplification. |
| `SoftTypeStrategy` | `GuardPredicates` only. `BackendSorts` and `SortsAndGuards` are unsupported for task 12 because the current `AtpProblem` model has no function/predicate sort signatures, and sort-only encoding must not erase required soft-type facts. |
| `NativePropertySupport` | Does not authorize native declarations. `EncodedProperty::native_declaration` is unsupported until a concrete native SMT-LIB spec exists. |

Task-12 sort encoding is intentionally conservative: all terms live in one
fixed uninterpreted sort named `mizar_universe`, and Mizar soft-type facts are
preserved as explicit guard predicates and type-guard assertions. `Function`
declarations are emitted as functions from `mizar_universe` arguments to
`mizar_universe`; `Predicate` declarations are emitted as predicates from
`mizar_universe` arguments to `Bool`.

Any `AtpBinder` with a sort, any profile that requires backend sorts, or any
rendered use of an `AtpDeclarationKind::Sort` fails closed for this dialect.
The mere presence of an unused sort declaration does not fail by itself; task
12 does not render problem-owned sort declarations. A future sorted SMT-LIB
profile must add explicit declaration signatures, type preservation rules, and
tests before enabling `BackendSorts` or `SortsAndGuards`.

## Command Ordering And Labels

The encoder emits commands in this deterministic order:

1. `(set-logic QF_UF)` for `PropositionalOnly`, or `(set-logic UF)` for
   `FirstOrder`.
2. `(declare-sort mizar_universe 0)`.
3. Function declarations for every `AtpDeclarationKind::Function`, ordered by
   normalized `AtpProblem.declarations()`.
4. Predicate declarations for every `AtpDeclarationKind::Predicate`, ordered
   by normalized `AtpProblem.declarations()`.
5. `AtpProblem.axioms()` as named assertions `ax_<id>`.
6. `AtpProblem.type_context().guards()` as named assertions `tg_<id>`.
7. `EncodedProperty::axiom` rows as named assertions `prop_<id>`.
8. The negated `AtpProblem.conjecture()` as named assertion `neg_conj_<id>`.
9. `(check-sat)`.

The goal is never asserted positively and never copied into the premise
section. For the current `Unsat` contract, SMT-LIB validity checking is:
premises, generated type guards, encoded property axioms, and `not(goal)` are
asserted, then the backend is expected to report `unsat`.

Named assertions use this exact shape:

```text
(assert (! <formula> :named <label>))
```

The negated conjecture label has the `neg_conj_` prefix because the asserted
formula is not the conjecture itself. Formula labels use the source row's
stable dense id in base-10 decimal without leading zeroes, except that id `0`
renders as `0`.

## SMT-LIB Grammar

The renderer uses a fixed textual grammar:

- each command is one line followed by `\n`;
- exactly one `set-logic` command is emitted, and it is the first line;
- the document ends with exactly one newline after `(check-sat)`;
- no comments are emitted in semantic text;
- declaration argument sorts are separated by one space;
- application arguments are separated by one space;
- assertion labels use only the fixed prefixes described above and decimal
  dense ids;
- no solver options, `push` / `pop`, `reset`, `get-model`, `get-proof`,
  `get-unsat-core`, or backend-specific commands are emitted.

Function and predicate declarations render as:

```text
(declare-fun <function> (mizar_universe ...) mizar_universe)
(declare-fun <predicate> (mizar_universe ...) Bool)
```

Zero-arity functions and predicates use an empty argument list `()`.
`GeneratedBinder` declarations are not top-level constants; they are consumed
only when rendering active quantifier binders.

## Formula Rendering

Task 12 renders structured formula trees, not backend text stored in the
problem. The mapping is:

| `AtpFormulaTree` | SMT-LIB rendering |
|---|---|
| `True` | `true` |
| `False` | `false` |
| `Atom(P, args)` | `<pred>` or `(<pred> <term1> <term2> ...)` |
| `Equality { left, right }` | `(= <term> <term>)` when equality is supported |
| `Not(f)` | `(not <f>)` |
| `And(fs)` | `(and <f1> ... <fn>)`, rejecting an empty list |
| `Or(fs)` | `(or <f1> ... <fn>)`, rejecting an empty list |
| `Implies(a, b)` | `(=> <a> <b>)` |
| `Forall { binders, body }` | `(forall ((<var> mizar_universe) ...) <body>)` when quantifiers are first-order |
| `Exists { binders, body }` | `(exists ((<var> mizar_universe) ...) <body>)` when quantifiers are first-order |

Singleton `And` / `Or` render as the single child formula without an `and` or
`or` wrapper. Empty `And` / `Or` and empty quantifier binder lists fail closed.

The renderer must not simplify formulas, clausify, Skolemize, reorder
connective operands, drop duplicate operands, normalize quantifier structure,
inline definitions, invent substitutions, add quantifier patterns, add
solver-specific attributes, or approximate unsupported constructs. Empty
conjunctions, disjunctions, or quantifier binder lists are producer errors
because their intended logical identity is not encoded in `AtpProblem`.

Term rendering maps `AtpTerm::Variable` to an SMT-LIB variable name and
`AtpTerm::Function` to an SMT-LIB function/constant name. Formula atoms must
refer to predicate declarations and terms must refer to function or
generated-binder declarations with matching arity; a mismatch is a fail-closed
encoder error even if `AtpProblem::try_new` would normally prevent it.

The encoder must track active quantifier scope. `AtpTerm::Variable` may be
rendered only when the variable is bound by the current formula's active
`Forall` or `Exists` binder stack. Free variables are rejected. Within one
emitted formula, duplicate binder variables in a quantifier and nested binder
shadowing are unsupported and fail closed. A binder variable must resolve to a
`GeneratedBinder` declaration and to a matching
`AtpSymbolSource::GeneratedBinder` symbol-map row; declaration presence alone
is not enough.

## Name Mangling

SMT-LIB symbols are derived deterministically from `AtpProblem` declarations
and symbol-map source identities. The encoder must not reuse raw display
spelling as SMT-LIB syntax.

Task 12 uses these classes:

- fixed universe sort: literal `mizar_universe`;
- predicate/function/constant names: lower-case prefix `m_` followed by a
  deterministic lowercase hexadecimal encoding of a canonical symbol key;
- variables for generated binders: lower-case prefix `v_` followed by a
  deterministic lowercase hexadecimal encoding of a canonical binder key;
- assertion labels: lower-case section prefix (`ax_`, `tg_`, `prop_`,
  `neg_conj_`) followed by the stable dense id in base-10 decimal without
  leading zeroes except that id `0` renders as `0`.

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
SMT-LIB symbols, reserved-word collisions, illegal symbol characters, fixed
sort collisions, and assertion-label collisions before returning output. Any
collision fails closed; the encoder must not append traversal-order suffixes to
recover.

## Provenance And Metadata

Every emitted assertion label must be traceable to the original `AtpProblem`
row and provenance id:

- `ax_<id>` maps to `AtpFormulaId` and its `AtpProvenanceId`;
- `tg_<id>` maps to `AtpTypeGuardId` and its `AtpProvenanceId`;
- `prop_<id>` maps to `AtpPropertyId`, target symbol, and its
  `AtpProvenanceId`;
- `neg_conj_<id>` maps to the conjecture formula id and its
  `AtpProvenanceId`, and records that the emitted assertion is negated for the
  `Unsat` validity contract.

This metadata helps later untrusted candidate extraction explain which problem
items a backend mentioned. It does not turn backend-reported unsat cores,
used axioms, backend proof methods, traces, logs, SMT proof objects,
resolution traces, or legacy certificates into trusted acceptance material.

## Determinism

Equivalent validated `AtpProblem` inputs and the same dialect must produce
byte-identical SMT-LIB text and byte-identical side metadata. Determinism
covers:

- logic selection from the profile;
- command ordering;
- declaration ordering;
- assertion labels;
- symbol and variable mangling;
- connective and quantifier rendering;
- newline style, which is `\n`;
- absence of comments, timestamps, process ids, randomized suffixes, backend
  version strings, environment paths, diagnostic order, backend logs, proof
  traces, and SMT proof objects from semantic text.

The encoder must not mutate `AtpProblem` or recompute its semantic identity.
Formatting differences are observable API behavior and require golden tests.

## Public Enum Forward Compatibility

Task 22 applies the frontend task-25 policy to the `smtlib_encoder` module.
Public enums owned here are `#[non_exhaustive]` for downstream crates:
`SmtLibDialect`, `SmtLibAssertionItem`, and `SmtLibEncodingError`.

Public enum inventory: `SmtLibDialect`, `SmtLibAssertionItem`, `SmtLibEncodingError`.

Future dialects, assertion item classes, or error variants must be specified
before source uses them. Inside `mizar-atp`, matches that alter SMT-LIB text,
side metadata, unsupported-profile classification, or proof status must be
explicit and fail closed unless a paired spec documents an intentional fallback.

## Failure Semantics

Task-12 SMT-LIB encoding fails closed for malformed producer input and returns
unsupported/deferred outcomes for valid problems outside the supported
uninterpreted profile. No failure mode creates proof acceptance.

- `deferred`: arithmetic theories, arrays, datatypes, bit-vectors, sorted
  function/predicate signatures, `BackendSorts`, `SortsAndGuards`, native
  property declarations, quantifier patterns, solver options, `get-proof`,
  `get-unsat-core`, and backend-specific commands.
- `source_drift`: if source supports an SMT-LIB dialect, formula construct,
  sort encoding, native property shortcut, or command not specified here, add a
  paired spec update before implementation.
- `boundary_violation`: emitting backend proof methods, SMT proof objects,
  backend logs, instantiated formulas, SAT problems, kernel-derived material,
  legacy certificates, or resolution traces as trusted material is forbidden.
- `external_dependency_gap`: result classification, backend execution,
  candidate extraction, proof policy, witness publication, and cache promotion
  remain in later tasks/crates.

Unsupported SMT-LIB profiles may leave the VC open or select a different
explicit backend profile later. The encoder must not silently drop type guards,
replace unsupported formulas with `true`, assert the goal positively, or accept
`sat` / `unknown` as proof.

## Task-12 Test Expectations

Task 12 must add focused Rust coverage for:

- golden SMT-LIB output with `set-logic`, fixed `mizar_universe` sort,
  declarations, `ax_`, `tg_`, `prop_`, `neg_conj_`, and `(check-sat)` ordering;
- `QF_UF` selection for `PropositionalOnly` and `UF` selection for
  `FirstOrder`;
- exact rendering for every supported formula and term form, including
  singleton `And` / `Or` rendering and final newline behavior;
- premises plus negated conjecture polarity, with no positive goal assertion;
- byte-identical output under shuffled equivalent inputs and under diagnostic
  changes that do not alter semantic problem content;
- profile gates for missing `ConcreteFormat::SmtLib`, non-`SmtLibUninterpreted`
  fragments, unsupported soft-type strategies, equality-disabled formulas,
  quantifier-disabled formulas, sorted binders, and sort-dependent uses;
- unused `AtpDeclarationKind::Sort` rows are accepted, ignored for rendering,
  and absent from the SMT-LIB output in the supported guard-predicate profile;
- native-property declaration rejection;
- free-variable, duplicate-binder, shadowing, missing-declaration,
  missing-symbol-map, invalid-declaration, invalid-arity, and invalid-binder
  source failures;
- raw-name injection avoidance for function, predicate, constant, and
  generated-binder spellings;
- duplicate/illegal/reserved SMT-LIB symbol and assertion-label rejection;
- symbol-binding and assertion-label side metadata;
- absence of backend runner, kernel/SAT checking, proof acceptance, witness,
  cache, legacy certificate, resolution trace, SMT proof object, unsat-core
  trust, and trusted backend-material API surface.
