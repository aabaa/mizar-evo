# mizar-artifact VerifiedArtifact Schema

> 正本は英語です。英語版:
> [../en/verified_artifact.md](../en/verified_artifact.md)。

`VerifiedArtifact` は、1 つの verified source file に対する stable published
projection である。downstream package、LSP feature、documentation generation、
AI tooling が compiler-internal IR を読まずに source-shaped な verified metadata を
必要とするときに読む主要 artifact である。

task 10 がこの仕様を導入し、task 11 が schema、projection-input contract、
writer、validating reader、test を実装し、task 15 が provenance envelope と
local verified-artifact metadata 用の store-level artifact-hash exclusion を確定する。

## Ownership

verified-artifact schema が所有するもの:

- verified source file に対する stable top-level artifact shape。
- stale-artifact diagnostic と manifest validation に使う source identity、
  source hash、module identity。
- earlier phase から射影された dependency-facing export と local expression
  metadata。
- projected data としての verification obligation metadata と accepted proof status。
- external `ProofWitnessRef` value への参照。witness payload byte は含めない。
- downstream tooling 向けの stable projected diagnostic。
- artifact projection の compatibility と hash participation rule。

所有しないもの:

- raw AST arena、typed AST、core IR、control-flow IR、VC IR、ATP problem、kernel
  state。
- proof search、proof acceptance、kernel replay、verifier policy decision。
- proof witness payload byte schema や witness file I/O。
- internal cache record、cache-key lookup、proof-reuse validation、scheduler state。
- manifest transaction、atomic write、artifact-store file I/O。

schema family は `mizar-artifact/verified-artifact` である。version `1.0` が
task 11 の初期 version である。task 11 reader は `1.0..=1.0` の範囲を support し、
後続仕様が forward-compatible minor-version handling を明示的に宣言しない限り、missing、
malformed、異なる major、新しすぎる minor version を拒否する。

## Top-Level Shape

task 11 は次の canonical JSON field shape を使う。

```text
verified_artifact = {
  "schema_version": "1.0",
  "module": module,
  "source_file": string,
  "source_hash": source_hash_string,
  "verified_at": string | null,
  "interface_hash": interface_hash_string,
  "implementation_hash": implementation_hash_string,
  "exports": [verified_export, ...],
  "expressions": [expression_metadata, ...],
  "obligations": [obligation_metadata, ...],
  "proof_witnesses": [proof_witness_ref, ...],
  "diagnostics": [artifact_diagnostic, ...],
  "provenance": build_provenance
}
```

`module` field は [module_summary.md](./module_summary.md) が定義する identity
shape と同じものを使う。`source_file` は正規化された package-relative または
workspace-relative path である。absolute path であってはならず、正規化後に package
root から escape してはならない。task 11 は `CanonicalJson` boundary で lexical な
portable path shape を検証する。filesystem escape check は store I/O の作業として残る。

`source_hash` は artifact の生成に使った厳密な source text を記録する。stale-artifact
diagnostic と manifest consistency check に使われる。これは module identity ではない。

`verified_at` は optional な user-visible local metadata である。存在する場合、`Z` と
whole-second precision を使う RFC 3339 UTC timestamp である。例:
`2026-06-22T14:03:05Z`。reader は `Z` 以外の timezone offset、subsecond precision、
malformed date、空文字列を拒否する。`verified_at` は `interface_hash`、`implementation_hash`、
artifact publication equivalence、reproducibility comparison から除外される。reader はこれを
proof result の受理、dependency validation、publication eligibility の判断に使ってはならない。

ここに列挙した field はすべて必須である。存在しない値に意味がある field は JSON `null`
で符号化する。reader は必須 field の省略と、すべての schema object の unknown field を拒否する。

`source_range` は `ModuleSummary` と同じ byte-offset shape を使う。

```text
source_range = {
  "start_byte": non_negative_integer,
  "end_byte": non_negative_integer
}
```

reader は start が end より大きい range を拒否する。

## Exports

`exports` は verified source file から射影された externally visible declaration と
signature を含む。各 entry は、downstream package と tool が raw compiler IR を調べずに
export を識別、表示、依存できるだけの stable metadata を記録する。

task 11 は次の canonical JSON field shape を使う。

```text
verified_export = {
  "origin_id": string,
  "fully_qualified_name": string,
  "namespace_path": [string, ...],
  "visibility": "public" | "reexported",
  "export_kind": string,
  "source_range": source_range,
  "rendered_signature": string,
  "interface_fingerprint": interface_hash_string,
  "proof_status": "accepted" | "not_accepted" | "not_required" | null,
  "documentation_ref": diagnostic_hash_string | null
}
```

