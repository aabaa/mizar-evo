# Module: harness

> Canonical language: English. Japanese companion: [../ja/harness.md](../ja/harness.md).

## Purpose

This module defines the test harness that discovers cases, runs compiler profiles, checks expectations, and reports deterministic results.

## Public API

```rust
pub struct DiscoveryConfig {
    pub workspace_root: PathBuf,
    pub tests_root: PathBuf,
    pub manifest_path: PathBuf,
    pub profile: TestProfile,
    pub validation_mode: ValidationMode,
}

pub struct TestPlan {
    pub cases: Vec<TestCase>,
    pub manifest: TraceManifest,
    pub coverage_report: CoverageReport,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct CoverageReport {
    pub requirements: Vec<RequirementCoverage>,
    pub stages: Vec<StageCoverage>,
    pub pass_fail_mix: PassFailMix,
    pub architecture22_matrix: Architecture22MatrixReport,
}

pub struct RequirementCoverage {
    pub id: SpecRequirementId,
    pub stage: Stage,
    pub coverage: CoverageShape,
    pub required: bool,
    pub stored_status: RequirementStatus,
    pub computed_status: RequirementStatus,
    pub evidence: CoverageEvidenceSummary,
    pub missing_shapes: Vec<CoverageShape>,
}

pub struct StageCoverage {
    pub stage: Stage,
    pub requirements: usize,
    pub covered: usize,
    pub partial: usize,
    pub planned: usize,
    pub deferred: usize,
    pub obsolete: usize,
    pub missing_shapes: usize,
}

pub struct PassFailMix {
    pub pass: usize,
    pub fail: usize,
    pub total: usize,
    pub target_pass_percent: u8,
    pub target_fail_percent: u8,
}

pub struct Architecture22MatrixReport {
    pub scenarios: Vec<Architecture22ScenarioReport>,
    pub missing_scenarios: Vec<String>,
}

pub struct Architecture22ScenarioReport {
    pub scenario_id: String,
    pub equivalence_class: String,
    pub planned: usize,
    pub active: usize,
}

pub struct TestCase {
    pub id: TestCaseId,
    pub source_path: PathBuf,
    pub expectation_path: PathBuf,
    pub expectation: Expectation,
}

#[non_exhaustive]
pub enum TestProfile {
    Fast,
    Full,
    Stress,
    FuzzRegression,
    SnapshotUpdate,
}

#[non_exhaustive]
pub enum ValidationMode {
    Metadata,
    Development,
    Release,
}

pub struct ParseOnlyRunReport {
    pub results: Vec<ParseOnlyCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct DeclarationSymbolRunReport {
    pub results: Vec<DeclarationSymbolCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct TypeElaborationRunReport {
    pub results: Vec<TypeElaborationCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}
```

The generic `TestOutcome`/snapshot-reporting surface is future API. Current
active runners expose stage-specific report records while sharing the metadata
plan and validation diagnostics shown above.

## Public Enum Forward Compatibility

Task 12 applies the `mizar-frontend` task-25 procedure to the harness-facing
enum surface. These enums are downstream API and must remain
`#[non_exhaustive]`; downstream callers must keep wildcard match arms, while
`mizar-test` may keep crate-internal matches exhaustive for the currently known
variants.

| Public enum | Owner | Decision |
|---|---|---|
| `ValidationSeverity` | `diagnostic` reporting used by harness plans and runner reports | `#[non_exhaustive]` downstream forward-compatible surface. |
| `TestProfile` | `harness` profile selection | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ValidationMode` | `harness` validation strictness | `#[non_exhaustive]` downstream forward-compatible surface. |
| `HarnessError` | `harness` infrastructure failure boundary | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ParseOnlyCaseStatus` | `runner` parse-only report status | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DeclarationSymbolCaseStatus` | `runner` declaration-symbol report status | `#[non_exhaustive]` downstream forward-compatible surface. |
| `TypeElaborationCaseStatus` | `runner` type-elaboration report status | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module.

## Runner Modes

