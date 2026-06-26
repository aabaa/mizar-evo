# Crate Exit Report: mizar-kernel

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: complete。
Quality score: 95/100。
Score caps applied: none。

Closeout 後 correction: commit `c6d94fe51923aa0363ea7297bfe4e9f905aef076`
は task-22 evidence target を supersede する。この report は legacy resolution-trace
milestone の closeout record として残る。Tasks 23-29 は formula/substitution evidence
pipeline、trusted in-process SAT checking、legacy path migration audit のために crate を
再開する。

## Scope

Milestone scope:

- `mizar-kernel` crate を preliminary task 0 から task 22 と closeout task まで
  構築する。
- task-22 legacy phase-14 surface を所有する: normalized certificate parsing、
  canonical clause validation、MiniSAT-compatible resolution trace replay、
  substitution/alpha/free-variable replay、imported fact checking、explicit
  cluster/reduction trace replay、deterministic check-service orchestration。
- immutable normalized certificate と explicit kernel context を evidence として
  だけ消費する。構文解析や backend success は、それ自体では決して trust を
  与えない。
- SAT solving を task-22 legacy milestone の外に保ち、ATP backend execution、proof search、premise selection、overload
  resolution、cluster search、implicit coercion insertion、fallback inference、
  global mutable compiler state、proof-policy projection、cache lookup、artifact
  publication、未完成 producer/consumer integration は crate 外に保つ。

Post-correction scope:

- Kernel-derived SAT problem に対する SAT checking は、task 24 が dependency/wrapper を
  audit し、tasks 25-28 が validated formula/substitution evidence から problem を導出した後に
  限って trusted として許可される。
- Backend proof method、resolution trace、SMT proof object、backend log は trusted
  acceptance material の外に残る。
- Legacy certificate/resolution-trace acceptance は、task 29 が normal proof policy 向けに
  gate または retire するまで `source_drift` である。

Included:

- `doc/design/mizar-kernel/{en,ja}/` 配下の英日 crate plan、module specification、
  audit、closeout report。
- `crates/mizar-kernel/src/` 配下の Rust source。
- `crates/mizar-kernel/` 配下の crate-local unit test と lint-policy test。
- 残る gap を mock せず分類する source/spec、bilingual、public enum、
  determinism、soundness、module-boundary audit。

Excluded:

- task-22 legacy milestone のための `doc/spec` への直接編集。
- 既存 `.miz` fixture または expectation sidecar の rebaseline。
- Source-derived formula/substitution evidence corpus fixture または
  source-to-kernel-evidence runner。
- task-22 legacy milestone のための SAT solver または ATP backend implementation。
- Proof-policy projection、proof witness publication、cache hit acceptance、
  artifact validation。
- 未完成の `mizar-atp`、`mizar-proof`、`mizar-cache`、`mizar-artifact` seam との
  placeholder integration。
