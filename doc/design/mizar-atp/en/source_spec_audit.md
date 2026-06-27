# Source/Spec Correspondence Audit: mizar-atp

> Canonical language: English. Japanese companion:
> [../ja/source_spec_audit.md](../ja/source_spec_audit.md).

Task 23 audits the current `mizar-atp` public surface and promised behavior
after the public-enum policy gate. It changes no source behavior, public API,
`.miz` fixture, expectation, language specification, backend route, kernel
check, proof policy, artifact witness, or cache behavior. Remaining unavailable
behavior is recorded as explicit `external_dependency_gap` or `deferred` work
instead of being fabricated or treated as normative.

## Scope And Method

The inventory covers the public modules exported by
`crates/mizar-atp/src/lib.rs`, top-level public items in each module, public
entry functions, and behavior promised by the paired module specifications.
Public fields and inherent methods are covered by the owning public type and by
the existing lint-policy public-surface allowlists; they are not repeated as a
field-by-field table here.

Module specifications audited:

- [problem.md](./problem.md)
- [translator.md](./translator.md)
- [property_encoding.md](./property_encoding.md)
- [tptp_encoder.md](./tptp_encoder.md)
- [smtlib_encoder.md](./smtlib_encoder.md)
- [backend.md](./backend.md)
- [portfolio.md](./portfolio.md)
- [todo.md](./todo.md)

Result: No source/spec drift, unclassified `source_drift`,
`design_drift`, `source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict` is observed for the current
candidate-evidence producer implementation. Remaining gaps are the classified
deferred or external follow-ups listed below.

## Crate Module Exports

`src/lib.rs` exports exactly these mizar-atp-owned modules:

- `backend`
- `portfolio`
- `problem`
- `property_encoding`
- `smtlib_encoder`
- `tptp_encoder`
- `translator`

The corresponding source paths are:

- `crates/mizar-atp/src/backend.rs`
- `crates/mizar-atp/src/portfolio.rs`
- `crates/mizar-atp/src/problem.rs`
- `crates/mizar-atp/src/property_encoding.rs`
- `crates/mizar-atp/src/smtlib_encoder.rs`
- `crates/mizar-atp/src/tptp_encoder.rs`
- `crates/mizar-atp/src/translator.rs`

Evidence: `crates/mizar-atp/tests/lint_policy.rs` checks this list through
`atp_lib_exposes_only_spec_backed_modules`, requires matching EN/JA module
specs, and keeps crate files limited to current spec-backed sources.

## Public Surface Inventory

### `problem`

Source path: `crates/mizar-atp/src/problem.rs`. Spec: [problem.md](./problem.md).

Generated public newtypes and string keys:

- `AtpDeclarationId`, `AtpFormulaId`, `AtpPropertyId`, `AtpProvenanceId`,
  `AtpTypeGuardId`
- `AtpSymbolName`, `AtpSourceBinding`, `AtpDiagnosticKey`,
  `AtpDiagnosticMessage`, `AtpProfileName`, `AtpPayload`,
  `AtpRequiredProofStatus`

Literal top-level public items:

- `AtpProblemId`, `AtpFingerprint`, `AtpTargetBinding`, `LogicProfile`,
  `LogicFragment`, `EqualitySupport`, `QuantifierPolicy`, `SoftTypeStrategy`,
  `NativePropertySupport`, `ConcreteFormat`, `ExpectedBackendResult`,
  `AtpProblemParts`, `AtpProblem`, `AtpDeclaration`, `AtpDeclarationKind`,
  `AtpFormula`, `AtpFormulaTree`, `AtpAtom`, `AtpTerm`, `AtpBinder`,
  `AtpTypeContext`, `AtpTypeGuard`, `EncodedProperty`, `PropertyEncoding`,
  `AtpSymbolMapEntry`, `AtpSymbolSource`, `AtpProvenance`, `AtpSourceRef`,
  `AtpDiagnostic`, `AtpProblemError`

Public entry functions: none beyond constructors/accessors on the public data
types.

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Backend-neutral `AtpProblem` records declarations, axioms, conjecture, target binding, symbol map, provenance, properties, diagnostics, and `Unsat` polarity without embedding backend proofs or SAT clauses. | `AtpProblemParts`, `AtpProblem::try_new`, canonical identity/debug helpers. | problem module tests for construction, deterministic identity, debug rendering, expected-result restriction, prohibited trusted material, and validation rejection. | Implemented for explicit ATP problem payloads. |
| Logic profiles and formula trees preserve unsupported features as fail-closed profile errors rather than silently re-profiling. | `LogicProfile`, profile validation, formula/type-context validation helpers. | unsupported-profile, symbol-reference, missing-payload, and source/provenance rejection tests. | Implemented. |
| Public enums are downstream forward-compatible. | `#[non_exhaustive]` on public enums. | `atp_public_enums_are_non_exhaustive_and_documented`. | Guarded by task 22. |

