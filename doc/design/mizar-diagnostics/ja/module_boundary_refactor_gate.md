# Module-Boundary Refactor Gate: mizar-diagnostics

> Canonical language: English. English source:
> [../en/module_boundary_refactor_gate.md](../en/module_boundary_refactor_gate.md).

## Scope

この task-21 gate は、source/spec audit と bilingual documentation audit の後に source
layout を監査する。oversized files、mixed responsibilities、public API、diagnostic
behavior、deterministic rendering、artifact-facing schema、consumer-visible behavior を
変えずに split すべき private helper groups を確認する。

## Source Layout Audit

| File or group | Observation | Decision |
|---|---|---|
| `src/registry.rs` | public registry API と大きな spec-22 built-in descriptor table が混在していた。 | descriptor table と local allocation macro を private `src/registry/builtin.rs` へ split した。public `BUILTIN_DESCRIPTORS` は `registry` から re-export される。 |
| `src/failure_record.rs` | public record/draft/span/detail types と validation helpers、deterministic debug rendering helpers が混在していた。 | validation を private `src/failure_record/validation.rs`、debug rendering を private `src/failure_record/debug.rs` へ split した。public record APIs は `failure_record` に残す。 |
| `src/explain.rs` | 大きいが、explanation handles、preview bounds、store resolution、canonical keys、rendering の周辺でまだ cohesive である。 | この task では split しない。explanation behavior が増える場合は future task で validation/debug helpers を split してよいが、task 21 時点で review-blocking mixed responsibility は残っていない。 |
| `src/fix.rs` | 中程度の size で structured fix payloads に cohesive。 | split しない。 |
| `src/aggregator.rs` | 中程度の size で index construction、ordering、dedup、stale accounting に cohesive。 | split しない。 |
| `src/render.rs` | 中程度の size で CLI projection に cohesive。 | split しない。 |
| `src/sink.rs` | small で producer collection に cohesive。 | split しない。 |

## Refactor Result

この task は behavior-preserving private moves だけを行う。

- `registry::BUILTIN_DESCRIPTORS` は同じ path で public に利用できる。
- `failure_record` public types、constructors、accessors、errors、debug snapshot
  strings は変わらない。
- crate root から public module は追加しない。
- diagnostic code、message text、ordering rule、deduplication key、render output、
  fix payload、explanation payload、freshness rule は変更しない。
- LSP、driver、artifact、proof、kernel、cache、producer adoption boundary は追加しない。

moved APIs について source/spec audit scope を再実行した。移動した items は private
helpers または re-exported data であるため、source inventory を更新すれば public API trace は
valid のままである。この task の documentation additions について bilingual documentation
audit scope も再実行し、English/Japanese companions は paired のままである。

## Updated Source Inventory

module table は crate-owned public modules を変更せず、private helper submodules を記録する。

- registry: `src/registry.rs`, private `src/registry/builtin.rs`;
- failure records: `src/failure_record.rs`, private
  `src/failure_record/{validation,debug}.rs`;
- sink: `src/sink.rs`;
- aggregator: `src/aggregator.rs`;
- render: `src/render.rs`;
- fix: `src/fix.rs`;
- explain: `src/explain.rs`.

## Verification

behavior-preserving source move の focused verification:

```text
cargo fmt --check
cargo test -p mizar-diagnostics
cargo clippy -p mizar-diagnostics --all-targets -- -D warnings
git diff --check
git diff --cached --check
```
