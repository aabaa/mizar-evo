# Module: sat_checker

> 正本は英語です。英語版:
> [../en/sat_checker.md](../en/sat_checker.md)。

## 目的

`sat_checker` module は、task 24 が選んだ audited in-process Rust SAT checker の小さな
trusted wrapper を所有する。Task 24 は direct
`batsat = { version = "=0.6.0", default-features = false }` を選択した。完全な
監査は [sat_dependency_audit.md](./sat_dependency_audit.md) にある。これは
kernel-derived SAT problem が unsatisfiable かどうかだけを判定する。

## Trust Statement

この module は trusted kernel code である。SAT checking は、validated
formula/substitution evidence から `sat_encoding` が導出した SAT problem に対してだけ許可される。

この module は ATP search、premise selection、formula selection、substitution invention、
overload resolution、cluster search、implicit coercion insertion、fallback inference、source
loading、cache lookup、artifact lookup、network access、external process execution、
wall-clock / random-state read、unordered iteration dependence、mutable compiler-global state の
hidden read を行ってはならない。

## Wrapper API

Task 27 が期待する wrapper API:

```text
SatCheckInput
  problem
  limits

SatCheckResult
  Unsat
  Sat
  Rejected(reason)
```

Wrapper は model enumeration、proof search configuration、premise minimization、backend
profile、external solver command を expose しない。Limits は variables、clauses、literals、
dependency が support する場合の propagation/conflict steps、canonical input bytes を cover する。

## Dependency requirements

Task 24 は選択した pure-Rust dependency を記録する:

```text
batsat = { version = "=0.6.0", default-features = false }
```

Task 27 はこの dependency を正確に追加し、`batsat` と transitive dependency
`bit-vec` の lockfile resolution を検証し、crate-local dependency lint guard を
更新しなければならない。Audit は version pinning、license、determinism、unsafe
code、transitive dependencies、process/network behavior の不在、resource limits、API
surface、failure mapping を cover する。

Solver error、unsupported clause、limit exhaustion、satisfiable result、internal inconsistency は
non-acceptance outcome である。`Unsat` だけが caller に formula/substitution evidence の受理を
許可する。

## Gap classification

- `test_gap`: task 27 は satisfiable rejection、unsatisfiable acceptance、limit failure、
  solver error、deterministic outcome、external process/network behavior の不在を cover
  する。exact dependency/lockfile lint guard と、wrapper-owned deterministic `batsat`
  heuristic options の pinning / non-exposure も cover しなければならない。
- `source_drift`: task 24 は docs-only であり、`Cargo.toml`、`Cargo.lock`、
  `src/sat_checker.rs` は task 27 が wrapper を統合するまで変更しない。
- `deferred`: `batsat` は public exact conflict/propagation budget setter を持たない。
  task 27 は callback-based deterministic interruption を証明・test するか、unsupported
  step-budget request を拒否しなければならない。
