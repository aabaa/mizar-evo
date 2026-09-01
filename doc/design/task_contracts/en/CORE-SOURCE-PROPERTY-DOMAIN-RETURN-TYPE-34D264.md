# Task CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264: Task264 domain/return type input

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264.md](../ja/CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264.md).

Status: complete on task commit. This is a representation-only,
zero-semantic, zero-credit Core-34 prerequisite. It changes no language
behavior or protected `.miz`/expectation/trace test intent. It extends only
derived Rust validation-test intent and changes no diagnostic, obligation,
metadata, or coverage credit.

## Identity, authority, and readiness

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-DOMAIN-RETURN-TYPE-34D264` |
| Primary owner | `mizar-core::elaborator`, Core Task 34 |
| Required predecessors | Task264C `3cb1b31c`; Task33I264 `0f61a860`; Task34I264 `85648a07`; IR264 `e96e12d1` |
| Input | Complete `SourcePropertySelectorTypeContextHandoff` |
| Consumer | A separately reviewed Core35 Task264 term/formula body input, then Core36 |
| Owning plan | [mizar-core Task Index](../../mizar-core/en/00.crate_plan.md#task-index) |
| Coverage | Zero semantic, execution, trace, metadata, and coverage credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapters 5 and 7 and the two existing
Task264 `.miz` sources fix one implementation parameter `M` over the declared
`Task264Carrier` domain and the existing `marker -> set` property return. The
checker handoffs already authenticate parameter binding 0, written-type
application 0/root 0, property target 0, and return member 1/root 2.

There is no `spec_gap`. The missing durable Core-facing domain relation is
bounded `design_drift` and `source_drift`; extending the existing private
Task264 probe closes its bounded `test_gap`. The established task split owns
source-derived types in Core34, terms/formulas in Core35, and definitions in
Core36. Earlier derived wording that assigned domain/return types to Core35 is
repaired to that split; this does not change language or test intent.

## Frozen public API and validation

Add exactly this public surface to `mizar-core::elaborator`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyDomainTypeAssociation { /* private fields */ }

impl SourcePropertyDomainTypeAssociation {
    pub const fn binding(&self) -> BindingId;
    pub const fn application(&self) -> SourceTypeApplicationId;
    pub const fn root(&self) -> SourceTypeExpressionId;
    pub const fn carrier_item(&self) -> CoreItemId;
}

impl SourcePropertySelectorTypeContextHandoff {
    pub const fn domain(&self) -> &SourcePropertyDomainTypeAssociation;
}
```

The existing handoff gains one private `domain` field and retains the complete
carrier and source-type inputs by value. Its existing
`SourcePropertySelectorTypeAssociation` remains the return-side association.
No constructor, table, public mutator, or duplicate return API is added, and
the existing `source-property-selector-type-context-v1` debug bytes remain
unchanged.

Construction and postvalidation derive exactly one domain row by following
checker parameter 0's binding 0 and written type application 0 to application
0/root expression 0. Property target 0's subject must equal that same parameter
binding 0 before either the domain or return association is published. The
domain expression must remain the same-source whole
`Task264Carrier` symbol/contribution authenticated by the checker identity,
and the Core item registry must map that exact symbol to the retained
Task33I264 carrier item 0. The association is exactly
`binding/application/root/carrier-item = 0/0/0/0` for both Task264 profiles.
The existing return association remains exactly property symbol/member/root
`marker/1/2`. Validation uses typed ids and whole symbol identity, never
spelling, range, numeric id, FQN, debug text, or iteration order alone.

## Scope, tests, and forbidden behavior

Source edits are exactly `crates/mizar-core/src/elaborator.rs` and the existing
private Task264 assertion leaf. Extend the existing means/equals positive test
to assert the parameter-to-target subject edge, domain relation, registry join,
retained return relation,
deterministic replay, and unchanged debug bytes. The existing cross-profile
and foreign-environment test remains the fail-closed transaction proof. No new
`.miz`, expectation, trace, snapshot, test selector, or public test-only
mutator is permitted.

Do not add or infer a `CoreTypePredicate`, normalized type string, type guard,
binder, type fact, field-to-member-0 association, item, dependency, term,
formula, definition, correctness/coherence seed, diagnostic, obligation,
production route, or coverage credit. Do not lower `it`, `M.carrier`, means,
equals, correctness, coherence, proof, acceptance, or discharge. Task263 and
Task248/33LB are not inputs. `doc/spec`, existing `.miz`, expectations, trace
metadata, and protected artifacts remain unchanged.

## Artifacts, reviews, verification, and handoff

Derived owners are this paired contract; paired Core plan, elaborator,
decomposition, source/spec, TODO, ledger, bilingual, and boundary records;
the paired mizar-test harness records; and the central coverage audit.
`core_ir.rs`, checker source/design, mizar-vc, and language artifacts are
read-only dependencies.

