# Module: raw_lexer

> Canonical language: English. English canonical version: [../en/raw_lexer.md](../en/raw_lexer.md).

## Purpose

このモジュールは、Mizar Evo における字句解析器(lexer)の責務境界を定義します。

Mizar の字句分類(lexical classification)は文脈に依存します。インポートされたモジュールはユーザー定義シンボルを増やし、ユーザーシンボルは識別子と同じ形を取り得ます。さらに、局所的な識別子束縛がシンボル解釈を上書き(override)することもあります。そのため字句解析器は、すべての `Identifier` / `UserSymbol` の分類を、文脈自由(context-free)な単一パスで恒久的に確定する設計にしてはいけません。

現在の実装は、低レベルの生スキャナー(raw scanner)と、より高レベルな曖昧性解消(disambiguation)のエントリポイントの両方を公開しています。この文書では、利便用関数 `lex(&str)` のシェルを完全な文脈依存の字句解析器と誤解しないよう、各層の責務境界を明確にします。

## Public API Stability

`mizar-lexer` は現在 `0.1` クレートです。公開データ構造はパーサー向けの転送オブジェクトとして扱い、初期段階のパーサー・コーパス・統合コードが直接調べたり構築したりできるよう、フィールドを可視のままにします。

公開列挙型(enum)には `#[non_exhaustive]` を付けます。下流のクレートは、トークン種別、生トークン種別、診断コード、パーサーモード、インポート事前スキャンのカテゴリ、スコープスケルトンのカテゴリ、ソース前処理のカテゴリ、字句環境のエラーを照合するとき、ワイルドカードのアーム(arm)を含める必要があります。これにより、パーサー向け API が成熟するまでカテゴリを追加できる余地を保ちます。

次のパーサー統合向け `0.x` マイルストーンでは、下流のクレートは以下の互換性境界に依存してかまいません。

| Surface | 次のパーサーマイルストーンでの互換性の約束 |
|---|---|
| エントリポイント | `load_source_text_from_bytes`, `preprocess_source_for_lexing`, `scan_raw`, `scan_import_prelude`, `build_lexical_environment`, `build_scope_skeleton`, `lex`, `disambiguate` は、実行可能な字句解析器の受け渡し経路として維持します。 |
| 座標 | `SourceSpan` は、生成元の段階へ渡された正確なテキスト内の半開バイト範囲のままです。フィールド構築が不要な下流コードでは、`SourceSpan::new`, `try_new`, `len`, `is_empty`, `is_valid`, `contains` を安定したヘルパーとして優先します。`SourceSpan::new` は逆順の範囲を拒否します。外部の呼び出し側が可視フィールドから構築した不正なスパンは、防御的な境界 API が引き続き拒否します。 |
| トークンアクセス | `RawToken`, `RawTokenStream`, `Token`, `TokenStream`, `LexDiagnostic` は、当面パーサー向けフィールドを可視のままにします。ただし、可能な場合はコンストラクタとアクセサーメソッドを優先します。所有権を取り出す場合は、回復可能な診断を誤って落とさないよう `TokenStream::into_parts` を使います。これにより、読み取り専用のパーサー経路を変えずに、後でキャッシュ済みメタデータやラッパーを追加する余地を残します。 |
| ID とフィンガープリント | `ModuleId`, `SymbolId`, `ExportRank`, `LexicalSummaryFingerprint`, `LexicalEnvironmentFingerprint` は軽量な newtype のままです。新しい統合コードでは、タプルフィールドへの直接アクセスより `new`, `as_str`, `get` を優先します。フィンガープリントの具体的なアルゴリズムは、字句サマリー形状が変わる `0.x` リリース間ではまだ変わり得ます。 |
| 診断 | 診断コードとバイトスパンを互換性の対象にします。人間向けの `message` テキストは暫定であり、その文言を明示的に所有する fixture でない限り、下流ツールはスナップショットテストで固定すべきではありません。 |
| 可視の転送フィールド | 既存の可視フィールドは、このマイルストーンではパーサーとコーパス統合を単純に保つため維持します。ただし、すべてのフィールドが永久に構築可能であり続けるという約束ではありません。より強い安定性が必要なフィールドには、先にコンストラクタまたはアクセサーを追加します。 |

明示的な安定化マイルストーンを後で設けるまでは、`0.1` のマイナーリリースでも、字句解析器の境界の一貫性を保つために、公開フィールド・コンストラクタ・ヘルパー関数へ破壊的変更を加える可能性があります。

