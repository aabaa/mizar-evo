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
| PERF-B01 | mizar-driver | Replace routine source replay in IR storage | A01 | Inventory genuine theorem/algorithm paths; owner structural codecs or process-local sealed payloads; migrate by stage; count producer calls on normal encode/get. |
| PERF-B02 | mizar-ir | Share resident and decoded immutable IR | A01, B01 interface design | Avoid repeated hash/decode of immutable in-process bytes; exact schema/type/context/decoder identity; single-flight as safe; bounded active residency. |
| PERF-B03 | mizar-resolve | Real indexes and scope lookup | A01 | Replace insertion sorts and linear Symbol/Label/Definition/Registration/etc queries; index bindings and omitted witnesses; preserve duplicate, shadowing and recovery rules. |
| PERF-B04 | mizar-checker | DAG-preserving type canonicalization | A01 | Replace recursively owned StructuralTypeKey expansion with canonical child identities, deterministic order and collision-safe equality; preserve source/status merge; shared-subtype scaling. |
| PERF-B05 | mizar-checker | Share typed arena, contexts and handoffs | A01; B04 as applicable | Persistent context deltas, indexed BindingEnv, immutable side tables; avoid clone/reinstall and upstream recomputation without weakening owner checks. |
| PERF-B06 | mizar-core | Shared IR and lowering indexes | A01; B05 as applicable | Index TypedNodeId/TermId/FormulaId/references; preserve seed memoization; measure and reduce existing table clones, re-indexing and repeated lowering validation; evaluate Term/Formula hash-consing with semantic context and separate provenance. Measure Algorithm ProgramContext clones, branch-join precision and VC size growth; coordinate VC owner, preserve old-state and path semantics. |
| PERF-C01 | mizar-build | Actual worker execution and event-driven DAG | A01, B02, Step 6 | Bounded concurrent workers, ready queue, no artificial batch barrier; preserve dependency/resource/cancellation and canonical commits; prove actual overlap and measure module/VC scaling with C05. |
| PERF-C02 | mizar-ir | Last-use collection and memory budgets | B02, C01 lifecycle design | Scheduler-aware retain/release, snapshot indexes, blob refcounts; shorten Publisher/Storage lock-held hashing and collection; bounded resident/encoded memory, disk spill as needed; protect LSP/commit owners. |
| PERF-C03 | mizar-driver | Module and VC incremental reuse | A01, Step 6/7 accepted producers; C04 shared design | Module then declaration/VC reuse; integrate ProofReuseValidator and RebuildTriggerEvaluator where current-callsite review confirms gaps; unchanged inputs avoid ATP; kernel skip only with reviewed trusted acceptance contract, otherwise replay witness. Distinguish contract, definition and execution dependencies; reuse callers only when contracts and accepted guarantees remain valid, and measure local edits. |
| PERF-C04 | mizar-cache | Precise dependency and session reuse | A01, B03; shared C03/C04 design | Interface vs implementation and negative lookup/registration dependencies; integrate rebuild decisions, narrow frontend keys, reverse edges; verify/read immutable artifacts once per snapshot; VC slice index/shared fingerprints with VC owner; reuse import-scoped views with D01. |
| PERF-C05 | mizar-test | Incremental and parallel verification benchmarks | A06; corresponding C01/C03/C04 producer availability, not all packages complete | P0: establish cold/warm/edit and worker-count baselines as each real route becomes available; quantify module/VC reuse, ATP/Kernel work avoided, dependency invalidation, actual concurrency, RSS and equivalence. Grow workload geometrically; do not defer to E04. |
| PERF-D01 | mizar-checker | Cluster and overload search | A01, B03, B04 for implementation; early design independent of them | Index trigger/registration ID and type facts; replace repeated full-rule scans with worklist saturation; measure import-scoped view construction, filtering, cloning, reuse and invalidation with C04; site grouping, viability filters and context-safe specificity reuse; preserve traces, ambiguity and termination. Evaluate state-versioned type facts, safe cross-VC context reuse and reduction in solver obligations; preserve visibility and proof evidence. |
| PERF-D02 | mizar-checker | Template and reduction reuse | A01, B04, B06 for implementation; genuine producers; early design before generalization | Assess sharing scope; separate use-site identity from semantic instantiation keys (template, actuals, relevant environment); share instantiated signatures across sites/modules only under equivalent dependencies and keep occurrence provenance separate. Design rule indexes, rewrite strategy, normalization and termination before extending Reduction; measure safe memoization, no eager global expansion. Measure evidence-backed symbolic reduction on array/word workloads when genuine producers exist; respect the specified decreasing order and guard evidence. |
| PERF-D03 | mizar-atp | Evidence-driven ground instance search | early design now; A01 and genuine Step 7 producers for implementation | P0 design/integration: inventory actual Vampire substitution/inference extraction, exploit validated advisory information where available; avoid blind Cartesian enumeration, repeated SAT and redundant equality instances; consider incremental untrusted SAT screening; deterministic bounded fallback; independent Kernel validation. |
| PERF-D04 | mizar-kernel | Formula and SAT processing | A01; D03/D07 interfaces as applicable | Early design for equality generation and formula/Tseitin sharing; measure equality density, repeated formula nodes and SAT cost, reduce formula cloning/reconstruction/reserialization across ATP/Kernel boundaries without sharing trust authority. Kernel 32 remains pending/trigger-based until a pinned `batsat` version change; certificate redesign belongs to D07. |
| PERF-D05 | mizar-atp | Backend and portfolio scheduling | C01, Step 7 | Bounded concurrent processes, load balancing and shared resource budgets; safe preparation reuse, deterministic selection; measure success rate and time, retain ranking and re-proving tasks. Measure premise ranking scans; evaluate symbol-set/frequency precomputation and inverted indexes scoped to the actual candidate pool; measure repeated process/version-probe overhead; coordinate persistent sessions with SMT-05. |
| PERF-D06 | mizar-proof | Template proof evidence reuse | early design before general Template proof mechanism; genuine template/Core/VC/evidence producers for implementation | Evaluate reuse of parametric proof evidence versus identical-instantiation cached evidence, binding/actual constraints, dependency and policy compatibility, witness identity and occurrence provenance. Revalidate substituted evidence at the independent Kernel; no automatic transfer of acceptance between distinct instantiations. |
| PERF-D07 | mizar-kernel | SAT search and certificate checking separation | early design before generalized Step 7; A01/A06 and D03/D04 inputs for evaluation | Compare current trusted SAT re-search against untrusted solver-generated UNSAT evidence checked independently (e.g. suitable certificate formats); measure generation/transfer/checking cost, memory and failure modes. Review formula/CNF binding, trusted base, malformed-proof rejection, determinism and compatibility; retain current sound path until separately approved and validated. |
| PERF-E01 | mizar-lexer | Lexical lookup and frontend token allocation | A01 | Index local declarations with activation positions; source spans where safe; batch contextual disambiguation; preserve multi-pass semantics unless equivalence shown. |
| PERF-E02 | mizar-parser | Parser checkpoints and lookup | A01 | Avoid cloning claimed-child/trivia state for speculation; delimiter indexes preserving recovery; measure nested formulas. |
| PERF-E03 | mizar-artifact | Artifact serialization and internal storage formats | A01, B01 | Remove redundant JSON traversal/hash buffers with identical public bytes; binary internal cache only if measured useful; preserve integrity/version/publication. |
| PERF-E04 | mizar-test | Large-scale performance and AI feedback evaluation | A06; C05 for corresponding measurements | Extend early baselines to large libraries and sustained workloads; track RSS, allocation, replay, calls and optional agent cost (time/token/review overhead); no requirement to defer smaller benchmarks until this package. |

