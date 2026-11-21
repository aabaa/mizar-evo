# Task ATPK-THEOREM-FINDER: scoped theorem instance-finder compatibility
Canonical language: English; [Japanese pointer](../ja/ATPK-THEOREM-FINDER.md).
Status: frozen. Tier: full. Owner: [ATP plan](../../mizar-atp/en/00.crate_plan.md); consumer: parked Driver P22.
Dependencies: [backend-free finder](ATPK-INSTANCE-FINDER.md), existing [scoped theorem producer](../../mizar-atp/en/translator.md#source-theorem-scoped-sequent) and configured local handoff.
Authority: spec [§21.7.1–§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#217-atp-integration), [architecture 10](../../architecture/en/10.atp_backend_integration.md#evidence-extraction-by-instance-finding).
Gap: source_drift — the scoped producer has zero external axioms, one local guard and a universally closed guarded conjecture; the finder assumes axiom/formula count equality and a ground goal.

## Scope and boundary

Extend the existing finder for this bounded scoped theorem shape, using the
original local handoff and native encoded problem. Authenticate the one local
guard, shared binder, reflexive local equality, complete primitive provenance
joins and target/context bindings against the composite conjecture metadata.
Rebuild the exact handoff-bound metadata suffix; require identical canonical
source/module/binder prefixes across guard/binder/closure roles and the
GeneratedBinder(proof-let:D) association. The handoff manifest does not
independently authenticate Core source/module/binder identity.
Opaque source/module metadata is retained, never interpreted as formula content
or a new origin authority. The guard remains local, never an external axiom.
Use the existing deterministic equality-name search with the authenticated
local variable as a rigid equality term, outside the ground substitution domain.
Charge it against the existing term limit, with no guard instantiation charge;
retain literal equality-name and Kernel-query limits. Exact sufficient limits
are [1,0,1,2]. Return only an actual ordinary Kernel-checked candidate.
No candidate substitution, new public seam, language/specification intent,
Kernel/Proof acceptance, Driver registration, owner-gated work, source activation,
publication, reuse, or semantic coverage credit. General quantified conjectures,
non-reflexive scoped goals and broader theorem extraction remain deferred.
Existing generic finder and backend failure behavior must remain unchanged.

## Artifacts and tests

Source: existing ATP instance_finder.rs; translator/backend only if required by
primitive correspondence. Tests append to existing genuine source theorem tests;
use the current real source fixture and pinned guarded Vampire runner, with no
fabricated positive input. Owner: [finder API/invariants](../../mizar-atp/en/instance_finder.md), paired EN/JA; translator consumer deferral wording only if stale.
Audit: no coverage, ownership or general extraction deferral changes.
Matrix cap: six families (variants within each family share its oracle):
1. Genuine source through both encoders, repeat/purity and independent normal Kernel acceptance with original local wire.
2. Genuine guarded Vampire Unsat produces finder evidence with zero axioms; advisory absence/empty reports keep the existing meaning.
3. Stale encoded/target/handoff and altered primitive metadata fail closed.
4. Malformed binder/guard/equality/source/role or unsupported closure fails closed.
5. None and zero/exhausted term, equality-name or query limits yield no evidence; zero premise limit and exact [1,0,1,2] yield a candidate.
6. Existing generic finder and unsuccessful/truncated/non-Unsat backend regressions remain unchanged.
Before expanding the matrix, establish the genuine shared fixture and confirm
one case reaches finder setup; a setup gap outside ATP scope is a stop condition.

## Reviews and exit

Independent specification/test-intent, sufficiency, implementation, volume/scope,
and consistency/quality reviews. Run fmt, all-target/all-feature Clippy with
-D warnings, both required lint_policy suites, ATP owner tests, new genuine
source tests and a short guarded backend E2E. Run changed evidence obligations;
defer unchanged heavy matrices under protocol cadence. Full cargo test only on
a named cadence trigger; this local lane unit has no push or tenth-task trigger.
Exit: actual scoped theorem finder candidates pass independent normal Kernel
checking, refusals remain fail closed, reviews and applicable gates pass.