Stable owner sections are the
[elaborator API/invariants](../../mizar-core/en/elaborator.md#task-34d264-task264-domainreturn-type-input),
[source/spec mapping](../../mizar-core/en/source_spec_audit.md#core-34d264-task264-domainreturn-type-input-mapping),
and [private harness probe](../../mizar-test/en/harness.md#core-task-34d264-private-task264-domainreturn-type-probe).
The exact source-write set is `crates/mizar-core/src/elaborator.rs` and the
existing Task264 private assertion leaf. The paired owner documents above,
crate-plan/TODO/ledger/decomposition/bilingual/boundary records, and central
audit form the derived documentation-write set. `core_ir.rs`, all checker and
VC files, `doc/spec`, `.miz`, expectations, trace metadata, snapshots, and
protected artifacts are explicit no-impact/read-only surfaces.

Clean baseline is HEAD `e96e12d1767ab1d6a85e881328d5965e1afa15d1`.
`core_ir.rs` is `4393 / 146011`, SHA-256
`4e614a6ee98d0ef6b93dcd5d708728e41b79f613b16880269550051450793fd1`;
`elaborator.rs` is `23685 / 890564`, SHA-256
`1d78d960032e2f4086f712d258a8ec247aa12daeff88f51c6afe8f4d880a7162`;
the private Task264 leaf is `1022 / 44699`, SHA-256
`e584e3a36d8c8911d4e5f49209128cb35e81d0c93d254419476b93557a86fdca`;
the central audit is `7413 / 560700`, SHA-256
`a085dc14b0479cfab399ce5b594134b812b094b71d0885f948aa1ec1bea0f40a`.
Contract trees project `120/120 -> 121/121`; Core/checker/mizar-test tests
remain `164/580/646`. Protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` remains untouched.

Post-source measured artifacts are `elaborator.rs` `23785 / 893870`, SHA-256
`f65c64b2a59ab68689b6a53e0c334b231545cfcfe33f6b5e178ce18ebe3d7928`;
the private Task264 leaf `1064 / 46418`, SHA-256
`b474a1bc55997d79fe1a0e83ee194c25c28ee06a0f86f6a0abaa8b9f8bcf5b4d`;
and the central audit `7432 / 561797`, SHA-256
`4f8b5be69030061211b6b6ea87a1febcda3f390c7aaf099eba46b2b960f3b197`.
Contract trees are `121/121`; test counts remain `164/580/646`. These
source/audit measurements are final for the frozen write set; review and gate
results are recorded separately at closure.

Independent pre-source specification/API and bilingual/boundary reviews must
end with no blocking finding. Post-source test sufficiency,
implementation/default-deny, source/documentation/API, bilingual/boundary,
and final quality reviews must end with no blocking/high/medium finding. Run
the focused Task264 probe, affected package/lint/metadata suites, fmt, offline
metadata, warnings-denied all-feature Clippy, and enlarged-stack all-feature
workspace tests/doctests. Hard gates must pass `9/9`, quality must be at least
`90/100`, task-only commit and clean postcommit proof must close the task.

After completion, fresh inventory may freeze the smallest Core35 Task264 body
input. It must separately decide the exact Core variable representation for
parameter `M` and current-definition-result `it`; Core36 definition and
correctness/coherence remain deferred. Task277B remains not-ready.

## Completion evidence

Pre-source specification/API and bilingual/boundary reviews ended with no
remaining blocking/high/medium finding after the target-subject edge, ordered
plan linkage, stable owner links, exact no-impact map, and protected-versus-
derived test-intent wording were repaired. Post-source test-sufficiency,
implementation/default-deny, source/documentation/API, and bilingual/boundary
reviews likewise ended with none after active-state and measured-evidence
wording were synchronized.

Focused Task264 selector/type tests pass `2/2`. Core passes `164` unit, `2`
determinism, and `12` lint tests. mizar-test passes `646` library, `3` layout,
`15` lint, `137` metadata, `2` public-enum, and `21` snapshot tests. Formatting,
offline metadata, `git diff --check`, warnings-denied all-target/all-feature
Clippy, and enlarged-stack all-feature workspace tests/doctests pass. The
protected language/test artifacts, checker, VC, and `core_ir.rs` have no diff;
the frozen `core_ir.rs` hash and protected stash remain unchanged. Parent hard
gates pass `9/9`; the independent final result is recorded below.

The initial final-quality audit found only stale lifecycle evidence and held
gate 5/score invalid. This paired completion record and synchronized active-
owner statuses repaired that documentation-only finding. The finding-specific
read-only recheck found no blocking/high/medium finding, passed hard gates
`9/9`, assigned valid uncapped quality `100/100` with no score cap, and left
only task-only commit plus clean postcommit proof as operational closure.
