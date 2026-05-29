# Module: snapshot

> Canonical language: English. English canonical version: [../en/snapshot.md](../en/snapshot.md).

## Purpose

このモジュールは、`mizar-session` の不変なビルドスナップショットの同一性を定義します。

`BuildSnapshot` は、1 つの batch・watch・LSP ビルドリクエストが観測する、ビルド入力の状態全体を識別します。ソースバージョン、依存アーティファクト、ロックファイルの状態、ツールチェインの同一性、検証器構成を対象とします。下流の crate は `BuildSnapshotId` を用いて、失効したハンドルを拒否し、以前の出力を再利用する前にキャッシュ検証が必要かどうかを判断します。

## Public API

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

pub struct SourceVersion {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub source_hash: Hash,
    pub edition: Edition,
    pub origin: SourceOrigin,
}

pub enum SourceOrigin {
    Disk,
    OpenBuffer { version: LspDocumentVersion },
    Generated { generator: GeneratedSourceKind },
}

pub struct SnapshotLease {
    pub snapshot: BuildSnapshotId,
    pub reason: RetentionReason,
}

pub trait SnapshotRegistry {
    fn create_snapshot(&self, input: SnapshotInput) -> Result<BuildSnapshot, SnapshotError>;
    fn get(&self, id: BuildSnapshotId) -> Option<BuildSnapshotRef>;
    fn acquire_lease(&self, id: BuildSnapshotId, reason: RetentionReason) -> Result<SnapshotLease, SnapshotError>;
    fn release_lease(&self, lease: SnapshotLease);
    fn is_current_for_request(&self, id: BuildSnapshotId, request: BuildRequestId) -> bool;
}
```

具象レジストリは、スナップショットをメモリに保持し、ソース／キャッシュ向けのフィンガープリントだけを永続化してよいものとします。公開される識別子は不透明であり、パス、タイムスタンプ、メモリアドレス、タスクローカルなカウンタをエンコードしてはなりません。

## Dependencies

- Internal: `SourceVersion` に付随するソース座標テーブルのための `source_map`
- External: パス正規化、ハッシュ計算、パッケージメタデータ、LSP のドキュメントバージョン型

このモジュールは、`mizar-build`、`mizar-ir`、`mizar-cache`、`mizar-artifact`、`mizar-diagnostics`、`mizar-lsp` から消費されます。

## Data Structures

### Snapshot Identity

`BuildSnapshotId` は、正準的なスナップショット入力から導出されます。

- 正規化後のワークスペースルートの同一性
- ソートされたソースバージョンの要約
- 依存アーティファクトの同一性とコンテンツハッシュ
- ロックファイルのハッシュ
- ツールチェインの同一性と関連するスキーマバージョン
- 検証器構成のハッシュ

ビルドセッション ID、スケジューラのタスク ID、実時刻、メモリアドレス、保持リースからは導出しません。

ソーステキストが同一でも、依存アーティファクト、ロックファイルの状態、ツールチェインの同一性、検証器構成のいずれかが異なる 2 つのスナップショットは、異なる `BuildSnapshotId` を受け取らなければなりません。

### Source Version

`SourceVersion` は、キャッシュキー・アーティファクト・診断・LSP オーバーレイが用いる、ソース側の単位です。

記録する内容:

- スナップショット内での安定したソース同一性
- パッケージとモジュールの同一性
- 可能な場合は、ワークスペースまたはパッケージルートからの正規化パス
- ソースのコンテンツハッシュ
- 言語エディション
- 由来。LSP ビルドのオープンバッファバージョンを含む

`SourceId` はスナップショットにスコープされます。公開アーティファクトは、`SourceId` を互換性の保証として露出させるのではなく、モジュールパス・正規化パス・ソースハッシュを通して安定したソース同一性を射影しなければなりません。

### Snapshot Lease

`SnapshotLease` は、外部の利用側がまだスナップショットを参照している可能性がある間、そのスナップショットが回収されるのを防ぎます。

リースの理由には次があります。

- 実行中のビルドリクエスト
- watch のベースライン
- 公開された LSP スナップショット
- 診断インデックス
- 説明リクエスト
- `mizar-ir` におけるフェーズ出力の保持
- 保留中のキャッシュまたはアーティファクトのライタ

リースはスナップショットのメタデータとソースマップを保持します。ただし、それ自体ですべての IR 出力を保持するわけではありません。フェーズ出力の保持は `mizar-ir` が所有し、スナップショットへのリースを別途保持してよいものとします。

## Algorithm / Logic

### Snapshot Creation

1. ワークスペース・パッケージ・ソースの各パスを正規化する。
2. リクエストが選択したディスクファイル・オープンバッファ・生成ソースから `SourceVersion` レコードを作る。
3. ソースと依存の要約を正準キーでソートする。
4. 正準的なスナップショット入力をハッシュして `BuildSnapshotId` を作る。
5. 不変のスナップショットをレジストリに挿入する。
6. スナップショットと、実行中ビルドのリースを呼び出し側に返す。

### Freshness Check

スナップショットが「現行」であるのは、それがそのリクエスト世代について受理された最新のスナップショットである場合に限ります。watch および LSP ビルドは、診断やエディタ表示のために古いスナップショットを生かしておいてよいものの、古いスナップショットを現行のビルド結果として報告してはなりません。

下流の crate は、ハンドルを消費する前に `BuildSnapshotId` を比較すべきです。ID が異なる場合、利用側はそのハンドルを失効として拒否するか、担当するキャッシュ層でキャッシュ互換性の検証を呼び出さなければなりません。

### Retention and Collection

レジストリは、次の条件を満たすスナップショットを回収してよいものとします。

- それを参照するリースがない
- それを指名する現行のリクエスト世代がない
- それを指す、保持中のソースマップや診断の説明がない
- `mizar-ir` がそのスナップショットのフェーズ出力参照を解放済みである

回収は、インメモリのソーステキストとマップを取り除きます。ただし、別の層が安定したアーティファクトやキャッシュのデータとして明示的に保存したものは対象外です。

## Error Handling

`SnapshotError` には次が含まれます。

- 不正、または正規化できないソースパス
- 1 つのパッケージスナップショット内の重複するモジュールパス
- ビルドプランが参照する、欠落した依存アーティファクト
- 未対応のロックファイルまたはツールチェインのメタデータ
- 失効したオープンバッファバージョン
- 未知のスナップショット ID
- リース解放の不一致

ソースの可読性と UTF-8 検証の診断は、フロントエンドのソース読み込みフローが生成します。このモジュールは、ソース読み込みが有効なソース同一性を生成した後でのみ、結果のソースバージョンを記録します。

## Tests

主なシナリオ:

- 同一の正準入力は、同じ `BuildSnapshotId` を生成する
- ソーステキストの変更はスナップショット ID を変える
- 依存アーティファクトのハッシュの変更はスナップショット ID を変える
- 検証器構成の変更はスナップショット ID を変える
- パス正規化は、重複するソース同一性を防ぐ
- オープンバッファバージョンは、対象とする LSP リクエストに限ってディスクバージョンに優先する
- 失効した `BuildSnapshotId` は鮮度チェックで拒否される
- リースは、すべての利用側が解放するまでスナップショットを生かし続ける
- 回収されたスナップショットは `get` で取得できない

## Constraints and Assumptions

- スナップショットの同一性は、同じ正規化済みワークスペース入力に対して、マシンをまたいで決定的でなければならない。
- 絶対パスは、ローカル診断のために明示的に要求されない限り、公開アーティファクトに含めない。
- `BuildSnapshotId` は同一性と鮮度のトークンであり、証明上の権威ではない。
- スナップショットをまたぐキャッシュの再利用は `mizar-cache` の責務である。このモジュールは、等価性を検証するために必要な入力を提供するにとどまる。
