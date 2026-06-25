# Module: rejection

> Canonical language: English. Japanese companion:
> [../ja/rejection.md](../ja/rejection.md).

## Purpose

The `rejection` module owns the stable rejection vocabulary for
`mizar-kernel`. It refines
[architecture 15](../../architecture/en/15.kernel_certificate_format.md)
"Kernel Rejection Semantics" and
[architecture 19](../../architecture/en/19.failure_semantics.md).

Rejections are non-acceptance outcomes. They must be deterministic,
machine-readable, and stable across diagnostic wording changes. A backend
success report, cache hit, artifact witness, or policy projection must never
turn a kernel rejection into acceptance.

## Trust Statement

This module is trusted kernel code. It must define only explicit, structured
failure records and deterministic ordering rules.

The module must not perform proof search, premise selection, overload
resolution, cluster search, ATP search, implicit coercion insertion, fallback
inference, imported-fact lookup, cache lookup, artifact lookup, wall-clock or
random-state reads, unordered iteration, or hidden reads of mutable
compiler-global state.

## Owned Behavior

The module owns:

- phase-level rejection categories used by the kernel;
- stable structured detail keys used by parser and checker modules;
- deterministic locations for certificate bytes and checked evidence objects;
- deterministic ordering for multiple rejection records;
- compatibility rules for adding or renaming public rejection values;
- conversion policy from parser-owned errors to shared rejection records.

The module does not own:

- human diagnostic wording;
- source-level parse, resolve, type, overload, or cluster-loop failures;
- ATP backend timeout classification before a normalized certificate exists;
- proof-policy projection or user-facing proof status;
- cache, artifact, or proof-witness publication.

## Categories

`mizar-kernel` uses these phase-level categories:

| Category | Owner | Meaning |
|---|---|---|
| `certificate_rejection` | `certificate_parser` and checker-service witness normalization | The normalized certificate envelope, service-level witness envelope, schema, context binding, structural bytes, canonical ordering, or parser resource bounds are invalid. |
| `kernel_rejection` | Later checker modules | The parsed certificate is structurally readable, but replay or evidence checking fails. |

Architecture 19 also defines broader pipeline categories such as
`parse_error`, `resolve_error`, `type_error`, `overload_ambiguity`,
`cluster_loop`, and `atp_timeout`. Those are not owned by `mizar-kernel`.
When classification is ambiguous, choose the earliest phase that can soundly
detect the failure and fail closed.

## Stable Details

Detail keys are stable API values. Diagnostic text may change without changing
these keys.

| Detail key | Category | Meaning |
|---|---|---|
| `unsupported_certificate_format` | `certificate_rejection` | Unsupported domain separator, schema version, encoding version, kernel profile, hash-input algorithm, or unknown required section tag. |
| `context_mismatch` | `certificate_rejection` | The certificate is bound to a target VC or explicit context identity that does not match the caller-supplied kernel context. Unsupported profiles remain `unsupported_certificate_format`. |
| `malformed_certificate` | `certificate_rejection` | Malformed envelope, directory, item frame, field encoding, canonical order, duplicate id, structural reference, or generated-clause payload. |
| `malformed_witness_data` | `certificate_rejection` | Kernel check service witness data is structurally malformed before it can be treated as a normalized certificate or checked evidence object. |
| `missing_provenance` | `kernel_rejection` | Required candidate provenance, premise provenance, or immutable context provenance is absent, so the kernel cannot verify where the evidence came from. |
| `resource_exhaustion` | `certificate_rejection` or `kernel_rejection` | A deterministic size, count, recursion, memory, trace-length, or replay-cost limit is exceeded. |
| `invalid_substitution` | `kernel_rejection` | Capture avoidance, alpha-conversion, freshness, or free-variable side-condition replay fails. |
| `invalid_sat_proof` | `kernel_rejection` | Clause or MiniSAT-compatible resolution replay fails, including a pivot, parent, resolvent, or final-goal derivation mismatch. |
| `invalid_cluster_trace` | `kernel_rejection` | Explicit cluster or reduction trace replay fails, including hidden transitive expansion, invalid reduction substitution, or strategy-audit mismatch. |
| `unresolved_symbol` | `kernel_rejection` | A referenced symbol, imported theorem, imported axiom, VC, content fingerprint, or required imported proof status is unavailable, mismatched, or weaker than the current kernel profile permits. |
| `timeout` | `kernel_rejection` | A deterministic checker budget stops replay. Pure parser failures never emit this detail. |

