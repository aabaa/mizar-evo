# Module: raw_lexer

> Canonical language: English. English canonical version: [../en/raw_lexer.md](../en/raw_lexer.md).

## Purpose

この module は、Mizar Evo における lexer の責務境界を定義します。

Mizar の字句分類は文脈に依存します。import された module は user-defined symbol を増やし、user symbol は identifier と同じ形を取り得ます。さらに、局所的な identifier binding が symbol 解釈を override することもあります。そのため lexer は、すべての `Identifier` / `UserSymbol` 分類を context-free な 1 pass で確定する設計にしてはいけません。

現在の実装は、低レベルの raw scanner と、より高レベルな disambiguation entry point の両方を公開しています。この文書では、便利関数 `lex(&str)` の shell を完全な context-sensitive lexer と誤解しないよう、各層の責務境界を明確にします。

## Public API Stability

`mizar-lexer` は現在 `0.1` crate です。public data structures は parser-facing transfer objects として扱い、初期段階の parser、corpus、integration code が直接 inspect / construct できるよう fields を visible のままにします。

public enums には `#[non_exhaustive]` を付けます。downstream crates は token kinds、raw token kinds、diagnostic codes、parser modes、import pre-scan categories、scope-skeleton categories、source-preprocessing categories、lexical-environment errors を match するとき wildcard arm を含める必要があります。これにより parser-facing API が成熟するまで category を追加できる余地を保ちます。

明示的な stability milestone を後で設けるまでは、`0.1` minor releases でも lexer boundary の一貫性を保つために public fields、constructors、helper functions に breaking changes を加える可能性があります。

## Source Preconditions

`mizar-lexer` に渡される入力は raw file bytes ではありません。

この crate の外側にある source-loading layer は、以下を担当します。

- file read;
- UTF-8 validation;
- scanner entry point に渡す前の platform newline から LF-only text への正規化;
- 必要に応じた original file offsets への source map;
- package 内で source file をどのように見つけるかの決定。

一方、`mizar-lexer` は lexical boundary 用の source preprocessing helpers を提供します。

- ordinary comment、documentation comment、multi-line comment を lexical input から取り除く;
- comment trivia を source span とともに保持する;
- コメント内の newline だけを残し、行構造を崩さないようにする;
- comment removal が隣接する token-shaped text を連結してしまう場合は synthetic layout を挿入する;
- carriage return、code region 内の non-ASCII text、閉じていない multi-line comment を preprocessing diagnostic として報告する;
- 必要に応じて package-rooted `.miz` source name を検証する。

`mizar-lexer` は layout が以下のみであると仮定できます:

```text
space, tab, newline
```

Carriage return はこの layer では layout ではありません。`\r` が lexer に届いた場合、それは source-loading 側の不備か、意図的な malformed test fixture です。

## Source-Text Normalization Policy

`mizar-lexer` は Unicode normalization を行いません。code text に対して canonical normalization や compatibility normalization を適用してから lexical spelling rules を判定することはありません。

この layer では、code-region identifiers、numerals、reserved words、reserved symbols、user-symbol spellings は ASCII-only です。code region に届いた non-ASCII text は lexer boundary における malformed input です。preprocessing は `NonAsciiCode` として報告し、direct raw scanning は unsupported characters を reject します。ASCII spelling へ変換して受け入れることはありません。

comments と documentation comments は別扱いです。その text は source span 付きの raw Unicode trivia として保持されます。ただし、上記の comment-stripping rules に従い、newline structure は `lexical_text` に残します。lexer は comment/documentation text 内の Unicode を normalize せず、warning も reject も行いません。将来の documentation、source-loading、diagnostic policy は、lexer tokenization を変更せずに suspicious Unicode、confusables、normalization-sensitive text への warning を追加できます。

## Core Design

Lexing は概念的に 2 段階に分けます。

## 実装上のアルゴリズムの流れ

現在の crate は、便利関数 `lex` を使う場合でも、source preparation、raw scanning、final disambiguation を分けて扱います。