## Source Preconditions

`mizar-lexer` に渡される入力は、生のファイルバイト列ではありません。

このクレートの外側にあるソース読み込み層は、以下を担当します。

- ファイルの読み込み;
- UTF-8 の検証;
- パッケージが用意したソーステキストの先頭 UTF-8 BOM を、字句解析器に入る前に受け入れて取り除くこと;
- スキャナーのエントリポイントに渡す前の、プラットフォーム改行から LF のみのテキストへの正規化;
- 必要に応じた、元のファイルオフセットへのソースマップの保持;
- パッケージ内でソースファイルをどのように見つけるかの決定。

一方、`mizar-lexer` は字句の境界向けのソース前処理ヘルパーを提供します。

- テストや初期統合コードが実行可能なソース読み込みの境界を必要とする場合、クレートローカルの `load_source_text_from_bytes` ヘルパーで UTF-8 のソースバイト列を検証する;
- そのヘルパーで先頭 UTF-8 BOM を 1 つ取り除き、取り除いた後の読み込み済みテキストのオフセットから元のバイトオフセットへの最小限の読み込みマップを記録する;
- そのヘルパーで、字句解析器に入る前に CRLF の改行対を LF に正規化し、元のバイト範囲への正規化改行マップのセグメントを記録する;
- 通常コメント・ドキュメンテーションコメント・複数行コメントを字句入力から取り除く;
- コメントのトリビア(trivia)をソーススパンとともに保持する;
- コメント内の改行だけを残し、行構造を崩さないようにする;
- コメント除去によって隣接するトークンの形をしたテキストが連結してしまう場合は、合成のレイアウト(空白)を挿入する;
- キャリッジリターン、コード領域内の非 ASCII テキスト、閉じていない複数行コメントを、前処理の診断として報告する;
- 必要に応じて、パッケージを起点とする `.miz` ソース名を検証する。

`mizar-lexer` は、レイアウトが以下のみであると仮定できます。

```text
space, tab, newline
```

キャリッジリターンは、この層ではレイアウトではありません。ソース読み込みは、スキャナーに入る前に CRLF 対を LF に正規化します。単独の `\r` が字句解析器に届いた場合、それはソース読み込み側の不備、プラットフォーム改行ではない不正な入力、または意図的に不正にしたテストフィクスチャのいずれかです。

先頭 UTF-8 BOM も、字句解析器の機能ではなくソース読み込みの責務です。ディスク入力はバイト列 `EF BB BF` を含んでよく、パッケージが用意した開きバッファのテキストは、対応する先頭の `U+FEFF` を含んでよいです。ソースローダーは、`LoadedSource.text` を構築する前、または字句解析器のエントリポイントを呼ぶ前に、その先頭の符号(signature)だけを取り除きます。`preprocess_source_for_lexing` または `scan_raw` に届いた `U+FEFF` は、先頭以外のものを含めて、字句境界における不正な入力のままであり、この層で黙って捨ててはいけません。

## Source-Text Normalization Policy

`mizar-lexer` は Unicode 正規化を行いません。字句の綴り規則を判定する前に、コードテキストへ正準正規化や互換正規化を適用することはありません。

この層では、コード領域の識別子・数字・予約語・予約記号・ユーザーシンボルの綴りは ASCII のみです。コード領域に届いた非 ASCII テキストは、字句境界における不正な入力です。前処理はそれを `NonAsciiCode` として報告し、直接の生スキャンは未対応の文字を拒否します。ASCII の綴りへ変換して受け入れることはありません。

コメントとドキュメンテーションコメントは別扱いです。そのテキストは、ソーススパン付きの生の Unicode トリビアとして保持されます。ただし、上記のコメント除去規則に従い、改行の構造は `lexical_text` に残します。字句解析器は、コメント/ドキュメンテーションテキスト内の Unicode を正規化せず、警告も拒否もしません。将来のドキュメンテーション・ソース読み込み・診断の方針は、字句解析器のトークン化を変更せずに、紛らわしい Unicode・誤認しやすい文字(confusables)・正規化に敏感なテキストへの警告を追加できます。

## Core Design

字句解析は、概念的に 2 段階に分けます。

## 実装上のアルゴリズムの流れ

現在のクレートは、利便用関数 `lex` を使う場合でも、ソースの準備・生スキャン・最終曖昧性解消を分けて扱います。

