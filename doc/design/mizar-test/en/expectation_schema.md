# Module: expectation_schema

> Canonical language: English. Japanese companion: [../ja/expectation_schema.md](../ja/expectation_schema.md).

## Purpose

This module defines the `.expect.toml` sidecar schema used by `mizar-test`.

Expectation files are the authoritative contract for committed tests. They are
parsed before compiler execution and must not depend on the `.miz` frontend.

## Design Decision

Every executable corpus item has exactly one expectation sidecar.

The sidecar owns:

- test identity;
- staged model placement;
- spec traceability back-references;
- expected outcome;
- expected failure identity;
- deterministic diagnostics and snapshot requirements.

The source file owns only the input program or fixture payload.

## File Pairing

Expectation sidecars use the same stem as the input file:

```text
tests/miz/pass/parser/pass_parser_block_001.miz
tests/miz/pass/parser/pass_parser_block_001.expect.toml

tests/lexical/pass/pass_lexical_identifier_001.src
tests/lexical/pass/pass_lexical_identifier_001.expect.toml

tests/certificates/fail/sat/fail_certificate_sat_satisfiable_refutation_001.cert.json
tests/certificates/fail/sat/fail_certificate_sat_satisfiable_refutation_001.expect.toml
```

The harness rejects missing sidecars for fail, soundness, certificate,
snapshot, generated, fuzz-regression, and property-regression tests. Pass tests
may omit sidecars only when an explicit harness mode allows legacy discovery;
the committed evo2 corpus should include sidecars for all executable tests.

## Common Fields

All expectation files include:

```toml
schema_version = 1
id = "pass_lexical_identifier_basic_001"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "pass_lexical_identifier_basic_001.src"
expected_outcome = "pass"
spec_refs = [
  "spec.en.02.lexical.identifiers.basic",
]
```

Fields:

| Field | Type | Required | Meaning |
|---|---|---:|---|
| `schema_version` | integer | yes | Sidecar schema version. |
| `id` | string | yes | Stable test id. Must equal the file stem. |
| `kind` | string | yes | `pass`, `fail`, `snapshot`, `generated`, `fuzz_seed`, or `property_seed`. |
| `stage` | string | yes | Staged model stage. |
| `domain` | string | yes | Human-readable test domain. |
| `source` | string | yes | Input file path relative to the sidecar directory. |
| `expected_outcome` | string | yes | `pass`, `fail`, `snapshot`, or `metadata_only`. |
| `spec_refs` | array of strings | yes | Requirement ids from `tests/coverage/spec_trace.toml`. |
| `profiles` | non-empty array of strings | no | Harness profiles that include this test. Values are `fast`, `full`, `stress`, `fuzz_regression`/`fuzz-regression`, or `snapshot_update`/`snapshot-update`. Defaults to `["fast"]`. |
| `tags` | array of strings | no | Non-authoritative grouping tags. |
| `notes` | string | no | Short review note. Not used for matching. |
| `ast_profile` | string | no | AST rendering profile requested by parser-facing snapshot tests. |
| `snapshot_profiles` | non-empty array of strings | no | Snapshot profile ids that should be retained with the sidecar metadata. |
| `architecture22_scenarios` | non-empty sorted array of strings | no | Architecture-22 regression scenario ids covered by this metadata sidecar. |
| `architecture22_equivalence_class` | string | no | Optional registry equivalence class; only valid when every listed scenario has that class. |
| `architecture22_gate` | string | no | `planned` or `active`; defaults to `planned` when scenarios are present. |

Allowed `stage` values:

```text
lexical
parse_only
declaration_symbol
type_elaboration
formula_statement
proof_verification
advanced_semantics
```

The string values match [staged_model.md](./staged_model.md).

## Kind And Outcome Compatibility

`kind` describes the corpus role. `expected_outcome` describes the harness
result contract.

Allowed `kind` values:

| Kind | Meaning |
|---|---|
| `pass` | Ordinary accepting test. |
| `fail` | Ordinary rejecting test. |
| `snapshot` | Snapshot comparison test. |
| `generated` | Generated test with stored origin metadata. |
| `fuzz_seed` | Fuzz seed or promoted fuzz regression. |
| `property_seed` | Property-test seed or promoted property regression. |

Allowed `expected_outcome` values:

| Outcome | Meaning |
|---|---|
| `pass` | The payload must be accepted through `expected_phase`. |
| `fail` | The payload must be rejected at `expected_phase`. |
| `snapshot` | Snapshot hashes must match. |
| `metadata_only` | The sidecar is validated but no payload execution is expected. |

