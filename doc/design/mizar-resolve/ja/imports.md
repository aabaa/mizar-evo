# モジュール: imports

> 正本は英語です。英語版: [../en/imports.md](../en/imports.md)。

状態: task R-009 は、すでに解決済みの canonical candidate 上で canonical import graph
construction と cycle rejection を実装する。alias binding、relative-prefix interpretation、
unresolved-import recovery は task R-010 に残る。export validation は、必要な resolver
table を埋める後続の import、name、label、symbol task とともに成長する。

## 目的

このモジュールは、resolver が所有する semantic import / export resolution を
仕様化する。parse 済みの import/export directive 形状と phase 0 の module index
から、決定的な import graph、`ResolvedImport` entry、`ResolvedExport` entry、
および回復可能な unresolved-import record を作る。

resolver は source text の pre-scan、tokenize、parse、package discovery、
source file loading、build-side module index construction を行わない。frontend
preprocessing は、tokenization が暫定 lexical summary を読み込めるように、
candidate import stub を浅く pre-scan できるが、それらの stub は正本ではない。
semantic import resolution は resolver output を公開する前に、`SurfaceAst` から
すべての import を再検証する。

## 入力

- `SurfaceAst` の import / export directive node。source range、source order、
  recovered syntax marker を含む。
- 現在の module の canonical `ModuleId`。
- task R-007 の `ModuleIndexInput`。これは build-side `ModuleIndexProvider`
  contract によって backed される。
- 後続の resolver task が提供する current-workspace module の source-backed
  summary と dependency summary projection。この仕様は task R-024 より前に
  `mizar-artifact` の `ModuleSummary` schema を仮定しない。

parser が `SurfaceAst` node として表現できる場合、malformed / recovered syntax は
この phase に見える。parser が directive node を生成できない場合、resolver は
semantic directive を創作しない。

task R-009 の graph builder は path resolution の後から始まるため、canonical source /
target module identity だけを受け取る。その node universe は、builder に明示的に渡された
source module 集合と、その canonical import target である。graph ordering に参加すべき
zero-import module は、空の candidate set として明示的に渡す。unknown source / target
module は invalid builder input であり、最終的な semantic unresolved-import recovery は
task R-010 が所有する。

## 出力

import phase は次を生成する:

- semantic import resolution に参加する各 module の import-graph node。
- 現在の module から、成功裏に解決された dependency module への import edge。
- 解決済み acyclic graph node に対する決定的な topological order。
- syntactic import candidate ごとの `ResolvedImport` record。source range、
  source-order ordinal、利用可能な場合の canonical target module、任意の local
  alias、resolution status を含む。
- syntactic export candidate ごとの `ResolvedExport` record。source range、
  source-order ordinal、利用可能な場合の canonical target、resolution status を含む。
- failed path spelling、source range、failure class、後続診断に有用な partial
  namespace/package candidate を保持する unresolved-import record。

unresolved import は module を abort しない。resolver は failure を記録し、
利用不能な graph edge と imported export surface を省略し、独立した import、
export、local declaration の処理を継続する。

現在の `ResolvedAst` data shape は、spelling、range、failure class を持つ最小の
unresolved-import record を含む。以下で述べるより豊かな unresolved field は
task R-009 と R-010 の設計目標であり、実装は semantic contract を変えずに段階的に
着地してよい。

## 2 pass contract

### Pass A: candidate collection

candidate collection は、parse 済みの module-level import / export directive node を
source order で歩く。parser が `SurfaceAst` に表現した syntax だけを記録する。

各 import declaration について、Pass A は次を記録する:

- source range と source-order ordinal。
- 生の module path component。
- relative prefix（`.`、`..`、または none）。
- 任意の alias spelling と alias range。
- directive が malformed だが表現されている場合の recovery state。

branch import は branch member ごとに 1 candidate へ展開する。展開後の各
candidate は、precise member span と共有 base path provenance の両方を保持する。
これにより、source context を失わずに正確な member を診断できる。

Pass A は module existence、package identity、alias legality、visibility、
export validity、cycle を決定しない。

### Pass B: semantic validation

semantic validation は、収集した candidate を `ModuleIndexInput` に照らして解決する。
これは次を行う:

- absolute import を build-side module index の namespace root と namespace binding
  によって map する。
- relative import を current module identity から map する。
- alias を local namespace spelling として bind する。
- 現在利用可能な symbol/export summary の範囲で export target と private-item
  restriction を検証する。
- 解決済み module import の import graph edge を構築する。
- import cycle を拒否する。
- resolved / unresolved record を決定的順序で出力する。

frontend import stub と暫定 lexical summary は、parse 中の token classification の
理由を説明できるが、semantic import legality を検証しない。frontend と resolver が
食い違う場合、後続の semantic phase に対しては resolver output が正本である。

semantic validation が、frontend processing 中に暫定 lexicon entry を提供した import
を拒否した場合、resolver output は downstream consumer が依存する token
classification を tainted と印付けできるだけの provenance を保持しなければならない。
batch verification は tainted lexicon provenance に依存する semantic commitment を
抑制する。LSP recovery は navigation と後続 diagnostics のためにその tokenization を
使い続けてよい。

## module path resolution

import path resolution は次の順序に従う:

1. `.` または `..` を持つ path は current module と package に対する relative path である。
2. first component が build-side module index の namespace root または package-name
   binding と一致する path は cross-package であり、その binding を通じて解決する。