## Priority and sequencing exceptions

- PERF-A01 establishes the measurement method; A01's measurement ranking selects the next package; A02 is a draft candidate, not a fixed successor. A06 begins as soon as its measurement method is ready, before the optimization it measures; it does not globally block A02 or genuine feature work. C05 starts with available native routes and follows C01/C03/C04 incrementally. E04 is only the later scaling/agent-cost extension.
- PERF-D03 (original PERF-13/14, highest priority): design during Phase B and integrate with existing Step 7 as genuine inputs become ready; do not defer the search architecture until feature completion. Preserve the backend-free finder and treat backend data as advisory. Profile proof success as well as time.
- PERF-D01/D02/D04/D06/D07: review structural index, rewrite/instantiation, evidence and SAT designs before broadening their corresponding producers (D06 before general Template proofs; D07 before generalized Step 7 acceptance design). Design readiness does not depend on the later implementation prerequisites in the table. Implementation remains evidence- and producer-gated; D07 is a design/benchmark decision, not authorization to replace the current Kernel.
- Kernel Task 32 (original PERF-20) is pending, trigger-based work, neither active nor owner-parked. Re-entry requires a pinned `batsat` version change under the task-24 audit procedure. This inventory does not activate it or add a Step 7 gate.
- C03/C04: freeze the shared dependency and reuse contract first, then implement owner-specific slices (C04 dependency inputs before dependent C03 reuse, C03 consumers after those inputs). No cyclic implementation dependency. C05 can exercise each slice independently.
- Original priorities below are preserved as audit priorities, not blanket dispatch gates. High/P0 structural and integration prerequisites should be addressed before dependent generalization; P1/P2/P3 residuals are measurement-triggered and may move earlier. Investigations marked “design before implementation” must be considered before the relevant implementation, even when their package is in D. No new global feature-completion gate.

## Source audit anchors

- A: `crates/mizar-test/src/runner.rs`, `runner/type_elaboration/admission.rs`, `tests/metadata.rs`, `tests/lint_policy.rs`; Lexer/Frontend Criterion targets; AGENTS.md identifies the library-test compilation bottleneck.
- B: `mizar-driver/src/frontend_adapter/{checking,core,vc,candidate}.rs` replay codecs; `mizar-ir/src/storage.rs` in-memory blobs/repeated decode; `mizar-resolve/src/{env,names}.rs`; `mizar-checker/src/{type_checker,binding_env,source_context,source_overload}.rs`; `mizar-core/src/elaborator.rs`.
- C: `mizar-build/src/scheduler.rs` synchronous advance and repeated ready scans; `mizar-driver/src/driver.rs` Source/Frontend cache; `mizar-cache/src/{cache_store,proof_reuse,dependency_fingerprint}.rs`; `mizar-vc/src/dependency_slice.rs`. Reuse validator/trigger integration needs current-callsite verification; code-search misses alone are not proof of absence.
- D: checker registration/overload/template paths, ATP instance finder, Kernel formula/SAT processing. Reconfirm earlier audit findings; advisory substitution coverage, equality density and reduction sharing are open measurements.
- E: `mizar-lexer/src/lexical_environment.rs`, `mizar-frontend/src/lexing.rs`, `mizar-parser/src/module.rs`, `mizar-artifact/src/store.rs`. Existing benchmark edits rerun the full frontend, not actual incremental reuse.

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

Actual local compilation/link/test durations, in-flight work, representative workloads and priorities after measurement. Untrusted persistent IR and proof reuse authority require separate review from in-process immutable sharing. Numeric memory budgets and backend scaling targets remain unset.
