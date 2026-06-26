# Source/Spec Audit: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/source_spec_audit.md](../ja/source_spec_audit.md).

## Scope And Method

Task 20 audits the implemented `mizar-kernel` public surface against the paired
module specifications and the trusted-kernel prohibition boundary. The audit is
source-derived and deterministic: the crate root's public module exports and
each module's externally public top-level `pub` items are listed below. Public
fields, inherent methods, enum variants, trait impls, and private helpers are
covered by the owning public type and its module spec; they are not separately
enumerated by the mechanical guard.

The audit is intentionally not a behavior change. It records current
source/spec correspondence, classifies remaining gaps, and adds a lint guard so
future public API additions must update this audit and the module trust
statements. It does not add SAT solving, ATP backend invocation, proof search,
premise selection, overload resolution, cluster search, implicit coercion
insertion, fallback inference, source loading, cache lookup, artifact lookup,
or mutable compiler-global state access.

## Crate Module Exports

`src/lib.rs` exposes exactly these spec-backed modules:

- `certificate_parser` -> source `src/certificate_parser.rs`, spec
  [certificate_parser.md](./certificate_parser.md).
- `checker` -> source `src/checker.rs`, spec [checker.md](./checker.md).
- `clause` -> source `src/clause.rs`, spec [clause.md](./clause.md).
- `rejection` -> source `src/rejection.rs`, spec [rejection.md](./rejection.md).
- `resolution_trace` -> source `src/resolution_trace.rs`, spec
  [resolution_trace.md](./resolution_trace.md).
- `substitution_checker` -> source `src/substitution_checker.rs`, spec
  [substitution_checker.md](./substitution_checker.md).

## Public Surface Inventory

### `certificate_parser`

Source: `src/certificate_parser.rs`. Spec: [certificate_parser.md](./certificate_parser.md).

Covered top-level public items:

- `CertificateParseContext`
- `CertificateParseLimits`
- `ClauseValidationPolicy`
- `KernelProfileRecord`
- `ClauseTautologyPolicy`
- `CertificateHashInputAlgorithm`
- `Fingerprint`
- `ParsedCertificate`
- `SymbolManifestEntry`
- `VariableManifestEntry`
- `ImportedFactRef`
- `RequiredProofStatus`
- `GeneratedClause`
- `SubstitutionEntry`
- `ResolutionStep`
- `DerivedFact`
- `FinalGoalRef`
- `ClauseRef`
- `ClauseRefNamespace`
- `FinalGoalNamespace`
- `CertificateParseError`
- `FailureCategory`
- `CertificateRejectionDetail`
- `CertificateParseLocation`
- `SectionTag`
- `parse_certificate`

Correspondence summary:

- `CertificateParseContext`, `CertificateParseLimits`, and
  `ClauseValidationPolicy` implement the spec's parser resource and validation
  controls.
- `KernelProfileRecord`, manifests, references, generated clauses,
  substitution entries, resolution steps, derived facts, final-goal references,
  and section tags are the normalized certificate schema owned by this module.
- `Fingerprint`, `CertificateHashInputAlgorithm`, `CertificateParseError`,
  `FailureCategory`, `CertificateRejectionDetail`, and
  `CertificateParseLocation` implement deterministic hash-input and parser
  rejection records without granting semantic trust.
- `parse_certificate` performs byte parsing and structural validation only.

### `checker`

Source: `src/checker.rs`. Spec: [checker.md](./checker.md).

Covered top-level public items:

- `SUPPORTED_NORMALIZED_CLAUSE_FINGERPRINT_ALGORITHM_ID`
- `ImportedFactCheckLimits`
- `ImportedFactCheckInput`
- `ImportedFactPolicy`
- `ImportedFactContextLimits`
- `ImportedFactContext`
- `ImportedFactContextError`
- `ImportedFactEvidence`
- `ImportedFactNamespace`
- `AcceptedProofStatus`
- `ImportedFactCheckReport`
- `CheckedImportedFact`
- `ImportedFactCheckResult`
- `check_imported_facts`
- `KernelCheckInput`
- `KernelCheckPolicy`
- `KernelCheckLimits`
- `KernelCheckResult`
- `KernelCheckStatus`
- `CheckedDerivedFact`
- `CheckedFinalGoal`
- `UsedAxiom`
- `KernelCheckServiceResult`
- `check_kernel_certificate`
- `check_kernel_batch`
- `ClusterTraceReplayLimits`
- `ClusterTraceReplayInput`
- `CheckedFactContext`
- `ClusterTraceContext`
- `ClusterTraceContextError`
- `BaseFactNamespace`
- `ClusterStepEvidence`
- `ReductionStepEvidence`
- `ReductionBindingEvidence`
- `GuardEvidence`
- `CheckedFactRef`
- `ClusterTraceReplayReport`
- `CheckedClusterStep`
- `CheckedReductionStep`
- `ClusterTraceReplayResult`
- `replay_cluster_trace`