| Mode | Behavior |
|---|---|
| metadata plan | discover sidecars and validate layout, expectation schema, and traceability without executing payloads |
| parse-only | run active `.miz` parse-only cases through `mizar-frontend` and `MizarParserSeam` |
| declaration-symbol | run active `.miz` declaration-symbol cases through frontend parsing and resolver declaration/symbol collection |
| type-elaboration | run active `.miz` type-elaboration cases through frontend parsing and resolver declaration/symbol collection, extract supported reserve-only declaration payloads, delegate checker-owned `BindingEnv`/`DeclarationInput`/`DeclarationChecker` handoff production to the syntax-free `mizar-checker` seam, continue successful bare-builtin, task-55 bare local-mode-expansion, and task-56 one-edge local-mode chain cases through `TypedAst` and `ResolvedTypedAst`, confirm `mizar-core` summary-readiness through `ResolvedTypedAstSummary::from_ast`, prepare binder-only `CoreContext` input from the same reserve bindings, surface same-module attributed reserve declarations, local structure reserve heads, attributed local structure reserve heads, task-57 real local-mode expansions with local structure RHSs, task-58 real local-mode expansions with attributed builtin RHSs, task-59 attributed local-mode reserve heads with real direct bare-builtin expansions, task-60 attributed local-mode reserve heads with real direct local-structure RHS expansions, task-61 attributed local-mode reserve heads with real direct attributed-builtin RHS expansions, task-62 one-edge bare local-mode chains ending in local structure RHSs, and task-63 one-edge bare local-mode chains ending in attributed builtin RHSs as checker evidence-query gaps, surface same-module local mode reserve heads that lack the narrow task-55/task-56/task-57/task-58/task-59/task-60/task-61/task-62/task-63 expansion slices, including mixed attributed/bare local-mode sources and attributed chain dependencies, as checker mode-expansion payload gaps, and surface unsupported checker payload families as stable external dependency gaps |
| pass/fail | run `.miz` cases and match expected outcome |
| snapshot | compare canonical snapshot hashes |
| determinism | repeat runs and compare artifacts, diagnostics, and hashes |
| parallel-equivalence | compare sequential and parallel outputs |
| fuzz-regression | run minimized fuzz cases as ordinary committed tests |
| update | rewrite snapshots only when explicitly requested |

## Consumer Runner Pacing

Task 10 keeps runner support synchronized with consumer crates one increment at
a time. Prepared increments are implemented and verified; unprepared consumers
stay `paced/open` without placeholder runner modes, fake active fixtures, or
fabricated coverage.

