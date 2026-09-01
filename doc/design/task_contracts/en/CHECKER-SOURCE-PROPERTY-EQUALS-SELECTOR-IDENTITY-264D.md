# Task CHECKER-SOURCE-PROPERTY-EQUALS-SELECTOR-IDENTITY-264D: Task264 equals selector identity

> Canonical language: English. Japanese companion:
> [../ja/CHECKER-SOURCE-PROPERTY-EQUALS-SELECTOR-IDENTITY-264D.md](../ja/CHECKER-SOURCE-PROPERTY-EQUALS-SELECTOR-IDENTITY-264D.md).

Status: complete on task commit. This is a representation-only, zero-semantic,
zero-credit checker prerequisite. It changes no language
behavior, diagnostic, obligation, accepted test intent, trace status, metadata,
or coverage credit.

## Identity, authority, and classification

| Field | Frozen value |
|---|---|
| Task | `CHECKER-SOURCE-PROPERTY-EQUALS-SELECTOR-IDENTITY-264D` |
| Primary owner | `mizar-checker::source_property_implementation` |
| Required predecessor | Task264C carrier identity and the complete Task264 equals transaction |
| Inputs | Task264 `SymbolEnv` plus complete property, Task252 primary-term, and Task254 structure handoffs |
| Consumer | Task264 Core35 equals body input after a separate parameter/Core-variable prerequisite |
| Owning plan | [`mizar-checker` Task Index](../../mizar-checker/en/00.crate_plan.md#task-index) |
| Coverage | Zero semantic, execution, trace, metadata, and coverage credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapter 7 fixes equals as a direct-term
definition form; the existing Task264 `.miz` fixes that term as `M.carrier`.
The checker transaction already authenticates property parameter binding 0,
the equals structure term/member/base graph, and the local `carrier` field
resolver identity. This task only materializes their missing typed association.

There is no `spec_gap`. The unassociated Task254 `MemberIdentity` request is a
bounded `source_drift`; absent public association assertions are a bounded
`test_gap`. Core's inability to form a selector term without this edge remains
`design_drift` until this task completes. All other gap classes are absent.

## Frozen API and exact association

Add one branded immutable association, handoff, error, and producer beside the
existing Task264 owner. All fields are private; all three handoff dependencies
are retained by value, while `SymbolEnv` is borrowed for construction only:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorAssociation { /* private fields */ }

impl SourcePropertyEqualsSelectorAssociation {
    pub const fn implementation(&self) -> SourcePropertyImplementationId;
    pub const fn definiens(&self) -> SourcePropertyDefiniensId;
    pub const fn structure_term(&self) -> SourceStructureTermId;
    pub const fn member(&self) -> SourceStructureMemberId;
    pub const fn member_request(&self) -> SourceStructureRequestId;
    pub const fn base_edge(&self) -> SourceStructureEdgeId;
    pub const fn base_term(&self) -> SourcePrimaryTermId;
    pub const fn base_reference(&self) -> SourcePrimaryTermReferenceId;
    pub const fn base_binding(&self) -> BindingId;
    pub const fn selector_symbol(&self) -> &SymbolId;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorIdentityHandoff { /* private fields */ }

impl SourcePropertyEqualsSelectorIdentityHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn property(&self) -> &SourcePropertyImplementationHandoff;
    pub const fn terms(&self) -> &SourcePrimaryTermHandoff;
    pub const fn structures(&self) -> &SourceStructureHandoff;
    pub const fn association(&self) -> &SourcePropertyEqualsSelectorAssociation;
    pub fn debug_text(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyEqualsSelectorIdentityError {
    EnvironmentMismatch,
    UnsupportedProfile,
    DependencyMismatch,
    InvalidSelectorIdentity,
}

pub struct SourcePropertyEqualsSelectorIdentityProducer;

impl SourcePropertyEqualsSelectorIdentityProducer {
    pub fn build(
        env: &SymbolEnv,
        property: SourcePropertyImplementationHandoff,
        terms: SourcePrimaryTermHandoff,
        structures: SourceStructureHandoff,
    ) -> Result<SourcePropertyEqualsSelectorIdentityHandoff,
                SourcePropertyEqualsSelectorIdentityError>;
}
```

The exact association is
`implementation/definiens/structure-term/member/member-request/base-edge/
base-term/base-reference/base-binding = 0/0/0/0/0/0/0/0/0`. Its `selector_symbol` is the
whole `carrier_identity().field_symbol()` value. Construction uses the supplied
resolver environment to create the missing result edge. Construction proves,
and replay preserves in the private retained receipt, that:

- the environment and all three handoffs share one module/source transaction;
- the property is exactly the equals profile and its complete term/structure
  fingerprints equal the supplied handoffs;
- definiens 0 targets structure term 0;
- structure term 0 is normal `M.carrier` `SelectorAccess`, member 0 is its sole
  `Selector`, and request 0 is its matching `MemberIdentity` request;
- the sole `SelectorBase` edge targets primary term 0;
- primary term 0 is normal value-position variable `M`, reference 0 targets it
  with variable role/binding 0, and that binding equals parameter 0 and target
  0's subject; and
- the exact environment field symbol/definition/contribution is a normal local
  public/exported `Selector`, its primary spelling equals the selector member,
  its origin is the retained field origin, and it is in the sole local-source
  contribution effects; and
- the private association reproduces every exact id and that resolver result's
  whole field symbol.

Validation order is environment, supported equals profile, dependency
fingerprints, then association. Means returns `UnsupportedProfile`; mixed and
foreign inputs fail before publication. This is one bounded Task264 resolver
result, not generic Task254 request resolution. Spelling/shape selects the
exact occurrence, but cannot authenticate the result alone; the supplied
`SymbolEnv`, carrier-domain receipt, whole symbol/definition/contribution, and
normal provenance jointly authenticate the published edge. Replay has no
`SymbolEnv` and relies on private-field immutability plus the retained exact
resolver receipt; it does not claim an independent lower identity oracle.

`debug_text()` uses
`source-property-equals-selector-identity-debug-v1` and includes module, three
retained fingerprints, all nine scalar ids, and the whole selector FQN. It is
deterministic evidence only, never an identity oracle.

## Scope, tests, and forbidden behavior

Rust source edits are limited to the checker owner and its existing unit-test
support plus the existing private Task264 runner assertion leaf. Tests cover
equals construction/replay and exact getters/debug; means rejection;
same-environment profile mixing; foreign transactions; and internal malformed
fingerprint, term, reference, member, edge, request, binding, and retained
symbol associations. Existing `.miz`, expectations, trace metadata, runner
selection, Typed/Resolved slots, and Task264 owner debug bytes remain unchanged.

Do not add a generic resolved-member table, property/field Core item, Core
term/formula/definition, Core variable, normalized type/guard, `it`
representation, property-value evaluation, correctness/coherence seed,
diagnostic, obligation, proof, VC, active route, or coverage credit. Do not edit
`doc/spec`, existing `.miz`, expectations, trace metadata, or snapshots.

## Artifacts, baselines, reviews, and exit

Durable owners are this paired contract; paired checker plan, property design,
TODO, source/spec, bilingual, and module-boundary records; paired Core
decomposition/readiness records; paired mizar-test private-harness records; and
the central coverage audit. Stable owner sections are the checker property
design's [equals selector identity association](../../mizar-checker/en/source_property_implementation.md#equals-selector-identity-association)
and the Core decomposition's [Task264 Core35 readiness entry](../../mizar-core/en/source_family_decomposition.md#task-264d-task264-core35-selector-readiness).

Clean baseline is HEAD `b73bf407aa9382f0245a997877e8f66d7640b982`.
Checker owner source is `2625 / 94187`, SHA-256
`0a0f9d887aa6cda7ef11c18936cd27503326c587ad7c8bc3193565828e91fe58`;
checker unit support is `2152 / 77946`, SHA-256
`7a7ddb8730f0a39d5739a11a3a2ca094e3997a446fb5b8556b168e34cc48b54d`;
the private Task264 leaf is `1064 / 46418`, SHA-256
`b474a1bc55997d79fe1a0e83ee194c25c28ee06a0f86f6a0abaa8b9f8bcf5b4d`;
the central audit is `7432 / 561797`, SHA-256
`4f8b5be69030061211b6b6ea87a1febcda3f390c7aaf099eba46b2b960f3b197`.
Contract trees project `121/121 -> 122/122`; checker/Core/mizar-test unit
counts begin at `580/164/646`. Protected source, expectation, and trace hashes
remain those recorded by Task34D264, and protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` remains untouched.

Post-source measured artifacts are the checker owner `3095 / 113368`, SHA-256
`93590a77a860bcc2bf229ac7ecc369770df0f4b36af363e4c42fddead26ab82a`;
checker unit support `2516 / 91066`, SHA-256
`621a904bbda4613fca53845a4c74f48a9114ac6a71484cc921798f9b62dec090`;
the private Task264 leaf `1143 / 49788`, SHA-256
`c5d746b3d16bca7088aedfc3a31a286a84737790f8936d1ff48488a95e0b196d`;
and the central audit `7444 / 562566`, SHA-256
`9da542688b7e83bdaa4204576ee061aef36080a4c2f49f52163b6bfe40ad9de5`.
Contract trees are `122/122`; checker/Core/mizar-test unit counts are
`582/164/646`.

Independent pre-source specification/API and bilingual/boundary reviews must
end with no blocking finding. Post-source test sufficiency,
implementation/default-deny, source/documentation/API, bilingual/boundary, and
final read-only quality reviews must have no blocking/high/medium finding. Run
focused checker/private Task264 tests, affected package/lint/metadata suites,
formatting, warnings-denied all-feature Clippy, and enlarged-stack all-feature
workspace tests/doctests. Exit requires hard gates `9/9`, quality at least
`90/100`, a task-only commit, clean postcommit proof, and fresh inventory for
the Task264 parameter/Core-variable context prerequisite.

## Completion evidence

Independent pre-source specification/API and bilingual/boundary reviews ended
with no blocking/high/medium finding after the construction-only `SymbolEnv`,
public-enum policy, authority wording, Core readiness links, and EN/JA status
were repaired. Post-source test sufficiency and implementation/default-deny
reviews likewise ended with none after adding the base-edge coordinate, exact
`3/3/1` environment authentication, same-module foreign-source precedence,
resolver-provenance matrix, and corruption coverage for every retained
lower coordinate except the owner-fixed implementation/definiens pair, which
is positively asserted and replay-validated. Source/documentation/API and
bilingual/boundary review ended with none after the public inventory, measured
layout, legacy-compaction topology, canonical marker, and lifecycle state were
synchronized.

Focused checker Task264D tests pass `2/2`; the complete source-property family
passes `7/7`; the private Task264 selector/type tests pass `2/2`. Checker passes
`582` unit and `16` lint tests, Core remains `164`, and mizar-test passes `646`
unit, `3` layout, `15` lint, `137` metadata, `2` public-enum, and `21` snapshot
tests. Formatting, offline metadata, `git diff --check`, warnings-denied
all-target/all-feature Clippy, the coverage plan (`430` cases, `396`
requirements, zero errors, the unchanged `23` warnings), and enlarged-stack
all-feature workspace tests/doctests pass. Protected `.miz`, expectations,
trace metadata, and stash retain their frozen hashes. Parent hard gates pass
`9/9`. The independent final read-only audit found no finding, applied no
score cap, passed hard gates `9/9`, and assigned a valid `100/100`; only the
task-only commit plus clean postcommit proof remains as operational closure.
