# Task TEST-METAMORPHIC-LABEL-CONFINEMENT: structurally select proof-label confinement
Canonical language: English; [Japanese pointer](../ja/TEST-METAMORPHIC-LABEL-CONFINEMENT.md).
Status: complete. Tier: full. Dependencies: completed metamorphic guard and existing resolver source/label APIs.
Promote the existing lexer dependency for genuine full-source preprocessing; no version/lock or lexer behavior change.
Owner: [test plan](../../mizar-test/en/00.crate_plan.md); bounded slice of TEST-METAMORPHIC-BRIDGE-GENERALIZATION.
Authority: specification §§2.2/2.9, 15.10 and 16.4.2; unchanged B5C `.miz` cases.
Owner boundary: [harness](../../mizar-test/en/harness.md#proof-label-confinement-generalization).
Gap: source_drift; exact-byte dispatch bypasses genuine confinement checking after layout edits.
Readiness: existing SurfaceResolvedArena, ProofLabelSourceCollector and LabelResolver suffice.

## Scope

Replace production exact-fixture dispatch only for the two existing inner-to-outer
and sibling proof-label confinement topologies. Select from matching normal source
and AST structure, never case id, expected outcome/key, owner spelling, literal
source bytes, offsets, dense AST hash, or copied producer results.
Keep the bounded no-import, one-theorem-owner, one-local-proof-label and one-simple
unqualified-citation shape; derive names, ranges, paths and ordinals from actual
source/AST and authenticated local environment/contribution.
Require consistent source identity and source-byte token/range ownership, normal
recovery state and valid structural membership. Refuse mismatched/foreign inputs.
Trailing trivia beyond token extents remains admissible; ASTs do not attest exact raw-file identity.
Use the genuine source collector and LabelResolver. Report existing confinement
key only when the actual reference matches that known local label, its scope is
an enclosing or sibling scope outside declaration visibility, and resolution is
Unresolved with matching actual origin/site. Other structural or producer failures
must retain fail-closed input refusal or the pre-existing unaffected route.

## Tests and boundaries

Preserve original corpus/expectations, snapshot identities, admission, trace status,
public diagnostics and acceptance policy. Preserve historical exact-fixture/hash
and mutation tests under cfg(test); they do not select the new production route.
Add matching fresh-source/AST layouts and source-local renames, both topology
checks, source/AST mismatch, recovered/foreign identity/contribution and malformed
structure controls, plus visible-label and unknown-name controls against false
confinement. Test actual production dispatch independently of historical fixtures.
Remove only the four owned layout exception rows after genuine variants pass.
Run focused declaration-symbol tests/corpus and complete metamorphic matrix,
format/Clippy, applicable metadata/link/doc checks and protocol verification cadence.
Exit: original expectations and all four variants pass with no named exception;
independent specification, test, code, volume and consistency reviews have no findings.
