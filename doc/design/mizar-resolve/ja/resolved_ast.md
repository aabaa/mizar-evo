# mizar-resolve: ResolvedAst

> 正本は英語です。英語版: [../en/resolved_ast.md](../en/resolved_ast.md)。

## 目的

`ResolvedAst` は、1 つのモジュールに対する resolver 所有の、ソース形状を保つ
意味論的 snapshot である。後続の診断や editor 機能が必要とする `SurfaceAst`
の形を保ちながら、後続 phase が名前解決をやり直さずに消費できる安定した
module、name、label、import、origin 情報を付与する。

この文書は以下を精緻化する。

- [architecture 01](../../architecture/ja/01.ir_layers.md) の `ResolvedAst`
- [architecture 03](../../architecture/ja/03.module_and_symbol_resolution.md) の
  interface definition と recoverability policy

## 境界

`ResolvedAst` が所有するもの:

- 解決対象 source unit の canonical module identity
- semantic import/export resolution result
- namespace と symbol reference の解決結果
- label reference の解決結果
- 未解決・曖昧参照の明示表現
- 回復可能な構文に対する recovered semantic shell
- 正規化された semantic origin と provenance

`ResolvedAst` が所有しないもの:

- parsing、parser recovery、syntax vocabulary の変更
- type inference、selector type checking、overload winner selection
- cluster firing、coercion insertion、registration activation
- proof obligation generation や proof validity
- artifact schema emission

既知の gap: resolver 専用の public diagnostic code range は、外部診断仕様にまだ
予約されていない。`ResolvedAst` は structured diagnostic anchor や crate-local
diagnostic handle を保持してよいが、この仕様は user-facing な public resolver
code を創作しない。

## トップレベル形状

トップレベル形状は次のとおりである。

```rust
struct ResolvedAst {
    source_id: SourceId,
    module_id: ModuleId,
    nodes: ResolvedArena,
    name_refs: NameRefTable,
    label_refs: LabelRefTable,
    imports: ResolvedImports,
}
```

`source_id` は source-map lookup のための session-owned source identity である。
`module_id` は resolver の module-index 入力から得た canonical module identity
である。`nodes` はソース形状を持つ resolved node の arena である。reference
table は構造的な node arena と分離して意味論的判断を記録し、後続 phase が
source shape と reference outcome のどちらも直接 inspect できるようにする。

## 安定 identity

### `ModuleId`

`ModuleId` は canonical かつ alias-independent である。package identity と
正規化された module path から成る。local import alias、relative import spelling、
source file spelling は `ModuleId` の一部ではない。

`ModuleId` は `SourceId`、絶対 host path、session-local allocation counter、
display-only alias を含んではならない。

### `SymbolId`

`SymbolId` は安定しており、完全修飾される。構成要素は次のとおりである。

- 宣言元の `ModuleId`
- declaration kind と module 内の決定的な declaration position から導かれる
  resolver-owned local symbol identity
- artifact、deterministic debug rendering、candidate ordering に使う fully
  qualified name projection

複数の declaration が同じ surface spelling を共有する場合、local symbol identity
は `symbols.md` が指定する deterministic overload slot、relation ordinal、または
declaration ordinal を含まなければならない。hash iteration order、memory
address、`SourceId`、local import alias に依存してはならない。

`SymbolId` は、resolver が semantic declaration として表現できる declaration に
のみ割り当てる。未解決または曖昧な参照に、作り物の `SymbolId` を与えてはならない。

## node arena

`ResolvedArena` は安定した `ResolvedNodeId` を持つ `ResolvedNode` を格納する。

必須 node data:

- 元の `SurfaceAst` 形状に対応する source-shaped node kind
- source range、または generated/recovered anchor
- source order の 0 個以上の child `ResolvedNodeId`
- `RecoveryState` flag
- node 自身が resolver outcome を持つ場合に、resolved、unresolved、ambiguous、
  deferred、not-applicable node を区別する `NodeResolutionState`
- node-local な reference/import outcome に対する `NameRefTable`、
  `LabelRefTable`、`ResolvedImports` への安定 key
- 正規化された `SemanticOrigin`
- resolver-owned fact のための任意の node-local payload

arena invariant:

