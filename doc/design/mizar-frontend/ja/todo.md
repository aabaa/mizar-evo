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
| preprocess | [preprocess.md](./preprocess.md) | `src/preprocess.rs` | [ ] |
| lexical_env | [lexical_env.md](./lexical_env.md) | `src/lexical_env.rs` | [ ] |
| lexing | [lexing.md](./lexing.md) | `src/lexing.rs` | [ ] |
| parsing | [parsing.md](./parsing.md) | `src/parsing.rs` | [ ] |
| orchestration | [orchestration.md](./orchestration.md) | `src/orchestration.rs` | [ ] |

`mizar-frontend` は統制 crate なので、フェーズ順にボトムアップで構築する。まず座標橋渡し、次にパイプライン順で Step 1-5、最後にエンドツーエンドの coordinator である。`span_bridge` は後続の各フェーズが参照する共有プリミティブであり、`orchestration` は完全なパイプラインを配線する唯一のモジュールである。

依存順序: `span_bridge` → `source` → `preprocess` → `lexical_env` → `lexing` → `parsing` → `orchestration`。

## crate 前提条件

frontend の基盤は `mizar-session` と `mizar-lexer` に依存する。`mizar-syntax` や `mizar-parser` はまだ未実装であるため（トップレベル [../../todo.md](../../todo.md) は両者を未着手としている）、real parser seam のタスクが landing するまで hard dependency を追加しない。タスク 1-10 と、タスク 13-14 の stub coordinator 部分は `mizar-session` と `mizar-lexer` だけで実装できる。タスク 11-12 の real parser 呼び出しと syntax AST についての assertion、およびタスク 13-14 の real parser assertion は、最小限の `mizar-parser` エントリポイントと `mizar-syntax::SurfaceAst` に gate される。

## 解決済みおよび gate された決定

これらの公開 API 決定は、[../../todo.md](../../todo.md) の「Resolved And Open Decisions」でトップレベル管理されている。

- **字句解析器スパン橋渡し: 解決済み。** この crate は分離オプションを採用する。`mizar-lexer` はバイトオフセットスパンを保持し、`span_bridge`（タスク 1）がそれらを `mizar-session` `SourceRange` へ変換する。
- **パーサー支援字句解析契約: gate 中。** 現在の lexer は uniform な `ParserLexContext` を公開しており、位置別の文字列必須 span はまだ表現しない。位置別 `StringLiteral` 認識、注釈文字列引数内 Unicode の受理、parser-driven symbol-kind filter は、任意の parser state を晒さない狭い `ParserLexContext` / `ParserInputs` 契約に gate される。

## 順序付きタスク一覧

