# モジュール: imports

> 正本は英語です。英語版: [../en/imports.md](../en/imports.md)。

状態: task R-009 は、すでに解決済みの canonical candidate 上で canonical import graph
construction と cycle rejection を実装する。task R-010 は、alias binding、
relative-prefix interpretation、namespace/package binding、明示的な unresolved-import
recovery を graph layer へ供給する resolver-owned source-shaped path candidate seam を
実装する。構文木からの import 収集は以下の限定 producer が所有する。
回復 directive 全体の収集と export validation は後続の resolver 作業に残す。

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

## 暫定 frontend 候補

`ImportPathCandidate::from_frontend_imports(&LexicalEnvironmentRequest)` は、
信頼された未変更の事前走査 stub から `Option<Vec<Self>>` を返す。
要求順を ordinal とし、構成要素、相対接頭辞、別名と範囲、stub 範囲を保持する。
source segment が一つなら直接パス、二つなら分岐の基底・メンバー範囲を保持する。
それ以外の個数、要求と異なる source id、逆転範囲では全件を拒否し、空入力は成功する。
ソース本文の認証や import の適法性判定は行わない。
要求には事前走査診断や AST 回復マーカーがないため、部分 stub は暫定のままであり、
回復なしの構文とは主張しない。既存候補の回復既定値は変えない。
正式な resolver 公開は `SurfaceAst` から import を再収集・検証する必要がある。
このメソッドは意味グラフ、字句 summary、artifact アクセスを提供しない。

## 構文木からの import 候補

`ImportPathResolver::resolve_frontend(&FrontendOutput<SurfaceAst>, &SourceUnit)` は信頼された未変更の frontend 出力と期待する source から `Option<ImportPathResolution>` を返す。source の一致、source identity、AST/key、全体で recovery がないこと、alias/branch の由来と本文の token framing を含む parsed/prescan import 全件の順序付き対応を要求する。source の package/module identity を用いて既存 path resolver で解決し、未解決 path は型付き結果、対応不正は None、空 prelude は空の解決結果とする。AST-key の存在は封印済み由来を証明しない。currentness、親の認証、E0022-only 継続を含む診断 admission は driver/IR に残る。完全な graph、summary readiness、canonical storage や phase 完了の根拠にはしない。

`ImportPathCandidate::from_surface_ast(&SurfaceAst)` は信頼された未変更の
parser 出力から `Option<Vec<Self>>` を返す。表現された `Root` →
`CompilationUnit` → `ItemList` をたどり、直下の import prelude をソース順に
収集する。入れ子の directive をモジュール import に昇格しない。
正常な import なしの単位は空ベクターを返す。根の連鎖の欠落・未対応形状、
非 import 項目より後の import、回復中の最上位項目、不正または回復中の import
部分木は全件拒否する。最上位の ErrorRecovery ノードも含む。この AST だけでは
スキップされた遅い import と別のスキップ構文を区別できないためである。
他の表現された非 import 項目の内部にある回復は射影の対象外とする。
たどる import ノードと由来には、
AST と同じ SourceId、順序が正しく親の範囲内にある範囲、実在する子参照を要求する。
これは構造上の由来の確認であり、本文・UTF-8 位置・任意 AST の認証ではない。
ソース読込、parser 回復、封印済み出力への束縛は既存の所有者に残す。

既存 parser 形状から直接・相対パス、別名宣言、分岐メンバーを読む。
回復フラグに加え、import キーワードと終端セミコロン、カンマによる完全な区切り、
`as` 別名の完結、分岐の記号と閉じ波括弧を確認する。parser 診断が回復ノードを
残さない場合もあるため、残った構造上の子だけを見て欠落構文を正常な候補にしない。
候補範囲、生のパス要素、相対接頭辞、別名と範囲、展開した分岐の基底・
メンバー範囲を保持し、prelude 全体で連続する ordinal を付ける。
未対応形状から候補を創作せず拒否する。収集後は既存の意味的パス resolver を使い、
このメソッドでは存在確認、namespace 束縛、別名の適法性、公開範囲、循環を判定しない。
稼働中の declaration-symbol runner は独自の AST 走査を持たず、この producer を使う。
これは正常な import の部分集合であり、Pass A 全体の回復 directive 表現、
公開診断、resolver 出力公開は実装しない。既存の分岐・別名・回復ソース fixture の
意図は保ち、空・混合 prelude、相対形、最上位境界、不正構造、source/range 不一致を
言語仕様に新しい主張を加えず producer テストで確認する。

