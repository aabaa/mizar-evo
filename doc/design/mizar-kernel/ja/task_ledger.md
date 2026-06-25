# Task Ledger: mizar-kernel

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は `mizar-kernel` crate 自律作業の再開地点である。task を開始する
前に `git status`、`git log`、この表、[todo.md](./todo.md) を確認する。
task は commit が履歴に存在し、final review outcome、verification result、
deferred reason が分かるまで完了ではない。commit は自身の最終 hash を含め
られないため、self-hash は次 task 開始前に `git log` で確認し、後続の
bookkeeping commit または closeout task で反映する。

| Task | Status | Commit | Reviews | Verification | Deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | complete | `81ffb5561fc1b24ae355d216e1a455d2a487d923` | Spec/doc review: low pending-status finding fixed; final re-review no findings. Test sufficiency review: medium `--all-features` and conditional cross-crate verification findings fixed; final re-review no findings. Full implementation review: high sequencing and medium cluster-gate/status findings fixed; final re-review no findings. Source/doc consistency review: medium internal-04 rejection-reason and low JA companion-link findings fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Docs-only。paired crate plan と ledger を作成し、初期 `spec_gap`, `test_gap`, `design_drift`, `source_drift`, `external_dependency_gap`, `deferred`, `repo_metadata_conflict` 状態を分類し、kernel 禁止事項、trusted-baseline lint policy、strict linear task sequencing、internal-04 rejection reason coverage、cluster trace external-readiness gate を記録する。crate source は作らない。 |
| 1. Crate scaffold and trusted-baseline lint policy | complete | `63cbcd83a82005d8ffe98f7c87928fa46e95649c` | Spec/doc review: medium public-surface and dependency-escape findings fixed; low TODO/ledger timing finding resolved by final ledger update. Test sufficiency review: medium dependency-subtable and low workspace-member scanner findings fixed; final re-review no findings. Full implementation review: high task-0 hash, medium dependency-subtable, medium split-public-surface, and medium extern-ABI public-surface findings fixed; final re-review no findings. Source/doc consistency review: medium dependency-guard and low trusted-baseline decision findings fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Scaffold-only。workspace member、lockfile entry、最小 crate manifest、`#![forbid(unsafe_code)]` crate-root trust statement、lint-policy guard を追加する。Production dependency は `mizar-core` と `mizar-session` の完全一致。dev/build/target dependency section、public semantic surface、downstream ATP/proof/cache/artifact coupling、module spec、semantic module、`.miz` fixture、expectation、`doc/spec` edit は存在せず scope 外のまま。 |
| 2. Spec: `clause.md` | complete | `b0fa89a9eecc85da96bf8351fc2e147423747730` | Spec/doc review: high empty-clause, medium test-coverage, low trust-prohibition, and medium validation-context signature findings fixed; final re-review no findings. Test sufficiency review: medium planned-test coverage, medium hash-test coverage, and low symbol-kind ordering coverage findings fixed; final re-review no findings. Full implementation review: high empty-clause plus medium tautology-marker, validation-context, and canonical-ordering findings fixed; final re-review no findings. Source/doc consistency review: medium validation-context signature finding fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only。literal、canonical ordering、structural well-formedness、explicit empty / tautology form、clause-local validation context、deterministic rendering/hash inputs、trust statement、planned task-3 tests、clause-specific gaps の paired clause module specs を追加する。Rust source、`.miz` fixture、expectation、`doc/spec` edit はない。 |
| 3. Implement clause representation | complete | `4020ac12fafe24aa8205f7fd3df8ece37027804e` | Spec/doc review: medium public `Term` ordering と medium clause-hash preallocation finding を修正し、final re-review は blocking/high/medium finding なし。Test sufficiency review: high canonical-order coverage、medium hash-field / hash-exclusion coverage、low marker/single/empty coverage findings を修正し、final re-review は finding なし。Full implementation review: medium `Term` ordering、medium unchecked length casts、low missing `#[non_exhaustive]`、medium preallocation resource-bound findings を修正し、final re-review は blocking/high/medium finding なし。Source/doc consistency review: medium `Term` ordering drift を修正し、final re-review は blocking finding なし。ledger/TODO backfill はこの task で完了。 | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task。`clause` data model、validation context、deterministic ordering/rendering/hash input、明示的 empty と zero-payload tautology marker form、checked canonical byte framing、大きな allocation 前の resource-bound validation、module exposure、lint guard update を実装する。SAT/ATP/proof search、downstream ATP/proof/cache/artifact coupling、`.miz` fixture、expectation、`doc/spec` edit はない。binder contract または checker/trace boundary に触れない clause-only task のため、cross-crate `mizar-core` / `mizar-checker` tests は不要。 |
| 4. Spec: `certificate_parser.md` | complete | `b900639e4057ea2ba1a1158688a35e188ec` | Spec/doc review: high concrete-byte/schema gap と imported proof-status gap、medium hash-algorithm / failure-location findings を修正し、その後の high imported-fact id と medium non-parent `ClauseRef` findings も修正した。final re-review は blocking/high/medium finding なし。Test sufficiency review: high stable-failure / hash-coverage findings と medium reference/resource/ordering findings を修正し、final re-review は finding なし。Full implementation-boundary review: high concrete-byte gap と medium generated-clause/hash-dependency findings を修正し、final re-review は blocking/high/medium finding なし。Source/doc consistency review: medium task-3 backfill と schema-ownership/TODO findings を修正し、final re-review は blocking finding なし。 | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only。schema ownership、concrete schema/encoding v1 envelope、section tags、directory と item payload layout、parser-owned manifests、public clause integration、stable failure detail/location mapping、digest dependency を持たない hash-input rules、planned task-5 tests、`external_dependency_gap` / `deferred` records を含む paired certificate parser specs を追加する。Rust source、`.miz` fixture、expectation、`doc/spec` edit はない。 |
| 5. Implement certificate parsing and structural validation | complete | `60c92cc53c77ec3240fe5410fc04c449bd04b267` | Spec/doc review: range out-of-bounds と noncanonical/shuffled hash-input semantics の EN/JA clarification 後、final re-review は finding なし。Test sufficiency review: medium location/hash/resource/ordering gaps と、その後の medium resolution-step ordering gap を修正し、final re-review は finding なし。Full implementation review: high resource preallocation / generated-literal budget findings、medium term-budget / item-location findings を修正し、final re-review は finding なし。Source/doc consistency review: final re-review は finding なし。pre-commit bookkeeping のみ expected とされた。 | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-core` と `cargo test -p mizar-checker` は、この task が `mizar-core` binder contract や checker/trace boundary semantics を変更せず、certificate bytes を decode して既存の local `clause` API に generated clause validation を委譲するだけであるため実行していない。 | Rust source task。schema/encoding v1 envelope parsing、deterministic fixed-section directory validation、item-frame parsing、parser-owned manifest/reference/schema types、`clause` 経由の generated clause structural validation、stable rejection category/detail/location reporting、canonical hash input bytes、大きな allocation 前の parser resource limits、expanded lint coverage を実装する。paired certificate parser docs は実装済み range/hash semantics の明確化のみ更新する。SAT/ATP/proof search、producer/cache/artifact coupling、`.miz` fixture、expectation、`doc/spec` edit はない。External producer/consumer integration は `external_dependency_gap` / `deferred` のまま。 |
| 6. Spec: `rejection.md` | complete | `f4b1abc63a46cd7d628911aff4a7ce91c0c5555b` | Spec/doc review: medium EN/JA planned-test sync、context-mismatch/profile ambiguity、target sort-key、cluster mapping、witness-normalization ownership、ordering-id findings を修正し、final re-review は finding なし。Test sufficiency review: medium ordering、checker-location、category/detail ownership、parser target-key fallback、`clause_ref`、checker mapping findings を修正し、final re-review は finding なし。Full implementation-boundary review: medium profile ambiguity、imported proof-status、internal-04 reason coverage、target sort-key ownership、witness-normalization ownership、planned mapping wording findings を修正し、final re-review は finding なし。Source/doc consistency review: high context-mismatch drift と medium domain-separator mapping finding を修正し、final re-review は finding なし。 | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only。`certificate_rejection` / `kernel_rejection`、stable detail keys、parser/checker mappings、target-owned deterministic record ordering、evidence locations、compatibility policy、planned task-7 tests、internal-04 `missing_provenance` / `malformed_witness_data` coverage を定義する paired rejection specs を追加する。certificate parser docs も unsupported domain separator mapping を含むよう同期する。Rust source、`.miz` fixture、expectation、`doc/spec` edit はない。Downstream proof/cache/artifact consumers は `external_dependency_gap` / `deferred` のまま。 |
| 7. Implement rejection records | complete | `acc8e7d62adbee21cb49b8d134fe0d846ee60603` | Spec/doc review: medium fixed-width target sort-key と evidence-id ordering/doc-sync findings を修正し、final re-review は finding なし。Test sufficiency review: medium parser conversion、category/detail mapping、ordering、lint coverage findings と low atomic-token / isolated category/byte-offset/section ordering findings を修正し、final re-review は finding なし。Full implementation review: high public-field constructor-bypass finding を修正し、final re-review は finding なし。Source/doc consistency review: low reduction/derived/final ordering coverage finding を修正し、final re-review は finding なし。 | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-core` と `cargo test -p mizar-checker` は、この task が `mizar-core` binder contract や checker/trace replay boundary を変更せず、既存 parser data 上に共有 rejection record type と parser-error conversion を追加するだけであるため実行していない。 | Rust source task。stable rejection category/detail、category/detail ownership validation、target VC fingerprint ordering、structured evidence location、parser-error conversion、read-only rejection record、deterministic total ordering、module exposure、trusted-boundary lint expansion を実装する。paired rejection docs は実装済み ordering details のみ同期する。SAT/ATP/proof search、fallback inference、implicit coercion insertion、global mutable state、downstream proof/cache/artifact coupling、`.miz` fixture、expectation、`doc/spec` edit はない。Downstream proof/cache/artifact consumers は `external_dependency_gap` / `deferred` のまま。 |
| 8. Spec: `resolution_trace.md` | complete | `0b017553b3462eb78492d3aa84053b9d07a2fae4` | Spec/doc review: medium imported-parent context / final-goal checkedness findings を修正し、初期の stale sequencing concern は clean status/log で解決した。final re-review は finding なし。Test sufficiency review: medium rejection-record shape / provenance planned-test findings と low final-goal checkedness coverage finding を修正し、final re-review は finding なし。Full implementation-boundary review: high preallocation / term-depth findings と medium clause-context、clause-owned helper、imported-context validation、resource-classification findings を修正し、final re-review は finding なし。Source/doc consistency review: medium internal-04 `MissingProvenance` drift と low remaining internal-04 rejection-detail drift を修正し、final re-review は finding なし。 | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only。deterministic MiniSAT-compatible replay、explicit imported clause context、clause-reference ownership、pivot orientation、bounded resolvent construction、clause-owned non-allocating/depth-bounded helper requirements、final-goal checkedness、replay rejection mapping、planned task-9 tests、`external_dependency_gap` / `deferred` records を定義する paired resolution-trace specs を追加する。internal-04 EN/JA `RejectionReason` sketch を `MissingProvenance`、`MalformedWitnessData`、`InvalidClusterTrace` と同期する。Rust source、`.miz` fixture、expectation、`doc/spec` edit はない。Backend proof translation と proof/cache/artifact consumers は `external_dependency_gap` / `deferred` のまま。 |
| 9. Implement resolution trace checker | ready to commit | pending self-hash | Spec/doc review: medium clause-helper documentation と low imported-context/report-shape findings を修正し、final re-review は finding なし。Test sufficiency review: generated/imported/previous-step parent resource ordering、report binding、generated/resolution-step final goal、nested canonical-length helper coverage、stable rejection coverage を final re-review まで確認し finding なし。Full implementation review: high pre-clone resource-ordering findings と medium report binding、previous-step parent clone-order、stale docs findings を修正し、final re-review は finding なし。Source/doc consistency review: low `ResolutionTraceInput` target-fingerprint sketch finding を修正し、final re-review は finding なし。 | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task。deterministic resolution trace replay、explicit imported clause context、checked-step report、private replay binding 付き final-goal helper、stable kernel rejection mapping、clone 前の bounded borrowed parent validation、bounded resolvent accumulation、`clause` の term-depth / canonical-length helpers、cfg(test) certificate fixture construction、module exposure、lint guard update を実装する。SAT solver、ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state read、downstream ATP/proof/cache/artifact coupling、`.miz` fixture、expectation、`doc/spec` edit はない。Imported-fact availability/fingerprint/proof-status auditing、backend proof translation、checker orchestration、proof/cache/artifact consumers は `external_dependency_gap` / `deferred` のまま。 |
| 10. Spec: `substitution_checker.md` | not started | pending | pending | pending | task 9 commit が必要。Semantic dependency: task 4 certificate spec。Spec-only。 |
| 11. Implement substitution checking | not started | pending | pending | pending | task 10 commit が必要。Semantic dependency: task 7 rejection records。Rust source task。 |
| 12. Implement alpha-conversion and free-variable checks | not started | pending | pending | pending | task 11 commit が必要。Rust source task。 |
| 13. Spec: `checker.md` | not started | pending | pending | pending | task 12 commit が必要。Semantic dependencies: task 6 rejection spec、task 8 resolution spec、task 10 substitution spec。Spec-only。 |
| 14. Implement imported-fact checking | not started | pending | pending | pending | task 13 commit が必要。Rust source task。 |
| 15. Implement cluster trace replay | not started | pending | pending | pending | task 14 commit が必要。Semantic dependency: task 13 checker spec と external `mizar-checker` cluster trace payload readiness review または deferred record。Rust source task。 |
| 16. Kernel check service and deterministic batch ordering | not started | pending | pending | pending | task 15 commit が必要。Semantic dependencies: task 9 resolution checker、task 12 substitution checker、task 14 imported-fact checking、task 15 cluster replay。Rust source task。 |
| 17. Soundness fail-test corpus | not started | pending | pending | pending | task 16 commit が必要。Test/audit task。source-derived corpus runner gap は `external_dependency_gap` として残り得る。 |
| 18. Determinism and replay-cost suite | not started | pending | pending | pending | task 17 commit が必要。Semantic dependency: task 16 checker service。Test task。 |
| 19. Public-enum forward-compatibility policy | not started | pending | pending | pending | task 18 commit が必要。Semantic dependency: task 16 public API surface。Test/docs task。 |
| 20. Source/spec correspondence and prohibition audit | not started | pending | pending | pending | task 19 commit が必要。Audit task。 |
| 21. Bilingual documentation sync audit | not started | pending | pending | pending | task 20 commit が必要。Docs audit task。 |
| 22. Module-boundary refactor gate | not started | pending | pending | pending | task 21 commit が必要。Audit または move-only task。 |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | task 22 commit、全 hard gate pass、read-only quality review score >= 90/100 が必要。 |

