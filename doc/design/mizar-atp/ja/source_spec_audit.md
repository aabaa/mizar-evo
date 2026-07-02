# Source/Spec Correspondence Audit: mizar-atp

> 正本言語: 英語。英語正本:
> [../en/source_spec_audit.md](../en/source_spec_audit.md)。

Task 23 は public-enum policy gate 後の現在の `mizar-atp` public surface と
仕様が約束する挙動を監査する。Task 26 は task-25 Architecture-22 portfolio
completion-order contract について、この audit を再実行する。Task 27 は private test
module split について layout portion を再実行する。これらの audit は source behavior、
public API、`.miz` fixture、expectation、language specification、backend route、
kernel check、proof policy、artifact witness、cache behavior を変更しない。まだ利用
できない挙動は、捏造したり現在の実装都合を normative にしたりせず、明示的な
`external_dependency_gap` または `deferred` work として記録する。

## Scope And Method

この inventory は `crates/mizar-atp/src/lib.rs` が export する public module、各
module の top-level public item、public entry function、paired module specification が
約束する挙動を対象にする。public field と inherent method は、所有する public type と
既存の lint-policy public-surface allowlist が cover するため、ここでは field ごとの表を
繰り返さない。

監査した module specification:

- [problem.md](./problem.md)
- [translator.md](./translator.md)
- [property_encoding.md](./property_encoding.md)
- [tptp_encoder.md](./tptp_encoder.md)
- [smtlib_encoder.md](./smtlib_encoder.md)
- [backend.md](./backend.md)
- [portfolio.md](./portfolio.md)
- [todo.md](./todo.md)

結果: 現在の candidate-evidence producer implementation について、No source/spec drift
であり、未分類の `source_drift`、`design_drift`、`source_undocumented_behavior`、
`test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict` は観測されない。
残る gap は下に列挙する分類済み deferred / external follow-up である。

## Crate Module Exports

`src/lib.rs` は次の mizar-atp-owned module だけを export する:

- `backend`
- `portfolio`
- `problem`
- `property_encoding`
- `smtlib_encoder`
- `tptp_encoder`
- `translator`

対応する source path:

- `crates/mizar-atp/src/backend.rs`
- `crates/mizar-atp/src/portfolio.rs`
- `crates/mizar-atp/src/problem.rs`
- `crates/mizar-atp/src/property_encoding.rs`
- `crates/mizar-atp/src/smtlib_encoder.rs`
- `crates/mizar-atp/src/tptp_encoder.rs`
- `crates/mizar-atp/src/translator.rs`

task 27 後の対応する private test module path は次のとおり:

- `crates/mizar-atp/src/backend/tests.rs`
- `crates/mizar-atp/src/portfolio/tests.rs`
- `crates/mizar-atp/src/problem/tests.rs`
- `crates/mizar-atp/src/property_encoding/tests.rs`
- `crates/mizar-atp/src/smtlib_encoder/tests.rs`
- `crates/mizar-atp/src/tptp_encoder/tests.rs`
- `crates/mizar-atp/src/translator/tests.rs`

Evidence: `crates/mizar-atp/tests/lint_policy.rs` の
`atp_lib_exposes_only_spec_backed_modules` がこの list を確認し、対応する EN/JA module
spec を要求し、crate file を現在の spec-backed source に限定する。

## Public Surface Inventory

### `problem`

Source path: `crates/mizar-atp/src/problem.rs`。Spec: [problem.md](./problem.md)。

生成 public newtype と string key:

- `AtpDeclarationId`, `AtpFormulaId`, `AtpPropertyId`, `AtpProvenanceId`,
  `AtpTypeGuardId`
- `AtpSymbolName`, `AtpSourceBinding`, `AtpDiagnosticKey`,
  `AtpDiagnosticMessage`, `AtpProfileName`, `AtpPayload`,
  `AtpRequiredProofStatus`

literal top-level public item:

- `AtpProblemId`, `AtpFingerprint`, `AtpTargetBinding`, `LogicProfile`,
  `LogicFragment`, `EqualitySupport`, `QuantifierPolicy`, `SoftTypeStrategy`,
  `NativePropertySupport`, `ConcreteFormat`, `ExpectedBackendResult`,
  `AtpProblemParts`, `AtpProblem`, `AtpDeclaration`, `AtpDeclarationKind`,
  `AtpFormula`, `AtpFormulaTree`, `AtpAtom`, `AtpTerm`, `AtpBinder`,
  `AtpTypeContext`, `AtpTypeGuard`, `EncodedProperty`, `PropertyEncoding`,
  `AtpSymbolMapEntry`, `AtpSymbolSource`, `AtpProvenance`, `AtpSourceRef`,
  `AtpDiagnostic`, `AtpProblemError`

