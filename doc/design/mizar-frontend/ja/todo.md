# mizar-frontend TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| span_bridge | [span_bridge.md](./span_bridge.md) | `src/span_bridge.rs` | [x] |
| source | [source.md](./source.md) | `src/source.rs` | [x] |
| preprocess | [preprocess.md](./preprocess.md) | `src/preprocess.rs` | [x] |
| lexical_env | [lexical_env.md](./lexical_env.md) | `src/lexical_env.rs` | [x] |
| lexing | [lexing.md](./lexing.md) | `src/lexing.rs` | [x] |
| parsing | [parsing.md](./parsing.md) | `src/parsing.rs` | [~] |
| orchestration | [orchestration.md](./orchestration.md) | `src/orchestration.rs` | [ ] |

`mizar-frontend` は統制を担う crate なので、フェーズの順にボトムアップで構築する。まず座標の橋渡しを用意し、続いてパイプライン順に Step 1〜5、最後にエンドツーエンドのコーディネータを作る。`span_bridge` は後続の各フェーズが参照する共有プリミティブであり、`orchestration` はパイプライン全体を配線する唯一のモジュールである。

依存順序: `span_bridge` → `source` → `preprocess` → `lexical_env` → `lexing` → `parsing` → `orchestration`。

## crate の前提条件

フロントエンドの基盤は `mizar-session` と `mizar-lexer` に依存する。`mizar-syntax` と `mizar-parser` はまだ未実装なので（トップレベルの [../../todo.md](../../todo.md) でも両者は未着手扱い）、実 parser seam を扱うタスクが導入されるまでは、これらへの必須依存を追加しない。タスク 1〜10 と、タスク 13〜14 のスタブコーディネータ部分は、`mizar-session` と `mizar-lexer` だけで実装できる。タスク 11〜12 の実 parser 呼び出しと syntax AST に対する検証、およびタスク 13〜14 の実 parser 検証は、最小限の `mizar-parser` エントリポイントと `mizar-syntax::SurfaceAst` が揃ってから着手する。

## 解決済みおよび保留中の決定

これらの公開 API に関する決定は、[../../todo.md](../../todo.md) の「Resolved And Open Decisions」でトップレベルとして管理している。

- **字句解析器スパンの橋渡し: 解決済み。** この crate は分離方式を採用する。`mizar-lexer` はバイトオフセットのスパンを保持し、`span_bridge`（タスク 1）がそれらを `mizar-session` の `SourceRange` へ変換する。
- **パーサー支援字句解析の契約: 保留中。** 現在の字句解析器は一様な `ParserLexContext` を公開しており、位置ごとに異なる文字列必須スパンはまだ表現できない。位置別の `StringLiteral` 認識、注釈の文字列引数内での Unicode 受理、パーサー駆動の記号種別フィルタは、任意のパーサー状態を晒さない狭い `ParserLexContext` / `ParserInputs` 契約が固まってから着手する。

## 順序付きタスク一覧

