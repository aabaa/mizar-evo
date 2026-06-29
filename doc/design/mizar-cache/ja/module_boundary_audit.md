# mizar-cache Module Boundary 監査

> 正本は英語です。英語版:
> [../en/module_boundary_audit.md](../en/module_boundary_audit.md)。

task 22 は architecture-22 follow-up 後の `mizar-cache` source layout を監査する。
この task は 1 つの behavior-preserving refactor を行う。inline unit test を
private な `#[cfg(test)]` child module へ分割する。runtime implementation code、
public module、public API、diagnostic、deterministic rendering、
artifact-facing schema、cache lookup policy、proof-reuse validation policy、
proof acceptance policy、downstream integration status は変更しない。

## Boundary Decision

public module table は `src/lib.rs` が export する次の module のままである。

- `cache_key`
- `dependency_fingerprint`
- `cache_store`
- `proof_reuse`
- `cluster_db`

新しい production module は追加しない。新しい source path は private test
submodule だけである。

- `src/cache_key/tests.rs`
- `src/dependency_fingerprint/tests.rs`
- `src/cache_store/tests.rs`
- `src/proof_reuse/tests.rs`
- `src/cluster_db/tests.rs`

移動した file は owning module への従来の `use super::*` 関係を維持し、
`#[cfg(test)] mod tests;` により test build のときだけ compile される。API、
scheduling hook、`mizar-ir` adapter、publication token、proof-status projection、
kernel authority shortcut は公開しない。

## Source Layout Result

| Module | task 22 前 | task 22 後 | Decision |
|---|---:|---:|---|
| `cache_key` | inline 2005 行 | implementation 1241 行 + test 759 行 | test だけ分割。 |
| `dependency_fingerprint` | inline 2501 行 | implementation 1455 行 + test 1039 行 | test だけ分割。 |
| `cache_store` | inline 2632 行 | implementation 1726 行 + test 896 行 | test だけ分割。 |
| `proof_reuse` | inline 1094 行 | implementation 602 行 + test 491 行 | test だけ分割。 |
| `cluster_db` | inline 2626 行 | implementation 1491 行 + test 1133 行 | test だけ分割。 |

implementation file は module-owned data type、builder、validator、store、
deterministic canonicalization の責務を保つ。test helper は同じ module namespace
に colocate されたままだが、production file を review bottleneck にしない。

## Source/Spec And Bilingual Check

`source_spec_audit.md` は private test submodule path を列挙するよう更新され、
lint guard はすべての Rust source file を引き続き確認する。public module の
source path は変わらない。paired English/Japanese design document は move-only
scope で一致し、新しい gap ID は追加しない。

この refactor は次の境界を保つ。

- cache record と cache hit は optimization input に限られる。
- cache record、external evidence、diagnostic、log、timing metadata、
  write/arrival order は kernel-verified status や trusted `used_axioms` に
  ならない。
- proof reuse は `mizar-proof` validation metadata だけを消費し続ける。
- scheduler integration、`mizar-ir` adapter、artifact publication-token
  integration は既存の external dependency gap のままである。

## Verification Scope

task 22 は Rust test module を移動するため、required verification は Rust source
change set である。`cargo fmt --check`、`cargo test -p mizar-cache`、
`cargo clippy -p mizar-cache --all-targets -- -D warnings`、および crate workflow が
completion 前に要求する broader verification を実行する。

## Conclusion

task 22 は inline unit test により oversized になっていた implementation file という
具体的な review bottleneck を解消する。behavior、public API、authority boundary、
cache semantics、proof-reuse semantics、cluster-db visibility、integration
readiness は変更しない。
