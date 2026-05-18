# Module: lexical_environment

> Canonical language: English. English canonical version: [../en/lexical_environment.md](../en/lexical_environment.md).

## Purpose

This module builds the file-scoped active lexical environment consumed by token disambiguation.

Environment は built-in reserved tables と、import prelude で指定された modules の exported lexical symbol summaries を結合します。imports は top-of-file prelude に限定されるため、この environment は source file body 全体で安定します。

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

1. Built-in reserved words and reserved special symbols から開始する。
2. import-prelude order で resolved imports 由来の exported symbol shapes を追加する。
3. reserved-word or reserved-symbol collision rules に違反する conflicts を reject or mark する。
4. longest-match 用に trie などの prefix index を構築する。
5. import order, imported module summary fingerprints, built-in table versions から deterministic fingerprint を計算する。

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