## Task 9 handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 9
resolution-trace checker commit. Before starting task 10, verify a clean
worktree, confirm the task 9 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/mizar-kernel/en/clause.md,
doc/design/mizar-kernel/en/rejection.md,
doc/design/mizar-kernel/en/resolution_trace.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/16.substitution_and_binding.md,
doc/design/architecture/en/19.failure_semantics.md, and
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md.
Implement task 10 only: write paired English/Japanese substitution checker
specs at doc/design/mizar-kernel/en/substitution_checker.md and
doc/design/mizar-kernel/ja/substitution_checker.md. Specify substitution
application validation, alpha-conversion checking, free-variable side
conditions, binder-context evidence, stable rejection mapping, planned task-11
and task-12 tests, and independent re-checking of the mizar-core binder
contract without reusing resolver/checker mutable state. Do not add Rust source
and do not implement substitution checking, alpha checking, free-variable
checking, proof search, ATP search, premise selection, overload resolution,
cluster search, implicit coercion insertion, fallback inference, global mutable
state reads, or downstream proof/cache/artifact integration. Run git diff
--check and git diff --cached --check after explicit path staging. Use
review-only agents for the required AGENTS.md review phases and commit task 10
by itself.
```

Rationale: task 10 は、後続の Rust 実装 task が従う binder-sensitive checker を
仕様化する。Substitution、alpha conversion、free-variable side condition は
soundness-critical であり、inference ではなく explicit certificate replay に
留める必要があるため `xhigh` を維持する。typo-only docs sync だけなら低い
reasoning でよい。architecture 16 と現在の certificate schema が矛盾する場合だけ
上げる。

## Task 8 handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 8
resolution-trace spec commit. Before starting task 9, verify a clean worktree,
confirm the task 8 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/resolution_trace.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/mizar-kernel/en/clause.md,
doc/design/mizar-kernel/en/rejection.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/19.failure_semantics.md, and
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md.
Implement task 9 only: add the resolution trace replay checker, expose its
module, and add focused tests for valid replay, pivot polarity, resolvent
mismatch, imported-clause context/provenance, checked final-goal behavior,
stable rejection records, deterministic output, and replay resource limits.
Add only the small clause-owned non-allocating canonical length / bounded-writer
and depth-bounded validation helper(s) needed by the spec, plus a crate-private
certificate helper only if needed to derive the replay validation context from
public parsed data. Do not implement SAT solving, ATP proof translation, proof
search, imported-fact availability checking beyond explicit context validation,
substitution checking, cluster replay, checker orchestration, proof/cache/
artifact integration, fallback inference, implicit coercion insertion, or
global mutable state reads. Run cargo fmt --check, cargo test -p mizar-kernel,
cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases and commit
task 9 by itself.
```

