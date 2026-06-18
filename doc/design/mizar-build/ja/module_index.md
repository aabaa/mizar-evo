# mizar-build モジュール索引

> 正本は英語です。英語版: [../en/module_index.md](../en/module_index.md)。

## 目的

この文書は、`planner.md` が決定的な `BuildPlan` を生成した後に
`mizar-build` が生成する phase 0 のモジュール索引を定義する。

これはアーキテクチャ 03「Step 1: Build Package and Module Indexes」を
精緻化する。モジュール索引は、計画済み package を正準的な module identity と
source/artifact location に対応付ける。resolver はこの索引を provider input として
消費し、自力で workspace root を再発見したり package identity を創作したりしない。

## スコープ

`mizar-build` が所有するもの:

- `BuildPlan.packages` から導出される package index entry
- package name と artifact が宣言した言語 namespace root を package entry に
  対応付ける namespace binding
- 各 workspace package の source root 以下にある現在の workspace の `.miz`
  source file の発見
- package identity と package-relative module path からの正準的な `ModuleId` 構築
- dependency artifact から供給される dependency module summary の読み込みまたは参照
- 重複または不正な module entry に対する決定的 diagnostics

`mizar-build` が所有しないもの:

- import、export、declaration、alias の parse
- relative import path の解決や visibility の検証
- `SourceId`、`SourceMapId`、`BuildSnapshotId`、source hash の割り当て
- artifact がないときの synthetic dependency summary の生成
- overload、type、proof obligation、checker input の選択

これらの責務は `mizar-frontend`、`mizar-resolve`、`mizar-session`、後続の
semantic phase に残る。

## 入力

module-index construction は次を受け取る:

- 検証済みの `BuildPlan`
- workspace package source root 用の source-layout reader
- registry または既に build された package input 用の dependency artifact metadata
- plan に記録されたものと同じ toolchain identity と edition identity

source-layout reader は filesystem、virtual test fixture、driver が提供する
snapshot view のいずれを走査してもよいが、等価な source layout から得られる索引は
byte 単位で同一でなければならない。

## identity モデル

正準 module identity は package-scoped である:

```rust
struct ModuleId {
    package: PackageId,
    path: ModulePath,
}

struct ModulePath {
    components: Vec<String>,
}
```

`PackageId` は検証済み package manifest と lockfile に由来する。`ModulePath` は
source-root-relative な file path である `source_relative_path` から導出する:

- `lib.miz` は `ModulePath("lib")` に対応する。
- `groups/basic.miz` は `ModulePath("groups.basic")` に対応する。
- `.miz` extension は取り除く。
- path separator は `.` になる。
- すべての path component は有効な language identifier でなければならない。
- module-path conversion の前に path spelling は `/` separator へ正規化する。

local import alias は `ModuleId` に決して含めない。たとえば
`algebra.groups.basic as Grp` として import しても、alias なしで
`algebra.groups.basic` を import しても、同じ `ModuleId` に解決される。alias
binding は resolver が所有する。

package 内の relative または prefix なし import path は、resolver が現在の
module と package に照らして解釈する。cross-package import は package index が公開する
package namespace を使う。

module index は surface import path を parse しないが、resolver がそれを解釈するために
必要な namespace binding を公開しなければならない。現在の `BuildPlan` data は常に、
検証済み package id spelling を prefix とする package-name binding を生む。これは仕様第
23 章の `algebra.groups.basic` のような cross-package import と対応する。dependency
artifact または registry metadata が仕様第 12 章の言語 root（`std`、`pub`、`pkg`、
`dev`、`ext`）を宣言している場合、それらを追加の `NamespaceIndexEntry` entry として運ぶ。
indexer は plan または dependency artifact に存在しない language root を推測してはならない。
どの binding が surface path に一致するかを選び、未解決の namespace root を報告するのは
resolver である。

## データモデル

次の形は contract を定義するものであり、正確な Rust 名とは限らない:

```rust
struct ModuleIndex {
    packages: Vec<PackageIndexEntry>,
    namespace_bindings: Vec<NamespaceIndexEntry>,
    modules: Vec<ModuleIndexEntry>,
    dependency_summaries: Vec<DependencyModuleSummaryRef>,
}

struct PackageIndexEntry {
    package_id: PackageId,
    version: Version,
    edition: Edition,
    source: PackageIndexSource,
    dependencies: Vec<PackageId>,
}

struct NamespaceIndexEntry {
    root: NamespaceRoot,
    prefix: Vec<String>,
    package_id: PackageId,
}

enum NamespaceRoot {
    PackageName,
    Std,
    Pub,
    Pkg,
    Dev,
    Ext,
}

enum PackageIndexSource {
    Workspace {
        package_root: String,
        source_root: String,
        manifest_path: String,
    },
    RegistryArtifact {
        registry: String,
        checksum: String,
    },
}

struct ModuleIndexEntry {
    module: ModuleId,
    package_id: PackageId,
    module_path: ModulePath,
    location: ModuleIndexLocation,
    edition: Edition,
}

enum ModuleIndexLocation {
    WorkspaceFile {
        source_root: String,
        normalized_path: String,
        source_relative_path: String,
    },
    DependencySummary {
        artifact: String,
        content_hash: Hash,
    },
}

struct DependencyModuleSummaryRef {
    module: ModuleId,
    artifact: String,
    content_hash: Hash,
}
```

`package_root`、`source_root`、`manifest_path`、
`ModuleIndexLocation::WorkspaceFile.normalized_path`、
`ModuleIndexLocation::WorkspaceFile.source_relative_path` は normalized plan path であり、
session source identity ではない。workspace module では:

- `normalized_path` は package-root-relative で、必須の `src/` boundary を含む。
  例: `src/groups/basic.miz`
- `source_relative_path` は source-root-relative で、`src/` を含まない。
  例: `groups/basic.miz`
- `ModulePath` は `source_relative_path` から導出する。
- dependency-summary-backed module は source path を持たず、workspace file であるかのように
  source loading へ渡してはならない。

後続 phase が source text を load するときは、package root と package-root-relative な
`normalized_path` を `mizar-session` の source-loading boundary に渡さなければならない。
`mizar-session` は、その path を `NormalizedPath` として受理すること、`SourceId` の割り当て、
source hash の計算、snapshot membership の記録を引き続き所有する。

registry package は通常、workspace source file ではなく dependency-summary-backed な
module entry を提供する。artifact から受理した dependency module summary はすべて、
次の両方を生成する:

- 正準 `ModuleId` によって dependency summary content と hash を request できるようにする
  1 つの `DependencyModuleSummaryRef`
- workspace module と dependency module の両方に対して `provider.module()` が同じように
  動くようにする、`ModuleIndexLocation::DependencySummary` を持つ 1 つの `ModuleIndexEntry`

dependency artifact がない、または計画済み dependency package に対する有効な
module summary を含まない場合、construction は module content を捏造せず
`MissingDependencySummary` を報告しなければならない。Provider の
`UnavailableDependencySummary` error は、構築済み index に存在する module について、
dependency-summary-backed ではない module の dependency summary content を呼び出し側が
要求した場合に限る。

## 構築アルゴリズム

1. `BuildPlan.packages` の決定的な package-plan order から開始する。
2. 各 package plan に対して 1 つの `PackageIndexEntry` を作る。
3. package-name binding と artifact が宣言した language namespace binding ごとに
   正準 `NamespaceIndexEntry` record を作り、異なる package を指す重複
   `(root, prefix)` binding を拒否する。
4. 各 workspace package について、その `source_root` を再帰的に走査し、`.miz`
   file だけを集める。
5. 発見した各 path を package root からの相対 path（`src/` を含む
   `normalized_path`）と package source root からの相対 path（`src/` を含まない
   `source_relative_path`）の両方として正規化する。
6. 正規化した source-root-relative path を package-scoped `ModulePath` に変換する。
7. source file ごとに 1 つの workspace-file-backed `ModuleIndexEntry` を作る。
8. dependency artifact が供給する dependency module summary record を、正準
   `ModuleId` で key して読み込む、または参照し、対応する dependency-summary-backed
   module entry を作る。
9. namespace binding を `(root, prefix, package_id)` で整列する。
10. module を `(package_id, module_path, location key)` で整列する。
11. 重複した `(package_id, module_path)` entry を拒否する。
12. 完全な `ModuleIndex` または決定的 diagnostics を返す。

indexer は構文、import、declaration の内容を知るために source text を読まない。
file discovery のために filesystem metadata を必要とすることはあるが、semantic content は
後続の source/frontend phase が load する。

## resolver provider contract