`exports` が `ModuleSummary` より広いのは local artifact detail に限られる。implementation
body、proof body、raw type-checker fact、raw resolution table、kernel proof state を含んではならない。
export が importer に visible である場合、その dependency-facing interpretation は対応する
`ModuleSummary` projection と一致しなければならない。

`proof_status` は proof-producing phase から射影される。`VerifiedArtifact` は status を記録するが、
proof が受理されたかを決定しない。

## Expression Metadata

`expressions` は IDE、documentation、AI tooling 向けの stable で source-shaped な metadata を含む。
これは意図的に projection であり、serialized `TypedAst` や `ResolvedTypedAst` ではない。

task 11 は次の canonical JSON field shape を使う。

```text
expression_metadata = {
  "expression_id": string,
  "source_range": source_range,
  "expression_kind": string,
  "rendered_surface": string,
  "inferred_type": string | null,
  "resolved_symbol": string | null,
  "inserted_coercions": [string, ...],
  "active_thesis": string | null,
  "overload_resolution": overload_metadata | null
}

overload_metadata = {
  "root_symbol": string,
  "selected_candidate": string,
  "active_refinements": [string, ...],
  "coercion_summary": string | null
}
```

type、symbol、thesis、coercion、overload field は stable rendered summary または
producer-owned fingerprint である。arena index、debug formatter output、raw type-table row、
raw AST node、checker-local object identity を公開してはならない。

`expression_id` は同一 source、dependency graph、toolchain、verifier setting に対して決定的である。
proof evidence ではなく、cross-edit proof-reuse identity でもない。

## Obligations And Witness References

`obligations` は VC/proof phase から射影された verification obligation を記録する。source navigation、
display、publication status を保持するが、VC IR、ATP problem、proof certificate は埋め込まない。

task 11 は次の canonical JSON field shape を使う。

```text
obligation_metadata = {
  "obligation_id": string,
  "obligation_anchor": string | null,
  "owner_origin_id": string | null,
  "source_range": source_range,
  "obligation_kind": string,
  "statement_summary": string,
  "obligation_fingerprint": interface_hash_string,
  "vc_fingerprint": interface_hash_string,
  "local_context_fingerprint": interface_hash_string,
  "dependency_slice_fingerprint": interface_hash_string,
  "verifier_policy_fingerprint": interface_hash_string,
  "status": "accepted" | "open" | "rejected" | "externally_attested" | "not_required",
  "accepted_witness_obligation_id": string | null,
  "deterministic_discharge_hash": interface_hash_string | null,
  "diagnostic_ref": diagnostic_hash_string | null
}
```

`obligation_id` は `ProofWitnessRef` が使う stable id である。`obligation_anchor` は diagnostic、
repair、cache candidate matching のための best-effort cross-edit identity である。proof evidence
ではなく、reuse に十分でもなく、kernel に trust されない。

`obligation_fingerprint` は producer-owned な composite proof-reuse input
fingerprint である。VC semantic fingerprint、local proof context fingerprint、
dependency-slice fingerprint、verifier-policy fingerprint、および既存 witness を再利用できるかに影響する
producer-owned evidence requirement に commit する。`VerifiedArtifact` はこの field の hash
class と spelling を検証し、witness consistency に使う。real producer integration が入るまでは、
composite value の構築責務は VC/proof producer に残る。

`status = "accepted"` の obligation はすべて `accepted_witness_obligation_id` を自分自身の
`obligation_id` と同じ文字列に設定しなければならない。その id は `proof_witnesses` 内のちょうど
1 つの `ProofWitnessRef.obligation_id` に解決されなければならない。これは ATP certificate witness と、
受理された built-in または kernel-primitive discharge の両方を含む。task 9 はそれらの accepted
evidence class を `ProofWitnessRef` value として表現するためである。

task 11 は accepted-witness consistency tuple を検証する。

- witness の `obligation_id` は obligation の `obligation_id` と等しい。
- witness の `obligation_fingerprint` は obligation の `obligation_fingerprint` と等しい。
- witness の `kernel_acceptance.verifier_policy_fingerprint` は obligation の
  `verifier_policy_fingerprint` と等しい。
- obligation の `vc_fingerprint`、`local_context_fingerprint`、
  `dependency_slice_fingerprint`、`verifier_policy_fingerprint` は obligation
  metadata 内の interface fingerprint として存在し、composite `obligation_fingerprint` contract と
  `implementation_hash` participation に含まれる。