Public entry functions: public data type の constructor/accessor 以外はない。

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Backend-neutral `AtpProblem` は declaration、axiom、conjecture、target binding、symbol map、provenance、property、diagnostic、`Unsat` polarity を記録し、backend proof や SAT clause を埋め込まない。 | `AtpProblemParts`、`AtpProblem::try_new`、canonical identity/debug helper。 | construction、deterministic identity、debug rendering、expected-result restriction、prohibited trusted material、validation rejection の problem module tests。 | explicit ATP problem payload 向けに実装済み。 |
| Logic profile と formula tree は unsupported feature を silent re-profile せず、fail-closed profile error として保持する。 | `LogicProfile`、profile validation、formula/type-context validation helper。 | unsupported-profile、symbol-reference、missing-payload、source/provenance rejection tests。 | 実装済み。 |
| Public enum は downstream forward-compatible である。 | public enum の `#[non_exhaustive]`。 | `atp_public_enums_are_non_exhaustive_and_documented`。 | task 22 で guard 済み。 |

### `translator`

Source path: `crates/mizar-atp/src/translator.rs`。Spec: [translator.md](./translator.md)。

literal top-level public item:

- `AtpProjectionKey`, `AtpProjectionProvenance`,
  `AtpDeclarationProjection`, `AtpSymbolSourceProjection`,
  `AtpSoftTypeProjection`, `AtpSoftTypeRepresentation`,
  `AtpFormulaProjectionTarget`, `AtpFormulaProjection`,
  `AtpDeclarationTranslationInput`, `AtpTranslationInput`,
  `AtpDeclarationTranslation`, `AtpTranslationError`

Public entry functions:

- `translate_declarations`
- `translate_problem`

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Translation は explicit public `VcIr` / kernel-handoff projection payload を消費し、deterministic ordering で declaration、type guard、symbol map、provenance、axiom、conjecture を構築する。 | `translate_declarations`、`translate_problem`、projection validation / materialization helper。 | declaration ordering、formula materialization、imported formula、generated fact、deterministic shuffle tests。 | explicit projection payload 向けに実装済み。 |
| Unsupported checker-owned / type-predicate premise family と malformed / duplicate projection input は ATP が発明せず fail closed する。 | translator validation と error path。 | unsupported checker-owned/type-predicate、missing/malformed/duplicate projection、formula fingerprint mismatch、profile gate tests。 | 実装済み。source-derived extraction は external のまま。 |
| Proof hint と premise restriction は obligation を prune したり証明したりしない。 | translation input validation の proof-hint handling。 | proof-hint restriction tests。 | 実装済み。 |
| Public enum は downstream forward-compatible である。 | public enum の `#[non_exhaustive]`。 | `atp_public_enums_are_non_exhaustive_and_documented`。 | task 22 で guard 済み。 |

### `property_encoding`

Source path: `crates/mizar-atp/src/property_encoding.rs`。Spec:
[property_encoding.md](./property_encoding.md)。

literal top-level public item:

- `AtpPropertyFamily`, `AtpPropertyTargetKind`,
  `AtpPropertyEncodingStrategy`, `AtpPropertyBinderSort`,
  `AtpPropertyProjection`, `AtpPropertyEncodingInput`,
  `AtpPropertyEncodingBundle`, `AtpPropertyEncodingError`

Public entry functions:

- `encode_properties`

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Property encoding は指定済み axiom-form property、generated binder declaration、symbol-map row、provenance を backend-native shortcut なしで emit する。 | `encode_properties`、property projection validator、generated binder builder。 | every supported family、generated binder、target-kind/arity、duplicate、integration tests。 | axiom-form projection として実装済み。 |
| Unsupported native declaration request と unsupported profile feature は deferred/fail-closed のままにする。 | native-declaration と profile validation branch。 | native-declaration deferral と unsupported profile tests。 | rejection/deferred behavior として実装済み。 |
| Public enum は downstream forward-compatible である。 | public enum の `#[non_exhaustive]`。 | `atp_public_enums_are_non_exhaustive_and_documented`。 | task 22 で guard 済み。 |

### `tptp_encoder`

Source path: `crates/mizar-atp/src/tptp_encoder.rs`。Spec:
[tptp_encoder.md](./tptp_encoder.md)。

literal top-level public item:

- `TptpDialect`, `TptpEncodingOutput`, `TptpSymbolBinding`,
  `TptpFormulaLabel`, `TptpFormulaItem`, `TptpEncodingError`

