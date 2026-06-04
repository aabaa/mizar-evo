# モジュール: span_bridge

> 正本は英語です。英語版: [../en/span_bridge.md](../en/span_bridge.md)。

状態: planned。

## 目的

このモジュールは `mizar-lexer` のバイトスパンと `mizar-session` `SourceRange` 値の間の座標橋渡しを所有する。[../../todo.md](../../todo.md) の「Resolved And Open Decisions」に記録されたトップレベルの解決済み決定を実装する唯一の場所である。すなわち `mizar-lexer` は分離を保ち、自前のバイトオフセット `SourceSpan` を保持する一方、フロントエンドがそれらのスパンを session のソース座標へ変換する。

後続の各モジュール（前処理、字句解析、構文解析）は字句解析器相対のバイトスパンを生成する。このモジュールはそれらを source-id でスコープされた `SourceRange` / `MappedSourceRange` 値へ変換し、フロントエンド実行で登録されたマップを保持する session source-map service を所有する。I/O、トークン化、構文解析、意味的作業は行わない。

## 公開 API

```rust
pub struct SpanBridge { /* registered per-source maps */ }

impl SpanBridge {
    pub fn new() -> Self;

    pub fn source_map_service(&self) -> &dyn SourceMapService;

    pub fn register_source(
        &mut self,
        source_id: SourceId,
        line_map: LineMap,
        loading_map: Option<LoadingMap>,
    ) -> Result<(), SpanBridgeError>;

    pub fn register_preprocess_map(
        &mut self,
        source_id: SourceId,
        preprocess_map: PreprocessMap,
    ) -> Result<(), SpanBridgeError>;

    pub fn loaded_span(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<SourceRange, SpanBridgeError>;

    pub fn loaded_mapping(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<MappedSourceRange, SpanBridgeError>;

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

pub enum SpanBridgeError {
    SourceNotRegistered { source_id: SourceId },
    PreprocessMapNotRegistered { source_id: SourceId },
    ConflictingSourceRegistration { source_id: SourceId },
    ConflictingPreprocessMapRegistration { source_id: SourceId },
    SourceMap { source: SourceMapError },
}
```

`SourceRange`、`MappedSourceRange`、`LineMap`、`LoadingMap`、`PreprocessMap`、`SourceMapError`、`RetainedSourceMapService`、`SourceMapService` は `mizar-session` が所有する。`span_bridge` は `mizar-lexer` のバイトスパンをそれらへ適合させる。`loaded_span` は読み込み済みテキスト（Step 1 座標）のスパンを、読み込み済みテキスト座標の検証済み `SourceRange` へ変換する。生のファイル／エディタ入力バイトが必要な呼び出し側は `loaded_mapping` を使う。`LoadingMap` が登録されている場合、`loaded_mapping` は retained session `SourceMapService` に loaded-to-original 変換を委譲し、`original_input` を埋める。ソース読み込みがオフセット同一で `LoadingMap` を出さなかった場合、`loaded_mapping` は登録済み `LineMap` で読み込み済み範囲を検証し、`original_input = None` の exact `MappedSourceRange` を返す。`LoadingMap::identity` は合成・保持しない。`lexical_span` はコメント除去済み字句テキスト（Step 2 以降の座標）のスパンを session `MappedSourceRange` へ変換する。スパンが exact な読み込み済みソーステキストを持つ場合、`primary` はその読み込み済みソース範囲である。合成空白だけからなる場合は、session service が最良の anchor を degraded な `primary` へ昇格する。呼び出し側はその primary を exact なユーザー記述テキストとして扱わず、`MappedSourceRange.kind` と secondary anchor を確認しなければならない。

## 依存関係

- 内部: `source`、`preprocess`、`lexing`、`parsing` が消費する。フロントエンドで最も低レベルの統制モジュールである。
- 外部: `mizar-session`（`RetainedSourceMapService`、`SourceMapService`、`SourceRange`、`MappedSourceRange`、`LineMap`、`LoadingMap`、`PreprocessMap`、`SourceMapError`、`SourceId`）、`mizar-lexer`（`mizar_lexer::source` のバイトオフセットスパン型。この境界でのみ変換される）。

## データ構造

### 変換ステージ

橋渡しは `SourceId` ごとに 3 つの `mizar-session` マップ層を合成する。

| ステージ | 変換元 | 変換先 | 所有者 |
|---|---|---|---|
| lexical → loaded | 字句テキストのバイトオフセット | 読み込み済みテキストのバイトオフセット | `PreprocessMap` |
| loaded → original | 読み込み済みテキストのバイトオフセット | 元の入力のバイトオフセット | `LoadingMap` |
| offset → line/column | 読み込み済みテキストのバイトオフセット | 1 始まりの Unicode 列 | `LineMap` |

