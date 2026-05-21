# Module: raw_lexer

> Canonical language: English. English canonical version: [../en/raw_lexer.md](../en/raw_lexer.md).

## Purpose

This module defines the lexer boundary for Mizar Evo.

Mizar の字句分類は context-sensitive です。imported modules は user-defined symbols を追加し、user symbols は identifier-shaped になり得ます。さらに scoped identifier bindings が symbols を override する可能性があります。そのため lexer は、すべての `Identifier` / `UserSymbol` 分類を context-free な 1 pass で確定する設計にしてはいけません。

現在の最小実装は `Token`, `TokenKind::Identifier`, `LexError`, `lex(&str)` だけを公開します。この文書は、その最小 API を将来の context-sensitive lexer/parser boundary と矛盾しない形で育てるための設計を定義します。

## Source Preconditions

`mizar-lexer` に渡される input は raw file bytes ではありません。

Source-loading layer owns:

- file read;
- UTF-8 validation;
- platform newlines から LF-only text への normalization;
- ordinary comments を lexical input から除去し、comment metadata は別に保持すること;
- documentation comments を later attachment 用の trivia metadata として保持すること;
- 必要に応じた original file offsets への source map;
- lexing 前の code-region ASCII rule validation.

`mizar-lexer` は layout が以下のみであると仮定できます:

```text
space, tab, newline
```

Carriage return はこの layer では layout ではありません。`\r` が lexer に届いた場合、それは source-loading bug か意図的な malformed test fixture です。

## Core Design

Lexing は概念的に 2 段階に分けます。

## 実装上のアルゴリズムの流れ

現在の crate は、便利関数 `lex` を使う場合でも、source preparation、raw scanning、final disambiguation を分けて扱います。

1. `preprocess_source_for_lexing` は入力を先頭から順に走査します。通常コメントは lexical text から取り除きますが、コメント内の改行だけは残して行位置が崩れないようにします。コメント本体は source span 付きの trivia として保持します。`\r`、code region 内の non-ASCII 文字、閉じていない multi-line comment は preprocessing diagnostics として報告します。この処理はファイル読み込みや OS ごとの改行正規化は担当しません。
2. `scan_raw` は LF-only の lexical text を `char_indices` cursor で読みます。連続する layout は 1 個の `Layout` にまとめ、`@` から始まる annotation marker を認識し、`@` 以外の ASCII graphic characters は連続した run としてまとめます。その run がすべて digit なら `NumeralLike`、そうでなければ `LexemeRun` です。対応していない文字は `LexError` になります。
3. `disambiguate_reserved_shell` は `lex` が使う context-free な薄い shell です。layout を捨て、`NumeralLike` を `Numeral` にし、`@[` を reserved symbol にします。`LexemeRun` 全体については、reserved symbol、reserved word、identifier、または不透明な `LexemeRun` として分類します。
4. import、parser context、scope override が分類に影響する場合は `disambiguate` を使います。この経路では raw scanning はあえて coarse に保ち、`LexemeRun` の内部分割は disambiguator が reserved tables、active lexical environment、parser lexical context、`ScopeLexView` を見て行います。
5. `module_source_name_from_path` は scanner ではなく source boundary helper です。package name を検証し、`.miz` file が `src` root 配下にあること、source root が package name と一致すること、path separator の違いを吸収できること、namespace components がすべて identifier-shaped であることを確認します。

Raw scanner の重要な不変条件は span contiguity です。出力された raw token は必ず元入力の正確な byte slice を指し、raw token の lexeme を連結すると scanner input が復元できます。

### Stage 1: Raw Scan

Raw scanner は LF-only source text を読み、source span を保持する raw units を生成します。

Raw units は final language tokens ではありません。特に `LexemeRun` は、後で 1 個以上の final tokens に変換される graphic run です。

```rust
pub enum RawTokenKind {
    LexemeRun,
    NumeralLike,
    AnnotationMarker,
    Layout,
    Error,
}
```

`LexemeRun` は中心的な raw unit です。identifier-shaped spelling と punctuation-shaped spelling の両方を含みます:

```text
alpha
succ
foo'
+
*+
|.
x*+y
```

Raw scanner は span、spelling、後段の longest-match disambiguation に必要な構造を保持しなければなりません。active user symbols の認識を不可能にするような早すぎる分割をしてはいけません。

`LexemeRun` は意図的に coarse です。reserved punctuation である `.`, `..`, `,`, `;`、quotes、operator characters は run 内に現れ得ます。後続 modules は必要に応じて run の内部を inspect and split してよいですが、source spans を保持し、raw scanner に grammar context を要求してはいけません。

Comments and documentation comments は raw tokens ではありません。source-loading and preprocessing layers は ordinary comments を lexical input から除去し、documentation comments を trivia metadata として保持します。Import pre-scan and scope skeleton construction は preprocessed source metadata 経由でその trivia を skip できますが、comments を `RawTokenKind` values として受け取りません。

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