Compatibility:

| `kind` | Allowed `expected_outcome` |
|---|---|
| `pass` | `pass`, `snapshot` |
| `fail` | `fail`, `snapshot` |
| `snapshot` | `snapshot` |
| `generated` | `pass`, `fail`, `snapshot` |
| `fuzz_seed` | `fail`, `metadata_only` |
| `property_seed` | `pass`, `fail`, `metadata_only` |

`metadata_only` is allowed only for seed metadata that is not executed by the
current profile. It is not valid for committed `.miz`, `.src`, or `.cert.json`
payloads in the default fast profile.

## Pipeline Phase Values

Allowed `expected_phase` values:

| Phase | Meaning |
|---|---|
| `lex` | Lexical analysis. |
| `parse` | Parsing and surface syntax recovery. |
| `resolve` | Declaration collection and name/module resolution. |
| `type_check` | Type checking, attribute/mode checking, and early elaboration. |
| `elaboration` | Core elaboration and binder normalization. |
| `cluster_resolution` | Registration and cluster expansion. |
| `overload_resolution` | Overload and template candidate selection. |
| `statement_check` | Typed statement and local context checking. |
| `vc_generation` | Verification-condition generation. |
| `verification` | Proof search/policy verification boundary. |
| `certificate_check` | Certificate parsing and structural validation. |
| `kernel_check` | Kernel replay and rejection boundary. |

Later compiler crates may refine internal phases, but expectation files use
these stable external phase ids.

## Architecture-22 Matrix Metadata

Task 14 records the architecture-22 incremental/parallel verification
regression matrix without adding placeholder runners. Scenario metadata is
validated during expectation parsing and the metadata `plan` command.

`architecture22_scenarios` is optional. If it is absent,
`architecture22_gate` and `architecture22_equivalence_class` must also be
absent. If present, scenario ids must be known, non-empty, unique, and sorted
lexicographically; duplicate or unsorted arrays are validation errors rather
than silently normalized.

`architecture22_equivalence_class`, when present, must be a known registry
class and must match every listed scenario. Sidecars that cover multiple
classes omit this field; reporting uses the registry class for each scenario.

`architecture22_gate` defaults to `planned` when scenarios are present.
`active` is accepted only after a future consumer-specific increment gives the
listed scenario ids active eligibility. Existing active parse-only,
declaration-symbol, or type-elaboration tags do not make architecture-22 matrix
rows active.

Task 14 registry:

| Scenario id | Equivalence class | Active eligibility |
|---|---|---|
| `artifact_manifest_atomicity` | `atomic_publication` | none |
| `cache_hit_miss_timing` | `observable_outputs_equal` | none |
| `cache_key_race` | `single_canonical_publication` | none |
| `clean_incremental_artifact_equivalence` | `observable_outputs_equal` | none |
| `clean_parallel_equivalence` | `observable_outputs_equal` | none |
| `externally_attested_non_upgrade` | `evidence_class_not_upgraded` | none |
| `incremental_parallel_equivalence` | `observable_outputs_equal` | none |
| `missing_dependency_slice_cache_miss` | `cache_miss_only` | none |
| `notation_operator_invalidation` | `downstream_invalidation` | none |
| `proof_witness_mismatch` | `cache_miss_only` | none |
| `randomized_atp_completion_order` | `deterministic_policy_selection` | none |
| `randomized_ready_task_scheduling` | `canonical_order_equal` | none |
| `registration_cluster_invalidation` | `downstream_invalidation` | none |
| `registration_origin_deletion` | `downstream_invalidation` | none |
| `stale_snapshot_non_publication` | `stale_result_not_published` | none |
| `theorem_proof_body_invalidation` | `local_refresh_only` | none |
| `theorem_status_invalidation` | `downstream_invalidation` | none |
| `vcid_reorder_anchor_reuse` | `reuse_requires_full_identity` | none |

## Public Enum Forward Compatibility

Task 12 applies the `mizar-frontend` task-25 procedure to expectation-schema
enums and the crate-local TOML support enum exposed through the public module
tree. They are downstream-facing metadata surfaces and must remain
`#[non_exhaustive]`; downstream callers must keep wildcard match arms, while
`mizar-test` may keep internal matches exhaustive for currently known variants.

