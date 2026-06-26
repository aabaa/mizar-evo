# Source/Spec Audit: mizar-kernel

> 正本は英語です。英語版:
> [../en/source_spec_audit.md](../en/source_spec_audit.md)。

## Scope And Method

Task 20 は、実装済み `mizar-kernel` の public surface を paired module
specification と trusted-kernel prohibition boundary に照らして監査する。この
audit は source-derived かつ deterministic である。crate root の public module
export と、各 module の externally public な top-level `pub` item を下に列挙する。
Public field、inherent method、enum variant、trait impl、private helper は owning
public type と module spec によって cover される。機械的 guard では個別列挙しない。

この audit は意図的に behavior change ではない。現在の source/spec
correspondence を記録し、残る gap を分類し、将来の public API 追加時にこの audit
と module trust statement の更新を要求する lint guard を追加する。SAT solving、
ATP backend invocation、proof search、premise selection、overload resolution、
cluster search、implicit coercion insertion、fallback inference、source loading、
cache lookup、artifact lookup、mutable compiler-global state access は追加しない。

## Crate Module Exports

`src/lib.rs` は、仕様に裏付けられた次の module だけを expose する:

- `certificate_parser` -> source `src/certificate_parser.rs`, spec
  [certificate_parser.md](./certificate_parser.md).
- `checker` -> source `src/checker.rs`, spec [checker.md](./checker.md).
- `clause` -> source `src/clause.rs`, spec [clause.md](./clause.md).
- `formula_evidence` -> source `src/formula_evidence.rs`, spec
  [formula_evidence.md](./formula_evidence.md).
- `rejection` -> source `src/rejection.rs`, spec [rejection.md](./rejection.md).
- `resolution_trace` -> source `src/resolution_trace.rs`, spec
  [resolution_trace.md](./resolution_trace.md).
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

対応 summary:

- `CertificateParseContext`、`CertificateParseLimits`、`ClauseValidationPolicy`
  は parser resource / validation control を実装する。
- `KernelProfileRecord`、manifest、reference、generated clause、substitution
  entry、resolution step、derived fact、final-goal reference、section tag はこの
  module が所有する normalized certificate schema である。
- `Fingerprint`、`CertificateHashInputAlgorithm`、`CertificateParseError`、
  `FailureCategory`、`CertificateRejectionDetail`、`CertificateParseLocation`
  は semantic trust を与えず deterministic hash-input と parser rejection record
  を実装する。
- `parse_certificate` は byte parsing と structural validation だけを行う。

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

対応 summary:

- Imported-fact context、policy、status、evidence、report、result、
  `check_imported_facts` は immutable imported-fact validation boundary を実装する。
- `KernelCheckInput`、`KernelCheckPolicy`、`KernelCheckLimits`、
  `KernelCheckResult`、`KernelCheckStatus`、checked output record、service result
  alias、`check_kernel_certificate`、`check_kernel_batch` は policy-independent な
  phase-14 orchestration と deterministic batch ordering を実装する。
- Cluster/reduction context、evidence、checked-reference、report、result、
  `replay_cluster_trace` は explicit trace だけを replay する。cluster / reduction
  search は行わない。

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

対応 summary:

- Profile、validation context、tautology policy、clause form、literal、atom、
  term、symbol、variable、symbol kind は canonical clause representation と
  deterministic rendering を実装する。
- `ClauseError` はこの module が所有する structural well-formedness と resource
  failure を cover する。

### `formula_evidence`

Source: `src/formula_evidence.rs`. Spec: [formula_evidence.md](./formula_evidence.md).

Covered top-level public items:

- `SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID`
- `FormulaEvidenceParseContext`
- `FormulaEvidenceParseLimits`
- `ParsedKernelEvidence`
- `FormulaEvidenceEntry`
- `FormulaSourceClass`
- `FormulaSource`
- `ImportedFormulaSource`
- `Formula`
- `FormulaSubstitutionEvidence`
- `FormulaProvenance`
- `FinalGoalEvidence`
- `GoalPolarity`
- `FormulaEvidenceError`
- `FormulaEvidenceCheckResult`
- `parse_formula_evidence`

対応 summary:

- Parse context、limit、parsed evidence、formula/provenance/final-goal record、
  `parse_formula_evidence` は task-25 deterministic evidence envelope と structural
  parser を実装する。
