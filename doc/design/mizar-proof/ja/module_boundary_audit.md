# mizar-proof Module-Boundary Refactor Gate

> 正本: [../en/module_boundary_audit.md](../en/module_boundary_audit.md)。

## 範囲

task 19 は proof-reuse metadata contract 後の `mizar-proof` source layout を監査する:

- `crates/mizar-proof/src/policy.rs`;
- `crates/mizar-proof/src/selection.rs`;
- `crates/mizar-proof/src/status.rs`;
- `crates/mizar-proof/src/witness_store.rs`;
- `crates/mizar-proof/src/<module>/tests.rs` 以下の crate-private test module;
- `crates/mizar-proof/tests/` 以下の integration/lint test;
- paired module spec と internal module-layout ownership map。

この task は move-only である。public API、diagnostics、deterministic rendering、
artifact-facing schema、cache authority、proof policy、proof acceptance behavior を
変えてはならない。

## Refactor Decision

audit は、implementation file が review bottleneck になっていた主因が production
code の下に inline された crate-private unit tests であることを確認した。これらの
test は private helper への access を必要とするが、同じ physical file にある必要はない。

task 19 は各 inline unit-test module を private child module へ移動する:

| Parent module | Production file before | Production file after | Private test module | Boundary result |
|---|---:|---:|---|---|
| `policy` | 2,287 lines | 1,374 lines | `src/policy/tests.rs` | public API unchanged |
| `selection` | 2,679 lines | 1,592 lines | `src/selection/tests.rs` | public API unchanged |
| `status` | 2,433 lines | 1,151 lines | `src/status/tests.rs` | public API unchanged |
| `witness_store` | 2,565 lines | 1,439 lines | `src/witness_store/tests.rs` | public API unchanged |

各 parent は `#[cfg(test)] mod tests;` を宣言する。Rust privacy により child test
module は parent module boundary 内に残るため、production API を新しく公開せずに
private helper を引き続き検査できる。

この task では production helper cluster は移動しない。残る production section は
paired module spec とまだ一致している:

- `policy.rs` は policy classification、policy fingerprint、external evidence
  admission、early-stop policy query を所有する;
- `selection.rs` は evidence candidate、deterministic winner ordering、selected
  reuse metadata、artifact proof-selection merge を所有する;
- `status.rs` は obligation identity、trusted used-axiom propagation、status
  projection、artifact publication availability、proof-reuse validation metadata を
  所有する;
- `witness_store.rs` は witness draft staging、manifest-gated publication reference、
  witness provenance check を所有する。

private production helper をさらに分割すると、現時点では source/spec boundary の
明確な利点なしに module plumbing と review risk を増やす。task 20 quality review
または downstream integration が具体的な bottleneck を見つけた場合、後続の
move-only refactor として扱う。

## Source/Spec Recheck

| Area | Recheck | Result |
|---|---|---|
| Public API paths | public module は `policy`、`selection`、`status`、`witness_store` のままであり、public enum policy は同じ production source file を指す。 | consistent |
| Deterministic behavior | test body は assertion や fixture logic を変えずに移動した。 | consistent |
| Trust boundary | source module は kernel call、ATP backend execution、cache lookup、artifact manifest commit、trusted promotion logic を追加していない。 | consistent |
| Reuse metadata | task-17 selection/status API と tests は同じ parent module に残る。 | consistent |
| Lint guard | `proof_crate_tree_contains_task_nineteen_private_test_modules` allowlist は private test submodule のためだけに更新し、message は task 19 を指す。 | consistent |

## Bilingual Sync

task 19 はこの English canonical audit と Japanese companion を追加し、section、table、
task status、handoff update を対応させる。Japanese docs は Rust path、command name、
enum name、gap id などの stable identifier を English のまま保持する。

## Gap Classification

| ID | Class | Evidence | Handling |
|---|---|---|---|
| `PROOF19-G001` | `deferred` | production module はまだ 1,100 lines を超えるが、各 module は established module spec に従っており、review risk を増やさず complexity を下げる小さな private helper boundary はまだ特定されていない。 | task 20 quality review が concrete bottleneck を見つける場合、または後続 consumer が move-only split を要求する場合だけ再検討する。 |
| `PROOF19-G002` | resolved `repo_metadata_conflict` | task 19 中、`mizar-atp` task-28 closeout guard は、現在は正式な `crates/mizar-proof` crate をまだ forbidden placeholder と見なしていた。 | task 19 では report のみ。task-20 closeout 前に focused metadata correction commit `36d1a9c` で解消済み。 |

## 結論

task 19 は large inline test module を private child module へ移動して review friction を
下げる。この refactor は public API、diagnostics、deterministic output、
artifact-facing schema、trust boundary を保つ。placeholder cache/artifact/ATP
integration は追加しない。