各タスクは単独で実装・テスト・コミットできる大きさである。各タスクの依存行を正とする。`mizar-parser` / `mizar-syntax` が未提供のときは、gate された real parser タスクを飛ばし、それらに依存しない stubbed source → tokens coordinator タスクを進める。各タスクは `cargo test -p mizar-frontend` を green に保つこと（[推奨検証](#推奨検証)を参照）。

### crate の足場

1. **crate 骨格と座標橋渡し。** [x]
   - `mizar-frontend` crate をワークスペースに追加し、`mizar-session` と `mizar-lexer` に依存させる。ワークスペースの `[workspace.lints]` テーブルに `lints.workspace = true` でオプトインする（`mizar-session` と同様）。
   - `pub mod span_bridge;` を追加し、`SpanBridge`、`LexerByteSpan`、`SpanBridgeError` を定義する。`SpanBridge` は retained session source-map service を所有し、fallible な `register_source` / `register_preprocess_map`、および retained `mizar-session` map 上の `loaded_span`、`loaded_mapping`、`lexical_span` 変換を提供する。
   - `SpanBridgeError` は frontend-local な登録不変条件（`SourceNotRegistered`、`PreprocessMapNotRegistered`、source / preprocess 登録衝突）と、wrapped `mizar-session::SourceMapError` を区別する。
   - `SourceUnit` に付随する任意の session `LoadingMap` を再利用し、session 側 `PreprocessMap` を字句解析器の `SourcePreprocessMap` から導出する。identity load に対して `LoadingMap::identity` を合成・保持しない。`loaded_mapping` が loading map を見つけない場合、line map 検証後に `original_input = None` の exact な読み込み済み座標 mapping を返す。
   - テスト: BOM 除去テキスト上の `loaded_span` は読み込み済みテキスト座標のままで、`loaded_mapping` は `MappedSourceRange.original_input` を通じて元入力オフセットを報告する。字句スパンが preprocess map を通じて読み込み済みソース座標へ変換される。`LoadingMap` を持たない identity load は `original_input = None` の exact `loaded_mapping` を返す。除去コメントをまたぐスパンが第一 + 隣接アンカーを生む。synthetic-only lexical span は exact primary source range ではなく degraded anchor-backed mapping を生む。UTF-8 境界外および範囲外のスパンが拒否される。衝突するマップ登録が報告される。
   - 仕様: [span_bridge.md](./span_bridge.md)「Public API」「Algorithm / Logic」。

### モジュール: source (`src/source.rs`)

2. **`SourceUnit` とローダー橋渡し。** [x]
   - `pub mod source;` を追加する。`SourceUnit`、`SourceUnitRequest`、`SourceUnitLoader` トレイト、`FrontendSourceLoader<L: SourceLoader>` を定義し、`mizar_session::LoadedSource` をハッシュ・line map・loading map・normalized path・edition・origin・generated anchor を再計算せずに `SourceUnit` へ射影する `source_unit_from_loaded` を実装する。
   - `LoadedSource` はファイルシステムパスを保持しないので、`file_path` を呼び出し側提供の診断メタデータとして扱う。ディスク／オープンバッファ source では request/origin から、生成 source では normalized path または generated anchor から導出する。
   - 読み込んだ `LineMap` / `LoadingMap` を `SourceId` の下で mutable `SpanBridge` に登録する helper を提供する。統制層が読み込み後・前処理前にこの helper を呼び、`load_source_unit` 自体は bridge 状態を変更しない。
   - テスト: ディスク `LoadedSource` が同一の id／hash／line-map／loading-map で射影される。BOM／CRLF 正規化ソースは `Some(loading_map)` を運ぶ。恒等読み込みは `None` を運ぶ。normalized path と edition が保持される。オープンバッファ origin とバージョンが保持される。生成ソースは `generated_anchor` を保持する。`register_source_unit` が line/loading map を記録し、衝突する重複登録を報告する。session `SourceLoadError` がそのまま伝播する。
   - 依存: 1。仕様: [source.md](./source.md)「Public API」「Algorithm / Logic」。

### モジュール: preprocess (`src/preprocess.rs`)

3. **コメントとドキュメントコメントの前処理。** [ ]
   - `pub mod preprocess;` を追加する。`PreprocessedSource`、`LexicalText`、`Comment`、`DocComment`、`LexicalSourceMap`、`lexical_hash`、mapped `ImportStub` / `ImportStubPath` / `ImportStubRelativePrefix` / `ImportStubAlias`、および code/message、primary `SourceRange`、secondary `SourceAnchor` を持つ `PreprocessDiagnostic` を定義する。
   - `SourceUnit.source_text` に対して `mizar_lexer::preprocess_source_for_lexing` を駆動し、コメント／ドキュメントコメント／前処理診断のスパンを `SpanBridge` を通じて変換し、`LexicalSourceMap` を組み立てる。
   - テスト: 通常コメントが字句テキストから除去されるが正しい範囲を持つ `Comment` として保持される。ドキュメントコメントが生本文と範囲とともに保持される。注釈構文が字句テキストに残る。除去コメントをまたぐ字句範囲が合成マッピングを生む。synthetic whitespace は degraded anchor-backed mapping としてだけ表現される。コメントのみの編集で字句テキストが不変なら `lexical_hash` が安定する。コード領域の非 ASCII 文字と未終端ブロックコメントが報告・回復される。
   - 依存: 2。仕様: [preprocess.md](./preprocess.md)「Comments and Doc Comments」「Algorithm / Logic」。

4. **浅いインポート事前走査の統合。** [ ]
   - 字句テキストを生スキャン（`scan_raw`）し `mizar_lexer::scan_import_prelude` を実行する。変換済み `SourceRange` を持つ `import_stubs` を満たし、`ImportPrescanDiagnostic` を `diagnostics` に集める。
   - strict raw scan が失敗した場合は、字句テキスト全体（空ならソース先頭のゼロ長範囲）を覆う frontend-local なインポート事前走査診断を記録し、`import_stubs` を空にして、部分的な raw text から import を推測せず続行する。recoverable raw-scanner contract が存在するまでは、`mizar_lexer::LexError` が span を持つと仮定しない。
   - テスト: トップレベル `import` 形式が生パス・任意の alias・`path.relative`・`path.source_segments`・span を持つ `ImportStub` を生む。`./` と `../` の相対 prefix が区別されたまま保持される。不正なインポートが中断せずインポート事前走査診断を生む。インポート事前走査中の raw-scan 失敗が粗い診断と空の `import_stubs` を生む。provenance と決定的 fingerprint のためにインポート順が保持される。
   - 依存: 3。仕様: [preprocess.md](./preprocess.md)「Import Stubs」「Error Handling」。

### モジュール: lexical_env (`src/lexical_env.rs`)

5. **字句環境リクエストとプロバイダの継ぎ目。** [ ]
   - `pub mod lexical_env;` を追加する。`LexicalEnvironmentRequest`、`LexicalSummaryProvider`、`ResolvedImports`、`ResolvedImportEntry`、`ActiveLexicalEnvironmentResult`、`LexicalEnvironmentDiagnostic`、`LexicalEnvironmentDiagnosticCode`、`FrontendLexicalEnvironmentError` を定義し、`mizar-lexer` の環境型を再エクスポートする。
   - 各 resolved import の元 `ImportStub` ordinal と span を `ResolvedImportEntry` に保持する。`mizar_lexer::build_lexical_environment` へは順序付き lexer `ResolvedImport` だけを渡す。
   - lexer 呼び出し前に resolved import を `ModuleId` で canonicalize し、import 順で最初の stub を active provenance entry として保持する。duplicate-stub provenance は provider diagnostics 用に保持するが、現在の lexer には duplicate module id を渡さない。lexer の conflict error は module を識別するが frontend import ordinal を識別しないためである。
   - provider infrastructure failure には、lexer 所有の `LexicalEnvironmentError` ではなく `FrontendLexicalEnvironmentError` を使う。
   - テスト: 解決済みインポート + サマリを返す偽プロバイダが、正しい canonical provenance とインポート序数を持つ `UserSymbolIndex` を生む。同じ module へ解決される複数 stub は lexer 呼び出し前に deduplicate され、provider diagnostics は正しい元 import span に出る。予約テーブルが常に存在する。
   - 依存: 4。仕様: [lexical_env.md](./lexical_env.md)「Public API」。

6. **アクティブ字句環境の構築。** [ ]
   - `mizar_lexer::build_lexical_environment` を呼ぶ `build_active_lexical_environment` を実装する。プロバイダ診断を統合し、`LexicalEnvironmentFingerprint` を計算・表面化する。
   - lexer を呼ぶ前に、未解決 import と依存字句サマリが利用できない import を除外し、`LexicalEnvironmentDiagnostic` として表す。欠落 summary を持つ入力で lexer に環境構築を要求しない。
   - `LexicalEnvironmentError::UserSymbolImportConflict` は bounded deterministic retry で扱う。canonical `ResolvedImportEntry` provenance を使って後側の衝突 import を診断し、分かる場合は前側 import を secondary context に追加し、後側の衝突 module を除去して original canonical import ごとに高々 1 回 retry する。それ以外の lexer `LexicalEnvironmentError` は `FrontendLexicalEnvironmentError::MalformedSummary` として扱う。
   - テスト: 異なるモジュールからインポートされた同綴りユーザー記号は、決定的な字句環境衝突診断を生み、後側の衝突 module を落として retry する。同じ module の duplicate import は spurious conflict を作らない。未解決インポートが診断とともにより小さな環境へ縮退し、残りの記号が読み込まれる。欠落 summary が lexer 呼び出し前に除外・診断される。conflict 以外の lexer environment error は `MalformedSummary` になる。fingerprint が依存サマリの変更で変化し、ローカルのコメントのみの編集では安定である。
   - 依存: 5。仕様: [lexical_env.md](./lexical_env.md)「Algorithm / Logic」「Error Handling」。

### モジュール: lexing (`src/lexing.rs`)

7. **生スキャンとスコープスケルトンの配線。** [ ]
   - `pub mod lexing;` を追加する。`TokenizeRequest`、`InternedText = Arc<str>`、フロントエンドの `Token` / `TokenStream`（session スパン付き）、`LexingDiagnostic`、`LexingDiagnosticKind` / `LexingDiagnosticPayload` を定義し、`TokenKind` と `LexDiagnosticCode` / `ScopeSkeletonDiagnosticCode` などの raw lexer diagnostic code enum を再エクスポートする。
   - 生トークンから `ScopeSkeleton` / `ScopeLexView` を構築し、曖昧性解消器の入力を準備する。`ScopeSkeletonDiagnostic` は、raw span-bearing diagnostic struct を公開 `TokenStream` に保持せず `LexingDiagnostic` へ写像する。
   - テスト: 生スキャンが `LexemeRun` スパンを保持する。スコープビューが解決済み束縛なしで字句ブロック／文の形を反映する。
   - 依存: 6。仕様: [lexing.md](./lexing.md)「Scope Lex View」「Algorithm / Logic」。

8. **文脈依存曖昧性解消による `TokenStream`。** [ ]
   - アクティブ字句環境、スコープビュー、現在の `ParserLexContext`（parser-assisted contract が確定するまでは general/stub context）を与えて `disambiguate`（またはパーサー統合の `lex`）を実行する。各字句解析器トークンと診断のスパンを `SpanBridge` を通じて session `SourceRange` へ変換する。raw `LexDiagnostic` は code/message をコピーし、structured payload を mapped form で保持して frontend `LexingDiagnostic` へ変換する。rejected-candidate の入れ子 span は session range へ写像し、composite/degraded mapping 由来の secondary `SourceAnchor` を保持する。内部写像不変条件の失敗だけを `Err(SpanBridgeError)` として返す。
   - テスト: 識別子とつづりを共有するユーザー記号が最長一致で分類される。複合予約トークン（`.{`、`.*`、`.=`、`...`）が単一トークンとして字句解析される。引用符が general context では記号文字として字句解析され、bounded uniform `StringRequired` context では `StringLiteral` を生む。送出された各トークンスパンが妥当な第一 `SourceRange` へ解決され、隣接アンカーは診断用に保持される。rejected token candidate を持つ lexer payload は非 span payload data と mapped nested span を保持する。注釈／演算子位置の文字列リテラルテストは parser-assisted lexing contract まで延期する。
   - 依存: 7。仕様: [lexing.md](./lexing.md)「Token Stream」「Algorithm / Logic」。

9. **字句解析器回復のパススルー。** [ ]
   - `TokenKind::ErrorRecovery` スパンと lexer 診断を mapped `LexingDiagnostic` としてエンドツーエンドで保持する。strict raw-scan のハードエラーは粗い回復トークン 1 つと粗い `RawScan` 診断 1 つへ適合させ、frontend `tokenize` wrapper が回復可能入力問題に対して `Ok(TokenStream)` を返すことを保証する。
   - テスト: 不正なトークンが正しい `SourceRange` を持つ `ErrorRecovery` を送出しスキャンが再開する。不正な数値と文字列必須位置での不正な文字列リテラルが、回復可能トークンを落とさずに報告される。スコープスケルトン診断が mapped span とともに保持される。strict raw-scan 失敗は、現在の `LexError` が span / partial-token payload を持たないため粗い回復を生む。
   - 依存: 8。仕様: [lexing.md](./lexing.md)「Error Handling」。

### モジュール: parsing (`src/parsing.rs`)

10. **パーサー入力の組み立てとパーサーの継ぎ目。** [ ]
    - `pub mod parsing;` を追加する。`ParseRequest`、`ParserInputs`、`OperatorFixityTable`、`OperatorFixityEntry`、`OperatorAssociativity`、`StringRequiredContext`、`ParseOutput`、`ParserSeam`、`StubParserSeam` を定義し、アクティブ字句環境の構築後に、source edition と字句サマリが現在公開しているデータだけを使って `ParserInputs` を導出する。
    - `mizar-parser` が存在するまで、継ぎ目を `ast = None` と空の診断リストを返すスタブに対して実装し、ソース → トークンのパイプラインを実行可能にする。
    - テスト: `ParserInputs` は edition を運び、サマリが fixity を公開していないときは空の operator-fixity table を使い、stub source-to-token path では `StringRequiredContext::None` を使い、解決器の状態を運ばない。スタブ継ぎ目が `ast = None` を返す。
    - 依存: 8。仕様: [parsing.md](./parsing.md)「Parser Inputs」「Public API」。

11. **`mizar-parser` の呼び出し。** [ ]
    - スタブ継ぎ目を実際の `mizar-parser` エントリポイントに置き換える。`mizar-syntax::SurfaceAst` と構文診断をそのまま返す。
    - 最小限の `mizar-parser` / `mizar-syntax` を必要とする（トップレベル [../../todo.md](../../todo.md)）。それらの可用性でゲートする。
    - テスト: 整形式トークンストリームがソース順と範囲を保持した `SurfaceAst` へ解析される。サマリが fixity を公開したら、演算子結合性がユーザー中置演算子に対する正しい Pratt 優先順位を駆動する。注釈／演算子の文字列リテラルテストは、task 20 が real source text 向け parser-assisted lexing を確定するまで synthetic parser token stream を使う。
    - 依存: 10、加えて `mizar-parser`/`mizar-syntax`。文法位置の文字列リテラルを必要とする real source-text tests は 20 にも依存する。仕様: [parsing.md](./parsing.md)「Algorithm / Logic」。

12. **パーサー回復のパススルー。** [ ]
    - 回復不能な入力での `ast = None` と、返された `SurfaceAst` 内の明示的な回復ノード印を保持する。構文診断を通す。
    - テスト: `end` の欠落が同期点で回復し `ast = Some` と明示的エラーノードを生む。回復不能なストリームが診断とともに `ast = None` を返す。文字列必須位置での文字列リテラルの欠落は、task 20 が real source text 向け parser-assisted lexing を確定するまで synthetic token stream で期待される構文診断を確認する。
    - 依存: 11。仕様: [parsing.md](./parsing.md)「Error Handling」。

### モジュール: orchestration (`src/orchestration.rs`)

13. **フロントエンド coordinator と診断統合。** [ ]
    - `pub mod orchestration;` を追加する。`FrontendOutput`、`Frontend`、`FrontendDiagnostic`、`DiagnosticLocation`、`SourceLoadLocation`、`DiagnosticCode`、`DiagnosticClass`、`FrontendError` を定義する。`source` → `preprocess` → `lexical_env` → `lexing` → `parsing` を配線し、インポート事前走査・字句環境・スコープスケルトン・トークン化診断を含むすべてのフェーズ診断を [orchestration.md](./orchestration.md)「Diagnostic Merge Order」の決定的順序へ統合する。
    - `FrontendDiagnostic` は code、message、class、`DiagnosticLocation`、secondary `SourceAnchor`、任意の recovery note を持つ。range-backed 診断は `DiagnosticLocation::SourceRange` を使う。source-load 診断は利用可能な path、normalized path、open-buffer URI、generated anchor、または `Unknown` を持つ `DiagnosticLocation::SourceLoad` を使う。`FrontendError` は source-load、span-bridge、lexical-environment の hard failure を区別する。
    - テスト: `StubParserSeam` では、整形式ソースが source、preprocessing output、tokens、`ast = None`、parser 診断なしを返す。統合順序が繰り返し実行で同一であり、同じ class / start / diagnostic code を持つ診断も決定的に並ぶ。実 parser seam では `ast = Some` と構文診断順序の assertion を追加する。
    - 依存: 9、10。real-parser assertion は 12 に依存する。仕様: [orchestration.md](./orchestration.md)「Algorithm / Logic」「Diagnostic Merge Order」。

14. **回復不能な失敗の処理とエンドツーエンド出力。** [ ]
    - `FrontendError` を Step 1 読み込み失敗、source 登録／前処理／字句解析からの `SpanBridgeError` 不変条件違反、および字句環境構築からの `FrontendLexicalEnvironmentError` に対して返す。回復可能な問題は `FrontendOutput` 内の診断として保つ。
    - テスト: Step 1 読み込み失敗が file-level `DiagnosticLocation::SourceLoad` 診断を伴う `FrontendError` を返し出力なし。source-load 診断はゼロ長 `SourceRange` を捏造しない。`ast = None` を返す parser seam が先行診断を保持する。range-backed な統合診断が妥当な `SourceRange` を運ぶ。
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
    - コメントのみの編集が意味的出力を再利用でき、インポート／依存エクスポート編集および parser lexing context / parser-assisted lexing plan の変更がトークン化と下流層を無効化することを検証する。
    - 依存: 16。仕様: アーキテクチャのインクリメンタリティ表。

20. **パーサー支援字句解析契約の確定。** [ ]
    - 位置別 `ParserLexContext` を事前計算して 1 パスで曖昧性解消するか、狭い文脈オブジェクトを通じて構文解析と交錯させるかを確定し、選択した統合を [lexing.md](./lexing.md) と [parsing.md](./parsing.md) に記録する。
    - いずれの選択でも字句解析器を任意のパーサー状態から自由に保つ。このタスクは、文法位置の文字列必須判定または symbol-kind filter を必要とする real source-to-token-to-parser tests、位置別 `StringLiteral` テスト、注釈文字列引数内 Unicode の受理をブロックする。task 20 までは、タスク 11-12 は必要に応じて synthetic parser token stream を使う。
    - 依存: 10。real-parser validation は 11 にも依存する。仕様: トップレベル [../../todo.md](../../todo.md)「Resolved And Open Decisions」、[lexing.md](./lexing.md)、[parsing.md](./parsing.md)。

21. **恒久的な lint 強制。** [ ]
    - `crates/mizar-frontend/Cargo.toml` がワークスペースの `[workspace.lints]` テーブルにオプトインし、`cargo build`/`cargo test` が単独 clippy ゲートと同じ拒否を表面化することを確認する（`mizar-session` 方針と同様）。
    - 意図的な `allow` 例外は `allow` の隣に根拠とともに記録する。
    - テスト: `cargo clippy -p mizar-frontend --all-targets -- -D warnings` が通る。
    - 依存: 16。仕様: 本 TODO「推奨検証」。

22. **精密 raw-scan 回復契約。** [ ]
    - `mizar-lexer` が失敗 span と部分 raw token を返す recoverable raw scanner を公開するか、`mizar-frontend` が strict `scan_raw` 失敗に対して粗い字句テキスト全体回復だけを維持するかを決める。
    - recoverable raw scanner を追加する場合は、[preprocess.md](./preprocess.md) と [lexing.md](./lexing.md) を更新し、粗い診断／回復トークンを精密な失敗 span と同期境界からの継続に置き換える。
    - テスト: この契約が入るまでは strict `scan_raw` 失敗が粗い回復のままであることを確認する。契約後は、import pre-scan と tokenization が問題箇所の正確な span を報告し、利用可能な部分 raw token を保持することを確認する。
    - 依存: 9。仕様: [preprocess.md](./preprocess.md)、[lexing.md](./lexing.md)。

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

- `mizar-frontend` は統制 crate である。`mizar-session`、`mizar-lexer`、および実 parser seam が有効になった後の `mizar-syntax` / `mizar-parser` を統制するが、それらの中核アルゴリズムやデータ定義は何も所有しない。
- `mizar-lexer` を `mizar-session` から分離したまま保つ。字句解析器スパン → session `SourceRange` の橋渡しは `span_bridge` にのみ存在する。
- フロントエンドは構文を生成し、意味は生成しない。名前解決、型検査、オーバーロード選択、証明義務はここに属さない。
- フロントエンド成果物（`SourceUnit`、`PreprocessedSource`、`TokenStream`、`SurfaceAst`、`FrontendOutput`）は内部コンパイラデータであり、安定した外部スキーマではない。