1. `preprocess_source_for_lexing` は入力を先頭から順に走査します。コメントは字句テキストから取り除きますが、コメント内の改行は残して行位置が崩れないようにし、除去によって隣接するトークンの形をしたテキストが連結してしまう場合は合成レイアウトを挿入します。コメント本体はソーススパン付きのトリビアとして保持し、字句範囲から読み込み済みソース範囲へ戻す軽量な前処理マップも記録します。`\r`、コード領域内の非 ASCII 文字、閉じていない複数行コメントは前処理の診断として報告します。複数行コメントは入れ子になりません。`::=` の開始記号の後に最初に現れる `=::` がコメントを閉じ、内部の `::=` の綴りは通常のコメントテキストとして扱います。このヘルパーは、ファイルの読み込みや OS ごとの改行正規化は担当しません。
2. `scan_raw` は、LF のみの字句テキストを `char_indices` カーソルで読みます。連続するレイアウトを 1 個の `Layout` にまとめ、`@` から始まる注釈マーカー(annotation marker)を認識し、`@` 以外の ASCII 図形文字を連続したランとしてまとめます。そのランがすべて数字なら `NumeralLike`、そうでなければ `LexemeRun` です。対応していない文字は `LexError` になります。
3. `disambiguate_reserved_shell` は、`lex` が使う文脈自由な薄いシェルです。レイアウトを捨て、`NumeralLike` を `Numeral` にし、`@[` を予約記号にします。`LexemeRun` 全体については、予約記号・予約語・識別子、または不透明な `LexemeRun` として分類します。
4. インポート・パーサーコンテキスト・スコープによる上書きが分類に影響する場合は `disambiguate` を使います。この経路では生スキャンをあえて粗いままに保ち、`LexemeRun` の内部分割は、曖昧性解消器が予約テーブル・アクティブ字句環境・パーサーの字句コンテキスト・`ScopeLexView` を見て行います。
5. `module_source_name_from_path` は、スキャナーではなくソース境界のヘルパーです。パッケージ名を検証し、`.miz` ファイルが `src` ルート配下にあること、ソースルートがパッケージ名と一致すること、パス区切りの違いを吸収できること、名前空間の構成要素がすべて識別子の形であることを確認します。

生スキャナーの重要な不変条件は、スパンの連続性です。出力された生トークンは必ず元入力の正確なバイト列を指し、生トークンの字句単位(lexeme)を連結するとスキャナーの入力が復元できます。

### Source Coordinates

`SourceSpan` は、`mizar-lexer` 内部の正準的な座標型です。これは、トークンまたは診断を生成した正確なテキストに対するバイトオフセットを保持し、半開区間 `[start, end)` を表します。

呼び出し側は、座標空間を明示的に扱わなければなりません。`scan_raw` と `disambiguate` から生成される生トークンと最終トークンは、`scan_raw` に渡されたスキャナー入力を指します。その入力が `PreprocessedLexicalSource.lexical_text` の場合、スパンは字句テキストのオフセットであり、元の読み込み済み `.miz` テキストへのオフセットとは限りません。`SourceLineIndex` は、必ずスパンが指しているテキストと同じテキストから構築します。

字句テキストのオフセットから元の読み込み済みソースのオフセットへの対応付けは、明示的に扱います。`PreprocessedLexicalSource.preprocess_map` は、コメント除去と合成レイアウトのための軽量で字句解析器ローカルなマップを提供し、よりリッチなスナップショット/ソースマップの所有はセッション層に残ります。字句解析器は、前処理済みテキスト上のスパンを元のファイル座標として暗黙に扱ってはいけません。

ソース読み込みが先頭 BOM を取り除いたり CRLF の改行対を正規化したりした場合、字句解析器のスパンは生のファイルバイト列ではなく、正規化済みの読み込みテキストへのバイトオフセットとして測られます。ディスクソースでは、`LoadingMap` がそれらの読み込みテキストのオフセットを元のファイルのバイトオフセットへ関連付けます。BOM が取り除かれている場合、読み込みテキストのオフセット `0` は元のファイルのバイトオフセット `3` に対応します。正規化された LF は、元の 2 バイトの CRLF 範囲に対応します。取り除かれた BOM は、字句解析器の `SourceSpan` を持ちません。

