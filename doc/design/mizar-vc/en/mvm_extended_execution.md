# MVM R5 — Structured Execution, Failure and Budgets

> Canonical English; [Japanese companion](../ja/mvm_extended_execution.md).
> Design task: `MVM-R5-EXTENDED-EXECUTION-DESIGN`. Not executable.
> Existing [R4 executor design](./mvm_executor.md) and
> `ALG-MVM-EVALUATION`
> retain the evaluator owner; R6/R7 retain proof admission/publication.

## Semantic contract and stage split

[§20.1.3–4](../../../spec/en/20.algorithm_and_verification.md#2013-variable-declarations-var-const)
distinguishes mutable `var`, immutable `const`, and erased `ghost`;
[§20.2](../../../spec/en/20.algorithm_and_verification.md#202-control-flow)
defines flow; [§20.3](../../../spec/en/20.algorithm_and_verification.md#203-non-determinism-pick)
defines runtime Pick; [§20.9](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability)
governs computability, trust and resources. A mathematical algorithm may be
valid but unexecutable; executable failure is not a failed proof.

| R5 design slice | Source semantics to preserve | Required producer / positive and negative tests |
|---|---|---|
| **A. Local state and branches** | Local `var` assignment updates the current value; `const` is immutable. Evaluate computable `if` conditions, follow selected branch and its return/continuation. `assert` and `ensures` are verification conditions, **not VM-derived proof facts**. | Real Checker/Core 42–43, 46, 48, VC and R4; existing [`swapalgo`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz) is an Open VC baseline. Reject wrong target/type, uninitialized value, ghost-to-runtime dependence, swapped branch/return and unsupported condition. |
| **B. While/range/sequence control** | `while` checks its condition on successive iterations; `break` and `continue` use the enclosing loop. For range `to`/`downto`, evaluate bounds/step once, default step 1 and require positive `Nat`. Finite-sequence `for ... in` follows positions and retains duplicates. Invariants/decreasing are VC obligations, not dynamic proof shortcuts. | Core 43–44/48–53, actual CFG/exit semantics, VC and R4. The [`loopalgo` corpus](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz) is VC-only. Test zero/multiple iterations, positive/invalid steps, directions, early exits, continue, stale CFG and budget exhaustion. |
| **C. Calls and recursion** | Evaluate supported arguments by value, check the resolved call/visibility, use the exact verified body for every same-package/private-helper or validated public cross-package callee, and return through the proper frame. Partial callees may fail; `requires` and normal-return obligations are justified independently. | Resolver call edges, Core46/52, authentic R2 association, R3 closure/loader, VC/Proof gates, R4 frame semantics. Reject mismatched/absent call, stale dependency, private external call, unavailable executable, source fallback, wrong frame/argument and nested-depth exhaustion. |
| **D. Set/Pick and reflected matching** | A finite sequence is ordered; finite-set `for ... in` has unspecified order, with `processed V` ghost-only. A runtime Pick selects a concrete witness where computable; ghost-only Pick is erased but its nonemptiness proof remains. `match` compares syntax structure, tries cases in order, and uses `otherwise` or implicit fail for a partial unmatched case; no semantic equality is substituted. | Core44–45,49–50, real set enumeration and typed/accepted AST formation. Before *proof discharge*, R6 admission must check full-closure uniqueness/exemption for each applicable value-returning callee with non-ghost Pick or unordered set iteration. Reject invented enumeration, unproved uniqueness, missing `otherwise` in `terminating` algorithms (E0503), opaque decomposition, capture mismatch, unsupported runtime operation. |
| **E. Failure and ghost safety** | Reachable `fail` (including a partial unmatched `match`) ends the whole evaluation and cannot be caught. Unsupported noncomputable operators, zero division and unavailable Pick witnesses produce runtime failure, not a normal return. Ghost-only updates and annotations are erased without altering runtime control/data. | Core43/45/48/50/52/53, checked failure placement, type/effect and computability inputs, real R4 failure support. Reject `fail` inside terminating/ghost contexts at its source stage, ghost flow into returns/conditions/calls, fabricated value or E5. |
| **F. Budget and interruption** | `steps`, `timeout`, `nest` are execution-wide options; **0 disables that limit**, not the computation itself. Enforce real step/time/depth bounds over nested calls; cancellation or an exhausted budget gives no normal-return fact and never proves divergence. | R4 minimal accounting decision, R3 VM-versioned semantics, actual multi-step/recursive source. Test exact boundary, zero/default/unlimited, nested sharing, timeout/cancel, budget propagation and repeatable successful values. Do not reuse the [Pre-ATP discharge Limit Model](./discharge.md#limit-model) as VM gas. |

This is a *design partition*, not six ready implementation contracts.
R5-A through F may be frozen as separate existing-owner implementation
slices only after their exact source/CFG/loader oracles exist; no
synthetic accepted runtime result is sufficient.

## Runtime and proof boundary

R4/R5 may produce an internal report **only after actual finite normal
execution** of the complete authenticated input against the validated
executable and transitive dependencies, with VM/rule version and every
relevant original context/`C`. A failed/unsupported/limited execution
produces no E5 `returns_f(c,v)`. In a supported partial program a
successful concrete run does not establish a global E4 termination theorem;
a `terminating` promotion still requires its independent accepted E4.
`assert`/invariants and caller `requires` remain independently verified.

The R5 runtime's choice and set-enumeration *order* is not the logical
proof of deterministic results. [§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step)
requires a uniquely determined result for every applicable non-ghost
Pick or unordered set iteration throughout the evaluated closure.
`ALGDEF-COMPUTATION-ADMISSION`
owns closure generation and Proof acceptance. Only
`AST-COMPUTE` converts genuine
reports into candidates and applies replay/strict policy. The VM, its
failure messages and its output hashes never upgrade evidence.

Runtime failures and resource/incomplete stages must remain distinct.
The language specifies relevant errors in [§22.4.5](../../../spec/en/22.error_handling_and_diagnostics.md#2245-atp-timeout-and-computation-limit-handling);
this document does not allocate new diagnostics or resolve an
unspecified step-count / timeout precedence. Missing/forged/stale
artifacts fail **before** execution, not as mathematical falsehood.

## Readiness, test controls and Open Questions

- **R5-A** first candidate after R4: `swapalgo` local update/return, only
  once a genuine accepted body and validated artifact exist. Existing
  [sidecar](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.expect.toml)
  claims Open VC generation, not a runnable positive. Retain it unchanged.
- **R5-B–D**: author independent source-derived cases after their owning
  Core/CFG/Resolver/Checker and execution contracts are frozen; do not
  relabel the existing `loopalgo` [sidecar](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.expect.toml)
  as evaluated. Test both observable successful values and source/identity/
  version/proof/context tampering failures.
- **R5-E–F**: failure, ghost/noncomputation, exhausted resources and
  interrupted execution must never generate accepted witnesses, E5
  reports, `computed` or `kernel_verified` status. Negative tests also
  cover mixed success/failure in transitively evaluated callees and
  stronger release/replay policies without altering Proof.
- **Open R5-Q1:** freeze one exact VM instruction/step charging rule,
  timeout measurement/interrupt precedence and shared nested-call
  limits under R3/R4 versioned execution; no numeric charges are
  implied by the specification.
- **Open R5-Q2:** supported concrete witness/enumerator providers for
  Pick and set iteration, and what unavailable/noncanonical runtime
  choice traces may be retained internally without making an imported
  result order-sensitive or fabricating proof determinism.
- **Open R5-Q3:** per-feature source/CFG completeness, call-frame and
  match/Binder certification; a general R5 implementation must wait for
  real producer-specific accepted inputs, not merely a Core enum variant.

R5-A–F execution requires authentic R4 execution and the
[MVM-R3 owner design](../../mizar-artifact/en/mvm_runtime_export.md)
prerequisites for runtime loading. No new crate, grammar, code extraction,
public API or test-coverage credit follows from this document.