- witness の `kernel_acceptance.accepted_result_hash` は proof/kernel producer が供給する
  interface hash であり、参照された `ProofWitnessRef` を通じて含まれる。

`VerifiedArtifact` が検証するのは projected field 同士の reference consistency だけである。
witness replay、kernel acceptance の再計算、proof authority の決定は行わない。

`status = "not_required"` だけが trusted no-witness case である。active language と verifier rule の下で
proof obligation を持たない item に使う。この entry は `accepted_witness_obligation_id = null` を使わなければならない。
producer が deterministic discharge fingerprint を供給する場合は `deterministic_discharge_hash` に記録し、
class `interface` を使う。

open、rejected、externally attested な obligation は `accepted_witness_obligation_id = null` かつ
`deterministic_discharge_hash = null` を使わなければならない。externally attested evidence を
development workflow のために保持する場合は、別の diagnostic または development schema を使わなければならず、
kernel-checked witness を要求する release policy を満たさない。

`proof_witnesses` collection は task 9 の `ProofWitnessRef` object を含む。ordering と duplicate-obligation
rule は [proof_witness.md](./proof_witness.md) に従う。すなわち obligation id、fingerprint、
status、evidence kind、path、artifact hash で sort し、後続 schema が単一 obligation に複数の
accepted witness を明示的に support しない限り、1 つの verified artifact 内の duplicate
`obligation_id` を拒否する。

`VerifiedArtifact` は witness payload を load せず inline もしない。consumer は replay、audit、
diagnosis、hash validation を明示的に要求した場合だけ、artifact-store と manifest reader を通じて
witness file を load する。

## Diagnostics

`diagnostics` は completed build pass が発行した stable projected diagnostic を含む。diagnostic は
artifact metadata であり、proof authority ではない。

task 11 は次の canonical JSON field shape を使う。

```text
artifact_diagnostic = {
  "diagnostic_id": string,
  "code": string,
  "severity": "error" | "warning" | "info" | "hint",
  "primary_range": source_range | null,
  "message_key": string,
  "rendered_message": string,
  "related": [diagnostic_related, ...],
  "explanation_ref": diagnostic_hash_string | null
}

diagnostic_related = {
  "source_range": source_range,
  "message_key": string,
  "rendered_message": string
}
```

`code`、`severity`、ordering key、`message_key` は stable である。wording の改善により
rendered message は変わりうる。stable behavior を必要とする consumer は diagnostic code と
structured field を key にしなければならない。diagnostic は raw debug dump field name、raw IR node
index、proof/kernel-internal state を公開してはならない。

## Provenance Envelope

`provenance` は reproducibility と debugging metadata を記録する。consumer が artifact の生成方法を
理解する助けになるが、dependency artifact、source hash、verifier policy fingerprint、proof witness
hash の validation の代替ではない。

task 15 は published verified artifact 向けの、この crate が所有する provenance envelope を確定する。

```text
build_provenance = {
  "toolchain": string,
  "language_edition": string,
  "lockfile_hash": artifact_hash_string,
  "verifier_config_hash": interface_hash_string,
  "dependency_artifact_hashes": [dependency_artifact_hash, ...],
  "cache_key": string | null
}

dependency_artifact_hash = {
  "module": module,
  "interface_hash": interface_hash_string,
  "implementation_hash": implementation_hash_string | null,
  "artifact_hash": artifact_hash_string | null
}
```

`verifier_config_hash` は verifier policy と configuration の安定した producer-owned settings
fingerprint を記録する。cache key は opaque な producer-owned fingerprint である。
`mizar-artifact` は producer が供給した場合にだけ記録する。cache lookup、cache invalidation、
proof-reuse validation は `mizar-cache` が所有し続ける。`cache_key` は hash-excluded な
local/cache metadata である。空でない string または JSON `null` として parse されるが、
`interface_hash`、`implementation_hash`、artifact publication equivalence、reproducibility
comparison には参加しない。

## Hash String Domains

`source_hash` は `ModuleSummary` と同じ exact source text hash string を使う。

```text
source_hash_string =
  "mizar-session/hash-text/v1:" lower_hex_32_byte_digest
```

この schema のその他すべての hash field は task 3 の artifact-framed hash string 形式を使う。

```text
mizar-artifact/artifact-framed-hash-text/v1:<class>:<schema_family>:<schema_version>:<digest>
```