| Public enum | Owner | Decision |
|---|---|---|
| `TestKind` | `expectation` corpus role and layout surface | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ExpectedOutcome` | `expectation` result contract | `#[non_exhaustive]` downstream forward-compatible surface. |
| `PipelinePhase` | `expectation` phase boundary ids | `#[non_exhaustive]` downstream forward-compatible surface. |
| `Architecture22Gate` | `expectation` architecture-22 planned/active metadata gate | `#[non_exhaustive]` downstream forward-compatible surface. |
| `TomlValue` | `toml_lite` parser support for expectation and manifest metadata | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module.

## Pass Expectations

Pass expectations require no failure identity.

```toml
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
```

Fields:

| Field | Required | Meaning |
|---|---:|---|
| `expected_phase` | yes | Latest phase the harness must execute for this test. |
| `diagnostic_codes` | yes | Expected diagnostics. Empty means no diagnostics. |
| `snapshots` | no | Current parse-only `SurfaceAst` baseline path, when applicable. |

The harness fails a pass test if an error diagnostic is emitted unless that
diagnostic is explicitly allowed by the expectation.

## Fail Expectations

Fail expectations must state the stable failure identity.

```toml
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "type_error"
rejection_reason = "invalid_type_argument"
diagnostic_codes = ["E-TYPE-INVALID-ARGUMENT"]
stable_detail_key = "types.dependent_mode.invalid_argument"
```

Fields:

| Field | Required | Meaning |
|---|---:|---|
| `expected_phase` | yes | Earliest phase that must soundly reject the input. |
| `failure_category` | yes | Stable category from failure semantics. |
| `rejection_reason` | conditional | Required for certificate and kernel rejection; optional otherwise. |
| `diagnostic_codes` | yes | Stable diagnostic codes in deterministic order. |
| `diagnostic_payloads` | no | Optional stable summaries for machine-readable diagnostic payloads in deterministic order. |
| `snapshots` | no | Current parse-only `SurfaceAst` baseline path, when applicable. |
| `stable_detail_key` | yes | Stable detail identity independent of diagnostic wording. |

A fail test that succeeds is a harness failure. A fail test that fails earlier
than expected is also a harness failure unless the expectation is deliberately
updated to the earlier sound boundary.

## Lexical Expectations

Lexical fixtures may check tokens and lexical diagnostics without invoking the
parser.

```toml
stage = "lexical"
expected_outcome = "pass"
expected_phase = "lex"

[[tokens]]
kind = "identifier"
lexeme = "alpha"

[[tokens]]
kind = "reserved"
lexeme = "definition"
```

Token expectations are optional for smoke tests but required for fixtures that
claim token-level coverage.

Lexical fixtures may also provide `diagnostic_payloads` when the test owns a
machine-readable diagnostic payload contract. These summaries complement
`diagnostic_codes` and avoid matching human-facing message text.

## Parse-Only Expectations

Parse-only fixtures check syntactic acceptance, rejection, or AST shape without
semantic validation.

```toml
stage = "parse_only"
expected_phase = "parse"
tags = ["active_parse_only"]
ast_profile = "surface"
snapshot_profiles = ["surface_ast"]
```

The parse-only corpus runner executes only `.miz` pass/fail sidecars that carry
the `active_parse_only` tag. Untagged parse-only sidecars are still discovered
and traced, but remain inactive seed metadata for future grammar work. For the
current runner, `diagnostic_codes` compare against bare parser syntax keys such
as `missing_end`.

Active parse-only pass/fail sidecars may also use the transitional
`snapshots` field:

```toml
snapshots = "snapshots/parser/pass_parser_minimal_token_stream_001.surface_ast.snap"
```

The path is relative to `tests/`, must stay under `tests/snapshots/`, and names
a committed `SurfaceAst::snapshot_text()` baseline. The parse-only runner
compares that baseline byte-for-byte after diagnostics match. A requested
snapshot with no parser AST, a missing or unreadable baseline, or a content
mismatch is a harness failure. Normal parse-only runs never rewrite snapshot
baselines.

If a current parser recovery case also emits frontend recovery diagnostics from
preprocessing or lexing, the sidecar may add
`allow_frontend_recovery_diagnostics` to assert only the parser syntax keys.
Without that opt-in, non-syntax diagnostics are part of the assertion result.

Parse-only expectations must not include type, resolver, proof, certificate, or
kernel failure identities.

## Declaration And Symbol Expectations

Declaration and symbol expectations assert clean resolver execution, selected
positive declaration-symbol facts, or resolver failures.

```toml
stage = "declaration_symbol"
expected_phase = "resolve"
expected_outcome = "pass"
diagnostic_codes = []
tags = ["active_declaration_symbol"]
```

