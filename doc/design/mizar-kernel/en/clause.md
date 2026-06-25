# Module: clause

> Canonical language: English. Japanese companion: [../ja/clause.md](../ja/clause.md).

## Purpose

The `clause` module owns the normalized SAT-clause representation consumed by
the trusted kernel. It refines the "Clause Representation" section of
[architecture 15](../../architecture/en/15.kernel_certificate_format.md).

Clause values are evidence data. A well-formed clause is not a proof and does
not grant acceptance. Later certificate and trace checkers may trust only the
positive result of replaying explicit evidence over these canonical values.

## Trust Statement

This module is trusted kernel code. It checks structure, canonical ordering,
deterministic rendering, and hash inputs for literals and clauses. It must stay
small, deterministic, and total over bounded inputs.

The module must not perform proof search, heuristic premise selection, overload
resolution, cluster search, ATP search, implicit coercion insertion, fallback
inference, acceptance from backend-reported success alone, or hidden reads of
mutable compiler-global state, wall-clock time, random state, filesystem caches,
unordered iteration, or allocation addresses.

## Owned Behavior

The module owns:

- literal polarity and atom identity;
- canonical literal ordering;
- duplicate literal removal during normalization;
- structural well-formedness checks for atoms, terms, and clauses;
- tautology classification according to an explicit kernel-profile option;
- deterministic text rendering used by tests and diagnostics;
- stable hash input bytes for clauses, excluding nondeterministic data.

The module does not own:

- certificate parsing;
- parent-reference validation;
- resolution trace replay;
- substitution or alpha-equivalence checking;
- imported-fact availability;
- proof-policy projection.

## Data Model

The implementation uses these logical data shapes. Concrete Rust names may
follow the source style chosen in task 3.

```text
Clause
  profile
  literals
  form

Literal
  polarity
  atom

Atom
  symbol
  arguments

Term
  variable
  application
  binder_normalized
```

`profile` records the schema version, tautology policy, and canonical encoding
version. `form` is one of:

- `ordinary`, carrying one or more canonical literals;
- `empty`, carrying zero literals and representing the contradiction endpoint of
  a refutation trace;
- `tautology`, carrying zero literals when the profile permits explicit
  tautology markers.

`empty` and `tautology` are distinct forms. An empty clause is unsatisfiable
evidence that can be derived by resolution replay. A tautology marker records a
clause that is always true and must never be used as a contradiction.

Symbols, variables, and generated binders are compared only by stable normalized
ids supplied by earlier phases or by the certificate. Display names and source
ranges are diagnostic-only and are excluded from semantic equality and hashes.

## Canonical Ordering

Canonical ordering is total and platform-independent:

1. polarity, with negative literals before positive literals;
2. atom symbol kind in this fixed order: predicate, functor-as-predicate,
   equality, built-in relation;
3. stable symbol id within the symbol kind;
4. atom arity;
5. normalized argument encoding bytes;
6. literal canonical bytes as a final tie breaker.

Term ordering is the byte order of the term's normalized encoding. Variable
terms encode canonical variable ids, not parser encounter order, allocation
addresses, display names, or hash-map iteration order.

Canonical bytes use a length-prefixed binary grammar:

```text
u8 schema_tag
u8 form_or_term_tag
u32 length
bytes payload
```

All integers are unsigned big-endian values. Nested payloads are encoded by
concatenating their length-prefixed canonical child bytes in canonical order.
This byte grammar is the ordering tie breaker and the hash input source.

The module must not derive any semantic fact from ordering. Ordering exists only
to make equality, rendering, and hashing deterministic.

## Structural Well-Formedness

A literal is well formed only when:

- its atom has a stable symbol id;
- its arity matches the number of encoded arguments;
- every argument has a normalized encoding;
- every variable id is in canonical form for the certificate context;
- no display-only field participates in equality or hashing.

A clause is well formed only when:

- it is tied to one explicit clause profile;
- its form/literal payload matches the profile rules:
  `ordinary` has at least one literal, `empty` has zero literals, and
  `tautology` has zero literals and is allowed only when the profile permits
  tautology markers;
- its ordinary literals are sorted by canonical order;
- duplicate ordinary literals have been removed;
- it respects configured literal-count, term-size, and term-recursion-depth
  bounds.

Task 3 uses a minimal clause-local validation context:

```text
ClauseValidationContext
  profile
  allowed_symbol_kinds
  known_symbol_ids
  canonical_variable_ids
  max_literals
  max_term_encoding_bytes
  max_term_recursion_depth
```

The context is explicit input to clause construction. It is not a global symbol
table and must not be populated by resolver, checker, ATP, cache, or artifact
state. Later certificate parsing may provide the context from normalized
certificate metadata, but task 3 must be able to test clause-local validation
without parsing certificates.

