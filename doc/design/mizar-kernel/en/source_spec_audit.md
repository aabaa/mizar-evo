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
- `formula_evidence` -> source `src/formula_evidence.rs`, spec
  [formula_evidence.md](./formula_evidence.md).
- `rejection` -> source `src/rejection.rs`, spec [rejection.md](./rejection.md).
- `resolution_trace` -> source `src/resolution_trace.rs`, spec
  [resolution_trace.md](./resolution_trace.md).
- `sat_checker` -> source `src/sat_checker.rs`, spec
  [sat_checker.md](./sat_checker.md).
- `sat_encoding` -> source `src/sat_encoding.rs`, spec
  [sat_encoding.md](./sat_encoding.md).
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
- `FormulaEvidenceContext`
- `FormulaImportedFactEvidence`
- `KERNEL_CONTEXT_IDENTITY_SCHEMA_VERSION`
- `KernelContextIdentityPayload`
- `KernelContextIdentityEntry`
- `KernelContextIdentitySource`
- `KernelFormulaProducerRef`
- `KernelVcGeneratedFormulaId`
- `ImportedFactNamespace`
- `AcceptedProofStatus`
- `ImportedFactCheckReport`
- `CheckedImportedFact`
- `ImportedFactCheckResult`
- `check_imported_facts`
- `KernelCheckInput`
- `KernelEvidenceCheckInput`
- `KernelCheckPolicy`
- `KernelCheckLimits`
- `KernelEvidenceCheckLimits`
- `KernelCheckResult`
- `KernelCheckStatus`
- `KernelEvidenceCheckKind`
- `CheckedDerivedFact`
- `CheckedFinalGoal`
- `UsedAxiom`
- `KernelCheckServiceResult`
- `check_kernel_evidence`
- `check_kernel_evidence_batch`
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
- `KernelEvidenceCheckInput`, `FormulaEvidenceContext`,
  `FormulaImportedFactEvidence`, `KernelContextIdentityPayload`,
  `KernelContextIdentityEntry`, `KernelContextIdentitySource`,
  `KernelFormulaProducerRef`, `KernelVcGeneratedFormulaId`,
  `KernelEvidenceCheckLimits`, `check_kernel_evidence`, and
  `check_kernel_evidence_batch` implement the task-28/task-30/task-31
  SAT-backed normal service path over parsed formula/substitution evidence,
  immutable imported formula/context-identity context, explicit caller check
  kind, deterministic SAT encoding, and the trusted SAT checker.
  `KernelEvidenceCheckKind` binds proof-obligation checks to
  `AssertFalseForRefutation` and consistency checks to
  `AssertTrueForConsistency`; mismatches reject as
  `certificate_rejection/context_mismatch` at `final_goal.polarity`.
  `KERNEL_CONTEXT_IDENTITY_SCHEMA_VERSION` and the context-identity payload
  types bind local-hypothesis, cited-premise, and generated-VC-fact formula
  entries to immutable task-28 context rows before SAT encoding.
- `KernelCheckInput`, `KernelCheckPolicy`, `KernelCheckLimits`,
  `KernelCheckResult`, `KernelCheckStatus`, checked output records, service
  result alias, `check_kernel_certificate`, and `check_kernel_batch` retain
  the legacy phase-14 orchestration and deterministic batch ordering inventory
  only behind the task-29 `allow_legacy_certificate_audit` gate. Default normal
  proof policy rejects this surface before replay.
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

### `formula_evidence`

Source: `src/formula_evidence.rs`. Spec: [formula_evidence.md](./formula_evidence.md).

Covered top-level public items:

- `SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID`
- `IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID`
- `canonical_imported_statement_projection_payload`
- `FormulaEvidenceParseContext`
- `FormulaEvidenceParseLimits`
- `ParsedKernelEvidence`
- `FormulaEvidenceEntry`
- `FormulaSourceClass`
- `FormulaSource`
- `ImportedStatementProjection`
- `ImportedFormulaSource`
- `Formula`
- `FormulaSubstitutionEvidence`
- `FormulaProvenance`
- `FinalGoalEvidence`
- `GoalPolarity`
- `FormulaEvidenceError`
- `FormulaEvidenceCheckResult`
- `parse_formula_evidence`

Correspondence summary:

- Parse context, limits, parsed evidence, formula/provenance/final-goal
  records, and `parse_formula_evidence` implement the task-25 deterministic
  evidence envelope and structural parser. Parsed evidence exposes read-only
  accessors so callers cannot mutate validated formula/provenance/target
  bindings before checker handoff.
- `ImportedStatementProjection`,
  `canonical_imported_statement_projection_payload`, source binding records,
  and architecture-18 / formula-tree fingerprint constants implement the
  task-33 imported-statement projection contract without source lookup or rich
  formula reconstruction.
