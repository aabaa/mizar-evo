# Module: ids

> Canonical language: English. English canonical version: [../en/ids.md](../en/ids.md).

## Purpose

このモジュールは、`mizar-session` が所有する不透明な識別子を定義します。

これらの識別子により、ソース・スナップショット・診断・LSP・キャッシュ・IR 側の各 crate は、パス、メモリアドレス、スケジューラのカウンタ、不安定なフロントエンド内部構造を露出させることなく、同一性について合意できます。各 ID 型は、自身のスコープ、順序付けの規則、シリアライズ境界を文書化します。

## Public API

```rust
pub struct Hash([u8; Self::BYTE_LEN]);

impl Hash {
    pub const BYTE_LEN: usize;
    pub const fn from_bytes(bytes: [u8; Self::BYTE_LEN]) -> Self;
    pub const fn as_bytes(&self) -> &[u8; Self::BYTE_LEN];
}

pub struct BuildSessionId(OpaqueId);
pub struct BuildRequestId(OpaqueId);
pub struct BuildSnapshotId(Hash);
pub struct SourceId(OpaqueId);
pub struct SourceMapId(OpaqueId);
pub struct SnapshotLeaseId(OpaqueId);

pub trait SessionIdAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError>;
    fn next_request_id(&self) -> Result<BuildRequestId, IdError>;
    fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError>;
    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError>;
    fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError>;
}

pub struct InMemorySessionIdAllocator { /* private fields */ }

impl InMemorySessionIdAllocator {
    pub const fn new() -> Self;
}

impl Default for InMemorySessionIdAllocator {
    fn default() -> Self;
}

impl BuildSnapshotId {
    pub const SERIALIZED_LEN: usize;
    pub fn to_published_schema_string(self) -> Result<String, IdError>;
    pub fn from_published_schema_str(serialized: &str) -> Result<Self, IdError>;
}

impl FromStr for BuildSnapshotId {
    type Err = IdError;
    fn from_str(serialized: &str) -> Result<Self, Self::Err>;
}

impl BuildSessionId {
    pub fn to_published_schema_string(self) -> Result<String, IdError>;
}

impl BuildRequestId {
    pub fn to_published_schema_string(self) -> Result<String, IdError>;
}

impl SourceId {
    pub fn to_published_schema_string(self) -> Result<String, IdError>;
}

impl SourceMapId {
    pub fn to_published_schema_string(self) -> Result<String, IdError>;
}

impl SnapshotLeaseId {
    pub fn to_published_schema_string(self) -> Result<String, IdError>;
}

#[non_exhaustive]
pub enum IdError {
    MalformedSerializedId,
    WrongIdDomain,
    UnknownSnapshotRegistry,
    AllocatorOverflow,
    NonPersistableSerialization,
}
```

`BuildSnapshotId` は内容から導出されるフィンガープリントです。その他の ID は不透明なレジストリ上の同一性であり、その値は発行元のレジストリを通してのみ意味を持ちます。
`Hash` は、内容由来 ID、ソースハッシュ、アーティファクトフィンガープリントが用いる固定長 digest wrapper です。`Hash::from_bytes` と `Hash::as_bytes` は、呼び出し側がすでに正準ハッシュバイトを所有している場合、または別の正準エンコードへ安定したバイト列を渡す必要がある場合の低レベル helper として意図的に public です。これら自体は公開シリアライズ形式を定義しません。公開スキーマは `BuildSnapshotId::to_published_schema_string` のように、所有元の ID またはアーティファクト形式を用いなければなりません。

## Dependencies

- Internal: なし
- External: ハッシュ計算と安定したエンコードのユーティリティ

このモジュールは、他のすべての `mizar-session` モジュールと、スナップショットやソースのハンドルを保存する下流の crate から消費されます。

## Data Structures

### Identifier Scope

| Identifier | Scope | Derived From | May Persist? |
|---|---|---|---|
| `BuildSessionId` | コンパイラドライバの 1 回の実行 | アロケータ | no |
| `BuildRequestId` | batch/watch/LSP リクエストの 1 世代 | アロケータ | no |
| `BuildSnapshotId` | ビルド入力の状態全体 | 正準スナップショットハッシュ | yes（来歴として） |
| `SourceId` | 1 つの `BuildSnapshotId` | スナップショットレジストリの割り当て | no |
| `SourceMapId` | 保持される 1 つのソースマップ | アロケータ | no |
| `SnapshotLeaseId` | 生存中の 1 つの保持リース | アロケータ | no |

