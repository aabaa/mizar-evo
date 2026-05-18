# Module: import_prescan

> Canonical language: English. English canonical version: [../en/import_prescan.md](../en/import_prescan.md).

## Purpose

This module extracts the module import prelude from raw lexer output.

Import pre-scan は、final token disambiguation and parsing の前に active lexical environment を構築するために存在します。これは shallow でなければなりません。import-shaped syntax と source spans は認識しますが、modules の resolve、symbols の load、package visibility の validate は行いません。

## Public API

Expected API direction:

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

pub fn scan_import_prelude(raw: &RawTokenStream) -> ImportPrelude;
```

Ownership must remain:

- input is `RawTokenStream`;
- output is raw import spelling and source spans;
- module resolution happens elsewhere.

## Algorithm

1. Leading layout and module-level documentation trivia を skip する。
2. contiguous top-level `import` statements を読む。
3. 各 import statement から 1 個以上の module alias declarations を collect する。
4. 最初の non-import top-level raw unit で stop する。
5. reliable prelude extraction を妨げる malformed import statement に syntax diagnostic を出す。

Scanner は prelude 終了後に import を探してはいけません。後続の import-shaped text は、Chapter 12 import-placement rules に従い parser が syntax error として扱います。

## Accepted Syntax

The pre-scan recognizes the Chapter 12 import syntax:

```ebnf
import_stmt       ::= "import" module_alias_decl { "," module_alias_decl } ";" ;
module_alias_decl ::= module_path [ "as" module_identifier ] ;
module_path       ::= [ relative_prefix ] module_identifier { "." module_identifier } ;
relative_prefix   ::= "." | ".." ;
module_identifier ::= identifier ;
```

Recognition is spelling-based. `import` and `as` are reserved-word spellings, and module path components are identifier-shaped raw lexemes.

Pre-scan は raw scan が punctuation を事前に分割することを要求してはいけません。`.`、`..`、`,`、`;` を認識するために `LexemeRun` の内部を inspect and split してよいですが、source spans を保持しなければなりません。たとえば `std.algebra.group;` を覆う raw run からも、module path `std.algebra.group` と terminating semicolon を抽出できます。

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
- `as` without an alias;
- empty module path component;
- unexpected token before the prelude terminator.

## Tests

Tests should cover:

- empty prelude;
- one import;
- comma-separated imports;
- aliases;
- relative imports;
- prelude termination at `export`, `definition`, `registration`, and theorem-like items;
- malformed import recovery;
- no scan for imports after the prelude terminates.
