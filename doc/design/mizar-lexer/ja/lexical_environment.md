# Module: lexical_environment

> Canonical language: English. English canonical version: [../en/lexical_environment.md](../en/lexical_environment.md).

## Purpose

この module は、token disambiguation が参照する file-scoped active lexical environment を構築します。

Environment は built-in reserved table と、import prelude で指定された module の exported lexical symbol summary を結合します。import は top-of-file prelude に限定されるため、この environment は source file body 全体で安定します。

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
```

この module は、longest-match disambiguation 用の lookup helper を公開します。

```rust
impl ActiveLexicalEnvironment {
    pub fn reserved_word(&self, spelling: &str) -> Option<&'static str>;
    pub fn reserved_symbol(&self, spelling: &str) -> Option<&'static str>;
    pub fn user_symbol(&self, spelling: &str) -> Option<&UserSymbolCandidate>;
    pub fn longest_user_symbol_at(&self, input: &str, start: usize) -> Vec<UserSymbolCandidate>;
}
```

## Data Model

`ExportedSymbolShape` は full semantic IR ではなく、lexical shape だけを保持します。

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

`UserSymbolKind` は visible symbol の parser/resolver category を記録します。category は functor、predicate、mode、attribute、structure、selector、constructor です。`UserSymbolArity` は argument count shape を exact count、bounded range、lower-bounded range として記録します。これらは parser/resolver-facing summary であり、full type signature ではありません。

Active environment は以下を扱います。

- identifier-shaped symbols;
- punctuation-shaped symbols;
- symbols containing `.`;
- import conflict detection for equal-spelling imported candidates;
- diagnostics 用の安定した provenance.
- downstream parser / resolver phases 用の symbol kind と arity metadata.

`ModuleLexicalSummary` は producer 側で canonicalize された artifact です。summary を作る component は、lexer environment builder に渡す前に `exported_symbols` を deterministic order に正規化しておく必要があります。canonical order は、少なくとも以下の lexical identity と provenance に基づきます。

1. `spelling`
2. `source_module`
3. `symbol_id`
4. `kind`
5. `arity`
6. `export_rank`

`build_lexical_environment` はこの contract を前提にしており、summary 内部を並べ替えません。これにより environment fingerprint は、environment builder がその場で選んだ順序ではなく、imported module の canonical lexical summary に反応します。

この producer-side summary order は、`UserSymbolIndex` 内部で使う active-candidate order とは独立しています。summary が import された後、same-spelling candidates は lookup と diagnostics の安定性のため、import ordinal、export rank、kind、arity、source module、symbol id の順で sort されます。

## Algorithm

現在の実装は、すでに resolve 済みの import から deterministic lookup object を構築します。

1. `ModuleLexicalSummary` を `ModuleId` で index します。同じ module id の summary が複数渡された場合、Rust value として完全に同一なら受け入れます。内容が異なる duplicate summary は construction error です。
2. stable FNV-style fingerprint に version string と built-in reserved word / reserved symbol tables を宣言順で書き込みます。
3. `ResolvedImport` を import-prelude order で走査します。各 import について対応する lexical summary を必須とし、import ordinal、module id、summary fingerprint を active environment fingerprint に加えます。
4. summary 内の exported symbol shape は、index する前に spelling と arity を検証します。spelling は user-symbol spelling でなければならず、reserved word と衝突してはいけません。reserved special symbol との完全一致も原則として禁止しますが、仕様上の例外である `.` だけは許可します。arity shape は maximum が minimum より小さくてはいけません。
5. exported shape を `UserSymbolCandidate` に変換します。このとき、symbol を定義・export した `source_module` と、現在の file が import した `imported_module` の両方に加えて、symbol kind と arity metadata を保持します。前者は provenance、後者は conflict diagnostics に効きます。
6. candidate を `UserSymbolIndex` に挿入します。異なる import から同じ spelling が来た場合は `UserSymbolImportConflict` として拒否します。同じ import 内の同じ spelling は overload candidate として保持でき、上記の active-candidate order で安定化します。
7. borrowed reserved tables、完成した user-symbol index、deterministic fingerprint を持つ `ActiveLexicalEnvironment` を返します。

`UserSymbolIndex` は exact-spelling lookup、deterministic ordering、conflict diagnostics のために canonical な `BTreeMap<String, Vec<UserSymbolCandidate>>` を保持します。加えて同じ spelling 群を ASCII byte trie として保持し、longest-prefix lookup を高速化します。`longest_user_symbol_at` は指定 byte offset から trie を歩き、最も深い terminal node を記録し、その spelling の visible import ordinal に属する candidate を返します。candidate discovery は imported symbol 全体数ではなく、scan した spelling 長と返却 candidate 数に比例します。public lookup semantics は従来と同じです。

実装上の補足:

- `ModuleId` と `SymbolId` は `mizar-lexer` 内の lightweight string newtype です。それ自体は module の存在や semantic resolution を意味しません。
- `ModuleLexicalSummary.exported_symbols` は producer 側で canonicalize 済みであることを前提にします。sorting と summary fingerprint の安定性は summary construction 側の責務であり、environment construction 側の責務ではありません。
- `UserSymbolCandidate.source_module` は lexical summary 由来の defining/exporting provenance を保持し、`imported_module` は conflict diagnostics のために current file の resolved import で指定された module を記録する。
- `UserSymbolCandidate.kind` と `UserSymbolCandidate.arity` は active candidate ごとに保持します。これにより後続 parser / resolver phases は module summary を作り直さずに same-spelling overload を filter / distinguish できます。
- `.` は reserved-special-symbol collision rule に対する仕様上の例外です。それ以外の reserved symbol spelling との完全一致は拒否します。
- 異なる import から来た equal-spelling user symbol は environment construction conflict として拒否します。
- fingerprint には process-randomized hashing ではなく、内部の stable byte hasher を使い、symbol kind と arity metadata も含めます。
- trie は内部の acceleration structure であり、fingerprinting や summary canonicalization には影響しません。

## Non-Goals

この module は以下を行いません。

- source text を parse する;
- import syntax を resolve する;
- full module IR を load する;
- local scope overrides を decide する;
- symbol use が type-correct か decide する;
- overload winner を choose する.

## Error Handling

Error は environment construction failure であり、tokenization failure ではありません。

- missing module lexical summary for a resolved import;
- inconsistent duplicate summary for the same module id;
- exported symbol collides illegally with a reserved word or reserved special symbol;
- different imports が export する equal-spelling user symbols は conflict する;
- invalid user-symbol spelling.
- invalid user-symbol arity shape.

同じ imported module 内の same-spelling user symbol は deterministic candidate として表現できます。一方、異なる import から来た same-spelling symbol は conflict として拒否します。Import order と summary order は error として診断しませんが、deterministic input contract の一部であり、environment fingerprint に反映されます。

## Tests

テストでは以下を確認します。

- reserved table が常に存在すること;
- imported symbol が可視になること;
- 異なる import から来た equal-spelling user symbol が deterministic に拒否されること;
- reserved collision が拒否されること;
- deterministic input order の下で environment fingerprint が安定すること;
- identifier-shaped / punctuation-shaped symbol に対する longest-match query に答えられること。
- 多数の imported symbols と overlapping spelling があっても trie-backed lookup が longest-match behavior を保つこと。
- same-spelling overload candidates の kind / arity metadata が保持されること。
- kind または arity metadata が変わると environment fingerprint が変わること。
