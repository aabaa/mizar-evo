# Crate Exit Report: mizar-checker

> 正本は英語です。英語版:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: complete。
Quality score: 94/100。
Score caps applied: none。

## Scope

Milestone scope:

- task 34 の module-boundary gate まで `mizar-checker` crate を構築する。
- phase 6-8 の explicit-payload type checking、registration / cluster trace data
  layer、overload resolution、final resolved typed AST assembly を所有する。
- source-derived semantic corpus execution、proof / artifact acceptance、
  public diagnostic code allocation、downstream CoreIr / ControlFlowIr / VC、
  kernel、proof integration は、owning crate と payload seam が存在するまで
  defer する。

Included:

- English / Japanese checker crate plan と module specification。
- `crates/mizar-checker/src/` 配下の checker Rust source。
- `crates/mizar-checker/tests/lint_policy.rs` の checker policy / audit tests。
- `tests/coverage/spec_trace.toml` の deferred corpus traceability row。

Excluded:

- `doc/spec` への直接 edit。
- 既存 `.miz` fixture や expectation の rebaseline。
- full AST semantic payload の source-to-checker extraction。
- proof search、`VcId` assignment、kernel replay、artifact publication、cache
  reuse、public diagnostic-code registry allocation。

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 0 | `f96dc7c` | `docs(checker-plan): add autonomous crate plan` |
| 1 | `7e5c855` | `feat(checker-task-1): scaffold mizar-checker crate` |
| 2 | `1c5c12e` | `docs(checker-task-2): specify typed ast shape` |
| 3 | `902bfd9` | `feat(checker-task-3): implement typed ast data shapes` |
| 4 | `8443aa1` | `docs(checker-task-4): specify binding environment` |
| 5 | `c9989db` | `feat(checker-task-5): implement binding environment` |
| 6 | `234566c` | `docs(checker-task-6): specify type checker` |
| 7 | `28a679a` | `feat(checker-task-7): implement type normalization` |
| 8 | `9408028` | `feat(checker-task-8): implement declaration checking` |
| 9 | `4731ba4` | `feat(checker-task-9): implement term formula inference` |
| 10 | `fa5cb0e` | `feat(checker-task-10): implement coercion obligations` |
| 11 | `61daf6b` | `feat(checker-task-11): implement type fact queries` |
| 12 | `8860acd` | `feat(checker-task-12): add type elaboration corpus runner` |
| 13 | `358256d` | `docs(checker-task-13): specify registration resolution` |
| 14 | `43473d8` | `feat(checker-task-14): implement registration index` |
| 15 | `27a8dc9` | `docs(checker-task-15): specify cluster trace` |
| 16 | `89a4629` | `feat(checker-task-16): implement cluster trace closure` |
| 17 | `8f00d81` | `feat(checker-task-17): implement cluster saturation bounds` |
| 18 | `e760b7e` | `feat(checker-task-18): record reduction trace steps` |
| 19 | `cb9a66c` | `feat(checker-task-19): validate pending registrations` |
| 20 | `b9cf866` | `feat(checker-task-20): enforce existential gates` |
| 21 | `9f2ad8b` | `docs(checker-task-21): specify overload resolution` |
| 22 | `54c7eec` | `feat(checker-task-22): collect overload candidates` |
| 23 | `61c2fe6` | `feat(checker-task-23): expand template candidates` |
| 24 | `7a53387` | `feat(checker-task-24): filter viable overload candidates` |
| 25 | `9e1539a` | `feat(checker-task-25): build specificity graphs` |
| 26 | `39fdaae` | `feat(checker-task-26): resolve overload selections` |
| 27 | `bb69f8a` | `docs(checker-task-27): specify resolved typed ast` |
| 28 | `426ebdf` | `feat(checker-task-28): assemble resolved typed ast` |
| 29 | `7d37506` | `docs(checker-task-29): record deferred corpus obligations` |
| 30 | `4f887bf` | `test(checker-task-30): add determinism suite` |
| 31 | `ad6f5bc` | `test(checker-task-31): guard enum compatibility policy` |
| 32 | `d6ecfd7` | `test(checker-task-32): audit source spec correspondence` |
| 33 | `30a854e` | `docs(checker-task-33): audit bilingual documentation sync` |
| 34 | `7e64293` | `docs(checker-task-34): record module boundary audit` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | English canonical module specs cover implemented checker-owned behavior; this milestone made no `doc/spec` change. |
| Test contract | passed | 実装済み behavior は task-local Rust tests、lint-policy guards、determinism coverage、または explicit deferred traceability を持つ。 |
| Traceability | passed | `source_spec_audit.md`、`module_boundary_audit.md`、`tests/coverage/spec_trace.toml` が implementation / test / deferred link を記録する。 |
| Design/source sync | passed | Source/spec、bilingual、module-boundary audit guard が public surface、doc pair、source layout を cover する。 |
| Boundary discipline | passed | crate は explicit checker payload boundary を維持し、proof、VC、artifact publication、parser/source extraction を所有しない。 |
| Verification | passed | required broad commands と cached diff check は通過済み。 |
| Residual risk | passed with deferred items | 残りは下の deferred / external dependency gap として分類する。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 18/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 4/5 |
| Handoff quality | 5/5 |
| Total | 94/100 |

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | fix 後、blocking/high/medium finding なし。 |
| Test sufficiency review | blocking/high/medium finding なし。 |
| Full implementation review | fix 後、blocking/high/medium finding なし。 |
| Source/documentation consistency review | fix 後、blocking/high/medium finding なし。 |
| Read-only crate quality review | blocking/high/medium finding なし。score は 94/100、score cap なし。 |