The declaration-symbol corpus runner executes only `.miz` pass/fail sidecars
that carry the `active_declaration_symbol` tag. Untagged sidecars are still
discovered and traced, but remain inactive seed metadata.

The current runner checks that pass cases produce no frontend assertion
diagnostics and no resolver symbol diagnostics. Pass sidecars may additionally
set `declaration_symbol_payloads` to assert exact, sorted SymbolEnv-derived
fact keys. The supported fact keys are built only from stable symbol and
definition data: primary spelling (percent-escaped), symbol kind, definition
kind, visibility, and export status. They do not use source ranges, ids,
signatures, snapshots, imports, name references, label references, Core IR, VC,
or proof payloads. Sidecars must not include unimplemented `[[symbols]]` table
assertions.

```toml
declaration_symbol_payloads = [
  "declaration_symbol.symbol.kind.VisibleTheorem.theorem",
  "declaration_symbol.definition.kind.VisibleTheorem.theorem",
]
```

`declaration_symbol_payloads` is valid only on active declaration-symbol pass
expectations, and every entry must be non-empty.

Until public resolver diagnostic codes are specified, all active
declaration-symbol cases keep `diagnostic_codes = []`; the active gate rejects
non-empty values. Active fail cases assert resolver-owned internal detail keys
through `diagnostic_payloads`, falling back to `stable_detail_key` when no
payload list is provided:

```toml
expected_outcome = "fail"
expected_phase = "resolve"
failure_category = "resolve_error"
stable_detail_key = "declaration_symbol.symbol.duplicate_declaration"
diagnostic_codes = []
diagnostic_payloads = [
  "declaration_symbol.symbol.duplicate_declaration",
]
tags = ["active_declaration_symbol"]
```

Once a stable resolver diagnostic-code range exists, resolver fail cases may
also assert user-facing codes in `diagnostic_codes`.

## Type And Elaboration Expectations

Type and elaboration expectations may assert normalized types, inserted views,
or type diagnostics.

```toml
stage = "type_elaboration"
expected_phase = "type_check"

[[types]]
subject = "X"
expected = "set"
```

These tests may use only built-ins and symbols admitted by lower stages unless
the expectation explicitly targets a missing prerequisite.

Active `.miz` sidecars for the `type-elaboration` runner must carry
`tags = ["active_type_elaboration"]`, use `stage = "type_elaboration"` and
`expected_phase = "type_check"`, and leave `diagnostic_codes = []` until public
checker diagnostic codes are specified. The runner executes frontend parsing
and resolver symbol collection before checker work.

The supported source-derived pass slice is limited to reserve-only builtin
`set` and `object` declarations: top-level unrecovered reserve items whose
segments have one or more identifiers and exactly one bare builtin
type-expression head, with no attributes, arguments, parameter prefixes, or
non-builtin symbol heads. Such pass cases must contain at least one reserve
binding that the runner extracts into syntax-free source reserve payloads. The
checker-owned source reserve seam builds the module `BindingEnv`, one
`DeclarationInput` per binding, binding-specific `TypeExpressionInput` sites,
and `DeclarationChecker` output; the runner then continues through `TypedAst`,
`ResolvedTypedAst`, and a summary-only `mizar-core`
`ResolvedTypedAstSummary::from_ast` readiness read plus binder-only
`CoreContext` preparation. Multiple identifiers sharing one source
type-expression range must still use distinct typed sites. The summary/context
readiness checks must not be treated as `CoreIr`, `ControlFlowIr`, VC, or proof
execution. The case must be covered by a pass-slice traceability row and assert
empty `diagnostic_codes` with no internal detail payloads:

```toml
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
```

When lower stages succeed but a case needs unsupported source-to-checker
payload families, the runner reports the stable external-gap detail key
`type_elaboration.external_dependency.ast_payload_extraction`:

```toml
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.external_dependency.ast_payload_extraction",
]
tags = ["active_type_elaboration"]
```

