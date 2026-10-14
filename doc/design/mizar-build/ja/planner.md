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

produce_build_plan では収集済み member の既存相対座標 (root '.' を含む) と、同じ name/exact-version の workspace lock source.path を一致させる。package 名と directory 名の一致は要求しない。欠落/version/kind error の意味は保持し、それ以外で path が不一致なら既存順序で internal InvalidLockSource を '<lockfile>'、'<package>.source.path'、'<locked> != <member>' として返す。raw acquisition と manifest-only validator は変更せず、alias 正規化や metadata/currentness の認証を意味しない。

planner は既存 lockfile を検証する。`mizar.lock` の更新や再書き込みは行わない。
lockfile generation と registry solving は同じ data model の上にある将来作業である。

## Lock content fingerprint

`lockfile_content_hash(&Lockfile) -> mizar_session::Hash` は直接構築した任意の値を含む現行 typed lock の全 field を fingerprint にする。schema・assignment・provenance・現在の file の検証ではなく、file 読み込みや parser/planner policy を変更しない。raw TOML の comment/空白/順序、workspace root/member manifest、source text、toolchain、effective configuration、dependency artifact は別入力である。
既存 Artifact `CanonicalJson` の array/string と `canonical_json_string` を使う。root は `[schema, packages]`、package は `[name, version, source, dependencies]`、dependency は `[name, version]`、source は `["workspace", path]` または `["registry", registry, checksum]`。schema は full u64 の十進 string、version は prerelease/build を含む exact SemVer display string。dependency/package array は完全な canonical JSON tuple text で整列し、重複数を保持する。parser/validator が拒否する任意 typed 値も全現行 field を保持する。
preimage は UTF-8 prefix `mizar-build/parsed-lock-content/v1`、1 LF、canonical root JSON string (末尾 1 LF) の順。既存 Session `hash_text` の source-text-domain/length/BLAKE3 処理へ渡す Build content profile であり、新しい raw hash primitive、Artifact hash class、Cache key domain ではない。typed lock field の追加は profile 拡張レビューを必要とする。schema integer の切り捨て、Debug/raw-TOML encoding は使わない。
独立 literal golden で frame/field 順序を固定する。全 scalar/variant/edge、full u64/SemVer、入力順/重複数を検証し、実 native file と生成 hash を genuine snapshot/SourceLoad cache carrier に渡す。carrier は transport のみを示し、未知 toolchain field は既存 parser で失敗する。残りの fixture context に認証の意味はなく、service は実 NoKey を保ち、currentness/configuration/toolchain/pin/registry/runtime/Proof/cache 受理/bootstrap は依存待ちである。

## Native workspace input acquisition

`read_native_workspace_inputs(&WorkspaceRoot) -> Result<(Vec<WorkspacePackage>, Lockfile), String>` は明示された実 root から既存 input data を読み、親 directory 探索・toolchain/policy/context 選択を行わない。canonicalization 前の root metadata query に標準 path component を使い、末尾 separator/dot で最終 symlink 判定を回避させない。通常 root directory を要求する。
root/mizar.workspace を symlink をたどらず確認し、NotFound の不存在だけで単一 root package member `.` を選ぶ。他の取得失敗・不正 workspace は fallback なしの Err。存在時は実通常 UTF-8 file を読み parse_workspace_manifest に渡し、root/mizar.pkg が存在しても列挙 member のみを取得する。member normalization/order/duplicate/default 意味論は既存 parser が所有する。
検証済み member を canonical root の下へ解決し、通常 directory component と canonical containment を要求する。その位置の通常 mizar.pkg だけを取得して parse_package_manifest に渡す。symlink/dangling/special file・未対応 path/text encoding・missing member/file・metadata/read/parse failure は取得/parser String Err。source text は読まない。package name 重複は後続の既存 planner 検証へ残し、合成 merged package を作らない。
必須の通常 root/mizar.lock を UTF-8 で取得し parse_lockfile に渡す。欠落 lock 生成・dependency version 選択・registry 取得・file 書換えはしない。全取得/parse 成功後だけ既存 WorkspacePackage の member 対応と Lockfile を返し、先行 member 後の失敗も部分 tuple を返さない。String error は input 取得失敗で、新しい public 診断 code・parser/CLI 診断方針の置換ではない。
tuple は parsed data であり検証済み BuildPlan・認証済み manifest/lock/current snapshot ではない。cross-package/lock assignment は produce_build_plan の責任。実 runtime toolchain・lock/config digest・policy・dependency/capture/currentness 対応は caller/producer 所有で、placeholder identity・受理済み出力は供給しない。package toolchain constraint metadata は後述の境界に従う。locked workspace path の対応は produce_build_plan のみで検証し、package replay metadata は後述の computation policy boundary に従う。
実単一 package と列挙 multi-member file を既存 plan/layout・保護 native SourceLoad/Frontend/import discovery へ渡し、明示 clean parser input・実 NoKey と比較する。error control は workspace の absence/不正の区別、missing/late member/lock、不正 input、root/member/file link、部分成功なしを調べる。旧 parser/planner/layout/capture/diagnostics/試験は変えない。
filesystem read は atomic metadata snapshot・fresh hash attestation ではなく、caller が native submit 前に実 canonical context へ束縛する。source ID/hash・registry artifact・native CLI/LSP mapping/settings・digest/toolchain/pin producer・proof/cache/publication・全 Steps 6–8 は保留する。

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
    toolchain_constraint: Option<VersionConstraint>,
    version: Version,
    source: PackagePlanSource,
    edition: Edition,
    dependencies: Vec<ResolvedPackageDependency>,
    verifier_config: VerifierConfig,
    build_config: BuildConfig,
}

