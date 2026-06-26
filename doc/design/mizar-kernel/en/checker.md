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

## Post-Closeout Correction

The task-13 through task-22 checker is a legacy normalized-certificate
orchestrator centered on resolution replay. Architecture 15 now supersedes
that acceptance contract with formula/substitution evidence checked by a
trusted in-process Rust SAT checker. Until tasks 23-29 in
[todo.md](./todo.md) land, this module specification records both the legacy
implemented surface and the required correction.

The corrected checker accepts only caller-supplied evidence: formula refs or
formulas, explicit substitutions, provenance bindings, and target/goal
binding. It derives instantiated formulas and the SAT problem internally, then
runs the trusted SAT checker over that deterministic problem. It must not
search for formulas, invent substitutions, try alternate encodings, minimize
premises, call ATP/SAT child processes, or fall back to inference outside the
evidence.

## Trust Statement

This module is trusted kernel code. It may accept a proof only after all
required evidence has been replayed or checked from explicit immutable inputs.

The module must not perform proof search, ATP search, premise selection,
overload resolution, cluster search, registration activation, implicit
coercion insertion, fallback inference, source loading, cache lookup, artifact
lookup, wall-clock or random-state reads, unordered iteration, or hidden reads
of mutable compiler-global state. SAT activity is limited to deterministic
checking of the kernel-built problem through the trusted in-process SAT
checker selected by the dependency audit. Backend-reported success,
backend-reported used axioms, resolver output, cache hits, artifact metadata,
or policy permission never replace kernel checking.

Task 20 audited the legacy trust boundary as including no SAT solving. Tasks
23-29 replace that with the stricter distinction between prohibited proof
search and permitted trusted SAT checking over supplied evidence. The boundary
continues to include no ATP search or backend invocation, no premise
selection, no overload resolution, no cluster search, no implicit coercion
insertion, no fallback inference, no acceptance from backend-reported success
alone, no source loading, no cache lookup, no artifact lookup, no wall-clock
or random-state reads, no unordered iteration dependence, and no hidden reads
of mutable compiler-global state.

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
  requested_cluster_trace_steps
  checker_policy
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

CheckedFactContext
  imported_axioms: sorted imported_fact_id set
  imported_theorems: sorted imported_fact_id set
  generated_clauses: sorted generated_clause_id set

CheckedFactRef
  imported_axiom(imported_fact_id)
  imported_theorem(imported_fact_id)
  generated_clause(generated_clause_id)
  trace_step(cluster_or_reduction_step_id)

CheckerPolicy
  imported_fact_policy: ImportedFactPolicy

ImportedFactPolicy
  allow_externally_attested_imports
```

Concrete Rust types may use sorted vectors instead of maps, as long as
constructors reject over-budget context entry counts before sorting,
canonicalize input order within that bound, or reject duplicate ids
deterministically before replay. Context entries are validated at first use in
certificate order; unused context entries are ignored after the bounded
constructor succeeds.

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
fingerprint of the normalized clause under the parsed certificate's clause
profile. Task 14 supports only clause fingerprint algorithm id `1`, defined as
the exact `Clause::canonical_hash_input()` bytes with no cryptographic digest
step. Other normalized-clause fingerprint algorithm ids fail closed as
`unresolved_symbol` for the imported fact until a documented digest registry is
added. The recomputed fingerprint must equal `normalized_clause_fingerprint`,
and both it and the evidence `statement_fingerprint` must equal the parsed
`ImportedFactRef.statement_fingerprint`. Clause shape or profile mismatch in
the provided immutable evidence is `missing_provenance`; unsupported
fingerprint algorithm, imported identity, clause-content fingerprint, or
proof-status mismatch is `unresolved_symbol`.

There is no caller-supplied `imported_clause_context` in `KernelCheckInput`.
The only imported-clause context passed to `resolution_trace` is the
checker-owned context constructed from imported facts that passed identity,
fingerprint, proof-status, and clause-validation checks. This prevents
unchecked clauses from bypassing imported-fact validation.

`cluster_trace_context` remains an explicit evidence input. Task 15 implements
bounded replay of requested cluster/reduction trace ids by checking explicit
dependencies and normalized commitments. Producer-side generation of richer
active-rule payloads remains `external_dependency_gap`/`deferred`; missing or
unsupported payload evidence is rejected rather than accepted by a placeholder.

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
  generated_fact_fingerprint

ReductionStepEvidence
  reduction_step_id
  applied_reduction
  rule_fqn
  enclosing_term_before
  redex_path
  source_redex
  target_term
  substitution
  required_guard_ids
  discharged_guards
  rule_view
  selection_key
  strategy_audit_key
  result_fingerprint

GuardEvidence
  guard_id
  source_fact_ref
  checked_dependency_ref
```

