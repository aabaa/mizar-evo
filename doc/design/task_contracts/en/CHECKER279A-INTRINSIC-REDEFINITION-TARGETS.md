# Task CHECKER279A-INTRINSIC-REDEFINITION-TARGETS: intrinsic declaration targets
Canonical language: English; [Japanese pointer](../ja/CHECKER279A-INTRINSIC-REDEFINITION-TARGETS.md).
Status: bounded implementation complete; broader Tasks278/279 and Task49 remain open. Tier: full; public-seam/large-change plan first.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: real mizar-test type-elaboration Cases.
API/invariants: [source redefinition](../../mizar-checker/en/source_redefinition.md); runner: [harness](../../mizar-test/en/harness.md).
Authority: EN03 §§3.4–3.5, EN06 §6.7, EN09 §9.6, EN10 §10.7, EN19 §§19.4.1/19.5 and EN16 §16.6.
Gaps: mixed-source declaration materialization, declaration-owned ordinary-root input, typed target/coherence transport and joint element rules.
## Frozen intrinsic source
Admit normal module-local attribute, binary predicate and unary equals-functor declarations followed by one redefinition; representative Cases separate the three forms.
Each definition block contains its parameters followed by exactly one ordinary declaration or redefinition.
Ordinary parameter positions are Object or Set and refined parameters are Set; require strict sharpening, matching notation/ordered loci and distinct real declaration labels.
Attribute/predicate bodies are equality of their own formal variables; the functor body is its actual variable and return sharpens Object to Set.
Coherence is an actual unlabelled Proof with one unjustified Thus thesis; retain a Pending request without checking that proof.
Omitted coherence target must uniquely match a visible earlier ordinary root of the same actual source kind, notation and arity.
Normalize the complete eligible declaration signatures in one owned domain; require position-wise intrinsic sharpening and at least one strict position.
Check functor return compatibility separately; return type, declaration/import order and any use-site winner never choose a target.
Support no imported/template/nominal/attributed signatures, explicit targets, refinement chains, notation aliases or accepted coherence in this initial profile.
Preserve existing marked-Set/registration predicate and attribute consumers and all their checks as protected predecessors; their migration is deferred.
## Declaration-root prerequisite and public surface
Task278 owns the sealed ordinary declaration-root result; Task279's three genuine source-kind rules consume that result before target checking.
Use actual original SymbolId/kind/source/type sites and ordered formal bindings; source guard arity includes the attribute subject, unlike resolver metadata; never fabricate a use or parse SignatureShell/goal text.
Two common sealed values, source_redefinition::{SourceOrdinaryRootHandoff, SourceRedefinitionHandoff}, serve all three forms; no per-form public DTO or parallel AST.
Roots retain immutable original TypedAst, SurfaceResolvedArena and SymbolEnv authority plus existing normalization/context/type payloads.
The root result authenticates the complete mixed graph without choosing redefinition targets; ordinary roots are their own actual SymbolIds.
Target relations remain typed symbol/root/coherence-site/obligation joins, queryable in both existing AST containers.
Reuse CandidateDeclarationKind, BindingEnv, TypeNormalizer, TermFormulaChecker, original SourceNode rows and InitialObligationTable.
Do not bypass or silently generalize the sealed legacy per-form context/definition constructors or use proof-local declaration surrogates.
Join each attribute subject to its actual direct declaration binder; existing formal resolution authenticates body and predicate/functor loci.
## Transaction and trust
One common TypedAst slot/setter and one ResolvedTypedAst slot/getter retain the handoff; no per-form slot or projection adapter.
Add truthful AttributeRedefinitionCoherence and FunctorRedefinitionCoherence kinds; retain PredicateRedefinitionCoherence and actual CoherenceCondition owners.
All requests remain Pending; target inference supplies no accepted equivalence, value equation, active refinement, fact, checked proof or VC.
Root and inference type IDs keep their owning universes and join through original sites/types, never coincident numeric IDs.
Fresh construction/replay authenticates original/native/source/module/symbol/context/binder/type/body/coherence/census correspondence.
Stage all node/context/type/obligation changes, validate before assignment and reject foreign, repeated or overlapping installation transactionally.
Resolved assembly replays the same associations; declaration-only sources keep genuinely empty overload stages and no selected-root claim.
AttributeRedefinition and FunctorRedefinition source rules ship together, used by construction and fresh replay; update only their blocked ledger rows on completion.
## Test-first and exit
Add three real type-elaboration sources/sidecars for intrinsic attribute, predicate and functor redefinition before semantic implementation.
Use existing stage-compatible trace rows without changing status/coverage/required/prior evidence; no new semantic/proof acceptance credit.
Freeze the public interface and complete manual primary oracles from actual sources before semantic logic.
Interface-only unsupported scaffolding may make the primary compile; compile errors are not RED.
Require clean actual frontend/resolver/generic receiver and formal-role preconditions, then genuine executable unsupported RED; parent reads the complete primary before logic GO.
Assert complete original declaration/formal/body/type/context census, ordered root signatures, actual targets, truthful Pending obligations and both installed containers.
Cover renaming/layout, wrong/foreign/future/non-strict/reverse/ambiguous targets, type universes, kind/arity/loci/body/return/coherence swaps and rollback.
Refuse unsupported explicit/chain/imported/attributed forms; preserve existing active predecessors and every old semantic oracle.
Each real Case must inspect installed root/target/type/Pending data, not merely producer success; no fake or inactive-only consumer completes this task.
Independently review source-derived inventory/selection/count maintenance before editing existing assertions; never rebaseline from output.
Update only paired owner/harness API/input boundaries, permitted plan/TODO links and the coverage audit's bounded transport ownership.
Run fmt, workspace Clippy, BOTH lint_policy, whole checker/mizar-test owners, metadata, verifier standard and affected type/overload/AST groups.
Run short type-elaboration/syntax-smoke and protected predecessor Cases; new selected sources require full cargo test before commit.
Independent PRE/spec/test-intent/trust, executable RED, POST/sufficiency, pure volume and consistency precede final nine gates/score>=90.