1. `preprocess_source_for_lexing` は入力を先頭から順に走査します。コメントは lexical text から取り除きますが、コメント内の改行は残して行位置が崩れないようにし、削除によって隣接する token-shaped text が連結してしまう場合は synthetic layout を挿入します。コメント本体は source span 付きの trivia として保持します。`\r`、code region 内の non-ASCII 文字、閉じていない multi-line comment は preprocessing diagnostics として報告します。multi-line comment は nest しません。`::=` opener の後に最初に現れる `=::` が comment を閉じ、内部の `::=` spelling は通常の comment text として扱います。この helper はファイル読み込みや OS ごとの改行正規化は担当しません。
2. `scan_raw` は LF-only の lexical text を `char_indices` cursor で読みます。連続する layout は 1 個の `Layout` にまとめ、`@` から始まる annotation marker を認識し、`@` 以外の ASCII graphic characters は連続した run としてまとめます。その run がすべて digit なら `NumeralLike`、そうでなければ `LexemeRun` です。対応していない文字は `LexError` になります。
3. `disambiguate_reserved_shell` は `lex` が使う context-free な薄い shell です。layout を捨て、`NumeralLike` を `Numeral` にし、`@[` を reserved symbol にします。`LexemeRun` 全体については、reserved symbol、reserved word、identifier、または不透明な `LexemeRun` として分類します。
4. import、parser context、scope override が分類に影響する場合は `disambiguate` を使います。この経路では raw scanning はあえて coarse に保ち、`LexemeRun` の内部分割は disambiguator が reserved tables、active lexical environment、parser lexical context、`ScopeLexView` を見て行います。
5. `module_source_name_from_path` は scanner ではなく source boundary helper です。package name を検証し、`.miz` file が `src` root 配下にあること、source root が package name と一致すること、path separator の違いを吸収できること、namespace components がすべて identifier-shaped であることを確認します。

Raw scanner の重要な不変条件は span contiguity です。出力された raw token は必ず元入力の正確な byte slice を指し、raw token の lexeme を連結すると scanner input が復元できます。

### Source Coordinates

`SourceSpan` は `mizar-lexer` 内部の canonical coordinate type です。これは token または diagnostic を生成した正確な text に対する byte offset を保持し、半開区間 `[start, end)` を表します。

caller は coordinate space を明示的に扱わなければなりません。`scan_raw` と `disambiguate` から生成される raw token と final token は、`scan_raw` に渡された scanner input を指します。その input が `PreprocessedLexicalSource.lexical_text` の場合、span は lexical-text offset であり、元の loaded `.miz` text への offset とは限りません。`SourceLineIndex` は、必ず span が指している text と同じ text から構築します。

lexical-text offset から original loaded-source offset への mapping は source map または session layer の責務です。lexer は preprocessed text 上の span を original file coordinate として暗黙に扱ってはいけません。

lexer は raw token や final token のすべてに line/column number を保存してはいけません。Line/column は diagnostics、debug output、snapshots、LSP bridge が human-readable coordinate を必要とする時に、source text から計算する derived view です。これにより location data の重複を避け、token value の中で複数の coordinate system が混ざることを防ぎます。

`mizar-lexer` は lexer-local に使える lightweight line-index helper を提供します。

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

内部規約は zero-based line と zero-based byte column です。`location` と `range` は、要求された offset または span が indexed source text の外側を指す場合、または UTF-8 character boundary ではない場合に `None` を返します。人間向け diagnostics では formatting 時に one-based display number へ変換できます。LSP-specific な UTF-16 position は token に保存せず、同じ byte offset から LSP bridge または dedicated adapter が計算します。

この helper は source-loading abstraction ではありません。Session layer は open buffers、snapshots、source maps、LSP integration のために `LoadedSource` 上でより rich な `LineMap` を保持できます。`mizar-lexer` が持つのは、`&str` から lexer diagnostics and tests を読みやすくするために必要な coordinate conversion だけです。

### Stage 1: Raw Scan

Raw scanner は LF-only source text を読み、source span を保持する raw unit を生成します。

Raw unit は final language token ではありません。特に `LexemeRun` は graphic character の連続であり、後で 1 個以上の final token に変換されます。

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

`scan_raw` は現在、unsupported raw input に対して `RawTokenKind::Error` を emit するのではなく、`LexError` を返します。`Error` variant は、malformed raw unit を後段の disambiguation まで運びたい caller や将来の recovery path のために残しています。

`LexemeRun` は中心的な raw unit です。identifier-shaped spelling と punctuation-shaped spelling の両方を含みます。

```text
alpha
succ
foo'
+
*+
|.
x*+y
```

Raw scanner は span、spelling、後段の longest-match disambiguation に必要な構造を保持しなければなりません。active user symbol の認識を不可能にするような早すぎる分割は避けます。

`LexemeRun` は意図的に粗い単位です。reserved punctuation である `.`, `..`, `,`, `;`、quote、operator character は run 内に現れ得ます。後続 module は必要に応じて run の内部を調べて分割してよいですが、source span を保持し、raw scanner に grammar context を要求してはいけません。

Comment と documentation comment は raw token ではありません。`preprocess_source_for_lexing` はそれらを lexical input から取り除き、trivia と source span を別に保持し、`lexical_text` には newline を残します。また、inline comment の削除によって隣接する token-shaped text が連結してしまう場合は synthetic space を挿入します。multi-line comment は nest せず、最初の closing `=::` が comment を終了します。Import pre-scan と scope skeleton construction はその lexical text に対して動作するため、comment を `RawTokenKind` として受け取ることはありません。

