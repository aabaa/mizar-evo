# mizar-proof Witness Store

> Canonical language: English. Japanese companion: [../ja/witness_store.md](../ja/witness_store.md).

## Purpose

`mizar-proof` owns proof witness draft staging and publication references for
selected trusted proof outcomes.

The witness store is not proof authority. It never accepts proofs, reruns the
kernel, runs ATP backends, solves SAT problems, queries caches, or writes the
artifact manifest. It records bytes and metadata for evidence that has already
been accepted by `mizar-kernel`, computes stable content hashes, and exposes a
publication reference only after the artifact publication boundary proves that
the witness is manifest-reachable.

## Inputs

The store consumes `ProofWitnessDraft` values only for trusted selections that
also match the deterministic status projection for the same obligation:

- `KernelVerified` formula/substitution kernel evidence with a matching
  accepted proof-obligation kernel result;
- `DischargedBuiltin` as an internal staged outcome; the explicit file route below carries an unpublished reference only.

The draft carries:

- `obligation_id`, `ObligationAnchor`, and `obligation_fingerprint`;
- selected winner class and selected evidence hash copied from the status
  projection and kernel metadata token;
- witness payload schema family and schema version;
- canonical witness payload bytes;
- kernel-acceptance metadata required by `ProofWitnessRef` version `2.0`;
- verifier policy fingerprint and checker/evidence schema versions;
- provenance metadata, including target binding, formula evidence,
  substitution evidence, provenance, optional formula context, accepted result,
  dependency artifact, and build snapshot fingerprints;
- optional diagnostics-owned provenance references for non-trusted attachments.

Externally attested evidence, policy assumptions, open obligations, rejected
obligations, no-selectable evidence, backend logs, backend proof methods,
resolution traces, SMT proof objects, cache records, and backend-reported axiom
lists do not produce trusted witness drafts.

## State Machine

Witness handling has three states:

1. `ProofWitnessDraft`: producer-owned bytes plus trusted metadata and matching status projection; not stored, published or cache-validating by itself.
   Metadata fields stay opaque with a crate-internal generic binder; bounded [Status token production](./status.md#source-theorem-trusted-witness-metadata-with-wire) and [Draft composition](./status.md#source-theorem-witness-draft) supply freshly checked metadata.
2. `ProofWitnessStagedRef`: generic `stage` records the draft path/hash/schema, obligation, selected class and provenance; its `KernelVerified` reference is an unpublished candidate.
   The explicit file route below instead records its validated file profile/hash and unpublished builtin reference; neither route establishes committed reachability.
3. `ProofWitnessPublishedRef`: returned by `publish_ref` only after the
   artifact publication boundary supplies a committed witness publication proof
   that matches the staged tuple and makes it reachable from the committed
   main artifact.

`stage` must happen before artifact commit so the artifact layer can write and
hash witness bytes. `publish_ref` must happen after the committed manifest
entry references the same witness path, obligation id, obligation fingerprint,
and witness artifact hash, and after the module manifest entry references the
same main `VerifiedArtifact` proof-witness set. A standalone matching witness
tuple is not enough. The artifact-owned `CommittedWitnessPublicationProof`
input must bind the witness entry to the committed module artifact entry or
equivalent committed `VerifiedArtifact` reference set, and must show that the
manifest `proof_witnesses` array exactly covers the artifact
`proof_witnesses` collection. The token stays opaque; callers cannot synthesize
matching tuples. The bounded source-theorem constructor below uses the existing
opaque Artifact commit as origin evidence, rather than accepting public tuples.
Calling `publish_ref` without committed publication evidence is an error.

`CommittedWitnessPublicationProof::from_source_theorem_manifest_commit` takes an
actual `ManifestCommit`, root, full module identity and coordinator-expected snapshot.
Fresh all-flags manifest validation must match the commit hash and whole manifest;
normal1.1 main validation and exact witness coverage precede concrete source-theorem
candidate FILE hash/reference validation. Require a nonempty KernelVerified set.
The expected snapshot is compared with staged provenance by unchanged `publish_ref`;
it is not a commit snapshot receipt, current-root/source authentication or lease.
The token retains past validated reachability: fresh construction rechecks files,
while later mutation does not retroactively alter an existing token or certify reuse.
Normal main-hash exclusions and identical copied roots remain valid; no exclusive filesystem origin is authenticated.
Other payload profiles and DischargedBuiltin publication remain unsupported.

`publish_ref` does not invent a new artifact reference after commit. It
validates that the unpublished reference candidate emitted by `stage` is the
same reference recorded in the committed `VerifiedArtifact` and manifest, then
returns a published wrapper for that same reference.

## Stable Hashing

For generic `stage`, the proof witness hash is the artifact-framed hash of the exact staged payload
bytes and payload schema identity:

- schema family;
- schema version;
- canonical payload bytes;
- witness payload hash domain;
- selected accepted evidence hash;
- verifier policy fingerprint;
- obligation fingerprint.

The hash must not include temporary file paths, staging directory names,
arrival order, backend completion time, process id, wall-clock time, random
data, cache-hit metadata, or manifest commit timing.

If two drafts have identical payload bytes and identical hash inputs, they
produce the same witness hash. If any trusted input changes, including accepted
evidence hash, policy fingerprint, obligation fingerprint, payload schema
version, or payload bytes, the witness hash changes or staging is rejected.

### Payload hash value

`ProofWitnessDraft::payload_hash_value(&self) -> CanonicalJson` returns an owned copy of the existing seven-key generic OBJECT through the unchanged hash encoder: hash_domain, obligation_fingerprint, payload_bytes_hex, payload_schema_family, payload_schema_version, selected_evidence_hash and verifier_policy_fingerprint. It preserves exact existing encodings/domains; caller edits do not mutate the draft. No I/O, validation or new producer/schema is added. This hash input lacks the top-level schema_version required for [published schemas](../../mizar-artifact/en/store.md#schema-versions); actual files remain [unreferenced and unpublished](../../mizar-artifact/en/store.md#store-layout). This legacy profile remains headerless; the [separate file schema](#source-theorem-witness-file) does not change it. Never add a header/exclusion here, substitute raw STRING/composite profiles, or change staging, Builtin/publication guards, acceptance or reuse credit.

Extend only existing source_theorem_reflexivity_witness_draft_matches_captured_inputs after its original pure checks; preserve every old line/helper/fixture. Genuine native Disk/OpenBuffer capture supplies the draft. Independently assemble the whole seven-key expected OBJECT from the genuine draft/wire; compare the new value, existing Artifact writer/readback whole value/full canonical bytes, writer/read/stage digest and family/version/domain, replay and draft purity. Use existing temporary root's unpublished-witness-hash-values subroot with the unchanged draft path so old no-file assertions still hold. Builtin staged reference stays None; never forge committed metadata or add a helper/lower I/O matrix. No publication, currentness, Hit, CompleteSlice or broader gate credit follows.

## Source algorithm state witness file
The Status [producer](./status.md#optional-bounded-source-algorithm-state-witness-file) supplies FILE family mizar-proof/source-algorithm-state-witness-file1.0: an OBJECT with EXACTLY schema_version, schema_family, obligation_id, obligation_fingerprint, payload_schema_family, payload_schema_version, payload_bytes_hex, kernel_acceptance. All except kernel_acceptance are strings. Hash the WHOLE canonical OBJECT under its Artifact1.0 domain with NO exclusions; never relabel the separate generic seven-key Draft hash. Shared crate-private SOURCE_ALGORITHM_STATE_WITNESS_FILE_SCHEMA_FAMILY serves producer identity/hash, reader compatibility and reader hashing; only the producer's local family binding changes to this constant, with every other old producer byte/output preserved.
Identity uses the original id and Interface/mizar-proof/source-algorithm-state-original-obligation1.0 fingerprint. The ONLY payload profile is mizar-proof/source-algorithm-state-transitivity-checked-wire2.0, represented by nonempty even-length lowercase ASCII hex of the ENTIRE payload. kernel_acceptance is the full ten-field [Artifact metadata](../../mizar-artifact/en/proof_witness.md#conceptual-shape), including required nullable formula_context_hash, with evidence_schema_version2.0. Reference is explicit Ref3.0/DischargedBuiltin/FormulaSubstitutionKernelEvidence using actual path/identity/full metadata and the FILE Artifact hash.
`read_source_algorithm_state_witness_file(&PublishedArtifactRead, &ProofWitnessRef) -> Result<Vec<u8>, String>` is one inline structural reader before the COMPLETE theorem-reader doccomment. Require OBJECT; FIRST check path-aware SchemaVersionSupport(family,1,0,UpToSupported), mapping missing/non-string headers as the old theorem reader does; then exact family, exact key set/types, exact payload pair, and safe full hex recovery. Reject empty/odd/uppercase/nonhex/non-ASCII BEFORE byte-offset slicing. No migration or forward-minor extension is declared.
Recompute WHOLE state FILE hash/no exclusions and require read.artifact_hash=Some(computed), since PublishedArtifactRead is caller-copyable. Reconstruct untrusted reference JSON with FIXED3.0/discharged_builtin/formula_substitution_kernel_evidence, actual FILE id/fingerprint/metadata/read.path and computed typed hash. Call unchanged read_builtin_proof_witness_ref with actual path and expected.witness_artifact_hash, then require WHOLE decoded reference=expected, exact original Interface/state family1.0 and evidence2.0. Return only ENTIRE decoded bytes; preserve lower errors as Strings. Do not infer class from expected or hand-parse metadata.
Own errors are source algorithm state witness file must be an object; invalid source algorithm state witness file fields; source algorithm state witness file {key} must be a string; source algorithm state witness file kernel_acceptance must be an object; invalid source algorithm state witness file payload profile; invalid source algorithm state witness file payload hex; source algorithm state witness file read hash mismatch; source algorithm state witness file reference mismatch. Exact-family incompatibility follows the old theorem format naming family, actual version, supported range and artifact path. Preserve this order and all lower schema/codec Strings.
This reader validates UNTRUSTED structural association, never Kernel evidence, source acceptance, snapshot/filesystem currentness or publication. Coherent edits/relabels with matching profile fields/metadata/hash/reference MAY recover arbitrary bytes; do not restrict arbitrary metadata digests/checker versions/context beyond the existing codec and profile guards to invent authenticity. Only the separate [state staging association](#source-algorithm-state-file-staging) compares an actual supplied Draft; reader success alone grants no such association. Enclosing Artifact/commit/token/publication/reuse/Hit/Driver/registered/SAME/general finder/Steps6–8 stay deferred; theorem reader/stager, generic stage refNone, builtin publish refusal and raw Open/Deferred/incomplete/MISS remain unchanged.
### Source algorithm state witness file reader tests
Append ONLY source_algorithm_state_witness_file_reader_matches_genuine_transport and source_algorithm_state_witness_file_reader_rejects_profiles_and_associations before final term_proof_tests brace, using unchanged native Disk helper. BEFORE reader Rust obtain genuine72 whole triple/file/ref, real canonical Artifact transport complete bytes/LF/domain/digest and independent untrusted RefJSON from FILE fields through the old Builtin reader. Expected ENTIRE wire bytes come directly from the genuine Draft; inherit72 independent FILE hex encoding rather than copying the reader hex decoder into tests. The mandatory final existing72 regression families retain actual71 whole-tuple association, full FILE/metadata serialization, theorem refusal/generic-stage refNone/raw/publication controls; inherit71 semantic/Kernel matrices. New families use compact genuine source/path/policy variants and reader/read-reference repeat/input purity; no new module helper or fabricated check/snapshot/Accepted/token. The negative family may use ONE local I/O-only closure at THREE or more distinct actual call sites to share existing Artifact write/read; it performs no decoding, profile checks, hash reinterpretation or error-oracle logic.
Negative data starts from actual72 products and uses real canonical transport; omit expected read digest or coherently update ordinary FILE/reference hash only to reach deeper guards. Cover compatibility-before-bad-content against actual SchemaVersionSupport; OBJECT/family/missing/unknown/types/profile/empty/odd/uppercase/nonhex/non-ASCII hex; representative nested metadata lower refusals; read hash None/wrong domain/post-read value edits; foreign safe/non-proof paths; full reference/id/policy/metadata/path/hash/class/version/kind mismatches; coherently matched wrong original family/version/evidence version with successful lower controls before profile refusal. Inherit Artifact exhaustive malformed metadata/codec/I/O and72 optional-source errors; add no copied reader adapter.
Phase1 executes existing lower baselines only; phase2 after actual baseline green adds new reader byte/error assertions from this written profile/error contract. Explicit POSITIVE STRUCTURAL LIMITATION: coherently rewrite/re-hash a genuine untrusted FILE/ref with safe payload hex00 using real transport; reader returns [0] while actual original Draft/wire remain unchanged and different. This is untrusted-data recovery, never a new Accepted result. Final focused checks also run existing72 families to prove the shared family binding preserves whole producer outputs.

## Source theorem witness file
The source Status [producer](./status.md#source-theorem-witness-file-candidate) supplies a separate top-level OBJECT with exactly eight keys: schema_version, schema_family, obligation_id, obligation_fingerprint, payload_schema_family, payload_schema_version, payload_bytes_hex, kernel_acceptance. All except kernel_acceptance are strings. The file family is `mizar-proof/source-theorem-witness-file`, version1.0, Artifact hash class, whole-object canonical hashing with no exclusions. One crate-private SOURCE_THEOREM_WITNESS_FILE_SCHEMA_FAMILY constant is shared by producer hashing, reader compatibility and reader hashing; no public schema type or helper is added.
Identity is the actual original obligation id and Interface hash family `mizar-proof/source-theorem-original-obligation`1.0. Payload is SAME-call nonempty lowercase even-length ASCII hex in EXACTLY two profiles: `mizar-proof/source-theorem-reflexivity-checked-wire`2.0 for Builtin/ref3, or `mizar-proof/source-theorem-candidate-checked-wire`2.0 for KernelVerified/normalref2; the new [candidate producer](./status.md#source-theorem-candidate-witness-file) preserves the existing eight-key grammar/file1.0. Other profile/version pairs are refused. kernel_acceptance is exactly the existing [Artifact metadata object](../../mizar-artifact/en/proof_witness.md#conceptual-shape), including explicit null for absent formula_context_hash; its evidence_schema_version is2.0. Do not reuse/reinterpret legacy seven-key hashes, raw STRING/composite identities or their references.
`read_source_theorem_witness_file(&PublishedArtifactRead, &ProofWitnessRef) -> Result<Vec<u8>, String>` validates untrusted structural data only. Require an OBJECT; check schema_version first with path-aware SchemaVersionSupport(family,1,0,UpToSupported), then exact schema_family (an incompatibility reports family, actual version, supported range and path), exact key set/field types, the exact admitted payload-family/version pair and safe hex recovery. No migration or forward-minor extension is declared. Reject non-ASCII before byte slicing; reject empty/odd/uppercase/nonhex payloads.
Recompute the whole canonical file hash in its own domain and require read.artifact_hash=Some(recomputed), rather than trusting a caller-constructible PublishedArtifactRead. Derive reference class/version ONLY from the admitted explicit file profile: reflexivity2.0 selects Builtin/ref3/read_builtin_proof_witness_ref, candidate2.0 selects KernelVerified/ref2/normal read_proof_witness_ref. Assemble untrusted JSON from actual file identity/metadata/read.path/recomputed typed hash and selected class/version; use the corresponding unchanged Artifact reader with actual path and expected.witness_artifact_hash, then require the entire decoded reference equals expected and the original fingerprint/evidence versions match this profile. Return only recovered wire bytes; propagate lower errors as strings. The reader neither parses/replays kernel evidence nor creates acceptance or a committed publication token.
Artifact retains canonical parsing, duplicate-key/UTF-8/path/I/O integrity and reference/metadata validation. Existing Builtin stage/publication guards, status/cache gaps and incomplete dependency slices stay unchanged; the new file/reference is unreferenced until a separate committed publication owner exists.
### Source theorem witness file tests
Append only source_theorem_witness_file_matches_genuine_artifact_transport and source_theorem_witness_file_rejects_invalid_schema_and_associations in existing term_proof_tests. Preserve old bytes/imports/helpers/fixtures. Use actual native Disk/OpenBuffer capture, Frontend and original resolver inputs. Before each new success independently call unchanged original-source draft acquisition; compare its whole triple and independently assembled eight-key file/full metadata/hash/ref with the new products. Check actual Artifact write/read whole canonical bytes, hash/domain, recovered wire, replay, received-input purity and real source/policy variants; keep actual original Open/incomplete/miss/publication-gap/stageNone controls.
Negative file data starts from genuine products and uses actual Artifact transport. Cover compatibility before malformed content, wrong family, unknown/missing/wrong fields, malformed hex, nested metadata, foreign identity/policy/path/reference/hash and absent/wrong read hash. Generic transport may omit expected hash to reach schema errors; that is not proof authority. Compare genuine upstream producer errors/precedence with unchanged acquisition; do not duplicate lower parser/I/O/kernel matrices or construct fake positive Accepted/token/receipt/complete slices. Defensive unconstructible guards receive static review.

## Source algorithm state file staging
`ProofWitnessDraft::stage_source_algorithm_state_file(self, read: &PublishedArtifactRead, expected: &ProofWitnessRef) -> Result<ProofWitnessStagedRef, String>` is one inline consuming method before the COMPLETE old theorem-stager doccomment. Accept no supplied replacement wire/hash/metadata/token; preserve every other old production/test/helper byte.
FIRST call unchanged read_source_algorithm_state_witness_file(read, expected), preserving its exact String before inspecting Draft mismatch. Reader success already proves expected is fixed Builtin Ref3, including proof_status DischargedBuiltin through lower validation and WHOLE equality; delegate that postcondition. Require EXACT self pair (selected_class,payload family)=(DischargedBuiltin,mizar-proof/source-algorithm-state-transitivity-checked-wire), without wildcard or theorem alternative. The unchanged two-profile theorem stager retains its separate class/profile/status coupling.
Require exact obligation id, WHOLE typed obligation fingerprint, witness path, WHOLE kernel_acceptance, payload version2.0/canonical_bytes_required=true, recovered ENTIRE bytes=self.payload_bytes and expected_selected_proof_witness_hash=None. Any own mismatch is source algorithm state witness file staging mismatch. Do not invent an anchor field/hash or parse/replay evidence/metadata; actual anchor and checked-wire authority remain producer-owned.
Then call unchanged stage(self) ONCE, mapping its error to String. Change ONLY staged.payload_schema to shared state FILE family1.0/canonical=true, staged.staged_payload_hash to expected.witness_artifact_hash.clone() and staged.artifact_reference_candidate to Some(expected.clone()); retain every other generic staged field/WHOLE provenance by move. Generic seven-key hash/profile and canonical FILE hash/profile stay distinct, with no source projection or selected-witness-hash mutation.
This is an unpublished structural association with a supplied Draft, not authentication that arbitrary public Draft payloads were Kernel-accepted. It performs no Source/Kernel replay, I/O, source/snapshot/filesystem currentness, enclosing Artifact/manifest/commit/token/publication/cache Hit/Driver/registered/SAME/general finder/Steps6–8 work. Class stays Builtin; unconditional publish_ref refusal BEFORE committed-proof validation and private committed-token construction stay exact. Raw incomplete/MISS/publication gaps remain inherited.
### Source algorithm state file staging tests
Append ONLY source_algorithm_state_witness_file_staging_matches_genuine_association and source_algorithm_state_witness_file_staging_preserves_reader_errors_and_rejects_mismatch before final term_proof_tests brace. Use unchanged native Disk state helper, actual72 products, actual73 reader success after real canonical Artifact write/read, and actual generic stage baseline BEFORE new method Rust. Compare all retained staged getters/WHOLE provenance and exactly the three changed schema/hash/WHOLE reference fields; use compact genuine source/policy/path variants, new-method repeat and read/reference/product purity. ONE negative-family local I/O-only closure may share actual write/read at THREE or more distinct sites, with no decoder/profile/hash reinterpretation/error oracle; no new module helper/type/fixture/fake Accepted/check/snapshot/token.
For genuine source/policy/path cross-association show actual reader success and relevant real differences FIRST, then exact own mismatch; use actual values without cross-snapshot hash-stability assumptions. Coherent ordinary wirehex00/id/fingerprint/policy-or-accepted-metadata edits use real transport/re-hash/matching reference; prove reader success (entire original bytes or [0]) then genuine checked-Draft association refusal. Representative bad read hash/schema plus bad content/expected reference or path combine with an actually foreign Draft to prove reader-FIRST and exact lower String preservation; inherit exhaustive73 reader matrices.
Use a genuine foreign theorem Builtin Draft from unchanged with_source_theorem_file_staging_inputs, with state reader success before own refusal; unchanged theorem stager still rejects actual state FILE. Genuine KernelVerified Drafts are constructible through existing candidate factories, but no fresh native candidate capture belongs to this slice: inherit their actual class/profile matrices and independently review the exact self pair plus delegated reader postcondition statically. The foreign Builtin case does NOT exercise a different selected class; do not label KernelVerified unconstructible or fabricate an accepted state Draft from an unrelated token/snapshot.
Admitted genuine state factories expose only version2.0/canonical=true/selectedHashNone and no setters. Other generic values are not globally impossible; review exact defensive predicates and constructors statically without injecting private fields or reconstructing purported accepted-state payloads. Private committed-token construction is unavailable to this external harness; reuse existing private unit publication guards. Final broad source_algorithm_state_witness_file_ focused checks MUST run existing72two+73two+new stagingtwo, inheriting producer/Kernel/serializer/reader/raw/generic-stage/theorem/publication controls.

## Explicit source theorem file staging
`ProofWitnessDraft::stage_source_theorem_file(self, read: &PublishedArtifactRead, expected: &ProofWitnessRef) -> Result<ProofWitnessStagedRef, String>` consumes an existing draft and structurally associates its bytes/metadata with the separate file profile; add the inline method immediately before the draft's first getter.
FIRST call read_source_theorem_witness_file(read, expected), preserving its error String. Then require EXACT triple (actual draft class, actual draft payload family, validated expected.proof_status) either (DischargedBuiltin, reflexivity-checked-wire, DischargedBuiltin) OR (KernelVerified, candidate-checked-wire, KernelVerified), with the complete mizar-proof/source-theorem- family literals above. Preserve matching obligation id/fingerprint, witness path/WHOLE kernel_acceptance, payload version2.0/canonical_bytes_required=true, recovered bytes equal the ENTIRE draft payload, and absent expected_selected_proof_witness_hash. The validated reference status is necessary: independently admitting each draft/profile and each file/profile would permit coherent cross-profile relabeling. Any mismatch returns internal source theorem witness file staging mismatch.
Call unchanged stage(self), preserving its error String. Replace only its payload_schema with SOURCE_THEOREM_WITNESS_FILE_SCHEMA_FAMILY1.0/canonical=true, staged_payload_hash with expected.witness_artifact_hash, and artifact_reference_candidate with Some(expected.clone()); preserve all other staged fields. The old generic hash/wire profile and this canonical-file hash/profile are distinct; do not rehash, reinterpret or overwrite the source draft.
This is an unpublished structural association, not authentication that arbitrary caller-supplied draft bytes were Kernel-checked. Actual source producers retain checked-wire authority; this method does not parse/replay evidence, call Kernel, do I/O or attest filesystem/snapshot currentness. Preserve generic stage, builtin publish_ref refusal, status/reuse gaps and whole raw incomplete/MISS boundaries; no committed token, witness publication, acceptance or enclosing gate credit.
### Explicit source theorem file staging tests
Append only source_theorem_file_staging_matches_genuine_file_and_legacy_boundaries, source_theorem_file_staging_rejects_genuine_cross_associations and source_theorem_file_staging_preserves_reader_refusals_and_inputs plus one private with_source_theorem_file_staging_inputs helper used by all three real families; preserve every old byte/helper/fixture.
Before new calls obtain genuine native Disk/OpenBuffer capture, original unaugmented Resolver inputs, existing whole source file-candidate tuple, generic stage and actual Artifact write/read/recovery. Compare every retained old staged field, exact new file schema/hash/reference, replay/purity and unchanged source status/full slice/publication/MISS; use actual old values without a new hash grammar or cross-edit stability assumption.
Use genuine independent source/policy/path products to establish matching-reader success before cross-association refusal. Starting from genuine files, also test coherently rehashed untrusted wire/identity/metadata edits and actual lower-reader failures; compare lower Strings before the new call and establish the exercised mismatch. No fake positive draft/check/Core/VC/Accepted/token/commit or duplicate lower matrices; static review covers unconstructible trusted-class/selected-hash guards.

Candidate file tests and coherent opposite-profile structural-read-success/Draft-stage-refusal controls are owned by [candidate file tests](./status.md#source-theorem-candidate-witness-file). Existing Builtin positives/errors remain exact; no new reader/helper/type/default, grammar, acceptance, actual commit or token is added.

## Publication References

For `KernelVerified` formula/substitution evidence, `stage` prepares an
unpublished `ProofWitnessRef` version `2.0` candidate with:

- `proof_status = "kernel_verified"`;
- `evidence_kind = "formula_substitution_kernel_evidence"`;
- the staged witness path and witness artifact hash;
- kernel acceptance metadata copied from the accepted proof-obligation kernel
  evidence boundary.

Successful `publish_ref` returns that same reference as publication-reachable
only after committed manifest reachability is proven.

The store must not publish a `ProofWitnessRef` for non-trusted statuses. It must
also not rewrite unsupported trusted statuses into `kernel_verified`.

`DischargedBuiltin` publication remains an `external_dependency_gap`: `publish_ref` refuses it until real Artifact-owned committed reachability is supplied.
Generic `stage` retains its internal hash and absent reference. The explicit file route may carry an unpublished reference3.0 to its separate file hash.
Neither path supplies `selected_proof_witness_hash`, replaces the required deterministic-discharge reuse input, upgrades status or establishes publication/cache authority.

## Provenance Metadata

Staged and published records preserve provenance needed for diagnostics and
reuse validation:

- build snapshot and producer identity;
- selected candidate id and selected winner class;
- kernel evidence origin;
- target VC fingerprint and obligation fingerprint;
- dependency slice and dependency artifact fingerprints;
- verifier policy fingerprint;
- checker and evidence schema versions;
- accepted result hash and trusted used-axiom reference hash when available;
- diagnostics-owned references for advisory backend data.

Provenance metadata must agree with the selected status projection, kernel
acceptance metadata, and trusted used-axiom token when those trusted values are
present. The dependency artifact fingerprint is preserved as producer-owned
reuse metadata in task 11; authoritative binding to committed dependency
artifacts is deferred to artifact/cache integration. Provenance metadata does
not expand the trust boundary. Backend logs, externally attested citations,
cache records, and diagnostic hints remain diagnostic or reuse-validation
material only.

## Failure Semantics

The store rejects or reports a gap for:

- unsupported witness class or evidence kind;
- missing or mismatched accepted evidence hash;
- draft selected class, selected evidence hash, obligation identity, policy
  fingerprint, selected candidate id, kernel evidence origin, schema version,
  or trusted `used_axioms` reference that does not match status projection and
  kernel acceptance metadata;
- malformed payload schema identity;
- empty payload bytes when the payload schema requires canonical bytes; byte-
  level canonicality validation is deferred to concrete producer-owned schema
  validators;
- witness path escaping `proof-witnesses/`;
- hash mismatch between staged bytes and manifest reference;
- duplicate manifest reference for one obligation;
- `publish_ref` before a matching committed witness publication proof exists;
- manifest witness entries that are not bound to the committed main artifact or
  do not exactly cover the `VerifiedArtifact.proof_witnesses` collection;
- stale build snapshot or mismatched obligation fingerprint;
- attempted publication of externally attested, assumed, open, rejected, or
  no-selectable evidence as trusted witness material.

Failures are deterministic diagnostics or typed store errors. They never become
trusted proof status and never synthesize trusted `used_axioms`.

## Cache And Reuse Boundary

For publishable `KernelVerified` witnesses, selected witness hashes and
published witness refs participate in proof-reuse validation, but they are not
proof authority. A cache record may reuse a proof only when the witness hash,
selected evidence hash, obligation fingerprint, policy fingerprint, schema
versions, and accepted proof-obligation kernel metadata all match the current
validation predicate. Dependency artifact fingerprints are part of that
predicate only after downstream artifact/cache owners supply an authoritative
binding; task 11 preserves them as producer-owned reuse metadata. A selected
candidate may carry
`selected_proof_witness_hash` when it has a witness payload artifact hash
(`witness_artifact_hash`) that the current artifact witness schema can
reference. That hash is not a hash of the `ProofWitnessRef` metadata object and
is not a committed publication proof. A committed `ProofWitnessPublishedRef`
still comes only from `publish_ref` after the artifact boundary supplies a
matching committed witness publication proof. Until artifact support exists,
`DischargedBuiltin` reuse continues to use `deterministic_discharge_hash`; any
internal staged hash is not a selected proof witness hash. Cache hits cannot
publish witnesses, upgrade non-trusted statuses, or create trusted
`used_axioms`.

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `WITNESS10-G001` | `external_dependency_gap` | `mizar-artifact` `ProofWitnessRef` version `2.0` currently accepts only `kernel_verified` formula/substitution evidence. `DischargedBuiltin` witness publication remains unsupported and must not be collapsed. |
| `WITNESS10-G002` | `deferred` | The bounded source-theorem candidate route in [State Machine](#state-machine) supplies exact committed coverage; general publication integration remains deferred. This module does not write manifests. |
| `WITNESS10-G003` | `deferred` | Concrete payload schemas beyond formula/substitution witness bytes remain producer-owned. The store hashes schema identity and bytes but does not interpret backend proof payloads. |
| `WITNESS11-G001` | `external_dependency_gap` | The opaque Artifact commit enables only the [bounded source-theorem constructor](#state-machine). Other production token profiles remain unavailable; matching public tuples never establish commit origin. |
| `WITNESS11-G002` | `external_dependency_gap` | Generic caller-metadata admission remains unavailable; [State Machine](#state-machine) owns the bounded checked-token routes. |
| `WITNESS11-G003` | `deferred` | Task 11 records whether a payload schema requires canonical bytes and rejects empty payloads under that attestation, but byte-level canonicality remains producer-owned until concrete payload schemas expose validators. The witness hash still covers exact bytes and schema identity. |
| `WITNESS11-G004` | `deferred` | Task 11 preserves `dependency_artifact_fingerprint` as producer-owned provenance. Binding it to committed dependency artifacts and cache validation remains deferred to artifact/cache integration; it must not promote witness status by itself. |

## Public Enum Policy

Task 14 applies the public-enum forward-compatibility procedure to this
module. The public witness-store error enum is a downstream-facing API surface
and must remain `#[non_exhaustive]`; downstream consumers must keep wildcard
match arms. New error variants require paired spec/test review and must not
weaken the opaque publication-token or trusted-metadata boundaries.

| Enum | Compatibility decision |
|---|---|
| `ProofWitnessStoreError` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Non-Goals

The witness store does not run proof search, perform premise selection, invent
substitutions, call ATP or SAT backends, call the kernel, query caches, write
artifact manifests, or publish placeholder witness references.
