# モジュール: source

> 正本は英語です。英語版: [../en/source.md](../en/source.md)。

## 目的

このモジュールは、`SourceVersion` 値を作るためのソースレコードを定義します。

正規化されたソースパス、検証済みのソーステキストハンドル、ソースハッシュ、および LSP リクエストが提供するオープンバッファのソーステキストを所有します。スナップショット用のソース同一性を準備しますが、コメントの前処理、トークン化、構文解析、インポート解決は行いません。

## 公開 API

```rust
pub struct NormalizedPath(String);

impl NormalizedPath {
    pub fn as_str(&self) -> &str;
}

pub struct SourceInput {
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub edition: Edition,
    pub origin: SourceOriginInput,
}

pub struct DiskSourceLoader { /* package root */ }

impl DiskSourceLoader {
    pub fn new(package_root: impl Into<PathBuf>) -> Self;
    pub fn package_root(&self) -> &Path;
}

#[non_exhaustive]
pub enum SourceOriginInput {
    Disk { path: PathBuf },
    OpenBuffer {
        uri: DocumentUri,
        expected_version: LspDocumentVersion,
        actual_version: LspDocumentVersion,
        text: Arc<str>,
    },
    Generated { generator: GeneratedSourceKind, text: Arc<str>, anchor: Option<SourceAnchor> },
}

pub struct LoadedSource {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub text: Arc<str>,
    pub source_hash: Hash,
    pub edition: Edition,
    pub origin: SourceOrigin,
    pub line_map: LineMap,
    pub loading_map: Option<LoadingMap>,
    pub generated_anchor: Option<SourceAnchor>,
}

pub trait SourceLoader {
    fn load(
        &self,
        snapshot: BuildSnapshotId,
        input: SourceInput,
        ids: &dyn SessionIdAllocator,
    ) -> Result<LoadedSource, SourceLoadError>;
    fn normalize_path(&self, package_root: &Path, path: &Path) -> Result<NormalizedPath, SourceLoadError>;
    fn hash_text(&self, text: &str) -> Hash;
}

pub fn normalize_path(package_root: &Path, path: &Path) -> Result<NormalizedPath, SourceLoadError>;
pub fn hash_text(text: &str) -> Hash;
pub fn normalize_source_path(package_root: &Path, path: &Path) -> Result<NormalizedPath, SourcePathError>;

#[non_exhaustive]
pub enum SourcePathError {
    UnsupportedPathEncoding { path: PathBuf },
    PackageRootUnavailable { path: PathBuf, kind: io::ErrorKind },
    SourcePathUnavailable { path: PathBuf, kind: io::ErrorKind },
    OutsidePackageRoot { package_root: PathBuf, path: PathBuf },
    NonCanonicalPathAlias { requested: PathBuf, canonical: PathBuf },
    NonCanonicalPathSpelling { requested: PathBuf, canonical: PathBuf },
    InvalidNamespaceComponent { component: String },
    MissingSourceRoot { path: PathBuf },
    UnsupportedExtension { path: PathBuf },
}

#[non_exhaustive]
pub enum SourceLoadError {
    SourcePathOutsidePackageRoot { package_root: PathBuf, path: PathBuf },
    UnsupportedFileExtension { path: PathBuf },
    InvalidUtf8 { path: Option<PathBuf> },
    UnreadableSourceFile { path: PathBuf, kind: io::ErrorKind },
    DuplicateModulePath { package_id: PackageId, module_path: ModulePath },
    StaleLspDocumentVersion { expected: LspDocumentVersion, actual: LspDocumentVersion },
    UnmappedOpenBufferUri { uri: DocumentUri },
    GeneratedSourceWithoutMetadata { module_path: ModulePath },
    SourceIdAllocation { error: IdError },
    UnsupportedSourceOrigin { origin: SourceOriginKind },
    InvalidSourcePath { error: SourcePathError },
}

#[non_exhaustive]
pub enum SourceOriginKind {
    Disk,
    OpenBuffer,
    Generated,
}
```

