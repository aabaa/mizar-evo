# Module: import_prescan

> Canonical language: English. Japanese companion: [../ja/import_prescan.md](../ja/import_prescan.md).

## Purpose

This module extracts the module import prelude from raw lexer output.

The import pre-scan exists so the active lexical environment can be built before final token disambiguation and parsing. It must remain shallow: it recognizes import-shaped syntax and source spans, but it does not resolve modules, load symbols, or validate package visibility.

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

pub struct RawModulePath {
    pub spelling: String,
    pub components: Vec<RawModulePathComponent>,
    pub source_segments: Vec<SourceRange>,
    pub span: SourceRange,
}

pub fn scan_import_prelude(raw: &RawTokenStream) -> ImportPrelude;
```

The exact Rust names may evolve, but the ownership must remain:

- input is `RawTokenStream`;
- output is raw import spelling and source spans;
- module resolution happens elsewhere.

## Algorithm

1. Skip leading layout and module-level documentation trivia preserved by preprocessing.
2. Read contiguous top-level `import` statements.
3. For each import statement, collect one or more module alias declarations.
4. Stop at the first non-import top-level raw unit.
5. Report a syntax diagnostic for any malformed import statement that prevents reliable prelude extraction.

The scanner must not continue looking for imports after the prelude ends. Later import-shaped text is owned by the parser as a syntax error under Chapter 12 import-placement rules.

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

Branch import syntax is shorthand for multiple module paths under the same prefix. For example, `import algebra.linear.{eigen_value, jordan};` contributes raw stubs for `algebra.linear.eigen_value` and `algebra.linear.jordan`. Because a branch-expanded spelling is not necessarily contiguous in source text, consumers should use `source_segments` when they need exact source coverage.

The pre-scan must not require raw scan to split punctuation in advance. It may inspect and split inside a `LexemeRun` to recognize `.`, `..`, `,`, `;`, `.{`, and `}` while preserving source spans. For example, a raw run covering `std.algebra.group;` can still yield the module path `std.algebra.group` and the terminating semicolon.

## Non-Goals

The import pre-scan must not:

- resolve absolute or relative module paths;
- check whether a module exists;
- compute import cycles;
- load exported symbols;
- decide whether aliases conflict;
- inspect full imported module IR;
- parse ordinary module declarations.

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
- branch imports;
- aliases;
- relative imports;
- prelude termination at `export`, `definition`, `registration`, and theorem-like items;
- malformed import recovery;
- no scan for imports after the prelude terminates.
