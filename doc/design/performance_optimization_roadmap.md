# Performance optimization roadmap

Canonical language: English. Planning inventory, not a second dispatch queue. The [top-level TODO](./todo.md) Sequential Execution Plan remains the ordering authority. PERF-A01 and later implementation work dispatch only after the VC53-E1 chain completes; A01 measures the post-chain sealed revision and its ranking selects the next package. Owner-gated scope remains excluded. Audit base: `5593498e1d43240af96368732b2dfc93392f65a6` (2026-11-23). Findings are static observations, not measured speedups. Rows are work packages, not permission to dispatch unfrozen implementation contracts. Reconcile current local/in-flight work, verify each finding against current code and freeze a paired owner contract before dispatch. Existing owner tasks take precedence over duplicate work.

## Principles

- Preserve FOL semantics, acceptance policy, independent Kernel checking, canonical external outputs, diagnostics and provenance. No skipped trust checks, fabricated proof status, coverage changes or arbitrary new resource limits for speed.
- Reuse immutable producer results; do not replay upstream computation merely to cross a process-local owner boundary. Disk/untrusted payloads require validated context and structural/semantic authority as appropriate.
- Measure before and after on identical workloads; distinguish cold/warm/edit, test compile/run and product runtime. Record results outside doc; no numeric speed gate before baselines. Preserve local receipt rules and active test obligations.
- A (development velocity and baseline) first; B (structural debt) before broad dependent implementations where practical; C (Step 6/7 integration) follows actual producers, with C05 developed alongside C01/C03. D/E are measured optimization work, not blanket post-completion deferrals: early design and readiness exceptions below are mandatory planning priorities. No global stop for a blocked package; retain M1/MVM dispatch constraints.

## Existing work to reuse

| Existing owner work | Treatment |
|---|---|
| `VELOCITY-LARGE-FILE-SPLIT`, `VELOCITY-GENERIC-SOURCE-TRANSPORT`, `VELOCITY-ELEMENT-RULE-TRANSPORT`, `QUALITY-METAMORPHIC-GUARD` | Preserve completed scope; actual compilation targets, generic corpus and reusable fixtures are residual work. |
| Driver tasks 53/57/60/63/68/72/76/84/93/97/101 | Retain completed replay handoffs; B01 changes storage mechanism, not semantic completion. |
| Build 24/28, Driver 16, Test 14, Cache 20/24, Proof 17/21, VC 20/26 | Reuse equivalence and fingerprint contracts; C01/C03 extend real execution. |
| `ATPK-INSTANCE-FINDER`, `ATPK-KERNEL-EQUALITY-INSTANCES`, `ATPK-PREMISE-RANKING`, `ATPK-BENCHMARK` | Existing Step 7 owns backend integration and success-rate evaluation; D03 is an early design/integration priority, D04/D05/D07 address distinct measured or architectural residuals. Do not duplicate existing producers. |
| Lexer benchmark, Frontend 27, Checker 14/16/17/23/25, Core seed memoization and normalized type interning | Extend implemented baseline; do not restart. |

## Work packages

