# Module: formula_evidence

> Canonical language: English. Japanese companion:
> [../ja/formula_evidence.md](../ja/formula_evidence.md).

## Purpose

The `formula_evidence` module owns the corrected kernel evidence schema that
supersedes legacy resolution-trace certificates for normal proof acceptance.
It refines architecture 08, architecture 15, architecture 16, architecture 19,
and internal 04.

The schema is trusted input only in the narrow sense that it is the material the
kernel is asked to check. It is not accepted until provenance, substitutions,
target binding, formula identity, deterministic SAT encoding, and SAT
refutation all pass.

## Trust Statement

This module is trusted kernel code. It may parse and structurally validate
formula/substitution evidence, but parsing never grants proof acceptance.

The module has no proof search, no SAT solving, no ATP search or backend
invocation, no premise selection, no overload resolution, no cluster search, no
implicit coercion insertion, no fallback inference, no acceptance from
backend-reported success alone, no source loading, no cache lookup, no artifact
lookup, no wall-clock or random-state reads, no unordered iteration dependence,
and no hidden reads of mutable compiler-global state. It must not accept
instantiated formulas, SAT clauses, backend proof methods, resolution traces,
SMT proof objects, or backend logs as trusted payload.

## Evidence Shape

The corrected evidence object is:

```text
KernelEvidence
  schema_version
  encoding_version
  kernel_profile
  target_vc
  symbol_manifest
  variable_manifest
  formula_evidence
  substitutions
  provenance
  final_goal
```

`formula_evidence` entries record formulas available to the target VC:

```text
FormulaEvidenceEntry
  formula_id
  source_class
  formula
  formula_fingerprint
  required_proof_status?
  imported_fact_ref?
  local_context_ref?
  vc_fact_ref?
  provenance_ref
```

`source_class` is one of local hypothesis, cited premise, generated VC fact,
accepted imported axiom, accepted imported theorem, or policy-bounded built-in
fact. The first implementation may model formulas as a kernel-owned
propositional formula tree over normalized `clause::Atom` values; richer
Mizar/core formulas must be added only with a paired spec update and tests.

Each entry carries a tree-only formula fingerprint plus a separate
kernel-derived entry hash input over the source class and provenance binding.
Neither hash input includes source paths, backend logs, timestamps, worker
order, display names after binding, or SAT clauses.

`ParsedKernelEvidence` preserves parser-validated bindings through private
fields and read-only accessors. Callers may inspect the target, profile,
manifest, formula, substitution, provenance, final-goal, and canonical hash
input records, but they must not be able to mutate or reconstruct a parsed
object after validation and before kernel checking.

## Canonical V1 Envelope

Task 25 implements a deterministic binary envelope owned by
`src/formula_evidence.rs`. The v1 envelope uses domain separator
`MIZAR_KERNEL_EVIDENCE\0`, schema version `1`, encoding version `1`, the
existing `certificate_parser::KernelProfileRecord`, the expected target VC
fingerprint, and fixed-order sections:

1. symbol manifest;
2. variable manifest;
3. formula evidence entries;
4. substitution evidence records;
5. provenance entries;
6. final goal.

Each section is a sequence of length-framed items. The parser rejects unknown
schema or encoding versions, noncanonical section order, trailing bytes,
duplicate ids, unsorted id lists, section count mismatches, and resource-limit
violations. The canonical hash input for a parsed evidence object is the exact
validated envelope bytes; producers must not provide a separate trusted hash
payload.

The symbol and variable manifests are structural inputs to formula validation
only. They define the `clause::ClauseValidationContext` used to check
`clause::Atom` and `clause::Term` values in formulas and substitution payloads.
They do not authorize hidden symbol lookup, overload resolution, or source
loading.

## Formula Grammar

The first implementation supports this propositional formula grammar:

```text
Formula =
  Atom(clause::Atom)
  Not(Formula)
  And(nonempty Formula list)
  Or(nonempty Formula list)
```

All child lists are length-framed, bounded by parser limits, and kept in caller
order. The parser validates every atom against the manifest-derived clause
context and rejects malformed terms, missing symbols, unsupported symbol kinds,
noncanonical variables, oversized terms, empty conjunctions/disjunctions, and
excessive formula depth or node counts. Formula rendering is deterministic and
is used only for diagnostics/tests, not as trusted input.

The normalized formula fingerprint algorithm for task 25 is
`SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID = 2`. Its digest is the canonical
formula hash input bytes derived by the kernel from the parsed formula tree.
The parser recomputes this fingerprint and rejects mismatches. This is a stable
identity binding, not a cryptographic acceptance claim.

Formula fingerprints are tree-only identities. Formula entries also have a
kernel-derived entry hash input over formula id, source class, source binding,
tree fingerprint, and provenance reference, but that entry hash is not a
caller-supplied trusted field. Provenance checks compare explicit target and
formula-tree fingerprints; they do not treat the entry hash as proof
acceptance material.

## Source And Provenance Binding

Each formula entry binds to exactly one source binding whose shape must match
its `source_class`:

- local hypothesis and cited premise entries use a nonzero local-context id;
- generated VC facts use a nonzero VC-fact id;
- accepted imported axioms and accepted imported theorems use package id,
  module path, exported item id, statement fingerprint, and required proof
  status;
- policy-bounded built-ins use a nonempty built-in id.

Every formula entry references one provenance entry. Provenance entries bind a
provenance id, the target VC fingerprint, the formula-tree fingerprint, and an
opaque nonempty producer-owned payload. The parser rejects missing provenance,
empty provenance payloads, provenance target-binding mismatches, and formula
fingerprint mismatches. Imported source bindings additionally require the imported
statement fingerprint to equal the formula-tree fingerprint until richer source
formula projection is specified.