Supported checker-owned diagnostic slices may instead assert the checker detail
keys produced by the source reserve seam. Task 50 permits same-module
attributed builtin reserve heads to stop at
`type_elaboration.checker.checker.declaration.deferred.evidence_query`; task
51 permits unique same-module `LocalSource` `SymbolKind::Mode` reserve heads
with no attributes or type arguments to stop at
`type_elaboration.checker.checker.type.external.mode_expansion_payload`, with
the paired recovery key when emitted; task 52 permits unique same-module
`LocalSource` `SymbolKind::Structure` reserve heads with no attributes or type
arguments to stop at
`type_elaboration.checker.checker.declaration.deferred.evidence_query`; task
53 permits those local structure heads to carry same-module no-argument
attribute payloads and stop at the same evidence-query key because full
attributed-type existential evidence is still absent; task 54 permits local
mode heads to carry same-module no-argument attribute payloads and stop at the
mode-expansion key without an evidence-query key when no supported real
mode expansion is available or the same mode is mixed with a bare reserve use;
task 57 permits real local-mode expansions whose RHS is a
same-module local structure head to stop at the evidence-query key because
base-shape/constructor-witness evidence is still absent; task 58 permits real
local-mode expansions whose RHS is an attributed builtin head to stop at the
evidence-query key because attributed-type existential evidence is still
absent; task 59 permits attributed local-mode reserve heads with real direct
bare-builtin expansions to stop at the evidence-query key because
attributed-type existential evidence is still absent; task 60 permits
attributed local-mode reserve heads with real direct local-structure RHS
expansions to stop at the evidence-query key because base-shape/
constructor-witness and full attributed-type evidence are still absent; task 61
permits attributed local-mode reserve heads with real direct
attributed-builtin RHS expansions to stop at the evidence-query key because
full attributed-type evidence is still absent; task 62 permits one-edge bare
local-mode chains ending in local structure RHSs to stop at the evidence-query
key because base-shape/constructor-witness evidence is still absent; task 63
permits one-edge bare local-mode chains ending in attributed builtin RHSs to
stop at the evidence-query key because attributed-type existential evidence is
still absent; task 64 permits attributed local-mode reserve heads with
one-edge bare-builtin chains to stop at the evidence-query key because
attributed-type existential evidence is still absent; task 65 permits attributed
local-mode reserve heads with one-edge structure-RHS chains to stop at the
evidence-query key because structure base-shape/constructor-witness and full
attributed-type existential evidence are still absent; task 66 permits
attributed local-mode reserve heads with one-edge attributed-builtin-RHS chains
to stop at the evidence-query key because full attributed-type existential
evidence is still absent. Task 67 permits structure-qualified attribute
references to stop at the external extraction-gap key because real qualifier
and attribute-owner provenance are still absent. Task 68 permits
argument-bearing local-mode reserve heads to stop at the external
extraction-gap key because real type-argument and term-argument provenance are
still absent. Task 69 permits argument-bearing local-structure reserve heads to
stop at the external extraction-gap key because real type-argument and
term-argument provenance are still absent. Task 70 permits bracket-form
local-mode reserve heads to stop at the external extraction-gap key before
bracket type-argument payload extraction or mode-head resolution because real
bracket type-argument and `qua`-argument provenance are still absent. Task 71
permits bracket-form local-structure reserve heads to stop at the
external extraction-gap key before bracket type-argument payload extraction or
structure-head resolution because real bracket type-argument and
`qua`-argument provenance are still absent. Tasks 67-71 are fail cases, not
pass-slice coverage. Task 72 permits two-edge bare local-mode chain pass
sidecars, task 73 permits three-edge bare local-mode chain pass sidecars, and
task 74 permits structural bare local-mode chain pass sidecars to use an empty
diagnostic payload list; unsupported structural-guard failures still require
the checker missing mode-expansion payload plus recovery detail keys.
Task 75 permits forward local-mode reserve head fail sidecars, task 76 permits
forward local-structure reserve head fail sidecars, and task 77 permits forward
local-attribute reserve type fail sidecars to use
`failure_category = "lower_stage_error"` with
`stable_detail_key = "type_elaboration.lower_stage.frontend:malformed_type_expression"`
when a reserve head names a later local declaration before it is active. Those
sidecars do not credit checker `ModeExpansion`, structure type-head,
base-shape, constructor-witness, `AttributeInput`, or attributed-type evidence
production.
Task 78 permits future imported structure reserve-head fail sidecars outside
the task-83 `R` and task-97 `TypeCaseStruct` bridges to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "imported_structure_type_head_payload_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`
when the source reaches the active runner through the documented
`parser.type_fixtures` import summary. These sidecars do not credit real
imported module AST extraction, imported structure provenance, structure
type-head payload extraction, base-shape or constructor-witness evidence,
positive structure elaboration, CoreIr, ControlFlowIr, VC, or proof payloads.
Tasks 83 and 97 permit the documented imported structure `R` and
`TypeCaseStruct` sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "imported_structure_evidence_payload_gap"` and
`stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"`
after the runner passes the real imported structure type head to the checker.
These sidecars credit only imported provenance/type-head extraction and do not
credit imported module AST extraction, base-shape/constructor-witness evidence,
positive imported structure elaboration, CoreIr, ControlFlowIr, VC, or proof
payloads.
Task 79 permits imported mode reserve-head fail sidecars outside the task-82
`TypeCaseMode` bridge to use `failure_category = "external_dependency_gap"` with
`rejection_reason = "imported_mode_expansion_payload_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`
when the source reaches the active runner through the documented
`parser.type_fixtures` import summary. These sidecars do not credit real
imported module AST extraction, imported mode provenance, mode type-head
payload extraction, `ModeExpansion` payloads, positive mode elaboration, CoreIr,
ControlFlowIr, VC, or proof payloads.
Task 82 permits that same imported mode reserve-head sidecar to move to
`stable_detail_key = "type_elaboration.checker.checker.type.external.mode_expansion_payload"`
once the runner passes the real imported `SymbolKind::Mode` symbol head from
`SymbolEnv`. It credits imported mode provenance and type-head payload
extraction only; it still does not credit imported module AST extraction,
`ModeExpansion` payloads, positive mode elaboration, CoreIr, ControlFlowIr, VC,
or proof payloads.
Task 80 permits broader imported attribute reserve fail sidecars outside the
task-84 `TypeCaseAttr` bridge and task-85 negative `empty`/builtin-`set` bridge
to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "imported_attribute_payload_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`
when the source reaches the active runner through the documented
`parser.type_fixtures` import summary. These sidecars do not credit real
imported module AST extraction, imported attribute provenance, `AttributeInput`
payload extraction, attributed-type evidence, positive attributed type
elaboration, CoreIr, ControlFlowIr, VC, or proof payloads.
Task 84 permits the documented `TypeCaseAttr` sidecar to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "imported_attribute_evidence_payload_gap"` and
`stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"`
after the active runner passes the real imported attribute provenance and
no-argument checker `AttributeInput` payload. This sidecar does not credit
imported module AST extraction, attributed-type existential/evidence payloads,
positive imported attributed type elaboration, generic imported attributes,
qualified owner provenance, attribute arguments, CoreIr, ControlFlowIr, VC, or
proof payloads.
Task 85 permits the existing `non empty set` sidecar to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "imported_empty_attribute_evidence_payload_gap"` and
`stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"`
after the active runner passes the real imported `empty` attribute provenance
and negative no-argument checker `AttributeInput` payload over builtin `set`.
This sidecar does not credit imported module AST extraction, attributed-type
existential/evidence payloads, positive `empty set`, imported `empty` on
non-`set` heads, broader imported attributes, qualified owner provenance,
attribute arguments, CoreIr, ControlFlowIr, VC, or proof payloads.
Separate task-85 boundary sidecars for positive `empty set` and
`non empty object` keep
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`
with `rejection_reason = "positive_imported_empty_attribute_payload_gap"` or
`"imported_empty_non_set_head_payload_gap"`; they document unsupported payload
shapes and do not credit checker `AttributeInput` handoff.
Task 86 permits formula-only theorem fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "formula_statement_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that the source reached parser/resolver execution, but
they do not credit a `formula_statement` runner, checker theorem/formula
payload extraction, recorded facts, proof skeletons, CoreIr, ControlFlowIr, VC,
or proof payloads.
Tasks 87, 98, 100, 101, and 102 permit term-bearing theorem formula fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "term_formula_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that a theorem formula containing Chapter 13 term
surfaces and Chapter 14 atomic formulas, including task-87 numeral/builtin
equality, task-98 imported predicate/functor applications, task-100 builtin
membership, task-101 builtin inequality, and task-102 builtin type assertion,
reached parser/resolver execution,
but they do not credit imported semantic payloads, term/formula payload
extraction, membership operand type inference/checking, inequality desugaring
or equality semantic checking, type-assertion type payload extraction,
type-assertion semantic checking, term inference, formula checking, recorded
facts, theorem acceptance, a `formula_statement` runner, proof skeletons, CoreIr,
ControlFlowIr, VC, or proof payloads.
Task 99 permits formula connective/quantifier theorem fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "formula_connective_quantifier_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that Chapter 14 connective and quantifier formula
surfaces reached parser/resolver execution inside a theorem declaration, but
they do not credit formula payload extraction, quantifier binder/context
payloads, formula checking, recorded facts, theorem acceptance, a
`formula_statement` runner, CoreIr, ControlFlowIr, VC, or proof payloads.
Task 88 permits proof-block theorem fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "proof_skeleton_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that a theorem proof block with a `thus thesis;`
conclusion reached parser/resolver execution, but they do not credit proof
skeleton payload extraction, local proof context, formula payload extraction,
recorded facts, theorem acceptance, a `formula_statement` runner, CoreIr,
ControlFlowIr, VC, or proof payloads.
Task 89 permits statement-proof theorem fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "statement_proof_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that theorem proof statements justified by nested proof
blocks reached parser/resolver execution, but they do not credit statement
proof payload extraction, nested proof skeleton payloads, local proof context,
formula payload extraction, label-reference semantic checking, recorded facts,
theorem acceptance, a `formula_statement` runner, CoreIr, ControlFlowIr, VC, or
proof payloads.
Task 93 permits proof-local declaration statement fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "proof_local_declaration_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that `let`, `given`, `consider`, `set`, and
`reconsider` statements inside a theorem proof reached parser/resolver
execution, but they do not credit proof-local declaration payload extraction,
local proof context, formula/term payloads, RHS term inference, reconsider
coercion/obligation evidence, recorded facts, theorem acceptance, a
`formula_statement` runner, CoreIr, ControlFlowIr, VC, or proof payloads.
Task 94 permits proof-local inline definition fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "inline_definition_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that `deffunc` and `defpred` statements inside a theorem
proof reached parser/resolver execution, but they do not credit inline
definition formal/body payload extraction, local abbreviation expansion,
term/formula body payloads, guard evidence, recorded facts, theorem acceptance, a
`formula_statement` runner, CoreIr, ControlFlowIr, VC, or proof payloads.
Task 95 permits registration-block fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "registration_block_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that a top-level `registration` block reached
parser/resolver execution, but they do not credit registration-item payload
extraction, correctness-condition/proof-obligation payloads, accepted
activation/evidence status, cluster/reduction semantics, Chapter 17 semantic
rows, a `formula_statement` or `advanced_semantics` runner, CoreIr,
ControlFlowIr, VC, or proof payloads.
Task 96 permits redefinition/notation fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "redefinition_notation_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that synonym/antonym aliases and attribute, predicate,
and functor redefinition declarations reached parser/resolver execution, but
they do not credit redefinition payload extraction, notation alias relation
payloads, redefinition target inference, coherence proof-obligation payloads,
overload candidate payloads, Chapter 11 alias semantic resolution, Chapter 19
overload/redefinition semantics, a `formula_statement` or `advanced_semantics`
runner, CoreIr, ControlFlowIr, VC, or proof payloads.
Task 90 permits predicate/functor definition fail sidecars to use
`failure_category = "external_dependency_gap"` with
`rejection_reason = "definition_declaration_payload_extraction_gap"` and
`stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"`.
These sidecars document that definition declarations reached parser/resolver
execution, but they do not credit checker definition declaration payload
extraction, definition-local context, formula/term body payloads, overload
payloads, recorded facts, a `formula_statement` runner, CoreIr, ControlFlowIr,
VC, or proof payloads.
Task 91 permits attribute definition fail sidecars to use the same
`failure_category`, `rejection_reason`, and `stable_detail_key`. These sidecars
document that attribute definitions reached parser/resolver execution, but they
do not credit checker attribute definition declaration payload extraction,
definition-local context, formula-definiens payloads, attributed-type evidence,
recorded facts, a `formula_statement` runner, CoreIr, ControlFlowIr, VC, or
proof payloads.
Task 92 permits mode/structure definition fail sidecars to use the same
`failure_category`, `rejection_reason`, and `stable_detail_key`. These sidecars
document that mode and structure definitions reached parser/resolver execution,
but they do not credit checker mode/structure definition declaration payload
extraction, mode expansion, structure base-shape/constructor/selector evidence,
definition-local context, recorded facts, a `formula_statement` runner, CoreIr,
ControlFlowIr, VC, or proof payloads.

