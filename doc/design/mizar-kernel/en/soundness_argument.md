# Kernel Soundness Argument And Pre-Implementation Audit

> Canonical language: English. Japanese companion:
> [../ja/soundness_argument.md](../ja/soundness_argument.md).

## Purpose

This document is the soundness argument for the trusted `mizar-kernel`
acceptance boundary (pipeline phase 14) and the record of a pre-implementation
audit of the certificate format and checking semantics. It consolidates, in one
place:

- the complete enumeration of invariants the kernel checks before acceptance;
- the exact meaning of "replay": what the kernel recomputes and what it never
  trusts;
- the rejection taxonomy and the attack each category prevents;
- edge cases that substitution, alpha-conversion, and clause well-formedness
  checking could plausibly miss, with their current disposition;
- audit findings, with severity, including candidate paths by which search-side
  unsoundness could leak into acceptance.

It audits the corrected formula/substitution evidence path (architecture 15
"Post-Closeout Correction", tasks 23-29) as the normal acceptance contract, and
treats the legacy resolution-trace certificate as migration/audit inventory
only. It refines and cross-checks
[15.kernel_certificate_format.md](../../architecture/en/15.kernel_certificate_format.md),
[16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md),
[08.reasoning_boundary.md](../../architecture/en/08.reasoning_boundary.md),
[19.failure_semantics.md](../../architecture/en/19.failure_semantics.md), and
[20.test_strategy.md](../../architecture/en/20.test_strategy.md), together with
the module specs of this crate. It is an audit artifact: it introduces no new
acceptance behavior, and where it found gaps it either patched the architecture
documents (recorded below) or filed findings for follow-up tasks.

## Trust Model

The trusted computing base for proof acceptance is:

- `mizar-kernel` source, including its parser, checkers, SAT encoding, and the
  `sat_checker` wrapper;
- the audited `batsat = "=0.6.0"` dependency (with `bit-vec 0.5.1`), pinned and
  wrapped per [sat_dependency_audit.md](./sat_dependency_audit.md);
- `mizar-core` data shapes whose binder contract the kernel independently
  re-checks (the kernel consumes shapes, not validity claims);
- `mizar-session` infrastructure types.

Everything else is untrusted with respect to acceptance: ATP backends and their
logs, `mizar-atp` translation, `mizar-vc` generation and discharge evidence,
resolver/checker state, caches, artifacts, and the evidence producer that
assembled the `KernelEvidence` bytes. A caller-supplied field is never
acceptance material by itself; it is either (a) re-derived and compared, (b)
checked against the caller's immutable context, or (c) ignored/rejected.

One input is trusted by construction and must be produced by the phase-14
orchestrator, not by the evidence producer: the immutable check context
(`FormulaEvidenceContext`, expected target VC fingerprint, policy, limits).
The soundness of the whole scheme depends on the orchestrator populating that
context only from kernel-accepted dependency artifacts and the canonical VC
identity, never from the same channel that delivered the evidence bytes. See
finding F2.

## Checked Invariants

The kernel accepts evidence only when every invariant below holds. Identifiers
are used by the reject-first corpus under `tests/certificates/` and by the
findings section.

### E. Envelope and structural invariants (`formula_evidence`, task 25)

- E1. Domain separator is `MIZAR_KERNEL_EVIDENCE\0`; schema and encoding
  versions are supported. Unknown values reject as
  `unsupported_certificate_format`.
- E2. Sections appear in the fixed v1 order (symbol manifest, variable
  manifest, formula evidence, substitutions, provenance, final goal); each
  section is length-framed; byte ranges are consumed exactly; trailing bytes
  reject.
- E3. Ids within each section are unique and sorted; duplicate or unsorted ids
  reject. References resolve only to ids declared in the same evidence object
  (or, for imported facts, to caller context entries).
- E4. All list counts, byte lengths, term sizes, recursion depths, and node
  counts are checked against deterministic limits before allocation
  (`resource_exhaustion`).