Malformed structures are rejected as certificate or kernel errors by the caller.
This module reports the precise structural reason but does not decide the
phase-level diagnostic policy.

## Normalization

Normalization is a deterministic pure function:

```text
raw literals + ClauseValidationContext -> normalized clause or rejection
```

Normalization:

- validates every literal and term;
- sorts ordinary literals by canonical order;
- removes duplicate ordinary literals;
- detects opposite-polarity duplicate atoms;
- applies the profile's tautology rule.

When the profile rejects tautologies, any clause containing both an atom and its
opposite-polarity counterpart is invalid. When the profile allows explicit
tautology markers, the normalized result must use the zero-literal `tautology`
form rather than keep the contradictory literal pair or preserve any witness
pair. Non-contradictory literals from the raw input are discarded in this form so
that every tautological clause has exactly one rendering and hash.

An input with no literals normalizes to the zero-literal `empty` form, not to a
tautology marker.

## Rendering And Hashing

Debug rendering must be deterministic and suitable for snapshot tests. It uses:

- schema/profile version;
- clause form;
- literal count;
- canonical literal renderings in order;
- stable ids and normalized argument encodings.

Stable clause hashes are computed from canonical bytes that include:

- a domain separator for kernel clauses;
- schema/profile version;
- tautology policy;
- clause form;
- canonical literal bytes.

Hashes must not include file paths, source ranges, display names, timestamps,
backend runtime logs, allocation addresses, map/set iteration order, or worker
completion order.

Trusted replay modules may need resource accounting before constructing large
temporary byte vectors. The clause module owns any non-allocating canonical
length or bounded-writer helpers used for this accounting. Callers must not
duplicate the clause encoder to estimate canonical sizes.

Trusted replay modules may also need to re-check already owned canonical clause
parts under a smaller replay budget. The clause module owns any borrowed
canonical-part validation helper used for that path so callers do not clone
large literal or term trees before literal-count, term-size, or term-depth
limits have been checked.

## Failure Classes

This module can produce structural rejection details for:

- missing or unstable symbol ids;
- arity mismatch;
- malformed term encodings;
- noncanonical variable ids;
- duplicate literals before normalization when duplicates are not accepted by a
  construction path;
- ordinary-form empty payloads and non-empty `empty` or `tautology` payloads;
- disallowed tautologies;
- literal-count, term-size, or term-recursion-depth resource exhaustion.

Callers map these details to the stable rejection categories specified by
`rejection.md` after that module exists. Until then, task 3 tests should assert
module-local structural error identity, not artifact-facing diagnostic text.

## Planned Tests

Task 3 must add Rust tests for:

- valid single-literal and multi-literal clauses;
- deterministic ordering by polarity, symbol id, arity, and argument encoding;
- symbol-kind precedence where different symbol kinds have otherwise comparable
  ids, so sorting by id alone cannot pass;
- duplicate literal removal;
- empty-clause / contradiction form rendering and hash stability;
- disallowed tautology rejection;
- allowed zero-payload tautology marker normalization;
- malformed arity, malformed term encoding, missing-symbol, and unsupported
  symbol-kind rejection;
- noncanonical variable-id rejection through an explicit clause-local validation
  context;
- explicit profile/form payload rules, including empty ordinary payload
  rejection and non-empty `empty` / `tautology` payload rejection;
- literal-count and term-size resource exhaustion;
- term-recursion-depth resource exhaustion through the depth limit in
  `ClauseValidationContext`;
- non-allocating canonical length or bounded-writer helpers returning the same
  lengths as canonical byte encoders without requiring callers to duplicate the
  encoder;
- borrowed canonical-part validation rejecting over-budget canonical clauses
  before callers clone their literal vectors;
- duplicate-before-normalization rejection for any construction path that
  bypasses normalizing constructors;
- rendering stability;
- hash-input stability under shuffled input literal order;
- hash-input coverage for the domain separator, schema/profile version,
  tautology policy, clause form, and canonical literal bytes;
- hash exclusion tests showing display names, source ranges, file paths,
  timestamps, backend logs, allocation order, and worker completion order do not
  affect canonical bytes.

No `.miz` fixture, expectation sidecar, or `doc/spec` change is required for
this module-spec task.

## Gaps And Deferred Items

| ID | Class | Evidence | Action |
|---|---|---|---|
| CLAUSE-G001 | `source_drift` / `test_gap` | `src/clause.rs` does not exist before task 3. | Task 3 implements this spec with focused Rust tests. |
| CLAUSE-G002 | `external_dependency_gap` | Normalized certificates and backend trace producers do not exist in `mizar-atp`. | Treat raw backend formats as producer-owned; this module accepts only normalized clause inputs. |
| CLAUSE-G003 | `deferred` | Artifact-facing clause snapshots and certificate corpus fixtures require later parser/checker/test harness tasks. | Keep task 3 coverage crate-local until those consumers exist. |
