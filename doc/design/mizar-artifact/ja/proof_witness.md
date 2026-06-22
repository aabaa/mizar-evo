# ProofWitnessRef

> 正本は英語です。英語版: [../en/proof_witness.md](../en/proof_witness.md)。

## 目的

`ProofWitnessRef` は公開 artifact から外部 witness file への安定参照である。
Main artifact に witness bytes を埋め込まず、accepted proof evidence を探して
hash-check するための metadata を記録する。

## Schema

Reference schema version は `mizar-artifact/proof-witness-ref/v1`。

Fields:

- `schema_version`。
- `witness_id`。
- `obligation_id`。
- `path`。
- `content_hash`。
- `evidence_kind`。
- `kernel_status`。
- `kernel_profile`。
- `accepted_by`。
- `used_axioms`。

Referenced witness file は producer-owned schema を持つ。この crate は bytes を
hash で記録・検証するが、kernel certificate semantics を発明しない。

## Evidence boundary

`kernel_status` は `mizar-kernel` / `mizar-proof` から供給された projected
acceptance metadata だけを記録してよい。`mizar-artifact` は ATP success、cache
hit、synthetic test producer を trusted proof status に変換してはならない。

Witness bytes は `VerifiedArtifact` 外に留まる。Main artifact は
`ProofWitnessRef` values だけを格納し、manifest は witness entries を格納する。

## Hash validation

Witness bytes がある場合、`ProofWitnessRef` は canonical content hash が
`content_hash` と一致することを検証する。Mismatch は publication または read
validation における hard artifact integrity error である。

## Ordering

Artifact 内の witness references は obligation id、witness id、path、content hash
で sort する。