Correspondence summary:

- Imported-fact context, policy, status, evidence, report, result, and
  `check_imported_facts` implement the spec's immutable imported-fact
  validation boundary.
- `KernelCheckInput`, `KernelCheckPolicy`, `KernelCheckLimits`,
  `KernelCheckResult`, `KernelCheckStatus`, checked output records, service
  result alias, `check_kernel_certificate`, and `check_kernel_batch` implement
  policy-independent phase-14 orchestration and deterministic batch ordering.
- Cluster/reduction context, evidence, checked-reference, report, result, and
  `replay_cluster_trace` replay explicit traces only. They do not perform
  cluster or reduction search.

### `clause`

Source: `src/clause.rs`. Spec: [clause.md](./clause.md).

Covered top-level public items:

- `ClauseProfile`
- `TautologyPolicy`
- `ClauseValidationContext`
- `Clause`
- `ClauseForm`
- `Literal`
- `Polarity`
- `Atom`
- `Term`
- `SymbolKey`
- `SymbolId`
- `VariableId`
- `SymbolKind`
- `ClauseError`

Correspondence summary:

- Profiles, validation context, tautology policy, clause forms, literals,
  atoms, terms, symbols, variables, and symbol kinds implement canonical clause
  representation and deterministic rendering.
- `ClauseError` covers structural well-formedness and resource failures owned
  by this module.

### `rejection`

Source: `src/rejection.rs`. Spec: [rejection.md](./rejection.md).

Covered top-level public items:

- `TargetVcFingerprint`
- `RejectionCategory`
- `RejectionDetail`
- `ClauseRefNamespace`
- `ClauseRef`
- `RejectionLocation`
- `RejectionRecord`
- `RejectionRecordError`

Correspondence summary:

- Target fingerprints, categories, details, clause references, locations, and
  records implement stable deterministic rejection output.
- `RejectionRecordError` guards construction-time category/detail and
  reference-shape mismatches.

### `resolution_trace`

Source: `src/resolution_trace.rs`. Spec: [resolution_trace.md](./resolution_trace.md).

Covered top-level public items:

- `ResolutionReplayLimits`
- `ResolutionTraceInput`
- `ImportedClauseEntry`
- `ImportedClauseContext`
- `ImportedClauseContextError`
- `ResolutionReplayReport`
- `CheckedResolutionStep`
- `ResolutionReplayResult`
- `replay_resolution_trace`
- `checked_resolution_final_goal`

Correspondence summary:

- Replay limits, trace input, imported clause context, report, checked steps,
  and result alias implement deterministic MiniSAT-compatible resolution trace
  replay.
- `replay_resolution_trace` and `checked_resolution_final_goal` check explicit
  parent clauses and final-goal binding; they do not invoke a SAT solver or ATP
  backend.

### `substitution_checker`

Source: `src/substitution_checker.rs`. Spec: [substitution_checker.md](./substitution_checker.md).

Covered top-level public items:

- `SubstitutionReplayLimits`
- `SubstitutionCheckInput`
- `SubstitutionPayloadEntry`
- `SubstitutionPayload`
- `Replacement`
- `FreshnessWitness`
- `FreeVariableConstraint`
- `TermPath`
- `TermPathSegment`
- `SubstitutionContext`
- `SubstitutionContextError`
- `SubstitutionCheckReport`
- `CheckedSubstitution`
- `SubstitutionCheckResult`
- `replay_substitutions`
- `checked_substitutions_for_input`

Correspondence summary:

- Replay limits, input, payloads, replacements, witnesses, free-variable
  constraints, term paths, and context implement explicit substitution,
  alpha/freshness, and free-variable evidence replay.
