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
- 試みた label spelling と scope family を持つ `Unresolved(UnresolvedLabelRef)`

`LabelRef` は正規化された label-origin path と use-site range を記録する。
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
