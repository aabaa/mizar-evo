# Architecture: Source and Frontend

> Canonical language: English. Japanese companion: [../ja/source_and_frontend.md](../ja/source_and_frontend.md).

## Purpose

This document defines how the Mizar Evo frontend loads `.miz` source files and turns them into comments, tokens, and a source-shaped syntax tree for module and name resolution.

It refines phases 1-3 of [00.pipeline_overview.md](./00.pipeline_overview.md). In particular, it defines the boundaries for context-sensitive lexing, active lexicons, user-defined symbols, dot handling, fully tokenized string literals, doc comments, and annotation attachment.

## Context

- [00.pipeline_overview.md](./00.pipeline_overview.md) — overall pipeline; this document refines phases 1-3
- [ir_layers.md](./ir_layers.md) — `SourceUnit`, `PreprocessedSource`, `TokenStream`, `SurfaceAst`
- [doc/spec/en/02.lexical_structure.md](../../../spec/en/02.lexical_structure.md) — lexical structure, comments, annotations, lexer/parser responsibility split
- [doc/spec/en/11.symbol_management.md](../../../spec/en/11.symbol_management.md) — user-defined symbols and active lexicon
- [doc/spec/en/12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md) — module paths and namespace references
- [doc/spec/en/16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md) — citations and proof syntax
- [doc/spec/en/21.source_code_annotation_and_atp.md](../../../spec/en/21.source_code_annotation_and_atp.md) — library annotations and display annotations
- [doc/spec/en/22.error_handling_and_diagnostics.md](../../../spec/en/22.error_handling_and_diagnostics.md) — syntax diagnostics and source spans
- [doc/spec/en/23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md) — package layout and build lifecycle

### Pipeline Position

| Phase | Input | Output | This Document Defines |
|---|---|---|---|
| 1. Source Loading / Preprocessing | `.miz` files, `BuildPlan` | `SourceUnit`, `PreprocessedSource` | file validation, line map, comment/doc comment separation, import pre-scan |
| 2. Lexing | `PreprocessedSource`, active lexicon seed | `TokenStream` | reserved tokens, user symbols, longest-match, source-preserving tokens |
| 3. Parsing | `TokenStream` | `SurfaceAst` | syntax tree, annotation attachment, recovery |

## Design Decisions

### Frontend Produces Syntax, Not Semantics

The frontend must not perform semantic name resolution or type checking.

It may:

- validate UTF-8 and code-region ASCII constraints;
- identify comments and doc comments;
- pre-scan import declarations enough to request an active lexicon seed;
- tokenize using reserved words, reserved symbols, and active user symbols;
- parse source into a surface-shaped AST;
- perform syntax-level error recovery.

It must not:

- decide whether a dotted path denotes a namespace or a selected term when that requires scope knowledge;
- choose overload winners;
- infer expression types;
- fire cluster registrations;
- generate proof obligations.

### Import Pre-Scan Is Shallow

The frontend needs import information before final tokenization because imported modules extend the active lexicon. However, full import resolution belongs to the module resolver.

Therefore, preprocessing performs only a shallow import pre-scan:

- recognizes top-level `import` forms using a restricted lexical mode;
- extracts raw dotted module paths and source spans;
- reports only malformed import syntax that prevents lexicon loading;
- delegates package/module existence, visibility, and export checks to later phases.

The build planner / module resolver uses these `ImportStub`s to construct an `ActiveLexiconSeed`, then lexing proceeds with the symbols visible from those imports.

### Lexing Uses an Active Lexicon but Remains Syntax-Oriented

The lexer applies the longest-match rule against:

1. reserved words;
2. reserved special symbols;
3. user-defined symbols from the active lexicon;
4. identifier and numeral rules.

When a token shape matches both identifier syntax and an active user symbol, the lexer emits a symbol token. If the shape is not active as a user symbol, it emits an identifier token.

The lexer does not know whether a symbol use is legally applicable in a term/formula context. That is a parser/checker concern.

### Dot Handling Is Split Across Lexer, Parser, and Resolver

`.` is deliberately not resolved in one place.

| Role | Layer | Rule |
|---|---|---|
| compound reserved tokens `.{`, `.*`, `.=`, `...` | lexer | recognized as single reserved tokens |
| user-defined symbols containing `.` | lexer | recognized by active lexicon and longest-match |
| selector access / update | parser + resolver | parsed as a possible selector chain in term contexts |
| namespace separator | parser + resolver | parsed in import, citation, annotation, and qualified-name contexts |
| variable shadowing of namespace paths | resolver | local variables win over namespace components |

