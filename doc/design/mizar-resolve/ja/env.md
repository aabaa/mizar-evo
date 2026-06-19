# mizar-resolve: SymbolEnv

> 正本は英語です。英語版: [../en/env.md](../en/env.md)。

## 目的

`SymbolEnv` は、ある module から見える resolver-owned の indexed signature
environment である。declaration shell と signature collection が安定した semantic
identity を割り当てた後に構築され、type checking、proof preparation、downstream
module resolution、incremental invalidation が消費する boundary object になる。

この文書は以下を精緻化する。

- [architecture 01](../../architecture/ja/01.ir_layers.md) の `SymbolEnv`
- [architecture 03](../../architecture/ja/03.module_and_symbol_resolution.md) の
  symbol environment と signature collection
- [spec chapter 11](../../../spec/ja/11.symbol_management.md) の scope、visibility、
  import、public/private symbol

## 境界

`SymbolEnv` が所有するもの:

- local、imported、re-exported、visible symbol の index
- declaration definition と opaque resolver-level signature shell
- visible label projection と label contribution provenance
- winner selection 前の overload candidate group
- checker activation 前の resolver-side registration declaration
- namespace、import、export、alias、re-export visibility graph edge
- type checking なしに発見できる declaration dependency edge
- 決定的 invalidation のための source contribution tracking

`SymbolEnv` が所有しないもの:

- parser や syntax recovery behavior
- type inference や expression type fact
- overload winner selection
- selector type checking
- cluster firing や特定 term に対する registration activation
- proof obligation generation、ATP premise selection、proof status
- build planning、source loading、artifact storage

既知の gap: resolver 専用の public diagnostic code range はまだ予約されていない。
`SymbolEnv` は structured diagnostic anchor と crate-local failure class を保持してよいが、
この仕様は user-facing な public resolver code を創作しない。

## トップレベル形状

トップレベル形状は次のとおりである。

```rust
struct SymbolEnv {
    module_id: ModuleId,
    imports: ResolvedImportIndex,
    exports: ResolvedExportIndex,
    symbols: SymbolIndex,
    labels: LabelIndex,
    definitions: DefinitionIndex,
    overloads: OverloadIndex,
    registrations: RegistrationIndex,
    namespace_graph: NamespaceGraph,
    declaration_dependencies: DeclarationDependencyIndex,
    contributions: SourceContributionIndex,
    module_summaries: ModuleSummaryIndex,
}
```

`module_id` は、この environment が表す canonical module である。`imports` と
`exports` は `ResolvedAst` 由来の resolver-owned projection である。index family は
決定的で query-oriented であり、後続 phase は raw map iteration order を観測せずに
`SymbolId`、fully qualified name、visible spelling、namespace、source contribution
から lookup できなければならない。

`module_summaries` は resolver/checker boundary で利用できる in-memory dependency
summary index である。artifact-backed summary reuse は task R-024 が仕様化するため、
この data shape で創作してはならない。

この文書の `RegistrationIndex` は、checker validation と obligation acceptance より前の
registration に対する resolver-side declaration index である。
[architecture 04](../../architecture/ja/04.type_and_registration_resolution.md) が説明する
checker-side active registration index とは別物である。

## SymbolIndex

`SymbolIndex` は environment が知るすべての symbol identity を格納する。

- current module の local declaration
- resolved import を通じて見える imported public symbol
- facade module を通じて見える legal re-export
- edition が有効にする built-in prelude symbol（resolver-visible candidate として
  表現される場合）

必須 lookup projection:

- `SymbolId` から symbol entry
- fully qualified name から `SymbolId`
- visible spelling と namespace から deterministic candidate list
- defining `ModuleId` から exported entry
- source contribution id から contributed entry

各 symbol entry は次を記録する。

- `SymbolId`
- kind
- visibility と export status
- primary spelling と任意の notation spelling
- defining origin と contribution id
- signature-collection task が埋める任意の opaque signature shell
- synonym、antonym、redefinition に対する relation metadata（resolver-owned
  declaration fact の場合）

`SymbolIndex` は inferred expression type、selected overload winner、activated
registration fact、proof validity を格納してはならない。

## LabelIndex

`LabelIndex` は label declaration と visible label projection を ordinary symbol とは別に
格納する。

必須 projection:

- current module が宣言した local label
- importer から見える exported theorem/lemma label
- resolved import を通じて見える imported public label
- label origin path から label entry
- visible label spelling と namespace から deterministic candidate list
- source contribution id から contributed label

各 label entry は次を記録する。

- 安定した label identity または origin path
- label kind
- visibility と export status
- primary spelling
- defining origin と contribution id
- label shell が recovered syntax 由来の場合の recovery state

`LabelIndex` は proof validity を解決せず、obligation anchor も生成しない。後続の
proof/VC phase が消費する label provenance を保持する。

## DefinitionIndex

`DefinitionIndex` は `SymbolId` を key として declaration definition を格納する。

記録する resolver-owned declaration fact:

- declaration kind と visibility
- parameter と binder shell id
- syntactic arity と notation shape
- 利用できる場合の source doc/comment attachment id
- 正規化された semantic origin
- declaration collection が生成した duplicate/conflict classification
- declaration collection 中に発見した syntactic dependency reference
- `symbols.md` が定義した後の opaque per-kind signature payload

`DefinitionIndex` は declaration-pass index である。signature が malformed または
recovered であることを記録してよいが、type expression が well-typed か、proof
obligation が満たされるかを決めてはならない。

## OverloadIndex

`OverloadIndex` は、resolver-visible spelling または notation slot を共有する
candidate declaration を group 化する。

必須 grouping key:

- namespace または module visibility context
- surface spelling または symbolic notation
- symbol kind family
- type checking なしに利用できる arity または syntactic shape

