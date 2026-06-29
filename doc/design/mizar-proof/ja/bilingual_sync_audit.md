# mizar-proof Bilingual Documentation Sync Audit

> 正本は英語です。英語版:
> [../en/bilingual_sync_audit.md](../en/bilingual_sync_audit.md)。

## Scope

task 16 は `doc/design/mizar-proof/en/` 配下の English canonical document と
`doc/design/mizar-proof/ja/` 配下の Japanese companion をすべて比較する。

監査対象:

| English canonical | Japanese companion | Result |
|---|---|---|
| `00.crate_plan.md` | `00.crate_plan.md` | synchronized。task-16 metadata update が必要 |
| `policy.md` | `policy.md` | synchronized |
| `selection.md` | `selection.md` | synchronized |
| `status.md` | `status.md` | synchronized |
| `witness_store.md` | `witness_store.md` | synchronized |
| `source_spec_audit.md` | `source_spec_audit.md` | synchronized |
| `todo.md` | `todo.md` | synchronized。task-16 status update が必要 |
| `task_ledger.md` | `task_ledger.md` | synchronized。task-15 hash backfill と task-17 handoff が必要 |
| `bilingual_sync_audit.md` | `bilingual_sync_audit.md` | この task で作成 |

companion を欠く file はない。Japanese placeholder も残っていない。

## Method

この audit は以下を確認した:

- paired file name と canonical-language link;
- heading structure と section order;
- task status、task ledger、current handoff、recommended reasoning text;
- trust-boundary wording: `mizar-proof` は proof acceptor ではなく、trusted
  acceptance は `mizar-kernel` だけに由来し、external/backend/cache material は
  trusted status や trusted `used_axioms` に昇格しない;
- cache reuse、`DischargedBuiltin` artifact witness、artifact publication token、
  copied kernel metadata、payload canonicality validator、ATP early-stop
  integration に関する deferred / `external_dependency_gap` wording;
- public enum policy statement と no-exhaustive-exception statement。

Japanese document は stable English identifier、enum name、status string、gap id、
command name を意図的に保持する。その terminology は drift ではない。

## Findings

blocking bilingual drift は見つからなかった。必要な edit は task-local metadata
update だけである:

| ID | Classification | Finding | Resolution |
|---|---|---|---|
| `PROOF16-SYNC-001` | `design_drift` | task 16 前の `00.crate_plan.md` は `bilingual_sync_audit.md` を planned としていた。 | paired crate plan を更新し、`source_spec_audit.md` と `bilingual_sync_audit.md` を completed、後続 audit を planned のままにする。 |
| `PROOF16-SYNC-002` | `design_drift` | task 16 前の `todo.md` は task 16 を incomplete としていた。 | paired TODO に task-16 completion と audit summary を記録する。 |
| `PROOF16-SYNC-003` | `design_drift` | `task_ledger.md` は task 16 が ledger を編集するまで task 15 の self-hash を含められない。 | task 15 を backfill し、task 16 を追加し、handoff を task 17 に進める。 |

これらは expected task-finalization update であり、source/spec behavior drift ではない。

## Trust Boundary Check

paired document は次の canonical constraint に合意している:

- `mizar-proof` は proof policy、deterministic winner selection、status
  projection、witness staging/publication reference、early-stop policy hook、
  proof-reuse validation metadata を所有する。
- `mizar-proof` は ATP backend、SAT solving、kernel acceptance、proof search、
  premise selection、substitution invention、cache lookup、artifact manifest
  commit を実行しない。
- trusted acceptance と trusted `used_axioms` は accepted
  `mizar-kernel::checker::KernelCheckResult` value だけに由来する。
- externally attested evidence、backend diagnostics、backend proof payload、
  backend-reported axiom list、cache record、policy assumption、open obligation、
  witness metadata は、accepted kernel result が独立に trusted class を支えない限り
  non-trusted のままである。
- `require_kernel_certificates` は externally attested evidence と policy
  assumption が winner になることを防ぐ。
- arrival order、completion time、runtime duration、worker/process id、
  temporary path、cache timing は proof identity ではない。

## Remaining Gaps

task 16 は新しい implementation gap を導入しない。module spec と source/spec audit に
記録済みの `deferred` / `external_dependency_gap` を保持する:

| Gap area | Classification | Current owner / future task |
|---|---|---|
| cache-facing proof-reuse export contract | `deferred` | task 17 / future cache consumer |
| `DischargedBuiltin` artifact witness schema support | `external_dependency_gap` | `mizar-artifact` |
| committed witness publication proof token | `external_dependency_gap` | artifact publication boundary |
| copied kernel acceptance metadata for witness drafts | `external_dependency_gap` | kernel/artifact boundary |
| byte-level witness payload canonicality validators | `deferred` | concrete payload producers |
| live ATP early-stop adoption/cancellation wiring | `external_dependency_gap` | `mizar-atp` |

task-16 bilingual sync audit 中に `repo_metadata_conflict` は観測されなかった。
後続の ATP closeout metadata conflict は tasks 18-20 で記録され、focused correction
commit `36d1a9c` で解消済みである。

## Conclusion

English canonical documentation と Japanese companion は現在の `mizar-proof` state で
synchronized している。task 16 は documentation-only であり、source behavior は変更しない。
