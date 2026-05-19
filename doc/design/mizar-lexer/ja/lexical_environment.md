# Module: lexical_environment

> Canonical language: English. English canonical version: [../en/lexical_environment.md](../en/lexical_environment.md).

## Purpose

This module builds the file-scoped active lexical environment consumed by token disambiguation.

Environment は built-in reserved tables と、import prelude で指定された modules の exported lexical symbol summaries を結合します。imports は top-of-file prelude に限定されるため、この environment は source file body 全体で安定します。

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

1. Built-in reserved words and reserved special symbols から開始する。
2. import-prelude order で resolved imports 由来の exported symbol shapes を追加する。
3. reserved-word or reserved-symbol collision rules に違反する conflicts を reject or mark する。
4. longest-match 用に trie などの prefix index を構築する。
5. import order, imported module summary fingerprints, built-in table versions から deterministic fingerprint を計算する。

Current implementation notes:

- `ModuleId` and `SymbolId` are lightweight string newtypes in `mizar-lexer`; they do not imply module existence or semantic resolution.
- `ModuleLexicalSummary.exported_symbols` is assumed to be canonicalized by its producer; summary construction, not environment construction, owns sorting and summary fingerprint stability.
- `UserSymbolCandidate.source_module` は lexical summary 由来の defining/exporting provenance を保持し、`imported_module` は conflict diagnostics のために current file の resolved import で指定された module を記録する。
- `.` remains the spec-defined exception to the reserved-special-symbol collision rule; other exact reserved symbol spellings are rejected.
- equal-spelling imported user symbols from different imports are rejected as environment construction conflicts.
- fingerprints use an internal stable byte hasher rather than process-randomized hashing.

## Non-Goals

This module must not:

- source text を parse する;
- import syntax を resolve する;
- full module IR を load する;
- local scope overrides を decide する;
- symbol use が type-correct か decide する;
- overload winners を choose する.

## Error Handling

Errors are environment construction failures, not tokenization failures:

- missing module lexical summary for a resolved import;
- inconsistent duplicate summary for the same module id;
- exported symbol collides illegally with a reserved word or reserved special symbol;
- different imports が export する equal-spelling user symbols は conflict する;
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
