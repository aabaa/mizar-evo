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
| 9. Implement resolution trace checker | complete | `28b7e7122c8cad04a6526d8de8cdfd0394d8bb3c` | Spec/doc review: medium clause-helper documentation と low imported-context/report-shape findings を修正し、final re-review は finding なし。Test sufficiency review: generated/imported/previous-step parent resource ordering、report binding、generated/resolution-step final goal、nested canonical-length helper coverage、stable rejection coverage を final re-review まで確認し finding なし。Full implementation review: high pre-clone resource-ordering findings と medium report binding、previous-step parent clone-order、stale docs findings を修正し、final re-review は finding なし。Source/doc consistency review: low `ResolutionTraceInput` target-fingerprint sketch finding を修正し、final re-review は finding なし。 | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task。deterministic resolution trace replay、explicit imported clause context、checked-step report、private replay binding 付き final-goal helper、stable kernel rejection mapping、clone 前の bounded borrowed parent validation、bounded resolvent accumulation、`clause` の term-depth / canonical-length helpers、cfg(test) certificate fixture construction、module exposure、lint guard update を実装する。SAT solver、ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state read、downstream ATP/proof/cache/artifact coupling、`.miz` fixture、expectation、`doc/spec` edit はない。Imported-fact availability/fingerprint/proof-status auditing、backend proof translation、checker orchestration、proof/cache/artifact consumers は `external_dependency_gap` / `deferred` のまま。 |
| 10. Spec: `substitution_checker.md` | complete | `d79506c6e0b7029fb1512454b0eff72579362df7` | Spec/doc review: binder-context authority、duplicate context classification、byte grammar、side-condition schema、duplicate witness classification、local-abbreviation guard evidence、captured-free-variable boundary findings を修正し、final re-review は finding なし。Test sufficiency review: resource、binder-decode、side-condition、rejection-location、report-binding、prohibition、manifest、first-use、trailing-byte、unused witness、payload-positive、rewrite-path、payload actual-term coverage findings を修正し、final re-review は finding なし。Full implementation-boundary review: high missing explicit payload finding と nested-resource、task-split、report-binding、captured-free-variable findings を修正し、final re-review は finding なし。Source/doc consistency review: final re-review は finding なし。 | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only。explicit immutable substitution payload context、binder-context grammar、payload/replacement rules、side-condition evidence schema、replay limits、direct-substitution と alpha/FV の task split、stable rejection mapping、planned task-11/task-12 tests、kernel prohibitions を含む paired substitution checker specs を追加する。Rust source、`.miz` fixture、expectation、`doc/spec` edit はない。Substitution payload の inline certificate encoding、local-abbreviation closure/type-guard evidence、captured-free-variable closure evidence、source-derived substitution certificate、downstream proof/cache/artifact consumer は `external_dependency_gap` / `deferred` のまま。missing / deferred payload evidence は推論せず拒否する。 |
| 11. Implement substitution checking | complete | `b97c4a3a700fec986d3e203b1a88d23edcfba7f3` | Spec/doc review: deterministic-counter ownership、under-binder task split、planned-test sync、replayed-target resource-test findings を修正し、final re-review は finding なし。Test sufficiency review: binder decode matrix、side-condition shape、resource limits、malformed/deferred payload、precise location、duplicate context id、lint token、actual-term byte limit、ambiguous context fallback、side-context canonicalization findings を修正し、final re-review は finding なし。Full implementation review: high binder-frame compatibility、pre-clone replay budget、binder-aware capture、binder-context count preallocation、lookup complexity findings を修正し、final re-review は finding なし。Source/doc consistency review: replayed target depth coverage と context fallback consistency findings を修正し、final re-review は finding なし。 | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-core` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-checker` は、この task が checker/trace boundary orchestration を変更しないため実行しない。 | Rust source task。explicit immutable payload evidence 上の direct substitution replay、deterministic byte/count/frame checks 付き binder-context decoding、manifest と frame/term compatibility checks、first-use context lookup/fallback、payload validation、direct capture rejection、pre-clone replay budget measurement、side-condition shape/owner/path/resource validation、deterministic checked-substitution report、module exposure、lint coverage を実装する。Alpha-conversion semantics、semantic freshness/free-variable replay、local-abbreviation closure/type-guard evidence、captured-free-variable closure evidence、inline payload certificate encoding、source-derived substitution certificate production、checker orchestration、downstream proof/cache/artifact consumer は `external_dependency_gap` / `deferred` のまま。missing/deferred evidence は推論せず拒否する。 |
| 12. Implement alpha-conversion, freshness, and free-variable checks | complete | `577f6f220b93d94c9796208829216f43a8e2e3d4` | Spec/doc review: 初回 blocking deterministic-freshness、alpha-correspondence、capture-set derivation gaps と medium TODO/ledger drift findings を paired documentation updates で修正し、final re-review は blocking/high finding なし。Test sufficiency review: high unreferenced-witness、bound-renaming、stale-avoided-set、captured-free-variable、shuffled-rejection coverage findings を修正し、final re-review は finding なし。Full implementation review: medium recomputed avoided/capture-set resource findings と avoided-limit exclusion edge を修正し、final re-review は finding なし。Source/doc consistency review: medium Task 12 status drift finding を修正し、final re-review は finding なし。 | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-core` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-checker` は、この task が checker/trace boundary orchestration を変更しないため実行しない。 | Rust source task。referenced freshness witness 上の semantic alpha-conversion、manifest-order candidate stream からの deterministic freshness counter replay、resource bounds 付き avoided-variable recomputation、resource bounds 付き target capture-set recomputation、free-variable side-condition replay、non-capturing under-binder substitution acceptance、shuffled context construction 下の deterministic checked report / rejection、paired architecture/substitution docs、Task 11 hash backfill を実装する。Local-abbreviation closure/type-guard evidence、captured-free-variable closure evidence、inline payload certificate encoding、source-derived substitution certificate production、checker orchestration、downstream proof/cache/artifact consumer は `external_dependency_gap` / `deferred` のまま。missing/deferred evidence は推論せず拒否する。 |
| 13. Spec: `checker.md` | complete | `865231081df7538faea132c499d9c57d5ecfa9cb` | Spec/doc review: medium reduction strategy-audit field finding を修正し、その後 imported-clause fingerprint binding、derived-fact payload authority、budget split、planned-test、status fixes を review して blocking/high/medium finding なし。Test sufficiency review: medium full-prohibition coverage、deterministic rejection coverage、imported-clause fingerprint planned tests、resource/timeout split planned tests、derived-payload authority planned tests を修正し、final re-review は finding なし。Full implementation-boundary review: high imported-clause content binding と medium derived-payload authority / budget mapping findings を修正し、final re-review は finding なし。Source/doc consistency review: medium budget mapping と TODO/ledger status findings を修正し、final re-review は finding なし。 | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only。kernel check-service pipeline、normalized clause fingerprint binding 付き imported fact context、caller-supplied imported-clause context の禁止、cluster/reduction trace boundary、certificate-owned derived-fact payload authority、derived facts、final-goal acceptance、deterministic rejection/resource mapping、planned task-14/15/16 tests、deferred external integrations の paired checker module specs を追加する。Rust source、`.miz` fixture、expectation、`doc/spec` edit はない。Source-derived certificate、ATP proof translation、`mizar-checker` による cluster trace payload production、proof/cache/artifact consumers は `external_dependency_gap` / `deferred` のまま。Missing producer / consumer integration は mock しない。 |
| 14. Implement imported-fact checking | complete | `874881b42d5c008336a34cb4cfaf24f7b403a1fb` | Spec/doc review: high missing fingerprint algorithm semantics と medium external-attestation profile gate findings を task-14 algorithm id 1 と explicit imported-fact policy で修正し、low input-shape note も修正した。final re-review は finding なし。Test sufficiency review: medium evidence statement-fingerprint、proof-status matrix、variable-manifest、unsupported evidence algorithm、context/report canonicalization findings を修正し、final re-review は blocking/high/medium finding なし。Full implementation review: medium canonical-byte allocation-before-budget と unbounded context sorting findings を non-allocating length precheck と bounded context constructor で修正し、final re-review は finding なし。Source/doc consistency review: medium proof-status ordering と resource-ordering drift を修正し、final re-review は finding なし。 | `cargo fmt --check` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-kernel` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task。imported-fact evidence context、bounded context canonicalization、proof-status/profile-policy checking、task-14 normalized clause fingerprint algorithm id 1、pre-allocation resource checks 付き normalized clause fingerprint binding、resolution replay 用 imported clause context construction、task-13 hash backfill、spec-backed checker module の lint exposure を実装する。Source-derived imported contexts、task-14 algorithm id 1 を超える digest registry、full check-service orchestration、cluster replay、proof-policy projection、downstream proof/cache/artifact consumers は `external_dependency_gap` / `deferred` のまま。SAT/ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state、`.miz` fixture、expectation、`doc/spec` edit はない。 |
| 15. Implement cluster trace replay | complete | `77262c0d890026dd614225f2d762861fe192169b` | Spec/doc review: medium requested-id pipeline wording と、先行する reduction authority/order/context wording findings を修正し、final re-review は blocking/high/medium finding なし。Test sufficiency review: high transitive requested-only/global-order coverage と medium base-fact、guard exact-match、context canonicalization、resource coverage findings を修正し、final re-review は blocking/high/medium finding なし。Full implementation review: high reduction binding/commitment resource-bound finding と medium unused-context runtime-limit、missing dependency location、stale docs、required-guard commitment wording findings を修正し、final re-review は blocking/high/medium finding なし。Source/doc consistency review: medium reduction-authority docs、unused-context runtime-limit wording、stale deferred wording、required-guard commitment sync findings を修正し、final re-review は blocking/high/medium finding なし。 | `cargo fmt --check` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-kernel` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task。bounded requested cluster/reduction trace replay、global cluster/reduction id namespace、requested-id transitive dependency closure in global order、explicit checked imported/generated/earlier-trace dependencies、bounded context canonicalization、construction 後の unused-context ignore semantics、required guards を含む normalized cluster fact / reduction audit / reduction result commitments、stable rejection mapping and locations、task-14 hash backfill、focused fail-heavy tests を実装する。Richer producer-side active-rule payload validation、source-derived cluster trace production、normalized commitments を超える semantic redex/LHS-to-RHS validation、full check-service orchestration、proof-policy projection、downstream proof/cache/artifact consumers は `external_dependency_gap` / `deferred` のまま。SAT/ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state、`.miz` fixture、expectation、`doc/spec` edit はない。 |
| 16. Kernel check service and deterministic batch ordering | complete | `c0b8e6104f38d02e7bf8f6c1cda5900fb50bdfc1` | Spec/doc review: medium batch ordering、external rejection scope、task-15 ledger、TODO status、fail-fast wording、derived-fact table findings を修正し、final re-review は blocking/high/medium finding なし。Test sufficiency review: high nonempty substitution service coverage と service-level final-goal helper coverage を修正し、medium later timeout、derived-fact resource limit、fail-fast wording coverage も修正した。final re-review は blocking/high/medium finding なし。Full implementation review: high imported axiom/theorem namespace collapse を namespaced `CheckedFactRef` / `CheckedFactContext` と namespaced `used_axioms` で修正し、medium substitution report materialization ordering と generated-clause base-set duplicate findings を修正した。final re-review は blocking/high/medium finding なし。Source/doc consistency review: medium planned-test wording、TODO/ledger drift、TODO batch wording findings を修正し、final re-review は blocking/high/medium finding なし。 | `cargo fmt --check` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-kernel` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task。deterministic `KernelCheckInput` / `KernelCheckResult`、imported facts、substitutions、resolution replay、requested cluster/reduction replay、fail-closed derived facts、resolution helper 経由の final-goal binding、namespaced imported axiom/theorem fact references、cluster replay 用 generated-clause base-set deduplication、policy taint propagation、checker step timeout、report-record resource limits、target then caller order による deterministic batch ordering、paired docs/TODO updates、focused service tests を実装する。Derived-fact payload schemas、source-derived certificate / service envelopes、cancellation token plumbing、external worker scheduling、proof-policy projection、downstream proof/cache/artifact consumers は `external_dependency_gap` / `deferred` のまま。SAT/ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state、`.miz` fixture、expectation、`doc/spec` edit はない。 |
| 17. Soundness fail-test corpus | complete | `b7e1493050ed49110e4ddf7a7a75d971bdf72c59` | Spec/doc review: medium assertion identity と service-path scope findings を implementation spec で修正し、final re-review は blocking/high/medium finding なし。Test sufficiency review: medium service-path reduction mutation と strict timeout/resource identity findings を修正し、final re-review は blocking/high/medium finding なし。Full implementation review: blocking/high/medium finding なし。Source/doc consistency review: blocking/high/medium finding なし。 | `cargo fmt --check` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-kernel` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging。 | Test/audit task。service-path single-mutation soundness fail corpus を追加し、imported fact identity、substitution replay、resolution replay、cluster trace replay、reduction trace replay、final-goal binding、derived-fact fail-closed behavior、timeout、resource exhaustion を cover する。各 corpus case は rejected status、category、detail、stable detail key、exact location、exactly one rejection、partial accepted outputs なしを assert する。Source-derived corpus runner と external producer fixtures は `external_dependency_gap` / `deferred` のまま。`.miz` fixture、expectation、`doc/spec`、SAT/ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state、external dependency mock は追加しない。 |
| 18. Determinism and replay-cost suite | complete | `3d1942e97ea245d2fae09dac4e26cefd67c02bd1` | Spec/doc review: medium batch-determinism ambiguity を distinct-target equality と equal-target caller-order tie preservation に分けて修正し、low Task 17 backfill と stable-rendering wording findings に対応した。final spec re-review は blocking/high/medium finding なし。Test sufficiency review: finding なし。Full implementation review: blocking/high/medium finding なし。Source/doc consistency review: finding なし。 | Focused Task 18 tests passed; `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging。 | Test hardening task。repeated single checks、shuffled imported/cluster context construction、shuffled requested trace ids、distinct-target batch permutations、equal-target tie preservation、public rendering surrogate としての stable rejection keys/locations の決定的 equality coverage を追加する。cluster/reduction trace counts と checker pipeline/report budgets の exact replay-cost assertions を追加する。Source-derived runner、benchmark、randomness、property-test dependency、external producer fixture、`.miz` fixture、expectation、`doc/spec`、SAT/ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state、downstream ATP/proof/cache/artifact coupling は scope 外のまま。 |
| 19. Public-enum forward-compatibility policy | complete | `981fa7a05fe8de11168bd862d81cbd7d486347c0` | Spec/doc review: medium documentation/inventory guard finding を paired `public_enum_policy.md` を canonical exact enum inventory とし、lint-checked EN/JA source match を要求して修正した。low rejection compatibility wording と bookkeeping findings に対応した。final spec re-review は blocking/high/medium finding なし。Test sufficiency review: finding なし。Full implementation review: blocking/high/medium finding なし。Source/doc consistency review: blocking/high/medium finding なし。 | `cargo test -p mizar-kernel kernel_public_enums_are_forward_compatible_and_documented` passed; `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging。`cargo test -p mizar-core` と `cargo test -p mizar-checker` は、この task が public API compatibility annotations と lint/docs だけを追加し、binder contracts や checker/trace replay semantics を変更しないため不要。 | Docs/test/source-annotation task。すべての public enum を exhaustive exception なしの forward-compatible と分類し、不足していた `#[non_exhaustive]` marker を追加し、immediate marker と exact source-to-policy inventory synchronization の両方を要求するよう lint coverage を広げる。Rejection category/detail の stable-key spelling、meaning、phase ownership、ordering、removal、rename、remapping は compatibility review 対象のまま。variant 変更、runtime behavior 変更、`doc/spec`、`.miz`、expectation、SAT/ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state、downstream ATP/proof/cache/artifact coupling は追加しない。 |
| 20. Source/spec correspondence and prohibition audit | complete | `fb81213c33d5b2a31eb976a4fa6804bfc0ffe6c5` | Spec/doc review: medium test-traceability / stale-bookkeeping findings を修正し、final re-review は blocking/high/medium finding なし。Test sufficiency review: medium scanner blind spot、stale-doc allowance、weak Trust Statement wording、whole-doc gap check findings を修正し、final re-review は blocking/high/medium finding なし。Full implementation review: medium public-surface scanner finding を修正し、final re-review は finding なし。Source/doc consistency review: finding なし、final re-review は blocking/high/medium finding なし。 | `cargo test -p mizar-kernel source_spec` passed; `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging。Task 20 は audit docs と lint guard だけを変更し、binder contract や checker/trace runtime behavior を変更しないため、`cargo test -p mizar-core` と `cargo test -p mizar-checker` は不要。 | Audit/lint task。public API、spec、test、deferred traceability を含む paired source/spec audit docs を追加し、module Trust Statement を task-20 prohibition wording で強化し、exact public module/item inventory、Trust Statement prohibition wording、row-local gap classification、scanner regression cases の lint coverage を広げる。Source-derived certificate/service envelopes、ATP proof translation、`mizar-checker` cluster/reduction payload production、derived-fact payload schema、service-envelope normalization/cancellation/worker scheduling、downstream proof/cache/artifact consumers、downstream wildcard-arm checks は `external_dependency_gap` / `deferred` のまま。runtime behavior change、`doc/spec`、`.miz`、expectation、SAT/ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state、downstream dependency coupling は追加しない。 |
| 21. Bilingual documentation sync audit | ready to commit | pending self-hash | Spec/doc review: medium Task21 handoff、English-canonical conflict authority、stronger read-only validation findings を修正し、final re-review は blocking/high/medium finding なし。Test sufficiency review: blocking/high/medium finding なし。Full implementation review: medium self-inventory / crate-plan verification wording findings を修正し、final re-review は blocking/high/medium finding なし。Source/doc consistency review: blocking/high/medium finding なし。 | Deterministic file-pair and companion-link check passed; `cargo test -p mizar-kernel --test lint_policy` passed; `git diff --check` passed。この task は documentation だけを変更し Rust source や executable lint behavior を変更しないため、`cargo fmt`、full `cargo test -p mizar-kernel`、clippy は不要。`git diff --cached --check` は explicit staging 後に pending。 | Docs audit task。paired bilingual sync audit docs を追加し、Task20 hash/status を backfill し、English canonical authority を維持し、file/link/heading/table/inventory/status sync checks を記録し、Task21-to-Task22 handoff を更新し、external producer/consumer gaps は `external_dependency_gap` / `deferred` として残す。runtime behavior change、public API change、`doc/spec`、`.miz`、expectation、SAT/ATP/proof search、premise selection、overload resolution、cluster search、implicit coercion insertion、fallback inference、global mutable state、downstream dependency coupling は追加しない。 |
| 22. Module-boundary refactor gate | not started | pending | pending | pending | task 21 commit が必要。Audit または move-only task。 |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | task 22 commit、全 hard gate pass、read-only quality review score >= 90/100 が必要。 |

## Task 21 handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 21
bilingual documentation sync audit commit. Before starting task 22, verify a
clean worktree, confirm the task 21 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md,
doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/source_spec_audit.md,
doc/design/mizar-kernel/en/bilingual_sync_audit.md, and the paired module
specs. Implement task 22 only: perform the module-boundary refactor gate, add
paired `module_boundary_audit.md` files, and split private implementation
modules only if the audit finds an oversized or mixed-responsibility file that
can be moved without behavior or public API changes. Do not change public APIs,
diagnostics, deterministic renderings, artifact-facing schemas,
consumer-visible behavior, certificate semantics, rejection semantics,
`doc/spec`, `.miz` fixtures, expectations, SAT/ATP/proof search, premise
selection, overload resolution, cluster search, implicit coercion insertion,
fallback inference, global mutable state, or downstream ATP/proof/cache/artifact
integration. If no move-only split is required, record the audit outcome and do
not edit Rust source. Use review-only agents for the required AGENTS.md review
phases. Run `git diff --check` and `git diff --cached --check`; if Rust source
moves are made, additionally run `cargo fmt --check`,
`cargo test -p mizar-kernel`, and
`cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings`.
Commit task 22 by itself with explicit path staging.
```