Task 31 makes local hypothesis, cited premise, and generated VC fact entries
acceptable only when `FormulaEvidenceContext` carries the task-28 context
identity payload for the same target. That payload is immutable caller context:
the kernel checks its target VC, recomputes its `context_identity_hash()` from
the architecture-15 v1 line grammar, and requires every non-imported formula
entry to match exactly one row by source class/id, formula id, and formula
fingerprint. The row's producer formula ref remains part of the recomputed
context-identity hash, but it is not a separate parser-envelope field. The
payload's canonical handoff hash is the opaque `mizar-vc` formula-envelope
handoff hash, not
`ParsedKernelEvidence::canonical_hash_input()`, and the kernel must not derive
one from the parser's binary envelope bytes. Missing, stale, hash-mismatched,
missing-row, or ambiguous-row context identity rejects as `missing_provenance`
before SAT encoding.

## Substitutions

Substitution records are explicit evidence. They identify the formula they are
applied to, the normalized binder context, side conditions, and the substitution
payload needed by `substitution_checker`.

Instantiated formulas are not trusted fields. Task 26 derives instantiated
formulas by applying checked substitutions to the source formulas. Missing,
stale, duplicate, or inconsistent substitution provenance is a kernel
rejection, not an invitation to infer a repair.

Task 25 stores substitution records as explicit payload evidence only:

```text
FormulaSubstitutionEvidence
  substitution_id
  source_formula_id
  binder_context_encoding
  payload: substitution_checker::SubstitutionPayload
  freshness_witnesses
  free_variable_constraints
  provenance_ref
```

There is no target or instantiated formula field. The parser structurally
validates ids, referenced source formulas, provenance binding, term paths,
payload owner ids, replacement roles, witness owners, free-variable constraint
owners, deterministic ordering, and resource limits. Semantic replay and
formula instantiation remain task 26 work.

## Provenance And Target Binding

Every formula must bind to one available proof source. Imported facts must bind
to stable package/module/item identity, statement fingerprint, and required
proof status. Local context and generated VC facts must bind to caller-supplied
target, VC provenance, and the task-31 context identity rows. The kernel must
derive trusted `used_axioms` only from accepted formula evidence whose source
class is an accepted imported axiom or theorem.

`final_goal` records the target formula and refutation polarity. The kernel
checks the supplied formulas plus the negated goal according to the profile. It
must reject evidence whose target VC, goal fingerprint, kernel profile, or
context identity does not match the caller's immutable context.

The task-25 final goal record contains a standalone goal formula, goal
polarity, formula fingerprint, and provenance reference. It is not part of the
asserted premise formula set and is not a source for `used_axioms`. The parser
structurally validates the goal formula with the same manifest-derived context,
requires its fingerprint to match the goal formula tree, and requires its
provenance to bind the target VC and goal fingerprint. It records the target
binding. Acceptance is still granted only by the checker service after tasks
26-31 instantiate formula evidence, encode the SAT problem, run the trusted SAT
checker, bind proof-obligation polarity, and verify non-imported context
identity; the `formula_evidence` parser alone never grants trust.

## Legacy Evidence

Legacy `Certificate`, `generated_clauses`, `resolution_trace`, backend proof
method fields, and backend logs are compatibility or migration-audit material
only. Under normal proof policy they map to unsupported evidence and cannot
produce accepted `KernelCheckResult`, `used_axioms`, proof witnesses, cache
promotion, or artifact `kernel_verified` status.

## Gap Classification

- `design_drift` / `source_drift`: the task-22 source kept legacy
  resolution-trace certificates in `checker`; task 29 gates that path behind
  explicit migration/audit policy so normal proof policy rejects it before
  replay.
- resolved `test_gap`: task 31 adds valid local/cited/generated
  context-identity acceptance, missing/stale payload rejection, formula-id and
  row-mutation rejection, goal-as-hypothesis rejection, constructor/runtime
  context-identity limits, PolicyBoundedBuiltin exemption coverage, and a
  task-28 line-grammar golden vector.
- `external_dependency_gap`: full source-derived formula payloads from VC/ATP
  producers are not complete yet; the kernel schema must reject missing
  producer payloads instead of fabricating them.
- `deferred`: artifact witness projection, ATP candidate evidence production,
  source-to-kernel-evidence runner activation, and richer producer-owned
  payload schemas remain later tasks and must not be stubbed here.

## Rejection Mapping

Task 25 separates envelope parsing from evidence-binding validation. Envelope
and byte-shape errors map to `certificate_rejection`; evidence-binding errors
map to `kernel_rejection`. Stable details are:

- domain, schema, encoding, section order, and profile support failures use
  `unsupported_certificate_format`;
- envelope expected-target mismatches use certificate-level `context_mismatch`;
- provenance or final-goal target-binding mismatches use kernel
  `missing_provenance`;
- malformed formula/source/substitution/final-goal bytes use
  `malformed_witness_data`;
- missing or inconsistent source/provenance/goal bindings use kernel
  `missing_provenance`;
- parser and canonical-byte limits use `resource_exhaustion`.

Legacy `Certificate` / `resolution_trace` bytes do not share the task-25
domain separator and therefore reject as unsupported under this parser.

## Planned Tests

Task 25 must test structural round-trips, stable hash/rendering, unknown schema
versions, duplicate ids, malformed formulas, missing provenance, imported fact
identity/fingerprint mismatches, missing target/goal binding, and legacy
evidence rejected outside migration/audit mode.
