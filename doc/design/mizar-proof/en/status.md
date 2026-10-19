# mizar-proof Status Projection

> Canonical language: English. Japanese companion: [../ja/status.md](../ja/status.md).

## Purpose

`mizar-proof` owns the projection from deterministic proof selections into
artifact-facing and diagnostics-facing proof status records.

Status projection is not proof acceptance. Trusted acceptance and trusted
`used_axioms` originate only from `mizar-kernel` `KernelCheckResult` values
whose status is `KernelCheckStatus::Accepted`, whose evidence check kind is
`ProofObligation`, and that are selected as `KernelVerified` or
`DischargedBuiltin`. Externally attested evidence, backend diagnostics,
backend-reported axiom lists, proof cache records, witness metadata, policy
assumptions, and accepted consistency checks never become kernel-verified
status or trusted `used_axioms`.

## Inputs

Projection consumes:

- the `ArtifactProofSelection` stream produced by deterministic selection
  merge;
- the active `VerifierPolicy` or an equivalent publication profile for the
  current build;
- stable obligation identity supplied by the VC/artifact producer:
  `VcId`, obligation id, `ObligationAnchor`, obligation fingerprint, canonical
  VC fingerprint, canonical local-context fingerprint, and dependency-slice
  fingerprint;
- optional trusted kernel result references for selected trusted classes,
  bound to the selected accepted evidence hash;
- optional diagnostic or explanation references produced by the policy,
  kernel, ATP, or diagnostics owner.

Projection must reject stale or mismatched inputs. A `VcId`,
`ObligationAnchor`, arrival order, completion time, runtime duration, backend
profile runtime, or cache hit alone is never proof identity and is never
sufficient for status reuse.

## Status Model

The internal projection result keeps the selection winner class and the
projected obligation status separate:

| Winner class | Projected obligation status | Trusted | Notes |
|---|---|---:|---|
| `KernelVerified` | `accepted` | yes | Requires a matching accepted proof-obligation kernel result. Projecting the accepted artifact status as publishable also requires matching selected kernel-witness metadata; committed witness refs are published separately by `witness_store::publish_ref`. |
| `DischargedBuiltin` | `accepted` | yes | Requires a matching accepted proof-obligation kernel result. Final artifact witness publication remains an `external_dependency_gap` until artifact schema support exists. |
| `PolicyPermittedExternal` | `externally_attested` | no | Policy-controlled evidence only. It carries no trusted `used_axioms`. |
| `PolicyAssumed` | `policy_assumed` internal status | no | Must remain distinct from accepted and externally attested status. Current artifact schema lacks this public obligation status, so artifact publication is `external_dependency_gap` unless a later schema adds it. |
| `PolicyOpen` | `open` | no | Carries an explanation reference when available. |
| `Rejected` | `rejected` | no | Carries the selected rejection or policy/kernel diagnostic reference. |
| `NoSelectableEvidence` | `open` or `rejected` | no | `open` only when the active policy permits publishing open obligations through `AllowPolicyOpen`; `RecordDiagnostic` is diagnostics-only and projects to `rejected`. It carries a no-selectable-evidence explanation. |

`not_required` is reserved for producer-owned obligations that do not require a
proof selection. It is not emitted from `ArtifactProofSelection`.

The projection must never collapse `DischargedBuiltin` into `KernelVerified`,
`PolicyAssumed` into `externally_attested`, or any non-trusted status into
`accepted`.

## Trusted `used_axioms`

Trusted `used_axioms` projection is allowed only when all of the following hold:

1. the selected winner class is `KernelVerified` or `DischargedBuiltin`;
2. the selected proof selection reports trusted `used_axioms` availability;
3. the projection input includes the accepted proof-obligation kernel result
   or a trusted kernel-owned reference derived from it;
4. the accepted proof-obligation kernel evidence hash matches the selection's
   selected evidence hash;
5. the kernel result is not policy-tainted, has status `Accepted`, and has
   evidence check kind `ProofObligation`.

When these conditions hold, projection may expose a trusted used-axiom
reference or ordered axiom list exactly as supplied by the kernel-owned
accepted result. Projection must not merge in backend-reported axiom lists,
externally attested citations, cache dependency records, diagnostic hints, or
policy-assumption dependencies.

All non-trusted statuses set trusted `used_axioms` to absent. They may carry
untrusted diagnostic suggestions only through diagnostics-owned references that
are not accepted dependency facts.

## Diagnostics And Explanations

Every non-accepted projection has a stable explanation surface made from
metadata and diagnostics-owned references:

- `externally_attested` records the external admission status, policy
  fingerprint, selected class, selection reason, and optional explanation ref;
- `policy_assumed` records the selected class, selection reason, policy
  fingerprint, and optional explanation ref;
- `open` records the selected class, selection reason, ordered diagnostic refs,
  and optional explanation ref;
- `rejected` records the selected class, selection reason, ordered diagnostic
  refs, and optional explanation ref;
- `NoSelectableEvidence` records the selected class, selection reason, ordered
  diagnostic refs, optional explanation ref, and generated diagnostic result
  id.

Structured policy-assumption, open-obligation, rejection-layer, and backend
exhaustion details are diagnostics-owned payloads referenced through stable
diagnostic or explanation refs. Status projection does not embed those payloads
directly.

Diagnostic ordering follows architecture 19: source identity, source range,
phase order, severity, diagnostic code, and stable detail key. Parallel
completion order, backend runtime, and cache lookup timing do not participate
in diagnostic ordering.

## Artifact Projection

Projection may populate artifact obligation fields only from stable projection
data:

- `status`;
- `accepted_witness_obligation_id` when a trusted accepted witness is
  publishable;
- `diagnostic_ref` or explanation reference for non-accepted outcomes;
- policy and obligation fingerprints used by artifact consistency checks.

Current `mizar-artifact` `ProofWitnessRef` schema version `2.0` accepts
trusted `ProofWitnessRef` values for `kernel_verified` formula/substitution
evidence only. Therefore:

- `KernelVerified` can project an accepted artifact status as publishable only
  when matching selected witness metadata is available; status projection does
  not itself publish a committed witness ref;
- `DischargedBuiltin` remains an accepted internal projection but cannot publish
  a trusted artifact witness ref yet; its deterministic discharge hash remains
  internal projection and proof-reuse metadata under the current artifact schema
  and must not be written as an accepted artifact obligation field;
- `PolicyAssumed` cannot be losslessly represented in current
  `ObligationStatus`.

These are integration gaps, not permission to emit placeholder witnesses or to
rename statuses.

## Proof Reuse Metadata

Status projection exports validation metadata for proof reuse:

- selected winner class;
- projected obligation status;
- `ObligationAnchor`;
- obligation fingerprint;
- canonical VC fingerprint;
- canonical local-context fingerprint;
- dependency-slice fingerprint;
- policy fingerprint;
- selected evidence hash;
- accepted proof-obligation goal polarity for trusted kernel selections, when
  present;
- selected proof witness payload artifact hash (`witness_artifact_hash`) when
  available for a `KernelVerified` candidate; this is not a committed
  `ProofWitnessPublishedRef`;
- deterministic discharge hash, when present;
- trusted used-axiom reference hash, when present;
- external admission status, when present;
- matching proof-evidence identity, including selected candidate id,
  selected-candidate provenance hash, selection reason, and tie-break key hash;
- dependency artifact fingerprint and compatible dependency/proof-reuse schema
  versions, when supplied by the artifact/cache boundary;
- diagnostic or explanation reference hashes.