- E5. The canonical hash input is the exact validated envelope bytes. There is
  no producer-supplied trusted hash field.

### B. Target, profile, and context binding invariants

- B1. The evidence `target_vc` equals the caller's expected target VC
  fingerprint (`context_mismatch` otherwise).
- B2. The evidence `kernel_profile` is supported by, and equal to, the profile
  the checker is configured for (`unsupported_certificate_format`).
- B3. The final goal's fingerprint is recomputed from the goal formula tree and
  must match; its provenance must bind the target VC and the goal fingerprint
  (`missing_provenance`).
- B4. The final goal polarity must match the check kind that the caller's
  immutable context requires for this VC. Proof acceptance of a proof
  obligation requires refutation polarity (goal asserted false); acceptance
  under any other polarity is a `context_mismatch` (architecture 15,
  "Goal Polarity Is Bound By The Target Obligation"; finding F1). Implemented
  by `mizar-kernel` task 30 in `check_kernel_evidence`; accepted consistency
  checks are carried as `ConsistencyCheck` and are non-selectable diagnostic
  evidence for downstream proof policy.
- B5. Missing caller context, or a context provenance fingerprint that does not
  match what the evidence claims to bind, rejects as `missing_provenance`.

### F. Formula invariants

- F1. Every formula parses in the supported grammar (`Atom`/`Not`/`And`/`Or`
  over normalized `clause::Atom` values); empty conjunctions/disjunctions,
  malformed terms, unknown symbols, and manifest-incompatible variables reject.
- F2. Atom identity is the canonical injective byte encoding; equal bytes if
  and only if equal normalized atoms. No display name, source path, or
  allocation order participates.
- F3. Each formula's tree fingerprint is recomputed by the kernel and must
  equal the recorded fingerprint (stable identity binding, not acceptance).
- F4. Symbol and variable manifests authorize structure validation only; they
  never trigger symbol lookup, overload resolution, or source loading.

### P. Provenance and source-binding invariants

- P1. Every formula entry binds exactly one source binding whose shape matches
  its `source_class`, and references exactly one provenance entry binding the
  target VC and the formula fingerprint.
- P2. Imported axiom/theorem entries carry the full identity 5-tuple (package
  id, module path, exported item id, statement fingerprint, required proof
  status) and must match a caller-context `FormulaImportedFactEvidence` entry
  exactly; absence, identity mismatch, or fingerprint mismatch is
  `unresolved_symbol`.
- P3. Proof-status strength is ordered `kernel_verified > discharged_builtin >
  externally_attested_policy_permitted`; evidence accepted under a weaker
  status than required is `unresolved_symbol`; externally attested imports are
  additionally gated by the profile policy and taint the result.
