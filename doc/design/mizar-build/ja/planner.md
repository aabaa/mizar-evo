# Module: planner

> 正本は英語です。英語版: [../en/planner.md](../en/planner.md)。

## 目的

`planner` は `mizar-build` の pipeline phase 0 を実装する。package、
workspace、lockfile の metadata を読み取り、source loading、module index、
resolver setup、後続の scheduler graph construction が消費する決定的な
`BuildPlan` を生成する。

planner が所有するのは package level の事実だけである。manifest を検証し、
package dependency を lockfile に照合して解決し、edition と verifier/build
configuration を記録し、正準 package 順序を出力する。`.miz` source text の
読み取り、source id の割り当て、snapshot 作成、import 解析、name resolution、
verification phase の実行は行わない。

## 文脈

- [doc/spec/ja/23.package_management_and_build_system.md](../../../spec/ja/23.package_management_and_build_system.md)
- [doc/design/architecture/ja/00.pipeline_overview.md](../../architecture/ja/00.pipeline_overview.md)
- [doc/design/architecture/ja/03.module_and_symbol_resolution.md](../../architecture/ja/03.module_and_symbol_resolution.md)
- [doc/design/architecture/ja/14.parallel_verification_and_scheduling.md](../../architecture/ja/14.parallel_verification_and_scheduling.md)
- [doc/design/internal/ja/01.compiler_driver_and_pipeline_scheduler.md](../../internal/ja/01.compiler_driver_and_pipeline_scheduler.md)
- [doc/design/mizar-session/ja/source.md](../../mizar-session/ja/source.md)
- [doc/design/mizar-session/ja/snapshot.md](../../mizar-session/ja/snapshot.md)

## 入力

### Package Manifest: `mizar.pkg`

各 package は package root に TOML package manifest を 1 つ持つ。

```toml
[package]
name = "algebra"
version = "1.2.3"
authors = ["Alice <alice@example.com>"]
license = "MIT"
description = "Algebraic structures"
edition = "2025"

[dependencies]
mml_core = "^1.0.0"
topology = { version = "^0.9.0", features = ["metric"] }

[dev-dependencies]
test_utils = "^0.2.0"

[verifier]
max_axioms = 128
atp_timeout = 30
default_solver = "auto"
require_kernel_certificates = true

[build]
incremental = true
cache_dir = ".mizar-cache"
artifact_dir = "build"
```

必須 field:

| Field | 規則 |
|---|---|
| `package.name` | 小文字 `snake_case` で、`[a-z][a-z0-9]*(?:_[a-z0-9]+)*` に一致すること。hyphenated spelling は拒否し、正規化しない |
| `package.version` | SemVer 2.0.0 version |

省略可能な package field は第 23 章の既定値に従う。`package.edition` が省略
された場合は toolchain の現在の stable Mizar edition を使う。初期に対応する
stable edition は `2025` である。

Dependency entry は package id を key とし、version constraint 文字列、または
必須 `version` と省略可能な `features` を持つ inline table のどちらかで書く。
Dependency package id は `package.name` と同じ package-id spelling rule に従う。
feature 文字列は manifest data として保持する。feature semantics は phase 0
ではなく、将来の registry/feature resolver の責務である。

`dev-dependencies` は `dependencies` と同じように parse/validate されるが、
dependency kind `dev` として印を付ける。`BuildPlan` は normal と dev の
dependency record を常に保持する。planner API は driver request から dependency
selection（通常検証では `normal`、test/development target では `normal + dev`）
を受け取り、この plan の dependency graph でどの edge を active にするかを決める。

未知の top-level table、既知 table 内の未知 field、dependency table 内の重複
package id、絶対 build path、package root から脱出する path、不正な SemVer
version、不正な version constraint、不正な solver 名は manifest error である。

### Workspace Manifest: `mizar.workspace`

multi-package workspace は workspace root に TOML workspace manifest を持つ。

```toml
[workspace]
members = ["algebra", "topology"]
```

workspace manifest では `workspace.members` が必須である。各 member path は
workspace root からの相対 directory path である。特別な member path `.` は
workspace root package を意味する。それ以外の member path は空であってはならず、
絶対 path、`.`/`..` component、workspace root からの脱出、host-specific な
非正準 spelling を含んではならない。各 member directory は `mizar.pkg` を
含まなければならない。

