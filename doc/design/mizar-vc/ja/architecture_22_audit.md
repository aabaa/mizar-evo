# Architecture-22 Follow-Up Audit: mizar-vc

> 正本言語: 英語。英語正本:
> [../en/architecture_22_audit.md](../en/architecture_22_audit.md)。

Task 21 は、cross-edit reuse identity wiring が commit
`2f3eb323be8080bf231e1b69dfc9e9e729bb45f9` で入った後に、
[architecture 22](../../architecture/ja/22.incremental_verification_contract.md)
に対して Task 20 implementation を監査する。これは docs-only audit であり、Rust source、
`.miz` fixture、expectation、`doc/spec`、traceability metadata、runner support、
downstream ATP/kernel/proof/cache integration は変更しない。

## 範囲と方法

この audit は architecture-22 の次の要求を確認した:

- clean-build equivalence と cache-miss fallback;
- `VcId` と `ObligationAnchor` の identity boundary;
- canonical VC / local-context fingerprint;
- dependency-slice completeness と reuse 可能な fingerprint payload;
- verifier-policy と deterministic-discharge evidence gate;
- Task 20 後の二言語ドキュメント同期。

確認した mizar-vc document:

- [vc_ir.md](./vc_ir.md)
- [generator.md](./generator.md)
- [discharge.md](./discharge.md)
- [dependency_slice.md](./dependency_slice.md)
- [source_spec_audit.md](./source_spec_audit.md)
- [bilingual_sync_audit.md](./bilingual_sync_audit.md)
- [todo.md](./todo.md)
- [task_ledger.md](./task_ledger.md)

## Architecture-22 対応

| architecture-22 requirement | Task 20 後の mizar-vc status | 分類 |
|---|---|---|
| `VcId`、source range、parser/node/arena id、task-local row id だけでは cross-edit proof reuse に十分ではない。 | `VcId` は snapshot-local のままである。`ObligationAnchor` の source-shape payload は `VcId`、source range、`SourceId`、handoff id、candidate sort key、dense owner row id を除外する。Dependency-slice の reuse 可能 fingerprint は diagnostic local key ではなく stable payload を hash する。 | 現在の deterministic-discharge reuse candidate 向けに実装済み。 |
| `ObligationAnchor` の一致だけでは不十分である。 | `ProofReuseCandidateKey` は complete anchor、current matching dependency slice、canonical VC fingerprint、local-context fingerprint、compatible policy fingerprint、matching newly produced deterministic discharge evidence も要求する。 | deterministic discharge branch 向けに実装済み。 |
| Canonical VC fingerprint は goal、premise、proof hint、generated formula payload を対象にし、できない場合は fail closed する。 | `CanonicalVcFingerprint` は owning `VcSet` 経由で generated formula を解決する。raw core formula id、hint/premise 経由の definition id、diagnostic、cycle、stable binder payload を持たない quantified formula は fail closed する。 | stable generated payload 向けに実装済み。不完全な upstream payload は fail closed のまま。 |
| Canonical local-context fingerprint は stable context payload と policy input を対象にし、できない場合は fail closed する。 | `LocalContextFingerprint` は sort key、binder 以外の context kind、resolved formula payload、provenance、explicit verifier-policy input を対象にする。Binder declaration と未解決 core/generated formula payload は fail closed する。 | stable local context 向けに実装済み。binder/core payload は fail closed のまま。 |
| reuse に使う dependency slice は complete でなければならず、missing dependency data を no dependency と解釈してはならない。 | `DependencySlice` は unknown coverage を `IncompleteUncacheable` にする。raw `CoreFormulaId`、`CoreDefinitionId`、unresolved generated formula、quantified formula、binder declaration、opaque trace/import/computation marker、missing replay data、incomplete anchor は conservative unknown を生む。`ProofReuseCandidateKey` は incomplete slice を拒否する。 | 現在の slice family 向けに実装済み。missing upstream payload は external gap のまま。 |
| verifier policy と proof witness または deterministic discharge hash が一致しなければならない。 | Task 20 は deterministic-discharge branch を実装する。key は policy input / status policy を含み、current VC の status evidence と一致する newly produced replayable deterministic evidence record を要求する。Proof-witness hash と consumer validation は未実装である。 | deterministic-discharge branch は実装済み。proof-witness branch は deferred/external。 |
| cache lookup、kernel acceptance、proof policy、ATP certificate、artifact consumer は reuse を受理する前に validation しなければならない。 | `mizar-vc` は untrusted reusable input だけを生成する。この crate では downstream cache、ATP、kernel、proof、artifact consumer は key を受理しない。 | `external_dependency_gap` / `deferred`。 |