- P4. Local-hypothesis, cited-premise, and generated-VC-fact bindings must be
  verifiable against the caller's immutable context identity, not merely
  well-shaped (architecture 15, "Context Identity Covers Non-Imported Source
  Bindings"; finding F2). The corrected checker requires the task-28
  context-identity payload before SAT encoding, matches every non-imported
  formula entry against immutable source/id, formula-id, and formula-fingerprint
  rows, and rejects missing, stale, or ambiguous identity as
  `missing_provenance`.
- P5. `used_axioms` is derived only from accepted formula evidence whose source
  class is accepted imported axiom/theorem. Backend-reported used-axiom lists
  are never trusted.

### S. Substitution, alpha-conversion, and freshness invariants

- S1. Substitution payloads are explicit context evidence; a referenced payload
  that is absent is `missing_provenance`, and the kernel never infers a payload
  by diffing source and target terms.
- S2. Binder contexts decode from the entry's own `binder_context_encoding`
  under the v1 grammar; unknown versions/roles, truncation, duplicate frames,
  noncanonical order, and frame/term incompatibility reject as
  `invalid_substitution`.
- S3. Replay is capture-avoiding: a free variable of an inserted actual term
  must not become bound; a collision without a justifying freshness witness is
  `invalid_substitution`, never a silent rename.
- S4. Alpha-equivalence is decided by normalized binder structure and stable
  ids; renaming must be injective per scope; no free variable becomes bound and
  no bound variable escapes.
- S5. Freshness witnesses are fully recomputed: the avoided set is rebuilt from
  the free variables of the source binder body (minus the bound variable) plus
  free variables of inserted actuals; the witness's generated id must be absent
  from that set; the deterministic counter must equal the id's position in the
  manifest-derived candidate stream. Any mismatch is `invalid_substitution`.
- S6. Free-variable side conditions are recomputed at the recorded path from
  the normalized target binder stack; a recorded capture set is not
  self-attesting and must equal the recomputed set exactly.
- S7. Simultaneous-map semantics: multiple replacements apply as one map keyed
  by formal variable id; replacement actual terms are not rewritten by other
  entries of the same payload (no sequential-composition ambiguity).
- S8. Formal variables must occur in the source formula; unsupported payload
  kinds, roles, and non-root rewrite paths (until specified) reject as
  `invalid_substitution`, fail-closed.

### I. Instantiation and SAT-encoding invariants (`sat_encoding`, task 26)

- I1. Instantiated formulas are kernel-derived from checked source formulas and
  checked substitutions. Caller-supplied instantiated formulas, SAT clauses,
  resolution traces, backend proof methods, and logs are ignored or rejected as
  trusted payload.
- I2. SAT variable assignment is deterministic: atom variables by sorted
  canonical atom bytes, Tseitin auxiliaries in deterministic traversal order.
  Equivalent caller order produces identical canonical SAT bytes.
- I3. The encoded problem asserts exactly: all premise formulas, all derived
  instantiations, and the standalone goal with the polarity checked by B4. The
  goal is never also asserted as a premise, and never feeds `used_axioms`.
- I4. Canonical SAT bytes are a diagnostic/check-trace artifact, not trusted
  input; encoded-problem fields are read-only outside the module.

### C. Trusted SAT-check invariants (`sat_checker`, task 27)

- C1. Acceptance evidence is only `SatCheckResult::Unsat` over the
  kernel-derived problem. `Sat` is non-acceptance evidence; solver errors,
  unsupported clauses, and limit failures reject deterministically.
- C2. All `batsat` heuristic and randomization options are pinned to audited
  deterministic values and not exposed to callers; no proof production, model
  enumeration, DIMACS, callback, or process/network surface is reachable.
- C3. Size limits (variables, clauses, literals, clause width, canonical
  bytes) are enforced before solver construction. Exact conflict/propagation
  budgets are unsupported and requesting them rejects (finding F3).

### R. Result and orchestration invariants (`checker`, task 28)

- R1. `accepted` requires every prior invariant class to pass; any sub-check
  failure rejects the whole input, with no repair and no alternate pipeline.
- R2. Batch results are ordered by target VC fingerprint then caller input
  order; worker completion order never affects results or rejection ordering.
- R3. Policy taint from externally attested imports propagates to the result
  and can never be laundered into unqualified `kernel_verified`.
- R4. Deterministic step budgets stop replay as `timeout`; size/memory budgets
  as `resource_exhaustion`; both are non-acceptance and leave the obligation
  unverified.

### L. Legacy-path invariants

- L1. Legacy `Certificate` / `resolution_trace` bytes do not share the v1
  domain separator and reject as `unsupported_certificate_format` under the
  task-25 parser.
- L2. `KernelCheckPolicy.allow_legacy_certificate_audit` defaults to `false`;
  audit mode may replay for inspection but still returns `Rejected` with an
  `unsupported_certificate_format` audit record, and never populates trusted
  `final_goal`, `used_axioms`, witnesses, cache promotion, or
  `kernel_verified`.
- L3. Within audit replay, the legacy invariants still hold: canonical clause
  encoding (sorted, duplicate-free literals), pivot with opposite polarity in
  both parents, resolvent recomputed and compared, parent references only to
  imported clauses, generated clauses, or strictly earlier steps (forward and
  self references are malformed), and final goal resolving to the checked
  canonical empty clause.

### D. Determinism and resource invariants

- D1. Identical evidence bytes, context, limits, and policy produce
  byte-identical results and rejection orderings across platforms and worker
  counts.
- D2. No wall-clock, random state, environment, file system, cache, artifact,
  or global mutable state is read on any accept/reject path.
- D3. Every budget is checked before the corresponding allocation or recursion.

## Replay Semantics

"Replay" is the pure function

```text
(evidence_bytes, immutable_context, limits, policy)
  -> KernelCheckResult (accepted | rejected + stable rejection records)
```

What the kernel recomputes (and therefore does not trust as a field):

| Recorded field | Kernel action |
|---|---|
| formula fingerprints, goal fingerprint | recomputed from parsed trees and compared |
| entry hash inputs, canonical evidence hash | derived from validated bytes only |
| substitution target terms | re-derived by capture-avoiding replay and compared structurally |
| freshness witnesses (avoided set, counter) | fully recomputed from normalized evidence |
| free-variable capture sets | recomputed at the recorded path |
| instantiated formulas | derived from checked formulas + substitutions |
| SAT problem (variables, clauses) | derived deterministically; never read from input |
| UNSAT result | recomputed by the wrapped in-process checker |
| legacy resolvents, cluster/reduction commitments (audit mode) | recomputed and compared |

What the kernel checks against the caller's immutable context (trusted only as
orchestrator input, never producer input): target VC fingerprint, required
check kind / goal polarity, imported-fact identity and proof status, context
provenance fingerprints, policy gates, limits.