`mizar.workspace` が存在しない場合、workspace は `mizar.pkg` を含む directory
を root とする single-package workspace である。両方の file が同じ root に
存在する場合、member discovery については workspace manifest が正本であり、
root の `mizar.pkg` は `members` によって member として列挙されている場合に
のみ有効である。

member package は workspace lockfile と cache root を共有する。重複 member
path、重複 package id、member package の manifest name とその package に対する
dependency key の不一致は manifest error である。

### Lockfile: `mizar.lock`

lockfile は workspace root の TOML file であり、再現可能 build に使う正確な
dependency assignment を記録する。

```toml
schema_version = 1

[[package]]
name = "algebra"
version = "1.2.3"
source = { kind = "workspace", path = "algebra" }
dependencies = [
  { name = "mml_core", version = "1.0.0" },
]

[[package]]
name = "mml_core"
version = "1.0.0"
source = { kind = "registry", registry = "default", checksum = "sha256:..." }
dependencies = []
```

この planner が対応する lockfile schema は `schema_version = 1` だけである。
各 package entry は次を持つ。

| Field | 規則 |
|---|---|
| `name` | package id spelling rule |
| `version` | exact SemVer version |
| `source.kind` | `workspace` または `registry` |
| `source.path` | workspace package では必須。workspace root からの相対 path |
| `source.registry` | registry package では必須 |
| `source.checksum` | registry package では必須 |
| `dependencies` | exact `{ name, version }` entry の array |

Lockfile entry は package id で一意である。workspace member は lockfile に存在し、
version が一致し、`source.kind = "workspace"` でなければならない。manifest
dependency は lockfile 内のちょうど 1 つの package entry に対応し、その version
は manifest constraint を満たさなければならない。lockfile dependency edge は
存在する locked package を指さなければならない。

planner は既存 lockfile を検証する。`mizar.lock` の更新や再書き込みは行わない。
lockfile generation と registry solving は同じ data model の上にある将来作業である。

## BuildPlan Model

```rust
struct PlanRequest {
    workspace_root: WorkspaceRoot,
    dependency_selection: DependencySelection,
    toolchain: ToolchainInfo,
}

enum DependencySelection {
    Normal,
    NormalAndDev,
}

struct BuildPlan {
    workspace_root: WorkspaceRoot,
    packages: Vec<PackagePlan>,
    dependency_graph: DependencyGraph,
    lockfile: Lockfile,
    toolchain: ToolchainInfo,
    verifier_config: WorkspaceVerifierConfig,
    build_config: WorkspaceBuildConfig,
}

struct PackagePlan {
    package_id: PackageId,
    version: Version,
    root: PackageRoot,
    source_root: PackageSourceRoot,
    manifest_path: NormalizedManifestPath,
    edition: Edition,
    dependencies: Vec<PackageDependency>,
    verifier_config: VerifierConfig,
    build_config: BuildConfig,
}

struct PackageDependency {
    package_id: PackageId,
    requested: VersionConstraint,
    resolved: Version,
    kind: DependencyKind,
    features: Vec<String>,
}

enum DependencyKind {
    Normal,
    Dev,
}
```

`PackageId`、`Edition`、`WorkspaceRoot`、`ToolchainInfo`、snapshot-facing な
source identity wrapper は `mizar-session` 由来である。planner は検証済みの
package id、edition、root、normalized path をそれらの API に渡す。`SourceId`、
`BuildSnapshotId`、source hash を生成してはならない。

`source_root` は package root の下にある normalized `src` directory である。
planner はこの path を記録するが、module を列挙せず、その directory が既に存在
することも要求しない。source tree が欠落または不正な場合の diagnostics は、
source loading と module indexing がそれぞれの error surface で報告する。

`packages` は dependencies before dependents の順に整列する。同順位は canonical
package id で決める。root workspace member は package plan として保持される。
locked registry package は、module index や後続 phase が artifact を必要とする
場合に dependency package plan として現れてよい。

`dependency_graph` は dependent から dependency への package-level correctness
edge を記録する。`packages` vector はそれらの stored edge に対する reverse
topological order であり、すべての dependency がすべての dependent より前に現れる。
graph は cycle、重複 package id、package 内の重複 dependency key、missing lock
entry、lockfile が manifest constraint を満たさない場合、未対応 edition を拒否する。
同一 workspace 内で同じ package id の複数 version を同時に許可しない。

