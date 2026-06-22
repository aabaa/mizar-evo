# mizar-artifact ModuleSummary Schema

> 正本は英語です。英語版: [../en/module_summary.md](../en/module_summary.md)。

## 目的

`ModuleSummary` は resolver と incremental-build consumer のための dependency-facing
artifact projection である。下流 module はこれにより、依存 source file や compiler-internal
IR を再読込せずに、exported name、label、lexical contribution、re-export、
interface fingerprint を読み込める。

この文書は architecture 03「Module Summary」と architecture 18 の interface
fingerprint 要件を精緻化する。[store.md](./store.md) の正準 store 規則に依存する。

## 範囲

`module_summary` schema が所有するもの:

- module summary の stable module identity と schema version field。
- importer から visible な exported symbol、label、lexical、re-export projection。
- dependency-facing reuse key かつ dependency fingerprint input である `interface_hash`。
- summary collection すべての canonical ordering rule。
- summary artifact の compatibility と reader validation 要件。

この schema が所有しないもの:

- resolver algorithm、import legality decision、name lookup execution。
- type inference、overload selection、proof search、proof acceptance、kernel validation。
- proof body、algorithm body、expression metadata、diagnostics、raw `SymbolEnv`、
  raw `ResolvedAst`、cache record。
- store canonical rule の利用を超える manifest transaction や artifact-store I/O。

## Top-Level Shape

schema family は `mizar-artifact/module-summary` である。version `1.0` を task 5 の
初期 version とする。

概念上の形:

```rust
struct ModuleSummary {
    schema_version: String,
    module: ModuleSummaryIdentity,
    source_hash: Hash,
    interface_hash: Hash,
    exported_symbols: Vec<ExportedSymbolSummary>,
    exported_labels: Vec<ExportedLabelSummary>,
    lexical_summary: ModuleLexicalSummary,
    reexports: Vec<ModuleReexportSummary>,
    dependency_interfaces: Vec<DependencyInterfaceRef>,
}
```

`source_hash` は、summary を生成した正確な source file content を記録し、reader が stale
artifact を診断できるようにする。これは `interface_hash` には含めない。body-only、
proof-body-only、diagnostic-only、comment-only の source change は、exported interface が
変わらない限り importer を invalidation してはならないためである。

## Module Identity

`ModuleSummaryIdentity` は、下流 tool から visible な stable package/module identity を含む。

- package id。
- 利用可能な場合は package version または lockfile identity。
- module path。
- language edition。

normalized source path と `source_hash` は stale-artifact diagnostic と manifest consistency check の
ための source metadata である。これらは module identity ではなく、schema-specific rule が source
path を importer-visible としない限り `interface_hash` にも含めない。local alias や filesystem
traversal detail は module identity ではない。manifest entry または requested import path が異なる
package/module を示す場合、reader は summary を拒否する。

## Exported Symbols

`exported_symbols` は dependency-facing declaration だけを含む。各 entry は次を記録する。

- build 間で同じ exported surface element を対応付ける stable origin id。
- fully-qualified exported name。
- exported namespace path と visibility。
- definition、theorem、mode、predicate、functor、attribute、struct、
  registration-facing declaration、notation、algorithm signature などの declaration kind。
- diagnostic と navigation に使える source range。
- importer が必要とする rendered surface signature または statement。
- その exported element の canonical interface fingerprint。
- importer visibility または interface validity がその status に依存する場合のみ、proof acceptance status。

summary は implementation body を含まない。

- theorem proof body。
- algorithm body。
- export も re-export もされない local definition と private declaration。
- expression metadata。
- ATP log、proof witness payload、kernel-internal proof state。

proof acceptance status は proof-producing phase が供給する projected data である。
`ModuleSummary` はその status を記録するだけで、proof validation や proof acceptance decision を行わない。

## Exported Labels

`exported_labels` は、下流 module が引用できる label を記録する。各 entry は次を記録する。

