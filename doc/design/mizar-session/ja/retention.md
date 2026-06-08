# モジュール: retention

> 正本は英語です。英語版: [../en/retention.md](../en/retention.md)。

## 目的

このモジュールは、`mizar-session` の保持リースと回収方針を定義します。

batch・watch・LSP・診断・説明・キャッシュ・アーティファクト・IR の各利用側がまだソーステキスト・ソースマップ・スナップショットメタデータを参照している間、それらを生かし続けます。型付き IR 出力を直接保持することはありません。IR 出力の保持は `mizar-ir` が所有し、そのハンドルが生存している間はスナップショットリースを保持してよいものとします。

## 公開 API

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

タスク 17 では、既知スナップショットの登録と、リースの保持／解放の計上を実装します。
`register_snapshot` は、他の層で生成されたスナップショットを保持マネージャが知るために記録します。
これはスナップショットを現行にはしません。`retain_existing_lease` は、`SnapshotRegistry::create_snapshot` が返す実行中ビルドのリースなど、スナップショットレジストリがすでに割り当てた `SnapshotLease` を、別のリース id を割り当てずに保持の台帳へ接続します。`RetainGuard` の解放は、呼び出し側から見ると冪等であるべきですが、重複した解放の試行は `RetentionError` として報告され、参照カウントをアンダーフローさせません。

タスク 18 では現行マークと回収を追加します。`mark_current` と
`unmark_current` は、マーク集合が変化したかを返します。存在しないスナップショットを
現行にしようとした場合は
`RetentionError::AttemptToMarkMissingSnapshotCurrent` を返します。`collect` は
保持マネージャのインメモリ状態だけを走査して `CollectionSummary` を返し、公開
アーティファクトやキャッシュレコードは削除しません。
`record_retained_resources` は、スナップショットが所有するインメモリのソース／マップ
使用量を記録します。この使用量はスナップショット自体が回収されるときに解放されます。
診断・説明・LSP・IR などがそれらのリソースを外部参照している場合は、引き続き生存中のリースで表します。

## 依存関係

- 内部: `ids`, `snapshot`, `source`, `source_map`
- 外部: 弱参照またはアリーナストレージのユーティリティ、トレース／ロギング

このモジュールは、スナップショットレジストリ、LSP スナップショットの公開、診断の集約、説明クエリ、キャッシュ／アーティファクトのライタ、`mizar-ir` から利用されます。

## データ構造

### 保持レコード

保持される各スナップショットは、次を含むレコードを持ちます。

- スナップショット ID
- 所有者と理由ごとの参照カウント
- それを指名するスナップショット単位の current mark
- 保持中の読み込み済みソース
- 保持中の行マップと前処理マップ
- 回収可否のメタデータ
- 任意で、デバッグ用の生成／解放トレース

このレコードはセッションローカルな保持状態を追跡します。公開アーティファクトにはシリアライズされません。
保持中のソース／マップのリソース数は回収会計のメタデータであり、独立した外部のキープアライブではありません。
それらのリソースをまだ必要とする利用側は、適切な `RetentionReason` のリースを保持します。

### 現行マーク

現行マークは、保持マネージャがそのスナップショットを、回収上の現行のインメモリ
ベースラインとして保持しなければならないことを意味します。これはリースとは別物です。
現行マークは回収を防ぎ、リースは特定の利用側のために回収を防ぎます。
現行マークはリクエスト世代をエンコードせず、鮮度も決定しません。
リクエストスコープの現行判定は `SnapshotRegistry::is_current_for_request` に残ります。

新しいスナップショットが現行になった後でも、古いスナップショットは、失効した診断や説明リクエストのためにリースを保持してよいものとします。

保持／解放は現行マークを作成・更新しません。
`DiagnosticIndex`、`ExplanationRequest`、`PublishedLspSnapshot`、`PhaseOutputReference` のために古いスナップショットを保持しても、その利用側のために生存させるだけで現行にはしません。
現行マークは、生存中のリースとは独立に解除できます。

### 回収サマリ

`CollectionSummary` は次を報告します。

- 走査したスナップショット数
- 回収したスナップショット数
- 解放したソースとマップ
- 現行マークのために回収を見送ったスナップショット
- 生存中のリースのために回収を見送ったスナップショット
- 失効または不一致のリースに関する診断

これはロギングとテストを目的とし、ビルドの意味論のためのものではありません。
回収時の失効した生存中リース、失効した計数台帳のエントリ、生存中リースのマップと計数台帳の不一致は、
診断サマリ専用の経路です。これらは `CollectionSummary::lease_diagnostics` で報告し、
`collect` が `RetentionError` を返す理由にはしません。

## アルゴリズム / ロジック

### 保持

`retain_snapshot` では:

1. スナップショットが存在することを検証する。
2. 保持マネージャがすでに知っている id を避けつつ `SnapshotLeaseId` を割り当てる。
3. 所有者／理由のカウントを増やす。
4. `RetainGuard` を返す。

`retain_existing_lease` では:

1. スナップショットが存在することを検証する。
2. 渡された `SnapshotLease` の所有者／理由の組み合わせを検証する。
3. 重複した id を割り当てず、既存のリース id を記録する。
4. 所有者／理由のカウントを増やし、`RetainGuard` を返す。

理由が診断・説明・LSP の失効アーティファクト表示・IR 出力の保持のいずれかである場合、失効したスナップショットを保持してよいものとします。ただし、それによってスナップショットを現行にしてはなりません。

### 解放

1. リース ID とスナップショット ID を検証する。
2. 所有者／理由のカウントを減らす。
3. リースを解放済みとして記録する。
4. リースが残っていなければ、保持レコードは生存中のリースによってはブロックされず、現行マークが残っていなければ回収できます。

解放は、別のスレッドが生存中のガードを通してまだ読み取れるデータを、同期的に削除してはなりません。

### 回収

回収器は、次の条件を満たすスナップショットを取り除いてよいものとします。

- それを参照する生存中のリースがない
- それを参照する現行マークがない
- ソースマップ・診断説明・LSP 公開・IR フェーズ出力参照のリースが生存していない

回収は、インメモリのソーステキスト・ソースマップ・スナップショットメタデータを破棄します。公開アーティファクトやキャッシュレコードは削除しません。

`PhaseOutputReference` は `IrStorage` が所有する通常の生存中リースとして表されるため、解放されるまで回収をブロックします。
キャッシュライタとアーティファクトライタの `PendingWrite` リースも生存中はブロックしますが、回収そのものはアーティファクト／キャッシュの削除出力を持ちません。
失効した生存中リース、または生存中リースのマップと参照カウント台帳の不一致は `CollectionSummary` に記録され、影響する登録済みスナップショットの回収判断は保守的になります。
したがって回収器は、回収時の不整合な状態を診断として観測可能にしつつ、走査のサマリを返します。
これらの回収時の診断には
`RetentionError::CollectionBlockedByInconsistentRetentionState` を使いません。

## エラー処理

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
`RetentionError::CollectionBlockedByInconsistentRetentionState` は公開されており、
リース id の重複割り当て、リース id の割り当て失敗、または生存中リースの計数台帳がすでに不整合な状態での解放など、
保持／解放／割り当ての経路だけで観測されます。
回収時の失効した、または不一致のリース状態には、代わりに
`CollectionSummary::lease_diagnostics` を使います。
これにより `collect` API を失敗しないものに保ち、すでに不整合な保持台帳を変更してしまう操作には、別のエラー経路を残します。

## テスト

主なシナリオ:

- 実行中ビルドのリースが回収を防ぐ
- 現行マークは、他のリースがなくても回収を防ぐ
- 失効した LSP または診断のリースは、スナップショットを現行にすることなく古いソースマップを保持する
- 最後のリースを解放すると、スナップショットが回収可能になる
- `mizar-ir` のフェーズ出力リースは、解放されるまでスナップショットの回収をブロックする
- 重複した解放は報告されるが、カウントをアンダーフローさせない
- 回収はアーティファクトやキャッシュレコードを削除しない
- 回収時の失効した、または不一致のリース状態は
  `CollectionSummary::lease_diagnostics` で報告し、保持／解放／割り当ての不整合は
  `RetentionError::CollectionBlockedByInconsistentRetentionState` を返す

## 制約と前提

- 保持はメモリの寿命を制御するものであり、意味的な有効性を制御するものではない。
- 古いスナップショットは、参照されている間は読み取り可能だが、置き換え後に現行として報告することはできない。
- 回収の順序が、決定的なビルド出力に影響してはならない。
- 保持マネージャは、watch／LSP モードにおいて、すべての過去のスナップショットを無期限に保持し続けることを避けなければならない。
- 本モジュールは、常駐集合メモリモデル（spec [§12.6.3](../../../spec/ja/12.modules_and_namespaces.md#1263-メモリモデル)、[§23.7.9](../../../spec/ja/23.package_management_and_build_system.md#2379-メモリ設計原則)、内部 [00.internal_overview.md](../../internal/ja/00.internal_overview.md)・[06.ir_storage_and_snapshot_handles.md](../../internal/ja/06.ir_storage_and_snapshot_handles.md)）のセッション層における実装地点である。常駐集合を、active な作業がなお参照するソーステキスト・ソースマップ・スナップショットメタデータに限定し、残りを回収する。`mizar-ir` はここで `PhaseOutputReference` リースを保持するため、IR 出力の寿命はソース／スナップショットの寿命と単一の回収ポリシーの下で合成される。
