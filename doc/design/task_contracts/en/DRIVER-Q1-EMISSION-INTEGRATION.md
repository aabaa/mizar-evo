# Task DRIVER-Q1-EMISSION-INTEGRATION: retained comment envelope emission
Canonical language: English; [Japanese pointer](../ja/DRIVER-Q1-EMISSION-INTEGRATION.md).
Status: frozen implementation scope. Tier: full (new Driver emission surface).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md); dependencies: [retention](DRIVER-Q1-COMMENT-RETENTION.md), [Artifact integration](ARTIFACT-Q1-MANIFEST-INTEGRATION.md); consumer: Doc, deferred.
Purpose: emit the exact retained Q1 envelope through the existing atomic store and manifest transaction.

## Authority And Scope
Authority: spec [2.9](../../../spec/en/02.lexical_structure.md#29-comments-and-annotations), [24.1](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax), approved option 1 [Q1 invariants](../../mizar-artifact/en/comment_projection.md), [retention boundary](../../mizar-driver/en/request.md#commented-draft-capture-and-retention), [store](../../mizar-artifact/en/store.md), [manifest profiles/transactions](../../mizar-artifact/en/manifest.md#declared-enclosing-profiles).
Gap: retained genuine drafts and Artifact envelope transactions exist, but Driver has no direct retained-draft emission handoff. This bounded source/design gap does not change language intent or publication eligibility.
Write scope: this pair; Driver request source and focused request tests; paired Driver request/plan links and minimal Artifact projection handoff-status sentences. Lower-owner source, protocol, spec, expectations and legacy theorem emission remain unchanged.
No source rereading/reparsing/reprojection, caller-supplied replacement draft, unavailable fallback, target inference, attachment/Markdown/tags, Q2–Q4 choices, diagnostics/recovery, proof/reuse authority, scheduler activation, event/token invention, audit/trace credit or milestone closure. Doc validated-publication consumption and Doc7/8/Artifact17 remain open.

## Emission Contract
Add explicit BuildSession::emit_commented_module(publisher, Session SnapshotRegistry, SourceId, artifact_root, main_path) returning an optional actual ManifestCommit or error. Missing retained entry returns None after existing currentness checks. Consume only the exact immutable retained CommentedArtifactDraft; preserve its handles, gaps and retention. Existing capture and legacy emission APIs remain unchanged.
Reuse commented_module validation before any writes and around store/readback/commit. Additionally require every retained handle to satisfy existing CurrentPackage authorization; OpenBuffer/InternalOnly cannot publish. Freshness callback repeats those checks against the original snapshot guard. These are existing point checks, not a new lease or atomic runtime-currentness guarantee.
Resolve an existing artifact root strictly beneath the captured source loader's canonical package root; derive its portable package-relative label. Require a fresh validated .mizir.json main path, refusing preexisting files without overwriting. No source-file I/O, replacement SourceId or serialized runtime handles.
Serialize and read back through the validating commented codec with its declared family/version/domain/exclusions and outer interface/implementation hashes; compare the complete decoded envelope to the retained value. Never select normal/Builtin readers or downgrade after failure.
Derive manifest package/module/source/provenance from the retained nested artifact and root label. Begin the existing ManifestTransaction before the main write, preserving base-conflict protection. Stage the actual main path/store hash and outer Artifact/Interface/Implementation references; copy complete witness references, leave optional sidecars absent. Existing transaction validates all references and witness files. It owns atomic publication and eligibility. No proof-witness generation or new proof-publication authority.
Return only the real commit after validating manifest readback and currentness. Failed checks expose no successful result; late failures may leave orphan files or an already committed manifest under existing lower-owner point-check semantics. Precommit failures preserve the prior manifest and its referenced main; never delete or rewrite unrelated files.

## Test Matrix And Exit
Six Rust cases (cap six), extending genuine native capture/Frontend/IR retention fixtures:
1. Disk retained draft emits through store/manifest; complete decoded envelope, outer hash classes/domain, exact text/maps/targets and manifest bindings match; source edit/removal and cloned session require no source reload.
2. Available-empty and empty doc body remain distinct; equivalent captures emit deterministic bytes/hashes; comment-only edits preserve envelope interface while changing implementation/full identity.
3. Missing entry returns None; OpenBuffer/InternalOnly refuses before writes; legacy projection/emission behavior remains covered unchanged.
4. Cancelled or independently obsolete Session/publisher, unreadable Frontend and collected/generation-mismatched required handles refuse with prior manifest/main intact.
5. Invalid/escaping roots or main paths, preexisting main, conflicting seed/base and missing or tampered witness references refuse without successful publication; orphan main files grant no publication.
6. Fresh validating manifest reads reject corrupted envelope/reference hashes; retained draft remains unchanged and later failed emissions preserve existing manifest/main bytes.
Independent specification, sufficiency, precision, volume/scope and final consistency reviews; full task-scoped gates and quality threshold. No spec-coverage status/owner changes.
Freeze: both lint_policy suites including links/fragments. Implementation: fmt, Clippy, both lint_policy suites, Driver owner suite, explicit emission regressions, existing Driver retention/IR Q1/Artifact Q1 regressions and short proof-verification end-to-end; CARGO_BUILD_JOBS=4. Unchanged resolver/evidence/cache heavy obligations defer to cadence; full cargo test only on a protocol trigger.
Exit: retained-draft Driver emission committed locally; Doc consumption and wider publication/semantic milestones stay open.