## 入力

- `SurfaceAst` の import / export directive node。source range、source order、
  recovered syntax marker を含む。
- 現在の module の canonical `ModuleId`。
- task R-007 の `ModuleIndexInput`。これは build-side `ModuleIndexProvider`
  contract によって backed される。
- current-workspace module の source-backed summary と dependency summary projection。
  artifact-backed dependency projection は R-024 の canonical な `mizar-artifact`
  `ModuleSummary` consumption が供給する。imports は artifact schema を定義しない。

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

resolver path candidate は、現在の `ModulePath` 文字列表現の `.` separator によって
module-path component を encode する。この component encoding は task R-010 の
resolver-side seam であり、parser syntax の所有権は `mizar-parser` /
`mizar-syntax` に残る。

branch import member は base path の absolute / relative context を継承する。現在の
grammar が提供するのは `.` と `..` だけである。より深い relative prefix の resolver
挙動は、parser syntax が変わるまで範囲外である。

unprefixed first component が package-local module component と namespace/package
binding の両方に解釈できる場合、cross-package import のために namespace binding が勝つ。
同じ first component を持つ package-local module は明示的な relative import で到達できる。

package-local fallback は、reserved namespace root と package-name binding のどちらにも
一致しない場合にだけ適用される。reserved namespace root が一致したが binding がない場合、
import は unknown namespace/package として unresolved になる。package-name または
namespace binding が一致したが残りの module path が未知の場合、import は unknown module
として unresolved になり、resolver は package-local path へ fallback しない。

## alias binding

alias は imported module の local namespace spelling である。canonical module
identity、exported module identity、graph order、artifact identity を変えない。

alias binding rules:

- `as` のない import は canonical final module path component を通じて見える。
- `as Alias` のある import は importing module 内で `Alias` を通じて見える。
- 同じ canonical module へ解決される duplicate import declaration は source record
  として保持する。一方 downstream import closure は canonical graph edge を 1 つだけ使う。
- 異なる canonical module を指す duplicate alias は決定的に拒否する。task R-010 は
  conflict している alias group のすべての member を unresolved とし、その alias group
  について graph edge を出力しない。
- reserved namespace root と衝突する alias は、task R-010 が決定的に拒否する。
- 既に bind 済みの imported namespace spelling と衝突する alias は、後続の import/name
  integration がそれらの namespace binding をこの phase に提供してから決定的に拒否する。

resolver は alias conflict について crate-local failure class を保持してよいが、
resolver diagnostic-code gap が閉じるまで public diagnostic code を創作してはならない。

task R-010 の source-shaped candidate は、caller が提供した場合、明示的な alias range、
branch base/member provenance、parser-recovery flag を保持する。これにより、resolver が
parser syntax traversal を所有せずに diagnostic provenance を保てる。

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

task R-010 は、task R-009 の canonical graph builder を呼ぶ前に unresolved path
candidate を filter する。未知の canonical module は引き続き direct graph builder input
として invalid である。

## unresolved import

unresolved import は first-class な resolver output であり、欠落 entry ではない。
各 record は次を保持する:

- 元の source range と path spelling。
- parse 可能な場合の normalized path component と relative prefix。
- source-order ordinal。
- failure class。
- failure までに見つかった partial package、namespace、module candidate。
- 該当する場合、parser から継承した recovery state。

仕様 §22.3.5 は import の意味 6 件に E0220～E0225 を割り当てるが、driver 所有の E0022-only 診断専用継続による E0220～E0224 射影以外では record を crate-local に保つ。E0225 の registry 採用は後続とする。
必須 class には unknown namespace/package、unknown module、package root から escape する
relative import、malformed recovered directive、duplicate alias、alias/root conflict、
unavailable dependency summary、illegal import candidate state、import cycle が含まれる。

task R-010 の interim `ImportPathResolution` は、この recovery information のうち
path-candidate subset を resolver-owned source-shaped record として保持し、成功裏に
解決された record だけを `ModuleImportCandidates` へ射影できる。dependency-summary
failure と cycle failure は、別個の後続または graph-layer record に残る。既存の
`ResolvedAst` import table は task R-004 由来の最小 unresolved import shape をまだ含む。
完全な `ResolvedImports` integration は、ここで創作せず、後続の source-walk と
import/name task と pair する。

