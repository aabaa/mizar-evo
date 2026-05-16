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

test metadata は invalid proof attempt を最初に検出する phase に応じて、proof failure、certificate rejection、または kernel rejection を期待しなければならない。

## Required Domains

| Domain | Required Cases |
|---|---|
| substitution | variable capture、binder collision、malformed substitution、alpha-conversion failure |
| certificate | malformed certificate、invalid substitution、invalid SAT proof、unresolved symbol、timeout、resource exhaustion |
| cluster | infinite chain、cyclic registration、unintended coercion、hidden transitive expansion |
| overload | ambiguous notation、hidden coercion、unstable resolution order |
| dependency | stale theorem statement fingerprint、stale cluster semantics、stale notation parse result |
| policy | `require_kernel_certificates` 下で externally attested evidence が reject されること |

## Expected Failure Contract

すべての fail/soundness test は次を宣言する。

```rust
pub struct FailExpectation {
    pub expected_phase: PipelinePhase,
    pub category: FailureCategory,
    pub rejection_reason: Option<RejectionReason>,
    pub diagnostic_codes: Vec<DiagnosticCode>,
    pub stable_detail_key: String,
}
```

harness は diagnostic wording より先に category と rejection reason を check する。diagnostic text は改善してよいが、failure identity は stable でなければならない。

## Algorithm / Logic

fail test execution:

1. requested profile で compiler を実行する。
2. structured failure records and diagnostics を collect する。
3. architecture-defined deterministic ordering で records を sort する。
4. expected phase、category、rejection reason、diagnostic code、stable detail key を match する。
5. compiler が success した、予期せず timeout した、弱い policy evidence しか emit しなかった、または異なる failure boundary を報告した場合 test を reject する。

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
- ambiguous overload は candidate iteration order に依存せず fail

## Constraints and Assumptions

- fail expectations は explicit and versioned である。
- harness は current compiler output から expected failure を推測しない。
- soundness cases は default test profile で実行する。