| ID | Primary owner | Scope | Dependencies / readiness | Acceptance intent |
|---|---|---|---|---|
| PERF-A01 | mizar-test | Baseline development and product performance | VC53-E1 chain complete and sealed | Cargo timings: clean/no-change/test edit/owner edit; separate compile/link/run/Clippy; opt-in phase counters and RSS for supported native routes. |
| PERF-A02 | mizar-test | Split actual test compilation targets | A01 ranking selects A02 | One measured large family into independently buildable targets; preserve private unit checks, assertions and inventory; compare costs; no mere file split. |
| PERF-A03 | mizar-test | Generic data-driven corpus runner and admission | A01 | Use real owner/Driver stages, explicit capabilities and shared assertions; retire fixture-ID routing only when genuine generic route exists; preserve failure and coverage. |
| PERF-A04 | mizar-test | Shared test fixtures, plan and mutations | A01; A03 where applicable | Share immutable plans and fixture bases; isolate mutations; index doc headings, separate policy checks when useful; bounded parallel corpus execution and stable reports. |
| PERF-A05 | mizar-test | Local feedback and Cargo tuning | A01, A02 | Measure debug info, linker, target reuse, scoped Clippy/test selection and repeated full-corpus determinism/mutation costs. Propose frequency changes only with preserved detection capability and owner approval of receipt/full-test policy; keep affected checks mandatory. |
| PERF-A06 | mizar-test | Component scaling and end-to-end baseline benchmarks | A01 measurement method; each genuine supported route | P0: geometric component workloads and small genuine end-to-end source-to-artifact benchmarks before relevant optimizations; separate phase/cold timings and memory/clone/call counts. Extend existing Criterion; unavailable routes remain explicit gaps, not fabricated coverage. |
| PERF-A07 | mizar-test | Storage and retrieval comparative benchmarks | A01/A06 measurement method; B07 candidate interfaces; baseline may precede B07 | P0: compare JSON, compressed JSON, ordinary binary, interned/DAG binary, indexed/mapped variants and selective compression; quantify total bytes including indexes, peak RSS/PSS, cold/warm import, build/update, retrieval latency/throughput and proof outcomes. Separate supported genuine paths from synthetic microbenchmarks. |
| PERF-A08 | mizar-test | Declarative statement inference benchmark | VC53-E1 sealed; A01/A06 method; genuine supported routes | P0, benchmark first: profile actual statement/VC workloads and proof-context changes; compare existing discharge, local algorithms, isolated/persistent backend, internal prover prototypes and evidence replay on identical allowed premises. Measure latency distribution, costs per phase, accepted success, failed attempts, memory and end-to-end throughput; no artificial acceptance. |
| PERF-B01 | mizar-driver | Replace routine source replay in IR storage | A01 | Inventory genuine theorem/algorithm paths; owner structural codecs or process-local sealed payloads; migrate by stage; count producer calls on normal encode/get. |
| PERF-B02 | mizar-ir | Share resident and decoded immutable IR | A01, B01 interface design | Avoid repeated hash/decode of immutable in-process bytes; exact schema/type/context/decoder identity; single-flight as safe; bounded active residency. |
| PERF-B03 | mizar-resolve | Real indexes and scope lookup | A01 | Replace insertion sorts and linear Symbol/Label/Definition/Registration/etc queries; index bindings and omitted witnesses; preserve duplicate, shadowing and recovery rules. |
| PERF-B04 | mizar-checker | DAG-preserving type canonicalization | A01 | Replace recursively owned StructuralTypeKey expansion with canonical child identities, deterministic order and collision-safe equality; preserve source/status merge; shared-subtype scaling. |
| PERF-B05 | mizar-checker | Share typed arena, contexts and handoffs | A01; B04 as applicable | Persistent context deltas, indexed BindingEnv, immutable side tables; avoid clone/reinstall and upstream recomputation without weakening owner checks. |
| PERF-B06 | mizar-core | Shared IR and lowering indexes | A01; B05 as applicable | Index TypedNodeId/TermId/FormulaId/references; preserve seed memoization; measure and reduce existing table clones, re-indexing and repeated lowering validation; evaluate Term/Formula hash-consing with semantic context and separate provenance. Measure Algorithm ProgramContext clones, branch-join precision and VC size growth; coordinate VC owner, preserve old-state and path semantics. |
| PERF-B07 | mizar-artifact | Integrated persistent knowledge and retrieval architecture | design after VC53-E1/A01, before affected structural implementation; A07 baseline as available | P0: compare ATP/ITP research and source implementations, choose shared semantic identity, module/package layout, canonical binary public format, static/dynamic indexes, validation and migration boundaries; review before B01/B02/B04/B06 persistence contracts. Design only; no schema or trust change here. |
| PERF-B08 | mizar-core | Term Bank and retrieval primitives | B07 reviewed design, B04/B06 interfaces, A07 baseline | P0: share canonical typed nodes and occurrence references; distinguish exact/variant, generalization, instance, unification and subterm relations; prototype discrimination/fingerprint/substitution/path/code-tree alternatives as workload warrants; verify against exhaustive reference with binders, type/context and visibility checks. B04/B06 own existing arena implementation, not duplicate banks. |
| PERF-C01 | mizar-build | Actual worker execution and event-driven DAG | A01, B02, Step 6 | Bounded concurrent workers, ready queue, no artificial batch barrier; preserve dependency/resource/cancellation and canonical commits; prove actual overlap and measure module/VC scaling with C05. |
| PERF-C02 | mizar-ir | Last-use collection and memory budgets | B02, C01 lifecycle design | Scheduler-aware retain/release, snapshot indexes, blob refcounts; shorten Publisher/Storage lock-held hashing and collection; bounded resident/encoded memory, disk spill as needed; protect LSP/commit owners. |
| PERF-C03 | mizar-driver | Module and VC incremental reuse | A01, Step 6/7 accepted producers; C04 shared design | Module then declaration/VC reuse; integrate ProofReuseValidator and RebuildTriggerEvaluator where current-callsite review confirms gaps; unchanged inputs avoid ATP; kernel skip only with reviewed trusted acceptance contract, otherwise replay witness. Distinguish contract, definition and execution dependencies; reuse callers only when contracts and accepted guarantees remain valid, and measure local edits. |
| PERF-C04 | mizar-cache | Precise dependency and session reuse | A01, B03; shared C03/C04 design | Interface vs implementation and negative lookup/registration dependencies; integrate rebuild decisions, narrow frontend keys, reverse edges; verify/read immutable artifacts once per snapshot; VC slice index/shared fingerprints with VC owner; reuse import-scoped views with D01. |
| PERF-C05 | mizar-test | Incremental and parallel verification benchmarks | A06; corresponding C01/C03/C04 producer availability, not all packages complete | P0: establish cold/warm/edit and worker-count baselines as each real route becomes available; quantify module/VC reuse, ATP/Kernel work avoided, dependency invalidation, actual concurrency, RSS and equivalence. Grow workload geometrically; do not defer to E04. |
| PERF-C06 | mizar-cache | Persistent library indexes and scoped consumer integration | B07, B08 relevant interfaces, E03 section reader as needed; B03/C04/D01/D02/D03/D05 owner slices | P0: immutable per-module index segments and dynamic local overlays, FQN/label/prefix/type/symbol and structural retrieval, origin filtering and dependency invalidation; no cloned global graph per import. Integrate LSP/doc/ATP/Checker consumers through their existing owners; rank separately from exact retrieval; optional natural-language search is P2, advisory and regenerable. |
| PERF-D01 | mizar-checker | Cluster and overload search | A01, B03, B04 for implementation; early design independent of them | Index trigger/registration ID and type facts; replace repeated full-rule scans with worklist saturation; measure import-scoped view construction, filtering, cloning, reuse and invalidation with C04; site grouping, viability filters and context-safe specificity reuse; preserve traces, ambiguity and termination. Evaluate state-versioned type facts, safe cross-VC context reuse and reduction in solver obligations; preserve visibility and proof evidence. |
| PERF-D02 | mizar-checker | Template and reduction reuse | A01, B04, B06 for implementation; genuine producers; early design before generalization | Assess sharing scope; separate use-site identity from semantic instantiation keys (template, actuals, relevant environment); share instantiated signatures across sites/modules only under equivalent dependencies and keep occurrence provenance separate. Design rule indexes, rewrite strategy, normalization and termination before extending Reduction; measure safe memoization, no eager global expansion. Measure evidence-backed symbolic reduction on array/word workloads when genuine producers exist; respect the specified decreasing order and guard evidence. |
| PERF-D03 | mizar-atp | Evidence-driven ground instance search | early design now; A01 and genuine Step 7 producers for implementation | P0 design/integration: inventory actual Vampire substitution/inference extraction, exploit validated advisory information where available; avoid blind Cartesian enumeration, repeated SAT and redundant equality instances; consider incremental untrusted SAT screening; deterministic bounded fallback; independent Kernel validation. |
| PERF-D04 | mizar-kernel | Formula and SAT processing | A01; D03/D07 interfaces as applicable | Early design for equality generation and formula/Tseitin sharing; measure equality density, repeated formula nodes and SAT cost, reduce formula cloning/reconstruction/reserialization across ATP/Kernel boundaries without sharing trust authority. Kernel 32 remains pending/trigger-based until a pinned `batsat` version change; certificate redesign belongs to D07. |
| PERF-D05 | mizar-atp | Backend and portfolio scheduling | C01, Step 7 | Bounded concurrent processes, load balancing and shared resource budgets; safe preparation reuse, deterministic selection; measure success rate and time, retain ranking and re-proving tasks. Measure premise ranking scans; evaluate symbol-set/frequency precomputation and inverted indexes scoped to the actual candidate pool; measure repeated process/version-probe overhead; coordinate persistent sessions with SMT-05. |
| PERF-D06 | mizar-proof | Template proof evidence reuse | early design before general Template proof mechanism; genuine template/Core/VC/evidence producers for implementation | Evaluate reuse of parametric proof evidence versus identical-instantiation cached evidence, binding/actual constraints, dependency and policy compatibility, witness identity and occurrence provenance. Revalidate substituted evidence at the independent Kernel; no automatic transfer of acceptance between distinct instantiations. |
| PERF-D07 | mizar-kernel | SAT search and certificate checking separation | early design before generalized Step 7; A01/A06 and D03/D04 inputs for evaluation | Compare current trusted SAT re-search against untrusted solver-generated UNSAT evidence checked independently (e.g. suitable certificate formats); measure generation/transfer/checking cost, memory and failure modes. Review formula/CNF binding, trusted base, malformed-proof rejection, determinism and compatibility; retain current sound path until separately approved and validated. |
| PERF-D08 | mizar-proof | Inference strategy and dispatcher design | A08 initial results; B07/B08 interfaces as available; coordinate D03–D07, SMT-01/03/05 | P0 design: automatic default, optional explicit strategy selection, measured bounded fallback, common candidate evidence boundary; compare local, embedded and external routes without changing proof acceptance or implicit premise scope. Choose implementation slices only after benchmark and review. |
| PERF-D09 | mizar-vc | Lightweight local inference and internal prover evaluation | A08 baseline, D08 design; B08 relevant retrieval primitives as needed | P0 comparison; implementation conditional: extend existing pre-ATP discharge with equality/congruence, indexed rewriting, propositional/Horn and bounded quantified inference; compare a small Metis-like first-order prover, not a new full ATP by default. ATP owns search adapters; Proof owns policy and Kernel owns evidence checking. |
| PERF-D10 | mizar-atp | Persistent and in-memory ATP interfaces | A08 baseline, D08 design; D05 and SMT-05 coordination | P0 comparison: isolated process, resident worker/IPC, shared-memory or library API where supported, translation reuse and context deltas. Investigate actual E/Vampire interfaces and licensing; measure isolation/reset/cancellation, startup, search and evidence overhead. SMT session implementation remains SMT-05; no assumed incremental ATP support. |
| PERF-D11 | mizar-proof | Algorithm-based programmable proof automation | A08 evidence, D08 design; genuine Algorithm/reflection/MVM and accepted evidence producers for implementation | P1 design: no independent tactic language; assess existing Algorithm/Template/reflected syntax and minimal justification/interface extension for guided inference and reusable proof algorithms. Compare explicit checked evidence and verified reflection; no arbitrary tactic runtime, untrusted status or privileged proof-state mutation. |
| PERF-D12 | mizar-proof | Fast checked evidence replay for statement verification | A08 replay baseline; C03/C04, D04/D07, E03/E05 as applicable | P0 design/measurement: separate discovery from rechecking; reuse validated evidence for exact obligation/context/policy, replay without re-running search where supported, measure reconstruction/check/IO separately. Coordinate existing reuse, witness and Kernel owners; no new acceptance authority or unconditional skip. |
| PERF-E01 | mizar-lexer | Lexical lookup and frontend token allocation | A01 | Index local declarations with activation positions; source spans where safe; batch contextual disambiguation; preserve multi-pass semantics unless equivalence shown. |
| PERF-E02 | mizar-parser | Parser checkpoints and lookup | A01 | Avoid cloning claimed-child/trivia state for speculation; delimiter indexes preserving recovery; measure nested formulas. |
| PERF-E03 | mizar-artifact | Canonical binary publication and internal storage migration | A01, B07 reviewed design, A07 comparison; B01/B02/B04/B06 interfaces as applicable | P0 design, measured staged implementation: binary as target for published semantic artifacts and persistent IR, JSON compatibility/export for tools. Compare fixed-width relative offsets, section directory, validated partial reads/mmap, portable versioned encodings, stable semantic hashes versus physical integrity, atomic manifests and migration. Retain existing JSON contracts until approved spec/schema migration; remove redundant JSON traversal meanwhile where justified. |
| PERF-E04 | mizar-test | Large-scale performance and AI feedback evaluation | A06; C05 for corresponding measurements | Extend early baselines to large libraries and sustained workloads; track RSS, allocation, replay, calls and optional agent cost (time/token/review overhead); no requirement to defer smaller benchmarks until this package. |
| PERF-E05 | mizar-artifact | Compact payloads and selective compression | B07, E03/B08 representation, A07 comparison | P0 structural compaction: string/symbol/namespace tables, numeric tags/hash bytes, source-range and dependency encoding, typed DAG references and shared interface payloads; P1 block compression of cold witnesses/traces/debug data where beneficial. Keep hot indexes directly accessible; measure cross-module/package sharing, file-count overhead and decompression costs; preserve evidence and source provenance. |

