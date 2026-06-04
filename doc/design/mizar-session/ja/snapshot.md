# モジュール: snapshot

> 正本は英語です。英語版: [../en/snapshot.md](../en/snapshot.md)。

## 目的

このモジュールは、`mizar-session` の不変なビルドスナップショットの同一性を定義します。

`BuildSnapshot` は、1 つの batch・watch・LSP ビルドリクエストが観測する、ビルド入力の状態全体を識別します。ソースバージョン、依存アーティファクト、ロックファイルの状態、ツールチェインの同一性、検証器構成を対象とします。下流の crate は `BuildSnapshotId` を用いて、失効したハンドルを拒否し、以前の出力を再利用する前にキャッシュ検証が必要かどうかを判断します。

## 公開 API

```rust
pub struct BuildSnapshot {
    pub id: BuildSnapshotId,
    pub workspace_root: WorkspaceRoot,
    pub source_versions: Vec<SourceVersion>,
    pub dependency_artifacts: Vec<DependencyArtifactRef>,
    pub lockfile_hash: Hash,
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
}

pub struct SnapshotInput {
    pub workspace_root: WorkspaceRoot,
    pub source_versions: Vec<SourceVersion>,
    pub dependency_artifacts: Vec<DependencyArtifactRef>,
    pub lockfile_hash: Hash,
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
}

pub struct WorkspaceRoot(String);
pub struct DependencyArtifactRef {
    pub artifact: String,
    pub content_hash: Hash,
}
pub struct ToolchainInfo(String);

impl WorkspaceRoot {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

impl DependencyArtifactRef {
    pub fn new(artifact: impl Into<String>, content_hash: Hash) -> Self;
}

impl ToolchainInfo {
    pub fn new(identity: impl Into<String>) -> Self;
    pub fn identity(&self) -> &str;
}

pub struct PackageId(String);
pub struct ModulePath(String);
pub struct Edition(String);
pub struct GeneratedSourceKind(String);

impl PackageId {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

impl ModulePath {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

impl Edition {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

impl GeneratedSourceKind {
    pub fn new(value: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

pub struct SourceVersion {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub source_hash: Hash,
    pub edition: Edition,
    pub origin: SourceOrigin,
}

#[non_exhaustive]
pub enum SourceOrigin {
    Disk,
    OpenBuffer { version: LspDocumentVersion },
    Generated { generator: GeneratedSourceKind },
}

impl SourceVersion {
    pub fn canonical_sort_key(&self) -> SourceVersionCanonicalKey<'_>;
}

// 不透明な比較キー: package id、module path、normalized path、source hash。
pub struct SourceVersionCanonicalKey<'a> { /* private fields */ }

pub fn sort_source_versions_canonical(source_versions: &mut [SourceVersion]);

pub struct SnapshotRegistry<A = InMemorySessionIdAllocator> { /* private fields */ }

impl SnapshotRegistry<InMemorySessionIdAllocator> {
    pub fn new() -> Self;
}

impl Default for SnapshotRegistry<InMemorySessionIdAllocator> {
    fn default() -> Self;
}

impl<A> SnapshotRegistry<A> {
    pub fn with_allocator(allocator: A) -> Self;
}

impl<A: SessionIdAllocator> SnapshotRegistry<A> {
    pub fn create_snapshot(
        &self,
        request: BuildRequestId,
        input: SnapshotInput,
    ) -> Result<(BuildSnapshot, SnapshotLease), SnapshotError>;
    pub fn acquire_lease(
        &self,
        snapshot: BuildSnapshotId,
        reason: RetentionReason,
    ) -> Result<SnapshotLease, SnapshotError>;
    pub fn release_lease(
        &self,
        snapshot: BuildSnapshotId,
        lease_id: SnapshotLeaseId,
    ) -> Result<(), SnapshotError>;
    pub fn get(&self, id: BuildSnapshotId) -> Option<BuildSnapshot>;
    pub fn is_current_for_request(&self, id: BuildSnapshotId, request: BuildRequestId) -> bool;
}

// snapshot／共有リース層が所有し、`retention` モジュールが再エクスポートする。
#[non_exhaustive]
pub enum RetentionReason {
    ActiveBuild,
    CurrentWatchBaseline,
    PublishedLspSnapshot,
    OpenBufferOverlay,
    DiagnosticIndex,
    ExplanationRequest,
    PhaseOutputReference,
    PendingWrite,
}

pub struct SnapshotLease {
    pub lease_id: SnapshotLeaseId,
    pub snapshot: BuildSnapshotId,
    pub reason: RetentionReason,
}

#[non_exhaustive]
pub enum SnapshotError {
    InvalidWorkspaceRoot { workspace_root: WorkspaceRoot },
    InvalidPackageId { package_id: PackageId },
    InvalidModulePath { package_id: PackageId, module_path: ModulePath },
    InvalidEdition { package_id: PackageId, module_path: ModulePath, edition: Edition },
    InvalidSourcePath { error: SourcePathError },
    DuplicateModulePath { package_id: PackageId, module_path: ModulePath },
    DuplicateSourceVersionIdentity {
        package_id: PackageId,
        module_path: ModulePath,
        normalized_path: NormalizedPath,
        source_hash: Hash,
    },
    MissingDependencyArtifact { artifact: String },
    UnsupportedLockfileMetadata { metadata: String },
    UnsupportedToolchainMetadata { metadata: String },
    StaleOpenBufferVersion { expected: LspDocumentVersion, actual: LspDocumentVersion },
    GeneratedSourceWithoutMetadata { module_path: ModulePath },
    UnknownSnapshotId { snapshot_id: BuildSnapshotId },
    LeaseReleaseMismatch {
        lease_id: SnapshotLeaseId,
        expected_snapshot: BuildSnapshotId,
        actual_snapshot: BuildSnapshotId,
    },
    UnknownSnapshotLease { lease_id: SnapshotLeaseId },
    DuplicateLeaseIdAllocation {
        lease_id: SnapshotLeaseId,
        existing_snapshot: BuildSnapshotId,
        allocated_snapshot: BuildSnapshotId,
    },
    LeaseIdAllocation { error: IdError },
}
```