`LoadedSource` は不変のソーステキストハンドルです。スナップショット生成は読み込み済みソースを消費し、その `SourceVersion` の要約を記録します。
`load` は対象の `BuildSnapshotId` を受け取り、`SessionIdAllocator` からスナップショットスコープの `SourceId` を発行できるようにします。
`LoadedSource.origin` は snapshot モジュールの `SourceOrigin` を使います。source モジュールは、読み込み済みレコード用の由来の列挙を重複定義しません。
`SourceLoader` の補助メソッドは、公開ヘルパーの `normalize_path` と `hash_text` に委譲します。`normalize_path` は `normalize_source_path` を再利用し、`hash_text` は正規化済みテキスト内容だけをハッシュ化します。
`DiskSourceLoader` は、パスと URI の正規化に用いるパッケージルートを所有します。ディスクファイル、`file://` ドキュメント URI から対応付けられるオープンバッファオーバーレイ、生成ソースフラグメントに対して `SourceLoader` を実装します。
`NormalizedPath::as_str` は、スナップショット同一性、診断、下流メタデータが、可変なパス表現を露出せずに正準パス表記を読めるよう、意図的に公開されています。`DocumentUri` と `LspDocumentVersion` は、ソースマップ座標型とともに定義される crate レベルの公開エイリアスであり、ここではオープンバッファのソース読み込みに用います。
`PackageId`、`ModulePath`、`Edition` の値は、上流のパッケージ計画とモジュール解決から渡されます。単一ソースの読み込みは、ソースパス、テキスト、オープンバッファの鮮度、生成ソースのメタデータを検証しつつ、それらの同一性の値を変更せず保持します。将来のソース読み込みアグリゲータは、ビルドプラン全体を見て重複するモジュールパスを `SourceLoadError::DuplicateModulePath` として拒否できますが、単一の `SourceLoader::load` 呼び出しには、そのエラーを送出するだけの文脈がありません。
`SnapshotRegistry::create_snapshot` は、レジストリスナップショットの最後のハッシュ化前検証境界のままです。

## 依存関係

- 内部: `ids`, `source_map`, `snapshot`
- 外部: ファイルシステム、パッケージメタデータ、ハッシュ計算、UTF-8 検証、LSP のドキュメント同期型

このモジュールは、スナップショット生成、フロントエンドのソース読み込み、LSP のオープンバッファオーバーレイ構築、診断、ドキュメント／抽出のソース利用側から利用されます。

## データ構造

### 正規化パス

`NormalizedPath` は、区切り子が正規化され、`.` や `..` の要素を含まない、ワークスペース相対またはパッケージ相対のパスです。

次を含んではなりません。

- 絶対パスのプレフィックス
- local-only と明示されない、シンボリックリンクを展開したホスト固有のパス
- プラットフォーム固有の区切り子の差異
- パッケージ管理下のソースパスに対する、正準でない大文字小文字の変種
- 予約語を含む、言語識別子ではない名前空間要素

ローカル診断は、表示用の絶対パスを別に保持してよいものとします。公開アーティファクトは正規化パスを用います。

### 読み込み済みソース

`LoadedSource` は、検証済み UTF-8 テキストと、そのテキストそのものに対する `LineMap` を含みます。ディスク入力とオープンバッファ入力では、ソース読み込みの正規化後のテキストを格納します。生成入力では、受理した生成テキストを 1 バイトもたがえず格納し、ソース読み込みによる BOM や改行の正規化は行いません。構築後は不変であり、スナップショットリース、LSP スナップショット、診断インデックス、ソースマップハンドルによって保持されることがあります。

`source_hash` は `LoadedSource.text` から計算されます。ディスク入力とオープンバッファ入力では、これは UTF-8 検証と、先頭 BOM の除去や改行の正規化といったソース読み込みの正規化を経た後のテキストです。オープンバッファの場合は、ディスク上のファイルではなく、正規化されたエディタ提供テキストが対象です。生成入力では、生成器から受理した正確な生成テキストのハッシュです。パッケージ化や診断のためにバイト単位で正確な来歴が必要な場合は、`source_hash` を再定義せず、来歴メタデータまたは別個の生コンテンツハッシュを用います。

`loading_map` は、`LoadedSource.text` が作られる前にソース読み込みがオフセットを変更した場合に存在します。これは正規化された読み込み済みテキストの範囲をソース読み込みの入力へ対応付けるもので、ディスクソースでは元ファイルのバイトオフセット、オープンバッファではエディタ提供テキストのバイトオフセットを指します。生成入力は `SourceOriginInput` 上に任意の `SourceAnchor` を持ちます。`LoadedSource.generated_anchor` はそのアンカーを保持します。生成読み込みはバイトオフセット変換を行わず、`LoadingMap` を出力しません。生成ソースの位置は、代わりに任意のアンカーと生成スパンのメタデータから復元します。ソース読み込みの変換がオフセットを変えなかった場合、この対応付けは恒等であり、省略してよいものとします。

### ソースの由来

`SourceOrigin` は、テキストの由来を区別します。

- `Disk`: パッケージツリーから読み込んだソースファイル
- `OpenBuffer`: 未保存のエディタテキスト
- `Generated`: コンパイラが生成した、またはツールが提供したソースフラグメント

オープンバッファのソースは、対象とする LSP リクエストまたは watch 世代に限ってディスクソースを上書きできます。通常のアーティファクト出力には書き込まれません。

## アルゴリズム / ロジック

### ディスクソースの読み込み

