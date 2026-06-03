# Module: retention

> Canonical language: English. English canonical version: [../en/retention.md](../en/retention.md).

## Purpose

このモジュールは、`mizar-session` の保持リースと回収方針を定義します。

batch・watch・LSP・診断・説明・キャッシュ・アーティファクト・IR の各利用側がまだソーステキスト・ソースマップ・スナップショットメタデータを参照している間、それらを生かし続けます。型付き IR 出力を直接保持することはありません。IR 出力の保持は `mizar-ir` が所有し、そのハンドルが生存している間はスナップショットリースを保持してよいものとします。

## Public API

```rust
pub struct RetentionManager<A = InMemorySessionIdAllocator> { /* private fields */ }

pub struct RetainSnapshotInput {
    pub snapshot: BuildSnapshotId,
    pub owner: RetainOwner,
    pub reason: RetentionReason,
}

pub struct RetainGuard {
    pub lease_id: SnapshotLeaseId,
    pub snapshot: BuildSnapshotId,
}

pub struct CollectionSummary {
    pub scanned: usize,
    pub collected: usize,
    pub released_sources: usize,
    pub released_maps: usize,
    pub skipped_current: usize,
    pub skipped_live_leases: usize,
    pub lease_diagnostics: Vec<RetentionLeaseDiagnostic>,
}

#[non_exhaustive]
pub enum RetentionLeaseDiagnostic {
    StaleLiveLease {
        lease_id: SnapshotLeaseId,
        snapshot: BuildSnapshotId,
    },
    StaleLeaseCount {
        snapshot: BuildSnapshotId,
        live_count: usize,
    },
    MismatchedLeaseCount {
        snapshot: BuildSnapshotId,
        expected_live_count: usize,
        actual_live_count: usize,
    },
}

pub struct RetainedSnapshotResources {
    pub sources: usize,
    pub maps: usize,
}

#[non_exhaustive]
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

#[non_exhaustive]
pub enum RetentionError {
    UnknownSnapshotId { snapshot_id: BuildSnapshotId },
    UnknownOrReleasedLeaseId { lease_id: SnapshotLeaseId },
    LeaseSnapshotMismatch {
        lease_id: SnapshotLeaseId,
        expected_snapshot: BuildSnapshotId,
        actual_snapshot: BuildSnapshotId,
    },
    InvalidOwnerReasonCombination {
        owner: RetainOwner,
        reason: RetentionReason,
    },
    AttemptToMarkMissingSnapshotCurrent { snapshot_id: BuildSnapshotId },
    CollectionBlockedByInconsistentRetentionState {
        snapshot_id: BuildSnapshotId,
        detail: &'static str,
    },
}

impl RetentionManager<InMemorySessionIdAllocator> {
    pub fn new() -> Self;
}

impl Default for RetentionManager<InMemorySessionIdAllocator> {
    fn default() -> Self;
}

impl<A> RetentionManager<A> {
    pub fn with_allocator(allocator: A) -> Self;
    pub fn register_snapshot(&self, snapshot: BuildSnapshotId) -> bool;
    pub fn record_retained_resources(
        &self,
        snapshot: BuildSnapshotId,
        resources: RetainedSnapshotResources,
    ) -> Result<(), RetentionError>;
    pub fn mark_current(&self, snapshot: BuildSnapshotId) -> Result<bool, RetentionError>;
    pub fn unmark_current(&self, snapshot: BuildSnapshotId) -> Result<bool, RetentionError>;
    pub fn collect(&self) -> CollectionSummary;
}

impl<A: SessionIdAllocator> RetentionManager<A> {
    pub fn retain_snapshot(&self, input: RetainSnapshotInput) -> Result<RetainGuard, RetentionError>;
    pub fn retain_existing_lease(&self, lease: SnapshotLease, owner: RetainOwner) -> Result<RetainGuard, RetentionError>;
    pub fn release(&self, guard: RetainGuard) -> Result<(), RetentionError>;
}
```