This metadata is a cache validation predicate. It is never proof authority.
Reuse additionally requires matching `ObligationAnchor`, canonical VC
fingerprint, canonical local-context fingerprint, dependency-slice fingerprint,
compatible verifier policy, matching proof evidence, and compatible referenced
dependency artifacts and schemas. Cache records cannot promote externally
attested, assumed, open, rejected, or no-selectable outcomes to trusted status,
and cannot synthesize trusted `used_axioms`.

Status projection also exports a stable proof-reuse validation hash over every
field above. `mizar-cache` may compare that hash and the structured fields as a
future reuse predicate, but a match only avoids recomputation after cache
validation. Missing dependency artifact/schema compatibility, policy
incompatibility, witness hash mismatch, deterministic discharge mismatch, or
proof-evidence identity mismatch, including an accepted-goal-polarity mismatch,
is a miss. A match never upgrades
`ExternallyAttested`, `PolicyAssumed`, `Open`, `Rejected`, or
`NoSelectableEvidence` to `Accepted`, and never creates trusted
`used_axioms`.

A complete proof-reuse predicate is class-aware: `KernelVerified` requires the
selected proof witness hash, `DischargedBuiltin` requires the deterministic
discharge hash, and non-trusted classes remain exported metadata only rather
than complete proof-reuse hits.

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `STATUS8-G001` | `external_dependency_gap` | Current artifact obligation status lacks a distinct public `policy_assumed` value. Projection must keep it distinct internally and defer artifact publication rather than collapse it. |
| `STATUS8-G002` | `external_dependency_gap` | Current `ProofWitnessRef` trusted reader rejects `DischargedBuiltin` witness publication. Projection may record the trusted status and deterministic discharge hash, but witness publication stays deferred. |
| `STATUS8-G003` | `deferred` | Diagnostics, artifact emission, manifest commit, cache lookup, and ATP early-stop integration consume this projection in later tasks. This spec defines stable metadata only. |

## Public Enum Policy

Task 14 applies the public-enum forward-compatibility procedure to this
module. All public status-projection enums are downstream-facing API surfaces
and must remain `#[non_exhaustive]`; downstream consumers must keep wildcard
match arms. Artifact-facing status enums additionally require artifact schema
compatibility review before new variants are published or mapped to current
artifact fields.

| Enum | Compatibility decision |
|---|---|
| `TrustedUsedAxiomsError` | forward-compatible |
| `ProjectedProofStatus` | forward-compatible |
| `CurrentArtifactObligationStatus` | forward-compatible with artifact compatibility review |
| `ArtifactPublicationGap` | forward-compatible with artifact compatibility review |
| `ArtifactStatusPublication` | forward-compatible with artifact compatibility review |
| `StatusProjectionError` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Non-Goals

Status projection does not run ATP backends, perform SAT solving, call the
kernel, invent substitutions, select premises, query proof caches, stage or
publish witnesses, write artifact manifests, or accept proofs.

## Source canonical components

