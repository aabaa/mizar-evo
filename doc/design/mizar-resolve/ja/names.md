# Module: names

> 正本は英語です。英語版: [../en/names.md](../en/names.md)。

状態: task R-012 は R-013 から R-016 に向けた resolver-owned name-resolution
contract を仕様化する。task R-013 は namespace lookup slice を実装する。対象は
resolved / unresolved import-alias record、reserved namespace root、
package-name binding、current-package fallback、internal namespace unresolved /
ambiguous record である。final symbol lookup、
symbol の visibility/shadowing、unresolved / ambiguous symbol reference、dot-chain
finalization は後続 task に残る。public resolver diagnostic 向けの code range は既知の
`spec_gap` のままであるため、本書は diagnostic class、payload、ordering を仕様化するが、
public numeric diagnostic code は創作しない。

## 参照

この設計は、resolver-owned name contract を次から導出する:

- architecture 03 Step 4 と "Namespaces Resolve Before Symbols"。
- local variable / binder の scope と shadowing に関する spec chapter 4。
- scope、visibility、import、conflict rule に関する spec chapter 11。
- module namespace、import/export placement、visibility に関する spec chapter 12。
- selector-access syntax と selector-name restriction に関する spec chapter 13。
- overload candidate construction と checker-owned winner selection に関する
  spec chapter 19。
- diagnostic payload requirement と現行 resolver-code `spec_gap` に関する
  spec chapter 22。
- parser/syntax と resolver の dot-role handoff に関する appendix A。
- resolver-local `resolved_ast.md`、`env.md`、`imports.md`、`declarations.md`。

## 目的

names phase は import と declaration shell が存在した後、type checking、proof
checking、overload winner selection、selector type checking の前に、source name-use
site を解決する。source-shaped syntax と resolver-owned index を消費し、
`ResolvedAst` に明示的な name outcome を記録する。

入力:

- 現在の module の `SurfaceAst`。
- `imports.md` 由来の resolved import graph、resolved alias binding、
  unresolved import-alias dependency record。
- `declarations.md` 由来の declaration shell。
- current-module declaration の preliminary shell-derived symbol identity。
- source-backed fixture または summary が利用可能な場合の dependency symbol / namespace
  projection。これは後で `symbols.md` により精緻化される。
- qualified citation の namespace prefix に限る label scope。label resolution
  そのものは `labels.md` が仕様化する。

出力:

- qualified name resolution が使う namespace lookup result。
- resolver が試みた ordinary name-use site の `NameRefTable` entry。
- 明示的な unresolved / ambiguous symbol-name record。
- final symbol lookup 前に使う internal namespace unresolved / ambiguous record。
- selector 判断が checker/type information に属する dotted syntax の
  `DeferredSelector` record。
- deterministic ordering を持つ internal resolver diagnostic record。

## 境界

names phase は次を行ってよい:

- module namespace、import alias、namespace prefix、constructor name、
  user-symbol spelling、builtin prelude name の解決。
- 後続の overload / checker phase に渡す deterministic candidate set の構築。
- name-level visibility、shadowing、unresolved reference、name-level ambiguity の判定。

次は行ってはならない:

- type-directed overload winner の選択。
- inferred argument type による candidate ranking。
- cluster firing、coercion insertion、`qua` validity の判定。
- selector や structure field の type checking。
- namespace prefix を超える proof label validation。
- public user-facing diagnostic code の創作。

## Symbol Identity Handoff

R-011 の declaration shell は意図的に final `SymbolId` を割り当てない。それでも
name task は `NameRefTable` を raw string で埋めてはならない。R-014 が ordinary
reference の resolved result を記録する前に、resolver は declaration shell と
imported summary entry から preliminary symbol-identity projection を導出しなければならない:

- declaring `ModuleId`。
- declaration kind family。
- shell 由来の deterministic declaration ordinal または structural path。
- type checking なしに利用可能な source spelling または notation slot。
- 同じ spelling の declaration が複数共存し得るが `symbols.md` が final slot をまだ
  割り当てていない場合の overload/relation placeholder slot。

この projection は `resolved_ast.md` が要求する `SymbolId` shape と同じものを生成する
必要があるが、complete `SymbolEnv` entry ではなく、name reference のための resolver
identity である。R-019〜R-021 は同じ identity を kind-specific signature、overload
family、relation link、exported summary data で精緻化する。declaration を semantic
declaration として表現できない場合、resolver は `SymbolId` を創作せず unresolved または
ambiguous result を記録する。

## Name-Use Site

resolver は、spelling が namespace、declaration、builtin、または selector 境界を
表し得る syntax node に対して name lookup を試みる。