タスク 17 では、既知スナップショットの登録と、lease の retain/release 計上を実装します。
`register_snapshot` は、他の層で生成されたスナップショットを retention manager が知るために記録します。
これはスナップショットを current にはしません。`retain_existing_lease` は、`SnapshotRegistry::create_snapshot` が返す active-build lease など、snapshot registry がすでに割り当てた `SnapshotLease` を、別の lease id を割り当てずに retention の台帳へ接続します。`RetainGuard` の解放は、呼び出し側から見ると冪等であるべきですが、重複した解放の試行は `RetentionError` として報告され、参照カウントをアンダーフローさせません。

タスク 18 では current mark と回収を追加します。`mark_current` と
`unmark_current` は mark 集合が変化したかを返します。存在しないスナップショットを
current にしようとした場合は
`RetentionError::AttemptToMarkMissingSnapshotCurrent` を返します。`collect` は
retention manager のインメモリ状態だけを走査して `CollectionSummary` を返し、公開
アーティファクトやキャッシュレコードは削除しません。
`record_retained_resources` は、スナップショットが所有するインメモリの source/map
footprint を記録します。この footprint はスナップショット自体が回収されるときに解放されます。
診断・説明・LSP・IR などがそれらのリソースを外部参照している場合は、引き続き生存中リースで表します。

## Dependencies

- Internal: `ids`, `snapshot`, `source`, `source_map`
- External: 弱参照またはアリーナストレージのユーティリティ、トレース／ロギング

このモジュールは、スナップショットレジストリ、LSP スナップショットの公開、診断の集約、説明クエリ、キャッシュ／アーティファクトのライタ、`mizar-ir` から利用されます。

## Data Structures

### Retention Record

保持される各スナップショットは、次を含むレコードを持ちます。

- スナップショット ID
- 所有者と理由ごとの参照カウント
- それを指名するスナップショット単位の current mark
- 保持中の読み込み済みソース
- 保持中の行マップと前処理マップ
- 回収可否のメタデータ
- 任意で、デバッグ用の生成／解放トレース

このレコードはセッションローカルな保持状態を追跡します。公開アーティファクトにはシリアライズされません。
保持中 source/map のリソース数は回収会計のメタデータであり、独立した外部 keep-alive ではありません。
それらのリソースをまだ必要とする利用側は、適切な `RetentionReason` のリースを保持します。

### Current Marks

current mark は、retention manager がそのスナップショットを回収上の現行インメモリ
ベースラインとして保持しなければならないことを意味します。これはリースとは別物です。
current mark は回収を防ぎ、リースは特定の利用側のために回収を防ぎます。
current mark はリクエスト世代をエンコードせず、鮮度も決定しません。
リクエストスコープの current 判定は `SnapshotRegistry::is_current_for_request` に残ります。

新しいスナップショットが現行になった後でも、古いスナップショットは、失効した診断や説明リクエストのためにリースを保持してよいものとします。

retain/release は current mark を作成・更新しません。
`DiagnosticIndex`、`ExplanationRequest`、`PublishedLspSnapshot`、`PhaseOutputReference` のために古いスナップショットを保持しても、その利用側のために生存させるだけで current にはしません。
current mark は、生存中のリースとは独立に解除できます。

### Collection Summary

`CollectionSummary` は次を報告します。

- 走査したスナップショット数
- 回収したスナップショット数
- 解放したソースとマップ
- 現行マークのために回収を見送ったスナップショット
- 生存中のリースのために回収を見送ったスナップショット
- 失効または不一致のリースに関する診断

これはロギングとテストを目的とし、ビルドの意味論のためのものではありません。
collection 時の stale live lease、stale count ledger entry、live lease map と count ledger の不一致は
diagnostic-summary-only の surface です。これらは `CollectionSummary::lease_diagnostics` で報告し、
`collect` が `RetentionError` を返す理由にはしません。

## Algorithm / Logic

### Retain

`retain_snapshot` では:

1. スナップショットが存在することを検証する。
2. retention manager がすでに知っている id を避けつつ `SnapshotLeaseId` を割り当てる。
3. 所有者／理由のカウントを増やす。
4. `RetainGuard` を返す。

`retain_existing_lease` では:

1. スナップショットが存在することを検証する。
2. 渡された `SnapshotLease` の owner/reason の組み合わせを検証する。
3. 重複した id を割り当てず、既存の lease id を記録する。
4. 所有者／理由のカウントを増やし、`RetainGuard` を返す。