Detailed type assertion tables and broader type pass expectations remain
deferred until the runner can build checker-owned payloads from `.miz` source
without inventing non-builtin declarations, imported symbols, unresolved or
ambiguous symbols, attribute or mode/structure arguments,
imported attributed structure heads, qualified attribute provenance,
type-argument, term-argument, bracket `type_arg_list`, or `qua`-argument
provenance, structure base-shape evidence,
terms, formulas, coercions, facts, overload evidence, CoreIr, ControlFlowIr,
VC payloads, or proof evidence.

## Formula, Statement, And Proof Expectations

Formula and statement expectations check typed formulas, statement structure,
labels, and local proof context.

Proof expectations add verification outcome checks:

```toml
stage = "proof_verification"
expected_phase = "verification"
expected_outcome = "fail"
failure_category = "kernel_rejection"
rejection_reason = "invalid_sat_proof"
diagnostic_codes = ["E-KERNEL-INVALID-PROOF"]
stable_detail_key = "soundness.false_arithmetic.one_eq_zero"
```

Soundness tests belong here or to `advanced_semantics`, depending on whether
they require certificate/kernel payloads.

## Certificate Expectations

Certificate tests use certificate payloads and never depend on parsing `.miz`.

```toml
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "fail_certificate_sat_satisfiable_refutation_001.cert.json"
expected_outcome = "fail"
expected_phase = "kernel_check"
failure_category = "kernel_rejection"
rejection_reason = "invalid_sat_refutation"
diagnostic_codes = []
stable_detail_key = "soundness.certificate.invalid_sat_refutation"
```