`timeout` and `resource_exhaustion` are not proof acceptance. They leave the
obligation unverified.

## Rejection Record

Task 7 should implement a shared record shaped like:

```text
RejectionRecord
  target_vc_fingerprint
  category
  detail
  location
  stable_detail_key
```

`target_vc_fingerprint` is the deterministic sort key supplied by the kernel
check input. Parser conversions always attach and order by the caller-supplied
expected target VC. A certificate's claimed `target_vc`, even when readable, is
diagnostic context only and must not become the ordering owner for a
`context_mismatch`.

`stable_detail_key` is the canonical snake-case spelling of `detail`. It is
used for snapshots, diagnostic ordering, and compatibility review. The record
may later carry diagnostic-facing context, but such context must not affect
acceptance or ordering.

Public rejection enums should be `#[non_exhaustive]` unless the public-enum
forward-compatibility audit records a justified exception.

## Locations

Rejection locations are deterministic and evidence-owned. They do not depend on
source paths, source ranges, display names, backend logs, allocation addresses,
wall-clock time, worker completion order, cache keys, artifact paths, or map/set
iteration order.

```text
RejectionLocation
  certificate_byte_offset?
  section_tag?
  item_index?
  field_path?
  clause_ref?
  resolution_step_id?
  substitution_id?
  imported_fact_id?
  cluster_trace_step_id?
  reduction_step_id?
  derived_fact_id?
  final_goal?
```

Rules:

- parser failures must preserve byte offsets and any known section, item, and
  field path;
- resolution failures must identify the failed step and the most precise parent,
  pivot, generated clause, or final-goal reference available;
- substitution failures must identify the substitution entry and the failed
  source, target, freshness, alpha, or free-variable field;
- imported-fact failures must identify the imported fact namespace and stable
  imported fact id;
- cluster and reduction trace failures must identify the explicit trace step
  and failed source type, rule, substitution, guard, or strategy-audit field
  when available;
- a location may be partial when earlier malformed bytes prevent a more precise
  reference, but it must remain deterministic.

## Parser Mapping

`certificate_parser` task 5 already exposes module-local parse errors. Task 7
must map them without losing information:

| Parser detail | Shared detail |
|---|---|
| `UnsupportedCertificateFormat` | `unsupported_certificate_format` |
| `ContextMismatch` | `context_mismatch` |
| `MalformedCertificate` | `malformed_certificate` |
| `ResourceExhaustion` | `resource_exhaustion` |

All parser errors keep category `certificate_rejection`. Parser errors must not
be remapped to `kernel_rejection`, `timeout`, or an ATP/backend failure.

## Checker Mapping

Later checker modules map failures as follows:

| Checker owner | Failure | Detail |
|---|---|---|
| `resolution_trace` | pivot polarity mismatch, parent lookup mismatch, resolvent mismatch, generated-clause mismatch, non-final derivation | `invalid_sat_proof` |
| `substitution_checker` | capture, alpha-conversion, freshness, or free-variable violation | `invalid_substitution` |
| `checker` cluster replay | hidden transitive expansion, cyclic explicit trace, invalid reduction substitution, guard mismatch, or strategy-audit mismatch | `invalid_cluster_trace` |
| `checker` imported facts | missing imported theorem or axiom, fingerprint mismatch, unavailable VC or symbol, or imported theorem accepted under a weaker status than the current kernel profile permits | `unresolved_symbol` |
| `checker` service | missing candidate, premise, or immutable context provenance | `missing_provenance` |
| `checker` service | malformed service-level witness data before normalized certificate replay | `malformed_witness_data` |
| `checker` service | deterministic replay budget stop | `timeout` or `resource_exhaustion`, depending on whether the budget is time-step style or size/memory/count style |