resolver は immutable provider を通じて module index を消費する。provider は
phase-service boundary であり、driver entry point ではない。

概念的には次を提供しなければならない:

```rust
trait ModuleIndexProvider {
    fn packages(&self) -> &[PackageIndexEntry];
    fn namespace_bindings(&self) -> &[NamespaceIndexEntry];
    fn package(&self, package: &PackageId) -> Result<&PackageIndexEntry, ModuleIndexProviderError>;
    fn package_for_namespace(&self, root: &NamespaceRoot, prefix: &[String]) -> Result<&PackageIndexEntry, ModuleIndexProviderError>;
    fn module(&self, module: &ModuleId) -> Result<&ModuleIndexEntry, ModuleIndexProviderError>;
    fn modules_for_package(&self, package: &PackageId) -> Result<&[ModuleIndexEntry], ModuleIndexProviderError>;
    fn dependency_summary(&self, module: &ModuleId) -> Result<&DependencyModuleSummaryRef, ModuleIndexProviderError>;
}
```

provider invariant:

- lookup は正準的な `PackageId` と `ModuleId` を key にする。
- local alias、import spelling、import order は lookup result を変えない。
- 返される slice はすべて正準順である。
- 同じ index に対する繰り返し呼び出しは等しい値を返す。
- provider error は決定的で side effect を持たない。
- real `ModuleIndex` が供給された後、provider error は代替 identity を割り当てたり
  workspace stub へ fallback したりしてはならない。
- `module()` は workspace module と dependency-summary-backed module の両方で動かなければならない。
  `dependency_summary()` は location が `DependencySummary` である module に対してのみ成功する。

`mizar-resolve` task 7 は resolver 側の interim seam と、そのテスト用
workspace-stub provider を所有する。その task が存在するまでは、`mizar-build` は
index の定義と構築を行えるが、`mizar-resolve` の代わりに resolver fixture や
resolver crate-local compatibility layer を創作してはならない。

## diagnostics

module-index diagnostics は phase 0 build diagnostics である。必要な category:

| category | 例 |
|---|---|
| source root | source root の欠落、source root が読めない |
| source path | source root 外の file、対応しない path encoding |
| module path | 不正な module component、空の module path |
| duplicate module | 2 つの file が同じ `(PackageId, ModulePath)` に対応する |
| namespace binding | 重複 prefix、対応しない namespace root、不正な prefix component |
| dependency artifact | module summary の欠落、不正な summary identity |
| provider | unknown package、unknown module、unavailable dependency summary |

diagnostics には、利用可能な場合は package id、normalized path または module id、
stable diagnostic category、rejected value を含める。package id、module path、
normalized path、category、value の順で整列する。

## 決定性

module index は filesystem traversal order、hash-map iteration order、platform path
separator、local import alias、current time、session-local source id に依存してはならない。
等価な source tree と dependency artifact は byte-identical な index と diagnostics を
生成しなければならない。

安定順序の規則:

- package entry は `BuildPlan.packages` に従う。
- namespace binding は `(root, prefix, package_id)` で整列する。`NamespaceRoot`
  variant は上記 enum に示した順で整列する。
- module entry は正準的な `(PackageId, ModulePath, location key)` で整列する。
- workspace-file location key は `normalized_path` である。dependency-summary
  location key は canonical artifact path と content-hash encoding を使う
  `(artifact, content_hash)` である。
- dependency summary は正準的な `ModuleId` で整列する。
- duplicate diagnostic では、最初の canonical path を保持される entry とし、
  後続の canonical path を整列順で報告する。
- provider result は格納された index と同じ正準順を使う。

## task 6 の終了条件

この仕様を引用する実装 task は次を満たさなければならない:

- `BuildPlan` と workspace source layout から `ModuleIndex` を構築する。
- `SourceId` や snapshot id を割り当てずに build-side provider/accessor contract を公開する。
- 複数 package fixture と alias-independent module identity をカバーする。
- source discovery が shuffle されても決定的 output を保つ。
- resolver stub fixture との provider parity を主張する前に、`mizar-resolve`
  task 7 の状態を確認する。

`mizar-resolve` task 7 がまだ open である場合、task 6 は resolver stub replacement と
resolver-fixture parity を外部 dependency gap として分類し、`mizar-build` の index と
build-side provider slice だけを完了しなければならない。resolver-owned な compatibility
work は `mizar-resolve` に残す。
