# Bilingual Documentation Sync Audit: mizar-atp

> Canonical language: English. Japanese companion:
> [../ja/bilingual_sync_audit.md](../ja/bilingual_sync_audit.md).

Task 24 audits the `mizar-atp` design documentation pairs after the
source/spec correspondence audit. Task 25 updates this audit record for the
portfolio completion-order independence gate deferral. Task 26 re-runs the
sync audit for the Architecture-22 follow-up. These audit edits change no Rust
production source, public API, `.miz` fixture, expectation, language
specification, backend route, kernel check, proof policy, artifact witness,
cache behavior, or downstream integration.

## Scope And Method

The audit covers every current Markdown document under
`doc/design/mizar-atp/en/` and its companion under `doc/design/mizar-atp/ja/`.
For each pair, the review checked:

- same filename in both language directories;
- substantive meaning of module responsibility, inputs/outputs, behavior
  rules, candidate-evidence status, deterministic ordering, proof/trust
  boundary, planned tests, public enum inventory, audit inventory, TODO task
  wording, and follow-up classifications;
- preservation of known `external_dependency_gap` and `deferred` records
  rather than silently resolving, weakening, or broadening them;
- no new trusted acceptance, backend proof material, resolution trace,
  SMT proof object, caller-supplied SAT problem payload, proof policy, artifact
  witness publication, proof-cache promotion, or placeholder downstream
  integration.

The Japanese companion may use idiomatic translation and may keep Rust
identifiers, phase names, schema names, and task names in English. The
synchronization rule is semantic: the companion must not omit, weaken, or add
normative meaning relative to the English canonical document.

Result: all current document pairs exist and are semantically synchronized. No
bilingual drift, missing companion, stale status, unclassified
`design_drift`, or `repo_metadata_conflict` was observed. Remaining unavailable
behavior is the classified external/deferred work already recorded in
[source_spec_audit.md](./source_spec_audit.md).

## Pair Inventory

| Document | Synchronized content checked | Result |
|---|---|---|
| `00.crate_plan.md` | Crate responsibility, authority order, design/source inventory, known gaps, task decomposition through task 26, hard gates, and verification expectations. | Synchronized. |
| `problem.md` | Backend-neutral `AtpProblem` data shape, logic profiles, formula/provenance/type-guard ownership, deterministic identity, prohibited trusted material, planned tests, and public enum inventory. | Synchronized. |
| `translator.md` | Explicit `VcIr` / kernel-handoff projection inputs, declaration/formula materialization, fail-closed unsupported premise classes, proof-hint non-pruning, deterministic ordering, planned tests, and public enum inventory. | Synchronized. |
| `property_encoding.md` | Axiom-form property projection, generated binder rows, provenance and symbol-map requirements, native-declaration deferral, planned tests, and public enum inventory. | Synchronized. |
| `tptp_encoder.md` | Deterministic FOF emission, label and symbol metadata, name mangling, unsupported typed/native/backend routes, planned tests, and public enum inventory. | Synchronized. |
| `smtlib_encoder.md` | Deterministic uninterpreted SMT-LIB emission, fixed universe sort, assertion metadata, unsupported theory/native/backend routes, planned tests, and public enum inventory. | Synchronized. |
| `backend.md` | Generic backend runner, command fingerprints, resource limits, run metadata, candidate-evidence-only `Proved`, prohibited trusted backend material, failure semantics, and public enum inventory. | Synchronized. |
| `portfolio.md` | Policy-neutral planning, no-early-stop collection, candidate/evidence-set ordering, fail-closed result matching, downstream proof-policy boundary, determinism suite, task-25 deferred completion-order gate, and public enum inventory. | Synchronized. |
| `source_spec_audit.md` | Public module exports, public surface inventory, cross-module evidence, ATP-AUDIT gap register including task-25 G005, task-26 Architecture-22 follow-up result, `ProofWitnessRef` / `VerifiedArtifact` artifact-surface acknowledgement, and no source/spec drift classification. | Synchronized. |
| `bilingual_sync_audit.md` | Audit scope, method, pair inventory, classification, task-24/task-25/task-26 sync edits, and remaining external/deferred work. | Synchronized by this paired audit document. |
| `todo.md` | Ordered task list, completed tasks through task 26, deferred task 15/16 status, public enum task status, source/spec audit status, task-25 dependency-gap status, task-26 follow-up-audit status, next task 27 audit wording, and verification expectations. | Synchronized. |

## Classification

Task 24 and its task-26 re-run record no new `spec_gap`, `test_gap`, `design_drift`,
`source_drift`, `source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, `repo_metadata_conflict`, or bilingual drift. Existing
classified records remain:

- `external_dependency_gap`: no paired real-output extraction spec/source
  module maps concrete backend output to kernel-parseable formula/substitution
  candidate payloads, and no supported real backend route is available in the
  verification environment.
- `external_dependency_gap`: `mizar-artifact` already owns `ProofWitnessRef`
  schema version `2.0` and `VerifiedArtifact` witness-reference validation for
  formula/substitution kernel evidence, but real ATP producer output,
  proof-policy selection, proof-cache integration, and real artifact witness
  publication remain external. `mizar-proof` and `mizar-cache` are design-only
  in this workspace.
- `deferred`: active `.miz` advanced-semantics execution and source-derived ATP extraction
  remain outside the current metadata-only corpus fixture.
- `deferred`: TPTP typed/CNF/include paths, SMT arithmetic/sorted signatures,
  solver options, proof commands, native declarations, and backend-native
  shortcuts remain unavailable until paired specs and tests exist.
- `external_dependency_gap`: task 25 re-evaluates portfolio early-stop
  finality and winner selection. They depend on downstream proof policy;
  raw backend completion order remains outside proof identity, and no
  proof-policy oracle or placeholder `mizar-proof` adapter is introduced.

## Task 24 Sync Edits

This task adds the paired bilingual sync audit documents, marks task 24
complete in the paired TODO files, records the task-24 status in the paired
crate plans, and extends `crates/mizar-atp/tests/lint_policy.rs` with a
bilingual audit guard.

No other paired content required synchronization. The audit deliberately does
not add real backend adapters, backend-output parsers, kernel calls, proof
policy, witness writers, cache promotion, placeholder `mizar-proof` /
`mizar-cache` crates, or trusted backend proof material.

## Task 25 Sync Edits

Task 25 marks the portfolio completion-order independence gate complete only as
a deferred/external_dependency_gap re-evaluation. The paired TODO, crate plan,
portfolio spec, source/spec audit, and this bilingual audit now state that the
release-policy winner/early-stop gate requires a real `mizar-proof` policy
owner. The edits deliberately do not add a mock proof-policy oracle,
placeholder proof-policy adapter, accepted proof state, kernel call,
witness/cache output, or trusted backend proof material.

## Task 26 Sync Edits

Task 26 re-runs the bilingual sync audit for the Architecture-22 follow-up.
The paired TODO, crate plan, source/spec audit, and this bilingual audit now
state that Architecture 22 forbids backend completion order and runtime
duration from becoming semantic proof identity. The re-run found no bilingual
drift, stale task status, `repo_metadata_conflict`, or new follow-up gap.
ATP-AUDIT-G005 remains the single policy-boundary / completion-order follow-up
until a real `mizar-proof` policy owner exists.