Certificate expectations must include `rejection_reason`.

## Snapshot Expectations

Snapshot expectations compare deterministic artifact hashes.

```toml
expected_outcome = "snapshot"

[[snapshots]]
profile = "surface_ast"
path = "pass_parser_block_001.surface_ast.json"
hash = "sha256:..."
```

Snapshot update mode is explicit. The harness must not rewrite snapshots during
normal pass/fail execution.

The `[[snapshots]]` hash registry is the future general snapshot contract. The
current parser task-38 slice above is only a parse-only `SurfaceAst` shortcut
for active pass/fail sidecars and does not complete general `kind = "snapshot"`
execution or hash-registry update mode.

## Generated, Fuzz, And Property Metadata

Generated and fuzz/property regression tests record provenance.

```toml
[origin]
schema_version = 1
kind = "generated"
generator = "grammar-smoke"
generator_version = "0.1.0"
seed = "0000000000000001"
profile = "lexical-identifiers"
expected_outcome = "pass"
minimized = false
```

`[origin]` is required for `kind = "generated"`, `kind = "fuzz_seed"`, and
`kind = "property_seed"` sidecars. `origin.kind` and `origin.expected_outcome`
match the sidecar's top-level `kind` and `expected_outcome`; metadata-only
handoff anchors therefore use `expected_outcome = "metadata_only"` in both
places. `origin.schema_version` is `1`; `generator`, `generator_version`,
`seed`, and `profile` are required non-empty strings; `minimized` is a boolean.
Unknown origin fields are rejected. Promoted fuzz failures remain `kind =
"fuzz_seed"` with `expected_outcome = "fail"` and keep executable fail identity
at the top level.