クレートローカルの `load_source_text_from_bytes` ヘルパーは、この規約のうち UTF-8・先頭 BOM・CRLF→LF の部分を実装します。不正な UTF-8 は `SourceLoadError::InvalidUtf8` として拒否し、バイト列を非可逆にデコードして `U+FFFD` にすることはありません。先頭 BOM を取り除いた場合、`LoadedSourceText.loading_map` は `RemovedLeadingBom { original: [0, 3) }` と、読み込みオフセットを元のバイトオフセットに対応付ける後続セグメントを返します。CRLF を正規化した場合は、単一の LF バイトの読み込み範囲と、2 バイトの CRLF の綴りの元範囲を持つ `NormalizedNewline` セグメントを記録します。単独の `\r` はこのヘルパーでは正規化せず、字句境界における不正な入力のままです。完全なファイル I/O、パス正規化、ハッシュ、リッチなセッションの `LineMap` の所有は、引き続き `mizar-lexer` の外側にあります。

字句解析器は、生トークンや最終トークンのすべてに行/列番号を保存してはいけません。行/列は、診断・デバッグ出力・スナップショット・LSP ブリッジが人間に読める座標を必要とするときに、ソーステキストから計算する導出ビューです。これにより位置データの重複を避け、トークンの値の中で複数の座標系が混ざることを防ぎます。

`mizar-lexer` は、字句解析器ローカルに使える軽量な行インデックスのヘルパーを提供します。

```rust
pub struct SourceLineIndex {
    line_starts: Vec<usize>,
    char_boundaries: Vec<usize>,
    source_len: usize,
}

pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

pub struct SourceLocationRange {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceLineIndex {
    pub fn new(source: &str) -> Self;
    pub fn location(&self, offset: usize) -> Option<SourceLocation>;
    pub fn range(&self, span: SourceSpan) -> Option<SourceLocationRange>;
}
```

内部規約は、0 始まりの行と 0 始まりのバイト列です。`location` と `range` は、要求されたオフセットまたはスパンがインデックス化されたソーステキストの外側を指す場合、または UTF-8 文字境界ではない場合に `None` を返します。人間向けの診断では、整形時に 1 始まりの表示番号へ変換できます。LSP 固有の UTF-16 位置はトークンに保存せず、同じバイトオフセットから LSP ブリッジまたは専用アダプターが計算します。

このヘルパーは、ソース読み込みの抽象ではありません。セッション層は、開きバッファ・スナップショット・ソースマップ・LSP 統合のために、`LoadedSource` 上でよりリッチな `LineMap` を保持できます。`mizar-lexer` が持つのは、`&str` から字句解析器の診断とテストを読みやすくするために必要な座標変換だけです。

### Stage 1: Raw Scan

生スキャナーは LF のみのソーステキストを読み、ソーススパンを保持する生の単位を生成します。

生の単位は最終的な言語トークンではありません。特に `LexemeRun` は図形文字の連続であり、後で 1 個以上の最終トークンに変換されます。

```rust
#[non_exhaustive]
pub enum RawTokenKind {
    LexemeRun,
    NumeralLike,
    AnnotationMarker,
    Layout,
    Error,
}
```

`scan_raw` は現在、未対応の生入力に対して `RawTokenKind::Error` を出力するのではなく、`LexError` を返します。`Error` バリアントは、不正な生の単位を後段の曖昧性解消まで運びたい呼び出し側や、将来の回復経路のために残しています。

`LexemeRun` は中心的な生の単位です。識別子の形をした綴りと、記号の形をした綴りの両方を含みます。

```text
alpha
succ
foo'
+
*+
|.
x*+y
```

生スキャナーは、スパン・綴り・後段の最長一致による曖昧性解消に必要な構造を保持しなければなりません。アクティブなユーザーシンボルの認識を不可能にするような、早すぎる分割は避けます。

`LexemeRun` は意図的に粗い単位です。予約句読点である `.`, `..`, `,`, `;`、引用符、演算子文字は、ラン内に現れ得ます。後続のモジュールは必要に応じてランの内部を調べて分割してよいですが、ソーススパンを保持し、生スキャナーに文法の文脈を要求してはいけません。

コメントとドキュメンテーションコメントは生トークンではありません。`preprocess_source_for_lexing` はそれらを字句入力から取り除き、トリビアとソーススパンを別に保持し、`lexical_text` には改行を残します。また、インラインコメントの除去によって隣接するトークンの形をしたテキストが連結してしまう場合は合成の空白を挿入し、元テキスト・除去コメント・合成空白/改行の前処理マップのセグメントを記録します。複数行コメントは入れ子にならず、最初の閉じ記号 `=::` がコメントを終了します。インポート事前スキャンとスコープスケルトンの構築は、その結果の字句テキストに対して動作するため、コメントを `RawTokenKind` の値として受け取ることはありません。

