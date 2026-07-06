# Module: sat_encoding

> Canonical language: English. Japanese companion:
> [../ja/sat_encoding.md](../ja/sat_encoding.md).

## Purpose

The `sat_encoding` module derives deterministic SAT problems from checked
formula/substitution evidence. It refines architecture 15 and architecture 16.

## Trust Statement

This module is trusted kernel code. It derives check artifacts from accepted
schema data; it does not consume SAT clauses supplied by a caller.

The module has no proof search, no SAT solving, no ATP search or backend
invocation, no premise selection, no overload resolution, no cluster search, no
implicit coercion insertion, no fallback inference, no acceptance from
backend-reported success alone, no source loading, no cache lookup, no artifact
lookup, no wall-clock or random-state reads, no unordered iteration dependence,
and no hidden reads of mutable compiler-global state. It must not accept
caller-supplied instantiated formulas, SAT clauses, backend proof methods,
resolution traces, SMT proof objects, or backend logs as trusted payload.

## Owned Behavior

The module owns:

- validating task-26 formula-instantiation substitution side conditions from
  checked `formula_evidence` records;
- deriving instantiated formulas from source formulas and explicit checked
  substitutions;
- assigning SAT variables deterministically by canonical atom bytes;
- producing a deterministic CNF/Tseitin problem from formulas plus the
  target goal asserted with the polarity recorded in `final_goal`;
- exposing read-only encoded-problem accessors and canonical bytes for
  diagnostics and replay checks.

The module does not decide whether the recorded goal polarity is appropriate
for the caller's target obligation. Task 30 assigns that binding to
`checker`: `check_kernel_evidence` rejects a `final_goal.polarity` that does
not match `KernelEvidenceCheckKind` before this module encodes the SAT
problem.

The module does not own SAT solving, ATP encoding, premise selection, formula
selection, substitution invention, source formula projection, or backend proof
extraction.

## Task-26 Instantiation Scope

The first source-backed implementation consumes a
`formula_evidence::ParsedKernelEvidence` value. It treats formula entries as
premise formulas, applies explicit substitution records to their named source
formula, and adds each derived instantiated formula to the asserted formula
set. The original premise formulas remain asserted; the standalone final goal
remains separate and is never used as a premise or `used_axioms` source.

Task 26 supports formula-wide formal-to-actual substitutions whose
`SubstitutionPayload` has a root `rewrite_path`. Replacement terms are
validated against the manifest-derived clause context and the explicit
`BinderContextV1` bytes, including binder ids that appear inside payload
`actual_term` records. Zero-frame binder contexts are accepted only through the
explicit v1 encoding; empty bytes, noncanonical frames, missing frames, and
unused frames reject as `invalid_substitution`. Formal variables must occur in
the source formula, and capture under normalized binder terms rejects as
`invalid_substitution`. This is a deterministic replay check over explicit
payloads, not substitution search.

For task 26, root `rewrite_path` means the substitution is applied to every
unbound occurrence of each formal variable inside every atom term of the
source formula tree. The replay walks formulas in their canonical tree order:
`Atom` arguments left to right, `Not` child, then `And`/`Or` children in stored
order. Multiple replacements are applied as a simultaneous map keyed by formal
variable id; replacement actual terms are not recursively rewritten by other
entries in the same payload. Substitution records are consumed in ascending
`substitution_id` order because the parser already requires that order. Each
derived formula fingerprint and canonical bytes are recomputed by the kernel
after instantiation and before SAT encoding.

Richer substitution shapes remain fail-closed until the producer schema
records enough formula-path and alpha-renaming information for replay:
non-root term rewrite paths, local-abbreviation expansion payloads,
non-empty freshness witnesses, and non-empty free-variable constraint lists
reject as `invalid_substitution` rather than being repaired or guessed.
This is classified as `external_dependency_gap` / `deferred`, not a stub.

## Encoding Rules

The first schema version uses a propositional encoding over normalized atoms.
Atom identity is the canonical `clause::Atom` byte encoding under the evidence
profile. Variables are assigned in sorted canonical atom-byte order, with any
auxiliary Tseitin variables allocated in deterministic traversal order after
all atom variables.

The encoded problem contains all premise evidence formulas and all
substitution-derived instantiated formulas asserted true. For
`AssertFalseForRefutation`, the standalone target goal is asserted false. For
`AssertTrueForConsistency`, the standalone target goal is asserted true. The
final-goal formula is not also asserted as a premise merely because it appears
in the evidence envelope. Equivalent caller order must produce identical
canonical SAT bytes.

Canonical SAT bytes are not DIMACS and are not a trusted caller payload. They
are a kernel-derived diagnostic/check-trace encoding containing the schema
version, target VC, atom-variable manifest, derived formula instances, and CNF
clauses.

The task-26 canonical SAT bytes use domain `MIZAR_KERNEL_SAT_PROBLEM\0`,
schema version `1`, encoding version `1`, target fingerprint, a sorted
atom-variable manifest, sorted assertion records, and CNF clauses. SAT
variables are positive `u32` ids starting at `1`; atom variables are assigned
by sorted canonical atom bytes; auxiliary variables follow in the exact order
created by a pre-order Tseitin traversal of the sorted assertions. A SAT
literal is `(variable_id, positive_bool)`, where `positive_bool = true` means
the variable itself and `false` means its negation. Every clause stores
literals sorted by `(variable_id, positive_bool)` after duplicate removal.

Tseitin clauses are emitted in traversal order and then each root assertion
adds one unit clause. For `And(children)`, output `o` and child literals
`c_i` emit `(!o or c_i)` for each child, then `(o or !c_1 or ... or !c_n)`.
For `Or(children)`, output `o` emits `(o or !c_i)` for each child, then
`(!o or c_1 or ... or c_n)`. `Not(child)` reuses the child's literal with
flipped polarity and does not allocate an auxiliary variable. `Atom` reuses
its atom variable. Assertion records are sorted by assertion kind, asserted
polarity, source formula id, substitution id, formula fingerprint, and
canonical formula bytes, so equivalent caller order yields identical bytes.

Instantiated formulas and SAT clauses are kernel-derived artifacts. The
`EncodedSatProblem` fields are private outside the encoding module and exposed
only through read-only accessors, so downstream callers cannot mutate the
target binding, assertions, atom manifest, clauses, or canonical bytes before
SAT checking. These artifacts may be recorded as diagnostic check traces, but
they are never trusted input fields.

## Rejections

Malformed formula structure, unsupported formula operators, unsupported
substitution replay shape, capture risk, missing provenance, target mismatch,
unsupported atom encoding, and canonical byte budget failures reject before
SAT checking. Resource limits reject as `resource_exhaustion`; substitution
side-condition failures reject as `invalid_substitution`; semantic encoding
failures in kernel-derived SAT material reject as `invalid_sat_refutation`;
provenance and target binding failures remain `missing_provenance`.

## Gap Classification

- `test_gap`: task 26 must cover stable encoding, substitution mutation
  rejection, equivalent-order determinism, target polarity, and resource
  limits.
- `external_dependency_gap`: richer quantified or theory-aware encodings wait
  for producer-owned formula payloads and paired specs; this module must not
  invent them.
- `external_dependency_gap` / `deferred`: formula-path substitutions,
  local-abbreviation expansion replay, and alpha-renaming witnesses require a
  producer-owned formula-substitution schema extension before they can be
  accepted.
