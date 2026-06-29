# Crate Exit Report: mizar-atp

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the current candidate-evidence producer milestone.
Quality score: 94/100.
Score caps applied: none.

Post-closeout metadata correction: after `mizar-proof` was formally
scaffolded and completed as a workspace crate, this report was corrected to
stop classifying it as a placeholder. The ATP milestone still does not depend
on `mizar-proof` or implement proof policy locally.

## Scope

Milestone scope:

- Build the `mizar-atp` workspace crate from task 1 through task 28.
- Own phase 13 candidate production for `VcStatus::NeedsAtp` obligations:
  backend-neutral `AtpProblem` construction, deterministic TPTP/SMT-LIB
  encodings, generic backend process execution, mock candidate classification,
  no-early-stop portfolio candidate collection, run metadata, and
  deterministic candidate handoff.
- Produce untrusted formula/substitution candidate payloads as the only
  kernel-checkable proof evidence from this crate. Counterexample and
  externally attested records remain untrusted diagnostics or policy-facing
  records, not trusted acceptance material.
- Keep proof acceptance, SAT checking, kernel calls, proof policy, winner
  selection, witness publication, proof-cache promotion, artifact writing,
  source-derived corpus extraction, and unavailable real-backend extraction
  classified as deferred or external dependency gaps.

Included:

- English/Japanese crate plan, module specifications, source/spec audit,
  bilingual sync audit, module-boundary audit, and this exit report under
  `doc/design/mizar-atp/{en,ja}/`.
- Rust source and private unit-test modules under `crates/mizar-atp/src/`.
- Crate-local lint, mock-backend corpus, and determinism tests under
  `crates/mizar-atp/tests/`.
- Metadata-only advanced-semantics corpus fixtures under `tests/property/`
  and traceability rows in `tests/coverage/spec_trace.toml`.

Excluded:

- Direct edits to `doc/spec`.
- Rebaselining existing `.miz` fixtures or expectation sidecars.
- Real backend adapters, backend-output parsers, or backend-specific
  formula/substitution candidate extraction.
- Trusted acceptance from backend proof methods, resolution traces, SMT proof
  objects, backend logs, instantiated formulas, or caller-supplied SAT
  problems.
- Kernel checking, trusted SAT checking, proof search, premise selection,
  substitution invention, overload resolution, cluster search, implicit
  coercion insertion, fallback inference, proof-policy winner selection,
  artifact witness publication, or proof-cache promotion.