enum PackagePlanSource {
    Workspace {
        root: String,
        source_root: String,
        manifest_path: String,
    },
    Registry {
        registry: String,
        checksum: String,
    },
}

struct ResolvedPackageDependency {
    package_id: String,
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

workspace package では、`source.source_root` は package root の下にある
normalized `src` directory である。この implementation slice は normalized plan
path と registry metadata を `String` value として公開する。後続の source/snapshot
API は、それぞれの boundary で引き続き `mizar-session` の identity wrapper を受け取る。
planner は source-root path を記録するが、module を列挙せず、その directory が既に存在
することも要求しない。source tree が欠落または不正な場合の diagnostics は、
source loading と module indexing がそれぞれの error surface で報告する。registry
package は `PackagePlanSource::Registry` を使い、workspace source root ではなく
dependency-artifact input である。

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
lockfile の `source.kind = "workspace"` entry は、この planner が発見して
manifest を parse/validate した workspace member に対してのみ有効である。この
implementation slice では、member ではない dependency は registry source metadata
を使わなければならない。

`WorkspaceVerifierConfig` と `WorkspaceBuildConfig` は deterministic aggregate である。
各 package の effective config を package plan order で並べた list を含み、
隣接する `BuildPlan.toolchain` がそれらの config を解釈する toolchain identity を記録する。
cross-package merge ではなく、package-specific setting を消さない。
Snapshot/config hash はこれらの aggregate record、または各 package の effective
verifier/build config と等価な canonical encoding を含めなければならない。
downstream code は package-scoped phase では package-local setting を使う。

## Current package configuration content fingerprint

`workspace_config_content_hash(&WorkspaceVerifierConfig, &WorkspaceBuildConfig) -> mizar_session::Hash` は既存の package 対応付き型付き aggregate 二つの内容を指紋化し、対応の妥当性・metadata 出所・現在のファイル・runtime の完全性を検証しない。両リストを独立に保持し、zip・merge・重複 ID 上書き・欠落 record の推測をしない。ファイルを読まず、parser/planner/runtime policy を変更しない。manifest 表記・コメント・default 省略、package edition/version/path、dependency graph、lock、toolchain、runtime policy/backend/seed/budget/invocation は別入力である。
既存 Artifact `CanonicalJson` と `canonical_json_string` を使う。root は `[verifier_records, build_records]`、verifier record は `[package_id, max_axioms, atp_timeout, solver, computation_replay, require_kernel_certificates]`、build record は `[package_id, incremental, cache_dir, artifact_dir]`。u32 全値は JSON integer、boolean は JSON boolean、ID/directory は厳密な string とする。既存 solver variant の string は `auto`, `vampire`, `e`, `cvc5`, `z3`。各独立リストを完全な canonical JSON record text で整列し、重複数・空リスト・直接構築された任意の値を保持する。リスト順とリスト間の位置対応は identity に寄与しない。
厳密な UTF-8 preimage は `mizar-build/package-config-content/v1`、一つの LF、末尾が一つの LF の canonical root JSON text であり、既存 Session `hash_text` の source-text-domain/length/BLAKE3 構成へ渡す。これは Build content profile であり、raw hash primitive・Artifact hash class・Cache key domain・完全な verifier_config_hash producer・Proof policy fingerprint ではない。現在の全 field が参加し、将来の field/solver variant は profile 拡張の review を要する。既存 parser による省略 default と明示 default の実効内容は等しい。
独立した literal golden が format/frame を固定する。test は各 scalar・solver・u32 全値・曖昧 string・package 対応・独立リスト順・重複数・不一致/空リストを扱う。実 native manifest と clean parsed plan から得る metadata digest を genuine snapshot/config cache carrier に渡し、config-only 変更と実 protected output/NoKey を確認する。他の fixture context に真正性を認めず、registry default metadata は認証済 dependency 設定ではない。完全な active config/context、caller plan/capture の完全な出所・currentness 対応、policy/invocation/replay、toolchain/pin、runtime/cache/Proof 受理、native editor bootstrap は保留する。

## Planning Algorithm

1. `mizar.workspace` または `mizar.pkg` から workspace mode を発見する。
2. workspace manifest、各 member package manifest、workspace lockfile の TOML を parse する。
3. schema、package id、SemVer version、version constraint、enum value、config default、member path、build path を検証する。
4. package id を key にした package table を作り、重複を拒否する。
5. すべての workspace package に対応する lockfile entry があることと、すべての
   lockfile workspace-source entry が発見済み workspace package に対応することを
   検証する。
6. 各 manifest dependency を lockfile に照合し、exact resolved version を集め、request の dependency selection に従って active edge を印付ける。
7. active package dependency graph を作り、cycle を拒否する。
8. 対応 edition を検査し、各 package の effective verifier/build config を
   package plan と aggregate config record の両方へ運ぶ。
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

## 公開 enum policy

この module が所有する exhaustive public enum exception はない。

| Enum | Policy |
|---|---|
| `LockSource` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `DependencyKind` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `VersionConstraint` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `VersionComparison` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `Solver` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `DependencySelection` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `PackagePlanSource` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `ManifestDiagnosticKind` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |
| `ManifestValidationError` | `#[non_exhaustive]`; downstream callers は wildcard match arms を含めなければならない。 |

## Toolchain Constraint Boundary

planner は任意の package toolchain 制約、workspace/解決済み依存の共通制約、リリース版の厳密な lock 選択、W0306 の不一致方針を所有する (§23.1–23.2/§22.6.6)。
選択には認証された依存 manifest と release-version 入力が必要で、欠落時に版を推測しない。有効な lock 選択を保ち、installer はそれを消費する。
開発時の不一致は警告、release 時はエラー。選択した release が backend build を固定し、インストールと再配布は driver が所有する。
planner テストは省略と既存の制約形式、不正/競合制約、依存/release 入力欠落、pin 維持、利用不能な版を扱い、CLI テストは不一致の両モードと、実行中/lock の版が一致して W0306 やエラーを出さない場合を扱う。
先送りの境界であり、lock schema/API 形状や実行時の導入済み状態を定めない。

## Package toolchain constraint metadata

既存 PackageManifest と PackagePlan に `toolchain_constraint: Option<VersionConstraint>` を追加する。既存 package.toolchain だけを受け入れる。取得済 workspace manifest の省略は None、string は既存 parse_version_constraint の exact/caret/tilde/range・部分版規則へ渡す。read_optional_string を再利用し、不正 TOML 型は内部 InvalidType、不正 string は InvalidVersionConstraint を '<package manifest>', 'package.toolchain' に保持する。既存 diagnostic 集約/順序と error barrier により不正 manifest を成功として返さない。新しい公開 diagnostic・構文/default・版選択・実行中の版との比較は導入しない。
既存 validated-manifest clone と workspace ごとの PackagePlan は package 固有の型付き制約を保持し、package 間の merge/intersection や指定された BuildPlan.toolchain の置換をしない。registry-only plan の None は制約 metadata 未取得を示し、未認証 dependency に制約がない証拠にはならない。直接構築した値は既存 validation のみを受け、新しい制約 validator・dependency fetch・release producer・lock schema/hash profile・runtime context を加えない。
test は省略・既存全形式/full SemVer・不正 string/型/diagnostic 位置・validated/native-parser/plan copy・独立した逆向き workspace 制約/順序・registry metadata 不在、実 native input から genuine Driver が保持する plan/protected output/NoKey を扱う。既存 PackagePlan test constructor は意図を変えず None を追加する。native reader の旧 unknown package.toolchain fixture は同じ値の toolchain_extra へ替え、unknown-field/error-agreement control を保持する。unknown lock-toolchain test は変更しない。全 intersection/selection・認証済 dependency/release data・lock pin/W0306/backend identity・caller context/currentness・cache/Proof/editor 連携は前節の境界で保留する。

## Computation Policy Boundary

Build は default-false の boolean computation_replay と package-local manifest configuration (§23.1) を所有する。driver が Proof へ渡し、Proof が明示 invocation の優先順位と release/trust policy を所有する。configuration は snapshot/cache identity に参加し、dependency の default は参照元 package policy を置換しない。テストは省略/false/true/nonboolean、workspace package、config-only invalidation、direct/transitive call の逆向き明示 option を区別する。

metadata slice は既存 VerifierConfig に computation_replay: bool を保持し、false Default と read_optional_bool を使う。既存 package/順序付き workspace config のコピーは参照元 package の値を保つ。verifier table の省略/空、field の省略/false/true/nonboolean、逆値の package と入力順、config のみの typed 差分、実 native acquisition/Driver retained plan をテストする。この slice は完全な config digest、invocation 優先順位、replay 実行、proof/cache 受理を供給せず、統合 producer は依存待ちとする。
