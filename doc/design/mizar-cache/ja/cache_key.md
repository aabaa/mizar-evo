# Module: cache_key

> 正本は英語です。英語版: [../en/cache_key.md](../en/cache_key.md)。

Status: task 2 で仕様化。source implementation は task 3 から開始する。

## Purpose

`cache_key` は `mizar-cache` の内部 build cache key を所有する。

`CacheKey` は、要求された phase output の純粋な content identity である。
cached output の reuse 可否に影響しうる identity、hash、schema version、
dependency slice、policy fingerprint をすべて記録する。key construction は
trust decision ではない。key match は cache record を後続の compatibility check、
dependency-footprint validation、proof-reuse validation の candidate にするだけである。

Cache key は内部 optimization data である。proof authority ではなく、cache record、
externally attested evidence、backend diagnostic、backend log、timing metadata、
cache hit/miss state を kernel-verified status や trusted `used_axioms` へ昇格させては
ならない。

## Public API

task 3 は以下の概念 API を公開する。実際の Rust field wrapper は newtype にしてよいが、
semantic field は test と audit code から見える形を保つ。

```rust
pub const CACHE_KEY_SCHEMA_VERSION: &str;
pub const CACHE_KEY_HASH_DOMAIN: &str;

pub struct CacheKey {
    pub cache_schema_version: SchemaVersion,
    pub phase: PipelinePhase,
    pub work_unit: WorkUnit,
    pub source_identity: Option<SourceIdentity>,
    pub input_hashes: Vec<NamedHash>,
    pub dependency_hashes: Vec<DependencyHash>,
    pub dependency_slices: Vec<DependencySliceHash>,
    pub config_hash: Hash,
    pub schema_versions: Vec<NamedSchemaVersion>,
    pub policy_fingerprint: PolicyFingerprint,
    pub validation_inputs: CacheValidationInputs,
    pub final_hash: Hash,
}

pub struct CacheKeyBuilder { ... }

impl CacheKeyBuilder {
    pub fn new(request: CacheKeyRequest) -> Self;
    pub fn build(self) -> CacheKeyBuildOutcome;
}

pub enum CacheKeyBuildOutcome {
    Cacheable(CacheKey),
    Uncacheable(CacheKey),
    NoKey(CacheKeyBuildRejection),
}
```

`CacheKeyBuilder` は `CacheKeyRequest` から `CacheKeyBuildOutcome` への pure
projection である。mutable scheduler state、wall-clock time、cache contents、
record arrival order、write order、backend runtime、diagnostic、filesystem freshness
を読んではならない。すでに渡された policy と proof-reuse validation fingerprint 以外の
proof status を検査してはならない。

outcome は明示的であり、key construction は panic や impossible placeholder key なしで
fail closed できる:

- `Cacheable`: required field と validation input がすべて存在し、
  `uncacheable` marker が適用されない。
- `Uncacheable`: diagnostics と deterministic miss accounting 用の canonical key を
  持つが、cache lookup は必ず miss として扱う。
- `NoKey`: unknown cache-key schema、conflicting duplicate canonical key、または
  deterministic miss accounting すら不可能にする required identity 欠落など、request が
  structurally invalid または unsupported である。

## 公開 enum policy

この module が所有する exhaustive public enum exception はない。すべての public
enum は `#[non_exhaustive]` とする。downstream match は wildcard arm を持たなければならず、
新しい variant は後続の仕様 task が behavior を定義するまで fail closed しなければならない。

| Public enum | 前方互換性の決定 |
|---|---|
| `FootprintCompleteness` | `#[non_exhaustive]`; 新しい completeness state は明示的に support されるまで reusable ではない。 |
| `CacheKeyBuildOutcome` | `#[non_exhaustive]`; 新しい outcome は downstream cache user で miss または no-key result として扱わなければならない。 |
| `CacheKeyBuildRejection` | `#[non_exhaustive]`; 新しい rejection reason は diagnostic-only であり、cache hit にしてはならない。 |

## Data Structures

### CacheKey

| Field | Meaning |
|---|---|
| `cache_schema_version` | cache key と cache record compatibility rule の schema version。不明な version は miss。 |
| `phase` | frontend、resolve、checker、VC、ATP、proof、artifact、cluster-db など、要求 output の pipeline phase。 |
| `work_unit` | package、module、item、VC、obligation、cluster-view unit などの phase-local unit identity。 |
| `source_identity` | 任意の source package/module/path/hash/edition identity。package-global または dependency-only work unit では absent。 |
| `input_hashes` | source、IR、side-table、producer output など、この phase に影響する直接 input hash。 |
| `dependency_hashes` | 参照される published dependency artifact hash、manifest hash、interface hash、implementation hash、lockfile hash。 |
| `dependency_slices` | cached output が使った明示的 dependency-slice fingerprint。proof/VC reuse では VC、local-context、dependency-slice fingerprint を含める。 |
| `config_hash` | computation limit と cache-affecting build setting を含む verifier/build configuration hash。 |
| `schema_versions` | phase output、artifact、proof-reuse metadata、dependency footprint、backend encoding、canonical serialization など意味に影響する schema version。 |
| `policy_fingerprint` | active verifier/proof policy fingerprint。policy compatibility は後で check し、final hash だけから推論しない。 |
| `validation_inputs` | lookup 後に compatibility check と proof reuse が比較する architecture-22 validation field。 |
| `final_hash` | `final_hash` 自身を除く全 field の canonical encoding に対する domain-separated hash。 |