`SnapshotRegistry::create_snapshot` は、レジストリスナップショットの公開された検証済み
構築経路です。入力からスナップショットや ID を直接作るヘルパーは、`mizar-session` の
テストと内部のレジストリコードだけが使う、crate 内に閉じた未検証の同一性ユーティリティ
です。これらは構築時の不変条件を検証せず、スナップショットをレジストリに挿入しません。
`BuildSnapshot` のフィールドは公開されているので、下流のコードはコピーやテスト用の
切り離されたスナップショットレコードを組み立てられます。ただし、取得・現行判定・リースの対象に
なるレジストリスナップショットは、`create_snapshot` が返したスナップショットだけです。

具象レジストリは、スナップショットをメモリに保持し、ソース／キャッシュ向けのフィンガープリントだけを永続化してよいものとします。公開される識別子は不透明であり、パス、タイムスタンプ、メモリアドレス、タスクローカルなカウンタをエンコードしてはなりません。

## 依存関係

- 内部: `SourceId` をキーとし、`SourceVersion` の同一性と併せて利用される
  座標テーブルのための `source_map`
- 外部: パス正規化、ハッシュ計算、パッケージメタデータ、LSP のドキュメントバージョン型
- 共有: `SnapshotLease.reason` は、本スナップショット／共有リース層で定義する `RetentionReason` を用いる。`retention` モジュールはこれを再定義せず、再エクスポートして再利用する。

このモジュールは、`mizar-build`、`mizar-ir`、`mizar-cache`、`mizar-artifact`、`mizar-diagnostics`、`mizar-lsp` から利用されます。

## データ構造

### スナップショットの同一性

`BuildSnapshotId` は、正準的なスナップショット入力から導出されます。

- 呼び出し側またはソース読み込み層が正規化済みの形で渡すワークスペースルートの同一性
- ソートされたソースバージョンの要約
- 依存アーティファクトの同一性とコンテンツハッシュ
- ロックファイルのハッシュ
- ツールチェインの同一性と関連するスキーマバージョン
- 検証器構成のハッシュ

ビルドセッション ID、スケジューラのタスク ID、実時刻、メモリアドレス、保持リースからは導出しません。
ハッシュ化に用いるソースバージョン要約には、package id、module path、
normalized path、ソースのコンテンツハッシュ、edition を含めます。`SourceId`
とソース由来のメタデータは除外し、アロケータ発行の id や、LSP／セッションローカルな
オーバーレイの詳細が内容同一性に影響しないようにします。

ソーステキストが同一でも、依存アーティファクト、ロックファイルの状態、ツールチェインの同一性、検証器構成のいずれかが異なる 2 つのスナップショットは、異なる `BuildSnapshotId` を受け取らなければなりません。

### ソースバージョン

`SourceVersion` は、キャッシュキー・アーティファクト・診断・LSP オーバーレイが用いる、ソース側の単位です。

記録する内容:

- スナップショット内での安定したソース同一性
- パッケージとモジュールの同一性
- 可能な場合は、ワークスペースまたはパッケージルートからの正規化パス
- ソースのコンテンツハッシュ
- 言語エディション
- 由来。LSP ビルドのオープンバッファバージョンを含む

