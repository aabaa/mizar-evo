# Module: source_map

> Canonical language: English. English canonical version: [../en/source_map.md](../en/source_map.md).

## Purpose

このモジュールは、`mizar-session` のソース座標テーブルと範囲変換を定義します。

フロントエンド・診断・LSP・ドキュメント・抽出・アーティファクト・IR サイドテーブルが、可変なソーステキストやフロントエンド内部構造を共有することなく、ソース範囲について合意できるようにします。生ソース範囲、行・列変換、前処理マップ、生成スパン、および合成テキスト向けの劣化した対応付けを扱います。

## Public API

```rust
pub struct LineMap {
    /* private fields */
}

impl LineMap {
    pub fn new(source_id: SourceId, text: &str) -> Self;
    pub fn with_source(source_id: SourceId, text: &str) -> Self;
    pub fn source_id(&self) -> SourceId;
    pub fn text_hash(&self) -> Hash;
    pub fn line_starts(&self) -> &[usize];
    pub fn line_column_for_source(
        &self,
        source_id: SourceId,
        offset: usize,
    ) -> Result<LineColumn, SourceMapError>;
    pub fn line_column_range(&self, range: SourceRange) -> Result<LineColumnRange, SourceMapError>;
    pub fn validate_range(&self, range: SourceRange) -> Result<(), SourceMapError>;
}

pub struct SourceRange {
    pub source_id: SourceId,
    pub start: usize,
    pub end: usize,
}

pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

pub struct LineColumn {
    pub line: u32,
    pub column: u32,
}

pub struct LineColumnRange {
    pub start: LineColumn,
    pub end: LineColumn,
}

pub struct LoadingMap {
    pub source_id: SourceId,
    pub loaded_text_hash: Hash,
    pub loaded_text_len: usize,
    pub origin: LoadingOrigin,
    pub segments: Vec<LoadingMapSegment>,
}

impl LoadingMap {
    pub fn new(
        source_id: SourceId,
        loaded_text: &str,
        origin: LoadingOrigin,
        segments: Vec<LoadingMapSegment>,
    ) -> Self;
    pub fn identity(source_id: SourceId, loaded_text: &str, origin: LoadingOrigin) -> Self;
    pub fn source_id(&self) -> SourceId;
    pub fn loaded_text_hash(&self) -> Hash;
    pub fn loaded_len(&self) -> usize;
    pub fn original_offset_for_loaded(
        &self,
        source_id: SourceId,
        offset: usize,
    ) -> Result<usize, SourceMapError>;
    pub fn original_range_for_loaded(
        &self,
        source_id: SourceId,
        loaded: TextRange,
    ) -> Result<LoadedToOriginalRange, SourceMapError>;
}

pub enum LoadingOrigin {
    DiskBytes { normalized_path: NormalizedPath },
    OpenBufferText { uri: DocumentUri, version: LspDocumentVersion },
    Generated,
}

pub enum LoadingMapSegment {
    Original {
        loaded: TextRange,
        original: TextRange,
    },
    RemovedLeadingBom {
        original: TextRange,
    },
    NormalizedNewline {
        loaded: TextRange,
        original: TextRange,
    },
}

pub struct LoadedToOriginalRange {
    pub original: TextRange,
    pub kind: LoadedToOriginalRangeKind,
}

pub enum LoadedToOriginalRangeKind {
    Exact,
    Degraded,
}

pub struct PreprocessMap {
    pub source_id: SourceId,
    pub lexical_text_hash: Hash,
    pub segments: Vec<PreprocessSegment>,
}

pub enum PreprocessSegment {
    Original {
        lexical: TextRange,
        source: SourceRange,
    },
    RemovedComment {
        source: SourceRange,
        kind: CommentKind,
    },
    SyntheticWhitespace {
        lexical: TextRange,
        anchor: SourceAnchor,
    },
}

pub enum SourceAnchor {
    Range(SourceRange),
    Point { source_id: SourceId, offset: usize },
    Generated { origin: GeneratedSpanOrigin },
}

pub trait SourceMapService {
    fn line_column(&self, range: SourceRange) -> Result<(LineColumn, LineColumn), SourceMapError>;
    fn original_range_for_loaded(&self, source_id: SourceId, loaded: TextRange) -> Result<MappedSourceRange, SourceMapError>;
    fn source_range_for_lexical(&self, source_id: SourceId, lexical: TextRange) -> Result<MappedSourceRange, SourceMapError>;
    fn attach_generated_span(&self, origin: GeneratedSpanOrigin) -> SourceAnchor;
    fn validate_range(&self, range: SourceRange) -> Result<(), SourceMapError>;
}
```

オフセットは、ソース読み込みの正規化後の検証済み UTF-8 `LoadedSource.text` へのバイトオフセットです。利用者向けの列は、フロントエンドアーキテクチャが定義する Unicode スカラー列の規則を用いるため、利用側が場当たり的に計算するのではなく、`LineMap` を通して変換しなければなりません。