- `Formula`, source binding records, substitution evidence, formula
  fingerprints, and entry hash inputs implement formula/substitution evidence
  identity without accepting instantiated formulas or SAT clauses as trusted
  payload.
- Rejection mapping separates envelope/byte-shape certificate rejections from
  provenance/target-binding kernel rejections and does not perform proof search
  or SAT solving.

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

### `sat_encoding`

Source: `src/sat_encoding.rs`. Spec: [sat_encoding.md](./sat_encoding.md).

Covered top-level public items:

- `SAT_PROBLEM_SCHEMA_VERSION`
- `SAT_PROBLEM_ENCODING_VERSION`
- `ASSERTION_KIND_PREMISE`
- `ASSERTION_KIND_SUBSTITUTION_INSTANCE`
- `ASSERTION_KIND_FINAL_GOAL`
- `SatEncodingContext`
- `SatEncodingLimits`
- `SatVariable`
- `SatLiteral`
- `SatClause`
- `SatAtomVariable`
- `EncodedFormulaAssertion`
- `EncodedSatProblem`
- `SatEncodingResult`
- `encode_formula_evidence`

Correspondence summary:

- Encoding context, limits, SAT variables, literals, clauses, atom-variable
  records, assertion records, encoded problem, result alias, and
  `encode_formula_evidence` implement task-26 formula instantiation and
  deterministic CNF/Tseitin encoding over parsed formula/substitution
  evidence.
- The module derives instantiated formulas and SAT clauses from checked
  formula evidence. It does not accept caller-supplied instantiated formulas
  or SAT clauses as trusted payload, does not solve SAT, and does not invoke
  ATP/backend processes.
- `EncodedSatProblem` exposes read-only accessors and keeps target binding,
  atom-variable manifest, assertions, clauses, and canonical bytes private
  outside the encoder, so callers cannot mutate the kernel-derived SAT
  material before checking.
- Unsupported richer substitution shapes reject fail-closed as
  `invalid_substitution` and remain `external_dependency_gap` / `deferred`.

### `sat_checker`

Source: `src/sat_checker.rs`. Spec: [sat_checker.md](./sat_checker.md).

Covered top-level public items:

- `SatCheckContext`
- `SatCheckLimits`
- `SatCheckReport`
- `SatCheckResult`
- `check_sat_problem`

Correspondence summary:

- SAT check context, deterministic input limits, checked report, result enum,
  and `check_sat_problem` implement the task-27 trusted wrapper over the exact
  audited in-process Rust SAT dependency.
- The module checks only `sat_encoding::EncodedSatProblem` values derived by
  the kernel. It accepts only dependency UNSAT as wrapper evidence, reports SAT
  as non-acceptance, maps unsupported step-budget requests to deterministic
  resource rejection, and maps dependency/internal inconsistencies to
  `invalid_sat_refutation`.
- The module does not expose `batsat` types, model/proof/DRAT/unsat-core
  material, DIMACS parsing/printing, solver command lines, heuristic knobs,
  callback surfaces, wall-clock timeouts, or backend proof methods. Task 28
  owns service acceptance wiring.

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

Task 25 promotes `formula_evidence` from planned design surface to
source-backed exported module. Task 26 promotes `sat_encoding` to a
source-backed exported module for kernel-derived instantiation and deterministic
SAT problem construction. Task 27 promotes `sat_checker` to a source-backed
exported module for trusted in-process Rust SAT checking over kernel-derived
SAT problems.

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