3. namespace binding と一致しない unprefixed path は package-local であり、current
   package に対して解決する。

cross-package import は `ModuleIndexInput` に対応 package を問い合わせ、残りの path
component を canonical module identity へ解決する。local import alias と source spelling
は canonical `ModuleId` の一部ではない。

package-local import と relative import は current module の package と path だけを使う:

- `.` は current module が属する module directory から解決する。
- `..` は current module が属する module directory の parent から解決する。
- unprefixed package-local path は current package root から解決する。
- package root からの escape は invalid である。

branch import member は base path の absolute / relative context を継承する。現在の
grammar が提供するのは `.` と `..` だけである。より深い relative prefix の resolver
挙動は、parser syntax が変わるまで範囲外である。

unprefixed first component が package-local module component と namespace/package
binding の両方に解釈できる場合、cross-package import のために namespace binding が勝つ。
同じ first component を持つ package-local module は明示的な relative import で到達できる。

## alias binding

alias は imported module の local namespace spelling である。canonical module
identity、exported module identity、graph order、artifact identity を変えない。

alias binding rules:

- `as` のない import は canonical final module path component を通じて見える。
- `as Alias` のある import は importing module 内で `Alias` を通じて見える。
- 同じ canonical module へ解決される duplicate import declaration は source record
  として保持する。一方 downstream import closure は canonical graph edge を 1 つだけ使う。
- 異なる canonical module を指す duplicate alias は決定的に拒否する。
- reserved namespace root または既に bind 済みの imported namespace spelling と衝突する
  alias は決定的に拒否する。

resolver は alias conflict について crate-local failure class を保持してよいが、
resolver diagnostic-code gap が閉じるまで public diagnostic code を創作してはならない。

## export resolution

`ResolvedExport` は semantic validation 後の export directive を表す。

resolver は次を検証する:

- exported module path が既知 module に解決されること。
- exported import alias が成功裏に bind された import を指すこと。
- re-exported module と symbol が、利用可能な summary に従って public であること。
- private item が export surface に copy されないこと。

詳細な symbol / label export validation は、後続の name、label、symbol task とともに
成長する。それらの table が存在するまでは、この phase は checker-owned fact を
捏造せず、unresolved または pending export target を記録する。
export failure record は、public resolver diagnostic code が仕様化されるまで、
unresolved export target と illegal private re-export を crate-local failure class として含む。

## cycle policy

import graph cycle は forbidden である。module import edge を解決した後、resolver は
strongly connected component を検出する。複数 module を持つ component、または
self-edge は cyclic として拒否する。

cycle record は決定的である:

- cycle 内の module は canonical `ModuleId` で並べる。
- edge は source-range offset、stable source-file proxy としての source module、
  次に canonical target module で並べる。
- cycle record は、最初に保持された cycle edge の source-range offset、
  stable source-file proxy としての source module、target module、source-order ordinal で並べる。
  R-009 ではすべての cycle record が同じ crate-local cycle failure class を持つため、
  source position が等しい場合はその edge candidate key を直接使う。

cyclic import は、影響を受ける graph edge を後続の import / name resolution から
利用不能にする。拒否された cycle の外側にある acyclic module は引き続き利用可能である。

topological order は解決済み acyclic module だけを含む。R-009 は accepted
`ImportGraph` の node list と edge list もその acyclic portion に限定する。cyclic
module は caller が後続 phase の degraded 処理を決めるまで `ImportCycle` record に
保持する。unresolved module import と拒否された cyclic component はその order から省き、
source provenance を持つ unresolved/cycle record として保持する。同時に ready になった
module は canonical `ModuleId` で並べる。

## unresolved import

unresolved import は first-class な resolver output であり、欠落 entry ではない。
各 record は次を保持する:

- 元の source range と path spelling。
- parse 可能な場合の normalized path component と relative prefix。
- source-order ordinal。
- failure class。
- failure までに見つかった partial package、namespace、module candidate。
- 該当する場合、parser から継承した recovery state。

failure class は public resolver diagnostic code が仕様化されるまで crate-local である。
必須 class には unknown namespace/package、unknown module、package root から escape する
relative import、malformed recovered directive、duplicate alias、alias/root conflict、
unavailable dependency summary、illegal import candidate state、import cycle が含まれる。

## determinism

resolution は、同等の source、module-index input、利用可能な summary に対して決定的である。

- source-order candidate は conflict check と user-facing provenance に使う。
- canonical graph edge は source-order conflict check 後に deduplicate し、source
  `ModuleId`、target `ModuleId`、保持した source provenance で sort する。cycle-record
  edge は source-range offset、source module、次に canonical target module で sort する。
  cycle record は最初に保持された cycle edge で sort する。
- `ResolvedImport` と `ResolvedExport` record は source-order ordinal を保持し、
  決定的 iteration を公開する。
- unresolved record と cycle record は source range、failure class、stable candidate
  key で sort する。

## boundary notes

- parser と syntax crate は directive syntax と recovery shape を所有する。
- frontend と lexer crate は shallow pre-scan、tokenization、暫定 lexical summary を所有する。
- `mizar-build` は package planning、module discovery、namespace binding、build-side
  module-index provider を所有する。
- resolver は semantic import/export validation、graph edge、alias binding、
  cycle rejection、unresolved-import representation を所有する。
- checker、type、proof、artifact crate は後続の type fact、proof fact、永続 artifact
  schema を所有する。
