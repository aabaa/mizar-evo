# mizar-artifact module 境界リファクタリング gate

> 正本は英語です。英語版:
> [../en/module_boundary_refactor.md](../en/module_boundary_refactor.md)。

状態: task 22 gate complete。

## 範囲

この gate は task 1-21 後の `mizar-artifact` source layout を監査する。
oversized file、混在した責務、module table と module spec boundary に沿って
分割すべき private helper を確認する。

この refactor は behavior-preserving である。public module name、public type、
public function、schema field name、canonical JSON ordering、hash input、diagnostic、
path validation、manifest transaction behavior、artifact-facing schema は変更しない。

## 結果

- public module path は `mizar_artifact::{store, module_summary,
  registration_summary, proof_witness, verified_artifact, manifest}` のまま。
- inline unit-test block を module ごとの private `tests.rs` file に移した:
  `store/tests.rs`、`module_summary/tests.rs`、`registration_summary/tests.rs`、
  `proof_witness/tests.rs`、`verified_artifact/tests.rs`、`manifest/tests.rs`。
- production root は既存 module spec と意図的に揃えたままにする。split 後に最大の
  root は `verified_artifact.rs`（約 3.3k 行）、`registration_summary.rs`
  （約 2.5k 行）、`manifest.rs`（約 2.3k 行）である。これらが大きいのは、
  validation、canonical JSON construction、hash participation、reader error
  reporting を含む 1 つの stable schema boundary を、それぞれ一緒に review する
  必要があるためである。
- 追加の production helper split は行わない。候補 helper block は schema と結合して
  おり、今移動すると mixed-responsibility boundary を減らすよりも private module
  間の visibility を増やしてしまう。
- task 19 の public-enum guard は、test-only `tests.rs` file を除外しつつ source file
  を再帰 scan するようにした。これにより将来の private production module は policy
  check から漏れない。
- 移動した tests とこの audit file について、source/spec correspondence と bilingual
  documentation synchronization scope を再実行し、新しい drift は見つからなかった。

## source layout

| Public module | Public source | task 22 後の private tests | Gate result |
|---|---|---|---|
| `store` | `src/store.rs` | `src/store/tests.rs` | Canonical JSON、hash framing、path safety、store I/O API は module root に残し、tests を implementation body から移した。 |
| `module_summary` | `src/module_summary.rs` | `src/module_summary/tests.rs` | Module-summary schema、canonical writer/reader、interface-hash helper は module root に残し、tests を移した。 |
| `registration_summary` | `src/registration_summary.rs` | `src/registration_summary/tests.rs` | Registration-summary schema、trace-reference validation、registration-interface hash helper は module root に残し、tests を移した。 |
| `proof_witness` | `src/proof_witness.rs` | `src/proof_witness/tests.rs` | Proof-witness reference schema と validation は module root に残し、tests を移した。 |
| `verified_artifact` | `src/verified_artifact.rs` | `src/verified_artifact/tests.rs` | Verified-artifact schema、provenance、hash-input helper、witness validation、reader/writer rule は module root に残し、tests を移した。 |
| `manifest` | `src/manifest.rs` | `src/manifest/tests.rs` | Manifest schema、file I/O、transaction writer、reference validation は module root に残し、tests を移した。 |

## 再実行した監査

- Source/spec correspondence: public API source root は引き続き module spec と一致する。
  [source_spec_correspondence.md](./source_spec_correspondence.md) の task-22 row は
  private test split を記録し、public API、behavior、diagnostic、rendering、
  artifact schema、boundary drift がないことを確認する。
- Bilingual documentation synchronization: この file を両言語ディレクトリへ追加し、
  [bilingual_documentation_sync.md](./bilingual_documentation_sync.md) は新しい pair を
  含むようになった。
- Boundary discipline: split は `mizar-artifact` 内に閉じている。raw IR ownership、
  cache-record ownership、scheduler state、proof authority、kernel acceptance behavior
  は追加しない。

## Verification

task 22 は crate の全 source module に触れるため、focused artifact verification と
広い workspace check の両方を要求する:

```text
cargo fmt --check
cargo test -p mizar-artifact
cargo clippy -p mizar-artifact --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