The current source inventory above is now the task-31 public surface: it adds
the formula/substitution evidence parser, SAT encoder, trusted SAT checker
wrapper, SAT-backed `check_kernel_evidence` service path, explicit
`allow_legacy_certificate_audit` gate for the task-22 legacy
`check_kernel_certificate` path, explicit proof-obligation/consistency check
kind binding, context-identity payload types for non-imported formula sources,
and imported-statement projection validation for corrected imported formula
sources. Default normal proof policy rejects legacy resolution-trace
certificates before replay; explicit audit mode remains migration-only,
returns rejected audit data after successful replay, and is not trusted
acceptance material. The task-31 context-identity payload is checked before SAT
encoding and binds local-hypothesis, cited-premise, and generated-VC-fact rows
to immutable task-28 source identity data. The task-33 imported-statement
projection keeps architecture-18 statement fingerprints distinct from kernel
formula-tree fingerprints, validates the canonical projection payload, and
requires the caller imported-fact context to carry the same projection before
SAT encoding.

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
| `checker` service orchestration | `crates/mizar-kernel/src/checker/tests.rs` | SAT-backed formula evidence acceptance/rejection, explicit check-kind/goal-polarity binding for both proof-obligation and consistency checks, F1-shaped polarity mismatch rejection before context/SAT work, task-31 context-identity acceptance/rejection for local/cited/generated formula sources, context-identity resource limits, task-28 golden line-grammar hashing, imported formula context proof-status and imported-statement projection checks, satisfiable-goal rejection, target mismatch rejection, deterministic evidence batch ties, normal-policy legacy certificate rejection, explicit legacy migration/audit service pipeline, substitution/report binding, generated-clause base sets, final-goal and derived-fact fail-closed behavior, mutation fail corpus, deterministic repetition/permutation results, replay-cost budgets, timeout/resource propagation, and target/input-order batch sorting. |
| `clause` | `crates/mizar-kernel/src/clause/tests.rs` | Canonical literal/term ordering, duplicate literal removal, empty versus tautology forms, tautology policy, malformed atom/term/symbol/variable rejection, profile/resource bounds, canonical constructor checks, stable rendering, and hash input exclusion of display data. |
| `formula_evidence` | `crates/mizar-kernel/src/formula_evidence/tests.rs` | Valid evidence envelope parsing, standalone final-goal separation, stable formula rendering/hash input, explicit substitution evidence payload parsing, unknown schema/domain rejection, duplicate ids, malformed formula rejection, missing provenance fail-closed behavior, imported-statement projection acceptance for distinct architecture-18 statement fingerprints, unsupported imported-statement/projection algorithm rejection, empty projection payload rejection, stale projection statement rejection, formula-projection mismatch rejection, noncanonical projection payload rejection, and provenance target-binding mismatch rejection. |
| `rejection` | `crates/mizar-kernel/src/rejection/tests.rs` | Stable keys, category/detail ownership, parser conversion, checker locations, owner mappings, deterministic ordering and tie-breakers, fixed-width target sort bytes, and public enum compatibility. |
| `resolution_trace` | `crates/mizar-kernel/src/resolution_trace/tests.rs` | Valid replay over generated/imported/previous-step parents, pivot and resolvent rejection, imported context sorting/provenance, first-use compatibility/depth checks, resource limits, tautology policy, defensive invariant rejection, final-goal checkedness, deterministic reports, deterministic rejection locations, and clause-owned depth/length helpers. |
| `sat_checker` | `crates/mizar-kernel/src/sat_checker/tests.rs` | Trusted wrapper outcomes for unsatisfiable and satisfiable kernel-derived SAT problems, deterministic repeated checks, input-limit rejection before solver construction, unsupported exact step-budget rejection without solver-hook accounting, invalid clause/literal shape rejection, and audited `batsat::SolverOpts` pinning. |
| `sat_encoding` | `crates/mizar-kernel/src/sat_encoding/tests.rs` | Stable deterministic CNF/Tseitin encoding, atom-variable ordering by canonical atom bytes, standalone goal polarity, formula-wide substitution-derived assertions, recomputed derived formula fingerprints, binder-context canonicality and actual-term compatibility checks, unbound-only nested-binder substitution, capture fail-closed behavior without alpha repair, and resource-limit rejection before SAT checking. |
| `substitution_checker` | `crates/mizar-kernel/src/substitution_checker/tests.rs` | Direct substitution replay, payload role validation, missing/malformed/deferred evidence rejection, target/manifest/capture checks without repair, alpha conversion, freshness witnesses, free-variable constraints, shuffled witness determinism, binder-context decoding, first-use side-condition rejection, resource limits, context canonicalization, and report binding. |
| Public-surface and trust lint | `crates/mizar-kernel/tests/lint_policy.rs` | Workspace/crate dependency boundary, source module exposure, public enum policy, forbidden producer/cache/artifact/nondeterminism tokens, exact source/spec audit inventory, read-only parsed formula evidence and SAT problem invariants, task-22 private-test traceability and tracked-file guard, Trust Statement prohibition wording, gap classification markers, and scanner regression cases. |

## Gap Classification

