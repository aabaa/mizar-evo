# Task CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264: Task264 Core owner disposition

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264.md](../ja/CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264.md).

Status: complete on the exact docs-only task commit. All independent reviews
ended with no findings after repairs; all nine hard gates pass at a valid
`99/100` with no score cap. The commit hash is reported in the final handoff
because a commit cannot embed its own hash. This task changes no language
behavior, public API, Rust source, test intent, or coverage credit.

## Identity, authority, and readiness

| Field | Contract value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264` |
| Primary owner | `mizar-core` Core Task 36 definition lowering |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/en/00.crate_plan.md) |
| Lower authority | Complete Checker Task 264 `SourcePropertyImplementationHandoff` |
| Superseded candidate | A Task-264-specific Core-33 item association |
| Result | No Core-33 item exists for the property-implementation shell |
| Coverage | Zero semantic, execution, trace, metadata, and coverage credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapter 7 §§7.4.1 and 7.8.2 define a
property implementation as a definition supplying a virtual structure
property value over a mode domain. Chapter 5 keeps that property distinct from
constructor fields. Chapter 16 keeps existence, uniqueness, and coherence at
the correctness-obligation boundary. The accepted Core task graph assigns
checker Tasks 259--264 definition shells, bodies, and correctness references to
Core 36 after Core 33--35.

Fresh inventory after Task 33I263 finds no `spec_gap`. The prior suggestion of
a Task-264 Core-33 item association is bounded `design_drift`: it did not yet
classify the property-implementation shell's lack of semantic item identity.
The absent complete same-source Core 33--35 route remains `source_drift` and
`test_gap` owned by later executable tasks. There is no current
`source_undocumented_behavior`, `test_expectation_drift`, or boundary repair.

## Exact inventory and disposition

The protected means/equals sources have SHA-256
`cc90659f10cae4ef68890624df9b8b9d3f0e830dae5e20cc195dc8b263c5fa2b` and
`175135aaf40b9eab1a28e73ca1aae9f250e66278410d50575cdd279f6d7a2784`.
Their expectations have SHA-256
`bced77302602f43f3237424aa2963e5522c1458e879e606c68d1a516cd737c3a` and
`c491d7ea65e1c096d869af4666a06a053a5a0b213d9e79483d13e5ec91b75b6e`.
Checker Task 264 publishes exactly one implementation, parameter, target, and
definiens in each transaction, with two correctness rows for `means` and zero
for `equals`; only the means profile appends two pending initial obligations.

The resolver property-implementation shell is deliberately context-only. It
has no signature projection, `SymbolId`, `DefinitionId`, contribution, or
semantic origin. Its target is the existing `marker` selector symbol and
definition, and its parameter type refers to `Task264Carrier` at a use site.
Neither identity is the semantic identity of the implementation shell. The
checker handoff also does not publish a carrier structure-definition owner
range/provenance suitable for constructing a Core item.

`CoreItem` requires a whole `SymbolId`, while current `CoreItemKind` has no
property-implementation or selector item kind. Therefore the only fail-closed
disposition is:

1. do not create a Core item or association row for the Task-264 shell;
2. do not alias the target selector to `Functor`, `Structure`, or another item
   kind;
3. do not add a speculative `CoreItemKind` variant, synthetic symbol,
   definition identity, dependency, source range, or provenance;
4. leave Task 33I263 as the last current Task-specific Core-33 item-association
   prerequisite; and
5. consume Task 264 only in Core 36, after an exact same-source Core 33--35
   context and lower type/term/formula owners are available.

Current `CoreDefinition` requires a `CoreItemId` owner, so Core 36 cannot merely
associate a body with the target selector. The existing Task263 Core context is
for a different source and different structures; it is not a Task264 input.
Before Task264 lowering begins, a separately reviewed lower checker task must
publish the Task264 transaction's own `Task264Carrier` definition/member
identity as a syntax-free same-source handoff. A subsequent Core-33 carrier
context and Core-34 structure-member prerequisite must then publish an
authenticated selector-owner mapping compatible with `CoreDefinition.item`,
or a separately reviewed CoreIR representation task must change that owner
model. This contract selects neither representation and does not authorize a
property-implementation-shell item. Core 36 is hard-blocked while this chain
is incomplete; it must not reconstruct an owner from names, ranges, numeric
ids, spelling, or another source's context.

Once that distinct owner prerequisite exists, the future Core-36 task may
associate the definition body and correctness references with the
authenticated target selector and domain, but it must not publish an accepted
property value, fact, axiom, proof, or discharged obligation.

## Scope, artifacts, and verification

This task changes only the paired contract; paired Core plan, TODO, ledger,
source-family decomposition, and bilingual audit; and
`doc/design/spec_coverage_audit.md`. It changes no checker or mizar-test design
document because their completed Task-264 contract and executable coverage do
not change. Contract trees project `115/115 -> 116/116`.

Protected source includes
`crates/mizar-checker/src/source_property_implementation.rs` at `2460 / 89030`,
SHA-256 `82a9c45e8a7201e85afe961aefde74f35dd49dac359d4be51062d507294b08ee`,
and `crates/mizar-core/src/core_ir.rs` at `4016 / 132375`, SHA-256
`4458bc2353c437d4427b39f96e0041bf1c321e19cff0ec4565c3f50084f83c4c`.
The protected trace SHA-256 is
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`;
the protected stash is `f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

Independent specification/equivalence, test-sufficiency, implementation/
boundary, source/documentation, and bilingual reviews ended with no findings
after repairing the Task Index ownership, `CoreDefinition.item` prerequisite,
and different-source successor error. Core lint passes `12/12`, mizar-test lint
passes `15/15` including recursive contract/link checks, and `git diff --check`
passes. Broad Rust suites were not rerun because this docs-only task changes no
source or test behavior. Final read-only quality found no findings, all nine
hard gates pass, and the valid uncapped score is `99/100`. Exact staging, the
docs-only commit, clean postcommit state, and fresh successor inventory remain
transactional exit steps.

## Audit impact and next handoff

The central audit records only corrected Core-33/Core-36 follow-up ownership.
No specification chapter, test, trace backlink/status, metadata count, runner
selection, or coverage credit changes.

After this task commits, fresh inventory selects the smallest checker-owned
lower `source_drift` prerequisite that can publish `Task264Carrier` plus its
`carrier`/`marker` member identities from the existing Task264 source without
changing test intent or semantics. Only its separately committed output may
feed a Task264 same-source Core-33 carrier context and Core-34 selector/type-
owner prerequisite. Task 264 remains parked and hard-blocked at Core 36 until
that chain exists; Task263 remains a protected different-source precedent, and
Task277B remains not-ready/zero-credit.
