# Module: test_and_implementation_plan

> Canonical language: English. English canonical version: [../en/test_and_implementation_plan.md](../en/test_and_implementation_plan.md).

## Purpose

この文書は `mizar-lexer` のテストケース作成と実装の推奨順序を定義します。

目的は、テスト corpus を実装より少し先行させつつ、bootstrap 実装の一時的な制約を恒久的な言語仕様として固定しないことです。Committed corpus tests は安定した language contract を表し、crate-local unit tests は feature 実装中の temporary implementation boundary を表してよい、という分担にします。

## Guiding Rules

- staged test model に従う。lexer tests は `stage = "lexical"`、`expected_phase = "lex"` に属する。
- `tests/lexical/` 以下の committed executable fixture は、隣接する `.expect.toml` を必ず持つ。
- test が claim する coverage は、test 追加前または同時に `tests/coverage/spec_trace.toml` へ追加する。
- 現在の bootstrap lexer が reject するという理由だけで、将来 valid になる language form を committed fail fixture にしない。
- temporary unsupported input behavior は crate-local unit test に置く。
- 1 fixture は 1 lexical rule をできるだけ小さく isolate する。
- `scan_raw` 導入後は、raw scanning tests と final token disambiguation tests を混ぜない。

## Traceability-First Requirement Inventory

Lexer fixtures をまとめて追加する前に、まず `doc/spec/en/02.lexical_structure.md` を checkable requirement records に分解する。

これは planning step であり、implementation が chapter 全体をすでに満たしているという claim ではない。Manifest は、tests が存在する前から known gaps を見えるようにするために使う。

Recommended workflow:

1. Chapter 2 を stable requirement ids に分解し、`tests/coverage/spec_trace.toml` に追加する。
2. 実装済みかつ test 済みの behavior は `status = "covered"` にする。
3. Known but untested behavior は `status = "planned"` にする。
4. Coverage を意図的に延期する場合だけ `status = "deferred"` を使い、`deferred_reason` を含める。
5. 各 record に `pass`, `fail`, `pass_and_fail`, `diagnostic`, `snapshot`, `property`, `manual_review` のいずれかの coverage shape を選ぶ。
6. Committed fixtures を後で追加するときに、その record の `tests = [...]` を埋め、各 `.expect.toml` の `spec_refs` に対応する id を追加する。

First inventory では、少なくとも次を含める。

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

Committed tests を持つ requirements だけを `covered` にする。たとえば current bootstrap state では、`spec.en.02.lexical.identifiers.basic` は `pass_lexical_identifier_basic_001` により covered とできる一方、reserved words、numerals、symbolic names、annotations、context-sensitive disambiguation は `planned` または `deferred` のままにする。

## Phase 0: Bootstrap Identifier Lexer -> Done

Status: current implementation surface.

Implementation surface:

- `Token`
- `TokenKind::Identifier`
- `LexError`
- `lex(&str)`

Committed corpus tests は、現在の API で表現できる安定 behavior だけを cover する。

- ASCII identifier start: `A-Z`, `a-z`, `_`
- ASCII identifier continuation: `A-Z`, `a-z`, `0-9`, `_`, `'`
- space、tab、LF の layout skip
- layout で区切られた複数 identifiers
- case-sensitive な spelling preservation

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

Crate-local unit tests では、temporary bootstrap rejection を cover してよい。

- numeral tokens が存在するまでは digit-starting input は unsupported;
- raw `LexemeRun` scanning が存在するまでは punctuation は unsupported;
- carriage return は lexer boundary では layout ではない;
- non-ASCII code-region text は、malformed fixture が意図的にこの layer に到達する場合を除き、lexer precondition の外側にある。

これらの temporary cases は、`tests/coverage/spec_trace.toml` で恒久的な fail coverage として claim しない。

Exit criteria:

- lexical pass fixtures が token expectations を持つ;
- traceability manifest がすべての new sidecar を参照する;
- `cargo test -p mizar-lexer` と metadata validation が pass する。

## Phase 1: Raw Scanner API -> Done

Target API direction:

```rust
pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError>;
```

Final token classes を増やす前に、source span を保持する raw units を導入する。

Raw tests should cover:

- empty input;
- chosen raw stream shape に応じた layout runs または layout skipping;
- identifier-shaped `LexemeRun` values;
- `+`, `*+`, `|.`, `.|` のような punctuation-shaped `LexemeRun` values;
- `x*+y` のような mixed runs;
- `0`, `42` のような numeral-like raw units;
- `@latex`, `@[` のような annotation marker shapes;
- stable diagnostics を持つ unsupported or malformed raw characters.

