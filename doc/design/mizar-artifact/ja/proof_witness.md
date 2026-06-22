# mizar-artifact ProofWitnessRef Schema

> 正本は英語です。英語版: [../en/proof_witness.md](../en/proof_witness.md)。

`ProofWitnessRef` は proof witness file の stable artifact projection である。
published verified artifact が witness payload を main artifact に埋め込まず、
path と hash で witness を指せるようにする。

この document は `ProofWitnessRef` schema を定義する。task 9 は
`CanonicalJson` boundary での schema、canonical value writer、validating
reader、test を実装する。

## Ownership

proof-witness reference schema が所有するもの:

- `VerifiedArtifact` の proof obligation が使う stable reference shape。
- package artifact root からの相対 witness path と path validation rule。
- witness file と proof-related fingerprint の artifact-framed hash。
- artifact reader と cache validator が必要とする projected kernel-acceptance metadata。
- witness payload を必要時だけ読む resident-set rule。

所有しないもの:

- ATP search、portfolio winner selection、backend certificate parsing。
- kernel replay、kernel acceptance、proof authority。
- witness payload file の byte schema。
- internal cache record や proof-reuse decision。
- manifest transaction や artifact-store file I/O。

reference の schema family は `mizar-artifact/proof-witness-ref` である。version
`1.0` が最初に support される version である。

## Conceptual Shape

task 9 は次の canonical JSON field shape を使う。

```text
proof_witness_ref = {
  "schema_version": "1.0",
  "obligation_id": string,
  "obligation_fingerprint": interface_hash_string,
  "proof_status": "kernel_verified" | "discharged_builtin",
  "evidence_kind": "atp_certificate" | "builtin_certificate" | "kernel_primitive",
  "witness_path": string,
  "witness_artifact_hash": artifact_hash_string,
  "kernel_acceptance": kernel_acceptance_metadata
}

kernel_acceptance_metadata = {
  "kernel_profile_fingerprint": interface_hash_string,
  "verifier_policy_fingerprint": interface_hash_string,
  "checker_schema_version": schema_version,
  "certificate_format": string | null,
  "accepted_result_hash": interface_hash_string,
  "used_axioms_hash": diagnostic_hash_string | null
}
```

`proof_status` は、proof phase と kernel phase がすでに生成した accepted status
を記録するだけである。これ自体は acceptance を発生させない。`kernel_verified`
には minimum kernel が受理した ATP certificate が必要である。
`discharged_builtin` には、受理された built-in certificate または許可された kernel
primitive が必要である。externally attested、open、pending、rejected な obligation
は trusted `ProofWitnessRef` value を生成しない。将来の development-only evidence
reference は、別の diagnostic または development schema を使わなければならない。

`evidence_kind` は accepted evidence class を記録する。`atp_certificate` は
`proof_status = "kernel_verified"` の場合だけ有効である。`builtin_certificate` と
`kernel_primitive` は `proof_status = "discharged_builtin"` の場合だけ有効である。
reader はその他の組み合わせを拒否する。

`certificate_format` は、witness payload が ATP certificate format などの format に
依存する場合、空でない string として存在する。certificate file format を持たない
allowed kernel primitive では JSON `null` である。具体的な format vocabulary は proof
crate と kernel crate が所有し、`mizar-artifact` は所有しない。task 9 reader は
`atp_certificate` と `builtin_certificate` では空でない `certificate_format` を要求し、
`kernel_primitive` では JSON `null` を要求する。

## Hash String Domains

task 9 は task 3 の artifact-framed hash string 形式を使う。

```text
mizar-artifact/artifact-framed-hash-text/v1:<class>:<schema_family>:<schema_version>:<digest>
```

`<digest>` は 64 文字の lowercase hexadecimal digest である。artifact-framed な
`schema_family` string は `registration_summary.md` と同じ grammar に従う。すなわち、
空でない、colon を含まない、slash 区切りの識別子であり、各 segment は ASCII letter、
ASCII digit、hyphen、underscore、dot だけを含む。

`witness_artifact_hash` は class `artifact` と witness payload schema family/version を
使う。task 8 の reference spec は witness payload schema family を定義しない。これは
proof/kernel witness producer work までの `external_dependency_gap` である。

`obligation_fingerprint`、`kernel_profile_fingerprint`、
`verifier_policy_fingerprint`、`accepted_result_hash` は class `interface` を使う。
これらは `mizar-artifact/proof-witness-ref` domain に書き換えず、producer が所有する
schema family と version を保持する。

`used_axioms_hash` は class `diagnostic` を使う。citation refinement と diagnostic を
支えるためである。これは proof authority ではなく、kernel acceptance metadata を
置き換えない。

## Witness Paths And Publication

