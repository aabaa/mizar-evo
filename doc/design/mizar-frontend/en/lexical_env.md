# Module: lexical_env

> Canonical language: English. Japanese companion: [../ja/lexical_env.md](../ja/lexical_env.md).

Status: planned.

## Purpose

This module implements the frontend pipeline Step 3 (active lexical environment
construction) as a frontend-adjacent coordination service. It turns the shallow
`ImportStub`s from preprocessing into an `ActiveLexicalEnvironment` that the
lexer uses for context-sensitive longest-match disambiguation.

It coordinates between preprocessing output, the build planner / module
resolver (which turn import stubs into resolved imports and lightweight module
lexical summaries), and `mizar_lexer::build_lexical_environment`. It does not
perform full import resolution: package/module existence, visibility, and export
legality belong to module resolution.

See
[architecture/en/02.source_and_frontend.md](../../architecture/en/02.source_and_frontend.md)
"Step 3: Build ActiveLexicalEnvironment" and "Import Pre-Scan Is Shallow".

## Public API

```rust
pub struct LexicalEnvironmentRequest<'a> {
    pub source_id: SourceId,
    pub import_stubs: &'a [ImportStub],
    pub edition: Edition,
}

pub trait LexicalSummaryProvider {
    fn resolve_imports(
        &self,
        request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError>;
}

pub struct ResolvedImports {
    pub imports: Vec<ResolvedImport>,
    pub summaries: Vec<ModuleLexicalSummary>,
    pub diagnostics: Vec<LexicalEnvironmentDiagnostic>,
}

pub fn build_active_lexical_environment(
    request: &LexicalEnvironmentRequest<'_>,
    provider: &dyn LexicalSummaryProvider,
) -> Result<ActiveLexicalEnvironmentResult, FrontendLexicalEnvironmentError>;

pub struct ActiveLexicalEnvironmentResult {
    pub environment: ActiveLexicalEnvironment,
    pub fingerprint: LexicalEnvironmentFingerprint,
    pub diagnostics: Vec<LexicalEnvironmentDiagnostic>,
}

pub enum FrontendLexicalEnvironmentError {
    ProviderUnavailable { message: String },
    MalformedSummary { source: LexicalEnvironmentError },
}
```

`ResolvedImport`, `ModuleLexicalSummary`, `ActiveLexicalEnvironment`,
`LexicalEnvironmentFingerprint`, and `LexicalEnvironmentError` are re-exported
from `mizar-lexer`. `FrontendLexicalEnvironmentError` is owned by the frontend
and wraps provider infrastructure failures plus unrecoverable lexer structural
errors. `LexicalSummaryProvider` is the seam by which the build planner /
resolver supplies already-resolved imports plus lexical summaries; the frontend
never reaches into module IR to build them.

## Dependencies

- Internal: `preprocess` (provides `ImportStub`s), `lexing` (consumes the
  `ActiveLexicalEnvironment` and its fingerprint).
- External: `mizar-lexer` (`build_lexical_environment`,
  `ActiveLexicalEnvironment`, `ResolvedImport`, `ModuleLexicalSummary`,
  `UserSymbolIndex`, `ExportRank`, `LexicalEnvironmentFingerprint`,
  `LexicalEnvironmentError`), `mizar-session` (`SourceId`, `Edition`), and the
  build-plan / resolver service behind `LexicalSummaryProvider`.

This module is consumed by lexing and, through the fingerprint, by the
incremental cache.

## Data Structures

### Active Lexical Environment

`ActiveLexicalEnvironment` (owned by `mizar-lexer`) holds the reserved-word and
reserved-symbol tables, a `UserSymbolIndex` of user-defined symbolic names
exported by imported modules, and a `LexicalEnvironmentFingerprint`. It records
import order for deterministic fingerprints and provenance, and it records
symbol provenance for diagnostics. Equal-spelling imported user-symbol conflicts
are not resolved by import-order tie-breaking in the frontend; the provider or
`mizar-lexer` reports them as lexical-environment conflicts. It is
lexer-local: it captures lexical shape and provenance, not full module IR or
semantic applicability.

### Fingerprint and Cache Key

`LexicalEnvironmentFingerprint` summarizes resolved imports, dependency
lexical-summary fingerprints, and import order. It is the cache key for the
active lexical environment and a component of the `TokenStream` cache key, so a
dependency export change can correctly invalidate tokenization even when the
local file is unchanged.

## Algorithm / Logic

### Build the active lexical environment

1. Ask the `LexicalSummaryProvider` to resolve the `ImportStub`s into
   `ResolvedImport`s and `ModuleLexicalSummary` values, recording import order.
2. Collect provider-side diagnostics (unresolved import, missing dependency
   lexical summary) without inventing semantic facts. The provider returns only
   resolved imports that have matching summaries; unresolved imports and imports
   with unavailable summaries are omitted from the lexer call because
   `mizar_lexer::build_lexical_environment` treats missing summaries as structural
   errors.
3. Call `mizar_lexer::build_lexical_environment` with the reserved tables,
   resolved imports, and summaries to assemble the `UserSymbolIndex` and
   `LexicalEnvironmentFingerprint`.
4. Convert recoverable import-level lexer errors, such as deterministic
   user-symbol import conflicts, into `LexicalEnvironmentDiagnostic`s and degrade
   to a smaller active environment. Return hard `FrontendLexicalEnvironmentError`s
   only for provider infrastructure failures or malformed summary data that cannot
   be safely degraded.
5. Return the environment, fingerprint, and merged diagnostics.

If an import cannot be resolved, the environment is still built from the imports
that did resolve, so the rest of the file can be tokenized; the failure is a
diagnostic, not a hard stop.

## Error Handling

`LexicalEnvironmentError` (from `mizar-lexer`) covers structural failures such
as conflicting summary fingerprints or malformed summary data. Provider
infrastructure failures are not expressible with that lexer-owned enum, so the
frontend wraps hard failures in `FrontendLexicalEnvironmentError`.
Provider-side issues — an import that resolves to no module, or a dependency
whose lexical summary is unavailable — are carried as
`LexicalEnvironmentDiagnostic`s and the affected imports are omitted before
calling the lexer, so the pipeline degrades to a smaller active environment
rather than failing the whole file. Recoverable lexer-side lexical conflicts are
likewise surfaced as diagnostics with the conflicting imported symbol/module
excluded from the active environment. Import legality (visibility, export rank
conflicts beyond lexical shape) is deferred to module resolution and is never
decided here.

## Tests

Key scenarios:

- import stubs plus module lexical summaries produce a `UserSymbolIndex` whose
  candidates carry the correct provenance and import ordinal;
- equal-spelling user symbols imported from different modules produce a
  deterministic lexical-environment conflict, while overlapping symbols with
  different spellings are still available for lexer longest-match selection;
- an unresolved import degrades to a smaller environment with a diagnostic, and
  the remaining symbols still load;
- the `LexicalEnvironmentFingerprint` changes when a dependency lexical summary
  changes and is stable when only comments change in the local file;
- reserved words and reserved symbols are always present regardless of imports.

## Constraints and Assumptions

- This module coordinates import resolution; it does not perform it.
- Full import legality belongs to module resolution; only lexical shape and
  provenance are captured here.
- The active lexical environment can change token boundaries, so its fingerprint
  is part of the `TokenStream` cache key.
- Reserved tables are built-in and independent of imports.
