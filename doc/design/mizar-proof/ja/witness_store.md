# mizar-proof Witness Store

> 正本: [../en/witness_store.md](../en/witness_store.md)。

## 目的

`mizar-proof` は、選択された trusted proof outcome の proof witness draft staging と
publication reference を所有する。

witness store は proof authority ではない。proof を accept せず、kernel を再実行せず、
ATP backend を実行せず、SAT solving を行わず、cache を query せず、artifact manifest
を書かない。すでに `mizar-kernel` が accept した evidence の byte と metadata を記録し、
安定した content hash を計算し、artifact publication boundary が witness の manifest
到達可能性を証明した後にだけ publication reference を公開する。

## 入力

store は、同じ obligation の deterministic status projection とも一致する trusted
selection に対する `ProofWitnessDraft` だけを消費する:

- 対応する accepted kernel result を持つ `KernelVerified` formula/substitution kernel evidence;
- artifact schema support が存在するまで internal staged outcome に留まる `DischargedBuiltin`。

draft は次を持つ:

- `obligation_id`、`ObligationAnchor`、`obligation_fingerprint`;
- status projection と kernel metadata token から copy された selected winner class と
  selected evidence hash;
- witness payload schema family と schema version;
- canonical witness payload bytes;
- `ProofWitnessRef` version `2.0` が要求する kernel-acceptance metadata;
- verifier policy fingerprint と checker/evidence schema version;
- target binding、formula evidence、substitution evidence、provenance、optional formula
  context、accepted result、dependency artifact、build snapshot fingerprint を含む
  provenance metadata;
- non-trusted attachment のための optional diagnostics-owned provenance reference。

externally attested evidence、policy assumption、open obligation、rejected obligation、
no-selectable evidence、backend log、backend proof method、resolution trace、SMT proof object、
cache record、backend-reported axiom list は trusted witness draft を作らない。

## State Machine

witness handling には 3 つの状態がある:

1. `ProofWitnessDraft`: producer-owned bytes、trusted kernel metadata、対応する status
   projection。kernel metadata token は、kernel/artifact integration が copy 済みの
   acceptance metadata を供給するまで public constructor を持たない。store されず、
   publish されず、それ自体では cache validation にならない。
2. `ProofWitnessStagedRef`: `stage` が返す。stable witness path candidate、payload hash、
   payload schema、obligation identity、selected class、provenance metadata と、
   publish 可能な `KernelVerified` evidence では artifact builder が commit 前に
   `VerifiedArtifact.proof_witnesses` へ埋め込める unpublished `ProofWitnessRef`
   candidate を記録する。
   まだ publication-reachable ではない。
3. `ProofWitnessPublishedRef`: artifact publication boundary が staged tuple と一致し、
   committed main artifact から到達可能にする committed witness publication proof を
   供給した後にだけ `publish_ref` が返す。

artifact layer が witness bytes を write/hash できるよう、`stage` は artifact commit の前に
行わなければならない。`publish_ref` は committed manifest entry が同じ witness path、
obligation id、obligation fingerprint、witness artifact hash を参照し、module manifest
entry が同じ main `VerifiedArtifact` proof-witness set を参照した後にだけ行う。単独で一致
する witness tuple だけでは十分ではない。artifact-owned な
`CommittedWitnessPublicationProof` input は、witness entry を committed module artifact
entry または同等の committed `VerifiedArtifact` reference set に bind し、manifest の
`proof_witnesses` array が artifact の `proof_witnesses` collection をちょうど cover することを
示さなければならない。その artifact-owned token が存在するまで、source API は proof token
を public constructor なしの opaque token として保つ。caller は一致する tuple を合成しては
ならない。その publication proof が存在する前に `publish_ref` を呼ぶと error である。

`publish_ref` は commit 後に新しい artifact reference を発明しない。`stage` が出した
unpublished reference candidate が committed `VerifiedArtifact` と manifest に記録された
同じ reference であることを検証し、その同じ reference に対する published wrapper を返す。

