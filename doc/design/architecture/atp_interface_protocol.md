# Architecture: ATP Interface Protocol

## Purpose

外部ATPとやり取りする**通信言語（問題エンコーディング形式）**の設計を定義する。
Mizar の内部表現をどのようにTPTP / SMT-LIB に変換し、どのような戦略でATPに渡すかを規定する。

## Context

- [00.pipeline_overview.md](./00.pipeline_overview.md) — 全体パイプライン。本文書は Phase 13 の ATP problem translation を詳細化する
- [doc/idea/property_verification_and_atp.md](../../idea/property_verification_and_atp.md) — TPTP/SMT-LIB エンコーディング例
- [doc/spec/21.source_code_annotation_and_atp.md](../../spec/21.source_code_annotation_and_atp.md) — ATP translation, property encoding, proof certificates
- [doc/spec/23.package_management_and_build_system.md](../../spec/23.package_management_and_build_system.md) — verifier settings, ATP logs, used-axiom recording
- [reasoning_boundary.md](./reasoning_boundary.md) — 推論の切り分け
- [atp_backend_integration.md](./atp_backend_integration.md) — backend process execution

### Pipeline Position

本文書は Phase 13 のうち、`VcIr` / `AtpProblem` から TPTP / SMT-LIB の concrete problem を生成する境界を扱う。
外部 prover の起動、timeout、portfolio 実行、certificate 受け取りは [atp_backend_integration.md](./atp_backend_integration.md) が扱う。

## Design Decisions

### 対応フォーマット

<!-- TODO: 各フォーマットの詳細仕様を記述 -->

| Format | Target ATPs | Use Case |
|---|---|---|
| TPTP (FOF/TFF) | Vampire, E | 一階述語論理の推論 |
| SMT-LIB 2 | CVC5, Z3 | 等式推論、算術を含む推論 |

### エンコーディング方針

<!-- TODO: 各項目の変換規則を詳細に記述 -->

- **型のエンコーディング**: Mizar の型階層をTPTPのソートまたはガード述語に変換
- **述語・関数のエンコーディング**: Mizar のシンボルをTPTP/SMT-LIBのシンボルにマッピング
- **Property のエンコーディング**: 性質を公理として渡すか、ネイティブ機能として宣言するか

### Property エンコーディング戦略

<!-- TODO: idea から昇格して詳細化 -->

| Property | TPTP (Vampire/E) | SMT-LIB (CVC5/Z3) |
|---|---|---|
| `commutativity` | AC宣言（ネイティブ） | `forall` 公理 |
| `symmetry` | 含意公理 | `forall` 公理 |
| `reflexivity` | 全称公理 | `forall` 公理 |
| `idempotence` | 等式公理 | 等式公理 |
| `involutiveness` | 等式公理 | 等式公理 |
| `projectivity` | 等式公理 | 等式公理 |
| `asymmetry` | 含意公理 | 含意公理 |
| `connectedness` | 含意公理 | 含意公理 |
| `irreflexivity` | 否定公理 | 否定公理 |

### Alternatives Considered

<!-- TODO: 詳細な比較を記述 -->

1. **TPTP のみ**: シンプルだがSMTソルバの算術推論を活用できない
2. **SMT-LIB のみ**: 算術に強いがVampire/Eの一般FOL性能を活用できない
3. **両方対応（採用方針）**: 問題の性質に応じて最適なバックエンドを選択

### Adopted Approach

デュアルフォーマット対応。理由:
- Vampire/E は純粋FOLに特化して高性能
- CVC5/Z3 は等式・算術混合推論に強い
- 問題の特性に応じた自動選択を可能にする

## Interface Definitions

### 入力: Mizar 内部表現

<!-- TODO: 内部ASTの型定義を記述 -->

```
struct ATPProblem {
    conjecture: Formula,        // 証明すべきゴール
    premises: Vec<Formula>,     // 利用可能な前提（by で参照された定理等）
    properties: Vec<Property>,  // 適用される性質（commutativity 等）
    type_context: TypeContext,  // 型情報
}
```

### 出力: エンコードされた問題文字列

<!-- TODO: 具体的な出力例を記述 -->

- TPTP形式: `fof(...)` / `tff(...)` の列
- SMT-LIB形式: `(assert ...)` / `(check-sat)` の列

## Affected Modules

- `doc/design/mizar-atp/translator.md` — TPTP/SMT-LIB への変換実装
- `doc/design/mizar-atp/encoder.md` — Property エンコーダ
- → [reasoning_boundary.md](./reasoning_boundary.md)
- → [atp_backend_integration.md](./atp_backend_integration.md)

## Constraints and Assumptions

- エンコーディングは可逆でなくてよい（証明証明書の検証はカーネル側で行う）
- TPTP/SMT-LIB の標準仕様に準拠する
- ATP固有の拡張機能（Vampire の `--ac_reasoning` 等）は条件付きで利用可