What the kernel never does on any path: proof search, premise selection or
minimization, substitution invention, overload resolution, cluster search,
registration activation, implicit coercion insertion, fallback inference,
alternate encodings, ATP/SAT child processes, acceptance from backend-reported
success, or heuristic repair of malformed evidence.

## Rejection Taxonomy And Attacks Prevented

| Stable detail | Category | Attack or failure it prevents |
|---|---|---|
| `unsupported_certificate_format` | certificate | schema/profile downgrade attacks; smuggling the legacy resolution-trace path, backend proof methods, SMT objects, or logs into normal acceptance |
| `malformed_certificate` / `malformed_witness_data` | certificate | parser confusion: duplicate ids, unsorted lists, trailing bytes, noncanonical encodings that could make two readers disagree about evidence content |
| `context_mismatch` | certificate | replaying a valid certificate against a different VC; goal-polarity confusion (B4); profile/context splicing |
| `missing_provenance` | kernel | premise injection: formulas or substitutions whose origin cannot be verified against the immutable context; absent payloads "repaired" by inference |
| `unresolved_symbol` | kernel | dependency-slice desynchronization: citing an imported theorem whose statement fingerprint, identity, or accepted proof status does not match the kernel-accepted artifact; proof-status laundering (externally attested presented as kernel-verified) |
| `invalid_substitution` | kernel | variable capture, binder collision, alpha-renaming forgery, stale/forged freshness witnesses, forged capture sets, unsupported payload shapes accepted silently |
| `invalid_sat_refutation` | kernel | claiming refutation when the derived problem is satisfiable; corrupting kernel-derived SAT material; accepting without an UNSAT wrapper result |
| `invalid_sat_proof` (legacy) | kernel | forged resolution steps, wrong pivots, forward/cyclic derivations, non-empty final clauses in audit replay |
| `invalid_cluster_trace` | kernel | hidden transitive cluster expansion, forged reduction commitments, guard-evidence mismatches |
| `timeout` / `resource_exhaustion` | either | turning nontermination or memory blowup into implicit acceptance; both leave the obligation unverified |

## Edge Cases Reviewed

Each case below was checked against the module specs; disposition is
`covered` (an invariant explicitly handles it), `fail-closed` (rejected by a
conservative rule pending richer specification), or `finding` (see next
section).

