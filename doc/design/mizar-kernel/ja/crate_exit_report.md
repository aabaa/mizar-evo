# Crate Exit Report: mizar-kernel

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: complete。
Quality score: 95/100。
Score caps applied: none。

Closeout 後 correction: commit `c6d94fe51923aa0363ea7297bfe4e9f905aef076`
は task-22 evidence target を supersede する。Tasks 23-34 は corrected
formula/substitution evidence pipeline、trusted in-process SAT checking、legacy path
migration audit、明示的な proof-obligation / consistency goal-polarity binding、
非 import formula source に対する kernel-side context-identity verification、
imported-statement projection validation、legacy tautology-marker audit semantics を
完了する。Task 34 は Step 3 F9 closure point であり、その self-hash は commit 後の
後続 bookkeeping point で記録する。

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

- Kernel-derived SAT problem に対する SAT checking は、task 24 が選択した direct
  `batsat = { version = "=0.6.0", default-features = false }` dependency を通じ、
  task 27 が wrapper を統合し、tasks 25-28 が validated formula/substitution evidence
  から problem を導出した後に限って trusted として許可される。
- Backend proof method、resolution trace、SMT proof object、backend log は trusted
  acceptance material の外に残る。
- Legacy certificate/resolution-trace checking は
  `KernelCheckPolicy.allow_legacy_certificate_audit` の背後に gate される。Default
  normal proof policy は replay 前に拒否する。Explicit audit mode は checked-record
  diagnostics のために replay してよいが、それでも trusted `final_goal` や `used_axioms`
  を持たない `Rejected` を返すため、migration/audit-only である。
- `check_kernel_evidence` は caller が `KernelEvidenceCheckKind` を宣言することを
  要求する。Proof obligation は refutation polarity、consistency check は
  consistency polarity を要求し、accepted consistency check は downstream
  `mizar-proof` policy / selection / status / witness boundary では diagnostic-only /
  non-trusted material として運ばれる。
