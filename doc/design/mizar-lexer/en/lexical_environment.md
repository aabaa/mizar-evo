# Module: lexical_environment

> Canonical language: English. Japanese companion: [../ja/lexical_environment.md](../ja/lexical_environment.md).

## Purpose

This module builds the active lexical environment consumed by token
disambiguation.

The implemented environment combines built-in reserved tables with exported
lexical symbol summaries from modules named by the import prelude. It also
provides a source-position-sensitive layer for current-module lexical
declarations: local declarations extend the imported environment only after
their declaration item is complete.

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

pub struct LocalLexicalDeclarations {
    pub user_symbols: Vec<LocalUserSymbolDeclaration>,
    pub operator_declarations: Vec<LocalOperatorDeclaration>,
}

pub struct LocalUserSymbolDeclaration {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub export_rank: ExportRank,
    pub kind: UserSymbolKind,
    pub arity: UserSymbolArity,
    pub declared_at: SourceSpan,
    pub activation_start: SourcePos,
}

pub struct LocalOperatorDeclaration {
    pub spelling: String,
    pub source_module: ModuleId,
    pub declared_at: SourceSpan,
    pub activation_start: SourcePos,
    pub operator: Option<ExportedOperatorMetadata>,
}

pub struct ActiveOperatorMetadata {
    pub spelling: String,
    pub source_module: ModuleId,
    pub declared_at: SourceSpan,
    pub activation_start: SourcePos,
    pub operator: ExportedOperatorMetadata,
}

pub fn collect_local_lexical_declarations(
    raw: &RawTokenStream,
    current_module: ModuleId,
) -> LocalLexicalDeclarations;

