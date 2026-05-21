# Module: import_prescan

> Canonical language: English. English canonical version: [../en/import_prescan.md](../en/import_prescan.md).

## Purpose

This module extracts the module import prelude from raw lexer output.

Import pre-scan は、final token disambiguation and parsing の前に active lexical environment を構築するために存在します。これは shallow でなければなりません。import-shaped syntax と source spans は認識しますが、modules の resolve、symbols の load、package visibility の validate は行いません。

## Public API

Implemented API:

```rust
pub struct ImportPrelude {
    pub imports: Vec<ImportStub>,
    pub end: SourcePos,
    pub diagnostics: Vec<ImportPrescanDiagnostic>,
}

pub struct ImportStub {
    pub path: RawModulePath,
    pub alias: Option<RawModuleAlias>,
    pub span: SourceRange,
}

pub struct RawModulePath {
    pub spelling: String,
    pub relative: Option<RawModuleRelativePrefix>,
    pub components: Vec<RawModulePathComponent>,
    pub source_segments: Vec<SourceRange>,
    pub span: SourceRange,
}

pub fn scan_import_prelude(raw: &RawTokenStream) -> ImportPrelude;
```

Ownership boundary:

- input is `RawTokenStream`;
- output is raw import spelling and source spans;
- module resolution happens elsewhere.

## Algorithm

現在の実装は、小さな token splitter と recoverable statement parser の組み合わせです。

1. まず `RawTokenStream` を import pre-scan 専用 token に変換します。layout は無視します。`LexemeRun` は source span を保ったまま、`Word`, `.`, `..`, `,`, `;`, `*`, `{`, `}`, `Other` に分割します。`NumeralLike`、annotation marker、raw error は `Other` として扱います。
2. prelude end position は、最初の non-layout token の start に初期化します。空の stream なら `0` です。
3. cursor が word `import` を指している間、import statement を読みます。`import` を消費した後、semicolon、EOF、または malformed boundary に到達するまで comma-separated module alias declarations を繰り返し読みます。
4. `parse_module_path` は optional relative prefix (`.` または `..`) を受け取り、その後に identifier-shaped path components を dot 区切りで読みます。dot の次が `{` の場合は branch import の開始として扱い、base path には含めません。
5. `parse_module_alias_decls` は optional `as alias` suffix を読みます。alias は identifier-shaped でなければなりません。alias が欠けていても path が復元できている場合、diagnostic を出したうえで import stub は保持します。
6. branch import は `base.{child, other}` を読み、複数の `ImportStub` に展開します。展開後の spelling は source text 上で連続していないため、`source_segments` に base span と branch component span の両方を記録します。
7. malformed input では、信頼できる最小の span に diagnostic を付けます。少なくとも1つの declaration を復元できたのに semicolon がなければ `MissingSemicolon`、そうでなければ `UnexpectedToken` または path/alias に応じたより具体的な diagnostic を出し、statement end まで recovery します。
8. top-level import statement の開始ではない token に到達したら、そこで prelude scanning は完全に終了します。

Scanner は prelude 終了後に import を探してはいけません。後続の import-shaped text は、Chapter 12 import-placement rules に従い parser が syntax error として扱います。

この algorithm は、`import` や `as` がその parser position で legal token かどうかを reserved table に問い合わせません。これは parser 前の浅い pass であり、後続 phase が module resolution と active lexical environment construction を行うために必要な raw import shape を集めるだけです。

## Accepted Syntax

The pre-scan recognizes the Chapter 12 import syntax:

```ebnf
import_stmt       ::= "import" module_alias_decl { "," module_alias_decl } ";" ;
module_alias_decl ::= module_path [ "as" module_identifier ]
                    | module_branch_import ;
module_branch_import
                  ::= module_path ".{" module_identifier { "," module_identifier } "}" ;
module_path       ::= [ relative_prefix ] module_identifier { "." module_identifier } ;
relative_prefix   ::= "." | ".." ;
module_identifier ::= identifier ;
```

Recognition is spelling-based. `import` and `as` are reserved-word spellings, and module path components are identifier-shaped raw lexemes.

Branch import syntax は、同じ prefix 配下の複数 module paths の shorthand です。たとえば `import algebra.linear.{eigen_value, jordan};` は `algebra.linear.eigen_value` と `algebra.linear.jordan` の raw stubs に展開されます。Branch 展開された spelling は source text 上で必ずしも contiguous ではないため、正確な source coverage が必要な consumer は `source_segments` を使います。

Pre-scan は raw scan が punctuation を事前に分割することを要求してはいけません。`.`、`..`、`,`、`;`、`.{`、`}` を認識するために `LexemeRun` の内部を inspect and split してよいですが、source spans を保持しなければなりません。たとえば `std.algebra.group;` を覆う raw run からも、module path `std.algebra.group` と terminating semicolon を抽出できます。

## Non-Goals

The import pre-scan must not:

- absolute or relative module paths を resolve する;
- module existence を check する;
- import cycles を compute する;
- exported symbols を load する;
- aliases conflict を decide する;
- full imported module IR を inspect する;
- ordinary module declarations を parse する.

## Error Handling

Malformed import prelude syntax emits diagnostics but should preserve as many `ImportStub`s as can be recovered deterministically.

Examples:

- missing semicolon after an import statement;
- comma の前、または relative prefix の後に module path がない;
- `as` without an alias;
- empty module path component;
- branch import list の後に `}` がない;
- unexpected token before the prelude terminator.

## Tests

Tests should cover:

- empty prelude;
- one import;
- comma-separated imports;
- branch imports;
- aliases;
- relative imports;
- prelude termination at `export`, `definition`, `registration`, and theorem-like items;
- malformed import recovery;
- no scan for imports after the prelude terminates.