### `translator`

Source path: `crates/mizar-atp/src/translator.rs`. Spec: [translator.md](./translator.md).

Literal top-level public items:

- `AtpProjectionKey`, `AtpProjectionProvenance`,
  `AtpDeclarationProjection`, `AtpSymbolSourceProjection`,
  `AtpSoftTypeProjection`, `AtpSoftTypeRepresentation`,
  `AtpFormulaProjectionTarget`, `AtpFormulaProjection`,
  `AtpDeclarationTranslationInput`, `AtpTranslationInput`,
  `AtpDeclarationTranslation`, `AtpTranslationError`

Public entry functions:

- `translate_declarations`
- `translate_problem`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Translation consumes explicit public `VcIr` / kernel-handoff projection payloads and builds declarations, type guards, symbol maps, provenance, axioms, and conjecture with deterministic ordering. | `translate_declarations`, `translate_problem`, projection validation and materialization helpers. | declaration ordering, formula materialization, imported formula, generated fact, and deterministic shuffle tests. | Implemented for explicit projection payloads. |
| Unsupported checker-owned/type-predicate premise families and malformed or duplicate projection inputs fail closed instead of being invented by ATP. | translator validation and error paths. | unsupported checker-owned/type-predicate, missing/malformed/duplicate projection, formula fingerprint mismatch, and profile gate tests. | Implemented; source-derived extraction remains external. |
| Proof hints and premise restrictions do not prune or prove obligations. | proof-hint handling in translation input validation. | proof-hint restriction tests. | Implemented. |
| Public enums are downstream forward-compatible. | `#[non_exhaustive]` on public enums. | `atp_public_enums_are_non_exhaustive_and_documented`. | Guarded by task 22. |

### `property_encoding`

Source path: `crates/mizar-atp/src/property_encoding.rs`. Spec:
[property_encoding.md](./property_encoding.md).

Literal top-level public items:

- `AtpPropertyFamily`, `AtpPropertyTargetKind`,
  `AtpPropertyEncodingStrategy`, `AtpPropertyBinderSort`,
  `AtpPropertyProjection`, `AtpPropertyEncodingInput`,
  `AtpPropertyEncodingBundle`, `AtpPropertyEncodingError`

Public entry functions:

- `encode_properties`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Property encoding emits specified axiom-form properties, generated binder declarations, symbol-map rows, and provenance without backend-native shortcuts. | `encode_properties`, property projection validators, generated binder builders. | every supported family, generated binder, target-kind/arity, duplicate, and integration tests. | Implemented for axiom-form projection. |
| Unsupported native declaration requests and unsupported profile features stay deferred/fail-closed. | native-declaration and profile validation branches. | native-declaration deferral and unsupported profile tests. | Implemented as rejection/deferred behavior. |
| Public enums are downstream forward-compatible. | `#[non_exhaustive]` on public enums. | `atp_public_enums_are_non_exhaustive_and_documented`. | Guarded by task 22. |

### `tptp_encoder`

Source path: `crates/mizar-atp/src/tptp_encoder.rs`. Spec:
[tptp_encoder.md](./tptp_encoder.md).

Literal top-level public items:

- `TptpDialect`, `TptpEncodingOutput`, `TptpSymbolBinding`,
  `TptpFormulaLabel`, `TptpFormulaItem`, `TptpEncodingError`

Public entry functions:

- `encode_tptp`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Deterministic FOF output is emitted with stable labels, symbol bindings, name mangling, and side metadata. | `encode_tptp`, renderer and metadata builders. | golden output, deterministic output, label/binding, and raw-name injection tests. | Implemented for FOF. |
| Unsupported typed/CNF/include/native/backend features fail closed or remain deferred. | profile and native-declaration checks. | unsupported profile/native, scope/sorted-binder, duplicate/illegal-name, and validation-failure tests. | Implemented for current fail-closed coverage; typed/native routes remain deferred. |
| Public enums are downstream forward-compatible. | `#[non_exhaustive]` on public enums. | `atp_public_enums_are_non_exhaustive_and_documented`. | Guarded by task 22. |

### `smtlib_encoder`

Source path: `crates/mizar-atp/src/smtlib_encoder.rs`. Spec:
[smtlib_encoder.md](./smtlib_encoder.md).

Literal top-level public items:

- `SmtLibDialect`, `SmtLibEncodingOutput`, `SmtLibSymbolBinding`,
  `SmtLibAssertionLabel`, `SmtLibAssertionItem`, `SmtLibEncodingError`

Public entry functions:

