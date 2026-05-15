# Architecture: ATP Backend Integration

> Canonical language: English. English canonical version: [../en/atp_backend_integration.md](../en/atp_backend_integration.md).

## 目的

この文書は、Mizar Evo が external ATP/SMT backends を実行する方法を定義する。対象は process execution、timeout handling、portfolio execution、result classification、proof certificate collection である。

これは pipeline phase 13 のうち backend dispatch を詳細化する。Problem encoding は [atp_interface_protocol.md](./atp_interface_protocol.md) が扱い、proof acceptance は phase 14 の kernel certificate checking で final になる。

## Context

- [00.pipeline_overview.md](./00.pipeline_overview.md) — overall pipeline
- [reasoning_boundary.md](./reasoning_boundary.md) — reasoning responsibility split
- [atp_interface_protocol.md](./atp_interface_protocol.md) — problem encoding protocols
- [doc/spec/en/21.source_code_annotation_and_atp.md](../../../spec/en/21.source_code_annotation_and_atp.md) — backend provers、portfolio execution、certificate formats
- [doc/spec/en/22.error_handling_and_diagnostics.md](../../../spec/en/22.error_handling_and_diagnostics.md) — ATP timeout and proof diagnostics
- [doc/spec/en/23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md) — verifier config、logs、artifact output

## Design Decisions

### Supported Backends

| Backend | Type | Input Format | Certificate Format | Priority |
|---|---|---|---|---|
| Vampire | ATP | TPTP | TSTP | Primary |
| E | ATP | TPTP | TSTP | Primary |
| CVC5 | SMT | SMT-LIB | LFSC / Alethe | Primary for theory-heavy goals |
| Z3 | SMT | SMT-LIB | proof log / externally attested | Optional |

### Process Model

Backends は child processes として起動する。

- Input は stdin または temporary problem file 経由で渡す。
- Stdout と stderr は logs のために capture する。
- Timeout は obligation ごとに設定する。default は `mizar.pkg` の `[verifier].atp_timeout` である。
- Crash した backend は backend error として報告され、verifier を crash させない。

### Portfolio Execution

selected solver が `auto` の場合、Mizar Evo は複数 backend を parallel に起動してよい。kernel-accepted certificate を返した最初の backend が勝者になる。残りの process は terminate される。

Reproducibility のため、portfolio run は次を記録しなければならない。

- backend names and versions
- concrete input hash
- timeout and resource limits
- random seed when applicable
- certificate status

### Result Classification

```rust
enum ATPResult {
    Proved(ProofCertificate),
    Disproved,
    Timeout,
    Unknown,
    Error(ATPError),
}
```

`Proved` は backend が evidence を生成したことを意味する。kernel が proof を受理したことは意味しない。

## Alternatives Considered

1. **Library linking**: process overhead は低いが、licensing と version isolation が難しい。
2. **Long-running prover daemon**: startup cost は低いが、resource management が複雑になる。
3. **Child process execution**: isolation と version management が単純で、overhead も許容範囲である。

## Adopted Approach

Mizar Evo は child process execution と optional portfolio parallelism を採用する。

## Interface Definitions

```rust
trait ATPBackend {
    fn name(&self) -> &str;
    fn solve(&self, problem: EncodedProblem, timeout: Duration) -> Result<ATPResult, ATPError>;
}
```

`EncodedProblem` は concrete backend input と、logs / reproducibility に必要な metadata を持つ。

## Affected Modules

- `doc/design/mizar-atp/backend.md` — backend trait and process execution
- `doc/design/mizar-atp/portfolio.md` — portfolio execution strategy
- `doc/design/mizar-atp/certificate.md` — certificate parsing
- `doc/design/mizar-kernel/certificate.md` — certificate validation
- [atp_interface_protocol.md](./atp_interface_protocol.md)

## Constraints and Assumptions

- Backend binaries は `PATH` 上にあるか、明示的に設定されることを期待する。
- Backend versions は verifier artifacts と ATP logs に記録する。
- Backend crashes は graceful に扱う。
- Kernel certificate checking が trusted acceptance boundary である。