| Consumer task | Stage / runner | mizar-test status | Next condition |
|---|---|---|---|
| `mizar-parser` task 3 | `parse_only` / `parse-only` | prepared/implemented; active `.miz` pass/fail sidecars use `active_parse_only`, and untagged parse-only metadata stays planned | Keep the transitional `SurfaceAst` snapshot shortcut until the general snapshot runner lands. |
| `mizar-resolve` task 23 | `declaration_symbol` / `declaration-symbol` | prepared/implemented; active sidecars use `active_declaration_symbol`, public resolver diagnostic-code matching remains gated | Open public diagnostic-code assertions only after resolver diagnostic ranges are specified. |
| `mizar-checker` task 12 plus task 16-20, task 48 source bridge continuation, task 50 attributed reserve evidence-gap bridge, task 51 local mode expansion-gap bridge, task 52 local structure evidence-gap bridge, task 53 attributed local structure evidence-gap bridge, task 54 attributed local mode expansion-gap bridge, task 55 bare local mode expansion bridge, task 56 local-mode expansion chain bridge, task 57 local-mode structure-RHS evidence-gap bridge, task 58 local-mode attributed-builtin-RHS evidence-gap bridge, task 59 attributed local-mode reserve evidence-gap bridge, task 60 attributed local-mode structure-RHS evidence-gap bridge, task 61 attributed local-mode attributed-builtin-RHS evidence-gap bridge, task 62 local-mode structure-RHS chain evidence-gap bridge, task 63 local-mode attributed-builtin-RHS chain evidence-gap bridge, reserve summary-readiness, and binder-only core context follow-up | `type_elaboration` / `type-elaboration` | prepared/implemented; active sidecars use `active_type_elaboration`, lower stages run first, reserve-only builtin `set`/`object` declarations are extracted from `.miz` AST into syntax-free checker payloads, same-module attribute symbols already present in `SymbolEnv` may be attached to builtin reserve type payloads, same-module local mode reserve heads, and same-module local structure reserve heads, same-module local mode and structure symbols may be used as argument-free reserve heads, task 55 additionally extracts a real mode expansion only for bare local-mode reserve uses whose unique preceding no-argument same-module mode definition has a bare builtin RHS and no definition-local context, task 56 extracts a one-edge local-mode chain only when the dependency mode already has that accepted task-55 builtin RHS expansion, task 57 extracts a real local-mode expansion whose RHS is a same-module local structure head as a terminal expansion payload, task 58 extracts a real local-mode expansion whose RHS is an attributed builtin head as a terminal expansion payload, task 59 extracts a real direct bare-builtin local-mode expansion for an attributed local-mode reserve head when the same mode is not also used as a bare reserve head, task 60 extracts a real direct local-structure RHS local-mode expansion for an attributed local-mode reserve head when the same mode is not also used as a bare reserve head, task 61 extracts a real direct attributed-builtin RHS local-mode expansion for an attributed local-mode reserve head when the same mode is not also used as a bare reserve head, task 62 extracts both real expansions for a one-edge bare local-mode chain ending in a same-module local structure RHS under the same unique/unrecovered/preceding/no-context source constraints, task 63 extracts both real expansions for a one-edge bare local-mode chain ending in an attributed builtin RHS under the same source constraints plus argument-free same-module RHS attributes, `mizar-checker` produces the checker-owned `BindingEnv`, one `DeclarationInput` per binding, binding-specific `TypeExpressionInput` sites, and `DeclarationChecker` output, successful bare-builtin, task-55 bare local-mode, and task-56 chain cases continue through `TypedAst`, checker-owned `ResolvedTypedAst`, a `mizar-core` `ResolvedTypedAstSummary::from_ast` read, and binder-only `CoreContext` preparation, while attributed reserve, local-structure, task-57 structure-RHS expansion, task-58 attributed-RHS expansion, task-59 attributed local-mode expansion, task-60 attributed local-mode structure-RHS expansion, task-61 attributed local-mode attributed-RHS expansion, task-62 local-mode structure-RHS chain expansion, and task-63 local-mode attributed-RHS chain expansion cases stop at the checker `MissingEvidenceQuery` diagnostic and local-mode cases outside task 55/56/57/58/59/60/61/62/63, including mixed attributed/bare local-mode sources or attributed chain dependencies, stop at the missing mode-expansion diagnostic; unsupported checker payload families stay on `type_elaboration.external_dependency.ast_payload_extraction` | Broader type pass/fail semantic assertions wait for AST-wide source-to-checker payload extraction and real existential/evidence-query/mode-expansion/base-shape inputs beyond the task-55 bare builtin RHS, task-56 one-edge chain, task-57 structure-RHS diagnostic slice, task-58 attributed-RHS diagnostic slice, task-59 attributed local-mode reserve diagnostic slice, task-60 attributed local-mode structure-RHS diagnostic slice, task-61 attributed local-mode attributed-RHS diagnostic slice, task-62 local-mode structure-RHS chain diagnostic slice, and task-63 local-mode attributed-RHS chain diagnostic slice. |
| `mizar-checker` task 29 | `formula_statement` / `advanced_semantics` | paced/open; trace rows are deferred and no active fixture is fabricated | Add runner support only after statement/formula and advanced-semantics source payload seams exist. |
| `mizar-vc` task 15 | `proof_verification` | paced/open; VC/proof-verification obligations are deferred | Add runner support only after source-to-core/source-to-VC extraction and downstream verification seams exist. |
| `mizar-atp` task 20 | `advanced_semantics` metadata handoff | paced/open in `mizar-test`; metadata-only property fixtures may be consumed by `mizar-atp` Rust tests | Add active `.miz` ATP runner support only after source-derived ATP extraction and proof-policy/kernel handoff seams exist. |
| `mizar-kernel` task 17 | proof/certificate/kernel evidence | paced/open; fail/soundness metadata is validated without active proof/certificate/kernel execution | Add runner support only after source-to-evidence or certificate execution seams exist. |