All fuzz seed sidecars preserve their original failure category family through
`origin.original_failure_category`. For promoted executable fuzz failures, that
origin category must match the top-level `failure_category`.

## Validation

The harness validates:

1. The sidecar parses as TOML.
2. `schema_version` is supported.
3. `id` equals the sidecar stem.
4. `source` exists and has the same stem.
5. `kind`, `stage`, and `expected_outcome` are compatible.
6. `spec_refs` are non-empty for committed tests and exist in the traceability
   manifest.
7. Fail expectations include failure identity fields.
8. Certificate and kernel rejections include `rejection_reason`.
9. Diagnostic codes are sorted in the expected deterministic order.
10. Transitional parse-only `snapshots` paths are active parse-only pass/fail
    only, clean tests-root-relative paths under `snapshots/`, and `.snap`
    files; missing, unreadable, or mismatched baselines are harness failures.
11. General snapshot entries use supported hash algorithms.
12. Generated/fuzz/property tests include origin metadata.
13. Architecture-22 matrix metadata uses known sorted scenario ids, known gate
    values, and matching equivalence classes; orphan gate/class fields are
    rejected.
14. Unknown fields are rejected unless the schema version explicitly permits
    extensions.

Validation of coverage completeness depends on the validation mode defined in
[traceability.md](./traceability.md). Schema validation itself is mode
independent.

## Constraints And Assumptions

- Expectations are reviewed source, not generated truth from current compiler
  output.
- Diagnostic text is not matched by default; stable diagnostic codes and detail
  keys are matched.
- Sidecar parsing must work even when the corresponding source file is invalid.
- Schema migrations are explicit and versioned.