## Priority and sequencing exceptions

- PERF-A01 establishes the measurement method; A01's measurement ranking selects the next package; A02 is a draft candidate, not a fixed successor. A06 begins as soon as its measurement method is ready, before the optimization it measures; it does not globally block A02 or genuine feature work. C05 starts with available native routes and follows C01/C03/C04 incrementally. E04 is only the later scaling/agent-cost extension.
- PERF-D03 (original PERF-13/14, highest priority): design during Phase B and integrate with existing Step 7 as genuine inputs become ready; do not defer the search architecture until feature completion. Preserve the backend-free finder and treat backend data as advisory. Profile proof success as well as time.
- PERF-D01/D02/D04/D06/D07: review structural index, rewrite/instantiation, evidence and SAT designs before broadening their corresponding producers (D06 before general Template proofs; D07 before generalized Step 7 acceptance design). Design readiness does not depend on the later implementation prerequisites in the table. Implementation remains evidence- and producer-gated; D07 is a design/benchmark decision, not authorization to replace the current Kernel.
- Kernel Task 32 (original PERF-20) is pending, trigger-based work, neither active nor owner-parked. Re-entry requires a pinned `batsat` version change under the task-24 audit procedure. This inventory does not activate it or add a Step 7 gate.
- C03/C04: freeze the shared dependency and reuse contract first, then implement owner-specific slices (C04 dependency inputs before dependent C03 reuse, C03 consumers after those inputs). No cyclic implementation dependency. C05 can exercise each slice independently.
- Original priorities below are preserved as audit priorities, not blanket dispatch gates. High/P0 structural and integration prerequisites should be addressed before dependent generalization; P1/P2/P3 residuals are measurement-triggered and may move earlier. Investigations marked “design before implementation” must be considered before the relevant implementation, even when their package is in D. No new global feature-completion gate.

