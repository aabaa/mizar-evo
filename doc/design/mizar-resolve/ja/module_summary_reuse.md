# ModuleSummary Reuse

> Canonical language: English. Canonical document:
> [../en/module_summary_reuse.md](../en/module_summary_reuse.md).

## 目的

Task R-024 は `mizar-artifact` が publish した dependency-facing
`ModuleSummary` artifact を消費し、resolver の import/name/lexical consumer が
dependency source file を読み直さずに exported dependency projection を利用できるように
する。

この文書は architecture 03 の "Module Summary" と architecture 18 の dependency
fingerprint の resolver 側を refine する。canonical artifact schema、writer、reader、
hash framing、compatibility policy は引き続き `mizar-artifact` が所有する。

## 範囲

`mizar-resolve` が所有する:

- canonical な `mizar-build::module_index::ModuleId` による dependency summary の要求。
- `mizar-artifact` reader API による、供給された canonical summary の検証。
- 検証済み exported symbol、label、lexical、re-export、dependency-interface projection
  から resolver-owned `SymbolEnv` summary contribution index への変換。
- summary がない、互換でない、不一致、または invalid な場合の deterministic な
  crate-local fallback record。
- test における source-backed projection と summary-backed projection の比較。

`mizar-resolve` が所有しない:

- artifact JSON syntax、schema version policy、reader/writer implementation、hash framing。
- store または manifest I/O。
- dependency-summary-backed module の source loading。
- proof acceptance、type checking、overload winner selection、cache reuse、semver classification。
- public resolver diagnostic code。

## 入力契約

resolver は独立した 2 つの入力を受け取る:

1. `mizar_resolve::module_index::ModuleIndexInput` からの build-owned module index provider。
2. artifact boundary で読まれた canonical
   `mizar_artifact::module_summary::ModuleSummary` value または canonical JSON value。

resolver は provider に `DependencyModuleSummaryRef` を問い合わせ、module が
summary-backed かを判定し、fallback diagnostic に artifact path を保持してよい。
ただし、後続の build/artifact contract が明示するまでは
`DependencyModuleSummaryRef::content_hash` を interface hash と解釈してはならない。
optional expected interface hash は artifact-owned reader への validation input である。
module-index provider は artifact の完全な `ModuleSummaryIdentity` lockfile field を公開しないため、
resolver は read 後に既知の identity field だけを検証する: package id、package version、
module path、language edition である。`lockfile_identity` を持つ summary は、これらの既知
field が一致する場合に受理する。

## Summary-backed projection

検証済み summary は 1 つの `ContributionKind::Summary` record と、canonical dependency
module で key される 1 つの `ModuleSummaryIndex` entry を供給する。

exported symbol は resolver-owned `SymbolEntry` record に lowered される:

- `SymbolId.module` は canonical dependency `ModuleId`。
- `SymbolId.local` は stable exported origin id から導出する。
- `SymbolId.fqn` は summary の fully qualified name。
- 既知の public serialized visibility は public/exported にする。既知の private serialized
  visibility は private/local-only に留める。未知の visibility value は access を広げず、
  deterministic fallback record を生成する。
- declaration kind、rendered signature、interface fingerprint は resolver signature shell 内の
  opaque string として保持する。

exported label は、visibility が既知の public または private value の場合に限り `LabelIndex`
entry に lowered される。既知の public label は public export status を受け取り、既知の
private label は local-only に留まる。未知の visibility value は deterministic fallback record を
生成する。未知の serialized label target kind も、artifact-only string に proof または label
semantics を捏造しないよう fallback record を生成する。

lexical contribution は exported summary symbol と対応付けられる場合に限り
`ModuleLexicalSummaryIndex` entry に lowered する。対応付けられない lexical contribution は
deterministic fallback record として表現し、resolver が symbol identity を捏造しないようにする。

re-export と dependency-interface reference は generated anchor を持つ declaration dependency
edge として保存する。それらは dependency-facing fact に限られ、export legality、proof status、
cache reuse を検証しない。

## Fallback と diagnostic

summary reuse は fail-closed である。resolver は次の場合に source-backed resolution へ
fallback する。ただし source fallback は、caller が同じ canonical module について
source-backed resolver input を供給している場合に限る。workspace または in-memory source
representation を持たない dependency-summary-backed module では、resolver は
unavailable/incompatible summary result を記録し、source loading は build/session/driver owner
に残す。

fallback が必要になる条件:

- provider がその module の dependency summary を持たない。
- artifact reader が schema version、canonical order、hash、shape を拒否する。
- resolver が知っている module identity field が一致しない。
- caller-supplied expected interface hash が一致しない。
- summary を resolver-owned projection へ map するには必要な identity を捏造する必要がある。

fallback record は crate-local/internal で deterministic である。canonical module、可能なら
artifact path、fallback reason、stable detail text を含む。public diagnostic code は割り当てない。

## Source-backed agreement

shared fixture で、summary-backed と source-backed resolution は exported resolver-facing fact が
一致すれば agreement とみなす:

- exported symbol の fully qualified name、kind、visibility/export status、opaque rendered signature。
- exported label の name、owner path、visibility/export status、target kind payload。
- exported symbol と対応付けられた exported lexical contribution key。
- dependency module identity と interface hash reference。

比較では source range、source id、local private symbol、proof body、algorithm body、
diagnostic wording、internal artifact bytes は意図的に無視する。

## Determinism

summary-backed projection order は `mizar-artifact` がすでに強制している canonical ordering に従う。
resolver index は既存の stable key で引き続き sort するため、byte-equivalent summary を繰り返し
消費すると byte-identical な `SymbolEnv` snapshot と fallback record が生成されなければならない。

## Public enum forward-compatibility

Task R-024 はこの module に resolver public-enum decision procedure を適用する。
`ModuleSummaryReuseReason` は public な crate-local/internal diagnostic surface であり、
`#[non_exhaustive]` のまま維持しなければならない。downstream consumer は、将来の
summary-unavailable、artifact-rejected、unsupported-projection class に対して wildcard または
fallback handling を含めなければならない。

- `ModuleSummaryReuseReason`