## Algorithm / Logic

1. Discover tests through `layout` under the known payload roots
   `miz`, `lexical`, `certificates`, `generated`, `fuzz`, `property`,
   `stress`, and `snapshots`.
2. Parse and validate every discovered sidecar, then build a canonical
   `TestPlan` whose returned `cases` are filtered by `DiscoveryConfig.profile`.
   Missing `profiles` defaults to `["fast"]`; `Full` includes every valid
   parsed case. Duplicate ids, traceability links, and diagnostics are checked
   across all parsed sidecars, not only the filtered cases.
3. For `parse-only`, select only cases with `stage = "parse_only"`,
   `expected_phase = "parse"`, `.miz` payloads, pass/fail outcomes, and
   `tags = ["active_parse_only"]`. Untagged parse-only sidecars remain
   discovery and traceability metadata.
4. For `declaration-symbol`, select only cases with
   `stage = "declaration_symbol"`, `expected_phase = "resolve"`, `.miz`
   payloads, pass/fail outcomes, and `tags = ["active_declaration_symbol"]`.
   Untagged declaration-symbol sidecars remain discovery and traceability
   metadata.
5. For `type-elaboration`, select only cases with
   `stage = "type_elaboration"`, `expected_phase = "type_check"`, `.miz`
   payloads, pass/fail outcomes, and `tags = ["active_type_elaboration"]`.
   Untagged type-elaboration sidecars remain discovery and traceability
   metadata.
6. Run cases in deterministic display order, even when execution is parallel.
7. Capture compiler outputs as structured records.
8. Match pass/fail expectations before snapshot expectations.
9. Compare general `[[snapshots]]` entries by canonical hash; the current
   parse-only `SurfaceAst` shortcut compares committed text baselines
   byte-for-byte as described below.
10. Report failures with phase, failure category, rejection reason, diagnostic code, and snapshot diff summary.

The current parse-only runner copies each active corpus file into a temporary
`src/` package, runs the real frontend parser seam, requires pass cases to
produce an AST with no assertion diagnostics, and compares fail cases against
the expected bare syntax diagnostic keys. For this syntax-only mode, the runner
uses a harness provider that resolves every frontend import stub to a
`ResolvedImportEntry` with matching `stub_ordinal` and `stub_span`, plus one
`ModuleLexicalSummary` per distinct module id. Summaries contain no exported
symbols except for the narrow `parser.type_fixtures` fixture module, which
injects parser-owned attribute, mode, structure, predicate, and functor shapes
needed by type-expression and operator syntax fixtures. No other import summary
exports symbols; the summaries exist only to keep import syntax cases from
depending on semantic module availability. If parser syntax diagnostics and non-syntax
frontend recovery diagnostics both appear, the runner reports all diagnostic
codes unless the sidecar explicitly includes
`allow_frontend_recovery_diagnostics`. Active parse-only pass/fail sidecars may
also set the transitional `snapshots = "snapshots/parser/<id>.surface_ast.snap"`
field. For those cases, after diagnostics match, the runner requires a
`SurfaceAst` and compares `SurfaceAst::snapshot_text()` with the committed
baseline under `tests/snapshots/`. Snapshot baselines are never rewritten during
normal parse-only runs.

An expectation tagged `active_parse_only` but missing one of the runnable case
predicates is a harness error rather than a silent skip.

