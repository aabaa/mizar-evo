# Task Ledger: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/task_ledger.md](../ja/task_ledger.md).

This ledger is the restart point for autonomous `mizar-kernel` crate work.
Before starting any task, check `git status`, `git log`, this table, and
[todo.md](./todo.md). A task is complete only when its commit exists in
history, final review outcomes are known, verification results are known, and
deferred reasons are recorded. A commit cannot contain its own final hash, so
self-hashes are verified from `git log` before the next task starts and
backfilled by a later committed bookkeeping point or the closeout task.

| Task | Status | Commit | Reviews | Verification | Deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | complete | `81ffb5561fc1b24ae355d216e1a455d2a487d923` | Spec/doc review: low pending-status finding fixed; final re-review no findings. Test sufficiency review: medium `--all-features` and conditional cross-crate verification findings fixed; final re-review no findings. Full implementation review: high sequencing and medium cluster-gate/status findings fixed; final re-review no findings. Source/doc consistency review: medium internal-04 rejection-reason and low JA companion-link findings fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Docs-only. Creates paired crate plan and ledger, classifies initial `spec_gap`, `test_gap`, `design_drift`, `source_drift`, `external_dependency_gap`, `deferred`, and `repo_metadata_conflict` state, records kernel prohibitions and trusted-baseline lint policy, strict linear task sequencing, internal-04 rejection reason coverage, and cluster trace external-readiness gates, and does not create crate source. |
| 1. Crate scaffold and trusted-baseline lint policy | complete | `63cbcd83a82005d8ffe98f7c87928fa46e95649c` | Spec/doc review: medium public-surface and dependency-escape findings fixed; low TODO/ledger timing finding resolved by final ledger update. Test sufficiency review: medium dependency-subtable and low workspace-member scanner findings fixed; final re-review no findings. Full implementation review: high task-0 hash, medium dependency-subtable, medium split-public-surface, and medium extern-ABI public-surface findings fixed; final re-review no findings. Source/doc consistency review: medium dependency-guard and low trusted-baseline decision findings fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Scaffold-only. Adds workspace member, lockfile entry, minimal crate manifest, `#![forbid(unsafe_code)]` crate-root trust statement, and lint-policy guard. Production dependencies are exactly `mizar-core` and `mizar-session`; dev/build/target dependency sections, public semantic surface, downstream ATP/proof/cache/artifact coupling, module specs, semantic modules, `.miz` fixtures, expectations, and `doc/spec` edits remain absent/out of scope. |
| 2. Spec: `clause.md` | complete | `b0fa89a9eecc85da96bf8351fc2e147423747730` | Spec/doc review: high empty-clause, medium test-coverage, low trust-prohibition, and medium validation-context signature findings fixed; final re-review no findings. Test sufficiency review: medium planned-test coverage, medium hash-test coverage, and low symbol-kind ordering coverage findings fixed; final re-review no findings. Full implementation review: high empty-clause plus medium tautology-marker, validation-context, and canonical-ordering findings fixed; final re-review no findings. Source/doc consistency review: medium validation-context signature finding fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired clause module specs for literals, canonical ordering, structural well-formedness, explicit empty and tautology forms, clause-local validation context, deterministic rendering/hash inputs, trust statement, planned task-3 tests, and clause-specific gaps. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. |
| 3. Implement clause representation | complete | `4020ac12fafe24aa8205f7fd3df8ece37027804e` | Spec/doc review: medium public `Term` ordering and medium clause-hash preallocation findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: high canonical-order coverage, medium hash-field and hash-exclusion coverage, and low marker/single/empty coverage findings fixed; final re-review no findings. Full implementation review: medium `Term` ordering, medium unchecked length casts, low missing `#[non_exhaustive]`, and medium preallocation resource-bound findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium `Term` ordering drift fixed; final re-review no blocking findings, with ledger/TODO backfill completed in this task. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task. Implements `clause` data model, validation context, deterministic ordering/rendering/hash input, explicit empty and zero-payload tautology marker forms, checked canonical byte framing, resource-bound validation before large allocation, module exposure, and lint guard update. No SAT/ATP/proof search, downstream ATP/proof/cache/artifact coupling, `.miz` fixtures, expectations, or `doc/spec` edits. Cross-crate `mizar-core`/`mizar-checker` tests not required because this task does not touch the binder contract or checker/trace boundary. |
| 4. Spec: `certificate_parser.md` | complete | `b900639e4057ea2ba1a1158688a35e188ec` | Spec/doc review: high concrete-byte/schema gap and imported proof-status gap plus medium hash-algorithm and failure-location findings fixed; later high imported-fact id and medium non-parent `ClauseRef` findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: high stable-failure and hash-coverage findings plus medium reference/resource/ordering findings fixed; final re-review no findings. Full implementation-boundary review: high concrete-byte gap and medium generated-clause/hash-dependency findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium task-3 backfill and schema-ownership/TODO findings fixed; final re-review no blocking findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired certificate parser specs with schema ownership, concrete schema/encoding v1 envelope, section tags, directory and item payload layouts, parser-owned manifests, public clause integration, stable failure detail/location mapping, hash-input rules without digest dependency, planned task-5 tests, and `external_dependency_gap`/`deferred` records. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. |
| 5. Implement certificate parsing and structural validation | complete | `60c92cc53c77ec3240fe5410fc04c449bd04b267` | Spec/doc review: final re-review no findings after EN/JA clarification for range out-of-bounds and noncanonical/shuffled hash-input semantics. Test sufficiency review: medium location/hash/resource/ordering gaps and later medium resolution-step ordering gap fixed; final re-review no findings. Full implementation review: high resource preallocation and generated-literal budget findings plus medium term-budget and item-location findings fixed; final re-review no findings. Source/doc consistency review: final re-review no findings, with only expected pre-commit bookkeeping noted. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-core` and `cargo test -p mizar-checker` were not run because this task does not alter the `mizar-core` binder contract or the checker/trace boundary semantics; it only decodes certificate bytes and delegates generated clause validation to the existing local `clause` API. | Rust source task. Implements schema/encoding v1 envelope parsing, deterministic fixed-section directory validation, item-frame parsing, parser-owned manifest/reference/schema types, generated clause structural validation through `clause`, stable rejection category/detail/location reporting, canonical hash input bytes, parser resource limits before large allocation, and expanded lint coverage. Updates paired certificate parser docs only to clarify implemented range/hash semantics. No SAT/ATP/proof search, producer/cache/artifact coupling, `.miz` fixtures, expectations, or `doc/spec` edits. External producer/consumer integration remains `external_dependency_gap`/`deferred`. |
| 6. Spec: `rejection.md` | complete | `f4b1abc63a46cd7d628911aff4a7ce91c0c5555b` | Spec/doc review: medium EN/JA planned-test sync, context-mismatch/profile ambiguity, target sort-key, cluster mapping, witness-normalization ownership, and ordering-id findings fixed; final re-review no findings. Test sufficiency review: medium ordering, checker-location, category/detail ownership, parser target-key fallback, `clause_ref`, and checker mapping findings fixed; final re-review no findings. Full implementation-boundary review: medium profile ambiguity, imported proof-status, internal-04 reason coverage, target sort-key ownership, witness-normalization ownership, and planned mapping wording findings fixed; final re-review no findings. Source/doc consistency review: high context-mismatch drift and medium domain-separator mapping finding fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired rejection specs defining `certificate_rejection`/`kernel_rejection`, stable detail keys, parser/checker mappings, target-owned deterministic record ordering, evidence locations, compatibility policy, planned task-7 tests, and internal-04 `missing_provenance`/`malformed_witness_data` coverage. Also syncs certificate parser docs to include unsupported domain separator mapping. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. Downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. |
| 7. Implement rejection records | complete | `acc8e7d62adbee21cb49b8d134fe0d846ee60603` | Spec/doc review: medium fixed-width target sort-key and evidence-id ordering/doc-sync findings fixed; final re-review no findings. Test sufficiency review: medium parser conversion, category/detail mapping, ordering, and lint coverage findings plus low atomic-token and isolated category/byte-offset/section ordering findings fixed; final re-review no findings. Full implementation review: high public-field constructor-bypass finding fixed; final re-review no findings. Source/doc consistency review: low reduction/derived/final ordering coverage finding fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-core` and `cargo test -p mizar-checker` were not run because this task does not alter the `mizar-core` binder contract or checker/trace replay boundary; it only adds shared rejection record types and parser-error conversion over existing parser data. | Rust source task. Implements stable rejection categories/details, category/detail ownership validation, target VC fingerprint ordering, structured evidence locations, parser-error conversion, read-only rejection records, deterministic total ordering, module exposure, and trusted-boundary lint expansion. Syncs paired rejection docs only for implemented ordering details. No SAT/ATP/proof search, fallback inference, implicit coercion insertion, global mutable state, downstream proof/cache/artifact coupling, `.miz` fixtures, expectations, or `doc/spec` edits. Downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. |
| 8. Spec: `resolution_trace.md` | complete | `0b017553b3462eb78492d3aa84053b9d07a2fae4` | Spec/doc review: medium imported-parent context and final-goal checkedness findings fixed; initial stale sequencing concern resolved by clean status/log; final re-review no findings. Test sufficiency review: medium rejection-record shape and provenance planned-test findings plus low final-goal checkedness coverage finding fixed; final re-review no findings. Full implementation-boundary review: high preallocation and term-depth findings plus medium clause-context, clause-owned helper, imported-context validation, and resource-classification findings fixed; final re-review no findings. Source/doc consistency review: medium internal-04 `MissingProvenance` drift and low remaining internal-04 rejection-detail drift fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired resolution-trace specs defining deterministic MiniSAT-compatible replay, explicit imported clause context, clause-reference ownership, pivot orientation, bounded resolvent construction, clause-owned non-allocating/depth-bounded helper requirements, final-goal checkedness, replay rejection mapping, planned task-9 tests, and `external_dependency_gap`/`deferred` records. Syncs internal-04 EN/JA `RejectionReason` sketch with `MissingProvenance`, `MalformedWitnessData`, and `InvalidClusterTrace`. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. Backend proof translation and proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. |
| 9. Implement resolution trace checker | complete | `28b7e7122c8cad04a6526d8de8cdfd0394d8bb3c` | Spec/doc review: medium clause-helper documentation and low imported-context/report-shape findings fixed; final re-review no findings. Test sufficiency review: generated/imported/previous-step parent resource ordering, report binding, generated/resolution-step final goals, nested canonical-length helper coverage, and stable rejection coverage reviewed through final re-review with no findings. Full implementation review: high pre-clone resource-ordering findings and medium report binding, previous-step parent clone-order, and stale docs findings fixed; final re-review no findings. Source/doc consistency review: low `ResolutionTraceInput` target-fingerprint sketch finding fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task. Implements deterministic resolution trace replay, explicit imported clause context, checked-step reports, final-goal helper with private replay binding, stable kernel rejection mapping, bounded borrowed parent validation before clone, bounded resolvent accumulation, term-depth and canonical-length helpers in `clause`, cfg(test) certificate fixture construction, module exposure, and lint guard updates. No SAT solver, ATP/proof search, premise selection, overload resolution, cluster search, implicit coercion insertion, fallback inference, global mutable state read, downstream ATP/proof/cache/artifact coupling, `.miz` fixtures, expectations, or `doc/spec` edits. Imported-fact availability/fingerprint/proof-status auditing, backend proof translation, checker orchestration, and proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. |
| 10. Spec: `substitution_checker.md` | complete | `d79506c6e0b7029fb1512454b0eff72579362df7` | Spec/doc review: binder-context authority, duplicate context classification, byte grammar, side-condition schemas, duplicate witness classification, local-abbreviation guard evidence, and captured-free-variable boundary findings fixed; final re-review no findings. Test sufficiency review: resource, binder-decode, side-condition, rejection-location, report-binding, prohibition, manifest, first-use, trailing-byte, unused witness, payload-positive, rewrite-path, and payload actual-term coverage findings fixed; final re-review no findings. Full implementation-boundary review: high missing explicit payload finding plus nested-resource, task-split, report-binding, and captured-free-variable findings fixed; final re-review no findings. Source/doc consistency review: final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired substitution checker specs with explicit immutable substitution payload context, binder-context grammar, payload/replacement rules, side-condition evidence schemas, replay limits, direct-substitution versus alpha/FV task split, stable rejection mapping, planned task-11/task-12 tests, and kernel prohibitions. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. Inline certificate encoding of substitution payloads, local-abbreviation closure/type-guard evidence, captured-free-variable closure evidence, source-derived substitution certificates, and downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`; missing or deferred payload evidence is rejected rather than inferred. |
| 11. Implement substitution checking | complete | `b97c4a3a700fec986d3e203b1a88d23edcfba7f3` | Spec/doc review: deterministic-counter ownership, under-binder task split, planned-test sync, and replayed-target resource-test findings fixed; final re-review no findings. Test sufficiency review: binder decode matrix, side-condition shape, resource limits, malformed/deferred payloads, precise locations, duplicate context ids, lint tokens, actual-term byte limits, ambiguous context fallback, and side-context canonicalization findings fixed; final re-review no findings. Full implementation review: high binder-frame compatibility, pre-clone replay budget, binder-aware capture, binder-context count preallocation, and lookup complexity findings fixed; final re-review no findings. Source/doc consistency review: replayed target depth coverage and context fallback consistency findings fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-core` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-checker` was not run because this task does not change checker/trace boundary orchestration. | Rust source task. Implements direct substitution replay over explicit immutable payload evidence, binder-context decoding with deterministic byte/count/frame checks, manifest and frame/term compatibility checks, first-use context lookup and fallback, payload validation, direct capture rejection, pre-clone replay budget measurement, side-condition shape/owner/path/resource validation, deterministic checked-substitution reports, module exposure, and lint coverage. Alpha-conversion semantics, semantic freshness/free-variable replay, local-abbreviation closure/type-guard evidence, captured-free-variable closure evidence, inline payload certificate encoding, source-derived substitution certificate production, checker orchestration, and downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`; missing/deferred evidence is rejected rather than inferred. |
| 12. Implement alpha-conversion, freshness, and free-variable checks | complete | `577f6f220b93d94c9796208829216f43a8e2e3d4` | Spec/doc review: initial blocking deterministic-freshness, alpha-correspondence, and capture-set derivation gaps plus medium TODO/ledger drift findings fixed by paired documentation updates; final re-review no blocking/high findings. Test sufficiency review: high unreferenced-witness, bound-renaming, stale-avoided-set, captured-free-variable, and shuffled-rejection coverage findings fixed; final re-review no findings. Full implementation review: medium recomputed avoided/capture-set resource findings and avoided-limit exclusion edge fixed; final re-review no findings. Source/doc consistency review: medium Task 12 status drift finding fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-core` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-checker` was not run because this task does not change checker/trace boundary orchestration. | Rust source task. Implements semantic alpha-conversion over referenced freshness witnesses, deterministic freshness counter replay from manifest-order candidate streams, avoided-variable recomputation with resource bounds, target capture-set recomputation with resource bounds, free-variable side-condition replay, non-capturing under-binder substitution acceptance, deterministic checked reports/rejections under shuffled context construction, paired architecture/substitution docs, and Task 11 hash backfill. Local-abbreviation closure/type-guard evidence, captured-free-variable closure evidence, inline payload certificate encoding, source-derived substitution certificate production, checker orchestration, and downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`; missing/deferred evidence is rejected rather than inferred. |
| 13. Spec: `checker.md` | complete | `865231081df7538faea132c499d9c57d5ecfa9cb` | Spec/doc review: medium reduction strategy-audit field finding fixed, then imported-clause fingerprint binding, derived-fact payload authority, budget split, planned-test, and status fixes reviewed with no blocking/high/medium findings. Test sufficiency review: medium full-prohibition coverage, deterministic rejection coverage, imported-clause fingerprint planned tests, resource/timeout split planned tests, and derived-payload authority planned tests fixed; final re-review no findings. Full implementation-boundary review: high imported-clause content binding plus medium derived-payload authority and budget mapping findings fixed; final re-review no findings. Source/doc consistency review: medium budget mapping and TODO/ledger status findings fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired checker module specs for the kernel check-service pipeline, imported fact context with normalized clause fingerprint binding, no caller-supplied imported-clause context, cluster/reduction trace boundary, certificate-owned derived-fact payload authority, derived facts, final-goal acceptance, deterministic rejection/resource mapping, planned task-14/15/16 tests, and deferred external integrations. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. Source-derived certificates, ATP proof translation, cluster trace payload production by `mizar-checker`, and proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`; missing producer/consumer integration is not mocked. |
| 14. Implement imported-fact checking | complete | `874881b42d5c008336a34cb4cfaf24f7b403a1fb` | Spec/doc review: high missing fingerprint algorithm semantics and medium external-attestation profile gate findings fixed with task-14 algorithm id 1 and explicit imported-fact policy; low input-shape note fixed; final re-review no findings. Test sufficiency review: medium evidence statement-fingerprint, proof-status matrix, variable-manifest, unsupported evidence algorithm, and context/report canonicalization findings fixed; final re-review no blocking/high/medium findings. Full implementation review: medium canonical-byte allocation-before-budget and unbounded context sorting findings fixed by non-allocating length precheck and bounded context constructor; final re-review no findings. Source/doc consistency review: medium proof-status ordering and resource-ordering drift fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-kernel` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task. Implements imported-fact evidence context, bounded context canonicalization, proof-status/profile-policy checking, task-14 normalized clause fingerprint algorithm id 1, normalized clause fingerprint binding with pre-allocation resource checks, imported clause context construction for resolution replay, task-13 hash backfill, and lint exposure for the spec-backed checker module. Source-derived imported contexts, digest registry beyond task-14 algorithm id 1, full check-service orchestration, cluster replay, proof-policy projection, and downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. No SAT/ATP/proof search, premise selection, overload resolution, cluster search, implicit coercion insertion, fallback inference, global mutable state, `.miz` fixture, expectation, or `doc/spec` edits. |
| 15. Implement cluster trace replay | complete | `77262c0d890026dd614225f2d762861fe192169b` | Spec/doc review: medium requested-id pipeline wording and earlier reduction authority/order/context wording findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: high transitive requested-only/global-order coverage plus medium base-fact, guard exact-match, context canonicalization, and resource coverage findings fixed; final re-review no blocking/high/medium findings. Full implementation review: high reduction binding/commitment resource-bound finding plus medium unused-context runtime-limit, missing dependency location, stale docs, and required-guard commitment wording findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium reduction-authority docs, unused-context runtime-limit wording, stale deferred wording, and required-guard commitment sync findings fixed; final re-review no blocking/high/medium findings. | `cargo fmt --check` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-kernel` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task. Implements bounded requested cluster/reduction trace replay, global cluster/reduction id namespace, requested-id transitive dependency closure in global order, explicit checked imported/generated/earlier-trace dependencies, bounded context canonicalization, unused-context ignore semantics after construction, normalized cluster fact / reduction audit / reduction result commitments including required guards, stable rejection mapping and locations, task-14 hash backfill, and focused fail-heavy tests. Rich producer-side active-rule payload validation, source-derived cluster trace production, semantic redex/LHS-to-RHS validation beyond normalized commitments, full check-service orchestration, proof-policy projection, and downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. No SAT/ATP/proof search, premise selection, overload resolution, cluster search, implicit coercion insertion, fallback inference, global mutable state, `.miz` fixture, expectation, or `doc/spec` edits. |
| 16. Kernel check service and deterministic batch ordering | complete | `c0b8e6104f38d02e7bf8f6c1cda5900fb50bdfc1` | Spec/doc review: medium batch ordering, external rejection scope, task-15 ledger, TODO status, fail-fast wording, and derived-fact table findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: high nonempty substitution service coverage and service-level final-goal helper coverage fixed, plus medium later timeout, derived-fact resource limit, and fail-fast wording coverage fixed; final re-review no blocking/high/medium findings. Full implementation review: high imported axiom/theorem namespace collapse fixed by namespaced `CheckedFactRef`/`CheckedFactContext` and namespaced `used_axioms`; medium substitution report materialization ordering and generated-clause base-set duplicate findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium planned-test wording, TODO/ledger drift, and TODO batch wording findings fixed; final re-review no blocking/high/medium findings. | `cargo fmt --check` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-kernel` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task. Implements deterministic `KernelCheckInput`/`KernelCheckResult`, single-certificate service orchestration across imported facts, substitutions, resolution replay, requested cluster/reduction replay, fail-closed derived facts, final-goal binding through the resolution helper, namespaced imported axiom/theorem fact references, generated-clause base-set deduplication for cluster replay, policy taint propagation, checker step timeout, report-record resource limits, deterministic batch ordering by target then caller order, paired docs/TODO updates, and focused service tests. Derived-fact payload schemas, source-derived certificate/service envelopes, cancellation token plumbing, external worker scheduling, proof-policy projection, and downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. No SAT/ATP/proof search, premise selection, overload resolution, cluster search, implicit coercion insertion, fallback inference, global mutable state, `.miz` fixture, expectation, or `doc/spec` edits. |
| 17. Soundness fail-test corpus | complete | `b7e1493050ed49110e4ddf7a7a75d971bdf72c59` | Spec/doc review: medium assertion identity and service-path scope findings fixed in the implementation spec; final re-review no blocking/high/medium findings. Test sufficiency review: medium service-path reduction mutation and strict timeout/resource identity findings fixed; final re-review no blocking/high/medium findings. Full implementation review: no blocking/high/medium findings. Source/doc consistency review: no blocking/high/medium findings. | `cargo fmt --check` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-kernel` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Test/audit task. Adds a service-path single-mutation soundness fail corpus covering imported fact identity, substitution replay, resolution replay, cluster trace replay, reduction trace replay, final-goal binding, derived-fact fail-closed behavior, timeout, and resource exhaustion. Every corpus case asserts rejected status, category, detail, stable detail key, exact location, exactly one rejection, and no partial accepted outputs. Source-derived corpus runners and external producer fixtures remain `external_dependency_gap`/`deferred`; no `.miz` fixture, expectation, `doc/spec`, SAT/ATP/proof search, premise selection, overload resolution, cluster search, implicit coercion insertion, fallback inference, global mutable state, or external dependency mock is added. |
| 18. Determinism and replay-cost suite | complete | `3d1942e97ea245d2fae09dac4e26cefd67c02bd1` | Spec/doc review: medium batch-determinism ambiguity fixed by separating distinct-target equality from equal-target caller-order tie preservation; low Task 17 backfill and stable-rendering wording findings addressed. Final spec re-review no blocking/high/medium findings. Test sufficiency review: no findings. Full implementation review: no blocking/high/medium findings. Source/doc consistency review: no findings. | Focused Task 18 tests passed; `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `cargo test -p mizar-core` passed; `cargo test -p mizar-checker` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Test hardening task. Adds deterministic equality coverage for repeated single checks, shuffled imported/cluster context construction, shuffled requested trace ids, distinct-target batch permutations, equal-target tie preservation, and stable rejection keys/locations as the public rendering surrogate. Adds exact replay-cost assertions for cluster/reduction trace counts and checker pipeline/report budgets. Source-derived runners, benchmarks, randomness, property-test dependencies, external producer fixtures, `.miz` fixtures, expectations, `doc/spec`, SAT/ATP/proof search, premise selection, overload resolution, cluster search, implicit coercion insertion, fallback inference, global mutable state, and downstream ATP/proof/cache/artifact coupling remain out of scope. |
| 19. Public-enum forward-compatibility policy | ready to commit | pending self-hash | Spec/doc review: medium documentation/inventory guard finding fixed by making paired `public_enum_policy.md` the canonical exact enum inventory and lint-checked EN/JA source match; low rejection compatibility wording and bookkeeping findings addressed. Final spec re-review no blocking/high/medium findings. Test sufficiency review: no findings. Full implementation review: no blocking/high/medium findings. Source/doc consistency review: no blocking/high/medium findings. | `cargo test -p mizar-kernel kernel_public_enums_are_forward_compatible_and_documented` passed; `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-core` and `cargo test -p mizar-checker` were not required because this task adds public API compatibility annotations and lint/docs only, without changing binder contracts or checker/trace replay semantics. | Docs/test/source-annotation task. Classifies every public enum as forward-compatible with no exhaustive exceptions, adds missing `#[non_exhaustive]` markers, and broadens lint coverage to require both immediate markers and exact source-to-policy inventory synchronization. Rejection category/detail stable-key spelling, meaning, phase ownership, ordering, removal, rename, and remapping remain compatibility-reviewed. No variant changes, runtime behavior changes, `doc/spec`, `.miz`, expectation, SAT/ATP/proof search, premise selection, overload resolution, cluster search, implicit coercion insertion, fallback inference, global mutable state, or downstream ATP/proof/cache/artifact coupling is added. |
| 20. Source/spec correspondence and prohibition audit | not started | pending | pending | pending | Requires task 19 commit. Audit task. |
| 21. Bilingual documentation sync audit | not started | pending | pending | pending | Requires task 20 commit. Docs audit task. |
| 22. Module-boundary refactor gate | not started | pending | pending | pending | Requires task 21 commit. Audit or move-only task. |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | Requires task 22 commit, all hard gates passing, and read-only quality review score >= 90/100. |

