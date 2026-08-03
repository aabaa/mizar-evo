# Source Attribute-Definition Transport

> Canonical language: English. Japanese companion:
> [../ja/source_attribute_definition.md](../ja/source_attribute_definition.md).

Status: frozen Checker Task 261 documentation prerequisite. No executable
artifact or coverage status changes in this prerequisite.

## Authority, Classification, And Scope

Checker Task 261 owns one syntax-free immutable intake for an ordinary,
unparameterized `attr ... means ...` definition. Its canonical authority is:

- Chapter 6 Sections 6.1, 6.2, 6.8.1, and 6.9 for attribute purpose, the
  mandatory label and subject, the optional pattern prefix, the ordinary
  attribute name, the formula definiens, and predicate-style FOL identity;
- Chapter 16 Sections 16.6 and 16.7.2 for definition-time correctness
  boundaries, including the fact that the only specified attribute-specific
  obligation row is redefinition `coherence`;
- the existing parser attribute-definition pass/recovery fixtures and tests,
  the active
  `fail_type_elaboration_attribute_definition_gap_001.miz` boundary and its
  sidecar/trace rows, and resolver declaration/signature tests; and
- the completed public Checker Tasks 248, 249, 252, and 256 transports plus
  the Task-259/260 definition-family ownership boundary.

The missing frozen contract is nonblocking `design_drift`; the absent
checker-owned producer is `source_drift`; and the absent exact real consumer
is a `test_gap`. There is no blocking `spec_gap`. Chapter 16 does not specify
an ordinary-attribute initial-obligation kind or goal. Task 261 therefore
adds no `InitialObligationTable` row and does not infer one from parser
correctness-clause support or current source behavior. Attribute formula
checking, definitional equivalence, acceptance, and fact publication remain
semantic deferrals.

Task 261 transports only definition identity, its two definition-local
parameters, the subject-to-binding link, the already produced equality
formula definiens, and resolver/lower provenance. It does not own a
parameter-prefix attribute, attribute use-site application, redefinition,
coherence, a case/otherwise definiens, formula truth, proof, acceptance,
cluster activation, fact/axiom publication, overload selection, IR, or VC.

## Frozen Exact Future Source

The dedicated future pass source is exactly these 116 UTF-8 bytes, including
the final LF:

```mizar
definition
  let x be set;
  let y be set;
  attr Task261AttributeDefinition: x is task261_marked means x = y;
end;
```

Its SHA-256 is
`ffd4954aad628d7946aaf7afb1b472a6bdfca7bce5ba0cf09f5b284c9dda07bf`.
It contains one normal definition block, two separately written builtin-set
parameters, one ordinary unparameterized attribute definition, subject `x`,
attribute spelling `task261_marked`, and one equality formula definiens. It
contains no `assume`, prefix parameter, argument-list use, `non`, qualifier,
redefinition, correctness clause, conditional branch, `otherwise`, import,
reserve, theorem, proof block, or recovery.

This source is derived from Chapter 6's ordinary definition syntax and closes
the classified exact `test_gap`. It intentionally reuses the already frozen
two-parameter lower profile. The existing 91-byte one-parameter `thesis`
fixture, sidecar, expectation, and trace rows remain byte-unchanged on their
broader extraction gap; Task 261 does not retrofit unsupported Task-248 or
formula-constant lower shapes merely to reclassify that historical fixture.

## Frozen Surface Profile

A read-only parser probe produces zero diagnostics, exactly 45 dense Surface
rows, root node 44, root range `0..115`, no expression root, and no recovery.
All rows, token spellings, ranges, recovery flags, and ordered children are a
literal private-runner oracle. The structurally relevant rows are:

| Node | Surface kind | Range | Task-261 role |
| ---: | --- | --- | --- |
| 24/25 | `TypeHead` / `TypeExpression` | `22..25` | parameter `x` written type |
| 27 | `DefinitionParameter` | `13..26` | first context parameter |
| 28/29 | `TypeHead` / `TypeExpression` | `38..41` | parameter `y` written type |
| 31 | `DefinitionParameter` | `29..42` | second context parameter |
| 32 | `AttributePattern` | `83..97` | unparameterized `task261_marked` pattern |
| 33/34 | `TermReference` / `TermExpression` | `104..105` | left definiens operand `x` |
| 35/36 | `TermReference` / `TermExpression` | `108..109` | right definiens operand `y` |
| 37/38/39 | equality / formula expression / formula definiens | `104..109` | exact formula body |
| 40 | `AttributeDefinition` | `45..110` | definition, subject, pattern, and body owner |
| 41 | `DefinitionBlockItem` | `0..115` | common parameter/definition owner |
| 44 | `Root` | `0..115` | complete Surface root |

