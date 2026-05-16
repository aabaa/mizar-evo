# Module: layout

> Canonical language: English. English canonical version: [../en/layout.md](../en/layout.md).

## 目的

この module は Mizar Evo tests の filesystem layout と metadata contract を定義する。

layout は large `.miz` corpora、fail-heavy regression testing、deterministic snapshots、expected outcomes の明確な ownership に最適化する。

## Public API

```rust
pub struct TestCase {
    pub id: TestCaseId,
    pub path: TestPath,
    pub kind: TestKind,
    pub domain: TestDomain,
    pub metadata: TestMetadata,
}

pub enum TestKind {
    Pass,
    Fail,
    Snapshot,
    FuzzSeed,
    PropertySeed,
    Generated,
}

pub struct TestMetadata {
    pub expected_phase: Option<PipelinePhase>,
    pub expected_failure: Option<FailureCategory>,
    pub expected_rejection: Option<RejectionReason>,
    pub expected_diagnostics: Vec<DiagnosticExpectation>,
    pub snapshot_profiles: Vec<SnapshotProfile>,
}
```

authoritative expectations は sidecar files に置く。fail、soundness、certificate、snapshot expectations は `.miz` frontend correctness に依存せず parse できなければならないため、sidecar を必須とする。inline metadata は parser が安全に無視できる non-authoritative tags に限って許可する。

## Directory Layout

required directories:

```text
tests/miz/pass/parser/
tests/miz/pass/types/
tests/miz/pass/attributes/
tests/miz/pass/clusters/
tests/miz/pass/theorems/

tests/miz/fail/parser/
tests/miz/fail/types/
tests/miz/fail/clusters/
tests/miz/fail/overload/
tests/miz/fail/substitution/
tests/miz/fail/soundness/

tests/generated/
tests/fuzz/
tests/property/

tests/certificates/
tests/snapshots/
```

追加 subdirectories は pass/fail/snapshot の区別を保つ場合にのみ追加できる。

## Naming Rules

test file names は stable snake_case names を使う。

```text
fail_soundness_false_arithmetic_001.miz
fail_substitution_capture_001.miz
pass_cluster_chain_basic_001.miz
snapshot_vc_simple_theorem_001.miz
```

rules:

- name は expected high-level outcome で始める
- name は semantic domain を含める
- numeric suffixes は stable であり unrelated cases に再利用しない
- minimized fuzz regressions は短い human-readable name と original seed metadata を保持する

## Expected Result Files

source test の例:

```text
tests/miz/fail/substitution/fail_substitution_capture_001.miz
tests/miz/fail/substitution/fail_substitution_capture_001.expect.toml
```

`expect.toml` は expected phase、failure category、rejection reason、diagnostic codes、snapshot profiles を記録する。

expected output は contract である。current compiler behavior から silent に regenerate してはならない。

## Algorithm / Logic

test discovery:

1. known test roots だけを walk する。
2. paths を canonical relative path で sort する。
3. `.miz` files と sidecar metadata を pair する。
4. fail、soundness、certificate、snapshot tests で metadata が missing の場合 reject する。
5. deterministic `TestPlan` を構築する。

## Tests

key scenarios:

- discovery order が filesystems をまたいで stable
- missing fail metadata は error
- duplicate test ids は reject
- generated / fuzz-minimized tests は discoverable だが origin で mark される
- unknown directories は explicit harness mode に従って ignore または reject

## Constraints and Assumptions

- test discovery は OS directory iteration order に依存してはならない。
- sidecar metadata schema は versioned である。
- pass tests は diagnostics なしを期待する場合にのみ expected diagnostics を省略できる。
- fail tests は expected failure category を明記しなければならない。
