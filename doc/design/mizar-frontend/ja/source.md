# モジュール: source

> 正本は英語です。英語版: [../en/source.md](../en/source.md)。

状態: 実装済み。

## 目的

このモジュールはフロントエンドパイプラインの Step 1（ソース読み込み）を実装し、前処理および以降のすべての Step が利用する `SourceUnit` を生成する。

`mizar-session` のソース同一性を、フロントエンドローカルな読み込み済みレコードへと橋渡しする。ラップした session ローダーが、ソースのバイト列を読み、UTF-8 を検証し、ソースメタデータを導出し、`LineMap` を構築し、読み込み済みテキストのオフセットから元の入力へ戻す任意の `LoadingMap` を出力する。このモジュールは、それらの読み込み済み値を `SourceUnit` に保持する。コメントの前処理、トークン化、構文解析、インポート解決は行わず、`SourceId` / `SourceVersion` の同一性を自前で割り当てることもしない。

`mizar-session` が、ソース同一性・ソースハッシュ・スナップショット所属を所有する。`mizar-frontend` は `mizar_session::LoadedSource` を利用し、[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Step 1: Load SourceUnit」が定義する `SourceUnit` へ整形する。このモジュールは、`mizar-session` がすでに読み込んだテキストを再ハッシュ・再正規化しない。

## 公開 API

```rust
pub struct SourceUnit {
    pub source_id: SourceId,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub edition: Edition,
    pub file_path: PathBuf,
    pub source_text: Arc<str>,
    pub source_hash: Hash,
    pub line_map: LineMap,
    pub loading_map: Option<LoadingMap>,
    pub origin: SourceOrigin,
    pub generated_anchor: Option<SourceAnchor>,
}

pub struct SourceUnitRequest {
    pub snapshot: BuildSnapshotId,
    pub input: SourceInput,
}

pub trait SourceUnitLoader {
    fn load_source_unit(
        &self,
        request: SourceUnitRequest,
        ids: &dyn SessionIdAllocator,
    ) -> Result<SourceUnit, SourceLoadError>;
}

pub struct FrontendSourceLoader<L: SourceLoader> { /* session loader */ }

impl<L: SourceLoader> FrontendSourceLoader<L> {
    pub fn new(loader: L) -> Self;
}

impl<L: SourceLoader> SourceUnitLoader for FrontendSourceLoader<L> { /* ... */ }

pub fn source_unit_from_loaded(
    loaded: LoadedSource,
    file_path: PathBuf,
) -> SourceUnit;

pub fn register_source_unit(
    bridge: &mut SpanBridge,
    source: &SourceUnit,
) -> Result<(), SpanBridgeError>;
```

`SourceUnit` は、アーキテクチャのインターフェースを反映しつつ、session の同一性メタデータを追加する。`normalized_path`、`edition`、`origin`、`generated_anchor` を保持することで、以降のフェーズ（前処理診断、LSP オーバーレイ、キャッシュキー）が、session レコードを読み直したり再計算したりせずに、正規パス、言語エディション、そしてディスク／オープンバッファ／生成テキストの別を判別できる。

`FrontendSourceLoader` は、任意の `mizar_session::SourceLoader`（たとえば `DiskSourceLoader`）をラップする。リクエストを session ローダーへ転送し、得られた `LoadedSource` を `source_unit_from_loaded` で `SourceUnit` へ射影する。フロントエンドは、自前のパス正規化・ハッシュ・BOM／改行規則を定義しない。これらは `mizar-session` に残る。

`LoadedSource` はファイルシステムパスを保持しない。呼び出し側は、ローカル診断メタデータとして `file_path` を渡す。ディスク／オープンバッファのローダーは、存在する場合はリクエストまたは origin URI からこれを導出し、生成ソースでは `normalized_path` または `generated_anchor` 由来の合成表示パスを使ってよい。この値は、公開ソース同一性やキャッシュキーには含めない。

`register_source_unit` は、source の `LineMap` と任意の `LoadingMap` を可変な `SpanBridge` へ記録する。読み込み自体は bridge 登録と独立なので、テストや呼び出し側は、ソースマップの状態を変えずに `LoadedSource` を射影できる。

## 依存関係

- 内部: `span_bridge`（後の座標変換のために `LineMap` / `LoadingMap` を登録）、`preprocess`（`SourceUnit` を利用）。
- 外部: `mizar-session`（`SourceLoader`、`LoadedSource`、`SourceInput`、`SourceId`、`NormalizedPath`、`Edition`、`LineMap`、`LoadingMap`、`SourceOrigin`、`SourceAnchor`、`SessionIdAllocator`、`BuildSnapshotId`）、session ローダー経由のファイルシステムとパッケージメタデータ。

このモジュールは、統制コーディネータと、単一ファイルの `SourceUnit` を必要とする LSP／ドキュメント利用側が利用する。

## データ構造

### SourceUnit

`SourceUnit` は、1 つの `.miz` ファイルまたは生成ソース断片に対する、ソースに忠実な読み込み済みレコードである。構築済みの `SourceUnit` は、不変なパイプライン入力として扱う。`source_text` は、`mizar-session` が生成した、検証済みかつソース読み込み正規化済みのテキストそのものである。`source_hash`、`line_map`、`loading_map`、`normalized_path`、`edition`、`origin`、`generated_anchor` は session の値であり、再計算せずにコピーする。`file_path` は診断用のローカル表示パスであり、公開される同一性には `normalized_path` を用いる。

`SourceUnit` は、[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md) の「Incrementality」における Step 1 の読み込み済みコンテンツアンカーである。コンテンツの同一性は、パッケージ／モジュール同一性、正規化パス、`source_hash`、エディションからなり、対応する session のソースバージョンサマリと一致する。`origin` とオープンバッファのバージョンは、鮮度・診断のメタデータとして保持するが、公開ソースバージョンのコンテンツ同一性には含めない。

### Loading Map の保持

`loading_map` は、ソース読み込みがオフセットを変えたとき（ディスク／オープンバッファテキストの先頭 BOM 除去、または CRLF→LF 正規化）にのみ `Some` となる。恒等読み込みおよび生成テキストでは `None` である。フロントエンドはマップを編集せず、session のマップを転送するだけなので、診断や LSP アダプタが任意の元入力ビューを必要とするとき、`span_bridge` が `MappedSourceRange.original_input` を通じてソース読み込み入力のバイトオフセットを公開できる。`SourceRange` 値そのものは、読み込み済みテキスト座標のままである。

## アルゴリズム / ロジック

### 単一 SourceUnit の読み込み

1. 対象の `BuildSnapshotId` と `SourceInput`（パッケージ id、モジュールパス、正規化パス、エディション、origin 入力）に対する `SourceUnitRequest` を構築する。
2. ラップした `mizar_session::SourceLoader::load` に委譲する。これが、パス正規化、パッケージルートの強制、バイト読み込み、UTF-8 検証、先頭 BOM 除去、CRLF→LF 正規化、ソースハッシュ計算、`LineMap` 構築、`LoadingMap` 出力を行う。
3. 返された `LoadedSource` を `SourceUnit` へ射影し、`source_id`、`package_id`、`module_path`、`normalized_path`、`edition`、テキスト、ハッシュ、line map、loading map、origin、`generated_anchor` をそのまま保持する。
4. `SourceUnit` を返す。

統制層は、読み込み直後・前処理より前に `register_source_unit` を呼び、読み込んだ `LineMap` / `LoadingMap` を可変な `SpanBridge` レジストリへ登録する。ソース読み込み自体は bridge の状態を変更しない。

フロントエンドはここで、自前のエンコーディング処理を行わない。コード領域の ASCII 検証は前処理に委ねる。このモジュールは、session が検証したエンコーディングと同一性を、前へ運ぶだけである。

## エラー処理

読み込みは `mizar_session::SourceLoadError` をそのまま表面化する。例として、パッケージルート外のソースパス、未対応拡張子、不正な UTF-8、読み取り不能ファイル、古いオープンバッファバージョン、マップ不能なオープンバッファ URI、メタデータのない生成ソース、重複するモジュールパス、未対応の origin、source-id 割り当ての失敗、およびパス正規化系の変種がある。この一覧は網羅的ではなく、ラップした session ローダーは現在の任意の `SourceLoadError` 変種を返しうる。フロントエンドは、統制層でこれらをファイルレベルの診断へ変換する。`mizar-session` がすでに分類している条件に対して、新しいエラーカテゴリを作らない。

読み込み失敗時は `SourceUnit` を生成しない。統制層が診断を報告し、前処理の前にそのファイルのパイプラインを停止する。

## テスト

主要シナリオ:

- ディスクの `LoadedSource` が、同一の `source_id`・`normalized_path`・`edition`・`source_hash`・`line_map`・`loading_map` を持つ `SourceUnit` へ射影される（再計算なし）。
- BOM 除去／CRLF 正規化されたディスクソースは、`Some(loading_map)` を `SourceUnit` へ運ぶ。
- 恒等読み込み（オフセット変更なし）は、`loading_map = None` を運ぶ。
- オープンバッファの `SourceUnit` は、`SourceOrigin::OpenBuffer` と検証済みのドキュメントバージョンを記録する。
- 生成ソースの `SourceUnit` は、`SourceOrigin::Generated` と `generated_anchor` を保持する。
- `register_source_unit` は、`LineMap` / `LoadingMap` を bridge へ記録し、衝突する重複登録は `SpanBridgeError` として報告する。
- session の `SourceLoadError`（不正 UTF-8、ルート外パス）が、再分類されずに伝播する。

## 制約と前提

- このモジュールは、自前でバイトを読み込み・正規化しない。`mizar-session` に委譲し、結果を整形するだけである。
- `source_hash`・`line_map`・`loading_map`・`normalized_path`・`edition` は、フロントエンドが再計算しない。
- `file_path` はローカル診断メタデータであり、公開同一性からは除外される。
- `SourceUnit` は構築後は不変なものとして扱われ、スナップショットリース・LSP ビュー・下流フェーズ出力に保持されうる。
