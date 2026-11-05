# Task CHECKER270-INLINE-BINDINGS: native inline closure and substitution requests

Canonical language: English; [Japanese pointer](../ja/CHECKER270-INLINE-BINDINGS.md).
Status: complete. Gate tier: full. Extend the existing native proof-local transaction.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); lower: [resolver plan](../../mizar-resolve/en/00.crate_plan.md).
Consumer: [test plan](../../mizar-test/en/00.crate_plan.md); dependencies: committed native269 rows/scope/slot, ready.
Gap: source_drift/test_gap: native inline roles lack authenticated checker request consumption.
API: [native bindings](../../mizar-checker/en/source_proof_local_declaration.md#native-proof-local-binding-boundary); runner: [harness](../../mizar-test/en/harness.md).
Audit impact: additive request-only coverage; semantic capture-substitution stays deferred.
Retained replay owner: [MT10-AS](../../mizar-test/en/todo.md), architecture16 Substitution Replay.
## Authority and boundary
- EN04 §4.4.3, EN15 §15.2.3–4/§15.11.3 and architecture16
  Definition-Time Closures own formal identity, definition-time capture and
  guards before expansion. Existing object/equality inline fixtures remain.
- Accept both InlineFunctorDefinition and InlinePredicateDefinition together;
  ship their blocked Task285 element rules with this producer.
- Reuse the existing handoff, original graph, sealed resolver scope, BindingEnv
  and Typed/Resolved slot; add no parallel body graph or request table.
- Inline names belong to the enclosing lexical context; ordered formals belong
  to the definition context. Preserve original body/type/actual roles and IDs.
- Recompute free captures from original body references and lexical ancestry,
  excluding formals and body-local binders, without transitive flattening.
- Validate definition kind/arity, formal order/guards, body/result-type roles,
  callee identity and ordered actual-to-formal substitution requests.
- Written guards remain pending. No inferred type, fulfilled guard, expanded
  result, proof fact, theorem acceptance, Core or VC is produced.
- Correct body-local capture selection only in opt-in proof-bindings resolution;
  preserve existing resolver profiles and semantic consumers.
## Consumer and tests
- Execute genuine build, Typed installation and Resolved replay through a real
  advanced-semantics request consumer; original associations must be consumed.
- A distinct request smoke uses outer `m`, object-typed formal `n`, body `n=m`,
  then a distinct shadowing `m` actual. Both inline kinds, zero/multiple ordered
  formals and zero-arity calls are covered across source regressions.
- Manual request oracles distinguish formal, captured and actual identities,
  original body/type sites, ordering and explicit pending guard state.
- Reject malformed/foreign roles, arity/body/guard/capture substitutions,
  coherent replay corruption and legacy-scope capture bypasses.
- Cover body-local quantifier/comprehension exclusion, layout/name variation,
  old269 forms, both-order slot exclusion and unchanged semantic tables.
- Test-first .miz/sidecar additions precede implementation. Existing semantic
  fixtures, expectations and trace statuses are preserved.
## Exit and retained work
- Task270 exits at the complete producer plus actual request-consumption
  boundary. The stronger MT10-AS capture-substitution smoke remains open.
- Keep the existing semantic capture-avoidance requirement deferred: explicit
  guard validation, substitution replay/result oracle and binder freshness are
  still required. A metadata request does not satisfy those obligations.
- Run fmt, Clippy, both mandatory lint_policy suites, owner/corruption tests,
  metadata/global-tally checks and actual advanced/declaration/type/formula
  commands. Resolver capture changes require affected heavy checks. Admission
  selection change and producer milestone exit require full cargo test.
Independent authority/test-intent, sufficiency, implementation, volume and consistency reviews are required.
