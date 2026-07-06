# mizar-artifact ProofWitnessRef Schema

> 正本は英語です。英語版: [../en/proof_witness.md](../en/proof_witness.md)。

`ProofWitnessRef` は proof witness file の stable artifact projection である。
published verified artifact が witness payload を main artifact に埋め込まず、
path と hash で witness を指せるようにする。

この document は `ProofWitnessRef` schema を定義する。task 9 は
`CanonicalJson` boundary での schema、canonical value writer、validating
reader、test を導入した。task 23 は trusted witness projection を legacy
certificate acceptance から formula/substitution kernel evidence へ改訂する。task 24 は
この schema を、監査後の kernel/vc goal-polarity と context-identity follow-up に
対して再点検する。

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
`1.0` は初期 certificate-reference version であった。version `2.0` が現在
support される version であり、formula/substitution kernel evidence reference
の最初の version である。

## Conceptual Shape

task 23 は次の canonical JSON field shape を使う。

```text
proof_witness_ref = {
  "schema_version": "2.0",
  "obligation_id": string,
  "obligation_fingerprint": interface_hash_string,
  "proof_status": "kernel_verified",
  "evidence_kind": "formula_substitution_kernel_evidence",
  "witness_path": string,
  "witness_artifact_hash": artifact_hash_string,
  "kernel_acceptance": kernel_acceptance_metadata
}

kernel_acceptance_metadata = {
  "kernel_profile_fingerprint": interface_hash_string,
  "verifier_policy_fingerprint": interface_hash_string,
  "checker_schema_version": schema_version,
  "evidence_schema_version": schema_version,
  "target_binding_hash": interface_hash_string,
  "formula_evidence_hash": interface_hash_string,
  "substitution_evidence_hash": interface_hash_string,
  "provenance_hash": interface_hash_string,
  "formula_context_hash": interface_hash_string | null,
  "accepted_result_hash": interface_hash_string
}
```

`proof_status` は、proof phase と kernel phase がすでに生成した accepted status
を記録するだけである。これ自体は acceptance を発生させない。`kernel_verified`
は schema version `2.0` における唯一の trusted accepted status であり、kernel が
formula/substitution evidence を、自身で導出した deterministic instantiation と SAT
encoding に対して checked したことを意味する。externally attested、open、pending、
rejected、legacy certificate、backend-only obligation は trusted `ProofWitnessRef`
value を生成しない。development-only evidence reference は、別の diagnostic または
development schema を使わなければならない。

`evidence_kind` は accepted evidence class を記録する。schema version `2.0` の唯一の
trusted value は `formula_substitution_kernel_evidence` である。reader は legacy
`atp_certificate`、`builtin_certificate`、`kernel_primitive`、resolution trace、
SMT proof object、backend log、backend method、および上に列挙していない
status/evidence combination を拒否する。

`kernel_acceptance` は kernel evidence handoff と accepted result の stable
projection である。formula/substitution/provenance/target-binding hash、optional な
imported formula-context hash、schema version、policy fingerprint だけを記録する。
instantiated formula と SAT problem は caller-supplied trusted payload ではなく、
ここには保存しない。kernel は formula evidence と substitution evidence からそれらを
導出して acceptance を check する。

task 24 は schema version `2.0` を変更しない。修正後の kernel contract は
explicit proof-obligation goal polarity と non-imported source context identity を
owner-owned VC/kernel/proof pipeline に追加するが、`ProofWitnessRef` はそれらの
payload を再計算したり再描画したりしない。代わりに:

- `obligation_fingerprint` は producer-owned な composite witness/reuse
  fingerprint である。trusted formula/substitution witness について、producer
  contract は、explicit accepted goal-polarity decision と、handoff が存在する場合の
  task-28 `context_identity_hash()` を含む current VC/proof identity からこれを
  導出することを要求する。
- `kernel_acceptance.target_binding_hash` は selected obligation について受理された
  producer/kernel target と canonical evidence binding を指す。これは別個の
  context-identity hash の代替として使ってはならない。
- `kernel_acceptance.formula_context_hash` は imported axiom または theorem が
  関与する場合の imported formula context identity を運ぶ。accepted evidence
  boundary に imported formula context が含まれない場合だけ `null` になる。
- `kernel_acceptance.accepted_result_hash` は trusted proof owner から copied された
  accepted kernel/proof result hash である。proof publication が witness reference
  を stage する前に、selected evidence hash と一致しなければならない。

これは no-change schema decision である。`ProofWitnessRef` に `goal_polarity`、
`context_identity_hash`、proof-reuse validation hash の field は追加しない。将来の
producer が `obligation_fingerprint` と accepted-result metadata を
goal-polarity / context-identity check 後に導出したことを証明できない場合、artifact
publication はこの schema に field を捏造するのではなく、既存の producer-integration
gap により block されなければならない。