永続化されるアーティファクトは、アロケータが発行した ID を互換性の保証として扱ってはなりません。スキーマが明示的に許す場合に限り、内容から導出される ID を来歴として記録してよいものとします。

### Ordering

ID は意味的な順序を定義しません。

決定的な順序付けが必要な場合、呼び出し側は正準キーでソートしなければなりません。

- ソースバージョンは、パッケージ ID、モジュールパス、正規化パス、ソースハッシュの順
- 診断は、ソース範囲と診断の同一性の順
- アーティファクトは、モジュールパスと安定したアーティファクト ID の順

アロケータが発行した ID は、インメモリのマップとデバッグ出力に限って順序付けしてよいものとします。

### Serialization

内容から導出される ID は、正準的な小文字 16 進エンコードでシリアライズしてよいものとします。
`BuildSnapshotId` は、公開スキーマ形式として
`mizar-session-build-snapshot-v1:<64 lowercase hex digits>` を使います。この prefix は
シリアライズされた ID のドメインです。デシリアライズ時は、他のドメインと非正準な
16 進表記を拒否しなければなりません。

アロケータが発行した ID は、ローカルなデバッグダンプ、ログ、および可搬性がないと明示された開発用アーティファクトに限ってシリアライズしてよいものとします。公開アーティファクトとキャッシュキーは、代わりに正準的なソース・依存・ツールチェイン・構成の各ハッシュを用いなければなりません。

## Algorithm / Logic

### Content-Derived Id Construction

`BuildSnapshotId` は正準的なスナップショットエンコードから計算します。このエンコードは次を満たさなければなりません。

1. ID 種別を表すドメイン区切り子を含める。
2. 解釈がそれらに依存する場合、関連するスキーマとツールチェインの同一性を含める。
3. 順序のないコレクションは、ハッシュ計算の前にソートする。
4. セッションローカルな ID、タスク ID、メモリアドレス、タイムスタンプ、リース ID を除外する。

### Allocator-Issued Id Construction

アロケータが発行する ID は、所有元のレジストリ内で一意でなければなりません。呼び出し側が値から意味を推測できない限り、単調増加カウンタ、ランダムなノンス、アリーナ索引のいずれであってもかまいません。

アロケータの各メソッドは、新しい一意 ID を発行できない場合に `IdError::AllocatorOverflow` を返します。source/source-map/lease ID のメソッドは `BuildSnapshotId` を受け取り、スナップショットスコープの発行境界が API 上で見えるようにします。

## Error Handling

`IdError` には次が含まれます。

- 内容由来 ID のシリアライズ形式が不正
- ID のドメイン区切り子が誤り
- 未知のスナップショットレジストリ由来の ID
- アロケータのオーバーフロー
- 永続化できない ID を公開スキーマへシリアライズしようとした

整形式の ID を誤ったスナップショットで用いることは失効ハンドルエラーであり、このモジュールではなく `snapshot` または `retention` が報告します。
`IdError::UnknownSnapshotRegistry` は、レジストリを認識する allocator 実装のために予約されています。
この crate が提供するインメモリ allocator はこの variant を emit しません。
`BuildSnapshotId` 引数は、スナップショットスコープの割り当て境界を API 上に保つために受け取ります。
この variant は、`SessionIdAllocator` が public trait であり、将来または下流の allocator が
source/source-map/lease id のスナップショットスコープ割り当てについて、既知のレジストリに対するものかを検証できるよう public に残します。
そのような allocator を別の public API から使う場合、呼び出し側は allocator 呼び出しから直接、または
`SourceLoadError::SourceIdAllocation` や `SnapshotError::LeaseIdAllocation` のような所有元の error surface を通して観測します。
retention は allocator failure を内部的な inconsistent-retention-state error として扱います。

## Tests

主なシナリオ:

- 同一の正準入力に対して、内容由来 ID は決定的である
- ドメイン区切り子を変えると ID が変わる
- 順序のないソース入力は、正準的なソート後に同一のハッシュになる
- セッションローカルな ID はスナップショットハッシュに含まれない
- アロケータが発行する ID は 1 つのレジストリ内で一意である
- 公開スキーマへのシリアライズは、永続化できない ID を拒否する

## Constraints and Assumptions

- ID の値は、利用者向けの名前ではない。
- `SourceId` はスナップショットスコープであり、安定したアーティファクト同一性として用いてはならない。
- `BuildSnapshotId` は同一性と鮮度のトークンであり、証明上の権威ではない。
- デバッグ出力には不透明な ID を含めてよいが、再現可能なアーティファクトはそれらに依存してはならない。