candidate order は deterministic である。canonical fully qualified name、symbol kind、
source range、declaration ordinal を tie breaker として使う。

`OverloadIndex` は candidate availability と illegal grouping diagnostic を記録する。
winning overload を選ばず、inferred type で candidate を rank せず、coercion を
insert しない。

## RegistrationIndex

`RegistrationIndex` は checker activation 前の registration declaration を格納する。

必須内容:

- registration `SymbolId` または declaration id
- registration kind と syntactic target shell
- 適用可能な場合の visibility/export status
- normalized origin と contribution id
- 構文上言及された declaration への dependency reference
- malformed registration shell の recovery state

この index は checker に deterministic candidate list を公開してよい。registration を
発火させたり、cluster closure を計算したり、特定 term への applicability を決めては
ならない。

## DeclarationDependencyIndex

`DeclarationDependencyIndex` は、type checking なしに発見できる resolver-visible な
declaration dependency edge を記録する。

必須 edge data:

- source endpoint: declaration `SymbolId`、import/export entry id、namespace
  edge id、label origin path、または unresolved reference key
- target `SymbolId`、`ModuleId`、label origin path、または unresolved reference key
- import edge、re-export edge、signature mention、synonym target、antonym target、
  redefinition target、registration mention、label citation などの dependency kind
- source range または recovered anchor
- source contribution id

この index は deterministic invalidation と dependency diagnostic に使う。type-derived
dependency、cluster firing trace、selected overload winner、proof-obligation dependency を
encode してはならない。

import、export、namespace facade の構造自体は `NamespaceGraph` と import/export index が
所有する。`DeclarationDependencyIndex` は、その構造に依存する declaration、label、
projection を説明または invalidate するために必要な dependency edge だけを格納する。

## NamespaceGraph

`NamespaceGraph` は resolver-visible な namespace と module visibility relationship を
model 化する。

必須 node/edge kind:

- `ModuleId` を key とする canonical module node
- import 由来の local alias node
- export と re-export facade edge
- qualified lookup に使う namespace segment edge
- edition-enabled な built-in prelude root edge
- recovery が継続する場合の unresolved または recovered edge

各 edge は次を記録する。

- source range、または generated/recovered anchor
- source contribution id
- visibility
- 解決済みの場合の canonical target identity
- canonical identity と異なる場合の local spelling

この graph は package を discover せず、source を load せず、build-side module index を
構築しない。task R-007 が定義する resolver-side module-index input を消費する。

## source contribution tracking

`SourceContributionIndex` は、どの source unit または dependency summary が各 environment
entry を寄与したかを記録する。

contribution record は次を含まなければならない。

- environment 内で安定する contribution id
- contributing `ModuleId`
- workspace source contribution で利用できる場合の `SourceId`
- summary-backed contribution で利用できる場合の dependency summary identity/hash
- source range、generated anchor、または recovered anchor
- 生成された symbol id、definition id、overload group id、registration id、
  label id、namespace edge、declaration dependency edge、import/export entry、
  diagnostic anchor

canonical symbol identity は `SourceId` に依存してはならない。contribution tracking は
session source map と invalidation input へ戻るためにだけ `SourceId` を使う。

source unit が再 parse された場合、または dependency summary が変化した場合、
contribution index は remove、replace、revalidate する entry を識別する。downstream
query は次を区別できなければならない。

- local current-module contribution
- imported source-backed dependency contribution
- artifact/summary-backed dependency contribution
- built-in/prelude contribution

## invalidation note

`SymbolEnv` invalidation は deterministic で contribution-based である。

current module edit が invalidate するもの:

- その module の local `SymbolEnv`
- changed exported symbol、label、namespace edge、re-export projection を import する
  downstream `ResolvedAst` と `SymbolEnv`
- changed symbol または registration entry を消費する checker/type/proof phase

imported module summary edit が invalidate するもの:

- その summary から導かれた visible symbol と label projection
- changed candidate を含む overload group
- その summary から導かれた label projection と declaration dependency edge
- その summary から導かれた namespace edge と re-export projection
- changed summary を contribution record で参照する downstream module

comment または formatting だけに影響する変更は、recorded origin range、doc/comment
attachment、または consuming cache key の明示的な一部である source hash を変えない限り、
stable id や ordering を変えてはならない。

environment cache key は resolver schema version、`ResolvedAst` identity、
module-index input identity、dependency summary identity、language edition、
contribution fingerprint を含む。正確な cache-key structure は将来の incremental
build/cache layer が所有する。この仕様は、それを計算するために resolver が必要とする
data だけを定義する。

## 決定性

すべての index は deterministic iteration order を公開する。実装は、次を公開する前に
sort するか ordered map を使わなければならない。

- symbol entry
- label entry
- definition entry
- overload candidate
- registration entry
- namespace graph node と edge
- declaration dependency edge
- contribution record
- diagnostic と failure anchor

ordering は raw `HashMap`/`HashSet` iteration order、memory address、filesystem traversal
order、canonical identity が利用できる場合の local import alias spelling に依存しては
ならない。

## 計画中の data-shape test

Task R-005 は以下の focused unit test を追加しなければならない。

- 各 index family の insert/lookup round-trip
- symbol、overload candidate、registration、label、namespace edge、declaration
  dependency edge、contribution record の deterministic ordering
- local source、imported source-backed dependency、summary-backed dependency、
  built-in ごとの per-source contribution tracking
- changed contribution から affected symbol、overload group、label、namespace edge、
  declaration dependency、registration への invalidation lookup
- `SymbolEnv` entry に checker-owned fact が含まれないこと
- 同等入力からの repeated construction に対する stable behavior
