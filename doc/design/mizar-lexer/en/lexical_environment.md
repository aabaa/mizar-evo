# Module: lexical_environment

> Canonical language: English. Japanese companion: [../ja/lexical_environment.md](../ja/lexical_environment.md).

## Purpose

This module builds the file-scoped active lexical environment consumed by token disambiguation.

The environment combines built-in reserved tables with exported lexical symbol summaries from modules named by the import prelude. It is stable for the whole source file body because imports are restricted to the top-of-file prelude.

## Public API

Implemented API:

```rust
pub type ReservedWordTable = &'static [&'static str];
pub type ReservedSymbolTable = &'static [&'static str];

pub struct ActiveLexicalEnvironment {
    pub reserved_words: ReservedWordTable,
    pub reserved_symbols: ReservedSymbolTable,
    pub user_symbols: UserSymbolIndex,
    pub fingerprint: LexicalEnvironmentFingerprint,
}

pub struct ModuleLexicalSummary {
    pub module_id: ModuleId,
    pub exported_symbols: Vec<ExportedSymbolShape>,
    pub fingerprint: LexicalSummaryFingerprint,
}

pub struct ResolvedImport {
    pub module_id: ModuleId,
}

pub fn build_lexical_environment(
    imports: &[ResolvedImport],
    summaries: &[ModuleLexicalSummary],
) -> Result<ActiveLexicalEnvironment, LexicalEnvironmentError>;
```

The module exposes lookup helpers used by longest-match disambiguation:

```rust
impl ActiveLexicalEnvironment {
    pub fn reserved_word(&self, spelling: &str) -> Option<&'static str>;
    pub fn reserved_symbol(&self, spelling: &str) -> Option<&'static str>;
    pub fn user_symbol(&self, spelling: &str) -> Option<&UserSymbolCandidate>;
    pub fn longest_user_symbol_at(&self, input: &str, start: usize) -> Vec<UserSymbolCandidate>;
}
```

## Data Model

`ExportedSymbolShape` stores lexical shape, not full semantic IR:

```rust
pub struct ExportedSymbolShape {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub export_rank: ExportRank,
}

pub struct UserSymbolCandidate {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub imported_module: ModuleId,
    pub import_ordinal: usize,
    pub export_rank: ExportRank,
}
```

The active environment should support:

- identifier-shaped symbols;
- punctuation-shaped symbols;
- symbols containing `.`;
- import conflict detection for equal-spelling imported candidates;
- stable provenance for diagnostics.

`ModuleLexicalSummary` is a canonical producer-side artifact. The component that creates a
summary must normalize `exported_symbols` into deterministic order before handing it to the lexer
environment builder. The canonical order is by lexical identity and provenance, at minimum:

1. `spelling`
2. `source_module`
3. `symbol_id`
4. `export_rank`

`build_lexical_environment` relies on that contract and does not reorder a summary internally. This
keeps the environment fingerprint sensitive to the imported module's canonical lexical summary
rather than to an ad hoc order chosen by the environment builder.

## Algorithm

The implemented builder constructs a deterministic lookup object from already-resolved imports.

1. Index `ModuleLexicalSummary` values by `ModuleId`. Duplicate summaries are accepted only if they are byte-for-byte equivalent as Rust values; inconsistent duplicates fail construction.
2. Seed a stable FNV-style fingerprint with a version string and the built-in reserved word and reserved symbol tables in their declared order.
3. Walk `ResolvedImport` values in import-prelude order. For each import, require a matching lexical summary and add the import ordinal, module id, and summary fingerprint to the active-environment fingerprint.
4. For every exported symbol shape in that summary, validate the spelling before indexing it. The spelling must be a user-symbol spelling, must not collide with a reserved word, and must not collide with a reserved special symbol except for the spec-defined `.` exception.
5. Convert the exported shape into a `UserSymbolCandidate`, preserving both the source module that exported the symbol and the imported module through which the current file sees it.
6. Insert the candidate into `UserSymbolIndex`. Equal spellings from different imports are rejected as `UserSymbolImportConflict`. Equal spellings from the same import remain representable as overload candidates and are stored deterministically by import ordinal, export rank, source module, and symbol id.
7. Return `ActiveLexicalEnvironment` containing borrowed reserved tables, the completed user-symbol index, and the deterministic fingerprint.

Lookup uses a `BTreeMap<String, Vec<UserSymbolCandidate>>` rather than a trie. `longest_user_symbol_at` scans the map, keeps only spellings that prefix-match the source slice at the requested byte offset, retains the greatest byte length, and returns the candidates from the visible import ordinal for that spelling. The result is deterministic and sufficient for the current corpus size; a trie can replace it later without changing the public semantics.

Current implementation notes:

- `ModuleId` and `SymbolId` are lightweight string newtypes in `mizar-lexer`; they do not imply module existence or semantic resolution.
- `ModuleLexicalSummary.exported_symbols` is assumed to be canonicalized by its producer; summary construction, not environment construction, owns sorting and summary fingerprint stability.
- `UserSymbolCandidate.source_module` preserves the defining/exporting provenance from the lexical summary, while `imported_module` records the module named by the current file's resolved import for conflict diagnostics.
- `.` remains the spec-defined exception to the reserved-special-symbol collision rule; other exact reserved symbol spellings are rejected.
- equal-spelling imported user symbols from different imports are rejected as environment construction conflicts.
- fingerprints use an internal stable byte hasher rather than process-randomized hashing.

## Non-Goals

This module must not:

- parse source text;
- resolve import syntax;
- load full module IR;
- decide local scope overrides;
- decide whether a symbol use is type-correct;
- choose overload winners.

## Error Handling

Errors are environment construction failures, not tokenization failures:

- missing module lexical summary for a resolved import;
- inconsistent duplicate summary for the same module id;
- exported symbol collides illegally with a reserved word or reserved special symbol;
- equal-spelling user symbols exported by different imports conflict;
- invalid user-symbol spelling.

Ambiguous same-spelling user symbols from the same imported module remain representable as deterministic candidates; same-spelling symbols from different imports are rejected as conflicts. Import order and summary order are not diagnosed as errors, but they are part of the deterministic input contract and are reflected in the environment fingerprint.

## Tests

Tests should cover:

- reserved tables are always present;
- imported symbols are visible;
- equal-spelling user symbols from different imports are rejected deterministically;
- reserved collisions are rejected;
- environment fingerprints are stable under deterministic input ordering;
- the environment can answer longest-match queries for identifier-shaped and punctuation-shaped symbols.
