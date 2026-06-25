# Module: checker

> Canonical language: English. Japanese companion:
> [../ja/checker.md](../ja/checker.md).

## Purpose

The `checker` module owns phase-14 orchestration for a normalized kernel
certificate. It composes the parser, imported-fact validation, substitution
checking, resolution replay, explicit cluster/reduction trace replay, derived
fact validation, and final-goal acceptance into one policy-independent kernel
result.

The module refines
[architecture 15](../../architecture/en/15.kernel_certificate_format.md)
"Imported Facts" and "Kernel Rejection Semantics",
[architecture 17](../../architecture/en/17.cluster_trace_format.md), and
[internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
"Kernel Check Service".

## Trust Statement

This module is trusted kernel code. It may accept a proof only after all
required evidence has been replayed or checked from explicit immutable inputs.

The module must not perform proof search, ATP search, SAT solving, premise
selection, overload resolution, cluster search, registration activation,
implicit coercion insertion, fallback inference, source loading, cache lookup,
artifact lookup, wall-clock or random-state reads, unordered iteration, or
hidden reads of mutable compiler-global state. Backend-reported success,
backend-reported used axioms, resolver output, cache hits, artifact metadata,
or policy permission never replace kernel replay.

## Owned Behavior

The module owns:

- constructing one deterministic check pipeline over an already parsed
  `ParsedCertificate`;
- validating the certificate target, profile, and immutable kernel context
  binding;
- validating imported axioms and theorems by stable identity, statement
  fingerprint, and required proof status;
- supplying imported-clause evidence to `resolution_trace`;
- invoking `substitution_checker` and `resolution_trace` and verifying their
  private report bindings before using checked outputs;
- validating explicit cluster/reduction traces once their evidence schema is
  implemented;
- validating `derived_facts` and `final_goal`;
- extracting trusted `used_axioms` from checked imported fact references only;
- returning deterministic accepted/rejected `KernelCheckResult` values.

The module does not own:

- certificate byte parsing;
- clause normalization internals;
- MiniSAT resolution replay internals;
- substitution, alpha, freshness, or free-variable replay internals;
- cluster or reduction search;
- source-derived certificate production;
- proof-policy projection, witness publication, cache reuse, or artifact
  emission;
- choosing among multiple backend candidates.

## Input And Context

Task 13 specifies the check-service contract; tasks 14-16 implement it in
slices.

```text
KernelCheckInput
  target_vc_fingerprint
  parsed_certificate
  imported_fact_context
  substitution_context
  cluster_trace_context
  checker_limits

ImportedFactContext
  imported_axioms: sorted map imported_fact_id -> ImportedFactEvidence
  imported_theorems: sorted map imported_fact_id -> ImportedFactEvidence
  provenance_fingerprint

ImportedFactEvidence
  imported_fact_id
  package_id
  module_path
  exported_item_id
  statement_fingerprint
  accepted_proof_status
  normalized_clause_fingerprint
  clause

ClusterTraceContext
  cluster_steps: sorted map cluster_trace_step_id -> ClusterStepEvidence
  reduction_steps: sorted map reduction_step_id -> ReductionStepEvidence
  provenance_fingerprint
```

Concrete Rust types may use sorted vectors instead of maps, as long as
constructors canonicalize input order or reject duplicate ids
deterministically before replay. Context entries are validated at first use in
certificate order; unused context entries are ignored.

`ImportedFactEvidence` is caller-supplied immutable evidence. It is not
queried from a resolver, checker, ATP backend, cache, artifact, package index,
or global compiler table. Missing imported-fact context or provenance is
`missing_provenance`. A parsed imported fact whose evidence is absent,
identity-mismatched, fingerprint-mismatched, or accepted only under a weaker
status than `RequiredProofStatus` requires is `unresolved_symbol`.

`ImportedFactEvidence.clause` is the normalized clause made available to
resolution replay. It must validate against the parsed certificate's kernel
profile, symbol manifest, variable manifest, and checker limits before it is
passed into `resolution_trace`. The checker must recompute the canonical
fingerprint of the normalized clause under the parsed certificate's hash and
clause profile, require it to equal `normalized_clause_fingerprint`, and require
both the recomputed fingerprint and the evidence `statement_fingerprint` to
equal the parsed `ImportedFactRef.statement_fingerprint`. Clause shape or
profile mismatch in the provided immutable evidence is `missing_provenance`;
imported identity, clause-content fingerprint, or proof-status mismatch is
`unresolved_symbol`.

There is no caller-supplied `imported_clause_context` in `KernelCheckInput`.
The only imported-clause context passed to `resolution_trace` is the
checker-owned context constructed from imported facts that passed identity,
fingerprint, proof-status, and clause-validation checks. This prevents
unchecked clauses from bypassing imported-fact validation.

`cluster_trace_context` remains an explicit evidence input. Until task 15
implements cluster/reduction replay, any certificate evidence requiring it is
classified as `external_dependency_gap`/`deferred` for development planning and
must be rejected as missing or invalid evidence at runtime rather than
accepted by a placeholder.

Cluster and reduction evidence records must carry the replay fields from
architecture 17:

```text
ClusterStepEvidence
  cluster_trace_step_id
  source_type
  applied_cluster
  generated_attribute
  generated_type
  dependency

ReductionStepEvidence
  reduction_step_id
  applied_reduction
  rule_fqn
  enclosing_term_before
  redex_path
  source_redex
  target_term
  substitution
  discharged_guards
  rule_view
  selection_key

GuardEvidence
  guard_id
  source_fact_ref
  checked_dependency_ref
```

Strategy-audit fields such as `enclosing_term_before`, `redex_path`,
`rule_view`, and `selection_key` are checked as recorded evidence. The kernel
audits the selected redex and rule against explicit active-rule data and the
recorded enclosing term only; it does not search for an alternate redex or
rule.

## Result Shape

The success and failure surface is policy-independent:

```text
KernelCheckResult
  target_vc_fingerprint
  status
  checked_imports
  checked_substitutions
  checked_resolution_steps
  checked_cluster_steps
  checked_derived_facts
  final_goal
  used_axioms
  rejections

KernelCheckStatus
  accepted
  rejected

CheckedImportedFact
  namespace
  imported_fact_id
  statement_fingerprint
  accepted_proof_status
  policy_taint

CheckedDerivedFact
  derived_fact_id
  source_clause_ref
  payload_fingerprint
```

`accepted` means the normalized certificate's final goal was checked by this
crate. It does not encode artifact-facing proof status. `mizar-proof` or a
later policy layer may project accepted ATP certificates as `kernel_verified`
and accepted built-in certificates as `discharged_builtin`, but that projection
is outside this crate.

`used_axioms` is derived only from checked imported axiom/theorem references
actually used by the accepted certificate. Backend-reported used-axiom lists
are ignored unless the normalized certificate and imported-fact context make
the same facts checkable.

If any checked import has `accepted_proof_status =
externally_attested_policy_permitted`, the accepted kernel result carries a
policy taint on that import and on the aggregate result. A policy layer must
not project such a result as unqualified `kernel_verified`; it may only emit a
policy-controlled externally attested or mixed-status result. If the active
release policy forbids that taint, the immutable imported-fact context must not
present the external status as satisfying the requirement.

Batch checking is a deterministic wrapper around single-certificate checks:

```text
KernelCheckBatchInput
  checks: sorted Vec<KernelCheckInput>

KernelCheckBatchResult
  results: sorted Vec<KernelCheckResult>
```

Batch results are sorted by target VC fingerprint, evidence id when present,
and canonical input order. Worker completion order, cancellation arrival order,
or parallel scheduling must not affect result order.

## Check Pipeline

The checker runs these steps in deterministic order:

1. Confirm the parsed certificate target and kernel profile match the caller's
   expected target and checker configuration.
2. Build manifest-derived clause and term validation contexts from parsed
   certificate data and `checker_limits`.
3. Validate imported axiom and theorem references at first use against
   `ImportedFactContext`.
4. Construct the imported-clause context for `resolution_trace` from checked
   imported fact evidence only.
5. Replay substitutions through `substitution_checker` and keep only its
   checked report.
6. Replay the MiniSAT-compatible resolution trace through `resolution_trace`
   and keep only its checked report.
7. Replay explicit cluster/reduction traces when task 15 evidence is present.
8. Validate `derived_facts` by checking that each source clause reference is a
   checked generated, imported, resolution, substitution-derived, or
   cluster-derived fact as specified by its payload schema.
9. Validate `final_goal` by resolving the referenced generated clause,
   resolution step, or derived fact and checking that it is the empty
   obligation or canonical final fact required by the target VC.
10. Emit one accepted result, or a deterministic rejected result containing the
    earliest stable rejection records.

The checker must never repair failed sub-checker reports or try alternate
pipelines. If any sub-checker rejects evidence, the checker rejects the
certificate.

## Imported Fact Checking

Task 14 implements imported-fact validation before resolution replay.

For each parsed `ImportedFactRef`, the checker must compare:

- `imported_fact_id`;
- `package_id`;
- `module_path`;
- `exported_item_id`;
- `statement_fingerprint`;
- `required_proof_status`.

Proof-status strength is ordered:

```text
kernel_verified > discharged_builtin > externally_attested_policy_permitted
```

An evidence status satisfies the requirement only when it is at least as
strong as the parsed requirement and is allowed by the active kernel profile.
Externally attested facts are not kernel-verified; they are accepted only when
the parsed certificate explicitly permits that requirement and the immutable
context records the policy-permitted status. Release policies that forbid
external attestations are outside this module, but their decision must be
reflected before constructing the immutable imported-fact context.

Imported proof-status, identity, or fingerprint failure is `unresolved_symbol`
with `imported_fact_id`. Missing context or missing context provenance is
`missing_provenance`.

## Cluster And Reduction Trace Boundary

Task 15 implements explicit cluster/reduction trace replay. The checker spec
requires:

- no cluster search or registration activation;
- no hidden transitive expansion;
- every generated type fact, reduction result, and guard discharge to be backed
  by explicit trace evidence;
- every dependency fact referenced by a trace to have already been checked as
  an imported fact, generated fact, or earlier trace step;
- invalid cluster/reduction evidence to map to `invalid_cluster_trace`;
- missing trace context or missing trace provenance to map to
  `missing_provenance`.

If the upstream `mizar-checker` cluster trace payload is not ready when task 15
starts, that task must record the gap as `external_dependency_gap`/`deferred`
and keep runtime behavior fail-closed.

## Derived Facts And Final Goal

`ParsedCertificate.derived_facts` are certificate-owned assembly records. Task
16 validates their payload schema after imported facts and cluster/reduction
traces have concrete evidence contracts. Until then, unknown derived-fact
payloads are not accepted.

There is no caller-supplied derived-fact payload map in `ClusterTraceContext`.
The only payload authority is the parsed normalized certificate. Any checked
derived fact must bind to the parsed `derived_fact_id`, `source`, and payload
bytes, then validate that payload against already checked imported facts,
generated clauses, resolution steps, or cluster/reduction steps. External trace
evidence may justify dependencies, but it must not replace or supplement the
certificate-owned derived-fact payload.

`final_goal` acceptance is deterministic:

- `generated_clause` and `resolution_step` goals must resolve to checked
  clauses and must be the canonical empty clause unless a later spec adds a
  different final-fact schema;
- `derived_fact` goals must resolve to a checked derived fact whose payload
  schema explicitly states that it closes the target VC;
- a `generated_clause` final goal is accepted only when that generated clause
  is consumed by a successful checked replay path, such as a checked
  resolution final-goal helper or a checked derived-fact payload; mere presence
  in `ParsedCertificate.generated_clauses` is not proof acceptance;
- missing, unchecked, forward, or mismatched final-goal references are
  `invalid_sat_proof` or `invalid_cluster_trace` according to the failed
  evidence family;
- target mismatch is `context_mismatch`.

## Limits

`CheckerLimits` collects deterministic budgets and forwards the relevant
subsets to sub-checkers:

```text
CheckerLimits
  parser limits
  resolution replay limits
  substitution replay limits
  imported fact count
  imported clause validation limits
  cluster trace step count
  reduction trace step count
  derived fact count
  final report record count
```

Exceeding a checker-owned budget is `resource_exhaustion`. Budget checks must
run before allocating large temporary vectors, sorting unbounded context
entries, cloning imported clauses, or materializing reports.

## Rejection Mapping

| Failure | Detail | Location |
|---|---|---|
| Missing imported fact context, cluster trace context, substitution context, derived imported-clause context, or provenance | `missing_provenance` | field path plus imported fact, substitution, cluster, reduction, or final-goal id when known |
| Malformed service witness envelope before parsing or before normalized evidence can be selected | `malformed_witness_data` | service evidence field path |
| Imported fact identity, statement fingerprint, unavailable theorem/axiom, or proof-status strength mismatch | `unresolved_symbol` | `imported_fact_id` |
| Substitution replay failure | forwarded `invalid_substitution`, `missing_provenance`, or `resource_exhaustion` | forwarded substitution location |
| Resolution replay failure | forwarded `invalid_sat_proof`, `missing_provenance`, or `resource_exhaustion` | forwarded clause or resolution-step location |
| Cluster/reduction trace replay failure | `invalid_cluster_trace` | cluster or reduction step id |
| Derived fact payload mismatch or unchecked dependency | `invalid_sat_proof` or `invalid_cluster_trace` | `derived_fact_id` |
| Final goal mismatch or unchecked final reference | `invalid_sat_proof` | `final_goal` plus referenced id when known |
| Target VC or context binding mismatch | `context_mismatch` | target/context field path |
| Unsupported checker or certificate profile | `unsupported_certificate_format` | profile field path |
| Checker-owned deterministic resource budget exhausted | `resource_exhaustion` | checker budget field path |
| Cancellation or timeout budget exhausted after parsing | `timeout` | cancellation or timeout field path |

When multiple checks fail, deterministic ordering follows `rejection.md`.
Human diagnostic text may add context, but stable detail keys and locations
must not depend on display names, file paths, backend logs, cache keys, worker
completion order, allocation addresses, wall-clock time, or random state.

## Determinism And Cost

The checker processes parsed certificate vectors in their parser-validated
order. Context constructors canonicalize caller-supplied evidence before the
check starts. Reports sort only by stable ids and parser order.

Cost is linear in checked certificate records plus explicitly referenced
context evidence within configured limits. The checker must not scan unrelated
dependency artifacts or search for alternate facts, traces, substitutions, or
proofs.

Cancellation is cooperative and deterministic. The checker may stop only at
defined step-boundaries counted by `CheckerLimits`; a stopped check returns
`timeout`, never partial acceptance. Parser-owned malformed bytes remain
`malformed_certificate`; service-envelope evidence that cannot be normalized
into a certificate or explicit kernel evidence is `malformed_witness_data`.

## Gap Classification

- `spec_gap`: before task 13, no local `checker` module contract defined how
  sub-checker reports, imported facts, explicit cluster traces, derived facts,
  and final-goal acceptance compose. This spec closes that local contract for
  tasks 14-16.
- `test_gap`: task 14 needs imported-fact validation tests; task 15 needs
  explicit cluster/reduction trace replay tests or a recorded
  `external_dependency_gap`; task 16 needs end-to-end check-service and final
  goal tests.
- `external_dependency_gap`: source-derived certificates, ATP proof
  translation, cluster trace payload production by `mizar-checker`, and
  proof/cache/artifact consumers are not active inputs to this crate. Missing
  producer or consumer integration is not mocked here.
- `deferred`: proof-policy projection, witness storage, cache reuse, artifact
  emission, and backend-candidate selection remain outside `mizar-kernel`.

## Planned Tests

Task 14 must add Rust tests for:

- imported axiom and theorem evidence accepted only when identity,
  fingerprint, and proof status satisfy the parsed requirement;
- missing imported-fact context/provenance rejected as `missing_provenance`;
- unavailable or mismatched imported facts rejected as `unresolved_symbol`;
- imported clause evidence validated against the certificate profile, symbol
  manifest, variable manifest, and resource limits before resolution replay;
- mismatched `normalized_clause_fingerprint` and mismatched recomputed
  clause-content fingerprints rejected before imported clauses enter
  resolution replay;
- unused malformed imported context entries ignored.

Task 15 must add Rust tests for:

- explicit cluster and reduction traces accepted only from recorded evidence;
- hidden transitive expansion, invalid reduction substitution, missing guard
  evidence, dependency mismatch, and strategy-audit mismatch rejected as
  `invalid_cluster_trace`;
- missing cluster trace context/provenance rejected as `missing_provenance`;
- fail-closed `external_dependency_gap` behavior if upstream trace payloads are
  not ready.

Task 16 must add Rust tests for:

- full pipeline acceptance from checked imports, substitutions, resolution
  trace, optional cluster trace, derived facts, and final goal;
- final-goal mismatch and unchecked final references rejected deterministically;
- duplicate context ids, duplicate evidence ids, simultaneous imported/cluster
  context failures, and multiple rejection records sorted with stable locations;
- report/input binding preventing accidental reuse of sub-checker reports;
- deterministic result ordering under shuffled context construction and
  shuffled parallel batch completion;
- policy taint propagation for externally attested imported facts;
- external attempts to replace or supplement certificate-owned derived-fact
  payloads rejected before final-goal acceptance;
- malformed witness envelopes rejected as `malformed_witness_data`,
  deterministic timeout/cancellation budgets rejected as `timeout`, and
  checker-owned deterministic resource limits rejected as
  `resource_exhaustion`;
- the trusted-boundary lint/test set mirrors the trust statement: no proof
  search, ATP search, SAT solving, premise selection, overload resolution,
  cluster search, registration activation, implicit coercion insertion,
  fallback inference, source loading, hidden dependency-artifact reads,
  ATP/proof/cache/artifact coupling, unordered iteration, wall-clock/random
  read, or global mutable-state read.
