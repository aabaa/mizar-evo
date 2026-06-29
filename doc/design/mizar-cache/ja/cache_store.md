# Module: cache_store

> 正本は英語です。英語版: [../en/cache_store.md](../en/cache_store.md)。

Status: task 7 で仕様化。task 8 は inline record を実装し、task 9 は
content-addressed blob-backed record を実装する。

## Purpose

`cache_store` は `mizar-cache` の内部 cache record 永続化を所有する。

cache record は optimization artifact である。hit は、exact-key lookup、
header compatibility check、dependency footprint validation、proof-reuse
validation、output hash verification がすべて成功した後にだけ、後続 build の
作業を skip できる。cache record は proof authority ではなく、cache record、
externally attested evidence、backend diagnostic、backend log、timing metadata、
cache hit/miss state を kernel-verified status や trusted `used_axioms` に昇格しては
ならない。

`.mizar-cache/` またはその中の record を削除しても、source-level semantics、
published package compatibility、proof acceptance、artifact publication は変わっては
ならない。削除が引き起こせるのは、後続 build がより多くの作業を行うことだけである。

## Public API

task 8 と task 9 は小さな record/blob store API を公開する。実際の Rust 名は newtype を
使ってよいが、下の semantic decision は test と audit code から見える形を保つ。

```rust
pub const CACHE_RECORD_SCHEMA_VERSION: &str;
pub const CACHE_RECORD_MAGIC: &[u8];
pub const CACHE_BLOB_HASH_FAMILY: &str;

pub struct CacheStoreRoot { ... }
pub struct CacheRecordHeader { ... }
pub struct CacheRecord { ... }
pub struct CacheOutputDescriptor { ... }
pub struct CacheBlobRef { ... }

pub enum CacheLookupOutcome {
    Hit(Box<CacheRecord>),
    Miss(CacheMiss),
}

pub enum CacheInsertOutcome {
    Inserted,
    AlreadyPresent,
    RejectedUncacheable,
}
```

lookup は incompatible、unknown、incomplete、uncacheable、corrupt な record を
既定では miss として返さなければならない。明示的な cache-audit mode は corruption を
diagnostic にしてよいが、その record を hit にしてはならない。

## Store Layout

default root は設定された `cache_dir`、通常は `.mizar-cache/` である:

```text
.mizar-cache/
  records/
    <phase>/
      <cache-key-final-hash>.mcr
  blobs/
    <hash-family>/
      <digest>
  tmp/
  quarantine/
```

lookup path が exact phase と `CacheKey.final_hash` の deterministic function のままで
あれば、実装は digest prefix によって `records/` または `blobs/` を shard してよい。
directory order、file modification time、temporary file name、writer process id、
record arrival order は reuse input ではない。

`cluster-db/`、import-scoped view、resolution trace、diagnostic explanation backing
data は別の cache surface である。この文書は phase record と blob storage だけを
仕様化する。cluster-db indexing や build integration は実装しない。

## Record Identity

record path は以下から導く:

- `CacheKey.phase`;
- `CacheKey.final_hash`;
- cache record schema の path encoding version。

path は trust boundary ではない。candidate file を開いた後、lookup は header から
compatibility を再計算し、embedded key が requested key と一致することを検証しなければ
ならない。期待 path にある file が別の key、phase、work unit、schema、output hash を
持つ場合は miss であり、cache integrity diagnostic の候補になる。

## CacheRecordHeader

`CacheRecordHeader` は、exact-key path lookup 後に candidate を検証するために必要な値を
すべて記録する。task 8 は compatibility と dependency validation field の多くを
embedded `key.validation_inputs` の中に保存する。下の top-level row のうち "embedded" と
書かれているものは、独立した header field として重複させず、その key 経由で copy される。

| Field | Meaning |
|---|---|
| `cache_record_schema_version` | on-disk record encoding と compatibility version。unknown version は miss。 |
| `cache_key_schema_version` | embedded `CacheKey` の schema version。unknown または mismatch は miss。 |
| `key` | この record を生成した canonical `CacheKey`。display hash だけでなく構造的に比較する。 |
| `key_hash` | path と header diagnostic 用の `CacheKey.final_hash` の copy。 |
| `phase` | quick rejection と diagnostic 用に key から copy した pipeline phase。 |
| `work_unit` | quick rejection と diagnostic 用に key から copy した phase-local unit。 |
| `produced_by` | reuse check に使う toolchain identity と compatibility field。unknown compatibility は miss。 |
| `policy_fingerprint` | `key` に embedded。key から copy した verifier/proof policy fingerprint。incompatible policy は miss。 |
| `schema_versions` | `key` に embedded。解釈に影響する output、artifact、footprint、proof-reuse metadata、record schema。required schema が unknown なら miss。 |
| `dependency_footprint_hash` | `key.validation_inputs` に embedded。output が使った reusable dependency footprint hash。`Complete` と `ConservativeComplete` は reusable、`IncompleteUncacheable`、missing、unsupported footprint は miss。 |
| `dependencies` | `key.validation_inputs` に embedded。reuse 前に必要な dependency artifact availability と記録された domain/digest check。missing または changed artifact は miss。publication-token validation は外部のまま。 |
| `proof_reuse` | `key.validation_inputs` に embedded。任意の、`mizar-proof` が export した proof-reuse validation metadata。validation data でしかない。 |
| `output` | inline または blob-backed output bytes と hash を記述する `CacheOutputDescriptor`。 |
| `uncacheable` | lookup と insert に miss/reject を強制する明示 marker。 |
| `diagnostic_refs` | miss 説明用の任意の diagnostic-only reference。proof acceptance には影響しない。 |