- `Formula`、source binding record、substitution evidence、formula fingerprint、
  entry hash input は、instantiated formula や SAT clause を trusted payload として
  受理せず formula/substitution evidence identity を実装する。
- Rejection mapping は envelope / byte-shape certificate rejection と
  provenance / target-binding kernel rejection を分離し、proof search や SAT solving
  を行わない。

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

対応 summary:

- Target fingerprint、category、detail、clause reference、location、record は stable
  deterministic rejection output を実装する。
- `RejectionRecordError` は construction 時の category/detail と reference shape の
  mismatch を guard する。

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

対応 summary:

- Replay limit、trace input、imported clause context、report、checked step、result
  alias は deterministic MiniSAT-compatible resolution trace replay を実装する。
- `replay_resolution_trace` と `checked_resolution_final_goal` は explicit parent
  clause と final-goal binding を検査する。SAT solver や ATP backend は呼び出さない。

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

対応 summary:

- Encoding context、limit、SAT variable、literal、clause、atom-variable
  record、assertion record、encoded problem、result alias、
  `encode_formula_evidence` は、parsed formula/substitution evidence 上の
  task-26 formula instantiation と deterministic CNF/Tseitin encoding を実装する。
- この module は checked formula evidence から instantiated formula と SAT clause
  を導出する。Caller-supplied instantiated formula や SAT clause を trusted payload
  として受理せず、SAT solving や ATP/backend process invocation も行わない。
- より豊かな unsupported substitution shape は `invalid_substitution` として
  fail-closed で拒否され、`external_dependency_gap` / `deferred` のまま残る。

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

対応 summary:

- Replay limit、input、payload、replacement、witness、free-variable constraint、
  term path、context は explicit substitution、alpha/freshness、free-variable
  evidence replay を実装する。
- Report、checked substitution、result alias、replay helper は checked output だけを
  expose する。missing / malformed evidence は repair や inference ではなく拒否する。

## Trust Statement Audit

source-backed exported module specification はそれぞれ `## Trust Statement` section
を持ち、trusted-kernel statement と task-20 の完全な prohibition family を含むよう
guard される。Task 23 は SAT wording を修正する: proof search、ATP search or
backend invocation、premise selection、overload resolution、cluster search、implicit
coercion insertion、fallback inference、backend-reported success alone、source
loading、cache lookup、artifact lookup、wall-clock or random-state reads、unordered
iteration dependence、hidden reads of mutable compiler-global state は引き続き禁止される。
Trusted SAT checking は、validated formula/substitution evidence から kernel が導出した
SAT problem に対してだけ許可される。

Task 25 は `formula_evidence` を planned design surface から source-backed exported
module へ昇格させる。Task 26 は kernel-derived instantiation と deterministic SAT
problem construction のために `sat_encoding` を source-backed exported module へ
昇格させる。`sat_checker` は planned/unimplemented design surface のままであり、
task 27 が対応する exported module を追加するまでは executable source-backed guard
の対象には意図的に含めない。

## Closeout 後 correction addendum

Task 23 は source change より先に corrected design surface を追加する:

- `formula_evidence.md` は kernel-owned formula/substitution evidence schema と legacy
  unsupported handling を定義する;
- `sat_encoding.md` は kernel-derived deterministic SAT encoding を定義する;
- `sat_checker.md` は trusted in-process Rust SAT checker wrapper を定義する。

Task 24 は source change より先に dependency audit を追加する:

- `sat_dependency_audit.md` は task 24 による direct
  `batsat = { version = "=0.6.0", default-features = false }` の選択、却下した候補、
  unsafe-code audit、no-process/no-network audit、resource-limit gate、task 27 が
  符号化すべき dependency lint-policy revision を記録する。

現在の source inventory は task-26 public surface であり、formula/substitution evidence
parser と SAT encoder を追加する。一方で legacy `check_kernel_certificate` path は
corrected evidence format に対する `source_drift` / `design_drift` として残る。
Tasks 27-29 は、通常 proof policy が corrected pipeline に依存できるようになる前に、
SAT checker/service path を追加し legacy resolution-trace acceptance を gate または
retire しなければならない。

## Test Traceability