Strategy-audit fields such as `enclosing_term_before`, `redex_path`,
`rule_view`, and `selection_key` are checked as bounded recorded evidence and
bound to normalized commitments. Task 15 does not search for an alternate redex
or rule, and it does not infer missing active-rule data from registrations.

Task 15 treats `generated_fact_fingerprint`, `strategy_audit_key`, and
`result_fingerprint` as normalized replay commitments. They are not backend
assertions: the kernel recomputes deterministic canonical bytes from the
recorded step fields and rejects mismatches as `invalid_cluster_trace`.
`strategy_audit_key` is recomputed from `enclosing_term_before`, `redex_path`,
`rule_view`, and `selection_key`. Unsupported upstream trace payload production
remains an `external_dependency_gap`; runtime behavior must stay fail-closed.

Cluster and reduction step ids share one global ordered trace namespace.
`cluster_steps` and `reduction_steps` may be stored in separate sorted vectors
for type safety, but their ids must not overlap. A trace step may depend only
on imported/generated base facts or on a trace step with a strictly smaller id
that has already been replayed. Current-step and future-step dependencies are
`invalid_cluster_trace`.

Cluster trace context is required only when the certificate or checker service
requests replay of one or more cluster/reduction trace step ids. If no trace
step is requested, absent context is accepted and no cluster evidence is
checked. When trace ids are requested, context and provenance are mandatory;
the kernel replays the requested ids plus their explicit transitive trace-step
dependencies in global id order. Bounded but unrequested context entries are
ignored after constructor checks and are not counted against replay-time
cluster/reduction step limits.

Reduction rule authority is explicit evidence, not a lookup. Task 15 requires
the authority fields (`applied_reduction`, `rule_fqn`, `rule_view`,
`redex_path`, `source_redex`, `target_term`, `substitution`,
`required_guard_ids`, and `discharged_guards`) to be present, bounded, and
bound into normalized commitments. It does not yet semantically validate that
`redex_path` selects `source_redex` inside `enclosing_term_before` or that the
recorded local `LHS -> RHS` instance follows from a richer active-rule payload;
that producer-side payload format remains `external_dependency_gap` until it
is documented.

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
  checked_reduction_steps
  checked_derived_facts
  final_goal
  used_axioms
  policy_taint
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

CheckedFinalGoal
  namespace
  id
```

`accepted` means the normalized certificate's final goal was checked by this
crate. It does not encode artifact-facing proof status. `mizar-proof` or a
later policy layer may project accepted ATP certificates as `kernel_verified`
and accepted built-in certificates as `discharged_builtin`, but that projection
is outside this crate.

`used_axioms` is derived only from checked imported axiom/theorem references
actually used by the accepted certificate. Backend-reported used-axiom lists
are ignored unless the normalized certificate and imported-fact context make
the same facts checkable. Imported axiom and imported theorem ids remain
separate namespaces when deriving `used_axioms` and constructing
`CheckedFactContext`; `imported_axiom(1)` and `imported_theorem(1)` are
distinct checked facts.

If any checked import has `accepted_proof_status =
externally_attested_policy_permitted`, the accepted kernel result carries a
policy taint on that import and on the aggregate result. A policy layer must
not project such a result as unqualified `kernel_verified`; it may only emit a
policy-controlled externally attested or mixed-status result. If the active
release policy forbids that taint, the immutable imported-fact context must not
present the external status as satisfying the requirement.

Task 16 provides an in-crate batch helper that checks independent inputs and
sorts results by target VC fingerprint, then by caller input order for equal
targets. It does not spawn workers or read cancellation tokens; external
scheduler integration remains outside this crate.

Batch checking is a deterministic wrapper around single-certificate checks:

```text
KernelCheckBatchInput
  checks: Vec<KernelCheckInput> in caller order