1. パッケージルートからの相対パスへ正規化する。
2. パッケージルートの外側、または必須の `src/` ソースツリーの外側にあるパスを拒否する。
   パッケージルート内だが `src/` の外側にあるパスは、パッケージルート境界の
   カテゴリではなく、`SourcePathError::MissingSourceRoot` を保持する
   `SourceLoadError::InvalidSourcePath` として報告する。
3. ディスクからバイト列を読み込む。
4. UTF-8 を検証する。不正なバイトは行マップ構築の前に拒否し、損失のあるデコードで `U+FFFD` に置き換えてはならない。
5. 検証済みテキストが UTF-8 BOM シグネチャで始まる場合、先頭の `U+FEFF` を除去する。
6. 各 CRLF を 1 つの LF に置き換えて、ソース読み込みの改行を正規化する。単独の `\r` はプラットフォーム改行として扱わず、前処理へ届いた場合は不正な字句境界入力のままとする。
7. BOM の除去または改行の正規化がオフセットを変更した場合、正規化済み読み込みテキストのオフセットから元ファイルのバイトオフセットへの `LoadingMap` を記録する。
8. `LoadedSource.text` からソースハッシュを計算する。
9. `LoadedSource.text` 上に `LineMap` を構築する。
10. `LoadedSource` を返す。

エンコードシグネチャとして扱うのは先頭の UTF-8 BOM だけです。それ以外の位置にある `U+FEFF` は読み込み済みテキストに保持され、コード中に現れた場合は不正な字句境界文字のままです。

コード領域の ASCII 検証は前処理の責務です。このモジュールは、テキストのエンコードとソースの同一性のみを検証します。

### オープンバッファソースの読み込み

1. リクエストが期待する LSP ドキュメントバージョンと、LSP ブリッジが提供した実際のエディタバッファバージョンを比較し、失効したバージョンや構造的に不正なバージョンを `SourceId` 発行前に拒否する。
2. ドキュメント URI をパッケージのソースパスへ正規化する。`file://` ではない
   URI、デコード不能なパーセントエンコーディングを含む URI、パッケージルート外の
   file URI など、そもそもパッケージ相対パスになれない URI は
   `SourceLoadError::UnmappedOpenBufferUri` として報告する。
3. そのリクエストでは、エディタ提供テキストを正本として用いる。
4. BOM 付きディスクファイルのエディタ表示がディスクのソース読み込みと一致するように、パッケージが記述したオープンバッファテキストから先頭の `U+FEFF` を 1 つ除去する。
5. 各 CRLF を 1 つの LF に置き換えて、ソース読み込みの改行を正規化する。単独の `\r` は、フロントエンドや字句解析器の診断が一貫して拒否できるよう保持する。
6. 除去または改行の正規化がオフセットを変更した場合、正規化済み読み込みテキストのオフセットからエディタ提供テキストのバイトオフセットへの `LoadingMap` を記録する。
7. `LoadedSource.text` からソースハッシュと `LineMap` を計算する。
8. 検証済みの実際のドキュメントバージョン付きで、由来を `OpenBuffer` として記録する。

オープンバッファ URI がパッケージ相対パスへ対応付けられた後のソースパス検証は、
ディスク読み込みと同じ `normalize_source_path` の分類を使います。たとえば、
パッケージ `src/` 配下にある `.miz` 以外のファイルは
`SourceLoadError::UnsupportedFileExtension` として報告し、`src/` ルートの欠落、
非正準パス、不正な名前空間要素は
`SourceLoadError::InvalidSourcePath` を通じて報告します。

オープンバッファのテキストは、最後に検証されたアーティファクトより新しいことがあります。利用側は、アーティファクトのデータを暗黙のうちに最新として扱うのではなく、鮮度メタデータを引き回さなければなりません。LSP の診断と編集は、エディタドキュメントに対してプロトコルの UTF-16 位置規則を適用する前に、`LoadedSource.text` のオフセットを `loading_map` を通して変換しなければなりません。

### 生成ソースの読み込み

生成ソースには、空でない生成器の種別と、可能な場合は元ソースへのアンカーが必要です。読み込みは空の生成器メタデータを `SourceId` 割り当て前に拒否し、受理した生成器メタデータを `LoadedSource.origin` に保持し、任意のアンカーを `LoadedSource.generated_anchor` に保持します。

生成ソーステキストは API に `Arc<str>` として入るため、すでに UTF-8 です。ソースローダーはそのテキストを `LoadedSource.text` として 1 バイトもたがえず保持します。先頭の `U+FEFF` はエンコードシグネチャとして扱わず、CRLF ペアは LF に変換せず、単独の `\r` も変更しません。`source_hash` と `LineMap` は、この正確な生成テキストに対して計算されます。生成読み込みはソース読み込みのオフセット変換を行わないため、`loading_map` は `None` です。パッケージ記述ソースと同じ正規化を望む生成器は、`SourceOriginInput::Generated` を構築する前に自分の出力を正規化し、それでも生成器メタデータと必要なソースアンカーを記録しなければなりません。