理由が診断・説明・LSP の失効アーティファクト表示・IR 出力の保持のいずれかである場合、失効したスナップショットを保持してよいものとします。ただし、それによってスナップショットを現行にしてはなりません。

### Release

1. リース ID とスナップショット ID を検証する。
2. 所有者／理由のカウントを減らす。
3. リースを解放済みとして記録する。
4. リースが残っていなければ、retention record は生存中リースによってはブロックされず、current mark が残っていなければ回収できます。

解放は、別のスレッドが生存中のガードを通してまだ読み取れるデータを、同期的に削除してはなりません。

### Collection

回収器は、次の条件を満たすスナップショットを取り除いてよいものとします。

- それを参照する生存中のリースがない
- それを参照する現行マークがない
- source-map・診断説明・LSP 公開・IR phase-output 参照のリースが生存していない

回収は、インメモリのソーステキスト・ソースマップ・スナップショットメタデータを破棄します。公開アーティファクトやキャッシュレコードは削除しません。

`PhaseOutputReference` は `IrStorage` が所有する通常の生存中リースとして表されるため、解放されるまで回収をブロックします。
cache writer と artifact writer の `PendingWrite` リースも生存中はブロックしますが、回収そのものは artifact/cache の削除出力を持ちません。
失効した live lease、または live lease map と参照カウント台帳の不一致は `CollectionSummary` に記録され、影響する登録済みスナップショットの回収判断は保守的になります。
したがって collector は、collection 時の inconsistent state を診断として観測可能にしつつ、走査 summary を返します。
これらの collection-time diagnostics には
`RetentionError::CollectionBlockedByInconsistentRetentionState` を使いません。

## Error Handling

`RetentionError` には次が含まれます。

- 未知のスナップショット ID
- 未知、または既に解放済みのリース ID
- リースとスナップショットの不一致
- 不正な所有者／理由の組み合わせ
- 存在しないスナップショットを現行としてマークしようとした
- 一貫性のない保持状態によって回収がブロックされた

有効な owner/reason の組み合わせは次のとおりです。

- `Build(_)` と `ActiveBuild`
- `Watch` と `CurrentWatchBaseline`
- `Lsp(_)` と `PublishedLspSnapshot` または `OpenBufferOverlay`
- `Diagnostics` と `DiagnosticIndex`
- `Explanation` と `ExplanationRequest`
- `IrStorage` と `PhaseOutputReference`
- `CacheWriter` または `ArtifactWriter` と `PendingWrite`

不正な保持状態はコンパイラの内部エラーです。利用者向けのビルドは、可能な場合は以前の整合したスナップショットを使い続けるべきです。
`RetentionError::CollectionBlockedByInconsistentRetentionState` は public であり、
duplicate lease-id allocation、lease-id allocation failure、または live-lease count ledger がすでに不整合な状態での release など、
retain/release/allocation 経路だけで観測されます。
collection 時の stale または mismatched lease state は、代わりに
`CollectionSummary::lease_diagnostics` を使います。
これにより `collect` API を non-failing に保ち、すでに incoherent な retention ledger を変更してしまう操作には別の error surface を残します。

## Tests

主なシナリオ:

- 実行中ビルドのリースが回収を防ぐ
- 現行マークは、他のリースがなくても回収を防ぐ
- 失効した LSP または診断のリースは、スナップショットを現行にすることなく古いソースマップを保持する
- 最後のリースを解放すると、スナップショットが回収可能になる
- `mizar-ir` のフェーズ出力リースは、解放されるまでスナップショットの回収をブロックする
- 重複した解放は報告されるが、カウントをアンダーフローさせない
- 回収はアーティファクトやキャッシュレコードを削除しない
- collection 時の stale または mismatched lease state は
  `CollectionSummary::lease_diagnostics` で報告し、retain/release/allocation の不整合は
  `RetentionError::CollectionBlockedByInconsistentRetentionState` を返す

## Constraints and Assumptions

- 保持はメモリの寿命を制御するものであり、意味的な有効性を制御するものではない。
- 古いスナップショットは、参照されている間は読み取り可能だが、置き換え後に現行として報告することはできない。
- 回収の順序が、決定的なビルド出力に影響してはならない。
- 保持マネージャは、watch／LSP モードにおいて、すべての過去のスナップショットを無期限に保持し続けることを避けなければならない。