backend proof method、portfolio 名、solver log、resolution trace、SMT proof object は
trusted witness content ではない。保存する場合は、`kernel_acceptance` と accepted
witness identity の外にある diagnostic または provenance attachment に置かなければ
ならない。

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
使う。task 23 の fixture family は `mizar-kernel/formula-evidence-witness` である。
concrete producer publication は将来の proof/producer integration が所有する。

`obligation_fingerprint`、`kernel_profile_fingerprint`、
`verifier_policy_fingerprint`、`target_binding_hash`、`formula_evidence_hash`、
`substitution_evidence_hash`、`provenance_hash`、`formula_context_hash`、
`accepted_result_hash` は class `interface` を使う。これらは
`mizar-artifact/proof-witness-ref` domain に書き換えず、producer が所有する
schema family と version を保持する。

trusted hash field は backend proof method、resolution trace、certificate format、
solver log、used-axiom diagnostic を記録しない。これらはこの artifact schema における
proof authority ではない。

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

reference と validation contract は canonical-value boundary で実装される。
atomic write、manifest visibility、byte-level corruption diagnostic、filesystem escape
check は artifact store と manifest layer に属する。

## Resident-Set Discipline

verified artifact は `ProofWitnessRef` value を resident に保持してよいが、witness payload
を inline してはならない。downstream build、LSP feature、documentation tool、cache validator は
proof body を load せずに resident reference、hash、proof status、kernel-acceptance metadata
を検査してよい。

consumer は proof witness の audit、diagnostic、validation を明示的に行う場合だけ
external witness file を load する。cache hit、manifest entry、reference hash は、それ自体では
proof authority ではない。`mizar-artifact` は kernel が参照 evidence を受理したことを記録し、
後続 validation で参照先 byte が記録 hash と一致し続けていることを確認する。kernel acceptance
の再計算や prover 実行は行わない。

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

writer は `store.md` の正準 UTF-8 JSON rule を使い、current schema version を emit する。
reader は store boundary で生成された `CanonicalJson` value を対象に動作する。file
parsing と duplicate object-key detection は artifact-store I/O が所有する。reader は:

- 上で列挙した field をすべて必須とする。存在しない値を JSON `null` で表す field も含む。
- すべての schema object で unknown field を拒否する。
- field を解釈する前に schema-version compatibility を検査する。
- id、path、hash string の空文字列を拒否する。
- publication-safe でない witness path を拒否する。
- artifact-framed hash の construction label、class、schema-family grammar、
  schema-version grammar、digest spelling を検証する。
- unsupported な `proof_status`、`evidence_kind`、status/evidence の組み合わせを拒否する。
- `certificate_format` のような legacy certificate field、resolution trace、backend log、
  caller-supplied instantiated-formula または SAT problem payload を拒否する。
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

task 23 は schema version `1.0` 由来の legacy public enum variant を source
compatibility のため保持するが、現在の trusted schema では unsupported として扱う。
`DischargedBuiltin`、`AtpCertificate`、`BuiltinCertificate`、`KernelPrimitive` は downstream
source の match に現れ続けてよいが、現在の writer/reader は status/evidence validation を通じて拒否する。

| Enum | 前方互換性の判断 |
|---|---|
| `ProofStatus` | accepted proof-status category を文書化済み proof/kernel producer policy の下で拡張できるよう non-exhaustive。現在の trusted schema は `KernelVerified` だけを受理し、legacy `DischargedBuiltin` は source-compatible だが unsupported。 |
| `EvidenceKind` | accepted evidence class を文書化済み proof/kernel producer policy の下で拡張できるよう non-exhaustive。現在の trusted schema は `FormulaSubstitutionKernelEvidence` だけを受理し、legacy certificate/primitive variant は source-compatible だが unsupported。 |
| `ProofWitnessError` | proof-witness reference validation diagnostic を拡張できるよう non-exhaustive。 |

この module は exhaustive な public enum 例外を所有しない。

## Implementation Boundary

task 23 は schema version `2.0`、canonical value writer、validating `CanonicalJson`
reader、および round-trip、deterministic writer output、version mismatch rejection、
hash-domain validation、legacy certificate-field rejection、witness hash mismatch detection
の test を実装する。

task 24 は Rust schema の変更も version bump も追加しない。修正後の goal-polarity と
context-identity binding は新しい artifact-owned field ではなく owner-produced hash によって
運ばれるため、既存の schema `2.0` reference は同じ reader policy の下で読み取り可能な
ままである。

concrete witness payload publication、proof producer integration、full phase 15 emission は、
real producer output が存在するまで `external_dependency_gap` として残る。schema version `1.0`
の legacy certificate reference との互換性は、通常の trusted reader では保持しない。migration
reader が必要な場合は、trusted proof acceptance の外にある明示的な audit-only reader でなければ
ならない。
