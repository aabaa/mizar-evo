# SAT Dependency Audit: mizar-kernel Task 24

> 正本は英語です。英語版:
> [../en/sat_dependency_audit.md](../en/sat_dependency_audit.md)。

## Scope

Task 24 は、task 27 が wrapper を統合した後に `mizar-kernel` が信頼してよい
Rust SAT checker dependency を選び、監査する。この task は `Cargo.toml`、
`Cargo.lock`、Rust source を編集しない。source integration は、
formula/substitution evidence schema と deterministic SAT encoding task が正確な
in-crate input type を定義するまで意図的に deferred とする。

この依存決定が適用されるのは、kernel が validated formula/substitution
evidence から導出した SAT problem の SAT checking だけである。backend proof
method、resolution trace、SMT proof object、backend log、model、solver
configuration を trusted acceptance material にはしない。

## Decision

Task 27 で追加する selected direct dependency:

```toml
batsat = { version = "=0.6.0", default-features = false }
```

根拠:

- `batsat` は MIT license、build script なし、default feature なしの pure-Rust
  MiniSat 系 SAT solver crate である。
- published library source は external process execution、network access、
  filesystem access、host/environment random-state read、wall-clock read を含まない。
- direct normal dependency set は小さく、`bit-vec` だけである。
- direct `batsat` API は `rustsat-batsat` adapter 経路より小さく、RustSAT の
  public external-solver process interface を避けられる。
- kernel code は deterministic `sat_encoding` output から solver を構築し、
  in-process solver を呼び、`UNSAT` の場合だけ受理できる。

Task 27 はこの dependency を正確に追加し、crate-local dependency lint guard を
更新し、生成された `Cargo.lock` を検証しなければならない。Cargo が異なる
`batsat` version を解決する、feature を有効にする、またはここで監査した版と異なる
`bit-vec` version を解決する場合、task 27 は commit 前に停止し、この監査を
更新しなければならない。

## Audit Sources

監査日: 2026-06-26。

確認した command / artifact:

- `cargo search "SAT solver" --limit 20`
- `cargo info batsat@0.6.0`
- `cargo info bit-vec@0.5.1`
- local cargo registry 内の `batsat-0.6.0` と `bit-vec-0.5.1` の published
  source manifest
- published library source に対する `unsafe`、FFI、process、network、
  filesystem、host/environment random、wall-clock API の grep

local cargo registry で観測した published crate archive checksum:

| Crate | Version | SHA-256 |
|---|---:|---|
| `batsat` | `0.6.0` | `ec82b6bbce8ea42f5003417b699267860a9f4dd869fc9ba8faceac761d5afed1` |
| `bit-vec` | `0.5.1` | `f59bbe95d4e52a6398ec21238d31577f2b28a9d86807f06ca59d191d8440d0bb` |

これらの checksum は audit observation であり、`Cargo.lock` の代替ではない。
commit される lockfile verification は task 27 が所有する。

## Accepted Dependency Metadata

| Crate | Role | Version / requirement | License | Feature policy | Notes |
|---|---|---|---|---|---|
| `batsat` | direct SAT checker dependency | exact `=0.6.0` | MIT | `default-features = false`; `logging` を有効にしない | Pure-Rust MiniSat reimplementation。manifest は `build = false`。normal dependency は `bit-vec = "0.5.0"`。 |
| `bit-vec` | transitive bit-vector storage dependency | `batsat` の `0.5.0` requirement に対する expected lock resolution `0.5.1` | MIT/Apache-2.0 | default `std` のみ | Library source は process/network/filesystem/time/random API を含まない。dev-only benchmark code は `rand` を参照する。 |

License conclusion: MIT と MIT/Apache-2.0 は workspace の MIT metadata と互換で
ある。選択した dependency set について `repo_metadata_conflict` は記録しない。

## Unsafe-Code Audit

kernel crate 自体は `#![forbid(unsafe_code)]` を維持しなければならない。task 27
は `mizar-kernel` に unsafe Rust を追加してはならない。

選択した dependency tree は unsafe-free ではない:

- `batsat` は clause storage representation、literal/extra field の union access、
  raw-slice view、watched-literal update path で `unsafe` を使う。
- `bit-vec` は direct storage access や length mutation などの unsafe method を
  expose する。