Disambiguator は raw units を final tokens に変換します。入力として以下を使います:

- reserved words;
- reserved special symbols;
- imported module interface summaries 由来の active user symbols;
- current grammar position の parser expectation;
- symbol/identifier override rules が必要とする read-only scope view.

Longest-match は `LexemeRun` の内部で disambiguator が処理します。1 つの raw run は複数の final tokens になり得ます。

Example:

```text
raw:   LexemeRun("x*+y")
final: Identifier("x"), UserSymbol("*+"), Identifier("y")
```

full spelling を覆う active user symbol があり、scoped identifier rule による override がなければ、同じ raw run は以下にもなり得ます:

```text
raw:   LexemeRun("x*+y")
final: UserSymbol("x*+y")
```

Disambiguator は scope information を consume しますが、それを build しません。scope view は full parsing の前に dedicated scope-skeleton pre-scan により生成されます。

## Scope Skeleton Pre-Scan

Parser construction は token disambiguation に依存しますが、token disambiguation は scoped identifier binding が active user symbol を override するかを知る必要があります。parser/lexer cycle を避けるため、Mizar Evo は dedicated scope-skeleton pre-scan を使います。

Scope skeleton pre-scan は raw lexer output を読み、lexical binding ranges を近似するために必要な reserved-keyword-shaped structure だけを認識します。`SurfaceAst` は生成せず、semantic name resolution も行わず、identifier が定義済みかどうかも決めません。

It may recognize constructs such as:

- `definition`, `proof`, `now`, `end` のような lexical scope に影響する block delimiters;
- `let`, `for`, `reserve`, `given` のような binder-introducing reserved words and forms;
- reserved syntax から shape を recover できる comma-separated binding lists;
- full expression parsing なしに binding range を近似できる labels or local names.

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

Skeleton は malformed or unsupported source では bindings を under-approximate してよいです。ただし deterministic で source-span preserving でなければなりません。programs を semantically に accept/reject してはいけません。

Disambiguator は narrow projection だけを受け取ります:

```rust
pub trait ScopeLexView {
    fn binding_overrides_symbol(&self, spelling: &str, position: SourcePos) -> bool;
}
```

`ScopeLexView` は scope skeleton と、必要な場合は resolver-provided module-scope data から実装されます。lexer に full resolver state、type information、overload candidates、proof semantics を公開してはいけません。

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

1. reserved compound symbols
2. active user symbols
3. reserved words
4. identifier syntax
5. digit から始まる場合の numeral syntax
6. fallback error recovery

選択される candidate は、現在の parser expectation と override environment の下で有効な最長 candidate です。同じ長さの user symbols が複数 active な場合、lexical environment の import-order shadowing rule が winner を決めます。

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

Active lexical environment は、これらの summaries と built-in reserved tables から構築します。

Full module IR は syntax, resolution, verification, artifact data が必要な later phases だけが読み込みます。

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
}

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

`lex` は raw scanning と reserved-shell disambiguation を組み合わせた convenience wrapper です。context-sensitive な分類が必要な場合は、`disambiguator.md` に記載する `disambiguate` API を使います。

低レベルの spelling rule は helper predicates に集約されています。layout は space、tab、LF のみです。identifier は ASCII alphabetic または `_` で始まり、継続文字には digit と `'` も使えます。numeral は ASCII digit run です。user-symbol spelling は空でない ASCII graphic run で、`@` を含みません。string literal spelling は同じ quote で閉じる必要があり、escape できるのは `"`, `'`, `\` だけです。

## Context-Sensitive API Direction

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

Raw scanning errors は lexical layer に届いた malformed source shape を表します。

- source loading 後に残った non-LF carriage returns;
- source loading が reject しなかった unsupported non-ASCII code characters;
- impossible annotation markers.

Disambiguation errors は parser context などを考慮した後の tokenization failure を表します。

- source position に valid token candidate がない;
- deterministic shadowing rule のない equal-length ambiguity;
- grammar context が raw run 内のすべての candidates を禁止している.

未定義の identifier は lexing error ではありません。

## Tests

crate tests と corpus tests は以下を確認します。

- identifier、numeral、layout、annotation marker、reserved word、reserved symbol tables;
- source preprocessing diagnostics と module source naming boundary;
- `scan_raw` が早すぎる分割をせず `LexemeRun` spans を保持すること;
- full parsing 前に reserved-keyword-shaped binding structure から scope skeleton を build できること;
- longest-match chooses the longest active user symbol;
- identifier-shaped user symbols are disambiguated with lexical environment and scope override rules;
- imported symbol summaries are enough for lexical disambiguation without loading full IR;
- unresolved identifiers remain tokens and are rejected later by name resolution diagnostics;
- Phase 7 regression tests により span coverage、deterministic raw scanning、retokenization、import conflict、composite disambiguation behavior が保たれること。