| ID | Class | Evidence | Current action |
|---|---|---|---|
| KERNEL20-G001 | `external_dependency_gap` / `deferred` | Source-derived certificate and service envelopes are not produced by an active upstream crate or corpus runner. | Keep Rust fixture coverage and reject missing evidence; do not fabricate source-derived runner support. |
| KERNEL20-G002 | `external_dependency_gap` / `deferred` | Formula/substitution evidence candidate production is producer-owned and not available as a stable `mizar-atp` contract. ATP proof translation and MiniSAT-compatible backend trace extraction are legacy migration material, not trusted output. | Kernel checks normalized formula/substitution evidence after tasks 25-28; no ATP backend invocation or trusted proof translation is added. |
| KERNEL20-G003 | `external_dependency_gap` / `deferred` | Cluster/reduction payload production by `mizar-checker` is not a ready integration contract. | Kernel replays explicit cluster/reduction payloads only; no cluster search or payload synthesis is added. |
| KERNEL20-G004 | `external_dependency_gap` / `deferred` | Derived-fact payload schema beyond current explicit checked inputs remains downstream/provenance-owned. | Derived facts remain fail-closed unless backed by checked evidence. |
| KERNEL20-G005 | `external_dependency_gap` / `deferred` | Service-envelope normalization, cancellation token plumbing, and external worker scheduling are integration concerns outside the crate. | In-crate checks remain deterministic and synchronous over immutable inputs. |
| KERNEL20-G006 | `external_dependency_gap` / `deferred` | Downstream `mizar-proof`, `mizar-cache`, and `mizar-artifact` consumers are not ready as full proof-policy/cache/artifact contracts. `mizar-proof` has limited accepted proof-obligation status, used-axiom, and witness-boundary consumers after tasks 30-31, but richer proof-policy projection and externally authenticated evidence policy remain downstream; `mizar-cache` and `mizar-artifact` still have no proof-cache/artifact consumer contracts. | No dependency or placeholder cache/artifact integration is added; proof-policy expansion remains owned by downstream `mizar-proof` tasks. |
| KERNEL20-G007 | `deferred` | Downstream wildcard-arm checks for public enums must be enforced by downstream consumers after task 19. | Kernel enum inventory is documented and lint-guarded; downstream checks remain outside this crate. |
| KERNEL20-G008 | `source_undocumented_behavior` risk | Future public APIs or module exports could be added without audit updates. | `tests/lint_policy.rs` now fails unless this audit lists current public modules/items and module Trust Statement prohibitions. |
| KERNEL20-G009 | `repo_metadata_conflict` | None observed in task 20. | Report only if future metadata conflicts appear; do not auto-repair unrelated metadata. |
| KERNEL24-G001 | `deferred` | `batsat` lacks a public exact conflict/propagation budget setter. | Task 27 rejects unsupported step-budget requests before solver construction; exact solver-step budgets remain deferred until a dependency exposes a stable deterministic API. |
| KERNEL25-G001 | `deferred` | Task 25 parses and structurally validates formula/substitution evidence but does not instantiate formulas, encode SAT, call the SAT checker, or replace the legacy service acceptance path. | Tasks 26-28 derive instantiated formulas, build deterministic SAT problems, run the trusted SAT checker, and wire the SAT-backed service path without treating backend methods or legacy resolution traces as trusted material. |
| KERNEL26-G001 | `deferred` | Task 26 derives instantiated formulas and deterministic SAT problems, task 27 adds the trusted SAT checker wrapper, and task 28 adds the SAT-backed `check_kernel_evidence` service path. Richer formula-path and alpha-renaming substitution evidence is still not yet a producer-owned stable schema. | Richer substitution producers must extend the formula/substitution evidence schema before those shapes can be accepted. |
| KERNEL29-G001 | `source_drift` / `design_drift` closed by task 29 | The legacy `check_kernel_certificate` surface remains present for task-22 migration/audit inventory. | Task 29 gates it behind `KernelCheckPolicy.allow_legacy_certificate_audit`; default normal proof policy rejects legacy resolution-trace certificates before replay, explicit audit replay still returns `Rejected` without trusted `final_goal` / `used_axioms`, and this migration-only surface is re-audited during quality re-review. |
| KERNEL31-G001 | F2 `source_drift` / `design_drift` closed by task 31 | Before task 31, non-imported local-hypothesis, cited-premise, and generated-VC-fact formula sources were well-shaped but not checked against immutable source context rows. | Task 31 requires a context-identity payload for those sources, checks target binding, recomputes the documented task-28 context-identity hash, matches each non-imported formula entry against immutable rows, and rejects missing/stale/ambiguous rows before SAT encoding. |

## Verification Plan

Task 31 refreshes this audit while adding context-identity verification to the
SAT-backed checker service. Required verification is:

- `cargo test -p mizar-kernel source_spec_audit_covers_public_surface_and_prohibitions`;
- focused `cargo test -p mizar-kernel context_identity --lib`;
- focused `cargo test -p mizar-kernel sat_backed_kernel_evidence --lib`;
- `cargo fmt --check`;
- `cargo test -p mizar-kernel`;
- `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`;
- `git diff --check`;
- `git diff --cached --check` after explicit path staging.

Because task 31 changes checker service behavior but not `mizar-core`,
`mizar-vc`, `mizar-artifact`, or `mizar-checker` source semantics, broad
workspace `cargo clippy --all-targets --all-features -- -D warnings` and
`cargo test` provide final boundary confidence when practical.