1. **Fresh id chosen via an attacker-shaped variable manifest.** The candidate
   stream for freshness comes from the producer-controlled manifest, but S5
   recomputes the avoided set from the binder body and inserted actuals, so a
   colliding id cannot be certified fresh. Collisions with variables outside
   the binder scope are harmless because occurrences inside the scope are, by
   definition, free in the body and hence avoided. `covered`.
2. **Shadowing confusion.** Shadowed binders must use distinct stable ids
   (architecture 16); a term reusing one `binder_id` for two binder nodes is
   rejected (S2). Capture decisions never consult display names. `covered`.
3. **Sequential vs simultaneous substitution.** A formal variable occurring
   inside another replacement's actual term is not rewritten again (S7), so no
   order-dependent result can be certified. `covered`.
4. **Substitution chaining / cyclic derivation.** Substitution records may name
   only formula-evidence entries as sources; there is no derived-formula
   reference, so no chain or cycle exists in the corrected path. Legacy audit
   replay rejects forward/self parent references (L3). `covered`.
5. **Goal smuggled as premise.** The final goal is standalone and never
   asserted as a premise (I3). However, a producer can copy the goal formula
   into a premise entry labeled as a local hypothesis or VC fact; task-31
   context identity verification for non-imported bindings (P4) rejects that
   row unless it is present in the caller's immutable context. `covered`.
6. **Goal polarity confusion.** Asserting the goal true and obtaining UNSAT
   proves the premises refute the goal — accepting that as proof of the goal
   would be unsound. B4 binds polarity to the obligation's check kind.
   `finding` F1, patched in architecture 15.
7. **Contradictory premise set.** If genuine, kernel-checkable premises are
   contradictory, any goal is derivable; this is logically sound relative to
   the imported library and not detectable locally. Mitigation is provenance
   (P2) plus upstream acceptance of the imported facts themselves. `covered`
   (by trust model, with residual global-consistency assumption).
8. **Alpha-variant atoms as distinct SAT variables.** Atoms that differ only by
   binder naming inside terms encode to different bytes only if normalization
   failed upstream; the manifest-validated normalized encoding makes
   alpha-equivalent atoms byte-identical. A producer that fails to normalize
   loses completeness, never soundness. `covered`.
9. **Tseitin polarity errors.** Encoding is fully specified (gate clauses per
   operator, deterministic traversal); it is kernel code, covered by
   determinism and mutation tests, not by evidence trust. `covered`.
10. **Duplicate/unsorted clause literals, tautologies (legacy).** Canonical
    clause encoding rejects duplicates; tautology handling is profile-explicit
    (marker or reject). Mislabeling a clause `tautology` weakens premises only
    (incompleteness, not unsoundness). `covered` (legacy-only).
11. **Fingerprint collision.** Current algorithms are exact canonical bytes
    (identity), so collision-free. Any future digest algorithm must be
    collision-resistant or imported-fact identity could be spoofed; constraint
    now recorded in architecture 15. `finding` F5 (resolved by doc patch).
12. **Imported statement fingerprint vs formula-tree fingerprint.** The v1 rule
    requires equality, which blocks real (rich) imported statements until a
    projection is specified. Sound (fail-closed) but a completeness blocker.
    `finding` F6.
13. **Zero-frame and unused binder frames.** Accepted only through the explicit
    v1 encoding; empty bytes, missing frames, and unused frames reject
    (`invalid_substitution`). `fail-closed`.
14. **Under-binder replacement without capture.** Rejected in task 11,
    semantically accepted only by task-12 rules with witnesses. `fail-closed`.
15. **Local abbreviation closures (`captured_free_variable`).** Payload kind
    and role reserved and rejected until definition-site closure and type-guard
    evidence are specified. `fail-closed`.
16. **Solver nontermination within size limits.** `batsat` exposes no exact
    step budget; a small-but-hard derived problem can consume unbounded time
    inside the trusted wrapper. Never unsound (no acceptance without UNSAT)
    but an availability gap in the trusted base. `finding` F3.
