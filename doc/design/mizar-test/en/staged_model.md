# Module: staged_model

> Canonical language: English. Japanese companion: [../ja/staged_model.md](../ja/staged_model.md).

## Purpose

This module defines the staged model for adding Mizar Evo tests.

The model prevents test cases from depending on language features that have not
yet been pinned by lower-level tests. It uses pipeline maturity as the primary
ordering axis and `doc/spec/` chapters as the coverage axis.

## Design Decision

Test addition order is governed by compiler pipeline dependencies, not only by
specification chapter order.

`doc/spec/` owns what must eventually be covered. The pipeline owns when a test
can be added without accidentally testing an earlier unresolved feature.

```text
Primary ordering:  pipeline maturity / dependency layer
Coverage mapping:  doc/spec chapter and section
```

## Stages

| Stage | Stage Id | Fixture Style | Primary Pipeline Boundary | Spec Coverage |
|---|---|---|---|---|
| 1. Lexical | `lexical` | token fixtures, minimal source snippets | lexer | `02.lexical_structure` |
| 2. Parse-only | `parse_only` | `.miz` snippets checked only through parsing | parser | syntax portions of each chapter |
| 3. Declaration / symbol | `declaration_symbol` | declarations plus resolution expectations | symbol collection and name resolution | structs, attributes, modes, predicates, functors, modules |
| 4. Type / elaboration | `type_elaboration` | typed declarations and expressions | type checking and elaboration | type system, attributes, modes, terms, formulas |
| 5. Formula / statement | `formula_statement` | formulas and statements with resolved symbols | typed AST and statement checking | terms, formulas, statements |
| 6. Proof / verification | `proof_verification` | theorem/proof fixtures | VC generation and verification | theorems, proofs, algorithms |
| 7. Advanced semantics | `advanced_semantics` | focused integration and negative tests | clusters, overload, templates, substitution, ATP, certificates, kernel | advanced semantic chapters and guardrails |

`Stage Id` is the canonical value used in `.expect.toml`,
`tests/coverage/spec_trace.toml`, reports, and Rust enums. Display names may be
localized; stage ids must not be localized.

## Public Enum Forward Compatibility

Task 12 applies the `mizar-frontend` task-25 procedure to `Stage`. `Stage` is
shared by sidecars, trace manifests, reports, and downstream runner consumers,
so it must remain `#[non_exhaustive]`; downstream callers must keep wildcard
match arms. Crate-internal matches may stay exhaustive for the currently known
stage ids.

| Public enum | Decision |
|---|---|
| `Stage` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module.

## Stage Rules

### 1. Lexical

Lexical tests must not require parsing, name resolution, type checking, or
library symbols.

They cover:

- reserved words and identifiers;
- comments and annotations as tokens;
- symbolic characters and punctuation;
- malformed lexical input.

### 2. Parse-only

Parse-only tests assert that source shapes are accepted or rejected by the
parser. They must not claim semantic validity.

They cover:

- block structure;
- declaration forms;
- theorem, proof, and statement syntax;
- recovery from malformed syntax.

### 3. Declaration / Symbol

Declaration and symbol tests introduce the smallest valid declarations needed
for later stages. Undefined symbol failures belong here unless a later-stage
test explicitly targets resolver behavior.

The active runner for this stage is the `mizar-test declaration-symbol`
subcommand. It executes only `.miz` pass/fail expectations tagged
`active_declaration_symbol`; untagged declaration-symbol sidecars stay
traceability metadata until their owning resolver behavior is executable. While
public resolver diagnostic codes remain a specification gap, fail cases assert
crate-local internal detail keys rather than user-facing codes, and non-empty
`diagnostic_codes` are rejected by the active gate.

They cover:

- symbol registration;
- duplicate or conflicting declarations;
- visibility and qualification;
- undefined name diagnostics.

### 4. Type / Elaboration

Type and elaboration tests may use only built-ins and symbols already covered
by lower stages. They must not depend on proof search, overload ambiguity,
cluster saturation, or kernel evidence unless those are the explicit subject of
the test.

They cover:

- built-in radix types;
- mode and attribute use;
- type argument checking;
- term and formula elaboration.

### 5. Formula / Statement

Formula and statement tests check source forms after token, parse, name, and
type prerequisites are established.

They cover:

- equality and predicate application;
- quantifiers and binders;
- assumptions, labels, and local statements;
- statement-level failure classification.

### 6. Proof / Verification

Proof and verification tests may rely on earlier syntactic and semantic layers
being stable. They check proof boundaries, VC generation, and verifier
outcomes.

They cover:

- valid trivial proofs;
- proof reference resolution;
- failed proof obligations;
- deterministic verification diagnostics.

### 7. Advanced Semantics

Advanced tests are added only after all lower prerequisites have dedicated
coverage. They are fail-heavy and must state the precise expected failure
boundary.

They cover:

- cluster expansion and cycle detection;
- overload resolution and template inference;
- substitution and binder normalization;
- ATP interface behavior;
- certificate and kernel rejection;
- soundness regressions.

## Spec Mapping

Every committed corpus test records, either in sidecar metadata or a corpus
manifest, the specification section it covers.

The mapping is many-to-many:

- one spec section can require tests at multiple stages;
- one integration test can cover several spec sections;
- coverage credit is assigned only to stages whose prerequisites are already
  satisfied.

For example, a cluster-cycle test maps to `17.clusters_and_registrations`, but
it is not added at stage 2 merely because the syntax appears in that chapter.
It belongs to stage 7 unless it is explicitly a parse-only fixture.

Prerequisite enforcement is metadata-driven. A requirement that needs explicit
lower-stage credit lists lower-stage requirement ids in `depends_on`; a
requirement supplied by compiler built-ins can be declared with `built_in =
true`. The harness validates those declarations and withholds executable
coverage credit when they are not satisfied. It does not infer semantic
prerequisites from source text or specification prose.

## Admission Checklist

A `.miz` test can enter the committed corpus only when:

- its intended stage is declared;
- its referenced `doc/spec/` section is known;
- all lower-stage prerequisites are already covered or listed as built-ins;
- undefined library symbols are not used unless name resolution is the target;
- the expected phase is the earliest sound rejection point;
- fail expectations use sidecars and stable failure identities;
- the test is minimal for the behavior it claims to check.

## Growth Policy

The corpus grows forward through the staged model. When a higher-stage test is
desired before lower prerequisites exist, it is kept out of the committed
corpus or marked as draft material outside default discovery.

The default fast corpus favors a small number of trustworthy tests over broad
but ambiguous examples.