`SourceId` はスナップショットにスコープされます。公開アーティファクトは、`SourceId` を互換性の保証として露出させるのではなく、モジュールパス・正規化パス・ソースハッシュを通して安定したソース同一性を射影しなければなりません。

`WorkspaceRoot`、`PackageId`、`ModulePath`、`Edition`、`GeneratedSourceKind`
のコンストラクタは、失敗しない境界ラッパーのままにします。上流の
ビルドプラン層とソース読み込み層は、正規化済みで意味的に有効な値を渡すべきです。
一方で `SnapshotRegistry::create_snapshot` は、レジストリスナップショットの最後の
ハッシュ化前のガードです。空の `WorkspaceRoot`、空または空白を含む `PackageId`、
空または言語識別子として不正な `ModulePath` 要素（予約語を含む）、
空の `Edition`、手動構築された `SourceVersion` レコード内の空の生成ソース
メタデータ、重複するソースバージョンの同一性、重複するモジュールパスを、リース割り当て、
スナップショット登録、受理された入力のハッシュ化より前に拒否します。空でないパッケージ名の
正確な綴り規則は、パッケージ管理とモジュール名前空間の仕様が揃うまで、
上流のビルドプラン層へ委ねます。

### スナップショットリース

`SnapshotLease` は、外部の利用側がまだスナップショットを参照している可能性がある間、保持の回収処理がそのスナップショットを生存させるために使う、スナップショット層のハンドルです。
レジストリは `RetentionReason` ごとに生存中のリース数を追跡します。
`create_snapshot` が返す実行中ビルドのリースも、`acquire_lease` で取得した
リースと同じ方法で計上されます。

`acquire_lease` は、リース id を要求する前に未知のスナップショットを拒否します。
既知のスナップショットに対しては、レジストリの mutex の外でリース id を割り当て、
その結果のリースを mutex の下で記録します。リース id の割り当てが失敗した場合、
スナップショットレコード、現行マーク、生存中リース、リース数は変更されません。
`SessionIdAllocator` は一意なアロケータ id を発行する必要がありますが、
レジストリは防御的に、生存中のリース id の重複を状態変更の前に拒否します。
リース id の重複割り当ては、内部的なレジストリ／割り当てのエラーとして報告され、
スナップショットレコード、現行マーク、生存中リース、リース数は変更されません。

リースの理由には次があります。

- 実行中のビルドリクエスト
- watch のベースライン
- 公開された LSP スナップショット
- オープンバッファのオーバーレイ
- 診断インデックス
- 説明リクエスト
- `mizar-ir` におけるフェーズ出力の保持
- 保留中のキャッシュまたはアーティファクトのライタ

リースは現時点では、レジストリの計上の上でスナップショットメタデータを保持します。
retention モジュールは、同じリース状態を、ソースとソースマップの保持へ橋渡しします。
ただし、それ自体ですべての IR 出力を保持するわけではありません。フェーズ出力の保持は `mizar-ir` が所有し、スナップショットへのリースを別途保持してよいものとします。

## アルゴリズム / ロジック

### スナップショットの作成

1. source-loading 層から、読み込み済みの `SourceVersion` レコードを受け取る。
2. ソース同一性、依存アーティファクト参照、ロックファイルメタデータ、ツールチェインメタデータ、構造的に有効なオープンバッファバージョンを検証する。
3. ソースと依存の要約を正準キーでソートする。
4. 正準的なスナップショット入力をハッシュして `BuildSnapshotId` を作る。
5. 不変のスナップショットをレジストリに挿入する。
6. スナップショットと、実行中ビルドのリースを呼び出し側に返す。

ディスク、オープンバッファ、生成ソースの読み込みは source モジュールが実装します。
このレジストリは、その結果として得られたスナップショット入力を記録・検証するだけで、
ディスクやエディタバッファからソーステキストを読み込みません。期待値と実際値を比べる
オープンバッファの失効判定は、リクエストのドキュメントバージョンメタデータを所有する
ソース読み込み層で検証します。
生成ソースの読み込みは、`SourceId` を割り当てる前に、欠落した生成器メタデータを拒否します。
スナップショット作成は、直接渡された `SourceVersion` 入力に対しても同じ検証を繰り返し、
未検証の構築が空の生成ソースメタデータをスナップショット同一性へ流し込まないようにします。
重複するモジュールパスは、ハッシュ化前の最終的なスナップショット全体の検証境界として、常にここで再検査します。

### 鮮度チェック

スナップショットが「現行」であるのは、それがそのリクエスト世代について受理された最新のスナップショットである場合に限ります。watch および LSP ビルドは、診断やエディタ表示のために古いスナップショットを生かしておいてよいものの、古いスナップショットを現行のビルド結果として報告してはなりません。

