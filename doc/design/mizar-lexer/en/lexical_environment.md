# Module: lexical_environment

> Canonical language: English. Japanese companion: [../ja/lexical_environment.md](../ja/lexical_environment.md).

## Purpose

This module builds the file-scoped active lexical environment consumed by token disambiguation.

The environment combines built-in reserved tables with exported lexical symbol summaries from modules named by the import prelude. It is stable for the whole source file body because imports are restricted to the top-of-file prelude.

## Public API

Expected API direction:

```rust
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

pub fn build_lexical_environment(
    imports: &[ResolvedImport],
    summaries: &[ModuleLexicalSummary],
) -> Result<ActiveLexicalEnvironment, LexicalEnvironmentError>;
```

The module may expose lookup helpers optimized for longest-match:

```rust
impl ActiveLexicalEnvironment {
    pub fn reserved_word(&self, spelling: &str) -> Option<ReservedWord>;
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
```

The active environment should support:

- identifier-shaped symbols;
- punctuation-shaped symbols;
- symbols containing `.`;
- import-order tie breaking for equal-length candidates;
- stable provenance for diagnostics.

## Algorithm

1. Start from built-in reserved words and reserved special symbols.
2. Add exported symbol shapes from each resolved import in import-prelude order.
3. Reject or mark conflicts that violate reserved-word or reserved-symbol collision rules.
4. Build lookup structures, preferably tries or equivalent prefix indexes, for longest-match.
5. Compute a deterministic fingerprint from import order, imported module summary fingerprints, and built-in table versions.

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
- nondeterministic import order or summary order.

Ambiguous equal-length user symbols that are valid under import shadowing should remain representable as deterministic candidates.

## Tests

Tests should cover:

- reserved tables are always present;
- imported symbols are visible;
- later imports shadow earlier equal-length user symbols deterministically;
- reserved collisions are rejected;
- environment fingerprints are stable under deterministic input ordering;
- the environment can answer longest-match queries for identifier-shaped and punctuation-shaped symbols.