`WorkspaceVerifierConfig` と `WorkspaceBuildConfig` は deterministic aggregate である。
toolchain default と、各 package の effective config を package id 順に並べた list
を含む。cross-package merge ではなく、package-specific setting を消さない。
Snapshot/config hash はこれらの aggregate record、または各 package の effective
verifier/build config と等価な canonical encoding を含めなければならない。
downstream code は package-scoped phase では package-local setting を使う。

## Planning Algorithm

1. `mizar.workspace` または `mizar.pkg` から workspace mode を発見する。
2. workspace manifest、各 member package manifest、workspace lockfile の TOML を parse する。
3. schema、package id、SemVer version、version constraint、enum value、config default、member path、build path を検証する。
4. package id を key にした package table を作り、重複を拒否する。
5. すべての workspace package に対応する lockfile entry があることを検証する。
6. 各 manifest dependency を lockfile に照合し、exact resolved version を集め、request の dependency selection に従って active edge を印付ける。
7. active package dependency graph を作り、cycle を拒否する。
8. 対応 edition を検査し、verifier/build default を merge する。
9. canonical ordering と deterministic diagnostics を持つ `BuildPlan` を生成する。

Phase 0 は manifest/lockfile validation の blocking error で止まってよい。その場合も、
同じ実行が同じ error list を報告するように diagnostics は canonical order で返す。

## 決定性

planner の externally visible output はすべて正準である。

- package id は manifest spelling をそのまま使い、hyphen normalization は行わない。
- package plan は dependencies-before-dependents、その後 package id で整列する。
- dependencies は dependency kind、package id、resolved version、feature list で整列する。
- duplicate feature は manifest diagnostic とし、受理された feature list は plan 内で整列する。
- diagnostics は normalized path、manifest key path、利用可能なら source range、diagnostic category、rejected value で整列する。
- lockfile package entry は TOML order ではなく package id で解釈する。
- path output は normalized `/` separator を使い、published plan data には host-specific absolute path を含めない。

planner result は hash-map iteration order、filesystem traversal order、wall-clock
time、session-local id、task id、registry network timing に依存してはならない。

## Diagnostics

Planner diagnostics は build/manifest diagnostics である。少なくとも次を含む。

| Category | 例 |
|---|---|
| manifest syntax | invalid TOML, duplicate TOML keys |
| manifest schema | missing required tables or fields, unknown fields |
| package identity | invalid package id, duplicate package id |
| versioning | invalid SemVer, invalid constraint, lock version mismatch |
| workspace layout | missing member manifest, duplicate member, member outside root |
| lockfile | unsupported schema, missing package, unknown locked dependency |
| graph | dependency cycle, incompatible diamond dependency |
| config | unsupported edition, invalid solver, invalid build path |

Diagnostics は manifest path、key path、offending value、stable category を持つべきである。
TOML span が利用できる場合は primary span として付与する。span がない場合も、
path と key path が stable location である。

現在の外部 diagnostic-code specification には manifest 専用 range が予約されていない。
その range が割り当てられるまでは、`mizar-build` は structured planner diagnostics
を内部に保持し、driver boundary で user-facing build diagnostics に map する。

## Tests

planner test suite は次を covered する。

- valid single-package / multi-package workspace manifest;
- hyphenated name を含む invalid package-id spelling;
- required package/workspace/lockfile field の欠落;
- invalid TOML と deterministic multiple-error ordering;
- string form と inline-table form の dependency entry;
- lockfile missing package と version mismatch diagnostics;
- dependency cycle と incompatible duplicate version assignment;
- 同一 input、および TOML table order だけが違う input に対する deterministic `BuildPlan` equality。

## 制約と非目標

- この slice の planner は registry network resolution を行わない。
- planner は既存 lockfile を検証するが、書き換えない。
- planner は `.miz` module を発見しない。module discovery は `module_index.md` が仕様化する。
- planner は source/snapshot identity を割り当てない。それらは `mizar-session` が所有する。
- planner は `mizar-driver` に依存しない。driver が `mizar-build` に依存する。
- package dependency resolution は package level だけである。import graph resolution と symbol visibility は resolver の責務である。