The parameter declaration ranges are `17..18` and `33..34`; the definition
label is `50..76`; the subject token is `78..79`; and the attribute pattern is
`83..97`. Nodes 27, 31, and 40 are normal direct structural siblings of block
41 in that order. The private runner authenticates the complete source bytes,
final LF, all 45 rows, root, sibling order, containment, and subtree
partition. The checker receives no raw node number, `SurfaceAst`, syntax kind,
or parser token.

Pattern name and definition label descendants are excluded from Task-252/256
term/formula discovery. Only equality subtree 37 is the definiens lower root.

## Frozen Raw Resolver Provenance

The exact resolver result has two declaration shells, one signature
projection, zero symbol diagnostics, one attribute symbol, one attribute
definition, and one local-source contribution:

- shell 0 is `DefinitionBlock`, node/range `41/0..115`, ordinal 0, no parent;
- shell 1 is `AttributeDefinition`, node/range `40/45..110`, ordinal 1,
  parent shell 0;
- definition 0 is `SymbolKind::Attribute` / `DefinitionKind::Attribute`,
  spelling and notation `task261_marked`, structural path `[4,0,7,0]`, public,
  local, exported, overloadable, conflict-free, and contribution 0; and
- its opaque `parser-signature-v1` payload records representation provenance
  only and is never parsed by the checker.

Resolver parameters, binders, and syntactic arity are empty. Task 261 must not
infer the two context parameters, subject binding, pattern-prefix arity,
formula root, or correctness status from those empty fields or opaque text.
The exact private source oracle and lower handoffs own those associations.

## Frozen Lower Bundle

The exact source consumes only completed syntax-free lower transports:

| Owner | Exact active profile | Task-261 ownership use |
| --- | --- | --- |
| Task 248 | Profile B `1/2/2/2/2/2/0` | block context and ordered `x`/`y` bindings |
| Task 249 | `2/2/0` | the two binding-linked builtin-set type applications/expressions |
| Task 252 | `2/2/0` | the equality operands `x`, `y` and their binding references |
| Task 256 | `1/0/0/0/0/0/0/2/2` | one equality, two operand edges, two expected-type requests |
| Tasks 249R/250/251/253--255/257--258 | absent | no return, attribute use, evidence, term-root alternative, or broader formula |
| Tasks 259/260 | absent and isolated | no predicate/functor definition transaction |

Task 261 constructs the exact Task-248 Profile-B input locally from rows
27/31 and shell 41, just as Task 260 did for its own authenticated source. It
does not modify or generalize either Task-259/260 private helper or the public
Task-248 producer. Task-249 applications 0/1 correspond to the two written
types. Task-252 references 0/1 are the body operands and link bindings 0/1.
Task-256 formula 0 is the entire definiens.

All four lower debug fingerprints participate in Task-261 identity. Missing,
extra, stale, or cross-source lower handoffs fail before publication.

## Frozen Public Syntax-Free Contract

Implementation adds `source_attribute_definition.rs` with four dense ID
families. Each is `Copy + Eq + Ord + Hash`, exposes only `new` and `index`, and
is allocated by vector order:

```rust
pub struct SourceAttributeDefinitionId(usize);
pub struct SourceAttributeParameterId(usize);
pub struct SourceAttributeSubjectId(usize);
pub struct SourceAttributeDefiniensId(usize);
```

The exact input is:

```rust
pub struct SourceAttributeDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceAttributeDefinitionInput>,
    pub parameters: Vec<SourceAttributeParameterInput>,
    pub subjects: Vec<SourceAttributeSubjectInput>,
    pub definientia: Vec<SourceAttributeDefiniensInput>,
}

pub struct SourceAttributeDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceAttributeDefinitionRecovery,
    pub spelling: String,
    pub subject: SourceAttributeSubjectId,
    pub definiens: SourceAttributeDefiniensId,
}

pub struct SourceAttributeParameterInput {
    pub owner: SourceAttributeDefinitionId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceAttributeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceAttributeSubjectInput {
    pub owner: SourceAttributeDefinitionId,
    pub binding: BindingId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceAttributeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceAttributeDefiniensInput {
    pub owner: SourceAttributeDefinitionId,
    pub ordinal: usize,
    pub formula: SourceAtomicFormulaId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceAttributeDefinitionRecovery,
    pub spelling: String,
}

#[non_exhaustive]
pub enum SourceAttributeDefinitionRecovery { Normal, Degraded }
```