## Persistent knowledge storage and retrieval

Owner direction (2026-11-28): binary publication, size reduction and retrieval are one architecture workstream, not unrelated late optimizations. B07 establishes the design after the current VC53-E1/A01 boundary and development-efficiency prioritization; affected persistent representations must not be frozen independently before B07 review. No global stop for independent work. A07 establishes comparisons early and accompanies incremental implementation. The existing A01 ranking remains dispatch authority; implementation contracts are separately frozen.

- Research: inspect LEO-II/Leo-III (perfect sharing, occurrence/road-sign indexing, binders/explicit substitutions), E (Term Bank, discrimination/fingerprint/feature-vector indexes), Vampire (substitution/code trees), Lean (lazy discrimination indexes and compacted regions), Isabelle (nets), Agda (shared serialization), and Zipperposition where relevant. Record actual versions, source paths, licensing and applicability; no assumption that any algorithm wins or is directly transferable. Starting references: Theiß/Benzmüller, [Term Indexing for the LEO-II Prover](https://ceur-ws.org/Vol-212/03_Theiss.pdf); Schulz, [Fingerprint Indexing for Paramodulation and Rewriting](https://wwwlehre.dhbw-stuttgart.de/~sschulz/PAPERS/schulz_fp-index.pdf).
- Semantic representation: canonical typed Type/Term/Formula DAGs with binder-safe identity, free-variable/environment and template dependencies, collision-safe equality, separate occurrence/source/evidence IDs; exact sharing does not imply logical equivalence. Compare module-local immutable tables plus session sharing first, then package/global sharing without sacrificing independent compilation or stable incremental identities. Measure reverse occurrences and memoized structural features rather than storing large summaries indiscriminately.
- Retrieval: define query direction and completeness for exact, variant, generalization, instance, unification and subterm relations; distinguish complete candidate filters (no false negatives for supported relation) from heuristic premise ranking. Verify candidates with owner matching, guards, visibility, types, binding/occurs checks and accepted context; deterministic result ordering; never use index membership as proof. Compare exhaustive reference, query/update cost and total index bytes; no mandate to implement all trees.
- Physical storage: binary is the target, not a frozen codec. Compare ordinary binary versus compact ID/offset tables and mmap-capable sections; hot interface/index data separate from cold witnesses/traces/diagnostics. Preserve validated manifest reachability, content and semantic identity, portability, schema/version rejection, deterministic output, publication atomicity and independent Kernel acceptance. JSON export/compatibility remains available; no raw Rust heap dump, unsafe unchecked casts, or unvalidated external payloads. Section authentication must not silently weaken whole-publication checks; any trust/format migration needs separate approved contracts and EN/JA specification updates.
- Query architecture: static immutable library segments plus mutable local overlay; origin-scoped filtering without cloning the whole index; exact/prefix/range lookups, inverted symbol/type indexes, structural retrieval, reverse occurrences and forward/reverse dependencies. Reuse canonical semantic nodes across consumer queries. Optional full-text/vector search is advisory, not proof authority. Indexes are derived, versioned, rebuildable and bound to exact accepted publication; stale/malformed indexes fall back or fail closed according to owner contract.

| Existing work | Boundary with new work |
|---|---|
| B01/B02 | Existing replay elimination and immutable residency remain owners; adopt B07 representation/loader interface, not a second store. |
| B03/B04/B05/B06 | Existing resolver/type/core owners implement local indexing and sharing; B08 coordinates retrieval primitives and structural identity, no duplicate arenas. |
| C02/C03/C04 | Existing lifecycle, reuse and dependency owners; C06 implements persistent query indexes with those contracts, not new proof reuse authority. |
| D01/D02 | Checker owns cluster/overload/rewrite/template semantics; B08/C06 supply candidate retrieval, owner verifies applicability. |
| D03/D05 and ATPK-PREMISE-RANKING | ATP owns ground search and ranking; use B08/C06 retrieval where measured, keep existing Step 7 producer contracts. |
| E03/E05 | E03 owns format/loader/migration; E05 owns compaction and cold payload compression. E03 may ship minimal binary before optional compression; agree schema evolution before implementation. |
| A06/C05/E04 | A07 owns storage/retrieval comparisons; reuse existing benchmark harness and extend actual end-to-end/parallel workloads, not a duplicate harness. |

### Additional audit traceability

These new planning findings supplement (do not renumber) the 79 original audit findings. Earlier provisional STORE/INDEX/RET IDs are aliases for the following packages, not separate dispatch queues.

| Scope | Work packages |
|---|---|
| STORE-01/04 (binary layout, mapped/partial reads) | B07, E03 |
| STORE-02/03 (interning, persistent DAG) | B04, B06, B08, E05 |
| STORE-05 (cold payload compression) | E05 |
| STORE-BENCH / RET-10 (measurement) | A07 |
| INDEX-01/03 (names, types, visibility, dependencies) | B03, C04, C06 |
| INDEX-02 (structural retrieval) | B08, C06, D02 |
| INDEX-04/05 (premise and advisory text search) | C06, D05 |
| RET-01/03/07/08 (research, relations, static/dynamic, visibility) | B07, B08, C06 |
| RET-02/04/05/06 (bank and alternative indexes) | B06, B08, A07 |
| RET-09 (owner integration) | C06, D01, D02, D03, D05 |

## Declarative inference performance

Owner direction (2026-11-28): high verification throughput is a project requirement, without shifting routine search-control work to users. Keep declarative proof and automatic inference as the default; allow optional manual strategy selection and reusable proof algorithms through the existing Algorithm language, not a separate tactic language. A08 precedes architectural selection; D08–D12 are conditional design/evaluation work, not permission to change the grammar or trust model. Preserve the VC53-E1/A01 dispatch boundary and independent development-efficiency work.

- Baseline the actual pipeline, not a presumed external call for every statement: inventory VC pre-ATP discharge, backend-free finder, Proof routing, Kernel and genuine backend paths. Attribute source statement to all its VCs; count non-ATP and unsupported routes separately. Measure preparation/premise retrieval, encoding, process/IPC, initialization, solve, failed attempts, extraction/reconstruction, kernel checking and replay; cold/warm/edit and serial/parallel runs, p50/p95/p99 and accepted throughput, wall/CPU/RSS. Count backend reports separately from independently accepted proofs and keep timeouts/unknown/open/rejected distinct.
- Workloads: existing genuine Mizar-evo theorem/algorithm routes first; stratified real statement corpus (logic, equality, rewriting, typed/quantified facts, arithmetic and difficult cases), including invalid/unsupported controls. Investigate MML extraction of original local assumptions, cited premises, implicit type/registration facts and scope; record fidelity/unsupported cases, not a claim of Mizar-evo source compatibility. Compare matched allowed premise sets, budgets, seeds/versions and hardware; report original proof acceptance, success overlap and marginal coverage, latency including failed tiers. Synthetic cases supplement, never replace native measurements. No fixed success threshold before baseline.
- Compare existing isolated ATP/SMT, resident workers, supported direct APIs and lightweight internal inference on the same obligations. Review Metis/Isabelle, Lean grind and classic Mizar inference as references; distinguish search from evidence reconstruction. Prefer cheap exact lookup, congruence, indexed rewrite, propositional/Horn/E-matching candidates and measured bounded resolution before a full general-purpose internal ATP. Existing D01/D02 and B08/C06 supply retrieval and guards; no duplicated term banks or unchecked equality.
- D08 specifies automatic and explicit modes (internal, ATP, SMT, Algorithm as candidates), manual strict selection versus explicitly requested fallback, configuration scope/precedence, explainable routing, budget/cancellation and deterministic evidence selection as required by existing contracts. Selection changes search, not admissible premises or acceptance; unknown/timeout never proves false. A fixed sequential cascade is not mandated: measure wasted pre-processing, difficult-goal regressions, routing and parallel alternatives. No exact syntax is adopted here; existing by clauses remain valid.
- D10 separates reusable declarations/translation from search state; test changed goals, local assumptions, import/context versions, push/pop/reset, cancellation/crash and cross-query isolation. No solver-state reuse without an explicit valid context contract. SMT-05 retains incremental SMT ownership; D05 retains process/portfolio scheduling.
- D11 treats Algorithm as the programmable automation mechanism. Inventory existing reflected syntax, MVM/termination and computation admission; compare proof-evidence-producing algorithms and verified reflection, with scoped read-only goal/premise inputs, output validation, failure/resource behavior and reproducibility. Any new proof-facing syntax/API is a separately reviewed minimal generic extension; no dedicated tactic language or unverified metaprogram privilege.
- D12 uses existing C03/C04 dependency identity, Proof acceptance, Kernel checking and E03/E05 storage: exact-context evidence replay can avoid search, but reuse must not conflate formula equality with proof validity or bypass an unfulfilled checking obligation. Benchmark first discovery separately from recheck. Preserve current policy, witnesses and cache invalidation.

| Provisional finding | Work packages |
|---|---|
| INFER-01/02 (statement profile and cost separation) | A08 |
| INFER-03/04 (Metis-like/local inference comparison) | D09, A08, B08, D01/D02/D04 |
| INFER-05/06 (resident/embedded backends, context reuse) | D10, D05, SMT-05, C04 |
| INFER-07 (automatic/manual dispatcher) | D08 |
| INFER-08 (evidence storage/replay) | D12, C03/C04, D07, E03/E05 |
| INFER-09/10 (guided justification, Algorithm automation) | D08, D11 |

ATPK-BENCHMARK retains its distinct theorem re-proving and premise-selection conditions. A08 measures the actual fine-grained checking workload; share harnesses and source slices where possible, do not wait for complete MML migration or all new engines. The existing backend-free finder, guarded backend extraction, theory mappings and evidence checkers retain their owner contracts. D08 depends on initial A08 measurements, not completion of hypothetical D09–D12 implementations; A08 is extended as candidate implementations become available.

## Source audit anchors

- A: `crates/mizar-test/src/runner.rs`, `runner/type_elaboration/admission.rs`, `tests/metadata.rs`, `tests/lint_policy.rs`; Lexer/Frontend Criterion targets; AGENTS.md identifies the library-test compilation bottleneck.
- B: `mizar-driver/src/frontend_adapter/{checking,core,vc,candidate}.rs` replay codecs; `mizar-ir/src/storage.rs` in-memory blobs/repeated decode; `mizar-resolve/src/{env,names}.rs`; `mizar-checker/src/{type_checker,binding_env,source_context,source_overload}.rs`; `mizar-core/src/elaborator.rs`.
- C: `mizar-build/src/scheduler.rs` synchronous advance and repeated ready scans; `mizar-driver/src/driver.rs` Source/Frontend cache; `mizar-cache/src/{cache_store,proof_reuse,dependency_fingerprint}.rs`; `mizar-vc/src/dependency_slice.rs`. Reuse validator/trigger integration needs current-callsite verification; code-search misses alone are not proof of absence.
- D: checker registration/overload/template paths, ATP instance finder, Kernel formula/SAT processing. Reconfirm earlier audit findings; advisory substitution coverage, equality density and reduction sharing are open measurements.
- Persistent storage/retrieval audit (main `22d63fd0673b9b00e589ef4c363f67ca350277ec`): `mizar-artifact/src/{store,module_summary,verified_artifact}.rs` canonical JSON and repeated rendered metadata; `mizar-cache/src/cluster_db.rs` import-scoped filter/clone/sort; `mizar-lsp/src/completion.rs` repeated summary/shape lookup; `mizar-doc/src/doc_build.rs` full artifact read for index construction. Reconfirm actual routes before edits; these are static findings, not measured bottlenecks.
- E: `mizar-lexer/src/lexical_environment.rs`, `mizar-frontend/src/lexing.rs`, `mizar-parser/src/module.rs`, `mizar-artifact/src/store.rs`. Existing benchmark edits rerun the full frontend, not actual incremental reuse.

- Inference baseline anchors: `doc/design/mizar-vc/en/discharge.md` (pre-ATP discharge), `mizar-atp/en/{backend,portfolio,instance_finder}.md` under design, architecture 07/10, existing ATPK-BENCHMARK and SMT-05 scopes; re-audit real production call sites and distinguish specification from implementation before measurement.

## Benchmark and exit policy

A01 measures the development/runtime baseline; A06 supplies component and genuine end-to-end baselines; C05 measures actual incremental/parallel execution; E04 expands scale. Start with small supported source profiles; add geometric workloads for indexes, shared types, clusters, templates, ground instances and scheduler DAGs. Grow real source-to-artifact workloads only with genuine producers. Measure wall/CPU time, peak RSS, allocation/clone counts, replay, ATP/kernel/SAT calls, cache reasons, I/O and actual task overlap. Measure accepted/rejected inputs; changes to trust boundaries need separate reviewed contracts. Benchmark success grants no semantic coverage.

## Audit traceability

All 80 original audit rows are retained as 79 unique findings; the two PERF-06 rows are merged because sharing scope and use-site-dependent keys concern the same instantiation identity. PERF-07 signature sharing and PERF-12 proof evidence remain distinct. IDs in the first column are historical audit IDs, not dispatchable packages; cross-package references indicate shared responsibility. Original priorities are retained even where implementation is producer-gated.

| Original ID | Finding | Original priority | Work packages |
|---|---|---|---|
| PERF-01 | Registration trigger/ID lookup | high | PERF-D01, PERF-B03 |
| PERF-02 | Repeated full-rule cluster closure | high | PERF-D01 |
| PERF-03 | Type fact lookup | high | PERF-D01 |
| PERF-04 | All-pairs overload comparison | high | PERF-D01 |
| PERF-05 | Import-scoped view construction | investigate | PERF-D01, PERF-C04 |
| PERF-06 | Instantiation key/use-site dependence and sharing scope (merged two rows) | high + investigate | PERF-D02 |
| PERF-07 | Instantiated signature sharing | high | PERF-D02 |
| PERF-08 | CoreTerm structural deduplication | high | PERF-B06 |
| PERF-09 | CoreFormula structural deduplication | high | PERF-B06 |
| PERF-10 | Elaborator table cloning/re-indexing | medium–high | PERF-B06, PERF-B05 |
| PERF-11 | Reduction implementation strategy | design before implementation | PERF-D02 |
| PERF-12 | Template proof evidence across instantiations | design before implementation | PERF-D06 |
| PERF-13 | Ground instance Cartesian search and repeated SAT | highest | PERF-D03 |
| PERF-14 | Vampire substitution/inference information | highest | PERF-D03 |
| PERF-15 | Combinatorial equality axioms | high | PERF-D03, PERF-D04 |
| PERF-16 | Kernel SAT re-search | high / design change | PERF-D07, PERF-D04 |
| PERF-17 | Tseitin duplicate subformulas | high | PERF-D04 |
| PERF-18 | Formula cloning/reconstruction/reserialization | medium–high | PERF-D04, PERF-B06, PERF-E03 |
| PERF-19 | Sequential ATP portfolio | medium–high | PERF-D05 |
| PERF-20 | Kernel SAT search step budget | high | PERF-D04, PERF-D07, Kernel 32 |
| INC-01 | Large-scale incremental verification benchmark | P0 | PERF-C05, PERF-E04 |
| INC-02 | Proof reuse Driver integration | P0 | PERF-C03 |
| INC-03 | Verified module reuse | P0 | PERF-C03 |
| INC-04 | RebuildTrigger execution integration | P0 | PERF-C03, PERF-C04 |
| INC-05 | Local module dependencies | P1 | PERF-C04 |
| INC-06 | Session artifact sharing | P1 | PERF-C04 |
| INC-07 | Shared VC fingerprint memoization | P1 | PERF-C04 |
| INC-08 | VC dependency lookup index | P1 | PERF-C04 |
| INC-09 | Artifact serialization/hash deduplication | P2 | PERF-E03 |
| INC-10 | Evaluate binary internal cache | P3 | PERF-E03 |
| IR-01 | Eliminate replay on IR save | P0 | PERF-B01 |
| IR-02 | Direct IR serialization/rehydration | P0 | PERF-B01 |
| IR-03 | Share decoded blobs | P0 | PERF-B02 |
| IR-04 | Active IR residency | P1 | PERF-B02 |
| IR-05 | Last-use release integration | P1 | PERF-C02 |
| IR-06 | Blob refcounts/snapshot index | P1 | PERF-C02 |
| IR-07 | Budget-based disk spill | P2 | PERF-C02 |
| IR-08 | Term/formula hash-consing evaluation | P1 | PERF-B06 |
| IR-09 | IR clone/allocation measurement | P1 | PERF-A06, PERF-E04 |
| PAR-01 | Actual worker pool | P0 | PERF-C01 |
| PAR-02 | Remove batch barrier | P1 | PERF-C01 |
| PAR-03 | Event-driven ready queue | P1 | PERF-C01 |
| PAR-04 | Shorter Publisher/Storage lock scope | P1 | PERF-C02 |
| PAR-05 | Module/VC parallel benchmark | P0 | PERF-C05, PERF-C01 |
| PAR-06 | ATP load balancing/resource integration | P1 | PERF-D05, PERF-C01 |
| LEX-01 | Local symbol trie/position index | P1 | PERF-E01 |
| LEX-02 | Token string/source range sharing | P2 | PERF-E01 |
| LEX-03 | Remove per-token stream construction/clone | P1 | PERF-E01 |
| LEX-04 | Evaluate repeated disambiguation/scope construction | P2 | PERF-E01 |
| PARSER-01 | Speculation checkpoint instead of clone | P1 | PERF-E02 |
| PARSER-02 | Delimiter/boundary lookup index | P2 | PERF-E02 |
| RES-01 | Real SymbolIndex | P0 | PERF-B03 |
| RES-02 | Other Resolver indexes | P0 | PERF-B03 |
| RES-03 | Name/scope binding indexes | P0 | PERF-B03, PERF-B05 |
| RES-04 | Avoid repeated AST-wide variable scans | P1 | PERF-B03 |
| TYPE-01 | DAG-preserving canonicalization | P0 | PERF-B04 |
| TYPE-02 | Persistent binding context | P1 | PERF-B05 |
| TYPE-03 | Shared Typed AST/side tables | P0 | PERF-B05 |
| OVER-01 | Site-indexed overload candidates | P1 | PERF-D01 |
| OVER-02 | Specificity relation reuse | P1 | PERF-D01 |
| ELAB-01 | Typed AST/term/formula reverse index | P1 | PERF-B06 |
| ELAB-02 | Reduce repeated validation/cloning | P0 | PERF-B05, PERF-B06, PERF-B01 |
| TEST-01 | Cargo compile/link/test timing | P0 | PERF-A01 |
| TEST-02 | Split library test target | P0 | PERF-A02 |
| TEST-03 | Generic data-driven runner | P0 | PERF-A03 |
| TEST-04 | Retire per-case admission rules | P0 | PERF-A03 |
| TEST-05 | Shared fixtures/mutation engine | P1 | PERF-A04 |
| TEST-06 | Reuse test plans | P1 | PERF-A04 |
| TEST-07 | Parallel corpus execution | P1 | PERF-A04 |
| TEST-08 | Determinism/mutation test cadence | P1 | PERF-A05 |
| TEST-09 | Separate/accelerate lint and document checks | P1 | PERF-A04 |
| TEST-10 | Cargo profiles/fast linker | P1 | PERF-A05 |
| BENCH-01 | Per-phase measurement | P0 | PERF-A01, PERF-A06 |
| BENCH-02 | Component scaling benchmarks | P0 | PERF-A06 |
| BENCH-03 | End-to-end benchmark | P0 | PERF-A06 |
| BENCH-04 | Incremental verification benchmark | P0 | PERF-C05 |
| BENCH-05 | Parallel verification benchmark | P1 | PERF-C05 |
| BENCH-06 | Memory/structural sharing benchmarks | P1 | PERF-A06, PERF-B06, PERF-E04 |
| DEV-01 | Quantify AI development efficiency | P2 | PERF-E04 |

## Open questions

Actual local compilation/link/test durations, in-flight work, representative workloads and priorities after measurement. Untrusted persistent IR and proof reuse authority require separate review from in-process immutable sharing. Numeric memory budgets and backend scaling targets remain unset. B07 must resolve candidate file encoding, sharing scope, retrieval relation/API, static index layout, partial integrity verification and migration strategy; A07 determines which alternatives merit implementation.
