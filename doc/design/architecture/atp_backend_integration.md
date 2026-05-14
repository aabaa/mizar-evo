# Architecture: ATP Backend Integration

## Purpose

外部ATPプロセスとの**接続方式**を定義する。
プロセスの起動・管理、タイムアウト制御、証明証明書の受け取り、フォールバック戦略など、
ATPバックエンドとの統合に関わるアーキテクチャを規定する。

## Context

- [00.pipeline_overview.md](./00.pipeline_overview.md) — 全体パイプライン。本文書は Phase 13 の ATP backend dispatch を詳細化する
- [doc/spec/21.source_code_annotation_and_atp.md](../../spec/21.source_code_annotation_and_atp.md) — backend provers, portfolio execution, certificate formats
- [doc/spec/22.error_handling_and_diagnostics.md](../../spec/22.error_handling_and_diagnostics.md) — ATP timeout, proof failure, diagnostic reporting
- [doc/spec/23.package_management_and_build_system.md](../../spec/23.package_management_and_build_system.md) — verifier config, ATP logs, artifact output
- [reasoning_boundary.md](./reasoning_boundary.md) — 推論の切り分け
- [atp_interface_protocol.md](./atp_interface_protocol.md) — 通信言語設計

### Pipeline Position

本文書は Phase 13 のうち、encoded problem を外部 prover に渡し、結果と証明証明書を回収する境界を扱う。
TPTP / SMT-LIB への変換規則は [atp_interface_protocol.md](./atp_interface_protocol.md) が扱い、証明証明書の最終受理は Phase 14 の kernel certificate check が扱う。

## Design Decisions

### 対応バックエンド

<!-- TODO: 各バックエンドの詳細な対応方針を記述 -->

| Backend | Type | Format | Certificate Format | Priority |
|---|---|---|---|---|
| Vampire | ATP | TPTP | TSTP | Primary |
| E | ATP | TPTP | TSTP | Primary |
| CVC5 | SMT | SMT-LIB | LFSC/Alethe | Secondary |
| Z3 | SMT | SMT-LIB | (proof log) | Optional |

### プロセス管理方式

<!-- TODO: 詳細な実装方針を記述 -->

- **起動方式**: 各ATPを子プロセスとして起動（`std::process::Command`）
- **通信方式**: stdin/stdout パイプ経由で問題を渡し、結果を受け取る
- **タイムアウト**: ゴール毎に秒単位で設定可能。デフォルト値は `mizar.pkg` の `[verifier].atp_timeout` に従う
- **並列実行**: 複数ATPを並列起動し、最初に成功したものを採用（portfolio 方式）

### フォールバック戦略

<!-- TODO: 詳細な戦略を記述 -->

```
1. Vampire (TPTP, timeout T1)
2. E-prover (TPTP, timeout T2)     ← 1 が失敗した場合
3. CVC5 (SMT-LIB, timeout T3)      ← 1,2 が失敗した場合
4. 失敗報告                          ← 全て失敗
```

あるいは portfolio 方式:

```
┌─ Vampire  ─┐
├─ E-prover ─┼─→ 最初の成功を採用
└─ CVC5     ─┘
```

### 証明証明書の処理

<!-- TODO: 各形式の解析方法を記述 -->

- **TSTP 形式** (Vampire/E): 推論ステップの列として返却 → カーネルで検証
- **LFSC/Alethe 形式** (CVC5): 証明証明書として返却 → カーネルで検証
- カーネルはATPを信頼しない — 証明書の独立検証が必須

### Alternatives Considered

<!-- TODO: 詳細な比較を記述 -->

1. **ライブラリリンク方式**: ATPをライブラリとして組み込み
   - 長所: 通信オーバーヘッドなし
   - 短所: ライセンス制約、ATPのバージョンアップが困難
2. **サーバ常駐方式**: ATPをデーモンとして常時起動
   - 長所: 起動オーバーヘッドなし
   - 短所: リソース管理が複雑
3. **子プロセス方式（採用方針）**: ゴール毎にプロセス起動
   - 長所: シンプル、隔離性が高い、バージョン管理容易
   - 短所: 起動オーバーヘッド（軽微）

### Adopted Approach

子プロセス方式を採用。根拠:
- ATPのバージョンアップに追従しやすい
- プロセス分離による安全性
- Portfolio 方式との親和性が高い

## Interface Definitions

### Backend Trait

<!-- TODO: 詳細な Trait 定義を記述 -->

```rust
trait ATPBackend {
    /// ATP名 (e.g., "vampire", "eprover", "cvc5")
    fn name(&self) -> &str;

    /// 問題を解き、証明証明書を返す
    fn solve(&self, problem: &str, timeout: Duration) -> Result<ProofCertificate, ATPError>;
}
```

### 結果型

```rust
enum ATPResult {
    Proved(ProofCertificate),  // 証明成功 + 証明書
    Disproved,                  // 反例発見
    Timeout,                    // タイムアウト
    Unknown,                    // 判定不能
    Error(ATPError),           // プロセスエラー
}
```

## Affected Modules

- `doc/design/mizar-atp/backend.md` — Backend trait 実装
- `doc/design/mizar-atp/portfolio.md` — Portfolio 実行戦略
- `doc/design/mizar-atp/certificate.md` — 証明証明書パーサ
- `doc/design/mizar-kernel/checker.md` — 証明証明書検証
- → [atp_interface_protocol.md](./atp_interface_protocol.md)

## Constraints and Assumptions

- 外部ATPのバイナリはユーザーの `PATH` 上に存在することを前提とする
- ATPのバージョンは verifier artifact / ATP interaction log に記録される（[doc/spec/23.package_management_and_build_system.md](../../spec/23.package_management_and_build_system.md)）
- ATPプロセスのクラッシュは graceful に処理し、次のバックエンドにフォールバック
- ビルドシステムがATPバイナリの検出・バージョン確認を担当する