- `check_kernel_evidence` は local-hypothesis、cited-premise、generated-VC-fact
  formula source に task-28 context-identity payload を要求し、target と documented
  context-identity hash を検証し、各非 import formula entry を immutable source/id と
  fingerprint row に照合し、missing/stale/ambiguous row を SAT encoding 前に拒否する。

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
| 23 | `a326afc7a69913c1d716133620c2c608b78b0ae1` | `docs(kernel-task-23): correct evidence format` |
| 24 | `abc557d5f6f53b6530301a67c29570a23c67b874` | `docs(kernel-task-24): audit trusted SAT checker` |
| 25 | `35ef60ffba949254e71d86f9be2570b37e5f4a3c` | `feat(kernel-task-25): parse formula evidence` |
| 26 | `e48c4ffe78fa03c63f9ed60d4c3f81db95803af9` | `feat(kernel-task-26): encode formula evidence as SAT` |
| 27 | `222bf8bc30e59dd95818d828dd71ff823ff84f83` | `feat(kernel-task-27): wrap trusted SAT checker` |
| 28 | `43674a221dd5f43259c480846db7428f85ac9386` | `feat(kernel-task-28): check formula evidence with SAT` |
| 29 | `0cbcbf01c4b5c2e53c872d6edd35cf38065f90a8` | `fix(kernel-task-29): gate legacy certificate audit` |
| 30 | `f3197e12a8f7a2124da8ebbf0f678cf3cf6bd890` | `fix(kernel-task-30): bind evidence goal polarity` |
| 31 | `a62bae00bb23845e6636c8b39cebb9043898cc03` | `fix(kernel-task-31): verify context identity` |
| 33 | `0f3d7fa316cffbf7e55722fd255cb3fbf32d9249` | `feat(kernel-task-33): validate imported statement projections` |
| 34 | pending self-hash | `fix(kernel-task-34): pin legacy tautology marker` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Module spec、source/spec audit、bilingual sync audit、module-boundary audit、closeout review は unresolved blocking/high specification inconsistency がないことを記録する。 |
| Source behavior documented or deferred | passed | Public module、public item、test、promised behavior は `source_spec_audit.md` に trace され、unsupported source-derived / downstream behavior は silent に実装せず分類済み。 |
| Milestone-owned coverage | passed | Crate-local Rust test は canonical clause、certificate parsing、rejection record、resolution replay、substitution/alpha/FV replay、imported fact、cluster/reduction replay、checker orchestration、goal-polarity/check-kind binding、context-identity verification、determinism、replay cost、public enum policy、soundness mutation failure を cover する。 |
| Test expectation integrity | passed | 既存 `.miz` fixture または expectation sidecar は implementation behavior に合わせるため変更していない。Source-derived certificate corpus support は明示的に deferred のまま。 |
| Design/source synchronization | passed | Paired source/spec、bilingual、public enum、soundness、determinism、module-boundary audit は source layout と public module table に同期している。 |
| Boundary discipline | passed | task-22 legacy milestone は evidence だけを check し、SAT solver を含まない。Post-correction tasks は、kernel-derived SAT problem に対する task-24 audit 済み in-process SAT checker だけを追加できる。ATP backend、proof search、proof-policy projection、cache/artifact coupling、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state read は含まない。 |
| Verification | passed | Task-31 focused / crate-local check、broad clippy/test、diff check、cached-diff check は commit 前に passed。 |
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
| Implementation specification / documentation review | Task 34 spec/doc review は、premise-weakening tautology mislabeling と final-goal rejection を混同する medium wording issue と、service-test rejection-shape の low issue を最初に指摘した。Paired docs はこれらを分離し、marker mislabeling は premise を弱めるだけ、marker final goal は `invalid_sat_proof` として拒否されることを明記した。Service test は final-goal location の `kernel_rejection/invalid_sat_proof` を assert する。 |
| Test sufficiency review | Task 34 test review は finding なし。Resolution-trace test は marker replay と final-goal rejection を cover し、checker service test は explicit audit mode が successful-audit wrapping 前に marker final goal を拒否することを cover する。対称的な resolution-step marker final-goal case は任意かつ non-blocking。 |
| Full implementation review | Initial task 34 full implementation review は task-ledger と crate-exit-report bookkeeping drift の medium findings を指摘した。Task ledger、crate exit report、handoff と stale な source/spec-audit task reference を更新し、final re-review は finding なし。 |
| Source/documentation consistency review | Finding なし。EN/JA docs、task ledger、TODO state、handoff、Rust tests は task-34 audit-only tautology-marker semantics で一致している。Task 34 は spec coverage ownership、traceability metadata、owner crate、deferred coverage classification を変えないため、`doc/design/spec_coverage_audit.md` は unchanged のまま。 |
| Read-only crate quality review | Task 34 後も post-correction hard gate は満たされる。trusted acceptance は parsed formula/substitution evidence、kernel-derived SAT checking、proof-obligation polarity binding、verified context identity、imported-statement projection validation だけを通る。Legacy tautology marker は audit-only であり、trusted final goal や used axiom を populate できない。残る source-runner、producer、proof-policy、cache/artifact、より豊かな substitution gap は分類済みであり、valid quality score は 95/100 のまま 90 以上。 |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| KERNEL-CLOSEOUT-G001 | `external_dependency_gap` | formula/substitution evidence を feed する active source-to-kernel-evidence runner または `.miz` proof-verification corpus がない。 | Source-derived formula/substitution evidence corpus fixture を有効化する前に、owning staged-test/source-to-kernel-evidence runner を追加する。 |
| KERNEL-CLOSEOUT-G002 | `external_dependency_gap` | `mizar-atp` は active formula/substitution evidence candidate producer ではない。MiniSAT-compatible backend trace は legacy migration/audit material であり trusted output ではない。 | VC handoff contract が存在した後、candidate formula/substitution evidence production を中心に ATP crate を構築する。Trusted backend proof translation は追加しない。 |
| KERNEL-CLOSEOUT-G003 | `external_dependency_gap` | `mizar-proof` は accepted proof-obligation `KernelCheckResult` を status、trusted used-axiom、witness-boundary fixture level で消費するようになったが、より豊かな proof-policy projection と externally authenticated evidence policy は downstream に残る。 | `mizar-proof` の crate plan と consumer contract の下で proof-policy consumer を拡張する。 |
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
| `cargo fmt --check` | passed |
| `cargo test -p mizar-kernel tautology_outcomes_follow_the_active_clause_profile -- --nocapture` | passed |
| `cargo test -p mizar-kernel kernel_service_rejects_legacy_tautology_marker_final_goal_in_audit_mode -- --nocapture` | passed |
| `cargo test -p mizar-kernel source_spec_audit_covers_public_surface_and_prohibitions` | passed |
| `cargo test -p mizar-kernel` | passed |
| `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | explicit task-34 path staging 後に passed |

Unrun deferred commands:

- なし。上記 commands は current workspace を cover する。

## Next-Task Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue Step 3 with `mizar-proof` task 21 after the completed mizar-kernel
task-34 legacy tautology-marker semantics. Before editing, verify a clean
worktree, confirm the mizar-kernel task 34 commit in git log, and read
doc/design/todo.md,
doc/design/mizar-proof/en/todo.md,
doc/design/mizar-proof/en/00.crate_plan.md,
doc/design/mizar-proof/en/status.md,
doc/design/mizar-proof/en/witness.md,
doc/design/mizar-kernel/en/crate_exit_report.md,
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/checker.md,
doc/design/mizar-kernel/en/formula_evidence.md,
doc/design/mizar-kernel/en/resolution_trace.md,
doc/design/mizar-kernel/en/source_spec_audit.md,
doc/design/mizar-kernel/en/soundness_argument.md,
doc/design/mizar-vc/en/kernel_evidence_handoff.md,
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md,
doc/design/internal/en/07.crate_module_layout.md,
doc/design/architecture/en/08.reasoning_boundary.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/18.dependency_fingerprint.md,
doc/design/architecture/en/19.failure_semantics.md, and
tests/coverage/spec_trace.toml. Begin with mizar-proof task 21: align proof
policy consumers with the corrected kernel rejection taxonomy for F1/F2/F6/F9
without weakening proof-obligation-only trust, fabricating kernel payloads,
activating unverified fixtures, publishing proof rows, or rebaselining
expectations to current implementation behavior. Preserve one task per commit.
```

Rationale: `mizar-proof` task 21 は kernel F9 closure の後に来る、指定 Step 3
順序の次 task である。Trusted kernel proof-obligation acceptance と rejection taxonomy を
proof-policy consumer に投影する作業であり、legacy acceptance や fabricated bridge payload に
戻ってはならないため `xhigh` を維持する。typo-only documentation sync だけなら下げてよい。
repository metadata や specification contradiction が handoff を block する場合だけ上げる。
