# モジュール: span_bridge

> 正本は英語です。英語版: [../en/span_bridge.md](../en/span_bridge.md)。

状態: planned。

## 目的

このモジュールは `mizar-lexer` のバイトスパンと `mizar-session` `SourceRange` 値の間の座標橋渡しを所有する。[../../todo.md](../../todo.md) の「Lexer span bridging」に記録されたトップレベルのオープン決定をフロントエンドが解決する唯一の場所である。すなわち `mizar-lexer` は分離を保ち、自前のバイトオフセット `SourceSpan` を保持する一方、フロントエンドがそれらのスパンを session のソース座標へ変換する。

後続の各モジュール（前処理、字句解析、構文解析）は字句解析器相対のバイトスパンを生成する。このモジュールはそれらを source-id でスコープされた `SourceRange` 値へ変換し、`mizar-session` `SourceMapService` にソース単位のマップを登録して、診断と LSP 位置が一貫して解決されるようにする。I/O、トークン化、構文解析、意味的作業は行わない。

## 公開 API

```rust
pub struct SpanBridge { /* registered per-source maps */ }

impl SpanBridge {
    pub fn new(service: Arc<dyn SourceMapService>) -> Self;

    pub fn register_source(
        &self,
        source_id: SourceId,
        line_map: LineMap,
        loading_map: Option<LoadingMap>,
    );

    pub fn register_preprocess_map(
        &self,
        source_id: SourceId,
        preprocess_map: PreprocessMap,
    );

    pub fn loaded_span(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<SourceRange, SpanBridgeError>;

    pub fn lexical_span(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<MappedSourceRange, SpanBridgeError>;
}

pub struct LexerByteSpan {
    pub start: usize,
    pub end: usize,
}
```

`SourceRange`、`MappedSourceRange`、`LineMap`、`LoadingMap`、`PreprocessMap`、`SourceMapService` は `mizar-session` が所有する。`span_bridge` は `mizar-lexer` のバイトスパンをそれらへ適合させる。`loaded_span` は読み込み済みテキスト（Step 1 座標）のスパンを変換し、`lexical_span` はコメント除去済み字句テキスト（Step 2 以降の座標）のスパンを変換して、除去コメントをまたぐスパンに対しては第一の範囲に加えて隣接アンカーを含む `MappedSourceRange` を返す。

## 依存関係

- 内部: `source`、`preprocess`、`lexing`、`parsing` が消費する。フロントエンドで最も低レベルの統制モジュールである。
- 外部: `mizar-session`（`SourceMapService`、`SourceRange`、`MappedSourceRange`、`LineMap`、`LoadingMap`、`PreprocessMap`、`SourceId`）、`mizar-lexer`（`mizar_lexer::source` のバイトオフセットスパン型。この境界でのみ変換される）。

## データ構造

### 変換ステージ

橋渡しは `SourceId` ごとに 3 つの `mizar-session` マップ層を合成する。

| ステージ | 変換元 | 変換先 | 所有者 |
|---|---|---|---|
| lexical → loaded | 字句テキストのバイトオフセット | 読み込み済みテキストのバイトオフセット | `PreprocessMap` |
| loaded → original | 読み込み済みテキストのバイトオフセット | 元の入力のバイトオフセット | `LoadingMap` |
| offset → line/column | 読み込み済みテキストのバイトオフセット | 1 始まりの Unicode 列 | `LineMap` |

`loaded_span` は loading map と line map のみを用いる。`lexical_span` はさらに preprocess map を適用し、ゼロ長境界（例えば内部が除去コメントであった字句範囲）で隣接アンカーの合成を返す。橋渡しは session 側の `LoadingMap` / `PreprocessMap` を字句解析器の `SourceLoadingMap` / `SourcePreprocessMap` から導出する（または `SourceUnit` に既に付随する session `LoadingMap` を再利用する）ので、`SourceId` ごとに正準マップは正確に 1 つである。

### レジストリ

橋渡しは、Step 1-2 の間に登録されたマップを `SourceId` ごとに保持するレジストリを持つ。与えられた `SourceId` に対する登録は冪等である。既に登録済みのソースに対して異なるマップで再登録するのはプログラミングエラーであり、`SpanBridgeError` として表面化する。

## アルゴリズム / ロジック

### 読み込み済みテキストのスパン変換（Step 1 診断）

1. `span` が `source_id` の読み込み済みテキスト内にあることを検証する。
2. 開始／終了の読み込み済みオフセットを `LoadingMap` を通じて元の入力オフセットへ変換する（オフセットを変えた変換がなければ恒等）。
3. `source_id` でスコープされた `SourceRange` を構築して返す。

### 字句テキストのスパン変換（Step 2 以降のトークンと診断）

1. `span` が `source_id` の字句テキスト内にあることを検証する。
2. 字句オフセットを `PreprocessMap` を通じて読み込み済みオフセットへ変換し、スパンが除去コメントをまたぐ場合は第一に加えて隣接アンカーを生成する。
3. `LoadingMap` を通じて元の入力オフセットへ続ける。
4. session `SourceMapService` を用いて `MappedSourceRange`（第一の `SourceRange`、隣接アンカー、読み込み済み→元の `original_input` バイト）を返す。

すべての変換は算術を `mizar-session` に委譲する。このモジュールは正しいマップ層と正しい `SourceId` を選ぶだけである。

## エラー処理

`SpanBridgeError` は session `SourceMapService` が報告する失敗（未知のソース id、ソース／字句テキスト外の範囲、UTF-8 境界上にないオフセット、欠落した loading-map／preprocess-map セグメント、行／列オーバーフロー）に加え、フロントエンドローカルの「ソース未登録」／「マップ登録の衝突」の場合をラップする。橋渡しの失敗は内部不変条件の違反（宣言したソースに属さないスパン）であり、ユーザー診断ではない。統制層はこれを回復可能な字句／構文診断ではなくバグの表面として扱う。

## テスト

主要シナリオ:

- BOM 除去テキスト上の読み込み済みテキストスパンが、loading map を通じて正しい元のバイトオフセットへ変換される。
- 字句テキストスパンが preprocess map と loading map の両方を通じて期待される元の `SourceRange` へ変換される。
- 除去コメントをまたぐ字句スパンが、第一の範囲に加えて隣接アンカーを生む。
- UTF-8 境界上にないオフセットが、暗黙の切り詰めではなく拒否される。
- 登録済みテキスト長の外のスパンが session エラーで拒否される。
- 同じ `SourceId` に異なるマップを 2 つ登録すると、衝突する登録として報告される。

## 制約と前提

- `mizar-lexer` は `mizar-session` から分離を保つ。このモジュールが、字句解析器のバイトスパンを session `SourceRange` 値へ変換する唯一の場所である。
- `SourceId` ごとに正準の line／loading／preprocess マップは正確に 1 つである。
- すべての座標算術は `mizar-session` `SourceMapService` に委譲される。橋渡しはオフセット計算を再実装しない。
- 橋渡しの失敗は内部不変条件の違反であり、ユーザー向け診断ではない。
