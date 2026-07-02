# Crate Exit Report: mizar-build

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: complete。

Quality score: 96/100。

Score caps applied: なし。read-only closeout quality review は hard gates passing
かつ score-cap condition なしと判断した。

## Scope

Milestone scope: tasks 0-26 と closeout までの autonomous `mizar-build` crate
development。

Included:

- crate plan、ordered task decomposition、EN/JA module specs、audits。
- phase-0 planning と module-index source。
- deterministic task graph construction。
- scheduler、resource budget、cancellation、failure propagation、cache-aware
  scheduling seam、deterministic artifact commit boundary。
- batch integration、cross-boundary determinism、implemented-seam
  architecture-22 equivalence、bilingual/source-spec audits、module-boundary
  refactor gate。

Excluded:

- real driver-owned build sessions、event streams、phase registry semantics、
  driver-owned `salsa` cache-query integration。
- real `mizar-ir` sealed output handles、output storage、snapshot-handle
  rehydration。
- real producer artifact projection と publication tokens。
- `mizar-cache` `CacheKey`、dependency-fingerprint、proof-reuse validation
  implementation。

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | Pass | [00.crate_plan.md](./00.crate_plan.md)、module specs、[source_spec_correspondence.md](./source_spec_correspondence.md)、[architecture_22_follow_up_audit.md](./architecture_22_follow_up_audit.md) は unresolved blocking/high specification inconsistency がないことを記録する。 |
| Test contract | Pass with deferred low item | focused unit/integration tests は planning、module index、task graph、scheduler、resource、cancellation、failure、cache seam、artifact commit、batch integration、determinism、architecture-22 implemented-seam equivalence を覆う。BUILD-G-016 は documented non-blocking direct-helper `test_gap` として残る。 |
| Traceability | Pass | TODO task records、source/spec audit tables、bilingual audit tables、task-specific reports は implemented behavior を specs/tests へ trace する。`mizar-build` では `.miz` tests や expectations は変更していない。 |
| Design/source sync | Pass | task 22/25/26 audits は source、tests、English design docs、日本語 companion docs を同期している。 |
| Boundary discipline | Pass | `mizar-build` は `mizar-driver` に依存しない。利用できない driver-owned session/query inputs、IR handles、producer-token integrations、owner-provided dispatch inputs は `external_dependency_gap` である。cache hits は execution skips のままで、proof/semantic authority にならない。 |
| Verification | Pass | closeout verification は `cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test` に合格した。task 26 も `cargo test -p mizar-build`、`cargo clippy -p mizar-build --all-targets -- -D warnings`、adjacent cache/artifact/VC/proof tests、`git diff --check` に合格した。 |
| Residual risk | Pass with classified deferrals | 残る risk は BUILD-G-016 `test_gap` と、real driver、IR、producer-token、full real clean/incremental integration に関する external dependency gaps として分類済みである。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 15/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 96/100 |

## Deferred Items

この table は original closeout result を保持し、後続の driver availability が wording を
変える箇所に post-closeout task-27 annotations を追加する。上の closeout gates は
再採点しない。

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| BUILD-G-016 | `sorted_manifest_updates` は `commit_manifest_updates` を通じて間接的に covered だが、standalone ordering の direct focused test がない。 | future artifact-commit hardening task。 | method-level helper coverage を主張する前に direct focused test を追加する。 |
| BUILD-G-002 / BUILD-G-011 | closeout は当初 `mizar-driver` absent を記録していた。task 27 は残る build-owned 部分を scheduler-selected dispatch callback として実装する。driver-owned requests、sessions、event streams、phase registry semantics、cache-query adapter、`salsa` boundary は引き続き `mizar-build` の外側にある。 | completed task-27 dispatch seam と future driver-owned integration phases。 | scheduler-selected callback は `mizar-build` から公開される。remaining owner inputs、producer outputs、publisher handles、artifact tokens は owning crate から提供されなければならず、`mizar-build` に `mizar-driver` dependency を追加しない。 |
| BUILD-G-003 / BUILD-G-012 | real IR sealed output handles、output storage、snapshot rehydration は build-owned seam 経由では利用できない。 | future `mizar-ir` integration phase。 | owning crate から real IR output handles と rehydration boundary を公開し、`mizar-build` では placeholder handles を作らない。 |
| BUILD-G-004 / BUILD-G-013 | real producer artifact publication tokens と full phase-15 emission inputs は利用できない。 | future producer/artifact integration phase。 | real producer publication authority を提供し、`mizar-build` は token を mint せず消費する。 |
| BUILD-G-006 / BUILD-G-015 / BUILD-G-017 | full real resolver/checker/VC/proof/kernel/driver clean/incremental/parallel equivalence は external seams が存在するまで利用できない。 | future external integration phase。 | real driver sessions、IR rehydration、producer projection、publication tokens を接続する。 |
| BUILD-G-009 | driver-owned cache query integration、IR output rehydration、producer publication tokens はまだ存在しない。 | future driver/cache/artifact integration phase。 | driver-owned cache lookup が `mizar-cache` を呼び、`mizar-build` は decisions だけを消費し続ける。 |

## Human Review Surface

Primary human review should inspect:

- [00.crate_plan.md](./00.crate_plan.md)
- [todo.md](./todo.md)
- `doc/design/mizar-build/en/` 配下の module specs
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
- [architecture_22_follow_up_audit.md](./architecture_22_follow_up_audit.md)
- [module_boundary_refactor_gate.md](./module_boundary_refactor_gate.md)
- `crates/mizar-build/src/`
- `crates/mizar-build/tests/`

## Test Expectation Summary

`mizar-build` crate task stream は `.miz` tests、expectation TOML files、
language-spec files を変更していない。Rust tests は `crates/mizar-build` 配下でのみ
追加または移動した。

## Verification

Commands run:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
git diff --check
git diff --cached --check
```

Results: all passed。

Additional task-26 regression commands も合格した:

```text
cargo test -p mizar-build
cargo clippy -p mizar-build --all-targets -- -D warnings
cargo test -p mizar-cache
cargo test -p mizar-artifact
cargo test -p mizar-vc
cargo test -p mizar-proof
```

original closeout 時点では `mizar-driver` が存在せず、`cargo test -p mizar-driver` は
実行できなかった。task 27 は現在利用可能な driver checks と full workspace verification を
dispatch-seam contract について実行し、driver-owned runtime/session authority は
`mizar-build` の外側に保つ。

staged documentation check は、closeout-related paths だけを明示 stage した後の
closeout commit boundary で実行する。

## Handoff

Next recommended work: driver-owned build request/session/phase-registry と
cache-query boundary から始める external integration phase を、別 task stream として
開始する。推奨 reasoning: xhigh。次 phase は crate ownership、incremental
verification、artifact publication、cache reuse、proof-trust boundaries を横断する
ためである。docs-only inventory update なら lower reasoning でもよいが、
implementation または cross-crate design changes では xhigh を維持する。