## Task 11 Handoff

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

Rationale: task 12 completes the soundness-critical binder side of
substitution replay by accepting only explicitly justified alpha/freshness/FV
evidence. Keep `xhigh`; lower only for bookkeeping-only docs sync, and raise
only if architecture 16 and the implemented task-11 context model conflict.

## Task 10 Handoff

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

Rationale: task 11 starts binder-sensitive trusted kernel source and must
remain deterministic evidence replay over explicit payloads, not inferred
substitution search. Keep `xhigh`; lower only for typo-only docs sync, and
raise only if existing `mizar-core` binder APIs conflict with the explicit
kernel-owned recheck boundary.

## Task 9 Handoff

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

Rationale: task 10 specifies the binder-sensitive checker that later Rust
implementation tasks must follow. Keep `xhigh` because substitution, alpha
conversion, and free-variable side conditions are soundness-critical and must
remain explicit certificate replay rather than inference. Lower reasoning is
appropriate only for typo-only docs sync; raise only if architecture 16 and the
current certificate schema contradict each other.

## Task 8 Handoff

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

Rationale: task 9 implements the first semantic replay checker inside the
trusted kernel, so allocation bounds, depth bounds, parent orientation, and
stable rejection locations are soundness-critical. Keep `xhigh`; lower only for
comment-only follow-up, and raise only if the existing clause/parser APIs make
the specified helper boundaries impossible without a broader design update.