- すべての child id は同じ arena で確保された node を指す。
- root node は `module_id` に属する。
- parent/child edge は acyclic である。
- child order は決定的で、source-shaped である。
- 同等の入力を再解決すると同じ id と ordering が得られる。
- parser が recoverable surface node を生成した場合、未知または未対応の recovered
  syntax は黙って捨てず、recovered shell として表現する。

`NodeResolutionState` は traversal semantics のために必須である。後続 phase が
arena を walk する時、reference table を調べる前でも node が degraded であることを
観測できなければならない。詳細な candidate と failure class は、canonical な格納
場所を 1 つに保つため table 側に残す。

arena は inferred expression type、checker fact、final overload result、proof
obligation を格納してはならない。

## name reference table

`NameRefTable` は、resolver が解決を試みた name-use site を `NameResolution`
result に対応付ける。name-use site は node 全体、node 内 token、または
resolver-created reference anchor であってよいが、その key は `ResolvedAst` 内で
安定していなければならない。

必須 result variant:

- 解決済み declaration を表す `Resolved(SymbolRef)`
- 通常の source declaration ではない built-in identity を表す
  `ResolvedBuiltin(BuiltinRef)`
- dotted syntax が term base を持ち、残りの selector 判断に型情報が必要な場合の
  `DeferredSelector(DeferredSelectorRef)`
- 決定的な candidate list を持つ `Ambiguous(AmbiguousNameRef)`
- 試みた spelling と失敗した lookup class を持つ `Unresolved(UnresolvedNameRef)`

`SymbolRef` は target `SymbolId`、use-site range、任意の import/provenance 情報を
記録する。診断用に use site の local spelling を含めてよいが、identity は
`SymbolId` である。

曖昧 candidate list は canonical fully qualified name、module id、source range の
順に sort する。未解決または曖昧な root は明示的でなければならない。これにより、
後続 phase は作り物の semantic identity を cascade させず、依存 node を skip または
degrade できる。

## label reference table

label は ordinary symbol とは別 scope なので、`LabelRefTable` は `NameRefTable`
から分離する。

必須 result variant:

- theorem、definition、proof-step、registration label の解決を表す
  `Resolved(LabelRef)`
- 決定的な candidate を持つ `Ambiguous(AmbiguousLabelRef)`
- 試みた label spelling と expected scope family を持つ
  `Unresolved(UnresolvedLabelRef)`。expected family は具体的な label kind、または
  `by` reference 用の proof-step-or-theorem のような mixed citation family でよい。

`LabelRef` は canonical serialization の label-origin path と use-site range を
記録する。canonical serialization は identifier spelling を normalize せず、
exact parser token byte を保持する。
label-origin path は後続の `ObligationAnchor` 構築に十分な安定性を持たなければ
ならないが、resolver が obligation を生成することを意味しない。

詳細な label scope rule は `labels.md` で指定する。この文書は storage shape と
invariant だけを定義する。

## resolved imports

`ResolvedImports` は module import/export directive に対する resolver outcome を
格納する。

必須内容:

- source order のすべての import directive
- source order のすべての export directive
- 各 import/export directive outcome を所有する `ResolvedNodeId`
- 解決済み import/export の canonical module target
- 存在する場合の local alias spelling
- source spelling、range、failure class を持つ unresolved import/export entry
- import が candidate を可視にした場合、name/label reference からその import edge
  へ戻る provenance link

canonical dependency projection は決定的な `ModuleId` order で公開してよいが、
診断のために source-order record を残さなければならない。未解決 import は明示的に
表現し、module の残りの解決を中断しない。
node-local な import/export key は、同じ arena node を owner とする entry を
指さなければならない。

alias、relative-prefix、cycle rule の詳細は `imports.md` で指定する。この文書は
storage shape と recoverability requirement だけを定義する。

## recovered shell

parser が recoverable subtree を mark した場合、subtree が item、reference、label、
import、export の位置を識別するだけの source shape をまだ持つなら、resolver は
semantic shell を保持するべきである。

recovered-shell rule:

- 対応する node または table entry を recovered として mark する。
- source range と parser recovery anchor を保持する。
- 未解決または曖昧な参照を明示的に記録する。
- identity を決定的に表現できない declaration には `SymbolId` を割り当てない。
- 後続 child が malformed であることだけを理由に recoverable shell を落とさない。
- parser diagnostic を隠したり、syntax recovery を semantic validity に変換したりしない。

後続 phase は recovered shell を degraded input として扱い、それに依存する fact を
skip してよい。

## semantic origin と provenance