The current declaration-symbol runner copies each active `.miz` corpus file
into the same temporary package shape, runs the real frontend, then feeds the
resulting `SurfaceAst` through the resolver declaration-shell collector,
parser-backed signature projection extractor, and symbol collector. Pass cases
require no frontend assertion diagnostics and no resolver symbol diagnostics;
when `declaration_symbol_payloads` is present, the runner also compares those
expected keys against the exact sorted SymbolEnv-derived symbol/definition fact
keys. Fail cases compare the resolver's crate-local internal detail keys
against `diagnostic_payloads` when present, or `stable_detail_key` otherwise.
The runner does not require or invent public resolver diagnostic codes while
the diagnostic-code ownership gap remains open; active declaration-symbol
expectations with non-empty `diagnostic_codes` are harness errors.

An expectation tagged `active_declaration_symbol` but missing one of the
runnable case predicates is a harness error rather than a silent skip.

The current type-elaboration runner copies each active `.miz` corpus file into
the same temporary package shape, runs the real frontend, then feeds the
resulting `SurfaceAst` through the resolver declaration-shell collector,
parser-backed signature projection extractor, and symbol collector. This keeps
type-elaboration cases honest about lower-stage prerequisites before checker
payload extraction begins.

After lower stages pass, the runner extracts syntax-free reserve declaration
payloads only for unrecovered reserve sources whose segments have one or more
identifiers and a supported reserve type-expression head. Successful pass cases
still require the bare builtin `set` / `object` shape with no attributes,
arguments, parameter prefixes, or non-builtin symbol heads, except that task 55
adds a second pass slice for bare local-mode reserve heads when the runner can
derive a real `ModeExpansion` from a unique unrecovered preceding same-module
no-argument `ModeDefinition` with no definition-local context and a bare
builtin `set` / `object` RHS. Task 56 extends that pass slice to one-edge
same-module local-mode chains when the reserve head expands to a preceding
same-module no-argument local mode whose own preceding source definition has an
accepted task-55 bare builtin RHS expansion. Task 50 adds one
active fail slice: a same-module attribute symbol that resolver declaration/
symbol collection has already put in `SymbolEnv` may be attached to the builtin
reserve type payload, causing checker declaration checking to emit
`checker.declaration.deferred.evidence_query` rather than the broader AST
payload extraction gap. Task 51 adds a second active fail slice: a unique
same-module local mode symbol with no attributes or type arguments may be used
as the reserve type head, causing checker type normalization to emit
`checker.type.external.mode_expansion_payload` when neither the task-55 bare
expansion slice nor the task-56 one-edge chain slice applies. Task 52 adds a third active fail slice: a
unique same-module local structure symbol with no
attributes or type arguments may be used as the reserve type head, causing
checker declaration checking to emit
`checker.declaration.deferred.evidence_query` because real
base-shape/constructor-witness evidence extraction is still absent. Task 53
adds a fourth active fail slice: same-module source-derived attributes may be
attached to that local structure head, still causing
`checker.declaration.deferred.evidence_query` because Chapter 17 requires
existential evidence for the full normalized attributed type. Task 54 adds a
fifth active fail slice: same-module source-derived attributes may be attached
to a same-module local mode reserve head, still causing
`checker.type.external.mode_expansion_payload` when no supported real
expansion is available or the same mode is mixed with a bare reserve use; the
runner withholds task-55/task-56 expansions from mixed attributed/bare
local-mode sources. Task 56 also adds an active fail
slice proving that an attributed dependency in a local-mode chain withholds the
whole chain and reaches the same missing mode-expansion diagnostic rather than
inserting a partial `B -> A` payload. Task 57 adds another active fail slice:
a same-module no-argument local mode definition may have a bare same-module
local structure RHS, so the runner passes the real expansion payload to the
checker seam; because the checker still lacks source-derived
base-shape/constructor-witness evidence for that expanded structure radix, the
case reaches `checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 58 adds the parallel
attributed-builtin RHS diagnostic slice: a same-module no-argument local mode
definition may have an attributed builtin RHS, so the runner passes the real
expansion payload to the checker seam; because the checker still lacks
source-derived attributed-type existential evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 59 adds the matching
attributed local-mode reserve diagnostic slice: if a same-module attributed
local-mode reserve head has a real direct bare-builtin RHS expansion and the
same mode is not mixed with a bare reserve use, the runner passes that
expansion payload to the checker seam; because the checker still lacks
source-derived attributed-type existential evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 60 adds the direct
attributed local-mode structure-RHS diagnostic slice: if a same-module
attributed local-mode reserve head has a real direct local-structure RHS
expansion and the same mode is not mixed with a bare reserve use, the runner
passes that expansion payload to the checker seam; because the checker still
lacks source-derived base-shape/constructor-witness and full attributed-type
evidence, the case reaches `checker.declaration.deferred.evidence_query`
instead of `checker.type.external.mode_expansion_payload`. Task 61 adds the
direct attributed local-mode attributed-builtin-RHS diagnostic
slice: if a same-module attributed local-mode reserve head has a real direct
attributed-builtin RHS expansion and the same mode is not mixed with a bare
reserve use, the runner passes that expansion payload to the checker seam;
because the checker still lacks source-derived full attributed-type evidence,
the case reaches `checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 62 adds a one-edge bare
local-mode structure-RHS chain diagnostic slice: if `A is B` and `B is
LocalStruct` are unique, unrecovered, same-module, no-argument mode
definitions in source order after the unique local structure definition and
before the reserve use, the runner passes both real expansion payloads to the
checker seam; because the checker still lacks source-derived base-shape/
constructor-witness evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 63 adds a one-edge bare
local-mode attributed-builtin-RHS chain diagnostic slice: if `A is B` and the
terminal attributed-builtin mode definition (`B is marked set` or
`B is marked object`) are unique, unrecovered, same-module, no-argument mode
definitions in source order before the reserve use, and the RHS
attributes are argument-free same-module attributes, the runner passes both
real expansion payloads to the checker seam; because the checker still lacks
source-derived attributed-type existential evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Imported
attributes, imported modes or structures, unresolved or ambiguous symbols,
attribute arguments, qualified attribute disambiguation, mode/structure
arguments, parameterized or contextual mode definitions, attributed structure
RHSs outside the task-62 bare chain slice, structure-RHS chains outside the
task-60 direct attributed-root slice and task-62 bare chain slice,
attributed-RHS chains outside the task-61 direct slice and task-63 bare chain
slice, forward-reference or
cyclic local-mode chains, and non-reserve declarations remain outside this
source bridge.

