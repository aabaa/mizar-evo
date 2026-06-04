# mizar-frontend TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| span_bridge | [span_bridge.md](./span_bridge.md) | `src/span_bridge.rs` | [ ] |
| source | [source.md](./source.md) | `src/source.rs` | [ ] |
| preprocess | [preprocess.md](./preprocess.md) | `src/preprocess.rs` | [ ] |
| lexical_env | [lexical_env.md](./lexical_env.md) | `src/lexical_env.rs` | [ ] |
| lexing | [lexing.md](./lexing.md) | `src/lexing.rs` | [ ] |
| parsing | [parsing.md](./parsing.md) | `src/parsing.rs` | [ ] |
| orchestration | [orchestration.md](./orchestration.md) | `src/orchestration.rs` | [ ] |

`mizar-frontend` は統制 crate なので、フェーズ順にボトムアップで構築する。まず座標橋渡し、次にパイプライン順で Step 1-5、最後にエンドツーエンドの coordinator である。`span_bridge` は後続の各フェーズが参照する共有プリミティブであり、`orchestration` は完全なパイプラインを配線する唯一のモジュールである。

依存順序: `span_bridge` → `source` → `preprocess` → `lexical_env` → `lexing` → `parsing` → `orchestration`。

## crate 前提条件

`mizar-frontend` は `mizar-syntax`（`SurfaceAst`）と `mizar-parser`（文法エントリポイント）に依存するが、いずれもまだ未実装である（トップレベル [../../todo.md](../../todo.md) は両者を未着手としている）。下記のタスク 1-7 は `mizar-session` と `mizar-lexer` のみに対して実装できる。構文解析フェーズ（タスク 11-12）と完全な統制の構文解析ステップ（タスク 14）は、少なくとも最小限の `mizar-parser` エントリポイントと `mizar-syntax::SurfaceAst` を必要とする。それらが揃うまでは `parsing` モジュールのトレイト背後でパーサーの継ぎ目をスタブ化し、`ast = None` を返して、ソース → トークンのパイプラインをエンドツーエンドで実行できるようにする。

## 先に解決すべきオープン決定

これらは公開 API を左右し、[../../todo.md](../../todo.md) の「Open Decisions」でトップレベル管理されている。

- **字句解析器スパン橋渡し。** この crate は分離オプションを採用する。`mizar-lexer` はバイトオフセットスパンを保持し、`span_bridge`（タスク 1）がそれらを `mizar-session` `SourceRange` へ変換する。橋渡し API を凍結する前に、`mizar-session` の `source_map` 統合と照合して確認すること。
- **パーサー支援字句解析の契約。** タスク 10-12 を配線する前に、狭い `ParserLexContext` / `ParserInputs` リクエストオブジェクト（文字列必須位置と記号種フィルタ）を仕様化し、字句解析器が任意のパーサー状態を受け取らないようにすること。

## 順序付きタスク一覧

