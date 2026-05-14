# Architecture: Reasoning Boundary

## Purpose

Mizar Evo における**推論責務の切り分け**を定義する。
どの推論をMizar側（カーネル・型チェッカ）が担当し、どの推論を外部ATP側に委譲するかの境界線を明確にする。

## Context

- [00.pipeline_overview.md](./00.pipeline_overview.md) — 全体パイプライン。本文書は Phase 12-14 を詳細化する
- [doc/spec/16.theorems_and_proofs.md](../../spec/16.theorems_and_proofs.md) — 証明構文と `by` justification
- [doc/spec/17.clusters_and_registrations.md](../../spec/17.clusters_and_registrations.md) — クラスタ登録
- [doc/spec/20.algorithm_and_verification.md](../../spec/20.algorithm_and_verification.md) — algorithm verification と verification condition
- [doc/spec/21.source_code_annotation_and_atp.md](../../spec/21.source_code_annotation_and_atp.md) — annotation と ATP integration
- [doc/spec/23.package_management_and_build_system.md](../../spec/23.package_management_and_build_system.md) — build lifecycle, cluster-db, artifact output

### Pipeline Position

本文書は以下のフェーズ境界を扱う。

| Phase | Responsibility |
|---|---|
| 12. Pre-ATP Discharge | Mizar 側で閉じられる obligation を処理する |
| 13. ATP Translation / Dispatch | open VC を ATP problem に変換し、外部 prover に委譲する |
| 14. Kernel Certificate Check | ATP 結果を独立検証し、verified proof status を確定する |

## Design Decisions

### Mizar側（カーネル / 型チェッカ）が担う推論

<!-- TODO: 各項目の詳細な仕様を記述 -->

- **型チェック**: 型の整合性、サブタイプ関係の検証
- **クラスタ推論**: existential / conditional / functorial registration の適用
- **sethood 判定**: Fraenkel式使用時の sethood 登録確認（コンパイル時拒否）
- **等式展開**: definitional expansion（定義展開）
- **構文糖の脱糖**: 言語構文からコア論理形式への変換

### ATP側に委譲する推論

<!-- TODO: 各項目の詳細な仕様を記述 -->

- **一階述語論理の推論**: `by` justification に対する推論ステップの探索
- **等式推論**: 公理に基づく等式チェーン生成
- **Property活用**: commutativity, symmetry 等の性質を公理/AC宣言として渡す
- **反駁証明**: 仮定の否定から矛盾を導出

### Alternatives Considered

<!-- TODO: 比較テーブルを記述 -->

1. **全て組み込み推論器**（従来のMizar方式）
2. **全てATP委譲**（Sledgehammer方式）
3. **ハイブリッド: 型レベルはカーネル、論理推論はATP**（採用方針）

### Adopted Approach

ハイブリッド方式を採用。根拠:
- カーネルのTCB（信頼すべきコードベース）を最小化
- 型チェックはATPに渡すまでもなくローカルに決定可能
- 論理推論はATPの方が圧倒的に高性能

## Interface Definitions

<!-- TODO: カーネル → ATP への問題生成インターフェースを定義 -->

```
カーネル（型チェック完了）
   ↓ 局所的ゴール + 利用可能な前提の集合
問題生成器（Translator）
   ↓ TPTP / SMT-LIB 形式の問題
ATP バックエンド
   ↓ 証明証明書
カーネル（証明書検証）
```

## Affected Modules

- `doc/design/mizar-kernel/checker.md` — 型チェック・クラスタ推論
- `doc/design/mizar-vc/generator.md` — verification condition 生成
- `doc/design/mizar-atp/translator.md` — 問題変換
- `doc/design/mizar-atp/backend.md` — ATP呼び出し
- `doc/design/mizar-kernel/certificate.md` — 証明証明書検証
- → [atp_interface_protocol.md](./atp_interface_protocol.md)
- → [atp_backend_integration.md](./atp_backend_integration.md)

## Constraints and Assumptions

- カーネルはATPの結果を盲目的に信頼しない（de Bruijn criterion）
- 証明証明書の独立検証が必須
- ATP非可用時でもカーネル単体で型チェック・クラスタ推論は動作すること