pub fn is_constructor_name_spelling(value: &str) -> bool;
```

The module exposes lookup helpers used by longest-match disambiguation:

```rust
impl ActiveLexicalEnvironment {
    pub fn reserved_word(&self, spelling: &str) -> Option<&'static str>;
    pub fn reserved_symbol(&self, spelling: &str) -> Option<&'static str>;
    pub fn user_symbol(&self, spelling: &str) -> Option<&UserSymbolCandidate>;
    pub fn visible_user_symbols_at(
        &self,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<UserSymbolCandidate>;
    pub fn longest_user_symbol_at(&self, input: &str, start: usize) -> Vec<UserSymbolCandidate>;
    pub fn user_symbols_at(
        &self,
        spelling: &str,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<UserSymbolCandidate>;
    pub fn longest_user_symbol_at_position(
        &self,
        input: &str,
        start: usize,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<UserSymbolCandidate>;
    pub fn operator_metadata_at(
        &self,
        spelling: &str,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<ActiveOperatorMetadata>;
    pub fn visible_operator_metadata_at(
        &self,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<ActiveOperatorMetadata>;
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
    pub kind: UserSymbolKind,
    pub arity: UserSymbolArity,
}

pub struct UserSymbolCandidate {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub imported_module: ModuleId,
    pub import_ordinal: usize,
    pub export_rank: ExportRank,
    pub kind: UserSymbolKind,
    pub arity: UserSymbolArity,
}
```

`UserSymbolKind` records the parser/resolver category of a visible lexical
entry. Under the current language specification, arbitrary `user_symbol`
notation is admitted directly only for functors and predicates. Mode,
attribute, and structure entries must use `constructor_name` spellings:
ordinary identifiers or readable hyphenated names such as `1-sorted`,
`R-module`, or `C-star-algebraic`. Structure selectors are identifiers and are
not accepted as exported lexical-summary entries. The legacy generic
`Constructor` kind is also rejected at the summary boundary; constructor-like
lexical entries must be classified as `Mode`, `Attribute`, or `Structure`.
`UserSymbolArity` records the argument-count shape as an exact count, bounded
range, or lower-bounded range. These are parser/resolver-facing summaries, not
full type signatures.

The active environment should support:

- identifier-shaped functor/predicate symbols and constructor names;
- punctuation-shaped functor/predicate symbols;
- symbols containing `.`;
- import conflict detection for equal-spelling imported candidates;
- stable provenance for diagnostics.
- symbol kind and arity metadata for downstream parser and resolver phases;
- range-sensitive activation for current-module declarations.

`ModuleLexicalSummary` is a canonical producer-side artifact. The component that creates a
summary must normalize `exported_symbols` into deterministic order before handing it to the lexer
environment builder. The canonical order is by lexical identity and provenance, at minimum:

1. `spelling`
2. `source_module`
3. `symbol_id`
4. `kind`
5. `arity`
6. `export_rank`

`build_lexical_environment` relies on that contract and does not reorder a summary internally. This
keeps the environment fingerprint sensitive to the imported module's canonical lexical summary
rather than to an ad hoc order chosen by the environment builder.

This producer-side summary order is independent of the active-candidate order used inside
`UserSymbolIndex`. Once summaries are imported, same-spelling candidates are sorted for lookup and
diagnostic stability by import ordinal, export rank, kind, arity, source module, and symbol id.

## Algorithm

The implemented builder constructs a deterministic lookup object from
already-resolved imports.

1. Index `ModuleLexicalSummary` values by `ModuleId`. Duplicate summaries are accepted only if they are byte-for-byte equivalent as Rust values; inconsistent duplicates fail construction.
2. Seed a stable FNV-style fingerprint with a version string and the built-in reserved word and reserved symbol tables in their declared order.
3. Walk `ResolvedImport` values in import-prelude order. For each import, require a matching lexical summary and add the import ordinal, module id, and summary fingerprint to the active-environment fingerprint.
4. For every exported symbol shape in that summary, validate the spelling and arity before indexing it. Functor and predicate spellings must be valid `user_symbol` spellings. Mode, attribute, and structure spellings must be valid `constructor_name` spellings. Selector and generic constructor summary entries are rejected because selectors remain identifiers and constructor-like entries must be typed by their semantic category. Every exported spelling must avoid reserved-word collisions and reserved-special-symbol collisions; the spec-defined `.` exception is accepted only for functors. The arity shape must not have a maximum lower than its minimum.
5. Convert the exported shape into a `UserSymbolCandidate`, preserving both the source module that exported the symbol and the imported module through which the current file sees it, plus the symbol kind and arity metadata.
6. Insert the candidate into `UserSymbolIndex`. Equal spellings from different imports are rejected as `UserSymbolImportConflict`. Equal spellings from the same import remain representable as overload candidates and are stored in the active-candidate order described above.
7. Return `ActiveLexicalEnvironment` containing borrowed reserved tables, the completed user-symbol index, and the deterministic fingerprint.

`UserSymbolIndex` keeps a canonical `BTreeMap<String, Vec<UserSymbolCandidate>>` for exact-spelling lookup, deterministic ordering, and conflict diagnostics. It also maintains an ASCII byte trie over the same spellings for longest-prefix lookup. `longest_user_symbol_at` walks the trie from the requested byte offset, remembers the deepest terminal node, and returns the candidates from the visible import ordinal for that spelling. Candidate discovery is therefore proportional to the scanned spelling length plus the number of returned candidates, while preserving the previous public lookup semantics.

Current implementation notes:

- `ModuleId` and `SymbolId` are lightweight string newtypes in `mizar-lexer`; they do not imply module existence or semantic resolution.
- `ModuleLexicalSummary.exported_symbols` is assumed to be canonicalized by its producer; summary construction, not environment construction, owns sorting and summary fingerprint stability.
- `UserSymbolCandidate.source_module` preserves the defining/exporting provenance from the lexical summary, while `imported_module` records the module named by the current file's resolved import for conflict diagnostics.
- `UserSymbolCandidate.kind` and `UserSymbolCandidate.arity` are retained on every active candidate so later parser and resolver phases can filter or distinguish same-spelling overloads without rebuilding module summaries.
- `.` remains the spec-defined functor-only exception to the reserved-special-symbol collision rule; other exact reserved symbol spellings are rejected.
- equal-spelling imported user symbols from different imports are rejected as environment construction conflicts.
- fingerprints use an internal stable byte hasher rather than process-randomized hashing, and include symbol kind and arity metadata.
- the trie is an internal acceleration structure; it does not affect fingerprinting or summary canonicalization.
- `collect_local_lexical_declarations` performs a shallow raw-token prepass
  and records source-ordered current-module activation events for `pred`,
  `func`, `mode`, `attr`, `struct`, `synonym`, `antonym`, and operator
  declarations. It does not parse expressions, types, proofs, or full
  declaration bodies.
- For local `pred` and `func` declarations, the prepass records direct
  notation spellings from identifier-shaped call, prefix, infix, and postfix
  patterns, and records each non-delimiter symbolic piece of a symbolic or
  circumfix notation pattern. Contiguous hyphenated predicate/functor notation
  such as `foo-bar` is recorded as one user-symbol spelling when the adjacent
  pieces are not the simple single-locus `x-y` operator shape.
- For local `mode`, `attr`, and `struct` declarations, the prepass records
  only constructor-name spellings. Contiguous readable hyphenated constructor
  names are recorded as whole spellings, and operator-like symbolic names are
  not introduced as local constructor entries. In local `attr` declarations,
  the prepass first applies the attribute `param_prefix` split when the prefix
  is a numeral or a name found by its shallow preceding-`let` parameter scan,
  including later qualified segments and implicit names in the same `let`
  declaration. For shapes such as `n-dimensional`, `(row,col)-size`, and
  `implicit-shaped`, it records only the constructor-name suffix
  (`dimensional`, `size`, `shaped`) as the attribute spelling. Commas inside a
  parameterized type expression such as `Function of REAL, REAL` or
  `Function[REAL, REAL]` are not treated as declaring additional attribute
  parameters, while later explicit segments such as `X be set` can still
  contribute an uppercase one-letter parameter name. For `of`/`over` type
  argument lists followed by another explicit segment, the scan avoids treating
  earlier type arguments as parameters and only resumes at the segment boundary
  it can identify, including comma-separated value-name lists such as
  `g, k be set`. The scan stops at `such` and `by` so trailing condition or
  reference lists do not introduce fake attribute parameters.
- Local `synonym` and `antonym` declarations use a conservative shallow
  classification. If the alias side or original side contains clear
  operator-like notation evidence, the alias is recorded as predicate/functor
  style notation. Otherwise the alias head is recorded only if it is a
  constructor-name spelling. Full semantic alias-family classification remains
  resolver-owned; this prepass does not use type information to reinterpret an
  ambiguous word-only alias as arbitrary symbolic notation.
- Local user-symbol candidates are visible to
  `longest_user_symbol_at_position` only when the queried source position is
  greater than or equal to the activation offset at the end of the declaring
  item. For definitions with declaration-owned correctness/property clauses,
  that completion boundary includes the trailing clause and its proof block.
  The declaration's own header/definiens, its correctness trail, and all earlier
  text therefore cannot see the introduced spelling.
- Same-spelling local and imported entries are combined as overload candidates
  for downstream resolver phases. Local entries do not lexically shadow
  imported entries.
- `private` and `public` are ignored by the local lexical prepass. `algorithm`,
  `redefine`, inline `deffunc`, inline `defpred`, structure selectors, and
  field/property names do not introduce local lexer user-symbol dictionary
  entries.
- Operator declarations are recorded separately as activation events and do
  not introduce user-symbol candidates. Imported operator metadata is exposed
  as active from lexical byte offset `0`. Local operator metadata is exposed
  only when the declaration has parsed metadata, the declaration is active at
  the query position, and the declared spelling already had at least one
  active functor candidate with the matching arity at the operator declaration
  spelling. This implements the no-forward-reference rule without producing
  lexer diagnostics for invalid operator declarations.
- Parser-facing operator metadata is spelling-level metadata, not an overload
  root selection. Same-spelling imported and local functor candidates remain
  overload candidates for downstream resolution, while `operator_metadata_at`
  returns deterministic spelling/fixity/precedence entries sorted so later
  activation points are considered before earlier ones. Link-time conflict
  diagnostics for incompatible same-spelling metadata remain outside the lexer.

## Non-Goals

This module must not:

- parse source text;
- resolve import syntax;
- load full module IR;
- decide local scope overrides;
- decide whether a symbol use is type-correct;
- choose overload resolution results.

Because the active environment holds only compact `ModuleLexicalSummary` projections (symbol spelling, kind, and arity) of imported modules — never their definitions, proof bodies, or full module IR (see Non-Goals above) — it is the lexer-level expression of the resident-set memory model's "hold interfaces, not bodies" rule (spec [§12.6.3](../../../spec/en/12.modules_and_namespaces.md#1263-memory-model); architecture [03.module_and_symbol_resolution.md](../../architecture/en/03.module_and_symbol_resolution.md)). Imported summaries are supplied on demand through the `LexicalSummaryProvider` seam rather than loaded eagerly for the whole import closure.

## Error Handling

Errors are environment construction failures, not tokenization failures:

- missing module lexical summary for a resolved import;
- inconsistent duplicate summary for the same module id;
- exported symbol collides illegally with a reserved word or reserved special symbol;
- equal-spelling user symbols exported by different imports conflict;
- invalid user-symbol spelling.
- invalid constructor-name spelling for mode, attribute, or structure entries;
- invalid user-symbol arity shape.
- unsupported selector or generic constructor lexical-summary entry kind.

Ambiguous same-spelling user symbols from the same imported module remain representable as deterministic candidates; same-spelling symbols from different imports are rejected as conflicts. Import order and summary order are not diagnosed as errors, but they are part of the deterministic input contract and are reflected in the environment fingerprint.

## Cache and Fingerprint Contract

`LexicalEnvironmentFingerprint` must cover every active-lexical-environment input that can change token boundaries or environment-owned parser-facing token classification:

- reserved tables;
- imported lexical summaries;
- visible user-symbol spellings;
- user-symbol conflict decisions;
- import order where relevant;
- symbol provenance relevant to deterministic diagnostics.

Parser-facing lexical filters used by disambiguation are request inputs outside
the active lexical environment. `TokenStream` reuse composes the active
environment fingerprint with the parser lexing plan / filter hash; changing
those filters invalidates the token stream cache without rebuilding the
environment itself.

The lexer must not include semantic resolution, overload root choice, type facts, registration firing, or proof information in lexical fingerprints. Missing or unsupported lexical-summary fingerprint data forces frontend cache miss. A changed active lexical environment invalidates `TokenStream` reuse even when local source text is unchanged.

## Tests

Tests should cover:

- reserved tables are always present;
- imported symbols are visible;
- equal-spelling user symbols from different imports are rejected deterministically;
- reserved collisions are rejected;
- functor/predicate entries accept free-form notation while mode/attribute/
  structure entries require constructor names;
- selector and generic constructor summary entries are rejected;
- local readable hyphenated constructor names are recorded as whole spellings;
- local parameterized attribute declarations record the constructor-name suffix,
  not the `param_prefix` spelling;
- environment fingerprints are stable under deterministic input ordering;
- the environment can answer longest-match queries for identifier-shaped and punctuation-shaped symbols.
- trie-backed lookup preserves longest-match behavior with many imported symbols and overlapping spellings.
- kind and arity metadata are preserved for same-spelling overload candidates.
- environment fingerprints change when kind or arity metadata changes.
- local current-module declarations are inactive before their declaring item,
  inactive in their own header/definiens, and active at later source
  positions;
- same-spelling local/import candidates are both retained;
- `private`/`public` do not affect local lexical activation;
- operator declarations, `deffunc`, `defpred`, `algorithm`, and `redefine` do
  not introduce local user-symbol entries;
- parser-facing operator metadata queries include imported metadata, local
  before-use metadata, declaration-after-use non-activation, private/public
  no-op visibility, no-forward-reference rejection, and same-spelling overload
  preservation;
- synonym/antonym prepass activation is derived from the alias pattern before
  `for`, not from the original pattern after `for`.