17. **Evidence/context same-channel construction.** If the phase-14 caller lets
    the evidence producer also supply `FormulaEvidenceContext`, P2/P3 collapse.
    The context must come from the orchestrator's kernel-accepted artifacts.
    `finding` F2 (documented in Trust Model; enforcement is integration work).
18. **Batch tie-breaking.** Equal target fingerprints preserve caller input
    order; shuffled construction covered by determinism tests. `covered`.

## Audit Findings

Severity: **High** = a plausible unsound-acceptance path or trust-boundary
hole; **Medium** = soundness-adjacent ambiguity, drift, or availability gap in
the trusted base; **Low** = documentation/consistency debt.

- **F1 (High, patched). Goal polarity acceptance semantics were underspecified.**
  `sat_encoding.md` defines both `AssertFalseForRefutation` and
  `AssertTrueForConsistency`, and architecture 15 said only that `final_goal`
  records "the target formula to refute or prove". Nothing stated that *proof*
  acceptance of a proof obligation requires the refutation polarity. A producer
  could select consistency polarity for a VC whose premises entail `¬goal` and
  obtain UNSAT — i.e., certify a refuted goal as proved. Patched in this
  change: architecture 15 (en/ja) now binds goal polarity to the target
  obligation's check kind from the caller's immutable context and makes a
  mismatch `context_mismatch`. Corpus:
  `fail_certificate_sat_goal_polarity_mismatch_001`. The checker-side B4
  acceptance binding is implemented by `mizar-kernel` task 30, including
  fail-fast `final_goal.polarity` rejection and proof-policy refusal to trust
  accepted consistency checks as proof obligations. The producer-side
  `mizar-vc` handoff declaration/rejection gap was closed by `mizar-vc` task
  27.
- **F2 (High, patched). Non-imported source bindings were not
  verifiable from the specified context.** `FormulaEvidenceContext` carries
  imported axioms/theorems only. Local-hypothesis, cited-premise, and
  generated-VC-fact entries bind nonzero ids and an *opaque producer-owned*
  provenance payload — the kernel as specified can check shape and target
  binding but cannot check membership in the actual VC's local context or
  generated-fact set. An ATP-side producer (a different, untrusted channel from
  `mizar-vc`) could therefore label an arbitrary formula — including the goal
  itself — as a local hypothesis (edge case 5). Patched across the Step 1
  producer/consumer pair:
  architecture 15 (en/ja) now requires context identity to cover non-imported
  source bindings before such entries can be accepted, with fail-closed
  behavior until the verification data exists. The producer-side schema now
  separates the canonical formula-envelope hash from the task-28
  `context_identity_hash()`: the context payload binds each local/VC-fact row
  to the target VC and the opaque `mizar-vc` canonical formula-envelope
  handoff hash. The kernel must not recompute that canonical handoff hash from
  `ParsedKernelEvidence::canonical_hash_input()` because the parser hash input
  is the binary evidence envelope, not the `mizar-vc` handoff renderer; task 31
  instead verifies target/row membership and recomputes the task-28
  context-identity hash from the documented line grammar before acceptance.
  Regression coverage includes valid local/cited/generated rows, missing
  identity, stale target/hash/row payloads, duplicate rows, a goal labeled as a
  local hypothesis without a matching immutable row, and the task-28 golden
  context-identity line grammar.
  Corpus:
  `fail_certificate_symbols_unverifiable_local_hypothesis_001`.
- **F3 (Medium, deferred by design). No exact solver step budget in the trusted
  SAT wrapper.** `sat_checker.md` records that `batsat` 0.6.0 exposes no stable
  conflict/propagation budget, so only size limits guard solve time. This
  cannot produce unsound acceptance but is an availability gap inside the
  trusted base (checker `timeout` cannot fire during solving). Keep the
  deferral explicit and revisit when a dependency exposes a deterministic
  budget API. Corpus covers the enforceable side:
  `fail_certificate_resources_*`.