KernelCheckBatchResult
  results: sorted Vec<KernelCheckResult>
```

Batch results are sorted by target VC fingerprint, then by original caller
input index for equal targets. Evidence ids, worker completion order,
cancellation arrival order, or parallel scheduling must not affect result
order inside this crate.

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
7. Replay requested explicit cluster/reduction trace step ids, plus their
   explicit trace-step dependencies, when the certificate or checker service
   requests nonempty cluster evidence.
8. Reject `derived_facts` unless a documented payload schema is implemented.
   Task 16 does not invent a derived-fact payload language.
9. Validate `final_goal` by resolving the referenced generated clause,
   resolution step, or supported derived fact and checking that it is the
   empty obligation or canonical final fact required by the target VC.
10. Emit one accepted result, or a deterministic rejected result containing the
    earliest stable rejection record for that failed check.

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
context records the policy-permitted status. Task 14 receives an explicit
profile-policy gate for externally attested imports; if that gate disallows
external attestation, evidence with
`externally_attested_policy_permitted` is rejected as `unresolved_symbol` even
when the parsed requirement would otherwise allow it. Release policies that
forbid external attestations remain outside this module, but their decision is
represented by that immutable input gate rather than by a global lookup.

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
- imported dependencies to preserve their axiom/theorem namespace; a trace
  cannot refer to an ambiguous imported fact id;
- replay to be driven by requested trace step ids; unused evidence is ignored
  after bounded construction;
- cluster and reduction steps to share a single numeric trace order;
- reduction rule authority fields (`applied_reduction`, `rule_fqn`, selected
  redex, local rewrite instance, and required guards) to be represented in
  explicit normalized evidence;
- cluster generated-fact and reduction result commitments to be recomputed
  deterministically from recorded fields before acceptance;
- reduction required guards to be matched exactly by discharged guard evidence;
- invalid cluster/reduction evidence to map to `invalid_cluster_trace`;
- missing trace context or missing trace provenance to map to
  `missing_provenance`.

If the upstream `mizar-checker` cluster trace payload is not ready when task 15
starts, that task must record the gap as `external_dependency_gap`/`deferred`
and keep runtime behavior fail-closed.

## Derived Facts And Final Goal

`ParsedCertificate.derived_facts` are certificate-owned assembly records. Task
16 does not define a new derived-fact payload schema. Any nonempty
`derived_facts` vector, or a `final_goal` that points to `DerivedFact`, is
rejected fail-closed as `invalid_sat_proof` until a later paired spec and
implementation define the payload grammar. External attempts to supplement
derived-fact payloads through context maps are not accepted.

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
- `derived_fact` goals are rejected by task 16 because no derived-fact payload
  schema is implemented yet;
- a `generated_clause` final goal is accepted only when that generated clause
  is consumed by a successful checked replay path; in task 16 that path is the
  checked resolution final-goal helper. Mere presence in
  `ParsedCertificate.generated_clauses` is not proof acceptance;
- missing, unchecked, forward, or mismatched final-goal references are
  `invalid_sat_proof` or `invalid_cluster_trace` according to the failed
  evidence family;
- target mismatch is `context_mismatch`.

Task 16 accepts final goals only through the resolution report binding helper:
`GeneratedClause` and `ResolutionStep` final goals must be checked resolution
outputs and must be the canonical empty clause. It does not trust
caller-supplied sub-checker reports or unchecked generated clauses.

## Limits

`CheckerLimits` collects deterministic budgets and forwards the relevant
subsets to sub-checkers:

```text
CheckerLimits
  parser limits
  resolution replay limits
  substitution replay limits
  imported fact count
  imported fact context entry count
  imported clause validation limits
  cluster trace step count
  reduction trace step count
  cluster trace field byte count
  reduction guard evidence count
  reduction substitution binding count
  normalized commitment byte count
  checker pipeline step budget
  derived fact count
  final report record count