## 公開診断射影の前提

[仕様 §22.3.5](../../../spec/ja/22.error_handling_and_diagnostics.md#2235-モジュール-import-解決) が draft 件数、source 範囲、識別、batch 全体の変換規則を所有する。producer は実際の現行 parser 出力を読込み済み本文、snapshot、module 識別子に結合する。`from_surface_ast` は構造を調べるが本文を認証しない。循環射影は参加する全 source と保持済み graph edge も必要とする。builder の既存の offset 優先 cycle 順序は変えない。

下位テストでは実ソースを `MizarParserSeam` で解析し、AST import 収集、型付き module index による解決、graph 構築を検査できるが、driver 公開の証明にはならない。

| 意味 | ソース seed と必要な index 設定 |
|---|---|
| E0220 | `import mml.no_such;`、`mml` root 束縛なし |
| E0221 | `import dep.missing;`、`dep` を package に束縛し `missing` module を登録しない |
| E0222 | root module `app.main` から `import ..common;` |
| E0223 | `import dep.logic as shared, .util as shared;`、`dep` 束縛と `dep.logic`・local `app.util` を登録 |
| E0224 | `import dep.logic as mml;`、`dep` 束縛と `dep.logic` を登録 |
| E0225 | `app.main` が `.util`、`app.util` が `.main` を import。別途 `app.main` が `.main` を import。local module 2 件を登録 |

同じ directive 範囲を共有する不在の `dep` 分岐 member 2 件、別名 peer の重複、source をまたぐ循環の結合、自己循環、入力順の入替えも検査する。comment と複数 byte 文字の後の正確な source slice を確認し、他の SourceId、逆転・範囲外・UTF-8 境界外の range、source 欠落、snapshot 混在、未知・未割当 class、draft 構築失敗では batch の部分発行を拒否する。`mml` seed は仕様 §12.2.1 に従う。既存 source corpus が示すのは E0221/E0223 の意味だけで、公開 code はない。A28 Frontend が封印するのは正常出力だけであり、字句の事前解決が失敗 import AST の封印前に E0022 を発行し得る。driver の E0022-only 経路は、失敗出力を封印せず真正な実ソースから E0220～E0224 を射影する。一般の resolver と E0225 のテストには、完全な意味入力/graph 経路、workspace 字句 summary producer、残る registry/bridge 採用が引き続き必要である。

## determinism

resolution は、同等の source、module-index input、利用可能な summary に対して決定的である。

- source-order candidate は conflict check と user-facing provenance に使う。
- canonical graph edge は source-order conflict check 後に deduplicate し、source
  `ModuleId`、target `ModuleId`、保持した source provenance で sort する。cycle-record
  edge は source-range offset、source module、次に canonical target module で sort する。
  cycle record は最初に保持された cycle edge で sort する。
- `ResolvedImport` と `ResolvedExport` record は source-order ordinal を保持し、
  決定的 iteration を公開する。
- unresolved record は source-order ordinal、source range、failure class、stable
  candidate key で sort する。cycle record は source range、failure class、stable
  candidate key で sort する。

## 公開 enum の前方互換性

task R-026 は frontend task 25 の public-enum decision procedure をこの module に適用する。
`imports` が所有する公開 resolver enum はすべて forward-compatible API surface であり、
`#[non_exhaustive]` を維持しなければならない:

- `ImportPathPrefix`
- `ImportPathFailureClass`
- `ImportGraphBuildError`

この module は exhaustive な公開 enum 例外を所有しない。下流 consumer は wildcard
または fallback arm を持たなければならない。resolver 内部の match は、仕様化済みの
挙動を実装する範囲で、現在表現されている variant に対して exhaustive でよい。

## boundary notes

- parser と syntax crate は directive syntax と recovery shape を所有する。
- frontend と lexer crate は shallow pre-scan、tokenization、暫定 lexical summary を所有する。
- `mizar-build` は package planning、module discovery、namespace binding、build-side
  module-index provider を所有する。
- resolver は semantic import/export validation、graph edge、alias binding、
  cycle rejection、unresolved-import representation を所有する。
- checker、type、proof、artifact crate は後続の type fact、proof fact、永続 artifact
  schema を所有する。
