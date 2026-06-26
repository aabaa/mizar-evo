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
共通 lint vocabulary では、no SAT solving は caller-supplied、backend-supplied、または
search-oriented な SAT problem solving を行わないことを意味する。この module は
kernel-derived problem に対する trusted in-process acceptance check だけを行う。

この module は no proof search、no ATP search or backend invocation、no premise
selection、no formula selection、no substitution invention、no overload resolution、no
cluster search、no implicit coercion insertion、no fallback inference、no acceptance from
backend-reported success alone、no source loading、no cache lookup、no artifact lookup、no
network access、no external process execution、no wall-clock or random-state reads、no
unordered iteration dependence、no hidden reads of mutable compiler-global state を守る。

## Wrapper API

Task 27 が実装する wrapper API:

```text
SatCheckContext
  limits: SatCheckLimits

SatCheckLimits
  max_variables
  max_clauses
  max_literals
  max_literals_per_clause
  max_canonical_bytes
  max_conflicts = unsupported unless None
  max_propagations = unsupported unless None

SatCheckResult
  Unsat(SatCheckReport)
  Sat(SatCheckReport)
  Rejected(RejectionRecord)
```

Public entry point は `check_sat_problem(problem, context)` であり、`problem` は
`sat_encoding` が生成し、wrapper には read-only accessor だけで expose される
`EncodedSatProblem` である。Wrapper は `batsat` type、
model enumeration、proof search configuration、premise minimization、backend
profile、solver heuristic option、external solver command を expose しない。DRAT/proof
production、theory solver surfaces、callback surfaces、DIMACS parsing/printing、
model/statistics printing、`print_stats`、file parser paths、string parser paths も call
または expose しない。Limits は solver 構築前に variables、clauses、literals、clause
width、canonical input bytes を cover する。

Task 27 は dependency audit のうち callback interruption を使わない選択肢を採る。
`batsat` 0.6.0 には stable exact public conflict/propagation budget setter がないため、
non-`None` の `max_conflicts` または `max_propagations` request は solver construction 前に
deterministically reject する。Exact solver step budget は、dependency が stable
deterministic API を expose するまで `deferred` のままである。

## Dependency requirements

Task 24 は選択した pure-Rust dependency を記録する:

```text
batsat = { version = "=0.6.0", default-features = false }
```

Task 27 はこの dependency を正確に追加し、`batsat 0.6.0` と exact audited
transitive dependency `bit-vec 0.5.1` の lockfile resolution を検証し、crate-local
dependency lint guard を更新しなければならない。Audit は version pinning、license、determinism、unsafe
code、transitive dependencies、process/network behavior の不在、resource limits、API
surface、failure mapping を cover する。

Solver error、unsupported clause、limit exhaustion、satisfiable result、internal inconsistency は
non-acceptance outcome である。Task 27 は wrapper evidence だけを返す。`Unsat` を kernel
check service と normal proof-policy acceptance に wire する責務は task 28 が持つ。それまでは
`Unsat` は necessary acceptance evidence だが、単体では legacy service path を変更しない。

Task 27 は wrapper 内で `batsat::SolverOpts` の全 field を audited 0.6.0 default に pin し、
random and heuristic surface について明示的に assert する: `random_var_freq = 0.0`、
`random_seed = 91648253.0`、`rnd_pol = false`、`rnd_init_act = false`、
`phase_saving = 2`、`luby_restart = true`、`restart_first = 100`、`restart_inc = 2.0`。
これらの deterministic heuristic controls は test するが、`SatCheckContext` や evidence には
expose しない。

## Gap classification

- `test_gap`: task 27 は satisfiable rejection、unsatisfiable acceptance、limit failure、
  solver error、deterministic outcome、external process/network behavior の不在を cover
  する。exact dependency/lockfile lint guard と、wrapper-owned deterministic `batsat`
  heuristic options の pinning / non-exposure も cover しなければならない。
- `source_drift`: task 24 は docs-only であり、`Cargo.toml`、`Cargo.lock`、
  `src/sat_checker.rs` は task 27 が wrapper を統合するまで変更しない。
- `deferred`: `batsat` は public exact conflict/propagation budget setter を持たない。
  task 27 は wall-clock timeout や unstable callback accounting を使わず、unsupported
  step-budget request を拒否する。
