# アーキテクチャ設計仕様

> Canonical language: English. English canonical version: [../en/README.md](../en/README.md).

このディレクトリには、複数の module や crate にまたがる境界、protocol、設計判断を定義する**横断的な内部設計文書**を置く。

## 目的

module-level spec（`doc/design/<crate>/<module>.md`）が個別の Rust source file を説明するのに対し、architecture spec は単一 module だけでは答えられない問題を扱う。

- **どこが subsystem の境界か？** 例: kernel と ATP の境界
- **subsystem 間をつなぐ protocol は何か？** 例: TPTP, SMT-LIB
- **外部 tool をどう統合するか？** 例: process 管理、timeout
- **なぜその設計を選んだのか？**

## Index

| Document | Pipeline Phase | Description | Status |
|---|---:|---|---|
| [00.pipeline_overview.md](./00.pipeline_overview.md) | All | source file から verified artifact までの end-to-end pipeline | Draft |
| [ir_layers.md](./ir_layers.md) | All | pipeline phase 間の IR ownership boundary と安定性ルール | Draft |
| [source_and_frontend.md](./source_and_frontend.md) | 1-3 | source loading, preprocessing, lexing, parsing の境界 | Draft |
| [reasoning_boundary.md](./reasoning_boundary.md) | 12-14 | Mizar、ATP backend、kernel の reasoning responsibility split | Draft |
| [atp_interface_protocol.md](./atp_interface_protocol.md) | 13 | ATP problem format と encoding strategy | Draft |
| [atp_backend_integration.md](./atp_backend_integration.md) | 13 | 外部 ATP process execution、timeout handling、certificate collection | Draft |

`00.pipeline_overview.md` はこのディレクトリの親文書である。他の architecture 文書は、自分がどの pipeline phase を詳細化するかを明記し、Context section から overview へ link する。

## 文書テンプレート

各 architecture 文書は次の構成に従う。

```markdown
# Architecture: <Title>

## Purpose
この文書が扱う architecture problem。

## Context
関連する外部仕様や architecture 文書への参照。

## Design Decisions

### Alternatives Considered
検討した approach と trade-off。

### Adopted Approach
採用した設計と理由。

## Interface Definitions
subsystem 間の境界、API、data format。

## Affected Modules
この設計を実装する module-level spec と source file。
- `doc/design/<crate>/<module>.md` → `crates/<crate>/src/<module>.rs`

## Constraints and Assumptions
performance requirements, security considerations, compatibility など。
```

## 他の文書層との関係

| Layer | Directory | Granularity | Audience |
|---|---|---|---|
| External Spec | `doc/spec/` | Language features | Users |
| **Architecture** | **`doc/design/architecture/`** | **Cross-cutting subsystems** | **Developers** |
| Module Spec | `doc/design/<crate>/` | Individual files (1:1) | Developers |