`witness_path` は package artifact root からの相対 path であり、正規化後に
`proof-witnesses/` の下になければならない。task 9 は `CanonicalJson` boundary で lexical path
validation を行う。path は `/` 区切りを使い、`proof-witnesses/` で始まり、その prefix の後に
少なくとも 1 つの空でない child segment を含み、空 segment、`.` segment、`..` segment を含んでは
ならない。absolute path も許可しない。symlink と artifact-root escape の検査には filesystem access
が必要であり、artifact-store I/O に属する。manifest は publication index である。file が
`proof-witnesses/` の下に存在するだけでは publish されたことにならない。

manifest transaction は、manifest を commit する前に witness file を write し hash する。
`ProofWitnessRef` は、committed manifest entry が main artifact と witness file の両方を
参照したときにだけ publication-reachable になる。publication policy が witness を要求する
場合、file の欠落または hash mismatch は該当 artifact の publication を拒否する。

task 9 は canonical-value boundary で reference と validation contract を実装する。
atomic write、manifest visibility、byte-level corruption diagnostic、filesystem escape
check は artifact store と manifest task に deferred のままとする。

## Resident-Set Discipline

verified artifact は `ProofWitnessRef` value を resident に保持してよいが、witness payload
を inline してはならない。downstream build、LSP feature、documentation tool、cache validator は
proof body を load せずに resident reference、hash、proof status、kernel-acceptance metadata
を検査してよい。

consumer は proof witness の replay、audit、diagnostic、validation を明示的に行う場合だけ
external witness file を load する。cache hit、manifest entry、reference hash は、それ自体では
proof authority ではない。trust は accepted status を生成した proof/kernel phase と、参照先 byte
が記録 hash と一致し続けているという後続 validation からだけ生じる。

## Canonical Ordering

`VerifiedArtifact` が複数の `ProofWitnessRef` を含む場合、collection は次の順序で sort する。

1. `obligation_id`
2. `obligation_fingerprint`
3. `proof_status`
4. `evidence_kind`
5. `witness_path`
6. `witness_artifact_hash`

後続 schema が単一 obligation に複数の accepted witness を明示的に support しない限り、
reader は 1 つの verified artifact 内の duplicate `obligation_id` を拒否する。source traversal
order、ATP completion order、backend runtime、filesystem order は serialized byte に影響してはならない。

## Reader And Writer Requirements

task 9 writer は `store.md` の正準 UTF-8 JSON rule を使い、current schema version を emit する。
task 9 reader は store boundary で生成された `CanonicalJson` value を対象に動作する。file
parsing と duplicate object-key detection は artifact-store I/O が所有する。reader は:

- 上で列挙した field をすべて必須とする。存在しない値を JSON `null` で表す field も含む。
- すべての schema object で unknown field を拒否する。
- field を解釈する前に schema-version compatibility を検査する。
- id、path、certificate format、hash string の空文字列を拒否する。
- publication-safe でない witness path を拒否する。
- artifact-framed hash の construction label、class、schema-family grammar、
  schema-version grammar、digest spelling を検証する。
- unsupported な `proof_status`、`evidence_kind`、status/evidence の組み合わせを拒否する。
- caller が witness byte または manifest hash を供給した場合、`witness_artifact_hash` を検証する。
- proof payload の replay、proof acceptance、ATP 実行、internal cache record への fallback を行わない。

reader failure は artifact diagnostic である。proof authority を確立せず、externally attested
evidence へ黙って downgrade しない。

## 公開 enum の前方互換性

task 19 は frontend task 25 の public-enum 手続きを proof-witness reference API
に適用する。この module が所有するすべての public enum は forward-compatible API
surface であり、`#[non_exhaustive]` のままにしなければならない。downstream
consumer は match 時に wildcard fallback arm を持たなければならない。

これは API 互換性の判断であり、reader の寛容化ルールではない。artifact schema
reader は、将来の schema revision と version policy が受け入れ方法を明示しない限り、
unknown serialized enum value を引き続き拒否する。

| Enum | 前方互換性の判断 |
|---|---|
| `ProofStatus` | accepted proof-status category を文書化済み proof/kernel producer policy の下で拡張できるよう non-exhaustive。 |
| `EvidenceKind` | accepted evidence class を文書化済み proof/kernel producer policy の下で拡張できるよう non-exhaustive。 |
| `ProofWitnessError` | proof-witness reference validation diagnostic を拡張できるよう non-exhaustive。 |

この module は exhaustive な public enum 例外を所有しない。

## Implementation Boundary

task 9 は `ProofWitnessRef` schema、canonical value writer、validating
`CanonicalJson` reader、および round-trip と hash mismatch detection の test を実装する。

concrete witness payload schema、proof producer integration、accepted kernel result construction、
built-in certificate/primitive encoding は、proof crate と kernel crate が stable producer output を
公開するまで `external_dependency_gap` として残る。manifest/file I/O は artifact-store と manifest
task に deferred のままとする。