- Reports, checked substitutions, result alias, and replay helpers expose only
  checked outputs. Missing or malformed evidence is rejected rather than
  repaired or inferred.

## Trust Statement Audit

Each source-backed exported module specification has a `## Trust Statement`
section and is guarded to contain the trusted-kernel statement plus the full
task-20 prohibition family. Task 23 corrects the SAT wording: proof search, ATP
search or backend invocation, premise selection, overload resolution, cluster
search, implicit coercion insertion, fallback inference, backend-reported
success alone, source loading, cache lookup, artifact lookup, wall-clock or
random-state reads, unordered iteration dependence, and hidden reads of mutable
compiler-global state remain forbidden. Trusted SAT checking is allowed only
over SAT problems derived by the kernel from validated formula/substitution
evidence.

The new task-23 `formula_evidence`, `sat_encoding`, and `sat_checker` specs are
planned/unimplemented design surfaces. They are intentionally not included in
the executable source-backed guard until tasks 25-27 add the corresponding
exported modules.

## Post-Closeout Correction Addendum

Task 23 adds the corrected design surface before source changes:

- `formula_evidence.md` defines the kernel-owned formula/substitution evidence
  schema and legacy unsupported handling;
- `sat_encoding.md` defines kernel-derived deterministic SAT encoding;
- `sat_checker.md` defines the trusted in-process Rust SAT checker wrapper.

Task 24 adds the dependency audit before source changes:

- `sat_dependency_audit.md` records the task-24 selection of direct
  `batsat = { version = "=0.6.0", default-features = false }`, rejected
  candidates, unsafe-code audit, no-process/no-network audit, resource-limit
  gates, and the dependency lint-policy revision that task 27 must encode.

The current source inventory above remains the task-22 legacy public surface
until tasks 25-29 add the new modules and gate or retire legacy
resolution-trace acceptance. The legacy `check_kernel_certificate` path is
classified as `source_drift` / `design_drift` against the corrected evidence
format and must not be treated as the target normal proof policy.

## Test Traceability

The public surface above is exercised by module-local Rust tests and the
cross-module lint guard. Task 20 does not create source-derived `.miz` evidence
fixtures. After the task-23 correction, future corpus coverage must target
source-derived formula/substitution evidence; legacy certificate-runner work is
migration-only and remains deferred.

| Module / boundary | Test path | Covered behavior |
|---|---|---|
| `certificate_parser` | `crates/mizar-kernel/src/certificate_parser/tests.rs` | Valid schema parsing, unsupported headers/profiles, directory and item canonicality, resource exhaustion before allocation, imported fact references, manifest/generated-clause validation, substitution/resolution/derived/final references, deterministic collection order, deterministic hash input, and parser rejection classification. |
| `checker` imported facts | `crates/mizar-kernel/src/checker/tests.rs` | Imported axiom/theorem context validation, namespace preservation, proof-status checks, policy taint, fingerprint binding, duplicate context rejection, unused malformed entry handling, deterministic context/report ordering, and count/resource limits. |
| `checker` cluster/reduction replay | `crates/mizar-kernel/src/checker/tests.rs` | Valid trace replay, missing provenance, hidden/future dependency rejection, guard/result mismatches, bounded context construction, requested-step closure, unchecked base fact rejection, runtime limits, and deterministic canonical order. |
| `checker` service orchestration | `crates/mizar-kernel/src/checker/tests.rs` | Accepted service pipeline, substitution/report binding, generated-clause base sets, final-goal and derived-fact fail-closed behavior, mutation fail corpus, deterministic repetition/permutation results, deterministic batch ties, replay-cost budgets, timeout/resource propagation, and target/input-order batch sorting. |
| `clause` | `crates/mizar-kernel/src/clause/tests.rs` | Canonical literal/term ordering, duplicate literal removal, empty versus tautology forms, tautology policy, malformed atom/term/symbol/variable rejection, profile/resource bounds, canonical constructor checks, stable rendering, and hash input exclusion of display data. |
| `rejection` | `crates/mizar-kernel/src/rejection/tests.rs` | Stable keys, category/detail ownership, parser conversion, checker locations, owner mappings, deterministic ordering and tie-breakers, fixed-width target sort bytes, and public enum compatibility. |
| `resolution_trace` | `crates/mizar-kernel/src/resolution_trace/tests.rs` | Valid replay over generated/imported/previous-step parents, pivot and resolvent rejection, imported context sorting/provenance, first-use compatibility/depth checks, resource limits, tautology policy, defensive invariant rejection, final-goal checkedness, deterministic reports, deterministic rejection locations, and clause-owned depth/length helpers. |
| `substitution_checker` | `crates/mizar-kernel/src/substitution_checker/tests.rs` | Direct substitution replay, payload role validation, missing/malformed/deferred evidence rejection, target/manifest/capture checks without repair, alpha conversion, freshness witnesses, free-variable constraints, shuffled witness determinism, binder-context decoding, first-use side-condition rejection, resource limits, context canonicalization, and report binding. |
| Public-surface and trust lint | `crates/mizar-kernel/tests/lint_policy.rs` | Workspace/crate dependency boundary, source module exposure, public enum policy, forbidden producer/cache/artifact/nondeterminism tokens, exact source/spec audit inventory, task-22 private-test traceability and tracked-file guard, Trust Statement prohibition wording, gap classification markers, and scanner regression cases. |

