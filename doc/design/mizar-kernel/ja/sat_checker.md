# Module: sat_checker

> 正本は英語です。英語版:
> [../en/sat_checker.md](../en/sat_checker.md)。

## 目的

`sat_checker` module は、task 24 が選んだ audited in-process Rust SAT checker の小さな
trusted wrapper を所有する。これは kernel-derived SAT problem が unsatisfiable かどうかだけを
判定する。

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

Task 24 は選択した pure-Rust dependency、または dependency を追加しない明示的な audited
decision を記録しなければならない。Audit は version pinning、license、determinism、unsafe
code、transitive dependencies、process/network behavior の不在、resource limits、API surface、
failure mapping を cover する。

Solver error、unsupported clause、limit exhaustion、satisfiable result、internal inconsistency は
non-acceptance outcome である。`Unsat` だけが caller に formula/substitution evidence の受理を
許可する。

## Gap classification

- `test_gap`: task 27 は satisfiable rejection、unsatisfiable acceptance、limit failure、
  solver error、deterministic outcome、external process/network behavior の不在を cover する。
- `repo_metadata_conflict`: task 24 で dependency metadata conflict が見つかった場合は、
  user が明示的に repository metadata repair を許可しない限り report only とする。
