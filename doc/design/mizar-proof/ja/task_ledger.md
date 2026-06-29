# mizar-proof Task Ledger

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は autonomous crate-development task ごとに 1 commit を記録する。
task commit は自分自身の hash を含められないため、self-hash は ledger を編集する
次の番号付き task、または final closeout task でだけ backfill する。番号のない
追加 bookkeeping commit は許可しない。

| Task | Status | Commit | Review result | Verification | Summary |
|---|---|---|---|---|---|
| 0. Crate plan | complete | `a1f6d1dba4ecc46aa2d434c8da8ae0279a2c23ec` | 初回 review は ledger/TODO status drift、広すぎる ATP gap 分類、`discharged_builtin` artifact-witness gap の欠落を指摘し、docs を同期した。focused re-review は no findings。Test sufficiency review も docs-only verification で no findings。 | `git diff --check` は passed。task-0 path の明示 staging 後に `git diff --cached --check` は passed。 | paired crate plan と task ledger を追加し、kickoff gap を分類する。source は scaffold しない。 |
| 1. Crate scaffold and lint-policy guard | complete | `bc7d33efd8b7427fa026807e729642219f62f809` | 初回 review は stale ledger/handoff finalization wording のみを指摘した。scaffold、dependency boundary、lockfile、crate root、lint guard には finding なし。更新後の focused re-review は scaffold / handoff finding なし。Test sufficiency review は no findings。 | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; task-1 path の明示 staging 後に `git diff --cached --check` passed。 | workspace crate、scaffold-only crate root、dependency manifest、lint-policy guard を追加する。 |
| 2. Spec: `policy.md` | complete | `193598cd5db448b17c5cddb721a5e22010b241b6` | 初回 review は built-in discharge scheduling wording、`AssumedByPolicy`、artifact witness gap、fingerprint TODO、ledger finalization の問題を指摘した。修正後、focused spec、test-sufficiency、full、source-doc review は no findings。 | `git diff --check` passed。task-2 path の明示 staging 後に `git diff --cached --check` passed。 | verifier policy setting、candidate class、external evidence、open obligation、policy assumption、fingerprint surface、scheduling、early stop、diagnostics、trust boundary を覆う paired policy spec を追加する。 |
| 3. Policy evaluator | complete | `f3ac274d9cf7266f5ebdc4de94fe963f72ba67d5` | 初回 review は synthesized accepted-kernel input risk、evidence-kind test coverage 不足、negative scheduling test gap、kernel/policy rejection layering drift、task-4 external-admission overreach を指摘した。修正後、focused spec、test-sufficiency、full、source-doc review は no findings。 | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; task-3 path の明示 staging 後に `git diff --cached --check` passed。 | policy evaluator、normalized kernel-origin wrapper、candidate class、schedulability check、kernel rejection separation、deterministic policy fingerprint を実装し、policy module 用に lint guard を更新する。 |
| 4. Externally attested evidence handling | complete | `986f62a184fa8d0a7fa0c98ef8f6c9669d85d844` | 初回 review は具体的な publication-status mapping 欠落、`PolicyDecision.class` と diagnostic ownership の曖昧さ、public `trusted_used_axioms_allowed` invariant risk、module-status drift、matrix coverage gap、policy-tainted origin coverage gap を指摘した。修正後、focused spec、test-sufficiency、full、source-doc review は no findings。 | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; task-4 path の明示 staging 後に `git diff --cached --check` passed。 | external evidence admission label、安定した policy diagnostic、non-trusted admission matrix、policy-tainted kernel-result routing を trusted `used_axioms` なしで実装する。 |
| 5. Spec: `selection.md` | complete | `6802bbec9f75d33f6c28b06391d60101726b2d51` | 初回 review は rejected/all-diagnostic outcome の仕様不足、non-total な tie-break input、`DischargedBuiltin` trust wording、policy-assumption handling 欠落、TODO ordering drift、external recordable/selectable wording drift を指摘した。修正後、focused spec、test-sufficiency、full、source-doc review は no findings。 | `git diff --check` passed; task-5 path の明示 staging 後に `git diff --cached --check` passed。 | 決定的な winner class、total な tie-break identity、rejected/no-selectable diagnostic、reuse metadata、trusted/non-trusted 境界、completion-time 禁止、deferred downstream integration を覆う paired winner-selection spec を追加し、policy wording を明確化する。 |
| 6. Winner selection | complete | `9230c36464a58a4f35a43ae1f7dc9fcde6e5e94d` | 初回 review は trusted-marker spoofing、pending kernel-checkable input がある場合の non-trusted winner、duplicate-id diagnostic が surfaced されない問題、rejected-category ordering gap、witness publication gap 欠落、test 不足、accepted-evidence hash identity の弱さを指摘した。修正後、focused spec、test-sufficiency、full、source-doc review は no findings。 | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; task-6 path の明示 staging 後に `git diff --cached --check` passed。 | deterministic winner selection、kernel-derived evidence hash に bind された trusted-kernel evidence marker、total tie-break/rejection ordering、pending-kernel gating、duplicate-id diagnostic、no-selectable diagnostic、reuse metadata、`discharged_builtin` witness-publication gap marking、selection module lint guard update を実装する。 |
| 7. Artifact proof selection merge | complete | `6e9a5a0400aae2c4c8b5b8098594ecc0bd3d2949` | 初回 review は source/class compatibility validation の欠落、same-class merge tie-break、policy-assumed/rejected preservation、duplicate built-in discharge input の test gap を指摘した。修正後の focused spec、test-sufficiency、full、source-doc re-review は no findings。 | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; task-7 path の明示 staging 後に `git diff --cached --check` passed。 | portfolio selection と built-in discharge selection を `VcId` ごとに artifact-facing merge し、invalid source/class pair と duplicate same-source input を reject し、status projection や artifact publication なしで trusted / non-trusted class を保つ。 |
| 8. Spec: `status.md` | complete | `76f112237c55dab875cd3e26cbfa11d45439fe5e` | 初回 review は `DischargedBuiltin` artifact field overreach、architecture 22 proof-reuse metadata 不足、`KernelCheckResult::Accepted` wording drift、`ProofWitnessRef` schema-version wording の曖昧さ、task-17 TODO reuse-metadata drift を指摘した。修正後の focused spec、test-sufficiency、full、source-doc re-review は no findings。 | `git diff --check` passed; task-8 path の明示 staging 後に `git diff --cached --check` passed。 | artifact/diagnostic status、trusted `used_axioms` propagation、explanation reference、proof-reuse validation metadata、downstream gap を定義する paired status-projection spec を追加する。 |
| 9. Proof status projection | complete | `ccd0e05820f61c02614ac523bf444556c5b29fa5` | 初回 review は public trusted-used-axiom constructor path、non-trusted trusted-axiom rejection coverage、reuse metadata coverage、`NoSelectableEvidence` の `RecordDiagnostic` publication semantics の test/design gap を指摘した。修正後の focused spec、test-sufficiency、full、source-doc re-review は no findings。 | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; task-9 path の明示 staging 後に `git diff --cached --check` passed。 | `src/status.rs`、status module exposure、lint guard update を実装し、projected proof status、現在の artifact publication availability、trusted used-axiom boundary check、explanation reference、architecture-22 reuse metadata を提供する。 |
| 10. Spec: `witness_store.md` | complete | pending self-hash | 初回 review は曖昧な `DischargedBuiltin` reuse hash semantics、artifact reachability proof の仕様不足、task-11 trust-boundary test obligation の欠落、`selected_proof_witness_hash` domain drift、循環した `publish_ref` lifecycle wording を指摘した。修正後、focused spec、test-sufficiency、full、source-doc re-review は no findings。 | `git diff --check` passed; task-10 path の明示 staging 後に `git diff --cached --check` passed。 | draft staging、unpublished `ProofWitnessRef` candidate、stable witness payload artifact hashing、committed artifact-manifest reachability と exact `VerifiedArtifact.proof_witnesses` coverage 後のみの publication、provenance metadata、cache/reuse boundary、deferred な `DischargedBuiltin` witness publication support を覆う paired witness-store spec と selection/status wording 同期を追加する。 |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 11 after the
task-10 commit exists. First verify a clean worktree and confirm the task-10
commit in HEAD history. Then implement task 11: `ProofWitnessDraft` staging and
`publish_ref` against the `mizar-artifact` witness-reference schema. Keep
publication impossible before committed witness publication proof with exact
`VerifiedArtifact.proof_witnesses` coverage, preserve the current
`DischargedBuiltin` witness `external_dependency_gap`, and do not add artifact
manifest writes, cache lookup, ATP execution, kernel checking, SAT solving,
proof search, premise selection, substitution invention, or placeholder
downstream integration.
```

Rationale: task 11 は witness publication boundary を source API と test に移すが、
proof authority は広げない。path/hash canonicalization、manifest reachability、
trusted/non-trusted witness class は staged または unsupported material を誤って
promote しうるため `xhigh` を維持する。test が存在した後の mechanical refactor だけなら
lower reasoning でよい。artifact schema が新しい blocking dependency gap を露出する場合だけ
上げる。