すべての resolved node、reference result、import/export result、declaration shell は、
diagnostics、navigation、incremental invalidation、downstream anchor construction に
十分な正規化 provenance を持たなければならない。

必須 origin field:

- source-map lookup のための `source_id`
- canonical module ownership のための `module_id`
- source range、または generated/recovered anchor
- source-shaped structural path、または module 内の deterministic ordinal
- import 経由で導入された fact の場合は任意の import edge id
- recovered syntax 由来の場合は recovery marker

origin は absolute path、memory address、hash-map iteration order、local import alias
に依存してはならない。source range は diagnostics と navigation のためのものである。
canonical identity は必要に応じて `ModuleId`、`SymbolId`、label origin path、または
deterministic structural ordinal から得る。

後続の `ObligationAnchor` construction はこれらの origin field を消費してよいが、
`ResolvedAst` は obligation を作成しない。

## 決定性

すべての id、table iteration、ambiguous candidate ordering、unresolved entry
ordering、debug rendering input は、同等の source、module-index input、dependency
summary に対して、実行間・platform 間で決定的でなければならない。

実装は raw `HashMap` や `HashSet` の iteration order を public rendering、
snapshot、diagnostic、serialized projection に露出してはならない。

resolver snapshot baseline 用の human-readable debug rendering は versioned debug
format であり、published artifact schema ではない。LF line ending、
locale-independent decimal formatting、deterministic string escaping、および
不安定な実装 `Debug` output ではなく手書き variant name を使う。

## 公開 enum の前方互換性

task R-026 は frontend task 25 の public-enum decision procedure をこの module に適用する。
`resolved_ast` が所有する公開 resolver enum はすべて forward-compatible API surface であり、
`#[non_exhaustive]` を維持しなければならない:

- `RecoveryState`
- `NodeResolutionState`
- `NodeReferenceKey`
- `ResolvedArenaError`
- `SurfaceResolvedArenaError`
- `NameLookupClass`
- `NameResolution`
- `LabelKind`
- `LabelExpectation`
- `LabelResolution`
- `ImportResolution`
- `ImportFailureClass`
- `ExportFailureClass`
- `ExportTarget`
- `ResolvedAstError`

この module は exhaustive な公開 enum 例外を所有しない。下流 consumer は wildcard
または fallback arm を持たなければならない。resolver 内部の match は、仕様化済みの
挙動を実装する範囲で、現在表現されている variant に対して exhaustive でよい。

## 計画中の data-shape test

Task R-004 は以下の focused unit test を追加しなければならない。

- `ModuleId`、`SymbolId`、`ResolvedNodeId` allocation の決定性
- arena child-id validation と cycle rejection
- resolved、unresolved、ambiguous、builtin、deferred-selector result に対する
  `NameRefTable` round-trip
- resolved、unresolved、ambiguous result に対する `LabelRefTable` round-trip
- `ResolvedImports` の source-order record と canonical target projection
- arena traversal 中の unresolved、ambiguous、deferred、recovered node に対する
  `NodeResolutionState` preservation
- 同等入力の再解決でも安定する node-to-table key と node-to-import key
- recovered-shell flag と origin preservation
- candidate list と table iteration の deterministic ordering

## R-032A frozen Surface structural-arena contract

R-032A は Checker Task 258B5C の最初の implementation prerequisite である。
parser-owned `SurfaceAst` node と resolver-owned structural node の間に欠けている
validated production mapping を修復する。canonical resolver responsibility、本書と
architecture の resolved-AST contract、既存 `SurfaceAst` / `ResolvedArena` API
は十分な authority である。以前の checker 作業で観測した private structural clone
は inventory evidence にすぎず、authority ではない。

exact API は次である。

```rust
SurfaceResolvedArena::lower(
    ast: &SurfaceAst,
    module: &ModuleId,
) -> Result<SurfaceResolvedArena, SurfaceResolvedArenaError>

source_id(&self) -> SourceId
module(&self) -> &ModuleId
arena(&self) -> &ResolvedArena
resolved_node_for(&self, source: SurfaceNodeId) -> Option<ResolvedNodeId>
validate_against(
    &self,
    ast: &SurfaceAst,
    module: &ModuleId,
) -> Result<(), SurfaceResolvedArenaError>
```