- `encode_smtlib`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Deterministic uninterpreted SMT-LIB output is emitted with fixed universe sort, stable assertion labels, symbol bindings, and `Unsat` polarity. | `encode_smtlib`, renderer and metadata builders. | golden output, logic selection, deterministic output, labels/bindings, nullary predicate, and raw-name injection tests. | Implemented for uninterpreted SMT-LIB. |
| Arithmetic theories, sorted signatures, solver options, proof/unsat-core commands, native declarations, and backend execution remain out of scope. | profile/native/diagnostic rejection checks. | unsupported profile/native, scope/sorted-binder, duplicate/illegal-symbol, unused-sort, and validation-failure tests. | Implemented for current fail-closed coverage; theory/native routes remain deferred. |
| Public enums are downstream forward-compatible. | `#[non_exhaustive]` on public enums. | `atp_public_enums_are_non_exhaustive_and_documented`. | Guarded by task 22. |

### `backend`

Source path: `crates/mizar-atp/src/backend.rs`. Spec: [backend.md](./backend.md).

Literal top-level public items:

- `BackendRunId`, `BackendProfileId`, `BackendKind`,
  `EncodedBackendProblem`, `EncodedBackendProblemParts`, `BackendProfile`,
  `BackendVersionProbe`, `BackendCommand`, `BackendEnvironmentPolicy`,
  `BackendWorkingDirectoryPolicy`, `BackendIoMode`,
  `BackendLimitRequirement`, `BackendResourceLimits`,
  `BackendCancellationToken`, `BackendRunInput`, `BackendRunResult`,
  `BackendRunMetadata`, `BackendRunStatus`, `BackendObservedResult`,
  `BackendTermination`, `BackendExitStatus`, `BackendStreamCapture`,
  `BackendVersionRecord`, `BackendCandidateEvidence`,
  `BackendCandidatePayload`, `BackendCounterexample`, `BackendObservation`,
  `BackendDiagnostic`, `BackendConfigError`

Public entry functions:

- `run_backend`
- `classify_backend_observation`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| The generic backend runner uses deterministic command fingerprints, explicit stdin/private-file input delivery, version probes, resource limits, timeout/cancellation/crash handling, stream capture, and private temp cleanup. | `run_backend`, `BackendRunInput`, `BackendRunResult`, command/resource/capture helpers. | command fingerprint, stdin/private-file, version probe, timeout/cancellation/crash, drain-safe capture, temp cleanup, and unsupported-limit tests. | Implemented for generic direct-spawn and mock backend routes. |
| `Proved` is candidate-evidence-only and requires matching `Unsat` plus formula/substitution candidate payload; backend proof method, resolution trace, backend log, unsat core, and SMT proof object are never trusted. | `classify_backend_observation`, `BackendCandidateEvidence`, `BackendCandidatePayload`, status validation. | mock proved, prohibited payload, counterexample/unknown/error, truncation, and metadata mismatch tests. | Implemented for mock classification; real extraction remains external. |
| Stable run metadata is reproducibility metadata and stays outside trusted acceptance and candidate hashes. | `BackendRunMetadata`, `BackendRunResult::metadata`. | task-19 metadata projection tests. | Implemented. |
| Public enums are downstream forward-compatible. | `#[non_exhaustive]` on public enums. | `atp_public_enums_are_non_exhaustive_and_documented`. | Guarded by task 22. |

### `portfolio`

Source path: `crates/mizar-atp/src/portfolio.rs`. Spec: [portfolio.md](./portfolio.md).

Literal top-level public items:

- `PortfolioId`, `PortfolioInputParts`, `PortfolioInput`, `PortfolioBudget`,
  `PortfolioPolicyConstraints`, `PortfolioPlan`, `PortfolioEvidenceSet`,
  `PortfolioCandidateId`, `PortfolioCandidate`, `PortfolioCandidateKind`,
  `PortfolioEvidenceFormat`, `PortfolioStopSummary`, `PortfolioStopReason`,
  `PortfolioDiagnostic`, `PortfolioError`

Public entry functions:

- `plan_portfolio`
- `collect_portfolio_results`

| Spec promise | Source evidence | Test evidence | Status |
|---|---|---|---|
| Portfolio planning is policy-neutral and deterministic over prebuilt backend run inputs, budgets, and cancellation state. | `plan_portfolio`, `PortfolioInput`, `PortfolioPlan`, budget and diagnostic helpers. | no-schedulable/budget, shuffled planning, cancellation, and same-problem validation tests. | Implemented for prebuilt runs. |
| Candidate collection is deterministic, validates run/result metadata, rejects missing/unknown/duplicate results, and does not perform kernel checks or proof policy. | `collect_portfolio_results`, candidate hashing and evidence-set hashing helpers. | shuffled completion order, missing/unknown/duplicate, metadata mismatch, status-boundary, and determinism-suite tests. | Implemented for mock/prebuilt results. |
| Completion-order finality and early-stop winner selection require downstream proof policy and remain outside this crate. | stop-summary and no-early-stop design; absence of proof-policy hooks. | portfolio source prohibited-material guard and determinism suite. | Deferred/external. |
| Public enums are downstream forward-compatible. | `#[non_exhaustive]` on public enums. | `atp_public_enums_are_non_exhaustive_and_documented`. | Guarded by task 22. |

