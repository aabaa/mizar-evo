# Batch Integration Suite

> 正本言語: 英語。正本:
> [../en/batch_integration.md](../en/batch_integration.md).

## 目的

task 19 は、driver/IR owner seams と real producer publication token を
`mizar-build` がまだ消費できない段階で利用できた build-side integration path を対象にする。

1. deterministic な `BuildPlan` を生成する。
2. plan と source-layout provider から `ModuleIndex` を構築する。
3. index を `TaskGraph` へ展開する。
4. scheduler を batch mode で実行する。
5. caller-supplied artifact manifest updates を `mizar-artifact` 経由で
   commit する。

この suite は既存の `mizar-build` 境界の integration check である。
driver、IR、cache-key construction、dependency fingerprint construction、
proof-reuse validation、producer artifact publication integration の代替ではない。

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| BUILD-G-010 | `source_drift` / `test_gap` | task 19 以前は planner、module-index、task-graph、scheduler、cache seam、artifact-commit behavior に focused tests はあったが、plan -> graph -> schedule -> commit を単一 batch fixture で覆っていなかった。 | 利用可能な public APIs をまとめて実行する integration test を追加する。 |
| BUILD-G-011 | historical `external_dependency_gap`; task-27 `source_drift` / `test_gap` | earlier batch integration は、driver-owned requests、sessions、phase registry、event stream、salsa boundaries が `mizar-build` の外側にあったため entry-point agnostic だった。task 27 は driver consumption 用の scheduler-selected callback seam だけを追加する。 | integration test は entry-point agnostic に保つ。driver-owned APIs や `mizar-driver` dependency を追加しない。 |
| BUILD-G-012 | `external_dependency_gap` | real phase output handles と snapshot-handle rehydration は build-owned seam 経由では利用できない。 | scheduler integration では scheduler-owned synthetic output refs だけを使う。placeholder IR handles を作らない。 |
| BUILD-G-013 | `external_dependency_gap` | real producer artifact publication tokens と full phase-15 emission inputs は `mizar-build` に公開されていない。 | caller-supplied `ModuleArtifactEntry` values だけを `mizar-artifact` 経由で commit する。tokens を作らず、producer authority を仮造しない。 |

## Boundary Rules

- Batch integration は crate がすでに所有する public `mizar-build` APIs を
  使わなければならない。
- cache hit は execution-skip 候補にすぎない。integration suite は cache
  state を semantic acceptance、proof authority、trusted status upgrade と
  して扱ってはならない。
- suite は `mizar-cache` の `CacheKey`、dependency fingerprint、cache-store
  compatibility、proof-reuse validation logic を再実装してはならない。
- Artifact manifest records は publication records であり、proof authority
  ではない。scheduler outputs、backend results、externally attested proofs、
  cached data を accepted proof status へ昇格してはならない。
- Manifest commit order は worker completion order から独立して決定的で
  あり続ける。
- 利用できない driver、IR、producer-token integration は placeholder
  implementation ではなく `external_dependency_gap` として記録しなければ
  ならない。

## Test Shape

task-19 integration fixture は次を行う。

- 少なくとも 2 つの source modules を持つ小さな workspace plan を作る。
- static source layout から module index を構築する。
- 既知の source modules に complete module-dependency overlay を与える。
- driver dependency なしで synthetic phase outputs により graph を schedule
  する。これは real frontend phase-service execution なしに frontend-shaped task
  scheduling を覆う。driver-owned phase-service boundary は `mizar-build` に
  存在しないためである。
- completed `ArtifactCommit` tasks だけを manifest updates に集める。
- `mizar-artifact` test data によって minimal verified artifacts を publish
  し、その entries を `commit_manifest_updates` で commit する。
- 意図的に shuffled update input を与えた後の publication order を含め、
  deterministic task/result/module ordering を検証する。

Artifact schema validation は `mizar-artifact` が所有するため、fixture は
valid `mizar-artifact` records を作る helper data を使ってよい。ただしそれらの
helpers は `mizar-build` 内に新しい build authority、producer token、proof
acceptance path を作ってはならない。