All input structs derive `Debug + Clone + PartialEq + Eq`; recovery derives
`Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`. The exact
immutable row types and stored fields are:

| Row type | Stored fields, in API order |
| --- | --- |
| `SourceAttributeDefinition` | `id`, `symbol`, `definition`, `contribution`, `site`, `source_range`, `source_ordinal`, `context`, `recovery`, `spelling`, `subject`, `definiens`, derived `origin` |
| `SourceAttributeParameter` | `id`, `owner`, `ordinal`, `binding`, `written_type`, `site`, `source_range`, `declaration_range`, `context`, `recovery`, `spelling` |
| `SourceAttributeSubject` | `id`, `owner`, `binding`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourceAttributeDefiniens` | `id`, `owner`, `ordinal`, `formula`, `site`, `source_range`, `context`, `recovery`, `spelling` |

Every stored field has one same-named read-only getter. Dense and resolver
IDs, ordinals, ranges, contexts, and recovery return by value; `symbol`,
`site`, and derived `origin` return by shared reference; `spelling()` returns
`&str`. Rows have no public constructors, setters, mutable getters, or
replacement APIs.

The exact table and handoff surface is:

```rust
pub struct SourceAttributeDefinitionTable { /* private rows */ }
pub struct SourceAttributeParameterTable { /* private rows */ }
pub struct SourceAttributeSubjectTable { /* private rows */ }
pub struct SourceAttributeDefiniensTable { /* private rows */ }

pub struct SourceAttributeDefinitionHandoff { /* private fields */ }

impl SourceAttributeDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn source_context_fingerprint(&self) -> &str;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn source_term_fingerprint(&self) -> &str;
    pub fn source_atomic_formula_fingerprint(&self) -> &str;
    pub const fn definitions(&self) -> &SourceAttributeDefinitionTable;
    pub const fn parameters(&self) -> &SourceAttributeParameterTable;
    pub const fn subjects(&self) -> &SourceAttributeSubjectTable;
    pub const fn definientia(&self) -> &SourceAttributeDefiniensTable;
    pub fn debug_text(&self) -> String;
}
```

Each table exposes only `get(id) -> Option<&Row>`, source-ordered
`iter() -> impl Iterator<Item = (Id, &Row)>`, `const len() -> usize`, and
`const is_empty() -> bool`. The four fingerprints are derived by the producer
from the complete Task-248/249/252/256 `debug_text()` strings after shared
source/module/arena authentication. Callers cannot supply fingerprints.

The exact producer and error API is:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceAttributeDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidSubject { index: usize },
    InvalidDefiniens { index: usize },
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

pub struct SourceAttributeDefinitionProducer;

impl SourceAttributeDefinitionProducer {
    pub fn build(
        input: SourceAttributeDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        source_atomic_formula: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<
        SourceAttributeDefinitionHandoff,
        SourceAttributeDefinitionError,
    >;
}
```

`SourceAttributeDefinitionError` implements `Display` and `Error`, has no
`Default` or blanket conversion, and is fail-closed. Rows, tables, and the
handoff derive `Debug + Clone + PartialEq + Eq`; the producer is a unit struct.
The definition's `SemanticOrigin` and all dense IDs are derived and cannot be
caller supplied.

## Public Enum Policy And Debug Grammar