## Cross-Module Evidence

| Contract | Source/test correspondence |
|---|---|
| Crate scaffolding, dependency boundary, module exports, and spec-backed file set | `Cargo.toml`, `src/lib.rs`, and `tests/lint_policy.rs`; guarded by manifest, dependency, module-export, crate-file, paired-spec, and public API allowlist tests. |
| Formula/substitution candidate production stays untrusted until `mizar-kernel` checks it | `backend.rs` and `portfolio.rs` candidate records plus prohibited trusted-material lint tests; no kernel acceptance API is called. |
| Identical public `VcIr` inputs produce deterministic ATP problems, concrete encodings, mock backend classification, and portfolio candidate order | `crates/mizar-atp/tests/determinism_suite.rs`; `identical_vcir_inputs_produce_identical_problem_encodings_and_candidate_order`. |
| Metadata-only advanced-semantics corpus anchor does not fake active source-derived extraction | `crates/mizar-atp/tests/mock_backend_corpus.rs` and `tests/property/atp_mock_backend_integration_001.*`; active `.miz` runner remains deferred. |
| Public enum forward compatibility | Source attributes, EN/JA module inventories, and `atp_public_enums_are_non_exhaustive_and_documented`. |

## Remaining Classified Follow-Ups

| ID | Class | Evidence | Owner | Unblock condition | Target follow-up / downstream phase |
|---|---|---|---|---|---|
| ATP-AUDIT-G001 | `external_dependency_gap` | Tasks 15 and 16 record that no paired real-output extraction spec/source module maps concrete backend output to kernel-parseable formula/substitution candidate payloads, and supported backend executables were unavailable in verification. | `mizar-atp` with backend-specific specs. | Add EN/JA extraction specs, guarded real-backend fixture routes, and explicit candidate schema mapping that excludes backend proof material. | Reopen concrete backend route and polarity fixture tasks; do not add fake parsers or trusted backend proof objects. |
| ATP-AUDIT-G002 | `external_dependency_gap` | `mizar-artifact` already owns `ProofWitnessRef` schema version `2.0` and `VerifiedArtifact` witness-reference validation for formula/substitution kernel evidence, but real ATP producer output, proof-policy selection, proof-cache integration, and concrete witness publication are not connected; `mizar-proof` and `mizar-cache` are design-only in this workspace. | `mizar-proof`, `mizar-cache`, `mizar-artifact`, and integration owners. | Owner specs and workspace crates must connect proof-policy winner selection, real witness publication, cache promotion, and reuse metadata consumers to checked ATP/kernel evidence. | Keep proof policy, real artifact witness publication, and proof-cache promotion outside `mizar-atp`. |
| ATP-AUDIT-G003 | `deferred` | Task 20 uses a metadata-only `advanced_semantics` corpus fixture; no active `.miz` advanced-semantics runner or source-derived ATP extraction path exists in `mizar-test`. | `mizar-test` / source extraction owners. | Add active staged runner support and source-derived obligation-to-ATP extraction contracts. | Replace metadata-only coverage with active corpus coverage without changing ATP trust boundaries. |
| ATP-AUDIT-G004 | `deferred` | TPTP typed/CNF/include paths, SMT arithmetic/sorted signatures/solver options/proof commands, native declarations, and backend-native shortcuts are intentionally rejected or unimplemented by encoder specs and tests. | `mizar-atp` encoder owners. | Add paired EN/JA specs and focused tests for each concrete extension. | Extend concrete encoders only when the backend-neutral problem contract remains untrusted and fail-closed. |
| ATP-AUDIT-G005 | `external_dependency_gap` | Portfolio collection deliberately does not perform early-stop proof finality, winner selection, kernel checking, or proof policy. Task 25 depends on downstream policy authority. | `mizar-proof` / proof-policy owner with `mizar-atp` portfolio follow-up. | Downstream proof policy must report that pending candidates cannot displace the selected class. | Portfolio completion-order independence gate and any future early-stop integration. |

No `repo_metadata_conflict` was observed. No placeholder crate, placeholder
schema, resolution trace trusted path, backend proof method trusted path, SMT
proof object trusted path, trusted SAT problem payload, or fallback inference is
introduced by this audit.