## Task 7 Handoff

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

Rationale: task 8 defines the replay contract that the trusted checker will
implement in task 9. Keep `xhigh` because trace replay is part of the soundness
boundary and must stay a deterministic evidence checker rather than a search
or solver. Lower reasoning is appropriate only for typo-only docs sync; raise
only if architecture 15 or the existing certificate schema contradicts the
planned trace model.

## Task 6 Handoff

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

Rationale: task 7 turns the stable failure vocabulary into the shared record
surface that later checkers consume. Keep `xhigh` because category/detail
ownership, target-owned ordering, and parser conversion are part of the trusted
soundness boundary. Lower reasoning is appropriate only for typo-only docs
sync; raise only if parser APIs or architecture 19 conflict with the spec.

## Task 5 Handoff

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

Rationale: task 6 specifies the stable rejection vocabulary consumed by every
later checker. Keep `xhigh` because rejection categories are part of the trusted
boundary and architecture 19 failure-semantics compatibility policy. Lower
reasoning is appropriate only for typo-only synchronization; raise only if the
architecture documents contradict the existing parser rejection surface.

## Task 0 Handoff

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

Rationale: task 1 creates the trusted crate boundary and dependency guard that
all later kernel work relies on. Keep `xhigh` because dependency discipline,
trusted lint policy, and no-search/no-ATP boundaries are soundness-critical.
Lower reasoning is appropriate only for typo-only documentation cleanup; raise
only if repository metadata or contradictory specifications block the scaffold.