Important boundary:

- Raw scanner tests は import resolution、active user symbols、parser position、scoped bindings を要求しない。
- Comments は raw tokens ではない。comment stripping と documentation trivia retention は source loading and preprocessing の責務。

Recommended requirement ids:

```text
spec.en.02.lexical.raw.lexeme_runs
spec.en.02.lexical.raw.numeral_like
spec.en.02.lexical.annotations.markers
spec.en.02.lexical.preprocessing.comments_not_raw_tokens
```

Exit criteria:

- 現在の `lex(&str)` は bootstrap subset について raw scanning に delegate するか、compatibility wrapper として明示されている;
- raw spans が tests または snapshots で cover されている;
- punctuation は恒久的な lexer error として test されていない。

## Phase 2: Reserved Tables And Final Token Shell -> Done

Target implementation:

- reserved word table;
- reserved special symbol table;
- 必要に応じた reserved words、reserved symbols、numerals、user symbols、string literals、error recovery 用の final `TokenKind` variants;
- identifier、numeral、layout、reserved word、symbol-shape recognition の helper APIs.

テストでは以下を確認します。

- reserved word table の各 entry が spelling で認識される;
- reserved words は case-sensitive;
- reserved special symbols は `:=`, `<>`, `.=` , `.*`, `.{`, `@[`, `...` などの compound entries を含む;
- reserved compound symbols は longest spelling を優先する;
- reserved word を内部に含むだけの ordinary identifiers は、完全一致でない限り identifiers のまま。

Recommended requirement ids:

```text
spec.en.02.lexical.reserved_words.table
spec.en.02.lexical.reserved_words.case_sensitive
spec.en.02.lexical.reserved_symbols.table
spec.en.02.lexical.reserved_symbols.longest_compound
```

Exit criteria:

- reserved tables は data-driven、または spec と照合しやすい形で管理されている;
- tests が raw spelling recognition と context-sensitive disambiguation を区別している。

## Phase 3: Import Pre-Scan -> Done

Target API direction:

```rust
pub fn scan_import_prelude(raw: &RawTokenStream) -> ImportPrelude;
```

テストでは以下を確認します。

- empty prelude;
- one import;
- comma-separated imports;
- branch imports;
- `as` aliases;
- `.` and `..` relative imports;
- `export`, `definition`, `registration`, theorem-like items、またはその他の non-import top-level text での prelude termination;
- malformed import recovery;
- prelude termination 後の imports を scan しないこと。

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

- pre-scan は raw path spellings と source spans だけを返す;
- pre-scan は module existence、visibility、import cycles、exported symbols を resolve しない。

## Phase 4: Active Lexical Environment -> Done

Target API direction:

```rust
pub fn build_lexical_environment(
    imports: &[ResolvedImport],
    summaries: &[ModuleLexicalSummary],
) -> Result<ActiveLexicalEnvironment, LexicalEnvironmentError>;
```

テストでは以下を確認します。

- reserved tables は常に存在する;
- imported punctuation-shaped symbols が visible;
- imported identifier-shaped symbols が visible;
- `.` を含む symbols を index できる;
- equal-spelling user symbols from different imports が deterministic に reject される;
- illegal reserved-word and reserved-symbol collisions が reject される;
- environment fingerprints が deterministic input ordering に対して stable;
- longest-match lookup が identifier-shaped and punctuation-shaped symbols の両方で機能する。

Recommended requirement ids:

```text
spec.en.11.symbol_management.active_lexicon.imported_symbols
spec.en.11.symbol_management.active_lexicon.import_conflicts
spec.en.11.symbol_management.active_lexicon.reserved_collisions
spec.en.11.symbol_management.active_lexicon.fingerprint
```

Exit criteria:

- environment tests は full module IR ではなく lightweight module lexical summaries を使う;
- lookup behavior が repeated runs で deterministic。
- raw scanner tests、final token shell tests、import pre-scan tests とは分離し、crate-local lexical environment unit tests と traceability manifest で coverage を記録する。

## Phase 5: Scope Skeleton -> Done

Target API direction:

```rust
pub fn build_scope_skeleton(raw: &RawTokenStream) -> ScopeSkeleton;
```

テストでは以下を確認します。

- empty skeleton;
- simple `let x`-style binding;
- comma-separated binders;
- support され次第、`for`, `reserve`, `given` binder shapes;
- `definition`, `proof`, `now`, `end` の nested block ranges;
- malformed binders では names を捏造せず under-approximate すること;
- `ScopeLexView` が binding range 内でのみ true を返すこと;
- repeated runs で deterministic output。