For extracted payloads, the runner passes source/module identity, reserve
source range, binding spelling/ranges, supported type-expression spelling/
ranges/heads, supported same-module attribute symbol/range/polarity data, and
supported same-module local-mode expansion payloads, including the task-57
terminal local-structure RHS diagnostic slice, task-58 terminal
attributed-builtin RHS diagnostic slice, and task-59 attributed local-mode
reserve diagnostic slice, and task-60 attributed local-mode structure-RHS
diagnostic slice, and task-61 attributed local-mode attributed-RHS diagnostic
slice, task-62 local-mode structure-RHS chain diagnostic slice, and task-63
local-mode attributed-RHS chain diagnostic slice, to
`mizar-checker`'s source
reserve declaration seam. That checker-owned seam
builds the module `BindingEnv`, one `DeclarationInput` per binding, and
binding-specific `TypeExpressionInput` sites, so `reserve x, y for set` keeps
the shared source range while giving each binding a distinct typed site, and
runs `DeclarationChecker` against the collected `SymbolEnv`. The runner may
use the same checker-owned assembly helper to collect stable diagnostic keys
for active fail slices. If checker diagnostics are emitted, the active fail
case compares those keys and the runner does not credit downstream readiness
assertions. For diagnostic-free supported output, including the bare builtin
and task-55/56 local-mode expansion slices, the returned checker handoff is
credited as a checker-owned `TypedAst` with declaration and type-entry links
and as checker-owned `ResolvedTypedAst` projected using empty-but-real
cluster/overload predecessor outputs plus source-preserved node hints and
declaration expression metadata. The runner then passes that real
`ResolvedTypedAst` payload to `mizar-core`'s
`ResolvedTypedAstSummary::from_ast` and checks that the summary preserves the
source/module identity and has no checker recovery/diagnostic sites for the
successful reserve-only slice. It then prepares binder-only `CoreContextInput`
from the same real reserve bindings, with one `CoreVariableSeed` and one
`CoreBinderSeed` per extracted binding, no `CoreItemSeed`, and checks
source/module identity, binder source ranges, checker provenance, empty item
registry, empty core diagnostics, and an empty core worklist. This is a
summary/context readiness check only: it does not construct `CoreIr`,
`ControlFlowIr`, obligation seeds, VCs, or proof rows.
Active pass cases may assert this supported
source-derived slice with empty detail keys only when the source contains at
least one supported reserve binding and runner regression evidence confirms
that checker handoff construction, declaration checking, `TypedAst` assembly,
`ResolvedTypedAst` assembly, summary-readiness, and binder-only core context
readiness were exercised.