| Public enum | Compatibility policy |
| --- | --- |
| `SourceAttributeDefinitionRecovery` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen recovery classes. |
| `SourceAttributeDefinitionError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

There are no exhaustive public-enum exceptions. Stable row-family keys are
`source.definition.attribute`, `source.definition.attribute.parameter`,
`source.definition.attribute.subject`, and
`source.definition.attribute.definiens`.

`debug_text()` is the dependency fingerprint and emits exactly this family
order, one final LF, and no blank line. `Rust-debug` means standard escaped
`{:?}` rendering:

```text
source-attribute-definition-debug-v1
module: <ModuleId.path>
source-context-fingerprint: <Rust-debug String>
source-type-fingerprint: <Rust-debug String>
source-term-fingerprint: <Rust-debug String>
source-atomic-formula-fingerprint: <Rust-debug String>
definition#<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> ordinal=<n> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> origin_range=<start>..<end> origin_path=<Rust-debug [u32]> spelling=<Rust-debug String> subject=<id> definiens=<id>
parameter#<id> owner=<id> ordinal=<n> binding=<id> written_type=<id> range=<start>..<end> declaration_range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
subject#<id> owner=<id> binding=<id> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
definiens#<id> owner=<id> ordinal=<n> formula=<id> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
```

The admitted profile accepts only `TypedSiteRef::Node`, local unrecovered
resolver origins, and `Normal` rows. Role sites, imported/recovered origins,
and extra grammar branches fail closed. Typed and final debug include the
complete string exactly once; legacy empty rendering remains byte-identical.

## Exact Four-Table Oracle

The table cardinality is exactly `1/2/1/1` in
definition/parameter/subject/definiens order:

- definition 0 authenticates resolver definition/symbol/contribution 0,
  site 40, ordinal 0, range and local origin `45..110`, origin path
  `[4,0,7,0]`, context 1, normal recovery, spelling
  `attr Task261AttributeDefinition: x is task261_marked means x = y;`, subject
  0, and definiens 0;
- parameters 0/1 belong to definition 0 and preserve bindings 0/1,
  Task-249 applications 0/1, sites 27/31, ordinals 0/1, owner ranges
  `13..26` / `29..42`, declaration ranges `17..18` / `33..34`, context 1,
  and spellings `let x be set;` / `let y be set;`;
- subject 0 belongs to definition 0, links binding 0, uses definition site 40
  with token range `78..79`, context 1, normal recovery, and spelling `x`; and
- definiens 0 belongs to definition 0, has ordinal 0, links Task-256 atomic
  formula 0, site 39, range `104..109`, context 1, normal recovery, and
  spelling `x = y`.

There is no guard, property, correctness, prefix-parameter, attribute-use, or
initial-obligation row. Task 261 uses a strict no-read/no-write obligation
contract: its input, producer, handoff, and installer neither receive nor
inspect an `InitialObligationTable`, and they expose no obligation projection.
Installing the optional handoff changes only that option field, so every
preexisting obligation row and ID remains byte-identical; an error publishes
nothing.

## Validation, Fingerprints, And Semantic Boundary

`SourceAttributeDefinitionProducer::build` atomically validates the complete
syntax-free input against `SymbolEnv`, the Task-248 source context, Task-249
type handoff, Task-252 primary-term handoff, Task-256 atomic-formula handoff,
and typed arena. It returns the immutable handoff or no output.

Validation rejects missing, duplicate, reordered, dangling, cross-owner,
cross-module, recovered/degraded, stale-site, stale-range, stale-context,
stale-origin, stale-symbol/definition/contribution, stale-binding, stale-type,
stale-formula, stale-fingerprint, wrong spelling/ordinal/kind, partial, or
extra rows. Resolver `SymbolEntry`, `DefinitionEntry`, and contribution must
all match. Input is never sorted or repaired.

The body is only an occurrence link to `SourceAtomicFormulaId(0)`. Task 261
does not evaluate `x = y`, construct the attribute's FOL biconditional, check
formula truth, prove subject admissibility, infer an existential/type evidence
fact, or create an obligation. No accepted attribute, type fact, cluster fact,
axiom, theorem, proof status, CoreIr, ControlFlowIr, VcId, or VC is published.

Parameterized/prefixed attributes, formula cases/otherwise, formula constants,
composite/quantified bodies, redefinitions and coherence, qualifiers,
inherited attributes, negative use, attribute applications, and cluster
semantics remain deferred. Extending those families requires separate
canonical authority, lower-owner contracts, tests, and commits.

## Typed And Final Ownership

`TypedAst` adds one optional Task-261 field and one one-shot installer:

```rust
pub fn with_source_attribute_definition(
    self,
    handoff: SourceAttributeDefinitionHandoff,
) -> Result<Self, TypedAstError>;

pub const fn source_attribute_definition(
    &self,
) -> Option<&SourceAttributeDefinitionHandoff>;

TypedAstError::InvalidSourceAttributeDefinition
```

The installer authenticates all four lower fingerprints and the typed arena,
does not read or compare the obligation table, rejects prior Task-261
occupancy, and publishes only after complete validation. Its structural update
retains the existing obligation table unchanged. `TypedAstParts` gains no
Task-261 field or alternate install path.

`ResolvedTypedAst::assemble` obtains Task 261 only from the typed owner,
clone-preserves and revalidates it, and adds only:

```rust
pub const fn source_attribute_definition(
    &self,
) -> Option<&SourceAttributeDefinitionHandoff>;