- Resolver/checker mutable state の hidden use、implicit coercion insertion、
  overload resolution、cluster search、fallback inference、repair heuristic。

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 0 | `81ffb5561fc1b24ae355d216e1a455d2a487d923` | `docs(kernel-task-0): add autonomous crate plan` |
| 1 | `63cbcd83a82005d8ffe98f7c87928fa46e95649c` | `feat(kernel-task-1): scaffold mizar-kernel crate` |
| 2 | `b0fa89a9eecc85da96bf8351fc2e147423747730` | `docs(kernel-task-2): specify clause representation` |
| 3 | `4020ac12fafe24aa8205f7fd3df8ece37027804e` | `feat(kernel-task-3): add clause representation` |
| 4 | `b900639e4057ea2ba1a1158688a35e188ec991ec` | `docs(kernel-task-4): specify certificate parser` |
| 5 | `60c92cc53c77ec3240fe5410fc04c449bd04b267` | `feat(kernel-task-5): add certificate parser` |
| 6 | `f4b1abc63a46cd7d628911aff4a7ce91c0c5555b` | `docs(kernel-task-6): specify rejection records` |
| 7 | `acc8e7d62adbee21cb49b8d134fe0d846ee60603` | `feat(kernel-task-7): add rejection records` |
| 8 | `0b017553b3462eb78492d3aa84053b9d07a2fae4` | `docs(kernel-task-8): specify resolution trace replay` |
| 9 | `28b7e7122c8cad04a6526d8de8cdfd0394d8bb3c` | `feat(kernel-task-9): add resolution trace replay` |
| 10 | `d79506c6e0b7029fb1512454b0eff72579362df7` | `docs(kernel-task-10): specify substitution checker` |
| 11 | `b97c4a3a700fec986d3e203b1a88d23edcfba7f3` | `feat(kernel-task-11): add substitution checker` |
| 12 | `577f6f220b93d94c9796208829216f43a8e2e3d4` | `feat(kernel-task-12): add alpha and free-variable checks` |
| 13 | `865231081df7538faea132c499d9c57d5ecfa9cb` | `docs(kernel-task-13): specify checker orchestration` |
| 14 | `874881b42d5c008336a34cb4cfaf24f7b403a1fb` | `feat(kernel-task-14): add imported fact checking` |
| 15 | `77262c0ec36071bdab8ac5c1b22d14a4537ae68a` | `feat(kernel-task-15): add cluster trace replay` |
| 16 | `c0b8e6104f38d02e7bf8f6c1cda5900fb50bdfc1` | `feat(kernel-task-16): add kernel check service` |
| 17 | `b7e1493050ed49110e4ddf7a7a75d971bdf72c59` | `test(kernel-task-17): add soundness fail corpus` |
| 18 | `3d1942e97ea245d2fae09dac4e26cefd67c02bd1` | `test(kernel-task-18): add determinism replay-cost suite` |
| 19 | `981fa7a05fe8de11168bd862d81cbd7d486347c0` | `test(kernel-task-19): guard public enum policy` |
| 20 | `fb81213c33d5b2a31eb976a4fa6804bfc0ffe6c5` | `docs(kernel-task-20): audit source spec correspondence` |
| 21 | `73a919c16b48da82038fd7267e86e1a844cb4c6f` | `docs(kernel-task-21): audit bilingual docs sync` |
| 22 | `814e47bb9aaaff75ebfe4cc1be10d2eb4618498b` | `refactor(kernel-task-22): split module test boundaries` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Module spec、source/spec audit、bilingual sync audit、module-boundary audit、closeout review は unresolved blocking/high specification inconsistency がないことを記録する。 |
| Source behavior documented or deferred | passed | Public module、public item、test、promised behavior は `source_spec_audit.md` に trace され、unsupported source-derived / downstream behavior は silent に実装せず分類済み。 |
| Milestone-owned coverage | passed | Crate-local Rust test は canonical clause、certificate parsing、rejection record、resolution replay、substitution/alpha/FV replay、imported fact、cluster/reduction replay、checker orchestration、determinism、replay cost、public enum policy、soundness mutation failure を cover する。 |
| Test expectation integrity | passed | 既存 `.miz` fixture または expectation sidecar は implementation behavior に合わせるため変更していない。Source-derived certificate corpus support は明示的に deferred のまま。 |
| Design/source synchronization | passed | Paired source/spec、bilingual、public enum、soundness、determinism、module-boundary audit は source layout と public module table に同期している。 |
| Boundary discipline | passed | `mizar-kernel` は evidence だけを check する。SAT solver、ATP backend、proof search、proof-policy projection、cache/artifact coupling、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state read は含まない。 |
| Verification | passed | Closeout broad command、paired-document link/count check、diff check は commit 前に pass した。 |
| Residual risk | passed with classified items | 残る risk は下で `external_dependency_gap` または `deferred` として分類する。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 19/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 3/5 |
| Total | 95/100 |