## Stable Hashing

proof witness hash は、正確な staged payload bytes と payload schema identity に対する
artifact-framed hash である:

- schema family;
- schema version;
- canonical payload bytes;
- witness payload hash domain;
- selected accepted evidence hash;
- verifier policy fingerprint;
- obligation fingerprint。

hash には temporary file path、staging directory name、arrival order、backend completion
time、process id、wall-clock time、random data、cache-hit metadata、manifest commit timing を
含めてはならない。

2 つの draft が同一の payload bytes と同一の hash input を持つ場合、同じ witness hash を
生む。accepted evidence hash、policy fingerprint、obligation fingerprint、payload schema
version、payload bytes など trusted input が変わる場合、witness hash は変わるか staging
が reject される。

## Publication References

`KernelVerified` formula/substitution evidence では、`stage` が次を持つ unpublished
`ProofWitnessRef` version `2.0` candidate を準備する:

- `proof_status = "kernel_verified"`;
- `evidence_kind = "formula_substitution_kernel_evidence"`;
- staged witness path と witness artifact hash;
- accepted kernel evidence boundary から copy した kernel acceptance metadata。

`publish_ref` に成功すると、committed manifest reachability が証明された後にだけ、その同じ
reference を publication-reachable として返す。

store は non-trusted status の `ProofWitnessRef` を publish してはならない。unsupported
trusted status を `kernel_verified` に書き換えてもならない。

`DischargedBuiltin` は現在 artifact witness publication の `external_dependency_gap` のままである。
store は internal draft を stage し、stable reuse metadata を公開してよいが、`mizar-artifact` が
distinct な trusted `DischargedBuiltin` witness status/evidence combination を support するまで
`publish_ref` は unsupported-witness gap を返さなければならない。staged
`DischargedBuiltin` hash は internal かつ non-artifact-facing であり、
`selected_proof_witness_hash` として export してはならず、`ProofWitnessRef` に現れてはならず、
この gap が残る間は selection/status reuse metadata が要求する
`deterministic_discharge_hash` を置き換えてはならない。

## Provenance Metadata

staged record と published record は、diagnostics と reuse validation に必要な provenance を
保持する:

- build snapshot と producer identity;
- selected candidate id と selected winner class;
- kernel evidence origin;
- target VC fingerprint と obligation fingerprint;
- dependency slice と dependency artifact fingerprint;
- verifier policy fingerprint;
- checker と evidence schema version;
- 利用可能な場合の accepted result hash と trusted used-axiom reference hash;
- advisory backend data の diagnostics-owned reference。

trusted value が存在する場合、provenance metadata は selected status projection、kernel
acceptance metadata、trusted used-axiom token と一致しなければならない。provenance
metadata は trust boundary を広げない。dependency artifact fingerprint は task 11 では
producer-owned reuse metadata として保持される。committed dependency artifact への
authoritative binding は artifact/cache integration まで deferred である。backend log、
externally attested citation、cache record、diagnostic hint は diagnostic または
reuse-validation material のままである。

## Failure Semantics

store は次を reject するか gap として報告する:

- unsupported witness class または evidence kind;
- missing または mismatched accepted evidence hash;
- status projection と kernel acceptance metadata に一致しない draft selected class、
  selected evidence hash、obligation identity、policy fingerprint、selected candidate id、
  kernel evidence origin、schema version、trusted `used_axioms` reference;
- malformed payload schema identity;
- payload schema が canonical bytes を要求する場合の empty payload bytes。byte-level の
  canonicality validation は concrete producer-owned schema validator まで deferred;
- `proof-witnesses/` から escape する witness path;
- staged bytes と manifest reference の hash mismatch;
- 1 つの obligation に対する duplicate manifest reference;
- matching committed witness publication proof が存在する前の `publish_ref`;
- committed main artifact に bind されていない、または `VerifiedArtifact.proof_witnesses`
  collection をちょうど cover しない manifest witness entry;