### Import Pre-Scan and Active Lexical Environment

生スキャナーはインポートを解釈せず、モジュールシステムも知りません。生の単位を生成するだけです。

アクティブなユーザーシンボルは、別のインポート事前スキャンと環境構築の経路によって組み立てられます。

```text
LF-only source text
  -> raw scan
       LexemeRun spans を持つ RawTokenStream
  -> import pre-scan
       raw module path spellings を持つ top-level ImportStub values
  -> module resolver / build planner
       module ids and imported module lexical summaries
  -> lexical environment builder
       ActiveLexicalEnvironment
```

インポート事前スキャンは、制限付きの構文モードで生の字句解析器の出力を読みます。`.`、`..`、`,`、`;` などのインポート構文のために、`LexemeRun` のスパンの内部を調べて分割してよいです。モジュールパスの綴りとソーススパンを抽出するために必要な、トップレベルのインポート構造だけを認識します。パッケージ/モジュールの存在、可視性、再エクスポートの合法性、インポートシンボルの同一性を解決してはいけません。

アクティブ字句環境は、曖昧性解消器が消費する入力です。組み込みの予約テーブルと、インポートしたモジュールの字句サマリー由来のエクスポートされたユーザーシンボルの形状を含み、後続のパーサー/リゾルバでの絞り込み用に、軽量な種別とアリティのメタデータも保持します。その構築は生スキャンの外側にあります。

### Stage 2: Disambiguation

曖昧性解消器は、生の単位を最終トークンに変換します。入力として以下を使います。

- 予約語;
- 予約特殊記号;
- インポートしたモジュールのインターフェースサマリー由来の、アクティブなユーザーシンボル;
- 現在の文法位置におけるパーサーの期待;
- シンボル/識別子の上書き規則が必要とする、読み取り専用のスコープビュー.

最長一致は、`LexemeRun` の内部で曖昧性解消器が処理します。1 つの生ランは複数の最終トークンになり得ます。

例:

```text
raw:   LexemeRun("x*+y")
final: Identifier("x"), UserSymbol("*+"), Identifier("y")
```

綴り全体を覆うアクティブなユーザーシンボルがあり、スコープ付き識別子規則による上書きがなければ、同じ生ランは以下にもなり得ます。

```text
raw:   LexemeRun("x*+y")
final: UserSymbol("x*+y")
```

曖昧性解消器はスコープ情報を参照しますが、それを構築しません。スコープビューは、本格的なパースの前に、専用のスコープスケルトン事前スキャンによって生成されます。

## Scope Skeleton Pre-Scan

パーサーの構築はトークンの曖昧性解消に依存します。一方で、トークンの曖昧性解消は、スコープ付き識別子束縛がアクティブなユーザーシンボルを上書きするかを知る必要があります。このパーサー/字句解析器の循環を避けるため、Mizar Evo は専用のスコープスケルトン事前スキャンを使います。

スコープスケルトン事前スキャンは、生の字句解析器の出力を読み、字句上の束縛範囲を近似するために必要な、予約キーワードの形をした構造だけを認識します。`SurfaceAst` は生成せず、意味論的な名前解決も行わず、識別子が定義済みかどうかも判定しません。

認識の対象は、たとえば以下です。

- `definition`, `proof`, `now`, `end` のような、字句スコープに影響するブロック区切り;
- `let`, `for`, `reserve`, `given` のような、束縛子を導入する予約語と形式;
- 予約構文から形状を復元できる、カンマ区切りの束縛リスト;
- 式の全体をパースせずに束縛範囲を近似できる局所名.

結果は、字句上の上書きの問いにだけ答えるスコープスケルトンです。

```rust
pub struct ScopeSkeleton {
    pub frames: Vec<LexicalScopeFrame>,
}

pub struct LexicalScopeFrame {
    pub range: SourceRange,
    pub bindings: Vec<ScopedBindingShape>,
}

pub struct ScopedBindingShape {
    pub spelling: String,
    pub introduced_at: SourceRange,
}
```

スケルトンは、不正なソースや未対応のソースでは束縛を過小近似してよいです。ただし、決定的であり、ソーススパンを保持しなければなりません。プログラムを意味論的に受理/拒否してはいけません。