`LineColumn` は意図的に `usize` ではなく `u32` を用います。これらは生のメモリ索引ではなく、表示およびプロトコルに近い座標であり、範囲を有界に保つことで診断・アーティファクトメタデータ・LSP アダプタと揃います。読み込み済みソースが `u32::MAX` を超える利用者向け行数を持つ場合、または 1 行が `u32::MAX` を超える Unicode スカラー列を持つ場合、`LineMap` は飽和・ラップ・暗黙の縮小を行わず、`SourceMapError::LineColumnOverflow` を返します。LSP の位置は `u32` のままとし、UTF-16 の列については `mizar-lsp` ブリッジが独自に明示的なチェック付き縮小を行います。

## Dependencies

- Internal: `SourceId` とソースバージョンの同一性のための `snapshot`
- External: ハッシュ計算、UTF-8 テキストユーティリティ、LSP の範囲型

このモジュールは、フロントエンドの各フェーズ、`mizar-ir` のサイドテーブル、`mizar-diagnostics`、`mizar-lsp`、`mizar-artifact`、`mizar-doc`、`mizar-extract` から消費されます。

## Data Structures

### Line Map

`LineMap` は、`SourceVersion` が表すソーステキストそのものについて、ソース同一性・テキストハッシュ・各行の開始位置を記録します。

構築後は不変です。保持されたソース ID・テキストハッシュ・行開始位置は、可変なフィールドアクセスではなく accessor を通して参照します。利用側は、オフセットを利用者向けの行・列値に変換する前に、`source_id` が報告対象のスナップショットに属することを検証しなければなりません。

ディスクのソース読み込みが先頭の UTF-8 BOM を除去した場合、`LineMap` のバイトオフセット `0` は、元ファイルにおけるその BOM の直後の最初のバイトに当たります。生ファイルのバイト位置は、`SourceRange` や字句解析器のスパン座標を変更するのではなく、`LoadingMap` を通して復元します。

### Source Range

`SourceRange` は半開区間です（`start <= offset < end`）。

範囲は次を満たさなければなりません。

- 1 つの `SourceId` を参照する
- UTF-8 スカラー境界に整列したバイトオフセットを用いる
- ソーステキストの長さの内側にとどまる
- 長さ 0 の範囲は、挿入点と合成アンカーに限って保つ

### Loading Map

`LoadingMap` は、正規化された `LoadedSource.text` を、BOM 除去や改行正規化の前のソース読み込み入力と関連付けます。ディスクソースでは、`original` の範囲は UTF-8 検証後の元ファイルバイトへのバイトオフセットです。オープンバッファでは、`original` の範囲はエディタ提供 UTF-8 テキストへのバイトオフセットであり、その後 `mizar-lsp` ブリッジがプロトコルの UTF-16 位置へ変換します。生成ソースはここで生成由来を保持でき、生成テキストをソース位置に結び付けるアンカーは後続のアンカー対応で導入します。

先頭の UTF-8 BOM が除去された場合、この対応付けは元バイト範囲 `[0, 3)` に対する `RemovedLeadingBom` セグメントを記録し、最初の `Original` 読み込みセグメントは読み込みオフセット `0`・元バイトオフセット `3` から始まります。ソース読み込みが `LoadingMap` を省略してよいのは、読み込み済みテキストがソース読み込み入力とオフセットの上で同一である場合に限ります。オフセット上同一のテキストに対してマップを保持する場合、`LoadingMap::identity` は 1 つの `Original` セグメントでその関係を表します。

`LoadingMap::new` は、呼び出し側が渡したセグメントを完全な構造検証なしで記録します。これらのマップを構築するソースローダーは、セグメントの不変条件を保たなければなりません。読み込み範囲は昇順かつ非重複であること、`Original` セグメントでは読み込み/元バイト長が等しいこと、`NormalizedNewline` セグメントは CRLF→LF 正規化を表し通常は読み込み長 1・元長 2 であること、`RemovedLeadingBom` は先頭 UTF-8 BOM の元範囲 `[0, 3)` のみを表すこと、対応付ける読み込みバイト範囲がセグメントで覆われていることです。完全なソース読み込みを実装する task 14-15 では、これらの不変条件に対する構築ヘルパーまたはローダー側チェックを追加するべきです。

### Preprocess Map

`PreprocessMap` は、字句解析器が消費する字句テキストを元ソースと関連付けます。

`Original` セグメントは字句範囲をソース範囲へ戻します。`RemovedComment` セグメントは、コメントが字句入力から消えていても、通常コメントとドキュメントコメントの位置を保ちます。`SyntheticWhitespace` セグメントは、コメント除去やリカバリの後にトークンの分離を保つために挿入されたテキストを表します。

この対応付けについては、フロントエンドがスナップショット保持とサービスアクセスを所有します。保持されるセッションの `PreprocessMap` を構築する際、字句解析器のヘルパーが生成する軽量な前処理マップを再利用またはミラーしてよいものとします。後続のフェーズは、診断や構文ノードを元のソース位置に結び付けるためにこれを消費します。

### Generated Spans

生成スパンは、コンパイラが生成した要素に対応する正確なソース範囲がない場合に用います。たとえば次のようなものです。