各タスクは、単独で実装・テスト・コミットできる粒度になっている。依存関係は各タスクの「依存」行を正とする。`mizar-parser` / `mizar-syntax` がまだ無いときは、保留中の実 parser タスクを飛ばし、それらに依存しないスタブ版の source → tokens コーディネータタスクを先に進める。各タスクでは `cargo test -p mizar-frontend` を成功状態に保つこと（[推奨検証](#推奨検証)を参照）。

### crate の足場

1. **crate の骨格と座標の橋渡し。** [x]
   - `mizar-frontend` crate をワークスペースに追加し、`mizar-session` と `mizar-lexer` に依存させる。ワークスペースの `[workspace.lints]` テーブルに `lints.workspace = true` でオプトインする（`mizar-session` と同様）。
   - `pub mod span_bridge;` を追加し、`SpanBridge`、`LexerByteSpan`、`SpanBridgeError` を定義する。`SpanBridge` は session の常駐ソースマップサービスを所有し、失敗しうる `register_source` / `register_preprocess_map` と、常駐する `mizar-session` のマップ上で行う `loaded_span`・`loaded_mapping`・`lexical_span` の変換を提供する。
   - `SpanBridgeError` は、フロントエンドローカルな登録上の不変条件（`SourceNotRegistered`、`PreprocessMapNotRegistered`、ソース／前処理の登録衝突）と、ラップした `mizar-session::SourceMapError` を区別する。
   - `SourceUnit` に付随する任意の session `LoadingMap` を再利用し、session 側の `PreprocessMap` は字句解析器の `SourcePreprocessMap` から導出する。恒等読み込みに対して `LoadingMap::identity` を合成・保持しない。`loaded_mapping` が loading map を見つけられない場合は、line map による検証の後、`original_input = None` の厳密な読み込み済み座標マッピングを返す。
   - テスト: BOM を除去したテキスト上の `loaded_span` は読み込み済みテキスト座標のままで、`loaded_mapping` は `MappedSourceRange.original_input` を通じて元入力のオフセットを報告する。字句スパンが preprocess map を通じて読み込み済みソース座標へ変換される。`LoadingMap` を持たない恒等読み込みは `original_input = None` の厳密な `loaded_mapping` を返す。除去されたコメントをまたぐスパンが、第一の範囲に加えて隣接アンカーを生む。合成のみからなる字句スパンは、厳密な第一ソース範囲ではなく、アンカーに支えられた縮退マッピングを生む。UTF-8 境界外や範囲外のスパンが拒否される。衝突するマップ登録が報告される。
   - 仕様: [span_bridge.md](./span_bridge.md)「Public API」「Algorithm / Logic」。

### モジュール: source (`src/source.rs`)

2. **`SourceUnit` とローダーの橋渡し。** [x]
   - `pub mod source;` を追加する。`SourceUnit`、`SourceUnitRequest`、`SourceUnitLoader` トレイト、`FrontendSourceLoader<L: SourceLoader>` を定義し、`mizar_session::LoadedSource` を、ハッシュ・line map・loading map・正規化パス・エディション・origin・生成アンカーを再計算せずに `SourceUnit` へ射影する `source_unit_from_loaded` を実装する。
   - `LoadedSource` はファイルシステムパスを保持しないので、`file_path` は呼び出し側が提供する診断メタデータとして扱う。ディスク／オープンバッファのソースでは request/origin から、生成ソースでは正規化パスまたは生成アンカーから導出する。
   - 読み込んだ `LineMap` / `LoadingMap` を `SourceId` の下で可変な `SpanBridge` に登録するヘルパーを提供する。統制層が、読み込み後・前処理前にこのヘルパーを呼び出す。`load_source_unit` 自体は bridge の状態を変更しない。
   - テスト: ディスクの `LoadedSource` が同一の id／hash／line-map／loading-map で射影される。BOM／CRLF を正規化したソースは `Some(loading_map)` を運ぶ。恒等読み込みは `None` を運ぶ。正規化パスとエディションが保持される。オープンバッファの origin とバージョンが保持される。生成ソースは `generated_anchor` を保持する。`register_source_unit` が line/loading map を記録し、衝突する重複登録を報告する。session の `SourceLoadError` がそのまま伝播する。
   - 依存: 1。仕様: [source.md](./source.md)「Public API」「Algorithm / Logic」。

### モジュール: preprocess (`src/preprocess.rs`)

3. **コメントとドキュメントコメントの前処理。** [x]
   - `pub mod preprocess;` を追加する。`PreprocessedSource`、`LexicalText`、`Comment`、`DocComment`、`LexicalSourceMap`、`lexical_hash`、写像済みの `ImportStub` / `ImportStubPath` / `ImportStubRelativePrefix` / `ImportStubAlias`、および code/message・第一の `SourceRange`・副次の `SourceAnchor` を持つ `PreprocessDiagnostic` を定義する。
   - `SourceUnit.source_text` に対して `mizar_lexer::preprocess_source_for_lexing` を駆動し、コメント・ドキュメントコメント・前処理診断のスパンを `SpanBridge` を通じて変換し、`LexicalSourceMap` を組み立てる。
   - テスト: 通常コメントが字句テキストからは除去されるが、正しい範囲を持つ `Comment` として保持される。ドキュメントコメントが生本文と範囲とともに保持される。注釈構文が字句テキストに残る。除去されたコメントをまたぐ字句範囲が合成マッピングを生む。合成空白はアンカーに支えられた縮退マッピングとしてのみ表現される。コメントのみの編集で字句テキストが変わらなければ `lexical_hash` が安定する。コード領域の非 ASCII 文字と未終端ブロックコメントが報告・回復される。
   - 依存: 2。仕様: [preprocess.md](./preprocess.md)「Comments and Doc Comments」「Algorithm / Logic」。

4. **浅いインポート事前走査の統合。** [x]
   - 字句テキストを生スキャン（`scan_raw`）し、`mizar_lexer::scan_import_prelude` を実行する。変換済みの `SourceRange` を持つ `import_stubs` を満たし、`ImportPrescanDiagnostic` を `diagnostics` に集める。
   - 厳密な生スキャンが失敗した場合は、字句テキスト全体（空ならソース先頭のゼロ長範囲）を覆うフロントエンドローカルなインポート事前走査診断を記録し、`import_stubs` を空にして、部分的な生テキストから import を推測せずに続行する。回復可能な生スキャナーの契約が用意されるまでは、`mizar_lexer::LexError` がスパンを持つと仮定しない。
   - テスト: トップレベルの `import` 形式が、生パス・任意の alias・`path.relative`・`path.source_segments`・スパンを持つ `ImportStub` を生む。`.` と `..` の相対 prefix が current／parent インポートとして区別されたまま保持される。不正なインポートが中断せずにインポート事前走査診断を生む。インポート事前走査中の生スキャン失敗が、粗い診断と空の `import_stubs` を生む。出所と決定的なフィンガープリントのために、インポート順が保持される。
   - 依存: 3。仕様: [preprocess.md](./preprocess.md)「Import Stubs」「Error Handling」。

### モジュール: lexical_env (`src/lexical_env.rs`)

5. **字句環境リクエストとプロバイダの継ぎ目。** [x]
   - `pub mod lexical_env;` を追加する。`LexicalEnvironmentRequest`、`LexicalSummaryProvider`、`ResolvedImports`、`ResolvedImportEntry`、`ActiveLexicalEnvironmentResult`、`LexicalEnvironmentDiagnostic`、`LexicalEnvironmentDiagnosticCode`、`FrontendLexicalEnvironmentError` を定義し、`mizar-lexer` の環境型を再エクスポートする。
   - 各解決済みインポートについて、元の `ImportStub` の序数とスパンを `ResolvedImportEntry` に保持する。`mizar_lexer::build_lexical_environment` へは、順序付きの字句解析器 `ResolvedImport` だけを渡す。
   - 字句解析器を呼び出す前に、解決済みインポートを `ModuleId` で正規化し、インポート順で最初の stub を有効な出所エントリとして保持する。重複 stub の出所はプロバイダ診断のために保持するが、現在の字句解析器には重複した module id を渡さない。字句解析器の衝突エラーは module を識別するものの、フロントエンドのインポート序数までは識別しないためである。
   - プロバイダ基盤の障害には、字句解析器が所有する `LexicalEnvironmentError` ではなく `FrontendLexicalEnvironmentError` を使う。
   - テスト: 解決済みインポートとサマリを返す偽プロバイダが、正しい正準の出所とインポート序数を持つ `UserSymbolIndex` を生む。同じ module へ解決される複数の stub は、字句解析器の呼び出し前に重複排除され、プロバイダ診断は正しい元のインポートスパンに出る。予約テーブルが常に存在する。
   - 依存: 4。仕様: [lexical_env.md](./lexical_env.md)「Public API」。

6. **アクティブ字句環境の構築。** [x]
   - タスク 5 の `build_active_lexical_environment` エントリポイントを、残りの回復処理で拡張する。`mizar_lexer::build_lexical_environment` の呼び出し、プロバイダ診断の統合、表面化済みの `LexicalEnvironmentFingerprint` は維持する。
   - 字句解析器を呼ぶ前に、未解決のインポートと、依存する字句サマリが利用できないインポートを除外し、`LexicalEnvironmentDiagnostic` として表す。サマリが欠落した入力で字句解析器に環境構築を要求しない。
   - `LexicalEnvironmentError::UserSymbolImportConflict` は、有界で決定的な再試行で扱う。正準の `ResolvedImportEntry` の出所を使って後側の衝突インポートを診断し、分かる場合は前側のインポートを副次コンテキストに加え、後側の衝突 module を除いて、元の正準インポートごとに高々 1 回だけ再試行する。それ以外の字句解析器 `LexicalEnvironmentError` は `FrontendLexicalEnvironmentError::MalformedSummary` として扱う。
   - テスト: 異なるモジュールからインポートされた同綴りのユーザー記号は、決定的な字句環境衝突診断を生み、後側の衝突 module を落として再試行する。同じ module の重複インポートは偽の衝突を作らない。未解決のインポートは、診断とともに、より小さな環境へ縮退し、残りの記号が読み込まれる。欠落したサマリは、字句解析器の呼び出し前に除外・診断される。衝突以外の字句解析器環境エラーは `MalformedSummary` になる。フィンガープリントは依存サマリの変更で変化し、ローカルのコメントのみの編集では安定である。
   - 依存: 5。仕様: [lexical_env.md](./lexical_env.md)「Algorithm / Logic」「Error Handling」。

### モジュール: lexing (`src/lexing.rs`)

7. **生スキャンとスコープスケルトンの配線。** [x]
   - `pub mod lexing;` を追加する。`TokenizeRequest`、`InternedText = Arc<str>`、フロントエンドの `Token` / `TokenStream`（session スパン付き）、`LexingDiagnostic`、`LexingDiagnosticKind` / `LexingDiagnosticPayload` を定義し、`TokenKind` と、`LexDiagnosticCode` / `ScopeSkeletonDiagnosticCode` などの生の字句解析器診断コード enum を再エクスポートする。
   - 生トークンから `ScopeSkeleton` / `ScopeLexView` を構築し、曖昧性解消器への入力を準備する。`ScopeSkeletonDiagnostic` は、スパンを持つ生の診断構造体を公開 `TokenStream` に保持せず、`LexingDiagnostic` へ写像する。
   - テスト: 生スキャンが `LexemeRun` のスパンを保持する。スコープビューが、解決済みの束縛なしで字句ブロック／文の形を反映する。
   - 依存: 6。仕様: [lexing.md](./lexing.md)「Scope Lex View」「Algorithm / Logic」。

8. **文脈依存の曖昧性解消による `TokenStream`。** [x]
   - アクティブ字句環境、初回の生 `ScopeSkeleton` / `ScopeLexView`、最終 token 形状から作り直した contextual scope skeleton、現在の `ParserLexContext`（パーサー支援契約が確定するまでは一般／スタブの文脈）を与えて `disambiguate`（またはパーサー統合の `lex`）を実行する。各字句解析器トークンと診断のスパンを `SpanBridge` を通じて session の `SourceRange` へ変換する。生の `LexDiagnostic` は code/message をコピーし、構造化ペイロードは写像済みの形で保持して、フロントエンドの `LexingDiagnostic` へ変換する。棄却候補の入れ子スパンは session 範囲へ写像し、複合／縮退マッピング由来の副次 `SourceAnchor` を保持する。内部の写像不変条件の失敗だけを `Err(SpanBridgeError)` として返す。
   - テスト: 識別子とつづりを共有するユーザー記号が、最長一致で分類される。複合予約トークン（`.{`、`.*`、`.=`、`...`）が単一トークンとして字句解析される。引用符で囲まれた綴りは、アクティブな字句環境から記号として供給されない限り、一般文脈では写像済み字句解析器診断として棄却され、有界で一様な `StringRequired` 文脈では `StringLiteral` を生む。送出された各トークンスパンが妥当な第一 `SourceRange` へ解決され、隣接アンカーは診断用に保持される。棄却トークン候補を持つ字句解析器ペイロードは、スパン以外のペイロードデータと写像済みの入れ子スパンを保持する。注釈／演算子位置の文字列リテラルテストは、パーサー支援字句解析の契約まで延期する。
   - 依存: 7。仕様: [lexing.md](./lexing.md)「Token Stream」「Algorithm / Logic」。

9. **字句解析器の回復のパススルー。** [x]
   - `TokenKind::ErrorRecovery` のスパンと字句解析器診断を、写像済みの `LexingDiagnostic` としてエンドツーエンドで保持する。回復可能な曖昧性解消器／字句解析器診断を追加し、フロントエンドの `tokenize` ラッパーが回復可能な入力問題に対して `Ok(TokenStream)` を返すことを保証する。
   - テスト: 不正なトークンが、正しい `SourceRange` を持つ `ErrorRecovery` を送出し、スキャンが再開する。字句解析器が公開するようになった場合の不正数値診断と未対応 raw-token ケースが、回復可能トークンを落とさずに報告される。スコープスケルトン診断が、曖昧性解消後も写像済みスパンとともに保持される。
   - 依存: 8。仕様: [lexing.md](./lexing.md)「Error Handling」。

### モジュール: parsing (`src/parsing.rs`)

10. **パーサー入力の組み立てとパーサーの継ぎ目。** [x]
    - `pub mod parsing;` を追加する。`ParseRequest`、`ParserInputs`、`OperatorFixityTable`、`OperatorFixityEntry`、`OperatorAssociativity`、`StringRequiredContext`、`ParseOutput`、`ParserSeam`、`StubParserSeam` を定義し、アクティブ字句環境の構築後に、ソースのエディションと、字句サマリが現在公開しているデータだけを使って `ParserInputs` を導出する。
    - `mizar-parser` が存在するまでは、継ぎ目を `ast = None` と空の診断リストを返すスタブとして実装し、ソース → トークンのパイプラインを実行可能にする。
    - テスト: `ParserInputs` はエディションを運び、サマリが fixity を公開していないときは空の演算子 fixity テーブルを使い、スタブの source-to-token 経路では `StringRequiredContext::None` を使い、解決器の状態を運ばない。スタブの継ぎ目が `ast = None` を返す。
    - 依存: 8。仕様: [parsing.md](./parsing.md)「Parser Inputs」「Public API」。

11. **`mizar-parser` の呼び出し。** [ ]
    - スタブの継ぎ目を実際の `mizar-parser` エントリポイントに置き換える。`mizar-syntax::SurfaceAst` と構文診断をそのまま返す。
    - 最小限の `mizar-parser` / `mizar-syntax` を必要とする（トップレベルの [../../todo.md](../../todo.md)）。それらが利用可能になってから着手する。
    - テスト: 整形式のトークンストリームが、ソース順と範囲を保持した `SurfaceAst` へ解析される。サマリが fixity を公開したら、演算子の結合性が、ユーザー定義中置演算子に対する正しい Pratt 優先順位を駆動する。注釈／演算子の文字列リテラルテストは、タスク 20 が実ソーステキスト向けのパーサー支援字句解析を確定するまで、合成パーサートークンストリームを使う。
    - 依存: 10、加えて `mizar-parser`/`mizar-syntax`。文法位置の文字列リテラルを必要とする実ソーステキストのテストは、20 にも依存する。仕様: [parsing.md](./parsing.md)「Algorithm / Logic」。

12. **パーサーの回復のパススルー。** [ ]
    - 回復不能な入力での `ast = None` と、返された `SurfaceAst` 内の明示的な回復ノードの印を保持する。構文診断を通す。
    - テスト: `end` の欠落が同期点で回復し、`ast = Some` と明示的なエラーノードを生む。回復不能なストリームが、診断とともに `ast = None` を返す。文字列必須位置での文字列リテラルの欠落は、タスク 20 が実ソーステキスト向けのパーサー支援字句解析を確定するまで、合成トークンストリームで期待される構文診断を確認する。
    - 依存: 11。仕様: [parsing.md](./parsing.md)「Error Handling」。

### モジュール: orchestration (`src/orchestration.rs`)

13. **フロントエンドコーディネータと診断統合。** [ ]
    - `pub mod orchestration;` を追加する。`FrontendOutput`、`Frontend`、`FrontendDiagnostic`、`DiagnosticLocation`、`SourceLoadLocation`、`DiagnosticCode`、`DiagnosticClass`、`FrontendError` を定義する。`source` → `preprocess` → `lexical_env` → `lexing` → `parsing` を配線し、インポート事前走査・字句環境・スコープスケルトン・トークン化の診断を含むすべてのフェーズ診断を、[orchestration.md](./orchestration.md)「Diagnostic Merge Order」の決定的な順序へ統合する。
    - `FrontendDiagnostic` は、code、message、class、`DiagnosticLocation`、副次の `SourceAnchor`、任意の回復ノートを持つ。範囲付き診断は `DiagnosticLocation::SourceRange` を使う。ソース読み込み診断は、利用可能な path、正規化パス、オープンバッファ URI、生成アンカー、または `Unknown` を持つ `DiagnosticLocation::SourceLoad` を使う。`FrontendError` は、ソース読み込み・スパン橋渡し・字句環境の回復不能な失敗を区別する。
    - テスト: `StubParserSeam` では、整形式のソースが、source、前処理出力、tokens、`ast = None`、パーサー診断なしを返す。統合順序が繰り返し実行で同一であり、同じ class / start / diagnostic code を持つ診断も決定的に並ぶ。実 parser seam では、`ast = Some` と構文診断順序の検証を追加する。
    - 依存: 9、10。実 parser の検証は 12 に依存する。仕様: [orchestration.md](./orchestration.md)「Algorithm / Logic」「Diagnostic Merge Order」。

14. **回復不能な失敗の処理とエンドツーエンド出力。** [ ]
    - `FrontendError` を、Step 1 の読み込み失敗、ソース登録／前処理／字句解析からの `SpanBridgeError` 不変条件違反、および字句環境構築からの `FrontendLexicalEnvironmentError` に対して返す。回復可能な問題は `FrontendOutput` 内の診断として保つ。
    - テスト: Step 1 の読み込み失敗が、ファイルレベルの `DiagnosticLocation::SourceLoad` 診断を伴う `FrontendError` を返し、出力は返さない。ソース読み込み診断はゼロ長の `SourceRange` を捏造しない。`ast = None` を返すパーサーの継ぎ目が、先行する診断を保持する。範囲付きの統合診断が妥当な `SourceRange` を運ぶ。
    - 依存: 13。仕様: [orchestration.md](./orchestration.md)「Error Handling」。

### 横断的フォローアップの前のモジュール全体の保守

15. **実装のリファクタリングパス。** [ ]
    - 最初の実装パスの完了後に、`span_bridge`、`source`、`preprocess`、`lexical_env`、`lexing`、`parsing`、`orchestration` を見直す。
    - 明白なバグや仕様との不一致が露見しない限り、公開 API と挙動は安定に保つ。小さな局所的抽出と共有テストフィクスチャを優先する。
    - テスト: すべてのモジュールテストを成功状態に保つ。
    - 依存: 14。仕様: すべての mizar-frontend モジュール仕様。

16. **ソース／仕様の対応監査。** [ ]
    - フロントエンド仕様の各公開 API、エラー変種、タスク要件から、実装ソース／テストへの軽量なトレーサビリティ確認を構築する。
    - 欠落した実装、古くなった仕様文、欠落したテストを、監査に広範な変更を混ぜずにフォローアップタスクとして記録する。
    - 英語の正本仕様を先に確認し、次に日本語版が同じ API と挙動の約束を保っているか検証する。
    - 依存: 15。仕様: すべての mizar-frontend モジュール仕様と本 TODO。

## 横断的フォローアップタスク

17. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-frontend/en/` の各英語正本ドキュメントを `doc/design/mizar-frontend/ja/` の日本語版と比較し、API 一覧、タスク状態、用語、リンクを同期する。
    - 依存: 16。仕様: リポジトリのドキュメント方針。

18. **フロントエンドの決定性プロパティテスト。** [ ]
    - 同一入力が、内部スケジューリングと独立に同一の `FrontendOutput` 診断順序と同一のトークンスパンを生むこと、および `LexicalEnvironmentFingerprint` とキャッシュキーが等価な入力に対して安定であることについて、crate レベルの網羅を追加する。
    - 依存: 16。仕様: [orchestration.md](./orchestration.md)、[lexical_env.md](./lexical_env.md)。

19. **インクリメンタルキャッシュキーの配線。** [ ]
    - [architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)「Incrementality」の層状フロントエンドキャッシュキーを、どこで計算・保存するか（本 crate か、ドライバ／成果物層か）を決め、成果物単位のキー（`SourceUnit`、`PreprocessedSource`、`ActiveLexicalEnvironment`、`TokenStream`、`SurfaceAst`）を公開する。
    - コメントのみの編集が意味的出力を再利用でき、インポート／依存エクスポートの編集およびパーサー字句文脈／パーサー支援字句解析プランの変更が、トークン化と下流層を無効化することを検証する。
    - 依存: 16。仕様: アーキテクチャのインクリメンタリティ表。

20. **パーサー支援字句解析契約の確定。** [ ]
    - 位置別の `ParserLexContext` を事前計算して 1 パスで曖昧性解消するか、狭い文脈オブジェクトを通じて構文解析と交錯させるかを確定し、選択した統合方式を [lexing.md](./lexing.md) と [parsing.md](./parsing.md) に記録する。
    - いずれの選択でも、字句解析器を任意のパーサー状態から自由に保つ。このタスクは、文法位置の文字列必須判定または記号種別フィルタを必要とする実 source-to-token-to-parser テスト、位置別の `StringLiteral` テスト、注釈の文字列引数内での Unicode 受理をブロックする。タスク 20 までは、タスク 11〜12 は必要に応じて合成パーサートークンストリームを使う。
    - 依存: 10。実 parser の検証は 11 にも依存する。仕様: トップレベルの [../../todo.md](../../todo.md)「Resolved And Open Decisions」、[lexing.md](./lexing.md)、[parsing.md](./parsing.md)。

21. **恒久的な lint 強制。** [ ]
    - `crates/mizar-frontend/Cargo.toml` がワークスペースの `[workspace.lints]` テーブルにオプトインし、`cargo build`/`cargo test` が、単独の clippy ゲートと同じ拒否を表面化することを確認する（`mizar-session` の方針と同様）。
    - 意図的な `allow` 例外は、`allow` の隣に根拠とともに記録する。
    - テスト: `cargo clippy -p mizar-frontend --all-targets -- -D warnings` が通る。
    - 依存: 16。仕様: 本 TODO「推奨検証」。

22. **精密な生スキャン回復契約。** [ ]
    - `mizar-lexer` が、失敗スパンと部分的な生トークンを返す回復可能な生スキャナーを公開するか、`mizar-frontend` が厳密な `scan_raw` の失敗に対して字句テキスト全体の粗い回復だけを維持するかを決める。
    - 回復可能な生スキャナーを追加する場合は、[preprocess.md](./preprocess.md) と [lexing.md](./lexing.md) を更新し、粗い診断／回復トークンを、精密な失敗スパンと同期境界からの継続に置き換える。
    - テスト: この契約が入るまでは、厳密な `scan_raw` の失敗が粗い回復のままであることを確認する。契約後は、インポート事前走査とトークン化が、問題箇所の正確なスパンを報告し、利用可能な部分的生トークンを保持することを確認する。
    - 依存: 9。仕様: [preprocess.md](./preprocess.md)、[lexing.md](./lexing.md)。

## 推奨検証

各タスクの後に実行する。

```text
cargo test -p mizar-frontend
cargo clippy -p mizar-frontend --all-targets -- -D warnings
```

字句解析器／session 境界に触れるタスクでは、次も実行する。

```text
cargo test -p mizar-session
cargo test -p mizar-lexer
```

テストが通ったら、ここでタスクにチェックを入れる。

## 注記

- `mizar-frontend` は統制を担う crate である。`mizar-session`、`mizar-lexer`、および実 parser seam が有効になった後の `mizar-syntax` / `mizar-parser` を統制するが、それらの中核アルゴリズムやデータ定義は何も所有しない。
- `mizar-lexer` を `mizar-session` から分離したまま保つ。字句解析器スパンから session の `SourceRange` への橋渡しは `span_bridge` にのみ存在する。
- フロントエンドは構文を生成し、意味は生成しない。名前解決、型検査、オーバーロード選択、証明義務はここに属さない。
- フロントエンド成果物（`SourceUnit`、`PreprocessedSource`、`TokenStream`、`SurfaceAst`、`FrontendOutput`）は内部コンパイラデータであり、安定した外部スキーマではない。
