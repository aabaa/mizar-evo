# Task CHECKER269-NATIVE-BINDINGS: original-source proof-local binding transaction
Canonical language: English; [Japanese pointer](../ja/CHECKER269-NATIVE-BINDINGS.md).
Status: complete. Tier: full; plan first for the public seam and production scope.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); lower owner: [resolver plan](../../mizar-resolve/en/00.crate_plan.md).
Consumer: [test plan](../../mizar-test/en/00.crate_plan.md). Dependencies: generic rows, Task 286 and direct Set captures.
Authority: EN04 §§4.4.1,4.4.3,4.6; EN15 §§15.2.1–2,15.3.3–4,15.4.4,15.10; EN16 §16.4.2.
Gap: `source_drift`: native names/contexts/RHS associations lack a general checker transaction.
## Boundary
Transport Let, Given, Consider, Set and named Take within authenticated native occurrence grammar.
Reserve, quantifier and comprehension identities are supporting bindings; unnamed Take adds no name.
Set direct captures use definition-site identities; named-Take captures/expansion stay deferred.
No FOL generation, equality axiom, substitution application, type inference, facts or accepted proof.
## Scope
API owner: [native binding boundary](../../mizar-checker/en/source_proof_local_declaration.md#native-proof-local-binding-boundary).
One opaque SourceProofLocalBindingHandoff holds BindingEnv plus shared immutable original
TypedAst/native arena/symbol/sealed-scope authority; no parallel declaration/RHS DTO or public fields.
Add build_bindings on the existing declaration producer, reusing existing errors; getters are
binding_env(), source(), scope(), debug_text(). Original source/scope have real payload consumers.
Recompute source_nodes(native, original) and source_proof_scope_maps on construction and replay.
Use shared original-row helpers and require complete role-to-binding bijection, including unused names.
Preserve identities, lexical boundaries (including empty blocks), separate declaration/reference clocks
and original RHS ownership. Given/Consider share GivenWitness metadata without donor privileges.
Types stay missing. Set captures exclude RHS-local bound variables and do not flatten transitively.
Add one Typed/Resolved slot, fresh empty installation, independent replay and both-order exclusion.
Reject nested original transactions; retain existing APIs, donor restrictions and legacy outcomes.
Add opt-in resolve_proof_bindings and NamedWitness; existing resolver entry points stay unchanged.
The existing native scope validator lazily authenticates either genuine profile; the producer still
independently checks totality, cross-form duplicate names and defining-RHS exclusion.
New-mode RHS lookup excludes its own name, keeps earlier list entries and outer shadows.
## Executable authority
Add test-first combined .miz/sidecar plus named-Take forward refusal, with manually specified payloads.
Use active declaration_symbol domain checker.proof_local_bindings and existing payload-string schema;
execute actual build, Typed installation and Resolved replay after normal frontend/resolver refusals.
Emit binding ID/kind/name/scope/declaration clock; direct capture owner/target; reference ID/target/
scope/reference clock/RHS owner. No fixed source offsets or fixture-specific producer branches.
Add metadata-only trace backlinks; existing fixtures, sidecars, trace statuses and activation rows stay unchanged.
No formula_statement pass may stand for extraction alone. Synchronize owner API/harness and coverage audit.
## Verification and exit
Cover all five forms, mixed lists/conditions, layout/names, empty blocks, shadowing/restoration,
own/forward RHS, omitted unused names, foreign/same-ID authority, malformed roles, coherent capture/
environment corruption, duplicate/both-order installation and Resolved replay; preserve semantic tables.
Run fmt, Clippy, resolver/checker/runner tests, declaration/type/formula corpus, resolver standard and
heavy edit-versus-clean, Driver binding/refusal, docs/ledger/link checks. Evidence/cache heavy matrices
are unchanged and defer to periodic/integration cadence. Full cargo test at native producer milestone exit;
run immediately if implementation changes verification selection/admission policy.
Require independent authority/test-intent, sufficiency, implementation, volume and consistency reviews.
Exit: all five forms have actual active metadata consumption and independent replay with preserved RHS links.
Task 269 stays open if its remaining closure/application consumers are not satisfied; do not reclassify them.
Tasks 270/271 own inline closure requests/reconsider; proof-skeleton and acceptance remain downstream.