曖昧性解消器は、狭い射影だけを受け取ります。

```rust
pub trait ScopeLexView {
    fn binding_overrides_symbol(&self, spelling: &str, position: SourcePos) -> bool;
}
```

`ScopeLexView` は、スコープスケルトンと、必要な場合はリゾルバが提供するモジュールスコープのデータから実装されます。字句解析器に、完全なリゾルバの状態・型情報・オーバーロード候補・証明の意味論を公開してはいけません。

## Symbol and Identifier Boundary

`Identifier` は、識別子の形をしたソーステキストに対する最終トークンの種別です。これは、その名前が定義済みであることを意味しません。

未定義名の診断は名前解決の責務であり、生の字句解析の責務ではありません。

ただし、識別子の形をしたユーザーシンボルと識別子の最終分類には、スコープ情報が必要になる場合があります。言語仕様として、スコープ付き識別子束縛がアクティブなシンボルを上書きする場合、曖昧性解消器は `UserSymbol` に確定する前に、スコープ付き束縛環境を参照しなければなりません。

責務の境界は以下のとおりです。

| Question | Owner |
|---|---|
| この綴りは識別子構文に合うか | raw lexer helper |
| この綴りはアクティブなインポート済みユーザーシンボルか | lexical environment |
| この位置でスコープ付き識別子束縛が記号を上書きできるか | scope skeleton / `ScopeLexView` |
| 上書きを考慮した後、どの候補を選ぶか | disambiguator |
| 得られた識別子が定義済みで、この文法構文で合法か | name resolution / later semantic phases |
| 記号または識別子がどのオーバーロードを指すか | overload/type checking |

生の字句解析器は、これらを 1 つの判断に潰してはいけません。

## Longest-Match Rules

最長一致は、早い段階の生トークン分割ではなく、曖昧性解消器が適用します。

`LexemeRun` 内の各位置で、曖昧性解消器は以下の候補を検討します。

1. アクティブなユーザーシンボル
2. 予約複合記号
3. 予約語
4. 識別子構文
5. 数字から始まる場合の数字構文
6. フォールバックのエラー回復

選択される候補は、現在のパーサーの期待と上書き環境の下で有効な、最長の候補です。異なるインポートから来た同綴りのシンボルは、字句環境の構築時点ですでに拒否されています。同じインポート内の同綴りオーバーロードは、種別とアリティのメタデータ付きで、後続の意味論的な解決のために保持されますが、字句解析器が選ぶトークンの綴りは変わりません。

パーサーの期待は、単体では有効な候補を排除できます。たとえば、束縛子の識別子を期待する文法位置では識別子としての解釈を優先し、式の位置では記号としての解釈を許可できます。

## Imported Symbol Data

字句解析器は、インポートした `.miz` ファイルの完全な IR を読み込んではいけません。

インポートは、エクスポートされた字句シンボル、シンボルの種別とアリティの形状、診断用の十分な由来を含む、軽量なモジュールインターフェースサマリーを提供します。

```rust
pub struct ModuleLexicalSummary {
    pub module_id: ModuleId,
    pub exported_symbols: Vec<ExportedSymbolShape>,
    pub fingerprint: LexicalSummaryFingerprint,
}
```

アクティブ字句環境は、これらのサマリーと組み込みの予約テーブルから構築します。

完全なモジュール IR は、構文・解決・検証・成果物のデータを必要とする後続フェーズだけが読み込みます。

## Current Public API

現在のクレートローカル API は、ブートストラップ用の識別子字句解析器より広くなっています。

```rust
pub fn preprocess_source_for_lexing(input: &str) -> PreprocessedLexicalSource;

pub struct PreprocessedLexicalSource {
    pub lexical_text: String,
    pub comments: Vec<CommentTrivia>,
    pub diagnostics: Vec<SourcePreprocessDiagnostic>,
    pub preprocess_map: SourcePreprocessMap,
}

pub enum SourcePreprocessMapSegment {
    Original { lexical: SourceRange, source: SourceRange },
    RemovedComment { source: SourceRange, kind: CommentKind },
    SyntheticWhitespace { lexical: SourceRange, anchor: SourceRange },
}

pub fn module_source_name_from_path(
    package_name: &str,
    path: &str,
) -> Result<ModuleSourceName, ModuleNamingError>;

pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError>;
pub fn disambiguate_reserved_shell(raw: &RawTokenStream) -> Result<Vec<Token>, LexError>;
pub fn lex(input: &str) -> Result<Vec<Token>, LexError>;

pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: SourceSpan,
}

#[non_exhaustive]
pub enum TokenKind {
    Identifier,
    ReservedWord,
    ReservedSymbol,
    Numeral,
    LexemeRun,
    UserSymbol,
    StringLiteral,
    ErrorRecovery,
}
```

