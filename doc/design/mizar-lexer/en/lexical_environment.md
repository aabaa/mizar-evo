# Module: lexical_environment

> Canonical language: English. Japanese companion: [../ja/lexical_environment.md](../ja/lexical_environment.md).

## Purpose

This module builds the file-scoped active lexical environment consumed by token disambiguation.

The environment combines built-in reserved tables with exported lexical symbol summaries from modules named by the import prelude. It is stable for the whole source file body because imports are restricted to the top-of-file prelude.

## Public API

Implemented API direction:

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

The module may expose lookup helpers optimized for longest-match:

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

1. Start from built-in reserved words and reserved special symbols.
2. Add exported symbol shapes from each resolved import in import-prelude order.
3. Reject or mark conflicts that violate reserved-word or reserved-symbol collision rules.
4. Build lookup structures, preferably tries or equivalent prefix indexes, for longest-match.
5. Compute a deterministic fingerprint from import order, imported module summary fingerprints, and built-in table versions.

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
- nondeterministic import order or summary order.

Ambiguous same-spelling user symbols from the same imported module remain representable as deterministic candidates; same-spelling symbols from different imports are rejected as conflicts.

## Tests

Tests should cover:

- reserved tables are always present;
- imported symbols are visible;
- equal-spelling user symbols from different imports are rejected deterministically;
- reserved collisions are rejected;
- environment fingerprints are stable under deterministic input ordering;
- the environment can answer longest-match queries for identifier-shaped and punctuation-shaped symbols.