The frontend may produce an ambiguous source-shaped representation such as `DottedPathOrSelection` where semantic scope is required. The resolver later commits to namespace or selector interpretation.

### String Literals Are Fully Tokenized by the Lexer

`"` and `'` are admissible user-symbol characters, so the lexer cannot treat them as string delimiters globally.

The frontend still produces a complete `TokenStream` before parsing. To do this safely, the lexer uses a small grammar-derived `StringPositionRecognizer` that recognizes string-required positions without requiring parser callbacks.

String literals are recognized only at grammar positions that explicitly require a string literal, such as:

- annotation arguments like `@latex("...")`;
- operator declaration arguments such as `infix_operator("+", left, 80)`;
- future grammar positions registered as string-valued arguments.

Outside these positions, quote characters remain ordinary symbol characters governed by the active lexicon and longest-match rule.

The parser consumes already-tokenized `StringLiteral` tokens. It never drives the lexer cursor directly.

### Comments and Doc Comments Are Source Metadata

Comments are removed from lexical input, but they are not discarded.

- ordinary comments are retained for formatting/debug tooling only;
- doc comments are attached to the following documentable item when possible;
- doc comment attachment remains syntactic and may later be rejected if the target is not documentable;
- structured doc tags are parsed enough to preserve tag name, raw arguments, and source spans.

### Annotations Are Parser-Owned Syntax

Annotations are part of the surface syntax and must preserve source location and raw arguments.

Preprocessing does not collect annotations into a separate metadata channel. Annotation tokens remain in `lexical_text`, are emitted into `TokenStream`, and are parsed by the parser into `SurfaceAst` annotation nodes.

The parser validates annotation syntax, but semantic effects are deferred:

- `@[...]` library annotations attach to items as raw library labels;
- `@latex`, `@proof_hint`, `@show_*`, `@eval` are represented as annotation nodes;
- annotation registry validation may happen in the parser for syntax, but meaning is interpreted by later phases;
- annotations must not change core language semantics.

## Frontend Pipeline

### Step 1: Load SourceUnit

Input:

- package/workspace `BuildPlan`;
- `.miz` file path;
- expected package root and `src/` root.

Output:

- `SourceUnit`.

Responsibilities:

- read source bytes;
- validate UTF-8;
- compute `source_hash`;
- derive module path from file path;
- build `LineMap` using Unicode scalar column rules;
- emit file-level diagnostics.

Failure examples:

- unreadable source file;
- invalid UTF-8;
- file outside package `src/` root;
- invalid `.miz` module filename.

### Step 2: Preprocess Source

Input:

- `SourceUnit`.

Output:

- `PreprocessedSource`.

Responsibilities:

- validate code-region ASCII while allowing Unicode in comments and annotations;
- identify and remove ordinary comments from lexical input;
- preserve doc comments with source spans;
- preserve annotation syntax in lexical input for parser ownership;
- shallow-scan top-level imports;
- preserve exact mapping from lexical text back to source ranges.

Failure examples:

- unterminated block comment;
- illegal non-ASCII character in code region;
- import pre-scan failure that prevents active lexicon construction.

### Step 3: Build ActiveLexiconSeed

Input:

- `ImportStub`s from preprocessing;
- package/module indexes available from the build plan;
- already-built dependency symbol exports.

Output:

- `ActiveLexiconSeed`.

Responsibilities:

- gather user-defined symbolic names exported by imported modules;
- include built-in reserved symbols and keyword table;
- record import order for equal-length user-symbol tie breaking;
- record symbol provenance for diagnostics.

This step is a frontend-adjacent service: it uses shallow imports from the frontend, but full import legality remains part of module resolution.

### Step 4: Lex

Input:

- `PreprocessedSource`;
- `ActiveLexiconSeed`.

Output:

- `TokenStream`.

Responsibilities:

- emit reserved words and reserved special symbols;
- emit user-defined symbols under longest-match;
- emit identifiers and numerals;
- emit `StringLiteral` tokens at string-required positions;
- preserve source spans and original spelling;
- expose lexical diagnostics without losing recoverable tokens.

Failure examples:

- unknown or malformed token sequence;
- invalid numeral form;
- malformed string literal in a string-required position.

### Step 5: Parse

Input:

- `TokenStream`.

Output:

- `SurfaceAst`.

Responsibilities:

- parse modules, definitions, registrations, statements, terms, formulas, theorems, proofs, algorithms;
- parse annotation argument lists and attach annotations to syntax nodes;
- attach doc comments to nearby documentable items;
- preserve source order and source ranges;
- recover from syntax errors at synchronization points such as `;`, `end`, and top-level item keywords.

Failure examples:

- unexpected token;
- unmatched delimiter;
- missing `end`;
- expected string literal token is missing.

## Interface Definitions

### SourceUnit

```rust
struct SourceUnit {
    source_id: SourceId,
    package_id: PackageId,
    module_path: ModulePath,
    file_path: PathBuf,
    source_text: Arc<str>,
    source_hash: Hash,
    line_map: LineMap,
}
```

### PreprocessedSource

```rust
struct PreprocessedSource {
    source_id: SourceId,
    lexical_text: LexicalText,
    comments: Vec<Comment>,
    doc_comments: Vec<DocComment>,
    import_stubs: Vec<ImportStub>,
    source_map: LexicalSourceMap,
}
```

### ImportStub

```rust
struct ImportStub {
    source_range: SourceRange,
    raw_path: Vec<IdentifierText>,
    kind: ImportKind,
}

enum ImportKind {
    Module,
    Open,
    Reexport,
}
```

`ImportStub` is not a resolved import. It is enough to request lexicon entries and to produce good diagnostics if lexicon loading fails.

### ActiveLexiconSeed

```rust
struct ActiveLexiconSeed {
    reserved_words: ReservedWordTable,
    reserved_symbols: ReservedSymbolTable,
    user_symbols: Vec<LexiconEntry>,
}

struct LexiconEntry {
    spelling: SymbolText,
    symbol_id: Option<SymbolId>,
    source_module: ModuleId,
    import_rank: ImportRank,
}
```

`symbol_id` may be absent for symbols loaded from an index before full module resolution has completed. The resolver later validates and canonicalizes symbol identity.

### TokenStream

```rust
struct TokenStream {
    source_id: SourceId,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

struct Token {
    kind: TokenKind,
    span: SourceRange,
    text: InternedText,
}

enum TokenKind {
    ReservedWord(ReservedWord),
    ReservedSymbol(ReservedSymbol),
    UserSymbol(UserSymbolToken),
    Identifier,
    Numeral,
    StringLiteral,
    Error,
}
```

`StringLiteral` appears only where the lexer recognizes a grammar-defined string-required position.

### SurfaceAst

```rust
struct SurfaceAst {
    source_id: SourceId,
    module: SurfaceModule,
    nodes: AstArena<SurfaceNode>,
    trivia: TriviaMap,
    diagnostics: Vec<Diagnostic>,
}
```

### FrontendOutput

```rust
struct FrontendOutput {
    source: SourceUnit,
    preprocessed: PreprocessedSource,
    tokens: TokenStream,
    ast: Option<SurfaceAst>,
    diagnostics: Vec<Diagnostic>,
}
```

`ast = None` means parsing could not recover enough structure for later phases. Lexical and syntax diagnostics are still returned.

## Error Recovery

Frontend recovery aims to report multiple useful diagnostics without inventing semantic facts.

Lexer recovery:

- emits `TokenKind::Error` for malformed spans when possible;
- resumes at whitespace, reserved delimiters, or line boundaries;
- preserves the malformed source range for diagnostics.

Parser recovery:

- synchronizes at `;`, `end`, `definition`, `registration`, `theorem`, `lemma`, `proof`, `algorithm`, and EOF;
- creates explicit error nodes where a construct is missing;
- avoids creating fake identifiers or fake resolved symbols;
- keeps doc comments near their original source location, even if attachment fails.
- keeps malformed annotations as syntax-level error nodes when possible.

Recovered AST nodes must be marked so later phases can skip or degrade gracefully.

## Diagnostics

Frontend diagnostics use the syntax/lexical ranges from [doc/spec/en/22.error_handling_and_diagnostics.md](../../../spec/en/22.error_handling_and_diagnostics.md).

| Diagnostic Class | Phase | Example |
|---|---|---|
| lexical precondition | source loading / preprocessing | invalid UTF-8, non-ASCII code character |
| comment structure | preprocessing | unterminated block comment |
| tokenization | lexing | malformed literal, unknown token |
| syntax | parsing | unexpected token, missing `end`, unmatched delimiter |
| annotation syntax | parsing | malformed annotation argument list |