## Regression Evidence

Task 20 は Rust coverage を追加または更新した:

- cross-edit `VcId` shift で同じ proof-reuse key になること;
- generated-formula id shift で、kernel evidence handoff identity が供給される前の
  deterministic-discharge reuse fingerprint は同じになること; task 28 は canonical
  handoff または context-identity hash が shift する場合、current kernel-handoff
  proof-reuse key を意図的に conservative に invalidation する;
- policy と local-context の変更が reuse identity を変えること;
- stale slice set、pre-existing evidence、incomplete anchor、generated-goal
  change、stable evidence 欠落、unresolved payload が fail closed すること;
- generated seed family と algorithm candidate は source-shape hash を available
  に保ちつつ、raw core goal は canonical-goal incomplete のままであること;
- unresolved core formula、definition、generated diagnostic、quantified、binder payload
  が独立した unknown coverage を生むこと。

Task 21 は audit-only task なので新しい Rust test は追加しない。ledger に記録された Task 20
verification が source behavior の evidence である。

## 残る分類済み gap

- `external_dependency_gap`: active source-derived `proof_verification` support は
  `mizar-test` にまだ存在しない。VC Task 31 / `MT10-VC-T180` が最初の exact route、
  VC 32-55 が後続 `MT10-VC-PV/VC<n>` slice を所有する。
- `external_dependency_gap` / `deferred`: `mizar-kernel` は corrected
  formula/substitution evidence checking、completed VC Tasks 25-29 は producer handoff /
  identity payload を所有する。`mizar-atp`、`mizar-proof`、`mizar-cache` は存在するが、
  この VC milestone の active consumer
  としては未配線であるため、ATP translation、proof policy、cache lookup/reuse、
  artifact persistence、proof-witness validation はこの crate の外に残る。
- `external_dependency_gap`: registration、redefinition、reduction、call-precondition、
  branch、match、range-loop、collection-loop、term-derived/recursive termination、
  Pick non-emptiness、ghost-isolation
  zero-VC integration、authenticated trace context、source-derived core formula payload、
  definition payload、quantified binder payload、source-derived obligation payload family
  について、upstream explicit/stable payload はまだ不完全である。
- `spec_gap`: VC 53 には canonical partial-call admission semantics があるが、canonical
  termination-evidence producer、reference identity/schema、authentication contract/rule、
  owning test はない。blocked のままとし、transport/authentication mechanism を捏造しない。
- `deferred`: proof-witness hash、ATP/kernel/proof/cache validation、artifact consumer、
  source-derived runner integration は、architecture-22 reuse を deterministic-discharge
  candidate key の外で受理する前に、downstream または後続 task で実装しなければならない。

この audit 後の Task 20 identity contract には、`repo_metadata_conflict`、未分類の
`source_drift`、`design_drift`、`source_undocumented_behavior`、
`test_expectation_drift`、`boundary_violation` は観測されなかった。

## VC Task 30 source-derived identity addendum

Task 30 は follow-up ownership を変更するだけで architecture-22 reuse eligibility を
変更しない。exact Task-31 anchor は source-shape/empty-context identity を available に
保つが `CanonicalGoalHash` について incomplete のままで、reuse 不可である。Tasks 32-55
は real source-derived payload が実装された時だけ stable formula/context/substitution/
trace/dependency ingredient を所有する。missing payload は canonical hash を装った id、
label、range、marker ではなく conservative unknown のままにする。この更新は cache lookup、
proof reuse、discharge、verification、acceptance credit を与えない。