各タスクは単独で実装・テスト・コミットできる大きさである。タスクは依存順であり、後のタスクは先のものがマージ済みと仮定する。各タスクは `cargo test -p mizar-frontend` を green に保つこと（[推奨検証](#推奨検証)を参照）。

### crate の足場

1. **crate 骨格と座標橋渡し。** [ ]
   - `mizar-frontend` crate をワークスペースに追加し、`mizar-session` と `mizar-lexer` に依存させる。ワークスペースの `[workspace.lints]` テーブルに `lints.workspace = true` でオプトインする（`mizar-session` と同様）。
   - `pub mod span_bridge;` を追加し、`SpanBridge`、`LexerByteSpan`、`SpanBridgeError` を定義する。`register_source` / `register_preprocess_map` と `loaded_span` / `lexical_span` 変換を `mizar-session` `SourceMapService` 上で実装する。
   - session 側の `LoadingMap` / `PreprocessMap` を字句解析器の `SourceLoadingMap` / `SourcePreprocessMap` から導出する（または `SourceUnit` に付随する session `LoadingMap` を再利用する）ので、`SourceId` ごとに正準マップは 1 つである。
   - テスト: BOM 除去テキスト上の読み込み済みスパンが元のオフセットへ変換される。字句スパンが preprocess + loading マップを通じて変換される。除去コメントをまたぐスパンが第一 + 隣接アンカーを生む。UTF-8 境界外および範囲外のスパンが拒否される。衝突するマップ登録が報告される。
   - 仕様: [span_bridge.md](./span_bridge.md)「Public API」「Algorithm / Logic」。

### モジュール: source (`src/source.rs`)

2. **`SourceUnit` とローダー橋渡し。** [ ]
   - `pub mod source;` を追加する。`SourceUnit`、`SourceUnitRequest`、`SourceUnitLoader` トレイト、`FrontendSourceLoader<L: SourceLoader>` を定義し、`mizar_session::LoadedSource` をハッシュ・line map・loading map を再計算せずに `SourceUnit` へ射影する `source_unit_from_loaded` を実装する。
   - 読み込んだ `LineMap` / `LoadingMap` を `SourceId` の下で `SpanBridge` に登録する。
   - テスト: ディスク `LoadedSource` が同一の id／hash／line-map／loading-map で射影される。BOM／CRLF 正規化ソースは `Some(loading_map)` を運ぶ。恒等読み込みは `None` を運ぶ。オープンバッファ origin とバージョンが保持される。session `SourceLoadError` がそのまま伝播する。
   - 依存: 1。仕様: [source.md](./source.md)「Public API」「Algorithm / Logic」。

### モジュール: preprocess (`src/preprocess.rs`)

3. **コメントとドキュメントコメントの前処理。** [ ]
   - `pub mod preprocess;` を追加する。`PreprocessedSource`、`LexicalText`、`Comment`、`DocComment`、`LexicalSourceMap`、`PreprocessDiagnostic` を定義する。
   - `SourceUnit.source_text` に対して `mizar_lexer::preprocess_source_for_lexing` を駆動し、コメント／ドキュメントコメント／前処理診断のスパンを `SpanBridge` を通じて変換し、`LexicalSourceMap` を組み立てる。
   - テスト: 通常コメントが字句テキストから除去されるが正しい範囲を持つ `Comment` として保持される。ドキュメントコメントが生本文と範囲とともに保持される。注釈構文が字句テキストに残る。除去コメントをまたぐ字句範囲が合成マッピングを生む。コード領域の非 ASCII 文字と未終端ブロックコメントが報告・回復される。
   - 依存: 2。仕様: [preprocess.md](./preprocess.md)「Comments and Doc Comments」「Algorithm / Logic」。

4. **浅いインポート事前走査の統合。** [ ]
   - 字句テキストを生スキャン（`scan_raw`）し `mizar_lexer::scan_import_prelude` を実行する。変換済み `SourceRange` を持つ `import_stubs` を満たし、`ImportPrescanDiagnostic` を `diagnostics` に集める。
   - テスト: トップレベル `import` 形式が正しい種別・生パス・スパンを持つ `ImportStub` を生む。不正なインポートが中断せずインポート事前走査診断を生む。後のタイブレークのためにインポート順が保持される。
   - 依存: 3。仕様: [preprocess.md](./preprocess.md)「Import Stubs」「Error Handling」。

### モジュール: lexical_env (`src/lexical_env.rs`)

5. **字句環境リクエストとプロバイダの継ぎ目。** [ ]
   - `pub mod lexical_env;` を追加する。`LexicalEnvironmentRequest`、`LexicalSummaryProvider`、`ResolvedImports`、`ActiveLexicalEnvironmentResult`、`LexicalEnvironmentDiagnostic` を定義し、`mizar-lexer` の環境型を再エクスポートする。
   - テスト: 解決済みインポート + サマリを返す偽プロバイダが、正しい出所とインポート序数を持つ `UserSymbolIndex` を生む。予約テーブルが常に存在する。
   - 依存: 4。仕様: [lexical_env.md](./lexical_env.md)「Public API」。

6. **アクティブ字句環境の構築。** [ ]
   - `mizar_lexer::build_lexical_environment` を呼ぶ `build_active_lexical_environment` を実装する。プロバイダ診断を統合し、`LexicalEnvironmentFingerprint` を計算・表面化する。
   - テスト: インポート順のタイブレークが期待される同じ長さのユーザー記号を選ぶ。未解決インポートが診断とともにより小さな環境へ縮退し、残りの記号が読み込まれる。fingerprint が依存サマリの変更で変化し、ローカルのコメントのみの編集では安定である。
   - 依存: 5。仕様: [lexical_env.md](./lexical_env.md)「Algorithm / Logic」「Error Handling」。

### モジュール: lexing (`src/lexing.rs`)

7. **生スキャンとスコープスケルトンの配線。** [ ]
   - `pub mod lexing;` を追加する。`TokenizeRequest`、フロントエンドの `Token` / `TokenStream`（session スパン付き）を定義し、`TokenKind` / `LexDiagnostic` を再エクスポートする。
   - 生トークンから `ScopeSkeleton` / `ScopeLexView` を構築し、曖昧性解消器の入力を準備する。
   - テスト: 生スキャンが `LexemeRun` スパンを保持する。スコープビューが解決済み束縛なしで字句ブロック／文の形を反映する。
   - 依存: 6。仕様: [lexing.md](./lexing.md)「Scope Lex View」「Algorithm / Logic」。

8. **文脈依存曖昧性解消による `TokenStream`。** [ ]
   - アクティブ字句環境、スコープビュー、省略可能な `ParserLexContext` を与えて `disambiguate`（またはパーサー統合の `lex`）を実行する。各字句解析器スパンを `SpanBridge` を通じて session `SourceRange` へ変換する。
   - テスト: 識別子とつづりを共有するユーザー記号が最長一致で分類される。複合予約トークン（`.{`、`.*`、`.=`、`...`）が単一トークンとして字句解析される。引用符が文字列必須位置の外では記号文字として、注釈／演算子宣言引数では `StringLiteral` として字句解析される。送出された各トークンスパンが妥当な `SourceRange` へ解決される。
   - 依存: 7。仕様: [lexing.md](./lexing.md)「Token Stream」「Algorithm / Logic」。

9. **字句解析器回復のパススルー。** [ ]
   - `TokenKind::ErrorRecovery` スパンと `LexDiagnostic` をエンドツーエンドで保持する。`tokenize` が常に `TokenStream` を返すことを保証する。
   - テスト: 不正なトークンが正しい `SourceRange` を持つ `ErrorRecovery` を送出しスキャンが再開する。不正な数値と文字列必須位置での不正な文字列リテラルが、回復可能トークンを落とさずに報告される。
   - 依存: 8。仕様: [lexing.md](./lexing.md)「Error Handling」。

### モジュール: parsing (`src/parsing.rs`)

10. **パーサー入力の組み立てとパーサーの継ぎ目。** [ ]
    - `pub mod parsing;` を追加する。`ParseRequest`、`ParserInputs`、`ParseOutput`、パーサー継ぎ目トレイトを定義し、アクティブ字句環境から `ParserInputs`（エディション、演算子結合性、文字列必須位置）を導出する。
    - `mizar-parser` が存在するまで、継ぎ目を `ast = None` と空の診断リストを返すスタブに対して実装し、ソース → トークンのパイプラインを実行可能にする。
    - テスト: `ParserInputs` が環境から導出した演算子結合性と文字列必須位置を運び、解決器の状態を運ばない。スタブ継ぎ目が `ast = None` を返す。
    - 依存: 8。仕様: [parsing.md](./parsing.md)「Parser Inputs」「Public API」。

11. **`mizar-parser` の呼び出し。** [ ]
    - スタブ継ぎ目を実際の `mizar-parser` エントリポイントに置き換える。`mizar-syntax::SurfaceAst` と構文診断をそのまま返す。
    - 最小限の `mizar-parser` / `mizar-syntax` を必要とする（トップレベル [../../todo.md](../../todo.md)）。それらの可用性でゲートする。
    - テスト: 整形式トークンストリームがソース順と範囲を保持した `SurfaceAst` へ解析される。演算子結合性がユーザー中置演算子に対する正しい Pratt 優先順位を駆動する。注釈引数の `StringLiteral` が注釈ノードへ解析される。
    - 依存: 10、加えて `mizar-parser`/`mizar-syntax`。仕様: [parsing.md](./parsing.md)「Algorithm / Logic」。

12. **パーサー回復のパススルー。** [ ]
    - 回復不能な入力での `ast = None` と、返された `SurfaceAst` 内の明示的な回復ノード印を保持する。構文診断を通す。
    - テスト: `end` の欠落が同期点で回復し `ast = Some` と明示的エラーノードを生む。回復不能なストリームが診断とともに `ast = None` を返す。文字列必須位置での文字列リテラルの欠落が期待される構文診断を生む。
    - 依存: 11。仕様: [parsing.md](./parsing.md)「Error Handling」。

### モジュール: orchestration (`src/orchestration.rs`)

13. **フロントエンド coordinator と診断統合。** [ ]
    - `pub mod orchestration;` を追加する。`FrontendOutput`、`Frontend`、`FrontendDiagnostic`、`DiagnosticClass`、`FrontendError` を定義する。`source` → `preprocess` → `lexical_env` → `lexing` → `parsing` を配線し、すべてのフェーズ診断を [orchestration.md](./orchestration.md)「Diagnostic Merge Order」の決定的順序へ統合する。
    - テスト: 整形式ソースが診断なしで `ast = Some` を返す。字句前提・トークン化・構文のエラーを持つソースが決定的順序で 3 つすべてを報告する。統合順序が繰り返し実行で同一である。
    - 依存: 9、12。仕様: [orchestration.md](./orchestration.md)「Algorithm / Logic」「Diagnostic Merge Order」。

14. **回復不能な失敗の処理とエンドツーエンド出力。** [ ]
    - `FrontendError` を Step 1 読み込み失敗と `SpanBridgeError` 不変条件違反のみに対して返す。回復可能な問題は `FrontendOutput` 内の診断として保つ。
    - テスト: Step 1 読み込み失敗がファイルレベル診断を伴う `FrontendError` を返し出力なし。構文解析失敗が先行診断を保持しつつ `ast = None` を返す。統合診断が妥当な `SourceRange` を運ぶ。
    - 依存: 13。仕様: [orchestration.md](./orchestration.md)「Error Handling」。

### 横断的フォローアップの前のモジュール全体の保守

15. **実装リファクタリングパス。** [ ]
    - 最初の実装パス完了後に `span_bridge`、`source`、`preprocess`、`lexical_env`、`lexing`、`parsing`、`orchestration` を見直す。
    - 明白なバグや仕様不一致が露見しない限り公開 API と挙動を安定に保つ。小さな局所的抽出と共有テストフィクスチャを優先する。
    - テスト: すべてのモジュールテストを green に保つ。
    - 依存: 14。仕様: すべての mizar-frontend モジュール仕様。

16. **ソース／仕様対応監査。** [ ]
    - フロントエンド仕様の各公開 API、エラー変種、タスク要件から実装ソース／テストへの軽量なトレーサビリティ確認を構築する。
    - 欠落実装、古い仕様文、欠落テストを、監査に広範な変更を混ぜずにフォローアップタスクとして記録する。
    - 英語正本仕様を先に確認し、次に日本語companionが同じ API と挙動の約束を持つか検証する。
    - 依存: 15。仕様: すべての mizar-frontend モジュール仕様と本 TODO。

## 横断的フォローアップタスク

17. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-frontend/en/` の各英語正本ドキュメントを `doc/design/mizar-frontend/ja/` の日本語 companion と比較し、API 一覧、タスク状態、用語、リンクを同期する。
    - 依存: 16。仕様: リポジトリのドキュメント方針。

18. **フロントエンド決定性プロパティテスト。** [ ]
    - 同一入力が内部スケジューリングと独立に同一の `FrontendOutput` 診断順序と同一のトークンスパンを生むこと、および `LexicalEnvironmentFingerprint` とキャッシュキーが等価入力に対して安定であることの crate レベル網羅を追加する。
    - 依存: 16。仕様: [orchestration.md](./orchestration.md)、[lexical_env.md](./lexical_env.md)。

19. **インクリメンタルキャッシュキーの配線。** [ ]
    - [architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)「Incrementality」の層状フロントエンドキャッシュキーをどこで計算・保存するか（本 crate かドライバ／成果物層か）を決め、成果物単位のキー（`SourceUnit`、`PreprocessedSource`、`ActiveLexicalEnvironment`、`TokenStream`、`SurfaceAst`）を公開する。
    - コメントのみの編集が意味的出力を再利用でき、インポート／依存エクスポート編集がトークン化と下流層を無効化することを検証する。
    - 依存: 16。仕様: アーキテクチャのインクリメンタリティ表。

20. **パーサー支援字句解析契約の確定。** [ ]
    - `mizar-parser` が存在したら、曖昧性解消を事前計算した `ParserLexContext` で 1 パス実行するか、狭い文脈オブジェクトを通じて構文解析と交錯させるかを確定し、選択した統合を [lexing.md](./lexing.md) と [parsing.md](./parsing.md) に記録する。
    - いずれの選択でも字句解析器を任意のパーサー状態から自由に保つ。
    - 依存: 11。仕様: トップレベル [../../todo.md](../../todo.md)「Open Decisions」、[lexing.md](./lexing.md)、[parsing.md](./parsing.md)。

21. **恒久的な lint 強制。** [ ]
    - `crates/mizar-frontend/Cargo.toml` がワークスペースの `[workspace.lints]` テーブルにオプトインし、`cargo build`/`cargo test` が単独 clippy ゲートと同じ拒否を表面化することを確認する（`mizar-session` 方針と同様）。
    - 意図的な `allow` 例外は `allow` の隣に根拠とともに記録する。
    - テスト: `cargo clippy -p mizar-frontend --all-targets -- -D warnings` が通る。
    - 依存: 16。仕様: 本 TODO「推奨検証」。

## 推奨検証

各タスク後に実行する。

```text
cargo test -p mizar-frontend
cargo clippy -p mizar-frontend --all-targets -- -D warnings
```

字句解析器／session 境界に触れるタスクは次も実行する。

```text
cargo test -p mizar-session
cargo test -p mizar-lexer
```

テストが通ったらここでタスクをチェックする。

## 注記

- `mizar-frontend` は統制 crate である。`mizar-session`、`mizar-lexer`、`mizar-syntax`、`mizar-parser` を統制するが、それらの中核アルゴリズムやデータ定義は何も所有しない。
- `mizar-lexer` を `mizar-session` から分離したまま保つ。字句解析器スパン → session `SourceRange` の橋渡しは `span_bridge` にのみ存在する。
- フロントエンドは構文を生成し、意味は生成しない。名前解決、型検査、オーバーロード選択、証明義務はここに属さない。
- フロントエンド成果物（`SourceUnit`、`PreprocessedSource`、`TokenStream`、`SurfaceAst`、`FrontendOutput`）は内部コンパイラデータであり、安定した外部スキーマではない。