## Gap Classification

| ID | Class | Evidence | Current action |
|---|---|---|---|
| KERNEL20-G001 | `external_dependency_gap` / `deferred` | Source-derived certificate and service envelopes are not produced by an active upstream crate or corpus runner. | Keep Rust fixture coverage and reject missing evidence; do not fabricate source-derived runner support. |
| KERNEL20-G002 | `external_dependency_gap` / `deferred` | Formula/substitution evidence candidate production is producer-owned and not available as a stable `mizar-atp` contract. ATP proof translation and MiniSAT-compatible backend trace extraction are legacy migration material, not trusted output. | Kernel checks normalized formula/substitution evidence after tasks 25-28; no ATP backend invocation or trusted proof translation is added. |
| KERNEL20-G003 | `external_dependency_gap` / `deferred` | Cluster/reduction payload production by `mizar-checker` is not a ready integration contract. | Kernel replays explicit cluster/reduction payloads only; no cluster search or payload synthesis is added. |
| KERNEL20-G004 | `external_dependency_gap` / `deferred` | Derived-fact payload schema beyond current explicit checked inputs remains downstream/provenance-owned. | Derived facts remain fail-closed unless backed by checked evidence. |
| KERNEL20-G005 | `external_dependency_gap` / `deferred` | Service-envelope normalization, cancellation token plumbing, and external worker scheduling are integration concerns outside the crate. | In-crate checks remain deterministic and synchronous over immutable inputs. |
| KERNEL20-G006 | `external_dependency_gap` / `deferred` | Downstream `mizar-proof`, `mizar-cache`, and `mizar-artifact` consumers are not ready proof-policy/cache/artifact contracts. | No dependency or placeholder integration is added. |
| KERNEL20-G007 | `deferred` | Downstream wildcard-arm checks for public enums must be enforced by downstream consumers after task 19. | Kernel enum inventory is documented and lint-guarded; downstream checks remain outside this crate. |
| KERNEL20-G008 | `source_undocumented_behavior` risk | Future public APIs or module exports could be added without audit updates. | `tests/lint_policy.rs` now fails unless this audit lists current public modules/items and module Trust Statement prohibitions. |
| KERNEL20-G009 | `repo_metadata_conflict` | None observed in task 20. | Report only if future metadata conflicts appear; do not auto-repair unrelated metadata. |
| KERNEL24-G001 | `source_drift` / `deferred` | Task 24 selects `batsat`, but no manifest/source change has occurred yet. `batsat` also lacks a public exact conflict/propagation budget setter. | Task 27 must add the exact dependency, update dependency lint guards, verify lockfile resolution, and either prove deterministic callback interruption or reject unsupported step-budget requests. |

## Verification Plan

Task 20 is an audit/lint task with no runtime behavior change. Required
verification is:

- `cargo test -p mizar-kernel source_spec_audit_covers_public_surface_and_prohibitions`;
- `cargo fmt --check`;
- `cargo test -p mizar-kernel`;
- `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`;
- `git diff --check`;
- `git diff --cached --check` after explicit path staging.

`cargo test -p mizar-core` and `cargo test -p mizar-checker` are not required
unless this audit changes binder contracts or checker/trace behavior; task 20
does neither.