Rationale: task 9 は trusted kernel 内で最初の semantic replay checker を実装する。
allocation bound、depth bound、parent orientation、stable rejection location は
soundness-critical なので `xhigh` を維持する。comment-only follow-up だけなら下げて
よい。既存 clause/parser API が指定 helper boundary を広い design update なしに
実現できない場合だけ上げる。

## Task 7 handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 7
rejection-records commit. Before starting task 8, verify a clean worktree,
confirm the task 7 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/rejection.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/19.failure_semantics.md, and
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md.
Implement task 8 only: write paired English/Japanese resolution-trace specs
doc/design/mizar-kernel/en/resolution_trace.md and
doc/design/mizar-kernel/ja/resolution_trace.md. Specify deterministic
MiniSAT-compatible resolution trace replay/checking over the normalized
certificate schema, clause-reference ownership, antecedent/pivot validation,
linear replay/resource bounds, stable rejection mapping through
invalid_sat_proof/resource_exhaustion/missing_provenance, and explicit
kernel prohibitions. Do not add Rust source and do not implement a SAT solver,
ATP backend, proof search, premise selection, overload resolution, cluster
search, implicit coercion insertion, fallback inference, or global mutable
state. Run git diff --check and git diff --cached --check after explicit path
staging. Use review-only agents for the required AGENTS.md review phases and
commit task 8 by itself.
```

Rationale: task 8 は task 9 の trusted checker が実装する replay contract を
定義する。trace replay は soundness boundary の一部であり、search や solver
ではなく決定的な evidence checker に保つ必要があるため `xhigh` を維持する。
typo-only docs sync だけなら低い reasoning でよい。architecture 15 または
既存 certificate schema が planned trace model と矛盾する場合だけ上げる。

## Task 6 handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 6
rejection spec commit. Before starting task 7, verify a clean worktree, confirm
the task 6 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/rejection.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/19.failure_semantics.md, and
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md.
Implement task 7 only: add src/rejection.rs, expose it from src/lib.rs, and
add focused tests for stable category/detail keys, parser conversion preserving
target_vc_fingerprint and locations, deterministic ordering, allowed and
disallowed category/detail mappings, #[non_exhaustive] public enums, and
trusted-boundary lint coverage. Do not implement resolution, substitution,
imported-fact, cluster, or checker-service replay logic beyond record types and
test fixtures. Run cargo fmt --check, cargo test -p mizar-kernel,
cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases and commit
task 7 by itself.
```