```

Exceeding a checker-owned budget is `resource_exhaustion`. Budget checks must
run before allocating large temporary vectors, sorting unbounded context
entries, cloning imported clauses, or materializing reports.

## Rejection Mapping

| Failure | Detail | Location |
|---|---|---|
| Missing imported fact context, requested cluster trace context/provenance, substitution context, derived imported-clause context, or provenance | `missing_provenance` | field path plus imported fact, substitution, cluster, reduction, or final-goal id when known |
| Malformed external service witness envelope before parsing or before normalized evidence can be selected | `malformed_witness_data` | service evidence field path; external integration only, not produced by task 16 |
| Imported fact identity, statement fingerprint, unavailable theorem/axiom, or proof-status strength mismatch | `unresolved_symbol` | `imported_fact_id` |
| Substitution replay failure | forwarded `invalid_substitution`, `missing_provenance`, or `resource_exhaustion` | forwarded substitution location |
| Resolution replay failure | forwarded `invalid_sat_proof`, `missing_provenance`, or `resource_exhaustion` | forwarded clause or resolution-step location |
| Cluster/reduction trace replay failure | `invalid_cluster_trace` | cluster or reduction step id |
| Nonempty derived facts, derived-fact final goals, or derived payload validation before a documented schema exists | `invalid_sat_proof` | `derived_fact_id`; richer payload/dependency validation is deferred to a later schema task |
| Final goal mismatch or unchecked final reference | `invalid_sat_proof` | `final_goal` plus referenced id when known |
| Target VC or context binding mismatch | `context_mismatch` | target/context field path |
| Unsupported checker or certificate profile | `unsupported_certificate_format` | profile field path |
| Checker-owned deterministic resource budget exhausted | `resource_exhaustion` | checker budget field path |
| In-crate deterministic checker step budget exhausted | `timeout` | checker step field path |
| External cancellation budget exhausted after parsing | `timeout` | cancellation field path; external integration only, not produced by task 16 |

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

Task 16 exposes a deterministic checker step budget. A stopped check returns
`timeout`, never partial acceptance. Parser-owned malformed bytes remain
`malformed_certificate`. Service-envelope witness normalization and external
cancellation-token plumbing are integration work outside this crate; until a
documented envelope is provided, they remain `external_dependency_gap` and are
not mocked by the kernel service.

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
  translation, cluster trace payload production by `mizar-checker`, derived
  fact payload schemas, service-envelope witness normalization, cancellation
  token plumbing, and proof/cache/artifact consumers are not active inputs to
  this crate. Missing producer or consumer integration is not mocked here.
- `deferred`: proof-policy projection, witness storage, cache reuse, artifact
  emission, backend-candidate selection, and external worker scheduling remain
  outside `mizar-kernel`.

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
- hidden transitive expansion, malformed or over-budget reduction substitution
  evidence, missing guard evidence, dependency mismatch, and strategy-audit or
  result-commitment mismatch rejected as `invalid_cluster_trace` or
  `resource_exhaustion` according to the failed check;
- missing cluster trace context/provenance rejected as `missing_provenance`
  when nonempty trace ids are requested;
- fail-closed `external_dependency_gap` behavior if upstream trace payloads are
  not ready.

Task 16 must add Rust tests for:

- full pipeline acceptance from checked imports, substitutions, resolution
  trace, optional cluster trace, and final goal;
- final-goal mismatch and unchecked final references rejected deterministically;
- duplicate context ids, duplicate evidence ids, simultaneous imported/cluster
  context failures resolved by the fail-fast pipeline, and single rejection
  records with stable locations;
- report/input binding preventing accidental reuse of sub-checker reports;
- deterministic result ordering under shuffled context construction and
  shuffled caller input order, including equal-target tie preservation;
- policy taint propagation for externally attested imported facts;
- nonempty or final-goal-referenced derived facts rejected fail-closed until a
  documented payload schema exists;
- deterministic checker step budgets rejected as `timeout`, checker-owned
  deterministic resource limits rejected as `resource_exhaustion`, and
  service-envelope witness normalization/cancellation token plumbing recorded
  as `external_dependency_gap` rather than mocked;
- the trusted-boundary lint/test set mirrors the trust statement: no proof
  search, ATP search, external SAT/ATP process invocation, premise selection,
  overload resolution, cluster search, registration activation, implicit
  coercion insertion, fallback inference, source loading, hidden
  dependency-artifact reads, ATP/proof/cache/artifact coupling, unordered
  iteration, wall-clock/random read, or global mutable-state read; trusted
  in-process SAT checking over the kernel-built evidence problem is covered by
  post-closeout tasks 23-29.
