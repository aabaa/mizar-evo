# Module: import_prescan

> Canonical language: English. Japanese companion: [../ja/import_prescan.md](../ja/import_prescan.md).

## Purpose

This module extracts the module import prelude from raw lexer output.

The import pre-scan exists so the active lexical environment can be built before final token disambiguation and parsing. It must remain shallow: it recognizes import-shaped syntax and source spans, but it does not resolve modules, load symbols, or validate package visibility.

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

The ownership boundary is:

- input is `RawTokenStream`;
- output is raw import spelling and source spans;
- module resolution happens elsewhere.

## Algorithm

The implemented scanner is a small token splitter plus a recoverable statement parser.

1. Convert `RawTokenStream` into import-pre-scan tokens. Layout is ignored. `LexemeRun` values are split into `Word`, `.`, `..`, `,`, `;`, `*`, `{`, `}`, and `Other` pieces while preserving byte spans. `NumeralLike`, annotation markers, and raw errors are represented as `Other`.
2. Initialize the prelude end position to the first non-layout token start, or `0` for an empty stream.
3. While the cursor sees the word `import`, parse one import statement. The parser consumes the `import` word, then repeatedly parses comma-separated module alias declarations until it reaches `;`, EOF, or a malformed statement boundary.
4. `parse_module_path` accepts an optional relative prefix (`.` or `..`), then one or more identifier-shaped path components separated by dots. A dot followed by `{` is reserved for branch import parsing and does not become part of the base path.
5. `parse_module_alias_decls` adds an optional `as alias` suffix. The alias must be identifier-shaped. Missing aliases are diagnostic-only; the import stub is still kept when the path was recovered.
6. Branch imports consume `base.{child, other}` and expand them into multiple `ImportStub` values. Because the expanded spelling is not contiguous in source text, `source_segments` records both the base span and the branch component span.
7. On malformed input, diagnostics are attached to the smallest reliable span. If at least one declaration was recovered but no semicolon follows, the parser reports `MissingSemicolon`; otherwise it reports `UnexpectedToken` or a more specific path/alias diagnostic and recovers to the statement end.
8. Scanning stops permanently at the first token that is not the start of a top-level import statement.

The scanner must not continue looking for imports after the prelude ends. Later import-shaped text is owned by the parser as a syntax error under Chapter 12 import-placement rules.

The algorithm intentionally does not ask the reserved table whether `import` or `as` are currently legal parser tokens. It is a pre-parser pass whose only job is to gather enough raw import shape to let later phases resolve modules and build an active lexical environment.

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
- missing module path before a comma or after a relative prefix;
- `as` without an alias;
- empty module path component;
- missing `}` after a branch import list;
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