Rationale: task 7 は、後続 checker が使う stable failure vocabulary を shared
record surface にする。category/detail ownership、target-owned ordering、parser
conversion は trusted soundness boundary の一部なので `xhigh` を維持する。
typo-only docs sync だけなら低い reasoning でよい。parser API または
architecture 19 が spec と矛盾する場合だけ上げる。

## Task 5 handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 5
certificate parser implementation commit. Before starting task 6, verify a clean
worktree, confirm the task 5 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/architecture/en/15.kernel_certificate_format.md, and
doc/design/architecture/en/19.failure_semantics.md. Implement task 6 only:
write paired English/Japanese rejection semantics specs
doc/design/mizar-kernel/en/rejection.md and
doc/design/mizar-kernel/ja/rejection.md. Define stable parser/checker rejection
categories and structured details/locations without adding Rust source. Keep the
kernel as an evidence checker: no proof search, ATP search, premise selection,
overload resolution, cluster search, implicit coercion insertion, fallback
inference, or global mutable compiler state. Run git diff --check and
git diff --cached --check after explicit path staging. Use review-only agents
for the required AGENTS.md review phases and commit task 6 by itself.
```

Rationale: task 6 は後続のすべての checker が使う安定した rejection vocabulary
を仕様化する。rejection category は trusted boundary と architecture 19 の
failure-semantics compatibility policy の一部なので `xhigh` を維持する。
typo-only synchronization だけなら低い reasoning でよい。architecture documents
が既存 parser rejection surface と矛盾する場合だけ上げる。

## Task 0 handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 0
crate-plan commit. Before starting task 1, verify a clean worktree, confirm the
task 0 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/internal/en/07.crate_module_layout.md,
doc/design/architecture/en/08.reasoning_boundary.md,
doc/design/architecture/en/15.kernel_certificate_format.md, and
doc/design/architecture/en/19.failure_semantics.md. Implement task 1 only: add
the mizar-kernel workspace member, minimal crate manifest, crate-root trust
statement, and trusted-baseline lint-policy guard. Keep production dependencies
limited to mizar-session and mizar-core, forbid unsafe code, and do not expose
semantic modules until paired module specs exist. Run cargo fmt --check,
cargo test -p mizar-kernel, cargo clippy -p mizar-kernel --all-targets
--all-features -- -D warnings, git diff --check, and git diff --cached --check
after explicit path staging. Use review-only agents for the required AGENTS.md
review phases.
```

Rationale: task 1 は、後続の kernel 作業が依存する trusted crate boundary と
dependency guard を作る。dependency discipline、trusted lint policy、
no-search/no-ATP boundary は soundness-critical なので `xhigh` を維持する。
typo-only documentation cleanup だけなら低い reasoning でもよい。repository
metadata や矛盾仕様が scaffold を block する場合だけ上げる。