- stale build snapshot または mismatched obligation fingerprint;
- externally attested、assumed、open、rejected、no-selectable evidence を trusted witness
  material として publish しようとすること。

failure は deterministic diagnostic または typed store error である。trusted proof status に
ならず、trusted `used_axioms` も合成しない。

## Cache And Reuse Boundary

publish 可能な `KernelVerified` witness について、staged/published witness hash は
proof-reuse validation に参加するが、proof authority ではない。cache record は、witness
hash、selected evidence hash、obligation fingerprint、policy fingerprint、schema version、
accepted kernel metadata が現在の validation predicate とすべて一致する場合だけ proof を
reuse してよい。dependency artifact fingerprint は、downstream artifact/cache owner が
authoritative binding を供給した後にだけその predicate の一部になる。task 11 では
producer-owned reuse metadata として保持する。staged hash は successful publication を通じてだけ
artifact-facing な `selected_proof_witness_hash` になる。その hash は witness payload artifact
hash（`witness_artifact_hash`）であり、`ProofWitnessRef` metadata object の hash ではない。
artifact support が存在するまで、`DischargedBuiltin` reuse は
`deterministic_discharge_hash` を使い続ける。internal staged hash は selected proof witness
hash ではない。cache hit は witness を publish できず、non-trusted status を昇格させられず、
trusted `used_axioms` も作れない。

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `WITNESS10-G001` | `external_dependency_gap` | `mizar-artifact` `ProofWitnessRef` version `2.0` は現在 `kernel_verified` formula/substitution evidence だけを受理する。`DischargedBuiltin` witness publication は unsupported のままであり、collapse してはならない。 |
| `WITNESS10-G002` | `deferred` | artifact manifest commit integration が、`publish_ref` に必要な committed witness publication proof を供給する。これには committed main artifact への binding と `VerifiedArtifact.proof_witnesses` の exact coverage が含まれる。この spec は token/validation contract を定義するが manifest は書かない。 |
| `WITNESS10-G003` | `deferred` | formula/substitution witness bytes 以外の concrete payload schema は producer-owned のままである。store は schema identity と bytes を hash するが backend proof payload は解釈しない。 |
| `WITNESS11-G001` | `external_dependency_gap` | task 11 は、`mizar-artifact` が artifact-owned committed publication proof token を公開するまで `CommittedWitnessPublicationProof` を opaque かつ external には construct 不可能に保つ。internal test は token を構築してよいが、production caller は artifact integration を待たなければならない。 |
| `WITNESS11-G002` | `external_dependency_gap` | task 11 は、kernel/artifact boundary が copy 済みの `KernelAcceptanceMetadata` を公開するまで `TrustedKernelWitnessMetadata` を opaque かつ external には construct 不可能に保つ。`mizar-proof` は caller が合成した kernel acceptance metadata を trusted witness material として受理してはならない。 |
| `WITNESS11-G003` | `deferred` | task 11 は payload schema が canonical bytes を要求するかを記録し、その attestation 下では empty payload を reject するが、byte-level canonicality は concrete payload schema が validator を公開するまで producer-owned のままである。witness hash は引き続き exact bytes と schema identity を覆う。 |
| `WITNESS11-G004` | `deferred` | task 11 は `dependency_artifact_fingerprint` を producer-owned provenance として保持する。committed dependency artifact と cache validation への binding は artifact/cache integration まで deferred であり、それ単体で witness status を昇格してはならない。 |

## Public Enum Policy

task 14 は public-enum forward-compatibility procedure をこの module に適用する。
public witness-store error enum は downstream-facing API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard match
arm を保つ。新しい error variant には paired spec/test review が必要であり、opaque な
publication-token boundary や trusted-metadata boundary を弱めてはならない。

| Enum | Compatibility decision |
|---|---|
| `ProofWitnessStoreError` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Non-Goals

witness store は proof search、premise selection、substitution invention、ATP/SAT backend 呼び出し、
kernel 呼び出し、cache query、artifact manifest write、placeholder witness reference publication
を行わない。