### Import Pre-Scan and Active Lexical Environment

Raw scanner は imports を解釈せず、module system も知りません。raw units を生成するだけです。

Active user symbols は、別の import pre-scan と environment-building path により組み立てられます:

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

Import pre-scan は restricted syntax mode で raw lexer output を読みます。`.`、`..`、`,`、`;` などの import syntax のために、`LexemeRun` spans の内部を inspect and split してよいです。module path spellings と source spans を抽出するために必要な top-level import structure だけを認識します。package/module existence、visibility、re-export legality、imported symbol identity を resolve してはいけません。

Active lexical environment は disambiguator が consume する input です。built-in reserved tables と imported module lexical summaries 由来の exported user-symbol shapes を含みます。その構築は raw scanning の外側にあります。

### Stage 2: Disambiguation

Disambiguator は raw unit を final token に変換します。入力として以下を使います。

- reserved words;
- reserved special symbols;
- imported module interface summary 由来の active user symbols;
- current grammar position の parser expectation;
- symbol/identifier override rules が必要とする read-only scope view.

Longest-match は `LexemeRun` の内部で disambiguator が処理します。1 つの raw run は複数の final tokens になり得ます。

例:

```text
raw:   LexemeRun("x*+y")
final: Identifier("x"), UserSymbol("*+"), Identifier("y")
```

full spelling を覆う active user symbol があり、scoped identifier rule による override がなければ、同じ raw run は以下にもなり得ます。

```text
raw:   LexemeRun("x*+y")
final: UserSymbol("x*+y")
```

Disambiguator は scope information を参照しますが、それを構築しません。scope view は full parsing の前に dedicated scope-skeleton pre-scan によって生成されます。

## Scope Skeleton Pre-Scan

Parser construction は token disambiguation に依存します。一方で token disambiguation は、scoped identifier binding が active user symbol を override するかを知る必要があります。この parser/lexer cycle を避けるため、Mizar Evo は dedicated scope-skeleton pre-scan を使います。

Scope skeleton pre-scan は raw lexer output を読み、lexical binding range を近似するために必要な reserved-keyword-shaped structure だけを認識します。`SurfaceAst` は生成せず、semantic name resolution も行わず、identifier が定義済みかどうかも決めません。

認識対象は、たとえば以下です。

- `definition`, `proof`, `now`, `end` のような lexical scope に影響する block delimiters;
- `let`, `for`, `reserve`, `given` のような binder-introducing reserved words and forms;
- reserved syntax から shape を recover できる comma-separated binding lists;
- full expression parsing なしに binding range を近似できる local names.

結果は lexical override questions だけに答える scope skeleton です:

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

Skeleton は malformed source や unsupported source では binding を under-approximate してよいです。ただし deterministic で source span を保持しなければなりません。program を semantic に accept/reject してはいけません。

Disambiguator は narrow projection だけを受け取ります。

```rust
pub trait ScopeLexView {
    fn binding_overrides_symbol(&self, spelling: &str, position: SourcePos) -> bool;
}
```

`ScopeLexView` は scope skeleton と、必要な場合は resolver-provided module-scope data から実装されます。lexer に full resolver state、type information、overload candidate、proof semantics を公開してはいけません。

## Symbol and Identifier Boundary

`Identifier` は identifier-shaped source text の final token class です。これはその name が定義済みであることを意味しません。

Undefined-name diagnostics は name resolution の責務であり、raw lexing の責務ではありません。

ただし、identifier-shaped user symbols と identifiers の final classification には scope information が必要になる場合があります。言語仕様として scoped identifier binding が active symbol を override する場合、disambiguator は `UserSymbol` に確定する前に scoped binding environment を参照しなければなりません。

責務の境界は以下の通りです。

| Question | Owner |
|---|---|
| この spelling は identifier syntax に合うか | raw lexer helper |
| この spelling は active imported user symbol か | lexical environment |
| この位置で scoped identifier binding が symbol を override できるか | scope skeleton / `ScopeLexView` |
| override を考慮した後、どの candidate を選ぶか | disambiguator |
| 結果として得られた identifier が定義済みで、この grammar construct で合法か | name resolution / later semantic phases |
| symbol または identifier がどの overload を指すか | overload/type checking |

Raw lexer はこれらを 1 つの判断に潰してはいけません。

## Longest-Match Rules

Longest-match は early raw token splitting ではなく disambiguator が適用します。

`LexemeRun` 内の各位置で、disambiguator は以下の candidates を検討します。