- Placeholder `mizar-cache` crates or local proof-policy adapters. `mizar-proof`
  is now a formal downstream workspace crate, but this ATP milestone still does
  not depend on it or move proof policy into `mizar-atp`.

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 1 | `402020d2f8166c7ede5af6d4518711e3fdbba402` | `feat(atp-task-1): scaffold candidate evidence crate` |
| 2 | `9da4f2a6676937249035c19d3312574e5ea10334` | `docs(atp-task-2): specify backend-neutral problems` |
| 3 | `77a189282d5fccb8219f0070700ed933c5d08ca1` | `feat(atp-task-3): implement backend-neutral problems` |
| 4 | `3efa26876bf500248a8e588c0534a01cdec08756` | `docs(atp-task-4): specify vc translator` |
| 5 | `5ea8d2e823c90b739492b9bb7c72ea1b34a43d5a` | `feat(atp-task-5): translate declarations` |
| 6 | `6f00b61fd960ded7dc79db9ee5c3274fc5431425` | `feat(mizar-atp-task-6): translate axioms and conjectures` |
| 7 | `3bcd6ccee921c2325dac412d151493aeb97f8186` | `docs(mizar-atp-task-7): specify property encoding` |
| 8 | `975323520f65a35299eca645aca3060eb92da549` | `feat(mizar-atp-task-8): encode property axioms` |
| 9 | `06c42fff17c7cac070345eb1fb532669d26fbb6d` | `docs(mizar-atp-task-9): specify tptp encoder` |
| 10 | `0fe5bd55e62e1c7261097a2f6b06ae06149d5242` | `feat(mizar-atp-task-10): emit deterministic tptp fof` |
| 11 | `6af92de8fbe97157901e93cd69af9b43ccfa955c` | `docs(mizar-atp-task-11): specify smtlib encoder` |
| 12 | `90e96e87cdca02b670a92d710a212333759263d2` | `feat(mizar-atp-task-12): emit deterministic smtlib` |
| 13 | `2e77fa8854595756393fe93b10fc65731c40d945` | `docs(mizar-atp-task-13): specify backend runner` |
| 14 | `2f4f91f6c845c21dfb953eb11107b65b1457faa0` | `feat(mizar-atp-task-14): add backend runner` |
| 15 | `16582227516cfe961afdca6072aae4f9e7c9e6e5` | `docs(mizar-atp-task-15): defer concrete backend route` |
| 16 | `c3abdaab8f5e8ae9d61603199ea1ef83e392fb5a` | `docs(mizar-atp-task-16): defer real-output classification` |
| 17 | `4263b63e3c1f888a51531f593a39a197fb1649b4` | `docs(mizar-atp-task-17): specify portfolio handoff` |
| 18 | `e4e71979c928ebafcd41eaf6667c09b67a26e450` | `feat(mizar-atp-task-18): collect portfolio candidates` |
| 19 | `8cef987e782a7a4a46e853bc0b9bd42950729081` | `feat(mizar-atp-task-19): record backend run metadata` |
| 20 | `3b9d07fc619d324b021c10aba8467183924a7aa4` | `test(mizar-atp-task-20): add mock backend corpus suite` |
| 21 | `4d18e3f7de6f90600861d96c29e9cba54dfeecca` | `test(mizar-atp-task-21): add determinism suite` |
| 22 | `4269f43240bd4a5de003a695acebd7b894544f85` | `docs(mizar-atp-task-22): record enum forward compatibility` |
| 23 | `d3be92b92b9de8e810f6a31d2e0836d0e54561f5` | `docs(mizar-atp-task-23): add source spec audit` |
| 24 | `9d053bb129bc846733c430e2fafac44eb1c2c89b` | `docs(mizar-atp-task-24): add bilingual sync audit` |
| 25 | `93a480bde30dbb12014681ba1bac2300587b5a06` | `docs(mizar-atp-task-25): defer portfolio order gate` |
| 26 | `5dc6ab594c2ced35d8afc74e2c01707929a569d6` | `docs(mizar-atp-task-26): audit architecture order gate` |
| 27 | `f896d48f3ea4f915084343a4f88007b77f9941cb` | `refactor(mizar-atp-task-27): split private test modules` |
| 28 | `be55e0e80f8b6e5a0079b40554c4fa56c9d2fda7` | `docs(mizar-atp-task-28): add crate exit report` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Paired module specs, source/spec audit, bilingual sync audit, module-boundary audit, and closeout review record no unresolved blocking/high specification inconsistency. |
| Source behavior documented or deferred | passed | Public modules, public entry functions, public enums, and promised behavior are traced in `source_spec_audit.md`; unavailable real-backend and downstream behavior is classified instead of mocked. |
| Milestone-owned coverage | passed | Crate-local tests cover problem validation, translation, property encoding, concrete encoders, backend runner behavior, portfolio collection, metadata projection, public enum policy, mock corpus coverage, and determinism. |
| Test expectation integrity | passed | No existing `.miz` fixture or expectation sidecar was changed to match implementation behavior. Active source-derived ATP execution remains deferred. |
| Design/source synchronization | passed | Source/spec, bilingual, Architecture-22 follow-up, and module-boundary audits match the source layout and public module table. |
| Boundary discipline | passed | `mizar-atp` produces untrusted candidates only; it does not call the kernel, run SAT checking, accept proofs, select trusted winners, publish witnesses, or promote cache records. |
| Verification | passed | Crate-local Rust commands, broad workspace commands, and diff checks passed before the closeout commit. |
| Residual risk | passed with classified items | Remaining risks are listed below as `external_dependency_gap` or `deferred`. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