`SurfaceResolvedArena` は complete one-to-one、child-first、same-index の
structural arena を所有する。各 surface node について exact kind、ordered
children、range、recovery state、root、source、canonical module を保持する。
structural node の `SemanticOrigin` は exact surface range を anchor とし、
`structural_path = [surface_node_id.index()]`、semantic key なし、
`NodeResolutionState::NotApplicable`、`reference_key = None` とする。この arena
は structural provenance であり、semantic resolution を主張しない。

`SurfaceResolvedArena` は `Debug`, `Clone`, `PartialEq`, `Eq` を derive し、
`Copy` は要求しない。`SurfaceResolvedArenaError` は `Debug` を derive して
`Display` / `std::error::Error` を実装し、`Clone` / `Eq` / `Copy` は要求しない。

exact public declaration は次である。

```rust
#[derive(Debug)]
#[non_exhaustive]
pub enum SurfaceResolvedArenaError {
    MissingRoot,
    StructuralPathComponentOverflow { node: SurfaceNodeId },
    InvalidChildOrder {
        node: SurfaceNodeId,
        child: SurfaceNodeId,
    },
    InvalidArena(ResolvedArenaError),
    SourceMismatch,
    ModuleMismatch,
    NodeCountMismatch,
    RootMismatch,
    NodeKindMismatch { node: SurfaceNodeId },
    ChildListMismatch { node: SurfaceNodeId },
    RangeMismatch { node: SurfaceNodeId },
    RecoveryMismatch { node: SurfaceNodeId },
    ResolutionStateMismatch { node: SurfaceNodeId },
    ReferenceKeyMismatch { node: SurfaceNodeId },
    OriginMismatch { node: SurfaceNodeId },
    StructuralPathMismatch { node: SurfaceNodeId },
}
```

downstream match は wildcard arm を持つ。structural node state が exact
`NodeResolutionState::NotApplicable` でない場合は
`ResolutionStateMismatch`、reference key が non-`None` の場合は
`ReferenceKeyMismatch`。surface `usize` index から公開 path `u32` component
への変換は checked で、unwrap/saturation/truncation/panic を禁止する。test は
wrong source/module/arena、missing/stale node、shape/recovery、state/key injection、
root、overflow を含む全 declared mismatch を個別に拒否する。同等入力は complete
mapping と deterministic id を保持する。

R-032A implementation ownership は exact に
`crates/mizar-resolve/src/resolved_ast.rs`、
`crates/mizar-resolve/src/resolved_ast/tests.rs`、
sole R-026 `SurfaceResolvedArenaError` owning-spec decision entry 用
`crates/mizar-resolve/tests/lint_policy.rs`、同期した resolver design record
だけである。R-032B より前の1 commit とする。label collection、runner、fixture、
sidecar、trace status/count、parser/frontend production、checker/type/proof
semantics、Cargo/workspace metadata は変更しない。

arena node の minimal structural origin `[surface_node_id.index()]` と R-032B
`labels.md` の richer projection/reference table origin は意図的に異なる。
R-032A は前者、R-032B は後者を validate し、相互に代用しない。

### S-026 lower-stage dependency と validation order

documentation 後の fresh preflight で、then-existing syntax consumer API は valid
disconnected `SurfaceAst` node をその `SurfaceNodeId` と共に列挙できなかった。
resolver 内で workaround すれば High `boundary_violation`、existing
API が十分という以前の主張は `design_drift`。したがって R-032A source は、
exact dense `SurfaceAst::node_views()` accessor を追加する別 mizar-syntax
S-026 documentation/implementation task まで未実装だった。その dependency は
実装・検証済みで、dedicated commit と fresh inventory 後に R-032A は
unblocked。R-032A ownership 自体は上記 Rust 3 files のままで、S-026 を
consume し syntax を変更しない。

`SurfaceResolvedArena` は exact に `source_id: SourceId`、
`module: ModuleId`、`arena: ResolvedArena` を store する。complete same-index
mapping は intrinsic で parallel mutable map はない。`lower` は
`ast.node_views()` を dense forward order で consume する。
`resolved_node_for` は source index が arena に存在する場合だけ same-index
resolved id を返す。

validation は次の exact precedence で fail closed する。

1. wrapper source
2. wrapper module
3. contained-arena child/root validity
4. node count
5. root
6. dense node order ごとに kind、ordered children、range/anchor、recovery、
   resolution state、reference key、origin core、structural path