### SourceIdentity

`SourceIdentity` は以下を記録する:

- `package_id`;
- `module_path`;
- package または workspace policy に対する normalized source path;
- source content hash;
- language edition。

display path、absolute local path、open-buffer version number、source map allocation id、
diagnostics-only source origin、local filesystem metadata は除外する。

### Validation Inputs

`CacheValidationInputs` は、exact key match 後に cache lookup が検証する値を記録する:

- cache schema compatibility;
- producing toolchain compatibility;
- 全 dependency artifact hash と availability;
- complete dependency footprint status。すべての required dependency family を cover する
  conservative-complete footprint を含む;
- unsupported dependency footprint completeness または schema。これは miss のままであり、
  clean reuse と解釈してはならない;
- `uncacheable` marker state;
- verifier policy compatibility;
- output が proof/VC 関連の場合の canonical VC fingerprint;
- output が proof/VC 関連の場合の local-context fingerprint;
- dependency-slice fingerprint set;
- proof reuse が obligation-scoped の場合の `ObligationAnchor` fingerprint;
- recomputation skip として trusted に扱える proof reuse での selected proof witness hash
  または deterministic discharge hash;
- `mizar-proof` が export する proof-reuse metadata schema version と validation hash。

必要な validation input が欠けている場合、それは "no dependency" ではない。key または
record を uncacheable とし、miss を強制する。

## Canonical Ordering

`CacheKey` 内の vector はすべて hashing 前に coalesce し、sort する。
duplicate identity key は、同じ論理 input に対する複数値の衝突を検出する。
その後、canonical sort key が coalesce 済み entry を完全な semantic tuple で
順序付ける。

duplicate identity key は以下とする:

- `input_hashes`: `(name, domain)`;
- `dependency_hashes`: `(dependency_kind, package_id, module_path, name, domain)`;
- `dependency_slices`: `(slice_kind, owner, name, domain)`;
- `schema_versions`: `(schema_family, name)`;
- dependency artifact availability:
  `(package_id, module_path, artifact_kind, artifact_path, domain)`;
- dependency-slice validation fingerprint:
  `(slice_kind, owner, name, domain)`;
- policy/toolchain compatibility field: `(family, field_name)`;
- proof-reuse schema version: `(schema_family, name)`;
- proof-reuse evidence identity:
  `(obligation_anchor_fingerprint, evidence_kind, witness_or_discharge_domain)`;
- diagnostic-only ref: `(diagnostic_ref_kind, diagnostic_ref_hash)`。

canonical sort key は以下とする:

- `input_hashes`: `(name, domain, digest)`;
- `dependency_hashes`: `(dependency_kind, package_id, module_path, name, domain, digest)`;
- `dependency_slices`: `(slice_kind, owner, name, domain, digest)`;
- `schema_versions`: `(schema_family, name, version)`;
- validation-input collection: 対応する top-level field と同じ canonical key、
  または top-level field がない場合は下記の明示 key。

validation-input collection の ordering は以下とする:

- dependency artifact availability:
  `(package_id, module_path, artifact_kind, artifact_path, domain, digest)`;
- dependency-slice validation fingerprint:
  `(slice_kind, owner, name, domain, digest)`;
- policy compatibility field: `(policy_family, field_name)`;
- toolchain compatibility field: `(toolchain_component, field_name)`;
- proof-reuse schema version: `(schema_family, name, version)`;
- proof-reuse evidence identity:
  `(obligation_anchor_fingerprint, evidence_kind, witness_or_discharge_domain, witness_or_discharge_digest)`;
- miss explanation 専用の diagnostic-only ref:
  `(diagnostic_ref_kind, diagnostic_ref_hash)`。

同一 duplicate identity key かつ同一 payload の重複は coalesce する。同一
duplicate identity key で異なる payload の重複は invalid key request であり、
`CacheKey` を作る前に reject する。つまり、単一の論理 input 名が異なる digest
や version を黙って複数持つことはできない。順序依存の選択ではなく、fail-closed
な `NoKey` request になる。

ordering は `HashMap` iteration、worker completion order、cache record arrival order、
record write order、diagnostics order、ATP runtime、wall-clock time、process id、
thread id、temporary file name、filesystem directory order に依存してはならない。