`loaded_span` は登録済み line map に対してバイト範囲を検証し、読み込み済みテキスト座標の `SourceRange` を返す。`loaded_mapping` はさらに登録済み `LoadingMap` があれば合成し、第一範囲は読み込み済み座標のまま、`original_input` にソース読み込み入力のバイト範囲を含む `MappedSourceRange` を返す。`LoadingMap` が登録されていない場合、`loaded_mapping` は session service の loaded-to-original API を呼ばず、検証済み読み込み済み範囲を `original_input = None` で返す。その API は恒等変換であっても retained `LoadingMap` を必要とするためである。`lexical_span` は preprocess map を適用し、ゼロ長境界（例えば内部が除去コメントであった字句範囲）で隣接アンカーの合成を返し、synthetic-only span に対しては degraded な anchor-backed mapping を返す。橋渡しは session 側の `PreprocessMap` を字句解析器の `SourcePreprocessMap` から導出し、`SourceUnit` に付随する任意の session `LoadingMap` を再利用する。`SourceId` ごとに正準マップは正確に 1 つであり、frontend bridge は identity source-loading map を materialize しない。

### レジストリ

橋渡しは、Step 1-2 の間に登録されたマップを `SourceId` ごとに保持するレジストリを持ち、それらを自身が所有する `RetainedSourceMapService` へ挿入する。与えられた `SourceId` に対する登録は冪等である。既に登録済みのソースに対して異なるマップで再登録するのはプログラミングエラーであり、`SpanBridgeError` として表面化する。session の retained service 自体は `insert_*` で上書きするため、衝突検出は挿入前の frontend bridge の責務である。

## アルゴリズム / ロジック

### 読み込み済みテキストのスパン変換（Step 1 診断）

1. `span` が `source_id` の読み込み済みテキスト内にあることを検証する。
2. `source_id` でスコープされた、読み込み済みテキスト座標の `SourceRange` を構築する。
3. retained session source-map service を通じて範囲を検証し、返す。

`SourceRange` は生のファイル／エディタ入力オフセットを保持しない。LSP やソース読み込み診断のためにそれらのバイトが必要な場合、`loaded_mapping` が retained `LoadingMap` を使って `MappedSourceRange.original_input` に返す。ソース読み込みがオフセット同一で `LoadingMap` を出さなかった場合、`loaded_mapping` は `SourceMapService::original_range_for_loaded` を呼ばない。line map 検証後に、`original_input = None` の exact な読み込み済み座標 `MappedSourceRange` を構築し、二重の恒等マップを作らない。

### 字句テキストのスパン変換（Step 2 以降のトークンと診断）

1. `span` が `source_id` の字句テキスト内にあることを検証する。
2. 字句オフセットを `PreprocessMap` を通じて読み込み済みオフセットへ変換し、スパンが除去コメントをまたぐ場合は第一に加えて隣接アンカーを生成する。合成空白のように exact なユーザー記述範囲を持たない場合は degraded な anchor-backed primary になる。
3. retained session `SourceMapService` を用いて `MappedSourceRange` を返す。第一の `SourceRange` と隣接アンカーは読み込み済みソース座標である。元入力バイトは、loading map が存在するときに必要な消費者が `loaded_mapping` から得る任意ビューである。

すべての変換は算術を `mizar-session` に委譲する。このモジュールは正しいマップ層と正しい `SourceId` を選ぶだけである。

## エラー処理

`SpanBridgeError` は retained session `SourceMapService` が報告する失敗（未知のソース id、ソース／字句テキスト外の範囲、UTF-8 境界上にないオフセット、登録済みだが不完全な loading map を合成しようとした場合の欠落 loading-map セグメント、欠落した preprocess-map セグメント、行／列オーバーフロー）を `SpanBridgeError::SourceMap` として包み、さらにフロントエンドローカルの「ソース未登録」／「preprocess map 未登録」／「マップ登録の衝突」の場合を表す。橋渡しの失敗は内部不変条件の違反（宣言したソースに属さないスパン）であり、ユーザー診断ではない。統制層はこれを回復可能な字句／構文診断ではなくバグの表面として扱う。

## テスト

主要シナリオ:

- BOM 除去テキスト上の読み込み済みテキストスパンは妥当な読み込み済み `SourceRange` のままで、`loaded_mapping` が loading map を通じて正しい元のバイトオフセットを報告する。
- `LoadingMap` を持たない identity loaded source は、`LoadingMap::identity` を保持せず、`original_input = None` の exact `MappedSourceRange` を返す。
- 字句テキストスパンが preprocess map を通じて期待される読み込み済み `SourceRange` へ変換され、必要に応じて loading map から元入力バイトを参照できる。
- 除去コメントをまたぐ字句スパンが、第一の範囲に加えて隣接アンカーを生む。
- synthetic-only の字句スパンは、exact なユーザー記述範囲ではなく degraded な anchor fallback primary を持つ `MappedSourceRange` を返す。
- UTF-8 境界上にないオフセットが、暗黙の切り詰めではなく拒否される。
- 登録済みテキスト長の外のスパンが session エラーで拒否される。
- 同じ `SourceId` に異なるマップを 2 つ登録すると、衝突する登録として報告される。

## 制約と前提

- `mizar-lexer` は `mizar-session` から分離を保つ。このモジュールが、字句解析器のバイトスパンを session `SourceRange` 値へ変換する唯一の場所である。
- `SourceId` ごとに正準の line／loading／preprocess マップは正確に 1 つである。
- すべての座標算術は `mizar-session` `SourceMapService` に委譲される。橋渡しは、検証済み `TextRange` / `SourceRange` 要求の構築と重複登録検出を超えて、オフセット計算を再実装しない。
- 橋渡しの失敗は内部不変条件の違反であり、ユーザー向け診断ではない。
