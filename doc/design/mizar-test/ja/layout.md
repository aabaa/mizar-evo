# Module: layout

> Canonical language: English. English canonical version: [../en/layout.md](../en/layout.md).

## 目的

この module は Mizar Evo tests の filesystem discovery boundary と adjacent-sidecar layout contract を定義する。

layout は large `.miz` corpora、fail-heavy regression testing、deterministic snapshots、expected outcomes の明確な ownership に向けて最適化されている。

## Public API

```rust
pub struct DiscoveredLayout {
    pub payloads: Vec<PathBuf>,
    pub sidecars: Vec<PathBuf>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub fn discover(
    tests_root: &Path,
) -> Result<DiscoveredLayout, std::io::Error>;

pub fn unknown_roots(
    tests_root: &Path,
) -> Result<Vec<PathBuf>, std::io::Error>;
```

これは public `mizar_test::layout` module surface であり、crate root では
re-export しない。`DiscoveredLayout` は deterministic に sort した payload と
sidecar path inventory、および layout diagnostics を持つ。`discover` は optional
known root と adjacent sidecar の欠落を報告し、`unknown_roots` は sort 済み
unknown-directory inventory を返す。harness はこの inventory に
validation-mode policy を適用する。

authoritative expectations は sidecar files に置く。fail、soundness、certificate、snapshot expectations は `.miz` frontend correctness に依存せず parse できなければならないため、sidecar を必須とする。inline metadata は parser が安全に無視できる non-authoritative tags に限って許可する。

layout module は parsed test case または expectation metadata を所有しない。
`TestCase` と `TestPlan` は harness-owned、`TestKind` と `Expectation` は
expectation-schema-owned である。[harness.md](./harness.md) と
[expectation_schema.md](./expectation_schema.md) を参照する。

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
2. discover した `PathBuf` values を deterministic に sort する。
3. executable payload と sidecar の path inventory を別々に collect する。
4. optional known root または adjacent sidecar の欠落を layout diagnostic として記録する。
5. `DiscoveredLayout` を返す。harness が expectation を parse し、deterministic な `TestCase`/`TestPlan` projection を構築する。

## Tests

required layout-focused scenarios:

- discovery order が filesystems をまたいで stable
- missing executable payload metadata は error
- optional known root の欠落は warning になる
- unknown-directory inventory は deterministic に sort される

`crates/mizar-test/tests/layout.rs` は raw payload/sidecar ordering、
missing-known-root warning、複数 unknown-root ordering を直接検証する。現在の
harness/expectation integration は missing-sidecar rejection を検証し、duplicate
test id を reject し、generated / fuzz-minimized origin metadata を保持し、
unknown directory に explicit validation-mode policy を適用する。

## Constraints and Assumptions

- test discovery は OS directory iteration order に依存してはならない。
- sidecar metadata schema は versioned である。
- pass tests は diagnostics なしを期待する場合 `diagnostic_codes = []` を記録する。
- fail tests は expected failure category and stable detail key を明記しなければならない。