## Stable Hashing

final key hash は明示 domain を使う:

```text
mizar-cache/cache-key/v1
```

canonical encoding は length-prefixed かつ typed である。各 field は field tag、field
value、必要なら schema/version tag を含める。Hash value は display string だけではなく、
domain と digest bytes を encode する。`config_hash`、`policy_fingerprint`、
VC/context fingerprint のような bare hash では、field-specific な cache-key tag を
hash domain とする。

`phase` と `work_unit` は常に hash に含める。したがって直接 input hash がすべて同一でも、
二つの phase が意味的に衝突することはない。`cache_schema_version`、`schema_versions`、
`policy_fingerprint` も compatibility と interpretation に影響するため hash input である。

`final_hash` は canonical key fields だけから導出する。以下は除外する:

- cache hit/miss timing;
- record write order;
- record file path;
- temporary file path;
- backend runtime duration;
- owning phase が semantic output input と明示した場合を除く backend stdout/stderr bytes;
- owning phase が semantic と明示した場合を除く diagnostic wording または explanation preview;
- 明示的に local diagnostic-only path として扱われる absolute local path。それらも cache-key
  hash からは除外する。

## Fail-Closed Rules

key construction と後続 cache reuse は fail closed でなければならない:

- unsupported または unknown cache key schema は no key または miss;
- unsupported または unknown cache record schema は miss;
- unknown toolchain compatibility は miss;
- incomplete dependency footprint は uncacheable かつ miss;
- conservative-complete dependency footprint は、他の compatibility と validation input が
  すべて一致する場合は cacheable のままである;
- unsupported dependency footprint completeness または schema は miss;
- 明示的な `uncacheable` marker は常に miss;
- dependency artifact hash の欠落は miss;
- policy incompatibility は miss;
- proof witness hash mismatch は miss;
- deterministic discharge hash mismatch は miss;
- proof-reuse metadata schema mismatch は miss。
- unknown proof-reuse evidence kind は miss。

`CacheKeyBuilder` は uncacheable work unit に対して
`CacheKeyBuildOutcome::Uncacheable` を生成してもよい。必要な validation input が
欠けている場合も同様である。ただし、その場合は uncacheable validation input も
設定しなければならない。そのような key は reusable hit を生成してはならない。
structurally invalid request、unsupported cache-key schema、conflicting duplicate は
`CacheKeyBuildOutcome::NoKey` を生成する。

## Proof-Reuse Boundary

Proof reuse は `cache_key` の所有ではない。この module は identity と validation input を
記録するだけである。task 11 が `mizar-proof` metadata を消費し、cached proof-related
output が recomputation を skip できるかを決める。

key は後続 validation のために十分な data を含めなければならない:

- `ObligationAnchor` fingerprint;
- canonical VC fingerprint;
- canonical local-context fingerprint;
- dependency-slice fingerprints;
- dependency artifact hashes;
- verifier policy fingerprint;
- proof-reuse metadata schema version;
- `KernelVerified` reuse の selected proof witness hash;
- `DischargedBuiltin` reuse の deterministic discharge hash。

non-trusted proof class、externally attested record、policy assumption、open/rejected outcome、
backend diagnostic、backend log、cache record は diagnostics または metadata として存在しても
よいが、key construction や cache lookup によって trusted acceptance になってはならない。

## Tests

task 3 は少なくとも以下を cover する:

- identical request に対する deterministic key;
- input order に依存しない canonical vector sorting;
- semantic field のどの変更も `final_hash` を変えること;
- 同一重複 entry の coalesce と conflicting duplicate の reject;
- phase と work-unit の domain separation;
- policy/schema/toolchain validation field が final key に参加すること;
- proof-related validation input が final key に参加すること;
- complete-footprint marker が欠けると `Uncacheable` または `NoKey` outcome を強制すること;
- timing、arrival order、write order、diagnostics-only field が除外されること;
- `CacheKeyBuilder` が cache contents や mutable scheduler state を読まないこと。

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `CACHEKEY2-G001` | `external_dependency_gap` | `mizar-build` は現在 cache-aware scheduler seam を所有するが、end-to-end scheduler hit/miss integration はここでは未接続である。この仕様は key だけを定義する。 |
| `CACHEKEY2-G002` | `external_dependency_gap` | `mizar-ir` は現在 cache-adapter validation boundary を所有する。この仕様は adapter の key input を定義するが、placeholder adapter API や rehydration shortcut は作らない。 |
| `CACHEKEY2-G003` | `external_dependency_gap` | artifact committed-publication token は artifact/proof owner 側に残る。cache key は witness/publication hash を validation input としてだけ保持してよい。 |

## Non-Goals

この module は cache lookup、cache record read/write、proof winner selection、proof status
projection、proof reuse validation、ATP 実行、kernel call、artifact publication、
scheduler readiness decision を行わない。