減点理由は source-derived certificate corpus coverage が利用不能であること、external
certificate producer と downstream proof/cache/artifact consumer が未完成であること、
この milestone に real backend-generated MiniSAT trace がないこと、Task 22 の
test-module split 後も大きめの parent implementation file が maintenance watchlist に
残ることである。これらは分類済みで、crate-local milestone の所有 seam ではなく hard
gate failure でもないため score cap はない。

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | High invalid Task 15 ledger hash finding を full commit hash の backfill で修正した。final focused re-review は blocking/high/medium finding なし。 |
| Test sufficiency review | Medium paired-document link/check recording finding を deterministic pair/link check の実行と記録で修正した。final focused re-review は blocking/high/medium finding なし。 |
| Full implementation review | blocking/high/medium finding なし。Low provisional verification と abbreviated Task 4 hash notes は commit 前に修正した。 |
| Source/documentation consistency review | Medium provisional status/verification drift finding は closeout review と verification 完了後に修正した。final focused re-review は blocking/high/medium finding なし。 |
| Read-only crate quality review | Hard gates は pass し、blocking/high/medium finding はない。Valid quality score は 95/100 で 90 以上。 |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| KERNEL-CLOSEOUT-G001 | `external_dependency_gap` | formula/substitution evidence を feed する active source-to-kernel-evidence runner または `.miz` proof-verification corpus がない。 | Source-derived formula/substitution evidence corpus fixture を有効化する前に、owning staged-test/source-to-kernel-evidence runner を追加する。 |
| KERNEL-CLOSEOUT-G002 | `external_dependency_gap` | `mizar-atp` は active certificate/trace producer ではなく、real MiniSAT-compatible backend trace は stable producer contract として利用不能。 | ATP crate と trace extraction contract を構築する。kernel は normalized trace だけを replay し続ける。 |
| KERNEL-CLOSEOUT-G003 | `external_dependency_gap` | `mizar-proof` は `KernelCheckResult` の active policy consumer ではない。proof-status projection と externally authenticated evidence policy は downstream のまま。 | `mizar-proof` に crate plan と consumer contract つきで proof-policy consumer を追加する。 |
| KERNEL-CLOSEOUT-G004 | `external_dependency_gap` | `mizar-cache` と `mizar-artifact` は kernel output 用の active proof-cache/proof-witness consumer contract を提供しない。 | downstream cache/artifact phase が validation と publication contract を定義してから kernel coupling を追加する。 |
| KERNEL-CLOSEOUT-G005 | `external_dependency_gap` / `deferred` | Source-derived certificate/service envelope、derived-fact payload schema、service-envelope normalization、cancellation token plumbing、external worker scheduling は crate 外の integration concern。 | upstream/downstream contract が存在してから producer/consumer task を追加する。ここでは placeholder を追加しない。 |
| KERNEL-CLOSEOUT-G006 | `external_dependency_gap` / `deferred` | `mizar-checker` cluster/reduction payload production と、より rich な semantic redex/LHS-to-RHS producer validation は trusted kernel 外に残る。 | Kernel replay は explicit normalized commitment に限定する。source-side cluster payload producer は owning crate に追加する。 |
| KERNEL-CLOSEOUT-G007 | `deferred` | Local-abbreviation closure/type-guard evidence、captured-free-variable closure evidence、inline substitution payload encoding、algorithm id 1 を超える digest registry expansion、downstream wildcard-arm checks は future compatibility task のまま。 | Owning producer / consumer contract が要求する場合、独立した spec-backed task を追加する。 |
| KERNEL-CLOSEOUT-G008 | `deferred` | Task 22 後に parent runtime module はかなり小さくなったが、`checker`、`substitution_checker`、`certificate_parser` は大きい trusted module のまま。 | reviewability bottleneck が生じた場合だけ future move-only maintenance task で private runtime helper を分割する。behavior や API change を混ぜない。 |

`repo_metadata_conflict` は観測されなかった。

## Human Review Surface

- `doc/design/mizar-kernel/en/` 配下の英語正本。
- `doc/design/mizar-kernel/ja/` 配下の日本語 companion。
- `crates/mizar-kernel/src/` 配下の kernel source。
- `crates/mizar-kernel/` 配下の kernel lint-policy と unit test。
- Upstream/downstream context:
  `doc/design/mizar-vc/en/crate_exit_report.md`,
  `doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md`,
  `doc/design/internal/en/07.crate_module_layout.md`,
  `doc/design/architecture/en/08.reasoning_boundary.md`,
  `doc/design/architecture/en/15.kernel_certificate_format.md`,
  `doc/design/architecture/en/16.substitution_and_binding.md`,
  `doc/design/architecture/en/19.failure_semantics.md`。

## Test Expectation Summary

既存 `.miz` fixture や expectation sidecar は implementation behavior に合わせるために
変更していない。Milestone-owned behavior は Rust unit test、lint-policy guard、
soundness mutation test、determinism/replay-cost test、source/spec audit、または
explicit deferred row で cover される。Source-derived semantic corpus coverage は上記の
external runner / producer gap により blocked のまま。

## Verification Commands

| Command | Result |
|---|---|
| deterministic paired-document and companion-link check | passed |
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | explicit closeout path staging 後に passed |

Unrun deferred commands:

- なし。上記 broad workspace commands は current workspace を cover する。

## Next-Task Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Start the next verification pipeline crate after the completed mizar-kernel
closeout. Before editing, verify a clean worktree, confirm the mizar-kernel
closeout commit in git log, and read
doc/design/mizar-kernel/en/crate_exit_report.md,
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-atp/en/todo.md,
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md,
doc/design/internal/en/07.crate_module_layout.md,
doc/design/architecture/en/09.atp_interface_protocol.md,
doc/design/architecture/en/10.atp_backend_integration.md, and
doc/design/architecture/en/15.kernel_certificate_format.md. Begin with
preliminary task 0 for mizar-atp: create or update the paired English/Japanese
crate plan, classify specification gaps, test gaps, source/design drift,
external dependencies, and deferred items, and commit that plan as its own
task. Preserve the one-task-one-commit rule; do not scaffold mizar-atp source
until the task-0 plan commit exists.
```

Rationale: この task-22 handoff は tasks 23-29 で追跡する post-closeout correction に
supersede される。`mizar-atp` は trusted normalized certificate や
MiniSAT-compatible trace ではなく、candidate formula/substitution evidence を emit
しなければならない。external backend execution、evidence projection、kernel
soundness、downstream proof-policy boundary をまたぐため `xhigh` を維持する。
typo-only documentation sync だけなら下げてよい。repository metadata や
specification contradiction が crate plan を block する場合だけ上げる。