The kernel must never synthesize missing premises, search for alternate
parents, insert coercions, expand clusters, or accept because a backend said the
obligation was proved.

## Deterministic Ordering

When multiple rejections are reported in one batch, order them by:

1. target VC fingerprint bytes;
2. category order: `certificate_rejection`, then `kernel_rejection`;
3. certificate byte offset when present;
4. section tag, item index, and field path when present;
5. stable evidence id: imported fact id, generated clause id, resolution step
   id, substitution id, cluster trace step id, reduction step id, derived fact
   id, final-goal marker;
6. stable detail key.

Parallel checking and worker completion order must not affect this order.

## Compatibility Policy

Stable category and detail keys are snapshot-facing API. Changing their spelling,
meaning, phase ownership, or ordering requires a compatibility review and
public-enum policy update. Adding a new detail key is allowed only when:

- no existing key precisely describes the rejection;
- the owning phase is documented;
- tests assert its category, detail key, and deterministic location;
- source/documentation consistency is reviewed in the same task.

Human diagnostic messages may improve without compatibility review if the
stable category, detail, ordering, and location semantics remain unchanged.

## Gap Classification

- `spec_gap`: architecture 15 names rejection reasons but not the shared
  `mizar-kernel` record shape, parser mapping, checker mapping, or deterministic
  location rules. This module spec closes that gap for task 6.
- `test_gap`: task 7 still needs Rust tests for stable category/detail keys,
  parser error conversion, deterministic ordering, `#[non_exhaustive]` public
  enums, and the guarantee that pure parser failures never become `timeout`.
- `external_dependency_gap`: `mizar-proof`, `mizar-cache`, and
  `mizar-artifact` are not active consumers of kernel rejection records; do not
  add placeholder policy/cache/artifact coupling here.
- `deferred`: source-derived `.miz` rejection snapshots and downstream proof
  policy projection remain outside task 6.

## Planned Tests

Task 7 must add Rust tests for:

- one stable key per `FailureCategory` and rejection detail;
- allowed category/detail pairs and rejected invalid mappings, including parser
  details never becoming `kernel_rejection` and `timeout` never becoming
  `certificate_rejection`;
- parser-error conversion preserving `target_vc_fingerprint`, including the
  caller-supplied fallback when malformed bytes prevent reading `target_vc`,
  plus category, detail, byte offset, section, item index, and field path;
- checker-side locations preserving `resolution_step_id`, `substitution_id`,
  `clause_ref`, `imported_fact_id`, `cluster_trace_step_id`,
  `reduction_step_id`, `derived_fact_id`, and `final_goal`;
- checker-service mapping for `missing_provenance` to `missing_provenance`,
  `malformed_witness_data` to `malformed_witness_data`, and imported
  proof-status mismatch to `unresolved_symbol`;
- checker-owner mappings for resolution failures to `invalid_sat_proof`,
  substitution failures to `invalid_substitution`, cluster/reduction failures
  to `invalid_cluster_trace`, and missing or fingerprint-mismatched imported
  facts to `unresolved_symbol`;
- deterministic ordering independent of insertion or worker completion order,
  with explicit tie-breaker cases for `target_vc_fingerprint`, category order,
  byte offset, section/item/field, checker evidence ids, and stable detail key;
- pure parser failures never mapping to `timeout`;
- `timeout` and `resource_exhaustion` remaining non-acceptance outcomes;
- public rejection enums being forward-compatible per the planned
  `#[non_exhaustive]` policy;
- lint coverage showing rejection code does not import ATP/proof/cache/artifact
  crates, read imported facts from global state, read global mutable state, use
  wall-clock/random APIs, perform proof/premise/cluster/ATP search, resolve
  overloads, insert implicit coercions, fall back to inference, or depend on
  unordered map/set iteration.

No `.miz` fixture, expectation sidecar, `doc/spec`, or Rust source change is
required for this module-spec task.