Public entry functions:

- `encode_tptp`

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Deterministic FOF output は stable label、symbol binding、name mangling、side metadata とともに emit される。 | `encode_tptp`、renderer / metadata builder。 | golden output、deterministic output、label/binding、raw-name injection tests。 | FOF について実装済み。 |
| Unsupported typed / CNF / include / native / backend feature は fail closed または deferred のままにする。 | profile と native-declaration check。 | unsupported profile/native、scope/sorted-binder、duplicate/illegal-name、validation-failure tests。 | 現在の fail-closed coverage は実装済み。typed/native route は deferred のまま。 |
| Public enum は downstream forward-compatible である。 | public enum の `#[non_exhaustive]`。 | `atp_public_enums_are_non_exhaustive_and_documented`。 | task 22 で guard 済み。 |

### `smtlib_encoder`

Source path: `crates/mizar-atp/src/smtlib_encoder.rs`。Spec:
[smtlib_encoder.md](./smtlib_encoder.md)。

literal top-level public item:

- `SmtLibDialect`, `SmtLibEncodingOutput`, `SmtLibSymbolBinding`,
  `SmtLibAssertionLabel`, `SmtLibAssertionItem`, `SmtLibEncodingError`

Public entry functions:

- `encode_smtlib`

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Deterministic uninterpreted SMT-LIB output は fixed universe sort、stable assertion label、symbol binding、`Unsat` polarity とともに emit される。 | `encode_smtlib`、renderer / metadata builder。 | golden output、logic selection、deterministic output、labels/bindings、nullary predicate、raw-name injection tests。 | uninterpreted SMT-LIB について実装済み。 |
| Arithmetic theory、sorted signature、solver option、proof/unsat-core command、native declaration、backend execution は scope 外に残る。 | profile/native/diagnostic rejection check。 | unsupported profile/native、scope/sorted-binder、duplicate/illegal-symbol、unused-sort、validation-failure tests。 | 現在の fail-closed coverage は実装済み。theory/native route は deferred のまま。 |
| Public enum は downstream forward-compatible である。 | public enum の `#[non_exhaustive]`。 | `atp_public_enums_are_non_exhaustive_and_documented`。 | task 22 で guard 済み。 |

### `backend`

Source path: `crates/mizar-atp/src/backend.rs`。Spec: [backend.md](./backend.md)。

literal top-level public item:

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

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Generic backend runner は deterministic command fingerprint、explicit stdin/private-file input delivery、version probe、resource limit、timeout/cancellation/crash handling、stream capture、private temp cleanup を使う。 | `run_backend`、`BackendRunInput`、`BackendRunResult`、command/resource/capture helper。 | command fingerprint、stdin/private-file、version probe、timeout/cancellation/crash、drain-safe capture、temp cleanup、unsupported-limit tests。 | generic direct-spawn と mock backend route 向けに実装済み。 |
| `Proved` は candidate-evidence-only であり、matching `Unsat` と formula/substitution candidate payload を要求する。backend proof method、resolution trace、backend log、unsat core、SMT proof object は trusted ではない。 | `classify_backend_observation`、`BackendCandidateEvidence`、`BackendCandidatePayload`、status validation。 | mock proved、prohibited payload、counterexample/unknown/error、truncation、metadata mismatch tests。 | mock classification 向けに実装済み。real extraction は external のまま。 |
| Stable run metadata は reproducibility metadata であり、trusted acceptance と candidate hash の外に残る。 | `BackendRunMetadata`、`BackendRunResult::metadata`。 | task-19 metadata projection tests。 | 実装済み。 |
| Public enum は downstream forward-compatible である。 | public enum の `#[non_exhaustive]`。 | `atp_public_enums_are_non_exhaustive_and_documented`。 | task 22 で guard 済み。 |

### `portfolio`

Source path: `crates/mizar-atp/src/portfolio.rs`。Spec: [portfolio.md](./portfolio.md)。

literal top-level public item:

- `PortfolioId`, `PortfolioInputParts`, `PortfolioInput`, `PortfolioBudget`,
  `PortfolioPolicyConstraints`, `PortfolioPlan`, `PortfolioEvidenceSet`,
  `PortfolioCandidateId`, `PortfolioCandidate`, `PortfolioCandidateKind`,
  `PortfolioEvidenceFormat`, `PortfolioStopSummary`, `PortfolioStopReason`,
  `PortfolioDiagnostic`, `PortfolioError`

Public entry functions:

- `plan_portfolio`
- `collect_portfolio_results`