Recommended requirement ids:

```text
spec.en.02.lexical.scope_override.skeleton
spec.en.04.variables_and_constants.binders.lexical_shapes
spec.en.16.theorems_and_proofs.blocks.lexical_ranges
```

Exit criteria:

- skeleton construction は full expressions を parse しない;
- skeleton diagnostics は structural and recoverable;
- skeleton output は lexical override questions だけに答えられる。

## Phase 6: Disambiguator -> Done

Target API direction:

```rust
pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream;
```

テストでは以下を確認します。

- punctuation-shaped user symbols の longest-match;
- identifier-shaped user symbol と ordinary identifier の区別;
- active identifier-shaped symbols に対する scoped identifier binding override;
- reserved word emission;
- reserved compound symbol emission;
- namespace-path context;
- compound reserved tokens、user symbols、selector access、namespace paths に関する dot disambiguation;
- string literals は string-required parser contexts でのみ認識されること;
- lexical environment 経由の import conflict reporting;
- recovery が stable な `ErrorRecovery` token と diagnostic を emit すること。
- raw token と 1 対 1 に対応する場合、`LexemeRun` を分割する場合、string literal、recovery token のいずれでも final `Token` が source span を保持すること。
- token に line/column を保存せず、span が指す text と同じ text から構築した lightweight line index を通じて source span から line/column に変換できること。

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

- disambiguator は environment、parser context、scope view を consume するが build しない;
- undefined identifiers は lexical `Identifier` tokens のままで、name resolution が later phase で reject する;
- diagnostics は order and identity が stable。
- すべての final token が source span を持つこと。contiguous な source text に由来する token では、`token.lexeme` がその span の指す source slice と一致すること。
- line/column helper は valid な `SourceSpan` から zero-based byte-column location を derive し、out-of-range offset では `None` を返す。human-facing one-based position と LSP UTF-16 position は formatting/adapter の責務とする。

## Phase 7: Regression, Property, And Fuzz Handoff -> Done

Staged lexer APIs が安定した後、より広い regression coverage を追加する。

Recommended test families:

- raw scanning panic or nondeterminism に対する committed minimized fuzz regressions;
- span coverage and concatenation/re-tokenization invariants の property tests;
- generated user-symbol overlap cases;
- generated import-conflict cases;
- format が安定した raw streams and final token streams の snapshot tests.

Promotion rule:

- Generated or fuzz-discovered failure は、committed corpus に入れる前に minimize し、stable human-readable name を付け、`.expect.toml` と pair し、`tests/coverage/spec_trace.toml` から link する。

Completion criteria:

1. 仕様書 `doc/spec/en/02.lexical_structure.md` を満たしていることをテストできているか?
2. リファクタリングは行ったか?
3. エラー処理を含めているか、また負例をテストに含めたか?
4. 複雑な複合ケースをテストに含めているか?
5. アルゴリズムをレビューしたか

アルゴリズムレビューの内容は、この計画書に重複して書かず、各 module design document に記録します。

- [raw_lexer.md](./raw_lexer.md): source preprocessing、raw scanning、reserved-shell disambiguation の流れ。
- [import_prescan.md](./import_prescan.md): import 専用 token splitter、statement parser、recovery strategy。
- [lexical_environment.md](./lexical_environment.md): active environment construction、validation、lookup、fingerprinting。
- [scope_skeleton.md](./scope_skeleton.md): frame construction、binder lifetime assignment、recovery、override semantics。
- [disambiguator.md](./disambiguator.md): raw-token processing、candidate selection、parser-context filtering、string handling、recovery。

## Review Checklist For New Lexer Tests

New lexer test を commit する前に確認すること。

- その test は soundly check できる最も早い pipeline stage を target しているか。
- Behavior は temporary implementation gap ではなく stable language contract か。
- その module が明示的な target でない限り、fixture は parser、resolver、type checker、imported library semantics に依存していないか。
- sidecar は `schema_version`, `id`, `kind`, `stage`, `domain`, `source`, `expected_outcome`, `expected_phase`, `diagnostic_codes`, `spec_refs` を含むか。
- すべての `spec_refs` entry は `tests/coverage/spec_trace.toml` に存在するか。
- manifest は sidecar へ back-reference しているか。
- fixture は十分に小さく、stable snake_case name を持つか。
- diagnostics and token expectations は deterministic か。