`lex` は、生スキャンと予約シェルの曖昧性解消を組み合わせた利便用ラッパーです。この文脈自由な分類でもソース位置を落としてはいけないため、スパン付きの最終トークンを返します。文脈依存の分類が必要な場合は、[disambiguator.md](./disambiguator.md) に記載する `disambiguate` API を使います。

低レベルの綴り規則は、ヘルパー述語に集約されています。レイアウトは空白・タブ・LF のみです。識別子は ASCII の英字または `_` で始まり、継続文字には数字と `'` も使えます。数字は ASCII の数字のランです。ユーザーシンボルの綴りは空でない ASCII 図形文字のランで、`@` を含みません。文字列リテラルの綴りは同じ引用符で閉じる必要があり、エスケープできるのは `"`, `'`, `\` だけです。

## Context-Sensitive API

明示的な生スキャン/曖昧性解消の API は、現在の実装に存在します。

```rust
pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError>;

pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream;
```

`ScopeLexView` は、曖昧性解消器の外側で生成される、狭い読み取り専用ビューです。あるソース位置にスコープ付き識別子束縛が存在しアクティブなシンボルを上書きするか、という字句の曖昧性解消に必要な問いにだけ答えます。字句解析器に、完全なリゾルバや型検査器を公開してはいけません。

## Error Handling

生スキャンのエラーは、字句層に届いた不正なソース形状を表します。

- ソース読み込み後に残った、LF でないキャリッジリターン;
- ソース読み込みが拒否しなかった、未対応の非 ASCII コード文字;
- 垂直タブやフォームフィードなどの、未対応の ASCII 制御文字;
- あり得ない注釈マーカー.

曖昧性解消のエラーは、パーサーコンテキストなどを考慮した後のトークン化の失敗を表します。

- あるソース位置に有効なトークン候補がない;
- 文法の文脈が、生ラン内のすべての候補を禁止している.

未定義の識別子は字句解析のエラーではありません。

最終トークンのスパンは、字句解析器の境界の一部です。1 対 1 の対応では `RawToken` のスパンをコピーし、`LexemeRun` が複数の最終トークンに分割される場合は、生スパンの内部を細分します。下流のパーサー・診断・LSP・整形器・差分解析の各層は、生トークンを再参照しなくても、すべての最終トークンの位置を特定できなければなりません。

行/列の値は、最終トークンのスパンから `SourceLineIndex` またはセッション層の `LineMap` を通じて導出します。`Token` には保存しません。

## Tests

クレートのテストとコーパスのテストは、以下を確認します。

- 識別子・数字・レイアウト・注釈マーカー・予約語・予約記号のテーブル;
- ソース前処理の診断と、モジュールソース名の境界;
- 未対応の Unicode コード領域文字と未対応の ASCII 制御文字が、レイアウトやトークンテキストではなく、診断または安定した生スキャンの致命的エラーとして扱われること;
- `scan_raw` が早すぎる分割をせず `LexemeRun` のスパンを保持すること;
- 本格的なパースの前に、予約キーワードの形をした束縛構造からスコープスケルトンを構築できること;
- 最長一致が最長のアクティブなユーザーシンボルを選ぶこと;
- 識別子の形をしたユーザーシンボルが、字句環境とスコープ上書き規則に従って曖昧性解消されること;
- 完全な IR を読み込まなくても、インポートシンボルのサマリーだけで字句の曖昧性解消に足りること;
- 未解決の識別子はトークンとして残り、名前解決の診断は後続フェーズに委ねられること;
- `cargo-fuzz` のカバレッジにより、任意の有効な UTF-8 入力に対する `preprocess_source_for_lexing`、直接の `scan_raw`、前処理済み字句テキスト上の `scan_raw` を試験すること;
- フェーズ 7 のリグレッションテストにより、生/最終スパンのカバレッジ、決定的な生スキャン、再トークン化、インポート衝突、回復スパン、複合的な曖昧性解消の動作が保たれること。