下流の crate は、ハンドルを消費する前に `BuildSnapshotId` を比較すべきです。ID が異なる場合、利用側はそのハンドルを失効として拒否するか、担当するキャッシュ層でキャッシュ互換性の検証を呼び出さなければなりません。

### 保持と回収

現在のスナップショットレジストリは、リースと現行リクエストの状態を追跡しますが、回収は実装しません。
retention モジュールは、次の条件を満たすスナップショットを回収してよいものとします。

- 診断、LSP、IR、キャッシュ、アーティファクトライタのリースを含め、生存中のリースが参照していない
- それを指名する retention の現行マークがない

回収は、保持の会計メタデータとして記録されたインメモリのソーステキストと
マップリソースを取り除きます。ただし、別の層が安定したアーティファクトやキャッシュの
データとして明示的に保存したものは対象外です。

## エラー処理

`SnapshotError` には次が含まれます。

- 空のワークスペースルート同一性
- 空または空白を含む package id、不正な module path、または空の edition 同一性
- 不正、または正規化できないソースパス
- 1 つのパッケージスナップショット内の重複するモジュールパス
- スナップショットハッシュ化前に見つかった、重複するソースバージョンの同一性
- ビルドプランが参照する、欠落した依存アーティファクトまたはコンテンツフィンガープリント
- 未対応のロックファイルまたはツールチェインのメタデータ
- 失効したオープンバッファバージョン。タスク 11 では、読み込み済みスナップショット入力内の構造的に不正なバージョン値の拒否に限定し、期待値と実際値を比べる失効チェックはソース読み込みが行う
- 直接渡されたスナップショット入力内で、必須の生成器メタデータを欠く生成ソース
- 未知のスナップショット ID
- リース解放の不一致
- 未知のスナップショットリース ID（解放済みのリース ID を含む）
- カスタムまたはレジストリ対応のアロケータにより割り当てられた、重複するリース ID
- リース ID の割り当て失敗

ソースの可読性と UTF-8 検証の診断は、フロントエンドのソース読み込みフローが生成します。このモジュールは、ソース読み込みが有効なソース同一性を生成した後でのみ、結果のソースバージョンを記録します。
`SnapshotError::InvalidSourcePath` は、生のソースパス記述子を受け取る、または再検証するスナップショット構築経路のために予約されています。
現在の公開された `SnapshotRegistry::create_snapshot` API は、すでに正規化済みの
`SourceVersion` レコードを消費し、公開された呼び出し側は不正な `NormalizedPath` 値を構築できません。
そのため、既定のレジストリには、この変種を送出する現時点の公開された観測経路はありません。
将来の直接的なスナップショット構築やレジストリ再検証のフローが、破壊的な変種追加なしに
`SourcePathError` を報告できるよう、公開の非網羅 enum に残します。
現在観測可能なソース読み込みパスの失敗は `source` モジュールが報告し、通常は
`SourceLoadError::InvalidSourcePath` またはより具体的なパスカテゴリを通ります。

## テスト

主なシナリオ:

- 同一の正準入力は、同じ `BuildSnapshotId` を生成する
- ソーステキストの変更はスナップショット ID を変える
- 依存アーティファクトのハッシュの変更はスナップショット ID を変える
- 検証器構成の変更はスナップショット ID を変える
- パス正規化は、重複するソース同一性を防ぐ
- 空のワークスペースルート、空または空白を含む package id、空の module/edition 同一性、識別子として不正または予約語である module-path 要素、生成器メタデータを欠く生成ソースは、スナップショットのハッシュ化またはリース割り当てより前に拒否される
- 構造的に不正なオープンバッファバージョンは拒否される。期待値と実際値を比べる失効判定と、disk/open-buffer の上書き挙動は、ソース読み込みタスクが扱う
- 失効した `BuildSnapshotId` は鮮度チェックで拒否される
- リースは理由ごとに計上され、繰り返し取得したリース id は一意になり、
  アロケータの失敗とリース id の重複割り当てはレジストリ状態を変更せず、
  未知または不一致のリース解放は報告される
- 直接の未検証ヘルパーは公開されておらず、公開フィールドで作った `BuildSnapshot` レコードは、`create_snapshot` が登録するまで切り離されたままである

## 制約と前提

- スナップショットの同一性は、同じ正規化済みワークスペース入力に対して、マシンをまたいで決定的でなければならない。
- 絶対パスは、ローカル診断のために明示的に要求されない限り、公開アーティファクトに含めない。
- `BuildSnapshotId` は同一性と鮮度のトークンであり、証明上の権威ではない。
- スナップショットをまたぐキャッシュの再利用は `mizar-cache` の責務である。このモジュールは、等価性を検証するために必要な入力を提供するにとどまる。
