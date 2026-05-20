# Module: test_and_implementation_plan

> Canonical language: English. Japanese companion: [../ja/test_and_implementation_plan.md](../ja/test_and_implementation_plan.md).

## Purpose

This document defines the recommended order for adding `mizar-lexer` tests and implementation.

The goal is to keep the test corpus ahead of the implementation without encoding temporary bootstrap limitations as permanent language behavior. Committed corpus tests should describe stable language contracts. Crate-local unit tests may describe temporary implementation boundaries while a feature is still being built.

## Guiding Rules

- Follow the staged test model: lexer tests belong to `stage = "lexical"` and `expected_phase = "lex"`.
- Every committed executable fixture under `tests/lexical/` must have an adjacent `.expect.toml`.
- Add traceability entries in `tests/coverage/spec_trace.toml` before or with the tests that claim them.
- Do not add future-valid language forms as committed fail fixtures merely because the current bootstrap lexer rejects them.
- Use crate-local unit tests for temporary unsupported input behavior.
- Prefer small fixtures that isolate one lexical rule.
- Keep final token disambiguation tests out of raw scanning tests once `scan_raw` exists.

## Traceability-First Requirement Inventory

Before adding a batch of lexer fixtures, first inventory `doc/spec/en/02.lexical_structure.md` into checkable requirement records.

This is a planning step, not a claim that the implementation already satisfies the whole chapter. The manifest should make known gaps visible before the tests exist.

Recommended workflow:

1. Split Chapter 2 into stable requirement ids in `tests/coverage/spec_trace.toml`.
2. Mark implemented and tested behavior as `status = "covered"`.
3. Mark known but untested behavior as `status = "planned"`.
4. Use `status = "deferred"` only when coverage is intentionally postponed, and include `deferred_reason`.
5. Choose a coverage shape for each record: `pass`, `fail`, `pass_and_fail`, `diagnostic`, `snapshot`, `property`, or `manual_review`.
6. Add committed fixtures later by filling the record's `tests = [...]` list and adding matching `spec_refs` in each `.expect.toml`.

The first inventory should include at least:

- character set and source preconditions;
- layout and LF-only boundary;
- reserved words;
- reserved special symbols;
- user-defined symbolic names;
- longest-match behavior;
- dot disambiguation;
- identifiers;
- numerals;
- string literal context sensitivity;
- file and module naming lexical constraints;
- comments and annotation markers;
- lexical preprocessing boundaries.

Only requirements with committed tests should be `covered`. For example, in the current bootstrap state, `spec.en.02.lexical.identifiers.basic` may be covered by `pass_lexical_identifier_basic_001`, while reserved words, numerals, symbolic names, annotations, and context-sensitive disambiguation should remain `planned` or `deferred`.

## Phase 0: Bootstrap Identifier Lexer -> Done

Status: current implementation surface.

Implementation surface:

- `Token`
- `TokenKind::Identifier`
- `LexError`
- `lex(&str)`

Committed corpus tests should cover only stable behavior expressible through the current API:

- ASCII identifier start: `A-Z`, `a-z`, `_`
- ASCII identifier continuation: `A-Z`, `a-z`, `0-9`, `_`, `'`
- layout skipping for space, tab, and LF
- multiple identifiers separated by layout
- case-sensitive spelling preservation

Recommended fixtures:

```text
tests/lexical/pass/pass_lexical_identifier_basic_001.src
tests/lexical/pass/pass_lexical_identifier_start_ascii_001.src
tests/lexical/pass/pass_lexical_identifier_continue_ascii_001.src
tests/lexical/pass/pass_lexical_identifier_apostrophe_001.src
tests/lexical/pass/pass_lexical_layout_space_tab_lf_001.src
tests/lexical/pass/pass_lexical_identifier_case_sensitive_001.src
```

Recommended requirement ids:

```text
spec.en.02.lexical.identifiers.basic
spec.en.02.lexical.identifiers.start_ascii
spec.en.02.lexical.identifiers.continue_ascii
spec.en.02.lexical.identifiers.apostrophe
spec.en.02.lexical.layout.space_tab_lf
spec.en.02.lexical.identifiers.case_sensitive
```

Crate-local unit tests may cover temporary bootstrap rejection:

- digit-starting input is unsupported until numeral tokens exist;
- punctuation is unsupported until raw `LexemeRun` scanning exists;
- carriage return is not layout at the lexer boundary;
- non-ASCII code-region text is outside the lexer precondition unless a malformed fixture intentionally reaches this layer.

These temporary cases should not claim permanent fail coverage in `tests/coverage/spec_trace.toml`.

Exit criteria:

- lexical pass fixtures have token expectations;
- traceability manifest references every new sidecar;
- `cargo test -p mizar-lexer` and metadata validation pass.

## Phase 1: Raw Scanner API -> Done

Target API direction:

```rust
pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError>;
```

Implementation should introduce source-span-preserving raw units before adding final token classes.

Raw tests should cover:

- empty input;
- layout runs or layout skipping according to the chosen raw stream shape;
- identifier-shaped `LexemeRun` values;
- punctuation-shaped `LexemeRun` values such as `+`, `*+`, `|.`, `.|`;
- mixed runs such as `x*+y`;
- numeral-like raw units such as `0` and `42`;
- annotation marker shapes such as `@latex` and `@[`;
- unsupported or malformed raw characters with stable diagnostics.

Important boundary:

- Raw scanner tests must not require import resolution, active user symbols, parser position, or scoped bindings.
- Comments should not be raw tokens. Source loading and preprocessing own comment stripping and documentation trivia retention.

Recommended requirement ids:

```text
spec.en.02.lexical.raw.lexeme_runs
spec.en.02.lexical.raw.numeral_like
spec.en.02.lexical.annotations.markers
spec.en.02.lexical.preprocessing.comments_not_raw_tokens
```

Exit criteria:

- current `lex(&str)` either delegates to raw scanning for the bootstrap subset or is clearly documented as a compatibility wrapper;
- raw spans are covered by tests or snapshots;
- punctuation is no longer tested as a permanent lexer error.

## Phase 2: Reserved Tables And Final Token Shell -> Done

Target implementation:

- reserved word table;
- reserved special symbol table;
- final `TokenKind` variants for reserved words, reserved symbols, numerals, user symbols, string literals, and error recovery as needed;
- helper APIs for identifier, numeral, layout, reserved word, and symbol-shape recognition.

Tests should cover:

- every reserved word table entry is recognized by spelling;
- reserved words are case-sensitive;
- reserved special symbols include compound entries such as `:=`, `<>`, `.=` , `.*`, `.{`, `@[`, and `...`;
- reserved compound symbols prefer the longest spelling;
- ordinary identifiers that merely contain reserved words remain identifiers when not exactly equal.

Recommended requirement ids:

```text
spec.en.02.lexical.reserved_words.table
spec.en.02.lexical.reserved_words.case_sensitive
spec.en.02.lexical.reserved_symbols.table
spec.en.02.lexical.reserved_symbols.longest_compound
```

Exit criteria:

- reserved tables are data-driven or otherwise easy to audit against the spec;
- tests distinguish raw spelling recognition from context-sensitive disambiguation.

## Phase 3: Import Pre-Scan -> Done

Target API direction:

```rust
pub fn scan_import_prelude(raw: &RawTokenStream) -> ImportPrelude;
```

Tests should cover:

- empty prelude;
- one import;
- comma-separated imports;
- branch imports;
- aliases using `as`;
- relative imports using `.` and `..`;
- prelude termination at `export`, `definition`, `registration`, theorem-like items, or other non-import top-level text;
- malformed import recovery;
- no scan for imports after the prelude terminates.

Recommended requirement ids:

```text
spec.en.12.modules.import_prelude.basic
spec.en.12.modules.import_prelude.alias
spec.en.12.modules.import_prelude.relative
spec.en.12.modules.import_prelude.branch
spec.en.12.modules.import_prelude.termination
spec.en.12.modules.import_prelude.malformed_recovery
```

Exit criteria:

- pre-scan returns raw path spellings and source spans only;
- pre-scan does not resolve module existence, visibility, import cycles, or exported symbols.

## Phase 4: Active Lexical Environment -> Done

Target API direction:

```rust
pub fn build_lexical_environment(
    imports: &[ResolvedImport],
    summaries: &[ModuleLexicalSummary],
) -> Result<ActiveLexicalEnvironment, LexicalEnvironmentError>;
```

Tests should cover:

- reserved tables are always present;
- imported punctuation-shaped symbols are visible;
- imported identifier-shaped symbols are visible;
- symbols containing `.` can be indexed;
- equal-spelling user symbols from different imports are rejected deterministically;
- illegal reserved-word and reserved-symbol collisions are rejected;
- environment fingerprints are stable for deterministic input ordering;
- longest-match lookup works for identifier-shaped and punctuation-shaped symbols.

Recommended requirement ids:

```text
spec.en.11.symbol_management.active_lexicon.imported_symbols
spec.en.11.symbol_management.active_lexicon.import_conflicts
spec.en.11.symbol_management.active_lexicon.reserved_collisions
spec.en.11.symbol_management.active_lexicon.fingerprint
```

Exit criteria:

- environment tests use lightweight module lexical summaries, not full module IR;
- lookup behavior is deterministic under repeated runs.
- raw scanner tests, final token shell tests, and import pre-scan tests remain separate; coverage is recorded through crate-local lexical environment unit tests and the traceability manifest.

## Phase 5: Scope Skeleton -> Done

Target API direction:

```rust
pub fn build_scope_skeleton(raw: &RawTokenStream) -> ScopeSkeleton;
```

Tests should cover:

- empty skeleton;
- simple `let x`-style binding;
- comma-separated binders;
- `for`, `reserve`, and `given` binder shapes as they become supported;
- nested block ranges for `definition`, `proof`, `now`, and `end`;
- malformed binders under-approximate rather than inventing names;
- `ScopeLexView` returns true only inside the binding range;
- deterministic output for repeated runs.

Recommended requirement ids:

```text
spec.en.02.lexical.scope_override.skeleton
spec.en.04.variables_and_constants.binders.lexical_shapes
spec.en.16.theorems_and_proofs.blocks.lexical_ranges
```

Exit criteria:

- skeleton construction does not parse full expressions;
- skeleton diagnostics are structural and recoverable;
- skeleton output can answer only lexical override questions.

## Phase 6: Disambiguator

Target API direction:

```rust
pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream;
```

Tests should cover:

- longest-match over punctuation-shaped user symbols;
- identifier-shaped user symbol versus ordinary identifier;
- scoped identifier binding override for active identifier-shaped symbols;
- reserved word emission;
- reserved compound symbol emission;
- namespace-path context;
- dot disambiguation for compound reserved tokens, user symbols, selector access, and namespace paths;
- string literals only in string-required parser contexts;
- import conflict reporting through the lexical environment;
- recovery emits stable `Error` tokens and diagnostics.

Recommended requirement ids:

```text
spec.en.02.lexical.longest_match
spec.en.02.lexical.user_symbols.identifier_shaped
spec.en.02.lexical.user_symbols.punctuation_shaped
spec.en.02.lexical.dot_disambiguation
spec.en.02.lexical.string_literals.context_sensitive
spec.en.02.lexical.error_recovery
```

Exit criteria:

- disambiguator consumes environment, parser context, and scope view without building them;
- undefined identifiers remain lexical `Identifier` tokens and are rejected later by name resolution;
- diagnostics are stable in order and identity.

## Phase 7: Regression, Property, And Fuzz Handoff

After the staged lexer APIs are stable, add broader regression coverage.

Recommended test families:

- committed minimized fuzz regressions for raw scanning panics or nondeterminism;
- property tests for span coverage and concatenation/re-tokenization invariants;
- generated user-symbol overlap cases;
- generated import-conflict cases;
- snapshot tests for raw streams and final token streams when the format is stable.

Promotion rule:

- A generated or fuzz-discovered failure should be minimized, given a stable human-readable name, paired with `.expect.toml`, and linked from `tests/coverage/spec_trace.toml` before becoming part of the committed corpus.

## Review Checklist For New Lexer Tests

Before committing a new lexer test:

- Does the test target the earliest pipeline stage that can soundly check it?
- Is the behavior a stable language contract rather than a temporary implementation gap?
- Does the fixture avoid relying on parser, resolver, type checker, or imported library semantics unless that module is the explicit target?
- Does the sidecar include `schema_version`, `id`, `kind`, `stage`, `domain`, `source`, `expected_outcome`, `expected_phase`, `diagnostic_codes`, and `spec_refs`?
- Does every `spec_refs` entry exist in `tests/coverage/spec_trace.toml`?
- Does the manifest point back to the sidecar?
- Is the fixture minimal and named with stable snake_case?
- Are diagnostics and token expectations deterministic?
