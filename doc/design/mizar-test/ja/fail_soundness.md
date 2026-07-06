# Module: fail_soundness

> Canonical language: English. English canonical version: [../en/fail_soundness.md](../en/fail_soundness.md).

## 目的

この module は fail and soundness regression contracts を定義する。

優先事項は、pass してはならないものを reject することである。soundness failure は valid programs の受理漏れより重大である。

## Required Soundness Cases

次の theorem は決して pass してはならない。

```mizar
theorem
  1 = 0;
```

test metadata は stable detail key
`soundness.false_arithmetic.one_eq_zero` を使い、invalid proof attempt を最初に
検出する phase に応じて、proof failure、certificate rejection、または kernel
rejection を期待しなければならない。この case は既定の `fast` profile に残す。

## Required Domains

| Domain | Required Cases |
|---|---|
| substitution | variable capture、binder collision、malformed substitution、alpha-conversion failure |
| certificate | malformed certificate、invalid substitution、legacy invalid SAT proof、invalid SAT refutation、context mismatch、missing provenance、normal policy 下の unsupported legacy certificate、unresolved symbol、timeout、resource exhaustion |
| cluster | infinite chain、cyclic registration、unintended coercion、hidden transitive expansion |
| overload | ambiguous notation、hidden coercion、unstable resolution order。accepted coherent same-root refinement join を ambiguity failure として分類してはならない |
| dependency | stale theorem statement fingerprint、stale cluster semantics、stale notation parse result |
| policy | `require_kernel_certificates` 下で externally attested evidence が reject されること |

required fail/soundness case は domain の有無だけではなく、case 粒度で追跡する。
harness は次の stable failure identity を認識する。

| Stable detail key | Domain | Outcome | Failure categories | Rejection reasons | Stages | Expected phases |
|---|---|---|---|---|---|---|
| `soundness.false_arithmetic.one_eq_zero` | `soundness` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | phase-specific | `proof_verification`, `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.substitution.variable_capture` | `substitution` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.substitution.binder_collision` | `substitution` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.substitution.malformed_substitution` | `substitution` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.substitution.alpha_conversion_failure` | `substitution` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.certificate.malformed_certificate` | `certificate` | fail | `certificate_rejection` | `malformed_certificate` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.invalid_substitution` | `certificate` | fail | `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.invalid_sat_proof` | `certificate` | fail | `kernel_rejection` | `invalid_sat_proof` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.invalid_sat_refutation` | `certificate` | fail | `kernel_rejection` | `invalid_sat_refutation` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.context_mismatch` | `certificate` | fail | `certificate_rejection`, `kernel_rejection` | `context_mismatch` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.missing_provenance` | `certificate` | fail | `kernel_rejection` | `missing_provenance` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.unsupported_legacy_certificate` | `certificate` | fail | `certificate_rejection` | `unsupported_certificate_format` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.unresolved_symbol` | `certificate` | fail | `certificate_rejection`, `kernel_rejection` | `unresolved_symbol` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.timeout` | `certificate` | fail | `certificate_rejection`, `kernel_rejection` | `timeout` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.resource_exhaustion` | `certificate` | fail | `certificate_rejection`, `kernel_rejection` | `resource_exhaustion` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.cluster.infinite_chain` | `cluster` | fail | `cluster_error` | `cluster_loop` | `advanced_semantics` | `cluster_resolution` |
| `soundness.cluster.cyclic_registration` | `cluster` | fail | `cluster_error` | `cluster_loop` | `advanced_semantics` | `cluster_resolution` |
| `soundness.cluster.unintended_coercion` | `cluster` | fail | `cluster_error` | `unintended_coercion` | `advanced_semantics` | `cluster_resolution` |
| `soundness.cluster.hidden_transitive_expansion` | `cluster` | fail | `cluster_error` | `hidden_transitive_expansion` | `advanced_semantics` | `cluster_resolution` |
| `soundness.overload.ambiguous_notation` | `overload` | fail | `overload_error` | `ambiguous_notation` | `advanced_semantics` | `overload_resolution` |
| `soundness.overload.hidden_coercion` | `overload` | fail | `overload_error` | `hidden_coercion` | `advanced_semantics` | `overload_resolution` |
| `soundness.overload.unstable_resolution_order` | `overload` | fail | `overload_error` | `unstable_resolution_order` | `advanced_semantics` | `overload_resolution` |
| `soundness.dependency.stale_theorem_statement_fingerprint` | `dependency` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `stale_theorem_statement_fingerprint` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.dependency.stale_cluster_semantics` | `dependency` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `stale_cluster_semantics` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.dependency.stale_notation_parse_result` | `dependency` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `stale_notation_parse_result` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.policy.externally_attested_evidence_rejected` | `policy` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `externally_attested_evidence_rejected` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |

coherent same-root overload refinement case は false ambiguity diagnostic を防ぐための
required positive guard である。これは上の fail-case list には含めない。この pass
fixture の active 化は、overload assertion を実行できる advanced-semantics runner が
所有する。

## Expected Failure Contract

すべての fail/soundness test は次を宣言する。

```rust
pub struct FailExpectation {
    pub expected_phase: PipelinePhase,
    pub category: FailureCategory,
    pub rejection_reason: Option<RejectionReason>,
    pub diagnostic_codes: Vec<DiagnosticCode>,
    pub diagnostic_payloads: Vec<String>,
    pub stable_detail_key: String,
}
```

harness は diagnostic wording より先に category と rejection reason を check する。
diagnostic text は改善してよいが、failure identity は stable でなければならない。
fail expectation では `stable_detail_key` が常に必須である。
`diagnostic_codes` と `diagnostic_payloads` は、public diagnostic code または
machine-readable payload が存在する場合の順序付き assertion channel である。
certificate と kernel rejection は `rejection_reason` を含めなければならない。
通常の parser/resolver/type runner は、public diagnostic-code range がまだ意図的に
存在しない間、stable detail key を fallback identity として使ってよい。

## Algorithm / Logic

fail test execution:

1. requested profile で compiler を実行する。
2. structured failure records and diagnostics を collect する。
3. architecture-defined deterministic ordering で records を sort する。
4. expected phase、category、rejection reason、diagnostic code、stable detail key を match する。
5. compiler が success した、予期せず timeout した、弱い policy evidence しか emit しなかった、または異なる failure boundary を報告した場合 test を reject する。

metadata validation は、fail/soundness bookkeeping が active なときに上の case table を
強制する。Bookkeeping は trace manifest が fail/soundness requirement を含む場合、
または sidecar が認識済みの `soundness.*` stable detail key を使う場合に active になる。
metadata mode では missing required case を warning として報告し、development / release
mode では error として報告する。これは metadata gate に限られる。proof、certificate、
kernel、advanced-semantics runner が存在するまでは、harness はこれらの case に対する
active execution を捏造しない。

## Regression Rules

- fail/soundness test が pass し始めた場合、expectation が soundness proof と共に明示変更されない限り release blocker である。
- test は、同じ invalid input を sound に検出する場合にのみ earlier failure phase へ移動してよい。
- timeout は success ではない。
- resource exhaustion は success ではない。
- externally attested evidence は kernel verification と等価ではない。

## Tests

key scenarios:

- false arithmetic theorem fails
- malformed certificate は invalid SAT proof と別分類
- substitution capture は `invalid_substitution` で fail
- cluster cycle は `cluster_loop` で fail
- ambiguous ordinary overload は candidate iteration order に依存せず fail
- compatible joined facts を持つ coherent same-root redefinition は、ambiguous overload に誤分類されず pass

## Constraints and Assumptions

- fail expectations は explicit and versioned である。
- harness は current compiler output から expected failure を推測しない。
- soundness cases は default test profile で実行する。
