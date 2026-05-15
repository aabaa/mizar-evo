# 内部設計仕様

> Canonical language: English. English canonical version: [../en/README.md](../en/README.md).

このディレクトリには、architecture specifications を compiler modules、API、data structures、execution contracts へ落とし込む**内部 subsystem 設計文書**を置く。

## 目的

[`doc/design/architecture/ja/`](../../architecture/ja/README.md) の architecture documents は phase boundaries と subsystem responsibilities を定義する。Internal design documents は、それらの境界をどう実装するかを説明する。ただし、1:1 の module specification までは踏み込まない。

Internal design documents は次のような問いに答える。

- pipeline phase transition はどの service が所有するか？
- compiler subsystems 間で共有する data structures は何か？
- どの API が synchronous、asynchronous、event-based か？
- cache、diagnostics、cancellation、artifact commit の判断をどこで enforce するか？
- batch、watch、LSP builds は同じ compiler driver をどう共有するか？

## Index

Documents は implementation dependency order で番号付けする。未作成の番号付き文書は planned slot として残す。

| Document | Pipeline Phase | Description | Status |
|---|---:|---|---|
| [00.internal_overview.md](./00.internal_overview.md) | All | internal design scope、crate/service boundaries、architecture documents との関係 | Draft |
| [01.compiler_driver_and_pipeline_scheduler.md](./01.compiler_driver_and_pipeline_scheduler.md) | 0-16 | compiler driver、task graph scheduler、phase services、cancellation、cache lookup、diagnostics、artifact commit orchestration | Draft |
| [02.artifact_store_cache_key_and_manifest.md](./02.artifact_store_cache_key_and_manifest.md) | 15 | artifact store、cache key construction、manifest transactions、reproducible write protocol | Draft |
| [03.diagnostics_model_and_lsp_bridge.md](./03.diagnostics_model_and_lsp_bridge.md) | All, 15 | diagnostic registry、aggregation、explanation handles、LSP snapshot bridge、editor freshness model | Draft |
| [04.atp_portfolio_and_kernel_check_integration.md](./04.atp_portfolio_and_kernel_check_integration.md) | 13-14 | ATP portfolio execution、backend evidence selection、proof witness storage、kernel check scheduling | Draft |
| [05.documentation_extraction.md](./05.documentation_extraction.md) | 16 | documentation extraction inputs、render model、code extraction boundary、artifact consumers | Draft |

## 文書テンプレート

各 internal design document は次の構成に従う。

```markdown
# Internal Design: <Title>

## Purpose
この文書が扱う implementation problem。

## Context
この design が詳細化する architecture and spec documents への参照。

## Responsibilities
internal services、modules、data structures 間の具体的な ownership boundaries。

## Data Model
重要な internal types、identity rules、invariants。

## Control Flow
requests、phase outputs、diagnostics、cache records、artifacts が subsystem をどう流れるか。

## API Sketch
modules 間で使う traits、structs、events、service calls。

## Error Handling
failure、cancellation、recovery、diagnostic publication rules。

## Affected Modules
この設計を実装する expected module-level specs and source files。

## Constraints and Assumptions
performance、reproducibility、compatibility、trust-boundary constraints。
```

## 他の文書層との関係

| Layer | Directory | Granularity | Audience |
|---|---|---|---|
| External Spec | `doc/spec/ja/` | Language features and user-visible behavior | Users |
| Architecture | `doc/design/architecture/ja/` | Cross-cutting subsystem boundaries | Developers |
| **Internal Design** | **`doc/design/internal/ja/`** | **Subsystem APIs、data structures、execution contracts** | **Compiler developers** |
| Module Spec | `doc/design/<crate>/` | Individual files (1:1) | Developers |