- stable label origin id。
- label text。
- fully-qualified owner item。
- exported visibility。
- source range。
- theorem、definition、scheme、registration、または後続 schema が export する proof obligation
  label などの target kind。

private または module-local な proof-step label は除外する。ambiguous または duplicate な label を
この schema が正規化して隠すことはない。resolver diagnostic は `mizar-resolve` が所有する。

## Lexical Summary

`lexical_summary` は、importer の candidate active lexical environment を構築するために必要な
exported lexical contribution だけを含む。

- exported notation declaration と parse effect。
- exported reserved/user symbol contribution。
- lexical disambiguation に必要な exported vocabulary または namespace prefix。
- token classification に影響する lexical schema version または fingerprint field。

lexical summary は proof authority ではなく、import が legal かどうかも決めない。active lexical
environment construction は candidate summary を用いてよいが、semantic import resolution が後で
import を検証する。

## Re-exports And Dependencies

`reexports` は stable module identity により、item-level re-export が supported された場合は
exported item identity により、exported forwarding relationship を記録する。これにより provenance を
保持し、importer は original module と stable origin に対して diagnostic を報告できる。

`dependency_interfaces` は、この summary に影響した依存 `ModuleSummary` の interface hash を記録する。
dependency data の欠落を「依存なし」と解釈してはならない。不完全な dependency interface 情報を持つ
summary は、`mizar-cache` が所有する reuse decision において uncacheable になる。

## Interface Hash

`interface_hash` は `ModuleSummary` 内の importer-visible projection に対する canonical
dependency-facing key である。summary file の byte identity ではない。公開 file byte は manifest
path と store-level の `artifact_hash` が識別・検証する。

`interface_hash` は task 3 の `HashClass::Interface` domain により、canonical interface projection
から計算する。

hash に含めるもの:

- schema family と schema version。
- importer の解釈に影響する module identity field。
- language edition。
- exported symbol、label、lexical contribution、re-export。
- importer から visible な exported theorem statement と accepted proof status。
- exported algorithm signature と `requires` / `ensures` contract。
- summary 内に存在する dependency-facing notation、overload summary、coherent-refinement metadata。

hash から除外するもの:

- file 全体の `source_hash`。
- syntax-sensitive notation 以外の comment と formatting。
- proof body と algorithm body。
- local diagnostic と diagnostic wording。
- timestamp、local absolute path、worker id、backend timing、その他の hash-excluded local metadata。

`source_hash` byte が異なっていても、exported interface projection が同一である 2 つの summary は同じ
`interface_hash` を持つ。一方で、それぞれの manifest entry または store-level `artifact_hash` は
異なってよい。

## Canonical Ordering

すべての collection は決定的に serialize する。

- exported symbol は fully-qualified name、stable origin id、source range で sort する。
- exported label は label text、owner fully-qualified name、stable origin id、source range で sort する。
- lexical contribution は contribution kind と canonical lexical key で sort する。
- re-export は target module identity と item identity で sort する。
- dependency interface reference は package id、module path、interface hash で sort する。

insertion order、source traversal order、filesystem order、worker completion order が serialized byte や
hash に影響してはならない。

## Reader And Writer Requirements

writer は `store.md` の正準 UTF-8 JSON rule を使い、current schema version を emit する。reader は:

- summary field を解釈する前に schema-version compatibility を検査する。
- manifest entry、requested module、summary module identity が一致することを検証する。
- consuming command または manifest entry が要求する場合、`interface_hash` を検証する。
- raw-IR-shaped payload と unknown cache-record encoding を拒否する。
- この schema または後続互換 schema が定義する stable projected status field なしに accepted proof
  status を主張する summary を拒否する。

reader failure は artifact diagnostic である。proof authority を確立せず、internal cache record へ
黙って fallback してはならない。

## Deferred Implementation

task 4 はこの仕様だけを追加する。source implementation は task 5 に deferred とする。task 5 は
`ModuleSummary` schema、writer、validating reader、および round-trip、body-only change に対する
interface-hash stability、incompatible-version read の test を追加する。