source grep では、`batsat` library source に FFI (`extern "C"`), external process
API, network API, filesystem API は見つからなかった。したがって unsafe code は
external solver や host-environment trust expansion ではなく、dependency 内部の
memory-representation risk に留まる。

Task 27 は wrapper を小さく保つ:

- public `mizar-kernel` API に `batsat` type を expose しない;
- `batsat` の DRAT/proof、theory、statistics-printing、logging、model enumeration、
  callback surface を呼び出さない;
- `batsat` の DIMACS parsing/printing、model printing、`print_stats`、file/string
  parser path を呼び出さない。wrapper は kernel-derived `sat_encoding` data
  structure から直接 solver を構築する;
- kernel code から unsafe `bit-vec` API を呼び出さない。

残る risk: `batsat` の memory-safety correctness は trusted SAT checker
dependency の一部である。この correction では、依存が小さく、in-process、
pure Rust、MIT-licensed、process-free で、却下した代替より MiniSat compatibility
に近いため、この risk を受け入れる。将来 upgrade する場合は再監査が必要である。

## Process, Network, Time, And Randomness Audit

選択した dependency path は external solver/process trust edge を作ってはならない。

`batsat` library の published-source grep では、次の使用は見つからなかった:

- `std::process` または `Command`
- `std::net`, `TcpStream`, `UdpSocket`
- `std::fs`, `File`, `OpenOptions`
- `rand`, `thread_rng`, random API
- `std::time`, `Instant`, wall-clock timeout API

`batsat` は `SolverOpts` 内に deterministic seeded pseudo-random heuristic controls
を持つ: `random_var_freq`、`random_seed`、`rnd_pol`、`rnd_init_act`。default source
value は deterministic（`random_var_freq = 0.0`、固定 `random_seed`、`rnd_pol =
false`、`rnd_init_act = false`）だが、task 27 は明示的な options でこれらの値を
pin するか、caller がこれらを変えられる wrapper API を拒否しなければならない。
これらは host random-state read ではないが、solver heuristic であり trusted
input/evidence schema の外に留めなければならない。

`bit-vec` library source も process、network、filesystem、host-random、wall-clock
API を使わない。benchmark/dev material は `rand` を参照するが、production
dependency path には含まれない。

Task 27 は SAT wrapper を wall-clock time から独立させなければならない。resource
limit は deterministic count/size limit だけである。

## Determinism And Wrapper API

Task 27 は direct `batsat` API を `sat_checker` wrapper の背後で使うべきである。
public kernel-facing shape は task 27 が [sat_checker.md](./sat_checker.md) で refine する:

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

Acceptance mapping:

- 完全な kernel-derived problem に対する `batsat::lbool::FALSE` は
  `SatCheckResult::Unsat` に対応する。
- `batsat::lbool::TRUE` は `SatCheckResult::Sat` に対応し、決して acceptance では
  ない。
- `batsat::lbool::UNDEF`、wrapper interruption、unsupported encoding shape、
  solver error、internal inconsistency、limit exhaustion は
  `SatCheckResult::Rejected(reason)` に対応する。

Wrapper は次を expose してはならない:

- model enumeration;
- trusted material としての assumption または unsat-core extraction;
- DRAT/proof production;
- DIMACS parsing/printing または model/statistics printing;
- backend profile name;
- solver command line;
- proof-search configuration knob;
- premise minimization。
- `random_var_freq`、`random_seed`、`rnd_pol`、`rnd_init_act`、restart policy、
  phase-saving setting を含む solver heuristic knob。

## Resource Policy

trusted wrapper は solver construction 前に deterministic input limit 超過を拒否する:

- variable count;
- clause count;
- total literal count;
- maximum clause width;
- canonical SAT input byte length。

`batsat` は conflict/propagation counter と callback-based interruption を expose
するが、exact conflict / propagation budget 用の stable public setter は expose しない。
Task 27 はこの audit の unsupported-step-budget branch を選ぶ。Supported deterministic
input limits だけを expose し、non-`None` の conflict / propagation budget request は
solver construction 前に reject する。

wall-clock timeout へ fallback してはならない。limit exhaustion は常に
non-acceptance である。

## Rejected Candidates