| Surface node / role | name phase の挙動 |
|---|---|
| `NamespacePath` | 各 segment を module namespace、import alias、namespace child として解決する。 |
| `QualifiedSymbol` | leading segment を namespace path として解決し、target namespace 内で final symbol / constructor spelling を解決する。 |
| signature や statement 内の declaration reference | 参照 spelling と namespace のみを解決し、type meaning は defer する。 |
| selector に見える dotted chain | namespace と local-term の境界のみを決定し、selector type checking は defer する。 |
| builtin prelude spelling | edition で有効で、より早い semantic scope に shadow されていなければ `ResolvedBuiltin` として解決する。 |

parser recovery node や malformed path segment は捨てず、unresolved record として保持する。

## Scope Model

semantic lookup は次の tier を順に考慮する。

1. local proof / block / statement binding。
2. 現在の definition / theorem / template parameter。
3. use site で可視な current-module declaration。
4. 明示的に import された public symbol と namespace。
5. imported facade module を通じて可視になる re-exported public symbol。
6. active edition で有効な builtin prelude symbol。

current-module declaration は、その declaration item が完了した後にのみ可視になる。
仕様上、後続 declaration への forward reference は許されない。imported summary は
base lexical environment を seed するが、semantic lookup は参照を確定する前に
import と visibility を検証する。

local binding は namespace component を shadow する。leading segment が local variable
または parameter として in scope の場合、後続の dot は namespace separator ではなく
selector syntax として扱う。resolver が term base を識別できるが selector field /
property の判定に type information が必要な場合、`DeferredSelector` を記録する。

overload 可能な declaration spelling は deterministic candidate set として収集する。
names phase は、type checking なしに利用できる事実に限り、namespace、declaration
point、visibility、symbol family、syntactic arity で filter してよい。overload winner は
選択せず、checker-owned overload ambiguity も報告しない。正当な same-spelling overload
set は ordered candidate、または 1 つの resolver-visible overload group として downstream
へ渡す。namespace/import/visibility conflict、または 1 つの overload-capable group として
表現できない candidate set だけが resolver-owned `Ambiguous` result として残る。

## Namespace-Before-Symbol Resolution

qualified reference は 2 層で解決する。

1. leading segment を namespace path として解決する。
2. target namespace の exported symbol table、または current module の場合は
   current-module visible table 内で final spelling を解決する。

namespace layer は、segment が失敗した後に symbol lookup へ fallback しない。失敗した
segment はその segment range を anchor とする unresolved namespace result を 1 つ生成し、
dependent symbol lookup は target を創作せず、失敗した namespace root を記録する。

alias spelling は local provenance にすぎない。解決済み identity は import alias ではなく
canonical `ModuleId` と `SymbolId` を使う。

## Namespace Lookup Precedence

task R-013 は leading namespace candidate が収集された後から開始する。local term
binding による shadowing は引き続き R-016 の dot-chain layer の責務である。後続の
dot-chain pass が first segment を local term と証明した場合、その segment に対して
namespace lookup は試みない。

namespace candidate は次の順で解決する。

1. resolved import alias を最初に考慮する。import resolution は reserved root と
   衝突する alias を拒否するため、resolver-visible な resolved alias はすでに
   alias-shaped provenance である。
2. unresolved import alias は reserved root と package binding より前に考慮する。
   そのような alias に依存する namespace candidate は internal
   `UnresolvedImportAlias` dependency、または duplicate import record が複数の
   canonical target を保持する場合は `AmbiguousImportAlias` を記録する。root /
   package / current-package lookup へ fall through しない。
3. reserved namespace root（`std`、`pub`、`pkg`、`dev`、`ext`）は longest root
   binding で照合する。binding が存在しない場合、root の後の first segment、suffix
   が空の場合は root 自体を failing segment とする。
4. package-name namespace binding は longest-prefix matching を使い、
   current-package fallback より優先する。
5. current-package fallback は、import alias、unresolved import alias、reserved
   root、package-name binding のいずれも first segment に一致しない場合だけ試みる。
6. package が選択された後、残りの segment は indexed module を指さなければならない。
   missing module lookup は、選択された package にその prefix を持つ module が存在しない
   最初の segment を報告する。stale namespace/package provider state は user-facing な
   module miss ではなく crate-local `ProviderError` とする。

## Visibility And Shadowing

visibility は use site で解釈する。

- current-module code は declaration point の後に current-module の public / private
  declaration の両方を見られる。
- importer が見られるのは public declaration と正当な public re-export だけである。
- private dependency symbol は、provisional lexical environment に spelling が存在しても
  access 不可である。
- private symbol は current-module lookup には参加できるが、dependency-facing exported
  symbol table へ serialize してはならない。

shadowing は source と tier に依存する。