header は wall-clock hit/miss timing、filesystem freshness、temporary path、record write
order、backend runtime duration、process id、thread id を compatibility input として
含んではならない。

## Record Encoding

record store は単一の binary record envelope を使う:

```text
magic bytes
record format version
header length
canonical UTF-8 JSON header
payload length
payload bytes
```

JSON header は canonical である。object key は sort し、vector は `cache_key.md` と
`dependency_fingerprint.md` が定める canonical key で sort し、duplicate identity key は
write 前に reject する。payload は以下のいずれかである:

- `CacheOutputDescriptor` が content-addressed blob を指す場合は empty;
- 小さな output の場合は inline bytes。

output hash は record file path や record envelope ではなく、canonical output bytes に対して
計算する。key hash mismatch、payload length mismatch、malformed canonical JSON、
duplicate header key、unsupported enum variant、malformed blob descriptor、missing blob、
output hash mismatch は miss である。

## Blob Store

大きな output は以下に content-addressed に保存する:

```text
.mizar-cache/blobs/<hash-family>/<digest>
```

task 9 の hash family は `blake3` である。その digest は
`CacheOutputDescriptor.output_hash` に記録されるものと同じ domain-separated output hash の
lowercase hexadecimal encoding であり、現時点では canonical output bytes に対して
`mizar-cache/cache-record-output/v1` domain で frame した BLAKE3 hash である。reader は
unknown hash family、lowercase hex でない digest、長さが違う digest、path-like digest spelling を
blob read path を構築する前に reject しなければならない。

blob write は atomic である: `tmp/` に書き、flush し、digest を検証し、同一 filesystem の
hard link など create-new/no-overwrite semantics で final digest path に publish する。同一 bytes に
対する concurrent writer は同じ final file へ収束しなければならない。異なる bytes に対する
concurrent writer は digest check が成功しない限り同じ digest を共有できないため、mismatch は
cache integrity miss になる。

blob reference は内部用である。published artifact は cache blob を読まなくても読めなければ
ならない。

## Lookup

lookup は fail-closed である:

1. `CacheKeyBuildOutcome::NoKey` と `CacheKeyBuildOutcome::Uncacheable` は disk lookup 前に
   miss として reject する。
2. phase と `CacheKey.final_hash` から candidate path を導く。
3. candidate record が存在すれば読む。
4. record envelope と canonical header を decode する。
5. embedded key を requested key と構造的に比較する。
6. cache record schema、cache key schema、toolchain compatibility、verifier policy
   compatibility、output schema compatibility、proof-reuse metadata schema compatibility を
   check する。
7. reusable dependency footprint status、dependency artifact availability、記録された
   dependency artifact domain/digest、dependency hash、dependency-slice fingerprint を検証する。
8. proof/VC-related record では、key が要求する `mizar-proof` proof-reuse metadata、
   `ObligationAnchor`、canonical VC fingerprint、local-context fingerprint、
   dependency-slice fingerprint、selected proof witness hash、または deterministic discharge hash を
   検証する。proof phase と ATP phase の record は、VC metadata がすでに存在する場合でも
   proof-reuse evidence metadata を要求する。
9. inline bytes または blob bytes を読み、`output_hash` を検証する。
10. すべての check が成功した場合だけ `Hit` を返す。

失敗した check はすべて `Miss` を返す。unknown schema、unknown toolchain compatibility、
incomplete または unsupported dependency footprint、missing dependency artifact、
dependency artifact domain/digest mismatch、incomplete proof-reuse validation input、
unsupported proof evidence kind、明示 `uncacheable`、missing blob、output hash mismatch は
reusable data と解釈してはならない。

## Insert

insert は complete cacheable record だけを受け入れる:

- key outcome が `Cacheable`;
- `uncacheable` が false;
- dependency footprint が `Complete` または `ConservativeComplete`;
- compatibility field が known かつ supported;
- proof reuse が proof または ATP recomputation skip を起こす場合は proof-reuse
  validation metadata が存在し、VC reuse が recomputation skip を起こす場合は VC
  validation metadata が存在する;
- output hash が output bytes または blob bytes と一致する。

`Uncacheable` key または record は reusable record として insert しない。実装は diagnostic-only
miss accounting を別の場所に出力してよいが、その data は reusable `records/<phase>/`
namespace に置いてはならず、決して `Hit` を返してはならない。