`RangeMismatch` は origin anchor が exact
`SourceAnchor::Range(surface.range)` でない場合。`RecoveryMismatch` は
surface recovery、node `RecoveryState`、origin recovered flag の不一致。
`OriginMismatch` は range/recovery check 後の origin source/module/nonempty
import edge。`StructuralPathMismatch` は最後の exact one-component checked
path 専用。public builder input は child-first で real `u32` overflow は非現実的
なため、`InvalidChildOrder` と `StructuralPathComponentOverflow` は public
behavior を弱めたり unsafe construction を使わず private checked-core helper
から試験する。

R-032A test は independent mutation だけでなく simultaneous fault でこの
precedence を証明する。wrapper source は wrapper module/arena/count/root より
先、wrapper module は arena/count/root より先、invalid contained arena は
count/root より先、count は root より先、root は node-field mismatch より先。
adjacent per-node field を pair にして各 earlier field が next より先であること、
earlier dense node が later node の全 fault より先であることもfreezeする。
private checked-helper test は exact payload
`InvalidChildOrder { node, child }` と
`StructuralPathComponentOverflow { node }` をfreezeする。

### R-032A lint-policy ownership correction

implementation preflight で、frozen public `SurfaceResolvedArenaError` は
existing R-026 public-enum decision guard に必ずscanされることが判明した。
`crates/mizar-resolve/tests/lint_policy.rs` を除外すると、implementation は
mandatory guard failure または frozen ownership boundary 越境のどちらかになる。
これは High `design_drift` であり、`spec_gap`、`test_gap`、semantic decision
ではない。この文書はenumへR-026をすでに要求し、
`source_spec_correspondence.md`もguard ownerを`tests/lint_policy.rs`へ
割り当て済みである。

したがって prerequisite correction は later R-032A implementation scope に
`SurfaceResolvedArenaError` exact owning-spec decision entry だけを追加する。
別lint-policy change、runtime behavior、label collection、fixture、sidecar、
expectation、trace、specification、parser/frontend/checker source、
diagnostic code、Cargo changeはauthorizeしない。この同期documentation
correctionはseparate commitとし、three-Rust-file R-032A implementation commit
の前にfresh inventoryを要求する。

### R-032A implementation result

implementationはfrozen `SurfaceResolvedArena` API、public non-exhaustive
error table、exact three stored fieldsを提供する。全dense
`SurfaceAst::node_views()` entryをchild-first same-indexでlowerし、frozen
top-level/per-node precedenceをvalidateし、semantic state/keyなしのstructural
provenanceだけを保持する。focused evidenceはdisconnected/recovered/
root-not-last node、全named mismatch、invalid contained arena、simultaneous
precedence fault、exact helper payload、independent equivalent-input
determinism、out-of-range foreign id、downstream wildcard compatibilityを
coverする。R-032Bはseparateかつpending。

### R-032B lint-policy ownership dependency（current prerequisite）

fresh R-032B inventoryは、`labels.md`でfreezeしたpublic
`ProofLabelSourceCollectionError`にも同じmandatory R-026 ruleを適用する。
旧two-Rust-file R-032B wordingはdecision-table ownerを欠き、High
`design_drift`であり、semantic `spec_gap`、`test_gap`、test-intent changeでは
ない。later R-032B implementationはexact
`crates/mizar-resolve/src/labels.rs`、
`crates/mizar-resolve/src/labels/tests.rs`、
`crates/mizar-resolve/tests/lint_policy.rs`で、policy fileはsole
`ProofLabelSourceCollectionError` owning-spec decision
`spec_name: "labels.md"`だけを受けられる。このcross-family noteはimplemented
R-032A arenaを再オープンまたは変更しない。

current docs-only correctionはexact 31 design files、resolver 16、checker 8、
`mizar-test` 6、global ledger 1。source、specification、fixture、sidecar、
expectation、trace status/count、Cargo metadata、semantic contract、test intentは
変更しない。`spec_coverage_audit.md`はdeliberate no-op。independent
specification、test/scope、source/documentation consistency
reviewはすべて**NO FINDINGS**で、docs-only verification/count/hash gateはPASS。
independent final read-only qualityも**NO FINDINGS**で、全9 hard gates
PASS、capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）。task-only
staging/cached-diff review、commit、post-commit invariant/fresh-inventory
gateだけがpendingで、その後のfresh inventoryがseparate R-032B
implementationをgateする。