- local variable と parameter は同じ spelling の namespace prefix と ordinary symbol を
  shadow する。
- 同じ visible spelling の current-module declaration は imported declaration より先に
  考慮する。ただし overload candidate construction は、後続 overload rule が参加を許す
  imported candidate を保持してよい。
- qualification も overload-family relationship もない互換しない imported candidate が
  複数ある場合、任意に選ばず deterministic ambiguity にする。
- module path または alias による qualification は lookup をその qualified namespace に
  制限し、関係のない unqualified candidate を抑止する。

## Unresolved And Ambiguous References

失敗または曖昧な ordinary symbol lookup はすべて `NameRefTable` に明示する。
final symbol spelling へ到達する前に起きた namespace failure は、dependent name
reference または diagnostic から link される explicit internal namespace record とする。
これは ambiguous candidate payload が deterministic な `SymbolId` candidate list である
現行 `NameRefTable` shape と一致する。

必須 unresolved symbol-name class:

- resolved namespace 内の missing symbol。
- inaccessible private symbol。
- recovered / malformed final spelling。
- higher semantic lookup tier が失敗した後に、edition で無効な builtin。

より早い semantic scope に shadow された builtin は、その早い binding に解決する。
unresolved builtin ではない。

必須 internal namespace unresolved class:

- unknown namespace segment。
- unresolved import または alias dependency。
- stale module-index provider state。
- recovered / malformed namespace segment。

必須 ambiguous class:

- 1 つの overload-capable group として表現できない複数 visible symbol。
- qualification を必要とする互換しない imported candidate。
- semantics を創作しなければ ranking できない recovered syntax の surviving candidate。

必須 internal namespace ambiguity class:

- 同じ alias または segment に対する複数 namespace binding。
- unique namespace target を妨げる import alias collision。
- semantics を創作しなければ ranking できない recovered namespace syntax の surviving
  namespace candidate。

checker-owned overload ambiguity は names-phase ambiguity ではない。すべての candidate が
1 つの overload-capable set の正当な member である場合、resolver は deterministic
candidate set または preliminary group identity を記録し、viability、specificity、
best-root selection は後続の type/overload phase に残す。

`NameRefTable` の ambiguous symbol candidate list は `resolved_ast.md` に従い、
canonical fully qualified name、module id、source range、local symbol id の順に sort する。
internal namespace candidate list は canonical module id、namespace path、source range、
stable variant name を使う。ordering は hash-map iteration や filesystem order に依存してはならない。

1 つの unresolved root は 1 つの primary resolver diagnostic を生成するべきである。
dependent node は root unresolved key へ link し、別個の原因を追加しない限り cascaded
primary diagnostic を出さない。

## Dot-Chain Finalization Contract

parser と syntax crate は dot role を source-shaped のまま保持する。task R-016 は、
namespace qualification と selector access のどちらにもなり得る chain について、
resolver-owned semantic boundary を最終決定する。

R-016 の contract:

- first segment が local term binding に解決する場合、その chain は selector syntax であり、
  その segment に対する namespace lookup は試みない。
- first segment が import alias、namespace root、package binding、または current-module
  namespace に解決し、local binding が shadow していない場合、resolver は final symbol
  segment まで leading chain を namespace-qualified として扱う。
- resolver が term base を識別できるが selector validation に type information が必要な
  場合、`DeferredSelector` を記録する。
- namespace と selector のどちらの解釈も不可能な場合、最初に決定的に失敗した segment で
  unresolved とする。
- complete-source fixture によりより狭い規則が必要になった場合、最終 R-016 実装は本書を
  更新しなければならない。

この contract は handoff を記録する。selector field lookup や selector-call type checking
は行わない。

## Diagnostics

external diagnostic specification が resolver code range を割り当てるまでは、name
diagnostic は crate-local/internal record のままである。次を含まなければならない。

- failing use site または segment の primary source range。
- 関連 import、declaration、candidate の secondary range。
- failure class または ambiguity class。
- attempted spelling と normalized namespace prefix。
- ambiguous lookup の deterministic candidate key。
- cascade suppression のための root unresolved key。

diagnostic ordering は primary range、failure / ambiguity class name、attempted
spelling、stable candidate key を使う。public numeric code は
`doc/spec/en/22.error_handling_and_diagnostics.md` が resolver code ownership を割り当てる、
または委譲した後にだけ追加する。

## Implementation Task 向け Planned Tests

R-012 は documentation-only change なので test は追加しない。後続 task は次を cover する。

- namespace segment lookup と missing-segment range（R-013）。
- qualified / unqualified visibility、shadowing、private access（R-014）。
- unresolved / ambiguous candidate ordering と cascade suppression（R-015）。
- selector-vs-namespace dot-chain finalization（R-016）。