- 暗黙の義務
- 挿入された型強制や `qua` ノード
- 生成された証明の再生ステップ
- 複数の入力から導出されたドキュメントまたは抽出のレコード

生成スパンは、利用可能な最善のソースアンカーと理由を指す由来を含めなければなりません。診断は生成スパンを副次的な情報として表示してよいものの、主要な診断は、利用できる場合は元のソース範囲を優先すべきです。

## Algorithm / Logic

### Line/Column Conversion

1. `LineMap` のソーステキスト長に対して範囲を検証する。
2. `line_starts` を二分探索して、開始行と終了行を特定する。
3. 各行の開始位置から各オフセットまでの Unicode スカラー値を数える。
4. 診断・アーティファクト・CLI 整形のために、1 始まりの行と 1 始まりの列を返す。

LSP 変換は、プロトコルの UTF-16 位置規則を `mizar-lsp` ブリッジで適用しなければならず、このモジュール内では行いません。このモジュールはソースに対して安定した座標を公開します。

### Loaded-to-Original Mapping

1. `SourceId` に対応する `LoadingMap` が存在する場合は、それを用いる。
2. `LoadingMap` がない場合、読み込み済みテキストのオフセットを、ソース読み込み入力への恒等オフセットとして扱う。
3. 読み込み範囲が正規化セグメントをまたぐ場合、元バイト範囲を包む劣化した `LoadedToOriginalRange` を返す。後続の `SourceMapService` の複合返り値型は、前処理マップや生成スパンが利用可能になった時点で副次アンカーを付与できる。
4. オープンバッファでは、エディタテキストのバイトオフセットを返す。最終的な UTF-16 変換は LSP ブリッジが行う。

### Lexical-to-Source Mapping

1. 字句範囲と交差する前処理セグメントをすべて見つける。
2. 範囲が 1 つの連続した読み込みソース範囲に対応付けられる場合、診断がソース読み込み入力の座標を必要とするときは、その範囲を loaded-to-original 対応付けを通して対応付ける。
3. 除去済みまたは合成のセグメントをまたぐ場合、主要・副次のアンカーを持つ複合対応付けを返す。
4. 元ソースが存在しない場合、生成アンカーを返す。

複合対応付けは、診断・ドキュメントへの結び付け・説明メタデータに許可されます。キャッシュキーとアーティファクトハッシュは、ソースハッシュと安定した ID を用いなければならず、複合対応付けをシリアライズした整形済み形式を用いてはなりません。

### Source Map Retention

ソースマップは、スナップショットリース・診断インデックス・LSP 公開・IR サイドテーブルのいずれかが参照している間、所有元のスナップショットとともに保持されます。所有元のスナップショットが回収された後は破棄してよいものとします。

## Error Handling

`SourceMapError` には次が含まれます。

- 未知のソース ID
- ソーステキストの外側にある範囲
- UTF-8 境界に整列していないオフセット
- `u32` で表現できない行・列座標
- 読み込み済みテキストの外側にある読み込み範囲
- 前処理済みテキストの外側にある字句範囲
- 欠落した loading map セグメント
- 欠落した前処理セグメント
- 由来の理由を欠く生成スパン

不正な利用者ソースは、フロントエンドの診断が報告します。ソースマップエラーは、明示的に失効した LSP リクエストに由来する場合を除き、コンパイラのバグまたは失効ハンドルを示します。

## Tests

主なシナリオ:

- 行マップがバイトオフセットを Unicode スカラー列に変換する
- 行マップは、表現できない行・列値を暗黙に縮小せず、オーバーフローを報告する
- BOM 付きディスクファイルの行マップは、除去された BOM の後の読み込みテキストオフセット `0` から始まる
- 先頭 BOM が除去された場合、`LoadingMap` は読み込みテキストオフセット `0` を元ファイルバイトオフセット `3` へ対応付ける
- オープンバッファの `LoadingMap` は、LSP の UTF-16 変換の前に、読み込みテキストオフセットをエディタ提供テキストのバイトオフセットへ戻す
- CRLF と LF の入力は、ソース読み込み規則に従って決定的な行開始位置を生成する
- 除去されたコメントは、保たれたコメントのソース範囲へ対応付けられる
- コメント除去をまたぐ字句範囲は、複合対応付けを生成する
- 合成空白は、主要な利用者ソース範囲にはならない
- 生成アンカーは、その由来の理由を保つ
- 不正なバイトオフセットとソースをまたぐ範囲は拒否される
- LSP の UTF-16 変換はこのモジュールの外側にとどまる
- LSP の UTF-16 縮小は明示的であり、チェックなしのキャストを用いず、オーバーフローを報告する

## Constraints and Assumptions

- ソースマップは内部のコンパイラデータであり、この crate からは安定したスキーマとして公開されない。
- 公開アーティファクトには射影されたソース範囲を含めてよいが、前処理マップ全体は含めない。
- ソース範囲の変換は決定的でなければならず、スケジューリング順序に依存してはならない。
- このモジュールはファイルを直接読み込んではならない。ソース読み込みとスナップショット生成が提供するソーステキストと同一性をもとに動作する。