Rationale: task 22 は closeout 前の最後の structural gate である。Trusted
kernel code の layout change は review boundary や behavior を誤って変える
可能性があるため `xhigh` を維持する。move が不要な docs-only audit なら下げて
よく、必要な split が hidden public API または behavioral coupling を露出する
場合だけ上げる。

## Task 11 handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 11
substitution-checker implementation commit. Before starting task 12, verify a
clean worktree, confirm the task 11 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md,
doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/substitution_checker.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/mizar-kernel/en/rejection.md,
doc/design/mizar-kernel/en/clause.md,
crates/mizar-kernel/src/substitution_checker.rs,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/16.substitution_and_binding.md, and
doc/design/architecture/en/19.failure_semantics.md. Implement task 12 only:
extend `substitution_checker` with alpha-conversion, semantic freshness, and
free-variable side-condition replay over the explicit context records already
introduced by task 11. Verify architecture-16 deterministic freshness counters,
bound-variable renaming consistency, free-variable preservation, capture-set
constraints, and coherent report binding without proof search or repair
heuristics. Do not implement proof search, ATP/SAT search, premise selection,
overload resolution, cluster search, implicit coercion insertion, fallback
inference, local-abbreviation closure replay beyond explicitly specified
evidence, captured-free-variable closure replay unless the spec is updated
first, global mutable state reads, checker orchestration, or downstream
proof/cache/artifact integration. Add focused tests for alpha equivalence,
freshness counters, FV constraints, malformed side-condition semantics,
resource limits, deterministic rejection locations, and regression cases
around task-11 under-binder deferral. Run cargo fmt --check,
cargo test -p mizar-kernel, cargo clippy -p mizar-kernel --all-targets
--all-features -- -D warnings, cargo test -p mizar-core because the task
continues binder-sensitive kernel rechecking, git diff --check, and
git diff --cached --check after explicit path staging. Run
cargo test -p mizar-checker only if checker/trace boundary semantics are
touched; otherwise record why it was not run. Use review-only agents for the
required AGENTS.md review phases and commit task 12 by itself.
```

Rationale: task 12 は soundness-critical な binder 側の substitution replay を
完了し、明示的に正当化された alpha/freshness/FV evidence だけを受理する。
`xhigh` を維持する。bookkeeping-only docs sync のみなら下げてよい。
architecture 16 と実装済み task-11 context model が矛盾する場合だけ上げる。

## Task 10 handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 10
substitution-checker spec commit. Before starting task 11, verify a clean
worktree, confirm the task 10 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md,
doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/substitution_checker.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/mizar-kernel/en/rejection.md,
doc/design/mizar-kernel/en/clause.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/16.substitution_and_binding.md, and
doc/design/architecture/en/19.failure_semantics.md. Implement task 11 only:
add `crates/mizar-kernel/src/substitution_checker.rs`, expose it from
`src/lib.rs`, and implement direct substitution replay over explicit immutable
`SubstitutionContext` payload evidence. Implement binder-context decoding,
payload/replacement validation, direct capture-avoiding replay, deterministic
report binding, stable rejection mapping, and shape/owner/path/resource
validation for referenced freshness/free-variable records. Do not implement
alpha-conversion, semantic freshness replay, semantic free-variable replay,
local-abbreviation closure replay, captured-free-variable closure replay,
proof search, ATP/SAT search, premise selection, overload resolution, cluster
search, implicit coercion insertion, fallback inference, global mutable state
reads, or downstream proof/cache/artifact integration. Reject missing payloads,
deferred local-abbreviation payloads, and deferred captured-free-variable roles
rather than inferring or accepting them. Add focused tests for all planned
task-11 cases in `substitution_checker.md`, including no-diff-inference,
rewrite-path specificity, manifest/resource limits for payload actual terms,
first-use context behavior, private report binding, and prohibition lint
coverage. Run cargo fmt --check, cargo test -p mizar-kernel,
cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings,
cargo test -p mizar-core because the task independently re-checks binder
contracts, git diff --check, and git diff --cached --check after explicit path
staging. Run cargo test -p mizar-checker only if the implementation touches
checker/trace boundary semantics; otherwise record why it was not run. Use
review-only agents for the required AGENTS.md review phases and commit task 11
by itself.
```

Rationale: task 11 は binder-sensitive な trusted kernel source を開始し、
explicit payload 上の deterministic evidence replay に留めなければならず、
substitution search を推論してはならないため `xhigh` を維持する。typo-only
docs sync だけなら下げてよい。既存 `mizar-core` binder API が explicit
kernel-owned recheck boundary と矛盾する場合だけ上げる。

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
