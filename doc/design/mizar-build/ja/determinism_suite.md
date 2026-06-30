# Determinism Suite

> 正本言語: 英語。正本:
> [../en/determinism_suite.md](../en/determinism_suite.md).

## 目的

task 20 は、現在実装済みの `mizar-build` seams に対する cross-boundary
determinism coverage を広げる。suite は同一の logical inputs が、input
order、worker count、ready-queue timing、cache decisions、manifest-update
arrival order が変わっても、安定した plans、module indexes、task graphs、
clean scheduler records/events、cache-equivalent public payloads、artifact
manifest commits を生成することを検証する。

この suite は focused module tests の上に置く integration / regression layer
である。driver-owned build sessions、`mizar-ir` handles、cache-key
construction、dependency fingerprint construction、proof-reuse validation、
producer publication tokens は追加しない。

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| BUILD-G-014 | `test_gap` | task 20 以前は planner、module-index、scheduler、cache seam、artifact-commit modules に focused determinism tests はあったが、implemented seams を単一 deterministic pipeline projection として比較する cross-boundary suite はなかった。 | plan/index/graph/scheduler/commit determinism の table-driven integration suite を追加する。 |
| BUILD-G-015 | `external_dependency_gap` | real `mizar-driver` sessions、real `mizar-ir` output handles、producer publication tokens、full clean/incremental build execution は利用できない。 | task 20 は implemented seams に限定し、full clean/incremental equivalence は後続の external integration tasks に残す。placeholders は追加しない。 |

## Boundary Rules

- Clean sequential scheduler runs は deterministic scheduler projections の
  reference である。
- Worker count、synthetic completion order、ready-queue priority hints、
  manifest-update arrival order は、clean implemented-seam runs の canonical
  result/event collation や artifact publication order に影響してはならない。
- Cache hit/miss timing は execution progress と event shape を変え得る。
  validated hit は normal execution を skip するためである。ただし matched
  public payload、committed artifact projection、manifest hash、semantic
  acceptance、proof trust を変えてはならない。
- Cache hits は execution-skip records にすぎない。hit は clean outputs と
  一致し得るが、semantic acceptance、proof authority、producer publication
  authority、trusted-status promotion になってはならない。
- Artifact manifest commits は serialization boundaries であり続ける。
  Manifest updates は `mizar-artifact` へ渡す前に build-side deterministic key
  で sort される。
- suite は `mizar-cache` の `CacheKey`、dependency fingerprint、cache-store
  compatibility、proof-reuse validation logic を再実装してはならない。

## Test Shape

task-20 fixtures は次を覆う。

- shuffled だが logical equivalent な package/source inputs が同一の
  `BuildPlan`、`ModuleIndex`、`TaskGraph` values を生成すること。
- worker counts、priority hints、completion orders が異なる clean scheduler
  runs が同一の canonical `SchedulerRun` results と events を生成すること。
- validated hit が同一 public payload を持つ場合、cache-hit / cache-miss
  decision placement は matched public payload を保ち、clean reference と同一の
  committed artifacts と manifest hashes を残すこと。
- shuffled manifest updates が同一 manifests と build-side commit records を
  commit すること。

利用できない real driver/IR/producer-token paths は `external_dependency_gap` として
記録し、それらの skipped placeholder APIs を追加してはならない。