| spec promise | source evidence | test evidence | status |
|---|---|---|---|
| Portfolio planning は prebuilt backend run input、budget、cancellation state に対して policy-neutral かつ deterministic である。 | `plan_portfolio`、`PortfolioInput`、`PortfolioPlan`、budget / diagnostic helper。 | no-schedulable/budget、shuffled planning、cancellation、same-problem validation tests。 | prebuilt run 向けに実装済み。 |
| Candidate collection は deterministic で、run/result metadata を validate し、missing/unknown/duplicate result を reject し、kernel check や proof policy を行わない。 | `collect_portfolio_results`、candidate hash / evidence-set hash helper。 | shuffled completion order、missing/unknown/duplicate、metadata mismatch、status-boundary、determinism-suite tests。 | mock/prebuilt result 向けに実装済み。 |
| Completion-order finality と early-stop winner selection には downstream proof policy が必要であり、この crate の外に残る。 | stop-summary と no-early-stop design。proof-policy hook がないこと。 | portfolio source prohibited-material guard と determinism suite。 | deferred/external。 |
| Public enum は downstream forward-compatible である。 | public enum の `#[non_exhaustive]`。 | `atp_public_enums_are_non_exhaustive_and_documented`。 | task 22 で guard 済み。 |

## Cross-Module Evidence

| contract | source/test correspondence |
|---|---|
| Crate scaffolding、dependency boundary、module export、spec-backed file set | `Cargo.toml`、`src/lib.rs`、`tests/lint_policy.rs`; manifest、dependency、module-export、crate-file、paired-spec、public API allowlist tests が guard する。 |
| Formula/substitution candidate production は `mizar-kernel` が check するまで untrusted のままである | `backend.rs` と `portfolio.rs` の candidate record と prohibited trusted-material lint tests。kernel acceptance API は呼ばない。 |
| 同一 public `VcIr` input は deterministic ATP problem、concrete encoding、mock backend classification、portfolio candidate order を生成する | `crates/mizar-atp/tests/determinism_suite.rs`; `identical_vcir_inputs_produce_identical_problem_encodings_and_candidate_order`。 |
| Metadata-only advanced-semantics corpus anchor は active source-derived extraction を捏造しない | `crates/mizar-atp/tests/mock_backend_corpus.rs` と `tests/property/atp_mock_backend_integration_001.*`; active `.miz` runner は deferred のまま。 |
| Public enum forward compatibility | source attribute、EN/JA module inventory、`atp_public_enums_are_non_exhaustive_and_documented`。 |

## Task 26 Architecture-22 Follow-Up Audit

Task 26 は task-25 portfolio ordering / early-stop contract について source/spec audit
を再実行した。

Architecture 22 は、portfolio winner selection が active proof policy の下で deterministic
であることを要求する。"first backend to finish" は semantic winner rule になってはならず、
runtime duration は provenance として記録してよいが canonical proof identity に参加してはならない。

source/spec result:

- `src/portfolio.rs` は prebuilt backend run/result input に対する no-early-stop
  candidate handoff producer のままである。
- public source は proof-policy winner selector、early-stop oracle、kernel check
  result、accepted proof state、witness/cache writer、trusted backend proof material を
  まだ公開しない。
- task-18 と task-21 coverage は、no-early-stop path について shuffled backend
  completion order 下の deterministic candidate ordering を引き続き guard する。
- `atp_task_twenty_five_policy_gap_is_documented_and_guarded` と
  `atp_task_twenty_six_architecture_follow_up_audit_is_documented` は
  task-25/task-26 documentation marker と不変の gap classification を guard する。