1. active user symbols
2. reserved compound symbols
3. reserved words
4. identifier syntax
5. digit から始まる場合の numeral syntax
6. fallback error recovery

選択される candidate は、現在の parser expectation と override environment の下で有効な最長 candidate です。異なる import から来た同一 spelling の symbol は lexical environment の構築時点で拒否されます。同じ import 内の同一 spelling overload は後続の semantic resolution 用に保持されますが、lexer が選ぶ token spelling は変わりません。

Parser expectation は、単体では valid な candidate を排除できます。たとえば binder identifier を期待する grammar position では identifier interpretation を優先し、expression position では symbol interpretation を許可できます。

## Imported Symbol Data

Lexer は imported `.miz` files の full IR を読み込んではいけません。

Imports は exported lexical symbols と diagnostics 用 provenance を含む lightweight module interface summary を提供します:

```rust
pub struct ModuleLexicalSummary {
    pub module_id: ModuleId,
    pub exported_symbols: Vec<ExportedSymbolShape>,
    pub fingerprint: LexicalSummaryFingerprint,
}
```

Active lexical environment は、これらの summary と built-in reserved table から構築します。

Full module IR は syntax、resolution、verification、artifact data が必要な later phase だけが読み込みます。

## Current Public API

現在の crate-local API は、bootstrap 用の identifier lexer より広くなっています。

```rust
pub fn preprocess_source_for_lexing(input: &str) -> PreprocessedLexicalSource;
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

`lex` は raw scanning と reserved-shell disambiguation を組み合わせた convenience wrapper です。この context-free classification でも source location を落としてはいけないため、span 付き final token を返します。context-sensitive な分類が必要な場合は、[disambiguator.md](./disambiguator.md) に記載する `disambiguate` API を使います。

低レベルの spelling rule は helper predicate に集約されています。layout は space、tab、LF のみです。identifier は ASCII alphabetic または `_` で始まり、継続文字には digit と `'` も使えます。numeral は ASCII digit run です。user-symbol spelling は空でない ASCII graphic run で、`@` を含みません。string literal spelling は同じ quote で閉じる必要があり、escape できるのは `"`, `'`, `\` だけです。

## Context-Sensitive API

明示的な raw scanning / disambiguation API は現在の実装に存在します。

```rust
pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError>;

pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream;
```

`ScopeLexView` は disambiguator の外側で生成される narrow read-only view です。source position に scoped identifier binding が存在し active symbol を override するか、という lexical disambiguation に必要な質問だけに答えます。lexer に full resolver や type checker を公開してはいけません。

## Error Handling

Raw scanning error は lexical layer に届いた malformed source shape を表します。

- source loading 後に残った non-LF carriage returns;
- source loading が reject しなかった unsupported non-ASCII code characters;
- vertical tab や form feed などの unsupported ASCII control characters;
- impossible annotation markers.

Disambiguation error は parser context などを考慮した後の tokenization failure を表します。

- source position に valid token candidate がない;
- grammar context が raw run 内のすべての candidates を禁止している.

未定義の identifier は lexing error ではありません。

Final token span は lexer boundary の一部です。1 対 1 の対応では `RawToken` span をコピーし、`LexemeRun` が複数の final token に分割される場合は raw span の内側を subdivide します。下流の parser、diagnostic、LSP、formatter、incremental-analysis layer は、raw token を再参照しなくてもすべての final token の位置を特定できなければなりません。

Line/column values は final token span から `SourceLineIndex` または session layer の `LineMap` を通じて derive します。`Token` には保存しません。

## Tests

crate tests と corpus tests は以下を確認します。

- identifier、numeral、layout、annotation marker、reserved word、reserved symbol tables;
- source preprocessing diagnostics と module source naming boundary;
- unsupported Unicode code-region characters と unsupported ASCII control characters が layout や token text ではなく diagnostics または stable raw-scan hard errors として扱われること;
- `scan_raw` が早すぎる分割をせず `LexemeRun` spans を保持すること;
- full parsing 前に reserved-keyword-shaped binding structure から scope skeleton を build できること;
- longest-match が最長の active user symbol を選ぶこと;
- identifier-shaped user symbol が lexical environment と scope override rule に従って disambiguate されること;
- full IR を読み込まなくても imported symbol summary だけで lexical disambiguation に足りること;
- unresolved identifier は token として残り、name resolution diagnostics は後続 phase に委ねられること;
- `cargo-fuzz` coverage により、arbitrary valid UTF-8 input に対する `preprocess_source_for_lexing`、direct `scan_raw`、preprocessed lexical text 上の `scan_raw` を exercise すること;
- Phase 7 regression tests により raw/final span coverage、deterministic raw scanning、retokenization、import conflict、recovery spans、composite disambiguation behavior が保たれること。
