# Module: retention

> Canonical language: English. English canonical version: [../en/retention.md](../en/retention.md).

## Purpose

このモジュールは、`mizar-session` の保持リースと回収方針を定義します。

batch・watch・LSP・診断・説明・キャッシュ・アーティファクト・IR の各利用側がまだソーステキスト・ソースマップ・スナップショットメタデータを参照している間、それらを生かし続けます。型付き IR 出力を直接保持することはありません。IR 出力の保持は `mizar-ir` が所有し、そのハンドルが生存している間はスナップショットリースを保持してよいものとします。

## Public API

```rust
pub struct RetentionManager {
    // implementation-owned registry
}

pub struct RetainSnapshotInput {
    pub snapshot: BuildSnapshotId,
    pub owner: RetainOwner,
    pub reason: RetentionReason,
}

pub struct RetainGuard {
    pub lease_id: SnapshotLeaseId,
    pub snapshot: BuildSnapshotId,
}

pub enum RetainOwner {
    Build(BuildRequestId),
    Watch,
    Lsp(DocumentUri),
    Diagnostics,
    Explanation,
    IrStorage,
    CacheWriter,
    ArtifactWriter,
}

// `RetentionReason` は snapshot／共有リース層が所有し（snapshot.md 参照）、ここでは
// 再エクスポートする。`SnapshotLease.reason` が本モジュールの実装より前に必要とするため。
// variant:
//   ActiveBuild, CurrentWatchBaseline, PublishedLspSnapshot, OpenBufferOverlay,
//   DiagnosticIndex, ExplanationRequest, PhaseOutputReference, PendingWrite
pub use crate::snapshot::RetentionReason;

pub trait SnapshotRetention {
    fn retain_snapshot(&self, input: RetainSnapshotInput) -> Result<RetainGuard, RetentionError>;
    fn release(&self, guard: RetainGuard);
    fn mark_current(&self, request: BuildRequestId, snapshot: BuildSnapshotId) -> Result<(), RetentionError>;
    fn unmark_current(&self, request: BuildRequestId, snapshot: BuildSnapshotId);
    fn collect(&self) -> CollectionSummary;
}
```

`RetainGuard` の解放は、呼び出し側から見ると冪等であるべきですが、重複した解放の試行は開発者向け診断のために記録されます。

## Dependencies

- Internal: `ids`, `snapshot`, `source`, `source_map`
- External: 弱参照またはアリーナストレージのユーティリティ、トレース／ロギング

このモジュールは、スナップショットレジストリ、LSP スナップショットの公開、診断の集約、説明クエリ、キャッシュ／アーティファクトのライタ、`mizar-ir` から利用されます。

## Data Structures

### Retention Record

保持される各スナップショットは、次を含むレコードを持ちます。

- スナップショット ID
- 所有者と理由ごとの参照カウント
- それを指名する現行のリクエスト世代
- 保持中の読み込み済みソース
- 保持中の行マップと前処理マップ
- 回収可否のメタデータ
- 任意で、デバッグ用の生成／解放トレース

このレコードはセッションローカルな保持状態を追跡します。公開アーティファクトにはシリアライズされません。

### Current Marks

現行マークは、あるビルドリクエスト世代がそのスナップショットを現行として報告してよいことを意味します。これはリースとは別物です。現行マークは回収を防ぎつつ鮮度を制御するのに対し、リースは回収を防ぐだけです。

新しいスナップショットが現行になった後でも、古いスナップショットは、失効した診断や説明リクエストのためにリースを保持してよいものとします。

### Collection Summary

`CollectionSummary` は次を報告します。

- 走査したスナップショット数
- 回収したスナップショット数
- 解放したソースとマップ
- 現行マークのために回収を見送ったスナップショット
- 生存中のリースのために回収を見送ったスナップショット
- 失効または不一致のリースに関する診断

これはロギングとテストを目的とし、ビルドの意味論のためのものではありません。

## Algorithm / Logic

### Retain

1. スナップショットが存在することを検証する。
2. `SnapshotLeaseId` を割り当てる。
3. 所有者／理由のカウントを増やす。
4. `RetainGuard` を返す。

理由が診断・説明・LSP の失効アーティファクト表示・IR 出力の保持のいずれかである場合、失効したスナップショットを保持してよいものとします。ただし、それによってスナップショットを現行にしてはなりません。

### Release

1. リース ID とスナップショット ID を検証する。
2. 所有者／理由のカウントを減らす。
3. リースを解放済みとして記録する。
4. リースも現行マークも残っていなければ、スナップショットを回収可能にする。

解放は、別のスレッドが生存中のガードを通してまだ読み取れるデータを、同期的に削除してはなりません。

### Collection

回収器は、次の条件を満たすスナップショットを取り除いてよいものとします。

- それを参照する生存中のリースがない
- それを参照する現行マークがない
- それに対して登録されたソースマップ・診断の説明・LSP 公開がない
- 下流の `mizar-ir` が、それに対するフェーズ出力の保持リースをすべて解放済みである

回収は、インメモリのソーステキスト・ソースマップ・スナップショットメタデータを破棄します。公開アーティファクトやキャッシュレコードは削除しません。

## Error Handling

`RetentionError` には次が含まれます。

- 未知のスナップショット ID
- 未知、または既に解放済みのリース ID
- リースとスナップショットの不一致
- 不正な所有者／理由の組み合わせ
- 存在しないスナップショットを現行としてマークしようとした
- 一貫性のない保持状態によって回収がブロックされた

不正な保持状態はコンパイラの内部エラーです。利用者向けのビルドは、可能な場合は以前の整合したスナップショットを使い続けるべきです。

## Tests

主なシナリオ:

- 実行中ビルドのリースが回収を防ぐ
- 現行マークは、他のリースがなくても回収を防ぐ
- 失効した LSP または診断のリースは、スナップショットを現行にすることなく古いソースマップを保持する
- 最後のリースを解放すると、スナップショットが回収可能になる
- `mizar-ir` のフェーズ出力リースは、解放されるまでスナップショットの回収をブロックする
- 重複した解放は報告されるが、カウントをアンダーフローさせない
- 回収はアーティファクトやキャッシュレコードを削除しない

## Constraints and Assumptions

- 保持はメモリの寿命を制御するものであり、意味的な有効性を制御するものではない。
- 古いスナップショットは、参照されている間は読み取り可能だが、置き換え後に現行として報告することはできない。
- 回収の順序が、決定的なビルド出力に影響してはならない。
- 保持マネージャは、watch／LSP モードにおいて、すべての過去のスナップショットを無期限に保持し続けることを避けなければならない。
