# Module Boundary Audit: mizar-kernel

> 正本は英語です。英語版:
> [../en/module_boundary_audit.md](../en/module_boundary_audit.md)。

## Scope

Task 22 は crate closeout 前に `mizar-kernel` の source layout を audit する。この
audit は public module、private helper、tests が [todo.md](./todo.md)、
[source_spec_audit.md](./source_spec_audit.md)、
[internal 07](../../internal/ja/07.crate_module_layout.md) の module responsibility に
従っているかを確認する。

この task の唯一の runtime/module source change は module-local test module の
move-only split である。executable source-layout lint guard は、これらの
private test file を認識するよう更新する。Runtime code、public API、
diagnostics、deterministic rendering、artifact-facing schema、certificate
semantics、rejection semantics、`doc/spec`、
`.miz` fixture、expectation、SAT/ATP/proof search、premise selection、overload
resolution、cluster search、implicit coercion insertion、fallback inference、
global mutable state、downstream ATP/proof/cache/artifact integration は変更しない。

## Source Inventory

Public module set は `src/lib.rs` から export される6つの spec-backed module
だけである: `certificate_parser`, `checker`, `clause`, `rejection`,
`resolution_trace`, `substitution_checker`。

| Parent module | Before task 22 | After task 22 parent | New private test file | Classification | Action |
|---|---:|---:|---:|---|---|
| `certificate_parser` | 2971 lines | 1666 lines | `src/certificate_parser/tests.rs` (1295 lines) | `design_drift` / `source_drift` | inline tests の move-only split。 |
| `checker` | 5412 lines | 2211 lines | `src/checker/tests.rs` (3180 lines) | `design_drift` / `source_drift` | inline tests の move-only split。 |
| `clause` | 1462 lines | 924 lines | `src/clause/tests.rs` (534 lines) | `design_drift` / `source_drift` | inline tests の move-only split。 |
| `rejection` | 1077 lines | 472 lines | `src/rejection/tests.rs` (599 lines) | `design_drift` / `source_drift` | inline tests の move-only split。 |
| `resolution_trace` | 2114 lines | 653 lines | `src/resolution_trace/tests.rs` (1446 lines) | `design_drift` / `source_drift` | inline tests の move-only split。 |
| `substitution_checker` | 4719 lines | 2648 lines | `src/substitution_checker/tests.rs` (2054 lines) | `design_drift` / `source_drift` | inline tests の move-only split。 |

Drift subtype は review-boundary drift である。実装ファイルが trusted runtime code
と大きな test fixture を併せ持ち、public module responsibility を追加していないのに
runtime logic の review を難しくしていた。Tests を private `#[cfg(test)] mod tests;`
file へ移すことで、owning parent module 内に tests を保ったまま review pressure を
下げる。

## Boundary Decision

- Public module は追加、削除、rename しない。
- Task 22 では runtime helper を移動しない。
- `crates/mizar-kernel/src/` の新しい directory は `tests.rs` だけに使う private
  module directory である。
- Source layout lint はこれらの private test files だけを許可し、public module
  exposure 前の paired English/Japanese specs 要求を維持する。
- Task 22 lint guard は private test file と paired `module_boundary_audit.md`
  document が final verification 前に Git index で追跡対象になっていることを
  要求し、untracked split file を commit から漏らせないようにする。
- Task 20 source/spec audit は test traceability が新しい private test files を
  指すよう更新し、lint guard はその traceability path を検査する。
- Task 21 bilingual sync audit は、この新しい paired audit document を含むよう
  更新する。

## Gap Classification

| ID | Class | Evidence | Current action |
|---|---|---|---|
| KERNEL22-G001 | `design_drift` / `source_drift` | Inline `#[cfg(test)] mod tests` blocks made the trusted runtime modules large enough to obscure review boundaries. | Move-only private test-module split で修正する。 |
| KERNEL22-G002 | `external_dependency_gap` / `deferred` | Source-derived certificates、ATP proof translation、cluster/reduction payload producers、derived-fact payload schemas、service envelopes、downstream proof/cache/artifact consumers は crate 外に残る。 | external/deferred classification を保ち、placeholder integration は追加しない。 |
| KERNEL22-G003 | `repo_metadata_conflict` | Task 22 では観測なし。 | 将来 metadata conflict が見つかった場合だけ報告する。unrelated metadata は auto-repair しない。 |

## Verification Plan

Task 22 は意図した behavior change なしに Rust source layout を変更するため、必要な
verification は次のとおり:

- `cargo fmt --check`;
- `cargo test -p mizar-kernel`;
- `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`;
- `git diff --check`;
- explicit path staging 後の `git diff --cached --check`。

Task 22 lint guard は、new private test file と paired module-boundary audit
document が Git index で追跡対象になっていることを検査するため、explicit path
staging 後に再実行する。

Move によって binder contract や checker/trace runtime behavior への source change が
見つからない限り、`cargo test -p mizar-core` と `cargo test -p mizar-checker` は不要で
ある。Task 22 は private test-layout split のみである。