The score deducts for the intentionally deferred real-output extraction route,
missing active source-derived ATP corpus execution, and unavailable
proof/cache/artifact consumers. These items do not cap the score because they
are outside this crate's current milestone and are explicitly classified.

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | No findings for task 27 after paired module-boundary docs and audits were synchronized. |
| Test sufficiency review | Initial low finding that task-27 doc marker checks mentioned only two moved test modules was fixed by guarding all seven private test module paths. Final re-review found no findings. |
| Full implementation review | No findings. The task-27 split preserves cfg gates, include paths, public API, and trust boundaries. |
| Source/documentation consistency review | No findings. Source layout, EN/JA docs, and lint-policy expected file sets agree. |
| Read-only crate quality review | Hard gates pass with no blocking/high findings. Valid quality score: 94/100, which is >= 90. |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| ATP-CLOSEOUT-G001 | `external_dependency_gap` | No paired real-output extraction spec/source module maps concrete backend output to kernel-parseable formula/substitution candidate payloads, and no supported real backend executable route is verified in this environment. | Add backend-specific EN/JA extraction specs, guarded fixtures, and candidate mapping that excludes backend proof material before reopening tasks 15-16. |
| ATP-CLOSEOUT-G002 | `external_dependency_gap` | `mizar-proof` is now the workspace proof-policy owner, but this ATP milestone still has no integration that calls its release-policy finality, deterministic winner selection, proof-status projection, or witness-selection APIs. | Wire `mizar-atp` to formal `mizar-proof` APIs only in a separate integration task; do not add proof policy to `mizar-atp`. |
| ATP-CLOSEOUT-G003 | `external_dependency_gap` | `mizar-cache` is design-only and not a workspace crate, so proof-reuse validation, cache lookup, and policy-compatible reuse are unavailable. | Scaffold and complete `mizar-cache` under its own TODO only after explicit authorization; cache reuse must never upgrade evidence. |
| ATP-CLOSEOUT-G004 | `external_dependency_gap` / `deferred` | Real artifact witness publication and proof/cache consumer integration remain incomplete. `mizar-artifact` owns witness schema/projection, not ATP acceptance. | Downstream proof/cache/artifact owners connect checked kernel evidence to published witness refs with their own specs. |
| ATP-CLOSEOUT-G005 | `deferred` | Active `.miz` advanced-semantics execution and source-derived ATP extraction are unavailable; task 20 uses metadata-only corpus anchors. | Add the staged runner and source extraction contracts before replacing metadata-only coverage with active corpus coverage. |
| ATP-CLOSEOUT-G006 | `deferred` | Typed TPTP/CNF/include routes, SMT arithmetic/sorted signatures/options/proof commands, native declarations, and backend-native shortcuts remain unsupported by current specs. | Add paired specs and tests for each concrete extension before implementation. |

The stale task-28 statement that `mizar-proof` was a design-only non-workspace
placeholder was a `repo_metadata_conflict` after the proof crate milestone. It
is resolved by this metadata correction; no unresolved repo metadata conflict
remains.

## Human Review Surface

- Canonical English docs under `doc/design/mizar-atp/en/`.
- Japanese companions under `doc/design/mizar-atp/ja/`.
- ATP source under `crates/mizar-atp/src/`.
- ATP tests under `crates/mizar-atp/tests/` and module-local Rust tests.
- Metadata-only corpus fixtures under `tests/property/` and traceability rows
  in `tests/coverage/spec_trace.toml`.
- Upstream/downstream context:
  `doc/design/mizar-kernel/en/crate_exit_report.md`,
  `doc/design/mizar-vc/en/crate_exit_report.md`,
  `doc/design/mizar-artifact/en/todo.md`,
  `doc/design/mizar-proof/en/todo.md`,
  `doc/design/mizar-cache/en/todo.md`, and
  `doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md`.

## Test Expectation Summary

No existing `.miz` fixtures or expectation sidecars were changed to match
implementation behavior. Milestone-owned behavior is covered by Rust unit
tests, integration tests, lint-policy guards, determinism tests, mock-backend
corpus tests, source/spec audits, or explicit deferred gap records. Active
source-derived semantic corpus coverage remains blocked by the external runner
and extraction gaps listed above.

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo test -p mizar-atp --test lint_policy --offline` | passed |
| `cargo test -p mizar-atp --offline` | passed |
| `cargo clippy -p mizar-atp --all-targets --all-features --offline -- -D warnings` | passed |
| `cargo clippy --all-targets --all-features --offline -- -D warnings` | passed |
| `cargo test --offline` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | passed after explicit task-28 path staging |

Post-closeout metadata correction verification:

- The table above is retained as historical task-28 closeout evidence.
- The metadata correction that updates the `mizar-proof` workspace status reran
  `cargo fmt --check`, `cargo test -p mizar-atp --test lint_policy`,
  `cargo test -p mizar-atp`,
  `cargo clippy -p mizar-atp --all-targets -- -D warnings`, `cargo test`,
  `git diff --check`, and `git diff --cached --check` before the correction
  commit.

Unrun deferred commands:

- `cargo test -p mizar-cache` was not run because `mizar-cache` is not a
  workspace member. `mizar-proof` is now a workspace member and is covered by
  its own crate milestone and by current full-workspace verification.

## Next-Phase Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue the evidence-pipeline correction after the mizar-atp task 28 closeout
commit exists. First verify `git status --short` is clean and that
`f896d48f3ea4f915084343a4f88007b77f9941cb` plus
`be55e0e80f8b6e5a0079b40554c4fa56c9d2fda7` are in HEAD history. Do not create a
placeholder `mizar-cache` crate, and do not move proof policy into `mizar-atp`.
Use the formal `mizar-proof` APIs only in a separate ATP/proof integration
task. Treat proof-cache validation, real backend output extraction, real
artifact witness publication, and active source-derived ATP corpus execution as
external_dependency_gap / deferred until their owner specs and workspace crates
exist.
```