write は `tmp/` の temporary file を使い、complete encoded record を検証し、flush してから
create-new/no-overwrite semantics で公開する。実装は、flush 済み temporary file から final
record path への same-filesystem hard link を作り、その後 temporary file を削除してよい。同じ key
に対して競合する 2 writer は、byte-identical record を公開するか、片方が observable semantics を
変えずに負けなければならない。同じ validated key に対する divergent content は cache integrity miss
であり、競合する proof outcome ではない。

## Miss Reasons

miss reason は cache behavior の diagnostic である。proof status ではない。

| Condition | Required outcome |
|---|---|
| missing record | `Miss(NotFound)` |
| unknown record schema または key schema | `Miss(UnknownSchema)` |
| unknown または incompatible toolchain | `Miss(UnknownToolchain)` または `Miss(IncompatibleToolchain)` |
| incomplete dependency footprint | `Miss(IncompleteFootprint)` |
| unsupported dependency footprint completeness または schema | `Miss(UnsupportedFootprint)` |
| 明示 `uncacheable` marker | `Miss(Uncacheable)` |
| dependency artifact missing または hash mismatch | `Miss(DependencyUnavailable)` |
| policy incompatibility | `Miss(PolicyIncompatible)` |
| proof-reuse validation missing、mismatched、externally attested only、または unsupported | `Miss(ProofReuseInvalid)` |
| malformed envelope、malformed canonical JSON、duplicate header key、missing blob、または output hash mismatch | `Miss(CorruptRecord)` |

verbose mode はこれらの reason を build diagnostic に添付してよい。通常の build semantics は
record が存在しない場合と同じように進む。

## Deletability

すべての cache record と blob は削除可能である。source、published artifact、verifier policy、
dependency manifest からの clean rebuild は、cache contents に関係なく同じ externally visible
result を生成しなければならない。record 削除は以下を変えられない:

- accepted proof status;
- trusted `used_axioms`;
- artifact publication eligibility;
- importer-visible module summary または cluster-db view;
- optional cache-debug diagnostic を除く diagnostic ordering。

record が missing blob を参照する場合、lookup は miss する。garbage collection は unreferenced blob
と stale temporary file を、artifact change を公開せずに削除してよい。

## Proof And Trust Boundary

`cache_store` は `mizar-proof` が export する proof-reuse validation metadata、accepted proof witness
hash、deterministic discharge hash、dependency-slice fingerprint を保存してよい。これらの field は
validation predicate でしかない。

この module は以下をしてはならない:

- `KernelCheckResult` を作る、または再解釈する;
- proof を kernel-verified と mark する;
- trusted `used_axioms` を project する;
- proof winner を選ぶ;
- artifact 用の proof status を選ぶ;
- externally attested evidence、backend log、backend diagnostic、cache record を trusted proof
  evidence として扱う。

trusted acceptance は、proof/status owner layer が消費し project する `mizar-kernel` result だけに
由来する。

## Tests

task 8 は少なくとも以下を cover する:

- inline output の record round-trip;
- exact-key path lookup でも embedded key と header を検証すること;
- incompatible record schema、key schema、toolchain、policy、output schema が miss を返すこと;
- missing または domain/digest mismatch した dependency artifact が miss を返すこと;
- 他の compatibility check がすべて成功する場合、`Complete` と
  `ConservativeComplete` footprint が reusable record を返すこと;
- `uncacheable`、incomplete footprint、unsupported footprint、missing validation input が
  miss を返すこと;
- corrupted envelope/header/payload record が diagnostic reason つきの miss を返すこと;
- output hash mismatch が miss を返すこと;
- record write order と arrival order が lookup result を変えないこと。

task 9 は少なくとも以下を cover する:

- content digest による blob round-trip;
- missing blob、malformed descriptor、hash-family mismatch、digest/output mismatch が
  miss を返すこと;
- concurrent identical writer が収束すること;
- 同じ digest に対する divergent writer が reject されること;
- blob 削除が build semantics を変えずに miss を起こすこと。

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `CACHESTORE-G001` | `external_dependency_gap` | `mizar-build` scheduler integration は未準備。この仕様は lookup/insert semantics だけを定義し、placeholder scheduling は追加しない。 |
| `CACHESTORE-G002` | `external_dependency_gap` | `mizar-ir` cache adapter は存在しない。record は opaque output bytes を持ってよいが、IR adapter API はここで作らない。 |
| `CACHESTORE-G003` | `external_dependency_gap` | artifact committed publication-token integration は外部所有。現在の cache store は local availability と記録された dependency artifact domain/digest だけを check する。cache record は artifact owner が token を公開した後にだけ published artifact hash に依存できる。 |
| `CACHESTORE-G004` | `deferred` | `cluster-db` index storage は後続 cache task。この record store 仕様は unaccepted registration を importer-visible にしない。 |

## Non-Goals

この module は cache key construction、dependency fingerprint computation、
proof-reuse policy validation、proof winner selection、proof status projection、ATP 実行、
kernel 呼び出し、artifact publication、cluster-db view update、build scheduling を行わない。
