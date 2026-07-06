# Module-Boundary Refactor Gate: mizar-core

> 正本は英語です。英語版:
> [../en/module_boundary_audit.md](../en/module_boundary_audit.md)。

Task 24 は closeout 前に `mizar-core` source layout を監査する。oversized file、
混在した責務、private helper が、downstream consumer 向けに crate を完了扱いに
する前に behavior-neutral move を必要とするかを確認する。

この task は audit-only である。現在の public module layout は module
specification boundary と一致しており、split を要求する current review-bottleneck
は見つからず、blocking boundary violation も見つからないため、Task 24 では Rust
source を移動しない。

## Scope And Method

この audit は次を対象にする:

- `crates/mizar-core/src/lib.rs`
- `crates/mizar-core/src/core_ir.rs`
- `crates/mizar-core/src/binder_normalization.rs`
- `crates/mizar-core/src/elaborator.rs`
- `crates/mizar-core/src/control_flow.rs`
- `crates/mizar-core/tests/` の crate-local integration test
- `doc/design/mizar-core/{en,ja}/` の英日 module spec
- `crates/mizar-core/tests/lint_policy.rs` の lint / audit guard

review は source layout を `todo.md` の module table、source/spec audit、owning
module specification と比較する。file size は reviewability signal であり、それ
だけでは code move の理由にならない。file が module/spec boundary を越えて責務を
混在させる、予期しない public API を expose する、または将来作業の安全な review を
妨げる場合にのみ split が必要である。

## Source Inventory

| Source | Approx. lines at audit | Owning spec | Boundary result |
|---|---:|---|---|
| `src/lib.rs` | 9 | `todo.md` の module table | `binder_normalization`, `control_flow`, `core_ir`, `elaborator` だけを export する。drift なし。 |
| `src/core_ir.rs` | 4015 | `core_ir.md` | 大きいが cohesive な data-shape module。closeout 前の split は不要。 |
| `src/binder_normalization.rs` | 5828 | `binder_normalization.md` | 大きいが cohesive な binder/substitution/canonicalization module。future private helper extraction は optional。 |
| `src/elaborator.rs` | 16173 | `elaborator.md` | 最大の review-risk file だが、section は owning spec の six elaboration step に対応する。Task 30 の template type-parameter sethood fixture は Step 2 type/fact と Step 3 term/formula elaboration boundary 内に残る。この audit は split を要求する current review-bottleneck とは分類しない。 |
| `src/control_flow.rs` | 6718 | `control_flow.md` | 大きいが phase-10 CFG、contract、diagnostic、handoff section に対応する。この task で mandatory split はしない。 |
| `tests/determinism_suite.rs` | 627 | `00.crate_plan.md`, task 20 | cross-module integration test。boundary issue なし。 |
| `tests/lint_policy.rs` | 1167 | task 1, task 21, task 22 policy | policy/audit guard test。boundary issue なし。 |

`tests/lint_policy.rs` は現在の public module list を guard し、policy guard を更新する
まで semantic module file 内の public nested module / re-export を拒否し、public
enum policy drift と Task 22 source/spec audit inventory を検査する。implementation
file は物理的には大きいが、これらの guard により public boundary は明示されている。

Task 30 では explicit template type-parameter sethood payload、Fraenkel
cross-reference validation、Rust fixture を `src/elaborator.rs` に追加した後、
この audit を再確認した。public module boundary と owning spec は変わらない。
新しい局所的な Step 2/Step 3 elaboration behavior によって move-only split は
必要にならない。

## Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| CORE-BOUNDARY-G001 | `deferred` | `src/elaborator.rs` は最大の implementation file で、step-specific lowering helper と dense task-local test を含む。 | Step 1-6 helper/test section を public API や behavior を変えずに分ける dedicated move-only task へ defer する。 |
| CORE-BOUNDARY-G002 | `deferred` | `src/control_flow.rs` は CFG construction、contract/ghost/termination attachment、diagnostic、handoff、test を one phase-10 module に含む。 | reviewability bottleneck が発生した場合、future move-only task で private builder / diagnostic / handoff helper を分けてもよい。 |
| CORE-BOUNDARY-G003 | `deferred` | `src/binder_normalization.rs` は raw normalization、substitution、closure expansion、canonicalization、test を one binder module に含む。 | 必要なら closeout 後の future move-only task で private helper section を分けてもよい。 |
| CORE-BOUNDARY-G004 | `external_dependency_gap` | source-derived payload seam、downstream VC/kernel/proof/artifact consumer、active semantic snapshot はこの crate task の外に残る。 | 利用不能な downstream / upstream seam の placeholder module は作らない。 |

`boundary_violation`、`source_drift`、`source_undocumented_behavior`、
`repo_metadata_conflict`、blocking `design_drift` は見つからない。古い architecture-06
submodule 名は task-0 plan と module spec がすでに精緻化しており、この audit では
その historical design drift を再オープンしない。

## Split Decision

Task 24 では file split を行わない。

理由:

- Public module boundary は module table と owning spec にすでに一致している。
- 大きな implementation file はそれぞれ public module responsibility を中心に
  cohesive で、task-local test によって cover されている。
- この audit は large review-risk file を見つけたが、TODO rule に基づき split
  すべき current review-bottleneck implementation file は見つけていない。
- closeout 直前に数千行を移動すると、behavior gain のない mechanical churn と
  高い review cost が発生する。
- 安全な split は、public API、diagnostic、debug rendering を変えず、full Rust
  verification を行う dedicated move-only follow-up として扱うべきである。

将来 task が split を行う場合は、module-boundary audit を更新し、移動した API に対して
source/spec audit scope を再実行し、path/document 変更がある場合は bilingual
documentation sync scope も再実行する。新しい spec task が明示的に変更しない限り、
public module export は変えない。

## Verification

Task 24 は audit-only で Rust source を変更しないため:

- stage 前の `git diff --check`。
- 明示 path stage 後の `git diff --cached --check`。

review がこの task で source movement を要求した場合は次を実行する:

- `cargo fmt --check`
- `cargo test -p mizar-core`
- `cargo clippy -p mizar-core --all-targets -- -D warnings`