ResolvedTypedAstError::InvalidSourceAttributeDefinition
```

`ResolvedTypedAstInputs` gains no replaceable Task-261 field. Debug text starts
with `source-attribute-definition-debug-v1` and appears exactly once in typed
and final rendering. Empty legacy rendering stays byte-identical.

Task 261 is mutually isolated from current exact Task-259/260 transactions.
Its installer/final validation rejects an AST carrying either other definition
handoff, without editing Task-259/260 validation or promising install order.
A future mixed-definition contract must separately freeze same-source lower
ownership and any obligation ordering.

## Dedicated Consumer And Trace Intent

Implementation adds exactly one new active pass pair:

- `tests/miz/pass/types/pass_type_elaboration_attribute_definition_payload_001.miz`;
- `tests/miz/pass/types/pass_type_elaboration_attribute_definition_payload_001.expect.toml`.

The sidecar is `pass` / `type_elaboration` / `type_check`, has empty public
diagnostics and payloads, and cites only future requirement
`spec.en.checker.type_elaboration.source_attribute_definition_payload`. One
covered trace row backlinks only this sidecar. Passing credits exact source,
resolver, lower, and four-table transport only.

The private runner route matches only the complete 116-byte/45-row/resolver/
lower profile, before the generic attribute-definition extraction gap. Outcome,
stage, tags, diagnostics, payload, or filename cannot select it. The existing
one-parameter `thesis` gap, parser pass/recovery cases, Task-259/260 routes,
mixed-definition gap, and all other cases remain unchanged.

Implementation projects checker/runner library counts `444 -> 449` and
`516 -> 520`; active type cases `200 -> 201`; plan cases/requirements
`423/391 -> 424/392`; pass/fail `230/193 -> 231/193`; and type requirement
coverage `255/243 -> 256/244`. Declaration, parse, and proof active counts stay
`7/101/1`; warnings/errors stay `23/0`. These are projections, not changes in
the documentation prerequisite.

## Frozen Tests

The checker owns exactly five focused tests:

1. `source_attribute_definition_builds_exact_handoff_and_preserves_obligations`;
2. `source_attribute_definition_rejects_input_and_resolver_corruption`;
3. `source_attribute_definition_rejects_lower_dependency_and_fingerprint_corruption`;
4. `source_attribute_definition_installs_atomically_and_isolates_other_definition_families`;
5. `source_attribute_definition_finalizes_deterministically_without_semantic_publication`.

They cover every field/accessor/table/debug byte, independent input and
resolver corruption, every lower ID/owner/fingerprint, the strict obligation
no-read/no-write postcondition over a non-empty baseline, rollback/one-shot
ownership, Task-259/260 isolation, final clone and replay validation, and
empty proof/fact/acceptance/IR/VC outputs.

The runner owns exactly four focused tests:

1. `type_elaboration_runner_transports_exact_source_attribute_definition_payload`;
2. `source_attribute_definition_route_rejects_source_resolver_and_lower_corruption`;
3. `source_attribute_definition_route_selection_is_source_only_and_trace_is_reciprocal`;
4. `source_attribute_definition_route_publishes_no_semantic_outputs`.

They use a literal independent 45-row oracle, mutate loaded bytes/final LF,
every row family/range/recovery/child/order, excluded label/pattern subtrees,
resolver environment/projection/symbol/definition/contribution, and every
lower association. Selection and non-publication tests cover the sole trace
backlink, exact count consumers, old-gap isolation, and all semantic absences.

## Write Scope, Audit Impact, And Exit Criteria

The future implementation may change only the new checker producer/support;
checker `lib.rs`, typed/final owners and serializers, lint policy; one private
runner production/test leaf plus bounded facades; the one new fixture/sidecar/
trace row and mechanical active-count assertions; and synchronized derived
EN/JA plan/todo/audit records.

Parser, resolver, Cargo metadata, canonical `doc/spec`, every existing `.miz`,
every existing expectation/sidecar, Tasks 248/249/252/256 producers, Tasks
259/260 behavior, and unrelated semantics are forbidden. The documentation
prerequisite changes none of production, fixture, sidecar, expectation, trace
count/status, test count, CLI output, or recorded hashes.

`source_spec_audit.md`, module-boundary audits, mizar-test traceability, and
`spec_coverage_audit.md` record this frozen contract now and active partial
coverage only after implementation. Chapter 6 remains partial after Task 261;
the task credits one exact ordinary-definition transport, not general
attribute semantics.

The documentation prerequisite exits only after synchronized EN/JA, repeated
review-only **NO FINDINGS**, unchanged executable counts/hashes, all nine hard
gates PASS, quality at least 90/100 with no applicable cap, exact task-only
staging, one documentation commit, clean post-commit inventory, and automatic
return to Task 261 implementation. The implementation has the same gates and
one separate logical-task commit.