The runner still does not fabricate the missing AST-wide source-to-checker
bridge. Non-builtin declarations, imported attributes, imported modes or
structures, attribute arguments, mode/structure
arguments, structure base-shape evidence, terms, formulas, coercion sites, overload
evidence, recorded facts, CoreIr, ControlFlowIr, VC payloads, and proof
evidence remain outside the supported extraction slice. When an active case
needs one of those
unsupported payload families, the runner reports the stable detail key
`type_elaboration.external_dependency.ast_payload_extraction`. Active fail
cases may assert that key through `diagnostic_payloads` or `stable_detail_key`;
active pass cases outside the supported slice remain deferred rather than
passing through a stub. This runner does not publish `CoreIr`, `ControlFlowIr`,
VC seeds, proof rows, or public checker diagnostic codes.

Active type-elaboration expectations with non-empty `diagnostic_codes` are
harness errors until public checker diagnostic codes are specified. An
expectation tagged `active_type_elaboration` but missing one of the runnable
case predicates is a harness error rather than a silent skip.

General snapshot and determinism runner rows above are target-state harness
modes. Tasks 4 and 5 provide the shared `SnapshotRecord`, baseline
verify/update, and repeat-render comparison APIs, but this harness does not yet
parse general `[[snapshots]]` sidecar entries or run a general snapshot/update
subcommand. The active parse-only `SurfaceAst` shortcut remains the only
snapshot path wired into runner execution.

Architecture-22 matrix support is metadata/reporting-only in task 14. The
metadata plan validates `architecture22_scenarios`,
`architecture22_equivalence_class`, and `architecture22_gate`, then reports the
registry class plus planned/active counts for each required scenario. All
task-14 scenario rows have no active eligibility, so `architecture22_gate =
"active"` is rejected until a future consumer-specific increment wires real
clean/incremental/parallel/cache-race execution.

## Determinism Requirements

The harness checks that identical inputs produce:

- identical artifact hashes;
- identical snapshot hashes;
- identical diagnostic order;
- identical failure records;
- identical proof status;
- identical dependency slices.

Parallel execution may change runtime, not observable results.

Implemented task-11 coverage renders metadata plans and active runner reports
to deterministic byte strings and compares repeated builds/runs. Snapshot-level
determinism and parallel equivalence are covered by the general snapshot record
helpers; active parallel runner subcommands remain future work until a consumer
crate exposes parallel execution.

## Reporting

Reports must separate:

- unexpected success;
- unexpected failure;
- wrong failure category;
- wrong rejection reason;
- diagnostic order mismatch;
- snapshot mismatch;
- nondeterminism across repeated runs;
- harness infrastructure error.

## Tests

Key scenarios:

- fail test unexpectedly passes;
- pass test emits an error diagnostic;
- snapshot hash differs;
- metadata plan bytes differ across repeated builds;
- active runner report bytes differ across repeated runs;
- repeated run produces a different diagnostic order;
- generic snapshot parallel equivalence produces the same observable artifact
  as sequential snapshot generation.
- architecture-22 matrix metadata reports all required scenario ids as planned
  and rejects fake active rows before an owning consumer runner exists.

## Constraints and Assumptions

- Test execution order is not semantic ordering.
- The harness treats cache hits as compiler behavior to verify, not as proof authority.
- Snapshot update mode is opt-in and must be visible in command output.
