# Module: layout

> Canonical language: English. English canonical version: [../en/layout.md](../en/layout.md).

## 目的

この module は Mizar Evo tests の filesystem layout と metadata contract を定義する。

layout は large `.miz` corpora、fail-heavy regression testing、deterministic snapshots、expected outcomes の明確な ownership に向けて最適化されている。

## Public API

```rust
pub struct TestCase {
    pub id: TestCaseId,
    pub path: TestPath,
    pub kind: TestKind,
    pub domain: TestDomain,
    pub metadata: TestMetadata,
}

#[non_exhaustive]
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

`TestKind` はこの layout API に現れる expectation-owned corpus role enum である。
[expectation_schema.md](./expectation_schema.md) の public enum policy に従い、
downstream caller 向けに `#[non_exhaustive]` のままとする。

## Directory Layout

Committed corpus roots:

```text
tests/lexical/pass/
tests/lexical/fail/

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
tests/stress/

tests/certificates/
tests/snapshots/
```

Roots は first committed test まで存在しなくてもよい。Root が executable payloads を含む場合、それらの payloads は同じ sidecar and deterministic discovery rules に従う。

追加 subdirectories は pass/fail/snapshot の区別を保つ場合にのみ追加できる。

### Executable Payloads

Executable payloads は test inputs を表す files である。

| Extension | Meaning |
|---|---|
| `.miz` | Mizar source input. |
| `.src` | Lexical or parser source snippet input. |
| `.cert.json` | Certificate payload input. |
| `.fixture.toml` | Structured non-source fixture input. |

Committed corpus の every executable payload は、同じ stem の adjacent `.expect.toml` を持たなければならない。`README.md`、`.gitkeep`、snapshot output files は executable payloads ではない。

### Certificate Test Layout

certificate tests は、多くの certificate failure が `.miz` source file を必要としないため、専用の pass/fail split を使う。

```text
tests/certificates/pass/
tests/certificates/fail/malformed/
tests/certificates/fail/substitution/
tests/certificates/fail/sat/
tests/certificates/fail/symbols/
tests/certificates/fail/resources/
```

certificate payloads は、後続 schema が compact binary format を定義しない限り `.cert.json` を使う。すべての certificate test は隣接する `.expect.toml` を持つ。

```text
tests/certificates/fail/sat/fail_certificate_sat_satisfiable_refutation_001.cert.json
tests/certificates/fail/sat/fail_certificate_sat_satisfiable_refutation_001.expect.toml
```

expectation は expected `certificate_rejection` または `kernel_rejection` category と、`invalid_sat_proof`、`invalid_sat_refutation`、`invalid_substitution`、`malformed_certificate`、`context_mismatch`、`missing_provenance`、`unsupported_certificate_format`、`unresolved_symbol`、`timeout`、`resource_exhaustion` のような stable rejection reason を記録する。

## Naming Rules

test file names は stable snake_case names を使う。

```text
fail_soundness_false_arithmetic_001.miz
fail_substitution_capture_001.miz
pass_cluster_chain_basic_001.miz
snapshot_vc_simple_theorem_001.miz
```

rules:

- executable pass/fail/snapshot names は pass/fail/snapshot split 配下にある場合、expected high-level outcome で始める
- name は semantic domain を含める
- numeric suffixes は stable であり unrelated cases に再利用しない
- minimized fuzz regressions は短い human-readable name と original seed metadata を保持する
- oversized generated `.miz` files は default fast corpus ではなく `tests/stress/` と `stress` profile を使う

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
3. executable payload files と sidecar metadata を pair する。
4. every committed executable payload で metadata が missing の場合 reject する。
5. deterministic `TestPlan` を構築する。

## Tests

key scenarios:

- discovery order が filesystems をまたいで stable
- missing executable payload metadata は error
- duplicate test ids は reject
- generated / fuzz-minimized tests は discoverable だが origin で mark される
- unknown directories は explicit harness mode に従って ignore または reject

## Constraints and Assumptions

- test discovery は OS directory iteration order に依存してはならない。
- sidecar metadata schema は versioned である。
- pass tests は diagnostics なしを期待する場合 `diagnostic_codes = []` を記録する。
- fail tests は expected failure category and stable detail key を明記しなければならない。