`<digest>` は 64 文字の lowercase hexadecimal digest である。artifact-framed な
`schema_family` string は `registration_summary.md` の grammar に従う。すなわち、
空でない、colon を含まない、slash 区切りの識別子であり、各 segment は ASCII letter、
ASCII digit、hyphen、underscore、dot だけを含む。

task 11 は次の domain を検証する。

| Field | Required construction/class | Required schema family/version | Notes |
|---|---|---|---|
| `source_hash` | `mizar-session/hash-text/v1` | なし | 厳密な source text hash。 |
| `interface_hash` | artifact-framed `interface` | `mizar-artifact/verified-artifact`、artifact の `schema_version` | stored dependency-facing artifact hash。reader は再計算して比較する。 |
| `implementation_hash` | artifact-framed `implementation` | `mizar-artifact/verified-artifact`、artifact の `schema_version` | stored full stable projection hash。reader は再計算して比較する。 |
| `verified_export.interface_fingerprint` | artifact-framed `interface` | producer-owned、valid grammar | export fingerprint。 |
| `verified_export.documentation_ref` | artifact-framed `diagnostic` | producer-owned、valid grammar | optional documentation/diagnostic payload reference。 |
| `obligation_metadata.obligation_fingerprint` | artifact-framed `interface` | producer-owned、valid grammar | composite proof-reuse input fingerprint。 |
| `obligation_metadata.vc_fingerprint` | artifact-framed `interface` | producer-owned、valid grammar | VC semantic fingerprint。 |
| `obligation_metadata.local_context_fingerprint` | artifact-framed `interface` | producer-owned、valid grammar | local proof context fingerprint。 |
| `obligation_metadata.dependency_slice_fingerprint` | artifact-framed `interface` | producer-owned、valid grammar | dependency slice fingerprint。 |
| `obligation_metadata.verifier_policy_fingerprint` | artifact-framed `interface` | producer-owned、valid grammar | verifier policy fingerprint。 |
| `obligation_metadata.deterministic_discharge_hash` | artifact-framed `interface` | producer-owned、valid grammar | `not_required` 用の optional no-witness discharge fingerprint。 |
| `obligation_metadata.diagnostic_ref` | artifact-framed `diagnostic` | producer-owned、valid grammar | optional diagnostic/explanation payload reference。 |
| `artifact_diagnostic.explanation_ref` | artifact-framed `diagnostic` | producer-owned、valid grammar | optional structured explanation payload reference。 |
| `provenance.lockfile_hash` | artifact-framed `artifact` | producer-owned、valid grammar | lockfile byte/projection hash。 |
| `provenance.verifier_config_hash` | artifact-framed `interface` | producer-owned、valid grammar | verifier configuration fingerprint。 |
| `dependency_artifact_hash.interface_hash` | artifact-framed `interface` | dependency-owned、valid grammar | dependency-facing module hash。 |
| `dependency_artifact_hash.implementation_hash` | artifact-framed `implementation` | dependency-owned、valid grammar | optional dependency implementation hash。 |
| `dependency_artifact_hash.artifact_hash` | artifact-framed `artifact` | dependency-owned、valid grammar | optional dependency artifact file hash。 |

top-level `interface_hash` と `implementation_hash` だけが、この schema によって schema family と
version を固定される hash field である。producer-owned と dependency-owned の hash reference は、
別 crate の domain を書き換えずに class と spelling を検証できるよう、それぞれの schema family と
version を保持する。

`proof_witnesses` 内の `ProofWitnessRef` value は
[proof_witness.md](./proof_witness.md) の hash-domain rule を保持する。

## Hash Participation

task 11 は filtered canonical JSON projection から両方の top-level hash を計算し検証する。task 16 は
これらの projection builder を reusable helper として公開してよいが、task 11 の participation rule は変更しない。

`interface_hash` は次の importer-visible field だけに対して計算される。

- `schema_version`。
- `module`。
- 各 export の `origin_id`、`fully_qualified_name`、`namespace_path`、
  `visibility`、`export_kind`、`rendered_signature`、
  `interface_fingerprint`、`proof_status`。
- 各 dependency entry の `module` と `interface_hash`。

`interface_hash` は `source_file`、`source_hash`、`verified_at`、stored
`interface_hash` と `implementation_hash`、export `source_range`、export
`documentation_ref`、expression metadata、obligation、proof witness path と byte hash、
diagnostic、`lockfile_hash`、`verifier_config_hash`、dependency `implementation_hash`、
dependency `artifact_hash`、`toolchain`、`language_edition`、`cache_key`、将来の
local/cache-only provenance field を除外する。

`implementation_hash` は次の stable published field だけに対して計算される。

