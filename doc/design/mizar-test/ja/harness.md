# Module: harness

> Canonical language: English. English canonical version: [../en/harness.md](../en/harness.md).

## 目的

この module は test cases を discover し、compiler profiles を run し、expectations を check し、deterministic results を report する test harness を定義する。

## Public API

```rust
pub struct TestPlan {
    pub cases: Vec<TestCase>,
    pub profile: TestProfile,
}

pub enum TestProfile {
    Fast,
    Full,
    Stress,
    FuzzRegression,
    SnapshotUpdate,
}

pub struct TestOutcome {
    pub case: TestCaseId,
    pub status: TestStatus,
    pub diagnostics: Vec<Diagnostic>,
    pub snapshots: Vec<SnapshotRecord>,
}
```

## Runner Modes

| Mode | Behavior |
|---|---|
| pass/fail | `.miz` cases を run し expected outcome と match |
| snapshot | canonical snapshot hashes を compare |
| determinism | repeated runs を比較し artifacts、diagnostics、hashes を check |
| parallel-equivalence | sequential and parallel outputs を compare |
| fuzz-regression | minimized fuzz cases を ordinary committed tests として run |
| update | 明示要求された場合のみ snapshots を rewrite |

## Algorithm / Logic

1. `layout` を通して tests を discover する。
2. canonical `TestPlan` を構築する。
3. execution が parallel でも deterministic display order で cases を run する。
4. compiler outputs を structured records として capture する。
5. snapshot expectations より先に pass/fail expectations を match する。
6. snapshots を canonical hash で compare する。
7. phase、failure category、rejection reason、diagnostic code、snapshot diff summary 付きで failures を report する。

## Determinism Requirements

harness は identical inputs が次を生成することを check する。

- identical artifact hashes
- identical snapshot hashes
- identical diagnostic order
- identical failure records
- identical proof status
- identical dependency slices

parallel execution は runtime を変えてよいが、observable results を変えてはならない。

## Reporting

reports は次を区別する。

- unexpected success
- unexpected failure
- wrong failure category
- wrong rejection reason
- diagnostic order mismatch
- snapshot mismatch
- repeated runs across nondeterminism
- harness infrastructure error

## Tests

key scenarios:

- fail test が unexpected pass する
- pass test が error diagnostic を emit する
- snapshot hash が異なる
- repeated run が異なる diagnostic order を生成する
- parallel run が sequential run と同じ artifacts を生成する

## Constraints and Assumptions

- test execution order は semantic ordering ではない。
- harness は cache hits を検証対象の compiler behavior として扱い、proof authority としては扱わない。
- snapshot update mode は opt-in であり command output に見える形でなければならない。