Quality-review residual risk: semantic translation equivalence は mechanical に
証明されておらず human-reviewed のままである。また source-derived checker corpus
coverage は下記 deferred external dependency gap に blocked されている。quality reviewer
は、意図的に deferred した real `.miz` semantic coverage と、final cached diff check が
commit-time gate であることを理由に減点した。

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| MC-G002 | real semantic `.miz` checker coverage は source-to-checker extraction に blocked。 | `mizar-test` / checker extraction follow-up。 | active semantic runner と checker-ready source payload が存在する。 |
| MC-G004 | artifact producer / reuse integration は cross-crate work であり、checker-local schema invention ではない。 | `mizar-artifact` と future checker integration。 | accepted artifact summary producer / reuse path が利用可能。 |
| MC-G005 | public diagnostic code-space は未割り当て。 | `mizar-diagnostics` / checker diagnostic-code task。 | shared diagnostic registry と checker allocation policy が存在する。 |
| MC-G006 | parser/syntax template と scheme role は checker-ready ではない。 | `mizar-parser` / `mizar-syntax` / overload extraction。 | template/scheme role payload が source fabrication なしに利用可能。 |
| MC-G011 | AST-wide binding extraction、use-site scope payload、reserve payload、closure payload が missing。 | resolver/checker extraction follow-up。 | resolver が checker-ready binding payload を emit する。 |
| MC-G014 | AST-wide type-expression と mode/radix/attribute expansion payload が missing。 | resolver/checker extraction follow-up。 | checker-ready type-expression payload extraction が存在する。 |
| MC-G016 | declaration/type-site table、reserve default、RHS/body payload、evidence query が missing。 | checker extraction follow-up。 | source declaration と evidence が checker-owned payload として利用可能。 |
| MC-G017 | term/formula payload table、built-in、candidate signature、source `qua`、sethood evidence が missing。 | checker extraction follow-up。 | checker-ready term/formula/evidence payload が存在する。 |
| MC-G018 | coercion request table、inheritance graph、cluster evidence、proof-query result が missing。 | checker / proof integration follow-up。 | coercion evidence payload と proof-query input が存在する。 |
| MC-G019 | statement/proof assumption、theorem acceptance payload、phase-7 trace fact payload が missing。 | checker / proof integration follow-up。 | statement/proof payload extraction と accepted fact が存在する。 |
| MC-G020 | source-to-checker payload extraction が full semantic pass fixture を block している。 | checker extraction follow-up。 | AST-wide checker payload bridge が存在する。 |
| MC-G021 | registration payload extraction と accepted-status integration が missing。 | checker / artifact / proof integration follow-up。 | checker-ready registration payload と accepted activation status が存在する。 |
| MC-G023 | source-derived cluster/reduction fixture、artifact/cache integration、real trace extraction が missing。 | checker / artifact / cache follow-up。 | source-derived trace payload と artifact/cache integration が存在する。 |
| MC-G025 | accepted registration status の proof/artifact production/import が未接続。 | `mizar-proof` / `mizar-artifact` / checker integration。 | accepted registration status を produce/import できる。 |
| MC-G026 | source-derived existential gate case と artifact reuse が未接続。 | checker / artifact integration follow-up。 | accepted-status integration と source-derived gate payload が存在する。 |
| MC-G027 | source-derived overload payload、diagnostic code allocation、artifact emission/reuse、semantic fixture が missing。 | checker overload extraction / diagnostics / artifact follow-up。 | overload source payload、diagnostic code、artifact path が存在する。 |
| MC-G030 | `formula_statement` と `advanced_semantics` runner/tag support、および source payload extraction が missing。 | `mizar-test` / checker extraction follow-up。 | active runner と checker-ready payload seam が存在する。 |

## Human Review Surface

- `doc/design/mizar-checker/en/` の canonical English specs。
- `doc/design/mizar-checker/ja/` の Japanese companion。
- `crates/mizar-checker/src/` の checker source。
- `crates/mizar-checker/tests/lint_policy.rs` の checker lint / audit guard。
- `tests/coverage/spec_trace.toml` の deferred corpus traceability。

## Test Expectation Summary

既存 `.miz` fixture と expectation sidecar は implementation behavior に合わせて
rebaseline していない。新しい checker behavior は Rust tests と lint-policy guard
で cover し、未利用の semantic corpus coverage は明示的に defer している。

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo clippy -p mizar-checker --all-targets -- -D warnings` | passed |
| `cargo test -p mizar-checker` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | passed |

## Next-Task Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue from the completed mizar-checker autonomous crate milestone. Start the
next crate or integration task from a clean worktree after the closeout commit.
Use the mizar-checker crate exit report, source/spec audit, and deferred MC-G
rows as inputs. Do not fabricate source-to-checker payloads, accepted
registration status, artifact schemas, or active semantic corpus runners; first
select the owning crate/task for the missing seam and follow AGENTS.md with one
task per commit.
```

Rationale: downstream work crosses semantic boundaries into source extraction、
artifact/proof acceptance、diagnostics、later `mizar-core` / `mizar-vc` /
`mizar-kernel` / `mizar-proof` integration へ進むため。docs-only synchronization
や narrow lint-guard maintenance だけなら reasoning を下げてもよい。behavior、
type、trace、artifact、proof-boundary change では `xhigh` を維持または上げる。