Frontend diagnostics must include:

- stable diagnostic code;
- primary source span;
- secondary span where useful, such as the opening delimiter for a missing close;
- recovery note when later diagnostics may be affected.

Frontend diagnostics must not claim semantic facts such as "undefined symbol" or "ambiguous overload"; those belong to later phases.

## Incrementality

Frontend cache keys are layered.

| Output | Cache Key |
|---|---|
| `SourceUnit` | file path, source bytes |
| `PreprocessedSource` | `source_hash`, frontend version |
| `ActiveLexiconSeed` | import stubs, dependency export hashes, import order |
| `TokenStream` | preprocessed hash, active lexicon hash |
| `SurfaceAst` | token stream hash, parser version, edition |

Important invalidation rules:

- comment-only edits invalidate `PreprocessedSource` and documentation metadata, but may allow semantic outputs to be reused if lexical text is unchanged;
- import edits invalidate active lexicon, token stream, AST, and all semantic layers for that source file;
- dependency symbol export edits can invalidate tokenization even when the local source file is unchanged;
- parser version and language edition are part of the AST cache key.

## Alternatives Considered

### Alternative 1: Pure Lexer Before Import Resolution

Lex all source files without considering imported user symbols, then reinterpret tokens during parsing or name resolution.

This would make lexing simpler, but it conflicts with identifier-shaped user symbols and longest-match rules. It would also force later phases to split or merge token sequences, making source spans and parser recovery fragile.

### Alternative 2: Full Module Resolution Before Lexing

Resolve imports completely before lexing each file.

This gives the lexer precise symbol identities, but it creates a cycle: parsing is needed to understand module contents, while lexing needs imported symbols. The adopted shallow pre-scan breaks the cycle while preserving enough information to build the active lexicon.

### Alternative 3: Lexer Decides All Dot Roles

Classify `.` as selector, namespace separator, or user functor directly in the lexer.

This fails when variable shadowing or term context determines the correct interpretation. The adopted split keeps compound tokens and active user symbols in the lexer, while leaving selector/namespace decisions to parser and resolver.

### Alternative 4: Treat Quotes as Global String Delimiters

Always lex `'...'` and `"..."` as string literals.

This would conflict with user symbols such as postfix inverse notation. The adopted contextual lexer recognizes string literals only at grammar positions that require them while still producing a complete `TokenStream` before parsing.

## Adopted Approach

Use a staged frontend:

1. load source and build source mapping;
2. preprocess comments, doc comments, and shallow imports;
3. build an active lexicon seed from shallow imports and dependency exports;
4. lex with longest-match against reserved and active symbols, including contextual string literal recognition;
5. parse into source-shaped `SurfaceAst` with explicit recovery nodes.

This design keeps frontend output source-faithful while avoiding semantic commitments that belong to module resolution and type checking.

## Affected Modules

The Rust crate layout is not finalized. Expected module specs include:

- `doc/design/mizar-frontend/source.md` — source loading, UTF-8 validation, line map
- `doc/design/mizar-frontend/preprocess.md` — comments, doc comments, import pre-scan
- `doc/design/mizar-frontend/lexicon.md` — active lexicon seed and longest-match tables
- `doc/design/mizar-frontend/lexer.md` — tokenization
- `doc/design/mizar-frontend/parser.md` — parser and recovery
- `doc/design/mizar-syntax/ast.md` — `SurfaceAst` node definitions
- `doc/design/mizar-diagnostics/source_map.md` — source ranges and line/column mapping
- `doc/design/mizar-resolve/imports.md` — full import resolution consuming `ImportStub`

## Constraints and Assumptions

- Source files are UTF-8 and end with `.miz`.
- Code regions are ASCII; comments and annotations may contain Unicode.
- Import pre-scan is intentionally incomplete and cannot replace module resolution.
- Active lexicon can affect token boundaries, so tokenization depends on dependency export hashes.
- The lexer recognizes compound reserved tokens and active user symbols, but not semantic selector/namespace roles.
- The lexer emits complete `StringLiteral` tokens only at grammar positions that explicitly require strings.
- The parser owns annotation syntax and attachment; preprocessing does not collect annotations as a separate metadata channel.
- `SurfaceAst` is source-shaped and may contain recovery nodes; later semantic phases must tolerate or reject these explicitly.
- Frontend artifacts are internal compiler data, not stable public build artifacts.