上の public surface は module-local Rust tests と cross-module lint guard によって
exercise される。Task 20 は source-derived `.miz` evidence fixture を作らない。
Task-23 correction 後の将来 corpus coverage は source-derived formula/substitution
evidence を target にしなければならない。legacy certificate-runner work は
migration-only であり deferred のままである。

| Module / boundary | Test path | Covered behavior |
|---|---|---|
| `certificate_parser` | `crates/mizar-kernel/src/certificate_parser/tests.rs` | Valid schema parsing、unsupported header/profile、directory と item canonicality、allocation 前の resource exhaustion、imported fact reference、manifest/generated-clause validation、substitution/resolution/derived/final reference、deterministic collection order、deterministic hash input、parser rejection classification。 |
| `checker` imported facts | `crates/mizar-kernel/src/checker/tests.rs` | Imported axiom/theorem context validation、namespace preservation、proof-status check、policy taint、fingerprint binding、duplicate context rejection、unused malformed entry handling、deterministic context/report ordering、count/resource limit。 |
| `checker` cluster/reduction replay | `crates/mizar-kernel/src/checker/tests.rs` | Valid trace replay、missing provenance、hidden/future dependency rejection、guard/result mismatch、bounded context construction、requested-step closure、unchecked base fact rejection、runtime limit、deterministic canonical order。 |
| `checker` service orchestration | `crates/mizar-kernel/src/checker/tests.rs` | Accepted service pipeline、substitution/report binding、generated-clause base set、final-goal / derived-fact fail-closed behavior、mutation fail corpus、deterministic repetition/permutation result、deterministic batch tie、replay-cost budget、timeout/resource propagation、target/input-order batch sorting。 |
| `clause` | `crates/mizar-kernel/src/clause/tests.rs` | Canonical literal/term ordering、duplicate literal removal、empty versus tautology form、tautology policy、malformed atom/term/symbol/variable rejection、profile/resource bound、canonical constructor check、stable rendering、display data を除外する hash input。 |
| `formula_evidence` | `crates/mizar-kernel/src/formula_evidence/tests.rs` | Valid evidence envelope parsing、standalone final-goal separation、stable formula rendering/hash input、explicit substitution evidence payload parsing、unknown schema/domain rejection、duplicate id、malformed formula rejection、missing provenance fail-closed behavior、imported statement fingerprint mismatch rejection、provenance target-binding mismatch rejection。 |
| `rejection` | `crates/mizar-kernel/src/rejection/tests.rs` | Stable key、category/detail ownership、parser conversion、checker location、owner mapping、deterministic ordering and tie-breaker、fixed-width target sort bytes、public enum compatibility。 |
| `resolution_trace` | `crates/mizar-kernel/src/resolution_trace/tests.rs` | Generated/imported/previous-step parent 上の valid replay、pivot / resolvent rejection、imported context sorting/provenance、first-use compatibility/depth check、resource limit、tautology policy、defensive invariant rejection、final-goal checkedness、deterministic report、deterministic rejection location、clause-owned depth/length helper。 |
| `sat_encoding` | `crates/mizar-kernel/src/sat_encoding/tests.rs` | Stable deterministic CNF/Tseitin encoding、canonical atom bytes による atom-variable ordering、standalone goal polarity、formula-wide substitution-derived assertion、recomputed derived formula fingerprint、binder-context canonicality and actual-term compatibility checks、unbound-only nested-binder substitution、alpha repair なしの capture fail-closed behavior、SAT checking 前の resource-limit rejection。 |
| `substitution_checker` | `crates/mizar-kernel/src/substitution_checker/tests.rs` | Direct substitution replay、payload role validation、missing/malformed/deferred evidence rejection、repair なしの target/manifest/capture check、alpha conversion、freshness witness、free-variable constraint、shuffled witness determinism、binder-context decoding、first-use side-condition rejection、resource limit、context canonicalization、report binding。 |
| Public-surface and trust lint | `crates/mizar-kernel/tests/lint_policy.rs` | Workspace/crate dependency boundary、source module exposure、public enum policy、forbidden producer/cache/artifact/nondeterminism tokens、exact source/spec audit inventory、task-22 private-test traceability and tracked-file guard、Trust Statement prohibition wording、gap classification marker、scanner regression cases。 |

## Gap Classification

