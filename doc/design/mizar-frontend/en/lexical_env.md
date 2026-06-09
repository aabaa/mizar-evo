# Module: lexical_env

> Canonical language: English. Japanese companion: [../ja/lexical_env.md](../ja/lexical_env.md).

Status: implemented through task 6; the provider seam and active-environment recovery are in place.

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
    pub imports: Vec<ResolvedImportEntry>,
    pub summaries: Vec<ModuleLexicalSummary>,
    pub diagnostics: Vec<LexicalEnvironmentDiagnostic>,
}

pub struct ResolvedImportEntry {
    pub stub_ordinal: usize,
    pub stub_span: SourceRange,
    pub import: ResolvedImport,
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
    MalformedProviderProvenance { message: String },
    MalformedSummary { source: LexicalEnvironmentError },
}

pub struct LexicalEnvironmentDiagnostic {
    pub code: LexicalEnvironmentDiagnosticCode,
    pub message: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub import_ordinal: Option<usize>,
    pub module_id: Option<ModuleId>,
}

pub enum LexicalEnvironmentDiagnosticCode {
    UnresolvedImport,
    MissingSummary,
    UserSymbolImportConflict,
    InvalidUserSymbolSpelling,
    InvalidUserSymbolArity,
    ReservedWordCollision,
    ReservedSymbolCollision,
}
```

`ActiveLexicalEnvironment`, `ExportRank`, `ExportedSymbolShape`,
`LexicalEnvironmentError`, `LexicalEnvironmentFingerprint`,
`LexicalSummaryFingerprint`, `ModuleId`, `ModuleLexicalSummary`,
`ResolvedImport`, `SymbolId`, `UserSymbolArity`, `UserSymbolCandidate`,
`UserSymbolIndex`, `UserSymbolKind`, and `UserSymbolKindSet` are re-exported
from `mizar-lexer`. `ResolvedImportEntry` is frontend-owned
provenance: it wraps the lexer `ResolvedImport` with the source span and ordinal
of the `ImportStub` that produced it, so later diagnostics can point back to the
right import even when several stubs resolve to the same module. The frontend
passes only the ordered `ResolvedImport` values to
`mizar_lexer::build_lexical_environment`. Provider-returned resolved-import and
diagnostic provenance is validated against the request's `ImportStub` list
before it is used; a missing stub ordinal, stale stub span, span for another
source, or range-backed secondary anchor for another source is a malformed
provider contract rather than a recoverable import diagnostic.

Before the lexer call, the frontend canonicalizes resolved imports by `ModuleId`
in first-stub order and passes at most one `ResolvedImport` per module to the
lexer. This keeps the current lexer conflict contract unambiguous because
`LexicalEnvironmentError::UserSymbolImportConflict` reports module ids, not
frontend import ordinals. Duplicate stubs that resolve to the same module remain
available in provider diagnostics and provenance tables, but they are not passed
as duplicate active imports to the lexer.

`FrontendLexicalEnvironmentError` is owned by the frontend and wraps provider
infrastructure failures plus unrecoverable lexer structural errors.
`LexicalEnvironmentError::UserSymbolImportConflict` is the only recoverable
lexer-side case: the frontend reports it as a diagnostic, removes the later
conflicting import, and retries. All other lexer `LexicalEnvironmentError`
variants remain malformed summary/provider contracts.
`LexicalSummaryProvider` is the seam by which the build planner / resolver
supplies already-resolved imports plus lexical summaries; the frontend never
reaches into module IR to build them.

## Dependencies

- Internal: `preprocess` (provides `ImportStub`s), `lexing` (consumes the
  `ActiveLexicalEnvironment` and its fingerprint).
- External: `mizar-lexer` (`build_lexical_environment`,
  `ActiveLexicalEnvironment`, `ResolvedImport`, `ModuleLexicalSummary`,
  `UserSymbolIndex`, `ExportRank`, `LexicalEnvironmentFingerprint`,
  `LexicalEnvironmentError`, `ModuleId`), `mizar-session` (`SourceId`, `Edition`,
  `SourceRange`, `SourceAnchor`), and the build-plan / resolver service behind
  `LexicalSummaryProvider`.

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
   `ResolvedImportEntry`s and `ModuleLexicalSummary` values, recording import
   order and preserving the originating stub span for diagnostics. Validate each
   resolved entry's `stub_ordinal` and `stub_span`, plus provider-supplied
   diagnostic primary/secondary provenance, against the request before using it
   for diagnostics or canonicalization.
2. Collect provider-side diagnostics without inventing semantic facts. Add
   frontend exclusion diagnostics for import stubs that have no resolved entry
   and for canonical resolved imports whose dependency lexical summary is
   unavailable. Unresolved imports and imports with unavailable summaries are
   omitted from the lexer call because `mizar_lexer::build_lexical_environment`
   treats missing summaries as structural errors.
3. Canonicalize resolved imports by `ModuleId` in first-stub order, retaining a
   lookup from each active `ModuleId` to its canonical `ResolvedImportEntry`.
   Strip those canonical wrappers to the ordered lexer `ResolvedImport` list, then
   call `mizar_lexer::build_lexical_environment` with the resolved imports and
   summaries to assemble the `UserSymbolIndex` and `LexicalEnvironmentFingerprint`.
4. If the lexer returns `UserSymbolImportConflict { spelling, earlier_import,
   later_import }`, convert it into a `LexicalEnvironmentDiagnostic`, using the
   canonical `ResolvedImportEntry` for `later_import` as the primary span and the
   canonical entry for `earlier_import` as a secondary anchor. Remove the later
   conflicting module from the active import set and retry. Repeat this bounded
   retry until the lexer succeeds or there are no imports
   left; each retry removes at most one module, so it is deterministic and
   terminates in at most the original canonical import count. If either module id
   is missing from the canonical lookup, treat that as a provider/frontend
   invariant failure rather than guessing a span.
5. Treat every other lexer `LexicalEnvironmentError` as an unrecoverable malformed
   summary/provider contract and return
   `FrontendLexicalEnvironmentError::MalformedSummary`. Missing summaries should
   already have been diagnosed and omitted before the lexer call; seeing
   `MissingModuleSummary` here is therefore a provider/frontend invariant failure,
   not a user-facing recoverable diagnostic.
6. Return the environment, fingerprint, and merged diagnostics.

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
rather than failing the whole file. Provider provenance in resolved imports or
provider diagnostics that does not match the current request is reported as
`FrontendLexicalEnvironmentError::MalformedProviderProvenance`, because using it
would attach diagnostics to the wrong import or source. Recoverable lexer-side
handling is narrowed to `UserSymbolImportConflict`: the frontend diagnoses the
later conflicting import, adds the earlier import as secondary context, removes
the later import, and retries environment construction. Invalid
exported symbol spelling, invalid arity, reserved-word/symbol collisions,
inconsistent duplicate summaries, and unexpected missing summaries are hard
malformed-summary failures because the frontend cannot safely infer which subset
of a malformed dependency summary is usable. Import legality (visibility, export
rank conflicts beyond lexical shape) is deferred to module resolution and is
never decided here.

## Tests

Key scenarios:

- import stubs plus module lexical summaries produce a `UserSymbolIndex` whose
  candidates carry the correct provenance and import ordinal;
- diagnostics for unresolved imports, missing summaries, and user-symbol import
  conflicts point at the originating canonical `ImportStub` span;
- provider-returned resolved-import or diagnostic provenance with a missing stub
  ordinal, stale span, or foreign source is rejected as a hard provider-contract
  failure;
- duplicate resolved imports of the same module are canonicalized before the lexer
  call and do not create spurious user-symbol import conflicts;
- user-symbol import conflicts remove the later conflicting import, retry
  deterministically, and terminate after at most one retry per original canonical
  import;
- non-conflict lexer `LexicalEnvironmentError`s become
  `FrontendLexicalEnvironmentError::MalformedSummary` rather than silently
  dropping arbitrary summaries;
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
- The active lexical environment holds only compact `ModuleLexicalSummary`
  projections of imported modules, never their definitions or full module IR (the
  frontend never reaches into module IR — see Purpose and "Active Lexical
  Environment"). The `LexicalSummaryProvider` supplies these summaries for the
  current file's resolved imports rather than loading the whole import closure;
  this is the frontend-seam form of the resident-set memory model's "hold
  interfaces, not bodies" rule (spec
  [§12.6.3](../../../spec/en/12.modules_and_namespaces.md#1263-memory-model); see
  also [mizar-lexer lexical_environment.md](../../mizar-lexer/en/lexical_environment.md)).