Audit result: 新しい `source_drift`、`design_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、
`repo_metadata_conflict`、追加 `ATP-AUDIT-*` follow-up は見つからなかった。残る
completion-order / policy-boundary gap は ATP-AUDIT-G005 だけである。

## Task 27 Module-Boundary Layout Audit

Task 27 は module-boundary refactor gate について source/spec audit を再実行した。
この refactor は private test module split だけである:

- `src/backend.rs` は `cfg(all(test, unix))` の下で `src/backend/tests.rs` を指す。
- `src/portfolio.rs` は `cfg(test)` の下で `src/portfolio/tests.rs` を指す。
- `src/problem.rs` は `cfg(test)` の下で `src/problem/tests.rs` を指す。
- `src/property_encoding.rs` は `cfg(test)` の下で
  `src/property_encoding/tests.rs` を指す。
- `src/smtlib_encoder.rs` は `cfg(test)` の下で `src/smtlib_encoder/tests.rs` を指す。
- `src/tptp_encoder.rs` は `cfg(test)` の下で `src/tptp_encoder/tests.rs` を指す。
- `src/translator.rs` は `cfg(test)` の下で `src/translator/tests.rs` を指す。

source/spec result:

- public module export set と public API inventory は変更されていない。
- private test module は public API を定義せず、`src/lib.rs` から export されない。
- production code path、diagnostic、deterministic rendering、artifact-facing schema、
  candidate-evidence shape、kernel check、proof policy、witness/cache output、
  trusted backend material、trust-boundary behavior は変更されていない。
- `module_boundary_audit.md` は method と layout inventory を記録し、
  `atp_task_twenty_seven_module_boundary_layout_is_documented` は source tree と paired
  documentation marker を guard する。

Audit result: 新しい `source_drift`、`design_drift`、
`source_undocumented_behavior`、`test_expectation_drift`、`boundary_violation`、
`repo_metadata_conflict`、追加 `ATP-AUDIT-*` follow-up は見つからなかった。No new
ATP-AUDIT gap is required.

## Remaining Classified Follow-Ups

| ID | Class | Evidence | Owner | Unblock condition | Target follow-up / downstream phase |
|---|---|---|---|---|---|
| ATP-AUDIT-G001 | `external_dependency_gap` | task 15 と 16 は、concrete backend output を kernel-parseable formula/substitution candidate payload に map する paired real-output extraction spec/source module がなく、supported backend executable も verification で利用できなかったことを記録する。 | `mizar-atp` と backend-specific spec。 | EN/JA extraction spec、guarded real-backend fixture route、backend proof material を除外する explicit candidate schema mapping を追加する。 | concrete backend route と polarity fixture task を再開する。fake parser や trusted backend proof object は追加しない。 |
| ATP-AUDIT-G002 | `external_dependency_gap` | `mizar-artifact` は既に formula/substitution kernel evidence 向け `ProofWitnessRef` schema version `2.0` と `VerifiedArtifact` witness-reference validation を所有し、`mizar-proof` は proof-policy metadata を所有し、`mizar-cache` は現在 cache validation を所有するが、real ATP producer output、proof-policy selection integration、proof-cache integration、concrete witness publication は未接続である。 | `mizar-proof`、`mizar-cache`、`mizar-artifact`、integration owner。 | owner spec と workspace crate が、明示的な integration task を通じて proof-policy winner selection、real witness publication、cache promotion、reuse metadata consumer を checked ATP/kernel evidence に接続する。 | proof policy、real artifact witness publication、proof-cache promotion は `mizar-atp` の外に保つ。 |
| ATP-AUDIT-G003 | `deferred` | task 20 は metadata-only `advanced_semantics` corpus fixture を使う。active `.miz` advanced-semantics runner または source-derived ATP extraction path は `mizar-test` に存在しない。 | `mizar-test` / source extraction owner。 | active staged runner support と source-derived obligation-to-ATP extraction contract を追加する。 | ATP trust boundary を変えずに metadata-only coverage を active corpus coverage へ置き換える。 |
| ATP-AUDIT-G004 | `deferred` | TPTP typed/CNF/include path、SMT arithmetic/sorted signature/solver option/proof command、native declaration、backend-native shortcut は encoder spec と test により意図的に reject または未実装である。 | `mizar-atp` encoder owner。 | 各 concrete extension に paired EN/JA spec と focused test を追加する。 | backend-neutral problem contract を untrusted/fail-closed に保ったまま concrete encoder を拡張する。 |
| ATP-AUDIT-G005 | `external_dependency_gap` | task 25 は portfolio completion-order independence gate を再評価する。Portfolio collection は early-stop proof finality、release-policy winner selection、kernel checking、proof policy をまだ行わず、task-18/task-21 は shuffled completion order 下の no-early-stop deterministic handoff だけを cover する。 | `mizar-proof` / proof-policy owner と `mizar-atp` portfolio follow-up。 | 専用 ATP/proof integration task が `mizar-proof` の finality、winner selection、tie-breaking、candidate-displacement contract を消費する。 | その integration が存在した後にだけ portfolio completion-order independence gate を再開する。`mizar-atp` に mock early-stop oracle、placeholder proof-policy adapter、accepted state、kernel call、witness/cache output、trusted backend proof material を追加しない。 |

task-28 metadata correction 後に未解決の `repo_metadata_conflict` は残っていない。この audit
は placeholder crate、placeholder schema、trusted resolution trace path、trusted backend proof
method path、trusted SMT proof object path、trusted SAT problem payload、fallback inference を
導入しない。