- **F4 (Medium, patched). Architecture 15 evidence field list drifted from the
  implemented v1 envelope.** The `KernelEvidence` sketch listed
  `imported_axioms` / `imported_theorems` as top-level sections and omitted
  `symbol_manifest`, `variable_manifest`, and `provenance`, while the task-25
  envelope has no imported-fact sections (imported facts are source bindings
  plus caller context). Aligned in this change (en/ja).
- **F5 (Medium, patched). Fingerprint collision-resistance requirement was
  unstated.** Current fingerprint algorithms are exact canonical bytes, but
  nothing prevented a later weak digest from being registered, which would
  make imported-fact identity spoofable. Constraint added to architecture 15
  (en/ja).
- **F6 (Medium, reported). Imported-fact usability is blocked by the
  fingerprint-equality rule.** Requiring the imported statement fingerprint to
  equal the propositional formula-tree fingerprint means realistic imported
  statements (arch-18 statement fingerprints over rich formulas) cannot be
  cited until a source-formula projection is specified. Fail-closed and sound;
  needs a paired kernel/`mizar-vc` schema task before ATP-bound VCs citing
  imports can ever be accepted.
- **F7 (Medium, reported). `mizar-test` has no corrected-path rejection
  vocabulary.** The required-soundness-case registry
  (`REQUIRED_SOUNDNESS_CASES`) pins `soundness.certificate.invalid_sat_proof`
  to the legacy reason and has no `invalid_sat_refutation`, `context_mismatch`,
  `missing_provenance`, or legacy-gate case; architecture 20 explicitly asks
  for "invalid SAT refutation" and "unsupported legacy certificates under
  normal policy" coverage. The new corpus uses non-`soundness.` stable keys for
  corrected-path reasons (the registry rejects unknown `soundness.*` keys);
  extending the registry is a `mizar-test` follow-up task.
- **F8 (Low, resolved by `mizar-test` task 22). Directory naming drift.**
  Architecture 20 now lists `tests/certificates/` as the canonical certificate
  and kernel-evidence corpus root, matching the implemented `mizar-test`
  layout and this corpus. The retired audit-draft name
  `tests/kernel_evidence/` is kept only as historical context where needed.
- **F9 (Low, reported). Legacy tautology-marker semantics are profile-dependent
  and thinly specified.** Mislabeling weakens premises only, so this is not a
  soundness hole; it should still be pinned down or retired with the legacy
  path in task-29 follow-ups.

## Impact On Crate TODOs (report only; revisions are follow-up tasks)

- `doc/design/mizar-kernel/en/todo.md`: candidate new tasks — (a) B4
  goal-polarity binding in the corrected check service is implemented by task
  30; (b) context-identity verification for non-imported source bindings (F2)
  is implemented by task 31, using an immutable context payload that carries
  the opaque `mizar-vc` canonical formula-envelope handoff hash plus task-28
  `context_identity_hash()`; (c) revisit the solver step-budget deferral (F3);
  (d) specify
  the imported-statement projection that lifts the fingerprint-equality rule
  (F6, paired with `mizar-vc`).
- `doc/design/mizar-vc/en/todo.md`: producer-side goal-polarity declaration
  and consistency-polarity rejection is resolved by task 27; producer-side
  context-identity payload production for local/VC-fact verification is
  resolved by task 28.
- `doc/design/mizar-test/en/` (out of scope here, reported): extend the
  required soundness-case registry and layout/expectation docs with
  corrected-path rejection reasons (F7). The corpus root naming drift (F8) is
  resolved by task 22.

## Constraints And Assumptions

- This document records the audit baseline as of the architecture 15
  post-closeout correction; later schema tasks must update the invariant
  enumeration in the same change that alters checking semantics.
- The reject-first corpus under `tests/certificates/` references invariant ids
  from this document in its sidecar notes; renaming an invariant requires
  updating those notes in the same change.
- Nothing in this document weakens a module-spec prohibition; where this
  document and a module spec disagree, the stricter statement wins and the
  disagreement is a documentation bug to fix.