| ID | Class | Evidence | Current action |
|---|---|---|---|
| KERNEL20-G001 | `external_dependency_gap` / `deferred` | Source-derived certificate and service envelopes are not produced by an active upstream crate or corpus runner. | Rust fixture coverage を維持し、missing evidence は拒否する。source-derived runner support は fabricate しない。 |
| KERNEL20-G002 | `external_dependency_gap` / `deferred` | Formula/substitution evidence candidate production is producer-owned and not available as a stable `mizar-atp` contract. ATP proof translation と MiniSAT-compatible backend trace extraction は legacy migration material であり trusted output ではない。 | Tasks 25-28 後に kernel は normalized formula/substitution evidence を check する。ATP backend invocation や trusted proof translation は追加しない。 |
| KERNEL20-G003 | `external_dependency_gap` / `deferred` | Cluster/reduction payload production by `mizar-checker` is not a ready integration contract. | Kernel は explicit cluster/reduction payload だけを replay する。cluster search や payload synthesis は追加しない。 |
| KERNEL20-G004 | `external_dependency_gap` / `deferred` | Derived-fact payload schema beyond current explicit checked inputs remains downstream/provenance-owned. | Derived fact は checked evidence で裏付けられない限り fail-closed のまま。 |
| KERNEL20-G005 | `external_dependency_gap` / `deferred` | Service-envelope normalization, cancellation token plumbing, and external worker scheduling are integration concerns outside the crate. | In-crate check は immutable input 上の deterministic synchronous check のまま。 |
| KERNEL20-G006 | `external_dependency_gap` / `deferred` | Downstream `mizar-proof`, `mizar-cache`, and `mizar-artifact` consumers are not ready proof-policy/cache/artifact contracts. | dependency や placeholder integration は追加しない。 |
| KERNEL20-G007 | `deferred` | Downstream wildcard-arm checks for public enums must be enforced by downstream consumers after task 19. | Kernel enum inventory は documented / lint-guarded。downstream check は crate 外に残る。 |
| KERNEL20-G008 | `source_undocumented_behavior` risk | Future public APIs or module exports could be added without audit updates. | `tests/lint_policy.rs` は、この audit が current public modules/items と module Trust Statement prohibitions を列挙しない限り fail する。 |
| KERNEL20-G009 | `repo_metadata_conflict` | None observed in task 20. | 将来 metadata conflict が見つかった場合だけ報告する。unrelated metadata は auto-repair しない。 |
| KERNEL24-G001 | `source_drift` / `deferred` | Task 24 は `batsat` を選択するが、manifest/source change はまだない。`batsat` は public exact conflict/propagation budget setter も持たない。 | Task 27 は exact dependency を追加し、dependency lint guard を更新し、lockfile resolution を検証し、deterministic callback interruption を証明するか unsupported step-budget request を拒否しなければならない。 |
| KERNEL25-G001 | `deferred` | Task 25 は formula/substitution evidence を parse し structural validation するが、formula instantiation、SAT encoding、SAT checker 呼び出し、legacy service acceptance path の置換は行わない。 | Tasks 26-28 は instantiated formula を導出し、deterministic SAT problem を構築し、trusted SAT checker を実行し、backend method や legacy resolution trace を trusted material として扱わず service acceptance path を wire しなければならない。 |
| KERNEL26-G001 | `deferred` | Task 26 は instantiated formula と deterministic SAT problem を導出するが、trusted SAT checker の呼び出しや legacy service acceptance path の置換は行わない。より豊かな formula-path / alpha-renaming substitution evidence も producer-owned stable schema ではない。 | Tasks 27-28 は trusted SAT checker wrapper と service acceptance path を追加しなければならない。より豊かな substitution producer は、それらの shape を受理できるようになる前に formula/substitution evidence schema を拡張しなければならない。 |

## Verification Plan

Task 20 は audit/lint task であり runtime behavior change はない。必要な verification:

- `cargo test -p mizar-kernel source_spec_audit_covers_public_surface_and_prohibitions`;
- `cargo fmt --check`;
- `cargo test -p mizar-kernel`;
- `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`;
- `git diff --check`;
- explicit path staging 後の `git diff --cached --check`。

この audit は binder contract や checker/trace behavior を変更しないため、
`cargo test -p mizar-core` と `cargo test -p mizar-checker` は不要である。