| Candidate | Version inspected | Reason not selected |
|---|---:|---|
| `varisat` | `0.2.2` | MIT/Apache-2.0 で機能的には近いが、published build script が `drat-trim` と `rate` external command を probe する。transitive dependency tree も大きく、unsafe code も含む。 |
| `splr` | `0.17.2` | MPL-2.0 license と default `unsafe_access` feature が policy / trusted-boundary complexity を増やす。 |
| `sat-solver` | `0.2.1` | MIT だが、production dependency に CLI/allocator/walkdir surface があり、source に file/time/random API と広い unsafe usage がある。 |
| `screwsat` | `2.1.5` | MIT かつ dependency-light だが、library source が wall-clock `Instant`/`Duration` timeout API と広い unsafe code を使う。 |
| `oxiz-sat` | `0.2.3` | parallel/GPU/portfolio-related module と default feature complexity を持つ大きな solver surface で、kernel wrapper には過剰。 |
| `microsat` | `0.0.1` | GPL-3.0 license は意図する dependency policy と互換でない。 |
| `rsat` | `0.1.12` | MIT だが、stochastic local search、`rand`、`rayon`、file input、unstable pre-1.0 API surface を含む。 |
| `rustsat-batsat` | `0.7.5` | MIT で adapter としては viable だが、public external-solver process/file/tempfile surface を含む広い RustSAT interface を引き込む。direct `batsat` の方が trusted dependency が小さい。 |

## Dependency And Lint Policy Revision

Task 27 は task-1 dependency guard を次から:

```text
mizar-core
mizar-session
```

正確に次の production dependency set へ改訂する:

```text
batsat = { version = "=0.6.0", default-features = false }
mizar-core = { path = "../mizar-core" }
mizar-session = { path = "../mizar-session" }
```

後続 task が明示的に許可しない限り、guard は dev/build/target dependency section を
引き続き拒否する。さらに、別の SAT checker crate、RustSAT adapter、external solver
wrapper、ATP crate、proof/cache/artifact crate、process-spawning dependency が
`mizar-kernel` に追加されないことを guard しなければならない。

Task 27 は public `sat_checker` source module を統合し、この audit は wrapper が expose
する正確な dependency shape と同期し続けなければならない。Caller-facing API は
`batsat` または `bit-vec` type を expose してはならない。

## Failure Mapping

Wrapper は solver evidence outcome と rejection condition を分離する:

| Wrapper condition | Kernel detail |
|---|---|
| derived SAT problem が satisfiable | `SatCheckResult::Sat(SatCheckReport)`; non-acceptance wrapper evidence |
| accepted UNSAT result なしに dependency が `UNDEF` を返す | 記録された原因に応じて `invalid_sat_refutation` または deterministic budget detail |
| solving 前に input count/size limit を超過 | `resource_exhaustion` |
| unsupported conflict/propagation step-budget request | solver construction 前の `resource_exhaustion` |
| kernel derivation 後の unsupported clause/literal shape | `invalid_sat_refutation` |
| wrapper が捕捉した dependency panic、internal inconsistency、unexpected API result | `invalid_sat_refutation`; 決して acceptance ではない |

Task 27 は wrapper evidence だけを返す。`SatCheckResult::Unsat` は後続 acceptance に
必要だが、それを kernel check service と normal proof-policy acceptance に wire する責務は
task 28 が持つ。

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| KERNEL24-G001 | `deferred` | `batsat` は public exact conflict/propagation budget setter を持たない。 | Task 27 は supported input limit だけを expose し、unsupported step-budget request は solver construction 前に reject する。 |
| KERNEL24-G002 | `source_drift` resolved by task 27 | Task 27 は `sat_checker` source、exact `batsat` manifest dependency、lockfile guard、wrapper test を統合する。 | Dependency and lockfile lint guard を exact に保つ。将来 upgrade する場合は再監査が必要である。 |
| KERNEL24-G003 | `external_dependency_gap` | 新 pipeline 向け formula/substitution evidence candidate を emit する active ATP producer はまだない。 | Kernel test は synthetic のままにし、producer placeholder は追加しない。 |
| KERNEL24-G004 | `deferred` | `batsat` は host random state を読まないが、deterministic pseudo-random heuristic option を expose する。 | Task 27 はすべての `SolverOpts` field を audited default に pin し、heuristic knob を caller に expose しない。 |