- `schema_version`、`module`、`source_file`、`source_hash`。
- `source_range` と `documentation_ref` を含む full `exports`。
- full `expressions`。
- `obligation_fingerprint` とその component fingerprint を含む full `obligations`。
- full `proof_witnesses`。
- diagnostic range と related entry を含む full `diagnostics`。
- stable provenance field: `toolchain`、`language_edition`、`lockfile_hash`、
  `verifier_config_hash`、full `dependency_artifact_hashes` entry。

`implementation_hash` は `verified_at`、stored `interface_hash` と `implementation_hash`
field 自身、`cache_key`、将来の local/cache-only provenance field だけを除外する。

どちらの hash も task 3 の artifact-framed hash string と domain-separated hash class を使う。
manifest の artifact hash は store-level publication-equivalent content、すなわち宣言済み
hash exclusion 後の canonical JSON を検証する。raw filesystem byte は検証しない。

## Canonical Ordering

writer は serialization 前に collection を決定的に sort する。reader は unsorted collection と
duplicate identity key を拒否する。

nullable range は `null` を non-null range より前に sort する。non-null range は
`start_byte`、次に `end_byte` で sort する。

初期 ordering key は次の通りである。

- `exports`: `origin_id`、`fully_qualified_name`、`export_kind`、`source_range`。
- `expressions`: `expression_id`、`source_range`。
- `obligations`: `obligation_id`、`source_range`。
- `proof_witnesses`: `proof_witness.md` が定める順序。
- `diagnostics`: `diagnostic_id`、`code`、`primary_range`。
- `diagnostic.related`: `source_range`、`message_key`、`rendered_message`。
- `provenance.dependency_artifact_hashes`: module identity。

初期 duplicate identity key は次の通りである。

- `exports`: `origin_id`。
- `expressions`: `expression_id`。
- `obligations`: `obligation_id`。
- `proof_witnesses`: `proof_witness.md` が定める `obligation_id`。
- `diagnostics`: `diagnostic_id`。
- `diagnostic.related`: `source_range`、`message_key`、`rendered_message`。
- `provenance.dependency_artifact_hashes`: module identity。

source traversal order、hash map iteration order、ATP completion order、diagnostic emission race timing、
filesystem order は serialized byte に影響してはならない。

reader は列挙した各 collection boundary で duplicate identity key を拒否する。

## Reader And Writer Requirements

task 11 writer は [store.md](./store.md) の正準 UTF-8 JSON rule を使い、current schema version を
emit する。reader は:

- JSON-null field を含め、上で列挙した field をすべて必須とする。
- すべての schema object で unknown field を拒否する。
- field を解釈する前に schema-version compatibility を検査する。
- 空の required string、malformed source range、invalid path shape、malformed hash string、
  wrong hash class、duplicate identity、unsorted collection を拒否する。
- 上記の filtered projection から `interface_hash` と `implementation_hash` を再計算し、mismatch を拒否する。
- trusted witness が必要な場合、`accepted_witness_obligation_id` が matching
  `ProofWitnessRef.obligation_id` に解決され、accepted witness id が containing obligation id と
  等しく、witness の `obligation_fingerprint` が obligation の `obligation_fingerprint` と
  等しいことを検証する。
- accepted obligation はすべてちょうど 1 つの matching trusted witness を指すことを要求する。
- open、rejected、externally attested、not-required obligation に対する trusted witness reference を拒否し、
  deterministic discharge hash は not-required obligation にだけ許可する。
- `verified_at` と将来の local-only field を hash-excluded のままにする。
- artifact schema の検証中に raw IR、internal cache record、ATP log、witness payload、kernel state を読まない。

reader failure は artifact diagnostic である。proof authority を確立せず、internal cache record へ
黙って fallback せず、externally attested evidence を kernel-verified evidence に upgrade しない。

## Deferred Implementation

task 10 はこの仕様だけを追加する。task 11 は `VerifiedArtifact` schema、projection-input contract、
writer、validating reader、および round-trip、raw-IR-shaped payload rejection、stable diagnostic
ordering、schema-version compatibility、source-range validation、hash class/domain validation、
witness-reference consistency、hash participation、ownership-boundary field rejection の test を実装する。

task 15 は provenance record と、local verified-artifact field の store-level artifact-hash exclusion
helper を確定する。real producer integration は、resolver、checker、VC、proof、kernel crate が stable
projection input を公開するまで `external_dependency_gap` として残る。interface と implementation hash
input helper は task 16 に deferred。manifest/file I/O と atomic publication は task 13 と 14 で完了済み。