`source_registration_canonical_components(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId) -> Result<(CanonicalVcFingerprint, Hash, Hash), String>` returns the selected current leaf's whole VC fingerprint, goal hash and visible-context hash, in that order, using existing types and content profiles.
Freshly call the source proof checker, unchanged Core lowering and source VC generator on these immutable inputs, in that order. Reuse existing Proof generation schema source-existential-registration-v1 and VC schema vc-v1. Preserve all old source facades and their check/error order; this operation neither calls them nor performs default discharge or a Kernel check.
Call the [arena owner producer](../../mizar-resolve/en/symbols.md#arena-registration-owner-payloads) on the same source/env. Gather the distinct Core roots reached from every original VC goal and GeneratedFact premise through actual generated Ref/Not/And/Or rows. Use actual current ids only for traversal; admit no caller graph or reconstructed payload. Refuse missing/unsupported traversal data atomically with internal registration.proof.invalid_source_components.
Invoke [Core formula content](../../mizar-core/en/elaborator.md#source-registration-formula-payloads) with the same retained checker result, locally lowered Core, unmodified local owner map and that exact whole-root set. Then obtain the entire [canonical VC map](../../mizar-vc/en/vc_ir.md#source-registration-canonical-vc-fingerprints); require the selected current VcId, otherwise the same internal error. Obtain its [separate goal/context values](../../mizar-vc/en/vc_ir.md#source-registration-canonical-goal-and-context) with the same local Core/map. Propagate every upstream error unchanged before selection failure; return the triple only after all operations succeed.
This binds genuine producer outputs within this call; opaque owner/formula bytes are never caller inputs. It is raw bounded content, not an execution receipt or transferable trust token. Caller still owns source/snapshot currentness. Preserve existing hash algorithms/domains/schema framing and generic anchor/dependency/status behavior; no new encoder, public type, field, adapter, dependency or lower-stage API is needed.
Neither a valid tuple nor an unproved/false source body implies acceptance. Full normalized owner/path/source shape, composed phase context/dependencies, six-field Proof identity, policy/evidence/witness association, trusted metadata, reuse eligibility, source activation and publication remain separate. Proof23/Artifact25/17 and Steps6–8 remain open.
Three appended genuine-source families reuse existing setup: independent frozen-profile whole/goal/context expectations for current owner/leaf associations and equality with real lower producers; neutral edits, allocator/current-id relocation, supported declaration/registration reordering and participating module/owner/body changes with goal/context separation; real foreign/malformed/unsupported source inputs, absent selection, upstream error precedence and unchanged API inputs. Existing fixtures, expectations and all old tests remain untouched; no invented accepted control or duplicate producer-profile matrix is allowed.

## Source theorem canonical inputs

`source_theorem_canonical_inputs(check: &mizar_checker::type_checker::SourceTheoremCheck<'_>, snapshot: BuildSnapshotId, vc_id: VcId) -> Result<(CanonicalVcFingerprint, Hash, LocalContextFingerprint, DependencySlice), String>` is one inline public function in existing status.rs, returning VC, goal, local context and the full original owned slice, using existing types only. Caller supplies no graph, opaque map/bytes, identity, policy, evidence or accepted result.
First call unchanged lower_source_theorem_skeletons(check, None), preserving its actual error and retaining that fresh local Core.
Collect roots from every actual Core obligation seed, each context's copied ids followed by its optional goal, without indexing, deduplication or fallback. Call [Core56 source_theorem_formula_payloads](../../mizar-core/en/elaborator.md#source-theorem-formula-payloads) with the same borrowed check, Core and roots; preserve its complete profile validation and errors, including Lemma/computation refusal, without copying its validator.
Call [generate_source_theorem_vcs](../../mizar-vc/en/generator.md#source-theorem-vc-generation) on that Core with caller snapshot, generation schema source-theorem-local-v1 and VC schema vc-v1; preserve every surrounding operation, validation, order and error.
Call the original Open graph's [VC70 source_theorem_canonical_components](../../mizar-vc/en/vc_ir.md#source-theorem-canonical-components)(vc_id, &core, &formulas) with that exact unmodified local map. Preserve every VC69 replay/VC70 error unchanged and all existing content grammar/hash behavior; never relabel a handoff hash or re-encode values.
Only then run existing [try_compute_dependency_slices](../../mizar-vc/en/dependency_slice.md) over the entire same original graph with discharge_output None, preserving typed errors through Display. Select vc_id from those actual slices, clone its full DependencySlice or return exactly internal theorem.proof.invalid_source_canonical_inputs; return the four-tuple atomically after all operations succeed.
Keep actual Open/TheoremProofStep, entries, unknowns, IncompleteUncacheable/requires_cache_miss and the collector's original fingerprint. Generic Core fingerprints remain None and original anchors incomplete/unchanged; canonical content neither replaces conservative coverage nor makes unavailable Core/owner/syntax content participate in its partial fingerprint.
All inputs remain immutable. VcId selects this call's graph; an equal numeric foreign handle is indistinguishable. The association binds actual producers within this call, without an execution token or currentness authority; source/snapshot currentness remains caller-owned, and skeleton checking is not proof acceptance.
No default discharge, Kernel/ATP/policy operation, Complete marker, cache lookup, source activation, trusted metadata or witness/publication is produced. Source owner/declaration syntax, complete anchor/ProofObligationIdentity, composed phase dependencies, fresh normalized acceptance origin/classification/evidence/witness, ordinary ATP projection, Driver/runtime and Steps6–8 closure remain separate.
Append exactly two source_theorem_canonical_inputs_* families inside existing term_proof_tests, reusing with_step5c14_theorem_check without helper/extraction changes. Family one sends original T, distinct bound/local-name variants, trivia, genuine owner U and snapshots777/778 through the facade: compare all three components with independent fixed-schema literal hashes prepared/reviewed outside doc/ before Rust and actual lower VC70 output; independently regenerate the original generic graph under these schemas and collect with None to compare every slice getter/entry/unknown/status/fingerprint, including Open TheoremProofStep, nonempty unknowns/cache miss, None generic fingerprints and unchanged incomplete anchors. Require repeated equality and borrowed-check immutability; neutral edits/snapshots preserve components without assuming partial-slice fingerprint changes or stability.
Family two compares valid-source absent VcId(usize::MAX) directly with existing genuine generate_core_vcs/VC69 missing-target error; genuine proofless Open must preserve actual lowerer status_unsupported before valid/absent selection. Genuine Lemma and computation lower successfully and preserve actual Core56 profile errors before selection; computation's empty context exercises safe root collection. Obtain all error oracles directly from actual lowerers/producers and reach each asserted API; reuse Core56/VC69/VC70 matrices, preserving old source/test/helper/final-brace bytes and fixtures/expectations, with no fabricated successful lower, foreign-byte attestation, synthetic collector failure or Accepted/Hit/Complete control.

## Source registration candidate anchors

`source_registration_candidate_anchor(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId) -> Result<(ObligationAnchor, CanonicalVcFingerprint, Hash, Hash), String>` returns the selected current leaf's candidate anchor, whole VC fingerprint, goal and visible-context hashes, in that order. Use existing types; caller supplies no Core/VC graph, opaque payload, identity or accepted flag. Caller still owns source/snapshot currentness.
Freshly run the unchanged checker, Core lowering and source VC generator with the existing Proof generation schema source-existential-registration-v1 and VC schema vc-v1. Follow [source canonical components](#source-canonical-components) for actual arena owner production, whole-graph Ref/Not/And/Or Core-root collection, Core formula production and whole canonical VC map; preserve upstream errors and use internal `registration.proof.invalid_source_anchor` for missing/unsupported traversal or association. Inline this bounded assembly without modifying earlier facades or inventing a shared two-use pipeline helper.
Next call [Resolver syntax production](../../mizar-resolve/en/symbols.md#source-registration-syntax-payloads) on the same source/env. For every actual Core Registration item, join its actual SymbolId to the unmodified local owner and syntax maps and lift those values to its current CoreItemId; require complete syntax coverage. No parsing, manufactured origin/label, opaque caller map or downstream re-encoding is admitted. Call [VC candidate-anchor production](../../mizar-vc/en/vc_ir.md#source-registration-candidate-anchor-hashes) with this same Core/formula/syntax/owner data before selected-leaf lookup. Only after complete success require that VcId in both whole maps, otherwise the internal error, and obtain its genuine goal/context values.
Create existing Proof ObligationAnchor with the exact string `source-registration-candidate-anchor-v1:` followed by lowercase hexadecimal of all 32 VC candidate-hash bytes. This is a reversible presentation of the VC-owned profile, with no Proof-owned hash/encoder or identity substitution. Return only after the existing nonempty-anchor constructor succeeds. Preserve all immutable inputs and original anchors; no default discharge, Kernel call, policy evaluation, trusted metadata, Complete marker or dependency/status mutation occurs.
This call binds genuine producer outputs for a best-effort cross-edit candidate. Matching it alone never authorizes reuse; whole VC, local context, complete dependencies, compatible policy and checked evidence remain separately required by architecture22. Full structured anchor/Proof identity, source activation, accepted metadata, witnesses, publication and runtime integration remain deferred. A false/unproved source can still have a candidate; Proof23/Artifact25/17 and Steps6–8 remain open.
Shared integration tests are owned by the [VC profile](../../mizar-vc/en/vc_ir.md#source-registration-candidate-anchor-hashes).

## Source dependency slices

`source_registration_dependency_slice(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId) -> Result<DependencySlice, String>` returns the existing owned full [VC-local slice](../../mizar-vc/en/dependency_slice.md), including its entries, unknowns, completeness, status and fingerprint. Caller supplies no Core/VC graph, discharge output, payload, identity, policy or accepted flag.
Freshly call the unchanged source proof checker, Core lowering and source VC generator with the existing Proof generation schema source-existential-registration-v1 and VC schema vc-v1, propagating their errors unchanged. Compute the entire original Open graph's slices with existing try_compute_dependency_slices and discharge_output None; map any lower error through its Display text. Only then select the requested VcId, clone its existing slice, or return internal registration.proof.invalid_source_dependency_slice. Preserve immutable inputs and every earlier facade/body; inline this small pipeline without another helper, type, field, adapter, dependency or hash profile.
The VcId is a lookup handle into this call's graph, not a provenance/currentness receipt: a numerically equal handle from another graph is indistinguishable. Source/snapshot freshness remains caller-owned. Do not run default discharge, Kernel checks or policy evaluation, mutate status/anchors, or create trusted metadata, witnesses or publication.
Retain all generic Core/Anchor unknown markers and actual IncompleteUncacheable/cache-miss behavior; missing content is never absence. This raw VC-local fingerprint is not the composition of architecture22 phase footprints or the separate source canonical-content profiles. Full six-field Proof identity, complete dependencies, acceptance/reuse and Proof23/Artifact25/17/Steps6–8 remain separate.
Three appended genuine-source families reuse existing source setup: exact full-slice equality with original lower producers under the Proof schemas and single/aggregate leaves; neutral/participating source and snapshot changes retaining actual unknowns/cache misses rather than demanding unavailable content invalidate a partial hash; absent handles, real mixed/malformed/unsupported inputs and upstream-before-selection errors with immutable inputs. Preserve existing fixtures/expectations/tests and reuse lower slice matrices; no synthetic lower error or accepted control.

## Source original obligation identity

`source_registration_obligation_identity(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId) -> Result<(ProofObligationIdentity, DependencySlice), String>` returns bounded raw data, retaining the entire original Open slice, not only its hash. Call [candidate anchor](#source-registration-candidate-anchors) first, then [original slice](#source-dependency-slices), on exactly the same immutable arguments; propagate errors unchanged before constructing anything. No caller graph, identity, policy, evidence or accepted result is an input.
Set obligation_id to the exact anchor string; copy the anchor, whole canonical VC hash, visible-context hash and actual VC-local slice fingerprint into the existing identity fields. The obligation fingerprint uses Artifact CanonicalHashDomain, Interface family `mizar-proof/source-registration-original-obligation`, version 1.0, with no excluded paths. Hash a CanonicalJson object with exactly `obligation_anchor` (original string), `vc_fingerprint`, `canonical_goal_hash`, `local_context_fingerprint`, and `vc_local_dependency_slice_fingerprint` (each hash as lowercase hex of all 32 bytes). Reuse the existing nonempty identity constructor and lowercase-hex helper; constructor errors propagate through Display. No other profile or earlier facade body changes.
Return the full unmodified owned slice alongside the six-field value, including original Open status, entries, unknowns, IncompleteUncacheable and required cache miss. This field is explicitly VC-local, not architecture22 composed dependencies. A numeric handle selects the current input graph even when reused across edits; the same immutable source arguments bind this readout, while source/snapshot freshness remains caller-owned. Default discharge, Kernel/policy checks, accepted states, trusted tokens, status/witness publication and cache lookup are absent.
This original-source profile is independent of candidate/payload-dependent witness profiles and is not a generic composed identity or a reuse/acceptance receipt. Complete phase footprints and same-check original-source identity plus exact checked-payload consumer enforcement remain prerequisites to trusted witness/reuse integration; Proof23/Artifact25/17 and Steps6–8 remain open.
Three appended genuine-source families compare all fields/full slices with existing producers and an independent literal canonical object/domain, exercise neutral and participating edits/snapshots/current handles without demanding unavailable content changes alter a partial slice, and preserve upstream-before-selection failures and input immutability. False/unproved sources may have raw identities; original tests/fixtures/expectations remain unchanged.

## Source candidate checking

`check_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` owns a fresh single-leaf check, not registration activation or publication. The encoding tuple groups the existing explicit profile/format; it adds no configuration type. Re-run source checker and Core lowering, generate the original graph with existing source/VC schemas, then apply actual default Discharge. Use the [actual source candidate factory](../../mizar-atp/en/backend.md#source-registration-candidate-inputs) for the selected NeedsAtp leaf. Unsupported/foreign inputs, wrong or Discharged targets and unschedulable policy return errors without a checked result or fallback.
Compare the candidate's payload kind and complete target, input/provenance hashes, labels and symbol mappings against the locally produced encoded container. Derive UncheckedFormulaSubstitution's encoded_problem_matches from those comparisons; require existing FormulaSubstitution scheduling permission. Caller match booleans, encoded/candidate pairs, contexts, Accepted results or hashes are not inputs. No backend observation is constructed.
Independently rebuild the same authentic source handoff from the fresh Core/whole default-discharge graph. Derive expected target/profile, parse actual returned v2 bytes with schema 2 explicitly allowed, and construct the ordinary [Kernel context](../../mizar-kernel/en/checker.md) from this separate handoff's complete identity/producer entries. Reject imported/unsupported context classes rather than invent empty context. Invoke normal check_kernel_evidence as ProofObligation with existing default policy/limits; never enable legacy audit or reuse an earlier result.
Wrap the actual result with KernelPolicyInput::from_kernel_result and explicit AtpFormulaSubstitution origin, then call the existing evaluator. Return its actual PolicyDecision and KernelCheckResult, including actual rejection detail for false goals. Accepted/untainted normal proof checks receive KernelVerified under existing policy; rejection is never manually upgraded or replaced by Open/Builtin/external evidence. Preserve all policy criteria and fingerprints.
Return values are this call's checked output, not transferable authorization for selection, source activation, witness publication, artifact status or cache reuse. The caller owns current source/snapshot freshness. Preserve the original Open/Builtin facade’s public behavior. Generic by/built-in integration, search budgets, imported context, witness/artifact/cache publication and full milestone closure remain deferred.

## Source screened candidate attempt

`try_check_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<Option<(KernelCheckResult, PolicyDecision)>, String>` opts into one bounded source attempt. It accepts the ordinary checker's actual source inputs and policy, with no caller Core/VC/evidence pair/context, Accepted flag or limits override.
Keep the genuine source Checker/Core/original-VC/default-Discharge prefix first and unchanged. At the existing pair-production point, invoke ATP's [screen_source_registration_candidate](../../mizar-atp/en/backend.md#source-candidate-sat-screening) with that actual Core/VcSet/selected VcId, explicit profile/format and policy.instance_search_limits(). Lower producer/profile/whole-graph/target errors precede optional-limit admission; reading None or zero must not bypass the real screen. The ATP-owned admission counts do not bound earlier source reconstruction/extraction work.
Actual screen None (unconfigured, zero/exceeded admission counts, or SAT-not-Unsat) returns Ok(None) before post-pair Kernel scheduling permission. Thus valid lower inputs with absent admission and denied Kernel permission yield None; genuine malformed/foreign/unsupported inputs still error first. There is no legacy fallback, normal Kernel check, acceptance metadata or selection on absence; the screen may already have performed its pure SAT query.
Actual Some(encoded, candidate) feeds that exact pair into the existing payload-kind/full-container association, scheduling-permission, separately rebuilt original handoff, actual-v2 parse, normal check_source_kernel_handoff and ATP-origin KernelPolicyInput/evaluator tail, in that order. Retain default ProofObligation policy/Kernel limits and original permission errors. Do not discard/re-fetch the pair, run a second factory/check pipeline, use candidate-supplied context or invent acceptance data.
The shared tail retains its complete checking/metadata body and wraps its original five values in Some; only the actual screen can return None. This facade passes true to the existing [private payload checker](#source-candidate-witness-projection) and maps only this call's genuine result/decision, discarding other internal outputs as the compatibility checker does. A fresh Kernel rejection remains Some(rejected result, actual decision), never an upgrade or substituted result.
None denotes absence of this explicit attempt, not checked rejection or global-search exhaustion. Presence or absence itself conveys no transferable authority for selection, witness/publication, reuse, source activation or runtime use. Acceptance/classification comes only from the actual normal Kernel result and existing Proof policy. Source/snapshot freshness stays caller-owned; policy identity and all legacy public behavior remain unchanged. Full ATP30/31, Proof23 and Steps6–8, search/enumeration/cumulative enforcement, backend execution, complete footprints and trusted publication remain deferred.
Test design: append three compact genuine-source consumer families using existing helpers and source_candidate_check_bytes; preserve the entire old runner/tests.rs prefix, helpers/oracles/fixtures and source_registration_instance_search_limits_preserve_fresh_check_results. Cover None/zero and actual-derived exact/MAX/representative underflow versus viable legacy checking; screen plus fresh normal Kernel/policy compatibility for single/aggregate/renamed/negative sources in both formats, false-dependent absence and independent true siblings; upstream/foreign/wrong/Discharged/unsupported-profile errors, scheduling and combined-absence order, and source/policy purity. Derive counts from actual ground pools, parsed v2 substitutions, equality names and one query; reuse the lower coordinate matrix without duplicating SAT/Kernel matrices or synthetic Accepted controls.
Successful screen and ordinary pairs are equal, so runtime result/byte equality proves admission and fresh-result compatibility only. Independent code review must trace the exact screen-returned pair into the shared tail with no discarded pair/second fetch. No observer/counter/injected writer or static implementation-mirror test is added.

## Source candidate metadata

`check_source_registration_candidate_metadata(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision, Option<KernelAcceptanceMetadata>, BackendCandidateEvidence), String>` retains only this call's actual checked ATP source candidate and raw acceptance association. The compatibility checker and this facade each call the private payload checker described under [source witness projection](#source-candidate-witness-projection) once and discard only unused outputs. Preserve all producer/binding/scheduling/parser/context/check/error order and classifications above, the selector's independent source/candidate reconstruction and all Builtin paths. The raw component/version mapping remains unchanged.
Return the existing actual candidate container from the normal factory, including exactly the Formula-v2 bytes parsed and checked here, target, encoded-input hash, ATP provenance, labels, symbols and candidate identity. Accept no caller candidate/result/metadata; a successful encoding or returned container is no acceptance by itself. Original Open/Builtin targets are not substitutes for this post-default-discharge NeedsAtp target.
Metadata is Some only for KernelVerified plus actual Accepted, untainted ProofObligation, AssertFalseForRefutation and an available accepted identity from KernelPolicyInput with AtpFormulaSubstitution origin. Require the actual handoff's local_evidence_component_hashes; an unavailable component association yields None, never invented data or a changed check/classification. Rejected checks retain their actual candidate and None; upstream errors propagate unchanged.
Wrap raw profile/formula/substitutions/provenance components as Interface hashes with families `mizar-vc/local-evidence-component/{profile,formula,substitutions,provenance}` and version 1.0. Preserve the [VC canonical-v1 content profile](../../mizar-vc/en/kernel_evidence_handoff.md#local-evidence-component-hashes): these hashes exclude v2 section-7 equality-instance names. The returned complete checked candidate bytes include those names; these component identities are not a full v2 payload hash.
Policy is Interface `mizar-proof/verifier-policy`, version `(policy.schema_version(),0)`, digest actual policy fingerprint. Checker version is `(policy.checker_schema_version(),0)` and evidence version is `(parsed.schema_version(),0)`, recording the actual checked v2 wire rather than the handoff envelope's v1. The configured checker requirement does not authenticate a running implementation/toolchain.
Target binding is Interface `mizar-vc/kernel-evidence-handoff-canonical` 1.0 with this handoff's canonical hash. Accepted result is Interface `mizar-proof/kernel-accepted-evidence` 1.0 with this result's actual ATP-origin accepted identity. Formula_context_hash is None because imported context is unsupported; the distinct local context identity is checked by the normal pipeline, not substituted into that optional field. Local component provenance and ATP candidate provenance remain separate actual identities.
This is plain raw data, not TrustedKernelWitnessMetadata, a full composite obligation/current-source receipt, witness reference, full-payload content hash, backend execution or committed publication. Existing trusted-token constructors, normal/Builtin reference guards, staging/unsupported publication and reuse gaps remain; source/snapshot freshness remains caller-owned. Future witness production must bind the complete checked payload including equality names and the current context/target/policy/result before publication.
Three appended genuine source families compare independently produced candidate/wire/handoff/component/policy/result data for single/aggregate and renamed sources, both formats, snapshots and policy/schema/profile variants; real false-dependent leaves stay rejected with no metadata while independent leaves retain their own identity. Preserve exact compatibility-check and selector intent, mixed/foreign/unsupported and distinct FormulaSubstitution/Builtin permission failures. No modified existing fixture/expectation or synthetic accepted control is allowed.

## Source candidate witness projection

`project_source_registration_candidate_witness(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<(ProofSelection, ProofWitnessRef, Vec<u8>), String>` freshly returns only an unpublished normal ATP-source projection.
The private check_source_registration_candidate_payload keeps seven arguments, extending only its private encoding tuple to (LogicProfile, ConcreteFormat, bool): false uses the ordinary factory, true uses the [screened attempt](#source-screened-candidate-attempt). Return Result<Option<(KernelCheckResult, PolicyDecision, Option<KernelAcceptanceMetadata>, BackendCandidateEvidence, VcKernelEvidenceHandoff)>, String>; ordinary production is always Some. Its four callers are the compatibility checker, raw metadata facade, this projection and the screened attempt. The three legacy callers pass false and remove the internal Some with existing registration.proof.invalid_source_candidate on absence, then retain their original mapping/destructuring and unused-output discards. Preserve their public signatures, values, check/error order, policy None/zero behavior, independent selector reconstruction and all Builtin bodies; reuse the existing pure hex renderer without renaming it. No new type, field, adapter, dependency or lower-stage change.
Require the actual KernelVerified decision, same-call metadata, and genuine KernelPolicyInput with AtpFormulaSubstitution origin. Require the result target fingerprint to equal the actual candidate target, as in the existing selector. Use the actual candidate_id, encoded_problem_hash and ATP provenance with the actual target sort_bytes and supplied policy through the existing trusted-input/selection path; require KernelVerified selection. No second producer/check run or caller evidence is admitted. Non-accepted/missing-association or structural failures return registration.proof.invalid_source_candidate; propagate upstream errors unchanged.
Return exactly the Formula-v2 candidate bytes parsed and checked in this call, including section-7 equality-instance names. Retain the actual handoff's distinct local context_identity_hash; neither its canonical target-binding hash nor absent imported formula_context_hash substitutes for it. Preserve the raw metadata mapping above, including actual parsed evidence version 2 and configured checker requirement rather than runtime attestation.
Use Artifact CanonicalHashDomain with no excluded paths, version 1.0, for two producer-owned profiles. The Artifact family `mizar-proof/source-registration-candidate-witness-payload` hashes a CanonicalJson string of lowercase hex of every checked byte. It is distinct from canonical-v1 component hashes and the witness-store payload profile.
The Interface family `mizar-proof/source-registration-candidate-obligation` hashes a CanonicalJson object with these exact keys:
`checked_target_bytes`, `local_context_identity_hash`, `candidate_id`, `encoded_problem_hash`, `atp_provenance_hash`, `kernel_profile_fingerprint`, `verifier_policy_fingerprint`, `checker_schema_version`, `evidence_schema_version`, `target_binding_hash`, `formula_evidence_hash`, `substitution_evidence_hash`, `provenance_hash`, `formula_context_hash`, `accepted_result_hash`, `accepted_goal_polarity`, `witness_artifact_hash`.
Encode actual target sort_bytes and raw local-context/input/ATP-provenance digests as lowercase hex; candidate_id is its actual string. Copy metadata hashes and witness_artifact_hash as full artifact-hash strings, versions as canonical strings, absent imported context as null and actual accepted polarity as its stable string. No generic VC/discharge/dependency identity is substituted.
Build the existing normal ProofWitnessRef 2.0 with actual candidate_id as obligation_id, KernelVerified/FormulaSubstitutionKernelEvidence, this metadata and hashes. Path is `proof-witnesses/source-registration-candidate-v1/<lowercase-payload-digest>.bin`. Structurally validate through the existing normal Artifact value writer before returning selection/reference/bytes; neither Artifact codec nor schema changes.
This unpublished tuple creates no TrustedKernelWitnessMetadata, draft, stage/file write, committed-publication proof, selected witness/discharge reuse hash, source activation or runtime lease. Source/snapshot freshness remains caller-owned; normal/Builtin version guards, incomplete dependency slices, unsupported publication/reuse and full Proof23/Artifact25/17/Steps6–8 remain open.
Three appended genuine-source families compare independent wire/handoff/metadata/selection and normal-reference round-trip; exercise single/aggregate/renamed sources, both formats, snapshots, policy/schema/encoding profiles and all hash participants, including complete equality-name bytes and distinct local context. Preserve mixed/foreign/wrong/Discharged/unsupported and separate scheduling-permission failures, actual false-dependent Kernel rejection and independent accepted leaf identity. Structural hash/reference mutations are not accepted-source or publication evidence; existing fixtures, expectations and tests retain intent.

## Source candidate selection

`select_source_registration_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, (LogicProfile, ConcreteFormat), &VerifierPolicy) -> Result<ProofSelection, String>` owns bounded same-source orchestration. Call check_source_registration_candidate freshly with these immutable inputs and the supplied policy; never accept a caller check tuple, accepted flag or evidence pair. Preserve the original checking and Builtin facade public behavior.
Independently re-run the existing source checker/Core/original VC/default Discharge producers on the same inputs and obtain the actual source candidate factory's encoded container and candidate. Reuse existing schemas and owner APIs, with no new hash/codec/parser or check implementation. Require the fresh Kernel target algorithm/digest to equal the actual encoded target fingerprint before selection. Caller still owns current source/snapshot freshness; this local association creates no runtime lease or reusable source receipt.
Use the actual candidate_id with CandidateSourceId::new. Only the actual current decision's KernelVerified class may use ProofEvidenceCandidate::from_trusted_kernel_input from this fresh result with AtpFormulaSubstitution origin; failure to derive trusted evidence is an error. Otherwise preserve the actual decision with ProofEvidenceCandidate::new, without a trusted marker or invented evidence hash. In either case carry the actual candidate provenance hash. No backend/profile/format priorities or witness hashes are invented.
Construct ProofEvidenceSet with the fresh target's existing TargetVcFingerprint::sort_bytes identity, the actual encoded input_hash and a clone of the supplied immutable policy; select_winner receives this one bound candidate. Existing selection owns ordering, rejection diagnostics, accepted evidence identity/polarity and unpublished reuse metadata. Binding/producer/check/scheduling errors return Err without a selection; a genuinely rejected check yields the existing non-trusted rejected selection, never sibling acceptance or an Open/Builtin fallback.
Tests compare genuine original/renamed single and three-registration sources and both encodings against independently assembled existing-selector results from actual producers. False definitions must reject their own dependent leaves while preserving independent leaf identity. Fresh-source/policy/target/format failures return no trusted selection; FormulaSubstitution and Builtin permissions stay distinct. Preserve the checking tests and original Builtin behavior.
Return only this call's existing ProofSelection, with no selected witness or committed publication. It is not transferable authority for activation, artifact projection, trusted cache reuse or current source validity. Generic by/built-in acceptance, finder/backend execution, matched witness/obligation metadata, Artifact publication and full Steps 6–8 remain deferred.

## Source discharge checking

`check_source_registration_discharge_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` freshly reconstructs the original source graph and actual default Discharge, then obtains the selected original direct-premise handoff through VC's source association. Encode normal local evidence, require existing BuiltinKernelEvidence scheduling permission, parse with the original target/profile and invoke ordinary proof-obligation checking. Classify the actual result with BuiltinDischarge origin under the supplied policy; no caller evidence/result or receipt is accepted.
Only context construction and the normal Kernel call are shared privately with the existing ATP check and original built-in activation path. Their parser/schema/instance selection, errors, scheduling, origin, policy and activation behavior remain unchanged. Unsupported or foreign inputs and denied formats fail closed; false-definition direct leaves still prove their unchanged premise/goal. General acceptance, trusted witness metadata and publication remain deferred; caller-owned source/snapshot freshness is not a reuse receipt.

## Source discharge metadata

`check_source_registration_discharge_metadata` takes the existing discharge facade's six source/snapshot/target/policy inputs and returns `(KernelCheckResult, PolicyDecision, Option<KernelAcceptanceMetadata>)` inside its existing Result. One pipeline performs fresh reconstruction, original direct-premise association, normal encoding/scheduling/parsing/checking and BuiltinDischarge evaluation; both facades use the [payload producer](#source-discharge-witness-projection), discarding unused tuple values while preserving their public signatures and prior outcomes/errors.
Produce plain metadata only for actual DischargedBuiltin, untainted Accepted ProofObligation with the existing accepted identity and AssertFalseForRefutation. Bind the same call's original handoff/check/current policy. Missing or unsupported association supplies None without fallback hashes or upgrading the returned decision; no caller evidence, result or receipt is accepted.
All hash refs use Interface. Copy the [VC component profile](../../mizar-vc/en/kernel_evidence_handoff.md#local-evidence-component-hashes) into profile/formula/substitution/provenance digests, with families `mizar-vc/local-evidence-component/{profile,formula,substitutions,provenance}` and version 1.0. Target binding copies `handoff.canonical_hash()` in `mizar-vc/kernel-evidence-handoff-canonical` 1.0; separate context identity and goal-polarity checks remain with the existing normal checker.
Policy copies `policy_fingerprint().hash()` in `mizar-proof/verifier-policy`, version `(policy.schema_version(),0)`; accepted result copies BuiltinDischarge KernelPolicyInput's `accepted_evidence_hash()` in `mizar-proof/kernel-accepted-evidence` 1.0. Checker/evidence schema versions are `(policy.checker_schema_version(),0)` and `(envelope.schema_version(),0)`. The checker value records the configured policy requirement, not authentication of the checker implementation or running toolchain version. Supported direct handoffs have real empty substitutions and no imported context, so formula_context_hash is None.
This association is raw data, not a trusted witness token, composite obligation/current-source receipt or committed reachability. Artifact's existing Builtin/version guards, source freshness ownership, staging/publication/reuse and full Proof23/Artifact25 remain unchanged or deferred.

## Source discharge selection

`select_source_registration_discharge_candidate(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, &VerifierPolicy) -> Result<ProofSelection, String>` calls check_source_registration_discharge_metadata once with those immutable inputs. Preserve the existing checker/metadata/ATP-selection/activation bodies and errors; accept no caller result/evidence/metadata and perform no second source/Core/VC/discharge/handoff/check pipeline.
Use the fresh Kernel result's original target sort_bytes as obligation identity. CandidateSourceId is `source-registration-discharge-v1:` followed by lowercase hex of those bytes, including their existing algorithm/length framing. This is a stable candidate name, not a new VC or encoded-input hash. Preserve original Open target binding rather than relabeling it for an ATP translator.
Only an actual DischargedBuiltin decision with returned metadata may use from_trusted_kernel_input on this result with BuiltinDischarge origin; missing metadata/trusted evidence is an error. Carry that actual metadata's provenance digest. Other genuine decisions retain their untrusted candidate and no invented evidence/provenance; upstream errors yield no selection. Select with this actual target, None encoded input and the supplied policy through the existing selector and [input-availability rules](selection.md#encoded-input-availability).
Return only the existing ProofSelection, preserving actual class, accepted evidence identity and polarity. Do not create deterministic_discharge_hash or witness metadata: generic Core fingerprints/discharge hashes remain unavailable, dependency slices remain incomplete, built-in publication stays unsupported and trusted cache reuse is not established. Current source/snapshot freshness remains caller-owned; no activation, lease, status projection, witness staging or artifact commit is added.
Tests use genuine original/renamed direct type leaves, single/multiple registrations, snapshots and existing policies/schema/profile variants; compare independently assembled selection from the actual fresh checker/metadata. Preserve unsupported/foreign/mixed-input failures, valid direct leaves despite unrelated false definitions, and actual false-correctness candidate production/Kernel rejection. Do not fabricate a rejected built-in check where the current source producer instead returns an error. Existing ATP selection and proof/metadata tests retain their intent.

## Source discharge witness projection

`project_source_registration_discharge_witness(&SurfaceResolvedArena, &TypedArena, &SymbolEnv, BuildSnapshotId, VcId, &VerifierPolicy) -> Result<(ProofSelection, ProofWitnessRef, Vec<u8>), String>` freshly produces only an unpublished direct-premise built-in projection.
Retain the exact local Formula v1 bytes parsed and checked in this call. A private payload checker preserves the existing source/Core/VC/discharge/handoff/check/metadata body and errors; its three callers are the compatibility check, raw metadata facade and this projection. Existing selection still calls raw metadata once; ATP checking and activation stay unchanged.
Require actual DischargedBuiltin, same-call metadata and the existing trusted Kernel input. Assemble the same original-target candidate name, provenance and None-encoding selection as the source-selection facade, without a second producer run or caller evidence. Other outcomes yield the existing invalid_source_candidate family; propagate upstream errors unchanged.

Use Artifact's CanonicalHashDomain with no excluded paths for both new producer-owned references, version 1.0. The payload reference has class Artifact and family `mizar-proof/source-registration-discharge-witness-payload`; hash the CanonicalJson string containing lowercase hex of exactly checked bytes. This is a distinct content profile, not the existing witness-store payload hash.
The obligation reference has class Interface and family `mizar-proof/source-registration-discharge-obligation`. Hash a CanonicalJson object with these exact keys:
`original_target_bytes` (lowercase hex of actual original Kernel target sort_bytes), `kernel_profile_fingerprint`, `verifier_policy_fingerprint`, `checker_schema_version`, `evidence_schema_version`, `target_binding_hash`, `formula_evidence_hash`, `substitution_evidence_hash`, `provenance_hash`, `formula_context_hash`, `accepted_result_hash`, `accepted_goal_polarity`, `witness_artifact_hash`.
Copy every metadata hash as its full artifact-hash string, both versions as their canonical strings, absent imported context as null, actual accepted polarity as its stable string, and the genuine payload hash as its full artifact-hash string. No generic VC/discharge/dependency fingerprint is substituted.

Use the selected source-target name as obligation_id; witness_path is `proof-witnesses/source-registration-discharge-v1/<lowercase-payload-digest>.bin`. Share lowercase hex only across the three actual new-producer call sites (target, payload, path), without changing the existing selector's formatting/errors. Build the [explicit built-in reference](../../mizar-artifact/en/proof_witness.md#built-in-reference-profile) from this metadata and hashes, and structurally validate it through Artifact's value writer before returning it with this selection and these bytes.
This tuple does not create TrustedKernelWitnessMetadata, a ProofWitnessDraft, file I/O, committed-publication proof, accepted witness status projection or selected witness/discharge reuse hash. Existing unsupported publication, incomplete dependency slices and conservative generic identities remain; current source/snapshot freshness remains caller-owned. Full source-produced artifact assembly, Proof witness staging/publication integration and complete Proof23/Artifact25 remain deferred.
Tests use fresh genuine single/multiple source leaves, renamed sources, snapshots/policies, exact reconstructed normal payloads and raw metadata, canonical round-trip, hash/path participants and upstream failures. Preserve real false-correctness Kernel rejection and old check/metadata/selection intent. Artifact shape mutations never count as source acceptance/publication evidence.

## Source existential-registration proof

`prove_source_existential_registration(&SurfaceResolvedArena, &TypedArena, &SymbolEnv,
BuildSnapshotId, &VerifierPolicy) -> Result<RegistrationDatabase, String>` freshly invokes
[checker](../../mizar-checker/en/registration_resolution.md#source-existential-registration-proof),
Core, VC and both source handoffs. The [VC local transport](../../mizar-vc/en/kernel_evidence_handoff.md#local-binary-transport) frames their actual fields; Proof first parses six-section
Formula v1, including full terms, substitutions, provenance, context identity and
AssertFalseForRefutation, without checking acceptance. ATP's [untrusted name writer](../../mizar-atp/en/backend.md#closed-choice-reflexivity-names) supplies Formula v2 names from that parsed input. Proof runs the parser and check_kernel_evidence
itself for both leaves; caller Accepted inputs, kernel-result summaries, cached handoffs and
diagnostic/hash text cannot substitute.

Require existing UncheckedBuiltinDischarge scheduling permission before checking, then use
KernelEvidenceOrigin::BuiltinDischarge. Both fresh Accepted, untainted ProofObligation results must
pass existing KernelPolicyInput/ProofPolicyEvaluator as genuine DischargedBuiltin. Construct
existing ActivationInput and RegistrationDatabase inside this call with the same authenticated
SymbolEnv and checker validation, binding the full pattern, correctness owner, original
parent/nonempty/leaf association, source/module/snapshot, handoff hashes and current policy. Failure
produces no activation or mutation of the pending input; no new kernel receipt or trust policy is
introduced.

The returned database is this call's module-local output, not a transferable proof receipt. Later
consumers must use this producer within the same authenticated source orchestration, never accept an
arbitrary returned/public database as proof authority. Preserve item-order visibility and no
retroactive facts. The bounded C3 widening runner may consume this fresh local output immediately in that orchestration;
the bounded C13 consumer belongs to [harness orchestration](../../mizar-test/en/harness.md#attributed-ambiguity-admission); the bounded C5 narrower-loci consumer may likewise immediately consume the single-registration output with the same immutable inputs; broader Task274, artifact publication and cluster/reduction effects remain deferred.

## Three-registration source-proof extension

The implemented additional source profile uses the unchanged prove_source_existential_registration signature and freshly reconstructs the complete checker/Core/VC graph for all three authored C13 registrations.
All six ordinary Formula-v2 leaf checks must succeed under the current policy before constructing one same-environment database containing exactly the three full patterns. Failure returns no partial activation or mutation of earlier pending inputs.
Each association binds its own registration, parent, independent nonempty seed, both leaves, actual definition dependencies, handoff hashes and source/module/snapshot. No callback, caller receipt or another registration's acceptance substitutes.
Retain the single-registration behavior and all non-transferable-database restrictions. This prerequisite provides no C13 overload/inhabitation-consumer credit, C3 widening, general Task274/import, cluster closure or publication.

## Source theorem original obligation identity

`source_theorem_obligation_identity(source: &mizar_resolve::resolved_ast::SurfaceResolvedArena, check: &mizar_checker::type_checker::SourceTheoremCheck<'_>, symbols: &mizar_resolve::env::SymbolEnv, snapshot: mizar_session::BuildSnapshotId, vc_id: VcId) -> Result<(ProofObligationIdentity, mizar_vc::dependency_slice::DependencySlice), String>` is one additive inline function in `crates/mizar-proof/src/status.rs`.
First call unchanged SourceVariableSemanticsChecker::check_theorem_skeletons(source, check.typed_ast(), check.scope(), symbols, check.labels(), check.resolved_labels(), None), preserving its exact error. None is the admitted ordinary profile, not an algorithm-backed receipt reconstruction or a new scope/label/currentness seal.
Require check.owners() == fresh.owners() as borrowed slices; mismatch returns internal `theorem.proof.invalid_source_identity`. Never clone/relabel owner facts or accept caller owner/hash/maps.
Next call unchanged [source_theorem_canonical_inputs](#source-theorem-canonical-inputs)(&fresh, snapshot, vc_id), preserving upstream errors before syntax/owner work and retaining its exact (vc, goal, context, entire original slice). Reuse its actual lower-first Core/VC pipeline and whole original Open graph None collector; no duplicated setup, schema/status/unknown/completeness change, dropped entry or manufactured discharge.
Then call unchanged [Resolver syntax producer](../../mizar-resolve/en/symbols.md#source-theorem-syntax-payloads) source_theorem_syntax_payloads_from_arena(source, symbols) on the same operands and propagate errors. Require one genuine fresh owner and exactly one syntax row keyed by its actual owner.symbol, never spelling/id fallback; require role Theorem, status Unmodified, recovery Normal and checked public/private visibility. Later mismatch uses the same internal identity error without broadening the earlier ordinary profile.
Materialize existing CanonicalJson with exactly seven STRING fields from that owner: package_id = owner.symbol.module().package().as_str(), module_path = owner.symbol.module().path().as_str(), namespace = the same module-path text established by whole-env replay, spelling = owner.spelling, role = "theorem" after role validation, status = "unmodified" after status validation, visibility = actual checked visibility. Serialize with existing canonical_json_bytes, including its trailing LF; no new owner representation/encoding helper.
IDs, FQN, contribution/site, ranges and provenance paths are association-only, excluded from owner content. Pass actual owner JSON bytes, borrowed unchanged same-source syntax and unchanged canonical triple to the [VC raw candidate function](../../mizar-vc/en/vc_ir.md#source-theorem-candidate-anchor-hash); VC owns its hashing grammar.
Construct existing ObligationAnchor from exact `source-theorem-candidate-anchor-v1:` plus existing source_discharge_hex of all 32 candidate hash bytes; this is reversible presentation, not Proof-owned anchor hashing. Preserve constructor errors as to_string.
Construct existing CanonicalJson with exactly five STRING keys: obligation_anchor = candidate string, vc_fingerprint = lowercase 32-byte hex of vc.hash(), canonical_goal_hash = lowercase 32-byte hex of goal, local_context_fingerprint = lowercase 32-byte hex of context.hash(), vc_local_dependency_slice_fingerprint = lowercase 32-byte hex of the original slice.fingerprint().hash().
Hash that object with existing CanonicalHashDomain(HashClass::Interface, "mizar-proof/source-theorem-original-obligation", SchemaVersion::new(1, 0)) and no excluded paths. Call existing ProofObligationIdentity::new(candidate.as_str().to_owned(), candidate, obligation_fingerprint, vc.hash(), context.hash(), original slice.fingerprint().hash()), preserving constructor errors as to_string; obligation_id and anchor intentionally match. Return identity and the entire original owned slice atomically without mutation.
This binds actual producers without downstream acceptance/reuse/publication authority. Source/snapshot/filesystem currentness remains caller-owned; matching foreign numeric ids attest nothing. Preserve original anchors, incomplete footprints/cache misses, acceptance origin/evidence/witness, ordinary ATP, Driver/runtime/cache and broad Steps6–8 deferrals.
Test plumbing exposes the existing source_registration_owner_inputs as pub(super) and maps dotted modules to hierarchical source paths with parent directories, preserving single-component paths and producer operands; extract only existing scope/type/label/check assembly into private with_source_theorem_check(ast, source, symbols, consume) in term_proof_tests; the original wrapper retains exact source preparation and parameters. The wrapper plus three new families use that helper; preserve other existing bytes and test intent.
Exactly three source_theorem_obligation_identity_* families cover exact_genuine_fields_and_slices; neutral_allocation_and_participating_edits; preserve_upstream_errors_and_inputs. Use genuine arbitrary-source Frontend/parser/collectors and fresh same-arena/env/check; first obtain actual old canonical inputs for VcId::new(0), snapshots 777/778. Independent pre-implementation normalized-owner/syntax/Core/VC/candidate literals and a separately written five-key object using the actual old slice fingerprint verify all six getters, exact anchor/id and full slice/entries/unknowns/status/kind/completeness/fingerprint/cache miss; repeatability, raw empty-owner/syntax errors and input/prior-result purity are required.
Neutral Unicode trivia/whitespace and real SourceId skip7 must preserve candidate while retaining the actual old full slice; never infer full identity/slice snapshot stability. Genuine package/module including dotted path, owner label, binder and proof-local spelling edits participate appropriately. Reuse Resolver private-neutrality and lower profile matrices; invent no successful private wrapper, reordered/leading-Lemma receipt or semantic-equivalence claim.
Compare new errors with actual existing checker replay for genuine foreign/augmented/recovered source/env, including absent VcId to detect order. Where ordinary Lemma/open or multiowner checks genuinely construct, compare to actual old canonical-input errors; valid source with absent selection also preserves the old error. Every asserted API executes; use no fabricated positive AST/owner/check, Accepted/Hit/Complete control or placeholder dependency.

## Source theorem reflexivity check

`check_source_theorem_reflexivity(source: &SurfaceResolvedArena, check: &SourceTheoremCheck<'_>, symbols: &SymbolEnv, snapshot: BuildSnapshotId, vc_id: VcId, policy: &VerifierPolicy) -> Result<(KernelCheckResult, PolicyDecision), String>` is one inline function in existing status.rs.
First replay ordinary SourceVariableSemanticsChecker::check_theorem_skeletons with supplied source/env, check.typed_ast()/scope()/labels()/resolved_labels() and None; preserve errors, then require borrowed check.owners() == fresh.owners(), otherwise internal theorem.proof.invalid_source_evidence.
Lower the fresh check with None, call the [raw graph API](../../mizar-vc/en/generator.md#source-theorem-vc-generation) with source-theorem-local-v1/vc-v1 and caller snapshot, then unchanged build_source_theorem_kernel_handoff on that Core/graph/selected id. Preserve original Open graph/anchors/status and VC69's whole replay/profile validation; do not call default discharge, Core56 or canonical inputs for unused metadata.
Use the actual original handoff envelope's target/profile for normal schema1 encode_local_evidence(None) and FormulaEvidenceParseContext::v1 parsing. Apply unchanged ATP43 encode_local_variable_reflexivity_names to that parsed actual variable-equality goal; encode schema2 with those names from the SAME handoff, explicitly admit schema2 in the same target/profile parse context and parse normally.
This procedure intentionally constructs deterministic local-reflexivity BuiltinDischarge evidence; origin is explicit before the result, never inferred from KernelAccepted or generic Unknown. No ATP encoded problem or encoded_problem_matches operand exists here.
Only after actual stable wire exists, require existing ProofPolicyEvaluator scheduling for UncheckedBuiltinDischarge before Kernel; refusal is internal theorem.proof.kernel_policy_rejected. Reuse unchanged check_source_kernel_handoff with the actual original handoff/parsed schema2, retaining its registration.proof.context error.
Pass actual result to KernelPolicyInput::from_kernel_result with KernelEvidenceOrigin::BuiltinDischarge and existing evaluator.evaluate_candidate; return the real result/decision, including genuine rejection. Preserve checker/lower/raw graph/VC69/encode/name String errors directly; parser errors alone map Debug as theorem.proof.parse:{error:?}; own profile/owner mismatch uses theorem.proof.invalid_source_evidence. No public diagnostic registry/emission changes.
Exactly three appended genuine-source families in existing term_proof_tests reuse source_registration_owner_inputs and with_source_theorem_check without altering old fixture/helper intent; write normally formatted code with local import aliases.
Family one covers original/renamed/trivia/allocation/module/snapshot variants, actual normal KernelAccepted and DischargedBuiltin, exact original target against independent generic graph construction with actual fixed schemas, ProofObligation kind, taint/used-axioms, purity and repeatability; preserve old canonical/identity/full-slice outcomes.
Family two exercises actual enabled/disabled/strict policy-format scheduling: genuine enabled success and disabled refusal through this API, without caller booleans or synthetic accepted inputs.
Family three preserves actual checker upstream errors before absent selection/policy for foreign/augmented/recovered source/env; compare genuine Lemma/open/multi/cited profile errors as constructible to actual lower/VC69 and retain old missing-selected-id failure. Reuse existing no-name/foreign-target/context refusals and lower/oracle matrices; add no duplicate Kernel-context wrapper or changed old expectation.
Inputs stay immutable; caller owns filesystem/source/snapshot currentness. This returns no identity metadata, witness, publication, cache/reuse authority, full acceptance gate or broad Steps6–8 closure; no fabricated DischargeOutput/Accepted/Hit/Complete, caller payload/hash/graph substitute or changes to Kernel/policy/encoder/try_discharge.