生成ソースのテキストは診断・ドキュメント・抽出に用いてよいものの、パッケージが記述した `.miz` ソースと取り違えてはなりません。

## エラー処理

`SourceLoadError` には次が含まれます。

- パッケージルートの外側にあるソースパス
- パッケージルート内だが必須の `src/` ソースツリーの外側にあるソースパス。
  `SourcePathError::MissingSourceRoot` を保持する `InvalidSourcePath` として報告する
- 未対応のファイル拡張子
- 不正な UTF-8
- 読み取れないソースファイル
- 将来のソース読み込みアグリゲータがビルドプラン全体から見つける重複するモジュールパス。単一ソースの `DiskSourceLoader::load` はこの変種を送出しない
- 失効した、または構造的に不正な LSP ドキュメントバージョン
- パッケージ相対パスになれないオープンバッファ URI
- 必須の生成器メタデータを欠く生成ソース
- `SessionIdAllocator` による source id 発行失敗
- `SourceOriginInput` の一部だけを意図的に実装する具象ローダーに対する未対応のソース由来
- 明示的なパスカテゴリに収まらない `normalize_source_path` 由来のその他の正規化エラー

利用者向けの読み取り失敗・エンコード失敗は、フロントエンドやビルドの診断として発行されます。内部の呼び出し側は構造化エラーも受け取るため、スナップショット生成はビルドリクエストが致命的か回復可能かを判断できます。
アロケータの失敗は元の `IdError` を保持します。特に、アロケータのオーバーフローは暗黙にソース同一性へ変換されません。
URI からパスへの対応付けに成功した後のソースパス検証では、ディスク読み込みと
オープンバッファ読み込みが同じエラーカテゴリを共有します。
`UnmappedOpenBufferUri` は、`file://` でないスキーム、デコード不能なパーセント
エンコーディング、パッケージルート外の file URI など、URI 対応付けそのものの失敗に
限定します。
予約済みに見えるソース読み込み変種のトレーサビリティは次のとおりです。

| 変種 | 現在の分類 | 公開された観測経路 |
|---|---|---|
| `SourceLoadError::InvalidSourcePath` | 公開されたソース読み込みパス正規化の経路 | `DiskSourceLoader::load` はディスクソースと対応付け済みのオープンバッファソースについて、`SourceLoader::normalize_path` と公開ヘルパーの `normalize_path` は公開された正規化経路として、`src/` ルートの欠落、非正準のエイリアス、非正準の綴り、不正な名前空間要素などの `normalize_source_path` の失敗をこの変種へ写します。 |
| `SourceLoadError::UnsupportedSourceOrigin` | カスタムローダー専用 | 既定の `DiskSourceLoader` は現在の `SourceOriginInput` 変種（`Disk`、`OpenBuffer`、`Generated`）をすべてサポートし、この変種を送出しません。`SourceLoader` が公開トレイトであり、下流の具象ローダーがソース由来の一部だけを意図的に実装できるよう、公開のまま残します。 |

## テスト

主なシナリオ:

- 同一テキストのディスクソースとオープンバッファソースは、同じソースハッシュを生成するが由来は異なる
- オープンバッファソースは、期待 version と実際のドキュメントバージョンが一致する場合に限ってディスクテキストを上書きする
- 不正な UTF-8 は行マップ構築の前に拒否され、損失のあるデコードで置換文字に変換されない
- 先頭の UTF-8 BOM は受理され、行マップ構築の前に除去される
- 先頭以外の `U+FEFF` は、ソース読み込みで除去されない
- オープンバッファの BOM 除去と改行正規化は、エディタ提供テキストのオフセットへ戻す loading map を保つ
- パス正規化は、パッケージルートの外側にあるパスと、パッケージルート内だが `src/` の外側にあるパスを拒否する
- CRLF と LF の扱いが `LineMap` の期待と一致する
- 生成ソース入力は、空でない生成器メタデータと任意のアンカーを持ち、読み込み済み生成ソースは `SourceOrigin` に生成器メタデータを、`LoadedSource.generated_anchor` に任意のアンカーを保つ
- 先頭 `U+FEFF` と CRLF を含む生成ソーステキストは 1 バイトもたがえず保持され、その正確なテキストとしてハッシュ化され、`LoadingMap` を出力しない
- ソースハッシュは、絶対パスやドキュメントバージョンを含まない

## 制約と前提

- このモジュールは、ソーステキストの構文解析・前処理・トークン化を行わない。
- ソースハッシュはコンテンツハッシュであり、それ自体が鮮度の判断ではない。
- 絶対パスはローカル診断用のメタデータであり、公開されるソース同一性からは除外される。
- ソーステキストは、スナップショット・ソースマップ・診断・LSP ビュー・下流の利用側がリースを保持している間だけ保持される。
